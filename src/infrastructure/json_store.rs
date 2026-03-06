use std::fs;
use std::path::PathBuf;

use crate::domain::{PortmanError, Registry, RegistryStore};

/// Adapter: persists the Registry as pretty-printed JSON on disk.
pub struct JsonRegistryStore {
    path: PathBuf,
}

impl JsonRegistryStore {
    pub fn new() -> Result<Self, PortmanError> {
        let home = dirs::home_dir().ok_or_else(|| {
            PortmanError::Other("Could not determine home directory".into())
        })?;
        let dir = home.join(".portman");
        fs::create_dir_all(&dir)?;
        Ok(Self {
            path: dir.join("registry.json"),
        })
    }

    /// Create a store backed by a specific file path (useful for testing).
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }
}

impl RegistryStore for JsonRegistryStore {
    fn load(&self) -> Result<Registry, PortmanError> {
        if !self.path.exists() {
            return Ok(Registry::new());
        }
        let content = fs::read_to_string(&self.path)?;
        if content.trim().is_empty() {
            return Ok(Registry::new());
        }
        let registry: Registry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    fn save(&self, registry: &Registry) -> Result<(), PortmanError> {
        let content = serde_json::to_string_pretty(registry)?;
        fs::write(&self.path, format!("{content}\n"))?;
        Ok(())
    }
}
