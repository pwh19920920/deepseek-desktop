/// Harness sidecar process management.
///
/// Spawns the `dsh web` command as a child process, discovers its listening
/// port from stdout, and provides a handle for lifecycle management.

use std::path::PathBuf;
use tokio::process::Command;
use tracing::info;

pub mod port_parser;

/// Handle to a running harness sidecar process.
pub struct SidecarHandle {
    pub port: u16,
    pub child: tokio::process::Child,
    /// Path to the `dsh` CLI binary.
    pub dsh_path: PathBuf,
}

impl SidecarHandle {
    /// The base URL the WebView should load.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

/// Spawn the harness sidecar and wait for it to report its listening port.
///
/// The harness is invoked as:
/// ```notest
/// node <dsh_path> web --port 0
/// ```
/// Port `0` tells the harness to let the OS assign a random free port.
pub async fn spawn_sidecar(
    node_path: PathBuf,
    dsh_path: PathBuf,
) -> anyhow::Result<SidecarHandle> {
    let mut cmd = Command::new(&node_path);
    cmd.args([
        dsh_path.to_string_lossy().as_ref(),
        "web",
        "--port",
        "0",
    ]);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    info!(
        "spawning sidecar: {:?} web --port 0",
        dsh_path
    );

    let mut child = cmd.spawn()?;
    let discovered_port = port_parser::discover_port(&mut child).await?;

    Ok(SidecarHandle {
        port: discovered_port,
        child,
        dsh_path,
    })
}
