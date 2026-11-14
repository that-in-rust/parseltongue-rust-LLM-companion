# Part 1 — item nodes, containment and the frozen checkpoint

**State: MET for the bounded graph.** 45 nodes (43 original items, 2 file areas), 43
`contains` edges, 33 impl header rows, written and read back from a frozen generation.
Macro-generated items are not nodes yet; that is [part 5](part-5-macro-generated-items.md).

## Requirements

- **REQ-RUST-033.0 — harness.** A real-source test feeds the pinned tree through the production
  entry point and reads only the published generation. It fails, not passes empty, when a
  required stage did not run.
- **REQ-RUST-038.0 — sysroot switch.** With `--sysroot-source-scope-kind excluded`, no sysroot
  unit is loaded, the scope is recorded in the generation, and no sysroot identity is
  invented. The default is `included`. An excluded run cannot be an SOP PASS.
- **REQ-RUST-038.1 — absent sysroot names.** In an excluded run, a path that reaches `core`
  through an import, or a prelude name such as `Default`, is stored unresolved with
  `external_source_missing`. A resolved or ambiguous path is never changed by this rule.
- **REQ-RUST-035.0 — impl headers.** Every active impl block, hand-written or macro-generated,
  has one header row: its occurrence, self type, trait when present, and members. An
  unresolved self type or trait carries one reason. itoa has 33.
- **REQ-RUST-043.0 — source container.** Every item has exactly one lexical container: its
  file area, or the enclosing inline module or trait. An impl member uses the container of its
  impl. Self type and trait are metadata, never a second parent. Items in a function body are
  evidence, not nodes.
- **REQ-RUST-045.0 — incomplete checkpoint.** The `full-item-dependency-graph` stage persists
  nodes and edges under one frozen generation, reads them back, and fails if they differ. It
  stays marked `incomplete`, stores unknown visibility as unknown, rejects a stale or
  cancelled input, and leaves the previous output serving on failure.

## Tests on pinned itoa

```text
requirement  test
033.0        test_itoa_pipeline_reads_published
038.0        test_sysroot_scope_excluded_units, test_itoa_pipeline_reads_published
038.1        test_sysroot_reached_names_reasoned
035.0        test_impl_header_row_contents
043.0        test_original_member_graph_roundtrip, test_impl_header_row_contents
045.0        test_original_member_graph_roundtrip, test_default_new_call_roundtrip
```

## Still missing

- Effective visibility and outside-crate reachability are stored as unknown.
- Each active `use` is a node, but no `imports` edge leaves it
  ([part 4](part-4-signatures-other-edge-kinds.md)).
