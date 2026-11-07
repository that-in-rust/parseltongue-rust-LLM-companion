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
Part 5   2026-11-06   Product summary (four timelines + recommendation) and ELI5 walkthrough of the CODEMAP
                      (appended upstream as b55d7451e; §10 summarises Part 3, §11 is the plain-language version of Part 2)
Part 6   2026-11-06   Proposal 01 — the one big thing: structure as a shared, checkable object between developer, LLM and compiler
                      (verbatim copy of Deep-Insight-04.md: synthesis, simulated reception by Devin and by GLM-5.3, where they disagree)
Part 7   2026-11-06   Proposal 01 restructured top-down by the Minto pyramid principle
                      (verbatim copy of Deep-Insight-05.md: SCQA intro, governing thought, three key lines, residual risk)
Part 7b  2026-11-06   Deep-Insight-05 revised: the thesis stated without source references or attributions
                      (verbatim copy of the revised Deep-Insight-05.md; the first version stays as Part 7)
Part 7c  2026-11-06   Deep-Insight-05 revision 3: verification / diff / as-built removed from scope; the future is the not-yet-exact part of one map
                      (verbatim copy; Parts 7 and 7b stay)
Part 7d  2026-11-06   Deep-Insight-05 revision 4: runtime = own CLI tools × tiny models (CPU or GPU); rust-analyzer/tree-sitter as teacher and judge only;
                      the future slot is never filled by the tool; §4 system definition and §5 entity model with authority matrix (verbatim copy)
Part 7e  2026-11-06   Deep-Insight-05 as a standalone document (no references to earlier notes), Minto structure, Shreyas-Doshi voice (verbatim copy)
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


---

# Part 5 - 2026-11-06 - Product summary and ELI5 walkthrough

Append-only extension, 2026-11-06: the product scenarios and ELI5 walkthrough below supplement the earlier thesis. All preceding research text is preserved unchanged. The four paths are alternative starting jobs for the same CODEMAP foundation, not four simultaneous product commitments.

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

---

# Part 6 — 2026-11-06 — Proposal 01: the one big thing, structure as a shared, checkable object (verbatim copy of Deep-Insight-04.md)

*Everything in this Part is the **first proposal** (Proposal 01) for the product, not a settled design.*

*Devin's synthesis of Parts 1–5 against the founder's statement, plus GLM-5.3's independent reception simulation and the agree/disagree table. Kept in Deep-Insight-04.md too.*

*2026-11-06. Written after reading Deep-Insight-01 Parts 1–5 (the original differentiation, the CODEMAP rewrite, four futures, training shortcuts, ELI5 walkthrough) against the founder's own statement of the product. §1–§5 are Devin's synthesis; §6 is GLM-5.3's independent read of the same material (verbatim, via the zcode SOP); §7 says where the two disagree. Append-only: later revisions go in as new sections or as a new Part of the collected Deep-Insight-01.*

### 0. The founder's statement (verbatim)

> a Rust LLM companion that is a CLI tool as well as an LLM, bidirectionally talking to each other. It helps you map out your large Rust codebase, but also helps you plan the structure of your Rust codebase. That dependency graph will be current and future, and it will also work for broken code. It will have at least two levels: file-to-file dependencies; and public-interface dependencies, or at least everything that is public interface, like comments, imports, and global statements. These are the things that neither rust-analyzer nor any other code MCP can do right now.

### 1. The one big thing

**Make the structure of a Rust codebase — what connects to what — a first-class object that a human, an LLM and the compiler can all read, propose changes to, and check against each other, at every moment including the moments when the code does not compile.**

Everything in Parts 1–5 is a piece of that one object or a way of getting it:

```text
                        ┌───────────────────────────────────────────────┐
                        │   THE MAP   (file graph + public-interface graph) │
                        │   three time-states, same vocabulary:              │
                        │     AS-IS      what the code establishes today     │
                        │     INTENDED   what we agreed to build             │
                        │     AS-BUILT   what the changed code establishes   │
                        └───────────────────────────────────────────────┘
        reads / writes ▲                 ▲ builds / judges                ▲ reads / proposes
                       │                 │                                │
                 DEVELOPER         CLI + COMPILER/RESOLVER             LLM AGENT
             approves intent,     extracts, stores, diffs, checks,    navigates, explains,
             reviews the report   keeps provenance honest             drafts the intended map

   Part 1  →  REACH: the map still exists when the resolver cannot run (student model / tree-sitter skeleton)
   Part 2  →  the map itself: records, evidence envelope, as-is vs intended vs as-built, threat model
   Part 3  →  which JOB sells the map first (navigate / design / review / component)
   Part 4  →  how cheaply the fallback extractor can be trained
   Part 5  →  the same thing in plain words
   grep-replacement note / seven-event protocol  →  how the LLM READS the map in 3,000-token pieces
   Sikos/MISP note  →  how facts, vocabulary and opinions stay separate INSIDE the map
```

The analogy that fits: **blueprint, as-built drawing, and building inspector.** Every physical engineering discipline keeps a drawing of what was intended, a drawing of what was built, and someone whose job is to diff them. Software has the code and nothing else; agents inherit that poverty and re-derive the architecture from grep on every task. rust-analyzer is a superb instrument for the as-built drawing at the level of a single cursor. Nothing holds the intended drawing, and nothing diffs the two.

Why "bidirectional" is the right word and not decoration:

```text
   CLI → LLM     bounded neighbourhood · blast radius · path · gaps/unresolved · the before/intended/after report
   LLM → CLI     a question (7 words) · a proposal (graph patch) · an annotation (L6 meaning) · a sighting (confirm/deny an edge)
   CLI → compiler   extract, re-extract, validate a proposed edge
   compiler → CLI   exact facts, or "cannot resolve" — which the map records rather than hides
```

The loop is the product. A one-way "LLM reads a graph" tool is Sourcegraph-for-agents; a one-way "LLM writes a plan" tool is an ADR generator. The thing nobody has is the round trip: *propose → build → re-extract → compare → report*, with the map staying usable while the code is mid-change and broken.

### 2. Why "works for broken code" is the hinge, not a footnote

The code is most often broken **during exactly the work the tool is for**: a cross-cutting refactor. Halfway through extracting `OrderStore` from `PostgresStore`, 40 files fail to type-check, rust-analyzer's type information degrades to `{unknown}` across the blast radius, and the agent is flying blind at the moment it most needs the map. The intended graph is the only stable thing in the room. Two things make the map survive that moment:

1. **Level 1 and the public skeleton need no type-checker.** `mod`, `use`, `extern crate`, `pub fn` signatures, `pub struct` fields, `static`/`const`, doc comments, `#[cfg]` — tree-sitter reads these from a file that does not compile (measured: 0.07 s for 42k LOC). So the file graph and the interface *inventory* are always available; only resolution degrades.
2. **Provenance tells the agent what degraded.** Each edge is `exact | projected | proposed | unresolved`; mid-refactor the share of `unresolved` rises and the report says so, instead of silently returning fewer edges (Part 2 §5 "false completeness").

This reframes Part 1: the student model is not "faster than rust-analyzer", it is **the thing that keeps L3/L4 edges flowing while the compiler is unhappy** — same job as the tree-sitter skeleton, one rung higher.

### 3. The two levels, made precise

```text
   LEVEL 1  file ↔ file             edge kinds:  mod-declares · use-imports · include!/path attrs · build-script/cfg gates
                                    source:      tree-sitter; always available
   LEVEL 2  public surface of each file ("everything that is public interface"):
              doc comment of the file / module          → L6 anchor the LLM can read first
              use / extern crate / pub use (re-exports)  → EXPOSES_AS / REEXPORTS
              pub items with full signature             → SIGNATURE_USES / PARAMETER / RETURNS / IMPLEMENTS
              pub static / const / lazy globals         → the "global statements"
              #[cfg] / feature gates on all of the above
              macro invocations at item level (name + span; expansion when the resolver is available)
            edges between surfaces = the public-interface dependency graph
   LEVEL 3+ private bodies, calls, flow — kept, shown on expansion (the earlier L3–L6 ladder); not part of the default view
```

The test for "is it in level 2": *could this line change what another file is allowed to write?* Doc comments pass (they are the contract's prose), `use` lines pass, globals pass, a private helper body fails.

### 4. Simulated reception (Devin's run; GLM-5.3's is in §6)

```text
 persona                        first reaction                                   what converts them
 ─────────────────────────────  ───────────────────────────────────────────────  ─────────────────────────────────────────────
 maintainer, 300k-LOC Rust DB   "another architecture doc nobody updates"        intended map is drafted FROM the PR text by the
                                                                                  agent; the report catches one real boundary leak
 rust-analyzer contributor      "RA has 90 % of this; be an RA consumer"          we ARE an RA consumer; we contribute a `codemap`
                                                                                  export upstream rather than reimplement
 agent-harness builder          "show me fewer tool calls, same pass rate"        RQ1 A/B: tool calls, tokens, correctness on
                                                                                  real tasks (Part 3's three-arm pilot)
 HN top comment                 "LLM code graphs always go stale; macros?"        provenance tags + the measured macro/proc-macro
                                                                                  numbers; incremental re-extract in <1 s
 newcomer in Cursor             "I just want to understand this repo"             the map alone (Timeline A); they never touch intent
 CTO, 40-person team            "CI noise, false confidence, one more bot"        advisory-only report, seeded-mismatch recall,
                                                                                  no merge gating until measured
 ML researcher                  "tiny model beats RA? no."                        we never claim that; the ablation arm isolates
                                                                                  the model's contribution; honest negatives published
 security engineer              "unsafe surface and blast radius, please"         `unsafe_surface`, `impact`, RustSec seeds (H-49)
                                                                                  — a first-class query, labelled heuristic
```

The pattern: every persona's objection is some form of *"will it be fresh, honest, and worth the ceremony?"* — which is why Part 2's threat model and the MISP fact/vocabulary/opinion split are not academic: they are the answer to the objection.

### 5. The 90-second demo that shows the one big thing

```text
  0:00  pt map .                      → "11 crates, 1,204 files, 9,318 public items; 97 % resolved; 3 % unresolved (proc-macro)"
  0:10  pt ask "where do retries get decided"
                                      → 4 candidates, picks RetryPolicy::should_retry; neighbourhood, 2,400 tokens, blast radius 31
  0:25  pt propose "make retry policy pluggable behind a trait"
                                      → the LLM drafts a graph patch: +trait RetryPolicy, Client→trait instead of →FixedBackoff,
                                        FixedBackoff implements RetryPolicy; 6 consumers listed; maintainer approves
  0:40  agent edits; halfway the crate does not compile (14 errors)
  0:50  pt check                      → still answers: "intended 3 / achieved 2 / missing 1 (FixedBackoff lacks impl) /
                                        unexpected 1 (Client gained a dep on tokio::time) · 9 edges unresolved while broken"
  1:10  agent fixes; cargo check green; pt check → "3/3 achieved, 0 unexpected"
  1:25  pt report → the PR description: before/intended/after, with evidence links.
```

What must be true for this to work: L1+L2 extraction on non-compiling files; stable item identity across a rename/move (Part 2 §5 "identity churn"); a proposal format the LLM can emit and the CLI can diff; incremental re-extract fast enough to run on every save. None of these needs the student model. The model enters only at 0:50, to keep L3 edges alive while the compiler is down.

### 6. GLM-5.3's independent read (verbatim)

*Run 2026-11-06 via `~/bin/zcode` (Claude Code → Z.ai Coding Plan, model glm-5.3, read-only tools), prompt at `/home/ubuntu/zcode-runs/prompt-reception-di04.txt` (reproduced in `devin-research/sop/SOP-zai-glm-coding-plan-second-opinion.md` style). GLM read the collected Deep-Insight-01 Parts 1–5, the grep-replacement note and the resolver-builds thesis; it did not see §1–§5 above. 2,326 words, unedited except heading levels. Everything in this section is GLM-5.3's opinion, stored as an attributed annotation, not as a fact of the research.*

#### 1. The one big thing

**Parseltongue is trying to make the dependency graph itself the working interface between a coding agent and a codebase — a versioned map that is read to navigate, patched to plan, and diffed to verify, with every edge carrying its provenance.**

The graph is not the output; it is the shared medium. Everything else is one of four verbs performed on that medium. Part 1 supplies the positioning discipline — Shape (agent-shaped, budgeted queries over exact facts) over Reach (broken-code fallback) over Honesty (per-edge provenance), and the warning that leading with the fallback positions you as "an LSP for broken days." Part 2 supplies the actual data contract — the CODEMAP record set (§2.1), typed edges (§2.3), and the one genuinely novel mechanism in the whole corpus: current map / proposed graph patch / after map as three distinguishable states, compared (§3, §3.2). Part 3 supplies go-to-market: four jobs (navigate, plan, review, embed), with the recommendation "A plus one small B/C loop." Part 4 is an internal lab note on cheapening the training experiment. Part 5 restates 3 and 2 in plain language. The unifying idea is strong precisely because navigation, planning, and review are the same map in three tenses — present, future, past. The corpus's own drift (grep-replacement → CODEMAP → four futures) is the search for this sentence.

#### 2. Is the founder's framing right?

| Claim | Verdict | Evidence |
|---|---|---|
| Bidirectional CLI↔LLM | **Partly** | As settled in `1793882149` ("resolver builds, LLM steers"), the loop is LLM-asks / tool-answers — which is just every MCP tool (Serena, CodexGraph). The genuinely bidirectional part is the *proposal* loop (Part 2 §3), where the LLM writes into the map. Also, "an LLM" is ambiguous: the in-tool tiny model's only shipped evidence is 0 accepted bindings vs 56 by a rule (`1793879701` §3). |
| Map + plan | **Partly** | Mapping: cargo-modules already draws file/module dependency graphs; Sourcegraph SCIP gives precise refs and call hierarchies at scale; rust-analyzer holds all of it internally. Planning as before/intended/after graph patches: precedent is reflexion models (1995) and Moderne/OpenRewrite's recipe diffs (Java, recipe-driven, not LLM-proposed). Nobody ships this for Rust. Novel, demand unproven. |
| Current + future graph | **Partly true, unvalidated** | The "future" graph is a record format (Part 2 §3.1), not a demonstrated capability. ArchUnit (Java) asserts intended architecture but doesn't author it with an LLM. This is the most defensible novelty and the least evidenced. |
| Works for broken code | **Partly, and oversold** | rust-analyzer supports detached files and non-Cargo projects (`rust-project.json`, Bazel) — Part 3's own "External grounding" concedes this. Every tree-sitter indexer (Aider repo map, Cursor, Serena's fallback) already "works" on uncompilable code syntactically. The only new version — a model drawing resolved edges without a resolver — currently has negative evidence (0-vs-56 bindings; semver experiment agents found 10/19 edges, Part 2 §6.2). And cargo-semver-checks needs the crate to *build*, so the fallback niche is real but smaller than claimed. |
| Two levels incl. comments/imports/globals | **Partly, with a category error** | Each level exists separately: file graph (cargo-modules, Aider repo map) and public API surface (cargo-public-api, cargo-semver-checks). The *linking* of the two with typed edges and visibility scope (Part 2 §2.2–2.3) is the defensible increment. But "comments, imports, global statements" as public interface is muddled: imports are derivable evidence, statics are just items, and doc comments are rustdoc's province. Making comments graph nodes buys the identity-churn failure mode Part 2 §5 itself warns about. |
| "Neither rust-analyzer nor any other code MCP can do this" | **False as stated** | Part 3's own external-grounding paragraph names cargo-semver-checks and RA's non-Cargo support. Serena MCP does LSP-backed symbol navigation for agents; Aider's repo map is literally a dependency graph ranked into an LLM context budget; GitNexus ships a dependency-graph MCP; CodeQL/Semgrep/Moderne do typed graphs and mass refactor verification. What nobody has is the *combination*: exact Rust edges + budgeted agent queries + current/proposed distinction + per-edge provenance. Combination claims need one killer piece; the before/intended/after conformance report is it. |

#### 3. Simulated reception

**(a) Maintainer, 300k-LOC Rust database:** "I know my architecture; that's not my bottleneck. My pain is a 40-crate workspace with sqlx proc macros and a 25-minute CI. Your extractor will choke on the macros, then go stale the moment someone edits, and *I'm* the one who debugs it. Also cold start is a solved problem — RA loads once and stays up for weeks." **Convert:** run clean on their repo with proc macros on, show incremental re-extract under 5 seconds after an edit, and show one `impact()` answer that is right where grep is provably wrong. Nothing else converts this person.

**(b) rust-analyzer contributor:** "This is call-hierarchy plus find-references behind an MCP wrapper, taking credit for our index. If budgeted neighborhoods and path queries are good ideas, they belong in RA or an LSX extension, not a parallel store that will drift from our semantics the first proc macro lands. And you're reshipping our data — what's the attribution story?" **Convert:** contribute the query shapes upstream, or demonstrate queries RA structurally can't express in the one-cursor LSP model: cross-snapshot graph diff, token-budgeted ego networks, proposal conformance. Those genuinely don't fit LSP.

**(c) Agent-harness builder:** "I ship twelve tools; my usage logs show agents call the graph one rarely and misuse it when they do. Every new server is latency, auth, another failure mode in my traces. Serena already gives me symbols over LSP. Why is yours the one that changes task outcomes?" **Convert:** a drop-in MCP server, sub-100ms queries, no build step, and an eval on *their* harness where agents with your tool beat agents with grep at equal token budget. Adoption is measured in my logs, not your benchmark.

**(d) Hacker News top comment:** "So: an MCP server wrapping rust-analyzer, plus a 500MB model that guesses edges when the compiler can't. The first half is a weekend project; the second half is wrong answers with confidence scores attached. 'Works on broken code' means 'sometimes lies,' and the whole thing dies the moment RA ships an agent mode." **Convert:** a blind, reproducible demo on a repo that doesn't build, with the degraded-provenance banner visible and honest, and the before/intended/after report catching a regression that diff review missed. Skeptics are converted by an artifact they can run, not a thesis.

**(e) Rust newcomer using Cursor:** "I just @-mention files and ask; Cursor indexes everything already and never made me learn what a dependency graph is. I tried your tool on my half-written project and it said 40% of edges were 'syntactic' — I don't know what to do with that, and I don't care. I want to know where to start, not what an ego network is." **Convert:** one command — `where do I even start` — that prints a ten-item guided tour of an unfamiliar crate with what to read first, working on their broken WIP code with zero config. If the first run isn't magical, this user is gone in 90 seconds.

**(f) CTO, 40-person Rust team:** "Another component in the toolchain means another review, another vendor, another thing that breaks during an audit. My seniors don't want agents designing architecture, my juniors shouldn't trust a graph with coverage gaps, and you want an MCP server reading my proprietary source — where does it phone home? Show me the failure mode where this *reduces* risk." **Convert:** on-prem, single static binary, zero egress, and one pilot where the conformance report catches an agent-introduced architectural regression in CI that a human reviewer missed. Priced against rework avoided, not seats.

**(g) ML researcher:** "A sub-500MB model doing held-out exact-match on interface graphs at 78–95%? That's a distillation ceiling claim with no repo-disjoint splits, pretraining contamination unaddressed, item windows that leak resolution context — Part 2 §6.1 admits this — and your own learned-binding experiment accepted *zero* bindings where a rule got 56. The scaling-ladder idea is sound; the rest is a hypothesis wearing a product's clothes." **Convert:** publish the benchmark — repo-disjoint, int4-scored, abstention-rate reported — and show one rung where the model beats the rule baseline at a fixed wrong-edge budget on fresh crates. That's a paper and a product component at once.

**(h) Security engineer:** "My fear is false assurance. 'No dependents found' from a graph that skipped two crates under a non-default cfg is worse than no tool — someone will delete code based on it. And to resolve proc macros you run build scripts, which is arbitrary code execution on untrusted repos inside a tool agents drive autonomously." **Convert:** the coverage envelope printed on every answer (scoped exclusions, failed extractions, cfg assumptions), a cfg-aware `unsafe_surface` query validated against a real audit, and a documented no-build mode with explicitly degraded provenance. Show me the failure states or I assume they're hidden.

#### 4. The demo that would make people believe

**"One map, three tenses" — 90 seconds, screen-recorded, on a real 60k-LOC multi-crate workspace the tool has never seen.**

- **0:00–0:15** Fresh clone. `parsel index` finishes; provenance banner: 61k items, 99.4% edges `resolver`, 0.6% `syntactic` (two crates, build-script failure), cfg = default. *The honesty layer, on screen, immediately.*
- **0:15–0:35** An agent (real harness, visible tool calls) gets: "support a second WAL page size without duplicating the recovery loop." It runs `find "wal recovery"` → four 80-token headlines, picks one; `impact(WalRecovery::replay)` → callers, impls, two call sites under `cfg(feature = "v2")`, each with file:line. Every id real.
- **0:35–0:55** *Before editing*: the agent emits a graph patch — introduce `PageSize` trait, list implementors, dependency flips. Split-screen shows current map → intended map. Maintainer approves with one keystroke. *This is the bidirectionality claim, made concrete.*
- **0:55–1:15** Agent implements. `parsel verify`: three-way report — 7 achieved, 1 missing (a constructor still takes `u16`), 1 unexpected (a new edge into `checkpoint.rs`, outside proposal scope) — each clickable. *The thing no other tool can say.*
- **1:15–1:30** `git checkout broken-branch` (doesn't compile). Same `impact` query; banner degrades to `substrate: tree-sitter+model, 82% edges confidence < 0.9` — and still points at the right files. *Broken code, honestly labeled.*

**What must be technically true:** cold index of 60k LOC in seconds; incremental re-extract in seconds after edits; stable identity across the rename the edit causes (Part 2 §5 identity problem — unsolved in the docs); correct resolution under cfg and proc macros; a minimal proposal schema a real harness emits unaided; and the model fallback actually at ~80%+ on unseen code — currently *unproven* (Part 2 §6.2). Items one, two, and six are the risk; the demo dies first on re-extract latency and identity churn.

#### 5. Where it dies

**1. Navigation alone isn't differentiated.** Frontier models with grep/read keep improving; the map saves tokens but doesn't change task outcomes, and Serena/Aider/Cursor own the "structured context" reflex. **Earliest cheap signal:** Part 3's own three-arm pilot (baseline agent vs +CODEMAP vs +model) on three unseen repos — if the baseline arm matches the tool arm on task correctness, the navigator is a feature, not a product. One week, zero training.

**2. The tiny model never earns its seat.** Evidence is already 0-vs-56 against a rule. If the model stays dead weight, the "rust-analyzer in the weights" story collapses and what's left is an RA-backed MCP server that upstream or Serena can commoditize in a release. **Earliest cheap signal:** scaling-ladder rung 2 (10M tokens, ~2 hours of Mac GPU, Part 4 §4): if the held-out curve is flat, or below the rule baseline at a fixed wrong-edge budget, stop training and keep rules.

**3. Nobody authors the future map.** Timeline B requires maintainers to write structural intent *before* code — a ceremony most teams have rejected in every prior form (design docs, ArchUnit tests, UML). If they won't, the product shrinks to navigator/reviewer and the "current + future" thesis is dead; you have a utility, not a company. **Earliest cheap signal:** Part 3's own inflection point — do two real maintainers, unprompted, create a *second* graph-patch proposal on actual disputed work? That's observable within one design cycle.

#### 6. What to cut from the five Parts

- **Part 4 entirely** from the product corpus. FLOP arithmetic and ELI5 lever diagrams are lab-notebook material; publishing them inside the thesis makes the model look more load-bearing than its evidence supports.
- **Part 1 §3's GPU tables** (estimated tok/s and RAM). Superseded and self-caveated by Part 2 §6.2; every number in them is a liability in front of personas (b) and (g). Keep §8's Shape/Reach/Honesty ordering; delete the tables.
- **The 100 MB memory gate** (Part 3 constraints). A research constraint masquerading as a product requirement, and the main force dragging the tiny model into the story. Keep as an engineering note; cut as a gate.
- **Timeline D** (SDK/component). Part 3 itself calls it highest-variance; defer until an external integrator asks twice.
- **Comments/imports/globals as interface nodes.** Imports are evidence fields, not entities; statics are ordinary items; doc-comment nodes invite the identity-churn failure Part 2 §5 documents. Defer all three to a v2 that has users.
- **The seven-event protocol as product spec.** K=4 anchors, 3,000-token clusters, and seven events are implementation detail (and `1793879701` §8 already recommends against the 12k-token upfront spend). Ship three tools — `find`, `impact`, `verify`.
- **Subsystem clustering / architectural ontology** (Part 2 §4.2). Already hedged ("need not add a third level"); make the deferral explicit and permanent until asked for.
- **CI gating ambitions in Timeline C.** Advisory report only; merge-blocking is the noisy-bot death Part 3 names.
- **The collected file itself.** Five parts, three of which restate the others, is the symptom: one canonical one-page thesis plus a changelog. Part 5's §11 is the sixth retelling of the same idea — the idea does not need translating six times before it has a user.

### 7. Where Devin and GLM-5.3 agree, disagree, and what changes because of it

Legend: **AGREE** = both reads say it · **ADOPT** = GLM was right, Devin's §1–§5 above should be read with this correction · **HOLD** = Devin keeps its position and says why · **CHECK** = a GLM claim neither of us has verified.

```text
 topic                                   Devin §1–§5                      GLM-5.3 §1–§6                        verdict
 ──────────────────────────────────────  ───────────────────────────────  ───────────────────────────────────  ────────
 the one big thing                       map as shared checkable object   map as working interface: read to    AGREE — same sentence,
                                         for human / LLM / compiler        navigate, patch to plan, diff to      two wordings. "Three
                                         in three time-states              verify, provenance on every edge     tenses" is the keeper.
 "neither RA nor any MCP can do this"    not examined                     false as stated; true only as the    ADOPT — say "the
                                                                            COMBINATION; killer piece is the     combination"; never the
                                                                            before/intended/after report         absolute.
 bidirectional                           the proposal round-trip IS       LLM-asks/tool-answers is every MCP;   AGREE — only the
                                         the product                       only the proposal loop is new         proposal loop is new.
 "works for broken code" = the hinge     yes: the refactor is when the    oversold: tree-sitter indexers         HOLD, narrowed. Syntactic
                                         code is broken; map must         already "work" on broken code; RA     indexing is not the claim;
                                         survive that moment               loads non-Cargo projects; model       the claim is the INTENDED
                                                                            fallback has negative evidence        map + provenance degradation
                                                                            (0 vs 56 bindings; 10/19 edges)       on the SAME diffable map.
                                                                                                                  ADOPT: the model is not
                                                                                                                  part of the hinge; evidence
                                                                                                                  against it stands.
 comments / imports / globals in level 2 listed as part of the public    category error: imports are evidence  ADOPT — they are FIELDS of the
                                         surface                           fields, statics are items, doc        item/module record (doc, uses,
                                                                            comments as nodes invite identity     cfg, globals), not new node
                                                                            churn                                 kinds. The founder's request
                                                                                                                  is met by the record, not by
                                                                                                                  the node list.
 the tiny model                          enters at 0:50 of the demo to    "hypothesis wearing a product's        AGREE — demo and v1 need no
                                         keep L3 alive while broken        clothes"; rung-2 flat curve → stop     model; H-46 stays a ladder.
 death mode #3: nobody authors intent    missed (assumed the agent drafts GLM's strongest point: every prior      ADOPT — biggest risk in the
                                         it from PR text)                   intent ceremony (UML, ArchUnit,       corpus. Mitigation stays:
                                                                            design docs) was rejected             agent drafts, human only
                                                                                                                  approves; signal = do two
                                                                                                                  maintainers make a 2nd
                                                                                                                  proposal unprompted.
 ship surface                            seven-event protocol             three tools: find · impact · verify    ADOPT for v1; the protocol is
                                                                                                                  the internal design of `find`.
 cut Part 4, cut the collected file      —                                lab notebook; "sixth retelling"        HOLD on deletion (append-only
                                                                                                                  rule; history is evidence).
                                                                                                                  ADOPT the remedy: one canonical
                                                                                                                  page (thesis §13 is the draft)
                                                                                                                  + this file as changelog.
 100 MB RSS gate                         not discussed                    research constraint posing as a        ADOPT — gate for the fallback
                                                                            product gate                          rung only, not the product.
 demo order                              broken state mid-refactor        provenance banner first, broken        MERGE — banner at 0:00, break
                                         (the real scenario)               branch last                            mid-refactor, report, then
                                                                                                                  the degraded banner on screen.
 newcomer persona                        "the map alone"                  one command: `where do I even start`  ADOPT — that is Timeline A's
                                                                            → 10-item guided tour, zero config    first command.
 Serena / GitNexus / Moderne precedents  not examined                     named as existing                      CHECK — Serena (LSP-over-MCP)
                                                                                                                  and Aider repo map are known;
                                                                                                                  GitNexus and the Moderne
                                                                                                                  recipe-diff comparison are
                                                                                                                  unverified here.
```

What survives both reads, in one paragraph: the product is **one map in three tenses — as-is, intended, as-built — that an agent reads to navigate (`find`, `impact`), writes to plan (a graph patch a human approves), and the CLI diffs to verify (`verify` → achieved / missing / unexpected / unresolved), with provenance on every edge so the report stays honest while the code is mid-refactor and broken.** Comments, imports, cfg and globals ride on the item records as fields. The tiny model is a later rung for keeping resolved edges alive when the resolver cannot; it is not in the demo and not in v1. The absolute "nobody can do this" becomes "nobody ships the combination; the conformance report is the piece that does not exist anywhere."

### 8. The three cheapest signals before writing more documents

```text
 1. three-arm pilot (Part 3): baseline agent · +map (no model) · +map+model, 3 unseen repos, same token budget
       → if arm 1 ≈ arm 2 on correctness, navigation is a feature, not a product.     ~1 week, zero training
 2. intent authoring: on real disputed work, do two maintainers produce a SECOND graph-patch proposal unprompted?
       → if no, Timeline B collapses to navigator + reviewer.                            one design cycle
 3. H-46 ladder rung 2 (10 M tokens, ~2 h Mac or 1.5 min H100): held-out curve vs the rule baseline (56 bindings)
       → flat or below the rule → keep rules, stop training.                             one afternoon
```

Neither read changes the storage rule from Part 2 and the MISP note: exact facts, shared vocabulary and opinions (including everything in §4, §6 and §7 of this document) stay in separate layers with their creator named. This section's verdicts are Devin's opinion about GLM-5.3's opinion; the measured facts are the ones with numbers attached in Parts 1–2.

### 9. ELI5 ASCII of Proposal 01, and how the Sikos cybersecurity-knowledge-graph paper fits

*Added 2026-11-06 after the founder asked for a plain-language picture of this document and for the link to Sikos, L. F., "Cybersecurity knowledge graphs", Knowledge and Information Systems (2023), https://link.springer.com/article/10.1007/s10115-023-01860-3 — the paper behind ledger entries H-47…H-59.*

```text
DEEP-INSIGHT 04 IN ONE PICTURE

   a building has three drawings.  software has only the building.

        BLUEPRINT              AS-BUILT DRAWING             INSPECTOR
        what we agreed         what actually stands         diffs the two, writes the report
             │                        │                          │
             ▼                        ▼                          ▼
        INTENDED map             AS-IS map                   verify
        (LLM drafts a            (compiler / tree-sitter     achieved / missing /
         graph patch,             extracts it; every edge     unexpected / unresolved
         human approves)          says WHO found it)

   the one big thing:  ONE map, THREE tenses, SAME ids.
        agent READS it   →  find "where do retries get decided"   ·   impact(RetryPolicy)
        agent WRITES it  →  "add trait RetryPolicy; Client → trait instead of → FixedBackoff"
        CLI   DIFFS  it  →  after the edit: 2 of 3 done, 1 missing, 1 edge you didn't ask for

   WHY "BROKEN CODE" MATTERS
        the moment you most need the map is halfway through a refactor — and that is exactly when
        the code doesn't compile and rust-analyzer's answers degrade.
        file graph + public skeleton (mod / use / pub fn / pub struct / static / cfg / doc comment)
        need NO type-checker → they stay up.  only "resolved" edges fade — and the map SAYS they faded:

             ┌──────────────────────────────────────────────┐
             │ 1,204 files · 9,318 pub items                │
             │ 97 % exact  ·  3 % unresolved (proc-macro)   │   ← this banner is the product's honesty
             │ while broken:  9 more edges unresolved       │
             └──────────────────────────────────────────────┘

   TWO LEVELS = what the agent sees first
        level 1   file ↔ file            (mod, use, include!, cfg gates)
        level 2   each file's PUBLIC face (signatures, re-exports, globals, cfg, doc comment)
                  ← comments / imports / globals are FIELDS on each item's record, not new boxes on the graph
        level 3+  bodies, calls, flow     (there, hidden, opens on request)

   WHO BELIEVES IT, AND WHEN
        maintainer     "stale index + proc macros"       → re-extract in seconds, macros on
        RA contributor "that's our index in a wrapper"   → queries LSP can't do: diff two snapshots, verify a plan
        HN             "'works on broken code' = lies"   → the banner above, on a repo that doesn't build
        CTO/security   "false 'no dependents'"           → coverage envelope on every answer, zero egress
        newcomer       "where do I even start"           → 10-item tour, zero config, or gone in 90 s

   HOW IT DIES         nobody writes the INTENDED map (UML, ArchUnit, design docs all died this way)
   HOW WE HEDGE        the agent drafts it, the human only clicks approve
   CHEAP SIGNAL        do two maintainers make a SECOND proposal, unprompted?


HOW THE SIKOS PAPER (KAIS 2023, "Cybersecurity knowledge graphs") FITS

   the paper is NOT about code.  it is about a field that already solved OUR storage problem:
   many sources, different trust, facts mixed with guesses, shared across teams, never trusted blindly.

        cyber-threat world                         our Rust map
        ───────────────────────────────────        ─────────────────────────────────────────
        IOC observed by a sensor  (hard fact)      edge found by the compiler       (exact)
        analyst's attribution     (opinion)        LLM's "this is the retry loop"   (L6 meaning)
        STIX / MISP taxonomy      (shared words)   our node/edge kinds, tags        (vocabulary)
        confidence + provenance on every object    exact | projected | proposed | unresolved + creator
        sighting: "I saw this too / this is wrong" agent or human confirms / denies an edge
        warninglist: known false positives         "this edge is a macro artefact, ignore"
        first-seen / last-seen                     when did this edge appear / disappear across commits
        feeds between orgs, each with trust level  map shared across devices / agents / CI, provenance kept

   the lesson we took (H-47…H-59):   FACT  ≠  VOCABULARY  ≠  OPINION  — three layers, never written into
   each other, any query can peel one off.   that rule is what lets the INTENDED map (an opinion about the
   future) and the LLM's annotations sit next to compiler facts without poisoning them — which is exactly
   the objection every persona above raises ("will it lie to me?").

   so:   DI-04 says WHAT the object is (one map, three tenses).
         Sikos/MISP says HOW to store it so humans, LLMs and compilers can all write into it and still trust it.
```

### 10. Proposal 01 under full openness: open-source tool, open weights, open training data

*Added 2026-11-06 after the founder set the context: Parseltongue is a fully open-source tool, the fallback model ships with open weights, and the training data is published. This section re-derives §1–§9 under that assumption. It appends; it does not replace. Items marked CHECK are not verified.*

#### 10.1 What openness changes in the one big thing

The sentence does not change. What changes is **who can write into the map and who can audit the writer**:

```text
   CLOSED (implicit in §1–§9)                     OPEN (this section)
   ─────────────────────────────────────────      ──────────────────────────────────────────────────────
   one vendor extracts, stores, diffs             anyone can read the extractor; every "exact" edge is
                                                  reproducible by running the same binary on the same commit
   provenance = "trust our label"                 provenance = "run it yourself" — the honesty layer becomes
                                                  checkable, not asserted
   the fallback model is a black box              open weights + open data + open eval → the "does it lie?"
                                                  question has a public answer per release
   the map of std/crates.io is private to the     the map of every crate version can be a PUBLIC FEED,
   tool                                           built once, downloaded by everyone (MISP feeds, H-59)
```

The last line is the genuinely new idea that openness unlocks. Reach (Part 1 §8) assumed the model must *memorise* std and the top crates' signatures because the resolver cannot run. Under openness the facts are **downloaded, not memorised**: a public, versioned, compiler-exact interface graph per crate@version — think docs.rs, but the graph instead of the HTML. The model then needs only the *algorithm* (binding, chaining, abstention), which is what §Q1 of the 100 MB discussion said is the small, defensible part. Reach shrinks from "model knows the ecosystem" to "model + feed".

#### 10.2 The asset moves from the binary to the corpus

```text
   what is easy to copy            an MCP server over rust-analyzer   (GLM's HN persona: "a weekend project")
   what is hard to copy            the first open corpus of compiler-exact Rust interface graphs,
                                   repo-disjoint splits, post-cutoff held-outs, abstention labels, int4 scores,
                                   and a public leaderboard that anyone can submit a model to
   what compounds                  every published crate graph in the feed; every sighting/warninglist entry
                                   a user contributes (MISP H-51/H-53); every held-out repo added after a
                                   model's cutoff
```

This flips GLM's death mode #2 ("the tiny model never earns its seat"): in the open setting the model is **one entry on a leaderboard the project owns**, not the product. If the rule baseline (56 bindings) wins, the leaderboard says so and the project is still the benchmark everyone cites. The dataset is the paper the ML persona asked for, published before any model claim.

#### 10.3 Reception, re-run under openness

```text
 persona                 §4/§6 objection                        what openness does to it
 ──────────────────────  ─────────────────────────────────────  ─────────────────────────────────────────────────────
 rust-analyzer contrib.  "reshipping our index; attribution?"   dissolves if licence-compatible (RA is MIT/Apache-2.0) and
                                                                 the `codemap` exporter is offered upstream as a PR; the
                                                                 remaining objection is semantic drift — answered by the
                                                                 open conformance test suite pinned to an RA version
 CTO, 40-person team     "egress, vendor, audit"                 dissolves: single static binary, no network, source
                                                                 auditable; the new objection is "who maintains it in
                                                                 3 years?" (see 10.4)
 security engineer       "false assurance; build-script ACE"     stronger: the coverage envelope is code they can read;
                                                                 the no-build mode is a documented flag they can diff
 HN top comment          "weekend project + confident guesses"   the artefact is runnable; the guesses have a public
                                                                 scorecard with abstention rate; the comment becomes
                                                                 "nice benchmark" — which is the win
 ML researcher           "distillation ceiling, contamination"   converts: repo-disjoint, post-cutoff held-outs, int4
                                                                 scores all published; the dataset is citable
 maintainer, 300k LOC    "will choke on my macros, go stale"     unchanged — openness does not fix proc macros or
                                                                 re-extract latency; still converted only by a clean run
 agent-harness builder   "why is yours the one that matters?"    slightly better: no auth, no vendor; still converted
                                                                 only by their own logs (three-arm pilot)
 newcomer in Cursor      "where do I start"                      unchanged — openness is invisible to them
 NEW: crate author       (not in §4/§6)                          "my crate's graph is in a public feed — is the licence
                                                                 respected, can I correct it?" → feed carries the
                                                                 crate's licence + a sighting channel for corrections
 NEW: contributor        (not in §4/§6)                          "can I add a query / an edge kind / a language?" →
                                                                 the record schema and plugin points must be documented
                                                                 before the first release or the project gets forks
```

Net: two of the three hardest personas (RA contributor, CTO) are mostly answered by openness itself; the maintainer and harness builder are not — they are answered only by engineering.

#### 10.4 New risks that openness creates

```text
 risk                                     why it is real                                   mitigation
 ───────────────────────────────────────  ───────────────────────────────────────────────  ────────────────────────────────────────
 dataset licence poisoning                 training records derived from GPL / no-licence    licence filter at ingest (permissive
                                           repos cannot be redistributed as open data        only); per-record provenance carries
                                                                                              repo, commit, licence, extractor version
 LLM-produced labels (L6 meaning) may       Z.ai Coding Plan / API terms on redistributing    CHECK the terms before publishing any
   not be redistributable                  outputs or using them to train other models       GLM-written label; keep L6 labels in a
                                           are not verified here                              separate, separately licensed layer
 "open weights" ≠ "open model"             weights without data + eval + training recipe      publish all four; a model card that
                                           is the industry's usual half-measure               lists held-out repos by name
 upstream absorption                       rust-analyzer could add `codemap` export +         this is a WIN for the mission and a
                                           agent queries; the tool becomes a thin layer       loss only for a business; the asset
                                                                                              (corpus, feed, benchmark) survives it
 maintainership                            tracking RA/rustc churn is a permanent cost;        pin RA version per release; conformance
                                           an abandoned extractor silently produces stale      suite in CI; the feed's `last_seen`
                                           "exact" edges — the worst failure in Part 2 §5      shows staleness instead of hiding it
 public feed trust                         a poisoned or stale crate graph in the feed is       feeds are signed per extractor version;
                                           false completeness at ecosystem scale               MISP-style distribution levels and
                                                                                              warninglists (H-53/H-59)
 no revenue                                a community project does not fund the three-arm    the pilot and rung-2 ladder cost < $50
                                           pilot or the maintainer                             of compute; the maintainer is the cost
```

#### 10.5 What is explicitly open in each layer

```text
   layer                                  licence / artefact                                         why this layer
   ─────────────────────────────────────  ─────────────────────────────────────────────────────────  ───────────────────────────
   CLI + extractor + query engine         Apache-2.0 OR MIT (matches rustc / RA)                      drop-in for the ecosystem
   record schema (Part 2 §2.1) + ids      versioned spec file in the repo; semver'd                   forks and plugins need it
   exact graphs of crates.io versions     public feed, per-crate licence passed through                the Reach fix (10.1)
   training corpus (RA-labelled)          permissive-only sources; CC-BY-4.0 or ODC-BY on the labels   the asset (10.2)
   L6 meaning labels (LLM-written)        separate layer; licence depends on CHECK in 10.4             opinion ≠ fact (Sikos rule)
   fallback model weights                 open weights + model card + eval harness + recipe           "open model", not half
   benchmark + leaderboard                repo-disjoint, post-cutoff held-outs, int4 scores            the thing that outlives us
```

#### 10.6 The demo under openness (delta to §5 / GLM §4)

Two frames change:

```text
  0:00  pt map .   → banner now also prints:  extractor v0.3 (rust-analyzer 2026-10-27) · std + 38 crate graphs from feed
  1:25  pt report  → the PR description ends with a reproducibility line:
                     "re-run: pt verify --at <sha> --extractor v0.3 → identical report"  (anyone can check it)
```

#### 10.7 ELI5 ASCII

```text
   CLOSED:   "trust us, this edge is exact."                 OPEN:   "here is the binary, the commit, the RA version. run it."

   CLOSED:   the model must MEMORISE std + 100 crates.       OPEN:   the graphs of std + every crate are a public download;
                                                                     the model only learns HOW to connect, not WHAT exists.

   CLOSED:   the product is the tool.                        OPEN:   the product is the CORPUS + the FEED + the BENCHMARK;
                                                                     the tool is how you use them; the model is one leaderboard row.

   who stops objecting:   RA contributors (we are upstream-friendly) · CTOs (no egress, auditable) · ML researchers (citable data)
   who does not care:     the maintainer with proc macros · the newcomer · the harness builder — only engineering converts them
   new people to answer:  crate authors ("that's my crate in your feed") · contributors ("where is the schema?")

   new things to get right:   licences on every training record · Z.ai label terms (CHECK) · signed feeds · an RA-version pin
```

#### 10.8 What changes in §7's verdicts

- GLM death mode #2 (model never earns its seat) → **neutralised**: the benchmark is the deliverable; the model is optional.
- GLM "rust-analyzer ships agent mode and you die" → **reclassified** as mission success; only the corpus/feed/benchmark need to outlive the tool.
- GLM "cut the 100 MB gate" → stands; under openness the gate is a leaderboard *column*, not a product requirement.
- GLM death mode #3 (nobody authors intent) → **unchanged**; openness does not make anyone write a blueprint.
- §7 "nobody ships the combination" → add: *and nobody publishes the compiler-exact Rust graph corpus or a per-crate graph feed* — that absence is verifiable and, unlike the product claim, cheap to fill first.

---

# Part 7 — 2026-11-06 — Proposal 01 restructured top-down (Minto pyramid) (verbatim copy of Deep-Insight-05.md)

*2026-11-06. A re-ordering of Deep-Insight-04 (Proposal 01, §0–§10) by the Minto pyramid principle: answer first, then three mutually exclusive and collectively exhaustive key lines, each resting on grouped evidence. No new claims; every leaf cites the DI-04 section it comes from. DI-04 stays as written. Later revisions are appended as new sections here or as a new Part of the collected Deep-Insight-01.*

---

### Introduction (S-C-Q-A)

**Situation.** Coding agents are now doing cross-cutting changes to large Rust codebases. Their only structural instrument is grep; rust-analyzer knows the exact structure but answers one cursor at a time, and its answers degrade exactly when a refactor has broken the build. (DI-04 §1, §2)

**Complication.** Nothing holds the *intended* structure of the code, nothing diffs intended against actual, and no tool admits what it could not resolve. So agents re-derive the architecture from grep on every task, make plans no one can check, and return "no dependents" when they mean "could not see". (DI-04 §1, §2, §6 personas a/d/h)

**Question.** What is the one thing to build, and why would anyone adopt it over rust-analyzer, Serena, Aider's repo map or Cursor's index?

**Answer — the governing thought:**

> **Build the structure of a Rust codebase as one open, provenance-carrying map in three tenses — as-is, intended, as-built — that an agent reads to navigate, writes to plan, and the CLI diffs to verify; publish the compiler-exact corpus and per-crate feed first, because that is the part nobody has and the part that outlives any tool or model.**

```text
                              ┌──────────────────────────────────────────────────────┐
                              │  ONE OPEN MAP, THREE TENSES, PROVENANCE ON EVERY EDGE │
                              │  read to navigate · write to plan · diff to verify    │
                              │  corpus + feed first, tool second, model last         │
                              └──────────────────────────────────────────────────────┘
                 ┌────────────────────────────┼──────────────────────────────┐
   ┌─────────────┴─────────────┐  ┌────────────┴────────────┐   ┌─────────────┴──────────────┐
   │ 1  WHAT IT IS              │  │ 2  WHY IT IS CREDIBLE    │   │ 3  WHAT TO DO FIRST         │
   │ a shared, checkable object │  │ Rust makes it exact;     │   │ three cheap signals;        │
   │ with three tenses, two     │  │ the broken moment is the │   │ ship find · impact · verify;│
   │ visible levels, and a      │  │ real one; openness       │   │ publish corpus + feed       │
   │ storage rule that keeps    │  │ answers the hard         │   │ before any model claim;     │
   │ facts and opinions apart   │  │ personas; evidence exists│   │ and what not to do          │
   └───────────────────────────┘  └─────────────────────────┘   └────────────────────────────┘
```

---

### Key line 1 — WHAT IT IS: a shared, checkable object, not a tool

#### 1.1 One map, three tenses, same ids
- AS-IS: what the source establishes today (extracted). INTENDED: a graph patch the LLM drafts and a human approves. AS-BUILT: re-extracted after the edit. The loop `propose → build → re-extract → compare → report` is the product; one-way "LLM reads a graph" is Sourcegraph-for-agents, one-way "LLM writes a plan" is an ADR generator. (DI-04 §1)
- Analogy that holds: blueprint, as-built drawing, building inspector. Software keeps only the building. (DI-04 §1, §9)
- "Bidirectional" is earned only by the proposal loop; LLM-asks/tool-answers is every MCP server. (DI-04 §7, GLM §2)

#### 1.2 Two visible levels; deeper levels on request
- Level 1 file ↔ file (`mod`, `use`, `include!`, cfg gates) — tree-sitter, always available. Level 2 the public face of each file: signatures, re-exports, `pub static/const`, `#[cfg]`, doc comment, item-level macro names. Level 3+ (bodies, calls, flow) kept, hidden, opened on expansion. (DI-04 §3)
- Comments, imports and globals are **fields of the item/module record**, not new node kinds — imports are evidence on `USE` edges, statics are items, doc-comment nodes invite identity churn. The founder's requirement is met by the record. (DI-04 §7 ADOPT, §9)
- Membership test for level 2: *could this line change what another file is allowed to write?* (DI-04 §3)

#### 1.3 The storage rule that makes it trustworthy
- Every edge carries `exact | projected | proposed | unresolved`, plus creator, extractor version, cfg context. An unresolved result is never shown as absence. (DI-04 §2, Part 2 §5)
- Three layers never written into each other — compiler facts ≠ shared vocabulary ≠ opinions (LLM meaning, proposals, sightings) — taken from Sikos's cybersecurity knowledge graphs and MISP: provenance and confidence per object, sightings, warninglists, first/last-seen, feeds with trust levels (H-47…H-59). This is what lets an opinion about the future (the intended map) sit next to compiler facts without poisoning them. (DI-04 §9)

#### 1.4 Under full openness the object has four parts, and the tool is the smallest
- Apache-2.0/MIT CLI + extractor; versioned record schema; **public feed of compiler-exact graphs per crate@version** (docs.rs, but the graph); **open training corpus** with repo-disjoint, post-cutoff held-outs; open weights with card + eval + recipe; a leaderboard. (DI-04 §10.1, §10.5)
- The feed shrinks "Reach": the model no longer memorises std and 100 crates — it downloads the facts and learns only how to connect them. (DI-04 §10.1)
- The model is one leaderboard row, not the product. (DI-04 §10.2)

---

### Key line 2 — WHY IT IS CREDIBLE: Rust, the broken moment, openness, and evidence

#### 2.1 Rust is the one mainstream language where the map is a fact
- Static names, `pub` in source, one impl per trait+type, no reflection → rust-analyzer/rustdoc produce an exact public-interface graph; any approximation can be scored against truth. (thesis §13; DI-04 §1)
- Measured: tree-sitter 0.07 s and rust-analyzer 29 s / 2.3 GB on 42k LOC, ~0 unknown expressions with proc macros on; warm `cargo check` 0.69 s. (Part 1–2; DI-04 §2)

#### 2.2 "Works for broken code" is the hinge because the refactor is when the code is broken
- Halfway through extracting a trait, 40 files fail to type-check and RA's types go `{unknown}` across the blast radius; the intended map is the only stable thing in the room. (DI-04 §2)
- Level 1 and the public skeleton need no type-checker, so they stay up; only resolution degrades, and the banner says by how much. (DI-04 §2, §9)
- Narrowed after GLM: syntactic indexing on broken files is not the claim (Aider, Cursor, Serena do that); the claim is the intended map plus provenance degradation on the *same diffable map*. The model is not part of the hinge; evidence against it (0 vs 56 bindings; 10/19 edges) stands. (DI-04 §7 HOLD/ADOPT)

#### 2.3 The absolute claim is false; the combination claim is true and has one killer piece
- Each piece exists somewhere: cargo-modules (file graph), cargo-public-api / cargo-semver-checks (public surface), Serena (LSP over MCP), Aider repo map (ranked graph in a token budget), RA non-Cargo projects. (GLM §2; DI-04 §7)
- Nobody ships exact Rust edges + budgeted agent queries + current/proposed distinction + per-edge provenance. The before/intended/after conformance report exists nowhere. Under openness, add: nobody publishes the compiler-exact Rust graph corpus or a per-crate graph feed. (DI-04 §7, §10.8)

#### 2.4 Openness answers the hardest personas; engineering answers the rest
- Dissolved by openness: RA contributor (licence-compatible, `codemap` export offered upstream), CTO (single static binary, zero egress, auditable), ML researcher (citable repo-disjoint data), HN ("weekend project" → "nice benchmark"). (DI-04 §10.3)
- Not touched by openness: the 300k-LOC maintainer (proc macros, re-extract latency), the harness builder (their own logs), the newcomer (`where do I even start`). Only a clean run, sub-second incremental re-extract and a three-arm A/B convert them. (DI-04 §4, §6, §10.3)
- New personas created by openness: crate authors (licence passthrough, correction channel) and contributors (documented schema and plugin points before v1). (DI-04 §10.3)

#### 2.5 The honest negatives are already in the record
- Frontier models omitted 6 of 14 signature dependencies; source-only agents found 10/19 local edges; the learned-binding experiment accepted 0 bindings where a rule accepted 56; 65 % of calls are method calls whose receivers need other items' return types. (Parts 1–3; DI-04 §6 persona g)
- Therefore: no model in the demo, no model in v1, H-46 stays a ladder with a stop rule. (DI-04 §5, §7)

---

### Key line 3 — WHAT TO DO FIRST: three signals, three tools, corpus before model

#### 3.1 Run the three cheapest signals before writing more documents
```text
 1. three-arm pilot: baseline agent · +map (no model) · +map+model, 3 unseen repos, equal token budget
       arm 1 ≈ arm 2 on correctness → navigation is a feature, not a product         ~1 week, $0 training
 2. intent authoring: on real disputed work, do two maintainers produce a SECOND graph patch unprompted?
       no → Timeline B collapses to navigator + reviewer                              one design cycle
 3. H-46 ladder rung 2 (10 M tokens; ~2 h Mac or 1.5 min H100): held-out curve vs the 56-binding rule
       flat or below → keep rules, stop training                                      one afternoon
```
(DI-04 §8; GLM §5)

#### 3.2 Ship three tools, not a protocol
- `find` (the seven-event retrieval is its internals), `impact`, `verify` (achieved / missing / unexpected / unresolved). Advisory report only; no merge gating until seeded-mismatch recall is measured. (DI-04 §7; GLM §6)
- The 90-second demo: banner with provenance at 0:00; `find` → `propose` → agent edits → build breaks → `verify` still answers with the unresolved count → fix → `verify` green → `report` with a reproducibility line (`pt verify --at <sha> --extractor v0.3`). (DI-04 §5, §10.6; GLM §4)
- Technical prerequisites, none of which need the model: L1+L2 extraction on non-compiling files; stable item identity across rename/move (Part 2 §5, unsolved); a graph-patch format the LLM emits and the CLI diffs; incremental re-extract fast enough to run on every save. (DI-04 §5)

#### 3.3 Publish the corpus and the feed before any model claim
- They are the cheapest verifiable "nobody has this"; they are the paper the ML persona asked for; they neutralise GLM death mode #2 (the model is optional once the benchmark is the deliverable) and reclassify "RA ships agent mode" as mission success. (DI-04 §10.2, §10.8)
- Requirements: permissive-only sources with per-record repo/commit/licence/extractor-version; signed feeds with MISP-style distribution levels and warninglists; RA version pinned per release with a conformance suite in CI; `last_seen` so staleness is shown, never hidden. (DI-04 §10.4, §10.5)
- CHECK before publishing: Z.ai terms on redistributing GLM-written L6 labels; keep that layer separately licensed until verified. (DI-04 §10.4)

#### 3.4 What not to do
- Do not lead with the model or the 100 MB gate (a leaderboard column, not a product requirement). (DI-04 §7, §10.8)
- Do not make comments/imports/globals graph nodes. (DI-04 §7)
- Do not claim "neither rust-analyzer nor any MCP can do this"; say "nobody ships the combination". (DI-04 §7)
- Do not delete history to look tidy: one canonical page (thesis §13 / this document) plus the collected DI-01 as changelog. (DI-04 §7 HOLD)
- Do not treat anything in GLM's or Devin's reception tables as market evidence; they are attributed opinions in the opinion layer. (DI-04 §6 preamble, §8)

---

### Residual risk that no key line removes
**Nobody authors the intended map.** UML, ArchUnit tests and design docs were all rejected as ceremony; openness does not change that. The only mitigation is that the agent drafts the patch from the PR text and the human merely approves; signal 3.1-2 is the test. If it fails, the product is a navigator plus reviewer — a utility, not a company — and the corpus/feed/benchmark are still worth having. (DI-04 §7, §10.8; GLM §5)

---

### Appendix A — where each DI-04 section landed in the pyramid

```text
 DI-04 §0 founder's statement        → Introduction (Situation / Question)
 DI-04 §1 one big thing              → Governing thought; 1.1
 DI-04 §2 broken code is the hinge   → 2.2
 DI-04 §3 two levels                 → 1.2
 DI-04 §4 Devin's reception          → 2.4 (grouped by argument, not by persona)
 DI-04 §5 90-second demo             → 3.2
 DI-04 §6 GLM-5.3 verbatim           → 2.3, 2.4, 2.5, 3.1 (as attributed opinion); full text stays in DI-04
 DI-04 §7 agree / disagree table     → 1.2, 2.2, 2.3, 3.4
 DI-04 §8 three cheapest signals     → 3.1
 DI-04 §9 ELI5 + Sikos               → 1.1, 1.3
 DI-04 §10 full openness             → 1.4, 2.4, 3.3, Residual risk
```

### Appendix B — the pyramid as a reading order for different readers

```text
 30 seconds   Introduction → governing thought
 3 minutes    + the three key-line headings and 3.1
 15 minutes   + every numbered leaf
 the evidence  DI-04 (verbatim GLM review, full tables) → collected DI-01 Parts 1–5 → thesis §13 → ledger H-1…H-59
```

---

# Part 7b — 2026-11-06 — The thesis (revised Deep-Insight-05.md, references and attributions removed; verbatim)

*2026-11-06. Proposal 01 stated top-down: the answer first, then the three reasons it is the answer, then the evidence under each. Nothing here is hedged by attribution; what is measured is marked measured, what is a bet is marked bet.*

---

### The answer

**A Rust codebase's structure — what connects to what — should exist as one open map in three tenses: what the code is, what we intend it to be, and what it became after the change. The agent reads the map to navigate, writes to it to plan, and the tool diffs it to verify. Every edge says how it was found and whether it was resolved, so the map stays usable and honest while the code is mid-refactor and does not compile. The compiler-exact corpus and the per-crate graph feed are published first, because they are the part no one has and the part that outlives any tool or model.**

```text
                 ┌────────────────────────────────────────────────────────────────┐
                 │   ONE OPEN MAP · THREE TENSES · PROVENANCE ON EVERY EDGE        │
                 │   agent reads it to navigate · writes it to plan · tool diffs it │
                 │   corpus and feed first · tool second · model last               │
                 └────────────────────────────────────────────────────────────────┘
                   ┌───────────────────────┼────────────────────────────┐
     ┌─────────────┴────────────┐  ┌────────┴──────────┐   ┌─────────────┴────────────┐
     │ 1  the object is new     │  │ 2  Rust makes it   │   │ 3  the order of work is  │
     │    and nothing else is   │  │    exact, and the  │   │    corpus → three tools  │
     │    the product           │  │    broken moment   │   │    → model, gated by     │
     │                          │  │    is the real one │   │    three cheap signals   │
     └──────────────────────────┘  └───────────────────┘   └──────────────────────────┘
```

---

### 1. The object is new, and nothing else is the product

#### 1.1 Three tenses on one vocabulary
- **As-is**: the graph the source establishes today. **Intended**: a graph patch — add this trait, flip these dependencies — drafted by the agent, approved by a human. **As-built**: the graph re-extracted after the edit.
- The product is the loop: *propose → build → re-extract → compare → report*. The report has four columns: achieved, missing, unexpected, unresolved.
- Every discipline that builds physical things keeps a blueprint, an as-built drawing, and an inspector who diffs them. Software keeps only the building. Agents inherit that poverty and re-derive the architecture from grep on every task.
- A tool where the LLM only reads a graph is a code search engine. A tool where the LLM only writes a plan is a design-doc generator. The round trip is the only thing that is not already shipping.

#### 1.2 Two levels are visible; the rest opens on demand
- **Level 1** — file to file: module declarations, imports, include paths, cfg gates. Readable by a syntax parser; never unavailable.
- **Level 2** — the public face of each file: every public signature, re-export, static and constant, cfg and feature gate, the doc comment, item-level macro names. The test for membership: *could this line change what another file is allowed to write?*
- Comments, imports and globals are fields on the item record, not nodes. Imports are evidence on dependency edges; statics are items; a doc comment is the contract's prose attached to the item it documents.
- Bodies, calls, data flow, lock order, Send/Sync boundaries, and the semantic layer ("this is the retry loop") are kept and shown on expansion. They are not the default view.

#### 1.3 Honesty is a storage rule, not a promise
- Every edge carries one of `exact · projected · proposed · unresolved`, the creator, the extractor version and the build context. An unresolved edge is never displayed as absence.
- Three layers are never written into each other: compiler facts, shared vocabulary (node and edge kinds, tags), and opinions (the intended map, semantic annotations, model proposals, human or agent sightings). Any query can peel a layer off.
- This is the same structure the cyber-threat-intelligence field settled on for knowledge graphs fed by many sources of unequal trust: provenance and confidence on every object, sightings to confirm or deny, warninglists for known false positives, first-seen and last-seen, feeds with distribution levels. It is what lets an opinion about the future sit next to a compiler fact without poisoning it.

#### 1.4 Open means the asset is the corpus, not the binary
- Four published artefacts: the Apache/MIT tool and its versioned record schema; **a public feed of compiler-exact graphs for every crate version** — docs.rs, but the graph; **the training corpus** with repository-disjoint splits and held-out repositories newer than any base model's cutoff; and open weights with the card, the eval harness and the recipe.
- The feed changes what a fallback model must know. It no longer memorises the standard library and a hundred crates; it downloads the facts and learns only how to connect them — binding, chaining, abstaining. That is the small, defensible part.
- The model is one row on a leaderboard the project owns. If a rule beats it, the leaderboard says so and the project is still the benchmark everyone cites.

---

### 2. Rust makes the map exact, and the broken moment is the real one

#### 2.1 Rust is the one mainstream language where the map is a fact
- Names are static, visibility is written in the source, each trait is implemented once per type, there is no reflection and no eval. The compiler's resolver produces the public-interface graph exactly, so every approximation — a large model, a small model, a heuristic — can be scored against truth. No other mainstream language allows this.
- Measured on a 42k-line, eleven-crate workspace: syntax parse 0.07 s; full rust-analyzer analysis 29 s at 2.3 GB; near-zero unknown expressions with proc macros enabled; warm incremental `cargo check` 0.69 s.
- Measured on two large public Rust codebases: 65 % of call sites are method calls, and only a quarter of their receivers can be typed from the enclosing function alone. There is no parser-only shortcut to the resolved graph. The resolver builds; nothing else can.

#### 2.2 The code is broken exactly when the map is needed most
- The tool is for cross-cutting change. Halfway through extracting a trait, forty files fail to type-check and the resolver's answers degrade across the whole blast radius. The intended map is the only stable thing in the room.
- Level 1 and the public skeleton need no type-checker, so they stay up on a file that does not compile. Only resolution degrades — and the banner says by how much: "97 % exact, 3 % unresolved; while broken, 9 more edges unresolved."
- The claim is not that a syntax parser can index broken files; several tools do that. The claim is that the intended map and the degradation of the actual map live on the same diffable object, so `verify` still answers mid-refactor. No fallback model is required for this; the model is a later rung for keeping resolved edges alive when the resolver cannot.

#### 2.3 The absolute claim is false; the combination claim is true and has one killer piece
- Each piece exists somewhere: module graphs, public-API diffs, LSP-over-MCP symbol navigation, ranked repo maps within a token budget, resolver support for non-Cargo projects.
- No one ships exact Rust edges plus budgeted agent queries plus the current/intended distinction plus per-edge provenance. The before/intended/after conformance report exists nowhere. The compiler-exact Rust graph corpus and the per-crate graph feed exist nowhere. Those absences are verifiable and cheap to fill.

#### 2.4 Who is convinced by what
- **Convinced by openness alone**: the rust-analyzer contributor (licence-compatible, exporter offered upstream), the CTO (single static binary, no egress, auditable), the ML researcher (citable repository-disjoint data), the skeptic who calls it a weekend project (the artefact is runnable and the guesses have a public scorecard).
- **Convinced only by engineering**: the maintainer of a 300k-line workspace (proc macros on, re-extract in seconds, one `impact` answer that grep gets wrong), the harness builder (an A/B in their own logs), the newcomer (one command, `where do I even start`, that works on broken work-in-progress with no configuration).
- **Created by openness**: the crate author who finds their crate in the feed (licence passed through, a correction channel) and the contributor who wants to add a query or an edge kind (schema and plugin points documented before the first release).

#### 2.5 The negatives are already on the record and shape the order of work
- Frontier models asked to draw signature dependencies omitted six of fourteen; agents working from source alone found ten of nineteen local edges; a learned binding model accepted zero bindings where a rule accepted fifty-six. An omitted edge looks like independence, which is the most dangerous output a structural tool can produce.
- Therefore no model is in the demo and no model is in version one. The model is a ladder with a stop rule, run after the corpus exists.

---

### 3. The order of work: corpus, three tools, then the model — gated by three signals

#### 3.1 Three signals, each cheap, each able to kill a branch of the thesis
```text
 1. three-arm pilot on three unseen repositories, equal token budget:
      baseline agent · agent + map without model · agent + map + model
      if the first two arms tie on correctness, navigation is a feature, not a product      ~1 week, no training
 2. intent authoring on real disputed work:
      do two maintainers produce a SECOND graph patch unprompted?
      if not, the planning tense collapses and the product is a navigator plus a reviewer   one design cycle
 3. model ladder, second rung (10 M tokens, an afternoon):
      held-out curve against the rule baseline; flat or below → keep rules, stop training   one afternoon, < $1
```

#### 3.2 Three tools, not a protocol
- `find` — seven words in, four candidate entities out, each with its public anchor, paths, blast radius and a bounded neighbourhood under a token budget. The retrieval protocol is its internals, not its interface.
- `impact` — callers, implementors, constructors, cfg-gated call sites, with file and line, and the coverage envelope printed on every answer.
- `verify` — the intended patch against the as-built graph: achieved, missing, unexpected, unresolved. Advisory only; it blocks no merge until its recall on seeded mismatches is measured.

#### 3.3 The ninety-second proof
```text
  0:00  map      banner: crates, files, public items; % exact; % unresolved and why; extractor + resolver version; feed crates loaded
  0:10  find     "where do retries get decided"  →  four candidates; picks one; neighbourhood under budget; blast radius
  0:25  propose  "make retry policy pluggable behind a trait"  →  graph patch on screen; human approves
  0:40  edit     halfway, the crate does not compile
  0:50  verify   still answers: intended 3 · achieved 2 · missing 1 · unexpected 1 · 9 edges unresolved while broken
  1:10  fix      build green; verify: 3 of 3, nothing unexpected
  1:25  report   before / intended / after, with a reproducibility line anyone can re-run at that commit and extractor version
```
- Prerequisites, none of which need a model: level-1 and level-2 extraction on non-compiling files; stable item identity across rename and move; a patch format the agent emits and the tool diffs; incremental re-extraction fast enough to run on every save. Identity and re-extraction latency are where the demo dies first.

#### 3.4 Publish the corpus and the feed before any product or model claim
- Permissive-licence sources only, with repository, commit, licence and extractor version on every record. Signed feeds with distribution levels and warninglists. The resolver version pinned per release, with a conformance suite in CI. A last-seen stamp on every edge so staleness is shown, never hidden.
- Semantic labels written by a large model live in their own layer with their own licence, checked before publication.

#### 3.5 What is deliberately not done
- No model-first story. No memory gate as a product requirement — it is a leaderboard column.
- No comment, import or global as a graph node.
- No claim that nothing else can do any of this; the claim is that nobody ships the combination, and nobody publishes the corpus or the feed.
- No deletion of earlier research to look tidy: one canonical page, and the collected record as the changelog.
- No reception table treated as market evidence; the market evidence is the three signals.

---

### The one risk the thesis does not remove

**Nobody writes the intended map.** Every earlier form of architectural intent — diagrams, architecture tests, design documents — was rejected as ceremony. Openness does not change that. The only mitigation is that the agent drafts the patch from the work already described in the pull request and the human merely approves. Signal 2 is the test. If it fails, the product is a navigator and a reviewer — a utility, not a company — and the corpus, the feed and the benchmark are still worth having built.

---

# Part 7c — 2026-11-06 — The thesis, revision 3 (verify/diff/as-built removed from scope; verbatim copy of Deep-Insight-05.md)

*2026-11-06, revision 3. Proposal 01 stated top-down: the answer first, then the three reasons it is the answer, then the evidence under each. Scope change in this revision: there is no verification step, no diff, no "as-built" map and no conformance report. The future is simply the part of the one map that has not turned real yet. What is measured is marked measured; what is a bet is marked bet.*

---

### The answer

**A Rust codebase's structure — what connects to what — should exist as one open map that holds both what the code is and what we intend it to become. The agent reads the map to navigate and writes to it to plan; the compiler fills in the facts. Every edge says how it was found: `exact`, `unresolved`, or `proposed`. A proposed edge is the future; it turns exact on its own the moment the compiler finds it in the source. The map stays readable and honest while the code is mid-refactor and does not compile. The compiler-exact corpus and the per-crate graph feed are published first, because they are the part no one has and the part that outlives any tool or model.**

```text
                 ┌───────────────────────────────────────────────────────────────┐
                 │   ONE OPEN MAP · CURRENT AND FUTURE EDGES · PROVENANCE ON EACH │
                 │   agent reads it to navigate · writes it to plan · compiler    │
                 │   turns proposed edges exact · corpus and feed first           │
                 └───────────────────────────────────────────────────────────────┘
                   ┌──────────────────────┼─────────────────────────────┐
     ┌─────────────┴────────────┐  ┌───────┴───────────┐   ┌─────────────┴────────────┐
     │ 1  the object is new     │  │ 2  Rust makes it   │   │ 3  the order of work is  │
     │    and nothing else is   │  │    exact, and the  │   │    corpus → three tools  │
     │    the product           │  │    broken moment   │   │    → model, gated by     │
     │                          │  │    is the real one │   │    three cheap signals   │
     └──────────────────────────┘  └───────────────────┘   └──────────────────────────┘
```

---

### 1. The object is new, and nothing else is the product

#### 1.1 Current and future on one vocabulary
- The map has one set of nodes and edges. An edge found by the compiler is `exact`. An edge the compiler could not settle is `unresolved`. An edge the agent or a human adds because the code *should* have it is `proposed`. Same ids, same kinds, same queries.
- Planning is writing proposed edges: "a trait `RetryPolicy`; `Client` depends on it instead of on `FixedBackoff`; `FixedBackoff` implements it." Nothing else is required of the plan. No document, no checklist, no step that grades it.
- The plan completes itself. Each time the source is re-extracted, every proposed edge the compiler now finds becomes exact; the rest stay proposed and visible. "Done" is a map with no proposed edges left in that region. New exact edges nobody proposed are simply shown; the map does not judge them.
- Every discipline that builds physical things draws what it intends before it builds. Software keeps only the building. Agents inherit that poverty and re-derive the architecture from grep on every task; they also plan in prose, which cannot be looked up later. A proposed edge can.
- A tool where the LLM only reads a graph is a code search engine. A map the LLM can also write the future into is the thing not already shipping.

#### 1.2 Two levels are visible; the rest opens on demand
- **Level 1** — file to file: module declarations, imports, include paths, cfg gates. Readable by a syntax parser; never unavailable.
- **Level 2** — the public face of each file: every public signature, re-export, static and constant, cfg and feature gate, the doc comment, item-level macro names. The test for membership: *could this line change what another file is allowed to write?*
- Comments, imports and globals are fields on the item record, not nodes. Imports are evidence on dependency edges; statics are items; a doc comment is the contract's prose attached to the item it documents.
- Bodies, calls, data flow, lock order, Send/Sync boundaries, and the semantic layer ("this is the retry loop") are kept and shown on expansion. They are not the default view.

#### 1.3 Honesty is a storage rule, not a promise
- Every edge carries `exact · unresolved · proposed`, the creator, the extractor version and the build context. An unresolved edge is never displayed as absence; a proposed edge is never displayed as fact.
- Three layers are never written into each other: compiler facts, shared vocabulary (node and edge kinds, tags), and opinions (proposed edges, semantic annotations, model guesses, human or agent sightings). Any query can peel a layer off.
- This is the structure the cyber-threat-intelligence field settled on for knowledge graphs fed by many sources of unequal trust: provenance and confidence on every object, sightings to confirm or deny, warninglists for known false positives, first-seen and last-seen, feeds with distribution levels. It is what lets an opinion about the future sit next to a compiler fact without poisoning it.

#### 1.4 Open means the asset is the corpus, not the binary
- Four published artefacts: the Apache/MIT tool and its versioned record schema; **a public feed of compiler-exact graphs for every crate version** — docs.rs, but the graph; **the training corpus** with repository-disjoint splits and held-out repositories newer than any base model's cutoff; and open weights with the card, the eval harness and the recipe.
- The feed changes what a fallback model must know. It no longer memorises the standard library and a hundred crates; it downloads the facts and learns only how to connect them — binding, chaining, abstaining. That is the small, defensible part.
- The model is one row on a leaderboard the project owns. If a rule beats it, the leaderboard says so and the project is still the benchmark everyone cites.

---

### 2. Rust makes the map exact, and the broken moment is the real one

#### 2.1 Rust is the one mainstream language where the map is a fact
- Names are static, visibility is written in the source, each trait is implemented once per type, there is no reflection and no eval. The compiler's resolver produces the public-interface graph exactly, so every approximation — a large model, a small model, a heuristic — can be scored against truth. No other mainstream language allows this.
- Measured on a 42k-line, eleven-crate workspace: syntax parse 0.07 s; full rust-analyzer analysis 29 s at 2.3 GB; near-zero unknown expressions with proc macros enabled; warm incremental `cargo check` 0.69 s.
- Measured on two large public Rust codebases: 65 % of call sites are method calls, and only a quarter of their receivers can be typed from the enclosing function alone. There is no parser-only shortcut to the resolved graph. The resolver builds; nothing else can.

#### 2.2 The code is broken exactly when the map is needed most
- The tool is for cross-cutting change. Halfway through extracting a trait, forty files fail to type-check and the resolver's answers degrade across the whole blast radius. The proposed edges are the only stable thing in the room: they say where the work is going while the exact edges are in flux.
- Level 1 and the public skeleton need no type-checker, so they stay up on a file that does not compile. Only resolution degrades — and the banner says by how much: "97 % exact, 3 % unresolved; while broken, 9 more edges unresolved."
- The claim is not that a syntax parser can index broken files; several tools do that. The claim is that the current and the future edges live on the same object, so the agent can keep navigating by the plan while the compiler catches up. No fallback model is required for this; the model is a later rung for keeping resolved edges alive when the resolver cannot.

#### 2.3 The absolute claim is false; the combination claim is true and has one missing piece
- Each piece exists somewhere: module graphs, public-API listings, LSP-over-MCP symbol navigation, ranked repo maps within a token budget, resolver support for non-Cargo projects.
- No one ships exact Rust edges plus budgeted agent queries plus proposed edges on the same map plus per-edge provenance. A map the agent can write the future into exists nowhere. The compiler-exact Rust graph corpus and the per-crate graph feed exist nowhere. Those absences are verifiable and cheap to fill.

#### 2.4 Who is convinced by what
- **Convinced by openness alone**: the rust-analyzer contributor (licence-compatible, exporter offered upstream), the CTO (single static binary, no egress, auditable), the ML researcher (citable repository-disjoint data), the skeptic who calls it a weekend project (the artefact is runnable and the guesses have a public scorecard).
- **Convinced only by engineering**: the maintainer of a 300k-line workspace (proc macros on, re-extract in seconds, one `impact` answer that grep gets wrong), the harness builder (an A/B in their own logs), the newcomer (one command, `where do I even start`, that works on broken work-in-progress with no configuration).
- **Created by openness**: the crate author who finds their crate in the feed (licence passed through, a correction channel) and the contributor who wants to add a query or an edge kind (schema and plugin points documented before the first release).

#### 2.5 The negatives are already on the record and shape the order of work
- Frontier models asked to draw signature dependencies omitted six of fourteen; agents working from source alone found ten of nineteen local edges; a learned binding model accepted zero bindings where a rule accepted fifty-six. An omitted edge looks like independence, which is the most dangerous output a structural tool can produce.
- Therefore no model is in the demo and no model is in version one. The model is a ladder with a stop rule, run after the corpus exists.

---

### 3. The order of work: corpus, three tools, then the model — gated by three signals

#### 3.1 Three signals, each cheap, each able to kill a branch of the thesis
```text
 1. three-arm pilot on three unseen repositories, equal token budget:
      baseline agent · agent + map without model · agent + map + model
      if the first two arms tie on correctness, navigation is a feature, not a product      ~1 week, no training
 2. future-map authoring on real work:
      do agents and maintainers keep adding proposed edges after the first week, unprompted?
      if not, the future half of the map is dead and the product is a navigator              one design cycle
 3. model ladder, second rung (10 M tokens, an afternoon):
      held-out curve against the rule baseline; flat or below → keep rules, stop training   one afternoon, < $1
```

#### 3.2 Three tools, not a protocol
- `find` — seven words in, four candidate entities out, each with its public anchor, paths, blast radius and a bounded neighbourhood under a token budget. The retrieval protocol is its internals, not its interface.
- `impact` — callers, implementors, constructors, cfg-gated call sites, with file and line, and the coverage envelope printed on every answer. Proposed edges appear in the same answer, marked.
- `propose` — writes proposed nodes and edges into the map, with creator and reason. There is no fourth tool that grades them; re-extraction turns them exact or leaves them proposed.

#### 3.3 The ninety-second proof
```text
  0:00  map      banner: crates, files, public items; % exact; % unresolved and why; extractor + resolver version; feed crates loaded
  0:10  find     "where do retries get decided"  →  four candidates; picks one; neighbourhood under budget; blast radius
  0:25  propose  "make retry policy pluggable behind a trait"  →  three proposed edges appear on the map, dashed
  0:40  edit     halfway, the crate does not compile
  0:50  impact   still answers: exact edges that survive, 9 newly unresolved, the 3 proposed edges still showing the way
  1:10  fix      build green; re-extract: 2 proposed edges are now exact, 1 is still proposed — the agent sees what is left
  1:25  map      the dashed edge turns solid after the last edit; the region has no proposed edges; the plan simply became the code
```
- Prerequisites, none of which need a model: level-1 and level-2 extraction on non-compiling files; stable item identity across rename and move; a proposal format the agent emits; incremental re-extraction fast enough to run on every save. Identity and re-extraction latency are where the demo dies first.

#### 3.4 Publish the corpus and the feed before any product or model claim
- Permissive-licence sources only, with repository, commit, licence and extractor version on every record. Signed feeds with distribution levels and warninglists. The resolver version pinned per release, with a conformance suite in CI. A last-seen stamp on every edge so staleness is shown, never hidden.
- Semantic labels written by a large model live in their own layer with their own licence, checked before publication.

#### 3.5 What is deliberately not done
- No verification step, no diff, no "as-built" map, no conformance report, no CI gate. The future is the part of the map that is not yet exact; the compiler is the only judge, and it judges by extraction, not by a tool.
- No model-first story. No memory gate as a product requirement — it is a leaderboard column.
- No comment, import or global as a graph node.
- No claim that nothing else can do any of this; the claim is that nobody ships the combination, and nobody publishes the corpus or the feed.
- No deletion of earlier research to look tidy: one canonical page, and the collected record as the changelog.
- No reception table treated as market evidence; the market evidence is the three signals.

---

### The one risk the thesis does not remove

**Nobody writes the future half of the map.** Every earlier form of architectural intent — diagrams, architecture tests, design documents — was rejected as ceremony. Openness does not change that. The mitigation is that writing the future is one `propose` call the agent makes anyway while planning, and it costs nothing to leave the edges there. Signal 2 is the test. If it fails, the product is a navigator — a utility, not a company — and the corpus, the feed and the benchmark are still worth having built.

---

# Part 7d — 2026-11-06 — The thesis, revision 4 (system definition and entity model; verbatim copy of Deep-Insight-05.md)

*2026-11-06, revision 4. Proposal 01 stated top-down: the answer, the three reasons, the evidence under each, then the system definition and the entity model. Scope decisions carried by this revision: (a) no verification step, diff, as-built map or conformance report; (b) rust-analyzer and tree-sitter are not runtime components — they are the teacher that labels the training corpus and the judge that scores the model; (c) Parseltongue at runtime is one CLI interface over its own deterministic CLI tools and a set of tiny task-models that call each other to answer the agent-LLM; (d) the tiny models run on CPU or GPU; (e) the future half of the map is a structure the tool provides and never fills — only the orchestrator-LLM, the agent-LLM or a human writes into it. Earlier revisions are kept in the collected Deep-Insight-01 as Parts 7, 7b and 7c; this revision supersedes them where they differ. What is measured is marked measured; what is a bet is marked bet.*

---

### The answer

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

### 1. The object is new, and nothing else is the product

#### 1.1 Current and future on one vocabulary
- The map has one set of node and edge kinds. An edge produced by the extractor from real source is `extracted`, with a confidence. An edge the extractor declined to emit is `abstained`, recorded so that absence is never silent. An edge the agent-LLM, the orchestrator-LLM or a human adds because the code *should* have it is `proposed`.
- The tool provides the proposed slot and never fills it. Parseltongue has no component that invents future structure. The agent obtains proposed edges in one of two ways: it states them directly, or it hands its drafted code to the same extractor that reads real code, and labels the output `proposed`. Either way the author of record is the LLM or the human, not the tool.
- The plan completes itself. Each time real source is extracted, a proposed edge that now appears as extracted is marked realized; the rest stay proposed and visible. "Done" is a region with no open proposals. Extracted edges nobody proposed are simply shown; the map does not judge them.
- Every discipline that builds physical things draws what it intends before it builds. Software keeps only the building. Agents inherit that poverty: they re-derive the architecture from grep on every task and plan in prose that cannot be looked up later. A proposed edge can.
- A tool where the LLM only reads a graph is a code search engine. A map the LLM can also write the future into, on the same vocabulary it navigates by, is the thing not already shipping.

#### 1.2 Two levels are visible; the rest opens on demand
- **Level 1** — file to file: module declarations, imports, include paths, cfg gates.
- **Level 2** — the public face of each file: every public signature, re-export, static and constant, cfg and feature gate, the doc comment, item-level macro names. The test for membership: *could this line change what another file is allowed to write?*
- Comments, imports and globals are fields on the item record, not nodes. Imports are evidence on dependency edges; statics are items; a doc comment is the contract's prose attached to the item it documents.
- Bodies, calls, data flow, lock order, Send/Sync boundaries, and the semantic layer ("this is the retry loop") are kept and shown on expansion. They are not the default view.

#### 1.3 Honesty is a storage rule, not a promise
- Every record names its creator (which model and revision, which tool, which LLM, which human), its confidence, the input digest it was produced from, and the extractor version. Abstention is a record, not a gap. A proposed edge is never displayed as a fact about the code.
- Three layers are never written into each other: extracted structure, shared vocabulary (node and edge kinds, tags), and opinions (proposed edges, semantic labels, sightings). Any query can peel a layer off.
- This is the structure the cyber-threat-intelligence field settled on for knowledge graphs fed by many sources of unequal trust: provenance and confidence on every object, sightings to confirm or deny, warninglists for known false positives, first-seen and last-seen, feeds with distribution levels. It is what lets an opinion about the future sit next to an extracted edge without poisoning it.

#### 1.4 Open means the asset is the corpus, not the binary
- Four published artefacts: the Apache/MIT CLI and its versioned record schema; **a public feed of compiler-labelled graphs for every crate version** — docs.rs, but the graph; **the training corpus** with repository-disjoint splits and held-out repositories newer than any base model's cutoff; and the tiny models as open weights with card, eval harness and recipe.
- The feed changes what the extractor must know. It need not memorise the standard library and a hundred crates; it downloads those facts and learns only how to connect them — binding, chaining, abstaining. That is the small, defensible part.
- Each tiny model is one row on a leaderboard the project owns, scored by the compiler on held-out repositories. If a rule beats a model, the leaderboard says so and the project is still the benchmark everyone cites.

---

### 2. Rust makes the truth exact, and the broken moment is the real one

#### 2.1 Rust is the one mainstream language where the map has a ground truth
- Names are static, visibility is written in the source, each trait is implemented once per type, there is no reflection and no eval. The compiler's resolver produces the public-interface graph exactly, so every extractor — a large model, a tiny model, a heuristic — can be scored against truth. No other mainstream language allows this. This is why the teacher-and-judge arrangement is possible at all.
- Measured on a 42k-line, eleven-crate workspace: full rust-analyzer analysis 29 s at 2.3 GB; near-zero unknown expressions with proc macros enabled. That is the cost of truth, paid once per corpus release, off-line.
- Measured on two large public Rust codebases: 65 % of call sites are method calls, and only a quarter of their receivers can be typed from the enclosing function alone. The extractor cannot be a syntax-shaped trick; it has to learn resolution, or fetch the needed signatures from the feed, or abstain.

#### 2.2 The code is broken exactly when the map is needed most
- The tool is for cross-cutting change. Halfway through extracting a trait, forty files fail to type-check and anything that depends on a build or a type-checker degrades across the whole blast radius. A model that reads text does not fail to load; it reads what is there and abstains where it cannot resolve. The proposed edges say where the work is going while the extracted edges are in flux.
- The claim is not that a parser can index broken files; several tools do that. The claim is that the current and the future edges live on the same object, extracted by the same model from real code and from drafts, so the agent keeps navigating by the plan while the code catches up.

#### 2.3 The absolute claim is false; the combination claim is true and has one missing piece
- Each piece exists somewhere: module graphs, public-API listings, LSP-over-MCP symbol navigation, ranked repo maps within a token budget.
- No one ships a build-free, open-weight extractor for Rust structure plus budgeted agent queries plus proposed edges on the same map plus per-record provenance. A map the agent can write the future into exists nowhere. The compiler-labelled Rust graph corpus and the per-crate graph feed exist nowhere. Those absences are verifiable and cheap to fill.

#### 2.4 Who is convinced by what
- **Convinced by openness alone**: the rust-analyzer contributor (we consume their output off-line and publish the scores), the CTO (single binary, no egress, auditable, no build step in the agent's loop), the ML researcher (citable repository-disjoint data, a compiler as judge), the skeptic who calls it a wrapper (there is nothing to wrap; the extractor is ours and scored in public).
- **Convinced only by engineering**: the maintainer of a 300k-line workspace (proc macros, re-extraction in seconds, one `impact` answer that grep gets wrong), the harness builder (an A/B in their own logs), the newcomer (one command, `where do I even start`, on broken work-in-progress with no configuration).
- **Created by openness**: the crate author who finds their crate in the feed (licence passed through, a correction channel) and the contributor who wants to add a query, an edge kind or a model (schema and plugin points documented before the first release).

#### 2.5 The negatives are already on the record and shape the order of work
- Frontier models asked to draw signature dependencies omitted six of fourteen; agents working from source alone found ten of nineteen local edges; a learned binding model accepted zero bindings where a rule accepted fifty-six. An omitted edge looks like independence, which is the most dangerous output a structural tool can produce.
- Therefore the extractor ships only with its leaderboard row visible, abstention on, and the corpus published first. "If the tiny model works" is the bet of this proposal; the ladder in 3.1 is how it is tested cheaply, and a rule-based extractor remains a row on the same leaderboard.

---

### 3. The order of work: corpus, three tools, then the models — gated by three signals

#### 3.1 Three signals, each cheap, each able to kill a branch of the thesis
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

#### 3.2 Three tools, not a protocol
- `find` — seven words in, four candidate entities out, each with its public anchor, paths, blast radius and a bounded neighbourhood under a token budget. The retrieval protocol is its internals, not its interface.
- `impact` — callers, implementors, constructors, cfg-gated sites, with file and line, and the coverage envelope printed on every answer. Proposed edges appear in the same answer, marked.
- `propose` — writes proposed nodes and edges into the map on behalf of a named LLM or human, with reason. The tool validates shape and identity only; it never originates content.
- `map` is the extraction entry point (real source or an agent draft); `feed` fetches published crate graphs. Both are internal to the dispatcher; the agent sees three verbs.

#### 3.3 The ninety-second proof
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

#### 3.4 Publish the corpus and the feed before any product or model claim
- Permissive-licence sources only, with repository, commit, licence and labeller version on every record. Signed feeds with distribution levels and warninglists. The teacher's version pinned per corpus release. A last-seen stamp on every record so staleness is shown, never hidden.
- Semantic labels written by a large model live in their own layer with their own licence, checked before publication.

#### 3.5 What is deliberately not done
- No verification step, diff, as-built map, conformance report or CI gate. The future is the part of the map that is not yet realized; realization is observed by extraction, not judged by a tool.
- No rust-analyzer, tree-sitter or cargo in the runtime. They are teacher and judge; a build is never required to answer the agent.
- No tool-authored future. The proposed slot is a structure; only an LLM or a human fills it.
- No comment, import or global as a graph node.
- No claim that nothing else can do any of this; the claim is that nobody ships the combination, and nobody publishes the corpus or the feed.
- No deletion of earlier research to look tidy: one canonical page, and the collected record as the changelog.
- No reception table treated as market evidence; the market evidence is the three signals.

---

### 4. The system, defined

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

### 5. Entities, defined

#### 5.1 Actors
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

#### 5.2 Structural entities
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

#### 5.3 Status of an Item or Edge, and who may move it
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

#### 5.4 Authority matrix
```text
                 Snapshot  Item/Edge extracted  Item/Edge proposed  Annotation  Sighting  Coverage  FeedEntry  status flags
 TEACHER/JUDGE      —              —                   —                —           —          —        corpus      —
 MODEL              —           create                 —             create         —          —          —         —
 TOOL             create           —                   —                —           —       create    publish    realized · stale
 LLM                —              —                create            create      create       —          —       withdraw own
 HUMAN              —              —                create            create      create       —          —       withdraw any
```

---

### 6. What this revision changes, and why
- *Verification, diff, as-built, report* — removed. They were a second mechanism for something the map already does by being re-extracted; an unnecessary complication.
- *rust-analyzer and tree-sitter at runtime* — removed. If the tiny extractor works there is no runtime use for them; if it does not, the proposal fails on its own terms and a parser-based rung sits on the same leaderboard. They remain the teacher and the judge, which is the only arrangement in which an open-weight extractor can be trusted.
- *"Zero LLM inside Parseltongue, CPU-only"* — superseded. Parseltongue is now defined as a CLI over its own tools and tiny models; CPU is the floor, GPU is allowed.
- *Who authors the future* — fixed: the tool provides the slot and never fills it. Only an LLM or a human writes `proposed`.
- *Entities* — made explicit: identity rules, required fields, creators, statuses, transitions and an authority matrix, so that "honest" is checkable in a schema rather than asserted in prose.

---

### The one risk the thesis does not remove

**Nobody writes the future half of the map.** Every earlier form of architectural intent — diagrams, architecture tests, design documents — was rejected as ceremony. Openness does not change that. The mitigation is that writing the future is one `propose` call the agent makes anyway while planning, and the extractor can lift it from the agent's own draft. Signal 2 is the test. If it fails, the product is a navigator — a utility, not a company — and the corpus, the feed and the benchmark are still worth having built. The second risk, named rather than removed: the extractor may not reach the bar. Signal 3 tests it for less than a dollar.

---

# Part 7e — 2026-11-06 — Parseltongue: the map an agent can read, write, and trust (standalone Deep-Insight-05.md; verbatim copy)

*A standalone statement of the product. Answer first, then the three things that must be true, then the evidence under each, then the system and its entities. Where a number is measured it says measured. Where something is a bet it says bet.*

---

### The one-line answer

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

### What most people get wrong about this problem

Most people think the problem is *search*. Agents cannot find things in big codebases, so give them better search. That is the apparent problem.

The real problem is that **software has no drawing of itself.** A bridge has a blueprint, an as-built survey, and an inspector. Software has the building and nothing else. So every agent, on every task, re-derives the architecture from grep, plans in prose nobody can look up later, and edits with a picture of the system that lives only in its context window and dies when the window closes.

Search makes re-deriving faster. It does not stop the re-deriving. The thing that stops it is a map that persists, that the agent can write its intentions into, and that stays readable at the exact moment the code is torn apart.

The second thing people get wrong: they assume the map must come from the compiler at runtime. The compiler is the only source of *truth* about Rust structure — that part is correct. But a compiler needs a workspace, a build, two gigabytes and half a minute, and it degrades exactly when a refactor has broken the build. The compiler's job is to **teach** a small model what the map looks like and to **grade** that model on code it has never seen. Its job is not to be in the loop every time the agent asks a question.

---

### Three things that have to be true

```text
   1. THE OBJECT IS NEW        one map, current and future, one vocabulary, honest by construction
   2. RUST MAKES IT POSSIBLE   the only mainstream language with a ground truth to learn from and be graded against
   3. THE ORDER IS RIGHT       corpus and feed first · three verbs second · models last · three cheap kill-signals
```

---

### 1. The object is new, and nothing else is the product

#### 1.1 Current and future on one vocabulary

The map has one set of node kinds and one set of edge kinds. Three things can be true of an edge:

- `extracted` — a model read real source and produced it, with a confidence.
- `abstained` — a model saw the site and declined to resolve it. Recorded, so that absence is never silent.
- `proposed` — an LLM or a human wrote it because the code *should* have it.

That is the whole mechanism for planning. There is no plan document, no checklist, no step where a tool grades the plan. The agent writes the future as proposed edges — directly, or by handing its drafted code to the same extractor that reads real code. When real source is next extracted and the same edge appears, the proposed edge is marked `realized`. Done is a region with no open proposals. The plan becomes the code by being re-read, not by being verified.

**The tool provides the slot and never fills it.** Parseltongue has no component that invents future structure. The author of record for every proposed edge is a named LLM or a named human.

#### 1.2 Two levels are visible; the rest is one expansion away

- **Level 1 — file to file.** Module declarations, imports, include paths, cfg gates.
- **Level 2 — the public face of each file.** Every public signature, re-export, static and constant, cfg and feature gate, the doc comment, the item-level macro names. The membership test is one question: *could this line change what another file is allowed to write?*

Comments, imports and globals are **fields on the item record, not nodes**. An import is evidence on a dependency edge. A static is an item. A doc comment is the contract's prose, attached to the item it documents. Making them nodes buys identity churn and nothing else.

Bodies, calls, data flow, lock order, Send/Sync boundaries and semantic labels are all kept. They open on expansion. They are not the default view, because the default view is what an agent reads under a token budget.

#### 1.3 Honest is a storage rule, not a promise

Every record names its creator — which model at which revision, which tool, which LLM, which human — its confidence, the digest of the input it was produced from, and the schema version. Abstention is a record. A proposed edge is never rendered as a fact about the code.

Three layers are never written into each other:

```text
   extracted structure      what a model read in the source
   shared vocabulary        node kinds, edge kinds, tags — versioned by schema release, never by a model
   opinions                 proposed edges · semantic labels · sightings ("I confirm / I deny this edge")
```

Any query can peel a layer off. This is the discipline threat-intelligence platforms arrived at for knowledge graphs fed by many sources of unequal trust — provenance on every object, sightings, warninglists for known false positives, first-seen and last-seen, feeds with distribution levels. It is what lets an opinion about the future sit next to an extracted edge without contaminating it.

#### 1.4 Open means the asset is the corpus, not the binary

Four artefacts are published, and the binary is the least valuable:

1. The CLI and its versioned record schema. Apache-2.0 or MIT.
2. **A public feed of compiler-labelled graphs for every crate version.** docs.rs, but the graph.
3. **The training corpus**, with repository-disjoint splits and held-out repositories newer than any base model's cutoff.
4. The tiny models as open weights, with model card, eval harness and training recipe.

The feed changes what the extractor must know. It no longer has to memorise the standard library and a hundred crates; it fetches those facts and learns only how to connect them — bind, chain, abstain. That is the small, defensible part.

Each model is one row on a leaderboard the project owns, scored by the compiler on repositories the model never saw. If a rule-based extractor beats a model, the leaderboard says so, and the project is still the benchmark everyone cites.

---

### 2. Rust makes it possible

#### 2.1 The only mainstream language with a ground truth

Names are static. Visibility is written in the source. Each trait is implemented once per type. There is no reflection and no eval. So the compiler's resolver produces the public-interface graph *exactly* — and anything that approximates it, from a frontier model to a fifty-megabyte one, can be scored against truth. No other mainstream language allows this. It is the only reason a teacher-and-judge arrangement works.

Measured on a 42k-line, eleven-crate workspace: full rust-analyzer analysis takes 29 seconds and 2.3 GB, with near-zero unknown expressions once proc macros are enabled. That is the price of truth. It is paid once per corpus release, off-line, by the teacher — not by the agent on every question.

Measured on two large public Rust codebases: 65 % of call sites are method calls, and only a quarter of their receivers can be typed from the enclosing function alone. The lesson is sharp. **The extractor cannot be a syntax-shaped trick.** It has to learn resolution, or fetch the missing signatures from the feed, or abstain. Anything that silently guesses here produces the most dangerous output a structural tool can produce: an omitted edge that looks like independence.

#### 2.2 The broken moment is the real moment

The tool is for cross-cutting change. Halfway through extracting a trait, forty files fail to type-check and anything that depends on a build degrades across the whole blast radius. That is not an edge case; it is the centre of the use case.

A model that reads text does not fail to load. It reads what is there and abstains where it cannot resolve. The proposed edges say where the work is going while the extracted edges are in flux. Current and future live on the same object, extracted by the same model from real code and from drafts, so the agent keeps navigating by the plan while the code catches up.

#### 2.3 The absolute claim is false; the combination claim is true

Each piece exists somewhere: module graphs, public-API listings, symbol navigation over LSP, repo maps ranked into a token budget. Claiming "nobody can do any of this" would be wrong and would be caught in the first comment thread.

What exists nowhere: a build-free, open-weight extractor of Rust structure, plus budgeted agent queries, plus proposed edges on the same map, plus per-record provenance. A map the agent can write the future into. A compiler-labelled Rust graph corpus. A per-crate graph feed. Those absences are verifiable, and the last two are cheap to fill first.

#### 2.4 Who believes it, and what it takes

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

#### 2.5 The negatives are already on the record

Frontier models asked to draw signature dependencies omitted six of fourteen. Agents working from source alone found ten of nineteen local edges. A learned binding model accepted zero bindings where a rule accepted fifty-six.

This is why the extractor ships with its leaderboard row visible, abstention on, and the corpus published first. *If the tiny model works* is the bet of this proposal. It is tested for less than a dollar, below.

---

### 3. The order of work

#### 3.1 Three signals, each cheap, each able to kill a branch

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

#### 3.2 Three verbs, not a protocol

- **`find`** — seven words in; four candidate entities out, each with its public anchor, paths, blast radius and a bounded neighbourhood under a token budget. The retrieval machinery is internal.
- **`impact`** — callers, implementors, constructors, cfg-gated sites, with file and line, and the coverage envelope printed on every answer. Proposed edges appear in the same answer, marked.
- **`propose`** — writes proposed nodes and edges on behalf of a named LLM or human, with a reason. The tool validates shape and identity. It never originates content.

`map` (extract real source or a draft) and `feed` (fetch published crate graphs) exist, but the agent does not see them; the dispatcher does.

#### 3.3 The ninety-second proof

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

#### 3.4 Publish the corpus and the feed before any product or model claim

Permissive-licence sources only; repository, commit, licence and teacher version on every record. Signed feeds with distribution levels and warninglists. A last-seen stamp on every record so staleness is shown, never hidden. Semantic labels written by a large model live in their own layer with their own licence, checked before publication.

#### 3.5 Non-goals — stated so they stay non-goals

- **No verification step, diff, as-built map, conformance report or CI gate.** The future is the part of the map not yet realized; realization is observed by extraction, not judged by a tool.
- **No compiler, rust-analyzer, tree-sitter or cargo at runtime.** They are teacher and judge. A build is never required to answer the agent.
- **No tool-authored future.** The proposed slot is a structure; only an LLM or a human fills it.
- **No comment, import or global as a graph node.**
- **No "nothing else can do this."** The claim is that nobody ships the combination, and nobody publishes the corpus or the feed.
- **No reception table treated as market evidence.** The market evidence is the three signals.

---

### 4. The system, defined

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

### 5. The entities, defined

#### 5.1 Actors and their authority

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

#### 5.2 Structural entities

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

#### 5.3 Status, and who may move it

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

#### 5.4 The authority matrix

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

### The one risk this does not remove

**Nobody writes the future half of the map.** Every earlier form of architectural intent — diagrams, architecture tests, design documents — was rejected as ceremony, and openness does not change human nature. The mitigation is that writing the future is one `propose` call the agent makes anyway while planning, and the extractor can lift it from the agent's own draft. Signal 2 is the test. If it fails, the product is a navigator — a utility, not a company — and the corpus, the feed and the benchmark are still worth having built.

The second risk, named rather than removed: the extractor may not reach the bar. Signal 3 tests it for less than a dollar. A proposal that can be killed for a dollar in an afternoon is a proposal worth making.

---

### Broad ideas — seven threads, and where I land on each

*Appended after the thesis. These are the loose ends that decide whether the thesis is buildable. Each thread: what the question is, what is actually known, and a position to argue with.*

#### B1. The graph has to live somewhere

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

#### B2. Shrink a big model down, or grow a small one up?

The question is whether the small model should start as a *distilled and quantised* big model rather than a small model trained from scratch, because the big model already carries an approximate understanding of Rust that a from-scratch model would have to re-learn.

Yes — with one correction about what "shrink" can mean.

- **Quantisation alone does not get there.** Four-bit weights make a model four times smaller than sixteen-bit, not fifty times. A 7B model at four bits is still ~4 GB. Quantisation is the last step, never the strategy.
- **Pruning plus distillation does work, but by a factor of two to four per stage, not fifty.** The published pattern is depth- and width-pruning a parent model and distilling the pruned child on a small fraction of the parent's original training tokens; small production models of 1–3B parameters have been made this way from 8B parents. The child keeps most of the parent's capability at a fraction of the cost of training it from nothing.
- **Logit distillation from a large teacher is cheap and composable.** The large model's distribution over our corpus becomes a soft target alongside the compiler's hard labels. The compiler says *what* the edge is; the big model's distribution says *how a Rust-literate reader hesitates*, which is exactly what abstention needs.

So the realistic path is not "take a 70B model and nuff it down to 100 MB." It is: **start from the smallest existing code-pretrained model with the Rust prior already in it — the 0.5B–1.5B class — distil onto our compiler-labelled corpus with the big model's logits as a second teacher, then quantise.** That lands at roughly 300 MB to 1 GB on disk. Training from scratch is only cheaper if the task is narrow enough to be an encoder with heads, which is the binding-selection shape, not the extraction shape.

The position, bluntly: a model with the Rust prior and 1 GB beats a model without it at 100 MB, for this task, every time. Which forces B3.

#### B3. Where the CPU stops being enough

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

#### B4. The CLI stores both levels, and the database is not the API

The agent never sees the database. It sees three verbs. The store exposes one internal trait:

```text
   put(records) → snapshot id
   get(id) → record
   neighbours(id, kinds, depth, budget) → bounded subgraph
   search(text) → candidate ids
   proposals(region) → open proposals
```

Both levels are the same tables: a level-1 edge is `uses`/`declares_mod` between file-scoped items; a level-2 edge is `signature_uses`/`implements`/`reexports` between public items. The level is a filter, not a schema.

#### B5. Search: what FFF gets right, and what it does not do

FFF (dmtrKovalenko/fff) is a Rust file-search engine built for AI agents and editors — fuzzy path matching with frecency and git signals, ranked for "the file the user means," and shipped as an SDK across languages. It is the right reference for the *file-level* half of `find`: it has solved ranking files the way humans and agents actually name them, and it is fast.

What it does not do is the item level. Our hit is an item, not a file: `auth::session::create#fn`, with its public anchor, its folder chain and its blast radius. Position: **adopt FFF — or its ranking ideas — for the file retriever, and keep our own item retriever**: symbol trie for exact and prefix, trigram for typos, BM25 over sub-tokens of name, path and first doc line, fused by reciprocal rank. Both feed the same candidate list. The important property is that every candidate is a real id from the index, so nothing can be invented; FFF has the same property at file granularity, which is why it belongs.

#### B6. Queryable means "zero in," and the workflow is already written

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

#### B7. A conversation, not a query

The agent and the tool talk. Three things follow.

- **The session has state.** Candidates shown, cluster chosen, budget spent, proposals open. Event 6 is meaningless without it.
- **The tool may ask back.** "Did you mean the trait or the impl?" "I abstained on nine receivers in this crate — the feed has signatures for `tokio@1.38`; fetch?" "Three open proposals touch this region; show them?" A tool that can ask is a tool that can stay honest under a budget instead of guessing.
- **One wire format, two carriers.** JSON lines over stdin/stdout for any orchestrator; an MCP adapter over the same messages for editors. The verbs are the vocabulary; the session is the grammar; the dispatcher is the speaker.

What the conversation is *not*: the tool does not plan, does not write code, does not decide what the future should be. It answers, it asks when it must, and it records what the LLM or the human decides. The authority matrix above is the whole contract, and the conversation never exceeds it.

#### The decisions these seven force

```text
   store         record log is truth · SQLite-semantics view first (SQLite or Turso, by benchmark) · SurrealDB the graph-native candidate · others behind a trait
   model         start from a 0.5–1.5B code model · distil on the compiler corpus with big-model logits · quantise last
   hardware      1 GB VRAM target for extraction · CPU floor for the verbs · 100 MB RSS gate for index + verbs only
   search        FFF-class file retriever + own item retriever · every candidate is a real id
   zero-in       public anchor + folder chain + blast radius 1 on every hit, always
   conversation  stateful session · the tool may ask · JSON lines + MCP adapter over the same messages
```

#### B8. The 10-million-line prediction — tiny model on a GPU versus rust-analyzer

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

##### B8.1 The same prediction on a Mac mini

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

##### B8.2 Why reading is not free, and the four ways to make it so

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

#### B9. What size of model — meaningful classification, faster than rust-analyzer, in less peak RAM

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

#### B10. What a cybersecurity paper knows about code maps that the code-map literature does not

*Deep research on Sikos, "Cybersecurity knowledge graphs", Knowledge and Information Systems 65 (2023), open access, in the repository as `deep-insight-sikos-2023-cybersecurity-knowledge-graphs-KAIS.pdf`. Read against the arXiv literature on graphs for LLM agents over repositories (2024–2026) and against the industry systems that already run the pattern in production. Where a precedent was read from its abstract only, it says so.*

##### The claim

The paper is a survey of knowledge graphs for threat intelligence. It contains no code, no compiler and no LLM. Its contribution to Parseltongue is one pattern, stated in §1 and used in §7, that the entire code-graph-for-agents literature lacks: **a graph assembled from many sources of unequal trust, where every statement carries its source, where agreement between independent sources is the measure of correctness, and where the graph is designed to be queried with some sources switched off.** The code-graph papers have graphs. None of them has a trust model. That gap is the product.

##### What the paper actually says

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

##### What the code-graph literature has, and what it lacks

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

##### The industry already runs the pattern — in a different field

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

##### Why the code-graph field missed it

Each system has one source — a parser, a compiler, an embedding — and one author. When there is one source there is nothing to disagree with, so provenance looks like bureaucracy. The moment there are two sources it becomes the whole question: *which edge do I believe?* Parseltongue has six writers by design — extractor, binder, ranker, labeller, feed, agent, human — and the first paper to address that situation squarely is a cybersecurity survey, because cybersecurity has had many sources from the beginning.

The paper's single most transferable sentence is the one about two independent sources confirming a score. It is the mechanism behind `realized`: the agent proposed the edge, the extractor later found it, two independent actors agree, the record is now the most trustworthy kind in the map. It is also the mechanism for raising confidence without a compiler at runtime: two extractor revisions, or an extractor and the feed, agreeing on an edge is Sikos's "high likeliness of correctness" — and their disagreement is exactly what `abstained` is for.

##### What this adds to the thesis

Nothing changes in the entity model; this section is why it looks the way it does. Three things sharpen:

1. **Named graph per actor and revision is the storage primitive**, not a tag on a record. It is what makes "answer without the model layer" a filter and what lets a model revision be retired without touching anyone else's records.
2. **Agreement is the confidence function.** Extracted-matches-proposed, extractor-matches-feed, revision-matches-revision — each is a sighting by an independent actor. Confidence is computed from sightings, never asserted by one writer.
3. **The fourth cheap signal**: the same three-repository pilot, with the provenance and abstention fields shown to the agent in one arm and hidden in the other. If the arm that can see them makes fewer edits on unresolved edges, the trust layer earns its place in the output, not only in the store.

##### Sources

- Sikos, L. F. "Cybersecurity knowledge graphs." *Knowledge and Information Systems* 65, 3511–3531 (2023). https://doi.org/10.1007/s10115-023-01860-3 — open access; PDF in this repository.
- arXiv: 2406.12276 · 2408.03910 · 2410.14684 · 2503.09089 · 2503.22424 · 2505.14394 · 2505.16901 · 2507.14791 · 2510.04905 (survey) · 2603.24837 · 2607.25431 · 2608.01927 · 2608.29606 · 2611.07286.
- STIX 2.1 specification (OASIS); MISP project documentation; W3C PROV-O; Dong et al., "Knowledge Vault" (KDD 2014); Kythe, Glean, SCIP documentation; Joern; in-toto, SLSA, Sigstore. Industry entries are from working knowledge of the systems, not re-verified for this section except STIX's Sighting and Opinion definitions.
