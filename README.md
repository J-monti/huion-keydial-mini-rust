# Huion KeyDial Mini - Linux Driver & GUI

A userspace driver and configuration GUI for the **Huion KeyDial Mini** Bluetooth macro keypad on Linux. All 15 programmable buttons and the rotary dial are fully remappable, with per-application profiles that switch automatically based on the focused window.

## Features

- **Full button remapping** — Assign any key or key chord (e.g. Ctrl+Shift+Z) to all 15 programmable buttons
- **Dial support** — Map the rotary dial's clockwise, counter-clockwise, and click actions to keys or key chords
- **Per-app profiles** — Automatically switch mappings based on the active window (e.g. different layouts for Krita vs Firefox)
- **Config hot-reload** — Edit `config.yaml` and changes take effect immediately, no restart needed
- **Desktop notifications** — Brief notification when the active profile changes
- **Tauri GUI** — Visual configuration tool with device layout, drag-and-drop key assignment, and profile management
- **Wayland & X11** — KWin scripting for Wayland window detection, xprop polling fallback for X11
- **Systemd integration** — Runs as a user service with automatic restart and boot-time retry

## Button Layout

```
 ┌──────┬──────┬──────┬──────┐
 │  1   │  2   │  3   │  4   │  Row 0
 ├──────┼──────┼──────┼──────┤
 │  5   │  6   │  7   │  8   │  Row 1
 ├──────┼──────┼──────┼──────┤
 │  9   │  10  │  11  │  12  │  Row 2
 ├──────┼──────┼──────┼──────┤
 │ Ctrl │ Alt  │Shift │      │
 ├──────┴──────┼──────┤  13  │  Row 3-4
 │     14      │  15  │      │
 └─────────────┴──────┴──────┘
         ╭───────╮
         │ Dial  │  CW / CCW / Click
         ╰───────╯
```

Buttons 1-12, 13-15 are remappable. Ctrl, Alt, and Shift are hardware modifiers and cannot be remapped.

## Prerequisites

- **Linux** with BlueZ (Bluetooth stack)
- **Rust** toolchain (1.70+)
- User must be in the **`input` group** (for `/dev/uinput` access)
- Bluetooth adapter paired with the Huion KeyDial Mini

### System packages

#### Arch Linux / Manjaro

```bash
# Driver only
sudo pacman -S bluez bluez-utils dbus

# GUI (additional)
sudo pacman -S webkit2gtk-4.1 gtk3 libayatana-appindicator
```

#### Ubuntu / Debian

```bash
# Driver only
sudo apt install bluez dbus

# GUI (additional)
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
```

#### Fedora

```bash
# Driver only
sudo dnf install bluez dbus

# GUI (additional)
sudo dnf install webkit2gtk4.1-devel gtk3-devel libayatana-appindicator-gtk3-devel
```

### User permissions

```bash
# Add yourself to the input group (required for /dev/uinput)
sudo usermod -aG input $USER

# Verify /dev/uinput permissions (should be group=input, mode=0660)
ls -la /dev/uinput
# crw-rw---- 1 root input 10, 223 ... /dev/uinput

# Log out and back in for group changes to take effect
```

## Building

```bash
git clone https://github.com/J-monti/huion-keydial-rust.git
cd huion-keydial-rust

# Build everything (driver + GUI)
cargo build --release

# Or build individually
cargo build --release -p huion-keydial-mini    # driver only
cargo build --release -p huion-gui             # GUI only
```

Binaries are placed in `target/release/`:
- `huion-keydial-mini` — the driver daemon
- `huion-keydial-mini-gui` — the configuration GUI

## Installation

### Driver (systemd service)

```bash
./dist/install-service.sh
```

This installs the driver binary to `~/.local/bin/` and enables a systemd user service that starts on login. Manage it with:

```bash
systemctl --user start huion-keydial-mini     # start now
systemctl --user stop huion-keydial-mini      # stop
systemctl --user restart huion-keydial-mini   # restart
systemctl --user status huion-keydial-mini    # check status
journalctl --user -u huion-keydial-mini -f    # follow logs
```

### GUI (desktop app)

```bash
./dist/install-desktop.sh
```

This installs the GUI binary, icon, `.desktop` file, and an autostart entry. The app will appear in your application launcher as "Huion KeyDial Mini" and start automatically on login.

To skip autostart:

```bash
./dist/install-desktop.sh --no-autostart
```

> **Note (KDE Wayland):** The `.desktop` file includes `WEBKIT_DISABLE_DMABUF_RENDERER=1` in the Exec line, which is needed for WebKit rendering on KDE Wayland.

### PATH

Ensure `~/.local/bin` is in your PATH. Add to your shell rc file if needed:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Pairing the Device

1. Put the KeyDial Mini in pairing mode (hold the Bluetooth button until the LED flashes)
2. Pair via your desktop's Bluetooth settings, or:

```bash
bluetoothctl
> scan on
> pair <MAC_ADDRESS>
> trust <MAC_ADDRESS>
> connect <MAC_ADDRESS>
```

The driver auto-detects any connected device with "huion" or "keydial" in its name. To target a specific device by MAC address, set `device_address` in the config or use `--device`:

```bash
huion-keydial-mini --device AA:BB:CC:DD:EE:FF
```

## Configuration

Config file location: `~/.config/huion-keydial-mini/config.yaml`

A default config is created automatically on first run. Edit it directly (changes are hot-reloaded) or use the GUI.

### Example config

```yaml
device_address: null    # auto-detect, or set a MAC like "20:25:03:CC:78:88"
debug_mode: false

default:
  button_mappings:
    '14':               # Button 1 — Ctrl+Shift+C
      - KEY_LEFTCTRL
      - KEY_LEFTSHIFT
      - KEY_C
    '15':               # Button 3 — Ctrl+C (Copy)
      - KEY_LEFTCTRL
      - KEY_C
    '76':               # Button 4 — Ctrl+V (Paste)
      - KEY_LEFTCTRL
      - KEY_V
    '12':               # Button 5 — Ctrl+Z (Undo)
      - KEY_LEFTCTRL
      - KEY_Z
    '29':               # Button 10 — Ctrl+Y (Redo)
      - KEY_LEFTCTRL
      - KEY_Y
    '22':               # Button 9 — Ctrl+S (Save)
      - KEY_LEFTCTRL
      - KEY_S
  dial:
    cw: KEY_VOLUMEUP
    ccw: KEY_VOLUMEDOWN
    click: KEY_MUTE

profiles:
  krita:
    wm_class:
      - krita
    button_mappings: {}   # inherits from default
    dial:
      cw: KEY_EQUAL       # zoom in
      ccw: KEY_MINUS      # zoom out
      click: null         # inherits default (mute)

  firefox:
    wm_class:
      - firefox
      - Navigator
    button_mappings: {}
    dial:
      cw: KEY_RIGHT       # next tab
      ccw: KEY_LEFT        # prev tab
      click: null
```

### Button HID codes

Use these HID codes as keys in `button_mappings`:

| Button | HID Code | Button | HID Code |
|--------|----------|--------|----------|
| 1      | `14`     | 9      | `22`     |
| 2      | `10`     | 10     | `29`     |
| 3      | `15`     | 11     | `6`      |
| 4      | `76`     | 12     | `25`     |
| 5      | `12`     | 13     | `40`     |
| 6      | `7`      | 14     | `44`     |
| 7      | `5`      | 15     | `17`     |
| 8      | `8`      |        |          |

### Dial chords

Dial actions support both single keys and key chords:

```yaml
dial:
  cw: KEY_VOLUMEUP                  # single key
  ccw:                               # key chord
    - KEY_LEFTCTRL
    - KEY_MINUS
  click: KEY_MUTE
```

### Available key names

Modifiers: `KEY_LEFTCTRL`, `KEY_LEFTSHIFT`, `KEY_LEFTALT`, `KEY_LEFTMETA`, `KEY_RIGHTCTRL`, `KEY_RIGHTSHIFT`, `KEY_RIGHTALT`, `KEY_RIGHTMETA`

Letters: `KEY_A` through `KEY_Z`

Numbers: `KEY_0` through `KEY_9`

Function keys: `KEY_F1` through `KEY_F12`

Navigation: `KEY_UP`, `KEY_DOWN`, `KEY_LEFT`, `KEY_RIGHT`, `KEY_HOME`, `KEY_END`, `KEY_PAGEUP`, `KEY_PAGEDOWN`, `KEY_INSERT`, `KEY_DELETE`

Editing: `KEY_ENTER`, `KEY_ESC`, `KEY_BACKSPACE`, `KEY_TAB`, `KEY_SPACE`

Media: `KEY_VOLUMEUP`, `KEY_VOLUMEDOWN`, `KEY_MUTE`, `KEY_PLAYPAUSE`, `KEY_NEXTSONG`, `KEY_PREVIOUSSONG`, `KEY_STOPCD`

Symbols: `KEY_MINUS`, `KEY_EQUAL`, `KEY_LEFTBRACE`, `KEY_RIGHTBRACE`, `KEY_BACKSLASH`, `KEY_SEMICOLON`, `KEY_APOSTROPHE`, `KEY_GRAVE`, `KEY_COMMA`, `KEY_DOT`, `KEY_SLASH`

Other: `KEY_CAPSLOCK`, `KEY_NUMLOCK`, `KEY_SCROLLLOCK`, `KEY_SYSRQ`, `KEY_PRINTSCREEN`, `KEY_PAUSE`, `KEY_MENU`

### Per-app profiles

Profiles match by `wm_class` — the window class name reported by your window manager. To find the class name for an app:

**KDE Wayland:** Right-click the title bar > More Actions > Configure Special Application Settings — the class is shown at the top.

**X11:**
```bash
xprop WM_CLASS    # then click the target window
```

Profile matching is case-insensitive. Unmatched windows use the `default` profile. App-specific `button_mappings` are merged on top of the default; unset buttons inherit from default. Dial settings inherit individually per field.

## CLI Usage

```
huion-keydial-mini [OPTIONS]

Options:
  -c, --config <PATH>    Path to config file [default: ~/.config/huion-keydial-mini/config.yaml]
  -d, --device <MAC>     Device MAC address (overrides config)
      --debug            Enable debug output
  -h, --help             Print help
```

Debug mode prints HID reports, key events, profile switches, and dial activity to stdout.

## Architecture

```
┌─────────────────┐     Bluetooth/GATT      ┌──────────────────┐
│  KeyDial Mini   │ ◄──────────────────────► │     BlueZ        │
└─────────────────┘                          └────────┬─────────┘
                                                      │ DBus
                                              ┌───────▼─────────┐
                                              │  huion-keydial-  │
                                              │  mini (driver)   │
                                              ├─────────────────┤
                                              │ • HID parsing    │
                                              │ • Chord emission  │
                                              │ • Profile resolve │
                                              │ • Config watcher  │
                                              └───────┬─────────┘
                                                      │ evdev/uinput
                                              ┌───────▼─────────┐
                                              │  Linux Input     │
                                              │  Subsystem       │
                                              └─────────────────┘

┌─────────────────┐     Session DBus         ┌──────────────────┐
│  KWin / xprop   │ ◄──────────────────────► │  Window Watcher  │
└─────────────────┘                          └──────────────────┘

┌─────────────────┐     config.yaml          ┌──────────────────┐
│  huion-gui      │ ◄──────────────────────► │  huion-config    │
│  (Tauri app)    │                          │  (shared crate)  │
└─────────────────┘                          └──────────────────┘
```

The workspace has three crates:

- **huion-config** — Shared configuration types, YAML serialization, profile resolution
- **huion-driver** (`huion-keydial-mini`) — Main daemon: Bluetooth connection, HID parsing, uinput virtual device, window detection, config hot-reload
- **huion-gui** (`huion-keydial-mini-gui`) — Tauri 2 desktop app for visual configuration

## Troubleshooting

**Driver starts but no keypresses register:**
- Check that only one driver instance is running: `pgrep -fa huion-keydial`
- Verify your user is in the `input` group: `groups`
- Check `/dev/uinput` permissions: `ls -la /dev/uinput`

**Profile switching not working after boot:**
- The driver retries KWin connection for up to 60 seconds after boot. Check logs: `journalctl --user -u huion-keydial-mini`
- If it still fails, restart the service once your desktop is fully loaded: `systemctl --user restart huion-keydial-mini`

**Device not found:**
- Ensure the KeyDial is paired, trusted, and connected via Bluetooth
- Check with `bluetoothctl info <MAC>` that the device shows `Connected: yes`
- The driver polls every 3 seconds for reconnection

**GUI won't launch on KDE Wayland:**
- The `.desktop` file sets `WEBKIT_DISABLE_DMABUF_RENDERER=1`. If launching manually, prefix with: `env WEBKIT_DISABLE_DMABUF_RENDERER=1 huion-keydial-mini-gui`

**Double keypresses:**
- Multiple driver instances are running. Kill extras: `pgrep -fa huion-keydial` and stop duplicates

## License

MIT
