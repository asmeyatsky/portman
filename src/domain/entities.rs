use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::errors::PortmanError;

/// Value object representing a single port assignment. Immutable by design —
/// reassignment produces a new Assignment via Registry::assign().
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Assignment {
    pub port: u16,
    pub assigned_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistryConfig {
    pub default_range_start: u16,
    pub default_range_end: u16,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            default_range_start: 8400,
            default_range_end: 8999,
        }
    }
}

fn default_version() -> u32 {
    1
}

/// Aggregate root — the single source of truth for port assignments.
/// Uses BTreeMap for deterministic, diff-friendly JSON output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Registry {
    #[serde(default = "default_version")]
    pub version: u32,
    pub assignments: BTreeMap<String, Assignment>,
    #[serde(default)]
    pub config: RegistryConfig,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            version: 1,
            assignments: BTreeMap::new(),
            config: RegistryConfig::default(),
        }
    }

    /// Assign a port to a named project. Enforces two invariants:
    /// 1. No two projects share the same port (unless --force)
    /// 2. A project name maps to exactly one port (unless --force)
    pub fn assign(
        &mut self,
        name: &str,
        port: u16,
        notes: Option<String>,
        force: bool,
    ) -> Result<(), PortmanError> {
        // Check if port is taken by a different project
        if let Some((existing_name, _)) = self.find_by_port(port) {
            if existing_name != name {
                if !force {
                    return Err(PortmanError::PortConflict {
                        port,
                        name: existing_name.to_string(),
                    });
                }
                let existing_name = existing_name.to_string();
                self.assignments.remove(&existing_name);
            }
        }

        // Check if name already exists with a different port
        if let Some(existing) = self.assignments.get(name) {
            if existing.port != port && !force {
                return Err(PortmanError::NameConflict(name.to_string(), existing.port));
            }
        }

        let assignment = Assignment {
            port,
            assigned_at: chrono::Utc::now()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            notes,
        };
        self.assignments.insert(name.to_string(), assignment);
        Ok(())
    }

    pub fn release(&mut self, name: &str) -> Result<Assignment, PortmanError> {
        self.assignments
            .remove(name)
            .ok_or_else(|| PortmanError::ProjectNotFound(name.to_string()))
    }

    pub fn release_all(&mut self) -> usize {
        let count = self.assignments.len();
        self.assignments.clear();
        count
    }

    pub fn find_by_port(&self, port: u16) -> Option<(&str, &Assignment)> {
        self.assignments
            .iter()
            .find(|(_, a)| a.port == port)
            .map(|(name, a)| (name.as_str(), a))
    }

    pub fn is_port_assigned(&self, port: u16) -> bool {
        self.assignments.values().any(|a| a.port == port)
    }
}
