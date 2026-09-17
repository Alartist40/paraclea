//! Data binding from BibleReader and LibraryEngine into GalaxySystem nodes via multi-tier celestial hierarchy.

use paraclea_core::bible::BibleReader;
use paraclea_core::library::LibraryEngine;

use super::physics::{pseudo_scatter, GalaxyNode, GalaxySystem};

const FLAGSHIP_TAGS: &[&str] = &[
    "KJV", "WEB", "BSB", "NIV", "ESV", "ASV", "RV1909", "VULGATA", "LUTHER", "SYNODAL", "CUV", "ALMEIDA", "WAKNA", "WASNA",
];

pub fn build_galaxy(_reader: &BibleReader, library: &LibraryEngine) -> GalaxySystem {
    let mut system = GalaxySystem::new();

    // 1. Central Core: Holy Scripture (The Central Star / Sun) at coordinate (0, 0, 0)
    system.add_node(GalaxyNode::new_sun("sun_word", "Holy Scripture (The Word)"));

    // 2. Languages as Sub-Themes (Primary Planets orbiting the Central Sun)
    let all_languages = BibleReader::list_languages();
    let mut prioritized_langs = Vec::new();
    let priority_codes = ["eng", "spa", "ell", "heb", "lat", "deu", "fra", "rus", "chi", "por", "ita", "kor", "ukr", "tgl"];
    
    for code in priority_codes {
        if let Some(l) = all_languages.iter().find(|lang| lang.code.eq_ignore_ascii_case(code)) {
            prioritized_langs.push(l.clone());
        }
    }
    for l in &all_languages {
        if !prioritized_langs.iter().any(|p| p.code == l.code) && prioritized_langs.len() < 14 {
            prioritized_langs.push(l.clone());
        }
    }

    let num_langs = prioritized_langs.len().max(1);

    for (l_idx, lang_meta) in prioritized_langs.iter().enumerate() {
        let planet_id = format!("lang_{}", lang_meta.code);
        let base_radius = 3.8 + (l_idx as f32 % 4.0) * 0.45;
        let orbit_speed = 0.078 + ((l_idx as f32) * 0.007) % 0.035;
        let phase = (l_idx as f32 / num_langs as f32) * 2.0 * std::f32::consts::PI;
        let inclination = ((l_idx as f32 * 0.8).sin()) * 0.22;

        system.add_node(GalaxyNode::new_planet(
            &planet_id,
            &lang_meta.name,
            &lang_meta.code.to_uppercase(),
            base_radius,
            orbit_speed,
            phase,
            inclination,
            0.88,
        ));

        // 3. Bible Translations as Moons rotating around their corresponding Language Planet
        let translations = BibleReader::list_translations_for_lang(&lang_meta.code);
        let num_trans = translations.len().max(1);

        for (t_idx, trans) in translations.iter().take(6).enumerate() {
            let moon_id = format!("trans_{}_{}", lang_meta.code, trans.tag);
            let m_radius = 0.7 + (t_idx as f32) * 0.22;
            let m_speed = 0.185 + (t_idx as f32) * 0.032;
            let m_phase = (t_idx as f32 / num_trans as f32) * 2.0 * std::f32::consts::PI;
            let is_flagship = FLAGSHIP_TAGS.iter().any(|&f| f.eq_ignore_ascii_case(&trans.tag));
            let brightness = if is_flagship { 0.72 } else { 0.48 };

            system.add_node(GalaxyNode::new_moon(
                &moon_id,
                &trans.name,
                &trans.tag,
                &planet_id,
                m_radius,
                m_speed,
                m_phase,
                brightness,
                is_flagship,
            ));
        }
    }

    // 4. Core Generalist Dust Cloud (Dense 80-particle cloud orbiting near center)
    let core_glyphs = ['+', '∘', '·', '✦', '*'];
    for g_idx in 0..80 {
        let seed = 0xCAFE_BABE_0000 + (g_idx as u64) * 97 + 13;
        let offset = pseudo_scatter(seed, 1.5);
        let brightness = 0.28 + (g_idx as f32 % 5.0) * 0.048;
        let node_id = format!("core_dust_{}", g_idx);
        let node_name = format!("Scripture Core Dust #{}", g_idx + 1);

        let mut node = GalaxyNode::new_asteroid(
            &node_id,
            &node_name,
            super::physics::NodeSubtype::CoreDust,
            None,
            (offset[0] * offset[0] + offset[2] * offset[2]).sqrt().max(0.3),
            0.042 + (g_idx as f32 % 4.0) * 0.008,
            (g_idx as f32 / 80.0) * 2.0 * std::f32::consts::PI,
            brightness,
        );
        node.pos = offset;
        node.glyph = core_glyphs[g_idx % core_glyphs.len()];
        system.add_node(node);
    }

    // 4b. Ambient Inter-Orbit Dust (fills gaps between planet orbits)
    let amb_glyphs = ['+', '∘', '·'];
    for d_idx in 0..50 {
        let seed = 0xDEAD_BEEF_0000 + (d_idx as u64) * 53 + 7;
        let angle = (d_idx as f32 / 50.0) * 2.0 * std::f32::consts::PI;
        let radius = 2.0 + (d_idx as f32 % 6.0) * 0.7;
        let y_jitter = pseudo_scatter(seed, 0.4);
        let pos = [
            radius * angle.cos(),
            y_jitter[1],
            radius * angle.sin(),
        ];
        let brightness = 0.22 + (d_idx as f32 % 4.0) * 0.04;
        let mut node = GalaxyNode::new_asteroid(
            &format!("ambient_dust_{}", d_idx),
            &format!("Ambient Dust #{}", d_idx + 1),
            super::physics::NodeSubtype::CoreDust,
            None,
            radius,
            0.03 + (d_idx as f32 % 5.0) * 0.006,
            angle,
            brightness,
        );
        node.pos = pos;
        node.glyph = amb_glyphs[d_idx % amb_glyphs.len()];
        system.add_node(node);
    }

    // 5. Library Knowledge Categories as Outer Decks & Saturn-like Chapter Rings
    let categories = library.list_categories();
    let num_cats = categories.len().max(1);
    let ring_glyphs = ['+', '∘', '·'];

    for (c_idx, cat) in categories.iter().enumerate() {
        let ast_id = format!("cat_{}", cat);
        let ast_name = format!("Library Deck: [{}]", cat.to_uppercase());
        let cat_radius = 8.0 + (c_idx as f32) * 0.7;
        let cat_speed = 0.038 + (c_idx as f32) * 0.005;
        let cat_phase = (c_idx as f32 / num_cats as f32) * 2.0 * std::f32::consts::PI;

        system.add_node(GalaxyNode::new_asteroid(
            &ast_id,
            &ast_name,
            super::physics::NodeSubtype::CategoryDeck,
            None,
            cat_radius,
            cat_speed,
            cat_phase,
            0.62,
        ));

        // Saturn-like ring particles around each category deck (8 particles)
        for p_idx in 0..8 {
            let p_id = format!("cat_ring_{}_{}", cat, p_idx);
            let p_name = format!("{} Chapter Cluster #{}", cat.to_uppercase(), p_idx + 1);
            let p_radius = 0.8 + (p_idx as f32) * 0.18;
            let p_speed = 0.14 + (p_idx as f32) * 0.018;
            let p_phase = (p_idx as f32 / 8.0) * 2.0 * std::f32::consts::PI;

            let mut p_node = GalaxyNode::new_asteroid(
                &p_id,
                &p_name,
                super::physics::NodeSubtype::CategoryRingDust,
                Some(&ast_id),
                p_radius,
                p_speed,
                p_phase,
                0.40,
            );
            p_node.glyph = ring_glyphs[p_idx % ring_glyphs.len()];
            system.add_node(p_node);
        }
    }

    // Initialize initial positions
    system.update(0.0);
    system
}
