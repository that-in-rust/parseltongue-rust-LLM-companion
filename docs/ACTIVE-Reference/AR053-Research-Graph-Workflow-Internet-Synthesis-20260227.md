# AR053: Internet Synthesis for Graph + Dependency Workflows
Date: 2026-02-27
Status: Draft reference (web-verified)

## Scope
This note maps external research and industry practices to Parseltongue workflows defined in:
- `/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/README.md`
- `/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/ACTIVE-Reference/AR048-v173-pt04-bidirectional-workflow.md`
- `/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/ARCHIVE/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md`
- `/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/ACTIVE-Reference/AR027-Prep-Compiled-Research-Best-Ideas.md`

## One-line Thesis
Best practice in 2025-2026 converges on: **semantic retrieval for recall, graph/dependency traversal for truth, LLM for judgment only on ambiguous cases**.

This aligns with AR048's three-layer model:
1. Compiler/semantic truth
2. LLM judgment
3. Fast graph algorithms

## External Evidence (What the Internet Shows)

### 1) Precise code navigation beats pure text search for dependency truth
- LSP 3.17 formalizes call hierarchy/type hierarchy primitives for deterministic navigation.
- Sourcegraph explicitly uses precise navigation when available and search-based fallback otherwise.
- SCIP standardizes language-agnostic precise code intelligence indexing.

Implication for Parseltongue:
- Keep dependency/caller/callee answers graph-backed first.
- Use fuzzy/semantic search as fallback and candidate expansion, not core truth.

### 2) Call graph quality depends on typed semantics and runtime ambiguity handling
- CodeQL docs distinguish declared target vs possible runtime callees.
- For dynamic languages, call graph precision is inherently variable and should expose imprecision.

Implication for Parseltongue:
- Typed edge metadata (`direct`, `trait`, `dyn`, `closure`, etc.) should stay first-class.
- Responses should expose confidence/imprecision fields for non-Rust or ambiguous edges.

### 3) GraphRAG systems now use hybrid retrieval, not graph-only or vector-only
- Microsoft GraphRAG local search combines KG + text chunks.
- DRIFT/global approaches optimize cost by pruning and staged retrieval.
- KET-RAG reports multi-granular indexing can cut cost while preserving quality.

Implication for Parseltongue:
- Add dual-lane retrieval for LLM context:
  - Lane A: graph neighbors (high-trust)
  - Lane B: semantic chunks (high-recall)
- Merge with provenance and truth-grade gating.

### 4) Dependency workflows are now path-centric and transitivity-aware
- GitHub dependency graph emphasizes direct vs transitive and "show paths".
- Semgrep dependency search adds dependency path visualization and depth control.

Implication for Parseltongue:
- Keep blast radius count, but add explicit path rendering modes:
  - shortest path(s)
  - top-K critical transitive chains
  - path-class breakdown by edge type

### 5) Code property graph practice validates multi-view graph overlays
- Joern CPG merges AST/CFG/PDG views and supports layered analysis/export.

Implication for Parseltongue:
- Treat semantic overlays (pt04 typed relations) as layered graph enrichment.
- Avoid flattening everything into one undifferentiated edge type.

### 6) Graph-construction trust is now a security concern
- Recent GraphRAG poisoning research shows small text edits can distort generated graphs when extraction is LLM-led.

Implication for Parseltongue:
- BR01/BR06 direction is correct: deterministic extraction and explicit evidence promotion gates.
- LLM-extracted relations must remain non-canonical until validated.

## Direct Mapping to Current README Workflows

### Workflow: Orient -> Search -> Read -> Trace -> Blast Radius -> Smart Context
Recommended upgrade:
1. `Search` step becomes hybrid candidate generation (semantic + fuzzy + type filters).
2. `Trace` and `Blast Radius` remain deterministic graph truth.
3. `Smart Context` merges graph neighborhood + semantic evidence with confidence/provenance.

### Workflow: Bug hunting via reverse/forward traversal
Recommended upgrade:
1. Start with deterministic callers/callees.
2. Add semantic "gap scan" for likely missed entry points by naming mismatch.
3. Rank suspects by typed path criticality + centrality, not similarity alone.

### Workflow: Safe refactoring
Recommended upgrade:
1. Keep typed blast radius as primary decision signal.
2. Add "path categories" summary:
   - trait boundary crossings
   - direct internal calls
   - async/closure-heavy chains
3. Use LLM only for strategy wording and tradeoff explanation.

## Endpoint-level Design Moves (Low risk, high value)

1. Add retrieval mode controls to context/search endpoints:
   - `mode=graph_only|hybrid|semantic_only`
   - default `hybrid` for discovery endpoints, `graph_only` for risk/impact endpoints

2. Add path-centric blast radius options:
   - `include_paths=true`
   - `path_limit=20`
   - `path_strategy=shortest|diverse|critical`

3. Add trust annotations on every mixed result:
   - `match_source=graph_exact|semantic_hint`
   - `truth_grade=verified|heuristic|evidence_only`
   - `confidence` + `provenance`

4. Add progressive disclosure to reduce token costs:
   - `detail=summary|standard|full`

## What Not To Do
1. Do not let similarity rank override deterministic dependency truth.
2. Do not promote LLM-extracted graph relations to canonical without validation gates.
3. Do not collapse typed edges into generic "calls" if you need accurate impact reasoning.

## Sources (web)
- LSP 3.17 specification: call hierarchy/type hierarchy
  - https://ntaylormullen.github.io/language-server-protocol/specifications/specification-3-17/
- Sourcegraph precise code navigation and SCIP
  - https://sourcegraph.com/docs/code_intelligence/explanations/precise_code_intelligence
  - https://sourcegraph.com/docs/code-search/code-navigation/writing_an_indexer
- CodeQL call graph guidance
  - https://codeql.github.com/docs/codeql-language-guides/codeql-library-for-go/
  - https://codeql.github.com/docs/codeql-language-guides/codeql-library-for-javascript/
  - https://codeql.github.com/docs/codeql-language-guides/navigating-the-call-graph/
- Microsoft GraphRAG query engine and search modes
  - https://microsoft.github.io/graphrag/query/overview/
  - https://www.microsoft.com/en-us/research/blog/introducing-drift-search-combining-global-and-local-search-methods-to-improve-quality-and-efficiency/
  - https://www.microsoft.com/en-us/research/blog/graphrag-improving-global-search-via-dynamic-community-selection/
- KET-RAG (multi-granular GraphRAG indexing)
  - https://arxiv.org/abs/2502.09304
- Dependency graph workflows
  - https://docs.github.com/en/code-security/concepts/supply-chain-security/about-the-dependency-graph
  - https://semgrep.dev/docs/semgrep-supply-chain/dependency-search
- Code Property Graph references
  - https://docs.joern.io/code-property-graph/
  - https://docs.joern.io/export/
  - https://cpg.joern.io/
- GraphRAG poisoning risk
  - https://arxiv.org/abs/2508.04276

