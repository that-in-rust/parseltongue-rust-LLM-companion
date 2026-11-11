# agent-working-method-playbook

Apply these methods within AGENTS.md constraints; use only the workflow and research depth the task needs. This is a distillation, not a mandate to create every source template for every change.

## Principles

Source: [A00-LLM-Principles01.md](https://github.com/that-in-rust/agent-room-of-requirements/blob/main/A00-LLM-Principles01.md).

- Supply precise names, types, constraints and relevant examples; repair weak input context before retrying an unhelpful answer.
  Why: retrieval and composition depend on the context available to the agent.
- Explore the problem, constrain interfaces and limits, refine the simplest solution, then verify it.
  Why: a fluent first draft is not evidence of correctness.
- Save requirements, architecture decisions and open questions after major decisions rather than trusting conversation memory.
  Why: long sessions lose earlier constraints.
- Explain the proposed solution in plain steps; challenge assumptions, runtime edges and maintenance risks, then simplify it.
  Why: critique exposes weak reasoning before repair becomes expensive.
- Maintain a scoped negative-knowledge file with concrete failures, causes, detection signals, safer replacements and proof gates; consult it when prompting and reviewing.
  Why: explicit exclusions prevent repeated mistakes more reliably than positive examples alone.
- Write a failing executable specification before behavioral implementation, including regression coverage for fixes; use structural acceptance checks for documentation-only changes.
  Why: observable pass/fail criteria remove interpretation gaps that prose leaves open.
- Name new components and artifacts with exactly four descriptive hyphenated words, following AGENTS.md rather than weakening its rule to a preference.
  Why: stable, specific names improve retrieval without creating competing nomenclature.
- Choose the lightest safe process for the classified work type and revisit the classification when scope grows.
  Why: heavy process wastes repair time while shallow process builds the wrong system.
- Start requirements from user value and iterate architecture alongside them; remove scope when design reveals a simpler useful result.
  Why: implementation discoveries should reduce work, not merely expand it.
- Serialize current phase, requirements, decisions, rejected options, test results, open questions and next steps in durable files before interruption or handoff.
  Why: another session must be able to restart without guessing; keep research additions append-only.
- Proceed if correctness can be defined now; research when facts are missing; escalate when requirements cannot define correctness.
  Why: explicit delegation boundaries preserve useful autonomy without silent scope drift.
- Record causes of failures, successful fixes and repeatable wins, then feed them into tests and review habits.
  Why: delivery improves only when observed outcomes change future behavior.

## Workflow and task routing

Source: [A01-LLM-Workflow01.md](https://github.com/that-in-rust/agent-room-of-requirements/blob/main/A01-LLM-Workflow01.md).

- Classify work using the questions below in order; choose the first applicable route.
  Why: uncertainty and existing patterns determine necessary process, not the size of the request alone.

```text
Did something break?                 yes -> bug
Does a suitable existing pattern fit? yes -> enhancement
Is the new capability bounded?        yes -> feature
Otherwise                                 product
```

### bug-repair-regression-workflow

- Reproduce → diagnose the reachable root cause → write a failing regression test → make the smallest fix → run relevant checks → record why the failure escaped.
  Why: the shortest safe repair ends with both proof and protection against recurrence.
- Load relevant failure notes and test inventory; skip product discovery, broad architecture exploration and spikes unless diagnosis requires them.
  Why: unrelated planning adds cost without increasing repair confidence.

### enhancement-existing-pattern-workflow

- Confirm existing rails → write a short value/scope statement → locate the closest prior example → write failing tests → implement and refine → review and deliver.
  Why: extending a proven pattern is safer and cheaper than inventing another one.
- Load dependency context, existing interfaces and idioms; promote to feature when no suitable prior pattern exists.
  Why: discovering new rails invalidates an enhancement's low-uncertainty premise.

### feature-bounded-capability-workflow

- Write requirements from the user journey → iterate requirements and architecture together → minimize scope → define executable specs → implement with TDD → review → deliver and observe.
  Why: bounded novelty needs design, but the first requirements draft must not become sacred.
- Load relevant principles, failures, idioms, dependency graph, interfaces, tests and domain references only as needed.
  Why: a focused context supports design without importing unrelated discovery work.

### product-discovery-validation-workflow

- Gather signals → define the problem and sizing → state hypotheses → spike risky feasibility, desirability and viability assumptions → co-design minimal requirements and architecture → build the smallest useful result with TDD → observe and iterate or pivot.
  Why: an unclear system needs evidence that the problem and solution are worth building before scale.
- Load discovery and spike guidance first, then relevant domain and implementation references for the build phase.
  Why: activity and detailed architecture are not substitutes for validation.

- Set a task-appropriate time budget and reclassify work that repeatedly exceeds its expected scope; treat source durations as illustrative, not delivery guarantees.
  Why: budget overruns can reveal hidden uncertainty rather than slow execution.
- Load principles and known failures first, then only phase, domain and codebase references that the selected route needs; use actual repository paths rather than assuming the source's S/C/D inventories exist here.
  Why: progressive loading preserves attention and prevents invented dependencies.

## Research, idioms and skill building

Source: [A02-Mega-Idiomatic-Prompt.md](https://github.com/that-in-rust/agent-room-of-requirements/blob/main/A02-Mega-Idiomatic-Prompt.md).

- State target technology/version, runtime, delivery surface, audience, quality priorities, selected modes and unknown constraints before researching patterns.
  Why: idiomatic defaults change across libraries, CLIs, services, ingestion pipelines and operational boundaries.
- Read long sources completely in chunks of at most 100 lines; append evidence after each chunk and consolidate duplicates or contradictions after each file.
  Why: incremental capture prevents late synthesis from silently dropping evidence.
- Maintain an evidence ledger containing source URL/path, category, authority, version/date, line range, claims, candidate idioms, failure boundaries, proof gates and uncertainty; record skipped binary, empty, duplicate or irrelevant sources honestly.
  Why: recommendations need traceable evidence, not an impression of having read everything.
- Prefer official references, style/API docs, release/migration notes, maintainer examples and maintained production code; consult advisories, testing/build/runtime docs and community discussion where relevant.
  Why: authority, freshness and representative use matter more than popularity.
- Rank sources from 5 (official or maintainer) through 4 (serious production), 3 (current evidenced community), 2 (narrow tutorial) to 1 (stale or unverified); preserve conflicting claims with their version/context boundaries.
  Why: disagreement must be explained rather than averaged into false certainty.
- Mine types, parsing/trust boundaries, errors, async/resource ownership, architecture, testing, security, operations and documentation as applicable; do not stop at syntax.
  Why: reliability failures cross language and runtime boundaries.
- Score candidate idioms using the rubric below, state scores as judgments rather than measurements, and require a strong source for each default.
  Why: explicit ranking favors prevention and proof over fashionable or clever patterns.

```text
pattern-failure-prevention-power       25
pattern-misuse-resistance-strength     15
diagnostic-feedback-quality-value      10
ecosystem-guidance-fit-strength        15
applicability-breadth-usage-range       10
maintainability-change-survival-score   10
verification-gate-evidence-strength     10
cost-tradeoff-clarity-level              5

93–100: default adoption
86–92:  strong with context
80–85:  adopt only when constraints match
Below 80: quarantine unless experimental work is requested
```

- Specify each pattern's adoption boundary, exclusions, prevented failure, misuse resistance, diagnostic value, tradeoff, minimal example and verification gate.
  Why: a recommendation without boundaries is an invitation to misapply it.
- Pair every positive idiom with a failure entry containing cause/effect, detection signal, safer replacement and proof gate; inspect silent errors, unchecked inputs, hidden state, blocking async work, unbounded concurrency, missing cleanup and unsafe retries.
  Why: a pattern is useful when it prevents a recognizable failure class.
- Review with three to five perspectives, including a skeptic; ask what breaks under stress or maintenance and what evidence would change each recommendation.
  Why: independent concerns expose adoption costs that a single enthusiastic lens misses.
- Trace executable requirements to tests, exact commands and observed results; measure nonfunctional claims and explicitly label fact, inference and unresolved uncertainty.
  Why: confidence and numeric scores are not runtime evidence.
- Present premise/boundaries → source evidence → ranked patterns and thesis → mode-specific defaults → boundary design → verification → failure registry → optional skill packaging → refresh queries → material open questions, using vertical sections and fenced text records.
  Why: readers should find a usable default and its proof without decoding a content dump.
- Package skills only when requested, with a four-word hyphenated name, precise triggers, inclusion/exclusion rules, mode routing, execution steps, lean core instructions, reference map, guardrails, output expectations, repo-native checks and source provenance.
  Why: progressive disclosure makes a reusable skill discoverable without loading its entire research corpus.
- Preserve refresh queries for official docs, style guides, release notes, reference implementations, tests, advisories and migrations; ask only questions that materially change recommendations.
  Why: future updates should target evidence that can invalidate today's defaults.
- Before delivery, check all active parameters, source strength, applicability boundaries, tradeoffs, proof gates, safer replacements, version labels, naming and reference links; summarize corrections made during review.
  Why: the final artifact must support both human decisions and reliable agent reuse.

## Not adopted

- Do not adopt A00's optional four-word naming or A02's snake_case and space-separated reusable labels; AGENTS.md requires explicit four-word hyphenated names.
- Do not copy A01's S01–S08, C01–C04 and D01–D04 paths or fixed time estimates as repository requirements; those inventories are external and durations are illustrative, not measured here.
- Do not adopt A02's wide scoreboard, evidence and registry tables; preserve their information as vertical sections or fenced text under the repository's formatting rule.
- Do not load every A02 mode or require its full twelve-part research artifact for routine repairs; A00/A01 require the lightest safe process.
- Do not import MERN, Tauri, MLX, frontend/mobile-host or other unrelated stack prescriptions from examples or linked material; this repository's relevant Rust surfaces govern selection.
- Do not treat A00's model analogy or turn-by-turn context decay diagram as measured guarantees; retain concrete context preparation and checkpointing practices instead.
- Do not interpret state serialization as permission to rewrite old research; AGENTS.md makes devin-research append-only and requires mirroring standalone Deep-Insight files.
