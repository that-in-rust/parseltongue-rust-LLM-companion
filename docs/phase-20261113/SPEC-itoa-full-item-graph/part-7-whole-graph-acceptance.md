# Part 7 — whole-graph acceptance

**State: NOT STARTED. Stub: the work packet is written when this part is picked.**

## Requirement

- **REQ-RUST-044.0 — whole-graph acceptance.** The exported nodes, edges, evidence ranges and
  unresolved sites equal the frozen expected graph, with no extra, missing or wrong row; the
  output is identical at 1, 4 and 16 workers; source, configuration and executable digests,
  time, peak memory and disk are recorded.

Test to write: `test_itoa_whole_graph_equality`. Procedure:
[SOP](../SOP-itoa-full-item-graph-acceptance.md) S1 to S4.

## Blocked on or waiting for

- The expected graph is an unreviewed draft: 339 of 368 edges are blocked
  ([dataset](../itoa-target-dataset/README.md)).
- No comparator exists for full-graph rows.
- The two open points in the [index](README.md): one reason list, and the mapping from
  symbolic labels to canonical identities.
- Parts 3 to 6.
