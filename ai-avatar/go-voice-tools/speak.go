package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// cmdSpeak runs TTS inference using the piper binary.
//
// Learning moment: cmd.Stdin = strings.NewReader(text) pipes Go strings
// directly into a subprocess — no temp files needed.
func cmdSpeak(args []string) {
	fs := flag.NewFlagSet("speak", flag.ExitOnError)
	text := fs.String("text", "Hello, I am Paraclea.", "Text to speak")
	outputPath := fs.String("output", "/tmp/piper_output.wav", "Output wav path")
	modelPath := fs.String("model", "../models/en_US-lessac-medium.onnx", "Piper ONNX model")
	configPath := fs.String("config", "../models/en_US-lessac-medium.onnx.json", "Piper JSON config")
	fs.Parse(args)

	// Find piper binary — check .venv first, then PATH
	piperBin := "piper"
	venvPiper := "../.venv/bin/piper"
	if _, err := os.Stat(venvPiper); err == nil {
		piperBin = venvPiper
	}

	cmd := exec.Command(piperBin,
		"--model", *modelPath,
		"--config", *configPath,
		"--output_file", *outputPath,
	)
	cmd.Stdin = strings.NewReader(*text)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "piper failed: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Generated: %s\n", *outputPath)
}

// Need strings import — but it's already imported in transcribe.go
// Go files in the same package share imports via the package declaration.
// Actually, we need to add "strings" to this file's imports since Go
// requires each file to import what it uses.
