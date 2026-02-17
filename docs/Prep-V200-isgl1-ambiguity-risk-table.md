# v200: ISGL1 Key Format Ambiguity & Risk Assessment (Expanded)

**Generated**: 2026-02-16
**Context**: Challenging the "MEDIUM risk" rating on key format from `v200-architecture-opinion-analysis.md`
**Sources**: `Prep-V200-Key-Format-Design.md`, `RESEARCH-isgl1v3-exhaustive-graph-identity.md`, `isgl1_v2.rs`, `entities.rs`, `PRD_v173.md`
**Verdict**: Risk is LOWER than initially claimed. Research is ~80% complete. But deeper code reading surfaces 18 sub-questions across 7 areas.

---

## Why This Reassessment Exists

In `v200-architecture-opinion-analysis.md`, I rated the key format as:

```
rust-llm-core           parseltongue-core ✓    Key format is MEDIUM
```

And said:

> "The PRD says this is week 1-2 but it deserves its own RFC.
>  Everything builds on it. Get it wrong, rebuild everything."

This document challenges that rating by auditing what research ALREADY EXISTS, what's actually ambiguous, and — after reading the implementation code — what hidden questions the design docs don't address.

---

## Research Already Completed

| Document | What It Resolved | Lines |
|----------|-----------------|-------|
| `Prep-V200-Key-Format-Design.md` | 8 v2 problems identified, 6 competitor systems analyzed, 10 MUST + 5 SHOULD + 4 MUST NOT requirements, 4 candidate designs evaluated, recommendation made | 762 |
| `RESEARCH-isgl1v3-exhaustive-graph-identity.md` | `\|\|\|` delimiter proven (zero collisions across 15 languages), exhaustive coverage taxonomy (5 entity classes), 6 edge types, migration strategy | 346 |
| `isgl1_v2.rs` (current impl) | Baseline implementation: 366 lines, sanitization layer, birth timestamp, incremental matching | 366 |
| `entities.rs` (current types) | `Language` enum (15 variants), `EntityType` (17 variants), `InterfaceSignature` with `module_path`, `Isgl1Key` newtype, `DependencyEdge`, `SlimEntityGraphSnapshot` | 1,883 |
| `PRD_v173.md` (taint sections) | ISGL1 v3 keys used in taint finding payloads, response categorization by entity class | ~100 |

**Total research already written: ~3,457 lines across 5 documents.**

---

## Per-Crate Ambiguity Table

Each v2.0.0 crate's relationship with the key format, what's DECIDED vs AMBIGUOUS.

| Crate | Role With Keys | What's DECIDED | What's AMBIGUOUS | Ambiguity Level |
|-------|---------------|----------------|------------------|-----------------|
| **rust-llm-core** | DEFINES EntityKey struct | Candidate D+B hybrid. 10 MUST requirements. 6 fields: `language`, `kind`, `scope`, `name`, `file_path`, `discriminator`. `\|\|\|` serialization. | Q1: Scope type. Q2: Interning. Q3: Path normalization. Q8: EntityKind taxonomy mismatch. Q9: ImplBlock data loss. Q10: LanguageSpecific gap. Q11: EntityClass taxonomy mismatch. Q15: Isgl1Key newtype fate. Q16: Temporal state fate. Q17: Two research docs conflict on line numbers in keys. | LOW-MEDIUM |
| **rust-llm-parse** (was rust-llm-01) | GENERATES keys | `\|\|\|` delimiter. No sanitization. Raw file paths. Entity taxonomy defined. | Q4: Scope extraction depth. Q12: How to populate scope from tree-sitter per language. Q13: Discriminator content when tree-sitter can't extract params. | LOW |
| **rust-llm-crosslang** (was rust-llm-02) | REFERENCES keys across language boundaries | Language prefix distinguishes cross-lang keys. File path field enables matching. | Q5: External entity key format. Q14: Cross-language key matching heuristic. | MEDIUM |
| **rust-llm-bridge** (was rust-llm-03) | MAPS rust-analyzer DefIds TO keys | Must produce SAME key as tree-sitter. Deterministic (M9). | Q6: DefId-to-EntityKey mapping. Q18: rust-analyzer scope vs tree-sitter scope reconciliation. | MEDIUM |
| **rust-llm-rules** (was rust-llm-04) | JOINS on keys in Ascent Datalog | Typed struct = pattern matching on `EntityKind` directly. No string parsing in rules. | None — Candidate D struct solves this. Ascent pattern matching on typed fields is unambiguous. | NONE |
| **rust-llm-store** (was rust-llm-05) | INDEXES on keys in HashMaps | `Hash + Eq` derived on struct. No string hashing concern. | Q2: String interning for scope/name fields (shared with core). Q7: DependencyEdge and SlimEntityGraphSnapshot migration. | LOW |
| **rust-llm-http** (was rust-llm-06) | RETURNS keys in JSON | Display trait → `rust\|\|\|fn\|\|\|auth::handlers\|\|\|login\|\|\|src/auth.rs\|\|\|d0`. LLM-readable. No URL encoding issues. | None — serialization format is fully specified. | NONE |
| **rust-llm-mcp** (was rust-llm-07) | RETURNS keys to Claude/Cursor | Same Display serialization. MCP tool_result is JSON string. | None — same as HTTP. | NONE |

### Ambiguity Heatmap

```
              DECIDED        AMBIGUOUS        UNKNOWN
              ████████       ████████         ████████

rust-llm-core   ████████████░░░░░░░░       9 sub-questions
rust-llm-parse  ██████████████████░░       3 sub-questions
rust-llm-cross  ████████████░░░░░░░░       2 sub-questions
rust-llm-bridge ██████████░░░░░░░░░░       2 sub-questions
rust-llm-rules  ████████████████████       0 sub-questions  ← FULLY RESOLVED
rust-llm-store  ████████████████░░░░       2 sub-questions
rust-llm-http   ████████████████████       0 sub-questions  ← FULLY RESOLVED
rust-llm-mcp    ████████████████████       0 sub-questions  ← FULLY RESOLVED

█ = decided    ░ = ambiguous

Total: 18 sub-questions across 7 major areas
```

---

## The 18 Open Questions (Exhaustive, Detailed)

### Area 1: Scope Representation (Q1, Q12)

**Q1: `scope: Vec<String>` vs `Vec<ScopeSegment>` enum?**

The Prep-V200 doc proposes two versions of the same field:

```rust
// Simplified (from recommendation section 6.2):
pub scope: Vec<String>,
// Serialized as: "my_crate.auth.handlers" (joined with '.')

// Rich (from candidate D, section 4.4):
pub scope: Vec<ScopeSegment>,
pub enum ScopeSegment {
    Crate(String),
    Module(String),
    Class(String),
    Impl { type_name: String, trait_name: Option<String> },
    Namespace(String),
}
```

Sub-questions:
- Does `Vec<String>` lose information that Ascent rules need? E.g., distinguishing "this scope segment is a class vs a module" matters for rules like "methods must not call module-level functions directly."
- If `Vec<ScopeSegment>`, how does it serialize to `|||`? The recommendation says scope joins with `.` — but `Impl { type_name: "Foo", trait_name: Some("Display") }` doesn't have a natural `.`-joined form.
- The CURRENT codebase already has `InterfaceSignature.module_path: Vec<String>` (entities.rs:265). This IS `Vec<String>`. Is v2.0.0 just renaming this field, or genuinely changing its semantics?
- Can we start with `Vec<String>` and add `ScopeSegment` later without breaking key equality? Answer: YES, if the Display serialization doesn't change. The struct fields change but the `|||` output stays the same.

**Decision path**: Start with `Vec<String>`. If Ascent rules need typed scope segments, introduce `ScopeSegment` as an internal enrichment that doesn't affect the serialized key. This is what the recommendation already says.

**Difficulty**: LOW — the current codebase already uses `Vec<String>` for `module_path`.

---

**Q12: How to populate scope from tree-sitter, per language?**

Tree-sitter gives you an AST, not a module path. Extracting the scope requires language-specific knowledge:

| Language | Scope extraction method | Complexity |
|----------|------------------------|------------|
| Rust | Walk `mod` declarations up from entity. `pub(crate)` visibility hints. | MEDIUM — nested modules in same file vs separate files |
| Python | Walk `class` nesting. Package = directory structure + `__init__.py`. | MEDIUM — relative imports, `__all__`, dynamic module creation |
| JavaScript | No native modules. Scope = file path. Classes nest. | LOW — scope ≈ file path |
| TypeScript | Same as JS + namespaces (`namespace Foo { ... }`). | LOW-MEDIUM — namespaces add one layer |
| Java | Package from `package com.example;` declaration. Class nesting. | LOW — one package per file, explicit declaration |
| Go | Package from `package main` declaration. No class nesting. | LOW — one package per file |
| C | No modules. Scope = file path. | LOW — no module system |
| C++ | Namespaces (`namespace foo { namespace bar { ... } }`). Class nesting. | MEDIUM — anonymous namespaces, inline namespaces |
| Ruby | Module/class nesting (`module Foo; class Bar; end; end`). | MEDIUM — open classes, reopening modules |
| PHP | Namespaces (`namespace App\Models;`). | LOW — one namespace per file |
| C# | Namespaces + class nesting. File-scoped namespaces (C# 10+). | LOW-MEDIUM — file-scoped changes structure |
| Swift | No module keyword in source. Module = target name from build system. | HARD — need build system info |

Sub-questions:
- For Swift, the module name comes from the build system (Xcode target), not from source code. Tree-sitter can't extract it. Do we leave scope empty for Swift? Or use the directory path as a proxy?
- For C, there are no modules. Is `scope: vec![]` acceptable? The key becomes `c|||fn||||||my_function|||src/utils.c|||d0` with an empty scope field. Does `split("|||")` still give 6 fields? Yes — empty string between `|||` delimiters is valid.
- For Ruby, classes can be reopened (`class String; def custom_method; end; end`). The scope for `custom_method` is `String`, but the file might be `lib/extensions/string_ext.rb`. The scope is "logically" `String` but "physically" `lib.extensions.string_ext`. Which one do we use?
- For Python, a function inside a function (`def outer(): def inner(): ...`) — is the scope `["outer"]` or is `inner` a separate entity with scope pointing to `outer`?

**Decision path**: Best-effort. Empty scope is valid (degrades to v2 behavior, which already works). Each language improves independently. Priority order: Rust > Python > TypeScript/JavaScript > Java > Go > rest.

**Difficulty**: MEDIUM overall, but LOW per individual language since each is independent.

---

### Area 2: String Interning (Q2)

**Q2: Should keys be interned?**

With 50K-60K entities (v3 exhaustive coverage estimate from ISGL1 v3 research) and ~300K edges (each storing two keys), memory layout matters.

Current situation:
```rust
// Every edge stores two full String keys:
pub struct DependencyEdge {
    pub from_key: Isgl1Key,  // Isgl1Key(String) — full allocation
    pub to_key: Isgl1Key,    // Isgl1Key(String) — full allocation
    ...
}

// With 300K edges, that's 600K String allocations just for keys in edges.
// Average key length ~60 bytes → ~36MB of key strings in edges alone.
```

With interning:
```rust
// Keys are interned — edges store cheap 8-byte handles:
pub struct DependencyEdge {
    pub from_key: InternedKey,  // 8 bytes, pointer comparison for Eq
    pub to_key: InternedKey,    // 8 bytes
    ...
}

// With 300K edges: 600K × 8 bytes = ~4.8MB (vs ~36MB)
// Plus interner table: 60K unique keys × ~60 bytes = ~3.6MB
// Total: ~8.4MB vs ~36MB — 4.3x reduction
```

Sub-questions:
- Is 36MB vs 8.4MB significant? For an embedded tool, yes. For a server, probably not. The v2.0.0 PRD says "embeddable like SQLite." SQLite is obsessive about memory. So this matters.
- Which interner crate? Options: `lasso` (most popular, 0.7.3, concurrent), `string-interner` (simpler, no concurrent), `internment` (global, Arc-based). All are well-maintained.
- Does interning complicate serialization? Yes — you can't serialize an interned key handle to JSON. You'd need to resolve it back to a string at the HTTP/MCP boundary. But this is the same as the Candidate D model (struct internally, string externally).
- Can this be added later without breaking changes? YES. `InternedKey` can implement `Display` with the same `|||` format. External consumers never see the difference.

**Decision path**: Don't intern in v1. Measure memory usage at 50K entities. If >100MB, add interning. If <100MB, skip it.

**Difficulty**: LOW — standard Rust pattern, well-documented crates, can be deferred.

---

### Area 3: Path Normalization (Q3)

**Q3: When are two file paths "the same"?**

```
src/auth.rs          — canonical
./src/auth.rs        — leading dot-slash
src//auth.rs         — double slash
src\auth.rs          — Windows backslash
/abs/path/src/auth.rs — absolute path
```

Should all of these produce the same key? If not, two entities in the "same" file get different keys depending on how the path was provided.

Sub-questions:
- **When does this actually happen?** Tree-sitter receives file paths from `walkdir` during ingestion. `walkdir` returns consistent relative paths. So within a single ingest, paths are consistent. BUT: if an LLM sends a query with `./src/auth.rs` and the key stores `src/auth.rs`, the lookup fails.
- **Windows paths**: Does Parseltongue run on Windows? The README says yes (v1.7.2 has Windows SQLite workflow). So `\` vs `/` is a real concern in keys.
- **Absolute vs relative**: The current `extract_semantic_path()` (isgl1_v2.rs:127) strips the extension and replaces separators. It doesn't handle absolute paths. Does the new key store relative or absolute paths?
- **Trailing slashes, double slashes**: These are pathological but possible. A single `canonicalize_file_path()` function fixes all of them.

Concrete normalization function:
```rust
fn normalize_file_path(path: &str) -> String {
    path.replace('\\', "/")         // Windows → Unix
        .trim_start_matches("./")   // Strip leading ./
        .replace("//", "/")         // Collapse double slashes
        // Do NOT resolve to absolute path — keep relative
        .to_string()
}
```

Sub-questions:
- Should this function live in `rust-llm-core` (so all crates use it) or in `rust-llm-parse` (so only the extractor normalizes)? Answer: `rust-llm-core`. The HTTP server also needs to normalize incoming query parameters.
- Should we normalize on key CREATION or on key COMPARISON? Answer: on creation. Normalize once, compare cheaply forever. This is what SCIP does (canonical form at emit time).

**Decision path**: One `normalize_file_path()` function in `rust-llm-core`, called at key construction time. 5 lines of code.

**Difficulty**: LOW — one function, well-defined rules.

---

### Area 4: Scope Extraction Depth (Q4)

**Q4: Best-effort empty scope or block on full module paths for all 12 languages?**

This is the most important process question, not a technical one. Two strategies:

**Strategy A: Block until complete**
```
Week 1-2: EntityKey struct
Week 2-6: Write scope extractors for ALL 12 languages
Week 7+:  Start building consumers (graph, context, safety)
          — delayed 4 weeks
```

**Strategy B: Best-effort, ship empty**
```
Week 1-2: EntityKey struct with scope: Vec<String>
Week 2-3: Scope extractor for Rust (primary language, rust-analyzer validates)
Week 3+:  Start building consumers immediately
          — scope for other languages improves in background
```

Sub-questions:
- **What happens when scope is empty?** The key becomes `python|||fn||||||login|||src/auth.py|||d0`. The empty scope means: "we know this is a Python function called `login` in `src/auth.py`, but we don't know its module path." Is this worse than v2? No — v2 has `python:fn:login:__src_auth:T1706284800` which also has no module path. Empty scope = v2 parity.
- **Do consumers (graph, context, rules) break with empty scope?** Let's check each:
  - Graph algorithms: operate on entity keys as opaque nodes. Empty scope doesn't affect SCC, PageRank, k-core, Leiden, blast radius. **NO BREAK.**
  - Context optimizer: ranks entities by relevance. Module path improves ranking ("same module = more relevant"). Empty scope = slightly worse ranking but not broken. **DEGRADED, NOT BROKEN.**
  - Ascent rules: can pattern-match on scope. Rules like "entities in same module that call each other" need scope. Empty scope = rule doesn't fire. **SILENT LOSS**, but the rule can check for non-empty scope first.
  - Cross-lang edges: match entities by name + file path across languages. Scope helps disambiguate. Empty scope = more false matches. **DEGRADED, NOT BROKEN.**
- **Does empty scope cause key collisions?** Only if two entities in the same file have the same name and kind but different scopes. Example: `mod a { fn login() {} } mod b { fn login() {} }` in `src/auth.rs`. With empty scope: both produce `rust|||fn||||||login|||src/auth.rs|||d0`. **COLLISION.** The discriminator must handle this — `d0` vs `d1` by position.
- **How many languages need scope for collision avoidance?** Only languages with nested modules/classes in the same file: Rust (nested `mod`), Python (nested `class/def`), Ruby (open classes), C++ (nested namespaces), C# (nested classes). Java, Go, C, PHP, Swift, TypeScript — one entity scope per file is typical.

**Decision path**: Strategy B. Ship with Rust scope (validated by rust-analyzer bridge). Python scope second. Others as needed. Empty scope + discriminator fallback prevents collisions.

**Difficulty**: MEDIUM — the strategy is clear but each language's scope extractor is its own unit of work.

---

### Area 5: External Entities (Q5)

**Q5: How to key external dependencies?**

When code calls `std::io::Read::read()` or `numpy.array()`, the callee doesn't have a local file. What goes in the key?

Proposal from Prep-V200 doc (section 6.4):
```
rust|||trait|||std::io|||Read|||EXTERNAL:std|||d0
python|||fn|||numpy|||array|||EXTERNAL:numpy|||d0
```

Sub-questions:
- **Is `EXTERNAL:` a magic prefix?** Yes. Any key with `file_path` starting with `EXTERNAL:` means "this entity is not in the analyzed codebase." Is this robust? What if someone has a directory literally called `EXTERNAL:std`? Extremely unlikely on Unix (`:` is valid in directory names but nobody does this), impossible on Windows (`:` is reserved). Acceptable.
- **How deep does the external key go?** For `std::collections::HashMap::insert()`:
  - Minimal: `rust|||fn|||std|||insert|||EXTERNAL:std|||d0`
  - Full: `rust|||method|||std::collections::HashMap|||insert|||EXTERNAL:std|||String_V`
  - Which one? The minimal version loses disambiguation. Two methods named `insert` in `std` would collide. The full version requires knowing the full qualified path of the external entity — which tree-sitter can't always determine.
- **How does tree-sitter know something is external?** It doesn't. When code says `use std::io::Read;` and then calls `reader.read()`, tree-sitter sees the call but doesn't know `read` is from `std::io::Read` vs a local definition. Resolving external references requires either:
  1. rust-analyzer (for Rust only — it resolves all types)
  2. Import statement analysis (for all languages — parse `use`/`import` statements)
  3. Heuristic: "if we can't find the callee in our entity index, it's external"
  - Option 3 is simplest and language-agnostic. The extractor builds the entity index first, then for each call edge, if the target isn't in the index, create an `EXTERNAL:` entity.
- **Do external entities participate in graph algorithms?** They should — they're real dependencies. A function that calls `serde::Serialize::serialize()` has a dependency on `serde`. PageRank should count this. But external entities have no code body, no line range, no scope — they're thin nodes.
- **How many external entities per codebase?** For a Rust crate using 20 dependencies, rough estimate: 200-500 external entities (functions/types called from deps). For a Python project with 30 pip packages: 300-1000. This is significant — external entities could be 10-50% of the graph.

**Decision path**: Use `EXTERNAL:{package}` in `file_path`. Populate scope from import statements (best-effort). Create external entities lazily when a call target isn't in the entity index. Let graph algorithms include them.

**Difficulty**: LOW-MEDIUM — the format is simple, but the "when to create" logic needs thought.

---

### Area 6: rust-analyzer Bridge (Q6, Q18)

**Q6: How to map rust-analyzer DefIds to EntityKeys?**

This is the hardest single question because it requires TWO INDEPENDENT SYSTEMS to produce the SAME key for the same entity.

```
tree-sitter (rust-llm-parse)     rust-analyzer (rust-llm-bridge)
         │                                │
         ▼                                ▼
    EntityKey for                   EntityKey for
    fn login()                      fn login()
         │                                │
         └──────── MUST BE EQUAL ─────────┘
```

How tree-sitter sees `fn login()`:
```
tree-sitter node:
  kind: "function_item"
  name: "login"
  file: "src/auth.rs"
  start_line: 15, end_line: 42
  parent: "mod handlers" (from AST walk)
```

How rust-analyzer sees `fn login()`:
```
rust-analyzer DefId:
  FunctionId(InternKey(423))
  crate: my_crate
  module: auth::handlers
  name: login
  qualified: my_crate::auth::handlers::login
  file: src/auth.rs
  range: 15:1..42:1
```

The tree-sitter extractor has: `name=login`, `file=src/auth.rs`, `scope=["handlers"]` (maybe, if scope extraction works).
The rust-analyzer bridge has: `name=login`, `file=src/auth.rs`, `scope=["my_crate", "auth", "handlers"]` (always complete).

Sub-questions:
- **Scope depth mismatch**: rust-analyzer always knows the full module path. Tree-sitter might only know the immediate parent. If scope is `["handlers"]` from tree-sitter but `["my_crate", "auth", "handlers"]` from rust-analyzer, the keys won't match.
  - Solution A: rust-analyzer "dumbs down" its scope to match tree-sitter's depth. Wasteful — throws away good data.
  - Solution B: tree-sitter always extracts full scope for Rust (it can — `mod` declarations are explicit). Then both produce `["my_crate", "auth", "handlers"]`.
  - Solution C: The bridge produces facts that ENRICH tree-sitter's EntityKey — it doesn't create its own. The bridge sees DefId → looks up the tree-sitter entity by name + file + line range → adds compiler-truth attributes. No second key creation.
  - **Solution C is architecturally cleanest.** One key producer (tree-sitter), one enricher (rust-analyzer). No reconciliation needed.
- **But Solution C means rust-analyzer facts can't exist without tree-sitter facts.** If tree-sitter misses an entity (macro-generated code, for example), rust-analyzer can't contribute it. Is this acceptable?
  - Macro-generated entities are invisible to tree-sitter but visible to rust-analyzer (which expands macros). If we want those in the graph, the bridge MUST be able to create new entities, not just enrich existing ones.
  - Compromise: bridge can create entities for things tree-sitter missed, but uses the SAME key format. Since it has the full qualified path, its keys are a superset of tree-sitter's.
- **How does rust-analyzer identify "this is the same entity tree-sitter extracted"?** Match on: (file_path, name, entity_kind, approximate_line_range). If tree-sitter says "fn login at src/auth.rs:15-42" and rust-analyzer says "fn login at src/auth.rs:15-42", it's the same entity. This is the same 3-priority matching as `match_entity_with_old_index()` in isgl1_v2.rs — but between two extractors instead of between two time points.

**Decision path**: Solution C + compromise. Tree-sitter is the primary key producer. rust-analyzer enriches existing entities and creates new ones for macro-expanded/generated code. Matching is by (file, name, kind, line range).

**Difficulty**: MEDIUM — bounded but requires careful implementation.

---

**Q18: rust-analyzer scope vs tree-sitter scope reconciliation**

Even with Solution C, there's a subtlety: if tree-sitter extracts `scope: ["handlers"]` and rust-analyzer knows it's `["my_crate", "auth", "handlers"]`, should the enrichment step UPGRADE the scope?

Sub-questions:
- If we upgrade the scope, the key changes (because scope is part of the key). That means: entity created with short scope → enriched with full scope → key changes → all edges referencing the old key are broken.
- Solution: don't change the key after creation. Store the rust-analyzer's full module path as a SEPARATE field (`compiler_scope: Vec<String>`), not in the key's `scope` field. The key is immutable after creation.
- OR: for Rust specifically, always extract full scope from tree-sitter (since `mod` declarations are explicit). Then rust-analyzer's scope and tree-sitter's scope already match.

**Decision path**: For Rust, extract full scope from tree-sitter (achievable — `mod` declarations are in the AST). For other languages, scope comes from tree-sitter only (no bridge). Rust-analyzer enrichment adds attributes, not key changes.

**Difficulty**: LOW for Rust (mod declarations are explicit). N/A for other languages (no bridge).

---

### Area 7: Discriminator Population (Q7, Q13)

**Q7: Discriminator when tree-sitter can't extract parameter types**

The discriminator field solves overloads. Three fallback levels from the Prep doc:

```
Level 1: ParamTypes   — "String_int" (if tree-sitter can extract param types)
Level 2: Index        — "d0", "d1", "d2" (positional, by order of appearance in file)
Level 3: ContentHash  — "h_ab12cd34" (hash of function body, last resort)
```

Sub-questions:
- **When can tree-sitter extract param types?**
  | Language | Tree-sitter param types? | Notes |
  |----------|------------------------|-------|
  | Rust | YES — type annotations are mandatory | `fn parse(input: &str, depth: usize)` → `str_usize` |
  | TypeScript | YES — type annotations common | `function parse(input: string, depth: number)` → `string_number` |
  | Java | YES — type annotations mandatory | `void parse(String input, int depth)` → `String_int` |
  | Go | YES — type annotations mandatory | `func parse(input string, depth int)` → `string_int` |
  | C/C++ | YES — type declarations mandatory | `int convert(double x)` → `double` |
  | Python | SOMETIMES — type hints optional | `def parse(input: str, depth: int)` → `str_int` if hints present, fallback if not |
  | JavaScript | NO — no type system | Always falls back to Level 2 or 3 |
  | Ruby | NO — no type system | Always falls back to Level 2 or 3 |
  | PHP | SOMETIMES — type hints optional (PHP 7+) | `function parse(string $input, int $depth)` → `string_int` if hints present |
  | Swift | YES — type annotations mandatory | `func parse(input: String, depth: Int)` → `String_Int` |
  | C# | YES — type annotations mandatory | `void Parse(string input, int depth)` → `string_int` |

- **For Python/JavaScript/Ruby without type info, is positional index stable?** "d0" means "first overload in file order." But Python doesn't HAVE overloads — each `def parse()` at the same scope level replaces the previous one. So collisions in Python mean: same name + same scope but DIFFERENT scope levels (e.g., `parse` at module level vs `parse` inside a class). Positional index handles this.
- **For JavaScript, do we ever have two functions with the same name in the same file?** Yes — named function expressions, methods in different objects, nested functions. Example:
  ```javascript
  const api = { parse: function(input) { ... } };
  const xml = { parse: function(input) { ... } };
  ```
  Tree-sitter sees both as `parse` in the same file. Scope should disambiguate (`api.parse` vs `xml.parse`). If scope extraction fails, discriminator `d0` vs `d1` by line order.
- **ContentHash (Level 3): When is this needed?** Only when: (a) no type info, (b) same name and scope, (c) same line order across re-parses. This is extremely rare. It's a safety net, not a primary mechanism.
- **ContentHash stability**: If someone changes the function body but not the signature, the ContentHash changes, the key changes, all edges break. Is this acceptable? For Level 3 entities (worst case), yes — they were already degenerate cases. The key SHOULD change if we can't distinguish the entity any other way.

**Decision path**: Use Level 1 (ParamTypes) for statically-typed languages. Use Level 2 (Index) for dynamically-typed languages. Reserve Level 3 (ContentHash) for true ambiguity. Document which languages use which level.

**Difficulty**: LOW — tree-sitter already extracts parameter types for most languages.

---

**Q13: How to format the discriminator string from param types?**

When tree-sitter extracts `fn parse(input: &str, depth: usize)`, the discriminator should be `str_usize`. But details matter:

Sub-questions:
- **Reference/pointer stripping**: Is `&str` → `str` or `ref_str`? Proposal: strip references. `&str`, `&mut str`, `*const str` all → `str`. Reason: overloads differentiate on base type, not pointer/reference wrapper.
- **Generic type handling**: `fn process(items: Vec<String>)` → discriminator `Vec_String` or `Vec`? Proposal: include generic params. `Vec_String`. Otherwise `Vec<String>` and `Vec<u8>` collide.
- **Separator between param types**: Underscore (`_`) could collide with param type names containing underscores. Use `_` anyway? Or a different separator? Proposal: `_` is fine. Type names with underscores (e.g., `c_int`) are rare, and the discriminator doesn't need to be parseable — just unique.
- **Order**: Left to right, same as declaration order. `fn(a: i32, b: String)` → `i32_String`.
- **Empty params**: `fn init()` → discriminator `d0` (no params = default discriminator).
- **Self param in methods**: `fn login(&self, username: &str)` → discriminator `str` (skip `self`/`&self`/`&mut self`).

**Decision path**: Strip references, include generic params, `_` separator, skip self. One `format_discriminator()` function.

**Difficulty**: LOW — string formatting, well-defined rules.

---

### Area 8: Taxonomy Mismatches Between Research Docs (Q8, Q9, Q10, Q11, Q15, Q16, Q17)

These questions emerge from reading the actual codebase and noticing that the THREE research documents (Prep-V200, ISGL1 v3 Research, entities.rs) don't fully agree.

---

**Q8: EntityKind taxonomy mismatch**

The Prep-V200 doc (Candidate D) proposes:
```rust
pub enum EntityKind {
    Function, Method, Class, Struct, Enum, Trait, Interface,
    Module, Namespace, Impl, Macro, Variable, Constant,
    Table, View, ImportBlock, TestFunction,
}
// 17 variants
```

The current codebase `EntityType` (entities.rs:101) has:
```rust
pub enum EntityType {
    Function, Method, Struct, Enum, Trait, Interface, Module,
    ImplBlock { trait_name: Option<String>, struct_name: String },
    Macro, ProcMacro, TestFunction, Class, Variable, Constant,
    Table, View,
}
// 16 variants (ImplBlock carries data)
```

The ISGL1 v3 research proposes 5 entity CLASSES:
```
CoreCode, TestCode, ImportBlock, GapFragment, UnparsedConstruct
```

Sub-questions:
- `ProcMacro` exists in v1 but not in the Prep-V200 EntityKind. Was it intentionally dropped or accidentally omitted?
- `Namespace` is new in Prep-V200 (for C++ `namespace {}` and C# `namespace {}`). Currently these are probably classified as `Module`. Is `Namespace` distinct from `Module`? In Rust, no. In C++, yes — namespaces are not modules.
- `ImportBlock` appears in BOTH the Prep-V200 EntityKind AND the v3 entity class taxonomy. Are these the same concept or different? In v3, `ImportBlock` is an entity CLASS (like CoreCode). In Prep-V200, it's an entity KIND (like Function). These are orthogonal dimensions: a `CoreCode` entity has kind `Function`. An `ImportBlock` entity has kind... `ImportBlock`? That's circular. Resolution: entity CLASS and entity KIND are separate fields. An import entity has `class: ImportBlock` and `kind: ImportStatement` (or similar).
- v3 proposes `GapFragment` and `UnparsedConstruct` as entity classes. These don't appear in Prep-V200's EntityKind at all. Should they be EntityKind variants too? Or is EntityKind only for "real" code entities, and GapFragment/UnparsedConstruct are identified solely by their EntityClass?

**Decision path**: EntityKind = what the entity IS (function, class, struct). EntityClass = how exhaustive coverage categorizes it (core code, test, import, gap, unparsed). Two separate enums. `ProcMacro` kept. `Namespace` added for C++/C#.

**Difficulty**: LOW — taxonomy alignment, no code complexity.

---

**Q9: ImplBlock data loss**

Current `EntityType::ImplBlock` carries data:
```rust
ImplBlock {
    trait_name: Option<String>,  // e.g., Some("Display")
    struct_name: String,         // e.g., "MyStruct"
}
```

Prep-V200's `EntityKind::Impl` is a flat variant with no data. The trait and struct names would need to go... where?

Sub-questions:
- Into the `scope` field? `scope: ["MyStruct"]` or `scope: ["MyStruct", "Display"]`?
- Into the `name` field? `name: "impl Display for MyStruct"`?
- Into a separate field on `EntityKey`? That would add a field just for impl blocks.
- SCIP's approach: `UserService+Serialize+` suffix. The `+` suffix means "impl block" and the trait name is part of the descriptor chain.
- Proposal: `name: "impl_Display_for_MyStruct"` with `kind: Impl`. The name is human-readable and unique. The scope carries the module path.

**Decision path**: Encode impl block identity in the `name` field as `impl_{Trait}_for_{Type}` or `impl_{Type}` for inherent impls. No extra fields needed.

**Difficulty**: LOW — string formatting convention.

---

**Q10: LanguageSpecificSignature only covers 5 of 15 languages**

Current `entities.rs` has `LanguageSpecificSignature` enum:
```rust
pub enum LanguageSpecificSignature {
    Rust(RustSignature),
    JavaScript(JavascriptSignature),
    TypeScript(TypeScriptSignature),
    Python(PythonSignature),
    Java(JavaSignature),
    // Missing: Go, C, C++, Ruby, PHP, C#, Swift, Kotlin, Scala, SQL
}
```

Sub-questions:
- Does v2.0.0 EntityKey need language-specific signature data? No — the key is language-agnostic by design. Signature data stays in the ENTITY payload, not in the key.
- Does v2.0.0 need language-specific signatures at all? Yes — for discriminator extraction. The Rust signature tells you parameter types for overload disambiguation.
- But the key format paper says to extract param types from tree-sitter, not from a separate signature struct. So the discriminator comes directly from tree-sitter AST, not from `LanguageSpecificSignature`. This makes `LanguageSpecificSignature` a v1 artifact that v2.0.0 may not need in the same form.

**Decision path**: EntityKey doesn't carry language-specific data. Discriminator extraction reads tree-sitter AST directly. `LanguageSpecificSignature` may be simplified or removed in v2.0.0.

**Difficulty**: LOW — this is a v2.0.0 simplification, not a complication.

---

**Q11: EntityClass taxonomy mismatch**

Current `entities.rs` has 2 classes:
```rust
pub enum EntityClass {
    TestImplementation,
    CodeImplementation,
}
```

ISGL1 v3 research proposes 5 classes:
```
CoreCode, TestCode, ImportBlock, GapFragment, UnparsedConstruct
```

Sub-questions:
- Is `CodeImplementation` → `CoreCode` a rename? Yes.
- Is `TestImplementation` → `TestCode` a rename? Yes.
- When are `ImportBlock`, `GapFragment`, `UnparsedConstruct` introduced? They exist in v3 exhaustive coverage but the Prep-V200 key format doc focuses on the key struct, not the entity class enum. The class enum is a separate concern.
- Does the key need to contain the entity class? The ISGL1 v3 format has `entity_type` which could be `import`, `gap`, etc. But the Prep-V200 EntityKind enum has `ImportBlock` as a kind. So kind and class overlap. Resolution per Q8: keep them separate.

**Decision path**: Expand EntityClass to 5 variants per v3 research. This is orthogonal to the key format.

**Difficulty**: LOW — enum expansion, no key format impact.

---

**Q15: What happens to the `Isgl1Key` newtype?**

Current codebase has `Isgl1Key(String)` (entities.rs:987) — a newtype wrapper that enforces non-empty keys. It's used in `DependencyEdge`:

```rust
pub struct DependencyEdge {
    pub from_key: Isgl1Key,
    pub to_key: Isgl1Key,
    ...
}
```

v2.0.0 replaces `String`-based keys with `EntityKey` struct. So:
- `Isgl1Key` is deleted.
- `DependencyEdge` changes to `from_key: EntityKey, to_key: EntityKey`.
- `SlimEntityGraphSnapshot.isgl1_key: String` changes to `entity_key: EntityKey` (or its serialized form).
- `CodeEntity.isgl1_key: String` changes to `entity_key: EntityKey`.
- `validate_isgl1_key()` which currently checks for hyphens (!) is deleted entirely.

Sub-questions:
- How many call sites reference `Isgl1Key` or `isgl1_key`? Every entity, every edge, every query handler. This is a pervasive change.
- Is this a migration concern? No — v2.0.0 is a clean break. Old types stay in parseltongue-core (frozen v1.x). New types live in rust-llm-core.
- Does `validate_isgl1_key()` (entities.rs:755) make sense for v3 keys? No — it checks for hyphens, which are a v1 artifact. `|||` keys don't have hyphens. Delete it.

**Decision path**: Clean break. New `EntityKey` struct replaces `Isgl1Key`. No migration, no backward compat.

**Difficulty**: NONE — clean break means no migration complexity.

---

**Q16: Does v2.0.0 keep TemporalState?**

`CodeEntity` currently has `TemporalState` (entities.rs:158) tracking current/future state with `TemporalAction` (Create/Edit/Delete). This was designed for the original Tool 2 "LLM suggests changes" workflow.

Sub-questions:
- Does the FactSet protocol in v2.0.0 have temporal state? The PRD doesn't mention it. The FactSet is described as a typed collection of facts from a single analysis pass.
- If TemporalState is dropped, `CodeEntity` simplifies significantly. No `current_code`/`future_code` duality. No `TemporalAction` enum. No `validate_with_indicators()`.
- This isn't a key format question directly, but it affects the entity payload that keys identify.

**Decision path**: Likely dropped in v2.0.0. FactSet entities are point-in-time snapshots. Temporal tracking (if needed) moves to a version-comparison layer.

**Difficulty**: LOW — simplification.

---

**Q17: ISGL1 v3 research and Prep-V200 disagree on line numbers in keys**

ISGL1 v3 research format:
```
{language}|||{entity_type}|||{entity_name}|||{file_path}|||{line_start}|||{line_end}
```
Line numbers ARE part of the key. 6 fields.

Prep-V200 recommendation (Candidate D + B):
```
{language}|||{kind}|||{scope}|||{name}|||{file_path}|||{discriminator}
```
Line numbers are NOT part of the key. 6 fields, but `scope` and `discriminator` replace `line_start` and `line_end`.

Sub-questions:
- Which is canonical? Prep-V200 is newer (2026-02-16) and more thoroughly researched (762 lines with competitor analysis). ISGL1 v3 research (2026-02-14) is earlier and focused on delimiter + coverage, not key structure.
- The v3 research explicitly says line numbers make keys UNSTABLE: "Stability across edits: Unstable (line numbers shift)." The Prep-V200 says M3: "Stable across minor edits." These are contradictory.
- The v3 research argued instability doesn't matter because "the graph is rebuilt from source every time." But the Prep-V200 argues stability DOES matter because keys appear in HTTP responses and LLM conversations — a key that changes on every edit breaks LLM tool chains.
- **Resolution**: Prep-V200 wins. No line numbers in keys. Scope + discriminator instead. The v3 research's contribution is the `|||` delimiter and the entity class taxonomy, NOT the line-numbers-in-keys format.

**Decision path**: Use Prep-V200 format (scope + discriminator, no line numbers). Adopt v3's `|||` delimiter and entity class taxonomy.

**Difficulty**: NONE — just needs documentation to clarify which doc is canonical.

---

### Area 9: Cross-Language Key Matching (Q14)

**Q14: How do cross-language edges match entities?**

Cross-language edges connect entities across language boundaries. Examples:
- Rust `#[pyfunction] fn authenticate()` is called from Python `authenticate()`
- TypeScript `import { process } from 'wasm-bridge'` calls Rust `#[wasm_bindgen] fn process()`
- Python gRPC stub `AuthService.Login()` connects to Go `func (s *Server) Login()`

Sub-questions:
- **Name matching is necessary but not sufficient.** `authenticate` in Rust and `authenticate` in Python could be the same function (via PyO3) or two unrelated functions. What additional signals disambiguate?
  - **Annotation matching**: `#[pyfunction]`, `#[wasm_bindgen]`, `@grpc.method` — these bridge annotations explicitly declare cross-language exposure.
  - **File proximity**: If `auth.rs` and `auth.py` are in the same directory, they're more likely to be related.
  - **Import matching**: Python's `from auth_module import authenticate` + Rust's `#[pymodule] fn auth_module` — the module name matches.
- **Key compatibility**: A Rust entity key and a Python entity key will NEVER be string-equal (different language prefix). Cross-language edges store (rust_key, python_key, CrossLangCall). The matching logic is separate from key equality — it's a cross-language RESOLVER that creates edges, not a key comparison.
- **Does this affect key format?** Not directly. The key format doesn't need to change for cross-language support. The resolver just needs to be able to extract `name` and `file_path` from keys for matching. With `|||` delimiter and `split("|||")`, this is trivial.

**Decision path**: Cross-language matching is a resolver concern (in rust-llm-crosslang), not a key format concern. The key format supports it by being human-readable and splittable.

**Difficulty**: MEDIUM for the resolver, but LOW for key format impact.

---

## What v2 Problems Are FULLY Resolved

Cross-referencing the 8 problems from `Prep-V200-Key-Format-Design.md`:

| # | Problem | Status | Resolution |
|---|---------|--------|-----------|
| 1.2 | No scope/namespace awareness | RESOLVED | `scope: Vec<String>` in EntityKey struct |
| 1.3 | Overload/signature collisions | RESOLVED | `discriminator` field with ParamTypes / Index / ContentHash fallback chain |
| 1.4 | No module path | RESOLVED | `scope` field carries module path. Best-effort empty scope for unknown cases. |
| 1.5 | No file context | RESOLVED | `file_path: String` carries raw file path. No sanitization. |
| 1.6 | Delimiter collisions (`:`) | RESOLVED | `\|\|\|` delimiter. Zero collisions proven across 15 languages. |
| 1.7 | Generic type sanitization burden | RESOLVED | `\|\|\|` makes `<>` safe. `sanitize_entity_name_for_isgl1()` deleted. |
| 1.8 | Birth timestamp is a lie | RESOLVED | Removed. Discriminator is honest (param types or positional index). |
| NEW | CozoDB dependency in key format | RESOLVED | v2.0.0 drops CozoDB. Key format has zero CozoDB constraints. |

**Score: 8/8 problems resolved in the design. 0 unresolved design problems.**

---

## Question Difficulty Distribution

```
18 total questions across 9 areas:

NONE:     3 questions (17%)  — already resolved, just needs documentation
LOW:      9 questions (50%)  — standard engineering, proposals exist, bounded
MEDIUM:   6 questions (33%)  — requires design work, but each is bounded
HIGH:     0 questions (0%)   — nothing fundamentally unsolved
UNKNOWN:  0 questions (0%)   — no "we don't know what we don't know"

By area:
  Area 1 (Scope):           Q1 LOW, Q12 MEDIUM        — start with Vec<String>
  Area 2 (Interning):       Q2 LOW                     — defer, measure first
  Area 3 (Path):            Q3 LOW                     — one function
  Area 4 (Depth):           Q4 MEDIUM                  — process decision, not technical
  Area 5 (External):        Q5 LOW-MEDIUM              — EXTERNAL: prefix
  Area 6 (Bridge):          Q6 MEDIUM, Q18 LOW         — Solution C + compromise
  Area 7 (Discriminator):   Q7 LOW, Q13 LOW            — format rules defined
  Area 8 (Taxonomy):        Q8-Q11 LOW, Q15-Q17 NONE   — alignment, not invention
  Area 9 (Cross-lang):      Q14 MEDIUM (resolver, not key) — doesn't affect format
```

---

## Revised Risk Rating

### Original Rating (from opinion doc)

```
rust-llm-core   Key format is MEDIUM
```

### Evidence-Based Revision

```
DESIGN risk:           LOW     — Candidate D+B recommended, 8/8 problems solved
DELIMITER risk:        NONE    — ||| proven, zero collisions
IMPLEMENTATION risk:   LOW     — 366 lines of v2 as reference, ~60 lines for new struct
INTEGRATION risk:      LOW-MED — Q6 (rust-analyzer mapping) needs design, Solution C identified
TAXONOMY risk:         LOW     — Q8-Q11 are alignment exercises, not inventions
CROSS-LANG risk:       LOW     — key format supports it; resolver is separate concern
OVERALL risk:          LOW     — down from MEDIUM
```

### What Changed My Assessment

1. **3,457 lines of existing research** across 5 documents. The RFC already exists.
2. **All 8 v2 problems are RESOLVED** in the design, not just identified.
3. **18 sub-questions found**, but 12 of 18 are LOW or NONE difficulty.
4. **3 of 8 crates have ZERO ambiguity** (rules, HTTP, MCP).
5. **The two hardest questions** (Q6: bridge mapping, Q14: cross-lang resolver) don't affect the key FORMAT — they affect the key PRODUCTION logic, which is in separate crates.
6. **The taxonomy mismatches** (Q8-Q11, Q15-Q17) are alignment exercises — choosing which existing proposal is canonical — not open research.

---

## Resolve Order (Implementation Sequence)

```
Week 1:   Q8, Q9, Q10, Q11, Q15, Q16, Q17  — taxonomy alignment (all LOW/NONE)
          Write EntityKey struct, EntityKind enum, EntityClass enum
          Write Display impl with ||| serialization
          Write normalize_file_path() (Q3)
          Write FromStr impl (parse ||| string back to struct)

Week 2:   Q1, Q13, Q7  — scope as Vec<String>, discriminator format rules
          Write Rust scope extractor (Q12 partial — Rust only)
          Write format_discriminator() function

Week 3:   Q4, Q12  — scope extraction for Python, TypeScript, Java
          Test with real codebases, verify no collisions

Week 4:   Q6, Q18  — rust-analyzer bridge mapping (Solution C)
          Test: tree-sitter key == bridge-enriched key for same entity

Week 5:   Q5, Q14  — external entities, cross-lang resolver prototype
          Q2 — measure memory, decide on interning

All 18 questions resolved in 5 weeks, parallelizable with other crate work.
```

---

## Summary

```
INITIAL CLAIM:    "Key format is MEDIUM risk — needs its own RFC"
EVIDENCE:         3,457 lines of existing research across 5 documents
                  8/8 design problems resolved
                  18 sub-questions found, 12 LOW/NONE + 6 MEDIUM + 0 HIGH
                  3/8 crates have ZERO remaining ambiguity
                  5 open questions from Prep doc + 13 new from code reading
                  All questions have proposals or clear decision paths
REVISED CLAIM:    "Key format is LOW risk — RFC exists, questions are bounded"
```

---

*Generated 2026-02-16. Expanded risk reassessment for Parseltongue v2.0.0 ISGL1 key format. 18 questions across 9 areas.*
