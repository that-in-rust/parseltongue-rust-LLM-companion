//! AST Exploration Tests - Understanding tree-sitter structure for complex patterns

use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

fn print_tree(node: tree_sitter::Node, source: &str, indent: usize, max_indent: usize) {
    if indent > max_indent {
        return;
    }

    let kind = node.kind();
    let text = if node.child_count() == 0 && node.byte_range().len() < 50 {
        format!(" \"{}\"", &source[node.byte_range()])
    } else {
        String::new()
    };

    println!("{}{} [{}:{}]{}",
        "  ".repeat(indent),
        kind,
        node.start_position().row,
        node.start_position().column,
        text
    );

    if node.child_count() > 0 {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            print_tree(child, source, indent + 1, max_indent);
        }
    }
}

#[test]
fn explore_nested_call_in_struct_construction() {
    let source = r#"
impl Config {
    fn new() -> Self {
        Self {
            settings: create_defaults(),
        }
    }
}
"#;

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();

    println!("\n=== AST for nested call in struct construction ===");
    print_tree(tree.root_node(), source, 0, 15);

    // Test our current query
    let dep_query = r#"
(call_expression
  function: [
    (identifier) @reference.call
    (field_expression
      field: (field_identifier) @reference.call)
    (scoped_identifier
      name: (identifier) @reference.call)
  ]) @dependency.call
"#;

    println!("\n=== Current Query Matches ===");
    let query = Query::new(&tree_sitter_rust::LANGUAGE.into(), dep_query).unwrap();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut match_count = 0;
    while let Some(m) = matches.next() {
        match_count += 1;
        println!("Match #{}:", match_count);
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let text = &source[capture.node.byte_range()];
            println!("  {} = \"{}\" at line {}", capture_name, text, capture.node.start_position().row);
        }
    }

    assert!(match_count > 0, "Should capture the create_defaults() call");
}

#[test]
fn explore_method_calls_in_chain() {
    let source = r#"
fn main() {
    let users = vec![1, 2, 3];
    let result: Vec<i32> = users.iter().map(|x| validate(*x)).collect();
}

fn validate(x: i32) -> bool {
    x > 0
}
"#;

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();

    println!("\n=== AST for chained method calls ===");
    print_tree(tree.root_node(), source, 0, 15);

    let dep_query = r#"
(call_expression
  function: [
    (identifier) @reference.call
    (field_expression
      field: (field_identifier) @reference.call)
    (scoped_identifier
      name: (identifier) @reference.call)
  ]) @dependency.call
"#;

    println!("\n=== Query Matches for Method Chains ===");
    let query = Query::new(&tree_sitter_rust::LANGUAGE.into(), dep_query).unwrap();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut match_count = 0;
    while let Some(m) = matches.next() {
        match_count += 1;
        println!("Match #{}:", match_count);
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let text = &source[capture.node.byte_range()];
            println!("  {} = \"{}\" at line {}", capture_name, text, capture.node.start_position().row);
        }
    }

    println!("\nTotal matches: {}", match_count);
}

#[test]
fn explore_macro_invocations() {
    let source = r#"
fn main() {
    println!("{:?}", config.get("test"));
    vec![create_item(), create_item()];
}
"#;

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();

    println!("\n=== AST for macro invocations ===");
    print_tree(tree.root_node(), source, 0, 15);

    // Check if macro_invocation nodes exist
    let macro_query = r#"
(macro_invocation
  macro: (identifier) @macro_name) @macro
"#;

    println!("\n=== Macro Invocation Matches ===");
    let query = Query::new(&tree_sitter_rust::LANGUAGE.into(), macro_query).unwrap();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut match_count = 0;
    while let Some(m) = matches.next() {
        match_count += 1;
        println!("Match #{}:", match_count);
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let text = &source[capture.node.byte_range()];
            println!("  {} = \"{}\" at line {}", capture_name, text, capture.node.start_position().row);
        }
    }

    println!("\nTotal macro matches: {}", match_count);

    // Now check for calls WITHIN macros
    let call_query = r#"
(call_expression
  function: [
    (identifier) @reference.call
    (field_expression
      field: (field_identifier) @reference.call)
  ]) @dependency.call
"#;

    println!("\n=== Call Expressions (including within macros) ===");
    let query = Query::new(&tree_sitter_rust::LANGUAGE.into(), call_query).unwrap();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut match_count = 0;
    while let Some(m) = matches.next() {
        match_count += 1;
        println!("Match #{}:", match_count);
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let text = &source[capture.node.byte_range()];
            println!("  {} = \"{}\" at line {}", capture_name, text, capture.node.start_position().row);
        }
    }

    println!("\nTotal call matches: {}", match_count);
}

#[test]
fn explore_calls_in_control_flow() {
    let source = r#"
fn main() {
    if validate(5) {
        process(5)
    } else {
        fallback()
    }
}
"#;

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();

    println!("\n=== AST for calls in control flow ===");
    print_tree(tree.root_node(), source, 0, 15);

    let dep_query = r#"
(call_expression
  function: (identifier) @reference.call) @dependency.call
"#;

    println!("\n=== Query Matches ===");
    let query = Query::new(&tree_sitter_rust::LANGUAGE.into(), dep_query).unwrap();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut match_count = 0;
    while let Some(m) = matches.next() {
        match_count += 1;
        println!("Match #{}:", match_count);
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let text = &source[capture.node.byte_range()];
            println!("  {} = \"{}\" at line {}", capture_name, text, capture.node.start_position().row);
        }
    }

    println!("\nTotal matches: {} (expected 3: validate, process, fallback)", match_count);
    assert_eq!(match_count, 3, "Should capture all three function calls");
}
