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

Added 2026-11-16, at the user's request.

**Seen on this VM.** A run of the release executable stopped at the graph stage:

```text
IO Error: Extension "/home/ubuntu/.duckdb/extensions/v2.0.0/linux_amd64/json.duckdb_extension" not found.
```

- The run passed after the file was downloaded from `extensions.duckdb.org` into that
  directory (51,158,462 bytes, sha256 `dd65944f...6bf57d1`).
- The PRD says the graph projection "loads the bundled `json` extension". On this build it
  is not bundled: the code runs `LOAD json`, and DuckDB looks in the home directory.
- So production reads a file that is neither the pinned source nor the `core` source, and
  SOP S1 does not freeze it.
- Not checked: how the file came to be on the machine of the accepted run, and whether the
  S2 audit of that run listed the read.

**Open before any code:** build the extension into the executable, or freeze the file by
digest in S1. Either way the PRD line is corrected.
