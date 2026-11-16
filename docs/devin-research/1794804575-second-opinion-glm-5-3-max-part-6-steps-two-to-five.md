# Second opinion — GLM-5.3 (thinking max) — part 6 steps 6.2 to 6.5, built code

- Asked: 2026-11-16, epoch 1794804575, at commit f600a7954. Read-only run (`~/bin/zpi`), 498 s, exit 0.
- Why: the review of the built steps 6.2 to 6.5 was owed at the end of part 6.
- The reviewer could not compile or run code. Each claim used below was checked by running
  the release executable on a small crate that compiles (see "What I checked").

## Prompt (verbatim)

```text
You are a read-only reviewer of committed Rust changes in this repository (branch research/tiny-rust-code-mapper, HEAD f600a7954). Do not edit files. Review steps 6.2 to 6.5 of part 6 of the itoa full item graph: named external targets (REQ-RUST-053.0, rules N1..N8). A named external node is a WRITTEN name outside read source (for example `core::mem::size_of`, `Copy`, `u8::MAX`); it has no file, kind or signature and does not claim the item's defining path.

Read:
- docs/phase-20261113/SPEC-itoa-full-item-graph/part-6-targets-in-core.md (rules N1..N8, the matrix, "Step 6.2/6.3/6.4/6.5 as built", open questions)
- crates/parseltongue-syntax-span-extractor/src/external-named-target-records.rs : build_external_proof_value, decode_external_name_record, collect_trait_member_names, collect_external_endpoint_nodes
- crates/parseltongue-syntax-span-extractor/src/type-relative-member-records.rs : find_builtin_member_name, find_admitted_member_target, read_impl_member_index
- crates/parseltongue-syntax-span-extractor/src/original-call-resolution-records.rs : find_builtin_call_member and the code around its call (the `builtin_member` value, the dispatch kind arms, where a written-path external callee is taken from the stored lookup)
- crates/parseltongue-syntax-span-extractor/src/body-value-read-records.rs : find_type_relative_const and the block that calls find_builtin_member_name
- crates/parseltongue-syntax-span-extractor/src/signature-type-edge-records.rs : resolve_type_path_target and the external-name branch (step 6.3)
- crates/parseltongue-syntax-span-extractor/src/impl-header-implements-records.rs : read_header_external_name (step 6.4)
- crates/parseltongue-duckdb-analysis-adapter/src/namespace-final-path-products.sql : the external name proof (path_root_names_crate, use_import_names_path, implicit_prelude_name) and its glob / binder conditions
- crates/parseltongue-duckdb-analysis-adapter/src/namespace-final-output-records.rs : read_external_name_record, build_external_row_value
- crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs : validate_hld_external_key, is_hld_builtin_type
- tests: type-relative-member-tests.rs (test_builtin_member_external_name), callee-path-call-tests.rs (test_builtin_member_call_target, test_external_callee_names_target), body-read-edge-tests.rs (test_builtin_member_read_target), signature-type-edge-tests.rs (test_external_type_names_target, test_written_path_last_name), impl-header-implements-tests.rs (test_external_trait_names_target), published-generation-copy-tests.rs (test_external_name_proof_rows), crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs (test_itoa_external_path_calls, test_itoa_external_type_uses, test_itoa_prelude_name_targets, test_itoa_builtin_member_targets)
- crates/parseltongue-cli-dispatcher-binary/tests/pinned-itoa-source-tree/src/lib.rs (the real source)

The repository rule: an edge target must be proven from stored source evidence; a row that cannot be proven keeps no target, with a record of why. A wrong target is a defect; a missed target is not. For an external node, "wrong" means: the written name on the node is not the name the site's path denotes under Rust name resolution (for example a local item, import, glob import, macro-generated item, generic parameter or local trait member could supply that name instead), or the row claims more than was read.

Questions. Answer each with file:line evidence, and say plainly when you could not verify something.
Q1 (6.5 soundness). Is there a Rust program, compiling on stable, where a call `T::name(..)` or a read `T::NAME` gets the external node `<builtin>::<name>` although the member is declared in READ source? Consider: an inherent-looking member supplied by a local trait impl for the builtin (`impl Tr for u8 { const MAX }`), by a blanket impl `impl<X> Tr for X`, by a macro-generated impl or trait, by a trait in another module or behind `use .. as`, by a trait declared inside a function body, a trait with a DEFAULT member the impl does not override, a supertrait member, a local `mod u8`, `struct u8` or `type u8` or `use x as u8` shadowing the builtin name, a generic parameter named like a builtin, `Self` inside `impl Tr for u8` and inside a trait, a type alias to a builtin (`type Word = u64; Word::MAX`), a raw identifier, and `<u8>::MAX` / `<u8 as Tr>::MAX`.
Q2 (6.5 guards). collect_trait_member_names reads the names local traits declare from node rows. Is there a trait member in read source that is NOT a node row at the time each of the three producers (reads, calls, signatures) builds the index, so the N5 block misses it? Are members of traits written by macro expansion included?
Q3 (6.2 written-path calls). Find a case where a call `a::b::c(..)` gets an external node whose written path is not what the path denotes: the first segment bound by a nearer `use`, a local module, an extern crate rename, a glob import, a macro-expanded `use`, an enum variant or associated function reached through an imported TYPE (`MaybeUninit::<u8>::uninit`: is `core::mem::MaybeUninit::uninit` an honest written name?), generic arguments in middle segments, `crate::`/`self::`/`super::` roots.
Q4 (6.3 type paths, 6.4 prelude names). Find a case where a type path or a trait in an impl header / bound gets an external name wrongly: a generic parameter named `Copy`, a local trait `Clone` in an outer or sibling module, a glob import that could supply the name, `#![no_implicit_prelude]`, edition differences in the prelude, a derive-generated impl, a macro_rules-generated `use`. Is the SQL condition "only request_kind = 'import' rows count as glob imports or binders" too weak anywhere (does any other stored request kind bind a name in a scope)?
Q5 (node shape). collect_external_endpoint_nodes makes one node per distinct external key. Can two different sites produce keys that differ only in kind for the same written text (for example PreludeName `Copy` vs a CrateRootedPath or BuiltinMember), or the same key for names that denote different items (two modules each importing a different `mem`)? Is merging by written name acceptable given the node claims no defining path?
Q6 (tests). Any test listed above that cannot fail, has a wrong expectation, or a missing case that would let a wrong target through? Name the three most valuable missing cases.
Q7 (method calls). 138 method calls remain without a target; the receiver's type is not proven. Without reading `core`, is there any subset whose written target name can be proven from itoa's own source alone (for example a receiver that is a parameter with a written builtin type, or a field of a local struct), and what would the honest node name be? Say what must be refused.
Keep the answer under 1200 words. Number the answers Q1..Q7.
```

## Answer (verbatim)

```text
# Review: part 6 steps 6.2–6.5 (HEAD f600a7954, read-only)

**Q1 (6.5 soundness).** I could not construct a compiling counterexample. The name is given only when three guards all pass: the stored self type's symbol kind is `Builtin` (impl-header-source-records.rs:366-379 — a text match never decides this), the member lookup over stored impl headers returned `no_stored_impl_member` (type-relative-member-records.rs:169-171, outcome at 337-344), and no trait node declares the name (type-relative-member-records.rs:172-176). Checked cases:

- Local trait impl for the builtin, impl of an external trait, blanket impl: all produce a candidate row — the candidate scan includes headers with the same resolved self type *or any unresolved self type* (type-relative-member-records.rs:300-305), so every stored impl (generic, negative, body-local) with that member name yields an outcome ≠ `no_stored_impl_member` → refused. Body-local impls are stored with `impl_function_body_ancestry_flag` (impl-header-source-records.rs:507) and are not filtered out of the scan.
- Trait in another module, behind `use .. as`, default member, supertrait member, trait in a fn body: guarded by the name set when the trait's members are nodes (see Q2); a body-local trait supplying the member to `u8` also requires a body-local impl, which is in the index — so a compiling program is covered.
- `struct u8` / `mod u8` / `type u8` / generic param named `u8`: builtins are bound at the prelude scope (syntax-namespace-input-extraction.rs:388-400), so any nearer binding (modules, aliases, TYPE_PARAM bindings at 1716/645-646) wins; `resolve_type` then sees a non-Builtin symbol kind → unresolved → no name. I verified the injection point, not the resolver's precedence arithmetic.
- `Self` in `impl Tr for u8`: read from the owner header (body-value-read-records.rs:232-234); naming it `u8::NAME` is the documented decision. `Self` in a trait → no owner impl → refused. `<u8>::MAX` / `<u8 as Tr>::MAX`: refused, `read_type_relative_parts` demands a plain identifier (type-relative-member-records.rs:52-66) and qualified paths go through the stored-impl path. `type Word = u64; Word::MAX`: either resolves to the alias (refused) or names `u64::MAX`, which is what the path denotes — a naming-policy question (spec "Step 6.5 as built"), not a wrong target under N2.

Residual unverifiable: macro-expanded *traits* (itoa's macros emit impls only) and prelude-vs-binding precedence. Neither is reachable in pinned itoa.

**Q2 (6.5 guards).** All three producers build the index from the same `graph.input_node_records` (body-value-read-records.rs:89-95, signature-type-edge-records.rs:146-152, original-call-resolution-records.rs:83-89), i.e. the completed node stage input — no ordering gap inside a run. `collect_trait_member_names` does *not* filter `node_function_body_ancestry_flag` (external-named-target-records.rs:86-93), so body-local trait members are included whenever they are node rows; that they are produced (not just fixture-constructible, signature-type-edge-tests.rs:283-286) I could not verify from source. Macro-expanded impl members are demonstrably captured (the itoa tests label reads `const _ #impl_Integer!(i8, u8)`, itoa-source-acceptance-tests.rs:4438); a macro-expanded *trait* member is untested and unverified. This is the one gap worth a test.

**Q3 (6.2).** Every wrong-name vector I checked is refused: nearer `use`/local module (proof 2 requires no nearer binder, namespace-final-path-products.sql:132-140); glob imports block the whole chain (114-116, 138-140); `crate::`/`self::`/`super::` roots have their own segment kinds ("crate"/"self"/"super", syntax-namespace-record-codec.rs:285-291) and fail `plain_flag` (SQL:68-69), so no name is composed from them; extern-crate renames aren't among the five names → refused. Generic middle segments are dropped (`mem::size_of::<Self>` → `core::mem::size_of`, itoa test at 4492) — lossy but honest. `MaybeUninit::<u8>::uninit` → `core::mem::MaybeUninit::uninit` is honest: the import's path is proved crate-rooted and `uninit` denotes that associated function. Two weaknesses: (a) a leading `::` path skips the whole chain check (`first_name_position=1`, SQL:101-107) — correct for 2018+ extern-prelude semantics, wrong for edition 2015 where `::core` is crate-root-relative; itoa is unaffected. (b) a `macro_rules!`-generated `use`/glob: if expanded imports are not recorded as import requests, a macro-generated glob could supply a name that still gets a proof — unverified, no test.

**Q4 (6.3/6.4).** Generic parameter named `Copy`: TYPE_PARAM is a type-namespace binding, so the prelude proof's "no scope on the chain declares the name" check blocks it (SQL:152-155). Local `Clone`/glob: blocked by binder/glob CTEs. `#![no_implicit_prelude]`: `unit_implicit_prelude_flag: !has("no_implicit_prelude")` (sysroot-prelude-context-rules.rs:26) → no prelude scope → refused. Edition differences in the prelude: the name claim ("this written name is external") holds for any edition's prelude; the prelude scope construction is upstream and I did not verify edition awareness — say so. `request_kind='import'` is not too weak: the only stored source kinds are `"import"` and `"signature"` (namespace-contract-input-codec.rs:119,140), and signature requests (trait bounds) bind nothing — the supertrait-binder regression is tested (published-generation-copy-tests.rs:608-618). Priorities are only 2/3 (namespace-contract-input-codec.rs:200; namespace-unit-link-inputs.rs:175), so `priority>=2` (SQL:79) excludes nothing.

**Q5 (node shape).** Key = (kind, crate, path) per generation (external-named-target-records.rs:131-152). Kind/text collisions are structurally prevented for honest proofs: a prelude name is one segment, a builtin member is exactly two, a crate-rooted path's root is one of five crate names. Two modules importing different crates (`core::mem` vs `std::mem`) give different crate fields → different nodes; two modules importing the *same* written name share a node that claims only that written name — acceptable given N2 claims no defining path. `collect_external_endpoint_nodes` emits no file/kind/owner/visibility/signature and validation enforces shape (hld-materialization-core-records.rs:1023-1033). Acceptable.

**Q6 (tests).** All listed tests assert concrete rows against a real pipeline run over pinned itoa (`run_original_graph_fixture`) or against real decode logic; none is vacuous. Expectations I spot-checked (7 calls incl. `MaybeUninit::<u8>::uninit`→`core::mem::MaybeUninit::uninit`, 4 prelude rows, 39 builtin-member rows with `no_stored_impl_member` and empty candidates, 138 unresolved, 145 nodes) match the source. Three most valuable missing cases: (1) SQL-level proof that a binder or glob in an *enclosing* scope (depth > 0 on the recursive chain, SQL:37-42) withholds — only same-scope blockers are tested (published-generation-copy-tests.rs:582-606); the chain walk is the riskiest untested logic. (2) A macro-expanded (or body-local) trait declaring the member must withhold the builtin name — covers the Q2 dependency. (3) A `macro_rules!`-generated `use`/glob — if unrecorded, a prelude or crate-rooted name could be wrongly proven.

**Q7 (method calls).** With current stored evidence, none of the 138 is provable. Receivers are: `self` of macro-parameter type (`self.unsigned_abs()`, `Unsigned::fmt`), integer literals (`999.try_into()`, `1_00_00.try_into()`), inference results (`.expect`, `.unwrap`, `.len` on `i.write(..)`'s return), `Self::Buffer` aliases, and indexed/coerced receivers. The only subset provable *in principle* from itoa's source: `.write(..)` on `buf[offset]` where the enclosing parameter writes `buf: &mut [MaybeUninit<u8>]` (enc_16lsd, impl_Unsigned) — the element type is fully written and `MaybeUninit` is import-proved, so an honest node would be `core::mem::MaybeUninit::write` (crate-rooted, composed through the `use`) — but it needs parameter-type propagation plus one indexing/autoderef step, machinery this phase does not have. Must be refused: receivers typed by generics or macro parameters, literals, inference-only receivers, and `[u8]`/`[u8; 200]` receivers (`DECIMAL_PAIRS.0.get_unchecked`) — `[u8]` is not in `HLD_BUILTIN_TYPE_NAMES` (hld-materialization-core-records.rs:7-10) and the unsize coercion is not written, so no honest builtin-member name exists for them. Recording the method name + reason on those rows (N6, as today) is correct.
```

## What I checked, and what it showed

Three one-file crates, each checked with `cargo check`, run with the release executable
(`--sysroot-source-scope-kind excluded`, final stage `full-item-dependency-graph`), rows read
from `edge-evidence-record-table`. Sources: `/home/ubuntu/phase6-work/probe-gaps{,2,3}/p/src/lib.rs`.

```text
written                                                       stored row                         verdict
u8::MAX, trait Gen { const MAX } written by a macro           no target, name in local trait     right
u64::from(1u8), same trait declares fn from                   no target, name in local trait     right
u8::MIN                                                       external u8::MIN                   right
u8::BITS, fn-body trait Loc { const BITS } with no impl       external u8::BITS                  right (no impl supplies it)
{ use crate::fake as core; { core::mem::size_of() } }         no target, prefix shadowed         right
core::hint::spin_loop() under a macro-written use .. as core  the local function                 right
core::mem::size_of::<u8>()                                    external core::mem::size_of        right
u8::FOO, fn-body trait and impl that declares FOO             no target, single_member_unproven  right
u16::make(), fn-body trait and impl that declares make        no target, member_lookup_unproven  right
u64::BAR2, trait with a default const and impl<T> .. for T    no target, name in local trait     right
u8::FOO2, fn-body trait with DEFAULT const, impl Loc for u8 {}   external u8::FOO2               WRONG
u16::mk(), fn-body trait with DEFAULT fn, impl<T> Loc4 for T {}  external u16::mk                WRONG
u64::FOO5, same with the impl in a nested block                  external u64::FOO5              WRONG
```

## Findings

- **The reviewer's Q1 verdict is wrong in one case, and I found it by running code, not from
  the review.** It says a trait in a function body "also requires a body-local impl, which is
  in the index — so a compiling program is covered". An impl that takes the trait's default
  member declares no member of that name, so the member lookup says `no_stored_impl_member`;
  the trait is not a node, so its member names are not in the withheld set. The row got an
  external name for a member that is local. Three such rows, above. itoa has no impl in a
  function body, so no itoa row was wrong.
- **Fixed** (REQ-RUST-053.0 N5, fifth condition): a stored trait impl whose self type is this
  builtin or was not resolved, and whose trait was not resolved, withholds the name
  (`external_member_trait_not_resolved`).
- **Measured on the way:** the stored header of an impl written in a function body has neither
  its self type nor its trait resolved, even for `impl Loc for u8`. So one such impl
  withholds the builtin-member names of the whole crate. That misses targets and gives no
  wrong one.

## Adopted

- Its test gap 2 (a macro-written or function-body trait that declares the member): built as
  the real-pipeline test `test_external_name_shadow_cases`, which also holds the three wrong
  rows above as fixed.
- Its Q7 reading of the 138 method calls: none provable with what is stored; `buf[offset].write(..)`
  with the written parameter type `&mut [MaybeUninit<u8>]` is the one shape provable in
  principle. Recorded as an open question, not built.

## Not built, recorded as open questions in the part file

- Gap 1: a lookup-level test with a binder or glob in an enclosing scope. The pipeline case
  (`use` in an enclosing block) is covered by the test above through another mechanism.
- Gap 3: a `use` or glob written by a macro, as an import request. The one case run
  (a macro-written `use crate::fake as core`) leads to the local function.
- A leading `::` path under edition 2015, and whether the prelude scope depends on the edition.
