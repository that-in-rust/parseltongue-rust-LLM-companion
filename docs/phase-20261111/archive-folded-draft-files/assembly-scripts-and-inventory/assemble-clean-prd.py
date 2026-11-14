#!/usr/bin/env python3
"""Assembles the clean PRD from the phase-20261111 PRD and the phase-20261113 itoa PRD."""
import re, pathlib
R = pathlib.Path('/home/ubuntu/repos/parseltongue-integration/docs')
def load(p):
    L = (R/p).read_text().split('\n'); f = False; real = []
    for i, l in enumerate(L):
        if l.startswith('```'): f = not f
        if l.startswith('#') and not f: real.append(i)
    return L, real + [len(L)]
A, AH = load('phase-20261111/PRD-phase-20261111-compiling-hld.md')
B, BH = load('phase-20261113/archive-folded-draft-files/PRD-phase-20261113-itoa-hld-current.md')
def body(src, line):
    L, H = (A, AH) if src == 'A' else (B, BH)
    i = line - 1; assert i in H, (src, line, L[i])
    return '\n'.join(L[i + 1:H[H.index(i) + 1]]).strip('\n') + '\n'
def sub(t, a, b):
    assert t.count(a) == 1, a[:60]
    return t.replace(a, b)
cols = re.search(r'```text\nrun_generation_digest_value.*?```', body('A', 48), re.S).group(0)
arch = body('A', 91)
arch = sub(arch, "Turso is the only database engine (superseded by §16: Turso for writes and serving, DuckDB for analysis).\nPin turso",
           "Turso is the write and serving engine. Pin turso")
arch = sub(arch, "Stage 8 is merged into stages 6/7, which write evidence directly; no hidden resolved-site buffer crosses stages.",
           "No hidden resolved-site buffer crosses stages.")
arch = sub(arch, "- L2 stage crates: synchronous logic over row-store traits, plus copy-adapted pure macro algorithms.\n- L3 adapters: Turso runtime, parser/cargo bridges, optional next-solver adapter.",
           "- L2: write and analysis ports, scheduler, copy-adapted pure macro algorithms.\n- L3 adapters: Turso runtime, DuckDB analysis, parser/cargo bridges, optional next-solver adapter.")
dec14 = body('A', 263)
dec14 = sub(dec14, "six total write attempts, 2 GiB process-tree RSS.", "six total write attempts.")
ad = '\n'.join(B[183 - 1:273 - 1]).strip('\n') + '\n'
ad = sub(ad, "            The parent PRD §5 lists no separate visibility stage; its §17 now shows the\n            registry's stage names without deciding where the work is computed.",
         "            Section 5 shows the registry's stage names without deciding where the work\n            is computed.")
ad = sub(ad, " recommend  A, and add one line to the parent PRD. It is the smaller change.", " recommend  A. It is the smaller change.")
ad = sub(ad, " option A   keep sysroot ingestion (parent PRD as written)", " option A   keep sysroot ingestion")
ad = sub(ad, "            - conflicts with \"no invented sysroot identities\"; needs a parent PRD revision", "            - conflicts with \"no invented sysroot identities\"; needs a change to section 3")
ad = sub(ad, "            + reuses the existing path-candidate SQL; joins stay in DuckDB (parent §16)", "            + reuses the existing path-candidate SQL; joins stay in DuckDB (section 4)")
ad = sub(ad, "            - more rows in the analysis copy; needs a SPEC requirement revision", "            - more rows in the analysis copy; needs a SPEC requirement change")
ad = sub(ad, "            + matches the parent PRD's stage 7 wording", "            + matches section 5's stage 7 wording")
ad = sub(ad, "             inherent methods (parent 7a); trait methods through a direct bound (parent 7b)", "             inherent methods (7a); trait methods through a direct bound (7b)")
ad = sub(ad, " note       under DD-8 this table", " note       under rule 8 of section 3 this table")
od = re.search(r'```text\n( OD-2.*?)```', body('B', 153), re.S).group(1)
accept = body('B', 69)
accept = sub(accept, "The five HLD tables are derived from those two by the\nlevel rules and are compared as well. The preceding pipeline tables\n(`itoa-hld-preceding-tables.md`) are internals, not targets.",
             "The five HLD tables are derived from those two by the\nlevel rules and are compared as well. The pipeline's other tables are internals, not targets.\nA draft of the target, written by an LLM from the source and not frozen, is in\n`itoa-target-dataset/` (73 nodes, 368 edges).")
accept = sub(accept, "Passing this phase is not a pass of the parent PRD's M8 or of the five-repository SOP.", "Passing Phase 1 is not a pass of M8 or of the five-repository SOP.")
status = body('B', 108)
build = body('B', 273)
pin = re.search(r'```text\n repository.*?```', body('B', 23), re.S).group(0)
narrow = sub(body('B', 60), " one compilation unit plus whatever `core` handling decision AD-1 selects", " one compilation unit plus whatever `core` handling AD-1 selects")
narrow = sub(narrow, " third-party dependencies, async, dyn dispatch, the optional trait solver (parent 7c)", " third-party dependencies, async, dyn dispatch, the optional trait solver (7c)")
m = body('A', 230)

out = f"""# PRD — phase-20261113: full dependency graph and HLD for Rust code that compiles

Status: draft. Owner: human-architect-intent-author.
This file states what is in force now. It replaces `docs/phase-20261111/PRD-phase-20261111-compiling-hld.md`
and the earlier itoa-scoped PRD, both kept as history.
Executable contracts: SPEC-phase-20261113-executable-requirements.md.
Design: ARCH-phase-20261113-pipeline-graph-tables.md.
Procedure: `docs/phase-20261111/SOP-phase-20261111-hld-current-e2e-test.md`.
**M** marks observed facts; unmeasured budgets are targets, not results.

## 1. Objective

Build, for compiling Rust workspaces, one full-item-dependency-graph and the HLD levels derived
from it, using CPU tools only: no LLM and no training in production.
The full graph holds every item outside function bodies, private ones included, and every
dependency proven from source. hld-current-dependency-edges and the roll-ups are views of it.
An ext-llm-coding-agent uses the interface level to plan across modules and the full graph to
see what a private helper touches before editing it.

Port rust-analyzer algorithms over persisted rows rather than host its analysis database in production.
Aim for exact extracted dependencies, report unknowns without guessing, and enforce SPEC's explicit
precision/coverage gates rather than claiming that compilation guarantees a complete graph.

D1: cargo metadata --offline. D2: five repositories qualify equally: rust-analyzer, ripgrep, tokio, serde,
cargo-modules; development is driven first by one small pinned library, itoa (section 10).
D3: proc-macro-shape-table checked against real oracle expansions. D4/KL1: never execute build scripts
or read their saved target/ outputs. D5: rustc only --print cfg / --print sysroot during production.
D6: section 3.

## 2. Source closure, targets and context

{body('A', 27)}
## 3. Product: the full graph and the levels derived from it

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
```

Rules (D6):

```text
 1  The full graph is built first and is a product. A level adds no fact; it only filters
    nodes, collapses paths and counts.
 2  Full-graph nodes: every module, function, struct, enum, trait, type alias, const, static
    and associated item owned by the crate root, a module, a trait or an impl, private
    items included. Nothing inside a fn body. Impl blocks are not nodes.
 3  Each source file has one file-public-area-node, separate from its module node, kept at
    the interface level even when the module is private. It contains the file's top-level
    nodes.
 4  Each use outside fn bodies is a node, with one imports edge per resolved binding.
 5  An inline module is an ordinary module node with its own scope.
 6  Every extracted edge is stored, private -> private included. Relation kinds: calls,
    uses_type, implements, reexports, contains, derives, imports, reads (a read of a const,
    static or associated const).
 7  A call, type use or read targets the real item, never the use line; the use line is
    evidence. A function has edges only to what it uses, not to every import in scope.
 8  The full graph is the two existing tables above; no table is added.
 9  The interface level keeps the crate root, non-private items, file-public-area-nodes and
    use nodes; private items are dropped there and remain as steps in evidence paths.
 10 Collapse walks only dependency edges: calls, uses_type, implements, reexports, derives,
    reads. A contains or imports edge is never a step of a collapsed path; it shows at the
    interface level only as a direct edge to a kept or external target.
```

Visibility is effective visibility, not the `pub` token alone. Trait members, trait-impl members and
enum variants inherit their interface visibility; inherent associated items respect their own
restrictions; private-equivalent pub(self) does not qualify. External reachability is a separate
fact that follows ancestor modules and public reexport aliases. Public or restricted reexports
produce one edge per resolved binding, including aliases and globs, sourced at the nearest
selected enclosing module with the private lexical module kept in evidence. SPEC section 4 defines
the collapse algorithm: cycles terminate, diamonds retain provenance without duplicate HLD rows,
genuine self-edges are retained, and an unresolved intermediate link never creates an extracted
successor. File/folder roll-ups count deduplicated HLD rows, not the number of private paths.

Edges are extraction only. A generic or dynamic call targets the trait method once, never a
guessed implementation. Unknowns stay in evidence and site rows with one reason each.
hld-current-dependency-edges is written solely by cpu-tool-hld-table-builder, never by an agent
or a person. Its owned columns, defined with full semantics in SPEC section 5:

{cols}

Via values: direct/collapsed. Origin values: same_crate/workspace_crate/std/external_crate.
Every node carries node_reachable_outside_crate; external targets preserve package source and target
identity, not just crate/version. Full span identity and snapshot are recoverable through the
generation/occurrence keys.

## 4. Architecture and storage

Two databases and no third. Turso holds durable rows, generation markers, frozen snapshots and
atomic publication, with BEGIN CONCURRENT/MVCC for concurrent stage writes. DuckDB is the
analysis engine: it receives a copy of one complete published generation through the Rust
Appender, with per-table count and digest equality checked before any analysis, and owns name
resolution rounds, signature and impl joins, method candidates, HLD collapse and roll-ups.
Name resolution uses complete, non-monotone stratified SQL rounds, not append-only accumulation.
Rust macro expansion alternates with SQL and writes its evidence to Turso. Only final product
tables copy back, with digest verification. DuckDB runs bundled, with no extension installed or
loaded, no external access, and its temp directory inside the run directory.

{arch}
## 5. Stages and ownership

Every stage reads declared tables and writes declared tables, with generation markers on every
write. The Turso registry gives each table one owner stage. Stage 0 bootstraps from the explicit
filesystem/process boundary; ingest alone reads source bytes; resolvers never read source files
behind the table interface.

```text
 stage                               engine          writes                                    state
 0  crate graph                      Rust            crate-dependency-graph-table,             built
                                                     workspace-config-state-records
 1  source ingest                    Rust            file-source-text-table                    built
 2/3 syntax walker                   Rust            span-reference-text-table, span-byte-     built
                                                     ownership-ranges, field-signature-detail-
                                                     table, body-site-reference-table,
                                                     namespace input tables, lang-item-
                                                     definition-table, macro scope records
 4+5 name resolution + item macros   DuckDB + Rust   module-scope-binding-table, name-         built, not
                                     expander        resolution-scope-records, path-resolution- qualified
                                                     site-records, macro-expanded-item-table
 6  signature-type-lowering-stage    Rust + DuckDB   lowered-signature-type-table,             not built
                                                     impl-header-index-table
 7  semantic-edge-evidence-stage     Rust + DuckDB   edge-evidence-record-table: every edge    not built
                                                     of the full graph, all eight kinds
 -  item-effective-visibility-stage  open (AD-3)     item-effective-visibility-records: every  not built
                                                     node of the full graph
 9  hld-current-materialization-     DuckDB          the five HLD tables; reads only the two   built on
    stage                                            full-graph tables and unit provenance     fixtures
 10 cpu-tool-status-coverage-banner  Rust            per-level-coverage-banner                 partial
 11 cpu-tool-oracle-edge-differ      DuckDB, offline oracle-edge-agreement-table               built for
                                                                                               paths
```

Stage 7 has three sub-steps: 7a locals, inherent methods and autoderef; 7b traits in scope;
7c the optional next-solver adapter. Stage 0 verifies lock presence before metadata and immutable
bytes after it, and loads proc-macro-shape-records.toml into proc-macro-shape-table. Closure-discovery
requests return to persisted ingest/walker work; no stage reads unknown files directly. Stage 11
never spawns analysis engines from inside a production mapping run.

## 6. Porting sources

{body('A', 164)}
## 7. Oracle and evidence

{body('A', 182)}
## 8. Process and unresolved-reason rules

{body('A', 195)}
## 9. Requirements, verification and scoring

SPEC section 3 is the single executable wording. Do not maintain an abbreviated conflicting
WHEN/THEN copy here. The requirements cover input/process integrity (001–004/014–015), resolution
and macros (005–008/010.3–010.4/011.2/013/016–018), the full graph and HLD levels (009/019/029–032),
engines and generations (005.2/016.2/020–022/024), memory (011/024.5/024.9), deterministic export
(012), the comparator and oracle (021.2/025–027) and diagnostics (024.6–024.7/028).

{body('A', 218)}
## 10. Phases and milestones

### Phase 1 — one small pinned library

Development is driven by a real-source test on one library that a person can read in full.
Phase 1 produces its full graph and derived HLD tables from real source through the production
pipeline and compares them row by row against an independently reviewed target.

{pin}

Narrower than the whole product:

{narrow}
Acceptance:

{accept}
Where the pipeline stands (read from the code, 2026-11-13):

{status}
Build order:

{build}
### Phase 2 — five repositories, SPEC M0–M8 is authoritative

Order: serde, rust-analyzer, ripgrep, tokio, cargo-modules.

{m}
## 11. Budgets and measurements

Final target: peak resident memory of the whole production process tree at most 500,000,000 bytes,
enforced only after working end-to-end results exist for all five repositories. Until then memory is
measured and reported, with no interim cap, and correctness comes first.
Target: body inference under 30 minutes on 4 cores for rust-analyzer plus selected dependencies.
Measure summed resident bytes at 10 ms intervals with child accounting and record telemetry limitations;
exclude harness/indexer processes. Disk size and stage wall time are reported, not invented thresholds.
The full graph is larger than the HLD on every repository; its size is measured and reported.
No budget or correctness result is claimed achieved by this document.

## 12. Out of scope

{body('A', 255)}
Not in Phase 1: the five Phase 2 repositories, the native sysroot timeout except as far as AD-1
option A requires it, CLI query verbs, the coverage banner, and memory or speed optimisation.

## 13. Decisions

### Decided

{dec14}
Decided on 2026-11-13: rules 1 to 10 of section 3.

### Open, graph and level rules

```text
{od} OD-9  Is collapse limited to chains of calls and reads? Today the last step's kind labels
       the edge, so a public function inherits the type uses of the private helpers it calls
       (64 of the 134 collapsed edges in the itoa draft dataset).
```

OD-7 and OD-8 change rows of the itoa target dataset, so they block freezing it.

### Open, pipeline choices

Each one states what the code does today, the options, and what each option costs.
A recommendation is given where the evidence supports one. None is decided.

{ad}
## 14. Provenance and licensing

SPEC section 10 lists repositories, exact known commits, modes (copy-adapt/link-pinned/rewrite-from/
oracle/idea-only) and licenses. Copied code retains headers and third-party-license-notices.md in its
crate. cargo-modules MPL-2.0 code is idea-only, never copied.

## 15. What this document does not claim

```text
 no stage after name resolution exists for real source
 no run on itoa has been made
 the itoa target dataset is an LLM-parsed draft, not frozen
 rules 1 to 10 of section 3 are written down, not implemented: the code has no file_public_area
   item kind, no imports or reads relation kind, and its collapse walks every edge kind
 the recommendations in section 13 are design judgement, not measurements
```
"""
out = re.sub(r'\n{3,}', '\n\n', out)
for a, b in ((" parent stage             state ", " stage                    state "), ("so under DD-9 the lib.rs", "so under rule 10 the lib.rs"), ("SPEC §2.3", "SPEC section 4"), ("SPEC §2.2 and SOP", "SPEC section 4 and the SOP"), ("SPEC §8", "SPEC section 10"), ("SPEC §4's", "SPEC section 6's"), ("SOP §3", "SOP §3")):
    out = out.replace(a, b)
(R/'phase-20261113/PRD-phase-20261113-dependency-graph-hld.md').write_text(out)
print('lines', out.count('\n'), 'fences', out.count('\n```'))
print([l for l in out.split('\n') if re.search(r'parent|§1[67]|[Rr]evis|supersed|DD-\d|decision AD', l)][:20])
