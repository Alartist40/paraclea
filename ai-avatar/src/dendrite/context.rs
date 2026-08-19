//! Dynamic context assembly from the DENDRITE knowledge graph.
//!
//! Builds a system prompt by scoring nodes for relevance to the user's query
//! and including their 1-hop neighborhoods.

use crate::dendrite::{Dendrite, Node, NodeType};
use crate::memory::estimate_tokens;
use crate::memory::now_timestamp;
use std::collections::{HashMap, HashSet};

/// Assembles a prompt context from the knowledge graph.
pub struct DendriteContext<'a> {
    dendrite: &'a Dendrite,
}

impl<'a> DendriteContext<'a> {
    pub fn new(dendrite: &'a Dendrite) -> Self {
        Self { dendrite }
    }

    /// Build a system prompt string from relevant graph nodes.
    ///
    /// Budget is roughly estimated in tokens (naive chars/4 heuristic).
    pub fn build_prompt(&self, user_message: &str, max_tokens: usize) -> String {
        let mut selected: Vec<(&Node, f32)> = Vec::new();
        let mut seen = HashSet::new();

        // ── 1. Core identity nodes (always included, 40% budget) ───────────
        let core_budget = (max_tokens as f32 * 0.4) as usize;
        let core_ids = ["identity", "soul", "agents", "tools"];
        for id in &core_ids {
            if let Some(node) = self.dendrite.get(id) {
                if seen.insert(id.to_string()) {
                    selected.push((node, 1000.0)); // force high score
                }
            }
        }

        // ── 2. Relevance search ────────────────────────────────────────────
        let mut matches: HashMap<String, f32> = HashMap::new();

        // FTS5 primary search
        if let Ok(fts_ids) = self.dendrite.fts_search(user_message) {
            for id in fts_ids {
                *matches.entry(id).or_insert(0.0) += 10.0;
            }
        }

        // Substring search on full query
        for node in self.dendrite.all_nodes() {
            let score = self.score_node(node, user_message);
            if score > 0.0 {
                *matches.entry(node.id.clone()).or_insert(0.0) += score;
            }
        }

        // Word-by-word search (ignore stop words and short words)
        let stop_words: HashSet<&str> = ["the", "a", "an", "is", "are", "was", "were",
            "be", "been", "being", "have", "has", "had", "do", "does", "did", "will",
            "would", "could", "should", "may", "might", "must", "shall", "can", "need",
            "dare", "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
            "from", "as", "into", "through", "during", "before", "after", "above",
            "below", "between", "under", "and", "but", "or", "yet", "so", "if",
            "because", "although", "though", "while", "where", "when", "that",
            "which", "who", "whom", "whose", "what", "this", "these", "those",
            "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us",
            "them", "my", "your", "his", "its", "our", "their"].iter().cloned().collect();

        for word in user_message.to_lowercase().split_whitespace() {
            let w = word.trim_matches(|c: char| !c.is_alphanumeric());
            if w.len() < 3 || stop_words.contains(w) {
                continue;
            }
            for node in self.dendrite.search(w) {
                *matches.entry(node.id.clone()).or_insert(0.0) += 3.0;
            }
        }

        // ── 3. Neighborhood expansion ──────────────────────────────────────
        let mut expanded = matches.clone();
        for (id, base_score) in &matches {
            if let Some(node) = self.dendrite.get(id) {
                for neighbor in self.dendrite.neighbors(id) {
                    let bonus = base_score * 0.3;
                    *expanded.entry(neighbor.id.clone()).or_insert(0.0) += bonus;
                }
            }
        }

        // ── 4. Score and sort ──────────────────────────────────────────────
        let now = now_timestamp();
        let mut scored: Vec<(&Node, f32)> = Vec::new();
        for (id, score) in expanded {
            if let Some(node) = self.dendrite.get(&id) {
                if seen.insert(id.clone()) {
                    let final_score = score + recency_boost(node, now) + connectivity_bonus(node) + type_bonus(node);
                    scored.push((node, final_score));
                }
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // ── 5. Assemble prompt within token budget ─────────────────────────
        let mut prompt_parts = Vec::new();
        let mut used_tokens = estimate_tokens(&selected.iter().map(|(n, _)| format_node(n)).collect::<Vec<_>>().join("\n"));

        for (node, score) in scored {
            let text = format_node(node);
            let tokens = estimate_tokens(&text);
            if used_tokens + tokens > max_tokens {
                break;
            }
            used_tokens += tokens;
            selected.push((node, score));
        }

        // Re-sort selected by type priority for nicer prompt ordering.
        selected.sort_by(|a, b| type_priority(a.0.node_type).cmp(&type_priority(b.0.node_type)));

        for (node, _) in selected {
            prompt_parts.push(format_node(node));
        }

        prompt_parts.join("\n\n")
    }

    fn score_node(&self, node: &Node, query: &str) -> f32 {
        let q = query.to_lowercase();
        let mut score = 0.0f32;

        if node.title.to_lowercase().contains(&q) {
            score += 15.0;
        }
        let content_lower = node.content.to_lowercase();
        if content_lower.contains(&q) {
            let occurrences = content_lower.matches(&q).count() as f32;
            score += 2.0 * occurrences;
        }
        for tag in &node.tags {
            if tag.contains(&q) {
                score += 5.0;
            }
        }
        score
    }
}

fn format_node(node: &Node) -> String {
    let mut lines = vec![
        format!("## {} ({})", node.title, node.node_type),
    ];
    if !node.tags.is_empty() {
        lines.push(format!("Tags: #{}" , node.tags.join(" #")));
    }
    lines.push(node.content.clone());
    lines.join("\n")
}

fn recency_boost(node: &Node, now: i64) -> f32 {
    let age_days = (now - node.updated_at).max(0) as f32 / 86400.0;
    // Linear decay over 7 days.
    (1.0 - (age_days / 7.0).min(1.0)) * 5.0
}

fn connectivity_bonus(node: &Node) -> f32 {
    let count = node.links.len() + node.backlinks.len();
    count as f32 * 0.3
}

fn type_bonus(node: &Node) -> f32 {
    match node.node_type {
        NodeType::Identity => 10.0,
        NodeType::Person => 5.0,
        NodeType::Project => 3.0,
        _ => 0.0,
    }
}

fn type_priority(t: NodeType) -> u8 {
    match t {
        NodeType::Identity => 0,
        NodeType::Person => 1,
        NodeType::Project => 2,
        NodeType::Concept => 3,
        NodeType::Event => 4,
        NodeType::Memory => 5,
        NodeType::Custom => 6,
    }
}
