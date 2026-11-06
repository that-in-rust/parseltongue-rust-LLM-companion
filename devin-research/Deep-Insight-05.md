# Deep Insight 05 — The thesis

*2026-11-06, revision 3. Proposal 01 stated top-down: the answer first, then the three reasons it is the answer, then the evidence under each. Scope change in this revision: there is no verification step, no diff, no "as-built" map and no conformance report. The future is simply the part of the one map that has not turned real yet. What is measured is marked measured; what is a bet is marked bet.*

---

## The answer

**A Rust codebase's structure — what connects to what — should exist as one open map that holds both what the code is and what we intend it to become. The agent reads the map to navigate and writes to it to plan; the compiler fills in the facts. Every edge says how it was found: `exact`, `unresolved`, or `proposed`. A proposed edge is the future; it turns exact on its own the moment the compiler finds it in the source. The map stays readable and honest while the code is mid-refactor and does not compile. The compiler-exact corpus and the per-crate graph feed are published first, because they are the part no one has and the part that outlives any tool or model.**

```text
                 ┌───────────────────────────────────────────────────────────────┐
                 │   ONE OPEN MAP · CURRENT AND FUTURE EDGES · PROVENANCE ON EACH │
                 │   agent reads it to navigate · writes it to plan · compiler    │
                 │   turns proposed edges exact · corpus and feed first           │
                 └───────────────────────────────────────────────────────────────┘
                   ┌──────────────────────┼─────────────────────────────┐
     ┌─────────────┴────────────┐  ┌───────┴───────────┐   ┌─────────────┴────────────┐
     │ 1  the object is new     │  │ 2  Rust makes it   │   │ 3  the order of work is  │
     │    and nothing else is   │  │    exact, and the  │   │    corpus → three tools  │
     │    the product           │  │    broken moment   │   │    → model, gated by     │
     │                          │  │    is the real one │   │    three cheap signals   │
     └──────────────────────────┘  └───────────────────┘   └──────────────────────────┘
```

---

## 1. The object is new, and nothing else is the product

### 1.1 Current and future on one vocabulary
- The map has one set of nodes and edges. An edge found by the compiler is `exact`. An edge the compiler could not settle is `unresolved`. An edge the agent or a human adds because the code *should* have it is `proposed`. Same ids, same kinds, same queries.
- Planning is writing proposed edges: "a trait `RetryPolicy`; `Client` depends on it instead of on `FixedBackoff`; `FixedBackoff` implements it." Nothing else is required of the plan. No document, no checklist, no step that grades it.
- The plan completes itself. Each time the source is re-extracted, every proposed edge the compiler now finds becomes exact; the rest stay proposed and visible. "Done" is a map with no proposed edges left in that region. New exact edges nobody proposed are simply shown; the map does not judge them.
- Every discipline that builds physical things draws what it intends before it builds. Software keeps only the building. Agents inherit that poverty and re-derive the architecture from grep on every task; they also plan in prose, which cannot be looked up later. A proposed edge can.
- A tool where the LLM only reads a graph is a code search engine. A map the LLM can also write the future into is the thing not already shipping.

### 1.2 Two levels are visible; the rest opens on demand
- **Level 1** — file to file: module declarations, imports, include paths, cfg gates. Readable by a syntax parser; never unavailable.
- **Level 2** — the public face of each file: every public signature, re-export, static and constant, cfg and feature gate, the doc comment, item-level macro names. The test for membership: *could this line change what another file is allowed to write?*
- Comments, imports and globals are fields on the item record, not nodes. Imports are evidence on dependency edges; statics are items; a doc comment is the contract's prose attached to the item it documents.
- Bodies, calls, data flow, lock order, Send/Sync boundaries, and the semantic layer ("this is the retry loop") are kept and shown on expansion. They are not the default view.

### 1.3 Honesty is a storage rule, not a promise
- Every edge carries `exact · unresolved · proposed`, the creator, the extractor version and the build context. An unresolved edge is never displayed as absence; a proposed edge is never displayed as fact.
- Three layers are never written into each other: compiler facts, shared vocabulary (node and edge kinds, tags), and opinions (proposed edges, semantic annotations, model guesses, human or agent sightings). Any query can peel a layer off.
- This is the structure the cyber-threat-intelligence field settled on for knowledge graphs fed by many sources of unequal trust: provenance and confidence on every object, sightings to confirm or deny, warninglists for known false positives, first-seen and last-seen, feeds with distribution levels. It is what lets an opinion about the future sit next to a compiler fact without poisoning it.

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
- The tool is for cross-cutting change. Halfway through extracting a trait, forty files fail to type-check and the resolver's answers degrade across the whole blast radius. The proposed edges are the only stable thing in the room: they say where the work is going while the exact edges are in flux.
- Level 1 and the public skeleton need no type-checker, so they stay up on a file that does not compile. Only resolution degrades — and the banner says by how much: "97 % exact, 3 % unresolved; while broken, 9 more edges unresolved."
- The claim is not that a syntax parser can index broken files; several tools do that. The claim is that the current and the future edges live on the same object, so the agent can keep navigating by the plan while the compiler catches up. No fallback model is required for this; the model is a later rung for keeping resolved edges alive when the resolver cannot.

### 2.3 The absolute claim is false; the combination claim is true and has one missing piece
- Each piece exists somewhere: module graphs, public-API listings, LSP-over-MCP symbol navigation, ranked repo maps within a token budget, resolver support for non-Cargo projects.
- No one ships exact Rust edges plus budgeted agent queries plus proposed edges on the same map plus per-edge provenance. A map the agent can write the future into exists nowhere. The compiler-exact Rust graph corpus and the per-crate graph feed exist nowhere. Those absences are verifiable and cheap to fill.

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
 2. future-map authoring on real work:
      do agents and maintainers keep adding proposed edges after the first week, unprompted?
      if not, the future half of the map is dead and the product is a navigator              one design cycle
 3. model ladder, second rung (10 M tokens, an afternoon):
      held-out curve against the rule baseline; flat or below → keep rules, stop training   one afternoon, < $1
```

### 3.2 Three tools, not a protocol
- `find` — seven words in, four candidate entities out, each with its public anchor, paths, blast radius and a bounded neighbourhood under a token budget. The retrieval protocol is its internals, not its interface.
- `impact` — callers, implementors, constructors, cfg-gated call sites, with file and line, and the coverage envelope printed on every answer. Proposed edges appear in the same answer, marked.
- `propose` — writes proposed nodes and edges into the map, with creator and reason. There is no fourth tool that grades them; re-extraction turns them exact or leaves them proposed.

### 3.3 The ninety-second proof
```text
  0:00  map      banner: crates, files, public items; % exact; % unresolved and why; extractor + resolver version; feed crates loaded
  0:10  find     "where do retries get decided"  →  four candidates; picks one; neighbourhood under budget; blast radius
  0:25  propose  "make retry policy pluggable behind a trait"  →  three proposed edges appear on the map, dashed
  0:40  edit     halfway, the crate does not compile
  0:50  impact   still answers: exact edges that survive, 9 newly unresolved, the 3 proposed edges still showing the way
  1:10  fix      build green; re-extract: 2 proposed edges are now exact, 1 is still proposed — the agent sees what is left
  1:25  map      the dashed edge turns solid after the last edit; the region has no proposed edges; the plan simply became the code
```
- Prerequisites, none of which need a model: level-1 and level-2 extraction on non-compiling files; stable item identity across rename and move; a proposal format the agent emits; incremental re-extraction fast enough to run on every save. Identity and re-extraction latency are where the demo dies first.

### 3.4 Publish the corpus and the feed before any product or model claim
- Permissive-licence sources only, with repository, commit, licence and extractor version on every record. Signed feeds with distribution levels and warninglists. The resolver version pinned per release, with a conformance suite in CI. A last-seen stamp on every edge so staleness is shown, never hidden.
- Semantic labels written by a large model live in their own layer with their own licence, checked before publication.

### 3.5 What is deliberately not done
- No verification step, no diff, no "as-built" map, no conformance report, no CI gate. The future is the part of the map that is not yet exact; the compiler is the only judge, and it judges by extraction, not by a tool.
- No model-first story. No memory gate as a product requirement — it is a leaderboard column.
- No comment, import or global as a graph node.
- No claim that nothing else can do any of this; the claim is that nobody ships the combination, and nobody publishes the corpus or the feed.
- No deletion of earlier research to look tidy: one canonical page, and the collected record as the changelog.
- No reception table treated as market evidence; the market evidence is the three signals.

---

## The one risk the thesis does not remove

**Nobody writes the future half of the map.** Every earlier form of architectural intent — diagrams, architecture tests, design documents — was rejected as ceremony. Openness does not change that. The mitigation is that writing the future is one `propose` call the agent makes anyway while planning, and it costs nothing to leave the edges there. Signal 2 is the test. If it fails, the product is a navigator — a utility, not a company — and the corpus, the feed and the benchmark are still worth having built.
