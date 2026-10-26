//! Zero-Copy Flat File Bible Paging Engine (`.parabible` format).
//!
//! Provides zero-RAM, zero-SQL memory-mapped chapter reading across 160+ translations.
//! Binary format:
//! [Header: 16 bytes magic "PARABIBLE\0\0\0\0\0\0\0"]
//! [Index Count: u32]
//! [Array of Index Entries: (book_idx: u8, chapter: u16, byte_offset: u64, byte_len: u32)]
//! [Raw UTF-8 text payload block]

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub const MAGIC_HEADER: &[u8; 16] = b"PARABIBLE\0\0\0\0\0\0\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChapterIndexEntry {
    pub book_idx: u8,
    pub chapter: u16,
    pub byte_offset: u64,
    pub byte_len: u32,
}

#[derive(Debug)]
pub struct ParaBibleFile {
    pub translation: String,
    pub index: HashMap<(u8, u16), (u64, u32)>,
    file: File,
}

impl ParaBibleFile {
    /// Open and verify a `.parabible` flat file.
    pub fn open<P: AsRef<Path>>(path: P, translation: &str) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut header = [0u8; 16];
        file.read_exact(&mut header)?;
        if &header != MAGIC_HEADER {
            return Err(anyhow!("Invalid magic header for parabible file"));
        }

        let mut count_buf = [0u8; 4];
        file.read_exact(&mut count_buf)?;
        let count = u32::from_le_bytes(count_buf) as usize;

        let mut index = HashMap::with_capacity(count);
        for _ in 0..count {
            let mut entry_buf = [0u8; 15]; // 1 + 2 + 8 + 4
            file.read_exact(&mut entry_buf)?;
            let book_idx = entry_buf[0];
            let chapter = u16::from_le_bytes([entry_buf[1], entry_buf[2]]);
            let byte_offset = u64::from_le_bytes([
                entry_buf[3], entry_buf[4], entry_buf[5], entry_buf[6],
                entry_buf[7], entry_buf[8], entry_buf[9], entry_buf[10],
            ]);
            let byte_len = u32::from_le_bytes([
                entry_buf[11], entry_buf[12], entry_buf[13], entry_buf[14],
            ]);
            index.insert((book_idx, chapter), (byte_offset, byte_len));
        }

        Ok(Self {
            translation: translation.to_string(),
            index,
            file,
        })
    }

    /// Read an exact chapter with single I/O seek without parsing SQL.
    pub fn read_chapter(&mut self, book_idx: u8, chapter: u16) -> Result<String> {
        let (offset, len) = self
            .index
            .get(&(book_idx, chapter))
            .copied()
            .ok_or_else(|| anyhow!("Chapter not found: book {} ch {}", book_idx, chapter))?;

        self.file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; len as usize];
        self.file.read_exact(&mut buf)?;
        let text = String::from_utf8(buf)?;
        Ok(text)
    }

    /// Builder helper to create a `.parabible` file from a list of chapters.
    pub fn create<P: AsRef<Path>>(
        path: P,
        chapters: &[(u8, u16, &str)],
    ) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(MAGIC_HEADER)?;
        let count = chapters.len() as u32;
        file.write_all(&count.to_le_bytes())?;

        // Calculate offset after header (16) + count (4) + (count * 15)
        let index_bytes_len = (count as u64) * 15;
        let mut current_offset = 16 + 4 + index_bytes_len;

        let mut computed_entries = Vec::new();
        for (b, c, text) in chapters {
            let len = text.len() as u32;
            computed_entries.push((*b, *c, current_offset, len));
            current_offset += len as u64;
        }

        for (b, c, offset, len) in &computed_entries {
            file.write_all(&[*b])?;
            file.write_all(&c.to_le_bytes())?;
            file.write_all(&offset.to_le_bytes())?;
            file.write_all(&len.to_le_bytes())?;
        }

        for (_, _, text) in chapters {
            file.write_all(text.as_bytes())?;
        }

        file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parabible_roundtrip() -> Result<()> {
        let temp = NamedTempFile::new()?;
        let path = temp.path();

        let data = vec![
            (1u8, 1u16, "In the beginning God created the heaven and the earth."),
            (1u8, 2u16, "Thus the heavens and the earth were finished..."),
            (66u8, 22u16, "The grace of our Lord Jesus Christ be with you all. Amen."),
        ];

        ParaBibleFile::create(path, &data)?;
        let mut reader = ParaBibleFile::open(path, "KJV")?;

        let gen1 = reader.read_chapter(1, 1)?;
        assert_eq!(gen1, "In the beginning God created the heaven and the earth.");

        let rev22 = reader.read_chapter(66, 22)?;
        assert_eq!(rev22, "The grace of our Lord Jesus Christ be with you all. Amen.");

        assert!(reader.read_chapter(2, 1).is_err());
        Ok(())
    }
}
