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

    fn scan_directory(&mut self) -> Result<()> {
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

    pub fn reload(&mut self) -> Result<()> {
        self.books.clear();
        if !self.library_dir.exists() {
            let _ = fs::create_dir_all(&self.library_dir);
            let _ = self.create_sample_categories();
        }

        self.scan_directory()?;

        if self.books.is_empty() {
            let _ = self.create_sample_categories();
            let _ = self.scan_directory();
        }

        Ok(())
    }

    fn create_sample_categories(&self) -> Result<()> {
        // 1. Psychology & Wellness
        let psych_dir = self.library_dir.join("psychology");
        let _ = fs::create_dir_all(&psych_dir);
        let psych_book = GenericBook {
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
        let _ = fs::write(psych_dir.join("principles_of_mind.json"), serde_json::to_string_pretty(&psych_book)?);

        // 2. Survival & Preparedness
        let surv_dir = self.library_dir.join("survival");
        let _ = fs::create_dir_all(&surv_dir);
        let surv_book = GenericBook {
            title: "Emergency Preparedness & Bushcraft Manual".to_string(),
            author: Some("Field Preparedness Institute".to_string()),
            category: "survival".to_string(),
            chapters: vec![
                BookChapter {
                    chapter_number: 1,
                    title: "Water Purification & Filtration".to_string(),
                    content: "Access to clean water is the first priority in survival. Methods include boiling (rolling boil for 1-3 minutes), solar disinfection (SODIS), charcoal and sand filtration beds, and chemical purification tablets.".to_string(),
                },
                BookChapter {
                    chapter_number: 2,
                    title: "Shelter & Thermal Regulation".to_string(),
                    content: "Hypothermia and hyperthermia are rapid threats. Insulate yourself from the ground using dry leaves or branches, construct debris huts or lean-tos, and ensure wind and moisture barriers.".to_string(),
                },
            ],
            file_path: None,
        };
        let _ = fs::write(surv_dir.join("emergency_preparedness.json"), serde_json::to_string_pretty(&surv_book)?);

        // 3. Medical First Aid
        let med_dir = self.library_dir.join("medical");
        let _ = fs::create_dir_all(&med_dir);
        let med_book = GenericBook {
            title: "Field Trauma & Emergency First Aid".to_string(),
            author: Some("Medical Field Corps".to_string()),
            category: "medical".to_string(),
            chapters: vec![
                BookChapter {
                    chapter_number: 1,
                    title: "Hemorrhage Control & Tourniquets".to_string(),
                    content: "Direct pressure is the initial response to external bleeding. For life-threatening extremity arterial bleeding, apply a commercial tourniquet 2-3 inches above the wound high and tight until bleeding stops.".to_string(),
                },
                BookChapter {
                    chapter_number: 2,
                    title: "Airway Management & Recovery Position".to_string(),
                    content: "Ensure patent airway using head-tilt chin-lift or jaw thrust for suspected spinal trauma. Place unresponsive breathing casualties in lateral recovery position to prevent aspiration.".to_string(),
                },
            ],
            file_path: None,
        };
        let _ = fs::write(med_dir.join("field_first_aid.json"), serde_json::to_string_pretty(&med_book)?);

        // 4. Ellen G. White Spiritual Works
        let egw_dir = self.library_dir.join("egw");
        let _ = fs::create_dir_all(&egw_dir);
        let egw_book = GenericBook {
            title: "Steps to Christ".to_string(),
            author: Some("Ellen G. White".to_string()),
            category: "egw".to_string(),
            chapters: vec![
                BookChapter {
                    chapter_number: 1,
                    title: "God's Love for Man".to_string(),
                    content: "Nature and revelation alike testify of God's love. Our Father in heaven is the source of life, of wisdom, and of joy. Look at the wonderful and beautiful things of nature. Think of their marvelous adaptation to the needs and happiness, not only of man, but of all living creatures. The sunshine and the rain, that gladden and refresh the earth, the hills and seas and plains, all speak to us of the Creator's love.".to_string(),
                },
                BookChapter {
                    chapter_number: 2,
                    title: "The Sinner's Need of Christ".to_string(),
                    content: "Man was originally endowed with noble powers and a well-balanced mind. He was perfect in his being, and in harmony with God. His thoughts were pure, his aims holy. But through disobedience, his powers were perverted, and selfishness took the place of love.".to_string(),
                },
            ],
            file_path: None,
        };
        let _ = fs::write(egw_dir.join("steps_to_christ.json"), serde_json::to_string_pretty(&egw_book)?);

        // 5. Educational & Philosophy
        let edu_dir = self.library_dir.join("educational");
        let _ = fs::create_dir_all(&edu_dir);
        let edu_book = GenericBook {
            title: "Foundations of Natural Philosophy & Science".to_string(),
            author: Some("Classical Scholarship".to_string()),
            category: "educational".to_string(),
            chapters: vec![
                BookChapter {
                    chapter_number: 1,
                    title: "Observation, Hypothesis & Scientific Inquiry".to_string(),
                    content: "Knowledge advances through rigorous empirical observation, formulation of falsifiable hypotheses, deductive reasoning, and iterative experimental verification.".to_string(),
                },
                BookChapter {
                    chapter_number: 2,
                    title: "Astronomy & the Structure of the Cosmos".to_string(),
                    content: "From planetary orbits governed by celestial mechanics to stellar nucleosynthesis and galactic clusters, cosmological laws display remarkable symmetry and fine-tuning.".to_string(),
                },
            ],
            file_path: None,
        };
        let _ = fs::write(edu_dir.join("foundations_of_philosophy.json"), serde_json::to_string_pretty(&edu_book)?);

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_library_engine_initializes_samples() {
        let dir = tempdir().expect("Failed to create tempdir");
        let engine = LibraryEngine::new(dir.path().to_path_buf());

        let categories = engine.list_categories();
        assert_eq!(categories.len(), 5);
        assert!(categories.contains(&"psychology".to_string()));
        assert!(categories.contains(&"survival".to_string()));
        assert!(categories.contains(&"medical".to_string()));
        assert!(categories.contains(&"egw".to_string()));
        assert!(categories.contains(&"educational".to_string()));

        let books = engine.list_books(None);
        assert_eq!(books.len(), 5);

        let egw_books = engine.list_books(Some("egw"));
        assert_eq!(egw_books.len(), 1);
        assert_eq!(egw_books[0].title, "Steps to Christ");

        let chapter_res = engine.read_chapter("Steps to Christ", 1);
        assert!(chapter_res.is_some());
        let (book, ch) = chapter_res.unwrap();
        assert_eq!(book.title, "Steps to Christ");
        assert_eq!(ch.chapter_number, 1);
        assert!(ch.content.contains("Nature and revelation alike testify"));
    }
}

