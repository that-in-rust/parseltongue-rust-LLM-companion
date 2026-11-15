# Second opinion: GLM-5.3 (pi, max reasoning, read-only) on the part 4d working tree

Date: 2026-11-15. Harness: `~/bin/zpi` (pi, `zai/glm-5.3`, `--thinking max`, tools read/grep/find/ls).
Subject: the uncommitted part 4d diff (import outcome table, `resolve_use_import_edges`, full-graph wiring),
before it was committed. Rule 5 of `AGENTS.md`.

## What I did with each claim

```text
claim                                                       checked how                                   result
----------------------------------------------------------  --------------------------------------------  ---------------------------
path table is unchanged by the SQL change                   export of both stores, row-by-row diff        confirmed: 301 rows, 0 differ
import table holds 8 rows on itoa, 4 of kind Signature      export of the table                           confirmed: 8 rows
  -> fix: store the request kind, write a row only for      RED bf4be6dd4, GREEN fc6019e03                fixed: 4 rows on itoa
     kind import (also keeps the prelude import out)
test_skipped_import_sites_counted passes with two           read the test                                 confirmed; now two globs,
  counters swapped                                                                                        expected [2,1,1] (0990f9a68)
trait_solver_needed maps to Other                           RED 0990f9a68 (Other where TraitSolverNeeded) fixed: 4f42d9c87
an Import request with a use node and no outcome row        new assertion in test_import_scope_cancel_    confirmed: typed error
  aborts with InvalidEvidence                                 errors                                      InvalidEvidence; kept as is
ambiguous is not tested at the SQL seam                     read test_import_outcome_rows_written         confirmed; not added (the
                                                                                                          fixture has no second
                                                                                                          candidate); matrix corrected
one site with two targets (R3) has no behavioural test      read the fixture                              confirmed; not added; open
                                                                                                          question Q5 in the part file
global -> leading "::", defining_crate -> "$crate"          read read_import_path_text                    confirmed; neither occurs in
                                                                                                          itoa; open question Q6
endpoints are local items only, so a resolved import into   read the resolver                             confirmed; the planned change
  core stays MissingTarget until part 6                                                                   point of part 6
alias is a separate metadata field, R6 wording ambiguous    read the resolver                             confirmed; R6 reworded
only external_source_missing is reachable for imports today not checked in source                         recorded as the reviewer's
                                                                                                          claim in the part file
a use node range cannot contain another use node range      not checked                                   not relied on
```

Where I disagree: none of the six defects. One recommendation is not taken in this part: a typed skip
for an import whose unit is not a mapped output context. On pinned itoa every unit with a `use` node
is mapped, and a silent skip would hide a store defect. It is open question Q7 in the part file.

## Prompt

```text
You are reviewing uncommitted work in the git repository at /home/ubuntu/repos/parseltongue-rust-LLM-companion (branch research/tiny-rust-code-mapper). Read-only: do not edit anything. Use read/grep/find/ls. Cite file and line for every claim.

Context. Part 4d of docs/phase-20261113/SPEC-itoa-full-item-graph/ (file part-4d-imports-from-use-items.md, REQ-RUST-049.0, R1..R12) adds `imports` edges from original Rust `use` items to the full item dependency graph for pinned itoa 1.0.18 (/home/ubuntu/repos/rust-tools/itoa). Only itoa is in scope.

The uncommitted change (see `git status`, `git diff`, and the two untracked files):
1. crates/parseltongue-duckdb-analysis-adapter/src/namespace-final-path-products.sql: adds column import_request_flag = the request key exists in "namespace-import-request-records"; LEFT JOIN on path metadata so a request with no path metadata but an import request is kept.
2. crates/parseltongue-duckdb-analysis-adapter/src/namespace-final-output-records.rs: PathNamespaceProduct::finish now returns the path row (when path metadata exists, exactly as before) AND an extra row in a new table import-resolution-site-records {import_site_occurrence_key, import_resolution_state_kind, import_target_symbol_rows, import_candidate_target_rows, import_unresolved_reason_kind} when the request key is an import request.
   History: a first version routed a request to EITHER table by r.source_kind='import'. That moved 6 rows out of path-resolution-site-records on itoa (301 -> 295) and one call (`i.write(..)` in Buffer::format, resolved via the bound walk I: Integer + Copy) lost its target (resolved_calls 3 -> 2). The current version restores 301 path rows and resolved_calls=3.
3. crates/parseltongue-syntax-span-extractor/src/use-import-edge-records.rs (new) + use-import-edge-tests.rs (new): resolve_use_import_edges joins syntax-namespace-input-imports (kind "Import" only) with import-resolution-site-records and the UseStatement nodes of the graph input.
4. crates/parseltongue-cli-dispatcher-binary/src/full-graph-checkpoint-facade.rs: calls it, appends rows, prints imports= and three counters.
5. Table registered in crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs and crates/parseltongue-cli-dispatcher-binary/src/namespace-pipeline-publication-facade.rs.

Measured on pinned itoa (release build): nodes=58 calls=61 resolved_calls=3 type_uses=34 implements=32 reads=21 imports=4 imports_glob_skipped=0 imports_in_expanded_items=0 imports_without_source=0, 195 edges. The 4 rows are unresolved ExternalSourceMissing with import_path_text core::hint, core::mem, core::mem::MaybeUninit, core::str.

Questions. Answer each with evidence, and say plainly where I am wrong.
Q1. Correctness of the SQL change: can the LEFT JOIN / the new WHERE / the extra DISTINCT subquery join change, duplicate or drop any path-resolution-site-records row compared with HEAD, on any input (not just itoa)? Can a request key be both a path request with metadata and an import request, and is emitting both rows right?
Q2. Against R1..R12 of the part file: which requirement does the resolver violate or fail to cover? Look especially at: the order of the three skip checks (expanded, no source node, glob); a glob import inside a function body; a use nested in an inline module; a `use` whose range contains another node's range; dedup of rows; the handling of "resolved" with zero graph-node targets; reason mapping Some(_) => Other.
Q3. import_path_text: is the segment-kind mapping in read_import_path_text complete against what the syntax stage actually writes into import_path_segment_rows? Find the writer and list every kind it can write.
Q4. resolve_use_import_edges builds an ImplHeaderSourceContext (reads workspace-config-state-records) only to call read_single_target_identity. Is there a simpler existing decoder? Is anything wrong with reusing it?
Q5. Tests: use-import-edge-tests.rs is 386 lines for 5 tests. Which requirement has no behavioural test? Which test would still pass if the resolver were wrong in a way that matters (weak assertions)?
Q6. Anything in this diff that will bite when macro-generated items become nodes (part 5) or when core targets are named external targets (part 6, built from import_path_text)?
Q7. What did I fail to ask?

Format: numbered answers Q1..Q7, each: verdict (ok / defect / risk), evidence (file:line), and the smallest fix. End with a list of defects ordered by severity. Be concrete and brief; no praise.

```

## Output, verbatim

# Review of part 4d (`imports` edges)

## Q1 — SQL change

**Verdict: ok (with one letter-of-spec caveat, see Q2/R1).**

- No path row can change, duplicate or drop. The new join is `LEFT JOIN (SELECT DISTINCT generation_key,attempt_key,request_key FROM "namespace-import-request-records") i` (namespace-final-path-products.sql:80-81) — at most one row per request key, so it cannot fan out. The new column `i.request_key IS NOT NULL AS import_request_flag` (:67) is a function of `r.request_key` alone, so adding it to `SELECT DISTINCT` (:59) cannot split or merge rows. The `WHERE p.metadata IS NOT NULL OR i.request_key IS NOT NULL` (:82) is a pure widening: every HEAD row (metadata present) is still selected with identical column values; only metadata-less import requests are added. Grouping in Rust is per `request_key` in `ORDER BY r.request_key,...` order (:83), unchanged; `PathNamespaceProduct::finish` (namespace-final-output-records.rs:81-107) emits the path row exactly as before and chains the import row after it. The old `path metadata absent` error branch is now unreachable (WHERE guarantees one of the two) — dead but harmless (namespace-final-output-records.rs:178-181).
- Yes, a key can be both, and it is the *common* case on itoa. The import site is the `UseTree` occurrence (syntax-namespace-input-extraction.rs:1470) and the signature path site is the `Path` occurrence (:1507); for a plain `use core::hint;` (no group, no alias) both nodes have the same text range → the same occurrence key → the same `request_key`, which then has path metadata *and* an import request row. That is exactly the 6 rows your first version routed away (301→295) and the lost `i.write` walk target (`read_type_path_records` reads path-resolution-site-records only, impl-header-source-records.rs:270-289). Emitting both rows is right: the path row is load-bearing for calls/type_uses; the import row is the R1 outcome.
- Smallest fix: none needed for correctness. Optional: delete the dead `value.is_none() && import.is_none()` branch or keep it as a guard.

## Q2 — Against R1..R12

**Verdict: R2, R3, R4, R6, R7, R8, R11, R12 implemented as written; three deviations, all low severity.**

- **R1 (letter).** The flag is existence-only, so `import-resolution-site-records` gets a row for *every* import request, including kind `Signature` (8 rows on itoa, not 4). R1 says "one row per import request of kind Import". The SQL cannot filter by kind: the frozen codec hardcodes `"import"` as source_kind for every import row (frozen-namespace-source-codec.rs:449-453), so `r.source_kind` is useless here; the real kind survives only in the published JSON, which the resolver filters (use-import-edge-records.rs:93). Fix: one sentence in the part file ("outcome rows are written for all import requests; R8 filtering happens in the resolver"), or persist the kind into `namespace-import-request-records`.
- **R9/R10 precedence.** Order is expanded → without-source → glob (use-import-edge-records.rs:96-110). A glob `use` in a function body is counted `imports_without_source`, and a glob in expanded text as `imports_in_expanded_items`; R9's unconditional "counted as imports_glob_skipped" says otherwise, R10 says otherwise for the body case — the spec contradicts itself on the overlap. Counters only, no edge effect. Fix: fix the precedence in the part file (current order is the sensible one).
- **R5 (letter).** `Some(_) => Other` (use-import-edge-records.rs:243-248) discards the stored reason text. Today only `external_source_missing` (SQL literal, :73) and `cfg_inactive` can arrive via the frozen path (reasons pass through verbatim snake_case, frozen-namespace-source-codec.rs:285-288), so nothing is lost on itoa; but the in-process contract codec writes the same enum as Debug CamelCase (namespace-contract-input-codec.rs:140), and `build_script`/`proc_macro`/`unknown_receiver`/`trait_solver_needed` would all silently become `Other`. Fix: match the remaining snake names explicitly (they exist in `SiteUnresolvedReasonKind`, core lib.rs:235-241) and put the raw string into dispatch metadata when `Other`.
- Order-of-checks detail that *is* right: the kind check precedes the glob check, which matters because the syntax stage writes `import_glob_expansion_flag: true` as a sentinel on every Signature bound request (syntax-namespace-input-extraction.rs:1447) — glob-first would miscount all 4 Signature rows. Your fixture models this correctly (kind "Signature" + glob true).
- Glob in a fn body: see R9/R10 above; produces no row either way.
- Use in an inline module: works — `find_enclosing_use_node` matches unit + file + byte-range containment (use-import-edge-records.rs:182-204); module nesting is irrelevant.
- A `use` containing another node's range: two holders → `None` → counted without_source (:200-204). Unreachable in original text (USE items don't nest, expanded use nodes are excluded at :167-172), so R2's "exactly one" holds.
- Dedup: one row per distinct target per site — targets go through a `BTreeSet` (:266-272) and the evidence key carries the site (:127-135), then sort+dedup (:145-146). Matches "one edge row per source site".
- Resolved with zero graph-node targets → one `MissingTarget` row (:267-270), per R4. Caveat: a target that *is* an item but has ≠1 stored identity is silently treated as not-a-node (see Q4).

## Q3 — `import_path_text` segment mapping

**Verdict: complete.**

The writer is `encode_path_segment_values` (syntax-namespace-record-codec.rs:281-293) and it can write exactly six kinds: `name`, `crate`, `self`, `super`, `global`, `defining_crate`. `read_import_path_text` maps all six (use-import-edge-records.rs:205-226), and the frozen codec accepts exactly the same set (frozen-namespace-source-codec.rs:297-300). Two rendering conventions are unpinned contract decisions for part 6: `global` → `""` produces a leading `::` (`::core::mem`), and `defining_crate` → the invented string `"$crate"`. Neither is reachable for original-text uses today. Fix: write both renderings into the part-6 contract when it is specified.

## Q4 — reusing `ImplHeaderSourceContext`

**Verdict: ok to reuse; no simpler decoder exists; two real drawbacks.**

- It is the established pattern: `resolve_original_call_edges` does the identical workspace read + `read_impl_source_context` (original-call-resolution-records.rs:60-65). And there is no simpler decoder for your input: `import_target_symbol_rows` are *symbol rows* (target metadata + namespaces), not identities; `decode_symbol_target_value` decodes identity values only (syntax-namespace-record-codec.rs:137-143). The symbol-row → graph-identity mapping is exactly what the items table + `read_single_target_identity` (impl-header-source-records.rs:293-308) provides. Reconstructing an occurrence key from the symbol row's file/range fields would be a second, riskier copy of that mapping.
- Drawback 1: you build a 6-table context (units, roots, spans, targets, paths, workspace — impl-header-source-records.rs:47-95) and use only `targets`. An `InvalidReference` from roots/spans now aborts the imports step for a reason unrelated to imports.
- Drawback 2: `read_single_target_identity` returns `None` both for "not an item" and for "conflicting identities" (impl-header-source-records.rs:297-308); a real node with two identities degrades to `MissingTarget` with no trace. Per Rule 1 that is a guess-shaped silent drop.
- Fix: smallest — keep the reuse, add a comment; better — split a `read_symbol_target_index` (items table only) out of `read_impl_source_context`, and make ≠1 identities a typed outcome rather than `MissingTarget`.

## Q5 — Tests

**Verdict: defect (gaps), plus two weak spots.**

- **No behavioural test for R3's multi-target site** ("one site may have more than one row — type and macro namespace"). Every fixture outcome has 0 or 1 target (use-import-edge-tests.rs:120-129). A resolver that kept only the first target would pass all 5 tests.
- **The `InvalidEvidence` paths have no test**: an Import request with no outcome row (use-import-edge-records.rs:111-116) and a bad segment kind (:205-215) are uncovered; `test_import_scope_cancel_errors` covers only cancel and scope.
- **R1 fixture diverges from the matrix**: the promised "ambiguous" case is absent — `test_import_outcome_rows_written` asserts resolved / unresolved-null-reason / external_source_missing (published-generation-copy-tests.rs:454-463); no test that the analysis stage writes an *ambiguous* import outcome.
- Weak assertions: (a) `make_import_fixture_graph` builds no use node for body uses (uses after the first `fn` are skipped, use-import-edge-tests.rs:49-53), so the `node_function_body_ancestry_flag` filter in `collect_use_node_keys` is untestable by these fixtures — removing it would still pass; (b) the skipped-counters test cannot distinguish glob-vs-without_source attribution for overlapping cases (the glob has a source node, the body use is never a glob) — fine today, but it won't catch a reordering when someone adds the glob-in-body case; (c) the itoa test asserts `use_nodes.contains(source)` but not *which* use node per site (itoa-source-acceptance-tests.rs:2330-2335) — the fixtures do pin that, so acceptable.
- Also note: the extracted-target path has **zero real-source coverage** — on itoa all 4 rows are unresolved, so R3/R4's target matching runs only against fixtures.

## Q6 — Part 5 / part 6 exposure

**Verdict: risk, two concrete bites.**

- **Part 5 (macro items as nodes):** (a) duplicate identities for one lookup key (exactly what macro-generated duplicates create) silently degrade resolved imports to `MissingTarget` via `read_single_target_identity` (impl-header-source-records.rs:297-308); (b) macro-emitted `use` items stay skipped forever — sites with a chain count `imports_in_expanded_items` (use-import-edge-records.rs:96-101) and macro use nodes are excluded as sources (:167-172), so part 5 must reopen R10 or the counter becomes a permanent hole.
- **Part 6 (core targets from `import_path_text`):** (a) a sysroot-*included* run hard-errors: sysroot units have `unit_mapped_target_flag: false` (sysroot-compilation-context-rows.rs:43) so the final SQL's INNER JOIN on mapped contexts (namespace-final-path-products.sql:71; contexts built at namespace-final-output-records.rs:136) yields no outcome rows for their import requests, and the resolver turns a missing outcome into `InvalidEvidence`, aborting the whole graph. Today masked by `--sysroot-source-scope-kind excluded`. (b) The `""`→`::` and `"$crate"` renderings (Q3) are part of part 6's input contract but unspecified.
- Fix: decide the missing-outcome policy now (counted skip vs hard error) and record it; pin the two path-text renderings in the part-6 spec.

## Q7 — What you failed to ask

1. **What is actually in the new table** — 8 rows on itoa, not 4 (Signature outcomes included; Q2/R1). Nobody asked what `export-published-stage-rows` will show.
2. **The missing-outcome policy** (Q6a) — an import request with no outcome row kills the entire full-graph run; every other resolver degrades to a typed unresolved reason.
3. **An accounting identity** — `rows + glob + expanded + without_source == Import-kind requests` is true on itoa (4+0+0+0 of 4) but asserted nowhere; a swallowed request class would be invisible.
4. **Measurement** — the question reports counts but no wall-time/peak-memory numbers; Rule 3 requires them for the log entry.
5. **`import_candidate_target_rows` is written and never read** — fine per R1, but ambiguous detection relies solely on `import_resolution_state_kind`; worth one sentence so part 6 doesn't rediscover it.

## Defects by severity

1. **Risk (medium):** missing outcome row → hard `InvalidEvidence` abort of the whole graph; reachable with sysroot included or any unmapped unit (use-import-edge-records.rs:111-116 + namespace-final-path-products.sql:71 + namespace-final-output-records.rs:136). Decide policy, then counted-skip or typed reason.
2. **Defect (low, spec letter):** `import-resolution-site-records` contains Signature-kind rows; SQL cannot filter because the frozen codec hardcodes source_kind `'import'` (namespace-final-path-products.sql:67,80-81; frozen-namespace-source-codec.rs:449-453). Fix the part file or persist the kind.
3. **Risk (low):** `Some(_) => Other` drops stored reasons; contract-path reasons are Debug CamelCase and would all map to `Other` (use-import-edge-records.rs:243-248; namespace-contract-input-codec.rs:140). Map the six snake names, keep raw text in metadata.
4. **Risk (low, part 5):** ≠1 identities per target silently degrade resolved imports to `MissingTarget` (impl-header-source-records.rs:297-308). Typed outcome needed before macro items are nodes.
5. **Test gap (low):** no multi-target site test (R3), no missing-outcome/bad-segment error tests, no ambiguous-outcome test in the R1 fixture, no real-source coverage of the extracted-target path.
6. **Spec ambiguity (low):** R9/R10 precedence for glob-in-body / glob-in-expanded contradicts R9's unconditional wording; code picks R10 (use-import-edge-records.rs:96-110). One sentence in the part file.
7. **Cosmetic:** dead `path metadata absent` branch (namespace-final-output-records.rs:178-181); 6-table context built for a 1-table need; `workspace` row missing silently becomes `""` (use-import-edge-records.rs:64-70).
dge-records.rs:113-117) fails the whole publication instead of storing a typed reason (AGENTS Rule 1 prefers stored reasons). Unreachable while every mapped unit gets outcome rows (the JOIN on `namespace-mapped-output-contexts`, sql:72-73), but worth a sentence in the part file.

---

**Defects by severity**

1. **R1 letter violation — `import-resolution-site-records` stores outcome rows for Signature-kind requests too** (sql:80 subquery cannot filter by kind; codec writes all kinds; *derived* 8 rows on itoa, 4 used; untested). Fix: persist the kind in `namespace-import-request-records` and filter the subquery; add a test that a Signature request yields no stored outcome row.
2. **R8 behaviourally unpinned** — the skip test's Signature fixture has `glob: true`, so removing the kind check keeps tests green (tests:335-341; production always sets glob=true on Signature, extraction:1450). Fix: a `glob: false` Signature fixture with a resolvable target.
3. **Unreachable-but-wrong reason arm** — `trait_solver_needed` would map to `Other` though `TraitSolverNeeded` exists; `cfg_inactive`/`Other` arms are currently dead because the group reason always comes from the NULL-reason import leg (sql:83 + output-records group start). Fix: one match arm + a comment on why the arm is dormant.
4. **R2 two-holders branch and `global`/`defining_crate` path texts untested** (use-import-edge-records.rs:193-200, 218-219). Fix: two small fixtures.
5. **Counter precedence undocumented** — glob-in-body counts `imports_without_source` (order at lines 99-111); R9/R10 both claim it. Fix: one sentence in the part file.
6. **Part 6 text traps** — `"::core::mem"` and `"$crate"` path texts (lines 218-219) need a rule before part 6 parses them. Fix: decide in the part file or backlog now.
7. **Process** — one large uncommitted change vs the spec's six steps; LOG current state, part §6 gates, and run cost not yet recorded.

