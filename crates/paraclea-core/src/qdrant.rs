//! Qdrant Vector Database HTTP REST Client for Paraclea
//!
//! Handles collection creation, point upserts (embeddings + metadata payloads),
//! and vector similarity search queries against Qdrant (`http://localhost:6333`).

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct QdrantClient {
    client: Client,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: Value,
    pub score: f32,
    pub payload: Value,
}

impl QdrantClient {
    /// Create new Qdrant client instance.
    pub fn new(base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client for Qdrant")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Health check to verify Qdrant server connectivity.
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/healthz", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Create vector collection with Cosine similarity distance.
    pub async fn create_collection(&self, collection_name: &str, vector_dim: usize) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        let body = serde_json::json!({
            "vectors": {
                "size": vector_dim,
                "distance": "Cosine"
            }
        });

        let resp = self.client.put(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let _ = resp.text().await;
        }
        Ok(())
    }

    /// Upsert single vector point with payload metadata.
    pub async fn upsert(
        &self,
        collection_name: &str,
        point_id: Value,
        vector: Vec<f32>,
        payload: Value,
    ) -> Result<()> {
        let url = format!("{}/collections/{}/points", self.base_url, collection_name);
        let body = serde_json::json!({
            "points": [{
                "id": point_id,
                "vector": vector,
                "payload": payload
            }]
        });

        let resp = self.client.put(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Qdrant upsert failed: {}", err);
        }
        Ok(())
    }

    /// Perform vector similarity search.
    pub async fn search(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: u64,
    ) -> Result<Vec<VectorSearchResult>> {
        let url = format!("{}/collections/{}/points/search", self.base_url, collection_name);
        let body = serde_json::json!({
            "vector": query_vector,
            "limit": limit,
            "with_payload": true
        });

        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Qdrant search failed on collection '{}': {}", collection_name, err);
        }

        let res_json: Value = resp.json().await?;
        let mut results = Vec::new();

        if let Some(arr) = res_json["result"].as_array() {
            for item in arr {
                if let Ok(sr) = serde_json::from_value::<VectorSearchResult>(item.clone()) {
                    results.push(sr);
                }
            }
        }

        Ok(results)
    }
}
