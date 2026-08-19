// voice — Go-based voice training toolchain for Paraclea.
//
// Subcommands:
//   segment    Chop an audiobook into clips
//   transcribe Run Whisper on all clips
//   metadata   Build LJSpeech metadata.csv
//   speak      TTS inference with Piper
//   dataset    Run segment + transcribe + metadata in one shot
//
// Usage:
//   go build -o voice
//   ./voice segment -input ../../alice.mp3 -out dataset/wavs
//   ./voice transcribe -in dataset/wavs
//   ./voice metadata -in dataset/wavs -out dataset/metadata.csv
//   ./voice speak -text "Hello world" -out hello.wav
//   ./voice dataset -input ../../alice.mp3 -output dataset

package main

import (
	"fmt"
	"os"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	cmd := os.Args[1]
	switch cmd {
	case "segment":
		cmdSegment(os.Args[2:])
	case "transcribe":
		cmdTranscribe(os.Args[2:])
	case "metadata":
		cmdMetadata(os.Args[2:])
	case "speak":
		cmdSpeak(os.Args[2:])
	case "dataset":
		cmdDataset(os.Args[2:])
	case "help", "-h", "--help":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n\n", cmd)
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println(`voice — Paraclea Voice Toolchain (Go)

Subcommands:
  segment     Chop audiobook into clips
  transcribe  Run whisper.cpp on all clips
  metadata    Build LJSpeech metadata.csv
  speak       TTS inference with Piper
  dataset     Run segment + transcribe + metadata together

Examples:
  voice segment -input alice.mp3 -output dataset/wavs
  voice transcribe -input dataset/wavs
  voice metadata -input dataset/wavs -output dataset/metadata.csv
  voice speak -text "Hello world" -output hello.wav
  voice dataset -input alice.mp3 -output dataset`)
}
