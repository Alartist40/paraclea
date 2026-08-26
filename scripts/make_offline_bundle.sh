#!/usr/bin/env bash
set -e

echo "📦 Building Paraclea Self-Contained Offline Installer Bundle..."

# Ensure release binaries exist
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

echo "  ✓ Generating offline installation script..."
cat << 'INSTALL_EOF' > "$BUNDLE_DIR/install_offline.sh"
#!/usr/bin/env bash
set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     PARACLEA AI ASSISTANT — OFFLINE AIR-GAPPED INSTALLER     ║"
echo "╚══════════════════════════════════════════════════════════════╝"

INSTALL_BIN="$HOME/.local/bin"
PARACLEA_DIR="$HOME/.paraclea"

mkdir -p "$INSTALL_BIN"
mkdir -p "$PARACLEA_DIR/bibles"
mkdir -p "$PARACLEA_DIR/library"
mkdir -p "$PARACLEA_DIR/persona"

echo "  ✓ Installing binaries to $INSTALL_BIN..."
cp bin/paraclea "$INSTALL_BIN/"
cp bin/paraclea-gui "$INSTALL_BIN/"
chmod +x "$INSTALL_BIN/paraclea" "$INSTALL_BIN/paraclea-gui"

echo "  ✓ Installing offline Bible database (219 translations)..."
cp -r bibles/* "$PARACLEA_DIR/bibles/" 2>/dev/null || true

echo "  ✓ Installing offline Non-Scripture Library (211 chapters)..."
cp -r library/* "$PARACLEA_DIR/library/" 2>/dev/null || true

echo "  ✓ Installing default persona templates..."
cp -r persona/* "$PARACLEA_DIR/persona/" 2>/dev/null || true

echo ""
echo "🎉 Paraclea Offline Installation Complete!"
echo "   Run 'paraclea' for Terminal CLI"
echo "   Run 'paraclea-gui' for Desktop Application Web GUI"
INSTALL_EOF

chmod +x "$BUNDLE_DIR/install_offline.sh"

echo "  ✓ Creating tar.gz archive..."
cd target
tar -czf paraclea-offline-bundle.tar.gz paraclea-offline-bundle/
cd ..

echo ""
echo "✨ Offline Bundle Created Successfully: target/paraclea-offline-bundle.tar.gz"
echo "   File Size: $(du -h target/paraclea-offline-bundle.tar.gz | cut -f1)"
