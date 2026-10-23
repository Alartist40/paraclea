#!/usr/bin/env bash
# Paraclea Portable AppImage Packaging Script
set -e

echo "=== Building Paraclea Portable AppImage ==="

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPDIR="${ROOT_DIR}/target/AppDir"
OUTPUT_DIR="${ROOT_DIR}/target/dist"

# 1. Compile release binaries
echo "Compiling optimized release binaries..."
cargo build --release -p paraclea-cli -p paraclea-gui

# 2. Prepare AppDir structure
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${OUTPUT_DIR}"

cp "${ROOT_DIR}/target/release/paraclea" "${APPDIR}/usr/bin/paraclea"
cp "${ROOT_DIR}/target/release/paraclea-gui" "${APPDIR}/usr/bin/paraclea-gui"

# 3. Create Desktop Entry
cat << 'EOF' > "${APPDIR}/paraclea.desktop"
[Desktop Entry]
Name=Paraclea
Comment=Pure Rust Sovereign AI Companion & 160-Version Bible Engine
Exec=paraclea
Icon=paraclea
Terminal=true
Type=Application
Categories=Education;Science;ConsoleOnly;
EOF

# 4. Create AppRun Launcher
cat << 'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/paraclea" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

# 5. Fetch appimagetool if not present
if ! command -v appimagetool >/dev/null 2>&1; then
    echo "Downloading standalone appimagetool..."
    curl -sL "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage" -o /tmp/appimagetool
    chmod +x /tmp/appimagetool
    TOOL="/tmp/appimagetool"
else
    TOOL="appimagetool"
fi

# 6. Generate AppImage with embedded update information
echo "Generating final AppImage bundle..."
ARCH=x86_64 "${TOOL}" \
    -u "zsync|https://github.com/Alartist40/paraclea/releases/latest/download/Paraclea-x86_64.AppImage.zsync" \
    "${APPDIR}" "${OUTPUT_DIR}/Paraclea-x86_64.AppImage"

echo "=== Success! Paraclea AppImage created at ${OUTPUT_DIR}/Paraclea-x86_64.AppImage ==="
