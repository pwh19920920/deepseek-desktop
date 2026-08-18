use std::path::PathBuf;
use std::sync::mpsc;
use tauri::Manager;
use tauri_plugin_shell::process::CommandChild;
use tokio::runtime::Runtime;
use tracing::{error, info};

pub mod capabilities;
pub mod commands;
pub mod dsh;
pub mod error;
pub mod ipc;
pub mod paths;
