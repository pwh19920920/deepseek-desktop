use tauri_plugin_shell::process::CommandChild;

/// Shutdown the sidecar child process.
pub fn shutdown_child(child: CommandChild) -> anyhow::Result<()> {
    child.kill()?;
    Ok(())
}
