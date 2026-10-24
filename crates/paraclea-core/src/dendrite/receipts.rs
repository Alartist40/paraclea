//! Scripture & Chapter Read Receipts Registry
//!
//! Tracks Bible chapters and library passages already loaded into the active conversation.
//! Provides surgical passage slicing and emits micro-stubs on repeat turns, saving ~1,500 tokens
//! per extended study session.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A registered receipt for loaded scripture or library passage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadReceipt {
    pub receipt_id: usize,
    pub reference: String,
    pub content_hash: u64,
    pub loaded_at_turn: usize,
    pub word_count: usize,
}

/// Registry of read receipts across the active dialogue session.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReadReceiptRegistry {
    pub receipts: HashMap<String, ReadReceipt>,
    next_id: usize,
}

impl ReadReceiptRegistry {
    pub fn new() -> Self {
        Self {
            receipts: HashMap::new(),
            next_id: 1,
        }
    }

    /// Computes deterministic 64-bit content hash.
    fn hash_content(content: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in content.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// Evaluates whether a passage should be emitted in full or compressed into a read receipt stub.
    pub fn format_passage_context(&mut self, reference: &str, full_text: &str, current_turn: usize) -> (String, bool) {
        let trimmed_ref = reference.trim();
        let content_hash = Self::hash_content(full_text);

        if let Some(existing) = self.receipts.get(trimmed_ref) {
            if existing.content_hash == content_hash {
                // Receipt hit: return micro-stub
                let stub = format!(
                    "[Scripture/Passage unchanged since read receipt #{}: {}]",
                    existing.receipt_id, existing.reference
                );
                return (stub, true);
            }
        }

        // New or modified passage: register receipt and return sliced text
        let word_count = full_text.split_whitespace().count();
        let receipt = ReadReceipt {
            receipt_id: self.next_id,
            reference: trimmed_ref.to_string(),
            content_hash,
            loaded_at_turn: current_turn,
            word_count,
        };
        self.next_id += 1;
        self.receipts.insert(trimmed_ref.to_string(), receipt.clone());

        let header = format!("[paraclea-read #{}: {} | words={}]\n{}", receipt.receipt_id, receipt.reference, receipt.word_count, full_text);
        (header, false)
    }

    /// Surgical passage slicer: slices target verse ± 1, or bounds library text to ~120 words.
    pub fn slice_passage(text: &str, target_verse: Option<usize>, max_words: usize) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() <= max_words && target_verse.is_none() {
            return text.to_string();
        }

        if let Some(verse_num) = target_verse {
            // Verse window slicer: look for markers like "16 ", "17 " or "[16]"
            let lines: Vec<&str> = text.lines().collect();
            let mut matched_indices = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                let v_str = format!("{}", verse_num);
                let v_bracket = format!("[{}]", verse_num);
                if trimmed.starts_with(&v_str) || trimmed.starts_with(&v_bracket) || trimmed.contains(&v_bracket) {
                    matched_indices.push(i);
                }
            }

            if let Some(&first_match) = matched_indices.first() {
                let start = first_match.saturating_sub(1);
                let end = (first_match + 2).min(lines.len());
                return lines[start..end].join("\n");
            }
        }

        // Default word-bounded slice
        words[..max_words.min(words.len())].join(" ")
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_read_receipt_lifecycle() {
        let mut registry = ReadReceiptRegistry::new();
        let reference = "John 3:16";
        let passage = "For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life.";

        // Turn 1: Initial load
        let (t1_text, was_stub1) = registry.format_passage_context(reference, passage, 1);
        assert!(!was_stub1);
        assert!(t1_text.contains("[paraclea-read #1: John 3:16"));

        // Turn 2: Repeat load - must be a micro-stub
        let (t2_text, was_stub2) = registry.format_passage_context(reference, passage, 2);
        assert!(was_stub2);
        assert_eq!(t2_text, "[Scripture/Passage unchanged since read receipt #1: John 3:16]");
    }

    #[test]
    fn test_surgical_passage_slice() {
        let passage = "15 That whosoever believeth in him should not perish.\n16 For God so loved the world.\n17 For God sent not his Son into the world to condemn.";
        let sliced = ReadReceiptRegistry::slice_passage(passage, Some(16), 120);
        assert!(sliced.contains("16 For God so loved the world."));
        assert!(sliced.contains("15 That whosoever"));
        assert!(sliced.contains("17 For God sent not"));
    }
}
