# Rust Idioms Checklist Patterns

**Decision supported.** This section helps decide which API-type, error, ownership, async, architecture, testing, tooling, documentation, and observability idioms should govern a Rust change. The seed title does not define how to select, sequence, and verify Rust idioms for a library, CLI, service, async worker, or refactor with different API, ownership, concurrency, and compatibility risks.

**Recommended default and causal basis.** Model invariants in types, expose borrowed and domain-shaped APIs, keep errors actionable, own async cancellation and backpressure, minimize lock scope and unnecessary cloning, preserve dependency boundaries, and prove claims with repository-aligned tests and gates. Rust's compiler prevents many classes of misuse but cannot choose product invariants, error semantics, task ownership, lock granularity, dependency direction, test depth, or operational evidence.

**Operating conditions and assumptions.** The crate role and MSRV are known, callers and failure modes are identifiable, concurrency and performance claims are bounded, repository commands are available, and changes can be tested at the right layer. Use this reference for general Rust design review and route backend, embedded, unsafe, FFI, cryptography, or framework specifics to dedicated evidence.

**Failure boundary and alternatives.** The checklist is applied mechanically, generic traits or `AsRef` obscure APIs, cloning is removed at readability cost, async is introduced without need, all-feature gates conflict with the workspace, or compile success substitutes for behavior. Bounded alternatives and recoveries: keep a concrete API until callers demand abstraction, use owned values at real ownership boundaries, remain synchronous, use a focused test and feature set, preserve existing error conventions, or isolate legacy design behind typed adapters.

**Counterexample, gotchas, and operational comparison.** Newtypes without validation, `anyhow` leaking into libraries, locks across await, unbounded channels, cancellation orphaning effects, `Arc` used by habit, traits with one unneeded implementation, and benchmarks without constraints. Good: a library parses a newtype with `TryFrom`, exposes `&str`, returns contextual variants, and property-tests invariants. Bad: add generic conversions and clones everywhere. Borderline: `Arc<Mutex<_>>` is valid only when shared ownership and critical sections are explicit and stress-tested.

**Verification, evidence, and uncertainty.** Inspect public signatures and invalid states, exercise error variants and context, audit panics and clones, test cancellation and channel saturation, inspect lock spans, verify dependency direction, run unit, property, integration, docs, build, format, lint, and targeted benchmarks from repository config. The fully read local checklist directly covers nine review areas and concrete high-value questions. The two local paths are duplicate copies, external sources were not refreshed, and the checklist cannot define current crate APIs, project MSRV, framework behavior, or performance budgets.

**Second-order consequence.** Idiomatic Rust is not maximum type or trait sophistication; it is the smallest design that makes ownership, invalid states, failure, and concurrency obligations visible.

**Revision decision.** Add task classification, causal idiom selection, counterexamples, compatibility and feature boundaries, staged alternatives, operational examples, and claim-linked verification.

**Retained seed evidence.** The exact title, two duplicate local rows, and four external rows remain preserved as the frozen evidence base. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

## Source Evidence Mapping Table

**Decision supported.** This section helps decide which evidence may support a general Rust idiom, an async-runtime claim, a framework claim, or only historical provenance. The seed lists two byte-identical monthly copies of one checklist plus API, Tokio, and Axum sources, while the checklist itself names two unavailable absolute-path source bases.

**Recommended default and causal basis.** Treat the two local rows as one checklist lineage, preserve its inaccessible upstream references as a provenance limitation, use API Guidelines for applicable public API claims, and use Tokio or Axum only for locked-version ecosystem mechanics. Duplicate files do not corroborate, absent upstreams cannot be audited, and framework repositories cannot establish universal Rust design guidance.

**Operating conditions and assumptions.** Every claim names its layer, local lineage, version or project context, current freshness status, and direct repository or test corroboration. Apply the six frozen rows only to capable claims and treat project code, MSRV, Cargo features, and tests as direct evidence.

**Failure boundary and alternatives.** Two copies become two votes, inaccessible downloads are presented as verified, Axum examples govern non-web crates, Tokio design implies all async code, or external source prestige replaces local needs. Bounded alternatives and recoveries: downgrade unsupported history, inspect repository code and Cargo metadata, consult locked crate source or rustdoc, create a minimal compile or concurrency fixture, or route domain claims.

**Counterexample, gotchas, and operational comparison.** Copy drift in future months, API guideline scope, feature flags, MSRV differences, docs.rs latest, framework-specific types entering core guidance, and absent source licenses or context. Good: cite the local API checklist and current project signatures, then use API Guidelines for one named convention. Bad: cite Axum to justify a CLI trait. Borderline: Tokio guidance is relevant only for a Tokio runtime path with pinned features.

**Verification, evidence, and uncertainty.** Hash local copies, record inaccessible upstreams, build a claim ledger with layer and version, inspect Cargo files and repository conventions, run targeted fixtures, and flag conflicts and unused sources. Both local copies were read and hash-identical, and the external rows target relevant but narrower primary ecosystems. Upstream download files and public pages were not read or refreshed, so provenance and current mechanism claims remain bounded.

**Second-order consequence.** Source lineage matters because a compact checklist is synthesis evidence, not independent proof of every idiom it contains.

**Revision decision.** Add duplicate-lineage handling, unavailable-upstream warning, claim-specific authority, locked-version gates, counterexamples, and a provenance ledger.

**Retained seed evidence.** All two local and four external rows remain exactly preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| source_location_path_key | source_category_artifact_type | source_authority_confidence_level | source_usage_synthesis_role |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/codex-skills-202602/idiomatic-rust-coder-01/references/rust-idioms-checklist.md | local_corpus_source_material | local_corpus_sourced_fact | skill entrypoint or reusable agent prompt |
| archive/monthly-agent-snapshots/codex-skills-202603/idiomatic-rust-coder-01/references/rust-idioms-checklist.md | local_corpus_source_material | local_corpus_sourced_fact | skill entrypoint or reusable agent prompt |
| https://github.com/rust-lang/api-guidelines | external_research_source_material | external_research_sourced_fact | Rust library-team API design recommendations |
| https://github.com/tokio-rs/tokio | external_research_source_material | external_research_sourced_fact | async runtime implementation and operational model |
| https://github.com/tokio-rs/axum | external_research_source_material | external_research_sourced_fact | Rust web framework implementation and design claims |
| https://docs.rs/axum/latest/axum/ | external_research_source_material | external_research_sourced_fact | published crate documentation for routing and extractors |

## Pattern Scoreboard Ranking Table

**Decision supported.** This section helps decide how to order these safeguards without treating scores as confidence, idiomaticity, safety, or measured review effectiveness. The seed assigns 95, 91, and 88 to source mapping, evidence separation, and verification without a scale, sample, crate class, or defect outcome.

**Recommended default and causal basis.** Establish source and repository context, separate direct facts from synthesis, and attach behavior, compile, concurrency, and documentation gates; then prioritize concrete idioms by the change's invariants and failure cost. An async lock issue, public API break, library error leak, or unnecessary clone has different relevance independent of small editorial score differences.

**Operating conditions and assumptions.** Each row becomes an observable gate, task risk reorders reviewer attention, and all applicable safeguards remain required. Retain numbers as editorial metadata, not assurance, compatibility, benchmark, or quality scores.

**Failure boundary and alternatives.** 95 becomes an idiomaticity percentage, lower-ranked verification is skipped, scores resolve source conflict, or aggregate rank hides a panic or cancellation gap. Bounded alternatives and recoveries: use a risk matrix by API and lifecycle, classify hard invariants versus contextual style, order checks by blast radius, or combine rows into one completion gate.

**Counterexample, gotchas, and operational comparison.** False precision, treating convention as correctness, optimizing compile attempts, comparing editorial scores across references, and rewarding generic abstraction. Good: prioritize cancellation and lock scope for a worker while preserving API and evidence checks. Bad: call code 95-percent idiomatic. Borderline: use rank only to order review attention.

**Verification, evidence, and uncertainty.** Record crate role, applicable safeguards, ordering rationale, defects caught, skipped controls and boundaries, final gate outcomes, and residual uncertainty. The three seed rows are exact source facts, but no calibration or empirical validation is supplied. Relative value changes with library, binary, runtime, feature, MSRV, unsafe, and performance context.

**Second-order consequence.** The scoreboard is useful as a process guard, whereas idiomatic quality remains a claim-by-claim design and evidence judgment.

**Revision decision.** State scale limits, connect safeguards to risk classes, add task-specific reorderings, require applicable gates, and prohibit achieved-rate claims.

**Retained seed evidence.** The identifiers, 95, 91, and 88 values, tiers, and prevention targets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| pattern_identifier_stable_key | pattern_score_numeric_value | pattern_tier_adoption_level | pattern_failure_prevention_target |
| --- | ---: | --- | --- |
| `rust_idioms_checklist_patterns` | 95 | default_adoption_pattern_tier | Source Map First: Load local rust idioms material before synthesizing checklist patterns recommendations. |
| `rust_idioms_checklist_patterns` | 91 | default_adoption_pattern_tier | Evidence Boundary Split: Separate local facts, external facts, and inference before giving agent instructions. |
| `rust_idioms_checklist_patterns` | 88 | default_adoption_pattern_tier | Verification Gate Coupling: Attach each recommendation to at least one command, checklist, or review gate. |

## Idiomatic Thesis Synthesis Statement

**Decision supported.** This section helps decide what governing rule should determine whether a Rust pattern improves a design rather than merely resembles familiar style. The seed says to combine local and external evidence with verification but does not explain that idioms are tools for making invariants, ownership, failure, concurrency, and dependency obligations explicit.

**Recommended default and causal basis.** Choose the smallest API and architecture that expose required invariants, ownership transfer, failure semantics, task lifecycle, and dependency direction, then verify those obligations at compile, behavior, concurrency, and integration layers. Rust syntax and compiler checks constrain possibilities but cannot decide domain types, abstraction cost, cancellation semantics, lock scope, error context, or test sufficiency.

**Operating conditions and assumptions.** The design has known callers, invariants, errors, ownership, runtime, features, MSRV, dependencies, and measurable claims. Apply the thesis to general safe Rust and route unsafe, FFI, embedded, cryptographic, framework, and distributed concerns to dedicated guidance.

**Failure boundary and alternatives.** Idiomatic becomes more generics, fewer clones at any cost, universal async, traits without seams, one error crate everywhere, or all-feature commands applied regardless of workspace design. Bounded alternatives and recoveries: keep concrete types, use an owned boundary, preserve synchronous flow, expose a small enum, return a repository-native error, add a trait only at a real substitution seam, or stage architecture changes.

**Counterexample, gotchas, and operational comparison.** Semver effects in public APIs, hidden allocations in ergonomic conversions, cancellation after partial effects, lock guards hidden in helpers, trait object object-safety, feature combinations, and doctests that compile under different cfg. Good: an ID newtype validates on `TryFrom`, a service borrows inputs, and errors preserve actionable variants. Bad: genericize every input and wrap all state in `Arc`. Borderline: `AsRef<str>` helps only when it simplifies real callers without ambiguous conversions.

**Verification, evidence, and uncertainty.** Derive obligations, inspect public API and semver, compile representative call sites, test invalid states and errors, stress task and lock behavior, inspect allocations where claimed, check feature and MSRV matrices, run docs and integration workflows. The local checklist directly covers the obligation categories and high-value review questions. Its brevity and duplicate lineage leave current crate, language, framework, and project constraints to direct evidence.

**Second-order consequence.** Idiomatic design minimizes hidden contracts, not necessarily type count, line count, cloning count, or abstraction depth.

**Revision decision.** Replace generic synthesis with obligation mapping, smallest-fit alternatives, counterexamples, semver and compatibility limits, and layered verification.

**Retained seed evidence.** The local, external, and combined evidence statements remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`local_corpus_sourced_fact`: The local row for `rust_idioms_checklist_patterns` contains 2 source path(s), which should be treated as the first retrieval surface for this theme.
`external_research_sourced_fact`: The external source map provides public documentation, repository, or ecosystem analogues where available.
`combined_evidence_inference_note`: Reliable use of Rust Idioms Checklist Patterns comes from loading the local corpus first, checking public ecosystem guidance second, and converting both into verification-backed agent decisions.

## Local Corpus Source Map

**Decision supported.** This section helps decide which checklist area should guide the present Rust change and when direct repository or domain evidence must take over. The seed lists two identical checklist files but does not route reviewers among API design, errors, async, ownership, architecture, tests, tooling, docs, observability, and review questions.

**Recommended default and causal basis.** Route signatures and invariants to API design, failure contracts to errors, tasks and locks to async hygiene, clones and sharing to ownership, dependencies to architecture, claims to testing, repository commands to tooling, and runtime diagnosis to docs and observability. Heading-level routing keeps a compact checklist actionable and avoids loading every idiom as a mandatory style transformation.

**Operating conditions and assumptions.** The task is classified, the exact bullet and obligation are read, repository conventions and Cargo metadata are inspected, and unaddressed domain risks are routed. Use it for review orientation and require project or primary evidence for mechanisms and claims.

**Failure boundary and alternatives.** Two copies imply corroboration, every bullet applies to every crate, error crate examples become mandates, or review questions replace code and tests. Bounded alternatives and recoveries: read project APIs and tests first, consult locked crate source, build a minimal compile fixture, use a specialized reference, or retain the current concrete design.

**Counterexample, gotchas, and operational comparison.** The checklist source basis is unavailable, month copies can diverge later, L1/L3 vocabulary may not match repository modules, and all-feature commands can be invalid by design. Good: route a worker refactor to cancellation, bounded channels, lock scope, stress tests, and tracing. Bad: use async guidance for a parser library. Borderline: property tests apply when invariants justify generators and shrink behavior.

**Verification, evidence, and uncertainty.** Record task class, checklist heading and bullet, source hash, repository files, Cargo features and MSRV, current mechanism evidence, selected gates, conflicts, and stop condition. The full 73-line checklist was read and the two monthly copies are byte-identical. One unique synthesis source and unavailable upstreams limit independent local support.

**Second-order consequence.** A concise checklist is strongest as a routing index and weakest as standalone proof.

**Revision decision.** Turn duplicate rows into one heading router, add source-basis warnings, stop conditions, misuse examples, repository handoff, and a review audit.

**Retained seed evidence.** Both exact local paths, titles, heading signals, and usage roles remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| local_source_filepath_value | local_source_title_text | local_source_heading_signals | local_source_usage_role |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/codex-skills-202602/idiomatic-rust-coder-01/references/rust-idioms-checklist.md | Rust Idioms Checklist | Rust Idioms Checklist; 1) API and Type Design; 2) Error Handling; 3) Async and Concurrency Hygiene; 4) Ownership and Memory; 5) Architecture Boundaries | skill entrypoint or reusable agent prompt |
| archive/monthly-agent-snapshots/codex-skills-202603/idiomatic-rust-coder-01/references/rust-idioms-checklist.md | Rust Idioms Checklist | Rust Idioms Checklist; 1) API and Type Design; 2) Error Handling; 3) Async and Concurrency Hygiene; 4) Ownership and Memory; 5) Architecture Boundaries | skill entrypoint or reusable agent prompt |

## External Research Source Map

**Decision supported.** This section helps decide which external source may support an idiom claim and what project evidence is still required. The seed lists Rust API Guidelines, Tokio, Axum source, and Axum docs without defining when library API, runtime, framework implementation, or published crate documentation is relevant.

**Recommended default and causal basis.** Use API Guidelines for applicable public library conventions, Tokio for locked runtime behavior, Axum source for implementation questions, and matching Axum documentation for framework contracts; ignore ecosystem rows when the crate does not use them. Current primary sources can clarify mechanisms, but they cannot select domain invariants, repository architecture, MSRV, features, or performance budgets.

**Operating conditions and assumptions.** The crate and claim layer are named, versions and features are pinned, repository conventions agree, and compile or behavior fixtures confirm composition. Use each row only for its layer and keep repository obligations primary.

**Failure boundary and alternatives.** Docs.rs latest overrides Cargo.lock, Axum governs generic Rust, Tokio implies async best practice everywhere, or API recommendations are treated as compiler rules. Bounded alternatives and recoveries: inspect local rustdoc and dependency source, use standard-library documentation, preserve existing patterns, run a minimal feature fixture, or route to another ecosystem source.

**Counterexample, gotchas, and operational comparison.** Feature flags, major-version differences, API guideline scope, executor assumptions, middleware types entering domain layers, and examples omitting semver or MSRV. Good: use locked Tokio docs to validate cancellation behavior in a Tokio worker. Bad: cite Axum for an iterator API. Borderline: an API guideline applies only after public callers and compatibility impact are considered.

**Verification, evidence, and uncertainty.** Record URL and refresh status, crate version and features, Cargo files, exact mechanism, public or internal scope, compile and runtime fixture, conflicts, and selected authority. The four source families are relevant to their stated layers. No browsing occurred, so current repository, crate, and documentation behavior remains unverified.

**Second-order consequence.** External evidence should answer a narrow mechanism question, not convert an ecosystem's design into a universal idiom.

**Revision decision.** Add claim-layer authority, locked-version and feature rules, MSRV and semver limits, alternatives, composition tests, and freshness labels.

**Retained seed evidence.** All four external URLs, names, roles, and evidence labels remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| external_source_url_value | external_source_name_text | external_source_usage_role | evidence_boundary_label_value |
| --- | --- | --- | --- |
| https://github.com/rust-lang/api-guidelines | Rust API Guidelines | Rust library-team API design recommendations | external_research_sourced_fact |
| https://github.com/tokio-rs/tokio | Tokio repository | async runtime implementation and operational model | external_research_sourced_fact |
| https://github.com/tokio-rs/axum | Axum repository | Rust web framework implementation and design claims | external_research_sourced_fact |
| https://docs.rs/axum/latest/axum/ | Axum docs.rs | published crate documentation for routing and extractors | external_research_sourced_fact |

## Anti Pattern Registry Table

**Decision supported.** This section helps decide which code or design signature should block a Rust change and what smaller idiom should replace it. The seed lists generic evidence failures but omits concrete Rust misuse such as container-shaped APIs, unjustified conversions, primitive invariants, application errors in libraries, production panics, blocking async, locks across await, unbounded channels, clone and Arc habits, premature traits, and dependency leaks.

**Recommended default and causal basis.** Reject unnecessary concrete containers, fallible `From`, invalid-state primitives, library-wide `anyhow`, unchecked panics, runtime blocking, await-held guards, unbounded work, cancellation omission, unexplained hot-path clones, habitual shared ownership, speculative traits, and core-to-ecosystem coupling. These patterns hide caller, failure, ownership, scheduling, memory, or dependency obligations even when the code compiles.

**Operating conditions and assumptions.** Each anti-pattern has a concrete signal, violated obligation, scoped replacement, counterexample, and compile, behavior, concurrency, or architecture test. Apply to general safe Rust and route specialized runtime, unsafe, and domain concerns.

**Failure boundary and alternatives.** Warnings become style nits, all clones are removed, generics replace clarity, every error gets a custom enum, or abstractions are added without real callers and tests. Bounded alternatives and recoveries: borrow slices and strings, use concrete owned parameters at transfer boundaries, validate newtypes, use `TryFrom`, preserve repository errors, isolate blocking work, shorten guards, bound queues, or keep concrete dependencies behind module boundaries.

**Counterexample, gotchas, and operational comparison.** `AsRef` monomorphization and ambiguity, zero-cost claims without measurement, panic in tests versus production, hidden guards in iterators, cancellation safety of partial effects, `Arc` cycles, and traits affecting object safety or semver. Good: replace a raw ID with validated `TryFrom` and test invalid values. Bad: replace every `String` parameter with generic `AsRef<str>`. Borderline: cloning is acceptable when ownership clarity outweighs measured cost.

**Verification, evidence, and uncertainty.** Audit public signatures, conversions, panics, error layers, async blocking, guard lifetimes, channel bounds, cancellation points, clones and allocations, ownership graphs, trait consumers, dependency edges, and regression tests. The local checklist directly names every major misuse and safer direction. Performance, unsafe code, FFI, and framework-specific anti-patterns need additional evidence.

**Second-order consequence.** The common failure is hiding a boundary behind convenience, whether the boundary is ownership, failure, scheduling, or architecture.

**Revision decision.** Retain the generic rows and add code-level signatures, causal obligations, bounded replacements, counterexamples, and targeted verification.

**Retained seed evidence.** Context-free output, unsourced claims, and unverified instructions remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| anti_pattern_failure_name | failure_cause_risk_reason | safer_default_replacement_pattern | detection_signal_review_method |
| --- | --- | --- | --- |
| context_free_summary_output | agent skips local corpus and produces generic advice | source_map_first_synthesis | verify every listed local path appears in the generated file |
| unsourced_recommendation_claims | guidance appears authoritative without source boundary | evidence_boundary_split_pattern | check for local, external, and inference labels |
| unverified_agent_instruction | recommendation cannot be checked by command or review gate | verification_gate_coupling | require concrete gate in generated reference |

## Verification Gate Command Set

**Decision supported.** This section helps decide which gates prove the evolved reference and a concrete Rust change without imposing unsupported workspace-wide commands. The seed provides only a generated-reference verifier, while idiom claims need API compile fixtures, error tests, concurrency stress, cancellation and saturation cases, architecture checks, docs, features, MSRV, format, lint, test, and build evidence.

**Recommended default and causal basis.** Run the focused reference validator, inspect Cargo and CI configuration, test relevant APIs and failures, add concurrency or property cases when obligations require them, then run repository-approved format, lint, test, docs, build, feature, and MSRV gates. Compilation cannot prove error usefulness, cancellation, backpressure, architecture, doctest accuracy, feature compatibility, or performance claims.

**Operating conditions and assumptions.** Each claim maps to the smallest capable gate, command scope matches the workspace, feature combinations are intentional, and failures identify API, behavior, concurrency, documentation, or compatibility layers. Separate reference QA from crate behavior, workspace compatibility, and specialized unsafe or runtime verification.

**Failure boundary and alternatives.** The archive verifier certifies code, `cargo test` alone certifies concurrency, all-features is forced on mutually exclusive features, Clippy replaces design review, or benchmark results lack workload. Bounded alternatives and recoveries: compile a public-call fixture, use targeted crate and feature gates, property-test invariants, stress shared state, run doctests, preserve existing CI wrappers, or withhold claims not reproducible.

**Counterexample, gotchas, and operational comparison.** Workspace default members, feature unification, target-specific code, doctest cfg, MSRV toolchain, flaky scheduling, optimized versus debug behavior, and generated bindings. Good: compile caller examples, test error variants, saturate a bounded channel, cancel work, run scoped CI gates, and verify docs. Bad: run only `cargo check`. Borderline: skip all-features when combinations are invalid and document the tested matrix.

**Verification, evidence, and uncertainty.** Capture toolchain, MSRV, Cargo and feature graph, exact commands, public fixtures, error and edge tests, stress schedule, cancellation, docs, architecture evidence, build targets, benchmark protocol, exits, and residual gaps. The local checklist directly lists layered testing and tooling gates and makes performance tests conditional on constraints. No target workspace is supplied, so exact commands and feature matrices remain repository-dependent.

**Second-order consequence.** Verification should mirror the hidden obligation each idiom exposes; no single cargo command covers them all.

**Revision decision.** Add repository discovery, claim-to-gate mapping, API and concurrency fixtures, feature and MSRV matrices, docs, conditional benchmarks, and the focused validator.

**Retained seed evidence.** The original final-stage reference-generation command remains preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`verification_gate_command_set`: Run the repository verifier after editing this file.

```bash
python3 archive/monthly-agent-snapshots/idiomatic-references-202606/tools/verify_reference_generation.py --stage final
```

## Agent Usage Decision Guide

**Decision supported.** This section helps decide whether this checklist applies and which idiom areas should govern the Rust task. The seed activates on theme mentions but does not classify crate role, API exposure, error boundary, runtime, ownership graph, dependency layer, invariant depth, or performance claim.

**Recommended default and causal basis.** Classify library versus binary, public versus internal API, fallible boundaries, ownership transfer, async runtime, shared state, channel pressure, dependency direction, invariant complexity, MSRV, features, and measurable constraints. Classification avoids applying web-runtime or generic-library patterns to the wrong crate and keeps review proportional to actual obligations.

**Operating conditions and assumptions.** The task is general safe Rust, repository conventions are inspectable, affected callers and effects are known, and each chosen pattern can be verified. Activate for general Rust design and review, then exit when a specialized model is needed.

**Failure boundary and alternatives.** Unsafe internals, FFI, embedded constraints, cryptography, distributed protocols, framework delivery, or domain-specific security dominates. Bounded alternatives and recoveries: route to backend, executable-spec, unsafe, FFI, embedded, async-runtime, performance, security, or framework references while retaining applicable general contracts.

**Counterexample, gotchas, and operational comparison.** Activating for every compiler error, treating Clippy as architecture, adding a trait to mock one function, rewriting ownership without blast-radius analysis, and benchmarking before correctness. Good: a public parser selects borrowed APIs, newtypes, `TryFrom`, error variants, property tests, docs, and MSRV gates. Bad: apply bounded-channel guidance to a sync library. Borderline: a CLI can use `anyhow` at orchestration while library modules keep typed errors.

**Verification, evidence, and uncertainty.** Record crate role, API and ownership boundaries, runtime and features, selected checklist areas, skipped bullets with reasons, adjacent routes, tests, commands, and stop condition. The nine local checklist areas provide a direct general-Rust classifier. Project and domain requirements can override or add to this compact map.

**Second-order consequence.** Idiom selection should be asymmetric: public, concurrent, and invariant-heavy code needs stronger explicit contracts than simple private glue.

**Revision decision.** Replace keyword routing with crate and obligation classification, a reduced path for simple code, specialized exits, and an auditable decision record.

**Retained seed evidence.** The local-first, explicit-gate, external-check, and evidence-label bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`agent_usage_decision_guide`: Use this reference when a task mentions this theme, one of the listed local source paths, or a nearby technology/workflow surface.

- Start with the local corpus source map.
- Prefer patterns with explicit verification gates.
- Treat external sources as freshness and ecosystem checks, not replacements for local repo conventions.
- Preserve the evidence boundary labels when reusing recommendations.

## User Journey Scenario

**Decision supported.** This section helps decide how an engineer should turn one Rust requirement into the smallest idiomatic design whose invariants and failure behavior are reviewable. The seed names a Rust reliability engineer choosing ownership, async, error, and crate boundaries but does not show a progression from requirement to obligations, failing tests, API shape, implementation, integration, and release evidence.

**Recommended default and causal basis.** Inventory callers, values, invalid states, ownership, errors, effects, runtime, dependencies, features, and claims; write failing behavior or compile fixtures; choose the smallest types and seams; implement; refactor; and run layered gates. Designing obligations and tests before abstractions prevents compiler-driven churn from becoming architecture and keeps every idiom tied to a user-visible or system property.

**Operating conditions and assumptions.** The requirement is bounded, callers and compatibility are known, failure and concurrency cases can be reproduced, and repository gates are available. Use this journey for bounded safe-Rust changes and route broad migrations or specialized code.

**Failure boundary and alternatives.** The task spans unrelated crates, no behavior baseline exists, public compatibility is unknown, or the change depends on unreviewed unsafe or domain protocols. Bounded alternatives and recoveries: characterize legacy behavior, add one newtype adapter, keep a concrete function, stage error changes, stay synchronous, isolate runtime work, or narrow the workspace slice.

**Counterexample, gotchas, and operational comparison.** Tests overfitting implementation, compile-fail fixtures without stable diagnostics, refactors changing semver, feature interactions, doctests omitted, and optimization preceding evidence. Good: define a validated identifier and error contract, write invalid and caller fixtures, implement `TryFrom`, then test docs and features. Bad: introduce traits and async before behavior. Borderline: a legacy API can retain owned strings behind a new borrowed internal seam.

**Verification, evidence, and uncertainty.** Preserve requirement, caller map, obligation inventory, red evidence, API and error decisions, ownership and task model, dependency changes, feature and MSRV matrix, green tests, refactor checks, docs, integration, and rollback. The local checklist directly supports API-first, TDD, architecture, tooling, and review decisions. Repository scale, public semver policy, and domain constraints determine the exact sequence.

**Second-order consequence.** The compiler is most useful after the design states what must be representable, borrowable, fallible, cancellable, and testable.

**Revision decision.** Replace the generic role statement with an obligation-to-release journey, checkpoints, alternatives, failure exits, evidence packet, and realistic contrasts.

**Retained seed evidence.** The Rust engineer role, safe API and explicit error need, boundary decision, and opening trigger remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Role based opening scenario: The Rust reliability engineer is starting from a requirement that needs a safe API, explicit error model, and testable boundary and needs a reference that turns source evidence into an executable next step.
Primary user starting state: The user has a `rust_idioms_checklist_patterns` task, one or more local source paths, and uncertainty about which pattern should drive implementation.
Decision being made: choosing the idiomatic ownership, async, error, and crate-boundary shape before coding.
Reference opening trigger: Open this file when the task mentions rust idioms checklist patterns, any mapped local source path, or an adjacent workflow with the same failure mode.

## Decision Tradeoff Guide

**Decision supported.** This section helps decide which idiom to adopt now, adapt to repository constraints, defer, or avoid for the change's callers and obligations. The seed frames adopt, adapt, and avoid around source agreement rather than Rust choices among borrowed and owned data, concrete and generic APIs, newtypes, typed and contextual errors, sync and async, cloning, shared ownership, traits, and layers.

**Recommended default and causal basis.** Adopt patterns that expose real invariants and failure, adapt API and gate shape to MSRV and features, defer abstraction until a seam exists, and avoid complexity that adds hidden contracts without verification value. Each idiom exchanges ergonomics, compile time, semver surface, allocation, indirection, runtime complexity, and testability.

**Operating conditions and assumptions.** The record compares caller diversity, ownership transfer, invariant strength, failure consumers, concurrency, performance evidence, compatibility, dependency direction, and migration cost. Decide per API and lifecycle, not through repository-wide slogans.

**Failure boundary and alternatives.** Source agreement triggers generic APIs, adaptation means cloning everywhere, avoidance leaves panics, or elegance outranks stable callers and tests. Bounded alternatives and recoveries: use `&str` or owned `String` according to lifetime, concrete input or `AsRef`, newtype or validated constructor, error enum or application context, sync or async, clone or borrow, module seam or trait.

**Counterexample, gotchas, and operational comparison.** Lifetime complexity leaking to callers, blanket conversions obscuring ambiguity, error enums becoming enormous, async-trait costs, `Arc` contagion, and layering vocabulary not matching the codebase. Good: borrow a parse input and return an owned domain value; use typed library errors and application context. Bad: genericize every parameter. Borderline: clone a small value at a concurrency boundary when it simplifies ownership and measured cost is negligible.

**Verification, evidence, and uncertainty.** Write a tradeoff record with callers, invariants, ownership, errors, runtime, allocations, semver, MSRV, features, alternatives, wrong-choice cost, tests, rollout, and rollback. The local checklist directly names all compared design directions. It provides no quantitative cost model and project conventions may choose differently.

**Second-order consequence.** The right idiom minimizes total contract complexity across implementation and callers, not complexity inside one function.

**Revision decision.** Reframe the four seed options around contract visibility, ergonomics, semver, runtime, compatibility, staged alternatives, and rollback.

**Retained seed evidence.** Adopt when, Adapt when, Avoid when, and Cost of being wrong remain exactly preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| decision_option_name | when_to_choose_condition | tradeoff_cost_description | verification_question_prompt |
| --- | --- | --- | --- |
| Adopt when | local corpus and external evidence agree on the rust idioms checklist patterns pattern | fastest path, but can copy stale local assumptions | Does the selected pattern appear in the canonical source and current external evidence? |
| Adapt when | local sources are strong but public ecosystem guidance has changed | preserves repo fit, but requires explicit inference notes | Did the reference label the local fact, external fact, and combined inference separately? |
| Avoid when | source evidence is thin, conflicting, or unrelated to the user journey | prevents false confidence, but may require deeper research | Is there a confidence warning or adjacent reference route? |
| Cost of being wrong | wrong rust idioms checklist patterns guidance can send an agent to the wrong files, tests, or abstraction | wasted implementation loop and weaker verification | Would a reviewer know what to undo and what to inspect next? |

## Local Corpus Hierarchy

**Decision supported.** This section helps decide which evidence controls when duplicate checklist guidance, project architecture, current mechanism semantics, and observed behavior disagree. The seed marks February canonical and March supporting even though their bytes match, and it does not define precedence with repository conventions, Cargo metadata, current language or crate docs, compile results, and runtime tests.

**Recommended default and causal basis.** Treat both months as one archived synthesis lineage, use repository APIs and tests for current behavior, Cargo and CI for compatibility policy, locked source or docs for mechanisms, and targeted fixtures for change claims. Duplicate age labels do not create authority differences, and general idioms cannot override existing public contracts or feature and MSRV constraints without migration evidence.

**Operating conditions and assumptions.** The claim is classified, one lineage is counted, direct project evidence is preserved, current sources are versioned, conflicts are logged, and scope narrows when uncertainty remains. Preserve seed roles for archive compatibility while treating the copies as one lineage and direct evidence separately.

**Failure boundary and alternatives.** Canonical month wins by label, two copies imply consensus, existing code is automatically idiomatic, latest docs override lock files, or compile success proves concurrency behavior. Bounded alternatives and recoveries: retain competing designs, create a minimal compile or runtime fixture, stage a compatibility change, consult specialized primary evidence, or keep the concrete implementation.

**Counterexample, gotchas, and operational comparison.** Repository tests encoding accidental API, feature-gated public items, MSRV differences, workspace dependency inheritance, target-specific code, and generated bindings. Good: checklist suggests TaskGroup-like structured ownership conceptually, but project runtime and MSRV determine actual mechanism. Bad: February copy overrides March despite identical bytes. Borderline: current guidance applies only after semver and feature impact.

**Verification, evidence, and uncertainty.** Maintain a conflict ledger with claim, source hash, project code and tests, Cargo and CI, toolchain and crates, fixture results, chosen authority, migration, and residual uncertainty. The local files are proven byte-identical and the checklist is explicit about its review purpose. Its two named upstream download sources are unavailable and external material was not refreshed.

**Second-order consequence.** Claim-relative hierarchy prevents historical labels from masquerading as independent technical authority.

**Revision decision.** Add duplicate collapse, project and Cargo precedence, versioned mechanism evidence, conflict ledger, compatibility handling, and refresh triggers.

**Retained seed evidence.** Both hierarchy rows and their canonical and supporting labels remain unchanged. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Classification vocabulary includes canonical, supporting, legacy, duplicate, and conflicting source roles.

| local_source_filepath_value | corpus_hierarchy_role | heading_signal_to_convert | reviewer_question_to_answer |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/codex-skills-202602/idiomatic-rust-coder-01/references/rust-idioms-checklist.md | canonical primary source | Rust Idioms Checklist; 1) API and Type Design; 2) Error Handling | What guidance, warning, or example should this source contribute to Rust Idioms Checklist Patterns? |
| archive/monthly-agent-snapshots/codex-skills-202603/idiomatic-rust-coder-01/references/rust-idioms-checklist.md | supporting detail source | Rust Idioms Checklist; 1) API and Type Design; 2) Error Handling | What guidance, warning, or example should this source contribute to Rust Idioms Checklist Patterns? |

## Theme Specific Artifact

**Decision supported.** This section helps decide what artifact should accompany a consequential Rust idiom or refactor decision so another reviewer can reproduce and challenge it. The seed requests a worked example with goal, decision, failure, and verification but its three generic fields omit callers, invariants, API, ownership, errors, async lifecycle, dependency layers, compatibility, tests, and alternatives.

**Recommended default and causal basis.** Create an idiom decision packet containing requirement, callers, invariants, invalid states, input and ownership model, errors, effects, runtime and cancellation, shared state, dependency layer, MSRV and features, alternatives, failure cases, tests, docs, performance claims, rollout, and rollback. The packet ties each abstraction to an obligation and exposes costs that code style alone hides from reviewers.

**Operating conditions and assumptions.** Every field has direct code or test evidence, rejected alternatives include consequences, public compatibility is explicit, and unmeasured performance claims are withheld. Require the full packet for public, concurrent, or architectural changes and justify lighter scope for private local edits.

**Failure boundary and alternatives.** The artifact is a checklist copy, API choice lacks callers, ownership diagrams omit cancellation, tests only compile, or rollback cannot restore public compatibility. Bounded alternatives and recoveries: use a smaller packet for private local changes, add a concurrency appendix for workers, add a semver matrix for libraries, or route unsafe and framework decisions.

**Counterexample, gotchas, and operational comparison.** Caller fixtures diverging from real use, public enums becoming non-exhaustive concerns, feature combinations, doctest contexts, benchmark cfg, and architecture diagrams not matching imports. Good: a parser packet compares `&str`, owned output, `TryFrom`, typed errors, property tests, docs, and MSRV. Bad: state that newtypes are idiomatic. Borderline: a worker packet justifies `Arc` and bounded channels with stress and cancellation evidence.

**Verification, evidence, and uncertainty.** Replay caller and invalid-state fixtures, inspect error and ownership paths, stress concurrency, verify dependency graph, run feature and MSRV matrices, execute docs and integration tests, reproduce benchmarks, and test rollback. The local checklist supplies every core packet dimension and the seed explicitly asks for a worked operational artifact. Specialized unsafe, domain, and framework concerns require additional fields and sources.

**Second-order consequence.** The artifact makes idiomaticity falsifiable by connecting each pattern to a caller, invariant, or lifecycle test.

**Revision decision.** Expand the three rows into an obligation, API, ownership, error, runtime, architecture, compatibility, test, and rollback record.

**Retained seed evidence.** The user goal, decision boundary, and verification gate rows remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Theme specific artifact: worked rust idioms checklist patterns example with user goal, decision point, failure mode, and verification gate.

| artifact_field_name | artifact_completion_rule | evidence_source_hint |
| --- | --- | --- |
| user_goal_statement | state the user's concrete goal before applying Rust Idioms Checklist Patterns | local corpus hierarchy plus critique findings |
| decision_boundary_rule | define the point where this reference should be used or avoided | decision tradeoff guide |
| verification_gate_rule | define the command, checklist, or review question that proves the artifact worked | verification gate command set |

## Worked Example Set

**Decision supported.** This section helps decide what concrete behavior distinguishes a well-justified idiom, a fashionable misuse, and a conditionally acceptable compromise. The seed gives generic source-loading contrasts but no Rust examples involving borrowed APIs, newtypes, conversions, errors, ownership, async locks, channels, traits, layers, or tests.

**Recommended default and causal basis.** Judge examples by caller ergonomics, invariant strength, failure clarity, ownership visibility, runtime safety, dependency direction, compatibility, and claim-linked evidence. Code can look conventionally Rust-like while overgeneralizing APIs, hiding cancellation, or adding abstractions that make callers and tests worse.

**Operating conditions and assumptions.** The good case solves a named obligation, the bad case has a plausible cost, and the borderline case states evidence and boundaries that make only a narrower choice valid. Use examples for review calibration, not copy-ready universal APIs.

**Failure boundary and alternatives.** Examples vary only in syntax, the misuse is cartoonish, the good case assumes performance, or the compromise receives permanent approval without revisit criteria. Bounded alternatives and recoveries: use public parser, file adapter, application error boundary, async worker, shared cache, trait seam, or architecture split examples.

**Counterexample, gotchas, and operational comparison.** Borrowed input tied to returned references, conversion ambiguity, error context lost, lock guard hidden, channel capacity arbitrary, and trait mocking diverging from production. Good: parse `&str` into a validated owned ID with `TryFrom` and property tests. Bad: accept every `AsRef<str>` and panic on invalid input. Borderline: clone a small config into tasks only with bounded task count, clear ownership, and negligible measured cost.

**Verification, evidence, and uncertainty.** For each case, enumerate callers, invariants, types, conversions, errors, ownership, tasks, locks, channels, dependencies, MSRV, features, tests, docs, measured claims, and reviewer verdict. All example dimensions are directly represented in the local checklist. Examples are schematic and must be adapted to repository and domain contracts.

**Second-order consequence.** Borderline examples teach that a non-minimal operation can be idiomatic when it makes ownership and failure simpler at a verified cost.

**Revision decision.** Replace generic source contrasts with public API, panic, and task-ownership cases that expose consequences, alternatives, tests, and limits.

**Retained seed evidence.** The original good, bad, and borderline evidence-boundary statements remain retained. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Good example: Use Rust Idioms Checklist Patterns after loading the canonical source, confirming the external evidence boundary, and writing a verification gate before implementation.
Bad example: Use Rust Idioms Checklist Patterns as a generic tutorial while ignoring the mapped local paths, source hierarchy, and cost of being wrong.
Borderline case: Use Rust Idioms Checklist Patterns only after adding a confidence warning when local evidence is thin or conflicts with current ecosystem guidance.

## Outcome Metrics and Feedback Loops

**Decision supported.** This section helps decide which signals show Rust changes are becoming easier to reason about without gaming clone, trait, lint, or compile counts. The seed values fewer compile attempts and review comments and flags hidden ownership or error tradeoffs, but those workflow counts do not measure contract quality, concurrency safety, compatibility, or verified claims.

**Recommended default and causal basis.** Track public API rework, invalid-state escapes, production panic findings, error-context loss, cancellation and contention failures, unbounded work, dependency-boundary violations, feature and MSRV regressions, doctest drift, and unverified performance claims. Compile and review efficiency can improve while behavior and architecture degrade; obligation-specific outcomes reveal whether idioms reduce hidden contracts.

**Operating conditions and assumptions.** Metrics define crate, obligation, denominator, toolchain, features, workload, baseline, adverse threshold, owner, and feedback action. Measure Rust design and review quality, not production SLOs or security assurance without dedicated metrics.

**Failure boundary and alternatives.** Zero Clippy warnings proves design, fewer clones imply speed, more traits imply testability, compile attempts exclude cached work, or review comments are suppressed. Bounded alternatives and recoveries: use qualitative architecture audits at low volume, sample public and concurrent changes, track regression cases, or maintain an expiring idiom-debt ledger.

**Counterexample, gotchas, and operational comparison.** Generated code, feature-gated paths, expected panics in tests, intentionally cloned small data, flaky stress tests, and one design issue producing several diagnostics. Good: track every public API change requiring caller revision and every cancellation regression. Bad: advertise fewer compile attempts as idiomaticity. Borderline: review-comment reduction is useful only beside defect and compatibility evidence.

**Verification, evidence, and uncertainty.** Publish definitions, crate and versions, raw counts and denominators, exclusions, fixtures, feature matrix, failures, uncertainty, owner, and feedback-to-change trace. The local review questions directly support obligation-focused signals and the seed preserves workflow-efficiency context. No repository baseline or causal dataset accompanies the corpus.

**Second-order consequence.** The strongest metric is fewer surprises at boundaries: callers, errors, tasks, and features should behave as the design packet predicts.

**Revision decision.** Replace compile-centric interpretation with API, failure, concurrency, architecture, compatibility, docs, and evidence signals while retaining efficiency as secondary.

**Retained seed evidence.** The compile-and-review indicator, hidden-tradeoff failure signal, and refresh cadence remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Leading indicator: compile attempts and review comments decrease because the API shape is explicit before implementation.
Failure signal: the reference hides ownership or error tradeoffs behind generic guidance.
Review cadence: Re-run the verifier after every generated-reference edit and refresh external sources when public APIs, docs, or tooling releases change.

## Completeness Checklist

**Decision supported.** This section helps decide what evidence must be present before this Rust idioms reference can govern an implementation or review decision. The seed verifies seven document-shape elements but does not prove that the reference covers API callers, invalid states, conversions, typed failures, async cancellation, bounded work, dependency direction, compatibility, documentation, or measured claims.

**Recommended default and causal basis.** Require traceable source boundaries, named callers and invariants, an API and ownership model, explicit errors and effects, async lifecycle controls, architecture placement, MSRV and feature assumptions, happy/error/edge tests, documentation evidence, measured performance claims, and an adjacent route for excluded concerns. A structurally complete document can still conceal the exact contracts that make an apparently idiomatic Rust choice correct or dangerous.

**Operating conditions and assumptions.** Every checklist item points to a section, code site, test, command, or reviewer decision and distinguishes mandatory evidence from optional reinforcement. Apply the full gate to public, concurrent, cross-layer, or measured changes; scale it down explicitly for private syntax-preserving edits.

**Failure boundary and alternatives.** Completion is inferred from section presence, Clippy cleanliness, compilation, or a polished example while panic paths, cancellation, public compatibility, or unsupported claims remain unexamined. Bounded alternatives and recoveries: use a reduced private-change checklist for local refactors, a concurrency extension for workers, a semver and feature matrix for libraries, or a dedicated unsafe and security review for higher-risk code.

**Counterexample, gotchas, and operational comparison.** Macro-generated APIs, doctest-only compilation contexts, hidden default features, transitive runtime dependencies, test-only panics, cancellation after partial effects, and downstream callers outside the workspace. A complete parser review names borrowed inputs, validated owned outputs, TryFrom errors, callers, property tests, docs, MSRV, and rollback; a compile-only review remains incomplete even when formatting and lints pass.

**Verification, evidence, and uncertainty.** Map every checklist item to evidence, run the repository-adapted fmt, lint, test, build, feature, MSRV, documentation, integration, stress, and benchmark gates that apply, then record exclusions and ownership for follow-up. The local checklist directly supports these API, error, async, ownership, architecture, testing, tooling, documentation, observability, and review dimensions. The corpus does not define one universal command matrix, minimum supported Rust version, feature set, security model, or production service-level objective.

**Second-order consequence.** Completeness is closure over obligations and failure paths, not the number of headings or commands.

**Revision decision.** Replace the seven generic document checks with an evidence-indexed completion gate spanning source, behavior, lifecycle, compatibility, verification, and routing.

**Retained seed evidence.** The role scenario, decision guide, hierarchy, artifact, examples, metrics, and adjacent-routing checks remain preserved as the document-level subset. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- The role scenario names the user, starting state, decision, and trigger for Rust Idioms Checklist Patterns.
- The decision guide includes Adopt when, Adapt when, Avoid when, and Cost of being wrong.
- The local corpus hierarchy identifies canonical and supporting sources or gives a confidence warning.
- The theme specific artifact is concrete enough to review without reading every mapped source.
- The examples cover good, bad, and borderline usage.
- The metrics section names one leading indicator and one failure signal.
- The adjacent routing section points to a better reference when this one is not the right fit.

## Adjacent Reference Routing

**Decision supported.** This section helps decide when a reviewer should keep using this cross-cutting idioms checklist and when a specialized Rust reference must own the next decision. The seed correctly says to hand off backend delivery, executable specifications, and quality gates but then supplies three tautological rust, idioms, and checklist placeholders that do not identify a destination, trigger, or return condition.

**Recommended default and causal basis.** Stay here for API shape, conversions, errors, ownership, async hygiene, layering, tests, documentation, and review questions; route requirements to rust_executable_reference_maps, HTTP delivery to rust_backend_reference_routing, release checks to rust_conventions_quality_gates or rust_quality_gate_patterns, and threat or resilience controls to rust_backend_security_resilience. Routing by governing obligation prevents a broad checklist from inventing protocol, deployment, security, or executable-spec details that require their own evidence and verification.

**Operating conditions and assumptions.** The handoff names the unresolved question, target reference, evidence already gathered, expected artifact, owner, and condition for returning to this checklist. Do not route away merely because a specialized reference mentions Rust; hand off only when its narrower obligation controls the decision.

**Failure boundary and alternatives.** Routing is based only on a Rust keyword, several references appear equally valid without precedence, the target is unavailable, or the handoff discards caller and failure evidence already collected. Bounded alternatives and recoveries: combine two references under one explicit owner for cross-cutting changes, use repository documentation when it is more authoritative, escalate novel unsafe or FFI work, or record a provisional route with an expiry when no exact reference exists.

**Counterexample, gotchas, and operational comparison.** Backend code also needs ordinary idioms, quality gates cannot choose an API, executable requirements do not prove runtime safety, security guidance can conflict with ergonomics, and file names may exist while their evidence is stale. Good: route an Axum timeout and idempotency decision to backend reliability while retaining this checklist for typed errors and cancellation. Bad: use this checklist as a deployment runbook. Borderline: pair quality-gate and idiom references for a public crate release with one owner and separate pass criteria.

**Verification, evidence, and uncertainty.** Confirm the destination exists, inspect its scope and freshness, state the exact unresolved obligation, preserve source labels and current evidence, run the target gate, and return here to recheck API, ownership, errors, layering, tests, and docs. The seed explicitly identifies backend, executable, and quality-gate pivots, and the local reference inventory supplies named destinations for those scopes. File inventory alone does not prove every adjacent reference is current, complete, or canonical for a particular repository.

**Second-order consequence.** Routing is an evidence-preserving state transition rather than a link list; ownership and return criteria matter as much as destination names.

**Revision decision.** Replace generic adjacent labels with a trigger-to-destination table, precedence rules, handoff payload, unavailable-route fallback, and return checklist.

**Retained seed evidence.** The original guidance to route HTTP delivery, executable specifications, and review gates remains the routing backbone. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Adjacent reference guidance: Use backend, executable, or quality-gate Rust references when the question shifts to HTTP delivery, specs, or review gates.
Adjacent reference 1: consider the rust adjacent reference when the current task pivots away from rust idioms checklist patterns.
Adjacent reference 2: consider the idioms adjacent reference when the current task pivots away from rust idioms checklist patterns.
Adjacent reference 3: consider the checklist adjacent reference when the current task pivots away from rust idioms checklist patterns.

## Workload Model

**Decision supported.** This section helps decide how to bound a Rust idiom review so its callers, invariants, ownership paths, feature combinations, and verification remain inspectable. The seed usefully frames systems implementation, CLI or service hardening, async review, one bounded workspace slice, and an integration path, but its fixed 100-symbol ceiling is ungrounded and its source-pressure row truncates the local checklist.

**Recommended default and causal basis.** Scope work by contract crossings: affected crates and public items, downstream callers, state transitions, error variants, spawned tasks, locks and channels, external effects, dependency-layer changes, feature sets, MSRV targets, documentation examples, and required integration paths. Review cost grows faster with ownership, concurrency, compatibility, and effect boundaries than with raw symbol or line counts.

**Operating conditions and assumptions.** The workload packet states repository, commit, crate graph, changed contracts, direct and external callers, runtime model, data scale, features, toolchains, fixtures, performance envelope, owner, and stop or split triggers. This model guides checklist application; production capacity, security scope, and migration choreography require specialized workload and service evidence.

**Failure boundary and alternatives.** A nominally small patch changes a public enum, cancellation semantics, feature resolution, or unsafe boundary; alternatively, a large mechanical private edit is split solely because it crosses an arbitrary symbol count. Bounded alternatives and recoveries: split by public API, runtime lifecycle, persistence effect, dependency layer, or feature matrix; stage a compatibility adapter; sample generated code; or commission a dedicated benchmark and architecture review.

**Counterexample, gotchas, and operational comparison.** Proc-macro expansion, build scripts, examples as downstream crates, doctest features, workspace inheritance, target-specific dependencies, hidden callers outside the repository, and tests that use a different runtime flavor. Good: bound a parser change to one crate, two public callers, one error enum, default and no-default features, docs, and an integration fixture. Bad: approve a worker refactor because it changes fewer than 100 symbols while ignoring spawned-task shutdown. Borderline: review a broad rename as one unit after proving behavior and contracts are unchanged.

**Verification, evidence, and uncertainty.** Capture a before-state inventory, map callers and dependencies, list ownership and effect boundaries, enumerate feature and toolchain matrices, identify integration and stress paths, compare observed scope with split triggers, and record residual external callers. The local checklist supports crate-bounded reasoning across API, errors, async, ownership, architecture, testing, tooling, and documentation. The corpus supplies no repository-specific threshold for symbols, crates, callers, workloads, supported targets, or reviewer capacity.

**Second-order consequence.** The useful unit of Rust review is a coherent set of obligations whose failure and rollback can be tested together, not a fixed quantity of changed syntax.

**Revision decision.** Replace the fixed symbol threshold and truncated source row with an obligation inventory, risk multipliers, explicit split triggers, workload examples, and evidence-based boundary adjustment.

**Retained seed evidence.** The systems, CLI, service, async, safety-sensitive, bounded-slice, integration-path, source-check, and required-artifact intentions remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`combined_evidence_inference_note`: Treat Rust Idioms Checklist Patterns as a rust system operating reference, not as a prose summary.

| workload_dimension_name | workload_boundary_value | verification_pressure_point |
| --- | --- | --- |
| primary_usage_surface | systems implementation, CLI or service hardening, async/runtime review, and safety-sensitive refactoring where compile success is necessary but not sufficient | verify that the reference changes the next implementation or review action |
| bounded_capacity_model | one crate or bounded workspace slice, up to 100 changed symbols, and one integration path that exercises error handling | split the task or create a narrower reference when this boundary is exceeded |
| source_pressure_model | local heading signals include Rust Idioms Checklist; 1) API and Type Design; 2) Error Handling; 3) Async and Concurrency Hygiene; 4) Ownership and Memory; 5) Architecture Boundarie | compare guidance against canonical local sources before following external examples |
| artifact_pressure_model | required artifact is worked rust idioms checklist patterns example with user goal, decision point, failure mode, and verification gate | require the artifact before claiming the reference is operationally usable |

## Reliability Target

**Decision supported.** This section helps decide which measurable properties make repeated use of this Rust idioms reference dependable rather than merely consistent in format. The seed establishes source-label preservation, sampled routing accuracy, zero unsupported operational claims, and recovery clarity, but its 18-of-20 decision threshold has no baseline and the targets omit API, panic, error, cancellation, feature, documentation, and dependency outcomes.

**Recommended default and causal basis.** Treat complete evidence labeling and zero unsupported production claims as deterministic release invariants, then set baseline-relative sampled targets for correct routing, public API rework, invalid-state escape, panic exposure, error-context retention, cancellation, bounded work, feature and MSRV compatibility, dependency direction, docs, and recovery clarity. A reference can preserve labels and still recommend an API that burdens callers or an async pattern that leaks work, so reliability must cover decisions and lifecycle outcomes.

**Operating conditions and assumptions.** Each target defines population, numerator, denominator, repository and toolchain context, feature set, reviewer rubric, observation window, owner, alert threshold, corrective action, and tolerated uncertainty. These targets assess use of the idioms reference; service availability, security assurance, and business correctness need dedicated objectives and telemetry.

**Failure boundary and alternatives.** A tiny convenience sample produces a percentage, reviewers know the expected answer, several findings share one root cause, deterministic checks are averaged with judgment calls, or a target lacks a response when breached. Bounded alternatives and recoveries: use full enumeration for small batches, blinded double review for ambiguous routes, incident and regression sampling for rare failures, or qualitative case review until enough comparable observations exist.

**Counterexample, gotchas, and operational comparison.** Feature-gated items omitted from samples, external callers unavailable, expected test panics, intentionally detached tasks, platform-specific dependencies, doctest environments, and review disagreements counted as reference failures. Good: require every operational claim to cite evidence and separately baseline sampled route agreement with disagreement review. Bad: claim 90 percent decision accuracy from 18 selected easy cases. Borderline: use a provisional threshold only when the sample protocol, uncertainty, and recalibration date are explicit.

**Verification, evidence, and uncertainty.** Run structural label and claim scans over the complete artifact, sample decisions from a declared population, capture independent verdicts and disagreements, replay representative error and cancellation fixtures, exercise feature and MSRV matrices, and trace every breach to an owner and correction. The seed directly supports evidence, routing, unsupported-claim, and recovery targets, while the local checklist supports the additional API, error, runtime, architecture, compatibility, test, and documentation dimensions. No historical defect rate, reviewer-agreement baseline, confidence interval, production incident sample, or repository service target accompanies the corpus.

**Second-order consequence.** Reference reliability is the probability that its advice preserves the governing contract under realistic callers and failure paths, not the proportion of rows that look complete.

**Revision decision.** Retain mechanically auditable source and claim invariants, replace the fixed sampled threshold with a baseline-relative protocol, and add obligation-specific outcome targets with breach responses.

**Retained seed evidence.** Source-boundary preservation, decision-route sampling, unsupported-claim rejection, and explicit recovery paths remain the four top-level reliability concerns. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| reliability_target_name | measurable_threshold_value | evidence_collection_method |
| --- | --- | --- |
| source_boundary_preservation | 100 percent of recommendations keep local, external, and inference boundaries visible | review generated text for the three evidence labels before reuse |
| decision_accuracy_sample | at least 18 of 20 sampled uses route to the correct adopt, adapt, avoid, or adjacent-reference decision | sample downstream tasks and record reviewer verdicts |
| unsupported_claim_rate | zero unsupported production, security, latency, or reliability claims in final guidance | reject claims without source row, explicit inference note, or verification method |
| recovery_path_clarity | every avoid or failure case names the rollback, escalation, or next-reference route | inspect failure-mode and adjacent-routing sections together |

## Failure Mode Table

**Decision supported.** This section helps decide how to recognize, contain, and recover from the distinct ways Rust code can look idiomatic while violating a caller, failure, lifecycle, architecture, or evidence contract. The seed names source drift, generic advice, runtime contention, and production panic, but it omits overgeneralized APIs, representable invalid states, lost error context, blocking async work, locks across await, unbounded queues, missing cancellation, clone-heavy hot paths, layer inversion, ornamental traits, feature regressions, stale docs, and unmeasured optimization.

**Recommended default and causal basis.** Maintain a failure registry with trigger, causal mechanism, observable symptom, detection gate, immediate containment, corrective pattern, rollback, owner, residual risk, and adjacent-reference escalation for every in-scope failure class. Different symptoms can share a cause and similar symptoms can require opposite fixes; causal classification prevents reflexive rewrites such as adding traits, clones, retries, or locks.

**Operating conditions and assumptions.** Rows use observable conditions, name the obligation at risk, distinguish prevention from detection, preserve data and cancellation semantics during recovery, and have at least one executable or reviewable check. The registry covers idiom and review failures; exploit analysis, distributed consistency, database recovery, and service incident response need narrower evidence.

**Failure boundary and alternatives.** Warnings are style slogans, mitigation repeats the preferred idiom without diagnosis, rollback cannot restore public compatibility, concurrency tests omit shutdown, or the table treats compiler success as evidence that runtime and caller contracts hold. Bounded alternatives and recoveries: route unsafe and FFI hazards to specialized review, route protocol and persistence failures to backend references, create a repository incident row for novel failures, or mark an uncertain mechanism pending instrumentation.

**Counterexample, gotchas, and operational comparison.** Panic hooks hiding abort behavior, locks nested through helper calls, bounded queues with unbounded producers elsewhere, cancellation after partial writes, error conversion erasing source chains, feature-unified CI, and external callers absent from tests. Good: classify a worker hang as a lock held across await, prove it with task dumps, move blocking work, add cancellation and a stress fixture, then test rollback. Bad: prescribe Arc everywhere. Borderline: retain a clone when it removes a lifetime hazard and measured bounded cost is acceptable.

**Verification, evidence, and uncertainty.** For each row construct or replay the smallest failing fixture, observe the stated symptom, confirm the causal mechanism, apply containment, run happy/error/edge plus stress or feature checks, test rollback, and record whether residual risk requires routing. The local checklist explicitly supplies the omitted API, error, async, ownership, architecture, testing, tooling, documentation, and observability failure classes. The corpus does not establish their frequency, severity distribution, repository-specific incident history, or security impact.

**Second-order consequence.** The most dangerous idiom failures are hidden contract transfers, where convenience in one function pushes panic, ownership, queueing, or compatibility cost onto callers and operators.

**Revision decision.** Expand four broad rows into a causal registry spanning API, state, error, async, ownership, architecture, compatibility, testing, docs, and unsupported claims, with detection and rollback columns.

**Retained seed evidence.** Source drift, generic unverified advice, concurrent contention, and unchecked panic remain preserved as high-priority rows. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| failure_mode_name | likely_trigger_condition | required_mitigation_action |
| --- | --- | --- |
| source drift hides truth | external or local guidance changes after the reference was written | refresh public evidence, rerun local corpus gate, and mark stale claims before reuse |
| generic advice escapes review | agent copies plausible best practices without theme-specific verification | block completion until the verification gate names concrete command, reviewer question, or metric |
| runtime contention masks failure | async tasks, locks, or shared state degrade under concurrent load | add contention benchmark, timeout budget, and structured cancellation path |
| panic path reaches production | unwrap or unchecked invariant survives compile and unit tests | require panic audit, Result propagation, and integration failure fixture |

## Retry Backpressure Guidance

**Decision supported.** This section helps decide when a failed evidence, test, tool, or runtime operation may be retried and what mechanism prevents retries from hiding deterministic defects or amplifying load. The seed usefully limits stale-evidence refresh, stops work on red gates, checkpoints long workflows, and assigns one file owner, but it conflates reference-generation retries with application retries and leaves idempotency, attempt budgets, deadlines, queue capacity, cancellation, and overload evidence implicit.

**Recommended default and causal basis.** Classify the failure first; replay a flaky observation only under a bounded diagnostic budget, refresh stale evidence once, fix deterministic code or assertion failures instead of retrying, and route production retry policy to a backend reference that can prove idempotency, deadlines, capacity, cancellation, and service-specific limits. Retry converts time and capacity into another chance of success, so an unclassified retry can duplicate effects, lengthen shutdown, saturate a queue, or make a persistent defect look intermittent.

**Operating conditions and assumptions.** The policy records failure class, retryable operation, idempotency basis, maximum attempts, total deadline, delay strategy, queue capacity, cancellation behavior, telemetry, terminal error, owner, and escalation or rollback. This section governs reference workflow and generic Rust lifecycle hygiene; production network, database, and queue retries require protocol-specific reliability evidence.

**Failure boundary and alternatives.** Compiler errors, panics, invariant violations, deterministic test assertions, authorization failures, or schema incompatibilities are retried; a side effect lacks a deduplication key; or new work enters faster than bounded consumers can drain it. Bounded alternatives and recoveries: fail fast with typed context, quarantine a flaky test, pause producers, shed optional work, reduce concurrency, use a dead-letter or manual review path, roll back a change, or narrow the evidence question.

**Counterexample, gotchas, and operational comparison.** Timeouts after a successful remote effect, nested library and application retries, cancellation between effect and acknowledgment, jitter that exceeds a parent deadline, bounded channel capacity without bounded producers, and retries that erase the first error chain. Good: rerun one suspected timing-sensitive stress fixture with seed and scheduler details, then quarantine and investigate if it diverges. Bad: loop cargo test until green. Borderline: retry an idempotent read within a parent deadline only after a specialized backend policy defines attempts and overload behavior.

**Verification, evidence, and uncertainty.** Inject each classified failure, count attempts and elapsed budget, observe queue depth and cancellation, prove duplicate effects are prevented where relevant, preserve first and terminal errors, test overload and shutdown, and confirm deterministic failures stop immediately. The seed directly supports classification, one bounded refresh, red-gate backpressure, checkpointing, and ownership; the local checklist supports bounded channels, cancellation, stress testing, and explicit errors. The corpus does not define transport semantics, idempotency storage, service latency budgets, queue capacities, retry intervals, or production overload targets.

**Second-order consequence.** Backpressure is part of correctness because it preserves the system's ability to reject and explain work before resource exhaustion destroys the evidence needed for diagnosis.

**Revision decision.** Separate evidence replay from runtime retry, add a retryability decision record and stop conditions, and connect workflow backpressure to bounded channels, cancellation, overload telemetry, and handoff rules.

**Retained seed evidence.** Transient-versus-stale classification, one bounded evidence refresh, stopping new work on red gates, frequent checkpoints, sole ownership, and merge-time path checks remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- Retry only after the failed verification signal is classified as transient, stale evidence, missing context, or true contradiction.
- Use one bounded retry for stale external evidence refresh, then escalate to a human or a narrower source-specific reference.
- Apply backpressure by stopping new generation or implementation work when source coverage, critique coverage, or verification gates are red.
- For long-running agent workflows, checkpoint after each batch and re-read the current spec before any broad rewrite, commit, push, or destructive operation.
- For distributed execution, assign one owner per generated file or theme batch and require merge-time verification of exact path, heading, and evidence-boundary invariants.

## Observability Checklist

**Decision supported.** This section helps decide which review and runtime observations are needed to confirm that the chosen Rust idioms preserve the documented API, failure, ownership, and lifecycle model. The seed records sources, proof commands, latency percentiles, errors, timeouts, retries, saturation, rollback triggers, and compact journal evidence, but it does not connect telemetry to Rust callers, typed errors, spawned-task ownership, cancellation, queue capacity, lock contention, features, toolchains, or secret handling.

**Recommended default and causal basis.** Record artifact and revision identity, source boundary, requirement and caller, toolchain and target, features, command and fixture, typed error category and source chain, task parentage and shutdown result, queue depth and capacity, lock contention where relevant, latency and allocation only for claimed paths, rollback trigger, owner, and redacted unresolved risk. Observability is useful when an event can be mapped back to a design obligation; raw logs or aggregate timing cannot reveal a detached task, erased error cause, wrong feature set, or hidden caller contract.

**Operating conditions and assumptions.** Fields have stable names and units, cardinality is bounded, correlation follows one operation across effects, cancellation and terminal states are distinguishable, sensitive inputs are redacted, and dashboards or journal summaries drive a named action. Libraries should not impose service telemetry policy, and this checklist does not replace privacy review, incident response, or production SLO design.

**Failure boundary and alternatives.** Debug strings become the schema, payloads or secrets are logged, task IDs have no parent, error counts merge causes, percentiles omit workload and sample size, queue saturation lacks capacity, or telemetry exists only on the happy path. Bounded alternatives and recoveries: use test assertions and structured command summaries for libraries, tracing spans for services, bounded event samples for high-volume paths, or a temporary diagnostic build with an expiry and removal gate.

**Counterexample, gotchas, and operational comparison.** Display versus Debug leaking data, span guards crossing await unexpectedly, sampling away rare terminal failures, panic paths bypassing normal events, metrics labels built from user input, feature-gated instrumentation, and clock distortion in short benchmarks. Good: a worker span records parent task, bounded queue occupancy, typed terminal result, cancellation reason, toolchain, and feature set without payload contents. Bad: log every request body and report average latency. Borderline: a library emits no runtime telemetry but supplies executable examples, typed errors, and caller-visible diagnostics.

**Verification, evidence, and uncertainty.** Exercise happy, error, timeout, cancellation, saturation, panic-containment, feature, and rollback paths; inspect field presence, units, cardinality, correlation, redaction, and terminal state; then prove each alert or journal signal has an owner and response. The local checklist explicitly recommends structured tracing at key boundaries and forbids secrets, while the seed supplies source, command, latency, error, timeout, retry, saturation, rollback, and compact-audit signals. The corpus does not select a tracing implementation, metric backend, sampling rate, retention policy, privacy classification, or service alert threshold.

**Second-order consequence.** Good Rust observability is an executable shadow of ownership and error design: it should make task lifetime, resource pressure, and failure propagation visible without exposing domain data.

**Revision decision.** Turn the generic list into an obligation-linked telemetry and journal schema with Rust runtime fields, redaction rules, path coverage, cardinality limits, and response ownership.

**Retained seed evidence.** Loaded and skipped sources, exact proof, latency and reviewer time, failures and saturation, rollback, leading and failure signals, and compact audit summaries remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- Record which local sources were loaded and which were intentionally skipped.
- Record the exact verification command, reviewer question, or rendered artifact used as proof.
- Record p50/p95/p99 latency or reviewer-time measurements when the reference affects runtime or workflow speed.
- Capture error count, timeout count, retry count, saturation signal, and rollback trigger.
- Record leading indicator and failure signal from this file in the coverage manifest or journal when the reference drives real work.
- Keep audit evidence small enough to review: command output summary, changed-file list, and unresolved-risk notes are preferred over raw transcript dumps.

## Performance Verification Method

**Decision supported.** This section helps decide how to verify a Rust performance claim without turning clone avoidance, iterators, allocations, async primitives, or one local timing into cargo-cult rules. The seed properly requests memory, panic, concurrency, workload, tool-call, latency, and reviewer-decision evidence, but its universal local p95 below 10 ms is unsupported and mixes runtime performance with documentation workflow efficiency.

**Recommended default and causal basis.** Start from a named requirement and representative operation, define workload and data distribution, baseline and comparison, toolchain and target, build profile and features, runtime topology, warmup and repetitions, latency or throughput statistic, allocation or memory metric when relevant, correctness guard, variance, regression threshold, and reproducible command. Rust optimizations change ownership and abstraction costs, while compiler settings, allocator, CPU scheduling, runtime flavor, feature resolution, and input shape can dominate measurements.

**Operating conditions and assumptions.** The benchmark isolates the claimed path while retaining realistic callers, verifies output and errors, reports raw samples or distribution summaries, compares like-for-like builds, discloses noise, and links a regression to a decision or rollback. Microbenchmarks support only the measured operation and environment; production capacity, distributed latency, cost, and service objectives need system-level evidence.

**Failure boundary and alternatives.** A debug build supports a production claim, averages hide tails, one run supplies precision, dead-code elimination removes work, fixtures exclude errors or contention, allocations are inferred from clones, or faster code weakens cancellation and readability without requirement value. Bounded alternatives and recoveries: use complexity and allocation inspection before timing, profile end to end for system bottlenecks, run a contention or soak fixture for async claims, retain clearer code under an explicit exception, or defer optimization until a measured requirement exists.

**Counterexample, gotchas, and operational comparison.** Incremental build effects, thermal and frequency scaling, benchmark harness overhead, async runtime startup, allocator caching, unstable hash order, unrealistic cache warmth, feature-unified dependencies, and measurement code changing scheduling. Good: compare parser versions in release mode on declared input distributions, assert identical typed results, report variance and allocations, and keep the simpler version if the requirement is already met. Bad: ban clone because one microbenchmark is faster. Borderline: accept a documented local benchmark only as exploratory evidence pending representative target hardware.

**Verification, evidence, and uncertainty.** Freeze repository revision and environment, run correctness tests first, execute repeated baseline and candidate measurements, inspect distributions and profiles, test error and concurrent paths where claimed, reproduce on a second run or environment, and record decision, uncertainty, and rollback threshold. The local checklist limits performance tests to stated constraints and advises borrowing over cloning only on hot paths, while the seed identifies memory, panic, concurrency, workload, latency, and reviewer-efficiency evidence. The corpus provides no valid universal latency, allocation, throughput, hardware, variance, or regression threshold.

**Second-order consequence.** Performance evidence is strongest when it can reject an optimization as unnecessary or contract-damaging, not only when it can celebrate a faster number.

**Revision decision.** Remove the universal 10 ms gate, separate code-runtime and review-workflow metrics, and add a reproducible claim-to-benchmark protocol with correctness, variance, alternative, and rollback checks.

**Retained seed evidence.** Memory, panic, concurrency, explicit exceptions, compile-and-review indicators, ownership and error failure signals, measurement-packet fields, reviewer actionability, and stop conditions remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Performance method: require memory, panic, and concurrency evidence; hot-path operations need p95 under 10 ms locally or an explicit benchmark exception.
Leading indicator to measure: compile attempts and review comments decrease because the API shape is explicit before implementation.
Failure signal to watch: the reference hides ownership or error tradeoffs behind generic guidance.
Required measurement packet: capture input size, source count, tool-call count, wall-clock time, p50/p95/p99 latency where runtime applies, and reviewer decision time where documentation applies.
Pass condition: the reference must let a reviewer identify the correct next action, verification gate, and stop condition without reading unrelated files.
Fail condition: the reference encourages implementation before the workload, reliability target, and failure-mode table are explicit.

## Scale Boundary Statement

**Decision supported.** This section helps decide when this checklist can still coordinate an idiom decision and when system, migration, security, or organizational scale requires decomposition and specialized ownership. The seed correctly stops at multiple systems, unbounded discovery, production traffic without an SLO, distributed file ownership, context drift, and an unnarrowed large codebase, but saying more than one ownership boundary is automatically out of scope would exclude ordinary useful Rust reviews.

**Recommended default and causal basis.** Continue while one owner can enumerate affected contracts and verify their callers, errors, task lifecycles, dependencies, features, and rollback; split or route when independent services or runtimes, unsafe or FFI boundaries, public ecosystem compatibility, cross-team migrations, unbounded source graphs, production SLOs, security threats, or irreducible feature matrices prevent that closure. Scale becomes unsafe when evidence and rollback no longer fit one coherent obligation set, not merely when code contains several borrows, tasks, modules, or ownership transfers.

**Operating conditions and assumptions.** The boundary review inventories systems, crates, teams, public consumers, runtime domains, effects, data migrations, feature and target matrices, security classification, SLOs, verification owners, merge order, rollback units, and evidence freshness. This checklist may remain one input after routing, but it must not claim ownership of deployment choreography, distributed consistency, threat modeling, or organization-wide compatibility.

**Failure boundary and alternatives.** A monolithic packet hides independent failure domains, parallel owners edit the same artifact, a split severs caller-to-callee evidence, production advice proceeds without traffic objectives, or decomposition is triggered solely by line count. Bounded alternatives and recoveries: split by service, runtime, public contract, migration phase, feature family, threat boundary, or rollback unit; introduce adapters; stage dual compatibility; use graph-based narrowing; or appoint an integration verifier across owned slices.

**Counterexample, gotchas, and operational comparison.** Shared types coupling nominally separate crates, build scripts crossing layers, feature unification, generated clients, external consumers, database migrations that outlive binaries, cancellation crossing process boundaries, and rollback order differing from deployment order. Good: keep a multi-module library refactor here when one owner can enumerate callers and run every feature and integration gate. Bad: use the checklist alone to redesign two services under live traffic. Borderline: split a workspace migration by crate but retain one compatibility matrix and integration owner.

**Verification, evidence, and uncertainty.** Construct the system and ownership inventory, prove each slice has complete inputs and outputs, assign one writer and verifier, define merge and rollback order, run cross-slice integration checks, confirm specialized SLO or security evidence, and re-evaluate the boundary after scope changes. The seed directly supports system, source, traffic, distributed-ownership, context, and graph-narrowing boundaries, while the local checklist supports layer and testability decomposition. The corpus provides no universal crate count, team count, feature cardinality, traffic volume, or risk threshold at which decomposition becomes mandatory.

**Second-order consequence.** The governing scale metric is evidence closure: a change is too large for this reference when no reviewer can connect every recommendation to its affected contract, failure path, and recovery unit.

**Revision decision.** Replace the one-ownership-boundary cutoff with evidence-closure criteria, enumerate escalation triggers and decomposition axes, and add integration ownership plus re-entry checks.

**Retained seed evidence.** Multiple systems, unbounded discovery, missing production SLOs, sole file ownership, merge checkpoints, context-drift control, and dependency or source-graph narrowing remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Rust Idioms Checklist Patterns stops being sufficient when the task spans multiple independent systems, more than one ownership boundary, unbounded source discovery, or production traffic without an explicit SLO.
Under distributed execution, split work by theme file and preserve one verification owner per split; do not let parallel agents rewrite the same reference without a merge checkpoint.
Under long-running agent workflows, treat context drift as a reliability failure: checkpoint state, summarize open risks, and verify against the spec before continuing.
Under large-codebase scale, require dependency or source-graph narrowing before applying this reference; a source list without ranked canonical guidance is not enough.

## Future Refresh Search Queries

**Decision supported.** This section helps decide which future searches can detect meaningful drift in the evidence behind this checklist without treating search rank or recent snippets as authority. The seed offers generic official-documentation, GitHub-example, and release-note searches for the title phrase, which is unlikely to find authoritative updates about API design, errors, async cancellation, channels, MSRV, features, Clippy, rustdoc, or framework boundaries.

**Recommended default and causal basis.** Search official Rust API and Cargo guidance by exact obligation, official Tokio guidance for blocking, cancellation, locks, and channels, framework documentation only for framework-specific boundaries, release and migration notes for behavior changes, and repository history for concrete versioned examples. Narrow obligation-plus-source queries reduce terminology noise and make each result comparable to a specific recommendation, test, compatibility assumption, or routing decision.

**Operating conditions and assumptions.** Every query records purpose, preferred domain or repository, version and date, expected evidence type, affected section, inclusion criteria, conflicting result, and refresh decision. Queries discover candidate evidence; they do not establish correctness, repository fit, production safety, or permission to override stronger local contracts.

**Failure boundary and alternatives.** The title phrase is searched verbatim, snippets substitute for opened sources, blog recency outranks canonical documentation, examples omit version and features, or a new recommendation is copied without caller and failure verification. Bounded alternatives and recoveries: inspect local dependency documentation and changelogs offline, compare pinned source versions, use repository issue or pull-request history for ambiguous behavior, or defer a claim and retain the current uncertainty label.

**Counterexample, gotchas, and operational comparison.** Stable versus nightly Rust differences, Tokio major-version drift, Axum examples on unreleased branches, Cargo feature resolver changes, MSRV policy separate from compiler capability, deprecated Clippy lints, and search results that collapse distinct async runtimes. Good: query official Tokio documentation for lock guards across await and map the result to the async-hygiene row with version and feature context. Bad: search best Rust idioms 2026 and promote the first list. Borderline: use a maintained repository example as supporting evidence only after matching its dependency versions and runtime assumptions.

**Verification, evidence, and uncertainty.** Open and read the complete authoritative source during the future refresh, record retrieval date and version, compare the exact claim with local corpus text, reproduce any code example under declared features and toolchain, document conflicts, and rerun affected gates before revising. The seed requires official docs, repository examples, and release notes, and the local source map already identifies Rust, Tokio, and Axum evidence families. No web refresh was performed for this evolution, so current public availability, versions, redirects, and wording remain unverified.

**Second-order consequence.** A refresh query is valuable only when it is attached in advance to a decision that could change and an executable check that can falsify the new guidance.

**Revision decision.** Replace three title-only searches with an obligation-indexed query matrix and a capture, conflict, reproduction, and acceptance protocol.

**Retained seed evidence.** Official documentation, GitHub repository examples, and release or migration notes remain the three retained search families. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| search_query_label_name | search_query_text_value |
| --- | --- |
| `official_docs_query_phrase` | rust idioms checklist patterns official documentation best practices |
| `github_repository_query_phrase` | rust idioms checklist patterns GitHub repository examples |
| `release_notes_query_phrase` | rust idioms checklist patterns changelog release notes migration |

## Evidence Boundary Notes

**Decision supported.** This section helps decide how a reader can distinguish preserved evidence from interpretation, reproduce its provenance, and know when a Rust recommendation must be reverified. The seed defines local-corpus, external-research, and combined-inference labels but gives no claim-level locator, provenance status, conflict rule, confidence update, freshness rule, or treatment for identical archived copies and unverified public links.

**Recommended default and causal basis.** Attach each consequential claim to an evidence ledger entry containing boundary label, exact path or URL, heading or locator, archive or version identity, content hash when local, retrieval status, faithful paraphrase, inference step, confidence, contradiction, verification gate, owner, and refresh trigger. Source labels alone do not prevent a paraphrase from becoming broader than its source or an archived duplicate from being mistaken for independent corroboration.

**Operating conditions and assumptions.** Local facts can be traced to bytes and headings, external facts identify whether the complete source was actually read, inferences name their premises and counterconditions, conflicting evidence remains visible, and operational claims have a test or explicit unknown. This ledger establishes traceability for this artifact, not legal provenance, security assurance, upstream endorsement, or universal Rust correctness.

**Failure boundary and alternatives.** Two byte-identical copies are counted as two authorities, a retained URL is described as current without opening it, an inference inherits the certainty of a source fact, a generic best practice receives local status, or a benchmark threshold appears without data. Bounded alternatives and recoveries: downgrade a claim to provisional guidance, quote only a small necessary excerpt with context, preserve contradictory rows pending resolution, route specialized claims, or omit advice that cannot be bounded and verified.

**Counterexample, gotchas, and operational comparison.** Archive paths implying canonical ownership, copied publication dates, redirects and mutable documentation, versionless main-branch examples, identical text with different metadata, local policy overriding ecosystem convention, and hashes proving identity but not correctness. Good: state that the two archived checklist copies are byte-identical at SHA-256 65be5f14e72be078ec7cdbdc7857e95696d7aefe99ee6e1830ccbc2c21df1317 and count them as one textual source. Bad: call four preserved URLs freshly verified. Borderline: retain cancellation advice as combined inference until repository runtime behavior is tested.

**Verification, evidence, and uncertainty.** Recompute local hashes, confirm paths and headings, compare duplicate bytes, check that every external assertion reflects its recorded retrieval status, audit inference wording for expanded scope, scan numerical claims for methods, and trace each recommendation to a gate or uncertainty note. The three boundary labels are explicit in the seed, both archived local checklist copies were completely read and found byte-identical, and their shared content directly supports the core idiom dimensions. The unavailable absolute upstream download paths were not recoverable, the four public links were intentionally not browsed, and archive identity does not establish original authorship or present-day currency.

**Second-order consequence.** Evidence discipline changes the recommendation itself: unsupported precision is removed, duplicates stop inflating confidence, and uncertainty becomes a routing or verification action.

**Revision decision.** Expand the three label definitions into claim-level provenance, duplication, conflict, confidence, freshness, verification, and refresh rules with the observed local hash and no-browse boundary.

**Retained seed evidence.** Local-corpus fact, external-research fact, and combined-evidence inference remain the required top-level labels. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- `local_corpus_sourced_fact`: statements tied directly to the local source paths above.
- `external_research_sourced_fact`: statements tied to the public URLs above.
- `combined_evidence_inference_note`: synthesis that combines local and public evidence into agent guidance.
