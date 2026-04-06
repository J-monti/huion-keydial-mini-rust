use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Deserialize a dial value that can be either a single string or array of strings.
/// Accepts: `"KEY_X"` -> `Some(vec!["KEY_X"])`, `["KEY_X", "KEY_Y"]` -> `Some(vec![...])`, `null` -> `None`
fn deserialize_dial_keys<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de;

    struct DialKeysVisitor;

    impl<'de> de::Visitor<'de> for DialKeysVisitor {
        type Value = Option<Vec<String>>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("null, a key name string, or an array of key name strings")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(vec![v.to_string()]))
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(Some(vec![v]))
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut keys = Vec::new();
            while let Some(key) = seq.next_element::<String>()? {
                keys.push(key);
            }
            if keys.is_empty() {
                Ok(None)
            } else {
                Ok(Some(keys))
            }
        }
    }

    deserializer.deserialize_any(DialKeysVisitor)
}

/// Serialize dial keys: single-element vecs as a plain string, multi-element as array.
fn serialize_dial_keys<S>(value: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(keys) if keys.len() == 1 => serializer.serialize_str(&keys[0]),
        Some(keys) => keys.serialize(serializer),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub device_address: Option<String>,
    pub debug_mode: bool,
    pub default: Profile,
    pub profiles: HashMap<String, AppProfile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Profile {
    pub button_mappings: HashMap<String, Vec<String>>,
    pub dial: DialSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppProfile {
    pub wm_class: Vec<String>,
    pub button_mappings: HashMap<String, Vec<String>>,
    pub dial: DialSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DialSettings {
    #[serde(default, deserialize_with = "deserialize_dial_keys", serialize_with = "serialize_dial_keys")]
    pub cw: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_dial_keys", serialize_with = "serialize_dial_keys")]
    pub ccw: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_dial_keys", serialize_with = "serialize_dial_keys")]
    pub click: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedProfile {
    pub name: String,
    pub button_mappings: HashMap<String, Vec<String>>,
    pub dial: DialSettings,
}

#[derive(Debug, Clone, Serialize)]
pub struct ButtonInfo {
    pub hid_code: &'static str,
    pub label: &'static str,
    pub row: u8,
    pub col: u8,
    pub row_span: u8,
    pub col_span: u8,
    pub remappable: bool,
}

pub const BUTTONS: &[ButtonInfo] = &[
    ButtonInfo { hid_code: "14", label: "1",  row: 0, col: 0, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "10", label: "2",  row: 0, col: 1, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "15", label: "3",  row: 0, col: 2, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "76", label: "4",  row: 0, col: 3, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "12", label: "5",  row: 1, col: 0, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "7",  label: "6",  row: 1, col: 1, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "5",  label: "7",  row: 1, col: 2, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "8",  label: "8",  row: 1, col: 3, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "22", label: "9",  row: 2, col: 0, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "29", label: "10", row: 2, col: 1, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "6",  label: "11", row: 2, col: 2, row_span: 1, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "25", label: "12", row: 2, col: 3, row_span: 1, col_span: 1, remappable: true },
    // Row 4: native modifier buttons (not remappable)
    ButtonInfo { hid_code: "m1", label: "Ctrl",  row: 3, col: 0, row_span: 1, col_span: 1, remappable: false },
    ButtonInfo { hid_code: "m4", label: "Alt",   row: 3, col: 1, row_span: 1, col_span: 1, remappable: false },
    ButtonInfo { hid_code: "m2", label: "Shift", row: 3, col: 2, row_span: 1, col_span: 1, remappable: false },
    // Spanning buttons
    ButtonInfo { hid_code: "40", label: "13", row: 3, col: 3, row_span: 2, col_span: 1, remappable: true },
    ButtonInfo { hid_code: "44", label: "14", row: 4, col: 0, row_span: 1, col_span: 2, remappable: true },
    ButtonInfo { hid_code: "17", label: "15", row: 4, col: 2, row_span: 1, col_span: 1, remappable: true },
];

impl Default for Config {
    fn default() -> Self {
        Self {
            device_address: None,
            debug_mode: false,
            default: Profile::default(),
            profiles: HashMap::new(),
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        let mut button_mappings = HashMap::new();
        // Button 1 (HID 0x0E): Ctrl+Shift+C
        button_mappings.insert("14".into(), vec!["KEY_LEFTCTRL".into(), "KEY_LEFTSHIFT".into(), "KEY_C".into()]);
        // Button 2 (HID 0x0A): Ctrl+Shift+V
        button_mappings.insert("10".into(), vec!["KEY_LEFTCTRL".into(), "KEY_LEFTSHIFT".into(), "KEY_V".into()]);
        // Button 3 (HID 0x0F): Ctrl+C
        button_mappings.insert("15".into(), vec!["KEY_LEFTCTRL".into(), "KEY_C".into()]);
        // Button 4 (HID 0x4C): Ctrl+V
        button_mappings.insert("76".into(), vec!["KEY_LEFTCTRL".into(), "KEY_V".into()]);
        // Button 5 (HID 0x0C): Ctrl+Z (Undo)
        button_mappings.insert("12".into(), vec!["KEY_LEFTCTRL".into(), "KEY_Z".into()]);
        // Button 10 (HID 0x1D): Ctrl+Y (Redo)
        button_mappings.insert("29".into(), vec!["KEY_LEFTCTRL".into(), "KEY_Y".into()]);

        Self {
            button_mappings,
            dial: DialSettings {
                cw: Some(vec!["KEY_VOLUMEUP".into()]),
                ccw: Some(vec!["KEY_VOLUMEDOWN".into()]),
                click: Some(vec!["KEY_MUTE".into()]),
            },
        }
    }
}

impl Default for AppProfile {
    fn default() -> Self {
        Self {
            wm_class: Vec::new(),
            button_mappings: HashMap::new(),
            dial: DialSettings::default(),
        }
    }
}

impl Default for DialSettings {
    fn default() -> Self {
        Self {
            cw: None,
            ccw: None,
            click: None,
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> Self {
        let config_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            Self::default_config_path()
        };

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match serde_yaml::from_str(&contents) {
                    Ok(config) => {
                        println!("Loaded config from {}", config_path.display());
                        return config;
                    }
                    Err(e) => eprintln!("Warning: failed to parse {}: {e}", config_path.display()),
                },
                Err(e) => eprintln!("Warning: failed to read {}: {e}", config_path.display()),
            }
        }

        println!("Using default config");
        Self::default()
    }

    pub fn save(&self, path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            Self::default_config_path()
        };

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let yaml = serde_yaml::to_string(self)?;
        let tmp_path = config_path.with_extension("yaml.tmp");
        std::fs::write(&tmp_path, yaml)?;
        std::fs::rename(&tmp_path, &config_path)?;
        Ok(())
    }

    pub fn resolve_profile(&self, wm_class: Option<&str>) -> ResolvedProfile {
        let wm_class = match wm_class {
            Some(c) => c,
            None => return self.default_resolved(),
        };

        let wm_lower = wm_class.to_lowercase();

        for (name, app_profile) in &self.profiles {
            let matched = app_profile.wm_class.iter().any(|c| c.to_lowercase() == wm_lower);
            if matched {
                return self.merge_with_app(name, app_profile);
            }
        }

        self.default_resolved()
    }

    fn default_resolved(&self) -> ResolvedProfile {
        ResolvedProfile {
            name: "default".into(),
            button_mappings: self.default.button_mappings.clone(),
            dial: self.default.dial.clone(),
        }
    }

    fn merge_with_app(&self, name: &str, app: &AppProfile) -> ResolvedProfile {
        let mut button_mappings = self.default.button_mappings.clone();
        for (k, v) in &app.button_mappings {
            button_mappings.insert(k.clone(), v.clone());
        }

        let dial = DialSettings {
            cw: app.dial.cw.clone().or_else(|| self.default.dial.cw.clone()),
            ccw: app.dial.ccw.clone().or_else(|| self.default.dial.ccw.clone()),
            click: app.dial.click.clone().or_else(|| self.default.dial.click.clone()),
        };

        ResolvedProfile {
            name: name.into(),
            button_mappings,
            dial,
        }
    }

    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("huion-keydial-mini")
            .join("config.yaml")
    }
}

pub fn all_key_names() -> Vec<&'static str> {
    vec![
        "KEY_A", "KEY_B", "KEY_C", "KEY_D", "KEY_E", "KEY_F",
        "KEY_G", "KEY_H", "KEY_I", "KEY_J", "KEY_K", "KEY_L",
        "KEY_M", "KEY_N", "KEY_O", "KEY_P", "KEY_Q", "KEY_R",
        "KEY_S", "KEY_T", "KEY_U", "KEY_V", "KEY_W", "KEY_X",
        "KEY_Y", "KEY_Z",
        "KEY_1", "KEY_2", "KEY_3", "KEY_4", "KEY_5", "KEY_6",
        "KEY_7", "KEY_8", "KEY_9", "KEY_0",
        "KEY_F1", "KEY_F2", "KEY_F3", "KEY_F4", "KEY_F5", "KEY_F6",
        "KEY_F7", "KEY_F8", "KEY_F9", "KEY_F10", "KEY_F11", "KEY_F12",
        "KEY_ENTER", "KEY_ESC", "KEY_BACKSPACE", "KEY_TAB", "KEY_SPACE",
        "KEY_MINUS", "KEY_EQUAL",
        "KEY_LEFTBRACE", "KEY_RIGHTBRACE", "KEY_BACKSLASH",
        "KEY_SEMICOLON", "KEY_APOSTROPHE", "KEY_GRAVE",
        "KEY_COMMA", "KEY_DOT", "KEY_SLASH", "KEY_CAPSLOCK",
        "KEY_SYSRQ", "KEY_PRINTSCREEN", "KEY_SCROLLLOCK", "KEY_PAUSE",
        "KEY_INSERT", "KEY_HOME", "KEY_PAGEUP",
        "KEY_DELETE", "KEY_END", "KEY_PAGEDOWN",
        "KEY_RIGHT", "KEY_LEFT", "KEY_DOWN", "KEY_UP",
        "KEY_NUMLOCK",
        "KEY_LEFTCTRL", "KEY_LEFTSHIFT", "KEY_LEFTALT", "KEY_LEFTMETA",
        "KEY_RIGHTCTRL", "KEY_RIGHTSHIFT", "KEY_RIGHTALT", "KEY_RIGHTMETA",
        "KEY_VOLUMEUP", "KEY_VOLUMEDOWN", "KEY_MUTE",
        "KEY_PLAYPAUSE", "KEY_NEXTSONG", "KEY_PREVIOUSSONG", "KEY_STOPCD",
        "KEY_MENU",
    ]
}
