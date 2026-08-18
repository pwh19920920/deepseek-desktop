use std::path::PathBuf;

pub mod port;
pub mod shutdown;
pub mod spawn;

/// Handle to a running harness sidecar process.
pub struct SidecarHandle {
    pub port: u16,
    pub child: tauri_plugin_shell::process::CommandChild,
    pub dsh_path: PathBuf,
}

impl SidecarHandle {
    /// The base URL the WebView should load.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

pub use shutdown::shutdown_child;
pub use spawn::spawn_sidecar;

#[cfg(test)]
mod tests {
    use super::port;

    #[test]
    fn test_parse_port_from_harness_format() {
        let line = "dsh web: http://127.0.0.1:52631\n";
        assert_eq!(port::extract_port(line).unwrap(), Some(52631));
    }

    #[test]
    fn test_parse_port_from_listening_line() {
        let line = "  dsh: server listening on 127.0.0.1:49152\n";
        assert_eq!(port::extract_port(line).unwrap(), Some(49152));
    }

    #[test]
    fn test_parse_port_from_different_format() {
        let line = "[webserver] bound to 127.0.0.1:8080\n";
        assert_eq!(port::extract_port(line).unwrap(), Some(8080));
    }

    #[test]
    fn test_parse_port_returns_none_for_non_matching_line() {
        let line = "some random log output\n";
        assert_eq!(port::extract_port(line).unwrap(), None);
    }

    #[test]
    fn test_parse_port_handles_empty_string() {
        assert_eq!(port::extract_port("").unwrap(), None);
    }
}
