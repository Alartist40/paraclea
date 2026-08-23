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

pub fn code_to_language_name(code: &str) -> &'static str {
    match code.to_lowercase().as_str() {
        "afr" | "af" => "Afrikaans",
        "amh" | "am" => "Amharic (አማርኛ)",
        "arb" | "ar" => "Arabic (العربية)",
        "ben" | "bn" => "Bengali (বাংলা)",
        "zho" | "zh" => "Chinese (中文)",
        "nld" | "nl" => "Dutch (Nederlands)",
        "eng" | "en" => "English",
        "fra" | "fr" => "French (Français)",
        "deu" | "de" => "German (Deutsch)",
        "ell" | "el" | "grc" => "Greek (Ελληνικά)",
        "guj" | "gu" => "Gujarati (ગુજરાતી)",
        "hau" | "ha" => "Hausa",
        "heb" | "he" => "Hebrew (עברית)",
        "hin" | "hi" => "Hindi (हिन्दी)",
        "hun" | "hu" => "Hungarian (Magyar)",
        "ibo" | "ig" => "Igbo",
        "jpn" | "ja" => "Japanese (日本語)",
        "kor" | "ko" => "Korean (한국어)",
        "lat" | "la" => "Latin (Vulgata)",
        "lug" | "lg" => "Luganda",
        "mal" | "ml" => "Malayalam (മലയാളം)",
        "nep" | "ne" => "Nepali (नेपाली)",
        "nso" => "Sepedi (Northern Sotho)",
        "por" | "pt" => "Portuguese (Português)",
        "pan" | "pa" => "Punjabi (ਪੰਜਾਬੀ)",
        "rus" | "ru" => "Russian (Русский)",
        "sna" | "sn" => "Shona",
        "spa" | "es" => "Spanish (Español)",
        "swh" | "sw" => "Swahili (Kiswahili)",
        "tam" | "ta" => "Tamil (தமிழ்)",
        "tel" | "te" => "Telugu (తెలుగు)",
        "tsn" | "tn" => "Setswana",
        "twi" | "tw" => "Twi (Ghana - Akuapem & Asante)",
        "xho" | "xh" => "Xhosa (isiXhosa)",
        "yor" | "yo" => "Yoruba",
        "zul" | "zu" => "Zulu (isiZulu)",
        _ => "Other Language",
    }
}

pub fn format_translation_name(stem: &str, lang_code: &str) -> String {
    let lower = stem.to_lowercase();
    match lower.as_str() {
        "en-kjv" | "kjv" => "King James Version (KJV 1611 - Authorized)".to_string(),
        "en-nkjv" | "nkjv" => "New King James Version (NKJV)".to_string(),
        "en-niv" | "niv" => "New International Version (NIV)".to_string(),
        "en-nlt" | "nlt" => "New Living Translation (NLT)".to_string(),
        "en-esv" | "esv" => "English Standard Version (ESV)".to_string(),
        "en-nasb" | "nasb" => "New American Standard Bible (NASB)".to_string(),
        "en-asv" | "asv" => "American Standard Version (ASV 1901)".to_string(),
        "en-bsb" | "bsb" => "Berean Standard Bible (BSB)".to_string(),
        "en-web" | "web" => "World English Bible (WEB)".to_string(),
        "en-webbe" | "webbe" => "World English Bible British Edition (WEBBE)".to_string(),
        "en-webus" | "webus" => "World English Bible US Edition (WEBUS)".to_string(),
        "en-gnv" | "gnv" => "Geneva Bible (1599)".to_string(),
        "en-ylt" | "ylt" => "Young's Literal Translation (1898)".to_string(),
        "en-dra" | "dra" => "Douay-Rheims American Edition (1899)".to_string(),
        "en-rv" | "rv" => "Revised Version (1885)".to_string(),
        "tw-wakna" => "Twi Akuapem Bible (Nkwa Asɛm)".to_string(),
        "tw-wasna" => "Twi Asante Bible (Nkwa Asɛm)".to_string(),
        "zulu_bible" | "zul" => "Zulu Bible (isiZulu)".to_string(),
        "xhosa_bible" | "xho" => "Xhosa Bible (isiXhosa)".to_string(),
        "afrikaans_bible" | "afr" => "Afrikaans Bible".to_string(),
        "swahili_bible" | "swh" => "Swahili Bible (Kiswahili)".to_string(),
        "amharic_bible" | "amh" => "Amharic Bible (አማርኛ)".to_string(),
        "hausa_bible" | "hau" => "Hausa Bible".to_string(),
        "yoruba_bible" | "yo" => "Yoruba Bible".to_string(),
        "igbo_bible" | "ig" => "Igbo Bible".to_string(),
        _ => {
            let clean = lower.replace("en-", "").replace("us-", "").replace('_', " ").to_uppercase();
            format!("{} ({})", clean, code_to_language_name(lang_code))
        }
    }
}

pub fn matches_tag(file_stem: &str, target_tag: &str) -> bool {
    let s = file_stem.to_lowercase();
    let t = target_tag.to_lowercase();
    if s == t {
        return true;
    }
    if (t == "kjv" || t == "en-kjv" || t == "en_kjv") && (s == "en-kjv" || s == "kjv" || s == "en_kjv") {
        return true;
    }
    if (t == "nkjv" || t == "en-nkjv") && (s == "en-nkjv" || s == "nkjv") {
        return true;
    }
    if (t == "niv" || t == "en-niv") && (s == "en-niv" || s == "niv") {
        return true;
    }
    if (t == "nlt" || t == "en-nlt") && (s == "en-nlt" || s == "nlt") {
        return true;
    }
    if (t == "esv" || t == "en-esv") && (s == "en-esv" || s == "esv") {
        return true;
    }
    if s.replace("en-", "") == t.replace("en-", "") {
        return true;
    }
    false
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

    /// Dynamic listing of all supported Bible languages discovered in ~/.paraclea/bibles/, sorted strictly alphabetically.
    pub fn list_languages() -> Vec<LanguageOption> {
        let mut raw_map: Vec<(String, String)> = Vec::new(); // (code, display_name)

        if let Ok(home) = std::env::var("HOME") {
            let bibles_dir = PathBuf::from(home).join(".paraclea/bibles");
            if bibles_dir.exists() {
                if let Ok(entries) = fs::read_dir(&bibles_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            let code = entry.file_name().to_string_lossy().to_string();
                            let name_mapped = code_to_language_name(&code);
                            let display_name = if name_mapped == "Other Language" {
                                format!("Language ({})", code.to_uppercase())
                            } else {
                                name_mapped.to_string()
                            };
                            raw_map.push((code, display_name));
                        }
                    }
                }
            }
        }

        // Sort strictly in alphabetical order by display_name
        raw_map.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
        raw_map.dedup_by(|a, b| a.1 == b.1);

        let mut languages = Vec::new();
        for (id, (code, name)) in raw_map.into_iter().enumerate() {
            languages.push(LanguageOption {
                id: id + 1,
                name,
                code,
            });
        }

        if languages.is_empty() {
            languages.push(LanguageOption { id: 1, name: "English".to_string(), code: "eng".to_string() });
        }

        languages
    }

    /// Dynamic listing of available Bible translations for a given language code, sorted alphabetically.
    pub fn list_translations_for_lang(lang_code: &str) -> Vec<TranslationOption> {
        let mut raw_items = Vec::new();

        if let Ok(home) = std::env::var("HOME") {
            let bibles_dir = PathBuf::from(home).join(format!(".paraclea/bibles/{}", lang_code));
            if bibles_dir.exists() {
                if let Ok(entries) = fs::read_dir(&bibles_dir) {
                    let mut files: Vec<PathBuf> = entries
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| p.extension().and_then(|ext| ext.to_str()) == Some("json"))
                        .collect();

                    for f in files {
                        let stem = f.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        let tag = stem.to_uppercase();
                        let display_name = format_translation_name(&stem, lang_code);
                        let is_easy = stem.contains("easy") || stem.contains("bbe") || stem.contains("nlt") || stem.contains("web");
                        raw_items.push((tag, display_name, is_easy, f.to_string_lossy().to_string()));
                    }
                }
            }
        }

        // Prioritize KJV first for English, then sort alphabetically by display_name
        raw_items.sort_by(|a, b| {
            let is_a_kjv = a.0 == "EN-KJV" || a.0 == "KJV";
            let is_b_kjv = b.0 == "EN-KJV" || b.0 == "KJV";
            if is_a_kjv { return std::cmp::Ordering::Less; }
            if is_b_kjv { return std::cmp::Ordering::Greater; }
            a.1.to_lowercase().cmp(&b.1.to_lowercase())
        });

        let mut options = Vec::new();
        for (id, (tag, name, is_easy, file_path)) in raw_items.into_iter().enumerate() {
            options.push(TranslationOption {
                id: id + 1,
                tag,
                name,
                language: code_to_language_name(lang_code).to_string(),
                is_easy,
                file_path: Some(file_path),
            });
        }

        if options.is_empty() {
            options.push(TranslationOption {
                id: 1,
                tag: "KJV".to_string(),
                name: "King James Version (KJV 1611 - Authorized)".to_string(),
                language: "English".to_string(),
                is_easy: false,
                file_path: Some("data/kjv.json".to_string()),
            });
        }

        options
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
}

pub fn find_json_bible_file(translation_tag: &str) -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        let bibles_dir = PathBuf::from(home).join(".paraclea/bibles");
        if bibles_dir.exists() {
            if let Ok(entries) = fs::read_dir(&bibles_dir) {
                for lang_entry in entries.flatten() {
                    if lang_entry.path().is_dir() {
                        if let Ok(files) = fs::read_dir(lang_entry.path()) {
                            for f in files.flatten() {
                                let p = f.path();
                                if p.extension().and_then(|e| e.to_str()) == Some("json") {
                                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                        if matches_tag(stem, translation_tag) {
                                            return Some(p);
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
    None
}

impl BibleReader {
    /// Read single verse with target translation tag fallback.
    pub fn read_translation_verse(
        &self,
        translation_tag: &str,
        book_name: &str,
        chapter: usize,
        verse: usize,
    ) -> Option<String> {
        if let Some(target_file) = find_json_bible_file(translation_tag) {
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
        if let Some(target_file) = find_json_bible_file(translation_tag) {
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
                                        let result = verses
                                            .iter()
                                            .enumerate()
                                            .map(|(idx, v)| (idx + 1, v.as_str().unwrap_or("").to_string()))
                                            .collect();
                                        return Some(result);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

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
