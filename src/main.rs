/// portman — Local Port Registry CLI
///
/// Composition root: wires infrastructure adapters to application commands.
/// Dependency injection is done here at the top level, keeping all other
/// layers decoupled from concrete implementations.

mod cli;

use clap::Parser;
use colored::Colorize;

use cli::{Cli, Command};
use portman::commands;
use portman::infrastructure::{JsonRegistryStore, OsSocketChecker};

fn main() {
    let cli = Cli::parse();

    let store = match JsonRegistryStore::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    };
    let socket = OsSocketChecker;

    let result = match cli.command {
        Command::Assign {
            name,
            port,
            force,
            notes,
        } => commands::assign::assign(&store, &name, port, force, notes),

        Command::Release { name, all } => {
            commands::assign::release(&store, name.as_deref(), all)
        }

        Command::List { json, active } => commands::list::list(&store, &socket, json, active),

        Command::Check { port } => commands::check::check(&store, &socket, port),

        Command::Next { from, range } => {
            commands::check::next(&store, &socket, from, range.as_deref())
        }

        Command::Status => commands::list::status(&store, &socket),

        Command::Export { format } => commands::export::export(&store, &format),

        Command::Import { file, merge } => commands::export::import(&store, &file, merge),
    };

    if let Err(e) = result {
        eprintln!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    }
}
