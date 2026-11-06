# Deep Insight 01: Parseltongue as an LLM-facing CODEMAP of current and future Rust systems

Written originally 2026-11-05; thesis revised 2026-11-06 on `research/tiny-rust-code-mapper`.

Status: product and representation proposal, not a claim that the complete system exists. This revision replaces the earlier cold-start-led positioning. Earlier performance estimates remain research hypotheses, summarized below; the original wording remains in Git history.

## 1. The thesis

**Parseltongue should describe the Rust codebase that exists, let an LLM propose the codebase it wants, and check the difference.**

Its core representation has two linked levels:

1. File dependencies: which source files depend on which other files, and why.
2. Public-interface dependencies: which boundary-visible contracts depend on which other contracts, and how those contracts map to files, declarations, implementations, and exported names.

The graph generator remains central. The graph becomes something an agent can navigate, discuss, propose changes to, and verify against code. A tiny local model and a CLI/MCP tool can be complementary parts of the product, not competing definitions of it.

The product promise is:

> Understand what connects today. Propose what should connect tomorrow. Verify what actually changed.

The differentiator is not automatically fastest search, replacing rust-analyzer, or drawing a more attractive diagram. The hypothesis is that a reliable, compact, bidirectional map improves an agent's ability to understand and change large Rust systems while making the change inspectable by a human.

## 2. What a CODEMAP is

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

### 2.1 Minimum conceptual record set

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

### 2.2 Public means visible across a specified boundary

For large Rust systems, external crate APIs alone are often too narrow a view of internal architecture. Proposed default: include externally accessible APIs and internal cross-module contracts, labeled by their effective visibility scope.

Rust has unrestricted and restricted visibility, ancestor-module constraints, and re-exports. A `pub` token alone does not establish external reachability; a file is not necessarily a Rust module. Store declared visibility, effective accessibility where established, declaration path, and exposure path separately. See the [Rust visibility reference](https://doc.rust-lang.org/reference/visibility-and-privacy.html).

The exact inclusion policy for internal contracts remains a product decision to validate. Do not silently reinterpret crate-private items as externally public.

### 2.3 Typed edges prevent misleading answers

Start with a compact, versioned vocabulary. Useful distinctions include `parameter_type`, `returns`, `field_type`, `associated_type`, `implements`, `reexports`, `calls`, and `may_call`.

An interface mentioning a type is not proof of a call to that type's methods. A trait implementation is not proof that a particular runtime invocation reaches it. A call graph is not a complete control-flow or data-flow graph.

Forward and backward traversal should specify relation filters. "Who depends on this error type?" and "Who calls this function?" are different queries, even though both traverse edges backward.

## 3. Current and proposed maps share vocabulary, not authority

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

### 3.1 Example: decoupling checkout from storage

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

### 3.2 The three-way comparison

After implementation, compare:

1. The before map: what was true when the proposal was made.
2. The intended map: what the approved proposal requested.
3. The after map: what the changed source actually establishes.

Report achieved changes, missing changes, unexpected changes, and unverifiable regions. Scope the comparison to the proposal: unrelated repository changes are not automatically architectural violations.

The underlying idea has precedent in [software reflexion models](https://www.cs.ubc.ca/~murphy/papers/rm/fse95.html), which compare high-level and extracted source models through mappings. Our product hypothesis is an agent-friendly, incrementally maintained Rust workflow around that comparison, not the invention of design conformance analysis.

## 4. How agents use it on large codebases

### 4.1 Navigate both ways

An agent searches for a concept, anchors it to an interface or file, expands a bounded neighborhood, and opens the exact source evidence needed for the task. It can start from a file and discover contracts, or start from a contract and discover its files and consumers.

Backward traversal supports change assessment. Forward traversal supports dependency investigation. A path query can explain how two apparently unrelated areas are connected, subject to the available relation types and evidence.

Every result should disclose snapshot, filters, truncation, and analysis gaps. A small response is useful only if the agent knows what was left out. Start with simple depth and token bounds; introduce sophisticated ranking only when task evaluation justifies it.

### 4.2 Design before editing

Given a goal such as "separate persistence from parsing," an LLM proposes alternative graph patches. Parseltongue compares the affected contracts, dependency directions, compatibility risks, required implementation work, and unresolved assumptions.

Files and interfaces can be grouped into named subsystems as a view. This need not add a third extraction level or an elaborate architectural ontology to the first version.

The LLM supplies candidate designs and rationale. The tool supplies grounded neighborhoods, structural checks, and later comparison with implementation. A tiny model may contribute extraction or ranking; it need not be a general-purpose architect.

### 4.3 Review against intent

A maintainer should be able to ask: "Did this patch move us toward the design we approved?" The response should identify changed contracts and dependencies, not merely repeat the file diff.

This is a possible trust mechanism for agent-generated changes. It complements source review and tests; it cannot certify a PR as safe from graph agreement alone.

## 5. The failure and trust model

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

## 6. Where the tiny model belongs

The product can be a local CLI/MCP tool plus a task-specific tiny model. The model contributes to the CODEMAP contract; the host system owns scanning, snapshotting, bounded retrieval, validation, reconciliation, and queries.

Possible learned tasks include selecting among candidate bindings, proposing unresolved dependency records, identifying useful next context, or ranking explanations of missing analysis. These are alternatives to evaluate, not capabilities already established.

Direct source-to-graph generation remains a legitimate research path. Its output needs schema validation, provenance, abstention, and evaluation on unseen repositories. A fallback proposal is not confirmed merely because the primary resolver could not resolve it.

The consuming coding LLM can use the resulting map to propose designs. Training a small model to reproduce observed graphs does not, by itself, train architectural judgment or validate future-map quality.

### 6.1 What Deep Insight 02 contributes

[Deep-Insight-02](Deep-Insight-02.md) proposes a pretrained base, item-level examples, separate skeleton and resolution tasks, a training-data scaling ladder, held-out repositories, and evaluation of the quantized artifact.

The transferable lesson is to define the record-emission task and its evaluation before committing to a large training run. Its training times, memory projections, accuracy expectations, and claimed savings are estimates, not measured product guarantees.

Item windows must contain context available at inference time. Giving the model "the signatures this function references" can leak the answer if identifying those references is the task. Context retrieval failures belong in evaluation, not outside the benchmark.

### 6.2 What the earlier performance thesis does and does not establish

Retain cold-start latency, low memory, offline operation, and usefulness on partial projects as candidate advantages. Do not present them as proven wins or universal disadvantages of rust-analyzer.

- Compare equivalent workloads: time to a useful answer, cold and warm runs, graph scope, and correctness. One file is not an entire workspace.
- No `Cargo.toml` does not automatically mean rust-analyzer cannot support the project; alternative project configuration may be available.
- A model trained on resolver outputs is not guaranteed to match the resolver, nor mathematically forbidden from outperforming a particular resolver run. Independent evidence is needed for either claim.
- A reported accuracy from another model/task is not a ceiling for this task. No fixed 78% or 95% outcome has been established here.
- Artifact size is not runtime RAM. Quantized weights, tokenizer, context/cache, temporary buffers, and host allocations all count toward the measured envelope.
- A sub-100 MB learned helper does not imply a sub-100 MB whole-repository analyzer. State separately whether a budget covers the model process, the CLI plus model, or all resolver subprocesses and caches.

The [semver experiment](../experiments/public-interface-semver/README.md) gives a concrete warning: source-only agents identified all seven exported nominal types and thirteen public inherent methods in that assignment, but found only ten of nineteen scored local field/signature dependencies. That is evidence of omissions in a narrow experiment, not a general model accuracy estimate.

## 7. Product options and recommendation

| Option | Value | Limitation |
| --- | --- | --- |
| Read-only navigation map | Faster grounded orientation and dependency lookup | Does not capture architectural intent |
| Navigation plus design workspace | Propose contract changes and verify implementation against intent | Requires stable mapping, proposal semantics, and useful comparison |
| Autonomous architecture generator | Attempts end-to-end design selection and implementation | Adds an unvalidated architecture-quality problem and much greater trust burden |

Recommend the second as the product direction, with navigation as its first usable increment. This keeps both current and future maps in view without making autonomous design quality a prerequisite for usefulness.

## 8. First experiment and success criteria

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

## 9. Sources, decisions, and open questions

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

# Appendix A — Deep Insight 01 as originally written (2026-11-05, preserved verbatim, append-only)

*Commit `c52bb41bb` replaced the body of this file with the CODEMAP thesis above. Per the append-only rule for research notes, the full earlier text is kept below unchanged (its own heading levels shifted one down). Nothing in it is retracted here; §6.2 above says what of it still stands.*

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

Append-only extension, 2026-11-06: the product scenarios and ELI5 walkthrough below supplement the earlier thesis. All preceding research text, including Appendix A, is preserved unchanged. The four paths are alternative starting jobs for the same CODEMAP foundation, not four simultaneous product commitments.

## 10. Four product timelines and recommendation

The choice is not "tiny model versus CLI." It is which repeated job makes their combination valuable. All four paths retain file dependencies, interface dependencies, source mappings, and separate current/proposed maps. They are conditional scenarios, not market forecasts or a plan to build four products simultaneously.

### 10.1 Timeline A: the local navigator

**Combination: tiny model + CLI + MCP.** Help a coding agent navigate unfamiliar Rust code. The model contributes bounded graph records or missing-binding proposals; the host manages source evidence, queries, and updates.

In week one, compare real navigation tasks with and without the graph and learned additions. In month one, package useful navigation. In the first quarter, look for repeat use on unfamiliar repositories. If this works, Parseltongue could become a routinely installed Rust companion over the following year.

The main risk is fewer tokens but worse answers. Users abandon it if checking the map costs more effort than exploring the source. Daily use can be low-ceremony, but the builder inherits installation support, extraction edge cases, and freshness problems. A meaningful success signal is voluntary return usage with preserved or improved task correctness.

### 10.2 Timeline B: the design workspace

**Combination: current map + proposed map + implementation verification.** Start with a real refactoring, such as separating storage from parsing. The coding LLM proposes interface and dependency changes; the maintainer approves them before implementation.

In week one, compare alternatives for one disputed boundary. In month one, complete a before/intended/after cycle. In the first quarter, test repeated use across cross-cutting changes. Over a year, accepted proposals and their implementation outcomes could become useful design history.

The main risk is another document nobody maintains. The decisive signal is a maintainer voluntarily creating a second proposal. This path costs more attention before coding, but may reduce late-stage disagreements and rewrites. The tool must allow evidence from implementation to improve the design, not merely label every deviation as disobedience.

### 10.3 Timeline C: the intent-aware reviewer

**Combination: graph deltas + CLI/CI + a small learned assistant.** Start after a change exists. Show changed contracts, new dependencies, affected consumers, and discrepancies from approved intent. When no approved intent exists, provide a descriptive change report rather than invent an authoritative plan.

In week one, replay real PRs and deliberately seeded mismatches. In month one, produce advisory reports with evidence links. In the first quarter, test reviewer retention and introduce only narrowly scoped, validated checks. Over a year, this could become a reliable change-review companion.

The main risk is another noisy review bot, or false reassurance from an incomplete graph. A small model may improve coverage or rank findings, but its conclusions should not automatically block merges. Success means reviewers repeatedly act on useful findings; failure means notification fatigue and repeated manual corrections.

### 10.4 Timeline D: the reusable graph component

**Combination: specialized model + runtime + SDK, with a reference CLI.** Make the graph generator reusable inside other coding tools. Model artifacts, evaluation data, and a stable record format become distribution assets.

In week one, freeze a narrow task, record schema, and memory boundary. In month one, test generalization on fresh crates. In the first quarter, work with independent tool builders on their own workloads. Over a year, retained integrations could make the component useful infrastructure rather than a one-off demonstration.

The main risk is months of research without integration demand. This is the most model-centered path and the highest-uncertainty starting point under current evidence. The builder spends more time on data, runtimes, licenses, and compatibility; an external integration that becomes worse when the component is removed is a stronger signal than downloads alone.

### 10.5 Cross-timeline comparison

| Path | Strongest opportunity | Critical uncertainty | Reversibility |
| --- | --- | --- | --- |
| A: Navigator | Frequent daily use | Does it improve actual coding tasks? | High: graph and runtime support the other paths |
| B: Design workspace | Closest to the full CODEMAP thesis | Will teams record intent before implementation? | Medium-high if proposal records stay portable |
| C: Reviewer | Concrete, inspectable review value | Can useful findings outweigh noise? | High for the graph/diff core; lower for CI-specific work |
| D: Graph component | Reuse across agent products | Does learned capability generalize and earn integrations? | Medium: data and runtime remain useful, bespoke training may not |

### 10.6 Decision filter

Recommend **A with one small B/C loop**, not four simultaneous products: navigate the current map, propose one structural change, implement it, and inspect whether the resulting map matches the proposal. Navigation is the first usable increment; current-to-future design remains the product direction.

Evaluate three configurations on comparable tasks: the normal coding agent, the agent plus graph tooling, and the same tooling with the tiny model enabled. This separates the graph's value from the model's incremental value. Keep candidate retrieval and validation comparable between the latter two configurations.

Move toward B when maintainers repeatedly plan with the map, toward C when they repeatedly use its review report, and toward D when independent tool builders retain the component. If learned additions disappoint, narrow the learned task and report the result; a tool-only success does not validate the model hypothesis.

The sub-100 MB CPU target remains explicit, but model-process memory must not be confused with whole-application memory. The detailed evidence, scenarios, failure branches, and pilot gates are in [Deep Insight 03](Deep-Insight-03.md).

## 11. ELI5: a map, a plan, and a check

**Parseltongue helps an agent see how Rust code connects,
plan a change, and check whether the code followed the plan.**

Think of a large codebase as a workshop with many labeled
drawers. The map shows which drawers matter and which parts
fit together. It does not replace opening a drawer when you
need to inspect the actual part.

This is a proposed product, not a claim that every part is
already built. The tiny model helps with a narrow map task;
it does not need to be a general-purpose coding assistant.

### 11.1 Two levels: where things live and what connects

The file view tells you where to look. The interface view
tells you which named contracts create the connection.

```text
FILE VIEW

+------------------------+
| checkout.rs            |
+------------------------+
            |
            | depends on
            v
+------------------------+
| order.rs               |
+------------------------+

INTERFACE VIEW

+------------------------+
| Checkout::submit       |
+------------------------+
            |
            | takes a parameter of type
            v
+------------------------+
| Order                  |
+------------------------+

SOURCE MAPPING

checkout.rs
  +-- Checkout::submit(order: &Order)

order.rs
  +-- Order
```

An interface is a contract another part of the system can
use. Here the signature tells us that submitting needs an
Order. It does not tell us which methods submit calls.

Some contracts are visible only inside a crate or module.
The map labels that scope instead of calling all of them
externally public. Private code can still create important
connections underneath the simple view.

### 11.2 Two directions: follow a need or find its users

The same arrow supports two different questions. Reading
it backward does not create a new kind of dependency.

```text
Checkout::submit --parameter_type--> Order

FORWARD: start at Checkout::submit
  "Which types does this signature need?"
  Answer: Order

BACKWARD: start at Order
  "Which signatures mention this type?"
  Answer here: Checkout::submit

ZOOM IN: follow the source mapping
  "Show me the declaration and its evidence."
```

For a larger repository, the answer may be incomplete.
The tool must say whether results were cut off, analysis
failed, or some source was outside its supported scope.

### 11.3 The small model and the tool have different jobs

The model proposes answers to small graph questions.
Ordinary code manages files, records, checks, and queries.
The coding agent uses the result to do the larger task.

```text
+------------------------+
| CLI + graph engine     |
+------------------------+
            |
            | asks a bounded question
            v
+------------------------+
| Tiny local model       |
+------------------------+
            |
            | proposes graph records
            v
+------------------------+
| Evidence checks        |
+------------------------+
            |
            | keeps claim status explicit
            v
+------------------------+
| Queryable CODEMAP      |
+------------------------+
            |
            | supplies a small neighborhood
            v
+------------------------+
| Your coding agent      |
+------------------------+
```

This shows the learned path. Source-backed records can
also enter the map without going through the model.
Evidence checks may leave a proposal unverified; they
cannot magically make every answer true.

The small model's memory is only part of the bill. Files,
graph storage, runtime buffers, and any resolver processes
must be measured too. A small download is not proof of a
small whole-application memory footprint.

### 11.4 A plan is not the same thing as reality

Suppose checkout currently knows about one database.
We want it to use a shared storage contract instead.

```text
BEFORE: what the old code shows

  Checkout --depends_on--> PostgresStore

INTENDED: what we agree to build

  Checkout      --depends_on--> OrderStore
  PostgresStore --implements--> OrderStore

AFTER: what the changed code actually shows

  Extract again. Do not copy the intended arrows.
```

OrderStore is a proposed trait in this example. The goal
is to support another backend, not simply to add boxes.
For a single-backend product, the extra trait might be
unnecessary. The goal determines whether it is useful.

```text
+------------------------+
| Before map             |
+------------------------+
            |
            | discuss a scoped change
            v
+------------------------+
| Approved intended map  |
+------------------------+
            |
            | implement and re-extract
            v
+------------------------+
| Actual after map       |
+------------------------+
            |
            | compare with before and intent
            v
+------------------------+
| Change report          |
+------------------------+

The report separates:
  Achieved       The intended change is present.
  Missing        The intended change is absent.
  Unexpected     Something else changed in scope.
  Unverifiable   Available evidence cannot decide.
```

Only classify something as missing when the supported
analysis can establish that absence. An unknown is not
a failure or a success. Tests and code review still
check behavior the map does not describe.

### 11.5 Four products can use the same foundation

These are four starting jobs, not four required apps.

```text
CODEMAP + small model + host tool
  |
  +-- A. NAVIGATOR
  |     "Help me find the right code."
  |
  +-- B. DESIGN WORKSPACE
  |     "Help me plan different connections."
  |
  +-- C. REVIEW COMPANION
  |     "Show whether the change matches the plan."
  |
  +-- D. REUSABLE COMPONENT
        "Let my coding tool build this map too."
```

A gets daily navigation feedback. B tests whether teams
will plan with the map. C tests whether reviewers find
the changes clearer. D tests whether other builders
need the model and runtime inside their own products.

### 11.6 Start with one complete, small loop

The recommendation is A plus one small piece of B and C.
Do not build a whole design environment to test the idea.

```text
1. Find the relevant code.        [A: navigate]
                |
                v
2. Agree on one graph change.     [B: plan]
                |
                v
3. Implement and extract again.   [ordinary coding]
                |
                v
4. Compare intent with reality.   [C: review]
```

Test the model separately from the map. Compare the same
agent using normal tools, the graph tool without learned
additions, and the graph tool with the model enabled.

If the map helps but the model does not, that is useful
learning. It is not proof that the model succeeded.

### 11.7 Legend and reading notes

```text
Box              A component or named code item.
Labeled arrow    The specific relationship or action shown.
Source mapping   Where to inspect the underlying code.
Observed         Supported by evidence for this snapshot.
Hypothesized     A claim about today that needs checking.
Intended         A choice about tomorrow, not today's fact.
```

The diagrams are simplified teaching examples. An arrow
does not imply a runtime call unless it says so. A graph
that matches the plan does not prove the program is safe.

**See the connections. Agree on the change. Check the code.**
