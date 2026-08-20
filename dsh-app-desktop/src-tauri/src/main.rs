#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

/// Get the dsh home directory (from DSH_HOME env var, or ~/.dsh).
fn dsh_home() -> PathBuf {
    if let Ok(home) = std::env::var("DSH_HOME") {
        return PathBuf::from(home);
    }
    let base = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join(".dsh")
}

/// Get today's date as "YYYY-MM-DD".
fn today_date_string() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;

    let mut y = 1970i64;
    let mut d = days as i64;

    loop {
        let yd = if is_leap(y) { 366 } else { 365 };
        if d < yd {
            break;
        }
        d -= yd;
        y += 1;
    }

    let leap = is_leap(y);
    let months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 1usize;
    for &md in &months {
        if d < md {
            break;
        }
        d -= md;
        m += 1;
    }

    format!("{:04}-{:02}-{:02}", y, m, d + 1)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

/// Initialize tracing to write to a daily log file under `~/.dsh/logs/`.
fn setup_logging() {
    let log_dir = dsh_home().join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!(
            "[dsh-desktop] failed to create log dir {:?}: {}",
            log_dir, e
        );
        tracing_subscriber::fmt::init();
        return;
    }

    let date_str = today_date_string();
    let log_path = log_dir.join(format!("{}.log", date_str));

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(file) => {
            // Write a session start marker directly
            let _ = std::io::Write::write_all(&mut &file, b"\n--- dsh-desktop session start ---\n");
            // Drop the file handle; the closure will reopen it on each event
            drop(file);

            let log_path_inner = log_path.clone();
            let make_writer = move || {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path_inner)
                    .expect("failed to open log file for tracing")
            };

            let subscriber = tracing_subscriber::FmtSubscriber::builder()
                .with_writer(make_writer)
                .with_ansi(false)
                .finish();
            let _ = tracing::subscriber::set_global_default(subscriber);
        }
        Err(e) => {
            eprintln!(
                "[dsh-desktop] failed to open log file {:?}: {}",
                log_path, e
            );
            tracing_subscriber::fmt::init();
        }
    }
}

fn main() {
    setup_logging();
    tracing::info!("DeepSeek Harness Desktop starting...");
    deepseek_desktop_lib::run()
}
