use std::collections::HashMap;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue};

const HUION_NAMES: &[&str] = &["huion", "keydial"];

pub struct DeviceInfo {
    pub mac: String,
    pub name: String,
    pub dbus_path: String,
}

/// Convert MAC address to BlueZ DBus path
pub fn mac_to_dbus_path(mac: &str) -> String {
    format!("/org/bluez/hci0/dev_{}", mac.replace(':', "_"))
}

/// Find a connected Huion device, optionally filtered by MAC address
pub async fn find_device(conn: &Connection, target_mac: Option<&str>) -> Result<Option<DeviceInfo>, Box<dyn std::error::Error>> {
    let root_path = ObjectPath::try_from("/")?;
    let reply = conn.call_method(
        Some("org.bluez"), &root_path,
        Some("org.freedesktop.DBus.ObjectManager"), "GetManagedObjects",
        &(),
    ).await?;

    let objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> =
        reply.body().deserialize()?;

    for (path, interfaces) in &objects {
        let path_str = path.as_str();
        let Some(device_props) = interfaces.get("org.bluez.Device1") else { continue };

        // Check if connected
        let connected = device_props.get("Connected")
            .and_then(|v| bool::try_from(v.clone()).ok())
            .unwrap_or(false);
        if !connected { continue; }

        let name = device_props.get("Name")
            .and_then(|v| String::try_from(v.clone()).ok())
            .unwrap_or_default();

        let address = device_props.get("Address")
            .and_then(|v| String::try_from(v.clone()).ok())
            .unwrap_or_default();

        if let Some(target) = target_mac {
            if address.to_uppercase() == target.to_uppercase() {
                return Ok(Some(DeviceInfo {
                    mac: address,
                    name,
                    dbus_path: path_str.to_string(),
                }));
            }
        } else {
            let name_lower = name.to_lowercase();
            if HUION_NAMES.iter().any(|h| name_lower.contains(h)) {
                return Ok(Some(DeviceInfo {
                    mac: address,
                    name,
                    dbus_path: path_str.to_string(),
                }));
            }
        }
    }

    Ok(None)
}

/// Get all notify-capable GATT characteristics for a device
pub async fn get_notify_characteristics(
    conn: &Connection,
    device_path: &str,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let root_path = ObjectPath::try_from("/")?;
    let reply = conn.call_method(
        Some("org.bluez"), &root_path,
        Some("org.freedesktop.DBus.ObjectManager"), "GetManagedObjects",
        &(),
    ).await?;

    let objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> =
        reply.body().deserialize()?;

    let mut chars = Vec::new();
    for (path, interfaces) in &objects {
        let path_str = path.as_str();
        if !path_str.starts_with(device_path) { continue; }
        let Some(char_props) = interfaces.get("org.bluez.GattCharacteristic1") else { continue };
        let Some(flags_val) = char_props.get("Flags") else { continue };
        let Ok(flags) = <Vec<String>>::try_from(flags_val.clone()) else { continue };
        if !flags.iter().any(|f| f == "notify") { continue; }
        let uuid = char_props.get("UUID")
            .and_then(|v| String::try_from(v.clone()).ok())
            .unwrap_or_default();
        chars.push((path_str.to_string(), uuid));
    }

    Ok(chars)
}

/// Start GATT notifications for a list of characteristic paths
pub async fn start_notifications(
    conn: &Connection,
    char_paths: &[(String, String)],
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut subscribed = 0;
    for (char_path, uuid) in char_paths {
        let cp = ObjectPath::try_from(char_path.as_str())?;
        match conn.call_method(
            Some("org.bluez"), &cp,
            Some("org.bluez.GattCharacteristic1"), "StartNotify", &(),
        ).await {
            Ok(_) => {
                let short = char_path.split('/').last().unwrap_or(char_path);
                println!("  Subscribed: {uuid} ({short})");
                subscribed += 1;
            }
            Err(e) => {
                let short = char_path.split('/').last().unwrap_or(char_path);
                eprintln!("  Failed: {uuid} ({short}): {e}");
            }
        }
    }
    Ok(subscribed)
}

/// Subscribe to PropertiesChanged signals for a device path
pub async fn subscribe_signals(
    conn: &Connection,
    device_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let match_rule = format!(
        "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path_namespace='{device_path}'"
    );
    let dbus_path = ObjectPath::try_from("/org/freedesktop/DBus")?;
    conn.call_method(
        Some("org.freedesktop.DBus"), &dbus_path,
        Some("org.freedesktop.DBus"), "AddMatch", &(&match_rule,),
    ).await?;
    Ok(())
}
