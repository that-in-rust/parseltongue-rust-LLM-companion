# ES-V200-attempt-01: Executable Contract Ledger (Design101 Style)
Status: Requirements Draft v02
Date: 2026-02-17
Purpose: Convert V200 architecture intent into testable contracts with explicit preconditions, postconditions, error conditions, probes, and TDD sequencing.

## Canonical Inputs
- `docs/v200-docs/ES-V200-Hashing-Risks-v01.md` (primary hardening ledger, pass/probe ownership)
- `docs/v200-docs/ES-V200-Dependency-Graph-Contract-Hardening.md` (method contract)
- `docs/v200-docs/ES-V200-Decision-log-01.md` (binding scope decisions)
- `docs/v200-docs/Prep-V200-Key-Format-Design.md` (EntityKey decision)
- `docs/v200-docs/Prep-V200-Rust-Analyzer-API-Surface.md` (semantic capability contracts)
- `docs/v200-docs/Prep-V200-Tree-Sitter-Query-Patterns.md` (extractor coverage contracts)
- `docs/v200-docs/Prep-V200-Cross-Language-Detection-Heuristics.md` (boundary confidence contracts)
- `docs/v200-docs/Prep-V200-Datalog-Ascent-Rule-Patterns.md` (reasoning rule contracts)
- `docs/v200-docs/Prep-V200-MCP-Protocol-Integration.md` (transport/MCP contracts)
- `docs/v200-docs/Prep-V200-LLM-Context-Optimization-Research.md` (context ranking contracts)
- `docs/v200-docs/ES-V200-User-Journey-01.md` and addendum (companion UX contracts)
- `docs/v200-docs/PRD-v200.md` (clean-break requirement)

## Contract System Rules (Design101)
### RULE-C01: Executable Specification Only
Preconditions:
- Requirement text exists.
Contract:
WHEN a new requirement is added
THEN SHALL be written as executable clauses (`WHEN/THEN/SHALL`)
AND SHALL include preconditions, postconditions, and error conditions
AND SHALL NOT remain as narrative-only prose.

### RULE-C02: TDD Order Is Mandatory
Contract:
WHEN any contract is implemented
THEN SHALL follow `STUB -> RED -> GREEN -> REFACTOR`
AND SHALL define probes before score updates
AND SHALL NOT reduce risk/unclear from narrative confidence alone.

### RULE-C03: Evidence-First Score Movement
Contract:
WHEN crate Risk/Unclear scores are changed
THEN SHALL cite probe artifacts
AND SHALL include pass/fail outcomes for linked probes
AND SHALL NOT update scores from opinion.

### RULE-C04: Naming Constraint
Contract:
WHEN introducing new function/crate/command/folder names in V200 scope
THEN SHALL use four-word naming convention
AND SHALL follow `verb_constraint_target_qualifier` pattern.

## Binding Decision Contracts
### DEC-C01: Clean Break + Coexistence
Source: `PRD-v200.md`
Contract:
WHEN V200 ships
THEN SHALL require zero backward compatibility with v1.x interfaces
AND SHALL keep v1.x code present and buildable in-repo
AND SHALL NOT delete v1.x crates as part of V200 delivery.

### DEC-C02: Active Scope = 8 Core Crates
Source: `ES-V200-Decision-log-01.md`
Contract:
WHEN V200 implementation scope is evaluated
THEN SHALL include exactly these core crates:
`rust-llm-interface-gateway`, `rust-llm-core-foundation`, `rust-llm-tree-extractor`, `rust-llm-rust-semantics`, `rust-llm-cross-boundaries`, `rust-llm-graph-reasoning`, `rust-llm-store-runtime`, `rust-llm-test-harness`
AND SHALL exclude `rust-llm-context-packer` from V200
AND SHALL track context-packer only in V210 backlog with explicit re-entry criteria.

### DEC-C03: Companion Boundary
Source: decision log + user-journey docs
Contract:
WHEN desktop/Tauri behavior is specified
THEN SHALL treat Tauri as external companion consumer of gateway/runtime contracts
AND SHALL prioritize visual CLI-launcher workflows over deep in-app orchestration
AND SHALL NOT introduce Tauri as a new core dependency-graph crate.

### DEC-C04: EntityKey Strategy (Binding)
Source: `Prep-V200-Key-Format-Design.md` + ambiguity table
Contract:
WHEN entity identity is represented internally
THEN SHALL use typed `EntityKey` struct (hashable/equatable)
AND WHEN identity crosses HTTP/MCP boundaries
THEN SHALL serialize as `language|||kind|||scope|||name|||file_path|||discriminator`
AND SHALL keep key deterministic, parseable, overload-safe, and URL-safe
AND SHALL NOT use line numbers as key identity.

Error conditions:
- Scope extraction incomplete -> emit best-effort empty scope with explicit capability marker (no silent key mutation).
- Discriminator unavailable -> fallback policy required (`ParamTypes` -> `Index` -> `ContentHash`).

## Non-Negotiable Gate Contracts
### G1-CF-P1-F: Slim Types Gate
Owner: `rust-llm-core-foundation`
Preconditions:
- Entity/storage schema defined.
- Build/parse roundtrip API available.
Contract:
WHEN keys and entity/storage schema are produced
THEN SHALL remain canonical, minimal, and deterministic
AND SHALL roundtrip parse/build without semantic loss
AND SHALL show zero overload collisions on fixture corpus
AND SHALL NOT require sanitizer escapes for valid language symbols.

Error conditions:
- Collision detected -> hard fail gate.
- Non-roundtrippable key -> hard fail gate.

Probe links:
- `CF-P1-A..F`

### G2-SR-P2-F: Single Getter Contract Gate
Owner: `rust-llm-store-runtime`
Preconditions:
- Canonical read getter defined.
- Read-path call map available.
Contract:
WHEN any read path is executed
THEN SHALL pass through one canonical getter contract
AND SHALL preserve result/error parity across callers
AND SHALL NOT allow bypass direct data reads.

Error conditions:
- Bypass path discovered -> hard fail gate.
- parity mismatch -> hard fail gate.

Probe links:
- `SR-P2-F` (+ `SR-P2-A..E`, `SR-P2-G` as supporting probes)

### G3-GW-P7-F: Filesystem Source-Read Contract Gate
Owner: `rust-llm-interface-gateway`
Preconditions:
- Key resolves to file and line range.
- Filesystem access available.
Contract:
WHEN detail view is requested for an entity key
THEN SHALL return current disk source lines for that range
AND SHALL emit explicit errors for missing/moved/permission-denied files
AND SHALL NOT return stale cached source.

Probe links:
- `GW-P7-F`

### G4-TH-P8-F: Path Normalization Coverage Gate
Owner: `rust-llm-test-harness`
Preconditions:
- Coverage scanner sees relative and absolute path variants.
Contract:
WHEN coverage is computed
THEN SHALL canonicalize `./path`, `path`, and absolute path to one identity
AND SHALL aggregate counts under one canonical file
AND SHALL report zero path-variant duplicates.

Probe links:
- `TH-P8-F`

## Promoted Requirement Contracts (R1-R8)
### R1-GW-P7-G: Route Prefix Nesting
Contract:
WHEN request routes are mounted
THEN SHALL namespace by project slug plus active mode path
AND SHALL reject wrong-prefix access with deterministic guidance.

### R2-GW-P7-H: Auto Port + Port File Lifecycle
Contract:
WHEN gateway starts for a slug
THEN SHALL allocate an available port deterministically
AND SHALL write discovery file at `~/.parseltongue/{slug}.port`
AND SHALL clean lifecycle metadata on graceful stop.

### R3-GW-P7-I: Shutdown CLI Contract
Contract:
WHEN `parseltongue shutdown {slug}` executes
THEN SHALL resolve target from slug-aware discovery file
AND SHALL stop server gracefully and remove discovery file
AND SHALL report deterministic success/failure.

### R4-GW-P7-J: XML-Tagged Response Contract
Contract:
WHEN list/detail/query responses are exported for LLM use
THEN SHALL include deterministic semantic grouping tags
AND SHALL preserve structured payload consistency across endpoints.

### R5-GW-P7-K: Project Slug in URL
Contract:
WHEN server routes are exposed
THEN SHALL include slug in URL namespace
AND SHALL derive slug deterministically from workspace identity.

### R6-GW-P7-L: Slug-Aware Port File Naming
Contract:
WHEN discovery file is written
THEN SHALL include slug in file name and contents
AND SHALL support concurrent servers for different slugs.

### R7-SR-P2-G: Token Count at Ingest
Owner: `rust-llm-store-runtime` (produced by extractor path)
Contract:
WHEN entities are ingested
THEN SHALL compute and persist deterministic `token_count`
AND SHALL preserve totals across replay/snapshot operations.

### R8-TE-P4-F: Data-Flow Extraction
Owner: `rust-llm-tree-extractor`
Contract:
WHEN extraction runs
THEN SHALL emit assign/param/return flow edges
AND SHALL classify flow edge types deterministically
AND SHALL link flow edges to canonical entity keys.

## Per-Crate Core Contracts
### CRT-C00: Interface Gateway
Contract:
WHEN semantically equivalent requests arrive via CLI/HTTP/MCP
THEN SHALL produce equivalent core result digests
AND SHALL expose transport capability deltas explicitly
AND SHALL keep stdout protocol-safe for MCP stdio.

Critical probes:
- `GW-P7-A` parity
- `GW-P7-B` stdio hygiene
- `GW-P7-C..E` capability + readiness + timeout/cancel

### CRT-C01: Core Foundation
Contract:
WHEN identity and fact contracts are validated
THEN SHALL enforce deterministic key and schema invariants
AND SHALL reject ambiguous/non-canonical identity representations.

Critical probes:
- `CF-P1-A..F`

### CRT-C02: Tree Extractor
Contract:
WHEN parser/query extraction executes
THEN SHALL emit deterministic syntax facts with explicit degrade markers
AND SHALL maintain capture-schema completeness across supported languages
AND SHALL provide fixture-backed data-flow correctness.

Critical probes:
- `TE-P4-A..F`

### CRT-C03: Rust Semantics
Contract:
WHEN rust-analyzer enrichment runs
THEN SHALL expose resolved semantic facts or explicit degrade reasons
AND SHALL pin `ra_ap` versions exactly
AND SHALL bound memory/time envelopes with explicit budget outcomes.

Critical probes:
- `RS-P3-A..E`

### CRT-C04: Cross-Boundaries
Contract:
WHEN cross-language links are derived
THEN SHALL include pattern type, confidence, and support signals per edge
AND SHALL quarantine unresolved/config-indirect matches
AND SHALL keep dedupe outcomes deterministic.

Critical probes:
- `CB-P5-A..E`

### CRT-C05: Graph Reasoning
Contract:
WHEN Ascent rules and algorithms derive findings
THEN SHALL emit deterministic outputs with provenance
AND SHALL respect runtime bounds and partial-result markers
AND SHALL reject invalid rule/strata configurations explicitly.

Critical probes:
- `GR-P6-A..E`

### CRT-C06: Store Runtime
Contract:
WHEN fact commits, deltas, snapshots, and queries run
THEN SHALL remain atomic, idempotent, and consistency-checkable
AND SHALL enforce bounded query behavior
AND SHALL fail closed on compatibility mismatches.

Critical probes:
- `SR-P2-A..G`

### CRT-C07: Test Harness
Contract:
WHEN contract suites execute
THEN SHALL provide machine-executable expectations and deterministic verdict digests
AND SHALL fail fast on API drift
AND SHALL enforce cross-platform parity and flake/perf budgets.

Critical probes:
- `TH-P8-A..F`

## Semantic and Reasoning Feature Contracts
### SEM-C01: TypedCallEdges Leverage Contract
Source: pt04 workflow + compiled research
Contract:
WHEN Rust semantic enrichment is available
THEN SHALL include typed call semantics (`Direct`, `TraitMethod`, `DynDispatch`, `ClosureInvoke`)
AND SHALL expose trait/receiver metadata for downstream algorithms
AND SHALL support endpoint enrichment without endpoint topology changes.

### SEM-C02: Datalog Rule Pack Contract
Source: `Prep-V200-Datalog-Ascent-Rule-Patterns.md`
Contract:
WHEN reasoning MVP is declared complete
THEN SHALL include executable rules for at least:
- transitive reachability
- SCC membership/cycles
- dead code
- unsafe chain propagation
- architecture boundary violations
AND SHALL include rule provenance in emitted findings.

### SEM-C03: Cross-Language Confidence Contract
Source: cross-language heuristics
Contract:
WHEN boundary edges are scored
THEN SHALL apply threshold policy:
- `>= 0.80` high confidence (default include)
- `0.60-0.79` medium (include + uncertain marker)
- `0.40-0.59` low (opt-in)
- `< 0.40` reject
AND SHALL classify compile-time and runtime pattern confidence separately.

### SEM-C04: Context Ranking Contract (`get_context`)
Source: MCP + context optimization research
Contract:
WHEN `get_context` is called with focus entity and token budget
THEN SHALL rank by weighted signals:
- blast radius 0.30
- SCC membership 0.20
- Leiden community 0.10
- PageRank 0.10
- CK metrics 0.10
- cross-language 0.10
- k-core 0.05
- entropy 0.05
AND SHALL apply budget policy:
- 50% focus community
- 30% adjacent communities
- 15% architectural pillars
- 5% reserve
AND SHALL cap emitted context at 80% of stated budget
AND SHALL provide ranking explanation metadata.

## Transport and MCP Contracts
### TRN-C01: MCP-First Compatibility Contract
Source: MCP integration research
Contract:
WHEN MCP stdio server mode is active
THEN SHALL emit only JSON-RPC frames to stdout
AND SHALL route logs/diagnostics to stderr
AND SHALL support initialize -> initialized lifecycle.

### TRN-C02: HTTP Coexistence Contract
Contract:
WHEN HTTP and MCP are both enabled
THEN SHALL share one core analysis/query logic path
AND SHALL preserve parity for equivalent capability calls
AND SHALL expose capability asymmetry explicitly, never implicitly.

### TRN-C03: Companion Command-Generation Contract
Source: Tauri addendum
Contract:
WHEN companion UI surfaces analysis actions
THEN SHALL generate copy-pasteable CLI/curl workflows with slug+port correctness
AND SHALL treat terminal workflows as primary for deep/composed operations.

## Quality and Performance Contracts
### QLT-C01: Determinism
Contract:
WHEN identical inputs are processed in repeated runs
THEN SHALL produce stable output digests (excluding timestamps)
AND SHALL not reorder semantic meaning nondeterministically.

### QLT-C02: Performance Envelope
Contract:
WHEN operating on baseline workloads
THEN SHALL maintain target envelopes:
- cache hit < 100ms
- incremental reindex < 500ms
- full cycle < 5000ms
AND SHALL emit bounded-failure artifacts on budget breaches.

### QLT-C03: Review Utility Outcome
Contract:
WHEN architecture review workflows run
THEN SHALL prioritize compiler-verified facts over inferred guesses
AND SHALL preserve 99% token reduction objective for LLM-facing context packages.

## Delivery Sequencing Contracts (LNO)
### PHS-C01: Leverage-First
Contract:
WHEN V200 phases are scheduled
THEN SHALL deliver leverage-first capabilities before neutral/overhead items
AND SHALL prioritize:
- gate closure (`G1..G4`)
- promoted requirement closure (`R1..R8`)
- TypedCallEdges and core context ranking foundations.

### PHS-C02: Phase Exit Criteria
Contract:
WHEN moving from phase N to phase N+1
THEN SHALL require:
- all gate-linked F probes passed with artifacts
- contract probe regressions = zero for promoted requirements in-scope
- unresolved risks explicitly carried with owner + next probe.

## Open Contract Backlog (still unresolved)
### OQ-C01: EntityKey Scope Depth Policy
Owner: `rust-llm-core-foundation` + `rust-llm-tree-extractor`
- full extraction now vs best-effort with explicit capability flags.

### OQ-C02: External Entity File Path Marker
Owner: `rust-llm-core-foundation`
- canonical external marker strategy (e.g., `EXTERNAL:{package}`).

### OQ-C03: Discriminator Fallback Finalization
Owner: `rust-llm-core-foundation`
- finalize `ParamTypes -> Index -> ContentHash` transition rules.

### OQ-C04: rust-analyzer Pin Strategy
Owner: `rust-llm-rust-semantics`
- lock launch version and upgrade cadence policy.

### OQ-C05: Rule-Set MVP Boundary
Owner: `rust-llm-graph-reasoning`
- confirm MVP required rule subset and deferred rule backlog.

### OQ-C06: MCP/HTTP First-Ship Sequence
Owner: `rust-llm-interface-gateway`
- reconcile release order while preserving cross-transport parity contract.

### OQ-C07: User Journey Contract Completeness
Owner: product/contracts
- `ES-V200-User-Journey-01.md` sections 2/3/4 are placeholders and must be converted into executable journey contracts.

## Standard Contract Template (for all future additions)
```text
Contract ID:
Owner:
Source(s):

Preconditions:
- ...

Contract:
WHEN ...
THEN SHALL ...
AND SHALL ...
AND SHALL NOT ...

Error conditions:
- ...

Probe(s):
- ...

TDD sequence:
- STUB:
- RED:
- GREEN:
- REFACTOR:
```
