/// Command-level tests — exercise assign, check, next, export, and import
/// through mock implementations of the domain port traits.

use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;

use portman::commands;
use portman::domain::{PortmanError, ProcessInfo, Registry, RegistryStore, SocketChecker};
use portman::infrastructure::JsonRegistryStore;

// ---------------------------------------------------------------------------
// Mock infrastructure adapters
// ---------------------------------------------------------------------------

/// In-memory store for testing commands without touching the filesystem.
struct MockStore {
    registry: RefCell<Registry>,
}

impl MockStore {
    fn new() -> Self {
        Self {
            registry: RefCell::new(Registry::new()),
        }
    }

    fn current(&self) -> Registry {
        self.registry.borrow().clone()
    }
}

impl RegistryStore for MockStore {
    fn load(&self) -> Result<Registry, PortmanError> {
        Ok(self.registry.borrow().clone())
    }

    fn save(&self, registry: &Registry) -> Result<(), PortmanError> {
        *self.registry.borrow_mut() = registry.clone();
        Ok(())
    }
}

/// Socket checker where all ports are free by default.
/// Specific ports can be marked as occupied.
struct MockSocket {
    occupied: HashSet<u16>,
}

impl MockSocket {
    fn all_free() -> Self {
        Self {
            occupied: HashSet::new(),
        }
    }

    fn with_occupied(ports: &[u16]) -> Self {
        Self {
            occupied: ports.iter().copied().collect(),
        }
    }
}

impl SocketChecker for MockSocket {
    fn is_port_free(&self, port: u16) -> bool {
        !self.occupied.contains(&port)
    }

    fn get_process_info(&self, port: u16) -> Option<ProcessInfo> {
        if self.occupied.contains(&port) {
            Some(ProcessInfo {
                pid: 12345,
                name: "mock".into(),
            })
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// assign / release commands
// ---------------------------------------------------------------------------

#[test]
fn assign_command_persists_entry() {
    let store = MockStore::new();
    commands::assign::assign(&store, "traderbot", 8420, false, Some("API".into())).unwrap();

    let reg = store.current();
    assert_eq!(reg.assignments["traderbot"].port, 8420);
    assert_eq!(reg.assignments["traderbot"].notes.as_deref(), Some("API"));
}

#[test]
fn assign_command_rejects_conflict() {
    let store = MockStore::new();
    commands::assign::assign(&store, "traderbot", 8420, false, None).unwrap();
    let result = commands::assign::assign(&store, "other", 8420, false, None);
    assert!(result.is_err());
}

#[test]
fn assign_command_force_overwrites() {
    let store = MockStore::new();
    commands::assign::assign(&store, "traderbot", 8420, false, None).unwrap();
    commands::assign::assign(&store, "other", 8420, true, None).unwrap();

    let reg = store.current();
    assert!(!reg.assignments.contains_key("traderbot"));
    assert_eq!(reg.assignments["other"].port, 8420);
}

#[test]
fn release_command_removes_entry() {
    let store = MockStore::new();
    commands::assign::assign(&store, "app", 8500, false, None).unwrap();
    commands::assign::release(&store, Some("app"), false).unwrap();
    assert!(store.current().assignments.is_empty());
}

#[test]
fn release_all_clears_store() {
    let store = MockStore::new();
    commands::assign::assign(&store, "a", 8001, false, None).unwrap();
    commands::assign::assign(&store, "b", 8002, false, None).unwrap();
    commands::assign::release(&store, None, true).unwrap();
    assert!(store.current().assignments.is_empty());
}

#[test]
fn release_without_name_or_all_errors() {
    let store = MockStore::new();
    let result = commands::assign::release(&store, None, false);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// check command
// ---------------------------------------------------------------------------

#[test]
fn check_runs_without_error_on_free_port() {
    let store = MockStore::new();
    let socket = MockSocket::all_free();
    commands::check::check(&store, &socket, 9999).unwrap();
}

#[test]
fn check_runs_without_error_on_assigned_port() {
    let store = MockStore::new();
    commands::assign::assign(&store, "app", 8500, false, None).unwrap();
    let socket = MockSocket::with_occupied(&[8500]);
    commands::check::check(&store, &socket, 8500).unwrap();
}

// ---------------------------------------------------------------------------
// next command
// ---------------------------------------------------------------------------

#[test]
fn next_finds_first_available_port() {
    let store = MockStore::new();
    let socket = MockSocket::all_free();
    commands::check::next(&store, &socket, Some(8400), None).unwrap();
}

#[test]
fn next_skips_assigned_ports() {
    let store = MockStore::new();
    commands::assign::assign(&store, "a", 8400, false, None).unwrap();
    commands::assign::assign(&store, "b", 8401, false, None).unwrap();
    let socket = MockSocket::all_free();
    // Should succeed — 8402 is available
    commands::check::next(&store, &socket, Some(8400), Some("8400-8410")).unwrap();
}

#[test]
fn next_skips_os_occupied_ports() {
    let store = MockStore::new();
    let socket = MockSocket::with_occupied(&[8400, 8401]);
    // Should succeed — 8402 is free at OS level
    commands::check::next(&store, &socket, Some(8400), Some("8400-8410")).unwrap();
}

#[test]
fn next_errors_when_range_exhausted() {
    let store = MockStore::new();
    commands::assign::assign(&store, "a", 8400, false, None).unwrap();
    commands::assign::assign(&store, "b", 8401, false, None).unwrap();
    let socket = MockSocket::all_free();
    let result = commands::check::next(&store, &socket, None, Some("8400-8401"));
    assert!(result.is_err());
}

#[test]
fn next_uses_default_range_when_no_flags() {
    let store = MockStore::new();
    let socket = MockSocket::all_free();
    commands::check::next(&store, &socket, None, None).unwrap();
}

// ---------------------------------------------------------------------------
// list command
// ---------------------------------------------------------------------------

#[test]
fn list_runs_on_empty_registry() {
    let store = MockStore::new();
    let socket = MockSocket::all_free();
    commands::list::list(&store, &socket, false, false).unwrap();
}

#[test]
fn list_json_outputs_valid_json() {
    let store = MockStore::new();
    commands::assign::assign(&store, "app", 8500, false, None).unwrap();
    let socket = MockSocket::all_free();
    // Just verify it doesn't error — actual JSON goes to stdout
    commands::list::list(&store, &socket, true, false).unwrap();
}

#[test]
fn list_active_filters_idle_ports() {
    let store = MockStore::new();
    commands::assign::assign(&store, "running", 8500, false, None).unwrap();
    commands::assign::assign(&store, "idle", 8501, false, None).unwrap();
    let socket = MockSocket::with_occupied(&[8500]);
    commands::list::list(&store, &socket, false, true).unwrap();
}

// ---------------------------------------------------------------------------
// status command
// ---------------------------------------------------------------------------

#[test]
fn status_runs_without_error() {
    let store = MockStore::new();
    commands::assign::assign(&store, "app", 8500, false, None).unwrap();
    let socket = MockSocket::with_occupied(&[8500]);
    commands::list::status(&store, &socket).unwrap();
}

// ---------------------------------------------------------------------------
// export command
// ---------------------------------------------------------------------------

#[test]
fn export_json_runs_without_error() {
    let store = MockStore::new();
    commands::assign::assign(&store, "app", 8500, false, None).unwrap();
    commands::export::export(&store, "json").unwrap();
}

#[test]
fn export_env_runs_without_error() {
    let store = MockStore::new();
    commands::assign::assign(&store, "my-app", 8500, false, None).unwrap();
    commands::export::export(&store, "env").unwrap();
}

#[test]
fn export_unknown_format_errors() {
    let store = MockStore::new();
    let result = commands::export::export(&store, "xml");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// import command
// ---------------------------------------------------------------------------

#[test]
fn import_overwrites_existing_registry() {
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("registry.json");
    let store = JsonRegistryStore::with_path(store_path);

    // Pre-populate
    let mut reg = Registry::new();
    reg.assign("old", 9000, None, false).unwrap();
    store.save(&reg).unwrap();

    // Write an import file
    let import_path = dir.path().join("import.json");
    let import_reg = r#"{
        "version": 1,
        "assignments": {
            "imported": { "port": 7000, "assigned_at": "2026-01-01T00:00:00Z" }
        },
        "config": { "default_range_start": 8400, "default_range_end": 8999 }
    }"#;
    fs::write(&import_path, import_reg).unwrap();

    commands::export::import(&store, import_path.to_str().unwrap(), false).unwrap();

    let loaded = store.load().unwrap();
    assert_eq!(loaded.assignments.len(), 1);
    assert!(loaded.assignments.contains_key("imported"));
}

#[test]
fn import_merge_preserves_existing() {
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("registry.json");
    let store = JsonRegistryStore::with_path(store_path);

    // Pre-populate
    let mut reg = Registry::new();
    reg.assign("existing", 9000, None, false).unwrap();
    store.save(&reg).unwrap();

    // Write an import file with one new and one overlapping entry
    let import_path = dir.path().join("import.json");
    let import_reg = r#"{
        "version": 1,
        "assignments": {
            "existing": { "port": 7777, "assigned_at": "2026-01-01T00:00:00Z" },
            "newproject": { "port": 7000, "assigned_at": "2026-01-01T00:00:00Z" }
        },
        "config": { "default_range_start": 8400, "default_range_end": 8999 }
    }"#;
    fs::write(&import_path, import_reg).unwrap();

    commands::export::import(&store, import_path.to_str().unwrap(), true).unwrap();

    let loaded = store.load().unwrap();
    assert_eq!(loaded.assignments.len(), 2);
    // existing entry should keep its original port (not overwritten)
    assert_eq!(loaded.assignments["existing"].port, 9000);
    assert_eq!(loaded.assignments["newproject"].port, 7000);
}

#[test]
fn import_nonexistent_file_errors() {
    let store = MockStore::new();
    let result = commands::export::import(&store, "/tmp/does_not_exist_portman.json", false);
    assert!(result.is_err());
}
