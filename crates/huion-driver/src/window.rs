use std::sync::Arc;
use std::time::Duration;

use tokio::process::Command;
use tokio::sync::watch;
use zbus::object_server::SignalEmitter;
use zbus::{interface, Connection};

const KWIN_SCRIPT_NAME: &str = "huion_keydial_active_window";

/// JavaScript loaded into KWin that sends active-window changes to our DBus service.
const KWIN_SCRIPT: &str = r#"
function reportWindow(w) {
    if (w) {
        callDBus("org.huion.KeyDialMini", "/Window",
                 "org.huion.KeyDialMini.Window", "SetClass",
                 w.resourceClass || "");
    }
}
reportWindow(workspace.activeWindow);
workspace.windowActivated.connect(reportWindow);
"#;

/// DBus object that receives active-window class updates from the KWin script.
struct WindowReceiver {
    tx: Arc<watch::Sender<Option<String>>>,
}

#[interface(name = "org.huion.KeyDialMini.Window")]
impl WindowReceiver {
    async fn set_class(&self, class: String, #[zbus(signal_emitter)] _emitter: SignalEmitter<'_>) {
        let val = if class.is_empty() { None } else { Some(class) };
        let _ = self.tx.send(val);
    }
}

/// Start the active-window watcher. Returns a channel immediately and retries
/// KWin/X11 in the background until a backend connects (handles the case where
/// the driver starts before the desktop session is fully up).
pub fn start_watcher() -> watch::Receiver<Option<String>> {
    let (tx, rx) = watch::channel(None);
    let tx = Arc::new(tx);

    tokio::spawn(async move {
        const MAX_ATTEMPTS: u32 = 30;
        const RETRY_INTERVAL: Duration = Duration::from_secs(2);

        for attempt in 1..=MAX_ATTEMPTS {
            // Try KWin/Wayland first
            match start_kwin_watcher(tx.clone()).await {
                Ok(()) => {
                    println!("Using KWin Wayland window detection");
                    return;
                }
                Err(e) => {
                    if attempt == 1 {
                        eprintln!("KWin watcher unavailable ({e}), trying X11 fallback");
                    }
                }
            }

            // Try X11 fallback
            match start_x11_watcher(tx.clone()).await {
                Ok(()) => {
                    println!("Using X11 window detection (xprop)");
                    return;
                }
                Err(e) => {
                    if attempt == 1 {
                        eprintln!("X11 watcher unavailable ({e})");
                        eprintln!("Retrying window detection until desktop session is ready...");
                    }
                }
            }

            tokio::time::sleep(RETRY_INTERVAL).await;
        }

        eprintln!("Window detection unavailable after {MAX_ATTEMPTS} attempts; per-app profiles disabled");
    });

    rx
}

// ---------------------------------------------------------------------------
// KWin / Wayland backend
// ---------------------------------------------------------------------------

async fn start_kwin_watcher(tx: Arc<watch::Sender<Option<String>>>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::session().await?;
    conn.object_server().at("/Window", WindowReceiver { tx }).await?;
    conn.request_name("org.huion.KeyDialMini").await?;

    // Try loading the script once to verify KWin is available
    load_kwin_script(&conn).await.map_err(|e| -> Box<dyn std::error::Error> { e })?;
    println!("KWin active-window script loaded");

    // Keep the session bus connection alive
    tokio::spawn(async move {
        let _conn = conn;
        std::future::pending::<()>().await;
    });

    Ok(())
}

async fn load_kwin_script(conn: &Connection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = conn
        .call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "unloadScript",
            &(KWIN_SCRIPT_NAME),
        )
        .await;

    let script_path = std::env::temp_dir().join("huion-keydial-kwin-active-window.js");
    tokio::fs::write(&script_path, KWIN_SCRIPT).await?;

    let reply = conn
        .call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "loadScript",
            &(script_path.to_str().unwrap_or_default(), KWIN_SCRIPT_NAME),
        )
        .await?;

    let script_id: i32 = reply.body().deserialize()?;
    if script_id < 0 {
        return Err("KWin rejected the script".into());
    }

    conn.call_method(
        Some("org.kde.KWin"),
        "/Scripting",
        Some("org.kde.kwin.Scripting"),
        "start",
        &(),
    )
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// X11 backend (xprop polling)
// ---------------------------------------------------------------------------

async fn start_x11_watcher(tx: Arc<watch::Sender<Option<String>>>) -> Result<(), Box<dyn std::error::Error>> {
    // Verify xprop is available
    let check = Command::new("xprop").arg("-root").arg("-len").arg("0").output().await?;
    if !check.status.success() {
        return Err("xprop not available or no X display".into());
    }

    tokio::spawn(async move {
        let mut last_class: Option<String> = None;
        loop {
            let class = x11_active_window_class().await;
            if class != last_class {
                last_class = class.clone();
                if tx.send(class).is_err() {
                    return; // receiver dropped
                }
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    });

    Ok(())
}

/// Get the active window's WM_CLASS via xprop.
///
/// 1. `xprop -root _NET_ACTIVE_WINDOW` → window id
/// 2. `xprop -id <id> WM_CLASS` → class name
async fn x11_active_window_class() -> Option<String> {
    // Get active window ID
    let output = Command::new("xprop")
        .args(["-root", "-notype", "_NET_ACTIVE_WINDOW"])
        .output()
        .await
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output: "_NET_ACTIVE_WINDOW: window id # 0x4e00004"
    let window_id = stdout.split("# ").nth(1)?.trim();
    if window_id == "0x0" || window_id.is_empty() {
        return None;
    }

    // Get WM_CLASS for that window
    let output = Command::new("xprop")
        .args(["-id", window_id, "-notype", "WM_CLASS"])
        .output()
        .await
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output: 'WM_CLASS = "instance", "ClassName"'
    // We want the second value (class name)
    let class_part = stdout.split('=').nth(1)?;
    let class = class_part
        .split(',')
        .nth(1)?
        .trim()
        .trim_matches('"')
        .trim();

    if class.is_empty() {
        None
    } else {
        Some(class.to_string())
    }
}
