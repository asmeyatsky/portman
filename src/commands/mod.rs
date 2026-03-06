/// Application Layer — Use cases orchestrating domain logic through ports.
///
/// Each module corresponds to a group of related CLI commands.
/// Commands depend only on domain types and port traits, never on
/// concrete infrastructure.

pub mod assign;
pub mod check;
pub mod export;
pub mod list;
