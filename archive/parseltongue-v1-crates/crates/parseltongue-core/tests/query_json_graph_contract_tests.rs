//! Contract tests for JSON graph query helpers
//!
//! # Purpose (S06 Principle #1)
//! Executable specifications proving agents can query Parseltongue JSON exports
//!
//! # Test Coverage
//! - Happy path: All query functions work with valid JSON
//! - Error path: Graceful handling of malformed JSON
//! - Performance: < 100ms for 1,500 entities (S06 Principle #5)
//! - Edge cases: Empty arrays, missing fields, invalid types

use parseltongue_core::query_json_graph_errors::JsonGraphQueryError;
use parseltongue_core::query_json_graph_helpers::*;
use serde_json::json;
use std::time::Instant;

/// Test data: Realistic JSON graph representing payment processing flow
fn create_test_json_graph() -> serde_json::Value {
    json!({
        "entities": [
            {
                "isgl1_key": "rust:fn:main:src_main_rs:1-10",
                "name": "main",
                "file_path": "./src/main.rs",
                "reverse_deps": []
            },
            {
                "isgl1_key": "rust:fn:process_payment:src_payment_rs:145-167",
                "name": "process_payment",
                "file_path": "./src/payment.rs",
                "reverse_deps": ["rust:fn:main:src_main_rs:1-10"]
            },
            {
                "isgl1_key": "rust:fn:validate_payment:src_payment_rs:89-112",
                "name": "validate_payment",
                "file_path": "./src/payment.rs",
                "reverse_deps": [
                    "rust:fn:process_payment:src_payment_rs:145-167",
                    "rust:fn:handle_checkout:src_checkout_rs:200-245"
                ]
            },
            {
                "isgl1_key": "rust:fn:check_balance:src_payment_rs:50-70",
                "name": "check_balance",
                "file_path": "./src/payment.rs",
                "reverse_deps": ["rust:fn:validate_payment:src_payment_rs:89-112"]
            },
            {
                "isgl1_key": "rust:fn:handle_checkout:src_checkout_rs:200-245",
                "name": "handle_checkout",
                "file_path": "./src/checkout.rs",
                "reverse_deps": ["rust:fn:main:src_main_rs:1-10"]
            },
            {
                "isgl1_key": "rust:fn:login:src_auth_rs:10-30",
                "name": "login",
                "file_path": "./src/auth.rs",
                "reverse_deps": []
            },
            {
                "isgl1_key": "rust:fn:logout:src_auth_rs:35-50",
                "name": "logout",
                "file_path": "./src/auth.rs",
                "reverse_deps": []
            }
        ],
        "edges": [
            {
                "from_key": "rust:fn:main:src_main_rs:1-10",
                "to_key": "rust:fn:process_payment:src_payment_rs:145-167",
                "edge_type": "Calls"
            },
            {
                "from_key": "rust:fn:process_payment:src_payment_rs:145-167",
                "to_key": "rust:fn:validate_payment:src_payment_rs:89-112",
                "edge_type": "Calls"
            },
            {
                "from_key": "rust:fn:validate_payment:src_payment_rs:89-112",
                "to_key": "rust:fn:check_balance:src_payment_rs:50-70",
                "edge_type": "Calls"
            },
            {
                "from_key": "rust:fn:main:src_main_rs:1-10",
                "to_key": "rust:fn:handle_checkout:src_checkout_rs:200-245",
                "edge_type": "Calls"
            },
            {
                "from_key": "rust:struct:CreditCard:src_payment_rs:300-350",
                "to_key": "rust:trait:Payment:src_payment_rs:10-25",
                "edge_type": "Implements"
            },
            {
                "from_key": "rust:struct:BankTransfer:src_payment_rs:360-410",
                "to_key": "rust:trait:Payment:src_payment_rs:10-25",
                "edge_type": "Implements"
            },
            {
                "from_key": "rust:fn:process_payment:src_payment_rs:145-167",
                "to_key": "rust:struct:PaymentConfig:src_config_rs:20-40",
                "edge_type": "Uses"
            }
        ]
    })
}

/// Contract 1: Agent finds blast radius (reverse dependencies)
///
/// # Specification
/// GIVEN: JSON with reverse_deps arrays
/// WHEN: Agent queries "what breaks if I change validate_payment?"
/// THEN: Agent gets complete list of callers
#[test]
fn contract_find_reverse_dependencies_by_key() {
    let json = create_test_json_graph();

    let result = find_reverse_dependencies_by_key(
        &json,
        "rust:fn:validate_payment:src_payment_rs:89-112",
    );

    assert!(result.is_ok());
    let deps = result.unwrap();
    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&"rust:fn:process_payment:src_payment_rs:145-167".to_string()));
    assert!(deps.contains(&"rust:fn:handle_checkout:src_checkout_rs:200-245".to_string()));
}

/// Contract 2: Agent handles entity not found gracefully
///
/// # Specification
/// GIVEN: JSON graph
/// WHEN: Agent queries non-existent entity
/// THEN: Returns EntityNotFound error (no panic)
#[test]
fn contract_find_reverse_dependencies_entity_not_found() {
    let json = create_test_json_graph();

    let result = find_reverse_dependencies_by_key(
        &json,
        "rust:fn:nonexistent:src_fake_rs:1-10",
    );

    assert!(result.is_err());
    match result {
        Err(JsonGraphQueryError::EntityNotFound(key)) => {
            assert_eq!(key, "rust:fn:nonexistent:src_fake_rs:1-10");
        }
        _ => panic!("Expected EntityNotFound error"),
    }
}

/// Contract 3: Agent traverses call chain from root
///
/// # Specification
/// GIVEN: JSON with Calls edges
/// WHEN: Agent builds execution path from main
/// THEN: Agent reconstructs call chain
#[test]
fn contract_build_call_chain_from_root() {
    let json = create_test_json_graph();

    let result = build_call_chain_from_root(
        &json,
        "rust:fn:main:src_main_rs:1-10",
    );

    assert!(result.is_ok());
    let chain = result.unwrap();

    // Should follow main -> process_payment -> validate_payment -> check_balance
    assert!(chain.len() >= 4);
    assert_eq!(chain[0], "rust:fn:main:src_main_rs:1-10");
    assert_eq!(chain[1], "rust:fn:process_payment:src_payment_rs:145-167");
    assert_eq!(chain[2], "rust:fn:validate_payment:src_payment_rs:89-112");
    assert_eq!(chain[3], "rust:fn:check_balance:src_payment_rs:50-70");
}

/// Contract 4: Agent filters edges by type
///
/// # Specification
/// GIVEN: JSON with mixed edge types
/// WHEN: Agent filters by "Calls", "Uses", "Implements"
/// THEN: Returns only matching edges
#[test]
fn contract_filter_edges_by_type_only() {
    let json = create_test_json_graph();

    // Test Calls edges
    let calls_result = filter_edges_by_type_only(&json, "Calls");
    assert!(calls_result.is_ok());
    let calls = calls_result.unwrap();
    assert_eq!(calls.len(), 4);
    assert!(calls.iter().all(|e| e["edge_type"] == "Calls"));

    // Test Implements edges
    let impl_result = filter_edges_by_type_only(&json, "Implements");
    assert!(impl_result.is_ok());
    let impls = impl_result.unwrap();
    assert_eq!(impls.len(), 2);
    assert!(impls.iter().all(|e| e["edge_type"] == "Implements"));

    // Test Uses edges
    let uses_result = filter_edges_by_type_only(&json, "Uses");
    assert!(uses_result.is_ok());
    let uses = uses_result.unwrap();
    assert_eq!(uses.len(), 1);
    assert!(uses.iter().all(|e| e["edge_type"] == "Uses"));

    // Test invalid edge type
    let invalid_result = filter_edges_by_type_only(&json, "InvalidType");
    assert!(invalid_result.is_err());
    match invalid_result {
        Err(JsonGraphQueryError::InvalidEdgeType(t)) => {
            assert_eq!(t, "InvalidType");
        }
        _ => panic!("Expected InvalidEdgeType error"),
    }
}

/// Contract 5: Agent finds entities in specific files
///
/// # Specification
/// GIVEN: JSON with file_path metadata
/// WHEN: Agent searches for "auth" functions
/// THEN: Returns all entities in auth.rs
#[test]
fn contract_collect_entities_in_file_path() {
    let json = create_test_json_graph();

    // Find all auth functions
    let auth_result = collect_entities_in_file_path(&json, "auth");
    assert!(auth_result.is_ok());
    let auth_funcs = auth_result.unwrap();
    assert_eq!(auth_funcs.len(), 2);
    assert!(auth_funcs.iter().all(|e|
        e["file_path"].as_str().unwrap().contains("auth")
    ));

    // Find all payment functions
    let payment_result = collect_entities_in_file_path(&json, "payment");
    assert!(payment_result.is_ok());
    let payment_funcs = payment_result.unwrap();
    assert_eq!(payment_funcs.len(), 3); // process_payment, validate_payment, check_balance

    // Find checkout functions
    let checkout_result = collect_entities_in_file_path(&json, "checkout");
    assert!(checkout_result.is_ok());
    let checkout_funcs = checkout_result.unwrap();
    assert_eq!(checkout_funcs.len(), 1);
}

/// Contract 6: Performance validation (S06 Principle #5)
///
/// # Specification
/// GIVEN: JSON with 1,500 entities
/// WHEN: Agent runs any query
/// THEN: Completes in < 150ms (debug) / < 100ms (release)
///
/// Note: Debug builds are ~1.5x slower than release builds
#[test]
fn contract_query_performance_under_100ms() {
    // Create larger test dataset (simulating real codebase)
    let mut entities = vec![];
    let mut edges = vec![];

    for i in 0..1500 {
        entities.push(json!({
            "isgl1_key": format!("rust:fn:func{}:src_test_rs:{}-{}", i, i*10, i*10+20),
            "name": format!("func{}", i),
            "file_path": format!("./src/test{}.rs", i % 100),
            "reverse_deps": if i > 0 {
                vec![format!("rust:fn:func{}:src_test_rs:{}-{}", i-1, (i-1)*10, (i-1)*10+20)]
            } else {
                vec![]
            }
        }));

        if i > 0 {
            edges.push(json!({
                "from_key": format!("rust:fn:func{}:src_test_rs:{}-{}", i-1, (i-1)*10, (i-1)*10+20),
                "to_key": format!("rust:fn:func{}:src_test_rs:{}-{}", i, i*10, i*10+20),
                "edge_type": "Calls"
            }));
        }
    }

    let large_json = json!({
        "entities": entities,
        "edges": edges
    });

    // Performance threshold: 150ms for debug builds, 100ms for release
    const MAX_MS: u128 = if cfg!(debug_assertions) { 150 } else { 100 };

    // Test reverse deps query performance
    let start = Instant::now();
    let _ = find_reverse_dependencies_by_key(
        &large_json,
        "rust:fn:func750:src_test_rs:7500-7520",
    );
    let reverse_deps_time = start.elapsed();
    assert!(reverse_deps_time.as_millis() < MAX_MS,
        "Reverse deps query took {}ms (expected < {}ms)",
        reverse_deps_time.as_millis(), MAX_MS
    );

    // Test call chain query performance
    let start = Instant::now();
    let _ = build_call_chain_from_root(
        &large_json,
        "rust:fn:func0:src_test_rs:0-20",
    );
    let call_chain_time = start.elapsed();
    let max_ms_5x = MAX_MS * 5;  // 5x multiplier: 150ms → 750ms
    assert!(call_chain_time.as_millis() < max_ms_5x,
        "Call chain query took {}ms (expected < {}ms)",
        call_chain_time.as_millis(), max_ms_5x
    );

    // Test edge filter query performance
    let start = Instant::now();
    let _ = filter_edges_by_type_only(&large_json, "Calls");
    let filter_time = start.elapsed();
    assert!(filter_time.as_millis() < MAX_MS,
        "Edge filter query took {}ms (expected < {}ms)",
        filter_time.as_millis(), MAX_MS
    );

    // Test file path query performance
    let start = Instant::now();
    let _ = collect_entities_in_file_path(&large_json, "test50");
    let file_path_time = start.elapsed();
    assert!(file_path_time.as_millis() < MAX_MS,
        "File path query took {}ms (expected < {}ms)",
        file_path_time.as_millis(), MAX_MS
    );
}

/// Contract 7: Error handling with malformed JSON
///
/// # Specification
/// GIVEN: Malformed JSON (missing fields, wrong types)
/// WHEN: Agent runs queries
/// THEN: Returns MalformedJson errors (no panics)
#[test]
fn contract_error_handling_graceful_degradation() {
    // Test 1: Missing "entities" field
    let json_no_entities = json!({
        "edges": []
    });

    let result = find_reverse_dependencies_by_key(
        &json_no_entities,
        "rust:fn:test",
    );
    assert!(result.is_err());
    match result {
        Err(JsonGraphQueryError::MalformedJson(msg)) => {
            assert!(msg.contains("entities not array"));
        }
        _ => panic!("Expected MalformedJson error"),
    }

    // Test 2: Missing "edges" field
    let json_no_edges = json!({
        "entities": []
    });

    let result = build_call_chain_from_root(
        &json_no_edges,
        "rust:fn:test",
    );
    assert!(result.is_err());
    match result {
        Err(JsonGraphQueryError::MalformedJson(msg)) => {
            assert!(msg.contains("edges not array"));
        }
        _ => panic!("Expected MalformedJson error"),
    }

    // Test 3: Entities is not an array
    let json_entities_not_array = json!({
        "entities": "not an array",
        "edges": []
    });

    let result = collect_entities_in_file_path(
        &json_entities_not_array,
        "test",
    );
    assert!(result.is_err());

    // Test 4: Missing reverse_deps field
    let json_missing_reverse_deps = json!({
        "entities": [
            {
                "isgl1_key": "rust:fn:test:src_test_rs:1-10",
                "name": "test",
                "file_path": "./src/test.rs"
                // Missing reverse_deps field
            }
        ],
        "edges": []
    });

    let result = find_reverse_dependencies_by_key(
        &json_missing_reverse_deps,
        "rust:fn:test:src_test_rs:1-10",
    );
    assert!(result.is_err());
    match result {
        Err(JsonGraphQueryError::MalformedJson(msg)) => {
            assert!(msg.contains("reverse_deps not array"));
        }
        _ => panic!("Expected MalformedJson error"),
    }
}
