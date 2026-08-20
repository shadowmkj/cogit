#!/bin/sh
# ==============================================================================
# Cogit Installer Script for macOS and Linux
# ==============================================================================
# Usage:
#   curl -proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/shadowmkj/cogit/main/scripts/install.sh | sh
# ==============================================================================

set -eu

REPO="shadowmkj/cogit"
BIN_NAME="cogit"

echo "🦀 Installing Cogit..."

# 1. Detect operating system
OS="$(uname -s)"
case "$OS" in
  Linux*)  PLATFORM="unknown-linux-gnu" ;;
  Darwin*) PLATFORM="apple-darwin" ;;
  *)
    echo "Error: Unsupported operating system '$OS'." >&2
    exit 1
    ;;
esac

# 2. Detect CPU architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)   ARCH_TARGET="x86_64" ;;
  aarch64|arm64)  ARCH_TARGET="aarch64" ;;
  *)
    echo "Error: Unsupported architecture '$ARCH'." >&2
    exit 1
    ;;
esac

TARGET="${ARCH_TARGET}-${PLATFORM}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-${TARGET}.tar.gz"

# 3. Download and extract binary
TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

echo "📦 Downloading $BIN_NAME for $TARGET..."
if command -v curl >/dev/null 2>&1; then
  curl -sSL "$DOWNLOAD_URL" -o "$TMP_DIR/archive.tar.gz"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP_DIR/archive.tar.gz" "$DOWNLOAD_URL"
else
  echo "Error: curl or wget is required to download Cogit." >&2
  exit 1
fi

tar -xzf "$TMP_DIR/archive.tar.gz" -C "$TMP_DIR"

# 4. Determine install directory
INSTALL_DIR="/usr/local/bin"
USE_SUDO=0

if [ ! -w "$INSTALL_DIR" ]; then
  if [ "$(id -u)" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
  elif command -v sudo >/dev/null 2>&1 && [ -t 0 ]; then
    USE_SUDO=1
  else
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "$INSTALL_DIR"
  fi
fi

echo "🚀 Installing binary to $INSTALL_DIR/$BIN_NAME..."
if [ "$USE_SUDO" -eq 1 ]; then
  sudo install -m 755 "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
  mkdir -p "$INSTALL_DIR"
  install -m 755 "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
fi

echo "✅ Cogit successfully installed!"

# 5. Check if install directory is on PATH
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo "⚠️  Note: '$INSTALL_DIR' is not in your current PATH."
    echo "   Add it by adding this line to your shell configuration (.bashrc / .zshrc):"
    echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
    ;;
esac
