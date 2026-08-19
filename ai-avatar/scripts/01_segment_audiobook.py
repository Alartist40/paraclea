#!/usr/bin/env python3
"""
Segment the Alice in Wonderland audiobook into ~5-10 second clips.
Skips the intro (first 3 minutes) and extracts voice-acting segments.
Outputs to dataset/training_data/wavs/ in LJSpeech-compatible format.
"""

import subprocess
import os
import sys

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
AUDIO_FILE = os.path.join(PROJECT_ROOT, "..", "aliceinwonderland_01_gerstenberg_64kb.mp3")
OUTPUT_DIR = os.path.join(PROJECT_ROOT, "dataset", "training_data", "wavs")

# Skip first 3 min (intro/music), then take segments from 3min to end
# Segment length: 8 seconds with 1 second overlap
SEGMENT_SEC = 8
OVERLAP_SEC = 1
START_SKIP = 180  # Skip first 3 minutes

os.makedirs(OUTPUT_DIR, exist_ok=True)


def get_duration(path):
    result = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration",
         "-of", "default=noprint_wrappers=1:nokey=1", path],
        capture_output=True, text=True
    )
    return float(result.stdout.strip())


def main():
    if not os.path.exists(AUDIO_FILE):
        print(f"ERROR: Audio file not found: {AUDIO_FILE}")
        sys.exit(1)

    duration = get_duration(AUDIO_FILE)
    print(f"Audio duration: {duration:.1f}s")

    step = SEGMENT_SEC - OVERLAP_SEC
    idx = 0
    segments = []

    t = START_SKIP
    while t + SEGMENT_SEC <= duration:
        out_path = os.path.join(OUTPUT_DIR, f"alice_{idx:04d}.wav")
        cmd = [
            "ffmpeg", "-y", "-i", AUDIO_FILE,
            "-ss", str(t), "-t", str(SEGMENT_SEC),
            "-ar", "22050", "-ac", "1", "-acodec", "pcm_s16le",
            out_path
        ]
        subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        segments.append((idx, t, t + SEGMENT_SEC, out_path))
        idx += 1
        t += step

    print(f"Extracted {idx} segments ({SEGMENT_SEC}s each, {step}s step) from {START_SKIP}s to {duration:.0f}s")
    print(f"Output: {OUTPUT_DIR}")

    # Write a manifest for transcription
    manifest = os.path.join(PROJECT_ROOT, "dataset", "training_data", "manifest.txt")
    os.makedirs(os.path.dirname(manifest), exist_ok=True)
    with open(manifest, "w") as f:
        for idx, start, end, path in segments:
            f.write(f"{os.path.basename(path)}\t{start:.1f}\t{end:.1f}\n")
    print(f"Manifest written to: {manifest}")


if __name__ == "__main__":
    main()
