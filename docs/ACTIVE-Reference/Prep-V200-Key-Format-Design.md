# Prep: v2.0.0 Key Format Design

**Date**: 2026-02-16
**Status**: Research / Pre-Implementation
**Scope**: Entity key format for rust-llm-core (v2.0.0 clean break)
**Criticality**: HIGHEST -- every crate in v2.0.0 depends on this decision

---

## Why This Is the Most Critical Decision in v2.0.0

The entity key format is the single type that propagates through every layer of the v2.0.0 architecture:

```
rust-llm-01 (fact extractor)     -- GENERATES keys
rust-llm-02 (cross-lang edges)   -- REFERENCES keys across language boundaries
rust-llm-03 (rust-analyzer)      -- MAPS rust-analyzer DefIds TO keys
rust-llm-04 (reasoning engine)   -- JOINS on keys in Ascent Datalog rules
rust-llm-05 (knowledge store)    -- INDEXES on keys in HashMaps
rust-llm-06 (HTTP server)        -- RETURNS keys in JSON responses
rust-llm-07 (MCP server)         -- RETURNS keys to Claude/Cursor/VS Code
```

If we get the key format wrong, every crate needs to change. If we get it right, it becomes the stable identity layer that everything else is built on top of.

---

## Table of Contents

1. [ISGL1 v1/v2 Problems](#1-isgl1-v1v2-problems)
2. [How Other Tools Identify Entities](#2-how-other-tools-identify-entities)
3. [Requirements for Our Key Format](#3-requirements-for-our-key-format)
4. [Candidate Designs with Tradeoffs](#4-candidate-designs-with-tradeoffs)
5. [What Breaks If We Get It Wrong](#5-what-breaks-if-we-get-it-wrong)
6. [Recommendation](#6-recommendation)

---

## 1. ISGL1 v1/v2 Problems

### 1.1 The Current Format (ISGL1 v2.1)

The current key format in `parseltongue-core/src/isgl1_v2.rs` is:

```
{language}:{entity_type}:{entity_name}:{semantic_path}:T{birth_timestamp}
```

Concrete examples:
```
rust:fn:handle_auth:__src_auth:T1706284800
python:class:UserService:__src_services_user:T1706300000
csharp:fn:List__lt__string__gt__:__Controllers_ApiController:T1706310000
```

### 1.2 Problem: No Scope/Namespace Awareness

ISGL1 keys are flat. There is no representation of the containment hierarchy.

```rust
// File: src/auth.rs
mod auth {
    mod handlers {
        fn login() { ... }      // Key: rust:fn:login:__src_auth:T...
    }
    mod middleware {
        fn login() { ... }      // Key: rust:fn:login:__src_auth:T...  <-- COLLISION
    }
}
```

Both `login` functions in the same file produce identical keys minus the timestamp component. The birth timestamp provides a probabilistic tiebreaker (different hashes), but the key carries zero information about the module path `auth::handlers` vs `auth::middleware`.

This matters because:
- An LLM reading `rust:fn:login:__src_auth:T1706284800` cannot determine which `login` is being referenced
- Cross-reference queries ("who calls `auth::handlers::login`?") cannot distinguish targets
- Ascent Datalog rules joining on entity keys will silently merge distinct entities if timestamps happen to collide

### 1.3 Problem: Overload/Signature Collisions

Languages with function overloading produce multiple distinct functions with the same name:

```java
// Java
class Parser {
    void parse(String input) { ... }        // java:fn:parse:__src_Parser:T...
    void parse(String input, int depth) { } // java:fn:parse:__src_Parser:T...
    void parse(InputStream stream) { ... }  // java:fn:parse:__src_Parser:T...
}
```

```cpp
// C++
namespace utils {
    int convert(int x);                     // cpp:fn:convert:__src_utils:T...
    double convert(double x);               // cpp:fn:convert:__src_utils:T...
    std::string convert(const char* x);     // cpp:fn:convert:__src_utils:T...
}
```

ISGL1 has no parameter type information in the key. The birth timestamp hash is the only discriminator, and it is derived from `(file_path, entity_name)` -- meaning all three overloads produce the SAME hash. From `isgl1_v2.rs`:

```rust
pub fn compute_birth_timestamp(file_path: &str, entity_name: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    file_path.hash(&mut hasher);
    entity_name.hash(&mut hasher);
    // ...
}
```

The hash inputs are `("src/Parser.java", "parse")` for all three overloads. This is a deterministic collision.

### 1.4 Problem: No Module Path

ISGL1 uses `semantic_path` (a sanitized file path) but not the logical module path. These are different:

```
File path:    src/models/user.rs
Module path:  crate::models::user::UserProfile

File path:    lib/services/auth_service.py
Module path:  myapp.services.auth_service.AuthService
```

The file path tells you WHERE the code lives on disk. The module path tells you HOW the code is addressed in the language's type system. For cross-file resolution (imports, type references), the module path is what matters.

### 1.5 Problem: No File Context

The `semantic_path` in ISGL1 v2 strips the file extension and sanitizes:
- `src/auth.rs` becomes `__src_auth`
- `src/auth.py` becomes `__src_auth`

A Rust function and a Python function in files with the same stem produce ambiguous keys. The language prefix disambiguates in most cases, but the semantic path itself loses information.

### 1.6 Problem: Delimiter Collisions

The `:` delimiter collides with language syntax (documented in the existing ISGL1 v3 research):
- Rust: `std::io::Read` (module separator)
- Python: `x: int` (type annotation)
- Go: struct tags use `:`
- Windows: `C:\path` (drive letter)
- URLs: `http://host:port`

An LLM parsing `rust:fn:std::io::Read:__src_main:T170...` cannot reliably split on `:` to extract fields.

### 1.7 Problem: Generic Type Sanitization Burden

ISGL1 v2.1 introduced sanitization to handle generic types:

```
List<string>                    --> List__lt__string__gt__
Dictionary<string, object>      --> Dictionary__lt__string__c__object__gt__
```

This is fragile code (see `sanitize_entity_name_for_isgl1` in `isgl1_v2.rs` -- 30 lines of character-by-character replacement). It exists solely because the `:` delimiter cannot coexist with `<>` in CozoDB queries. A better delimiter eliminates this entire layer.

### 1.8 Problem: Birth Timestamp Is A Lie

The "birth timestamp" is not a real timestamp. It is a deterministic hash mapped to the range 2020-2030:

```rust
let base_timestamp = 1577836800; // 2020-01-01
let range = 315360000;           // ~10 years in seconds
let offset = (hash % range as u64) as i64;
base_timestamp + offset
```

It looks like a timestamp but is not one. This is confusing for debugging ("why was this entity born on 2023-07-15?") and provides no semantic value beyond being a hash. A hash should look like a hash.

---

## 2. How Other Tools Identify Entities

### 2.1 SCIP (Sourcegraph Code Intelligence Protocol)

**Source**: [SCIP GitHub Repository](https://github.com/sourcegraph/scip), [SCIP Protobuf Schema](https://github.com/sourcegraph/scip/blob/main/scip.proto), [Announcing SCIP Blog Post](https://sourcegraph.com/blog/announcing-scip)

SCIP was created by Sourcegraph to replace LSIF. It uses human-readable string IDs for symbols.

**Symbol String Grammar**:
```
<symbol>     := <scheme> ' ' <package> ' ' <descriptor>+
              | 'local ' <local-id>
<package>    := <manager> ' ' <package-name> ' ' <version>
<descriptor> := <namespace> | <type> | <term> | <method>
              | <type-parameter> | <parameter> | <meta> | <macro>
```

**Descriptor Suffixes**:

| Descriptor Kind | Suffix Character | Example |
|----------------|-----------------|---------|
| Namespace | `/` | `my_crate/` |
| Type | `#` | `Point#` |
| Term | `.` | `x.` |
| Method | `(` + optional disambiguator + `).` | `new().` |
| TypeParameter | `[` + name + `]` | `[T]` |
| Parameter | `(` + name + `)` | `(self)` |
| Meta | `:` | `derive:` |
| Macro | `!` | `println!` |

**Concrete Examples** (from rust-analyzer SCIP output):

```
rust-analyzer cargo ra-test 0.1.0 Point#
rust-analyzer cargo ra-test 0.1.0 Point#new().
rust-analyzer cargo foo . crate/
rust-analyzer cargo mylib 0.1.0 auth/handlers/login().
local 3
```

**Design Principles**:
- Human-readable: you can look at a SCIP symbol and understand what it refers to
- Hierarchical: descriptors chain to form fully qualified paths
- Cross-repo: package manager + name + version enable navigation across repositories
- Local symbols are document-scoped and never cross boundaries
- Method disambiguator handles overloads: `parse(0).` vs `parse(1).` (index-based)
- String-based IDs (not numeric) help with debugging and limiting blast radius of indexer bugs
- SCIP is a transmission format, not a storage format

**Relevance to Parseltongue**: SCIP's hierarchical descriptor chain is exactly what ISGL1 lacks. The `namespace/type#method().` pattern naturally encodes containment. The method disambiguator solves the overload collision problem. The package component enables cross-crate/cross-package references.

### 2.2 Kythe (Google)

**Source**: [Kythe Schema Reference](https://kythe.io/docs/schema/), [Kythe Storage Model](https://kythe.io/docs/kythe-storage.html), [Writing a New Indexer](https://kythe.io/docs/schema/writing-an-indexer.html), [Kythe URI Specification](https://kythe.io/docs/kythe-uri-spec.html)

Kythe emerged from Google's need to cross-reference their massive, multi-language internal codebase. It uses a 5-field "VName" (Vector-Name) as its naming primitive.

**VName Fields**:

| Field | Purpose | Example |
|-------|---------|---------|
| `corpus` | Repository or project | `github.com/my-org/my-repo` |
| `root` | Subset within corpus (often empty) | `src/` or empty |
| `path` | File path relative to root | `auth/handlers.rs` |
| `language` | Programming language | `rust`, `python`, `c++` |
| `signature` | Unique within (corpus, root, path, language) | Opaque, indexer-defined |

**URI Encoding (Tickets)**:
```
kythe://github.com/my-org/my-repo?lang=rust?path=src/auth.rs#fn_login
kythe://github.com/my-org/my-repo?lang=java?path=src/Parser.java#parse_String_int
```

**Design Principles**:
- Minimal basis: as few fields as possible to maintain uniqueness
- The signature field is intentionally opaque -- indexers are free to encode it however they want, as long as it is deterministic and has vanishingly small collision probability
- VNames for objects accessible from multiple compilation units MUST be generated consistently across all modules that reference them
- Extensible: new dimensions can be added without breaking existing VNames
- Anchors (source locations) share path/root/corpus with their parent file

**Relevance to Parseltongue**: Kythe's 5-field VName is the most mature design for cross-language entity identification at scale (Google scale). The separation of `corpus` from `path` from `signature` is clean. However, the opaque signature field trades human-readability for flexibility -- not ideal for LLM consumption where readability is paramount.

### 2.3 CodeQL (GitHub/Semmle)

**Source**: [CodeQL Documentation](https://codeql.github.com/docs/contents/), [Working with Source Locations](https://codeql.github.com/docs/codeql-language-guides/working-with-source-locations/), [Name Resolution](https://codeql.github.com/docs/ql-language-reference/name-resolution/)

CodeQL takes a fundamentally different approach: entities are database-local.

**Identification Scheme**:
- Language extractors produce TRAP files (Tuples Representing Abstract Properties)
- Each code element (class, method, expression) gets a database-local integer ID
- IDs are NOT stable across database rebuilds
- Location info: file + start line + start column + end line + end column (1-based, inclusive)
- Six namespaces for resolution: module, type, predicate, module signature, type signature, predicate signature
- An advanced option can construct entity IDs that encode the location in the TRAP file they came from (for debugging)

**Design Principles**:
- Entity identity is internal to the database -- no global identifiers
- Queries resolve symbols through QL's type system, not through key lookup
- Location-based identification: file + (startLine, startCol, endLine, endCol)
- Each language has its own database schema (`.dbscheme` file) with language-specific entity types

**Relevance to Parseltongue**: CodeQL's approach is NOT suitable for our use case. We need stable, human-readable keys that survive across HTTP responses, JSON serialization, MCP tool calls, and LLM reasoning. Database-local integer IDs fail all of those requirements. However, CodeQL's location model (file + line range) is relevant -- line ranges are useful supplementary information even if not part of the key itself.

### 2.4 LSP (Language Server Protocol)

**Source**: [LSP Specification 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)

LSP identifies entities by document URI + position (line + character).

**Identification Model**:
```typescript
interface TextDocumentPositionParams {
    textDocument: TextDocumentIdentifier;  // { uri: DocumentUri }
    position: Position;                     // { line: number, character: number }
}
```

**Design Principles**:
- "Describing data types at the level of the editor rather than at the level of the programming language model" -- this is why LSP succeeded
- URI is the universal document identifier
- Position is ephemeral -- it changes on every edit
- No persistent entity identity -- LSP is a request/response protocol, not a data model
- Language-neutral: applies to all programming languages equally

**Relevance to Parseltongue**: LSP proves that URI + position is sufficient for interactive navigation but NOT sufficient for persistent graph storage. We need keys that persist across sessions and are stable across minor edits. LSP's insight about language-neutrality via editor-level data types IS relevant: our keys should be understandable without knowing language-specific semantics.

### 2.5 rust-analyzer Internal Representation

**Source**: [rust-analyzer Guide](https://rust-analyzer.github.io/book/contributing/guide.html), [rust-analyzer Architecture](https://github.com/rust-lang/rust-analyzer/blob/c2064e8bcfdd937ef05a5e1ff59b532b4a37181e/docs/dev/architecture.md)

rust-analyzer uses opaque integer IDs internally, with a layered resolution model.

**Identification Model**:
- `FileId`: opaque integer identifying a source file (no path information)
- `HirFileId`: extends FileId to include macro-expanded pseudo-files
- `MacroCallId`: interned ID for a macro invocation = HirFileId + offset
- `DefId` equivalent: various interned IDs in `hir_def` crate identify definitions
- Source-to-def resolution: syntax (file + offset) --> HIR definitions via the `Semantics` / `source_to_def` infrastructure

**Design Principles**:
- Salsa-based incremental computation: IDs are inputs to memoized queries
- The `ide` crate API uses editor terminology (offsets, labels) not compiler terminology (DefId, HirId)
- IDs are NOT human-readable (opaque integers)
- IDs are NOT stable across sessions (interned per-session)

**Relevance to Parseltongue**: rust-analyzer's approach is optimized for incremental in-memory computation, not for serialized graph storage or LLM consumption. The interned IDs are fast for lookups but useless for persistence. However, in v2.0.0 `rust-llm-03` (rust-analyzer bridge), we need to MAP from rust-analyzer's internal IDs to our key format. This mapping must be deterministic and consistent.

### 2.6 SemanticDB (Scalameta)

**Source**: [SemanticDB Specification](https://scalameta.org/docs/semanticdb/specification.html), [Design of scip-java](https://sourcegraph.github.io/scip-java/docs/design.html)

SemanticDB is the predecessor to SCIP, pioneered in the Scala ecosystem.

**Symbol Format**:
- Global symbols: fully qualified path with designators (e.g., `scala/Int#`, `_root_.scala.Int#`)
- Local symbols: `local0`, `local1`, etc. (document-scoped)
- Symbols are NOT guaranteed globally unique -- tool authors must accompany them with out-of-band metadata

**Relevance to Parseltongue**: SemanticDB's non-guaranteed uniqueness is a cautionary tale. Our keys MUST be globally unique within the analyzed codebase. SCIP learned from SemanticDB and fixed this with the scheme + package + descriptor structure.

### 2.7 Summary Comparison Matrix

| System | ID Type | Human-Readable | Stable | Hierarchical | Cross-Language | Cross-Repo |
|--------|---------|---------------|--------|-------------|----------------|------------|
| ISGL1 v2 | Structured string | Partial | Yes (timestamp) | No | Yes (lang prefix) | No |
| SCIP | Structured string | Yes | Yes | Yes (descriptors) | Yes | Yes (package) |
| Kythe | 5-field VName | Partial (opaque sig) | Varies | No (flat VName) | Yes | Yes (corpus) |
| CodeQL | Database-local int | No | No | N/A | Per-language DB | No |
| LSP | URI + position | Yes | No | No | Language-neutral | N/A |
| rust-analyzer | Interned int | No | No (per-session) | Internal only | Rust-only | No |
| SemanticDB | String path | Yes | Varies | Yes | Java+Scala | Limited |

---

## 3. Requirements for Our Key Format

Derived from the v2.0.0 architecture (Prep-Doc-V200.md, PRD-v200.md) and the problems above.

### 3.1 MUST Requirements

| # | Requirement | Rationale |
|---|------------|-----------|
| M1 | **Unique across 12+ languages** | A Rust `fn login` and Python `def login` in different files MUST produce different keys. Same name + same file (different scope) MUST produce different keys. |
| M2 | **Human-readable** | LLMs parse and reason about keys. An LLM reading a key should understand what entity it refers to without consulting a lookup table. This is our core differentiator vs Kythe's opaque signatures. |
| M3 | **Stable across minor edits** | Adding a comment above a function should NOT change its key. Adding a blank line between functions should NOT change any keys. Changing a function's body should NOT change its key. |
| M4 | **Hierarchical** | Encode the containment path: file > module > class > method. This is needed for scope-aware queries ("list all methods in class X") and for Ascent rules that reason about containment. |
| M5 | **Hashable** | Keys are used as HashMap keys in rust-llm-05 (TypedAnalysisStore). Must implement `Hash + Eq` efficiently. |
| M6 | **Serializable** | Keys are returned in JSON (HTTP server), MessagePack (storage), and MCP tool responses. Must be UTF-8 string-safe with no binary content. |
| M7 | **Delimiter-safe** | The delimiter must NOT collide with any syntax in any of the 12 supported languages, file paths (Unix + Windows), or URLs. |
| M8 | **No sanitization required** | The key format must handle generic types (`List<T>`), module paths (`std::io::Read`), operator overloads, and language-specific special characters WITHOUT requiring an encoding/escaping layer. |
| M9 | **Deterministic** | Same source code, same file path, same analysis pass = same key. Every time. Across platforms. |
| M10 | **Overload-distinguishable** | Two functions with the same name but different parameter signatures MUST produce different keys. |

### 3.2 SHOULD Requirements

| # | Requirement | Rationale |
|---|------------|-----------|
| S1 | **Cross-crate/package references** | When entity A in crate `foo` calls entity B in crate `bar`, B's key should encode that it belongs to `bar`. This is needed for rust-llm-02 (cross-language edges). |
| S2 | **Sortable** | Keys should sort lexicographically in a useful order (e.g., by language, then by file, then by position). This makes debugging and display easier. |
| S3 | **Parseable** | Given a key string, it should be possible to extract individual components (language, type, name, path) without ambiguity. |
| S4 | **Compact** | Keys appear in every edge (twice -- from_key and to_key). A graph with 100K edges stores 200K key references. Key length matters for memory and serialization size. |
| S5 | **Supports external entities** | External dependencies (e.g., `std::io::Read`, `numpy.array`) should have representable keys even though they don't have a local file path. |

### 3.3 MUST NOT Requirements

| # | Requirement | Rationale |
|---|------------|-----------|
| N1 | **Must NOT use line numbers as identity** | Line numbers shift on every edit. Keys must survive refactors. |
| N2 | **Must NOT use opaque integer IDs** | Integers require a lookup table. LLMs cannot reason about `entity_42`. |
| N3 | **Must NOT require a database for resolution** | Keys must be self-contained. You should be able to read a key and know what entity it refers to without querying anything. |
| N4 | **Must NOT depend on CozoDB** | v2.0.0 drops CozoDB entirely. No CozoDB query-safe constraints. |

---

## 4. Candidate Designs with Tradeoffs

### 4.1 Candidate A: SCIP-Inspired Descriptor Chain

Adapt SCIP's descriptor chain format for our use case.

**Format**:
```
{scheme} {manager} {package} {version} {descriptor_chain}
```

**Parseltongue Adaptation**:
```
pt {lang} {crate_or_module} . {descriptor_chain}
```

**Descriptor Suffixes** (adapted from SCIP):

| Kind | Suffix | Example |
|------|--------|---------|
| Namespace/Module | `/` | `auth/` |
| Type (struct/class/enum) | `#` | `UserService#` |
| Function/Method | `()` | `login()` |
| Trait/Interface | `%` | `Serialize%` |
| Variable/Constant | `.` | `MAX_RETRIES.` |
| Impl Block | `+` | `UserService+Serialize+` |
| Macro | `!` | `println!` |
| Type Parameter | `[T]` | `[T]` |

**Concrete Examples**:
```
pt rust my_crate . auth/handlers/login()
pt rust my_crate . auth/handlers/UserService#new()
pt python myapp . services/auth_service/AuthService#login()
pt java com.example . parser/Parser#parse(String,int)
pt java com.example . parser/Parser#parse(InputStream)
pt cpp . . utils/convert(int)
pt cpp . . utils/convert(double)
pt rust std . io/Read%read()
pt typescript . . components/App/handleClick()
```

**Pros**:
- Hierarchical: the descriptor chain encodes the full containment path
- Human-readable: you can read the key and understand the entity
- Overloads resolved: parameter types in method descriptors distinguish overloads
- Cross-package: crate/module name in the key enables cross-crate references
- Proven design: SCIP is used at scale by Sourcegraph, GitHub, and GitLab
- No sanitization needed

**Cons**:
- Space-delimited top-level fields are unusual and can confuse string handling
- Requires building a descriptor chain during extraction -- more complex than flat key generation
- Parameter type extraction is not available from tree-sitter for all languages
- The `.` placeholder for empty fields is a magic value
- LLMs may struggle with the compact suffix notation (`#`, `()`, `/`) vs explicit labels

**Complexity**: HIGH -- requires resolving module paths and parameter types during extraction

### 4.2 Candidate B: Triple-Pipe Delimited with Module Path

Extend the ISGL1 v3 research (`|||` delimiter) with a module path component.

**Format**:
```
{language}|||{entity_type}|||{module_path}|||{entity_name}|||{file_path}|||{discriminator}
```

**Concrete Examples**:
```
rust|||fn|||auth::handlers|||login|||src/auth.rs|||d0
rust|||fn|||auth::middleware|||login|||src/auth.rs|||d0
python|||class|||services.auth_service|||AuthService|||lib/services/auth_service.py|||d0
java|||method|||com.example.parser.Parser|||parse|||src/Parser.java|||String_int
java|||method|||com.example.parser.Parser|||parse|||src/Parser.java|||InputStream
cpp|||fn|||utils|||convert|||src/utils.cpp|||int
cpp|||fn|||utils|||convert|||src/utils.cpp|||double
rust|||trait|||std::io|||Read|||EXTERNAL|||d0
typescript|||fn|||components.App|||handleClick|||src/components/App.tsx|||d0
```

**Discriminator Field**: Solves overloads.
- `d0` = default (no overload, or first occurrence)
- `String_int` = parameter type signature for overloaded methods
- When tree-sitter cannot determine parameter types, fall back to positional index: `d0`, `d1`, `d2`

**Pros**:
- Very human-readable: explicit labels for every field
- `|||` delimiter: zero collisions across all 15 languages (proven in ISGL1 v3 research)
- No sanitization needed: `<>`, `::`, `:`, `/` all safe within `|||`-delimited fields
- Module path is explicit: `auth::handlers` tells you the containment hierarchy
- File path is raw: `src/auth.rs` readable by LLMs for `Read` tool calls
- Simple to generate: flat string concatenation with `|||` separator
- Parseable: `split("|||")` gives exactly 6 fields every time
- The discriminator field explicitly handles the overload problem

**Cons**:
- Verbose: keys are longer than SCIP-style compact notation
- Module path extraction is language-dependent and non-trivial
- `|||` costs ~5 extra tokens per key vs `:` (negligible in practice)
- Six fields is a lot -- more opportunities for inconsistency
- Module path may not always be determinable from tree-sitter alone

**Complexity**: MEDIUM -- module path extraction adds work but descriptor chain assembly is not needed

### 4.3 Candidate C: Kythe-Inspired VName with Readable Signature

Adapt Kythe's 5-field VName but replace the opaque signature with a human-readable one.

**Rust Struct Representation**:
```rust
pub struct EntityKey {
    pub language: Language,
    pub package: String,
    pub file_path: String,
    pub qualified_name: String,
    pub discriminator: Option<String>
}
```

**String Serialization** (for JSON/HTTP/MCP):
```
{language}://{package}/{file_path}#{qualified_name}[{discriminator}]
```

**Concrete Examples**:
```
rust://my_crate/src/auth.rs#auth::handlers::login
rust://my_crate/src/auth.rs#auth::middleware::login
python://myapp/lib/services/auth_service.py#AuthService.login
java://com.example/src/Parser.java#Parser.parse[String,int]
java://com.example/src/Parser.java#Parser.parse[InputStream]
cpp:///src/utils.cpp#utils::convert[int]
cpp:///src/utils.cpp#utils::convert[double]
rust://std/EXTERNAL#io::Read
typescript:///src/components/App.tsx#App.handleClick
```

**Pros**:
- URI-like syntax is familiar to developers and LLMs
- Language, package, file, and qualified name are all explicit
- The `#` fragment separator cleanly divides "where" (file) from "what" (entity)
- Discriminator in `[]` handles overloads
- Compact: shorter than `|||` format for most entities
- Maps naturally to Kythe VNames if interop is ever needed

**Cons**:
- `://` and `#` are URL syntax -- could confuse URL parsers or LLMs
- `::` in qualified names (Rust) embeds language-specific syntax in the key
- `.` in qualified names (Python/Java) means the scope separator varies by language
- `[]` for discriminator could be confused with array syntax
- Empty package (`///`) looks like a malformed URL

**Complexity**: MEDIUM-HIGH -- URI parsing rules introduce edge cases

### 4.4 Candidate D: Structured Typed Key (Not A String)

Instead of encoding everything into a string, use a typed Rust struct as the key and serialize to string only at API boundaries.

**Rust Representation**:
```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntityKey {
    pub language: Language,
    pub kind: EntityKind,
    pub scope: Vec<ScopeSegment>,
    pub name: String,
    pub file_path: String,
    pub discriminator: Discriminator,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScopeSegment {
    Crate(String),
    Module(String),
    Class(String),
    Impl { type_name: String, trait_name: Option<String> },
    Namespace(String),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Discriminator {
    None,
    ParamTypes(Vec<String>),
    Index(u32),
    ContentHash(String),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum EntityKind {
    Function, Method, Class, Struct, Enum, Trait, Interface,
    Module, Namespace, Impl, Macro, Variable, Constant,
    Table, View, ImportBlock, TestFunction,
}
```

**String Serialization** (Display impl, used at HTTP/MCP boundaries):
```
rust|||fn|||my_crate::auth::handlers|||login|||src/auth.rs
java|||method|||com.example::Parser|||parse|||src/Parser.java|||[String,int]
```

**Internal Usage** (in Ascent rules, HashMaps, etc.): the struct itself, not the string.

**Pros**:
- Type safety: the Rust compiler enforces that keys have all required fields
- No parsing ambiguity: the struct IS the key, string serialization is just for display
- Scope is explicit and typed: `Vec<ScopeSegment>` is unambiguous
- Discriminator is typed: `ParamTypes(vec!["String", "int"])` vs `Index(0)` vs `ContentHash("ab12cd34")`
- Ergonomic for Ascent: Datalog rules can pattern-match on `EntityKind::Function` directly
- Efficient HashMap lookups: derived `Hash` on the struct, no string parsing
- Extensible: adding new EntityKinds or ScopeSegment variants is a type change, not a format change
- Catches bugs at compile time

**Cons**:
- More complex to implement: building `Vec<ScopeSegment>` from tree-sitter output requires understanding each language's scope rules
- Serialized form is larger in JSON vs a flat string
- Two representations: the struct (internal) and the string (external) must stay in sync
- Equality depends on ALL fields: if scope resolution is incomplete, keys for the same entity might not match
- Harder to construct in tests

**Complexity**: HIGH -- most engineering effort, but most correct by construction

---

## 5. What Breaks If We Get It Wrong

### 5.1 rust-llm-01 (Fact Extractor) -- GENERATES Keys

Every fact has a key. If the key format changes, every extraction function changes. The extractor is the ONLY component that creates keys from raw source code.

**What breaks**: If keys are ambiguous (collisions), the extractor silently produces wrong data. Two distinct entities get the same key. One overwrites the other in the store. The graph loses an entity with no error.

### 5.2 rust-llm-02 (Cross-Language Edges) -- REFERENCES Keys Across Boundaries

Cross-language edge detection matches entities across language boundaries (e.g., Rust FFI function matched to C function). Both sides must produce compatible keys.

**What breaks**: If the key format doesn't handle cross-language references cleanly, cross-language edges cannot be created. This is v2.0.0's differentiating feature.

### 5.3 rust-llm-03 (rust-analyzer Bridge) -- MAPS DefIds TO Keys

rust-analyzer's internal DefIds must be deterministically mapped to our key format. The same Rust entity identified by tree-sitter (in rust-llm-01) and by rust-analyzer (in rust-llm-03) must produce the SAME key.

**What breaks**: If tree-sitter and rust-analyzer produce different keys for the same entity, Ascent rules that join facts from both sources will fail silently -- the join key won't match. The "mix of tree-sitter x rust-analyzer" architecture (Prep-Doc-V200.md section 5) collapses.

### 5.4 rust-llm-04 (Reasoning Engine) -- JOINS On Keys

Ascent Datalog rules join relations on entity keys:
```rust
ascent! {
    relation entity(String, EntityInfo);
    relation edge(String, String, EdgeKind);
    relation unsafe_chain(String);

    unsafe_chain(F) :- entity(F, info), is_unsafe(info);
    unsafe_chain(F) :- edge(F, G, Calls), unsafe_chain(G);
}
```

**What breaks**: String keys with ambiguous delimiters cause silent join failures. Key A from tree-sitter doesn't match Key A from rust-analyzer because one used `::` and the other used `.` in the qualified name. The Ascent rule produces an empty result set and nobody notices until an LLM gets a wrong answer.

### 5.5 rust-llm-05 (Knowledge Store) -- INDEXES On Keys

The TypedAnalysisStore uses HashMap<EntityKey, EntityData>. The key is the primary index.

**What breaks**: If key hashing is expensive (long strings with repeated prefixes), every HashMap lookup is slow. If key equality is broken (two keys for the same entity that don't compare equal), the store silently duplicates data and wastes memory.

### 5.6 rust-llm-06 and rust-llm-07 (HTTP + MCP Servers) -- RETURN Keys

Keys appear in every API response. An LLM receives a key and may use it in a subsequent query:
1. LLM calls `/code-entities-search-fuzzy?q=login`
2. Response includes key `rust|||fn|||auth::handlers|||login|||src/auth.rs|||d0`
3. LLM calls `/blast-radius-impact-analysis?entity=rust|||fn|||auth::handlers|||login|||src/auth.rs|||d0`

**What breaks**: Keys with characters that require URL encoding (`#`, `?`, `&`, `=`, spaces) break when passed as query parameters. Keys longer than ~200 characters get truncated or mangled by HTTP clients. Keys that tokenize poorly consume the LLM's context budget.

---

## 6. Recommendation

### 6.1 Primary Recommendation: Candidate D (Structured Typed Key) with Candidate B's Serialization

Use a typed Rust struct as the internal key representation (Candidate D), with the `|||`-delimited string format (Candidate B) as the serialized/display form.

**Rationale**:

1. **Type safety at the core**: The key is a struct with `Hash + Eq` derived. Collisions are impossible if the struct fields are correct. Ascent rules pattern-match on typed fields, not string fragments.

2. **Human-readable at the boundary**: The `Display` trait serializes to `rust|||fn|||auth::handlers|||login|||src/auth.rs|||d0` for HTTP responses and MCP tool calls. LLMs see readable keys.

3. **Best of both worlds**: Internal code gets compile-time safety. External consumers get human-readable strings. Neither compromise is forced on the other.

4. **Discriminator handles overloads**: The typed `Discriminator` enum can be `ParamTypes` when tree-sitter can extract them, `Index` as a fallback, or `ContentHash` for truly ambiguous cases.

5. **Module path is a best-effort field**: The `scope: Vec<ScopeSegment>` can be populated when the extractor can determine module paths, and left empty when it cannot. An empty scope is still a valid key -- it just means "module path unknown." This degrades gracefully.

6. **`|||` delimiter is proven**: The ISGL1 v3 research already validated that `|||` has zero collisions across all 15 languages, file paths, and URLs.

### 6.2 Proposed Struct

```rust
/// Entity key for v2.0.0 (rust-llm-core)
///
/// Internal representation: typed struct with derived Hash + Eq.
/// External representation: |||--delimited string via Display trait.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntityKey {
    /// Programming language
    pub language: Language,
    /// Entity kind (function, class, struct, etc.)
    pub kind: EntityKind,
    /// Containment scope (crate > module > class), best-effort
    pub scope: Vec<String>,
    /// Entity name
    pub name: String,
    /// Relative file path (empty for external entities)
    pub file_path: String,
    /// Overload discriminator
    pub discriminator: String,
}
```

**Display Implementation**:
```
rust|||fn|||my_crate.auth.handlers|||login|||src/auth.rs|||d0
```

**Scope serialization**: scope segments joined with `.` (universal, no language-specific `::` or `/`).

### 6.3 Migration Note

This is a clean break (per PRD-v200.md Requirement #1). No migration from ISGL1 v2 to this format. No backward compatibility. The old `isgl1_v2.rs` stays in `parseltongue-core` (frozen v1.x). The new `EntityKey` lives in `rust-llm-core`.

### 6.4 Open Questions for Implementation

1. **Scope extraction depth**: How much effort do we spend extracting module paths from tree-sitter? Do we do best-effort (sometimes empty) or block on getting it right for all 12 languages before shipping?

2. **External entity keys**: For `std::io::Read` or `numpy.array`, what goes in `file_path`? Proposal: `EXTERNAL:{package_name}` (e.g., `EXTERNAL:std`, `EXTERNAL:numpy`).

3. **Discriminator population**: When tree-sitter cannot extract parameter types, do we use positional index (`d0`, `d1`) or content hash of the function body?

4. **Key normalization**: Should `rust|||fn|||auth.handlers|||login|||src/auth.rs|||d0` and `rust|||fn|||auth.handlers|||login|||./src/auth.rs|||d0` be considered the same key? (Path normalization.)

5. **String interning**: Should keys be interned (single allocation, pointer comparison) for performance in the knowledge store?

---

## References

- [SCIP GitHub Repository](https://github.com/sourcegraph/scip)
- [SCIP Protobuf Schema](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- [SCIP Design Document](https://github.com/sourcegraph/scip/blob/main/DESIGN.md)
- [Announcing SCIP (Sourcegraph Blog)](https://sourcegraph.com/blog/announcing-scip)
- [rust-analyzer SCIP Output](https://rust-lang.github.io/rust-analyzer/src/rust_analyzer/cli/scip.rs.html)
- [Improve SCIP symbols PR](https://github.com/rust-lang/rust-analyzer/pull/18758)
- [Kythe Schema Reference](https://kythe.io/docs/schema/)
- [Kythe Storage Model](https://kythe.io/docs/kythe-storage.html)
- [Kythe URI Specification](https://kythe.io/docs/kythe-uri-spec.html)
- [Writing a New Kythe Indexer](https://kythe.io/docs/schema/writing-an-indexer.html)
- [Kythe Schema Overview](https://kythe.io/docs/schema-overview.html)
- [CodeQL Documentation](https://codeql.github.com/docs/contents/)
- [CodeQL Name Resolution](https://codeql.github.com/docs/ql-language-reference/name-resolution/)
- [CodeQL Working with Source Locations](https://codeql.github.com/docs/codeql-language-guides/working-with-source-locations/)
- [LSP Specification 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
- [rust-analyzer Architecture](https://github.com/rust-lang/rust-analyzer/blob/c2064e8bcfdd937ef05a5e1ff59b532b4a37181e/docs/dev/architecture.md)
- [rust-analyzer Guide](https://rust-analyzer.github.io/book/contributing/guide.html)
- [SemanticDB Specification](https://scalameta.org/docs/semanticdb/specification.html)
- [Design of scip-java](https://sourcegraph.github.io/scip-java/docs/design.html)
- Internal: `crates/parseltongue-core/src/isgl1_v2.rs` (ISGL1 v2 implementation)
- Internal: `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` (Key generation)
- Internal: `crates/parseltongue-core/src/entities.rs` (Entity types and CodeEntity struct)
- Internal: `docs/RESEARCH-isgl1v3-exhaustive-graph-identity.md` (ISGL1 v3 research)
- Internal: `docs/Prep-Doc-V200.md` (v2.0.0 prep document)
- Internal: `docs/PRD-v200.md` (v2.0.0 product requirements)
