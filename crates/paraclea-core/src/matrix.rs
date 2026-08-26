//! Comparative Topic Matrix Engine for Paraclea
//!
//! Synthesizes Scripture, Spirit of Prophecy (EGW), and Practical Survival/Medical
//! insights into a unified comparative matrix table.

use serde::{Deserialize, Serialize};
use crate::bible::BibleReader;
use crate::library::LibraryEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixEntry {
    pub category: String,
    pub source: String,
    pub reference: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixResult {
    pub topic: String,
    pub scripture_matches: Vec<MatrixEntry>,
    pub egw_matches: Vec<MatrixEntry>,
    pub survival_matches: Vec<MatrixEntry>,
    pub formatted_markdown: String,
}

pub struct TopicMatrixEngine;

impl TopicMatrixEngine {
    /// Generate a Comparative Matrix across Scripture, EGW, and Survival databases for a topic.
    pub fn build_matrix(topic: &str, bibles: &BibleReader, library: &LibraryEngine) -> MatrixResult {
        let topic_lower = topic.to_lowercase();
        let topic_words: Vec<&str> = topic_lower.split_whitespace().collect();

        // 1. Search Scripture
        let mut scripture_matches = Vec::new();
        let bible_results = bibles.search_keyword(&topic_lower, 5);
        for verse in bible_results {
            scripture_matches.push(MatrixEntry {
                category: "Scripture".to_string(),
                source: verse.version.clone(),
                reference: format!("{} {}:{}", verse.book, verse.chapter, verse.verse),
                excerpt: verse.text.clone(),
            });
        }

        // 2. Search EGW & Library Books
        let mut egw_matches = Vec::new();
        let mut survival_matches = Vec::new();

        for book in &library.books {
            let cat = book.category.to_lowercase();
            for ch in &book.chapters {
                let content_lower = ch.content.to_lowercase();
                // Match if all topic words or main topic word appears
                let matches_topic = topic_words.iter().all(|w| content_lower.contains(w));
                if matches_topic {
                    // Extract matching snippet
                    let snippet = Self::extract_snippet(&ch.content, &topic_words);
                    let entry = MatrixEntry {
                        category: book.category.clone(),
                        source: book.title.clone(),
                        reference: format!("Chapter {}: {}", ch.chapter_number, ch.title),
                        excerpt: snippet,
                    };

                    if cat == "egw" {
                        if egw_matches.len() < 4 {
                            egw_matches.push(entry);
                        }
                    } else if cat == "survival" || cat == "medical" {
                        if survival_matches.len() < 4 {
                            survival_matches.push(entry);
                        }
                    }
                }
            }
        }

        // Format Markdown Table
        let mut md = String::new();
        md.push_str(&format!("# 📊 Comparative Matrix: Topic \"{}\"\n\n", topic));
        md.push_str("| Category | Source & Reference | Insight & Text Excerpt |\n");
        md.push_str("| :--- | :--- | :--- |\n");

        for item in &scripture_matches {
            md.push_str(&format!("| 📜 **{}** | `{}` ({}) | {} |\n", item.category, item.reference, item.source, item.excerpt.replace('\n', " ")));
        }

        for item in &egw_matches {
            md.push_str(&format!("| ✨ **EGW Prophecy** | {} - {} | {} |\n", item.source, item.reference, item.excerpt.replace('\n', " ")));
        }

        for item in &survival_matches {
            md.push_str(&format!("| 🛠️ **Practical Survival** | {} - {} | {} |\n", item.source, item.reference, item.excerpt.replace('\n', " ")));
        }

        MatrixResult {
            topic: topic.to_string(),
            scripture_matches,
            egw_matches,
            survival_matches,
            formatted_markdown: md,
        }
    }

    fn extract_snippet(text: &str, words: &[&str]) -> String {
        let text_lower = text.to_lowercase();
        let mut best_pos = 0;
        for w in words {
            if let Some(pos) = text_lower.find(w) {
                best_pos = pos;
                break;
            }
        }

        let start = best_pos.saturating_sub(60);
        let end = (best_pos + 180).min(text.len());
        let snippet = &text[start..end];

        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < text.len() { "..." } else { "" };
        format!("{}{}{}", prefix, snippet.trim(), suffix)
    }
}
