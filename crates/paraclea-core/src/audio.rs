//! Audio Output Playback Module for Paraclea
//!
//! Plays synthesized speech WAV audio through default system speakers using `rodio`.

use anyhow::Result;
use std::fs;
use std::process::Command;

pub struct AudioPlayer;

impl AudioPlayer {
    /// Play raw WAV audio bytes using aplay, paplay, or pw-play.
    pub fn play_wav_bytes(wav_bytes: &[u8]) -> Result<()> {
        if wav_bytes.is_empty() {
            return Ok(());
        }

        let temp_file = "/tmp/paraclea_speech.wav";
        let _ = fs::write(temp_file, wav_bytes);

        let _ = Command::new("aplay")
            .arg("-q")
            .arg(temp_file)
            .status()
            .or_else(|_| Command::new("paplay").arg(temp_file).status())
            .or_else(|_| Command::new("pw-play").arg(temp_file).status());

        Ok(())
    }
}
