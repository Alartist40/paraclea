#!/usr/bin/env python3
"""
Build LJSpeech-format metadata.csv from transcribed wav clips.
"""

import os
import glob

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WAV_DIR = os.path.join(PROJECT_ROOT, "dataset", "training_data", "wavs")
META_PATH = os.path.join(PROJECT_ROOT, "dataset", "training_data", "metadata.csv")


def main():
    wav_files = sorted(glob.glob(os.path.join(WAV_DIR, "*.wav")))
    entries = []

    for wav_path in wav_files:
        txt_path = wav_path.replace(".wav", ".txt")
        if not os.path.exists(txt_path):
            continue
        with open(txt_path, "r") as f:
            transcript = f.read().strip()
        if not transcript:
            continue
        # LJSpeech format: filename|transcript|normalized_transcript
        basename = os.path.basename(wav_path)
        entries.append(f"{basename}|{transcript}|{transcript}")

    with open(META_PATH, "w") as f:
        f.write("\n".join(entries))

    print(f"Wrote {len(entries)} entries to {META_PATH}")
    print(f"Dataset ready for Piper training!")


if __name__ == "__main__":
    main()
