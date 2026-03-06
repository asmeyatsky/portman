/// Domain layer unit tests — pure logic, no mocks, no infrastructure.
/// Per skill2026: domain tests verify business rules using only domain types.

use portman::domain::{Registry, PortmanError};

#[test]
fn new_registry_is_empty() {
    let registry = Registry::new();
    assert_eq!(registry.version, 1);
    assert!(registry.assignments.is_empty());
    assert_eq!(registry.config.default_range_start, 8400);
    assert_eq!(registry.config.default_range_end, 8999);
}

#[test]
fn assign_creates_entry() {
    let mut registry = Registry::new();
    registry
        .assign("traderbot", 8420, Some("FastAPI backend".into()), false)
        .unwrap();

    assert_eq!(registry.assignments.len(), 1);
    let assignment = &registry.assignments["traderbot"];
    assert_eq!(assignment.port, 8420);
    assert_eq!(assignment.notes.as_deref(), Some("FastAPI backend"));
}

#[test]
fn assign_duplicate_port_fails() {
    let mut registry = Registry::new();
    registry.assign("traderbot", 8420, None, false).unwrap();

    let result = registry.assign("stacklens", 8420, None, false);
    assert!(result.is_err());
    match result.unwrap_err() {
        PortmanError::PortConflict { port, name } => {
            assert_eq!(port, 8420);
            assert_eq!(name, "traderbot");
        }
        other => panic!("Expected PortConflict, got: {other}"),
    }
}

#[test]
fn assign_duplicate_port_with_force_succeeds() {
    let mut registry = Registry::new();
    registry.assign("traderbot", 8420, None, false).unwrap();
    registry.assign("stacklens", 8420, None, true).unwrap();

    assert!(!registry.assignments.contains_key("traderbot"));
    assert_eq!(registry.assignments["stacklens"].port, 8420);
}

#[test]
fn assign_same_name_different_port_fails() {
    let mut registry = Registry::new();
    registry.assign("traderbot", 8420, None, false).unwrap();

    let result = registry.assign("traderbot", 9999, None, false);
    assert!(result.is_err());
    match result.unwrap_err() {
        PortmanError::NameConflict(name, port) => {
            assert_eq!(name, "traderbot");
            assert_eq!(port, 8420);
        }
        other => panic!("Expected NameConflict, got: {other}"),
    }
}

#[test]
fn assign_same_name_same_port_updates() {
    let mut registry = Registry::new();
    registry.assign("traderbot", 8420, None, false).unwrap();
    registry
        .assign("traderbot", 8420, Some("updated note".into()), false)
        .unwrap();

    assert_eq!(registry.assignments.len(), 1);
    assert_eq!(
        registry.assignments["traderbot"].notes.as_deref(),
        Some("updated note")
    );
}

#[test]
fn release_removes_entry() {
    let mut registry = Registry::new();
    registry.assign("traderbot", 8420, None, false).unwrap();

    let released = registry.release("traderbot").unwrap();
    assert_eq!(released.port, 8420);
    assert!(registry.assignments.is_empty());
}

#[test]
fn release_nonexistent_project_fails() {
    let mut registry = Registry::new();
    let result = registry.release("ghost");
    assert!(matches!(result.unwrap_err(), PortmanError::ProjectNotFound(_)));
}

#[test]
fn release_all_clears_registry() {
    let mut registry = Registry::new();
    registry.assign("a", 8001, None, false).unwrap();
    registry.assign("b", 8002, None, false).unwrap();
    registry.assign("c", 8003, None, false).unwrap();

    let count = registry.release_all();
    assert_eq!(count, 3);
    assert!(registry.assignments.is_empty());
}

#[test]
fn find_by_port_returns_correct_entry() {
    let mut registry = Registry::new();
    registry.assign("traderbot", 8420, None, false).unwrap();
    registry.assign("stacklens", 8421, None, false).unwrap();

    let (name, assignment) = registry.find_by_port(8421).unwrap();
    assert_eq!(name, "stacklens");
    assert_eq!(assignment.port, 8421);
}

#[test]
fn find_by_port_returns_none_for_unassigned() {
    let registry = Registry::new();
    assert!(registry.find_by_port(9999).is_none());
}

#[test]
fn is_port_assigned_reflects_state() {
    let mut registry = Registry::new();
    assert!(!registry.is_port_assigned(8420));

    registry.assign("traderbot", 8420, None, false).unwrap();
    assert!(registry.is_port_assigned(8420));
    assert!(!registry.is_port_assigned(8421));
}
