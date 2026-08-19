use std::path::PathBuf;
use tauri::{Emitter, Manager};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri_plugin_shell::process::CommandChild;
use tokio::runtime::Runtime;
use tracing::{error, info};

pub mod capabilities;
pub mod commands;
pub mod dsh;
pub mod paths;

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
    let _ = app.emit("dsh-status", serde_json::json!({
        "status": status,
        "message": payload,
    }));
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
            let show_item = MenuItemBuilder::with_id("show", "显示窗口")
                .build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出")
                .build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // 获取 Tauri 从配置自动创建的托盘图标，添加菜单和事件
            let tray = app.handle().tray_by_id("main")
                .expect("tray icon should be created from config");
            tray.set_menu(Some(menu))?;
            tray.on_menu_event(|app, event| {
                match event.id().as_ref() {
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
                }
            });
            tray.on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });

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
        .run(|app_handle, event| {
            // macOS: when the dock icon is clicked and there are no visible windows,
            // restore the hidden window (which was hidden-on-close to tray instead of quitting)
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { has_visible_windows, .. } = event {
                if !has_visible_windows {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });
}