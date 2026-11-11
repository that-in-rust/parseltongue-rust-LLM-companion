# Rust Coder Reliability Patterns

**Decision supported.** This section helps decide what governing frame an agent adopts before writing, refactoring, or reviewing reliability-sensitive Rust. The seed no seed line states what the mapped bundle actually is, a second-generation Rust skill built as an explicit critique of rust-coder-01, keeping its process instincts and filling its named gap, failure-boundary idioms, with sixty scored patterns admitted only above an eighty-point reliability rubric floor.

**Recommended default and causal basis.** Route reliability-first Rust work through the skill's five-step workflow, map first, strongest patterns first, skeptical question throughout, boundary-matched gates, anti-patterns reread before handoff. The reference's own premise check says the first draft was too compressed, so the bundle's identity is corrective, more selective about what matters, more explicit about why patterns prevent bugs, and clearer about repo-local convention versus generally strong Rust practice.

**Operating conditions and assumptions.** The task is primarily about Rust that keeps working after the happy path, style and syntax questions deserve lighter references. This reference covers the reliability skill's doctrine for libraries, CLIs, async services, backends, and contained FFI, backend HTTP delivery specifics live with the rust backend themes.

**Failure boundary and alternatives.** Treating this theme as a generic Rust tips list flattens its architecture, the sixty patterns are rubric-ranked answers to one question, where is correctness won or lost, and reading them unranked discards the selection logic that is the file's main contribution. Bounded alternatives and recoveries: rust-coder-01 as the delivery-process companion, the SKILL.md's companion-use section assigns it executable specs, TDD-first planning, and naming discipline while this skill keeps the reliability playbook.

**Counterexample, gotchas, and operational comparison.** The skill deliberately spans more than backends, libraries, CLIs, parsers, protocol code, and FFI wrappers are in scope, which is exactly the territory the rust-web-backend delivery skill excludes, the two skills partition Rust work rather than compete for it. Good: a review that asks the skeptical question, what fails under malformed input, cancellation, backpressure, non-UTF-8 data, concurrency, or API evolution. Bad: cherry-picking the most advanced pattern as a buffet. Borderline: applying the stack to a toy script, harmless, the rubric's breadth criterion was scored for production code.

**Verification, evidence, and uncertainty.** Check that work claiming this theme names which rubric-scored patterns it applied and which gates verified them. All three mapped files read in full this session, 1610 source lines. How the 202603 pattern set has aged against current stable Rust is unknowable without external retrieval.

**Second-order consequence.** The bundle encodes a rare artifact, a self-aware revision that names what its predecessor gets right and where it is thin, so it doubles as a worked example of how to critique and supersede a skill without discarding it, guidance the corpus's own evolution mirrors.

**Revision decision.** Open with the corrective identity, rubric-floor selection, and the meta-rule, reach for the simplest pattern that preserves correctness boundaries, not the most impressive pattern in the file.

**Retained seed evidence.** The exact theme title and reliability framing remain unchanged. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

## Source Evidence Mapping Table

**Decision supported.** This section helps decide which source layer justifies each claim class reused from this theme. The seed three local rows share one usage role while the files form a deliberate three-layer funnel, a 43-line SKILL.md charter, a 26-line reference map for routing, and a 1541-line scored reference holding the actual doctrine, sized so agents load breadth only after routing.

**Recommended default and causal basis.** Cite SKILL.md for charter and workflow claims, the map for routing claims, and the reference by pattern ID for doctrine claims. The SKILL.md workflow orders the layers itself, read the reference map first, then load only the relevant sections of the full reference, so the mapping table's flat roles hide an authored loading protocol.

**Operating conditions and assumptions.** The 202603 archive paths remain stable for citation resolution. Provenance for this document's statements, the reliability reference's own source pool, S77, ripgrep patterns, and rust-analyzer analyses, is one layer further down and cited here only as the bundle reports it.

**Failure boundary and alternatives.** Loading the 1541-line reference first inverts the funnel, burning context on sixty patterns when the map's jump points would have named the three sections the task touches. Bounded alternatives and recoveries: a future mapping revision adding the upstream source-pool paths as candidate rows, giving the per-pattern source-basis blocks resolvable targets.

**Counterexample, gotchas, and operational comparison.** The four external URL rows are the rust-backend cluster's template set, tokio and axum anchors fit this theme's async sections loosely and its API-design, unsafe, and macro sections not at all, the widest template-to-content gap in the cluster. Good: citing RC2-02's source basis when reusing the parse-don't-validate doctrine. Bad: quoting a pattern's score as if the rubric were external consensus. Borderline: citing SKILL.md for the default-pattern list its step two compresses, acceptable, the reference sections hold the full versions.

**Verification, evidence, and uncertainty.** Trace disputed claims to the exact pattern ID, SKILL.md line, or map entry that contains them. Full reads this session of all three mapped files including every pattern's source-basis block. The upstream S77 and ripgrep notes were not read, their content is known only through the bundle's citations.

**Second-order consequence.** Every pattern carries a source-basis block naming its upstream evidence, S77 sections, ripgrep pattern numbers, or rust-analyzer patterns, so this bundle has per-claim provenance granularity most corpus themes lack, citations can be pattern-exact.

**Revision decision.** Annotate the rows with their funnel positions and note that per-pattern claims should cite pattern IDs, RC2-01 through RC2-60, the stable keys the file provides.

**Retained seed evidence.** All three local rows and four external rows with their column values remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| source_location_path_key | source_category_artifact_type | source_authority_confidence_level | source_usage_synthesis_role |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/SKILL.md | local_corpus_source_material | local_corpus_sourced_fact | skill entrypoint or reusable agent prompt |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/references/reference-map.md | local_corpus_source_material | local_corpus_sourced_fact | skill entrypoint or reusable agent prompt |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/references/rust-reliability-reference.md | local_corpus_source_material | local_corpus_sourced_fact | skill entrypoint or reusable agent prompt |
| https://github.com/rust-lang/api-guidelines | external_research_source_material | external_research_sourced_fact | Rust library-team API design recommendations |
| https://github.com/tokio-rs/tokio | external_research_source_material | external_research_sourced_fact | async runtime implementation and operational model |
| https://github.com/tokio-rs/axum | external_research_source_material | external_research_sourced_fact | Rust web framework implementation and design claims |
| https://docs.rs/axum/latest/axum/ | external_research_source_material | external_research_sourced_fact | published crate documentation for routing and extractors |

## Pattern Scoreboard Ranking Table

**Decision supported.** This section helps decide which reliability patterns deserve adoption effort first when several apply. The seed three corpus-process rows stand where the mapped file carries a real sixty-row scoreboard, every pattern scored out of one hundred against a published rubric, thirty-five points bug prevention, twenty misuse resistance, fifteen diagnosability, fifteen concurrency safety, fifteen breadth, with only patterns above eighty kept.

**Recommended default and causal basis.** Adopt patterns in score order within the sections the task touches, and treat anything on the not-elevated list as requiring explicit justification. The rubric makes the ranking auditable, RC2-02 newtypes at ninety-eight and RC2-20 no-blocking-across-await at ninety-seven top the table because they max the bug-prevention axis, while eighty-two-point entries like rustfmt survive on breadth, the scores encode why each pattern matters.

**Operating conditions and assumptions.** The task's boundary is identified first, scores order patterns within a boundary, they do not replace the boundary classification the decision ladder starts with. Ranking within the reliability doctrine, the corpus-process rows keep their own scope over how this document was generated.

**Failure boundary and alternatives.** Ignoring the scores flattens a deliberate priority order, an agent that reaches for RC2-50's HRTBs and GATs before RC2-02's newtypes has inverted the file's highest-confidence guidance about where correctness is won. Bounded alternatives and recoveries: frequency-of-incident ranking from a team's own defect history, complementary, the rubric predicts where bugs come from while incident history reports where they came from.

**Counterexample, gotchas, and operational comparison.** Scores are the author's rubric applied to 202603 sources, not community consensus, quoting RC2-12's ninety-seven as an ecosystem fact rather than a scored local judgment overstates its provenance. Good: starting an async service review at RC2-20, RC2-21, and RC2-22, the section's three highest scores. Bad: leading with PhantomData variance tricks on a plain CRUD crate. Borderline: adopting an eighty-three-point pattern first because the task is exactly its use case, correct, scores order defaults, tasks override.

**Verification, evidence, and uncertainty.** Check that pattern adoption sequences correlate with scores within the touched sections or document why not. The pattern map, rubric, floor rule, and not-elevated list read in full. Inter-rater stability of the rubric scores is untestable from one author's file.

**Second-order consequence.** The not-elevated list is the scoreboard's shadow and equally load-bearing, allocator overrides, ManuallyDrop tricks, lock-free blocks without verification plans, named as powerful but too easy to cargo-cult, the file ranks by teachability under LLM authorship, not by raw power.

**Revision decision.** Surface the real scoreboard's shape, the rubric, the eighty-point floor, the top tier above ninety-five, and the file's explicit list of patterns intentionally not elevated.

**Retained seed evidence.** The three scored seed rows with tier labels remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| pattern_identifier_stable_key | pattern_score_numeric_value | pattern_tier_adoption_level | pattern_failure_prevention_target |
| --- | ---: | --- | --- |
| `rust_coder_reliability_patterns` | 95 | default_adoption_pattern_tier | Source Map First: Load local rust coder material before synthesizing reliability patterns recommendations. |
| `rust_coder_reliability_patterns` | 91 | default_adoption_pattern_tier | Evidence Boundary Split: Separate local facts, external facts, and inference before giving agent instructions. |
| `rust_coder_reliability_patterns` | 88 | default_adoption_pattern_tier | Verification Gate Coupling: Attach each recommendation to at least one command, checklist, or review gate. |

## Idiomatic Thesis Synthesis Statement

**Decision supported.** This section helps decide what selection principle governs which Rust pattern any reliability task reaches for. The seed generic corpus formulas stand where the file states its own thesis twice, the selection rule, prefer patterns that reduce bug classes, misuse, and ambiguous failure behavior over patterns that merely look advanced, and the closing bet, correctness is won or lost in API commitments, error boundaries, async cancellation semantics, executable verification, and operational reality.

**Recommended default and causal basis.** Classify each task against the five fronts, then apply the simplest above-floor pattern that holds the touched boundary. The two statements bracket the file deliberately, the selection rule explains what got in, and the five-front bet explains how the ten sections group, the sixty patterns are the middle term between them.

**Operating conditions and assumptions.** The code will change under other hands, the thesis optimizes for survival under change, throwaway code weakens every clause. The thesis governs reliability pattern selection, delivery-process theses live with rust-coder-01 and backend-delivery themes.

**Failure boundary and alternatives.** Synthesizing this theme without the five fronts produces a pattern list with no map, agents cannot tell whether a task is an error-boundary problem or an API-commitment problem, the classification the whole file is organized to support. Bounded alternatives and recoveries: performance-first or ergonomics-first selection theses, both legitimate elsewhere, the file's coverage claim bounds where its reliability-first bet applies.

**Counterexample, gotchas, and operational comparison.** The thesis explicitly demotes impressiveness, so pattern-dense code that showcases GATs, typestate, and pin projection on a simple boundary violates the thesis even though every individual pattern is in the file. Good: a newtype and a typed error closing a boundary in twenty lines. Bad: a typestate protocol wrapping a two-state flag. Borderline: loom verification on moderately concurrent code, the thesis supports it when the interleaving space genuinely exceeds example tests.

**Verification, evidence, and uncertainty.** Review sampled changes for the simplest-pattern property, each applied pattern should be removable only by weakening a named boundary. The selection thesis, meta-rule, and closing thesis sections read in full. Whether the five fronts are exhaustive for reliability is the author's framing, not a proven partition.

**Second-order consequence.** The meta-rule in the LLM-usage section, reach for the simplest pattern that preserves correctness boundaries, is the thesis inverted into a stopping rule, the selection rule filters what enters the file and the meta-rule filters what leaves it into any one task, admission is maximal, application is minimal.

**Revision decision.** Restate both bracketing statements verbatim-close with their locations and connect each of the five fronts to its owning sections.

**Retained seed evidence.** The three labeled thesis statements remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`local_corpus_sourced_fact`: The local row for `rust_coder_reliability_patterns` contains 3 source path(s), which should be treated as the first retrieval surface for this theme.
`external_research_sourced_fact`: The external source map provides public documentation, repository, or ecosystem analogues where available.
`combined_evidence_inference_note`: Reliable use of Rust Coder Reliability Patterns comes from loading the local corpus first, checking public ecosystem guidance second, and converting both into verification-backed agent decisions.

## Local Corpus Source Map

**Decision supported.** This section helps decide how to consult sixteen hundred lines of reliability doctrine without loading them whole. The seed map rows list heading signals without the reading protocol the bundle ships, the reference map's use order, read the pattern map, open the sections matching the boundary you are touching, finish with high-value anti-patterns and the default reliability stack.

**Recommended default and causal basis.** Enter via the reference map's jump points, load only boundary-relevant sections, and close every consultation with the anti-pattern and stack reread. A 1541-line reference is unloadable whole into working attention, so the bundle authors built navigation as a first-class artifact, jump points naming nine entry surfaces and ripgrep commands for heading search.

**Operating conditions and assumptions.** The reader can classify their boundary, unclassifiable tasks start at the decision ladder in the LLM-usage section instead. Navigation of the three mapped files, upstream source-pool navigation is out of scope because those files are not mapped to this theme.

**Failure boundary and alternatives.** Consulting the reference without the use order produces either exhaustive reads that stall or keyword dives that miss the anti-pattern and stack rereads the protocol mandates at the end. Bounded alternatives and recoveries: direct ripgrep heading search with the shipped commands, the bundle itself endorses this for large-file navigation.

**Counterexample, gotchas, and operational comparison.** The SKILL.md heading signals in the seed row stop at Reference Use, omitting Output Expectations, the four-bullet contract that is the skill's most compact statement of what finished work looks like. Good: an async review that loads only section 3 plus the bookends. Bad: pasting the full reference into context for a newtype question. Borderline: reading the ten section headers to classify an ambiguous task, reasonable, the pattern map exists for exactly that.

**Verification, evidence, and uncertainty.** Check that consultations name their entry jump point and confirm the closing bookend reread. The reference map's 26 lines and SKILL.md's reference-use section read in full. Whether the use order is followed in practice by past consumers is unrecorded.

**Second-order consequence.** The bundle mandates bookended reading, enter through the pattern map and always exit through anti-patterns plus the default stack, so every consultation ends on rejection rules and priorities rather than on whichever pattern was read last, an ordering that resists recency-biased application.

**Revision decision.** Annotate the map with the three-step use order and the shipped search commands as the intended access paths.

**Retained seed evidence.** All three local source rows with titles, heading signals, and usage roles remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| local_source_filepath_value | local_source_title_text | local_source_heading_signals | local_source_usage_role |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/SKILL.md | rust-coder-02 | Rust Reliability; Overview; When To Use; Workflow; Companion Use; Reference Use | skill entrypoint or reusable agent prompt |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/references/reference-map.md | Reference Map | Reference Map; Jump Points; Section Search; Use Order | skill entrypoint or reusable agent prompt |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/references/rust-reliability-reference.md | rust-coder-02 | Rust Coder 02; Premise Check; What rust-coder-01 Gets Right; Where rust-coder-01 Is Thin; Source Pool; Selection Thesis | skill entrypoint or reusable agent prompt |

## External Research Source Map

**Decision supported.** This section helps decide which external targets deserve retrieval effort when this theme's claims need freshness. The seed four inherited web-framework anchors stand where this theme's real external surface is the crate vocabulary its patterns name, thiserror, anyhow, eyre, tokio, loom, proptest, expect-test, secrecy, sqlx, miette, color-eyre, parking_lot, cargo-llvm-cov, and the audit, deny, vet supply-chain trio.

**Recommended default and causal basis.** Prioritize dated fetches of the highest-scored crate-bearing patterns' crates when refreshing, and use the template rows only for their loose async-section relevance. The reliability doctrine is crate-anchored by design, patterns like RC2-12 and RC2-39 are recommendations of specific crates, so this theme's freshness risk lives in those crates' evolution, not in the axum routing docs the template rows point at.

**Operating conditions and assumptions.** Any docs fetch pins the version it observed against the 202603 anchor. External rows serve future freshness verification only, all four remained unretrieved throughout this evolution and confer no current-fact authority.

**Failure boundary and alternatives.** Treating the four rows as this theme's verification queue would freshen web-framework claims the file barely makes while the crate stack, the content most exposed to 202603 aging, has no queued fetch targets. Bounded alternatives and recoveries: reading a real workspace's Cargo.lock and advisory reports, which answers is our reliability stack current faster than documentation surveys.

**Counterexample, gotchas, and operational comparison.** The api-guidelines row is the one template anchor with genuine fit here, section 1's API-design patterns and RC2-14's non_exhaustive doctrine parallel official guideline territory, a retrieval there could confirm or date a full section. Good: a dated api-guidelines fetch checked against section 1's patterns. Bad: citing the axum docs row to freshen unsafe-containment claims. Borderline: a tokio fetch for the async section, relevant, still one crate out of the section's wider vocabulary.

**Verification, evidence, and uncertainty.** Confirm externally-flavored claims name a dated retrieval or carry the unretrieved-candidate label. Zero fetches this assignment, the crate vocabulary compiled from the sixty patterns read in full. Which named crates have had breaking or superseding releases since 202603 is unknowable without fetches.

**Second-order consequence.** The file's crate advice is consistently pattern-shaped rather than version-shaped, it recommends thiserror for the library-versus-binary error split, not any thiserror API, so most named crates could evolve considerably before the doctrine breaks, the aging surface is narrower than the crate count suggests.

**Revision decision.** Keep the rows as inherited candidates, flag the crate-vocabulary gap, and route real freshness needs through dated fetches of the named crates' repositories or docs.rs pages.

**Retained seed evidence.** All four external rows with their boundary labels remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| external_source_url_value | external_source_name_text | external_source_usage_role | evidence_boundary_label_value |
| --- | --- | --- | --- |
| https://github.com/rust-lang/api-guidelines | Rust API Guidelines | Rust library-team API design recommendations | external_research_sourced_fact |
| https://github.com/tokio-rs/tokio | Tokio repository | async runtime implementation and operational model | external_research_sourced_fact |
| https://github.com/tokio-rs/axum | Axum repository | Rust web framework implementation and design claims | external_research_sourced_fact |
| https://docs.rs/axum/latest/axum/ | Axum docs.rs | published crate documentation for routing and extractors | external_research_sourced_fact |

## Anti Pattern Registry Table

**Decision supported.** This section helps decide which Rust constructs reviewers must reject on sight in reliability-sensitive code. The seed three corpus-process rows stand where the mapped file ships a twelve-item high-value anti-pattern list to reject by default, from public APIs accepting &Vec<T> or &String and libraries returning anyhow::Error to unbounded channels in high-throughput paths, detached tasks with no lifecycle ownership, and concurrency code never modeled or stress-tested.

**Recommended default and causal basis.** Wire the greppable half into lint or CI checks and hold the judgment half as named review questions at handoff. Each rejection is a scored pattern inverted, the anyhow rejection inverts RC2-12, the lock-across-await rejection inverts RC2-20, so the list is the scoreboard's top tier restated as review vocabulary, cheap to check without reloading the patterns.

**Operating conditions and assumptions.** Reviewers know the list, an unread registry rejects nothing. Reliability rejections for Rust code under this theme, corpus-generation rejections keep the seed's three rows, and backend-specific rejections live with the backend themes.

**Failure boundary and alternatives.** Reviews without the list relitigate settled questions pattern by pattern, while reviews with it can reject on sight and cite the rejection, the file's own workflow puts the anti-pattern reread immediately before handoff for exactly this reason. Bounded alternatives and recoveries: clippy lint groups covering part of the greppable half, endorsed by RC2-34, they do not reach the judgment half or the API-evolution rejections.

**Counterexample, gotchas, and operational comparison.** The unwrap rejection is scoped, not absolute, it forbids unwrap in production paths without an explicit invariant story, so a blanket unwrap ban both overshoots the doctrine and hides the real requirement, the documented invariant. Good: blocking a PR that exports anyhow::Error from a library crate. Bad: approving a detached tokio::spawn with no shutdown story. Borderline: unwrap on a compile-time constant parse, acceptable exactly when the invariant comment says why it cannot fail.

**Verification, evidence, and uncertainty.** Check each registry row inverts a citable scored pattern and carries a working detection route. The high-value anti-patterns section and workflow step five read in full. Relative incidence of the twelve rejections in real LLM-authored Rust is unmeasured here.

**Second-order consequence.** The twelve items split into greppable and judgment classes, signatures like &Vec<T>, unwrap in production paths, and unbounded channel constructors are mechanically findable, while missing invariant stories and never-modeled concurrency need human judgment, so half the list can gate in CI and half must live in review.

**Revision decision.** Import the twelve rejections with their inverted-pattern citations and the handoff-reread rule from SKILL.md's workflow step five.

**Retained seed evidence.** The three seed process rows with their detection signals remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| anti_pattern_failure_name | failure_cause_risk_reason | safer_default_replacement_pattern | detection_signal_review_method |
| --- | --- | --- | --- |
| context_free_summary_output | agent skips local corpus and produces generic advice | source_map_first_synthesis | verify every listed local path appears in the generated file |
| unsourced_recommendation_claims | guidance appears authoritative without source boundary | evidence_boundary_split_pattern | check for local, external, and inference labels |
| unverified_agent_instruction | recommendation cannot be checked by command or review gate | verification_gate_coupling | require concrete gate in generated reference |

## Verification Gate Command Set

**Decision supported.** This section helps decide which verification gates a given Rust change must pass before handoff. The seed single document verifier stands where the bundle prescribes boundary-matched verification, SKILL.md's step four names cargo test, doctests, compile-fail tests, property or fuzz tests, clippy -D warnings, and any concurrency verification the code deserves, chosen per touched boundary rather than run as a fixed battery.

**Recommended default and causal basis.** Run clippy -D warnings and cargo test universally, and escalate to contract, property, fuzz, loom, or Miri gates as the touched boundary demands. Section 4 assigns each gate its jurisdiction, doctests for API contracts, compile-fail for misuse contracts, property tests for invariants, fuzzing for untrusted-input frontiers, loom for interleavings, so the gate mix is a function of what boundary changed.

**Operating conditions and assumptions.** Boundaries are named at review time, unlabeled changes get the universal floor plus a reviewer question about what they touched. Code verification gates for this theme's work, the seed's document verifier keeps its own corpus jurisdiction.

**Failure boundary and alternatives.** One-size gate batteries fail in both directions, running loom on everything wastes verification budget, while cargo test alone on concurrency code is exactly the passed-once superstition RC2-24 warns proves almost nothing. Bounded alternatives and recoveries: coverage-gated pipelines per RC2-58, useful as a floor for critical modules, the file marks coverage imperfect and its chase of arbitrary numbers an explicit avoid.

**Counterexample, gotchas, and operational comparison.** RC2-53 makes Miri part of the unsafe pattern itself, not an optional extra, unsafe code without Miri or equivalent verification fails the pattern it claims to follow. Good: a parser change shipping with property tests, a fuzz target, and clippy green. Bad: an atomic-ordering change verified by one passing test run. Borderline: skipping compile-fail tests on an internal-only API, defensible, misuse contracts matter most where callers are external.

**Verification, evidence, and uncertainty.** Sample merged changes for boundary declarations and confirm the matched gates ran. SKILL.md step four and the section 4 and RC2-24, RC2-53 gate doctrines read in full. The marginal defect yield of each gate class on typical services is unmeasured in this corpus.

**Second-order consequence.** The gates form a severity ladder mirroring the decision ladder's verification rung, from clippy always on, through contract gates when APIs change, to loom and Miri reserved for the concurrency and unsafe frontiers, so gate escalation is itself a signal of what kind of code the change is.

**Revision decision.** Adopt the boundary-to-gate mapping as the review contract, each change names its touched boundaries and shows the matched gates green.

**Retained seed evidence.** The seed's document verifier command block remains preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`verification_gate_command_set`: Run the repository verifier after editing this file.

```bash
python3 archive/monthly-agent-snapshots/idiomatic-references-202606/tools/verify_reference_generation.py --stage final
```

## Agent Usage Decision Guide

**Decision supported.** This section helps decide what protocol an agent follows to turn sixty patterns into one coherent implementation. The seed four generic bullets stand where the mapped file contains an explicit LLM operating manual, section 11's seven-rung decision ladder, classify the code, then choose ownership, error, concurrency, API-stability, and verification surfaces in order, with unsafe minimization as the final rung.

**Recommended default and causal basis.** Walk agents down the seven rungs in order for every reliability task, recording the choice made at each rung. The ladder exists because the file's own warning is that an LLM should not treat this as a buffet, sixty patterns invite arbitrary sampling, and the ladder converts them into forced sequential choices each with a bounded option set.

**Operating conditions and assumptions.** The task is reliability-shaped per the coverage claim, compiler internals, kernel lock-free work, and deep embedded targets are outside the file's own boundary claim. Agent behavior over this theme's material, generic dispatch lives with the routing theme and delivery-process agent rules with rust-coder-01's territory.

**Failure boundary and alternatives.** Buffet usage produces locally impressive, globally incoherent code, a GAT-heavy trait beside a stringly-typed error surface, the exact mismatch the ladder's ordered surfaces prevent. Bounded alternatives and recoveries: constraining agents to the six-step default reliability stack instead, the file's own fallback when you do not know what to prioritize, coarser but safe.

**Counterexample, gotchas, and operational comparison.** Rung one's classification list, library, binary, async service, FFI wrapper, proc-macro, or low-level component, is also the file's section selector, a misclassified task walks the right ladder through the wrong sections. Good: an agent trace showing all seven rung choices before code. Bad: an agent quoting five high-scored patterns and applying none coherently. Borderline: skipping the unsafe rung on safe-only code, correct, the rung is conditional by its own wording.

**Verification, evidence, and uncertainty.** Audit agent outputs for rung-by-rung records and reject buffet-style pattern citation without surface choices. Section 11's ladder, the meta-rule, and SKILL.md's workflow read in full. How reliably agents sustain ladder discipline across long sessions lacks measurement.

**Second-order consequence.** The ladder's rungs are ordered by irreversibility, code classification and ownership shape everything downstream while verification mix can be revised late, so the ladder front-loads the decisions that are costliest to change, a structure worth copying into any agent prompt built from this theme.

**Revision decision.** Replace the generic bullets with the seven rungs verbatim-close, plus the skeptical question from SKILL.md step three as the standing cross-check at every rung.

**Retained seed evidence.** The four seed usage bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`agent_usage_decision_guide`: Use this reference when a task mentions this theme, one of the listed local source paths, or a nearby technology/workflow surface.

- Start with the local corpus source map.
- Prefer patterns with explicit verification gates.
- Treat external sources as freshness and ecosystem checks, not replacements for local repo conventions.
- Preserve the evidence boundary labels when reusing recommendations.

## User Journey Scenario

**Decision supported.** This section helps decide which entry into this theme matches the reader's current task shape. The seed one generic engineer stands where the bundle's when-to-use section implies three distinct journeys, a designer shaping a new public API or error surface, a hardener retrofitting async cancellation and backpressure onto a running service, and a reviewer judging whether existing Rust will stay safe and diagnosable under change.

**Recommended default and causal basis.** Identify the journey before opening the reference and enter at that journey's surface rather than at the file top. The three journeys enter the reference at different sections and rungs, the designer at section 1 and the ownership rung, the hardener at section 3 and the concurrency rung, the reviewer at the anti-pattern list first, so journey identification determines the reading path.

**Operating conditions and assumptions.** The reader knows which journey they are on, hybrid tasks like review-then-fix run the reviewer entry first and the designer entry second. Journeys through reliability doctrine, backend feature journeys live with the backend themes and process journeys with rust-coder-01.

**Failure boundary and alternatives.** Documentation tuned to one journey fails the others, a reviewer sent through the full decision ladder wastes review time the anti-pattern list would have saved, and a designer starting at the anti-patterns gets rejections without the constructive patterns behind them. Bounded alternatives and recoveries: a single canonical reading order for all users, simpler to teach, it taxes every journey to subsidize none.

**Counterexample, gotchas, and operational comparison.** The hardener journey is where the skeptical question earns most, retrofits inherit invariants nobody documented, so each of its probes, cancellation, backpressure, non-UTF-8, concurrency, names a latent assumption the running system has been silently betting on. Good: a reviewer rejecting a lock-across-await in minutes from the anti-pattern list. Bad: a hardener redesigning the public API mid-incident because they entered at section 1. Borderline: a designer skimming the stack as a checklist, workable, the stack compresses but does not replace section 1's tradeoffs.

**Verification, evidence, and uncertainty.** Ask recent users which journey they ran and whether their entry surface served its tempo. The when-to-use section, section ordering, and anti-pattern placement read in full. The real frequency mix of the three journeys per team is unmeasured.

**Second-order consequence.** The reviewer journey is the bundle's cheapest high-value path, twelve rejections plus the six-step stack can be held in working memory whole, so review adoption can precede full doctrine adoption, a wedge sequence for teams that cannot absorb sixty patterns at once.

**Revision decision.** Recast the scenario as three entries with their sections and rungs, and keep the seed's role framing as the designer case.

**Retained seed evidence.** The role, starting state, decision, and trigger lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Role based opening scenario: The Rust reliability engineer is starting from a requirement that needs a safe API, explicit error model, and testable boundary and needs a reference that turns source evidence into an executable next step.
Primary user starting state: The user has a `rust_coder_reliability_patterns` task, one or more local source paths, and uncertainty about which pattern should drive implementation.
Decision being made: choosing the idiomatic ownership, async, error, and crate-boundary shape before coding.
Reference opening trigger: Open this file when the task mentions rust coder reliability patterns, any mapped local source path, or an adjacent workflow with the same failure mode.

## Decision Tradeoff Guide

**Decision supported.** This section helps decide how recurring Rust design arguments get settled under this theme. The seed template rows skip the live tensions the file adjudicates pattern by pattern, static versus dynamic dispatch as an explicit boundary decision, typed library errors versus ergonomic binary aggregation, combinator chains versus explicit matches, and interior mutability versus synchronization primitives.

**Recommended default and causal basis.** Resolve each axis by its pattern's boundary conditions, and record which clause applied when the resolution surprises a reviewer. Each tension gets a named pattern with use-when and avoid-when clauses, RC2-05, RC2-12, RC2-04, RC2-45, so the file's tradeoff method is to convert every recurring argument into a boundary condition rather than a preference.

**Operating conditions and assumptions.** The case matches a written clause, genuinely novel cases escalate rather than stretch the nearest clause. Tradeoffs within reliability doctrine, delivery-process tradeoffs and backend-stack tradeoffs live with their own themes.

**Failure boundary and alternatives.** Treating these as style preferences reopens settled tradeoffs per review, while the avoid-when clauses exist precisely to say when the favored option is wrong, generics in a plugin seam, context noise at every helper layer. Bounded alternatives and recoveries: team style guides fixing one side of each axis globally, cheaper to enforce, they discard the boundary information the clauses encode.

**Counterexample, gotchas, and operational comparison.** RC2-45's avoid-when carries an async trap in a sync-looking pattern, carrying RefCell or lock guards across suspension points, a tension that surfaces only when two axes, mutability choice and async discipline, intersect in one function. Good: a review comment citing RC2-05's avoid-when for a hot-loop dyn Trait. Bad: a blanket rule that combinators always beat match. Borderline: anyhow inside a binary's internal library crate, the RC2-12 boundary is the published API, internal crates sit near the line.

**Verification, evidence, and uncertainty.** Audit recent design arguments for clause citations versus preference assertions. The use-when and avoid-when clauses of RC2-04, RC2-05, RC2-12, and RC2-45 read in full. How often real cases fall between clauses is unmeasured here.

**Second-order consequence.** The file's avoid-when clauses are its most transferable innovation, every pattern ships with its own counter-indication, so the doctrine is self-limiting by construction, an agent can never correctly cite a pattern for a case its own avoid-when excludes.

**Revision decision.** Add the four axes with their pattern IDs and both clauses, use-when and avoid-when, per axis.

**Retained seed evidence.** The adopt, adapt, avoid, and cost-of-being-wrong rows remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| decision_option_name | when_to_choose_condition | tradeoff_cost_description | verification_question_prompt |
| --- | --- | --- | --- |
| Adopt when | local corpus and external evidence agree on the rust coder reliability patterns pattern | fastest path, but can copy stale local assumptions | Does the selected pattern appear in the canonical source and current external evidence? |
| Adapt when | local sources are strong but public ecosystem guidance has changed | preserves repo fit, but requires explicit inference notes | Did the reference label the local fact, external fact, and combined inference separately? |
| Avoid when | source evidence is thin, conflicting, or unrelated to the user journey | prevents false confidence, but may require deeper research | Is there a confidence warning or adjacent reference route? |
| Cost of being wrong | wrong rust coder reliability patterns guidance can send an agent to the wrong files, tests, or abstraction | wasted implementation loop and weaker verification | Would a reviewer know what to undo and what to inspect next? |

## Local Corpus Hierarchy

**Decision supported.** This section helps decide which file settles a question when the wrapper and the reference could both plausibly answer. The seed hierarchy rows crown the 43-line SKILL.md canonical and demote the 1541-line reliability reference to supporting, inverting the bundle's own architecture, the reference holds every scored pattern, the rubric, the anti-patterns, and the stack, while SKILL.md is the dispatch wrapper that routes into it.

**Recommended default and causal basis.** Resolve pattern-level disputes from the reference by pattern ID, and charter-level disputes, when to use the skill at all, from SKILL.md. The template assigned roles by file position, SKILL.md first, and this bundle is a clear case where position misleads, SKILL.md's own reference-use section defers to the reference for the full scored patterns, anti-patterns, and default reliability stack.

**Operating conditions and assumptions.** The bundle stays internally consistent, a reference revision that changes a pattern's clauses silently outdates SKILL.md's compressed step two. Precedence within this bundle, cross-theme precedence with rust-coder-01 and backend themes belongs to adjacent routing.

**Failure boundary and alternatives.** Taking the seed roles literally would settle doctrine disputes from the wrapper's compressed step two list, which names six defaults out of sixty patterns and none of their boundary clauses. Bounded alternatives and recoveries: treating the upstream source pool as a court of appeal, unavailable here, those files are unmapped and unread in this session.

**Counterexample, gotchas, and operational comparison.** The repo-local-versus-general section adds a second-order hierarchy, four rust-coder-01 conventions are marked repo-local while this file's content is marked generally strong Rust, so exporting the bundle's advice elsewhere requires knowing which list a claim came from. Good: settling an error-design dispute from RC2-12's clauses. Bad: quoting SKILL.md step two as the complete default list. Borderline: using the reference map's jump-point names as section authority, harmless for navigation, empty for doctrine.

**Verification, evidence, and uncertainty.** Test hierarchy claims against each file's self-declared role statements. All three files' self-descriptions and cross-references read in full. Whether future bundle revisions keep the wrapper and reference synchronized is unobserved.

**Second-order consequence.** The bundle also defines an external hierarchy seam in its companion-use section, rust-coder-01 owns process and this skill owns reliability, with an explicit combination rule when both apply, so hierarchy questions can cross bundles and still have a written answer.

**Revision decision.** Record the working hierarchy, the reliability reference canonical for doctrine, SKILL.md canonical only for charter, workflow, and companion-use rules, and the reference map as pure navigation.

**Retained seed evidence.** The three hierarchy rows and the classification vocabulary line remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Classification vocabulary includes canonical, supporting, legacy, duplicate, and conflicting source roles.

| local_source_filepath_value | corpus_hierarchy_role | heading_signal_to_convert | reviewer_question_to_answer |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/SKILL.md | canonical primary source | Rust Reliability; Overview; When To Use | What guidance, warning, or example should this source contribute to Rust Coder Reliability Patterns? |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/references/reference-map.md | supporting detail source | Reference Map; Jump Points; Section Search | What guidance, warning, or example should this source contribute to Rust Coder Reliability Patterns? |
| archive/monthly-agent-snapshots/codex-skills-202603/rust-coder-02/references/rust-reliability-reference.md | supporting detail source | Rust Coder 02; Premise Check; What rust-coder-01 Gets Right | What guidance, warning, or example should this source contribute to Rust Coder Reliability Patterns? |

## Theme Specific Artifact

**Decision supported.** This section helps decide what standing record preserves the ladder's choices for future readers of the code. The seed generic worked-example artifact misses this theme's natural object, a boundary ledger for the change at hand, one page recording the rung-one classification, the four surface choices, ownership, error, concurrency, API stability, the gates matched to each touched boundary, and any unsafe blocks with their SAFETY story.

**Recommended default and causal basis.** Require the ledger on changes that touch public APIs, error surfaces, concurrency, or unsafe, the four irreversible rungs. The decision ladder produces exactly these seven choices and the file's whole architecture assumes they are made deliberately, but nothing in the bundle persists them, so the reasoning evaporates at merge and reviewers of later changes re-derive it.

**Operating conditions and assumptions.** Changes are substantial enough to carry it, trivial fixes inherit the standing ledger of their crate. One ledger per substantial change or per crate boundary, suite manifests belong to the testing themes and dispatch cards to the entry themes.

**Failure boundary and alternatives.** Without the ledger, the next engineer cannot tell whether the crate's dyn Trait seam was an RC2-05 boundary decision or an accident, and accidental patterns get preserved with the same reverence as chosen ones. Bounded alternatives and recoveries: encoding choices as doc comments at each boundary, closer to the code, unaggregated and invisible at review time.

**Counterexample, gotchas, and operational comparison.** The unsafe inventory block must never be empty-by-omission, RC2-53's doctrine means an absent SAFETY story on present unsafe is a red ledger, absence of unsafe should be stated explicitly. Good: a PR ledger showing async service, borrowed inputs, typed errors, bounded channels, loom on the queue core. Bad: a merged concurrency change whose channel choice nobody can explain a quarter later. Borderline: one crate-level ledger amortized across small PRs, workable while the crate's boundaries genuinely hold still.

**Verification, evidence, and uncertainty.** Sample merged reliability changes for ledgers and spot-check one surface choice against the code. The decision ladder's seven rungs and RC2-53's documentation doctrine read in full. Ledger upkeep discipline across teams and churn is the untested variable.

**Second-order consequence.** The ledger converts the file's avoid-when clauses into future tripwires, recording that dyn Trait was chosen for a plugin seam tells the future optimizer exactly which condition, a hot loop appearing there, would invalidate the choice, decisions age visibly instead of silently.

**Revision decision.** Define the artifact as the boundary ledger with the seven ladder choices, matched gates, and unsafe inventory, attached to the PR or the crate's docs.

**Retained seed evidence.** The artifact field rows with completion rules and evidence hints remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Theme specific artifact: worked rust coder reliability patterns example with user goal, decision point, failure mode, and verification gate.

| artifact_field_name | artifact_completion_rule | evidence_source_hint |
| --- | --- | --- |
| user_goal_statement | state the user's concrete goal before applying Rust Coder Reliability Patterns | local corpus hierarchy plus critique findings |
| decision_boundary_rule | define the point where this reference should be used or avoided | decision tradeoff guide |
| verification_gate_rule | define the command, checklist, or review question that proves the artifact worked | verification gate command set |

## Worked Example Set

**Decision supported.** This section helps decide which exercise most efficiently builds coherent reliability-pattern composition. The seed abstract usage lines stand where this theme wants a full-ladder walkthrough, take one fallible-input feature, an ID-keyed lookup service, and walk it down all seven rungs, newtype the ID per RC2-02, type the error per RC2-12 and RC2-13, bound the channel per RC2-21, mark the enum non_exhaustive per RC2-14, then gate with doctests, a property test, and clippy.

**Recommended default and causal basis.** Run new reliability-track contributors through the walkthrough and the drill before they own boundary-touching changes. The ladder is learned by descent, each rung's choice constrains the next, the newtype shapes the error variants, the error shape decides what the doctest asserts, so a single connected walkthrough teaches the dependencies no per-pattern reading shows.

**Operating conditions and assumptions.** A practice crate exists, the walkthrough on throwaway code teaches motions but not the API-evolution stakes rung five carries. Teaching reliability composition, suite-construction walkthroughs live with the testing themes.

**Failure boundary and alternatives.** Engineers who know patterns in isolation assemble them incoherently, a beautifully typed error behind an API that takes &String, passing individual pattern checks while failing the ladder's coherence. Bounded alternatives and recoveries: studying ripgrep's patterns directly, the file's own upstream exemplar, richer and slower, without the graded structure.

**Counterexample, gotchas, and operational comparison.** The walkthrough must include one avoid-when moment, a place where the fancy option is declined, otherwise it teaches pattern application without the meta-rule's restraint, the half of the doctrine that prevents overuse. Good: a trainee ledger showing all seven rungs with one deliberate simplest-pattern choice. Bad: a training exercise that awards points per pattern used. Borderline: running the drill alone for reviewers, defensible, reviewers exercise recognition far more than construction.

**Verification, evidence, and uncertainty.** Have the trainee walk a fresh feature down the ladder unsupervised and audit the ledger. The ladder, the named patterns' clauses, and the anti-pattern list read in full. Skill retention without periodic boundary work is unmeasured.

**Second-order consequence.** The rejection drill and the walkthrough train opposite muscles, construction and recognition, and the anti-pattern list means recognition can be trained cheaply on synthetic code, twelve findable defects with citable names, a graded exercise the bundle effectively ships for free.

**Revision decision.** Anchor the section on the seven-rung walkthrough plus one rejection drill, present code violating three anti-patterns, unwrap in a production path, lock across await, unbounded channel, and have the learner find and cite all three.

**Retained seed evidence.** The good, bad, and borderline seed lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Good example: Use Rust Coder Reliability Patterns after loading the canonical source, confirming the external evidence boundary, and writing a verification gate before implementation.
Bad example: Use Rust Coder Reliability Patterns as a generic tutorial while ignoring the mapped local paths, source hierarchy, and cost of being wrong.
Borderline case: Use Rust Coder Reliability Patterns only after adding a confidence warning when local evidence is thin or conflicts with current ecosystem guidance.

## Outcome Metrics and Feedback Loops

**Decision supported.** This section helps decide which measurements prove this theme's discipline pays for itself. The seed compile-centric indicator misses what the doctrine predicts should move, boundary-defect escape rate, defects traceable to API, error, async, or unsafe boundaries reaching production, anti-pattern hit rate in review, and gate-escalation coverage, the share of concurrency and unsafe changes that actually ran their matched heavy gates.

**Recommended default and causal basis.** Tag the three event streams as they occur and review the metrics quarterly beside the escaped-defect count. Each metric tests one bundle claim in operation, escape rate tests the five-front thesis, hit rate tests whether the rejection list is being consulted, and escalation coverage tests whether boundary-matched verification survives deadline pressure.

**Operating conditions and assumptions.** Retrospectives classify defects by boundary honestly, the five fronts are the rubric that keeps hindsight classification consistent. Measuring this theme's adoption, suite-health ratios live with the testing themes and pipeline metrics with ops themes.

**Failure boundary and alternatives.** Without measurement the doctrine's cost is visible, ledgers, gates, review questions, while its benefit is not, and unmeasured disciplines lose budget negotiations to visible feature work. Bounded alternatives and recoveries: compile-attempt and review-round counts as the seed suggests, cheaper, they measure friction rather than the reliability outcomes the doctrine actually promises.

**Counterexample, gotchas, and operational comparison.** Gate-escalation coverage can be gamed by never declaring boundaries, so it must be read jointly with ledger presence, undeclared boundaries are the metric's denominator leak. Good: a quarter showing async-boundary escapes falling after RC2-20 through RC2-23 adoption. Bad: defending the ledger requirement with no data when its cost is challenged. Borderline: counting a lock-across-await caught in CI as a review hit, either stream is fine, double-counting is not.

**Verification, evidence, and uncertainty.** Confirm the three metrics exist, cite real event streams, and were consumed at the last quarterly review. The alignment between the bundle's claims and their measurable review-and-incident signatures. Healthy baselines for all three metrics await the first measured quarters.

**Second-order consequence.** Anti-pattern hit rate should rise then fall on adoption, rising as reviewers learn to see the twelve rejections, falling as authors internalize them, so the metric's healthy trajectory is a hump, and a flat zero more likely means unconsulted list than clean code.

**Revision decision.** Add the three metrics with collection points, incident retrospectives classified by boundary, review-comment tagging against the twelve rejections, and ledger audits for gate escalation.

**Retained seed evidence.** The leading indicator, failure signal, and review cadence lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Leading indicator: compile attempts and review comments decrease because the API shape is explicit before implementation.
Failure signal: the reference hides ownership or error tradeoffs behind generic guidance.
Review cadence: Re-run the verifier after every generated-reference edit and refresh external sources when public APIs, docs, or tooling releases change.

## Completeness Checklist

**Decision supported.** This section helps decide what must be confirmed before this reliability reference is declared faithful. The seed shape items never audit fidelity to the bundle's countable content, sixty patterns across ten sections, a five-part rubric summing to one hundred, an eighty-point floor, seven ladder rungs, twelve anti-patterns, six not-elevated patterns, a six-step stack, five workflow steps, and four output expectations.

**Recommended default and causal basis.** Run the count-and-value audit at every reread cycle and after any archive change touching the bundle. This theme's claims transcribe an enumerable source, so completeness reduces to counting the enumerables and spot-checking the highest-consequence values, the scores and clauses of the top-tier patterns.

**Operating conditions and assumptions.** The 202603 files remain at their paths for line-level comparison. Auditing this reference document against its source, auditing live adoption belongs to the metrics loop.

**Failure boundary and alternatives.** Count-passing paraphrase is the residual risk, an anti-pattern reworded from no locks across await to minimize locking near await keeps the count at twelve while dissolving the bright line that made the rejection checkable. Bounded alternatives and recoveries: checksumming the three files and re-auditing only on change, efficient, blind to drift introduced by this document's own future edits.

**Counterexample, gotchas, and operational comparison.** Auditing from memory of the bundle rather than the open file is the standing corpus failure mode, the pattern map makes the honest check cost minutes, leaving no excuse for the memory shortcut. Good: an audit catching a transposed score on RC2-21. Bad: declaring fidelity because all 26 headings exist. Borderline: accepting paraphrased why-it-matters prose, tolerable, IDs, scores, counts, and clause boundaries must survive exactly.

**Verification, evidence, and uncertainty.** Count the enumerables and diff the top-tier scores against the live pattern map. The fully enumerable structure of all three mapped files read in full. Whether score-level auditing needs corpus tooling remains open.

**Second-order consequence.** The pattern map is a built-in audit key, sixty IDs with scores in one table means any transcription of this theme can be diffed against a single source surface in minutes, the bundle is unusually cheap to audit for its size.

**Revision decision.** Add the count items plus an exact-value check on the top tier, RC2-02 at 98, RC2-12 and RC2-20 at 97, RC2-21 at 96, RC2-13 and RC2-53 at 95, each audit cycle.

**Retained seed evidence.** All seven seed checklist bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- The role scenario names the user, starting state, decision, and trigger for Rust Coder Reliability Patterns.
- The decision guide includes Adopt when, Adapt when, Avoid when, and Cost of being wrong.
- The local corpus hierarchy identifies canonical and supporting sources or gives a confidence warning.
- The theme specific artifact is concrete enough to review without reading every mapped source.
- The examples cover good, bad, and borderline usage.
- The metrics section names one leading indicator and one failure signal.
- The adjacent routing section points to a better reference when this one is not the right fit.

## Adjacent Reference Routing

**Decision supported.** This section helps decide where to send a Rust question this theme does not own. The seed token-split rows point nowhere while this theme's real neighbors are written into the bundle, rust-coder-01's territory for executable specs, TDD planning, and naming discipline per the companion-use rule, the rust backend themes for HTTP delivery specifics the coverage claim leaves out, and the entry themes for skill dispatch itself.

**Recommended default and causal basis.** Test each arriving question against the two written borders, and for excluded specialisms escalate with the coverage claim quoted rather than improvising. The bundle draws its own borders twice, companion-use names what rust-coder-01 owns, and the coverage claim names what nobody here owns, compiler internals, kernel lock-free systems, deep embedded no-std, advanced proc-macro internals.

**Operating conditions and assumptions.** Destination themes exist under their expected corpus names. Sending away what this theme cannot own, destination summaries are deliberately absent.

**Failure boundary and alternatives.** Keeping every Rust question here dilutes a reliability playbook into a general Rust helpdesk, while the four coverage exclusions answered confidently from this file would be authority the file explicitly disclaims. Bounded alternatives and recoveries: for the four excluded specialisms, external specialist sources labeled unretrieved, the file's boundary claim is the honest routing note.

**Counterexample, gotchas, and operational comparison.** Async questions split subtly, cancellation and backpressure doctrine lives here in section 3, while axum-specific handler and extractor mechanics belong to the backend themes, the word async alone does not decide the route. Good: routing a naming-convention dispute to rust-coder-01's territory with the companion rule cited. Bad: answering a no-std firmware question from this file's patterns. Borderline: a proc-macro hygiene question, RC2-52 covers hygiene and spans, internals beyond that are excluded by name.

**Verification, evidence, and uncertainty.** Sample recent forwarded and kept questions against the two written borders. The companion-use section and coverage claim read in full. Misroute frequency into this theme lacks measurement.

**Second-order consequence.** The companion-use rule is a combination protocol, not just a border, both skills apply to one task with this one as reliability playbook and rust-coder-01 as delivery process, so the routing decision here is sometimes both-and with assigned roles rather than either-or.

**Revision decision.** Route by the bundle's own borders, process and naming to rust-coder-01's theme, HTTP-delivery specifics to the backend themes, dispatch to entry themes, and the four excluded specialisms to external research with the exclusion noted.

**Retained seed evidence.** The three seed adjacent-reference lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Adjacent reference guidance: Use backend, executable, or quality-gate Rust references when the question shifts to HTTP delivery, specs, or review gates.
Adjacent reference 1: consider the rust adjacent reference when the current task pivots away from rust coder reliability patterns.
Adjacent reference 2: consider the coder adjacent reference when the current task pivots away from rust coder reliability patterns.
Adjacent reference 3: consider the reliability adjacent reference when the current task pivots away from rust coder reliability patterns.

## Workload Model

**Decision supported.** This section helps decide how to budget and split reliability work so review attention lands where boundaries cross. The seed's symbol-count boundary misses how this theme's effort actually distributes, per-boundary cost rather than per-symbol cost, a change touching one well-typed module is cheap at any size while a change crossing API, error, and concurrency surfaces at once carries three rungs of decisions and three gate escalations.

**Recommended default and causal basis.** Estimate rung engagement at planning time, and treat two-plus irreversible rungs in one change as a split trigger. The ladder prices work by surfaces crossed, each crossed surface adds a decision, its clauses, its ledger entry, and its matched gates, so two small diffs can differ tenfold in reliability workload.

**Operating conditions and assumptions.** Surfaces are identifiable at planning time, exploratory spikes may defer the count to their write-up. Budgeting reliability work under this theme, suite-founding economics live with the testing themes.

**Failure boundary and alternatives.** Symbol-count budgeting approves a fifty-symbol change that silently crosses four surfaces and stalls a two-hundred-symbol rename that crosses none, misallocating review attention in both directions. Bounded alternatives and recoveries: the seed's crate-slice and symbol-count boundaries, retained as coarse outer limits, surface counting prices what they cannot see.

**Counterexample, gotchas, and operational comparison.** Unsafe is the workload outlier, any unsafe engagement adds Miri, SAFETY documentation, and invariant review regardless of diff size, the seed's hundred-symbol boundary is irrelevant next to one raw pointer. Good: a plan splitting newtype introduction from the error redesign it enables. Bad: one PR converting an API to async while retyping its errors. Borderline: a rename crossing no surfaces at three hundred symbols, large but rung-free, the outer limit is the only brake.

**Verification, evidence, and uncertainty.** Compare recent change estimates against actuals split by engaged rungs. The ladder's rung structure and the gate-escalation doctrine read in full. Typical per-rung costs for a calibrated team are unestimated here.

**Second-order consequence.** The model implies a cheap planning trick, rung-count a change before writing it, a feature that would cross three irreversible surfaces in one PR is usually three PRs wearing one description, and the ladder's ordering even suggests the split sequence, ownership first, error second, concurrency third.

**Revision decision.** Restate the model in surface units, count the ladder rungs a change engages, budget decisions, ledger, and gates per engaged rung, and split changes that engage more than two irreversible rungs at once.

**Retained seed evidence.** The workload dimension rows with boundary values remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`combined_evidence_inference_note`: Treat Rust Coder Reliability Patterns as a rust system operating reference, not as a prose summary.

| workload_dimension_name | workload_boundary_value | verification_pressure_point |
| --- | --- | --- |
| primary_usage_surface | systems implementation, CLI or service hardening, async/runtime review, and safety-sensitive refactoring where compile success is necessary but not sufficient | verify that the reference changes the next implementation or review action |
| bounded_capacity_model | one crate or bounded workspace slice, up to 100 changed symbols, and one integration path that exercises error handling | split the task or create a narrower reference when this boundary is exceeded |
| source_pressure_model | local heading signals include Rust Reliability; Overview; When To Use; Workflow; Companion Use; Reference Use; Reference Map; Jump Points; Section Search; Use Order; Rust Coder 02; | compare guidance against canonical local sources before following external examples |
| artifact_pressure_model | required artifact is worked rust coder reliability patterns example with user goal, decision point, failure mode, and verification gate | require the artifact before claiming the reference is operationally usable |

## Reliability Target

**Decision supported.** This section helps decide which code invariants must demonstrably hold for this theme's guidance to count as adopted. The seed document-property thresholds stand where code invariants should, zero anti-pattern instances from the greppable half in merged code, every public fallible API carrying typed errors, every unsafe block carrying a SAFETY comment and a Miri-or-equivalent run, and every concurrency-touching change showing its matched heavy gate.

**Recommended default and causal basis.** Wire the mechanical targets into CI immediately and hold the review-bound targets as named checklist items. Each target restates a top-tier pattern as an invariant, the anti-pattern target operationalizes the rejection list, the typed-error target holds RC2-12's ninety-seven-point line, and the unsafe target is RC2-53's own definition of the pattern.

**Operating conditions and assumptions.** The codebase distinguishes library from binary crates, the typed-error target binds published library surfaces only, per RC2-12's own boundary. Code invariants for work under this theme, document-shape thresholds keep the seed's corpus jurisdiction.

**Failure boundary and alternatives.** Targets held as aspirations decay under deadline, one exported anyhow::Error from a library normalizes the next, and the rejection list's value is precisely that its lines are bright. Bounded alternatives and recoveries: quarantine-based tolerance of known violations with a burn-down list, honest as triage during adoption, corrosive as a steady state.

**Counterexample, gotchas, and operational comparison.** The unsafe target's Miri clause has real environments where Miri cannot run, FFI-heavy code, in which case RC2-53's or equivalent wording governs, the target is verification presence, not tool worship. Good: CI rejecting a &String parameter on a public function. Bad: declaring unsafe hygiene from the absence of crashes. Borderline: waiving the typed-error target on an internal tool binary, correct by the pattern's own clause, binaries aggregate.

**Verification, evidence, and uncertainty.** Review the four target audits at each release checkpoint with evidence artifacts attached. The rejection list, RC2-12, RC2-53, and the gate doctrine read in full. Grep precision for the mechanical half across code styles is implementation-dependent.

**Second-order consequence.** The four targets have a cost asymmetry worth exploiting, the anti-pattern and unsafe targets are enforceable almost free once wired into CI and grep, while the error and gate targets need human review, so wiring the cheap half first buys enforcement capacity for the expensive half.

**Revision decision.** Publish the four targets with evidence methods, grep and lint for the mechanical half, API review for error typing, unsafe inventory audit, and ledger check for gate escalation.

**Retained seed evidence.** All four seed reliability rows with thresholds remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| reliability_target_name | measurable_threshold_value | evidence_collection_method |
| --- | --- | --- |
| source_boundary_preservation | 100 percent of recommendations keep local, external, and inference boundaries visible | review generated text for the three evidence labels before reuse |
| decision_accuracy_sample | at least 18 of 20 sampled uses route to the correct adopt, adapt, avoid, or adjacent-reference decision | sample downstream tasks and record reviewer verdicts |
| unsupported_claim_rate | zero unsupported production, security, latency, or reliability claims in final guidance | reject claims without source row, explicit inference note, or verification method |
| recovery_path_clarity | every avoid or failure case names the rollback, escalation, or next-reference route | inspect failure-mode and adjacent-routing sections together |

## Failure Mode Table

**Decision supported.** This section helps decide which decay modes need standing probes and what each probe inspects. The seed's generic rows miss how this doctrine specifically decays, score rot where the 202603 rankings drift from current ecosystem reality, ladder skipping where agents jump to familiar rungs and improvise the rest, clause amnesia where patterns get applied without their avoid-when limits, and wrapper drift where SKILL.md's compressed lists diverge from the reference they summarize.

**Recommended default and causal basis.** Run the review-cycle probes continuously and the dated ecosystem probe quarterly, attributing findings to their decay mode. All four decay silently because the doctrine keeps working partially, a stale score still orders most patterns correctly, a skipped rung still produces code, an overapplied pattern still compiles, so each mode survives until its specific blind spot meets its specific trigger.

**Operating conditions and assumptions.** Probes are owned, an unowned probe list is this table's own failure applied to itself. Decay of this theme's doctrine in use, code-level failure modes are what the doctrine itself catalogs and are not repeated here.

**Failure boundary and alternatives.** Clause amnesia is the most damaging in practice, patterns applied beyond their avoid-when boundaries invert the meta-rule, producing the impressive-but-wrong code the file was built to prevent, while citing the file as authority for the inversion. Bounded alternatives and recoveries: periodic full re-derivation of the doctrine from fresh sources, the nuclear option wrapper drift and score rot both eventually justify.

**Counterexample, gotchas, and operational comparison.** Score rot and clause amnesia interact badly, a stale score cited without its clauses carries double-expired authority, so the ecosystem probe should prioritize exactly the patterns reviews cite most. Good: a probe catching that a recommended crate has been superseded since 202603. Bad: trusting a three-year-old score in a contested design review. Borderline: an agent citing a pattern without its avoid-when on an obviously-in-bounds case, harmless this once, the habit is the failure mode.

**Verification, evidence, and uncertainty.** Confirm each failure row names its probe, owner, and cadence, and sample one probe's recent findings. The bundle's wrapper-reference structure and clause architecture read in full. Actual decay onset speed for 202603 scores is unknowable without the dated probes.

**Second-order consequence.** Wrapper drift is the mode with a mechanical fix, SKILL.md step two names six defaults that must remain a faithful compression of the top tier, so a six-line comparison per refresh eliminates the mode entirely, the only one of the four that tooling can fully retire.

**Revision decision.** Add the four rows with probes, dated ecosystem spot-checks against top-tier scores, ledger audits for rung completeness, review sampling for avoid-when citations, and a wrapper-versus-reference diff at each corpus refresh.

**Retained seed evidence.** All four seed failure rows with mitigations remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| failure_mode_name | likely_trigger_condition | required_mitigation_action |
| --- | --- | --- |
| source drift hides truth | external or local guidance changes after the reference was written | refresh public evidence, rerun local corpus gate, and mark stale claims before reuse |
| generic advice escapes review | agent copies plausible best practices without theme-specific verification | block completion until the verification gate names concrete command, reviewer question, or metric |
| runtime contention masks failure | async tasks, locks, or shared state degrade under concurrent load | add contention benchmark, timeout budget, and structured cancellation path |
| panic path reaches production | unwrap or unchecked invariant survives compile and unit tests | require panic audit, Result propagation, and integration failure fixture |

## Retry Backpressure Guidance

**Decision supported.** This section helps decide how async work under this theme handles overload, cancellation, and retry without corrupting state. The seed's corpus-process bullets stand beside the file's own retry-adjacent doctrine, cancellation and backpressure as first-class design surfaces, bounded channels making throughput limits visible, cancellation tokens making shutdown cooperative, and cancel-safe select discipline preventing retried futures from corrupting half-consumed state.

**Recommended default and causal basis.** Bounded channels with explicit capacities, cancellation tokens on all long-lived tasks, and retries only at clean-ownership boundaries with budgets. RC2-21's stated logic is that unbounded queues hide problems until memory pressure becomes the symptom, so backpressure is not throttling etiquette but failure-visibility engineering, the bounded channel converts overload from a delayed crash into an immediate, attributable signal.

**Operating conditions and assumptions.** The runtime is tokio-shaped as the patterns assume, other runtimes keep the principles with different primitives. Retry and backpressure doctrine for code under this theme, corpus-process retry rules keep the seed's jurisdiction, and HTTP-level retry policy lives with the backend themes.

**Failure boundary and alternatives.** Retry logic wrapped around cancel-unsafe futures is the compounding trap, each retry of a future that owned half-read state re-enters corrupted ground, RC2-22's ownership discipline exists so that retrying is safe by construction rather than by luck. Bounded alternatives and recoveries: load-shedding at admission instead of backpressure through the pipeline, complementary, admission control protects the system while channel bounds localize the diagnosis.

**Counterexample, gotchas, and operational comparison.** Graceful shutdown is where all three patterns compose or collapse together, a shutdown path that cancels tasks holding unforwarded channel state needs the token, the bound, and cancel-safety simultaneously, testing shutdown tests the composition. Good: a worker pool with bounded intake, token-based shutdown, and select branches that own no partial reads. Bad: an unbounded mpsc feeding a retry loop over a cancel-unsafe future. Borderline: an unbounded channel on a provably low-volume control path, tolerated by RC2-21's own high-volume qualifier, the ledger should say so.

**Verification, evidence, and uncertainty.** Inject cancellation during load in a test harness and assert no message loss or duplicate processing. RC2-20 through RC2-23's clauses and rationales read in full. Correct channel capacities are workload-specific and outside the file's scope.

**Second-order consequence.** The doctrine links backpressure to diagnosability, a bounded channel's fullness is a measurable saturation signal while an unbounded one's growth is silent until terminal, so the bounded choice simultaneously prevents the failure and creates the observability the failure would need, one decision buying both.

**Revision decision.** Add the code-level regime, bound every high-throughput channel, token every long-lived task, keep select branches cancel-safe, and make retry decisions only at boundaries where state ownership is provably clean.

**Retained seed evidence.** All five seed retry and backpressure bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- Retry only after the failed verification signal is classified as transient, stale evidence, missing context, or true contradiction.
- Use one bounded retry for stale external evidence refresh, then escalate to a human or a narrower source-specific reference.
- Apply backpressure by stopping new generation or implementation work when source coverage, critique coverage, or verification gates are red.
- For long-running agent workflows, checkpoint after each batch and re-read the current spec before any broad rewrite, commit, push, or destructive operation.
- For distributed execution, assign one owner per generated file or theme batch and require merge-time verification of exact path, heading, and evidence-boundary invariants.

## Observability Checklist

**Decision supported.** This section helps decide what evidence running code under this theme must emit for failures to be reconstructable. The seed's generic records stand beside the file's own observability doctrine, RC2-38's structured tracing with spans, fields, and correlation IDs because async systems become opaque fast, RC2-39's secrecy wrappers because leaks are rarely caught by type-checking alone, and RC2-15's actionable diagnostics with exact locations and repair hints.

**Recommended default and causal basis.** Wire tracing at boundaries the ledger names, wrap secrets at type level, and hold diagnostics to the location-plus-hint bar in review. The three patterns share one premise, failures must be reconstructable after the fact, spans reconstruct control flow across tasks, typed secrets keep reconstruction artifacts safe to share, and located diagnostics make the reconstruction's endpoint actionable.

**Operating conditions and assumptions.** A tracing subscriber is actually configured, instrumented code with no collector is the observability equivalent of an unread registry. What code under this theme must record, corpus-audit records keep the seed's jurisdiction, and pipeline-level telemetry lives with ops themes.

**Failure boundary and alternatives.** The anti-pattern is named in RC2-38's avoid-when, dumping whole payloads without filtering, logs that are simultaneously an observability failure and a secret-handling failure, the two patterns violated by one lazy line. Bounded alternatives and recoveries: metrics-first observability without spans, cheaper at steady state, unable to reconstruct the cross-task causality async failures require.

**Counterexample, gotchas, and operational comparison.** Binary-safe diagnostics per RC2-16 constrain the logging layer too, paths and payloads that are not valid UTF-8 must survive into logs without lossy conversion, or the log lies precisely when the input was strange enough to matter. Good: an incident traced across five tasks by one correlation ID to a located parse error with a hint. Bad: a debug log printing a bearer token inside a dumped request. Borderline: skipping spans on a synchronous batch tool, defensible, RC2-38's premise is async opacity.

**Verification, evidence, and uncertainty.** Pick a recent failure and confirm it was reconstructable from emitted telemetry alone. RC2-15, RC2-16, RC2-38, and RC2-39 read in full. Span granularity that balances cost against reconstructability is workload-specific.

**Second-order consequence.** The diagnostics pattern doubles as an observability pattern, RC2-15's exact-location-plus-repair-hint rule means each error message is a pre-written incident note, so investment in diagnostic quality is investment in incident speed, the same text serving the user and the responder.

**Revision decision.** Add the code-level floor, spans with correlation fields on every async workflow, secrecy types on every credential, and located, hint-bearing diagnostics on every user-facing parse or config failure.

**Retained seed evidence.** All six seed observability bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- Record which local sources were loaded and which were intentionally skipped.
- Record the exact verification command, reviewer question, or rendered artifact used as proof.
- Record p50/p95/p99 latency or reviewer-time measurements when the reference affects runtime or workflow speed.
- Capture error count, timeout count, retry count, saturation signal, and rollback trigger.
- Record leading indicator and failure signal from this file in the coverage manifest or journal when the reference drives real work.
- Keep audit evidence small enough to review: command output summary, changed-file list, and unresolved-risk notes are preferred over raw transcript dumps.

## Performance Verification Method

**Decision supported.** This section helps decide how performance claims under this theme get verified without sacrificing the correctness the theme exists for. The seed's latency packet stands beside the file's distinctive stance, performance patterns are admitted only when they preserve correctness, section 6's own title, with panic-free arithmetic scored above builder ergonomics and prepare-once precomputation justified as much by local reasoning as by speed.

**Recommended default and causal basis.** Require benchmark evidence and the boundary audit together, and prefer wins that add named structure over wins that remove checks. The rubric has no raw-performance axis, so performance patterns earned their places through bug-prevention and misuse-resistance points, meaning this theme verifies performance work by checking correctness held, not just that numbers improved.

**Operating conditions and assumptions.** Benchmarks are stable enough to attribute deltas, noisy environments need repetition budgets before attribution. Verifying performance-flavored changes under this theme, service-level latency SLOs live with the backend and ops themes.

**Failure boundary and alternatives.** Performance work verified only by benchmarks invites exactly the regressions section 6 guards, unchecked indexing justified by a hot path, precomputation hiding freshness requirements, wins that trade correctness boundaries for microseconds. Bounded alternatives and recoveries: profile-guided optimization workflows, compatible, the boundary audit applies to whatever the profiler motivates.

**Counterexample, gotchas, and operational comparison.** RC2-41's avoid-when names the trap precisely, informal reasoning like we already proved this above in code that will change later, the proof decays as the code moves while the unchecked index remains, so removals of checks need invariants stated where the check was. Good: a Candidate-style precomputed view with a benchmark delta and a named type. Bad: replacing .get() with indexing in a loop because the bench improved. Borderline: saturating arithmetic swapped for checked in a hot path, fine when saturation is the documented semantic, not a silent overflow mask.

**Verification, evidence, and uncertainty.** Audit recent performance merges for the paired evidence, delta plus boundary audit. Section 6's four patterns and their clauses read in full. The real frequency of correctness-trading performance changes in reviewed codebases is unmeasured.

**Second-order consequence.** RC2-44's deepest claim is that the best wins are repeated-work eliminations that also improve reasoning, the derivation becomes a named value type, so this theme's performance ideal is refactoring-shaped, speed gained by making structure more explicit rather than less checked.

**Revision decision.** Verify performance changes on two axes jointly, the benchmark delta and a boundary audit confirming no checked operation became unchecked and no precomputation outlived its input's freshness.

**Retained seed evidence.** The method, indicator, measurement packet, and pass and fail lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Performance method: require memory, panic, and concurrency evidence; hot-path operations need p95 under 10 ms locally or an explicit benchmark exception.
Leading indicator to measure: compile attempts and review comments decrease because the API shape is explicit before implementation.
Failure signal to watch: the reference hides ownership or error tradeoffs behind generic guidance.
Required measurement packet: capture input size, source count, tool-call count, wall-clock time, p50/p95/p99 latency where runtime applies, and reviewer decision time where documentation applies.
Pass condition: the reference must let a reviewer identify the correct next action, verification gate, and stop condition without reading unrelated files.
Fail condition: the reference encourages implementation before the workload, reliability target, and failure-mode table are explicit.

## Scale Boundary Statement

**Decision supported.** This section helps decide at what domain edge this theme's guidance must be re-derived rather than stretched. The seed's task-splitting bounds stand beside the file's own boundary declaration, the coverage claim, comprehensive for libraries, CLIs, async services, backends, moderately low-level systems code, and contained FFI, and explicitly not a specialist manual for compiler internals, kernel-grade lock-free systems, deeply embedded no-std firmware, or extremely advanced proc-macro internals.

**Recommended default and causal basis.** Apply the doctrine freely within the six covered shapes, and at the four exclusions escalate to specialist sources with the exclusion quoted. The file frames this as a boundary claim, not a weakness claim, the four exclusions are domains where reliability doctrine exists but is different, lock-free kernels need verification regimes beyond loom models, no-std firmware inverts allocation assumptions the patterns quietly make.

**Operating conditions and assumptions.** Tasks are classifiable against the boundary, hybrid systems, a mostly-covered service with one lock-free core, split at the component line. Bounding this theme's applicability, task-size splitting keeps the seed's own rules.

**Failure boundary and alternatives.** Stretching the doctrine across its declared edge produces confident wrongness, applying RC2-59's channel matrix to a kernel scheduler or RC2-42's builder discipline to an allocation-free target, advice that is locally sensible and globally inapplicable. Bounded alternatives and recoveries: adopting specialist references for the excluded domains as corpus themes eventually, until then the exclusions route externally.

**Counterexample, gotchas, and operational comparison.** Moderately low-level is the boundary's soft phrase, arena allocators and memory-mapped structures sit in the gray band where RC2-53's containment doctrine still applies but the not-elevated list's caution begins, the gray band deserves the ledger's most explicit entries. Good: escalating a no-std allocator question with the coverage claim quoted. Bad: citing the channel matrix for kernel-context synchronization. Borderline: an advanced derive macro, RC2-52 covers hygiene and spans, internals beyond that are named out.

**Verification, evidence, and uncertainty.** At each new domain of work, record the boundary classification before applying the doctrine. The coverage claim and not-elevated list read in full. Where exactly moderately low-level ends is the author's soft edge, judgment fills it.

**Second-order consequence.** The boundary is drawn by verification feasibility rather than by difficulty, the covered domains are those where the file's gate set, tests, loom, Miri, can actually verify the patterns, and the excluded ones are where those gates lose traction, the coverage claim is really a claim about where the verification story holds.

**Revision decision.** Quote the coverage claim's six-in four-out boundary and add the operational seam the not-elevated list marks, lock-free building blocks without a verification plan sit just outside even within covered domains.

**Retained seed evidence.** All four seed scale-boundary paragraphs remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Rust Coder Reliability Patterns stops being sufficient when the task spans multiple independent systems, more than one ownership boundary, unbounded source discovery, or production traffic without an explicit SLO.
Under distributed execution, split work by theme file and preserve one verification owner per split; do not let parallel agents rewrite the same reference without a merge checkpoint.
Under long-running agent workflows, treat context drift as a reliability failure: checkpoint state, summarize open risks, and verify against the spec before continuing.
Under large-codebase scale, require dependency or source-graph narrowing before applying this reference; a source list without ranked canonical guidance is not enough.

## Future Refresh Search Queries

**Decision supported.** This section helps decide which probes reveal that this theme's crate stack or doctrine has gone stale. The seed theme-name queries would surface generic Rust content while this theme's staleness lives in named places, the top-tier crate stack, thiserror, anyhow, tokio, loom, proptest, secrecy, sqlx, miette, the standard-library surfaces the patterns cite, OnceLock, LazyLock, ExitCode, NonZero niches, and the upstream bundle's own revision state.

**Recommended default and causal basis.** Run the crate and std probes on a dated cadence against the 202603 anchor and check the archive at every corpus refresh. The doctrine is version-sensitive at two layers, crate APIs evolve on their own schedules and the standard library periodically absorbs crate territory, as it did for once-initialization, so refresh probes must watch both the crates and the std surfaces that could obsolete them.

**Operating conditions and assumptions.** Probe findings are triaged into example-breaking versus doctrine-breaking before any rewrite, crate churn is usually the former. Refreshing this document's ecosystem and lineage claims, doctrine-level refresh rides on upstream bundle revisions.

**Failure boundary and alternatives.** Name-based searching returns tutorials that neither confirm nor deny whether a 202603 crate recommendation still leads its niche, the only ecosystem question this theme's refresh needs answered. Bounded alternatives and recoveries: pinning the document explicitly to the 202603 crate set and marking every crate mention archive-dated, zero probe cost, honest, increasingly less useful.

**Counterexample, gotchas, and operational comparison.** Async-ecosystem probes skew toward framework news, this theme's async doctrine is runtime-discipline-shaped, cancellation, bounding, select safety, and survives framework churn that would invalidate a backend theme, so probe triage should resist framework-headline noise. Good: a probe noting std absorbed a cited crate's role and dating the affected pattern. Bad: logging that rust reliability tutorials say nothing new. Borderline: probing new verification tools beyond loom and Miri, adjacent and promising, findings extend the gate set rather than refresh it.

**Verification, evidence, and uncertainty.** Record each probe with its date and the specific pattern or crate claim it confirmed or flagged. The crate and std vocabulary compiled from all sixty patterns read in full. Actual churn since 202603 across the named stack is unknowable without the fetches.

**Second-order consequence.** The bundle's own history predicts its refresh shape, rust-coder-02 exists because rust-coder-01 was found thin, so the highest-consequence probe is not any crate check but the archive watch for a successor document, the lineage pattern suggests revision arrives as a new file rather than an edit.

**Revision decision.** Replace the name queries with targeted probes, release and advisory checks on the top-tier crate stack, std release-note scans for absorbed functionality, and the archive watch for a revised rust-coder bundle or a rust-coder-03.

**Retained seed evidence.** The three seed query rows remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| search_query_label_name | search_query_text_value |
| --- | --- |
| `official_docs_query_phrase` | rust coder reliability patterns official documentation best practices |
| `github_repository_query_phrase` | rust coder reliability patterns GitHub repository examples |
| `release_notes_query_phrase` | rust coder reliability patterns changelog release notes migration |

## Evidence Boundary Notes

**Decision supported.** This section helps decide under what label and with what attached caveats each claim here may be reused. The seed label definitions stand without this assignment's ledger, three mapped files read fully totaling 1610 lines, zero external URLs fetched, upstream source-pool files known only through the bundle's citations, and every adoption construct above marked as inference.

**Recommended default and causal basis.** Before reusing a claim, identify its stratum, cite the RC2 ID for doctrine, date the crate mentions, and attach reasoning to inference. The fact stratum is unusually deep here, sixty pattern IDs with scores, clauses, and source-basis blocks, the rubric weights, the ladder rungs, the twelve rejections, and the stack steps are all transcribable content, while the ledgers, probes, targets, and surface-unit workload model are this evolution's constructed machinery.

**Operating conditions and assumptions.** The archive paths stay stable so fact-class claims remain mechanically checkable. Reuse rules for this document's claims, transcribed doctrine travels with pattern-ID citations, constructed machinery travels only with its reasoning.

**Failure boundary and alternatives.** The costliest mislabel would present the scores or crate recommendations as current ecosystem consensus, they are one author's rubric applied to 202603 sources, authoritative as local doctrine and dated as ecosystem fact, the distinction is exactly what the labels protect. Bounded alternatives and recoveries: per-paragraph inline labels if the stratum-level split ever proves too coarse for a dispute.

**Counterexample, gotchas, and operational comparison.** The meta-rule travels especially well, reach for the simplest pattern that preserves correctness boundaries, and its source is section 11's LLM guidance specifically, quoting it as general engineering wisdom without the citation unmoors it from the ladder that gives it operational teeth. Good: reusing the no-locks-across-await rule cited as RC2-20. Bad: citing the boundary ledger as bundle doctrine. Borderline: reusing the rubric weights to score new patterns, legitimate as method, the weights themselves are one author's 202603 judgment.

**Verification, evidence, and uncertainty.** Sample claims from earlier sections and confirm each declares its stratum cleanly. This assignment's read ledger, three files read in full, zero retrievals. Readers can check the citations but not the reading behind them.

**Second-order consequence.** The source-basis blocks create a two-hop provenance chain unique in this cluster, each pattern cites S77, ripgrep, or rust-analyzer material that is in the repository but unmapped to this theme, so claims here are locally verifiable one hop deeper than the mapping table admits, an audit surface future refreshes can exploit without any external fetch.

**Revision decision.** Publish the strata, pattern-level content as local corpus fact citable by RC2 ID, bundle-structure claims as local fact citable by file and section, crate-ecosystem implications as archive-dated, and all adoption machinery as combined-evidence inference.

**Retained seed evidence.** The three labeled boundary definitions remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- `local_corpus_sourced_fact`: statements tied directly to the local source paths above.
- `external_research_sourced_fact`: statements tied to the public URLs above.
- `combined_evidence_inference_note`: synthesis that combines local and public evidence into agent guidance.
