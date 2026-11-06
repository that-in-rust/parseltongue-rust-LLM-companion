# Deep Insight 01 — collected, append-only

*This file holds every version of the Deep Insight 01 thesis and its follow-ups in one place, in the order they were written. Nothing is edited or removed when a later part supersedes an earlier one; later parts say what they revise. Rule for this file: append a new dated part, never rewrite an old one.*

```text
Part 1   2026-11-05   The key differentiation: a rust-analyzer-trained small model vs rust-analyzer
                      (cold start, RAM, "any file, no setup"; §8 Shape / Reach / Honesty positioning)
Part 2   2026-11-06   Parseltongue as an LLM-facing CODEMAP of current and future Rust systems
                      (rewrite committed as c52bb41bb; §6.2 qualifies Part 1's performance claims)
Part 3   2026-11-06   Four futures for a tiny-model Rust CODEMAP companion
                      (timeline-traverser exercise, committed as 894dbe0ab as Deep-Insight-03.md, kept there too)
Part 4   2026-11-05/06  Training-time shortcuts for the H-46 student: levers, scaling ladder, ELI5 ASCII
                      (verbatim copy of Deep-Insight-02.md, kept there too)
```

---

# Part 1 — 2026-11-05 — The key differentiation (original text, verbatim)

## Deep Insight 01 — The key differentiation: a rust-analyzer-trained small model wins on cold start, RAM and "any file, no setup"; rust-analyzer keeps the warm incremental regime

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Companion to ledger entry H-46 (`1793881000-hypotheses-ledger-…md`) and the ELI5 `1793941498-eli5-ascii-train-a-500mb-model-on-rust-analyzer-output-only-…md`. All rust-analyzer numbers are measured on this machine (`1793864191-…`, `1793873685-…`); all model numbers are estimates for a Mac mini M4 Pro GPU (~8 TFLOPS fp16, 273 GB/s; small-model decode is memory-bandwidth-bound, so decode speed ≈ bandwidth ÷ weight bytes). Verify with one real run before quoting.*

### 1. The question

If we post-train a ~500 MB model purely on rust-analyzer's views of the top-100 Rust repos (H-46), what can it honestly claim against rust-analyzer itself: faster? less RAM? It depends entirely on the regime.

### 2. CPU-only (the aim as originally stated): the model does not beat rust-analyzer on a loadable workspace

| | rust-analyzer (measured) | 500 MB student, CPU (est.) | 135M / 100 MB student, CPU (est.) |
|---|---|---|---|
| whole 42k-LOC workspace, cold | **29 s, 2.3 GB** (≈80 % of both in std + deps; cacheable) | 30 files × 15–25 s ≈ **10 min**, 0.6–1 GB | ≈ 2–3 min, ~0.2 GB |
| one file changed, index loaded | **milliseconds** | 15–25 s | 3–5 s |
| accuracy | ~100 % (0.02 % unknown types) | ≤ RA by construction; 78–95 % open | lower |
| needs | `cargo metadata`, dependency sources, build step for proc macros | the file | the file |

On CPU the only clean wins are "runs where RA cannot start" and "zero setup".

### 3. On the Mac mini GPU: cold start and RAM become real claims

| | rust-analyzer (measured) | 500 MB student on GPU (est.) | 135M / 100 MB student on GPU (est.) |
|---|---|---|---|
| one file: prefill 2k tokens + ~500 output tokens | ms (warm) / 29 s (cold load) | ~0.5 s prefill + ~2 s decode at 200–300 tok/s ≈ **2.5 s** | ≈ **0.7 s** |
| whole 42k-LOC workspace (30 files), cold, files batched | **29 s, 2.3 GB** | prefill compute-bound ≈ 15 s, batched decode nearly free → **~20–30 s, ~0.7 GB** | **~8 s, ~0.25 GB** |
| 10 M LOC, one file changed, nothing loaded | 10+ min load, then ms | **2.5 s, no load** | 0.7 s |
| accuracy | ~100 % | the open question (78–95 %) | lower |

```text
                 cold start          RAM              warm incremental      accuracy      setup
  rust-analyzer  29 s (42k LOC)      2.3 GB           ms        ◄ wins      ~100 % ◄ wins cargo + deps + build step
  500 MB on GPU  2.5 s per file ◄    ~0.7 GB ◄        2.5 s                 78–95 % ?     none ◄ wins
  135M on GPU    0.7 s per file ◄    ~0.25 GB ◄       0.7 s                 lower         none ◄ wins
```

### 4. What is defensible, and what is not

Defensible:
1. **Cold start / first answer** — seconds vs rust-analyzer's load time, and the gap grows with workspace size (10 M LOC: minutes vs 2.5 s).
2. **RAM** — 3–4× less for the 500 MB model, ~10× for the 135M model. Caveat: rust-analyzer with a pre-built std/deps index (H-33) would also shrink; compare against *that*, not against RA cold.
3. **Any file, no setup, no dependency download** — snippets, diffs, trees without `Cargo.toml`, Bazel/Buck, repos whose deps will not fetch; std and the popular crates live in the weights (44 % of references go there).
4. **Constant cost per file, independent of workspace size** — a startup property, not a steady-state one.
5. **Whole-workspace cold indexing** — a tie for 500M, a win for 135M, *but* comparing an approximate graph with an exact one.

Not defensible:
- "Faster than rust-analyzer" in the warm, incremental, editor regime: RA re-derives a changed body in milliseconds; the model needs seconds. This stays RA's.
- Any accuracy claim above RA's. The student is bounded by its teacher.

### 5. The headline sentence

> **"A 500 MB model gives you X % of rust-analyzer's graph for any Rust file in ~2.5 s on a Mac mini GPU, with no project setup, no dependency download, and a quarter of the memory."**

X is what the H-46 pilot measures. If X ≥ 95 this is a genuinely new tool (the per-file, no-setup rung of the ladder, usable in chat, in diffs, in CI on partial checkouts). If X ≈ 80 it is the fallback rung and the measured Cassini ceiling for Rust. The RAM and cold-start claims hold either way on GPU; the speed claim holds only for cold start.

### 6. Why this is the differentiation

Everything else in the stack (rust-analyzer-backed store, search tool, seven-event protocol) is exact engineering that others can also do. The thing nobody has is a toolchain-distilled model that **draws most of the exact graph for a file nobody can build**, in seconds, on a desktop GPU — with the exact resolver underneath for the cases where the project *does* load. "Resolver builds, student imitates anywhere, LLM steers."

### 7. Cheapest ways to turn estimates into measurements

- Tok/s and RSS of SmolLM2-135M and Qwen2.5-Coder-0.5B on the Mac mini under MLX, int8, 2k prefill + 500 decode (10 minutes).
- rust-analyzer cold vs with a pre-built std/deps cache on the same workspace (the fair RAM baseline).
- The H-46 pilot for X.

### 8. So what is the differentiation, really? (positioning, in the Shreyas Doshi register)

**The seductive-but-wrong one-liner:** *"Parseltongue gives you a dependency map when your LSP fails."*

It is seductive because it is true, measurable, and nobody else says it. It is wrong as the *headline* because it positions the product by its fallback. For the users who matter — agents inside 10 M-LOC systems code — the LSP usually does **not** fail. rust-analyzer loads. The agent's problem is not absence of a resolver; it is that the resolver hands it a keyhole (one cursor question at a time) onto 500 k items. Lead with the fallback and you have told the customer your product is for the day things are broken.

**The insight most people miss:** an LSP and a dependency map are not the same product at different quality levels. They are different *shapes* of the same facts.

```text
                       LSP (rust-analyzer)                     Parseltongue
   unit of question    a cursor position                       a concept, an item, a change
   unit of answer      one definition / one list of refs        a bounded neighbourhood, with a token budget
   who it is for       a human with an editor open              an agent with a context window
   what it cannot say  "blast radius", "path from test to       — these are its first-class queries
                        panic", "the 4 things closest to X"
   honesty             implicit (everything is exact)           explicit per edge: exact / model / heuristic
```

**The three layers of the differentiation, in order of weight:**

1. **Shape (the headline, 70 %).** The dependency map delivered as an agent-shaped, token-budgeted, queryable graph — neighbourhoods, blast radius, `via` chains, paths, clusters — over the *same exact facts* the compiler has. True on every day the LSP works, which is most days. This is grep-replacement and the seven-event protocol.
2. **Reach (the insurance, 20 %).** Where no resolver can start — a snippet in a chat, a diff, a tree without `Cargo.toml`, Bazel/Buck, a partial CI checkout, dependencies that will not fetch — the rust-analyzer-trained student draws ~X % of the same map anyway, in ~2.5 s on a desktop GPU, with no setup and a quarter of the RAM (§3–§5 above). The product never has to answer "cannot analyse".
3. **Honesty (the trust, 10 % of the pitch, 100 % of the adoption).** Every edge carries its provenance: resolver = exact, student/model = confidence < 1, ranking/clustering = heuristic. An LLM-built graph cannot offer this; an LSP does not need to. For an agent that will act on the map without a human in the loop, this is the property that decides whether it is used at all.

**Failure modes of getting the order wrong:**

- Lead with Reach → you are "an LSP for broken projects"; compared on accuracy against rust-analyzer and you lose by construction.
- Lead with Honesty → you are a research artefact; nobody buys provenance.
- Lead with Shape without Reach → the first time an agent pastes a snippet and gets "run cargo first", the story breaks.

**The positioning statement that holds all three:**

> *The dependency map of your code, in the pieces an agent can actually use — exact where the compiler can see, honest where it cannot, and still there when your LSP will not start.*

**The one-sentence test of whether we have built it:** an agent that has never seen the repository asks one seven-word question and, three tool calls later, is editing the right function with its blast radius in context — and can tell you which parts of that context were proven and which were guessed.

---

# Part 2 — 2026-11-06 — Parseltongue as an LLM-facing CODEMAP (rewrite, verbatim)

## Deep Insight 01: Parseltongue as an LLM-facing CODEMAP of current and future Rust systems

Written originally 2026-11-05; thesis revised 2026-11-06 on `research/tiny-rust-code-mapper`.

Status: product and representation proposal, not a claim that the complete system exists. This revision replaces the earlier cold-start-led positioning. Earlier performance estimates remain research hypotheses, summarized below; the original wording remains in Git history.

### 1. The thesis

**Parseltongue should describe the Rust codebase that exists, let an LLM propose the codebase it wants, and check the difference.**

Its core representation has two linked levels:

1. File dependencies: which source files depend on which other files, and why.
2. Public-interface dependencies: which boundary-visible contracts depend on which other contracts, and how those contracts map to files, declarations, implementations, and exported names.

The graph generator remains central. The graph becomes something an agent can navigate, discuss, propose changes to, and verify against code. A tiny local model and a CLI/MCP tool can be complementary parts of the product, not competing definitions of it.

The product promise is:

> Understand what connects today. Propose what should connect tomorrow. Verify what actually changed.

The differentiator is not automatically fastest search, replacing rust-analyzer, or drawing a more attractive diagram. The hypothesis is that a reliable, compact, bidirectional map improves an agent's ability to understand and change large Rust systems while making the change inspectable by a human.

### 2. What a CODEMAP is

**A CODEMAP is a versioned, queryable description of a codebase's files, boundary-visible interfaces, and typed dependencies, with mappings to source, supporting evidence, and explicit gaps.**

It is a data contract, not a particular database, visualization, parser, or model. A CLI result, an MCP response, a compact text neighborhood, and a visual diagram should be projections of the same underlying records.

| Component | Minimum meaning | Question answered |
| --- | --- | --- |
| File graph | Source files and classified dependencies between them | Which parts of the repository are connected? |
| Interface graph | Functions, types, traits, methods, relevant members, signatures, and typed relationships | Through which contracts are those parts connected? |
| Source mapping | Declaration sites, implementation sites, exported aliases, and evidence locations | Where should the agent inspect or change the code? |
| Evidence envelope | Source snapshot, target/features, tool versions, provenance, resolution status, and coverage limits | Under what conditions is this answer trustworthy? |

Two visible levels do not require shallow extraction. A public interface can call a private helper that calls another interface. The default view can summarize that route with a `via` path, but must label it as projected or transitive rather than invent a direct call.

The file graph cannot be calculated exclusively from public-interface edges. Private implementation code can introduce cross-file dependencies. Preserve its evidence even when the interface view omits those details. This is a presentation boundary, not a claim that private code is inactive.

#### 2.1 Minimum conceptual record set

```text
Snapshot: source revision and dirty-file digests, dependency inputs,
          target, feature profile, toolchain and extractor versions
File: identity, path, content revision, analysis status
Interface: identity, kind, qualified names, signature, visibility scope
Mapping: interface, declaration/implementation/exposure role, source location
Dependency: source, target, relation, evidence, resolution status, configuration
Coverage: scope, supported analyses, exclusions, failures, unresolved references
Proposal: base snapshot, graph patch, goal, constraints, assumptions, validation
```

These are proposed concepts, not an implemented schema. Storage and serialization can change without changing their meaning.

#### 2.2 Public means visible across a specified boundary

For large Rust systems, external crate APIs alone are often too narrow a view of internal architecture. Proposed default: include externally accessible APIs and internal cross-module contracts, labeled by their effective visibility scope.

Rust has unrestricted and restricted visibility, ancestor-module constraints, and re-exports. A `pub` token alone does not establish external reachability; a file is not necessarily a Rust module. Store declared visibility, effective accessibility where established, declaration path, and exposure path separately. See the [Rust visibility reference](https://doc.rust-lang.org/reference/visibility-and-privacy.html).

The exact inclusion policy for internal contracts remains a product decision to validate. Do not silently reinterpret crate-private items as externally public.

#### 2.3 Typed edges prevent misleading answers

Start with a compact, versioned vocabulary. Useful distinctions include `parameter_type`, `returns`, `field_type`, `associated_type`, `implements`, `reexports`, `calls`, and `may_call`.

An interface mentioning a type is not proof of a call to that type's methods. A trait implementation is not proof that a particular runtime invocation reaches it. A call graph is not a complete control-flow or data-flow graph.

Forward and backward traversal should specify relation filters. "Who depends on this error type?" and "Who calls this function?" are different queries, even though both traverse edges backward.

### 3. Current and proposed maps share vocabulary, not authority

| Dimension | Current CODEMAP | Proposed CODEMAP |
| --- | --- | --- |
| Meaning | What analysis establishes or explicitly hypothesizes about a source snapshot | What an agent or person wants to change |
| Authority | Source evidence and recorded analysis limitations | Design intent, goals, assumptions, and constraints |
| Changes | Re-extraction and reconciliation after source changes | Explicit additions, removals, moves, and signature/relationship edits |
| Validation | Freshness, evidence validity, and extraction correctness | Schema and structural checks, then implementation conformance |
| Unknowns | Missing, ambiguous, unsupported, or failed observations | Unresolved design choices and untested consequences |

**A future CODEMAP should be a proposed graph patch against a specific current snapshot.** An LLM should not replace the observed map with a persuasive diagram.

A proposal should identify retained interfaces, introduced interfaces, removed relationships, changed signatures, and required or forbidden dependencies. It should carry its goal and scope so that "better architecture" can be judged against a purpose.

There are three different states, not just true and false:

- An observed fact about today's code, with evidence.
- A hypothesis about today's code, awaiting confirmation.
- An intended relationship in tomorrow's design, awaiting implementation.

These must remain distinguishable even when all three originate through an LLM-facing tool.

#### 3.1 Example: decoupling checkout from storage

Suppose the current map records a concrete dependency from `Checkout` to `PostgresStore`. The design goal is to support a second persistence backend without duplicating checkout behavior.

| Record | Current | Proposed |
| --- | --- | --- |
| Checkout dependency | Concrete `PostgresStore` dependency | Dependency on an `OrderStore` trait contract |
| Storage contract | No separate trait | Introduce `OrderStore` with required operations |
| Implementation relationship | Concrete methods only | `PostgresStore` implements `OrderStore` |
| File ownership | Checkout and storage implementation sites | Existing sites plus a chosen contract declaration site |
| Validation obligation | Existing behavior | Compatibility, trait obligations, and preserved behavior |

The map can identify affected constructors, consumers, imports, signatures, and tests to inspect. It cannot conclude that a new trait is worthwhile simply because it changes graph shape. For a permanently single-backend system, the extra indirection might be unnecessary.

A type-checking contract sketch can detect some visibility, type, and trait problems before full implementation. It does not prove runtime behavior, performance, implementation call edges, or correctness of the design.

#### 3.2 The three-way comparison

After implementation, compare:

1. The before map: what was true when the proposal was made.
2. The intended map: what the approved proposal requested.
3. The after map: what the changed source actually establishes.

Report achieved changes, missing changes, unexpected changes, and unverifiable regions. Scope the comparison to the proposal: unrelated repository changes are not automatically architectural violations.

The underlying idea has precedent in [software reflexion models](https://www.cs.ubc.ca/~murphy/papers/rm/fse95.html), which compare high-level and extracted source models through mappings. Our product hypothesis is an agent-friendly, incrementally maintained Rust workflow around that comparison, not the invention of design conformance analysis.

### 4. How agents use it on large codebases

#### 4.1 Navigate both ways

An agent searches for a concept, anchors it to an interface or file, expands a bounded neighborhood, and opens the exact source evidence needed for the task. It can start from a file and discover contracts, or start from a contract and discover its files and consumers.

Backward traversal supports change assessment. Forward traversal supports dependency investigation. A path query can explain how two apparently unrelated areas are connected, subject to the available relation types and evidence.

Every result should disclose snapshot, filters, truncation, and analysis gaps. A small response is useful only if the agent knows what was left out. Start with simple depth and token bounds; introduce sophisticated ranking only when task evaluation justifies it.

#### 4.2 Design before editing

Given a goal such as "separate persistence from parsing," an LLM proposes alternative graph patches. Parseltongue compares the affected contracts, dependency directions, compatibility risks, required implementation work, and unresolved assumptions.

Files and interfaces can be grouped into named subsystems as a view. This need not add a third extraction level or an elaborate architectural ontology to the first version.

The LLM supplies candidate designs and rationale. The tool supplies grounded neighborhoods, structural checks, and later comparison with implementation. A tiny model may contribute extraction or ranking; it need not be a general-purpose architect.

#### 4.3 Review against intent

A maintainer should be able to ask: "Did this patch move us toward the design we approved?" The response should identify changed contracts and dependencies, not merely repeat the file diff.

This is a possible trust mechanism for agent-generated changes. It complements source review and tests; it cannot certify a PR as safe from graph agreement alone.

### 5. The failure and trust model

This section concerns misleading analysis and unsafe agent reliance, not a complete deployment-security review.

| Threat | Failure | Required behavior |
| --- | --- | --- |
| Invented certainty | A guessed relationship appears established | Keep provenance and resolution state on each claim; preserve conflicting evidence |
| False completeness | No result is interpreted as no dependency | Return supported scope, exclusions, failures, unresolved references, and truncation |
| Stale or wrong configuration | An accurate graph describes another revision or feature profile | Bind results to explicit source and build-context snapshots |
| Identity churn | A comment or moved span makes every interface look new | Separate entity identity, revision hashes, and source locations |
| Incorrect equivalence | Matching names or content hashes merge different entities | Use scope-aware identity and explicit reconciliation; retain ambiguous move matches |
| Hidden implementation dependence | Public-only extraction drops meaningful file relationships | Preserve private-route evidence beneath projected views |
| Proposal contamination | A desired edge becomes a fact before code exists | Isolate proposals from observations and require re-extraction after implementation |
| Overclaiming design quality | Fewer edges or successful compilation means "better" | Evaluate stated design goals, behavioral tests, and practical tradeoffs separately |

Identity deserves special care: a source-span hash identifies a content revision, not necessarily a logical interface. Two different interfaces can have identical text; one interface can change its signature. Renames and moves need explicit matching or uncertainty, not a universal hash trick.

Adding an ordinary comment may shift locations without changing interface dependencies. Re-extraction work and semantic graph changes are different things. Documentation comments, attributes, macro input, and source-sensitive tooling can require separate treatment; do not claim all comment-like edits are universally irrelevant.

The [facts, vocabularies, and annotations research note](1793951386-ideas-from-sikos-cybersecurity-knowledge-graphs-full-text-and-misp-threat-sharing-platform-for-the-rust-graph.md) provides a useful storage principle: annotations and model proposals should not overwrite source-backed observations. An agent following an edge once is additional evidence, not universal proof.

MCP reads should not implicitly authorize code edits, builds, network downloads, or proposal publication. Reading untrusted Rust and executing its build scripts or procedural macros are different trust boundaries. Any future validation runner needs explicit execution policy and isolation.

### 6. Where the tiny model belongs

The product can be a local CLI/MCP tool plus a task-specific tiny model. The model contributes to the CODEMAP contract; the host system owns scanning, snapshotting, bounded retrieval, validation, reconciliation, and queries.

Possible learned tasks include selecting among candidate bindings, proposing unresolved dependency records, identifying useful next context, or ranking explanations of missing analysis. These are alternatives to evaluate, not capabilities already established.

Direct source-to-graph generation remains a legitimate research path. Its output needs schema validation, provenance, abstention, and evaluation on unseen repositories. A fallback proposal is not confirmed merely because the primary resolver could not resolve it.

The consuming coding LLM can use the resulting map to propose designs. Training a small model to reproduce observed graphs does not, by itself, train architectural judgment or validate future-map quality.

#### 6.1 What Deep Insight 02 contributes

[Deep-Insight-02](Deep-Insight-02.md) proposes a pretrained base, item-level examples, separate skeleton and resolution tasks, a training-data scaling ladder, held-out repositories, and evaluation of the quantized artifact.

The transferable lesson is to define the record-emission task and its evaluation before committing to a large training run. Its training times, memory projections, accuracy expectations, and claimed savings are estimates, not measured product guarantees.

Item windows must contain context available at inference time. Giving the model "the signatures this function references" can leak the answer if identifying those references is the task. Context retrieval failures belong in evaluation, not outside the benchmark.

#### 6.2 What the earlier performance thesis does and does not establish

Retain cold-start latency, low memory, offline operation, and usefulness on partial projects as candidate advantages. Do not present them as proven wins or universal disadvantages of rust-analyzer.

- Compare equivalent workloads: time to a useful answer, cold and warm runs, graph scope, and correctness. One file is not an entire workspace.
- No `Cargo.toml` does not automatically mean rust-analyzer cannot support the project; alternative project configuration may be available.
- A model trained on resolver outputs is not guaranteed to match the resolver, nor mathematically forbidden from outperforming a particular resolver run. Independent evidence is needed for either claim.
- A reported accuracy from another model/task is not a ceiling for this task. No fixed 78% or 95% outcome has been established here.
- Artifact size is not runtime RAM. Quantized weights, tokenizer, context/cache, temporary buffers, and host allocations all count toward the measured envelope.
- A sub-100 MB learned helper does not imply a sub-100 MB whole-repository analyzer. State separately whether a budget covers the model process, the CLI plus model, or all resolver subprocesses and caches.

The [semver experiment](../experiments/public-interface-semver/README.md) gives a concrete warning: source-only agents identified all seven exported nominal types and thirteen public inherent methods in that assignment, but found only ten of nineteen scored local field/signature dependencies. That is evidence of omissions in a narrow experiment, not a general model accuracy estimate.

### 7. Product options and recommendation

| Option | Value | Limitation |
| --- | --- | --- |
| Read-only navigation map | Faster grounded orientation and dependency lookup | Does not capture architectural intent |
| Navigation plus design workspace | Propose contract changes and verify implementation against intent | Requires stable mapping, proposal semantics, and useful comparison |
| Autonomous architecture generator | Attempts end-to-end design selection and implementation | Adds an unvalidated architecture-quality problem and much greater trust burden |

Recommend the second as the product direction, with navigation as its first usable increment. This keeps both current and future maps in view without making autonomous design quality a prerequisite for usefulness.

### 8. First experiment and success criteria

Choose one real Rust refactoring with an explicit architectural goal. Capture a baseline map, ask an agent to propose a bounded graph patch, have a maintainer review the proposal, implement the change, and compare the resulting map with the intended one.

Compare graph-assisted work with the same coding agent using its normal file/search tools and access to Rust tooling. Keep source snapshots, goals, budgets, and validation tasks comparable. Expand to multiple unseen crates and tasks before generalizing.

Measure separately:

1. Extraction: edge precision and recall by relation, coverage, abstentions, stale records, and configuration handling.
2. Navigation: correct localization, downstream-task correctness, tool calls, context tokens, and time to useful evidence.
3. Design: maintainer assessment against the stated goal, compatibility, implementation effort, and unnecessary complexity.
4. Conformance: detection of intended, missing, unexpected, and unverifiable changes, including intentionally seeded mismatches.
5. Operations: CPU latency, peak resident memory, graph storage, incremental update cost, and installation friction.

No weighted aggregate should conceal a loss of correctness behind token savings. No memory claim should exclude a required process without naming that boundary.

The strongest early signal is not praise for a diagram. It is repeated use on real changes because the map makes the correct next action easier and the resulting change easier to review.

### 9. Sources, decisions, and open questions

Repository evidence:

- [Training strategy and scaling ladder](Deep-Insight-02.md).
- [Spec-first evidence-bearing graph thesis](1793858459-thesis-spec-first-evidence-bearing-rust-interface-graphs-for-coding-agents-strata-clarity-literature.md), especially sections 6, 8, and 9. This revision qualifies its claims about identity and what compiling skeletons establish.
- [Facts and annotation layers](1793951386-ideas-from-sikos-cybersecurity-knowledge-graphs-full-text-and-misp-threat-sharing-platform-for-the-rust-graph.md).
- [Pinned semver graph experiment and limitations](../experiments/public-interface-semver/README.md).
- [Pretrained binding helper experiment](../experiments/tiny-reexport/learned-binding/pretrained/README.md), a narrower learned task than general graph generation.

External sources:

- [Rust Reference: visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
- [Murphy, Notkin, and Sullivan: Software Reflexion Models](https://www.cs.ubc.ca/~murphy/papers/rm/fse95.html).

Open decisions: exact internal-interface inclusion policy; first relation types; smallest useful proposal language; identity reconciliation across moves; which tiny-model task adds measurable value; whether the sub-100 MB target applies to inference or the whole application.

For four alternative product paths, adoption scenarios, and decision gates, see [Deep Insight 03: four product timelines](Deep-Insight-03.md).

The thesis does not depend on resolving every one of these in advance. It does depend on keeping observations, hypotheses, and intended designs separate from the first version.

---

# Part 3 — 2026-11-06 — Four futures for a tiny-model Rust CODEMAP companion (verbatim copy of Deep-Insight-03.md)

## Deep Insight 03: Four futures for a tiny-model Rust CODEMAP companion

Date: 2026-11-06. Based on [Deep Insight 01](Deep-Insight-01.md), committed as `c52bb41bb` before this scenario exercise.

Method: `timeline-traverser`. These are conditional product scenarios, not forecasts, implementation commitments, or measured market demand. The four paths are alternative starting points; they are not four simultaneous products to build.

### Decision Frame

#### The actual fork

The question is not simply "model or CLI?" Parseltongue can combine both. The decision is **which repeated user job should make the combined product indispensable first**:

1. Navigating unfamiliar Rust code every day.
2. Designing a cross-cutting change before implementation.
3. Reviewing whether a change matches its stated intent.
4. Providing a reusable graph-generation component to other agent builders.

All four use the same two visible levels: file dependencies and boundary-visible interface dependencies, with mappings between them. Current observations, uncertain hypotheses, and proposed future designs remain distinct. None promises complete runtime control flow from interface signatures alone.

#### Goals, constraints, and success

- Primary technical audience: people and agents working on substantial Rust systems, such as storage engines, networking libraries, runtimes, compiler infrastructure, and multi-crate services. These are candidate segments, not validated customers.
- Product goal: make it easier to understand connections, propose changes to those connections, and verify resulting code against the proposal.
- Model goal: a specialized local model that contributes to graph construction or graph-based work. A general coding chatbot is outside scope.
- First memory gate: measure a declared CPU inference configuration against 100,000,000 bytes peak resident memory. Report whole-application and resolver memory separately. The existing experiments do not establish a whole-large-repository analyzer under that limit.
- If 100 MB becomes the budget for the entire application, use bounded source batches and disk-backed graph access and test that claim directly. Do not silently exempt the host process, page cache, or concurrent workers.
- Correctness: expose evidence, unresolved cases, feature configuration, stale snapshots, and truncated results. An empty neighborhood is not proof of independence.
- Human authority: proposed architecture does not become observed reality or authorize edits merely because an LLM emitted it.
- Commercial success: repeated voluntary use on real work, followed by willingness to support or pay for a specific workflow. Stars, downloads, benchmark scores, and enthusiasm are intermediate signals, not substitutes.

#### Horizon and assumptions

Use week 1, month 1, quarter 1, and year 1. These are learning checkpoints, not delivery estimates. Assume one main builder, reuse of existing experiments, and access to a small number of Rust maintainers willing to provide tasks and feedback. Recruiting those maintainers is itself an uncertain dependency.

Week 1 should collapse a key uncertainty, not deliver a general graph engine. Month 1 should produce a narrow usable loop. Quarter 1 should test repeat usage. Year 1 describes what could compound if earlier gates hold.

#### Evidence we actually have

| Evidence | What it supports | What it does not establish |
| --- | --- | --- |
| [Pinned semver graph experiment](../experiments/public-interface-semver/README.md) | Typed public-interface records, re-export mapping, configuration-specific graphs, and queryable output are concrete | General Rust coverage, a runtime call graph, large-workspace performance, or PMF |
| Same semver experiment: agents found 10 of 19 scored field/signature dependencies | Plausible inventories can omit important edges | A general accuracy score for all LLMs or all Rust crates |
| [Pretrained binding helper](../experiments/tiny-reexport/learned-binding/pretrained/README.md) | A 17.45 MB encoder with trained heads ran a narrow CPU inference task below 100 MB in the reported runs | A generative LLM, whole-source graph generation, or whole-system memory compliance |
| [Fresh-crate weight-edit comparison](../experiments/tiny-reexport/learned-binding/pretrained/weight-edit/README.md) | The unchanged and edited models each recovered seven confirmed reference sites after rules abstained on the fresh set | Seven independent reasoning mechanisms; the weight edit added no accepted decisions |
| [Deep Insight 02](Deep-Insight-02.md) | A proposed training ladder and narrower supervised tasks | Verified training times, a guaranteed accuracy ceiling, or a proven sub-100 MB graph model |

The model evidence is mixed: no accepted bindings on the original five releases, some confirmed residual bindings on the fresh set. Keep both observations. The reference sites and feature configurations matter more than a favorable headline.

External grounding: [rust-analyzer supports non-Cargo project descriptions](https://rust-analyzer.github.io/book/non_cargo_based_projects.html), so absent Cargo metadata is not by itself a defensible product boundary. [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) already scans public-API compatibility, so a review product needs value beyond listing signature breakages. [Software reflexion models](https://www.cs.ubc.ca/~murphy/papers/rm/fse95.html) provide precedent for comparing intended and extracted structures. None of these sources validates demand for Parseltongue's proposed combination.

### Timeline A

#### The Local Navigator: tiny model + CLI + MCP

**Product promise:** "Give your coding agent a compact, inspectable map of an unfamiliar Rust system."

**Initial user:** a contributor or engineer entering a repository they did not write. The coding agent consumes the map; the engineer benefits from fewer wrong turns and easier inspection of the agent's context.

**Opening move:** ship one local package that scans a bounded repository scope, builds the two linked views, serves forward/backward queries, and includes a small learned component. The model's initial task is deliberately narrow: propose missing bindings from retrieved candidates or emit bounded interface records. Both approaches must be compared with the same retrieval and deterministic baseline.

| Horizon | Action and causal chain | Conditional result |
| --- | --- | --- |
| Week 1 | Pick concrete navigation questions from unfamiliar crates. Run tool-only, model-assisted, and normal agent file/search workflows on matched tasks. | We learn whether the extra graph records change a useful next action, rather than merely improve an isolated score. |
| Month 1 | Package source anchoring, forward/backward neighborhoods, explicit unknowns, and model provenance. Integrate with one agent harness. | If installation is easy and answers lead to useful source, engineers start using it while exploring; otherwise the product remains a demonstration. |
| Quarter 1 | Reconcile edits incrementally and add bounded on-disk graph queries. Evaluate fresh crates, feature profiles, and larger workspaces. | Repeat users trust the same tool across sessions; failures reveal whether model gaps, configuration, or query presentation are the main cost. |
| Year 1 | Successful task traces and corrected graph examples improve the narrow model and retrieval policy, subject to permission and data licensing. | The product could become a routinely installed Rust companion. If large agent tools absorb equivalent navigation, it needs durable quality or operational advantages to retain users. |

**Two-way daily loop:** file or query to interface neighborhood to source; changed source back to refreshed graph and affected consumers. The model proposes structure; the host checks schemas and available evidence; the consuming LLM requests more context when the map is incomplete.

**Division of work:** the host manages source snapshots, candidate retrieval, cache invalidation, graph storage, and budgets. The small model processes bounded inputs. Graph search stays ordinary code. A separate semantic resolver can enrich or verify records where available, but its cost is not hidden in the model budget.

**Future-map progression:** after navigation is useful, permit a small proposal such as moving a contract or replacing a concrete dependency. Do not require an elaborate design editor before the first successful navigation task.

**Likely reactions, as hypotheses:** contributors may appreciate not having to learn the repository layout first. Experienced maintainers may dismiss a map that merely repeats known imports. Agent builders may welcome structured context but resist extra latency or unreliable tool selection.

**Lived experience:** low ceremony for users if one command produces useful evidence. For the builder, the burden is relentless edge cases, reproducibility, installation support, and keeping graphs fresh. It is less dramatic than a model demo, but daily usefulness is testable.

**Inflection point:** the model adds verified useful records or better task outcomes at an acceptable wrong-edge rate. If it does not, keep it as an explicitly experimental component and narrow the learned task; do not claim success from bundling weights.

**Failure branch:** token savings come with worse answers, or users must inspect every edge manually. They revert to their existing agent and file tools. A fast approximate graph is not a success if verification overhead exceeds the saved work.

**Success signal:** unaided return usage on new tasks, preserved or improved task correctness, and lower time to relevant evidence. A commercial extension such as team-managed caches or support is a hypothesis to test, not assumed revenue.

### Timeline B

#### The Design Workspace: current map + proposed map + implementation check

**Product promise:** "Agree on the interface-level change before asking an agent to write the implementation."

**Initial user:** a maintainer or staff engineer coordinating a refactoring across modules or contributors. The buyer, if one emerges, cares about preventing expensive rework rather than maximizing the number of indexed symbols.

**Opening move:** use the existing graph work to prepare one real design session. A coding LLM reads the current map and proposes a graph patch tied to an explicit goal. The maintainer edits or approves that patch before implementation. A CLI is sufficient initially; a visual view should inspect the same data, not become a separate architecture document.

| Horizon | Action and causal chain | Conditional result |
| --- | --- | --- |
| Week 1 | Choose a real change with a disputed boundary, such as introducing a storage trait. Compare two proposals using affected contracts and source evidence. | We discover whether the map clarifies an actual decision or merely adds a formalization chore. |
| Month 1 | Record proposal identity, base snapshot, scoped constraints, and the before/intended/after comparison for one completed change. | A maintainer can see whether the implementation achieved the agreed structure, without inferring intent from a large diff. |
| Quarter 1 | Add explicit rename/move mapping, proposal revisions, stale-base checks, and collaboration on a small set of repeated refactorings. | Teams may reuse approved contracts to coordinate contributors, but only if the proposal stays cheaper than the rework it avoids. |
| Year 1 | Approved decisions and their implementation outcomes form a navigable design history. | The product could become part of architectural change planning. If changes are too infrequent or highly informal, it remains a specialist workflow rather than a daily companion. |

**Two-way daily loop:** architectural goal to candidate map patch to source obligations; implementation back to actual map delta and proposed-design adjustments. A rejected implementation may reveal a flaw in the design, not just a disobedient agent.

**Division of work:** the tiny model helps construct/update the current map or rank narrowly scoped graph edits. The user's main coding LLM proposes alternatives and explains tradeoffs. Deterministic checks enforce graph schema, scope, identity, and configured constraints. Compilers and tests validate their own supported properties, not "architectural goodness."

**Creative advantage:** the graph patch is an executable part of the design discussion. The team can ask "which contracts must change for this option?" before modifying implementation bodies. Files provide actionable work locations; interfaces provide the design boundary.

**Likely reactions, as hypotheses:** senior engineers may value explicit contracts and dislike tools that prescribe fashionable abstractions. Contributors may appreciate agreed interfaces but resist stale plans that block practical fixes. An approval process must allow the design to evolve with evidence.

**Lived experience:** initial attention cost is higher than Timeline A. The potential relief is less late-stage disagreement and fewer large rewrites. The builder faces identity reconciliation, proposal merge conflicts, and ambiguous mappings rather than only extraction accuracy.

**Inflection point:** maintainers voluntarily create a second proposal before implementation. A diagram they admire once is not enough. If they only use the output after code is written, the stronger product may be Timeline C.

**Failure branch:** proposals become another stale document. Agents optimize for satisfying graph constraints while missing behavior, error handling, performance, or readability. Keep the scope explicit and continue source review and tests.

**Success signal:** fewer avoidable design revisions after implementation, accepted proposal-to-code traceability, and repeated use on cross-cutting changes. Pricing around team collaboration is a candidate business model, not yet evidence of willingness to pay.

### Timeline C

#### The Intent-Aware Reviewer: graph deltas + CLI/CI + a small learned assistant

**Product promise:** "Show whether this Rust change moved the code toward its stated design, and where the evidence is incomplete."

**Initial user:** an OSS maintainer or engineering team reviewing agent-generated changes. Start with advisory reports; do not make model conclusions automatic merge gates.

**Opening move:** take a historical PR with a known architectural change. Build base and head maps and compare them with a scoped statement of intent. If the PR lacks an approved intended map, say so: an inferred summary of intent is not an author-approved contract.

| Horizon | Action and causal chain | Conditional result |
| --- | --- | --- |
| Week 1 | Replay several PRs and seed explicit architectural mismatches in separate test cases. Ask reviewers to locate changed contracts with and without the report. | We learn whether the graph reveals something actionable beyond existing tests, API checks, and file diffs. |
| Month 1 | Publish a compact advisory artifact containing changed interfaces, typed dependency deltas, affected files, and evidence links. | Reviewers can inspect the highest-value changes without trusting a free-form LLM verdict. |
| Quarter 1 | Add opt-in proposal comparison and selected deterministic dependency rules. Keep model-derived claims outside required pass/fail checks. | Teams retain the tool only if useful findings outweigh noise and CI/maintenance cost. |
| Year 1 | Repeated corrections and approved exceptions improve report relevance and project-specific policy. | The product could become a trusted review companion. If false alarms persist, teams disable it regardless of model quality on extraction benchmarks. |

**Two-way daily loop:** proposed intent to implementation delta; review finding back to source or a revised design decision. Start from a changed file to see contracts, or from a changed contract to find consumers elsewhere in the repository.

**Division of work:** the host compares snapshots and separates code movement from semantic changes. The tiny model can recover bounded missing records or rank evidence-backed findings. The main coding LLM can explain the report. No model explanation is allowed to erase analysis gaps or claim behavioral safety.

**Position against existing tooling:** [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) already addresses public-API compatibility. The hypothesis here is value from internal boundary changes, dependency-direction constraints, evidence-linked explanations, and conformance to an approved proposal. Integrate or compare with established checks instead of claiming they do not exist.

**Likely reactions, as hypotheses:** maintainers may welcome a concise change witness but strongly resist another noisy bot. Contributors may feel unfairly judged if heuristic warnings resemble mandatory failures. Reports must distinguish fact, policy, suggestion, and unknown.

**Lived experience:** the successful version reduces reviewer attention cost. The unsuccessful version creates notification fatigue and arguments with a bot. Operationally, configuration matrices and untrusted PR code make validation execution and cache isolation important from the beginning.

**Inflection point:** reviewers keep opening the report and act on findings that were not already obvious. If all value comes from existing compatibility tools, the new product has not yet earned its place.

**Failure branch:** a missing edge is presented as proof that a change is isolated; a reviewer gains false confidence. This is worse than an unhelpful report. Coverage must be visible, and safety-critical gates must rely on scoped, validated checks.

**Success signal:** useful findings per reviewer minute, retained installation, lower investigation time, and no deterioration in decision quality. Paid private-repository workflows or policy support are possible, but require actual demand validation.

### Timeline D

#### The Graph Component: tiny source-to-record model + runtime + SDK

**Product promise:** "Embed a Rust-specific CODEMAP generator inside your own coding agent or developer tool."

**Initial user:** an agent-harness maintainer or developer-tool builder who wants a graph-building capability without creating a Rust-specific training and evaluation pipeline. This is an infrastructure product, not primarily a human-facing application.

**Opening move:** publish a small model and host runtime that jointly emit a deliberately limited, versioned graph schema. The CLI becomes the reference client and test harness. Unlike A, model capability and integrator reuse are the first adoption tests, not direct end-user navigation.

| Horizon | Action and causal chain | Conditional result |
| --- | --- | --- |
| Week 1 | Freeze a narrow record contract, candidate-retrieval policy, benchmark splits, abstention format, and CPU measurement boundary. Test existing checkpoints before training. | We find out whether model limitations are capacity, missing context, or unsuitable supervision, instead of assuming more tokens will fix them. |
| Month 1 | Train or adapt one bounded task and compare against rule/parser baselines on fresh crates. Package model, provenance, schema checks, and reproducible evaluation. | If it adds useful records within memory/latency limits, an integrator has a concrete capability to trial; otherwise it remains research. |
| Quarter 1 | Work with two independent integrators on their actual tasks. Fix portability, schema evolution, cancellation, artifact licensing, and context requirements. | Adoption tests whether the component is reusable beyond our own CLI and curated examples. |
| Year 1 | A permitted corpus of difficult cases and regression tests compounds model and runtime quality; conformance fixtures reduce integration cost. | The component could become reusable infrastructure. If deterministic context retrieval supplies nearly all gains, the model may not be the valuable standalone unit. |

**Two-way daily loop:** source/context to graph records; consumer failures and verified corrections back to evaluation and, with permission, training. Integrators consume the same records for navigation and future-map proposals, but those proposal workflows are not assumed to exist in the tiny model itself.

**Division of work:** the model predicts bounded records or bindings. Ordinary host code supplies candidate context, validates record shape, preserves source evidence, links batches, and stores results. This is not a promise to compile rust-analyzer into weights or to fit an entire repository into a single prompt.

**Training discipline:** repository-disjoint supervised splits, provenance and license tracking, feature-aware labels, explicit outside-candidate answers, and a genuinely untouched final evaluation set. Use a development set for the training ladder; repeatedly choosing checkpoints against the final test set would turn it into a validation set. Upstream pretraining contamination remains a separate uncertainty.

**Memory discipline:** measure the shipped quantized artifact, runtime, tokenizer, buffers, and bounded host work on CPU. A tiny encoder with trained heads is not automatically an LLM; name the actual model architecture accurately. A decoder-based graph generator under 100 MB remains to be demonstrated.

**Likely reactions, as hypotheses:** tool builders may value an open artifact but demand stable schemas, permissive-enough licensing, and reproducible gains. Researchers may engage with benchmarks without becoming product users. Downloads do not distinguish these audiences.

**Lived experience:** fewer interface-design tasks, more dataset work, negative results, runtime debugging, and integration support. The emotional risk is months of model optimization without knowing whether anyone needs the output. An early external integrator is therefore more valuable than a larger internal benchmark alone.

**Inflection point:** an independent tool uses the component on its own workload and retains it because removal makes outcomes worse. If integration only happens after extensive custom work per customer, the general SDK assumption may be wrong.

**Failure branch:** quality does not generalize, capacity runs out under the RAM limit, or a simple deterministic method matches the model. Release useful datasets and test fixtures, but do not confuse a worthwhile research artifact with a successful graph-generation product.

**Success signal:** retained external integrations, reproducible incremental graph value, and manageable compatibility costs. Commercial support or certified artifacts could support the work; neither is guaranteed by open-weight popularity.

### Cross-Timeline Analysis

#### Different jobs, one underlying representation

| Path | Primary repeated job | Upside | Main downside | Reversibility | Regret risk | Who must cooperate |
| --- | --- | --- | --- | --- | --- | --- |
| A: Local Navigator | Find the right contracts and source for a task | Frequent feedback; closest to tiny model + CLI companion | Navigation may be useful but insufficiently differentiated | High: graph/runtime carries into B, C, or D | Optimizing token savings without preserving correctness | Contributors and agent-harness users |
| B: Design Workspace | Agree on a structural change before implementation | Closest expression of current plus future CODEMAP | Extra planning ceremony and hard identity/mapping problems | Medium-high if proposals stay portable | Building a design environment nobody wants to maintain | Maintainers with real cross-cutting changes |
| C: Intent-Aware Reviewer | Understand and validate architectural deltas | Concrete review moment and an inspectable artifact | Noise, CI friction, and dangerous false reassurance | High for graph/diff core; integration work less reusable | Becoming another ignored review bot | Reviewers, contributors, and CI owners |
| D: Graph Component | Add Rust graph capability to another product | Reuse across tools; model-centered research and distribution | Highest dependence on generalizable learned capability and integrations | Medium: data and runtime remain useful, custom training may not | Spending heavily on model quality before proving demand | Independent tool builders and permitted data sources |

These are qualitative judgments, not probability estimates. A and C can yield user-workflow evidence earlier; B needs a real design cycle; D needs technical generalization and an integrator. That makes D the highest-variance starting point under current evidence, not necessarily the least valuable eventual outcome.

#### The important combinations

| Combination | Why it could work | Why not start with everything |
| --- | --- | --- |
| A + B | Navigate the current map, propose a bounded change, then navigate the intended one | Proposal authoring must not delay useful everyday reads |
| A + C | Same local graph powers development context and a review artifact | CI configuration, permissions, and reviewer UX add a separate operational burden |
| B + C | Approved intent becomes the basis for implementation conformance | Requires teams to actually record intent before implementation |
| A + D | CLI demonstrates the embedded model/runtime; SDK lets others reuse it | External API commitments can freeze a schema before real usage teaches us enough |

The most robust starting combination is **A with one small B/C loop**: local navigation plus one proposal and one before/intended/after report. It tests the full thesis without building a design IDE, a hosted review service, and a training platform at once.

#### What would be a durable advantage?

Potential advantages are hypotheses: a difficult-case dataset with defensible provenance; reliable Rust-specific identity and configuration handling; compact graph queries that improve real tasks; and accumulated project intent that users find valuable to preserve. Merely using a small model, offering MCP, or storing a graph is not itself evidence of defensibility.

The smallest model is not automatically the best product. A slightly larger model that changes correct outcomes could be more useful, but it would violate a strict 100 MB requirement. Do not quietly relax that constraint to rescue a weak result; either narrow the task or explicitly reconsider the product envelope.

#### Shared failure branches

- A model learns plausible public names but misses dependencies: inventory accuracy looks strong while navigation remains unsafe.
- A wrong feature profile makes both the baseline and model evaluation misleading.
- An agent follows a projected relationship as though it were a direct runtime call.
- A comment or rename creates noisy graph churn because identity and content revision were conflated.
- A proposed edge is rendered indistinguishably from an observed one.
- A graph passes structural checks while the implementation changes behavior incorrectly.
- Privacy or licensing prevents using retained project traces as training examples.
- The system demonstrates lower inference RSS by moving required work into an unreported subprocess.

These risks are shared infrastructure work, not a reason to add unrelated graph algorithms to the first product.

### Decision Filter

#### Recommended direction

**Describe Parseltongue as a Rust CODEMAP companion: a local CLI/MCP tool plus a specialized small model that helps construct and maintain the map consumed by larger coding agents.** Start with A, retain B as the product direction, and use a small C-style report to make the result inspectable. Keep D available as a packaging decision after the model/runtime demonstrates reusable value.

This recommendation does not demote the graph generator to a generic chatbot accessory. The generator is the common foundation. It also does not assume that a tiny model can independently design a large Rust system.

#### The next uncertainty-collapsing experiment

Run a bounded pilot on three Rust repositories not used for our supervised fitting: one library with re-exports, one multi-crate system, and one feature-heavy or macro-heavy project. This is a proposed test matrix, not a statistical sample sufficient to claim broad generalization.

For each repository, use one real navigation task and one small architectural change. Keep approved intent separate from model-inferred intent. Use the same main coding agent and comparable budgets in these arms:

| Arm | Setup | What it isolates |
| --- | --- | --- |
| Baseline | Agent with normal file/search tools and available Rust tooling | Existing workflow performance |
| Tool | Same agent plus CODEMAP built without learned additions | Value of the representation and query shape |
| Tool + model | Same tool, candidate retrieval, and validation with learned additions enabled | Incremental contribution of the tiny model |

Where repeated tasks would leak solutions to the same human, use matched tasks or counterbalanced assignments. Record code revision, build profile, tool/model versions, CPU, wall time, peak RSS by process boundary, task outcome, graph omissions, and reviewer decisions. A blinded reviewer should assess correctness and design fit where feasible.

Suggested pilot gates, explicitly chosen for learning rather than claimed industry standards:

1. At least two independent maintainers return for another real task without the builder operating the tool for them. This is an early retention signal, not PMF proof.
2. The map-assisted workflow does not lose task correctness in the pilot and produces useful source localization or conformance evidence beyond the baseline. Small samples cannot establish statistical non-inferiority.
3. Learned additions provide independently confirmed value beyond the same retrieval/rules baseline on more than one crate or failure family. Report wrong, unverified, and abstained records alongside successes; set any deployment error budget before final evaluation.
4. Every intentionally seeded proposal mismatch in the supported pilot relation set is either detected or explicitly marked outside coverage. Unsupported scope must not turn into a misleading pass.
5. The declared CPU inference envelope meets the 100 MB gate on the target setup. Report the full application's memory even when it is higher; failure of the whole-application target cannot be hidden by passing the model-only target.

#### How to choose after the pilot

- **Strongest if things go normally:** A provides frequent use and shared infrastructure; extend into B when users request pre-edit planning.
- **Safest if learned generalization disappoints:** preserve the CODEMAP tool and labeled evaluation corpus, narrow the model's task, and report the missed research goal honestly. Do not pretend the model-enabled thesis was validated by a tool-only win.
- **Choose C sooner** if maintainers consistently value the change report but do not want to author proposals. In that case, distinguish descriptive review from approved-intent conformance.
- **Choose B sooner** if maintainers repeatedly use the map to resolve design disagreements before writing code.
- **Choose D sooner** only when independent builders request and retain the graph component and model contribution survives ablation.
- **Stop expanding scope** if users do not return, the map reduces answer quality, or model corrections consume more effort than they save. Investigate the failed assumption rather than adding features.

**Final thesis:** the opportunity is not a tiny model that knows everything about Rust. It is a small, inspectable local system that gives larger agents a reliable way to understand existing connections, propose different ones, and check what changed.

---

# Part 4 — 2026-11-05/06 — Training-time shortcuts for the H-46 student (verbatim copy of Deep-Insight-02.md)

## Deep Insight 02 — Shortcutting the training time: the dataset lever beats the model lever, and a scaling ladder beats both

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Follows Deep-Insight-01 and ledger entry H-46 (post-train a ~500 MB model on rust-analyzer output only). All numbers are 6·N·D estimates for a Mac mini M4 Pro (~2.5 TFLOPS effective in training) and one H100 (~400 TFLOPS effective); verify with a 10-minute MLX run before planning around them.*

### 1. Question

H-46 as written costs ~1 B tokens/epoch for a 0.5 B model = 3×10¹⁸ FLOPs ≈ 2 weeks per epoch on the Mac mini. Can we shortcut this by playing with model size or dataset size?

Yes. The dataset lever is much larger than the model lever, and the biggest shortcut is a procedure (a scaling ladder), not a number.

### 2. The levers, largest to smallest

| lever | what you do | time saved | what it costs |
|---|---|---|---|
| **Start from a code-pretrained base** (Qwen2.5-Coder-0.5B, SmolLM2-360M) instead of teaching syntax | skip phase 1 entirely — the base already parses Rust. Train only phase 2: source → compact records | **5–10×** (1 B → 100–200 M tokens) | nothing real; "learn the syntax first" was the expensive part and it is already paid for in the base |
| **Shrink the input, not the labels** | train on item-level windows (one fn + the signatures it references, ~300–500 tokens) instead of whole files; dedupe vendored/generated/test code (top-100 repos are maybe 40 % redundant) | **2–3×** | some cross-item context; fine for L2–L4, which are per-item anyway |
| **Smaller model** 0.5 B → 135 M → 50 M | | 3.7× / 10× | capacity for receiver typing (65 % of call edges) is what is at risk; 50 M probably fails it |
| **Narrow the task first** | phase 2a = item skeleton only (L2); phase 2b adds definitions + types (L3–L4) | the L2 model converges in ~10 M tokens | you get the public-skeleton extractor early and cheaply |
| LoRA, sequence packing, MLX fused kernels | | 1.3–2× | none, but these do not change the order of magnitude |

### 3. Combined

```text
   H-46 as written        0.5 B model, 1 B tokens, phases 1+2       3×10^18 FLOPs   ~2 weeks  Mac M4 Pro   ~2 h  H100
   with levers 1+2        0.5 B model, ~100 M tokens, phase 2 only  3×10^17 FLOPs   ~1.5 days Mac M4 Pro   ~15 min H100
   with levers 1+2+3      135 M model, ~100 M tokens, phase 2 only  8×10^16 FLOPs   ~8 h      Mac M4 Pro   ~4 min  H100
```

### 4. The real shortcut: a scaling ladder instead of a committed dataset size

```text
   step 0   zero training: few-shot the base model with the record schema  →  X₀   (1 h; measures what is already there)
   step 1   1 M tokens    → X₁      (~10 min on the Mac)
   step 2   10 M tokens   → X₂      (~2 h)
   step 3   100 M tokens  → X₃      (~1 day)                       stop as soon as X flattens
   held-out repos fixed across all steps; score at int4 (the shipped artefact)
```

Three points give the log-linear curve. By step 2 you know whether 100 M tokens reaches ~95 % or sits on the ~78 % (Cassini) ceiling — before spending the day. Fine-tuning curves of this kind usually flatten by 10–50 M dense label tokens, and rust-analyzer labels are dense (every output token is supervision), so the knee should come early.

X = held-out exact-match on (a) item skeleton, (b) definition path for 1 k sampled tokens, (c) receiver type for 1 k method calls — the H-46 metrics.

### 5. Do not cut

- Held-out **repos**, not held-out files (otherwise you measure memorisation).
- The `unresolved` / abstain examples (otherwise the model learns to invent edges; silent omission and invention are the failure modes that matter — see H-42).
- Evaluation at int4 — cutting it saves no training time and makes the result unusable for the shipped artefact.

### 6. One-line version

Use a code-pretrained base, train only the record-emission task on item windows, and climb 1 M → 10 M → 100 M tokens while watching the held-out curve; expect to know the answer in ~2 hours of Mac GPU time and to finish in a day, not two weeks.

### 7. ELI5 ASCII — the levers

```text
THE BILL:   time  =  6  ×  (model size)  ×  (tokens you feed it)        ÷  (how fast the chip is)
                                 ▲                 ▲                              ▲
                              lever 3           levers 1, 2, 4                 lever 5
                              (small)           (BIG)                          (fixed: you own one Mac)


LEVER 1 — don't teach the alphabet twice                   saves 5–10×

   old plan:    [ learn Rust syntax ]────────►[ learn to emit records ]      1,000 M tokens
                 ▲ 900 M tokens here
   shortcut:                                  [ learn to emit records ]        100 M tokens
                 "Qwen-Coder already reads Rust. Only teach it OUR output."


LEVER 2 — feed snacks, not the whole fridge                 saves 2–3×

   whole file in:   ┌────────────────────────────────────────┐ 3,000 tokens  → 1 label set
                    └────────────────────────────────────────┘
   item window in:  ┌──────┐ 400 tokens: this fn + the 5 signatures it touches → same labels
                    └──────┘
   + throw out duplicates (vendored / generated / test boilerplate ≈ 40 % of the top-100 repos)


LEVER 3 — a smaller brain                                   saves 3.7× (135 M)  or 10× (50 M)

   500 M  ■■■■■■■■■■  can hold "what type is x?" for 65 % of call edges   ✓
   135 M  ■■■        probably                                             ?
    50 M  ■          probably not                                         ✗   ← the lever with a cliff


LEVER 4 — start with the easy exam                          first useful model after ~10 M tokens

   2a   source  →  item skeleton (what's pub, what's a fn/struct/trait)        easy, converges fast
   2b   source  →  + where each name is defined, + the type of each receiver   hard, needs more tokens
   ship 2a while 2b trains.


LEVER 5 — tune the engine                                   saves 1.3–2× ; doesn't change the order of magnitude
   LoRA, sequence packing, MLX fused kernels


ADD THEM UP (Mac mini M4 Pro)

   H-46 as written           ██████████████ 2 weeks
   + lever 1 + 2 (0.5 B)     ██ 1.5 days
   + lever 3 (135 M)         ▌ 8 hours
   (same, one rented H100)   ▏ 4–15 minutes


THE SHORTCUT THAT ISN'T A LEVER — climb, don't jump

   tokens      1 M        10 M        100 M
   time       10 min      2 h         1 day
   score X    X₁ ───────► X₂ ───────► X₃
                            ▲
               by here you can see whether the line heads to 95 % or flattens at ~78 %
               → stop early if it flattens; only buy the day if the line is still climbing

   same 10 held-out REPOS at every rung    ·    score at int4    ·    keep the "unresolved" examples
```
