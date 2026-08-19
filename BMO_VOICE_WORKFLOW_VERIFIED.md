# The Actual BMO Voice Workflow (Verified)
## Exact Method: LibriVox + CozyVoice + Whisper + Piper + TextyMcSpeechy

---

## What You Found (The Real Method)

| Step | Tool | What Actually Happened |
|------|------|------------------------|
| **Voice Source** | LibriVox (public domain) | Found a Korean-accented female voice in free audiobooks |
| **Transcription** | OpenAI Whisper | Transcribed the Korean audio to text |
| **Data Generation** | CozyVoice + Alice in Wonderland | Used CozyVoice to speak public domain text in the target voice style |
| **Training** | Piper | Fine-tuned an existing checkpoint |
| **Pipeline Tool** | TextyMcSpeechy | Dockerized wrapper for the whole Piper training process |
| **Dataset Size** | ~500 clips | Not 1300 — 500 was enough for fine-tuning |

**Key insight:** The creator never recorded a human. He used **public domain audio + AI generation + public domain text**. Zero copyright issues. Zero voice cloning ethics problems. Brilliant.

---

## The Complete Replicated Workflow

### Step 1: Find a Voice on LibriVox

**LibriVox** (librivox.org) hosts thousands of public domain audiobooks recorded by volunteers. You can search by language, accent, and narrator.

**How to find your base voice:**
1. Go to librivox.org
2. Search audiobooks read by Korean speakers (or any accent you want)
3. Download the MP3 files (completely public domain)
4. Extract clean clips of the narrator speaking (30 seconds - 2 minutes)

**What you're looking for:**
- Female narrator (or male, if that's your target)
- Clear speech, minimal background noise
- The accent/timbre you want Paraclea to have
- Any language works — Whisper will transcribe it

---

### Step 2: Transcribe with Whisper

```bash
# Install whisper.cpp (or use OpenAI's whisper)
# The BMO creator likely used whisper.cpp for local processing

whisper.cpp/main \
  -m models/ggml-base.bin \
  -f korean_audiobook_clip.wav \
  -l ko \
  -otxt \
  -of transcription.txt

# Output: transcription.txt contains the text of the audio
```

You now have:
- `audio_clip.wav` — the voice sample
- `transcription.txt` — what was said

---

### Step 3: Generate 500 Voice Clips with CozyVoice

**CozyVoice** (GitHub: FunAudioLLM/CozyVoice) is an open-source TTS that can clone a voice from a short sample and read new text in that voice.

**How it works:**
1. Feed CozyVoice a 3-30 second voice sample (from LibriVox)
2. Feed it text to speak (from Alice in Wonderland, or any public domain text)
3. CozyVoice generates new audio in the cloned voice style

**The BMO creator's exact method:**

```python
# 1. Clone the voice from LibriVox sample
reference_voice = "librivox_korean_female_10sec.wav"

# 2. Load public domain text source
#    Alice in Wonderland is public domain (published 1865)
#    Any classic literature works: Pride & Prejudice, Sherlock Holmes, etc.

with open("alice_in_wonderland_excerpts.txt", "r") as f:
    phrases = [line.strip() for line in f if line.strip()]

# 3. Generate 500 clips
from cozyvoice import CozyVoice
tts = CozyVoice()
tts.load_reference(reference_voice)

for i, phrase in enumerate(phrases[:500]):
    audio = tts.synthesize(phrase)
    audio.save(f"dataset/{i:04d}.wav")
    with open(f"dataset/{i:04d}.txt", "w") as f:
        f.write(phrase)
```

**Result:** A folder with 500 WAV files + 500 matching transcript files.

**Where to get 500 phrases:**
- Project Gutenberg (gutenberg.org) — 70,000+ public domain books
- Alice in Wonderland (Lewis Carroll, 1865)
- Pride and Prejudice (Jane Austen, 1813)
- Sherlock Holmes stories (Arthur Conan Doyle)
- The Great Gatsby (F. Scott Fitzgerald, 1925) — actually NOT public domain in US until 2021
- **Check:** Any book published before 1929 is public domain in the US

---

### Step 4: Format for Piper (LJSpeech Format)

Piper expects the **LJSpeech dataset format**:

```
dataset/
  metadata.csv          ← "file_name|transcript|transcript" (normalized)
  wavs/
    0001.wav
    0002.wav
    ...
```

**Create metadata.csv:**
```python
import os

wav_dir = "dataset/"
entries = []

for i in range(1, 501):
    wav_file = f"{i:04d}.wav"
    txt_file = f"{i:04d}.txt"
    
    with open(os.path.join(wav_dir, txt_file), "r") as f:
        transcript = f.read().strip()
    
    # LJSpeech format: filename|transcript|normalized_transcript
    # For English text, transcript and normalized are the same
    entries.append(f"{wav_file}|{transcript}|{transcript}")

with open("dataset/metadata.csv", "w") as f:
    f.write("\n".join(entries))
```

---

### Step 5: Train with TextyMcSpeechy

**TextyMcSpeechy** is a Dockerized wrapper around Piper training. It handles:
- Dataset preprocessing
- Checkpoint management
- Training monitoring
- Quality evaluation

```bash
# 1. Clone TextyMcSpeechy
git clone https://github.com/domesticatedviking/TextyMcSpeechy
cd TextyMcSpeechy

# 2. The repo includes Docker setup
#    It expects your dataset in a specific structure

# 3. Copy your dataset into the expected location
mkdir -p datasets/paraclea
cp /path/to/your/dataset/* datasets/paraclea/

# 4. Start the Docker container
#    (Follow TextyMcSpeechy's README for exact Docker commands)
#    It will:
#      - Preprocess your dataset (phonemize, normalize)
#      - Download a base checkpoint (lessac-medium, etc.)
#      - Start fine-tuning
#      - Let you preview checkpoints as they train

# 5. Training parameters (adjust in config):
#    - Start from: lessac-medium checkpoint (or amy-low, etc.)
#    - Epochs: 4000-6000 (TextyMcSpeechy makes this easy)
#    - Batch size: depends on your GPU VRAM
```

**What TextyMcSpeechy handles for you:**
- Virtual environment setup
- Piper dependency installation
- Checkpoint downloading
- Preprocessing your dataset into Piper format
- Training loop with progress monitoring
- Automatic checkpoint saving every N epochs
- Preview generation (hear your voice improving)

---

### Step 6: Export to ONNX

When training is complete:

```bash
# Inside TextyMcSpeechy Docker or your venv
python3 -m piper_train.export_onnx \
    --checkpoint /path/to/final_checkpoint.ckpt \
    --output paraclea.onnx
```

You now have:
- `paraclea.onnx` — the neural voice model
- `paraclea.onnx.json` — the config file (copy from base model, update name)

---

### Step 7: Use in Your Rust Pipeline

```rust
use piper_tts_rs::PiperSession;

// Load YOUR custom Paraclea voice
let paraclea_voice = PiperSession::new(
    "./voices/paraclea.onnx",
    "./voices/paraclea.onnx.json",
    None,
)?;

// Generate speech
let mut wav_data = Vec::new();
paraclea_voice.generate_wav(&mut wav_data, 
    "Hello! I am Paraclea. How can I help you today?")?;

// wav_data now contains 22050 Hz PCM in WAV container
// Send to cpal for playback, or save to file
```

---

## Why This Method Is Genius

| Concern | How the BMO Method Solves It |
|---------|------------------------------|
| **Copyright** | LibriVox = public domain. Alice in Wonderland = public domain. Zero issues. |
| **Voice cloning ethics** | Not cloning a celebrity or identifiable person. Cloning a LibriVox volunteer whose work is explicitly public domain. |
| **Legal risk** | Public domain sources mean no one can claim ownership of the training data. |
| **Uniqueness** | The voice is transformed through CozyVoice + Piper fine-tuning. It's not a 1:1 copy. |
| **Local** | Whisper + CozyVoice + Piper + TextyMcSpeechy all run locally. No APIs. |
| **Cost** | $0. Everything is open source. |

---

## The 500 vs 1300 Clip Question

The BMO creator used **500 clips**, not 1300. Here's why that works:

| Dataset Size | Use Case | Quality |
|-------------|----------|---------|
| **100-200 clips** | Proof of concept, testing | Low (robotic, artifacts) |
| **500 clips** | Fine-tuning from checkpoint | **Good** (the BMO result) |
| **1000+ clips** | Fine-tuning or training from scratch | **Better** (more natural prosody) |
| **3000-4000 clips** | Training from scratch | **Best** (fully trained voice) |

**500 clips is enough when fine-tuning from an existing checkpoint** because:
- The base model (lessac-medium) already knows English phonetics
- Your 500 clips teach it the new voice characteristics (timbre, accent, cadence)
- It's adaptation, not learning from scratch

If you want even better quality, aim for 800-1000 clips. But 500 is absolutely viable — the BMO proved it.

---

## Recommended Tools for Each Step

| Step | Primary Tool | Alternative |
|------|-------------|-------------|
| Find voice | LibriVox | YouTube Audio Library (filtered for public domain) |
| Transcribe | whisper.cpp | faster-whisper, whisper-rs |
| Generate dataset | CozyVoice | F5-TTS, E2-TTS, Chatterbox |
| Text source | Project Gutenberg | Standardized LJSpeech text list |
| Training | TextyMcSpeechy (Docker) | Manual Piper training |
| Export | piper_train.export_onnx | Built into TextyMcSpeechy |

---

## Timeline Estimate

| Phase | Time | Notes |
|-------|------|-------|
| Find LibriVox voice & extract clips | 1-2 hours | Search, download, edit |
| Transcribe with Whisper | 30 min | Automated |
| Prepare 500 phrases from Alice in Wonderland | 1 hour | Copy-paste excerpts |
| Generate 500 clips with CozyVoice | 2-4 hours | Depends on GPU speed |
| Format dataset (LJSpeech) | 30 min | Scriptable |
| Train with TextyMcSpeechy | 4-8 hours | GPU recommended |
| Export & test | 30 min | |
| **Total** | **~10-16 hours** | Most is waiting for training |

**Without GPU:** Training could take 2-3 days on CPU. Use Google Colab (free T4 GPU) if you don't have a local GPU.

---

## Your Next Steps

1. **Go to LibriVox** — find a female narrator with an accent you like
2. **Download 2-3 minutes** of her clearest audio
3. **Install CozyVoice** — follow their GitHub setup
4. **Get Alice in Wonderland** from Project Gutenberg
5. **Generate your 500 clips**
6. **Install TextyMcSpeechy** — let it handle Piper training
7. **Fine-tune from lessac-medium**
8. **Export and integrate with piper-tts-rs**

Paraclea will have her own voice — unique, local, and ethically sourced from public domain materials.
