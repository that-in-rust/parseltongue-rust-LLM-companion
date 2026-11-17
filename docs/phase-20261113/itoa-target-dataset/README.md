# itoa target dataset — the expected full item graph

## The answer

**This folder holds the frozen expected full item dependency graph of pinned itoa 1.0.18:
151 nodes and 682 edge sites (473 rows when sites with the same source, relation and target
are grouped). It is the benchmark that `test_itoa_whole_graph_equality` compares a
production run against.**

- Written and checked against the pinned source by Devin alone. No second reviewer and no
  compiler oracle (user decision, 2026-11-16).
- Never a production input. The mapper must not read it.
- Rows are named by symbolic label (`Buffer::new`), not by production identity.
- 133 of the 682 sites are method calls expected to have no target: the receiver's type is
  declared in `core`, which a run with scope `excluded` does not read.
- **A second expected graph, for a run with scope `declarations`** (REQ-RUST-066.0): the
  graph above with the frozen target of 132 of those 133 calls written in. 170 nodes (19
  more, one per target label), 682 sites, 1 site without a target (`cast`, listed in the
  file with the label it is known to have and the requirement that keeps it open).
  `test_itoa_declarations_graph_equality` compares a `declarations` run against it.
- Not covered: operators and literals. They are not sites of the graph.

## Files

```text
itoa-expected-graph-nodes.json        151 nodes
itoa-expected-graph-edges.json        473 rows, 682 sites, each with its byte range and record
itoa-expected-impl-blocks.json        33 impl blocks (records, not nodes)
itoa-expected-unresolved-sites.json   10 sites of the source expected to store no row
itoa-expected-graph-summary.json/.md  counts
itoa-expected-graph-freeze.json       the freeze record: sha256 of the five files above, of the
                                      builder and rules files, and of the two source files
itoa-expected-core-member-targets.json  frozen: the expected target of each of the 133
                                      method calls into core (REQ-RUST-058.0)
itoa-expected-core-target-freeze.json its freeze record: sha256 of the file, of the builder,
                                      and of the core source it was read from
itoa-expected-core-target-builder.py  writes that file from the pinned source and the core
                                      source of the installed compiler; checks every citation
test_itoa_expected_core_targets.py    11 tests
itoa-expected-declarations-graph-nodes.json    170 nodes: the 151 above and 19 targets in core
itoa-expected-declarations-graph-edges.json    473 rows, 682 sites: 132 calls into core with
                                      their frozen target, 1 (`cast`) without
itoa-expected-declarations-graph-summary.json  counts
itoa-expected-declarations-graph-freeze.json   its freeze record: sha256 of the three files
                                      above, of the three files they are merged from, and
                                      of the builder
itoa-expected-declarations-graph-builder.py    merges the frozen graph files and the frozen
                                      targets file; reads no source, changes neither input
test_itoa_expected_declarations_graph.py       5 tests
itoa-expected-graph-builder.py        writes the files from the pinned source
itoa-expected-graph-rules.py          shared vocabulary and serialisation
test_itoa_expected_graph_rules.py     9 tests
```

## Changing a frozen row

A correction needs a source citation. Edit the builder, then:

```bash
python3 docs/phase-20261113/itoa-target-dataset/itoa-expected-graph-builder.py \
  --source-root /home/ubuntu/repos/rust-tools/itoa \
  --output-directory docs/phase-20261113/itoa-target-dataset --write-freeze-manifest
python3 -m unittest discover -s docs/phase-20261113/itoa-target-dataset -p 'test_*.py' -v
```

Without `--write-freeze-manifest` the freeze record is not rewritten, and both the Python
tests and `test_itoa_whole_graph_equality` fail on the digest. Every earlier PASS is void
after a new freeze.

The graph of a `declarations` run is merged from the two frozen sets, so it is written
again after either of them changes:

```bash
python3 docs/phase-20261113/itoa-target-dataset/itoa-expected-declarations-graph-builder.py \
  --write-freeze-manifest
```

The builder stops if an input differs from its own freeze record, if a call without a
target has no row in the targets file, or if a row of the targets file matches no call.
