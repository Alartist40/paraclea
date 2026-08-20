//! Bible Database, Book-to-Skill & Unified File Ingestion Engine for Paraclea
//!
//! Parses Scripture JSON datasets, structured Markdown book skills, and auto-detected file formats,
//! chunks text into overlapping semantic passages, generates embeddings via Ollama,
//! and indexes vectors into Qdrant vector collections.

use crate::detect::FileType;
use crate::ollama::OllamaClient;
use crate::qdrant::QdrantClient;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tracing::info;

pub struct BibleIngestor<'a> {
    pub ollama: &'a OllamaClient,
    pub qdrant: &'a QdrantClient,
    pub embed_model: String,
    pub collection: String,
}

impl<'a> BibleIngestor<'a> {
    pub fn new(
        ollama: &'a OllamaClient,
        qdrant: &'a QdrantClient,
        embed_model: &str,
        collection: &str,
    ) -> Self {
        Self {
            ollama,
            qdrant,
            embed_model: embed_model.to_string(),
            collection: collection.to_string(),
        }
    }

    /// Ingest a Bible JSON file into Qdrant vector storage.
    pub async fn ingest_json_file(&self, json_path: &Path) -> Result<usize> {
        let content = fs::read_to_string(json_path)
            .with_context(|| format!("Failed to read Bible JSON file: {:?}", json_path))?;

        let json_val: Value = serde_json::from_str(&content)
            .with_context(|| "Failed to parse Bible JSON structure")?;

        self.qdrant.create_collection(&self.collection, 768).await.ok();

        let mut total_chunks = 0;

        if let Some(verses_array) = json_val.as_array() {
            let mut chunk_buffer: Vec<Value> = Vec::new();
            for item in verses_array {
                chunk_buffer.push(item.clone());
                if chunk_buffer.len() >= 3 {
                    self.process_verse_chunk(&chunk_buffer).await?;
                    total_chunks += 1;
                    chunk_buffer.clear();
                }
            }
            if !chunk_buffer.is_empty() {
                self.process_verse_chunk(&chunk_buffer).await?;
                total_chunks += 1;
            }
        } else if let Some(books_map) = json_val.as_object() {
            for (book_name, chapters_val) in books_map {
                if let Some(chapters_map) = chapters_val.as_object() {
                    for (chap_num, verses_val) in chapters_map {
                        let chap_u32 = chap_num.parse::<u32>().unwrap_or(1);
                        if let Some(verses_map) = verses_val.as_object() {
                            let mut chunk_buffer = Vec::new();
                            for (verse_num, verse_text_val) in verses_map {
                                let verse_u32 = verse_num.parse::<u32>().unwrap_or(1);
                                let text = verse_text_val.as_str().unwrap_or("").to_string();

                                chunk_buffer.push(serde_json::json!({
                                    "book": book_name,
                                    "chapter": chap_u32,
                                    "verse": verse_u32,
                                    "text": text
                                }));

                                if chunk_buffer.len() >= 3 {
                                    self.process_verse_chunk(&chunk_buffer).await?;
                                    total_chunks += 1;
                                    chunk_buffer.clear();
                                }
                            }
                            if !chunk_buffer.is_empty() {
                                self.process_verse_chunk(&chunk_buffer).await?;
                                total_chunks += 1;
                            }
                        }
                    }
                }
            }
        }

        info!("Bible ingestion finished: indexed {} chunks into collection '{}'", total_chunks, self.collection);
        Ok(total_chunks)
    }

    async fn process_verse_chunk(&self, verses: &[Value]) -> Result<()> {
        if verses.is_empty() {
            return Ok(());
        }

        let first = &verses[0];
        let last = &verses[verses.len() - 1];

        let book = first["book"].as_str().unwrap_or("Bible");
        let chap = first["chapter"].as_u64().unwrap_or(1);
        let first_v = first["verse"].as_u64().unwrap_or(1);
        let last_v = last["verse"].as_u64().unwrap_or(first_v);

        let combined_text: Vec<String> = verses
            .iter()
            .map(|v| {
                format!(
                    "{}:{}",
                    v["verse"].as_u64().unwrap_or(1),
                    v["text"].as_str().unwrap_or("")
                )
            })
            .collect();

        let chunk_str = format!("{} {}:{}-{} {}", book, chap, first_v, last_v, combined_text.join(" "));
        let point_id = format!("{}-{}-{}-{}", book.to_lowercase(), chap, first_v, last_v);

        let vector = self.ollama.embed(&chunk_str, &self.embed_model).await?;
        let payload = serde_json::json!({
            "book": book,
            "chapter": chap,
            "verses": format!("{}-{}", first_v, last_v),
            "text": chunk_str,
            "source": "bible"
        });

        self.qdrant
            .upsert(&self.collection, serde_json::json!(point_id), vector, payload)
            .await?;

        Ok(())
    }
}

pub struct BookIngestor<'a> {
    pub ollama: &'a OllamaClient,
    pub qdrant: &'a QdrantClient,
    pub embed_model: String,
}

impl<'a> BookIngestor<'a> {
    pub fn new(ollama: &'a OllamaClient, qdrant: &'a QdrantClient, embed_model: &str) -> Self {
        Self {
            ollama,
            qdrant,
            embed_model: embed_model.to_string(),
        }
    }

    /// Ingest a directory of markdown chapter files (e.g. output from `book-to-skill`).
    pub async fn ingest_book_directory(&self, dir_path: &Path, collection: &str) -> Result<usize> {
        if !dir_path.exists() || !dir_path.is_dir() {
            anyhow::bail!("Invalid directory path: {:?}", dir_path);
        }

        self.qdrant.create_collection(collection, 768).await.ok();
        let mut total_chunks = 0;

        let entries = fs::read_dir(dir_path)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let filename = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                let text = fs::read_to_string(&path)?;

                let paragraphs: Vec<&str> = text
                    .split("\n\n")
                    .map(|p| p.trim())
                    .filter(|p| p.len() > 30)
                    .collect();

                for (idx, para) in paragraphs.iter().enumerate() {
                    let point_id = format!("{}-{}", filename.to_lowercase(), idx);
                    let vector = self.ollama.embed(para, &self.embed_model).await?;

                    let payload = serde_json::json!({
                        "book": filename,
                        "chapter": idx + 1,
                        "verses": format!("p{}", idx + 1),
                        "text": para,
                        "source": collection
                    });

                    self.qdrant
                        .upsert(collection, serde_json::json!(point_id), vector, payload)
                        .await?;

                    total_chunks += 1;
                }
            }
        }

        info!("Book ingestion finished: indexed {} chunks into collection '{}'", total_chunks, collection);
        Ok(total_chunks)
    }
}

/// Unified Ingestion router auto-detecting file format and dispatching to OCR or text vectorization.
pub async fn ingest_file(
    ollama: &OllamaClient,
    qdrant: &QdrantClient,
    embed_model: &str,
    ocr_model: &str,
    file_path: &Path,
    collection: &str,
) -> Result<String> {
    if !file_path.exists() {
        anyhow::bail!("File not found at path: {:?}", file_path);
    }

    if file_path.is_dir() {
        let ingestor = BookIngestor::new(ollama, qdrant, embed_model);
        let count = ingestor.ingest_book_directory(file_path, collection).await?;
        return Ok(format!("Indexed {} chapter sections from directory into collection '{}'", count, collection));
    }

    let file_type = FileType::from_path(file_path);

    match file_type {
        FileType::Json => {
            let ingestor = BibleIngestor::new(ollama, qdrant, embed_model, collection);
            let count = ingestor.ingest_json_file(file_path).await?;
            Ok(format!("Indexed {} Bible verse chunks into collection '{}'", count, collection))
        }
        FileType::Image => {
            let text = ollama.ocr_image(file_path, ocr_model).await?;
            if text.is_empty() {
                anyhow::bail!("OCR produced no extracted text.");
            }

            let file_stem = file_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            qdrant.create_collection(collection, 768).await.ok();

            let vector = ollama.embed(&text, embed_model).await?;
            let point_id = format!("ocr-{}-{}", file_stem.to_lowercase(), uuid::Uuid::new_v4());

            let payload = serde_json::json!({
                "book": file_stem,
                "chapter": 1,
                "verses": "ocr",
                "text": text,
                "source": collection
            });

            qdrant.upsert(collection, serde_json::json!(point_id), vector, payload).await?;
            Ok(format!("OCR extracted {} chars from {:?} and indexed into collection '{}'", text.len(), file_path.file_name().unwrap_or_default(), collection))
        }
        FileType::Text | FileType::Html | FileType::Rtf => {
            let content = fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read text file: {:?}", file_path))?;

            let file_stem = file_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            qdrant.create_collection(collection, 768).await.ok();

            let paragraphs: Vec<&str> = content
                .split("\n\n")
                .map(|p| p.trim())
                .filter(|p| p.len() > 30)
                .collect();

            let mut count = 0;
            for (idx, para) in paragraphs.iter().enumerate() {
                let point_id = format!("{}-{}", file_stem.to_lowercase(), idx);
                let vector = ollama.embed(para, embed_model).await?;

                let payload = serde_json::json!({
                    "book": file_stem,
                    "chapter": idx + 1,
                    "verses": format!("p{}", idx + 1),
                    "text": para,
                    "source": collection
                });

                qdrant.upsert(collection, serde_json::json!(point_id), vector, payload).await?;
                count += 1;
            }

            Ok(format!("Indexed {} text paragraphs from {:?} into collection '{}'", count, file_path.file_name().unwrap_or_default(), collection))
        }
        FileType::Pdf | FileType::Epub | FileType::Docx => {
            anyhow::bail!(
                "Format {:?} ({}) is a binary document. Convert to text/markdown or image first.",
                file_type,
                file_type.label()
            )
        }
        FileType::Unknown => {
            anyhow::bail!("Unsupported file type for path {:?}", file_path)
        }
    }
}
