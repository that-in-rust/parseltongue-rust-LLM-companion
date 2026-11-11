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
