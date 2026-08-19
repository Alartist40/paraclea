#!/bin/bash
# Paraclea AI Assistant One-Line Installer Script (Pure Rust Engine)
# Usage on any machine:
#   curl -fsSL https://raw.githubusercontent.com/Alartist40/paraclea/main/install.sh | bash

set -e

BOLD="\033[1m"
GOLD="\033[38;2;255;215;0m"
PURPLE="\033[38;2;177;74;237m"
GREEN="\033[0;32m"
RED="\033[0;31m"
RESET="\033[0m"

echo -e "${PURPLE}${BOLD}=================================================="${RESET}
echo -e "${GOLD}${BOLD}   Paraclea AI Companion Installer (v0.1.0 Rust)   "${RESET}
echo -e "${PURPLE}${BOLD}=================================================="${RESET}

# Detect operating system
OS="$(uname -s)"
if [ "$OS" != "Linux" ] && [ "$OS" != "Darwin" ]; then
    echo -e "${RED}ERROR: Paraclea installer currently supports Linux and macOS.${RESET}"
    exit 1
fi

# If not in source repository with Cargo.toml, clone to ~/.paraclea-src
if [ ! -f "Cargo.toml" ]; then
    INSTALL_DIR="$HOME/.paraclea-src"
    echo -e "${PURPLE}[0/3] Fetching Paraclea repository into $INSTALL_DIR...${RESET}"
    if [ -d "$INSTALL_DIR" ]; then
        cd "$INSTALL_DIR"
        git pull --quiet || true
    else
        git clone --quiet https://github.com/Alartist40/paraclea.git "$INSTALL_DIR"
        cd "$INSTALL_DIR"
    fi
fi

# Check Cargo / Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: Rust / Cargo toolchain not found.${RESET}"
    echo "Install Rust via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo -e "${PURPLE}[1/3] Initializing configuration and persona templates...${RESET}"
if [ ! -f "config.yaml" ]; then
    cat > config.yaml << 'CFGEOF'
system:
  name: "Paraclea"
  version: "0.1.0"

model:
  backend: "ollama"
  ollama:
    url: "http://localhost:11434"
    model: "llama3.2"
  local:
    path: "models"

voice:
  pocket_tts_url: "http://localhost:8000"
  pocket_tts_voice: "alba"
  pocket_tts_cli: "/home/xander/Documents/reference/pocket-tts/.venv/bin/pocket-tts"

persona:
  dir: "persona"
  heartbeat_interval: 15
CFGEOF
fi

mkdir -p persona/logs/daily

if [ ! -f "persona/IDENTITY.md" ]; then
    cat > persona/IDENTITY.md << 'IDEOF'
# IDENTITY

- **Name:** Paraclea
- **Nature:** AI Companion & Self-Developing Assistant Engine
- **Vibe:** Smart, intelligent, warm, attentive, expressive, witty, and loyal assistant.
- **Role:** Personal AI companion, pair programmer, assistant, and self-improving agent.
IDEOF
fi

echo -e "${PURPLE}[2/3] Compiling Paraclea Rust release binary...${RESET}"
cargo build --release

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

echo -e "${PURPLE}[3/3] Installing executable binary to $BIN_DIR/paraclea...${RESET}"
cp target/release/paraclea "$BIN_DIR/paraclea"
chmod +x "$BIN_DIR/paraclea"

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo -e "${GOLD}Adding $BIN_DIR to PATH in ~/.bashrc...${RESET}"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    if [ -f "$HOME/.zshrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
    fi
    export PATH="$HOME/.local/bin:$PATH"
fi

echo ""
echo -e "${GOLD}${BOLD}✓ Paraclea installation complete!${RESET}"
echo -e "${PURPLE}Binary installed to:${RESET} $BIN_DIR/paraclea"
echo ""
echo -e "${BOLD}Try running:${RESET}"
echo -e "  ${GOLD}paraclea list${RESET}         # List available Ollama & local models"
echo -e "  ${GOLD}paraclea run 1${RESET}        # Run model #1 from list"
echo -e "  ${GOLD}paraclea${RESET}              # Run interactive companion shell"
echo ""
