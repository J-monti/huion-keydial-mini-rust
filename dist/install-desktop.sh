#!/usr/bin/env bash
# Install .desktop file and optionally enable autostart for Huion KeyDial Mini GUI
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

APP_DIR="${HOME}/.local/share/applications"
AUTOSTART_DIR="${HOME}/.config/autostart"
ICON_DIR="${HOME}/.local/share/icons/hicolor/128x128/apps"
BIN_DIR="${HOME}/.local/bin"

BINARY="${REPO_ROOT}/target/release/huion-keydial-mini-gui"
ICON_SRC="${REPO_ROOT}/crates/huion-gui/icons/icon.png"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run 'cargo build --release -p huion-keydial-mini-gui' first."
    exit 1
fi

# Install binary
mkdir -p "$BIN_DIR"
cp "$BINARY" "$BIN_DIR/huion-keydial-mini-gui"
echo "Installed binary to $BIN_DIR/huion-keydial-mini-gui"

# Install icon
mkdir -p "$ICON_DIR"
cp "$ICON_SRC" "$ICON_DIR/huion-keydial-mini.png"
echo "Installed icon"

# Install .desktop file
mkdir -p "$APP_DIR"
cp "$SCRIPT_DIR/huion-keydial-mini-gui.desktop" "$APP_DIR/"
echo "Installed .desktop file to $APP_DIR"

# Install autostart entry
if [ "${1:-}" = "--no-autostart" ]; then
    echo "Skipping autostart (--no-autostart)"
else
    mkdir -p "$AUTOSTART_DIR"
    cp "$SCRIPT_DIR/huion-keydial-mini-gui-autostart.desktop" "$AUTOSTART_DIR/huion-keydial-mini-gui.desktop"
    echo "Installed autostart entry to $AUTOSTART_DIR"
fi

# Update desktop database
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

echo "Done. Make sure $BIN_DIR is in your PATH."
