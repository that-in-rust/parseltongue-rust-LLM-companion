# Deep Insight 04 — Proposal 01: a Rust codebase's structure as a shared, checkable object between developer, LLM and compiler

*Everything in this document is the **first proposal** (Proposal 01) for the product, not a settled design. Later proposals get their own numbers and are appended, never substituted.*

*2026-11-06. Written after reading Deep-Insight-01 Parts 1–5 (the original differentiation, the CODEMAP rewrite, four futures, training shortcuts, ELI5 walkthrough) against the founder's own statement of the product. §1–§5 are Devin's synthesis; §6 is GLM-5.3's independent read of the same material (verbatim, via the zcode SOP); §7 says where the two disagree. Append-only: later revisions go in as new sections or as a new Part of the collected Deep-Insight-01.*

## 0. The founder's statement (verbatim)

> a Rust LLM companion that is a CLI tool as well as an LLM, bidirectionally talking to each other. It helps you map out your large Rust codebase, but also helps you plan the structure of your Rust codebase. That dependency graph will be current and future, and it will also work for broken code. It will have at least two levels: file-to-file dependencies; and public-interface dependencies, or at least everything that is public interface, like comments, imports, and global statements. These are the things that neither rust-analyzer nor any other code MCP can do right now.

## 1. The one big thing

**Make the structure of a Rust codebase — what connects to what — a first-class object that a human, an LLM and the compiler can all read, propose changes to, and check against each other, at every moment including the moments when the code does not compile.**

Everything in Parts 1–5 is a piece of that one object or a way of getting it:

```text
                        ┌───────────────────────────────────────────────┐
                        │   THE MAP   (file graph + public-interface graph) │
                        │   three time-states, same vocabulary:              │
                        │     AS-IS      what the code establishes today     │
                        │     INTENDED   what we agreed to build             │
                        │     AS-BUILT   what the changed code establishes   │
                        └───────────────────────────────────────────────┘
        reads / writes ▲                 ▲ builds / judges                ▲ reads / proposes
                       │                 │                                │
                 DEVELOPER         CLI + COMPILER/RESOLVER             LLM AGENT
             approves intent,     extracts, stores, diffs, checks,    navigates, explains,
             reviews the report   keeps provenance honest             drafts the intended map

   Part 1  →  REACH: the map still exists when the resolver cannot run (student model / tree-sitter skeleton)
   Part 2  →  the map itself: records, evidence envelope, as-is vs intended vs as-built, threat model
   Part 3  →  which JOB sells the map first (navigate / design / review / component)
   Part 4  →  how cheaply the fallback extractor can be trained
   Part 5  →  the same thing in plain words
   grep-replacement note / seven-event protocol  →  how the LLM READS the map in 3,000-token pieces
   Sikos/MISP note  →  how facts, vocabulary and opinions stay separate INSIDE the map
```

The analogy that fits: **blueprint, as-built drawing, and building inspector.** Every physical engineering discipline keeps a drawing of what was intended, a drawing of what was built, and someone whose job is to diff them. Software has the code and nothing else; agents inherit that poverty and re-derive the architecture from grep on every task. rust-analyzer is a superb instrument for the as-built drawing at the level of a single cursor. Nothing holds the intended drawing, and nothing diffs the two.

Why "bidirectional" is the right word and not decoration:

```text
   CLI → LLM     bounded neighbourhood · blast radius · path · gaps/unresolved · the before/intended/after report
   LLM → CLI     a question (7 words) · a proposal (graph patch) · an annotation (L6 meaning) · a sighting (confirm/deny an edge)
   CLI → compiler   extract, re-extract, validate a proposed edge
   compiler → CLI   exact facts, or "cannot resolve" — which the map records rather than hides
```

The loop is the product. A one-way "LLM reads a graph" tool is Sourcegraph-for-agents; a one-way "LLM writes a plan" tool is an ADR generator. The thing nobody has is the round trip: *propose → build → re-extract → compare → report*, with the map staying usable while the code is mid-change and broken.

## 2. Why "works for broken code" is the hinge, not a footnote

The code is most often broken **during exactly the work the tool is for**: a cross-cutting refactor. Halfway through extracting `OrderStore` from `PostgresStore`, 40 files fail to type-check, rust-analyzer's type information degrades to `{unknown}` across the blast radius, and the agent is flying blind at the moment it most needs the map. The intended graph is the only stable thing in the room. Two things make the map survive that moment:

1. **Level 1 and the public skeleton need no type-checker.** `mod`, `use`, `extern crate`, `pub fn` signatures, `pub struct` fields, `static`/`const`, doc comments, `#[cfg]` — tree-sitter reads these from a file that does not compile (measured: 0.07 s for 42k LOC). So the file graph and the interface *inventory* are always available; only resolution degrades.
2. **Provenance tells the agent what degraded.** Each edge is `exact | projected | proposed | unresolved`; mid-refactor the share of `unresolved` rises and the report says so, instead of silently returning fewer edges (Part 2 §5 "false completeness").

This reframes Part 1: the student model is not "faster than rust-analyzer", it is **the thing that keeps L3/L4 edges flowing while the compiler is unhappy** — same job as the tree-sitter skeleton, one rung higher.

## 3. The two levels, made precise

```text
   LEVEL 1  file ↔ file             edge kinds:  mod-declares · use-imports · include!/path attrs · build-script/cfg gates
                                    source:      tree-sitter; always available
   LEVEL 2  public surface of each file ("everything that is public interface"):
              doc comment of the file / module          → L6 anchor the LLM can read first
              use / extern crate / pub use (re-exports)  → EXPOSES_AS / REEXPORTS
              pub items with full signature             → SIGNATURE_USES / PARAMETER / RETURNS / IMPLEMENTS
              pub static / const / lazy globals         → the "global statements"
              #[cfg] / feature gates on all of the above
              macro invocations at item level (name + span; expansion when the resolver is available)
            edges between surfaces = the public-interface dependency graph
   LEVEL 3+ private bodies, calls, flow — kept, shown on expansion (the earlier L3–L6 ladder); not part of the default view
```

The test for "is it in level 2": *could this line change what another file is allowed to write?* Doc comments pass (they are the contract's prose), `use` lines pass, globals pass, a private helper body fails.

## 4. Simulated reception (Devin's run; GLM-5.3's is in §6)

```text
 persona                        first reaction                                   what converts them
 ─────────────────────────────  ───────────────────────────────────────────────  ─────────────────────────────────────────────
 maintainer, 300k-LOC Rust DB   "another architecture doc nobody updates"        intended map is drafted FROM the PR text by the
                                                                                  agent; the report catches one real boundary leak
 rust-analyzer contributor      "RA has 90 % of this; be an RA consumer"          we ARE an RA consumer; we contribute a `codemap`
                                                                                  export upstream rather than reimplement
 agent-harness builder          "show me fewer tool calls, same pass rate"        RQ1 A/B: tool calls, tokens, correctness on
                                                                                  real tasks (Part 3's three-arm pilot)
 HN top comment                 "LLM code graphs always go stale; macros?"        provenance tags + the measured macro/proc-macro
                                                                                  numbers; incremental re-extract in <1 s
 newcomer in Cursor             "I just want to understand this repo"             the map alone (Timeline A); they never touch intent
 CTO, 40-person team            "CI noise, false confidence, one more bot"        advisory-only report, seeded-mismatch recall,
                                                                                  no merge gating until measured
 ML researcher                  "tiny model beats RA? no."                        we never claim that; the ablation arm isolates
                                                                                  the model's contribution; honest negatives published
 security engineer              "unsafe surface and blast radius, please"         `unsafe_surface`, `impact`, RustSec seeds (H-49)
                                                                                  — a first-class query, labelled heuristic
```

The pattern: every persona's objection is some form of *"will it be fresh, honest, and worth the ceremony?"* — which is why Part 2's threat model and the MISP fact/vocabulary/opinion split are not academic: they are the answer to the objection.

## 5. The 90-second demo that shows the one big thing

```text
  0:00  pt map .                      → "11 crates, 1,204 files, 9,318 public items; 97 % resolved; 3 % unresolved (proc-macro)"
  0:10  pt ask "where do retries get decided"
                                      → 4 candidates, picks RetryPolicy::should_retry; neighbourhood, 2,400 tokens, blast radius 31
  0:25  pt propose "make retry policy pluggable behind a trait"
                                      → the LLM drafts a graph patch: +trait RetryPolicy, Client→trait instead of →FixedBackoff,
                                        FixedBackoff implements RetryPolicy; 6 consumers listed; maintainer approves
  0:40  agent edits; halfway the crate does not compile (14 errors)
  0:50  pt check                      → still answers: "intended 3 / achieved 2 / missing 1 (FixedBackoff lacks impl) /
                                        unexpected 1 (Client gained a dep on tokio::time) · 9 edges unresolved while broken"
  1:10  agent fixes; cargo check green; pt check → "3/3 achieved, 0 unexpected"
  1:25  pt report → the PR description: before/intended/after, with evidence links.
```

What must be true for this to work: L1+L2 extraction on non-compiling files; stable item identity across a rename/move (Part 2 §5 "identity churn"); a proposal format the LLM can emit and the CLI can diff; incremental re-extract fast enough to run on every save. None of these needs the student model. The model enters only at 0:50, to keep L3 edges alive while the compiler is down.

## 6. GLM-5.3's independent read (verbatim)

*Run 2026-11-06 via `~/bin/zcode` (Claude Code → Z.ai Coding Plan, model glm-5.3, read-only tools), prompt at `/home/ubuntu/zcode-runs/prompt-reception-di04.txt` (reproduced in `devin-research/sop/SOP-zai-glm-coding-plan-second-opinion.md` style). GLM read the collected Deep-Insight-01 Parts 1–5, the grep-replacement note and the resolver-builds thesis; it did not see §1–§5 above. 2,326 words, unedited except heading levels. Everything in this section is GLM-5.3's opinion, stored as an attributed annotation, not as a fact of the research.*

### 1. The one big thing

**Parseltongue is trying to make the dependency graph itself the working interface between a coding agent and a codebase — a versioned map that is read to navigate, patched to plan, and diffed to verify, with every edge carrying its provenance.**

The graph is not the output; it is the shared medium. Everything else is one of four verbs performed on that medium. Part 1 supplies the positioning discipline — Shape (agent-shaped, budgeted queries over exact facts) over Reach (broken-code fallback) over Honesty (per-edge provenance), and the warning that leading with the fallback positions you as "an LSP for broken days." Part 2 supplies the actual data contract — the CODEMAP record set (§2.1), typed edges (§2.3), and the one genuinely novel mechanism in the whole corpus: current map / proposed graph patch / after map as three distinguishable states, compared (§3, §3.2). Part 3 supplies go-to-market: four jobs (navigate, plan, review, embed), with the recommendation "A plus one small B/C loop." Part 4 is an internal lab note on cheapening the training experiment. Part 5 restates 3 and 2 in plain language. The unifying idea is strong precisely because navigation, planning, and review are the same map in three tenses — present, future, past. The corpus's own drift (grep-replacement → CODEMAP → four futures) is the search for this sentence.

### 2. Is the founder's framing right?

| Claim | Verdict | Evidence |
|---|---|---|
| Bidirectional CLI↔LLM | **Partly** | As settled in `1793882149` ("resolver builds, LLM steers"), the loop is LLM-asks / tool-answers — which is just every MCP tool (Serena, CodexGraph). The genuinely bidirectional part is the *proposal* loop (Part 2 §3), where the LLM writes into the map. Also, "an LLM" is ambiguous: the in-tool tiny model's only shipped evidence is 0 accepted bindings vs 56 by a rule (`1793879701` §3). |
| Map + plan | **Partly** | Mapping: cargo-modules already draws file/module dependency graphs; Sourcegraph SCIP gives precise refs and call hierarchies at scale; rust-analyzer holds all of it internally. Planning as before/intended/after graph patches: precedent is reflexion models (1995) and Moderne/OpenRewrite's recipe diffs (Java, recipe-driven, not LLM-proposed). Nobody ships this for Rust. Novel, demand unproven. |
| Current + future graph | **Partly true, unvalidated** | The "future" graph is a record format (Part 2 §3.1), not a demonstrated capability. ArchUnit (Java) asserts intended architecture but doesn't author it with an LLM. This is the most defensible novelty and the least evidenced. |
| Works for broken code | **Partly, and oversold** | rust-analyzer supports detached files and non-Cargo projects (`rust-project.json`, Bazel) — Part 3's own "External grounding" concedes this. Every tree-sitter indexer (Aider repo map, Cursor, Serena's fallback) already "works" on uncompilable code syntactically. The only new version — a model drawing resolved edges without a resolver — currently has negative evidence (0-vs-56 bindings; semver experiment agents found 10/19 edges, Part 2 §6.2). And cargo-semver-checks needs the crate to *build*, so the fallback niche is real but smaller than claimed. |
| Two levels incl. comments/imports/globals | **Partly, with a category error** | Each level exists separately: file graph (cargo-modules, Aider repo map) and public API surface (cargo-public-api, cargo-semver-checks). The *linking* of the two with typed edges and visibility scope (Part 2 §2.2–2.3) is the defensible increment. But "comments, imports, global statements" as public interface is muddled: imports are derivable evidence, statics are just items, and doc comments are rustdoc's province. Making comments graph nodes buys the identity-churn failure mode Part 2 §5 itself warns about. |
| "Neither rust-analyzer nor any other code MCP can do this" | **False as stated** | Part 3's own external-grounding paragraph names cargo-semver-checks and RA's non-Cargo support. Serena MCP does LSP-backed symbol navigation for agents; Aider's repo map is literally a dependency graph ranked into an LLM context budget; GitNexus ships a dependency-graph MCP; CodeQL/Semgrep/Moderne do typed graphs and mass refactor verification. What nobody has is the *combination*: exact Rust edges + budgeted agent queries + current/proposed distinction + per-edge provenance. Combination claims need one killer piece; the before/intended/after conformance report is it. |

### 3. Simulated reception

**(a) Maintainer, 300k-LOC Rust database:** "I know my architecture; that's not my bottleneck. My pain is a 40-crate workspace with sqlx proc macros and a 25-minute CI. Your extractor will choke on the macros, then go stale the moment someone edits, and *I'm* the one who debugs it. Also cold start is a solved problem — RA loads once and stays up for weeks." **Convert:** run clean on their repo with proc macros on, show incremental re-extract under 5 seconds after an edit, and show one `impact()` answer that is right where grep is provably wrong. Nothing else converts this person.

**(b) rust-analyzer contributor:** "This is call-hierarchy plus find-references behind an MCP wrapper, taking credit for our index. If budgeted neighborhoods and path queries are good ideas, they belong in RA or an LSX extension, not a parallel store that will drift from our semantics the first proc macro lands. And you're reshipping our data — what's the attribution story?" **Convert:** contribute the query shapes upstream, or demonstrate queries RA structurally can't express in the one-cursor LSP model: cross-snapshot graph diff, token-budgeted ego networks, proposal conformance. Those genuinely don't fit LSP.

**(c) Agent-harness builder:** "I ship twelve tools; my usage logs show agents call the graph one rarely and misuse it when they do. Every new server is latency, auth, another failure mode in my traces. Serena already gives me symbols over LSP. Why is yours the one that changes task outcomes?" **Convert:** a drop-in MCP server, sub-100ms queries, no build step, and an eval on *their* harness where agents with your tool beat agents with grep at equal token budget. Adoption is measured in my logs, not your benchmark.

**(d) Hacker News top comment:** "So: an MCP server wrapping rust-analyzer, plus a 500MB model that guesses edges when the compiler can't. The first half is a weekend project; the second half is wrong answers with confidence scores attached. 'Works on broken code' means 'sometimes lies,' and the whole thing dies the moment RA ships an agent mode." **Convert:** a blind, reproducible demo on a repo that doesn't build, with the degraded-provenance banner visible and honest, and the before/intended/after report catching a regression that diff review missed. Skeptics are converted by an artifact they can run, not a thesis.

**(e) Rust newcomer using Cursor:** "I just @-mention files and ask; Cursor indexes everything already and never made me learn what a dependency graph is. I tried your tool on my half-written project and it said 40% of edges were 'syntactic' — I don't know what to do with that, and I don't care. I want to know where to start, not what an ego network is." **Convert:** one command — `where do I even start` — that prints a ten-item guided tour of an unfamiliar crate with what to read first, working on their broken WIP code with zero config. If the first run isn't magical, this user is gone in 90 seconds.

**(f) CTO, 40-person Rust team:** "Another component in the toolchain means another review, another vendor, another thing that breaks during an audit. My seniors don't want agents designing architecture, my juniors shouldn't trust a graph with coverage gaps, and you want an MCP server reading my proprietary source — where does it phone home? Show me the failure mode where this *reduces* risk." **Convert:** on-prem, single static binary, zero egress, and one pilot where the conformance report catches an agent-introduced architectural regression in CI that a human reviewer missed. Priced against rework avoided, not seats.

**(g) ML researcher:** "A sub-500MB model doing held-out exact-match on interface graphs at 78–95%? That's a distillation ceiling claim with no repo-disjoint splits, pretraining contamination unaddressed, item windows that leak resolution context — Part 2 §6.1 admits this — and your own learned-binding experiment accepted *zero* bindings where a rule got 56. The scaling-ladder idea is sound; the rest is a hypothesis wearing a product's clothes." **Convert:** publish the benchmark — repo-disjoint, int4-scored, abstention-rate reported — and show one rung where the model beats the rule baseline at a fixed wrong-edge budget on fresh crates. That's a paper and a product component at once.

**(h) Security engineer:** "My fear is false assurance. 'No dependents found' from a graph that skipped two crates under a non-default cfg is worse than no tool — someone will delete code based on it. And to resolve proc macros you run build scripts, which is arbitrary code execution on untrusted repos inside a tool agents drive autonomously." **Convert:** the coverage envelope printed on every answer (scoped exclusions, failed extractions, cfg assumptions), a cfg-aware `unsafe_surface` query validated against a real audit, and a documented no-build mode with explicitly degraded provenance. Show me the failure states or I assume they're hidden.

### 4. The demo that would make people believe

**"One map, three tenses" — 90 seconds, screen-recorded, on a real 60k-LOC multi-crate workspace the tool has never seen.**

- **0:00–0:15** Fresh clone. `parsel index` finishes; provenance banner: 61k items, 99.4% edges `resolver`, 0.6% `syntactic` (two crates, build-script failure), cfg = default. *The honesty layer, on screen, immediately.*
- **0:15–0:35** An agent (real harness, visible tool calls) gets: "support a second WAL page size without duplicating the recovery loop." It runs `find "wal recovery"` → four 80-token headlines, picks one; `impact(WalRecovery::replay)` → callers, impls, two call sites under `cfg(feature = "v2")`, each with file:line. Every id real.
- **0:35–0:55** *Before editing*: the agent emits a graph patch — introduce `PageSize` trait, list implementors, dependency flips. Split-screen shows current map → intended map. Maintainer approves with one keystroke. *This is the bidirectionality claim, made concrete.*
- **0:55–1:15** Agent implements. `parsel verify`: three-way report — 7 achieved, 1 missing (a constructor still takes `u16`), 1 unexpected (a new edge into `checkpoint.rs`, outside proposal scope) — each clickable. *The thing no other tool can say.*
- **1:15–1:30** `git checkout broken-branch` (doesn't compile). Same `impact` query; banner degrades to `substrate: tree-sitter+model, 82% edges confidence < 0.9` — and still points at the right files. *Broken code, honestly labeled.*

**What must be technically true:** cold index of 60k LOC in seconds; incremental re-extract in seconds after edits; stable identity across the rename the edit causes (Part 2 §5 identity problem — unsolved in the docs); correct resolution under cfg and proc macros; a minimal proposal schema a real harness emits unaided; and the model fallback actually at ~80%+ on unseen code — currently *unproven* (Part 2 §6.2). Items one, two, and six are the risk; the demo dies first on re-extract latency and identity churn.

### 5. Where it dies

**1. Navigation alone isn't differentiated.** Frontier models with grep/read keep improving; the map saves tokens but doesn't change task outcomes, and Serena/Aider/Cursor own the "structured context" reflex. **Earliest cheap signal:** Part 3's own three-arm pilot (baseline agent vs +CODEMAP vs +model) on three unseen repos — if the baseline arm matches the tool arm on task correctness, the navigator is a feature, not a product. One week, zero training.

**2. The tiny model never earns its seat.** Evidence is already 0-vs-56 against a rule. If the model stays dead weight, the "rust-analyzer in the weights" story collapses and what's left is an RA-backed MCP server that upstream or Serena can commoditize in a release. **Earliest cheap signal:** scaling-ladder rung 2 (10M tokens, ~2 hours of Mac GPU, Part 4 §4): if the held-out curve is flat, or below the rule baseline at a fixed wrong-edge budget, stop training and keep rules.

**3. Nobody authors the future map.** Timeline B requires maintainers to write structural intent *before* code — a ceremony most teams have rejected in every prior form (design docs, ArchUnit tests, UML). If they won't, the product shrinks to navigator/reviewer and the "current + future" thesis is dead; you have a utility, not a company. **Earliest cheap signal:** Part 3's own inflection point — do two real maintainers, unprompted, create a *second* graph-patch proposal on actual disputed work? That's observable within one design cycle.

### 6. What to cut from the five Parts

- **Part 4 entirely** from the product corpus. FLOP arithmetic and ELI5 lever diagrams are lab-notebook material; publishing them inside the thesis makes the model look more load-bearing than its evidence supports.
- **Part 1 §3's GPU tables** (estimated tok/s and RAM). Superseded and self-caveated by Part 2 §6.2; every number in them is a liability in front of personas (b) and (g). Keep §8's Shape/Reach/Honesty ordering; delete the tables.
- **The 100 MB memory gate** (Part 3 constraints). A research constraint masquerading as a product requirement, and the main force dragging the tiny model into the story. Keep as an engineering note; cut as a gate.
- **Timeline D** (SDK/component). Part 3 itself calls it highest-variance; defer until an external integrator asks twice.
- **Comments/imports/globals as interface nodes.** Imports are evidence fields, not entities; statics are ordinary items; doc-comment nodes invite the identity-churn failure Part 2 §5 documents. Defer all three to a v2 that has users.
- **The seven-event protocol as product spec.** K=4 anchors, 3,000-token clusters, and seven events are implementation detail (and `1793879701` §8 already recommends against the 12k-token upfront spend). Ship three tools — `find`, `impact`, `verify`.
- **Subsystem clustering / architectural ontology** (Part 2 §4.2). Already hedged ("need not add a third level"); make the deferral explicit and permanent until asked for.
- **CI gating ambitions in Timeline C.** Advisory report only; merge-blocking is the noisy-bot death Part 3 names.
- **The collected file itself.** Five parts, three of which restate the others, is the symptom: one canonical one-page thesis plus a changelog. Part 5's §11 is the sixth retelling of the same idea — the idea does not need translating six times before it has a user.

## 7. Where Devin and GLM-5.3 agree, disagree, and what changes because of it

Legend: **AGREE** = both reads say it · **ADOPT** = GLM was right, Devin's §1–§5 above should be read with this correction · **HOLD** = Devin keeps its position and says why · **CHECK** = a GLM claim neither of us has verified.

```text
 topic                                   Devin §1–§5                      GLM-5.3 §1–§6                        verdict
 ──────────────────────────────────────  ───────────────────────────────  ───────────────────────────────────  ────────
 the one big thing                       map as shared checkable object   map as working interface: read to    AGREE — same sentence,
                                         for human / LLM / compiler        navigate, patch to plan, diff to      two wordings. "Three
                                         in three time-states              verify, provenance on every edge     tenses" is the keeper.
 "neither RA nor any MCP can do this"    not examined                     false as stated; true only as the    ADOPT — say "the
                                                                            COMBINATION; killer piece is the     combination"; never the
                                                                            before/intended/after report         absolute.
 bidirectional                           the proposal round-trip IS       LLM-asks/tool-answers is every MCP;   AGREE — only the
                                         the product                       only the proposal loop is new         proposal loop is new.
 "works for broken code" = the hinge     yes: the refactor is when the    oversold: tree-sitter indexers         HOLD, narrowed. Syntactic
                                         code is broken; map must         already "work" on broken code; RA     indexing is not the claim;
                                         survive that moment               loads non-Cargo projects; model       the claim is the INTENDED
                                                                            fallback has negative evidence        map + provenance degradation
                                                                            (0 vs 56 bindings; 10/19 edges)       on the SAME diffable map.
                                                                                                                  ADOPT: the model is not
                                                                                                                  part of the hinge; evidence
                                                                                                                  against it stands.
 comments / imports / globals in level 2 listed as part of the public    category error: imports are evidence  ADOPT — they are FIELDS of the
                                         surface                           fields, statics are items, doc        item/module record (doc, uses,
                                                                            comments as nodes invite identity     cfg, globals), not new node
                                                                            churn                                 kinds. The founder's request
                                                                                                                  is met by the record, not by
                                                                                                                  the node list.
 the tiny model                          enters at 0:50 of the demo to    "hypothesis wearing a product's        AGREE — demo and v1 need no
                                         keep L3 alive while broken        clothes"; rung-2 flat curve → stop     model; H-46 stays a ladder.
 death mode #3: nobody authors intent    missed (assumed the agent drafts GLM's strongest point: every prior      ADOPT — biggest risk in the
                                         it from PR text)                   intent ceremony (UML, ArchUnit,       corpus. Mitigation stays:
                                                                            design docs) was rejected             agent drafts, human only
                                                                                                                  approves; signal = do two
                                                                                                                  maintainers make a 2nd
                                                                                                                  proposal unprompted.
 ship surface                            seven-event protocol             three tools: find · impact · verify    ADOPT for v1; the protocol is
                                                                                                                  the internal design of `find`.
 cut Part 4, cut the collected file      —                                lab notebook; "sixth retelling"        HOLD on deletion (append-only
                                                                                                                  rule; history is evidence).
                                                                                                                  ADOPT the remedy: one canonical
                                                                                                                  page (thesis §13 is the draft)
                                                                                                                  + this file as changelog.
 100 MB RSS gate                         not discussed                    research constraint posing as a        ADOPT — gate for the fallback
                                                                            product gate                          rung only, not the product.
 demo order                              broken state mid-refactor        provenance banner first, broken        MERGE — banner at 0:00, break
                                         (the real scenario)               branch last                            mid-refactor, report, then
                                                                                                                  the degraded banner on screen.
 newcomer persona                        "the map alone"                  one command: `where do I even start`  ADOPT — that is Timeline A's
                                                                            → 10-item guided tour, zero config    first command.
 Serena / GitNexus / Moderne precedents  not examined                     named as existing                      CHECK — Serena (LSP-over-MCP)
                                                                                                                  and Aider repo map are known;
                                                                                                                  GitNexus and the Moderne
                                                                                                                  recipe-diff comparison are
                                                                                                                  unverified here.
```

What survives both reads, in one paragraph: the product is **one map in three tenses — as-is, intended, as-built — that an agent reads to navigate (`find`, `impact`), writes to plan (a graph patch a human approves), and the CLI diffs to verify (`verify` → achieved / missing / unexpected / unresolved), with provenance on every edge so the report stays honest while the code is mid-refactor and broken.** Comments, imports, cfg and globals ride on the item records as fields. The tiny model is a later rung for keeping resolved edges alive when the resolver cannot; it is not in the demo and not in v1. The absolute "nobody can do this" becomes "nobody ships the combination; the conformance report is the piece that does not exist anywhere."

## 8. The three cheapest signals before writing more documents

```text
 1. three-arm pilot (Part 3): baseline agent · +map (no model) · +map+model, 3 unseen repos, same token budget
       → if arm 1 ≈ arm 2 on correctness, navigation is a feature, not a product.     ~1 week, zero training
 2. intent authoring: on real disputed work, do two maintainers produce a SECOND graph-patch proposal unprompted?
       → if no, Timeline B collapses to navigator + reviewer.                            one design cycle
 3. H-46 ladder rung 2 (10 M tokens, ~2 h Mac or 1.5 min H100): held-out curve vs the rule baseline (56 bindings)
       → flat or below the rule → keep rules, stop training.                             one afternoon
```

Neither read changes the storage rule from Part 2 and the MISP note: exact facts, shared vocabulary and opinions (including everything in §4, §6 and §7 of this document) stay in separate layers with their creator named. This section's verdicts are Devin's opinion about GLM-5.3's opinion; the measured facts are the ones with numbers attached in Parts 1–2.

## 9. ELI5 ASCII of Proposal 01, and how the Sikos cybersecurity-knowledge-graph paper fits

*Added 2026-11-06 after the founder asked for a plain-language picture of this document and for the link to Sikos, L. F., "Cybersecurity knowledge graphs", Knowledge and Information Systems (2023), https://link.springer.com/article/10.1007/s10115-023-01860-3 — the paper behind ledger entries H-47…H-59.*

```text
DEEP-INSIGHT 04 IN ONE PICTURE

   a building has three drawings.  software has only the building.

        BLUEPRINT              AS-BUILT DRAWING             INSPECTOR
        what we agreed         what actually stands         diffs the two, writes the report
             │                        │                          │
             ▼                        ▼                          ▼
        INTENDED map             AS-IS map                   verify
        (LLM drafts a            (compiler / tree-sitter     achieved / missing /
         graph patch,             extracts it; every edge     unexpected / unresolved
         human approves)          says WHO found it)

   the one big thing:  ONE map, THREE tenses, SAME ids.
        agent READS it   →  find "where do retries get decided"   ·   impact(RetryPolicy)
        agent WRITES it  →  "add trait RetryPolicy; Client → trait instead of → FixedBackoff"
        CLI   DIFFS  it  →  after the edit: 2 of 3 done, 1 missing, 1 edge you didn't ask for

   WHY "BROKEN CODE" MATTERS
        the moment you most need the map is halfway through a refactor — and that is exactly when
        the code doesn't compile and rust-analyzer's answers degrade.
        file graph + public skeleton (mod / use / pub fn / pub struct / static / cfg / doc comment)
        need NO type-checker → they stay up.  only "resolved" edges fade — and the map SAYS they faded:

             ┌──────────────────────────────────────────────┐
             │ 1,204 files · 9,318 pub items                │
             │ 97 % exact  ·  3 % unresolved (proc-macro)   │   ← this banner is the product's honesty
             │ while broken:  9 more edges unresolved       │
             └──────────────────────────────────────────────┘

   TWO LEVELS = what the agent sees first
        level 1   file ↔ file            (mod, use, include!, cfg gates)
        level 2   each file's PUBLIC face (signatures, re-exports, globals, cfg, doc comment)
                  ← comments / imports / globals are FIELDS on each item's record, not new boxes on the graph
        level 3+  bodies, calls, flow     (there, hidden, opens on request)

   WHO BELIEVES IT, AND WHEN
        maintainer     "stale index + proc macros"       → re-extract in seconds, macros on
        RA contributor "that's our index in a wrapper"   → queries LSP can't do: diff two snapshots, verify a plan
        HN             "'works on broken code' = lies"   → the banner above, on a repo that doesn't build
        CTO/security   "false 'no dependents'"           → coverage envelope on every answer, zero egress
        newcomer       "where do I even start"           → 10-item tour, zero config, or gone in 90 s

   HOW IT DIES         nobody writes the INTENDED map (UML, ArchUnit, design docs all died this way)
   HOW WE HEDGE        the agent drafts it, the human only clicks approve
   CHEAP SIGNAL        do two maintainers make a SECOND proposal, unprompted?


HOW THE SIKOS PAPER (KAIS 2023, "Cybersecurity knowledge graphs") FITS

   the paper is NOT about code.  it is about a field that already solved OUR storage problem:
   many sources, different trust, facts mixed with guesses, shared across teams, never trusted blindly.

        cyber-threat world                         our Rust map
        ───────────────────────────────────        ─────────────────────────────────────────
        IOC observed by a sensor  (hard fact)      edge found by the compiler       (exact)
        analyst's attribution     (opinion)        LLM's "this is the retry loop"   (L6 meaning)
        STIX / MISP taxonomy      (shared words)   our node/edge kinds, tags        (vocabulary)
        confidence + provenance on every object    exact | projected | proposed | unresolved + creator
        sighting: "I saw this too / this is wrong" agent or human confirms / denies an edge
        warninglist: known false positives         "this edge is a macro artefact, ignore"
        first-seen / last-seen                     when did this edge appear / disappear across commits
        feeds between orgs, each with trust level  map shared across devices / agents / CI, provenance kept

   the lesson we took (H-47…H-59):   FACT  ≠  VOCABULARY  ≠  OPINION  — three layers, never written into
   each other, any query can peel one off.   that rule is what lets the INTENDED map (an opinion about the
   future) and the LLM's annotations sit next to compiler facts without poisoning them — which is exactly
   the objection every persona above raises ("will it lie to me?").

   so:   DI-04 says WHAT the object is (one map, three tenses).
         Sikos/MISP says HOW to store it so humans, LLMs and compilers can all write into it and still trust it.
```

## 10. Proposal 01 under full openness: open-source tool, open weights, open training data

*Added 2026-11-06 after the founder set the context: Parseltongue is a fully open-source tool, the fallback model ships with open weights, and the training data is published. This section re-derives §1–§9 under that assumption. It appends; it does not replace. Items marked CHECK are not verified.*

### 10.1 What openness changes in the one big thing

The sentence does not change. What changes is **who can write into the map and who can audit the writer**:

```text
   CLOSED (implicit in §1–§9)                     OPEN (this section)
   ─────────────────────────────────────────      ──────────────────────────────────────────────────────
   one vendor extracts, stores, diffs             anyone can read the extractor; every "exact" edge is
                                                  reproducible by running the same binary on the same commit
   provenance = "trust our label"                 provenance = "run it yourself" — the honesty layer becomes
                                                  checkable, not asserted
   the fallback model is a black box              open weights + open data + open eval → the "does it lie?"
                                                  question has a public answer per release
   the map of std/crates.io is private to the     the map of every crate version can be a PUBLIC FEED,
   tool                                           built once, downloaded by everyone (MISP feeds, H-59)
```

The last line is the genuinely new idea that openness unlocks. Reach (Part 1 §8) assumed the model must *memorise* std and the top crates' signatures because the resolver cannot run. Under openness the facts are **downloaded, not memorised**: a public, versioned, compiler-exact interface graph per crate@version — think docs.rs, but the graph instead of the HTML. The model then needs only the *algorithm* (binding, chaining, abstention), which is what §Q1 of the 100 MB discussion said is the small, defensible part. Reach shrinks from "model knows the ecosystem" to "model + feed".

### 10.2 The asset moves from the binary to the corpus

```text
   what is easy to copy            an MCP server over rust-analyzer   (GLM's HN persona: "a weekend project")
   what is hard to copy            the first open corpus of compiler-exact Rust interface graphs,
                                   repo-disjoint splits, post-cutoff held-outs, abstention labels, int4 scores,
                                   and a public leaderboard that anyone can submit a model to
   what compounds                  every published crate graph in the feed; every sighting/warninglist entry
                                   a user contributes (MISP H-51/H-53); every held-out repo added after a
                                   model's cutoff
```

This flips GLM's death mode #2 ("the tiny model never earns its seat"): in the open setting the model is **one entry on a leaderboard the project owns**, not the product. If the rule baseline (56 bindings) wins, the leaderboard says so and the project is still the benchmark everyone cites. The dataset is the paper the ML persona asked for, published before any model claim.

### 10.3 Reception, re-run under openness

```text
 persona                 §4/§6 objection                        what openness does to it
 ──────────────────────  ─────────────────────────────────────  ─────────────────────────────────────────────────────
 rust-analyzer contrib.  "reshipping our index; attribution?"   dissolves if licence-compatible (RA is MIT/Apache-2.0) and
                                                                 the `codemap` exporter is offered upstream as a PR; the
                                                                 remaining objection is semantic drift — answered by the
                                                                 open conformance test suite pinned to an RA version
 CTO, 40-person team     "egress, vendor, audit"                 dissolves: single static binary, no network, source
                                                                 auditable; the new objection is "who maintains it in
                                                                 3 years?" (see 10.4)
 security engineer       "false assurance; build-script ACE"     stronger: the coverage envelope is code they can read;
                                                                 the no-build mode is a documented flag they can diff
 HN top comment          "weekend project + confident guesses"   the artefact is runnable; the guesses have a public
                                                                 scorecard with abstention rate; the comment becomes
                                                                 "nice benchmark" — which is the win
 ML researcher           "distillation ceiling, contamination"   converts: repo-disjoint, post-cutoff held-outs, int4
                                                                 scores all published; the dataset is citable
 maintainer, 300k LOC    "will choke on my macros, go stale"     unchanged — openness does not fix proc macros or
                                                                 re-extract latency; still converted only by a clean run
 agent-harness builder   "why is yours the one that matters?"    slightly better: no auth, no vendor; still converted
                                                                 only by their own logs (three-arm pilot)
 newcomer in Cursor      "where do I start"                      unchanged — openness is invisible to them
 NEW: crate author       (not in §4/§6)                          "my crate's graph is in a public feed — is the licence
                                                                 respected, can I correct it?" → feed carries the
                                                                 crate's licence + a sighting channel for corrections
 NEW: contributor        (not in §4/§6)                          "can I add a query / an edge kind / a language?" →
                                                                 the record schema and plugin points must be documented
                                                                 before the first release or the project gets forks
```

Net: two of the three hardest personas (RA contributor, CTO) are mostly answered by openness itself; the maintainer and harness builder are not — they are answered only by engineering.

### 10.4 New risks that openness creates

```text
 risk                                     why it is real                                   mitigation
 ───────────────────────────────────────  ───────────────────────────────────────────────  ────────────────────────────────────────
 dataset licence poisoning                 training records derived from GPL / no-licence    licence filter at ingest (permissive
                                           repos cannot be redistributed as open data        only); per-record provenance carries
                                                                                              repo, commit, licence, extractor version
 LLM-produced labels (L6 meaning) may       Z.ai Coding Plan / API terms on redistributing    CHECK the terms before publishing any
   not be redistributable                  outputs or using them to train other models       GLM-written label; keep L6 labels in a
                                           are not verified here                              separate, separately licensed layer
 "open weights" ≠ "open model"             weights without data + eval + training recipe      publish all four; a model card that
                                           is the industry's usual half-measure               lists held-out repos by name
 upstream absorption                       rust-analyzer could add `codemap` export +         this is a WIN for the mission and a
                                           agent queries; the tool becomes a thin layer       loss only for a business; the asset
                                                                                              (corpus, feed, benchmark) survives it
 maintainership                            tracking RA/rustc churn is a permanent cost;        pin RA version per release; conformance
                                           an abandoned extractor silently produces stale      suite in CI; the feed's `last_seen`
                                           "exact" edges — the worst failure in Part 2 §5      shows staleness instead of hiding it
 public feed trust                         a poisoned or stale crate graph in the feed is       feeds are signed per extractor version;
                                           false completeness at ecosystem scale               MISP-style distribution levels and
                                                                                              warninglists (H-53/H-59)
 no revenue                                a community project does not fund the three-arm    the pilot and rung-2 ladder cost < $50
                                           pilot or the maintainer                             of compute; the maintainer is the cost
```

### 10.5 What is explicitly open in each layer

```text
   layer                                  licence / artefact                                         why this layer
   ─────────────────────────────────────  ─────────────────────────────────────────────────────────  ───────────────────────────
   CLI + extractor + query engine         Apache-2.0 OR MIT (matches rustc / RA)                      drop-in for the ecosystem
   record schema (Part 2 §2.1) + ids      versioned spec file in the repo; semver'd                   forks and plugins need it
   exact graphs of crates.io versions     public feed, per-crate licence passed through                the Reach fix (10.1)
   training corpus (RA-labelled)          permissive-only sources; CC-BY-4.0 or ODC-BY on the labels   the asset (10.2)
   L6 meaning labels (LLM-written)        separate layer; licence depends on CHECK in 10.4             opinion ≠ fact (Sikos rule)
   fallback model weights                 open weights + model card + eval harness + recipe           "open model", not half
   benchmark + leaderboard                repo-disjoint, post-cutoff held-outs, int4 scores            the thing that outlives us
```

### 10.6 The demo under openness (delta to §5 / GLM §4)

Two frames change:

```text
  0:00  pt map .   → banner now also prints:  extractor v0.3 (rust-analyzer 2026-10-27) · std + 38 crate graphs from feed
  1:25  pt report  → the PR description ends with a reproducibility line:
                     "re-run: pt verify --at <sha> --extractor v0.3 → identical report"  (anyone can check it)
```

### 10.7 ELI5 ASCII

```text
   CLOSED:   "trust us, this edge is exact."                 OPEN:   "here is the binary, the commit, the RA version. run it."

   CLOSED:   the model must MEMORISE std + 100 crates.       OPEN:   the graphs of std + every crate are a public download;
                                                                     the model only learns HOW to connect, not WHAT exists.

   CLOSED:   the product is the tool.                        OPEN:   the product is the CORPUS + the FEED + the BENCHMARK;
                                                                     the tool is how you use them; the model is one leaderboard row.

   who stops objecting:   RA contributors (we are upstream-friendly) · CTOs (no egress, auditable) · ML researchers (citable data)
   who does not care:     the maintainer with proc macros · the newcomer · the harness builder — only engineering converts them
   new people to answer:  crate authors ("that's my crate in your feed") · contributors ("where is the schema?")

   new things to get right:   licences on every training record · Z.ai label terms (CHECK) · signed feeds · an RA-version pin
```

### 10.8 What changes in §7's verdicts

- GLM death mode #2 (model never earns its seat) → **neutralised**: the benchmark is the deliverable; the model is optional.
- GLM "rust-analyzer ships agent mode and you die" → **reclassified** as mission success; only the corpus/feed/benchmark need to outlive the tool.
- GLM "cut the 100 MB gate" → stands; under openness the gate is a leaderboard *column*, not a product requirement.
- GLM death mode #3 (nobody authors intent) → **unchanged**; openness does not make anyone write a blueprint.
- §7 "nobody ships the combination" → add: *and nobody publishes the compiler-exact Rust graph corpus or a per-crate graph feed* — that absence is verifiable and, unlike the product claim, cheap to fill first.
