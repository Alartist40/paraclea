# Paraclea Portable AppImage Distribution & Delta Auto-Update Guide

This guide explains how Paraclea can be packaged as a single standalone Linux executable (`Paraclea.AppImage`) and how end-users receive lightweight delta updates without redownloading the entire bundle.

---

## 1. What is an AppImage?

An **AppImage** is a self-mounting ISO/SquashFS filesystem containing the compiled binary, its shared libraries, default assets, and an execution wrapper.
- **Portability**: Runs on Ubuntu, Debian, Fedora, Arch, Alpine, Raspberry Pi OS, etc.
- **Zero Installation**: Mark executable (`chmod +x Paraclea-x86_64.AppImage`) and double click to run.
- **Sovereign Isolation**: Does not require root permissions (`sudo`) or package managers (`apt`, `pacman`).

---

## 2. How Do Users Update an AppImage?

The primary misconception is that updating an AppImage requires downloading a fresh 200 MB file every time. In production, AppImages use **Zsync Binary Delta Updates**.

### The Zsync Architecture

```
Developer Workspace                    GitHub Release Server                    User Device
───────────────────                    ─────────────────────                    ───────────
1. Compile Paraclea
2. appimagetool embeds
   Update Information
   into ELF header.
3. Generate:
   - Paraclea.AppImage (200 MB)  ───>  Uploads both files  ───> User runs: `paraclea --update`
   - Paraclea.AppImage.zsync (5 KB)                                     │
                                                                         v
                                                                 Downloads .zsync file (5 KB)
                                                                 Calculates block checksums
                                                                 against local AppImage
                                                                         │
                                                                         v
                                                                 Downloads ONLY modified bytes
                                                                 (e.g., 5 MB diff instead of 200 MB)
                                                                         │
                                                                         v
                                                                 Atomically replaces AppImage
```

---

## 3. Embedding Update Information During Build

When building with `appimagetool`, pass the `-u` update flag pointing to your GitHub repository:

```bash
appimagetool -u "zsync|https://github.com/Alartist40/paraclea/releases/latest/download/Paraclea-x86_64.AppImage.zsync" \
  AppDir Paraclea-x86_64.AppImage
```

This embeds the update metadata into an ELF header segment inside the binary.

---

## 4. How the User Updates

### Method A: Native Terminal Command (`paraclea --update`)
Paraclea can invoke the embedded `appimageupdate` utility or `zsync2`:

```bash
paraclea --update
```

Output:
```
Checking for updates at https://github.com/Alartist40/paraclea/releases...
New version found: v0.2.0 (Local: v0.1.0)
Downloading delta blocks: [====================] 100% (4.8 MB downloaded)
Verification successful. Paraclea has been updated to v0.2.0!
```

### Method B: Standalone AppImageUpdate GUI
Users who prefer desktop UIs can install `appimageupdatetool` or AppImageLauncher. Right-clicking the Paraclea icon in their Linux desktop environment offers **"Update"**, which checks GitHub and patches the file automatically.

---

## 5. User Data Preservation

AppImages run in read-only mounted filesystems. All user settings, database writes, and voice models are stored strictly in standard home directory paths:
- `~/.paraclea/dendrite.db` (Graph memory)
- `~/.paraclea/bibles/` (Downloaded and cached translations)
- `~/.paraclea/mesh/` (Reticulum cryptographic identity)

Updating or replacing the `Paraclea.AppImage` executable **never wipes or touches user data**.
