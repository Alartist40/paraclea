#!/usr/bin/env python3
"""
Simple TTS inference script using Piper.
Usage: python tts_speak.py "Hello world" [-o output.wav]
"""

import argparse
import os
import sys

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MODEL = os.path.join(PROJECT_ROOT, "models", "en_US-lessac-medium.onnx")
CONFIG = os.path.join(PROJECT_ROOT, "models", "en_US-lessac-medium.onnx.json")


def speak(text, output_path="/tmp/piper_output.wav"):
    import subprocess
    cmd = [
        "piper",
        "--model", MODEL,
        "--config", CONFIG,
        "--output_file", output_path,
    ]
    proc = subprocess.run(cmd, input=text.encode(), capture_output=True)
    if proc.returncode != 0:
        print(f"Error: {proc.stderr.decode()}")
        sys.exit(1)
    print(f"Generated: {output_path}")
    return output_path


def main():
    parser = argparse.ArgumentParser(description="Piper TTS")
    parser.add_argument("text", help="Text to speak")
    parser.add_argument("-o", "--output", default="/tmp/piper_output.wav", help="Output wav path")
    args = parser.parse_args()
    speak(args.text, args.output)


if __name__ == "__main__":
    main()
