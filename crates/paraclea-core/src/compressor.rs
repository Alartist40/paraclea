//! Output Compressor & Context Protection Engine (Adapted from Cynapse-Mini).
//!
//! Shrinks verbose command outputs, test runs, and multi-line scripture searches
//! into compact digests, enforces the 5-item reader rule, and spills large
//! payloads safely to local disk (`~/.paraclea/spill/`).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompressedResult {
    pub summary: String,
    pub signature: Option<String>,
    pub truncated: bool,
    pub original_len: usize,
    pub spill_path: Option<String>,
}

pub struct CompressorOptions {
    pub max_summary_length: usize,
    pub max_tail_lines: usize,
    pub spill_threshold: usize,
}

impl Default for CompressorOptions {
    fn default() -> Self {
        Self {
            max_summary_length: 1200,
            max_tail_lines: 20,
            spill_threshold: 1200,
        }
    }
}

/// Spills large raw outputs to `~/.paraclea/spill/<hash>.log` to keep context windows lean.
pub fn spill_to_disk(raw: &str) -> Option<PathBuf> {
    let spill_dir = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".paraclea").join("spill")
    } else {
        PathBuf::from(".paraclea").join("spill")
    };

    if fs::create_dir_all(&spill_dir).is_err() {
        return None;
    }

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    raw.hash(&mut hasher);
    let hash_val = hasher.finish();

    let file_path = spill_dir.join(format!("{:016x}.log", hash_val));
    if fs::write(&file_path, raw).is_ok() {
        Some(file_path)
    } else {
        None
    }
}

/// Enforces the 5-item reader limit for lists and multi-line search results.
pub fn cap_list_items(text: &str, max_items: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max_items {
        return text.to_string();
    }
    let shown = &lines[..max_items];
    let omitted = lines.len() - max_items;
    format!("{}\n[{} items shown, {} more omitted]", shown.join("\n"), max_items, omitted)
}

const ERROR_MARKERS: &[&str] = &[
    "error:",
    "error[",
    "failed:",
    "panic:",
    "fatal:",
    "exception:",
    "traceback",
    "assertionerror",
    "cannot find",
    "undefined",
    "syntax error",
];

fn extract_signature(text: &str, is_error: bool) -> Option<String> {
    if !is_error {
        return None;
    }
    let mut sigs = Vec::new();
    for line in text.lines() {
        let lower = line.to_lowercase();
        for marker in ERROR_MARKERS {
            if lower.contains(marker) {
                sigs.push(line.trim().to_string());
                break;
            }
        }
        if sigs.len() >= 3 {
            break;
        }
    }
    if sigs.is_empty() {
        None
    } else {
        Some(sigs.join(" | "))
    }
}

/// Main compression function for tool responses.
pub fn compress_tool_output(raw: &str, is_error: bool, opts: &CompressorOptions) -> CompressedResult {
    let original_len = raw.len();
    let signature = extract_signature(raw, is_error);

    if original_len <= opts.max_summary_length {
        return CompressedResult {
            summary: raw.to_string(),
            signature,
            truncated: false,
            original_len,
            spill_path: None,
        };
    }

    let spill_path = if original_len >= opts.spill_threshold {
        spill_to_disk(raw).map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };

    let lines: Vec<&str> = raw.lines().collect();
    let tail_lines = if lines.len() > opts.max_tail_lines {
        &lines[lines.len() - opts.max_tail_lines..]
    } else {
        &lines[..]
    };

    let summary = format!(
        "[Output truncated: {} bytes original]\n{}",
        original_len,
        tail_lines.join("\n")
    );

    CompressedResult {
        summary,
        signature,
        truncated: true,
        original_len,
        spill_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_list_items() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7";
        let capped = cap_list_items(text, 5);
        assert!(capped.contains("Line 5"));
        assert!(!capped.contains("Line 6"));
        assert!(capped.contains("[5 items shown, 2 more omitted]"));
    }

    #[test]
    fn test_compress_tool_output() {
        let raw = "a".repeat(2000);
        let opts = CompressorOptions::default();
        let compressed = compress_tool_output(&raw, false, &opts);
        assert!(compressed.truncated);
        assert_eq!(compressed.original_len, 2000);
        assert!(compressed.spill_path.is_some());
    }
}
