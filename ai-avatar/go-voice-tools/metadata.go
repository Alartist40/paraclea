package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// cmdMetadata builds LJSpeech-format metadata.csv from transcribed clips.
//
// Learning moment: bufio.Scanner is overkill here. For small files,
// os.ReadFile + strings.TrimSpace is simpler and just as fast.
func cmdMetadata(args []string) {
	fs := flag.NewFlagSet("metadata", flag.ExitOnError)
	inputDir := fs.String("input", "dataset/wavs", "Directory with wavs + txts")
	outputPath := fs.String("output", "dataset/metadata.csv", "Output CSV path")
	fs.Parse(args)

	pattern := filepath.Join(*inputDir, "*.wav")
	files, err := filepath.Glob(pattern)
	if err != nil {
		fmt.Fprintf(os.Stderr, "glob: %v\n", err)
		os.Exit(1)
	}

	var entries []string
	for _, wavPath := range files {
		txtPath := strings.TrimSuffix(wavPath, ".wav") + ".txt"
		data, err := os.ReadFile(txtPath)
		if err != nil {
			continue // No transcript — skip
		}
		transcript := strings.TrimSpace(string(data))
		if transcript == "" {
			continue
		}
		basename := filepath.Base(wavPath)
		// LJSpeech: filename|transcript|normalized_transcript
		entries = append(entries, fmt.Sprintf("%s|%s|%s", basename, transcript, transcript))
	}

	if err := os.MkdirAll(filepath.Dir(*outputPath), 0755); err != nil {
		fmt.Fprintf(os.Stderr, "mkdir: %v\n", err)
		os.Exit(1)
	}

	content := strings.Join(entries, "\n")
	if err := os.WriteFile(*outputPath, []byte(content), 0644); err != nil {
		fmt.Fprintf(os.Stderr, "write: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Wrote %d entries to %s\n", len(entries), *outputPath)
	fmt.Println("Dataset ready for Piper training!")
}
