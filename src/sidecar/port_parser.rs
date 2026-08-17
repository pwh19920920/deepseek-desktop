/// Port extraction from harness sidecar stdout.
///
/// The harness webserver plugin prints a line containing the bound port
/// when it starts listening. This module parses those lines to discover
/// the port assigned to the sidecar process.
use tokio::io::{AsyncBufReadExt, BufReader};

/// Regex patterns that match the harness webserver startup log lines.
const PORT_PATTERNS: &[&str] = &[
    r"listening on \S+:(\d+)",
    r"bound to \S+:(\d+)",
    r"server started on \S+:(\d+)",
    // Matches the harness output: "dsh web: http://127.0.0.1:52631"
    r"127\.0\.0\.1:(\d+)",
    r":(\d+)\s*$", // fallback: bare port at end of line
];

/// Extract the port number from a single log line.
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

/// Read lines from the sidecar's stdout until a port is discovered.
pub async fn discover_port(child: &mut tokio::process::Child) -> anyhow::Result<u16> {
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
