use tauri::Manager;
use serde::Serialize;

use crate::SidecarState;

#[derive(Serialize, Clone)]
pub struct DshStatus {
    pub running: bool,
    pub port: Option<u16>,
}

/// Get the harness sidecar status.
#[tauri::command]
pub fn dsh_status(app: AppHandle) -> Result<DshStatus, String> {
    match app.try_state::<SidecarState>() {
        Some(state) => Ok(DshStatus {
            running: true,
            port: Some(state.port),
        }),
        None => Ok(DshStatus {
            running: false,
            port: None,
        }),
    }
}
