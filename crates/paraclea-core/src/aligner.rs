//! Acoustic Read-Along Highlighter Engine.
//!
//! Synchronizes audio playback progress with textual viewports, computing
//! active character highlight spans for real-time read-along in TUI and Web GUI.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WordSpan {
    pub word: String,
    pub start_char: usize,
    pub end_char: usize,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAudioAlignment {
    pub full_text: String,
    pub total_duration_ms: u64,
    pub spans: Vec<WordSpan>,
}

impl TextAudioAlignment {
    /// Generates approximate linear word-level timestamps when precise phoneme
    /// models are running offline without external aligner daemons.
    pub fn synthesize_linear(text: &str, duration_ms: u64) -> Self {
        let words: Vec<&str> = text.split_whitespace().collect();
        let total_words = words.len();
        if total_words == 0 {
            return Self {
                full_text: text.to_string(),
                total_duration_ms: duration_ms,
                spans: Vec::new(),
            };
        }

        let time_per_word = duration_ms / (total_words as u64);
        let mut spans = Vec::with_capacity(total_words);
        let mut search_idx = 0;

        for (i, word) in words.iter().enumerate() {
            if let Some(pos) = text[search_idx..].find(word) {
                let start_char = search_idx + pos;
                let end_char = start_char + word.len();
                let start_time_ms = (i as u64) * time_per_word;
                let end_time_ms = if i == total_words - 1 {
                    duration_ms
                } else {
                    ((i + 1) as u64) * time_per_word
                };

                spans.push(WordSpan {
                    word: word.to_string(),
                    start_char,
                    end_char,
                    start_time_ms,
                    end_time_ms,
                });
                search_idx = end_char;
            }
        }

        Self {
            full_text: text.to_string(),
            total_duration_ms: duration_ms,
            spans,
        }
    }

    /// Locate the currently active WordSpan given current playback elapsed ms.
    pub fn get_active_span(&self, elapsed_ms: u64) -> Option<&WordSpan> {
        self.spans
            .iter()
            .find(|span| elapsed_ms >= span.start_time_ms && elapsed_ms < span.end_time_ms)
            .or_else(|| self.spans.last())
    }

    /// Renders text with the active word demarcated by ANSI reverse-video or brackets.
    pub fn render_highlighted(&self, elapsed_ms: u64) -> String {
        if let Some(span) = self.get_active_span(elapsed_ms) {
            let before = &self.full_text[..span.start_char];
            let active = &self.full_text[span.start_char..span.end_char];
            let after = &self.full_text[span.end_char..];
            format!("{}[>> {} <<]{}", before, active, after)
        } else {
            self.full_text.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acoustic_alignment_and_highlight() {
        let text = "For God so loved the world";
        let align = TextAudioAlignment::synthesize_linear(text, 2400);

        assert_eq!(align.spans.len(), 6);
        assert_eq!(align.spans[0].word, "For");
        assert_eq!(align.spans[5].word, "world");

        // Mid-sentence lookup: 900ms should hit "loved" (i=2, 800..1200ms)
        let active = align.get_active_span(1300).unwrap();
        assert_eq!(active.word, "loved");

        let rendered = align.render_highlighted(1300);
        assert!(rendered.contains("[>> loved <<]"));
    }
}
