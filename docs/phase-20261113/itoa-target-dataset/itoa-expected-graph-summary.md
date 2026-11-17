# itoa expected full item graph: summary

**Status: FROZEN_EXPECTED_GRAPH.** Written and checked against the pinned source by Devin alone; no second reviewer. Never a production input. Identities are symbolic labels, not production keys.

Every cited source anchor is checked against the pinned bytes. Whole-source coverage is not claimed: operators and literals are not covered. A type written in an expression is covered (REQ-RUST-018.3 R10).

```text
nodes: 151
edge_rows_grouped: 475
edge_sites: 684
edge_sites_without_target: 133
impl_block_records: 33
sites_expected_to_store_no_row: 10
relation calls: 103
relation uses_type: 124
relation implements: 32
relation reexports: 0
relation contains: 90
relation derives: 0
relation imports: 4
relation reads: 62
relation generates: 45
relation expands: 15
target builtin: 77
target external: 64
target node: 279
target unresolved: 55
source commit: af77385d0daf4d0e949e81f2588be2e44f69f086
```
