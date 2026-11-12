# native-namespace-memory-findings

## 2026-11-12 — measured negative knowledge

- Failure: original DuckDB cap error. Cause established only to candidate
  INSERT, not a particular join/allocation. Signal: separate copy and
  normalization succeed; round-2/3 candidate insertion and isolated INSERT
  EXPLAIN ANALYZE fail at 1288490188 bytes. Safer replacement to test:
  request-local bounded batches. Proof gate: unchanged full binding/path
  exports and completed native fixture, not just successful SELECT profiling.
- Rejected immediate causes: copy, normalization, scope-cycle validation.
  Runtime completion lines precede the path INSERT failure. This does not
  prove they have no effect on residency within a longer-lived connection.
- Rejected claim: a near-cap-sized ART index alone explains the failure.
  Pre-INSERT ART samples are 87–115 MB; SELECT body alone peaks near 1.2 GiB.
  Index contribution remains possible and unquantified.
- Trap: retained data plus spill caused ENOSPC before memory diagnosis.
  Signal: literal No space left on device; replacement: reclaim reproducible
  debug build cache, never retained inputs. Gate: replay reproduces actual OOM.
- Trap: SELECT EXPLAIN success does not establish INSERT success. Body
  produces 62554 rows; rollback-safe INSERT EXPLAIN still OOMs. All CSV
  operator counts are labeled SELECT-only. No full INSERT counts available.

## Incremental source ledger

Authority: local pinned source and measured logs, 2026-11-12.

- `crates/parseltongue-duckdb-analysis-adapter/src/namespace-frozen-resolution-rounds.rs:29–88`:
  actual resolver dispatch, preprocessing and round order.
- `crates/parseltongue-duckdb-analysis-adapter/src/namespace-path-candidate-records.rs:23–56`:
  generation/attempt filter, candidate PK, scoped DELETE, immediate memory
  sample, production INSERT and commit. Source alone does not prove runtime
  failure location; the independent INSERT replay supplies that link.
- `crates/parseltongue-duckdb-analysis-adapter/src/namespace-path-candidate-derivation.sql`:
  recursive derivation and request-key-partitioned windows; basis for batching
  hypothesis, not yet proof of a passing implementation.
- `hld-current-test-runs/20261112-namespace-phase-blocker/failing-query-operator-cardinalities.csv`:
  all 133 measured SELECT operator row counts/volumes. Not heap stacks.
- Artifact folder `/home/ubuntu/hld-current-test-artifacts/20261112-namespace-phase-probes/`:
  original input, sampled tree RSS, JSON/ASCII EXPLAIN, SQL, identity digests;
  journal/run report distinguishes cold query from live iteration state.

## Cheaper-hypothesis ledger — REQ-RUST-024.7

- Failure: explicit insertion-order false and one worker both still OOM.
  Detection: runtime false/2 versus false/1, same frozen INSERT and cap,
  rollback, 5.354s versus 9.034s. Cause not inferred from generic OOM hints.
  Safer replacement: keep baseline settings; test query shape instead.
- Failure: unindexed CTAS OOMs with target DELETE before it (5.926s CTAS)
  and also with DELETE postponed until after it (4.586s CTAS). Neither
  target INSERT nor DELETE has to execute for the latter failure.
  Detection: separate operation-0 logs plus a RED/GREEN test requiring
  original source rows to remain readable during materialization.
  Proof gate: a narrower SELECT/CTAS must finish before claiming a fix.
- Failure: recursive DISTINCT, unused lookahead removal and full-state
  USING KEY all OOM. Three failed H4 rewrites: stop, no more tuning here.
  Sparse keys can erase ambiguity; integer-ID state remains untested.
  Safer next cut: exact dictionaries including defining scope text and
  visibility markers, text rejoin, equality against current small fixtures.
- Measured allocator attribution: peak-time heaptrack stacks show
  normalization 1096399504 bytes via StandardBufferManager and 802144141
  via non-managed C++ BaseAppender stacks. Copy equivalents 1112667672
  and 142303607. Do not double-count outer Turso visitor stack frames.
  Residual RSS is unassigned; profiler RSS is not native qualification.
- Source ledger: native source-codec normalizer (fourteen appenders);
  namespace-path-candidate-derivation.sql (six strings/two integers, no list);
  hypothesis logs, peak-time full stacks and exclusive category JSON under
  home artifacts/20261112-namespace-phase-probes/namespace-insertion-hypothesis-results.
  Frozen input SHA256 remains e5e03f0e13e0aeb3a2abe17be43a941b982d4cb120432da8704959d16448b224.
