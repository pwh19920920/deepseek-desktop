use std::path::{Path, PathBuf};

/// Resolve the path to dsh's `lib/bin.js`.
pub fn resolve_dsh_path(app_resource_dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(ref resource_dir) = app_resource_dir {
        let bundled = resource_dir
            .join("resources")
            .join("dsh")
            .join("lib")
            .join("bin.js");
        if bundled.exists() {
            return Ok(bundled);
        }
    }
    let local = PathBuf::from("resources/dsh/lib/bin.js");
    if local.exists() {
        return Ok(local);
    }
    let node_modules = PathBuf::from("node_modules/@deepseek-ai/dsh/lib/bin.js");
    if node_modules.exists() {
        return Ok(node_modules);
    }
    anyhow::bail!(
        "dsh not found. Searched:\n  - bundled resource ({:?}/resources/dsh/lib/bin.js)\n  - resources/dsh/lib/bin.js\n  - node_modules/@deepseek-ai/dsh/lib/bin.js",
        app_resource_dir.as_ref().map(|d| d.display().to_string()).unwrap_or_default()
    )
}
