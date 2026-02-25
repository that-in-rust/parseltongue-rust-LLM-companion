# v200-arch-patterns-01
Status: Working reference
Date: 2026-02-25
Source studied: `CR09/codemogger`

## Pattern We Should Definitely Learn
### 1) Initial full scan + debounced incremental reindex loop
Score: **96/100**

Why this is strong:
- Prevents blind spots at startup (full baseline scan first).
- Keeps steady-state cheap (changed files only).
- Handles rapid edits without thrashing (debounce).
- Fits Big-Rock-01 truth loop and Big-Rock-02 freshness needs.

Implementation anchors in current repo:
- `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`
- `crates/pt08-http-code-query-server/src/file_watcher_integration_service.rs`
- `crates/pt08-http-code-query-server/src/incremental_reindex_core_logic.rs`

## What Else From codemogger We Should Consider
### 1) Hash-ledger + stale-file cleanup
Score: **90/100**
- Keep per-file hash cache and remove records for deleted files.
- Reduces wasted parse/insert work and keeps storage honest.
- Reference: `CR09/codemogger/src/db/store.ts`

### 2) Large-node AST splitting
Score: **88/100**
- Split oversized class/module/impl nodes into smaller retrievable units.
- Improves search quality and context packing granularity.
- Reference: `CR09/codemogger/src/chunk/treesitter.ts`

### 3) Query preprocessing for fuzzy asks
Score: **84/100**
- Strip stopwords/noise and preserve discriminative tokens.
- Helps keyword mode when prompt is conversational.
- Reference: `CR09/codemogger/src/search/query.ts`

### 4) Hybrid rank fusion (keyword + semantic)
Score: **86/100**
- Combine lexical and semantic rankings using RRF.
- Good candidate for "evidence search" lane, not canonical truth lane.
- Reference: `CR09/codemogger/src/search/rank.ts`

### 5) Embedder abstraction boundary
Score: **91/100**
- Keep embedding model behind trait/interface.
- Lets us swap local model/provider without core contract churn.
- Reference: `CR09/codemogger/src/index.ts`, `CR09/codemogger/src/embed/types.ts`

### 6) Batched pipeline phases (chunk -> store -> embed)
Score: **87/100**
- Better throughput and predictable memory profile.
- Cleanly separates parse/store/embedding steps.
- Reference: `CR09/codemogger/src/index.ts`

### 7) Searchability self-check guard
Score: **79/100**
- Detects "DB exists but not actually searchable" states early.
- Useful operational guardrail (locks/corruption/missing WAL-like issues).
- Reference: `CR09/codemogger/src/index.ts` (`verifySearchable`)

### 8) MCP ergonomics (`index`, `search`, `reindex`)
Score: **82/100**
- Good tool surface pattern for agent workflows.
- Dynamic tool descriptions by current project state are useful.
- Reference: `CR09/codemogger/src/mcp.ts`

## What Not To Adopt As Canonical Core
### 1) Line-span primary identity keys
Score: **18/100** (for canonical identity)
- `file:start:end` shifts on edits and is not stable enough for truth graph keys.
- Keep canonical EntityKey model in Parseltongue.
- Reference: `CR09/codemogger/src/chunk/types.ts`

### 2) Simplified ignore semantics
Score: **30/100** (for BR01 truth contract)
- Hidden-file skip and simplified ignore parsing can miss files.
- BR01 requires explicit full accountability of discovered files.
- Reference: `CR09/codemogger/src/scan/walker.ts`

## Proposed Adoption Strategy
1. Adopt patterns 1, 2, 5, 6 directly into canonical ingestion architecture.
2. Adopt patterns 3, 4, 8 inside optional evidence-search lane.
3. Explicitly reject line-span identity and simplified ignore behavior for canonical mode.
4. Keep Big-Rock-01 source of truth inside Parseltongue graph contracts.

---

## 90+ Patterns We Can Run Today (From Our Own `crates/`)
Status: Implemented and validated in local runs
Date: 2026-02-25

### 1) Initial full scan + incremental watcher handoff
Score: **97/100**

Why this is important:
- Solves the startup blind spot: watcher-only systems miss existing files until a change happens.
- Gives deterministic baseline state before real-time deltas start.
- Keeps Big-Rock-01 truthful because "what exists now" is captured first, then maintained.

How we can use it:
- Keep this as canonical ingest runtime for long-lived server mode.
- Reuse same pattern in future MCP/desktop runtime: `initial_scan -> watch_loop`.
- Treat scan failure as degraded mode, not fatal crash, so users still get service.

Code anchor snippet:
```rust
// http_server_startup_runner.rs
if !state.loaded_from_snapshot_flag && !db_path.is_empty() && db_path != "mem" {
    match crate::initial_scan::execute_initial_codebase_scan(watch_dir, &state).await {
        Ok(stats) => { /* baseline indexed */ }
        Err(e) => { println!("Continuing with file watcher (incremental only)"); }
    }
}
```

Primary references:
- `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`

### 2) Debounced per-file coalescing (last-write-wins)
Score: **96/100**

Why this is important:
- Editors emit bursty save/rename/metadata events.
- Without coalescing, reindex storms create noisy writes and stale intermediate states.
- Per-file timestamp map gives deterministic "only newest event is processed."

How we can use it:
- Keep this as required guardrail for all live file ingestion.
- Extend to batch-by-folder in future if monorepo event rates grow.
- Keep extension filter at callback boundary to avoid useless downstream work.

Code anchor snippet:
```rust
// file_watcher_integration_service.rs
map.insert(file_path.clone(), event_time);
tokio::time::sleep(Duration::from_millis(debounce_ms)).await;

let should_process = {
    let map = pending_changes.read().await;
    map.get(&file_path).is_some_and(|&recorded_time| recorded_time == event_time)
};
```

Primary references:
- `crates/pt08-http-code-query-server/src/file_watcher_integration_service.rs`
- `crates/pt01-folder-to-cozodb-streamer/src/file_watcher_tests.rs`

### 3) Hash-cache early return for unchanged files
Score: **95/100**

Why this is important:
- Prevents expensive parse/delete/reinsert cycles when bytes are identical.
- Dramatically reduces churn from noisy filesystem events.
- Makes watcher mode scalable across large repos.

How we can use it:
- Keep hash cache as mandatory pre-check in incremental path.
- Expose cache-hit metrics in observability to prove cost savings.
- Use same hash gate for any future queued async reindex jobs.

Code anchor snippet:
```rust
// incremental_reindex_core_logic.rs
let current_hash = compute_content_hash_sha256(&file_content);
let cached_hash = storage.get_cached_file_hash_value(file_path_string).await.ok().flatten();

if cached_hash.as_ref() == Some(&current_hash) {
    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    return Ok(IncrementalReindexResultData {
        file_path: file_path_string.to_string(),
        entities_before: 0,
        entities_after: 0,
        hash_changed: false,
        entities_added: 0,
        entities_removed: 0,
        edges_added: 0,
        edges_removed: 0,
        processing_time_ms,
    });
}
```

Primary references:
- `crates/pt08-http-code-query-server/src/incremental_reindex_core_logic.rs`
- `crates/pt08-http-code-query-server/tests/e2e_incremental_reindex_isgl1v2_tests.rs`

### 4) Stable entity identity matching (hash -> position -> new)
Score: **95/100**

Why this is important:
- Preserves identity across edits; avoids false deletes/recreates.
- Makes blast radius and history reasoning trustworthy over time.
- Handles both unchanged code and nearby line-shift edits.

How we can use it:
- Keep this matching strategy as default identity retention policy.
- Use tolerance window conservatively; tune per language only with evidence.
- Store match reason (`ContentMatch`/`PositionMatch`) for explainability in future UX.

Code anchor snippet:
```rust
// isgl1_v2.rs
if let Some(matched) = old_entities.iter().find(|old| {
    old.content_hash == new_entity.content_hash
        && old.name == new_entity.name
        && old.file_path == new_entity.file_path
}) {
    return EntityMatchResult::ContentMatch { old_key: matched.key.clone() };
}
```

Primary references:
- `crates/parseltongue-core/src/isgl1_v2.rs`
- `crates/parseltongue-core/tests/isgl1_v2_entity_matching_tests.rs`

### 5) Watcher service lifetime retention in app state
Score: **94/100**

Why this is important:
- Fixes a subtle runtime bug: watcher can die when startup scope exits.
- Prevents "server looks healthy but indexing silently stopped" failure mode.
- Turns background ingest into a reliable long-lived service.

How we can use it:
- Keep this pattern mandatory for every background task/service handle.
- Apply same lifecycle ownership pattern to future queue workers, schedulers, and stream consumers.
- Add status endpoint checks to detect dropped services quickly.

Code anchor snippet:
```rust
// http_server_startup_runner.rs
let mut service_arc = state.watcher_service_instance_arc.write().await;
*service_arc = Some(watcher_service); // retain for server lifetime
```

Primary references:
- `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`
- `crates/pt08-http-code-query-server/tests/watcher_service_lifetime_test.rs`

### 6) Ingestion observability and accountability report
Score: **93/100**

Why this is important:
- Gives explicit answer to "what was eligible, parsed, unparsed, and why."
- Makes BR01 file-accountability visible instead of assumed.
- Converts ingestion quality into measurable numbers and error logs.

How we can use it:
- Keep endpoint as default post-ingest health check.
- Feed summary into future FUJ setup flow so users see gaps early.
- Use unparsed file list as action queue for parser capability roadmap.

Code anchor snippet:
```rust
// ingestion_coverage_folder_handler.rs
let (all_files, eligible_files, walk_errors) = walk_directory_collect_files_and_errors(".");
let parsed_files = query_parsed_file_paths_from_database(&storage).await?;
let unparsed_files = compute_unparsed_files_list(&eligible_files, &parsed_files);
```

Primary references:
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`

### 7) Multi-algorithm graph analysis suite on shared graph
Score: **92/100**

Why this is important:
- Lets us reuse one graph for many high-value analyses (SCC, centrality, entropy, community).
- Creates compound insight: each algorithm explains different risk dimensions.
- Gives immediate BR02 launchpad once BR01 ingestion truth is stable.

How we can use it:
- Keep algorithms modular but run on the same graph contract.
- Add cross-check tests whenever a new graph algorithm is introduced.
- Use result union in ranking APIs rather than one-metric decisions.

Code anchor snippet:
```rust
// integration_cross_algorithm_tests.rs
let sccs = tarjan_strongly_connected_components(&graph);
let core_numbers = kcore_decomposition_layering_algorithm(&graph);
let pagerank = compute_pagerank_centrality_scores(&graph, 0.85, 100, 1e-6);
let entropy = compute_all_entity_entropy(&graph);
let (communities, _modularity) = leiden_community_detection_clustering(&graph, 1.0, 100);
```

Primary references:
- `crates/parseltongue-core/src/graph_analysis/integration_cross_algorithm_tests.rs`
- `crates/parseltongue-core/src/graph_analysis/*`

### 8) Thread-local parser/extractor caches for parallel ingest
Score: **91/100**

Why this is important:
- Tree-sitter parser is not cheap to repeatedly construct.
- Shared mutex parser is a throughput bottleneck under Rayon concurrency.
- Thread-local cache removes lock contention and improves ingest speed predictability.

How we can use it:
- Keep this as default parallel parsing strategy.
- Extend thread-local cache pattern to other expensive per-language components.
- Track per-thread parse counts in diagnostics for tuning.

Code anchor snippet:
```rust
thread_local! {
    static THREAD_PARSERS: std::cell::RefCell<HashMap<Language, Parser>> =
        std::cell::RefCell::new(HashMap::new());
}
```

Primary references:
- `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs`
- `crates/parseltongue/src/main.rs`

## Validation Notes (What Passed Today)
- `cargo test -p pt01-folder-to-cozodb-streamer file_watcher -- --nocapture` -> 20 passed
- `cargo test -p pt08-http-code-query-server --test watcher_service_lifetime_test -- --nocapture` -> 4 passed
- `cargo test -p pt08-http-code-query-server --lib incremental_reindex_core_logic -- --nocapture` -> 3 passed
- `cargo test -p parseltongue-core test_all_seven_algorithms_run_on_eight_node_graph -- --nocapture` -> 1 passed
- `cargo test -p parseltongue-core --test isgl1_v2_entity_matching_tests -- --nocapture` -> 5 passed
- `cargo test -p pt02-folder-to-ram-snapshot -- --nocapture` -> 1 passed

## Known Caveat
- Full `pt08` E2E suite currently has a test harness signature mismatch:
  `build_complete_router_instance(state, mode)` now takes two args, while some E2E tests still call one-arg form.
- This does not invalidate the above patterns; it blocks only full-suite compile in those test targets.
