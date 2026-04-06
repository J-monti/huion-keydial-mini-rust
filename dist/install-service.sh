#!/usr/bin/env bash
# Install the Huion KeyDial Mini driver as a systemd user service
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

SERVICE_DIR="${HOME}/.config/systemd/user"
BIN_DIR="${HOME}/.local/bin"

BINARY="${REPO_ROOT}/target/release/huion-keydial-mini"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run 'cargo build --release -p huion-keydial-mini' first."
    exit 1
fi

# Install binary
mkdir -p "$BIN_DIR"
cp "$BINARY" "$BIN_DIR/huion-keydial-mini"
echo "Installed binary to $BIN_DIR/huion-keydial-mini"

# Install systemd service
mkdir -p "$SERVICE_DIR"
cp "$SCRIPT_DIR/huion-keydial-mini.service" "$SERVICE_DIR/"
echo "Installed service to $SERVICE_DIR/huion-keydial-mini.service"

# Reload and enable
systemctl --user daemon-reload
systemctl --user enable huion-keydial-mini.service
echo "Service enabled. Start with: systemctl --user start huion-keydial-mini"
