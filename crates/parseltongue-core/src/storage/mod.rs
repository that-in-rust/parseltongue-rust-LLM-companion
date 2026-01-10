//! Storage implementations for Parseltongue.
//!
//! Provides real database storage using CozoDB with SQLite backend,
//! implementing the CodeGraphRepository trait for dependency injection.

pub mod cozo_client;

pub use cozo_client::CozoDbStorage;
