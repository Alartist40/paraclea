package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// cmdTranscribe runs whisper.cpp on every .wav file in a directory.
//
// Learning moment: filepath.Glob + sorted iteration is the Go idiom
// for batch file processing. No external glob libraries needed.
func cmdTranscribe(args []string) {
	fs := flag.NewFlagSet("transcribe", flag.ExitOnError)
	inputDir := fs.String("input", "dataset/wavs", "Directory containing wav clips")
	modelPath := fs.String("model", "../models/ggml-tiny.bin", "Path to whisper model")
	whisperBin := fs.String("whisper", "/tmp/whisper.cpp/build/bin/whisper-cli", "whisper.cpp binary")
	fs.Parse(args)

	// Verify whisper binary exists
	if _, err := os.Stat(*whisperBin); err != nil {
		fmt.Fprintf(os.Stderr, "whisper.cpp not found at %s\n", *whisperBin)
		fmt.Fprintln(os.Stderr, "Build it: cd /tmp && git clone https://github.com/ggerganov/whisper.cpp && cd whisper.cpp && cmake -B build && cmake --build build")
		os.Exit(1)
	}

	// Find all wav files
	pattern := filepath.Join(*inputDir, "*.wav")
	files, err := filepath.Glob(pattern)
	if err != nil {
		fmt.Fprintf(os.Stderr, "glob: %v\n", err)
		os.Exit(1)
	}
	if len(files) == 0 {
		fmt.Fprintf(os.Stderr, "No wav files found in %s\n", *inputDir)
		os.Exit(1)
	}

	fmt.Printf("Transcribing %d clips...\n\n", len(files))

	for _, wavPath := range files {
		txtPath := strings.TrimSuffix(wavPath, ".wav") + ".txt"
		if info, err := os.Stat(txtPath); err == nil && info.Size() > 0 {
			continue // Already transcribed — skip
		}

		text, err := runWhisper(*whisperBin, *modelPath, wavPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "  %s: %v\n", filepath.Base(wavPath), err)
			continue
		}

		if err := os.WriteFile(txtPath, []byte(text), 0644); err != nil {
			fmt.Fprintf(os.Stderr, "  write %s: %v\n", txtPath, err)
			continue
		}

		display := text
		if len(display) > 60 {
			display = display[:60] + "..."
		}
		fmt.Printf("  %s -> \"%s\"\n", filepath.Base(wavPath), display)
	}

	fmt.Println("\nTranscription complete!")
}

// runWhisper executes whisper-cli and returns the clean transcript.
// Learning moment: CombinedOutput captures both stdout and stderr.
// We filter whisper's internal log lines starting with "whisper_".
func runWhisper(bin, model, wav string) (string, error) {
	cmd := exec.Command(bin,
		"-m", model,
		"-f", wav,
		"-l", "en",
		"--no-timestamps",
		"-nt",
		"-np", // no prints (progress bar)
	)
	out, err := cmd.CombinedOutput()
	if err != nil {
		// whisper may write errors to stderr but still produce output
		// We try to extract text from whatever we got
	}

	lines := strings.Split(string(out), "\n")
	var result []string
	for _, line := range lines {
		line = strings.TrimSpace(line)
		// Skip whisper's internal log lines and blank lines
		if line == "" || strings.HasPrefix(line, "whisper_") || strings.HasPrefix(line, "[") {
			continue
		}
		result = append(result, line)
	}
	return strings.Join(result, " "), nil
}
