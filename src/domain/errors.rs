use thiserror::Error;

#[derive(Debug, Error)]
pub enum PortmanError {
    #[error("Port {port} is already assigned to '{name}'")]
    PortConflict { port: u16, name: String },

    #[error("Project '{0}' not found in registry")]
    ProjectNotFound(String),

    #[error("'{0}' is already assigned to port {1}. Use --force to reassign.")]
    NameConflict(String, u16),

    #[error("No available port found in range {0}-{1}")]
    NoAvailablePort(u16, u16),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
