//! Interactive Visual Bible Tree-Map & Heatmap Engine.
//!
//! Visualizes study inquiry density, cross-reference saturation, and read frequency
//! across all 66 canon books using ANSI/Unicode intensity levels:
//! ` ` (0%), `░` (25%), `▒` (50%), `▓` (75%), `█` (100%).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard 66-book Protestant canon order and metadata.
pub const CANON_BOOKS: &[(&str, &str, usize, usize)] = &[
    // (Name, Abbrev, Chapters, Testament 0=OT, 1=NT)
    ("Genesis", "GEN", 50, 0), ("Exodus", "EXO", 40, 0), ("Leviticus", "LEV", 27, 0),
    ("Numbers", "NUM", 36, 0), ("Deuteronomy", "DEU", 34, 0), ("Joshua", "JOS", 24, 0),
    ("Judges", "JDG", 21, 0), ("Ruth", "RUT", 4, 0), ("1 Samuel", "1SA", 31, 0),
    ("2 Samuel", "2SA", 24, 0), ("1 Kings", "1KI", 22, 0), ("2 Kings", "2KI", 25, 0),
    ("1 Chronicles", "1CH", 29, 0), ("2 Chronicles", "2CH", 36, 0), ("Ezra", "EZR", 10, 0),
    ("Nehemiah", "NEH", 13, 0), ("Esther", "EST", 10, 0), ("Job", "JOB", 42, 0),
    ("Psalms", "PSA", 150, 0), ("Proverbs", "PRO", 31, 0), ("Ecclesiastes", "ECC", 12, 0),
    ("Song of Solomon", "SNG", 8, 0), ("Isaiah", "ISA", 66, 0), ("Jeremiah", "JER", 52, 0),
    ("Lamentations", "LAM", 5, 0), ("Ezekiel", "EZK", 48, 0), ("Daniel", "DAN", 12, 0),
    ("Hosea", "HOS", 14, 0), ("Joel", "JOL", 3, 0), ("Amos", "AMO", 9, 0),
    ("Obadiah", "OBA", 1, 0), ("Jonah", "JON", 4, 0), ("Micah", "MIC", 7, 0),
    ("Nahum", "NAM", 3, 0), ("Habakkuk", "HAB", 3, 0), ("Zephaniah", "ZEP", 3, 0),
    ("Haggai", "HAG", 2, 0), ("Zechariah", "ZEC", 14, 0), ("Malachi", "MAL", 4, 0),
    // New Testament
    ("Matthew", "MAT", 28, 1), ("Mark", "MRK", 16, 1), ("Luke", "LUK", 24, 1),
    ("John", "JHN", 21, 1), ("Acts", "ACT", 28, 1), ("Romans", "ROM", 16, 1),
    ("1 Corinthians", "1CO", 16, 1), ("2 Corinthians", "2CO", 13, 1), ("Galatians", "GAL", 6, 1),
    ("Ephesians", "EPH", 6, 1), ("Philippians", "PHP", 4, 1), ("Colossians", "COL", 4, 1),
    ("1 Thessalonians", "1TH", 5, 1), ("2 Thessalonians", "2TH", 3, 1), ("1 Timothy", "1TI", 6, 1),
    ("2 Timothy", "2TI", 4, 1), ("Titus", "TIT", 3, 1), ("Philemon", "PHM", 1, 1),
    ("Hebrews", "HEB", 13, 1), ("James", "JAS", 5, 1), ("1 Peter", "1PE", 5, 1),
    ("2 Peter", "2PE", 3, 1), ("1 John", "1JN", 5, 1), ("2 John", "2JN", 1, 1),
    ("3 John", "3JN", 1, 1), ("Jude", "JUD", 1, 1), ("Revelation", "REV", 22, 1),
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BibleStudyHeatmap {
    /// Maps "Book:Chapter" to total inquiry/read count.
    pub chapter_hits: HashMap<String, u32>,
}

impl BibleStudyHeatmap {
    pub fn new() -> Self {
        Self {
            chapter_hits: HashMap::new(),
        }
    }

    /// Record a study or read event on a chapter.
    pub fn record_read(&mut self, book: &str, chapter: usize) {
        let key = format!("{}:{}", book.to_lowercase(), chapter);
        *self.chapter_hits.entry(key).or_insert(0) += 1;
    }

    /// Get intensity level char for a given read count compared to a ceiling.
    pub fn intensity_char(count: u32, max_count: u32) -> char {
        if count == 0 || max_count == 0 {
            ' '
        } else {
            let ratio = (count as f32) / (max_count as f32);
            if ratio < 0.20 {
                '░'
            } else if ratio < 0.50 {
                '▒'
            } else if ratio < 0.80 {
                '▓'
            } else {
                '█'
            }
        }
    }

    /// Calculate aggregate read count for an entire book.
    pub fn book_total(&self, book: &str) -> u32 {
        let prefix = format!("{}:", book.to_lowercase());
        self.chapter_hits
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| *v)
            .sum()
    }

    /// Render a compact multi-line ASCII/Unicode grid of the 66 books.
    pub fn render_ascii_grid(&self) -> String {
        let mut out = String::new();
        out.push_str("=== SCRIPTURE STUDY HEATMAP ===\n");
        out.push_str("Legend: [ ] Unread  [░] 1-25%  [▒] 26-50%  [▓] 51-75%  [█] 76-100%\n\n");

        let max_book_hits = CANON_BOOKS
            .iter()
            .map(|(name, _, _, _)| self.book_total(name))
            .max()
            .unwrap_or(1)
            .max(1);

        out.push_str("-- Old Testament --\n");
        let mut col = 0;
        for (name, abbrev, _, testament) in CANON_BOOKS {
            if *testament == 1 && col != 0 {
                out.push('\n');
                out.push_str("\n-- New Testament --\n");
                col = 0;
            } else if *testament == 1 && !out.contains("-- New Testament --") {
                out.push_str("\n-- New Testament --\n");
            }

            let total = self.book_total(name);
            let ch = Self::intensity_char(total, max_book_hits);
            out.push_str(&format!("{}:[{}] ", abbrev, ch));
            col += 1;
            if col % 6 == 0 {
                out.push('\n');
            }
        }
        if col % 6 != 0 {
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heatmap_recording_and_rendering() {
        let mut heatmap = BibleStudyHeatmap::new();
        heatmap.record_read("Genesis", 1);
        heatmap.record_read("Genesis", 1);
        heatmap.record_read("Genesis", 2);
        heatmap.record_read("Revelation", 14);

        assert_eq!(heatmap.book_total("Genesis"), 3);
        assert_eq!(heatmap.book_total("Revelation"), 1);
        assert_eq!(heatmap.book_total("Romans"), 0);

        let grid = heatmap.render_ascii_grid();
        assert!(grid.contains("GEN:[█]"));
        assert!(grid.contains("REV:"));
    }
}
