//! Strong's Hebrew and Greek Concordance & Original Language Inspector for Paraclea.
//!
//! Provides lexicon definitions, original lemma scripts, transliterations,
//! and theological definitions for biblical research.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrongEntry {
    pub number: &'static str,
    pub language: &'static str,
    pub lemma: &'static str,
    pub transliteration: &'static str,
    pub pronunciation: &'static str,
    pub definition: &'static str,
    pub kjv_translation: &'static str,
}

pub static STRONGS_DATABASE: &[StrongEntry] = &[
    // --- Hebrew (Old Testament) ---
    StrongEntry {
        number: "H7225",
        language: "Hebrew",
        lemma: "רֵאשִׁית",
        transliteration: "reshith",
        pronunciation: "ray-sheeth'",
        definition: "The first, beginning, best, chief, principal thing; the firstfruits in time, place, or rank.",
        kjv_translation: "beginning, chief(-est), first(-fruits), part, principal, time.",
    },
    StrongEntry {
        number: "H1254",
        language: "Hebrew",
        lemma: "בָּרָא",
        transliteration: "bara",
        pronunciation: "baw-raw'",
        definition: "To create, shape, form; an exclusively divine activity bringing into existence what did not previously exist.",
        kjv_translation: "create, creator, choose, make.",
    },
    StrongEntry {
        number: "H430",
        language: "Hebrew",
        lemma: "אֱלֹהִים",
        transliteration: "Elohim",
        pronunciation: "el-o-heem'",
        definition: "God, Supreme Divinity; plural of majesty expressing the fullness of divine power and creative excellence.",
        kjv_translation: "God, gods, judges, mighty.",
    },
    StrongEntry {
        number: "H8064",
        language: "Hebrew",
        lemma: "שָׁמַיִם",
        transliteration: "shamayim",
        pronunciation: "shaw-mah'-yim",
        definition: "The heavens, the sky, the upper realm of the universe, the celestial expanse.",
        kjv_translation: "heaven, air, astrologers, sky.",
    },
    StrongEntry {
        number: "H776",
        language: "Hebrew",
        lemma: "אֶרֶץ",
        transliteration: "erets",
        pronunciation: "eh'-rets",
        definition: "The earth, land, world, ground; material territory or the whole terrestrial sphere.",
        kjv_translation: "earth, land, ground, country, world.",
    },
    StrongEntry {
        number: "H7673",
        language: "Hebrew",
        lemma: "שָׁבַת",
        transliteration: "shabath",
        pronunciation: "shaw-bath'",
        definition: "To cease, desist, rest from work; to celebrate the holy day of completion and holy pause.",
        kjv_translation: "cease, rest, celebrate, keep sabbath.",
    },
    StrongEntry {
        number: "H7676",
        language: "Hebrew",
        lemma: "שַׁבָּת",
        transliteration: "shabbath",
        pronunciation: "shab-bawth'",
        definition: "Sabbath; the seventh day consecrated to holy cessation from secular labor and sacred communion with God.",
        kjv_translation: "Sabbath, week.",
    },
    StrongEntry {
        number: "H1285",
        language: "Hebrew",
        lemma: "בְּרִית",
        transliteration: "berith",
        pronunciation: "ber-eeth'",
        definition: "Covenant, alliance, pledge, divine contract of grace between God and His people.",
        kjv_translation: "covenant, confederacy, league.",
    },
    StrongEntry {
        number: "H2617",
        language: "Hebrew",
        lemma: "חֶסֶד",
        transliteration: "chesed",
        pronunciation: "kheh'-sed",
        definition: "Steadfast love, unfailing lovingkindness, covenant faithfulness, tender mercy.",
        kjv_translation: "mercy, kindness, lovingkindness, goodness.",
    },
    StrongEntry {
        number: "H3068",
        language: "Hebrew",
        lemma: "יְהוָה",
        transliteration: "YHWH",
        pronunciation: "yeh-ho-vaw'",
        definition: "The Self-Existent and Eternal Lord; the covenant name of God revealed to Moses.",
        kjv_translation: "LORD, GOD, Jehovah.",
    },
    StrongEntry {
        number: "H6944",
        language: "Hebrew",
        lemma: "קֹדֶשׁ",
        transliteration: "qodesh",
        pronunciation: "ko'-desh",
        definition: "Apartness, holiness, consecrated thing, sanctuary, sacred devotion.",
        kjv_translation: "holy, sanctuary, holiness, hallowed.",
    },
    StrongEntry {
        number: "H3444",
        language: "Hebrew",
        lemma: "יְשׁוּעָה",
        transliteration: "yeshuah",
        pronunciation: "yesh-oo'-aw",
        definition: "Salvation, deliverance, welfare, prosperity, victory; root of the name Jesus.",
        kjv_translation: "salvation, deliver, help, health.",
    },

    // --- Greek (New Testament) ---
    StrongEntry {
        number: "G3056",
        language: "Greek",
        lemma: "λόγος",
        transliteration: "logos",
        pronunciation: "log'-os",
        definition: "Word, speech, divine utterance, divine reason; in John 1:1, the Eternal Living Word incarnate.",
        kjv_translation: "word, saying, preaching, discourse, question.",
    },
    StrongEntry {
        number: "G2316",
        language: "Greek",
        lemma: "θεός",
        transliteration: "theos",
        pronunciation: "theh'-os",
        definition: "God, the one true Supreme Divinity, the Creator and Ruler of all things.",
        kjv_translation: "God, godly, Lord.",
    },
    StrongEntry {
        number: "G4151",
        language: "Greek",
        lemma: "πνεῦμα",
        transliteration: "pneuma",
        pronunciation: "pnyoo'-mah",
        definition: "Spirit, breath, wind, life-force; especially the Holy Spirit of God (Pneuma Hagion).",
        kjv_translation: "Spirit, Holy Ghost, wind, breath.",
    },
    StrongEntry {
        number: "G2424",
        language: "Greek",
        lemma: "Ἰησοῦς",
        transliteration: "Iesous",
        pronunciation: "ee-ay-sooce'",
        definition: "Jesus, of Hebrew origin (Yeshua / Joshua); signifying 'Yahweh is Salvation'.",
        kjv_translation: "Jesus.",
    },
    StrongEntry {
        number: "G5547",
        language: "Greek",
        lemma: "Χριστός",
        transliteration: "Christos",
        pronunciation: "khris-tos'",
        definition: "Christ, Anointed One; equivalent to Hebrew Mashiach (Messiah), consecrated as King and High Priest.",
        kjv_translation: "Christ.",
    },
    StrongEntry {
        number: "G4102",
        language: "Greek",
        lemma: "πίστις",
        transliteration: "pistis",
        pronunciation: "pis'-tis",
        definition: "Faith, conviction, holy trust, moral fidelity, assurance of things hoped for.",
        kjv_translation: "faith, fidelity, belief, assurance.",
    },
    StrongEntry {
        number: "G5485",
        language: "Greek",
        lemma: "χάρις",
        transliteration: "charis",
        pronunciation: "khar'-ece",
        definition: "Grace, unmerited divine favor, joyful lovingkindness extending redemption to the unworthy.",
        kjv_translation: "grace, favor, thank, pleasure.",
    },
    StrongEntry {
        number: "G26",
        language: "Greek",
        lemma: "ἀγάπη",
        transliteration: "agape",
        pronunciation: "ag-ah'-pay",
        definition: "Self-sacrificing, unconditional love, benevolence; the character and nature of God.",
        kjv_translation: "love, charity, dear.",
    },
    StrongEntry {
        number: "G3875",
        language: "Greek",
        lemma: "παράκλητος",
        transliteration: "parakletos",
        pronunciation: "par-ak'-lay-tos",
        definition: "One called alongside to help; Advocate, Intercessor, Comforter, Counselor, Paraclete.",
        kjv_translation: "Comforter, Advocate.",
    },
    StrongEntry {
        number: "G4521",
        language: "Greek",
        lemma: "σάββατον",
        transliteration: "sabbaton",
        pronunciation: "sab'-bat-on",
        definition: "The Sabbath, the seventh-day holy rest instituted at creation and observed by Christ and the apostles.",
        kjv_translation: "Sabbath, week.",
    },
    StrongEntry {
        number: "G4991",
        language: "Greek",
        lemma: "σωτηρία",
        transliteration: "soteria",
        pronunciation: "so-tay-ree'-ah",
        definition: "Deliverance, salvation, preservation from spiritual death and destruction.",
        kjv_translation: "salvation, deliver, health, save.",
    },
    StrongEntry {
        number: "G1343",
        language: "Greek",
        lemma: "δικαιοσύνη",
        transliteration: "dikaiosune",
        pronunciation: "dik-ah-yos-oo'-nay",
        definition: "Righteousness, justice, conformity to the divine will and character of God.",
        kjv_translation: "righteousness.",
    },
    StrongEntry {
        number: "G2222",
        language: "Greek",
        lemma: "ζωή",
        transliteration: "zoe",
        pronunciation: "dzo-ay'",
        definition: "Life, the principle of divine, eternal, imperishable life in Christ.",
        kjv_translation: "life, lifetime.",
    },
];

/// Normalize a Strong's key for tolerant lookup (e.g., "h7225", "H07225", "7225", "g3056").
pub fn normalize_key(input: &str) -> String {
    let s = input.trim().to_uppercase();
    if s.is_empty() {
        return s;
    }
    let first = s.chars().next().unwrap();
    if first == 'H' || first == 'G' {
        let num_part = &s[1..].trim_start_matches('0');
        format!("{}{}", first, num_part)
    } else {
        s
    }
}

/// Lookup a Strong's reference entry by number or keyword.
pub fn lookup(code: &str) -> Option<&'static StrongEntry> {
    let norm = normalize_key(code);
    STRONGS_DATABASE
        .iter()
        .find(|e| normalize_key(e.number) == norm || e.transliteration.eq_ignore_ascii_case(code.trim()))
}

/// Search Strong's dictionary by query string across number, lemma, transliteration, or definition.
pub fn search(query: &str) -> Vec<&'static StrongEntry> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return STRONGS_DATABASE.iter().collect();
    }
    STRONGS_DATABASE
        .iter()
        .filter(|e| {
            e.number.to_lowercase().contains(&q)
                || e.lemma.contains(&q)
                || e.transliteration.to_lowercase().contains(&q)
                || e.definition.to_lowercase().contains(&q)
                || e.kjv_translation.to_lowercase().contains(&q)
        })
        .collect()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_strongs_lookup() {
        let reshith = lookup("H7225").expect("H7225 should exist");
        assert_eq!(reshith.transliteration, "reshith");
        assert_eq!(reshith.lemma, "רֵאשִׁית");

        let logos = lookup("G3056").expect("G3056 should exist");
        assert_eq!(logos.transliteration, "logos");
        assert_eq!(logos.lemma, "λόγος");

        let paraclete = lookup("G3875").expect("G3875 should exist");
        assert_eq!(paraclete.transliteration, "parakletos");

        // Case-insensitive & leading zero tolerance
        assert_eq!(lookup("h7225"), Some(reshith));
        assert_eq!(lookup("g3056"), Some(logos));

        // Search by keyword
        let matches = search("Sabbath");
        assert!(!matches.is_empty());
    }
}
