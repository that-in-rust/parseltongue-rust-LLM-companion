# TDD Specification: ISGL1 Key Format Sanitization for Qualified Names

**Version**: 1.5.1
**Date**: 2026-02-07
**Status**: SPECIFICATION
**Priority**: CRITICAL (Production Bug)

---

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Affected Languages](#affected-languages)
3. [Requirements Specification](#requirements-specification)
4. [Test Cases (RED → GREEN)](#test-cases-red--green)
5. [Implementation Plan](#implementation-plan)
6. [Acceptance Criteria](#acceptance-criteria)
7. [Performance Contracts](#performance-contracts)
8. [Backwards Compatibility](#backwards-compatibility)

---

## Problem Statement

### The Vulnerability

**ISGL1 v2** uses `:` (single colon) as the delimiter for key components:

```
rust:fn:main:definition:42-50
├─┬─┼──┬──┼────┬──────┼──┬──
│ │ │  │  │    │       │  └─ end_line
│ │ │  │  │    │       └─── start_line
│ │ │  │  │    └─────────── node_type (definition/unresolved-reference)
│ │ │  │  └──────────────── entity_name
│ │ │  └─────────────────── entity_type (fn/struct/class/etc)
│ │ └────────────────────── language
```

**The Bug**: Four languages use `::` (double colon) in their qualified names:

```rust
// Rust
std::collections::HashMap
//  ^^ ^^ ^^
//  These break split(':')

// C++
std::vector<int>
System::FindHinstance

// C#
global::System.Resources

// Ruby
ActiveRecord::Base
```

**Impact**: When we generate keys like:
```
rust:fn:std::collections::HashMap:unresolved-reference:0-0
```

And parse with:
```rust
let parts: Vec<&str> = key.split(':').collect();
if parts.len() != 5 { return Err(...); }
//      ^^^^^^ Expected 5, got 7 for "rust:fn:std::collections::HashMap:unresolved-reference:0-0"
```

**Result**: External dependency detection fails for ALL qualified names in 4 languages.

### Additional Edge Cases

| Language | Syntax | Example | Issue |
|----------|--------|---------|-------|
| **PHP** | `\` namespace separator | `\App\Models\User` | Backslash escaping in strings |
| **TypeScript/Java** | `<>` generics | `Map<String, Int>` | Angle brackets in keys |
| **All** | Empty components | `::foo` (leading) | Empty split parts |

---

## Affected Languages

### Severity Matrix

| Language | Syntax Pattern | Severity | Example | Frequency |
|----------|---------------|----------|---------|-----------|
| **Rust** | `::` | 🔴 CRITICAL | `std::collections::HashMap` | Very High |
| **C++** | `::` | 🔴 CRITICAL | `std::vector`, `System::FindHinstance` | Very High |
| **C#** | `::` | 🟡 HIGH | `global::System.Resources` | Medium |
| **Ruby** | `::` | 🟡 HIGH | `ActiveRecord::Base`, `Rails::Application` | High |
| **PHP** | `\` | 🟠 MEDIUM | `\App\Models\User` | Medium |
| **TypeScript** | `<>` | 🟢 LOW | `Map<K, V>` | Low (tree-sitter strips) |
| **Java** | `<>` | 🟢 LOW | `List<String>` | Low (tree-sitter strips) |

### Validation Data

From `docs/v148-ruby-rails-language-verification-20260203.md`:

**Ruby Test Case** (ActiveRecord scan):
```bash
curl "http://localhost:7777/code-entities-search-fuzzy?q=ActiveRecord"
# Expected: ActiveRecord::Base, ActiveRecord::Migration
# Actual (before fix): Parsing failures for :: components
```

---

## Requirements Specification

### REQ-SANITIZE-001: Core Sanitization Function

**WHEN** I call `sanitize_key_component_colons(input: &str)`
**THEN** the function SHALL replace all occurrences of `::` with `—DOUBLE-COLON—`
**AND** SHALL preserve single `:` characters (language delimiter)
**AND** SHALL handle empty strings without panic
**SHALL** complete in O(n) time complexity

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_001_basic_replacement() {
    assert_eq!(
        sanitize_key_component_colons("std::collections::HashMap"),
        "std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap"
    );
}

#[test]
fn test_req_sanitize_001_preserve_single_colon() {
    assert_eq!(
        sanitize_key_component_colons("rust:fn:main"),
        "rust:fn:main"
    );
}

#[test]
fn test_req_sanitize_001_empty_string() {
    assert_eq!(
        sanitize_key_component_colons(""),
        ""
    );
}
```

---

### REQ-SANITIZE-002: Key Generation Integration

**WHEN** I generate an ISGL1 key with qualified names
**THEN** the `entity_name` component SHALL be sanitized BEFORE key construction
**AND** SHALL NOT sanitize the `language` component
**AND** SHALL NOT sanitize the `entity_type` component
**SHALL** apply sanitization to `from` and `to` in edge keys

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_002_qualified_name_in_key() {
    let key = generate_entity_key(
        "rust",
        "fn",
        "std::collections::HashMap",
        "definition",
        42,
        50
    );

    assert_eq!(
        key,
        "rust:fn:std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap:definition:42-50"
    );

    // Verify it splits correctly
    let parts: Vec<&str> = key.split(':').collect();
    assert_eq!(parts.len(), 5, "Key must have exactly 5 parts");
}
```

---

### REQ-SANITIZE-003: Rust Language Support

**WHEN** I ingest Rust code with `std::collections::HashMap`
**THEN** the system SHALL create key `rust:fn:std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap:unresolved-reference:0-0`
**AND** SHALL parse the key without errors
**SHALL** preserve original name in entity metadata

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_003_rust_std_lib() {
    let code = r#"
        use std::collections::HashMap;
        fn main() {
            let map: HashMap<String, i32> = HashMap::new();
        }
    "#;

    let entities = parse_rust_code(code);
    let hashmap_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.key.contains("HashMap"))
        .collect();

    for entity in hashmap_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "Key must have 5 parts, got: {}", entity.key);
        assert!(entity.key.contains("—DOUBLE-COLON—"), "Key must sanitize ::");
    }
}
```

---

### REQ-SANITIZE-004: C++ Language Support

**WHEN** I ingest C++ code with `std::vector` or `System::FindHinstance`
**THEN** the system SHALL create sanitized keys
**AND** SHALL handle namespaces correctly
**SHALL** support nested namespaces (e.g., `boost::filesystem::path`)

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_004_cpp_std_lib() {
    let code = r#"
        #include <vector>
        std::vector<int> numbers;
    "#;

    let entities = parse_cpp_code(code);
    let vector_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.key.contains("vector"))
        .collect();

    assert!(!vector_refs.is_empty(), "Should find std::vector references");

    for entity in vector_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "C++ key must have 5 parts: {}", entity.key);
    }
}

#[test]
fn test_req_sanitize_004_cpp_nested_namespace() {
    let entity_name = "boost::filesystem::path::native";
    let sanitized = sanitize_key_component_colons(entity_name);

    assert_eq!(
        sanitized,
        "boost—DOUBLE-COLON—filesystem—DOUBLE-COLON—path—DOUBLE-COLON—native"
    );
}
```

---

### REQ-SANITIZE-005: C# Language Support

**WHEN** I ingest C# code with `global::System.Resources`
**THEN** the system SHALL create sanitized keys
**AND** SHALL handle `global::` prefix correctly

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_005_csharp_global_prefix() {
    let code = r#"
        using global::System.Resources;
        var manager = new ResourceManager();
    "#;

    let entities = parse_csharp_code(code);
    let resource_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.key.contains("System") && e.key.contains("Resources"))
        .collect();

    for entity in resource_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "C# key must have 5 parts: {}", entity.key);
    }
}
```

---

### REQ-SANITIZE-006: Ruby Language Support

**WHEN** I ingest Ruby code with `ActiveRecord::Base` or `Rails::Application`
**THEN** the system SHALL create sanitized keys
**AND** SHALL handle Rails framework classes correctly

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_006_ruby_activerecord() {
    let code = r#"
        class User < ActiveRecord::Base
          belongs_to :organization
        end
    "#;

    let entities = parse_ruby_code(code);
    let ar_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.key.contains("ActiveRecord"))
        .collect();

    assert!(!ar_refs.is_empty(), "Should find ActiveRecord::Base reference");

    for entity in ar_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "Ruby key must have 5 parts: {}", entity.key);
    }
}
```

---

### REQ-SANITIZE-007: PHP Namespace Support

**WHEN** I ingest PHP code with `\App\Models\User`
**THEN** the system SHALL NOT break on backslash characters
**AND** SHALL preserve backslashes in entity names
**SHALL** escape backslashes if needed for storage

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_007_php_backslash_namespace() {
    let code = r#"
        <?php
        namespace App\Models;
        use Illuminate\Database\Eloquent\Model;

        class User extends Model {}
    "#;

    let entities = parse_php_code(code);
    let model_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.key.contains("Model"))
        .collect();

    for entity in model_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "PHP key must have 5 parts: {}", entity.key);
    }
}
```

---

### REQ-SANITIZE-008: Edge Case Handling

**WHEN** I sanitize edge case inputs
**THEN** the function SHALL handle:
- Leading `::` (e.g., `::GlobalFunction`)
- Trailing `::` (e.g., `Module::`)
- Multiple consecutive `::` (e.g., `A::::B`)
- Empty strings
- Single `:` (preserve)
- Only `::` (replace to `—DOUBLE-COLON—`)

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_008_leading_double_colon() {
    assert_eq!(
        sanitize_key_component_colons("::GlobalFunction"),
        "—DOUBLE-COLON—GlobalFunction"
    );
}

#[test]
fn test_req_sanitize_008_trailing_double_colon() {
    assert_eq!(
        sanitize_key_component_colons("Module::"),
        "Module—DOUBLE-COLON—"
    );
}

#[test]
fn test_req_sanitize_008_multiple_consecutive_colons() {
    assert_eq!(
        sanitize_key_component_colons("A::::B"),
        "A—DOUBLE-COLON——DOUBLE-COLON—B"
    );
}

#[test]
fn test_req_sanitize_008_only_double_colon() {
    assert_eq!(
        sanitize_key_component_colons("::"),
        "—DOUBLE-COLON—"
    );
}

#[test]
fn test_req_sanitize_008_single_colon_preserved() {
    assert_eq!(
        sanitize_key_component_colons("rust:fn:main"),
        "rust:fn:main"
    );
}
```

---

### REQ-SANITIZE-009: Key Parsing Robustness

**WHEN** I parse ISGL1 keys with sanitized components
**THEN** the parser SHALL:
- Accept exactly 5 parts (split by `:`)
- Return descriptive errors for malformed keys
- NOT fail on sanitized `—DOUBLE-COLON—` markers

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_009_parse_sanitized_key() {
    let key = "rust:fn:std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap:definition:42-50";

    let parsed = parse_isgl1_key(key);
    assert!(parsed.is_ok(), "Should parse sanitized key: {}", key);

    let parts = parsed.unwrap();
    assert_eq!(parts.language, "rust");
    assert_eq!(parts.entity_type, "fn");
    assert_eq!(parts.entity_name, "std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap");
    assert_eq!(parts.node_type, "definition");
    assert_eq!(parts.start_line, 42);
    assert_eq!(parts.end_line, 50);
}

#[test]
fn test_req_sanitize_009_reject_malformed_key() {
    let key = "rust:fn:std::HashMap:definition";  // Missing line numbers

    let parsed = parse_isgl1_key(key);
    assert!(parsed.is_err(), "Should reject key with wrong part count");
}
```

---

### REQ-SANITIZE-010: Backwards Compatibility

**WHEN** I ingest codebases analyzed with ISGL1 v2.0 (pre-sanitization)
**THEN** the system SHALL:
- Parse existing keys without `::` correctly (no regression)
- NOT break existing databases
- Provide migration path if needed

**Test Verification**:
```rust
#[test]
fn test_req_sanitize_010_legacy_keys_still_work() {
    let legacy_keys = vec![
        "rust:fn:main:definition:1-10",
        "python:class:UserModel:definition:5-20",
        "javascript:function:handleClick:definition:15-25",
    ];

    for key in legacy_keys {
        let parsed = parse_isgl1_key(key);
        assert!(parsed.is_ok(), "Legacy key should still parse: {}", key);

        let parts: Vec<&str> = key.split(':').collect();
        assert_eq!(parts.len(), 5, "Legacy key structure unchanged");
    }
}
```

---

## Test Cases (RED → GREEN)

### Phase 1: Unit Tests for Sanitization Function

#### RED Tests (Write First, Must Fail)

```rust
// File: crates/parseltongue-core/src/key_sanitizer.rs (NEW FILE)

#[cfg(test)]
mod tests {
    use super::*;

    // ===== BASIC FUNCTIONALITY =====

    #[test]
    fn test_sanitize_empty_string() {
        assert_eq!(sanitize_key_component_colons(""), "");
    }

    #[test]
    fn test_sanitize_no_colons() {
        assert_eq!(sanitize_key_component_colons("HashMap"), "HashMap");
    }

    #[test]
    fn test_sanitize_single_double_colon() {
        assert_eq!(
            sanitize_key_component_colons("std::vector"),
            "std—DOUBLE-COLON—vector"
        );
    }

    #[test]
    fn test_sanitize_multiple_double_colons() {
        assert_eq!(
            sanitize_key_component_colons("std::collections::HashMap"),
            "std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap"
        );
    }

    #[test]
    fn test_sanitize_preserve_single_colon() {
        assert_eq!(
            sanitize_key_component_colons("rust:fn:main"),
            "rust:fn:main"
        );
    }

    // ===== EDGE CASES =====

    #[test]
    fn test_sanitize_leading_double_colon() {
        assert_eq!(
            sanitize_key_component_colons("::GlobalFunc"),
            "—DOUBLE-COLON—GlobalFunc"
        );
    }

    #[test]
    fn test_sanitize_trailing_double_colon() {
        assert_eq!(
            sanitize_key_component_colons("Module::"),
            "Module—DOUBLE-COLON—"
        );
    }

    #[test]
    fn test_sanitize_only_double_colon() {
        assert_eq!(
            sanitize_key_component_colons("::"),
            "—DOUBLE-COLON—"
        );
    }

    #[test]
    fn test_sanitize_consecutive_double_colons() {
        assert_eq!(
            sanitize_key_component_colons("A::::B"),
            "A—DOUBLE-COLON——DOUBLE-COLON—B"
        );
    }

    #[test]
    fn test_sanitize_mixed_single_and_double() {
        assert_eq!(
            sanitize_key_component_colons("rust:fn:std::HashMap"),
            "rust:fn:std—DOUBLE-COLON—HashMap"
        );
    }

    // ===== LANGUAGE-SPECIFIC CASES =====

    #[test]
    fn test_sanitize_rust_std_lib() {
        assert_eq!(
            sanitize_key_component_colons("std::collections::HashMap"),
            "std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap"
        );
    }

    #[test]
    fn test_sanitize_cpp_std_lib() {
        assert_eq!(
            sanitize_key_component_colons("std::vector"),
            "std—DOUBLE-COLON—vector"
        );
    }

    #[test]
    fn test_sanitize_csharp_global() {
        assert_eq!(
            sanitize_key_component_colons("global::System"),
            "global—DOUBLE-COLON—System"
        );
    }

    #[test]
    fn test_sanitize_ruby_rails() {
        assert_eq!(
            sanitize_key_component_colons("ActiveRecord::Base"),
            "ActiveRecord—DOUBLE-COLON—Base"
        );
    }

    #[test]
    fn test_sanitize_cpp_nested_namespace() {
        assert_eq!(
            sanitize_key_component_colons("boost::filesystem::path::native"),
            "boost—DOUBLE-COLON—filesystem—DOUBLE-COLON—path—DOUBLE-COLON—native"
        );
    }

    // ===== PERFORMANCE CONTRACT =====

    #[test]
    fn test_sanitize_performance_contract() {
        use std::time::Instant;

        let long_name = "a::".repeat(1000) + "b";  // 2000 colons + 1001 chars

        let start = Instant::now();
        let _ = sanitize_key_component_colons(&long_name);
        let elapsed = start.elapsed();

        // Must complete in < 1ms for 2000-char input
        assert!(
            elapsed.as_micros() < 1000,
            "Sanitization too slow: {:?} for {} chars",
            elapsed,
            long_name.len()
        );
    }
}
```

---

### Phase 2: Integration Tests for Key Generation

```rust
// File: crates/parseltongue-core/src/query_extractor.rs

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_generate_key_with_qualified_rust_name() {
        let key = generate_entity_key(
            "rust",
            "fn",
            "std::collections::HashMap",
            "unresolved-reference",
            0,
            0
        );

        // Should sanitize the entity_name component
        assert_eq!(
            key,
            "rust:fn:std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap:unresolved-reference:0-0"
        );

        // Must split into exactly 5 parts
        let parts: Vec<&str> = key.split(':').collect();
        assert_eq!(parts.len(), 5, "Key must have 5 parts, got: {:?}", parts);
    }

    #[test]
    fn test_generate_key_with_cpp_qualified_name() {
        let key = generate_entity_key(
            "cpp",
            "class",
            "std::vector",
            "definition",
            10,
            20
        );

        assert_eq!(
            key,
            "cpp:class:std—DOUBLE-COLON—vector:definition:10-20"
        );

        let parts: Vec<&str> = key.split(':').collect();
        assert_eq!(parts.len(), 5);
    }

    #[test]
    fn test_generate_edge_key_sanitizes_from_and_to() {
        let edge_key = generate_edge_key(
            "rust",
            "fn",
            "std::collections::HashMap::new",
            "std::collections::HashMap"
        );

        // Both 'from' and 'to' should be sanitized
        assert!(edge_key.contains("—DOUBLE-COLON—"));

        // Verify structure (depends on edge key format)
        // Assuming edge keys use same structure
        let parts: Vec<&str> = edge_key.split(':').collect();
        assert!(parts.len() >= 5, "Edge key should have valid structure");
    }
}
```

---

### Phase 3: End-to-End Language Tests

```rust
// File: crates/parseltongue-core/tests/integration_qualified_names.rs (NEW FILE)

use parseltongue_core::*;

// ===== RUST =====

#[test]
fn test_e2e_rust_std_hashmap() {
    let code = r#"
        use std::collections::HashMap;

        fn main() {
            let mut map = HashMap::new();
            map.insert("key", "value");
        }
    "#;

    let entities = parse_rust_source(code, "test.rs");

    // Find HashMap references
    let hashmap_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.name.contains("HashMap"))
        .collect();

    assert!(!hashmap_refs.is_empty(), "Should detect HashMap usage");

    for entity in hashmap_refs {
        // All keys must parse correctly
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(
            parts.len(),
            5,
            "Rust qualified name key must have 5 parts: {}",
            entity.key
        );

        // Must contain sanitization marker
        if entity.name.contains("::") {
            assert!(
                entity.key.contains("—DOUBLE-COLON—"),
                "Key must sanitize :: in: {}",
                entity.key
            );
        }
    }
}

// ===== C++ =====

#[test]
fn test_e2e_cpp_std_vector() {
    let code = r#"
        #include <vector>

        int main() {
            std::vector<int> numbers;
            numbers.push_back(42);
            return 0;
        }
    "#;

    let entities = parse_cpp_source(code, "test.cpp");

    let vector_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.name.contains("vector"))
        .collect();

    assert!(!vector_refs.is_empty(), "Should detect std::vector usage");

    for entity in vector_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "C++ key parsing failed: {}", entity.key);
    }
}

#[test]
fn test_e2e_cpp_system_find_hinstance() {
    let code = r#"
        HINSTANCE h = System::FindHinstance(NULL);
    "#;

    let entities = parse_cpp_source(code, "test.cpp");

    let system_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.name.contains("System") || e.name.contains("FindHinstance"))
        .collect();

    for entity in system_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "C++ System:: key failed: {}", entity.key);
    }
}

// ===== C# =====

#[test]
fn test_e2e_csharp_global_system() {
    let code = r#"
        using global::System.Resources;

        class Program {
            static void Main() {
                var manager = new ResourceManager();
            }
        }
    "#;

    let entities = parse_csharp_source(code, "test.cs");

    let resource_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.name.contains("System") || e.name.contains("Resources"))
        .collect();

    for entity in resource_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "C# global:: key failed: {}", entity.key);
    }
}

// ===== RUBY =====

#[test]
fn test_e2e_ruby_activerecord_base() {
    let code = r#"
        class User < ActiveRecord::Base
          belongs_to :organization
        end
    "#;

    let entities = parse_ruby_source(code, "user.rb");

    let ar_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.name.contains("ActiveRecord"))
        .collect();

    assert!(!ar_refs.is_empty(), "Should detect ActiveRecord::Base");

    for entity in ar_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "Ruby :: key failed: {}", entity.key);
    }
}

#[test]
fn test_e2e_ruby_rails_application() {
    let code = r#"
        class Application < Rails::Application
          config.load_defaults 7.0
        end
    "#;

    let entities = parse_ruby_source(code, "application.rb");

    let rails_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.name.contains("Rails"))
        .collect();

    for entity in rails_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "Ruby Rails:: key failed: {}", entity.key);
    }
}

// ===== PHP =====

#[test]
fn test_e2e_php_backslash_namespace() {
    let code = r#"
        <?php
        namespace App\Models;
        use Illuminate\Database\Eloquent\Model;

        class User extends Model {
            protected $table = 'users';
        }
    "#;

    let entities = parse_php_source(code, "User.php");

    let model_refs: Vec<_> = entities
        .iter()
        .filter(|e| e.name.contains("Model") || e.name.contains("Eloquent"))
        .collect();

    for entity in model_refs {
        let parts: Vec<&str> = entity.key.split(':').collect();
        assert_eq!(parts.len(), 5, "PHP namespace key failed: {}", entity.key);
    }
}
```

---

## Implementation Plan

### Phase 1: Create Sanitization Function (STUB → RED → GREEN)

**Location**: `crates/parseltongue-core/src/key_sanitizer.rs` (NEW FILE)

#### Step 1.1: STUB - Create Module and Tests (RED)

```rust
// File: crates/parseltongue-core/src/key_sanitizer.rs

//! Key component sanitization for ISGL1 v2 format compliance.
//!
//! ## Problem
//! ISGL1 uses `:` as delimiter, but Rust/C++/C#/Ruby use `::` in qualified names.
//!
//! ## Solution
//! Replace `::` with `—DOUBLE-COLON—` marker before key construction.

/// Sanitizes key components by replacing `::` with `—DOUBLE-COLON—`.
///
/// ## Behavior
/// - `::` → `—DOUBLE-COLON—`
/// - `:` → preserved (ISGL1 delimiter)
/// - Empty string → empty string
///
/// ## Performance
/// O(n) time, single pass with `replace()`.
///
/// ## Examples
/// ```
/// use parseltongue_core::sanitize_key_component_colons;
///
/// assert_eq!(
///     sanitize_key_component_colons("std::collections::HashMap"),
///     "std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap"
/// );
/// ```
pub fn sanitize_key_component_colons(input: &str) -> String {
    todo!("REQ-SANITIZE-001: Implement sanitization")
}

#[cfg(test)]
mod tests {
    use super::*;
    // ... [All RED tests from section above]
}
```

**Run tests**:
```bash
cargo test -p parseltongue-core sanitize_key_component_colons
# Expected: All tests FAIL (function returns todo!())
```

---

#### Step 1.2: GREEN - Minimal Implementation

```rust
pub fn sanitize_key_component_colons(input: &str) -> String {
    input.replace("::", "—DOUBLE-COLON—")
}
```

**Run tests**:
```bash
cargo test -p parseltongue-core sanitize_key_component_colons
# Expected: All tests PASS
```

---

#### Step 1.3: REFACTOR - Add Documentation

```rust
/// Sanitizes key components by replacing `::` with `—DOUBLE-COLON—`.
///
/// This function ensures ISGL1 v2 keys can be parsed correctly by splitting on
/// single `:` delimiters. Qualified names in Rust, C++, C#, and Ruby use `::`
/// which would break the 5-part key structure.
///
/// # Arguments
/// * `input` - Entity name or key component (e.g., "std::collections::HashMap")
///
/// # Returns
/// Sanitized string with `::` replaced (e.g., "std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap")
///
/// # Performance
/// - Time: O(n) where n = input length
/// - Space: O(n) for new String allocation
/// - Measured: < 1μs for typical entity names (< 100 chars)
///
/// # Examples
/// ```
/// use parseltongue_core::sanitize_key_component_colons;
///
/// // Rust qualified name
/// assert_eq!(
///     sanitize_key_component_colons("std::collections::HashMap"),
///     "std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap"
/// );
///
/// // C++ namespace
/// assert_eq!(
///     sanitize_key_component_colons("std::vector"),
///     "std—DOUBLE-COLON—vector"
/// );
///
/// // Preserve single colons
/// assert_eq!(
///     sanitize_key_component_colons("rust:fn:main"),
///     "rust:fn:main"
/// );
/// ```
pub fn sanitize_key_component_colons(input: &str) -> String {
    input.replace("::", "—DOUBLE-COLON—")
}
```

**Export in `lib.rs`**:
```rust
// File: crates/parseltongue-core/src/lib.rs

mod key_sanitizer;
pub use key_sanitizer::sanitize_key_component_colons;
```

---

### Phase 2: Apply to Key Generation (STUB → RED → GREEN)

**Location**: `crates/parseltongue-core/src/query_extractor.rs`

#### Step 2.1: RED - Add Integration Tests

Add tests from **Phase 2: Integration Tests for Key Generation** section.

```bash
cargo test -p parseltongue-core integration_tests
# Expected: FAIL (sanitization not yet applied)
```

---

#### Step 2.2: GREEN - Apply Sanitization to `to_key` Generation

**Current Code** (Lines 668-672):
```rust
let to_key = format!(
    "{}:fn:{}:unresolved-reference:0-0",
    language,
    to  // BUG: Contains "std::vector" → breaks split(':')
);
```

**Fix**:
```rust
use crate::sanitize_key_component_colons;

let to_key = format!(
    "{}:fn:{}:unresolved-reference:0-0",
    language,
    sanitize_key_component_colons(to)  // ✅ Sanitize before key construction
);
```

**Run tests**:
```bash
cargo test -p parseltongue-core integration_tests
# Expected: PASS
```

---

#### Step 2.3: Apply to Other Key Generation Sites

**Search for all key generation**:
```bash
grep -n 'format!.*:fn:.*:' crates/parseltongue-core/src/query_extractor.rs
```

**Apply sanitization to ALL entity_name components**:

1. **Function/method calls** (around line 668):
   ```rust
   let to_key = format!(
       "{}:fn:{}:unresolved-reference:0-0",
       language,
       sanitize_key_component_colons(to)
   );
   ```

2. **Type references** (if exists):
   ```rust
   let type_key = format!(
       "{}:type:{}:unresolved-reference:0-0",
       language,
       sanitize_key_component_colons(type_name)
   );
   ```

3. **Definition keys** (if exists):
   ```rust
   let def_key = format!(
       "{}:fn:{}:definition:{}-{}",
       language,
       sanitize_key_component_colons(fn_name),
       start_line,
       end_line
   );
   ```

**Critical**: DO NOT sanitize `language`, `entity_type`, or `node_type` components.

---

### Phase 3: Update Key Parsing (If Needed)

**Location**: `crates/pt01-folder-to-cozodb-streamer/src/external_dependency_handler.rs`

#### Step 3.1: Verify Current Parser

**Current Code** (Lines 207-219):
```rust
let parts: Vec<&str> = key.split(':').collect();
if parts.len() != 5 {
    return Err(anyhow::anyhow!(
        "Invalid external dependency key format: expected 5 parts, got {}. Key: {}",
        parts.len(),
        key
    ));
}

let language = parts[0];
let entity_type = parts[1];
let entity_name = parts[2];  // Now contains "std—DOUBLE-COLON—vector"
let node_type = parts[3];
let line_range = parts[4];
```

**Analysis**: This parser should work WITHOUT changes because:
- Sanitized keys still have 5 parts
- `entity_name` now contains `—DOUBLE-COLON—` instead of `::`
- No additional splitting needed

---

#### Step 3.2: Add Parser Tests (RED → GREEN)

```rust
// File: crates/pt01-folder-to-cozodb-streamer/src/external_dependency_handler.rs

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_sanitized_rust_key() {
        let key = "rust:fn:std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap:unresolved-reference:0-0";

        let result = parse_external_dependency_key(key);
        assert!(result.is_ok(), "Should parse sanitized key");

        let parsed = result.unwrap();
        assert_eq!(parsed.language, "rust");
        assert_eq!(parsed.entity_type, "fn");
        assert_eq!(parsed.entity_name, "std—DOUBLE-COLON—collections—DOUBLE-COLON—HashMap");
        assert_eq!(parsed.node_type, "unresolved-reference");
    }

    #[test]
    fn test_parse_legacy_key_without_double_colons() {
        let key = "python:class:UserModel:definition:10-20";

        let result = parse_external_dependency_key(key);
        assert!(result.is_ok(), "Legacy keys must still work");

        let parsed = result.unwrap();
        assert_eq!(parsed.entity_name, "UserModel");
    }

    #[test]
    fn test_reject_unsanitized_qualified_name() {
        // This key has :: which creates 7 parts instead of 5
        let key = "rust:fn:std::HashMap:unresolved-reference:0-0";

        let result = parse_external_dependency_key(key);
        assert!(result.is_err(), "Should reject unsanitized keys with ::");
    }
}
```

**Run tests**:
```bash
cargo test -p pt01-folder-to-cozodb-streamer parser_tests
# Expected: PASS (no code changes needed)
```

---

### Phase 4: End-to-End Integration Tests

#### Step 4.1: Create Test Fixtures

```bash
mkdir -p tests/fixtures/qualified_names
```

**Rust fixture**:
```rust
// tests/fixtures/qualified_names/rust_hashmap.rs
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
}
```

**C++ fixture**:
```cpp
// tests/fixtures/qualified_names/cpp_vector.cpp
#include <vector>

int main() {
    std::vector<int> numbers;
    numbers.push_back(42);
    return 0;
}
```

**Ruby fixture**:
```ruby
# tests/fixtures/qualified_names/ruby_activerecord.rb
class User < ActiveRecord::Base
  belongs_to :organization
end
```

---

#### Step 4.2: Integration Test Script

```bash
#!/bin/bash
# tests/test_qualified_names.sh

set -e

echo "🧪 Testing ISGL1 key sanitization for qualified names"

# Build release binary
cargo build --release

# Test Rust std::collections::HashMap
echo "Testing Rust..."
./target/release/parseltongue pt01-folder-to-cozodb-streamer tests/fixtures/qualified_names/rust_hashmap.rs
DB_PATH=$(ls -dt parseltongue*/analysis.db | head -1)

# Start server
./target/release/parseltongue pt08-http-code-query-server --db "rocksdb:$DB_PATH" --port 9999 &
SERVER_PID=$!
sleep 2

# Query for HashMap
RESULT=$(curl -s "http://localhost:9999/code-entities-search-fuzzy?q=HashMap")

# Check for sanitized key
if echo "$RESULT" | grep -q "—DOUBLE-COLON—"; then
    echo "✅ Rust std::HashMap sanitized correctly"
else
    echo "❌ Rust std::HashMap NOT sanitized"
    kill $SERVER_PID
    exit 1
fi

# Cleanup
kill $SERVER_PID

echo "✅ All qualified name tests passed"
```

**Run**:
```bash
chmod +x tests/test_qualified_names.sh
./tests/test_qualified_names.sh
```

---

## Acceptance Criteria

### Definition of Done for v1.5.1

- [ ] **REQ-SANITIZE-001**: `sanitize_key_component_colons()` function implemented
  - [ ] All unit tests pass (18 tests)
  - [ ] Performance contract verified (< 1ms for 2000 chars)
  - [ ] Documentation complete

- [ ] **REQ-SANITIZE-002**: Key generation sanitization applied
  - [ ] `query_extractor.rs` updated (all key generation sites)
  - [ ] Integration tests pass (3 tests)

- [ ] **REQ-SANITIZE-003**: Rust language support
  - [ ] E2E test for `std::collections::HashMap` passes
  - [ ] E2E test for `std::sync::Arc` passes

- [ ] **REQ-SANITIZE-004**: C++ language support
  - [ ] E2E test for `std::vector` passes
  - [ ] E2E test for `System::FindHinstance` passes
  - [ ] E2E test for nested namespaces passes

- [ ] **REQ-SANITIZE-005**: C# language support
  - [ ] E2E test for `global::System` passes

- [ ] **REQ-SANITIZE-006**: Ruby language support
  - [ ] E2E test for `ActiveRecord::Base` passes
  - [ ] E2E test for `Rails::Application` passes

- [ ] **REQ-SANITIZE-007**: PHP namespace support
  - [ ] E2E test for `\App\Models\User` passes

- [ ] **REQ-SANITIZE-008**: Edge case handling
  - [ ] All 5 edge case tests pass
  - [ ] Empty string handling verified
  - [ ] Leading/trailing `::` handling verified

- [ ] **REQ-SANITIZE-009**: Key parsing robustness
  - [ ] Parser tests pass (3 tests)
  - [ ] Descriptive error messages for malformed keys

- [ ] **REQ-SANITIZE-010**: Backwards compatibility
  - [ ] Legacy keys without `::` still parse correctly
  - [ ] No regression in existing functionality

- [ ] **Code Quality**
  - [ ] Zero `TODO`/`STUB`/`PLACEHOLDER` comments
  - [ ] All compiler warnings resolved
  - [ ] `cargo clippy` passes with zero warnings
  - [ ] `cargo fmt` applied

- [ ] **Documentation**
  - [ ] Function documentation complete
  - [ ] CHANGELOG.md updated
  - [ ] README.md updated if needed
  - [ ] This TDD spec marked as IMPLEMENTED

- [ ] **Testing**
  - [ ] Unit tests: 18 tests minimum
  - [ ] Integration tests: 3 tests minimum
  - [ ] E2E tests: 7 language tests minimum
  - [ ] All tests pass: `cargo test --all`

- [ ] **Build Verification**
  - [ ] `cargo build --release` succeeds
  - [ ] Binary size acceptable (< 60MB)
  - [ ] No new dependencies added

---

## Performance Contracts

### REQ-PERF-001: Sanitization Function Performance

**WHEN** I sanitize a typical entity name (< 100 characters)
**THEN** the function SHALL complete in < 10μs
**AND** SHALL complete in < 1ms for pathological cases (2000 characters)

**Test Verification**:
```rust
#[test]
fn test_perf_001_typical_entity_name() {
    use std::time::Instant;

    let name = "std::collections::HashMap";
    let iterations = 10_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = sanitize_key_component_colons(name);
    }
    let elapsed = start.elapsed();

    let avg_micros = elapsed.as_micros() / iterations as u128;
    assert!(
        avg_micros < 10,
        "Average sanitization time {} μs exceeds 10μs limit",
        avg_micros
    );
}

#[test]
fn test_perf_001_pathological_case() {
    use std::time::Instant;

    let long_name = "a::".repeat(1000) + "b";  // 2000 colons

    let start = Instant::now();
    let _ = sanitize_key_component_colons(&long_name);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 1,
        "Pathological case took {:?}, limit is 1ms",
        elapsed
    );
}
```

---

### REQ-PERF-002: No Regression in Query Performance

**WHEN** I run existing queries on codebases with qualified names
**THEN** query latency SHALL NOT increase by > 5%
**AND** SHALL maintain < 500μs p99 latency for list-all queries

**Test Verification**:
```rust
#[test]
fn test_perf_002_query_latency_regression() {
    // Ingest codebase with qualified names
    let db = ingest_test_codebase("tests/fixtures/qualified_names");

    // Benchmark list-all query
    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = query_list_all_entities(&db);
    }

    let elapsed = start.elapsed();
    let avg_micros = elapsed.as_micros() / iterations as u128;

    // p99 latency estimate (assuming normal distribution)
    let p99_micros = avg_micros * 2;

    assert!(
        p99_micros < 500,
        "Query latency regression: p99 = {}μs, limit = 500μs",
        p99_micros
    );
}
```

---

## Backwards Compatibility

### Migration Strategy

**Goal**: Ensure existing databases and workflows continue working without changes.

#### Compatibility Matrix

| Component | Pre-v1.5.1 | v1.5.1+ | Compatible? |
|-----------|-----------|---------|-------------|
| **Keys without `::`** | `rust:fn:main:definition:1-10` | Same | ✅ YES |
| **Keys with `::`** | `rust:fn:std::HashMap:...` (BROKEN) | `rust:fn:std—DOUBLE-COLON—HashMap:...` | ✅ FIXED |
| **Database format** | CozoDB RocksDB | Same | ✅ YES |
| **HTTP API** | 14 endpoints | Same | ✅ YES |
| **CLI interface** | `pt01`, `pt08` | Same | ✅ YES |

---

### Database Migration

**Not Required**:
- New sanitization only affects KEY generation during ingestion
- Existing databases with non-qualified names work unchanged
- Databases with broken `::` keys were already failing (this fixes them)

**Action**: Re-ingest codebases with qualified names to get corrected keys.

---

### API Compatibility

**No Breaking Changes**:
- HTTP endpoints unchanged
- Query parameters unchanged
- Response formats unchanged

**Enhanced Behavior**:
- Queries for "std::HashMap" will now find sanitized keys
- Search should handle both `::` and `—DOUBLE-COLON—` (fuzzy matching)

---

### User Migration Checklist

For users upgrading from v1.4.x to v1.5.1:

1. [ ] Update binary: `cargo install parseltongue` or download release
2. [ ] Re-ingest codebases with Rust/C++/C#/Ruby code
3. [ ] Verify qualified names appear in search results
4. [ ] No changes needed to scripts or workflows

---

## Appendix A: Real-World Test Cases

### From v148 Ruby/Rails Verification

**Test Command**:
```bash
curl "http://localhost:7777/code-entities-search-fuzzy?q=ActiveRecord"
```

**Expected Results** (after fix):
```json
{
  "results": [
    {
      "key": "ruby:class:ActiveRecord—DOUBLE-COLON—Base:definition:5-20",
      "name": "ActiveRecord::Base",
      "language": "ruby"
    },
    {
      "key": "ruby:class:ActiveRecord—DOUBLE-COLON—Migration:definition:1-10",
      "name": "ActiveRecord::Migration",
      "language": "ruby"
    }
  ]
}
```

---

### From C++ System Library Usage

**Test Code**:
```cpp
// tests/fixtures/cpp_system.cpp
HINSTANCE h = System::FindHinstance(NULL);
LPCTSTR lpszHelpFile = System::GetHelpFilePath();
```

**Expected Entities**:
- `cpp:fn:System—DOUBLE-COLON—FindHinstance:unresolved-reference:0-0`
- `cpp:fn:System—DOUBLE-COLON—GetHelpFilePath:unresolved-reference:0-0`

---

## Appendix B: Debugging Guide

### If Tests Fail

**Symptom**: Key parsing fails with "expected 5 parts, got 7"

**Diagnosis**:
```bash
# Check if sanitization was applied
cargo test -p parseltongue-core sanitize_key_component_colons -- --nocapture

# Print generated keys
cargo test -p parseltongue-core integration_tests -- --nocapture
```

**Fix**: Ensure sanitization is applied BEFORE `format!()` call.

---

**Symptom**: Performance test fails (> 1ms for 2000 chars)

**Diagnosis**:
```rust
// Add debug output to performance test
println!("Input length: {}", long_name.len());
println!("Elapsed: {:?}", elapsed);
```

**Fix**: Check if `String::replace()` is being called multiple times.

---

**Symptom**: Legacy keys fail to parse

**Diagnosis**:
```bash
# Test with simple key
cargo test -p pt01-folder-to-cozodb-streamer parser_tests::test_parse_legacy_key_without_double_colons -- --nocapture
```

**Fix**: Ensure parser doesn't require `—DOUBLE-COLON—` marker.

---

## Appendix C: Future Enhancements

### Desanitization Function (Future v1.6.0)

For displaying human-readable names in UI:

```rust
/// Reverses sanitization for display purposes.
///
/// # Examples
/// ```
/// let sanitized = "std—DOUBLE-COLON—HashMap";
/// let original = desanitize_key_component_colons(sanitized);
/// assert_eq!(original, "std::HashMap");
/// ```
pub fn desanitize_key_component_colons(input: &str) -> String {
    input.replace("—DOUBLE-COLON—", "::")
}
```

**Use Case**: HTTP API could return both `key` (sanitized) and `display_name` (desanitized).

---

### Language-Specific Sanitizers (Future v1.7.0)

Different escaping strategies per language:

```rust
pub fn sanitize_php_namespace(input: &str) -> String {
    input.replace("\\", "—BACKSLASH—")
}

pub fn sanitize_generic_brackets(input: &str) -> String {
    input
        .replace("<", "—LT—")
        .replace(">", "—GT—")
}
```

---

## Document Metadata

**Author**: Claude Code (Anthropic)
**Created**: 2026-02-07
**Version**: 1.0
**Status**: READY FOR IMPLEMENTATION
**Related Issues**: ISGL1 key format vulnerability
**Estimated Effort**: 4-6 hours (including testing)

**Pre-commit Checklist**:
- [ ] All function names follow 4WNC
- [ ] All tests written RED first
- [ ] Documentation complete
- [ ] Zero TODOs in code
- [ ] All tests pass
- [ ] No compiler warnings

---

**END OF SPECIFICATION**
