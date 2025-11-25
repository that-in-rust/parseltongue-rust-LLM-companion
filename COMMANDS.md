# Parseltongue v1.0.0 - CLI Reference

Quick reference for all parseltongue commands. For examples and workflows, see [README.md](README.md) or the [agent file](.claude/agents/parseltongue-ultrathink-isg-explorer.md).

---

## pt01: Ingest (Folder → CozoDB)

Parse codebase and stream entities to CozoDB database.

### Basic Syntax
```bash
parseltongue pt01 <DIRECTORY> --db "rocksdb:<DB_PATH>"
```

### Options
- `DIRECTORY` - Path to code directory to parse
- `--db "rocksdb:<path>"` - Database path (MUST use rocksdb: prefix)

### Example
```bash
# Index current directory
parseltongue pt01 . --db "rocksdb:mycode.db"

# Index specific directory
parseltongue pt01 ./src --db "rocksdb:analysis.db"
```

### Output
```
Entities created: 1,247 (CODE only)
  └─ TEST entities: 3,821 (excluded for optimal LLM context)
Duration: 2.1s
✓ Indexing completed
```

---

## pt02: Query (CozoDB → JSON Exports)

Export entities from database with progressive detail levels.

### Three Levels

#### Level 0: Pure Edges (~3K tokens, 97% TSR)
```bash
parseltongue pt02-level00 \
  --where-clause "<QUERY>" \
  --output <FILE>.json \
  --db "rocksdb:<DB_PATH>"
```

**Output**: Caller→Callee edge list (dependency graph)
**Use**: Architecture overview, "what calls what?"

#### Level 1: Entity Signatures (~30K tokens, 85% TSR)
```bash
parseltongue pt02-level01 \
  --where-clause "<QUERY>" \
  --output <FILE>.json \
  --db "rocksdb:<DB_PATH>" \
  [--include-code <0|1>]
```

**Output**: Function signatures, reverse_deps, forward_deps
**Use**: Understanding interfaces, blast radius analysis
**Flag**: `--include-code 0` excludes implementation code (default), `1` includes it

#### Level 2: Full Type System (~60K tokens, 70% TSR)
```bash
parseltongue pt02-level02 \
  --where-clause "<QUERY>" \
  --output <FILE>.json \
  --db "rocksdb:<DB_PATH>" \
  [--include-code <0|1>]
```

**Output**: Everything from Level 1 + type parameters, trait bounds, where clauses
**Use**: Deep type analysis, generic bounds

---

### WHERE Clause Syntax (Datalog)

**Operators:**
- `,` = AND
- `;` = OR
- `=` = equality (not `==`)
- `~` = regex match
- `!=` = not equal

**Entity Fields:**
- `isgl1_key` - Unique ID (e.g., `"rust:fn:main:src_main_rs:1-10"`)
- `entity_name` - Function/struct name
- `entity_type` - `"function"`, `"struct"`, `"trait"`, etc.
- `file_path` - Source file path
- `interface_signature` - Function signature or struct definition
- `is_public` - `true`/`false`
- `is_test` - `true`/`false`
- `entity_class` - `"Implementation"` or `"Test"`

**Common Queries:**

```bash
# All entities (no filter)
--where-clause "ALL"

# Only public functions
--where-clause "is_public = true ; entity_class = 'Implementation'"

# Functions matching pattern
--where-clause "entity_name ~ 'payment'"

# Specific function by key
--where-clause "isgl1_key = 'rust:fn:calculate_total:src_billing_rs:42-67'"

# Public API in specific file
--where-clause "is_public = true , file_path ~ 'src/api'"

# Multiple conditions
--where-clause "entity_type = 'function' , is_public = true ; entity_name ~ 'test'"
```

---

### Dual File Export

All pt02 commands automatically create TWO files:

```bash
parseltongue pt02-level01 --output analysis.json --db "rocksdb:mycode.db"
# Creates:
#   analysis.json       (Production: entity_class = 'Implementation')
#   analysis_test.json  (Test: entity_class = 'Test')
```

---

## pt07: Visualize (CozoDB → Terminal)

Generate text-based visualizations from database.

### entity-count: Entity Distribution
```bash
parseltongue pt07 entity-count --db "rocksdb:<DB_PATH>"
```

**Output**: Bar chart showing entity counts by type
```
╔═══════════════════════════════════════════╗
║    Entity Count by Type (Impl Only)      ║
╠═══════════════════════════════════════════╣
║ function │████████████████░ 487           ║
║ struct   │████████░░░░░░░░░ 234           ║
║ trait    │███░░░░░░░░░░░░░░  89           ║
╚═══════════════════════════════════════════╝
```

### cycles: Circular Dependency Detection
```bash
parseltongue pt07 cycles --db "rocksdb:<DB_PATH>"
```

**Output**: Warning list of circular dependencies
```
⚠️  CIRCULAR DEPENDENCIES DETECTED

Cycle 1 (3 nodes):
  → module_a::init
  → module_b::setup
  → module_c::configure
  → module_a::init (cycle)
```

---

## Common Workflows

### Workflow 1: Full Codebase Onboarding
```bash
# 1. Ingest
parseltongue pt01 . --db "rocksdb:repo.db"

# 2. Get architecture overview
parseltongue pt02-level00 \
  --where-clause "ALL" \
  --output architecture.json \
  --db "rocksdb:repo.db"

# 3. Get public API surface
parseltongue pt02-level01 \
  --where-clause "is_public = true ; entity_class = 'Implementation'" \
  --output api.json \
  --db "rocksdb:repo.db"

# 4. Check for cycles
parseltongue pt07 cycles --db "rocksdb:repo.db"
```

### Workflow 2: Blast Radius Analysis
```bash
# 1. Find target function
parseltongue pt02-level01 --include-code 0 \
  --where-clause "entity_name ~ 'validate_payment'" \
  --output target.json \
  --db "rocksdb:repo.db"

# 2. Read reverse_deps field from target.json
# 3. Query those callers for full impact
```

### Workflow 3: Find Code by Return Type
```bash
# Find all functions returning Result<Payment>
parseltongue pt02-level01 --include-code 0 \
  --where-clause "interface_signature ~ 'Result<Payment>'" \
  --output payment_functions.json \
  --db "rocksdb:repo.db"
```

---

## Database Format

**Always use RocksDB prefix:**
```bash
--db "rocksdb:mydb.db"    # ✅ Correct
--db "mydb.db"            # ❌ Wrong (engine not specified)
```

**Path can be relative or absolute:**
```bash
--db "rocksdb:./analysis.db"           # Relative
--db "rocksdb:/tmp/my_analysis.db"     # Absolute
```

---

## Supported Languages

Parseltongue supports 12 languages via tree-sitter:

- Rust
- Python
- JavaScript
- TypeScript
- Go
- Java
- C
- C++
- Ruby
- PHP
- C#
- Swift

---

## Performance Characteristics

| Operation | Time | Token Cost | TSR |
|-----------|------|------------|-----|
| pt01 ingest (1500 entities) | 2.1s | - | - |
| pt02-level00 (all edges) | <50ms | 3K | 97% |
| pt02-level01 (all entities, no code) | <80ms | 30K | 85% |
| pt02-level01 (filtered, 20 entities) | <50ms | 2.3K | 98.9% |
| pt07 visualizations | <100ms | Minimal | - |

**TSR = Thinking Space Ratio**: (Context - Data) / Context
- Higher TSR = More context available for LLM reasoning

---

## Version Notes

**v1.0.0** (Current - MAJOR RELEASE):
- ✅ Removed editing tools (pt03-pt06)
- ✅ Focus: Pure analysis and search
- 🚨 **BREAKING**: No code modification capabilities
- 📋 Versioning: Skipped v0.10.x per .claude.md rules (v0.9.7 → v1.0.0)

**v0.9.7**:
- ✅ Query helpers for JSON traversal
- ✅ reverse_deps/forward_deps populated

---

## Need More Help?

- **Examples**: See [README.md](README.md)
- **Agent Workflows**: See [.claude/agents/parseltongue-ultrathink-isg-explorer.md](.claude/agents/parseltongue-ultrathink-isg-explorer.md)
- **Issues**: https://github.com/that-in-rust/parseltongue/issues
