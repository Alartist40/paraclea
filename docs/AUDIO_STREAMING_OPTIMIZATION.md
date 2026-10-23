# Paraclea Audio Streaming & Double-Buffered Jitter Optimization Guide

This document records the exact technical architecture, formulas, and best practices used in Paraclea to achieve instant, zero-latency speech playback and eliminate choppy sentence-by-sentence pauses.

---

## 1. The Latency Problem in LLM-to-TTS Pipelines

When integrating an LLM with Text-to-Speech (TTS), naive implementations suffer from one of two severe bottlenecks:

| Approach | Architecture | User Experience |
|---|---|---|
| **Full Buffering** | Wait for entire LLM generation $\rightarrow$ send full paragraph to TTS $\rightarrow$ play audio. | Massive delay (3–8 seconds) before the first word is heard. Unacceptable for interactive companions. |
| **Naive Sentence Splitting** | LLM streams $\rightarrow$ detects punctuation (`.!?`) $\rightarrow$ blocks calling thread while TTS synthesizes $\rightarrow$ blocks while `aplay` plays. | Choppy speech with 1–2 second awkward silences between every single sentence. |

---

## 2. Paraclea's Solution: The Asynchronous Double-Buffered Jitter Ring

Paraclea decouples **Text Generation**, **Audio Synthesis**, and **Sound Playback** into three concurrent pipelined stages operating across independent asynchronous task queues:

```
[LLM Token Stream] ──(Tokens)──> [Sentence Boundary Detector]
                                          │
                                   (Sentence Chunks)
                                          │
                                          v
                                [Async TTS Worker Pool]
                               (Synthesizes Sentence N+1)
                                          │
                                     (PCM Bytes)
                                          │
                                          v
                              [Double-Buffered Jitter Queue]
                              (Unbounded Tokio MPSC Channel)
                                          │
                                     (PCM Bytes)
                                          │
                                          v
                            [Dedicated Audio Output Worker]
                             (Plays Sentence N on OS Driver)
```

### Key Architectural Invariants

1. **Zero-Blocking Producer**: When the LLM outputs a clause or sentence boundary, the synthesized audio bytes are enqueued via `AsyncAudioStreamer::enqueue_chunk(&self, chunk)`. The calling UI thread never waits on audio hardware.
2. **Double-Buffering**: While **Sentence 1** is actively vibrating the speakers, **Sentence 2** is already being synthesized by the model in the background and placed in the playback FIFO queue.
3. **Continuous Playback**: Because synthesis of Sentence $N+1$ finishes *before* playback of Sentence $N$ completes, the playback worker immediately pops the next packet without any gap or audio buffer underrun.
4. **Time-to-First-Sound (TTFS)**: The user hears Diana speaking within **< 350 ms** of sending a message, even on low-power CPU hardware.

---

## 3. Real-Time Amplitude Calculation for UI Waveforms

To render the live Braille / ASCII audio spectrum in the terminal (`⡀⣀⣄⣤⣦⣶⣿`) without audio driver lag, Paraclea computes the **Root Mean Square (RMS)** of the playing 16-bit signed PCM audio directly in the worker loop:

$$\text{RMS} = \sqrt{\frac{1}{N} \sum_{i=1}^{N} \left(\frac{x_i}{32768}\right)^2}$$

- Extracted from raw 16-bit PCM samples following the standard 44-byte WAV header.
- Stored into an atomic fixed-point scalar (`AtomicU32`) mapped from $0.0000$ to $1.0000$.
- Ratatui / TUI can poll `AsyncAudioStreamer::current_amplitude()` at 60 FPS with zero thread contention or lock overhead.

---

## 4. How to Apply This Optimization to Future Projects

Whenever building local voice or streaming AI systems:
1. **Never play audio on the main/UI thread**. Spawn a long-lived OS thread or Tokio background task.
2. **Buffer audio chunks, not sentences**. Chunk sizes between 1.5 seconds and 3.5 seconds of speech represent the optimal sweet spot between TTFS and prosody quality.
3. **Use atomic states for audio telemetry**. Flags like `is_playing` and values like `amplitude` should be `AtomicBool` and `AtomicU32` so rendering loops never take mutex locks on the audio driver.
