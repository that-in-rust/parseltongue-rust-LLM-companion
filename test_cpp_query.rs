fn test_cpp_query() {
    use tree_sitter::{Parser, Query};
    
    let mut parser = Parser::new();
    let _result = parser.set_language(&tree_sitter_cpp::LANGUAGE.into());
    
    // Try the actual query from cpp.scm
    let query_text = r#"
; Functions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @definition.function
"#;

    println!("Attempting to parse query...");
    match Query::new(&tree_sitter_cpp::LANGUAGE.into(), query_text) {
        Ok(_q) => println!("✓ Query compiled successfully"),
        Err(e) => {
            println!("✗ Query failed to compile: {:?}", e);
            // Try simpler version
            let simple = "(function_definition) @test";
            match Query::new(&tree_sitter_cpp::LANGUAGE.into(), simple) {
                Ok(_) => println!("  Simple query works"),
                Err(e2) => println!("  Simple query also failed: {:?}", e2),
            }
        }
    }
}

fn main() {
    test_cpp_query();
}
