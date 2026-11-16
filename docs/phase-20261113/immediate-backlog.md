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
