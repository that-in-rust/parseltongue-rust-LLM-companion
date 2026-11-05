# Spec-first, evidence-bearing code graphs for coding agents

## A long-form thesis on what Strata and Clarity point at, what the literature says, and what a Rust public-interface graph should therefore become

Written 2026-11-05 on branch `research/tiny-rust-code-mapper`. Companion to the three earlier notes in `devin-research/` (usefulness/form/decisions; 10M-LOC and aggregation; Strata/Clarity inspection).

### Provenance and how to read this

- Every arXiv reference below was resolved through the arXiv API on 2026-11-05 (title, authors, year, abstract). Every classic (non-arXiv) reference was resolved through Crossref by DOI. Web references were fetched and the quoted passages copied from the live page on the same day.
- For most papers I read the abstract and, where the abstract gave numbers, I quote those numbers. I did **not** read every full text. Where a claim rests on an abstract alone it is marked *(abstract)*. Where a claim is my inference it is marked *(inference)*.
- The Strata facts come from the previous note's inspection of the distributed macOS bundle (`strings` on a stripped Swift binary, the SQLite schema, the embedded agent prompt) plus the public product page. They are not from source code.
- Numbered citations like [12] refer to the reference list at the end.

---

## 0. The thesis in five sentences

1. Strata and Clarity are two halves of a very old idea, the **software reflexion model** (Murphy, Notkin and Sullivan, 1995 [5]): a high-level model that someone *intends*, a source model *extracted* from the code, and a checked mapping between them. Strata is the intended model made first-class, versioned and agent-writable; Clarity is the extracted model made cheap, multi-language and evidence-bearing. Neither closes the loop between the two.
2. The 2023–2026 coding-agent literature has converged on the same shape independently: agents localize and edit better when given a **budgeted projection of a structural graph** rather than similarity search or raw files (RepoCoder → CodePlan → RepoGraph → CodexGraph → CGM → LARGER/RepoAtlas) [14–30], but the graph must be deterministic and evidence-backed to be trusted (RIG [29]) and token-cheap to be used (Aider's 1k-token map [31], Codebase-Memory's 10× token reduction [28]).
3. The same literature is blunt about the model-builds-the-graph hypothesis: **static tools beat LLMs at call-graph construction** (Venkatesh et al. 2024 [33], Su et al. 2025 [34]); LLMs earn their keep as *resolvers of residual hard cases inside a static framework* (LLift, IRIS, LLMDFA [35–37]) and as *decoders constrained by analysis* (Monitor-Guided Decoding [38], type-constrained decoding [39], MARIN [40]). Any Rust model should be positioned that way, not as a graph generator.
4. **Compile-before-commit** is not a Strata quirk; it is the unifying principle behind constrained decoding [41–46], verifier-in-the-loop RL [47–51], closed-loop verifiable generation (Clover [52]) and spec-driven workflows (Spec Kit, Kiro, Spec Kit Agents, SSDE, spec-driven test generation [53–58]). The gate is simultaneously the runtime guardrail and the training reward.
5. Rust is the one mainstream language where the intended model and the extracted model can be written in the **same formalism**: a Rust skeleton with `todo!()` bodies compiles, and rustdoc JSON [60] turns it into the same contract graph that the finished crate produces. That collapses the reflexion gap that every other spec tool (including Strata's D2 diagrams) has to bridge by hand, and it is the specific reason a Rust public-interface graph is worth building before a universal one.

Everything below is argument and evidence for those five sentences, then a concrete architecture, research questions with falsifiable predictions, and the case against.

---

## 1. The lineage: this problem is fifty years old

It helps to see Strata and Clarity as late entries in a long sequence, because the sequence tells you what has already failed and why.

**1972 – Parnas, "On the criteria to be used in decomposing systems into modules" [1].** The founding argument for information hiding: a module's *interface* is what other modules may depend on; the rest is a secret that can change. The phrase "public-interface dependency graph" is literally Parnas's criterion drawn as a graph. Rust's `pub`/private default is the most literal language-level encoding of this that has reached mass use *(inference)*.

**1981 – Steward, the Design Structure Matrix [2]; 2006 – MacCormack, Rusnak and Baldwin apply DSMs to open-source software [3].** The DSM is the dependency graph as an adjacency matrix, and its whole point is *aggregation*: you cluster rows into modules and read coupling, cycles and propagation cost off the matrix. Clarity's `modules`, `--collapse` and cycle break-sets are DSM operations under another name. MacCormack et al. showed on Mozilla and Linux that "propagation cost" (how many files a change can reach transitively) is a measurable, architecture-sensitive quantity. That is the metric our "what breaks if I change X" query computes per item instead of per file.

**1987 – Ferrante, Ottenstein and Warren, the Program Dependence Graph [6]; 1990 – Horwitz, Reps and Binkley, the System Dependence Graph and interprocedural slicing [7].** These are the intra- and inter-procedural *data and control* dependence graphs. Our proposed `CALLS`/`SIGNATURE_USES` graph is a coarse projection of an SDG where the nodes are items rather than statements. The reason to stay coarse is cost and stability, not ignorance of the finer graph; the SDG is what you build *inside* an item when the agent asks for it *(inference)*.

**1995/1996/2001 – call-graph construction as a discipline [8–10].** Dean, Grove and Chambers's Class Hierarchy Analysis, Bacon and Sweeney's Rapid Type Analysis, and Grove and Chambers's unifying framework established that call graphs for languages with dynamic dispatch are *over-approximations* whose precision is a tunable cost. Rust changes the terms: with no inheritance, coherence-bounded trait impls and explicit `dyn`, the over-approximation is confined to trait-object call sites, which are syntactically visible. This is the formal reason the earlier note's "trait-object fan-out is one of three hard problems" claim holds *(inference from [8–10] plus Rust semantics)*.

**1992 – Perry and Wolf, architecture erosion and drift [11]; 1995/2001 – Murphy, Notkin and Sullivan, software reflexion models [5, 12].** Perry and Wolf named the disease: implementation diverges from intended architecture. Murphy et al. prescribed the treatment: write the *high-level model* you believe in, extract a *source model* from the code, declare a *map* between them, and compute a reflexion model that shows convergences, divergences and absences. Their 2001 TSE paper reports the method applied to Excel and others. This is the single most important reference in this document, because **Strata is a high-level-model editor and Clarity is a source-model extractor, and the reflexion computation between them is the missing product.**

**2014 – Yamaguchi et al., Code Property Graphs [13].** The CPG merged AST, control-flow and PDG into one queryable property graph and showed that vulnerability patterns become graph traversals. This is the ancestor of CodexGraph-style "agent writes Cypher against a code graph" [21] and the strongest argument for a *single store with multiple projections* rather than several bespoke graphs.

**2015 – Néron, Tolmach, Visser and Wachsmuth, scope graphs ("A Theory of Name Resolution") [4]; 2021 – GitHub's stack graphs [62, 63].** Scope graphs are a language-independent theory of name binding; stack graphs are GitHub's incremental, per-file, build-free realisation of them, built precisely so that cross-file navigation at GitHub scale could be computed without running anybody's build. GitHub's blog [62] states the two constraints: "we must be able to rely on incremental processing and storage, reusing the results that we've already calculated and saved for the files that haven't changed," and "we don't want to require manual per-repository configuration." Those are exactly the constraints that justify a per-file Rust extractor with host-side linking (decision D2 in the first note) and they were met in production without a model.

**1993 – Eades, Lin and Smyth's feedback-arc-set heuristic [64].** Clarity's cycle break-sets ("exact minima for bounded components and labelled heuristics for larger ones", README [65]) are the minimum feedback arc set problem, NP-hard in general (Karp 1972), with Eades–Lin–Smyth as the standard fast heuristic. Worth knowing so we do not re-derive it.

**Reading of the lineage.** Every generation built the graph and most found that humans rarely *look* at it. What changed in 2023 is the appearance of a consumer that will query it thousands of times a day without complaint. The 2006 DSM paper and the 2001 reflexion paper were written for architects who had minutes per decision; a coding agent has milliseconds and no patience for a 3,000-node picture. That consumer shift, not any new graph theory, is why the artefact is worth building now *(inference)*.

---

## 2. Where Strata and Clarity sit, precisely

### 2.1 Clarity = DSM + reflexion source-model, at file granularity, with evidence

From the README [65], verified today: "Clarity works at file granularity. It recovers coupling shape, not runtime behavior or full API contracts." And under *What Clarity Is Not*: "It does not provide: full symbol-level call graphs, runtime behavior analysis, API contract verification, semantic correctness guarantees. It is a structural verification tool."

Three design choices are genuinely good and transfer directly:

1. **Evidence on every edge.** `DependencyEvidence{symbol, kind, relationship, reference_file/line, declaration_file/line, confidence}` with a closed relationship taxonomy (`import`, `re-export`, `call`, `type-reference`, `module-declaration`, …). An agent that is told *why* an edge exists can decide whether to trust it; an agent given a bare adjacency list cannot. The Repository Intelligence Graph paper [29] reaches the same conclusion from the build-system side: edges must "trace back to concrete build and test definitions" to be treated as authoritative *(abstract)*.
2. **A query grammar, not an endpoint zoo.** `docs/show-surface-v2.md` factors every query as Snapshot × Anchor × Lens × Render. This is the same factoring the agent-interface literature arrives at: SWE-agent's finding that the *agent-computer interface* (few, well-shaped commands with good feedback) matters as much as the model [19], and Hsieh et al.'s finding that tool *documentation* alone enables zero-shot tool use [66].
3. **Cycles as a first-class, verified output.** Break sets are re-verified after computation. Strata does the same for diagrams (compile before store). The pattern "every emitted artefact is re-checked by a deterministic procedure before a human or agent sees it" is the thread that runs through this whole document.

Its ceiling is also precise. The Rust evidence extractor (`parser_rust_evidence.go`) classifies an identifier followed by `(` or `::<` as a `call` and a capitalised identifier as a `type-reference`. That is a lexical heuristic, which is fine for file-level coupling shape and useless for "who calls `Store::open`" at item level. Clarity is therefore the right **baseline** for a Rust item-level graph and the wrong **foundation** for it.

### 2.2 Strata = the high-level model, made first-class, compiled, versioned and agent-writable

From the product page [67], verified today: "Your coding agent connects over a local MCP server and reads the same diagram you see. It builds against the spec instead of a prompt it half-remembers." And: "Each edit compiles before it lands and is captured as an immutable snapshot. Diff any two versions, restore any of them, and watch the spec evolve with the build."

From the bundle inspection (previous note): constrained *kits* (`user_flow`, `discovery_tree`, freeform D2); a `describe_kit → validate_artifact → create/update_artifact` tool sequence with structured diagnostics (severity, message, line, node); optimistic concurrency through an opaque `version_handle`; a `version` table whose rows carry `source`, `parent_id`, `render_ok`, `render_log`; and an embedded instruction to keep diagrams to roughly 8–15 nodes.

What Strata gets right is not the diagramming. It is four workflow decisions:

1. **Spec is an artefact with a compiler.** A D2 document either compiles or it does not. Natural-language specs have no such gate. This is the *compile-before-commit* principle, and it is what makes the artefact safe to let an agent write.
2. **Constrained authoring by default.** Kits restrict what an agent can say, so the validator can say more. The constrained-decoding literature (Section 4) is the formal version of this trade.
3. **Immutable versions with parent pointers.** The spec has a history, so "what did the spec say when this code was written" is answerable, and the spec can be diffed like code.
4. **A budget on the artefact.** 8–15 nodes is a context-window discipline disguised as a design-aesthetics rule. Aider's repo map [31] enforces the same kind of budget (default `--map-tokens` 1k) algorithmically; RepoAtlas [27] selects "a task-relevant region under a fixed budget" *(abstract)*.

What Strata cannot do, by construction, is check the spec against the code. D2 nodes are strings; nothing ties "Checkout service" to `crate::checkout::Service`. The reflexion loop is open.

### 2.3 The gap between them is the product

Put Murphy's three pieces side by side:

| Reflexion-model component | Strata | Clarity | Missing for Rust |
|---|---|---|---|
| High-level (intended) model | D2 diagrams, versioned, agent-writable | — | A **Rust skeleton**: real `pub` signatures, `todo!()` bodies, that compiles |
| Source (extracted) model | — | File-level import/`mod`/re-export graph with evidence | **Item-level** graph: items, impls, signatures, calls, with rustdoc/rust-analyzer-grade resolution |
| Map between them | — | — | **Identity**: `crate@ver::path::Item` is the same string in both, so the map is the identity function |
| Reflexion computation | — | `cycles`, `modules` (structural checks only) | convergence/divergence/absence per public item; semver diff; architecture rules |

The last column is the earlier notes' proposal restated as the closing of a 30-year-old loop. The reason it is tractable now for Rust and was not for Murphy's C/C++ Excel is the identity row: when the intended model is itself Rust, the mapping problem that consumed most of the effort in reflexion-model studies disappears *(inference)*.

---

## 3. What coding agents actually need from a graph: the 2023–2026 evidence

The repository-level code-generation literature is large; what follows are the papers that changed the consensus, and the lesson each one carries for the graph's *form*.

**RepoCoder (Zhang et al., 2023) [14]** — iterative retrieval–generation for repo-level completion. Established that cross-file context is the bottleneck and that similarity retrieval alone is weak at finding it.

**RepoBench (Liu et al., 2023) [15]; Long Code Arena (Bogomolov et al., 2024) [16]; CodeRAG-Bench (Wang et al., 2024) [17]; SWE-bench (Jimenez et al., 2023) [18]** — the benchmarks. CodeRAG-Bench's finding that retrieval helps only when the retriever is good enough is the quantitative version of "a bad graph is worse than no graph" *(abstract)*.

**CodePlan (Bairi et al., 2023, FSE 2024) [20]** — repository-level edits (migrations, type annotation) framed as planning over "a novel combination of an incremental dependency analysis, a change may-impact analysis, and adaptive planning" *(abstract)*. Lesson: the graph must be **incremental** and must support **may-impact** queries; both are in our decision list, and CodePlan is the citation for them.

**SWE-agent (Yang et al., 2024) [19]** — "Agent-Computer Interfaces" matter: the shape, count and feedback of tools changes success rates substantially. Lesson: the MCP surface is a research object, not plumbing.

**Agentless (Xia et al., 2024) [22]** — a three-phase localization/repair/validation pipeline with no autonomous agent competed with full agents on SWE-bench. Lesson: *localization* is most of the problem, and localization is what a structural graph is for.

**RepoGraph (Ouyang et al., 2024) [23]** — a line-level repository code graph plugged into four SWE-bench systems "substantially boosts the performance of all systems" *(abstract)*. Lesson: a graph is a plug-in that improves whichever agent you have, which is the right product shape for Parseltongue.

**CodexGraph (Liu et al., 2024) [21]** — the agent writes graph-database queries (Cypher) against an extracted code graph; evaluated on CrossCodeEval, SWE-bench, EvoCodeBench. Lesson: a *query language* generalises across tasks where "manual tools and APIs are typically task-specific" *(abstract)*. This argues for exposing Clarity-style Snapshot × Anchor × Lens × Render *and* a raw query escape hatch.

**GraphCoder (Liu et al., 2024) [24]; DraCo (Cheng et al., 2024) [25]** — code-context graphs and dataflow-guided context graphs for completion; DraCo reports +3.43% exact match and identifier-F1 gains over import/similarity retrieval on Python *(abstract)*. Lesson: dataflow/type edges, not just imports, are what move completion quality — i.e. `SIGNATURE_USES` and `CALLS` matter more than `DEPENDS_ON_CRATE`.

**CGM, Code Graph Model (Tao et al., 2025) [26]** — integrates the repository graph into the LLM's attention via an adapter; 43.00% on SWE-bench Lite with open models and an agentless graph-RAG loop *(abstract)*. Lesson: the graph can be consumed *inside* the model, not only as text. This is the credible alternative to our "small model builds the graph": a small model that *reads* a compiler-built graph.

**GRACE (Wang et al., 2025) [30]** — "multi-level, multi-semantic code graph that unifies file structures, ASTs, call graphs, class hierarchies, and data flow graphs" with hybrid GNN + text retrieval *(abstract)*. Lesson: the field is converging on a single multi-level store with projections, exactly the "store everything, show the contract" position.

**Prometheus (Pan et al., 2025) [32]** — "the Needle-in-a-Haystack problem persists even with million-token context windows" *(abstract)*; represents the repo as a unified graph with memory across steps. Lesson: long context does not retire the graph; it retires the excuse for not having one.

**LARGER (Hu et al., 2026) [27a]** — formalises "Lexically Anchored Structural Localization": turn a grep hit into a high-precision structural entry point and expose "confidence-filtered local neighborhoods within the agent's existing search loop" *(abstract)*. Lesson: the graph should answer "expand the neighbourhood of this symbol, filtered by confidence" as a single call, and confidence must be stored per edge.

**RepoAtlas (Zhang et al., 2026) [27]** — "rendering the full repository graph yields visual representations that are too dense to perceive reliably, whereas a one-shot local view becomes stale"; solution is a select–project–refresh loop under a fixed budget *(abstract)*. Lesson: the five-layer hierarchical view from the earlier note needs a *refresh* operation as the agent moves, not a static snapshot.

**Codebase-Memory (Vogel et al., 2026) [28]** — a tree-sitter knowledge graph served over MCP, 66 languages, evaluated on 31 repositories: "83% answer quality versus 92% for a file-exploration agent, at ten times fewer tokens and 2.1 times fewer tool calls" *(abstract)*. This is the most important negative-and-positive result in the set. Positive: the token economics of a graph over MCP are real. Negative: a *tree-sitter-only* graph loses nine points of answer quality to plain file exploration. That is the quality ceiling of Parseltongue's current approach, measured by someone else. The way through is better resolution, not more languages.

**Repository Intelligence Graph (Cherny-Shahar et al., 2026) [29]** — "a deterministic, evidence backed architectural map … that agents can treat as the authoritative description of repository structure," evaluated with Claude Code, Cursor and Codex *(abstract)*. Lesson: *determinism* and *evidence* are what let an agent treat a graph as authoritative rather than as one more noisy hint. A model-generated graph without provenance and abstention can never be authoritative in this sense.

**Lost in the Middle (Liu et al., 2023) [68]** — models use the beginning and end of long contexts better than the middle. Lesson: a budgeted, ranked projection is not just cheaper; it is more *accurate* than dumping more text.

### 3.1 Six requirements the evidence imposes on the graph's form

1. **Structural over lexical for localization**, but anchored *by* lexical hits (LARGER, Agentless). The graph needs a fast symbol-name index as its front door.
2. **Edges carry evidence and confidence** (Clarity, RIG, LARGER). Our edge metadata list (provenance, resolution status, confidence, cfg) is not over-engineering; it is the minimum the literature uses.
3. **Budgeted, ranked, refreshable projections** (Aider, RepoAtlas, Lost in the Middle). Each layer of the five-layer view needs a token budget and a ranking (PageRank-on-the-graph as Aider does, or task-conditioned as RepoAtlas does).
4. **Incremental and may-impact** (CodePlan, stack graphs). Per-file extraction with host-side linking is the architecture that gives both.
5. **Type/dataflow edges, not just imports** (DraCo, GraphCoder, GRACE). `SIGNATURE_USES` and `CALLS` are where the value is; crate/module edges are table stakes Clarity already gives away free.
6. **Deterministic baseline, authoritative by construction** (RIG, Codebase-Memory's 9-point gap). The compiler-derived graph is the product; anything a model adds must be labelled as such and verified before it is merged.

---

## 4. Can a model build the graph? What the evidence says about LLMs doing static analysis

This is the research bet in the Parseltongue docs, so it deserves the harshest reading.

**Venkatesh et al. 2024, "An Empirical Study of LLMs for Type and Call Graph Analysis in Python and JavaScript" [33].** 24 LLMs evaluated; "for call-graph generation, traditional static analysis tools such as PyCG for Python and Jelly for JavaScript consistently outperform LLMs" *(abstract)*. Type inference was more favourable to large models. Note the languages: Python and JavaScript, where (per the earlier note) resolution is undecidable in practice and the *ground truth itself* is a static over-approximation.

**Su et al. 2025, "Do Code LLMs Do Static Analysis?" [34].** Tests call-graph, AST and dataflow generation against code intelligence tasks; the framing is that code LLMs "could exhibit a wholly alien thought process" and the study questions whether static-analysis skill and task skill are linked in LLMs at all *(abstract)*.

**LLift (Li et al., 2023) [35], IRIS (Li et al., 2024) [36], LLMDFA (Wang et al., 2024) [37].** The successful pattern in LLM-assisted static analysis is the opposite of "LLM emits the graph": a classical analysis (CodeQL in IRIS; a dataflow framework in LLMDFA; UBITect in LLift) does the bulk, and the LLM is asked narrow, well-posed questions at the points where the classical analysis is imprecise or needs a specification (e.g., is this function a taint source? does this path's condition hold?). The LLM is a **resolver of residuals under a static harness**.

**Monitor-Guided Decoding (Agrawal et al., 2023) [38].** A language server monitors decoding and masks tokens that would dereference non-existent members. "LMs with fewer parameters, when augmented with MGD, can outperform larger LMs. With MGD, SantaCoder-1.1B achieves better compilation rate and next-identifier match than" a much larger model *(abstract)*. Two lessons: (a) a static oracle plus a small model beats a big model without one, which is the strongest published support for the small-local-model thesis, but *with the oracle on the outside*; (b) the oracle here is a language server, i.e. exactly the thing rust-analyzer already is for Rust.

**Type-constrained decoding (Mündler et al., 2025) [39]** extends this from "does the symbol exist" to "is the expression well-typed", with prefix automata over a type system (formalised for a simply-typed language, implemented for TypeScript). **MARIN (Chen et al., 2025) [40]** mines "hierarchical dependencies" of the function being written and uses them for "Dependency Constrained Decoding" so that generated API calls exist in the project *(abstract)*. Both are direct uses of a dependency graph as a decoding constraint — the "catch made-up APIs" use case from the 10M-LOC note, already published for other languages.

### 4.1 What this means for the Rust model bet

- **Positioning.** The model should be sold, trained and evaluated as a *resolver and proposer under a verifier*, never as a graph generator. Its inputs are a file, its `use` tree, the crate module map and a candidate edge; its outputs are a resolution plus abstention. The verifier (rustdoc JSON / rust-analyzer when the crate builds; structural consistency checks when it does not) decides what enters the graph. This matches the candidate-edge-classifier design already in `docs-lossfunk/01` and matches how LLift/IRIS/LLMDFA succeeded.
- **Why Rust changes the Venkatesh result.** In Python the training label is PyCG's over-approximation; the student learns a heuristic's errors. In Rust the label is the compiler's resolution, which is *exact* for non-macro, non-`dyn` sites. Supervised distillation from exact labels is a different, easier problem than the one Venkatesh measured *(inference; this is the hypothesis the project should test first, and it is falsifiable)*.
- **The honest comparison class** is not tree-sitter and not GPT-class models; it is rust-analyzer running on a broken tree. rust-analyzer already does best-effort resolution on non-compiling code. The model has to beat it on (i) latency per edit at 10M LOC, (ii) memory, or (iii) macro/attribute-heavy code where r-a's expansion gives up. If it beats it on none of the three, CGM-style *graph-reading* models [26] are the better use of a small model.
- **Kill criterion.** If, on a held-out repository set, model-resolved edges verified against rustdoc JSON do not exceed rust-analyzer's precision/recall on the *same broken snapshots* in at least one of the three regimes above, stop training resolvers and redirect the model budget to ranking/summarising projections of the compiler graph.

---

## 5. Compile-before-commit as a general principle

Strata's `validate_artifact` is one instance of a pattern the literature has converged on from three directions.

### 5.1 Constrained decoding: make invalid output unrepresentable

PICARD (Scholak et al., 2021) [41] rejected SQL tokens incrementally; Synchromesh (Poesia et al., 2022) [42] added semantic constraints (Constrained Semantic Decoding) and showed large gains on code generation; Geng et al. (2023) [43] formalised grammar-constrained decoding without fine-tuning; Willard and Louf's Outlines (2023) [44] made it an FSM over the vocabulary; SynCode (Ugare et al., 2024) [45] and XGrammar (Dong et al., 2024) [46] made it fast enough for production. The frontier moved from syntax to types (Mündler [39]) and to project dependencies (MARIN [40], MGD [38]).

**Application.** A model-emitted graph fragment (a JSONL record `{src, dst, rel, span, …}`) should be decoded under a grammar (the record schema) *and* a semantic monitor (does `src` exist in the current file's item list? is `rel` legal between these node kinds? is `dst` in the `use`-reachable set or marked `external`/`unresolved`?). The monitor is cheap because Rust's resolution rules are local; it is the same monitor the verifier uses afterwards. This is Strata's kit-plus-validator idea moved into the decoder.

### 5.2 Verifier-in-the-loop learning: make the gate the reward

CodeRL (Le et al., 2022) [47] used unit-test outcomes as RL reward; RLTF (Liu et al., 2023) [48] made the test feedback fine-grained; RLCF (Jain et al., 2023) [49] used *compiler* feedback plus a discriminator for "coarse-tuning"; StepCoder (Dou et al., 2024) [50] curriculum-trained on compiler feedback; RLEF (Gehring et al., 2024) [51] grounded code LLMs in execution feedback with RL and reported large gains in sample efficiency. The Rust-specific instances are RustAssistant / "Fixing Rust Compilation Errors using LLMs" (Deligiannis et al., 2023) [69], AutoVerus (Yang et al., 2024) [70] which uses the Verus verifier as the loop oracle, and the C-to-Rust translators Syzygy [71] and SmartC2Rust [72] which iterate against `rustc` and tests.

**Application.** Strata-style structured diagnostics from the graph validator (`severity, message, span, node`) are a reward signal, not just a UI. Precision against rustdoc JSON per relation type, plus a penalty for confident wrong edges and a smaller penalty for abstention, is the reward. The same loop that guards runtime output trains the model. Self-Debugging [73] and Reflexion [74] are the inference-time versions (feed the diagnostics back and retry), which work without any training and should ship first.

### 5.3 Closed-loop consistency: triangulate

Clover (Sun et al., 2023) [52] checks pairwise consistency among code, docstring and formal annotation, with the thesis that an incorrect artefact is unlikely to be consistent with two independent others. For us the triad is **skeleton (intended) – extracted graph (actual) – tests/semver checks (behavioural)**. The reflexion computation of Section 2.3 is a Clover-style consistency check with the mapping made trivial by shared identity.

### 5.4 The principle, stated

> Nothing a model writes becomes part of the artefact until a deterministic procedure that does not use the model has accepted it; the acceptance procedure emits structured diagnostics; the diagnostics are the model's reward.

Strata applies it to D2. Clarity applies a weaker form (re-verify break sets). The whole constrained-decoding and RL-from-feedback literature applies it to code. Applying it to the *graph itself* is the design move.

---

## 6. Spec-first for agents: what the 2025–2026 wave shows, and why a Rust skeleton is the right spec medium

Spec-driven development for agents went from a slogan to tooling in roughly eighteen months.

- **GitHub Spec Kit [53]** (README, verified): "Define what to build and why, then turn the specification into a plan, tasks, and working code … Create a constitution once per project and repeat the remaining steps for each feature." Commands `/speckit-constitution`, `/speckit-specify`, `/speckit-plan`, `/speckit-tasks`, and later implement.
- **Kiro specs [54]** (docs, verified): every spec is three files, `requirements.md` (user stories with acceptance criteria), `design.md` ("technical architecture, sequence diagrams, and implementation considerations"), `tasks.md`; a three-phase workflow.
- **Spec Kit Agents (Taghavi et al., 2026) [55]** diagnoses the failure of plain SDD: agents "remain 'context blind' in large, evolving repositories, leading to hallucinated APIs and architectural violations," and fixes it with "read-only probing hooks [that] ground each stage (Specify, Plan, Tasks, Implement) in repository evidence, while validation hooks check intermediate artifacts against the environment." Gains were modest but significant (+0.15 on a 1–5 judge score over 128 runs; 58.2% Pass@1 on SWE-bench Lite) *(abstract)*.
- **Structured Spec-Driven Engineering (Feng et al., 2026) [56]** argues that "structured specifications as LLM inputs make high-quality, repository-level code generation a tangible goal, while at the same time offering superior verifiability" *(abstract)*.
- **Spec-driven test generation at Google (Tufano et al., 2026) [57]**: having the agent first write pre/post-conditions and undefined behaviour as a semi-formal spec improved bug detection by 9.8 percentage points (p = 0.035) and branch coverage by 2.5 points on production bugs *(abstract)*.
- **TDD for code generation (Mathews and Nagappan, 2024) [58]** and **AlphaCodium (Ridnik et al., 2024) [59]**: tests-as-spec and flow engineering, respectively, showing that giving the model something checkable to aim at improves results even at function scale.
- **Strata** is the visual, versioned instance of the same move.

The common finding is that the spec helps *when it is checkable against the repository*, and that the failure mode of specs that are not grounded is exactly "hallucinated APIs and architectural violations" [55]. Natural-language `design.md` files and D2 diagrams are not checkable against code; they rely on the agent to keep them honest.

### 6.1 Why the Rust skeleton closes the loop

A Rust *skeleton* is a crate (or module) in which every public item has its real signature, attributes, docs and trait impl headers, and every body is `todo!()` (or `unimplemented!()`). It has four properties no other spec medium has at once:

1. **It compiles.** `cargo check` is its validator; type errors, missing trait bounds, coherence violations and visibility mistakes are caught *in the spec*, before any implementation exists. This is compile-before-commit with the real compiler as the gate.
2. **It is the extracted model.** rustdoc JSON [60] on the skeleton produces the same item/impl/signature graph that rustdoc on the finished crate produces. The intended model and the source model are the same data type, so the reflexion map is `id`.
3. **It is diffable with existing tools.** cargo-semver-checks [61] and cargo-public-api [75] already diff public APIs between two versions; "implementation drifted from skeleton" is a semver-style diff between skeleton and crate.
4. **It is parallelisable.** Once contracts compile, `n` agents or engineers implement bodies against them concurrently; the graph shows at once when an implementation needs a contract change, which is the Parnas/Kernel-headers workflow at agent speed.

What Strata contributes to this is the *workflow discipline*: skeleton versions are immutable, each has a parent, the agent must `validate` before `commit`, concurrent edits are serialised by a version handle, and projections are budgeted (8–15 "nodes" becomes "one module's public surface, ≤ N tokens"). What Clarity contributes is the evidence and query grammar for reading the extracted side.

### 6.2 What the reflexion computation yields, concretely

For each public item in the skeleton: *converged* (signature identical in implementation), *diverged* (signature changed — classify with semver-checks lints as major/minor), *absent* (in skeleton, not implemented), *unplanned* (in implementation, not in skeleton). For each edge in the skeleton's `SIGNATURE_USES`/`IMPLEMENTS` graph, the same. Plus architecture rules as graph predicates ("no edge from `drivers::*` to `fs::internal::*`", "no `std` reachability from `no_std` crates", "every `pub unsafe fn` has a `# Safety` doc and a safety-tag annotation [76]").

This is the PR Change Witness from `docs-lossfunk/03`, derived rather than designed: a PR is a version transition, and the witness is the reflexion delta.

---

## 7. Why Rust, with references this time

The earlier note argued from language semantics; here is the external evidence.

- **The compiler already emits the public-interface graph.** RFC 2963, rustdoc JSON [60], gives items, visibility, re-exports, impls and generics as structured data. cargo-semver-checks [61] queries it through Trustfall [77] and is in the CI of major crates; the README notes the format "is unstable and changes often," which is an engineering cost, not a conceptual one.
- **Interface breakage is common and measurable.** Gruevski and Nowak's study of the top 1,000 crates [78] found "around 1 in 31 releases had at least one semver violation … 464 releases (3.22%)" and "172 crates (17.2%) had at least one release with a semver violation." Nowak's thesis [79] describes the tool. The public-interface graph is therefore not an academic object in Rust; it is already the unit of ecosystem-wide breakage.
- **Call-level dependency networks were built for all of crates.io.** Präzi (Hejderup et al., 2021) [80] "constructs a dependency network at the more fine-grained function-level, instead of at the manifest level" for Rust specifically, showing the cross-crate call graph is computable at ecosystem scale (via LLVM/MIR, requiring builds).
- **Unsafe has a graph structure and a vocabulary.** Astrauskas et al. (2020) [81] and Evans et al. (2020) [82] measured how unsafe is used; Cui et al. (2023) [83] defined 19 safety properties over 416 std unsafe APIs; Rao et al. (2024) [84] proposed an **"unsafety isolation graph"** with four isolation types and nine structural patterns that split it into audit units; Rao et al. (2025) [76] introduced safety tags and a linter that "enforces structural consistency between local safety annotations and callee requirements," applicable to 96.1% of std's public unsafe APIs. Rudra (Bae et al., 2021) [85] found hundreds of memory-safety bugs by pattern-matching unsafe usage at ecosystem scale. RustBelt (Jung et al., 2017) [86] is the semantic foundation that makes "safe encapsulation of unsafe" a precise claim. Together these say the "systems-specific edges" idea from the 10M-LOC note — unsafe boundaries, safety obligations flowing from callee to caller — already has a research literature and a graph formulation; it is the most defensible differentiator over generic code-graph tools.
- **Rust compiler-error repair is a studied loop.** Deligiannis et al. [69] (RustAssistant) and the C-to-Rust line [71, 72] show `rustc` diagnostics are a usable feedback channel for LLMs, which is what the compile-before-commit gate relies on.

The second-best language remains Go; the specific Rust advantages that no other language shares are coherence-bounded `IMPLEMENTS` edges and a compiler-emitted public-API dataset. Everything else is a matter of degree.

---

## 8. Evidence, provenance and immutable snapshots

Three independent sources say the same thing about trust:

- **Clarity** attaches `relationship`, `confidence`, reference and declaration locations to each edge and lets `--explain` show them.
- **RIG** [29] describes itself as "deterministic, evidence backed" and treats that as the property that lets agents consider it authoritative.
- **SCIP / LSIF / Kythe** [87, 88] — the industrial cross-reference formats — store per-occurrence roles and symbol identities, not bare adjacency; Kythe's whole design is a schema of *facts* with provenance.

And one human-factors source explains why the *diff* form matters: Bacchelli and Bird's study of code review at Microsoft [89] found that *understanding* the change and its context is the dominant challenge of review, well ahead of finding defects. A reviewer (human or agent) shown "this PR changed these 3 public contracts, these 11 downstream items are affected, these 2 trait impls are now missing" is being given the thing the study says they lack.

Design consequences, all already in the earlier decision list but now with citations attached:

1. Edge record = Clarity's evidence schema + `cfg` predicate + provenance (`rustdoc` / `rust-analyzer` / `parser` / `model@rev`) + resolution status (`resolved | ambiguous | external | unresolved`). Model-sourced edges never collapse into compiler-sourced ones.
2. Graph versions are immutable, content-addressed, with parent pointers (Strata's `version` table shape). `diff(v1, v2)` is a first-class query; cargo-semver-checks lints run on the public subset of the diff.
3. Identity is `crate@ver::module::path::Item` with spans as attributes, so diffs and training labels line up across edits (SCIP symbol strings are the precedent for stable, path-shaped identifiers [87]).

---

## 9. The agent surface: MCP, tool count and tool documentation

- **MCP** [90] is JSON-RPC 2.0 with resources, prompts and tools; Strata, Codebase-Memory [28] and most 2025–2026 code-graph tools expose themselves this way. There is no reason to invent transport.
- **Tool documentation is the interface.** Hsieh et al. (2023) [66] showed documentation alone enables zero-shot tool use; ToolLLM [91] and Gorilla [92] showed API-grounded models hallucinate less when the API surface is well described; Trilcke et al. (2025) [93] coined "docstring engineering" for MCP tools after observing that tool-selection reliability depended on it. SWE-agent [19] showed fewer, better-shaped commands beat many raw ones.
- **Over-privilege is a real audit target** (Huang et al., 2026 [94]); a graph server should be read-only by default with a separate, explicitly enabled write surface for skeleton authoring, which is also what Strata's loopback-only design implies.

Proposed surface (eight tools, each documented with examples; the Clarity grammar underneath):

| Tool | Snapshot × Anchor × Lens × Render |
|---|---|
| `find_items(query)` | lexical front door → stable ids (LARGER's anchor) |
| `describe_item(id, depth)` | item: signature, docs, public-to-public edges, `via` lists |
| `surface(scope, budget_tokens)` | module/crate skeleton, ranked and budgeted (Aider/RepoAtlas) |
| `impact(id, direction, depth, filters)` | may-impact closure with evidence per hop (CodePlan) |
| `implementors(trait_id)` / `path(a, b)` | coherence-bounded enumeration; shortest evidence path |
| `diff(v1, v2, lens)` | reflexion delta + semver classification |
| `validate_fragment(fragment)` | compile-before-commit for model/agent-authored edges or skeleton items; structured diagnostics |
| `commit_version(parent_handle, fragment)` | optimistic concurrency, immutable version |

Plus a raw-query escape hatch (CodexGraph's lesson) for tasks nobody anticipated.

---

## 10. Small models: what is realistic

- Distillation is standard (Hinton et al. 2015 [95]; Distilling Step-by-Step [96]); small code models are strong when data is curated (Textbooks Are All You Need [97]; Qwen2.5-Coder [98]; SmolLM2 [99]).
- The strongest published evidence *for* a small local model in this role is MGD [38]: a 1.1B model plus a static oracle beat a much larger model on compilation rate. The oracle was external. The strongest evidence *against* the model building the graph unaided is Venkatesh [33].
- CGM [26] shows a credible alternative use of a small model's capacity: read a compiler-built graph through an adapter and plan edits.

Realistic plan: (1) ship the compiler-derived graph with no model; (2) ship inference-time use of any local model as a *resolver under the validator* on broken files, measured against rust-analyzer on the same snapshots; (3) only then train, with validator diagnostics as reward, and only if step 2 shows a regime where the model wins. The 4 GB / sub-100 MB budget discussion in `docs-lossfunk/01` stays, but it gates step 3, not steps 1–2.

---

## 11. Synthesis: the architecture, in one picture

```
                 intended model                      extracted model
            ┌─────────────────────┐             ┌──────────────────────────┐
  agent ───►│ Rust skeleton vN    │  identity   │ item-level graph vN      │◄─── rustdoc JSON /
  (MCP)     │ (pub sigs, todo!()) │◄──────────► │ items, impls, sig-uses,  │     rust-analyzer /
            │ compiles via cargo  │  = `id`     │ calls, unsafe edges,     │     per-file parser
            └─────────┬───────────┘             │ evidence + cfg + prov.   │◄─── model resolver
                      │                         └─────────────┬────────────┘     (under validator)
                      │   validate_fragment / commit_version   │
                      ▼                                        ▼
            ┌──────────────────────────────────────────────────────────────┐
            │ immutable, content-addressed versions; diff(vA, vB)          │
            │ reflexion delta: converged / diverged / absent / unplanned   │
            │ semver lints; architecture-rule predicates; unsafe audit     │
            └──────────────────────────────┬───────────────────────────────┘
                                           ▼
            budgeted projections: workspace → crate → module skeleton → item → body
            (ranked, refreshable, token-capped; CALLS collapsed through private `via`)
```

**Build order (what the evidence supports):**

1. Item-level extraction from rustdoc JSON + rust-analyzer for crates that build; per-file parser with host-side linking as the fallback; Clarity's Rust adapter and `Codebase-Memory` as baselines to beat on answer quality per token.
2. Evidence-bearing edge store, stable identities, immutable versions, `diff`.
3. The eight-tool MCP surface with budgeted projections; measure tokens-per-resolved-task against file exploration (the Codebase-Memory protocol [28]).
4. Skeleton authoring + `validate_fragment` + reflexion delta (the Strata loop over Rust).
5. Unsafe/FFI/`Send`/`Sync` edges using the unsafety-isolation-graph and safety-tag formulations [84, 76].
6. Model as resolver under the validator, only if step 1's fallback is measurably the bottleneck.

**Research questions, each with a falsifiable prediction:**

| # | Question | Prediction | Metric |
|---|---|---|---|
| RQ1 | Does an item-level Rust graph beat a tree-sitter file-level graph for agent Q&A? | Closes most of Codebase-Memory's 9-point gap to file exploration at ≤ 2× its tokens | answer quality, tokens, tool calls on ≥ 30 Rust repos |
| RQ2 | Does skeleton-first + reflexion delta reduce agent contract violations? | Fewer "unplanned public item" and "signature diverged" events per merged PR vs prompt-only SDD | reflexion delta counts; compile-on-first-try rate |
| RQ3 | On broken snapshots, does a small resolver under the validator beat rust-analyzer on anything? | Wins only on macro-heavy files and on latency at ≥ 1M LOC; loses on precision elsewhere | per-relation P/R vs rustdoc ground truth; p95 latency; RSS |
| RQ4 | Do unsafe-obligation edges change agent behaviour? | Agents given callee safety obligations write `// SAFETY:` comments that pass safety-tool-style checks more often | safety-tag consistency rate |
| RQ5 | Is a budgeted, refreshed projection better than a static one? | RepoAtlas-style select–project–refresh beats one-shot module skeleton on long tasks | task success vs horizon length |
| RQ6 | Does the validator-as-reward improve a resolver? | Precision up, abstention calibrated; confident-wrong rate down by half vs SFT-only | calibration curves |
| RQ7 | Does `diff` as a PR witness help reviewers? | Review time down, understanding-related comments down (Bacchelli & Bird categories) | review study |

---

## 12. The case against (what the sceptic should keep saying)

1. **"Long context will make all of this unnecessary."** Prometheus [32] and Lost in the Middle [68] say otherwise today; but the margin shrinks every year. Mitigation: the value that survives long context is *determinism and evidence* (RIG), *diff/witness* (review), and *spec gating* (SDD). Build those, not the retrieval trick.
2. **"The compiler already does this; rust-analyzer already does this."** Largely true for crates that build. The honest product for those crates is a thin, evidence-bearing, versioned, MCP-exposed layer over rustdoc JSON and rust-analyzer, and that is still worth building because nobody ships it with the Strata loop on top. The model is only for the regime where those tools cannot run or cannot keep up, and that regime must be measured, not assumed.
3. **"File-level is enough for most agent tasks."** Codebase-Memory's 83% vs 92% says file-level graphs *lose* to plain exploration on quality while winning on tokens. Item-level resolution is the lever that can win on both; if it does not, Clarity has already won this space and the right move is to contribute a better Rust adapter to it.
4. **"Spec-first is a fad; engineers won't maintain skeletons."** Agents, not engineers, maintain them, and `cargo check` enforces them. The risk is real for the *human* workflow and should be tested with RQ2 before any UI is built.
5. **"Macros break everything."** They break the parser path, not the rustdoc path (rustdoc sees expanded items). The parser/model path must mark macro-generated regions as `unresolved` rather than guess; this is the abstention requirement again.
6. **"You are reinventing reflexion models / DSMs / Kythe."** Yes, deliberately, for one language, with a compiler-grade identity, and for a consumer that did not exist when those were built. That is the whole argument.

---

## References

Format: authors, year, title, venue or arXiv id, URL. "Verified" means resolved by API/DOI on 2026-11-05.

### Classic foundations (Crossref-verified)

1. Parnas, D. L. (1972). On the criteria to be used in decomposing systems into modules. *Communications of the ACM*. DOI 10.1145/361598.361623.
2. Steward, D. V. (1981). The design structure system: A method for managing the design of complex systems. *IEEE Transactions on Engineering Management*. DOI 10.1109/TEM.1981.6448589.
3. MacCormack, A., Rusnak, J., Baldwin, C. Y. (2006). Exploring the structure of complex software designs: An empirical study of open source and proprietary code. *Management Science*. DOI 10.1287/mnsc.1060.0552.
4. Néron, P., Tolmach, A., Visser, E., Wachsmuth, G. (2015). A theory of name resolution. *ESOP, LNCS*. DOI 10.1007/978-3-662-46669-8_9.
5. Murphy, G. C., Notkin, D., Sullivan, K. (1995). Software reflexion models: Bridging the gap between source and high-level models. *ACM SIGSOFT FSE*. DOI 10.1145/222124.222136.
6. Ferrante, J., Ottenstein, K. J., Warren, J. D. (1987). The program dependence graph and its use in optimization. *ACM TOPLAS*. DOI 10.1145/24039.24041.
7. Horwitz, S., Reps, T., Binkley, D. (1990). Interprocedural slicing using dependence graphs. *ACM TOPLAS*. DOI 10.1145/77606.77608.
8. Dean, J., Grove, D., Chambers, C. (1995). Optimization of object-oriented programs using static class hierarchy analysis. *ECOOP, LNCS*. DOI 10.1007/3-540-49538-X_5.
9. Bacon, D. F., Sweeney, P. F. (1996). Fast static analysis of C++ virtual function calls. *OOPSLA*. DOI 10.1145/236337.236371.
10. Grove, D., Chambers, C. (2001). A framework for call graph construction algorithms. *ACM TOPLAS*. DOI 10.1145/506315.506316.
11. Perry, D. E., Wolf, A. L. (1992). Foundations for the study of software architecture. *ACM SIGSOFT SEN*. DOI 10.1145/141874.141884.
12. Murphy, G. C., Notkin, D., Sullivan, K. J. (2001). Software reflexion models: Bridging the gap between design and implementation. *IEEE TSE*. DOI 10.1109/32.917525.
13. Yamaguchi, F., Golde, N., Arp, D., Rieck, K. (2014). Modeling and discovering vulnerabilities with code property graphs. *IEEE S&P*. DOI 10.1109/SP.2014.44.

### Repository-level agents and code graphs (arXiv-verified)

14. Zhang, F. et al. (2023). RepoCoder: Repository-level code completion through iterative retrieval and generation. arXiv:2303.12570. https://arxiv.org/abs/2303.12570
15. Liu, T. et al. (2023). RepoBench: Benchmarking repository-level code auto-completion systems. arXiv:2306.03091. https://arxiv.org/abs/2306.03091
16. Bogomolov, E. et al. (2024). Long Code Arena: a set of benchmarks for long-context code models. arXiv:2406.11612. https://arxiv.org/abs/2406.11612
17. Wang, Z. Z. et al. (2024). CodeRAG-Bench: Can retrieval augment code generation? arXiv:2406.14497. https://arxiv.org/abs/2406.14497
18. Jimenez, C. E. et al. (2023). SWE-bench: Can language models resolve real-world GitHub issues? arXiv:2310.06770. https://arxiv.org/abs/2310.06770
19. Yang, J. et al. (2024). SWE-agent: Agent-computer interfaces enable automated software engineering. arXiv:2405.15793. https://arxiv.org/abs/2405.15793
20. Bairi, R. et al. (2023). CodePlan: Repository-level coding using LLMs and planning. arXiv:2309.12499 (FSE 2024). https://arxiv.org/abs/2309.12499 ; code/data https://github.com/microsoft/codeplan
21. Liu, X. et al. (2024). CodexGraph: Bridging large language models and code repositories via code graph databases. arXiv:2408.03910. https://arxiv.org/abs/2408.03910
22. Xia, C. S. et al. (2024). Agentless: Demystifying LLM-based software engineering agents. arXiv:2407.01489. https://arxiv.org/abs/2407.01489
23. Ouyang, S. et al. (2024). RepoGraph: Enhancing AI software engineering with repository-level code graph. arXiv:2410.14684. https://arxiv.org/abs/2410.14684
24. Liu, W. et al. (2024). GraphCoder: Enhancing repository-level code completion via code context graph-based retrieval and language model. arXiv:2406.07003. https://arxiv.org/abs/2406.07003
25. Cheng, W. et al. (2024). Dataflow-guided retrieval augmentation for repository-level code completion (DraCo). arXiv:2405.19782. https://arxiv.org/abs/2405.19782
26. Tao, H. et al. (2025). Code Graph Model (CGM): A graph-integrated large language model for repository-level software engineering tasks. arXiv:2505.16901. https://arxiv.org/abs/2505.16901
27. Zhang, Y. et al. (2026). RepoAtlas: Guiding coding agents via evolving multimodal repository views. arXiv:2609.16936. https://arxiv.org/abs/2609.16936
27a. Hu, Y. et al. (2026). LARGER: Lexically anchored repository graph exploration and retrieval. arXiv:2605.16352. https://arxiv.org/abs/2605.16352
28. Vogel, S. et al. (2026). Codebase-Memory: Tree-sitter-based knowledge graphs for LLM code exploration via MCP. arXiv:2603.27277. https://arxiv.org/abs/2603.27277
29. Cherny-Shahar, A. et al. (2026). Repository Intelligence Graph: Deterministic architectural map for LLM code assistants. arXiv:2601.10112. https://arxiv.org/abs/2601.10112
30. Wang, Y. et al. (2025). GRACE: Graph-guided repository-aware code completion through hierarchical code fusion. arXiv:2509.05980. https://arxiv.org/abs/2509.05980
31. Aider. Repository map (documentation; graph-ranked, `--map-tokens` default 1k). https://aider.chat/docs/repomap.html
32. Pan, Z. et al. (2025). Prometheus: Towards long-horizon codebase navigation for repository-level problem solving. arXiv:2507.19942. https://arxiv.org/abs/2507.19942

### LLMs and static analysis; constrained decoding (arXiv-verified)

33. Venkatesh, A. P. S. et al. (2024). An empirical study of large language models for type and call graph analysis in Python and JavaScript. arXiv:2410.00603. https://arxiv.org/abs/2410.00603
34. Su, C. et al. (2025). Do code LLMs do static analysis? arXiv:2505.12118. https://arxiv.org/abs/2505.12118
35. Li, H. et al. (2023). The Hitchhiker's guide to program analysis: A journey with large language models (LLift). arXiv:2308.00245. https://arxiv.org/abs/2308.00245
36. Li, Z. et al. (2024). IRIS: LLM-assisted static analysis for detecting security vulnerabilities. arXiv:2405.17238. https://arxiv.org/abs/2405.17238
37. Wang, C. et al. (2024). LLMDFA: Analyzing dataflow in code with large language models. arXiv:2402.10754. https://arxiv.org/abs/2402.10754
38. Agrawal, L. A. et al. (2023). Guiding language models of code with global context using monitors (Monitor-Guided Decoding). arXiv:2306.10763. https://arxiv.org/abs/2306.10763 ; library https://github.com/microsoft/multilspy
39. Mündler, N. et al. (2025). Type-constrained code generation with language models. arXiv:2504.09246. https://arxiv.org/abs/2504.09246
40. Chen, X. et al. (2025). Towards mitigating API hallucination in code generated by LLMs with hierarchical dependency aware (MARIN). arXiv:2505.05057. https://arxiv.org/abs/2505.05057
41. Scholak, T., Schucher, N., Bahdanau, D. (2021). PICARD: Parsing incrementally for constrained auto-regressive decoding from language models. arXiv:2109.05093. https://arxiv.org/abs/2109.05093
42. Poesia, G. et al. (2022). Synchromesh: Reliable code generation from pre-trained language models. arXiv:2201.11227. https://arxiv.org/abs/2201.11227
43. Geng, S. et al. (2023). Grammar-constrained decoding for structured NLP tasks without finetuning. arXiv:2305.13971. https://arxiv.org/abs/2305.13971
44. Willard, B. T., Louf, R. (2023). Efficient guided generation for large language models (Outlines). arXiv:2307.09702. https://arxiv.org/abs/2307.09702
45. Ugare, S. et al. (2024). SynCode: LLM generation with grammar augmentation. arXiv:2403.01632. https://arxiv.org/abs/2403.01632
46. Dong, Y. et al. (2024). XGrammar: Flexible and efficient structured generation engine for large language models. arXiv:2411.15100. https://arxiv.org/abs/2411.15100

### Verifier / compiler feedback in the loop (arXiv-verified)

47. Le, H. et al. (2022). CodeRL: Mastering code generation through pretrained models and deep reinforcement learning. arXiv:2207.01780. https://arxiv.org/abs/2207.01780
48. Liu, J. et al. (2023). RLTF: Reinforcement learning from unit test feedback. arXiv:2307.04349. https://arxiv.org/abs/2307.04349
49. Jain, A. et al. (2023). Coarse-tuning models of code with reinforcement learning feedback (RLCF). arXiv:2305.18341. https://arxiv.org/abs/2305.18341
50. Dou, S. et al. (2024). StepCoder: Improve code generation with reinforcement learning from compiler feedback. arXiv:2402.01391. https://arxiv.org/abs/2402.01391
51. Gehring, J. et al. (2024). RLEF: Grounding code LLMs in execution feedback with reinforcement learning. arXiv:2410.02089. https://arxiv.org/abs/2410.02089
52. Sun, C. et al. (2023). Clover: Closed-loop verifiable code generation. arXiv:2310.17807. https://arxiv.org/abs/2310.17807

### Spec-driven development for agents (web / arXiv-verified)

53. GitHub. Spec Kit (README). https://github.com/github/spec-kit
54. Kiro. Specs (documentation: `requirements.md`, `design.md`, `tasks.md`). https://kiro.dev/docs/specs/
55. Taghavi, M. et al. (2026). Spec Kit Agents: Context-grounded agentic workflows. arXiv:2604.05278. https://arxiv.org/abs/2604.05278
56. Feng, Y. et al. (2026). LLM-assisted repository-level generation with structured spec-driven engineering. arXiv:2605.02455. https://arxiv.org/abs/2605.02455
57. Tufano, M. et al. (2026). Grounding AI agents in contracts: An empirical evaluation of spec-driven test generation. arXiv:2608.17177. https://arxiv.org/abs/2608.17177
58. Mathews, N. S., Nagappan, M. (2024). Test-driven development for code generation. arXiv:2402.13521. https://arxiv.org/abs/2402.13521
59. Ridnik, T., Kredo, D., Friedman, I. (2024). Code generation with AlphaCodium: From prompt engineering to flow engineering. arXiv:2401.08500. https://arxiv.org/abs/2401.08500

### Rust-specific tooling and research (web / arXiv / Crossref-verified)

60. Rust RFC 2963: rustdoc JSON. https://rust-lang.github.io/rfcs/2963-rustdoc-json.html
61. Gruevski, P. et al. cargo-semver-checks (README; uses rustdoc JSON; format "is unstable and changes often"). https://github.com/obi1kenobi/cargo-semver-checks
62. GitHub. Introducing stack graphs (blog). https://github.blog/open-source/introducing-stack-graphs/
63. GitHub. stack-graphs (repository). https://github.com/github/stack-graphs
64. Eades, P., Lin, X., Smyth, W. F. (1993). A fast and effective heuristic for the feedback arc set problem. *Information Processing Letters*. DOI 10.1016/0020-0190(93)90079-O.
65. LegacyCodeHQ. clarity-cli (README; Apache-2.0). https://github.com/LegacyCodeHQ/clarity-cli
66. Hsieh, C.-Y. et al. (2023). Tool documentation enables zero-shot tool-usage with large language models. arXiv:2308.00675. https://arxiv.org/abs/2308.00675
67. Legacy Code Labs. Strata product page. https://legacycode.com/tools/strata ; download https://legacycode.com/download/strata
68. Liu, N. F. et al. (2023). Lost in the middle: How language models use long contexts. arXiv:2307.03172. https://arxiv.org/abs/2307.03172
69. Deligiannis, P. et al. (2023). Fixing Rust compilation errors using LLMs (RustAssistant). arXiv:2308.05177. https://arxiv.org/abs/2308.05177
70. Yang, C. et al. (2024). AutoVerus: Automated proof generation for Rust code. arXiv:2409.13082. https://arxiv.org/abs/2409.13082
71. Shetty, M. et al. (2024). Syzygy: Dual code-test C to (safe) Rust translation using LLMs and dynamic analysis. arXiv:2412.14234. https://arxiv.org/abs/2412.14234
72. Shiraishi, T. et al. (2024). SmartC2Rust: Iterative, feedback-driven C-to-Rust translation via large language models. arXiv:2409.10506. https://arxiv.org/abs/2409.10506
73. Chen, X. et al. (2023). Teaching large language models to self-debug. arXiv:2304.05128. https://arxiv.org/abs/2304.05128
74. Shinn, N. et al. (2023). Reflexion: Language agents with verbal reinforcement learning. arXiv:2303.11366. https://arxiv.org/abs/2303.11366
75. cargo-public-api: List and diff the public API of Rust library crates. https://github.com/cargo-public-api/cargo-public-api
76. Rao, Z. et al. (2025). Annotating and auditing the safety properties of unsafe Rust (safety tags, safety-tool). arXiv:2504.21312. https://arxiv.org/abs/2504.21312
77. Gruevski, P. Trustfall query engine. https://github.com/obi1kenobi/trustfall
78. Nowak, T., Gruevski, P. (2023). Semver violations are common, better tooling is the answer (blog; top-1000 crates study: 3.22% of releases, 17.2% of crates). https://predr.ag/blog/semver-violations-are-common-better-tooling-is-the-answer/
79. Nowak, T. (2023). Accelerating package expansion in Rust through development of a semantic versioning tool. arXiv:2308.14623. https://arxiv.org/abs/2308.14623
80. Hejderup, J., Beller, M., Triantafyllou, K., Gousios, G. (2021). Präzi: From package-based to call-based dependency networks. arXiv:2101.09563. https://arxiv.org/abs/2101.09563 ; https://github.com/praezi/rust
81. Astrauskas, V. et al. (2020). How do programmers use unsafe Rust? *PACMPL (OOPSLA)*. DOI 10.1145/3428204.
82. Evans, A. N., Campbell, B., Soffa, M. L. (2020). Is Rust used safely by software developers? *ICSE*. DOI 10.1145/3377811.3380413.
83. Cui, M. et al. (2023). Is unsafe an Achilles' heel? A comprehensive study of safety requirements in unsafe Rust programming. arXiv:2308.04785. https://arxiv.org/abs/2308.04785
84. Rao, Z. et al. (2024). Characterizing unsafe code encapsulation in real-world Rust systems (unsafety isolation graph). arXiv:2406.07936. https://arxiv.org/abs/2406.07936
85. Bae, Y. et al. (2021). Rudra: Finding memory safety bugs in Rust at the ecosystem scale. *SOSP*. DOI 10.1145/3477132.3483570.
86. Jung, R., Jourdan, J.-H., Krebbers, R., Dreyer, D. (2017). RustBelt: Securing the foundations of the Rust programming language. *PACMPL (POPL 2018)*. DOI 10.1145/3158154.

### Indexing formats, review, MCP, small models (web / arXiv / Crossref-verified)

87. Sourcegraph. SCIP Code Intelligence Protocol. https://github.com/sourcegraph/scip
88. Google. Kythe. https://github.com/kythe/kythe
89. Bacchelli, A., Bird, C. (2013). Expectations, outcomes, and challenges of modern code review. *ICSE*. DOI 10.1109/ICSE.2013.6606617.
90. Model Context Protocol specification (2025-06-18). https://modelcontextprotocol.io/specification/2025-06-18
91. Qin, Y. et al. (2023). ToolLLM: Facilitating large language models to master 16000+ real-world APIs. arXiv:2307.16789. https://arxiv.org/abs/2307.16789
92. Patil, S. G. et al. (2023). Gorilla: Large language model connected with massive APIs. arXiv:2305.15334. https://arxiv.org/abs/2305.15334
93. Trilcke, P. et al. (2025). Agentic DraCor and the art of docstring engineering: Evaluating MCP-empowered LLM usage of the DraCor API. arXiv:2508.13774. https://arxiv.org/abs/2508.13774
94. Huang, Y. et al. (2026). Auditing MCP servers for over-privileged tool capabilities. arXiv:2603.21641. https://arxiv.org/abs/2603.21641
95. Hinton, G., Vinyals, O., Dean, J. (2015). Distilling the knowledge in a neural network. arXiv:1503.02531. https://arxiv.org/abs/1503.02531
96. Hsieh, C.-Y. et al. (2023). Distilling step-by-step! Outperforming larger language models with less training data and smaller model sizes. arXiv:2305.02301. https://arxiv.org/abs/2305.02301
97. Gunasekar, S. et al. (2023). Textbooks are all you need. arXiv:2306.11644. https://arxiv.org/abs/2306.11644
98. Hui, B. et al. (2024). Qwen2.5-Coder technical report. arXiv:2409.12186. https://arxiv.org/abs/2409.12186
99. Allal, L. B. et al. (2025). SmolLM2: When smol goes big — data-centric training of a small language model. arXiv:2502.02737. https://arxiv.org/abs/2502.02737

### Additional context consulted

100. Schmid, L. et al. (2025). Software architecture meets LLMs: A systematic literature review (notes conformance checking as an under-studied area). arXiv:2505.16697. https://arxiv.org/abs/2505.16697
101. Li, R. et al. (2023/2025). Towards automated identification of violation symptoms of architecture erosion. arXiv:2306.08616. https://arxiv.org/abs/2306.08616
102. Liu, Y. et al. (2026). Architecture-aware multi-design generation for repository-level feature addition (RAIM). arXiv:2603.01814. https://arxiv.org/abs/2603.01814
103. Jain, N. et al. (2024). On mitigating code LLM hallucinations with API documentation (CloudAPIBench; GPT-4o 38.58% valid low-frequency API invocations). arXiv:2407.09726. https://arxiv.org/abs/2407.09726
104. Edge, D. et al. (2024). From local to global: A Graph RAG approach to query-focused summarization. arXiv:2404.16130. https://arxiv.org/abs/2404.16130
105. Wang, X. et al. (2024). Executable code actions elicit better LLM agents (CodeAct). arXiv:2402.01030. https://arxiv.org/abs/2402.01030
106. D2 declarative diagramming language (bundled in Strata as `Resources/bin/d2`). https://d2lang.com/
107. cargo-modules: Visualize/analyze a Rust crate's internal structure. https://github.com/regexident/cargo-modules

### Not verified in this pass (named from memory in earlier notes; check before citing)

Karp (1972) NP-completeness of feedback arc set; Baldwin & Clark, *Design Rules* (2000); Lattix / Structure101 DSM tools; RLS save-analysis removal date; Kythe's Rust indexer; Charon/Aeneas, MirChecker, Kani, Prusti details; Sourcegraph Cody and Cursor indexing internals.
