# SOP — end-to-end test of `hld-current-dependency-edges` (phase-20261111)

Name: `hld-current-e2e-test-procedure` · Status: draft · Applies to: every milestone gate from M2 on, and every release
Inputs: `PRD-phase-20261111-compiling-hld.md`, `SPEC-phase-20261111-executable-requirements.md` (REQ-RUST-001 … 022)

**Rule: this SOP is run on ALL FIVE repositories, every time.** A run that skips a repository is not a pass.

```text
 the five    rust-analyzer · ripgrep · tokio · serde · cargo-modules   (PRD D2; pinned commits in SPEC §8)
 verdict     HLD current WORKS  ⇔  every step below passes on all five repositories
```

## 0. What "works" means — three independent witnesses

```text
 witness                              role                              trusted for
 ──────────────────────────────────── ───────────────────────────────── ───────────────────────────────────────────────────
 parseltongue (under test)            produces hld-current-dependency-edges + edge-evidence-record-table
 offline-teacher-rust-analyzer        ANSWER KEY (oracle)               exact targets of paths, method calls, impls
 codebase-memory-mcp (0.12.1, M)      INDEPENDENT CROSS-CHECK           a second, differently built graph: finds edges
                                                                        both other witnesses might miss, file/folder
                                                                        structure, index coverage gaps
```

codebase-memory is **not** an answer key. Measured on rust-analyzer (2026-11-05): 70,991 CALLS edges, of which 29,752
(42 %) have strategy `suffix_match` (name heuristic, confidence as low as 0.55); only `lsp_*` strategies (lsp_direct 5,336,
lsp_method_dispatch 1,236, lsp_trait_dispatch 271, lsp_ufcs 353, lsp_constructor 263, lsp_short_name_unique 7,525) are
type-aware. So the SOP uses it in two ways only: (1) structure and coverage, (2) a triage signal on disagreements —
filtered by strategy and confidence. It never overrules rust-analyzer.

## 1. Preconditions (once per repository, before any run)

```text
 P1  checkout at the pinned commit (SPEC §8); `git status` clean; commit hash written to the run report
 P2  `cargo check --workspace` passed ONCE when the corpus was chosen (outside Parseltongue; PRD §2 "workspace that compiles")
 P3  dependencies already downloaded: `cargo fetch` done earlier; the run itself is offline (no network)
 P4  toolchain recorded: `rustc -V`, `cargo -V`, `rust-analyzer --version` (oracle, M today: 1.99.1 58cf8f9),
     `codebase-memory-mcp --version`, parseltongue git hash
 P5  machine recorded: cores, RAM, OS — needed to read REQ 011 memory and the time budgets
```

## 2. The procedure — run steps S1–S10 for each repository in turn

Placeholders: `$REPO` = absolute path of the checkout, `$NAME` = short name, `$OUT` = `e2e-runs/<date>/$NAME/`.
Parseltongue verbs are the planned CLI (names fixed at M0); every other command exists today.

```text
 S1  INDEX WITH CODEBASE-MEMORY (fresh, full)
       codebase-memory-mcp cli index_repository --repo-path $REPO --mode full
       codebase-memory-mcp cli list_projects                   → note the project name for $REPO  ($CMP)
       codebase-memory-mcp cli index_status --project $CMP     → succeeded; node and edge counts → report
       codebase-memory-mcp cli get_graph_schema --project $CMP → label and edge-type counts → report
     pass: index succeeded at the pinned commit (stale index = re-index; never reuse an older generation)

 S2  INDEX COVERAGE — what codebase-memory could NOT parse
       codebase-memory-mcp cli check_index_coverage --project $CMP --scopes '["."]' --diagnostics full --format json
       codebase-memory-mcp cli query_graph --project $CMP --graph missed --query "MATCH (f) RETURN f.file_path"
     record: partially parsed / unusable files (rust-analyzer M: 137 partial, 148 unusable)
     rule:   any disagreement in S7 that lands in a missed file is bucket CM-GAP, never a Parseltongue failure

 S3  RUN PARSELTONGUE (full pipeline, CPU only)
       /usr/bin/time -v parseltongue map-workspace-full-run --workspace $REPO --db $OUT/pt.db --threads 4  2> $OUT/time.txt
       parseltongue status-coverage-banner-show --db $OUT/pt.db --json > $OUT/banner.json
       parseltongue hld-current-export-sorted --db $OUT/pt.db > $OUT/hld.tsv
     pass: exit 0; no WorkspaceNotCompilingError (REQ 004); child processes ⊆ {cargo metadata --offline, rustc --print cfg,
           rustc --print sysroot} (REQ 001, checked with `strace -f -e trace=execve`); no build.rs executed (REQ 015)

 S4  RUN THE ORACLE (offline rust-analyzer, same commit, same features)
       rust-analyzer scip $REPO --output $OUT/ra.scip                         → definitions + references per occurrence
       rust-analyzer analysis-stats $REPO --disable-build-scripts > $OUT/ra-stats-nobuild.txt
       rust-analyzer analysis-stats $REPO > $OUT/ra-stats-full.txt
     the two analysis-stats runs isolate KL1: what changes between them is build-script dependent (expected unresolved
     in Parseltongue as unresolved(build_script)), never counted as a Parseltongue failure

 S5  DIFF AGAINST THE ORACLE
       parseltongue oracle-edge-differ-run --db $OUT/pt.db --scip $OUT/ra.scip > $OUT/oracle-diff.json
     writes oracle-edge-agreement-table; reads off REQ 005 (paths ≥ 99.5 %), 006 (macro items ≥ 99 %), 013 (proc-macro
     shapes 100 %), 018 (signature edges ≥ 99.5 %), 007 (method precision ≥ 99 %), 008 (coverage ≥ 95 % after 7c)

 S6  STRUCTURAL CROSS-CHECK WITH CODEBASE-MEMORY (files, folders, items)
       query_graph --project $CMP --query "MATCH (f:File) RETURN count(f)"
       query_graph --project $CMP --query "MATCH (n) WHERE n:Function OR n:Method OR n:Struct OR n:Enum OR n:Interface
                                            OR n:Type RETURN labels(n)[0], count(*)"
       query_graph --project $CMP --query "MATCH (a)-[r:IMPLEMENTS]->(b) RETURN a.qualified_name, b.qualified_name"
     compare with parseltongue:
       C1  .rs file count of the workspace           equal, except files listed in S2 as missed
       C2  item counts per kind                       explained difference only (fn-body items, macro-expanded items,
                                                       cfg_inactive items — each difference assigned to one of these)
       C3  every IMPLEMENTS edge (impl Trait for T)   present in hld-current as `implements`, or explained

 S7  EDGE CROSS-CHECK WITH CODEBASE-MEMORY (type-aware edges only)
       query_graph --project $CMP --max-rows 99998 --format json --query
         "MATCH (a)-[r:CALLS]->(b) WHERE r.strategy STARTS WITH 'lsp_' AND r.strategy <> 'lsp_short_name_unique'
          RETURN a.qualified_name, a.file_path, a.start_line, b.qualified_name, b.file_path, r.strategy, r.confidence"
       (page with --cursor until has_more = false; the 100 k-row ceiling applies)
     map each edge to file-level: (a.file_path → b.file_path) and compare with hld-file-level-rollup
     classify every file pair that differs into exactly one bucket:
       AGREE        all three witnesses have it
       PT-BUG       rust-analyzer has it, parseltongue does not, and the banner gives no reason  → FAIL, open an issue
       PT-EXTRA     parseltongue has it, rust-analyzer does not                                  → FAIL (precision)
       KNOWN-GAP    parseltongue has it unresolved with a reason (proc_macro, build_script, cfg_inactive,
                    external_source_missing, trait_solver_needed) and S4/S5 confirm that cause → counted, not a failure
       CM-ONLY      only codebase-memory has it (heuristic or missed-file) → logged, not a failure
       CM-GAP       codebase-memory lacks it because of S2 coverage → logged, not a failure
     pass: 0 PT-EXTRA; PT-BUG ≤ the REQ thresholds above; every KNOWN-GAP has a reason row

 S8  SPOT CHECK BY READING CODE (50 random HLD rows, fixed seed = run date)
       for each sampled row: codebase-memory-mcp cli get_code_snippet --project $CMP --qualified-name <node01>
       a human or ext-llm-coding-agent confirms the call / type use / impl is in the snippet (or in the collapsed
       private chain named by the evidence rows)
     pass: 50 / 50 confirmed; any miss is a PT-EXTRA and fails the run

 S9  HLD SHAPE RULES (REQ 019, 009)
       every node: top-level item / inline-mod item / top-level impl or trait item, with non-private visibility
       no node inside a fn body; private `use` never a node; every `pub use` → one reexports edge
       every via = collapsed row has evidence rows naming the private items
       reachable_outside_crate agrees with rust-analyzer's visibility for 100 % of nodes
       codebase-memory check: trace_path --function-name <a collapsed row's private helper> --direction both
       → its callers and callees contain the two HLD endpoints (when both are in non-missed files)

 S10 DETERMINISM, MEMORY, TIME (REQ 012, 011, budgets)
       re-run S3 with --threads 1 and --threads 16 → hld.tsv byte-identical to the 4-thread run (`cmp`)
       peak RSS from time.txt ("Maximum resident set size") ≤ 2 GB on rust-analyzer (target); recorded for all five
       wall time recorded per stage from banner.json
```

## 3. Pass / fail per repository

```text
 PASS  S1–S10 all pass, AND the REQ thresholds hold:
         005 ≥ 99.5 % · 006 ≥ 99 % · 013 100 % · 018 ≥ 99.5 % · 007 ≥ 99 % precision · 008 reported (≥ 95 % after 7c)
         009 / 019 100 % · 012 identical · 011 ≤ 2 GB (rust-analyzer)
 FAIL  any PT-EXTRA, any S8 miss, any forbidden process, any non-identical re-run, any threshold missed
 BLOCKED  a precondition P1–P5 not met → fix it and run again; never record BLOCKED as PASS
```

HLD current **works** only when all five are PASS in the same run (same Parseltongue hash, same toolchain).

## 4. Run report (one file per run: `e2e-runs/<date>/REPORT.md`, append-only)

```text
 header        date · parseltongue hash · toolchain (P4) · machine (P5)
 per repo      pinned commit · codebase-memory nodes / edges / missed files (S1, S2)
               parseltongue: files, spans, nodes, HLD rows, peak RSS, wall time (S3, S10)
               oracle agreement per REQ (S5) — measured numbers, marked M
               S6 C1–C3 results · S7 bucket counts (AGREE, PT-BUG, PT-EXTRA, KNOWN-GAP by reason, CM-ONLY, CM-GAP)
               S8 50/50 · S9 · S10
               verdict PASS / FAIL / BLOCKED, with the first failing step
 summary       one line per repo; overall verdict; issues opened for every PT-BUG / PT-EXTRA
```

## 5. Rules

```text
 R1  all five repositories, every run (no subset, no "it passed last time")
 R2  rust-analyzer is the answer key; codebase-memory is a cross-check and never overrules it
 R3  codebase-memory absence is not proof of absence (S2 missed files, heuristic strategies)
 R4  numbers in the report are measured; targets are written as targets
 R5  parseltongue is never patched during a run; a fix means a new run of all five
 R6  re-index codebase-memory at the pinned commit for every run (S1); never reuse an old generation
```
