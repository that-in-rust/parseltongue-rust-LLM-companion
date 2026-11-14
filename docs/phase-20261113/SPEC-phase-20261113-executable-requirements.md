# Executable specs — phase-20261113: full dependency graph, HLD levels derived from it

Status: contracts. This file states only what is in force now; it replaces
`docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md`, which is kept as history.
Requirement ids and test names are unchanged, so code and journal references still resolve;
section 9 maps every superseded id. Product intent: PRD-phase-20261113-dependency-graph-hld.md.
Design: ARCH-phase-20261113-pipeline-graph-tables.md. Procedure: the phase-20261111 SOP.
**M** denotes observations, not targets. New owned identifiers, columns and artifact basenames
use four words. A named test contract is not a passing test; section 6 says which exist in code.

**Active Phase 1:** detailed full-map production, evidence and comparison (REQ-RUST-044.0).
Interface selection, HLD collapse, roll-ups, derived publication and projection-only decisions
are deferred in the [backlog](../phase1-abstractions-decision-backlog.md). Their existing
contracts and regressions remain valid when invoked; they are not Phase 1 exit gates.
The original-top-level mode is a retained diagnostic, not the full-map implementation path.

## 1. Scope and oracle
Map lib + bin targets of workspace members and their normal dependency closure. Do not map test, bench,
example or build-script targets or dev-dependencies. Record proc-macro/build dependency host contexts in the
crate graph for provenance, not as mapped application targets. Proc-macro shapes are the only production
substitute for executing proc macros. Test cfg is OFF; default feature selection is per compilation unit.
The active source closure is files reachable from those targets' module trees under active cfg, including
ordinary file includes and expansions discovered during resolution. Store other .rs files as bytes only.
Inactive items encountered in reached files are counted; unreachable files are not required to parse.
A successful external cargo check does not imply arbitrary repository files parse.

The offline answer key is `oracle-semantic-export-harness`, built from the rust-analyzer commit in §10.
Record its executable digest, source commit, toolchain and configuration. The installed rust-analyzer
1.99.1 (58cf8f9) is NOT the oracle for checkout 2a362a41a. The harness may link ide/hir/salsa and run proc
macros: it is outside Parseltongue's production dependency/process boundary. No LLM or training anywhere.

- PRIMARY: build scripts disabled; proc macros enabled; mapped target/feature/cfg/edition policy identical
  to production, including the --no-test equivalent. Shape comparisons use real proc-macro expansions.
- SECONDARY: identical inputs except build scripts enabled, also with proc macros enabled. Export per-site
  semantic rows, not analysis-stats aggregates. PRIMARY versus SECONDARY identifies the KL1 difference set;
  aggregate timing/statistics cannot attribute individual missing edges.
- Persist path targets, invocation-keyed expansion items/signatures and body sites, impl relations,
  method/call targets, signature type references, and effective visibility/reachability.
  SCIP is optional secondary occurrence evidence only; it does not export these complete semantic relations.
- Oracle source omissions, failed proc-macro loading or unexportable relations BLOCK the affected gate;
  never manufacture agreement or silently remove unsupported oracle sites.

## 2. Architecture

Two databases and no third. Turso is the concurrent write side and the serving store: durable
rows, generation markers, frozen snapshots, atomic publication. DuckDB is the analysis side:
it works on a copy of one complete published generation and owns every join, fixed point,
recursion, deduplication and roll-up. Rust owns parsing, macro expansion and per-body work.
No LLM, salsa or rust-analyzer semantic crate in production.

The pipeline builds one full-item-dependency-graph first. Later abstractions are queries over
that logical graph and its linked evidence; expensive derivations may be materialized. A level
adds no semantic fact; it filters nodes, collapses paths and counts. Queries cannot replace
missing parsing, expansion, resolution or inference. No separate source-analysis pipeline is
introduced per abstraction. The following diagram includes the deferred HLD query layer.
Owned names: `full-item-dependency-graph` (the two tables below taken together) and
`file-public-area-node`. `file_public_area`, `imports` and `reads` are enum values. The levels
are `public-item-l2-interface` and `folder-file-l1-rollup`. Body sites live only in
`body-site-reference-table`; no parallel call-site store is added. No benchmark reference data
may enter the production mapper.

```text
 source
   -> Rust: parse, expand, resolve, infer                 facts
   -> Turso: full-item-dependency-graph                   stored, never collapsed
        item-effective-visibility-records   one row per node
        edge-evidence-record-table          one row per edge fact
   -> DuckDB: levels                                      filter, collapse, count
        public-item-l2-interface    hld-current-interface-nodes, hld-current-dependency-edges,
                                    hld-edge-evidence-paths
        folder-file-l1-rollup       hld-file-level-rollup, hld-folder-level-rollup
   -> Turso: the five HLD tables, published atomically
```
```mermaid
flowchart TD
    A["cargo metadata + ingest"] --> B["Rust: parse spans / fields / body sites; Turso concurrent"]
    B --> C["Complete published generation: Rust Appender copy"]
    C --> D["DuckDB: name-resolution SQL fixed point"]
    D <--> E["Rust: macro expansion, writes rows back to Turso"]
    D --> F["DuckDB: signature type-path SQL join"]
    F --> G["DuckDB: impl / trait index SQL"]
    G --> H["Rust: per-body lowering"]
    H --> I["DuckDB: method candidates SQL + Rust: per-body inference"]
    I --> G2["Turso: full graph, nodes + edges"]
    G2 --> J["DuckDB: HLD levels, chain-collapse recursive CTE + rollups"]
    J --> K["DuckDB: oracle comparator SQL, S5–S7"]
```

### Crate inventory and stage ownership

- Existing `parseltongue-crate-graph-loader`: metadata and ingestion.
- Existing `parseltongue-syntax-span-extractor`: syntax and scope/import/item/
  visibility extraction; ordered textual macro scope; later per-body lowering.
- Existing `parseltongue-turso-store-adapter`: concurrent write-side evidence,
  BEGIN CONCURRENT/MVCC and complete generation publication.
- Existing `parseltongue-port-macro-expander`: Rust CPU expansion; output is
  persisted to Turso before the next frozen generation is copied.
- New `parseltongue-duckdb-analysis-adapter`: Appender handoff, resolution SQL,
  signature joins, impl/trait indexes, method candidates, HLD collapse/rollups
  and S5–S7 comparison. Inference remains Rust per body.
- Existing `parseltongue-table-store-traits`: L2 write and analysis ports.
- Existing `parseltongue-core-row-types`: dependency-free identities.
- Existing `parseltongue-stage-worker-scheduler`: bounded Rust stage workers.
- Existing `parseltongue-cli-dispatcher-binary`: composition and barriers.
- Existing `parseltongue-macro-span-records`: parser span compatibility.
- Deleted `parseltongue-port-name-resolver`: move its contract
  cases to DuckDB; remove its engine and obsolete in-memory orchestration.
- Oracle-only `offline-teacher-rust-analyzer`: external CPU oracle, never linked
  into production. Parser source pins remain unchanged.

L1 `parseltongue-core-row-types` is dependency-free with hand-written error enums. L2 declares
write and analysis ports. L3 holds Turso, DuckDB, graph loading, syntax extraction and macro
implementation. The CLI composes stages through ports.

Stages read declared tables and write declared tables; nothing is handed between stages in
memory. Name resolution and item-position macro expansion are one crate fixed point. The Turso
registry in code gives each table exactly one owner stage name, and it governs:

```text
 signature-type-lowering-stage       lowered-signature-type-table, impl-header-index-table
 semantic-edge-evidence-stage        edge-evidence-record-table          every edge, all kinds
 item-effective-visibility-stage     item-effective-visibility-records   every node
 hld-current-materialization-stage   the five HLD tables; reads only the two full-graph
                                     tables and unit provenance
```

## 3. Requirements

Each contract includes its clauses and the rules of section 4.

### 3.1 Input and process integrity

#### REQ-RUST-001.2 — offline graph and process boundary

WHEN production mapping starts, THEN check Cargo.lock exists before metadata, hash it before/after
`cargo metadata --offline`, and reject absence or mutation with CrateGraphLoadError before publishing rows.
Missing dependencies/metadata failure SHALL preserve stderr and publish no graph rows.
Write one crate-dependency-graph-table row per selected package/target plus explicitly tagged dependency
contexts (§1), preserving Cargo package ID/source, target name/kind, root, edition, features, cfg, host/target
role and dependency alias. Alias is not crate identity. Parseltongue's direct children SHALL be only
`cargo metadata --offline`, `rustc --print cfg`, `rustc --print sysroot`; reject other executable requests.
Under the metadata Cargo process only, permit rustc discovery descendants: `rustc -vV` or
`rustc - --crate-name ___` with approved discovery `--print=` flags, exactly the six
`--crate-type` selectors bin/rlib/dylib/cdylib/staticlib/proc-macro, and `-Wwarnings`.
No other flags, real source inputs or output artifacts are permitted. Approved print kinds are
file-names/sysroot/split-debuginfo/crate-name/cfg; version-only `rustc -vV` remains permitted.
Reject every other descendant, including real-crate compilation, build-script binaries and network activity.
The strace audit SHALL reconstruct ancestry and validate full argument vectors, not executable names alone.
SOP S2 scope pagination requires a stable generation, strictly advancing next_offset, has_more=false,
and accumulated entry count equal to scope_total. The per-page scope_truncated flag is informational.
Harness builds/exports, version recording and corpus qualification are outside this allowlist.
Metadata SHALL use a read-only input checkout or equivalent write guard; lock mutation fails before publication
and leaves the original lock bytes intact (test with a writable isolated fixture and restoration guard).

#### REQ-RUST-002.0 — bytes and no-op ingest

WHEN any source file is ingested, THEN stored bytes SHALL equal disk bytes, including non-UTF-8 and line endings.
An unchanged content digest SHALL cause zero file-content writes; a new generation may add references/markers.
Bytes-only files SHALL not be parsed or assigned semantic spans.

#### REQ-RUST-003.1 — source coverage

WHEN an active-closure file is walked, THEN its flattened ownership partition (item-owned ranges plus classified
gaps) SHALL tile its bytes contiguously without overlap and with zero "other" bytes. Nested semantic spans
may overlap their parents; count bytes once in the ownership partition, not once per nested span.

#### REQ-RUST-004.2 — parser boundary

WHEN an active-closure file has a pinned-parser error or non-UTF-8 bytes, THEN stop with
WorkspaceNotCompilingError carrying path and first error/byte offset; do not recover or publish the generation.
The same bytes in an unreachable fixture SHALL be stored only, not reject the workspace.
Pin parser compatibility to the selected toolchain/editions; an incompatibility blocks corpus acceptance.

#### REQ-RUST-014.1 — compilation contexts

WHEN a unit is loaded, THEN store cfg/features/edition on that unit, not a universal cfg set.
Use rustc print cfg for the selected target; host proc-macro/build contexts are tagged separately; sysroot
crates use target cfg. Features are package/unit-specific normal-dependency resolution from metadata.
Record the Cargo resolver version and derive selected-unit feature contexts from declared dependency kinds
and metadata, rather than copying one globally unified resolve-node feature list into every target.
The target triple is recorded (host triple by default); an explicit --target argument is permitted only on
the print-cfg command, never to compile. Preserve each unit's own edition and edition-specific prelude.
Cfg(test) is OFF. Inactive items in reached files SHALL be unresolved(cfg_inactive) and counted.
Cfg/edition differences never share semantic rows solely because file bytes match.

#### REQ-RUST-015.1 — build-script limitation

WHEN a crate has build.rs, THEN production SHALL neither execute that target nor read saved outputs under
target/ (including symlink escapes). OUT_DIR-dependent includes and unavailable env!/option_env! inputs
SHALL be unresolved(build_script). Populate CARGO_PKG_* and CARGO_MANIFEST_DIR from metadata.
Build-script-only cfg is OFF; count its inactive items as cfg_inactive, with KL1 attribution only when
the paired oracle exports establish that cause. Banner SHALL report build_script counts per crate.
Ordinary non-generated include! files are allowed through file-source-text-table, not target/ reads.

### 3.2 Name resolution and macros

#### REQ-RUST-005.1 — paths

WHEN name resolution completes, THEN use-tree and signature-path targets SHALL match PRIMARY in ≥99.5%
of eligible path sites (§4 (matching and scoring)). Persist every mismatch, unresolved answer and exclusion with a reason in
oracle-edge-agreement-table. M2 alone excludes-and-reports proc-macro-dependent sites; M3 removes that
temporary exclusion. Unknown proc macros are not a blanket exemption from final coverage.

#### REQ-RUST-016.1 — fixed point and module semantics

WHEN resolving a crate, THEN imports and item-position macro expansion SHALL share one fixed point.
Pin the external rust-analyzer constant FIXED_POINT_LIMIT = 8192; our loop permits 8192 changing
macro-expansion rounds, then errors on an attempted 8193rd round with NameResolutionStuckError.
Count progress by newly canonicalized items/bindings, not duplicate writes. Include attribute reseeding.
Publish complete bindings/expanded items per crate atomically; no reader sees an intermediate round.
Implement the module, hygiene, glob and prelude fixtures in §4 (resolution fixtures).

#### REQ-RUST-010.3 — non-monotone resolution rounds

Resolution is NON-MONOTONE. Recompute the complete binding set
each round, stratifying explicit/named candidates before globs. Explicit
bindings shadow globs for the same scope/name/namespace even when the explicit
import resolves later. Distinct glob targets produce ambiguity rows, never an
arbitrary winner. Preserve determinate/indeterminate import states as in the
pinned RA semantics. A Rust driver compares canonical full binding-set digests;
stop only when unchanged, or fail at 8192 rounds. Append-only INSERT SELECT is
not sufficient. Required RED cases: `late-explicit-overrides-glob`,
`conflicting-glob-targets-ambiguous`, `cyclic-glob-imports-terminate`.

#### REQ-RUST-010.4 — path, prelude and visibility semantics

Preserve self/super/crate paths, 2015/2018 roots, extern crate
aliases, no_std/no_implicit_prelude, use Trait as _, use Enum::*, normalized raw
identifiers, pub(in path), visibility filtered at the importer, and block def
scopes chained to module scopes. Required RED cases:
`module-path-prefix-cases`, `edition-path-root-cases`,
`external-crate-alias-cases`, `disabled-prelude-context-cases`,
`anonymous-trait-import-cases`, `enum-glob-variant-cases`,
`raw-identifier-normalization-cases`, `restricted-visibility-path-cases`,
`glob-importer-visibility-cases`, `body-module-scope-chaining`.

#### REQ-RUST-011.2 — textual macro scope and expansion alternation

Compute textual macro_rules scope and macro_use module order
in Rust, per file, in source order. Emit `legacy-macro-scope-records`; SQL
joins those rows rather than reconstructing textual order. Alternate SQL
rounds with newly resolved item macro expansion in Rust. Persist hygiene,
defining-crate identity, invocation chains, virtual files and expanded syntax
in Turso before copying the next complete published generation.
Required RED case: `legacy-macro-textual-order`.

#### REQ-RUST-006.2 — declarative/built-in expansion

WHEN macro_rules! or built-in macros expand at item/body position, THEN ≥99% of eligible invocations
SHALL match PRIMARY's canonical produced-item and body-site sets. Each expanded item SHALL carry
macro origin, invocation occurrence key and expansion provenance. Ordinary include! files join the closure.

#### REQ-RUST-013.1 — proc-macro shapes

WHEN an invocation matches proc-macro-shape-table by Cargo package ID/version and macro name, THEN
its (kind, name, normalized signature) set SHALL equal PRIMARY's real expansion for 100% of matching
invocations, not just one example per shape row. Each shape row SHALL have a fixture exercising it.
Unknown macro/version SHALL be unresolved(proc_macro), not guessed. Store the checked-in shape data
as proc-macro-shape-records.toml and load it into its declared table at generation start.

#### REQ-RUST-017.1 — lang items and incoherent impls

WHEN sysroot sources are resolved, THEN lang-item-definition-table SHALL contain every recognized
lang item present in the selected sysroot. M: the pinned lang_item.rs declaration has 169 sym rows;
this is a source observation, not a required sysroot count.
Include primitive and permitted incoherent inherent impls using rust-analyzer's type-attribute flag and
sysroot-origin search; distinguish rustc_has_incoherent_inherent_impls from rustc_allow_incoherent_impl.
Index population is tested at M2/M4 as available; primitive method lookup is tested at M5.

#### S5 semantic path partition — classification rule

The oracle publishes `path_lookup_scope_kind` on every exported path row.
`module-scope` means its semantic targets are module-bound definitions or
builtin leaves, and each resolved qualifier is a module/crate or enum.
Enum variants remain module-scope; enum qualification alone is not an
associated-type lookup. `type-relative` means the oracle identifies a generic
type/const parameter, Self, an associated-item container, or qualification
through a non-module/non-enum type or trait. Generic parameter names alone
are in this deferred partition. Their namespaces must match the path context:
a same-spelled const parameter cannot capture a type-namespace definition.
The rule uses HIR resolution, semantic generic ownership and associated-item
containers, never capitalization, path spelling or suffix heuristics.

Missing, unsupported or conflicting semantic classification is `unknown`,
retained and reported BLOCKED rather than silently included or discarded.
When a known qualifier already proves type-relative lookup, that path stays
in the deferred partition even if another component remains unresolved.
M2 agreement scores the module-scope partition only; report type-relative
counts for M4–M6, unknown counts and proc-macro exclusions separately.

### 3.3 Signatures and bodies

#### REQ-RUST-018.1 — signatures and impls

WHEN signatures/impl headers are lowered, THEN every type path SHALL be resolved or carry one reason.
Canonical uses_type/implements relations SHALL match PRIMARY at ≥99.5% (§4 (matching and scoring)); persist all disagreements.

#### REQ-RUST-007.2 — site precision

WHEN an extracted call/method site is compared, THEN method-call target precision SHALL be ≥99%.
Count wrong or missing PRIMARY targets for extracted sites as incorrect; do not excuse an extracted guess
using a reason. Every unresolved site SHALL carry exactly one §4 (unresolved reasons) reason. Compare ordinary call targets
as well, reporting their precision separately; method-call precision is this requirement's threshold.

#### REQ-RUST-008.1 — mandatory final coverage

WHEN each selected inference sub-step finishes, THEN report method-call coverage. At M8, coverage after
the last selected sub-step SHALL be ≥95% on each of the five repos. M7 may be skipped only when 7b
already meets that gate on all five; skipping it does not remove the M8 gate.

### 3.4 Full graph and HLD levels

#### REQ-RUST-032.0 — full item dependency graph

WHEN semantic facts are produced for a compilation unit, THEN the
full-item-dependency-graph SHALL be stored in two existing tables and no new
ones: item-effective-visibility-records holds one row per graph node and
edge-evidence-record-table holds one row per edge fact.

Nodes. Every module, function, struct, enum, trait, type alias, const, static,
associated function, associated const and associated type owned by the crate
root, a module, a trait or an impl SHALL be a node whatever its visibility,
private items included. Nothing with fn-body ancestry is a node. Each node
stores effective visibility under §4 (visibility and collapse) and node_reachable_outside_crate as
separate facts; unknown visibility is a typed error, never private.

WHEN a source file belongs to the unit, THEN exactly one file-public-area-node
SHALL exist for it, distinct from the file's module node, with item kind
`file_public_area` and an identity made of the compilation unit and the file
path. WHEN a `use` declaration is outside fn bodies and cfg-active, THEN it
SHALL be a node with item kind `use_statement`, whatever its visibility. An
inline module is an ordinary module node with its own scope; it does not see
the imports of the file that contains it.

Edges. Every relation proven from source SHALL be stored with extracted status,
including relations between two private items. The full graph is never
collapsed. An unresolved site is stored with its reason and is not an edge.
Each file-public-area-node has one `contains` edge to each top-level node of
its file. Each use node has one `imports` edge per resolved binding, aliases
and globs included. A function that reads a const, static or associated const
has one `reads` edge to it.

An edge's source is a full-graph node. The one exception is an `implements`
fact whose self type is not local (`impl Integer for u8`): its source is the
builtin or external type, it is stored, and it never seeds an interface-level
edge; no local node is invented for the primitive (REQ-RUST-029.0). Impl blocks
are not nodes; an impl member's row names its impl block as owner evidence.

WHEN a call, type use or read resolves a name through an import, THEN the edge
SHALL target the resolved item, not the use node; the use node's occurrence is
retained in that edge's evidence. A function SHALL have edges only to items it
uses, never to every import visible in its scope.

#### REQ-RUST-019.2 — interface level is a filter

WHEN the public-item-l2-interface level is built, THEN it SHALL be derived from
the full-item-dependency-graph by filtering nodes, collapsing paths and
deduplicating, and SHALL add no fact that is absent from the full graph.

A full-graph node is selected when it is the crate root, or a
file-public-area-node, or a use node, or an item whose effective visibility is
public or restricted-non-private. File-public-area-nodes are selected even when
the file's module is private. Private items are not selected at this level;
they remain full-graph nodes and appear as intermediates in evidence paths.
Restricted or public uses create one reexports edge per resolved exported binding (aliases
and globs included), sourced at the nearest enclosing selected module, or the crate root; the
lexical reexporting module is retained in evidence even when it is private. Store
node_reachable_outside_crate from complete module and reexport reachability, including
private parents.

WHEN a collapsed path is built, THEN no hop of it SHALL be a `contains` or
`imports` edge, neither the first hop nor a later one (user decision,
2026-11-13). Collapse walks calls, uses_type, implements, reexports, derives
and reads. A `contains` or `imports` edge appears at this level only as a
direct edge whose target is selected or external; one whose target is an
unselected private item yields no interface-level row and stays in the full
graph. Not in code yet: HLD_RECURSIVE_PATH_SQL walks every relation kind today.

WHEN folder-file-l1-rollup rows are built, THEN they SHALL count interface-level
edges only. A file-public-area-node is bucketed in its own file.

#### REQ-RUST-009.2 — extracted HLD and collapse

WHEN HLD is built, THEN each edge connects selected nodes or a selected node and a canonical external item,
has extracted status only, and collapsed rows name their complete private evidence chain(s).
Apply §4 (visibility and collapse)'s cycle/diamond/self-edge rules. Unresolved intermediate links SHALL not yield extracted edges.
File/folder roll-ups SHALL equal the deduplicated HLD aggregation for the published generation.
A collapsed path never contains a `contains` or `imports` hop (REQ-RUST-019.2).

#### REQ-RUST-030.0 — typed HLD materialization

WHEN complete compatible semantic inputs are provided, THEN the HLD builder
SHALL emit REQ 019.2 selected nodes, REQ 009.2 extracted edges and every distinct
canonical simple evidence path. It SHALL retain selected zero-degree nodes.
Effective visibility facts include item kind, owner, fn-body ancestry and
outside-crate reachability; unknown visibility SHALL NOT silently mean private.
Relation kinds are those of REQ-RUST-030.1. Generic/concrete dispatch is evidence metadata,
not an additional relation kind.

WHEN traversal reaches a selected or canonical external target, THEN it SHALL
stop; otherwise it SHALL traverse only extracted relations through private
items, preserving the terminal relation kind. Unresolved links terminate paths.
Cycles SHALL terminate using per-path occurrence identities. Diamonds SHALL
merge rows by (generation, source, target, relation kind, via kind), retain all
distinct paths, and count the union of original evidence-row keys. Genuine
direct/finite-path self-edges remain once; private cycles alone invent none.

WHEN a semantic input has a malformed identity, missing local endpoint,
conflicting duplicate, incompatible generation or missing completion marker,
THEN the builder SHALL return a structured typed error before publication.
Pure L1 types SHALL have zero dependencies and reuse the established full
compilation/occurrence identity, not a parallel display-name key scheme.

##### Input validation

HLD Builtin node/target identities SHALL use only the primitive type vocabulary
seeded by the namespace extractor: bool, char, str, usize/isize, signed and
unsigned integers through 128 bits, and f16/f32/f64/f128. Ordinary definitions
including core functions and methods SHALL retain full Definition identity.
An ExternalDefinition whose full compilation-unit key is represented by a
LocalItem or CrateRoot in the same input SHALL be rejected; an external
definition in another explicitly classified workspace unit remains valid.
Evidence-site and owning-source occurrences SHALL share the full compilation
unit key, while their file paths may differ for macro provenance.

Local items SHALL carry known owner/item semantics. FunctionBody ownership
requires body ancestry; CrateRoot, Builtin and ExternalDefinition records SHALL
not carry body ancestry. Nested module items with body ancestry remain valid
input but unselected. Unsupported Field/EnumVariant ownership, Other/unknown
owners and inconsistent ancestry SHALL fail with a typed validation error.
Identical duplicate provenance/node/evidence records are legal; conflicting
duplicates fail. Inferred, Proposed and Unresolved evidence SHALL never become
extracted output, and unresolved reason metadata remains in its input record.

##### Path and collapse SQL

The DuckDB adapter SHALL seed a recursive CTE from extracted edges whose source
is selected, extend paths only through nonselected LocalItem endpoints, and
stop at selected or canonical external/builtin endpoints. Each path SHALL carry full
source identity, ordered endpoint identities and ordered original evidence-row
identities. The terminal edge's relation kind SHALL label the HLD row.

Per-path visited identities SHALL prevent repeating private endpoints while
allowing one finite return to the selected source as a terminal self-edge.
Private cycles alone SHALL emit no edge. Distinct ordered evidence paths SHALL
remain distinct even when endpoint sequences match. DuckDB SQL SHALL group HLD
rows and count the distinct original evidence-key union per
generation/source/target/relation/via group by UNNESTing ordered evidence
lists. Rust SHALL only decode and sort the typed output, without traversing
or aggregating the graph.

This checkpoint SHALL report GraphPathsOnly; file and folder roll-ups remain
None and CompleteGraph remains reserved until roll-ups exist. It does not
implement input-marker validation, durable publication or real-source
producers.

##### File and folder roll-ups

The adapter SHALL materialize or reuse one deduplicated HLD edge relation in
SQL. File and folder roll-ups SHALL GROUP that relation, never raw paths or
unnested evidence; each full HLD edge key contributes exactly once at each
level. Relation and collapse-via kind SHALL remain grouping dimensions.

File buckets SHALL use SourceFile(full compilation-unit identity, normalized
relative path) for LocalItem and ExternalDefinition nodes with explicit
verified source-relative paths; CrateRoot nodes SHALL use the explicit
CrateRoot bucket. ExternalDefinition nodes without verified file metadata
and Builtin nodes SHALL use CanonicalExternalIdentity(full target key).
Missing external file paths SHALL never be guessed from labels or occurrence
names.

Folder buckets SHALL use the immediate parent directory of known files, with
`.` for root-level files and the full compilation-unit identity. CrateRoot
and CanonicalExternalIdentity buckets SHALL remain explicit. The L1
HldFolderEndpointBucket SHALL be distinct from HldEndpointRollupBucket so a
folder path is never stored as a file path.

For complete typed facts, output SHALL be CompleteGraph with Some(file rows)
and Some(folder rows), including Some(empty) for empty input. This denotes
complete materialization of the supplied facts only; it proves neither
frozen-stage-marker completion nor source-pipeline, publication or corpus
qualification. Rust may normalize/encode each endpoint bucket and decode/sort
typed rows; grouping and counting SHALL remain SQL operations.

#### REQ-RUST-030.1 — relation kinds

Resolved relations use only calls / uses_type / implements / reexports /
contains / derives / imports / reads. Generic/concrete dispatch stays evidence
metadata. The typed materializer input and output records, their validation and
their canonical JSON SHALL accept the two added kinds and the
`file_public_area` item kind.

#### REQ-RUST-029.0 — source-derived reference separation

WHEN the itoa reference is constructed, THEN source-written items SHALL carry
exact source byte evidence and generated items SHALL distinguish template,
invocation, substitution and declaration/implementation owner. Each of the
twelve selected integer implementations has distinct associated occurrences;
the bodyless trait declaration never owns an implementation body's calls.
Local visibility and external reachability SHALL be evaluated separately under
REQ 019.2: public `Sealed` items under a private parent and `pub(crate) mulhi`
qualify locally. Impl blocks are header/evidence containers, not D6 nodes.

WHEN a required context, expansion identity or external target is unverified,
THEN the reference SHALL remain DRAFT and enumerate that gap. Semantic families,
display-name identifiers and source-only target guesses SHALL NOT count as
accepted canonical rows. Production SHALL NOT read expected-reference data.

#### REQ-RUST-031.0 — transactional HLD serving

WHEN HLD output is persisted, THEN `hld-current-interface-nodes`,
`hld-current-dependency-edges`, `hld-edge-evidence-paths`,
`hld-file-level-rollup` and `hld-folder-level-rollup` SHALL be one atomic
stage-owned Turso work unit. Publication SHALL require exact compatible
semantic-input completion, not mere table existence. Cancellation or write
failure SHALL leave the preceding publication unchanged.

WHEN inputs are reordered or repeated in another operational attempt, THEN
sorted semantic output SHALL remain identical after excluding attempt identity.
Empty completed fixtures SHALL be distinguished from missing/incomplete input.
Roll-ups SHALL count each deduplicated HLD row once at each level, not path
multiplicity. File keys include compilation context and normalized relative
path; folder keys use immediate parent directories with root `.`. An external
target without verified definition-file evidence SHALL retain its canonical
external-identity bucket, explicitly distinct from a source-file bucket; no
guessed local file or silent edge exclusion is permitted.

##### Durable Turso publication contract

The L2 HldGraphPublicationPort accepts immutable source snapshot scope, mutable
destination scope, typed HldMaterializationOutputRecord, and RunCancelFlagValue;
it returns the new frozen scope or typed HldPublicationPortError. The L3
TursoWorkspaceStoreAdapter uses the existing generic stage schema; L1 remains
dependency-free.

Register lowered-signature-type-table and impl-header-index-table under
signature-type-lowering-stage; item-effective-visibility-records under
item-effective-visibility-stage; edge-evidence-record-table under
semantic-edge-evidence-stage. Keep legacy owners unchanged. The five semantic
HLD tables remain hld-current-interface-nodes, hld-current-dependency-edges,
hld-edge-evidence-paths, hld-file-level-rollup, hld-folder-level-rollup;
owner: hld-current-materialization-stage. No input decoders/producers are
added.

Require source scope in frozen-stage-publication-records, resolve its base
attempt/revision, match destination generation/base attempt and all output
generation keys, require CompleteGraph and both Some roll-ups, and require
markers visible at that exact revision for crate-graph, source-ingest,
syntax-spans, syntax-namespace-input-records, name-resolution,
signature-type-lowering-stage, item-effective-visibility-stage, and
semantic-edge-evidence-stage. Later mutable-base markers cannot qualify an
older snapshot. Preserve workspace_required_stage_names and the generic gate.

One transaction writes all five tables under hld/current, writes unit/global
completion markers, seals a snapshot and switches the workspace manifest.
The global HLD completion marker payload records input snapshot alias, base
attempt and revision as persistence metadata, separate from semantic payloads.
Semantic keys/payloads use canonical JSON for full typed identities, enum
variants, optional signatures and ordered proofs; no Debug strings, guessed
fields, transient IDs or attempt IDs.

Cancellation is checked throughout and before commit; failure rolls back rows,
markers, snapshot metadata/sealing and manifest, preserving the previous view.
Reject sealed HLD units; clear only unpublished HLD rows/markers in the
destination scope. Retain bounded Busy retries without erasing typed errors.
For each required input owner, reject publication with typed
StaleInputSnapshot if any current-scope completion/work-unit marker is absent
from the supplied frozen revision. This exact-closure check is HLD-specific;
generic snapshot semantics remain unchanged. Generic writes owned by
hld-current-materialization-stage are rejected after any of its markers is
sealed, including global completion and work units other than hld/current.
Before publishing, clear all unsealed HLD-owned rows and markers in the
destination scope so stale work units cannot enter the new view.
This checks snapshot compatibility, not producer or source-pipeline completion.

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| REQ-RUST-029.0 | test_itoa_reference_occurrence_inventory | reference contract | distinct generated owners, selected internal interfaces, explicit unfrozen gaps | exact inventory |
| REQ-RUST-030.0 | test_hld_identity_validation_rules | unit | full context identity and source-relative locations validated | invalid inputs rejected |
| REQ-RUST-030.0 / 019.1 | test_hld_node_visibility_rules | unit/integration | internal-public selected; body-local/private-equivalent excluded; degree-zero retained | exact node set |
| REQ-RUST-030.0 | test_hld_direct_edge_projection | integration | canonical local/external targets and extraction-only output | exact edge set |
| REQ-RUST-030.0 | test_hld_private_path_materialized | integration | prior private-path fixture now emits the terminal collapsed row | exact key, path and evidence |
| REQ-RUST-030.0 | test_hld_private_terminal_kind | integration | private Calls hop followed by terminal UsesType hop preserves UsesType | exact key, path and evidence |
| REQ-RUST-030.0 | test_hld_selected_target_stop | integration | traversal does not continue past a selected target | exact terminal rows only |
| REQ-RUST-030.0 | test_hld_diamond_evidence_union | integration | direct/collapsed buckets, evidence-separated paths and unique union | exact rows, paths and counts |
| REQ-RUST-030.0 | test_hld_cycle_self_edges | integration | private cycles terminate; finite and direct self-edges remain distinct | exact rows and paths |
| REQ-RUST-030.0 | test_hld_unresolved_path_stop | integration | unresolved, inferred and proposed hops stop while extracted branches remain | exact row/path set |
| REQ-RUST-030.0 | test_hld_invalid_input_rejection | negative integration | malformed identity/missing endpoint fails | typed error |
| REQ-RUST-030.0 | test_hld_duplicate_conflict_rejection | negative integration | conflicting duplicate is not overwritten | typed error |
| REQ-RUST-030.0 / 031.0 | test_hld_generation_input_isolation | integration | wrong generation/base attempt, partial graph, or unfrozen source cannot publish | typed rejection; zero foreign rows |
| REQ-RUST-030.0 / 031.0 | test_hld_missing_input_markers | negative integration | missing frozen marker and marker added after older snapshot are rejected | no publication |
| REQ-RUST-031.0 | test_hld_stale_input_snapshot | negative integration | a later semantic work-unit marker outside the frozen revision is rejected with typed error | no HLD rows or manifest switch |
| REQ-RUST-031.0 | test_hld_sealed_any_marker | negative integration | global HLD completion and another HLD unit in a frozen view cannot be replaced | sealed view unchanged |
| REQ-RUST-031.0 | test_hld_sealed_node_rewrite | negative integration | generic HLD owner writes through another unit or completion marker cannot mutate a sealed row | old payload unchanged |
| REQ-RUST-031.0 | test_hld_unsealed_other_staging | integration | stale unsealed HLD rows/markers from other units are cleared before publication | new view excludes stale unit |
| REQ-RUST-031.0 | test_hld_cancel_preserves_publication | integration | mid-transaction cancellation rolls back and preserves previous view | no partial rows; unchanged manifest/output |
| REQ-RUST-031.0 | test_hld_failed_write_rollback | integration | later SQL insert failure rolls back all outputs, markers, snapshot and manifest | old view unchanged; no partial rows |
| REQ-RUST-031.0 / 012.1 | test_hld_order_attempt_determinism | integration | reordered/duplicate inputs and attempt token preserve nodes, edges, paths and roll-ups; reused adapter clears on empty input | output equality including roll-ups; empty output |
| REQ-RUST-031.0 / 009.2 | test_hld_rollup_edge_counts | integration | path/evidence multiplicity never inflates deduplicated file/folder groups; relation/via stay distinct and functions merge by folder | exact group counts; sums at both levels equal HLD row count |
| REQ-RUST-031.0 / 009.2 | test_hld_rollup_bucket_context | integration | same relative path in distinct units, root file, CrateRoot, verified/missing external files and Builtin preserve typed buckets | exact typed bucket keys |
| REQ-RUST-031.0 | test_hld_empty_input_rollups | integration | completed empty typed input yields CompleteGraph and present empty roll-ups | Some(empty) at both levels |
| REQ-RUST-031.0 | test_hld_complete_empty_snapshot | integration | complete empty output is durably published, distinct from missing stage | five empty tables and visible markers |
| REQ-RUST-031.0 | test_hld_publication_round_trip | integration | DuckDB typed fixture is published/read through Turso; prior/new views remain isolated | exact five tables and snapshot metadata |

These are required test contracts, not passing evidence. Record STUB/RED/GREEN/
REFACTOR output and exact test implementation locations in the journal. The
pinned full-map comparison is the Phase 1 gate; source-to-HLD comparison is deferred (BL-06).

#### REQ-RUST-025.0 — independent-source-grounded-hld-current-audit

WHEN an independent model audits one pinned repository's HLD current, THEN
Pass A SHALL derive candidate nodes and dependencies from pinned source and
configuration without reading production HLD rows; record byte/line evidence,
canonical compilation and occurrence keys, examined scopes, exclusions,
unknowns and coverage limits; and freeze the canonical expected artifact digest
before Pass B. Pass B SHALL compare that frozen artifact with the matching
generation's production HLD current rows, including false/missing edges,
effective visibility, reexports, private-chain cycles/diamonds/self-edges,
provenance and deduplicated file/folder roll-ups.

Each repository SHALL be reported separately. A missing production export is
BLOCKED/NOT STARTED. Model results are non-authoritative candidate evidence:
model agreement SHALL NOT pass a gate or alter the PRIMARY denominator;
source-confirmed false edges SHALL fail, and unresolved disputes SHALL block
the scoped audit. The existing deterministic PRIMARY and S8/S9 acceptance
rules remain authoritative. The S8 sample algorithm is unchanged; source-first
supplementary samples SHALL cover possible missing edges. Denominators and
sampling limits SHALL be explicit, and no complete-golden-table claim is
allowed without exhaustive independently verified coverage.

For the bounded source audit, `source_pin.config_sha256` SHALL equal SHA-256 of
the separate immutable canonical audit configuration bundle. It SHALL NOT be
the mapper's per-unit BLAKE3 digest or `RunGenerationDigestValue`; the bundle
SHALL preserve the selected policy, exact tool inputs, and per-unit key and
configuration fields without inventing occurrence keys.

Verification matrix additions — all planned tests are PENDING and do not yet
exist in the source tree:

```text
req_id: REQ-RUST-025.0
test_id: test_pass_a_source_only
test_type: fixture
assertion: Pass A uses source/config evidence only, with no production HLD rows
metric: every candidate key has a pinned source digest and line range

req_id: REQ-RUST-025.0
test_id: test_frozen_artifact_digest_integrity
test_type: fixture
assertion: Pass B cannot begin with a missing or changed frozen Pass A digest
metric: exact SHA-256 over canonical expected-artifact bytes

req_id: REQ-RUST-025.0
test_id: test_false_edge_has_evidence
test_type: fixture
assertion: a source-confirmed extra HLD row is reported as false
metric: exact false-edge keys and source evidence references

req_id: REQ-RUST-025.0
test_id: test_missing_edge_source_first
test_type: fixture
assertion: source-selected expectations detect an absent production edge
metric: exact missing-edge keys, including source-first sample count

req_id: REQ-RUST-025.0
test_id: test_unresolved_dispute_blocks_audit
test_type: fixture
assertion: unsupported or disputed evidence blocks, never passes
metric: blocked finding count and explicit causal reason

req_id: REQ-RUST-025.0
test_id: test_source_digest_mismatch_blocks
test_type: fixture
assertion: an input source/config digest mismatch blocks comparison
metric: expected and observed SHA-256 values

req_id: REQ-RUST-025.0
test_id: test_cycle_no_fake_self
test_type: fixture
assertion: cycle traversal terminates without inventing a self-edge
metric: exact canonical edge-key set

req_id: REQ-RUST-025.0
test_id: test_diamond_dedup_keeps_paths
test_type: fixture
assertion: a diamond yields one HLD key and retains every evidence path
metric: unique row count and distinct path/evidence counts

req_id: REQ-RUST-025.0
test_id: test_rollup_counts_unique_rows
test_type: fixture
assertion: file/folder roll-ups count deduplicated HLD rows, not paths
metric: exact grouped row counts

req_id: REQ-RUST-025.0
test_id: test_missing_export_blocks_audit
test_type: fixture
assertion: absent HLD current export is BLOCKED/NOT STARTED
metric: zero comparisons and a non-success disposition

req_id: REQ-RUST-025.0
test_id: test_s8_sample_algorithm_unchanged
test_type: contract
assertion: the supplementary model audit does not alter SOP S8 sampling
metric: identical seed, algorithm, keys and deterministic sample count

req_id: REQ-RUST-025.0
test_id: test_config_bundle_digest_contract
test_type: fixture
assertion: source_pin.config_sha256 hashes the canonical immutable config bundle, not a unit or generation digest
metric: exact bundle SHA-256 and per-unit mapper BLAKE3 inputs
```

#### REQ-RUST-033.0 — real-source acceptance harness

WHEN a real-source acceptance test runs, THEN it SHALL feed a pinned source tree through the
production pipeline entry point, read only rows of the published generation, and hold every
expected row in test assertions. No pre-resolved fact, fixture row or reference data SHALL be an
input to the pipeline. The test SHALL fail, not pass empty, when a required stage has not run.

#### REQ-RUST-034.0 — body path resolution

WHEN a path occurs inside a function body, const or static initializer, THEN it SHALL be
resolved to an item occurrence or stored unresolved with exactly one reason of section 4.
A name shadowed by a local binding SHALL NOT resolve to an item.

#### REQ-RUST-035.0 — impl header rows

WHEN an impl block is active, hand-written or macro-generated, THEN impl-header-index-table
SHALL hold one row with its occurrence, resolved self type, resolved trait target when present,
and its member occurrences. An unresolved self type or trait carries one reason.

The header producer SHALL reuse the persisted parser and namespace facts, not reparse source
or infer identity from a display name. Each header retains its parser occurrence key, canonical
semantic occurrence, direct cfg-active member occurrences, self/trait path outcomes, negative-impl
flag, body ancestry and lexical source container (REQ-RUST-043.0). Generic/unsupported type
forms remain reasoned until signature lowering supports them. Missing external traits retain
`external_source_missing`; they SHALL NOT supply assumed visibility or a fabricated core target.
Producing headers alone SHALL NOT mark the entire signature-lowering stage complete.
The producer requires the completed name-resolution snapshot and SHALL NOT run when the
requested final stage is `syntax-spans` (REQ-RUST-024.11).

`test_impl_header_row_contents` is executable: its initial pinned-source RED reaches 33 active
impl spans and observes zero header rows. This is supporting evidence for the active
REQ-RUST-036.0 call checkpoint, not completion of semantic resolution.

#### REQ-RUST-036.0 — type-relative lookup

WHEN a path has the form `Type::member`, THEN lookup SHALL resolve `Type`, then search inherent
impls of that type, then impls of traits in scope for that type, and emit one extracted target
only when exactly one member matches. Zero or several matches SHALL be unresolved with a
reason, never guessed. A generic parameter head resolves through its bounds to the trait member.

#### REQ-RUST-037.0 — HLD input assembly

WHEN the HLD stage starts, THEN its typed input SHALL be assembled only from
item-effective-visibility-records, edge-evidence-record-table and unit provenance of one frozen
generation whose required stage markers are complete. A missing marker SHALL be a typed error
before any analysis.

#### REQ-RUST-038.0 — sysroot source scope switch

WHEN `map-workspace-full-run` is given `--sysroot-source-scope-kind excluded`, THEN no sysroot
unit SHALL be added to the crate graph or ingested; the published workspace configuration SHALL
record `workspace_sysroot_scope_kind = "excluded"`, which enters the generation digest; and no
path SHALL resolve to an invented sysroot identity. A path whose first name segment (after an
optional leading `::`) is `core`, `alloc`, `std`, `proc_macro` or `test`, and for which the
unit's extern scope has no binding of that name, SHALL be stored unresolved with reason
`external_source_missing`. The default SHALL be `included`, with behaviour unchanged. Any other
value SHALL be a usage error. A run with `excluded` is local-only: its comparisons are labelled
local-only and it cannot be an SOP PASS (PRD AD-1, interim option B).

#### REQ-RUST-038.1 — names that reach an absent sysroot

WHEN a run uses `excluded` and a path stays unresolved and not ambiguous, THEN it SHALL be stored
with reason `external_source_missing` in each of these two cases, and in no other new case:

1. Its first name segment is bound, in the path's lexical scope or an enclosing scope that
   lookup may enter, by a non-glob import whose own path starts at a sysroot crate with no unit
   in the generation (`use core::mem::MaybeUninit;` then `MaybeUninit<u8>`).
2. It is a one-segment path that is not an import, no lexical scope on its lookup chain binds
   the name, and the chain ends at the standard prelude, whose source is not in the generation
   (`Default`, `Clone`, `Copy`).

A resolved or ambiguous path is never changed by this rule, and no target is invented. Paths
that fail for another cause keep their own reason or none: a type-relative path such as
`Self::MAX_STR_LEN` is not covered here (REQ-RUST-036.0). Measured on pinned itoa before this
rule (2026-11-13, 202 path sites): 13 `MaybeUninit<u8>` sites fall under case 1 and 4 prelude
names (`Copy` ×2, `Default`, `Clone`) under case 2.

#### REQ-RUST-039.0 — Phase 1 reference candidates and table contracts

WHEN Phase 1 itoa reference artifacts are rebuilt offline, THEN the two full-graph files SHALL
preserve the complete proposed source-derived node and edge candidate inventory, including
private items, all eight relation kinds (zero-occurrence kinds included), impl-block records,
source evidence, and unresolved or inactive diagnostics. The inventory SHALL remain labelled
`LLM_PARSED_DRAFT`, authored by Devin from pinned source, with symbolic noncanonical identities.
No independent compiler/oracle verification has frozen it. It SHALL NOT be treated as extracted
production rows or consumed by the production mapper.

The artifact set SHALL show that complete candidate inventory beside a separate
`ELIGIBLE_LOCAL_ONLY_DIAGNOSTIC` subset and explicit blocked/proposed candidate classifications.
The five expected HLD outputs, retained as deferred reference artifacts, SHALL be distinct
and derived only from eligible rows when HLD work resumes:
`itoa-expected-interface-nodes.json`, `itoa-expected-dependency-edges.json`,
`itoa-expected-evidence-paths.json`, `itoa-expected-file-rollup.json` and
`itoa-expected-folder-rollup.json`. Blocked rows and exclusions SHALL remain separately
inspectable and counted. Counts SHALL be recomputed from the artifacts and SHALL NOT be acceptance
constants. Unknown visibility, external readable
labels, unresolved/proposed facts, macro file ownership, inferred concrete core-dependent calls,
and rows depending on undecided OD-2, OD-8 or the remaining file-module/area part of OD-7 SHALL
never become eligible by default. REQ-RUST-043.0 settles original source containment, but
previously blocked candidate rows require source review before promotion.
OD-9 remains an alternate proposal and does not suspend the current rule 10 six-kind collapse.
Local-only expected outputs SHALL omit external extracted edges; an unverified reason remains a
blocked diagnostic, not a claimed site fact. A macro-owned file/folder roll-up SHALL be BLOCKED,
not assigned to an inferred file.

WHEN reference HLD outputs are derived, THEN `contains` and `imports` SHALL be excluded from every
collapsed hop, including the first; direct and collapsed rows SHALL remain distinct; cycles SHALL
terminate; diamond paths SHALL retain distinct evidence while producing deduplicated output rows;
and file/folder roll-ups SHALL count deduplicated HLD edges, not paths. Impl blocks SHALL remain
records, never nodes. Full-graph calls and type/read references SHALL target items rather than
`use` nodes; the `use` occurrence remains evidence. A direct or collapsed expected output SHALL
not contain a fact excluded from its input eligibility subset.

The machine-readable `itoa-table-contract-manifest.json` SHALL give every registered table its
exact registry owner, physical schema, observed serialized fields with provenance citations, key
contract, upstream contract status, applicable REQ ids, row/evidence assertions and comparison
status. Unobserved conceptual fields SHALL be marked `PROPOSED_NOT_IMPLEMENTED`; no canonical key
may be invented. A typed input interface SHALL be tagged as a record, not a registered table. The
contract order SHALL proceed from impl headers/signatures through name-resolution and namespace
inputs, then body/signature sites and spans, with publication/control contracts distinguished from
semantic inputs.

The source checkpoint SHALL validate actual pinned `src/lib.rs` bytes and record the exact
zero-based half-open range for `Buffer::new()` inside `<Buffer as Default>::default`; generation
linkage SHALL be unverified absent matching file-source export evidence. Site evidence and exact
resolved target evidence SHALL have separate statuses. Backward verification SHALL validate the
required export set, exact table-name headers, generation/attempt consistency and stage marker
provenance; banner summaries are not frozen-stage proof. Export failure is `ERROR`, a missing
required export is `BLOCKED`, and counts or heuristic matches SHALL never produce `PASS`. Overall
verification remains `BLOCKED` until canonical mapping and full HLD comparison exist. Exact wrong-target
same-count comparisons SHALL fail when canonical rows are comparable and otherwise remain
`BLOCKED`. A pin or source-digest mismatch SHALL fail before generation.

This requirement builds reference/contract tooling only. It does not implement a mapper, a table
producer, an input decoder, or a production semantic rule; Phase 2 remains the five-repository gate.

#### REQ-RUST-040.0 — original-file top-level containment slice

WHEN `map-workspace-full-run` receives `--mapping-final-stage-kind original-top-level-containment`,
THEN it SHALL preserve the default `name-resolution` behavior when the option is absent and SHALL
publish only the explicitly scoped structural product: one `FilePublicArea` node for every included
source file reached through source closure, including files with no items, and `contains` edges from
that area to cfg-active, original-file top-level item nodes. A normal Rust module declaration is
source closure, not macro expansion: its origin kind SHALL be persisted as
`source_request_origin_kind = "module_declaration"`, and a module request whose parent has empty
expansion provenance SHALL keep an empty `file_expansion_provenance_chain`. Include/macro source
requests SHALL retain nonempty expansion provenance (`source_request_origin_kind = "macro_include"`)
and SHALL NOT create original-file areas or definitions. Only reached `items` requests with empty
expansion provenance after that distinction create areas. This slice SHALL exclude nested,
associated, function-body and macro-expanded items and SHALL NOT claim semantic dependency
completeness or resolve calls, types, imports, reads, impls or reexports. `use` statements remain
top-level nodes; `contains` and `imports` are direct-only and cannot be collapse hops, while the
existing six decided dependency kinds retain their collapse behavior.

`FilePublicArea` SHALL use a graph-only typed identity keyed by compilation-unit identity and a
normalized source-relative file path. It SHALL be distinct from `NameSymbolTargetKey` and from
span-bearing `SemanticOccurrenceKeyValue`; no fake occurrence or namespace symbol may represent a
file. Normal item endpoints SHALL wrap their existing `NameSymbolTargetKey` in
`HldTargetIdentityValue`. Source ownership and cfg activity SHALL be derived from the persisted
`file-source-text-table`, `span-reference-text-table` and `span-byte-ownership-ranges` rows, not
labels or hard-coded source facts. Frozen graph codecs SHALL use the observed row fields: node
target identities in `node_target_identity`, edge sources in `edge_source_target_identity`, edge
kinds in `edge_relation_kind`, resolutions in `edge_resolution`, and canonical edge evidence in
`edge_evidence_identity`. Acceptance assertions SHALL decode canonical unit and occurrence
subfields from those identities and derive the canonical unit independently from the root
namespace scope mapped by the raw crate-graph unit key.

The final-stage value, generation configuration, source-coverage marker and HLD publication marker
SHALL all carry the exact scope. `HldSourceCoverageKind` SHALL distinguish full semantic scope from
original-top-level containment and SHALL be orthogonal to `HldInputCompletenessKind` and
`HldOutputCompletenessKind`: `CompleteTypedInput` and `CompleteGraph` mean only that supplied typed
records are complete/materialized, not that semantic source was fully analyzed. Local item
visibility and outside-crate reachability are separate facts. Public items remain selected even
under private ancestors; unproven reachability is `None`, not false. Full-semantic input rejects
unknown reachability for ordinary local items; structural scope permits it and records the unknown
coverage diagnostic. An unresolved semantic dependency SHALL prevent a complete semantic-source
claim. The five existing HLD tables and existing full-graph input tables SHALL be used; no parallel
provisional database or production fixture input is allowed.
The existing strict full-semantic publication prerequisites SHALL remain unchanged, while a
distinct structural prerequisite set SHALL be selected from the same frozen configuration and
typed scope. Publication SHALL verify generation, attempt, config and stage-marker scope and remain
atomic.

The pinned-itoa acceptance test SHALL prove that persisted bytes, occurrence identity, active cfg
state and top-level ownership identify the original `src/lib.rs` declaration of `Buffer`, then
trace the file-area and `Buffer` identities through exactly one direct `contains` edge and its
evidence path. `Buffer`'s edge SHALL be present in the matching deduplicated HLD edge set. Each
roll-up count SHALL equal the cardinality of that exact HLD edge set after matching unit, file or
folder path, relation and via kind; it SHALL NOT assume Buffer is the bucket's only item. It SHALL
match product rows to full-graph input and source evidence by canonical identity, not display name
or aggregate count alone. A renamed-source fixture SHALL exercise public/private/use/inline-module,
nested-body, cfg-off, empty reached-file, and source-mutation cases without hard-coded itoa names.

WHEN an original-file item has restricted visibility, THEN classification SHALL use its owning
module and resolved visibility scope, not a `pub(` prefix. `pub(self)` and a restriction resolving
to the item's own module SHALL be private-equivalent. The full graph SHALL retain these items;
ordinary private-equivalent items SHALL not become interface nodes. Top-level use-node selection
remains the explicit exception. A restriction reaching an ancestor module SHALL remain distinct
from public visibility and from outside-crate reachability. Unknown restriction scopes SHALL fail
closed rather than acquire public/non-private visibility by spelling.
WHEN an original source file belongs to a child module, THEN its item owner kind and qualified
path SHALL come from frozen namespace ownership, not `CrateRoot` or the physical filename alone.
File-area identity SHALL remain separate from module identity. Same-named declarations in different
modules SHALL retain distinct canonical identities and module-qualified paths.

#### REQ-RUST-041.0 — frozen full-graph storage and scoped HLD publication

WHEN typed graph input is written from a frozen source snapshot, THEN the L2 graph-input store SHALL
verify its frozen revision, resolved base generation/attempt, typed invariants, frozen workspace
coverage configuration, and root namespace unit provenance before writing the registered
`item-effective-visibility-records` and `edge-evidence-record-table` tables. It SHALL complete both
graph owner stages and return a fresh analysis-only frozen snapshot without changing the workspace
serving manifest. The reader SHALL reconstruct the typed input only from those persisted rows plus
the snapshot's workspace configuration, raw crate graph unit mapping, and root
`syntax-namespace-input-scopes` rows. A complete marker with zero rows is a valid empty input; a
missing marker is a typed error. Malformed, duplicate/conflicting, mismatched-scope, mismatched
generation/attempt, or stale-after-freeze rows/markers SHALL be rejected.

Structural marker payloads SHALL carry
`mapping_scope_kind = "original-top-level-containment"` and
`source_coverage_kind = "original-file-top-level-only"`; those values SHALL match frozen config and
the typed input/output exactly. Legacy FullSemantic config/markers with no scope keys remain
compatible and SHALL NOT be upgraded to structural scope. Structural publication SHALL require
crate-graph, source-ingest, syntax-spans, syntax-namespace-input-records, name-resolution,
item-effective-visibility-stage and semantic-edge-evidence-stage, without requiring signature
lowering; FullSemantic prerequisites remain unchanged. Configured required-stage gates still apply,
including an HLD stage required by the same final transaction. The five HLD outputs, completion
markers, final snapshot and workspace-serving manifest SHALL commit or roll back atomically.
Structural unknown local-item reachability SHALL remain `None` and be counted in the HLD completion
diagnostic; FullSemantic unknown local-item reachability SHALL be rejected.
Graph owner completion markers SHALL preserve the original source snapshot identity, base scope and
revision. Assembly and final publication SHALL recheck the exact source-stage closure at that
recorded revision; source work added after graph derivation SHALL not be laundered by a newer graph
snapshot.

#### REQ-RUST-042.0 — generic supertrait method dependency

WHEN a method-call receiver has a generic parameter type with bounds leading through a trait to
a supertrait, THEN method lookup SHALL use the resolved receiver type, bound declarations and
supertrait member identities. If exactly one applicable declaration is proven, it SHALL emit one
extracted `calls` dependency at that site to the canonical trait method, with generic dispatch
distinguished from a selected concrete implementation. Name equality alone SHALL not select a
target. Missing, ambiguous or unsupported lookup SHALL retain one typed unresolved reason and
SHALL NOT emit an extracted edge for that site. Cyclic bounds SHALL terminate without fabricating
a target. Lexical shadowing SHALL use the receiver binding at the call site.

The pinned-itoa checkpoint is `Buffer::format -> private::Sealed::write` at `i.write(...)`:
`i: I`, `I: Integer`, and `Integer: private::Sealed`. The expected generic edge SHALL NOT be
expanded into calls to every integer implementation. Associated type projection `I::Buffer`
SHALL retain its trait-member identity; an unknown concrete substitution SHALL not be guessed.
This bounded contract does not claim general method lookup, concrete implementation selection or
whole-body inference is complete.

The fact SHALL survive source-derived production records, full-graph persistence and frozen
readback under REQ-RUST-044.0, and its evidence SHALL trace to the
original body-site bytes and canonical endpoint declarations. Renaming the source trait/method
SHALL change the corresponding identities without changing production logic. A unit-fixture
ambiguity test is not a claim that an ill-typed source crate passes compilation.
Later HLD derivation/publication SHALL preserve that fact and evidence under the retained
HLD contracts; those derived outputs are not active Phase 1 prerequisites.

REQ-RUST-033.0–036.0 and the full-map contract REQ-RUST-044.0 govern active verification.
REQ-RUST-037.0/041.0 still govern any later HLD assembly/publication. No semantic call may be smuggled into
REQ-RUST-040.0 structural-only output or published by bypassing full-semantic source/marker gates.
A separately bounded semantic publication scope, if needed, requires an explicit contract before
implementation; absent those prerequisites the publication checkpoint is BLOCKED, not GREEN.
This local checkpoint does not waive sysroot provenance or the whole-itoa Phase 1 exit.

### 3.5 Engines, storage and generations

#### REQ-RUST-005.2 — write side and analysis side

Turso is the concurrent write side; DuckDB is the analysis side.
No third database, LLM, salsa or native rust-analyzer semantic crate is allowed.
Rust owns syntax, macro expansion and per-body work, not a global definition map.

#### REQ-RUST-016.2 — DuckDB owns relational analysis

DuckDB SQL owns signature joins, impl/trait indexes, method
candidate sets, HLD recursive chain collapse and rollups. Rust lowering and
inference are bounded per body. Do not restore the in-memory resolver fallback.

#### REQ-RUST-010.2 — dependency boundary

WHEN the production binary is built, THEN cargo tree SHALL contain no ra_ap_hir*, ra_ap_ide*,
ra_ap_base_db or salsa; parseltongue-core-row-types SHALL have zero dependencies.
Copy-adapt mbe into parseltongue-port-macro-expander (L2); never link unchanged mbe with its salsa
dependency. If span is needed, link it with default-features = false and verify the full dependency tree.
Offline harness ide/hir/salsa dependencies SHALL not leak into the production binary.

#### REQ-RUST-020.1 — generations and stage isolation

WHEN a stage runs, THEN read only declared tables and rewrite only its generation's declared outputs.
Rows and markers SHALL carry run_generation_digest_value (§5); reject missing, stale or incompatible
input markers with StageInputMissingError. Identical inputs yield identical semantic output rows, excluding
operational timing/counter metadata. Shared library rows use the same compatibility key.
A new generation is invisible until atomically published; beginning a rerun invalidates its staging completion,
not the previously published generation. No cross-database partial generation may become visible.

#### REQ-RUST-021.1 — concurrency

WHEN N workers write, THEN use Turso BEGIN CONCURRENT per unit, permit at most N units in flight,
and preserve exactly the 1-worker row set/digest. Retry a conflict at most 5 times after the initial attempt
(6 total attempts), with delays 10, 20, 40, 80, 160 ms; rollback and reread/recompute on retry.
Only conflict errors retry; other errors surface immediately. Exhaustion returns StoreWriteConflictError.
Cancellation interrupts backoff. No unbounded queues or cross-unit memory retention.

#### REQ-RUST-022.1 — cancellation

WHEN Ctrl-C/SIGTERM interrupts a stage, THEN rollback any uncommitted unit, publish no partial unit
or new generation, and leave no completed marker for the interrupted staging stage.
Already committed staging units may be reused only under the same compatibility key.
Rerun SHALL equal an uninterrupted run. A previously published generation remains readable and immutable.

#### REQ-RUST-024.1 — published-generation copy

Copy only a COMPLETE PUBLISHED Turso generation using the Rust
duckdb Appender. No SQLite extension, database ATTACH, runtime download,
INSTALL or LOAD. Before any analysis, verify per-table row counts AND content
digests agree between Turso and DuckDB. Preserve keys, exact payloads,
generation/attempt identity and atomic snapshot completion; cancel/failure
leaves no usable partial snapshot.
Required RED: `published-generation-copy-equality`.

#### REQ-RUST-024.2 — generation keys and product copy-back

Every DuckDB row carries its generation key. DuckDB is the
analysis source of truth; only final HLD/product tables copy back into Turso,
with a digest check. Rust macro outputs are write-side evidence, not copied
analysis intermediates. Required RED: `product-publication-digest-equality`.

#### REQ-RUST-024.3 — total ordering and thread determinism

Every consumed query uses ORDER BY a total key; every tie-break
is total. Canonical digests include the complete sorted binding set and all
determinacy/ambiguity state. Single-thread and multi-thread DuckDB exports must
be byte-identical. Required RED: `duckdb-thread-export-determinism`.

#### REQ-RUST-024.4 — engine isolation settings

Set autoinstall_known_extensions=false,
autoload_known_extensions=false, allow_community_extensions=false and
enable_external_access=false. Use bundled DuckDB without installing or loading
extensions, and keep temp_directory inside the run directory. Until all-five
DuckDB S5 results exist, do not set memory_limit or impose a memory cap; report
the effective DuckDB default, memory tags, phase duration and whole-process-tree
peak RSS under REQ-RUST-024.9 and SOP revision R9. Enforce the final
500,000,000-byte process-tree target only after all-five S5.
Required RED: `serde-analysis-residency-budget`.

#### REQ-RUST-024.8 — bounded-projected-namespace-handoff

WHEN copying published namespace inputs, the adapter SHALL copy only declared
consumed columns/tables, retain compilation/source identities and required
macro/product metadata, and verify row count plus length-framed content digest
of exactly the projected rows on both sides of the Appender transfer (R4).
Unknown non-namespace table copies retain their existing full-row contract.
All namespace appenders SHALL flush at bounded row/byte intervals and be
dropped before CHECKPOINT after a committed copy/normalization. The native
analysis database SHALL be file-backed. Diagnostics SHALL distinguish live
post-phase tag residency from peak allocations and report database paths,
physical storage segments and logical column byte widths. Small-fixture
resolution products SHALL remain identical across handoff policy changes.

Verification additions:

```text
req_id: REQ-RUST-024.8
test_id: test_namespace_storage_snapshot_contents
test_type: unit
assertion: file-backed path, ordered tag/table/column/segment measurements
metric: exact payload logical byte length and populated database path
```

```text
req_id: REQ-RUST-024.8
test_id: test_namespace_source_projection_identity
test_type: unit
assertion: declared columns preserve identity/metadata; generic rows byte-exact; invalid objects rejected
metric: exact projected JSON and unchanged row key
req_id: REQ-RUST-024.8
test_id: test_namespace_copy_projection_columns
test_type: integration
assertion: full-source count accepted while undeclared data never stored
metric: four rows verified through projected count/digest handoff
req_id: REQ-RUST-024.8
test_id: test_namespace_projected_product_equality
test_type: integration
assertion: independently reconstructed full-row handoff exports exactly the same resolved products
metric: byte-identical nonempty ordered products
req_id: REQ-RUST-024.8
test_id: test_namespace_appender_batch_limits
test_type: unit
assertion: row/byte thresholds flush and reset, including oversized row
metric: 1024 rows or 1048576 value bytes
```

#### REQ-RUST-024.10 — namespace-resolution-publication-receipts

WHEN the CLI maps a published workspace generation, THEN it SHALL order selected
compilation units dependency-first, publish Rust macro outputs through Turso,
and refresh frozen snapshots before continuing resolution. It SHALL publish
final DuckDB products to Turso only after fixed-point completion and verify
each product table's row count and ordered BLAKE3 row-leaf digest. It SHALL
publish a receipt containing those proofs for every product table, including
zero-row tables, before the name-resolution completion marker. Empty output
tables remain empty and receive no stale rows. A failed or cancelled attempt
SHALL not publish a completion marker or replace the previous workspace
generation. Re-running an unchanged source generation SHALL retain the same
semantic product counts and digests.

```text
req_id: REQ-RUST-024.10
test_id: test_empty_resolution_stage_rows
test_type: integration
assertion: empty path products remain empty, receipt proofs reconcile, and completion follows the receipt
metric: exact counts, ordered digest proof, published completion marker
```

```text
req_id: REQ-RUST-024.10
test_id: test_persisted_resolution_path_answers
test_type: integration
assertion: independently re-read nonempty published Turso products; unique keys, exact counts, and ordered BLAKE3 row-leaf digests match every receipt proof, including the zero-row product
metric: exact product-table set, row counts, unique keys, and per-table digest equality
```

```text
req_id: REQ-RUST-024.10
test_id: test_stage_rerun_is_identical
test_type: integration
assertion: deterministic rerun preserves semantic outputs and a failed rerun retains the previous generation
metric: stable generation/product counts, no failed-generation publication
```

#### REQ-RUST-024.11 — published-stage-coverage-status

WHEN the CLI emits a coverage banner, THEN it SHALL retain the existing
`stage_implementation_status_values` availability map and add exactly four
`stage_publication_status_values` keys: `crate-graph`, `source-ingest`,
`syntax-spans`, and `name-resolution`. Each publication status SHALL be
`complete` only when that stage's persisted `__complete__` marker exists in the
currently published workspace generation and attempt; otherwise it SHALL be
`pending`. With no published generation, all four statuses SHALL be `pending`.
Markers persisted in another generation SHALL NOT complete the current
publication status.

```text
req_id: REQ-RUST-024.11
test_id: test_syntax_publication_status_complete
test_type: integration
assertion: syntax-spans publication is complete only for its published marker; exact four-key map
metric: published marker-to-status equality
```

```text
req_id: REQ-RUST-024.11
test_id: test_source_publication_status_pending
test_type: unit
assertion: source-only published scope with no syntax marker has vacuous tiling but pending syntax publication while the availability map remains compatible
metric: availability/publication distinction, vacuous tiling and pending syntax status
```

```text
req_id: REQ-RUST-024.11
test_id: test_stale_publication_marker_pending
test_type: integration
assertion: a name-resolution completion marker in an un-published other generation does not complete the current publication
metric: published-scope marker isolation
```

```text
req_id: REQ-RUST-024.11
test_id: test_publication_status_without_generation
test_type: integration
assertion: a database without a publication reports all four statuses pending
metric: all-pending status map without publication
```

```text
req_id: REQ-RUST-024.11
test_id: test_persisted_resolution_path_answers
test_type: integration
assertion: a full published namespace run reports all four stage markers complete in its generation
metric: exact publication-status map for name-resolution completion
```

### 3.6 Memory and determinism

#### REQ-RUST-011.1 — memory

WHEN mapping rust-analyzer plus its selected library roots, THEN peak simultaneous summed resident memory
of the production process tree SHALL be ≤2 GiB (2 × 1024³ bytes). Include allowed child processes, exclude
offline harness/indexers; record process-tree samples and cgroup memory telemetry, not just parent time -v RSS.
Sample summed RSS every 10 ms and record short-lived child peaks with OS accounting; report sampling limits.
The gate uses the larger observed process-tree RSS peak, converted from bytes to GiB, not cgroup page-cache usage.
The threshold in force is REQ-RUST-024.5's; the 2 GiB figure above is the superseded gate.

#### REQ-RUST-024.5 — final memory target

REQ-RUST-024.5 supersedes the process-tree acceptance threshold in
REQ-RUST-024.4: the final target is peak RSS ≤500 MB (500,000,000 bytes) for
the whole process tree. Enforce this target after working end-to-end DuckDB M2
and S5 results exist for all five repositories. Until those S5 results exist,
prioritize correctness, do not optimize RAM or set an interim memory cap, and
record measured tree peaks, wall time, resolution rounds and DuckDB memory tags.
The historical 2 GiB gate and approximately 1.2 GiB engine cap are superseded.

#### REQ-RUST-024.9 — correctness-memory-observation-only

WHEN the correctness workspace is constructed before all-five DuckDB S5 results
exist, THEN it SHALL use DuckDB's effective default without setting an explicit
memory_limit. It SHALL report current_setting('memory_limit'), per-phase DuckDB
memory tags, elapsed time, and whole-process-tree peak RSS; it SHALL retain
run-local spilling, bundled-only extensions, and external-access denial. It
SHALL NOT enforce an interim RSS or memory-cap gate. After all-five S5 results
exist, it SHALL enforce whole-process-tree peak RSS ≤500,000,000 bytes.
Compatibility callers passing the former cap parameter SHALL NOT restore an
explicit limit.

```text
req_id: REQ-RUST-024.9
test_id: test_namespace_constructor_uses_default
test_type: unit
assertion: adapter effective memory_limit equals the independent DuckDB default and telemetry reports it
metric: exact setting equality, no explicit cap
```

```text
req_id: REQ-RUST-024.9
test_id: test_namespace_constructor_rejects_missing
test_type: unit
assertion: measured constructor rejects a missing run directory
metric: constructor error returned, no database opened
```

```text
req_id: REQ-RUST-024.9
test_id: test_legacy_constructor_ignores_limit
test_type: unit
assertion: compatibility constructor does not apply its former memory limit
metric: effective setting equals the independent DuckDB default
```

```text
req_id: REQ-RUST-024.9
test_id: test_constructor_rejects_invalid_inputs
test_type: unit
assertion: invalid compatibility-constructor inputs still return an error
metric: error propagated without opening an invalid analysis run
```

```text
req_id: REQ-RUST-024.9
test_id: test_namespace_probe_settings_limits
test_type: unit
assertion: phase diagnostics retain setting tags while reporting the default memory policy
metric: exact setting/tag values, no explicit cap
```

```text
req_id: REQ-RUST-024.9
test_id: test_namespace_phase_empty_success
test_type: unit
assertion: empty successful analysis still reports the effective memory setting
metric: diagnostics and phase completion are present
```

#### REQ-RUST-012.1 — determinism

WHEN the same source/config/tool compatibility key is mapped twice with 1, 4 and 16 workers, THEN sorted
semantic exports SHALL be byte-identical. Compare stable occurrence IDs and HLD including evidence counts;
exclude timings. Collision fixtures SHALL not merge unrelated semantic occurrences.

### 3.7 Comparator and oracle

#### REQ-RUST-021.2 — comparison in DuckDB and path partitions

S5–S7 comparison runs in DuckDB SQL with exact mapped-unit,
PRIMARY/SECONDARY and proc-macro policies unchanged. Publish an exhaustive
oracle-row partition. M2 scores module-scope paths, including module-bound
primitive names and enum variants. Type-relative/associated paths requiring
lookup through a type, generic parameter, Self, trait or qualified projection
are counted/reported but scored at M4–M6. Enum variant lookup alone is not
associated-item lookup. Classify using oracle semantic resolution/container
kind, not spelling heuristics. Unknown classifications are reported as blocked,
not silently dropped. Reconcile all partition counts to the oracle rows;
preserve exclusions and top disagreement categories. Zero denominators are
N/A or BLOCKED. Required RED: `module-associated-denominator-partition`.

#### REQ-RUST-026.0 — comparator-persisted-evidence-validation

WHEN the comparator validates a persisted oracle export for a source scope,
THEN it SHALL validate source scope, mapped unit configuration and context
resolution, the persisted oracle envelope and required arrays, actual
`oracle_proc_macro_rows` producer entries, typed enum/boolean/namespace
values, and required evidence strings. It SHALL reject malformed or failed
macro evidence, ambiguous or unmapped site units, invalid ranges/provenance,
and invalid reason or classification values before publication, and SHALL
never fabricate a production classification.

#### REQ-RUST-026.1 — comparator-semantic-identity-canonicalization

WHEN the comparator canonicalizes semantic identities, THEN it SHALL match
the producer semantic-unit digest exactly as `[cfg, features, edition, role]`
with no alternate digest shapes; distinguish host/target roles; normalize
package-relative paths from persisted roots; preserve site context and
expansion provenance; canonicalize target kind/namespaces and builtin
identity; merge split namespace evidence for identical targets; collapse exact
duplicates; and reject conflicting duplicates.

#### REQ-RUST-026.2 — comparator-primary-partition-semantics

WHEN the comparator partitions PRIMARY rows against production, THEN it SHALL
assign each PRIMARY raw row one explicit partition/reason while reconciling
duplicates; score only known module-scope sites; keep deferred and unknown
classifications explicit, with unknown blocked even when a reason exists;
exclude from the PRIMARY denominator only PRIMARY `cfg_inactive`; retain
production and SECONDARY reasons as evidence; preserve mismatch precedence
over exclusions; and use actual proc-macro evidence to exclude an empty PRIMARY
answer only when production is empty or absent, never for a wrong nonempty
target. An empty denominator SHALL report N/A.

#### REQ-RUST-026.3 — comparator-secondary-evidence-visibility

WHEN the comparator reports production-only and SECONDARY differences, THEN it
SHALL retain production-only paths separately without changing the PRIMARY
denominator; preserve SECONDARY-only KL1 and same-site changed-target evidence;
and keep a production-only site that also appears in SECONDARY visible in both
the production-only count and SECONDARY evidence.

#### REQ-RUST-026.4 — comparator-cancellation-safe-publication

WHEN the comparator builds or publishes an agreement generation, THEN it SHALL
carry source generation and attempt through copied, canonical, and evidence
DuckDB rows; qualify joins, filters, and ordering by both; verify per-table
counts and scope-framed digests; recheck source stability during reads and
before publication; and validate or cancel before publication. Source changes
or in-flight write cancellation SHALL preserve the prior agreement generation.
It SHALL persist to the distinct registered Turso agreement store without
mutating source evidence or exposing stale rows after repeated oracle changes.

#### REQ-RUST-026.5 — comparator-deterministic-semantic-export

WHEN the comparator exports semantic results under supported thread counts,
THEN it SHALL produce deterministic semantic rows and summaries across those
counts and SHALL separate operational attempt identity from semantic export
identity.

#### Authoritative REQ-RUST-026 test matrix

Each listed active test doc-tag is located under every matching requirement
identifier. Tests with duplicate names in different test modules/crates are
separately active tests; the shared name is listed once per requirement.

**REQ-RUST-026.0**
- `test_ambiguous_unit_contexts_fail`
- `test_invalid_context_kind_rejected`
- `test_invalid_reason_enum_rejected`
- `test_invalid_proc_flag_rejected`
- `test_invalid_namespace_enum_rejected`
- `test_unit_configuration_contracts_fail`
- `test_unmapped_site_units_rejected`
- `test_site_provenance_entries_validated`
- `test_site_evidence_required_values`
- `test_oracle_envelope_flags_strict`
- `test_scope_oracle_errors_block`
- `test_malformed_macro_record_rejected`
- `test_macro_failure_blocks_publish`
- `test_oracle_disagreements_keep_reasons`

**REQ-RUST-026.1**
- `test_site_context_identity_distinct`
- `test_exact_duplicates_collapse_rows`
- `test_conflicting_duplicate_payloads_fail`
- `test_expansion_provenance_separates_sites`
- `test_package_paths_normalize_lexically`
- `test_malformed_target_ranges_fail`
- `test_target_namespaces_are_canonicalized`
- `test_builtin_target_names_canonicalize`
- `test_builtin_namespace_changes_identity`
- `test_split_target_namespaces_merge`
- `test_role_changes_semantic_digest`
- `test_exact_comparison_publishes_turso`
- `test_oracle_disagreements_keep_reasons`

**REQ-RUST-026.2**
- `test_unknown_scope_is_blocked`
- `test_proc_wrong_target_mismatch`
- `test_proc_empty_answer_excluded`
- `test_proc_empty_primary_exclusions` — latest regression covers empty and absent production; wrong-target guard remains `test_proc_wrong_target_mismatch`.
- `test_blocked_partition_reconciles_duplicates`
- `test_enum_variant_module_scored`
- `test_primary_classes_partition_once`
- `test_external_source_blocked_rate`
- `test_primary_missing_production_nonmatch`
- `test_source_blocked_preserves_cfg`
- `test_production_reason_keeps_mismatch`
- `test_secondary_reason_nonimpact_primary`
- `test_empty_denominator_is_na`
- `test_unresolved_targets_remain_mismatch`
- `test_production_cfg_keeps_mismatch`
- `test_secondary_cfg_preserves_match`
- `test_cfg_wrong_target_mismatch`
- `test_unknown_cfg_remains_blocked`
- `test_primary_cfg_excludes_scored`
- `test_persisted_known_answer_matrix`
- `test_oracle_disagreements_keep_reasons`

**REQ-RUST-026.3**
- `test_extra_production_secondary_counts`
- `test_production_only_rows_visible`
- `test_primary_classes_partition_once`
- `test_external_source_blocked_rate`
- `test_changed_targets_persist_semantically`
- `test_persisted_known_answer_matrix`
- `test_oracle_disagreements_keep_reasons`

**REQ-RUST-026.4**
- `test_database_backing_uses_file`
- `test_published_comparison_is_deterministic`
- `test_comparison_settings_are_verified`
- `test_context_cleanup_sql_error`
- `test_context_cleanup_after_cancel`
- `test_exact_comparison_publishes_turso`
- `test_comparison_telemetry_survives_publication`
- `test_persisted_known_answer_matrix`
- `test_scope_oracle_errors_block`
- `test_oracle_agreement_registry_roundtrip`
- `test_cancel_prevents_agreement_publish`
- `test_source_changes_after_reads`
- `test_source_changes_before_publish`
- `test_write_cancellation_preserves_generation`
- `test_source_unpublished_generation_fails`
- `test_oracle_disagreements_keep_reasons`

**REQ-RUST-026.5**
- `test_published_comparison_is_deterministic`
- `test_thread_counts_export_identically`

#### REQ-RUST-027.0 — mapped-path-export-capability

WHEN the oracle source manifest reports the current mapped exporter capability,
THEN `oracle_export_support_kind` SHALL be `mapped_semantic_paths`. This value
describes the existing mapped PRIMARY/SECONDARY path-resolution exporter only;
it is metadata, independent of REQ-RUST-026 comparator behavior, and SHALL NOT
imply a PASS, HLD status, complete S4 semantic-family support, or corpus/S5
qualification.

WHEN building the mapped artifact cache, the live harness arguments SHALL be
`--workspace-source-root-path PATH --mapped-unit-manifest-path FILE
--oracle-artifact-cache-output FILE`. WHEN exporting a mapped path envelope,
the arguments SHALL be ordered as
`--workspace-source-root-path PATH --mapped-unit-manifest-path FILE
--oracle-path-export-output FILE --oracle-configuration-kind PRIMARY|SECONDARY`
followed, when used, by `--oracle-macro-cache-manifest FILE`. Each export SHALL
write one pretty JSON envelope consumed as a single JSON value, not JSONL.

The current route fixes target context to the compiler default, enables no extra
features, keeps default features enabled, and disables test targets. It does not
implement the full invocation-expansion, signature, body-site, impl, method,
visibility, or reachability families; it also does not emit per-site expansion
provenance or proc-macro-dependence fields. Those broader S4 families remain
requirements and are not qualified by this metadata value.

Verification mapping:

```text
req_id: REQ-RUST-027.0
test_id: test_oracle_capability_reports_paths
test_type: unit
assertion: source manifest reports the limited mapped-path capability value
metric: exact oracle_export_support_kind value mapped_semantic_paths
```

### 3.8 Diagnostics and test harness

#### REQ-RUST-024.6 — namespace-analysis-phase-diagnostics

WHEN copying, normalizing or resolving a frozen generation fails, the L3
analysis boundary SHALL return a distinct thiserror phase variant with the
original error as its source, not flatten the phase into a text-only store
error. Every phase SHALL emit start/completion status, elapsed seconds and
DuckDB memory/temporary-storage byte counts from duckdb_memory(), together
with database sizes from pragma_database_size(). Telemetry failure SHALL
not replace the original phase failure. Retained native fixture input SHALL
support separate phase replay at 1,288,490,188 bytes without re-ingestion.

Verification: `test_namespace_phase_failure_context` forces each phase to
fail and checks its variant/source; `test_namespace_phase_empty_success`
checks empty success and real memory diagnostics.

Additional verification: `test_namespace_telemetry_failure_preserves` keeps
the original failure when observation fails; `test_namespace_query_explain_rows`
and `test_namespace_query_explain_error` cover normal/empty/error operator
profiling; `test_namespace_insert_plan_rollback` proves diagnostic insertion
profiles leave preexisting candidate rows unchanged on success and error.

#### REQ-RUST-024.7 — namespace-insertion-hypothesis-probes

WHEN diagnosing bounded namespace insertion, rollback-safe probes SHALL
report effective DuckDB version, threads, preserve_insertion_order and memory
limit, compare one setting at a time, and retain the frozen source. Probe
threads SHALL be limited to one or two and insertion-order preservation SHALL
remain disabled. The selected production correction SHALL preserve complete
small-fixture exports and finish resolution at 1288490188 bytes. Copy and
normalization RSS attribution SHALL distinguish measured allocator stacks
from tracked DuckDB bytes and from unassigned process RSS.

Verification matrix addition:

```text
req_id: REQ-RUST-024.7
test_id: test_namespace_probe_settings_limits
test_type: unit
assertion: effective false insertion order, threads 1/2 only, cap unchanged
metric: exact current_setting values and rejected invalid thread counts
```

```text
req_id: REQ-RUST-024.7
test_id: test_namespace_materialization_probe_rollback
test_type: unit
assertion: unconstrained CTAS followed by constrained INSERT; rollback on normal, empty and constraint-error cases
metric: original candidate rows unchanged; both successful operator profiles returned
```

#### REQ-RUST-028.0 — child-stderr-live-visibility

WHEN the CLI integration-test harness launches a workspace-mapping child,
THEN it SHALL create retained stdout and diagnostic-stderr files beside the
fixture database before spawning the child, and diagnostic stderr bytes SHALL
be readable from the file while the child is still running. A retained native
fixture SHALL keep that file under `PARSELTONGUE_NATIVE_FIXTURE_DIRECTORY` so
an external timeout does not remove already-written diagnostics.

AND the helper SHALL return the child's exact exit status, stdout bytes, and
stderr bytes to existing assertions, including for a nonzero exit. Spawn and
output-read errors SHALL be surfaced, and dropping an unfinished capture SHALL
terminate and reap its child.

##### Test-harness-only L3 design

This is a CLI integration-test harness defect, not a mapper, resolver, or
semantic-output defect. Private test utilities use `std::process::Command`,
`Stdio`, and adjacent files at the L3 boundary; no production module, trait,
dependency, pin, lockfile, or mapper behavior changes. Unique per-process
capture paths prevent concurrent fixture collisions. A controlled test-harness
subprocess writes a diagnostic, signals readiness, then blocks on parent input;
the test reads the retained file before releasing it and uses bounded readiness
and exit checks. The existing helper waits for normal child completion and
reconstructs `std::process::Output` from the captured files.

##### Authoritative REQ-RUST-028 test matrix

- `test_child_stderr_live_visibility` — observes the diagnostic bytes before a synchronized child can finish.
- `test_child_output_exit_preservation` — checks fixture stdout/stderr payload bytes and exit code 23.
- `test_child_spawn_error_propagation` — checks a missing executable returns `NotFound`.
- `test_child_output_read_error` — checks a missing captured output file returns `NotFound` after completion.
- `test_child_fixture_waits_release` — ignored fixture process invoked by live-visibility, exit-preservation, and output-read cases; not an independent runtime gate.

#### REQ-RUST-028.1 — per-refresh-namespace-operational-trace

WHEN the namespace pipeline starts a frozen-input refresh, completes its copied
source inputs, resolves those inputs, or exhausts a unit's macro-expansion port,
THEN it SHALL emit unconditional operational stderr diagnostics with structured
JSON values serialized by `serde_json`. Refresh-start diagnostics SHALL identify
the generation and frozen attempt, an explicit initial state or the triggering
unit and outer expansion ordinal, completed invocation occurrence identities,
and emitted semantic-row count. After copy, the pipeline SHALL report a stable
source INPUT digest composed from existing verified copied-table manifest table
names, row counts, and content digests in deterministic table order; the digest
SHALL exclude snapshot identity and timing and SHALL NOT rescan or rehash copied
row payloads. Successful resolver diagnostics SHALL report the binding digest
and inner SQL fixed-point round count separately. Per-unit macro-port exhaustion
SHALL be emitted only after the port returns `None`. These diagnostics SHALL not
change resolution scheduling, SQL, publication semantics, memory policy, or
semantic output tables.

##### Authoritative REQ-RUST-028.1 test matrix

- `test_generated_import_refresh_trace` — proves a dependency-generated public
  type resolves at its importing consumer and checks the operational refresh,
  copied-input digest, fixed-point, and post-`None` exhaustion trace.
- `test_source_digest_snapshot_stable` — proves equal copied content has the
  same source-input digest across snapshot identities and changed manifest
  content changes that digest.
- `test_source_digest_empty_rejected` — proves an empty copied-manifest set
  cannot produce a source-input digest.
- `test_source_digest_duplicates_rejected` — proves duplicate table manifests
  cannot produce an ambiguous source-input digest.

#### REQ-RUST-028.2 — namespace-import-statement-trace

WHEN the frozen namespace resolver runs the checked-in
`namespace-import-binding-rounds.sql` for an inner fixed-point round, THEN it
SHALL validate the complete fixed-script boundary contract before executing
any statement, preserve every SQL byte and its declared order, and execute
each of the nine statements with the existing DuckDB `execute_batch` API,
without adding a transaction. For every statement it SHALL emit JSON-serialized
operational `started` and `complete` or `failed` stderr events carrying the
generation, attempt, inner round, stable one-based ordinal, stable statement
name, and elapsed milliseconds. On the first DuckDB error it SHALL emit the
failure event, propagate the same underlying error through the existing
`NameResolutionStageError` boundary, and execute no later statement. An
unsupported or changed boundary contract SHALL fail before any statement is
executed or statement event is emitted. SQL semantics, binding digests,
publication behavior, fixed-point scheduling, engine settings, and semantic
outputs SHALL remain unchanged.

##### Authoritative REQ-RUST-028.2 test matrix

- `test_import_sql_statement_boundaries` — checks the nine fixed named slices
  reconstruct the production SQL byte-for-byte.
- `test_sql_quoted_comment_boundaries` — checks quoted and commented
  semicolons are not treated as statement boundaries.
- `test_import_sql_rejects_unsupported` — checks changed/unsupported scripts
  fail before SQL execution or diagnostics.
- `test_import_batch_output_equivalence` — compares the old whole-batch and
  traced production SQL over direct-import, glob, macro-prelude, and visibility
  inputs, including exact sorted rows and binding digest.
- `test_import_batch_trace_markers` — checks successful per-statement JSON
  markers, identity fields, order, and elapsed-duration fields.
- `test_import_batch_failure_equivalence` — compares an injected middle
  statement failure under autocommit and explicit transaction rollback,
  including underlying error propagation and no later statement execution.

## 4. Rules

### Matching and scoring

Normalize by workspace-relative or package-source-relative paths, UTF-8 byte ranges, compilation-unit key,
site kind and deterministic expansion provenance/invocation key. External targets use package source ID,
target name/kind and qualified item path, not crate@version alone. Align harness and production keys
before scoring. Duplicate occurrences collapse by this key; different compilation units never collapse.

- 005: numerator = exact target matches; denominator = eligible PRIMARY use/signature path sites.
  Missing/unresolved production answers count as nonmatches.
- 006: numerator = invocations with equal canonical item tuples AND body-site tuples; denominator =
  eligible PRIMARY declarative/built-in invocations. Compare sets sorted by provenance and occurrence;
  a missing expansion is a nonmatch. Body tuples include site kind, range/provenance and callee syntax;
  resolved call targets are scored by 007, not required to finish the M2 expansion gate.
- 013: numerator = matching invocation item/signature sets; denominator = all matching-shape PRIMARY
  invocations. Require a nonempty fixture for every shape row, including versions, plus the unknown-version case.
- 018: numerator = exact canonical relation tuples present in both; denominator = union of PRIMARY and
  extracted production signature/impl relation tuples. This penalizes both missing and extra relations.
- 007: numerator = correctly targeted extracted method sites; denominator = all extracted method sites.
  A site absent from PRIMARY but emitted as extracted counts as incorrect.
- 008: numerator = correctly targeted extracted method sites; denominator = eligible PRIMARY method sites.
  A wrong extracted answer does not increase coverage.
- 019: numerator = nodes with correct selection/effective visibility/reachability; denominator = union of
  PRIMARY and production candidate occurrence keys; require 100%, including correct rejection cases.

Eligibility excludes inactive cfg and source absent under PRIMARY's KL1 limitation; record each exclusion with
oracle evidence. PRIMARY-visible unknown_receiver, trait_solver_needed, proc_macro and external_source_missing
remain coverage/path failures as applicable, not removed denominators. Secondary-only sites are recorded
in the KL1 difference set and excluded from PRIMARY denominators. No exception for known oracle errors:
those block the gate. M2's temporary proc-macro exclusion is separately counted and ends at M3.
Compute every gate per repository, never pool the five to hide a failure. Empty denominator = N/A, not 100%;
corpus gates require at least one eligible observation per metric/repo or are BLOCKED. Empty fixture cases
assert zero rows without claiming percentage success. Report raw numerator, denominator, exclusions and rate.

### Authoritative unresolved reasons

SiteUnresolvedReasonKind is the only reason enum for all stages; enum values remain exempt from naming.
Exactly one primary reason per unresolved site; select the earliest causal blocker in this precedence:
cfg_inactive → build_script → proc_macro → external_source_missing → unknown_receiver → trait_solver_needed.
Store additional causal evidence separately, not additional primary reasons.

- cfg_inactive: outside active analysis; counted, excluded from PRIMARY denominators.
- build_script: unavailable generated source/env; KL1 exemption requires a matched PRIMARY/SECONDARY diff.
  If PRIMARY exposes the site anyway, unresolved production remains a nonmatch.
- proc_macro: unknown/unsupported shape; counted, remains a nonmatch when PRIMARY exposes the site.
- external_source_missing: counted; gate BLOCKED if the required oracle source is missing too,
  otherwise unresolved production counts as a nonmatch.
- unknown_receiver: counted; unresolved production is a nonmatch, not an exception.
- trait_solver_needed: counted; unresolved production is a nonmatch, not an exception.

### Required resolution fixtures

Each fixture has a normal, empty/inactive and conflicting/error case under test_module_resolution_fixture_rules.

- #[path] and nested modules: resolve relative to the declaring module's directory, including inline-module
  path rules; cfg_attr is evaluated before module discovery. Store reached ordinary files in the closure.
- include!: resolve relative to the invocation's source file, preserve included-file provenance and hygiene;
  nested includes join the closure. include_str!/include_bytes! read bytes only. Never follow target/ output.
- Hygiene: persist syntax contexts, definition/call-site provenance and $crate's defining crate; test
  identical macro text invoked in two crates without binding leakage.
- Globs/aliases: fixed point with explicit-import precedence, namespace separation and ambiguity detection;
  ambiguous bindings cannot become extracted targets. Reexports enumerate resolved exported bindings.
- Extern prelude: edition-specific 2015/2018/2021/2024 rules, renamed dependencies, no_std/no_core and
  implicit std/core/prelude suppression. Tests include multiple versions and identical names from distinct sources.
- Target/context fixtures: lib+bin sharing one file, differing features/cfg, host proc-macro context,
  sysroot target cfg, test cfg OFF and excluded dev/test/bench/example targets.

### Visibility and collapse rules

Private-equivalent restrictions such as pub(self) do not qualify as non-private visibility.
Effective visibility follows Rust item-kind semantics: trait members and trait-impl members inherit their
trait/interface visibility; enum variants inherit enum visibility; inherent members use declared restrictions.
A node's local visibility is distinct from external reachability through ancestor modules and reexport aliases.
Fn-body ancestry always excludes nodes, including inline modules nested in a body.
The exporter and tests compare the same item-kind visibility model. Do not infer external reachability from pub alone.
Reexports use REQ 019.2's nearest selected enclosing module/root endpoint and retain the lexical module in evidence.

These rules apply to the interface level; the full graph stores every node and edge and is never
collapsed (REQ-RUST-032.0). Emit direct resolved relations first. For each selected source, traverse
resolved evidence through private items, never along a `contains` or `imports` edge, stopping at the first selected/external target; retain the terminal relation kind for collapsed edges.
Use visited occurrence keys per path to stop cycles; enumerate canonical simple paths in sorted order.
Keep one HLD row per (generation, source, target, relation kind, via kind); retain each distinct evidence path
and count unique evidence-row keys across those paths. Diamonds merge the HLD row, not provenance.
Retain genuine direct/finite-path self-edges once; never invent a self-edge from a cycle alone.
An unresolved link stops that path without an extracted successor. Roll-ups count HLD rows, not path multiplicity.

## 5. Persisted interfaces

### Identity, columns and generations

SpanDigestIdValue is a 16-byte truncated CONTENT identity only. Retain its complete 32-byte BLAKE3 digest
and canonical digest inputs; on truncation collision, preserve both full-digest records, never alias them.
SemanticOccurrenceKeyValue identifies (compilation unit, normalized file path, byte range, full content
digest, expansion provenance/invocation chain). Source bytes alone never identify a semantic occurrence.
CrateDenseIndexValue is assigned by sorted Cargo package ID + target name/kind + host/target context,
not sorted name/version alone. Alias is an edge property. Stable full semantic keys survive dense-ID rebuilding.
RunGenerationDigestValue hashes source snapshot digest, normalized per-unit configuration and tool versions
(including parser, shapes and schema). Identical inputs may reuse a generation; operational attempts are separate.
Source snapshots cover immutable input roots and lockfiles, excluding .git, target/ and oracle output locations.
An identical-key rerun uses a separate staging attempt; it never overwrites the immutable published attempt.
Publication selects that completed attempt, while deterministic semantic exports omit the attempt sequence.

Owned row fields/SQL columns (four semantic words, underscores in code; hyphens in artifact names):

```text
run_generation_digest_value: compatibility key on every semantic row and marker
stage_attempt_sequence_value: operational staging attempt, excluded from semantic identity/export
stage_unit_work_key: atomic file/crate/body work-unit identity
stage_input_generation_keys: exact compatible upstream generations
stage_owned_table_names: declared stage input/output table inventory
unit_cargo_package_identifier: full Cargo package ID, including source
unit_cargo_target_name: Cargo target name
unit_cargo_target_kind: lib/bin or explicitly tagged host dependency context
unit_dependency_alias_text: dependency edge's local alias
unit_target_cfg_values: normalized cfg set
unit_enabled_feature_names: normalized per-unit feature set
unit_rust_edition_value: selected edition
unit_compilation_role_kind: host or target
unit_source_root_path: normalized immutable source root
file_source_relative_path: normalized source-root-relative path
file_source_content_bytes: byte-exact input bytes
file_content_digest_full: complete BLAKE3 file digest
file_dependency_origin_kind: workspace/std/external source classification
span_content_digest_full: complete 32-byte content identity
span_semantic_occurrence_key: context-qualified occurrence
span_source_file_path: source-root-relative occurrence path
span_item_kind_value: parsed item kind
span_declared_visibility_kind: explicit syntax visibility
span_byte_start_offset: inclusive byte offset
span_byte_end_offset: exclusive byte offset
span_macro_invocation_key: invocation semantic occurrence
span_expansion_provenance_chain: syntax context and invocation ancestry
node_reachable_outside_crate: external reachability, yes/no
node_effective_visibility_scope: item-kind visibility after restrictions
site_unresolved_reason_kind: exactly one authoritative causal blocker
site_callee_syntax_text: expanded call's syntactic callee, separate from its resolved target
oracle_primary_target_reference: PRIMARY semantic target
oracle_secondary_target_reference: SECONDARY semantic target, if present
oracle_comparison_outcome_kind: match/mismatch/unresolved/excluded observation
oracle_semantic_record_kind: path/expansion/impl/call/signature/visibility observation
edge_source_occurrence_key: selected source occurrence
edge_target_reference_value: selected/external canonical target
edge_source_signature_text: assembled source signature
edge_target_signature_text: assembled target signature
edge_source_qualified_path: source path in compilation context
edge_target_qualified_path: canonical target path including package source/target
edge_relation_kind_value: calls/uses_type/implements/reexports/contains/derives/imports/reads (REQ-RUST-030.1)
edge_collapse_via_kind: direct/collapsed
edge_evidence_row_count: unique supporting evidence rows
edge_dependency_origin_kind: same_crate/workspace_crate/std/external_crate
edge_resolution_status_kind: extracted for HLD, unresolved allowed in evidence
stage_publication_state_kind: staging/complete/published
```

Use the above names in schemas; do not reintroduce short aliases in proposed owned structs.
SpanByteRangeValue rejects end before start. EdgeTargetReferenceValue carries semantic keys, not content IDs.
SiteUnresolvedReasonKind and BodySiteResolutionOutcome distinguish extracted and one-reason unresolved data.

stage-run-generation-records stores generation, stage/unit completion and publication manifests.
Workspace staging writes and shared-library staging writes are invisible. Commit immutable compatible library
rows first, then atomically switch the workspace publication manifest referencing those exact library generations.
Readers pin that manifest; library garbage collection cannot delete referenced generations. No distributed
transaction is assumed: library-first failures leave unreachable rows, not a partially published workspace.
A crate fixed point atomically publishes its bindings, spans and expansion rows within the staging generation.
A downstream stage requires the exact upstream generation/unit completion, not mere table existence.

### Trait seams and errors

```text
CrateGraphReadStore: read_crate_dependency_rows, read_unit_dependency_rows
SpanRowReadStore: read_file_span_rows, read_crate_span_rows, read_span_by_digest
FieldDetailReadStore: read_span_field_rows
BodySiteReadStore: read_body_site_rows
ScopeBindingReadStore: resolve_module_name_target
ImplHeaderLookupIndex: read_self_impl_rows, read_trait_impl_rows
LangItemReadStore: read_named_lang_item
ProcMacroShapeLookup: read_macro_shape_rows
StageRowWriteSink: write_stage_unit_rows
StageRunMarkerStore: begin_stage_generation_write, complete_stage_generation_write,
                     check_stage_generation_complete, publish_workspace_generation_manifest
```

Closure-discovery requests are persisted work records in stage-run-generation-records; ingest/walker
complete those records before the crate fixed point resumes. No file request exists only in memory
across stage boundaries. Stage markers distinguish a complete discovery batch from final closure completion.
Every lookup is generation/context-qualified; span lookup takes full digest plus occurrence context.
Typed boundaries: CrateGraphLoadError, WorkspaceNotCompilingError, NameResolutionStuckError,
StageInputMissingError, StoreWriteConflictError, StoreBackendFailureError. Unresolved rows are data,
not fatal errors. Missing module/include files or ambiguous compiler-invalid inputs produce
WorkspaceNotCompilingError; unsupported but compiling semantics produce a counted reason, never a guess.

### Turso and cancellation

Pin turso =0.9.0-pre.5. M: crates.io reports stable 0.8.3 and max prerelease 0.9.0-pre.5.
The inspected prerelease has async query/execute/prepare/build and TransactionBehavior::Concurrent.
parseltongue-turso-store-adapter owns a runtime and exposes synchronous traits; L2 holds no lock across await.
Use one connection/transaction per unit and the exact REQ 021 retry schedule; verify MVCC under 1/4/16 workers.
API availability is source-verified; throughput and conflict behavior are not yet measured.
A shared RunCancelFlagValue interrupts unit work/backoff; rollback before releasing an interrupted unit.

## 6. Verification matrix

Of the 205 test names in this file, 155 exist in the code today and 50 are planned. The code has 292 tests in total; those not named here
are supporting tests.

### Requirement → test → SOP crosswalk

All names below have four words after test_. S3 includes a separate fixture/build suite, so negative tests
do not mutate corpus checkouts. S5 percentages use §4 (matching and scoring); S6–S9 cross-checks do not override PRIMARY.
Every entry names a clause assertion, not merely a requirement-level coverage claim.

```text
001 → S3
  test_crate_graph_offline_only: selected package/target fields, aliases, direct/descendant argument allowlists.
  test_lockfile_guard_preserves_bytes: absent lock rejects before spawn; mutation rejects and restores bytes.
  test_metadata_failure_writes_nothing: missing download/nonzero exit preserves stderr, publishes zero rows.
002 → S3
  test_ingest_bytes_stay_identical: random bytes including invalid UTF-8/newlines stored unchanged.
  test_unchanged_digest_skips_write: unchanged bytes write zero content rows; new marker allowed.
003 → S3
  test_span_gap_tiling_identity: nested spans flatten to one exact ownership partition; empty file valid.
004 → S3
  test_parse_error_stops_run: active error rejects with path, no publication.
  test_non_utf8_stops_run: active invalid bytes include offset; bytes-only fixture does not reject.
014 → S3
  test_cfg_from_rustc_print: per-unit features/cfg/edition, host/target and sysroot contexts agree.
    Include resolver-version/normal-dependency feature isolation and print-only target arguments.
  test_inactive_cfg_items_counted: test cfg OFF, inactive reached items counted; unreachable files bytes-only.
015 → S3, S4, S5
  test_build_script_never_executed: exec/read audit forbids scripts and target reads, including symlinks.
  test_out_dir_include_unresolved: generated include and unknown env/option_env get build_script.
  test_cargo_env_from_metadata: metadata env values; build-only cfg OFF; per-crate banner reasons exact.
005 → S4, S5
  test_nameres_matches_oracle_paths: per-repo formula, absent answers and M2 exclusions.
  test_oracle_disagreements_keep_reasons: every mismatch/unresolved/exclusion persisted; oracle missing blocks.
016 → S3, S5
  test_macro_import_fixed_point: generated use resolves inside same crate publication.
  test_fixed_point_limit_errors: 8192 changing rounds allowed, attempted 8193 errors.
  test_module_resolution_fixture_rules: all §4 (resolution fixtures) normal/inactive/error variants.
006 → S4, S5
  test_macro_rules_items_match: item/body invocation sets and origin/invocation provenance.
  test_body_macro_sites_match: nested body expansions and matched call-site tuples.
013 → S4, S5
  test_macro_shapes_match_oracle: every invocation, each shape exercised, version/signature diff on failure.
  test_unknown_proc_macro_unresolved: absent shape/version produces one proc_macro reason.
017 → S3, S5
  test_lang_items_in_sysroot: recognized present items found, not hard-coded to 169.
  test_incoherent_impl_headers_indexed: M4 indexes sysroot primitive/flagged-type impls.
  test_incoherent_primitive_impls_indexed: M5 resolves f32::abs and str::len through the index.
018 → S4, S5
  test_signature_edges_match_oracle: resolved-or-reason type paths; union-denominator signature/impl agreement.
007 → S4, S5, S7
  test_method_target_precision_oracle: site-level ≥99%, extra/wrong sites penalized; calls reported separately.
  test_unresolved_has_one_reason: exact enum/precedence, unknown_receiver and build_script included.
008 → S3, S5, S10
  test_method_coverage_per_substep: report selected steps, mandatory ≥95% at M8, M7-skip case.
019 → S5, S9
  test_body_items_never_nodes: body ancestry rejects even pub nested modules.
  test_effective_visibility_node_selection: positive/negative item kinds and restricted visibility.
  test_pub_use_reexport_edge: aliases/globs one edge per binding, nearest selected module/root endpoints.
  test_reachable_outside_crate_flag: private parents and public reexports agree with PRIMARY.
009 → S6, S8, S9
  test_hld_rows_connect_nodes: selected/external endpoints, extracted-only status.
  test_private_chain_collapse_rules: cycles/diamonds/self-edges, unresolved stop, complete provenance/counts.
  test_hld_rollup_counts_match: file/folder aggregates match deduplicated HLD.
010 → S3
  test_no_analysis_crates_linked: production tree forbids ide/hir/base_db/salsa; L1 zero deps.
  test_macro_dependencies_exclude_salsa: copied mbe + optional span default-features=false has no salsa.
020 → S3
  test_stage_rerun_is_identical: stage outputs stable under same compatibility key.
  test_stage_refuses_missing_input: missing/stale/incompatible marker rejects.
  test_stage_table_access_restricted: spy-store rejects undeclared reads/writes.
  test_generation_publication_is_atomic: shared DB/library-first crash cases preserve old manifest.
021 → S3, S10
  test_concurrent_writers_lose_nothing: N=1/4/16 rows/digests equal.
  test_conflict_retry_is_bounded: six attempts, exact delays, nonconflicts not retried, cancellation interrupts.
  test_in_flight_units_bounded: observed maximum ≤N.
022 → S3
  test_cancel_then_rerun_identical: cancel within/between units, rerun equals clean.
  test_cancel_publication_stays_consistent: no partial reader visibility/completed staging marker.
011 → S10
  test_peak_memory_within_budget: process-tree peak ≤2 GiB, telemetry and limits recorded.
012 → S3, S10
  test_hld_export_is_deterministic: two runs per N=1/4/16, stable sorted semantic export.
  test_semantic_identity_collision_safety: forced truncation collisions, same bytes/different paths/contexts.
  test_oracle_matching_empty_cases: §4 (matching and scoring) empty/N/A, duplicate keys and per-repo aggregation.
009/019/020 → S6, S7, S8, S9
  test_crosscheck_evidence_never_overrules: CM-GAP cannot excuse oracle error; coverage pagination complete.
  test_sample_rows_record_keys: zero/small/large samples, explicit seed, every sampled miss fails.
028 → test-harness-only; no SOP/corpus execution
  test_child_stderr_live_visibility: diagnostic bytes are readable while the controlled child is blocked and alive.
  test_child_output_exit_preservation: fixture stdout/stderr payload bytes and nonzero exit status are preserved.
  test_child_spawn_error_propagation: spawn errors remain observable at the test-helper boundary.
  test_child_output_read_error: output-file read errors are returned after the child is reaped.
  test_child_fixture_waits_release: ignored subprocess fixture used by live-visibility, exit-preservation, and output-read cases.
```

Build gates: cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test;
cargo build --release; production cargo tree. Oracle corpus suite is separate/ignored plus nightly,
never linked into production. S1/S2 are external-tool preconditions/coverage checks, not semantic requirements.

### Build-order contracts (none written yet)

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| REQ-RUST-033.0 / 038.0 | test_itoa_pipeline_reads_published | real-source integration | digest-checked pinned itoa runs through the production entry point with sysroot scope `excluded` and no pre-resolved input | rows read from the published generation only |
| REQ-RUST-038.0 | test_sysroot_scope_excluded_units | integration | `excluded` adds no sysroot unit, records the scope, leaves `core` paths unresolved with a reason; an unknown value is a usage error | exact rows |
| REQ-RUST-038.1 | test_sysroot_reached_names_reasoned | real-source integration | on pinned itoa under `excluded`, names imported from `core` and std-prelude names are unresolved `external_source_missing`; type-relative paths are not relabelled | 13 + 4 sites reasoned; 16 type-relative sites keep no reason |
| REQ-RUST-034.0 | test_body_path_item_resolution | real-source integration | a body path resolves to its item; a shadowed name does not | exact target or one reason |
| REQ-RUST-035.0 | test_impl_header_row_contents | real-source integration | each active impl has one row with self type, trait and members | exact rows |
| REQ-RUST-035.0 | test_impl_target_namespace_metadata | unit | namespace-export decoration does not change target identity; different source ranges remain distinct | exact identity keys |
| REQ-RUST-036.0 | test_default_calls_buffer_new | real-source integration | `<Buffer as Default>::default` calls `Buffer::new` | exact source, target, evidence line |
| REQ-RUST-037.0 | test_hld_input_table_assembly | integration | typed HLD input comes only from the two full-graph tables; a missing marker is a typed error | exact records |
| REQ-RUST-032.0 / 019.2 | test_itoa_whole_graph_equality | real-source end-to-end | output equals the frozen target dataset | 0 missing, 0 extra, 0 wrong |

### Full graph and level contracts (none written yet)

These are required test contracts, not passing evidence, and none is written.
Component tests on typed fixtures do not replace the pinned-itoa real-source
acceptance of SOP §1.1.

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| REQ-RUST-032.0 | test_graph_private_item_nodes | real-source integration | private items outside fn bodies are nodes; body items are not | exact node set |
| REQ-RUST-032.0 | test_graph_private_edge_retention | real-source integration | private → private relations are stored uncollapsed | exact edge set |
| REQ-RUST-032.0 | test_graph_file_area_containment | real-source integration | one area node per source file, distinct from the module node, containing its top-level nodes | exact nodes and edges |
| REQ-RUST-032.0 | test_graph_use_statement_nodes | real-source integration | each cfg-active use outside fn bodies is a node; cfg-inactive ones are absent | exact node set |
| REQ-RUST-032.0 | test_graph_import_binding_edges | real-source integration | one imports edge per resolved binding, aliases and globs included | exact edge set |
| REQ-RUST-032.0 | test_graph_constant_read_edges | real-source integration | a const, static or associated-const read yields one reads edge | exact edge set |
| REQ-RUST-032.0 | test_graph_real_item_targets | real-source integration | an edge resolved through an import targets the item and keeps the use node as evidence | exact target and evidence |
| REQ-RUST-032.0 | test_graph_import_access_exclusion | real-source integration | a function has no edge to an import it does not use | zero such edges |
| REQ-RUST-032.0 | test_graph_inline_module_scope | real-source integration | an inline module is a module node and does not resolve through its file's imports | exact resolution |
| REQ-RUST-019.2 | test_level_filter_adds_nothing | integration | every interface-level node and evidence hop exists in the full graph | zero additions |
| REQ-RUST-019.2 | test_level_area_use_selection | integration | area and use nodes are selected, including in a file whose module is private | exact node set |
| REQ-RUST-019.2 | test_level_private_item_exclusion | integration | private items are unselected but named in collapsed evidence paths | exact rows and paths |
| REQ-RUST-019.2 | test_level_collapse_kind_exclusion | integration | selected -contains-> private -calls-> X and use -imports-> private -calls-> X emit no collapsed edge; direct contains/imports rows to selected targets remain | exact rows |
| REQ-RUST-030.1 | test_hld_added_relation_kinds | unit/integration | imports and reads round-trip through typed input, DuckDB and output | exact rows |

### Phase 1 reference dataset contracts

These offline Python tests validate reference tooling only. They do not establish that a Rust
producer or the production pipeline implements the associated row contract.

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| REQ-RUST-039.0 | test_candidate_inventory_rows_preserved; test_candidate_provenance_metadata_explicit; test_relation_kinds_include_zeroes; test_dangling_node_endpoints_block | offline Python | candidate rows are LLM_PARSED_DRAFT with no compiler/oracle freeze; complete inventory remains beside a distinct eligible local-only subset and blocked rows; all eight relation kinds are explicit; dangling node endpoints are blocked | exact provenance metadata; source row-set preservation; recomputed eligibility; zero eligible dangling endpoints |
| REQ-RUST-039.0 | test_collapse_excludes_forbidden_hops; test_forbidden_direct_external_targets | offline Python | contains/imports cannot participate in collapsed hops, but direct edges to selected, external or builtin terminals are retained | zero forbidden hops in collapsed evidence; exact direct terminal rows |
| REQ-RUST-039.0 | test_cycles_stop_repeating_nodes; test_diamond_paths_keep_provenance; test_direct_collapsed_keys_separate | offline Python | cycle termination, distinct diamond evidence, separate direct/collapsed keys | exact path/evidence sets |
| REQ-RUST-039.0 | test_nonextracted_statuses_stay_blocked; test_unresolved_reason_not_asserted | offline Python | unresolved/proposed/external-unverified and undecided rows are excluded without asserting unknown reasons | zero in eligible extracted rows |
| REQ-RUST-039.0 | test_wrong_target_count_rejected; test_mixed_generation_exports_error; test_export_failure_stays_error; test_invalid_export_payload_errors; test_missing_marker_stays_blocked | offline Python | exact same-count wrong target fails; mixed generation and malformed export payloads fail closed; missing marker is incomplete | exact status: FAIL, ERROR or BLOCKED, never PASS by count |
| REQ-RUST-039.0 | test_empty_graph_outputs_empty; test_rollups_count_unique_edges; test_rollup_edges_accounted_once; test_macro_rollup_stays_blocked | offline Python | empty input stays empty; roll-ups count unique HLD rows; every expected edge is counted once or explicitly excluded, including builtin targets | exact rows, exclusions, reasons and status |
| REQ-RUST-039.0 | test_dataset_output_is_deterministic; test_source_pin_digest_mismatch; test_syntax_semantic_gap_reported; test_contract_manifest_blocks_unbuilt | offline Python | deterministic bytes, pin enforcement, syntax/semantic distinction, fail-closed unbuilt contracts | byte equality and exact blocker statuses |
| REQ-RUST-039.0 | test_unflagged_contains_allowed; test_primitive_builtin_is_allowed; test_unknown_builtin_target_blocked; test_primitive_vocabulary_matches_source; test_all_six_relations_collapse; test_od9_does_not_suppress | offline Python | preserve decided containment and primitive vocabulary; block only flagged OD-7 ownership; apply current six-kind collapse | exact eligibility, collapsed relation set and unsuppressed OD-9 metadata |
| REQ-RUST-039.0 | test_wrong_target_site_rejected; test_unresolved_target_site_blocked; test_checkpoint_source_bytes_checked | offline Python | same-site wrong or unresolved targets are not semantic proof; source bytes and digest are checked from actual bytes | exact source-evidence and target status |
| REQ-RUST-039.0 | test_registry_owners_are_exact; test_payload_fields_have_provenance; test_payload_citations_are_exact; test_storage_keys_match_schema; test_content_store_not_control; test_typed_input_is_record | offline Python | registered owner/schema fields cite registry, serialized rows or source types; typed inputs are not fake tables | exact owners, payload keys and record kinds |
| REQ-RUST-039.0 | test_candidate_coverage_gaps_explicit; test_ladder_lists_outputs_first | offline Python | source-coverage limits are explicit; five HLD outputs precede full-graph artifacts and upstream contracts | exact ordered report rows and limitation set |
| REQ-RUST-039.0 | test_missing_required_export_blocks; test_mismatched_header_name_errors; test_mismatched_row_generation_errors; test_stage_attempt_mismatch_errors; test_marker_summary_is_unfrozen; test_ladder_verifier_stays_blocked | offline Python | validate required export names, row generation and attempts; banners are not frozen proof; overall comparison remains blocked | exact ERROR/BLOCKED statuses |
| REQ-RUST-039.0 | test_hld_outputs_required_exports; test_summary_scope_mismatch_errors; test_summary_scope_missing_unverified; test_empty_exports_remain_blocked | offline Python | five HLD outputs are required; stage summaries are scoped to export generation/attempt or remain unverified; summary evidence never freezes a stage | exact missing-export, mismatch, unverified and empty-export statuses |
| REQ-RUST-039.0 | test_flagged_contains_remains_blocked | offline Python | retain actual OD-7 containment blockers while unflagged decided containment remains eligible | exact OD-7 blocker classification |
| REQ-RUST-039.0 | test_ladder_contract_dag_order | offline Python | report all registered contracts once in backward owner-layer order after HLD and full-graph inputs | exact contract row coverage and ordering |
| REQ-RUST-040.0 | test_itoa_top_level_containment | pinned-itoa integration | Buffer's direct contains edge is in the canonical deduplicated edge set; file/folder roll-up counts equal exact matching unit/path/relation/via cardinalities, with other top-level items sharing buckets | exact identities and edge-set cardinality |
| REQ-RUST-040.0 | test_source_fixture_scope_rules | real-source fixture | renamed public/private/use/inline/nested-body/cfg-off items, normal child-module provenance, macro-include exclusion, and an empty reached file follow original top-level scope; child-file `pub` reachability remains unknown absent proof | exact request-origin kinds, expansion chains, source identities, cfg and ownership rows |
| REQ-RUST-040.0 | test_source_mutation_identity_changes | real-source mutation | changing a top-level identifier changes occurrence keys and contains-edge endpoints; no label-only match | before/after canonical identity sets |
| REQ-RUST-040.0 | test_source_visibility_scope_equivalence | source fixture | private/self/equivalent same-module restrictions stay out of interface nodes; ancestor restrictions stay distinct | exact full-graph visibility and selected node sets |
| REQ-RUST-040.0 | test_source_module_owner_paths | source fixture | child-file owners and qualified paths derive from namespaces; identical names do not collide | exact owner/path/identity sets |
| REQ-RUST-040.0 | test_source_namespace_input_validation | L3 boundary unit | invalid, conflicting or stale namespace contexts and malformed use-leaf ranges fail closed | valid scope values preserved; zero contexts accepted for malformed inputs |
| REQ-RUST-040.0 | test_file_area_path_normalizes; test_file_area_units_differ; test_file_area_invalid_unit_rejected; test_area_edge_materialization_roundtrip; test_area_evidence_path_rejected; test_area_evidence_unit_rejected; test_structural_imports_edge_rejected; test_structural_local_source_rejected; test_structural_expanded_node_rejected; test_area_signature_rejected; test_local_area_kind_rejected; test_semantic_unknown_reachability_rejected; test_hld_recursive_edge_guards; test_hld_node_visibility_rules; test_hld_area_identity_published; test_hld_symbol_codec_unchanged; test_source_missing_evidence_detected; test_source_file_digest_mismatch; test_source_span_range_rejected; test_source_span_utf8_rejected; test_source_span_text_mismatch; test_source_unknown_syntax_rejected | L1/L3 unit | file-area evidence is exact to unit/path; structural-only shape and reachability contracts are enforced; source projection detects missing and malformed byte evidence and rejects undecided active syntax; DuckDB uses typed materialization and Turso persists typed area identity without changing symbol JSON | materialized area identity/buckets/evidence and exact persisted identity values |
| REQ-RUST-041.0 | test_hld_graph_input_roundtrip; test_hld_unfrozen_source_scope; test_hld_missing_graph_marker; test_hld_valid_empty_graph_input; test_hld_full_semantic_scope_marker; test_hld_conflicting_graph_row_key; test_hld_frozen_scope_mismatch; test_hld_stale_graph_marker; test_hld_unknown_reachability_scope; test_hld_structural_publication_rollback; test_hld_missing_root_unit_scope; test_hld_malformed_sysroot_flag; test_hld_malformed_scope_config; test_hld_source_race_rejected; test_hld_source_marker_mismatch; test_hld_source_marker_missing; test_hld_assembly_cancel_checkpoints | Turso integration | graph input is written and reassembled from one exact frozen revision; structural scope stays explicit; staged publication is atomic across all five HLD tables | exact typed record equality, missing-vs-empty marker distinction, row-key integrity, legacy FullSemantic marker compatibility, canonical source revision closure, cancellation during assembly, and unchanged prior serving rows on failure |
| REQ-RUST-039.0 | test_ladder_manifest_is_immutable; test_builder_pin_cli_supported | offline Python | report-only output does not mutate manifest; explicit pin/source paths permit bundle regeneration | byte-identical manifest and successful CLI execution |

The HLD materialization and publication matrix (test_hld_*) is in section 3.4 under REQ-RUST-031.0.

### Queued verification contracts — not executable evidence

The following test IDs are planned work, not discovered tests or passing gates. Promote each to
the executable matrix only after adding the test and recording nonzero discovery and behavior in
the TDD journal. A packet cannot pass while one of its required tests remains planned.

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| REQ-RUST-042.0 | test_generic_supertrait_method_target | pinned-source integration | one generic format-to-Sealed-write dependency at the exact call site | 1 matching extracted edge; 0 wrong targets or concrete fan-out |
| REQ-RUST-042.0 | test_generic_lookup_rejects_distractor | resolver unit | unrelated same-name method is not selected | 0 distractor targets |
| REQ-RUST-042.0 | test_generic_ambiguity_remains_unresolved | resolver unit | competing applicable candidates remain unresolved | 0 extracted targets; 1 typed reason |
| REQ-RUST-042.0 | test_generic_cycles_terminate_reasoned | resolver unit | cyclic bound traversal terminates and retains unresolved evidence when no unique applicable member is proven | termination; no fabricated target |
| REQ-RUST-042.0 | test_generic_shadowing_uses_binding | source fixture | receiver follows lexical binding at call, not same-name outer generic parameter | exact receiver/target identity |
| REQ-RUST-042.0 | test_generic_rename_changes_identity | source mutation | renamed trait/method updates canonical targets | exact before/after identity sets |
| REQ-RUST-044.0 / 042.0 | test_generic_full_map_roundtrip | frozen-row integration | generic edge and exact evidence survive full-map readback without an HLD projection | exact endpoints, dispatch, site and generation |
| REQ-RUST-044.0 / 032.0 | test_full_map_reference_equality | pinned-source integration | scoped full map and evidence match independently frozen expected sets | 0 unexplained missing/extra/wrong rows |
| REQ-RUST-044.0 / 020.1 | test_full_map_frozen_readback | store integration | full-map readback rejects missing/stale generation markers without depending on HLD tables | exact compatible rows or typed error |

Deferred queued contract (BL-06), not a prerequisite of the active generic full-map packet:

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| REQ-RUST-037.0 / 041.0 / 042.0 | test_generic_hld_evidence_roundtrip | published-row integration | full graph, HLD and source evidence agree through frozen readback | 0 broken evidence links; exact endpoints/site |

#### REQ-RUST-043.0 — original item source containment

WHEN an original non-body item is projected into the full graph, THEN it SHALL have one lexical
source container: its file-public-area-node at file level, or the enclosing inline module or
trait for nested declarations. An impl block is not a graph node; an impl member uses the
impl's enclosing source container. The implementation type and implemented trait SHALL NOT
be additional `contains` parents. The impl occurrence and self-type/trait relationships remain
separate owner metadata backed by `impl-header-index-table`.

WHEN an impl is nested inside a function body, THEN its descendants SHALL remain body evidence,
not full-graph nodes. Same-named methods in different impls retain distinct canonical occurrences.
An unavailable trait/self-type identity remains unresolved with a reason; source containment
SHALL NOT manufacture a semantic target or visibility. This decision does not assign generated
items to original files, add a file-module/area relationship, or permit contains/imports collapse.

Queued tests (not execution evidence): `test_impl_source_container_identity`,
`test_impl_nested_container_identity`, `test_impl_body_members_excluded`.
The semantic-call checkpoint continues to use REQ-RUST-034.0–036.0.

#### REQ-RUST-044.0 — detailed full-map acceptance

WHEN the pinned-itoa Phase 1 output is assessed, THEN acceptance SHALL compare the scoped
full-item-dependency-graph and linked source, signature, body, resolution, expansion and
unresolved/inactive evidence against an independently reviewed frozen reference. No unexplained
extra, missing or wrong row is permitted. Private items and direct private-to-private
dependencies SHALL remain present under REQ-RUST-032.0; incomplete semantic coverage SHALL NOT
pass merely because every missing fact has an unresolved label.

WHEN full-map rows are read back, THEN their identities, exact source evidence and compatible
generation/attempt/revision markers SHALL be verified without requiring interface selection,
private-path collapse, roll-ups or the five HLD output tables. Readback of an incomplete source
scope is explicitly diagnostic, not a full-map completion marker. Sysroot exclusion remains
diagnostic and SHALL NOT qualify complete selected-source semantics.

REQ-RUST-020.1/021.1 generation isolation, failure/cancellation safety and determinism remain
in force. A full-map-only serving transition requires an explicit stage/marker contract before
implementation; this requirement does not relax the existing HLD publisher or claim such a
transition exists. Acceptance records frozen source/configuration/executable identity, exact
comparison results, 1/4/16-worker semantic equality, elapsed time, whole-tree RSS and disk usage.

WHEN a later abstraction is derived, THEN it SHALL be a query over the compatible frozen full
graph and retained evidence, optionally materialized. A derived view SHALL NOT invent semantic
facts, overwrite direct facts or require a separate source-analysis pipeline. Missing semantic
inputs require upstream work; separately authored intent is not an extracted source fact.

### REQ-RUST-045.0 — Incomplete full-map development checkpoints

WHEN original item/member and direct-call producers are being developed, THEN the full-map
checkpoint SHALL reuse canonical graph identities, node/edge records and frozen input storage,
without requiring an HLD projection. `IncompleteTypedInput` SHALL remain explicit through
persistence and assembly. Unknown effective visibility/reachability SHALL NOT be invented;
the existing HLD validator SHALL continue rejecting incomplete inputs and unknown visibility.
This checkpoint is bounded development evidence, never complete selected-source qualification.

The `full-item-dependency-graph` CLI final stage SHALL run the existing source/namespace/impl
pipeline, original-item projection and Rust call resolution, then persist/read back the graph
under one compatible frozen source generation/attempt. Node and edge markers SHALL retain the
exact input snapshot and incomplete-input state. Required source markers, stale input rejection,
cancellation and previous-serving-output preservation remain enforced. Final workspace publication
requires both graph owners complete for the bounded checkpoint; no HLD marker/table is required.
Global completion here denotes the bounded row bundle, not semantic completeness. HLD publication
SHALL NOT accept that bundle as complete. The old top-level diagnostic remains unchanged.

Original non-body items SHALL have canonical source identities, exact persisted source evidence,
one lexical container and separate impl ownership evidence. Impl blocks and body-local descendants
SHALL NOT become graph nodes. Generated items remain in source/expansion evidence and outside this
original-item checkpoint, explicitly not qualified. Source files have separate file-area identities.
Visibility/signature/body facts that are not lowered remain explicit, not guessed.

Rust type-relative lookup SHALL use resolved canonical type identities and impl member occurrences,
not method names alone. A uniquely proven non-generic inherent associated-function target may emit
an extracted call. Missing/ambiguous/generic/unsupported receiver or dispatch cases SHALL remain
unresolved. Lexical shadowing SHALL NOT fall back to an unrelated module declaration. Every call
retains exact site and source-owner evidence. Reference/fixture facts never feed production.

Verification contracts: `test_original_member_graph_roundtrip`,
`test_default_new_call_roundtrip` and `test_child_file_owner_identity`, plus producer/storage negative tests. Discovery and nonzero
execution must precede any pass claim. `test_full_incomplete_checkpoint_roundtrip`,
`test_full_known_visibility_incomplete`, `test_full_sealed_markers_immutable`,
`test_full_stale_source_rejected` and `test_preserve_supplied_frozen_snapshot` cover the
incomplete/frozen-boundary contracts; component evidence does not qualify the combined pipeline.

## 7. Implementation plan

Development order (2026-11-13): Phase 1 is the single pinned itoa library in
[SOP §1.1](SOP-phase-20261113-full-item-dependency-graph.md#11-phase-1--pinned-source-vertical-hld-acceptance)
and [phase-one-benchmark-pin.json](../phase-20261111/phase-one-benchmark-pin.json).
Construct and independently verify its expected full-item-dependency-graph and detailed evidence
from source before production comparison, then repeatedly test each available stage against
that frozen reference. The scoped node/edge, evidence and diagnostic sets require exact agreement;
REQ-RUST-012.1/020.1/032.0/044.0 apply. HLD-specific expectations are deferred, not relaxed.
A future stage is NOT STARTED,
not a passing empty table. This section specifies a benchmark procedure, not
implemented runner support, a completed expected graph, or a passing run.

Active acceptance order: original item/member nodes and REQ-RUST-043.0 source containers toward
REQ-RUST-034.0–036.0 `<Buffer as Default>::default -> Buffer::new`; then REQ-RUST-042.0 generic
supertrait dispatch. REQ-RUST-040.0 remains a bounded diagnostic regression, not the product.
Keep one real-source acceptance target active. Supporting tests repair its observed missing or
incorrect boundary; component totals do not replace rerunning that target. Work backwards from
frozen full-map rows to semantic records and exact bytes. Retain observed failure
evidence and record the earliest missing producer, not merely the last empty output table.

Phase 2 retains the original five-repository corpus, scored serde first, then
rust-analyzer, ripgrep, tokio and cargo-modules. M0–M8 completion and release gates
remain all-five; a Phase 1 benchmark PASS is separately named and does not satisfy
them. Optional model review remains non-authoritative under REQ-RUST-025.0.
The Phase 1 reference must account for the entire selected library and active
macro expansions, but not recursively enumerate external sysroot internals.
No proc-macro-shape result may be inferred from itoa with `no-panic` disabled.
The following M0–M8 plan is retained for later corpus qualification (BF-01/BF-04), not active
Phase 1 scope. M8 requires explicit resumption of the deferred HLD work.

- M0: workspace skeleton, L1 IDs/errors, fake store, dependency gates, pinned oracle harness/build manifest.
  RED: 010, identity fixtures under 012; no final HLD export expected yet.
- M1: graph/ingest/parser, Turso runtime, generation markers, scheduler.
  RED: 001–004, 014–015, 020–022; measure write throughput at 1/4/16 and publish file/span counts.
- M2: combined name-resolution/macro fixed point, built-ins, lang-item discovery.
  RED: 005, 006, 016, lang-item clause of 017. Proc-macro-dependent sites excluded-and-reported only here.
- M3: proc-macro shapes for serde, salsa, thiserror, tokio main/test and corpus derives.
  RED: 013; rerun 005/006 without M2's temporary exclusion.
- M4: signatures and impl index. RED: 018, test_incoherent_impl_headers_indexed.
- M5: body 7a, locals/inherent methods/autoderef. RED: 007 and 017 primitive lookup; report 008.
- M6: body 7b, traits in scope. Gate 007, report 008; all five ≥95% permits skipping M7.
- M7: optional 7c solver adapter; gate 007/010 and report 008.
- M8: HLD/collapse/roll-ups/banner. RED: 009/019/011/012; mandatory final 008 and all contracts green.

Owned milestone fixture gates run when their component exists; 012 identity fixtures start M0, final export
determinism starts M8. §6's future clauses are not required to pass before their milestone.
Use SOP §3 subsets at intermediate milestones, full S1–S10 on all five at M8/releases.
Record measured results as M; a proposed test is not a passed test.

DuckDB landing order for Phase 2 (serde first):

Land bundled DuckDB skeleton and offline/budget/copy contracts; delete obsolete
in-memory/profiling paths, preserving resolver contracts. Then SQL resolution
GREEN, serde wall time/rounds/tree RSS, comparator SQL, all-five S5. Continue
S6/S7 with SOP codebase-memory: list_projects, exact root_path match, schema,
complete paginated coverage and reconciliation. Commit/push STUB, RED, GREEN,
REFACTOR separately. No success is inferred from unmeasured stages.

### Quality gate results

Documentation contracts only. Runtime tests/builds are not claimed executed; no implementation is supplied here.
Clause-level planned coverage is enumerated in §6. Naming/fence/stale-reference checks are documentation gates.

### Resolved decisions and remaining measurements

Q1 resolved: turso =0.9.0-pre.5 async; L3 owns runtime, L2 sync. Stable observed: 0.8.3.
Q2 resolved: copy-adapt mbe; span only with default-features=false; production no-salsa dependency gate.
Q3 resolved: proc-macro-shape-records.toml feeds proc-macro-shape-table; invocation fixtures validate each row.
Q4 resolved: Phase 1 lib+bin, default features, test cfg OFF; test-enabled runs are a Phase 2 option.
Remaining measurements: corpus commit pins at M1, oracle executable digest after build, throughput/conflict
behavior and process-tree RSS. None is fabricated as an observed result.

## 8. Open points

Active full-map correctness choices follow. Do not implement an undecided alternative.

- What relates a file's module node to its area node. Original-item lexical source
  containment is decided by REQ-RUST-043.0; contains/imports collapse remains prohibited.
- Whether `reads` and `uses_type` may have a non-function source: a const
  initializer, an associated type, a struct field type or an array length
  (`const MAX_STR_LEN: usize = <u64 as Integer>::MAX_STR_LEN`).
- Enum variants and struct fields as full-graph nodes; REQ-RUST-030.0 still
  rejects Field/EnumVariant ownership.
- Generated-item identity and actual source/container provenance for `contains` and evidence.
- AD-1 to AD-5 of the PRD: how references into `core` are handled, who resolves paths inside
  function bodies, where effective visibility is computed, how much body inference the first
  version has, and what the output is compared against.

Projection-only decisions and absent-from-itoa `pub use` treatment moved to backlog BD-01–BD-06.
Canonical frozen full-map read/export remains active; consumer-facing denormalization does not.
- Column lists for the two full-graph tables.

## 9. Id history

```text
 REQ-RUST-019.1  D6 nodes, reexports  replaced by 019.2. Its reexport and reachability clauses
                                      are in 019.2; "private uses are evidence only" and "no use
                                      statement becomes a node" are withdrawn. One code test
                                      still cites 019.1 and reads as 019.2.
 011.1  2 GiB process-tree gate       text kept; threshold replaced by 024.5
 024.4  memory threshold wording      threshold replaced by 024.5; the rest is in force
 "Turso is the only engine"           replaced by 005.2
 serde-first development order        replaced by itoa Phase 1 first; serde is first in Phase 2
 stage 8 merged into stages 6/7       edge rows have one owner stage in the registry (section 2)
```

Section numbers of the earlier SPEC, as cited by the SOP and the journals, map to this file as:

```text
 §1 scope and oracle -> 1        §2 requirements -> 3          §2.1-§2.4 rules -> 4
 §3.1 layers and stages -> 2     §3.2-§3.4 interfaces -> 5     §4 crosswalk -> 6
 §5 plan, §6, §7 -> 7            §8 repositories -> 10
```

Ids that share a family number but are separate requirements, all in force: 005.1 and 005.2;
010.2, 010.3 and 010.4; 011.1 and 011.2; 016.1 and 016.2; 021.1 and 021.2; every 024.x, 026.x
and 028.x.

## 10. Repositories we take from

Modes: copy-adapt keeps upstream headers and rewires inputs to tables; link-pinned uses an exact published
version; rewrite-from follows structure without copying; oracle runs offline outside production; idea-only
copies/links no code; tooling is for agents, never part of production. Record full chosen hashes before gates.

### Rust analysis sources

Repository: https://github.com/rust-lang/rust-analyzer
License: MIT OR Apache-2.0.
Pinned parser/oracle source: upstream 2a362a41 + patch bf6b2099d6351c37039b01447a9bb64f9a54a586.
Full upstream base: 2a362a41a8eb6e090f1a12d9b712df85af6f0c33.
The minimal opt_item grammar patch routes statement-position const move/pipe closures
to expression parsing, retaining source tokens. Upstream-style success/error snapshots
are included. Reviewable patch: tools/parser-const-closure-statements.patch.
Reproducible fork object: tools/parser-source-fork-bundle.bundle (base prerequisite).
After cloning upstream, fetch that bundle's HEAD and detach at the full patch hash.
The parser and oracle manifests require this same clean commit.
The earlier nameres/MBe copy-adaptations retain their recorded 6e37b2d62127d445d77069031b416c48c9b0b4f2 origin.
- Copy-adapt crates/hir-def/src/nameres/ collector/path resolution into the crate fixed point.
- Copy-adapt crates/mbe/src/ expander/transcriber without salsa; crates/hir-expand/src/builtin/
  fn_macro.rs, attr_macro.rs and derives; lang_item.rs recognized lang-item names.
- Rewrite-from hir-def/src/expr_store/lower.rs; hir-ty/src/lower.rs, infer/, method_resolution.rs,
  next_solver/interner.rs; project-model/ process/input handling; ide-db/src/prime_caches.rs scheduling.
- Link syntax/parser directly from the exact clean oracle checkout, with ra-ap-rustc_lexer 0.180.
  parser-source-pin-manifest.json and a shared build-time verifier enforce the source pin.
  M: registry 0.0.362 and the earlier checkout both reject Rust 1.99 final associated functions;
  the new checkout's grammar accepts them. Do not rewrite tokens to compensate.
- Build oracle-semantic-export-harness from this exact checkout with its ide/hir crates.
  Record executable digest/build manifest after build; the installed 58cf8f9 binary is not substituted.
- Never link ra_ap_hir*, ra_ap_ide*, ra_ap_base_db, salsa or the proc-macro server into production.
  Offline oracle dependencies are a separate build target, not an exception to production's cargo-tree gate.

Repository: https://github.com/rust-lang/rust
License: MIT OR Apache-2.0.
Solver source/version pin: at M7, compatible with the pinned rust-analyzer solver interface.
- Link-pinned ra-ap-rustc_type_ir / ra-ap-rustc_next_trait_solver only at M7.
- Input: toolchain library/ sources (core/alloc/std), found through rustc --print sysroot, parsed not compiled.
  Record toolchain/source hashes and target cfg; missing matching sources block affected gates.

### Database and support sources

Repository: https://github.com/tursodatabase/turso
License: MIT.
Pinned crate: turso =0.9.0-pre.5 (stable observed 0.8.3).
Mode: link-pinned; only database engine; async L3 runtime and BEGIN CONCURRENT / MVCC.
Source evidence: https://docs.rs/crate/turso/0.9.0-pre.5/source/src/connection.rs
and https://docs.rs/crate/turso/0.9.0-pre.5/source/src/transaction.rs.
Version evidence: https://crates.io/api/v1/crates/turso.
Write throughput/conflict correctness remains a measurement gate, not established by API inspection.

Repository: https://github.com/oli-obk/cargo_metadata
License: MIT.
Mode: link-pinned at M0; typed cargo metadata --offline JSON.
M: registry observations 0.19.2 and 0.23.1; chosen exact version is recorded before use.

Repository: https://github.com/BLAKE3-team/BLAKE3
License: CC0 / Apache-2.0.
Mode: link-pinned at M0; file/content/semantic/generation digest construction.

Repository: https://github.com/proptest-rs/proptest
License: MIT OR Apache-2.0.
Mode: pinned dev-dependency; byte, reason, identity and graph-invariant fixtures.

### Oracle corpus sources

All five are equal test cases; external offline lib+bin qualification once, never production compilation.
- https://github.com/rust-lang/rust-analyzer
  Commit: upstream 2a362a41 + patch bf6b2099d6351c37039b01447a9bb64f9a54a586.
- https://github.com/BurntSushi/ripgrep
  Full commit: 178cc027e283fd965cb72d0a7395b219eada170c.
- https://github.com/tokio-rs/tokio
  Full commit: ab0814fc8c62b3d8be40aad068e2fed6562cfeb0.
- https://github.com/serde-rs/serde
  Full commit: aa7913ad0211fef960df68e7812a6211145a888c.
- https://github.com/regexident/cargo-modules
  Full commit: 48455ca5961ccfc3196b5d6bd5dd3efec97374c0.

M1 source pins and one-time qualification outcomes: corpus-source-pin-manifest.json beside this SPEC.
All five cargo fetch and cargo check --workspace commands passed on 2026-11-11, outside production.
These checks do not establish semantic oracle agreement or a passing M1 mapping run.

Oracle capability clarification (REQ-RUST-027.0): a source pin identifies the checkout used by the
harness; it does not establish which semantic families the current exporter supports or qualify an
S4/S5 run. The current `mapped_semantic_paths` capability is limited to mapped path-resolution
envelopes. It does not claim full S4 family support, PASS/HLD status, or corpus qualification.

### Ideas without copying

- cargo-modules: MPL-2.0; visibility/module rendering for node_reachable_outside_crate.
  Never copy/link its code.
- https://github.com/cargo-public-api/cargo-public-api
  Observed short commit: 68b218915; public API/reachability comparison ideas.
- https://github.com/obi1kenobi/cargo-semver-checks
  Observed short commit: 161450e93; public items/reexport-path ideas.
- https://github.com/praezi/rust
  Observed short commit: ec5aa0ed2; ecosystem dependency/evidence graph ideas.
- https://github.com/willcrichton/flowistry
  Observed short commit: 693ceda92; body-flow ideas, not its compiler driver.

### Agent tooling sources

https://github.com/DeusData/codebase-memory-mcp
License: MIT. Version observed: 0.12.1. Agent exploration and independent structural cross-check only.

https://github.com/that-in-rust/agent-room-of-requirements
Observed short commit: 117afc24b; executable-spec/reliability method ideas.

Copied code keeps its original license header and a third-party-license-notices.md entry in the copying
crate. MPL-2.0 cargo-modules code is never copied. No source pin or runtime outcome is fabricated here.
