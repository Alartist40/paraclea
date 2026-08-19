//! Sentiment analysis pipeline.
//!
//! Provides a trait so we can swap in `rust-bert` later without touching
//! downstream code.  The default implementation is a fast, zero-dependency
//! rule-based classifier good enough for prototyping.

use crate::emotion::EmotionState;
use anyhow::Result;

/// Trait for text → emotion analysis.
pub trait SentimentAnalyzer: Send + Sync {
    fn analyze(&self, text: &str) -> Result<EmotionState>;
}

/// A lightweight keyword-based sentiment analyzer.
///
/// Scans the input for emotion-bearing words and builds a weighted
/// `EmotionState`.  It is instant, offline, and requires no ML model.
pub struct RuleBasedAnalyzer {
    joy_words: Vec<&'static str>,
    trust_words: Vec<&'static str>,
    fear_words: Vec<&'static str>,
    surprise_words: Vec<&'static str>,
    sadness_words: Vec<&'static str>,
    disgust_words: Vec<&'static str>,
    anger_words: Vec<&'static str>,
    anticipation_words: Vec<&'static str>,
}

impl Default for RuleBasedAnalyzer {
    fn default() -> Self {
        Self {
            joy_words: vec![
                "happy", "joy", "glad", "delighted", "cheerful", "excited",
                "wonderful", "great", "fantastic", "love", "like", "enjoy",
                "smile", "laugh", "awesome", "amazing", "good", "best",
                "yay", "hooray", "pleased", "grateful", "proud", "optimistic",
            ],
            trust_words: vec![
                "trust", "believe", "confident", "sure", "safe", "secure",
                "rely", "faith", "honest", "loyal", "respect", "admire",
            ],
            fear_words: vec![
                "afraid", "scared", "fear", "worried", "nervous", "anxious",
                "terrified", "panic", "dread", "frightened", "uneasy",
            ],
            surprise_words: vec![
                "surprised", "shocked", "amazed", "astonished", "wow",
                "unexpected", "sudden", "startled", "stunned", "whoa",
            ],
            sadness_words: vec![
                "sad", "sorry", "unhappy", "depressed", "disappointed",
                "miserable", "grief", "sorrow", "cry", "tears", "hurt",
                "lonely", "heartbroken", "melancholy", "blue", "down",
            ],
            disgust_words: vec![
                "disgusting", "gross", "revolting", "sick", "nasty",
                "awful", "terrible", "hate", "dislike", "repulsive",
            ],
            anger_words: vec![
                "angry", "mad", "furious", "rage", "annoyed", "irritated",
                "frustrated", "hate", "outraged", "hostile", "bitter",
            ],
            anticipation_words: vec![
                "excited", "eager", "hopeful", "curious", "interested",
                "looking forward", "can't wait", "anticipate", "expect",
            ],
        }
    }
}

impl SentimentAnalyzer for RuleBasedAnalyzer {
    fn analyze(&self, text: &str) -> Result<EmotionState> {
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();

        let count = |list: &[&'static str]| -> f32 {
            list.iter()
                .map(|&kw| {
                    words
                        .iter()
                        .filter(|&&w| w.contains(kw))
                        .count() as f32
                })
                .sum::<f32>()
                .min(1.0)
        };

        let mut state = EmotionState {
            joy: count(&self.joy_words),
            trust: count(&self.trust_words),
            fear: count(&self.fear_words),
            surprise: count(&self.surprise_words),
            sadness: count(&self.sadness_words),
            disgust: count(&self.disgust_words),
            anger: count(&self.anger_words),
            anticipation: count(&self.anticipation_words),
            ..Default::default()
        };

        state.recompute_derived();
        Ok(state)
    }
}

/// A mock analyzer that always returns neutral emotion.
pub struct MockAnalyzer;

impl SentimentAnalyzer for MockAnalyzer {
    fn analyze(&self, _text: &str) -> Result<EmotionState> {
        Ok(EmotionState::default())
    }
}
