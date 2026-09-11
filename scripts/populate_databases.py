#!/usr/bin/env python3
"""
Paraclea Complete Multi-Language Bible & Multi-Category Library Database Population Engine.
Processes all 140+ Bible translations from scrollmapper/bible_databases and 17+ languages from godlytalias/Bible-Database.
Outputs standardized, canonical JSON structures to ~/.paraclea/bibles/<lang>/<translation>.json, ~/.paraclea/data/, and bibles/.
"""

import os
import sys
import json
import re
import shutil
from pathlib import Path

HOME = Path.home()
PARACLEA_DIR = HOME / ".paraclea"
BIBLES_DIR = PARACLEA_DIR / "bibles"
DATA_DIR = PARACLEA_DIR / "data"
LIBRARY_DIR = PARACLEA_DIR / "library"
PROJECT_DIR = Path(__file__).resolve().parent.parent
PROJECT_DATA_DIR = PROJECT_DIR / "data"
PROJECT_BIBLES_DIR = PROJECT_DIR / "bibles"
SOURCES_DIR = HOME / ".cache/paraclea_sources"

CANONICAL_BOOKS = [
    "Genesis", "Exodus", "Leviticus", "Numbers", "Deuteronomy", "Joshua",
    "Judges", "Ruth", "1 Samuel", "2 Samuel", "1 Kings", "2 Kings", "1 Chronicles",
    "2 Chronicles", "Ezra", "Nehemiah", "Esther", "Job", "Psalms", "Proverbs", "Ecclesiastes",
    "Song of Solomon", "Isaiah", "Jeremiah", "Lamentations", "Ezekiel", "Daniel",
    "Hosea", "Joel", "Amos", "Obadiah", "Jonah", "Micah", "Nahum", "Habakkuk", "Zephaniah",
    "Haggai", "Zechariah", "Malachi", "Matthew", "Mark", "Luke", "John", "Acts", "Romans",
    "1 Corinthians", "2 Corinthians", "Galatians", "Ephesians", "Philippians", "Colossians",
    "1 Thessalonians", "2 Thessalonians", "1 Timothy", "2 Timothy", "Titus", "Philemon",
    "Hebrews", "James", "1 Peter", "2 Peter", "1 John", "2 John", "3 John", "Jude", "Revelation"
]

def ensure_dirs():
    BIBLES_DIR.mkdir(parents=True, exist_ok=True)
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    LIBRARY_DIR.mkdir(parents=True, exist_ok=True)
    PROJECT_DATA_DIR.mkdir(parents=True, exist_ok=True)
    PROJECT_BIBLES_DIR.mkdir(parents=True, exist_ok=True)

def standardize_scrollmapper_json(src_path):
    with open(src_path, 'r', encoding='utf-8', errors='ignore') as f:
        raw = json.load(f)

    books_raw = raw.get("books", []) if isinstance(raw, dict) else (raw if isinstance(raw, list) else [])
    standard_books = []

    for b in books_raw:
        b_name = b.get("name", "").strip()
        chapters_data = b.get("chapters", [])
        standard_chapters = []

        for ch in chapters_data:
            if isinstance(ch, dict):
                verses_list = ch.get("verses", [])
                chapter_verses = []
                for v in verses_list:
                    if isinstance(v, dict):
                        v_text = v.get("text", "")
                        v_text_clean = re.sub(r'\{[A-Za-z0-9]+\}', '', v_text)
                        v_text_clean = re.sub(r'<[0-9]+>', '', v_text_clean)
                        v_text_clean = re.sub(r'\s+', ' ', v_text_clean).strip()
                        chapter_verses.append(v_text_clean)
                    elif isinstance(v, str):
                        chapter_verses.append(v.strip())
                standard_chapters.append(chapter_verses)
            elif isinstance(ch, list):
                standard_chapters.append([v if isinstance(v, str) else str(v) for v in ch])

        if b_name and standard_chapters:
            standard_books.append({
                "name": b_name,
                "chapters": standard_chapters
            })

    return standard_books

def standardize_godlytalias_json(src_path):
    with open(src_path, 'r', encoding='utf-8', errors='ignore') as f:
        raw = json.load(f)

    raw_books = raw.get("Book", []) if isinstance(raw, dict) else []
    standard_books = []

    for idx, b in enumerate(raw_books):
        b_name = CANONICAL_BOOKS[idx] if idx < len(CANONICAL_BOOKS) else f"Book {idx + 1}"
        raw_chapters = b.get("Chapter", [])
        standard_chapters = []

        for ch in raw_chapters:
            raw_verses = ch.get("Verse", [])
            chapter_verses = []
            for v in raw_verses:
                v_text = v.get("Verse", "").strip() if isinstance(v, dict) else str(v)
                chapter_verses.append(v_text)
            standard_chapters.append(chapter_verses)

        if standard_chapters:
            standard_books.append({
                "name": b_name,
                "chapters": standard_chapters
            })

    return standard_books

def classify_scrollmapper_file(filename):
    stem = filename.replace('.json', '')
    if stem.startswith('Fre'): return 'fra', stem.lower()
    if stem.startswith('Ger'): return 'deu', stem.lower()
    if stem.startswith('Spa'): return 'spa', stem.lower()
    if stem.startswith('Por'): return 'por', stem.lower()
    if stem.startswith('Rus'): return 'rus', stem.lower()
    if stem.startswith('Chi'): return 'zho', stem.lower()
    if stem.startswith('Jap'): return 'jpn', stem.lower()
    if stem.startswith('Kor'): return 'kor', stem.lower()
    if stem.startswith('Dut') or stem.startswith('Nl'): return 'nld', stem.lower()
    if stem.startswith('Fin'): return 'fin', stem.lower()
    if stem.startswith('Swe'): return 'swe', stem.lower()
    if stem.startswith('Nor') or stem == 'Norsk': return 'nor', stem.lower()
    if stem.startswith('Da'): return 'dan', stem.lower()
    if stem.startswith('Cze'): return 'ces', stem.lower()
    if stem.startswith('Pol'): return 'pol', stem.lower()
    if stem.startswith('Hun'): return 'hun', stem.lower()
    if stem.startswith('Gre') or stem in ['Byz', 'TR', 'StatResGNT']: return 'ell', stem.lower()
    if stem.startswith('Heb') or stem in ['WLC']: return 'heb', stem.lower()
    if stem.startswith('Vulg') or stem == 'Vulgate': return 'lat', stem.lower()
    if stem.startswith('Ukr'): return 'ukr', stem.lower()
    if stem.startswith('Viet'): return 'vie', stem.lower()
    if stem.startswith('Thai'): return 'tha', stem.lower()
    if stem.startswith('Tag'): return 'tgl', stem.lower()
    if stem.startswith('Alb'): return 'alb', stem.lower()
    if stem.startswith('Arm'): return 'arm', stem.lower()
    if stem.startswith('Ceb'): return 'ceb', stem.lower()
    if stem.startswith('Cro'): return 'hrv', stem.lower()
    if stem.startswith('Sr'): return 'srp', stem.lower()
    if stem.startswith('Slo'): return 'slv', stem.lower()
    if stem == 'Haitian': return 'hat', stem.lower()
    if stem == 'Maori': return 'mri', stem.lower()
    if stem == 'Mal1910': return 'mal', stem.lower()
    if stem == 'Est': return 'est', stem.lower()
    if stem == 'Esperanto': return 'epo', stem.lower()
    if stem.startswith('Cop'): return 'cop', stem.lower()
    if stem.startswith('CSl'): return 'chu', stem.lower()
    if stem == 'Wulfila': return 'got', stem.lower()
    if stem == 'Peshitta': return 'syr', stem.lower()
    if stem == 'Che1860': return 'che', stem.lower()
    if stem == 'BurJudson': return 'mya', stem.lower()
    if stem == 'BeaMRK': return 'ita', stem.lower()
    if stem == 'ManxGaelic': return 'glv', stem.lower()
    if stem == 'MapM': return 'arn', stem.lower()
    if stem == 'Mg1865': return 'mlg', stem.lower()
    if stem == 'PohnOld': return 'pon', stem.lower()
    if stem == 'Tausug': return 'tsg', stem.lower()
    if stem == 'TpiKJPB': return 'tpi', stem.lower()
    if stem == 'vlsJoNT': return 'vls', stem.lower()
    if stem == 'sml_BL_2008': return 'sml', stem.lower()
    if stem == 'SP': return 'sam', stem.lower()
    return 'eng', stem.lower()

def process_all_scrollmapper():
    sm_json_dir = SOURCES_DIR / "bible_databases/formats/json"
    if not sm_json_dir.exists():
        print(f"Warning: {sm_json_dir} not found.")
        return 0

    count = 0
    for json_file in sorted(sm_json_dir.glob("*.json")):
        lang_code, tag = classify_scrollmapper_file(json_file.name)
        data = standardize_scrollmapper_json(json_file)
        if not data:
            continue

        out_name = f"{tag}.json"

        # Save to ~/.paraclea/bibles/<lang>/
        dest_dir = BIBLES_DIR / lang_code
        dest_dir.mkdir(parents=True, exist_ok=True)
        with open(dest_dir / out_name, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)

        # Save to project/bibles/<lang>/
        proj_dest_dir = PROJECT_BIBLES_DIR / lang_code
        proj_dest_dir.mkdir(parents=True, exist_ok=True)
        with open(proj_dest_dir / out_name, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)

        # Handle canonical KJV and WEB shortcuts
        if json_file.name == "KJV.json":
            with open(DATA_DIR / "kjv.json", 'w', encoding='utf-8') as f:
                json.dump(data, f, ensure_ascii=False, indent=2)
            with open(PROJECT_DATA_DIR / "kjv.json", 'w', encoding='utf-8') as f:
                json.dump(data, f, ensure_ascii=False, indent=2)
        elif json_file.name in ["Webster.json", "BSB.json"]:
            if not (DATA_DIR / "web.json").exists() or json_file.name == "Webster.json":
                with open(dest_dir / "web.json", 'w', encoding='utf-8') as f:
                    json.dump(data, f, ensure_ascii=False, indent=2)
                with open(proj_dest_dir / "web.json", 'w', encoding='utf-8') as f:
                    json.dump(data, f, ensure_ascii=False, indent=2)
                with open(DATA_DIR / "web.json", 'w', encoding='utf-8') as f:
                    json.dump(data, f, ensure_ascii=False, indent=2)
                with open(PROJECT_DATA_DIR / "web.json", 'w', encoding='utf-8') as f:
                    json.dump(data, f, ensure_ascii=False, indent=2)

        count += 1
    return count

def process_all_godlytalias():
    gt_dir = SOURCES_DIR / "Bible-Database"
    if not gt_dir.exists():
        return 0

    lang_code_map = {
        "Afrikaans": "afr",
        "Bengali": "ben",
        "Gujarati": "guj",
        "Hindi": "hin",
        "Hungarian": "hun",
        "Indonesian": "ind",
        "Kannada": "kan",
        "Malayalam": "mal",
        "Marathi": "mar",
        "Nepali": "nep",
        "Oriya": "ori",
        "Punjabi": "pan",
        "Sepedi": "nso",
        "Tamil": "tam",
        "Telugu": "tel",
        "Xhosa": "xho",
        "Zulu": "zul",
    }

    count = 0
    for folder_name, lang_code in lang_code_map.items():
        src_path = gt_dir / folder_name / "bible.json"
        if src_path.exists():
            data = standardize_godlytalias_json(src_path)
            if data:
                dest_dir = BIBLES_DIR / lang_code
                dest_dir.mkdir(parents=True, exist_ok=True)
                dest_file = dest_dir / f"{lang_code}_bible.json"
                with open(dest_file, 'w', encoding='utf-8') as f:
                    json.dump(data, f, ensure_ascii=False, indent=2)

                proj_dest_dir = PROJECT_BIBLES_DIR / lang_code
                proj_dest_dir.mkdir(parents=True, exist_ok=True)
                with open(proj_dest_dir / f"{lang_code}_bible.json", 'w', encoding='utf-8') as f:
                    json.dump(data, f, ensure_ascii=False, indent=2)

                count += 1
    return count

def create_sample_libraries():
    # 1. Psychology & Wellness
    psych_dir = LIBRARY_DIR / "psychology"
    psych_dir.mkdir(parents=True, exist_ok=True)
    psych_book = {
        "title": "Principles of Mind & Wellness",
        "author": "Paraclea Research",
        "category": "psychology",
        "chapters": [
            {
                "chapter_number": 1,
                "title": "The Architecture of Peace & Focus",
                "content": "True mental peace begins with daily cognitive reflection, emotional stillness, and structured contemplation. When the mind focuses on higher wisdom, anxiety naturally recedes."
            },
            {
                "chapter_number": 2,
                "title": "Habits, Memory & Cognition",
                "content": "Memory is an interconnected web of experiences and associations. Building habits of daily study and active note-taking strengthens neuroplasticity and long-term retention."
            }
        ]
    }
    with open(psych_dir / "principles_of_mind.json", 'w', encoding='utf-8') as f:
        json.dump(psych_book, f, indent=2)

    # 2. Survival & Preparedness
    surv_dir = LIBRARY_DIR / "survival"
    surv_dir.mkdir(parents=True, exist_ok=True)
    surv_book = {
        "title": "Emergency Preparedness & Bushcraft Manual",
        "author": "Field Preparedness Institute",
        "category": "survival",
        "chapters": [
            {
                "chapter_number": 1,
                "title": "Water Purification & Filtration",
                "content": "Access to clean water is the first priority in survival. Methods include boiling (rolling boil for 1-3 minutes), solar disinfection (SODIS), charcoal and sand filtration beds, and chemical purification tablets."
            },
            {
                "chapter_number": 2,
                "title": "Shelter & Thermal Regulation",
                "content": "Hypothermia and hyperthermia are rapid threats. Insulate yourself from the ground using dry leaves or branches, construct debris huts or lean-tos, and ensure wind and moisture barriers."
            },
            {
                "chapter_number": 3,
                "title": "Emergency Foraging & Plant Identification",
                "content": "Apply the Universal Edibility Test with caution. Avoid plants with white/yellow berries, milky sap, almond scent, or 3-leaved growth patterns unless verified."
            }
        ]
    }
    with open(surv_dir / "emergency_preparedness.json", 'w', encoding='utf-8') as f:
        json.dump(surv_book, f, indent=2)

    # 3. Medical First Aid
    med_dir = LIBRARY_DIR / "medical"
    med_dir.mkdir(parents=True, exist_ok=True)
    med_book = {
        "title": "Field Trauma & Emergency First Aid",
        "author": "Medical Field Corps",
        "category": "medical",
        "chapters": [
            {
                "chapter_number": 1,
                "title": "Hemorrhage Control & Tourniquets",
                "content": "Direct pressure is the initial response to external bleeding. For life-threatening extremity arterial bleeding, apply a commercial tourniquet 2-3 inches above the wound high and tight until bleeding stops."
            },
            {
                "chapter_number": 2,
                "title": "Airway Management & Recovery Position",
                "content": "Ensure patent airway using head-tilt chin-lift or jaw thrust for suspected spinal trauma. Place unresponsive breathing casualties in lateral recovery position to prevent aspiration."
            }
        ]
    }
    with open(med_dir / "field_first_aid.json", 'w', encoding='utf-8') as f:
        json.dump(med_book, f, indent=2)

    # 4. Ellen G. White Spiritual Works
    egw_dir = LIBRARY_DIR / "egw"
    egw_dir.mkdir(parents=True, exist_ok=True)
    egw_book = {
        "title": "Steps to Christ",
        "author": "Ellen G. White",
        "category": "egw",
        "chapters": [
            {
                "chapter_number": 1,
                "title": "God's Love for Man",
                "content": "Nature and revelation alike testify of God's love. Our Father in heaven is the source of life, of wisdom, and of joy. Look at the wonderful and beautiful things of nature. Think of their marvelous adaptation to the needs and happiness, not only of man, but of all living creatures. The sunshine and the rain, that gladden and refresh the earth, the hills and seas and plains, all speak to us of the Creator's love."
            },
            {
                "chapter_number": 2,
                "title": "The Sinner's Need of Christ",
                "content": "Man was originally endowed with noble powers and a well-balanced mind. He was perfect in his being, and in harmony with God. His thoughts were pure, his aims holy. But through disobedience, his powers were perverted, and selfishness took the place of love."
            },
            {
                "chapter_number": 3,
                "title": "Repentance",
                "content": "Repentance includes sorrow for sin and a turning away from it. We shall not renounce sin unless we see its sinfulness; until we turn away from it in heart, there will be no real change in the life."
            }
        ]
    }
    with open(egw_dir / "steps_to_christ.json", 'w', encoding='utf-8') as f:
        json.dump(egw_book, f, indent=2)

    # 5. Educational & Philosophy
    edu_dir = LIBRARY_DIR / "educational"
    edu_dir.mkdir(parents=True, exist_ok=True)
    edu_book = {
        "title": "Foundations of Natural Philosophy & Science",
        "author": "Classical Scholarship",
        "category": "educational",
        "chapters": [
            {
                "chapter_number": 1,
                "title": "Observation, Hypothesis & Scientific Inquiry",
                "content": "Knowledge advances through rigorous empirical observation, formulation of falsifiable hypotheses, deductive reasoning, and iterative experimental verification."
            },
            {
                "chapter_number": 2,
                "title": "Astronomy & the Structure of the Cosmos",
                "content": "From planetary orbits governed by celestial mechanics to stellar nucleosynthesis and galactic clusters, cosmological laws display remarkable symmetry and fine-tuning."
            }
        ]
    }
    with open(edu_dir / "foundations_of_philosophy.json", 'w', encoding='utf-8') as f:
        json.dump(edu_book, f, indent=2)

def main():
    print("🚀 Populating Complete Paraclea Multi-Language Bible & Library Databases...")
    ensure_dirs()
    sm_count = process_all_scrollmapper()
    gt_count = process_all_godlytalias()
    create_sample_libraries()
    total_versions = sm_count + gt_count
    print(f"✅ Successfully processed {total_versions} Bible versions ({sm_count} from scrollmapper, {gt_count} from Bible-Database)!")

if __name__ == "__main__":
    main()
