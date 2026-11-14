# SPEC inventory (scratch)

## headings
1: # Executable specs — phase-20261111: HLD for Rust code that compiles
8: ## 1. Scope and oracle
35: ## 2. Executable requirements
39: ### REQ-RUST-001.2 — offline graph and process boundary
61: ### REQ-RUST-002.0 — bytes and no-op ingest
67: ### REQ-RUST-003.1 — source coverage
73: ### REQ-RUST-004.2 — parser boundary
80: ### REQ-RUST-014.1 — compilation contexts
92: ### REQ-RUST-015.1 — build-script limitation
101: ### REQ-RUST-005.1 — paths
108: ### REQ-RUST-016.1 — fixed point and module semantics
117: ### REQ-RUST-006.2 — declarative/built-in expansion
123: ### REQ-RUST-013.1 — proc-macro shapes
131: ### REQ-RUST-017.1 — lang items and incoherent impls
140: ### REQ-RUST-018.1 — signatures and impls
145: ### REQ-RUST-007.2 — site precision
152: ### REQ-RUST-008.1 — mandatory final coverage
158: ### REQ-RUST-019.1 — D6 nodes and reexports
173: ### REQ-RUST-009.2 — extracted HLD and collapse
180: ### REQ-RUST-010.2 — dependency boundary
188: ### REQ-RUST-020.1 — generations and stage isolation
197: ### REQ-RUST-021.1 — concurrency
205: ### REQ-RUST-022.1 — cancellation
212: ### REQ-RUST-011.1 — memory
220: ### REQ-RUST-012.1 — determinism
226: ### 2.1 Matching and scoring
259: ### 2.2 Authoritative unresolved reasons
275: ### 2.3 Required resolution fixtures
292: ### 2.4 Visibility and collapse rules
310: ## 3. Rust design and persisted interfaces
312: ### 3.1 Layers and stages
331: ### 3.2 Identity, columns and generations
409: ### 3.3 Trait seams and errors
434: ### 3.4 Turso and cancellation
443: ## 4. Requirement → test → SOP crosswalk
539: ## 5. Implementation plan — authoritative M0–M8
579: ## 6. Quality gate results
584: ## 7. Resolved decisions and remaining measurements
593: ## 8. Repositories we take from
599: ### Rust analysis sources
633: ### Database and support sources
657: ### Oracle corpus sources
680: ### Ideas without copying
693: ### Agent tooling sources
704: ## Architecture revision — Turso write side / DuckDB analysis side
724: ### Crate inventory and stage ownership
746: ### Revised requirements and named RED contracts
827: ### §3.1 layer-table appended revision
835: ### §5 milestone-plan appended revision
847: ### Memory acceptance revision — correctness milestone first
857: ### S5 semantic path partition — published classification rule
878: ## REQ-RUST-024.6 — namespace-analysis-phase-diagnostics
899: ## REQ-RUST-024.7 — namespace-insertion-hypothesis-probes
928: ## REQ-RUST-024.8 — bounded-projected-namespace-handoff
974: ## REQ-RUST-024.9 — correctness-memory-observation-only
1034: ## REQ-RUST-024.10 — namespace-resolution-publication-receipts
1072: ## REQ-RUST-024.11 — published-stage-coverage-status
1124: ## REQ-RUST-025.0 — independent-source-grounded-hld-current-audit
1229: ## REQ-RUST-026.0 — comparator-persisted-evidence-validation
1240: ## REQ-RUST-026.1 — comparator-semantic-identity-canonicalization
1250: ## REQ-RUST-026.2 — comparator-primary-partition-semantics
1262: ## REQ-RUST-026.3 — comparator-secondary-evidence-visibility
1270: ## REQ-RUST-026.4 — comparator-cancellation-safe-publication
1281: ## REQ-RUST-026.5 — comparator-deterministic-semantic-export
1288: ### Authoritative REQ-RUST-026 test matrix
1379: ## REQ-RUST-027.0 — mapped-path-export-capability
1414: ## REQ-RUST-028.0 — child-stderr-live-visibility
1428: ### Test-harness-only L3 design
1440: ### Authoritative REQ-RUST-028 test matrix
1448: ## REQ-RUST-028.1 — per-refresh-namespace-operational-trace
1466: ### Authoritative REQ-RUST-028.1 test matrix
1479: ## REQ-RUST-028.2 — namespace-import-statement-trace
1497: ### Authoritative REQ-RUST-028.2 test matrix
1514: ## Phase 1 revision — itoa graph-first construction
1528: ### REQ-RUST-029.0 — source-derived reference separation
1544: ### REQ-RUST-030.0 — typed HLD materialization
1569: #### Historical bounded implementation checkpoint — direct projection only
1582: #### Identity/input-validation increment
1603: #### Historical recursive DuckDB path/collapse checkpoint
1625: #### DuckDB file/folder roll-up checkpoint
1653: ### REQ-RUST-031.0 — transactional HLD serving
1672: ### Authoritative Phase 1 HLD verification matrix
1674: #### Durable Turso publication contract
1755: ## Phase 1 revision — full graph first, levels derived (2026-11-13)
1780: ### REQ-RUST-032.0 — full item dependency graph
1821: ### REQ-RUST-019.2 — interface level is a filter (replaces 019.1)
1847: ### REQ-RUST-030.1 — relation kinds
1855: ### Not decided in this revision
1878: ### Named RED contracts for this revision

## requirement definitions (line: text)
39: ### REQ-RUST-001.2 — offline graph and process boundary
61: ### REQ-RUST-002.0 — bytes and no-op ingest
67: ### REQ-RUST-003.1 — source coverage
73: ### REQ-RUST-004.2 — parser boundary
80: ### REQ-RUST-014.1 — compilation contexts
92: ### REQ-RUST-015.1 — build-script limitation
101: ### REQ-RUST-005.1 — paths
108: ### REQ-RUST-016.1 — fixed point and module semantics
117: ### REQ-RUST-006.2 — declarative/built-in expansion
123: ### REQ-RUST-013.1 — proc-macro shapes
131: ### REQ-RUST-017.1 — lang items and incoherent impls
140: ### REQ-RUST-018.1 — signatures and impls
145: ### REQ-RUST-007.2 — site precision
152: ### REQ-RUST-008.1 — mandatory final coverage
158: ### REQ-RUST-019.1 — D6 nodes and reexports
173: ### REQ-RUST-009.2 — extracted HLD and collapse
180: ### REQ-RUST-010.2 — dependency boundary
188: ### REQ-RUST-020.1 — generations and stage isolation
197: ### REQ-RUST-021.1 — concurrency
205: ### REQ-RUST-022.1 — cancellation
212: ### REQ-RUST-011.1 — memory
220: ### REQ-RUST-012.1 — determinism
751: REQ-RUST-005.2: Turso is the concurrent write side; DuckDB is the analysis side.
755: REQ-RUST-010.3: Resolution is NON-MONOTONE. Recompute the complete binding set
765: REQ-RUST-011.2: Compute textual macro_rules scope and macro_use module order
773: REQ-RUST-010.4: Preserve self/super/crate paths, 2015/2018 roots, extern crate
783: REQ-RUST-016.2: DuckDB SQL owns signature joins, impl/trait indexes, method
787: REQ-RUST-021.2: S5–S7 comparison runs in DuckDB SQL with exact mapped-unit,
799: REQ-RUST-024.1: Copy only a COMPLETE PUBLISHED Turso generation using the Rust
807: REQ-RUST-024.2: Every DuckDB row carries its generation key. DuckDB is the
812: REQ-RUST-024.3: Every consumed query uses ORDER BY a total key; every tie-break
817: REQ-RUST-024.4: Set autoinstall_known_extensions=false,
849: REQ-RUST-024.5 supersedes the process-tree acceptance threshold in
850: REQ-RUST-024.4: the final target is peak RSS ≤500 MB (500,000,000 bytes) for
878: ## REQ-RUST-024.6 — namespace-analysis-phase-diagnostics
899: ## REQ-RUST-024.7 — namespace-insertion-hypothesis-probes
928: ## REQ-RUST-024.8 — bounded-projected-namespace-handoff
974: ## REQ-RUST-024.9 — correctness-memory-observation-only
1034: ## REQ-RUST-024.10 — namespace-resolution-publication-receipts
1072: ## REQ-RUST-024.11 — published-stage-coverage-status
1124: ## REQ-RUST-025.0 — independent-source-grounded-hld-current-audit
1229: ## REQ-RUST-026.0 — comparator-persisted-evidence-validation
1240: ## REQ-RUST-026.1 — comparator-semantic-identity-canonicalization
1250: ## REQ-RUST-026.2 — comparator-primary-partition-semantics
1262: ## REQ-RUST-026.3 — comparator-secondary-evidence-visibility
1270: ## REQ-RUST-026.4 — comparator-cancellation-safe-publication
1281: ## REQ-RUST-026.5 — comparator-deterministic-semantic-export
1379: ## REQ-RUST-027.0 — mapped-path-export-capability
1414: ## REQ-RUST-028.0 — child-stderr-live-visibility
1448: ## REQ-RUST-028.1 — per-refresh-namespace-operational-trace
1479: ## REQ-RUST-028.2 — namespace-import-statement-trace
1528: ### REQ-RUST-029.0 — source-derived reference separation
1544: ### REQ-RUST-030.0 — typed HLD materialization
1653: ### REQ-RUST-031.0 — transactional HLD serving
1765: REQ-RUST-031.0 stand, read with "selected" meaning REQ-RUST-019.2's filter.
1780: ### REQ-RUST-032.0 — full item dependency graph
1821: ### REQ-RUST-019.2 — interface level is a filter (replaces 019.1)
1847: ### REQ-RUST-030.1 — relation kinds

## latest version per id family
001: versions [2] latest .2
002: versions [0] latest .0
003: versions [1] latest .1
004: versions [2] latest .2
005: versions [1, 2] latest .2
006: versions [2] latest .2
007: versions [2] latest .2
008: versions [1] latest .1
009: versions [2] latest .2
010: versions [2, 3, 4] latest .4
011: versions [1, 2] latest .2
012: versions [1] latest .1
013: versions [1] latest .1
014: versions [1] latest .1
015: versions [1] latest .1
016: versions [1, 2] latest .2
017: versions [1] latest .1
018: versions [1] latest .1
019: versions [1, 2] latest .2
020: versions [1] latest .1
021: versions [1, 2] latest .2
022: versions [1] latest .1
024: versions [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] latest .11
025: versions [0] latest .0
026: versions [0, 1, 2, 3, 4, 5] latest .5
027: versions [0] latest .0
028: versions [0, 1, 2] latest .2
029: versions [0] latest .0
030: versions [0, 1] latest .1
031: versions [0] latest .0
032: versions [0] latest .0

## REQ ids cited in code (count id)
      1 REQ-RUST-005.1
      3 REQ-RUST-009.2
      1 REQ-RUST-019.1
      2 REQ-RUST-020.1
      1 REQ-RUST-021.1
      2 REQ-RUST-024.1
      3 REQ-RUST-024.10
      5 REQ-RUST-024.11
      2 REQ-RUST-024.2
      6 REQ-RUST-024.6
      2 REQ-RUST-024.7
      5 REQ-RUST-024.8
      6 REQ-RUST-024.9
     14 REQ-RUST-026.0
     13 REQ-RUST-026.1
     22 REQ-RUST-026.2
      7 REQ-RUST-026.3
     16 REQ-RUST-026.4
      2 REQ-RUST-026.5
      1 REQ-RUST-027.0
      5 REQ-RUST-028.0
      4 REQ-RUST-028.1
      6 REQ-RUST-028.2
     12 REQ-RUST-030.0
     16 REQ-RUST-031.0


## tests named in SPEC but absent in code: 50 of 205

## tests in code not named in SPEC: 137 of 292

## Notes for the rewrite (written 2026-11-13 before context summary)

- Suffix meaning differs by family. For 001-022 the suffix is a VERSION (005.1 -> 005.2,
  010.2 -> 010.3 -> 010.4, 011.1 -> 011.2, 016.1 -> 016.2, 019.1 -> 019.2, 021.1 -> 021.2).
  CAUTION: check each: 010.3 / 010.4 / 011.2 / 016.2 / 021.2 / 005.2 were introduced in the
  "Architecture revision - Turso write side / DuckDB analysis side" section and may be NEW
  requirements that reuse a family number rather than replacements. Read that section
  (old SPEC ~lines 703-860) before merging.
- For 024.x, 026.x, 028.x the suffixes are SEPARATE requirements (024.6 phase diagnostics,
  024.8 bounded handoff, 024.9 memory observation only, 024.10 receipts, 024.11 coverage
  status, 026.0-026.5 comparator, 028.0-028.2 stderr/trace). Keep every one. 024.5 supersedes
  the threshold in 024.4 only.
- Code cites these ids and they must stay resolvable in the clean SPEC (add an id-history
  table for superseded ones): 005.1, 009.2, 019.1 (superseded by 019.2), 020.1, 021.1,
  024.1, 024.2, 024.6-024.11, 026.0-026.5, 027.0, 028.0-028.2, 030.0, 031.0.
- 205 test names appear in the old SPEC; 50 of them are not in code (planned). 137 code tests
  are not named in the SPEC. tests.json holds both lists. The clean SPEC must keep every
  test name the old SPEC lists and mark planned vs present-in-code from tests.json.
- Decided rules to state as current truth (no "revision" wording):
  full-item-dependency-graph first (032.0); interface level = filter (019.2); relation kinds
  calls/uses_type/implements/reexports/contains/derives/imports/reads (030.1); collapse never
  walks contains or imports; file-public-area-node per file; every use is a node; inline
  module ordinary; edges target real item, use node is evidence; full graph = the two tables
  item-effective-visibility-records + edge-evidence-record-table; impl blocks not nodes
  (029.0); builtin-source implements facts stored, never seed interface edges; registry gives
  each table one owner stage; Turso write/serving + DuckDB analysis, no third database.
- Open points (list once): OD-7 containment; OD-8 non-function reads/uses_type source;
  collapse limited to calls/reads chains? (64 of 134 collapsed itoa edges are inherited
  uses_type); pub use imports+reexports; enum variants/fields; macro-generated file
  assignment (I4); agent query surface (T2); private items visible across files (L7);
  AD-1 core handling; AD-2 body-path resolver; AD-3 visibility stage placement; AD-4 body
  inference scope; AD-5 reference source.
- Target file names: docs/phase-20261113/PRD-phase-20261113-dependency-graph-hld.md,
  SPEC-phase-20261113-executable-requirements.md, ARCH-phase-20261113-pipeline-graph-tables.md.
  Archive folder for folded drafts: docs/phase-20261113/archive-folded-draft-files/.
  SOP and journals are NOT rewritten; fix references only. AGENTS.md canonical paths updated.
- itoa target dataset stays at docs/phase-20261113/itoa-target-dataset/ (73 nodes, 368 edges).

## VERIFIED after reading old SPEC lines 704-877 (overrides the CAUTION above)

- 005.2, 010.3, 010.4, 011.2, 016.2, 021.2 are NEW, SEPARATE requirements that reuse a
  family number. They do NOT replace 005.1 (path agreement >=99.5%), 010.2 (dependency
  boundary), 011.1 (memory), 016.1 (fixed point), 021.1 (concurrency). Keep ALL of them
  as distinct requirements in the clean SPEC, each with its own text:
    005.2  Turso write side / DuckDB analysis side; no third DB, LLM, salsa, RA semantic crate
    010.3  non-monotone resolution, explicit shadows glob, ambiguity rows, 8192 rounds;
           RED cases late-explicit-overrides-glob, conflicting-glob-targets-ambiguous,
           cyclic-glob-imports-terminate
    011.2  textual macro_rules scope in Rust, legacy-macro-scope-records, SQL/Rust
           alternation; RED legacy-macro-textual-order
    010.4  path prefixes, edition roots, extern aliases, preludes, use Trait as _, enum globs,
           raw identifiers, pub(in path), importer visibility, block scopes; ten RED cases
    016.2  DuckDB SQL owns signature joins, impl/trait indexes, method candidates, HLD
           collapse and roll-ups; no in-memory resolver fallback
    021.2  S5-S7 comparison in DuckDB SQL; module-scope vs type-relative partition;
           RED module-associated-denominator-partition
    024.1  copy only a complete published generation via Appender; count+digest equality
    024.2  generation key on every DuckDB row; only final product tables copy back
    024.3  total ORDER BY; thread-count determinism
    024.4  extensions off, no external access, temp dir in run dir; no memory cap until
           all-five S5
    024.5  final target peak RSS <= 500,000,000 bytes whole tree, enforced after all-five S5;
           supersedes the 2 GiB gate of 011.1 and the ~1.2 GiB engine cap
- Real supersessions only: 019.1 -> 019.2; 011.1 threshold -> 024.5; "Turso is the only
  engine" -> 005.2; serde-first development order -> itoa Phase 1 first (serde first in
  Phase 2); stage 8 merged into 6/7 writes -> registry single owner stage per table.
- Also keep: S5 path-partition classification rule (old lines 857-877), crate inventory
  (724-745), mermaid stage flow (710-722, add the full-graph step).
- Still unread in full: old SPEC lines 878-1513 (024.6-024.11, 025.0, 026.x, 027.0, 028.x).
  These are self-contained requirement sections with their own test matrices; carry them
  over verbatim in meaning, dropping only "revision/historical" framing.

## Assembly plan for the clean SPEC (decided 2026-11-13)

Build it with a script (.rewrite-scratch/assemble-clean-spec.py) so nothing is dropped:
parse the old SPEC into blocks by heading line (list above) and by "REQ-RUST-nnn.m:" paragraph
starts inside old lines 746-826; copy blocks verbatim into a new thematic order; apply
targeted substitutions; hand-write only the HLD/full-graph blocks. Then diff the set of REQ
ids and test names old vs new (must be a superset) and check fences.

New order:
 1 Scope and oracle                      old 8-34 verbatim
 2 Architecture                          new intro (two databases, full graph first) + old
                                         710-745 (mermaid: add full-graph step; crate list) +
                                         old 3.1 (312-330; drop parseltongue-port-name-resolver,
                                         state registry single-owner rule)
 3 Requirements
   3.1 input/process    001.2 002.0 003.1 004.2 014.1 015.1
   3.2 resolution       005.1 016.1 010.3 010.4 011.2 006.2 013.1 017.1 + S5 partition rule
   3.3 signatures/body  018.1 007.2 008.1
   3.4 graph and levels 032.0 019.2 009.2 030.0(+validation and SQL text, no "historical
                        checkpoint" framing) 030.1 029.0 031.0(+publication contract) 025.0
   3.5 engines/storage  005.2 016.2 010.2 020.1 021.1 022.1 024.1-024.5 024.8 024.10 024.11
   3.6 memory/determinism 011.1 (threshold now 024.5) 012.1 024.9
   3.7 comparator       021.2 026.0-026.5 027.0
   3.8 diagnostics      024.6 024.7 028.0 028.1 028.2
 4 Rules: scoring (2.1), unresolved reasons (2.2), fixtures (2.3), visibility+collapse (2.4,
   rewritten: collapse never walks contains/imports; 019.2 filter)
 5 Persisted interfaces: columns (3.2, relation kinds incl. imports/reads), seams (3.3),
   Turso (3.4)
 6 Verification matrix: old section 4 crosswalk + per-section matrices + 14 new contracts
 7 Plan: Phase 1 itoa first, then M0-M8 all-five
 8 Open points (one list)   9 Id history   10 Sources (old section 8)
