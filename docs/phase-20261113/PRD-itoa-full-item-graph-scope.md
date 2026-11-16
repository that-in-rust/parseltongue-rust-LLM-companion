# PRD — full item dependency graph of itoa

## The answer

**This phase builds one thing: the full item dependency graph of pinned itoa 1.0.18, produced
deterministically by Rust code from the real source, with no LLM in production. Every item
outside a function body is a node, private ones included, and every dependency proven from
source is an edge with evidence. The phase is done when that graph equals the frozen expected
itoa graph.**

Out of scope for this phase: every other repository, any rust-analyzer export or oracle
comparison, and every higher-abstraction table (interface level, HLD, file and folder
roll-ups). Those are later queries over this graph, not part of it.

Status on 2026-11-14: **in progress, not passed.**

```text
itoa run today, sysroot excluded, marked incomplete by the code
nodes   45    43 items + 2 file areas                       draft expected: 73
edges  104    43 contains + 61 calls                        draft expected: 368
              2 calls resolved, 59 stored unresolved with a reason
kinds    2 of the 6 that itoa needs: contains, calls
proven   <Buffer as Default>::default --calls--> Buffer::new, from source through frozen readback
next     Buffer::format --calls--> private::Sealed::write
```

Proof procedure: [SOP](SOP-itoa-full-item-graph-acceptance.md). Requirements:
[SPEC](SPEC-itoa-full-item-graph/README.md). Design:
[ARCH](ARCH-itoa-full-item-graph-pipeline.md). Expected graph:
[itoa-target-dataset](itoa-target-dataset/). History: [log](LOG-itoa-full-item-graph-progress.md).

## 1 — What is built

One graph in two tables: `item-effective-visibility-records` (one row per node) and
`edge-evidence-record-table` (one row per edge fact). Each row links back to the source range
that proves it.

Graph rules:

1. Nodes: every module, function, struct, enum, trait, type alias, const, static and associated
   item, private items included. Nothing inside a function body. Impl blocks are not nodes.
2. One file-area node per source file, separate from its module node. It contains the file's
   top-level nodes.
3. Each `use` outside a function body is a node, with one `imports` edge per resolved binding.
4. An inline module is an ordinary module node with its own scope.
5. Every extracted edge is stored uncollapsed, private to private included. The edge type has
   ten kinds: `calls`, `uses_type`, `implements`, `reexports`, `contains`, `derives`,
   `imports`, `reads`, and the two structural kinds for macros, `generates` and `expands`.
   itoa needs eight of them; it has no `derives` or `reexports`.
6. A call, type use or read targets the real item. The `use` line is evidence, never the target.
7. The graph is the two tables above. No table is added.

What is never guessed:

- Unknown visibility and unknown reachability are stored as unknown.
- A site that cannot be resolved is stored unresolved with one typed reason.
- A generic or dynamic call targets the trait method, not a guessed implementation.

## 2 — How it is built

Ownership:

- **Rust** parses, expands macros, lowers signatures, infers within bodies, resolves calls and
  creates canonical identities.
- **DuckDB** does relational work on a copy of one published generation: name-resolution
  rounds, item and container projection, joins and deduplication.
- **Turso** holds durable rows, generations, frozen snapshots and atomic publication.
- No third database. No memory crosses a stage boundary; every stage reads and writes declared
  tables.

Stages:

```text
stage                                writes                                    state today
0   crate graph                      crate-dependency-graph-table              built
1   source ingest                    file-source-text-table                    built
2/3 syntax walker                    span, signature-detail, body-site rows    built
4+5 name resolution + item macros    bindings, path-resolution sites,          built
                                     macro-expanded items
6   signature-type-lowering-stage    lowered signatures, impl-header-index     impl headers only
7   semantic-edge-evidence-stage     edge-evidence-record-table                contains and calls only
-   item-effective-visibility-stage  item-effective-visibility-records         visibility stored unknown
```

The run for this phase is
`parseltongue map-workspace-full-run --mapping-final-stage-kind full-item-dependency-graph`.

Process boundary: production may start only `cargo metadata --offline`, `rustc --print cfg`
and `rustc --print sysroot`. It never compiles, never runs build scripts or proc macros, never
reads `target/`, and never uses the network.

## 3 — How it is proven

Only the SOP decides PASS. Unit tests are progress.

```text
repository  dtolnay/itoa 1.0.18, commit af77385d0daf4d0e949e81f2588be2e44f69f086
target      lib, edition 2021, x86_64-unknown-linux-gnu, no features, no build script
size        src/lib.rs 466 lines, src/u128_ext.rs 22 lines
```

Acceptance:

- **A1** The production binary runs on the pinned source with no pre-resolved input.
- **A2** Every expected node and edge is present with full identity and evidence range.
- **A3** No unexplained extra, missing or wrong node, edge or evidence row.
- **A4** Unresolved and inactive sites match the reviewed list. A reason does not excuse a
  resolvable missing edge.
- **A5** Source, configuration and executable digests, time, peak memory and disk are recorded.
- **A6** Frozen readback preserves evidence and generation identity; a cancelled or failed run
  leaves the previous output serving; output is identical at 1, 4 and 16 workers.

The expected graph is written from the pinned source, reviewed row by row and frozen before
comparison. Today's dataset is an unreviewed draft. It is comparison data only, never
production input.

Build order, one real-source test per relationship:

```text
done  <Buffer as Default>::default --calls--> Buffer::new
next  Buffer::format --calls--> private::Sealed::write          one generic edge, not twelve
      <Buffer as Clone>::clone --calls--> Buffer::new
      Integer --uses_type--> private::Sealed
      <u128 as Sealed>::write --> <u128 as Unsigned>::fmt --> div_rem_1e16 --> u128_ext::mulhi
      Buffer::new --reads--> <i128 as Integer>::MAX_STR_LEN
      Buffer::format --calls--> core::hint::unreachable_unchecked   the use node is evidence
      use nodes of lib.rs --imports--> core::hint, core::mem, core::mem::MaybeUninit, core::str
      file-area nodes --contains--> their top-level nodes
last  whole library equal to the frozen expected graph (A1–A6)
```

Method: expected rows live only in the test; fix the earliest missing producer; add only
general behaviour, nothing keyed on itoa names.

The next target must prove `i: I -> Integer -> private::Sealed -> write` and emit exactly one
generic target, with distractor, shadowing, ambiguity and rename checks.

## Reference

### Pins

- itoa: [phase-one-benchmark-pin.json](../phase-20261111/phase-one-benchmark-pin.json).
- Turso `=0.9.0-pre.5`, concurrent writes. A busy publication is tried up to 6 times, waiting
  10, 20, 40, 80, then 160 ms.
- DuckDB `=1.20000.1`, bundled, no extensions, no external access, run-local temp directory.

### Decisions in force

- **Calls** are resolved in Rust, reading persisted namespace and impl rows.
- **Nodes** are projected in DuckDB and written under `item-effective-visibility-stage`.
- **Inference:** build only what the current target needs; store the rest unresolved.
- **Expected graph:** written from the pinned source and reviewed. No oracle.
- **Containment:** each item has one lexical source container. File-level items and impl
  members belong to the file area; nested items belong to their inline module or trait. Impl
  identity, self type and trait are metadata, never a second parent.
- **Missing source (2026-11-14).** Codebases are expected to be partial or broken, so an edge
  is one of three things: *proven* (every candidate was visible in source); *proven, gap* (one
  target found, and the edge names the source that was missing, so it is correct if the code
  compiles); *unresolved* (zero or several targets, with one reason). A PASS run needs zero
  gap edges, or an explicit accepted list.
- **Memory:** measured and reported on every run; no cap in this phase.

- **Targets in `core` (2026-11-15).** Stored with a named external target: the name as far as
  itoa's source proves it, the crate, and a flag that the target's source was not read. The
  sysroot source stays out of the run. Chosen because it is the most explicit form for an
  agent reading the graph.
  - **What the name is (2026-11-16, REQ-RUST-053.0).** The path as the read source writes
    it. It claims "this item is outside the read source", never where the item is defined
    (a written path may be a re-export) and never which impl declares a member.
  - **Three kinds of name.** A path rooted at a crate with no unit in the run
    (`core::mem::MaybeUninit`; the crate is the root segment). A one-segment name that only
    the implicit prelude can bind (`Default`; no crate is claimed). A member reached
    through a builtin type that no read source declares (`u8::MAX`, `u128::from`; no
    crate and no impl are claimed).
  - **One node per written name.** The edge is stored as resolved with that node as its
    target. How each site reached the name (path root, `use` item, prelude, builtin
    member) is on the edge, so an import and a use of the same path share one node.
  - **No name is given** when a nearer scope could bind the first name differently, when a
    trait in read source declares a member of the same name, or for a method call. The row
    stays unresolved with its reason.
  - **A call by a written path (2026-11-16).** The row says `external_written_path` as how
    its target was chosen, so an agent can tell it from a call to a node of read source.
    `MaybeUninit::<u8>::uninit` targets `core::mem::MaybeUninit::uninit`: the written path
    with generic arguments dropped. Whether that is an inherent function is not claimed.
- **Macro definitions and calls (2026-11-15).** Both are nodes, for the same reason.
- **Design rule (user, 2026-11-15).** "Our intention should be to provide maximum
  information which helps the coding agent code better." When a graph design choice is
  open, the form that tells a coding agent the most is chosen and recorded here as decided.
- **One edge row per source site (2026-11-15).** Repeated sites with the same source,
  relation and target stay separate rows, each with its own range. Rows are grouped only
  when they are compared to the expected graph.
- **Macro-generated items (2026-11-15).**
  - Identity: the macro call plus the item's place in that call's expanded text. Two calls
    of one macro give two nodes.
  - Container: the file area or module where the call is written (one `contains` row), as
    for original impl members.
  - Generating call: named twice, as a field on the node and as a `generates` edge from
    the call node to the item (itoa: 45 rows). `generates` is a new relation kind.
  - A macro call points to the definition it expands by an `expands` edge (itoa: 15 rows).
    The target is the definition the stored textual-scope lookup names at the highest scope
    revision. The row is sited at the whole call and carries the macro name and the index
    of the arm that matched, so a reader knows which arm to edit. `expands` is a new
    relation kind (decided and built 2026-11-16, part 5c).
  - A generated item is an edge source like any other item. A row that starts in expanded
    text is sited at its occurrence in that expanded text, because that is the text the
    edge read. Its metadata names where its first and last token were copied from in the
    source file (definition or call), stored apart, so an agent knows where to edit
    (decided 2026-11-16, part 5d; built for `reads` and `calls` from generated function
    bodies and `uses_type` from generated function signatures, metadata key
    `expansion_source_origin`).
  - Every `calls` row carries the call's text, the callee's text, the call's shape
    (`call_shape_kind`) and how its target was chosen (`dispatch_kind`). A row without a
    target has `dispatch_kind` `undetermined` and says what stopped the lookup
    (`unresolved_detail_kind`), so an agent can tell "receiver type not determined" from
    "callee path never looked up" without reading the resolver. The typed reason is kept
    as it was (decided and built 2026-11-16, part 5d step 4, under the maximum-information
    rule).
  - A member reached through a type (`Self::NAME`, `u8::NAME`) that name lookup left
    without a target gets the impl member as target (`<u8 as Integer>::MAX_STR_LEN`, not
    the trait's declaration) only when the stored impl headers prove it: exactly one stored
    impl of that type, or of a type with no stored target, declares the name, and that one
    is a trait impl of the type with a stored trait, no generic parameter and no negation.
    The type is read from stored rows (the owner's impl header for `Self`, a Type lookup
    for any other prefix), never matched by text. Whether the trait is in scope is not
    checked. The row carries the lookup record with or without a target: every candidate
    impl, the outcome, and what was not read (for a builtin type its inherent items and
    trait impls in crates not read; for a type of the crate, blanket trait impls in crates
    not read), so an agent sees why a row has no target and what the target rests on
    (decided 2026-11-16 under the maximum-information rule; built in part 5d step 5 for
    `reads` and `uses_type`, not yet for `calls`).
  - A call whose callee is a path (`f(..)`, `m::f(..)`) gets its target from the stored
    name lookup of the whole callee path, and only when that lookup names exactly one
    function that is a graph node and the path is one name or is qualified by a module.
    Every call whose callee path was looked up carries the record of that lookup (state,
    reason, kinds of the symbols found, the qualifier's kind, the outcome), with or without
    a target (decided and built 2026-11-16, part 5d step 6).
  - A trait's method called through the trait (`Unsigned::fmt(x, ..)`) targets the trait's
    method declaration, which the path proves; the impl members that could run are listed
    as evidence, and one becomes the target only when the receiver's type is stored. A
    tuple struct called as a function (`DecimalPairs(..)`) targets the struct node with its
    own dispatch kind (both decided 2026-11-16 under the maximum-information rule; built
    2026-11-16, part 5d step 7). Each trait-method row lists the candidate impl members
    with the type each impl is for, and says that impls in unread crates were not read.
    When the first argument is the calling method's own by-value `self`, and that method
    is in an impl for a stored type, the target is the one impl member of the trait for
    that type (`dispatch_kind: trait_path_impl_method`); the row keeps the trait's
    declaration, the candidate list, the chosen member, and the impl and type that proved
    it (decided under the maximum-information rule; built 2026-11-16, part 5d step 8). A
    second stored impl that may be of the same trait for the same type leaves the row on
    the trait's declaration.
  - **Method call on a cast receiver (decided and built 2026-11-16, part 5d step 9).**
    `(x as T).name(..)` targets the one stored impl member `name` for `T`
    (`dispatch_kind: cast_receiver_impl_method`) when `T` is a stored struct, enum, union
    or builtin, and the member's trait is the trait of the impl the call is written in,
    which is what puts that trait in scope. Decided under the maximum-information rule:
    - The target is given although `core` is not read. For a builtin, an inherent method
      of that name in `core`, or another trait's impl in an unread crate, would change the
      answer; the row says so under `sources_not_read` instead of withholding a target
      that the read source proves.
    - The row carries the cast's type, every candidate impl, the chosen member, and the
      trait and type of the impl the call is in, so an agent can judge it from the row.
    - A row that fails one condition has no target and names the condition
      (`cast_type_not_stored`, `cast_member_unproven`, `cast_member_not_method`,
      `cast_trait_scope_unproven`, `cast_member_not_node`).
  - Built (part 5a): identity, container, and the generating call in both forms, for a
    macro call written in a source file. A macro call written by another macro is not a
    node yet; a run that meets one stops with a typed error and guesses nothing.
  - Not built: `expands` for a macro called by a path (`crate::m!`); a run that meets one
    stops with a typed error. itoa has none.
- **Node labels (2026-11-15, built in part 5b).** The label is display text copied from
  source; it is not the identity.
  - It starts with the crate target name and the file path (`itoa::src/lib.rs::`), not the
    cargo package id, which changes with the checkout. The package id stays in the identity.
  - An impl member names its impl header: `<u8 as private::Sealed>::write`, `Buffer::new`.
    An impl block is not a node, so this is the only place a reader sees the impl.
  - A macro call and a use item read as their own source text: `impl_Integer!(i8, u8)`,
    `use core::mem::{self, MaybeUninit}`.
  - A generated `const _` names its call: `const _ #impl_Integer!(i8, u8)`. Other
    generated items do not repeat the call text; they have the node field and `generates`.
  - Chosen by the design rule above. On itoa all 103 labels are now distinct (was 45).

- **Qualified paths (2026-11-16, built in part 5d step 10b).** `<T as Trait>::NAME` targets
  the member of the one stored impl of `Trait` for `T`, not the trait's declaration, when
  the stored impls prove exactly one. Chosen by the design rule: an agent changing
  `<u64 as Integer>::MAX_STR_LEN` must see that the `usize` const reads that one, not all
  twelve. The row keeps the trait's declaration, every candidate impl and the sources not
  read; when the member is not proven the target stays the declaration.

- **Arguments of assertion macros (2026-11-16, built in part 5d step 11).** A value path
  in the arguments of `assert!`, `debug_assert!` and the four `_eq` / `_ne` forms is a
  `reads` row of the item the call is written in. The arguments are parsed again as
  expressions; the macro is not expanded. Chosen by the design rule: an agent changing a
  const must see the assertion that checks it. The row says it was read from a macro's
  arguments, names the macro, and names what was not read (the macro's definition; a macro
  of that name imported by `use`). No row when a `macro_rules!` of that name is in
  textual scope, when the arguments do not parse, or for the name of a named format
  argument. Other macros' arguments are not read.

- **Non-function sources (2026-11-16, built in part 5d step 10a for `reads` and step 10c
  for `uses_type`).** A `uses_type` row from a const, static or type alias names the kind
  of that item (`declared_type_owner_kind`); a qualified path (`<u64 as Sealed>::Buffer`)
  is one row and each type inside it is one more, so an agent sees every type the item
  depends on. `reads` and
  `uses_type` start at any item a path is written in: a const or static (its initializer),
  a type alias or struct (an array length), not only a function. Chosen by the design rule:
  an agent changing `MAX_STR_LEN` must see that `Buffer`'s size reads it. A row whose
  target is not proven keeps the lookup record and the sources not read, and is never
  marked as external unless that is proven.

### Open

- **Targets in `core`, not built.** 119 of the 368 draft edges point into `core`. Resolving them needs
  the sysroot source in the run; measured on 2026-11-13, that run did not finish name
  resolution in 2,400 s (the run report is deleted). Runs today use
  `--sysroot-source-scope-kind excluded` and store those edges unresolved; the named external
  target is not built yet.
- **Macro-generated items, not built.** 45 of the 73 draft nodes come from `impl_Integer`,
  `impl_Integer_size` and `impl_Unsigned` calls. The rules are decided (above); the store
  already holds the 45 items as spans (measured 2026-11-15).
- **Enum variants and struct fields:** nodes, or evidence only.
- **File module and file area:** how a file's module node relates to its file-area node.
- **Edge reasons.** The code has two unresolved-reason lists: `SiteUnresolvedReasonKind` for
  sites and `HldUnresolvedReasonKind` for graph edges. One must win before unresolved edges
  are compared (A4).

The enum-variant, file-module and non-function-source points block the affected expected
rows from being frozen.

### Out of scope

- Any repository other than itoa.
- Any rust-analyzer export, oracle harness or oracle comparison.
- Interface level, HLD tables, file and folder roll-ups, coverage banner.
- Recovering code that does not compile; any LLM or training.
- Running proc macros or build scripts, or reading their outputs.
- Test, bench and example targets; feature exploration.
- Memory or speed optimisation.

Code for some of these exists in the repository from earlier work (the HLD builder, the
`original-top-level-containment` stage, the oracle comparison command). This phase does not
use, extend or test against it.
