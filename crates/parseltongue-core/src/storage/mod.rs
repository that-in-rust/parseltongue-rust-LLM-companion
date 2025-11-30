//! Storage implementations for Parseltongue.
//!
//! Provides real database storage using CozoDB with SQLite backend,
//! implementing the CodeGraphRepository trait for dependency injection.
//!
//! # PRD v1.4.0 Schema
//! The `prd_schema_definition_tables` module provides the full 14-table
//! schema required by PRD v1.4.0, including CODE/TEST separation,
//! pre-computed metrics, and temporal coupling support.

pub mod cozo_client;
pub mod prd_schema_definition_tables;

pub use cozo_client::CozoDbStorage;
pub use prd_schema_definition_tables::PrdSchemaDefinitionTables;
