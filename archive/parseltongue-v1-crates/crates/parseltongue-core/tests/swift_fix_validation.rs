/// Comprehensive validation that Swift entity extraction fix works end-to-end
///
/// This test validates the fix for the "Failed to create query" error that
/// occurred in v0.8.9 when Swift was the only language failing among 12 supported.
///
/// Root cause: swift.scm used node types (struct_declaration, enum_declaration)
/// that don't exist in tree-sitter-swift v0.7 grammar.
///
/// Fix: Use class_declaration for all type declarations (class/struct/enum)
///      and protocol_declaration for protocols.
use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

#[test]
fn test_swift_query_compiles_without_error() {
    // This was failing with: "Failed to create query" error
    // Specifically at row: 12 (struct_declaration line)
    let result = QueryBasedExtractor::new();

    assert!(
        result.is_ok(),
        "QueryBasedExtractor should initialize successfully. \
         If this fails with 'Failed to create query', check entity_queries/swift.scm \
         for invalid node types."
    );

    println!("✅ Swift query compiled successfully (no 'Failed to create query' error)");
}

#[test]
fn test_swift_extracts_all_entity_types() {
    let swift_code = r#"
// Functions
func globalFunction() {
    print("Hello")
}

// Class
class MyClass {
    func classMethod() {}
}

// Struct
struct MyStruct {
    func structMethod() {}
}

// Enum
enum MyEnum {
    case value
}

// Protocol
protocol MyProtocol {
    func protocolMethod()
}
"#;

    let mut extractor = QueryBasedExtractor::new()
        .expect("Failed to initialize QueryBasedExtractor");

    let (entities, _) = extractor
        .parse_source(swift_code, Path::new("test.swift"), Language::Swift)
        .expect("Failed to parse Swift code");

    println!("\n=== Extracted Entities ===");
    for entity in &entities {
        println!(
            "{:?}: {} (line {})",
            entity.entity_type, entity.name, entity.line_range.0
        );
    }

    // Validate function extraction
    assert!(
        entities.iter().any(|e| e.name == "globalFunction"),
        "Should extract global function"
    );

    // Validate class extraction
    assert!(
        entities.iter().any(|e| e.name == "MyClass"),
        "Should extract class (via class_declaration node)"
    );

    // Validate struct extraction
    // NOTE: Swift uses class_declaration for structs, so tagged as Class
    assert!(
        entities.iter().any(|e| e.name == "MyStruct"),
        "Should extract struct (via class_declaration node)"
    );

    // Validate enum extraction
    // NOTE: Swift uses class_declaration for enums, so tagged as Class
    assert!(
        entities.iter().any(|e| e.name == "MyEnum"),
        "Should extract enum (via class_declaration node)"
    );

    // Validate protocol extraction
    assert!(
        entities.iter().any(|e| e.name == "MyProtocol"),
        "Should extract protocol (via protocol_declaration node)"
    );

    // Validate methods are extracted
    assert!(
        entities.iter().any(|e| e.name == "classMethod"),
        "Should extract class methods"
    );

    println!("\n✅ All Swift entity types extracted successfully");
    println!("   Total entities: {}", entities.len());
}

#[test]
fn test_swift_protocol_uses_interface_entity_type() {
    let swift_code = r#"
protocol Drawable {
    func draw()
}
"#;

    let mut extractor = QueryBasedExtractor::new().unwrap();
    let (entities, _) = extractor
        .parse_source(swift_code, Path::new("test.swift"), Language::Swift)
        .unwrap();

    let protocol_entity = entities
        .iter()
        .find(|e| e.name == "Drawable")
        .expect("Should extract protocol");

    assert_eq!(
        protocol_entity.entity_type,
        parseltongue_core::query_extractor::EntityType::Interface,
        "Protocols should be tagged as Interface type (not Trait)"
    );

    println!("✅ Protocol correctly tagged as Interface entity type");
}

#[test]
fn test_swift_real_world_code_extraction() {
    // Real-world Swift code with multiple entity types
    let swift_code = r#"
import Foundation

// MARK: - Data Models

struct User {
    let id: UUID
    let name: String
    let email: String
}

enum UserRole {
    case admin
    case moderator
    case user
}

// MARK: - Protocols

protocol UserRepository {
    func fetch(id: UUID) async throws -> User
    func save(_ user: User) async throws
}

// MARK: - Implementations

class InMemoryUserRepository: UserRepository {
    private var users: [UUID: User] = [:]

    func fetch(id: UUID) async throws -> User {
        guard let user = users[id] else {
            throw RepositoryError.notFound
        }
        return user
    }

    func save(_ user: User) async throws {
        users[user.id] = user
    }
}

// MARK: - Utility Functions

func validateEmail(_ email: String) -> Bool {
    let regex = try! NSRegularExpression(pattern: "^[A-Z0-9._%+-]+@[A-Z0-9.-]+\\.[A-Z]{2,}$")
    return regex.firstMatch(in: email, range: NSRange(email.startIndex..., in: email)) != nil
}

func generateUserKey(for user: User) -> String {
    return "user:\(user.id.uuidString)"
}
"#;

    let mut extractor = QueryBasedExtractor::new().unwrap();
    let (entities, _) = extractor
        .parse_source(swift_code, Path::new("UserRepository.swift"), Language::Swift)
        .unwrap();

    println!("\n=== Real-world Swift Code Extraction ===");
    for entity in &entities {
        println!(
            "{:?}: {} (lines {}-{})",
            entity.entity_type, entity.name, entity.line_range.0, entity.line_range.1
        );
    }

    // Validate expected entities
    assert!(entities.iter().any(|e| e.name == "User"), "Should extract User struct");
    assert!(entities.iter().any(|e| e.name == "UserRole"), "Should extract UserRole enum");
    assert!(entities.iter().any(|e| e.name == "UserRepository"), "Should extract UserRepository protocol");
    assert!(entities.iter().any(|e| e.name == "InMemoryUserRepository"), "Should extract InMemoryUserRepository class");
    assert!(entities.iter().any(|e| e.name == "fetch"), "Should extract fetch method");
    assert!(entities.iter().any(|e| e.name == "save"), "Should extract save method");
    assert!(entities.iter().any(|e| e.name == "validateEmail"), "Should extract validateEmail function");
    assert!(entities.iter().any(|e| e.name == "generateUserKey"), "Should extract generateUserKey function");

    let entity_count = entities.len();
    assert!(
        entity_count >= 8,
        "Should extract at least 8 entities, got {}",
        entity_count
    );

    println!("\n✅ Real-world Swift code extraction successful");
    println!("   Extracted {} entities from production-like code", entity_count);
}
