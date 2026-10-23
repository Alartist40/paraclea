# USB Sneakernet Sync Appliance Architecture Guide
**Zero-Cloud, Air-Gapped, Cryptographically Authenticated Off-Grid Synchronization**

---

## 1. Executive Summary & Design Goals

When deploying autonomous field computers (e.g. Orange Pi 5, Raspberry Pi 4/5, portable rugged laptops) in wilderness, humanitarian, or off-grid scenarios, conventional cloud synchronization (APIs, WebSockets, Git remotes) is either impossible or dangerous.

**Sneakernet Synchronization** uses physical removable media (USB flash drives, SD cards) as secure, delay-tolerant transport carriers. The appliance automatically:
1. Detects the insertion of trusted removable media without manual mount commands.
2. Exchanges differential updates (new study notes, cross-references, bookmarks, mesh packets).
3. Encrypts every payload using **AES-256-GCM** authenticated encryption with **PBKDF2** key derivation.
4. Flushes kernel writeback caches and safely prepares the media for physical extraction.

---

## 2. Removable Media Discovery Across Operating Systems

Rather than hardcoding Linux paths, the discovery engine inspects platform-specific mount points:

```rust
pub fn find_usb_vault(marker: &str) -> Option<PathBuf> {
    let mut search_roots = Vec::new();

    #[cfg(target_os = "linux")]
    {
        // Modern udisks2 mount paths
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
        // Drive letters D:\ through Z:\
        for letter in b'D'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let path = PathBuf::from(&drive);
            if path.exists() {
                search_roots.push(path);
            }
        }
    }

    for root in search_roots {
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let candidate = entry.path().join(marker);
                if candidate.exists() && candidate.is_dir() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}
```

---

## 3. Cryptographic Envelope Format

Every sync bundle is stored in the vault as an authenticated binary envelope:
`delta_<ISO8601_or_unix_ts>.enc`

### Binary Envelope Layout
```
+-------------------------------------------------------------+
| Salt (16 bytes)                                             |
+-------------------------------------------------------------+
| Nonce / IV (12 bytes)                                       |
+-------------------------------------------------------------+
| AES-256-GCM Ciphertext + Poly1305 Tag (16 bytes tag appended)|
+-------------------------------------------------------------+
```

1. **Key Derivation**:
   - `PBKDF2-HMAC-SHA256` with 100,000 iterations.
   - Salt: 16 bytes generated from cryptographically secure RNG (`OsRng`).
2. **Authenticated Encryption**:
   - `AES-256-GCM`: provides confidentiality AND integrity. Any physical tampering or bit corruption causes immediate authentication failure upon import.

---

## 4. Conflict Resolution & Delta Manifests

Each payload contains a structured `SyncManifest`:

```json
{
  "device_id": "paraclea-node-a7f4",
  "exported_at": "2026-10-23T14:30:00Z",
  "items": [
    {
      "id": "dendrite-note-981",
      "entity_type": "note",
      "timestamp": 1729693800,
      "sha256": "4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a",
      "content": "Study insights on Sanctuary typology..."
    }
  ]
}
```

### Resolution Rules:
- **Last-Write-Wins (LWW)**: Monotonically increasing timestamps resolve state when entities share identical primary keys.
- **Additive Graph Merge**: In knowledge graphs (like Dendrite v2), notes and cross-reference edges are set unions: new nodes and links are appended without destroying older notes.

---

## 5. Safe Automated Extraction (Preventing Data Loss)

Physical media removal while kernel dirty pages are in RAM will corrupt FAT32/exFAT/ext4 tables.

### Safe Unmount Sequence:
1. **Flush Kernel Buffers**:
   ```rust
   // Linux / macOS
   std::process::Command::new("sync").status().ok();
   ```
2. **Issue Device Eject / Unmount**:
   ```rust
   // Linux udisks2 (no root required)
   std::process::Command::new("udisksctl")
       .args(["unmount", "-b", &device_path])
       .status()
       .ok();
   ```
3. **Notify User**:
   Trigger a TUI Toast: `"[USB Vault Synced & Safe to Remove]"` in gold highlight.

---

## 6. How to Reuse in Your Other Rust Projects

1. Copy `sneakernet.rs` into your project's crate.
2. Call `SneakernetSync::find_usb_vault(".my_app_vault")`.
3. If detected, call `export_delta()` or `import_delta()` with the user's master passphrase.
