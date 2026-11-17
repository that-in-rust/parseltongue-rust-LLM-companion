# ARCH — pipeline, graph and tables for the itoa full item graph

## The answer

**One pipeline turns the pinned itoa source into one stored graph of items and their
dependencies. Rust produces facts, DuckDB does the relational work, Turso keeps the durable
frozen result. Every stage reads tables and writes tables; nothing else crosses a stage
boundary.**

Scope: itoa only, the full item graph only. No other repository, no oracle, no
higher-abstraction table.

Built: source to resolved names, then one graph step that writes item nodes and eight edge
kinds (`contains`, `calls`, `uses_type`, `reads`, `implements`, `imports`, `generates`,
`expands`) and reads them back from a frozen generation. Not built: lowered signature types,
type inference in bodies and a trait solver. A method call whose receiver type is declared in `core` is named from a syntax-only
read of `core` (scope `declarations`, part 8): 132 of 133 in the accepted run of
2026-11-17; `cast` has no target. With `excluded` none of the 133 has one.

```text
source files
  -> Rust    crate graph, ingest, one parse per file                  built
  -> DuckDB  name resolution rounds  <->  Rust item-macro expander    built
  -> Rust    impl headers                                             built
  -> Rust    method declarations of core (scope `declarations`)       built
  -> DuckDB  item nodes, contains, generates, expands                 built
  -> Rust    calls, uses_type, implements, reads, imports             built
  -> Turso   frozen full graph, read back and compared                built: marked incomplete
```

Scope: [PRD](PRD-itoa-full-item-graph-scope.md). Proof:
[SOP](SOP-itoa-full-item-graph-acceptance.md). Requirements:
[SPEC](SPEC-itoa-full-item-graph/README.md). Crates and files:
[crate map](MAP-parseltongue-crates-as-measured.md).

## 1 — The pipeline as built

Entry: `run_workspace_mapping_pipeline` in `parseltongue-cli-dispatcher-binary/src/main.rs`.

```text
step  function (crate)                                          writes
1     load_workspace_crate_graph (crate-graph-loader)           crate-dependency-graph-table,
                                                                workspace-config-state-records
2     ingest_workspace_source_files (crate-graph-loader)        file-source-text-table
3     extract_file_syntax_rows (syntax-span-extractor)          span-reference-text-table,
                                                                span-byte-ownership-ranges,
                                                                field-signature-detail-table,
                                                                body-site-reference-table,
                                                                six syntax-namespace-input-* tables,
                                                                lang-item-definition-table
4     prepare_sysroot_staging_scope (cli)                       sysroot crates as extra units
5     run_namespace_analysis_pipeline (duckdb-analysis-adapter  module-scope-binding-table,
      + port-macro-expander)                                    name-resolution-scope-records,
                                                                path-resolution-site-records,
                                                                macro-expanded-item-table
6     persist_impl_header_rows (syntax-span-extractor)          impl-header-index-table
6a    persist_sysroot_member_rows (cli)                         sysroot-member-declaration-records
7     publish_full_graph_checkpoint (cli)                       item-effective-visibility-records,
                                                                edge-evidence-record-table
```

- Step 4 runs when a file mentions `std`, `core`, `alloc`, `Clone` and similar, and
  `--sysroot-source-scope-kind` is `included` (the default). itoa triggers it. `excluded` and
  `declarations` add no sysroot crate. The accepted run of 2026-11-17 passes
  `declarations` (PRD, decided 2026-11-16); the one of 2026-11-16 passed `excluded`.
- Step 6a runs only with `declarations`. It reads the `.rs` files under
  `<sysroot>/lib/rustlib/src/rust/library/core/src` by syntax
  (`extract_member_declaration_rows`) and stores seven kinds of row: a member of a trait or
  impl (with its return type text, generic names and relaxed bounds), a type name
  declaration, a `use` leaf of a prelude file, the name tokens of a macro call, a name
  a file binds at file level (`file_scope_name_binding`: item, module, extern crate, `use`
  leaf, glob, or a form whose names are not read), a macro call written directly in an
  impl (`impl_macro_call`), and a file-level `macro_rules!` with its arm count
  (`macro_definition`). A `fn` or `const` in a macro's body is stored with its arm, its
  nesting depth and its attributes. Those files are an input of the run: the
  SOP freezes them by digest. The run id covers them: the workspace row stores
  `workspace_sysroot_source_record` (file count, byte count, the S1 digest of the files),
  the run id hashes that row, and the run fails if the bytes it parsed differ from the
  record (REQ-RUST-065.0).
- A receiver that is itself a method call into `core` takes the return type that
  declaration writes. A type name in it that `core` declares more than once (`Result`) is
  read only when the file's own scope and the prelude import prove which declaration it is
  (`find_file_scope_type`); the proof is stored on the edge row.
- A method called on a const of a builtin type (`u8::MAX.ilog10()`) is named when one
  macro call in `impl u8` is bound, by `#[macro_use]` scope, to one one-arm macro whose
  body declares both the const and the method (`find_builtin_constant_member`); every row
  of that chain is stored on the edge row.
- A method call into `core` is named by the impl's self type as `core` writes it
  (`[T]::len`). When a prelude trait declares the same name and cannot be ruled out, the
  target is stored with that assumption on the row, never without it.
- Step 5 copies one frozen Turso generation into a DuckDB file and runs SQL rounds for scopes,
  bindings, imports and path candidates. Each resolved item macro is expanded in Rust; its
  output becomes a virtual source file that goes back through step 3. Repeat to a fixed point.
- Step 7 runs only with `--mapping-final-stage-kind full-item-dependency-graph`:
  1. `derive_original_graph_input` (DuckDB) projects item nodes and the `contains`,
     `generates` and `expands` edges.
  2. Five Rust resolvers in `syntax-span-extractor` read the stored rows:
     `resolve_original_call_edges`, `resolve_signature_type_edges`,
     `resolve_impl_header_edges`, `resolve_body_read_edges`, `resolve_use_import_edges`.
  3. `collect_builtin_endpoint_nodes` and `collect_external_endpoint_nodes` add a node for
     each builtin type and each name outside the read source that an edge targets.
  4. `FullGraphInputStorePort` writes the graph to Turso, reads the frozen copy back, and fails
     unless it equals what was derived.
  5. The result is marked `incomplete`; that value is written in the code, not computed.

Three facts in the code that shape the remaining work:

- **C1** Name resolution takes only import and signature paths (`NameResolutionPathKind`). Body
  paths are stored as text in `body-site-reference-table`; only step 7 reads them.
- **C2** Header paths are resolved. Full signature types (parameters, returns, bounds) are not
  lowered into rows: `lowered-signature-type-table` has no writer. Resolvers work from
  resolved path lookups; part 8 adds the written type text of a parameter, of `self`, of a
  static, of a field, and of the return type of a `core` method. So a receiver is typed only
  in the shapes a resolver handles.
- **C3** itoa with the sysroot did not finish step 5 in 2,400 s (2026-11-13); on 2026-11-16
  the same run stopped after 171 s with a store copy mismatch. Acceptance runs exclude it.

### After the run: a DuckDB copy of the graph

Not a pipeline step. A second binary, `parseltongue-graph-dataset-loader`, reads the two
files `export-published-stage-rows` writes for the node and edge tables, checks them, and
writes one DuckDB file (user decision, 2026-11-17;
[part 9](SPEC-itoa-full-item-graph/part-9-graph-export-into-duckdb.md)).

```text
workspace-store.db (Turso)  --export-published-stage-rows-->  nodes.json, edges.json
nodes.json, edges.json      --parseltongue-graph-dataset-loader-->  graph.duckdb
```

- **Not a store of the run.** Turso stays the store, and the comparison with the expected
  graph reads Turso. The mapper does not depend on the loader crate, and the loader depends
  on no crate of this workspace.
- **Three tables:** `full_graph_dataset_info` (one row: run generation, row counts, the
  digests of the two input files), `full_graph_node_rows` and `full_graph_edge_rows`.
- **Nothing is lost.** Each row keeps its whole exported payload as JSON text with sorted
  keys. The other columns copy payload fields for SQL; an edge's source and target columns
  hold a node's key, so they join to the node table.
- **Checked before written:** one run generation in both files, no duplicate identity,
  every edge source a node, every edge target a node or absent with a reason.
- **All or nothing:** the file is built beside the output, read back and compared, then
  linked to the output path; an output path that exists is an error.
- **No DuckDB extension** is installed or loaded, unlike the run itself.

## 2 — The tables, top to bottom

Levels 1 and 2 are the full graph, the deliverable. Levels 3–7 are the facts it is built from.

```text
level  table                                  owner stage                        state
1      edge-evidence-record-table             semantic-edge-evidence-stage       eight kinds
2      item-effective-visibility-records      item-effective-visibility-stage    three fields filled
3      impl-header-index-table                signature-type-lowering-stage      produced
       lowered-signature-type-table                                              no producer
       sysroot-member-declaration-records     sysroot-member-declaration-stage   part 8 only
4      path-resolution-site-records           name-resolution                    produced
       name-resolution-scope-records
       module-scope-binding-table
       macro-expanded-item-table
5      syntax-namespace-input-* (six)         syntax-namespace-input-records     produced
       lang-item-definition-table
6      field-signature-detail-table           syntax-spans                       produced; text only
       body-site-reference-table
7      span-reference-text-table              syntax-spans                       produced
```

The three node fields are effective visibility, reachable outside crate, and signature
text. The syntax stage prints the signature and the declared visibility onto each span row;
`resolve_node_visibility_fields` sets the effective visibility and the reachability in
the checkpoint, after all edges are resolved (SPEC part 11). `sysroot-member-declaration-records` is written only with scope `declarations`.

Supporting tables:

- Inputs: `file-source-text-table`, `span-byte-ownership-ranges`,
  `crate-dependency-graph-table`, `workspace-config-state-records`.
- Work queues: `module-source-closure-requests`, `syntax-file-work-requests`.
- Control: `stage-run-generation-records`, `frozen-stage-publication-records`.
- Run receipts: `analysis-resolution-receipt-records`.

The registry of table owners is `STAGE_OWNED_TABLE_NAMES` in
`parseltongue-turso-store-adapter/src/stage-table-store-backend.rs`.

Columns the two graph tables carry:

- **Node:** target reference, item kind, owner kind, body ancestry, effective visibility,
  reachable outside crate, source path, qualified path, signature text.
- **Edge:** source reference, target reference, relation kind, resolution status, evidence row
  key, source site, dispatch metadata.

## 3 — The graph rules

Nodes:

- **G1** Every item is a node whatever its visibility: module, function, struct, enum, trait,
  type alias, const, static, associated function, const and type. A `macro_rules!` definition
  and a macro call at item position are nodes too; a macro-generated item records the call
  that generated it.
- **G2** Nothing inside a function body is a node. Body contents are evidence.
- **G3** The crate root is a node.
- **G4** Each item node stores effective visibility: public, restricted, or private; a macro
  call stores not applicable. Trait members and trait-impl members take the trait's
  visibility; inherent members keep their own. The crate root, a file area, a name outside
  the read source and a builtin type store none.
- **G5** Reachable from outside the crate is a separate column.
- **G6** Unknown visibility never silently means private.
- **G7** One file-area node per source file, separate from the file's module node.
- **G8** Each `use` declaration is a node.
- **G9** An inline module is an ordinary module node with its own scope.
- **G11** Impl blocks are not nodes. `impl Integer for u8` is an `implements` fact whose source
  is the builtin type.

Edges:

- **F2** Only facts proven from source are edges. An unresolved site is stored with a reason.
- **F3** A target is a graph node, a named external item, or a builtin primitive type. A named
  external item carries its name as far as the read source proves it, its crate, and a flag
  that its own source was not read. itoa's targets in `core` are stored this way.
- **F4** A generic or trait call targets the trait method once, with the bound chain in dispatch
  metadata. It never fans out to implementations.
- **F5** `pub use` gives one `reexports` edge per resolved binding. itoa has none.
- **F6** Every extracted edge is stored, private to private included. Nothing is collapsed.
- **F7** A file-area node `contains` each top-level node of its file. The crate root
  `contains` the file area of the root file, and a `mod name;` module that of the file it
  names; an inline module has no file area.
- **F8** A `use` node `imports` each item it binds.
- **F9** A function `reads` each const or static it reads.
- **F10** A function has edges only to what it uses, never to every import in scope.
- **F11** A call or type edge targets the real item; the `use` it came through is evidence.
- **F13** One lexical container per item. File-level items and impl members belong to the file
  area; nested items belong to their inline module or trait. Impl identity, self type and trait
  are metadata, never a second parent.
- **F15** One edge row per source site. Two sites with the same source, relation and target are
  two rows, each with its own range. The expected graph lists one row per source, relation and
  target with its sites; stored rows are grouped the same way before comparison.

Identity:

- **I1** Every node and edge endpoint is a full occurrence key: compilation unit, file, byte
  range, syntax kind, expansion chain. No display names, no invented sysroot spans.
- **I3** Malformed identity, a missing local endpoint, a conflicting duplicate or a wrong
  generation is a typed error before publication.

Open:

- **G10** Enum variants and struct fields are nodes, each contained by its type (PRD, decided
  2026-11-18; SPEC part 13, not built).
- **F12** Does a `pub use` carry both an `imports` and a `reexports` edge?
- **G4, G5** are rules without a producer: every node stores them empty or unknown.

Decided since (PRD, "Decisions in force"):

- **F14** `reads` and `uses_type` start at any item a path is written in.
- **I4** A macro-generated item is identified by its macro call and its place in that call's
  expansion; a row sited in expanded text also names where its tokens were copied from.

## 4 — What remains

The build order of the PRD is done up to the whole-graph comparison, which passed at
`822da80cf` (scope `excluded`) and at `753665eb9` (scope `declarations`). What the pipeline still lacks:

```text
what                                              why it is hard
targets for method calls into core (part 8)       needs the receiver's type and core's
                                                  declarations; no lowered types (C2), no
                                                  trait solver. Built for most shapes.
methods written by a macro in core, beyond the    built only for a one-arm macro called
proven form                                       once in an impl of a builtin type, found
                                                  by `#[macro_use]` scope (ilog10). Not built:
                                                  several arms, impls under `cfg` (usize),
                                                  a macro called through another macro
a receiver that is a local variable, beyond the   built only for one plain `let` whose
proven form                                       initializer is a method call named through
                                                  a bound, with a return type of builtin or
                                                  `core` names (string.len()). Not built: a
                                                  written type, a pattern, any other
                                                  initializer, a name used elsewhere in the
                                                  unit
an impl with a bound on its type parameter        cast on `*mut T` with `T: PointeeSized`;
                                                  nothing stored proves the bound restricts
                                                  no type
rows that depend on the source directory          file identity is the absolute path, and
                                                  the run id hashes those rows
the run with the whole sysroot (C3)               stops with a store copy mismatch
```

Counts, steps and the next target are in
[SPEC part 8](SPEC-itoa-full-item-graph/part-8-method-calls-into-core.md) and on the
[log](LOG-itoa-full-item-graph-progress.md)'s "Where we are" page, not here.

What reading itoa's source showed, still in force:

- **E3** A reference to an associated const is a `reads` edge.
- **E4** Some itoa-to-itoa edges need `core`. The signed `write` calls
  `Unsigned::fmt(self.unsigned_abs(), ..)`; choosing the impl needs the return type of a core
  method.
- **E7** Body macros are not expanded. The arguments of the assertion macros are parsed again
  as expressions for `reads` rows (PRD).
- **E8** A trait-impl member's visibility comes from the trait, so the visibility step reads impl
  headers.
- **E9** Node rows need a qualified path and signature text, for example
  `<u8 as private::Sealed>::write`. The label carries the path (PRD, node labels); the
  signature is not stored.

itoa's shape: three item macros (`impl_Integer`, `impl_Integer_size`, `impl_Unsigned`) generate
12 `Integer` impls, 12 `private::Sealed` impls and 4 `Unsigned` impls; five impls are
hand-written.
