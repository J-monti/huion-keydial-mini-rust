mod bluetooth;
mod config;
mod hid;
mod uinput;

use std::collections::HashMap;

use clap::Parser;
use evdev::uinput::VirtualDevice;
use futures_util::StreamExt;
use tokio::signal;
use zbus::Connection;
use zbus::zvariant::OwnedValue;

use crate::config::Config;

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

    if let Some(mac) = &cli.device {
        cfg.device_address = Some(mac.clone());
    }
    if cli.debug {
        cfg.debug_mode = true;
    }

    println!("Huion KeyDial Mini driver");

    // Create virtual input device
    let mut vdev = uinput::create_device()?;
    println!("Created virtual input device");

    // Connect to system DBus
    let conn = Connection::system().await?;
    println!("Connected to system DBus");

    // Main loop: find device, attach, process events, reconnect on disconnect
    loop {
        match run_device_session(&conn, &cfg, &mut vdev).await {
            Ok(()) => println!("Device disconnected"),
            Err(e) => eprintln!("Session error: {e}"),
        }

        println!("Waiting for device to reconnect...");

        // Wait for a device to appear, polling every 3 seconds
        // Also listen for Ctrl+C
        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    println!("\nShutting down");
                    return Ok(());
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                    if let Ok(Some(_)) = bluetooth::find_device(&conn, cfg.device_address.as_deref()).await {
                        break;
                    }
                }
            }
        }
    }
}

async fn run_device_session(
    conn: &Connection,
    cfg: &Config,
    vdev: &mut VirtualDevice,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find the device
    let device = match bluetooth::find_device(conn, cfg.device_address.as_deref()).await? {
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

                match data.len() {
                    8 => process_hid_report(&data, &mut prev_modifiers, &mut prev_keys, vdev, cfg)?,
                    6 => process_dial_report(&data, vdev, cfg)?,
                    2 => process_dial_click(&data, vdev, cfg)?,
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
) -> Result<(), Box<dyn std::error::Error>> {
    let dial_val = data[5] as i8;
    if dial_val == 0 { return Ok(()); }

    let (key, direction) = if dial_val < 0 {
        // 0xff (-1) = clockwise
        let key_name = cfg.dial_settings.dial_cw.as_deref().unwrap_or("KEY_VOLUMEUP");
        (hid::key_name_to_evdev(key_name), "CW")
    } else {
        // 0x01 (+1) = counterclockwise
        let key_name = cfg.dial_settings.dial_ccw.as_deref().unwrap_or("KEY_VOLUMEDOWN");
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
) -> Result<(), Box<dyn std::error::Error>> {
    let pressed = data[0] != 0;
    let key_name = cfg.dial_settings.dial_click.as_deref().unwrap_or("KEY_MUTE");

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
