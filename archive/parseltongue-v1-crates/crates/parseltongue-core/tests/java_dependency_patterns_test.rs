// Java Dependency Pattern Tests (v1.4.9)
// REQ-JAVA-001.0 through REQ-JAVA-005.0
// Following TDD: RED -> GREEN -> REFACTOR
//
// P0 CRITICAL: Java is missing constructor call detection!

use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

// ============================================================================
// Helper Function
// ============================================================================

fn parse_java_code_extract_edges(code: &str) -> Vec<parseltongue_core::entities::DependencyEdge> {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, Path::new("Test.java"), Language::Java)
        .expect("Failed to parse Java code");
    edges
}

// ============================================================================
// REQ-JAVA-001.0: Constructor Calls (object_creation_expression)
// ============================================================================

#[test]
fn test_java_constructor_simple() {
    let code = r#"
public class Manager {
    public void create() {
        Person p = new Person();
        DataModel m = new DataModel();
    }
}
"#;

    let edges = parse_java_code_extract_edges(code);

    println!("\n=== Java Constructor (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect constructor calls to Person and DataModel
    let person_edges = edges.iter().any(|e| e.to_key.as_str().contains("Person"));
    let model_edges = edges.iter().any(|e| e.to_key.as_str().contains("DataModel"));

    assert!(person_edges, "Expected edge for Person constructor");
    assert!(model_edges, "Expected edge for DataModel constructor");
}

#[test]
fn test_java_constructor_generic() {
    let code = r#"
public class Container {
    public void init() {
        ArrayList<String> list = new ArrayList<>();
        HashMap<String, Integer> map = new HashMap<>();
    }
}
"#;

    let edges = parse_java_code_extract_edges(code);

    println!("\n=== Java Constructor (Generic) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect generic constructor calls
    let arraylist_edges = edges.iter().any(|e| e.to_key.as_str().contains("ArrayList"));
    let hashmap_edges = edges.iter().any(|e| e.to_key.as_str().contains("HashMap"));

    assert!(arraylist_edges, "Expected edge for ArrayList constructor");
    assert!(hashmap_edges, "Expected edge for HashMap constructor");
}

// ============================================================================
// REQ-JAVA-002.0: Field Access
// ============================================================================

#[test]
fn test_java_field_access_simple() {
    let code = r#"
public class Reader {
    public void read(Config config) {
        String val = config.getSetting();
        int port = config.getPort();
    }
}
"#;

    let edges = parse_java_code_extract_edges(code);

    println!("\n=== Java Field Access (Method Calls) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Java typically uses getters/setters, which are method calls
    // Field access nodes may not always be captured as separate dependencies
    let getter_edges = edges.iter().any(|e|
        e.to_key.as_str().contains("getSetting") || e.to_key.as_str().contains("getPort")
    );

    assert!(getter_edges, "Expected edges for getter method calls (field access pattern)");
}

// ============================================================================
// REQ-JAVA-003.0: Stream API / Collection Operations
// ============================================================================

#[test]
fn test_java_stream_operations() {
    let code = r#"
import java.util.List;
import java.util.stream.Collectors;

public class DataService {
    public List<String> getNames(List<User> users) {
        return users.stream()
            .filter(u -> u.isActive())
            .map(u -> u.getName())
            .collect(Collectors.toList());
    }
}
"#;

    let edges = parse_java_code_extract_edges(code);

    println!("\n=== Java Stream Operations Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect stream, filter, map, collect operations
    let stream_edges = edges.iter().any(|e| e.to_key.as_str().contains("stream"));
    let filter_edges = edges.iter().any(|e| e.to_key.as_str().contains("filter"));
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("map"));
    let collect_edges = edges.iter().any(|e| e.to_key.as_str().contains("collect"));

    assert!(stream_edges, "Expected edge for stream operation");
    assert!(filter_edges, "Expected edge for filter operation");
    assert!(map_edges, "Expected edge for map operation");
    assert!(collect_edges, "Expected edge for collect operation");
}

// ============================================================================
// REQ-JAVA-004.0: Generic Type References
// ============================================================================

#[test]
fn test_java_generic_type_variable() {
    let code = r#"
public class Repository {
    private List<User> users;
    private Map<String, List<Order>> ordersByUser;

    public void process(Set<Item> items) {
        // process logic
    }
}
"#;

    let edges = parse_java_code_extract_edges(code);

    println!("\n=== Java Generic Types Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect generic type references
    let list_edges = edges.iter().any(|e| e.to_key.as_str().contains("List"));
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("Map"));
    let set_edges = edges.iter().any(|e| e.to_key.as_str().contains("Set"));

    assert!(list_edges, "Expected edge for List generic type");
    assert!(map_edges, "Expected edge for Map generic type");
    assert!(set_edges, "Expected edge for Set generic type");
}

// ============================================================================
// REQ-JAVA-005.0: Annotations (Bonus Pattern)
// ============================================================================

#[test]
fn test_java_annotations() {
    let code = r#"
import javax.persistence.Entity;
import javax.persistence.Id;

@Entity
@Table(name = "users")
public class User {
    @Id
    @GeneratedValue
    private Long id;

    @Override
    public String toString() {
        return "User";
    }
}
"#;

    let edges = parse_java_code_extract_edges(code);

    println!("\n=== Java Annotations Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect annotation usage
    let entity_edges = edges.iter().any(|e| e.to_key.as_str().contains("Entity"));
    let has_annotations = edges.iter().any(|e|
        e.to_key.as_str().contains("Entity") ||
        e.to_key.as_str().contains("Id") ||
        e.to_key.as_str().contains("Override")
    );

    assert!(entity_edges, "Expected edge for Entity annotation");
    assert!(has_annotations, "Expected at least one annotation edge");
    // Note: @Override may not be captured as it's a built-in annotation without explicit import
}

// ============================================================================
// Integration Test: Complex Real-World Java Code
// ============================================================================

#[test]
fn test_java_integration_complex_service() {
    let code = r#"
import java.util.List;
import java.util.stream.Collectors;

public class UserService {
    private UserRepository repository;

    public UserService() {
        this.repository = new UserRepository();
    }

    public List<String> getActiveUserNames() {
        return repository.findAll().stream()
            .filter(user -> user.isActive())
            .map(user -> user.getName())
            .collect(Collectors.toList());
    }

    public User createUser(String name) {
        User user = new User();
        user.name = name;
        return repository.save(user);
    }
}
"#;

    let edges = parse_java_code_extract_edges(code);

    println!("\n=== Java Integration Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect various patterns:
    // 1. Constructor calls: new UserRepository(), new User()
    // 2. Field access: user.name
    // 3. Method calls: findAll(), stream(), filter(), map(), collect(), save()

    assert!(edges.len() >= 5, "Expected at least 5 edges from complex service");

    // Check for key dependencies
    let has_constructor = edges.iter().any(|e|
        e.to_key.as_str().contains("UserRepository") || e.to_key.as_str().contains("User")
    );
    let has_stream_ops = edges.iter().any(|e|
        e.to_key.as_str().contains("stream") ||
        e.to_key.as_str().contains("filter") ||
        e.to_key.as_str().contains("map")
    );

    assert!(has_constructor, "Expected constructor call edges");
    assert!(has_stream_ops, "Expected stream operation edges");
}
