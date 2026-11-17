# CLI query backlog — research draft, not a product contract

## Answer first

This is a broad, source-traceable menu of **64 distinct proposed operations plus 6 rejected X concepts (which have no CLI verbs)** for a read-only query surface over one compatible published generation of pinned itoa 1.0.18. **None is shipped.** This is documentation-only ideation, not a command/schema contract or producer acceptance result. The production feasibility review inspected code/schema paths but did not open published rows; every `E` item remains a candidate until compatible rows are validated. [Current phase boundary](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/0f4cc3367a5f7a352307baf900b6ec050ad38dff/docs/phase-20261116/ReadMe.md#L14-L39); [published export route](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/0f4cc3367a5f7a352307baf900b6ec050ad38dff/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L66-L101).

The producer/dataset boundary is unchanged: no new extraction, graph/schema/dataset/database, embeddings, LLM-generated facts, edits, reindexing, HTTP/MCP implementation or multi-repository claim. A missing field makes only the dependent query/field conditional; it is **not** work assigned to this CLI phase. Root `README.md` endpoints are historical lineage, not current runtime evidence. [phase scope](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/0f4cc3367a5f7a352307baf900b6ec050ad38dff/docs/phase-20261116/ReadMe.md#L20-L39); [historical root README](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305).

## How to read the backlog

- **Priority:** P0 = first useful slice; P1 = common navigation after that; P2 = valuable derived/advanced query; Later = explicit deferral, not planned implementation.
- **Feasibility:** E = existing-fact candidate; D = derived read-only; C = conditional on missing or unverified published facts; X = outside this phase. E is not proof of a populated accepted generation; D still needs compatible input rows and deterministic output; C does not request producer enrichment.
- Every API ID has one unique four-word `query-...` verb. Exact/prefix/fuzzy are modes of one search verb; snapshot sections are modes of one compare verb. Pagination/truncation are envelope fields/options, not extra verbs. X concept IDs have no verb and are not commands. No `--ndjson` or pagination-only command is added.
- Shared family evidence/bounds rules are factored into each family page; every entry still states job, operands, outputs, evidence profile, completeness/bounds, dependencies and clickable source citations.

### Curated families
- [Orientation and capabilities](cli-api-families/01-orientation-capabilities.md)
- [Symbol and source lookup](cli-api-families/02-symbol-source-lookup.md)
- [Typed relations, paths and impact](cli-api-families/03-relations-paths-impact.md)
- [Progressive context and conversation](cli-api-families/04-progressive-context.md)
- [Derived structure and analytics](cli-api-families/05-derived-structure-analytics.md)
- [Evidence and diagnostics](cli-api-families/06-evidence-diagnostics.md)
- [Compatible snapshot comparison](cli-api-families/07-snapshot-comparison.md)
- [Conditional and deferred ideas](cli-api-families/08-conditional-deferred.md)

## Shared query contract (proposed)

1. Open stores read-only; do not create/migrate/reindex. Bind each query result and every table within one input to one generation digest **and** attempt sequence. For a two-snapshot comparison, left/right identities are expected to differ; enforce consistency within each side and report cross-side parser/config/unit/source differences. Cursors/receipts are self-contained caller-held JSON, not server-side result/session IDs. Current export reads the publication scope and does not expose historical generation selection; no published rows were inspected. [header](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/0f4cc3367a5f7a352307baf900b6ec050ad38dff/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L11-L23), [scope/export](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/0f4cc3367a5f7a352307baf900b6ec050ad38dff/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L66-L101).
2. JSON is canonical; text is a rendering of the same typed result. Diagnostics/progress go to stderr. Operation discovery supplies versioned schemas. Do not create a separate API for output format or pagination.
3. Exact, prefix and fuzzy modes are explicit. Lexical/graph ranking selects candidates only; it is heuristic, never a source fact. Candidate IDs must be real published IDs; stable IDs are snapshot-bound, not names/lines.
4. Preserve typed relations and individual source sites. Keep extracted/proven, unresolved, gap, inferred, proposed, external, excluded and unsupported states distinct. A missing edge or empty result never proves unused code, no runtime path, safety or a complete scan. [requirements (review bundle: supplements/80-repository-synthesis-draft.md L34–L43)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz).
5. Distinguish scan/page completeness, semantic resolution, source/build scope and capability availability. Unknown is not zero. Bound rows, bytes, nodes, edges, paths, depth and time. Cursors bind generation/query/filters/order/algorithm version. A budget too small for one evidence-complete record returns typed `too_small`, not malformed JSON or stripped evidence. [response contract (review bundle: supplements/80-repository-synthesis-draft.md L188–L263)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz).
6. Current source is editable only after digest/range verification; historical source can remain valid historical evidence. No semantic extraction is run while reading source. Token counts require a named tokenizer/version or a labelled estimate; no throughput, latency or token-savings claim without measurement.
7. External LLM reasoning stays with the caller. Caller-supplied focus IDs, rules, weights or seen receipts affect selection/evaluation only; they never become stored facts or persistent learned memory.

## Proposed response envelope and exit statuses

JSON is canonical; text is a rendering of the same typed result. The shape below is illustrative, not an implemented schema or observed output.

```json
{
  "schema_version": "navigation.v1",
  "operation": "query-list-direct-callers",
  "snapshot": {"generation_digest": "<digest>", "attempt_sequence": "<sequence>"},
  "scope": {"mode": "boundary", "paths": ["src/**"]},
  "result_state": "partial",
  "items": [],
  "completeness": {"scan": "complete", "page": "truncated", "semantic_resolution": "partial", "total": null},
  "continuation": {"receipt": "<self-contained caller-held JSON>"},
  "warnings": []
}
```

| Exit | Meaning | Required envelope/result behavior |
|---:|---|---|
| 0 | Valid result, including empty or partial | Structured state explains empty/partial/truncated; partial is not failure. |
| 1 | Operational failure | Typed diagnostic; no fabricated result. |
| 2 | Invalid request | Identify invalid operand/option combination. |
| 3 | Ambiguous exact operand | Return alternatives; never silently select one. |
| 4 | Exact operand absent within declared lookup scope | Do not fall back to fuzzy mode. |
| 5 | Source or snapshot drift | Keep historical evidence labelled; do not call it current editable source. |
| 6 | Unsupported capability | Distinct from an empty successful result. |
| 7 | Resource limit before any valid result | Typed limit failure; if complete records already exist, return status 0 plus explicit truncation/continuation. |

Diagnostics/progress belong on stderr. Unknown totals stay null/unknown, not zero. A too-small budget must not strip evidence or emit malformed JSON. These are proposed acceptance semantics, not tests run. [response contract (review bundle: supplements/80-repository-synthesis-draft.md L254–L263)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz).

## Five phases / seven events (illustrative protocol)

The historic five-phase/seven-event proposal is retained as a retrieval protocol, not as a promise that its missing public-visibility, signature, control/data-flow or git facts exist. Fallback to named lexical/graph neighborhoods and mark unavailable fields. Ranking and budget selection are heuristic; only recorded IDs, sites and source bytes can be evidence. [five phases/seven events](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L9-L50); [heuristic vs exact](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108).

| Phase | Events | Proposed behavior |
|---|---|---|
| 1. Search | 1–2 | Caller supplies terms; return real item IDs and lexical match reasons. Do not require a semantic model or git recency. |
| 2. Anchor | 3 | Follow only available recorded containment/ownership relations; if public visibility or a boundary is absent, return the hit itself and label the anchor fact unavailable. |
| 3. Cluster | 4 | Preview bounded candidate cards/neighborhoods. Heuristic ordering is labeled; whole evidence records are preserved. |
| 4. Disambiguate | 5–6 | Present cards to caller/external LLM; caller chooses an ID, chooses **none**, asks for **next**, or quits. `none` is valid; `next` returns the next bounded page, not a guessed choice. |
| 5. Deep dive | 7 | For a chosen ID, compose exact available source, relationships, evidence and unknowns within limits; control/data-flow, git history or contracts appear only if separately supported. |

Seven events: (1) caller query; (2) candidate list; (3) available anchor/via chain; (4) bounded neighborhood cards; (5) cards presented for choice; (6) choose ID / none / next / quit; (7) bounded evidence/source deep dive. The LLM never executes inside the CLI. Historical token allocations are examples, not measurements.

## Multi-turn recipes (all illustrative; not runnable today)

1. **Unfamiliar code:** `CLI-ORI-001` → `CLI-SYM-001` with `--match exact|prefix|fuzzy` → `CLI-SYM-005` → `CLI-REL-002`/`CLI-REL-003` → `CLI-SYM-007`. Stop where evidence ends.
2. **Ambiguous term:** `CLI-SYM-001 --match fuzzy` returns cards + reasons → external caller chooses a stable ID, `none`, or `next` → `CLI-SYM-004` compares candidates → only then `CLI-SYM-005`/`CLI-SYM-007`.
3. **Change review:** `CLI-REL-012` (P1) returns mapped impact candidates + witness paths and a separate unmapped-seed list → `CLI-REL-010` expands a selected path; `CLI-REL-015` exports a bounded raw subgraph separately from the reason-ranked context bundle → `CLI-EVD-001` reads evidence → `CLI-SYM-007` checks digest-matched source. `CLI-SYM-009` can find stored span overlaps at a byte position/range but is not compiler binding or scope-variable resolution. No break-probability or safety claim.
4. **Macro-generated item:** `CLI-REL-014` returns available boundary origins → `CLI-EVD-004` explains missing mapping → `CLI-CND-004` is conditional; do not fabricate an edit location.
5. **Architecture overview:** `CLI-ORI-004` → `CLI-ANA-002` folder rollups → `CLI-ANA-003` leaf drilldown → `CLI-REL-001` site evidence. Group by recorded source paths, not inferred module links.
6. **Resume context:** `CLI-CTX-008` exports a caller-held receipt bound to generation/query → later `CLI-CTX-006` returns unseen items for that same snapshot; a separate left/right comparison may use distinct snapshots, while each side must remain internally consistent.
7. **Unsupported field:** `CLI-EVD-004` returns a typed missing/unsupported state for signature, field, module-file or runtime fact → continue with available neighbors; no producer work is assigned here.

## Merged modes and compatibility IDs (not extra verbs)

| Former stable ID | Canonical operation | Retained mode / meaning | Separate command? |
|---|---|---|---|
| `CLI-SYM-002` | `CLI-SYM-001` `query-search-symbol-candidates` | `--match prefix` | No |
| `CLI-SYM-003` | `CLI-SYM-001` `query-search-symbol-candidates` | `--match fuzzy` (heuristic ranking) | No |
| `CLI-CTX-007` | `CLI-CTX-003` `query-build-evidence-context` | Self-contained cursor receipt option | No |
| `CLI-EVD-005` | `CLI-EVD-003` `query-explain-result-completeness` | Truncation/continuation fields | No |
| `CLI-SNP-002` | `CLI-SNP-001` `query-compare-published-snapshots` | `section=relations` | No |
| `CLI-SNP-003` | `CLI-SNP-001` `query-compare-published-snapshots` | `section=source-hashes` | No |

All result/cursor references are caller-held self-contained JSON; no server-side result/session state is required.

## First slice and proposed acceptance cases

**Small first slice:** `CLI-ORI-001`/`002`/`003`, `CLI-SYM-001`/`005`/`007`, `CLI-REL-001`/`002`/`003`, and `CLI-EVD-001`/`003`/`004`. This is status/capability, exact symbol selection, available source, recorded relation exploration and evidence—not a requirement to implement the whole menu.

Proposed cases (not tests claimed run):

- no publication vs published-but-empty table vs unsupported field produce distinct states;
- exact no-match and ambiguous exact name do not silently fuzzy-select;
- fuzzy ordering is deterministic, exposes method/match reasons, and never claims truth;
- source reads verify source digest and return typed drift/missing-source outcomes;
- duplicate source sites remain distinct and unresolved/inferred/proposed states survive filtering;
- a too-small budget never truncates mid-record or removes evidence; continuation rejects a different generation/query;
- complete scan with unresolved sites differs from truncated page and unknown totals;
- a folder rollup reconciles to enumerated leaf contributors and discloses unknown coverage;
- analytics output is repeatable for identical parameters and reports algorithm/version/seed/direction/relations;
- caller-supplied rules yield `unknown` where facts are absent, with witnesses for evaluations;
- snapshot comparison refuses mixed attempt/configuration identity and reports ambiguous item matches;
- choose `none`, `next`, and `quit` are valid protocol transitions without implicit selection.

## Analytics: keep the formula, reject the story

Retain mathematically derived views only when the input graph, formula, direction, relation allowlist, algorithm/version and seed are explicit. Useful candidates include degree distributions, SCC/cycle membership, k-core numbers, centrality, graph partitions, entropy over a named distribution, relation motifs, file/folder count rollups and cross-boundary edge counts with leaf drilldown. These describe the selected graph projection. They do **not** establish technical debt hours, code complexity, runtime bottlenecks, defect probability, safe extraction, “dead code,” or a proven architecture violation. Caller-supplied rules may be evaluated against available facts; missing facts return unknown. This narrows broad reviewer skepticism without treating every metric as useful. [metric boundary (review bundle: supplements/80-repository-synthesis-draft.md L182–L186)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [README historical metric routes](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L297-L305); [rollup research](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68).

## 80-repository evidence limits

The synthesis reconciles reports and spot-checked selected sources; it is not 80 runtime installations, a common benchmark, an exhaustive market census or proof advertised features work. Batch 2 includes README-only findings; other batches mix code paths, pinned docs and unpinned descriptions. Lite-session reports do not establish uniform model use or `codebase-memory` use. Community/vendor/first-person results are not independent confirmations; Graphify and CodeGraph experiments are narrow evidence, not product outcomes. Stars informed discovery only. No accuracy, speedup, token saving or market-fit claim is imported as a priority score. [research method/caveats (review bundle: supplements/80-repository-synthesis-draft.md L16–L26)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [quality caveats (review bundle: supplements/80-repository-synthesis-draft.md L356–L415)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz).

## Legacy route map and complete inventory

- [All 26 distinct historical root-README routes, individually mapped](legacy-route-map.md). The canonical coverage/method summary is [markdown-review-coverage.md](markdown-review-coverage.md). Duplicate query-string variants are options on the same method/path, not new CLI verbs.
- [Every frozen repository and supplemental Markdown path, one-by-one](markdown-source-inventory.md). The frozen denominator remains 393 repository + 34 supplemental paths; the three later document versions are delta notes, not additional denominator rows.
- The raw ledgers, reports and review-validation artifacts remain external supporting attachments. The machine manifest records their filenames/hashes and all API/range-idea counts; this draft does not turn report-level `full_text` metadata into independent proof of reading.

## Deferred and rejected rationale

- Historical HTTP/CozoDB route spelling is lineage only; no HTTP/MCP runtime is proposed. Reindex/watch/edit/mutation routes are X/out-of-phase.
- Approximate lexical/graph ranking may help selection only and must say which score/algorithm produced ordering; it is never source truth.
- No new extractor, schema/table/database, embedding index, persistent learned memory or LLM-generated fact is authorized. Missing facts block only their dependent query.
- Reject SQALE/debt hours, “complexity” from proxy counts, risk/safety scores, runtime-impact claims, and test-impact/coverage claims absent measured facts.
- Keep direct evidence and derived views separate; do not average proven and unresolved rows into a confidence score.
