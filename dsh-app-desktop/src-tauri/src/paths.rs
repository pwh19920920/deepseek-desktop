use std::path::PathBuf;

/// Resolve the path to dsh's `lib/bin.js`.
///
/// Search order:
/// 1. `resource_dir/resources/dsh/lib/bin.js` (Tauri's traditional resource layout)
/// 2. `resource_dir/_up_/resources/dsh/lib/bin.js` (Tauri 2's `_up_` layout)
/// 3. `../resources/dsh/lib/bin.js` (relative to src-tauri/, development)
/// 4. `../node_modules/@deepseek-ai/dsh/lib/bin.js` (development fallback)
pub fn resolve_dsh_path(app_resource_dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(ref resource_dir) = app_resource_dir {
        // Traditional layout: resource_dir/resources/dsh/lib/bin.js
        let bundled = resource_dir
            .join("resources")
            .join("dsh")
            .join("lib")
            .join("bin.js");
        if bundled.exists() {
            return Ok(bundled);
        }
        // Tauri 2 `_up_` layout: resource_dir/_up_/resources/dsh/lib/bin.js
        let bundled_up = resource_dir
            .join("_up_")
            .join("resources")
            .join("dsh")
            .join("lib")
            .join("bin.js");
        if bundled_up.exists() {
            return Ok(bundled_up);
        }
    }
    let local = PathBuf::from("../resources/dsh/lib/bin.js");
    if local.exists() {
        return Ok(local);
    }
    let node_modules = PathBuf::from("../node_modules/@deepseek-ai/dsh/lib/bin.js");
    if node_modules.exists() {
        return Ok(node_modules);
    }
    anyhow::bail!(
        "dsh not found. Searched:\n  - bundled resource ({:?}/resources/dsh/lib/bin.js)\n  - bundled resource ({:?}/_up_/resources/dsh/lib/bin.js)\n  - ../resources/dsh/lib/bin.js\n  - ../node_modules/@deepseek-ai/dsh/lib/bin.js",
        app_resource_dir.as_ref().map(|d| d.display().to_string()).unwrap_or_default(),
        app_resource_dir.as_ref().map(|d| d.display().to_string()).unwrap_or_default(),
    )
}
