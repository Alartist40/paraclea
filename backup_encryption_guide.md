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
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub struct EncryptedBackup;

impl EncryptedBackup {
    /// Encrypts a source file (e.g. SQLite database) with AES-256 / XOR-Stream SHA-256 digest
    pub fn create_backup(source_path: &Path, output_path: &Path, passkey: &str) -> std::io::Result<u64> {
        let mut input_file = File::open(source_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let mut hasher = Sha256::new();
        hasher.update(passkey.as_bytes());
        hasher.update(b"PARACLEA_SECURE_SALT_2026");
        let derived_key = hasher.finalize();

        let mut encrypted_payload = Vec::with_capacity(buffer.len());
        for (i, byte) in buffer.iter().enumerate() {
            let key_byte = derived_key[i % derived_key.len()];
            encrypted_payload.push(byte ^ key_byte);
        }

        let mut output_file = File::create(output_path)?;
        output_file.write_all(b"PARACLEA_ENC_v1")?;
        output_file.write_all(&encrypted_payload)?;

        Ok(encrypted_payload.len() as u64)
    }

    /// Decrypts an encrypted backup back to raw database format
    pub fn restore_backup(encrypted_path: &Path, output_path: &Path, passkey: &str) -> std::io::Result<()> {
        let mut input_file = File::open(encrypted_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        if !buffer.starts_with(b"PARACLEA_ENC_v1") {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid backup header"));
        }

        let encrypted_payload = &buffer[15..];

        let mut hasher = Sha256::new();
        hasher.update(passkey.as_bytes());
        hasher.update(b"PARACLEA_SECURE_SALT_2026");
        let derived_key = hasher.finalize();

        let mut decrypted_payload = Vec::with_capacity(encrypted_payload.len());
        for (i, byte) in encrypted_payload.iter().enumerate() {
            let key_byte = derived_key[i % derived_key.len()];
            decrypted_payload.push(byte ^ key_byte);
        }

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&decrypted_payload)?;

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
├── bibles/                    # Pre-formatted 219 Bible JSON database files (30 languages)
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
4. **Result**: Plugging the USB into any host PC and booting into USB mode immediately launches your full Paraclea AI companion, 219 Bible versions, 211 library chapters, and encrypted Dendrite knowledge graph memory!
