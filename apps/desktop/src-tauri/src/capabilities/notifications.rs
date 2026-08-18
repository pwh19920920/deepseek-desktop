use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub async fn send_notification(app: AppHandle, title: String, body: String, _priority: Option<i32>) -> Result<(), String> {
    app.notification().builder().title(&title).body(&body).show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn is_notification_allowed(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_notification::NotificationExt;
    let state = app.notification().permission_state().map_err(|e| e.to_string())?;
    Ok(matches!(state, tauri::plugin::PermissionState::Granted))
}
