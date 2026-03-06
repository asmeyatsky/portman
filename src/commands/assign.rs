use colored::Colorize;

use crate::domain::{PortmanError, RegistryStore};

pub fn assign(
    store: &dyn RegistryStore,
    name: &str,
    port: u16,
    force: bool,
    notes: Option<String>,
) -> Result<(), PortmanError> {
    let mut registry = store.load()?;
    registry.assign(name, port, notes, force)?;
    store.save(&registry)?;
    println!(
        "{} Assigned {} → {}",
        "✓".green(),
        name.bold(),
        port.to_string().cyan()
    );
    Ok(())
}

pub fn release(
    store: &dyn RegistryStore,
    name: Option<&str>,
    all: bool,
) -> Result<(), PortmanError> {
    let mut registry = store.load()?;

    if all {
        let count = registry.release_all();
        store.save(&registry)?;
        println!("{} Released all {} assignments", "✓".green(), count);
    } else {
        let name = name.ok_or_else(|| {
            PortmanError::Other(
                "Provide a project name or use --all to clear everything".into(),
            )
        })?;
        let assignment = registry.release(name)?;
        store.save(&registry)?;
        println!(
            "{} Released {} (was port {})",
            "✓".green(),
            name.bold(),
            assignment.port.to_string().cyan()
        );
    }

    Ok(())
}
