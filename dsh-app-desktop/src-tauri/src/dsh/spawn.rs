use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tracing::info;

use super::{port, SidecarHandle};

/// Spawn the harness sidecar using Tauri's shell sidecar API.
pub async fn spawn_sidecar(app: &AppHandle, dsh_path: PathBuf) -> anyhow::Result<SidecarHandle> {
    // On Windows, Tauri's externalBin places node.exe at the app root, not in
    // binaries/.  We fall back to finding it via the current executable path.
    let cmd = if cfg!(target_os = "windows") {
        let exe_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("cannot find executable parent"))?
            .to_path_buf();
        let node_path = exe_dir.join("node.exe");
        info!("spawning node.exe directly at {:?}", node_path);
        app.shell().command(&*node_path.to_string_lossy())
    } else {
        app.shell().sidecar("node")?
    };

    let cmd = cmd.args([dsh_path.to_string_lossy().as_ref(), "web", "--port", "0"]);

    info!("spawning harness sidecar: {:?} {}", dsh_path, "--port 0");

    let (mut cmd_events, child) = cmd.spawn()?;
    let discovered_port = port::discover_port(&mut cmd_events).await?;

    Ok(SidecarHandle {
        port: discovered_port,
        child,
        dsh_path,
    })
}
