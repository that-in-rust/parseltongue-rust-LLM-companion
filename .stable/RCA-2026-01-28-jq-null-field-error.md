# Root Cause Analysis: jq "null and string cannot have containment checked" Error

**Date**: 2026-01-28
**Severity**: User Experience / Documentation
**Status**: RESOLVED
**Affected**: Users querying `/dependency-edges-list-all` with jq

---

## 1. Executive Summary

Users querying the Parseltongue HTTP API with jq encountered errors when filtering edges by target entity. The error message `null (null) and string ("useAuthStore") cannot have their containment checked` indicated that jq was receiving null values where strings were expected.

**Root Cause**: The jq queries used field name `.to` but the actual API field name is `.to_key`.

---

## 2. Error Symptoms

### Observed Errors

```bash
# User's query
curl -s "http://localhost:7777/dependency-edges-list-all" | jq '.data.edges | map(select(.to | contains("useAuthStore"))) | length'

# Error output
jq: error (at <stdin>:0): null (null) and string ("useAuthStore") cannot have their containment checked
```

### Additional Failed Queries

| Query Pattern | Error |
|--------------|-------|
| `select(.to \| contains("useAuth"))` | null and string cannot have containment checked |
| `select(.to \| contains("useSidebar"))` | null and string cannot have containment checked |
| `select(.to \| contains("useBroadcast"))` | null and string cannot have containment checked |

---

## 3. Root Cause Analysis

### 3.1 The Problem

The jq queries referenced a field named `.to` which **does not exist** in the API response. When jq accesses a non-existent field, it returns `null`. The `contains()` function cannot operate on `null` values, causing the error.

### 3.2 Actual API Response Structure

```json
{
  "from_key": "javascript:file:_Users_joy_dev_studio_app_api-keys_controller_js:1-1",
  "to_key": "javascript:module:useAuthStore:0-0",
  "edge_type": "Uses",
  "source_location": "./api-keys/controller.js:5"
}
```

**Note**: The field is `to_key`, NOT `to`.

### 3.3 Code Evidence

From `dependency_edges_list_handler.rs:40-45`:

```rust
#[derive(Debug, Serialize)]
pub struct EdgeDataPayloadItem {
    pub from_key: String,   // ← from_key
    pub to_key: String,     // ← to_key (NOT "to")
    pub edge_type: String,
    pub source_location: String,
}
```

### 3.4 Why This Design?

The `_key` suffix convention is intentional:

1. **Consistency**: All entity references use the `_key` suffix (`from_key`, `to_key`, `entity_key`)
2. **Clarity**: Distinguishes keys from display names
3. **CozoDB Schema**: Matches the underlying database schema:
   ```
   DependencyEdges{from_key, to_key, edge_type, source_location}
   ```

---

## 4. Correct Queries

### Fixed jq Queries

```bash
# WRONG - uses non-existent field .to
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '.data.edges | map(select(.to | contains("useAuth"))) | length'

# CORRECT - uses actual field .to_key
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '.data.edges | map(select(.to_key | contains("useAuth"))) | length'
```

### Additional Examples

```bash
# Find all edges TO a specific entity
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '[.data.edges[] | select(.to_key | contains("useAuthStore"))]'

# Find all edges FROM a specific file
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '[.data.edges[] | select(.from_key | contains("controller"))]'

# Count edges by type
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '.data.edges | group_by(.edge_type) | map({type: .[0].edge_type, count: length})'
```

---

## 5. Available API Fields

### Edge Response Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `from_key` | string | Source entity key | `javascript:fn:handleClick:_src_app_js:10-25` |
| `to_key` | string | Target entity key | `javascript:module:useAuth:0-0` |
| `edge_type` | string | Relationship type | `Uses`, `Calls`, `Imports` |
| `source_location` | string | File:line reference | `./src/app.js:15` |

### Entity Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `key` | string | Unique entity identifier |
| `name` | string | Human-readable name |
| `entity_type` | string | `fn`, `struct`, `class`, `module`, etc. |
| `language` | string | `rust`, `javascript`, `python`, etc. |
| `file_path` | string | Source file path |
| `start_line` | number | Starting line number |
| `end_line` | number | Ending line number |

---

## 6. Prevention Measures

### 6.1 For Users

1. **Check API documentation first**: Use `/api-reference-documentation-help` endpoint
2. **Inspect response structure**: Run query without filters first to see actual field names
3. **Use defensive jq**: Add null checks: `select(.to_key != null and (.to_key | contains("X")))`

### 6.2 For Documentation

- [ ] Add jq examples to README.md showing correct field names
- [ ] Add field name reference table to API documentation
- [ ] Consider adding `.to` as an alias for `.to_key` in future versions

---

## 7. Timeline

| Time | Event |
|------|-------|
| T+0 | User runs jq query with `.to` field |
| T+0 | jq returns null for non-existent field |
| T+0 | `contains()` fails on null, error displayed |
| T+5m | User attempts variations, all fail |
| T+10m | Investigation reveals field name mismatch |

---

## 8. Lessons Learned

1. **Field naming conventions matter**: The `_key` suffix, while consistent internally, may not be intuitive for new users
2. **Error messages are indirect**: jq's error doesn't indicate "field doesn't exist" - it says "null cannot have containment checked"
3. **Documentation gaps**: README examples didn't cover jq filtering patterns

---

## 9. Action Items

| Priority | Action | Owner | Status |
|----------|--------|-------|--------|
| P1 | Add jq examples with correct field names to README | - | TODO |
| P2 | Add field reference table to API docs | - | TODO |
| P3 | Consider adding `.to`/`.from` aliases | - | FUTURE |

---

## 10. Appendix: Quick Reference

### Correct Field Names

```
Edge fields:    from_key, to_key, edge_type, source_location
Entity fields:  key, name, entity_type, language, file_path, start_line, end_line
```

### jq Cheat Sheet for Parseltongue

```bash
# List all edges
curl -s http://localhost:7777/dependency-edges-list-all | jq '.data.edges'

# Filter by to_key containing string
... | jq '[.data.edges[] | select(.to_key | contains("Auth"))]'

# Filter by from_key containing string
... | jq '[.data.edges[] | select(.from_key | contains("controller"))]'

# Get unique edge types
... | jq '[.data.edges[].edge_type] | unique'

# Count edges per type
... | jq '.data.edges | group_by(.edge_type) | map({type: .[0].edge_type, count: length})'
```
