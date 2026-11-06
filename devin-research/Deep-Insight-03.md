# Deep Insight 03: Four futures for a tiny-model Rust CODEMAP companion

Date: 2026-11-06. Based on [Deep Insight 01](Deep-Insight-01.md), committed as `c52bb41bb` before this scenario exercise.

Method: `timeline-traverser`. These are conditional product scenarios, not forecasts, implementation commitments, or measured market demand. The four paths are alternative starting points; they are not four simultaneous products to build.

## Decision Frame

### The actual fork

The question is not simply "model or CLI?" Parseltongue can combine both. The decision is **which repeated user job should make the combined product indispensable first**:

1. Navigating unfamiliar Rust code every day.
2. Designing a cross-cutting change before implementation.
3. Reviewing whether a change matches its stated intent.
4. Providing a reusable graph-generation component to other agent builders.

All four use the same two visible levels: file dependencies and boundary-visible interface dependencies, with mappings between them. Current observations, uncertain hypotheses, and proposed future designs remain distinct. None promises complete runtime control flow from interface signatures alone.

### Goals, constraints, and success

- Primary technical audience: people and agents working on substantial Rust systems, such as storage engines, networking libraries, runtimes, compiler infrastructure, and multi-crate services. These are candidate segments, not validated customers.
- Product goal: make it easier to understand connections, propose changes to those connections, and verify resulting code against the proposal.
- Model goal: a specialized local model that contributes to graph construction or graph-based work. A general coding chatbot is outside scope.
- First memory gate: measure a declared CPU inference configuration against 100,000,000 bytes peak resident memory. Report whole-application and resolver memory separately. The existing experiments do not establish a whole-large-repository analyzer under that limit.
- If 100 MB becomes the budget for the entire application, use bounded source batches and disk-backed graph access and test that claim directly. Do not silently exempt the host process, page cache, or concurrent workers.
- Correctness: expose evidence, unresolved cases, feature configuration, stale snapshots, and truncated results. An empty neighborhood is not proof of independence.
- Human authority: proposed architecture does not become observed reality or authorize edits merely because an LLM emitted it.
- Commercial success: repeated voluntary use on real work, followed by willingness to support or pay for a specific workflow. Stars, downloads, benchmark scores, and enthusiasm are intermediate signals, not substitutes.

### Horizon and assumptions

Use week 1, month 1, quarter 1, and year 1. These are learning checkpoints, not delivery estimates. Assume one main builder, reuse of existing experiments, and access to a small number of Rust maintainers willing to provide tasks and feedback. Recruiting those maintainers is itself an uncertain dependency.

Week 1 should collapse a key uncertainty, not deliver a general graph engine. Month 1 should produce a narrow usable loop. Quarter 1 should test repeat usage. Year 1 describes what could compound if earlier gates hold.

### Evidence we actually have

| Evidence | What it supports | What it does not establish |
| --- | --- | --- |
| [Pinned semver graph experiment](../experiments/public-interface-semver/README.md) | Typed public-interface records, re-export mapping, configuration-specific graphs, and queryable output are concrete | General Rust coverage, a runtime call graph, large-workspace performance, or PMF |
| Same semver experiment: agents found 10 of 19 scored field/signature dependencies | Plausible inventories can omit important edges | A general accuracy score for all LLMs or all Rust crates |
| [Pretrained binding helper](../experiments/tiny-reexport/learned-binding/pretrained/README.md) | A 17.45 MB encoder with trained heads ran a narrow CPU inference task below 100 MB in the reported runs | A generative LLM, whole-source graph generation, or whole-system memory compliance |
| [Fresh-crate weight-edit comparison](../experiments/tiny-reexport/learned-binding/pretrained/weight-edit/README.md) | The unchanged and edited models each recovered seven confirmed reference sites after rules abstained on the fresh set | Seven independent reasoning mechanisms; the weight edit added no accepted decisions |
| [Deep Insight 02](Deep-Insight-02.md) | A proposed training ladder and narrower supervised tasks | Verified training times, a guaranteed accuracy ceiling, or a proven sub-100 MB graph model |

The model evidence is mixed: no accepted bindings on the original five releases, some confirmed residual bindings on the fresh set. Keep both observations. The reference sites and feature configurations matter more than a favorable headline.

External grounding: [rust-analyzer supports non-Cargo project descriptions](https://rust-analyzer.github.io/book/non_cargo_based_projects.html), so absent Cargo metadata is not by itself a defensible product boundary. [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) already scans public-API compatibility, so a review product needs value beyond listing signature breakages. [Software reflexion models](https://www.cs.ubc.ca/~murphy/papers/rm/fse95.html) provide precedent for comparing intended and extracted structures. None of these sources validates demand for Parseltongue's proposed combination.

## Timeline A

### The Local Navigator: tiny model + CLI + MCP

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

## Timeline B

### The Design Workspace: current map + proposed map + implementation check

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

## Timeline C

### The Intent-Aware Reviewer: graph deltas + CLI/CI + a small learned assistant

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

## Timeline D

### The Graph Component: tiny source-to-record model + runtime + SDK

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

## Cross-Timeline Analysis

### Different jobs, one underlying representation

| Path | Primary repeated job | Upside | Main downside | Reversibility | Regret risk | Who must cooperate |
| --- | --- | --- | --- | --- | --- | --- |
| A: Local Navigator | Find the right contracts and source for a task | Frequent feedback; closest to tiny model + CLI companion | Navigation may be useful but insufficiently differentiated | High: graph/runtime carries into B, C, or D | Optimizing token savings without preserving correctness | Contributors and agent-harness users |
| B: Design Workspace | Agree on a structural change before implementation | Closest expression of current plus future CODEMAP | Extra planning ceremony and hard identity/mapping problems | Medium-high if proposals stay portable | Building a design environment nobody wants to maintain | Maintainers with real cross-cutting changes |
| C: Intent-Aware Reviewer | Understand and validate architectural deltas | Concrete review moment and an inspectable artifact | Noise, CI friction, and dangerous false reassurance | High for graph/diff core; integration work less reusable | Becoming another ignored review bot | Reviewers, contributors, and CI owners |
| D: Graph Component | Add Rust graph capability to another product | Reuse across tools; model-centered research and distribution | Highest dependence on generalizable learned capability and integrations | Medium: data and runtime remain useful, custom training may not | Spending heavily on model quality before proving demand | Independent tool builders and permitted data sources |

These are qualitative judgments, not probability estimates. A and C can yield user-workflow evidence earlier; B needs a real design cycle; D needs technical generalization and an integrator. That makes D the highest-variance starting point under current evidence, not necessarily the least valuable eventual outcome.

### The important combinations

| Combination | Why it could work | Why not start with everything |
| --- | --- | --- |
| A + B | Navigate the current map, propose a bounded change, then navigate the intended one | Proposal authoring must not delay useful everyday reads |
| A + C | Same local graph powers development context and a review artifact | CI configuration, permissions, and reviewer UX add a separate operational burden |
| B + C | Approved intent becomes the basis for implementation conformance | Requires teams to actually record intent before implementation |
| A + D | CLI demonstrates the embedded model/runtime; SDK lets others reuse it | External API commitments can freeze a schema before real usage teaches us enough |

The most robust starting combination is **A with one small B/C loop**: local navigation plus one proposal and one before/intended/after report. It tests the full thesis without building a design IDE, a hosted review service, and a training platform at once.

### What would be a durable advantage?

Potential advantages are hypotheses: a difficult-case dataset with defensible provenance; reliable Rust-specific identity and configuration handling; compact graph queries that improve real tasks; and accumulated project intent that users find valuable to preserve. Merely using a small model, offering MCP, or storing a graph is not itself evidence of defensibility.

The smallest model is not automatically the best product. A slightly larger model that changes correct outcomes could be more useful, but it would violate a strict 100 MB requirement. Do not quietly relax that constraint to rescue a weak result; either narrow the task or explicitly reconsider the product envelope.

### Shared failure branches

- A model learns plausible public names but misses dependencies: inventory accuracy looks strong while navigation remains unsafe.
- A wrong feature profile makes both the baseline and model evaluation misleading.
- An agent follows a projected relationship as though it were a direct runtime call.
- A comment or rename creates noisy graph churn because identity and content revision were conflated.
- A proposed edge is rendered indistinguishably from an observed one.
- A graph passes structural checks while the implementation changes behavior incorrectly.
- Privacy or licensing prevents using retained project traces as training examples.
- The system demonstrates lower inference RSS by moving required work into an unreported subprocess.

These risks are shared infrastructure work, not a reason to add unrelated graph algorithms to the first product.

## Decision Filter

### Recommended direction

**Describe Parseltongue as a Rust CODEMAP companion: a local CLI/MCP tool plus a specialized small model that helps construct and maintain the map consumed by larger coding agents.** Start with A, retain B as the product direction, and use a small C-style report to make the result inspectable. Keep D available as a packaging decision after the model/runtime demonstrates reusable value.

This recommendation does not demote the graph generator to a generic chatbot accessory. The generator is the common foundation. It also does not assume that a tiny model can independently design a large Rust system.

### The next uncertainty-collapsing experiment

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

### How to choose after the pilot

- **Strongest if things go normally:** A provides frequent use and shared infrastructure; extend into B when users request pre-edit planning.
- **Safest if learned generalization disappoints:** preserve the CODEMAP tool and labeled evaluation corpus, narrow the model's task, and report the missed research goal honestly. Do not pretend the model-enabled thesis was validated by a tool-only win.
- **Choose C sooner** if maintainers consistently value the change report but do not want to author proposals. In that case, distinguish descriptive review from approved-intent conformance.
- **Choose B sooner** if maintainers repeatedly use the map to resolve design disagreements before writing code.
- **Choose D sooner** only when independent builders request and retain the graph component and model contribution survives ablation.
- **Stop expanding scope** if users do not return, the map reduces answer quality, or model corrections consume more effort than they save. Investigate the failed assumption rather than adding features.

**Final thesis:** the opportunity is not a tiny model that knows everything about Rust. It is a small, inspectable local system that gives larger agents a reliable way to understand existing connections, propose different ones, and check what changed.
