#!/usr/bin/env bash
# Install the Huion KeyDial Mini driver as a systemd user service
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

SERVICE_DIR="${HOME}/.config/systemd/user"
BIN_DIR="${HOME}/.local/bin"

# Build release binaries
echo "Building release binaries..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" -p huion-keydial-mini -p huion-gui

# Install binaries
mkdir -p "$BIN_DIR"
cp "$REPO_ROOT/target/release/huion-keydial-mini" "$BIN_DIR/huion-keydial-mini"
cp "$REPO_ROOT/target/release/huion-gui" "$BIN_DIR/huion-gui"
echo "Installed binaries to $BIN_DIR/"

# Install systemd service
mkdir -p "$SERVICE_DIR"
cp "$SCRIPT_DIR/huion-keydial-mini.service" "$SERVICE_DIR/"
echo "Installed service to $SERVICE_DIR/huion-keydial-mini.service"

# Reload and enable
systemctl --user daemon-reload
systemctl --user enable huion-keydial-mini.service
echo "Service enabled. Start with: systemctl --user start huion-keydial-mini"
