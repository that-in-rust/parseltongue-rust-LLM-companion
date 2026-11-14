# Thesis: resolver builds, LLM steers. "LLM + search tool" beats the parser for *navigating* a multi-level dependency graph, and loses to it for *constructing* one

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Answers the question: "if my job is primarily to create a dependency graph at multiple levels, am I not seeking an LLM which only takes decisions, made bidirectional with a search tool, and won't the net result of LLM + search beat the parser?" Evidence is drawn from the measurements in `1793864191-…`, the limits in `1793873685-…`, the seven-event protocol in `1793879701-…`, the weight-space note `1793877276-…`, and the user's own `experiments/tiny-reexport/learned-binding/`.*

---

## 1. The thesis in one paragraph

"Create a dependency graph" is two jobs with opposite winners. **Job 1, produce the facts** (who calls whom, who implements what, what is public, at crate / module / item / body level): a deterministic resolver wins, because the graph must be exhaustive, exact, cheap to rebuild on every edit, and trustworthy enough that an agent can treat every id as real. An LLM that "only takes decisions", even with a search tool, builds a graph by sampling and judgment, which is the wrong instrument for a lookup problem. **Job 2, navigate the graph to answer a question**: an LLM making decisions, bidirectionally coupled to a search tool that returns small exact structure, beats both a parser-only dump and an LLM reading files, because the parser cannot choose what matters and the LLM cannot be exhaustive. The product is therefore **LLM + search tool *on top of* the resolver**, never *instead of* it. The resolver is what makes the search tool worth steering.

```text
                 ┌─────────────┐   decides what to ask, what to open, when to stop
                 │     LLM     │   (never asked to compute an edge it could look up)
                 └──────┬──────┘
          query ▲      │ small, exact, budgeted answers
                │      ▼
                 ┌─────────────┐   find / anchor / cluster / impact / path / diff
                 │ search tool │   (the hinge: makes the resolver's facts steerable)
                 └──────┬──────┘
                        │ reads
                        ▼
                 ┌─────────────┐   L1–L4 records, every id real, every edge tagged exact
                 │  resolver   │   (rust-analyzer-derived; tree-sitter fallback tagged
                 └─────────────┘    syntactic; LLM proposals tagged confidence<1)
```

## 2. Why the LLM loses Job 1 (constructing the graph)

| Reason | Evidence (all from this session unless cited) |
|---|---|
| **Exhaustiveness.** A graph needs *every* edge, not a sample. 10 M LOC ≈ 500k items. | rust-analyzer: 42k-LOC, 11-crate workspace fully resolved in 29 s / 2.3 GB, 0.02 % unknown types with proc macros on (`1793864191-…` §2.1, `1793873685-…` §1). An LLM making one decision per item at ~500 tokens ≈ 250 M tokens per full build, re-run after every edit; the resolver re-derives only the changed body (salsa invariant: "typing inside a function body never invalidates global derived data"). |
| **The hard 65 % are lookups, not judgments.** | 65 % of call sites are `recv.method()` whose target depends on the receiver's type; ~75 % of those receivers are typed by information in *another* file (`1793864191-…` §2.3). 44 % of all references resolve outside the workspace (30 % std, 14 % external crates; §2.4). "LLM + search" would search for the type, read the signature, decide — that *is* name resolution, performed slowly and with sampling noise. |
| **Shipped evidence says so.** | Cassini-1.0 (purpose-trained Qwen3-4B extractor): 77.6 % F1 on Py/JS/TS and still hands its JSON to a deterministic resolver. The user's `learned-binding/pretrained`: fused MiniLM ranker accepted **0** bindings on five real crates where a same-module rule confirmed **56**. Literature in the thesis `1793858459-…`: static tools beat 24 LLMs at call-graph construction (Venkatesh et al. 2024; Su et al. 2025). |
| **Trust is a property of the construction method.** | The agent-facing promise is "every id is real, every edge tagged exact". A graph built by decisions has no such property; the resolver would be needed anyway to verify it, so build with it directly. |
| **Rust makes the resolver cheap and local.** | Lexical visibility, static name resolution, coherence (one impl per type×trait), explicit `dyn`, typed signatures, crates as versioned units (`1793853341-…` §5). The resolver is a parser that finishes the job, not "the compiler" (`1793864191-…` §1). |

## 3. Why the LLM wins Job 2 (navigating the graph)

| Reason | Evidence |
|---|---|
| **The parser cannot choose.** 500k items; a question touches 20. Relevance is a judgment about the *question*, which no static tool has. | The agent literature converged on budgeted projections of an exact graph rather than raw files (CodePlan, RepoGraph, CodexGraph, CGM, RepoAtlas, RIG — `1793858459-…`). Tree-sitter-only graphs over MCP cut tokens 10× but lost 9 points of answer quality to file exploration (Codebase-Memory 2026) — the loss comes from inexact edges, not from having an LLM steer. |
| **The LLM cannot be exhaustive or exact, but doesn't need to be when the tool is.** | Seven-event protocol (`1793879701-…`): ≤ 32k tokens for a fully explored question in 10 M LOC, every entity real. The LLM's decisions are *which 4 anchors, which cluster, which bodies* — cheap decisions over exact candidates. |
| **Bidirectionality is the point.** | Each round-trip narrows: query → 4 anchors → 1 cluster → 20k tokens of exact context → next query, with grounded example queries generated from real ids. This loop is strictly better than either side alone: parser-only has no stopping rule, LLM-only has no ground truth. |

## 4. Where the LLM *does* add graph content

Only the residual the resolver cannot produce, always as proposals with `confidence < 1`, validated by the resolver when it can run:

1. **L6 meaning** — what a module is *for*, which invariant an `unsafe` block protects, which of 40 public items are the real entry points, lock-ordering intent. No static tool has this; it is the layer where the LLM is not competing with an exact tool (`1793864191-…` §4, §6).
2. **No-cargo fallback** — snippets, diffs, broken manifests, foreign build systems: tree-sitter gives L2 exactly, the LLM proposes L3/L4 (Cassini regime), tagged `substrate: llm`.
3. **Hints on the resolver's blind spots** — `dyn`/generic fan-out beyond the locally known implementer set; items under inactive `cfg`; proc-macro output when the build step has not run (`1793873685-…` §4).

## 5. The experiment that settles it (cheap)

Take one crate that rust-analyzer loads (e.g. `cargo-public-api`, 9k LOC). Pick 20 public items. Give an agent only `grep` + `read_file` and ask it to produce the public→public call edges (with `via` for private hops) for those 20 items. Compare with rust-analyzer's edges for the same items.

| Metric | Prediction |
|---|---|
| Edge F1, agent with grep/read | < 70 % (method-call targets and re-exports will be missed or guessed) |
| Edge F1, rust-analyzer | ≈ 100 % for static calls; `dyn`/generic edges reported to the trait method |
| Tokens spent, agent | > 50k |
| Tokens spent, resolver | 0 (29 s CPU for the whole workspace) |
| Then: agent given the seven-event search tool over the resolver's graph, asked a *question* about the crate ("what breaks if `Builder::build` changes its return type?") | correct in ≤ 10k tokens; same question with grep/read only: more tokens, incomplete caller list |

If the grep/read agent reaches ≥ 95 % edge F1 at comparable cost, this thesis is wrong and the resolver is optional. If it lands where predicted, Job 1 belongs to the resolver and Job 2 to the LLM, and "instead of the parser" should be dropped from the vocabulary in favour of "on top of the parser".

## 6. One-line consequences for the roadmap

- Build the resolver-backed store and the search tool first (CPU-only, no LLM inside — consistent with the stated aim).
- Expose the bidirectional loop (events 1–7) as the agent surface; the LLM supplies decisions, never edges.
- Spend model effort on L6 and the fallback regime, not on re-deriving L1–L4.
- Record this in the ledger (`1793881000-…`) as **H-32: resolver builds, LLM steers** — status `supported`; the §5 experiment is its test.
