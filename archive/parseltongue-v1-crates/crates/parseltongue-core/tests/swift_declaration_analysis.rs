/// Analyze how Swift distinguishes between class/struct/enum
use tree_sitter::{Parser, Language};

#[test]
fn analyze_swift_declaration_types() {
    let swift_code = r#"
class MyClass {
    var x: Int
}

struct MyStruct {
    var y: Int
}

enum MyEnum {
    case first
}

protocol MyProtocol {
    func test()
}
"#;

    let mut parser = Parser::new();
    let swift_lang: Language = tree_sitter_swift::LANGUAGE.into();
    parser.set_language(&swift_lang).unwrap();

    let tree = parser.parse(swift_code, None).unwrap();
    let root = tree.root_node();

    println!("\n=== Swift Declaration Analysis ===");

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "class_declaration" || child.kind() == "protocol_declaration" {
            println!("\nNode: {}", child.kind());

            // Find the keyword child
            for i in 0..child.child_count() {
                if let Some(sub_child) = child.child(i) {
                    let kind = sub_child.kind();
                    let text = &swift_code[sub_child.byte_range()];
                    println!("  Child {}: {} = \"{}\"", i, kind, text.trim());

                    // Stop after first few children
                    if i > 5 {
                        break;
                    }
                }
            }
        }
    }
}
