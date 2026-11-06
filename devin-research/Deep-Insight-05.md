# Deep Insight 05 — Proposal 01, restructured top-down (Minto pyramid)

*2026-11-06. A re-ordering of Deep-Insight-04 (Proposal 01, §0–§10) by the Minto pyramid principle: answer first, then three mutually exclusive and collectively exhaustive key lines, each resting on grouped evidence. No new claims; every leaf cites the DI-04 section it comes from. DI-04 stays as written. Later revisions are appended as new sections here or as a new Part of the collected Deep-Insight-01.*

---

## Introduction (S-C-Q-A)

**Situation.** Coding agents are now doing cross-cutting changes to large Rust codebases. Their only structural instrument is grep; rust-analyzer knows the exact structure but answers one cursor at a time, and its answers degrade exactly when a refactor has broken the build. (DI-04 §1, §2)

**Complication.** Nothing holds the *intended* structure of the code, nothing diffs intended against actual, and no tool admits what it could not resolve. So agents re-derive the architecture from grep on every task, make plans no one can check, and return "no dependents" when they mean "could not see". (DI-04 §1, §2, §6 personas a/d/h)

**Question.** What is the one thing to build, and why would anyone adopt it over rust-analyzer, Serena, Aider's repo map or Cursor's index?

**Answer — the governing thought:**

> **Build the structure of a Rust codebase as one open, provenance-carrying map in three tenses — as-is, intended, as-built — that an agent reads to navigate, writes to plan, and the CLI diffs to verify; publish the compiler-exact corpus and per-crate feed first, because that is the part nobody has and the part that outlives any tool or model.**

```text
                              ┌──────────────────────────────────────────────────────┐
                              │  ONE OPEN MAP, THREE TENSES, PROVENANCE ON EVERY EDGE │
                              │  read to navigate · write to plan · diff to verify    │
                              │  corpus + feed first, tool second, model last         │
                              └──────────────────────────────────────────────────────┘
                 ┌────────────────────────────┼──────────────────────────────┐
   ┌─────────────┴─────────────┐  ┌────────────┴────────────┐   ┌─────────────┴──────────────┐
   │ 1  WHAT IT IS              │  │ 2  WHY IT IS CREDIBLE    │   │ 3  WHAT TO DO FIRST         │
   │ a shared, checkable object │  │ Rust makes it exact;     │   │ three cheap signals;        │
   │ with three tenses, two     │  │ the broken moment is the │   │ ship find · impact · verify;│
   │ visible levels, and a      │  │ real one; openness       │   │ publish corpus + feed       │
   │ storage rule that keeps    │  │ answers the hard         │   │ before any model claim;     │
   │ facts and opinions apart   │  │ personas; evidence exists│   │ and what not to do          │
   └───────────────────────────┘  └─────────────────────────┘   └────────────────────────────┘
```

---

## Key line 1 — WHAT IT IS: a shared, checkable object, not a tool

### 1.1 One map, three tenses, same ids
- AS-IS: what the source establishes today (extracted). INTENDED: a graph patch the LLM drafts and a human approves. AS-BUILT: re-extracted after the edit. The loop `propose → build → re-extract → compare → report` is the product; one-way "LLM reads a graph" is Sourcegraph-for-agents, one-way "LLM writes a plan" is an ADR generator. (DI-04 §1)
- Analogy that holds: blueprint, as-built drawing, building inspector. Software keeps only the building. (DI-04 §1, §9)
- "Bidirectional" is earned only by the proposal loop; LLM-asks/tool-answers is every MCP server. (DI-04 §7, GLM §2)

### 1.2 Two visible levels; deeper levels on request
- Level 1 file ↔ file (`mod`, `use`, `include!`, cfg gates) — tree-sitter, always available. Level 2 the public face of each file: signatures, re-exports, `pub static/const`, `#[cfg]`, doc comment, item-level macro names. Level 3+ (bodies, calls, flow) kept, hidden, opened on expansion. (DI-04 §3)
- Comments, imports and globals are **fields of the item/module record**, not new node kinds — imports are evidence on `USE` edges, statics are items, doc-comment nodes invite identity churn. The founder's requirement is met by the record. (DI-04 §7 ADOPT, §9)
- Membership test for level 2: *could this line change what another file is allowed to write?* (DI-04 §3)

### 1.3 The storage rule that makes it trustworthy
- Every edge carries `exact | projected | proposed | unresolved`, plus creator, extractor version, cfg context. An unresolved result is never shown as absence. (DI-04 §2, Part 2 §5)
- Three layers never written into each other — compiler facts ≠ shared vocabulary ≠ opinions (LLM meaning, proposals, sightings) — taken from Sikos's cybersecurity knowledge graphs and MISP: provenance and confidence per object, sightings, warninglists, first/last-seen, feeds with trust levels (H-47…H-59). This is what lets an opinion about the future (the intended map) sit next to compiler facts without poisoning them. (DI-04 §9)

### 1.4 Under full openness the object has four parts, and the tool is the smallest
- Apache-2.0/MIT CLI + extractor; versioned record schema; **public feed of compiler-exact graphs per crate@version** (docs.rs, but the graph); **open training corpus** with repo-disjoint, post-cutoff held-outs; open weights with card + eval + recipe; a leaderboard. (DI-04 §10.1, §10.5)
- The feed shrinks "Reach": the model no longer memorises std and 100 crates — it downloads the facts and learns only how to connect them. (DI-04 §10.1)
- The model is one leaderboard row, not the product. (DI-04 §10.2)

---

## Key line 2 — WHY IT IS CREDIBLE: Rust, the broken moment, openness, and evidence

### 2.1 Rust is the one mainstream language where the map is a fact
- Static names, `pub` in source, one impl per trait+type, no reflection → rust-analyzer/rustdoc produce an exact public-interface graph; any approximation can be scored against truth. (thesis §13; DI-04 §1)
- Measured: tree-sitter 0.07 s and rust-analyzer 29 s / 2.3 GB on 42k LOC, ~0 unknown expressions with proc macros on; warm `cargo check` 0.69 s. (Part 1–2; DI-04 §2)

### 2.2 "Works for broken code" is the hinge because the refactor is when the code is broken
- Halfway through extracting a trait, 40 files fail to type-check and RA's types go `{unknown}` across the blast radius; the intended map is the only stable thing in the room. (DI-04 §2)
- Level 1 and the public skeleton need no type-checker, so they stay up; only resolution degrades, and the banner says by how much. (DI-04 §2, §9)
- Narrowed after GLM: syntactic indexing on broken files is not the claim (Aider, Cursor, Serena do that); the claim is the intended map plus provenance degradation on the *same diffable map*. The model is not part of the hinge; evidence against it (0 vs 56 bindings; 10/19 edges) stands. (DI-04 §7 HOLD/ADOPT)

### 2.3 The absolute claim is false; the combination claim is true and has one killer piece
- Each piece exists somewhere: cargo-modules (file graph), cargo-public-api / cargo-semver-checks (public surface), Serena (LSP over MCP), Aider repo map (ranked graph in a token budget), RA non-Cargo projects. (GLM §2; DI-04 §7)
- Nobody ships exact Rust edges + budgeted agent queries + current/proposed distinction + per-edge provenance. The before/intended/after conformance report exists nowhere. Under openness, add: nobody publishes the compiler-exact Rust graph corpus or a per-crate graph feed. (DI-04 §7, §10.8)

### 2.4 Openness answers the hardest personas; engineering answers the rest
- Dissolved by openness: RA contributor (licence-compatible, `codemap` export offered upstream), CTO (single static binary, zero egress, auditable), ML researcher (citable repo-disjoint data), HN ("weekend project" → "nice benchmark"). (DI-04 §10.3)
- Not touched by openness: the 300k-LOC maintainer (proc macros, re-extract latency), the harness builder (their own logs), the newcomer (`where do I even start`). Only a clean run, sub-second incremental re-extract and a three-arm A/B convert them. (DI-04 §4, §6, §10.3)
- New personas created by openness: crate authors (licence passthrough, correction channel) and contributors (documented schema and plugin points before v1). (DI-04 §10.3)

### 2.5 The honest negatives are already in the record
- Frontier models omitted 6 of 14 signature dependencies; source-only agents found 10/19 local edges; the learned-binding experiment accepted 0 bindings where a rule accepted 56; 65 % of calls are method calls whose receivers need other items' return types. (Parts 1–3; DI-04 §6 persona g)
- Therefore: no model in the demo, no model in v1, H-46 stays a ladder with a stop rule. (DI-04 §5, §7)

---

## Key line 3 — WHAT TO DO FIRST: three signals, three tools, corpus before model

### 3.1 Run the three cheapest signals before writing more documents
```text
 1. three-arm pilot: baseline agent · +map (no model) · +map+model, 3 unseen repos, equal token budget
       arm 1 ≈ arm 2 on correctness → navigation is a feature, not a product         ~1 week, $0 training
 2. intent authoring: on real disputed work, do two maintainers produce a SECOND graph patch unprompted?
       no → Timeline B collapses to navigator + reviewer                              one design cycle
 3. H-46 ladder rung 2 (10 M tokens; ~2 h Mac or 1.5 min H100): held-out curve vs the 56-binding rule
       flat or below → keep rules, stop training                                      one afternoon
```
(DI-04 §8; GLM §5)

### 3.2 Ship three tools, not a protocol
- `find` (the seven-event retrieval is its internals), `impact`, `verify` (achieved / missing / unexpected / unresolved). Advisory report only; no merge gating until seeded-mismatch recall is measured. (DI-04 §7; GLM §6)
- The 90-second demo: banner with provenance at 0:00; `find` → `propose` → agent edits → build breaks → `verify` still answers with the unresolved count → fix → `verify` green → `report` with a reproducibility line (`pt verify --at <sha> --extractor v0.3`). (DI-04 §5, §10.6; GLM §4)
- Technical prerequisites, none of which need the model: L1+L2 extraction on non-compiling files; stable item identity across rename/move (Part 2 §5, unsolved); a graph-patch format the LLM emits and the CLI diffs; incremental re-extract fast enough to run on every save. (DI-04 §5)

### 3.3 Publish the corpus and the feed before any model claim
- They are the cheapest verifiable "nobody has this"; they are the paper the ML persona asked for; they neutralise GLM death mode #2 (the model is optional once the benchmark is the deliverable) and reclassify "RA ships agent mode" as mission success. (DI-04 §10.2, §10.8)
- Requirements: permissive-only sources with per-record repo/commit/licence/extractor-version; signed feeds with MISP-style distribution levels and warninglists; RA version pinned per release with a conformance suite in CI; `last_seen` so staleness is shown, never hidden. (DI-04 §10.4, §10.5)
- CHECK before publishing: Z.ai terms on redistributing GLM-written L6 labels; keep that layer separately licensed until verified. (DI-04 §10.4)

### 3.4 What not to do
- Do not lead with the model or the 100 MB gate (a leaderboard column, not a product requirement). (DI-04 §7, §10.8)
- Do not make comments/imports/globals graph nodes. (DI-04 §7)
- Do not claim "neither rust-analyzer nor any MCP can do this"; say "nobody ships the combination". (DI-04 §7)
- Do not delete history to look tidy: one canonical page (thesis §13 / this document) plus the collected DI-01 as changelog. (DI-04 §7 HOLD)
- Do not treat anything in GLM's or Devin's reception tables as market evidence; they are attributed opinions in the opinion layer. (DI-04 §6 preamble, §8)

---

## Residual risk that no key line removes
**Nobody authors the intended map.** UML, ArchUnit tests and design docs were all rejected as ceremony; openness does not change that. The only mitigation is that the agent drafts the patch from the PR text and the human merely approves; signal 3.1-2 is the test. If it fails, the product is a navigator plus reviewer — a utility, not a company — and the corpus/feed/benchmark are still worth having. (DI-04 §7, §10.8; GLM §5)

---

## Appendix A — where each DI-04 section landed in the pyramid

```text
 DI-04 §0 founder's statement        → Introduction (Situation / Question)
 DI-04 §1 one big thing              → Governing thought; 1.1
 DI-04 §2 broken code is the hinge   → 2.2
 DI-04 §3 two levels                 → 1.2
 DI-04 §4 Devin's reception          → 2.4 (grouped by argument, not by persona)
 DI-04 §5 90-second demo             → 3.2
 DI-04 §6 GLM-5.3 verbatim           → 2.3, 2.4, 2.5, 3.1 (as attributed opinion); full text stays in DI-04
 DI-04 §7 agree / disagree table     → 1.2, 2.2, 2.3, 3.4
 DI-04 §8 three cheapest signals     → 3.1
 DI-04 §9 ELI5 + Sikos               → 1.1, 1.3
 DI-04 §10 full openness             → 1.4, 2.4, 3.3, Residual risk
```

## Appendix B — the pyramid as a reading order for different readers

```text
 30 seconds   Introduction → governing thought
 3 minutes    + the three key-line headings and 3.1
 15 minutes   + every numbered leaf
 the evidence  DI-04 (verbatim GLM review, full tables) → collected DI-01 Parts 1–5 → thesis §13 → ledger H-1…H-59
```
