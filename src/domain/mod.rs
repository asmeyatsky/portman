/// Domain Layer — Core business logic with zero infrastructure dependencies.
///
/// Architectural Intent:
/// - Registry is the aggregate root managing port assignments
/// - Assignment is an immutable value object
/// - All business rules (conflict detection, validation) live here
/// - External capabilities abstracted behind port traits (RegistryStore, SocketChecker)
/// - Domain layer has NO knowledge of JSON files, TCP sockets, or CLI frameworks

mod entities;
mod errors;
mod ports;

pub use entities::{Assignment, Registry, RegistryConfig};
pub use errors::PortmanError;
pub use ports::{ProcessInfo, RegistryStore, SocketChecker};
