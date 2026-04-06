mod bluetooth;
mod hid;
mod uinput;
mod window;

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use clap::Parser;
use evdev::uinput::VirtualDevice;
use futures_util::StreamExt;
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::signal;
use tokio::sync::watch;
use zbus::Connection;
use zbus::zvariant::OwnedValue;

use huion_config::{Config, ResolvedProfile};

#[derive(Parser)]
#[command(name = "huion-keydial-mini", about = "Huion KeyDial Mini driver")]
struct Cli {
    /// Path to config file
    #[arg(short, long)]
    config: Option<String>,

    /// Device MAC address (overrides config)
    #[arg(short, long)]
    device: Option<String>,

    /// Enable debug output
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut cfg = Config::load(cli.config.as_deref());

    // Apply CLI overrides
    let cli_device = cli.device.clone();
    let cli_debug = cli.debug;
    if let Some(mac) = &cli_device {
        cfg.device_address = Some(mac.clone());
    }
    if cli_debug {
        cfg.debug_mode = true;
    }

    println!("Huion KeyDial Mini driver");

    // Set up config watch channel
    let (cfg_tx, cfg_rx) = watch::channel(cfg);

    // Determine config file path for watching
    let config_path = cli.config.as_ref()
        .map(|p| std::path::PathBuf::from(p))
        .unwrap_or_else(Config::default_config_path);

    // Spawn config file watcher
    spawn_config_watcher(config_path, cfg_tx, cli_device, cli_debug);

    // Create virtual input device
    let mut vdev = uinput::create_device()?;
    println!("Created virtual input device");

    // Connect to system DBus
    let conn = Connection::system().await?;
    println!("Connected to system DBus");

    // Start active window watcher (KWin script via session DBus)
    let window_rx = match window::start_watcher().await {
        Ok(rx) => {
            println!("Started active window watcher");
            Some(rx)
        }
        Err(e) => {
            eprintln!("Warning: window watcher unavailable: {e}");
            eprintln!("Per-app profiles will not auto-switch; using default profile");
            None
        }
    };

    // Main loop: find device, attach, process events, reconnect on disconnect
    loop {
        match run_device_session(&conn, &cfg_rx, &mut vdev, window_rx.as_ref()).await {
            Ok(()) => println!("Device disconnected"),
            Err(e) => eprintln!("Session error: {e}"),
        }

        println!("Waiting for device to reconnect...");

        // Wait for a device to appear, polling every 3 seconds
        // Also listen for Ctrl+C
        loop {
            let device_address = cfg_rx.borrow().device_address.clone();
            tokio::select! {
                _ = signal::ctrl_c() => {
                    println!("\nShutting down");
                    return Ok(());
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                    if let Ok(Some(_)) = bluetooth::find_device(&conn, device_address.as_deref()).await {
                        break;
                    }
                }
            }
        }
    }
}

fn spawn_config_watcher(
    config_path: std::path::PathBuf,
    cfg_tx: watch::Sender<Config>,
    cli_device: Option<String>,
    cli_debug: bool,
) {
    let config_filename = config_path.file_name()
        .unwrap_or_default()
        .to_os_string();
    let watch_dir = config_path.parent()
        .unwrap_or_else(|| Path::new("/"))
        .to_path_buf();

    // notify uses std::sync::mpsc, so we bridge to async via spawn_blocking
    let (fs_tx, fs_rx) = std::sync::mpsc::channel();

    // Start the filesystem watcher
    let _watcher = match notify::recommended_watcher(fs_tx) {
        Ok(mut w) => {
            if let Err(e) = w.watch(&watch_dir, RecursiveMode::NonRecursive) {
                eprintln!("Warning: could not watch config directory: {e}");
                return;
            }
            println!("Watching config file: {}", config_path.display());
            // Move watcher into the async task so it stays alive
            w
        }
        Err(e) => {
            eprintln!("Warning: could not create config watcher: {e}");
            return;
        }
    };

    tokio::task::spawn_blocking(move || {
        let _keep_alive = _watcher;
        let mut last_reload = Instant::now();

        loop {
            match fs_rx.recv() {
                Ok(Ok(event)) => {
                    // Only react to modify events
                    if !matches!(event.kind, EventKind::Modify(_)) {
                        continue;
                    }

                    // Filter for our config filename
                    let is_config = event.paths.iter().any(|p| {
                        p.file_name()
                            .map(|f| f == config_filename)
                            .unwrap_or(false)
                    });
                    if !is_config {
                        continue;
                    }

                    // Debounce: skip if < 500ms since last reload
                    let now = Instant::now();
                    if now.duration_since(last_reload).as_millis() < 500 {
                        continue;
                    }
                    last_reload = now;

                    // Reload config
                    let config_path_str = config_path.to_string_lossy().to_string();
                    let mut new_cfg = Config::load(Some(&config_path_str));

                    // Re-apply CLI overrides
                    if let Some(ref mac) = cli_device {
                        new_cfg.device_address = Some(mac.clone());
                    }
                    if cli_debug {
                        new_cfg.debug_mode = true;
                    }

                    match cfg_tx.send(new_cfg) {
                        Ok(()) => println!("Config reloaded"),
                        Err(_) => {
                            // Receiver dropped, main task is gone
                            break;
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Config reload failed: {e}");
                }
                Err(_) => {
                    // Channel closed, watcher dropped
                    break;
                }
            }
        }
    });
}

async fn run_device_session(
    conn: &Connection,
    cfg_rx: &watch::Receiver<Config>,
    vdev: &mut VirtualDevice,
    window_rx: Option<&tokio::sync::watch::Receiver<Option<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find the device
    let device_address = cfg_rx.borrow().device_address.clone();
    let device = match bluetooth::find_device(conn, device_address.as_deref()).await? {
        Some(d) => d,
        None => {
            println!("No connected Huion device found");
            return Ok(());
        }
    };

    println!("Found: {} ({})", device.name, device.mac);

    // Get and subscribe to notify characteristics
    let chars = bluetooth::get_notify_characteristics(conn, &device.dbus_path).await?;
    println!("Found {} notify characteristics", chars.len());

    let subscribed = bluetooth::start_notifications(conn, &chars).await?;
    if subscribed == 0 {
        return Err("Could not subscribe to any characteristics".into());
    }
    println!("Subscribed to {subscribed} characteristics");

    // Subscribe to DBus signals
    bluetooth::subscribe_signals(conn, &device.dbus_path).await?;

    println!("\n=== Listening for input ===\n");

    // Process events
    let mut prev_modifiers: u8 = 0;
    let mut prev_keys: Vec<u8> = Vec::new();
    let mut stream = zbus::MessageStream::from(conn);

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("\nShutting down");
                std::process::exit(0);
            }
            msg_result = stream.next() => {
                let Some(msg_result) = msg_result else { break };
                let msg = match msg_result {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let hdr = msg.header();
                if hdr.member().map(|m| m.as_str()) != Some("PropertiesChanged") { continue; }
                let msg_path = match hdr.path() {
                    Some(p) => p.as_str().to_string(),
                    None => continue,
                };
                if !msg_path.starts_with(&device.dbus_path) { continue; }

                let body = msg.body();
                let Ok((iface, changed, _inv)): Result<(String, HashMap<String, OwnedValue>, Vec<String>), _> =
                    body.deserialize() else { continue };

                if iface != "org.bluez.GattCharacteristic1" { continue; }
                let Some(value) = changed.get("Value") else { continue };
                let Ok(data) = <Vec<u8>>::try_from(value.clone()) else { continue };

                // Check for disconnect (device property change)
                if iface == "org.bluez.Device1" {
                    if let Some(conn_val) = changed.get("Connected") {
                        if let Ok(false) = bool::try_from(conn_val.clone()) {
                            println!("Device disconnected via DBus");
                            return Ok(());
                        }
                    }
                }

                // Get current config snapshot
                let cfg = cfg_rx.borrow().clone();

                // Resolve active profile based on focused window
                let active_wm_class = window_rx.map(|rx| rx.borrow().clone()).unwrap_or(None);
                let profile = cfg.resolve_profile(active_wm_class.as_deref());

                if cfg.debug_mode {
                    if let Some(ref class) = active_wm_class {
                        println!("  [window: {class}]");
                    }
                }

                match data.len() {
                    8 => process_hid_report(&data, &mut prev_modifiers, &mut prev_keys, vdev, &cfg, &profile)?,
                    6 => process_dial_report(&data, vdev, &cfg, &profile)?,
                    2 => process_dial_click(&data, vdev, &cfg, &profile)?,
                    1 => {
                        if cfg.debug_mode { println!("Battery: {}%", data[0]); }
                    }
                    _ => {
                        if cfg.debug_mode { eprintln!("Unknown data ({}B): {:02x?}", data.len(), data); }
                    }
                }
            }
        }
    }

    Ok(())
}

fn process_hid_report(
    data: &[u8],
    prev_modifiers: &mut u8,
    prev_keys: &mut Vec<u8>,
    vdev: &mut VirtualDevice,
    cfg: &Config,
    profile: &ResolvedProfile,
) -> Result<(), Box<dyn std::error::Error>> {
    let modifiers = data[0];
    let cur_keys: Vec<u8> = data[2..8].iter().copied().filter(|&k| k != 0).collect();

    if cfg.debug_mode {
        println!("HID: mod=0x{modifiers:02x} keys={cur_keys:?}");
    }

    // Modifier changes
    let mod_changed = modifiers ^ *prev_modifiers;
    for bit in 0..8u8 {
        let mask = 1 << bit;
        if mod_changed & mask != 0 {
            for key in hid::modifier_keys(mask) {
                let pressed = modifiers & mask != 0;
                vdev.emit(&[evdev::InputEvent::new(
                    evdev::EventType::KEY, key.code(), i32::from(pressed),
                )])?;
                if cfg.debug_mode {
                    println!("  {} modifier: {key:?}", if pressed { "PRESS" } else { "RELEASE" });
                }
            }
        }
    }

    // Released keys
    for &k in prev_keys.iter() {
        if !cur_keys.contains(&k) {
            // Skip release for remapped buttons (chord was tapped on press)
            if profile.button_mappings.contains_key(&k.to_string()) {
                continue;
            }
            if let Some(key) = hid::hid_to_key(k) {
                vdev.emit(&[evdev::InputEvent::new(evdev::EventType::KEY, key.code(), 0)])?;
                if cfg.debug_mode {
                    println!("  RELEASE: {key:?}");
                }
            }
        }
    }

    // Pressed keys
    for &k in &cur_keys {
        if !prev_keys.contains(&k) {
            // Check for button remapping
            if let Some(chord) = profile.button_mappings.get(&k.to_string()) {
                let keys: Vec<_> = chord.iter().filter_map(|name| hid::key_name_to_evdev(name)).collect();
                // Tap: press all atomically, then release all atomically
                let presses: Vec<_> = keys.iter().map(|key| evdev::InputEvent::new(evdev::EventType::KEY, key.code(), 1)).collect();
                let releases: Vec<_> = keys.iter().rev().map(|key| evdev::InputEvent::new(evdev::EventType::KEY, key.code(), 0)).collect();
                vdev.emit(&presses)?;
                vdev.emit(&releases)?;
                if cfg.debug_mode {
                    println!("  CHORD: {chord:?}");
                }
                continue;
            }
            if let Some(key) = hid::hid_to_key(k) {
                vdev.emit(&[evdev::InputEvent::new(evdev::EventType::KEY, key.code(), 1)])?;
                if cfg.debug_mode {
                    println!("  PRESS: {key:?}");
                }
            }
        }
    }

    *prev_modifiers = modifiers;
    *prev_keys = cur_keys;
    Ok(())
}

fn process_dial_report(
    data: &[u8],
    vdev: &mut VirtualDevice,
    cfg: &Config,
    profile: &ResolvedProfile,
) -> Result<(), Box<dyn std::error::Error>> {
    let dial_val = data[5] as i8;
    if dial_val == 0 { return Ok(()); }

    let (key, direction) = if dial_val < 0 {
        // 0xff (-1) = clockwise
        let key_name = profile.dial.cw.as_deref().unwrap_or("KEY_VOLUMEUP");
        (hid::key_name_to_evdev(key_name), "CW")
    } else {
        // 0x01 (+1) = counterclockwise
        let key_name = profile.dial.ccw.as_deref().unwrap_or("KEY_VOLUMEDOWN");
        (hid::key_name_to_evdev(key_name), "CCW")
    };

    if let Some(key) = key {
        // Tap: press then release
        vdev.emit(&[evdev::InputEvent::new(evdev::EventType::KEY, key.code(), 1)])?;
        vdev.emit(&[evdev::InputEvent::new(evdev::EventType::KEY, key.code(), 0)])?;
        if cfg.debug_mode {
            println!("  DIAL {direction}: {key:?}");
        }
    }

    Ok(())
}

fn process_dial_click(
    data: &[u8],
    vdev: &mut VirtualDevice,
    cfg: &Config,
    profile: &ResolvedProfile,
) -> Result<(), Box<dyn std::error::Error>> {
    let pressed = data[0] != 0;
    let key_name = profile.dial.click.as_deref().unwrap_or("KEY_MUTE");

    if let Some(key) = hid::key_name_to_evdev(key_name) {
        vdev.emit(&[evdev::InputEvent::new(
            evdev::EventType::KEY, key.code(), i32::from(pressed),
        )])?;
        if cfg.debug_mode {
            println!("  DIAL CLICK {}: {key:?}", if pressed { "PRESS" } else { "RELEASE" });
        }
    }

    Ok(())
}
