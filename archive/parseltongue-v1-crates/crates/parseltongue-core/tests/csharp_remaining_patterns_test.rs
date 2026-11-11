// C# Remaining Dependency Pattern Tests (v1.4.8)
// REQ-CSHARP-002.0 through REQ-CSHARP-004.0
// Following TDD: RED -> GREEN -> REFACTOR

use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

// ============================================================================
// Helper Function
// ============================================================================

fn parse_csharp_code_extract_edges(code: &str) -> Vec<parseltongue_core::entities::DependencyEdge> {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, Path::new("test.cs"), Language::CSharp)
        .expect("Failed to parse C# code");
    edges
}

// ============================================================================
// REQ-CSHARP-002.0: Property Access (non-call)
// ============================================================================

#[test]
fn test_csharp_property_access_simple() {
    let code = r#"
public class Service {
    public void Process(User user) {
        string name = user.Name;
        int age = user.Age;
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== Property Access (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect property access to Name and Age
    let name_edges = edges.iter().any(|e| e.to_key.as_str().contains("Name"));
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("Age"));

    assert!(name_edges, "Expected edge for Name property access");
    assert!(age_edges, "Expected edge for Age property access");
}

#[test]
fn test_csharp_property_assignment() {
    let code = r#"
public class Service {
    public void Update(User user) {
        user.Age = 30;
        user.Status = "active";
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== Property Assignment Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect property assignments
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("Age"));
    let status_edges = edges.iter().any(|e| e.to_key.as_str().contains("Status"));

    assert!(age_edges, "Expected edge for Age property assignment");
    assert!(status_edges, "Expected edge for Status property assignment");
}

#[test]
fn test_csharp_property_chaining() {
    let code = r#"
public class Service {
    public void Process(Config config) {
        var timeout = config.Settings.Timeout;
        var host = config.Connection.Host;
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== Property Chaining Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect chained property access
    let has_settings = edges.iter().any(|e| e.to_key.as_str().contains("Settings"));
    let has_timeout = edges.iter().any(|e| e.to_key.as_str().contains("Timeout"));
    let has_connection = edges.iter().any(|e| e.to_key.as_str().contains("Connection"));
    let has_host = edges.iter().any(|e| e.to_key.as_str().contains("Host"));

    assert!(has_settings || has_timeout, "Expected edge for Settings or Timeout property");
    assert!(has_connection || has_host, "Expected edge for Connection or Host property");
}

// ============================================================================
// REQ-CSHARP-003.0: LINQ Operations
// ============================================================================

#[test]
fn test_csharp_linq_where_select() {
    let code = r#"
public class DataService {
    public List<string> GetNames(List<User> users) {
        return users
            .Where(u => u.Active)
            .Select(u => u.Name)
            .ToList();
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== LINQ Where/Select Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect LINQ methods
    let where_edges = edges.iter().any(|e| e.to_key.as_str().contains("Where"));
    let select_edges = edges.iter().any(|e| e.to_key.as_str().contains("Select"));
    let tolist_edges = edges.iter().any(|e| e.to_key.as_str().contains("ToList"));

    assert!(where_edges, "Expected edge for Where LINQ method");
    assert!(select_edges, "Expected edge for Select LINQ method");
    assert!(tolist_edges, "Expected edge for ToList LINQ method");
}

#[test]
fn test_csharp_linq_aggregate_operations() {
    let code = r#"
public class Calculator {
    public int Calculate(List<int> numbers) {
        var count = numbers.Count();
        var sum = numbers.Sum();
        var avg = numbers.Average();
        return sum;
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== LINQ Aggregate Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect aggregate LINQ methods
    let count_edges = edges.iter().any(|e| e.to_key.as_str().contains("Count"));
    let sum_edges = edges.iter().any(|e| e.to_key.as_str().contains("Sum"));
    let avg_edges = edges.iter().any(|e| e.to_key.as_str().contains("Average"));

    assert!(count_edges, "Expected edge for Count LINQ method");
    assert!(sum_edges, "Expected edge for Sum LINQ method");
    assert!(avg_edges, "Expected edge for Average LINQ method");
}

#[test]
fn test_csharp_linq_ordering() {
    let code = r#"
public class Sorter {
    public List<User> Sort(List<User> users) {
        return users
            .OrderBy(u => u.Name)
            .ThenBy(u => u.Age)
            .ToList();
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== LINQ Ordering Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect ordering LINQ methods
    let orderby_edges = edges.iter().any(|e| e.to_key.as_str().contains("OrderBy"));
    let thenby_edges = edges.iter().any(|e| e.to_key.as_str().contains("ThenBy"));

    assert!(orderby_edges, "Expected edge for OrderBy LINQ method");
    assert!(thenby_edges, "Expected edge for ThenBy LINQ method");
}

#[test]
fn test_csharp_linq_first_single_operations() {
    let code = r#"
public class Finder {
    public User Find(List<User> users) {
        var first = users.FirstOrDefault();
        var single = users.SingleOrDefault();
        var last = users.Last();
        return first;
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== LINQ First/Single Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect First/Single/Last LINQ methods
    let first_edges = edges.iter().any(|e| e.to_key.as_str().contains("FirstOrDefault"));
    let single_edges = edges.iter().any(|e| e.to_key.as_str().contains("SingleOrDefault"));
    let last_edges = edges.iter().any(|e| e.to_key.as_str().contains("Last"));

    assert!(first_edges, "Expected edge for FirstOrDefault LINQ method");
    assert!(single_edges, "Expected edge for SingleOrDefault LINQ method");
    assert!(last_edges, "Expected edge for Last LINQ method");
}

#[test]
fn test_csharp_linq_set_operations() {
    let code = r#"
public class SetProcessor {
    public List<int> Process(List<int> a, List<int> b) {
        var distinct = a.Distinct();
        var union = a.Union(b);
        var intersect = a.Intersect(b);
        return distinct.ToList();
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== LINQ Set Operations Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect set LINQ methods
    let distinct_edges = edges.iter().any(|e| e.to_key.as_str().contains("Distinct"));
    let union_edges = edges.iter().any(|e| e.to_key.as_str().contains("Union"));
    let intersect_edges = edges.iter().any(|e| e.to_key.as_str().contains("Intersect"));

    assert!(distinct_edges, "Expected edge for Distinct LINQ method");
    assert!(union_edges, "Expected edge for Union LINQ method");
    assert!(intersect_edges, "Expected edge for Intersect LINQ method");
}

// ============================================================================
// REQ-CSHARP-004.0: Async/Await
// ============================================================================

#[test]
fn test_csharp_async_await_simple() {
    let code = r#"
public class AsyncService {
    public async Task ProcessAsync() {
        var data = await FetchDataAsync();
        await SaveAsync(data);
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== Async/Await (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect async method calls
    let fetch_edges = edges.iter().any(|e| e.to_key.as_str().contains("FetchDataAsync"));
    let save_edges = edges.iter().any(|e| e.to_key.as_str().contains("SaveAsync"));

    assert!(fetch_edges, "Expected edge for FetchDataAsync method");
    assert!(save_edges, "Expected edge for SaveAsync method");
}

#[test]
fn test_csharp_async_await_member_access() {
    let code = r#"
public class ApiClient {
    public async Task GetUserAsync(int id) {
        var response = await this.HttpClient.GetAsync(url);
        var data = await response.Content.ReadAsync();
        var json = await ParseJsonAsync(data);
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== Async/Await Member Access Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect async method calls on members
    let get_edges = edges.iter().any(|e| e.to_key.as_str().contains("GetAsync"));
    let read_edges = edges.iter().any(|e| e.to_key.as_str().contains("ReadAsync"));
    let parse_edges = edges.iter().any(|e| e.to_key.as_str().contains("ParseJsonAsync"));

    assert!(get_edges, "Expected edge for GetAsync method");
    assert!(read_edges, "Expected edge for ReadAsync method");
    assert!(parse_edges, "Expected edge for ParseJsonAsync method");
}

#[test]
fn test_csharp_async_await_task_operations() {
    let code = r#"
public class TaskManager {
    public async Task RunAsync() {
        await Task.Delay(1000);
        await Task.WhenAll(task1, task2);
        await Task.WhenAny(task3, task4);
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== Async Task Operations Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect Task utility methods
    let delay_edges = edges.iter().any(|e| e.to_key.as_str().contains("Delay"));
    let whenall_edges = edges.iter().any(|e| e.to_key.as_str().contains("WhenAll"));
    let whenany_edges = edges.iter().any(|e| e.to_key.as_str().contains("WhenAny"));

    assert!(delay_edges, "Expected edge for Task.Delay method");
    assert!(whenall_edges, "Expected edge for Task.WhenAll method");
    assert!(whenany_edges, "Expected edge for Task.WhenAny method");
}

// ============================================================================
// Integration Test: Multiple Patterns
// ============================================================================

#[test]
fn test_csharp_combined_patterns() {
    let code = r#"
public class CompleteService {
    public async Task<List<string>> ProcessAsync(List<User> users) {
        // Property access
        var count = users.Count;

        // LINQ operations
        var active = users
            .Where(u => u.Active)
            .OrderBy(u => u.Name)
            .ToList();

        // Async/await
        var data = await FetchDataAsync();

        // Constructor
        var result = new List<string>();

        // Property assignment
        result.Capacity = count;

        return result;
    }
}
"#;

    let edges = parse_csharp_code_extract_edges(code);

    println!("\n=== Combined Patterns Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect multiple pattern types
    let has_property = edges.iter().any(|e| {
        e.to_key.as_str().contains("Count") || e.to_key.as_str().contains("Capacity")
    });
    let has_linq = edges.iter().any(|e| {
        e.to_key.as_str().contains("Where") || e.to_key.as_str().contains("OrderBy")
    });
    let has_async = edges.iter().any(|e| e.to_key.as_str().contains("FetchDataAsync"));
    let has_constructor = edges.iter().any(|e| e.to_key.as_str().contains("List"));

    assert!(has_property, "Expected property access edges");
    assert!(has_linq, "Expected LINQ method edges");
    assert!(has_async, "Expected async/await edges");
    assert!(has_constructor, "Expected constructor edges");

    // Should have at least 6 edges (conservative estimate)
    assert!(
        edges.len() >= 6,
        "Expected at least 6 edges for combined patterns, found {}",
        edges.len()
    );
}
