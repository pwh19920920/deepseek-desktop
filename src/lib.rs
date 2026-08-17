use std::path::PathBuf;
use std::sync::mpsc;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::runtime::Runtime;
use tracing::{info, error};

pub mod sidecar;
pub mod capabilities;

/// Handle to the running harness sidecar, stored in Tauri state.
#[derive(Clone)]
pub struct SidecarState {
    pub port: u16,
    child: std::sync::Arc<std::sync::Mutex<Option<tokio::process::Child>>>,
    #[allow(dead_code)]
    dsh_path: PathBuf,
}

impl SidecarState {
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

/// Message sent from the sidecar thread to the main Tauri thread.
enum SidecarMessage {
    Ready(sidecar::SidecarHandle),
    Error(String),
}

fn find_node() -> anyhow::Result<PathBuf> {
    which::which("node").map_err(|e| anyhow::anyhow!("node not found in PATH: {}", e))
}

/// Resolve the path to the `dsh` CLI binary.
fn resolve_dsh_path() -> PathBuf {
    let local_bin = PathBuf::from("node_modules/.bin/dsh");
    if local_bin.exists() {
        return local_bin;
    }
    let lib_bin = PathBuf::from("node_modules/@deepseek-ai/dsh/lib/bin.js");
    if lib_bin.exists() {
        return lib_bin;
    }
    if let Ok(path) = which::which("dsh") {
        return path;
    }
    PathBuf::from("dsh")
}

/// Clean up the sidecar process. Must be called on the Tokio runtime thread.
fn shutdown_sidecar_blocking(state: &SidecarState) -> anyhow::Result<()> {
    let mut child_opt = state.child.lock().unwrap();
    if let Some(mut child) = child_opt.take() {
        // Spawn the async kill on a blocking thread since we can't await here
        std::thread::spawn(move || {
            let rt = Runtime::new().expect("failed to create tokio runtime for shutdown");
            rt.block_on(async {
                let _ = child.kill().await;
                let _ = child.wait().await;
            });
        });
    }
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            capabilities::file_picker::pick_directory,
            capabilities::file_picker::pick_file,
            capabilities::file_picker::list_directory,
            capabilities::notifications::send_notification,
            capabilities::notifications::is_notification_allowed,
            capabilities::opener::open_path,
            capabilities::opener::open_text_file,
            capabilities::opener::open_url,
        ])
        .setup(|app| {
            let (tx, rx) = mpsc::channel::<SidecarMessage>();
            let handle = app.handle().clone();

            std::thread::spawn(move || {
                let rt = Runtime::new().expect("failed to create tokio runtime");
                let result = rt.block_on(async {
                    let node = find_node()?;
                    let dsh_path = resolve_dsh_path();
                    info!(
                        "spawning harness sidecar node={:?} dsh={:?}",
                        node, dsh_path
                    );
                    sidecar::spawn_sidecar(node, dsh_path).await
                });

                match result {
                    Ok(sc) => { let _ = tx.send(SidecarMessage::Ready(sc)); }
                    Err(e) => { let _ = tx.send(SidecarMessage::Error(e.to_string())); }
                }
            });

            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("DeepSeek Harness Desktop")
                .inner_size(1280.0, 800.0)
                .resizable(true)
                .build()?;

            match rx.recv() {
                Ok(SidecarMessage::Ready(sc)) => {
                    let url_str = sc.url();
                    info!("harness sidecar ready at {}", url_str);
                    let state = SidecarState {
                        port: sc.port,
                        child: std::sync::Arc::new(std::sync::Mutex::new(Some(sc.child))),
                        dsh_path: sc.dsh_path,
                    };
                    app.manage(state);
                    if let Some(window) = handle.get_webview_window("main") {
                        let _ = window.navigate(url_str.parse().expect("valid URL"));
                    }
                }
                Ok(SidecarMessage::Error(e)) => {
                    error!("failed to spawn harness sidecar: {}", e);
                    if let Some(window) = handle.get_webview_window("main") {
                        let _ = window.set_title("DeepSeek Harness - Error");
                    }
                }
                Err(e) => {
                    error!("failed to receive sidecar message: {}", e);
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(state) = window.try_state::<SidecarState>() {
                    // Shutdown is synchronous from the main thread
                    if let Err(e) = shutdown_sidecar_blocking(&state) {
                        error!("error shutting down sidecar: {}", e);
                    }
                }
                api.prevent_close();
                window.hide().unwrap_or_default();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running deepseek-desktop");
}
