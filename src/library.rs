//! Generic Multi-Category Library Engine for Paraclea
//!
//! Manages non-scripture book collections (EGW, Psychology, Survival, History, Classics)
//! stored under `$HOME/.paraclea/library/<category>/<book>.json` or `.md`/`.txt`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookChapter {
    pub chapter_number: usize,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericBook {
    pub title: String,
    pub author: Option<String>,
    pub category: String,
    pub chapters: Vec<BookChapter>,
    pub file_path: Option<String>,
}

pub struct LibraryEngine {
    pub library_dir: PathBuf,
    pub books: Vec<GenericBook>,
}

impl LibraryEngine {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        let library_dir = dir.as_ref().to_path_buf();
        let mut engine = Self {
            library_dir,
            books: Vec::new(),
        };
        let _ = engine.reload();
        engine
    }

    pub fn load_auto() -> Self {
        let dir = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".paraclea/library")
        } else {
            PathBuf::from("library")
        };
        Self::new(dir)
    }

    pub fn reload(&mut self) -> Result<()> {
        self.books.clear();
        if !self.library_dir.exists() {
            let _ = fs::create_dir_all(&self.library_dir);
            self.create_sample_category()?;
            return Ok(());
        }

        let cat_entries = fs::read_dir(&self.library_dir)
            .context("Failed to read library directory")?;

        for entry in cat_entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let category_name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if let Ok(files) = fs::read_dir(&path) {
                    for f_entry in files.flatten() {
                        let f_path = f_entry.path();
                        if let Some(ext) = f_path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if ext_str == "json" {
                                if let Ok(book) = Self::load_json_book(&f_path, &category_name) {
                                    self.books.push(book);
                                }
                            } else if ext_str == "md" || ext_str == "txt" {
                                if let Ok(book) = Self::load_text_book(&f_path, &category_name) {
                                    self.books.push(book);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn create_sample_category(&self) -> Result<()> {
        let sample_dir = self.library_dir.join("psychology");
        let _ = fs::create_dir_all(&sample_dir);
        let sample_book = GenericBook {
            title: "Principles of Mind & Wellness".to_string(),
            author: Some("Paraclea Research".to_string()),
            category: "psychology".to_string(),
            chapters: vec![
                BookChapter {
                    chapter_number: 1,
                    title: "The Architecture of Peace & Focus".to_string(),
                    content: "True mental peace begins with daily cognitive reflection, emotional stillness, and structured contemplation. When the mind focuses on higher wisdom, anxiety naturally recedes.".to_string(),
                },
                BookChapter {
                    chapter_number: 2,
                    title: "Habits, Memory & Cognition".to_string(),
                    content: "Memory is an interconnected web of experiences and associations. Building habits of daily study and active note-taking strengthens neuroplasticity and long-term retention.".to_string(),
                },
            ],
            file_path: None,
        };

        let sample_json = serde_json::to_string_pretty(&sample_book)?;
        let sample_path = sample_dir.join("principles_of_mind.json");
        let _ = fs::write(sample_path, sample_json);
        Ok(())
    }

    fn load_json_book(path: &Path, category: &str) -> Result<GenericBook> {
        let content = fs::read_to_string(path)?;
        let mut book: GenericBook = serde_json::from_str(&content)?;
        book.category = category.to_string();
        book.file_path = Some(path.to_string_lossy().to_string());
        Ok(book)
    }

    fn load_text_book(path: &Path, category: &str) -> Result<GenericBook> {
        let raw = fs::read_to_string(path)?;
        let title = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .replace('_', " ");

        let chapters = vec![BookChapter {
            chapter_number: 1,
            title: "Chapter 1".to_string(),
            content: raw,
        }];

        Ok(GenericBook {
            title,
            author: None,
            category: category.to_string(),
            chapters,
            file_path: Some(path.to_string_lossy().to_string()),
        })
    }

    pub fn list_categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.books.iter().map(|b| b.category.clone()).collect();
        cats.sort();
        cats.dedup();
        cats
    }

    pub fn list_books(&self, category: Option<&str>) -> Vec<&GenericBook> {
        self.books
            .iter()
            .filter(|b| {
                if let Some(cat) = category {
                    b.category.eq_ignore_ascii_case(cat)
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn find_book(&self, name_or_query: &str) -> Option<&GenericBook> {
        let q = name_or_query.trim().to_lowercase();
        self.books.iter().find(|b| {
            b.title.to_lowercase().contains(&q)
                || b.title.to_lowercase().replace(' ', "_").contains(&q)
        })
    }

    pub fn read_chapter(&self, book_query: &str, chapter_num: usize) -> Option<(&GenericBook, &BookChapter)> {
        let book = self.find_book(book_query)?;
        let chapter = book.chapters.iter().find(|c| c.chapter_number == chapter_num)?;
        Some((book, chapter))
    }
}
