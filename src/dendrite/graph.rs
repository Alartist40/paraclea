//! DENDRITE — In-memory Knowledge Graph Subsystem for Paraclea.
//!
//! Faithful port of Cynapse Dendrite graph stack. Nodes carry
//! wiki-links (`[[target]]`) and hashtags (`#tag`) that are parsed from
//! content; backlinks are auto-wired and kept in sync on every mutation.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn link_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap())
}

fn tag_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#([A-Za-z0-9_-]+)").unwrap())
}

/// Classifies what kind of knowledge a node holds across the 4-tier memory model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Core self / agent persona (L3)
    Identity,
    /// A real person (user, contact) (L1 / L3)
    Person,
    /// Abstract concept or topic (L2)
    Concept,
    /// A project or task (L2)
    Project,
    /// Procedural skill or workflow (L2)
    Procedure,
    /// Something that happened / event (L1)
    Event,
    /// Atomic fact or preference (L1)
    AtomicFact,
    /// Raw chat turn log (L0)
    TurnLog,
    /// Episodic memory entry (L1)
    Memory,
    /// User-defined
    Custom,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Identity => "identity",
            NodeType::Person => "person",
            NodeType::Concept => "concept",
            NodeType::Project => "project",
            NodeType::Procedure => "procedure",
            NodeType::Event => "event",
            NodeType::AtomicFact => "atomic_fact",
            NodeType::TurnLog => "turn_log",
            NodeType::Memory => "memory",
            NodeType::Custom => "custom",
        }
    }

    pub fn label(&self) -> &'static str {
        self.as_str()
    }

    pub fn tier(&self) -> u8 {
        match self {
            NodeType::TurnLog => 0,
            NodeType::AtomicFact | NodeType::Memory | NodeType::Event | NodeType::Person => 1,
            NodeType::Procedure | NodeType::Project | NodeType::Concept => 2,
            NodeType::Identity => 3,
            NodeType::Custom => 1,
        }
    }

    pub fn from_str(s: &str) -> NodeType {
        match s {
            "identity" => NodeType::Identity,
            "person" => NodeType::Person,
            "concept" => NodeType::Concept,
            "project" => NodeType::Project,
            "procedure" => NodeType::Procedure,
            "event" => NodeType::Event,
            "atomic_fact" => NodeType::AtomicFact,
            "turn_log" => NodeType::TurnLog,
            "memory" => NodeType::Memory,
            _ => NodeType::Custom,
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single knowledge node in the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub content: String,
    pub node_type: NodeType,
    pub tags: Vec<String>,
    /// Outgoing [[links]]
    pub links: Vec<String>,
    /// Auto-maintained incoming links
    pub backlinks: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Node {
    fn placeholder(id: String, now: i64) -> Node {
        Node {
            title: id.clone(),
            id,
            content: String::new(),
            node_type: NodeType::Custom,
            tags: Vec::new(),
            links: Vec::new(),
            backlinks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Default)]
struct DendriteInner {
    nodes: HashMap<String, Node>,
}

/// The in-memory graph. Thread-safe: all mutations take an internal lock,
/// and `on_change` callbacks are invoked after the lock is released.
pub struct Dendrite {
    inner: Mutex<DendriteInner>,
    on_change: Mutex<Vec<std::sync::Arc<dyn Fn() + Send + Sync>>>,
}

fn lock_inner(inner: &Mutex<DendriteInner>) -> MutexGuard<'_, DendriteInner> {
    inner.lock().unwrap_or_else(|e| e.into_inner())
}

impl Default for Dendrite {
    fn default() -> Self {
        Self::new()
    }
}

impl Dendrite {
    pub fn new() -> Dendrite {
        Dendrite {
            inner: Mutex::new(DendriteInner::default()),
            on_change: Mutex::new(Vec::new()),
        }
    }

    /// Register a callback invoked on every mutation.
    pub fn register_on_change(&self, cb: std::sync::Arc<dyn Fn() + Send + Sync>) {
        if let Ok(mut cbs) = self.on_change.lock() {
            cbs.push(cb);
        }
    }

    fn notify(&self) {
        let callbacks = self
            .on_change
            .lock()
            .map(|cbs| cbs.clone())
            .unwrap_or_default();
        for cb in callbacks {
            cb();
        }
    }

    /// Insert a hydrated node directly (used by the store on load). Skips
    /// backlink recalculation because backlinks are already stored.
    pub fn insert_hydrated(&self, node: Node) {
        lock_inner(&self.inner).nodes.insert(node.id.clone(), node);
    }

    /// Create or fully replace a node, re-wiring all backlinks.
    pub fn upsert(
        &self,
        id: &str,
        title: &str,
        content: &str,
        node_type: NodeType,
        tags: Option<Vec<String>>,
    ) -> Node {
        let now = timestamp();
        let links = parse_links(content);
        let tags = tags.unwrap_or_else(|| parse_tags(content));

        let mut inner = lock_inner(&self.inner);

        // Remove old backlinks from the previous version of this node.
        let old_links: Vec<String> = inner
            .nodes
            .get(id)
            .map(|old| old.links.clone())
            .unwrap_or_default();
        for old_link in &old_links {
            if let Some(target) = inner.nodes.get_mut(old_link) {
                target.backlinks = remove_str(&target.backlinks, id);
            }
        }

        let node = match inner.nodes.entry(id.to_string()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                let n = e.get_mut();
                n.title = title.to_string();
                n.content = content.to_string();
                n.node_type = node_type;
                n.tags = tags;
                n.links = links.clone();
                n.updated_at = now;
                n.clone()
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let n = Node {
                    id: id.to_string(),
                    title: title.to_string(),
                    content: content.to_string(),
                    node_type,
                    tags,
                    links: links.clone(),
                    backlinks: Vec::new(),
                    created_at: now,
                    updated_at: now,
                };
                e.insert(n.clone());
                n
            }
        };

        // Wire new backlinks.
        for link in &links {
            if !inner.nodes.contains_key(link) {
                inner
                    .nodes
                    .insert(link.clone(), Node::placeholder(link.clone(), now));
            }
            if let Some(target) = inner.nodes.get_mut(link) {
                if !target.backlinks.contains(&node.id) {
                    target.backlinks.push(node.id.clone());
                }
            }
        }

        drop(inner);
        self.notify();
        node
    }

    /// Delete a node and clean up all references.
    pub fn delete(&self, id: &str) -> bool {
        let mut inner = lock_inner(&self.inner);
        let node = match inner.nodes.get(id) {
            Some(n) => n.clone(),
            None => return false,
        };

        for link in &node.links {
            if let Some(target) = inner.nodes.get_mut(link) {
                target.backlinks = remove_str(&target.backlinks, id);
            }
        }
        for n in inner.nodes.values_mut() {
            n.links = remove_str(&n.links, id);
        }

        inner.nodes.remove(id);
        drop(inner);
        self.notify();
        true
    }

    pub fn get(&self, id: &str) -> Option<Node> {
        lock_inner(&self.inner).nodes.get(id).cloned()
    }

    /// All nodes sorted by UpdatedAt descending.
    pub fn all(&self) -> Vec<Node> {
        let inner = lock_inner(&self.inner);
        let mut nodes: Vec<Node> = inner.nodes.values().cloned().collect();
        nodes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        nodes
    }

    /// Return all nodes belonging to a specific memory tier (0..3).
    pub fn by_tier(&self, tier: u8) -> Vec<Node> {
        let inner = lock_inner(&self.inner);
        let mut nodes: Vec<Node> = inner
            .nodes
            .values()
            .filter(|n| n.node_type.tier() == tier)
            .cloned()
            .collect();
        nodes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        nodes
    }

    /// Fast in-memory BM25 term relevance search.
    pub fn search_bm25(&self, query: &str, limit: usize) -> Vec<(Node, f32)> {
        let inner = lock_inner(&self.inner);
        let query_tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|s| s.len() >= 2)
            .map(|s| s.to_string())
            .collect();

        if query_tokens.is_empty() {
            return Vec::new();
        }

        let total_docs = inner.nodes.len() as f32;
        if total_docs == 0.0 {
            return Vec::new();
        }

        let mut scored: Vec<(Node, f32)> = Vec::new();
        let k1 = 1.2f32;
        let b = 0.75f32;

        let avg_len = inner
            .nodes
            .values()
            .map(|n| (n.title.len() + n.content.len()) as f32)
            .sum::<f32>()
            / total_docs.max(1.0);

        for node in inner.nodes.values() {
            let doc_text = format!("{} {} {}", node.title, node.content, node.tags.join(" ")).to_lowercase();
            let doc_len = doc_text.len() as f32;
            let mut score = 0.0f32;

            for token in &query_tokens {
                let count = doc_text.matches(token).count() as f32;
                if count > 0.0 {
                    let idf = ((total_docs + 1.0) / (1.0 + count)).ln();
                    let tf = (count * (k1 + 1.0)) / (count + k1 * (1.0 - b + b * (doc_len / avg_len.max(1.0))));
                    score += idf * tf;
                }
            }

            if score > 0.0 {
                scored.push((node.clone(), score));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if limit > 0 && scored.len() > limit {
            scored.truncate(limit);
        }
        scored
    }

    /// Nodes whose title, content, or tags contain the query (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<Node> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let inner = lock_inner(&self.inner);
        inner
            .nodes
            .values()
            .filter(|n| {
                n.title.to_lowercase().contains(&q)
                    || n.content.to_lowercase().contains(&q)
                    || contains_str_fold(&n.tags, &q)
            })
            .cloned()
            .collect()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<Node> {
        let inner = lock_inner(&self.inner);
        inner
            .nodes
            .values()
            .filter(|n| contains_str_fold(&n.tags, tag))
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        lock_inner(&self.inner).nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn to_node_id(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "_")
}

pub(crate) fn parse_links(content: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut links = Vec::new();
    for caps in link_pattern().captures_iter(content) {
        if let Some(m) = caps.get(1) {
            let id = to_node_id(m.as_str());
            if seen.insert(id.clone()) {
                links.push(id);
            }
        }
    }
    links
}

pub(crate) fn parse_tags(content: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tags = Vec::new();
    for caps in tag_pattern().captures_iter(content) {
        if let Some(m) = caps.get(1) {
            let t = m.as_str().to_string();
            if seen.insert(t.clone()) {
                tags.push(t);
            }
        }
    }
    tags
}

pub(crate) fn contains_str_fold(slice: &[String], item: &str) -> bool {
    slice.iter().any(|s| s.eq_ignore_ascii_case(item))
}

pub(crate) fn remove_str(slice: &[String], item: &str) -> Vec<String> {
    slice.iter().filter(|s| *s != item).cloned().collect()
}
