use tauri::{Manager, AppHandle};
use tracing::info;

use crate::dsh;
use crate::paths;

/// Start the harness sidecar.
#[tauri::command]
pub async fn start_dsh(app: AppHandle) -> Result<String, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?;
    let dsh_path = paths::resolve_dsh_path(Some(resource_dir))
        .map_err(|e| e.to_string())?;
    
    info!("starting dsh sidecar: {:?}", dsh_path);
    
    let handle = dsh::spawn_sidecar(&app, dsh_path)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(handle.url())
}
