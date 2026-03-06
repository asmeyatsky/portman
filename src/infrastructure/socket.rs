use std::net::TcpListener;
use std::process::Command;

use crate::domain::{ProcessInfo, SocketChecker};

/// Adapter: checks live OS socket state using stdlib TcpListener and lsof.
///
/// Key Rust lesson: TcpListener demonstrates RAII — the socket is released
/// the moment the binding drops out of scope. No explicit close needed.
pub struct OsSocketChecker;

impl SocketChecker for OsSocketChecker {
    fn is_port_free(&self, port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    fn get_process_info(&self, port: u16) -> Option<ProcessInfo> {
        let output = Command::new("lsof")
            .args(["-i", &format!(":{port}"), "-sTCP:LISTEN", "-t"])
            .output()
            .ok()?;

        let pid_str = String::from_utf8_lossy(&output.stdout);
        let pid: u32 = pid_str.trim().lines().next()?.parse().ok()?;

        let ps_output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .ok()?;

        let name = String::from_utf8_lossy(&ps_output.stdout).trim().to_string();

        Some(ProcessInfo {
            pid,
            name: if name.is_empty() {
                "unknown".into()
            } else {
                name
            },
        })
    }
}
