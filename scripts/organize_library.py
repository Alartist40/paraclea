#!/usr/bin/env python3
"""
Paraclea Complete Multi-Language Bible & Library Standardization Engine
Aggregates, deduplicates, and converts 200+ Bible versions (including African languages like Twi, Zulu, Xhosa, Afrikaans, Swahili, Amharic, Hausa, Yoruba) and non-scripture book categories (EGW, Medical, Survival, Classics) into `$HOME/.paraclea/`.
"""

import os
import sys
import json
import glob
import hashlib
import re
from pathlib import Path

HOME = Path.home()
PARACLEA_DIR = HOME / ".paraclea"
BIBLES_DIR = PARACLEA_DIR / "bibles"
LIBRARY_DIR = PARACLEA_DIR / "library"
REF_DIR = HOME / "Documents/reference"

ISO_LANG_MAP = {
    "english": "eng", "en": "eng", "kjv": "eng", "niv": "eng", "nlt": "eng", "esv": "eng", "nasb": "eng", "nkjv": "eng", "asv": "eng", "web": "eng",
    "spanish": "spa", "es": "spa", "rv1909": "spa", "rv": "spa", "reina": "spa",
    "french": "fra", "fr": "fra", "lsg": "fra", "segond": "fra",
    "german": "deu", "de": "deu", "luther": "deu",
    "chinese": "zho", "zh": "zho", "cuv": "zho", "ncv": "zho",
    "arabic": "arb", "ar": "arb",
    "hindi": "hin", "hi": "hin",
    "bengali": "ben", "bn": "ben",
    "telugu": "tel", "te": "tel",
    "tamil": "tam", "ta": "tam",
    "kannada": "kan", "kn": "kan",
    "malayalam": "mal", "ml": "mal",
    "gujarati": "guj", "gu": "guj",
    "punjabi": "pan", "pa": "pan",
    "zulu": "zul", "zu": "zul",
    "xhosa": "xho", "xh": "xho",
    "afrikaans": "afr", "af": "afr",
    "sepedi": "nso", "nso": "nso",
    "nepali": "nep", "ne": "nep",
    "hungarian": "hun", "hu": "hun",
    "twi": "twi", "tw": "twi", "asante": "twi", "akuapem": "twi",
    "amharic": "amh", "am": "amh",
    "swahili": "swh", "sw": "swh",
    "portuguese": "por", "pt": "por",
    "russian": "rus", "ru": "rus",
    "latin": "lat", "vulg": "lat",
    "greek": "ell", "el": "ell", "tr": "ell",
    "hebrew": "heb", "he": "heb", "wlc": "heb",
    "hausa": "hau", "ha": "hau",
    "igbo": "ibo", "ig": "ibo",
    "yoruba": "yor", "yo": "yor",
    "shona": "sna", "sn": "sna",
    "setswana": "tsn", "tn": "tsn",
    "luganda": "lug", "lg": "lug",
}

processed_hashes = set()
processed_count = 0
lang_summary = {}

def ensure_dirs():
    BIBLES_DIR.mkdir(parents=True, exist_ok=True)
    LIBRARY_DIR.mkdir(parents=True, exist_ok=True)

def parse_lang_code(text):
    text_clean = re.sub(r'[^a-zA-Z]', ' ', text).lower()
    for word in text_clean.split():
        if word in ISO_LANG_MAP:
            return ISO_LANG_MAP[word]
    # Check 2-letter prefix if available
    prefix = text_clean[:2] if len(text_clean) >= 2 else text_clean
    return ISO_LANG_MAP.get(prefix, "eng")

def save_bible(lang_code, tag_name, books_data):
    global processed_count
    if not books_data:
        return False
    
    # Calculate dataset signature hash for deduplication
    sig = hashlib.md5(json.dumps(books_data[:2], sort_keys=True).encode('utf-8')).hexdigest()
    if sig in processed_hashes:
        return False
    processed_hashes.add(sig)

    lang_dir = BIBLES_DIR / lang_code
    lang_dir.mkdir(parents=True, exist_ok=True)
    
    clean_tag = re.sub(r'[^a-zA-Z0-9_-]', '_', tag_name).lower()
    out_file = lang_dir / f"{clean_tag}.json"
    
    with open(out_file, 'w', encoding='utf-8') as f:
        json.dump(books_data, f, ensure_ascii=False, indent=2)
    
    processed_count += 1
    lang_summary[lang_code] = lang_summary.get(lang_code, 0) + 1
    print(f"  ✓ Processed Bible [{lang_code.upper()}]: {clean_tag} ({len(books_data)} books)")
    return True

def convert_bible_api_repo():
    print("📦 Inspecting wldeh/bible-api (200+ multi-language versions)...")
    bibles_dir = REF_DIR / "bible-api" / "bibles"
    if not bibles_dir.exists():
        return

    for b_dir in bibles_dir.glob("*"):
        if not b_dir.is_dir():
            continue
        
        tag = b_dir.name
        lang_code = parse_lang_code(tag)
        books_dir = b_dir / "books"
        
        if not books_dir.exists():
            continue

        books = []
        for bk_dir in sorted(books_dir.glob("*")):
            if not bk_dir.is_dir():
                continue
            bk_name = bk_dir.name
            ch_dir = bk_dir / "chapters"
            if not ch_dir.exists():
                continue
            
            chapters = []
            ch_files = sorted(ch_dir.glob("*.json"), key=lambda p: int(p.stem) if p.stem.isdigit() else 999)
            for ch_file in ch_files:
                try:
                    with open(ch_file, 'r', encoding='utf-8') as f:
                        ch_json = json.load(f)
                    verse_texts = []
                    if "data" in ch_json and isinstance(ch_json["data"], list):
                        for item in ch_json["data"]:
                            if "text" in item:
                                verse_texts.append(item["text"])
                    if verse_texts:
                        chapters.append(verse_texts)
                except Exception:
                    pass
            
            if chapters:
                books.append({"name": bk_name.capitalize(), "chapters": chapters})
        
        if books:
            save_bible(lang_code, tag, books)

def convert_godlytalias_repo():
    print("📦 Inspecting godlytalias/Bible-Database (African & Asian languages)...")
    repo_dir = REF_DIR / "Bible-Database"
    if not repo_dir.exists():
        return

    for json_file in repo_dir.glob("**/*.json"):
        folder_name = json_file.parent.name
        lang_code = parse_lang_code(folder_name)
        tag = f"{folder_name.lower()}_bible"
        
        try:
            with open(json_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
            
            books = []
            if isinstance(data, list):
                # Standard array format
                for bk in data:
                    if isinstance(bk, dict) and "name" in bk and "chapters" in bk:
                        books.append(bk)
            elif isinstance(data, dict):
                for bk_name, chapters_val in data.items():
                    if isinstance(chapters_val, list):
                        books.append({"name": bk_name, "chapters": chapters_val})
            
            if books:
                save_bible(lang_code, tag, books)
        except Exception:
            pass

def convert_bible_databases_repo():
    print("📦 Inspecting scrollmapper/bible_databases...")
    repo_dir = REF_DIR / "bible_databases"
    if not repo_dir.exists():
        return

    for json_file in repo_dir.glob("**/*.json"):
        if "cross" in json_file.name.lower() or "key" in json_file.name.lower():
            continue
        lang_code = parse_lang_code(json_file.stem)
        try:
            with open(json_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
            if isinstance(data, list) and len(data) > 0 and isinstance(data[0], dict):
                save_bible(lang_code, json_file.stem, data)
        except Exception:
            pass

def parse_full_egw_books():
    print("📦 Organizing Ellen G. White Spiritual Library...")
    import parse_egw
    parse_egw.parse_steps_to_christ()
    parse_egw.parse_the_great_controversy()
    parse_egw.parse_education()
    parse_egw.parse_the_desire_of_ages()

def parse_survival_manuals():
    print("📦 Organizing Survival & Medical Field Manuals...")
    surv_dir = LIBRARY_DIR / "survival"
    med_dir = LIBRARY_DIR / "medical"
    surv_dir.mkdir(parents=True, exist_ok=True)
    med_dir.mkdir(parents=True, exist_ok=True)

    wiki_md_dir = REF_DIR / "SurvivalManual/android/src/main/assets/md"
    if wiki_md_dir.exists():
        md_files = sorted(wiki_md_dir.glob("*.md"))
        surv_chapters = []
        med_chapters = []
        c_num = 1
        m_num = 1

        for f in md_files:
            if f.stem in ["Home", "Credits", "Apps", "FAQ", "_Footer"]:
                continue
            content = f.read_text(encoding='utf-8', errors='ignore').strip()
            if not content:
                continue
            
            clean_title = f.stem.replace('_', ' ')
            clean_title = re.sub(r'([a-z])([A-Z])', r'\1 \2', clean_title)

            if f.stem in ["Medicine", "DangerousArthropods", "Poisonous-Plants"]:
                med_chapters.append({
                    "chapter_number": m_num,
                    "title": clean_title,
                    "content": content
                })
                m_num += 1
            else:
                surv_chapters.append({
                    "chapter_number": c_num,
                    "title": clean_title,
                    "content": content
                })
                c_num += 1

        if surv_chapters:
            surv_book = {
                "title": "Libre Survival & Bushcraft Manual (FM 21-76)",
                "author": "US Army & SurvivalManual Contributors",
                "category": "survival",
                "chapters": surv_chapters
            }
            with open(surv_dir / "libre_survival_manual.json", 'w', encoding='utf-8') as f:
                json.dump(surv_book, f, ensure_ascii=False, indent=2)
            print(f"  ✓ Processed Survival Manual ({len(surv_chapters)} full chapters)")

        if med_chapters:
            med_book = {
                "title": "Field Trauma & Emergency First Aid Manual",
                "author": "Medical Field Corps & Survival Contributors",
                "category": "medical",
                "chapters": med_chapters
            }
            with open(med_dir / "field_first_aid.json", 'w', encoding='utf-8') as f:
                json.dump(med_book, f, ensure_ascii=False, indent=2)
            print(f"  ✓ Processed Medical Field Manual ({len(med_chapters)} full chapters)")

def main():
    print("🚀 Starting Paraclea Complete Multi-Language Bible & Multi-Category Library Standardization...")
    ensure_dirs()
    convert_godlytalias_repo()
    convert_bible_api_repo()
    convert_bible_databases_repo()
    parse_full_egw_books()
    parse_survival_manuals()

    print("\n" + "="*60)
    print(f"🎉 Paraclea Database Standardization Complete!")
    print(f"   • Total Unique Bible Versions Formatted: {processed_count}")
    print(f"   • Languages Covered: {len(lang_summary)}")
    for l_code, count in sorted(lang_summary.items()):
        print(f"     - [{l_code.upper()}]: {count} versions")
    print(f"   • Storage Directory: {BIBLES_DIR}")
    print("="*60)

if __name__ == "__main__":
    main()
