# Second opinion: part 5b node labels (GLM-5.3, pi, max reasoning, read-only)

Date: 2026-11-15. Harness: `~/bin/zpi` (pi, `--thinking max`, read-only tools). One run, about 5 minutes.

## What I took, after checking it in source

- One string label, not typed parts. Checked: no production code parses the label
  (`hld-materialization-duckdb-store.rs`, `hld-graph-input-turso.rs`, `hld-graph-publication-turso.rs` store it verbatim).
- Call text only on generated items that have no name.
- Normalize whitespace myself; there is no stored normalized path text.
- Crate target name as the prefix. Checked: `CrateCompilationUnitKey.unit_cargo_target_name`
  (`parseltongue-core-row-types/src/lib.rs:84-90`).
- The two 5a tests that find `const _` by `ends_with("::_")` must find it by kind.
- Fallback when the header has no readable self text (`impl-header-source-records.rs:483` uses `.ok()`).
- The `r#` strip stays.

## Where I disagree

- **Cut rule.** The review says cut a call's label at the first newline. I cut at 160
  characters after joining lines, because rustfmt writes long `use` lists and attributes
  on several lines and a first-newline cut would leave `use a::{ …`. On itoa the two rules
  give the same labels (all 15 calls and 3 use items are one line).

## Not checked by me

- The claim that the expander appends tokens one `push_str` each
  (`declarative-expansion-source-facade.rs:275`). The stored text does read `private ::Sealed` (measured).
- The review counts a fourth `use` in `src/u128_ext.rs:2`. It is under `#[cfg(feature = "no-panic")]`,
  which is off, so the store has 3 use nodes (measured).

## Prompt

```text
You are a read-only design reviewer. Do not edit files. Repository: the current directory (branch research/tiny-rust-code-mapper). Use the codebase-memory tools and read files to check every claim; cite file and line.

# Subject: part 5b, readable node labels in the itoa full item graph

Goal of the product: a source-backed dependency graph that a coding agent reads to write code. Rule in force (AGENTS.md Rule 6): when a design choice is open, take the form that gives a coding agent the most information, and never store a guessed fact.

## Measured today (pinned itoa 1.0.18, commit 4ea998560)
- 103 nodes, 285 edges. Node label field: `node_qualified_path_text` on `HldNodeInputRecord` (crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs).
- Labels are built in crates/parseltongue-duckdb-analysis-adapter/src/original-item-graph-projection.rs (about line 113) and generated-item-graph-projection.rs (about line 108) as
  `<cargo package id>::<file path>::<enclosing MODULE and TRAIT names>::<declared name, or the syntax kind if none>`.
- Only 45 of 103 labels are distinct. Collisions:
  - 15 macro calls all read `...::src/lib.rs::MACRO_CALL`
  - 3 use items all read `...::src/lib.rs::USE`
  - 5 generated `const _` all read `...::src/lib.rs::_`
  - 12 generated `MAX_STR_LEN`, 12 generated assoc type `Buffer`, 12 generated `write`, 4 generated + 1 original `fmt` (impl members carry no impl segment)
  - original impl members read `...::src/lib.rs::new`, `::format`, `::default`, `::clone`
- Identity is NOT the label: every node has a typed identity key (unit, file, byte range, digest, kind, expansion chain). Generated nodes also carry `node_generating_call_reference` and a `generates` edge from their macro call.
- `impl-header-index-table` rows carry `impl_self_type_resolution.type_source_evidence.type_source_text` (e.g. "u8", "Buffer") and `impl_trait_type_resolution.type_source_evidence.type_source_text` (e.g. "Integer", "Default"; in expanded text it reads "private ::Sealed" with a space, as the expander prints tokens). Both projections already receive the matching header per impl member (`impl_bridges`).
- The draft expected graph (docs/phase-20261113/itoa-target-dataset/itoa-expected-graph-nodes.json, field `label`) uses: `<u8 as private::Sealed>::write`, `Buffer::new`, `<Buffer as Default>::default`, `use core::hint`, `use core::mem::{self, MaybeUninit}`, `const _ #impl_Integer(i8,u8)`, `u128_ext::mulhi`, `private::Sealed::write`. It has no macro call or macro definition nodes yet.

## My proposed rule (lean)
Keep the prefix `<package id>::<file path>::<module and trait names>`. Then:
1. Impl member, original or generated: add one segment from its impl header before the name: `<SelfText as TraitText>` for a trait impl, `SelfText` for an inherent impl. Text is the header's source text with whitespace around `::` removed. Example: `itoa::src/lib.rs::<u8 as private::Sealed>::write`, `itoa::src/lib.rs::Buffer::new`.
2. Macro call: last segment is the call's own source text without the trailing `;`, whitespace runs collapsed to one space: `itoa::src/lib.rs::impl_Integer!(i8, u8)`.
3. Use item: last segment is its source text without the trailing `;`: `itoa::src/lib.rs::use core::mem::{self, MaybeUninit}`.
4. A generated item with no name of its own (`const _`): `const _` followed by its call: `itoa::src/lib.rs::const _ #impl_Integer!(i8, u8)`.
5. Other generated items do NOT get the call text in the label (they have the field and the edge).
6. No truncation. Labels need not be unique; the acceptance test for itoa pins that all 103 are distinct.
7. The label stays one string field. No new columns.

## Questions. Answer each with evidence, then say what I failed to ask.
A. Is one string the right carrier, or should the label be split into typed parts (impl self text, impl trait text, call text) as separate node fields so an agent can filter without parsing? What does Rule 6 favour, and what would it cost in the files that store and read nodes (DuckDB store, Turso graph input, publication)?
B. Rule 5: should every generated item carry its call text in the label, for uniformity, or only the unnamed ones?
C. Whitespace: the expander prints "private ::Sealed". Is "remove whitespace around ::" safe and sufficient, or is there a stored normalized path text I should use instead? Name the field and file.
D. Long calls: a macro call can be hundreds of lines. Is "no truncation" right for a label, given the call's full text is already reachable by its span? Propose an exact rule if not.
E. Does anything else read `node_qualified_path_text` and depend on today's form (collapse, roll-ups, comparison, HLD levels, tests)? List file and line for each reader that would break or change meaning.
F. The package id prefix is `path+file:///abs/checkout/path#1.0.18`, which changes between checkouts. Should the label use the crate name instead? Where is the crate name available in the projection?
G. Which 5 real-source assertions would you pin in the itoa acceptance test for this part?

Be concrete and brief. Mark each claim as CHECKED (you read the lines) or GUESS.

```

## Output, verbatim

```text
All checks done. Here is the review.

# Part 5b review — node labels

## Your measured claims, verified

- Label form `<package id>::<file path>::<module/trait names>::<name-or-kind>` — **CHECKED**: `original-item-graph-projection.rs:113-123` (name falls back to syntax kind at :117-118), `generated-item-graph-projection.rs:105-117` (CONST falls back to `_`). Lexical names are MODULE/TRAIT ancestors only — **CHECKED** `original-item-graph-projection.sql:43-46`; an impl member's `contains` source is therefore the file area (:64-69 selects container MODULE/TRAIT only; `original-item-graph-projection.rs:107-113`).
- 15 macro calls — **CHECKED** in pinned source: 5×`impl_Integer!` (lib.rs:185-189), 6×`impl_Integer_size!` (:211-216), 4×`impl_Unsigned!` (:329-332). 3 use items in lib.rs (:56-58) plus 1 in u128_ext.rs (:2). Stored-node counts (103/285/45 distinct) — **REPORTED**, not re-measured (read-only review).
- Header text fields — **CHECKED**: `impl-header-index-table` rows carry `impl_self_type_resolution` / `impl_trait_type_resolution` (`impl-header-source-records.rs:153-159`), each `{resolution_state_kind, unresolved_reason_kind, target_identity, type_source_evidence}` with `type_source_evidence.type_source_text` = raw byte slice (:483, :314-316, :374-381). Both projections already receive the full header payload per impl member via `impl_bridges` (`original-item-graph-projection.sql:47-52`), validated to exactly one bridge (`original-item-graph-projection.rs:332-355`).
- "45 of 103" appears as a live pinned count — **CHECKED** `itoa-source-acceptance-tests.rs:2686` (`assert_eq!(named, 45)`).

## A. One string vs typed parts

One string is the right carrier **for the label's job**, but note the real gap your rule 1 exposes: the published graph has **no member→impl linkage at all**. Impl blocks are records, not nodes; the member's `contains` edge comes from the file area; `node_owner_kind_value: Impl` is the only hint. The `implements` edge carries `self_type_source_text`/`trait_source_text` only as self-type→trait metadata (`impl-header-implements-records.rs:144-148`), not per member. So the label segment is currently the *only* place an agent can see which impl a member belongs to — your rule 1 is justified under Rule 6, not merely cosmetic.

Typed parts as separate node fields would cost: new fields on `HldNodeInputRecord` (`hld-materialization-core-records.rs:203-216`, dep-free crate), the 12-column DuckDB insert (`hld-materialization-duckdb-store.rs:495-506`) plus its CREATE TABLE, the published-row copy (:636-641), Turso input JSON write/read (`hld-graph-input-turso.rs:765`, `:922`), Turso publication payload (`hld-graph-publication-turso.rs:310-318`), and validation (:811 checks only non-empty today). That is ~6 files for filterability the agent already has in another form: the typed data exists per impl in `impl-header-index-table` keyed by `impl_semantic_occurrence_key`, which the bridge already matches to the member. **Recommendation**: one string now; backlog a typed `node_impl_header_reference` field (the bridge's header occurrence key) if filterability is later wanted. Do not reuse `node_signature_text` for it — it exists, is plumbed end-to-end, is always `None` today (**CHECKED**: no `node_signature_text: Some` in `crates/`), and should stay reserved for a signature.

## B. Call text on all generated items, or only unnamed?

Only the unnamed ones — your lean is right. Named generated items already carry the call as typed data: `node_generating_call_reference` (`generated-item-graph-projection.rs:131`) plus a `generates` edge (:133-152). Putting call text in their labels duplicates reachable information while making every label long — Rule 6's test is "helps the coding agent code better," not "maximize bytes per row". The draft dataset agrees: generated named labels like `<u8 as private::Sealed>::write` (nodes.json:1059) carry no call text (**CHECKED**).

## C. Whitespace

There is **no stored normalized path text** — **CHECKED**. The candidates are `type_source_text` (raw slice, `impl-header-source-records.rs:483`), `type_simple_path_flag` (:484), and the resolved `target_identity` (node reference, not text). So you must normalize the raw text yourself, as display-only text (raw stays in evidence — fine under Rule 1). "Remove whitespace around `::`" is sufficient for the itoa case (`private ::Sealed` → `private::Sealed`); I could not fully verify the expander's token spacing (tokens are appended one `push_str` each, `declarative-expansion-source-facade.rs:275` — the space claim is **REPORTED**). To be safe for expanded generics, define the rule as: collapse whitespace runs to one space, then drop spaces adjacent to `::`. Do **not** substitute the resolved target's name for the written text: the label should say what the source says; the resolution is already an edge.

## D. Long calls

"No truncation" is wrong for a label. The full text is provably reachable via the call's byte range + digest in its identity, so truncation loses no proven fact, while a 300-line label actively harms scanning (and bloats every Turso row). Exact rule I'd propose: take the call's source text; strip trailing `;` and trailing whitespace; collapse whitespace runs to one space; if the result contains a newline, cut at the first newline and append ` …`. All 15 itoa calls are single-line (**CHECKED** lib.rs:185-216, 329-332), so the acceptance outcome is identical under either rule — pick the bounded one now, before a multi-line call arrives.

## E. Readers of `node_qualified_path_text`

No production code parses or matches on the label; every non-test reader stores/publishes it verbatim — **CHECKED** by grep across `crates/`:

- `hld-materialization-duckdb-store.rs:504` (insert), `:638` (published copy) — opaque.
- `hld-graph-input-turso.rs:765` (write), `:922` (read, only `required_string`) — opaque.
- `hld-graph-publication-turso.rs:315` (publish) — opaque.
- `hld-original-source-projection.rs:259, 341, 360` — a **different stage** (`original-top-level-containment`) with its own label form `{package}::{module path}::{name}` (no file segment). Its exact form is pinned by `itoa-source-acceptance-tests.rs:1763-1766`. Leave this file alone and that test is unaffected.

Tests that break or change meaning with your rules:

- `itoa-source-acceptance-tests.rs:2493` and `:2680` — both `ends_with("::_")` to find the `const _` node. **Rule 4 breaks both**; :2680 is also how the test locates the call text today. Must be rewritten to find `const _` by item kind + owner, not label suffix.
- `itoa-source-acceptance-tests.rs:1697-1699, 1911-1913, 1924-1926, 1951-1953, 2563-2564` — `ends_with("::{name}")`; safe while the declared name stays the last segment (your rules keep it last except for use/macro-call/`const _` nodes, which these lines don't target).
- `original-item-projection-tests.rs:244, 281, 378`, `generated-item-projection-tests.rs:222` — `ends_with` on fixture labels; names stay last, should pass, but fixture MACRO_CALL/USE label expectations in those files will change text.
- `original-call-resolution-tests.rs:221, 265, 270, 367, 381, 889` and `use-import-edge-tests.rs:65`, `impl-header-implements-tests.rs:37, 125, 193` — build their own inputs with their own labels; independent of the projection. **CHECKED**.

## F. Crate name vs package id

Yes, use the crate (target) name. It is already in hand: both projections hold a `CrateCompilationUnitKey` (`context.unit` / `area.unit`) whose sibling field is `unit_cargo_target_name` (`parseltongue-core-row-types/src/lib.rs:84-90`; used right next to the identifier at `original-item-graph-projection.rs:119`, `generated-item-graph-projection.rs:111`). `unit_cargo_target_name` is the name rustc gives the crate, which is what an agent reasons with, and it matches the dataset's root label `itoa` and the acceptance test's canonical unit (`itoa-source-acceptance-tests.rs:87-89`). The checkout-dependent package id stays on `node_target_identity`, so nothing is lost. Caveats: a package with a lib and a bin of the same target name yields two units with equal crate names (the file-path segment keeps labels distinct in practice), and the published unit row already carries `cargo_target_name` (`hld-graph-publication-turso.rs:451-453`) so the join exists.

## G. Five real-source assertions to pin

1. **Distinctness**: all 103 `node_qualified_path_text` values are pairwise distinct (BTreeSet len == node count), and no label contains the checkout path (`file:///` / workspace root) — pins F.
2. **Original impl members carry the header segment**: exact labels `itoa::src/lib.rs::Buffer::new`, `::Buffer::format`, `::<Buffer as Default>::default`, `::<Buffer as Clone>::clone`.
3. **Generated impl members carry the expanded header segment**: exact `::<u8 as private::Sealed>::write`, `::<u8 as Integer>::MAX_STR_LEN`; the 12 `write`, 12 `Buffer`, 12 `MAX_STR_LEN` label sets are each 12-way distinct.
4. **Macro-call and use labels equal source text**: for each of the 15 macro-call nodes and 3 lib.rs use nodes, the last label segment equals the pinned file's own byte-slice at the node's range (trailing `;` stripped) — the test slices the real file, so nothing expected is hardcoded.
5. **`const _` labels name their call**: the 5 generated `const _` nodes each end with `const _ #` + their own generating call's label segment, cross-checked against the `generates` edge target.

## What you failed to ask

- **Dataset alignment.** The draft's `const _ #impl_Integer(i8,u8)` (nodes.json:456-584) has no `!` and no space after the comma; your rule 4 produces `const _ #impl_Integer!(i8, u8)`. Also the dataset labels carry **no** `package::file` prefix at all, while your rule keeps it. Decide now which form wins and how acceptance compares stored labels to `label` (exact vs suffix match), or the S1 freeze will churn this part twice.
- **Fallbacks for rule 1.** `type_source_text` can be `null` (non-UTF8 slice, `impl-header-source-records.rs:483` uses `.ok()`), and negative impls have a null trait resolution (`impl_negative_implementation_flag`, handled at `impl-header-implements-records.rs:107`). The label rule needs a typed fallback segment (self text only, or the kind) — never a guess (Rule 1).
- **Raw identifiers.** Today's code strips `r#` (`original-item-graph-projection.rs:123`); your rules are silent. itoa has none, so this is untested either way — state that the strip stays.
- **Process, not design**: this is a graph-form choice — record it in the PRD "Decisions in force" and the backlog in the same commit (Rule 6), write the part file first (Rule 3), and new helpers need four-word names with REQ-RUST doc comments.

```
