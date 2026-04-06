use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub device_address: Option<String>,
    pub debug_mode: bool,
    pub key_mappings: HashMap<String, String>,
    pub dial_settings: DialSettings,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DialSettings {
    #[serde(rename = "DIAL_CW")]
    pub dial_cw: Option<String>,
    #[serde(rename = "DIAL_CCW")]
    pub dial_ccw: Option<String>,
    #[serde(rename = "DIAL_CLICK")]
    pub dial_click: Option<String>,
    pub sensitivity: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device_address: None,
            debug_mode: false,
            key_mappings: HashMap::new(),
            dial_settings: DialSettings::default(),
        }
    }
}

impl Default for DialSettings {
    fn default() -> Self {
        Self {
            dial_cw: Some("KEY_VOLUMEUP".into()),
            dial_ccw: Some("KEY_VOLUMEDOWN".into()),
            dial_click: Some("KEY_MUTE".into()),
            sensitivity: 1.0,
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

    fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("huion-keydial-mini")
            .join("config.yaml")
    }
}
