//! Bible Navigation, Reading & Comparative Study Engine for Paraclea
//!
//! Provides interactive Bible reading with guided chapter/verse bounds lookup,
//! language/translation preference management, and side-by-side translation comparison.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageOption {
    pub id: usize,
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationOption {
    pub id: usize,
    pub tag: String,
    pub name: String,
    pub language: String,
    pub is_easy: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookMeta {
    pub name: String,
    pub total_chapters: usize,
    pub chapter_verse_counts: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct BibleReader {
    pub books: Vec<BookMeta>,
    pub raw_data: Option<Value>,
}

impl BibleReader {
    /// Load Bible dataset from primary JSON file (e.g. data/kjv.json).
    pub fn load_primary<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            anyhow::bail!("Bible JSON dataset not found at {:?}", path_ref);
        }

        let content_raw = fs::read_to_string(path_ref)
            .with_context(|| format!("Failed to read Bible dataset: {:?}", path_ref))?;
        let content = content_raw.trim_start_matches('\u{feff}');
        let json_val: Value = serde_json::from_str(content)
            .with_context(|| "Failed to parse Bible JSON structure")?;

        let mut books = Vec::new();
        if let Some(array_val) = json_val.as_array() {
            for book_obj in array_val {
                let name = book_obj.get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let mut chapter_counts = Vec::new();
                if let Some(chapters) = book_obj.get("chapters").and_then(|c| c.as_array()) {
                    for chap_val in chapters {
                        let verse_count = chap_val.as_array().map(|v| v.len()).unwrap_or(0);
                        chapter_counts.push(verse_count);
                    }
                }

                let total_chapters = chapter_counts.len();
                books.push(BookMeta {
                    name,
                    total_chapters,
                    chapter_verse_counts: chapter_counts,
                });
            }
        }

        Ok(Self {
            books,
            raw_data: Some(json_val),
        })
    }

    /// Automatically locate and load Bible dataset from standard global or repository paths.
    pub fn load_auto() -> Result<Self> {
        let candidates = vec![
            std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".paraclea/data/kjv.json")),
            Some(PathBuf::from("/home/orangepi/Documents/portfolio/paraclea/data/kjv.json")),
            Some(PathBuf::from("data/kjv.json")),
            Some(PathBuf::from("../data/kjv.json")),
        ];

        for cand in candidates.into_iter().flatten() {
            if cand.exists() {
                if let Ok(reader) = Self::load_primary(&cand) {
                    return Ok(reader);
                }
            }
        }

        anyhow::bail!("Bible JSON dataset not found on system.");
    }

    /// List supported languages.
    pub fn list_languages() -> Vec<LanguageOption> {
        vec![
            LanguageOption { id: 1, name: "English".to_string(), code: "en".to_string() },
            LanguageOption { id: 2, name: "Spanish (Español)".to_string(), code: "es".to_string() },
            LanguageOption { id: 3, name: "French (Français)".to_string(), code: "fr".to_string() },
            LanguageOption { id: 4, name: "Japanese (日本語)".to_string(), code: "ja".to_string() },
            LanguageOption { id: 5, name: "Chinese (中文)".to_string(), code: "zh".to_string() },
        ]
    }

    /// List available translations for a given language code.
    pub fn list_translations_for_lang(lang_code: &str) -> Vec<TranslationOption> {
        match lang_code {
            "es" => vec![
                TranslationOption { id: 1, tag: "RVR".to_string(), name: "Reina-Valera 1960 (Spanish)".to_string(), language: "Spanish".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "NVI".to_string(), name: "Nueva Versión Internacional (Easy Spanish)".to_string(), language: "Spanish".to_string(), is_easy: true, file_path: None },
            ],
            "fr" => vec![
                TranslationOption { id: 1, tag: "LSG".to_string(), name: "Louis Segond (French)".to_string(), language: "French".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "BDS".to_string(), name: "La Bible du Semeur (Easy French)".to_string(), language: "French".to_string(), is_easy: true, file_path: None },
            ],
            "ja" => vec![
                TranslationOption { id: 1, tag: "SHIN".to_string(), name: "Sh改訳 (Japanese New Revised)".to_string(), language: "Japanese".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "KOUGO".to_string(), name: "Kougo-yaku (Easy Colloquial Japanese)".to_string(), language: "Japanese".to_string(), is_easy: true, file_path: None },
            ],
            "zh" => vec![
                TranslationOption { id: 1, tag: "CUV".to_string(), name: "Chinese Union Version (和合本)".to_string(), language: "Chinese".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "CNV".to_string(), name: "Chinese New Version (新譯本 Easy Chinese)".to_string(), language: "Chinese".to_string(), is_easy: true, file_path: None },
            ],
            _ => vec![
                TranslationOption { id: 1, tag: "KJV".to_string(), name: "King James Version (Authorized KJV)".to_string(), language: "English".to_string(), is_easy: false, file_path: Some("data/kjv.json".to_string()) },
                TranslationOption { id: 2, tag: "WEB".to_string(), name: "World English Bible (Modern English)".to_string(), language: "English".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 3, tag: "BBE".to_string(), name: "Bible in Basic English (Easy English)".to_string(), language: "English".to_string(), is_easy: true, file_path: None },
            ],
        }
    }

    /// Search for book by name (fuzzy matching case-insensitive).
    pub fn find_book(&self, query: &str) -> Option<&BookMeta> {
        let q = query.trim().to_lowercase();
        self.books.iter().find(|b| {
            b.name.to_lowercase() == q || b.name.to_lowercase().starts_with(&q)
        })
    }

    /// Get chapter count for a book.
    pub fn get_chapter_count(&self, book_name: &str) -> Option<usize> {
        self.find_book(book_name).map(|b| b.total_chapters)
    }

    /// Get verse count for a chapter in a book.
    pub fn get_verse_count(&self, book_name: &str, chapter: usize) -> Option<usize> {
        let book = self.find_book(book_name)?;
        if chapter >= 1 && chapter <= book.total_chapters {
            Some(book.chapter_verse_counts[chapter - 1])
        } else {
            None
        }
    }

    /// Read single verse.
    pub fn read_verse(&self, book_name: &str, chapter: usize, verse: usize) -> Option<String> {
        let raw = self.raw_data.as_ref()?;
        let books = raw.as_array()?;

        let q = book_name.trim().to_lowercase();
        let target_book = books.iter().find(|b| {
            let name = b.get("name").and_then(|n| n.as_str()).unwrap_or("").to_lowercase();
            name == q || name.starts_with(&q)
        })?;

        let chapters = target_book.get("chapters")?.as_array()?;
        if chapter < 1 || chapter > chapters.len() {
            return None;
        }

        let verses = chapters[chapter - 1].as_array()?;
        if verse < 1 || verse > verses.len() {
            return None;
        }

        verses[verse - 1].as_str().map(|s| s.to_string())
    }

    /// Read all verses in a chapter.
    pub fn read_chapter(&self, book_name: &str, chapter: usize) -> Option<Vec<(usize, String)>> {
        let raw = self.raw_data.as_ref()?;
        let books = raw.as_array()?;

        let q = book_name.trim().to_lowercase();
        let target_book = books.iter().find(|b| {
            let name = b.get("name").and_then(|n| n.as_str()).unwrap_or("").to_lowercase();
            name == q || name.starts_with(&q)
        })?;

        let chapters = target_book.get("chapters")?.as_array()?;
        if chapter < 1 || chapter > chapters.len() {
            return None;
        }

        let verses = chapters[chapter - 1].as_array()?;
        let result = verses
            .iter()
            .enumerate()
            .map(|(idx, v)| (idx + 1, v.as_str().unwrap_or("").to_string()))
            .collect();

        Some(result)
    }
}
