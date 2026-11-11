/// Test protocol query matching
use tree_sitter::{Query, Language, Parser, QueryCursor, StreamingIterator};

#[test]
fn test_protocol_query_matching() {
    let swift_code = r#"
protocol Drawable {
    func draw()
}
"#;

    let mut parser = Parser::new();
    let swift_lang: Language = tree_sitter_swift::LANGUAGE.into();
    parser.set_language(&swift_lang).unwrap();

    let tree = parser.parse(swift_code, None).unwrap();

    // Test just the protocol query
    let protocol_query = r#"
(protocol_declaration
  name: (type_identifier) @name) @definition.interface
"#;

    println!("\n=== Testing Protocol Query ===");
    println!("Query:\n{}", protocol_query);

    match Query::new(&swift_lang, protocol_query) {
        Ok(query) => {
            println!("✅ Protocol query compiled");

            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), swift_code.as_bytes());

            let mut match_count = 0;
            while let Some(m) = matches.next() {
                match_count += 1;
                println!("\nMatch #{}:", match_count);
                for capture in m.captures {
                    let capture_name = &query.capture_names()[capture.index as usize];
                    let text = &swift_code[capture.node.byte_range()];
                    println!("  Capture '{}': {}", capture_name, text.trim());
                }
            }

            if match_count == 0 {
                println!("❌ No matches found!");
            }

            assert!(match_count > 0, "Protocol query should match");
        }
        Err(e) => {
            panic!("❌ Protocol query compilation failed: {:?}", e);
        }
    }
}

#[test]
fn test_full_swift_query() {
    let swift_code = r#"
func myFunc() {}
class MyClass {}
struct MyStruct {}
protocol MyProtocol {}
"#;

    let mut parser = Parser::new();
    let swift_lang: Language = tree_sitter_swift::LANGUAGE.into();
    parser.set_language(&swift_lang).unwrap();

    let tree = parser.parse(swift_code, None).unwrap();

    let full_query = include_str!("../../../entity_queries/swift.scm");

    println!("\n=== Testing Full Swift Query ===");

    let query = Query::new(&swift_lang, full_query).unwrap();

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), swift_code.as_bytes());

    println!("\n=== Matches ===");
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let text = &swift_code[capture.node.byte_range()];

            if *capture_name == "name" {
                println!("Entity: {}", text.trim());
            } else if capture_name.starts_with("definition.") {
                println!("  Type: {}", capture_name);
            }
        }
    }
}
