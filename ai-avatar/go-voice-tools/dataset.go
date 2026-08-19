package main

import (
	"flag"
	"fmt"
	"os"
)

// cmdDataset runs segment + transcribe + metadata in one command.
//
// Learning moment: os.Args manipulation lets us reuse existing commands
// without duplicating code. We rebuild the argument slices and call
// the same functions the CLI would call.
func cmdDataset(args []string) {
	fs := flag.NewFlagSet("dataset", flag.ExitOnError)
	input := fs.String("input", "", "Input audiobook file")
	outputDir := fs.String("output", "dataset", "Output directory")
	segmentSec := fs.Int("seg", 8, "Seconds per clip")
	overlapSec := fs.Int("overlap", 1, "Seconds overlap")
	skipSec := fs.Int("skip", 180, "Seconds to skip")
	modelPath := fs.String("model", "../models/ggml-tiny.bin", "Whisper model path")
	whisperBin := fs.String("whisper", "/tmp/whisper.cpp/build/bin/whisper-cli", "Whisper binary")
	fs.Parse(args)

	if *input == "" {
		fmt.Fprintln(os.Stderr, "Error: -input is required")
		fs.Usage()
		os.Exit(1)
	}

	wavDir := *outputDir + "/wavs"
	metaPath := *outputDir + "/metadata.csv"

	// Step 1: Segment
	fmt.Println("\n=== Step 1: Segmenting ===")
	segmentArgs := []string{
		"-input", *input,
		"-output", wavDir,
		"-seg", fmt.Sprintf("%d", *segmentSec),
		"-overlap", fmt.Sprintf("%d", *overlapSec),
		"-skip", fmt.Sprintf("%d", *skipSec),
	}
	cmdSegment(segmentArgs)

	// Step 2: Transcribe
	fmt.Println("\n=== Step 2: Transcribing ===")
	transcribeArgs := []string{
		"-input", wavDir,
		"-model", *modelPath,
		"-whisper", *whisperBin,
	}
	cmdTranscribe(transcribeArgs)

	// Step 3: Metadata
	fmt.Println("\n=== Step 3: Building metadata ===")
	metadataArgs := []string{
		"-input", wavDir,
		"-output", metaPath,
	}
	cmdMetadata(metadataArgs)

	fmt.Println("\n=== Dataset pipeline complete! ===")
	fmt.Printf("Output: %s\n", *outputDir)
}
