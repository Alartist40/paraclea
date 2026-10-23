//! USB Sneakernet Synchronization Appliance
//!
//! Provides zero-cloud, cryptographically authenticated off-grid synchronization
//! for air-gapped field nodes using removable USB/SD media with AES-256-GCM.

use crate::backup::EncryptedBackup;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{bail, Context, Result};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const SNEAKERNET_VAULT_DIR: &str = ".paraclea_vault";
pub const SNEAKERNET_MAGIC: &[u8] = b"SNEAKERNET_v1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

/// A discrete synchronized payload item (e.g. Dendrite note, cross-reference edge, bookmark).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncItem {
    pub id: String,
    pub entity_type: String,
    pub timestamp: i64,
    pub content: String,
}

/// An export manifest describing all bundled synchronization items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManifest {
    pub node_id: String,
    pub exported_at: i64,
    pub items: Vec<SyncItem>,
}

pub struct SneakernetEngine;

impl SneakernetEngine {
    /// Detect an active USB vault folder on removable media.
    pub fn find_vault(custom_mount: Option<&Path>) -> Option<PathBuf> {
        if let Some(m) = custom_mount {
            let candidate = m.join(SNEAKERNET_VAULT_DIR);
            if candidate.exists() && candidate.is_dir() {
                return Some(candidate);
            }
        }

        let mut search_roots = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(user) = std::env::var("USER") {
                search_roots.push(PathBuf::from(format!("/media/{}", user)));
                search_roots.push(PathBuf::from(format!("/run/media/{}", user)));
            }
            search_roots.push(PathBuf::from("/mnt"));
            search_roots.push(PathBuf::from("/media"));
        }

        #[cfg(target_os = "macos")]
        {
            search_roots.push(PathBuf::from("/Volumes"));
        }

        #[cfg(target_os = "windows")]
        {
            for letter in b'D'..=b'Z' {
                let drive = format!("{}:\\", letter as char);
                let path = PathBuf::from(&drive);
                if path.exists() {
                    search_roots.push(path);
                }
            }
        }

        for root in search_roots {
            if let Ok(entries) = fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let candidate = entry.path().join(SNEAKERNET_VAULT_DIR);
                    if candidate.exists() && candidate.is_dir() {
                        return Some(candidate);
                    }
                }
            }
        }

        None
    }

    /// Export items into an encrypted delta envelope inside the USB vault.
    pub fn export_delta(
        vault_dir: &Path,
        node_id: &str,
        items: Vec<SyncItem>,
        passkey: &str,
    ) -> Result<PathBuf> {
        if passkey.trim().is_empty() {
            bail!("Passphrase cannot be empty for sneakernet sync");
        }

        fs::create_dir_all(vault_dir)
            .with_context(|| format!("Failed to create vault directory {:?}", vault_dir))?;

        let timestamp = chrono::Utc::now().timestamp();
        let manifest = SyncManifest {
            node_id: node_id.to_string(),
            exported_at: timestamp,
            items,
        };

        let plaintext = serde_json::to_vec(&manifest)?;

        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce_bytes);

        let key = EncryptedBackup::derive_key(passkey.trim(), &salt);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("AES-256 initialization error: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let delta_filename = format!("delta_{}_{}.enc", node_id, timestamp);
        let output_path = vault_dir.join(delta_filename);

        let mut file = File::create(&output_path)?;
        file.write_all(SNEAKERNET_MAGIC)?;
        file.write_all(&salt)?;
        file.write_all(&nonce_bytes)?;
        file.write_all(&ciphertext)?;
        file.flush()?;

        Ok(output_path)
    }

    /// Import and decrypt all delta envelopes found in the USB vault.
    pub fn import_delta(vault_dir: &Path, passkey: &str) -> Result<Vec<SyncItem>> {
        if !vault_dir.exists() {
            bail!("Vault directory {:?} does not exist", vault_dir);
        }

        let mut all_items = Vec::new();
        let entries = fs::read_dir(vault_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("enc") {
                let mut file = File::open(&path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                let header_len = SNEAKERNET_MAGIC.len() + SALT_LEN + NONCE_LEN;
                if buffer.len() < header_len || !buffer.starts_with(SNEAKERNET_MAGIC) {
                    continue; // Skip unrecognized or corrupted files
                }

                let salt = &buffer[SNEAKERNET_MAGIC.len()..SNEAKERNET_MAGIC.len() + SALT_LEN];
                let nonce_bytes = &buffer[SNEAKERNET_MAGIC.len() + SALT_LEN..header_len];
                let ciphertext = &buffer[header_len..];

                let key = EncryptedBackup::derive_key(passkey.trim(), salt);
                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| anyhow::anyhow!("AES-256 init error: {}", e))?;
                let nonce = Nonce::from_slice(nonce_bytes);

                let plaintext = match cipher.decrypt(nonce, ciphertext) {
                    Ok(pt) => pt,
                    Err(_) => bail!("Decryption failed for {:?}: invalid passphrase or tampered payload", path),
                };

                let manifest: SyncManifest = serde_json::from_slice(&plaintext)?;
                all_items.extend(manifest.items);
            }
        }

        // Deduplicate items by ID, keeping the newest timestamp
        all_items.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        all_items.dedup_by(|a, b| a.id == b.id);

        Ok(all_items)
    }

    /// Safely flush buffers to avoid filesystem corruption before extraction.
    pub fn safe_unmount(_mount_point: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            let _ = std::process::Command::new("sync").status();
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    pub fn test_sneakernet_sync_roundtrip() {
        let dir = tempdir().unwrap();
        let vault_dir = dir.path().join(SNEAKERNET_VAULT_DIR);
        let passkey = "SecretWildernessVault2026!";

        let item1 = SyncItem {
            id: "note-1".to_string(),
            entity_type: "study_note".to_string(),
            timestamp: 100,
            content: "Dan 8:14 sanctuary cleansing reflection".to_string(),
        };
        let item2 = SyncItem {
            id: "note-2".to_string(),
            entity_type: "study_note".to_string(),
            timestamp: 105,
            content: "Rev 14:12 patience of the saints".to_string(),
        };

        // Export delta
        let delta_path = SneakernetEngine::export_delta(
            &vault_dir,
            "node_alpha",
            vec![item1.clone(), item2.clone()],
            passkey,
        )
        .expect("Export delta should succeed");

        assert!(delta_path.exists());

        // Import delta
        let imported = SneakernetEngine::import_delta(&vault_dir, passkey)
            .expect("Import delta should succeed");

        assert_eq!(imported.len(), 2);
        assert!(imported.contains(&item1));
        assert!(imported.contains(&item2));

        // Wrong passkey test
        let fail_result = SneakernetEngine::import_delta(&vault_dir, "WrongPassword!");
        assert!(fail_result.is_err());
    }
}
