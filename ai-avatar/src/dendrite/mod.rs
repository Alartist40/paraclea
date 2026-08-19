//! DENDRITE — Knowledge graph memory with wiki-links and FTS search.
//!
//! Inspired by Obsidian/Zettelkasten: nodes link via `[[wiki-links]]` syntax,
//! backlinks are auto-maintained, and relevance scoring drives context assembly.

pub mod context;
pub mod store;

use crate::memory::now_timestamp;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Types of knowledge nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Identity,
    Person,
    Concept,
    Project,
    Event,
    Memory,
    Custom,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NodeType::Identity => "identity",
            NodeType::Person => "person",
            NodeType::Concept => "concept",
            NodeType::Project => "project",
            NodeType::Event => "event",
            NodeType::Memory => "memory",
            NodeType::Custom => "custom",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for NodeType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "identity" => Ok(NodeType::Identity),
            "person" => Ok(NodeType::Person),
            "concept" => Ok(NodeType::Concept),
            "project" => Ok(NodeType::Project),
            "event" => Ok(NodeType::Event),
            "memory" => Ok(NodeType::Memory),
            _ => Ok(NodeType::Custom),
        }
    }
}

/// A node in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub content: String,
    pub node_type: NodeType,
    pub tags: Vec<String>,
    pub links: Vec<String>,      // outgoing [[wiki-links]] (node ids)
    pub backlinks: Vec<String>,  // auto-maintained incoming
    pub created_at: i64,
    pub updated_at: i64,
}

impl Node {
    /// Parse `[[wiki-links]]` from content.
    pub fn parse_links(content: &str) -> Vec<String> {
        let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        re.captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_lowercase()))
            .collect()
    }
}

/// In-memory knowledge graph with persistent backing.
pub struct Dendrite {
    nodes: HashMap<String, Node>,
    store: crate::dendrite::store::DendriteStore,
}

impl Dendrite {
    pub fn open(db_path: std::path::PathBuf) -> Result<Self> {
        let store = crate::dendrite::store::DendriteStore::open(db_path)?;
        let nodes = store.load_all()?;
        tracing::info!("Dendrite loaded {} nodes", nodes.len());
        Ok(Self { nodes, store })
    }

    /// Create or update a node. Rewires backlinks automatically.
    pub fn upsert(
        &mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        node_type: NodeType,
        tags: Vec<String>,
    ) -> Result<()> {
        let id = id.into();
        let title = title.into();
        let content = content.into();
        let links = Node::parse_links(&content);
        let now = now_timestamp();

        let node = Node {
            id: id.clone(),
            title: title.clone(),
            content: content.clone(),
            node_type,
            tags: tags.iter().map(|t| t.to_lowercase()).collect(),
            links: links.clone(),
            backlinks: Vec::new(), // rebuilt below
            created_at: self.nodes.get(&id).map(|n| n.created_at).unwrap_or(now),
            updated_at: now,
        };

        // Remove old backlinks before updating.
        let old_links: Vec<String> = self.nodes.get(&id)
            .map(|n| n.links.clone())
            .unwrap_or_default();
        for target in &old_links {
            if let Some(target_node) = self.nodes.get_mut(target) {
                target_node.backlinks.retain(|b| b != &id);
            }
        }

        // Add new backlinks.
        for target in &links {
            if let Some(target_node) = self.nodes.get_mut(target) {
                if !target_node.backlinks.contains(&id) {
                    target_node.backlinks.push(id.clone());
                }
            }
        }

        self.store.save(&node)?;
        self.nodes.insert(id, node);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn delete(&mut self, id: &str) -> Result<()> {
        if let Some(node) = self.nodes.remove(id) {
            for target in &node.links {
                if let Some(target_node) = self.nodes.get_mut(target) {
                    target_node.backlinks.retain(|b| b != id);
                }
            }
            self.store.delete(id)?;
        }
        Ok(())
    }

    /// 1-hop neighbors (linked + backlinked).
    pub fn neighbors(&self, id: &str) -> Vec<&Node> {
        let mut result = Vec::new();
        if let Some(node) = self.nodes.get(id) {
            let mut seen = HashSet::new();
            seen.insert(id.to_string());
            for link in &node.links {
                if seen.insert(link.clone()) {
                    if let Some(n) = self.nodes.get(link) {
                        result.push(n);
                    }
                }
            }
            for backlink in &node.backlinks {
                if seen.insert(backlink.clone()) {
                    if let Some(n) = self.nodes.get(backlink) {
                        result.push(n);
                    }
                }
            }
        }
        result
    }

    /// 2-hop neighborhood.
    pub fn neighbors_2hop(&self, id: &str) -> Vec<&Node> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        seen.insert(id.to_string());

        let first = self.neighbors(id);
        for n in &first {
            if seen.insert(n.id.clone()) {
                result.push(*n);
            }
        }
        for n in first {
            for nn in self.neighbors(&n.id) {
                if seen.insert(nn.id.clone()) {
                    result.push(nn);
                }
            }
        }
        result
    }

    pub fn all_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Node> {
        let q = query.to_lowercase();
        self.nodes
            .values()
            .filter(|n| {
                n.title.to_lowercase().contains(&q)
                    || n.content.to_lowercase().contains(&q)
                    || n.tags.iter().any(|t| t.contains(&q))
            })
            .collect()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&Node> {
        let t = tag.to_lowercase();
        self.nodes
            .values()
            .filter(|n| n.tags.contains(&t))
            .collect()
    }

    /// Full-text search via SQLite FTS5 (falls back to substring if FTS unavailable).
    pub fn fts_search(&self, query: &str) -> Result<Vec<String>> {
        self.store.fts_search(query)
    }
}
