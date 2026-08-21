use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tracing::{info, warn};

use super::{port, SidecarHandle};

/// Clean up stale lock files from previous sessions.
/// We scan ~/.dsh subdirectories but DO NOT follow symlinks into node_modules,
/// which would trigger Windows Defender scans over 54MB/10k+ files.
fn cleanup_stale_locks() {
    let dsh_home = std::env::var("DSH_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        format!("{}/.dsh", home)
    });
    let dsh_root = std::path::PathBuf::from(&dsh_home);
    if !dsh_root.exists() {
        return;
    }

    // Known subdirectories that may contain lock files (profiles, task-board, storages, etc.)
    // We scan each one shallowly, but never follow symlinks.
    let subdirs = ["profiles", "task-board", "storages", "sessions"];

    for subdir in subdirs {
        let subdir_path = dsh_root.join(subdir);
        if !subdir_path.exists() {
            continue;
        }
        remove_locks_shallow(&subdir_path);
    }
}

/// Remove .lock/.ledger files in a directory, going one level deep.
/// Symlinks are skipped to avoid walking into node_modules.
fn remove_locks_shallow(dir: &std::path::Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Skip symlinks - they point to node_modules or other large trees
            if let Ok(meta) = std::fs::symlink_metadata(&path) {
                if meta.file_type().is_symlink() {
                    continue;
                }
            }
            if path.is_dir() {
                // Go one level deeper, still skipping symlinks
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if let Ok(meta) = std::fs::symlink_metadata(&sub_path) {
                            if meta.file_type().is_symlink() {
                                continue;
                            }
                        }
                        if let Some(ext) = sub_path.extension() {
                            if ext == "lock" || ext == "ledger" {
                                warn!("removing stale lock file: {:?}", sub_path);
                                let _ = std::fs::remove_file(&sub_path);
                            }
                        }
                    }
                }
            } else if let Some(ext) = path.extension() {
                if ext == "lock" || ext == "ledger" {
                    warn!("removing stale lock file: {:?}", path);
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
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
