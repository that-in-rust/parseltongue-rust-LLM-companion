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

### B8. The 10-million-line prediction — tiny model on a GPU versus rust-analyzer

*A prediction, not a measurement. Inputs: one measured point for rust-analyzer (42k lines, eleven crates: 29 s cold, 2.3 GB); the bandwidth and compute rules from B3; a 500 MB extractor (≈ 0.5B parameters at 8-bit, or 1B at 4-bit) running on one consumer GPU with 1 GB of VRAM budget. Ten million lines ≈ 100 million tokens of Rust.*

The question people ask is "which one maps the codebase faster." That is one race out of three, and the least important one.

```text
                              rust-analyzer                             tiny model on one consumer GPU
 ───────────────────────────  ───────────────────────────────────────  ───────────────────────────────────────────────
 1  FULL COLD MAP             ~1–2 h if it fits in one process;         prefill: 2 × 0.5e9 × 1e8 ≈ 1e17 FLOPs → ~10 min
    (once per corpus,         realistically sharded per crate across    decode: ~20 M record tokens, batched → ~20–60 min
     once per clone)          many cores; tens of GB resident; a 10 M-  total ~30–70 min; scales linearly with more GPUs
                              line single workspace is outside its      ─────────────────────────────────────────────
                              design envelope                           VERDICT  tie, to 2× faster.  not the win.

 2  RE-MAP AFTER ONE EDIT     salsa is incremental, but a cross-crate    only the changed file is re-read: a 500-line file
    (a thousand times a day)  "all references" re-type-checks every     is ~5k tokens → ~1 s.  every other verb reads the
                              body that could see the change →          persisted store → milliseconds
                              seconds to minutes at this scale          ─────────────────────────────────────────────
                                                                        VERDICT  model wins, 10–100×.  this is the win.

 3  SESSION START             no persisted index: the whole workspace    the store is on disk; nothing to load; mmap
    (every editor / agent     is re-derived every start → tens of        the records and answer
     launch)                  minutes at 10 M lines                      ─────────────────────────────────────────────
                                                                        VERDICT  model wins by the entire cold-map time

    MEMORY                    tens of GB RAM                             ~1 GB VRAM + a store on disk
    BROKEN / NO CARGO /       degrades, or will not load                 reads it
    SNIPPET / DRAFT
    CORRECTNESS               exact                                      lossy; abstains; scored by the compiler off-line
```

**The prediction in one line: for a one-shot full map of ten million lines the tiny model on a GPU is roughly as fast as rust-analyzer — within a factor of two either way — and that race does not matter. The costs that dominate a codebase this size are the per-edit re-map and the per-session reload, and there a model plus a persisted record store wins by one to two orders of magnitude, because it touches one file and never re-derives.**

The architectural reason, stated once: rust-analyzer *computes* the graph on demand from source, every session, and must re-compute whatever a change could invalidate. Parseltongue *stores* the graph as records and re-extracts only the file whose digest changed. One is a function; the other is a database with a function in front of it. At 42k lines the difference is a 29-second annoyance. At ten million it is the product.

Two things would break the prediction:

1. **If a changed file forces re-extraction of its dependants** — because the model needs their bodies to re-bind receivers — then per-edit cost grows with blast radius and regime 2 collapses toward rust-analyzer's. The feed (external signatures fetched, not inferred), the level-2 records of local items (already in the store), and abstention (say so, do not re-read) are the three mechanisms that keep the unit of work at one file. If they fail, so does the prediction.
2. **If rust-analyzer ships a persisted on-disk index**, regime 3 closes and only regimes 2 and memory remain. That is a plausible development and would be good for everyone; the proposal should not depend on it never happening.

On a CPU, for completeness: regime 1 becomes one to two days (prefill is compute-bound and a laptop CPU is roughly two orders of magnitude short of a GPU), regime 2 becomes 10–30 s per changed file, regime 3 is unchanged. The verbs never run the model, so a CPU-only user keeps every advantage except extraction speed. That is why B3 says lean into the GPU for extraction and keep the CPU as the floor for everything else.

Confidence: medium on the shape of the three regimes; low on every number until the one-afternoon measurement in B3 exists. The cheapest way to raise it: extract one 42k-line workspace with a 0.5B code model on one GPU and put the wall-clock next to the 29 seconds.

#### B8.1 The same prediction on a Mac mini

*Apple Silicon changes two inputs — compute and bandwidth are lower, memory is unified and larger — and one conclusion.*

```text
 Mac mini M4 (base)    10-core GPU · ~120 GB/s · 16–32 GB unified · ~4 TFLOPS-class
 Mac mini M4 Pro       20-core GPU · ~273 GB/s · up to 64 GB unified · ~10 TFLOPS-class
 consumer discrete GPU ~1 TB/s · 100+ TFLOPS-class (FP16)   → the mini has 10–30× less compute, 3–8× less bandwidth

                              rust-analyzer on the mini                 500 MB model on the mini's GPU (Metal)
 ───────────────────────────  ────────────────────────────────────────  ─────────────────────────────────────────────────
 1  FULL COLD MAP, 10 M LOC   tens of GB resident → does not fit a       prefill 1e17 FLOPs at ~5 TFLOPS effective ≈ 5–6 h
                              16 or 24 GB mini; borderline at 64 GB;     decode of ~20 M record tokens, batched ≈ 1–3 h
                              ~1–2 h if it fits, sharded per crate       M4 Pro ≈ 7–9 h · base M4 ≈ 15–20 h — overnight
                                                                         VERDICT  rust-analyzer ~5× faster — if it fits at all
 2  RE-MAP ONE EDITED FILE    seconds to minutes                         5k-token file: ~1 s prefill + ~0.5 s decode ≈ 2 s
                                                                         VERDICT  model wins, 10–50×
 3  SESSION START             tens of minutes                            zero; the store is on disk
                                                                         VERDICT  model wins outright
    MEMORY                    the binding constraint on a mini           model 0.5–2 GB + store; the agent gets the rest
```

What flips: on a Mac mini the cold map is an overnight batch job, not a tie — and it does not matter, because it happens once per clone and the mini is the machine that is always on. What the mini adds: unified memory removes the VRAM ceiling, so a 1.5–3B model at 4-bit (1–2 GB) costs nothing on a 24 GB machine. B2's "the bigger model with the Rust prior" becomes the default there — a longer one-time map traded for better edges on every edit after.

rust-analyzer is memory-bound on the same machine. At ten million lines it needs the 64 GB configuration or per-crate sharding to run at all; the model needs 1–2 GB wherever it runs.

Practical: the GPU path is llama.cpp over Metal or MLX. The Neural Engine is not usable for this through open tooling today; ignore the TOPS figure. The deployment shape this suggests is the obvious one — the mini holds the store and runs extraction on every push; laptops and agents read. Confidence as above. The B3 afternoon measurement on this hardware is literally: extract the 42k-line workspace on the mini and put the wall-clock next to 29 seconds.

#### B8.2 Why reading is not free, and the four ways to make it so

*The natural objection to B8: "we read ten million lines and output five percent of it — why is inference slow at all?" The answer is the single most useful fact about putting a model in the loop.*

In a transformer, reading costs exactly as much compute per token as writing. The intuition that reading is cheap comes from grep, where a byte costs about ten operations. For a 0.5B-parameter model a token costs about a billion multiply-adds, because every input token is pushed through every weight.

```text
   grep           reads 10 M lines        ~10 ops per byte                  → about a second
   0.5B model     reads 100 M tokens      ~1e9 ops per token (2 × params)   → 1e17 ops — the prefill number in B8
                                           100-TFLOPS GPU ≈ 10 min · Mac mini ≈ 5 h · laptop CPU ≈ a day

   output         ~20 M record tokens — a fifth of the input, not a twentieth; records are verbose —
                  costs the SAME ops per token, plus a second tax: tokens come out one at a time, and each one
                  re-reads the weights. Decode is bandwidth-bound unless batched. That is why a fifth of the
                  tokens shows up as 20–60 minutes.
```

The slowness is not a flaw in the estimate. It is the price of a neural network on every token. Which points directly at the four ways to make it cheap — all of them design decisions, none of them hardware.

```text
 1  DON'T DECODE — CLASSIFY        an encoder, not a generator. One forward pass per file; records are read off
                                   tagged spans and their pairings. Output cost → ~0. The cold map becomes prefill
                                   only. This is the binding-helper shape, scaled up to extraction.

 2  DON'T READ BODIES FOR L1 / L2  mod · use · pub signatures · cfg · doc comments are ~30 % of a crate's tokens.
                                   The two visible levels need no bodies. 100 M → ~30 M tokens. Bodies are read
                                   lazily, per region, on expansion.

 3  CASCADE — RULES FIRST          `use a::b::C;` and `pub fn f(x: T) -> U` are regular grammar. A deterministic
                                   scanner of our own handles them and the model sees only the ambiguous sites —
                                   method receivers, macro output, glob imports, trait dispatch. Another 3–5× fewer
                                   model tokens, and the model's task gets narrower, which is the same thing as
                                   easier to train.

 4  SHORT HANDLES, NOT PATHS       records refer to items by index id, not by the full
                                   `crate::module::path::Name#kind` string. Output tokens ÷ 3–5.
```

Applied together:

```text
   L1 + L2 map of 10 M lines   ≈ 10–20 M model tokens · encoder pass · no decode
                                 discrete GPU ≈ 2 min · Mac mini ≈ 1 h · CPU ≈ a few hours
   L3+ (bodies, calls, flow)    stays the overnight job — and is done lazily, per region the agent actually opens
```

One caveat, stated so it is not mistaken for a reversal. Point 3 puts a deterministic pass in front of the model. That is not tree-sitter and not rust-analyzer; it is our own scanner over Rust tokens for the parts of the language where there is nothing to learn. The model earns its place on the 65 % of call sites that are method calls, not on `use` lines. A model that is asked to re-derive `use std::collections::HashMap;` is being paid to do grep's job at a billion times the cost.

The corollary for B2 and B3: once the model only sees the hard sites, the token count drops by an order of magnitude and the argument for the bigger model with the Rust prior gets stronger, not weaker — the same budget buys a better reader of fewer tokens.

### B9. What size of model — meaningful classification, faster than rust-analyzer, in less peak RAM

*The question has three constraints. They do not pull in the same direction, so each is taken alone and then combined. Baseline: rust-analyzer at 42k lines — 29 s cold, 2.3 GB peak, ≈ 1,500 lines/s — where the 2.3 GB is mostly a fixed cost (std and dependencies) that grows with the workspace.*

**RAM is the easy axis.** A model's footprint is constant; rust-analyzer's grows with the codebase. At 8-bit, weights on disk ≈ parameters in megabytes; peak RSS ≈ weights plus 100–200 MB of activations and runtime. Anything up to ~1.5B parameters at 4-bit (≈ 0.8 GB) beats 2.3 GB on every codebase, and the gap widens with size.

**Speed is the hard axis — and model size matters less than the cascade.** To match 1,500 lines/s the model must process ≈ 15k tokens of source per second. With rules handling level 1 and level 2 and the model seeing only the ambiguous sites — 10–20 % of tokens — the model needs 2–3k tokens/s. Encoder, one pass, no decode.

```text
 params          int8 disk    peak RSS      CPU tok/s          Mac mini GPU   discrete GPU   beats RA on cold-map speed?
                                            (~200 GFLOPS eff.)
 ─────────────   ──────────   ───────────   ────────────────   ────────────   ────────────   ────────────────────────────
 ~20–40M         20–40 MB     100–150 MB    3,000–5,000        50k+           500k+          yes, even on CPU
 ~100–150M       100–150 MB   300–400 MB    700–1,000          ~20k           200k+          GPU yes · CPU roughly a tie
 ~400M           400 MB       ~0.7 GB       ~250               ~6k            60k+           GPU yes · CPU no
 0.5–1.5B        0.5–0.8 GB   1–1.2 GB      decode-bound,      ~500           5k+            GPU tie · CPU no
 (generative)                               50–100
```

**"Meaningful" is the judgment call.** After the cascade the model's jobs are: pick the receiver's type among k candidates, pick the target of a method call, decide macro and glob cases, abstain. The published pattern for code encoders is that below ~30M parameters they stop carrying type-like signal; our own 17 MB encoder with trained heads ran under 100 MB RSS, and its learned variant accepted zero bindings — consistent with "too small to learn it", on thin evidence. The first tier where code-understanding tasks are reliably useful is the 100–150M encoder class.

The prediction:

```text
 sweet spot   100–150M encoder · int8 · ~150 MB on disk · ≤ 400 MB peak RSS · rules-first cascade in front
              → ~6× less RAM than RA on a 42k-line workspace, more on larger ones
              → faster than RA's cold map on any GPU, including a Mac mini; roughly a tie on CPU
              → wins outright on per-edit and session start, where RA re-derives and the store does not
 floor        ~30M, if it must also win on CPU cold-map speed; expect weaker binding and more abstention
 ceiling      ~400M, past which a CPU can no longer keep pace with RA's cold map; still 3× less RAM
 note         the 500 MB generative model is a GPU answer and a speed tie — it buys the Rust prior, not speed
```

One tension, stated: a 100 MB peak-RSS gate admits only the ~30M class. If meaningful classification needs 100M or more, the gate for the extractor should be about 400 MB; the 100 MB gate can stand for the index and the verbs, which never run the model.

Confidence: medium on the tiers, low on the exact crossovers. The second rung of the training ladder — ten million tokens, one afternoon — run at 30M and at 125M, turns this prediction into a measurement.

#### B9.1 The 2 GB GPU, and the Linux kernel as the yardstick

*Two anchors added to B9. First: the model size for a post-trained extractor of file-level and public-interface-level dependencies that must run really fast on a 2 GB GPU. Second: the Linux kernel, pulled and counted, as the real "very large codebase" the earlier ten-million-line scenario stood in for.*

**The 2 GB answer: a 100–150M-parameter encoder at int8 (~150 MB) if the post-trained model tags spans; a 0.5B-class decoder at 4–8-bit (300–500 MB) if it must generate records. Nothing above ~0.5B is "really fast" on a 2 GB card.**

The budget is smaller than it looks:

```text
 2 GB VRAM card (GTX 1050 / MX-class laptop GPU ≈ 2–4 TFLOPS FP16, 100–200 GB/s)
   − CUDA / driver context                         ~300–500 MB   the tax nobody budgets for
   − runtime, tokenizer, buffers                    ~100 MB
   = usable for weights + activations / KV cache    ~1.3–1.5 GB

 model                  weights     activations / KV (4k ctx)          fits?    L1/L2 speed on this card
 ─────────────────────  ──────────  ────────────────────────────────   ──────   ─────────────────────────────────────
 encoder 30M   int8      30 MB      ~100 MB at batch 32                 yes      ~40k tok/s   overkill; weaker binding
 encoder 125M  int8     125 MB      ~300 MB at batch 32                 yes      ~10k tok/s   ← the sweet spot
 encoder 400M  int8     400 MB      ~600 MB at batch 16                 yes      ~3k tok/s
 decoder 0.5B  Q8       500 MB      KV ~12 KB/tok → 50 MB/seq, batch 8  yes      prefill ~3k tok/s · decode 300–500 tok/s
 decoder 1.5B  Q4      ~900 MB      KV ~28 KB/tok → 110 MB/seq, batch 2 barely   prefill ~1k tok/s · decode ~150 tok/s — not fast
 decoder 3B    Q4      ~1.8 GB      —                                   no
```

What "really fast" buys at the sweet spot, for the two visible levels only — bodies skipped (~30 % of tokens remain), rules handling `use`, `mod` and plain signatures so the model sees 10–20 % of the source:

```text
 42k-line workspace    ~400k tokens → 60–120k model tokens → ~10 s on the 125M encoder    (rust-analyzer: 29 s, 2.3 GB)
 one edited file       ~5k tokens → well under a second
 10 M lines            ~1 h as a batch
 peak memory           < 1 GB total, constant with codebase size
```

Two caveats. Post-training a *decoder* to emit records is the natural thing to do with an existing code LLM, and 0.5B is the ceiling at which it stays fast on 2 GB; the encoder route is faster and smaller but means training heads for span-tagging and pairing rather than prompting. And a 2 GB card is bandwidth-poor, so decode-heavy designs degrade first — the B8.2 rules (classify, do not decode; short handles) matter more here than on a large GPU.

**The yardstick.** torvalds/linux at `d48723711` (2026-11-06), shallow clone, counted with tokei 12.1.2:

```text
 language          code lines    comments     blanks     files
 C                 20,146,151   2,998,278  3,887,502    37,763
 C header           8,713,020   1,661,436    803,525    27,281
 Device Tree        1,829,191      96,190    301,905     6,642
 Rust                 132,651      16,341     18,201       568     rust/ tree 131k; the rest in drivers/ and samples/
 all languages     33,386,417   4,999,539  5,465,271    90,959     43.9 M lines including comments and blanks

 C + headers + Rust source    1.49 GB  ≈ 370–430 M tokens  ≈ 39 bytes per line  ≈ 10–11 tokens per line
 public-face proxies          27,307 header files · 39,432 EXPORT_SYMBOL* · 5,669 Rust pub items
```

Three things the count settles:

1. **The ten-million-line scenario was calibrated correctly.** 10–11 tokens per line measured; "10 M lines ≈ 100 M tokens" holds. The kernel is about four times that scenario — ~38 M lines of C, headers and Rust, ~400 M tokens. Every cold-map figure in B8 and B8.1 multiplies by four; per-edit and session-start figures do not move, which is the point of B8.
2. **If the kernel were Rust** — it is not; Parseltongue is Rust-only, and the kernel's actual Rust is 133k lines, which rust-analyzer handles in about a minute — the two-level map with the cascade puts 40–80 M tokens through the model: **125M encoder on a 2 GB card ≈ 1–2 h · discrete GPU ≈ 5 min · Mac mini ≈ 20–40 min.** The full body-level map with a 0.5B generative model: discrete GPU 2–5 h, Mac mini 30 h or more, 2 GB card not viable.
3. **The public face is tiny relative to the body.** 27k headers and 39k exported symbols against 29 M lines: the level-2 layer is a fraction of a percent of the items by count. That is why the two visible levels are cheap to extract and to hold in memory, and why bodies must be read lazily or not at all.

#### B9.2 B9.1 explained slowly, in nine steps

*The same material as B9.1, walked through for a reader who does not carry the arithmetic in their head.*

**Step 1 — what the model has to do.** Read Rust text, output records: "file A uses file B", "`pub fn foo(x: T) -> U` lives in module m". Levels 1 and 2 only — no function bodies. Two ways a model can do that:

```text
 ENCODER  reads the text once and paints a label on each token ("this is a type name", "this pairs with that
          import"). Records are read off the labels. One pass, nothing generated. A highlighter.
 DECODER  reads the text, then WRITES the records out token by token, the way a chat model types. Hundreds of
          extra steps per file.
```

**Step 2 — why the GPU's 2 GB is really ~1.4 GB.** Before any model loads, the GPU driver reserves 300–500 MB just to exist. Add ~100 MB for the runtime, tokenizer and scratch buffers. What remains for the model is 1.3–1.5 GB. A model that "fits in 2 GB" on paper does not.

**Step 3 — how much a model weighs.** At 8-bit, one parameter is one byte: 125M parameters = 125 MB, 0.5B = 500 MB; at 4-bit, half. Weights are half the story — the model also needs working memory while it runs:

```text
 encoder   activations — scratch proportional to (batch × text length); ~300 MB at 32 files at once
 decoder   the KV cache — it must remember every token read so far to write the next one; ~12 KB per token for a
           0.5B model → a 4k-token file holds ~50 MB in flight; eight files at once = 400 MB. This is what pushes
           decoders out of 2 GB first.
```

**Step 4 — how fast the card is.** A 2 GB card does 2–4 trillion operations a second. Reading one token through a model costs 2 × parameters operations:

```text
 125M encoder   2 × 125M = 250M ops/token  → ~3 trillion ÷ 250M ≈ 10,000 tokens/s
 0.5B decoder   2 × 500M = 1B ops/token    → ~3,000 tokens/s reading (prefill)
                writing is one token per step, and each step re-reads all 500 MB of weights from memory;
                the card moves ~150 GB/s → 150 ÷ 0.5 ≈ 300 tokens/s writing
```

That last line is why "really fast" ends at 0.5B on this card: writing is bandwidth-bound, and the card is bandwidth-poor.

**Step 5 — steps 2–4 together are the table.** 30M encoder: fits trivially, 40k tok/s, too small to learn type-like decisions. 125M: fits, 10k tok/s, large enough to be useful — the sweet spot. 400M: fits, three times slower. 0.5B decoder: fits with eight files in flight, reads fast, writes slowly. 1.5B at 4-bit: 900 MB of weights plus KV — two files in flight, 150 tok/s writing — "fits" but is not fast. 3B: does not fit.

**Step 6 — how much text the model actually sees.** A 42k-line workspace is ~400k tokens, but:

```text
 100 %   all tokens
  30 %   after dropping function bodies — levels 1–2 do not need them
 10–20 % after a rule scanner handles `use …;`, `mod …;` and plain signatures — regular grammar, nothing to learn
```

The model sees 60–120k tokens; at 10k tok/s that is ~10 s. rust-analyzer on the same workspace: 29 s and 2.3 GB. One edited file ≈ 5k tokens → under a second. Memory stays under 1 GB regardless of codebase size, because the model is the same size and the store lives on disk.

**Step 7 — the two caveats in plain words.** (a) Post-training an existing code LLM yields a decoder — the natural path — and 0.5B is as large as it can be and still feel fast on 2 GB; the encoder path is faster and smaller but must be trained differently (tag spans, pair them). (b) Because the card is bandwidth-poor, anything that writes a lot hurts most; "classify instead of generate" and "short ids instead of long paths" matter more on a cheap card than on a large GPU.

**Step 8 — the Linux kernel as a ruler.** 20.1 M lines of C, 8.7 M of headers, 133k of Rust; 33.4 M code lines, 43.9 M with comments and blanks. The C/H/Rust source is 1.49 GB ≈ 400 M tokens → 10–11 tokens per line. The earlier assumption "10 M lines ≈ 100 M tokens" is confirmed; the kernel is ~4× that scenario, so every cold-map time multiplies by four while per-edit and start-up times do not change.

**Step 9 — what the kernel teaches about the two levels.** The kernel's public face is 27k header files and 39k exported symbols against 29 M lines of body — a fraction of a percent by volume. That is why levels 1–2 are cheap to extract and hold in memory, and why bodies must be read lazily, only for the region an agent opens. (Parseltongue is Rust-only; the kernel's actual Rust is 133k lines, which rust-analyzer handles in about a minute. "If the kernel were Rust" is a thought experiment: L1+L2 ≈ 1–2 h on a 2 GB card, ~5 min on a desktop GPU, 20–40 min on a Mac mini.)

In one line: **a 125M highlighter-style model fits a 2 GB card with room to spare and maps a 42k-line workspace in ~10 s because it looks only at the 10–20 % of tokens that need judgment; the kernel confirms the token arithmetic and shows the public interface is tiny next to the bodies.**

#### B9.3 The first map of a kernel-sized codebase on an 8 GB MacBook Air — step by step

*Hardware: MacBook Air, 8 GB unified memory, M1/M2/M3 base (7–10 GPU cores ≈ 2.5–4 TFLOPS peak, 68–100 GB/s, fanless). Codebase: 38 M lines ≈ 400 M tokens, treated as Rust. Two visible levels, rules-first cascade, bodies lazy. Envelope math.*

```text
 STEP 0  what the machine really gives you
         8 GB unified; macOS takes ~2.5–3 GB → ~5 GB for us. The GPU shares that memory — no VRAM wall.
         Fanless: after ~10 min of sustained GPU load it throttles to ~1–1.5 TFLOPS effective. Plug it in; keep it
         awake. rust-analyzer on a codebase this size needs tens of GB → it cannot run here at all.

 STEP 1  read + hash every file                                                        CPU      ~3–5 min
         1.49 GB of text; content digests + tokenising on 4 performance cores ≈ 100 MB/s.
         Output: a File record per file (path @ digest) and the Snapshot id.  RAM: streaming, < 500 MB.

 STEP 2  rule scanner — the regular-grammar parts of L1/L2                              CPU      ~2–5 min
         `mod`, `use`, `include!`, `cfg`, plain `pub fn/struct/trait/enum` signatures, doc comments.
         ~100 MB/s per core. Produces most L1 edges and most L2 items directly, plus a list of AMBIGUOUS SITES
         (glob imports, macro-generated items, trait-dispatch signatures, re-export chains).
         Bodies are skipped — 70 % of tokens never leave the disk.

 STEP 3  the model pass — ambiguous sites only                                          GPU      THE LONG STEP
         40–80 M tokens reach the model (10–20 % of source).
         125M encoder, int8: 250 MFLOP/token at ~1.2 TFLOPS sustained → ~4–5k tok/s
           → 40 M tok ≈ 2.5 h · 80 M tok ≈ 4.5–5 h                                            ≈ 3–5 h
         (0.5B generative model instead: prefill 4× slower ≈ 10–18 h, plus writing ~15 M record tokens at
           ~1–2k tok/s batched ≈ 3–4 h → about a day. On this machine the encoder route is not a nicety.)
         RAM: model 125 MB + activations ~300 MB. Batch files so the GPU never idles.

 STEP 4  write records to the store                                                     CPU      ~2–3 min
         ~5–10 M items and edges at L1/L2. Batched inserts ≈ 200–500k rows/s. Disk: a few GB. RAM: 1–2 GB cache.

 STEP 5  build indexes — trie, trigram, BM25 over names, paths, first doc lines           CPU      ~5–10 min

 STEP 6  fetch external crate graphs from the feed (std + dependencies)                 network  ~1–5 min
         Nothing under `target/` or the cargo registry is ever read by the model.

 STEP 7  coverage banner — files scanned · % extracted ≥ 0.9 · % abstained · model and feed revisions   instant
         The map is queryable. L3 is not built; it extracts lazily, per region an agent opens.

 TOTAL first L1+L2 map on an 8 GB Air     ≈ 3.5–5.5 h with the 125M encoder — an afternoon, plugged in
                                           ≈ 1 day with a 0.5B generative model
 peak RAM, whole pipeline                  ~2–3 GB — fits beside macOS; rust-analyzer cannot start on this hardware

 for scale    Mac mini M4 Pro ≈ 20–40 min · 2 GB discrete card ≈ 1–2 h · desktop GPU ≈ 5 min
 afterwards   one edited file ≈ 1–2 s on the Air; session start = 0 — the store is on disk
```

Three honest notes. Step 3 dominates; the only levers are the cascade (how many tokens reach the model) and encoder-versus-decoder — not the chip. Thermal throttling is why the estimate is 3–5 h and not the 1.5 h the peak figure would promise; cooling genuinely matters on a fanless machine. And a Rust codebase of kernel size does not exist; the largest real Rust workspaces (the compiler, Servo — one to three million lines) would take 15–45 min on the same Air.

#### B9.4 L1 and L2, defined — and which steps run on the CPU

**The two levels the agent sees by default:**

```text
 L1   FILE ↔ FILE             which file depends on which. Evidence: `mod foo;`, `use crate::a::b::C;`, `include!()`,
                              `#[cfg(...)]` gates. Answers: if I touch this file, which files can notice?
 L2   PUBLIC FACE of a file   every item another file is allowed to use: `pub fn` / `pub struct` / `pub trait` /
                              `pub enum` signatures, `pub use` re-exports, `pub static` / `const`, cfg and feature
                              gates, the doc comment. Answers: what does this file promise to the rest of the code?
 L3+  not default             what happens inside bodies: calls, constructions, field reads and writes, awaits,
                              panics, unsafe. Extracted lazily, only for a region an agent opens.
```

The membership test for L2 is one question: *could this line change what another file is allowed to write?* Comments, imports and globals are fields on these records, not separate nodes.

**Who does the work, step by step:**

```text
 step 1   read + hash files                 CPU       I/O and hashing
 step 2   rule scanner                       CPU       regular-grammar matching over tokens. This alone yields MOST of
                                                       L1 and MOST of L2 — `mod`, `use`, plain `pub fn` signatures
                                                       need no model
 step 3   the model, on ambiguous sites      GPU       glob imports, macro-generated items, re-export chains,
                                                       trait-dispatch signatures; later, method receivers in L3.
                                                       Runs on CPU too, 5–10× slower on an Air
 step 4   write store                        CPU
 step 5   build indexes                      CPU
 step 6   feed fetch                         network
 step 7   find / impact / propose, forever   CPU       they read the store; they never run the model
```

L1 and L2 are therefore mostly CPU work — the deterministic scanner produces perhaps 80–90 % of their records — and the GPU is used for the slice the rules cannot decide. The GPU-heavy part of the product is L3, which is exactly the part built lazily.

One consequence, stated plainly: if the model never clears the quality bar, L1 and L2 still exist from the scanner alone, on CPU, with `abstained` marked where the rules gave up. The model improves the map; it is not what makes the map exist.

#### B9.5 The simulation when everything is done by the model

*No rule scanner, no body-skipping: the model reads every token of a kernel-sized Rust codebase (400 M tokens) and produces L1, L2 and L3 itself. Hardware: the 8 GB MacBook Air of B9.3 (≈ 1.2 TFLOPS sustained, 68–100 GB/s). Envelope math.*

```text
 STEP 1  read + hash + tokenise 1.49 GB                                   CPU     ~3–5 min

 STEP 2  the model reads EVERYTHING — 400 M tokens                        GPU
         cost per token = 2 × parameters
         ┌──────────────────────────────┬──────────────────────┬─────────────────────────────────────────┐
         │ model                        │ reading (prefill)    │ writing records (decoder only)          │
         ├──────────────────────────────┼──────────────────────┼─────────────────────────────────────────┤
         │ 125M encoder, int8           │ 1e17 FLOPs ≈ 23 h    │ none — records read off tagged spans    │
         │ 0.5B decoder, Q8             │ 4e17 FLOPs ≈ 4 days  │ ~80 M record tokens (L1+L2+L3) at       │
         │                              │                      │ ~1.5k tok/s batched ≈ 15 h              │
         │ 1.5B decoder, Q4             │ ≈ 12 days            │ ≈ 2 days                                │
         └──────────────────────────────┴──────────────────────┴─────────────────────────────────────────┘
         RAM is fine throughout (model ≤ 1 GB, KV/activations ≤ 1 GB; unified memory has no VRAM wall).
         Over days the run WILL be interrupted — plugged in, kept awake, cooled — so extraction must checkpoint
         per file; the append-only store makes it resumable for free.

 STEP 3  write store — ~10× more records than L1+L2 (every call, construction, field access)   CPU   ~20–30 min
 STEP 4  indexes                                                                                 CPU   ~15–30 min
 STEP 5  feed fetch, coverage banner                                                                   minutes

 TOTAL, everything by model, 8 GB Air       125M encoder ≈ 1 day · 0.5B decoder ≈ 4.5–5 days · 1.5B ≈ 2 weeks

 same job elsewhere                          encoder 125M      decoder 0.5B
   2 GB discrete card                        ≈ 14 h            ≈ 3 days
   Mac mini M4 Pro                           ≈ 5–6 h           ≈ 28–30 h
   desktop GPU (4090 class)                  ≈ 30 min          ≈ 3–4 h

 the two shortcuts, for contrast (Air, encoder)   skip bodies → 120 M tokens ≈ 7 h · + rules on L1/L2 → 3–5 h
```

What the simulation says:

1. **Reading is the bill, not writing.** Even the encoder, which writes nothing, needs a day on the Air: 400 M tokens × 250 MFLOP = 1e17 operations, and the Air sustains ~1e12 per second. No output format changes that number; only reading fewer tokens does.
2. **A decoder makes reading 4× worse and then adds a writing phase** that is bandwidth-bound — each written token re-reads all the weights, and the Air moves ~100 GB/s ÷ 0.5 GB ≈ 200 tok/s per stream. That is the difference between overnight and most of a week.
3. **Memory is never the problem on this machine; time is.** The 8 GB limit does not bite below 1.5B parameters. rust-analyzer could not start at all on this codebase size.
4. **After the first map, everything is fast regardless:** an edited 5k-token file ≈ 1 s (encoder) or ≈ 10 s (decoder); `find` and `impact` read the store in milliseconds; session start is zero.

"Everything by model" is feasible on a MacBook Air for a kernel-sized Rust codebase — once, overnight, with the encoder — and it is a week with a generative model. The cascade and body-skipping are not what make it possible; they turn a day into an afternoon.

#### B9.6 Why reading 400 million tokens is expensive

A model does not *read* a token; it *computes* it. Every token, trivial or subtle, is pushed through every weight in the network. There is no skimming.

```text
 what one token costs, by reader
   grep                        ~10 operations        compare bytes, move on
   regex / rule scanner        ~100
   tokenizer                   ~1,000
   125M-parameter model        ~250,000,000          every token × every weight, through every layer, plus attention
   0.5B model                  ~1,000,000,000
   frontier model              ~1,000,000,000,000

 for 400 M tokens
   grep     400 M × 10      = 4 billion ops                      → about a second on a laptop
   125M     400 M × 250 M   = 100,000,000,000,000,000 ops (1e17)
            a laptop GPU sustains ~1e12 ops/s                    → 100,000 s ≈ 28 h
            the whole answer is 1e17 ÷ 1e12
```

Three intuitions:

1. **The weights are the understanding, and all of them are consulted for every token.** A 125M model is 125 million numbers; a token's meaning is produced by multiplying its vector through all of them, layer by layer. A translator who must consult every one of 125 million dictionary entries for every word — that is the arithmetic, literally. Humans skim; a transformer has a fixed cost per token whether the token is `;` or a subtle trait bound.
2. **It is not slow — it is a lot.** 1e17 operations is a hundred million billion. The laptop does a trillion a second and still needs a day. A human reading 400 M tokens (~300 M words) at 250 words a minute would need 2.3 years; the model is ~800× faster than a person. 400 M tokens is simply enormous — 1.5 GB of text, the entire Linux kernel.
3. **Why it feels wrong: chat.** In a chat the model reads a few thousand tokens — 4k × 250 M = 1e12, one second. Scale 4k to 400 M, a factor of 100,000, and one second becomes 28 hours. Linear, without mercy.

Two corollaries drive the design:

- **The only way to make it cheaper is to send fewer tokens through the model.** Not a better format, not a shorter output — fewer tokens. Skipping bodies (−70 %) and the rules-first scanner (−80 % of the remainder) route `;` and `use std::fmt;` to the 100-operation reader and save the 250-million-operation reader for sites that need judgment. Same map, ~20× less compute.
- **Hardware moves the constant, not the shape.** A desktop GPU is ~50× the Air → 30 minutes. Still 1e17 operations; it does more of them per second.

In one line: **reading 400 M tokens is expensive because a model spends 250 million operations understanding each one, and 400 M × 250 M is 1e17 — a day for a laptop, half an hour for a desktop GPU, a second for grep, which understands nothing.**

#### B9.7 A 2 MB model, and the three ways to a one-minute map

**If the model were 2 MB.** At int8 that is ~2 million parameters → ~4 million operations per token. Same arithmetic as B9.6, new constant:

```text
 everything by model, 400 M tokens, 2 MB model
   compute          400 M × 4 M = 1.6e15 ops
   8 GB Air, GPU    ~1.2e12 ops/s sustained          → ~1,300 s ≈ 22 min
   8 GB Air, CPU    4 cores ≈ 4e11 int8 ops/s        → ~70 min — no GPU needed at this size
   desktop GPU      under a minute of compute; the run is then bound by reading and tokenising 1.49 GB (~3–5 min)
 with the shortcuts  skip bodies (120 M tok) → ~7 min on the Air · + rules → ~2 min
 memory              negligible; the store's page cache dominates

 the ladder, Air, all 400 M tokens    grep ~1 s · rule scanner ~1–2 min · 2 MB model ~20 min · 125M ~1 day · 0.5B ~4–5 days
```

A 2 MB model is 60× cheaper than the 125M one, and a day becomes ~20 minutes — linear, as B9.6 says. The catch is what 2 million parameters can learn: the size of a classic sequence tagger. It can learn local, syntax-shaped decisions — "this token is a type name", "this `pub fn` ends here", "this `use` path maps to that item" — so it can *be* the rule scanner, learned rather than hand-written and more tolerant of odd formatting and macros than regexes. It cannot do what the 125M model is for: pick a receiver's type from context, resolve a glob import, choose between candidate targets. Our own 17 MB encoder-with-heads, eight times this size, learned zero accepted bindings. So: **a 2 MB model yields L1 + L2 in ~20 minutes for a kernel-sized codebase on a laptop, with `abstained` on everything that needs judgment — a learned scanner, not an extractor.** That is a fine first rung: the cascade's front stage, with the 125M model reserved for the 10–20 % it abstains on.

**The one-minute question.** One minute on an 8 GB Air is ~7e13 GPU operations (plus ~2e13 on CPU). Divide by cost per token:

```text
 model     ops/token    tokens the Air can push through in 60 s    share of a 400 M-token codebase
 125M       250 M        ~280k                                      0.07 %
 30M         60 M        ~1.2 M                                     0.3 %
 2 MB         4 M        ~17 M                                      4 %
```

No model — not even a 2 MB one — can read a kernel-sized codebase in a minute on this laptop. Every one-minute method is a way of making the model read at most 4 % of the tokens. There are exactly three such ways, and they compose.

```text
 METHOD 1   FETCH, DON'T COMPUTE                                                        ~1 min, network-bound
            The graph was already extracted by someone with time: the teacher/CI for public crates (the feed), the
            team's always-on box for private code (B11, timeline B). The laptop downloads signed records, verifies
            file digests against its checkout, and the map exists. Any file whose digest does not match falls through
            to method 3. L1+L2 for a kernel-sized tree ≈ 5–10 M records ≈ a few hundred MB compressed.
            The tiny model's role at map time: none. It runs later, lazily, on what the feed lacks.

 METHOD 2   THE MODEL READS HEADLINES, NOT TEXT                                         ~1 min, compute-bound
            A deterministic scanner (CPU, ~400 MB/s on 4 cores → 1.49 GB in ~4 s) reduces the source to the lines
            that can carry L1/L2 facts: `mod` / `use` / `include!` / `cfg` lines and `pub …` signature lines with
            their doc comment. In Rust that is ~2–4 % of tokens ≈ 8–17 M tokens. A 2–5 MB tagger (span tags +
            pairing) reads exactly those → 17 M × 4 M ops ≈ 60 s on the Air's GPU.
            Everything that needs judgment — glob imports, macro output, re-export chains — is written as
            `abstained`, not resolved, and resolved on demand when `find` / `impact` first touches that region, by
            the 125M model over a few thousand tokens (≈ 1 s each). The one-minute map is honest and complete at
            L1/L2; it is not finished resolving, and the coverage banner says so.

 METHOD 3   CONTENT-ADDRESSED CACHE — sccache for graphs                                ~1 min after the first time
            Extraction results are keyed by (file content digest, model revision, schema version). Same bytes
            anywhere → same records, fetched rather than recomputed. For a public tree a shared cache hits ~100 %;
            for private code it hits on everything unchanged since the last map and on all vendored or dependency
            code. The model runs only on misses: after a `git pull` touching 400 files, 400 × 5k tokens = 2 M tokens
            → ~10 s on the 125M model, under a second each on the 2 MB tagger.
            The first-ever map of a private tree is not a minute — it is B9.3's afternoon. Every map after it is.
```

```text
                      first map, public code    first map, private code     every later map
 1 FETCH              yes — the whole answer     partial (deps, std)         same
 2 HEADLINES          fills the gaps             yes — the whole answer      unchanged files skip it
 3 CACHE              method 1 at file level     builds as you go            yes — the whole answer
 the model's job      none at map time           2 MB tagger on ~4 % of      2 MB tagger / 125M on misses only
                                                 tokens; 125M lazily
```

The uncomfortable corollary: a one-minute first map of a *private*, kernel-sized Rust codebase on an Air is possible only through method 2, and only because it defers judgment. A product that promised a minute and then quietly guessed at the abstained sites to look complete would be the exact failure the thesis exists to prevent. The banner — "L1/L2 complete · 6 % of sites abstained, resolving on demand" — is what makes the minute legitimate.

#### B9.8 What a 30-minute base map opens up

Thirty minutes on the Air ≈ 2.2e15 GPU operations (plus ~0.7e15 on CPU). In tokens:

```text
 model     ops/token    tokens in 30 min    share of 400 M
 125M       250 M        ~9 M                 2 %
 30M         60 M        ~36 M                9 %
 2 MB         4 M        ~540 M               135 % — the whole codebase, bodies included, with headroom
```

The first thing that opens is categorical: **at thirty minutes a tiny model can read every token, not just headlines.** Three ways to spend the budget; they are not exclusive.

```text
 SPEND A   2 MB tagger over ALL 400 M tokens (~22 min) + the remainder: 125M on the ~2 M hardest tokens
           → L1 + L2 everywhere, AND an L3 SKELETON everywhere: every call site, construction, field access, await
             and unsafe block recorded as a span with an unresolved target. Nothing skipped, nothing guessed.
 SPEND B   30M model over all ~36 M L1/L2-relevant tokens (~30 min)
           → far fewer abstentions in the base map (glob imports, re-export chains, macro items resolved); the
             banner moves from "~6 % abstained" to "~1–2 %". No L3.
 SPEND C   two different tiny models over the same L1/L2 sites (2 MB tagger + 30M binder, ~25 min)
           → CONFIDENCE FROM AGREEMENT: where both produce the same edge, confidence rises; where they differ,
             `abstained`. Sikos' "two independent sources" (B10), running on a laptop with no compiler.
```

What those unlock:

1. **`impact` works at body level on day one.** With the L3 skeleton, "who calls this?" answers syntactically at once — "47 call sites name `retry_policy`: 41 resolved, 6 pending" — and the 125M model resolves the pending ones lazily, ~1 s each, as the agent touches them. Blast radius stops being an L1/L2 approximation.
2. **Identity across renames and moves** becomes tractable: body skeletons (the shape of calls and constructions inside a function) give a second fingerprint besides the name, so a moved function keeps its id and the level-3 cache survives refactors — the demo's first death mode.
3. **The cache is fully seeded**, so every later map — after a pull, after a branch switch — takes seconds, and the one-minute promise of B9.7 holds for everything afterwards, private code included.
4. **Confidence without a compiler.** The map carries graded trust at runtime, not only "extracted / abstained" — the first time the provenance layer does something the agent can feel: it can skip low-confidence edges under a token budget.
5. **Drafts map at full depth.** The agent's proposed code passes through the warm pipeline in seconds, so `propose` can accept a 2,000-line sketch and return its edges, not only its signatures.
6. **No Mac mini needed for individuals.** "Start the map, get coffee" is acceptable for a kernel-sized tree; for real large Rust workspaces (one to three million lines) the same pipeline is 2–5 minutes — session-start territory for an agent harness.
7. **Two extractor revisions in one run** give the leaderboard a free signal: disagreement sites are exactly the training examples the next rung needs.

Recommended split: **A plus a slice of C** — the 2 MB model over everything (~22 min), then the remaining ~8 min for the 30M/125M model on the sites the tagger flagged ambiguous, recording agreement where both ran. Skeleton, seeded cache and graded confidence in one pass; B's extra resolution arrives lazily afterwards for free.

Thirty minutes does not make the tiny model smarter; it makes it *complete*. At one minute the map is honest but deferred; at thirty it is honest and whole, and everything after that is seconds.

#### B9.9 Time-to-map as an equation — and every candidate model on Hugging Face plugged into it

*The question: write y, the wall-clock time to build the first code map, as a function of the variables that actually move it — quantisation (int8 / int4 / int2), model origin (a large model pruned and distilled down versus a small model trained up), parameter count, model type, number of passes — then put real models into it. Device constants come from measured llama.cpp numbers on Apple Silicon; model facts from Hugging Face model cards fetched 2026-11-06; everything else is envelope math and says so.*

##### The equation

```text
 y  =  T_io  +  Σ over passes p of  [  READ_p  +  WRITE_p  ]  +  T_store  +  T_index  +  T_feed

 READ_p   =  N · (1 − h) · f_p · r_ctx(L)  ·  2 · P_act,p
             ─────────────────────────────────────────────          prefill: every routed token × every active weight
                        η_pre(P_act,p) · C_dev

 WRITE_p  =  N · (1 − h) · f_p · o_p  ·  b_p · P_tot,p                decode (decoders only): every output token re-reads
             ──────────────────────────────────────────              all the weights from memory, batched until compute-bound
                 min( η_dec · B_dev · batch_p ,  η_pre · C_dev / (2·P_act,p) · b_p · P_tot,p )

 N_eff for the NEXT pass  =  N · f_p · a_p          a_p = abstention rate of pass p — what it hands upward
```

The variables, what moves them, and how much they matter for a kernel-sized Rust corpus on an 8 GB Air:

```text
 #   variable                       meaning                                        range here                      effect on y
 ──  ─────────────────────────────  ─────────────────────────────────────────────  ──────────────────────────────  ─────────────────────────
  1  N          corpus tokens        measured 10–11 tok/line; kernel = 400 M        4 M (42k lines) … 400 M         linear
  2  f_p        routing fraction     share of tokens pass p's model actually reads  1.0 all · 0.3 no bodies ·       linear — THE lever
                                                                                    0.10–0.20 with rules in front
  3  P_act      active params/token  the compute cost; MoE: active ≪ total          2 M … 4 B                       linear in READ
  4  P_tot      total params         memory and decode bandwidth                    = P_act except MoE              linear in WRITE, memory
  5  b          bytes per weight     fp16 2 · int8 1 · int4 0.5 · int2/ternary ¼   int8 · int4 · int2               WRITE and memory only —
     (int8 / int4 / int2)            measured: Q4 prefill ≈ Q8 ≈ F16 prefill        (int2 needs special kernels)     prefill does NOT speed up
  6  arch       model type           encoder (o=0) · decoder · SSM (no KV, linear   encoder · decoder · SSM · MoE   decides whether WRITE exists;
                                     in L) · MoE (P_act < P_tot) · enc-dec                                          SSM: r_ctx = 1 at any L
  7  o_p        output ratio         decoder output tokens per input token           0 enc · 0.1–0.3 dec ·           linear in WRITE; short ids
                                                                                    0.3–0.5 verbose JSON (Cassini)   cut it 3–5×
  8  L, r_ctx   context length       attention overhead per token grows with L       1.0 at ≤ 4k · ~1.3 at 20k      multiplies READ (transformers)
  9  C_dev      sustained compute    FLOP/s the device really delivers (prefill)     M1 Air 1.1e12 · M2 Air 1.7e12 · inverse
                                                                                    M4 Air 2.2e12 · M4 Pro mini 6e12
 10  B_dev      memory bandwidth     GB/s, ~80 % achieved                            68 · 100 · 120 · 273            inverse (WRITE)
 11  η_pre/dec  utilisation          tiny models cannot fill a GPU: launch overhead  7B 1.0 · 125M ~0.6 · 2 M ~0.2   inverse — bites tiny models
 12  batch      decode batching      parallel sequences until compute-bound          1 … 32                          WRITE only
 13  k, a_p     passes & abstention  how many models run and what each hands up      1 … 3 · a = 0.02–0.20           Σ; cascades shrink N per pass
 14  h          cache / feed hits    share of files whose records already exist      0 first map · 0.9+ later        linear — the other lever
 15  origin     nerfed-down vs       pruned+distilled from big vs trained small      see below                        acts through a_p (quality),
                small-from-scratch                                                                                   not through FLOPs
 16  T_io, T_store, T_index          read+hash 1.49 GB · write ~5–50 M records · index   ~5 + 3–30 + 5–30 min        floor ≈ 15 min on an Air
```

##### The device constants, measured not guessed

From ggml-org/llama.cpp discussion #4167 (LLaMA-7B, pp512 / tg128; 7B ≈ 1.34e10 FLOP per token):

```text
 chip        BW GB/s  GPU cores  F16 PP t/s  Q8 PP   Q4 PP   Q8 TG   Q4 TG   → effective prefill FLOP/s   → effective decode GB/s
 M1            68      7–8         —          108–117 108–118  7.9     14.2     1.5e12                        53 (Q8) · 54 (Q4)
 M2           100      8–10        201        147–181 146–180  12.2    21.9     2.0–2.7e12                    82 · 83
 M4           120      10          230        224     221      13.5    24.1     3.1e12                        90 · 92
 M4 Pro       273      16–20       381–464    367–450 364–440  30.5    50.7     5.1–6.2e12                    205 · 193
```

Two facts this table settles for the equation. **Quantisation does not speed up prefill** — Q4, Q8 and F16 prefill are within a few percent of each other on every chip, because prefill is compute-bound and the weights are dequantised on the way in. **Quantisation does speed up decode** almost exactly in proportion to bytes — Q4 TG ≈ 1.8× Q8 TG — because decode is bandwidth-bound. So variable 5 lives entirely in WRITE and in memory. A fanless Air sustains roughly 0.7 of the Mac-mini figure for the same chip after ten minutes; the C_dev values above apply that.

##### The models, fetched from Hugging Face today

```text
 model (HF id)                          params  type            licence         notes                                   2·P FLOP/tok
 ────────────────────────────────────── ─────── ─────────────── ─────────────── ─────────────────────────────────────── ────────────
 from-scratch tagger (hypothetical)       2–5 M  encoder/CRF     —               the learned rule scanner (B9.7)          4e6–1e7
 Salesforce/codet5-small                   60 M  enc-dec         Apache-2.0      encoder half usable alone                1.2e8
 microsoft/codebert-base                  125 M  encoder         MIT             code-pretrained, 512 ctx                 2.5e8
 microsoft/unixcoder-base                 125 M  encoder         Apache-2.0      code + AST pretraining                   2.5e8
 state-spaces/mamba-130m-hf               129 M  SSM decoder     Apache-2.0      linear in L, no KV; whole-file context   2.6e8
 HuggingFaceTB/SmolLM2-135M               135 M  decoder         Apache-2.0      general; deep-thin                        2.7e8
 answerdotai/ModernBERT-base              150 M  encoder         Apache-2.0      8k ctx, code in pretraining mix          3.0e8
 bigcode/tiny_starcoder_py                164 M  decoder         OpenRAIL-M      Python only                              3.3e8
 HuggingFaceTB/SmolLM2-360M               362 M  decoder         Apache-2.0                                                7.2e8
 answerdotai/ModernBERT-large             396 M  encoder         Apache-2.0      8k ctx                                   7.9e8
 Qwen/Qwen2.5-Coder-0.5B                  494 M  decoder         Apache-2.0      strongest code prior at this size        1.0e9
 Qwen/Qwen3-0.6B                          752 M  decoder         Apache-2.0      (0.44 B non-embedding)                   1.2e9
 microsoft/bitnet-b1.58-2B-4T             ~2 B   ternary decoder MIT             native 1.58-bit; CPU kernels (bitnet.cpp) 4e9 (adds)
 meta-llama/Llama-3.2-1B                  1.24 B decoder         Llama 3.2       PRUNED + DISTILLED from 8B               2.5e9
 deepseek-ai/deepseek-coder-1.3b-base     1.3 B  decoder         DeepSeek        code-pretrained                          2.7e9
 Qwen/Qwen2.5-Coder-1.5B                  1.54 B decoder         Apache-2.0                                                3.1e9
 google/codegemma-2b                      2.5 B  decoder         Gemma                                                     5e9
 bigcode/starcoder2-3b                    3.0 B  decoder         OpenRAIL-M                                                6e9
 ibm-granite/granite-3b-code-base-2k      3.5 B  decoder         Apache-2.0                                                7e9
 LatentForce-ai/Cassini-1.0               4.0 B  decoder (LoRA)  Apache-2.0      Qwen3-4B, LoRA r=1, 20,480 ctx; emits    8e9
                                                                                 imports/defs/refs/calls/type-assignments
                                                                                 as JSON; 77.6 % F1 on Py/JS/TS at $0.17
                                                                                 vs 17–43× costlier frontier models
 JetBrains/Mellum-4b-base                 4.0 B  decoder         Apache-2.0      code completion                          8e9
 nvidia/Minitron-4B-Base                  4 B    decoder         NVIDIA open     PRUNED + DISTILLED from Nemotron-15B     8e9
 Tesslate/Tessa-Rust-T1-7B                7.6 B  decoder         Apache-2.0      Rust-tuned; far outside the budget       1.5e10
```

Cassini is the one model on the list trained for exactly this task. It is the existence proof — a small LLM can emit per-file dependency facts well enough to be cheaper than frontier models — and it is 4 billion parameters with 20k context and verbose JSON output (o ≈ 0.3–0.5). On an 8 GB Air that is weeks for a kernel-sized corpus. It proves the task; it does not fit the device.

##### Four passes of the equation on an 8 GB M2 Air (C_dev 1.7e12 sustained, B_dev 80 GB/s achieved), N = 400 M

```text
 PASS 1  one model reads everything   (f = 1, h = 0, int8, o = 0.2 for decoders, short handles)
   model                 η     READ          WRITE         y (+15 min floor)
   tagger 3 M            0.2   ~1.5 h        —             ~1.7 h          utilisation, not FLOPs, is the limit at this size
   codet5-small enc 60M  0.5   16 h          —             16 h
   CodeBERT/UniXcoder    0.6   27 h          —             ~1.1 d
   Mamba-130m            0.6   28 h          —             ~1.2 d          same cost as the encoder, but sees whole files
   SmolLM2-135M          0.6   29 h          ~7 h          ~1.5 d
   ModernBERT-base       0.6   33 h          —             ~1.4 d
   ModernBERT-large      0.7   75 h          —             ~3.1 d
   Qwen2.5-Coder-0.5B    0.7   93 h          ~18 h         ~4.6 d
   Llama-3.2-1B          0.8   8.5 d         ~1.9 d        ~10 d
   Qwen2.5-Coder-1.5B    0.8   10.6 d        ~2.4 d        ~13 d
   Cassini-1.0 (4B, 20k ctx, o≈0.4, int4) 0.8   ~36 d    ~10 d         ~46 d
   → under 30 min: nothing reads everything except a ≤ ~2 M model at near-perfect utilisation.

 PASS 2  skip bodies + rules in front   (f = 0.15; same models)
   tagger 3 M            ~15 min  → the 30-minute map          CodeBERT / UniXcoder / Mamba-130m   ~4 h
   ModernBERT-base       ~5 h                                  Qwen2.5-Coder-0.5B                  ~17 h
   Llama-3.2-1B          ~1.6 d                                Cassini-1.0                         ~7 d
   → f is worth 6.7× by itself; it is the single biggest term after N.

 PASS 3  cascade   (tagger f=1 · 125M encoder on the tagger's abstentions a=0.03 → f=0.03 · 0.5B decoder on its
                    abstentions a=0.15 → f=0.0045)
   tagger 3 M everything          ~1.5 h  (→ ~25 min if η reaches 0.7 through batching and fused kernels)
   + UniXcoder-class on 12 M tok  ~50 min
   + Qwen2.5-Coder-0.5B on 1.8 M tok, o=0.2   ~30 min read + ~5 min write
   y ≈ 3 h today, ≈ 1.7 h with a well-utilised tagger — L1 + L2 + L3 skeleton + real binding where it matters.
   → the cascade turns parameter count from a linear cost into a cost on 3 % and 0.45 % of the corpus.

 PASS 4  second map / public code   (h = 0.9 from the content-addressed cache or the feed; cascade of pass 3)
   y ≈ 15 min floor + 10 % of pass 3 ≈ 25–35 min — regardless of which models sit in the cascade.
   → after the first map, the equation is dominated by T_io + T_index, not by the model.
```

##### What the variables say about the two design questions asked

**int8 vs int4 vs int2.** Measured: prefill unchanged; decode 1.8× per halving; memory halves. Quality: int8 is effectively lossless; int4 costs small models more than large ones (a 0.5B model loses measurably, a 7B barely) and is the practical floor for sub-1B decoders; int2 only works with codebook methods (QuIP#, AQLM) or natively trained ternary weights (BitNet b1.58) and has no mainstream Metal kernels — on an Air today int2 is a memory story, not a speed story. For this product's shape — encoders and cascades, almost no decode — **quantisation is the least important variable in the equation**: it changes WRITE, and the design has made WRITE small.

**Nerfed-down big model vs small model trained up.** The evidence favours pruning-plus-distillation whenever a parent exists at 2–4× the target size: Minitron derived 8B and 4B models from a 15B parent with up to 40× fewer training tokens and up to 16 points better MMLU than same-size models trained from scratch; Llama-3.2-1B/3B were produced the same way. MobileLLM shows that below 1B the *architecture* (deep and thin, embedding sharing) adds 2.7–4.3 points at 125M/350M over prior same-size models. Below ~10M parameters there is nothing to prune down to, and the model is a tagger trained from scratch. So the rule is: **≥ 100M — start from a pretrained or pruned-and-distilled code model (UniXcoder, ModernBERT, Qwen2.5-Coder-0.5B) and post-train; ≤ 10M — train from scratch; in between, distil from the 125M.** Origin changes a_p — how much the pass hands upward — and therefore the size of every later term, which is why it matters even though it never appears in the FLOP count.

##### Sensitivity, ranked

```text
 1  f   routing fraction            6–10×     rules and body-skipping; the only lever that beats hardware
 2  P_act                           linear    pick the smallest model that clears the bar for ITS pass
 3  h   cache / feed                up to 10× every map after the first; the feed for public code
 4  η   utilisation at tiny sizes   3–5×      a 3 M model at η 0.2 is as slow as a 15 M model at η 1.0 — batch and fuse
 5  arch (encoder vs decoder, o)    1.2–2×    removes WRITE entirely; SSM removes r_ctx
 6  C_dev                           1.5–4×    M1 Air → M4 Pro mini
 7  b   int8 / int4 / int2          ≤ 1.8× on WRITE only; nil on READ
 8  L, r_ctx                        ≤ 1.3×    keep windows ≤ 4k unless the architecture is linear
```

##### Sources

- ggml-org/llama.cpp, discussion 4167, "Performance of llama.cpp on Apple Silicon M-series" (table rows for M1, M2, M4, M4 Pro, commit 8e672ef).
- Hugging Face model cards fetched 2026-11-06: LatentForce-ai/Cassini-1.0 (base Qwen/Qwen3-4B, LoRA r=1 α=2, 20,480 ctx, 77.6 % F1, $0.17 total inference cost vs $3.03–$7.57 for frontier models); microsoft/codebert-base; microsoft/unixcoder-base; Salesforce/codet5-small; answerdotai/ModernBERT-base and -large; HuggingFaceTB/SmolLM2-135M and -360M; Qwen/Qwen2.5-Coder-0.5B and -1.5B; Qwen/Qwen3-0.6B; state-spaces/mamba-130m-hf; microsoft/bitnet-b1.58-2B-4T; meta-llama/Llama-3.2-1B; deepseek-ai/deepseek-coder-1.3b-base; google/codegemma-2b; bigcode/starcoder2-3b; bigcode/tiny_starcoder_py; ibm-granite/granite-3b-code-base-2k; JetBrains/Mellum-4b-base; nvidia/Minitron-4B-Base; Tesslate/Tessa-Rust-T1-7B. A Hugging Face search for "call graph", "code dependency", "static analysis" and "rust code model" surfaced no other purpose-built extraction models.
- arXiv 2407.14679 (Minitron: pruning + distillation), 2402.14905 (MobileLLM), 2312.00752 (Mamba), 2402.04396 (QuIP#), 2401.06118 (AQLM), 2504.12285 (BitNet b1.58 2B4T).
- Utilisation factors η, the Air's 0.7 thermal factor, abstention rates a_p and output ratios o_p are estimates, not measurements; the equation is built so that each can be replaced by a measured value without changing its shape.

### B10. What a cybersecurity paper knows about code maps that the code-map literature does not

*Deep research on Sikos, "Cybersecurity knowledge graphs", Knowledge and Information Systems 65 (2023), open access, in the repository as `deep-insight-sikos-2023-cybersecurity-knowledge-graphs-KAIS.pdf`. Read against the arXiv literature on graphs for LLM agents over repositories (2024–2026) and against the industry systems that already run the pattern in production. Where a precedent was read from its abstract only, it says so.*

#### The claim

The paper is a survey of knowledge graphs for threat intelligence. It contains no code, no compiler and no LLM. Its contribution to Parseltongue is one pattern, stated in §1 and used in §7, that the entire code-graph-for-agents literature lacks: **a graph assembled from many sources of unequal trust, where every statement carries its source, where agreement between independent sources is the measure of correctness, and where the graph is designed to be queried with some sources switched off.** The code-graph papers have graphs. None of them has a trust model. That gap is the product.

#### What the paper actually says

Four kinds of knowledge graph (§1): **Type I** — a graph of the thing itself (the network, the attack paths). **Type II** — a graph of intelligence about the thing (threat data, behaviour). **Type III** — a graph that is a vocabulary (an ontology of concepts, device types, vulnerability properties). **Type IV** — *multiple, uniquely identified graphs — named graphs — connected to each other, each capturing data from a different source, for amalgamation and dimensionality reduction.* The paper writes a Type IV graph formally as `KGₙ {s p o t}`: every triple lives in a named graph that is its source, with time.

Data fusion (§7): sources of different kinds and trust — scanners, intrusion detectors, firewall rules, routing messages, public datasets — are aggregated into one graph. Two things are said that matter here. *"The base score in this case has been confirmed by two independent sources … such matches indicate high likeliness of information correctness and trustworthiness."* And: *"The above representation also allows provenance or other metadata to be captured for each statement, making it possible to weight … information for incomplete, non-matching, or contradictory statements."* Federated queries then find entities that satisfy conditions across several sources at once.

Reasoning (§4) and learning (§5): provenance graphs where a node's score grows with the number of alerts on the paths through it; attack graphs with probabilities on paths; a-priori labels with confidence propagated over a weighted graph to score unlabelled nodes. All of it runs over the same fused graph without writing into the source layers.

Translated once:

```text
   Sikos                                   Parseltongue
   ─────────────────────────────────────   ──────────────────────────────────────────────────────────
   Type I   graph of the thing             extracted items and edges — the code's structure
   Type II  intelligence about the thing   annotations · proposals — what someone says about or wants of it
   Type III vocabulary                     the versioned node/edge kinds and tags — never written by a model
   Type IV  named graph per source          one named graph per actor and revision: extractor@rev, binder@rev,
                                           feed publisher, agent-LLM, human — connected by shared identities
   per-statement provenance + time         creator · confidence · input digest · schema version · first/last seen
   two independent sources agree           a proposed edge matched by a later extracted edge = realized;
                                           two extractor revisions agreeing = raised confidence
   query with sources switched off          "answer without the model layer" is a filter, not a rebuild
   alerts raise a node's score on a path    failing tests / panics raise the score of the items on their call paths
   a-priori labels propagated with conf.    unsafe · RustSec · untested seeds propagated to a risk lens on pub items
```

#### What the code-graph literature has, and what it lacks

Every paper below builds a graph over a repository so an LLM can navigate it. Read together they settle two questions the thesis depends on, and leave one open.

```text
 work                                      year  graph it builds                           has trust model?     read from
 ────────────────────────────────────────  ────  ────────────────────────────────────────  ───────────────────  ─────────
 CodeNav  (arXiv 2406.12276)               2024  indexed code blocks, search + import       no                   abstract
 CodexGraph  (2408.03910)                  2024  code graph in a graph DB; agent writes     no                   abstract
                                                 graph queries
 RepoGraph  (2410.14684)                   2024  repository-level line/def graph, plug-in   no                   abstract
 LocAgent  (2503.09089)                    2025  directed heterogeneous graph — files,      no; fine-tuned 32B   abstract
                                                 classes, functions; imports, calls,        reaches SOTA at ~86 %
                                                 inheritance                                lower cost
 Iterative code-graph search (2503.22424)  2025  graph searched breadth/depth under LLM     no                   abstract
                                                 control
 KG-based repo-level codegen (2505.14394)  2025  repository as a knowledge graph for        no                   abstract
                                                 retrieval
 Code Graph Model, CGM  (2505.16901)       2025  graph fed into an open LLM directly        no                   abstract
 RepoScope  (2507.14791)                   2025  call-chain-aware multi-view context        no                   abstract
 Partial dependency graph  (2608.01927)    2026  on-demand partial graph, not a static      no                   abstract
                                                 global one
 codebadger  (2603.24837)                  2026  Joern code property graph behind MCP       no                   abstract
                                                 tools for slicing
 CodeNib  (2607.25431)                     2026  lexical + dense + structural views of one  PARTLY — one manifest  abstract
                                                 commit behind one manifest                 per commit; views can
                                                                                            be checked against a
                                                                                            rebuild or live server;
                                                                                            "execution status is
                                                                                            not correctness"
 How should agents see code? (2611.07286)  2026  AST · call graph · module deps · docs ·    no (evaluation)      abstract
                                                 hybrid, as queryable maps
```

Settled, by their numbers: **a map helps when localization is the bottleneck, and the help is mostly tokens.** The 2026 evaluation of five map workflows against a raw-tool baseline: fault localization recall@10 up from 0.717 to 0.757–0.829 with 25–39 % fewer tokens; repository-level repair resolution roughly unchanged (44.3–50.6 % against 49.0 %) with 17–26 % fewer tokens; benefits "task-dependent rather than universal." LocAgent: a graph plus a fine-tuned open 32B model matched proprietary agents at about 86 % lower cost and lifted issue resolution 12 % at Pass@10. CodeNib: handing an agent graph-derived candidates and then retiring them resolved 79 of 200 SWE-bench Verified tasks against 56 for a repository map and 66 for on-demand exploration. This is the evidence for the three verbs and for the token budget inside `find`; it is also the reason the thesis does not promise correctness gains on authoring.

Settled, by their absence: **nobody carries provenance or confidence on the edges.** Twelve systems, one graph each, one source each, no statement of who produced an edge or how sure they were, no way to answer with a source excluded. Only CodeNib names the problem — an incremental index "can complete every update without error and still return the wrong graph half the time" — and its answer is a per-commit manifest and a check against a rebuild, which is a provenance mechanism for one source. The multi-source case — compiler, model, feed, agent, human writing into one map — is not in the literature at all.

Open: whether the trust model changes agent outcomes, not only agent honesty. No paper has tested a map with confidence against the same map without it. That is a fourth cheap signal, and it is ours to run.

#### The industry already runs the pattern — in a different field

The trust layer does not need inventing. It needs porting.

```text
 precedent                        what it standardised                                  what we take
 ───────────────────────────────  ───────────────────────────────────────────────────  ─────────────────────────────────
 STIX 2.1 (OASIS)                 confidence on every object; Sighting as a separate     sighting record · opinion record ·
                                  relationship ("an intelligence assertion that          confidence as a first-class field
                                  something was seen"); Opinion object with a five-
                                  point agree/disagree scale
 MISP (open-source CTI platform)  sightings · proposals (ShadowAttribute) · warning-     warninglists for derive/prelude
                                  lists of known false positives · taxonomies as        noise · machine tags as the only
                                  machine tags · galaxies · feeds with distribution     classification · signed feeds with
                                  levels · first_seen / last_seen · to_ids flag          distribution · first/last seen ·
                                                                                        "actionable" flag
 W3C PROV-O                       entity · activity · agent; wasGeneratedBy,            the shape of the provenance record:
                                  wasAttributedTo, wasDerivedFrom                        record ← activity(model@rev, input
                                                                                        digest) ← agent
 Knowledge Vault (Google, 2014)   web-scale facts from many extractors; per-fact        confidence from extractor agreement
                                  confidence from extractor agreement plus prior;       plus a prior from the feed; facts
                                  facts never deleted, only scored                      never deleted
 Kythe · Glean · SCIP             a schema of code facts that many indexers write        one record schema, many writers;
 (Google · Meta · Sourcegraph)    into one store; cross-references served from facts     the extractor is a writer like any
                                  not from a live analyzer                               other
 Joern code property graph        one graph that merges AST, control flow and data       levels as overlays on shared ids,
                                  flow on shared nodes                                   not separate graphs
 in-toto · SLSA · Sigstore        signed attestations about how an artifact was          a feed entry is a signed attestation
                                  produced, verifiable by anyone                         of a crate's graph by a named
                                                                                        publisher with a named teacher
```

STIX and MISP are the paper's own field, and they are the proof that the pattern survives contact with adversarial, incomplete, contradictory data at scale across organisations that do not trust each other. That is a harder environment than a codebase. If per-statement provenance, sightings, warninglists and distribution levels work for indicators of compromise shared between governments, they work for edges shared between a compiler, a model and an agent.

#### Why the code-graph field missed it

Each system has one source — a parser, a compiler, an embedding — and one author. When there is one source there is nothing to disagree with, so provenance looks like bureaucracy. The moment there are two sources it becomes the whole question: *which edge do I believe?* Parseltongue has six writers by design — extractor, binder, ranker, labeller, feed, agent, human — and the first paper to address that situation squarely is a cybersecurity survey, because cybersecurity has had many sources from the beginning.

The paper's single most transferable sentence is the one about two independent sources confirming a score. It is the mechanism behind `realized`: the agent proposed the edge, the extractor later found it, two independent actors agree, the record is now the most trustworthy kind in the map. It is also the mechanism for raising confidence without a compiler at runtime: two extractor revisions, or an extractor and the feed, agreeing on an edge is Sikos's "high likeliness of correctness" — and their disagreement is exactly what `abstained` is for.

#### What this adds to the thesis

Nothing changes in the entity model; this section is why it looks the way it does. Three things sharpen:

1. **Named graph per actor and revision is the storage primitive**, not a tag on a record. It is what makes "answer without the model layer" a filter and what lets a model revision be retired without touching anyone else's records.
2. **Agreement is the confidence function.** Extracted-matches-proposed, extractor-matches-feed, revision-matches-revision — each is a sighting by an independent actor. Confidence is computed from sightings, never asserted by one writer.
3. **The fourth cheap signal**: the same three-repository pilot, with the provenance and abstention fields shown to the agent in one arm and hidden in the other. If the arm that can see them makes fewer edits on unresolved edges, the trust layer earns its place in the output, not only in the store.

#### Sources

- Sikos, L. F. "Cybersecurity knowledge graphs." *Knowledge and Information Systems* 65, 3511–3531 (2023). https://doi.org/10.1007/s10115-023-01860-3 — open access; PDF in this repository.
- arXiv: 2406.12276 · 2408.03910 · 2410.14684 · 2503.09089 · 2503.22424 · 2505.14394 · 2505.16901 · 2507.14791 · 2510.04905 (survey) · 2603.24837 · 2607.25431 · 2608.01927 · 2608.29606 · 2611.07286.
- STIX 2.1 specification (OASIS); MISP project documentation; W3C PROV-O; Dong et al., "Knowledge Vault" (KDD 2014); Kythe, Glean, SCIP documentation; Joern; in-toto, SLSA, Sigstore. Industry entries are from working knowledge of the systems, not re-verified for this section except STIX's Sighting and Opinion definitions.

### B11. Five shapes the product could take — traversed as timelines

*The thesis fixes what the map is. It does not fix what the product around it looks like, and for a very large codebase that choice decides whether anyone ever runs the extractor. This section does not argue by preference; it runs five plausible product shapes forward in time and compares them. Assumptions are stated. Likelihoods are judgments.*

#### Decision frame

- **Fork in the road:** given one map (extracted + proposed, two visible levels, provenance on every edge) and the extractor that produces it, which *product shape* do we build first for mapping very large Rust codebases?
- **Desired outcome:** an agent working on a large Rust workspace reaches for Parseltongue instead of grep within its first session, and keeps reaching for it; the corpus, feed and benchmark exist regardless.
- **Hard constraints:** open source, open weights, open data; no compiler, rust-analyzer or tree-sitter at runtime; the tool never authors the future half of the map; a 500 MB–1 GB extractor that wants a GPU for whole-workspace work; one or two people building.
- **Time horizon:** week 1 · month 1 · quarter 1 · year 1.
- **What counts as failure:** the extractor is built and nobody runs it on a large codebase; or it runs, and the agent's outcomes do not change.

#### Timeline A — The CLI navigator: map lazily, answer fast

- **Opening move:** ship one binary. `parseltongue map .` extracts level 1 and level 2 only, rules first, model on the ambiguous sites; bodies are extracted on first expansion of a region and cached. `find`, `impact`, `propose` read the store.
- **Week 1:** on a 42k-line workspace the two-level map takes seconds; the agent's first `find` returns four anchors with blast radius. On a 2 M-line workspace it takes a few minutes on a laptop GPU, overnight on CPU. Early users with large workspaces ask for a progress bar and a way to map one crate at a time.
- **Month 1:** per-crate mapping lands; the level-3 cache grows only where agents actually look, which turns out to be a few percent of the tree. The feed supplies external signatures so the extractor never reads `target/`. The first complaint is staleness after a `git pull` that touched 400 files — re-extraction on save handles edits; bulk changes need a batch pass.
- **Quarter 1:** a watch mode re-extracts changed files in the background. The three-arm pilot runs on this shape. If the agent-plus-map arm wins on tokens and localization, the product has a reason to exist; if it does not, this shape has cost the least to find out.
- **Long-term shape:** a utility every Rust agent harness shells out to. Low ceiling as a company, high floor as a project.
- **Likelihood of reaching quarter 1 intact:** high — it is the smallest shape that exercises the whole thesis.
- **Stress points:** "why is my first map taking an hour" on a CPU-only machine; identity churn across renames breaking the level-3 cache.
- **Inflection points:** whether lazy level-3 extraction covers what agents open (if agents open everything, laziness buys nothing); whether watch mode stays under a second per file.

#### Timeline B — The always-on map server: one box owns the map, everyone reads

- **Opening move:** the map lives on a machine that does not sleep — a Mac mini, a CI runner, a workstation. It extracts the whole workspace once, re-extracts on every push, and serves `find` / `impact` / `propose` over a socket. Laptops and agents run a thin client.
- **Week 1:** the full map of a 5 M-line workspace is built overnight on the box. Every developer's first query the next morning is instant. Nobody's laptop has a model on it.
- **Month 1:** proposals from many agents land in one map; sightings accumulate; the authority matrix earns its keep because there are now several LLMs and humans writing into the same store. Someone asks who may withdraw whose proposal.
- **Quarter 1:** the map is on the critical path of the team's agent workflow. Downtime of the box is downtime of the agents. Access control, backup and a second box appear on the roadmap, and the project is now operating a service.
- **Long-term shape:** an internal platform; the obvious hosted-product path if one ever wanted it. The furthest from "one binary", the closest to how large organisations actually use shared code intelligence.
- **Likelihood of reaching quarter 1 intact:** medium — the engineering is modest; the operational commitment is the risk.
- **Stress points:** operating a service with one or two people; the cold map of a 10 M-line workspace taking a night even on a GPU; the box becoming a single point of failure.
- **Inflection points:** whether a team exists to adopt it (it needs more than one user to be worth running); whether multi-writer semantics are designed before the second writer appears.

#### Timeline C — Feed first: publish the graphs, let others build the tool

- **Opening move:** run the teacher on every crate version on crates.io, publish signed per-crate graphs as a public feed, and publish the compiler-labelled corpus and benchmark. Ship the record schema and a reference reader. Build the extractor and the verbs second.
- **Week 1:** the feed is a static site and an index. People fetch `tokio@1.38` as a graph and look at it. The first external issue is a licence question from a crate author; the second is a request for a different serialisation.
- **Month 1:** two external projects consume the feed for their own purposes — one for documentation, one for an unrelated agent tool. Neither is Parseltongue. The corpus gets cited. The extractor is still unbuilt.
- **Quarter 1:** the first open model trained on the corpus appears on the leaderboard — possibly not ours. The product that reads the map has a feed to lean on from day one and never has to extract `std`.
- **Long-term shape:** the project becomes the data layer of the Rust agent ecosystem. The tool may or may not be the one most people use, and by the thesis's own statement that is mission success.
- **Likelihood of reaching quarter 1 intact:** high — the inputs are deterministic and the work is batch; nothing depends on a model being good.
- **Stress points:** licence filtering and takedown handling; teacher versions drifting; the long quiet period in which there is data and no demo.
- **Inflection points:** whether anyone other than us consumes the feed in month 1 (if nobody does, the "durable asset" claim weakens); whether the leaderboard's first row is a rule-based extractor — which is fine, and will feel bad.

#### Timeline D — Editor-embedded: the map lives where the code is edited

- **Opening move:** an MCP server plus editor extension. The map is extracted for open files and their level-1 neighbourhood first, the rest of the workspace in the background. The agent inside the editor sees `find` / `impact` / `propose` as tools; the human sees the map's proposed edges inline.
- **Week 1:** a newcomer in an AI editor types "where do I even start" and gets a ten-item tour of a workspace that does not currently build. This is the demo that converts in ninety seconds, and it converts people who never read a thesis.
- **Month 1:** background extraction of a large workspace competes with the editor and rust-analyzer for the same machine. The model moves to a GPU or is throttled; on a 2 GB card the encoder route is the only one that stays responsive. Proposed edges drawn in the gutter get noticed, and get authored, because the authoring is one click away from the code.
- **Quarter 1:** two or three editors supported, each with its own quirks; the MCP surface is stable, the extensions are a maintenance tax. Signal 2 — do people keep writing proposals — gets its best possible test here, because this is the only shape where authoring is cheap for humans.
- **Long-term shape:** the way most individuals meet the product; the shape that competes most directly with the editor's own code intelligence.
- **Likelihood of reaching quarter 1 intact:** medium — the UI work is open-ended and competes with the core.
- **Stress points:** three editors, three bug trackers; the extractor and rust-analyzer fighting for memory on a laptop; the temptation to replace rust-analyzer's job rather than complement it.
- **Inflection points:** whether the proposed-edges gutter is used by anyone twice; whether background extraction can be made invisible on a laptop.

#### Timeline E — Benchmark first: a research project with a leaderboard, no product yet

- **Opening move:** publish the corpus, the held-out repositories, the scoring harness and the leaderboard, with a rule-based extractor as row one. Train the ladder in public. Do not ship a product until a model clears the bar on the leaderboard.
- **Week 1:** the harness scores the rule extractor against the compiler on twenty unseen repositories. The number is public and probably unflattering for method calls.
- **Month 1:** rung two of the ladder is trained and scored; the result either beats the rules at a fixed wrong-edge budget or it does not. Either way the question "if the tiny model works" has a number, which is more than it has today.
- **Quarter 1:** if a model clears the bar, every other timeline starts from a proven extractor and the product debate is purely about shape. If none does, the project pivots to the rule extractor plus feed as the runtime — still useful, no longer the bet.
- **Long-term shape:** an open benchmark the field uses, with or without our product. The least product, the most truth.
- **Likelihood of reaching quarter 1 intact:** high — but with the real possibility that the result ends the model bet.
- **Stress points:** months with nothing an agent can run; the first leaderboard being won by rules; funding patience.
- **Inflection points:** rung two's score; whether external teams submit rows.

#### Cross-timeline analysis

```text
 path                upside                          downside                      reversible?   regret risk          who has to cooperate
 ──────────────────  ──────────────────────────────  ────────────────────────────  ───────────   ───────────────────  ──────────────────────────────
 A  CLI navigator    smallest full test of the       low ceiling; CPU users wait   fully         low — cheap to learn  one harness author willing to
                     thesis; one binary; pilot-ready  an hour for the first map                   it's "just a utility" shell out to it
 B  map server       instant for everyone; multi-    operating a service with      mostly        medium — effort sunk  a team with a box and a reason
                     writer map is where the entity  two people; single point of                 into ops, not product to share one map
                     model proves itself              failure; needs a team
 C  feed first       deterministic; durable; others  no demo for a quarter;        fully         low — the asset       crate authors (licences);
                     can build on it; no model risk   licence work; quiet                         outlives mistakes     one external consumer
 D  editor-embedded  converts non-readers in 90 s;   open-ended UI; three editors; mostly        medium — UI time is   editor vendors' MCP surfaces;
                     only shape where humans author  fights RA for laptop memory                 never recovered       a GPU in the laptop
                     proposals cheaply
 E  benchmark first  answers "does the model work"   nothing to run for months;    fully         low — truth is never  external teams to submit rows
                     before anything is built on it  may end the model bet early                 regretted; patience is
```

Variance: C and E are low-variance — their outputs exist whatever the model does. A is low-variance in cost and medium in outcome. B and D are high-variance: large upside if a team or editor adopts, large sunk cost if not.

The hidden coupling: **A, B and D all need the same store and the same three verbs; C and E produce what every other path consumes.** The shapes are not rivals — they are an order.

#### Decision filter

- **Strongest if everything goes normally:** C then A. Publish the feed and corpus while the extractor is being built (C is batch work and does not block on model quality), ship the CLI navigator the moment the two-level map is fast on a 42k-line workspace, and run the three-arm pilot on A. Everything in B and D is a thin layer over A's store and verbs — add B when a team asks for it, D when the pilot shows agents use the map.
- **Safest if things go badly:** C and E. If the model never clears the bar, the feed, corpus and leaderboard are still the thing nobody else has, and the rule extractor plus feed is still a usable runtime behind the same verbs. No other path degrades this gracefully.
- **The experiment that collapses uncertainty fastest:** rung two of the training ladder scored on the public harness — one afternoon, under a dollar — run before the shape debate consumes another week. Its result decides whether A's extractor is a model or a rule set; nothing about the store, the verbs, the feed or the entity model waits on it.

Said in one line: **build the data first, the utility second, the service and the editor only when someone asks — and let a one-afternoon measurement, not a preference, decide what sits inside the extractor.**

### B12. Replacing LSP with a tiny model and a CLI

*Three questions, answered in order: what the model should talk to while it finds dependencies; which deterministic, non-LSP tools can search code and where each belongs; and what "replace LSP" means request by request.*

#### B12.1 Who the extractor talks to — and why it is not FFF, and Cargo is not enough

**Does the model need to ask FFF many questions to find dependencies?** No — wrong oracle. FFF answers "which *file* does this name mean" (fuzzy path match plus frecency). Dependencies are discovered by *reading source*, and the question the extractor keeps asking is "where is item `X` defined, and what is its signature?" — a lookup into the store and the feed, not a file search.

```text
 model reads a file ──"candidates for `Sender`?"──►  index (trie + feed)  ──►  [tokio::sync::mpsc::Sender,
                    ◄── picks one, or abstains ───                               std::sync::mpsc::Sender, crate::bus::Sender]
 FFF sits one layer up: inside `find`, turning the agent's words into candidate FILES. The extractor never calls it.
```

The binder is chatty — a few lookups per ambiguous site — against an in-process index at microseconds each, so it is free. Chattiness toward a subprocess would not be.

**Is Cargo enough?** Cargo gives the *crate* graph — which crates, versions, features, cfg — exact and cheap through `cargo metadata`, no build. That is level 0, above L1, and a deterministic input the extractor reads before touching any file. It gives nothing below that: no file-to-file, no item-to-item, no signatures. And it needs a manifest, so it is absent for snippets, drafts and un-manifested code.

**Is rust-analyzer's problem "no agentic interface"?** Partly. Precisely:

```text
 RA HAS      LSP (agents can drive it) · `rust-analyzer scip`, a batch export of the whole definition/reference graph ·
             near-exact answers
 RA LACKS    a persisted store (re-derives every session) · budgeted, neighbourhood-shaped answers (LSP is one cursor,
             one question) · a provenance/abstention vocabulary · any slot for proposed structure · operation without
             a loadable workspace · a footprint that fits small machines
```

"No agentic interface" is the visible symptom. The cause is that rust-analyzer is a *function* that must be re-run inside a loaded workspace, and agents need a *database* they can query cheaply, write intent into, and trust by degrees. Two consequences: `rust-analyzer scip` is how the teacher labels the corpus — it already exports the graph we want learned, so the teacher pipeline is mostly plumbing; and the product's differentiation is not "RA behind MCP" (that exists) but the store, the budgeted verbs, the proposed layer, and running where RA cannot.

#### B12.2 Deterministic, non-LSP code search — what exists and where it belongs

```text
 layer                   tool(s)                                 how it works                            exact?   place in Parseltongue
 ──────────────────────  ──────────────────────────────────────  ──────────────────────────────────────  ───────  ────────────────────────────────
 FILES by name           FFF · fzf · skim                        fuzzy path + frecency / git signals     n/a      the file retriever inside `find`
 TEXT / regex            ripgrep · ugrep · git grep              SIMD regex over bytes, gitignore-aware  exact    the floor; ~1 s over the kernel;
                                                                                                                  lines, not entities
 TRIGRAM INDEX           Zoekt · Hound · Livegrep                prebuilt trigram index → instant regex  exact    the idea (our own trigram index);
                                                                 over very large repos                            not the daemons
 STRUCTURAL / AST        ast-grep · Comby · Semgrep (OSS)        patterns with holes matched on the      exact    excellent, but ast-grep is built on
                                                                 syntax tree (`fn $N($$$) -> Result<$T>`)          tree-sitter → dev tool, not runtime,
                                                                                                                  unless that rule is relaxed
 SYMBOL INDEX, untyped   universal-ctags · GNU GLOBAL · cscope   regex-ish definitions table             mostly   what our rule scanner produces itself
 SYMBOL INDEX, typed     SCIP via `rust-analyzer scip` · Kythe · exact def/ref graph exported as a file, exact    OFF-LINE — the teacher's labelling
                         Glean                                   queried without the analyzer                     format
 SEMANTIC QUERY          CodeQL (Rust supported) · Joern         code property graph + query language;   exact    needs an extractor/build pass →
                                                                 build-shaped, slow                               teacher/judge material at most
 RUST-SPECIFIC           cargo metadata · cargo tree ·           crate graph · module tree · public API  exact    cargo metadata at runtime (level 0);
                         cargo-modules · rustdoc JSON ·          dump · semver diff                               the rest need a build → teacher side
                         cargo-public-api · cargo-semver-checks
```

The runtime search stack is therefore: a ripgrep-class regex engine, an FFF-class file ranker, and our own trie / trigram / BM25 over the *records*. The first two find text and files; the third finds *entities* — `crate::mod::Item#kind` with a public anchor and a blast radius — which none of the others do. ast-grep is the one tool worth missing: structural patterns are how one would check a proposed edge ("does any `impl RetryPolicy for $T` exist yet?") without a model; the same few patterns can be written against the scanner's own spans.

In one line: ripgrep finds lines, FFF finds files, SCIP finds symbols off-line, Cargo finds crates — and the gap all of them leave, a persisted entity graph with public anchors and provenance queryable under a token budget, is the product.

#### B12.3 Replacing LSP, request by request

The deterministic tools above are inputs, not the replacement. **The replacement for LSP is the store the tiny model fills plus the CLI verbs that read it.**

```text
 LSP request an agent makes        Parseltongue equivalent                           data produced by          gap versus LSP
 ────────────────────────────────  ────────────────────────────────────────────────  ────────────────────────  ─────────────────────────────
 textDocument/definition           `find <name>` → item record + span                 scanner (plain cases) ·   none for L2 items; method
                                                                                      model (binding)           targets carry confidence
 textDocument/references           `impact <item>` → callers, users, implementors     model (L3 calls) · store  lossy where abstained — and the
                                                                                                                coverage banner says so
 workspace/symbol                  `find` over trie / trigram / BM25                  store / index             better: ranked, budgeted
 documentSymbol (outline)          `find --file f` → the L2 face of the file           scanner                   none
 hover (type, signature, docs)     the item record: signature text · doc · cfg        scanner + model           no inferred types of locals
 implementation / typeDefinition   `impact --kind implements`                          model + store             confidence-graded
 callHierarchy, incoming/outgoing  `impact --depth 2`, bounded by token budget         model + store             budgeted, not exhaustive
 rename                            out of scope — the agent edits; `impact` says where  —                        LSP's rename is exact; ours is
                                                                                                                a list to act on
 diagnostics / type errors         NOT REPLACED — the compiler's job; `cargo check`    rustc, when the agent     the one thing LSP does that we
                                   stays the agent's separate tool                     chooses to run it         never claim
 what LSP cannot do                proposed edges · provenance · abstention ·          LLM / human · store       —
                                   persisted store · snippets, drafts, broken code
```

Three points about the replacement, stated plainly:

1. **It is a different contract.** LSP is exact-or-nothing from one cursor inside a loaded workspace. Ours is complete-with-confidence from a persisted store, anywhere. For an agent planning a cross-cutting change the second is more useful; for "is this exact type right" the first still wins — which is why `cargo check` stays a separate tool the agent runs when it wants the compiler's verdict, not something Parseltongue wraps.
2. **The tiny model is the only non-deterministic part, and it sits at write time, not query time.** Every verb is a deterministic read of records. The agent converses with the CLI; the model converses with the index while filling the store. That separation is what makes answers reproducible and auditable even though a model produced the edges.
3. **The replacement is complete at L1/L2 from the scanner alone** — definition, outline, symbols, hover-as-signature. The model upgrades references and call hierarchy from "syntactic mentions" to "resolved edges with confidence." The LSP replacement is shippable before the model clears the bar; the model raises its ceiling.

### B13. The multi-level code map, defined

*What the tiny model and the CLI build together. One map, three nested levels — folder, file, public interface — where every node has a kind and metadata, every dependency is an edge with a kind, and an edge at an upper level is the roll-up of edges below it. Crate (from `cargo metadata`) sits above as level 0 context; bodies (L3) sit below and are extracted lazily. This section fixes the vocabulary; §5 fixes identity and authority.*

#### B13.1 The three levels and how they nest

```text
 level  node          what it is                                  edge "A → B" means                              derived by
 ─────  ────────────  ──────────────────────────────────────────  ──────────────────────────────────────────────  ───────────────────────
 L0     Crate         a Cargo package@version                     A declares a dependency on B (features, cfg)   cargo metadata (no build)
 L1a    Folder        a directory under the crate root            some file in A depends on some file in B       ROLL-UP of L1b
 L1b    File          one .rs file (content-digested)             A names B: `mod`, `use`, `include!`, cfg,       scanner; model for
                                                                  or an item in A uses an item in B               ambiguous imports
 L2     Item          one public (or crate-visible) item          A's signature or face refers to B               scanner (plain) ·
                                                                                                                  model (binding)
 L3     Item (body)   the same items, their bodies                A's body calls / constructs / reads B           model, lazily
```

The roll-up rule, which is what makes it one map rather than three:

```text
 an edge  A → B  exists at level n   ⇔   at least one edge  a → b  exists at level n+1 with a ⊂ A and b ⊂ B
 weight(A → B)      = number of contributing lower edges
 kinds(A → B)       = the set of lower kinds that contributed  (e.g. {uses, signature_uses, calls})
 cfg(A → B)         = union of lower cfgs
 confidence(A → B)  = [min, max] of lower confidences — a range, never an average
 status(A → B)      = extracted if any contributor is extracted; proposed if all contributors are proposed
```

So `impact` at folder level is not a separate analysis; it is the same records summed. Expanding a folder edge shows the file edges that made it; expanding a file edge shows the item edges; expanding an item edge shows the span.

#### B13.2 Node kinds and their metadata

Every node carries the same base record; each level adds its own kind and fields.

```text
 BASE (all levels)   id (per §5) · kind · display name · crate · cfg · creator (scanner@rev | model@rev | feed@publisher |
                     llm | human) · confidence · status (extracted | abstained | inferred | proposed | realized | stale) ·
                     first_seen / last_seen snapshot · tags (machine tags, e.g. role:entry-point — annotations, not identity)

 CRATE  (L0)   kind ∈ { lib, bin, proc-macro, workspace-member, external, std }
               fields: name, version, edition, features enabled, source (path | registry | git), licence

 FOLDER (L1a)  kind ∈ { crate-root, src, module-dir (has mod.rs or <name>.rs), tests, examples, benches, bin, build, generated }
               fields: path, file count, item count by kind (rolled up), lines, languages present, owning module path

 FILE   (L1b)  kind ∈ { lib-root (lib.rs), bin-root (main.rs), module (foo.rs / foo/mod.rs), inline-only (no items),
                        test, bench, example, build-script, included (pulled in by include!), generated }
               fields: path, content digest, module path it defines, lines, tokens, pub item count, cfg gates at file top,
                       doc comment (//! first line), has_unsafe, has_macros_defined, extraction coverage (% of sites resolved)

 ITEM   (L2)   kind ∈ { fn, struct, enum, union, trait, type-alias, const, static, macro (macro_rules / proc), impl,
                        assoc-fn, assoc-const, assoc-type, field, variant, extern-fn, extern-static, mod (inline) }
               fields: path (crate::module::Name), visibility declared (pub | pub(crate) | pub(super) | pub(in path) | private)
                       and visibility effective (after re-exports, if known), signature text (normalised), generics and
                       bounds, flags {async, unsafe, const, extern-abi}, attributes of interest (derive list, cfg, deprecated,
                       must_use, inline, repr), doc comment (first line + full), span (file, line range), body digest (L3),
                       size (lines), for impl: trait + self type; for field: type; for variant: payload shape
```

The L2 membership test is one question: *could this line change what another file is allowed to write?* Comments, imports and globals are fields on these records, not nodes: a doc comment is the item's `doc`, an import is the evidence span on a `uses` edge, a static is an item of kind `static`.

#### B13.3 Edge kinds and their metadata

```text
 BASE (all edges)   id = hash(src, dst, kind, cfg) · kind · src id · dst id · cfg · evidence span(s) · creator · confidence ·
                    status · weight (1 for leaf edges; count for roll-ups) · via (collapsed private path, if the edge is
                    projected through non-public items) · first_seen / last_seen

 L0  crate         depends_on (normal | dev | build) · enables_feature
 L1  file / folder declares_mod        `mod foo;` in A declares the module whose file is B
                   uses                `use` in A names an item that lives in B   (evidence: the use path)
                   includes            `include!` / `include_str!` of B
                   cfg_gates           A's inclusion of B is behind a cfg
                   tests               A is a test file exercising B
                   depends_on          roll-up: any L2/L3 edge from an item in A to an item in B (weight, kinds, cfg range)
 L2  item face     signature_uses      a type in A's signature is B
                   parameter           A takes B as a parameter type             (specialisation of signature_uses)
                   returns             A returns B                                (specialisation of signature_uses)
                   field_type          a field of A has type B
                   implements          A (an impl) implements trait B for a self type
                   trait_bound         A's generics are bounded by trait B
                   associated_type     A's associated type resolves to B
                   reexports           `pub use` in A re-exports B under a new path
                   derives             A derives B (Debug, Clone, serde::Serialize …)
                   macro_generates     item A was produced by macro B   (status often abstained without expansion)
 L3  item body     calls · may_call (dyn / generic dispatch; candidates ranked) · constructs · reads_field · writes_field ·
                   awaits · panics_via · unsafe_uses · influences (data flow sketch)
 any level         proposed_* — any of the above with status proposed, authored by an LLM or human (§5); the tool never
                   creates one
```

The vocabulary is versioned by schema release; a model may not invent a kind. Where the extractor sees a site but cannot name the target it writes an edge to a placeholder target with status `abstained` and the span — so absence is never silent, and the roll-up shows "File A → 3 unresolved sites" rather than nothing.

#### B13.4 Who produces what

```text
                              scanner (deterministic, CPU)             tiny model (CPU / GPU)                LLM / human
 L0  crates                   cargo metadata                            —                                     —
 L1  folders                  roll-up                                   —                                     —
 L1  files   declares_mod,    all plain cases                           glob imports (`use a::*`), imports     —
             uses, includes,                                            through macros or cfg'd re-export
             cfg_gates                                                  chains → inferred with confidence
 L2  items   nodes + fields   every item with a parseable signature;    items produced by macros; effective
                              doc, cfg, attrs, span, digest              visibility through re-export chains
 L2  edges   signature_uses,  when the type path is fully qualified or  when the name must be bound among
             returns, field,  resolves by a plain `use`                  candidates (binder); trait_bound and
             implements…                                                 associated_type resolution
 L3  bodies                   call-site SPANS (where a call happens)    targets: receiver typing, method
                                                                         resolution, may_call ranking, abstention
 proposed    —                —                                          all of it
 annotations —                —                                          labeller model (opinion layer) · LLM · human
```

The scanner alone yields L0, L1 and most of L2 — a complete, honest map with `abstained` where the rules stop. The model raises the ceiling: fewer abstentions at L1/L2, and the whole of L3.

#### B13.5 A worked example

A small crate, `retry`:

```text
 retry/
   Cargo.toml
   src/lib.rs            pub mod policy;  pub mod client;  pub use client::Client;
   src/policy/mod.rs     pub trait RetryPolicy { fn next_delay(&self, attempt: u32) -> Option<Duration>; }
                         mod fixed;  pub use fixed::FixedBackoff;
   src/policy/fixed.rs   pub struct FixedBackoff { pub delay: Duration }
                         impl RetryPolicy for FixedBackoff { … }
   src/client/mod.rs     use crate::policy::RetryPolicy;
                         pub struct Client { policy: Box<dyn RetryPolicy> }
                         impl Client { pub fn send(&self, req: Request) -> Result<Response, Error> { … self.policy.next_delay(n) … } }
```

The three levels, as the map holds them:

```text
 L2  ITEM EDGES (leaf records)
   client::Client#struct            ──field_type──►      policy::RetryPolicy#trait        (Box<dyn RetryPolicy>)       scanner  1.00
   client::Client::send#assoc-fn    ──parameter──►       Request#struct                                                 scanner  1.00
   client::Client::send#assoc-fn    ──returns──►         Response#struct · Error#enum                                   scanner  1.00
   policy::fixed::<Impl#1>#impl     ──implements──►      policy::RetryPolicy#trait  (for FixedBackoff)                  scanner  1.00
   policy::mod.rs                   ──reexports──►       policy::fixed::FixedBackoff#struct  (as policy::FixedBackoff)   scanner  1.00
   lib.rs                           ──reexports──►       client::Client#struct               (as retry::Client)          scanner  1.00
   client::Client::send#assoc-fn    ──may_call──►        policy::RetryPolicy::next_delay#assoc-fn    (dyn dispatch)      model    0.93  [L3]
                                      candidates: FixedBackoff::next_delay (sole implementor in workspace)

 L1b FILE EDGES (roll-up + direct)
   src/lib.rs          ──declares_mod──►  src/policy/mod.rs · src/client/mod.rs                         weight 2
   src/lib.rs          ──reexports──►     src/client/mod.rs                                             weight 1
   src/client/mod.rs   ──uses──►          src/policy/mod.rs           (evidence: use crate::policy::RetryPolicy)
   src/client/mod.rs   ──depends_on──►    src/policy/mod.rs           weight 2  kinds {field_type, may_call}  conf [0.93, 1.00]
   src/policy/mod.rs   ──declares_mod──►  src/policy/fixed.rs
   src/policy/fixed.rs ──depends_on──►    src/policy/mod.rs           weight 1  kinds {implements}

 L1a FOLDER EDGES (roll-up)
   src/client/  ──depends_on──►  src/policy/      weight 3  kinds {uses, field_type, may_call}  conf [0.93, 1.00]
   src/         ──contains──►    src/policy/ · src/client/

 L0  retry  ──depends_on──►  (none external)
```

What the agent sees, in order, under a budget: the folder view (two folders, one edge, ~60 tokens); expanding it, the file view (five files, six edges, ~250 tokens); expanding `client/mod.rs → policy/mod.rs`, the item edges that made it, with the one model-produced `may_call` marked by its confidence and its candidate list. `impact RetryPolicy` reads upward from the trait: one implementor, one field type, one dyn call site — and the coverage banner reports that nothing in this crate was abstained.

A proposed edge sits in the same picture: if the agent plans `pub struct ExponentialBackoff` implementing `RetryPolicy`, `propose` writes `policy::ExponentialBackoff#struct ──implements (proposed, author: agent-LLM)──► RetryPolicy#trait`, it rolls up as a dashed `fixed.rs`-sibling edge into `src/policy/`, and it becomes `realized` the first time the extractor finds the real impl.

#### B13.6 What each level is for

```text
 level    question it answers fastest                           typical size for a 300k-line workspace    default token cost
 L0       what crates, features, cfg am I in?                    30–80 crates                              ~200
 L1a      which parts of the tree touch each other?              100–400 folders, ~1–3k edges             ~500 (top-k by weight)
 L1b      if I change this file, which files can notice?         1–5k files, ~10–50k edges                ~1k (neighbourhood)
 L2       what does this file / item promise to the rest?        10–50k pub items, ~50–200k edges         ~3k (one anchor + radius 1)
 L3       who actually calls / constructs / reads this?          ~100k–1M sites                           ≤ 20k (one deep dive)
```

The levels are views over one set of records, not separate graphs. That is the whole design: the tiny model and the scanner write leaf records once; folders and files are sums; the agent descends only as far as its question requires.

### B14. Can a 100 MB model reach 98 % of what the deterministic tools give? — five timelines

*The question as asked: take a 100 MB model, train it on the top-100 Rust repositories and their compiler-labelled outputs, forty epochs, and ask whether it reaches 98 % of what rust-analyzer gives. Two answers, because the target matters: against "everything LSP returns" the answer is no, for structural reasons; against the multi-level code map (Deep Insight 06) the answer is yes on precision, with coverage as the honest second number. Simulated as timelines; assumptions stated; likelihoods are judgments.*

#### Decision frame

- **Fork:** train a 100 MB (≈ 100M-parameter, int8) model on ~100 repositories × teacher labels × 40 epochs, and rely on it to replace deterministic answers at 98 %.
- **Win:** ≥ 98 % agreement with the compiler on *held-out* repositories — never the 100 — at every level of the map.
- **Constraints:** 100 MB; input is source text (plus whatever context we choose to add); output is records.
- **Assumptions:** the top-100 Rust repositories are ~15–25 M lines ≈ 200 M tokens; 40 epochs ≈ 8 B training tokens (~80 tokens per parameter — ample compute, but forty passes over the *same* text); teacher labels are near-perfect and unlimited, since any crate that compiles yields them.
- **Failure:** 98 % on the training repositories, 80 % on the next one — memorisation.

#### Timeline A — Train exactly as described: text in, edges out

- **Week 1:** L1 and L2 converge fast — `mod`, `use`, `pub fn` are regular grammar; 99 %+ by epoch three.
- **Month 1:** held-out method-call resolution plateaus at ~70–85 %. Receivers need types that live in *other* crates' signatures; the model has only seen them for the 100 repositories' dependencies. Epochs 10–40 raise training accuracy, not held-out accuracy. The model begins to "know" that `.send()` is `tokio::mpsc` because it was, four hundred times.
- **Quarter 1:** shipped as-is, it emits *confident wrong edges* — the worst failure a structural tool can have. Abstention is bolted on afterwards and does not fit.
- **Likelihood of 98 % everywhere:** very low. **Stress:** a leaderboard that is excellent on the 100 and collapses on repository 101.

#### Timeline B — Same model; it classifies among retrieved candidates

- **Opening move:** change the task. For each ambiguous site the index and the feed supply the k candidate targets *with their signatures*; the model picks one or abstains. Closed-set classification, not open generation.
- **Month 1:** held-out method resolution rises to ~90–95 %, because the missing information is now in the input. Cross-crate resolution works for crates the model never saw — the feed carries their signatures.
- **Quarter 1:** the residue is the genuinely hard set: generic dispatch, identically named trait methods on blanket impls, macro-generated items. 98 % is in sight per edge kind.
- **Likelihood:** medium-high. **Inflection:** retrieval quality — if the right target is not among the k, the model cannot choose it.

#### Timeline C — Encoder with heads, in a cascade, abstention as a first-class output

- **Opening move:** rules → 2–5 MB tagger → 100 MB encoder-binder over B's candidates, trained with an explicit "none of these / insufficient context" class; scored as precision *at a coverage*.
- **Month 1:** precision on resolved edges ≥ 98 % at ~85–90 % coverage; the remainder are `abstained` with spans. The only timeline where 98 % is literally reached — as a precision figure with coverage printed beside it.
- **Quarter 1:** coverage climbs as the feed fills and the tagger improves: L1/L2 ~99 %; L3 precision 98 % at ~90 %.
- **Likelihood:** high for the precision target; coverage is the open question. **Stress:** users who want one number, not two.

#### Timeline D — Change the data recipe: 2,000 repositories × 2 epochs, distilled

- **Opening move:** the same compute as 100 × 40, spent on breadth: thousands of crates from the feed, two or three epochs, rename and alias augmentation, logits from a large code model as a soft teacher beside the compiler's hard labels.
- **Month 1:** the held-out gap nearly closes — generalisation rather than memorisation. Pruning and distilling a 0.5B code model down to 100M beats training 100M from scratch; the published pruning-plus-distillation results transfer (fewer tokens, better scores).
- **Likelihood:** high as an improvement on any of A–C. This is the training recipe, not a product path.

#### Timeline E — Chase 98 % on everything LSP returns: local types, generic instantiations, macro-expanded items

- **Month 1:** the model is re-implementing type inference from text. Local types depend on return types of calls in other crates, under generics, after macro expansion that is not in the text at all.
- **Quarter 1:** it works on the 100 repositories and nowhere else. Stop. These are the compiler's answers; `cargo check` remains the agent's separate tool.
- **Likelihood of 98 %:** near nil at 100 MB; low at any size without compiler-shaped machinery.

#### Cross-timeline analysis

```text
 path                           held-out L1/L2   held-out method calls             confident-wrong risk   reversible   regret
 A  text → edges, 100 × 40       99 %             70–85 %, memorises                HIGH                   yes          high — fine until repo 101
 B  classify among candidates    99 %             90–95 %                           medium                 yes          low
 C  cascade + abstention         99 %             98 % precision @ 85–90 % coverage LOW — it abstains      yes          low — the honest 98
 D  breadth + distillation       +                +3–8 points on any of the above   —                      yes          none — it is the recipe
 E  everything, incl. types      —                no                                HIGH                   —            high
```

Why it may work: Rust names are static; signatures carry the types; `use` paths qualify most items; roughly three quarters of edges are syntactic; candidates are retrievable exactly; labels are free and perfect. Why it may not: about 65 % of call sites are method calls and only about a quarter of their receivers are typeable from the enclosing function; proc-macro output is not in the text; generics and blanket impls need global reasoning; forty epochs over a hundred repositories memorises; and a 17 MB encoder attempt learned zero accepted bindings — size was not the only reason, task framing was.

#### Decision filter

- **Strongest if things go normally:** C, built on B's input, trained D's way — 98 % precision with coverage printed beside it; L1/L2 at 99 %.
- **Safest if things go badly:** C — abstention turns failure into coverage rather than wrong edges, and the scanner alone still yields L1/L2.
- **Fastest collapse of uncertainty:** rung two of the training ladder — 10 M tokens, held-out repositories, a **precision-versus-coverage curve** for a 30M and a 100M binder with candidates in the input. One afternoon, under a dollar of GPU. If the curve reaches 98 % precision above 80 % coverage, the thesis holds; if it cannot reach 98 % at any coverage, the model is a tagger and the resolver stays rule-plus-feed.

#### The reframing: the output is the multi-level code map

Against the map as defined in Deep Insight 06 — L0–L3 records with typed edges, roll-ups, and `abstained` as a legitimate output — timeline E disappears and the 98 % becomes well-defined:

```text
 level / record                      what the model must get right                   98 % reachable?       how
 L0  crates                          —                                                100 %                 cargo metadata; no model
 L1a folders                         —                                                = L1b                 roll-up
 L1b file edges                      mod / use / include / cfg; glob and macro imports 99 %+                 scanner; model on globs
 L2  item nodes and fields           kind, visibility, signature, doc, cfg, span       99 %+                 scanner; model only for macro items
 L2  item edges                      bind the types named in signatures               98–99 %               candidates from index/feed → classify;
     (signature_uses, implements,                                                                            these names are almost always
      field_type, reexports …)                                                                               `use`-qualified — an easy closed set
 L3  calls / constructs / reads      resolve receivers; dyn and generic dispatch       98 % PRECISION at     abstention carries the rest; coverage
     (the bulk of the map by count)                                                   ~85–90 % coverage     climbs with the feed and the binder
 roll-ups · confidence ranges ·      arithmetic over leaves                            100 %                 deterministic
 status · abstained placeholders
```

By the map's own definition — where an unresolved site is a *correct* `abstained` record, not a miss — **a 100 MB binder can reach ≥ 98 % precision on every extracted record, with L0–L2 near 99 % and L3 coverage as the honest second number (~85–90 % at first, rising).** What it cannot do is make L3 *recall* 98 % on day one; by edge count L3 dominates, so overall agreement with the compiler begins near 90–93 % recall at ≥ 98 % precision. That is a usable map, and the coverage banner says exactly where it stopped.

Two caveats survive the reframing. The recipe still matters more than the size: 100 × 40 still memorises, and held-out L3 precision will look like 98 % on the 100 and fall on repository 101 unless the input is candidates with signatures (B) and the data is breadth-first and distilled (D); the map does not change that, it makes the failure visible as coverage rather than as wrong edges — provided abstention is trained in (C). And `macro_generates` plus effective visibility through re-export chains are the L2 records most likely to sit at `abstained`, because proc-macro output is not in the text; the map allows that, so it costs coverage, not precision.

The honest claim, with the map as the target: **≥ 98 % precision, coverage printed** — achievable at 100 MB. The rung-two precision-versus-coverage plot is still the one-afternoon test that turns the claim into a number.

### B16. Span-first: index the public spans, derive the graph later

*A tiny tagger cuts source into spans of public items and writes them into one embedded table before any edge exists; edges are derived afterwards by SQL joins, by a model on the ambiguous rows, and by the agent reading spans. The full statement is Deep Insight 07; this is the short form and why it belongs here.*

**What changes**

```text
 usual order     text ──model──► edges ──► store ──► views
 span-first      text ──tiny tagger──► spans (kind · name · path · visibility · file · range · text · digest)
                 ──► one table ──► edges derived at query time or in the background
```

The span is the primitive; items are rows; edges are views. The leaf records of the multi-level map become spans, and the roll-ups compute unchanged. What flips is the order of work: index first, analyse later.

**Why it is possible and cheap**

- Public items are ~1 % of lines: a kernel-sized Rust tree is ~3 M spans and 100–300 MB of text; one file.
- Segmentation is a 2–5 MB tagger's job — boundaries, kind, name — no binding, no judgment. Kernel-size on an 8 GB laptop in ~20 minutes; a real workspace in seconds.
- Identity is the span digest: unchanged items are the same row across commits; a move is the same digest at a new path; an update is a row replace.

**What the analysis becomes**

- A `use crate::policy::RetryPolicy` and a span named `RetryPolicy` under `policy/` join into a candidate edge in one SQL statement — exact when the name is unique in scope, `inferred` with a confidence otherwise. That join is the binder's candidate list, so the model runs only on the rows the join could not settle.
- Full-text search over span text makes `find` grounded: the agent gets the actual signature and doc comment.
- The agent reads spans directly when that beats computing an edge — the default path, with the graph as acceleration.
- `impact` and roll-ups materialise from the edge table as needed.

**What it does not change**

- The method-call problem: receivers still need typing; L3 still needs the model, still lazily.
- Private glue: anchors need the path from a private hit to its public parent, so private items stay as light rows (kind, name, file, range, no text).
- False positives from name joins are stored as `inferred`, never `extracted`.

**Why it belongs in this thesis**

It is the cheapest form of the floor product: exact at the span level, on CPU, in minutes, with the model spent only where joins stop. It also makes the three-arm pilot cheaper — the middle arm can be spans alone, before a single edge exists.
