import os, re, json

EGW_SRC_DIR = '/home/orangepi/Documents/reference/EGW'
EGW_DEST_DIR = os.path.expanduser('~/.paraclea/library/egw')

os.makedirs(EGW_DEST_DIR, exist_ok=True)

def clean_ocr_text(text):
    # Remove running header page numbers and headers like "142 STEPS TO CHRIST." or "STEPS TO CHRIST."
    text = re.sub(r'\b\d+\s+STEPS\s+TO\s+CHRIST\b\.?', '', text, flags=re.IGNORECASE)
    text = re.sub(r'\bSTEPS\s+TO\s+CHRIST\b\.?', '', text, flags=re.IGNORECASE)
    text = re.sub(r'\b\d+\s+THE\s+DESIRE\s+OF\s+AGES\b\.?', '', text, flags=re.IGNORECASE)
    text = re.sub(r'\bTHE\s+DESIRE\s+OF\s+AGES\b\.?', '', text, flags=re.IGNORECASE)
    text = re.sub(r'\b\d+\s+THE\s+GREAT\s+CONTROVERSY\b\.?', '', text, flags=re.IGNORECASE)
    text = re.sub(r'\bTHE\s+GREAT\s+CONTROVERSY\b\.?', '', text, flags=re.IGNORECASE)
    
    # Remove OCR symbol noise
    text = re.sub(r'\^\^[A-Za-z0-9]+\^', '', text)
    text = re.sub(r'\^[a-z0-9]+\b', '', text)
    
    # Fix hyphenated line breaks (e.g. "expe- \n rience" -> "experience")
    text = re.sub(r'(\b[a-zA-Z]+)-\s*\n\s*([a-zA-Z]+\b)', r'\1\2', text)
    
    # Replace multiple spaces with a single space
    text = re.sub(r'[ \t]+', ' ', text)
    
    lines = [l.strip() for l in text.split('\n')]
    lines = [l for l in lines if l]
    return '\n\n'.join(lines).strip()

def parse_steps_to_christ():
    fpath = os.path.join(EGW_SRC_DIR, 'steps_to_christ.txt')
    with open(fpath, 'r', encoding='utf-8', errors='ignore') as fp:
        lines = fp.readlines()
        
    # Full body text is between line 200 and line 5125
    body_lines = lines[200:5125]
    full_text = ''.join(body_lines)
    
    chapter_headers = [
        "GOD'S LOVE FOR MAN",
        "THE SINNER'S NEED OF CHRIST",
        "REPENTANCE",
        "CONFESSION",
        "CONSECRATION",
        "FAITH AND ACCEPTANCE",
        "THE TEST OF DISCIPLESHIP",
        "GROWING UP INTO CHRIST",
        "THE WORK AND THE LIFE",
        "A KNOWLEDGE OF GOD",
        "THE PRIVILEGE OF PRAYER",
        "WHAT TO DO WITH DOUBT",
        "REJOICING IN THE LORD"
    ]
    
    positions = []
    for idx, h in enumerate(chapter_headers):
        # Match header line with loose spacing
        words = h.split()
        pat = r'^\s*' + r'\s+'.join(re.escape(w) for w in words) + r'\b.*$'
        m = re.search(pat, full_text, re.IGNORECASE | re.MULTILINE)
        if m:
            positions.append((m.start(), h.title()))
        else:
            print(f"ERROR: Steps to Christ header missing: {h}")
            
    positions.sort(key=lambda x: x[0])
    
    chapters = []
    for i in range(len(positions)):
        start_pos, title = positions[i]
        end_pos = positions[i+1][0] if i+1 < len(positions) else len(full_text)
        raw_content = full_text[start_pos:end_pos]
        content_clean = clean_ocr_text(raw_content)
        chapters.append({
            "chapter_number": i + 1,
            "title": f"Chapter {i+1}: {title}",
            "content": content_clean
        })
        
    book_json = {
        "title": "Steps to Christ",
        "author": "Ellen G. White",
        "category": "egw",
        "chapters": chapters
    }
    
    out_path = os.path.join(EGW_DEST_DIR, 'steps_to_christ.json')
    with open(out_path, 'w', encoding='utf-8') as fp:
        json.dump(book_json, fp, indent=2)
    print(f"✓ Created {out_path} ({len(chapters)} complete chapters)")

def parse_the_great_controversy():
    fpath = os.path.join(EGW_SRC_DIR, 'the_great_controversy.txt')
    with open(fpath, 'r', encoding='utf-8', errors='ignore') as fp:
        text = fp.read()
        
    start_match = re.search(r'1\.\s+THE\s+DESTRUCTION\s+OF\s+JERUSALEM\.', text)
    if not start_match:
        print("ERROR: Could not find start of Great Controversy body")
        return
        
    body_text = text[start_match.start():]
    chapter_regex = re.compile(r'^\s*(\d{1,2})\.\s+([“\"A-Z\s\’\'-]+)\.\s*$', re.MULTILINE)
    
    matches = list(chapter_regex.finditer(body_text))
    chapters = []
    
    for i in range(len(matches)):
        m = matches[i]
        ch_num = int(m.group(1))
        ch_raw_title = m.group(2).strip().replace('“', '').replace('”', '').replace('"', '').title()
        
        c_start = m.end()
        c_end = matches[i+1].start() if i+1 < len(matches) else len(body_text)
        
        raw_content = body_text[c_start:c_end]
        content_clean = clean_ocr_text(raw_content)
        
        chapters.append({
            "chapter_number": ch_num,
            "title": f"Chapter {ch_num}: {ch_raw_title}",
            "content": content_clean
        })
        
    book_json = {
        "title": "The Great Controversy",
        "author": "Ellen G. White",
        "category": "egw",
        "chapters": chapters
    }
    
    out_path = os.path.join(EGW_DEST_DIR, 'the_great_controversy.json')
    with open(out_path, 'w', encoding='utf-8') as fp:
        json.dump(book_json, fp, indent=2)
    print(f"✓ Created {out_path} ({len(chapters)} complete chapters)")

def parse_education():
    fpath = os.path.join(EGW_SRC_DIR, 'education.txt')
    with open(fpath, 'r', encoding='utf-8', errors='ignore') as fp:
        lines = fp.readlines()
        
    body_lines = lines[200:9610]
    full_text = ''.join(body_lines)
    
    # Matches lines like "_Source and Aim of True Education_" or "_The Eden School_"
    chapter_regex = re.compile(r'^\s*_([A-Za-z0-9\s\,\-\’\']+_\s*$)', re.MULTILINE)
    
    # Filter out category titles like _FIRST PRINCIPLES_ or _ILLUSTRATIONS_
    skip_titles = {'_FIRST PRINCIPLES_', '_ILLUSTRATIONS_', '_THE MASTER TEACHER_', '_NATURE TEACHING_', '_THE BIBLE AS AN EDUCATOR_', '_PHYSICAL CULTURE_', '_CHARACTER-BUILDING_', '_THE UNDER-TEACHER_', '_THE HIGHER COURSE_'}
    
    matches = []
    for m in chapter_regex.finditer(full_text):
        raw_title = m.group(1).strip()
        if raw_title not in skip_titles:
            clean_title = raw_title.strip('_').strip()
            matches.append((m.start(), clean_title))
            
    chapters = []
    for i in range(len(matches)):
        c_start, clean_title = matches[i]
        c_end = matches[i+1][0] if i+1 < len(matches) else len(full_text)
        
        raw_content = full_text[c_start:c_end]
        content_clean = clean_ocr_text(raw_content)
        
        chapters.append({
            "chapter_number": i + 1,
            "title": f"Chapter {i+1}: {clean_title}",
            "content": content_clean
        })
        
    book_json = {
        "title": "Education",
        "author": "Ellen G. White",
        "category": "egw",
        "chapters": chapters
    }
    
    out_path = os.path.join(EGW_DEST_DIR, 'education.json')
    with open(out_path, 'w', encoding='utf-8') as fp:
        json.dump(book_json, fp, indent=2)
    print(f"✓ Created {out_path} ({len(chapters)} complete chapters)")

def parse_the_desire_of_ages():
    fpath = os.path.join(EGW_SRC_DIR, 'the_desire_of_ages.txt')
    with open(fpath, 'r', encoding='utf-8', errors='ignore') as fp:
        lines = fp.readlines()
        
    body_lines = lines[455:40200]
    full_text = ''.join(body_lines)
    
    chapter_regex = re.compile(r'^\s*([«\s]*CHAPTER\s+[A-Z\-\s]+[\.,]?)\s*$', re.MULTILINE)
    
    matches = list(chapter_regex.finditer(full_text))
    chapters = []
    
    for i in range(len(matches)):
        m = matches[i]
        
        c_start = m.start()
        c_end = matches[i+1].start() if i+1 < len(matches) else len(full_text)
        
        raw_content = full_text[c_start:c_end]
        raw_lines = [l.strip() for l in raw_content.split('\n') if l.strip()]
        
        ch_title_str = f"Chapter {i+1}"
        if len(raw_lines) > 1:
            title_line = raw_lines[1]
            if not title_line.startswith("This chapter is based on"):
                title_line = re.sub(r'[\._\-]+$', '', title_line).strip()
                ch_title_str = f"Chapter {i+1}: {title_line}"
                
        content_clean = clean_ocr_text(raw_content)
        
        chapters.append({
            "chapter_number": i + 1,
            "title": ch_title_str,
            "content": content_clean
        })
        
    book_json = {
        "title": "The Desire of Ages",
        "author": "Ellen G. White",
        "category": "egw",
        "chapters": chapters
    }
    
    out_path = os.path.join(EGW_DEST_DIR, 'desire_of_ages.json')
    with open(out_path, 'w', encoding='utf-8') as fp:
        json.dump(book_json, fp, indent=2)
    print(f"✓ Created {out_path} ({len(chapters)} complete chapters)")

if __name__ == '__main__':
    print("Parsing and standardizing Ellen G. White library...")
    parse_steps_to_christ()
    parse_the_great_controversy()
    parse_education()
    parse_the_desire_of_ages()
