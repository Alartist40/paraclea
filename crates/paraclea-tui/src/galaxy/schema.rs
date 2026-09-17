//! Paraclea DatabaseSchema implementation binding Scripture & Library knowledge into Mazzaroth.

use mazzaroth::schema::{Category, DatabaseSchema, Deck, DustConfig, Item};
use paraclea_core::bible::BibleReader;
use paraclea_core::library::LibraryEngine;

const FLAGSHIP_TAGS: &[&str] = &[
    "KJV", "WEB", "BSB", "NIV", "ESV", "ASV", "RV1909", "VULGATA", "LUTHER", "SYNODAL", "CUV",
    "ALMEIDA", "WAKNA", "WASNA",
];

pub struct ParacleaSchema<'a> {
    pub reader: &'a BibleReader,
    pub library: &'a LibraryEngine,
}

impl<'a> ParacleaSchema<'a> {
    pub fn new(reader: &'a BibleReader, library: &'a LibraryEngine) -> Self {
        Self { reader, library }
    }
}

impl<'a> DatabaseSchema for ParacleaSchema<'a> {
    fn root_id(&self) -> &str {
        "sun_word"
    }

    fn root_name(&self) -> &str {
        "Holy Scripture (The Word)"
    }

    fn categories(&self) -> Vec<Category> {
        let all_languages = BibleReader::list_languages();
        let mut prioritized = Vec::new();
        let priority_codes = [
            "eng", "spa", "ell", "heb", "lat", "deu", "fra", "rus", "chi", "por", "ita", "kor",
            "ukr", "tgl",
        ];

        for code in priority_codes {
            if let Some(l) = all_languages.iter().find(|lang| lang.code.eq_ignore_ascii_case(code)) {
                prioritized.push(l.clone());
            }
        }
        for l in &all_languages {
            if !prioritized.iter().any(|p| p.code == l.code) && prioritized.len() < 14 {
                prioritized.push(l.clone());
            }
        }

        prioritized
            .into_iter()
            .map(|l| Category::new(format!("lang_{}", l.code), l.name, l.code.to_uppercase()))
            .collect()
    }

    fn items_for_category(&self, category_id: &str) -> Vec<Item> {
        let lang_code = category_id.strip_prefix("lang_").unwrap_or(category_id);
        let translations = BibleReader::list_translations_for_lang(lang_code);

        translations
            .into_iter()
            .take(6)
            .map(|t| {
                let is_flagship = FLAGSHIP_TAGS.iter().any(|&f| f.eq_ignore_ascii_case(&t.tag));
                Item::new(
                    format!("trans_{}_{}", lang_code, t.tag),
                    t.name,
                    t.tag,
                    is_flagship,
                )
            })
            .collect()
    }

    fn outer_decks(&self) -> Vec<Deck> {
        self.library
            .list_categories()
            .into_iter()
            .map(|cat| {
                Deck::new(
                    format!("cat_{}", cat),
                    format!("Library Deck: [{}]", cat.to_uppercase()),
                    cat.to_uppercase(),
                )
            })
            .collect()
    }

    fn particles_for_deck(&self, _deck_id: &str) -> usize {
        8
    }

    fn core_dust_config(&self) -> DustConfig {
        DustConfig {
            count: 80,
            scatter: 1.5,
        }
    }

    fn ambient_dust_count(&self) -> usize {
        50
    }
}
