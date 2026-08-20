use std::path::{Path, PathBuf};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_shell::process::CommandChild;
use tokio::runtime::Runtime;
use tracing::{error, info};

pub mod capabilities;
pub mod commands;
pub mod dsh;
pub mod paths;

/// Profile patch template (mirrors dsh-app-boot's PROFILE_PATCH_TEMPLATE).
const PROFILE_PATCH_TEMPLATE: &str = r#"# Your patch layer for this dsh profile, applied after every bundle layer:
# a top-level YAML array of loader patch entries (id-targeted config
# overrides, disables, and insert lists; `!!js` expressions allowed).
[]
"#;

/// Profile pnpm workspace config (mirrors dsh-app-boot's PROFILE_PNPM_WORKSPACE).
const PROFILE_PNPM_WORKSPACE: &str = "packages:
  - .

nodeLinker: hoisted
autoInstallPeers: false
";

/// Ensure the web profile includes dshmarket in its dependencies and bundles.
/// Creates the profile directory on first run; upgrades existing profiles.
fn ensure_dshmarket_in_profile(app: &tauri::App) {
    let dsh_home = std::env::var("DSH_HOME").unwrap_or_else(|_| {
        app.path()
            .home_dir()
            .unwrap_or_default()
            .join(".dsh")
            .to_string_lossy()
            .to_string()
    });
    let profile_dir = PathBuf::from(dsh_home).join("profiles").join("web");
    let pkg_path = profile_dir.join("package.json");

    // Resolve the bundled dshmarket path (from app resources, or dev node_modules)
    let bundled_dshmarket = resolve_bundled_dshmarket(app);

    if !pkg_path.exists() {
        // First run: create the profile directory with dshmarket pre-configured.
        std::fs::create_dir_all(&profile_dir).unwrap_or_else(|e| {
            error!(
                "failed to create profile directory {:?}: {}",
                profile_dir, e
            );
        });
        let pkg = serde_json::json!({
            "name": "dsh-profile-web",
            "private": true,
            "dependencies": {
                "dshmarket": "*"
            },
            "dsh": {
                "profile": {
                    "bundles": [
                        "@deepseek-ai/dsh-base",
                        "@deepseek-ai/dsh-web-app",
                        "dshmarket"
                    ]
                }
            }
        });
        std::fs::write(&pkg_path, serde_json::to_string_pretty(&pkg).unwrap()).unwrap_or_else(
            |e| {
                error!("failed to write profile package.json: {}", e);
            },
        );
        std::fs::write(profile_dir.join("cordis.patch.yml"), PROFILE_PATCH_TEMPLATE).ok();
        std::fs::write(
            profile_dir.join("pnpm-workspace.yaml"),
            PROFILE_PNPM_WORKSPACE,
        )
        .ok();
        // Create symlink so the Cordis Loader can import dshmarket from the profile
        symlink_dshmarket(&profile_dir, &bundled_dshmarket);
        info!(
            "dshmarket pre-configured in new profile at {:?}",
            profile_dir
        );
        return;
    }

    // Upgrade existing profile: add dshmarket if missing.
    let content = match std::fs::read_to_string(&pkg_path) {
        Ok(c) => c,
        Err(e) => {
            error!("failed to read profile package.json: {}", e);
            return;
        }
    };
    let mut pkg: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            error!("failed to parse profile package.json: {}", e);
            return;
        }
    };

    // Already has dshmarket?
    if pkg
        .get("dependencies")
        .and_then(|d| d.get("dshmarket"))
        .is_some()
    {
        // Ensure symlink exists even if profile was already set up
        symlink_dshmarket(&profile_dir, &bundled_dshmarket);
        return;
    }

    // Add dshmarket to dependencies
    if let Some(deps) = pkg.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        deps.insert("dshmarket".into(), serde_json::json!("*"));
    } else {
        pkg["dependencies"] = serde_json::json!({"dshmarket": "*"});
    }

    // Add dshmarket to bundles
    if let Some(bundles) = pkg
        .pointer_mut("/dsh/profile/bundles")
        .and_then(|b| b.as_array_mut())
    {
        if !bundles.iter().any(|b| b == "dshmarket") {
            bundles.push(serde_json::json!("dshmarket"));
        }
    } else {
        let profile = pkg
            .pointer_mut("/dsh/profile")
            .and_then(|p| p.as_object_mut());
        if profile.is_none() {
            pkg["dsh"] = serde_json::json!({
                "profile": {
                    "bundles": ["dshmarket"]
                }
            });
        } else {
            pkg["dsh"]["profile"]["bundles"] = serde_json::json!(["dshmarket"]);
        }
    }

    std::fs::write(&pkg_path, serde_json::to_string_pretty(&pkg).unwrap()).unwrap_or_else(|e| {
        error!("failed to write updated profile package.json: {}", e);
    });
    // Create symlink so the Cordis Loader can import dshmarket from the profile
    symlink_dshmarket(&profile_dir, &bundled_dshmarket);
    info!("dshmarket added to existing profile at {:?}", profile_dir);
}

/// Resolve the path to the bundled dshmarket package, checking multiple locations.
fn resolve_bundled_dshmarket(app: &tauri::App) -> Option<PathBuf> {
    // 1. Bundled in app resources: {resource_dir}/resources/dsh/node_modules/dshmarket/
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled = resource_dir
            .join("resources")
            .join("dsh")
            .join("node_modules")
            .join("dshmarket");
        if bundled.join("package.json").exists() {
            return Some(bundled);
        }
        // 2. Tauri 2 _up_ layout: {resource_dir}/_up_/resources/dsh/node_modules/dshmarket/
        let bundled_up = resource_dir
            .join("_up_")
            .join("resources")
            .join("dsh")
            .join("node_modules")
            .join("dshmarket");
        if bundled_up.join("package.json").exists() {
            return Some(bundled_up);
        }
    }
    // 3. Development: in dsh-app-desktop node_modules
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("node_modules")
        .join("dshmarket");
    if dev.join("package.json").exists() {
        return Some(dev);
    }
    // 4. Root workspace node_modules
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../node_modules/dshmarket");
    if ws.join("package.json").exists() {
        return Some(ws);
    }
    None
}

/// Create a symlink from the profile's node_modules to the bundled dshmarket.
/// If the user has a real installation (not a symlink), leave it untouched —
/// this allows upgrading dshmarket independently of the app bundle.
fn symlink_dshmarket(profile_dir: &Path, bundled_dshmarket: &Option<PathBuf>) {
    let Some(src) = bundled_dshmarket else {
        error!("dshmarket not found in bundled resources, skipping symlink");
        return;
    };
    let link_path = profile_dir.join("node_modules").join("dshmarket");

    // If the path already exists, leave it alone — the user may have
    // upgraded or replaced it with a real installation.
    if link_path.exists() {
        if let Ok(meta) = std::fs::symlink_metadata(&link_path) {
            if meta.file_type().is_symlink() {
                let target = std::fs::read_link(&link_path).unwrap_or_default();
                let target_pkg = target.join("package.json");
                if target_pkg.exists() {
                    return; // existing symlink is valid
                }
                // broken symlink — remove and recreate
                let _ = std::fs::remove_file(&link_path);
            } else {
                // Real directory (not a symlink) — user upgraded it, leave alone
                info!("dshmarket is a real installation in profile, not overwriting");
                return;
            }
        } else {
            // Can't read metadata, leave alone
            return;
        }
    }

    std::fs::create_dir_all(profile_dir.join("node_modules")).ok();
    #[cfg(target_os = "windows")]
    {
        std::os::windows::fs::symlink_dir(&canonicalize_path(src), &link_path).ok();
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(canonicalize_path(src), &link_path).ok();
    }
    if link_path.exists() || link_path.is_symlink() {
        info!("symlinked bundled dshmarket into profile node_modules");
    } else {
        error!("failed to symlink dshmarket into profile node_modules");
    }
}

/// Canonicalize a path for symlink creation, handling relative paths.
fn canonicalize_path(path: &PathBuf) -> PathBuf {
    if path.is_absolute() {
        path.clone()
    } else {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.clone())
    }
}

/// Handle to the running harness sidecar, stored in Tauri state.
#[derive(Clone)]
pub struct SidecarState {
    pub port: u16,
    #[allow(dead_code)]
    child: std::sync::Arc<std::sync::Mutex<Option<CommandChild>>>,
    #[allow(dead_code)]
    dsh_path: PathBuf,
}

impl SidecarState {
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

/// Emit sidecar status events to the frontend.
fn emit_status(app: &tauri::AppHandle, status: &str, payload: &str) {
    let _ = app.emit(
        "dsh-status",
        serde_json::json!({
            "status": status,
            "message": payload,
        }),
    );
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Core commands
            commands::start::start_dsh,
            commands::stop::stop_dsh,
            commands::status::dsh_status,
            // Native capabilities
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

            let resource_dir = app.path().resource_dir().unwrap_or_default();
            let dsh_path = match paths::resolve_dsh_path(Some(resource_dir.clone())) {
                Ok(p) => p,
                Err(e) => {
                    error!("failed to resolve dsh path: {}", e);
                    emit_status(app.handle(), "error", &format!("dsh not found: {}", e));
                    return Ok(());
                }
            };
            info!("dsh resolved at {:?}", dsh_path);

            // Build system tray icon
            let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // 获取 Tauri 从配置自动创建的托盘图标，添加菜单和事件
            let tray = app
                .handle()
                .tray_by_id("main")
                .expect("tray icon should be created from config");
            tray.set_menu(Some(menu))?;
            tray.on_menu_event(|app, event| match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            });
            tray.on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });

            // Ensure the web profile has dshmarket configured before starting dsh.
            // This runs on every launch but is a no-op after the first setup.
            ensure_dshmarket_in_profile(app);

            let app_for_sidecar = app.handle().clone();
            std::thread::spawn(move || {
                let rt = Runtime::new().expect("failed to create tokio runtime");
                let result = rt.block_on(async {
                    info!("spawning harness sidecar dsh={:?}", dsh_path);
                    dsh::spawn_sidecar(&app_for_sidecar, dsh_path).await
                });

                match result {
                    Ok(sc) => {
                        let url_str = sc.url();
                        info!("harness sidecar ready at {}", url_str);
                        let state = SidecarState {
                            port: sc.port,
                            child: std::sync::Arc::new(std::sync::Mutex::new(Some(sc.child))),
                            dsh_path: sc.dsh_path,
                        };
                        app_for_sidecar.manage(state);

                        // Navigate the WebView to the sidecar URL
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.navigate(url_str.parse().expect("valid URL"));
                        }

                        // Notify frontend
                        emit_status(&app_for_sidecar, "ready", &url_str);
                    }
                    Err(e) => {
                        error!("failed to spawn harness sidecar: {}", e);
                        emit_status(&app_for_sidecar, "error", &e.to_string());
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Don't quit — just hide to tray, keep the sidecar running
                api.prevent_close();
                window.hide().unwrap_or_default();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building deepseek-desktop")
        .run(|_app_handle, _event| {
            // macOS: when the dock icon is clicked and there are no visible windows,
            // restore the hidden window (which was hidden-on-close to tray instead of quitting)
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen {
                has_visible_windows,
                ..
            } = _event
            {
                if !has_visible_windows {
                    if let Some(window) = _app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });
}
