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

---

## Broad ideas — seven threads, and where I land on each

*Appended after the thesis. These are the loose ends that decide whether the thesis is buildable. Each thread: what the question is, what is actually known, and a position to argue with.*

### B1. The graph has to live somewhere

The thesis says "store." It does not say in what. The requirement is narrower than it sounds:

```text
   must                                      need not
   ─────────────────────────────────────    ──────────────────────────────────
   two levels, file and public interface     deep graph algorithms at write time
   append-only, per-snapshot records          multi-writer concurrency
   identity lookups in microseconds           a query language the agent sees
   k-hop neighbourhoods, k ≤ 3, bounded       distributed anything
   one file on disk, zero services            a daemon
```

Position: **the store is a format first and a database second.** Records are content-addressed and append-only; the on-disk truth is a log of records plus a snapshot index. The database is a *view* over that log, rebuildable at any time. That decision is what lets the backend be swapped without migrating anyone's map.

For the first backend, SQLite with recursive CTEs covers every query in the three verbs: two tables (`item`, `edge`) plus a `snapshot` table, k-hop by recursive CTE with a budget cut, trigram and FTS5 for search. It is embedded, one file, in every language's standard toolkit, and nobody will ask why.

Embedded databases with graph capability exist and should sit behind the same trait, not in front of it. The field, as of November 2026:

```text
 option                 what it is                                              embedded how            licence       status
 ─────────────────────  ──────────────────────────────────────────────────────  ──────────────────────  ────────────  ──────────────────────────
 SQLite + FTS5          relational; k-hop by recursive CTE; trigram + FTS       in-process, one file    public domain the default; nobody asks why
 Turso                  SQLite rewritten in Rust, file-compatible; WITH RECURSIVE  in-process, Rust crate  MIT           active (25k stars); crate
                        supported; FTS via Tantivy (BM25 fts_score, MATCH;      `turso`, no C, no       (open        0.9-pre on crates.io; runs
                        not FTS5); vector functions + vector index; MVCC         server; also WASM/      contribution) in production per the
                        BEGIN CONCURRENT; CDC; encryption at rest                browser                               project; the team's stated
                                                                                                                      successor to libSQL
                                                                                                                      (its C fork of SQLite)
 SurrealDB              multi-model document-graph; RELATE edges and record     in-process via the      BSL 1.1 →     active (4.0 on crates.io,
                                                                                                                      33k stars);
                        links; graph traversal (->edge->node) and full-text     Rust SDK: in-memory     Apache-2.0    not OSI-approved today —
                        in SurrealQL; vector index                              or SurrealKV engine     on a timer    fine as a dependency of an
                                                                                (kv-mem · kv-surrealkv;               Apache/MIT tool, worth saying
                                                                                RocksDB engine dropped),
                                                                                no server
 DuckDB + DuckPGQ       columnar SQL + SQL/PGQ graph pattern matching           in-process              MIT           active
 LadybugDB (Kùzu fork)  property graph, Cypher, vector + full-text              in-process              MIT           Kùzu archived Oct 2025;
                                                                                                                      fork active
 Oxigraph               RDF triple store, SPARQL — our records are triples      in-process, Rust        Apache-2.0    active; RDF feels foreign
                        already (src, kind, dst)                                                                      to most contributors
 HelixDB                graph + vector + full-text OLTP, Rust                   server-first; embed      Apache-2.0    active, young
                                                                                story to be checked
 CozoDB                 Datalog, relational-graph-vector, Rust                  in-process              MPL-2.0       quiet since late 2024
 IndraDB                graph database, Rust                                    in-process              MPL-2.0       quiet since mid-2025
 ArcadeDB · Cayley      multi-model (Java) · graph (Go)                         embeddable in their     Apache-2.0    wrong language for a Rust
                                                                                own runtimes                          CLI; listed for completeness
```

Turso is the row that changes the default. It is the same SQL and the same recursive-CTE path as SQLite, so every query in the three verbs ports unchanged, but it is a Rust crate rather than a C library behind FFI: one toolchain, one binary, cross-compiles wherever Rust does, and it runs in the browser through WebAssembly, which is where an editor-embedded Parseltongue would otherwise need a second store. Its FTS is Tantivy-backed with BM25 scoring — the retriever `find` needs, inside the store rather than beside it — and the vector functions mean a learned retriever can sit in the same file later without a new dependency. The honest caveats: the crate is pre-1.0, FTS syntax is Turso-specific (a SQLite file written by Turso with an FTS index is not readable by stock SQLite), and open contribution means the surface moves. Position: **SQLite semantics are the default; whether the engine under them is SQLite or Turso is decided by the same depth-3 benchmark plus a binary-size and cross-compile check, with Turso favoured if it ties.**

SurrealDB is the strongest graph-native candidate after the SQLite family: it is Rust, it embeds without a server, its graph edges are first-class records (an edge table with its own fields is exactly our Edge entity), and it carries full-text and vector indexes in the same engine, which would let `find`'s retrievers live in one place. Two things to say plainly: the licence is Business Source, not open source — acceptable for a dependency, not for anything we would ask contributors to fork — and its query planner's behaviour on bounded k-hop traversals under a token budget is the thing to measure, not assume. Each option is worth exactly one benchmark — blast radius at depth 3 over a 10k-item crate under a token budget — and none is worth a dependency before that number exists.

### B2. Shrink a big model down, or grow a small one up?

The question is whether the small model should start as a *distilled and quantised* big model rather than a small model trained from scratch, because the big model already carries an approximate understanding of Rust that a from-scratch model would have to re-learn.

Yes — with one correction about what "shrink" can mean.

- **Quantisation alone does not get there.** Four-bit weights make a model four times smaller than sixteen-bit, not fifty times. A 7B model at four bits is still ~4 GB. Quantisation is the last step, never the strategy.
- **Pruning plus distillation does work, but by a factor of two to four per stage, not fifty.** The published pattern is depth- and width-pruning a parent model and distilling the pruned child on a small fraction of the parent's original training tokens; small production models of 1–3B parameters have been made this way from 8B parents. The child keeps most of the parent's capability at a fraction of the cost of training it from nothing.
- **Logit distillation from a large teacher is cheap and composable.** The large model's distribution over our corpus becomes a soft target alongside the compiler's hard labels. The compiler says *what* the edge is; the big model's distribution says *how a Rust-literate reader hesitates*, which is exactly what abstention needs.

So the realistic path is not "take a 70B model and nuff it down to 100 MB." It is: **start from the smallest existing code-pretrained model with the Rust prior already in it — the 0.5B–1.5B class — distil onto our compiler-labelled corpus with the big model's logits as a second teacher, then quantise.** That lands at roughly 300 MB to 1 GB on disk. Training from scratch is only cheaper if the task is narrow enough to be an encoder with heads, which is the binding-selection shape, not the extraction shape.

The position, bluntly: a model with the Rust prior and 1 GB beats a model without it at 100 MB, for this task, every time. Which forces B3.

### B3. Where the CPU stops being enough

Two facts settle most of the argument.

First, token generation is memory-bandwidth-bound: each generated token reads every weight once. Tokens per second is, to a first approximation, memory bandwidth divided by model bytes. Second, *prefill* — reading the source into the model — is compute-bound, and an extractor spends almost all of its time in prefill, because it reads thousands of tokens of Rust and emits a few hundred tokens of records.

```text
   model on disk    laptop CPU ≈ 60 GB/s        Apple-class unified ≈ 200–400 GB/s    consumer GPU ≈ 500–1000 GB/s
   ─────────────    ─────────────────────────   ──────────────────────────────────    ─────────────────────────────
   100 MB           bandwidth says 600 tok/s;   fine                                  overkill
                    compute caps it lower
   500 MB           ~100 tok/s                  ~500 tok/s                            1,000+ tok/s, and batching
   1 GB             ~50 tok/s                   ~250 tok/s                            1,000+ tok/s, and batching
   1 GB, prefill    a 4k-token file takes        seconds                               well under a second
                    tens of seconds
```

These are envelope numbers, not measurements; the measurement is a one-afternoon job once a candidate model exists. But the shape is not in doubt:

- Up to ~100 MB, a CPU is fine for generation and a GPU buys little. This is the encoder-with-heads regime.
- At 500 MB to 1 GB — the regime B2 says the extractor lives in — a CPU still *works* for the interactive verbs, because `find` and `impact` do not run the extractor at all; they read the store. The extractor runs on save and on `propose`. That is where a CPU is slow and a GPU is fast.
- Whole-workspace extraction — a million lines — is a batch job. On a CPU it is lunch; on a 1 GB-VRAM GPU it is minutes.

Position: **lean into the GPU for extraction, keep the CPU as the floor for the verbs.** A 1 GB VRAM budget is the right target: it fits a 1.5B-parameter model at four bits with headroom for activations, it exists on every laptop with a discrete GPU and on every Apple machine via unified memory, and it does not exclude anyone — the CPU path produces the same records, just slower. The earlier 100 MB peak-RSS gate stays as the gate for the *verbs and the index*, not for the extractor.

### B4. The CLI stores both levels, and the database is not the API

The agent never sees the database. It sees three verbs. The store exposes one internal trait:

```text
   put(records) → snapshot id
   get(id) → record
   neighbours(id, kinds, depth, budget) → bounded subgraph
   search(text) → candidate ids
   proposals(region) → open proposals
```

Both levels are the same tables: a level-1 edge is `uses`/`declares_mod` between file-scoped items; a level-2 edge is `signature_uses`/`implements`/`reexports` between public items. The level is a filter, not a schema.

### B5. Search: what FFF gets right, and what it does not do

FFF (dmtrKovalenko/fff) is a Rust file-search engine built for AI agents and editors — fuzzy path matching with frecency and git signals, ranked for "the file the user means," and shipped as an SDK across languages. It is the right reference for the *file-level* half of `find`: it has solved ranking files the way humans and agents actually name them, and it is fast.

What it does not do is the item level. Our hit is an item, not a file: `auth::session::create#fn`, with its public anchor, its folder chain and its blast radius. Position: **adopt FFF — or its ranking ideas — for the file retriever, and keep our own item retriever**: symbol trie for exact and prefix, trigram for typos, BM25 over sub-tokens of name, path and first doc line, fused by reciprocal rank. Both feed the same candidate list. The important property is that every candidate is a real id from the index, so nothing can be invented; FFF has the same property at file granularity, which is why it belongs.

### B6. Queryable means "zero in," and the workflow is already written

The workflow is seven events. It is the inside of `find`, restated here so this document stands alone:

```text
   1  the agent asks in ≤ 7 words                  "where do retries get decided"
   2  four closest entities                         real ids from trie + trigram + BM25, fused
   3  for each: the encapsulating PUBLIC interface  BFS upward until visibility ≥ crate boundary;
      + folder chain + blast radius 1                record the via chain
   4  curtail each to ≤ 3,000 tokens                (default: 80-token headlines; expand on demand)
   5  show the four, ask which to open
   6  the agent picks one · says none (page 2) · quits
   7  deep dive ≤ 20,000 tokens                     bounded subgraph to depth 3, full source of the anchor
                                                    and its top callers, control-flow and data-flow highlights,
                                                    cfg context, open proposals in the region, and
                                                    grounded example queries for the next step
```

"Zero in" is event 3. A hit inside a private function is worthless until the agent knows the public thing that encloses it and what breaks if it changes. That is the one answer grep cannot give and the one answer the agent asks for next anyway. Position: **the encapsulating public interface, the folder chain and blast radius 1 are returned with every hit, always, before anything else.** This is the core of what a codebase MCP should be, and it is cheap once the two levels exist.

### B7. A conversation, not a query

The agent and the tool talk. Three things follow.

- **The session has state.** Candidates shown, cluster chosen, budget spent, proposals open. Event 6 is meaningless without it.
- **The tool may ask back.** "Did you mean the trait or the impl?" "I abstained on nine receivers in this crate — the feed has signatures for `tokio@1.38`; fetch?" "Three open proposals touch this region; show them?" A tool that can ask is a tool that can stay honest under a budget instead of guessing.
- **One wire format, two carriers.** JSON lines over stdin/stdout for any orchestrator; an MCP adapter over the same messages for editors. The verbs are the vocabulary; the session is the grammar; the dispatcher is the speaker.

What the conversation is *not*: the tool does not plan, does not write code, does not decide what the future should be. It answers, it asks when it must, and it records what the LLM or the human decides. The authority matrix above is the whole contract, and the conversation never exceeds it.

### The decisions these seven force

```text
   store         record log is truth · SQLite-semantics view first (SQLite or Turso, by benchmark) · SurrealDB the graph-native candidate · others behind a trait
   model         start from a 0.5–1.5B code model · distil on the compiler corpus with big-model logits · quantise last
   hardware      1 GB VRAM target for extraction · CPU floor for the verbs · 100 MB RSS gate for index + verbs only
   search        FFF-class file retriever + own item retriever · every candidate is a real id
   zero-in       public anchor + folder chain + blast radius 1 on every hit, always
   conversation  stateful session · the tool may ask · JSON lines + MCP adapter over the same messages
```
