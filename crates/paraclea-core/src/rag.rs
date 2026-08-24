//! RAG Retrieval Engine & Multi-Model Router for Paraclea
//!
//! Embeds user queries, retrieves relevant Scripture & book passages from Qdrant vector storage,
//! constructs RAG prompt contexts, and routes complex reasoning tasks to heavy models.

use crate::ollama::OllamaClient;
use crate::qdrant::QdrantClient;
use anyhow::Result;
use tracing::debug;

pub struct RagEngine<'a> {
    pub ollama: &'a OllamaClient,
    pub qdrant: &'a QdrantClient,
}

pub struct RagRetrievalResult {
    pub context_text: String,
    pub sources: Vec<String>,
}

impl<'a> RagEngine<'a> {
    pub fn new(ollama: &'a OllamaClient, qdrant: &'a QdrantClient) -> Self {
        Self { ollama, qdrant }
    }

    /// Retrieve relevant passages from Qdrant vector storage.
    pub async fn retrieve_context(
        &self,
        query: &str,
        collection: &str,
        limit: u64,
        embed_model: &str,
    ) -> Result<RagRetrievalResult> {
        debug!("Embedding query for RAG retrieval in collection '{}'", collection);

        // 1. Generate query embedding vector
        let query_vector = match self.ollama.embed(query, embed_model).await {
            Ok(v) => v,
            Err(e) => {
                debug!("Failed to generate query embedding: {}", e);
                return Ok(RagRetrievalResult {
                    context_text: String::new(),
                    sources: Vec::new(),
                });
            }
        };

        // 2. Perform vector search in Qdrant
        let search_results = match self.qdrant.search(collection, query_vector, limit).await {
            Ok(res) => res,
            Err(e) => {
                debug!("Qdrant search query failed: {}", e);
                return Ok(RagRetrievalResult {
                    context_text: String::new(),
                    sources: Vec::new(),
                });
            }
        };

        // 3. Format context string and sources
        let mut context_text = String::new();
        let mut sources = Vec::new();

        for item in search_results {
            let payload = item.payload;
            let book = payload["book"].as_str().unwrap_or("Reference");
            let chap = payload["chapter"].as_u64().unwrap_or(1);
            let verses = payload["verses"].as_str().unwrap_or("1");
            let text = payload["text"].as_str().unwrap_or("");

            if !text.is_empty() {
                let citation = format!("{} {}:{}", book, chap, verses);
                context_text.push_str(&format!("[{}]\n{}\n\n", citation, text));
                sources.push(citation);
            }
        }

        Ok(RagRetrievalResult {
            context_text,
            sources,
        })
    }

    /// Route model selection based on question complexity and keywords.
    pub fn route_model<'b>(&self, query: &str, default_model: &'b str, heavy_model: &'b str) -> &'b str {
        let q_lower = query.to_lowercase();
        let is_complex = query.len() > 180
            || q_lower.contains("compare")
            || q_lower.contains("theology")
            || q_lower.contains("prophecy")
            || q_lower.contains("typology")
            || q_lower.contains("exegesis")
            || q_lower.contains("analyze")
            || q_lower.contains("fulfillment")
            || q_lower.contains("greek")
            || q_lower.contains("hebrew");

        if is_complex {
            heavy_model
        } else {
            default_model
        }
    }
}
