#!/bin/bash
# Paraclea AI Companion Assistant & Self-Developing RAG Engine One-Line Installer Script
# Usage on any machine:
#   curl -fsSL https://raw.githubusercontent.com/Alartist40/paraclea/main/install.sh | bash

set -e

BOLD="\033[1m"
GOLD="\033[38;2;255;215;0m"
PURPLE="\033[38;2;177;74;237m"
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
RESET="\033[0m"

echo -e "${PURPLE}${BOLD}╔══════════════════════════════════════════╗"${RESET}
echo -e "${GOLD}${BOLD}║     Paraclea — The Helper Installer      ║"${RESET}
echo -e "${PURPLE}${BOLD}╚══════════════════════════════════════════╝"${RESET}

# Detect OS & Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case $ARCH in
    x86_64)  ARCH_TAG="x86_64-unknown-linux-gnu" ;;
    aarch64|arm64) ARCH_TAG="aarch64-unknown-linux-musl" ;;
    *) echo -e "${RED}Unsupported architecture: $ARCH${RESET}"; exit 1 ;;
esac

case $OS in
    linux) ;;
    darwin) ARCH_TAG="${ARCH_TAG/-unknown-linux-gnu/-apple-darwin}" ;;
    *) echo -e "${RED}Unsupported OS: $OS${RESET}"; exit 1 ;;
esac

echo -e "${PURPLE}📦 Detected Platform: $OS / $ARCH ($ARCH_TAG)${RESET}"

INSTALL_DIR="$HOME/.paraclea"
BIN_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR/bin" "$INSTALL_DIR/qdrant" "$INSTALL_DIR/data" "$INSTALL_DIR/bibles" "$INSTALL_DIR/library" "$INSTALL_DIR/persona/logs/daily" "$BIN_DIR"

# Check system dependencies
check_cmd() { command -v "$1" >/dev/null 2>&1; }

MISSING=""
if ! check_cmd git; then MISSING="$MISSING git"; fi
if ! check_cmd curl; then MISSING="$MISSING curl"; fi

if [ -n "$MISSING" ]; then
    echo -e "${YELLOW}⚠️ Missing system packages:$MISSING${RESET}"
    echo "Please install git & curl first."
    exit 1
fi

# Install Rust toolchain if missing
if ! check_cmd cargo; then
    echo -e "${GOLD}🦀 Rust toolchain not found. Installing Rust...${RESET}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Install Ollama if missing
if ! check_cmd ollama; then
    echo -e "${GOLD}🤖 Ollama not found. Installing Ollama...${RESET}"
    curl -fsSL https://ollama.com/install.sh | sh || true
fi

# Download Qdrant single binary executable
if [ ! -f "$INSTALL_DIR/bin/qdrant" ]; then
    echo -e "${PURPLE}🔍 Downloading Qdrant Vector Database ($ARCH_TAG)...${RESET}"
    QDRANT_URL="https://github.com/qdrant/qdrant/releases/latest/download/qdrant-${ARCH_TAG}.tar.gz"
    if curl -fsSL "$QDRANT_URL" -o "$INSTALL_DIR/bin/qdrant.tar.gz"; then
        tar -xzf "$INSTALL_DIR/bin/qdrant.tar.gz" -C "$INSTALL_DIR/bin"
        rm -f "$INSTALL_DIR/bin/qdrant.tar.gz"
        chmod +x "$INSTALL_DIR/bin/qdrant"
        echo -e "${GREEN}✅ Qdrant binary ready in $INSTALL_DIR/bin/qdrant${RESET}"
    else
        echo -e "${YELLOW}⚠️ Downloaded Qdrant binary skipped. Ensure Qdrant is available on system.${RESET}"
    fi
fi

# Start Qdrant in background inside ~/.paraclea/qdrant
if ! curl -s http://localhost:6333/collections >/dev/null 2>&1; then
    if [ -f "$INSTALL_DIR/bin/qdrant" ]; then
        echo -e "${PURPLE}🚀 Starting Qdrant vector database background process...${RESET}"
        (cd "$INSTALL_DIR/qdrant" && nohup "$INSTALL_DIR/bin/qdrant" > "$INSTALL_DIR/qdrant.log" 2>&1 &)
        sleep 2
    fi
fi

# Clone or update Paraclea source repository
if [ ! -f "Cargo.toml" ]; then
    SRC_DIR="$INSTALL_DIR/paraclea"
    echo -e "${PURPLE}📥 Fetching Paraclea codebase into $SRC_DIR...${RESET}"
    if [ -d "$SRC_DIR" ]; then
        cd "$SRC_DIR"
        git pull --quiet || true
    else
        git clone --quiet https://github.com/Alartist40/paraclea.git "$SRC_DIR"
        cd "$SRC_DIR"
    fi
fi

# Create default configuration file in ~/.paraclea/config.yaml
if [ ! -f "$INSTALL_DIR/config.yaml" ]; then
    cat > "$INSTALL_DIR/config.yaml" << CFGEOF
system:
  name: "Paraclea"
  version: "0.1.0"

model:
  backend: "ollama"
  ollama:
    url: "http://localhost:11434"
    model: "ministral-3:3b"
    heavy_model: "qwen3.5:9b"
    embed_model: "nomic-embed-text"
    ocr_model: "frob/unlimited-ocr:q8_0"
  local:
    path: "models"

vector_db:
  qdrant_url: "http://localhost:6333"
  collection_bible: "bible"
  collection_books: "books"
  collection_survival: "survival"

voice:
  pocket_tts_url: "http://localhost:8000"
  pocket_tts_voice: "alba"
  pocket_tts_cli: "$HOME/.paraclea/bin/pocket-tts"

persona:
  dir: "$INSTALL_DIR/persona"
  heartbeat_interval: 15
CFGEOF
    echo -e "${GREEN}✅ Default config.yaml created at $INSTALL_DIR/config.yaml.${RESET}"
fi

# Copy persona templates if persona folder exists in source repo
if [ -d "persona" ]; then
    cp -rn persona/* "$INSTALL_DIR/persona/" 2>/dev/null || true
fi

# Prompt user for installation preference (CLI vs CLI + Desktop GUI)
echo ""
echo -e "${GOLD}${BOLD}Choose your installation preference:${RESET}"
echo -e "  ${PURPLE}[1]${RESET} CLI Only (Pure Rust Terminal Assistant - Fast & Minimal) [Default]"
echo -e "  ${PURPLE}[2]${RESET} CLI + Desktop Application GUI"
echo ""
if [ -t 0 ]; then
    read -p "Select option [1-2] (Default: 1): " INSTALL_CHOICE
else
    INSTALL_CHOICE="1"
fi
INSTALL_CHOICE="${INSTALL_CHOICE:-1}"

# Build & install workspace binaries
echo -e "${PURPLE}🔨 Building Paraclea release binaries (Rust opt-level 3 workspace)...${RESET}"
cargo build --release --workspace

if [ -f "data/kjv.json" ]; then
    cp "data/kjv.json" "$INSTALL_DIR/data/kjv.json"
fi

install -m 755 target/release/paraclea "$BIN_DIR/paraclea"

if [ "$INSTALL_CHOICE" == "2" ] && [ -f "target/release/paraclea-gui" ]; then
    echo -e "${PURPLE}🖥 Installing Paraclea Desktop GUI Application...${RESET}"
    install -m 755 target/release/paraclea-gui "$BIN_DIR/paraclea-gui"
    
    DESKTOP_DIR="$HOME/.local/share/applications"
    mkdir -p "$DESKTOP_DIR"
    cat > "$DESKTOP_DIR/paraclea.desktop" << DESKEOF
[Desktop Entry]
Name=Paraclea AI Companion
Comment=Pure Rust AI Companion Assistant & Multi-Category Library
Exec=$BIN_DIR/paraclea-gui
Icon=utilities-terminal
Terminal=false
Type=Application
Categories=Utility;Education;
DESKEOF
    echo -e "${GREEN}✅ Desktop menu shortcut created at $DESKTOP_DIR/paraclea.desktop${RESET}"
fi

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo -e "${GOLD}Adding $BIN_DIR to PATH in ~/.bashrc...${RESET}"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    if [ -f "$HOME/.zshrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
    fi
    export PATH="$HOME/.local/bin:$PATH"
fi

echo ""
echo -e "${GOLD}${BOLD}╔══════════════════════════════════════════╗"${RESET}
echo -e "${GOLD}${BOLD}║     ✅ Paraclea Installed Successfully!  ║"${RESET}
echo -e "${GOLD}${BOLD}╚══════════════════════════════════════════╝"${RESET}
echo ""
echo -e "${PURPLE}📍 Binary location:${RESET} $BIN_DIR/paraclea"
if [ "$INSTALL_CHOICE" == "2" ]; then
    echo -e "${PURPLE}📍 Desktop GUI location:${RESET} $BIN_DIR/paraclea-gui"
fi
echo -e "${PURPLE}📍 Data & Config:${RESET} $INSTALL_DIR/"
echo -e "${PURPLE}📍 Qdrant location:${RESET} $INSTALL_DIR/bin/qdrant"
echo ""
echo -e "${BOLD}Quick Commands:${RESET}"
echo -e "  ${GOLD}paraclea doctor${RESET}                           # Run system diagnostics"
echo -e "  ${GOLD}paraclea${RESET}                                  # Start Paraclea REPL shell"
if [ "$INSTALL_CHOICE" == "2" ]; then
    echo -e "  ${GOLD}paraclea-gui${RESET}                              # Launch Paraclea Desktop Application"
fi
echo ""
