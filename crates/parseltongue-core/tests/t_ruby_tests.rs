// Ruby Dependency Edge Tests (T160-T179)
//
// Tests for Ruby dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T160: Ruby Constructor (Class.new) Edges
// ============================================================================

#[test]
fn t160_ruby_constructor_basic_new() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T160-ruby-constructor-new-edges",
        "basic_new.rb",
    );

    println!("\n=== T160: Ruby Constructor Basic ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect User.new and Config.new as constructor calls
    let user_edge = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let config_edge = edges.iter().any(|e| e.to_key.as_str().contains("Config"));

    assert!(user_edge, "Expected edge for User.new constructor");
    assert!(config_edge, "Expected edge for Config.new constructor");
}

#[test]
fn t160_ruby_constructor_with_args() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T160-ruby-constructor-new-edges",
        "new_with_args.rb",
    );

    println!("\n=== T160: Ruby Constructor With Args ===");
    println!("Edges found: {}", edges.len());

    // Should detect all three constructor calls
    let person_edge = edges.iter().any(|e| e.to_key.as_str().contains("Person"));
    let logger_edge = edges.iter().any(|e| e.to_key.as_str().contains("Logger"));
    let server_edge = edges.iter().any(|e| e.to_key.as_str().contains("Server"));

    assert!(person_edge, "Expected edge for Person.new");
    assert!(logger_edge, "Expected edge for Logger.new");
    assert!(server_edge, "Expected edge for Server.new");
}

#[test]
fn t160_ruby_constructor_qualified() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T160-ruby-constructor-new-edges",
        "qualified_new.rb",
    );

    println!("\n=== T160: Ruby Constructor Qualified ===");
    println!("Edges found: {}", edges.len());

    // Should detect qualified constructor calls
    let user_edge = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let db_edge = edges.iter().any(|e| e.to_key.as_str().contains("Connection"));

    assert!(user_edge, "Expected edge for Models::User.new");
    assert!(db_edge, "Expected edge for Database::Connection.new");
}

// ============================================================================
// T162: Ruby Block Methods (.each, .map, .select)
// ============================================================================

#[test]
fn t162_ruby_block_methods_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T162-ruby-block-method-edges",
        "block_basic.rb",
    );

    println!("\n=== T162: Ruby Block Methods Basic ===");
    println!("Edges found: {}", edges.len());

    // Should detect block method calls
    let each_edge = edges.iter().any(|e| e.to_key.as_str().contains("each"));
    let map_edge = edges.iter().any(|e| e.to_key.as_str().contains("map"));
    let select_edge = edges.iter().any(|e| e.to_key.as_str().contains("select"));

    assert!(each_edge, "Expected edge for .each block method");
    assert!(map_edge, "Expected edge for .map block method");
    assert!(select_edge, "Expected edge for .select block method");
}

#[test]
fn t162_ruby_block_methods_do_end() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T162-ruby-block-method-edges",
        "block_do_end.rb",
    );

    println!("\n=== T162: Ruby Block Methods (do...end) ===");
    println!("Edges found: {}", edges.len());

    // Should detect do...end block methods
    let each_edge = edges.iter().any(|e| e.to_key.as_str().contains("each"));
    let select_edge = edges.iter().any(|e| e.to_key.as_str().contains("select"));

    assert!(each_edge, "Expected edge for .each (do...end) block method");
    assert!(select_edge, "Expected edge for .select (do...end) block method");
}

#[test]
fn t162_ruby_block_methods_chained() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T162-ruby-block-method-edges",
        "block_chained.rb",
    );

    println!("\n=== T162: Ruby Block Methods Chained ===");
    println!("Edges found: {}", edges.len());

    // Should detect chained method calls
    let select_edge = edges.iter().any(|e| e.to_key.as_str().contains("select"));
    let map_edge = edges.iter().any(|e| e.to_key.as_str().contains("map"));
    let compact_edge = edges.iter().any(|e| e.to_key.as_str().contains("compact"));
    let uniq_edge = edges.iter().any(|e| e.to_key.as_str().contains("uniq"));
    let sort_edge = edges.iter().any(|e| e.to_key.as_str().contains("sort"));

    assert!(select_edge, "Expected edge for .select");
    assert!(map_edge, "Expected edge for .map");
    assert!(compact_edge, "Expected edge for .compact");
    assert!(uniq_edge, "Expected edge for .uniq");
    assert!(sort_edge, "Expected edge for .sort");
}

// ============================================================================
// T164: Ruby Attribute Access
// ============================================================================

#[test]
fn t164_ruby_attribute_access_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T164-ruby-attribute-access-edges",
        "attr_basic.rb",
    );

    println!("\n=== T164: Ruby Attribute Access Basic ===");
    println!("Edges found: {}", edges.len());

    // Should detect attribute access
    let name_edge = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edge = edges.iter().any(|e| e.to_key.as_str().contains("age"));
    let email_edge = edges.iter().any(|e| e.to_key.as_str().contains("email"));

    assert!(name_edge, "Expected edge for .name attribute access");
    assert!(age_edge, "Expected edge for .age attribute access");
    assert!(email_edge, "Expected edge for .email attribute access");
}

#[test]
fn t164_ruby_attribute_chained_access() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T164-ruby-attribute-access-edges",
        "attr_chained.rb",
    );

    println!("\n=== T164: Ruby Attribute Access Chained ===");
    println!("Edges found: {}", edges.len());

    // Should detect chained attribute access
    let parent_edge = edges.iter().any(|e| e.to_key.as_str().contains("parent"));
    let child_edge = edges.iter().any(|e| e.to_key.as_str().contains("child"));
    let value_edge = edges.iter().any(|e| e.to_key.as_str().contains("value"));
    let config_edge = edges.iter().any(|e| e.to_key.as_str().contains("config"));

    assert!(parent_edge, "Expected edge for .parent");
    assert!(child_edge, "Expected edge for .child");
    assert!(value_edge, "Expected edge for .value");
    assert!(config_edge, "Expected edge for .config");
}

// ============================================================================
// T165: Ruby Module Inclusion (include, extend, prepend)
// ============================================================================
// SKIPPED: Tree-sitter Ruby grammar limitation
// The tree-sitter Ruby parser doesn't expose constant arguments for bareword
// method calls like `include Enumerable` in a queryable way.
// This is marked as P2 (low priority) and requires either tree-sitter updates
// or custom parsing logic. T-folder serves as aspirational fixture.

#[test]
#[ignore] // Ruby module inclusion not yet implemented (REQ-RUBY-003.0)
fn t165_ruby_module_inclusion_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T165-ruby-module-inclusion-edges",
        "module_basic.rb",
    );

    println!("\n=== T165: Ruby Module Inclusion Basic ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect include, extend, prepend statements
    let include_edge = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Enumerable"));
    let extend_edge = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Comparable"));
    let prepend_edge = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Serializable"));

    assert!(include_edge, "Expected edge for include Enumerable");
    assert!(extend_edge, "Expected edge for extend Comparable");
    assert!(prepend_edge, "Expected edge for prepend Serializable");
}

#[test]
#[ignore] // Ruby module inclusion not yet implemented (REQ-RUBY-003.0)
fn t165_ruby_module_inclusion_qualified() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T165-ruby-module-inclusion-edges",
        "module_qualified.rb",
    );

    println!("\n=== T165: Ruby Module Inclusion Qualified ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect qualified module inclusion
    let callbacks_edge = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Callbacks"));
    let validations_edge = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Validations"));
    let cacheable_edge = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Cacheable"));

    assert!(
        callbacks_edge,
        "Expected edge for include ActiveSupport::Callbacks"
    );
    assert!(
        validations_edge,
        "Expected edge for extend ActiveSupport::Validations"
    );
    assert!(
        cacheable_edge,
        "Expected edge for prepend Concerns::Cacheable"
    );
}
