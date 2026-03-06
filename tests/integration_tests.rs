/// Integration tests — exercise the full assign/conflict/release cycle
/// and JSON round-trip through the real JsonRegistryStore adapter.

use std::path::PathBuf;

use portman::domain::{Registry, RegistryStore};
use portman::infrastructure::JsonRegistryStore;

fn temp_store() -> (JsonRegistryStore, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("registry.json");
    (JsonRegistryStore::with_path(path), dir)
}

#[test]
fn load_returns_empty_registry_when_file_missing() {
    let (store, _dir) = temp_store();
    let registry = store.load().unwrap();
    assert!(registry.assignments.is_empty());
}

#[test]
fn save_and_load_round_trip() {
    let (store, _dir) = temp_store();

    let mut registry = Registry::new();
    registry
        .assign("traderbot", 8420, Some("FastAPI backend".into()), false)
        .unwrap();
    registry.assign("stacklens", 8421, None, false).unwrap();

    store.save(&registry).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded.assignments.len(), 2);
    assert_eq!(loaded.assignments["traderbot"].port, 8420);
    assert_eq!(
        loaded.assignments["traderbot"].notes.as_deref(),
        Some("FastAPI backend")
    );
    assert_eq!(loaded.assignments["stacklens"].port, 8421);
}

#[test]
fn assign_conflict_release_cycle() {
    let (store, _dir) = temp_store();

    // Assign
    let mut registry = store.load().unwrap();
    registry.assign("app", 8500, None, false).unwrap();
    store.save(&registry).unwrap();

    // Conflict
    let mut registry = store.load().unwrap();
    let result = registry.assign("other", 8500, None, false);
    assert!(result.is_err());

    // Release
    let mut registry = store.load().unwrap();
    registry.release("app").unwrap();
    store.save(&registry).unwrap();

    // Now the port is free in the registry
    let mut registry = store.load().unwrap();
    registry.assign("other", 8500, None, false).unwrap();
    store.save(&registry).unwrap();

    let final_reg = store.load().unwrap();
    assert_eq!(final_reg.assignments.len(), 1);
    assert_eq!(final_reg.assignments["other"].port, 8500);
}

#[test]
fn load_sample_fixture() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_registry.json");
    let store = JsonRegistryStore::with_path(fixture);
    let registry = store.load().unwrap();

    assert_eq!(registry.version, 1);
    assert_eq!(registry.assignments.len(), 3);
    assert_eq!(registry.assignments["traderbot"].port, 8420);
    assert_eq!(registry.assignments["forge"].port, 8450);
    assert_eq!(registry.assignments["claudeguard"].port, 8460);
}

#[test]
fn json_output_is_sorted_by_key() {
    let (store, _dir) = temp_store();

    let mut registry = Registry::new();
    registry.assign("zebra", 8001, None, false).unwrap();
    registry.assign("alpha", 8002, None, false).unwrap();
    registry.assign("middle", 8003, None, false).unwrap();
    store.save(&registry).unwrap();

    let loaded = store.load().unwrap();
    let json = serde_json::to_string_pretty(&loaded.assignments).unwrap();

    let alpha_pos = json.find("alpha").unwrap();
    let middle_pos = json.find("middle").unwrap();
    let zebra_pos = json.find("zebra").unwrap();

    assert!(alpha_pos < middle_pos);
    assert!(middle_pos < zebra_pos);
}
