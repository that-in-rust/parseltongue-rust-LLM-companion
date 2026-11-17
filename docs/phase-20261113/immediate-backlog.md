# Immediate backlog

**Work that is decided to be worth doing next, but is not part of the acceptance run in
progress. One entry per idea, newest last. Nothing here is built, and nothing here changes
the frozen expected graph until its own expected rows are written and frozen.**

Longer-term and undecided work is in
[BACKLOG-deferred-work-open-decisions.md](BACKLOG-deferred-work-open-decisions.md). Status
is on the "Where we are" page of [the log](LOG-itoa-full-item-graph-progress.md).

## 1. Link a trait method to the methods that implement it

Added 2026-11-16, at the user's request, from a review of the frozen graph as a navigation
aid for a coding agent.

**The gap.** A walk along `calls` edges stops at a trait's declaration and never reaches
the code that runs.

```text
Buffer::format --calls--> private::Sealed::write        the declaration in the trait
private::Sealed::write --(nothing)--> the 12 bodies     <i8 as private::Sealed>::write ..
```

**Measured in the frozen expected graph (151 nodes, 682 edge sites):**

```text
trait member                 impl members that are nodes   edge from the trait member to them
private::Sealed::write       12                            none
private::Sealed::Buffer      12                            none
Integer::MAX_STR_LEN         12                            none
Unsigned::fmt                 5                            none
```

- The only edges that leave these four trait members are `uses_type` edges of their
  signatures.
- The 41 impl members are nodes, and each impl block is a stored record, so the link can be
  derived from rows the mapper already stores. Not measured: how much code that takes.
- `Default::default` and `Clone::clone` each have one impl member; their traits are in
  `core`, which is not mapped, so there is no trait-member node to link from.

**What the link says.** "This impl member is the implementation of that trait member for
this self type." It does not say the impl member is called. `Buffer::format` keeps its one
`calls` edge to `private::Sealed::write`; it does not gain 12 `calls` edges.

**Open before any code:**

- **The relation kind.** The graph has ten kinds and none means this. `implements` today
  joins a type to a trait. Either a new kind, or `implements` between two members. A
  decision for the user.
- **Direction.** From the impl member to the trait member (like `implements`), with the
  reverse found by query. Proposed, not decided.
- **Expected rows first.** 41 rows, written from the pinned source and frozen, before the
  producer. Then the whole graph is re-frozen and S1 to S4 run again.

## 2. Link the crate and each module to their source files

Added 2026-11-16, at the user's request.

**The gap.** A walk that starts at the crate finds no source file.

```text
node         outgoing edges in the frozen expected graph
itoa         none
u128_ext     none
```

- The two file-area nodes exist (`area:src/lib.rs`, `area:src/u128_ext.rs`) and hold the
  items: `area:src/lib.rs` has 82 `contains` edges, `area:src/u128_ext.rs` has 1.
- Nothing joins the crate to its file areas, or the module `u128_ext` to the file that
  holds its items.

**Open before any code:** the relation kind. An item has one lexical container (PRD), so
this is a link between a module and its file, not a second `contains` parent.

## 3. Link a type to its own methods

Added 2026-11-16, at the user's request.

**The gap.** `Buffer` has no link to `Buffer::new` or `Buffer::format`.

```text
Buffer, outgoing edges in the frozen expected graph:  implements 3, reads 1, uses_type 2
```

- The methods are nodes, contained by the file area.
- Each impl block is a stored record that lists its members (33 records), so the link can
  be derived from stored rows. Not measured: how much code that takes.

**Open before any code:** the relation kind. It must not be `contains`: impl members keep
the impl's lexical container (PRD). Entry 1 and this entry may share one decision.

## 4. The flag "source not read" on a target named from `core`'s declarations

Added 2026-11-16, at the user's request.

**Seen** in one exported row of a run with `--sysroot-source-scope-kind declarations` on this
VM (the call `buf[offset].write(b'0')`, target `MaybeUninit<T>::write`):

```text
external_name_kind          core_declared_member
external_site_proof_kind    core_declaration_probe
external_source_read_flag   false
```

- The declaration of that method was read: the row cites its file and byte offsets.
- An agent that reads the flag concludes the opposite.
- Not checked: the other 16 targets named this way, and what the flag is meant to say for
  them (the body of the method is not read; its declaration is).

**Open before any code:** what the flag means for a declaration that was read, and
whether it becomes two fields.

## 5. Node fields that are empty on every node

Added 2026-11-16, at the user's request. Already listed as open in the
[PRD](PRD-itoa-full-item-graph-scope.md) ("Node fields").

- Signature text, visibility and reachable-outside-the-crate are not filled on any node.
- `test_itoa_whole_graph_equality` asserts that they are empty, and does not compare them.
- An agent cannot tell what a function takes or returns without opening the source.

**Open before any code:** the expected value of each field for the 151 nodes, written
from source and frozen first.

## 6. DuckDB's `json` extension is read from the home directory

Added 2026-11-16, at the user's request. **Closed 2026-11-17 by SPEC part 10:** the
extension is compiled into the executable and `LOAD json` is removed. A run with an empty
`HOME` publishes the graph, and a traced run opens no path with `.duckdb/`. The text below
is the record of the defect.

**Seen on this VM.** A run of the release executable stopped at the graph stage:

```text
IO Error: Extension "/home/ubuntu/.duckdb/extensions/v2.0.0/linux_amd64/json.duckdb_extension" not found.
```

- The run passed after the file was downloaded from `extensions.duckdb.org` into that
  directory (51,158,462 bytes, sha256 `dd65944f...6bf57d1`).
- The PRD said the graph projection "loads the bundled `json` extension". On this build it
  is not bundled: the code runs `LOAD json`, and DuckDB looks in the home directory. The
  PRD line now says so (corrected 2026-11-17); the defect itself is open.
- So production reads a file that is neither the pinned source nor the `core` source, and
  SOP S1 does not freeze it.
- Not checked: how the file came to be on the machine of the accepted run, and whether the
  S2 audit of that run listed the read.

**Open before any code:** build the extension into the executable, or freeze the file by
digest in S1.

## 7. Name the one call into `core` that is left: `cast`

Added 2026-11-17.

**Seen in the data.** `self.bytes.as_mut_ptr().cast::<u8>()` in `Buffer::format` is the
only one of the 133 method calls into `core` that a `declarations` run stores without a
target. The frozen targets file gives its target as `*mut T::cast`, and the compile check
of 2026-11-16 agrees.

- The run finds the one declaration, in `impl<T: PointeeSized> *mut T` of
  `ptr/mut_ptr.rs`. It stops because the impl has a bound, and no stored row proves that
  `PointeeSized` restricts no type (REQ-RUST-062.0 C7).
- The expected graph of a `declarations` run lists this site without a target, with the
  known label beside it (REQ-RUST-066.0 M4).

**Open before any code:** which source facts prove that every type meets
`PointeeSized` (its `#[lang = "pointee_sized"]` declaration in `marker.rs` is one
candidate), written as a requirement first. When built, the label moves out of the
builder's list of unmerged labels and the graph is frozen again.

## Navigation-data additions from the research

Added 2026-11-17, from the source-backed navigation critique and the 80-repository
[research synthesis](https://app.devin.ai/sessions/ddc1e3dd95934b5a8ea0dcfd9da06478).
Entries 1–6 remain in force; the additions below do not duplicate their ownership,
signature or reproducibility work. Row counts and implementation cost are not measured.
Each addition needs its own source-written expected rows and freeze before producer code.

**These are producer/dataset ideas, not work for the
[CLI phase](../phase-20261116/ReadMe.md).** That phase consumes the available dataset and
reports its limitations; none of these additions is a prerequisite for CLI API ideation.
Nothing here authorizes changing the acceptance run already in progress.

## 8. Role-aware occurrences, including field references

**The navigation need.** A coding agent needs to find each use of a symbol, not just items
that depend on it. The graph's `reads` relation is for consts/statics, not a general field
access index. Field representation is already an open decision in the
[deferred backlog](BACKLOG-deferred-work-open-decisions.md) (OD-4).

**Add beyond that decision:** field identities and source-backed occurrence sites with
roles such as definition, reference, read, write and import. Start from itoa's named
`Buffer.bytes` and tuple `DecimalPairs.0` fields. Preserve each source site and its resolved
target; record an explicit reason when the target or role cannot be proven. Do not infer
semantic roles from identifier spelling or merge distinct sites into a single edge.

**Before code:** freeze field definitions and their occurrence ranges from pinned source;
decide their relationship to the existing item graph without silently changing `reads`.
Extend, rather than replace, entry 5's signatures/visibility/reachability comparison.

## 9. Exact macro template and invocation edit provenance

**The navigation need.** Existing `generates` and `expands` rows identify generated items,
calls and definitions. An agent also needs the editable source locations and which
generated items share the template it is about to change.

**Add:** explicit invocation, template-definition and relevant template-fragment anchors
for generated items/sites, linked to the existing expansion identities and evidence.
Keep original-source ranges separate from expansion ranges; mark an anchor unavailable
when exact provenance is not established. Do not present an invocation span as the exact
definition span of every generated member, or fabricate a one-to-one source range.

**Before code:** write source-backed expectations for representative generated items and
co-generated groups, then freeze the full addition. Preserve existing provenance records;
this is an enrichment of their edit routes, not a second macro-expansion engine.

## 10. Source-linked documentation and contracts

**The navigation need.** A signature alone does not tell an agent the documented
preconditions, safety obligations or usage of an item. Entry 5 remains the signature task;
this entry concerns documentation associated with that item.

**Add:** rustdoc text and source anchors, preserving sections such as Safety, Panics,
Errors and Examples where present. Keep documentation written in source separate from
generated explanations and from semantic guarantees proved by the mapper. A documented
claim is evidence of what the author wrote, not proof that every caller satisfies it.

**Before code:** freeze documentation ownership, exact source ranges and text/digests from
pinned itoa. Define how macro-generated items inherit or cite template documentation;
record an unavailable association rather than guessing. Do not ingest examples or tests
as graph inputs implicitly just because documentation mentions them.

## 11. Explicit source-scope and availability records

**The navigation need.** An absent caller can mean no recorded caller, excluded source,
inactive configuration or unsupported analysis. A coding agent must not read all of those
as "unused". The source scope also determines what absence a query can truthfully claim.

**Add or make explicit from existing input records:** the indexed source manifest, content
digests, build/cfg assumptions, and per-source indexed/excluded/inactive/unsupported/failed
states with typed reasons. Distinguish unresolved semantic sites from deliberately
excluded or inactive sites; identify external declarations separately from bodies read.
Entry 4 owns the declaration/body flag correction; this entry does not duplicate it.

Benchmarks, tests and examples outside the current graph stay explicitly outside it.
Widening that scope is a separate source/expected-data decision, not a query fallback.
Tie any new coverage records to the producer generation and frozen input identities;
freshness must not silently describe a different working tree as the indexed snapshot.

**Before code:** inventory existing persisted input/availability records so they are reused,
then freeze the missing coverage expectations, including negative and excluded cases.
The CLI can expose existing coverage facts now and report unavailable facts without
building these producer records itself.
