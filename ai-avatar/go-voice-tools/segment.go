package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
)

// cmdSegment chops an audiobook into fixed-length clips using ffmpeg.
//
// Learning moment: Go's os/exec is clean and explicit. No hidden shell
// interpolation — you pass a []string of args exactly as they should be.
func cmdSegment(args []string) {
	fs := flag.NewFlagSet("segment", flag.ExitOnError)
	input := fs.String("input", "", "Input audio file (mp3, wav, etc.)")
	outputDir := fs.String("output", "dataset/wavs", "Output directory for clips")
	segmentSec := fs.Int("seg", 8, "Seconds per clip")
	overlapSec := fs.Int("overlap", 1, "Seconds of overlap between clips")
	skipSec := fs.Int("skip", 180, "Seconds to skip at start (intro/music)")
	fs.Parse(args)

	if *input == "" {
		fmt.Fprintln(os.Stderr, "Error: -input is required")
		fs.Usage()
		os.Exit(1)
	}

	if err := os.MkdirAll(*outputDir, 0755); err != nil {
		fmt.Fprintf(os.Stderr, "mkdir: %v\n", err)
		os.Exit(1)
	}

	duration, err := probeDuration(*input)
	if err != nil {
		fmt.Fprintf(os.Stderr, "ffprobe failed: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("Audio duration: %.1fs (%.1f min)\n", duration, duration/60)

	step := *segmentSec - *overlapSec
	idx := 0
	t := *skipSec

	for t+*segmentSec <= int(duration) {
		outName := fmt.Sprintf("clip_%04d.wav", idx)
		outPath := filepath.Join(*outputDir, outName)

		// ffmpeg command as explicit args — no shell quoting issues
		cmd := exec.Command("ffmpeg",
			"-y",               // overwrite output
			"-i", *input,       // input file
			"-ss", strconv.Itoa(t), // start time
			"-t", strconv.Itoa(*segmentSec), // duration
			"-ar", "22050",     // sample rate
			"-ac", "1",         // mono
			"-acodec", "pcm_s16le", // 16-bit PCM
			outPath,
		)
		cmd.Stdout = nil
		cmd.Stderr = nil
		if err := cmd.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "ffmpeg failed for %s: %v\n", outName, err)
			// Continue — one bad segment shouldn't kill the whole batch
		}

		idx++
		t += step
	}

	fmt.Printf("Extracted %d segments (%ds each, %ds step) from %ds to %.0fs\n",
		idx, *segmentSec, step, *skipSec, duration)
	fmt.Printf("Output: %s\n", *outputDir)

	// Write manifest
	manifestPath := filepath.Join(filepath.Dir(*outputDir), "manifest.txt")
	f, err := os.Create(manifestPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "manifest: %v\n", err)
		return
	}
	defer f.Close()
	for i := 0; i < idx; i++ {
		start := *skipSec + i*step
		end := start + *segmentSec
		fmt.Fprintf(f, "clip_%04d.wav\t%d\t%d\n", i, start, end)
	}
	fmt.Printf("Manifest: %s\n", manifestPath)
}

// probeDuration runs ffprobe to get audio duration in seconds.
// Learning moment: strings.TrimSpace + strconv.ParseFloat is Go's
// standard pattern for parsing command output.
func probeDuration(path string) (float64, error) {
	cmd := exec.Command("ffprobe",
		"-v", "error",
		"-show_entries", "format=duration",
		"-of", "default=noprint_wrappers=1:nokey=1",
		path,
	)
	out, err := cmd.Output()
	if err != nil {
		return 0, err
	}
	durationStr := strings.TrimSpace(string(out))
	return strconv.ParseFloat(durationStr, 64)
}
