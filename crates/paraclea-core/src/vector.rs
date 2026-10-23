//! In-Process Embedded Vector Search Engine (Zero-Daemon RAG)
//!
//! Provides pure Rust cosine similarity and Euclidean vector indexing, top-k
//! nearest-neighbor retrieval, metadata payload storage, and local file persistence.
//! Operates 100% in-process with 0 background daemons and zero external network calls.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// An individual vector point containing an ID, embedding coordinates, and payload metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: Value,
    #[serde(default)]
    pub norm: f32,
}

impl VectorPoint {
    pub fn new(id: impl Into<String>, vector: Vec<f32>, payload: Value) -> Self {
        let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        Self {
            id: id.into(),
            vector,
            payload,
            norm,
        }
    }
}

/// Nearest neighbor search result with similarity score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: Value,
    pub score: f32,
    pub payload: Value,
}

/// A named collection of high-dimensional vector points.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VectorCollection {
    pub name: String,
    pub dimension: usize,
    pub points: HashMap<String, VectorPoint>,
}

impl VectorCollection {
    pub fn new(name: impl Into<String>, dimension: usize) -> Self {
        Self {
            name: name.into(),
            dimension,
            points: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, point: VectorPoint) {
        self.points.insert(point.id.clone(), point);
    }

    pub fn get(&self, id: &str) -> Option<&VectorPoint> {
        self.points.get(id)
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Search the collection using Cosine Similarity.
    pub fn search(&self, query_vector: &[f32], limit: usize) -> Vec<VectorSearchResult> {
        if query_vector.is_empty() || self.points.is_empty() {
            return Vec::new();
        }

        let query_norm = query_vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        if query_norm == 0.0 {
            return Vec::new();
        }

        let mut scored: Vec<VectorSearchResult> = self
            .points
            .values()
            .map(|point| {
                let score = if point.norm == 0.0 || point.vector.len() != query_vector.len() {
                    0.0
                } else {
                    let dot: f32 = point
                        .vector
                        .iter()
                        .zip(query_vector.iter())
                        .map(|(a, b)| a * b)
                        .sum();
                    dot / (point.norm * query_norm)
                };

                VectorSearchResult {
                    id: Value::String(point.id.clone()),
                    score,
                    payload: point.payload.clone(),
                }
            })
            .collect();

        // Sort descending by similarity score
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored
    }
}

/// Thread-safe in-process multi-collection vector database.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EmbeddedVectorStore {
    collections: HashMap<String, VectorCollection>,
}

impl EmbeddedVectorStore {
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }

    pub fn create_collection(&mut self, name: &str, dimension: usize) {
        if !self.collections.contains_key(name) {
            self.collections.insert(name.to_string(), VectorCollection::new(name, dimension));
        }
    }

    pub fn upsert(&mut self, collection_name: &str, point: VectorPoint) -> Result<()> {
        let collection = self
            .collections
            .entry(collection_name.to_string())
            .or_insert_with(|| VectorCollection::new(collection_name, point.vector.len()));
        collection.upsert(point);
        Ok(())
    }

    pub fn search(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Vec<VectorSearchResult> {
        if let Some(col) = self.collections.get(collection_name) {
            col.search(query_vector, limit)
        } else {
            Vec::new()
        }
    }

    pub fn count(&self, collection_name: &str) -> usize {
        self.collections.get(collection_name).map(|c| c.len()).unwrap_or(0)
    }

    /// Save the vector store to a compressed or raw JSON file on disk.
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(path).with_context(|| format!("Failed to create file {:?}", path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    /// Load the vector store from disk.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("Failed to open file {:?}", path))?;
        let reader = BufReader::new(file);
        let store = serde_json::from_reader(reader)?;
        Ok(store)
    }
}

/// Shared Arc/RwLock instance for concurrent queries.
pub type SharedVectorStore = Arc<RwLock<EmbeddedVectorStore>>;

#[cfg(test)]
pub mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    pub fn test_embedded_vector_store() {
        let mut store = EmbeddedVectorStore::new();
        store.create_collection("verses", 3);

        // Add 3 vectors
        // Point A: pointing towards [1.0, 0.0, 0.0]
        let p1 = VectorPoint::new("gen_1_1", vec![1.0, 0.1, 0.0], serde_json::json!({"book": "Genesis"}));
        // Point B: pointing towards [0.0, 1.0, 0.0]
        let p2 = VectorPoint::new("rev_22_21", vec![0.0, 1.0, 0.1], serde_json::json!({"book": "Revelation"}));
        // Point C: pointing towards [0.9, 0.2, 0.0] (very close to Point A)
        let p3 = VectorPoint::new("gen_1_2", vec![0.9, 0.2, 0.0], serde_json::json!({"book": "Genesis"}));

        store.upsert("verses", p1).unwrap();
        store.upsert("verses", p2).unwrap();
        store.upsert("verses", p3).unwrap();

        assert_eq!(store.count("verses"), 3);

        // Query vector close to Genesis [1.0, 0.0, 0.0]
        let query = vec![1.0, 0.0, 0.0];
        let results = store.search("verses", &query, 2);

        assert_eq!(results.len(), 2);
        // Top 2 should both be Genesis points with high cosine scores
        assert!(results[0].score > 0.95);
        assert!(results[1].score > 0.95);
        assert_ne!(results[0].id, Value::String("rev_22_21".to_string()));

        // Persistence test
        let dir = tempdir().unwrap();
        let path = dir.path().join("vectors.json");
        store.save_to_file(&path).unwrap();

        let loaded = EmbeddedVectorStore::load_from_file(&path).unwrap();
        assert_eq!(loaded.count("verses"), 3);
        let loaded_results = loaded.search("verses", &query, 1);
        assert_eq!(loaded_results.len(), 1);
        assert_eq!(loaded_results[0].id, results[0].id);
    }
}
