# Second opinion: part 5d, generated items as edge sources (GLM-5.3, pi, max reasoning, read-only)

Date: 2026-11-16. Files on the VM: `/home/ubuntu/zcode-runs/20261116-part-5d-generated-edge-sources-{prompt.txt,pi.md}`.
About 5 minutes. Asked before the 5d rules were written (AGENTS.md Rule 5).

## What I checked, and the result

```text
claim                                                    checked how                          result
all 136 generated calls read TraitSolverNeeded because   read original-call-resolution-       right
of one catch-all on two syntax flags                     records.rs lines 181-186
read_stored_reason_kind turns every stored reason but    read signature-type-edge-records.rs  right
two into TraitSolverNeeded                               lines 372-378
supertrait bound rows are indexed in trait_owner_keys    read original-call-resolution-       right
                                                         records.rs lines 284-294
test_expanded_body_read_counted inverts                  it failed; replaced by               right
                                                         test_expanded_site_owner_mismatch
the signature twin test pins "left to part 5"            read signature-type-edge-tests.rs    right
                                                         lines 241-255
read_edge_site_range returns only the byte pair and its  read the 10 callers; line 676        right; test_itoa_reads_
callers slice src/lib.rs                                 compares the pair alone              row_counts now filters
                                                                                              to original-text sites
the experiment's +34 reads are 24 with a target and 10   real-source test asserts 24 and 10   right
without
the validator blesses identities in expanded text        the run with 34 generated rows       right by behaviour; the
(lines 836-840) and exempts them from the file-area      passes the frozen read-back          lines were not opened
check (798-807)
codecs and frozen read-back carry the expansion chain    same run: 334 edges read back equal  right by behaviour
const initializers and type aliases may have no stored   not checked yet                      open; decides whether
body_value_path rows                                                                          step 6 is a producer or
                                                                                              a namespace-stage change
Self in generated signatures already resolves from       not checked yet                      open
impl-header-index-table rows
the Rust rule: inherent associated items shadow trait    marked INFERRED by the reviewer;     not checked against
items on a concrete type                                 known language rule                  rustc here
```

## What I take from it

- **Site.** The expanded-text occurrence stays the site of a generated-site row; the place in
  the real source goes in the row's metadata. The reviewer's reason is stronger than mine: the
  site's digest is of the text the edge read, and for `u8::MAX_STR_LEN` no contiguous range of
  `src/lib.rs` has that text (the first token comes from the call, the rest from the definition).
- **Origin shape.** Store the origin of the first token and of the last token separately, each
  with its kind. I had planned one merged range.
- **Order.** Build the origin helper before `uses_type` and `calls`, so each lands with it.
- **Trait members through a builtin type.** The rule may only name a trait member when a trait
  of the crate that the self type implements declares that name; it must not fire for `MIN`,
  `MAX`, `BITS` or `ilog10`, which are inherent items of `core`.

## Where I differ

- **Order, in part.** Step 1 (`reads` from generated function bodies) was already failing and
  then passing when the review came back, so it is committed without origin metadata and the
  origin helper is the next step, applied to `reads` first. The reviewer wanted the helper first.
- **The stage counter.** The reviewer calls `reads_in_expanded_bodies` meaningless after step 1.
  I keep it with a new meaning: stored `reads` rows sited in expanded text (34 on itoa).

## Prompt

```text
You are a second reader with no stake in the plan. Read-only review. Repository: /home/ubuntu/repos/parseltongue-rust-LLM-companion (branch research/tiny-rust-code-mapper). Read the repository yourself and cite path:line for every claim. Read first: AGENTS.md, docs/phase-20261113/SPEC-itoa-full-item-graph/part-5-macro-generated-items.md, docs/phase-20261113/PRD-itoa-full-item-graph-scope.md ("Decisions in force"), docs/phase-20261113/itoa-target-dataset/ (expected graph, draft), and the pinned source crates/parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree/src/lib.rs.

GOAL
A deterministic, source-backed full item dependency graph of pinned itoa for an external coding agent. User rule: "our intention should be to provide maximum information which helps the coding agent code better" - but never a guessed fact. Part 5d: items written by macro expansions (45 generated nodes: 5 const _, 12 MAX_STR_LEN, 12 Buffer type aliases, 12 write, 4 fmt) become edge SOURCES for reads, uses_type and calls.

MEASURED STATE (release runs on pinned itoa, commit 255163ea0)
- Stored today: 103 nodes, 300 edges (88 contains, 61 calls [3 resolved], 45 generates, 34 uses_type, 32 implements, 21 reads, 15 expands, 4 imports).
- Three producers in crates/parseltongue-syntax-span-extractor/src skip every site whose span_expansion_provenance_chain is not empty: body-value-read-records.rs (resolve_body_read_edges, about line 97), signature-type-edge-records.rs (build_signature_row_edges, about line 232), original-call-resolution-records.rs (build_original_call_edge, about line 109).
- Experiment (not committed): with only those three filters removed, the run completes, the validator accepts every row, and the store holds 502 edges:
  reads 55 (+34): 24 extracted <uN as Unsigned>::fmt -> DECIMAL_PAIRS (6 per fmt, 4 fmt); 10 unresolved TraitSolverNeeded in <iN/uN as private::Sealed>::write with site text "Self ::MAX_STR_LEN" and "u8 ::MAX_STR_LEN" (expanded text has a space before ::).
  uses_type 66 (+32): 12 extracted write -> "Self ::Buffer" = <T as private::Sealed>::Buffer; 12 write -> builtin str; 4 fmt -> builtin int; 4 fmt -> "Self ::Buffer" unresolved TraitSolverNeeded (impl Unsigned for uN; Buffer is declared in private::Sealed, reached through Unsigned: Integer: private::Sealed).
  calls 197 (+136): all 136 unresolved TraitSolverNeeded (10 from MAX_STR_LEN const initializers, 84 from fmt, 42 from write). The call row metadata carries call_syntax_evidence and dispatch_kind "original_associated_function" but NOT the callee source text.
- The site of each such row is an occurrence in expanded text: path .parseltongue/macro-source/<call digest>/expanded.rs, byte range in the expanded text, chain [call digest]. The expanded text is not a file on disk; it is in the store.
- Expected graph (draft, hand-derived) for generated sources: reads 56 grouped rows / 76 sites; uses_type 66; calls 80 grouped rows / 136 sites. It sites them in src/lib.rs inside the macro_rules! definition body. Node-target expected rows: fmt -> divmod100 (4), write -> <T as Unsigned>::fmt (10), write -> write (2), write -> slice_buffer_to_str (10), write/Buffer/MAX_STR_LEN -> <T as Integer>::MAX_STR_LEN (22), fmt -> DECIMAL_PAIRS (4), fmt/write/Buffer -> <T as Sealed>::Buffer (18). The rest target core.
- Not produced even by the experiment: reads and type uses whose source is a generated const (const _ initializer, MAX_STR_LEN type and initializer) or a generated type alias (Buffer = [MaybeUninit<u8>; Self::MAX_STR_LEN]).
- macro token origin rows exist in the store (used by part 5c macro-call-expands-projection.rs for definition-origin checks): each expanded token maps to a range in the macro definition or in the call arguments.

MY PLAN FOR 5d (leans; challenge each)
S1 reads from generated function bodies: remove the filter; +34 rows; evidence site stays the expanded-text occurrence.
S2 uses_type from generated function signatures: remove the filter; +32 rows.
S3 calls from generated bodies: remove the filter; +136 rows, unresolved for now; add the callee/call source text to call metadata (original and generated), and rename the dispatch kind so a generated call is not labelled "original".
S4 every row whose site is in expanded text also carries, in metadata, the range in real source text (src/lib.rs) its first and last tokens came from, with origin kind (macro definition body or call argument), read from the stored token origin rows. Reason: an agent edits src/lib.rs, not expanded text; and part 7 must compare with expected-graph sites in src/lib.rs. Lean: metadata on the edge row, evidence key unchanged.
S5 targets inside generated impls: "uN::MAX_STR_LEN" / "Self::MAX_STR_LEN" -> the generated assoc const of impl Integer for uN (16 rows incl. 6 original-body rows), "Self::Buffer" in impl Unsigned for uN -> <uN as private::Sealed>::Buffer via supertraits (4 rows). Lean: resolve by the stored impl headers for the builtin self type, state "extracted with a recorded gap": inherent items of the builtin type in core were not read (core source excluded), same form as the existing bound_read_evidence that records Copy was not read.
S6 non-function sources: const initializers / const types / type alias bodies as edge sources (reads, uses_type; calls already appear for const initializers).
5e afterwards: call targets (write -> fmt -> div_rem_1e16 -> mulhi; bare names, module prefixes, generic args).

QUESTIONS
1. Is the expanded-text occurrence the right evidence site for generated rows, with real-source origin in metadata (S4)? Or should the site be the real-source range and the expanded occurrence be metadata? Consider: one definition-body token is read by 5 expansions (5 different source nodes); the validator in crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs has site/source consistency checks; one-row-per-site rule.
2. Find anything in the validator, the DuckDB/Turso store codecs, the HLD collapse/builder, or the frozen readback that assumes an edge site or source has an empty expansion chain and would silently misbehave (not fail) once S1-S3 land. Cite lines.
3. S5: is "extracted with a recorded gap" honest for uN::MAX_STR_LEN given Rust's rule that inherent associated items take priority over trait items and core is not read? What exactly must the gap record say? Is there a case in pinned itoa where it would be wrong?
4. For "Self::Buffer" inside impl Unsigned for u8: what is the exact Rust resolution rule, and what must be read from stored rows to prove the target without a trait solver?
5. S3: the reason on all unresolved calls is TraitSolverNeeded, the default when no reason was stored. Which stored facts allow more precise typed reasons, cheaply? Look at how original calls get reasons.
6. Order: is S1,S2,S3,S4,S5,S6 right, or should S4 (real-source origin) come first because every later row depends on its shape?
7. What do existing tests pin that will change (name tests and lines)? In particular tests in crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs that slice src/lib.rs by an edge site range, and fixture tests in the span extractor that assert expanded sites give no rows.
8. Tell me what I did not ask.

Answer in under 1500 words. Mark each claim as READ (you opened the lines) or INFERRED.
```

## Output (verbatim)

I have read the repository. Review follows. (I am the second reader invoked with this prompt; all claims cite lines I opened.)

**Overall:** the plan is sound and matches stored facts. Two corrections: S4's origin metadata must carry *both* token-origin ranges, not one merged range; and the S1–S3 order should fold S4's helper in immediately, not after S3. Details below.

**Q1 — evidence site.** Keep the expanded-text occurrence as the site; real-source origin in metadata. READ:
- The occurrence identity is (unit, path, range, digest, chain) and the validator rebuilds it from parts (`hld-materialization-core-records.rs:1036-1050`); digest is of the site text. A real-source site for e.g. `u8::MAX_STR_LEN` would have to be the definition text `$Unsigned::MAX_STR_LEN` — a different string, so the digest-checked text identity (`impl-header-source-records.rs:229-236`) no longer names what the edge read. Mixed-origin sites are worse: `u8::MAX_STR_LEN`'s first token comes from the call argument, the rest from the definition — no contiguous real-source range has that text.
- The producer check "site text must be a slice of its owner's text" (`original-call-resolution-records.rs:140-168`) passes today for generated owners only because both live in expanded text; a real-source site against an expanded owner errors (loud, but wrong).
- The validator already blesses generated identities whose text lives in expanded source (`hld-materialization-core-records.rs:836-840`) and exempts expanded-chain sites from the file-area path check (`:798-807`).
- Five expansions sharing one definition range cause no key collisions either way (source is in the evidence key), so this is not the deciding factor; identity integrity is.
- Part 7 comparison: the expected graph sites generated-source rows at definition-body anchors in src/lib.rs (READ: `itoa-expected-graph-edges.json:286-313`, line 164). Grouped comparison needs each stored site's origin *listed*, which metadata supplies.

**Q2 — silent misbehavior.** Codecs and frozen readback are safe: the DuckDB occurrence encoder carries the chain (`hld-materialization-duckdb-store.rs:1116-1128`), Turso publication too (`hld-graph-publication-turso.rs:465-471`), the edge-evidence row key includes the full evidence identity so no collisions (`hld-graph-input-turso.rs:785-799`), and the checkpoint compares assembled == input (loud) (`full-graph-checkpoint-facade.rs:117-141`). Three real findings:
1. `read_stored_reason_kind` (`signature-type-edge-records.rs:372-378`): every stored reason except `external_source_missing`/`cfg_inactive` — including null and `unknown_receiver` — becomes `TraitSolverNeeded`. For generated rows this mislabels en masse; see Q5.
2. `resolve_type` silently returns `unknown_receiver` when the paths map has no entry for a site key (`impl-header-source-records.rs:281-321`) — if any upstream stage later stops storing path rows for expanded sites, reasons degrade without an error. Not triggered today (your experiment resolved fmt→DECIMAL_PAIRS), but it is the one silent path.
3. `read_edge_site_range` (`itoa-source-acceptance-tests.rs:601-607`) returns only the byte pair; callers slice `src/lib.rs` with it (`:2264-2268`). Expanded-text offsets are small enough to land *inside* src/lib.rs, so S1 rows produce wrong text slices and misleading assertion failures rather than clean panics — the helper must grow path+chain awareness.

Not affected: `hld-original-source-projection.sql:80-157` and `original-item-graph-projection.sql:11` exclude expanded spans from *node* projection only; `macro-call-expands-projection.sql:38,69` constrains expands rows only.

**Q3 — the gap is honest if worded exactly.** Rust resolves `Self::NAME` on a concrete type by searching inherent associated items first, then trait items of traits whose impls/bounds are in scope (INFERRED — language rule, not citable here). Since core is excluded, "no inherent `MAX_STR_LEN` on `u8`" is unprovable, so "extracted with a recorded gap" is the correct state. The gap must say: the self type (`u8`), what was not read (inherent associated items of the builtin type; sysroot excluded), the precedence rule that makes the target conditional (inherent shadows trait), and — for `Buffer` — that prelude/core traits in scope were also not read. Where it would be *wrong* in pinned itoa: the same path shapes `u8::MIN`, `i128::BITS`, `u8::MAX.ilog10()` in the generated `const _` asserts (READ: pinned lib.rs lines ~155-163) — `MIN`/`MAX`/`BITS`/`ilog10` genuinely resolve to inherent core items, not Integer items. So S5's rule must only fire when an in-scope itoa trait declares the member name; it must not invent trait targets for inherently-named items. The existing `bound_read_evidence` precedent (READ: `body-value-read-records.rs:232-258`, `missing_source_traits`) is the right form.

**Q4 — `Self::Buffer` in `impl Unsigned for u8`.** Rule (INFERRED): candidates are inherent items of `u8`, then associated items of every trait `T` where `u8: T` is nameable in scope and `T` declares `Buffer`; unique survivor wins. Stored facts that prove it without a solver, all READ: impl-header self-type resolutions map members to builtin types (`signature-type-edge-records.rs:196-217`); supertrait bound rows with `bound_subject_kind: "supertrait"` are indexed (`original-call-resolution-records.rs:290-301`, `BoundMethodLookupIndex.trait_owner_keys`); the trait `Sealed` and its `type Buffer` member are itoa source nodes; and `u8: Sealed` comes from the *generated* `impl private::Sealed for u8` header row — note the `impl Unsigned for u8` row alone does not prove it. Extend the existing bound walk from methods to associated types; do not write a new resolver.

**Q5 — cheaper typed reasons.** READ `original-call-resolution-records.rs:181-207`: original calls get `AmbiguousTarget` from multi-target path rows, `CfgInactive`/`ExternalSourceMissing`/`TraitSolverNeeded`/`MissingTarget` from the receiver's stored `unresolved_reason_kind`, and the catch-all at `:183-186` fires when `call_supported_path_flag != true || call_lexical_shadowing_flag != false`. That catch-all is why all 136 read `TraitSolverNeeded`. Cheap precision: (a) the flags are already in the stored `call_syntax_evidence` for expanded sites — compute them; (b) run the receiver-path switch on the expanded sites' `path-resolution-site-records` rows (they exist; the key includes chain, `impl-header-source-records.rs:281-292`); (c) bare-name calls (`divmod100`, `slice_buffer_to_str`) should resolve outright — `endpoints` now contains the generated fn nodes, and 5e is exactly this. Also copy `site_syntax_source_text` into call metadata, mirroring `read_source_text` in reads (`body-value-read-records.rs:192-198`).

**Q6 — order.** Change it: build the S4 origin helper *first* (one function reading `macro_token_origin_rows`, READ shape at `declarative-macro-expansion-records.rs:16-31` and `macro-call-expands-projection.rs:126-153`), then land S1, S2, S3 each *with* its origin metadata and the dispatch-kind rename inside S3's commit. Reasons: every later step (S5 gaps, S6, part 7) reads that metadata; adding it later re-touches all three producers and re-shapes already-frozen rows; and small-commit rules favor one producer-touch per step anyway. One correction to S4's shape: for mixed-origin sites, store *both* the first and last token origin ranges with their origin kinds — a single merged first..last range spans call argument to definition body and matches no real text.

**Q7 — tests that change.** READ: `test_expanded_body_read_counted` (`body-read-edge-tests.rs:248-273`) asserts zero edges and `read_expanded_site_count == 1` — inverts; the signature twin at `signature-type-edge-tests.rs:240-254` ("left to part 5") inverts; `test_itoa_reads_row_counts` (`itoa-source-acceptance-tests.rs:2256-2303`) pins 21 rows, slices src/lib.rs by site offsets (`:2264-2268`), and pins `TraitSolverNeeded` with "no reason was stored" — all three break. Unaffected (filtered by original owner or by relation): `test_decimal_pairs_static_read` (`:714-745`), `test_format_sealed_write_call` (`:499`), `test_generated_const_node_contained` (`:2417`), `test_generated_item_node_counts` (`:2540`), generates/expands counts (`:2696`, `:2748`). The stage line counter `reads_in_expanded_bodies` (`full-graph-checkpoint-facade.rs:106,109`) becomes meaningless — update it in the same commit.

**Q8 — not asked.**
1. The experiment's +34 reads already include the 24 `fmt → DECIMAL_PAIRS` rows the expected graph wants *and* the 10 `Self ::MAX_STR_LEN`/`u8 ::MAX_STR_LEN` reads your plan defers to S5 — check whether S1 as scoped will emit those 10 unresolved and say so in the spec, or the RED test will surprise you.
2. S6's gap is structural, not a filter: reads require PATH_EXPR sites with `site_path_context_kind == "body_value_path"` (`body-value-read-records.rs:142-158`); const initializers and type-alias bodies likely have no such rows — verify what the namespace stage stores for those owners before speccing S6, or S6 becomes a namespace-stage change, not a producer change.
3. `read_member_self_types` keys by member occurrence identity — generated impl members are in `impl-header-index-table` (5a R8), so `Self` in generated signatures is already resolvable from stored rows; say so in the S2/S5 design rather than adding a lookup.
4. Sites with expanded text like `Self ::MAX_STR_LEN` (space before `::`) will appear in metadata text; pin the exact expanded text in a fixture so label/metadata formats don't drift.

**Where I disagree with the plan as written:** the S4 range shape (merged first..last) and the S4-after-S3 ordering; both argued above. Everything else I confirmed against the lines cited.
