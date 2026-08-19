//! Audio Output Playback Module for Paraclea
//!
//! Plays synthesized speech WAV audio through default system speakers using `rodio`.

use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

pub struct AudioPlayer;

impl AudioPlayer {
    /// Play raw WAV audio bytes synchronously on background audio thread.
    pub fn play_wav_bytes(wav_bytes: &[u8]) -> Result<()> {
        if wav_bytes.is_empty() {
            return Ok(());
        }

        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(res) => res,
            Err(_) => return Ok(()),
        };

        let sink = match Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        let cursor = Cursor::new(wav_bytes.to_vec());
        let source = match Decoder::new(cursor) {
            Ok(src) => src,
            Err(_) => return Ok(()),
        };

        sink.append(source);
        sink.sleep_until_end();
        Ok(())
    }
}
