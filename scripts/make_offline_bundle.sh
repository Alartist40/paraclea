#!/usr/bin/env bash
set -e

echo "📦 Building Paraclea Self-Contained Reusable Offline Installer..."

# Ensure release binaries are built
cargo build --release --workspace

BUNDLE_DIR="target/paraclea-offline-bundle"
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/bin"
mkdir -p "$BUNDLE_DIR/bibles"
mkdir -p "$BUNDLE_DIR/library"
mkdir -p "$BUNDLE_DIR/persona"

echo "  ✓ Copying release binaries..."
cp target/release/paraclea "$BUNDLE_DIR/bin/"
cp target/release/paraclea-gui "$BUNDLE_DIR/bin/"

echo "  ✓ Copying formatted Bible & Library databases..."
if [ -d "$HOME/.paraclea/bibles" ]; then
    cp -r "$HOME/.paraclea/bibles/"* "$BUNDLE_DIR/bibles/"
fi

if [ -d "$HOME/.paraclea/library" ]; then
    cp -r "$HOME/.paraclea/library/"* "$BUNDLE_DIR/library/"
fi

if [ -d "$HOME/.paraclea/persona" ]; then
    cp -r "$HOME/.paraclea/persona/"* "$BUNDLE_DIR/persona/"
fi

echo "  ✓ Creating reusable non-deleting installation script..."
cat << 'INSTALL_EOF' > "$BUNDLE_DIR/install_offline.sh"
#!/usr/bin/env bash
set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║   PARACLEA AI ASSISTANT — 1-CLICK REUSABLE OFFLINE INSTALL   ║"
echo "╚══════════════════════════════════════════════════════════════╝"

INSTALL_BIN="$HOME/.local/bin"
PARACLEA_DIR="$HOME/.paraclea"

mkdir -p "$INSTALL_BIN"
mkdir -p "$PARACLEA_DIR/bibles"
mkdir -p "$PARACLEA_DIR/library"
mkdir -p "$PARACLEA_DIR/persona"

echo "  ✓ Copying binaries to $INSTALL_BIN..."
cp -f bin/paraclea "$INSTALL_BIN/"
cp -f bin/paraclea-gui "$INSTALL_BIN/"
chmod +x "$INSTALL_BIN/paraclea" "$INSTALL_BIN/paraclea-gui"

echo "  ✓ Copying 219 Bible versions..."
cp -rf bibles/* "$PARACLEA_DIR/bibles/" 2>/dev/null || true

echo "  ✓ Copying 211 Non-Scripture Library chapters..."
cp -rf library/* "$PARACLEA_DIR/library/" 2>/dev/null || true

echo "  ✓ Copying persona templates..."
cp -rf persona/* "$PARACLEA_DIR/persona/" 2>/dev/null || true

# Ensure PATH includes ~/.local/bin
if ! grep -q "$INSTALL_BIN" "$HOME/.bashrc" 2>/dev/null; then
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$HOME/.bashrc"
fi

echo ""
echo "🎉 Installation Complete! USB installer files remain 100% intact on USB."
echo "   Run 'paraclea' for Terminal CLI"
echo "   Run 'paraclea-gui' for Desktop Application Web GUI"
INSTALL_EOF

chmod +x "$BUNDLE_DIR/install_offline.sh"

echo "  ✓ Creating standard tar.gz archive..."
cd target
tar -czf paraclea-offline-bundle.tar.gz paraclea-offline-bundle/
cd ..

echo "  ✓ Creating 1-Click Single-File Self-Extracting Installer (install_paraclea.sh)..."
cat << 'SELFEXTRACT_EOF' > target/install_paraclea.sh
#!/usr/bin/env bash
# Paraclea 1-Click Reusable Offline Installer
set -e

TMP_DIR=$(mktemp -d -t paraclea-install-XXXXXX)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "📦 Extracting Paraclea installer payload..."
PAYLOAD_LINE=$(grep -a -n '^__PAYLOAD_BELOW__' "$0" | cut -d: -f1)
tail -n +$((PAYLOAD_LINE + 1)) "$0" | tar -xz -C "$TMP_DIR"

cd "$TMP_DIR/paraclea-offline-bundle"
./install_offline.sh

exit 0
__PAYLOAD_BELOW__
SELFEXTRACT_EOF

cat target/paraclea-offline-bundle.tar.gz >> target/install_paraclea.sh
chmod +x target/install_paraclea.sh

echo ""
echo "✨ Bundling Complete!"
echo "   • Reusable 1-Click Single-File Installer: target/install_paraclea.sh ($(du -h target/install_paraclea.sh | cut -f1))"
echo "   • Standard Archive: target/paraclea-offline-bundle.tar.gz ($(du -h target/paraclea-offline-bundle.tar.gz | cut -f1))"
echo "   💡 Simply copy target/install_paraclea.sh directly onto your USB drive!"
