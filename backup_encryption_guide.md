# 🔒 Reusable Developer Guide: Encrypted Backup & Self-Contained Offline USB Systems

This guide explains how to implement a secure, **1-Click Encrypted USB Backup System** using **AES-256 / SHA-256 key derivation**, as well as how to build **Self-Contained Offline Installer Bundles** and **Bootable USB Portable Environments** that can turn any computer into your personal working environment without internet access.

---

## 🎯 High-Level Architecture

```
┌─────────────────┐       ┌────────────────────────┐       ┌────────────────────────┐
│  Raw Database   │ ───>  │  AES-256-GCM / PBKDF2  │ ───>  │ Encrypted Archive File │
│(dendrite.db/JSON│       │ Encryption Engine      │       │(paraclea_backup_...enc)│
└─────────────────┘       └────────────────────────┘       └────────────────────────┘
                                                                       │
                                                                       ▼
                                                           ┌────────────────────────┐
                                                           │ Saved to USB Drive     │
                                                           │ (/media/$USER/USB/...) │
                                                           └────────────────────────┘
```

### Why AES-256 + PBKDF2?
1. **Confidentiality**: Even if the USB flash drive is lost or stolen, your personal knowledge graph (`dendrite.db`), custom AI persona files, and study notes cannot be decrypted without your master passphrase.
2. **Integrity Verification**: Magic headers (`PARACLEA_ENC_v1`) ensure backup archives cannot be tampered with or corrupted.
3. **Cross-Platform Compatibility**: Standardized binary stream format readable across Linux, macOS, and Windows.

---

## 💻 Rust Backup Implementation

Below is the production-ready Rust module used in Paraclea:

```rust
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{bail, Context, Result};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub const MAGIC_V2: &[u8] = b"PARACLEA_ENC_v2";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const PBKDF2_ROUNDS: u32 = 100_000;

pub struct EncryptedBackup;

impl EncryptedBackup {
    /// Derive 32-byte key from passkey and random salt via PBKDF2-HMAC-SHA256
    pub fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, PBKDF2_ROUNDS, &mut key);
        key
    }

    /// Encrypts a source file (e.g. SQLite database) with AES-256-GCM AEAD
    pub fn create_backup(source_path: &Path, output_path: &Path, passkey: &str) -> Result<u64> {
        let mut input_file = File::open(source_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce_bytes);

        let key = Self::derive_key(passkey.trim(), &salt);
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let mut output_file = File::create(output_path)?;
        output_file.write_all(MAGIC_V2)?;
        output_file.write_all(&salt)?;
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;

        Ok((MAGIC_V2.len() + SALT_LEN + NONCE_LEN + ciphertext.len()) as u64)
    }

    /// Decrypts an authenticated AES-256-GCM backup back to raw database format
    pub fn restore_backup(encrypted_path: &Path, output_path: &Path, passkey: &str) -> Result<()> {
        let mut input_file = File::open(encrypted_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        if !buffer.starts_with(MAGIC_V2) {
            bail!("Invalid backup header");
        }

        let header_len = MAGIC_V2.len();
        let salt = &buffer[header_len..header_len + SALT_LEN];
        let nonce_bytes = &buffer[header_len + SALT_LEN..header_len + SALT_LEN + NONCE_LEN];
        let ciphertext = &buffer[header_len + SALT_LEN + NONCE_LEN..];

        let key = Self::derive_key(passkey.trim(), salt);
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Decryption failed: invalid passphrase or corrupted file"))?;

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        Ok(())
    }
}
```

---

## 🚀 Building a Self-Contained Offline Installer Bundle (`.tar.gz`)

When working in off-grid environments without internet access or Rust compilation toolchains, a **Self-Contained Offline Installer Bundle** packages pre-compiled binaries, formatted databases, and model weights into a single portable archive (`paraclea-offline-bundle.tar.gz`).

### 📦 Bundle Contents & Directory Structure
```
paraclea-offline-bundle/
├── install_offline.sh        # Zero-dependency Bash installation script
├── bin/
│   ├── paraclea               # Pre-compiled CLI binary (aarch64 / x86_64)
│   └── paraclea-gui           # Pre-compiled Desktop GUI binary
├── bibles/                    # Pre-formatted 160 Bible JSON database files (30 languages)
├── library/                   # Pre-formatted 7 non-scripture books (211 chapters)
└── persona/                   # System persona & SOUL template markdown files
```

### 🔨 How to Generate the Offline Bundle (`make_offline_bundle.sh`)
Run the bundling script on a build machine:
```bash
./scripts/make_offline_bundle.sh
```

This creates `paraclea-offline-bundle.tar.gz`. You can save this archive onto a USB drive.

### 💾 Installing on an Air-Gapped / Off-Grid Target Machine
Plug in your USB drive on any new machine without internet, unpack, and run:
```bash
tar -xzf paraclea-offline-bundle.tar.gz
cd paraclea-offline-bundle
./install_offline.sh
```
This instantly installs `paraclea` and `paraclea-gui` into `~/.local/bin/` and sets up `$HOME/.paraclea/` without compiling code or downloading any data over the internet!

---

## 🛸 Scaling to Bootable USB Operating Systems

To scale this architecture into a **Bootable USB Portable Working Environment**:
1. **Live Linux ISO Base**: Use a lightweight distro like Alpine, Debian Live, or Archiso.
2. **Persistence Partition**: Create a secondary encrypted LUKS partition on the USB drive mapped to `/home/$USER/`.
3. **Auto-Start Hook**: Include `~/.local/bin/paraclea-gui` in systemd or XDG autostart (`~/.config/autostart/paraclea.desktop`).
4. **Result**: Plugging the USB into any host PC and booting into USB mode immediately launches your full Paraclea AI companion, 160 Bible versions, 211 library chapters, and encrypted Dendrite knowledge graph memory!
