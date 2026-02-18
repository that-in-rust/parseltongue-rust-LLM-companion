# PRD Contracts: Parseltongue V200 FINAL

**Generated**: 2026-02-17
**Version**: 02-FINAL
**Status**: DEFINITIVE SYNTHESIS
**Sources**: v200-docs directory (18 documents) + ES-V200-attempt-01.md + ES-V200-Hashing-Risks-v01.md + ES-V200-Decision-log-01.md

---

## Document Status Legend

| Status | Meaning |
|--------|---------|
| **DECIDED** | Final decision locked, no further discussion |
| **PROPOSED** | Recommended approach, pending CF-P1 probe validation |
| **DEFERRED** | Intentionally moved to v210+ |
| **OPEN** | Requires resolution before Phase 1 |

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Contracts](#2-architecture-contracts)
3. [Non-Negotiable Gates (G1-G4)](#3-non-negotiable-gates-g1-g4)
4. [Promoted Requirements (R1-R8)](#4-promoted-requirements-r1-r8)
5. [Three-Layer Architecture](#5-three-layer-architecture)
6. [EntityKey Format Decision](#6-entitykey-format-decision)
7. [Probe Sets (CF/SR/RS/TE/CB/GR/GW/TH)](#7-probe-sets)
8. [The Killer Feature: get_context MCP Tool](#8-the-killer-feature-get_context-mcp-tool)
9. [Risk Matrix and Evidence](#9-risk-matrix-and-evidence)
10. [Implementation Phases (LNO)](#10-implementation-phases-lno)
11. [Open Questions](#11-open-questions)
12. [Datalog Rules (18 Total)](#12-datalog-rules-18-total)
13. [API Contracts](#13-api-contracts)
14. [Source Document Index](#14-source-document-index)

---

## 1. Executive Summary

### 1.1 What V200 Is [DECIDED]

**Parseltongue V200** is a **clean-room rewrite** that transforms code analysis from syntactic pattern-matching into a **three-layer architecture** combining:

1. **Compiler Truth** (rust-analyzer for Rust, tree-sitter for 12 languages)
2. **LLM Judgment** (business context, naming, design intent)
3. **CPU Graph Algorithms** (enriched with semantic edge types)

**Core Value Proposition**: 99% token reduction (2-5K vs 500K raw dumps), 31x faster than grep, with **compiler-verified** insights that eliminate LLM hallucination on technical questions.

**Source**: `ES-V200-attempt-01.md` Part I

### 1.2 Positioning Statement [DECIDED]

> **rust-llm: The code intelligence layer that AI coding tools are missing.**

Every AI coding tool (Cursor, Aider, Continue, Cody, Claude Code, Copilot) needs to understand code architecture to give better answers. None of them have it. rust-llm provides it as a library, MCP server, or HTTP API.

**Source**: `Prep-V200-Max-Adoption-Architecture-Strategy.md`

### 1.3 Architecture Scope [DECIDED]

| Decision | Status | Source |
|----------|--------|--------|
| 8 crates in V200 scope | DECIDED | `ES-V200-Decision-log-01.md` |
| Clean-room rewrite, no pt* dependencies | DECIDED | `ES-V200-attempt-01.md` |
| context-packer deferred to v210 | DECIDED | `ES-V200-Decision-log-01.md` |
| Tauri app as external companion (not core crate) | DECIDED | `ES-V200-Decision-log-01.md` |

---

## 2. Architecture Contracts

### 2.1 Crate Architecture (8 Crates) [DECIDED]

```
rust-llm-interface-gateway   -- CLI/HTTP/MCP transport
rust-llm-core-foundation     -- Entity keys + contracts
rust-llm-tree-extractor      -- 12-language tree-sitter
rust-llm-rust-semantics      -- rust-analyzer integration
rust-llm-cross-boundaries    -- Cross-language edge linking
rust-llm-graph-reasoning     -- Datalog reasoning (Ascent)
rust-llm-store-runtime       -- Graph persistence
rust-llm-test-harness        -- Contract testing + CI gates
```

**Source**: `ES-V200-Hashing-Risks-v01.md`

### 2.2 Dependency Flow [DECIDED]

```
rust-llm-interface-gateway (ingest/query)
    ├── rust-llm-tree-extractor (entities)
    ├── rust-llm-rust-semantics (Rust enrichment)
    ├── rust-llm-cross-boundaries (cross-lang edges)
    ├── rust-llm-graph-reasoning (derived facts)
    └── rust-llm-store-runtime (persistence)

rust-llm-core-foundation (shared contracts) ─── all crates depend on this
rust-llm-test-harness (contract gates) ─── validates all crates
```

**Source**: `ES-V200-Hashing-Risks-v01.md` Mermaid diagram

### 2.3 Public Interface Snapshot [DECIDED]

| Crate | Main Public Interface | Input | Output |
|-------|----------------------|-------|--------|
| interface-gateway | normalize/capability/cancel + ingest/query | CLI/HTTP/MCP request DTOs | Canonical reports + mapped responses |
| core-foundation | build/parse/verify keys + contract checks | Identity/fact batches | Stable keys + validation reports |
| tree-extractor | parse/query/normalize extraction contracts | File set + language parsers | Syntax facts + dependency edges |
| rust-semantics | semantics + proc/build + degrade metadata | Cargo workspace + RA config | Resolved facts + degrade annotations |
| cross-boundaries | extract/match/score boundary links | Syntax+semantic fact batches | Boundary edges + confidence scores |
| graph-reasoning | derive/score/explain reasoning outputs | Facts + edges + constraints | Derived findings and priorities |
| store-runtime | commit/query/delta/snapshot/consistency | Fact+edge batches / queries | Persisted graph + bounded result set |
| test-harness | fixture/ci/flake/perf contract gates | Suite/probe definitions | Pass/fail + risk probe artifacts |

**Source**: `ES-V200-Hashing-Risks-v01.md`

---

## 3. Non-Negotiable Gates (G1-G4) [DECIDED]

These gates **BLOCK shipping** until passed with evidence.

| ID | Gate | Exact Meaning | Crate Owner | Probe Link |
|----|------|---------------|-------------|------------|
| **G1** | Slim types gate | Entity/storage schema stays canonical, minimal, deterministic | `rust-llm-core-foundation` | CF-P1-F |
| **G2** | Single getter contract gate | All read paths go through one storage getter contract | `rust-llm-store-runtime` | SR-P2-F |
| **G3** | Filesystem source-read contract gate | Detail view returns current disk lines with explicit error contract | `rust-llm-interface-gateway` | GW-P7-F |
| **G4** | Path normalization coverage gate | Coverage treats `./path`, `path`, and absolute path as one file | `rust-llm-test-harness` | TH-P8-F |

**Source**: `ES-V200-Hashing-Risks-v01.md` Section "V200 Non-Negotiable Hardening Gates"

### 3.1 WHEN/THEN/SHALL Contracts for Gates

#### G1: Slim Types Gate [DECIDED]

```
WHEN core-foundation builds entity key from EntityIdentityInput
THEN SHALL produce deterministic key string with zero format ambiguity
AND SHALL parse key string back to identical EntityIdentityView
AND SHALL detect overload collisions (same file + same name but different signature)
AND SHALL not require sanitization for valid language symbols
AND SHALL remain stable under whitespace/comment-only edits
```

**Pass Criterion**: Zero overload collisions across 12-language fixture corpus, deterministic roundtrip.

**Source**: `ES-V200-attempt-01.md` Part III

#### G2: Single Getter Contract Gate [DECIDED]

```
WHEN any crate queries graph data from store-runtime
THEN SHALL resolve through exactly one canonical getter contract
AND SHALL return Result<T, StoreError> with explicit error types
AND SHALL never bypass getter with direct database access
AND SHALL maintain result/error parity across all read paths
```

**Pass Criterion**: 100% of read paths go through single getter contract with result/error parity.

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 02

#### G3: Filesystem Source-Read Contract Gate [DECIDED]

```
WHEN interface-gateway serves `/code-entity-detail-view?key=X`
THEN SHALL read current file contents from disk (not cache)
AND SHALL return lines matching entity's line_range
AND SHALL return explicit error for missing/moved/permission-denied files
AND SHALL never return stale cached source
```

**Pass Criterion**: Detail-view fixtures for valid/missing/moved files + line-range outcomes.

**Source**: `ES-V200-attempt-01.md` Part III

#### G4: Path Normalization Coverage Gate [DECIDED]

```
WHEN test-harness measures coverage for a file
THEN SHALL treat `./src/main.rs`, `src/main.rs`, and `/abs/path/src/main.rs` as identical
AND SHALL aggregate entity counts across all path variants
AND SHALL report zero path-variant duplicates in coverage reports
```

**Pass Criterion**: `./x` vs `x` vs absolute fixture parity in coverage outputs.

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 08

---

## 4. Promoted Requirements (R1-R8) [DECIDED]

These requirements are **pre-approved** for V200 MVP.

| ID | Requirement | Crate Owner | Probe Link | Intent |
|----|-------------|-------------|------------|--------|
| **R1** | Route prefix nesting | `rust-llm-interface-gateway` | GW-P7-G | Stable namespaced routing by active mode |
| **R2** | Auto port + port file | `rust-llm-interface-gateway` | GW-P7-H | Deterministic startup/discovery lifecycle |
| **R3** | Shutdown CLI command | `rust-llm-interface-gateway` | GW-P7-I | Graceful stop contract from CLI to server |
| **R4** | XML-tagged responses | `rust-llm-interface-gateway` | GW-P7-J | Semantic response grouping for LLM consumption |
| **R5** | Project slug in URL | `rust-llm-interface-gateway` | GW-P7-K | Self-describing multi-project endpoint identity |
| **R6** | Slug in port file | `rust-llm-interface-gateway` | GW-P7-L | Slug-aware server discovery path |
| **R7** | Token count at ingest | `rust-llm-store-runtime` | SR-P2-G | Persist real `token_count` for deterministic use |
| **R8** | Data-flow tree-sitter queries | `rust-llm-tree-extractor` | TE-P4-F | Extract assign/param/return flow edges |

**Source**: `ES-V200-Decision-log-01.md`, `ES-V200-Hashing-Risks-v01.md`

### 4.1 WHEN/THEN/SHALL Contracts for Requirements

#### R1: Route Prefix Nesting [DECIDED]

```
WHEN gateway receives HTTP request for project "myapp"
THEN SHALL route to `/myapp/{endpoint}` namespace
AND SHALL reject requests to wrong prefix with explicit error
AND SHALL support multiple projects simultaneously
AND SHALL maintain endpoint compatibility within each namespace
```

**Source**: `ES-V200-attempt-01.md` Part IV

#### R2: Auto Port + Port File [DECIDED]

```
WHEN gateway starts HTTP server for project slug "myapp"
THEN SHALL auto-assign available port (default 7777, fallback 7778+)
AND SHALL write port to `~/.parseltongue/myapp.port`
AND SHALL include timestamp and process ID in port file
AND SHALL cleanup port file on graceful shutdown
```

**Source**: `ES-V200-attempt-01.md` Part IV

#### R3: Shutdown CLI Command [DECIDED]

```
WHEN user runs `parseltongue shutdown --slug myapp`
THEN SHALL read port from `~/.parseltongue/myapp.port`
AND SHALL send graceful shutdown signal to HTTP server
AND SHALL wait for server confirmation or timeout (5s)
AND SHALL remove port file after shutdown
```

**Source**: `ES-V200-attempt-01.md` Part IV

#### R4: XML-Tagged Responses [DECIDED]

```
WHEN gateway serves any list/detail/query endpoint
THEN SHALL wrap response in semantic XML tags
AND SHALL nest entities/edges under appropriate parent tags
AND SHALL preserve JSON structure within XML CDATA sections for complex fields
AND SHALL include schema version in root XML element
```

**Source**: `ES-V200-attempt-01.md` Part IV

#### R5: Project Slug in URL [DECIDED]

```
WHEN gateway mounts HTTP routes
THEN SHALL derive slug from ingested folder name or explicit config
AND SHALL prefix all endpoints with `/{slug}/`
AND SHALL reject ambiguous slug derivations with explicit error
```

**Source**: `ES-V200-attempt-01.md` Part IV

#### R6: Slug in Port File [DECIDED]

```
WHEN gateway writes port discovery file
THEN SHALL name file `~/.parseltongue/{slug}.port`
AND SHALL include slug in file contents for verification
AND SHALL support concurrent servers for different slugs
```

**Source**: `ES-V200-attempt-01.md` Part IV

#### R7: Token Count at Ingest [DECIDED]

```
WHEN tree-extractor parses entity
THEN SHALL compute token count using tiktoken or equivalent
AND SHALL persist `token_count` field in entity record
AND SHALL maintain token count totals across delta updates
AND SHALL export token counts in snapshot format
```

**Source**: `ES-V200-attempt-01.md` Part IV

#### R8: Data-Flow Tree-Sitter Queries [DECIDED]

```
WHEN tree-extractor processes source file
THEN SHALL extract assignment edges (X = func())
AND SHALL extract parameter flow edges (func(X))
AND SHALL extract return flow edges (return X)
AND SHALL classify flow edge types (Assign, Param, Return)
AND SHALL link data-flow edges to existing entity keys
```

**Source**: `ES-V200-attempt-01.md` Part IV

---

## 5. Three-Layer Architecture [DECIDED]

### 5.1 The Fundamental Insight

The LLM was doing TWO jobs:
1. **Type-level semantics**: "What type is this? Which trait? Is this async?" — questions with CORRECT ANSWERS
2. **Business judgment**: "Is this code critical? Is this cycle intentional?" — questions requiring INTERPRETATION

**pt04/rust-semantics gives us compiler-grade answers to job #1.** Not guesses. Not 88% confidence. **100% ground truth.**

**Source**: `ES-V200-attempt-01.md` Part I

### 5.2 What Changes vs What Stays

| Question | Before (LLM guesses) | After (Compiler knows) | Still needs LLM? |
|----------|---------------------|------------------------|------------------|
| What type does `authenticate` return? | LLM reads source: "probably Result<User, Error>" | `Result<User, AuthError>` (exact) | No |
| Is this a trait dispatch or direct call? | LLM infers from naming | `TraitMethod via AuthService` (exact) | No |
| Which trait hierarchy? | LLM guesses from visible impls | `A: Handler -> Service -> Debug + Send` (exact) | No |
| What does this closure capture? | LLM usually can't see this | `[db: &mut Conn (MutableRef), config: Arc<Config> (SharedRef)]` | No |
| How many distinct traits does X consume? | LLM reads all callees manually | `unique_traits_consumed: 5` (exact count) | No |
| Is this code revenue-critical? | N/A | N/A | **Yes** |
| Is this cycle a design pattern? | Compiler provides evidence | LLM applies judgment | **Sometimes** |
| What should we name this module? | N/A | N/A | **Yes** |
| How should we explain this to a developer? | N/A | N/A | **Yes** |

**Pattern**: Compiler handles **WHAT IS**. LLM handles **WHAT IT MEANS** and **WHAT TO DO ABOUT IT**.

**Source**: `ES-V200-attempt-01.md` Part I

### 5.3 Semantic Enrichment: TypedCallEdges

**One CozoDB relation** unlocks semantic enrichment across ALL endpoints:

```datalog
TypedCallEdges {
    from_key: String,
    to_key: String =>
    call_kind: String,      # Direct, TraitMethod, DynDispatch, ClosureInvoke
    via_trait: String?,      # Which trait, if any
    receiver_type: String?,  # The concrete receiver type
}
```

This single relation enables:
- Cycle classification (INTENTIONAL_PATTERN vs VIOLATION based on `call_kind`)
- Module boundaries via trait seeds (Leiden clustering on `via_trait`)
- Complexity analysis via trait counting (`unique_traits_consumed`)
- Refactoring evidence (trait dispatch vs direct call breakdown)
- Typed PageRank/betweenness (filter by `call_kind=TraitMethod`)
- Entropy over 8 edge types (vs 3 syntactic types)

**Source**: `ES-V200-attempt-01.md` Part V

---

## 6. EntityKey Format Decision

### 6.1 Status: PROPOSED (Pending CF-P1 Probes)

ZAI proposes `|||` delimiter format, but ground truth shows TBD pending CF-P1 probes.

**Decision Status**: **PROPOSED** - Awaiting CF-P1-F gate probe validation

### 6.2 PROPOSED Format

```
{language}|||{entity_kind}|||{scope}|||{name}|||{file_path}|||{discriminator}
```

**Example**:
```
rust|||fn|||my_crate::auth::handlers|||login|||src/auth.rs|||d0
python|||class|||services.auth_service|||AuthService|||lib/services/auth_service.py|||d0
java|||method|||com.example.parser.Parser|||parse|||src/Parser.java|||String_int
```

**Source**: `Prep-V200-Key-Format-Design.md`, `ES-V200-attempt-01.md` Part X (Open Question #1)

### 6.3 PROPOSED Rust Struct

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntityKey {
    pub language: Language,
    pub kind: EntityKind,
    pub scope: Vec<String>,
    pub name: String,
    pub file_path: String,
    pub discriminator: String,
}
```

**Source**: `Prep-V200-Key-Format-Design.md`

### 6.4 EntityKey Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| M1 | Unique across 12+ languages | Same name + same file = different keys |
| M2 | Human-readable | LLMs must understand keys without lookup |
| M3 | Stable across minor edits | Comments/blank lines don't change keys |
| M4 | Hierarchical | Encode containment: file > module > class > method |
| M5 | Hashable | HashMap key efficiency |
| M6 | Serializable | JSON/MessagePack/MCP support |
| M7 | Delimiter-safe | `|||` has zero collisions across 15 languages |
| M8 | No sanitization required | Raw generic types, module paths |
| M9 | Deterministic | Same input = same key, every time |
| M10 | Overload-distinguishable | Same name + different params = different keys |

**Source**: `Prep-V200-Key-Format-Design.md`

### 6.5 Key Format Risk Assessment

| Area | Risk Level | Open Questions |
|------|------------|----------------|
| Scope representation | LOW | Q1: Vec<String> vs Vec<ScopeSegment> |
| Scope extraction | MEDIUM | Q12: Per-language extraction depth |
| String interning | LOW | Q2: Memory vs complexity tradeoff |
| Path normalization | LOW | Q3: One normalize_file_path() function |
| External entities | LOW-MEDIUM | Q5: EXTERNAL: prefix format |
| rust-analyzer bridge | MEDIUM | Q6: DefId-to-EntityKey mapping |
| Discriminator | LOW | Q7, Q13: Format rules defined |

**Verdict**: Key format is LOW risk overall. RFC exists, questions are bounded.

**Source**: `Prep-V200-isgl1-ambiguity-risk-table.md`

---

## 7. Probe Sets

### 7.1 Core-Foundation Probes (CF-P1)

| Probe | Intent | Pass Criterion |
|-------|--------|----------------|
| CF-P1-A | Overload collision corpus probe | 0 collisions for overload set across Java/C++/C#/TS fixtures |
| CF-P1-B | Minor-edit stability mutation probe | Keys unchanged for whitespace/comment-only edits |
| CF-P1-C | Build/parse roundtrip probe | parse(build(identity)) == canonical identity view |
| CF-P1-D | Delimiter-safety cross-language probe | No escaping/sanitization needed for valid language symbols |
| CF-P1-E | External entity identity probe | Stable keys for std/crate/third-party references |
| **CF-P1-F** | **Slim types gate probe (G1)** | Canonical entity/storage schema remains minimal and deterministic |

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 01

### 7.2 Store-Runtime Probes (SR-P2)

| Probe | Intent | Pass Criterion |
|-------|--------|----------------|
| SR-P2-A | Bounded query guard probe | Heavy queries are rejected/segmented before unsafe allocation |
| SR-P2-B | Atomic commit rollback probe | Mid-commit failure yields zero partial writes |
| SR-P2-C | Idempotent delta replay probe | Replay(delta) twice => identical final state hash |
| SR-P2-D | Snapshot compatibility probe | Version mismatch yields explicit fail or migration path |
| SR-P2-E | Transient parse-failure quarantine probe | Single transient parse fail cannot hard-delete stable records |
| **SR-P2-F** | **Single getter contract gate probe (G2)** | All read-path callers resolve through one getter contract with result/error parity |
| SR-P2-G | Token-count ingest contract probe (R7) | Ingest persists deterministic `token_count` per entity and replay preserves totals |

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 02

### 7.3 Rust-Semantics Probes (RS-P3)

| Probe | Intent | Pass Criterion |
|-------|--------|----------------|
| RS-P3-A | Version-pinning canary probe | Any ra_ap skew fails fast with explicit diagnostic |
| RS-P3-B | Proc-macro chaos probe | Crash/unavailable macro yields degrade annotation, not silence |
| RS-P3-C | Build-script variance probe | Output delta is deterministic or flagged as non-deterministic |
| RS-P3-D | Semantic degrade integrity probe | Unknown/degraded facts are tagged and excluded from strict joins |
| RS-P3-E | Resource envelope probe | Ingest obeys configured memory/time ceiling with graceful exit |

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 03

### 7.4 Tree-Extractor Probes (TE-P4)

| Probe | Intent | Pass Criterion |
|-------|--------|----------------|
| TE-P4-A | Blind-spot fixture probe | Captures const-object exports + decorated defs in golden set |
| TE-P4-B | Capture-schema contract probe | 100% of emitted facts include required capture schema fields |
| TE-P4-C | Degrade-visibility probe | Partial parse paths always emit explicit failure/degrade tags |
| TE-P4-D | Determinism replay probe | Two identical runs produce identical extraction digest |
| TE-P4-E | Incremental-range probe | Range-bounded run equals full run on changed spans only |
| **TE-P4-F** | **Data-flow extraction contract probe (R8)** | Assign/param/return edges are extracted with fixture-backed parity |

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 04

### 7.5 Cross-Boundaries Probes (CB-P5)

| Probe | Intent | Pass Criterion |
|-------|--------|----------------|
| CB-P5-A | Compile-time boundary precision probe | FFI/WASM/PyO3 fixture precision >= 0.90 with reproducible edge set |
| CB-P5-B | Runtime ambiguity containment probe | HTTP/MQ fixtures maintain bounded FP rate under threshold policy |
| CB-P5-C | Confidence calibration replay probe | Threshold flips are deterministic and fully explainable by signals |
| CB-P5-D | Multi-pattern dedupe stability probe | Same source/target pair resolves to stable final edge + reasons |
| CB-P5-E | External/config-indirect quarantine probe | Non-literal/external matches are tagged unresolved, not promoted |

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 05

### 7.6 Graph-Reasoning Probes (GR-P6)

| Probe | Intent | Pass Criterion |
|-------|--------|----------------|
| GR-P6-A | Rule-soundness golden-corpus probe | Derived taint/policy/reachability sets match expected fixtures |
| GR-P6-B | Provenance completeness probe | 100% findings include explain path + rule identifier |
| GR-P6-C | Confidence calibration probe | Confidence bands track fixture truth labels with bounded drift |
| GR-P6-D | Delta-workload budget probe | Repeated update batches stay within budget or emit bounded partial markers |
| GR-P6-E | Determinism-ordering probe | Input permutation does not change final reasoning digest |

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 06

### 7.7 Interface-Gateway Probes (GW-P7)

| Probe | Intent | Pass Criterion |
|-------|--------|----------------|
| GW-P7-A | Cross-transport parity probe | Same semantic request (CLI/HTTP/MCP) yields identical result digest |
| GW-P7-B | Stdio hygiene probe | MCP stdio run emits protocol frames on stdout and logs on stderr only |
| GW-P7-C | Mode/capability negotiation probe | /db and /mem capability deltas are machine-readable and stable |
| GW-P7-D | Fail-closed readiness probe | Backend init failure blocks data-dependent routes with explicit readiness error |
| GW-P7-E | Cancellation/timeout contract probe | Deadline breach yields deterministic cancellation outcome and no partial-success report |
| **GW-P7-F** | **Filesystem source-read contract gate probe (G3)** | Detail-view path returns current disk lines for valid range and explicit errors for missing/moved files |
| GW-P7-G | Route-prefix nesting contract probe (R1) | Mode namespace routes are stable and wrong-prefix guidance is deterministic |
| GW-P7-H | Auto-port lifecycle contract probe (R2) | Startup allocates available port and writes discoverable port file consistently |
| GW-P7-I | Shutdown CLI contract probe (R3) | CLI shutdown path reaches server stop endpoint and removes discovery file |
| GW-P7-J | XML-tagged response contract probe (R4) | Response payloads expose deterministic semantic grouping keys for LLM use |
| GW-P7-K | Project-slug URL contract probe (R5) | Routes include project slug with stable derivation from workspace identity |
| GW-P7-L | Slug-aware port-file contract probe (R6) | Discovery file naming includes slug + mode and resolves target server deterministically |

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 07

### 7.8 Test-Harness Probes (TH-P8)

| Probe | Intent | Pass Criterion |
|-------|--------|----------------|
| TH-P8-A | Fixture-contract executability probe | EXPECTED corpus is machine-validated with zero manual interpretation paths |
| TH-P8-B | API-drift compile gate probe | Test suites fail-fast on interface signature drift with actionable diagnostics |
| TH-P8-C | Cross-platform parity probe | Linux/macOS/Windows verdict digests match for identical fixture sets |
| TH-P8-D | Flake-budget stability probe | Repeat runs stay within defined flake threshold window |
| TH-P8-E | Performance/concurrency budget probe | Contract+probe suites meet runtime budgets under parallel load |
| **TH-P8-F** | **Path-normalization coverage gate probe (G4)** | Coverage treats ./path, path, and absolute path as one canonical file |

**Source**: `ES-V200-Hashing-Risks-v01.md` Pass 08

---

## 8. The Killer Feature: get_context MCP Tool [DECIDED]

### 8.1 Why This Matters

The **moat** isn't tree-sitter parsing or rust-analyzer integration. Those are table stakes. The moat is:
> **Architecturally-ranked LLM context that no other tool can provide.**

**Source**: `ES-V200-attempt-01.md` Part XII

### 8.2 MCP Tool Schema

```json
{
  "name": "get_context",
  "description": "Get the most architecturally relevant code context for a given entity, optimized for your token budget. Uses PageRank, blast radius, SCC membership, Leiden community clustering, and coupling metrics to rank what to include.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "entity": {
        "type": "string",
        "description": "Entity key (e.g., 'rust:fn:handle_request')"
      },
      "token_budget": {
        "type": "integer",
        "description": "Maximum tokens to include (default: 4096)",
        "default": 4096
      },
      "include_callers": {
        "type": "boolean",
        "description": "Include entities that call this entity",
        "default": true
      },
      "include_callees": {
        "type": "boolean",
        "description": "Include entities this entity calls",
        "default": true
      }
    },
    "required": ["entity"]
  }
}
```

**Source**: `Prep-V200-MCP-Protocol-Integration.md`, `ES-V200-attempt-01.md` Part XII

### 8.3 Ranking Signals (8 Total, Calibrated Weights)

| Signal | Weight | Purpose |
|--------|--------|---------|
| **Blast Radius** | 0.30 | Local relevance - distance from focus entity |
| **SCC Membership** | 0.20 | Mandatory co-inclusion - tightly coupled group |
| **Leiden Community** | 0.10 | Module cohesion - same community bonus |
| **PageRank** | 0.10 | Global importance - architectural pillars |
| **CK Metrics** | 0.10 | Complexity-driven need - high coupling = more context |
| **Cross-Language** | 0.10 | API contract relevance - FFI/WASM boundaries |
| **K-Core** | 0.05 | Structural depth - core vs periphery |
| **Entropy** | 0.05 | Complexity amplifier - high entropy = more context |

**Total**: 1.00 (fully calibrated system)

**Source**: `Prep-V200-LLM-Context-Optimization-Research.md`, `ES-V200-attempt-01.md` Part XII

### 8.4 Token Budgeting Algorithm (Hybrid Approach)

```
1. Compute unified ranking scores for all entities
2. Identify Leiden communities; allocate budget proportionally:
   - Community with focus entity: 50%
   - Adjacent communities (1-hop): 30%
   - High-PageRank pillars: 15%
   - Reserve: 5% for cross-language + SCC completions
3. Within each community, use hierarchical phases:
   - Phase 1 (40%): Signatures of all relevant entities
   - Phase 2 (40%): Bodies of highest-ranked entities
   - Phase 3 (20%): Relationship annotations
4. Cap at 80% of stated budget (Context Rot research finding)
```

**Source**: `Prep-V200-LLM-Context-Optimization-Research.md`, `ES-V200-attempt-01.md` Part XII

---

## 9. Risk Matrix and Evidence

### 9.1 Baseline Risk/Unclear Matrix

| Crate | Risk / 5 | Unclear / 5 | Why Baseline is Not Low |
|-------|----------|-------------|-------------------------|
| rust-llm-interface-gateway | 3 | 2 | Unified behavior across CLI/HTTP/MCP + cancellation |
| rust-llm-core-foundation | 4 | 4 | Key model and contract stability affect all crates |
| rust-llm-tree-extractor | 4 | 3 | 12-language query correctness and normalization gaps |
| rust-llm-rust-semantics | 5 | 4 | RA/proc-macro/build-script reliability and churn |
| rust-llm-cross-boundaries | 4 | 4 | Heuristic linking quality and confidence calibration |
| rust-llm-graph-reasoning | 4 | 3 | Rule correctness/scale tradeoffs in V200 scope |
| rust-llm-store-runtime | 5 | 4 | Delta consistency, indexing, snapshot durability |
| rust-llm-test-harness | 3 | 3 | Fixture breadth vs CI time and anti-flakiness design |

**Source**: `ES-V200-Hashing-Risks-v01.md` Section "Baseline Risk/Unclear Matrix (v01)"

### 9.2 Risk Hash Snapshot (After All Passes)

```
rust-llm-core-foundation:R4-U2-E4
rust-llm-store-runtime:R5-U4-E4
rust-llm-rust-semantics:R5-U4-E5
rust-llm-tree-extractor:R4-U3-E5
rust-llm-cross-boundaries:R4-U4-E6
rust-llm-graph-reasoning:R4-U3-E7
rust-llm-interface-gateway:R3-U2-E8
rust-llm-test-harness:R3-U3-E9
```

**Source**: `ES-V200-Hashing-Risks-v01.md` Section "Risk Hash Snapshot History"

### 9.3 Evidence Captured (E01-E55)

Evidence items are documented in `ES-V200-Hashing-Risks-v01.md` across all 8 passes:

- **E01-E05**: core-foundation (key model evidence)
- **E03-E06**: store-runtime (memory/snapshot evidence)
- **E07-E11**: rust-semantics (RA integration evidence)
- **E12-E16**: tree-extractor (query pattern evidence)
- **E17-E22**: cross-boundaries (detection pattern evidence)
- **E23-E29**: graph-reasoning (Ascent/rule evidence)
- **E37-E44**: interface-gateway (transport evidence)
- **E45-E53**: test-harness (fixture/CI evidence)
- **E54-E55**: core-foundation (historical key evidence)

### 9.4 Gate Evidence Backlog (E56-E59, Pending Capture)

| ID | Gate | Artifact Set |
|----|------|--------------|
| E56 | G1 | Slim-types artifact set (canonical schema snapshot + deterministic serialization digest) |
| E57 | G2 | Single-getter artifact set (handler-to-getter call-path map + result/error parity report) |
| E58 | G3 | Source-read artifact set (detail-view fixtures for valid/missing/moved files + line-range outcomes) |
| E59 | G4 | Path-normalization artifact set (`./x` vs `x` vs absolute fixture parity in coverage outputs) |

**Source**: `ES-V200-Hashing-Risks-v01.md` Section "Gate Evidence Backlog"

### 9.5 Promoted Requirement Evidence Backlog (E60-E67, Pending Capture)

| ID | Requirement | Artifact Set |
|----|-------------|--------------|
| E60 | R1 | Route-prefix artifact set (mode namespace fixtures + wrong-prefix error contract) |
| E61 | R2 | Auto-port artifact set (startup assignment + discovery file lifecycle traces) |
| E62 | R3 | Shutdown-cli artifact set (CLI-to-endpoint shutdown handshake and cleanup report) |
| E63 | R4 | XML-tagged response artifact set (grouped-schema fixtures across list/detail/query responses) |
| E64 | R5 | Project-slug URL artifact set (slug derivation and route mounting parity fixtures) |
| E65 | R6 | Slug port-file artifact set (slug-aware discovery filename and lookup parity) |
| E66 | R7 | Token-count ingest artifact set (`token_count` persistence and replay parity digest) |
| E67 | R8 | Data-flow extraction artifact set (assign/param/return edge fixture truth set) |

**Source**: `ES-V200-Hashing-Risks-v01.md` Section "Promoted Requirement Evidence Backlog"

### 9.6 Hard Scoring Policy

- Do not reduce Risk/Unclear for gate-linked crates until the linked F-probe passes with artifact references.
- Gate to probe mapping:
  - G1 -> CF-P1-F
  - G2 -> SR-P2-F
  - G3 -> GW-P7-F
  - G4 -> TH-P8-F

**Source**: `ES-V200-Hashing-Risks-v01.md` Section "Hard Scoring Policy for Gate-Linked Crates"

---

## 10. Implementation Phases (LNO)

### 10.1 LNO Framework [DECIDED]

Based on Shreyas Doshi's Leverage-Neutral-Overhead prioritization:

| Phase | Category | Description |
|-------|----------|-------------|
| **LEVERAGE** | Ship First (80% of Value) | TypedCallEdges only |
| **NEUTRAL** | Build Next (If Phase 1 Proves Valuable) | TraitImpls + SupertraitEdges |
| **OVERHEAD** | Build Last (Or Never) | Full MCP, TypeLayouts |

**Source**: `ES-V200-attempt-01.md` Part VII

### 10.2 LEVERAGE: Phase 1 [DECIDED]

**Deliverables**:
- `rust-llm-core-foundation` (keys + contracts)
- `rust-llm-tree-extractor` (12 languages)
- `rust-llm-rust-semantics` (TypedCallEdges only)
- `rust-llm-store-runtime` (single getter contract)
- `rust-llm-interface-gateway` (HTTP only, no MCP yet)

**Measure**: Does Leiden modularity improve? Do LLM code reviews catch more issues?

**Source**: `ES-V200-attempt-01.md` Part VII

### 10.3 NEUTRAL: Phase 2 [DECIDED]

**Deliverables**:
- TraitImpls + SupertraitEdges
- `/trait-hierarchy-graph-view` endpoint
- Better cycle classification (supertrait pattern matching)
- SemanticTypes (Return Types, Params, Visibility, Async/Unsafe)

**Source**: `ES-V200-attempt-01.md` Part VII

### 10.4 OVERHEAD: Deferred [DECIDED]

- TypeLayouts (performance optimization, niche)
- ClosureCaptures as separate endpoint (should be IN TypedCallEdges)
- Generic instantiation maps (niche)
- Full MCP protocol support (wait for adoption signal)

**Source**: `ES-V200-attempt-01.md` Part VII

### 10.5 Deferred to v210 [DECIDED]

| Item | Reason | Source |
|------|--------|--------|
| rust-llm-context-packer | V200 focuses on graph-grounded context adequacy, not token minimization | `ES-V200-Decision-log-01.md` |
| Token-minimization strategies | Optimization layer that can mask weak retrieval/fidelity | `ES-V200-Decision-log-01.md` |

---

## 11. Open Questions

### 11.1 CRITICAL: EntityKey Format [OPEN]

**Question**: Decide format for `build_entity_key_string()`

**Status**: ZAI proposes `|||` delimiter format, ground truth shows TBD pending CF-P1 probes.

**Resolution Needed**: Phase 1

**Source**: `ES-V200-attempt-01.md` Part X (Open Question #1)

### 11.2 HIGH Priority [OPEN]

| ID | Question | Status | Source |
|----|----------|--------|--------|
| Q2 | TypedCallEdges Schema: Should `generic_instantiation` be separate field or nested JSON? | OPEN | `ES-V200-attempt-01.md` Part X |
| Q3 | Semantic Coverage Reporting: How to surface "93.5% semantic coverage, 4 proc-macro files lack typed edges"? | OPEN | `ES-V200-attempt-01.md` Part X |
| Q5 | MCP vs HTTP Priority: Ship HTTP-only first (Phase 1), or delay until MCP is ready? | **RECOMMENDED: HTTP first** | `ES-V200-attempt-01.md` Part X |
| Q8 | rust-analyzer Version Pinning: Which `ra_ap` version to pin? How to handle API churn? | OPEN | `ES-V200-attempt-01.md` Part X |
| Q9 | Datalog Rule Catalog: Which of the 18 rules are MVP vs Phase 2? | **RECOMMENDED: SCC, Reachability, Dead Code, God Object in Phase 1** | `ES-V200-attempt-01.md` Part X |
| Q10 | Cross-Language Confidence Thresholds: Accept ZAI thresholds (0.80+, 0.60-0.79, 0.40-0.59)? | OPEN | `ES-V200-attempt-01.md` Part X |

### 11.3 MEDIUM Priority [OPEN]

| ID | Question | Status | Source |
|----|----------|--------|--------|
| Q4 | LLM Integration Point: Should LLM calls be in-process (via library) or out-of-process (via HTTP)? | OPEN | `ES-V200-attempt-01.md` Part X |
| Q6 | Tauri App Scope: Should Tauri app be in V200 scope, or shipped separately post-V200? | **RECOMMENDED: External companion** | `ES-V200-attempt-01.md` Part X |
| Q7 | Test Fixtures: Reuse 94 v173 fixtures, or create NEW fixtures for V200 contracts? | **RECOMMENDED: Create NEW fixtures** | `ES-V200-attempt-01.md` Part X |

---

## 12. Datalog Rules (18 Total) [DECIDED]

These rules are what competitors (CodeQL, Semgrep) lock behind proprietary systems. rust-llm open-sources them.

| Rule | Purpose | Phase |
|------|---------|-------|
| **Transitive Trait Hierarchy** | `all_supers(t, s)` via supertrait recursion | Phase 2 |
| **Unsafe Call Chain** | `unsafe_chain(f)` - functions transitively reaching unsafe | Phase 1 |
| **Taint Analysis** | `tainted(f)` - data flow from sources to sinks | Phase 2 |
| **Reachability** | `reachable(a, b)` - full transitive closure | Phase 1 |
| **Dead Code Detection** | `dead_code(f)` - unreachable from entry points | Phase 1 |
| **Layer Violation** | `layer_violation(f, g)` - architectural boundary violations | Phase 2 |
| **Async Boundary** | `async_boundary(f, g)` - async calling sync (blocking) | Phase 2 |
| **Circular Deps** | `circular_dep(a, b)` - mutual module reachability | Phase 1 |
| **Coupling Metrics** | `cbo(m)` - Coupling Between Objects via Datalog | Phase 2 |
| **API Surface** | `api_surface(e)` - public entities and dependencies | Phase 2 |
| **Closure Captures** | `closure_captures_unsafe(c, p)` - unsafe context captures | Phase 2 |
| **Error Propagation** | `error_chain(a, b)` - Result/Option flow chains | Phase 2 |
| **Module Cohesion** | `same_module_edge(e1, e2)` - internal module connections | Phase 2 |
| **Test Coverage Gap** | `untested_pub_fn(f)` - public functions without tests | Phase 2 |
| **Derive Macro Inference** | `derive_impl(key, trait)` - #[derive(...)] trait impls | Phase 2 |
| **God Object** | `god_object(f)` - high fan-in + fan-out | Phase 1 |
| **SCC Membership** | `in_scc(a, rep)` - entity group membership | Phase 1 |
| **Cross-Language Edge Join** | `ffi_match(rust_key, c_key)` - FFI name matching | Phase 2 |

**Source**: `Prep-V200-Datalog-Ascent-Rule-Patterns.md`, `ES-V200-attempt-01.md` Part XII

---

## 13. API Contracts

### 13.1 HTTP REST Endpoints (22 Total) [DECIDED]

| Category | Endpoint | Description |
|----------|----------|-------------|
| Core | `/server-health-check-status` | Health check |
| Core | `/codebase-statistics-overview-summary` | Stats summary |
| Core | `/api-reference-documentation-help` | API docs |
| Entity | `/code-entities-list-all` | All entities |
| Entity | `/code-entity-detail-view/{key}` | Entity detail |
| Entity | `/code-entities-search-fuzzy?q=pattern` | Fuzzy search |
| Graph | `/dependency-edges-list-all` | All edges |
| Graph | `/reverse-callers-query-graph?entity=X` | Who calls X? |
| Graph | `/forward-callees-query-graph?entity=X` | What does X call? |
| Analysis | `/blast-radius-impact-analysis?entity=X&hops=N` | Impact analysis |
| Analysis | `/circular-dependency-detection-scan` | Cycle detection |
| Analysis | `/complexity-hotspots-ranking-view?top=N` | Coupling hotspots |
| Analysis | `/semantic-cluster-grouping-list` | Module clusters |
| Advanced | `/smart-context-token-budget?focus=X&tokens=N` | LLM context |
| Graph v1.6 | `/strongly-connected-components-analysis` | Tarjan SCC |
| Graph v1.6 | `/technical-debt-sqale-scoring?entity=X` | SQALE tech debt |
| Graph v1.6 | `/kcore-decomposition-layering-analysis?k=N` | K-core layering |
| Graph v1.6 | `/centrality-measures-entity-ranking?method=pagerank` | PageRank/Betweenness |
| Graph v1.6 | `/entropy-complexity-measurement-scores?entity=X` | Shannon entropy |
| Graph v1.6 | `/coupling-cohesion-metrics-suite?entity=X` | CK metrics |
| Graph v1.6 | `/leiden-community-detection-clusters` | Leiden clustering |
| Coverage | `/ingestion-coverage-folder-report?depth=N` | Ingestion coverage |

**Source**: `CLAUDE.md` (project instructions)

### 13.2 Cross-Language Detection Patterns [DECIDED]

| Pattern | Base Confidence | False Positive Rate |
|---------|-----------------|---------------------|
| FFI (raw extern) | 0.90 | 3-8% |
| FFI (CXX bridge) | 0.95 | 1-3% |
| WASM (wasm_bindgen) | 0.88 | 5-10% |
| PyO3 | 0.85 | ~12% |
| JNI | 0.88 | Varies |
| gRPC | 0.85 | 5-10% |
| HTTP | 0.65 | 10-20% |
| Message Queue | 0.70 | 15-25% |

**Confidence Thresholds**:
- >= 0.80: High confidence, include in graph by default
- 0.60-0.79: Medium confidence, include with "uncertain" flag
- 0.40-0.59: Low confidence, include only if user opts in
- < 0.40: Rejected, do not include

**Source**: `Prep-V200-Cross-Language-Detection-Heuristics.md`

---

## 14. Source Document Index

| Document | Lines | Key Contribution |
|----------|-------|------------------|
| ES-V200-attempt-01.md | 762 | Comprehensive requirements draft |
| ES-V200-Hashing-Risks-v01.md | 1189 | 8-crate architecture + gates + probes |
| ES-V200-Decision-log-01.md | ~60 | Binding decisions |
| ES-V200-Dependency-Graph-Contract-Hardening.md | 120 | Pass method definition |
| ES-V200-User-Journey-01.md | 650 | Tauri journey + acceptance criteria |
| ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md | 400 | Tauri as CLI launcher |
| PRD-v200.md | 20 | Clean break requirements |
| Prep-V200-Competitive-Deep-Dive.md | 1400 | Competitor analysis |
| Prep-V200-Compiled-Research-Best-Ideas.md | 800 | Priority matrix |
| Prep-V200-Cross-Language-Detection-Heuristics.md | 1822 | 5 patterns + confidence scoring |
| Prep-V200-Datalog-Ascent-Rule-Patterns.md | 1200 | 18 derived rules |
| Prep-V200-Key-Format-Design.md | 762 | 4 candidates + recommendation |
| Prep-V200-LLM-Context-Optimization-Research.md | 1071 | 8 ranking signals |
| Prep-V200-Max-Adoption-Architecture-Strategy.md | 800 | 4 architecture options |
| Prep-V200-MCP-Protocol-Integration.md | 1100 | MCP spec + tool mapping |
| Prep-V200-Rust-Analyzer-API-Surface.md | 500 | 20 API methods |
| Prep-V200-Tree-Sitter-Query-Patterns.md | 1400 | Query syntax + patterns |
| Prep-V200-isgl1-ambiguity-risk-table.md | 800 | 18 questions + reassessment |
| v200-doc-index-01.md | 30 | Curated index |

**Total Research**: ~15,000 lines across 18 documents

---

## Appendix A: WHEN/THEN/SHALL Contract Template

All V200 requirements and gates follow this format:

```
WHEN {precondition describing input state and action}
THEN SHALL {postcondition describing guaranteed outcome}
AND SHALL {additional postcondition}
AND SHALL NOT {forbidden outcome}
```

This format makes requirements **executable** and **testable**.

**Source**: `ES-V200-attempt-01.md` Appendix B

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| EntityKey | Unique identifier format for code entities |
| TypedCallEdges | Primary semantic enrichment relation |
| Leiden | Community detection algorithm |
| Tarjan SCC | Strongly Connected Components algorithm |
| SQALE | Software Quality Assessment based on Lifecycle Expectations |
| CK Metrics | Chidamber-Kemerer metrics (CBO, LCOM, RFC, WMC) |
| LNO | Leverage-Neutral-Overhead prioritization |
| RAII | Resource Acquisition Is Initialization |
| MCP | Model Context Protocol |
| FFI | Foreign Function Interface |

---

## Appendix C: High-Priority Hardening Queue (80-89)

| Crate | Item | Contract/Probe Linkage |
|-------|------|------------------------|
| rust-llm-test-harness | #19, #20 | TH-P8 (coverage contract and fixture checks) |
| rust-llm-tree-extractor | #26, #23 | TE-P4 (extraction integrity + clean artifacts) |
| rust-llm-interface-gateway | #6, #24 | GW-P7 (mode contract + runtime hygiene) |
| rust-llm-store-runtime | #3 | SR-P2 (snapshot/consistency and replay checks) |

**Source**: `ES-V200-Hashing-Risks-v01.md` Section "V200 High-Priority Hardening Queue"

---

*Generated: 2026-02-17*
*Status: FINAL DEFINITIVE SYNTHESIS*
*Total Source Documents: 18 + 3 primary sources*
