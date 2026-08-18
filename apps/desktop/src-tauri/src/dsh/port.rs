use tauri_plugin_shell::process::CommandEvent;
use tokio::sync::mpsc::Receiver;

const PORT_PATTERNS: &[&str] = &[
    r"listening on \S+:(\d+)",
    r"bound to \S+:(\d+)",
    r"server started on \S+:(\d+)",
    r"127\.0\.0\.1:(\d+)",
    r":(\d+)\s*$",
];

pub fn extract_port(line: &str) -> anyhow::Result<Option<u16>> {
    for pattern in PORT_PATTERNS {
        let re = regex::Regex::new(pattern)?;
        if let Some(captures) = re.captures(line) {
            let port_str = captures
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("port capture group not found"))?
                .as_str();
            let port: u16 = port_str.parse()?;
            return Ok(Some(port));
        }
    }
    Ok(None)
}

pub async fn discover_port(cmd_events: &mut Receiver<CommandEvent>) -> anyhow::Result<u16> {
    loop {
        match cmd_events.recv().await {
            Some(CommandEvent::Stdout(line)) => {
                if let Ok(line_str) = String::from_utf8(line) {
                    if let Some(port) = extract_port(&line_str)? {
                        return Ok(port);
                    }
                }
            }
            Some(CommandEvent::Terminated(_)) => {
                return Err(anyhow::anyhow!("sidecar exited before port discovery"));
            }
            None => {
                return Err(anyhow::anyhow!("sidecar event stream closed"));
            }
            _ => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port_from_harness_format() {
        let line = "dsh web: http://127.0.0.1:52631\n";
        assert_eq!(extract_port(line).unwrap(), Some(52631));
    }

    #[test]
    fn test_parse_port_from_listening_line() {
        let line = "  dsh: server listening on 127.0.0.1:49152\n";
        assert_eq!(extract_port(line).unwrap(), Some(49152));
    }

    #[test]
    fn test_parse_port_from_different_format() {
        let line = "[webserver] bound to 127.0.0.1:8080\n";
        assert_eq!(extract_port(line).unwrap(), Some(8080));
    }

    #[test]
    fn test_parse_port_returns_none_for_non_matching_line() {
        let line = "some random log output\n";
        assert_eq!(extract_port(line).unwrap(), None);
    }

    #[test]
    fn test_parse_port_handles_empty_string() {
        assert_eq!(extract_port("").unwrap(), None);
    }
}
