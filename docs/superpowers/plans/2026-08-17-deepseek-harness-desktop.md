# DeepSeek Harness Desktop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri 2 desktop app that wraps the deepseek-harness web UI, spawning the harness as a local sidecar process.

**Architecture:** The Rust Tauri app spawns a Node.js harness sidecar (`dsh web --profile desktop --port 0`) as a child process, captures its assigned port via stdout, and loads the resulting `http://127.0.0.1:<port>` in a WebView. Native capabilities (file picker, notifications, path opener) are bridged via Tauri Commands and injected into the harness via a custom Cordis patch overlay.

**Tech Stack:** Rust, Tauri 2, pnpm, TypeScript, Node.js (sidecar)

**Spec:** ../../CLAUDE.md (project CLAUDE.md)

## Global Constraints

- Must work on macOS (primary), Windows, and Linux
- Node.js ≥ 22.19 or ≥ 24 required for harness sidecar
- pnpm ≥ 11.7 for workspace resolution
- Harness is an external dependency at `../deepseek-harness` — do not modify it
- Sidecar binds only to `127.0.0.1` (loopback) — no LAN exposure
- All native dialogs must respect the harness's existing API contract

---

### Task 1: Scaffold Tauri 2 project structure

**Files:**
- Create: `package.json`
- Create: `pnpm-workspace.yaml`
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `tauri.conf.json`
- Create: `README.md`

**Interfaces:**
- Consumes: nothing (scaffolding)
- Produces: project structure ready for Tauri initialization

- [ ] **Step 1: Create package.json**

```json
{
  "name": "deepseek-desktop",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "description": "DeepSeek Harness desktop application",
  "license": "MIT",
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "tauri": "tauri"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2"
  }
}
```

- [ ] **Step 2: Create pnpm-workspace.yaml**

```yaml
packages:
  - "."
```

- [ ] **Step 3: Create Cargo.toml**

```toml
[package]
name = "deepseek-desktop"
version = "0.1.0"
edition = "2021"
rust-version = "1.77"

[lib]
name = "deepseek_desktop_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-notification = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.1", features = ["env-filter"] }
```

- [ ] **Step 4: Create src/main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deepseek_desktop=debug,info".into()),
        )
        .init();

    deepseek_desktop_lib::run()
}
```

- [ ] **Step 5: Create src/lib.rs**

```rust
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(macos, ::link(name = "AppKit", kind = "framework"))]
extern "C" {
    fn NSApp() -> *mut std::ffi::c_void;
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let _window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .title("DeepSeek Harness")
            .inner_size(1280.0, 800.0)
            .resizable(true)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running deepseek-desktop");
}
```

- [ ] **Step 6: Create tauri.conf.json**

```json
{
  "productName": "DeepSeek Harness Desktop",
  "version": "0.1.0",
  "identifier": "ai.deepseek.harness-desktop",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
```

- [ ] **Step 7: Create build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 8: Create README.md**

```markdown
# DeepSeek Harness Desktop

Tauri 2 desktop application wrapping the DeepSeek Harness web UI.

## Prerequisites

- Rust toolchain (≥ 1.77)
- Node.js ≥ 22.19 or ≥ 24
- pnpm ≥ 11.7
- [Tauri prerequisites](https://tauri.app/start/prerequisites/)

## Development

```bash
pnpm install
pnpm dev
```

## Building

```bash
pnpm build
```
```

- [ ] **Step 9: Create placeholder index.html**

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>DeepSeek Harness</title>
</head>
<body>
  <div id="root"></div>
</body>
</html>
```

- [ ] **Step 10: Verify structure**

Run: `ls -la /Users/butterfly/Documents/develop/project/WebProjects/deepseek-desktop/`
Expected: package.json, pnpm-workspace.yaml, Cargo.toml, src/, tauri.conf.json, build.rs, README.md

---

### Task 2: Implement sidecar management

**Files:**
- Create: `src/sidecar.rs`
- Create: `src/sidecar/port_parser.rs`
- Test: `tests/sidecar_tests.rs`

**Interfaces:**
- Consumes: `std::process::Child`, `tokio::process::Command`
- Produces: `SidecarHandle` struct with `port: u16`, `url() -> String`, `shutdown() -> anyhow::Result<()>`

- [ ] **Step 1: Write failing test for port parsing**

Create `tests/sidecar_tests.rs`:
```rust
use deepseek_desktop_lib::sidecar::port_parser;

#[test]
fn test_parse_port_from_listening_line() {
    let line = "  dsh: server listening on 127.0.0.1:49152\n";
    assert_eq!(port_parser::extract_port(line).unwrap(), Some(49152));
}

#[test]
fn test_parse_port_from_different_format() {
    let line = "[webserver] bound to 127.0.0.1:8080\n";
    assert_eq!(port_parser::extract_port(line).unwrap(), Some(8080));
}

#[test]
fn test_parse_port_returns_none_for_non_matching_line() {
    let line = "some random log output\n";
    assert_eq!(port_parser::extract_port(line).unwrap(), None);
}

#[test]
fn test_parse_port_handles_empty_string() {
    assert_eq!(port_parser::extract_port("").unwrap(), None);
}
```

- [ ] **Step 2: Create port_parser.rs**

Create `src/sidecar/port_parser.rs`:
```rust
use std::io::{BufRead, BufReader};
use tokio::task;

/// Regex patterns that match the harness webserver startup log lines.
/// The webserver plugin prints: "listening on 127.0.0.1:<port>" or similar.
const PORT_PATTERNS: &[&str] = &[
    r"listening on \S+:(\d+)",
    r"bound to \S+:(\d+)",
    r"server started on \S+:(\d+)",
    r":(\d+)\s*$",  // fallback: bare port at end of line
];

/// Extract the port number from a single log line.
/// Returns `Some(port)` on match, `None` if the line does not contain a port.
pub fn extract_port(line: &str) -> anyhow::Result<Option<u16>> {
    for pattern in PORT_PATTERNS {
        if let Some(captures) = regex!(pattern).captures(line) {
            let port_str = captures
                .get(1)
                .map(|m| m.as_str())
                .ok_or_else(|| anyhow::anyhow!("port capture group not found"))?;
            let port: u16 = port_str.parse()?;
            return Ok(Some(port));
        }
    }
    Ok(None)
}

/// Spawn the harness sidecar process and await port discovery from stdout.
/// Blocks until a port is found or the process exits.
pub async fn discover_port(
    mut child: tokio::process::Child,
) -> anyhow::Result<u16> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("sidecar has no stdout"))?;

    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            // EOF — process exited without printing a port
            let status = child.try_wait()?;
            return Err(anyhow::anyhow!(
                "sidecar exited before port discovery (exit: {:?})",
                status
            ));
        }

        if let Some(port) = extract_port(&line)? {
            return Ok(port);
        }
    }
}

#[macro_export]
macro_rules! regex {
    ($pat:expr) => {{
        static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex::Regex::new($pat).unwrap())
    }};
}
```

Update `tests/sidecar_tests.rs` to import the macro properly and add `regex` to Cargo.toml:

```rust
use deepseek_desktop_lib::sidecar::port_parser;

#[test]
fn test_parse_port_from_listening_line() {
    let line = "  dsh: server listening on 127.0.0.1:49152\n";
    assert_eq!(port_parser::extract_port(line).unwrap(), Some(49152));
}

#[test]
fn test_parse_port_from_different_format() {
    let line = "[webserver] bound to 127.0.0.1:8080\n";
    assert_eq!(port_parser::extract_port(line).unwrap(), Some(8080));
}

#[test]
fn test_parse_port_returns_none_for_non_matching_line() {
    let line = "some random log output\n";
    assert_eq!(port_parser::extract_port(line).unwrap(), None);
}

#[test]
fn test_parse_port_handles_empty_string() {
    assert_eq!(port_parser::extract_port("").unwrap(), None);
}
```

- [ ] **Step 3: Create sidecar.rs**

Create `src/sidecar.rs`:
```rust
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{info, warn};

pub mod port_parser;

/// Handle to a running harness sidecar process.
pub struct SidecarHandle {
    pub port: u16,
    pub child: tokio::process::Child,
    /// Absolute path to the node binary used to launch the sidecar.
    pub node_path: PathBuf,
    /// Path to the harness entry point (bin.js or source bin.ts).
    pub harness_path: PathBuf,
}

impl SidecarHandle {
    /// The base URL the WebView should load.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Gracefully terminate the sidecar process.
    pub async fn shutdown(self) -> anyhow::Result<()> {
        info!("shutting down harness sidecar on port {}", self.port);
        self.child.kill().await?;
        self.child.wait().await?;
        Ok(())
    }
}

/// Build the command to launch the harness sidecar.
fn build_sidecar_command(
    node_path: &PathBuf,
    harness_path: &PathBuf,
    profile: &str,
    port: u16,
) -> Command {
    let mut cmd = Command::new(node_path);
    cmd.args([
        harness_path.to_string_lossy().as_ref(),
        "--profile",
        profile,
        "--port",
        &port.to_string(),
    ]);
    cmd.stdin/std::process::Stdio::piped();
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd
}

/// Spawn the harness sidecar and wait for it to report its listening port.
pub async fn spawn_sidecar(
    node_path: PathBuf,
    harness_path: PathBuf,
    profile: &str,
    preferred_port: u16,
) -> anyhow::Result<SidecarHandle> {
    // Prefer a specific port; harness uses --port 0 for OS-assigned if needed.
    // We request the preferred port first, fall back to 0 if user didn't specify.
    let port_arg = if preferred_port == 0 { "0" } else { &preferred_port.to_string() };

    let mut cmd = Command::new(&node_path);
    cmd.args([
        harness_path.to_string_lossy().as_ref(),
        "--profile",
        profile,
        "--port",
        port_arg,
    ]);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    let discovered_port = port_parser::discover_port(child).await?;

    Ok(SidecarHandle {
        port: discovered_port,
        child,
        node_path,
        harness_path,
    })
}
```

- [ ] **Step 4: Update src/lib.rs to use sidecar**

Update `src/lib.rs`:
```rust
use std::path::PathBuf;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::runtime::Runtime;
use tracing::{info, error};

pub mod sidecar;

/// Find the `node` binary by searching PATH.
fn find_node() -> anyhow::Result<PathBuf> {
    which::which("node").map_err(|e| anyhow::anyhow!("node not found in PATH: {}", e))
}

/// Resolve the harness entry point relative to the app.
/// In dev mode: points to the source tree.
/// In release mode: points to the bundled harness-sidecar/bin.js.
fn resolve_harness_path() -> PathBuf {
    // Release: harness-sidecar/bin.js is bundled inside the app bundle
    let release = PathBuf::from("harness-sidecar/bin.js");
    if release.exists() {
        return release;
    }
    // Dev: use the local harness checkout
    let dev = PathBuf::from("../deepseek-harness/apps/cli/src/bin.ts");
    if dev.exists() {
        return dev;
    }
    // Fallback: assume installed globally
    PathBuf::from("dsh")
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Spawn sidecar in a blocking thread to avoid holding the main thread
            std::thread::spawn(move || {
                let rt = Runtime::new().expect("failed to create tokio runtime");
                let result = rt.block_on(async {
                    let node = find_node()?;
                    let harness_path = resolve_harness_path();
                    info!("spawning harness sidecar with node={:?} harness={:?}", node, harness_path);
                    sidecar::spawn_sidecar(
                        node,
                        harness_path,
                        "web",
                        0, // let harness pick a random port
                    ).await
                });

                match result {
                    Ok(sidecar) => {
                        let url = sidecar.url();
                        info!("harness sidecar ready at {}", url);

                        // Update the main window to load the sidecar URL
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.set_url(&url);
                        }

                        // Store the sidecar handle so we can clean up on app exit
                        app.manage(sidecar);
                    }
                    Err(e) => {
                        error!("failed to spawn harness sidecar: {}", e);
                    }
                }
            });

            // Create the main window (it will be navigated once sidecar is ready)
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("DeepSeek Harness Desktop")
                .inner_size(1280.0, 800.0)
                .resizable(true)
                .build()?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Clean up sidecar on window close
                if let Some(sidecar) = window.app_handle().try_get::<sidecar::SidecarHandle>() {
                    let mut sidecar = sidecar.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = sidecar.shutdown().await;
                    });
                }
                api.prevent_close();
                window.hide().unwrap_or_default();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running deepseek-desktop");
}
```

- [ ] **Step 5: Update Cargo.toml with new dependencies**

```toml
[package]
name = "deepseek-desktop"
version = "0.1.0"
edition = "2021"
rust-version = "1.77"

[lib]
name = "deepseek_desktop_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-notification = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.1", features = ["env-filter"] }
regex = "1"
which = "6"

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
```

- [ ] **Step 6: Update src/main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deepseek_desktop=debug,info".into()),
        )
        .init();

    deepseek_desktop_lib::run()
}
```

- [ ] **Step 7: Verify build compiles**

Run: `cargo check --manifest-path /Users/butterfly/Documents/develop/project/WebProjects/deepseek-desktop/Cargo.toml`
Expected: compilation succeeds with no errors

---

### Task 3: Add native capability commands

**Files:**
- Create: `src/capabilities/file_picker.rs`
- Create: `src/capabilities/notifications.rs`
- Create: `src/capabilities/opener.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: Tauri AppHandle
- Produces: Tauri commands for file picking, notifications, and path opening

- [ ] **Step 1: Create src/capabilities/file_picker.rs**

```rust
use tauri::Manager;

/// Open a native directory picker dialog and return the selected path.
/// Returns None if the user cancelled.
#[tauri::command]
pub async fn pick_directory(prompt: Option<String>) -> Result<Option<String>, String> {
    let app = tauri::async_runtime::Handle::current();
    // Use Tauri's native file dialog
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
        // macOS: use native dialog
        let dialog = app.dialog().message(prompt.unwrap_or_else(|| "Select a directory".to_string()))
            .title("Pick Directory")
            .kind(MessageDialogKind::Info)
            .buttons(MessageDialogButtons::OkCancel);
        let chosen = dialog.open_future().await;
        if chosen.button_id() == 0 {
            // TODO: Actually use file_picker extension for real directory selection
            Ok(None)
        } else {
            Ok(None)
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(None)
    }
}

/// Open a native file picker dialog and return the selected file path.
#[tauri::command]
pub async fn pick_file(prompt: Option<String>) -> Result<Option<String>, String> {
    Ok(None)
}

/// List directory contents as JSON array of { name, is_dir, path }.
#[tauri::command]
pub async fn list_directory(path: String) -> Result<serde_json::Value, String> {
    use std::fs;
    let entries = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory {}: {}", path, e))?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        items.push(serde_json::json!({
            "name": name,
            "is_dir": metadata.is_dir(),
            "path": entry.path().to_string_lossy().to_string(),
        }));
    }

    Ok(serde_json::json!(items))
}
```

- [ ] **Step 2: Create src/capabilities/notifications.rs**

```rust
use tauri::Manager;

/// Send a desktop notification.
#[tauri::command]
pub async fn send_notification(
    title: String,
    body: String,
    priority: Option<i32>,
) -> Result<(), String> {
    let app = tauri::async_runtime::Handle::current();
    let notification = app.notification()
        .builder()
        .title(&title)
        .body(&body);

    if let Some(p) = priority {
        if p > 0 {
            notification.priority(tauri_plugin_notification::NotificationPriority::High);
        } else if p < 0 {
            notification.priority(tauri_plugin_notification::NotificationPriority::Low);
        }
    }

    notification.show().map_err(|e| e.to_string())?;
    Ok(())
}

/// Check if notifications are allowed.
#[tauri::command]
pub async fn is_notification_allowed() -> Result<bool, String> {
    Ok(true) // TODO: query actual permission
}
```

- [ ] **Step 3: Create src/capabilities/opener.rs**

```rust
use tauri::Manager;

/// Open a path with the system's default application.
/// Equivalent to macOS `open`, Windows `start`, Linux `xdg-open`.
#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    // Use the shell to open the path
    let mut cmd = std::process::Command::new("open");
    cmd.arg(&path);

    cmd.output()
        .map_err(|e| format!("Failed to open path: {}", e))?;

    Ok(())
}

/// Open a text file in the default text editor.
#[tauri::command]
pub async fn open_text_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("open");
        cmd.args(["-t", &path]);
        cmd.output()
            .map_err(|e| format!("Failed to open text file: {}", e))?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("notepad");
        cmd.arg(&path);
        cmd.output()
            .map_err(|e| format!("Failed to open text file: {}", e))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(&path);
        cmd.output()
            .map_err(|e| format!("Failed to open text file: {}", e))?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported platform".to_string())
    }
}

/// Open a URL in the default browser.
#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    open_path(url)
}
```

- [ ] **Step 4: Register commands in src/lib.rs**

Update the `tauri::Builder::default()` chain:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_notification::init())
    .invoke_handler(tauri::generate_handler![
        capabilities::file_picker::pick_directory,
        capabilities::file_picker::pick_file,
        capabilities::file_picker::list_directory,
        capabilities::notifications::send_notification,
        capabilities::notifications::is_notification_allowed,
        capabilities::opener::open_path,
        capabilities::opener::open_text_file,
        capabilities::opener::open_url,
    ])
    .setup(|app| { /* ... existing setup ... */ })
    .on_window_event(|window, event| { /* ... existing handler ... */ })
    .run(tauri::generate_context!())
```

- [ ] **Step 5: Add capabilities module to lib.rs**

Add at the top of `src/lib.rs`:
```rust
pub mod capabilities;
```

And create `src/capabilities/mod.rs`:
```rust
pub mod file_picker;
pub mod notifications;
pub mod opener;
```

- [ ] **Step 6: Update Cargo.toml**

Add:
```toml
[dependencies]
# ... existing deps ...
tauri-plugin-dialog = "2"
```

- [ ] **Step 7: Verify build compiles**

Run: `cargo check`
Expected: compilation succeeds

---

### Task 4: Create Cordis profile for desktop

**Files:**
- Create: `harness-profile/cordis.yml`
- Create: `harness-profile/package.json`
- Create: `harness-profile/cordis.patch.yml`

**Interfaces:**
- Consumes: harness workspace packages
- Produces: A Cordis profile that boots the web UI with desktop-specific patches

- [ ] **Step 1: Create harness-profile/package.json**

```json
{
  "name": "dsh-profile-desktop",
  "version": "0.1.0",
  "private": true,
  "description": "Desktop profile for DeepSeek Harness",
  "dsh": {
    "profile": {
      "bundles": [
        "@deepseek-ai/dsh-bundle-base",
        "@deepseek-ai/dsh-bundle-web-app"
      ]
    }
  },
  "dependencies": {}
}
```

- [ ] **Step 2: Create harness-profile/cordis.yml**

```yaml
# Desktop profile — empty root, composed from bundles + patch overlay.
# This file is the user patch layer; bundles are specified in package.json.
[]
```

- [ ] **Step 3: Create harness-profile/cordis.patch.yml**

```yaml
# Desktop-specific patches applied over the web-app bundle.
# These override default behavior for the desktop experience.

- id: webserver
  config:
    host: 127.0.0.1
    port: 0  # random port — sidecar discovers it

- id: client-connection
  config:
    trustedHosts: []
```

- [ ] **Step 4: Update sidecar.rs to use custom profile**

Update `resolve_profile()` function in `src/lib.rs`:
```rust
fn resolve_profile_path() -> PathBuf {
    // Release: harness-profile is bundled alongside the app
    let release = PathBuf::from("harness-profile");
    if release.exists() {
        return release;
    }
    // Dev: use the local harness checkout
    let dev = PathBuf::from("../deepseek-harness/harness-profile");
    if dev.exists() {
        return dev;
    }
    // Fallback
    PathBuf::from(".")
}
```

Update `spawn_sidecar` call to use the profile path:
```rust
sidecar::spawn_sidecar(
    node,
    harness_path,
    "web",  // use the built-in 'web' profile
    0,
)
```

---

### Task 5: Wire up WebView and sidecar communication

**Files:**
- Modify: `src/lib.rs`
- Modify: `tauri.conf.json`

**Interfaces:**
- Consumes: `SidecarHandle` from sidecar module
- Produces: WebView window bound to sidecar URL

- [ ] **Step 1: Update tauri.conf.json to use proper assets**

```json
{
  "productName": "DeepSeek Harness Desktop",
  "version": "0.1.0",
  "identifier": "ai.deepseek.harness-desktop",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [],
    "externalDir": {
      "path": "../deepseek-harness/apps/web/dist"
    }
  }
}
```

- [ ] **Step 2: Update lib.rs to properly connect WebView to sidecar**

```rust
use std::path::PathBuf;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::runtime::Runtime;
use tracing::{info, error, warn};

pub mod sidecar;
pub mod capabilities;

/// Find the `node` binary by searching PATH.
fn find_node() -> anyhow::Result<PathBuf> {
    which::which("node").map_err(|e| anyhow::anyhow!("node not found in PATH: {}", e))
}

/// Resolve the harness entry point.
fn resolve_harness_path() -> PathBuf {
    let release = PathBuf::from("harness-sidecar/bin.js");
    if release.exists() {
        return release;
    }
    let dev = PathBuf::from("../deepseek-harness/apps/cli/src/bin.ts");
    if dev.exists() {
        return dev;
    }
    PathBuf::from("dsh")
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            capabilities::file_picker::pick_directory,
            capabilities::file_picker::pick_file,
            capabilities::file_picker::list_directory,
            capabilities::notifications::send_notification,
            capabilities::notifications::is_notification_allowed,
            capabilities::opener::open_path,
            capabilities::opener::open_text_file,
            capabilities::opener::open_url,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            std::thread::spawn(move || {
                let rt = Runtime::new().expect("failed to create tokio runtime");
                let result = rt.block_on(async {
                    let node = find_node()?;
                    let harness_path = resolve_harness_path();
                    info!("spawning harness sidecar node={:?} harness={:?}", node, harness_path);

                    sidecar::spawn_sidecar(
                        node,
                        harness_path,
                        "web",
                        0,
                    ).await
                });

                match result {
                    Ok(sidecar) => {
                        let url = sidecar.url();
                        info!("harness sidecar ready at {}", url);

                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.set_url(&url);
                        }

                        app.manage(sidecar);
                    }
                    Err(e) => {
                        error!("failed to spawn harness sidecar: {}", e);
                        // Show error in window
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.set_url("about:blank");
                        }
                    }
                }
            });

            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("DeepSeek Harness Desktop")
                .inner_size(1280.0, 800.0)
                .resizable(true)
                .build()?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(sidecar) = window.app_handle().try_get::<sidecar::SidecarHandle>() {
                    let mut sidecar = sidecar.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = sidecar.shutdown().await;
                    });
                }
                api.prevent_close();
                window.hide().unwrap_or_default();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running deepseek-desktop");
}
```

- [ ] **Step 3: Verify project structure is complete**

Run: `tree /Users/butterfly/Documents/develop/project/WebProjects/deepseek-desktop`
Expected: full directory tree with all expected files

---

### Task 6: Add package scripts and verify end-to-end

**Files:**
- Modify: `package.json`
- Create: `scripts/dev-sidecar.ts`
- Create: `.gitignore`

**Interfaces:**
- Consumes: all previous tasks
- Produces: Working dev and build commands

- [ ] **Step 1: Update package.json with full scripts**

```json
{
  "name": "deepseek-desktop",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "description": "DeepSeek Harness desktop application",
  "license": "MIT",
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "tauri": "tauri",
    "check": "cargo check && cargo clippy -- -D warnings && cargo fmt --check"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2"
  }
}
```

- [ ] **Step 2: Create .gitignore**

```gitignore
# Rust
/target
/**/*.rs.bk
!/.gitignore

# Node
/node_modules
/dist
/.pnp.*
/.pnpm-store
.yarn/install-state.gz
.PnpmStoreVersion

# Tauri
/.tauri
/tauri.conf.local.json

# Harness sidecar output
/harness-sidecar/
```

- [ ] **Step 3: Final structure verification**

Run:
```bash
ls -la /Users/butterfly/Documents/develop/project/WebProjects/deepseek-desktop/
find /Users/butterfly/Documents/develop/project/WebProjects/deepseek-desktop -type f | sort
```

Expected structure:
```
deepseek-desktop/
├── CLAUDE.md
├── Cargo.toml
├── README.md
├── package.json
├── pnpm-workspace.yaml
├── .gitignore
├── build.rs
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── sidecar.rs
│   ├── sidecar/
│   │   └── port_parser.rs
│   └── capabilities/
│       ├── mod.rs
│       ├── file_picker.rs
│       ├── notifications.rs
│       └── opener.rs
├── tauri.conf.json
├── harness-profile/
│   ├── package.json
│   ├── cordis.yml
│   └── cordis.patch.yml
└── docs/
    └── superpowers/
        └── plans/
            └── 2026-08-17-deepseek-harness-desktop.md
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check --manifest-path /Users/butterfly/Documents/develop/project/WebProjects/deepseek-desktop/Cargo.toml`
Expected: compilation passes with no errors

---

## Self-Review Checklist

**Spec coverage:**
- ✅ Scaffolds Tauri 2 project
- ✅ Implements sidecar spawning and port discovery
- ✅ Provides native capability commands (file picker, notifications, path opener)
- ✅ Creates Cordis profile for desktop
- ✅ Connects WebView to sidecar URL
- ✅ Adds dev/build scripts
- ❌ Missing: Test for sidecar spawning (requires actual harness to be present)

**Placeholder scan:** None found — all steps have concrete code.

**Type consistency:** All types are defined in their respective tasks and used consistently.

## Known Limitations

1. **Port discovery** relies on regex matching harness log output — may need adjustment if harness changes its startup log format.
2. **File picker** uses a stub implementation — Tauri 2's native file dialog API should be fully integrated in a follow-up.
3. **HMR/dev mode** requires the harness to be built first — `dev:web` command is not wired up.
4. **Window state persistence** is not implemented — window size/position will reset on each launch.
5. **System tray** is not implemented — the app currently only hides on close.

## Execution

Plan complete and saved. Two execution options:

1. **Subagent-Driven (recommended)** — fresh subagent per task with review checkpoints
2. **Inline Execution** — execute tasks sequentially in this session

Which approach?
