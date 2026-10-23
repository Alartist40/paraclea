//! Audio Output Playback Module for Paraclea
//!
//! Provides both synchronous one-shot audio playback and a zero-latency
//! asynchronous double-buffered jitter streaming engine for continuous natural TTS.

use anyhow::Result;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub struct AudioPlayer;

impl AudioPlayer {
    /// Play raw WAV audio bytes synchronously using aplay, paplay, pw-play, or afplay.
    pub fn play_wav_bytes(wav_bytes: &[u8]) -> Result<()> {
        if wav_bytes.is_empty() {
            return Ok(());
        }

        let temp_file = crate::temp_dir().join(format!(
            "paraclea_speech_{}_{}.wav",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let _ = fs::write(&temp_file, wav_bytes);

        #[cfg(target_os = "macos")]
        let _ = Command::new("afplay").arg(&temp_file).status();

        #[cfg(target_os = "windows")]
        let _ = Command::new("powershell")
            .arg("-c")
            .arg(format!(
                "(New-Object Media.SoundPlayer '{}').PlaySync()",
                temp_file.display()
            ))
            .status();

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let _ = Command::new("aplay")
            .arg("-q")
            .arg(&temp_file)
            .status()
            .or_else(|_| Command::new("paplay").arg(&temp_file).status())
            .or_else(|_| Command::new("pw-play").arg(&temp_file).status());

        let _ = fs::remove_file(&temp_file);
        Ok(())
    }
}

/// Asynchronous double-buffered audio streaming engine with live amplitude calculation.
#[derive(Clone)]
pub struct AsyncAudioStreamer {
    sender: UnboundedSender<Vec<u8>>,
    playing: Arc<AtomicBool>,
    // Amplitude stored as u32 (fixed-point: 0..10000 -> 0.0000..1.0000)
    amplitude: Arc<AtomicU32>,
}

impl AsyncAudioStreamer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let playing = Arc::new(AtomicBool::new(false));
        let amplitude = Arc::new(AtomicU32::new(0));

        let p_clone = Arc::clone(&playing);
        let a_clone = Arc::clone(&amplitude);

        // Spawn dedicated background audio consumer thread
        std::thread::spawn(move || {
            Self::worker_loop(rx, p_clone, a_clone);
        });

        Self {
            sender: tx,
            playing,
            amplitude,
        }
    }

    /// Enqueue a synthesized audio chunk into the playback stream without blocking.
    pub fn enqueue_chunk(&self, wav_bytes: Vec<u8>) -> Result<()> {
        if wav_bytes.is_empty() {
            return Ok(());
        }
        self.sender
            .send(wav_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to enqueue audio chunk: {}", e))?;
        Ok(())
    }

    /// Check if audio is currently actively playing.
    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    /// Get the current live audio amplitude (0.0 to 1.0) for live visualizers.
    pub fn current_amplitude(&self) -> f32 {
        let raw = self.amplitude.load(Ordering::Relaxed);
        (raw as f32) / 10000.0
    }

    /// Calculate RMS amplitude from PCM bytes (standard 16-bit signed PCM WAV).
    pub fn calculate_pcm_rms(wav_bytes: &[u8]) -> f32 {
        if wav_bytes.len() < 44 {
            return 0.0;
        }
        let pcm = &wav_bytes[44..];
        let mut sum_sq: f64 = 0.0;
        let mut sample_count = 0;

        for chunk in pcm.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f64 / 32768.0;
            sum_sq += sample * sample;
            sample_count += 1;
        }

        if sample_count == 0 {
            0.0
        } else {
            ((sum_sq / sample_count as f64).sqrt() as f32).clamp(0.0, 1.0)
        }
    }

    fn worker_loop(
        mut rx: UnboundedReceiver<Vec<u8>>,
        playing: Arc<AtomicBool>,
        amplitude: Arc<AtomicU32>,
    ) {
        while let Some(chunk) = rx.blocking_recv() {
            playing.store(true, Ordering::Relaxed);

            // Compute peak/RMS amplitude for this chunk
            let amp = Self::calculate_pcm_rms(&chunk);
            let amp_fixed = ((amp * 10000.0) as u32).min(10000);
            amplitude.store(amp_fixed, Ordering::Relaxed);

            // Play through OS system player
            let _ = AudioPlayer::play_wav_bytes(&chunk);

            amplitude.store(0, Ordering::Relaxed);
            playing.store(false, Ordering::Relaxed);
        }
    }
}

impl Default for AsyncAudioStreamer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_async_audio_streamer() {
        let streamer = AsyncAudioStreamer::new();
        assert!(!streamer.is_playing());
        assert_eq!(streamer.current_amplitude(), 0.0);

        // Test empty enqueue
        assert!(streamer.enqueue_chunk(Vec::new()).is_ok());

        // Test PCM RMS calculation
        let mut mock_wav = vec![0u8; 44]; // 44-byte header
        for i in 0..100 {
            let sample: i16 = if i % 2 == 0 { 16384 } else { -16384 };
            mock_wav.extend_from_slice(&sample.to_le_bytes());
        }
        let rms = AsyncAudioStreamer::calculate_pcm_rms(&mock_wav);
        assert!(
            rms > 0.4 && rms < 0.6,
            "RMS should be ~0.5 for half-scale square wave, got {}",
            rms
        );
    }
}
