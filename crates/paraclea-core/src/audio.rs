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

        let temp_file = crate::temp_dir().join(format!("paraclea_speech_{}_{}.wav", std::process::id(), uuid::Uuid::new_v4()));
        let _ = fs::write(&temp_file, wav_bytes);

        #[cfg(target_os = "macos")]
        let _ = Command::new("afplay").arg(&temp_file).status();

        #[cfg(target_os = "windows")]
        let _ = Command::new("powershell")
            .arg("-c")
            .arg(format!("(New-Object Media.SoundPlayer '{}').PlaySync()", temp_file.display()))
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
