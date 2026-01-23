# ADR-001: Entity Key Normalization for Diff Stability

> **Status**: Proposed
> **Date**: 2026-01-22
> **Decision**: Use stable identity extraction for entity matching

---

## Context

Parseltongue entity keys encode line numbers:
```
rust:fn:handle_auth:__crates_path_src_auth_rs:10-50
                                              ^^^^^^
                                              Line numbers
```

When code is added or removed ABOVE a function, all line numbers below shift:
```
Before: rust:fn:helper:__path:20-30
After:  rust:fn:helper:__path:24-34   // 4 lines added above
```

The current diff algorithm compares keys directly:
- `20-30` key not found in live -> marked as REMOVED
- `24-34` key not found in base -> marked as ADDED

**Result**: One unchanged function appears as both REMOVED and ADDED.

At scale, adding a single comment at line 1 could make ALL entities appear changed.

---

## Decision

Implement **two-phase key matching**:

### Phase 1: Extract Stable Identity

```
Full Key:      rust:fn:handle_auth:__crates_path_src_auth_rs:10-50
Stable ID:     rust:fn:handle_auth:__crates_path_src_auth_rs
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
               Everything except line numbers
```

Algorithm:
```rust
fn extract_stable_identity(key: &str) -> &str {
    // Entity keys end with :start-end
    // Find the last colon before line numbers
    if let Some(last_colon) = key.rfind(':') {
        if let Some(second_last_colon) = key[..last_colon].rfind(':') {
            // Verify the suffix is line numbers (digits and hyphen)
            let suffix = &key[second_last_colon + 1..];
            if suffix.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ':') {
                return &key[..second_last_colon];
            }
        }
    }
    // Fallback: return full key if pattern doesn't match
    key
}
```

### Phase 2: Match and Classify

```rust
#[derive(Debug, PartialEq)]
enum EntityChangeType {
    Unchanged,           // Same key exactly
    Moved,               // Same stable ID, different lines, same file
    Relocated,           // Same stable ID, different file
    Modified,            // Same stable ID, content changed (if hash available)
    Added,               // Stable ID only in live
    Removed,             // Stable ID only in base
}

fn classify_entity_change(
    base_entities: &HashMap<String, Entity>,  // keyed by stable ID
    live_entities: &HashMap<String, Entity>,
    stable_id: &str
) -> EntityChangeType {
    match (base_entities.get(stable_id), live_entities.get(stable_id)) {
        (None, Some(_)) => EntityChangeType::Added,
        (Some(_), None) => EntityChangeType::Removed,
        (Some(base), Some(live)) => {
            if base.key == live.key {
                EntityChangeType::Unchanged
            } else if base.file_path != live.file_path {
                EntityChangeType::Relocated
            } else {
                // Same file, different lines
                EntityChangeType::Moved
            }
        }
        (None, None) => unreachable!(),
    }
}
```

---

## Consequences

### Positive

1. **Accurate diffs**: Adding blank lines won't show false changes
2. **Rename detection foundation**: Stable ID enables future rename detection
3. **Reduced noise**: Users see actual changes, not line-number churn
4. **Trust**: Users will trust the tool because it shows meaningful changes

### Negative

1. **Collision risk**: Two functions with same name in different spans
   - Mitigation: Include path_hash in stable ID (already done)
   - Risk: Function split/merge still ambiguous

2. **Complexity**: Two-phase matching is more complex than direct key comparison
   - Mitigation: Well-tested, isolated module

3. **Performance**: Extra parsing step per entity
   - Mitigation: O(1) string operation, negligible at 215 entities

---

## Implementation

### Data Structures

```rust
/// Entity with both full key and stable identity
pub struct NormalizedEntity {
    /// Full key with line numbers: rust:fn:main:path:10-50
    pub full_key: String,

    /// Stable identity without lines: rust:fn:main:path
    pub stable_id: String,

    /// Original entity data
    pub entity: Entity,
}

impl NormalizedEntity {
    pub fn from_entity(entity: Entity) -> Self {
        let stable_id = extract_stable_identity(&entity.key).to_string();
        Self {
            full_key: entity.key.clone(),
            stable_id,
            entity,
        }
    }
}
```

### Diff Algorithm

```rust
pub fn compute_entity_diff(
    base_entities: Vec<Entity>,
    live_entities: Vec<Entity>,
) -> EntityDiff {
    // Normalize all entities
    let base_normalized: HashMap<String, NormalizedEntity> = base_entities
        .into_iter()
        .map(NormalizedEntity::from_entity)
        .map(|e| (e.stable_id.clone(), e))
        .collect();

    let live_normalized: HashMap<String, NormalizedEntity> = live_entities
        .into_iter()
        .map(NormalizedEntity::from_entity)
        .map(|e| (e.stable_id.clone(), e))
        .collect();

    let all_ids: HashSet<_> = base_normalized.keys()
        .chain(live_normalized.keys())
        .collect();

    let mut diff = EntityDiff::default();

    for stable_id in all_ids {
        match classify_entity_change(&base_normalized, &live_normalized, stable_id) {
            EntityChangeType::Added => {
                diff.added.push(live_normalized[stable_id].entity.clone());
            }
            EntityChangeType::Removed => {
                diff.removed.push(base_normalized[stable_id].entity.clone());
            }
            EntityChangeType::Moved => {
                diff.moved.push(MovedEntity {
                    base: base_normalized[stable_id].entity.clone(),
                    live: live_normalized[stable_id].entity.clone(),
                });
            }
            EntityChangeType::Relocated => {
                diff.relocated.push(RelocatedEntity {
                    base: base_normalized[stable_id].entity.clone(),
                    live: live_normalized[stable_id].entity.clone(),
                });
            }
            EntityChangeType::Unchanged => {
                // Don't include in diff
            }
            EntityChangeType::Modified => {
                diff.modified.push(ModifiedEntity {
                    base: base_normalized[stable_id].entity.clone(),
                    live: live_normalized[stable_id].entity.clone(),
                });
            }
        }
    }

    diff
}
```

---

## Test Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_identity_extraction() {
        let key = "rust:fn:main:__crates_src_main_rs:10-50";
        assert_eq!(
            extract_stable_identity(key),
            "rust:fn:main:__crates_src_main_rs"
        );
    }

    #[test]
    fn test_external_entity_stable_id() {
        // External entities have 0-0 which should still be stripped
        let key = "rust:fn:map:unknown:0-0";
        assert_eq!(
            extract_stable_identity(key),
            "rust:fn:map:unknown"
        );
    }

    #[test]
    fn test_moved_entity_detected() {
        let base = vec![entity("rust:fn:main:path:10-50")];
        let live = vec![entity("rust:fn:main:path:15-55")];  // Shifted 5 lines

        let diff = compute_entity_diff(base, live);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.moved.len(), 1);
    }

    #[test]
    fn test_actually_added_entity() {
        let base = vec![];
        let live = vec![entity("rust:fn:new_func:path:10-20")];

        let diff = compute_entity_diff(base, live);

        assert_eq!(diff.added.len(), 1);
        assert!(diff.removed.is_empty());
        assert!(diff.moved.is_empty());
    }

    #[test]
    fn test_actually_removed_entity() {
        let base = vec![entity("rust:fn:old_func:path:10-20")];
        let live = vec![];

        let diff = compute_entity_diff(base, live);

        assert!(diff.added.is_empty());
        assert_eq!(diff.removed.len(), 1);
        assert!(diff.moved.is_empty());
    }

    #[test]
    fn test_relocated_entity() {
        let base = vec![entity_with_path("rust:fn:helper:__old_path:10-20", "./old/file.rs")];
        let live = vec![entity_with_path("rust:fn:helper:__new_path:10-20", "./new/file.rs")];

        let diff = compute_entity_diff(base, live);

        // This is tricky: stable ID includes path_hash, so different paths = different IDs
        // Actually: old_path vs new_path means different stable IDs
        // So this appears as added + removed, not relocated
        // DECISION: Relocate detection requires same name, different path_hash
        // For MVP: Treat as added + removed (can't reliably detect)
    }

    #[test]
    fn test_unchanged_entity_not_in_diff() {
        let base = vec![entity("rust:fn:stable:path:10-20")];
        let live = vec![entity("rust:fn:stable:path:10-20")];

        let diff = compute_entity_diff(base, live);

        assert!(diff.is_empty());
    }
}

// Test helpers
fn entity(key: &str) -> Entity {
    Entity {
        key: key.to_string(),
        file_path: "./test/file.rs".to_string(),
        entity_type: "fn".to_string(),
        entity_class: "CODE".to_string(),
        language: "rust".to_string(),
    }
}

fn entity_with_path(key: &str, path: &str) -> Entity {
    Entity {
        key: key.to_string(),
        file_path: path.to_string(),
        entity_type: "fn".to_string(),
        entity_class: "CODE".to_string(),
        language: "rust".to_string(),
    }
}
```

---

## Visualization Impact

The diff result feeds into Three.js visualization. Update node status to include new types:

```typescript
type NodeStatus =
  | 'added'      // Green, pulse, [+] label
  | 'removed'    // Red, fade out, [-] label
  | 'modified'   // Yellow, subtle pulse, [~] label
  | 'moved'      // Blue, arrow animation, [->] label  // NEW
  | 'relocated'  // Purple, path change indicator      // NEW
  | 'neighbor'   // Orange, medium brightness
  | 'ambient'    // Gray, fog-like

const STATUS_COLORS: Record<NodeStatus, string> = {
  added:     '#00ff88',
  removed:   '#ff4444',
  modified:  '#ffcc00',
  moved:     '#4a9eff',  // Blue for same file, different lines
  relocated: '#da77f2',  // Purple for different file
  neighbor:  '#ffa94d',
  ambient:   '#888888',
};
```

---

## Related ADRs

- ADR-002 (future): Content hashing for modification detection
- ADR-003 (future): Rename detection heuristics

---

*ADR-001 authored: 2026-01-22*
