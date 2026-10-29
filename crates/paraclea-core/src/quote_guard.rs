//! Scripture Hallucination Prevention & Direct Quotation Guard.
//!
//! Detects biblical citations in model responses, cross-references against verified
//! database verses, detects textual divergence, and enforces canonical verbatim fidelity.

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CitationMatch {
    pub book: String,
    pub chapter: usize,
    pub verse: usize,
    pub raw_citation: String,
    pub quoted_text_in_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub citation: CitationMatch,
    pub canonical_text: String,
    pub is_exact: bool,
    pub divergence_ratio: f32,
    pub auto_corrected_text: Option<String>,
}

pub struct QuoteGuard;

impl QuoteGuard {
    /// Extracts biblical citations like "John 3:16", "Genesis 1:1", "1 Cor 13:4" from text.
    pub fn extract_citations(text: &str) -> Vec<CitationMatch> {
        let pattern = r"\b((?:[1-3]\s+)?[A-Za-z]+)\s+([0-9]{1,3}):([0-9]{1,3})\b";
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut matches = Vec::new();
        for cap in re.captures_iter(text) {
            let book = cap[1].to_string();
            let chapter: usize = match cap[2].parse() {
                Ok(c) => c,
                Err(_) => continue,
            };
            let verse: usize = match cap[3].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let raw_citation = format!("{} {}:{}", book, chapter, verse);
            matches.push(CitationMatch {
                book,
                chapter,
                verse,
                raw_citation,
                quoted_text_in_response: None,
            });
        }
        matches
    }

    /// Calculates normalized word-overlap token divergence between 0.0 (identical) and 1.0 (completely divergent).
    pub fn calculate_divergence(quoted: &str, canonical: &str) -> f32 {
        let q_words: Vec<String> = quoted
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        let c_words: Vec<String> = canonical
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        if q_words.is_empty() || c_words.is_empty() {
            return 1.0;
        }

        let mut matches = 0;
        for w in &q_words {
            if c_words.contains(w) {
                matches += 1;
            }
        }

        let similarity = (matches as f32) / (q_words.len().max(c_words.len()) as f32);
        (1.0 - similarity).clamp(0.0, 1.0)
    }

    /// Evaluates quotation and produces verified result.
    pub fn verify_quote(
        citation: CitationMatch,
        quoted_passage: Option<&str>,
        canonical_text: &str,
    ) -> VerificationResult {
        if let Some(quoted) = quoted_passage {
            let divergence = Self::calculate_divergence(quoted, canonical_text);
            let is_exact = divergence <= 0.05;
            let auto_corrected_text = if !is_exact {
                Some(canonical_text.to_string())
            } else {
                None
            };

            VerificationResult {
                citation,
                canonical_text: canonical_text.to_string(),
                is_exact,
                divergence_ratio: divergence,
                auto_corrected_text,
            }
        } else {
            VerificationResult {
                citation,
                canonical_text: canonical_text.to_string(),
                is_exact: true,
                divergence_ratio: 0.0,
                auto_corrected_text: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_extraction() {
        let text = "As stated in John 3:16 and later in 1 Corinthians 13:4, love is patient.";
        let hits = QuoteGuard::extract_citations(text);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].raw_citation, "John 3:16");
        assert_eq!(hits[1].raw_citation, "1 Corinthians 13:4");
    }

    #[test]
    fn test_quote_divergence_calculation() {
        let canonical = "For God so loved the world, that he gave his only begotten Son";
        let exact = "For God so loved the world that he gave his only begotten Son";
        let hallucinated = "God demonstrated eternal love to humanity by sacrificing Christ";

        let exact_div = QuoteGuard::calculate_divergence(exact, canonical);
        assert!(exact_div <= 0.05);

        let hallu_div = QuoteGuard::calculate_divergence(hallucinated, canonical);
        assert!(hallu_div > 0.50);
    }
}
