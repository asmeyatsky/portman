use super::{PortmanError, Registry};

/// Process information for a port that is actively in use.
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

/// Port for persisting and loading the registry (infrastructure boundary).
pub trait RegistryStore {
    fn load(&self) -> Result<Registry, PortmanError>;
    fn save(&self, registry: &Registry) -> Result<(), PortmanError>;
}

/// Port for checking live OS socket state (infrastructure boundary).
pub trait SocketChecker {
    fn is_port_free(&self, port: u16) -> bool;
    fn get_process_info(&self, port: u16) -> Option<ProcessInfo>;
}
