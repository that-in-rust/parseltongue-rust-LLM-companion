//! Parseltongue Core Library
//!
//! This crate provides the foundational types, traits, and utilities used across
//! all Parseltongue tools. Following TDD-first principles with executable
//! specifications and functional programming patterns.

#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(missing_docs)]

pub mod diff; // v1.1.0: Diff visualization system
pub mod entities;
pub mod entity_class_specifications;
pub mod error;
pub mod interfaces;
pub mod output_path_resolver; // v0.9.7: Timestamped folder creation
pub mod query_extractor;
pub mod query_json_graph_errors; // v0.9.7: Agent query error types
pub mod query_json_graph_helpers; // v0.9.7: Agent JSON graph traversal
pub mod serializers; // v0.10.0: Core serialization (JSON, TOON)
pub mod storage;
pub mod temporal;
pub mod workspace; // v2.0.0 Phase 2.1: Workspace management types

// Re-export commonly used types
pub use entities::*;
pub use error::*;
pub use interfaces::*;
pub use query_json_graph_errors::*; // v0.9.7: Query error types
pub use query_json_graph_helpers::*; // v0.9.7: Agent query functions
pub use serializers::*; // Export Serializer trait + implementations
pub use storage::*;
pub use temporal::*;
pub use workspace::*; // v2.0.0: Workspace management types