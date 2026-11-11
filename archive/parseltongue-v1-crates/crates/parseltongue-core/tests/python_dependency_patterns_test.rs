// Python Dependency Pattern Tests (v1.4.9)
//
// Tests for comprehensive Python dependency detection:
// 1. Constructor calls (capitalized class instantiation)
// 2. Attribute access (obj.attr)
// 3. Async/await (await func())
// 4. Decorators (@decorator)
// 5. Type hints (List[T], Dict[K, V])
//
// TDD Phase: RED (tests written first, should fail)

use parseltongue_core::entities::{DependencyEdge, Language};
use parseltongue_core::query_extractor::{ParsedEntity, QueryBasedExtractor};
use std::path::Path;

/// Helper: Parse Python code and extract entities + edges
fn extract_python_dependencies(
    code: &str,
) -> (Vec<ParsedEntity>, Vec<DependencyEdge>) {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    extractor
        .parse_source(code, Path::new("test.py"), Language::Python)
        .expect("Failed to parse Python code")
}

// ============================================================================
// PATTERN 1: Constructor Calls (Capitalized Class Instantiation)
// ============================================================================

#[test]
fn test_python_constructor_class_call() {
    let code = r#"
class Service:
    def create(self):
        user = User()
        person = Person("John", 30)
        config = Configuration()
"#;

    let (entities, edges) = extract_python_dependencies(code);

    println!("=== Constructor Call Test ===");
    println!("Entities found: {}", entities.len());
    for entity in &entities {
        println!("  Entity: {} ({:?})", entity.name, entity.entity_type);
    }
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    // Verify constructor edges exist
    let constructor_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("User")
                || to_key_str.contains("Person")
                || to_key_str.contains("Configuration")
        })
        .collect();

    assert!(
        constructor_edges.len() >= 3,
        "Expected at least 3 constructor edges (User, Person, Configuration), found: {}",
        constructor_edges.len()
    );
}

#[test]
fn test_python_constructor_qualified() {
    let code = r#"
class Factory:
    def build(self):
        user = models.User()
        db = database.Connection()
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    let qualified_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("User") || e.to_key.to_string().contains("Connection"))
        .collect();

    assert!(
        qualified_edges.len() >= 2,
        "Expected at least 2 qualified constructor edges, found: {}",
        qualified_edges.len()
    );
}

// ============================================================================
// PATTERN 2: Attribute Access (Property Access)
// ============================================================================

#[test]
fn test_python_attribute_access() {
    let code = r#"
def process(user):
    name = user.name
    user.age = 30
    value = obj.parent.child
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    println!("=== Attribute Access Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let attribute_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("name")
                || e.to_key.to_string().contains("age")
                || e.to_key.to_string().contains("parent")
                || e.to_key.to_string().contains("child")
        })
        .collect();

    assert!(
        attribute_edges.len() >= 2,
        "Expected at least 2 attribute access edges, found: {}",
        attribute_edges.len()
    );
}

#[test]
fn test_python_property_getter_setter() {
    let code = r#"
class DataModel:
    def update(self, obj):
        val = obj.setting
        obj.config = "new_value"
        result = obj.data.value
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    let property_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("setting")
                || e.to_key.to_string().contains("config")
                || e.to_key.to_string().contains("data")
        })
        .collect();

    assert!(
        property_edges.len() >= 2,
        "Expected at least 2 property access edges, found: {}",
        property_edges.len()
    );
}

// ============================================================================
// PATTERN 3: Async/Await Operations
// ============================================================================

#[test]
fn test_python_async_await() {
    let code = r#"
async def load_data():
    result = await fetch_data()
    await self.save_async()
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    println!("=== Async/Await Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let async_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("fetch_data") || e.to_key.to_string().contains("save_async"))
        .collect();

    assert!(
        async_edges.len() >= 2,
        "Expected at least 2 async call edges, found: {}",
        async_edges.len()
    );
}

#[test]
fn test_python_async_class_method() {
    let code = r#"
class Fetcher:
    async def load(self):
        data = await get_user()
        result = await process_data(data)
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    let async_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("get_user") || e.to_key.to_string().contains("process_data"))
        .collect();

    assert!(
        async_edges.len() >= 2,
        "Expected at least 2 async method edges, found: {}",
        async_edges.len()
    );
}

// ============================================================================
// PATTERN 4: Decorators
// ============================================================================

#[test]
fn test_python_decorators() {
    let code = r#"
class MyClass:
    @property
    def name(self):
        return self._name

    @staticmethod
    def create():
        pass

    @app.route("/")
    def index():
        pass
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    println!("=== Decorator Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let decorator_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("property")
                || e.to_key.to_string().contains("staticmethod")
                || e.to_key.to_string().contains("route")
        })
        .collect();

    assert!(
        decorator_edges.len() >= 2,
        "Expected at least 2 decorator edges, found: {}",
        decorator_edges.len()
    );
}

#[test]
fn test_python_function_decorators() {
    let code = r#"
@login_required
@validate_input
def process_order(order_id):
    pass
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    let decorator_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("login_required") || e.to_key.to_string().contains("validate_input"))
        .collect();

    assert!(
        decorator_edges.len() >= 2,
        "Expected at least 2 function decorator edges, found: {}",
        decorator_edges.len()
    );
}

// ============================================================================
// PATTERN 5: Type Hints / Annotations
// ============================================================================

#[test]
fn test_python_type_hints() {
    let code = r#"
from typing import List, Dict

def process(items: List[str]) -> Dict[str, int]:
    users: List[User] = []
    return {}
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    println!("=== Type Hints Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let type_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("List")
                || e.to_key.to_string().contains("Dict")
                || e.to_key.to_string().contains("User")
        })
        .collect();

    assert!(
        type_edges.len() >= 2,
        "Expected at least 2 type hint edges, found: {}",
        type_edges.len()
    );
}

#[test]
fn test_python_optional_types() {
    let code = r#"
from typing import Optional, Union

def lookup(key: str) -> Optional[User]:
    value: Union[str, int] = get_value()
    return None
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    let type_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("Optional")
                || e.to_key.to_string().contains("Union")
                || e.to_key.to_string().contains("User")
        })
        .collect();

    assert!(
        type_edges.len() >= 2,
        "Expected at least 2 optional type edges, found: {}",
        type_edges.len()
    );
}

// ============================================================================
// PATTERN 6: List/Dict Comprehensions (Bonus)
// ============================================================================

#[test]
fn test_python_list_comprehensions() {
    let code = r#"
class Transformer:
    def transform(self, items):
        names = [process(x) for x in items]
        filtered = [validate(y) for y in data if check(y)]
"#;

    let (_entities, edges) = extract_python_dependencies(code);

    println!("=== List Comprehension Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let comprehension_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("process")
                || e.to_key.to_string().contains("validate")
                || e.to_key.to_string().contains("check")
        })
        .collect();

    assert!(
        comprehension_edges.len() >= 2,
        "Expected at least 2 comprehension call edges, found: {}",
        comprehension_edges.len()
    );
}

// ============================================================================
// INTEGRATION TEST: Multiple Patterns Together
// ============================================================================

#[test]
fn test_python_edge_integration() {
    let code = r#"
from typing import List

class UserService:
    @property
    def config(self):
        return self._config

    async def process_users(self, users: List[User]):
        for user in users:
            name = user.name
            result = await save_user(user)
            logger = Logger()
"#;

    let (entities, edges) = extract_python_dependencies(code);

    println!("=== Integration Test ===");
    println!("Entities: {}", entities.len());
    println!("Edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect:
    // - Type hints: List, User
    // - Decorators: property
    // - Async calls: save_user
    // - Attribute access: user.name
    // - Constructor calls: Logger()

    assert!(
        edges.len() >= 5,
        "Expected at least 5 edges from integration test, found: {}",
        edges.len()
    );
}
