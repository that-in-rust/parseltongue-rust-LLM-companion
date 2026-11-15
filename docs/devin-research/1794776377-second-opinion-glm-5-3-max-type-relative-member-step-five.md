# Second opinion: part 5d step 5, a member reached through a type (rule G5)

GLM-5.3 through pi, `--thinking max`, read-only tools. One run, 2026-11-16. It ran while the
code was being written, so it reviewed the design and the uncommitted code together.

## What I adopted, after checking the source

```text
1  For a builtin type, trait impls in crates not read are as unread as its inherent items.
   The row now records both.
2  A row left without a target still carries the lookup record: every candidate impl and
   the outcome kind (no_stored_impl_member, single_member_unproven,
   several_stored_impl_members). Before, such a row said only TraitSolverNeeded.
3  Rule G5 reworded to what the code does: any trait with a stored target, impls whose self
   type has no stored target count as candidates, the target must be a graph node, a function
   nested in a method has no Self, and whether the trait is in scope is not checked.
4  The pinned path totals: the 12 new prefix requests are counted apart (measured 12:
   11 builtins and 1 type parameter, as it predicted).
```

## Where I disagree

```text
It says a type of this crate has nothing unread, by the orphan rule. Not so: a blanket impl
(impl<T> Trait for T) in a crate that was not read applies to a type of this crate. The row
for a non-builtin type records "blanket trait impls in crates whose source was not read".
```

## Not done from its list

```text
Its fixture list has 8 cases at pipeline level. Built at the level of the lookup function:
inherent veto, unknown self type, two trait impls, kind match, target identity. Built on
syntax: the prefix request. Not built: an impl inside a function body, Self in a nested
function against Self in a closure, a prefix that is a use alias. Recorded as Q23.
Its claims about rustc (trait in scope, nested fn and Self) were not checked against a
compiler.
```

## Prompt (verbatim)

```text
You are reviewing a design, read-only. Do not edit any file. Repository: /home/ubuntu/repos/parseltongue-rust-LLM-companion (branch research/tiny-rust-code-mapper). Verification target: pinned dtolnay/itoa 1.0.18.

Read first: docs/phase-20261113/SPEC-itoa-full-item-graph/part-5-macro-generated-items.md section 10 (rules G1-G8, esp. G5), AGENTS.md Rule 6, crates/parseltongue-syntax-span-extractor/src/{body-value-read-records.rs,signature-type-edge-records.rs,impl-header-source-records.rs,original-call-resolution-records.rs,syntax-namespace-input-extraction.rs (around line 1480)}.

Measured today on pinned itoa: 16 'reads' rows and 5 'uses_type' rows have no target because the path is type-relative:
- reads: 5 'Self ::MAX_STR_LEN' and 5 'uN ::MAX_STR_LEN' in macro-generated '<iN as private::Sealed>::write' bodies; 5 'u128::MAX_STR_LEN' in the hand-written '<u128 as Unsigned>::fmt'; 1 'i128::MAX_STR_LEN' in 'Buffer::new'.
- uses_type: 'Self::Buffer' in the signature of '<uN as Unsigned>::fmt' (4 generated, 1 hand-written); the alias Buffer is a member of 'impl private::Sealed for uN', another impl of the same type.
The stored path rows for these are 'unresolved' with no reason and no candidates. The stored impl-header-index-table has 33 rows, each with impl_self_type_resolution.target_identity (e.g. builtin u8), impl_trait_type_resolution, and impl_member_occurrence_rows (name, syntax kind, occurrence key).

Proposed step 5 design (rule G5, made precise):
1. Self type T of 'P::NAME': if P is 'Self', T is the stored self type of the impl header whose member list contains the site's owner. Otherwise the namespace stage gets a new path request for the prefix of a two-segment body value path (context kind 'body_value_type_prefix', namespace Type), and T is its single stored target when that is a builtin, struct, enum or union. No text matching of builtin names.
2. Candidates: stored impl headers with self type identity == T, a trait resolved to a trait of this crate, not negative, no generic parameters, and a member named NAME of the needed kind (CONST for reads, TYPE_ALIAS for uses_type). If any inherent impl header of T (no trait) has a member NAME the rule does not fire.
3. Exactly one candidate: the edge target is that IMPL member (e.g. '<u8 as Integer>::MAX_STR_LEN'), not the trait's declaration. The draft expected graph also targets the impl member. The row's metadata records: the self type text and identity, the trait text, the impl header occurrence, and what was not read (for a builtin type: its inherent items in crate core, which would shadow the trait member).
4. Zero or several candidates: the row stays unresolved as today (no trait-default fallback in this step).
5. 'Unsigned::fmt(self, buf)' calls (prefix resolves to a trait) are NOT in this step.

Questions. Answer each with evidence from the files (path and line):
Q1. Is targeting the impl member (not the trait declaration) sound under Rust's rules given what is stored? Name any case in compiling Rust where 'exactly one stored trait impl of T with member NAME, no inherent impl in this crate with NAME' picks a wrong target (other than an inherent item in an unread crate, which the row records). Consider generic impls, blanket impls, cfg-inactive impls, impls inside function bodies, and impls in other modules not visible at the site (trait not in scope: does that matter for associated consts/types reached through a type path?).
Q2. Is the new prefix request the right way to prove what 'u128' names at the site, or does a stored row already prove it? Check whether adding a Type request for the qualifier could change any existing path-resolution-site-records row or its counts (the existing tests pin path totals).
Q3. For 'Self': is 'the impl header whose member list contains the owner' correct for a site whose owner is a function nested in a method body, or a closure? What is site_owner_occurrence_key for those?
Q4. Anything in this design that stores a guessed fact, or loses information a coding agent would need?
Q5. Smallest set of fixture tests that would each fail for a distinct wrong implementation.
Be concrete and brief. Mark each claim VERIFIED-IN-SOURCE (with file:line) or INFERENCE.
```

## Output (verbatim)

Review complete. Note: the working tree already contains this design implemented (`type-relative-member-records.rs` wired into both producers, and `check_body_value_prefix` in the namespace extraction), while the LOG/spec still say step 5 "not built" — so this review covers both the design and that code.

## Verdict

The design is sound: I found no case within read sources where "exactly one stored trait impl of T with member NAME, no inherent impl in the index with NAME" picks a wrong target. Two real defects: the `sources_not_read` gap is under-claimed for builtins (foreign *trait* impls are as unread as inherent items), and the pinned path totals in the itoa acceptance test will go red and must be updated as part of the step. Spec/code mismatches listed under Q4.

## Q1 — Is targeting the impl member sound?

**Yes.** The expected graph agrees (targets `<i128 as Integer>::MAX_STR_LEN`, `itoa-expected-graph-edges.json:310`, `<u128 as Integer>::MAX_STR_LEN` :693, `<i16 as private::Sealed>::Buffer` :1343). Rust resolves `T::NAME` by: inherent items of T first, then trait impls; ambiguity if several. Case by case:

- **Inherent impl in this crate with NAME (any kind):** becomes a second candidate → rule does not fire. VERIFIED-IN-SOURCE `type-relative-member-records.rs:85-97` (candidates are collected by name across all impls of T, kind checked only on the single survivor at :104-108). Name-only veto matches Rust: inherent items shadow by name across kinds. INFERENCE (Rust rule).
- **Generic impls:** two paths block them — self type with generic args is not a simple path → `resolve_type` returns unresolved (`impl-header-source-records.rs:325-327`, `type_simple_path_flag` from `extract_impl_header_syntax:495-507`), and *every* impl whose self type is unresolved is counted as a candidate for any T (`type-relative-member-records.rs:76-84`). Also `impl_generic_parameters_flag` is a proven condition (:105). Blocking is conservative, never wrong.
- **Blanket impls** (`impl<T> W for T`): self type `T` resolves to TypeParam, rejected by the Struct/Enum/Union/Builtin check (`impl-header-source-records.rs:366-369`) → unresolved → candidate → blocks. VERIFIED-IN-SOURCE. This one is *necessary*: ignoring it would fire on `S::NAME` while real rustc has two candidates.
- **cfg-inactive impls:** never stored — `persist_impl_header_rows` errors on an inactive IMPL span (`impl-header-source-records.rs:126-130`); since runs succeed with 33 rows, none is stored. VERIFIED-IN-SOURCE (code) + INFERENCE (that the field table holds only active IMPL rows). Side observation: a crate with an inactive impl block that *did* reach the field table would fail the run rather than skip — worth confirming that is intended.
- **Impls inside function bodies:** stored with `impl_function_body_ancestry_flag`, they can only *block* (candidate), never fire: a body-local member is not a graph node, and both producers filter the found target through `endpoints` (built from nodes with `node_function_body_ancestry_flag == false`, `body-value-read-records.rs:66-72, 275-285`; `signature-type-edge-records.rs:268-273`). VERIFIED-IN-SOURCE. Note the endpoints filter is load-bearing but G5/design point 2/3 never mention it — the spec should.
- **Trait not in scope:** for `T::NAME` in real Rust the trait must be in scope (the well-known E0599 "items from traits can only be used if the trait is in scope", e.g. `Vec::from_iter` without `use std::iter::FromIterator`). INFERENCE (rustc behavior; nothing in this repo pins it). It cannot produce a wrong single-candidate target: two in-scope-or-stored candidates → we block; exactly one stored candidate + trait not in scope → the code would not compile via that path at all. And itoa's own main case *requires* not checking scope: `Self::Buffer` in `<uN as Unsigned>::fmt` reaches `private::Sealed`'s member although `Sealed` is never use-imported at the site (`lib.rs:128-131`, `:255-258`) — rustc allows it via the impl's trait/supertrait scope rules. A scope-checking design would fail itoa. Not checking scope is correct given nothing scope-shaped is stored.
- **Coherence makes local types safe:** for a struct/enum/union of this crate, all impls (inherent and trait) are in this crate — inherent impls must be in the defining crate, foreign crates cannot impl foreign traits for local types. So `sources_not_read: []` for non-builtins is exactly right. INFERENCE (orphan rule), consistent with code `type-relative-member-records.rs:113-115`.

**The one residual wrong-target case:** for a *builtin* T, trait impls in unread crates (core, other deps) are as unread as inherent items. If a core trait impl of `u8` declared `MAX_STR_LEN` and were in scope at the site, real rustc could resolve there (or be ambiguous) while we fire at the crate impl. Nothing in the row records that gap — see Q4.

## Q2 — The new prefix request

**No stored row proves the prefix today; the new request is the right mechanism.** The full path `u128::MAX_STR_LEN` is requested in the Value namespace (`body_value_path`) and stored unresolved with no reason — pinned by `count_sites(..., Value::Null)` assertions (`itoa-source-acceptance-tests.rs:2199-2203`). The only Type-namespace prefix requests today are `body_call_type_prefix`, for call callees only (`original-call-resolution-records.rs:629-648`; 29 rows pinned at `itoa-source-acceptance-tests.rs:2156-2157`). `check_body_value_prefix` now adds the qualifier of a two-segment body value path as a Type request in lexical scope (`syntax-namespace-input-extraction.rs:1485-1507`), with `Self` excluded because a `Self` segment has no `name_ref` (`body-value-read-records.rs:426-437`). Builtins are proven by the prelude binding table (`syntax-namespace-input-extraction.rs:388-409`), so shadowing (a module or alias named like a primitive) is honored by the engine rather than by text — this is exactly why "no text matching" is right.

**Effect on existing rows/counts:** new rows only; request resolution is per-request, so existing rows keep their ranges and states. But the pinned totals move — this test will go RED and must be updated in the same step:

- `itoa-source-acceptance-tests.rs:2205` `paths.len() == 202` and `:2206` `count_state("resolved") == 164` — the new `body_value_type_prefix` rows are not partitioned out there. Predicted new rows, by site enumeration (PREDICTED, must be measured): 5 `uN` prefixes in the five generated signed `write` bodies, 5 `u128` in the hand-written `impl Unsigned for u128::fmt` (`lib.rs:347-360`), 1 `i128` in `Buffer::new` (`lib.rs:99`), and 1 `I` in `Buffer::format` (`lib.rs:110`, `I::MAX_STR_LEN` is a body value path; `I::Buffer` at :108 is a generic-arg type, not a PathExpr, so no request). So 202 → 214, resolved 164 → 175 (11 builtins + 1 TypeParam). The `I` row is unusable by `resolve_type` (TypeParam rejected) and correctly falls through to the existing bound walk (`body-value-read-records.rs:287-310`) — no behavior change for that read.
- The look-up join is sound: the producer synthesizes the prefix range `[site.start, site.start + prefix.len())` (`body-value-read-records.rs:188-199`) and the stored request row's range is the prefix path node's own range restored the same way (`syntax-namespace-input-extraction.rs:1495-1500`); both keys include file, range and chain (`impl-header-source-records.rs:81-90, 316-327`). Whitespace inside the site text (`Self ::MAX_STR_LEN`, Q20) is handled by trim (`read_type_relative_parts`, `type-relative-member-records.rs:22-35`); three-segment and `<T as Trait>` forms are excluded by the plain-identifier test.

## Q3 — `Self` and the owner

`site_owner_occurrence_key` is the nearest ancestor that is an `ast::Item` (`lib.rs:646-651`, `check_syntax_item_node` at `lib.rs:400-416`); impl member rows store `member_source_occurrence_key` from the same key builder (`impl-header-source-records.rs:479-486`), so `read_owner_self_type` matches exact keys.

- **Closure in a method:** closures are expressions, not Items, so the owner is the enclosing `fn` → found in the impl member list → Self = impl self type → fires. Correct: closures inherit the method's `Self`. VERIFIED-IN-SOURCE (key mechanics).
- **Function nested in a method body:** `ast::Fn` *is* an Item, so the owner is the nested fn itself; it is in no member list (members come only from the impl's assoc-item list, `extract_impl_header_syntax:477-486`) → `read_owner_self_type` → None → rule does not fire (`type-relative-member-records.rs:49-63` returns Option; `find_type_relative_const` `body-value-read-records.rs:170-206` yields None). This is correct, not merely conservative: a nested fn item cannot use the outer `Self` (nested items cannot reference outer generic parameters, and `Self` is one). INFERENCE (Rust semantics) — worth stating in the spec so a later refactor doesn't "fix" it by walking up.
- Same story on the signature side: `Self::Buffer` rows resolve the self type through `read_member_self_types`, keyed by the owner's identity (`signature-type-edge-records.rs:203-229, 250-265`); a nested fn's signature rows would find no entry → no edge.

## Q4 — Guessed facts and lost information

No guessed target is stored: the target is a stored member occurrence, and the row carries its proof (`self_type_identity`, trait text, `impl_header_occurrence`, `type-relative-member-records.rs:109-118`). Four gaps:

1. **Under-claimed gap for builtins (real defect):** `sources_not_read` records only "inherent items of builtin type u8". Foreign *trait* impls of the builtin are equally unread and equally able to shadow or conflict (Q1). Recommend: for builtin targets add "trait impls of builtin type u8 in unread crates". For local struct/enum/union targets the empty list is right by coherence — but a reader cannot tell "nothing unread" from "not computed"; record the claim.
2. **Spec/code mismatches.** (a) Design point 2 and PRD:200-203 say "a trait of this crate"; the code accepts any resolved trait (`resolution_state_kind == "resolved"`, kind checked only as `Trait` in `resolve_type`) — not unsound (any impl member is the right target), but spec and code disagree; align them. (b) The design omits that impls with *unresolved* self types also count as candidates (stronger conservatism than "inherent veto"); PRD/spec should say so. (c) `uses_type` fires only for prefix `Self` (`signature-type-edge-records.rs:250-252`) while `reads` fires for any prefix — an asymmetry itoa never exercises; state it. (d) The endpoints filter (body-local impls) is part of the rule but appears nowhere in G5.
3. **Blocked rows lose the candidates.** When zero/several candidates keep the row unresolved (design point 4), the metadata carries no candidate list or detail kind — an agent cannot tell "two impls both declare NAME" from "trait solver needed". Rule 6 favors recording them (a `type_relative_candidate_count` plus header occurrences would do).
4. **`impl trait of this crate` wording vs itoa's own case:** the member `Buffer` lives in an impl of `private::Sealed` while the site sits in `impl Unsigned` — the rule deliberately does not require the owner's trait and the member's trait to be related. That is right (Rust resolves by type + name), but G5's original "directly or through a stored supertrait row" wording suggests supertrait walking; the implemented name search across impls of T is simpler and at least as informative. Update G5's wording.

## Q5 — Smallest fixture set (each fails a distinct wrong implementation)

Existing fixture harness supports all of these (`persist_impl_header_rows` + synthetic path rows, e.g. `signature-type-edge-tests.rs:417`, `original-call-resolution-tests.rs:176-186`).

1. **Inherent veto:** inherent impl of S with member `N` + trait impl with `const N`; read `S::N` → unresolved. Fails: an impl that counts only trait impls as candidates.
2. **Blanket block:** `impl<T> W for T { const N }` + `impl A for S { const N }` → unresolved. Fails: an impl that ignores unresolved-self-type headers. (Distinct from 1: this one can be actually wrong, not just conservative.)
3. **Two trait impls, two traits, same member name** → unresolved. Fails: an impl that picks the owner's header's trait or "first match".
4. **Body-local impl only:** `Self::N` inside `fn f() { impl A for S { const N } ... }` → no edge (member not a node), row unresolved. Fails: an impl missing the endpoints filter.
5. **Nested fn vs closure pair:** `Self::N` in a nested fn → unresolved; `Self::N` in a closure in a method → fires at the impl member. Fails: ancestor-walking or closure-blind owner lookup respectively (two tests, opposite directions).
6. **Prefix by namespace, not text:** `use m::S as Alias;` then `Alias::N` → fires with m::S's identity. Fails: any text-to-builtin/name mapping.
7. **Target identity:** target occurrence equals the impl member's occurrence (not `trait A`'s own `const N` node), metadata names impl header + sources_not_read. Fails: trait-declaration targeting.
8. **Kind match:** signature `Self::N` where the single member `N` is a CONST → `uses_type` stays unresolved. Fails: name-only matching without `member_syntax_kind_value == kind` (`type-relative-member-records.rs:108`).

Plus, on real source: `test_generated_fmt_static_reads` (its "10 generated rows without a target" claim, spec 10.4) and `test_itoa_reads_row_counts` flip as the 16 reads gain targets; and the pinned path totals from Q2 (202/164) must be updated with the new `body_value_type_prefix` partition and count — predicted 12 rows, 11 resolving to builtins plus 1 TypeParam.

**Claims not checked against a compiler** (INFERENCE, flagged per Rule 1): the trait-in-scope rule for type-relative associated items; nested-fn inability to name `Self`; the orphan-rule argument for local types. None is verifiable from stored data; the first two only argue that the design is conservative where it could not prove, and itoa's compiling source (`lib.rs` is the pinned dtolnay/itoa 1.0.18) exercises the `Self::Buffer`-through-supertrait case that settles the scope question empirically.

