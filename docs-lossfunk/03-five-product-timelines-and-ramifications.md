# Five timelines for one Rust public-interface dependency graph

**Decision:** Build the graph, not a PR reviewer. Each timeline below starts with the same artifact: a versioned, source-linked map of the externally reachable contracts in a Rust workspace and the dependencies needed to reason about them. Each tests a different reason a systems programmer would actually keep that map. The sequence is illustrative, not a delivery estimate or evidence of demand.

The [README](../README.md) documents the starting point: an installed CLI can already create a graph from source and let agents query it. The [model experiment](01-small-model-interface-graph-research.md) asks whether a local, open-weight graph generator can produce a *better graph* on unseen Rust OSS. Launching a new viewer, replacing storage or attracting PR reviewers would not answer that question.

## Timeline A: The maintainer publishes the map

**Start.** A new contributor asks why a storage error type is visible in an ostensibly unrelated API. A Cargo dependency list narrows the search to crates; a text search yields every reference, public and private.

**First useful artifact.** Export the public graph at a pinned revision and feature set. The contributor follows an exported return type to its definition, then traverses the inverse relationship to the public signatures that expose it. Both ends link to source. Re-export aliases remain entry points to the same definition rather than duplicate contracts.

**Consequence.** The map can accompany a release independently of any code assistant. A maintainer can check it without installing the model that made it. If results have no configuration or return type uses from bodies as public API dependencies, maintainers cannot rely on the artifact. Test with blind downstream-dependency questions and score exact contract and evidence recovery.

## Timeline B: A specialist learns to construct the map

**Start.** Run a frozen Tree-sitter-based Parseltongue over a licensed, pinned corpus. Select the proposed top 50 OSS Rust libraries by a declared rule. Collect disagreements on re-exports, generic bounds, macro products, associated types and same-name methods.

**First useful artifact.** Generate teacher graphs with the application-specified Codex 5.6 xhigh recipe, recording the actual model/configuration used. Review a stratified sample independently, including declarations and dependencies *missing* from teacher output. Train an open-weight student to propose declarations and typed edges from source; test whether reading back a partial graph improves linking.

**Consequence.** The corpus and generation code can be valuable even if the student loses. On additional held-out repositories compare the frozen baseline, student and Rust-aware reference under matched inputs. If gains vanish outside training families, report that as failed generalization. If only the parser-assisted arm wins, describe it as a hybrid.

## Timeline C: A systems programmer follows an execution path

**Start.** Someone investigating an I/O subsystem asks, "What happens after this public read operation?" The important path may pass through a private adapter to a trait method. The caller does not care that a helper is private; they care whether the path exists.

**First useful artifact.** From the exported method, show a direct call when its target is known, a private witness when a path crosses implementation, and a trait target with possible implementations when dispatch is uncertain. Traverse backwards from the trait or adapter to callers. Never turn a list of implementations into a claim that all were executed.

**Consequence.** A compact graph can guide control-flow and data-flow investigations without pretending to be a runtime trace. A type in a return position is evidence about a boundary value, *not* proof that one particular allocation flows from a particular input. Measure accurate answers and justified abstentions, not merely path length.

## Timeline D: The coding harness uses the graph before opening files

**Start.** A harness on a fresh clone is asked which public contracts would be affected by a type change. It can already search names; what it lacks is a justified stopping point for exploration.

**First useful artifact.** Given a stable symbol, look up incoming signature references, return a bounded neighborhood with sources, then open only the evidence needed for the answer. In reverse, start from a source occurrence and recover the contract, canonical target and other exposing signatures. Keep lookup and traversal deterministic; don't run the model again to invent an answer already in the graph.

**Consequence.** A wrong confident edge can cost more tokens than a missing edge if it steers the harness into the wrong subsystem. Run paired tasks with the same harness and token budget. Count correct answers, false confident answers and *all* source and tool-result tokens, separately from the full graph/source token ratio. The application's roughly 5% size target does not by itself establish task utility.

## Timeline E: The graph stays usable at systems scale

**Start.** Rebuilding on every edit is unattractive in a large workspace. A comment insertion moves byte positions, a re-export edit changes who can name a type, and a feature switch changes which code exists. Treating all three identically either wastes work or leaves stale facts.

**First useful artifact.** Reconcile fresh evidence with durable symbol identities, then publish an atomic current revision. Refresh source locations on ordinary comment edits without churning unaffected contract identities. On re-export or feature changes, invalidate dependent exposure paths, including in unchanged files. Compare incremental output with a clean rebuild for the same configuration.

**Consequence.** A disk-backed graph with bounded CPU working sets may be useful on repositories much larger than the model's context. Prove that by reporting the largest *actually tested* workspace, cold/warm index times, peak inference-worker RAM and total process footprint. Less than 100 MB is the preferred inference target; an OS-scale corpus of tens of millions of lines is an ambition, not a benchmark result.

## What to do first

| Order | Deliverable | What would change the decision |
|---|---|---|
| 1 | One hand-audited cross-crate fixture with a re-export, signature reference, private witness, and ambiguous dispatch; canonical export and source evidence. | If two reviewers cannot agree what an edge means, fix the schema before training. |
| 2 | Reproducible baseline plus teacher-generation and adjudication recipe. | If a deterministic tool already solves a relation, retain it as a comparator and investigate a measured gap. |
| 3 | Student versus baseline on unseen systems workspaces; memory and token measurements. | A smaller graph that omits contracts or a small download that overruns inference RAM cannot claim the corresponding win. |
| 4 | Open weights on Hugging Face, licensed dataset and code, working Parseltongue integration. | If a component misses its target, publish the limitation and the useful artifacts rather than changing the success metric. |

Review, CI and refactoring can later consume this same export. None is a substitute for producing it. A positive result would mean another person can reproduce a graph on a new codebase and use it to answer a source-checkable question they could not answer as reliably with the existing tool.

## Historical discovery notes

To extend the earlier research, a bounded Hacker News Algolia pass searched story titles/content for `small model`, `code graph`, `Rust architecture`, and `Qwenputer`, filtering 2026-10-04 through 2026-11-03 as available at retrieval. The API returned 85, 34, 7, and 4 matches respectively; up to the latest 12 per query were retrieved in this pass. Overlap and irrelevant matches were not treated as separate evidence. This is discovery sampling, not an exhaustive survey or a popularity ranking.

| Discovery | Discussion date/link | Primary source | Lesson for Parseltongue, not a transfer claim |
|---|---|---|---|
| Vesalius | [2026-10-26 HN post](https://news.ycombinator.com/item?id=50250206) | [Project](https://github.com/haplollc/Vesalius) | Local graph distribution already exists. |
| Qwenputer | [2026-10-28 HN post](https://news.ycombinator.com/item?id=50277757) | [Project](https://github.com/divagr18/qwenputer) | Executable machinery inside weights is possible in a different setting; test native execution first. |
| Nouli | [2026-10-29 HN post](https://news.ycombinator.com/item?id=50288442) | [Project](https://github.com/nouli-ai/nouli) | A ledger connecting decisions and supporting facts is useful inspiration for assertion provenance. |
| Cerebellum | [2026-10-30 HN post](https://news.ycombinator.com/item?id=50309386) | [Project](https://github.com/cerebellum-labs/cerebellum) | Its reversible/irreversible action routing suggests separating graph investigation from consequential authorization. |
| bootgpt | [2026-10-31 HN post](https://news.ycombinator.com/item?id=50315782) | [Project](https://github.com/kerlenton/bootgpt) | Extremely small single-purpose inference can be engineered; its generated boot-log task says nothing about Rust graph accuracy. |
| Nomos | [2026-11-02 HN post](https://news.ycombinator.com/item?id=50337394) | [Project](https://github.com/mgantlett/nomos) | Protected checks and attributable receipts are relevant when the agent can edit the code being evaluated. |

These dates identify discussion events, not necessarily first project releases. Project descriptions above are author-reported. No security, accuracy, performance, or production-readiness claim was independently reproduced.

The useful cross-domain synthesis is **consequence-sensitive verification**: a proposed graph edge can be explored cheaply; a relationship used to block a release needs stronger independent evidence; neither a tiny model nor a clean decision journal grants merge authority.

## Further reading

Two project notes explain the difference between the artifact and its consumers: [Enriching ISG for LLM Code Reasoning](https://docs.google.com/document/d/1CKlPh-Sd9kuS_61gwql81C9JTPLMu2B69IqJ6drD6-s/edit) treats review as a use of the graph; [AI Codebase Navigation Tools](https://docs.google.com/document/d/1SgpSdCa3SEGhppECVQ1BpOwhXcTAaNsTqpqE17yU0fA/edit) examines the agent workflow. See the [complete Google Docs source register](02-learned-graph-compiler-thesis.md#google-docs-research-sources).

The links below collect the sources actually used for the analysis. The annotations state their role; they do not imply independent reproduction. The HN discovery links are collected in the preceding table. Older documents contain additional historical references that were not all reverified here.

| Reference | Use and limit |
|---|---|
| [Rust visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html) | Exposure semantics, restricted visibility, and associated-item rules. |
| [Rust conditional compilation](https://doc.rust-lang.org/reference/conditional-compilation.html) | Configuration-dependent source and analysis scope. |
| [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) | Build-time execution and generated-code boundary. |
| [rustdoc unstable features](https://doc.rust-lang.org/rustdoc/unstable-features.html) | JSON/toolchain compatibility must be pinned; potential reference data. |
| [cargo-public-api](https://github.com/cargo-public-api/cargo-public-api) | Existing public API inventory and comparison; product baseline. |
| [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) | Existing API checks; README describes rustdoc compatibility and query-driven lints. |
| [Cargo Bylaw](https://github.com/danielgerlag/cargo-bylaw) | Existing Rust architecture enforcement; not independently benchmarked here. |
| [rust-analyzer features](https://rust-analyzer.github.io/book/features.html) | Existing Rust navigation/refactoring baseline. |
| [rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html) | Incremental semantic-analysis and crate-context precedent. |
| [SCIP](https://github.com/scip-code/scip) | Symbol/occurrence interchange; not identical to call semantics. |
| [Vesalius](https://github.com/haplollc/Vesalius) | Competing local graph distribution. |
| [Empirical type/call-graph study, revision 2](https://arxiv.org/abs/2410.00603v2) | Contrary evidence from Python/JavaScript; not a Rust result. |
| [CG-Bench conference abstract](https://conf.researchr.org/details/icfp-splash-2025/lmpl-2025-papers/3/CG-Bench-Can-Language-Models-Assist-Call-Graph-Construction-in-the-Real-World-) | Small reviewed C/C++ call-target study; not full-graph evaluation. |
| [CodeAct](https://arxiv.org/abs/2402.01030) | Executable action-loop precedent. |
| [Recursive Language Models](https://arxiv.org/abs/2512.24601) | External context inspection; different scale/task. |
| [Tiny Recursive Models](https://arxiv.org/abs/2510.04871) | Structured recurrent refinement on puzzle tasks. |
| [Recursive-model quality-estimation transfer](https://arxiv.org/abs/2603.14593) | Counterexample to automatic recurrence transfer. |
| [CoSIL](https://arxiv.org/abs/2503.22424) | Iterative graph-guided issue localization, not tiny Rust construction. |
| [Qwen2.5-Coder-0.5B-Instruct](https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct) | Small code-model candidate and card metadata. |
| [Qwen3.5-0.8B](https://huggingface.co/Qwen/Qwen3.5-0.8B) | Task-specific fine-tuning candidate; multimodal packaging caveat. |
| [AMD-Llama-135m-code](https://huggingface.co/amd/AMD-Llama-135m-code) | Smaller code-model control and context limitation. |
| [Needle 3](https://huggingface.co/Cactus-Compute/needle3) | Tiny structured-action/extraction precedent; author-reported artifact sizes. |
| [Gutsy](https://github.com/kouhxp/gutsy) | CPU decision-model precedent, not declaration discovery evidence. |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | Native quantized runtime option; exact architecture support requires testing. |
| [MLX LM](https://github.com/ml-explore/mlx-lm) | Apple-silicon fine-tuning/inference option. |
| [Candle](https://github.com/huggingface/candle) | Rust ML runtime option, not required implementation language. |
| [Monty](https://github.com/pydantic/monty) | Constrained execution with explicit host access. |
| [Qwenputer](https://github.com/divagr18/qwenputer) | Compiled in-weight computation precedent; no claimed budget/task transfer. |
| [Nouli](https://github.com/nouli-ai/nouli) | Ledger-backed decision/fact provenance inspiration. |
| [Cerebellum](https://github.com/cerebellum-labs/cerebellum) | Consequence-sensitive routing analogy, not code-analysis validation. |
| [bootgpt](https://github.com/kerlenton/bootgpt) | Extreme single-purpose model packaging; unrelated semantic task. |
| [Nomos](https://github.com/mgantlett/nomos) | Protected-check/receipt inspiration; no security audit performed. |

Local sources:

- [Learned graph compiler thesis](02-learned-graph-compiler-thesis.md): primary technical hypothesis and representation.
- [Earlier small-model research](01-small-model-interface-graph-research.md): model options, baselines, dataset and success criteria.
- [Current-state PRD](../docs/research004/prd-v20260710.md): graph requirements and narrow HTTP capture scope.
- [Current-state architecture proposal](../docs/research004/Arch-v202607210.md): proposed capture/accounting system, not an implemented rewrite.
- [Visual trust and change evidence](../docs/research004/Sol-Insights-02.md): preceding product framing.
- [Public-interface graphs and source accounting](../docs/research004/Sol-Insights-03.md): earlier projection/identity thesis.
- [Query extractor](../crates/parseltongue-core/src/query_extractor.rs): current entity/edge extraction evidence.
- [Adjacency representation](../crates/parseltongue-core/src/graph_analysis/adjacency_list_graph_representation.rs): current native graph integration evidence.
