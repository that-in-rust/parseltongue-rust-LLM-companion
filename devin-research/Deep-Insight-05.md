# Deep Insight 05 — The thesis

*2026-11-06. Proposal 01 stated top-down: the answer first, then the three reasons it is the answer, then the evidence under each. Nothing here is hedged by attribution; what is measured is marked measured, what is a bet is marked bet.*

---

## The answer

**A Rust codebase's structure — what connects to what — should exist as one open map in three tenses: what the code is, what we intend it to be, and what it became after the change. The agent reads the map to navigate, writes to it to plan, and the tool diffs it to verify. Every edge says how it was found and whether it was resolved, so the map stays usable and honest while the code is mid-refactor and does not compile. The compiler-exact corpus and the per-crate graph feed are published first, because they are the part no one has and the part that outlives any tool or model.**

```text
                 ┌────────────────────────────────────────────────────────────────┐
                 │   ONE OPEN MAP · THREE TENSES · PROVENANCE ON EVERY EDGE        │
                 │   agent reads it to navigate · writes it to plan · tool diffs it │
                 │   corpus and feed first · tool second · model last               │
                 └────────────────────────────────────────────────────────────────┘
                   ┌───────────────────────┼────────────────────────────┐
     ┌─────────────┴────────────┐  ┌────────┴──────────┐   ┌─────────────┴────────────┐
     │ 1  the object is new     │  │ 2  Rust makes it   │   │ 3  the order of work is  │
     │    and nothing else is   │  │    exact, and the  │   │    corpus → three tools  │
     │    the product           │  │    broken moment   │   │    → model, gated by     │
     │                          │  │    is the real one │   │    three cheap signals   │
     └──────────────────────────┘  └───────────────────┘   └──────────────────────────┘
```

---

## 1. The object is new, and nothing else is the product

### 1.1 Three tenses on one vocabulary
- **As-is**: the graph the source establishes today. **Intended**: a graph patch — add this trait, flip these dependencies — drafted by the agent, approved by a human. **As-built**: the graph re-extracted after the edit.
- The product is the loop: *propose → build → re-extract → compare → report*. The report has four columns: achieved, missing, unexpected, unresolved.
- Every discipline that builds physical things keeps a blueprint, an as-built drawing, and an inspector who diffs them. Software keeps only the building. Agents inherit that poverty and re-derive the architecture from grep on every task.
- A tool where the LLM only reads a graph is a code search engine. A tool where the LLM only writes a plan is a design-doc generator. The round trip is the only thing that is not already shipping.

### 1.2 Two levels are visible; the rest opens on demand
- **Level 1** — file to file: module declarations, imports, include paths, cfg gates. Readable by a syntax parser; never unavailable.
- **Level 2** — the public face of each file: every public signature, re-export, static and constant, cfg and feature gate, the doc comment, item-level macro names. The test for membership: *could this line change what another file is allowed to write?*
- Comments, imports and globals are fields on the item record, not nodes. Imports are evidence on dependency edges; statics are items; a doc comment is the contract's prose attached to the item it documents.
- Bodies, calls, data flow, lock order, Send/Sync boundaries, and the semantic layer ("this is the retry loop") are kept and shown on expansion. They are not the default view.

### 1.3 Honesty is a storage rule, not a promise
- Every edge carries one of `exact · projected · proposed · unresolved`, the creator, the extractor version and the build context. An unresolved edge is never displayed as absence.
- Three layers are never written into each other: compiler facts, shared vocabulary (node and edge kinds, tags), and opinions (the intended map, semantic annotations, model proposals, human or agent sightings). Any query can peel a layer off.
- This is the same structure the cyber-threat-intelligence field settled on for knowledge graphs fed by many sources of unequal trust: provenance and confidence on every object, sightings to confirm or deny, warninglists for known false positives, first-seen and last-seen, feeds with distribution levels. It is what lets an opinion about the future sit next to a compiler fact without poisoning it.

### 1.4 Open means the asset is the corpus, not the binary
- Four published artefacts: the Apache/MIT tool and its versioned record schema; **a public feed of compiler-exact graphs for every crate version** — docs.rs, but the graph; **the training corpus** with repository-disjoint splits and held-out repositories newer than any base model's cutoff; and open weights with the card, the eval harness and the recipe.
- The feed changes what a fallback model must know. It no longer memorises the standard library and a hundred crates; it downloads the facts and learns only how to connect them — binding, chaining, abstaining. That is the small, defensible part.
- The model is one row on a leaderboard the project owns. If a rule beats it, the leaderboard says so and the project is still the benchmark everyone cites.

---

## 2. Rust makes the map exact, and the broken moment is the real one

### 2.1 Rust is the one mainstream language where the map is a fact
- Names are static, visibility is written in the source, each trait is implemented once per type, there is no reflection and no eval. The compiler's resolver produces the public-interface graph exactly, so every approximation — a large model, a small model, a heuristic — can be scored against truth. No other mainstream language allows this.
- Measured on a 42k-line, eleven-crate workspace: syntax parse 0.07 s; full rust-analyzer analysis 29 s at 2.3 GB; near-zero unknown expressions with proc macros enabled; warm incremental `cargo check` 0.69 s.
- Measured on two large public Rust codebases: 65 % of call sites are method calls, and only a quarter of their receivers can be typed from the enclosing function alone. There is no parser-only shortcut to the resolved graph. The resolver builds; nothing else can.

### 2.2 The code is broken exactly when the map is needed most
- The tool is for cross-cutting change. Halfway through extracting a trait, forty files fail to type-check and the resolver's answers degrade across the whole blast radius. The intended map is the only stable thing in the room.
- Level 1 and the public skeleton need no type-checker, so they stay up on a file that does not compile. Only resolution degrades — and the banner says by how much: "97 % exact, 3 % unresolved; while broken, 9 more edges unresolved."
- The claim is not that a syntax parser can index broken files; several tools do that. The claim is that the intended map and the degradation of the actual map live on the same diffable object, so `verify` still answers mid-refactor. No fallback model is required for this; the model is a later rung for keeping resolved edges alive when the resolver cannot.

### 2.3 The absolute claim is false; the combination claim is true and has one killer piece
- Each piece exists somewhere: module graphs, public-API diffs, LSP-over-MCP symbol navigation, ranked repo maps within a token budget, resolver support for non-Cargo projects.
- No one ships exact Rust edges plus budgeted agent queries plus the current/intended distinction plus per-edge provenance. The before/intended/after conformance report exists nowhere. The compiler-exact Rust graph corpus and the per-crate graph feed exist nowhere. Those absences are verifiable and cheap to fill.

### 2.4 Who is convinced by what
- **Convinced by openness alone**: the rust-analyzer contributor (licence-compatible, exporter offered upstream), the CTO (single static binary, no egress, auditable), the ML researcher (citable repository-disjoint data), the skeptic who calls it a weekend project (the artefact is runnable and the guesses have a public scorecard).
- **Convinced only by engineering**: the maintainer of a 300k-line workspace (proc macros on, re-extract in seconds, one `impact` answer that grep gets wrong), the harness builder (an A/B in their own logs), the newcomer (one command, `where do I even start`, that works on broken work-in-progress with no configuration).
- **Created by openness**: the crate author who finds their crate in the feed (licence passed through, a correction channel) and the contributor who wants to add a query or an edge kind (schema and plugin points documented before the first release).

### 2.5 The negatives are already on the record and shape the order of work
- Frontier models asked to draw signature dependencies omitted six of fourteen; agents working from source alone found ten of nineteen local edges; a learned binding model accepted zero bindings where a rule accepted fifty-six. An omitted edge looks like independence, which is the most dangerous output a structural tool can produce.
- Therefore no model is in the demo and no model is in version one. The model is a ladder with a stop rule, run after the corpus exists.

---

## 3. The order of work: corpus, three tools, then the model — gated by three signals

### 3.1 Three signals, each cheap, each able to kill a branch of the thesis
```text
 1. three-arm pilot on three unseen repositories, equal token budget:
      baseline agent · agent + map without model · agent + map + model
      if the first two arms tie on correctness, navigation is a feature, not a product      ~1 week, no training
 2. intent authoring on real disputed work:
      do two maintainers produce a SECOND graph patch unprompted?
      if not, the planning tense collapses and the product is a navigator plus a reviewer   one design cycle
 3. model ladder, second rung (10 M tokens, an afternoon):
      held-out curve against the rule baseline; flat or below → keep rules, stop training   one afternoon, < $1
```

### 3.2 Three tools, not a protocol
- `find` — seven words in, four candidate entities out, each with its public anchor, paths, blast radius and a bounded neighbourhood under a token budget. The retrieval protocol is its internals, not its interface.
- `impact` — callers, implementors, constructors, cfg-gated call sites, with file and line, and the coverage envelope printed on every answer.
- `verify` — the intended patch against the as-built graph: achieved, missing, unexpected, unresolved. Advisory only; it blocks no merge until its recall on seeded mismatches is measured.

### 3.3 The ninety-second proof
```text
  0:00  map      banner: crates, files, public items; % exact; % unresolved and why; extractor + resolver version; feed crates loaded
  0:10  find     "where do retries get decided"  →  four candidates; picks one; neighbourhood under budget; blast radius
  0:25  propose  "make retry policy pluggable behind a trait"  →  graph patch on screen; human approves
  0:40  edit     halfway, the crate does not compile
  0:50  verify   still answers: intended 3 · achieved 2 · missing 1 · unexpected 1 · 9 edges unresolved while broken
  1:10  fix      build green; verify: 3 of 3, nothing unexpected
  1:25  report   before / intended / after, with a reproducibility line anyone can re-run at that commit and extractor version
```
- Prerequisites, none of which need a model: level-1 and level-2 extraction on non-compiling files; stable item identity across rename and move; a patch format the agent emits and the tool diffs; incremental re-extraction fast enough to run on every save. Identity and re-extraction latency are where the demo dies first.

### 3.4 Publish the corpus and the feed before any product or model claim
- Permissive-licence sources only, with repository, commit, licence and extractor version on every record. Signed feeds with distribution levels and warninglists. The resolver version pinned per release, with a conformance suite in CI. A last-seen stamp on every edge so staleness is shown, never hidden.
- Semantic labels written by a large model live in their own layer with their own licence, checked before publication.

### 3.5 What is deliberately not done
- No model-first story. No memory gate as a product requirement — it is a leaderboard column.
- No comment, import or global as a graph node.
- No claim that nothing else can do any of this; the claim is that nobody ships the combination, and nobody publishes the corpus or the feed.
- No deletion of earlier research to look tidy: one canonical page, and the collected record as the changelog.
- No reception table treated as market evidence; the market evidence is the three signals.

---

## The one risk the thesis does not remove

**Nobody writes the intended map.** Every earlier form of architectural intent — diagrams, architecture tests, design documents — was rejected as ceremony. Openness does not change that. The only mitigation is that the agent drafts the patch from the work already described in the pull request and the human merely approves. Signal 2 is the test. If it fails, the product is a navigator and a reviewer — a utility, not a company — and the corpus, the feed and the benchmark are still worth having built.
