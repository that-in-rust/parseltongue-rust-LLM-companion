# 07: Rust Interface Definitions for Diff Visualization System

> **Status**: Proposed (TDD RED Phase - Test Stubs Only)
> **Date**: 2026-01-23
> **Related**: ADR_001_KEY_NORMALIZATION.md, 04_DATA_STRUCTURES.md

---

## Overview

This document defines the Rust trait and struct interfaces for the diff visualization system.
These definitions follow patterns discovered via the Parseltongue HTTP API and adhere to the
4-word naming convention from CLAUDE.md.

**Design Philosophy**:
- Trait-based dependency injection for testability
- Layered architecture (L1 Core -> L2 Std -> L3 External)
- TDD workflow: STUB -> RED -> GREEN -> REFACTOR

---

## 1. Patterns Discovered via API

### 1.1 Entity Key Format

From API query `/code-entities-search-fuzzy?q=entity`:

```
{language}:{entity_type}:{name}:{path_hash}:{start_line}-{end_line}

Examples discovered:
- rust:fn:calculate_entity_coupling_scores:__crates_pt08-http-code-query-server_src_...:118-192
- rust:struct:ParsedEntity:__crates_parseltongue-core_src_query_extractor_rs:36-43
- rust:method:entity_to_params:__crates_parseltongue-core_src_storage_cozo_client_rs:948-1102
- rust:fn:map:unknown:0-0  (external reference)
```

### 1.2 Existing Struct Patterns

From API query `/code-entities-search-fuzzy?q=struct`:

Naming pattern observed: `{Domain}{Purpose}{Suffix}`
- `EntityDetailDataPayload` - data payload for entity details
- `SearchResultEntityItem` - individual search result item
- `BlastRadiusQueryParamsStruct` - query parameters for blast radius
- `CozoDbStorage` - storage implementation

### 1.3 Module Structure

From API query `/semantic-cluster-grouping-list`:

- Cluster 1: 215 entities (core functionality, tests, temporal state)
- Cluster 2: 164 entities (handlers, transformations, query building)
- 48 total clusters organizing 1005 entities

### 1.4 Existing Method Patterns

From API query `/forward-callees-query-graph`:

Storage methods follow pattern: `{verb}_{target}` or `{verb}_{target}_{qualifier}`
- `entity_to_params` - conversion method
- `get_entity`, `store_entity`, `delete_entity` - CRUD operations
- `get_changed_entities` - filtered query

---

## 2. Core Data Structures

### 2.1 NormalizedEntityKey Struct

Encapsulates entity key with extracted stable identity for diff matching.

```rust
/// Normalized entity key with stable identity extraction.
///
/// # Key Format
/// Full: `rust:fn:main:__crates_src_main_rs:10-50`
/// Stable: `rust:fn:main:__crates_src_main_rs` (line numbers stripped)
///
/// # Invariants
/// - `full_key` always contains the complete original key
/// - `stable_identity` never contains line number suffix
/// - `line_range` is None for external entities (0-0 pattern)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedEntityKeyData {
    /// Original full key with line numbers
    pub full_key: String,

    /// Stable identity without line numbers (for diff matching)
    pub stable_identity: String,

    /// Extracted language component (e.g., "rust", "python")
    pub language_component: String,

    /// Extracted entity type (e.g., "fn", "struct", "method")
    pub entity_type_component: String,

    /// Extracted name component
    pub name_component: String,

    /// Extracted path hash component
    pub path_hash_component: String,

    /// Parsed line range, None for external entities
    pub line_range: Option<LineRangeData>,
}

/// Line range extracted from entity key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineRangeData {
    pub start_line: u32,
    pub end_line: u32,
}
```

### 2.2 EntityChangeType Enum

Classification of entity changes between base and live states.

```rust
/// Classification of how an entity changed between base and live states.
///
/// # Ordering (by severity/importance)
/// 1. Removed - entity no longer exists
/// 2. Added - new entity appeared
/// 3. Relocated - same stable ID, different file path
/// 4. Modified - same stable ID, content changed (future: hash-based)
/// 5. Moved - same stable ID, different line numbers in same file
/// 6. Unchanged - no change detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityChangeTypeClassification {
    /// Entity exists only in base (deleted in live)
    Removed,

    /// Entity exists only in live (new addition)
    Added,

    /// Same stable ID, moved to different file
    Relocated,

    /// Same stable ID, content changed (hash differs)
    Modified,

    /// Same stable ID, different line numbers in same file
    Moved,

    /// No detectable change
    Unchanged,
}
```

### 2.3 DiffResult Struct

Complete diff result containing all changes.

```rust
/// Complete diff result between base and live entity sets.
///
/// # Structure
/// - `summary`: Quick overview counts
/// - `entity_changes_list`: Detailed per-entity changes
/// - `edge_changes_list`: Dependency edge changes
/// - `affected_neighbors_list`: 1-hop neighbors of changed entities
#[derive(Debug, Clone, Default)]
pub struct DiffResultDataPayload {
    /// Summary statistics for quick overview
    pub summary: DiffSummaryDataPayload,

    /// List of entity changes with full details
    pub entity_changes_list: Vec<EntityChangeDataItem>,

    /// List of edge changes (added/removed dependencies)
    pub edge_changes_list: Vec<EdgeChangeDataItem>,

    /// Entity keys of 1-hop neighbors affected by changes
    pub affected_neighbors_list: Vec<String>,
}

/// Summary statistics for diff result.
#[derive(Debug, Clone, Default)]
pub struct DiffSummaryDataPayload {
    pub entities_added_count: usize,
    pub entities_removed_count: usize,
    pub entities_modified_count: usize,
    pub entities_moved_count: usize,
    pub entities_relocated_count: usize,
    pub edges_added_count: usize,
    pub edges_removed_count: usize,
    pub total_blast_radius: usize,
}

/// Single entity change with before/after details.
#[derive(Debug, Clone)]
pub struct EntityChangeDataItem {
    /// Type of change detected
    pub change_type: EntityChangeTypeClassification,

    /// Stable identity for matching
    pub stable_identity: String,

    /// Entity state in base (None for Added)
    pub base_entity_data: Option<EntityDataPayload>,

    /// Entity state in live (None for Removed)
    pub live_entity_data: Option<EntityDataPayload>,

    /// For Moved: number of lines shifted (positive = down)
    pub lines_shifted_count: Option<i32>,

    /// Human-readable description of change
    pub change_description: String,
}

/// Entity data matching API response format.
/// Mirrors existing pattern from code_entities_list_all_handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDataPayload {
    pub key: String,
    pub file_path: String,
    pub entity_type: String,
    pub entity_class: String,
    pub language: String,
}

/// Edge change classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeChangeTypeClassification {
    Added,
    Removed,
}

/// Single edge change with details.
#[derive(Debug, Clone)]
pub struct EdgeChangeDataItem {
    pub change_type: EdgeChangeTypeClassification,
    pub from_key: String,
    pub to_key: String,
    pub edge_type: String,
    pub source_location: String,
}
```

---

## 3. Trait Definitions

### 3.1 KeyNormalizerTrait

Trait for extracting stable identity from entity keys.

```rust
/// Trait for normalizing entity keys by extracting stable identity.
///
/// # Implementors
/// - `DefaultKeyNormalizerImpl` - production implementation
/// - `MockKeyNormalizerImpl` - test double
///
/// # Contract
/// - MUST extract stable identity without line numbers
/// - MUST handle external entities (0-0 suffix)
/// - MUST be deterministic (same input = same output)
/// - SHOULD be O(1) string operations
pub trait KeyNormalizerTrait: Send + Sync {
    /// Extract stable identity from full entity key.
    ///
    /// # Arguments
    /// * `full_key` - Complete entity key with line numbers
    ///
    /// # Returns
    /// Normalized key data with all components parsed
    ///
    /// # Errors
    /// Returns error if key format is invalid
    fn extract_stable_identity_from_key(
        &self,
        full_key: &str,
    ) -> Result<NormalizedEntityKeyData, KeyNormalizationErrorType>;

    /// Check if two keys refer to the same logical entity.
    ///
    /// # Arguments
    /// * `key_a` - First entity key
    /// * `key_b` - Second entity key
    ///
    /// # Returns
    /// true if stable identities match
    fn check_keys_match_stable_identity(
        &self,
        key_a: &str,
        key_b: &str,
    ) -> bool;

    /// Check if key represents an external entity.
    ///
    /// # Arguments
    /// * `full_key` - Entity key to check
    ///
    /// # Returns
    /// true if key matches external pattern (unknown:0-0)
    fn check_key_is_external_entity(
        &self,
        full_key: &str,
    ) -> bool;
}

/// Errors from key normalization operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyNormalizationErrorType {
    /// Key does not match expected format
    InvalidKeyFormatError { key: String, reason: String },

    /// Line range could not be parsed
    InvalidLineRangeError { suffix: String },

    /// Key is empty or whitespace-only
    EmptyKeyError,
}
```

### 3.2 EntityDifferTrait

Trait for computing diffs between entity sets.

```rust
/// Trait for computing diffs between base and live entity sets.
///
/// # Implementors
/// - `DefaultEntityDifferImpl` - production implementation
/// - `MockEntityDifferImpl` - test double
///
/// # Contract
/// - MUST use stable identity matching (not full key comparison)
/// - MUST classify all entities in both sets
/// - MUST calculate blast radius (1-hop neighbors)
/// - SHOULD be O(n) where n = total entities in both sets
pub trait EntityDifferTrait: Send + Sync {
    /// Compute diff between base and live entity sets.
    ///
    /// # Arguments
    /// * `base_entities` - Entities from base snapshot
    /// * `live_entities` - Entities from live/current state
    ///
    /// # Returns
    /// Complete diff result with all changes classified
    fn compute_entity_diff_result(
        &self,
        base_entities: &[EntityDataPayload],
        live_entities: &[EntityDataPayload],
    ) -> DiffResultDataPayload;

    /// Classify change type for a single entity.
    ///
    /// # Arguments
    /// * `base_entity` - Entity in base (None if not present)
    /// * `live_entity` - Entity in live (None if not present)
    ///
    /// # Returns
    /// Classification of change type
    fn classify_single_entity_change(
        &self,
        base_entity: Option<&EntityDataPayload>,
        live_entity: Option<&EntityDataPayload>,
    ) -> EntityChangeTypeClassification;

    /// Calculate lines shifted between base and live positions.
    ///
    /// # Arguments
    /// * `base_key` - Full key from base snapshot
    /// * `live_key` - Full key from live state
    ///
    /// # Returns
    /// Number of lines shifted (positive = moved down, negative = moved up)
    /// None if either key lacks valid line numbers
    fn calculate_lines_shifted_count(
        &self,
        base_key: &str,
        live_key: &str,
    ) -> Option<i32>;
}
```

### 3.3 BlastRadiusCalculatorTrait

Trait for calculating affected neighbors (blast radius).

```rust
/// Trait for calculating blast radius of changed entities.
///
/// # Contract
/// - MUST return 1-hop neighbors of changed entities
/// - MUST not include the changed entities themselves in neighbors
/// - MUST handle cycles without infinite loops
/// - SHOULD use existing dependency graph data
pub trait BlastRadiusCalculatorTrait: Send + Sync {
    /// Calculate 1-hop neighbors of changed entities.
    ///
    /// # Arguments
    /// * `changed_entity_keys` - Keys of entities that changed
    /// * `dependency_edges` - All dependency edges in graph
    ///
    /// # Returns
    /// Set of entity keys that are 1-hop neighbors
    fn calculate_affected_neighbors_set(
        &self,
        changed_entity_keys: &[String],
        dependency_edges: &[EdgeDataPayload],
    ) -> Vec<String>;
}

/// Edge data matching API response format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeDataPayload {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: String,
    pub source_location: String,
}
```

### 3.4 DiffVisualizationTransformerTrait

Trait for transforming diff results to visualization format.

```rust
/// Trait for transforming diff results into visualization-ready format.
///
/// # Contract
/// - MUST assign visual status to all nodes
/// - MUST preserve entity relationships
/// - MUST support status: added, removed, modified, moved, neighbor, ambient
pub trait DiffVisualizationTransformerTrait: Send + Sync {
    /// Transform diff result to visualization node list.
    ///
    /// # Arguments
    /// * `diff_result` - Computed diff result
    /// * `all_entities` - Complete entity list for ambient nodes
    ///
    /// # Returns
    /// List of visualization nodes with status assigned
    fn transform_diff_to_visualization_nodes(
        &self,
        diff_result: &DiffResultDataPayload,
        all_entities: &[EntityDataPayload],
    ) -> Vec<VisualizationNodeDataPayload>;
}

/// Node status for visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeVisualizationStatusType {
    Added,
    Removed,
    Modified,
    Moved,
    Neighbor,
    Ambient,
}

/// Visualization node data.
#[derive(Debug, Clone)]
pub struct VisualizationNodeDataPayload {
    pub key: String,
    pub stable_id: String,
    pub status: NodeVisualizationStatusType,
    pub entity_type: String,
    pub is_external: bool,
}
```

---

## 4. Test Stubs (RED Phase)

All test stubs are marked `#[ignore]` until implementation begins.

```rust
#[cfg(test)]
mod key_normalizer_trait_tests {
    use super::*;

    #[test]
    #[ignore = "RED: Implement extract_stable_identity_from_key"]
    fn test_extract_stable_identity_basic_function() {
        // Given: A full entity key with line numbers
        // rust:fn:main:__crates_src_main_rs:10-50
        // When: extract_stable_identity_from_key is called
        // Then: Returns stable identity without line numbers
        // rust:fn:main:__crates_src_main_rs
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement extract_stable_identity_from_key"]
    fn test_extract_stable_identity_external_entity() {
        // Given: An external entity key with 0-0 suffix
        // rust:fn:map:unknown:0-0
        // When: extract_stable_identity_from_key is called
        // Then: Returns stable identity and marks as external
        // rust:fn:map:unknown, is_external=true
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement extract_stable_identity_from_key"]
    fn test_extract_stable_identity_method_key() {
        // Given: A method entity key
        // rust:method:entity_to_params:__crates_parseltongue-core_src_storage_cozo_client_rs:948-1102
        // When: extract_stable_identity_from_key is called
        // Then: Correctly parses all components
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement extract_stable_identity_from_key"]
    fn test_extract_stable_identity_invalid_format() {
        // Given: A malformed entity key
        // When: extract_stable_identity_from_key is called
        // Then: Returns InvalidKeyFormatError
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement check_keys_match_stable_identity"]
    fn test_check_keys_match_same_entity_different_lines() {
        // Given: Two keys with same stable identity but different lines
        // rust:fn:main:path:10-50 and rust:fn:main:path:15-55
        // When: check_keys_match_stable_identity is called
        // Then: Returns true
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement check_keys_match_stable_identity"]
    fn test_check_keys_match_different_entities() {
        // Given: Two keys with different stable identities
        // rust:fn:main:path:10-50 and rust:fn:helper:path:10-50
        // When: check_keys_match_stable_identity is called
        // Then: Returns false
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement check_key_is_external_entity"]
    fn test_check_key_is_external_entity_true() {
        // Given: An external entity key
        // rust:fn:map:unknown:0-0
        // When: check_key_is_external_entity is called
        // Then: Returns true
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement check_key_is_external_entity"]
    fn test_check_key_is_external_entity_false() {
        // Given: A local entity key
        // rust:fn:main:__crates_src_main_rs:10-50
        // When: check_key_is_external_entity is called
        // Then: Returns false
        todo!("Implement test")
    }
}

#[cfg(test)]
mod entity_differ_trait_tests {
    use super::*;

    #[test]
    #[ignore = "RED: Implement compute_entity_diff_result"]
    fn test_compute_diff_empty_both_sets() {
        // Given: Empty base and live entity sets
        // When: compute_entity_diff_result is called
        // Then: Returns empty diff with zero counts
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement compute_entity_diff_result"]
    fn test_compute_diff_added_entity() {
        // Given: Empty base, one entity in live
        // When: compute_entity_diff_result is called
        // Then: Returns diff with one Added entity
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement compute_entity_diff_result"]
    fn test_compute_diff_removed_entity() {
        // Given: One entity in base, empty live
        // When: compute_entity_diff_result is called
        // Then: Returns diff with one Removed entity
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement compute_entity_diff_result"]
    fn test_compute_diff_moved_entity() {
        // Given: Same entity in base and live, different line numbers
        // rust:fn:main:path:10-50 -> rust:fn:main:path:15-55
        // When: compute_entity_diff_result is called
        // Then: Returns diff with one Moved entity, lines_shifted=5
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement compute_entity_diff_result"]
    fn test_compute_diff_unchanged_entity() {
        // Given: Same entity with identical key in base and live
        // When: compute_entity_diff_result is called
        // Then: Returns empty diff (unchanged not reported)
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement compute_entity_diff_result"]
    fn test_compute_diff_mixed_changes() {
        // Given: Base with [A, B, C], Live with [B', D]
        // Where B' is B moved 5 lines
        // When: compute_entity_diff_result is called
        // Then: Returns A=Removed, B=Moved, C=Removed, D=Added
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement classify_single_entity_change"]
    fn test_classify_added() {
        // Given: None in base, Some in live
        // When: classify_single_entity_change is called
        // Then: Returns Added
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement classify_single_entity_change"]
    fn test_classify_removed() {
        // Given: Some in base, None in live
        // When: classify_single_entity_change is called
        // Then: Returns Removed
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement classify_single_entity_change"]
    fn test_classify_moved_same_file() {
        // Given: Same stable ID, same file, different lines
        // When: classify_single_entity_change is called
        // Then: Returns Moved
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement classify_single_entity_change"]
    fn test_classify_relocated_different_file() {
        // Given: Same name, different file path
        // When: classify_single_entity_change is called
        // Then: Returns Relocated
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement calculate_lines_shifted_count"]
    fn test_calculate_lines_shifted_positive() {
        // Given: Entity moved down 5 lines
        // rust:fn:main:path:10-50 -> rust:fn:main:path:15-55
        // When: calculate_lines_shifted_count is called
        // Then: Returns Some(5)
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement calculate_lines_shifted_count"]
    fn test_calculate_lines_shifted_negative() {
        // Given: Entity moved up 3 lines
        // rust:fn:main:path:20-30 -> rust:fn:main:path:17-27
        // When: calculate_lines_shifted_count is called
        // Then: Returns Some(-3)
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement calculate_lines_shifted_count"]
    fn test_calculate_lines_shifted_external_entity() {
        // Given: External entity key
        // rust:fn:map:unknown:0-0
        // When: calculate_lines_shifted_count is called
        // Then: Returns None
        todo!("Implement test")
    }
}

#[cfg(test)]
mod blast_radius_calculator_tests {
    use super::*;

    #[test]
    #[ignore = "RED: Implement calculate_affected_neighbors_set"]
    fn test_calculate_neighbors_empty_changes() {
        // Given: No changed entities
        // When: calculate_affected_neighbors_set is called
        // Then: Returns empty set
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement calculate_affected_neighbors_set"]
    fn test_calculate_neighbors_single_hop() {
        // Given: Entity A changed, A -> B edge exists
        // When: calculate_affected_neighbors_set is called
        // Then: Returns [B]
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement calculate_affected_neighbors_set"]
    fn test_calculate_neighbors_reverse_edge() {
        // Given: Entity A changed, B -> A edge exists
        // When: calculate_affected_neighbors_set is called
        // Then: Returns [B] (callers are neighbors too)
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement calculate_affected_neighbors_set"]
    fn test_calculate_neighbors_excludes_changed() {
        // Given: Entities A, B changed, A -> B edge exists
        // When: calculate_affected_neighbors_set is called
        // Then: Returns empty (B is changed, not a neighbor)
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement calculate_affected_neighbors_set"]
    fn test_calculate_neighbors_deduplicates() {
        // Given: A changed, A -> C and B changed, B -> C
        // When: calculate_affected_neighbors_set is called
        // Then: Returns [C] (not [C, C])
        todo!("Implement test")
    }
}

#[cfg(test)]
mod diff_visualization_transformer_tests {
    use super::*;

    #[test]
    #[ignore = "RED: Implement transform_diff_to_visualization_nodes"]
    fn test_transform_added_entity_status() {
        // Given: Diff with one Added entity
        // When: transform_diff_to_visualization_nodes is called
        // Then: Returns node with status=Added
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement transform_diff_to_visualization_nodes"]
    fn test_transform_neighbor_entity_status() {
        // Given: Diff with affected_neighbors containing entity X
        // When: transform_diff_to_visualization_nodes is called
        // Then: Entity X has status=Neighbor
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement transform_diff_to_visualization_nodes"]
    fn test_transform_ambient_entity_status() {
        // Given: Entity not in diff and not in neighbors
        // When: transform_diff_to_visualization_nodes is called
        // Then: Entity has status=Ambient
        todo!("Implement test")
    }

    #[test]
    #[ignore = "RED: Implement transform_diff_to_visualization_nodes"]
    fn test_transform_external_entity_marked() {
        // Given: External entity (unknown:0-0 pattern)
        // When: transform_diff_to_visualization_nodes is called
        // Then: Node has is_external=true
        todo!("Implement test")
    }
}

#[cfg(test)]
mod performance_contract_tests {
    use super::*;

    #[test]
    #[ignore = "RED: Performance contract - key normalization"]
    fn test_key_normalization_performance_contract() {
        // Given: 10,000 entity keys
        // When: extract_stable_identity_from_key called on all
        // Then: Completes in < 100ms (10us per key)
        todo!("Implement performance test")
    }

    #[test]
    #[ignore = "RED: Performance contract - diff computation"]
    fn test_diff_computation_performance_contract() {
        // Given: 1,000 entities in base, 1,000 in live
        // When: compute_entity_diff_result is called
        // Then: Completes in < 500ms
        todo!("Implement performance test")
    }

    #[test]
    #[ignore = "RED: Performance contract - blast radius"]
    fn test_blast_radius_performance_contract() {
        // Given: 100 changed entities, 10,000 edges
        // When: calculate_affected_neighbors_set is called
        // Then: Completes in < 200ms
        todo!("Implement performance test")
    }
}
```

---

## 5. Implementation Notes

### 5.1 Suggested Module Organization

```
crates/parseltongue-core/src/
  diff/
    mod.rs                          # Module exports
    key_normalizer_impl.rs          # KeyNormalizerTrait impl
    entity_differ_impl.rs           # EntityDifferTrait impl
    blast_radius_calculator_impl.rs # BlastRadiusCalculatorTrait impl
    visualization_transformer_impl.rs # DiffVisualizationTransformerTrait impl
    types.rs                        # All struct/enum definitions
```

### 5.2 Dependency on Existing Code

From API discovery, these existing components will be useful:
- `CozoDbStorage::get_all_entities` - fetch entity lists
- `CozoDbStorage::get_all_dependencies` - fetch edge lists
- `extract_file_path_from_entity_key` pattern - key parsing
- `sanitize_path_for_key_format` - path handling

### 5.3 Error Handling Strategy

Following CLAUDE.md conventions:
- Use `thiserror` for library errors (KeyNormalizationErrorType)
- Traits return Result where operations can fail
- Diff computation should not panic on malformed data

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-23 | Initial creation with trait/struct definitions and test stubs |

---

*This document follows TDD RED phase - all tests are stubs awaiting implementation.*
