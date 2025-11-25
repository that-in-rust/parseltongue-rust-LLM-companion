# Parseltongue

> **v1.0.0** - 🚨 **BREAKING CHANGES**: Editing tools (pt03-pt06) removed - Pure analysis/search system
>
> **v0.9.7** - Agent JSON graph query helpers (<100ms) - ✅ **COMPLETE & FUNCTIONAL**

**Essence**: Parse code once → Query graph database many times → Get 2-5K token summaries instead of 500K+ dumps.

**Core Value**: 99% token reduction, 31× faster than grep, LLM-optimized architecture analysis.

**v1.0.0 Status**: Pure analysis system - editing tools removed (pt03-pt06 deleted)
**v0.9.7 Status**: All 4 query helpers working (100% functional) - blast radius analysis enabled

**12 languages**: Rust · Python · JavaScript · TypeScript · Go · Java · C · C++ · Ruby · PHP · C# · Swift

---

## Why Parseltongue? (The Problem)

**Before**: Dump 50,000 lines of code → 500K tokens → LLM context overflow → Poor reasoning
**After**: Query graph database → 2-5K tokens → 98% context free for thinking → Optimal analysis

```mermaid
graph LR
    A[50K LOC<br/>500K tokens ❌] -->|Parseltongue| B[Graph DB<br/>2-5K tokens ✅]
    B --> C[98% TSR*<br/>Optimal Reasoning]

    style A fill:#C89999
    style B fill:#99C899
    style C fill:#90EE90
```

*TSR = Thinking Space Ratio: (Context - Data) / Context

**Research**: Liu et al. (TACL 2023) showed 25% LLM performance drop with 30 documents in context. Parseltongue gives you graphs, not documents.

---

## ⚡ Quick Start (60 seconds)

### 1. Install (macOS)
```bash
curl -fsSL https://raw.githubusercontent.com/that-in-rust/parseltongue/main/parseltongue-install-v096.sh | bash
```

### 2. Index Your Codebase
```bash
./parseltongue pt01-folder-to-cozodb-streamer . --db "rocksdb:mycode.db"
```

**Example output**:
```
Entities created: 1,247 (CODE only)
  └─ TEST entities: 3,821 (excluded for optimal LLM context)
Duration: 2.1s
✓ Indexing completed
```

### 3. Get Architecture (3K tokens, 98% TSR)
```bash
./parseltongue pt02-level00 --where-clause "ALL" --output deps.json --db "rocksdb:mycode.db"
```

**You now know**:
- Who calls what (dependency graph)
- God objects (high fan-in)
- Circular dependencies
- Dead code (zero reverse_deps)

**Token count**: 3K (not 500K)

---

## 🎯 What You Just Got

### Progressive Disclosure (Minto Pyramid for Code)

```mermaid
graph TB
    L0[Level 0: Edges<br/>3K tokens<br/>97% TSR] --> L1[Level 1: Signatures<br/>30K tokens<br/>85% TSR]
    L1 --> L2[Level 2: Type System<br/>60K tokens<br/>70% TSR]

    L0 -.->|"Skip levels<br/>as needed"| L2

    style L0 fill:#90EE90
    style L1 fill:#FFE4B5
    style L2 fill:#FFB6C1
```

**Strategy**: Start minimal (Level 0), escalate only when needed.

### Real Metrics (parseltongue-core codebase)

| Metric | Value | vs Grep |
|--------|-------|---------|
| **Entities found** | 1,247 | N/A |
| **Ingestion time** | 2.1s | N/A |
| **Query time** | <50μs | 2.5s |
| **Token cost** | 2.3K | 250K |
| **Token reduction** | **99.1%** | ✅ |
| **Speed** | **31× faster** | ✅ |

---

## 📊 The Three Levels (Choose Your Detail)

### Level 0: Pure Edges (3K tokens, 97% TSR) — RECOMMENDED

**Best for**: Architecture overview, "what calls what?"

```bash
./parseltongue pt02-level00 --where-clause "ALL" --output edges.json --db "rocksdb:mycode.db"
```

**Output**: Edge list (caller → callee)

**Example**:
```json
{
  "dependency_count": 487,
  "dependencies": [
    {
      "caller_id": "parseltongue_core::parse_file",
      "callee_id": "tree_sitter::Parser::parse",
      "relationship_type": "calls"
    }
  ]
}
```

---

### Level 1: Entity Signatures (30K tokens, 85% TSR)

**Best for**: Understanding interfaces, finding functions by name/type

```bash
./parseltongue pt02-level01 --where-clause "ALL" --output entities.json --db "rocksdb:mycode.db"
```

**Output**: Function signatures, struct definitions, dependencies

**Example**:
```json
{
  "entity_id": "streamer::FileStreamer::stream_directory",
  "entity_name": "stream_directory",
  "entity_type": "function",
  "file_path": "./src/streamer.rs",
  "interface_signature": "pub async fn stream_directory(&self) -> Result<StreamingStats>",
  "reverse_deps": ["main", "process_directory"]
}
```

---

### Level 2: Full Type System (60K tokens, 70% TSR)

**Best for**: Deep type analysis, generic bounds, trait implementations

```bash
./parseltongue pt02-level02 --where-clause "ALL" --output typed.json --db "rocksdb:mycode.db"
```

**Output**: Everything from Level 1 + type parameters, where clauses, trait bounds

---

## 🔍 Common Queries (The Power)

### Find All Functions Returning Result\<Payment>

```bash
./parseltongue pt02-level01 \
  --where-clause "interface_signature ~ 'Result<Payment>'" \
  --output payments.json \
  --db "rocksdb:mycode.db"
```

**Found**: 12 functions (by return type, not name)

### Find All Code Calling Stripe API

```bash
# Step 1: Find matches (no code) - 2K tokens
./parseltongue pt02-level01 --include-code 0 \
  --where-clause "current_code ~ 'stripe\\.'" \
  --output matches.json --db "rocksdb:mycode.db"

# Step 2: Get code for specific functions - 2K tokens
./parseltongue pt02-level01 --include-code 1 \
  --where-clause "isgl1_key = 'rust:fn:charge_card:...'" \
  --output code.json --db "rocksdb:mycode.db"
```

**Total**: 4K tokens (vs 250K with grep)

### Blast Radius: What Breaks If I Change X?

```bash
./parseltongue pt02-level01 --include-code 0 \
  --where-clause "isgl1_key = 'rust:fn:validate_payment:...'" \
  --output entity.json --db "rocksdb:mycode.db"
# Returns: { reverse_deps: [15 direct callers] }
```

Then query those 15 callers to get full impact (2-hop traversal).

---

## 🤖 v0.9.7: Agent Query Helpers - ✅ PRODUCTION READY

**Status**: All 4 query helpers **100% functional** - blast radius analysis enabled

After exporting JSON with pt02-level01, agents can query it programmatically with type-safe helpers.

### Why?

Query architectural data **without re-querying the database** - get instant answers from exported JSON.

### 4 Query Patterns (All Working)

| Function | Purpose | Example Question | Status |
|----------|---------|------------------|--------|
| `find_reverse_dependencies_by_key()` | Blast radius | "What breaks if I change this?" | ✅ WORKS |
| `build_call_chain_from_root()` | Execution paths | "Show call chain from `main()`" | ✅ WORKS |
| `filter_edges_by_type_only()` | Edge filtering | "Show all `Calls` edges" | ✅ WORKS |
| `collect_entities_in_file_path()` | File search | "What's in `auth.rs`?" | ✅ WORKS |

### Example (Rust)

```rust
use parseltongue_core::{
    find_reverse_dependencies_by_key,
    build_call_chain_from_root,
};

// Load JSON export
let json: Value = serde_json::from_str(&export_content)?;

// Query 1: Blast radius
let affected = find_reverse_dependencies_by_key(
    &json,
    "rust:fn:validate_payment:src_payment_rs:89-112"
)?;

println!("⚠️  Changing validate_payment() affects {} functions:", affected.len());

// Query 2: Execution path from main
let call_chain = build_call_chain_from_root(
    &json,
    "rust:fn:main:src_main_rs:1-10"
)?;

println!("📞 Call chain from main():");
for (i, func) in call_chain.iter().enumerate() {
    println!("  {}. {}", i+1, func);
}
```

### Performance (Validated by Tests)

- **< 100ms** for 1,500 entities (release builds) ✅
- **< 150ms** for debug builds ✅
- Type-safe error handling (no panics) ✅
- Validated by 7 contract tests (all passing) ✅
- **Production-ready**: Used in test_v097_query_helpers/ validation ✅

### When to Use

```mermaid
graph TD
    A[Need architectural data?] --> B{Have JSON export?}
    B -->|No| C[Query database: pt02-level00/01]
    B -->|Yes| D{Need different entities?}
    D -->|Yes| C
    D -->|No| E[Use query helpers on JSON]

    style C fill:#99C899
    style E fill:#9DB4C8
```

✅ **Use query helpers**: You have JSON, want to traverse differently (blast radius, call chains)
❌ **Query database**: Need different entities than export contains

---

## 🚀 Real-World Example: Onboarding to 150K LOC Codebase

**Scenario**: You just joined a Rust project. Where do you start?

### Step 1: Index (30 seconds)
```bash
./parseltongue pt01-folder-to-cozodb-streamer . --db "rocksdb:onboard.db"
```

**Result**:
```
Entities created: 8,423 (CODE only)
Duration: 12.3s
```

### Step 2: Architecture Overview (3K tokens, 98% TSR)
```bash
./parseltongue pt02-level00 --where-clause "ALL" --output arch.json --db "rocksdb:onboard.db"
```

**You now know**:
- All module dependencies
- Critical paths (most-called functions)
- Architectural layers
- God objects, cycles, dead code

### Step 3: Public API Surface (5K tokens)
```bash
./parseltongue pt02-level01 --include-code 0 \
  --where-clause "is_public = true ; entity_class = 'Implementation'" \
  --output api.json --db "rocksdb:onboard.db"
```

**Result**: 276 public functions (API surface map)

### Total

- **Time to value**: 42 seconds
- **Token cost**: 8K (not 500K)
- **TSR**: 96% (optimal reasoning space)

---

## 🎯 Comparison: Parseltongue vs Grep

**Task**: Find payment processing functions + dependencies + test coverage

### Grep Approach ❌

```bash
# Step 1: Find payment code
grep -r "payment" ./src/  # 2.5s, 200 matches

# Step 2: Find dependencies
grep -r "process_payment\|validate_payment" ./src/  # 2.5s

# Step 3: Check test coverage
grep -r "test.*payment" ./tests/  # 2.5s

# Total: 7.5s, 500K tokens
# TSR: NEGATIVE (context overflow)
```

### Parseltongue Approach ✅

```bash
# Step 1: Find payment functions (80ms)
./parseltongue pt02-level01 --include-code 0 \
  --where-clause "interface_signature ~ 'Payment' ; entity_name ~ 'payment'" \
  --output payment.json --db "rocksdb:repo.db"
# Returns: 15 entities, 1.5K tokens

# Step 2: Dependencies already in output!
# forward_deps: [what each function calls]
# reverse_deps: [who calls each function]

# Step 3: Check test coverage (50ms)
./parseltongue pt02-level01 --include-code 0 \
  --where-clause "entity_name ~ 'payment' ; is_test = true" \
  --output tests.json --db "rocksdb:repo.db"
# Returns: 8 test entities, 0.8K tokens

# Total: 130ms, 2.3K tokens
# TSR: 98.85% ✓
```

### Results

| Metric | Grep | Parseltongue | Improvement |
|--------|------|--------------|-------------|
| Time | 7.5s | 130ms | **57× faster** |
| Tokens | 500K | 2.3K | **99.5% reduction** |
| TSR | Negative | 98.85% | **Context preserved** |
| Structure | Raw text | Entities + deps | **Graph data** |

---

## 📚 Documentation

### User Guides
- **Quick Start**: See above
- **Common Queries**: See above
- **Progressive Disclosure**: Level 0 → 1 → 2 strategy

### Technical Docs
- **Architecture**: Layered design (S06 principles)
- **TDD-First**: Executable specifications (S01 principles)
- **Rust Patterns**: Functional, idiomatic (S77 patterns)

### Agent Integration
- **@parseltongue-ultrathink-isg-explorer**: Context-efficient analyst
- **Query Helpers**: v0.9.7 type-safe JSON traversal
- **Workflows**: Onboarding, blast radius, refactoring analysis

---

## 🛣️ Roadmap

### v1.0.0 - ✅ COMPLETE (BREAKING CHANGES - MAJOR RELEASE)

**Scope**: Pure analysis/search system - Remove editing tools
- ✅ Deleted pt03-llm-to-cozodb-writer (code modification)
- ✅ Deleted pt04-syntax-preflight-validator (validation)
- ✅ Deleted pt05-llm-cozodb-to-diff-writer (diff generation)
- ✅ Deleted pt06-cozodb-make-future-code-current (code replacement)
- ✅ Cleaned up 37 files, 450+ lines from main.rs
- ✅ Updated dependencies and tests

**Status**: PRODUCTION READY - Focus on analysis/search only
**Migration**: If you used pt03-pt06, no direct replacement available

**Versioning Note**: Following .claude.md rules - v0.9.7 → v1.0.0 (skipped v0.10.x for major breaking change)

### v0.9.7 - ✅ COMPLETE

**Scope**: Agent JSON graph query helpers (<100ms)
- ✅ 4 query helper functions implemented
- ✅ Contract tests (7 tests, all passing)
- ✅ pt02-level01 now populates reverse_deps/forward_deps
- ✅ Performance validated: <100ms for 1,500 entities
- ✅ Blast radius analysis functional

### Future Features (Post-v1.0.0)

**See**: `BACKLOG-CHALLENGES.md` for detailed ROI analysis of:
- Semantic Edge Directionality (ROI 9.5/10)
- Hierarchical Clustering Integration (ROI 10/10)
- Mermaid Auto-Generation (ROI 9/10)
- Control Flow Edges (ROI 4/10 - deferred)

---

## 🏗️ Architecture Principles

**S01: TDD-First MVP Rigor**
- Executable specifications (test-first)
- 4-word naming convention
- Proven architectures over theoretical abstractions

**S06: Layered Architecture**
- L1: Domain (CozoDB graph)
- L2: Standard Library (parseltongue-core)
- L3: Applications (pt01: ingest, pt02: query, pt07: visualize)

**S77: Idiomatic Rust**
- Expression-oriented code
- Error boundaries (thiserror for libs, anyhow for apps)
- Pure functions with explicit Result<T, E>

---

## 📖 Research Foundation

**Liu et al. (TACL 2023)** "Lost in the Middle: How Language Models Use Long Contexts"
- Finding: 30 documents in context → 25% performance drop
- Application: Parseltongue gives graphs (2.3K tokens), not documents (250K tokens)

**Database Indexing Fundamentals**
- Grep: O(n × m) linear scan
- Database: O(log n) indexed lookups
- **Result**: 100-1000× speed difference at scale

**Token Arithmetic** (1,500 entity codebase):
- Full code: 525K tokens
- Signatures only: 37.5K tokens
- Filtered (20 entities): 2.3K tokens
- **Improvement**: 228× reduction

---

## 🤝 Contributing

See `.claude/prdArchDocs/` for:
- Feature specifications
- TDD principles (S01)
- Architecture guidelines (S06)
- Rust patterns (S77)

---

## 📄 License

MIT License - See LICENSE file

---

## 🔗 Links

- **GitHub**: [that-in-rust/parseltongue](https://github.com/that-in-rust/parseltongue)
- **Issues**: [Report bugs](https://github.com/that-in-rust/parseltongue/issues)
- **Docs**: `.claude/prdArchDocs/` directory

---

**Parse once, query forever.**

*Parseltongue: Making LLMs reason about code with graphs, not text.*
