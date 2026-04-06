use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub device_address: Option<String>,
    pub debug_mode: bool,
    pub default: Profile,
    pub profiles: HashMap<String, AppProfile>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Profile {
    pub button_mappings: HashMap<String, Vec<String>>,
    pub dial: DialSettings,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AppProfile {
    pub wm_class: Vec<String>,
    pub button_mappings: HashMap<String, Vec<String>>,
    pub dial: DialSettings,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DialSettings {
    pub cw: Option<String>,
    pub ccw: Option<String>,
    pub click: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    pub button_mappings: HashMap<String, Vec<String>>,
    pub dial: DialSettings,
}

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

        Self {
            button_mappings,
            dial: DialSettings {
                cw: Some("KEY_VOLUMEUP".into()),
                ccw: Some("KEY_VOLUMEDOWN".into()),
                click: Some("KEY_MUTE".into()),
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

    pub fn resolve_profile(&self, wm_class: Option<&str>) -> ResolvedProfile {
        let wm_class = match wm_class {
            Some(c) => c,
            None => return self.default_resolved(),
        };

        let wm_lower = wm_class.to_lowercase();

        for app_profile in self.profiles.values() {
            let matched = app_profile.wm_class.iter().any(|c| c.to_lowercase() == wm_lower);
            if matched {
                return self.merge_with_app(app_profile);
            }
        }

        self.default_resolved()
    }

    fn default_resolved(&self) -> ResolvedProfile {
        ResolvedProfile {
            button_mappings: self.default.button_mappings.clone(),
            dial: self.default.dial.clone(),
        }
    }

    fn merge_with_app(&self, app: &AppProfile) -> ResolvedProfile {
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
