//! Custom Cross-Reference & Link Manager for Paraclea
//!
//! Bridges Scripture verses and non-scripture books via Dendrite v2 Knowledge Graph nodes.

use anyhow::Result;
use std::sync::Arc;

use crate::dendrite::{Dendrite, DendriteStore, NodeType};

pub struct CrossReferenceLinker {
    graph: Arc<Dendrite>,
    store: Option<Arc<DendriteStore>>,
}

impl CrossReferenceLinker {
    pub fn new(graph: Arc<Dendrite>, store: Option<Arc<DendriteStore>>) -> Self {
        Self { graph, store }
    }

    /// Create a custom bidirectional cross-reference between Scripture and non-scripture library passages.
    pub fn create_cross_reference(
        &self,
        source: &str,
        target: &str,
        notes: &str,
    ) -> Result<String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let source_id = source.trim().to_lowercase().replace(' ', "_");
        let target_id = target.trim().to_lowercase().replace(' ', "_");
        let node_id = format!("crossref_{}_{}_{}", source_id, target_id, timestamp);

        let title = format!("Cross-Ref: {} ↔ {}", source, target);
        let content = format!(
            "Cross-Reference Association:\n- Source: [[{}]]\n- Target: [[{}]]\n\nStudy Notes:\n{}",
            source_id, target_id, notes
        );

        let tags = vec![
            "#cross_reference".to_string(),
            format!("#source_{}", source_id),
            format!("#target_{}", target_id),
        ];

        let node = self.graph.upsert(
            &node_id,
            &title,
            &content,
            NodeType::Concept,
            Some(tags),
        );

        if let Some(ref store) = self.store {
            store.save(&node)?;
        }

        Ok(node_id)
    }

    /// Search for cross-references matching a verse or topic.
    pub fn find_cross_references(&self, query: &str) -> Vec<crate::dendrite::Node> {
        let bm25_results = self.graph.search_bm25(query, 10);
        if !bm25_results.is_empty() {
            return bm25_results
                .into_iter()
                .map(|(node, _score)| node)
                .filter(|node| node.tags.contains(&"#cross_reference".to_string()) || node.content.contains("[["))
                .collect();
        }
        self.graph.search(query)
            .into_iter()
            .filter(|node| node.tags.contains(&"#cross_reference".to_string()) || node.content.contains("[["))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossref_creation_and_search() {
        let graph = Arc::new(Dendrite::new());
        let linker = CrossReferenceLinker::new(Arc::clone(&graph), None);

        let res = linker.create_cross_reference(
            "Genesis 1:1",
            "Natural Philosophy Ch 1",
            "Creation parallel to cosmological observations.",
        );
        assert!(res.is_ok());

        let results = linker.find_cross_references("Genesis");
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Genesis 1:1"));
        assert!(results[0].content.contains("Creation parallel"));
    }
}

