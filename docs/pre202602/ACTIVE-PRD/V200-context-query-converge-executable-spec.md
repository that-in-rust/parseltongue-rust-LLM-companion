# V200 Context Query Converge - Executable Spec

Feature outcome: add one CPU-only HTTP endpoint that replaces multi-step grep-like exploration with a deterministic query -> candidate clusters -> deep dive workflow.

Actors and boundaries:
- Caller: LLM or human operator using the existing HTTP server.
- Serving crate: `pt08-http-code-query-server`.
- Data source: pre-indexed CozoDB graph plus on-demand source reads from the existing ingestion pipeline.
- Runtime constraints: no GPU, no external model API, no in-pipeline LLM reasoning.

Failure modes to cover:
- empty or whitespace query
- no matching entities
- duplicate results that anchor to the same public interface
- selected candidate not present in current result set
- snapshot mode or missing code bodies for deep context
- non-Rust or non-public entities that cannot produce verified public anchoring

Performance and reliability limits:
- `disambiguate` response p95 <= `600 ms` on a fixture with `25_000` entities and `120_000` dependency edges
- `resolved` response p95 <= `2_000 ms` on the same fixture when code bodies are available
- 0 panics for invalid request bodies, missing candidates, or empty graphs

Language/runtime constraints:
- v1 is Rust-first for verified anchoring and verified deep context
- non-Rust entities may participate only as heuristic context unless explicit verification metadata exists
- endpoint shall be exposed under `/{mode}/context-query-converge`

## Executable Requirements

### REQ-CQC-001.0: Accept stateless converge requests

**WHEN** the caller sends `POST /{mode}/context-query-converge` with JSON containing `query` and no `selected_candidate_key`
**THEN** the system SHALL treat the request as a first-pass convergence request
**AND** SHALL return one of `disambiguate` or `no_match`
**SHALL** reject empty or whitespace-only `query` with HTTP `400`

### REQ-CQC-002.0: Return at most four candidate clusters

**WHEN** a first-pass convergence request produces ranked candidate entities
**THEN** the system SHALL return at most `4` candidate clusters sorted by deterministic score descending
**AND** SHALL return fewer than `4` clusters when fewer distinct anchored candidates exist
**SHALL** deduplicate candidates that collapse to the same anchor entity

### REQ-CQC-003.0: Anchor each candidate to a public interface

**WHEN** a ranked candidate is private or otherwise not directly suitable for user presentation
**THEN** the system SHALL resolve the nearest enclosing or upward-reachable public interface for that candidate
**AND** SHALL include `anchor_key`, `anchor_name`, `anchor_kind`, `module_path`, and folder-path metadata in the returned cluster
**SHALL** mark the cluster with `trust_grade=heuristic` when verified public anchoring is unavailable

### REQ-CQC-004.0: Build preview clusters under a hard token cap

**WHEN** the endpoint returns `disambiguate`
**THEN** each candidate cluster SHALL fit within an estimated preview budget of `3000` tokens or less
**AND** SHALL include, at minimum, the anchor summary, score breakdown, hop-1 blast radius summary, and top related entities
**SHALL** omit low-priority related entities instead of exceeding the preview budget

### REQ-CQC-005.0: Resolve a selected candidate to deep context

**WHEN** the caller sends `POST /{mode}/context-query-converge` with both `query` and `selected_candidate_key`
**THEN** the system SHALL return `resolved` when the selected candidate is valid for the current query
**AND** SHALL include anchor code context, related entities, and a deeper graph slice for the selected cluster
**SHALL** cap estimated deep-context output at `20_000` tokens

### REQ-CQC-006.0: Expose control-flow and data-flow sections honestly

**WHEN** the endpoint returns `resolved`
**THEN** the response SHALL contain separate sections for `control_flow`, `data_flow`, and `compiler_context`
**AND** SHALL populate each section only when the underlying graph or compiler evidence exists
**SHALL** mark unavailable sections explicitly instead of fabricating inferred detail

### REQ-CQC-007.0: Preserve deterministic state transitions

**WHEN** the same indexed graph receives the same request body twice
**THEN** the endpoint SHALL return the same state and the same candidate ordering
**AND** SHALL use stable tie-breaking based on canonical entity key when scores match
**SHALL** avoid server-side session dependency for basic request correctness

### REQ-CQC-008.0: Reuse service logic outside handler modules

**WHEN** the feature is implemented in `pt08-http-code-query-server`
**THEN** ranking, anchoring, cluster packing, and deep-packet construction SHALL live in reusable non-handler modules
**AND** SHALL not depend on one HTTP handler calling another HTTP handler
**SHALL** keep the public route integration isolated to a new handler module

### REQ-CQC-009.0: Operate without external model services

**WHEN** the server starts and processes converge requests
**THEN** the feature SHALL work with no GPU dependency and no external LLM or embedding API credentials
**AND** SHALL rely only on local graph data, local source reads, and in-process scoring logic
**SHALL** fail only on local data availability or request validity issues, not on network reachability to a model provider

### REQ-CQC-010.0: Degrade safely in snapshot or partial-data mode

**WHEN** the server runs in snapshot mode or code bodies are unavailable
**THEN** the endpoint SHALL still support `disambiguate` when graph metadata is present
**AND** SHALL downgrade deep context sections that require code bodies or compiler evidence
**SHALL** return explicit degradation metadata instead of HTTP `500`

### REQ-CQC-011.0: Meet measurable latency targets

**WHEN** performance tests run on the converge fixture with `25_000` entities and `120_000` edges
**THEN** first-pass `disambiguate` SHALL complete with p95 latency <= `600 ms`
**AND** selected-candidate `resolved` SHALL complete with p95 latency <= `2_000 ms`
**SHALL** record p99 and median timings in benchmark output for regression tracking

### REQ-CQC-012.0: Register one route and one response contract

**WHEN** the route builder is initialized
**THEN** the server SHALL expose exactly one new feature route for this workflow: `POST /{mode}/context-query-converge`
**AND** SHALL use a single response envelope containing `state`, `request_echo`, `trust_grade`, and `data`
**SHALL** keep follow-up selection on the same route instead of adding separate pick or resolve endpoints

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-CQC-001.0 | TEST-UNIT-CQC-001 | unit | empty query returns HTTP 400 with typed error payload | validation |
| REQ-CQC-001.0 | TEST-INTEG-CQC-001 | integration | first-pass JSON request returns `disambiguate` or `no_match` only | contract |
| REQ-CQC-002.0 | TEST-UNIT-CQC-002 | unit | ranked candidates are capped at four after dedupe | ranking |
| REQ-CQC-002.0 | TEST-INTEG-CQC-002 | integration | duplicate raw hits that share one anchor appear once in response | correctness |
| REQ-CQC-003.0 | TEST-UNIT-CQC-003 | unit | private Rust function resolves to expected public anchor | anchoring |
| REQ-CQC-003.0 | TEST-INTEG-CQC-003 | integration | non-verifiable anchor is returned with `trust_grade=heuristic` | honesty |
| REQ-CQC-004.0 | TEST-UNIT-CQC-004 | unit | preview packet builder never exceeds 3000 estimated tokens | token budget |
| REQ-CQC-004.0 | TEST-INTEG-CQC-004 | integration | `disambiguate` response includes anchor summary plus hop-1 blast radius metadata | response shape |
| REQ-CQC-005.0 | TEST-INTEG-CQC-005 | integration | valid `selected_candidate_key` returns `resolved` with deep packet | resolution |
| REQ-CQC-005.0 | TEST-UNIT-CQC-005 | unit | invalid `selected_candidate_key` returns HTTP 400 or `no_match` without panic | robustness |
| REQ-CQC-006.0 | TEST-INTEG-CQC-006 | integration | unavailable flow sections are marked unavailable, not fabricated | trust |
| REQ-CQC-007.0 | TEST-UNIT-CQC-006 | unit | repeated identical requests produce identical ordering and tie-break behavior | determinism |
| REQ-CQC-008.0 | TEST-UNIT-CQC-007 | unit | handler delegates to service-layer modules instead of calling other handlers | architecture |
| REQ-CQC-009.0 | TEST-INTEG-CQC-007 | integration | endpoint succeeds with no model-provider environment variables configured | offline operation |
| REQ-CQC-010.0 | TEST-INTEG-CQC-008 | integration | snapshot mode returns degraded-but-valid payload instead of 500 | degradation |
| REQ-CQC-011.0 | TEST-PERF-CQC-001 | performance | p95 `disambiguate` latency <= 600 ms on converge fixture | latency |
| REQ-CQC-011.0 | TEST-PERF-CQC-002 | performance | p95 `resolved` latency <= 2000 ms on converge fixture | latency |
| REQ-CQC-012.0 | TEST-INTEG-CQC-009 | integration | route exists at `/{mode}/context-query-converge` and uses one envelope schema | API consistency |

## TDD Plan

### STUB

1. Add failing request/response contract tests for `POST /{mode}/context-query-converge` in `crates/pt08-http-code-query-server/tests/`.
2. Add unit tests for:
   - candidate capping
   - anchor deduplication
   - preview token budgeting
   - deterministic ordering
3. Add a dedicated fixture for convergence flows, preferably a small Rust auth-style graph and one synthetic large-graph benchmark fixture.
4. Define new modules with four-word names:
   - `context_query_converge_handler`
   - `context_query_converge_service`
   - `public_anchor_resolution_module`
   - `candidate_cluster_builder_module`
   - `deep_context_packet_module`

### RED

1. Register the new route in `route_definition_builder_module` with no implementation and confirm route tests fail.
2. Run targeted tests and capture expected failures:
   - missing route
   - missing request/response types
   - missing anchor resolver
   - missing packet builder
3. Confirm snapshot-mode degradation tests fail for the right reason rather than with panics.

### GREEN

1. Implement stateless request parsing and response envelope.
2. Extract reusable search/ranking logic out of handler-local functions where needed.
3. Implement public-anchor resolution for Rust-first verified flows and heuristic fallback for non-verifiable cases.
4. Implement preview packet packing with hard 3000-token cap.
5. Implement selected-candidate deep packet packing with hard 20,000-token cap.
6. Add degradation metadata for snapshot mode and missing code-body scenarios.

### REFACTOR

1. Remove duplicated graph traversal logic now spread across fuzzy search, blast radius, semantic cluster, and smart context code paths.
2. Normalize score breakdown fields and token estimation helpers into shared modules.
3. Keep all new public code symbols on four-word naming.
4. Tighten trust-grade semantics:
   - `verified`
   - `graph_verified`
   - `heuristic`
   - `degraded`

### VERIFY

1. Run `cargo test -p pt08-http-code-query-server`.
2. Run `cargo test -p parseltongue-core`.
3. Run full workspace tests if targeted crates pass.
4. Run performance benchmarks for converge fixture and store p50/p95/p99 numbers.
5. Verify the new route works in both `/db` and `/mem` mode semantics where supported.

## Quality Gates

### Pre-Commit Quality Gates

- [ ] Every requirement has a stable `REQ-CQC-*` ID.
- [ ] Every `REQ-CQC-*` ID has at least one linked test.
- [ ] `POST /{mode}/context-query-converge` is the only new workflow route added for this feature.
- [ ] Build passes with `cargo build --release`.
- [ ] Targeted tests pass for `pt08-http-code-query-server` and `parseltongue-core`.
- [ ] No `TODO`, `STUB`, or `FIXME` is introduced in new code.
- [ ] No new public symbol violates four-word naming.
- [ ] Performance claims are backed by benchmark output with p50/p95/p99 numbers.
- [ ] Snapshot-mode degradation is explicit and covered by tests.
- [ ] Response payload exposes trust grade rather than implying unsupported certainty.

## Open Questions

1. Should first-pass ranking use only existing graph/search signals in v1, or is a git-recency lane required before the feature is useful?
2. What is the canonical preview cluster ID: anchor entity key only, or anchor key plus ranking hash?
3. Should `selected_candidate_key` be validated only against current request results, or accepted if it exists globally in the graph?
4. Do we want deep packets to include raw code snippets by default, or only metadata pointers plus explicit follow-up reads in snapshot mode?
5. For non-Rust languages, should v1 return heuristic anchor clusters at all, or hide them until tiered language support is explicitly shipped?
6. Does `resolved` need compiler-context sections immediately, or should v1 ship graph-only deep packets and reserve compiler evidence for a second slice?
