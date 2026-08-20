//! File Format Auto-Detection Module for Paraclea
//!
//! Categorizes files by extension and specifies the appropriate ingestion route
//! (Vision OCR, PDF/Doc extract, Text chunking, or Bible DB JSON).

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Image,      // png, jpg, jpeg, webp, bmp, gif, tiff
    Pdf,        // pdf
    Epub,       // epub
    Docx,       // docx
    Text,       // txt, md, markdown, rst, adoc
    Json,       // json (Bible DB)
    Html,       // html, htm
    Rtf,        // rtf
    Unknown,
}

impl FileType {
    /// Detect file type from file extension.
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref() {
            Some("png") | Some("jpg") | Some("jpeg") | Some("webp")
            | Some("bmp") | Some("gif") | Some("tiff") | Some("tif") => FileType::Image,
            Some("pdf") => FileType::Pdf,
            Some("epub") => FileType::Epub,
            Some("docx") => FileType::Docx,
            Some("txt") | Some("md") | Some("markdown") | Some("rst")
            | Some("adoc") | Some("asciidoc") => FileType::Text,
            Some("json") => FileType::Json,
            Some("html") | Some("htm") => FileType::Html,
            Some("rtf") => FileType::Rtf,
            _ => FileType::Unknown,
        }
    }

    /// Return human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            FileType::Image => "Image / Document Scan (OCR)",
            FileType::Pdf => "PDF Document",
            FileType::Epub => "EPUB E-Book",
            FileType::Docx => "Word Document",
            FileType::Text => "Plain Text / Markdown",
            FileType::Json => "Bible Database JSON",
            FileType::Html => "HTML Web Page",
            FileType::Rtf => "Rich Text Document",
            FileType::Unknown => "Unknown Format",
        }
    }

    /// Return assigned ingestion processing route.
    #[allow(dead_code)]
    pub fn ingest_route(&self) -> &'static str {
        match self {
            FileType::Image => "ocr",
            FileType::Pdf | FileType::Epub | FileType::Docx | FileType::Html | FileType::Rtf => "extract-then-embed",
            FileType::Text => "chunk-then-embed",
            FileType::Json => "bible-ingest",
            FileType::Unknown => "reject",
        }
    }
}
