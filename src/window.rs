use std::sync::Arc;
use std::time::Duration;

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

/// Start the active-window watcher. Returns a receiver with the current WM class.
///
/// 1. Registers a DBus service (`org.huion.KeyDialMini`) on the session bus.
/// 2. Loads a small KWin script that hooks `workspace.windowActivated` and
///    calls our `SetClass` method whenever focus changes.
pub async fn start_watcher() -> Result<watch::Receiver<Option<String>>, Box<dyn std::error::Error>> {
    let (tx, rx) = watch::channel(None);
    let tx = Arc::new(tx);

    // Register our DBus service on the session bus
    let conn = Connection::session().await?;
    conn.object_server().at("/Window", WindowReceiver { tx }).await?;
    conn.request_name("org.huion.KeyDialMini").await?;

    // Spawn task to load the KWin script and keep the session bus connection alive
    tokio::spawn(async move {
        loop {
            let result = load_kwin_script(&conn).await;
            match result {
                Ok(()) => {
                    println!("KWin active-window script loaded");
                    break;
                }
                Err(e) => {
                    eprintln!("Window watcher: failed to load KWin script: {e}");
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        // Keep connection alive so our DBus object server continues to receive calls
        std::future::pending::<()>().await;
    });

    Ok(rx)
}

/// Write the JS to a temp file, load it into KWin via Scripting DBus, and start it.
async fn load_kwin_script(conn: &Connection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Unload any previous instance
    let _ = conn
        .call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "unloadScript",
            &(KWIN_SCRIPT_NAME),
        )
        .await;

    // Write script to temp file
    let script_path = std::env::temp_dir().join("huion-keydial-kwin-active-window.js");
    tokio::fs::write(&script_path, KWIN_SCRIPT).await?;

    // Load the script
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

    // Start all loaded scripts
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
