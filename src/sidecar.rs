use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tracing::info;

pub mod port_parser;

/// Handle to a running harness sidecar process.
pub struct SidecarHandle {
    pub port: u16,
    /// Child process handle for lifecycle management.
    pub child: CommandChild,
    /// Path to the dsh entry point.
    pub dsh_path: PathBuf,
}

impl SidecarHandle {
    /// The base URL the WebView should load.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

/// Spawn the harness sidecar using Tauri's shell sidecar API.
///
/// This uses `app.shell().sidecar("node")` which automatically resolves the
/// externalBin binary configured in tauri.conf.json.
///
/// Runs: `<node> <dsh_path> web --port 0`
/// The harness prints a line like `dsh web: http://127.0.0.1:52631` on startup.
pub async fn spawn_sidecar(app: &AppHandle, dsh_path: PathBuf) -> anyhow::Result<SidecarHandle> {
    // Use Tauri's shell sidecar API — this resolves binaries/node-{target-triple}
    let cmd = app.shell().sidecar("node")?;
    let cmd = cmd.args([dsh_path.to_string_lossy().as_ref(), "web", "--port", "0"]);

    info!(
        "spawning harness sidecar via shell.sidecar(\"node\"): {:?}",
        dsh_path
    );

    let (mut cmd_events, child) = cmd.spawn()?;

    // Discover port from stdout events
    let discovered_port = discover_port_from_events(&mut cmd_events).await?;

    Ok(SidecarHandle {
        port: discovered_port,
        child,
        dsh_path,
    })
}

/// Read CommandEvents until we find a port number.
async fn discover_port_from_events(
    cmd_events: &mut tokio::sync::mpsc::Receiver<CommandEvent>,
) -> anyhow::Result<u16> {
    loop {
        match cmd_events.recv().await {
            Some(CommandEvent::Stdout(line)) => {
                if let Ok(line_str) = String::from_utf8(line) {
                    if let Some(port) = port_parser::extract_port(&line_str)? {
                        return Ok(port);
                    }
                }
            }
            Some(CommandEvent::Terminated(_)) => {
                return Err(anyhow::anyhow!("sidecar exited before port discovery"));
            }
            None => {
                return Err(anyhow::anyhow!("sidecar event stream closed"));
            }
            // Stderr and Error events are ignored for port discovery
            _ => continue,
        }
    }
}
