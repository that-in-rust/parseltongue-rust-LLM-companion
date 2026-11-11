/// Integration test for Swift entity extraction with real code
use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

#[test]
fn test_swift_entity_extraction_integration() {
    let swift_code = r#"
// Test Swift file for entity extraction
import Foundation

// Function
func calculateSum(a: Int, b: Int) -> Int {
    return a + b
}

// Class
class UserManager {
    var users: [String] = []

    func addUser(name: String) {
        users.append(name)
    }
}

// Struct
struct Point {
    var x: Double
    var y: Double

    func distance(to other: Point) -> Double {
        let dx = x - other.x
        let dy = y - other.y
        return sqrt(dx * dx + dy * dy)
    }
}

// Enum
enum Direction {
    case north
    case south
    case east
    case west
}

// Protocol
protocol Drawable {
    func draw()
}
"#;

    let mut extractor = QueryBasedExtractor::new()
        .expect("Failed to initialize QueryBasedExtractor");

    let (entities, _deps) = extractor
        .parse_source(swift_code, Path::new("test.swift"), Language::Swift)
        .expect("Failed to parse Swift code");

    println!("\n=== Extracted Swift Entities ===");
    for entity in &entities {
        println!(
            "{:?}: {} (lines {}-{})",
            entity.entity_type, entity.name, entity.line_range.0, entity.line_range.1
        );
    }

    // Verify expected entities
    assert!(
        entities.iter().any(|e| e.name == "calculateSum"),
        "Should extract function 'calculateSum'"
    );
    assert!(
        entities.iter().any(|e| e.name == "UserManager"),
        "Should extract class 'UserManager'"
    );
    assert!(
        entities.iter().any(|e| e.name == "Point"),
        "Should extract struct 'Point'"
    );
    assert!(
        entities.iter().any(|e| e.name == "Direction"),
        "Should extract enum 'Direction'"
    );
    assert!(
        entities.iter().any(|e| e.name == "Drawable"),
        "Should extract protocol 'Drawable'"
    );

    // Verify methods inside types are also extracted
    assert!(
        entities.iter().any(|e| e.name == "addUser"),
        "Should extract method 'addUser'"
    );
    assert!(
        entities.iter().any(|e| e.name == "distance"),
        "Should extract method 'distance'"
    );

    println!("\n✅ Swift integration test passed! Extracted {} entities", entities.len());
}
