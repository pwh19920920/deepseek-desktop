use tauri::AppHandle;

#[tauri::command]
pub async fn open_path(_app: AppHandle, path: String) -> Result<(), String> {
    let mut cmd = std::process::Command::new(open_cmd());
    cmd.arg(&path);
    cmd.output().map_err(|e| format!("Failed to open path: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn open_text_file(_app: AppHandle, path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("open");
        cmd.args(["-t", &path]);
        cmd.output().map_err(|e| format!("Failed to open text file: {}", e))?;
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("notepad");
        cmd.arg(&path);
        cmd.output().map_err(|e| format!("Failed to open text file: {}", e))?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(&path);
        cmd.output().map_err(|e| format!("Failed to open text file: {}", e))?;
        return Ok(());
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    Err("Unsupported platform".to_string())
}

#[tauri::command]
pub async fn open_url(_app: AppHandle, url: String) -> Result<(), String> {
    open_path(_app, url).await
}

fn open_cmd() -> &'static str {
    if cfg!(target_os = "macos") { "open" }
    else if cfg!(target_os = "windows") { "start" }
    else { "xdg-open" }
}
