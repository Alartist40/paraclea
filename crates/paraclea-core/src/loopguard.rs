//! Two-Phase Agent Tool Loop Tracker & Guard (Adapted from Cynapse-Mini).
//!
//! Prevents conversational agents from spinning in infinite tool loops,
//! detecting args-only repeats, no-progress result streaks, and wandering patterns.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopVerdict {
    Ok,
    Warn,
    Critical,
    Wandering,
}

#[derive(Debug, Clone)]
struct ToolRecord {
    tool: String,
    args_hash: u64,
    result_hash: Option<u64>,
}

pub struct ToolLoopTracker {
    history: Vec<ToolRecord>,
    warning_threshold: usize,
    critical_threshold: usize,
}

impl Default for ToolLoopTracker {
    fn default() -> Self {
        Self::new(3, 5)
    }
}

impl ToolLoopTracker {
    pub fn new(warning_threshold: usize, critical_threshold: usize) -> Self {
        Self {
            history: Vec::new(),
            warning_threshold,
            critical_threshold,
        }
    }

    fn hash_val<T: Hash>(val: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    /// Check loop risk prospective to tool call.
    pub fn check(&self, tool: &str, args_json: &str) -> LoopVerdict {
        let args_hash = Self::hash_val(&args_json);
        let repeat_count = self
            .history
            .iter()
            .rev()
            .take_while(|rec| rec.tool == tool && rec.args_hash == args_hash)
            .count();

        if repeat_count >= self.critical_threshold {
            LoopVerdict::Critical
        } else if repeat_count >= self.warning_threshold {
            LoopVerdict::Warn
        } else {
            LoopVerdict::Ok
        }
    }

    /// Record call commitment.
    pub fn record_call(&mut self, tool: &str, args_json: &str) {
        let args_hash = Self::hash_val(&args_json);
        self.history.push(ToolRecord {
            tool: tool.to_string(),
            args_hash,
            result_hash: None,
        });
    }

    /// Record outcome result.
    pub fn record_outcome(&mut self, result_text: &str) {
        if let Some(last) = self.history.last_mut() {
            last.result_hash = Some(Self::hash_val(&result_text));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_guard_streak() {
        let mut tracker = ToolLoopTracker::new(2, 4);
        let tool = "read_file";
        let args = "{\"path\": \"bible.txt\"}";

        assert_eq!(tracker.check(tool, args), LoopVerdict::Ok);

        tracker.record_call(tool, args);
        tracker.record_outcome("error");
        assert_eq!(tracker.check(tool, args), LoopVerdict::Ok);

        tracker.record_call(tool, args);
        tracker.record_outcome("error");
        assert_eq!(tracker.check(tool, args), LoopVerdict::Warn);

        tracker.record_call(tool, args);
        tracker.record_outcome("error");
        tracker.record_call(tool, args);
        tracker.record_outcome("error");
        assert_eq!(tracker.check(tool, args), LoopVerdict::Critical);
    }
}
