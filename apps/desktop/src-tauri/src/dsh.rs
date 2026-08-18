use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;
use tracing::info;

pub mod port;

/// Handle to a running harness sidecar process.
pub struct SidecarHandle {
    pub port: u16,
    pub child: CommandChild,
    pub dsh_path: PathBuf,
}

impl SidecarHandle {
    /// The base URL the WebView should load.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

/// Spawn the harness sidecar using Tauri's shell sidecar API.
pub async fn spawn_sidecar(app: &AppHandle, dsh_path: PathBuf) -> anyhow::Result<SidecarHandle> {
    let cmd = app.shell().sidecar("node")?;
    let cmd = cmd.args([dsh_path.to_string_lossy().as_ref(), "web", "--port", "0"]);

    info!(
        "spawning harness sidecar via shell.sidecar(\"node\"): {:?}",
        dsh_path
    );

    let (mut cmd_events, child) = cmd.spawn()?;
    let discovered_port = port::discover_port(&mut cmd_events).await?;

    Ok(SidecarHandle {
        port: discovered_port,
        child,
        dsh_path,
    })
}
