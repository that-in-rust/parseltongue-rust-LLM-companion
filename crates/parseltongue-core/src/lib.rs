//! Parseltongue Core Library
//!
//! This crate provides the foundational types, traits, and utilities used across
//! all Parseltongue tools. Following TDD-first principles with executable
//! specifications and functional programming patterns.

#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(missing_docs)]

pub mod entities;
pub mod entity_class_specifications;
// pub mod entity_conversion; // P5: Entity conversion utilities (TODO: implement)
pub mod error;
// pub mod file_parser; // P1: Thread-safe file parser facade (TODO: implement)
pub mod interfaces;
pub mod isgl1_v2; // v1.4.5: ISGL1 v2 stable entity identity with birth timestamps
pub mod output_path_resolver; // v0.9.7: Timestamped folder creation
pub mod query_extractor;
pub mod query_json_graph_errors; // v0.9.7: Agent query error types
pub mod query_json_graph_helpers; // v0.9.7: Agent JSON graph traversal
pub mod serializers; // v0.10.0: Core serialization (JSON, TOON)
pub mod storage;
pub mod temporal;

// Re-export commonly used types
pub use entities::*;
pub use error::*;
pub use interfaces::*;
pub use query_json_graph_errors::*; // v0.9.7: Query error types
pub use query_json_graph_helpers::*; // v0.9.7: Agent query functions
pub use serializers::*; // Export Serializer trait + implementations
pub use storage::*;
pub use temporal::*;