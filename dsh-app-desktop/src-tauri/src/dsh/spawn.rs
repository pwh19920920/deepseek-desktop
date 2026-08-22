use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
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

/// Find an available port on localhost.
fn find_available_port() -> Option<u16> {
    // Try ports in the range 52000-53000
    for port in 52000..53000 {
        if std::net::TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
            return Some(port);
        }
    }
    None
}

/// Spawn the harness sidecar using Tauri's shell sidecar API.
pub async fn spawn_sidecar(app: &AppHandle, dsh_path: PathBuf) -> anyhow::Result<SidecarHandle> {
    // Clean up stale locks from previous sessions before starting
    cleanup_stale_locks();

    // Find a fixed port so restart can reuse the same port
    let port = find_available_port().ok_or_else(|| anyhow::anyhow!("no available port found"))?;
    info!("using fixed port {}", port);

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

    let cmd = cmd.args([&dsh_arg, "web", "--port", &port.to_string()]);

    info!("spawning harness sidecar: {} --port {}", dsh_arg, port);

    let (mut cmd_events, child) = cmd.spawn()?;
    let discovered_port = port::discover_port(&mut cmd_events).await?;

    // Verify the port matches what we requested
    if discovered_port != port {
        warn!(
            "sidecar bound to port {} instead of requested {}",
            discovered_port, port
        );
    }

    Ok(SidecarHandle {
        port: discovered_port,
        child,
        dsh_path,
    })
}

/// Watch for sidecar restart and handle WebView reload.
/// When dsh-market triggers a restart, the sidecar process exits and a new one
/// takes over. This function detects that and reloads the WebView.
pub fn watch_for_restart(
    app: AppHandle,
    port: u16,
    _child: Arc<std::sync::Mutex<Option<tauri_plugin_shell::process::CommandChild>>>,
) {
    std::thread::spawn(move || {
        info!("watching for sidecar restart on port {}", port);

        // Poll to check if the sidecar is still serving
        let check_interval = std::time::Duration::from_secs(1);
        let mut was_serving = true;

        loop {
            let is_serving = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok();

            if was_serving && !is_serving {
                // Sidecar stopped serving - it might be restarting
                info!(
                    "sidecar stopped serving on port {}, watching for restart...",
                    port
                );

                // dsh-market's restart helper waits up to 30s for port release + 20s for new process
                let restart_timeout = std::time::Duration::from_secs(60);
                let start = std::time::Instant::now();

                while start.elapsed() < restart_timeout {
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
                        info!(
                            "restart detected - new sidecar is listening on port {}",
                            port
                        );

                        // Notify frontend about restart
                        let _ = app.emit(
                            "dsh-status",
                            serde_json::json!({
                                "status": "restarting",
                                "message": "正在重启，请稍候…",
                                "port": port,
                            }),
                        );

                        // Wait a moment for the new process to fully initialize
                        std::thread::sleep(std::time::Duration::from_secs(1));

                        // Reload the WebView
                        if let Some(window) = app.get_webview_window("main") {
                            let url = format!("http://127.0.0.1:{}", port);
                            info!("reloading WebView to {}", url);
                            let _ = window.navigate(url.parse().expect("valid URL"));
                        }

                        // Notify frontend that restart is complete
                        let _ = app.emit(
                            "dsh-status",
                            serde_json::json!({
                                "status": "ready",
                                "message": format!("http://127.0.0.1:{}", port),
                            }),
                        );

                        was_serving = true;
                        break;
                    }
                }

                // If no restart detected within timeout, report error
                if !was_serving {
                    warn!("sidecar exited without restart detected");
                    let _ = app.emit(
                        "dsh-status",
                        serde_json::json!({
                            "status": "error",
                            "message": "Sidecar 进程意外退出",
                        }),
                    );
                    return; // Exit the thread
                }
            }

            std::thread::sleep(check_interval);
        }
    });
}
