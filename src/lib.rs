use std::path::PathBuf;
use std::sync::mpsc;
use tauri::Manager;
use tauri_plugin_shell::process::CommandChild;
use tokio::runtime::Runtime;
use tracing::{error, info};

pub mod capabilities;
pub mod sidecar;

/// Handle to the running harness sidecar, stored in Tauri state.
#[derive(Clone)]
pub struct SidecarState {
    pub port: u16,
    child: std::sync::Arc<std::sync::Mutex<Option<CommandChild>>>,
    #[allow(dead_code)]
    dsh_path: PathBuf,
}

impl SidecarState {
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

enum SidecarMessage {
    Ready(sidecar::SidecarHandle),
    Error(String),
}

/// Resolve the path to dsh's `lib/bin.js`.
///
/// Priority:
/// 1. Bundled resource in the .app (`Contents/Resources/resources/dsh/lib/bin.js`) — release builds
/// 2. Project-local `resources/dsh/lib/bin.js` — dev builds from this repo
/// 3. `node_modules/@deepseek-ai/dsh/lib/bin.js` — dev builds with pnpm workspace
fn resolve_dsh_path(app_resource_dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    // Release build: dsh is bundled under {resource_dir}/resources/dsh/lib/bin.js
    // (Tauri preserves the source directory structure when copying resources)
    if let Some(ref resource_dir) = app_resource_dir {
        let bundled = resource_dir
            .join("resources")
            .join("dsh")
            .join("lib")
            .join("bin.js");
        if bundled.exists() {
            return Ok(bundled);
        }
    }
    // Dev build: look in project-local resources/
    let local = PathBuf::from("resources/dsh/lib/bin.js");
    if local.exists() {
        return Ok(local);
    }
    // Dev build with pnpm workspace: fall back to node_modules
    let node_modules = PathBuf::from("node_modules/@deepseek-ai/dsh/lib/bin.js");
    if node_modules.exists() {
        return Ok(node_modules);
    }
    anyhow::bail!(
        "dsh not found. Searched:\n  - bundled resource ({:?}/resources/dsh/lib/bin.js)\n  - resources/dsh/lib/bin.js\n  - node_modules/@deepseek-ai/dsh/lib/bin.js",
        app_resource_dir.as_ref().map(|d| d.display().to_string()).unwrap_or_default()
    )
}

fn shutdown_sidecar_blocking(state: &SidecarState) -> anyhow::Result<()> {
    let mut child_opt = state.child.lock().unwrap();
    if let Some(child) = child_opt.take() {
        std::thread::spawn(move || {
            let _ = child.kill();
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

            // Resolve dsh path before moving handle into the thread
            let resource_dir = app.path().resource_dir().unwrap_or_default();
            let dsh_path = resolve_dsh_path(Some(resource_dir.clone()))?;
            info!("dsh resolved at {:?}", dsh_path);

            // Spawn sidecar in a separate thread
            let app_for_sidecar = app.handle().clone();
            std::thread::spawn(move || {
                let rt = Runtime::new().expect("failed to create tokio runtime");
                let result = rt.block_on(async {
                    info!("spawning harness sidecar dsh={:?}", dsh_path);
                    sidecar::spawn_sidecar(&app_for_sidecar, dsh_path).await
                });

                match result {
                    Ok(sc) => {
                        let _ = tx.send(SidecarMessage::Ready(sc));
                    }
                    Err(e) => {
                        let _ = tx.send(SidecarMessage::Error(e.to_string()));
                    }
                }
            });

            // Wait for sidecar to be ready on the main thread
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
