# Part 4 — signatures and the other edge kinds

**State: NOT STARTED. Stub: the work packet is written when this part is picked.**

## Requirement

- **REQ-RUST-018.2 — signatures and bounds.** Parameter and return types, `Self`, generic
  parameters, bounds and supertraits are stored as resolved records. Every type path resolves
  or carries one reason. (Revision 018.1 measured this against an oracle; 018.2 measures it
  against the expected graph.)
- **REQ-RUST-032.0 — the full item graph.** Two tables, no others:
  `item-effective-visibility-records` for nodes and `edge-evidence-record-table` for edges.
  Every item outside a function body is a node, private ones included; each source file has
  one file-area node; each active `use` is a node. Every proven dependency is one uncollapsed
  edge with the source range of its site, targeting the real item and never the `use` line.

Edge kinds this part adds: `implements` from impl headers, `uses_type` from signatures and
bodies, `reads` of consts and statics, `imports` from `use` nodes. Today only `contains` and
`calls` have a producer.

First itoa target: `<Buffer as Clone>::clone --calls--> Buffer::new` together with its
`implements` and `uses_type` edges.

## Blocked on or waiting for

- Part 3 writes the first bound and supertrait records; this part extends them to full
  signatures.
- Decision OD-8: may `reads` and `uses_type` start at a non-function item
  ([backlog](../BACKLOG-deferred-work-open-decisions.md)).
