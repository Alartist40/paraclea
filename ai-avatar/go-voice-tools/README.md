# voice — Go Voice Toolchain

A single static binary for the Paraclea voice pipeline. Replaces all Python scripts.

## Build

```bash
cd go-voice-tools
go build -o voice .
```

## Commands

```bash
./voice segment    -input file.mp3 -output dir/          # Chop audiobook into clips
./voice transcribe -input dir/ -model ggml-tiny.bin      # Run Whisper on clips
./voice metadata   -input dir/ -output metadata.csv      # Build LJSpeech CSV
./voice speak      -text "hello" -output out.wav         # TTS inference
./voice dataset    -input file.mp3 -output dir/          # Run all three at once
./voice help                                              # Show usage
```

## Makefile

```bash
make build     # Compile
make segment   # Segment Alice audiobook
make speak     # Generate test WAV
make dataset   # Run full pipeline
make install   # Copy to /usr/local/bin
```

## What stays in Python?

Only **Piper neural training** (Step 8) — it requires PyTorch/GPU. Run that on Google Colab using `Paraclea_Voice_Training.ipynb` in the project root. The notebook has detailed explanations in every markdown cell.

## Code notes for learning

- `segment.go` — `os/exec.Command` with explicit `[]string` args (no shell injection)
- `transcribe.go` — `filepath.Glob` + skip-if-exists pattern for resumable batch jobs
- `speak.go` — `strings.NewReader` piped into subprocess stdin
- `dataset.go` — reusing command functions by rebuilding their `[]string` args
