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
            LanguageOption { id: 4, name: "German (Deutsch)".to_string(), code: "de".to_string() },
            LanguageOption { id: 5, name: "Chinese (中文)".to_string(), code: "zh".to_string() },
            LanguageOption { id: 6, name: "Japanese (日本語)".to_string(), code: "ja".to_string() },
            LanguageOption { id: 7, name: "Portuguese (Português)".to_string(), code: "pt".to_string() },
            LanguageOption { id: 8, name: "Russian (Русский)".to_string(), code: "ru".to_string() },
            LanguageOption { id: 9, name: "Greek (Ελληνικά)".to_string(), code: "grc".to_string() },
            LanguageOption { id: 10, name: "Hebrew (עברית)".to_string(), code: "he".to_string() },
            LanguageOption { id: 11, name: "Latin (Vulgata)".to_string(), code: "la".to_string() },
            LanguageOption { id: 12, name: "Dutch (Nederlands)".to_string(), code: "nl".to_string() },
            LanguageOption { id: 13, name: "Korean (한국어)".to_string(), code: "ko".to_string() },
            LanguageOption { id: 14, name: "Tagalog (Filipino)".to_string(), code: "tl".to_string() },
        ]
    }

    /// List available translations for a given language code.
    pub fn list_translations_for_lang(lang_code: &str) -> Vec<TranslationOption> {
        match lang_code {
            "es" => vec![
                TranslationOption { id: 1, tag: "SpaRV".to_string(), name: "La Santa Biblia Reina-Valera (1909)".to_string(), language: "Spanish".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "SpaRV1865".to_string(), name: "Reina-Valera 1865".to_string(), language: "Spanish".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 3, tag: "SpaRVG".to_string(), name: "Reina Valera Gómez".to_string(), language: "Spanish".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 4, tag: "SpaPlatense".to_string(), name: "Biblia Platense (Straubinger)".to_string(), language: "Spanish".to_string(), is_easy: true, file_path: None },
            ],
            "fr" => vec![
                TranslationOption { id: 1, tag: "FreBBB".to_string(), name: "French Bible Bovet Bonnet (1900)".to_string(), language: "French".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 2, tag: "FreBDM1744".to_string(), name: "Bible David Martin (1744)".to_string(), language: "French".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 3, tag: "FreCrampon".to_string(), name: "La Bible Augustin Crampon (1923)".to_string(), language: "French".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 4, tag: "FreGeneve1669".to_string(), name: "Bible de Genève (1669)".to_string(), language: "French".to_string(), is_easy: false, file_path: None },
            ],
            "de" => vec![
                TranslationOption { id: 1, tag: "GerBoLut".to_string(), name: "Luther 1545 Bibel (moderne)".to_string(), language: "German".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 2, tag: "GerElb1905".to_string(), name: "Unrevidierte Elberfelder (1905)".to_string(), language: "German".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 3, tag: "GerMenge".to_string(), name: "Menge-Bibel (1939)".to_string(), language: "German".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 4, tag: "GerSch".to_string(), name: "Schlachter Bibel (1951)".to_string(), language: "German".to_string(), is_easy: true, file_path: None },
            ],
            "zh" => vec![
                TranslationOption { id: 1, tag: "ChiUn".to_string(), name: "和合本 (Chinese Union Version)".to_string(), language: "Chinese".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "ChiSB".to_string(), name: "思高本 (Sīgāo Běn)".to_string(), language: "Chinese".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 3, tag: "ChiUnL".to_string(), name: "文理和合本 (Classical Chinese)".to_string(), language: "Chinese".to_string(), is_easy: false, file_path: None },
            ],
            "ja" => vec![
                TranslationOption { id: 1, tag: "JapKougo".to_string(), name: "口語訳聖書 (Kougo-yaku 1954)".to_string(), language: "Japanese".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 2, tag: "JapBungo".to_string(), name: "文語訳聖書 (Bungo-yaku Meiji)".to_string(), language: "Japanese".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 3, tag: "JapDenmo".to_string(), name: "電網聖書 (Denmo Bible)".to_string(), language: "Japanese".to_string(), is_easy: true, file_path: None },
            ],
            "pt" => vec![
                TranslationOption { id: 1, tag: "PorBLivre".to_string(), name: "Bíblia Livre (Portuguese)".to_string(), language: "Portuguese".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 2, tag: "PorNVA".to_string(), name: "Bíblia Nova Versão de Acesso Livre".to_string(), language: "Portuguese".to_string(), is_easy: true, file_path: None },
            ],
            "ru" => vec![
                TranslationOption { id: 1, tag: "RusSynodal".to_string(), name: "Синодальный Перевод (Synodal)".to_string(), language: "Russian".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "RusMakarij".to_string(), name: "Перевод Макария (Makarij)".to_string(), language: "Russian".to_string(), is_easy: true, file_path: None },
            ],
            "grc" => vec![
                TranslationOption { id: 1, tag: "TR".to_string(), name: "Textus Receptus (1550/1894)".to_string(), language: "Greek".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "Byz".to_string(), name: "Byzantine Textform (2013)".to_string(), language: "Greek".to_string(), is_easy: false, file_path: None },
            ],
            "he" => vec![
                TranslationOption { id: 1, tag: "HebModern".to_string(), name: "Modern Hebrew Bible".to_string(), language: "Hebrew".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 2, tag: "WLC".to_string(), name: "Westminster Leningrad Codex".to_string(), language: "Hebrew".to_string(), is_easy: false, file_path: None },
            ],
            "la" => vec![
                TranslationOption { id: 1, tag: "Vulgate".to_string(), name: "Biblia Sacra Vulgata (Latin)".to_string(), language: "Latin".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 2, tag: "VulgClementine".to_string(), name: "Clementine Vulgate".to_string(), language: "Latin".to_string(), is_easy: false, file_path: None },
            ],
            _ => vec![
                TranslationOption { id: 1, tag: "KJV".to_string(), name: "King James Version (Authorized KJV)".to_string(), language: "English".to_string(), is_easy: false, file_path: Some("data/kjv.json".to_string()) },
                TranslationOption { id: 2, tag: "BBE".to_string(), name: "Bible in Basic English (Easy English)".to_string(), language: "English".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 3, tag: "AKJV".to_string(), name: "American King James Version".to_string(), language: "English".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 4, tag: "ASV".to_string(), name: "American Standard Version (1901)".to_string(), language: "English".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 5, tag: "BSB".to_string(), name: "Berean Standard Bible".to_string(), language: "English".to_string(), is_easy: true, file_path: None },
                TranslationOption { id: 6, tag: "Darby".to_string(), name: "Darby Bible (1889)".to_string(), language: "English".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 7, tag: "Geneva1599".to_string(), name: "Geneva Bible (1599)".to_string(), language: "English".to_string(), is_easy: false, file_path: None },
                TranslationOption { id: 8, tag: "YLT".to_string(), name: "Young's Literal Translation (1898)".to_string(), language: "English".to_string(), is_easy: false, file_path: None },
            ],
        }
    }
}

pub const OLD_TESTAMENT_BOOKS: &[&str] = &[
    "Genesis", "Exodus", "Leviticus", "Numbers", "Deuteronomy", "Joshua", "Judges", "Ruth",
    "1 Samuel", "2 Samuel", "1 Kings", "2 Kings", "1 Chronicles", "2 Chronicles", "Ezra",
    "Nehemiah", "Esther", "Job", "Psalms", "Proverbs", "Ecclesiastes", "Song of Solomon",
    "Isaiah", "Jeremiah", "Lamentations", "Ezekiel", "Daniel", "Hosea", "Joel", "Amos",
    "Obadiah", "Jonah", "Micah", "Nahum", "Habakkuk", "Zephaniah", "Haggai", "Zechariah", "Malachi"
];

pub const NEW_TESTAMENT_BOOKS: &[&str] = &[
    "Matthew", "Mark", "Luke", "John", "Acts", "Romans", "1 Corinthians", "2 Corinthians",
    "Galatians", "Ephesians", "Philippians", "Colossians", "1 Thessalonians", "2 Thessalonians",
    "1 Timothy", "2 Timothy", "Titus", "Philemon", "Hebrews", "James", "1 Peter", "2 Peter",
    "1 John", "2 John", "3 John", "Jude", "Revelation"
];

impl BibleReader {
    /// Search for book by name or common alias (fuzzy matching case-insensitive).
    pub fn find_book(&self, query: &str) -> Option<&BookMeta> {
        let q = query.trim().to_lowercase();
        let normalized = match q.as_str() {
            "songs of solomon" | "songs of solomon " | "song of songs" | "canticles" => "song of solomon",
            "psalm" | "psalms" => "psalms",
            "revelations" => "revelation",
            "1samuel" => "1 samuel",
            "2samuel" => "2 samuel",
            "1kings" => "1 kings",
            "2kings" => "2 kings",
            "1chronicles" => "1 chronicles",
            "2chronicles" => "2 chronicles",
            "1corinthians" => "1 corinthians",
            "2corinthians" => "2 corinthians",
            "1thessalonians" => "1 thessalonians",
            "2thessalonians" => "2 thessalonians",
            "1timothy" => "1 timothy",
            "2timothy" => "2 timothy",
            "1peter" => "1 peter",
            "2peter" => "2 peter",
            "1john" => "1 john",
            "2john" => "2 john",
            "3john" => "3 john",
            _ => q.as_str(),
        };

        self.books.iter().find(|b| {
            let bn = b.name.to_lowercase();
            bn == normalized || bn.starts_with(normalized) || normalized.starts_with(&bn)
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

    /// Read single verse with target translation tag fallback.
    pub fn read_translation_verse(
        &self,
        translation_tag: &str,
        book_name: &str,
        chapter: usize,
        verse: usize,
    ) -> Option<String> {
        // 1. Check dynamic JSON Bible files in ~/.paraclea/bibles/
        if let Ok(home) = std::env::var("HOME") {
            let bibles_dir = PathBuf::from(home).join(".paraclea/bibles");
            if bibles_dir.exists() {
                if let Ok(entries) = fs::read_dir(&bibles_dir) {
                    for lang_entry in entries.flatten() {
                        if lang_entry.path().is_dir() {
                            let target_file = lang_entry.path().join(format!("{}.json", translation_tag.to_lowercase()));
                            if target_file.exists() {
                                if let Ok(content) = fs::read_to_string(&target_file) {
                                    if let Ok(val) = serde_json::from_str::<Value>(&content) {
                                        if let Some(books) = val.as_array() {
                                            let q = book_name.trim().to_lowercase();
                                            if let Some(bk) = books.iter().find(|b| {
                                                let name = b.get("name").and_then(|n| n.as_str()).unwrap_or("").to_lowercase();
                                                name == q || name.starts_with(&q)
                                            }) {
                                                if let Some(chaps) = bk.get("chapters").and_then(|c| c.as_array()) {
                                                    if chapter >= 1 && chapter <= chaps.len() {
                                                        if let Some(verses) = chaps[chapter - 1].as_array() {
                                                            if verse >= 1 && verse <= verses.len() {
                                                                if let Some(t) = verses[verse - 1].as_str() {
                                                                    return Some(t.to_string());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Fallback to CSV Bibles
        if let Some(csv_path) = CsvBibleReader::locate_csv(translation_tag) {
            if let Some(text) = CsvBibleReader::read_verse(&csv_path, book_name, chapter, verse) {
                return Some(text);
            }
        }
        self.read_verse(book_name, chapter, verse)
    }

    /// Read full chapter with target translation tag fallback.
    pub fn read_translation_chapter(
        &self,
        translation_tag: &str,
        book_name: &str,
        chapter: usize,
    ) -> Option<Vec<(usize, String)>> {
        if let Some(csv_path) = CsvBibleReader::locate_csv(translation_tag) {
            if let Some(verses) = CsvBibleReader::read_chapter(&csv_path, book_name, chapter) {
                return Some(verses);
            }
        }
        self.read_chapter(book_name, chapter)
    }
}

pub struct CsvBibleReader;

impl CsvBibleReader {
    pub fn locate_csv(tag: &str) -> Option<PathBuf> {
        let filename = format!("{}.csv", tag);
        let candidate_paths = vec![
            PathBuf::from(format!("/home/orangepi/Documents/reference/bible_databases/formats/csv/{}", filename)),
            PathBuf::from(format!("data/{}", filename)),
            PathBuf::from(format!("../Documents/reference/bible_databases/formats/csv/{}", filename)),
        ];
        candidate_paths.into_iter().find(|p| p.exists())
    }

    pub fn read_verse(csv_path: &Path, book_name: &str, chapter: usize, verse: usize) -> Option<String> {
        let content = fs::read_to_string(csv_path).ok()?;
        let q_book = book_name.trim().to_lowercase();
        let target_chap = chapter.to_string();
        let target_v = verse.to_string();

        for line in content.lines().skip(1) {
            let mut parts = line.splitn(4, ',');
            let b = match parts.next() {
                Some(val) => val.trim().to_lowercase(),
                None => continue,
            };
            let c = match parts.next() {
                Some(val) => val.trim(),
                None => continue,
            };
            let v = match parts.next() {
                Some(val) => val.trim(),
                None => continue,
            };
            let text = match parts.next() {
                Some(val) => val.trim().trim_matches('"'),
                None => continue,
            };

            if (b == q_book || b.starts_with(&q_book)) && c == target_chap && v == target_v {
                return Some(text.to_string());
            }
        }
        None
    }

    pub fn read_chapter(csv_path: &Path, book_name: &str, chapter: usize) -> Option<Vec<(usize, String)>> {
        let content = fs::read_to_string(csv_path).ok()?;
        let q_book = book_name.trim().to_lowercase();
        let target_chap = chapter.to_string();
        let mut verses = Vec::new();

        for line in content.lines().skip(1) {
            let mut parts = line.splitn(4, ',');
            let b = match parts.next() {
                Some(val) => val.trim().to_lowercase(),
                None => continue,
            };
            let c = match parts.next() {
                Some(val) => val.trim(),
                None => continue,
            };
            let v_str = match parts.next() {
                Some(val) => val.trim(),
                None => continue,
            };
            let text = match parts.next() {
                Some(val) => val.trim().trim_matches('"'),
                None => continue,
            };

            if (b == q_book || b.starts_with(&q_book)) && c == target_chap {
                if let Ok(v_num) = v_str.parse::<usize>() {
                    verses.push((v_num, text.to_string()));
                }
            }
        }

        if verses.is_empty() {
            None
        } else {
            Some(verses)
        }
    }
}
