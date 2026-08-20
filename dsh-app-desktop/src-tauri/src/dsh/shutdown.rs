use tauri_plugin_shell::process::CommandChild;
use tracing::info;

/// Shutdown the sidecar child process.
pub fn shutdown_child(child: CommandChild) -> anyhow::Result<()> {
    info!("shutting down harness sidecar...");
    child.kill()?;
    info!("harness sidecar terminated");
    Ok(())
}
