# Deep Insight 05 — Parseltongue: the map an agent can read, write, and trust

*A standalone statement of the product. Answer first, then the three things that must be true, then the evidence under each, then the system and its entities. Where a number is measured it says measured. Where something is a bet it says bet.*

---

## The one-line answer

**Give every coding agent one map of a Rust codebase that holds what the code is and what the agent intends it to become — extracted by small open models from any Rust text, including text that does not compile, served through one CLI, with every edge signed by whoever produced it.**

Everything else in this document is a consequence of that sentence.

```text
   agent-LLM · orchestrator-LLM · human
          │   a seven-word question in · a 3,000-token neighbourhood out · proposed edges in
          ▼
   ┌──────────────────────────── PARSELTONGUE  (one CLI) ─────────────────────────────┐
   │  dispatcher                                                                        │
   │   ├─ tiny task-models (open weights, CPU or GPU)   extract · bind · rank · abstain  │
   │   └─ deterministic tools (ours)                    store · index · find · impact · feed │
   │  models and tools call each other; the dispatcher composes one answer               │
   └──────────────────────────────────────────────────────────────────────────────────┘
          ▲ learns from                                    ▲ is scored by
     the Rust compiler, off-line (teacher)          the Rust compiler on unseen repos (judge)
```

---

## What most people get wrong about this problem

Most people think the problem is *search*. Agents cannot find things in big codebases, so give them better search. That is the apparent problem.

The real problem is that **software has no drawing of itself.** A bridge has a blueprint, an as-built survey, and an inspector. Software has the building and nothing else. So every agent, on every task, re-derives the architecture from grep, plans in prose nobody can look up later, and edits with a picture of the system that lives only in its context window and dies when the window closes.

Search makes re-deriving faster. It does not stop the re-deriving. The thing that stops it is a map that persists, that the agent can write its intentions into, and that stays readable at the exact moment the code is torn apart.

The second thing people get wrong: they assume the map must come from the compiler at runtime. The compiler is the only source of *truth* about Rust structure — that part is correct. But a compiler needs a workspace, a build, two gigabytes and half a minute, and it degrades exactly when a refactor has broken the build. The compiler's job is to **teach** a small model what the map looks like and to **grade** that model on code it has never seen. Its job is not to be in the loop every time the agent asks a question.

---

## Three things that have to be true

```text
   1. THE OBJECT IS NEW        one map, current and future, one vocabulary, honest by construction
   2. RUST MAKES IT POSSIBLE   the only mainstream language with a ground truth to learn from and be graded against
   3. THE ORDER IS RIGHT       corpus and feed first · three verbs second · models last · three cheap kill-signals
```

---

## 1. The object is new, and nothing else is the product

### 1.1 Current and future on one vocabulary

The map has one set of node kinds and one set of edge kinds. Three things can be true of an edge:

- `extracted` — a model read real source and produced it, with a confidence.
- `abstained` — a model saw the site and declined to resolve it. Recorded, so that absence is never silent.
- `proposed` — an LLM or a human wrote it because the code *should* have it.

That is the whole mechanism for planning. There is no plan document, no checklist, no step where a tool grades the plan. The agent writes the future as proposed edges — directly, or by handing its drafted code to the same extractor that reads real code. When real source is next extracted and the same edge appears, the proposed edge is marked `realized`. Done is a region with no open proposals. The plan becomes the code by being re-read, not by being verified.

**The tool provides the slot and never fills it.** Parseltongue has no component that invents future structure. The author of record for every proposed edge is a named LLM or a named human.

### 1.2 Two levels are visible; the rest is one expansion away

- **Level 1 — file to file.** Module declarations, imports, include paths, cfg gates.
- **Level 2 — the public face of each file.** Every public signature, re-export, static and constant, cfg and feature gate, the doc comment, the item-level macro names. The membership test is one question: *could this line change what another file is allowed to write?*

Comments, imports and globals are **fields on the item record, not nodes**. An import is evidence on a dependency edge. A static is an item. A doc comment is the contract's prose, attached to the item it documents. Making them nodes buys identity churn and nothing else.

Bodies, calls, data flow, lock order, Send/Sync boundaries and semantic labels are all kept. They open on expansion. They are not the default view, because the default view is what an agent reads under a token budget.

### 1.3 Honest is a storage rule, not a promise

Every record names its creator — which model at which revision, which tool, which LLM, which human — its confidence, the digest of the input it was produced from, and the schema version. Abstention is a record. A proposed edge is never rendered as a fact about the code.

Three layers are never written into each other:

```text
   extracted structure      what a model read in the source
   shared vocabulary        node kinds, edge kinds, tags — versioned by schema release, never by a model
   opinions                 proposed edges · semantic labels · sightings ("I confirm / I deny this edge")
```

Any query can peel a layer off. This is the discipline threat-intelligence platforms arrived at for knowledge graphs fed by many sources of unequal trust — provenance on every object, sightings, warninglists for known false positives, first-seen and last-seen, feeds with distribution levels. It is what lets an opinion about the future sit next to an extracted edge without contaminating it.

### 1.4 Open means the asset is the corpus, not the binary

Four artefacts are published, and the binary is the least valuable:

1. The CLI and its versioned record schema. Apache-2.0 or MIT.
2. **A public feed of compiler-labelled graphs for every crate version.** docs.rs, but the graph.
3. **The training corpus**, with repository-disjoint splits and held-out repositories newer than any base model's cutoff.
4. The tiny models as open weights, with model card, eval harness and training recipe.

The feed changes what the extractor must know. It no longer has to memorise the standard library and a hundred crates; it fetches those facts and learns only how to connect them — bind, chain, abstain. That is the small, defensible part.

Each model is one row on a leaderboard the project owns, scored by the compiler on repositories the model never saw. If a rule-based extractor beats a model, the leaderboard says so, and the project is still the benchmark everyone cites.

---

## 2. Rust makes it possible

### 2.1 The only mainstream language with a ground truth

Names are static. Visibility is written in the source. Each trait is implemented once per type. There is no reflection and no eval. So the compiler's resolver produces the public-interface graph *exactly* — and anything that approximates it, from a frontier model to a fifty-megabyte one, can be scored against truth. No other mainstream language allows this. It is the only reason a teacher-and-judge arrangement works.

Measured on a 42k-line, eleven-crate workspace: full rust-analyzer analysis takes 29 seconds and 2.3 GB, with near-zero unknown expressions once proc macros are enabled. That is the price of truth. It is paid once per corpus release, off-line, by the teacher — not by the agent on every question.

Measured on two large public Rust codebases: 65 % of call sites are method calls, and only a quarter of their receivers can be typed from the enclosing function alone. The lesson is sharp. **The extractor cannot be a syntax-shaped trick.** It has to learn resolution, or fetch the missing signatures from the feed, or abstain. Anything that silently guesses here produces the most dangerous output a structural tool can produce: an omitted edge that looks like independence.

### 2.2 The broken moment is the real moment

The tool is for cross-cutting change. Halfway through extracting a trait, forty files fail to type-check and anything that depends on a build degrades across the whole blast radius. That is not an edge case; it is the centre of the use case.

A model that reads text does not fail to load. It reads what is there and abstains where it cannot resolve. The proposed edges say where the work is going while the extracted edges are in flux. Current and future live on the same object, extracted by the same model from real code and from drafts, so the agent keeps navigating by the plan while the code catches up.

### 2.3 The absolute claim is false; the combination claim is true

Each piece exists somewhere: module graphs, public-API listings, symbol navigation over LSP, repo maps ranked into a token budget. Claiming "nobody can do any of this" would be wrong and would be caught in the first comment thread.

What exists nowhere: a build-free, open-weight extractor of Rust structure, plus budgeted agent queries, plus proposed edges on the same map, plus per-record provenance. A map the agent can write the future into. A compiler-labelled Rust graph corpus. A per-crate graph feed. Those absences are verifiable, and the last two are cheap to fill first.

### 2.4 Who believes it, and what it takes

```text
   convinced by openness alone
      rust-analyzer contributor    we consume their output off-line and publish the scores; nothing is wrapped
      CTO of a Rust team           one binary, no egress, no build step in the agent's loop, source auditable
      ML researcher                citable repo-disjoint data with the compiler as judge
      the "weekend project" skeptic there is nothing to wrap; the extractor is ours and graded in public

   convinced only by engineering
      maintainer of 300k LOC       proc macros handled, re-extraction in seconds, one impact answer grep gets wrong
      agent-harness builder        an A/B in their own logs, not our benchmark
      newcomer in an AI editor     one command — "where do I even start" — on broken work-in-progress, zero config

   created by openness
      crate author                 "my crate is in your feed" → licence passed through, a correction channel
      contributor                  "where is the schema?" → documented before the first release, or forks
```

### 2.5 The negatives are already on the record

Frontier models asked to draw signature dependencies omitted six of fourteen. Agents working from source alone found ten of nineteen local edges. A learned binding model accepted zero bindings where a rule accepted fifty-six.

This is why the extractor ships with its leaderboard row visible, abstention on, and the corpus published first. *If the tiny model works* is the bet of this proposal. It is tested for less than a dollar, below.

---

## 3. The order of work

### 3.1 Three signals, each cheap, each able to kill a branch

```text
 1  three-arm pilot · three unseen repos · equal token budget
      baseline agent  vs  agent + map from a rule extractor  vs  agent + map from the tiny model
      if the first two tie on correctness → navigation is a feature, not a product            ~1 week · no training

 2  future-map authoring on real work
      do agents and maintainers keep writing proposed edges after week one, unprompted?
      if not → the future half of the map is dead; the product is a navigator                  one design cycle

 3  extractor ladder, second rung · 10 M tokens
      held-out score against the rule extractor at a fixed wrong-edge budget
      flat or below → keep rules, stop training                                                one afternoon · < $1
```

### 3.2 Three verbs, not a protocol

- **`find`** — seven words in; four candidate entities out, each with its public anchor, paths, blast radius and a bounded neighbourhood under a token budget. The retrieval machinery is internal.
- **`impact`** — callers, implementors, constructors, cfg-gated sites, with file and line, and the coverage envelope printed on every answer. Proposed edges appear in the same answer, marked.
- **`propose`** — writes proposed nodes and edges on behalf of a named LLM or human, with a reason. The tool validates shape and identity. It never originates content.

`map` (extract real source or a draft) and `feed` (fetch published crate graphs) exist, but the agent does not see them; the dispatcher does.

### 3.3 The ninety-second proof

```text
  0:00  map       banner — crates · files · public items · % extracted at confidence ≥ 0.9 · % abstained and why · model + feed versions
  0:10  find      "where do retries get decided"  →  four candidates · picks one · neighbourhood under budget · blast radius
  0:25  propose   the agent drafts a trait sketch; the extractor reads the draft; three edges enter the map as proposed, dashed
  0:40  edit      halfway, the crate does not compile — nothing in the loop needed it to
  0:50  impact    still answers: surviving edges · 9 newly abstained · the 3 proposed edges still showing the way
  1:10  edit      re-extraction: 2 proposed edges realized · 1 still open — the agent sees what is left
  1:25  map       the last dashed edge turns solid; the region has no open proposals; the plan simply became the code
```

Where this demo dies first: item identity across rename and move, and re-extraction latency on every save. Where the proposal dies: extractor accuracy. Those are the three engineering problems, in that order.

### 3.4 Publish the corpus and the feed before any product or model claim

Permissive-licence sources only; repository, commit, licence and teacher version on every record. Signed feeds with distribution levels and warninglists. A last-seen stamp on every record so staleness is shown, never hidden. Semantic labels written by a large model live in their own layer with their own licence, checked before publication.

### 3.5 Non-goals — stated so they stay non-goals

- **No verification step, diff, as-built map, conformance report or CI gate.** The future is the part of the map not yet realized; realization is observed by extraction, not judged by a tool.
- **No compiler, rust-analyzer, tree-sitter or cargo at runtime.** They are teacher and judge. A build is never required to answer the agent.
- **No tool-authored future.** The proposed slot is a structure; only an LLM or a human fills it.
- **No comment, import or global as a graph node.**
- **No "nothing else can do this."** The claim is that nobody ships the combination, and nobody publishes the corpus or the feed.
- **No reception table treated as market evidence.** The market evidence is the three signals.

---

## 4. The system, defined

```text
 component                what it is                                                 runs on     may write
 ───────────────────────  ─────────────────────────────────────────────────────────  ──────────  ──────────────────────────────
 dispatcher               one CLI; parses the agent's verb, routes, composes answer    CPU         nothing of its own
 extractor model(s)       tiny open-weight models: Rust text → items, signatures,      CPU / GPU   extracted · abstained
                          edges, each with confidence
 binder · ranker          choose among candidate targets; order a neighbourhood        CPU / GPU   inferred (with confidence)
 labeller                 one-line semantic headlines ("the retry loop"), roles        CPU / GPU   annotation (opinion layer)
 store                    content-addressed, per-snapshot, append-only                 CPU         status flags (realized · stale)
 index                    trie · trigram · BM25 over records                           CPU         nothing
 find · impact · propose  the three agent-facing verbs                                 CPU         propose writes on behalf of LLM/human
 feed client              fetches signed crate graphs                                  CPU         extracted (creator = publisher)
 teacher  (off-line)      the Rust compiler labelling the corpus                       not shipped the corpus
 judge    (off-line)      the Rust compiler scoring held-out repositories              not shipped the leaderboard
```

Models and tools call each other. The extractor asks the feed for a crate's signatures before binding a receiver. The binder asks the index for candidates. `impact` asks the ranker to order a neighbourhood. The dispatcher asks the labeller for a headline per candidate. None of these calls leaves the machine.

---

## 5. The entities, defined

### 5.1 Actors and their authority

```text
 actor     instances                                           may create
 ────────  ─────────────────────────────────────────────────  ───────────────────────────────────────────────
 TEACHER   the compiler, off-line                              corpus labels — never a record in a user's map
 JUDGE     the compiler on held-out repositories               leaderboard scores — never a record in a user's map
 MODEL     extractor · binder · ranker · labeller (named rev)  extracted · abstained · inferred · annotation
 TOOL      dispatcher · store · index · verbs · feed client     snapshots · identities · flags (realized · stale · withdrawn)
 LLM       agent-LLM · orchestrator-LLM (named)                 proposed · annotation · sighting
 HUMAN     a named user                                         proposed · annotation · sighting · withdrawal
```

Three rules. A MODEL or a TOOL never writes `proposed`. An LLM or a HUMAN never writes `extracted`. Nobody edits another actor's record — they add a sighting or a withdrawal that points at it.

### 5.2 Structural entities

```text
 entity       identity                                      required fields                                        creator
 ───────────  ────────────────────────────────────────────  ─────────────────────────────────────────────────────  ───────────
 Snapshot     hash(input digests · model revs · feed revs)  inputs with digests · time · requesting actor           TOOL
 Crate        name@version  (path crates: name@path-hash)   edition · features seen · source ∈ {local, feed}        MODEL
 File         crate-relative path @ content digest          crate · size · extraction status                       TOOL
 Module       crate::path                                   files · visibility · doc · cfg                          MODEL
 Item         crate::module::path::Name#kind                kind ∈ {Fn Struct Enum Union Trait TypeAlias Const      MODEL
                impls: ::<Impl#n> · assoc/field/variant:      Static Macro Impl AssocItem Field Variant ExternalItem}
                ::Name under the parent                      signature text · visibility (declared; effective if
                                                             known) · cfg · doc · span · file digest · confidence
 Edge         hash(src · dst · kind · cfg)                   src Item · dst Item|ExternalItem · kind · cfg ·         MODEL (extracted)
                                                             evidence span · confidence · status                   LLM/HUMAN (proposed)
 Proposal     ulid                                           base Snapshot · author · goal · member Items + Edges ·  LLM / HUMAN
                                                             state (derived from members, never asserted)
 Annotation   ulid                                           target id · text or tag · author · confidence          MODEL / LLM / HUMAN
 Sighting     ulid                                           target id · verdict ∈ {confirm, deny} · author          LLM / HUMAN
 Coverage     per Snapshot                                   files in scope · files abstained whole · cfg assumed ·  TOOL
                                                             counts by status
 FeedEntry    crate@version @ publisher key                  signed graph · licence · teacher version ·              TOOL (publisher)
                                                             first/last seen
```

**Edge kinds, by level.** Level 1: `declares_mod · uses · includes · cfg_gates`. Level 2: `signature_uses · parameter · returns · field · implements · associated_type · reexports · derives · macro_generates`. On expansion: `calls · may_call · constructs · reads_field · writes_field · awaits · panics_via · unsafe_uses · influences`. The vocabulary is versioned. A kind is added by a schema release, never by a model.

**Identity rule.** Identity never contains a span, a line, a body hash or a confidence. Those are fields. Two extractions of the same item from two versions of a file share an identity and have two records; the store keeps both under their snapshots.

### 5.3 Status, and who may move it

```text
 status      meaning                                                set by       may become
 ──────────  ────────────────────────────────────────────────────  ───────────  ─────────────────────────────────────
 extracted   a MODEL produced it from real source, with confidence  MODEL        stale        (TOOL, when the file digest changes)
 abstained   a MODEL saw the site and declined to resolve it        MODEL        extracted    (a later snapshot resolves it)
 inferred    binder/ranker chose among candidates, with confidence  MODEL        extracted · abstained
 proposed    an LLM or HUMAN asserts it should exist                LLM/HUMAN    realized (TOOL) · withdrawn (author, or any HUMAN)
 realized    a proposed record now matches an extracted one         TOOL         stale, together with its extracted record
 stale       the input it came from has changed                     TOOL         out of the current view; kept in history
```

Matching rule for `realized`: same identity, same kind. Signature equality is recorded as a flag, not required. No actor may delete a record. History is append-only.

### 5.4 The authority matrix

```text
             Snapshot   extracted   proposed   Annotation   Sighting   Coverage   FeedEntry   flags
 TEACHER        —           —          —           —            —          —        corpus      —
 JUDGE          —           —          —           —            —          —          —         —
 MODEL          —        create        —        create          —          —          —         —
 TOOL        create         —          —           —            —       create     publish   realized · stale
 LLM            —           —       create      create       create        —          —      withdraw own
 HUMAN          —           —       create      create       create        —          —      withdraw any
```

If a behaviour is not in this table, the system does not do it.

---

## The one risk this does not remove

**Nobody writes the future half of the map.** Every earlier form of architectural intent — diagrams, architecture tests, design documents — was rejected as ceremony, and openness does not change human nature. The mitigation is that writing the future is one `propose` call the agent makes anyway while planning, and the extractor can lift it from the agent's own draft. Signal 2 is the test. If it fails, the product is a navigator — a utility, not a company — and the corpus, the feed and the benchmark are still worth having built.

The second risk, named rather than removed: the extractor may not reach the bar. Signal 3 tests it for less than a dollar. A proposal that can be killed for a dollar in an afternoon is a proposal worth making.
