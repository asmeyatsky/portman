/// Infrastructure Layer — Adapters implementing domain ports.
///
/// - JsonRegistryStore: persists Registry to ~/.portman/registry.json
/// - OsSocketChecker: checks live port status via TcpListener and lsof

mod json_store;
mod socket;

pub use json_store::JsonRegistryStore;
pub use socket::OsSocketChecker;
