# AR055: ACTIVE-reference Consolidated Summary (AR000-AR054)

**Date**: 2026-02-27  
**Scope**: `docs/ACTIVE-reference`  
**Coverage**: 54 artifacts (52 `.md`, 1 `.txt`, 1 `.drawio`)

## Executive Synthesis

This corpus converges on one V200 direction:

1. Parseltongue should be a dependency-graph-first intelligence layer (not a raw code dump system).
2. Compiler-truth + graph reasoning + selective LLM judgment is the core architecture.
3. V200 should prioritize graph fidelity, deterministic contracts, and read-path correctness over token-optimization extras.
4. Context packing/ranking is explicitly deferred to V210 unless evidence proves graph+read-path is insufficient.

## Cross-Document Decisions That Repeat

- **Three-layer architecture**: compiler truth (rust-analyzer/tree-sitter) + LLM judgment + graph algorithms.
- **Contract-first build**: explicit WHEN/THEN/SHALL gates, especially around key formats, storage getter unification, and source-read accuracy.
- **Addressability over duplication**: stable entity/chunk addressing (`file + line span + key discipline`) appears as a recurring pattern.
- **Adoption strategy**: expose capability via MCP/HTTP/CLI surfaces; position as a shared intelligence substrate for AI coding tools.
- **Scope discipline**: keep V200 lean; defer context-packer/token-budget machinery and broader "nice-to-have" layers.

## Artifact-by-Artifact Summary

1. `AR000-Else-Block.md` — Binding decision frame documenting locked principles and non-revisitable choices.
2. `AR001-CONSOLIDATED_FEATURE_PMF_JOURNEY_MATRIX.md` — Consolidated PMF + journey mapping of feature packs.
3. `AR002-CONSOLIDATED_FEATURE_POSSIBILITIES_DEDUP.md` — Deduplicated union of possible features and options.
4. `AR003-CR-cachebro-202601.md` — Cachebro competitive thesis and relevant strategic learnings.
5. `AR004-CR-codex-architecture.md` — Deep architecture decomposition of Codex-like system structure.
6. `AR005-CR-codex-eli5-ascii-summary.md` — ELI5/ascii simplification of Codex architecture findings.
7. `AR006-CR-codex-graph-overview.md` — Graph-oriented overview of Codex-style code intelligence model.
8. `AR007-CR-codex-implications.md` — Product/architecture implications of Codex research for V200.
9. `AR008-CR-codex-vs-factory-droid.md` — Comparative analysis between Codex and Factory Droid approaches.
10. `AR009-CR-factory-droid-202601.md` — Factory Droid reverse/competitive thesis and extracted signals.
11. `AR010-CR-v173-01.md` — Parse failure and low-coverage repository failure-mode analysis.
12. `AR011-CR-v173-02.md` — Deeper competitor code pattern extraction applied to Parseltongue.
13. `AR012-CR-v173-03.md` — Code-level competitor feature deep-dive and transfer opportunities.
14. `AR013-CR-v173-04-oh-my-pi.md` — Competitive study of oh-my-pi workflow/capability patterns.
15. `AR014-CR07-codex-research-progress-tracker.md` — Progress tracker for Codex research stream.
16. `AR015-ES-Decision-log-01.md` — Moved/placeholder reference for decision-log relocation.
17. `AR016-ES-Dependency-Graph-Contract-Hardening.md` — Dependency-graph contract hardening and enforcement model.
18. `AR017-ES-Hashing-Risks-v01.md` — Hashing/key-stability risks and mitigation strategy for V200.
19. `AR018-ES-User-Journey-01.md` — User-journey framing for V200 interactions and outcomes.
20. `AR019-ES-User-Journey-Addendum-Tauri-CLI-Philosophy.md` — Reframe: Tauri as visual CLI launcher, not core logic host.
21. `AR020-ES-attempt-01.md` — Executable contract ledger and initial design-contract synthesis.
22. `AR021-FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md` — Finalized feature extraction matrix and status consolidation.
23. `AR022-IMPLEMENTATION-v173-slim-snapshot-plan.md` — v1.7.3 slim snapshot implementation plan.
24. `AR023-MASTER_FEATURE_EXTRACTION_TABLE.md` — Master extraction table with TDD/session-state orientation.
25. `AR024-PRD-Level01.md` — Early PRD level for Parseltongue v2.0.0 requirement framing.
26. `AR025-PRD_v173.md` — v1.7.3 PRD (dual-mode graph server + MCP + desktop app scope).
27. `AR026-Prep-Competitive-Deep-Dive.md` — Competitive deep-dive prep structure and research agenda.
28. `AR027-Prep-Compiled-Research-Best-Ideas.md` — Compiled high-signal ideas selected from prep research.
29. `AR028-Prep-Cross-Language-Detection-Heuristics.md` — Cross-language boundary detection heuristics for V200.
30. `AR029-Prep-Datalog-Ascent-Rule-Patterns.md` — Datalog/Ascent rule templates for graph reasoning.
31. `AR030-Prep-Key-Format-Design.md` — Critical key-format decision space and tradeoff analysis.
32. `AR031-Prep-LLM-Context-Optimization-Research.md` — Context optimization research and strategy options.
33. `AR032-Prep-MCP-Protocol-Integration.md` — MCP integration patterns, protocol concerns, and fit.
34. `AR033-Prep-Max-Adoption-Architecture-Strategy.md` — Architecture options optimized for adoption/distribution.
35. `AR034-Prep-Rust-Analyzer-API-Surface.md` — rust-analyzer API surface study for semantic enrichment.
36. `AR035-Prep-Tree-Sitter-Query-Patterns.md` — Tree-sitter query patterns for structured extraction.
37. `AR036-Prep-isgl1-ambiguity-risk-table.md` — Expanded ambiguity/risk table for ISGL1 key model.
38. `AR037-Priortization-v173.md` — Prioritization decisions for the v1.7.3 feature set.
39. `AR038-RESEARCH-v173-rustanalyzer-explore-agent-raw-dump.txt` — Raw exploration dump from rust-analyzer research pass.
40. `AR039-RESEARCH-v173-rustanalyzer-semantic-supergraph.md` — Semantic supergraph thesis based on rust-analyzer grounding.
41. `AR040-Research-Extraction-Tools-Per-Language.md` — Language-by-language dependency extraction tooling survey.
42. `AR041-THESIS-v173-slim-graph-address-model.md` — Thesis for slim graph + address model in pt02/pt03.
43. `AR042-ZAI-PRD-contracts-01.md` — Initial V200 PRD contracts draft.
44. `AR043-ZAI-PRD-contracts-02.md` — Final V200 PRD contracts synthesis (architecture, gates, APIs, probes).
45. `AR044-ZAI-REUSABLE-Patterns-01.md` — Reusable implementation/product patterns extracted from v173.
46. `AR045-ZAI-doc-progress-tracker.md` — Documentation-analysis progress tracker and completion status.
47. `AR046-drawio-diagram-lessons-learned.md` — Lessons learned from draw.io diagramming process.
48. `AR047-drawio-FUJ-Final-User-Journey.drawio` — Source draw.io asset for final FUJ diagram.
49. `AR048-v173-pt04-bidirectional-workflow.md` — Bidirectional workflow thesis upgraded to compiler-truth-first three-layer model.
50. `AR049-v199-Notes.md` — V199/V200 notes with companion-app boundary clarification.
51. `AR050-FUJ-Final-User-Journey.md` — Canonical final user journey and control-flow spine.
52. `AR052-v210-backlog.md` — V210 backlog and explicit deferral of context-packer from V200 scope.
53. `AR053-Research-Graph-Workflow-Internet-Synthesis-20260227.md` — Internet synthesis for graph/dependency workflows relevant to V200.
54. `AR054-CR-codemogger-v200-options-20260227.md` — Codemogger analysis with concrete V200 option set.

## Net Product Direction (from the full corpus)

- Build V200 around **graph correctness and deterministic retrieval contracts**.
- Keep **line-addressable source references** as first-class identifiers for retrieval and tooling interop.
- Treat semantic enrichment (rust-analyzer + tree-sitter) as **ground truth substrate**, not optional add-on.
- Use LLMs where they add judgment; avoid using them for facts compilers already resolve.
- Defer optimization layers until measured evidence shows necessity.
