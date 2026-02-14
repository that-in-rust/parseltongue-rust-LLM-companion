# RESEARCH: isgl1v3-exhaustive-graph-identity

**Date**: 2026-02-14
**Status**: Research / Pre-Implementation
**Scope**: ISGL1 v3 key format, `|||` delimiter, exhaustive file coverage, entity taxonomy, SharedFileContext edges

---

## Problem Statement

ISGL1 v2 has two structural flaws:

1. **The graph has holes.** Only named entities (functions, classes, structs) get keys. Imports, comments, module-level code, and unrecognized constructs are invisible. Coverage metrics show 60-80% per file — meaning 20-40% of every file does not exist in the graph.

2. **The `:` delimiter collides with code syntax.** Rust uses `::` for module paths (`std::io::Read`), Python uses `:` for type hints (`x: int`), Windows uses `C:\` drive letters, URLs use `:` for ports. An LLM parsing `rust:fn:std::io::Read:__src_main:T170...` can mis-split the key into the wrong number of fields.

Both flaws undermine Parseltongue's core value proposition: if the graph is the product, the graph must contain everything.

---

## ISGL1 v3 Key Format

```
{language}|||{entity_type}|||{entity_name}|||{file_path}|||{line_start}|||{line_end}
```

### Examples

```
rust|||fn|||handle_auth|||src/auth.rs|||15|||42
python|||class|||UserService|||src/services/user.py|||8|||95
csharp|||fn|||GetAsync<T>|||Controllers/ApiController.cs|||30|||55
javascript|||import|||react_imports|||src/App.jsx|||1|||3
rust|||gap|||between_login_register|||src/auth.rs|||21|||22
python|||test|||test_user_creation|||tests/test_user.py|||10|||35
```

### What Changed from v2

| Aspect | v2 | v3 |
|--------|----|----|
| Delimiter | `:` (collides with Rust, Python, URLs) | `|||` (zero collisions across 15 languages) |
| Location | Opaque birth timestamp `T1706284800` | Actual file path + line range |
| Path encoding | Sanitized `__src_auth` | Raw `src/auth.rs` (readable) |
| Generic sanitization | Required (`List__lt__T__gt__`) | Not needed (`List<T>` safe — `|||` can't be confused) |
| Coverage | Named entities only (~70%) | Every line of every file (100%) |
| Stability across edits | Stable (timestamp never changes) | Unstable (line numbers shift) |
| Rebuild model | Incremental match attempt | Full file re-parse (what we do anyway) |

### Why Key Instability Doesn't Matter

v2's birth timestamp solved incremental reindex key matching — preserving old keys when line numbers shift. But:

- pt01 does a full file re-parse on every ingest
- File watcher triggers a full file re-parse on change
- Edges are rebuilt from the same parse pass
- There is no scenario where old edges survive across a re-index with changed keys

The graph is rebuilt from source every time. Key stability between runs is a solution to a problem that doesn't exist in the current architecture. Every edge is generated in the same pass as the entity it points to. If an entity's line range shifts, its edges shift too — they're consistent within the scan.

---

## `|||` Delimiter: Collision Analysis

Research checked `|||` against all 15 supported languages (Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, Ruby, PHP, C#, Swift, Kotlin, Scala, SQL), file paths (Unix and Windows), and URLs.

### Collision Matrix (Abridged)

| Delimiter | Collisions Found |
|-----------|-----------------|
| `:` | Rust `::`, Python `x: int`, Go struct tags, Windows `C:\`, URLs `http://`, port `:8080` |
| `::` | Rust `std::io::Read`, C++ `std::cout`, Ruby `Foo::BAR` |
| `___` | Python `__init__`, C `__attribute__`, already used in v2 semantic paths |
| `->` | Rust `fn() -> i32`, C/C++ `ptr->field`, PHP `$this->foo` |
| `=>` | JS arrow `() => {}`, Rust match `x => y`, Ruby `{a => b}` |
| `\|` (single) | Rust closures `\|x\|`, shell pipes, markdown tables |
| `\|\|\|` | **Zero.** Never appears in any language syntax, identifier, file path, or URL. |

**`|||` is the only candidate with zero collisions across all 15 languages.**

Single `|` has edge cases (Rust closures, bitwise OR). But triple `|||` never appears as:
- A valid identifier in any language
- Part of any operator in any language
- A file path character on any OS
- A URL component

### LLM Tokenization

- `|||` tokenizes as 1-2 tokens in GPT-4o (o200k_base vocab) and Claude's BPE
- Single `:` is 1 token
- For 5 delimiters per key, worst case `|||` costs ~5 extra tokens vs `:`
- Parseltongue already achieves 99% token reduction (2-5K tokens vs 500K raw). 5 extra tokens per key is noise.

### LLM Parsing Reliability

- Triple-character delimiters create stronger "semantic anchoring" than single characters (prompt engineering research)
- LLMs trained on markdown tables (`| col1 | col2 |`) have strong pipe-as-separator priors
- JSON's `:` for key-value separation actively degrades LLM reasoning by 10-15% vs less syntactically loaded formats (published benchmark)
- TOON format (token-oriented, pipe-based) achieves 73.9% accuracy vs JSON's 69.7% in data retrieval tasks

### Bonus: No More Path Sanitization

With `:` as delimiter, file paths had to be encoded: `src/auth.rs` → `__src_auth` (remove extension, replace `/` with `_`, add `__` prefix). This was necessary because `/` and `.` inside the key would confuse `:` based parsing.

With `|||`, the raw file path `src/auth.rs` is safe — no character in a file path can be confused with `|||`. The key becomes self-describing: an LLM reads the key and knows exactly where to `Read` the code.

### Bonus: No More Generic Sanitization

v2.1 sanitized generic types: `List<string>` → `List__lt__string__gt__` because `<` and `>` could confuse CozoDB's Datalog parser when used in `:` delimited keys.

With `|||` delimiting, `<` and `>` inside the entity name segment cannot be confused with field boundaries. `csharp|||fn|||GetAsync<T>|||path|||30|||55` parses unambiguously. The sanitization layer can be removed entirely.

---

## Exhaustive File Coverage

### The Principle

Every line of every file maps to exactly one entity. No gaps. No invisible code. Coverage is 100% by definition.

### Current State (v2)

A 100-line file with entities at lines 5-20, 30-50, 60-80:

```
Lines  1-4  : invisible (imports)
Lines  5-20 : fn login          ← entity
Lines 21-29 : invisible (comments, blank lines)
Lines 30-50 : fn register       ← entity
Lines 51-59 : invisible (module-level code)
Lines 60-80 : fn verify_token   ← entity
Lines 81-100: invisible (test module)
```

39 lines invisible. 3 entities. 61% coverage.

### Proposed State (v3)

Same file, exhaustive coverage:

```
Lines  1-3  : ImportBlock        ← entity (use statements)
Lines  4-4  : GapFragment        ← entity (blank line)
Lines  5-20 : CoreCode           ← entity (fn login)
Lines 21-22 : GapFragment        ← entity (blank lines)
Lines 23-29 : UncollectableFragment ← entity (comment block)
Lines 30-50 : CoreCode           ← entity (fn register)
Lines 51-59 : UncollectableFragment ← entity (module-level constants)
Lines 60-80 : CoreCode           ← entity (fn verify_token)
Lines 81-100: TestCode           ← entity (test module)
```

0 lines invisible. 9 entities. 100% coverage.

---

## Entity Taxonomy

### Current (v2): Binary Classification

```
CodeImplementation  →  kept in graph
TestImplementation  →  excluded from graph entirely
```

Everything else (imports, comments, unrecognized constructs) → discarded.

### Proposed (v3): Five Classes

| Entity Class | What It Captures | In Graph? | Edges? |
|---|---|---|---|
| `CoreCode` | Functions, classes, structs, enums, traits, impl blocks | Yes | Full (Calls, Uses, Implements, SharedFileContext) |
| `TestCode` | Test functions, test modules, test classes | Yes (filterable) | Full (Calls, Uses, SharedFileContext) |
| `ImportBlock` | import/use/require/include statements | Yes | SharedFileContext + implicit ImportsFrom |
| `GapFragment` | Blank lines, standalone comments between entities | Yes (lightweight) | SharedFileContext only |
| `UnparsedConstruct` | Code constructs the tree-sitter query didn't match | Yes (as markers) | SharedFileContext only |

### Why Keep Tests

Current behavior excludes tests entirely. This means:
- No test-to-code edges in the graph
- LLMs can't answer "what tests cover this function?"
- Blast radius analysis misses test impact
- Community detection can't cluster test+code together

With `TestCode` as a filterable class, LLMs choose what they want:
- Production analysis: filter to `CoreCode`
- Full context: include `TestCode`
- Coverage analysis: count `TestCode` entities per `CoreCode` entity

### Why Keep Gaps

`GapFragment` entities seem wasteful — blank lines as graph nodes? But they serve a purpose:

1. **Exhaustive coverage guarantee**: every line maps to something, so file watcher line-range lookup always succeeds
2. **Structural signal**: a 20-line gap between two functions is different from a 0-line gap. It signals logical separation.
3. **Low cost**: gap fragments are tiny nodes with no code content, just a class label and line range

### Why Keep Unparsed Constructs

`UnparsedConstruct` entities capture what the parser missed. This is critical for:

1. **Transparency**: LLMs know there's code at lines 51-59 they can `Read` even though Parseltongue didn't understand it
2. **Parser improvement signals**: high `UnparsedConstruct` counts per language indicate parser gaps to fix
3. **No silent failures**: current behavior silently drops unrecognized code. v3 marks it explicitly.

---

## New Edge Types

### Current (v2): Three Edge Types

```
Calls       →  A calls function B
Uses        →  A uses type/interface B
Implements  →  A implements trait/interface B
```

### Proposed (v3): Six Edge Types

| Edge Type | Meaning | Created By |
|---|---|---|
| `Calls` | A calls function B | AST extraction (existing) |
| `Uses` | A uses type/interface B | AST extraction (existing) |
| `Implements` | A implements trait/interface B | AST extraction (existing) |
| `SharedFileContext` | A and B are in the same file | File membership (new) |
| `ImportsFrom` | File imports a module/package | Import statement parsing (new) |
| `TestedBy` | Test entity targets this code entity | Naming convention + proximity (new) |

### SharedFileContext Topology

Not pairwise (O(n^2) edges per file). Star topology via a file-level node:

```
[file|||src/auth.rs|||1|||100]
    ├── SharedFileContext → [rust|||import|||auth_imports|||src/auth.rs|||1|||3]
    ├── SharedFileContext → [rust|||fn|||login|||src/auth.rs|||5|||20]
    ├── SharedFileContext → [rust|||fn|||register|||src/auth.rs|||30|||50]
    ├── SharedFileContext → [rust|||fn|||verify_token|||src/auth.rs|||60|||80]
    └── SharedFileContext → [rust|||test|||test_login|||src/auth.rs|||81|||100]
```

One file node, N edges to its entities. LLM traversal: `login → file → register, verify_token, imports`. One hop gives you everything in the file.

### Edge Weight Consideration

Graph algorithms (PageRank, betweenness, k-core) treat all edges equally. A `SharedFileContext` edge should not carry the same weight as a `Calls` edge. Options:

1. **Typed edge filtering**: algorithms only traverse specific edge types (e.g., PageRank on `Calls`+`Uses` only)
2. **Numeric weights**: `Calls=1.0`, `Uses=0.8`, `Implements=0.9`, `SharedFileContext=0.1`, `ImportsFrom=0.3`
3. **Separate graphs**: one graph for explicit deps (Calls/Uses/Implements), one for implicit context (SharedFileContext/ImportsFrom)

Option 1 is simplest and already partially implemented (handlers filter by edge type).

---

## Impact on Scale

### Entity Count

| Metric | v2 (current) | v3 (proposed) |
|--------|-------------|---------------|
| Entities per file (avg) | 3-5 (named only) | 8-15 (exhaustive) |
| Total entities (19K codebase) | ~19,000 | ~50,000-60,000 |
| Total edges (144K codebase) | ~144,000 | ~250,000-300,000 |
| .ptgraph file size | ~5 MB | ~12-15 MB |
| CozoDB RAM (slim model) | ~504 MB | ~1.2-1.5 GB |

### Mitigation

Entity classes enable filtering at query time. An LLM requesting `/code-entities-list-all?class=CoreCode` gets the same count as v2. The extra entities exist for completeness but don't bloat every response.

---

## File Watcher Benefit

### Current Flow

```
file changed → re-parse ENTIRE file → extract entities → match against old index → update graph
```

### v3 Flow

```
file changed → diff gives changed line ranges
            → v3 key lookup: which entities span those lines?
            → re-parse changed entities + adjacent gaps
            → rebuild affected entities with new line ranges
            → update all entities below the edit (line shift)
            → rebuild edges for changed entities
```

The optimization: unchanged entities above the edit are not re-parsed. Their v3 keys don't change (lines didn't shift). Only entities at and below the edit point need updates.

Not a radical speedup (you still update line ranges for shifted entities), but you skip tree-sitter parsing for unchanged code.

---

## Migration Path: v2 → v3

### Breaking Change

v3 keys are incompatible with v2 keys. Every key changes format. All edges reference new keys. This is a full graph rebuild.

### Migration Strategy

1. v3 is a new key system, not a patch on v2
2. pt01 generates v3 keys on next full ingest
3. Old v2 .ptgraph files are not loadable (version field mismatch)
4. No incremental migration — clean break

This is acceptable because:
- pt01 already does full ingests
- .ptgraph files are snapshots, not databases
- No external system stores ISGL1 keys long-term

---

## Open Questions

1. **GapFragment granularity**: Should consecutive blank lines be one GapFragment or one per line? One per group (contiguous blank lines = single fragment) seems right.

2. **Import grouping**: Should each `use`/`import` statement be its own entity, or should consecutive imports be grouped into one `ImportBlock`? Grouping reduces entity count; individual gives finer edges.

3. **Comment attribution**: A comment directly above a function probably belongs to that function (doc comment), not a separate GapFragment. Where's the boundary?

4. **File-level entity**: Does the file itself get an entity node (for the star topology), or is SharedFileContext implemented differently?

5. **Edge weight integration**: Do existing graph algorithms (Tarjan, k-core, PageRank, Leiden) need modification to handle weighted/typed edges, or is filtering sufficient?

---

## Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Delimiter | `|||` | Zero collisions across 15 languages, strong LLM parsing |
| Key format | `lang|||type|||name|||filepath|||linestart|||lineend` | Self-describing, readable, no encoding needed |
| Coverage | Exhaustive (100%) | Every line maps to an entity |
| Entity classes | 5 (CoreCode, TestCode, ImportBlock, GapFragment, UnparsedConstruct) | Nothing invisible, everything filterable |
| Edge types | 6 (add SharedFileContext, ImportsFrom, TestedBy) | File-level context captured |
| Key stability | Abandoned (v2 timestamps removed) | Full rebuild on every ingest, stability has no consumer |
| Generic sanitization | Removed | `|||` delimiter makes `<>` in names safe |
| Path sanitization | Removed | `|||` delimiter makes `/` and `.` in paths safe |
