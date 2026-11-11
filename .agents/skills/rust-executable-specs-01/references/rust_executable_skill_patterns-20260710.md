# Rust Executable Skill Patterns

**Decision supported.** This section helps decide how to treat two platform-specific entrypoints and a stray duplicate as one theme. The seed no seed line states what this theme actually holds, sibling entrypoints to one skill, a 122-line Claude prompt and a 105-line Codex prompt that share the same four modes, seven-step workflow, and seven-part output contract while differing in how they reach depth, absolute install paths versus relative in-bundle links.

**Recommended default and causal basis.** Read both entrypoints when studying the skill's design and cite the platform-appropriate one when instructing an agent. One skill was authored for two agent platforms in the same 202604 sweep, so the platform adaptations, frontmatter shape, reference plumbing, verification list, are the theme's real content, the shared doctrine merely rides along.

**Operating conditions and assumptions.** Both archive copies stay put, and the unclassified duplicate remains byte-identical to the claude-skills copy. This reference covers the entrypoint pair and its duplicate, the routed deep references are covered by their own themes.

**Failure boundary and alternatives.** Readers treating the two entrypoints as redundant copies miss the platform deltas that matter operationally, the Claude variant references machine-absolute macOS paths that resolve nowhere in this corpus while the Codex variant's relative links resolve inside its own bundle. Bounded alternatives and recoveries: the standalone merged reference for consumers who want the doctrine without any entrypoint framing.

**Counterexample, gotchas, and operational comparison.** The Claude variant's Reference Use paths begin with a user home directory from the authoring machine, quoting them as working paths reproduces an environment leak, the corpus-resolvable equivalent lives under idiomatic-references-202604. Good: citing the Codex SKILL.md for in-bundle routing behavior. Bad: handing a Codex agent the Claude variant's absolute install paths. Borderline: quoting the shared workflow from either copy interchangeably, safe for the shared steps, unsafe for the deltas.

**Verification, evidence, and uncertainty.** Diff the claude-skills copy against the unclassified copy before relying on their equivalence. Full reads of all three files and the identity diff this session. Which variant was authored first is unrecorded in either file.

**Second-order consequence.** The pair preserves a porting decision trail, the Codex variant added a Context Strategy section and inline gate commands while dropping the Claude variant's Core Principle and Miri from the verification list, evidence that porting a skill across platforms edits doctrine at the margins even when intent is faithful transfer.

**Revision decision.** Open with the sibling-entrypoints fact, the byte-identical unclassified duplicate, and the central contrast, portable relative plumbing versus environment-bound absolute plumbing.

**Retained seed evidence.** The exact theme title and reference framing remain unchanged. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

## Source Evidence Mapping Table

**Decision supported.** This section helps decide how much independent evidential weight each of the three listed paths carries. The seed three local rows list paths without recording the pair structure, rows one and three are byte-identical files, so the table's three paths represent exactly two documents, and the honest evidence base is one Claude entrypoint, one Codex entrypoint, and one dated identity measurement.

**Recommended default and causal basis.** Cite the claude-skills path as the Claude variant's canonical location and treat the unclassified copy as filing residue. The unclassified folder holds an unsorted copy of the Claude variant, the same filing pattern the executable-specs merge showed, so deduplication is again the first mapping act before any synthesis weighs the sources.

**Operating conditions and assumptions.** The identity holds, divergence would promote the unclassified copy into a third witness needing its own read. Provenance for this document's statements, all four external URLs stayed unretrieved and confer no current-fact authority.

**Failure boundary and alternatives.** Counting three independent witnesses would double-weight the Claude variant's platform choices, its absolute paths and its Miri-bearing verification list, in any contrast against the single Codex witness. Bounded alternatives and recoveries: a future mapping revision recording cross-file identity relations directly in the table rather than in prose annotations.

**Counterexample, gotchas, and operational comparison.** The two variants share a name and title, so citations naming only rust-executable-specs-01 are ambiguous between platforms, claims about frontmatter, reference plumbing, or verification lists must name the file path, not the skill name. Good: a contrast claim citing both variant paths explicitly. Bad: three-source phrasing implying three independent documents. Borderline: citing the unclassified path in an unclassified-cleanup context, correct locally, the canonical citation still names claude-skills.

**Verification, evidence, and uncertainty.** Re-run the identity diff and confirm cited deltas against the named file paths. The wc and diff runs this session plus full reads of both variants. Whether the unclassified copy predates or postdates the claude-skills filing is unknown.

**Second-order consequence.** The 202604 sweep's duplicate trail now spans three themes, merge, reliability transplant, and this entrypoint copy, and in every case the duplicate lives in unclassified-yet, which makes that folder a predictable place to search for mirrors whenever future mapping work meets a 202604 artifact.

**Revision decision.** Annotate the mapping with the pair-plus-duplicate structure and date the diff that established it.

**Retained seed evidence.** All three local rows and all four external rows with their column values remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| source_location_path_key | source_category_artifact_type | source_authority_confidence_level | source_usage_synthesis_role |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/claude-skills-202604/rust-executable-specs-01.md | local_corpus_source_material | local_corpus_sourced_fact | local agent-usable source material |
| archive/monthly-agent-snapshots/codex-skills-202604/rust-executable-specs-01/SKILL.md | local_corpus_source_material | local_corpus_sourced_fact | skill entrypoint or reusable agent prompt |
| methodology/executable-specifications/rust/core.md | local_corpus_source_material | local_corpus_sourced_fact | local agent-usable source material |
| https://github.com/rust-lang/api-guidelines | external_research_source_material | external_research_sourced_fact | Rust library-team API design recommendations |
| https://github.com/tokio-rs/tokio | external_research_source_material | external_research_sourced_fact | async runtime implementation and operational model |
| https://github.com/tokio-rs/axum | external_research_source_material | external_research_sourced_fact | Rust web framework implementation and design claims |
| https://docs.rs/axum/latest/axum/ | external_research_source_material | external_research_sourced_fact | published crate documentation for routing and extractors |

## Pattern Scoreboard Ranking Table

**Decision supported.** This section helps decide which entrypoint's patterns to imitate when building or repairing agent-facing skill files. The seed three corpus-process rows stand where this theme's ranking question is which entrypoint pattern travels best, and the Codex variant's relative Context Strategy outranks the Claude variant's absolute Reference Use for portability, every link in the Codex file resolves wherever the bundle lands while both Claude paths break outside the authoring machine.

**Recommended default and causal basis.** Model new skill entrypoints on the Codex variant's plumbing and the shared skeleton, importing Claude-variant enrichments only with resolvable paths. The ranking follows from resolvability, an entrypoint's value is delivering agents to depth, and plumbing that survives relocation delivers in more habitats than plumbing bound to one home directory.

**Operating conditions and assumptions.** The bundle structure the relative links assume stays intact, relative plumbing rots too when folders move. Ranking the entrypoint patterns against each other, the routed references' internal rankings belong to their own themes.

**Failure boundary and alternatives.** Ranking by richness instead would favor the Claude variant, whose Core Principle triad and Miri-bearing list carry more doctrine, and produce agents holding better principles with dead pointers to the depth that operationalizes them. Bounded alternatives and recoveries: the merged standalone reference, which outranks both entrypoints for consumers who need depth without an agent frame.

**Counterexample, gotchas, and operational comparison.** The Codex verification list omits Miri while the Claude list includes it, so ranking the Codex variant first for plumbing must not silently endorse its thinner verification menu, the lists differ and the difference is doctrine. Good: a new skill using relative reference links plus the full Claude verification list. Bad: copying the absolute-path Reference Use block into a shared repo. Borderline: keeping model and color frontmatter in a non-Claude habitat, inert metadata, harmless but meaningless.

**Verification, evidence, and uncertainty.** Resolve every reference link in a candidate entrypoint from a fresh clone before ranking it usable. The resolvability contrast and shared-skeleton comparison from this session's full reads. Whether the Miri omission in the Codex list was deliberate platform tailoring or porting loss is unrecorded.

**Second-order consequence.** The shared skeleton, four modes, seven workflow steps, seven output sections, identical traceability shape, is what survived porting untouched, which measures the skill's true platform-invariant core, whatever both authors kept verbatim under different constraints is the part the skill cannot lose.

**Revision decision.** Rank relative in-bundle plumbing first, shared mode-and-contract skeleton second as the platform-invariant core, and platform-specific enrichments third, valuable where their environment assumptions hold.

**Retained seed evidence.** The three scored seed rows with tier labels remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| pattern_identifier_stable_key | pattern_score_numeric_value | pattern_tier_adoption_level | pattern_failure_prevention_target |
| --- | ---: | --- | --- |
| `rust_executable_skill_patterns` | 95 | default_adoption_pattern_tier | Source Map First: Load local rust executable material before synthesizing skill patterns recommendations. |
| `rust_executable_skill_patterns` | 91 | default_adoption_pattern_tier | Evidence Boundary Split: Separate local facts, external facts, and inference before giving agent instructions. |
| `rust_executable_skill_patterns` | 88 | default_adoption_pattern_tier | Verification Gate Coupling: Attach each recommendation to at least one command, checklist, or review gate. |

## Idiomatic Thesis Synthesis Statement

**Decision supported.** This section helps decide what an entrypoint should promise versus merely point at. The seed generic corpus formulas stand where this theme's thesis is that an entrypoint is a contract plus plumbing, both variants promise the same thing, classify by mode, specify with REQ-RUST contracts, design L1/L2/L3 before coding, verify by risk, return the seven-part contract, and differ only in how they wire the agent to depth.

**Recommended default and causal basis.** Audit entrypoint plumbing on relocation events and trust the ported contract until the upstream doctrine itself changes. The skill's authors separated what the agent must promise from where the agent must read, so the promise ported cleanly across platforms while the plumbing had to be rebuilt per habitat, a separation that made the port cheap and the deltas legible.

**Operating conditions and assumptions.** The contract truly is platform-invariant, contracts that embed platform tool names would fork on port like the verification lists did. The thesis governs entrypoint design, the promised doctrine's own content is the deep references' subject.

**Failure boundary and alternatives.** Entrypoints that mix promise and plumbing cannot port this way, every relocation edit risks touching doctrine, and the Claude variant's dead absolute paths show the failure in miniature, plumbing decayed while the promise around it stayed true. Bounded alternatives and recoveries: single-entrypoint designs that serve all platforms with conditional sections, denser, and every platform reads irrelevant instructions.

**Counterexample, gotchas, and operational comparison.** The verification-mix lists sit inside the contract yet differ between variants, proof that the contract-plumbing boundary was drawn imperfectly, list-shaped doctrine near platform concerns is where faithful ports silently diverge. Good: a port that rewrites only the Context Strategy section. Bad: a port that rewords workflow steps and quietly drops a verification gate. Borderline: adding platform-specific guardrails during a port, legitimate tailoring, it should be recorded as a delta, not a copy.

**Verification, evidence, and uncertainty.** Diff ported entrypoints section by section and classify each delta as contract or plumbing. The observed section-level deltas between the two variants. Whether the authors intended the contract-plumbing separation or arrived at it by habit is unrecorded.

**Second-order consequence.** The two variants demonstrate asymmetric decay, the shared contract has no environment dependencies and cannot rot locally, while both plumbing layers rot on their own schedules, absolute paths at machine change and relative links at folder moves, so entrypoint maintenance is almost entirely plumbing maintenance.

**Revision decision.** State the contract-plus-plumbing thesis and its corollary, port the contract verbatim, rebuild the plumbing natively, and audit the two separately.

**Retained seed evidence.** The three labeled thesis statements remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`local_corpus_sourced_fact`: The local row for `rust_executable_skill_patterns` contains 3 source path(s), which should be treated as the first retrieval surface for this theme.
`external_research_sourced_fact`: The external source map provides public documentation, repository, or ecosystem analogues where available.
`combined_evidence_inference_note`: Reliable use of Rust Executable Skill Patterns comes from loading the local corpus first, checking public ecosystem guidance second, and converting both into verification-backed agent decisions.

## Local Corpus Source Map

**Decision supported.** This section helps decide which file and section answers a given question about this skill's design. The seed heading signals in the map obscure the structural deltas that matter, the Claude variant carries Core Principle and Reference Use, the Codex variant carries Context Strategy and References instead, and those four section names are precisely where the platform adaptations live.

**Recommended default and causal basis.** Route wiring questions by variant-only headings and contract questions to either copy's shared sections. Shared sections carry the ported contract while variant-only sections carry the habitat wiring, so the heading diff is a reliable index into which file answers which question.

**Operating conditions and assumptions.** Heading names stay stable, these files are archive-frozen so the assumption is safe until deliberate edits. Navigation across the entrypoint pair, the deep references' navigation belongs to the routing theme.

**Failure boundary and alternatives.** Consumers navigating by the shared headings alone never encounter the wiring sections, and consumers asked about progressive context loading would search the Claude variant in vain, that doctrine exists only in the Codex file. Bounded alternatives and recoveries: ripgrep heading search across the bundle, endorsed inside the Codex variant itself for large files.

**Counterexample, gotchas, and operational comparison.** Both variants list heading signals in this seed's map drawn from their real structure, but the map's title column shows the same skill name three times, navigation by title collapses the pair, navigation must key on path. Good: answering a context-loading question from the Codex Context Strategy section. Bad: searching the Claude variant for progressive-loading rules. Borderline: answering guardrail questions from either file, mostly shared, the Claude variant adds semver and detached-task skepticism the Codex wording folds differently.

**Verification, evidence, and uncertainty.** Confirm claimed section locations by heading grep against the named paths. The full section inventories of both variants read this session. None about current structure, both files were read in full today.

**Second-order consequence.** The Codex variant's Context Strategy embeds a four-step reading order that mirrors the bundle router's default order, entrypoint and router encode the same navigation doctrine in two artifacts, which means navigation claims can be cross-checked between them and divergence between the two would signal bundle drift.

**Revision decision.** Annotate the map with the variant-only sections and their roles, Core Principle as the Claude-only framing triad, Reference Use as the Claude-only external pointer block, Context Strategy as the Codex-only progressive-loading order, References as the Codex-only link list.

**Retained seed evidence.** All three local source rows with title, heading signals, and usage role remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| local_source_filepath_value | local_source_title_text | local_source_heading_signals | local_source_usage_role |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/claude-skills-202604/rust-executable-specs-01.md | rust-executable-specs-01 | Rust Executable Specs 01; Core Principle; Mode Selection; Workflow; Output Contract; Reference Use | local agent-usable source material |
| archive/monthly-agent-snapshots/codex-skills-202604/rust-executable-specs-01/SKILL.md | rust-executable-specs-01 | Rust Executable Specs 01; Mode Selection; Workflow; Output Contract; Guardrails; Context Strategy | skill entrypoint or reusable agent prompt |
| methodology/executable-specifications/rust/core.md | rust-executable-specs-01 | Rust Executable Specs 01; Core Principle; Mode Selection; Workflow; Output Contract; Reference Use | local agent-usable source material |

## External Research Source Map

**Decision supported.** This section helps decide which external claims this theme owns versus inherits. The seed four inherited framework anchors stand while this theme's own external surface is thinner than any deep reference's, the entrypoints name modes, layers, and gate commands but cite no crates beyond the shared doctrine's vocabulary, so the URLs inherit relevance from the routed references rather than from these files.

**Recommended default and causal basis.** Inherit ecosystem freshness from the deep-reference themes and check platform-field currency when either entrypoint is redeployed. Entrypoints compress doctrine into instructions and strip citations doing so, external claims live where clauses live, in the deep references, leaving the entrypoint theme with almost no independently refreshable external assertion.

**Operating conditions and assumptions.** The deep-reference themes keep their external candidate sets, delegation needs a live delegate. External rows serve future freshness only, all four remained unretrieved throughout this evolution.

**Failure boundary and alternatives.** Spending retrieval effort here duplicates the origin themes' external verification for a third time, the same waste the reliability transplant's delegation rule already prevents for its copy. Bounded alternatives and recoveries: pruning template rows that no mapped source motivates, a mapping revision this evolution flags but does not perform.

**Counterexample, gotchas, and operational comparison.** The docs.rs axum row is doubly odd here, neither entrypoint mentions axum at all, the row is pure template inheritance, and treating it as theme-relevant would invent a connection no mapped file supports. Good: checking the model frontmatter against current platform offerings at redeploy time. Bad: fetching tokio releases to freshen an entrypoint that never names tokio. Borderline: verifying the four-command gate set is still canonical, real doctrine, owned by the gates theme, not here.

**Verification, evidence, and uncertainty.** Confirm the delegation note and the platform-field check are recorded before the next refresh. Zero fetches this assignment and the absence of crate citations in both entrypoints. Current validity of the frontmatter model name is unverifiable from local evidence.

**Second-order consequence.** The pair does carry one class of externally sensitive content the deep references lack, platform frontmatter, the model field names a specific Claude model tier, and model-name currency decays faster than any crate API, making the entrypoint theme's freshest external question one about agent platforms, not Rust.

**Revision decision.** Keep the rows as candidates, delegate crate and framework freshness to the deep-reference themes, and note the one genuinely local external fact, the Claude variant's live install path names an environment no longer observable from this corpus.

**Retained seed evidence.** All four external rows with their boundary labels remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| external_source_url_value | external_source_name_text | external_source_usage_role | evidence_boundary_label_value |
| --- | --- | --- | --- |
| https://github.com/rust-lang/api-guidelines | Rust API Guidelines | Rust library-team API design recommendations | external_research_sourced_fact |
| https://github.com/tokio-rs/tokio | Tokio repository | async runtime implementation and operational model | external_research_sourced_fact |
| https://github.com/tokio-rs/axum | Axum repository | Rust web framework implementation and design claims | external_research_sourced_fact |
| https://docs.rs/axum/latest/axum/ | Axum docs.rs | published crate documentation for routing and extractors | external_research_sourced_fact |

## Anti Pattern Registry Table

**Decision supported.** This section helps decide which entrypoint decay modes need standing detection and at which level each operates. The seed three corpus-process rows stand where the entrypoint pair exhibits its own failure catalogue, environment leakage, absolute authoring-machine paths shipped in a portable artifact, porting loss, doctrine items like Miri silently dropped between variants, and duplicate drift risk, the unclassified copy diverging from its claude-skills original.

**Recommended default and causal basis.** Run all three detections whenever either entrypoint is edited, redeployed, or reclassified. Each failure is visible in or implied by the mapped files themselves, the leak is printed in the Reference Use section, the loss shows in the verification-list diff, and the drift risk follows from the same duplication pattern two prior themes documented.

**Operating conditions and assumptions.** Both variants remain intended as the same skill, deliberate platform divergence would convert porting loss into legitimate tailoring needing a changelog instead. Registry rows for entrypoint-specific failure, agent-behavior anti-patterns the entrypoints teach belong to the deep references.

**Failure boundary and alternatives.** Unregistered, these decay differently, leaked paths fail loudly at first use, ported losses fail silently by narrowing verification menus, and duplicate drift fails by serving stale doctrine from the unsorted folder. Bounded alternatives and recoveries: structural prevention for the drift row by removing or symlinking the unclassified duplicate during future classification work.

**Counterexample, gotchas, and operational comparison.** The seed's own three process rows remain valid for how this reference file gets generated, the added rows govern the mapped artifacts instead, two registries at two levels, conflating them assigns document-generation fixes to skill-file defects. Good: a redeploy check catching the dead macOS paths before an agent receives them. Bad: assuming the Codex verification list is complete because the file reads cleanly. Borderline: the frontmatter model field aging, real decay, platform-level rather than doctrine-level, tracked under the external map's platform check.

**Verification, evidence, and uncertainty.** Confirm each added row names its probe and the level it applies to. The observed deltas and the leak visible in the Claude variant's text. Whether other 202604 skills share the same porting-loss pattern is unchecked.

**Second-order consequence.** Porting loss is the subtlest row because both variants remain individually coherent after the loss, only the cross-variant diff reveals that one platform's agents were told a smaller verification menu, correctness per file and correctness per skill are different properties and only the pair view checks the latter.

**Revision decision.** Add the three rows with detections, path-resolution checks for leakage, section-by-section variant diffs for porting loss, and the identity diff for duplicate drift.

**Retained seed evidence.** The three seed process rows with their detection signals remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| anti_pattern_failure_name | failure_cause_risk_reason | safer_default_replacement_pattern | detection_signal_review_method |
| --- | --- | --- | --- |
| context_free_summary_output | agent skips local corpus and produces generic advice | source_map_first_synthesis | verify every listed local path appears in the generated file |
| unsourced_recommendation_claims | guidance appears authoritative without source boundary | evidence_boundary_split_pattern | check for local, external, and inference labels |
| unverified_agent_instruction | recommendation cannot be checked by command or review gate | verification_gate_coupling | require concrete gate in generated reference |

## Verification Gate Command Set

**Decision supported.** This section helps decide whether a given entrypoint should carry gates by value or by reference. The seed document verifier stands where the pair's own gate story is a placement contrast, the Codex variant names the four cargo gates inside workflow step seven so its agents carry the floor in the entrypoint itself, while the Claude variant defers all gate detail to its referenced standalone file, gates by value versus gates by reference.

**Recommended default and causal basis.** Inline the four-command floor in entrypoints and cite the gates reference for escalations, accepting the small upstream-sync burden. The Codex habitat optimizes for progressive loading where the entrypoint may be all the agent has read, so critical gates travel inline, while the Claude habitat assumed the reference file installed beside the skill, making deferral safe there.

**Operating conditions and assumptions.** The inlined floor is kept synchronized with the gates theme's canonical set at audit time. How gate obligations travel in entrypoints, the gate commands' own content belongs to the gates theme.

**Failure boundary and alternatives.** Gates by reference fail closed only if the reference resolves, and the Claude variant's paths no longer do from this corpus, so a Claude agent following that copy today holds gate obligations it cannot dereference, the deferral pattern's exact failure mode realized. Bounded alternatives and recoveries: the document verifier in the seed remains the gate for this reference file itself, a different level entirely.

**Counterexample, gotchas, and operational comparison.** The Codex variant's step seven also includes rejection criteria, requirement coverage, performance evidence, concurrency validation, which are review gates rather than commands, an agent grepping for runnable lines misses half the step's obligations. Good: a new entrypoint inlining the floor with a dated sync note. Bad: an entrypoint deferring all gates to a path that no clone contains. Borderline: inlining the full escalation matrix too, self-sufficient, and now a second copy of matrix doctrine to keep synchronized.

**Verification, evidence, and uncertainty.** Resolve every gate reference and compare inlined gate text against the canonical set at each audit. The step-seven contrast between the two variants read this session. How often Claude-habitat installs actually had the reference file beside the skill is unknowable from here.

**Second-order consequence.** The two placements fail in opposite directions, inline gates rot when the canonical gate set changes upstream, deferred gates rot when plumbing breaks, so the placement decision is really a bet on which changes faster, gate doctrine or file layout, and this corpus's history shows layout churning faster.

**Revision decision.** State the placement rule this contrast teaches, inline the unconditional floor in any entrypoint, defer only the risk-conditional escalations to routed references.

**Retained seed evidence.** The seed's document verifier command block remains preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`verification_gate_command_set`: Run the repository verifier after editing this file.

```bash
python3 archive/monthly-agent-snapshots/idiomatic-references-202606/tools/verify_reference_generation.py --stage final
```

## Agent Usage Decision Guide

**Decision supported.** This section helps decide which variant an agent loads and what it must emit first once loaded. The seed four generic bullets stand where the entrypoints themselves are agent usage doctrine, both files tell an agent exactly how to behave, choose modes before planning, name the component type first, write REQ-RUST contracts, design layers before coding, pick verification by risk, and return the seven-part contract, so this section's job is teaching agents to choose between and correctly consume the two variants.

**Recommended default and causal basis.** Have agents state their active mode set and component type before producing any other output section. The variants target different agent platforms with different context economics, the Codex file assumes progressive loading with the entrypoint read first and depth pulled on demand, the Claude file assumes an installed skill with a resident reference, so variant choice follows the consuming platform's loading model.

**Operating conditions and assumptions.** The task is Rust work within the skill's coverage, both variants scope themselves to libraries, CLIs, services, backends, parsers, contained FFI, and reviews. Agent selection and consumption of the entrypoint pair, the behavioral doctrine both teach is shared and needs no per-variant restatement.

**Failure boundary and alternatives.** Cross-platform consumption misfires concretely, a progressive loader following the Claude variant has no Context Strategy to sequence its reads, and an install-based agent following the Codex variant may pull four references it could have had resident. Bounded alternatives and recoveries: the deep references directly for agents already past classification, both entrypoints route there anyway.

**Counterexample, gotchas, and operational comparison.** The two variants' default combinations tables order their rows differently, agents comparing the files may misread reordering as disagreement, the four combinations are semantically identical across both. Good: a Codex agent reading SKILL.md then pulling the reference map per its Context Strategy. Bad: an agent skipping mode selection and opening with an implementation plan. Borderline: an agent blending both variants' guardrail lists, harmless enrichment, the merged list should not be attributed to either file alone.

**Verification, evidence, and uncertainty.** Check agent outputs open with mode and component classification before requirements. The parallel workflow and mode sections read in both variants this session. How strictly deployed agents followed the output ordering in practice is unrecorded.

**Second-order consequence.** Both variants front-load the same first decision, mode selection before any planning or coding, which is the skill's deepest agent instruction, an agent that classifies the task's risk shape first inherits the right depth automatically, while every observed anti-pattern in the deep references begins with an agent that started coding before classifying.

**Revision decision.** Add the selection rule, match the variant to the platform's loading model, then follow the chosen file's own workflow verbatim including its plumbing sections.

**Retained seed evidence.** The four seed usage bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`agent_usage_decision_guide`: Use this reference when a task mentions this theme, one of the listed local source paths, or a nearby technology/workflow surface.

- Start with the local corpus source map.
- Prefer patterns with explicit verification gates.
- Treat external sources as freshness and ecosystem checks, not replacements for local repo conventions.
- Preserve the evidence boundary labels when reusing recommendations.

## User Journey Scenario

**Decision supported.** This section helps decide which arrival pattern fits the current consumer of the entrypoint pair. The seed one generic engineer stands where the pair's travelers differ by starting artifact, a platform integrator porting the skill to a third agent platform and needing the contract-plumbing split, an agent-behavior debugger tracing why a deployed agent skipped a gate and needing to know which variant it was fed, and a classifier deciding the unclassified duplicate's fate.

**Recommended default and causal basis.** Start every pair journey by identifying which variant is in play and whether its plumbing resolves in the current habitat. The theme's artifacts are consumed at deployment and maintenance time rather than during Rust work itself, so its journeys are meta-journeys, about the skill files, while the Rust-work journeys belong to the deep references the entrypoints route into.

**Operating conditions and assumptions.** The variants remain the skill's only entrypoints, a third port would extend each journey's artifact set. Journeys through the entrypoint pair, in-task Rust journeys belong to the routed themes.

**Failure boundary and alternatives.** The debugger's journey fails worst without variant awareness, gate-skipping behavior has different explanations per variant, a Codex agent had the floor inline and ignored it, a Claude agent may have hit the dead reference paths, same symptom, opposite root causes. Bounded alternatives and recoveries: the reliability transplant theme's maintenance journey for the deeper mirror-handling flow the classifier can reuse.

**Counterexample, gotchas, and operational comparison.** The classifier's journey looks trivial, delete the duplicate, but the unclassified copy may be load-bearing for anything that indexed unclassified-yet, so the safe journey ends in a recorded decision, not a silent removal. Good: a debugger checking whether the misbehaving agent's variant carried gates inline. Bad: porting to a new platform by editing the Claude variant in place. Borderline: reading the pair purely to learn the skill's doctrine, better served by the merged standalone reference.

**Verification, evidence, and uncertainty.** Ask recent consumers which variant and which resolution state their journey began from. The platform deltas and duplication structure established this session. Whether any indexing actually references the unclassified copy is unchecked.

**Second-order consequence.** The integrator's journey has a ready-made worked answer in the corpus, the Codex variant itself is the record of the last port, so porting to a third platform means repeating its visible decisions, rebuild plumbing natively, keep the skeleton verbatim, and audit list-shaped doctrine for silent loss, the previous port is the template for the next.

**Revision decision.** Recast the scenario as the three arrivals, integrator at the section-level variant diff, debugger at the specific variant's plumbing, classifier at the identity diff and the mirror-handling precedent.

**Retained seed evidence.** The role, starting state, decision, and trigger lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Role based opening scenario: The Rust reliability engineer is starting from a requirement that needs a safe API, explicit error model, and testable boundary and needs a reference that turns source evidence into an executable next step.
Primary user starting state: The user has a `rust_executable_skill_patterns` task, one or more local source paths, and uncertainty about which pattern should drive implementation.
Decision being made: choosing the idiomatic ownership, async, error, and crate-boundary shape before coding.
Reference opening trigger: Open this file when the task mentions rust executable skill patterns, any mapped local source path, or an adjacent workflow with the same failure mode.

## Decision Tradeoff Guide

**Decision supported.** This section helps decide how entrypoint packaging tensions get resolved and what evidence reopens each. The seed template rows skip the pair's live tensions, inline versus deferred gates, where self-sufficiency trades against upstream sync burden, richer versus leaner entrypoints, where the Claude variant's extra framing trades against the Codex variant's tighter loading cost, and per-platform forks versus one shared entrypoint, resolved here as forks with an unwritten sync obligation.

**Recommended default and causal basis.** Hedge on plumbing, inline the floor and resolve links at deploy, bet on doctrine, keep the skeleton shared and audit forks for silent divergence. Each tension was resolved differently per platform because the platforms price context differently, and the resolutions' costs are now visible in the corpus, dead paths from deferral, a thinner verification list from lean porting, and no changelog from the fork decision.

**Operating conditions and assumptions.** Audits actually compare the fork pair, an unaudited fork converts the bet into drift by default. Tuning entrypoint packaging decisions, the doctrine's own design tradeoffs belong to the deep references.

**Failure boundary and alternatives.** Each pole's failure is already observable or one step away, deferred gates dereference nothing here, the lean port lost Miri, and the fork pair has no mechanism to keep its shared skeleton aligned under future edits. Bounded alternatives and recoveries: collapsing both variants into one file with platform-conditional sections, rejected by the 202604 authors, revisitable if a third platform port materializes.

**Counterexample, gotchas, and operational comparison.** The richer-versus-leaner axis is not monotonic, the Codex variant is leaner overall yet added Context Strategy, porting decisions cut and add simultaneously, so leanness must be judged per section, not per file. Good: a port review comparing verification lists line by line. Bad: trusting fork alignment because both files share a name. Borderline: adding a sync-note convention between the variants, cheap alignment aid, one more artifact to maintain.

**Verification, evidence, and uncertainty.** Audit the pair's shared sections for divergence at each corpus refresh. The resolution costs visible in the mapped files this session. Whether future edits will hit either variant at all is unknown, both are archive-frozen today.

**Second-order consequence.** All three tensions share one underlying variable, how much the entrypoint trusts its surroundings, inline gates, rich framing, and a single shared file all hedge against broken surroundings while deferral, leanness, and forking all bet on intact ones, and this corpus's observed decay pattern, layout churn with stable doctrine, argues for hedging on plumbing and betting on doctrine.

**Revision decision.** Add the three axes with observed costs and reversal triggers, plumbing breakage reopens gate placement, verification-list drift reopens port leanness, and skeleton divergence reopens the fork decision.

**Retained seed evidence.** The adopt, adapt, avoid, and cost-of-being-wrong rows remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| decision_option_name | when_to_choose_condition | tradeoff_cost_description | verification_question_prompt |
| --- | --- | --- | --- |
| Adopt when | local corpus and external evidence agree on the rust executable skill patterns pattern | fastest path, but can copy stale local assumptions | Does the selected pattern appear in the canonical source and current external evidence? |
| Adapt when | local sources are strong but public ecosystem guidance has changed | preserves repo fit, but requires explicit inference notes | Did the reference label the local fact, external fact, and combined inference separately? |
| Avoid when | source evidence is thin, conflicting, or unrelated to the user journey | prevents false confidence, but may require deeper research | Is there a confidence warning or adjacent reference route? |
| Cost of being wrong | wrong rust executable skill patterns guidance can send an agent to the wrong files, tests, or abstraction | wasted implementation loop and weaker verification | Would a reviewer know what to undo and what to inspect next? |

## Local Corpus Hierarchy

**Decision supported.** This section helps decide which file settles a question when the two variants or the duplicate disagree. The seed one canonical row cannot express a two-variant theme, neither entrypoint outranks the other globally, the Codex variant is canonical for in-bundle consumption and progressive loading, the Claude variant is canonical for the install-based deployment model and carries doctrine the other dropped, and the unclassified duplicate ranks below both as filing residue.

**Recommended default and causal basis.** Classify the question's habitat first, apply the superset rule only for doctrine conflicts the habitat split cannot settle. Authority follows habitat fit as the reliability transplant theme established, and here both files are originals in their own habitats, so the hierarchy is a genuine peer split rather than an origin-mirror split.

**Operating conditions and assumptions.** The porting-loss reading is correct, if the Miri drop was deliberate Codex tailoring the superset rule mislabels tailoring as loss. Precedence within the entrypoint pair and duplicate, the deep references outrank all three for doctrine depth.

**Failure boundary and alternatives.** Declaring either variant primary corpus-wide breaks one habitat's consumers, Codex-first answers hand progressive loaders dead absolute paths never meant for them, Claude-first answers strip the loading strategy the bundle depends on. Bounded alternatives and recoveries: escalating doctrine conflicts upstream to the deep references, which both variants subordinate themselves to anyway.

**Counterexample, gotchas, and operational comparison.** The seed's hierarchy row grants the claude-skills path canonical rank from file position, which happens to align with the superset rule for doctrine but must not leak into plumbing questions, where the Codex variant rules its bundle. Good: settling a verification-mix dispute by the fuller Claude list plus deep-reference confirmation. Bad: citing the unclassified copy as an independent tiebreaker. Borderline: a frontmatter question, only the Claude variant has model metadata, no conflict possible, the habitat rule answers it trivially.

**Verification, evidence, and uncertainty.** Test hierarchy rulings against the habitat split and superset tiebreaker. The delta inventory and duplication structure from this session's reads. Authorial intent behind each cross-variant difference remains unrecorded.

**Second-order consequence.** This is the corpus's first peer hierarchy between non-identical siblings, the merge and transplant themes ranked identical bytes by habitat, this theme must rank different bytes claiming the same skill, and the tiebreaker that emerges, prefer the fuller list when variants disagree on doctrine, is a superset rule usable wherever ported artifacts disagree.

**Revision decision.** Record the peer hierarchy with the one asymmetry that matters, for shared-skeleton disputes the Claude variant's fuller lists are the safer reading because the observed porting direction lost content rather than added it.

**Retained seed evidence.** The single hierarchy row and the confidence warning remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Classification vocabulary includes canonical, supporting, legacy, duplicate, and conflicting source roles.

| local_source_filepath_value | corpus_hierarchy_role | heading_signal_to_convert | reviewer_question_to_answer |
| --- | --- | --- | --- |
| archive/monthly-agent-snapshots/claude-skills-202604/rust-executable-specs-01.md | canonical primary source | Rust Executable Specs 01; Core Principle; Mode Selection | What guidance, warning, or example should this source contribute to Rust Executable Skill Patterns? |
| archive/monthly-agent-snapshots/codex-skills-202604/rust-executable-specs-01/SKILL.md | legacy historical source | Rust Executable Specs 01; Mode Selection; Workflow | What guidance, warning, or example should this source contribute to Rust Executable Skill Patterns? |
| methodology/executable-specifications/rust/core.md | supporting context source | Rust Executable Specs 01; Core Principle; Mode Selection | What guidance, warning, or example should this source contribute to Rust Executable Skill Patterns? |

## Theme Specific Artifact

**Decision supported.** This section helps decide what standing evidence makes the pair's differences legible without repeated re-derivation. The seed generic worked-example artifact misses this theme's natural object, a variant delta table, one standing record listing every section-level difference between the two entrypoints, its classification as contract or plumbing, the suspected reason, platform tailoring or porting loss, and the resolution state of every path either file references.

**Recommended default and causal basis.** Build the table at the first consultation that needs any delta judgment and cite it thereafter. Every question this theme answers, which variant to load, which list to trust, what a third port must rebuild, reads from the same delta inventory, so recording it once beats re-deriving it at each consultation.

**Operating conditions and assumptions.** The table lives at a findable location adjacent to this reference's evolution record. One delta table for this pair, mirror ledgers for byte-identical copies remain the transplant theme's artifact.

**Failure boundary and alternatives.** Without the table, each consumer re-diffs the pair and re-judges each delta, and inconsistent judgments accumulate, one reader calls the Miri drop tailoring while another calls it loss, with no shared record to converge on. Bounded alternatives and recoveries: a corpus-wide ported-artifact register covering all 202604 platform pairs at once, heavier, worth it only if more pairs surface.

**Counterexample, gotchas, and operational comparison.** Cause judgments must be labeled as inference, nothing in either file states why any delta exists, a table presenting suspected causes as facts would manufacture provenance the corpus lacks. Good: a debugger citing the table's row for gate placement. Bad: a third port planned from memory of the diff. Borderline: extending the table with the duplicate's identity status, convenient co-location, it duplicates the transplant theme's ledger pattern at small scale.

**Verification, evidence, and uncertainty.** Check the table exists with all deltas classified and every path's resolution state dated. The delta set enumerated across this document's sections. The cause column can never exceed labeled inference without authorial testimony.

**Second-order consequence.** The delta table is what a porting changelog would have been had one been written, reconstructing it now converts undocumented porting history into corpus structure, and its hardest rows, the cause judgments, are exactly the rows whose absence makes the pair ambiguous today, the artifact's value concentrates where certainty is weakest.

**Revision decision.** Define the artifact as the delta table with four columns, section, delta description, contract-or-plumbing class, suspected cause, plus a path-resolution appendix covering both variants' references.

**Retained seed evidence.** The artifact field rows with completion rules and evidence hints remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Theme specific artifact: skill activation contract with progressive disclosure, misuse case, and verification gate.

| artifact_field_name | artifact_completion_rule | evidence_source_hint |
| --- | --- | --- |
| user_goal_statement | state the user's concrete goal before applying Rust Executable Skill Patterns | local corpus hierarchy plus critique findings |
| decision_boundary_rule | define the point where this reference should be used or avoided | decision tradeoff guide |
| verification_gate_rule | define the command, checklist, or review question that proves the artifact worked | verification gate command set |

## Worked Example Set

**Decision supported.** This section helps decide which exercises build porting and variant-selection judgment for this pair. The seed abstract usage lines stand where this theme wants a porting rehearsal, hand a learner the two variants and require a reconstruction of the port, which sections transferred verbatim, which were rebuilt, and which doctrine items vanished, with the Miri drop as the planted discovery that tests whether their diff was section-deep or line-deep.

**Recommended default and causal basis.** Run the rehearsal with anyone who will port or maintain agent skills and bank the best reconstruction as the delta table's seed. The pair's lessons live in details that casual reading skips, a learner who finds the verification-list difference unprompted has internalized the porting-loss category, while one who reports the files as equivalent has rehearsed exactly the oversight that ships thinner doctrine.

**Operating conditions and assumptions.** Learners work from the files alone first, showing them this reference beforehand converts discovery into transcription. Teaching pair analysis and porting discipline, Rust-work exercises belong to the deep references.

**Failure boundary and alternatives.** Unrehearsed maintainers port by vibes, copying the visible skeleton and improvising the plumbing, which reproduces both observed failure classes at once, environment leakage from copied paths and silent loss from unaudited lists. Bounded alternatives and recoveries: documentation-only transfer via this reference, cheaper, and it skips the discovery that makes the loss category stick.

**Counterexample, gotchas, and operational comparison.** The rehearsal must include the dead-path experience, let the learner attempt to resolve the Claude variant's install paths and fail, the visceral dead end teaches plumbing hygiene better than any stated rule. Good: a reconstruction that finds the Miri drop and classifies Context Strategy as rebuilt plumbing. Bad: certifying a porter who diffed headings only. Borderline: a learner who classifies the Miri drop as deliberate tailoring, defensible reading, the grade turns on whether they flagged the ambiguity.

**Verification, evidence, and uncertainty.** Compare rehearsal reconstructions against the full delta inventory. The discovery-shaped structure of the pair's differences. How well rehearsal-built judgment transfers to future unlike pairs is unmeasured.

**Second-order consequence.** The rehearsal doubles as artifact bootstrapping, a learner's completed reconstruction is a draft of the variant delta table, so training output feeds the theme's standing evidence, the corpus improves as a side effect of teaching it, a loop worth engineering deliberately wherever themes need artifacts nobody has budgeted.

**Revision decision.** Anchor the section on the porting rehearsal plus a consumption drill, give a learner a platform description and require the correct variant choice with the loading-model justification stated.

**Retained seed evidence.** The good, bad, and borderline seed lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Good example: Use Rust Executable Skill Patterns after loading the canonical source, confirming the external evidence boundary, and writing a verification gate before implementation.
Bad example: Use Rust Executable Skill Patterns as a generic tutorial while ignoring the mapped local paths, source hierarchy, and cost of being wrong.
Borderline case: Use Rust Executable Skill Patterns only after adding a confidence warning when local evidence is thin or conflicts with current ecosystem guidance.

## Outcome Metrics and Feedback Loops

**Decision supported.** This section helps decide which measurements prove the pair remains consumable, aligned, and correctly deployed. The seed compile-centric indicator misses what this theme should measure, plumbing resolution rate, the fraction of referenced paths in both variants that resolve from a fresh clone, fork alignment, whether the shared skeleton still matches across variants at each audit, and variant-fit accuracy, deployments where the loaded variant matched the platform's loading model.

**Recommended default and causal basis.** Collect resolution and alignment at each corpus audit and fit at each recorded deployment. Each metric watches one established failure class, resolution watches leakage and rot, alignment watches silent divergence, and fit watches the cross-platform misconsumption the usage guide warns about.

**Operating conditions and assumptions.** Deployments are recorded at all, fit cannot be measured from the archive alone. Measuring the entrypoint pair's health, agent-outcome metrics for Rust work belong to the deep references.

**Failure boundary and alternatives.** Unmeasured, the classes decay on their proven schedules, the Claude paths are already dead, proof that resolution rot happens here, and nothing currently distinguishes a corpus that knows it from one that does not. Bounded alternatives and recoveries: measuring nothing on frozen archives, defensible until the next port or edit, at which point baselines built now pay off.

**Counterexample, gotchas, and operational comparison.** Alignment must compare semantics, not bytes, the variants legitimately reorder their default-combination rows, a byte-level alignment metric would alarm on harmless ordering forever. Good: an audit recording two-of-two Codex links resolving and zero-of-two Claude paths. Bad: inferring pair health from the absence of agent complaints. Borderline: counting the duplicate's identity check here, already owned by the mapping section's diff, double-counting inflates apparent coverage.

**Verification, evidence, and uncertainty.** Confirm the three series exist with dated entries at the next refresh. The resolution states and delta inventory measured this session. Deployment frequency for either variant is invisible from the archive.

**Second-order consequence.** The resolution metric's baseline is already interesting, the Codex variant scores perfectly while the Claude variant scores zero on its two reference paths, a spread that quantifies the plumbing-strategy argument better than any prose, one number pair carries the whole gates-by-reference case.

**Revision decision.** Add the three metrics with collection points, clone-and-resolve for plumbing, section diff for alignment, and deployment records for fit.

**Retained seed evidence.** The leading indicator, failure signal, and review cadence lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Leading indicator: compile attempts and review comments decrease because the API shape is explicit before implementation.
Failure signal: the reference hides ownership or error tradeoffs behind generic guidance.
Review cadence: Re-run the verifier after every generated-reference edit and refresh external sources when public APIs, docs, or tooling releases change.

## Completeness Checklist

**Decision supported.** This section helps decide what must be confirmed before this pair reference is declared faithful. The seed shape items never audit this theme's distinctive claims, the pair structure with its byte-identical duplicate, the section-level delta inventory, the dead-path finding, and the peer hierarchy with its superset tiebreaker, none of which heading counts can confirm.

**Recommended default and causal basis.** Run the comparative audit alongside the standard count audit at every reread cycle. The theme's substance is comparative, its claims describe relations between two files and a duplicate, so completeness auditing must re-execute the comparisons, re-diff the duplicate, re-inventory the deltas, re-resolve the paths.

**Operating conditions and assumptions.** All five involved paths remain stable, moves convert comparison audits into provenance hunts first. Auditing this reference against its comparative claims, each entrypoint's internal completeness is trivially checkable by its own section list.

**Failure boundary and alternatives.** A shape-only audit passes this document while the unclassified copy diverges, a variant gets edited, or a folder move revives or kills paths, every comparative claim stale behind intact headings. Bounded alternatives and recoveries: freezing the comparisons as recorded facts with dates and auditing only the dates, cheaper, blind to actual change between audits.

**Counterexample, gotchas, and operational comparison.** The delta inventory check must tolerate presentation differences between this document's prose and the table artifact's rows, auditing for semantic agreement, not string equality, or the audit fails on phrasing forever. Good: an audit re-running diff, delta scan, and path resolution in one pass. Bad: declaring fidelity from 26 present headings. Borderline: skipping the delta re-derivation because both files are archive-frozen, tempting, the freeze assumption is itself what the audit exists to test.

**Verification, evidence, and uncertainty.** Execute the comparative items and date each result in the audit record. The comparative claim structure of every distinctive statement here. Whether the shared audit script will be built remains an open corpus question.

**Second-order consequence.** The three duplicated-material themes now share an audit shape, dated diff, reference resolution, cross-file consistency, confirming the convergence the transplant theme predicted, one relational audit script parameterized by file pairs would serve all three and every future mirror or port theme at marginal cost near zero.

**Revision decision.** Add the comparative items, identity diff on the duplicate, delta inventory re-derivation against both variants, path resolution for all references in both files, and a check that the delta table artifact exists and matches.

**Retained seed evidence.** All seven seed checklist bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- The role scenario names the user, starting state, decision, and trigger for Rust Executable Skill Patterns.
- The decision guide includes Adopt when, Adapt when, Avoid when, and Cost of being wrong.
- The local corpus hierarchy identifies canonical and supporting sources or gives a confidence warning.
- The theme specific artifact is concrete enough to review without reading every mapped source.
- The examples cover good, bad, and borderline usage.
- The metrics section names one leading indicator and one failure signal.
- The adjacent routing section points to a better reference when this one is not the right fit.

## Adjacent Reference Routing

**Decision supported.** This section helps decide where to send a question arriving at a theme that owns only its deltas and plumbing. The seed token-split rows point nowhere while this theme's neighbors are structurally determined, the reference-maps theme for the router the Codex variant's Context Strategy mirrors, the reliability transplant theme for the deep file both variants route reliability questions into, the gates theme for the command floor the Codex variant inlines, and the playbook's own theme for spec-mode depth neither entrypoint contains.

**Recommended default and causal basis.** Answer pair and deployment questions here and forward doctrine with the owning theme named. Entrypoints are pure routing artifacts, everything substantive they mention lives elsewhere by design, so nearly every doctrine question arriving here forwards, and the theme keeps only pair-structure, porting, and deployment questions.

**Operating conditions and assumptions.** The sibling themes remain evolved and stable, restructuring any of the five reopens the closure map. Sending away what the entrypoints only point at, destination content is deliberately not summarized here.

**Failure boundary and alternatives.** Answering doctrine locally from entrypoint summaries serves compression where clauses exist one hop away, the exact compression-substitution failure the transplant theme's registry rejects. Bounded alternatives and recoveries: for the unread playbook file both entrypoints reference, the honest answer remains unread-source description, flagged rather than improvised.

**Counterexample, gotchas, and operational comparison.** Questions about the Claude variant's Core Principle triad have no deeper home, the triad exists only in that entrypoint, one of the few doctrine fragments this theme genuinely owns rather than forwards. Good: forwarding a verification-matrix question to the deep reference while answering the placement contrast locally. Bad: expanding the entrypoints' one-line reliability rules into full guidance here. Borderline: the four-word naming convention, mentioned in both variants, owned by the conventions theme, forward with the mention noted.

**Verification, evidence, and uncertainty.** Sample recent answers for delta-versus-forwarded classification. The routing structure both entrypoints themselves declare. Whether the five-theme closure test will be run is an open follow-up.

**Second-order consequence.** This theme completes the executable-specs bundle's coverage in the corpus, router, playbook-adjacent merge, reliability transplant, gates, and now entrypoints each have an evolved home, so cross-references among these five themes can finally be checked for closure, every in-bundle pointer should land on an evolved theme, a closure test unavailable until this assignment.

**Revision decision.** Route by the own-your-delta rule the transplant theme established, keep variant deltas, plumbing states, and porting guidance, forward modes, contracts, layers, verification, and gates to their owning themes.

**Retained seed evidence.** The three seed adjacent-reference lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Adjacent reference guidance: Use backend, executable, or quality-gate Rust references when the question shifts to HTTP delivery, specs, or review gates.
Adjacent reference 1: consider the rust adjacent reference when the current task pivots away from rust executable skill patterns.
Adjacent reference 2: consider the executable adjacent reference when the current task pivots away from rust executable skill patterns.
Adjacent reference 3: consider the skill adjacent reference when the current task pivots away from rust executable skill patterns.

## Workload Model

**Decision supported.** This section helps decide how to budget work whose failure mode is being forgotten rather than being expensive. The seed symbol-count boundary misprices this theme's work, whose units are per-audit comparison cost, one diff, one delta re-scan, and path resolutions across two small files, per-port cost, the bounded rebuild a third platform would trigger, and per-deployment cost, a variant-fit check that takes one classification question.

**Recommended default and causal basis.** Piggyback all pair upkeep on the corpus audit cadence and treat the delta table as the porting accelerant it is. Both entrypoints total 227 lines, the smallest source set in the executable-specs cluster, so every operation on the pair is minutes-scale, and like the transplant theme's mirror upkeep, the danger is neglect through triviality rather than cost.

**Operating conditions and assumptions.** The audit cadence keeps running, inherited lapses propagate here exactly as they do for the mirror ledger. Budgeting pair upkeep and porting support, Rust-task workload belongs to the deep references' own models.

**Failure boundary and alternatives.** Skipped comparisons let the frozen-archive assumption run unverified, and the first unnoticed edit to either variant converts every cached judgment here into potential misinformation at zero alarm. Bounded alternatives and recoveries: the seed's crate-slice and symbol boundaries, retained as outer limits on any single consultation's scope.

**Counterexample, gotchas, and operational comparison.** The per-deployment unit hides outside the corpus, deployments happen in agent platforms this archive cannot observe, so that unit's budget is really documentation, making the fit rule findable enough that deployers spend the minute themselves. Good: audit records showing the pair checks appended to each cycle. Bad: a porting effort re-deriving deltas a maintained table already held. Borderline: batching path resolution to every second audit, tolerable for frozen archives, it doubles the window the freeze assumption runs unchecked.

**Verification, evidence, and uncertainty.** Check pair upkeep appears inside host cadence records with dated entries. The line counts and operation inventory established this session. Whether a third-platform port will ever materialize is unknown, the budget exists regardless.

**Second-order consequence.** The porting unit is the only one with real variance, and its cost concentrates in exactly the rows the delta table pre-answers, which sections to rebuild and which lists to audit, so maintaining the minutes-scale audit artifact caps the half-day porting task's uncertainty, small standing work buying down the large episodic work's risk.

**Revision decision.** Restate the model in its three units, minutes per audit riding the existing cadence, a bounded half-day per hypothetical port, and near-zero per deployment check.

**Retained seed evidence.** The workload dimension rows with boundary values remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

`combined_evidence_inference_note`: Treat Rust Executable Skill Patterns as a rust system operating reference, not as a prose summary.

| workload_dimension_name | workload_boundary_value | verification_pressure_point |
| --- | --- | --- |
| primary_usage_surface | systems implementation, CLI or service hardening, async/runtime review, and safety-sensitive refactoring where compile success is necessary but not sufficient | verify that the reference changes the next implementation or review action |
| bounded_capacity_model | one crate or bounded workspace slice, up to 100 changed symbols, and one integration path that exercises error handling | split the task or create a narrower reference when this boundary is exceeded |
| source_pressure_model | local heading signals include Rust Executable Specs 01; Core Principle; Mode Selection; Workflow; Output Contract; Reference Use; Rust Executable Specs 01; Mode Selection; Workflow | compare guidance against canonical local sources before following external examples |
| artifact_pressure_model | required artifact is skill activation contract with progressive disclosure, misuse case, and verification gate | require the artifact before claiming the reference is operationally usable |

## Reliability Target

**Decision supported.** This section helps decide which pair invariants must demonstrably hold for this theme's guidance to count as adopted. The seed document-property thresholds stand where pair invariants should, the duplicate identity holding at every audit, the delta inventory matching the live files continuously, every plumbing claim carrying a dated resolution state, and the peer hierarchy's habitat split staying consistent with how the corpus actually answers questions.

**Recommended default and causal basis.** Hold all four invariants through the audit cadence with the ruling sample at corpus refresh. Each invariant guards one comparative claim class this theme established, identity guards the duplicate structure, inventory guards the delta claims, resolution guards the plumbing findings, and the hierarchy invariant guards against answers drifting from the declared split.

**Operating conditions and assumptions.** Questions about the pair actually flow through the corpus where sampling can see them. Pair invariants for this theme, the entrypoints' promised Rust reliability belongs to the doctrine they route to.

**Failure boundary and alternatives.** The invariants fail toward different victims, broken identity misleads classifiers, stale inventory misleads porters, undated resolution states mislead deployers, and hierarchy drift produces contradictory rulings from the same corpus. Bounded alternatives and recoveries: structural fixes, editing the Claude variant to point at corpus-resolvable paths, which would repair plumbing at the cost of falsifying an archival artifact.

**Counterexample, gotchas, and operational comparison.** The resolution invariant's healthy state differs per variant, Codex paths should stay resolving and Claude paths should stay documented-dead, an audit that just wants all paths green would misread the Claude rows as failures to fix rather than findings to preserve. Good: an audit series showing identity, inventory, and resolution states all dated and stable. Bad: declaring pair health from file timestamps alone. Borderline: fixing the dead paths in a copy while preserving the archive original, honest if labeled, it creates a third variant the inventory must then track.

**Verification, evidence, and uncertainty.** Review the four invariant series at each corpus checkpoint with dated evidence. The comparative claim structure established across this document. Whether ruling sampling is practical in this corpus's workflow is untested.

**Second-order consequence.** The hierarchy invariant is the novel one here, a documented precedence rule can rot socially while every file stays byte-stable, if consumers start treating the Claude variant as globally primary the corpus's behavior has diverged from its documentation with no file-level signal, reliability of rulings needs its own probe distinct from reliability of files.

**Revision decision.** Publish the four invariants with evidence methods, dated diffs, delta re-scans, resolution logs, and a periodic sample of answered questions against the hierarchy split.

**Retained seed evidence.** All four seed reliability rows with thresholds remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| reliability_target_name | measurable_threshold_value | evidence_collection_method |
| --- | --- | --- |
| source_boundary_preservation | 100 percent of recommendations keep local, external, and inference boundaries visible | review generated text for the three evidence labels before reuse |
| decision_accuracy_sample | at least 18 of 20 sampled uses route to the correct adopt, adapt, avoid, or adjacent-reference decision | sample downstream tasks and record reviewer verdicts |
| unsupported_claim_rate | zero unsupported production, security, latency, or reliability claims in final guidance | reject claims without source row, explicit inference note, or verification method |
| recovery_path_clarity | every avoid or failure case names the rollback, escalation, or next-reference route | inspect failure-mode and adjacent-routing sections together |

## Failure Mode Table

**Decision supported.** This section helps decide which pair-decay modes need standing probes and where each probe must live. The seed generic rows miss how an entrypoint pair specifically dies, silent variant edit, one sibling modified without the delta inventory updating, duplicate divergence, the unclassified copy drifting from its original, plumbing bit-rot, resolvable links dying as bundles move, and doctrine inversion, consumers reading the leaner ported list as the authoritative one and propagating the loss forward.

**Recommended default and causal basis.** Run the three mechanical probes on the audit cadence and the inversion check at every derivation from either variant. The first three are file-level decay with mechanical probes, while doctrine inversion is a social failure, the thinner list travels because it is shorter and newer-looking, and each propagation makes the lost item look more deliberately excluded.

**Operating conditions and assumptions.** Derivations are reviewable events, silent derivation outside the corpus defeats the inversion probe entirely. Decay of the pair and its consumption, Rust-work failure modes belong to the routed doctrine.

**Failure boundary and alternatives.** Inversion compounds worst, a third port built from the Codex variant inherits the Miri-less list, and two generations later the fuller Claude list reads as the outlier, porting loss laundered into apparent consensus by repetition. Bounded alternatives and recoveries: resolving the ambiguity at its root by testimony or by deep-reference comparison, the reliability reference includes Miri, which already arbitrates the specific known case.

**Counterexample, gotchas, and operational comparison.** The inversion probe must not overcorrect into Claude-variant absolutism, the fuller list wins only where variants disagree on shared doctrine, plumbing and platform sections legitimately differ and have no superset direction. Good: a derivation review catching a new skill built from the thinner list alone. Bad: trusting pair health because all diffs pass while derivatives quietly standardize on the lossy branch. Borderline: a deliberate future decision to drop Miri corpus-wide, legitimate if recorded, the probe's job is forcing the recording.

**Verification, evidence, and uncertainty.** Confirm each failure row names its probe, owner, cadence, and probe location. The deep reference's Miri row arbitrating the known list conflict. How many derivations from these entrypoints already exist outside the corpus is unknowable.

**Second-order consequence.** Doctrine inversion is the first failure mode in this cluster with no file-level signal at all, every file can be byte-perfect while the corpus's derived artifacts steadily prefer the lossy branch, which means its probe must live in derivation reviews, checking what new artifacts cite, not in any audit of the pair itself.

**Revision decision.** Add the four rows with probes, delta re-scan for silent edits, identity diff for divergence, clone resolution for bit-rot, and superset-rule citations in any list-bearing derivative for inversion.

**Retained seed evidence.** All four seed failure rows with mitigations remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| failure_mode_name | likely_trigger_condition | required_mitigation_action |
| --- | --- | --- |
| source drift hides truth | external or local guidance changes after the reference was written | refresh public evidence, rerun local corpus gate, and mark stale claims before reuse |
| generic advice escapes review | agent copies plausible best practices without theme-specific verification | block completion until the verification gate names concrete command, reviewer question, or metric |
| runtime contention masks failure | async tasks, locks, or shared state degrade under concurrent load | add contention benchmark, timeout budget, and structured cancellation path |
| panic path reaches production | unwrap or unchecked invariant survives compile and unit tests | require panic audit, Result propagation, and integration failure fixture |

## Retry Backpressure Guidance

**Decision supported.** This section helps decide what retry authority the entrypoint level carries versus routes. The seed corpus-process bullets stand beside what both entrypoints actually instruct about retry and backpressure, prefer bounded concurrency, stay cancellation-aware, avoid blocking and lock guards across await, and stay skeptical about backpressure and detached task lifecycles, compressed one-line rules whose full clauses live one route away in the reliability reference.

**Recommended default and causal basis.** Let entrypoint retry lines gate mode selection and require clause citations from the routed reference in any actual design. The entrypoints carry retry doctrine at exactly the depth an agent needs before routing, enough to flag the risk class and select Reliability Mode, and both files explicitly subordinate the details to their referenced depth.

**Operating conditions and assumptions.** The route to the reliability reference resolves, which holds in the Codex habitat and fails through the Claude variant's dead paths today. How retry doctrine surfaces at the entrypoint level, clause-level content belongs to the reliability theme.

**Failure boundary and alternatives.** Agents treating the one-liners as complete guidance ship bounded-channel prohibitions without sizing rules or cancellation wiring, the compression-substitution failure realized at the entrypoint level, one hop earlier than the merge's version of it. Bounded alternatives and recoveries: the backend themes' service-level retry guidance layered above these primitives for HTTP-facing work.

**Counterexample, gotchas, and operational comparison.** The Claude variant's guardrail adds detached-task lifecycles to the skeptic list while the Codex wording ends at API evolution, a small doctrine delta inside the retry-adjacent surface, derivatives quoting the skeptic list should quote the fuller version. Good: an agent flagging unbounded-channel risk, selecting Reliability Mode, and citing routed clauses in the design. Bad: a backpressure design justified entirely from an entrypoint one-liner. Borderline: a review comment quoting only the entrypoint rule, acceptable as a flag, insufficient as a resolution.

**Verification, evidence, and uncertainty.** Audit retry-bearing outputs for routed clause citations beyond entrypoint lines. The identical retry lines in both variants and the fuller Claude skeptic list, all read this session. Whether deployed agents respected the two-stage rule in practice is unrecorded.

**Second-order consequence.** Both variants preserved the retry one-liners identically through the port, unlike the verification lists, which suggests the authors treated concurrency skepticism as contract rather than plumbing, the porting process itself reveals which doctrine the skill considers load-bearing, and retry discipline made the cut on both platforms.

**Revision decision.** State the two-stage rule both files embody, the entrypoint's retry lines are classifiers that trigger Reliability Mode, and the routed reference is where designs are actually built.

**Retained seed evidence.** All five seed retry and backpressure bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- Retry only after the failed verification signal is classified as transient, stale evidence, missing context, or true contradiction.
- Use one bounded retry for stale external evidence refresh, then escalate to a human or a narrower source-specific reference.
- Apply backpressure by stopping new generation or implementation work when source coverage, critique coverage, or verification gates are red.
- For long-running agent workflows, checkpoint after each batch and re-read the current spec before any broad rewrite, commit, push, or destructive operation.
- For distributed execution, assign one owner per generated file or theme batch and require merge-time verification of exact path, heading, and evidence-boundary invariants.

## Observability Checklist

**Decision supported.** This section helps decide which records each level owes and where they must live. The seed generic records stand beside the pair's two observability surfaces, what the entrypoints tell agents to make observable, actionable diagnostics and gate-result reporting inside the seven-part output contract, and what the corpus must keep observable about the pair itself, the dated comparison records, resolution logs, and delta inventory this theme's sections established.

**Recommended default and causal basis.** Check contract-section presence structurally in agent outputs and emit pair records at their trigger events. The output contract is itself an observability device, forcing agents to surface mode choices, requirement coverage, and gate results in fixed positions where reviewers can find them, so the entrypoints teach observability by structure rather than by telemetry.

**Operating conditions and assumptions.** Records live in findable locations, the audit trail and the delta table, scattered evidence observes nothing. Observability at the entrypoint and pair levels, tracing and logging doctrine belongs to the routed reliability theme.

**Failure boundary and alternatives.** Agents skipping contract sections hide exactly the decisions reviews need, an output without the quality-gate section is the agent-level equivalent of a service without health checks, silent by omission. Bounded alternatives and recoveries: free-form agent outputs with richer prose, more expressive, and unauditable at scale.

**Counterexample, gotchas, and operational comparison.** The open-questions section is the contract's most skippable slot and its most valuable signal, an agent reporting zero open questions on vague work is observably overconfident, absence in that slot is data, not cleanliness. Good: a review rejecting an output missing its quality-gate section unread. Bad: a derivation consuming a variant with no record of which one. Borderline: recording only failed resolution checks, compact, it forfeits the healthy-series baseline the metrics section needs.

**Verification, evidence, and uncertainty.** Reconstruct pair state history and recent derivation choices from the records alone. The shared output contract and the record set established across this document. Record retention across corpus reorganizations remains unguaranteed.

**Second-order consequence.** The fixed section ordering both variants share makes agent outputs machine-checkable, a reviewer can verify section presence and order without reading content, the same cheap structural observability the corpus's own verifier applies to this document, entrypoint design and corpus tooling converged on the identical trick independently.

**Revision decision.** Keep the contract's reporting obligations by reference and specify the pair records concretely, each audit appends dated diff, delta, and resolution results, each derivation records which variant and list it consumed.

**Retained seed evidence.** All six seed observability bullets remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- Record which local sources were loaded and which were intentionally skipped.
- Record the exact verification command, reviewer question, or rendered artifact used as proof.
- Record p50/p95/p99 latency or reviewer-time measurements when the reference affects runtime or workflow speed.
- Capture error count, timeout count, retry count, saturation signal, and rollback trigger.
- Record leading indicator and failure signal from this file in the coverage manifest or journal when the reference drives real work.
- Keep audit evidence small enough to review: command output summary, changed-file list, and unresolved-risk notes are preferred over raw transcript dumps.

## Performance Verification Method

**Decision supported.** This section helps decide where performance rigor enters the workflow and what each later stage owes it. The seed latency packet stands beside what both entrypoints instruct about performance, reject vague claims like faster without thresholds, include performance in the verification mix when risk warrants, and report measurable evidence in the gate results, threshold discipline enforced at the specification stage before any code exists.

**Recommended default and causal basis.** Reject unthresholded performance language at intake and carry stated limits through matrix selection to gate reporting. The entrypoints position performance verification as a requirements problem, the REQ-RUST template's measurable-behavior clauses force units at contract-writing time, so performance rigor is front-loaded into specification rather than retrofitted at review.

**Operating conditions and assumptions.** Requirement review actually happens at intake, retrofitting thresholds after implementation reintroduces the anchoring problem. Performance discipline at the specification and entrypoint level, benchmark mechanics belong to the routed references and testing themes.

**Failure boundary and alternatives.** Work specified with vague speed claims cannot be performance-verified afterward regardless of benchmark quality, the missing thresholds make every measurement unanchored, which is why both variants reject the claims at intake rather than at gate time. Bounded alternatives and recoveries: service-level latency budgeting from the backend themes layered above requirement thresholds for deployed systems.

**Counterexample, gotchas, and operational comparison.** The pair itself has one measurable performance fact worth recording, both files are small enough that full-read costs are negligible for any agent, so context-budget arguments for skipping entrypoint reading fail on arithmetic, 227 lines total. Good: a vague faster request bounced back for thresholds before any design work. Bad: a benchmark suite built against requirements containing no numbers. Borderline: internal refactors with no stated performance requirements, no thresholds owed, the gate section should say so explicitly.

**Verification, evidence, and uncertainty.** Sample requirement sets for units and gate reports for threshold-anchored results. The rejection rules and matrix guidance read in both variants this session. How consistently intake rejection was practiced by deployed agents is unrecorded.

**Second-order consequence.** The entrypoints' rejection rule is the cheapest performance tool in the whole stack, refusing the word faster costs one review comment at intake and saves the unanchored benchmark cycle later, specification-stage rejection is performance verification performed at its highest-leverage point.

**Revision decision.** State the front-loading rule, thresholds enter at requirement writing, the verification matrix picks performance tests only where requirements state limits, and gate results report against those stated limits.

**Retained seed evidence.** The method, indicator, measurement packet, and pass and fail lines remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Performance method: require memory, panic, and concurrency evidence; hot-path operations need p95 under 10 ms locally or an explicit benchmark exception.
Leading indicator to measure: compile attempts and review comments decrease because the API shape is explicit before implementation.
Failure signal to watch: the reference hides ownership or error tradeoffs behind generic guidance.
Required measurement packet: capture input size, source count, tool-call count, wall-clock time, p50/p95/p99 latency where runtime applies, and reviewer decision time where documentation applies.
Pass condition: the reference must let a reviewer identify the correct next action, verification gate, and stop condition without reading unrelated files.
Fail condition: the reference encourages implementation before the workload, reliability target, and failure-mode table are explicit.

## Scale Boundary Statement

**Decision supported.** This section helps decide at what variant count and at which request domains the pair structure and skill coverage stop applying. The seed task-splitting bounds stand beside this theme's real scale limits, the two-variant structure stops scaling gracefully at the third platform port, where pairwise delta tables give way to a needed common-skeleton specification, and the skill's own coverage claim bounds it to application-level Rust, both variants decline unsafe-heavy, lock-free, and proc-macro-heavy defaults explicitly.

**Recommended default and causal basis.** Hold the pairwise structure at two variants, extract at three, and surface the declined-defaults list wherever the skill is newly deployed. Pairwise comparison is quadratic in variants exactly as mirror diffs are in copies, and the guardrails' declined defaults are authored limits that no port can widen, the same two limit classes the transplant theme found, combinatorial growth and inherited coverage edges.

**Operating conditions and assumptions.** A third port is noticed as such, an independent reimplementation that never references the pair would bypass the trigger. Bounding the pair structure and the skill's inherited coverage, the deep references' internal scale seams belong to their themes.

**Failure boundary and alternatives.** Past the port limit, delta tables multiply and drift independently, and past the coverage limit, consumers get entrypoints that confidently route kernel-grade concurrency questions into references that decline them, coverage failure by confident routing. Bounded alternatives and recoveries: canonical-specification-first design from the start for any future multi-platform skill, avoiding the extraction debt entirely.

**Counterexample, gotchas, and operational comparison.** The coverage limit lives in the guardrails sections both variants carry, but routed consumers may reach depth without reading guardrails, so the declined defaults belong in deployment documentation, not just inside the files agents may skim past. Good: a third port triggering skeleton extraction with the list conflicts resolved in the neutral file. Bad: three platform variants maintained by three pairwise tables. Borderline: consulting the skill for moderately unsafe-adjacent work, inside coverage if unsafe stays contained, the guardrail's contained-FFI wording draws the line.

**Verification, evidence, and uncertainty.** At each port or out-of-domain event, re-test both limits and record which trigger fired. The guardrail declinations in both variants and the combinatorics of pairwise maintenance. Whether any third platform port is contemplated anywhere is unknown.

**Second-order consequence.** The third-variant trigger would retroactively fix the pair's existing ambiguity, extracting the skeleton forces the Miri question to be decided once in the neutral file instead of differing silently per platform, scale pressure as the forcing function that resolves accumulated porting debt, growth as the repair opportunity.

**Revision decision.** Name both limits with triggers, a third variant triggers extraction of the shared skeleton into a platform-neutral specification with per-platform plumbing files, and out-of-coverage requests route to specialist sources the corpus does not currently hold.

**Retained seed evidence.** All four seed scale-boundary paragraphs remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

Rust Executable Skill Patterns stops being sufficient when the task spans multiple independent systems, more than one ownership boundary, unbounded source discovery, or production traffic without an explicit SLO.
Under distributed execution, split work by theme file and preserve one verification owner per split; do not let parallel agents rewrite the same reference without a merge checkpoint.
Under long-running agent workflows, treat context drift as a reliability failure: checkpoint state, summarize open risks, and verify against the spec before continuing.
Under large-codebase scale, require dependency or source-graph narrowing before applying this reference; a source list without ranked canonical guidance is not enough.

## Future Refresh Search Queries

**Decision supported.** This section helps decide which probes reveal that this theme's relations or platform assumptions have gone stale. The seed theme-name queries would surface generic Rust skill content while this theme's staleness has four homes, the duplicate identity relation, the variant delta inventory, the resolution states of both plumbing layers, and the platform frontmatter's currency against live agent platforms.

**Recommended default and causal basis.** Run the three internal probes at each corpus refresh and queue the platform check for the next external retrieval window. The theme's claims are comparative and environmental rather than doctrinal, so its refresh probes are the audit's own comparisons plus one genuinely external question, whether the named model tier and skill-loading conventions still exist on the platforms the variants target.

**Operating conditions and assumptions.** Platform documentation is retrievable when that window opens, until then the frontmatter claims stay archive-dated. Refreshing this document's comparative and environmental claims, Rust doctrine freshness delegates to the deep-reference themes.

**Failure boundary and alternatives.** Name-based searching returns rust specification material that neither confirms the pair's internal relations nor answers the platform-currency question, effort spent exactly where this theme has no claims. Bounded alternatives and recoveries: duplicating deep-reference ecosystem probes locally, prevented by the delegation rule two themes have now established.

**Counterexample, gotchas, and operational comparison.** The platform check must distinguish format currency from file validity, an outdated frontmatter field leaves the archive artifact historically accurate while making naive redeployment fail, staleness here means redeploy-with-review, never retroactive editing. Good: a refresh running all three internal probes and logging the platform check as queued. Bad: logging that rust skill searches returned nothing actionable. Borderline: probing whether newer executable-spec methodologies supersede the skill, real research, it refreshes the deep themes' claims, not this pair's relations.

**Verification, evidence, and uncertainty.** Record each probe with its date and the specific relation or assumption it confirmed. The comparative claim structure and the frontmatter facts read this session. Actual platform-side format drift since 202604 is unknowable without retrieval.

**Second-order consequence.** This theme carries the cluster's first genuinely external refresh question that is not about Rust at all, entrypoint formats are platform API surfaces, frontmatter fields and skill-loading conventions version like any API, and a corpus of agent skills therefore needs platform-currency probes alongside its language-ecosystem ones, a category this evolution names for future themes to reuse.

**Revision decision.** Replace the name queries with the four probes, identity diff, delta re-scan, dual resolution check, and a platform-convention check against current Claude and Codex skill documentation when external retrieval happens.

**Retained seed evidence.** The three seed query rows remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

| search_query_label_name | search_query_text_value |
| --- | --- |
| `official_docs_query_phrase` | rust executable skill patterns official documentation best practices |
| `github_repository_query_phrase` | rust executable skill patterns GitHub repository examples |
| `release_notes_query_phrase` | rust executable skill patterns changelog release notes migration |

## Evidence Boundary Notes

**Decision supported.** This section helps decide under what label each claim class here may be reused elsewhere. The seed label definitions stand without this assignment's distinctive ledger, all three mapped files read in full this session totaling 349 lines, one identity diff establishing the duplicate relation, zero external URLs fetched, and every cross-variant delta claim grounded in same-session parallel reads rather than memory or inference.

**Recommended default and causal basis.** Before reusing a claim, classify it as observed delta, dated measurement, labeled inference, or pending-retrieval, and carry the class forward. This evolution's evidence base is unusually direct, both variants were read minutes apart, so delta claims carry single-session eyewitness strength, the strongest local evidence class this cluster's themes have had available.

**Operating conditions and assumptions.** The three mapped files remain unedited, any edit re-opens every observation to re-verification. Reuse rules for this document's claims, each claim travels with its observation-versus-inference class.

**Failure boundary and alternatives.** The costliest mislabel would be presenting cause attributions with the same strength as delta observations, the Miri absence in the Codex list is an observed fact, why it is absent is labeled inference, and collapsing that distinction manufactures authorial testimony that does not exist. Bounded alternatives and recoveries: seeking authorial testimony or commit history to convert the direction inference into fact, unavailable in the archive itself.

**Counterexample, gotchas, and operational comparison.** The porting-direction inference, Claude first then Codex, colors several sections' framing yet rests only on the Codex file's leaner shape and added loading strategy, a reversed authorship history would flip porting-loss readings into enrichment readings, the inference is load-bearing and must stay visibly labeled. Good: reusing the verification-list delta as observed fact with the session date. Bad: citing porting loss as established history in a derived artifact. Borderline: reusing the superset tiebreaker without its inference caveat, operationally safe, epistemically sloppy.

**Verification, evidence, and uncertainty.** Sample claims from earlier sections and confirm each declares its class cleanly. This assignment's ledger, three full reads, one diff, zero retrievals. The porting-direction inference remains the document's single largest labeled unknown.

**Second-order consequence.** The delta-versus-cause split this ledger enforces is the ported-artifact analogue of the archive-versus-live split the spec enforces for external sources, both separate what was observed from what is being concluded, and naming the parallel gives future port-analysis themes a ready-made boundary vocabulary.

**Revision decision.** Publish the strata explicitly, file contents and deltas as local corpus facts from this session's full reads, the duplicate identity as a dated measurement, all cause attributions and the porting-direction reading as labeled inference, and platform-currency claims as unverifiable pending retrieval.

**Retained seed evidence.** The three labeled boundary definitions remain preserved. The original source facts, examples, and tables follow so that the evolved guidance remains auditable.

- `local_corpus_sourced_fact`: statements tied directly to the local source paths above.
- `external_research_sourced_fact`: statements tied to the public URLs above.
- `combined_evidence_inference_note`: synthesis that combines local and public evidence into agent guidance.
