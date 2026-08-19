#!/usr/bin/env python3
"""
Transcribe all wav clips using whisper.cpp.
Outputs transcript files next to each wav.
"""

import subprocess
import os
import sys
import glob

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WHISPER_BIN = "/tmp/whisper.cpp/build/bin/whisper-cli"
WHISPER_MODEL = os.path.join(PROJECT_ROOT, "models", "ggml-tiny.bin")
WAV_DIR = os.path.join(PROJECT_ROOT, "dataset", "training_data", "wavs")


def transcribe(wav_path):
    result = subprocess.run(
        [WHISPER_BIN, "-m", WHISPER_MODEL, "-f", wav_path,
         "-l", "en", "--no-timestamps", "-nt"],
        capture_output=True, text=True
    )
    # whisper-cli outputs transcript on stderr sometimes, sometimes stdout
    text = result.stdout.strip() or result.stderr.strip()
    # Clean up whisper output (remove [00:00:00.000 -> ...] lines if any)
    lines = [l for l in text.splitlines() if not l.strip().startswith("[") and l.strip()]
    return " ".join(lines).strip()


def main():
    if not os.path.exists(WHISPER_BIN):
        print(f"ERROR: whisper-cli not found at {WHISPER_BIN}")
        print("Build whisper.cpp first: cd /tmp/whisper.cpp && cmake -B build && cmake --build build -j$(nproc)")
        sys.exit(1)

    if not os.path.exists(WHISPER_MODEL):
        print(f"ERROR: Whisper model not found: {WHISPER_MODEL}")
        sys.exit(1)

    wav_files = sorted(glob.glob(os.path.join(WAV_DIR, "*.wav")))
    if not wav_files:
        print(f"ERROR: No wav files found in {WAV_DIR}")
        print("Run 01_segment_audiobook.py first.")
        sys.exit(1)

    print(f"Transcribing {len(wav_files)} clips...")
    for wav_path in wav_files:
        txt_path = wav_path.replace(".wav", ".txt")
        if os.path.exists(txt_path) and os.path.getsize(txt_path) > 0:
            continue  # Already transcribed

        text = transcribe(wav_path)
        with open(txt_path, "w") as f:
            f.write(text)
        print(f"  {os.path.basename(wav_path)} -> \"{text[:60]}...\"" if len(text) > 60 else f"  {os.path.basename(wav_path)} -> \"{text}\"")

    print("\nTranscription complete!")


if __name__ == "__main__":
    main()
