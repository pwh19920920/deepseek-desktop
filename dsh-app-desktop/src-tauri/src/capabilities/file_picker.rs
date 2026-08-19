use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, FilePath};

fn filepath_to_string(fp: FilePath) -> String {
    match fp {
        FilePath::Url(url) => url
            .to_file_path()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        FilePath::Path(path) => path.to_string_lossy().to_string(),
    }
}

#[tauri::command]
pub async fn pick_directory(
    app: AppHandle,
    prompt: Option<String>,
) -> Result<Option<String>, String> {
    let chosen: Option<FilePath> = app
        .dialog()
        .file()
        .set_title(prompt.as_deref().unwrap_or("Select a directory"))
        .blocking_pick_folder();
    Ok(chosen.map(filepath_to_string))
}

#[tauri::command]
pub async fn pick_file(app: AppHandle, prompt: Option<String>) -> Result<Option<String>, String> {
    let chosen: Option<FilePath> = app
        .dialog()
        .file()
        .set_title(prompt.as_deref().unwrap_or("Select a file"))
        .blocking_pick_file();
    Ok(chosen.map(filepath_to_string))
}

#[tauri::command]
pub async fn list_directory(path: String) -> Result<serde_json::Value, String> {
    use std::fs;
    let entries =
        fs::read_dir(&path).map_err(|e| format!("Failed to read directory {}: {}", path, e))?;
    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        items.push(serde_json::json!({"name": name, "is_dir": metadata.is_dir(), "path": entry.path().to_string_lossy().to_string()}));
    }
    Ok(serde_json::json!(items))
}
