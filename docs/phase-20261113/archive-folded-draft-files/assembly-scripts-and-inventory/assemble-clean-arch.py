#!/usr/bin/env python3
"""Assembles the clean architecture file from the folded phase-20261113 drafts."""
import re, pathlib
R = pathlib.Path('/home/ubuntu/repos/parseltongue-integration/docs/phase-20261113')
D = R/'archive-folded-draft-files'
def src(name): return D/name if (D/name).exists() else R/name
def load(name):
    L = src(name).read_text().split('\n'); f = False; H = []
    for i, l in enumerate(L):
        if l.startswith('```'): f = not f
        if l.startswith('#') and not f: H.append(i)
    return L, H + [len(L)]
def body(name, title):
    L, H = load(name)
    i = next(k for k in H[:-1] if L[k].lstrip('# ').startswith(title))
    return '\n'.join(L[i + 1:H[H.index(i) + 1]]).strip('\n') + '\n'
def sub(t, a, b):
    assert t.count(a) == 1, a[:60]
    return t.replace(a, b)
T, A, P, Z = 'Table-Specs-01.md', 'itoa-hld-build-architecture.md', 'itoa-hld-preceding-tables.md', 'itoa-architecture-haze-ratings.md'
target = body(A, '3. Target architecture')
target = sub(target, "The Turso + DuckDB split of PRD-phase-20261111 §16 is kept as it is. Rule for who does what:", "Rule for who does what:")
target = sub(target, "Stage numbers below are the PRD-phase-20261111 §5 numbers. Bottom (spans) to top (HLD).", "Stage numbers are the PRD section 5 numbers. Bottom (spans) to top (HLD).")
target = sub(target, "        The PRD lists no separate stage for this; the Turso registry and the publisher\n        already name one, so this design follows the code.", "        The Turso registry and the publisher already name this stage; where its work is\n        computed is PRD decision AD-3.")
target = sub(target, "        body paths  today no stage resolves them (F1). Two ways to do it: decision AD-2 in the PRD.", "        body paths  today no stage resolves them (C1). Two ways to do it: PRD decision AD-2.")
target = target.replace("PRD 6 ", "stage 6").replace("PRD 7 ", "stage 7").replace("PRD 9 ", "stage 9").replace("PRD 0-1", "stage 0-1").replace("PRD 2/3", "stage 2/3").replace("PRD 4+5", "stage 4+5").replace("Why PRD 7 is split", "Why stage 7 is split")
facts = body(A, '2. Four facts').replace(" F1 ", " C1 ").replace(" F2 ", " C2 ").replace(" F3 ", " C3 ").replace(" F4 ", " C4 ")
needs = body(A, '4. What itoa actually needs').replace("PRD 4+5", "stage 4+5").replace("PRD 6 ", "stage 6").replace("PRD 7 ", "stage 7").replace("PRD 9 ", "stage 9")
needs = sub(needs, "From the draft source inventory (48 selected occurrences, 197 call-site candidates; draft, not golden):", "From the draft target dataset (`itoa-target-dataset/`: 73 nodes, 368 edges; LLM-parsed, not frozen):")
shape = body(T, '1. Shape')
shape = shape[shape.index("```text\n Full graph tables"):]
tags = """```text
 [code]      in the builder code today
 [decided]   decided by the user on 2026-11-13; in the SPEC, not in the code
 [open]      not decided; listed in PRD section 13
```
"""
def rules(title):
    t = body(T, title).replace("[spec]   ", "[code]   ")
    return t
nodes = sub(rules('2. Full graph: node rules'), "                Reverses the SPEC line \"No use statement becomes a node\" (REQ 019.1).\n", "")
levels = rules('5. Levels')
ex = body(T, '7. itoa under these rules')
chain = body(P, 'The chain, top')
errs = body(Z, '1. What the walk-through found')
errs = sub(errs, "     Corrected in the three documents.\n", "")
errs = sub(errs, "     So some itoa-to-itoa edges depend on core. Corrected in the PRD (AD-1 option B).", "     So some itoa-to-itoa edges depend on core (PRD AD-1 option B).")
errs = sub(errs, """ E3  No relation kind fits a reference to an associated const.
     The relation kinds are calls, uses_type, implements, reexports, contains, derives.
     `Buffer::new` reads `i128::MAX_STR_LEN`, and the 12 MAX_STR_LEN impls are HLD nodes.
     The architecture does not say which edge, if any, that produces.""", """ E3  A reference to an associated const is a `reads` edge.
     `Buffer::new` reads `i128::MAX_STR_LEN`. Open: may the source be a const initializer or
     a type, not only a function (OD-8).""")
errs = sub(errs, " E1  Wrong example in my own documents.\n     itoa's", " E1  Slice 1 is a type-relative lookup, not a `Self` rule.\n     itoa's")
errs = sub(errs, "     The architecture only described `recv.name(..)` on `self` and on a generic receiver.\n", "")
errs = sub(errs, "     The architecture file does not say who renders or emits them.", "     Section 3 proposes the edge stage for `contains`; path and signature text are open (T2).")
out = f"""# Architecture — phase-20261113: pipeline, full graph and HLD tables

Date: 2026-11-13. States what is in force now. It folds together the earlier table rules,
build architecture, preceding-table chain and haze review; those drafts are kept in
`archive-folded-draft-files/`.
Product intent: PRD-phase-20261113-dependency-graph-hld.md.
Contracts: SPEC-phase-20261113-executable-requirements.md.
First checked on pinned itoa 1.0.18 (`af77385d0daf4d0e949e81f2588be2e44f69f086`, 2 files, 488 lines).
Nothing in this file has been run on itoa.

Every statement about the system carries one mark:

```text
 [code]      read in the current source on 2026-11-13 (file named beside it)
 [measured]  a number from an earlier run in this repository
 [proposal]  not built; a design choice in this document
```

## 1. What the pipeline does today

{body(A, '1. What the pipeline does today')}
## 2. Four facts in the code that decide the design

{facts}
## 3. Target pipeline

{target}
## 4. Tables, top (HLD) to bottom (spans)

Levels 1 and 2 together are the full dependency graph, a product in its own right. Level 0 is
derived from it. Levels 3 to 7 are pipeline internals.

{body(P, 'How to read the status words')}
{chain}
Not on the semantic chain:

{body(P, 'Not on the semantic chain')}
The gap:

{body(P, 'The gap in one line')}
## 5. Graph rules

Rule tags:

{tags}
{shape}
### Nodes

{nodes}
### Edges

{rules('3. Full graph: edge rules')}
### Identity

{rules('4. Identity rules')}
### Levels: each is a filter over the full graph

{levels}
### Columns

{body(T, '6. Columns')}
## 6. itoa

### What each stage must handle

{needs}
### Expected under the rules (not produced)

{ex}
### What each relationship needs

{body(P, 'What each itoa relationship needs')}
### What a walk through the source found

{errs}
### Build order — one relationship at a time, on real source

{body(A, '5. Build order')}
## 7. How unclear each task still is

Scale: 1 = clear, could start now. 3 = approach known, details open.
5 = not understood well enough to start; needs a decision or an experiment.

```text
 haze  task                                               what is still unclear
 ----  -------------------------------------------------  ------------------------------------------------
  1    run today's pipeline on itoa, unchanged            nothing; it is a measurement
  1    fix the two failing publication fixtures           nothing; both causes are known
  2    HLD builder (collapse, roll-ups)                   three known changes: two node kinds admitted,
                                                          two relation kinds, collapse skipping
                                                          contains and imports
  2    real-source test harness for slice 1               where the pinned source lives in the test
  2    impl-header rows for hand-written impls            row columns to be fixed in the SPEC
  2    body-position macros (E7)                          only: record them unresolved, add no edges
  2    reads edges (E3)                                   the relation exists; non-function source open
  2    use nodes, imports and contains edges              cheap joins; containment detail open (OD-7)
  3    body paths, first segment (AD-2)                   local shadowing; generic-parameter heads
  3    lowered signature types                            how deep the type tree goes for itoa
  3    uses_type and implements edge rules                which signature positions emit an edge
  3    method call on a generic receiver (i.write)        walking the supertrait Integer: Sealed
  3    effective visibility                               join with impl headers (E8); AD-3
  3    assembler into builder records                     path text and signature text (E9, T2)
  4    type-relative lookup (E2)                          five shapes, primitives, bounds; no code today
  4    calls with a concrete receiver (E5)                argument and cast typing; impl selection
  4    macro-generated identity and evidence (E6)         mapping generated spans back to lib.rs
  4    freezing the target dataset                        waits on OD-7, OD-8, OD-9, E5 and E6
  5    core handling (AD-1, E4)                           unmeasured; and how exact edges into core must be
```

## 8. Reference data

Production rows come only from deterministic Rust and SQL. The itoa target dataset in
`itoa-target-dataset/` was written by an LLM reading the pinned source. It is comparison data,
it is not frozen, and the production pipeline must never read it.

## 9. What this file does not claim

{body(A, '7. What this design does not claim')}"""
out = re.sub(r'\n{3,}', '\n\n', out)
for a, b in (("section 2 ", "section 5 "),):
    pass
(R/'ARCH-phase-20261113-pipeline-graph-tables.md').write_text(out.rstrip('\n') + '\n')
print('lines', out.count('\n'), 'fences', out.count('\n```'))
print([l[:110] for l in out.split('\n') if re.search(r'Revised|revis|PRD-phase-20261111|Table-Specs|preceding-tables\.md|haze-ratings|parent|section \d\b.*(above|below)|SPEC and code are unchanged|See section 8', l)])
