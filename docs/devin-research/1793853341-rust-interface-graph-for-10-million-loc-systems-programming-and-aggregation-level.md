# Rust Interface Graph: Value at 10M LOC Systems Programming, and the Level of Aggregation

Companion to `*-rust-public-interface-dependency-graph-usefulness-form-and-open-decisions.md` in this folder. Answers two follow-up questions: how the graph helps people write 10M-LOC systems code, and whether the graph should hold public interfaces only or also internals (nested functions, private helpers). Rust only; one graph.

---

## 1. How this helps at 10 million lines of code

At roughly 10 tokens per line (rough estimate, not measured), 10M LOC is about 100M tokens. Even a graph at 5% of that is about 5M tokens, which does not fit any context window. The graph therefore cannot be pasted into a prompt. It must be a **map the agent navigates**: workspace, then crate, then module, then public contracts, then a function body only when needed. That turns it into a tool for six jobs:

1. **Design interfaces first, build in parallel.** At this scale the skeleton is the architecture spec, like header files in C or the kernel. Fix the contracts, then many agents or engineers implement bodies against them concurrently. The graph shows immediately when one change breaks a contract another module depends on.
2. **Know what a change breaks, in seconds.** Changing a trait signature or public type affects implementers and callers across hundreds of crates. At this size `cargo check` takes minutes; a graph query takes milliseconds. The compiler stays the final judge; the graph is the fast inner loop.
3. **Catch invented APIs.** In large codebases agents hallucinate functions and paths. Check every call and type the agent writes against the graph before compiling.
4. **Enforce architecture rules.** Drivers must not depend on filesystem internals; `no_std` crates must not reach `std`; every public `unsafe fn` and every `extern "C"` boundary is auditable; behaviour per build target (target x feature matrix) is visible.
5. **Plan refactors safely.** Splitting crates, moving items, deprecating APIs: re-export edges show which public paths downstream code relies on.
6. **Select context for agents.** Pick the relevant 20K tokens out of 100M instead of grepping and hoping.

### Why a small local model rather than compiler tools alone
In a 10M LOC system the code is broken most of the time someone is editing it. The graph must update on every edit, while the code does not compile, locally, cheaply, on every agent step. rust-analyzer cannot do that at this scale. If that property is not needed, compiler tools are sufficient and the model is not justified.

### Honest gap, and the biggest opportunity
A plain interface or call graph does not capture what actually breaks systems code: lock ordering, `Send`/`Sync`, interrupt-context safety, atomics and memory ordering, allocation in hot paths, panics, FFI. Edges for these systems-specific facts would distinguish this from every generic code-graph tool.

### Decisions this raises
1. **Graph first or code first?** A design spec that agents write against, or a map derived from existing code. Spec-first is more valuable and changes the product.
2. **Systems-specific edges in v1 or stay structural?** Candidates: unsafe boundaries, FFI, locks, `Send`/`Sync`, panics.
3. **Which ~6 queries do agents need?** E.g. what breaks if I change X; who implements trait T; public surface of crate C; path from A to B; unsafe surface. Design the schema around those.
4. **How deep is the summary hierarchy?** Levels (workspace, crate, module, item) and token budget per level.
5. **Benchmark.** A real large Rust codebase (Rust-for-Linux, Servo, rustc) and a task-level metric: did the agent's change compile and pass tests, with fewer tokens? Graph accuracy alone is not enough.

---

## 2. Is a Rust public-interface dependency graph really a good idea?

Yes, with two conditions, and no in one case.

- **Yes**, because Rust is the one language where "public interface" is precise. Modules, visibility, traits and crate boundaries make the public surface a real contract, not a naming convention. A graph of it is a true artefact, not a heuristic.
- **Yes**, because above roughly 1M LOC an agent cannot navigate with grep and file reads. Every serious coding-agent team builds some form of this (Sourcegraph SCIP, Aider repo map, Cursor indexing). The only question is whether yours is better for Rust.
- **Yes, only if consumed by agents, not humans.** Graph visualisers for humans have been built many times and are rarely used. Agents asking "who implements this trait" thousands of times a day are the real user.
- **No**, if the plan is to hand an agent a flat list of every public item. That is a worse table of contents and nobody reads it.

Confidence: high that a compiler-derived Rust interface graph with the right queries helps agents on large codebases. Medium-low that a small model can build it well enough to replace compiler tools. The model is the research bet; the graph is the safe part.

---

## 3. Level of aggregation: store everything, show the contract

The questions that matter all need internals. "What breaks if I change `Store::open`" is useless if callers reach it through three private helpers. "Is this safe to call from interrupt context" depends on what the private body does. So the **store** must hold:

- every item, public or private, including private functions, impl blocks and fields, as nodes
- `CALLS` and `SIGNATURE_USES` edges between all of them

But the **default view is public-to-public**. Private items are collapsed: if public `A` calls private `h1`, which calls public `B`, the view shows `A -> B` with `via: [h1]`. The agent expands a private item only on request. This gives a small map and correct answers at the same time.

### Functions inside functions, closures, nested items
Do not make them nodes. Fold them into the enclosing item; their calls become calls of the parent. A separate node per closure gives the agent nothing and triples node count. Exception: a nested `fn` or item referenced by name from outside its parent (rare); promote it when it happens.

### What the agent sees, layer by layer

| Layer | Content | Typical size |
|---|---|---|
| 1. Workspace | crates and crate dependencies | a few hundred tokens |
| 2. Crate | modules and public item counts | small |
| 3. Module | public contracts as a Rust skeleton | the main working level |
| 4. Item | full signature, docs, public-to-public edges, `via` lists | on demand |
| 5. Body | actual source | on demand |

Layer 3 is the "public interface dependency graph" from the original pitch. Layers 1 and 2 are required so it fits in context. Layers 4 and 5 are required so the answers are right.

### Remaining decision: edge-set order
Build `EXPOSES_AS`, `SIGNATURE_USES`, `IMPLEMENTS` first (exact, cheap to verify against rustdoc JSON), then `CALLS` with private collapsing (what makes the graph useful rather than merely accurate).
