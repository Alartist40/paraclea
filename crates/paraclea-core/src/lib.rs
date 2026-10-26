pub mod aligner;
pub mod audio;
pub mod backup;
pub mod bible;
pub mod bulletin;
pub mod candle_stt;
pub mod compressor;
pub mod config;
pub mod crossref;
pub mod dendrite;
pub mod detect;
pub mod heatmap;
pub mod heartbeat;
pub mod ingest;
pub mod library;
pub mod loopguard;
pub mod matrix;
pub mod mesh;
pub mod mmap_bible;
pub mod needle;
pub mod ollama;
pub mod persona;
pub mod pocket_tts;
pub mod qdrant;
pub mod rag;
pub mod security;
pub mod sneakernet;
pub mod strongs;
pub mod timeline;
pub mod tools;
pub mod vector;

use std::path::PathBuf;

/// Returns the user's home directory across Linux, macOS, and Windows.
pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Returns the platform-appropriate temporary directory.
pub fn temp_dir() -> PathBuf {
    std::env::temp_dir()
}

/// Returns a Command configured with the platform's default shell (cmd on Windows, sh on Unix).
pub fn shell_command() -> std::process::Command {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
    } else {
        std::process::Command::new("sh")
    }
}

/// Returns the shell argument flag ("/C" on Windows, "-c" on Unix).
pub fn shell_arg() -> &'static str {
    if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_platform_helpers() {
        assert!(!home_dir().as_os_str().is_empty());
        assert!(!temp_dir().as_os_str().is_empty());
        assert!(!shell_arg().is_empty());
    }
}
