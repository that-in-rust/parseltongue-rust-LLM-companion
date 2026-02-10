// PHP Dependency Edge Tests (T180-T199)
//
// Tests for PHP dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T180: PHP Constructor Calls (new ClassName())
// ============================================================================

#[test]
fn t180_php_constructor_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T180-php-constructor-creation-edges",
        "constructor_simple.php",
    );

    println!("\n=== T180: PHP Constructor Simple ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect constructor calls
    let user_edge = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let config_edge = edges.iter().any(|e| e.to_key.as_str().contains("Config"));

    assert!(user_edge, "Expected edge for User constructor");
    assert!(config_edge, "Expected edge for Config constructor");
}

#[test]
fn t180_php_constructor_qualified_name() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T180-php-constructor-creation-edges",
        "constructor_qualified.php",
    );

    println!("\n=== T180: PHP Constructor Qualified ===");
    println!("Edges found: {}", edges.len());

    // Should detect qualified constructor calls
    let user_edge = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let logger_edge = edges.iter().any(|e| e.to_key.as_str().contains("Logger"));

    assert!(user_edge, "Expected edge for qualified User constructor");
    assert!(logger_edge, "Expected edge for qualified Logger constructor");
}

#[test]
fn t180_php_constructor_with_parameters() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T180-php-constructor-creation-edges",
        "constructor_params.php",
    );

    println!("\n=== T180: PHP Constructor With Parameters ===");
    println!("Edges found: {}", edges.len());

    let db_edge = edges.iter().any(|e| e.to_key.as_str().contains("Database"));
    let cache_edge = edges.iter().any(|e| e.to_key.as_str().contains("RedisCache"));

    assert!(db_edge, "Expected edge for Database constructor");
    assert!(cache_edge, "Expected edge for RedisCache constructor");
}

// ============================================================================
// T182: PHP Property Access ($obj->prop)
// ============================================================================

#[test]
fn t182_php_property_access_read() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T182-php-property-access-edges",
        "property_read.php",
    );

    println!("\n=== T182: PHP Property Access Read ===");
    println!("Edges found: {}", edges.len());

    // Should detect property access
    let name_edge = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edge = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edge, "Expected edge for name property access");
    assert!(age_edge, "Expected edge for age property access");
}

#[test]
fn t182_php_property_access_write() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T182-php-property-access-edges",
        "property_write.php",
    );

    println!("\n=== T182: PHP Property Access Write ===");
    println!("Edges found: {}", edges.len());

    let name_edge = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edge = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edge, "Expected edge for name property write");
    assert!(age_edge, "Expected edge for age property write");
}

#[test]
fn t182_php_property_access_chained() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T182-php-property-access-edges",
        "property_chained.php",
    );

    println!("\n=== T182: PHP Property Access Chained ===");
    println!("Edges found: {}", edges.len());

    // Should detect chained property access
    let has_property = edges.iter().any(|e| {
        e.to_key.as_str().contains("parent")
        || e.to_key.as_str().contains("child")
        || e.to_key.as_str().contains("value")
    });

    assert!(has_property, "Expected chained property access edges");
}

// ============================================================================
// T184: PHP Static Method Calls (Class::method())
// ============================================================================

#[test]
fn t184_php_static_method_call() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T184-php-static-method-edges",
        "static_simple.php",
    );

    println!("\n=== T184: PHP Static Method Call ===");
    println!("Edges found: {}", edges.len());

    // Should detect static method calls
    let create_edge = edges.iter().any(|e| e.to_key.as_str().contains("create"));
    let process_edge = edges.iter().any(|e| e.to_key.as_str().contains("process"));

    assert!(create_edge, "Expected edge for Factory::create()");
    assert!(process_edge, "Expected edge for Helper::process()");
}

#[test]
fn t184_php_static_method_qualified() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T184-php-static-method-edges",
        "static_qualified.php",
    );

    println!("\n=== T184: PHP Static Method Qualified ===");
    println!("Edges found: {}", edges.len());

    let build_edge = edges.iter().any(|e| e.to_key.as_str().contains("build"));
    let create_edge = edges.iter().any(|e| e.to_key.as_str().contains("create"));

    assert!(build_edge, "Expected edge for UserFactory::build()");
    assert!(create_edge, "Expected edge for LogFactory::create()");
}

#[test]
fn t184_php_static_method_with_parent_self() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T184-php-static-method-edges",
        "static_parent_self.php",
    );

    println!("\n=== T184: PHP Static Method (parent/self/static) ===");
    println!("Edges found: {}", edges.len());

    // Should detect parent/self/static calls
    let has_static_calls = edges.iter().any(|e| {
        e.to_key.as_str().contains("doWork")
        || e.to_key.as_str().contains("helper")
        || e.to_key.as_str().contains("factory")
    });

    assert!(has_static_calls, "Expected parent/self/static method calls");
}

// ============================================================================
// T186: PHP Regression Test
// ============================================================================

#[test]
fn t186_php_existing_patterns_not_broken() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T186-php-regression-all-patterns",
        "regression.php",
    );

    println!("\n=== T186: PHP Regression Test ===");
    println!("Edges found: {}", edges.len());

    // Should still detect existing patterns
    assert!(edges.len() > 0, "Expected edges from existing patterns");
}
