# Prep-v200-Hashing-Risks-v01
Status: Draft v01
Purpose: Make V200 clean-room crate risks measurable by documenting public interfaces, dependency edges, and a repeatable rubber-debug loop that reduces Risk/Unclear scores with evidence.
Method reference: `docs/Prep-v200-Dependency-Graph-Contract-Hardening.md`

## Scope
- Clean-room V200 only (no reuse from `pt*` crates).
- 9-crate architecture:
  - `rust-llm-interface-gateway`
  - `rust-llm-core-foundation`
  - `rust-llm-tree-extractor`
  - `rust-llm-rust-semantics`
  - `rust-llm-cross-boundaries`
  - `rust-llm-graph-reasoning`
  - `rust-llm-store-runtime`
  - `rust-llm-context-packer`
  - `rust-llm-test-harness`

## Control-Flow Start and Primary Divergence
- Control flow begins in `rust-llm-interface-gateway/src/main.rs`.
- First divergence from existing parseltongue is immediate at dispatch:
  - New flow routes only into `rust-llm-*` crates.
  - No dependency on `pt01`, `pt08`, or other `pt*` crates.

## Mermaid: Public Interface Dependency Graph (v01)
```mermaid
flowchart TD
    GW["rust-llm-interface-gateway
start_ingest_flow_pipeline(IngestRequest) -> IngestReport
start_query_flow_pipeline(QueryRequest) -> QueryResponse
start_delta_flow_pipeline(DeltaRequest) -> DeltaReport"]

    CF["rust-llm-core-foundation
build_entity_key_string(EntityIdentityInput) -> EntityKey
parse_entity_key_string(EntityKey) -> EntityIdentityView
verify_entity_key_roundtrip(EntityKeyRoundtripInput) -> RoundtripValidationReport
validate_fact_contract_rules(FactBatch) -> ContractValidationReport"]

    TE["rust-llm-tree-extractor
extract_syntax_fact_batch(ParseBatchRequest) -> SyntaxFactBatch
extract_dependency_edge_batch(ParseBatchRequest) -> DependencyEdgeBatch"]

    RS["rust-llm-rust-semantics
extract_resolved_rust_semantics(RustSemanticRequest) -> RustSemanticBatch
resolve_proc_macro_expansions(ProcMacroRequest) -> ProcMacroBatch
resolve_build_script_artifacts(BuildScriptRequest) -> BuildScriptBatch
annotate_semantic_degrade_reasons(SemanticDegradeInput) -> SemanticDegradeReport
verify_rust_semantic_capabilities(CapabilityRequest) -> CapabilityReport"]

    CB["rust-llm-cross-boundaries
detect_cross_language_links(BoundaryDetectionRequest) -> CrossLanguageEdgeBatch"]

    GR["rust-llm-graph-reasoning
derive_security_taint_flows(ReasoningRequest) -> TaintFlowBatch
derive_policy_violation_set(ReasoningRequest) -> PolicyViolationBatch
derive_context_priority_ranking(ReasoningRequest) -> ContextRankingBatch"]

    SR["rust-llm-store-runtime
commit_fact_batch_atomic(StoreCommitRequest) -> StoreCommitReport
query_graph_view_slice(GraphQueryRequest) -> GraphQueryResult
apply_delta_update_batch(DeltaUpdateRequest) -> DeltaUpdateReport
load_store_snapshot_state(SnapshotLoadRequest) -> SnapshotLoadReport
verify_store_consistency_state(ConsistencyCheckRequest) -> ConsistencyCheckReport"]

    CP["rust-llm-context-packer
pack_tagged_context_payload(ContextPackRequest) -> ContextPackResponse
enforce_token_budget_limits(BudgetRequest) -> BudgetDecision"]

    TH["rust-llm-test-harness
run_contract_test_suite(TestSuiteRequest) -> TestSuiteReport
run_scale_risk_probes(ProbeRequest) -> ProbeReport"]

    GW -- ingest --> TE
    GW -- rust-only enrich --> RS
    GW -- query --> SR
    GW -- response build --> CP

    TE --> CB
    RS --> CB
    CB --> GR
    GR --> SR
    SR --> CP

    CF -. shared contracts .-> GW
    CF -. shared contracts .-> TE
    CF -. shared contracts .-> RS
    CF -. shared contracts .-> CB
    CF -. shared contracts .-> GR
    CF -. shared contracts .-> SR
    CF -. shared contracts .-> CP
    CF -. shared contracts .-> TH

    TH -. contract gates .-> GW
    TH -. contract gates .-> TE
    TH -. contract gates .-> RS
    TH -. contract gates .-> CB
    TH -. contract gates .-> GR
    TH -. contract gates .-> SR
    TH -. contract gates .-> CP
```

## Public Interface Snapshot (ELI5)
```text
+----+------------------------------+--------------------------------------------+-------------------------------+--------------------------------------+
| #  | Crate                        | Main public interface                      | Input                         | Output                               |
+----+------------------------------+--------------------------------------------+-------------------------------+--------------------------------------+
| 0  | rust-llm-interface-gateway   | start_ingest/query/delta_flow_pipeline     | CLI/HTTP/MCP request DTOs     | Ingest/report/query response DTOs    |
| 1  | rust-llm-core-foundation     | build/parse/verify keys + contract checks  | Identity/fact batches         | Stable keys + validation reports     |
| 2  | rust-llm-tree-extractor      | extract_syntax_fact_batch                  | File set + language parsers   | Syntax facts + dependency edges      |
| 3  | rust-llm-rust-semantics      | semantics + proc/build + degrade metadata  | Cargo workspace + RA config   | Resolved facts + degrade annotations |
| 4  | rust-llm-cross-boundaries    | detect_cross_language_links                | Syntax+semantic fact batches  | Boundary edges + confidence scores   |
| 5  | rust-llm-graph-reasoning     | derive_taint/policy/context rankings       | Facts + edges + constraints   | Derived findings and priorities      |
| 6  | rust-llm-store-runtime       | commit/query/delta/snapshot/consistency    | Fact+edge batches / queries   | Persisted graph + bounded result set |
| 7  | rust-llm-context-packer      | pack_tagged_context_payload                | Query slices + token limits   | Tagged/token-bounded LLM payload     |
| 8  | rust-llm-test-harness        | run_contract_test_suite                    | Suite/probe definitions       | Pass/fail + risk probe artifacts     |
+----+------------------------------+--------------------------------------------+-------------------------------+--------------------------------------+
```

## Baseline Risk/Unclear Matrix (v01)
```text
+----+------------------------------+-----------+-------------+----------------------------------------------------------+
| #  | Crate                        | Risk / 5  | Unclear / 5 | Why baseline is not low                                  |
+----+------------------------------+-----------+-------------+----------------------------------------------------------+
| 0  | rust-llm-interface-gateway   | 3         | 2           | Unified behavior across CLI/HTTP/MCP + cancellation      |
| 1  | rust-llm-core-foundation     | 4         | 4           | Key model and contract stability affect all crates       |
| 2  | rust-llm-tree-extractor      | 4         | 3           | 12-language query correctness and normalization gaps      |
| 3  | rust-llm-rust-semantics      | 5         | 4           | RA/proc-macro/build-script reliability and churn         |
| 4  | rust-llm-cross-boundaries    | 4         | 4           | Heuristic linking quality and confidence calibration      |
| 5  | rust-llm-graph-reasoning     | 4         | 3           | Rule correctness/scale tradeoffs in V200 scope           |
| 6  | rust-llm-store-runtime       | 5         | 4           | Delta consistency, indexing, snapshot durability         |
| 7  | rust-llm-context-packer      | 3         | 3           | Ranking quality under strict token ceilings              |
| 8  | rust-llm-test-harness        | 3         | 3           | Fixture breadth vs CI time and anti-flakiness design     |
+----+------------------------------+-----------+-------------+----------------------------------------------------------+
```

## Rubber-Debug Loop (Step-by-Step, Repeated per Crate)
1. Freeze one interface contract.
2. List top-3 failure modes for that contract.
3. Build smallest executable probe that can falsify assumptions.
4. Record observed behavior and artifacts.
5. Update Risk/Unclear with evidence (not intuition).
6. Promote interface from `provisional` to `stable` only after passing probes.

## Per-Crate Information Collection Checklist (v01)
```text
+---------------------------+--------------------------------------------------+---------------------------------------------------------+
| Crate                     | Evidence to collect                               | Probe/output artifact                                   |
+---------------------------+--------------------------------------------------+---------------------------------------------------------+
| interface-gateway         | mode parity (CLI/HTTP/MCP), cancellation model    | request-lifecycle trace + error mapping table           |
| core-foundation           | key uniqueness + overload disambiguation          | key collision corpus + determinism report               |
| tree-extractor            | per-language capture completeness                 | fixture-to-capture diff report (12 languages)           |
| rust-semantics            | proc-macro/build-script success/degrade behavior  | RA workspace matrix with pass/fallback classifications  |
| cross-boundaries          | precision/recall on known boundary fixtures       | boundary edge confusion matrix                          |
| graph-reasoning           | rule correctness + runtime at scale               | golden-rule output + p50/p95 runtime report             |
| store-runtime             | delta correctness + crash durability              | mutation replay logs + recovery consistency checks      |
| context-packer            | token budget faithfulness + relevance ranking     | budget packing audit + human relevance spot-check       |
| test-harness              | flake rate + suite wall-clock budget             | CI stability trend and quarantine list                  |
+---------------------------+--------------------------------------------------+---------------------------------------------------------+
```

## Risk Hashing Snapshot Format (for each iteration)
Use a compact hash to track movement across iterations:
- Format: `crate:R{risk}-U{unclear}-E{evidence_count}`
- Example: `rust-llm-rust-semantics:R5-U4-E2`
- Iteration digest = sorted concatenation of all crate hashes.

This gives a quick “did uncertainty actually go down?” signal across v01, v02, v03.
## Pass Ledger
### Pass 01: `rust-llm-core-foundation` (Dependency Graph Contract Hardening)
Status: Contract freeze + hazard mapping complete. Probe execution pending.

#### 1) Contract freeze (v01)
```text
+------------------------------------------------------+-------------------------+-----------------------------------------------+
| Public interface                                     | Input                   | Output                                        |
+------------------------------------------------------+-------------------------+-----------------------------------------------+
| build_entity_key_string                              | EntityIdentityInput     | EntityKey                                     |
| parse_entity_key_string                              | EntityKey               | EntityIdentityView                            |
| verify_entity_key_roundtrip                          | EntityKeyRoundtripInput | RoundtripValidationReport                     |
| validate_fact_contract_rules                         | FactBatch               | ContractValidationReport                      |
+------------------------------------------------------+-------------------------+-----------------------------------------------+
```

#### 2) Rubber-duck dependency walk (core-foundation blast radius)
```text
- Upstream callers: gateway, tree-extractor, rust-semantics, cross-boundaries, graph-reasoning, store-runtime, context-packer, test-harness.
- Shared assumption across all callers: key is deterministic, parseable, overload-safe, and language-safe.
- If key contract breaks:
  - store-runtime index joins fail or silently merge distinct entities
  - cross-boundaries can create false link unions
  - graph-reasoning results become non-reproducible across runs
  - context-packer emits ambiguous references to LLMs
```

#### 3) Top failure modes found
```text
+----+-----------------------------------------------+--------------------------------------------------------------+----------------------+
| FM | Failure mode                                   | Why it matters                                                | Evidence status      |
+----+-----------------------------------------------+--------------------------------------------------------------+----------------------+
| 01 | Overload collision (same file + same name)     | Distinct overloaded funcs can collapse to one identity       | Confirmed legacy     |
| 02 | Delimiter/generic ambiguity                     | Coercive sanitization needed when format is not syntax-safe  | Confirmed legacy     |
| 03 | Path-derived semantic collapse                  | Logical namespace info is lost when key relies on file stem  | Confirmed design gap |
| 04 | Non-roundtrippable key components               | parse(build(x)) mismatch breaks dependency contract checks    | Unproven in v200     |
| 05 | External entity identity uncertainty            | crate/std/third-party entities may lack stable local anchors | Unproven in v200     |
+----+-----------------------------------------------+--------------------------------------------------------------+----------------------+
```

#### 4) Evidence captured this pass
```text
E01: Legacy key timestamp hash currently uses only file_path + entity_name as hash inputs,
     which is insufficient to distinguish overload signatures.
     Source: crates/parseltongue-core/src/isgl1_v2.rs (compute_birth_timestamp).

E02: Legacy key path had to add sanitize_entity_name_for_isgl1 for generic/delimiter safety,
     indicating format-level delimiter pressure rather than semantic clarity.
     Sources:
       - crates/parseltongue-core/src/isgl1_v2.rs (sanitize_entity_name_for_isgl1)
       - crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs (format_key)
```

#### 5) Probe set for next pass execution (core-foundation)
```text
+---------+----------------------------------------------+--------------------------------------------------------------+
| Probe   | Intent                                        | Pass criterion                                               |
+---------+----------------------------------------------+--------------------------------------------------------------+
| CF-P1-A | Overload collision corpus probe               | 0 collisions for overload set across Java/C++/C#/TS fixtures |
| CF-P1-B | Minor-edit stability mutation probe           | Keys unchanged for whitespace/comment-only edits             |
| CF-P1-C | Build/parse roundtrip probe                   | parse(build(identity)) == canonical identity view            |
| CF-P1-D | Delimiter-safety cross-language probe         | No escaping/sanitization needed for valid language symbols   |
| CF-P1-E | External entity identity probe                | Stable keys for std/crate/third-party references             |
+---------+----------------------------------------------+--------------------------------------------------------------+
```

#### 6) Score update for this pass
```text
- rust-llm-core-foundation remains Risk=4, Unclear=4.
- Rationale: evidence quality improved (E=2) but executable probe outcomes are pending.
```
### Pass 02: `rust-llm-store-runtime` (Dependency Graph Contract Hardening)
Status: Contract freeze + hazard mapping complete. Probe execution pending.

#### 1) Contract freeze (v01)
```text
+------------------------------------------------------+---------------------------+-----------------------------------------------+
| Public interface                                     | Input                     | Output                                        |
+------------------------------------------------------+---------------------------+-----------------------------------------------+
| commit_fact_batch_atomic                             | StoreCommitRequest        | StoreCommitReport                             |
| query_graph_view_slice                               | GraphQueryRequest         | GraphQueryResult                              |
| apply_delta_update_batch                             | DeltaUpdateRequest        | DeltaUpdateReport                             |
| load_store_snapshot_state                            | SnapshotLoadRequest       | SnapshotLoadReport                            |
| verify_store_consistency_state                       | ConsistencyCheckRequest   | ConsistencyCheckReport                        |
+------------------------------------------------------+---------------------------+-----------------------------------------------+
```

#### 2) Rubber-duck dependency walk (store-runtime blast radius)
```text
- Upstream callers: interface-gateway (ingest/query paths), graph-reasoning (derived fact commits), context-packer (slice reads), and watcher-driven delta paths.
- Shared assumption across callers: writes are atomic, reads are bounded, deltas are idempotent, and snapshot loads are contract-safe.
- If store contract breaks:
  - partial writes create entity/edge mismatch and corrupt downstream reasoning
  - unbounded query paths trigger memory spikes/OOM under scale
  - non-idempotent delta replays duplicate or orphan records
  - snapshot/schema drift produces silent data corruption
```

#### 3) Top failure modes found
```text
+----+--------------------------------------------------+--------------------------------------------------------------+----------------------+
| FM | Failure mode                                      | Why it matters                                                | Evidence status      |
+----+--------------------------------------------------+--------------------------------------------------------------+----------------------+
| 01 | Unbounded query path loads full graph             | Large-scale queries become OOM-prone                          | Confirmed legacy     |
| 02 | Non-atomic commit splits entities and edges       | Partial state leads to invalid joins and wrong analytics      | Unproven in v200     |
| 03 | Non-idempotent delta replay                       | Replayed updates can create duplicates/ghost nodes            | Unproven in v200     |
| 04 | Snapshot compatibility drift                      | Snapshot restore can silently violate runtime contract         | Confirmed design gap |
| 05 | Parse-failure delete path data loss               | Transient parse failures may delete valid historical entities | Confirmed legacy     |
+----+--------------------------------------------------+--------------------------------------------------------------+----------------------+
```

#### 4) Evidence captured this pass
```text
E03: At 1.6M edges, mem-first storage has ~3.2 GB base RAM cost before heavy queries,
     and full-scan paths add significant additional allocations.
     Source: docs/pre175/DECISION-v173-pt02-pt03-endpoint-selection.md

E04: For mem-first endpoint behavior at large scale, only 7/24 endpoints are safe on 8GB,
     while disk-backed strategy remains broadly viable.
     Source: docs/pre175/DECISION-v173-pt02-pt03-endpoint-selection.md

E05: Incremental reindex legacy logic includes a parse-failure path that deletes old
     entities and edges for the file, which is risky for transient parser instability.
     Source: crates/pt08-http-code-query-server/src/incremental_reindex_core_logic.rs

E06: Snapshot load path restores data into mem via full deserialize + bulk insert flow,
     with no explicit compatibility/consistency handshake in the load contract.
     Source: crates/pt08-http-code-query-server/src/snapshot_loader_module.rs
```

#### 5) Probe set for next pass execution (store-runtime)
```text
+---------+-----------------------------------------------------+---------------------------------------------------------------+
| Probe   | Intent                                               | Pass criterion                                                |
+---------+-----------------------------------------------------+---------------------------------------------------------------+
| SR-P2-A | Bounded query guard probe                            | Heavy queries are rejected/segmented before unsafe allocation |
| SR-P2-B | Atomic commit rollback probe                         | Mid-commit failure yields zero partial writes                 |
| SR-P2-C | Idempotent delta replay probe                        | Replay(delta) twice => identical final state hash             |
| SR-P2-D | Snapshot compatibility probe                         | Version mismatch yields explicit fail or migration path        |
| SR-P2-E | Transient parse-failure quarantine probe             | Single transient parse fail cannot hard-delete stable records  |
+---------+-----------------------------------------------------+---------------------------------------------------------------+
```

#### 6) Score update for this pass
```text
- rust-llm-store-runtime remains Risk=5, Unclear=4.
- Rationale: evidence increased (E=4), but no probe execution outcomes yet.
```
### Pass 03: `rust-llm-rust-semantics` (Dependency Graph Contract Hardening)
Status: Contract freeze + hazard mapping complete. Probe execution pending.

#### 1) Contract freeze (v01)
```text
+------------------------------------------------------+---------------------------+-----------------------------------------------+
| Public interface                                     | Input                     | Output                                        |
+------------------------------------------------------+---------------------------+-----------------------------------------------+
| extract_resolved_rust_semantics                      | RustSemanticRequest       | RustSemanticBatch                             |
| resolve_proc_macro_expansions                        | ProcMacroRequest          | ProcMacroBatch                                |
| resolve_build_script_artifacts                       | BuildScriptRequest        | BuildScriptBatch                              |
| annotate_semantic_degrade_reasons                    | SemanticDegradeInput      | SemanticDegradeReport                         |
| verify_rust_semantic_capabilities                    | CapabilityRequest         | CapabilityReport                              |
+------------------------------------------------------+---------------------------+-----------------------------------------------+
```

#### 2) Rubber-duck dependency walk (rust-semantics blast radius)
```text
- Upstream callers: interface-gateway ingest path and graph-reasoning enrichment path.
- Downstream dependents: cross-boundaries and graph-reasoning expect semantic accuracy labels.
- Shared assumption across callers: semantic facts are either resolved or explicitly marked degraded.
- If contract breaks:
  - trait/type edges can be wrong while still looking "valid"
  - proc-macro/build-script failures can silently downgrade analysis quality
  - runtime memory/time spikes can stall ingest and block downstream crates
```

#### 3) Top failure modes found
```text
+----+--------------------------------------------------+--------------------------------------------------------------+----------------------+
| FM | Failure mode                                      | Why it matters                                                | Evidence status      |
+----+--------------------------------------------------+--------------------------------------------------------------+----------------------+
| 01 | ra_ap API churn / version skew                    | Minor version drift can break compile/contracts               | Confirmed legacy     |
| 02 | Proc-macro expansion instability                  | Crashed/absent macros can hide trait impls and type facts     | Confirmed legacy     |
| 03 | Build-script variability                          | Environment/target variance can shift semantic outputs         | Confirmed design gap |
| 04 | Silent degrade without metadata                   | Downstream crates can treat unknowns as authoritative facts    | Unproven in v200     |
| 05 | Resource envelope overrun                         | Large workspaces can exceed practical memory/time budgets      | Confirmed legacy     |
+----+--------------------------------------------------+--------------------------------------------------------------+----------------------+
```

#### 4) Evidence captured this pass
```text
E07: rust-analyzer integration requires exact version pinning across all ra_ap crates;
     internal API mismatch is a known breakage vector.
     Source: docs/Prep-V200-Rust-Analyzer-API-Surface.md

E08: Proc-macro expansion has known failure paths and can fall back to unknown output;
     this can cascade into missing impl/type visibility if not surfaced explicitly.
     Source: docs/Prep-V200-Rust-Analyzer-API-Surface.md

E09: Build cost for RA path is materially higher (feature-gated) and intended to be optional;
     indicates mandatory capability negotiation at runtime.
     Source: docs/Prep-V200-Rust-Analyzer-API-Surface.md

E10: RA memory/time envelopes are non-trivial for large workspaces (2-4 GB, 10-60s range),
     requiring bounded execution policy and degrade handling.
     Source: docs/Prep-V200-Rust-Analyzer-API-Surface.md

E11: RA adds semantic classes impossible in tree-sitter alone (resolved types, trait impl
     closure, macro-expanded impls), so fallbacks must be explicit to preserve trust.
     Sources:
       - docs/Prep-V200-Rust-Analyzer-API-Surface.md
       - docs/Prep-V200-Compiled-Research-Best-Ideas.md
```

#### 5) Probe set for next pass execution (rust-semantics)
```text
+---------+-----------------------------------------------------+---------------------------------------------------------------+
| Probe   | Intent                                               | Pass criterion                                                |
+---------+-----------------------------------------------------+---------------------------------------------------------------+
| RS-P3-A | Version-pinning canary probe                         | Any ra_ap skew fails fast with explicit diagnostic             |
| RS-P3-B | Proc-macro chaos probe                               | Crash/unavailable macro yields degrade annotation, not silence |
| RS-P3-C | Build-script variance probe                          | Output delta is deterministic or flagged as non-deterministic  |
| RS-P3-D | Semantic degrade integrity probe                     | Unknown/degraded facts are tagged and excluded from strict joins|
| RS-P3-E | Resource envelope probe                              | Ingest obeys configured memory/time ceiling with graceful exit  |
+---------+-----------------------------------------------------+---------------------------------------------------------------+
```

#### 6) Score update for this pass
```text
- rust-llm-rust-semantics remains Risk=5, Unclear=4.
- Rationale: evidence increased (E=5), but probe execution outcomes are pending.
```

## Risk Hash Snapshot History
```text
Baseline:
  rust-llm-core-foundation:R4-U4-E0
  rust-llm-store-runtime:R5-U4-E0
  rust-llm-rust-semantics:R5-U4-E0

After Pass 01:
  rust-llm-core-foundation:R4-U4-E2

After Pass 02:
  rust-llm-store-runtime:R5-U4-E4

After Pass 03:
  rust-llm-rust-semantics:R5-U4-E5
```

## v01 Immediate Next Actions
1. Execute probe set CF-P1-A..E and attach artifacts.
2. Execute probe set SR-P2-A..E and attach artifacts.
3. Execute probe set RS-P3-A..E and attach artifacts.
4. Re-score `rust-llm-core-foundation`, `rust-llm-store-runtime`, and `rust-llm-rust-semantics` after probe evidence.
5. Start Pass 04 on `rust-llm-tree-extractor` and update dependency edges if contract shifts.
