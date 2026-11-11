// PHP Dependency Pattern Tests (v1.4.10)
// REQ-PHP-001.0 through REQ-PHP-003.0
// Following TDD: RED -> GREEN -> REFACTOR
//
// P3: PHP missing constructor, property access, static method calls

use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

// ============================================================================
// Helper Function
// ============================================================================

fn parse_php_code_extract_edges(code: &str) -> Vec<parseltongue_core::entities::DependencyEdge> {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, Path::new("test.php"), Language::Php)
        .expect("Failed to parse PHP code");
    edges
}

// ============================================================================
// REQ-PHP-001.0: Constructor Calls (object_creation_expression)
// ============================================================================

#[test]
fn test_php_constructor_simple() {
    let code = r#"<?php
class Service {
    public function create() {
        $user = new User();
        $config = new Config("test");
    }
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Constructor (simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect constructor calls
    let user_edge = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let config_edge = edges.iter().any(|e| e.to_key.as_str().contains("Config"));

    assert!(user_edge, "Expected edge for User constructor");
    assert!(config_edge, "Expected edge for Config constructor");
}

#[test]
fn test_php_constructor_qualified_name() {
    let code = r#"<?php
function build() {
    $user = new \Models\User();
    $logger = new \Utils\Logger();
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Constructor (qualified) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect qualified constructor calls
    let user_edge = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let logger_edge = edges.iter().any(|e| e.to_key.as_str().contains("Logger"));

    assert!(user_edge, "Expected edge for qualified User constructor");
    assert!(logger_edge, "Expected edge for qualified Logger constructor");
}

#[test]
fn test_php_constructor_with_parameters() {
    let code = r#"<?php
class Factory {
    public function make() {
        $db = new Database("localhost", 3306);
        $cache = new RedisCache("127.0.0.1", 6379);
    }
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Constructor (with parameters) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let db_edge = edges.iter().any(|e| e.to_key.as_str().contains("Database"));
    let cache_edge = edges.iter().any(|e| e.to_key.as_str().contains("RedisCache"));

    assert!(db_edge, "Expected edge for Database constructor");
    assert!(cache_edge, "Expected edge for RedisCache constructor");
}

// ============================================================================
// REQ-PHP-002.0: Property Access (member_access_expression)
// ============================================================================

#[test]
fn test_php_property_access_read() {
    let code = r#"<?php
function process($user) {
    $name = $user->name;
    $age = $user->age;
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Property Access (read) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect property access
    let name_edge = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edge = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edge, "Expected edge for name property access");
    assert!(age_edge, "Expected edge for age property access");
}

#[test]
fn test_php_property_access_write() {
    let code = r#"<?php
function update($user) {
    $user->name = "Alice";
    $user->age = 30;
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Property Access (write) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let name_edge = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edge = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edge, "Expected edge for name property write");
    assert!(age_edge, "Expected edge for age property write");
}

#[test]
fn test_php_property_access_chained() {
    let code = r#"<?php
function getValue($obj) {
    $result = $obj->parent->child->value;
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Property Access (chained) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect chained property access
    let has_property = edges.iter().any(|e| {
        e.to_key.as_str().contains("parent")
        || e.to_key.as_str().contains("child")
        || e.to_key.as_str().contains("value")
    });

    assert!(has_property, "Expected chained property access edges");
}

// ============================================================================
// REQ-PHP-003.0: Static Method Calls (scoped_call_expression)
// ============================================================================

#[test]
fn test_php_static_method_call() {
    let code = r#"<?php
function setup() {
    $instance = Factory::create();
    $result = Helper::process($data);
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Static Method Call Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect static method calls
    let create_edge = edges.iter().any(|e| e.to_key.as_str().contains("create"));
    let process_edge = edges.iter().any(|e| e.to_key.as_str().contains("process"));

    assert!(create_edge, "Expected edge for Factory::create()");
    assert!(process_edge, "Expected edge for Helper::process()");
}

#[test]
fn test_php_static_method_qualified() {
    let code = r#"<?php
function build() {
    $user = \Models\UserFactory::build();
    $logger = \Utils\LogFactory::create();
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Static Method (qualified) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let build_edge = edges.iter().any(|e| e.to_key.as_str().contains("build"));
    let create_edge = edges.iter().any(|e| e.to_key.as_str().contains("create"));

    assert!(build_edge, "Expected edge for UserFactory::build()");
    assert!(create_edge, "Expected edge for LogFactory::create()");
}

#[test]
fn test_php_static_method_with_parent_self() {
    let code = r#"<?php
class MyClass {
    public function test() {
        parent::doWork();
        self::helper();
        static::factory();
    }
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Static Method (parent/self/static) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect parent/self/static calls
    let has_static_calls = edges.iter().any(|e| {
        e.to_key.as_str().contains("doWork")
        || e.to_key.as_str().contains("helper")
        || e.to_key.as_str().contains("factory")
    });

    assert!(has_static_calls, "Expected parent/self/static method calls");
}

// ============================================================================
// REQ-PHP-004.0: Regression Test - Existing Patterns Still Work
// ============================================================================

#[test]
fn test_php_existing_patterns_not_broken() {
    let code = r#"<?php
use Some\Namespace\Class;

class MyClass {
    public function run() {
        doSomething();        // Function call
        $obj->method();       // Method call
    }
}
    "#;

    let edges = parse_php_code_extract_edges(code);

    println!("\n=== PHP Regression Test (existing patterns) ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  from: {} -> to: {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should still detect existing patterns
    assert!(edges.len() > 0, "Expected edges from existing patterns");
}
