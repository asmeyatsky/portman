use colored::Colorize;
use std::fs;

use crate::domain::{PortmanError, Registry, RegistryStore};

pub fn export(store: &dyn RegistryStore, format: &str) -> Result<(), PortmanError> {
    let registry = store.load()?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&registry)?);
        }
        "env" => {
            for (name, assignment) in &registry.assignments {
                let env_name = name.to_uppercase().replace('-', "_");
                println!("export {env_name}_PORT={}", assignment.port);
            }
        }
        _ => {
            return Err(PortmanError::Other(format!(
                "Unknown format '{format}'. Supported: json, env"
            )));
        }
    }

    Ok(())
}

pub fn import(store: &dyn RegistryStore, file: &str, merge: bool) -> Result<(), PortmanError> {
    let content = fs::read_to_string(file)?;
    let imported: Registry = serde_json::from_str(&content)?;

    if merge {
        let mut registry = store.load()?;
        let mut added = 0;
        for (name, assignment) in imported.assignments {
            if !registry.assignments.contains_key(&name) {
                registry.assignments.insert(name, assignment);
                added += 1;
            }
        }
        store.save(&registry)?;
        println!("{} Merged {} new assignments", "✓".green(), added);
    } else {
        store.save(&imported)?;
        println!(
            "{} Imported {} assignments",
            "✓".green(),
            imported.assignments.len()
        );
    }

    Ok(())
}
