use colored::Colorize;

use crate::domain::{PortmanError, RegistryStore, SocketChecker};

pub fn check(
    store: &dyn RegistryStore,
    socket: &dyn SocketChecker,
    port: u16,
) -> Result<(), PortmanError> {
    let registry = store.load()?;

    // Registry status
    let registry_status = match registry.find_by_port(port) {
        Some((name, _)) => format!("assigned to {}", name.bold()),
        None => "free".green().to_string(),
    };

    // Socket status
    let socket_status = if socket.is_port_free(port) {
        "free".green().to_string()
    } else {
        match socket.get_process_info(port) {
            Some(pi) => format!(
                "in use by PID {} ({})",
                pi.pid.to_string().yellow(),
                pi.name
            ),
            None => "in use".red().to_string(),
        }
    };

    println!("{:<11} {}", "Registry:".bold(), registry_status);
    println!("{:<11} {}", "Socket:".bold(), socket_status);

    Ok(())
}

/// Application-layer orchestration: finds the next port that is both
/// unassigned in the registry AND free at the OS socket level.
pub fn next(
    store: &dyn RegistryStore,
    socket: &dyn SocketChecker,
    from: Option<u16>,
    range: Option<&str>,
) -> Result<(), PortmanError> {
    let registry = store.load()?;

    let (start, end) = if let Some(range_str) = range {
        parse_range(range_str)?
    } else {
        let start = from.unwrap_or(registry.config.default_range_start);
        (start, registry.config.default_range_end)
    };

    let next_port = (start..=end)
        .find(|&port| !registry.is_port_assigned(port) && socket.is_port_free(port));

    match next_port {
        Some(port) => println!("Next available: {}", port.to_string().green().bold()),
        None => return Err(PortmanError::NoAvailablePort(start, end)),
    }

    Ok(())
}

fn parse_range(range: &str) -> Result<(u16, u16), PortmanError> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return Err(PortmanError::Other(format!(
            "Invalid range format '{range}'. Expected: start-end (e.g. 8400-8999)"
        )));
    }
    let start: u16 = parts[0]
        .parse()
        .map_err(|_| PortmanError::Other(format!("Invalid range start: '{}'", parts[0])))?;
    let end: u16 = parts[1]
        .parse()
        .map_err(|_| PortmanError::Other(format!("Invalid range end: '{}'", parts[1])))?;
    Ok((start, end))
}
