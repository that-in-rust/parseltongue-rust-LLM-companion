// ISGL1 v2 Cycle 3: Entity Matching Algorithm Tests
//
// These tests verify the 3-priority matching algorithm used during reindexing:
// Priority 1: Content hash match (code unchanged → keep old key)
// Priority 2: Position match (name + file + ~same line → keep old key)
// Priority 3: No match (new entity → assign new birth timestamp)

use parseltongue_core::entities::EntityType;
use parseltongue_core::isgl1_v2::{
    match_entity_with_old_index, EntityCandidate, OldEntity, EntityMatchResult
};

/// Test 1: Priority 1 - Hash match (entity moved, code unchanged)
#[test]
fn test_match_by_content_hash_priority1() {
    let new_entity = EntityCandidate {
        name: "calculate".to_string(),
        entity_type: EntityType::Function,
        file_path: "src/math.rs".to_string(),
        line_range: (50, 60),  // Moved from line 10 to line 50
        content_hash: "abc123".to_string(),
        code: "fn calculate() { }".to_string(),
    };

    let old_entities = vec![OldEntity {
        key: "rust:fn:calculate:__src_math_rs:T1706284800".to_string(),
        name: "calculate".to_string(),
        file_path: "src/math.rs".to_string(),
        line_range: (10, 20),  // Old position
        content_hash: "abc123".to_string(),  // SAME HASH
    }];

    let result = match_entity_with_old_index(&new_entity, &old_entities);

    assert_eq!(result, EntityMatchResult::ContentMatch {
        old_key: "rust:fn:calculate:__src_math_rs:T1706284800".to_string()
    });
}

/// Test 2: Priority 2 - Position match (code changed, position similar)
#[test]
fn test_match_by_position_when_hash_differs_priority2() {
    let new_entity = EntityCandidate {
        name: "process".to_string(),
        entity_type: EntityType::Function,
        file_path: "src/data.rs".to_string(),
        line_range: (25, 35),  // Within tolerance of old position (20)
        content_hash: "new_hash_456".to_string(),  // Changed code
        code: "fn process() { /* changed */ }".to_string(),
    };

    let old_entities = vec![OldEntity {
        key: "rust:fn:process:__src_data_rs:T1706284900".to_string(),
        name: "process".to_string(),
        file_path: "src/data.rs".to_string(),
        line_range: (20, 30),
        content_hash: "old_hash_789".to_string(),  // DIFFERENT HASH
    }];

    let result = match_entity_with_old_index(&new_entity, &old_entities);

    assert_eq!(result, EntityMatchResult::PositionMatch {
        old_key: "rust:fn:process:__src_data_rs:T1706284900".to_string()
    });
}

/// Test 3: Priority 3 - New entity (not in old index)
#[test]
fn test_new_entity_detection_priority3() {
    let new_entity = EntityCandidate {
        name: "brand_new_function".to_string(),
        entity_type: EntityType::Function,
        file_path: "src/new.rs".to_string(),
        line_range: (10, 20),
        content_hash: "xyz789".to_string(),
        code: "fn brand_new_function() { }".to_string(),
    };

    let old_entities = vec![]; // Empty index

    let result = match_entity_with_old_index(&new_entity, &old_entities);

    assert_eq!(result, EntityMatchResult::NewEntity);
}

/// Test 4: Position tolerance (within ±10 lines)
#[test]
fn test_position_match_within_tolerance() {
    let new_entity = EntityCandidate {
        name: "helper".to_string(),
        entity_type: EntityType::Function,
        file_path: "src/util.rs".to_string(),
        line_range: (108, 118),  // Old: 100, diff = 8 (within ±10)
        content_hash: "changed".to_string(),
        code: "fn helper() { /* changed */ }".to_string(),
    };

    let old_entities = vec![OldEntity {
        key: "rust:fn:helper:__src_util_rs:T1706285000".to_string(),
        name: "helper".to_string(),
        file_path: "src/util.rs".to_string(),
        line_range: (100, 110),
        content_hash: "original".to_string(),
    }];

    let result = match_entity_with_old_index(&new_entity, &old_entities);

    // Within tolerance → Position match
    assert_eq!(result, EntityMatchResult::PositionMatch {
        old_key: "rust:fn:helper:__src_util_rs:T1706285000".to_string()
    });
}

/// Test 5: Multiple candidates - hash takes precedence over position
#[test]
fn test_hash_priority_over_position() {
    let new_entity = EntityCandidate {
        name: "shared_name".to_string(),
        entity_type: EntityType::Function,
        file_path: "src/lib.rs".to_string(),
        line_range: (50, 60),
        content_hash: "match_this".to_string(),
        code: "fn shared_name() { }".to_string(),
    };

    let old_entities = vec![
        // Candidate 1: Close position, wrong hash
        OldEntity {
            key: "rust:fn:shared_name:__src_lib_rs:T1706285100".to_string(),
            name: "shared_name".to_string(),
            file_path: "src/lib.rs".to_string(),
            line_range: (48, 58),  // Closer position
            content_hash: "wrong_hash".to_string(),
        },
        // Candidate 2: Far position, CORRECT hash
        OldEntity {
            key: "rust:fn:shared_name:__src_lib_rs:T1706285200".to_string(),
            name: "shared_name".to_string(),
            file_path: "src/lib.rs".to_string(),
            line_range: (200, 210),  // Far away
            content_hash: "match_this".to_string(),  // MATCH!
        },
    ];

    let result = match_entity_with_old_index(&new_entity, &old_entities);

    // Hash match takes precedence over position proximity
    assert_eq!(result, EntityMatchResult::ContentMatch {
        old_key: "rust:fn:shared_name:__src_lib_rs:T1706285200".to_string()
    });
}
