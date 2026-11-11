use parseltongue_core::entities::EntityType;
use parseltongue_core::isgl1_v2::{
    format_key_v2, compute_content_hash, compute_birth_timestamp,
    extract_semantic_path, match_entity_with_old_index,
    EntityCandidate, OldEntity, EntityMatchResult,
};

/// Test 1: Initial indexing creates v2 keys
#[test]
fn test_initial_indexing_creates_v2_keys() {
    // Simulate 3 functions being indexed for first time
    let functions = vec![
        ("calculate", "fn calculate() -> i32 { 42 }", 10, 12),
        ("process", "fn process(x: i32) { println!(\"{}\", x); }", 20, 22),
        ("helper", "fn helper() {}", 30, 32),
    ];

    for (name, code, start, end) in functions {
        let file_path = "src/math.rs";
        let semantic_path = extract_semantic_path(file_path);
        let birth_ts = compute_birth_timestamp(file_path, name);
        let content_hash = compute_content_hash(code);
        let key = format_key_v2(EntityType::Function, name, "rust", &semantic_path, birth_ts);

        // Verify v2 key format
        assert!(key.starts_with("rust:fn:"));
        assert!(key.contains(&format!(":{}", name)));
        assert!(key.contains(":T"));
        assert!(!key.contains(&format!("{}-{}", start, end))); // No line numbers

        // Verify content hash
        assert_eq!(content_hash.len(), 64);
    }
}

/// Test 2: Reindex unchanged file → no false positives
#[test]
fn test_reindex_unchanged_file_no_false_positives() {
    let file_path = "src/lib.rs";
    let code = "fn main() { println!(\"Hello\"); }";

    // First index
    let name = "main";
    let semantic_path = extract_semantic_path(file_path);
    let birth_ts = compute_birth_timestamp(file_path, name);
    let content_hash = compute_content_hash(code);
    let key = format_key_v2(EntityType::Function, name, "rust", &semantic_path, birth_ts);

    let old_entities = vec![OldEntity {
        key: key.clone(),
        name: name.to_string(),
        file_path: file_path.to_string(),
        line_range: (10, 12),
        content_hash: content_hash.clone(),
    }];

    // Reindex (code unchanged)
    let new_entity = EntityCandidate {
        name: name.to_string(),
        entity_type: EntityType::Function,
        file_path: file_path.to_string(),
        line_range: (10, 12),
        content_hash: content_hash.clone(),
        code: code.to_string(),
    };

    let result = match_entity_with_old_index(&new_entity, &old_entities);

    // Should match on content hash
    assert_eq!(result, EntityMatchResult::ContentMatch { old_key: key });
}

/// THE KEY TEST: Test 3 - Add 100 lines at top → zero false positives
#[test]
fn test_add_100_lines_at_top_zero_false_positives() {
    let file_path = "src/core.rs";

    // Three functions in original file
    let functions = vec![
        ("alpha", "fn alpha() { let x = 1; }", 10, 12),
        ("beta", "fn beta() { let y = 2; }", 30, 32),
        ("gamma", "fn gamma() { let z = 3; }", 50, 52),
    ];

    // First index (original positions)
    let mut old_entities = vec![];
    for (name, code, start, end) in &functions {
        let semantic_path = extract_semantic_path(file_path);
        let birth_ts = compute_birth_timestamp(file_path, name);
        let content_hash = compute_content_hash(code);
        let key = format_key_v2(EntityType::Function, name, "rust", &semantic_path, birth_ts);

        old_entities.push(OldEntity {
            key,
            name: name.to_string(),
            file_path: file_path.to_string(),
            line_range: (*start, *end),
            content_hash,
        });
    }

    // Simulate adding 100 blank lines at top
    // Functions move from lines 10,30,50 → 110,130,150
    // But content_hash UNCHANGED!

    let new_functions = vec![
        ("alpha", "fn alpha() { let x = 1; }", 110, 112),  // Moved +100
        ("beta", "fn beta() { let y = 2; }", 130, 132),    // Moved +100
        ("gamma", "fn gamma() { let z = 3; }", 150, 152),  // Moved +100
    ];

    let mut match_results = vec![];
    for (name, code, new_start, new_end) in new_functions {
        let content_hash = compute_content_hash(code);
        let new_entity = EntityCandidate {
            name: name.to_string(),
            entity_type: EntityType::Function,
            file_path: file_path.to_string(),
            line_range: (new_start, new_end),
            content_hash: content_hash.clone(),
            code: code.to_string(),
        };

        let result = match_entity_with_old_index(&new_entity, &old_entities);
        match_results.push((name, result));
    }

    // THE KEY ASSERTION: All 3 functions should be ContentMatch
    // Zero false positives despite moving 100 lines!
    for (name, result) in match_results {
        match result {
            EntityMatchResult::ContentMatch { .. } => {
                // SUCCESS - Hash matched despite line number change
            }
            _ => panic!("FAILURE: {} should be ContentMatch, got {:?}", name, result),
        }
    }
}

/// Test 4: Modify one function → precise detection
#[test]
fn test_modify_one_function_precise_detection() {
    let file_path = "src/app.rs";

    // Three functions
    let original_functions = vec![
        ("func1", "fn func1() { 1 }", 10, 12),
        ("func2", "fn func2() { 2 }", 20, 22),
        ("func3", "fn func3() { 3 }", 30, 32),
    ];

    // First index
    let mut old_entities = vec![];
    for (name, code, start, end) in &original_functions {
        let semantic_path = extract_semantic_path(file_path);
        let birth_ts = compute_birth_timestamp(file_path, name);
        let content_hash = compute_content_hash(code);
        let key = format_key_v2(EntityType::Function, name, "rust", &semantic_path, birth_ts);

        old_entities.push(OldEntity {
            key,
            name: name.to_string(),
            file_path: file_path.to_string(),
            line_range: (*start, *end),
            content_hash,
        });
    }

    // Modify ONLY func2 (change code)
    let modified_functions = vec![
        ("func1", "fn func1() { 1 }", 10, 12),          // Unchanged
        ("func2", "fn func2() { 999 }", 20, 22),        // CHANGED
        ("func3", "fn func3() { 3 }", 30, 32),          // Unchanged
    ];

    let mut results = vec![];
    for (name, code, start, end) in modified_functions {
        let content_hash = compute_content_hash(code);
        let new_entity = EntityCandidate {
            name: name.to_string(),
            entity_type: EntityType::Function,
            file_path: file_path.to_string(),
            line_range: (start, end),
            content_hash: content_hash.clone(),
            code: code.to_string(),
        };

        let result = match_entity_with_old_index(&new_entity, &old_entities);
        results.push((name, result));
    }

    // Verify: 2 ContentMatch (unchanged) + 1 PositionMatch (changed)
    assert!(matches!(results[0].1, EntityMatchResult::ContentMatch { .. })); // func1
    assert!(matches!(results[1].1, EntityMatchResult::PositionMatch { .. })); // func2 (changed!)
    assert!(matches!(results[2].1, EntityMatchResult::ContentMatch { .. })); // func3
}

/// Test 5: Performance benchmark
#[test]
fn test_performance_large_codebase() {
    use std::time::Instant;

    let file_path = "src/large.rs";
    let num_functions = 100;

    // Create 100 functions
    let start_time = Instant::now();

    for i in 0..num_functions {
        let name = format!("func_{}", i);
        let code = format!("fn {}() {{ {} }}", name, i);
        let semantic_path = extract_semantic_path(file_path);
        let birth_ts = compute_birth_timestamp(file_path, &name);
        let _content_hash = compute_content_hash(&code);
        let _key = format_key_v2(EntityType::Function, &name, "rust", &semantic_path, birth_ts);
    }

    let duration = start_time.elapsed();

    // Should be fast: <500ms for 100 entities
    assert!(duration.as_millis() < 500, "Too slow: {}ms", duration.as_millis());
}
