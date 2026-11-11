# Fusion 01

*A working note on where tiny LLMs fit in Parseltongue. Self-contained. Numbers marked (M) were measured in this project; every likelihood is a judgement, and no model has been trained yet.*

## Hypothesis 01 — a post-trained tiny LLM can choose an edge it is shown; it cannot recall one

**Statement.** A tiny LLM (tens to a few hundred MB) can be post-trained to find dependency edges if, and only if, the correct target is already inside its input. The CPU builds a short list of candidates; the model picks one or abstains.

**How to prove it wrong.** Either of these kills it:
- the true target is often missing from the candidate list the CPU can build, or
- the model does not beat plain rules on repositories it has never seen, at the same wrong-edge budget.

### First principles: what a tiny model has to work with

```text
 it HAS                                                    it does NOT have
 ───────────────────────────────────────────────────────── ─────────────────────────────────────────────────────────
 Rust syntax and idioms, from pre-training                 your repository: it was never trained on it
 the public API of std and very popular crates, roughly    room for the whole repo: 1.5 M spans do not fit in a window
 whatever text is placed in its input window               a type checker: it estimates, it does not prove
 a procedure it was post-trained on ("pick one or abstain") a way to know it is wrong, unless it reports confidence
```

So post-training can teach a *procedure*. It cannot teach *facts about your code*. Facts must arrive in the input.

### Possibility, per use case

| Use case | Possibility | Why |
|---|---|---|
| Binding a method call to its target | Likely, if candidates are in the input | Exact, unlimited labels from rust-analyzer run offline; a closed choice. Evidence so far is one mixed experiment: no accepted bindings on five crates, seven confirmed on a fresh set. |
| Search embedding and ranking | Very likely | Standard technique; needs only matched pairs. |
| Identity and duplicates (is this the same item, renamed?) | Likely | Pair classification; rename commits in git history are free labels. |
| Repairing grey spans in broken files | Likely, low value | Labels are free (break good files on purpose), but the indentation rule already does most of it. |
| Guessing what a macro defines | Uncertain | Open-ended output; proc macros run arbitrary code; labels need builds. |

### Decision frame

- **Fork:** how to cut the problem so a tiny model can learn it.
- **Outcome wanted:** call edges at high precision, with abstention, on unseen repositories.
- **Hard constraints:** no rust-analyzer at runtime; 8 GB machine; one small model in memory at a time.
- **Horizon:** week 1, month 1, quarter 1.
- **Failure:** a model that does not beat plain rules, or that adds wrong edges silently.

### Timeline A — multiple choice

- **Opening move:** run rust-analyzer offline on about five repositories. For each method call, record the true target and the candidate list the CPU would offer.
- **Week 1:** before any training, measure the CPU-only hit rate and how often the true target is in the list. Train on 1 M tokens.
- **Month 1:** train to 10–100 M tokens; score on unseen repositories against the rule baseline.
- **Quarter 1:** add a loop: bind the first call, read its return type from the signature, narrow the next call, repeat.
- **Long-term shape:** three to five small specialised models.
- **Likelihood:** about 60 % that it clearly beats rules.
- **Stress points:** mostly dataset plumbing; chains stall on generic return types.
- **Inflection point:** if the true target is often missing from the list, no model can fix it.

### Timeline B — teach types, derive edges

- **Opening move:** export (expression, type) pairs from rust-analyzer. Every hard case reduces to "what type is this expression", and these labels are denser than edge labels.
- **Week 1:** measure how many types the CPU gets by propagation alone.
- **Month 1:** the model fills one unknown type from a candidate list, the CPU propagates, repeat.
- **Quarter 1:** generics and trait methods are the test.
- **Long-term shape:** one model with one skill, a learned approximate type checker.
- **Likelihood:** about 35 % that it beats A.
- **Stress points:** one wrong type poisons everything after it in that function; hard to debug.
- **Inflection point:** whether nested types can be chosen piece by piece rather than written out as text.

### Timeline C — one embedding, many uses

- **Opening move:** train one embedding model so matching pairs land close: question and item, call site and target, stub and implementation.
- **Week 1:** test an off-the-shelf small code embedder on search, with no training.
- **Month 1:** fine-tune; store one vector per public item at index time.
- **Quarter 1:** it serves search, ranking, duplicates and identity.
- **Long-term shape:** the meaning layer of the map; cheapest at runtime.
- **Likelihood:** about 80 % useful for search; about 15 % enough for binding. `Mutex::lock` and `SpinLock::lock` mean the same thing, and only the receiver's type separates them.
- **Stress points:** easy to demo, so it can feel like progress while call edges stay unsolved.
- **Inflection point:** whether it beats plain text search for agents.

### Cross-timeline analysis

| Path | Upside | Downside | Reversibility | Regret risk | Needs |
|---|---|---|---|---|---|
| A | Checkable output, calibrated abstain | Several models to maintain | High | Low | Good candidate lists |
| B | Highest ceiling | Compounding errors | Medium | High if it stalls | Dense type labels |
| C | One model, many uses | Does not solve binding | High | Low | Query and item pairs |

### Decision filter

- **Strongest if things go normally:** A for binding plus C for search.
- **Safest if things go badly:** C plus CPU-only binding with abstain. The map still has signature edges, search and intent.
- **Fastest experiment:** build one labelled set from rust-analyzer offline; it serves all three paths. Measure the CPU-only hit rate and the candidate-list ceiling before training, then train A at 1 M and 10 M tokens. Try B only if A stalls on generics.

## The workflow table, explained for a layman

The table is a recipe with fifteen steps. Each row is one step. Each column answers one question about that step.

```text
 column                              the question it answers
 ─────────────────────────────────── ──────────────────────────────────────────────────────────────────────────────
 step                                which step is this
 what happens                        what work gets done
 who does it → writes                which program does the work, and which table it fills in
 inspiration (use)                   whose code or idea we borrowed.  ✔ = we use their code   ○ = we copy only the idea
 tiny LLM in this step · when        is a small AI model involved, and under exactly what condition
 tiny-LLM estimate · in the 30 min?  how long it takes on a MacBook Air, and whether you wait for it (IN) or it runs later (BACKGROUND)
```

Picture the codebase as a huge library of pages, and Parseltongue as the librarian building a card index.

```text
 BUILDING THE INDEX  (steps 0–7, done once, then only for pages that change)

 0  photocopy every page exactly as it is, and stamp each copy with a fingerprint          no AI
    (same fingerprint next time = page unchanged = skip it)
 1  draw a box around every item on the page: each function, each type                     AI only for torn pages
    a torn page (broken code) still gets boxes, but the doubtful ones are coloured grey
 2  for every box, fill in an index card: its name, what it takes in, what it gives back   no AI
 3  inside every box, put a sticky note wherever the text says "now go and use X"          no AI
 4  match the name on each sticky note to the index card with that name                    no AI
    one card matches = done · several match = shortlist · none match = unknown
 5  for each shortlist: simple rules try first; if they are stuck, the small AI picks      AI, only on shortlists
    one card, or says "not sure". It never guesses silently.
 6  draw the big map: only the public boxes, with a line wherever one uses another         no AI
 7  print a scorecard: how much is mapped, how much is grey, how much is still unknown     no AI

 ── the index is usable here. You waited roughly 10–20 minutes for 10 million lines (an estimate; only the
    page-reading speed is measured). The AI helpers work afterwards, in the background.

 USING THE INDEX  (steps 8–14, every day)

 8   the coding agent asks: "where is X?"  "what breaks if I change X?"                    AI optional: sort a long answer
 9   the agent or a human writes down what SHOULD exist: new box, new line, line to remove no AI
 10  the agent reads the gap list: wanted but not built yet                                no AI
 11  the agent writes the code for exactly those gaps                                      (the agent itself)
 12  only the changed pages go through steps 0–7 again                                     same AI helpers, changed pages only
 13  the gap list ticks itself off when the wanted thing shows up in the real index        no AI
 14  a human compares the wish list with reality                                           no AI; corrections train the AI
```

What the two tiny-LLM columns say, in one line: of fifteen steps the small AI touches three (1, 5 and 8), always after plain rules have tried, and never while you are waiting for the first map.

## Can a tiny LLM find edges by reading a list of spans?

Short answer: not by reading the list. Yes, one reference at a time, if the right candidates are put in front of it. The three questions, in turn.

### 1. Can it find edges from a list of spans?

```text
 why "read the list, write the edges" fails
   a 10 M-line tree has about 1.5 M spans. A tiny model reads a few thousand tokens at once.
   edges are pairs. 1.5 M spans make about a trillion possible pairs. Nobody can check them one by one.
   a list of signatures does not say what the bodies call. The call is written inside the body.

 what works instead: turn it around
   start from one reference in one body:        ctx.lock()
   the CPU fetches every span named `lock`:     Mutex::lock · SpinLock::lock · RwLock::lock
   the model sees the body plus those three and picks one, or abstains
```

So the model never searches. The CPU searches by name, and the model judges. Edges that live in signatures (a function takes or returns a type) need no model at all: the type name is written there and is matched by path.

### 2. Can it tell which span depends on which?

The direction is free. The span that *contains* the reference depends on the span that is *referred to*. The parser already knows which box each reference sits in. The only open question is the identity of the target, which is question 3.

### 3. When a span says `function01`, does it know which `function01`?

A tiny model does not know this on its own, and mostly it does not have to. Rust decides it by fixed rules, and most of those rules can be followed on the CPU from text alone.

```text
 how Rust decides what a plain name means                         who can follow the rule
 ──────────────────────────────────────────────────────────────── ─────────────────────────────────────────────
 1  a local variable or an item declared in the same block         parser
 2  an item in the same module, or brought in by a `use` line      parser + the list of modules and files
 3  an item brought in by a glob import, `use foo::*`              CPU, if foo's items are indexed; otherwise unknown
 4  the standard prelude: Vec, Option, String, Box, Some, Ok …     a fixed, small table
 5  a crate name listed as a dependency in Cargo.toml              read Cargo.toml (text, no build)
```

```text
 where function01 can come from          how we can tell, without a compiler                              AI needed?
 ─────────────────────────────────────── ──────────────────────────────────────────────────────────────── ──────────────
 another file in the same crate          `use crate::a::function01` or `mod` lines + the module list      no
 another crate of yours (workspace)      `use other_crate::…` + Cargo.toml says it is a path dependency   no
 an external library                     `use serde::…` + Cargo.toml says it comes from crates.io         no, for "which crate"
 the standard library                    `use std::…`, or the name is in the prelude table                no
```

For calls written with a path, like `function01()` or `retry::function01()`, origin is a lookup. The share of path calls that resolve this way has not been measured yet.

The hard case is the method call, `x.function01()`. There is no path to follow. The answer depends on the type of `x`, and sometimes on which traits are in scope: `.context()` on a `Result` comes from the external crate `anyhow`, not from std. Method calls are 66 % of call sites (M). This is where a tiny LLM earns its place.

What the model brings from pre-training is a prior, not knowledge of your code: it has seen `.unwrap()`, `.iter()`, `.push()` and `tokio::spawn` many times and will lean the right way for std and popular crates. For rare crates and for your own items it has nothing, so those must be in its input.

### Creative ways to make the job smaller

1. **Sort by origin before binding.** If a method name is defined nowhere in your repository, the target is std or an external crate. For a map of *your* public interface that is a boundary edge ("uses crate X"), and the exact method need not be resolved. Only names that exist in the repository go to the binder. How much this removes is unmeasured and cheap to measure.
2. **Open-book exam.** Never ask the model to recall. Always show it the candidates' signatures. Post-training then teaches reading, not memorising.
3. **Decide per variable, not per call.** Ask once "what type is `ctx`", then every `ctx.…` call in that function follows on the CPU.
4. **Check the pick.** After the model chooses, the CPU verifies that the argument count and the `self` form match the chosen signature. A pick that fails the check becomes an abstain.
5. **Index dependency sources without building.** Cargo keeps the source of fetched dependencies on disk. The same parser can index their public items, so glob imports and external targets become lookups. Not tried here yet. A prebuilt public index of std and popular crates would do the same job.
6. **Use the embedding for recall only.** Nearest neighbours say "related", not "depends on". They are useful to propose candidates the name search missed, never as the edge itself.

### What this means for Hypothesis 01

The model's task is narrow and well posed: for an in-repository method call whose receiver type the CPU could not work out, choose among a handful of shown signatures or abstain. Whether it does that well is the open question, and the two measurements under "Fastest experiment" above decide it before any training is paid for.

## Hypothesis 02 — a tiny LLM can be taught to drive the span search itself

**Statement.** Instead of the CPU handing the model a fixed shortlist (Hypothesis 01), the model runs the search. It reads a span, asks the span search a question, reads the rows that come back, and then picks a target, searches again, or says "not sure". Proposed name: `tiny-llm-span-searcher`.

**How to prove it wrong.** Any of these kills it:
- on unseen repositories it does not find more correct targets than the fixed shortlist of Hypothesis 01, at the same wrong-edge budget;
- it needs more than about three searches per reference to get there;
- it cannot be kept to valid queries without a larger model than the 8 GB machine allows.

Nothing in this project has tested this yet. Every likelihood below is a judgement.

### What it looks like

```text
 span says:   self.client.send(req).await?.json()

 model asks:  search field `client` in this struct        → client: HttpClient
 model asks:  search fn `send` on HttpClient              → HttpClient::send(..) -> Result<Response>
 model asks:  search fn `json` on Response                → not in repo; `use reqwest::Response` → external: reqwest
 model says:  edge to HttpClient::send (yours) + boundary edge to crate reqwest
```

Each line is one turn: the model writes one small query, the span search (a database lookup, not a compiler crate) returns a few rows, and the model reads them before deciding its next move.

### Why it could beat a fixed shortlist

| Case | Fixed shortlist (Hypothesis 01) | Model-driven search (Hypothesis 02) |
|---|---|---|
| Chains, `a.b().c()` | Stalls after the first call unless the CPU loop handles it | Each call's return type tells the model what to search next |
| Re-exports and glob imports | Target may be missing from the list | Can follow a `pub use` to the real item |
| Trait methods | Needs the right trait already in the list | Can search for traits in scope that have that method name |
| External libraries | Not in the repo's span table | Can tell "yours" from "theirs" from `use` lines and `Cargo.toml` |

### What it needs

- **A span search.** The name and full-text index already in the design. Queries the model may write, kept to a tiny grammar:

```text
 find item   <name>                    any span with this name
 find method <name> on <type>          a method of a given type
 find field  <name> in <type>          a field of a given struct
 find trait-method <name>              traits that declare this method
 find use    <name> in <file>          the import line that brings a name in
 find crate  <name>                    is this a workspace crate, a dependency, or std
```

- **External libraries made searchable.** Either index the dependency sources Cargo keeps on disk with the same parser, or ship a prebuilt index of popular crates. Without one of these the model can name the crate but not the exact item. Neither has been tried here.
- **Training data.** Run rust-analyzer offline to get the true target of each reference. From the span tables, generate the shortest sequence of searches that reaches that target. The model learns to imitate those sequences. Labels are automatic; no human writes them.

### Risks

- **Small models wander.** Cap it at three searches per reference and constrain its output to the query grammar above, so it cannot write an invalid search.
- **Errors compound.** A wrong first hop spoils the rest, so it must be allowed to abstain at any hop, and the CPU should check the final pick against the chosen signature (argument count, `self` form).
- **It is slower.** Writing queries costs more than picking from a list. It should run only on what plain rules and the fixed shortlist leave over, in the background.
- **It needs a generating model.** Picking from a list can be done by a small encoder. Writing queries needs a decoder, which is larger and slower for the same quality.

### Likelihood

Roughly even odds that a model of a few hundred MB learns one-to-three-hop search reliably. Lower for chains through generic return types and for trait methods on external types.

### How it relates to Hypothesis 01

```text
 reference in a span
        │
        ▼
 plain rules (paths, use lines, prelude, uniqueness) ── resolved → edge
        │ not resolved
        ▼
 Hypothesis 01: fixed shortlist, model picks ─────────── resolved → edge (inferred)
        │ target not in the shortlist, or model abstains
        ▼
 Hypothesis 02: model searches, up to three hops ─────── resolved → edge (inferred)
        │ still not sure
        ▼
 abstained: the row stays visible and counted
```

The two are layers, not rivals. Hypothesis 02 is only worth building for the references Hypothesis 01 cannot reach.

### The measurement that decides how much this matters

The experiment already planned under Hypothesis 01 answers it before any training: on a labelled set from rust-analyzer, how often is the true target inside the simple name-search shortlist?

- If it is usually there, a fixed shortlist is enough and Hypothesis 02 is a small refinement.
- If it is often missing, model-driven search is the way to reach it, and the missing cases (sorted by cause: chain, re-export, trait method, external) say which query kinds to teach first.

### Model candidates for Hypothesis 02

Sizes and licences below were read from the Hugging Face model pages on 2026-11-09. Suitability is a judgement; none of these has been trained or run for this task here.

| Model | Parameters | Licence | Kind | Why consider it |
|---|---|---|---|---|
| Qwen2.5-Coder-0.5B-Instruct | 494 M | Apache-2.0 | writes text | Pre-trained on code; the smallest code model that can write queries |
| Qwen2.5-Coder-1.5B-Instruct | 1,544 M | Apache-2.0 | writes text | Stronger; better used as the teacher, or as the ceiling to compare against |
| SmolLM2-360M-Instruct | 362 M | Apache-2.0 | writes text | Smaller and faster; general, not code-specific |
| SmolLM2-135M-Instruct | 135 M | Apache-2.0 | writes text | The floor: does a model this small learn the query grammar at all |
| FunctionGemma-270M | 268 M | Gemma terms, gated | writes text | Built for calling tools, but the licence is not fully open |
| CodeT5+ 220M | 220 M (by name) | BSD-3 | reads then writes | Encoder reads the span, small decoder writes a short query |
| ModernBERT-base | 150 M | Apache-2.0 | reads only | Fast picker and scorer; cannot write queries |
| jina-embeddings-v2-base-code | 161 M | Apache-2.0 | embeds | Code-aware vectors for retrieval |
| CodeT5+ 110M embedding | 110 M (by name) | BSD-3 | embeds | Smaller code embedder |

"Writes text" models can produce a search query. "Reads only" and "embeds" models are smaller and faster, but can only choose or rank. Which kind is needed depends on the approach below.

### The training data, in general

One labelled set serves every approach. It is built offline:

1. Parse a repository into the span tables (the same pass the product uses).
2. Run rust-analyzer offline on the same repository and record, for every reference in every body, the true target and the type of every sub-expression.
3. From 1 and 2, derive the input and output that each approach needs.

Rules that hold for all five:
- Split by **repository**, never by file, so the score measures unseen code.
- Keep the "no answer" cases (target outside the repo, not resolvable) so the model learns to abstain.
- Score on the size and precision the model will ship at.

### Five approaches

**Approach 1 — the tool caller.** The model writes queries from the six-query grammar, one at a time, and reads the results.

```text
 model       Qwen2.5-Coder-0.5B, or SmolLM2-360M as the smaller option
 INPUT       the span text + the file's use lines + everything searched so far and what came back
 OUTPUT      one line: the next query, or  ANSWER <span id>,  or  ABSTAIN
 example in  span: self.client.send(req)   history: (none)
 example out find field client in Self
 data        shortest search sequence to the true target, generated by a script; one training row per step
```
Strength: the most general; handles chains and re-exports. Weakness: slowest; small models may wander.

**Approach 2 — the slot filler.** The model never writes free text. It labels words in the span, and plain code turns the labels into a query.

```text
 model       ModernBERT-base (reads only)
 INPUT       the span text with one reference marked
 OUTPUT      per word, a tag: RECEIVER · METHOD · TYPE-HINT · none      and a query kind out of six
 example in  let ctx: SpinLock<State> = …;  ctx.[lock]()
 example out METHOD = lock, TYPE-HINT = SpinLock, query kind = find method on type
 data        tags derived from the true target's type and the grammar; fully automatic
```
Strength: fast, cannot produce an invalid query. Weakness: one hop at a time; plain code must drive the loop.

**Approach 3 — the matchmaker.** No queries at all. The reference and every span are turned into vectors; the nearest spans are the candidates, and a small scorer picks.

```text
 model       jina-embeddings-v2-base-code or CodeT5+ 110M embedding, plus ModernBERT as the scorer
 INPUT       (a) the reference with its surrounding lines   (b) each span's signature
 OUTPUT      a vector for each; training pulls a reference close to its true target and away from same-named spans
 data        (reference, true target) pairs; the wrong-but-same-name spans are the hard negatives
```
Strength: one vector per span, computed once; search is a nearest-neighbour lookup. Weakness: `Mutex::lock` and `SpinLock::lock` look alike to an embedding; it needs the scorer to separate them.

**Approach 4 — the type writer.** The model answers one question, "what type is this expression", by choosing among type names that exist in the repo. Plain code then does the search.

```text
 model       ModernBERT-base to choose among listed types; CodeT5+ 220M if nested types must be written out
 INPUT       the span text with one expression marked + the type names visible in the file and its imports
 OUTPUT      one type name from that list, or UNKNOWN
 example in  [self.client]   visible types: HttpClient, Request, Response, RetryPolicy
 example out HttpClient
 data        every sub-expression in every body with its type, from rust-analyzer; very dense, millions of rows per repo
```
Strength: the densest labels; one answer per variable serves every call on it. Weakness: one wrong type spoils the calls after it; generics are hard.

**Approach 5 — the apprentice.** A larger model does the search first, and the small model learns from its successful attempts.

```text
 teacher     Qwen2.5-Coder-1.5B, or a larger model, run offline with the same six-query tool
 student     Qwen2.5-Coder-0.5B or SmolLM2-360M
 INPUT       same as Approach 1
 OUTPUT      same as Approach 1
 data        only the teacher's search sequences that ended on the true target (checked against rust-analyzer); failures are discarded
```
Strength: the sequences include natural detours and recoveries a script would never write. Weakness: costs teacher compute; the student inherits the teacher's habits.

### Side by side

| Approach | Model kind | Labels | Speed at runtime | Handles chains | Main risk |
|---|---|---|---|---|---|
| 1 Tool caller | writes text, ~0.5 B | scripted sequences | slowest | yes | wandering |
| 2 Slot filler | reads only, ~150 M | automatic tags | fast | via a code loop | too rigid |
| 3 Matchmaker | embedder + scorer | pairs | fastest | no | same-name confusion |
| 4 Type writer | reads only, ~150 M | densest | fast | yes, via types | compounding errors |
| 5 Apprentice | writes text, ~0.5 B | teacher's successes | slowest | yes | teacher cost |

### Which to try first

Approaches 2 and 4 are the cheapest to test: small read-only models, automatic labels, and a clear score. Approach 1 is the fullest form of the hypothesis and should be tried once the labelled set exists. Approach 3 is worth having anyway because the same vectors serve `find`. Approach 5 is a way to improve 1, not a starting point.
