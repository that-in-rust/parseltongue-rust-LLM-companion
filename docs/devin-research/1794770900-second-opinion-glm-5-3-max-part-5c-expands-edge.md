# Second opinion: part 5c, the `expands` edge (GLM-5.3, pi, max reasoning, read-only)

Date: 2026-11-16. Run directory on the VM: `/home/ubuntu/zcode-runs/5c-design/`. About 8 minutes.

## What I checked, and the result

```text
claim                                                    checked how                        result
no published table links a call to its definition        exported legacy-macro-scope-records  WRONG: 15 of 15 calls
                                                         from the store and joined it to      join to one definition
                                                         the node table
the macro path range is stored nowhere                   same export                          WRONG: it is the site of
                                                                                              the lookup row
Expands is the tenth relation kind, not the ninth        read the enum                        right
macro_source_range_start is the body start               not needed once the lookup table     not checked
                                                         is used
arm index fits edge_dispatch_metadata_text               read the record                      right
calls per definition are 5, 6 and 4                      my join gives the same               right
all three macros have one arm                            expansion rows all say arm 0         right
files and lines that break on a new kind                 used as the edit list                checked as edited
```

## Where I differ

- **Target source.** The review says to add a field to the expansion row. I use the stored
  textual-scope lookup, which already names the definition, and cross-check it against the
  token origins. A macro called by path has no such lookup; that case stops the run (E7).
- **Site.** Agreed: the whole call.

## Prompt

You are reviewing a design, read-only. Do not edit any file. Repository: /home/ubuntu/repos/parseltongue-rust-LLM-companion (branch research/tiny-rust-code-mapper).
Read first: AGENTS.md (Rules 4-6), docs/phase-20261113/SPEC-itoa-full-item-graph/part-5-macro-generated-items.md, docs/phase-20261113/PRD-itoa-full-item-graph-scope.md ("Decisions in force").

# Part 5c: the `expands` edge, macro call -> macro definition

Decided by the user: macro definitions and macro calls are graph nodes; a macro call points to the definition it expands by an `expands` edge. Design rule: give a coding agent the most source-backed information; never invent a fact.

Measured on pinned itoa 1.0.18 (store exported to /home/ubuntu/phase5b-work/verify/, files nodes.jsonl and mx.jsonl):
- 3 MacroDefinition nodes (impl_Integer, impl_Integer_size, impl_Unsigned), 15 MacroInvocation nodes, all written in src/lib.rs.
- macro-expanded-item-table: 15 rows with macro_resolution_state_kind "expanded" (macro_invocation_identity_record, macro_matched_arm_index, macro_token_origin_rows with token_source_origin_kind definition|invocation, token_original_byte_range, token_source_absolute_path) and 16 rows "unresolved" (reason external_source_missing) for macro calls inside expanded bodies.
- The row has no field naming the definition. The expander finds the definition through the resolved name binding: crates/parseltongue-syntax-span-extractor/src/persisted-macro-expansion-adapter.rs around line 325-390 (binding.binding_target_identity_key -> projection_target_metadata_rows -> target_macro_definition_record with macro_source_absolute_path, macro_source_range_start, macro_definition_unit_key). That record is written in syntax-namespace-input-extraction.rs around line 655-680.
- Graph nodes and edges are assembled in crates/parseltongue-duckdb-analysis-adapter/src/original-item-graph-projection.rs (+ .sql) and generated-item-graph-projection.rs (+ .sql). Relation kinds are in crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs (HldRelationKindValue; `Generates` was added in part 5a and is validated there).

# My lean

1. New relation kind `Expands`. One row per macro call that was expanded: source = the MacroInvocation node, target = the MacroDefinition node, evidence site = the macro path in the call (e.g. the bytes `impl_Integer` of `impl_Integer!(i8, u8);`), status extracted.
2. Find the target from the resolved name binding of the call's macro path (the same one the expander used), by matching macro_source_absolute_path + macro_source_range_start to the MACRO_RULES span that starts there. Cross-check: every `definition` token origin of that call's expanded row lies inside that span; a mismatch is a typed error, not a guess.
3. Store the matched arm index (macro_matched_arm_index) on the edge evidence if the edge record has room for it, because it tells an agent which arm of the definition to edit.
4. A call whose macro did not resolve (definition in another crate with no source, e.g. core) gets one unresolved `expands` row with reason external source missing, if the call is a node. Calls inside function bodies are not nodes today, so they get no row in 5c.
5. `Expands` is structural like `Generates`: it is not collapsed into dependency edges.

# What I want from you

A. Is the resolved binding the right source for the target, or should it be the token origins, or both? Find the exact published stage table and columns that hold "macro call path -> resolved binding" after a run (name the file and line where it is written and where the graph projection could read it). If no published table holds it, say so and say what the smallest addition is.
B. What exactly should the evidence site be: the macro path, the whole call, or the `!`? Where is the path range stored for a MACRO_CALL span?
C. Is there room on HldEdgeInputRecord for the matched arm index, and if not what is the smallest honest way to carry it?
D. What must the core validator check for an `Expands` row (mirror the `Generates` checks; list each one)?
E. Which existing tests will break when a 9th... (count them) relation kind is added (exhaustive matches, DuckDB enum text maps, Turso publication)? List files and lines.
F. Propose the real-source acceptance assertions for itoa (exact counts per definition: how many of the 15 calls point to each of the 3 definitions; check it in src/lib.rs of crates/parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree).
G. What did I fail to ask?

Give file paths and line numbers for every claim about the code. Separate what you verified by reading from what you infer. Be concrete and brief.


## Output (verbatim)

I read the part-5 file, the PRD decisions, the span extractor, the namespace codec, both graph projections, the core validator, both Turso codecs and the itoa source. Verdict up front: the lean is sound in substance (binding as target source, structural kind, unresolved rows for source-less macros), but it has two factual errors — the count of relation kinds (it is the 10th, not the 9th) and the meaning of `macro_source_range_start` (it is the macro *body* start, not the definition node start) — and one gap: no published table today holds the per-call binding, so "find the target from the resolved name binding" needs a small writer change before any projection can read it. Details below; **verified** means I read the lines, **inferred** means reasoning on top of them.

## A. Source of the target: the resolved binding. But it is not published per-call

**Binding, not token origins.** Verified: the expander resolves the definition through the binding (`crates/parseltongue-syntax-span-extractor/src/persisted-macro-expansion-adapter.rs:305–345`: `binding.binding_target_identity_key` → `read_macro_target_metadata` → `target_macro_definition_record`). Token origins (`crates/parseltongue-syntax-span-extractor/src/syntax-namespace-input-extraction.rs:1049–1058`) carry the definition's path and *body-relative absolute* offsets per token — enough as a cross-check, but they are absent for builtins and `include!` (`:855–879`: `macro_token_origin_rows: vec![]`) and never name the definition. So: binding as the source of truth, origins as the cross-check — your lean 1+2, confirmed.

**No published table holds "this call → this binding" today.** Verified:

- The per-call binding exists only transiently as `expansion_resolved_macro_rows` (`crates/parseltongue-duckdb-analysis-adapter/src/namespace-macro-alternation-rounds.rs:113–156`) and can be re-derived only inside the run-local DuckDB analysis DB (`namespace-frozen-macro-lookups.rs:44–85`, `namespace-macro-candidate-lookup.sql`), which is discarded after the run.
- Published product tables (`crates/parseltongue-cli-dispatcher-binary/src/namespace-pipeline-publication-facade.rs:24–28`): `module-scope-binding-table` holds bindings keyed by scope+name with `binding_target_metadata_value` (written `crates/parseltongue-duckdb-analysis-adapter/src/namespace-final-output-records.rs:196–224` — the metadata includes `target_macro_definition_record`), but nothing keys a binding to a call site.
- `path-resolution-site-records` **excludes** macro paths: macro requests get no `namespace-path-metadata-records` row (`crates/parseltongue-duckdb-analysis-adapter/src/frozen-namespace-source-codec.rs:421–431` appends metadata only for `syntax-namespace-input-paths`; the macros arm is `:455–471`), and `namespace-final-path-products.sql` filters `WHERE p.metadata IS NOT NULL OR i.request_key IS NOT NULL`.
- `macro-expanded-item-table` (published, `crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs:36`) is per-call but names no definition (writer: `syntax-namespace-input-extraction.rs:1072–1080`).

**Smallest honest addition (inferred, two options):**
1. *Preferred:* add one field to the expanded `macro-expanded-item-table` row — the binding's `binding_target_identity_key` (plus the resolved name), threaded from `persisted-macro-expansion-adapter.rs:339–346` into `project_macro_expansion_rows`. One writer line region, no consumer breaks (the projection reads named fields only).
2. Alternative: give macro requests path metadata so their outcomes land in `path-resolution-site-records`. Larger blast radius: the receipt digest and many tests parse that table (`workspace-cli-contract-tests.rs:2929–2969`, `itoa-source-acceptance-tests.rs:486,1294,2087`).

**How the target becomes a node identity.** Verified precedent: impl headers never trust the path row's target key directly; they join binding *metadata* against `syntax-namespace-input-items` (published, `stage-table-store-backend.rs:50`) and take the unique `binding_target_identity_key` (`crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs:72–80, 293–306`). For macros the same table holds the MACRO_RULES binding with its occurrence key. The binding target occurrence and the MacroDefinition node occurrence are built from the same node, file, bytes and digest (`build_source_occurrence_key`, `syntax-namespace-input-extraction.rs:468–505` vs `build_original_item_identity`, `crates/parseltongue-duckdb-analysis-adapter/src/original-item-graph-projection.rs:292–321`), so they should be equal; the validator's `MissingLocalEndpoint` check is the typed error if they are not. Note (inferred): on itoa this equality is proven for call targets (2 resolved `calls` rows pass validation) but not yet for a macro binding — the first RED test should assert it.

## B. Evidence site: the whole call, and the path range is stored nowhere

Verified: the MACRO_CALL span row covers the whole call (`crates/parseltongue-syntax-span-extractor/src/lib.rs:498–507`: `span_byte_start_offset/end` = node range); `invocation_macro_path_segments` carries names without ranges (`syntax-namespace-input-extraction.rs:1661–1701`); the label uses the whole call text (`item-node-label-text.rs:65`). So a path-range site must be *derived* by parsing the call text.

Recommendation (inferred from verified precedents): **site = the whole MACRO_CALL occurrence**, because (a) both existing name-resolution edges site at the whole syntactic unit — `imports` at the whole `use` statement (`use-import-edge-records.rs:135–160`), `calls` at the whole call expression; (b) `generates` sites at the same call occurrence, so `expands` and `generates` from one call share a site and differ only in relation kind (the evidence map keys on the full `HldEvidenceRowKey`, `hld-materialization-core-records.rs:806–812`, so no conflict); (c) it lets the validator check `site == source occurrence` by identity, exactly like the `generates` check at `:709–717`. If you insist on the path bytes, they must be digest-validated against the file like every other site — but it buys little: the whole call is 20 bytes and the reader sees the name in it.

## C. Arm index: yes, there is room — `edge_dispatch_metadata_text`

Verified: `HldEdgeInputRecord.edge_dispatch_metadata_text: Option<String>` (`hld-materialization-core-records.rs:268–274`), already carries JSON metadata for calls, imports, implements, reads and type uses (`original-call-resolution-records.rs:238`, `use-import-edge-records.rs:161`, `impl-header-implements-records.rs:160`, `body-value-read-records.rs:309`, `signature-type-edge-records.rs:299`), validated only as non-blank (`:971–976`), and round-trips through Turso (`hld-graph-input-turso.rs:812, 1003–1004`). Smallest honest form: `{"macro_matched_arm_index":N}` there, cross-checked in production against the `macro-expanded-item-table` row's `macro_matched_arm_index` (`syntax-namespace-input-extraction.rs:1076`). No schema change. Caveat (verified): on itoa all three macros are single-arm (`pinned-itoa-source-tree/src/lib.rs:135–183, 191–210, 259–330`), so every index is 0 — a nonzero index needs a fixture test, not an itoa test.

## D. Validator checks for an `Expands` row

Verified generic checks already run for every edge (`hld-materialization-core-records.rs`): generation digest (`:648–651`); dispatch metadata non-blank and site identity re-buildable (`:968–980`); source is a node (`:655–663`); source category LocalItem/FilePublicArea/CrateRoot (`:664–678`); Extracted target is a node (`:679–690`); evidence key self-consistency — source, relation kind and target in the key match the edge (`:740–751`); target unit provenance (`:752–761`); site unit equals source unit (`:770–785`); no conflicting duplicate evidence key (`:806–812`).

New `Expands`-specific checks, mirroring the `Generates` block (`:700–718`) and the node-field check (`:654–673`):

1. Source node kind is `MacroInvocation` (`node_item_kind_value == Some(MacroInvocation)`).
2. Target node kind is `MacroDefinition` (exists: `hld-materialization-core-records.rs:157`; mapped at `original-item-graph-projection.rs:401`).
3. Site equals the source call occurrence (if you take B's recommendation), i.e. the same `*call == evidence_site_occurrence_key` identity check as `:709–717`.
4. Completeness mirror of "a generated node without a call is rejected": every `MacroInvocation` node has exactly one `expands` row, Extracted or Unresolved. (There is no node field for `expands`, so the invariant lives on rows — say so in the spec.)
5. Unresolved rows carry a typed reason and no target; the producer must not emit Unresolved when the binding metadata exists (producer discipline + acceptance test, A4).

## E. What breaks with the new variant — and it is the 10th, not the 9th

Count them (verified, `hld-materialization-core-records.rs:220–231`): Calls, UsesType, Imports, Reads, Implements, Reexports, Contains, Derives, Generates = **9 today; `Expands` is the 10th**. Also stale: PRD §1 rule 5 still says "eight kinds" — must be fixed in the same commit.

Compile-time breaks (exhaustive matches, no wildcard):
- `crates/parseltongue-duckdb-analysis-adapter/src/hld-materialization-duckdb-store.rs:1277–1289` (`encode_hld_relation_kind`)
- `crates/parseltongue-turso-store-adapter/src/hld-graph-input-turso.rs:1457–1469` (`encode_relation_value`)
- `crates/parseltongue-turso-store-adapter/src/hld-graph-publication-turso.rs:574–585` (`encode_relation_kind_json`)

Silent runtime breaks (wildcard `_ =>` error arms — the compiler will not find these; the facade's `assembled != input` readback at `full-graph-checkpoint-facade.rs:118–127` is the guard):
- `hld-materialization-duckdb-store.rs:1052–1065` (`read_hld_relation_kind`)
- `hld-graph-input-turso.rs:1471–1488` (`decode_relation_value`)

Count assertions that break when 15 rows appear:
- `crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs:2741` — `assert_eq!(input.input_edge_records.len(), 285)` becomes 300.
- `crates/parseltongue-duckdb-analysis-adapter/src/original-item-projection-tests.rs:366` — asserts 16 edges on a fixture whose source contains `m!();` (`:577`): breaks **if** the spec emits unresolved `expands` rows for item-level calls with no expansion row (see G).
- Stage line: not a break, but add `expands={}` to `full-graph-checkpoint-facade.rs:93` (R9 pattern; the log records it).

## F. Acceptance assertions for itoa (verified against the pinned source)

Counts from `crates/parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree/src/lib.rs`:

```text
definition                defined at   calls (lines)         expands rows
macro_rules! impl_Integer     135      185,186,187,188,189        5
macro_rules! impl_Integer_size 191     211,212,213,214,215,216    6
macro_rules! impl_Unsigned     259     329,330,331,332            4
                                                        total    15
```

Assertions for `test_expands_edge_row_counts` (model it on `test_generates_edge_row_counts`, `itoa-source-acceptance-tests.rs:2693–2741`):

1. Exactly 15 `Expands` rows; all `Extracted`; zero `Unresolved` (all three definitions are in-crate with source). Row count == MacroInvocation node count == 15 (completeness, D4).
2. Per-target counts 5/6/4, each target the `MacroDefinition` node whose span is that `macro_rules!` item; each source a `MacroInvocation` node; site == the call occurrence.
3. Dispatch metadata `macro_matched_arm_index == 0` on all 15 (single-arm macros — verified by reading all three bodies).
4. Cross-check (the lean's #2, corrected): for each row, every `definition`-kind token origin in the matching `macro-expanded-item-table` row has `token_original_byte_range` within the target MACRO_RULES span's `[span_byte_start_offset, span_byte_end_offset)` on `token_source_absolute_path` == the definition file. Note the origins are absolute into the definition file starting at the *body* (`syntax-namespace-input-extraction.rs:1049–1058`, `start = macro_source_range_start` of the body), so containment, not equality of starts.
5. Total edges 300 (updating the 285 assertion).

## G. What you failed to ask

1. **`macro_source_range_start` is the body start, not the node start.** Verified at `syntax-namespace-input-extraction.rs:655–665`: the record's range is the *last token tree* (the body). Your lean 2 says "match … to the MACRO_RULES span that starts there" — wrong as stated; the correct rule is "the MACRO_RULES span that *contains* that range". If you instead store the binding target key (A), this trap disappears.
2. **Item-level calls with no expansion row.** `m!();` in fixtures and, in general, calls whose expansion never ran (unmapped units, `include!` in `target/`, source barrier): are they `Unresolved` rows (reason?) or no rows? This decides whether D4's invariant holds and whether `original-item-projection-tests.rs:366` breaks. Decide in the spec; the itoa test cannot pin it (all 15 itoa calls expanded).
3. **Builtins and `include!`.** A builtin (`assert!`) or `include!` call at item position is a MacroInvocation node, but its "definition" has no MACRO_RULES span (builtin has no source; `include!` writes a virtual file with `macro_source_range_start: 0`, `syntax-namespace-input-extraction.rs:855–879`). Zero such nodes on itoa — fixture tests must pin the chosen form (I would store Unresolved `ExternalSourceMissing`, consistent with the core-targets decision).
4. **Expected-dataset impact.** The draft expected graph has no macro nodes at all (part-5 Q4), so 15 `expands` rows are "extra rows" until it is repaired — SOP A3 says no unexplained extras; the part file and log must say this explicitly, as 5a did.
5. **`#[macro_export]` double bindings.** An exported macro gets a second binding at crate root (`syntax-namespace-input-extraction.rs:718–736`); the metadata join must dedup by target key (the BTreeSet in the impl-header pattern does). No itoa impact (no `#[macro_export]` in itoa's lib.rs — verified), but state it.
6. **Nested expansion boundary.** A call written by a macro has an expansion row but no node (R4 not built). Say in the spec that 5c covers source-written calls only, so R4 does not silently change row counts later.
7. **Docs in the same commit:** PRD rule 5's kind list ("eight kinds"), the "Not built: `expands`" line under Decisions in force, the part-5 sub-part table (5c state), ARCH relation-kind list, and the log's node/edge numbers (285 → 300).
8. **Measurement** (Rule 3): wall time and peak memory of the three runs, and the stage-line `expands=` count.

Nothing in your lean conflicts with Rule 6 once A's writer change is made; the design rule argues for the binding key *and* the arm index on the row, both of which you already chose.

