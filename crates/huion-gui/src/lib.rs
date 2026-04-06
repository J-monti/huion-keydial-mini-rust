use huion_config::{all_key_names, ButtonInfo, Config, BUTTONS};
use serde::Serialize;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize)]
pub struct AppInfo {
    pub name: String,
    pub wm_class: String,
    pub icon: Option<String>,
    pub desktop_file: String,
}

#[tauri::command]
fn load_config() -> Result<Config, String> {
    Ok(Config::load(None))
}

#[tauri::command]
fn save_config(config: Config) -> Result<(), String> {
    config.save(None).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_available_keys() -> Vec<&'static str> {
    all_key_names()
}

#[tauri::command]
fn get_button_layout() -> &'static [ButtonInfo] {
    BUTTONS
}

#[tauri::command]
fn list_installed_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let search_dirs = [
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("applications"),
    ];

    for dir in &search_dirs {
        if let Ok(entries) = glob::glob(&format!("{}/*.desktop", dir.display())) {
            for entry in entries.flatten() {
                if let Some(app) = parse_desktop_file(&entry) {
                    // Deduplicate by wm_class
                    if !apps.iter().any(|a: &AppInfo| a.wm_class == app.wm_class) {
                        apps.push(app);
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn parse_desktop_file(path: &std::path::Path) -> Option<AppInfo> {
    let content = std::fs::read_to_string(path).ok()?;

    let mut name = None;
    let mut wm_class = None;
    let mut icon = None;
    let mut no_display = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Name=") && name.is_none() {
            name = Some(line[5..].to_string());
        } else if line.starts_with("StartupWMClass=") {
            wm_class = Some(line[15..].to_string());
        } else if line.starts_with("Icon=") {
            icon = Some(line[5..].to_string());
        } else if line == "NoDisplay=true" {
            no_display = true;
        }
    }

    if no_display {
        return None;
    }

    let name = name?;
    // Use StartupWMClass if available, otherwise use desktop file basename
    let wm_class = wm_class.unwrap_or_else(|| {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    if wm_class.is_empty() {
        return None;
    }

    Some(AppInfo {
        name,
        wm_class,
        icon,
        desktop_file: path.to_string_lossy().to_string(),
    })
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            let show = MenuItemBuilder::with_id("show", "Open Settings").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).item(&show).separator().item(&quit).build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .tooltip("Huion KeyDial Mini")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Hide window on close instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            list_available_keys,
            list_installed_apps,
            get_button_layout,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
