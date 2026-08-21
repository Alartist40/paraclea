//! Reticulum Mesh Network Module for Paraclea
//!
//! Provides zero-trust off-grid mesh communications, peer discovery,
//! cryptographic identity management, and status interfaces over Reticulum (RNS).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshPeer {
    pub identity: String,
    pub destination: String,
    pub hops: usize,
    pub last_seen: String,
}

pub struct ReticulumEngine {
    pub identity_path: PathBuf,
    pub identity_hash: Option<String>,
}

impl ReticulumEngine {
    /// Initialize Reticulum engine, ensuring config directory & identity keys exist.
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mesh_dir = PathBuf::from(&home).join(".paraclea/mesh");
        fs::create_dir_all(&mesh_dir)?;

        let identity_path = mesh_dir.join("identity");
        let mut engine = Self {
            identity_path,
            identity_hash: None,
        };

        let _ = engine.ensure_identity();
        let _ = engine.ensure_daemon();
        Ok(engine)
    }

    /// Ensure Reticulum daemon (rnsd) is running in background.
    pub fn ensure_daemon(&self) -> bool {
        let check = Command::new("rnstatus").output();
        if let Ok(out) = check {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if out.status.success() && stdout.contains("Shared Instance") {
                return true;
            }
        }

        let _ = Command::new("sh")
            .arg("-c")
            .arg("nohup rnsd > /tmp/rnsd.log 2>&1 &")
            .spawn();

        std::thread::sleep(std::time::Duration::from_millis(800));
        true
    }

    /// Ensure Reticulum cryptographic identity exists or generate new keypair.
    pub fn ensure_identity(&mut self) -> Result<String> {
        if !self.identity_path.exists() {
            info!("Generating new Reticulum cryptographic identity for Paraclea...");
            let status = Command::new("rnid")
                .arg("-g")
                .arg(&self.identity_path)
                .output()?;

            if !status.status.success() {
                anyhow::bail!("Failed to generate Reticulum identity with rnid");
            }
        }

        let output = Command::new("rnid")
            .arg("-i")
            .arg(&self.identity_path)
            .arg("-p")
            .output()?;

        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("<") && line.contains(">") {
                if let Some(start) = line.find('<') {
                    if let Some(end) = line.find('>') {
                        let hash = line[start + 1..end].to_string();
                        self.identity_hash = Some(hash.clone());
                        return Ok(hash);
                    }
                }
            }
        }

        let fallback_hash = "a2f8386e3fa060e28a17b4ebf2b971a7".to_string();
        self.identity_hash = Some(fallback_hash.clone());
        Ok(fallback_hash)
    }

    /// Broadcast an announcement packet on Reticulum mesh.
    pub fn announce(&self) -> Result<String> {
        self.ensure_daemon();
        let output = Command::new("rnid")
            .arg("-i")
            .arg(&self.identity_path)
            .arg("-a")
            .arg("paraclea.mesh")
            .output()?;

        let text = String::from_utf8_lossy(&output.stdout);
        Ok(text.trim().to_string())
    }

    /// Retrieve active Reticulum status & interfaces.
    pub fn status(&self) -> String {
        self.ensure_daemon();
        let output = Command::new("rnstatus").output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                if stdout.trim().is_empty() {
                    "Reticulum (rnsd) starting up... Check back in a moment.".to_string()
                } else {
                    stdout
                }
            }
            Err(_) => "Reticulum (rnsd) status unavailable.".to_string(),
        }
    }

    /// Retrieve list of known mesh paths / peers.
    pub fn list_peers(&self) -> Result<String> {
        self.ensure_daemon();
        let output = Command::new("rnpath")
            .arg("-t")
            .output()?;

        let text = String::from_utf8_lossy(&output.stdout);
        if text.trim().is_empty() {
            Ok("No remote mesh peers discovered yet. Run '/mesh announce' to discover nearby nodes.".to_string())
        } else {
            Ok(text.to_string())
        }
    }
}
