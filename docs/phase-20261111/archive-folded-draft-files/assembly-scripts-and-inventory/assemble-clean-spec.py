#!/usr/bin/env python3
"""Assembles the clean SPEC from the layered phase-20261111 SPEC. Verbatim blocks keep their
wording; only revision framing is removed. Hand-written parts are marked HW below."""
import re, json, pathlib
R = pathlib.Path('/home/ubuntu/repos/parseltongue-integration/docs')
OLD = (R/'phase-20261111/SPEC-phase-20261111-executable-requirements.md').read_text().split('\n')
heads = [i for i, l in enumerate(OLD) if l.startswith('#')] + [len(OLD)]
infence = False; real = []
for i, l in enumerate(OLD):
    if l.startswith('```'): infence = not infence
    if l.startswith('#') and not infence: real.append(i)
real.append(len(OLD))
def blk(line, title=None, body_only=False, level='###'):
    i = line - 1; assert i in real, line
    j = real[real.index(i) + 1]
    body = OLD[i + 1:j]
    while body and not body[-1].strip(): body.pop()
    if body_only: return '\n'.join(body).strip('\n') + '\n'
    h = title or re.sub(r'^#+ ', '', OLD[i])
    return f"{level} {h}\n" + '\n'.join(body).rstrip('\n') + '\n'
def rng(a, b): return '\n'.join(OLD[a - 1:b]).strip('\n') + '\n'
def para(prefix, title):
    i = next(k for k, l in enumerate(OLD) if l.startswith(prefix))
    j = i
    while j < len(OLD) and OLD[j].strip(): j += 1
    text = '\n'.join(OLD[i:j]); text = text[len(prefix):].lstrip()
    return f"### {prefix.rstrip(':')} — {title}\n\n{text}\n"

P = []
P.append("""# Executable specs — phase-20261113: full dependency graph, HLD levels derived from it

Status: contracts. This file states only what is in force now; it replaces
`docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md`, which is kept as history.
Requirement ids and test names are unchanged, so code and journal references still resolve;
section 9 maps every superseded id. Product intent: PRD-phase-20261113-dependency-graph-hld.md.
Design: ARCH-phase-20261113-pipeline-graph-tables.md. Procedure: the phase-20261111 SOP.
**M** denotes observations, not targets. New owned identifiers, columns and artifact basenames
use four words. A named test contract is not a passing test; section 6 says which exist in code.

## 1. Scope and oracle
""")
P.append(blk(8, body_only=True))
P.append("""
## 2. Architecture

Two databases and no third. Turso is the concurrent write side and the serving store: durable
rows, generation markers, frozen snapshots, atomic publication. DuckDB is the analysis side:
it works on a copy of one complete published generation and owns every join, fixed point,
recursion, deduplication and roll-up. Rust owns parsing, macro expansion and per-body work.
No LLM, salsa or rust-analyzer semantic crate in production.

The pipeline builds one full-item-dependency-graph first and derives every HLD level from it.
A level adds no fact; it filters nodes, collapses paths and counts.
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
""")
P.append(rng(710, 723).replace('I --> J["DuckDB: HLD chain-collapse recursive CTE + rollups"]',
    'I --> G2["Turso: full graph, nodes + edges"]\n    G2 --> J["DuckDB: HLD levels, chain-collapse recursive CTE + rollups"]'))
P.append("\n### Crate inventory and stage ownership\n\n" + blk(724, body_only=True))
P.append("""
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
""")
for n in (39, 61, 67, 73, 80, 92): P.append("\n#" + blk(n))
P.append("\n### 3.2 Name resolution and macros\n")
for n in (101, 108): P.append("\n#" + blk(n))
P.append("\n#" + para("REQ-RUST-010.3:", "non-monotone resolution rounds"))
P.append("\n#" + para("REQ-RUST-010.4:", "path, prelude and visibility semantics"))
P.append("\n#" + para("REQ-RUST-011.2:", "textual macro scope and expansion alternation"))
for n in (117, 123, 131): P.append("\n#" + blk(n))
P.append("\n#" + blk(857, "S5 semantic path partition — classification rule"))
P.append("\n### 3.3 Signatures and bodies\n")
for n in (140, 145, 152): P.append("\n#" + blk(n))
P.append("\n### 3.4 Full graph and HLD levels\n")
P.append("\n#" + blk(1780))
P.append("\n#" + blk(1821, "REQ-RUST-019.2 — interface level is a filter").replace(
    "Reexport edges, node_reachable_outside_crate and the collapse algorithm are\nunchanged from REQ-RUST-019.1, REQ-RUST-009.2 and §2.4.",
    "Restricted or public uses create one reexports edge per resolved exported binding (aliases\nand globs included), sourced at the nearest enclosing selected module, or the crate root; the\nlexical reexporting module is retained in evidence even when it is private. Store\nnode_reachable_outside_crate from complete module and reexport reachability, including\nprivate parents.").replace(
    "This changes REQ-RUST-009.2 as built: HLD_RECURSIVE_PATH_SQL walks every\nrelation kind today.", "Not in code yet: HLD_RECURSIVE_PATH_SQL walks every relation kind today."))
P.append("\n#" + blk(173) + "A collapsed path never contains a `contains` or `imports` hop (REQ-RUST-019.2).\n")
P.append("\n#" + blk(1544).replace("REQ 019.1 selected nodes", "REQ 019.2 selected nodes").replace(
    "Resolved relations use only calls/uses_type/implements/reexports/contains/derives.\nGeneric/concrete dispatch is evidence metadata, not an additional relation kind.\n(Revised 2026-11-13 by REQ-RUST-030.1: imports and reads are added.)",
    "Relation kinds are those of REQ-RUST-030.1. Generic/concrete dispatch is evidence metadata,\nnot an additional relation kind."))
P.append("\n##" + blk(1582, "Input validation"))
P.append("\n##" + blk(1603, "Path and collapse SQL"))
P.append("\n##" + blk(1625, "File and folder roll-ups"))
P.append("\n#" + blk(1847))
P.append("\n#" + blk(1528).replace("REQ 019.1", "REQ 019.2"))
P.append("\n#" + blk(1653))
P.append("\n##" + blk(1674))
P.append("\n#" + blk(1124, level='###'))
P.append("\n### 3.5 Engines, storage and generations\n")
P.append("\n#" + para("REQ-RUST-005.2:", "write side and analysis side"))
P.append("\n#" + para("REQ-RUST-016.2:", "DuckDB owns relational analysis"))
for n in (180, 188, 197, 205): P.append("\n#" + blk(n))
for p_, t in (("REQ-RUST-024.1:", "published-generation copy"), ("REQ-RUST-024.2:", "generation keys and product copy-back"),
              ("REQ-RUST-024.3:", "total ordering and thread determinism"), ("REQ-RUST-024.4:", "engine isolation settings")):
    P.append("\n#" + para(p_, t))
for n in (928, 1034, 1072): P.append("\n#" + blk(n, level='###'))
P.append("\n### 3.6 Memory and determinism\n")
P.append("\n#" + blk(212) + "The threshold in force is REQ-RUST-024.5's; the 2 GiB figure above is the superseded gate.\n")
P.append("\n#### REQ-RUST-024.5 — final memory target\n\n" + blk(847, body_only=True))
P.append("\n#" + blk(974, level='###')); P.append("\n#" + blk(220))
P.append("\n### 3.7 Comparator and oracle\n")
P.append("\n#" + para("REQ-RUST-021.2:", "comparison in DuckDB and path partitions"))
for n in (1229, 1240, 1250, 1262, 1270, 1281): P.append("\n#" + blk(n, level='###'))
P.append("\n#" + blk(1288)); P.append("\n#" + blk(1379, level='###'))
P.append("\n### 3.8 Diagnostics and test harness\n")
for n in (878, 899, 1414): P.append("\n#" + blk(n, level='###'))
for n in (1428, 1440): P.append("\n##" + blk(n))
P.append("\n#" + blk(1448, level='###')); P.append("\n##" + blk(1466))
P.append("\n#" + blk(1479, level='###')); P.append("\n##" + blk(1497))
P.append("\n## 4. Rules\n")
for n in (226, 259, 275): P.append("\n" + blk(n, re.sub(r'^### \d\.\d ', '', OLD[n - 1])))
P.append("\n" + blk(292, "Visibility and collapse rules").replace("Reexports use REQ 019's", "Reexports use REQ 019.2's").replace(
    "Emit direct resolved relations first. For each selected source, traverse resolved evidence through private\nitems,",
    "These rules apply to the interface level; the full graph stores every node and edge and is never\ncollapsed (REQ-RUST-032.0). Emit direct resolved relations first. For each selected source, traverse\nresolved evidence through private items, never along a `contains` or `imports` edge,"))
P.append("\n## 5. Persisted interfaces\n")
P.append("\n" + blk(331, "Identity, columns and generations")); P.append("\n" + blk(409, "Trait seams and errors")); P.append("\n" + blk(434, "Turso and cancellation"))
P.append("\n## 6. Verification matrix\n\n")
P.append("TESTSTATUS\n")
P.append("\n### Requirement → test → SOP crosswalk\n\n" + blk(443, body_only=True))
P.append("\n### Full graph and level contracts (none written yet)\n\n" + blk(1878, body_only=True))
P.append("\nThe HLD materialization and publication matrix (test_hld_*) is in section 3.4 under REQ-RUST-031.0.\n")
P.append("\n## 7. Implementation plan\n\n" + blk(539, body_only=True))
P.append("\n" + blk(835, body_only=True).replace("The 2026-11-13 Phase 1 order above supersedes the serde-first development order\nin the following historical plan; serde remains first for Phase 2 qualification.\n", "DuckDB landing order for Phase 2 (serde first):\n"))
P.append("\n### Quality gate results\n\n" + blk(579, body_only=True))
P.append("\n### Resolved decisions and remaining measurements\n\n" + blk(584, body_only=True))
P.append("\n## 8. Open points\n\nDo not implement a choice for these; each needs a decision first.\n\n" + blk(1855, body_only=True).split('\n', 2)[2])
P.append("""- Whether collapse is limited to chains of `calls` and `reads`. Today the last hop's kind
  labels the edge, so a public function inherits the type uses of the private helpers it
  calls (64 of 134 collapsed edges in the itoa draft dataset).
- AD-1 to AD-5 of the PRD: how references into `core` are handled, who resolves paths inside
  function bodies, where effective visibility is computed, how much body inference the first
  version has, and what the output is compared against.
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

Ids that share a family number but are separate requirements, all in force: 005.1 and 005.2;
010.2, 010.3 and 010.4; 011.1 and 011.2; 016.1 and 016.2; 021.1 and 021.2; every 024.x, 026.x
and 028.x.

## 10. Repositories we take from
""")
P.append("\n" + blk(593, body_only=True))
for n in (599, 633, 657, 680, 693): P.append("\n" + blk(n))
out = ''.join(P)
out = re.sub(r'\n{3,}', '\n\n', out)
for x, y in (("- Deleted by this revision `parseltongue-port-name-resolver`", "- Deleted `parseltongue-port-name-resolver`"),
             ("This revision specifies a benchmark procedure", "This section specifies a benchmark procedure")):
    assert out.count(x) == 1, x
    out = out.replace(x, y)
T = json.load(open(R/'phase-20261113/.rewrite-scratch/tests.json'))
new_tests = sorted(set(re.findall(r'\btest_[a-z0-9]+(?:_[a-z0-9]+)+', out)))
present = [t for t in new_tests if t in T['tests_code']]
out = out.replace("TESTSTATUS\n", f"Of the {len(new_tests)} test names in this file, {len(present)} exist in the code today and "
    f"{len(new_tests) - len(present)} are planned. The code has {len(T['tests_code'])} tests in total; those not named here\nare supporting tests.\n")
(R/'phase-20261113/SPEC-phase-20261113-executable-requirements.md').write_text(out.rstrip('\n') + '\n')
old = '\n'.join(OLD); ids = lambda s: set(re.findall(r'REQ-RUST-\d{3}\.\d+', s))
print('lines', out.count('\n'), 'old', len(OLD))
print('missing ids', sorted(ids(old) - ids(out)))
print('missing tests', sorted(set(T['tests_spec']) - set(new_tests)))
print('fences', out.count('\n```'), 'revision words', len(re.findall(r'[Rr]evision|[Hh]istorical|supersede', out)))
kebab = lambda s: set(re.findall(r'`([a-z0-9]+(?:-[a-z0-9]+){2,})`', s))
print('missing backticked names', sorted(kebab(old) - kebab(out))[:40])
