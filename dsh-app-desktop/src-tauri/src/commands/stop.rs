use tauri::AppHandle;
// use tracing::error;

/// Stop the harness sidecar.
#[tauri::command]
#[allow(unused_variables)]
pub async fn stop_dsh(_app: AppHandle) -> Result<(), String> {
    // Sidecar shutdown is handled on app exit; the close button only hides the window
    Ok(())
}
