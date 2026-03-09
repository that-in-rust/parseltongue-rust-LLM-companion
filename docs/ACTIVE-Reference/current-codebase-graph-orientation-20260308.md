# Current Codebase Graph Orientation - 2026-03-08

This note captures the current Parseltongue 1.7.2 graph export for the repo so we can reference the actual indexed graph while deciding the next architecture.

## Artifact Paths

- Run folder:
  `/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/.pt172-runs/parseltongue-1.7.2-20260308-181954`
- Actual indexed workspace folder:
  `/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/parseltongue20260308182014`
- Combined export JSON:
  `/tmp/pt172-current-graph-20260308/current-codebase-dependency-graph-export.json`
- Raw query export directory:
  `/tmp/pt172-current-graph-20260308`

## Important Runtime Note

Parseltongue `1.7.2` did not place the real index into the run-folder database path during setup.
The actual graph was written to the timestamped workspace folder:

`/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/parseltongue20260308182014/analysis.db`

So any later reads should use the workspace database, not:

`/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/.pt172-runs/parseltongue-1.7.2-20260308-181954/db/codegraph.db`

## Headline Graph Facts

- Code entities: `1701`
- Dependency edges: `10810`
- Languages detected: `12`
- Languages:
  - `rust`
  - `java`
  - `csharp`
  - `typescript`
  - `javascript`
  - `python`
  - `ruby`
  - `php`
  - `swift`
  - `cpp`
  - `go`
  - `c`

## What The Graph Currently Represents

The export is not only the production Rust crates.
It also heavily includes fixture and reference material.

Top entity buckets:

- `unresolved-reference`: `879`
- `test-fixtures-preV200`: `366`
- `crates/pt08-http-code-query-server`: `255`
- `crates/parseltongue-core`: `83`
- `other`: `61`
- `tests-preV200`: `31`
- `crates/pt01-folder-to-cozodb-streamer`: `19`
- `crates/parseltongue`: `7`

This means the current total graph is useful as a baseline map, but not yet clean enough to treat as high-trust application-only architecture.

## Quality Caveats

- Unresolved-reference entities: `879 / 1701` (`51.67%`)
- Edges touching unresolved references: `6893 / 10810` (`63.77%`)
- Resolved-only edges: `3917`

Operational meaning:

- graph algorithms run
- counts are real
- cross-language breadth is visible
- but many rankings are polluted by unresolved or placeholder nodes

So this export is good for:

- seeing overall graph shape
- seeing where current indexing is noisy
- comparing crate-level and file-level density
- identifying what needs canonicalization next

It is not yet good for:

- trustworthy context convergence
- public-interface anchoring
- clean blast-radius reasoning

## Resolved Rust Entity Distribution

Resolved Rust entities: `467`

By crate:

- `pt08-http-code-query-server`: `255`
- `parseltongue-core`: `83`
- `pt01-folder-to-cozodb-streamer`: `19`
- `parseltongue` CLI: `7`

This says the current graph is most semantically dense around the HTTP server surface, not the ingestion engine.

## Most Populated Resolved Rust Files

- `crates/parseltongue-core/src/storage/cozo_client.rs`: `51`
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/mod.rs`: `26`
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`: `20`
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_diagnostics_coverage_handler.rs`: `18`
- `crates/pt01-folder-to-cozodb-streamer/src/lib.rs`: `16`
- `crates/parseltongue-core/src/lib.rs`: `13`
- `crates/pt08-http-code-query-server/src/lib.rs`: `11`

## PT08 Surface Snapshot

Top resolved files in `pt08-http-code-query-server`:

- `src/http_endpoint_handler_modules/mod.rs`: `26`
- `src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`: `20`
- `src/http_endpoint_handler_modules/ingestion_diagnostics_coverage_handler.rs`: `18`
- `src/lib.rs`: `11`
- `src/http_endpoint_handler_modules/blast_radius_impact_handler.rs`: `10`
- `src/http_endpoint_handler_modules/technical_debt_sqale_handler.rs`: `10`
- `src/http_endpoint_handler_modules/dependency_edges_list_handler.rs`: `9`
- `src/http_endpoint_handler_modules/reverse_callers_query_graph_handler.rs`: `9`

Interpretation:

- PT08 is a large, handler-centric surface
- diagnostics and coverage logic are heavily represented
- many endpoint handlers are still graph-isolated as separate files rather than one converged workflow

## Focused Context Slice

Focused entity used for a concrete subgraph slice:

`rust:fn:handle_smart_context_token_budget:___Users_amuldotexe_Desktop_parseltongue_rust_LLM_companion_crates_pt08_http_code_query_server_src_http_endpoint_handler_modules_smart_context_token_budget_handler:T1634449276`

Relevant raw files:

- `/tmp/pt172-current-graph-20260308/forward-callees-query-graph-handle_smart_context_token_budget.json`
- `/tmp/pt172-current-graph-20260308/smart-context-token-budget-handle_smart_context_token_budget-4000.json`

What this slice shows:

- the endpoint is visible as a concrete Rust function node
- its immediate callees are dominated by unresolved-reference nodes
- even the current "smart context" endpoint often points to unresolved callees instead of canonical internal entities

That is the strongest practical signal from this export:

**the graph exists, but canonical entity resolution is still the main bottleneck**

## Working Thesis From This Export

If we are deciding what to ship next, this graph suggests:

1. do not build more graph algorithms first
2. clean canonical entity resolution first
3. reduce unresolved-reference pollution first
4. separate production crate graph from fixture/reference graph
5. only then trust cluster ranking and deep context as a product surface

## Recommendation For Next Architecture Discussion

Use the combined export JSON as the source of truth for current-state analysis:

`/tmp/pt172-current-graph-20260308/current-codebase-dependency-graph-export.json`

Use it to answer:

- which nodes are real application entities vs placeholders
- where unresolved references dominate
- which crate surfaces are dense enough to support convergence workflows
- how much current PT08 logic is reusable vs should be replaced
