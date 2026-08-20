use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tracing::{info, warn};

use super::{port, SidecarHandle};

/// Clean up stale lock files from previous sessions that could prevent
/// the dsh sidecar from starting (e.g. plugin crash locks).
fn cleanup_stale_locks() {
    let dsh_home = std::env::var("DSH_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        format!("{}/.dsh", home)
    });
    let dsh_root = PathBuf::from(dsh_home);

    // Walk the dsh directory tree and remove stale lock/ledger files
    fn walk_remove(dir: &std::path::Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_remove(&path);
                } else if let Some(ext) = path.extension() {
                    if ext == "lock" || ext == "ledger" {
                        warn!("removing stale lock file: {:?}", path);
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }

    walk_remove(&dsh_root);
}

/// Spawn the harness sidecar using Tauri's shell sidecar API.
pub async fn spawn_sidecar(app: &AppHandle, dsh_path: PathBuf) -> anyhow::Result<SidecarHandle> {
    // Clean up stale locks from previous sessions before starting
    cleanup_stale_locks();

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

    // On Windows, strip the \\?\ extended-length path prefix that Tauri's
    // resource_dir() returns — Node.js cannot resolve paths with this prefix.
    let dsh_arg = if cfg!(target_os = "windows") {
        dsh_path.to_string_lossy().replacen("\\\\?\\", "", 1)
    } else {
        dsh_path.to_string_lossy().to_string()
    };

    let cmd = cmd.args([&dsh_arg, "web", "--port", "0"]);

    info!("spawning harness sidecar: {} --port 0", dsh_arg);

    let (mut cmd_events, child) = cmd.spawn()?;
    let discovered_port = port::discover_port(&mut cmd_events).await?;

    Ok(SidecarHandle {
        port: discovered_port,
        child,
        dsh_path,
    })
}
