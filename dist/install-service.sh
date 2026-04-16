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

# Stop the service if it's running so we can replace the binary (avoids ETXTBSY)
WAS_ACTIVE=0
if systemctl --user is-active --quiet huion-keydial-mini.service; then
    WAS_ACTIVE=1
    echo "Stopping running huion-keydial-mini service..."
    systemctl --user stop huion-keydial-mini.service
fi

# Install binaries (install(1) unlinks the target first, sidestepping ETXTBSY
# if huion-gui is currently running)
mkdir -p "$BIN_DIR"
install -m 755 "$REPO_ROOT/target/release/huion-keydial-mini" "$BIN_DIR/huion-keydial-mini"
install -m 755 "$REPO_ROOT/target/release/huion-gui" "$BIN_DIR/huion-gui"
echo "Installed binaries to $BIN_DIR/"

# Install systemd service
mkdir -p "$SERVICE_DIR"
cp "$SCRIPT_DIR/huion-keydial-mini.service" "$SERVICE_DIR/"
echo "Installed service to $SERVICE_DIR/huion-keydial-mini.service"

# Reload and enable
systemctl --user daemon-reload
systemctl --user enable huion-keydial-mini.service

if [ "$WAS_ACTIVE" = "1" ]; then
    echo "Restarting huion-keydial-mini service..."
    systemctl --user start huion-keydial-mini.service
else
    echo "Service enabled. Start with: systemctl --user start huion-keydial-mini"
fi
