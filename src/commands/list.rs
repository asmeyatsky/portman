use colored::Colorize;

use crate::domain::{PortmanError, RegistryStore, SocketChecker};

pub fn list(
    store: &dyn RegistryStore,
    socket: &dyn SocketChecker,
    json: bool,
    active: bool,
) -> Result<(), PortmanError> {
    let registry = store.load()?;

    if json {
        if active {
            let active_entries: std::collections::BTreeMap<&str, _> = registry
                .assignments
                .iter()
                .filter(|(_, a)| !socket.is_port_free(a.port))
                .map(|(n, a)| (n.as_str(), a))
                .collect();
            println!("{}", serde_json::to_string_pretty(&active_entries)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&registry.assignments)?);
        }
        return Ok(());
    }

    if registry.assignments.is_empty() {
        println!(
            "No port assignments. Use {} to add one.",
            "portman assign <name> <port>".bold()
        );
        return Ok(());
    }

    println!(
        "{:<16} {:<8} {}",
        "PROJECT".bold(),
        "PORT".bold(),
        "STATUS".bold()
    );

    for (name, assignment) in &registry.assignments {
        let is_active = !socket.is_port_free(assignment.port);

        if active && !is_active {
            continue;
        }

        print!("{:<16} {:<8} ", name, assignment.port);
        if is_active {
            let pid_info = match socket.get_process_info(assignment.port) {
                Some(pi) => format!("  (PID {})", pi.pid),
                None => String::new(),
            };
            println!("{} {}{}", "●".green(), "RUNNING".green(), pid_info);
        } else {
            println!("{} {}", "○".dimmed(), "idle".dimmed());
        }
    }

    Ok(())
}

pub fn status(
    store: &dyn RegistryStore,
    socket: &dyn SocketChecker,
) -> Result<(), PortmanError> {
    let registry = store.load()?;

    if registry.assignments.is_empty() {
        println!("No port assignments.");
        return Ok(());
    }

    println!(
        "{:<16} {:<8} {:<14} {}",
        "PROJECT".bold(),
        "PORT".bold(),
        "STATUS".bold(),
        "PROCESS".bold()
    );

    for (name, assignment) in &registry.assignments {
        let is_active = !socket.is_port_free(assignment.port);

        print!("{:<16} {:<8} ", name, assignment.port);
        if is_active {
            match socket.get_process_info(assignment.port) {
                Some(pi) => {
                    print!("{} {:<10} ", "●".green(), "RUNNING".green());
                    println!("{} (PID {})", pi.name, pi.pid);
                }
                None => {
                    print!("{} {:<10} ", "●".green(), "RUNNING".green());
                    println!("{}", "—".dimmed());
                }
            }
        } else {
            print!("{} {:<10} ", "○".dimmed(), "idle".dimmed());
            println!("{}", "—".dimmed());
        }
    }

    Ok(())
}
