use tauri::AppHandle;
// use tracing::error;

/// Stop the harness sidecar.
#[tauri::command]
pub async fn stop_dsh(_app: AppHandle) -> Result<(), String> {
    // Sidecar shutdown is handled in lib.rs on window close
    Ok(())
}
