# Bug Analysis: Qualified Name Key Format Issues (v1.5.1)

## Executive Summary

Two related bugs discovered when testing on larger codebases. Both involve **ISGL1 key format parsing failures** caused by qualified names containing `::` namespace separators, which conflict with the `:` delimiter used in ISGL1 keys.

---

## Bug #1: C# `global::` Namespace Prefix

### Observed Error
```
Warning: Invalid external dependency key format: csharp:fn:global::System.Resources.ResourceManager:unresolved-reference:0-0
```

### Root Cause Analysis

1. **ISGL1 Key Format**: Uses `:` as delimiter
   ```
   {lang}:{type}:{name}:{path}:{lines}
   ```
   Expected: 5 colon-separated parts

2. **C# Qualified Names**: Use `::` for global namespace alias
   ```csharp
   var rm = new global::System.Resources.ResourceManager("name", assembly);
   ```

3. **Key Generation** (`query_extractor.rs:668-671`):
   ```rust
   let to_key = format!(
       "{}:fn:{}:unresolved-reference:0-0",
       language,
       to  // Contains "global::System.Resources.ResourceManager"
   );
   ```

4. **Resulting Malformed Key**:
   ```
   csharp:fn:global::System.Resources.ResourceManager:unresolved-reference:0-0
   ```
   When split by `:`, produces 7 parts instead of 5:
   - `csharp`
   - `fn`
   - `global`
   - `` (empty)
   - `System.Resources.ResourceManager`
   - `unresolved-reference`
   - `0-0`

5. **Parser Failure** (`external_dependency_handler.rs:210`):
   ```rust
   if parts.len() != 5 {
       return Err(StreamerError::ParsingError { ... });
   }
   ```

### C# Patterns That Trigger This Bug

| Pattern | Example | Reason for `global::` |
|---------|---------|----------------------|
| Namespace conflict avoidance | `global::System.DateTime` | Disambiguate from local `System` class |
| Resource access | `global::System.Resources.ResourceManager` | Framework resource loading |
| Reflection | `global::System.Type.GetType()` | Type resolution |
| Interop | `global::System.Runtime.InteropServices` | P/Invoke declarations |

---

## Bug #2: C++ Namespace Separator

### Observed Error
```
Warning: Invalid external dependency key format: cpp:fn:System::FindHinstance:unresolved-reference:0-0
```

### Root Cause Analysis

1. **C++ Namespace Syntax**: Uses `::` for scope resolution
   ```cpp
   System::FindHinstance();
   Win32::MessageBox();
   std::vector<int>();
   ```

2. **Same Key Generation Issue** (`query_extractor.rs:668-671`):
   ```rust
   let to_key = format!(
       "{}:fn:{}:unresolved-reference:0-0",
       language,
       to  // Contains "System::FindHinstance"
   );
   ```

3. **Resulting Malformed Key**:
   ```
   cpp:fn:System::FindHinstance:unresolved-reference:0-0
   ```
   When split by `:`, produces 7 parts:
   - `cpp`
   - `fn`
   - `System`
   - `` (empty)
   - `FindHinstance`
   - `unresolved-reference`
   - `0-0`

### C++ Patterns That Trigger This Bug

| Pattern | Example | Context |
|---------|---------|---------|
| Namespace scoping | `std::cout`, `std::vector` | STL usage |
| Windows API | `System::FindHinstance` | Win32 calls |
| Static method calls | `ClassName::StaticMethod()` | Class-level calls |
| Nested namespaces | `boost::asio::ip::tcp` | Library APIs |
| Enum class access | `Color::Red` | C++11 scoped enums |

---

## Affected Code Locations

### 1. Key Generation (`crates/parseltongue-core/src/query_extractor.rs`)

```rust
// Line 631: External dependency name
format!("{}:module:{}:external-dependency-{}:0-0", language, item_name, crate_name)

// Line 634: Module reference
format!("{}:module:{}:0-0", language, to)

// Line 638: Fallback module
format!("{}:module:{}:0-0", language, to)

// Line 668-671: Unresolved reference (PRIMARY BUG LOCATION)
let to_key = format!(
    "{}:fn:{}:unresolved-reference:0-0",
    language,
    to  // <-- BUG: `to` may contain `::`
);
```

### 2. Key Parsing (`crates/pt01-folder-to-cozodb-streamer/src/external_dependency_handler.rs`)

```rust
// Line 207: Strict 5-part validation
let parts: Vec<&str> = key.split(':').collect();
if parts.len() != 5 {
    return Err(...);  // Fails for keys with ::
}

// Line 221-223: Component extraction
let language_str = parts[0];
let entity_type = parts[1].to_string();
let item_name = parts[2].to_string();  // Wrong if :: present
let file_path = parts[3];              // Wrong if :: present
```

---

## Impact Assessment

### Severity: **Medium**

- **Data Loss**: External dependency placeholders not created
- **Graph Incompleteness**: Orphaned edges with no target nodes
- **Query Failures**: Blast radius queries return incomplete results
- **User Experience**: Warning spam in console output

### Affected Languages

| Language | Affected | Separator Pattern |
|----------|----------|-------------------|
| C# | Yes | `global::`, `namespace::` |
| C++ | Yes | `::` everywhere |
| Rust | No | Uses `::` but in USE paths, not function names |
| Java | No | Uses `.` for packages |
| Python | No | Uses `.` for modules |
| Go | No | Uses `.` for packages |
| JavaScript | No | Uses `.` for objects |
| TypeScript | No | Uses `.` for namespaces |

### Frequency

- **C# codebases**: Moderate (global:: used for disambiguation)
- **C++ codebases**: High (:: is fundamental to the language)

---

## Architectural Analysis

### Why This Wasn't Caught Earlier

1. **Test fixtures used simple names**: No qualified names with `::`
2. **Rust-centric testing**: Rust uses `::` only in import paths
3. **Small codebase testing**: Parseltongue self-test doesn't exercise these patterns

### Design Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **Current: `:` delimiter** | Simple parsing, human-readable | Conflicts with `::` in names |
| **Alternative: `\|` delimiter** | No conflict with language syntax | Less readable, breaking change |
| **Alternative: JSON keys** | Fully flexible | Verbose, harder to read |
| **Alternative: Escape `:`** | Backwards compatible | Complex parsing logic |
| **Alternative: Sanitize names** | Minimal change, backwards compatible | Information loss (can't round-trip) |

---

## ISGL1 Key Format Specification Review

### Current Format (v2)
```
{language}:{entity_type}:{name}:{semantic_path}:T{timestamp}
```

### Assumptions That Break
1. **Assumption**: Entity names don't contain `:`
   - **Reality**: C#/C++ qualified names contain `::`

2. **Assumption**: 5 colon-separated parts
   - **Reality**: Variable parts when `::` present

### Format Constraints

The ISGL1 format must balance:
- Human readability (for debugging)
- Machine parseability (for queries)
- Uniqueness (for identity)
- Stability (across code changes)

---

## Questions to Consider Before Implementation

1. **Should we preserve the original qualified name?**
   - If sanitized, can we reverse it?
   - Does downstream code need the original form?

2. **Where should sanitization happen?**
   - At capture time (tree-sitter query)?
   - At key generation time (query_extractor)?
   - At parsing time (external_dependency_handler)?

3. **What about other problematic characters?**
   - `.` in qualified names (Java, Python)
   - `<>` in generics (C#, Java, C++)
   - `[]` in array types

4. **Should the ISGL1 format be revised?**
   - Use different delimiter?
   - Use encoding (URL-encode, base64)?
   - Use structured format (JSON)?

5. **Backwards compatibility considerations?**
   - Existing databases with old keys
   - API consumers expecting current format

---

## Recommended Investigation Steps

1. **Audit all key generation code paths** for potential `::` injection
2. **Review tree-sitter query captures** for C# and C++ to understand what raw values contain
3. **Test with real-world codebases** that heavily use namespaces
4. **Benchmark parsing overhead** of different sanitization approaches
5. **Document ISGL1 key format specification** with explicit character constraints

---

## Related Files for Deep Dive

| File | Relevance |
|------|-----------|
| `crates/parseltongue-core/src/query_extractor.rs` | Key generation logic |
| `crates/pt01-folder-to-cozodb-streamer/src/external_dependency_handler.rs` | Key parsing logic |
| `dependency_queries/c_sharp.scm` | C# tree-sitter queries |
| `dependency_queries/cpp.scm` | C++ tree-sitter queries |
| `crates/parseltongue-core/src/isgl1_v2.rs` | ISGL1 format specification |

---

## Test Cases Needed

```rust
// C# global:: prefix
"global::System.Resources.ResourceManager"

// C# nested namespace
"Microsoft::Win32::Registry"

// C++ namespace
"std::vector"
"System::FindHinstance"
"boost::asio::ip::tcp::socket"

// C++ static method
"ClassName::StaticMethod"

// Edge cases
"::global" // Leading ::
"name::" // Trailing ::
"a::b::c::d::e" // Deep nesting
```

---

*Analysis Date: 2026-02-07*
*Version: v1.5.1-pre*
*Author: Claude Code Analysis*
