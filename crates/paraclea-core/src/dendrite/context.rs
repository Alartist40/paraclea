//! DENDRITE system-prompt assembly & context selection for Paraclea.
//!
//! Assembles graph memory nodes into LLM prompt context with relevance scoring & token budgeting.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::dendrite::graph::{Dendrite, Node, NodeType};
use crate::dendrite::store::DendriteStore;

const DEFAULT_MAX_TOKENS: usize = 6000;
/// 40% of the token budget for core identity nodes.
const CORE_NODE_BUDGET: f64 = 0.40;

/// Core identity nodes always included first.
const CORE_IDS: [&str; 4] = ["identity", "soul", "user_profile", "study_habits"];

struct ContextInner {
    graph: Arc<Dendrite>,
    store: Option<Arc<DendriteStore>>,
    cached_prompt: String,
    cached_at: Instant,
    cache_ttl: std::time::Duration,
    dirty: bool,
}

/// Assembles the LLM system prompt from graph nodes.
pub struct DendriteContext {
    inner: Arc<Mutex<ContextInner>>,
}

impl DendriteContext {
    pub fn new(graph: Arc<Dendrite>, store: Option<Arc<DendriteStore>>) -> Arc<DendriteContext> {
        let inner = Arc::new(Mutex::new(ContextInner {
            graph,
            store,
            cached_prompt: String::new(),
            cached_at: Instant::now(),
            cache_ttl: std::time::Duration::from_secs(300),
            dirty: true,
        }));

        let weak = Arc::downgrade(&inner);
        let graph_arc = inner.lock().unwrap_or_else(|e| e.into_inner()).graph.clone();
        graph_arc.register_on_change(Arc::new(move || {
            if let Some(inner) = weak.upgrade() {
                if let Ok(mut i) = inner.lock() {
                    i.dirty = true;
                }
            }
        }));

        Arc::new(DendriteContext { inner })
    }

    /// Return the system prompt. If `user_message` is non-empty it biases
    /// context toward relevant nodes; otherwise a cached general prompt.
    pub fn build_prompt(&self, user_message: &str, max_tokens: usize) -> String {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let max_tokens = if max_tokens == 0 {
            DEFAULT_MAX_TOKENS
        } else {
            max_tokens
        };

        if !user_message.trim().is_empty() {
            inner.dirty = false;
            return assemble(&inner, user_message, max_tokens);
        }

        let now = Instant::now();
        if !inner.dirty && now.duration_since(inner.cached_at) < inner.cache_ttl && !inner.cached_prompt.is_empty() {
            return inner.cached_prompt.clone();
        }

        inner.dirty = false;
        let prompt = assemble(&inner, "", max_tokens);
        inner.cached_prompt = prompt.clone();
        inner.cached_at = now;
        prompt
    }
}

fn assemble(inner: &ContextInner, user_message: &str, max_tokens: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut used: usize = 0;
    let core_budget = ((max_tokens as f64) * CORE_NODE_BUDGET) as usize;

    for id in CORE_IDS {
        let node = match inner.graph.get(id) {
            Some(n) => n,
            None => continue,
        };
        let part = format!("## {}\n\n{}", node.title, node.content);
        let cost = estimate_tokens(&part);
        if used + cost > core_budget {
            break;
        }
        parts.push(part);
        used += cost;
    }

    if !user_message.trim().is_empty() {
        let candidates = find_relevant(inner, user_message);
        let scored = score(&candidates, user_message);
        for (node, _score) in scored {
            if CORE_IDS.contains(&node.id.as_str()) {
                continue;
            }
            let part = format!("## {}\n\n{}", node.title, node.content);
            let cost = estimate_tokens(&part);
            if used + cost > max_tokens {
                break;
            }
            parts.push(part);
            used += cost;
        }
    } else {
        for node in inner.graph.all() {
            if CORE_IDS.contains(&node.id.as_str()) {
                continue;
            }
            let part = format!("## {}\n\n{}", node.title, node.content);
            let cost = estimate_tokens(&part);
            if used + cost > max_tokens {
                break;
            }
            parts.push(part);
            used += cost;
        }
    }

    parts.join("\n\n---\n\n")
}

fn find_relevant(inner: &ContextInner, user_message: &str) -> Vec<Node> {
    let mut seen = HashSet::new();
    let mut out: Vec<Node> = Vec::new();

    let add_node = |n: &Node, out: &mut Vec<Node>, seen: &mut HashSet<String>| {
        if !seen.contains(&n.id) {
            seen.insert(n.id.clone());
            out.push(n.clone());
        }
    };

    if let Some(store) = &inner.store {
        if let Ok(ids) = store.fts_search(user_message, 20) {
            for id in ids {
                if let Some(n) = inner.graph.get(&id) {
                    add_node(&n, &mut out, &mut seen);
                }
            }
        }
    }

    for n in inner.graph.search(user_message) {
        add_node(&n, &mut out, &mut seen);
    }

    for word in user_message.to_lowercase().split_whitespace() {
        if word.chars().count() < 3 || is_stop_word(word) {
            continue;
        }
        for n in inner.graph.search(word) {
            add_node(&n, &mut out, &mut seen);
        }
        for n in inner.graph.by_tag(word) {
            add_node(&n, &mut out, &mut seen);
        }
    }

    out
}

type ScoredNode = (Node, f64);

fn score(nodes: &[Node], query: &str) -> Vec<ScoredNode> {
    let q = query.to_lowercase();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut scored: Vec<ScoredNode> = nodes
        .iter()
        .map(|n| {
            let mut s = 0.0;

            if n.title.to_lowercase().contains(&q) {
                s += 15.0;
            }
            s += count_occurrences(&n.content.to_lowercase(), &q) as f64 * 2.0;

            let age = (now - n.updated_at) as f64 / 86400.0;
            if age < 7.0 {
                s += (7.0 - age) * (5.0 / 7.0);
            }

            s += (n.links.len() + n.backlinks.len()) as f64 * 0.3;

            match n.node_type {
                NodeType::Identity => s += 10.0,
                NodeType::Person => s += 5.0,
                NodeType::Project => s += 3.0,
                NodeType::Procedure => s += 4.0,
                _ => {}
            }

            (n.clone(), s)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(needle).count()
}

pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}

fn stop_words() -> &'static HashSet<&'static str> {
    static WORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    WORDS.get_or_init(|| {
        [
            "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
            "one", "our", "out", "day", "get", "has", "him", "his", "how", "its", "may", "new",
            "now", "old", "see", "two", "who", "boy", "did", "she", "use", "way", "many", "oil",
            "sit", "set", "run", "eat", "far", "sea", "eye", "ago", "off", "too", "any", "say",
            "man", "try", "ask", "end", "why", "let", "put", "own", "tell", "when", "come", "here",
            "just", "like", "long", "make", "over", "such", "take", "than", "them", "well", "were",
            "what", "will", "with", "have", "from", "they", "know", "want", "been", "good", "much",
            "some", "time", "would", "there", "their", "could", "other", "after", "first", "never",
            "these", "think", "where", "being", "every", "great", "might", "shall", "still",
            "those", "while", "about", "should",
        ]
        .into_iter()
        .collect()
    })
}

fn is_stop_word(w: &str) -> bool {
    stop_words().contains(w)
}
