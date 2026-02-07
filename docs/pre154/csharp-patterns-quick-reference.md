# C# Dependency Patterns - Quick Reference

## Complete Pattern Support Matrix

| Pattern | Status | Example | Query File |
|---------|--------|---------|------------|
| **Method Calls** | ✅ | `obj.Method()` | c_sharp.scm |
| **Using Directives** | ✅ | `using System.Linq;` | c_sharp.scm |
| **Class Inheritance** | ✅ | `class Child : Parent` | c_sharp.scm |
| **Interface Implementation** | ✅ | `class Impl : IInterface` | c_sharp.scm |
| **Constructor Calls** | ✅ | `new List<string>()` | c_sharp.scm |
| **Property Access** | ✅ | `user.Name`, `obj.Prop = val` | c_sharp.scm |
| **LINQ Operations** | ✅ | `Where()`, `Select()`, etc. | c_sharp.scm |
| **Async/Await** | ✅ | `await FetchAsync()` | c_sharp.scm |

## Pattern Details

### 1. Property Access
```csharp
// Read
string name = user.Name;

// Write
user.Age = 30;

// Chained
var timeout = config.Settings.Timeout;
```

### 2. LINQ Operations (48 methods)

#### Filtering
```csharp
users.Where(u => u.Active)
```

#### Projection
```csharp
users.Select(u => u.Name)
```

#### Ordering
```csharp
users.OrderBy(u => u.Age)
     .ThenBy(u => u.Name)
```

#### Aggregation
```csharp
var count = items.Count();
var sum = numbers.Sum();
var avg = numbers.Average();
```

#### Element Operations
```csharp
var first = users.First();
var single = users.SingleOrDefault();
var last = users.Last();
```

#### Set Operations
```csharp
var distinct = items.Distinct();
var union = set1.Union(set2);
var intersection = set1.Intersect(set2);
```

#### Conversion
```csharp
var list = query.ToList();
var array = query.ToArray();
var dict = query.ToDictionary(x => x.Key);
```

### 3. Async/Await
```csharp
// Simple async call
var data = await FetchDataAsync();

// Chained async calls
var user = await response.Content.ReadAsync();

// Task utilities
await Task.Delay(1000);
await Task.WhenAll(task1, task2);
await Task.WhenAny(task3, task4);
```

## Supported LINQ Methods

### Complete List (48 methods):
- **Filtering**: `Where`
- **Projection**: `Select`
- **Ordering**: `OrderBy`, `OrderByDescending`, `ThenBy`, `ThenByDescending`, `Reverse`
- **Aggregation**: `Count`, `Sum`, `Average`, `Aggregate`, `Min`, `Max`
- **Element**: `First`, `FirstOrDefault`, `Last`, `LastOrDefault`, `Single`, `SingleOrDefault`, `ElementAt`, `ElementAtOrDefault`
- **Set**: `Distinct`, `Union`, `Intersect`, `Except`, `Concat`
- **Quantifiers**: `Any`, `All`, `Contains`, `SequenceEqual`
- **Partitioning**: `Take`, `Skip`, `TakeWhile`, `SkipWhile`
- **Joining**: `Join`, `GroupJoin`, `Zip`
- **Grouping**: `GroupBy`
- **Conversion**: `ToList`, `ToArray`, `ToDictionary`, `Cast`, `OfType`
- **Generation**: `DefaultIfEmpty`

## Testing

### Run All C# Tests
```bash
# All C# tests
cargo test csharp --all

# Specific test suites
cargo test --test csharp_remaining_patterns_test
cargo test --test csharp_constructor_detection_test
cargo test --test csharp_integration_validation_test
```

### Test Coverage
- **Property Access**: 3 tests
- **LINQ Operations**: 5 tests
- **Async/Await**: 3 tests
- **Combined Patterns**: 1 test
- **Constructor Calls**: 4 tests

**Total**: 16 tests covering C# dependency patterns

## Query File Location
```
dependency_queries/c_sharp.scm
```

## Tree-sitter Grammar
- **Language**: C#
- **Grammar**: `tree-sitter-c-sharp`
- **Version**: Compatible with C# 12

## Implementation Status

✅ **Phase 1** (v1.4.5): Constructor calls
✅ **Phase 2** (v1.4.8): Property access, LINQ, Async/Await

## Common Patterns Detected

### Web API Controller
```csharp
public class UserController : ControllerBase {
    private readonly IUserService _service;

    // Constructor injection ✅
    public UserController(IUserService service) {
        _service = service;
    }

    // Async endpoint ✅
    [HttpGet("{id}")]
    public async Task<ActionResult<User>> GetUser(int id) {
        var user = await _service.GetUserAsync(id);
        return user != null ? Ok(user) : NotFound();
    }

    // LINQ query ✅
    [HttpGet]
    public ActionResult<List<User>> GetActiveUsers() {
        var users = _service.GetAll()
            .Where(u => u.Active)
            .OrderBy(u => u.Name)
            .ToList();
        return Ok(users);
    }
}
```

### Data Service with LINQ
```csharp
public class DataService {
    // Property access ✅
    public int GetCount(List<Item> items) {
        return items.Count;
    }

    // LINQ operations ✅
    public List<Item> FilterAndSort(List<Item> items) {
        return items
            .Where(i => i.IsValid)
            .OrderBy(i => i.Priority)
            .ThenBy(i => i.Name)
            .Take(10)
            .ToList();
    }

    // Async with LINQ ✅
    public async Task<List<string>> ProcessAsync(List<int> ids) {
        var tasks = ids.Select(id => FetchAsync(id));
        var results = await Task.WhenAll(tasks);
        return results.Select(r => r.Name).ToList();
    }
}
```

## Performance Notes

- **Query Compilation**: All patterns compiled once at startup
- **Regex Performance**: LINQ method regex cached
- **Memory Impact**: Minimal (tree-sitter query caching)
- **Parse Speed**: No measurable impact vs baseline

## Known Limitations

1. **Generic Type Parameters**: `ReadAsAsync<User>()` may not capture `<User>` consistently
2. **Complex Lambda Expressions**: Very complex lambdas inside LINQ may not capture all nested calls
3. **Dynamic Invocation**: `dynamic` method calls not detected

## Version History

- **v1.4.5**: Constructor pattern implementation
- **v1.4.8**: Property access, LINQ, Async/Await patterns
- **v1.4.9**: (Planned) Generic type parameter improvements

---

**Last Updated**: 2026-02-06
**Status**: Complete ✅
