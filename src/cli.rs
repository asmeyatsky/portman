/// Presentation Layer — CLI argument parsing via clap derive macros.
///
/// Enum-based dispatch: each subcommand maps to a Command variant.
/// Global flags can be added to the Cli struct as needed.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "portman",
    about = "Local port registry CLI — know your ports",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Reserve a port for a named project
    Assign {
        /// Project name
        name: String,
        /// Port number to assign
        port: u16,
        /// Overwrite existing assignment
        #[arg(long)]
        force: bool,
        /// Optional note about the project
        #[arg(long)]
        notes: Option<String>,
    },

    /// Free the port assigned to a project
    Release {
        /// Project name (omit with --all to clear everything)
        name: Option<String>,
        /// Clear the entire registry
        #[arg(long)]
        all: bool,
    },

    /// Show all assignments with live socket status
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show only ports currently in use
        #[arg(long)]
        active: bool,
    },

    /// Report whether a port is free in the registry and on the OS
    Check {
        /// Port number to check
        port: u16,
    },

    /// Suggest the next unassigned port
    Next {
        /// Start search from this port
        #[arg(long)]
        from: Option<u16>,
        /// Constrain search to range (e.g. 8400-8999)
        #[arg(long)]
        range: Option<String>,
    },

    /// Show all assigned ports with live process info
    Status,

    /// Export the registry as JSON or shell env statements
    Export {
        /// Output format: json or env
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Import assignments from a JSON file
    Import {
        /// Path to JSON file
        file: String,
        /// Merge with existing entries instead of overwriting
        #[arg(long)]
        merge: bool,
    },
}
