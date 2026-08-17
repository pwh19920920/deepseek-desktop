use tauri_plugin_notification::NotificationExt;
use tauri::AppHandle;

/// Send a desktop notification.
#[tauri::command]
pub async fn send_notification(
    app: AppHandle,
    title: String,
    body: String,
    _priority: Option<i32>,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Check if notifications are allowed for this app.
#[tauri::command]
pub async fn is_notification_allowed(
    app: AppHandle,
) -> Result<bool, String> {
    use tauri_plugin_notification::NotificationExt;
    let state = app.notification().permission_state().map_err(|e| e.to_string())?;
    Ok(matches!(state, tauri::plugin::PermissionState::Granted))
}
