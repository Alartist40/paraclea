//! DENDRITE — In-memory Knowledge Graph Subsystem for Paraclea.
//!
//! Faithful port of Cynapse Dendrite graph stack. Nodes carry
//! wiki-links (`[[target]]`) and hashtags (`#tag`) that are parsed from
//! content; backlinks are auto-wired and kept in sync on every mutation.

use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> NodeType {
        s.parse().unwrap_or(NodeType::Custom)
    }
}

impl std::str::FromStr for NodeType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
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
        })
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single knowledge node in the graph with Engram evolutionary memory metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Canonical topic key for in-place evolutionary upserts (Engram pattern)
    #[serde(default)]
    pub topic_key: Option<String>,
    /// Number of evolutionary updates/revisions this topic has undergone
    #[serde(default = "default_revision_count")]
    pub revision_count: u32,
    /// SHA-256 fingerprint of content for rolling deduplication
    #[serde(default)]
    pub content_hash: Option<String>,
    /// Duplicate hit counter when identical content is re-encountered
    #[serde(default)]
    pub duplicate_count: u32,
    /// Timestamp of most recent observation or access
    #[serde(default)]
    pub last_seen_at: i64,
    /// Optional expiration / review timestamp for memory hygiene
    #[serde(default)]
    pub review_after: Option<i64>,
    /// State flag: "active", "needs_review", "decayed"
    #[serde(default = "default_state")]
    pub state: String,
}

fn default_revision_count() -> u32 {
    1
}

fn default_state() -> String {
    "active".to_string()
}

pub fn compute_content_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Layer 1: Compact preview card for celestial orbital nodes and quick index (~100-150 tokens)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactNodeCard {
    pub id: String,
    pub title: String,
    pub node_type: NodeType,
    pub topic_key: Option<String>,
    pub preview: String,
    pub tags: Vec<String>,
    pub revision_count: u32,
    pub updated_at: i64,
}

/// Layer 2: Cluster context showing surrounding constellation/connections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterContext {
    pub central_id: String,
    pub central_title: String,
    pub outgoing_links: Vec<String>,
    pub backlinks: Vec<String>,
    pub neighbor_titles: Vec<String>,
    pub tags: Vec<String>,
}

impl Node {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        node_type: NodeType,
    ) -> Node {
        let now = timestamp();
        let content_str = content.into();
        let hash = compute_content_hash(&content_str);
        let links = parse_links(&content_str);
        let tags = parse_tags(&content_str);

        Node {
            id: id.into(),
            title: title.into(),
            content: content_str,
            node_type,
            tags,
            links,
            backlinks: Vec::new(),
            created_at: now,
            updated_at: now,
            topic_key: None,
            revision_count: 1,
            content_hash: Some(hash),
            duplicate_count: 0,
            last_seen_at: now,
            review_after: None,
            state: "active".to_string(),
        }
    }

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
            topic_key: None,
            revision_count: 1,
            content_hash: None,
            duplicate_count: 0,
            last_seen_at: now,
            review_after: None,
            state: "active".to_string(),
        }
    }

    pub fn to_compact_card(&self, max_chars: usize) -> CompactNodeCard {
        let max_len = if max_chars == 0 { 150 } else { max_chars };
        let char_count = self.content.chars().count();
        let preview = if char_count > max_len {
            let truncated: String = self.content.chars().take(max_len).collect();
            format!("{}...", truncated)
        } else {
            self.content.clone()
        };

        CompactNodeCard {
            id: self.id.clone(),
            title: self.title.clone(),
            node_type: self.node_type,
            topic_key: self.topic_key.clone(),
            preview,
            tags: self.tags.clone(),
            revision_count: self.revision_count,
            updated_at: self.updated_at,
        }
    }

    pub fn is_stale_or_needs_review(&self, now: i64) -> bool {
        if let Some(review_after) = self.review_after {
            now >= review_after
        } else {
            false
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

        let hash = compute_content_hash(content);
        let node = match inner.nodes.entry(id.to_string()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                let n = e.get_mut();
                n.title = title.to_string();
                n.content = content.to_string();
                n.node_type = node_type;
                n.tags = tags;
                n.links = links.clone();
                n.revision_count = n.revision_count.saturating_add(1);
                n.content_hash = Some(hash);
                n.updated_at = now;
                n.last_seen_at = now;
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
                    topic_key: None,
                    revision_count: 1,
                    content_hash: Some(hash),
                    duplicate_count: 0,
                    last_seen_at: now,
                    review_after: None,
                    state: "active".to_string(),
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
        nodes.sort_by_key(|b| std::cmp::Reverse(b.updated_at));
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
        nodes.sort_by_key(|b| std::cmp::Reverse(b.updated_at));
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

        let mut df_map: HashMap<&str, usize> = HashMap::new();
        for token in &query_tokens {
            let df = inner
                .nodes
                .values()
                .filter(|n| {
                    let text = format!("{} {} {}", n.title, n.content, n.tags.join(" ")).to_lowercase();
                    text.contains(token.as_str())
                })
                .count();
            df_map.insert(token.as_str(), df);
        }

        for node in inner.nodes.values() {
            let doc_text = format!("{} {} {}", node.title, node.content, node.tags.join(" ")).to_lowercase();
            let doc_len = doc_text.len() as f32;
            let mut score = 0.0f32;

            for token in &query_tokens {
                let count = doc_text.matches(token).count() as f32;
                let df = *df_map.get(token.as_str()).unwrap_or(&0) as f32;
                if count > 0.0 {
                    let idf = (((total_docs - df + 0.5) / (df + 0.5)) + 1.0).ln().max(0.1);
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

    /// Evolutionary upsert with topic_key and rolling deduplication (Engram pattern).
    ///
    /// If a topic_key is provided and already exists:
    /// - If the content is unchanged, increments duplicate_count and updates last_seen_at.
    /// - If content changed, performs an in-place evolutionary update, incrementing revision_count.
    ///
    /// If no topic_key matches, but content_hash matches an existing node, increments duplicate_count.
    ///
    /// Otherwise, creates a new node. Returns `(resulting_node, was_mutated_or_created)`.
    pub fn upsert_by_topic(&self, mut node: Node) -> (Node, bool) {
        let now = timestamp();
        let content_hash = node
            .content_hash
            .clone()
            .unwrap_or_else(|| compute_content_hash(&node.content));
        node.content_hash = Some(content_hash.clone());

        let mut inner = lock_inner(&self.inner);

        // 1. Check existing node by topic_key if provided
        if let Some(ref tk) = node.topic_key {
            let existing_id = inner
                .nodes
                .values()
                .find(|n| n.topic_key.as_deref() == Some(tk.as_str()))
                .map(|n| n.id.clone());

            if let Some(eid) = existing_id {
                let (is_dup, old_links) = {
                    let existing = inner.nodes.get_mut(&eid).unwrap();
                    if existing.content == node.content || existing.content_hash.as_deref() == Some(&content_hash) {
                        existing.duplicate_count = existing.duplicate_count.saturating_add(1);
                        existing.last_seen_at = now;
                        (true, Vec::new())
                    } else {
                        (false, existing.links.clone())
                    }
                };

                if is_dup {
                    let ret = inner.nodes.get(&eid).cloned().unwrap();
                    drop(inner);
                    return (ret, false);
                }

                // In-place revision: remove old backlinks
                for old_link in &old_links {
                    if let Some(target) = inner.nodes.get_mut(old_link) {
                        target.backlinks = remove_str(&target.backlinks, &eid);
                    }
                }

                let new_links = parse_links(&node.content);
                let new_tags = if node.tags.is_empty() {
                    parse_tags(&node.content)
                } else {
                    node.tags.clone()
                };

                let updated_node = {
                    let existing = inner.nodes.get_mut(&eid).unwrap();
                    existing.title = node.title;
                    existing.content = node.content;
                    existing.node_type = node.node_type;
                    existing.tags = new_tags;
                    existing.links = new_links.clone();
                    existing.revision_count = existing.revision_count.saturating_add(1);
                    existing.updated_at = now;
                    existing.last_seen_at = now;
                    existing.content_hash = Some(content_hash);
                    existing.review_after = node.review_after;
                    existing.state = "active".to_string();
                    existing.clone()
                };

                // Wire new backlinks
                for link in &new_links {
                    if !inner.nodes.contains_key(link) {
                        inner.nodes.insert(link.clone(), Node::placeholder(link.clone(), now));
                    }
                    if let Some(target) = inner.nodes.get_mut(link) {
                        if !target.backlinks.contains(&updated_node.id) {
                            target.backlinks.push(updated_node.id.clone());
                        }
                    }
                }

                drop(inner);
                self.notify();
                return (updated_node, true);
            }
        }

        // 2. Rolling deduplication: check if identical content_hash already exists
        let dedup_id = inner
            .nodes
            .values()
            .find(|n| n.content_hash.as_deref() == Some(&content_hash) && !n.content.trim().is_empty())
            .map(|n| n.id.clone());

        if let Some(did) = dedup_id {
            let existing = inner.nodes.get_mut(&did).unwrap();
            existing.duplicate_count = existing.duplicate_count.saturating_add(1);
            existing.last_seen_at = now;
            let ret = existing.clone();
            drop(inner);
            return (ret, false);
        }

        // 3. New insertion
        let id = if node.id.is_empty() {
            to_node_id(&node.title)
        } else {
            node.id.clone()
        };
        node.id = id.clone();
        node.links = parse_links(&node.content);
        if node.tags.is_empty() {
            node.tags = parse_tags(&node.content);
        }
        node.created_at = now;
        node.updated_at = now;
        node.last_seen_at = now;
        node.revision_count = 1;
        node.duplicate_count = 0;

        // Wire backlinks
        for link in &node.links {
            if !inner.nodes.contains_key(link) {
                inner.nodes.insert(link.clone(), Node::placeholder(link.clone(), now));
            }
            if let Some(target) = inner.nodes.get_mut(link) {
                if !target.backlinks.contains(&node.id) {
                    target.backlinks.push(node.id.clone());
                }
            }
        }

        inner.nodes.insert(id, node.clone());
        drop(inner);
        self.notify();
        (node, true)
    }

    /// Find node by its canonical topic_key.
    pub fn find_by_topic_key(&self, topic_key: &str) -> Option<Node> {
        let inner = lock_inner(&self.inner);
        inner
            .nodes
            .values()
            .find(|n| n.topic_key.as_deref() == Some(topic_key))
            .cloned()
    }

    /// Find node by exact SHA-256 content hash.
    pub fn find_by_content_hash(&self, hash: &str) -> Option<Node> {
        let inner = lock_inner(&self.inner);
        inner
            .nodes
            .values()
            .find(|n| n.content_hash.as_deref() == Some(hash))
            .cloned()
    }

    /// Progressive Disclosure Layer 1: Compact orbital cards for 3D atlas or index preview.
    pub fn get_layer1_cards(&self, limit: usize) -> Vec<CompactNodeCard> {
        let all_nodes = self.all();
        let iter = all_nodes.into_iter();
        if limit > 0 {
            iter.take(limit).map(|n| n.to_compact_card(150)).collect()
        } else {
            iter.map(|n| n.to_compact_card(150)).collect()
        }
    }

    /// Progressive Disclosure Layer 2: Cluster / constellation context surrounding a target node.
    pub fn get_cluster_context(&self, id: &str) -> Option<ClusterContext> {
        let inner = lock_inner(&self.inner);
        let node = inner.nodes.get(id)?;

        let mut neighbor_titles = Vec::new();
        for link in &node.links {
            if let Some(neighbor) = inner.nodes.get(link) {
                neighbor_titles.push(neighbor.title.clone());
            }
        }
        for backlink in &node.backlinks {
            if let Some(neighbor) = inner.nodes.get(backlink) {
                if !neighbor_titles.contains(&neighbor.title) {
                    neighbor_titles.push(neighbor.title.clone());
                }
            }
        }

        Some(ClusterContext {
            central_id: node.id.clone(),
            central_title: node.title.clone(),
            outgoing_links: node.links.clone(),
            backlinks: node.backlinks.clone(),
            neighbor_titles,
            tags: node.tags.clone(),
        })
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
