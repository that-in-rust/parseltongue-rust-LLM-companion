# Deep Insight 05 — The thesis

*2026-11-06, revision 4. Proposal 01 stated top-down: the answer, the three reasons, the evidence under each, then the system definition and the entity model. Scope decisions carried by this revision: (a) no verification step, diff, as-built map or conformance report; (b) rust-analyzer and tree-sitter are not runtime components — they are the teacher that labels the training corpus and the judge that scores the model; (c) Parseltongue at runtime is one CLI interface over its own deterministic CLI tools and a set of tiny task-models that call each other to answer the agent-LLM; (d) the tiny models run on CPU or GPU; (e) the future half of the map is a structure the tool provides and never fills — only the orchestrator-LLM, the agent-LLM or a human writes into it. Earlier revisions are kept in the collected Deep-Insight-01 as Parts 7, 7b and 7c; this revision supersedes them where they differ. What is measured is marked measured; what is a bet is marked bet.*

---

## The answer

**A Rust codebase's structure — what connects to what — should exist as one open map that holds both what the code is and what the agent intends it to become. Parseltongue is the CLI the agent-LLM talks to: behind it, small open-weight models extract the map from any Rust text — a workspace, a diff, a snippet, code that does not compile, code the agent has only drafted — and deterministic tools store, index and answer bounded questions over it. Every edge says who produced it and how sure they were. The future is the part of the map the agent wrote in; it becomes current when the same edge is extracted from the real source. The compiler is the teacher the models learn from and the judge they are scored against — never a runtime dependency. The compiler-labelled corpus and the per-crate graph feed are published first, because they are the part no one has and the part that outlives any tool or model.**

```text
   agent-LLM / orchestrator-LLM / human
          │  natural-language question · bounded answer · proposed edges in · neighbourhoods out
          ▼
   ┌─────────────────────────────── PARSELTONGUE (one CLI) ────────────────────────────────┐
   │  dispatcher                                                                            │
   │  ├─ tiny task-models  (open weights, CPU or GPU)   extract · bind · rank · abstain · label │
   │  └─ deterministic CLI tools (ours)                 store · index · find · impact · feed  │
   │  the models and the tools call each other; the dispatcher composes one answer            │
   └────────────────────────────────────────────────────────────────────────────────────────┘
          ▲ trained on                               ▲ scored by
   rust-analyzer / rustc  (off-line teacher)   rust-analyzer / rustc on held-out repos  (off-line judge)
```

---

## 1. The object is new, and nothing else is the product

### 1.1 Current and future on one vocabulary
- The map has one set of node and edge kinds. An edge produced by the extractor from real source is `extracted`, with a confidence. An edge the extractor declined to emit is `abstained`, recorded so that absence is never silent. An edge the agent-LLM, the orchestrator-LLM or a human adds because the code *should* have it is `proposed`.
- The tool provides the proposed slot and never fills it. Parseltongue has no component that invents future structure. The agent obtains proposed edges in one of two ways: it states them directly, or it hands its drafted code to the same extractor that reads real code, and labels the output `proposed`. Either way the author of record is the LLM or the human, not the tool.
- The plan completes itself. Each time real source is extracted, a proposed edge that now appears as extracted is marked realized; the rest stay proposed and visible. "Done" is a region with no open proposals. Extracted edges nobody proposed are simply shown; the map does not judge them.
- Every discipline that builds physical things draws what it intends before it builds. Software keeps only the building. Agents inherit that poverty: they re-derive the architecture from grep on every task and plan in prose that cannot be looked up later. A proposed edge can.
- A tool where the LLM only reads a graph is a code search engine. A map the LLM can also write the future into, on the same vocabulary it navigates by, is the thing not already shipping.

### 1.2 Two levels are visible; the rest opens on demand
- **Level 1** — file to file: module declarations, imports, include paths, cfg gates.
- **Level 2** — the public face of each file: every public signature, re-export, static and constant, cfg and feature gate, the doc comment, item-level macro names. The test for membership: *could this line change what another file is allowed to write?*
- Comments, imports and globals are fields on the item record, not nodes. Imports are evidence on dependency edges; statics are items; a doc comment is the contract's prose attached to the item it documents.
- Bodies, calls, data flow, lock order, Send/Sync boundaries, and the semantic layer ("this is the retry loop") are kept and shown on expansion. They are not the default view.

### 1.3 Honesty is a storage rule, not a promise
- Every record names its creator (which model and revision, which tool, which LLM, which human), its confidence, the input digest it was produced from, and the extractor version. Abstention is a record, not a gap. A proposed edge is never displayed as a fact about the code.
- Three layers are never written into each other: extracted structure, shared vocabulary (node and edge kinds, tags), and opinions (proposed edges, semantic labels, sightings). Any query can peel a layer off.
- This is the structure the cyber-threat-intelligence field settled on for knowledge graphs fed by many sources of unequal trust: provenance and confidence on every object, sightings to confirm or deny, warninglists for known false positives, first-seen and last-seen, feeds with distribution levels. It is what lets an opinion about the future sit next to an extracted edge without poisoning it.

### 1.4 Open means the asset is the corpus, not the binary
- Four published artefacts: the Apache/MIT CLI and its versioned record schema; **a public feed of compiler-labelled graphs for every crate version** — docs.rs, but the graph; **the training corpus** with repository-disjoint splits and held-out repositories newer than any base model's cutoff; and the tiny models as open weights with card, eval harness and recipe.
- The feed changes what the extractor must know. It need not memorise the standard library and a hundred crates; it downloads those facts and learns only how to connect them — binding, chaining, abstaining. That is the small, defensible part.
- Each tiny model is one row on a leaderboard the project owns, scored by the compiler on held-out repositories. If a rule beats a model, the leaderboard says so and the project is still the benchmark everyone cites.

---

## 2. Rust makes the truth exact, and the broken moment is the real one

### 2.1 Rust is the one mainstream language where the map has a ground truth
- Names are static, visibility is written in the source, each trait is implemented once per type, there is no reflection and no eval. The compiler's resolver produces the public-interface graph exactly, so every extractor — a large model, a tiny model, a heuristic — can be scored against truth. No other mainstream language allows this. This is why the teacher-and-judge arrangement is possible at all.
- Measured on a 42k-line, eleven-crate workspace: full rust-analyzer analysis 29 s at 2.3 GB; near-zero unknown expressions with proc macros enabled. That is the cost of truth, paid once per corpus release, off-line.
- Measured on two large public Rust codebases: 65 % of call sites are method calls, and only a quarter of their receivers can be typed from the enclosing function alone. The extractor cannot be a syntax-shaped trick; it has to learn resolution, or fetch the needed signatures from the feed, or abstain.

### 2.2 The code is broken exactly when the map is needed most
- The tool is for cross-cutting change. Halfway through extracting a trait, forty files fail to type-check and anything that depends on a build or a type-checker degrades across the whole blast radius. A model that reads text does not fail to load; it reads what is there and abstains where it cannot resolve. The proposed edges say where the work is going while the extracted edges are in flux.
- The claim is not that a parser can index broken files; several tools do that. The claim is that the current and the future edges live on the same object, extracted by the same model from real code and from drafts, so the agent keeps navigating by the plan while the code catches up.

### 2.3 The absolute claim is false; the combination claim is true and has one missing piece
- Each piece exists somewhere: module graphs, public-API listings, LSP-over-MCP symbol navigation, ranked repo maps within a token budget.
- No one ships a build-free, open-weight extractor for Rust structure plus budgeted agent queries plus proposed edges on the same map plus per-record provenance. A map the agent can write the future into exists nowhere. The compiler-labelled Rust graph corpus and the per-crate graph feed exist nowhere. Those absences are verifiable and cheap to fill.

### 2.4 Who is convinced by what
- **Convinced by openness alone**: the rust-analyzer contributor (we consume their output off-line and publish the scores), the CTO (single binary, no egress, auditable, no build step in the agent's loop), the ML researcher (citable repository-disjoint data, a compiler as judge), the skeptic who calls it a wrapper (there is nothing to wrap; the extractor is ours and scored in public).
- **Convinced only by engineering**: the maintainer of a 300k-line workspace (proc macros, re-extraction in seconds, one `impact` answer that grep gets wrong), the harness builder (an A/B in their own logs), the newcomer (one command, `where do I even start`, on broken work-in-progress with no configuration).
- **Created by openness**: the crate author who finds their crate in the feed (licence passed through, a correction channel) and the contributor who wants to add a query, an edge kind or a model (schema and plugin points documented before the first release).

### 2.5 The negatives are already on the record and shape the order of work
- Frontier models asked to draw signature dependencies omitted six of fourteen; agents working from source alone found ten of nineteen local edges; a learned binding model accepted zero bindings where a rule accepted fifty-six. An omitted edge looks like independence, which is the most dangerous output a structural tool can produce.
- Therefore the extractor ships only with its leaderboard row visible, abstention on, and the corpus published first. "If the tiny model works" is the bet of this proposal; the ladder in 3.1 is how it is tested cheaply, and a rule-based extractor remains a row on the same leaderboard.

---

## 3. The order of work: corpus, three tools, then the models — gated by three signals

### 3.1 Three signals, each cheap, each able to kill a branch of the thesis
```text
 1. three-arm pilot on three unseen repositories, equal token budget:
      baseline agent · agent + map from the rule extractor · agent + map from the tiny model
      if the first two arms tie on correctness, navigation is a feature, not a product      ~1 week, no training
 2. future-map authoring on real work:
      do agents and maintainers keep writing proposed edges after the first week, unprompted?
      if not, the future half of the map is dead and the product is a navigator              one design cycle
 3. extractor ladder, second rung (10 M tokens, an afternoon):
      held-out score against the rule extractor at a fixed wrong-edge budget; flat or below → keep rules   one afternoon, < $1
```

### 3.2 Three tools, not a protocol
- `find` — seven words in, four candidate entities out, each with its public anchor, paths, blast radius and a bounded neighbourhood under a token budget. The retrieval protocol is its internals, not its interface.
- `impact` — callers, implementors, constructors, cfg-gated sites, with file and line, and the coverage envelope printed on every answer. Proposed edges appear in the same answer, marked.
- `propose` — writes proposed nodes and edges into the map on behalf of a named LLM or human, with reason. The tool validates shape and identity only; it never originates content.
- `map` is the extraction entry point (real source or an agent draft); `feed` fetches published crate graphs. Both are internal to the dispatcher; the agent sees three verbs.

### 3.3 The ninety-second proof
```text
  0:00  map      banner: crates, files, public items; % extracted at confidence ≥ 0.9; % abstained and why; model + feed versions
  0:10  find     "where do retries get decided"  →  four candidates; picks one; neighbourhood under budget; blast radius
  0:25  propose  the agent drafts a trait sketch; the extractor reads the draft; three edges enter the map as proposed, dashed
  0:40  edit     halfway, the crate does not compile; nothing in the loop needed it to
  0:50  impact   extracted edges that survive, 9 newly abstained, the 3 proposed edges still showing the way
  1:10  edit     re-extraction: 2 proposed edges are now realized, 1 is still open — the agent sees what is left
  1:25  map      the last dashed edge turns solid; the region has no open proposals; the plan simply became the code
```
- Prerequisites: an extractor whose level-1 and level-2 output on non-compiling files clears the leaderboard bar; stable item identity across rename and move; re-extraction fast enough to run on every save (GPU where available, CPU as the floor). Identity and re-extraction latency are where the demo dies first; extractor accuracy is where the proposal dies.

### 3.4 Publish the corpus and the feed before any product or model claim
- Permissive-licence sources only, with repository, commit, licence and labeller version on every record. Signed feeds with distribution levels and warninglists. The teacher's version pinned per corpus release. A last-seen stamp on every record so staleness is shown, never hidden.
- Semantic labels written by a large model live in their own layer with their own licence, checked before publication.

### 3.5 What is deliberately not done
- No verification step, diff, as-built map, conformance report or CI gate. The future is the part of the map that is not yet realized; realization is observed by extraction, not judged by a tool.
- No rust-analyzer, tree-sitter or cargo in the runtime. They are teacher and judge; a build is never required to answer the agent.
- No tool-authored future. The proposed slot is a structure; only an LLM or a human fills it.
- No comment, import or global as a graph node.
- No claim that nothing else can do any of this; the claim is that nobody ships the combination, and nobody publishes the corpus or the feed.
- No deletion of earlier research to look tidy: one canonical page, and the collected record as the changelog.
- No reception table treated as market evidence; the market evidence is the three signals.

---

## 4. The system, defined

```text
 component                 what it is                                          runs where        may write
 ────────────────────────  ──────────────────────────────────────────────────  ────────────────  ─────────────────────────────
 dispatcher                one CLI; parses the agent's verb, routes to models   CPU               nothing of its own
                           and tools, composes one bounded answer
 extractor model(s)        tiny open-weight models: Rust text → records          CPU or GPU        extracted · abstained
                           (items, signatures, edges) with confidence
 binder / ranker models    choose among candidate targets; order neighbourhoods  CPU or GPU        inferred (with confidence)
 labeller model            semantic labels ("retry loop", role, invariant)      CPU or GPU        annotation (opinion layer)
 store                     content-addressed records, per-snapshot, append-only  CPU               status flags (realized, stale)
 index                     trie · trigram · BM25 over records                    CPU               nothing
 find / impact / propose   the three agent-facing verbs                          CPU               propose writes on behalf of an LLM/human
 feed client               fetches signed crate graphs                           CPU               extracted (creator = feed publisher)
 teacher   (off-line)      rust-analyzer / rustc labelling the corpus            not shipped       the corpus
 judge     (off-line)      rust-analyzer / rustc scoring held-out repos          not shipped       the leaderboard
```

Models and tools call each other: the extractor asks the feed for a crate's signatures before binding a receiver; the binder asks the index for candidates; `impact` asks the ranker to order a neighbourhood; the dispatcher asks the labeller for a one-line headline per candidate. None of these calls leaves the machine.

---

## 5. Entities, defined

### 5.1 Actors
```text
 actor            instances                                      authority
 ───────────────  ─────────────────────────────────────────────  ────────────────────────────────────────────────
 TEACHER          rust-analyzer / rustc, off-line                 creates corpus labels; never writes to a user's map
 JUDGE            rust-analyzer / rustc on held-out repos         creates leaderboard scores; never writes to a user's map
 MODEL            extractor, binder, ranker, labeller (named rev) extracted · abstained · inferred · annotation
 TOOL             dispatcher, store, index, verbs, feed client     identity, snapshots, flags (realized, stale, withdrawn)
 LLM              agent-LLM, orchestrator-LLM (named)              proposed · annotation · sighting
 HUMAN            named user                                       proposed · annotation · sighting · withdraw
```
Rule: a MODEL or TOOL never writes `proposed`. An LLM or HUMAN never writes `extracted`. Nobody edits another actor's record; they add a sighting or a withdrawal that points at it.

### 5.2 Structural entities
```text
 entity        identity                                   required fields                                  creator
 ────────────  ─────────────────────────────────────────  ───────────────────────────────────────────────  ────────
 Snapshot      hash(input digests, model revs, feed revs) inputs (paths + digests), time, actor that asked  TOOL
 Crate         name@version (path crates: name@path-hash) edition, features seen, source (local | feed)     MODEL
 File          crate-relative path @ content digest        crate, size, extraction status                   TOOL
 Module        crate::path                                 file(s), visibility, doc, cfg                     MODEL
 Item          crate::module::path::Name#kind              kind ∈ {Fn, Struct, Enum, Union, Trait, TypeAlias, MODEL
                 (+ "::<Impl#n>" for impls, "::Name"         Const, Static, Macro, Impl, AssocItem, Field,
                 for assoc items / fields / variants)        Variant, ExternalItem}
                                                           signature text, visibility (declared; effective
                                                           if known), cfg, doc, span, file digest, confidence
 Edge          hash(src, dst, kind, cfg)                   src Item, dst Item | ExternalItem, kind (below),    MODEL (extracted)
                                                           cfg, evidence span, confidence, status            LLM/HUMAN (proposed)
 Proposal      ulid                                        base Snapshot, author (LLM | HUMAN), goal, set of  LLM / HUMAN
                                                           proposed Items + Edges, state (derived)
 Annotation    ulid                                        target id, text or tag, author, confidence         MODEL / LLM / HUMAN
 Sighting      ulid                                        target id, verdict ∈ {confirm, deny}, author       LLM / HUMAN
 Coverage      per Snapshot                                files in scope, files abstained whole, cfg assumed, TOOL
                                                           counts by status
 FeedEntry     crate@version @ publisher key               signed graph, licence, teacher version, first/last TOOL (publisher)
                                                           seen
```
Edge kinds, level 1: `declares_mod · uses · includes · cfg_gates`. Level 2: `signature_uses · parameter · returns · field · implements · associated_type · reexports · derives · macro_generates`. Level 3+ (on expansion): `calls · may_call · constructs · reads_field · writes_field · awaits · panics_via · unsafe_uses · influences`. The vocabulary is versioned; a kind is added by a schema release, never by a model.

Identity rule: identity never contains a span, a line, a hash of the body or a confidence. Those are fields. Two extractions of the same item from different files versions have the same identity and different records; the store keeps both under their snapshots.

### 5.3 Status of an Item or Edge, and who may move it
```text
 status       meaning                                                  set by        next states
 ───────────  ───────────────────────────────────────────────────────  ────────────  ───────────────────────────────
 extracted    MODEL produced it from real source, confidence attached  MODEL         stale (TOOL, when its file digest changes)
 abstained    MODEL saw a site and declined to resolve it              MODEL         extracted (a later snapshot resolves it)
 inferred     binder/ranker chose among candidates (confidence)         MODEL         extracted · abstained
 proposed     LLM or HUMAN asserts it should exist                      LLM / HUMAN   realized (TOOL) · withdrawn (author or HUMAN)
 realized     a proposed record now matches an extracted one           TOOL          stale, with the extracted record
 stale        the input it was extracted from has changed               TOOL          dropped from the current view, kept in history
```
Matching rule for `realized`: same identity, same kind; signature equality recorded as a flag, not required. The TOOL may set `realized`, `stale`, `withdrawn`; it may not set `extracted` or `proposed`. A MODEL may not set `proposed` or `realized`. No actor may delete a record; history is append-only.

### 5.4 Authority matrix
```text
                 Snapshot  Item/Edge extracted  Item/Edge proposed  Annotation  Sighting  Coverage  FeedEntry  status flags
 TEACHER/JUDGE      —              —                   —                —           —          —        corpus      —
 MODEL              —           create                 —             create         —          —          —         —
 TOOL             create           —                   —                —           —       create    publish    realized · stale
 LLM                —              —                create            create      create       —          —       withdraw own
 HUMAN              —              —                create            create      create       —          —       withdraw any
```

---

## 6. What this revision changes, and why
- *Verification, diff, as-built, report* — removed. They were a second mechanism for something the map already does by being re-extracted; an unnecessary complication.
- *rust-analyzer and tree-sitter at runtime* — removed. If the tiny extractor works there is no runtime use for them; if it does not, the proposal fails on its own terms and a parser-based rung sits on the same leaderboard. They remain the teacher and the judge, which is the only arrangement in which an open-weight extractor can be trusted.
- *"Zero LLM inside Parseltongue, CPU-only"* — superseded. Parseltongue is now defined as a CLI over its own tools and tiny models; CPU is the floor, GPU is allowed.
- *Who authors the future* — fixed: the tool provides the slot and never fills it. Only an LLM or a human writes `proposed`.
- *Entities* — made explicit: identity rules, required fields, creators, statuses, transitions and an authority matrix, so that "honest" is checkable in a schema rather than asserted in prose.

---

## The one risk the thesis does not remove

**Nobody writes the future half of the map.** Every earlier form of architectural intent — diagrams, architecture tests, design documents — was rejected as ceremony. Openness does not change that. The mitigation is that writing the future is one `propose` call the agent makes anyway while planning, and the extractor can lift it from the agent's own draft. Signal 2 is the test. If it fails, the product is a navigator — a utility, not a company — and the corpus, the feed and the benchmark are still worth having built. The second risk, named rather than removed: the extractor may not reach the bar. Signal 3 tests it for less than a dollar.
