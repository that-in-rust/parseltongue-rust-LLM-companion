//! Tool 1 Output Verification Test
//!
//! Validates that Tool 1 (folder-to-cozoDB-streamer) produces correct output
//! when indexing the parseltongue codebase itself.

use parseltongue_core::storage::CozoDbStorage;

#[tokio::test]
#[ignore] // Run manually with: cargo test --package parseltongue-core tool1_verification -- --ignored --nocapture
async fn verify_tool1_parseltongue_indexing() {
    let storage = CozoDbStorage::new("rocksdb:/tmp/parseltongue-rigorous-test.db")
        .await
        .expect("Failed to connect to test database");

    // Get all entities
    let entities = storage.get_all_entities().await.expect("Failed to get entities");

    println!("\n=== TOOL 1 DATA VERIFICATION ===\n");
    println!("Total Entities: {}", entities.len());
    assert!(entities.len() > 500, "Expected >500 entities from parseltongue codebase, got {}", entities.len());

    // Sample ISGL1 keys
    println!("\n--- Sample ISGL1 Keys (Line-Based Format Check) ---");
    for (i, entity) in entities.iter().take(10).enumerate() {
        println!("{}. {}", i+1, entity.isgl1_key);
        println!("   Type: {:?}", entity.interface_signature.entity_type);
        println!("   Name: {}", entity.interface_signature.name);
        println!("   File: {}", entity.interface_signature.file_path.display());
        println!("   Lines: {}-{}",
            entity.interface_signature.line_range.start,
            entity.interface_signature.line_range.end
        );
    }

    // TDD Classification breakdown
    println!("\n--- TDD Classification Breakdown ---");
    let mut test_count = 0;
    let mut code_count = 0;

    for entity in &entities {
        match entity.tdd_classification.entity_class {
            parseltongue_core::entities::EntityClass::TestImplementation => test_count += 1,
            parseltongue_core::entities::EntityClass::CodeImplementation => code_count += 1,
            _ => {}
        }
    }

    println!("TEST_IMPLEMENTATION: {}", test_count);
    println!("CODE_IMPLEMENTATION: {}", code_count);
    println!("Classification Rate: {:.1}%", (test_count + code_count) as f64 / entities.len() as f64 * 100.0);

    assert!(test_count > 0, "Should have some test entities");
    assert!(code_count > 0, "Should have some code entities");

    // Temporal state verification (should all be initial state after Tool 1)
    println!("\n--- Temporal State (should all be current_ind=1, future_ind=0) ---");
    let mut correct_state = 0;
    for entity in &entities {
        if entity.temporal_state.current_ind
            && !entity.temporal_state.future_ind
            && entity.temporal_state.future_action.is_none()
        {
            correct_state += 1;
        }
    }
    println!("Correct Initial State: {}/{} ({:.1}%)",
        correct_state, entities.len(),
        correct_state as f64 / entities.len() as f64 * 100.0
    );

    assert_eq!(correct_state, entities.len(),
        "All entities should have initial temporal state (1,0,None), but {}/{} were correct",
        correct_state, entities.len()
    );

    // Key format validation
    println!("\n--- ISGL1 Key Format Validation ---");
    let mut line_based_count = 0;
    let mut potential_hash_based = 0;

    for entity in &entities {
        let key = &entity.isgl1_key;
        // Line-based format check (very basic heuristic)
        if key.contains(':') && key.contains('-') {
            // Count colons - line-based should have multiple
            let colon_count = key.matches(':').count();
            if colon_count >= 2 {
                line_based_count += 1;
            } else {
                potential_hash_based += 1;
            }
        }
    }

    println!("Likely line-based keys: {} ({:.1}%)",
        line_based_count,
        line_based_count as f64 / entities.len() as f64 * 100.0
    );

    if potential_hash_based > 0 {
        println!("Potential hash-based keys: {} ({:.1}%)",
            potential_hash_based,
            potential_hash_based as f64 / entities.len() as f64 * 100.0
        );
    }

    // Tool 1 should only create line-based keys (hash-based is for Tool 2 Create operations)
    println!("\nExpected: 100% line-based keys (Tool 1 indexes existing code only)");

    // Sample entity detail inspection
    println!("\n--- Detailed Entity Inspection ---");
    if let Some(entity) = entities.first() {
        println!("First Entity:");
        println!("  ISGL1: {}", entity.isgl1_key);
        println!("  Name: {}", entity.interface_signature.name);
        println!("  Type: {:?}", entity.interface_signature.entity_type);
        println!("  Visibility: {:?}", entity.interface_signature.visibility);
        println!("  File: {}", entity.interface_signature.file_path.display());
        println!("  Lines: {:?}", entity.interface_signature.line_range);
        println!("  TDD Class: {:?}", entity.tdd_classification.entity_class);
        println!("  Temporal: (current={}, future={}, action={:?})",
            entity.temporal_state.current_ind,
            entity.temporal_state.future_ind,
            entity.temporal_state.future_action
        );
        println!("  Has Current Code: {}", entity.current_code.is_some());
        println!("  Has Future Code: {}", entity.future_code.is_some());
    }

    // Find test entities
    println!("\n--- Sample Test Entities ---");
    let test_entities: Vec<_> = entities.iter()
        .filter(|e| matches!(e.tdd_classification.entity_class,
            parseltongue_core::entities::EntityClass::TestImplementation))
        .take(5)
        .collect();

    for (i, entity) in test_entities.iter().enumerate() {
        println!("{}. {} [{}]",
            i+1,
            entity.interface_signature.name,
            entity.interface_signature.file_path.display()
        );
    }

    // Find code entities
    println!("\n--- Sample Code Entities ---");
    let code_entities: Vec<_> = entities.iter()
        .filter(|e| matches!(e.tdd_classification.entity_class,
            parseltongue_core::entities::EntityClass::CodeImplementation))
        .take(5)
        .collect();

    for (i, entity) in code_entities.iter().enumerate() {
        println!("{}. {} [{:?}] in {}",
            i+1,
            entity.interface_signature.name,
            entity.interface_signature.entity_type,
            entity.interface_signature.file_path.display()
        );
    }

    println!("\n✓ Tool 1 verification complete!\n");
}
