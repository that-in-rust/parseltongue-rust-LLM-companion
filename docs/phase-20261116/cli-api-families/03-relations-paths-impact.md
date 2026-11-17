# Typed relations, paths and impact

Site-preserving navigation, witness paths, raw bounded subgraphs, ownership and impact candidates.

**Status:** every API entry is proposed only; none is shipped. X feasibility items are separately listed as rejected concepts with no CLI verb. Priority and feasibility classes are defined in [`CLI-query-backlog.md`](../CLI-query-backlog.md).

<a id="family-evidence"></a>

## Shared evidence profile

Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

<a id="family-bounds"></a>

## Shared bounds and completeness profile

Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.


<a id="cli-rel-001"></a>

### CLI-REL-001 — `query-list-typed-neighbors` · P0 · E

**Job:** List direct incoming/outgoing neighbors for an explicit relation allowlist.

**Inputs:** Item ID; direction; relation list; scope/limit.

**Outputs:** Per-site neighbors, resolution and evidence IDs.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Published edge rows and generation header.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-002"></a>

### CLI-REL-002 — `query-list-direct-callers` · P0 · E

**Job:** Show direct recorded caller sites only when the stored target identity matches; never attach targetless unresolved calls to the requested item.

**Inputs:** Item ID; resolution filter; scope/limit.

**Outputs:** Only caller sites whose recorded target identity matches the requested item; any targetless unresolved sites are separately labelled scoped unknowns and are not attributed to this item.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** `calls` edges with recorded target identity; scoped unresolved-site listing is separate.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-003"></a>

### CLI-REL-003 — `query-list-direct-callees` · P0 · E

**Job:** Show direct recorded calls originating at one item.

**Inputs:** Item ID; resolution filter; scope/limit.

**Outputs:** Call sites originating at this item, with resolved targets and source-attributable unresolved targetless sites labelled as such; no guessed target.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** `calls` edges and edge evidence.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-004"></a>

### CLI-REL-004 — `query-list-reference-sites` · P1 · E

**Job:** List only the occurrence/relation kinds actually represented in the published graph.

**Inputs:** Item ID; relation/role allowlist; limit.

**Outputs:** Individual sites with exact role only when stored; unsupported roles are explicit.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Edge evidence and occurrence identities; no universal reference claim.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-005"></a>

### CLI-REL-005 — `query-list-import-bindings` · P1 · E

**Job:** Show recorded import/re-export sites and resolved targets.

**Inputs:** Module/file/item ID; direction; scope.

**Outputs:** Import site, alias/target fields if present, resolution and evidence.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** `imports` / `reexports` edge facts.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-006"></a>

### CLI-REL-006 — `query-list-type-dependencies` · P1 · E

**Job:** List recorded `uses_type` sites without inferring additional type flow.

**Inputs:** Item ID; direction; relation/site cap.

**Outputs:** Source sites and typed target status.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** `uses_type` edges and source ranges.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-007"></a>

### CLI-REL-007 — `query-inspect-impl-members` · P1 · C

**Job:** Inspect implementation-header member facts where the stored join is actually supported.

**Inputs:** Impl/container identity; optional self/trait filters.

**Outputs:** Member occurrences, header-path coverage and missing join fields.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Impl-header index; header coverage is limited and row join is unverified.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-008"></a>

### CLI-REL-008 — `query-list-trait-implementations` · P2 · C

**Job:** Find possible recorded implementations without converting trait calls into every body.

**Inputs:** Trait/member ID; scope; bound.

**Outputs:** Implementation records and evidence, with dispatch uncertainty.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Impl-member facts; actual compatible rows and semantic ownership join.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-009"></a>

### CLI-REL-009 — `query-explain-call-resolution` · P1 · E

**Job:** Explain the recorded resolution state for one call site.

**Inputs:** Site/evidence ID.

**Outputs:** Resolved target or typed unresolved/inferred/proposed reason and source anchor.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Edge evidence and resolution enum.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-010"></a>

### CLI-REL-010 — `query-trace-declared-paths` · P1 · D

**Job:** Return a bounded witness path through allowed recorded relations.

**Inputs:** Start ID; relation allowlist; direction/depth/limit.

**Outputs:** Ordered path with per-edge site evidence.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Stable graph rows; bounded traversal.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-011"></a>

### CLI-REL-011 — `query-find-connecting-paths` · P2 · D

**Job:** Find a bounded set of distinct connection paths between two IDs.

**Inputs:** From/to IDs; direction; relation allowlist; max paths/depth.

**Outputs:** Witness paths and explicit search-bound status.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Graph traversal; does not establish runtime connection.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-012"></a>

### CLI-REL-012 — `query-estimate-change-impact` · P1 · D

**Job:** Enumerate static candidate dependents reachable from a declared change scope.

**Inputs:** Changed item IDs; file/range seeds only where stored anchors exist; relation allowlist; depth/node caps.

**Outputs:** Mapped impact candidates with witness paths plus a separate unmapped-seed list; no parsing/name guessing and no “what breaks” claim.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Stored item/site anchors and selected relation rows; unmapped file/range seeds remain visible.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-013"></a>

### CLI-REL-013 — `query-list-ownership-chain` · P2 · C

**Job:** Show separately recorded lexical, type, trait and impl ownership links.

**Inputs:** Item ID; ownership kinds; depth.

**Outputs:** Known ownership chain and unavailable links without name matching.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Owner-kind and impl records; direct owner target is not established.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-014"></a>

### CLI-REL-014 — `query-inspect-macro-provenance` · P1 · E

**Job:** Return available generating-call and token-origin evidence for a generated item/site.

**Inputs:** Generated item or site ID; origin-depth cap.

**Outputs:** Definition/invocation boundary origins and explicit mapping gaps.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Macro expansion/origin tables; not a complete edit map.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)


<a id="cli-rel-015"></a>

### CLI-REL-015 — `query-export-bounded-subgraph` · P1 · D

**Job:** Export the raw selected node/edge subgraph as bounded data, distinct from a reason-ranked context receipt.

**Inputs:** Root item IDs or stored path anchors; relation allowlist; boundary/induced scope; depth/node/edge/byte caps.

**Outputs:** Raw selected node and edge rows with site evidence, snapshot identity and truncation/completeness; no summary ranking.

**Evidence:** Preserve each recorded source site, relation kind, resolution state and evidence identity. A targetless unresolved site cannot be attributed to a requested caller target; list it only as a separate scoped unknown, never as that item’s caller.

**Bounds/completeness:** Bound depth/nodes/edges/paths/bytes; state boundary vs induced scope; scan completeness differs from semantic completeness. No caller/path result is not proof of absence. File/range impact seeds without stored anchors are returned as unmapped; never parse or guess.

**Dependencies:** Published node/edge rows and stable IDs; caller supplies scope and bounds.

**Sources:** [S80REL: supplements/80-repository-synthesis-draft.md L165–L179 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80CONTRACT: supplements/80-repository-synthesis-draft.md L188–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301); [FIMPL L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171)
