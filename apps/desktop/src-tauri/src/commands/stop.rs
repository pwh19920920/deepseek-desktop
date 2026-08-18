use tauri::{Manager, AppHandle};
use tracing::error;

use crate::SidecarState;

/// Stop the harness sidecar.
#[tauri::command]
pub async fn stop_dsh(app: AppHandle) -> Result<(), String> {
    let state = app
        .state::<SidecarState>()
        .get()
        .map_err(|e| e.to_string())?;
    
    if let Some(child) = state.child.lock().unwrap().take() {
        if let Err(e) = child.kill() {
            error!("failed to kill sidecar: {}", e);
            return Err(e.to_string());
        }
    }
    
    Ok(())
}
