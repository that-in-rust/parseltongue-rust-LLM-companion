# Conditional and deferred ideas

Missing/unverified facts remain query-level conditions; X concepts are rejected and have no command verbs.

**Status:** every API entry is proposed only; none is shipped. X feasibility items are separately listed as rejected concepts with no CLI verb. Priority and feasibility classes are defined in [`CLI-query-backlog.md`](../CLI-query-backlog.md).

<a id="family-evidence"></a>

## Shared evidence profile

Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

<a id="family-bounds"></a>

## Shared bounds and completeness profile

Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.


<a id="cli-cnd-001"></a>

### CLI-CND-001 — `query-map-module-files` · P2 · C

**Job:** Return module-to-file associations only if a direct recorded relation is available.

**Inputs:** Module/file IDs; relation scope.

**Outputs:** Stored association sites or typed `unsupported`/`missing` state.

**Evidence:** Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

**Bounds/completeness:** Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.

**Dependencies:** No direct module/file link verified; never infer from names.

**Sources:** [S80FIELD: supplements/80-repository-synthesis-draft.md L127–L139 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FVIS L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FOWN L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171); [FMACRO L14–L30](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/declarative-macro-expansion-records.rs#L14-L30); [FBACKLOG L198–L220](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261113/immediate-backlog.md#L198-L220); [EDITORIAL: inputs/editorial-design.md L20–L39 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-cnd-002"></a>

### CLI-CND-002 — `query-find-field-occurrences` · P2 · C

**Job:** Find field definitions/reads/writes only when those occurrence facts are published.

**Inputs:** Field/source IDs; role filter.

**Outputs:** Field sites and roles or explicit missing-capability response.

**Evidence:** Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

**Bounds/completeness:** Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.

**Dependencies:** Field navigation is not established in current row audit.

**Sources:** [S80FIELD: supplements/80-repository-synthesis-draft.md L127–L139 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FVIS L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FOWN L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171); [FMACRO L14–L30](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/declarative-macro-expansion-records.rs#L14-L30); [FBACKLOG L198–L220](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261113/immediate-backlog.md#L198-L220); [EDITORIAL: inputs/editorial-design.md L20–L39 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-cnd-003"></a>

### CLI-CND-003 — `query-show-symbol-signatures` · P1 · C

**Job:** Return a signature only when the optional signature fact is populated and source-matched.

**Inputs:** Item ID; range kind; digest policy.

**Outputs:** Exact signature or typed unknown/missing result.

**Evidence:** Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

**Bounds/completeness:** Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.

**Dependencies:** Optional signature fields; published values unverified.

**Sources:** [S80FIELD: supplements/80-repository-synthesis-draft.md L127–L139 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FVIS L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FOWN L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171); [FMACRO L14–L30](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/declarative-macro-expansion-records.rs#L14-L30); [FBACKLOG L198–L220](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261113/immediate-backlog.md#L198-L220); [EDITORIAL: inputs/editorial-design.md L20–L39 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-cnd-004"></a>

### CLI-CND-004 — `query-map-macro-origins` · Later · C

**Job:** Map generated output to editable template/invocation spans only where full provenance exists.

**Inputs:** Generated site/item ID; origin policy.

**Outputs:** Exact mapped source spans or explicit partial/unmapped state.

**Evidence:** Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

**Bounds/completeness:** Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.

**Dependencies:** Current origins are boundary-token evidence, not a complete token edit map.

**Sources:** [S80FIELD: supplements/80-repository-synthesis-draft.md L127–L139 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FVIS L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FOWN L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171); [FMACRO L14–L30](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/declarative-macro-expansion-records.rs#L14-L30); [FBACKLOG L198–L220](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261113/immediate-backlog.md#L198-L220); [EDITORIAL: inputs/editorial-design.md L20–L39 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-cnd-005"></a>

### CLI-CND-005 — `query-list-public-surface` · P2 · C

**Job:** List public/reachable items only when effective visibility facts are populated for the requested scope.

**Inputs:** Snapshot; crate/path scope; visibility kind; limit.

**Outputs:** Known visibility rows plus `Unknown`/unavailable coverage; no claim of a complete public API unless coverage proves it.

**Evidence:** Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

**Bounds/completeness:** Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.

**Dependencies:** Optional effective-visibility/reachability fields; actual published values unverified.

**Sources:** [S80FIELD: supplements/80-repository-synthesis-draft.md L127–L139 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FVIS L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FOWN L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171); [FMACRO L14–L30](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/declarative-macro-expansion-records.rs#L14-L30); [FBACKLOG L198–L220](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261113/immediate-backlog.md#L198-L220); [EDITORIAL: inputs/editorial-design.md L20–L39 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-cnd-006"></a>

### CLI-CND-006 — `query-list-type-methods` · P2 · C

**Job:** Navigate a type to methods only through a verified stored type/impl/member association.

**Inputs:** Type or impl identity; member kind; scope.

**Outputs:** Stored member occurrences and association provenance, or typed missing/join-unverified response.

**Evidence:** Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

**Bounds/completeness:** Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.

**Dependencies:** Impl-header/member rows and type identity; current join/row coverage unverified.

**Sources:** [S80FIELD: supplements/80-repository-synthesis-draft.md L127–L139 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FVIS L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FOWN L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171); [FMACRO L14–L30](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/declarative-macro-expansion-records.rs#L14-L30); [FBACKLOG L198–L220](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261113/immediate-backlog.md#L198-L220); [EDITORIAL: inputs/editorial-design.md L20–L39 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-cnd-007"></a>

### CLI-CND-007 — `query-read-symbol-contract` · P2 · C

**Job:** Return exact source-backed contract/safety/attribute sections for an item only where those sections are stored.

**Inputs:** Item ID; contract kind; source/digest policy.

**Outputs:** Exact cited spans or typed unsupported/missing state; no synthesized safety guarantee.

**Evidence:** Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

**Bounds/completeness:** Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.

**Dependencies:** Published doc/contract/attribute ranges and matching source bytes; not verified in current rows.

**Sources:** [S80FIELD: supplements/80-repository-synthesis-draft.md L127–L139 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FVIS L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FOWN L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171); [FMACRO L14–L30](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/declarative-macro-expansion-records.rs#L14-L30); [FBACKLOG L198–L220](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261113/immediate-backlog.md#L198-L220); [EDITORIAL: inputs/editorial-design.md L20–L39 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-cnd-008"></a>

### CLI-CND-008 — `query-list-symbol-docs` · P2 · C

**Job:** List exact documentation sections attached to an item when documentation ranges are published.

**Inputs:** Item ID; doc-section limit; source digest policy.

**Outputs:** Stored doc ranges with source provenance or typed unavailable state; no LLM summary/fact.

**Evidence:** Current schema/code is only a candidate source; actual compatible published rows and capability coverage are unverified.

**Bounds/completeness:** Return `unsupported`, `missing`, or `unknown`, not an empty success; absence does not authorize producer/schema work in this phase.

**Dependencies:** Published documentation ranges and item association; actual rows unverified.

**Sources:** [S80FIELD: supplements/80-repository-synthesis-draft.md L127–L139 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FVIS L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FOWN L129–L171](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-syntax-span-extractor/src/impl-header-source-records.rs#L129-L171); [FMACRO L14–L30](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/declarative-macro-expansion-records.rs#L14-L30); [FBACKLOG L198–L220](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261113/immediate-backlog.md#L198-L220); [EDITORIAL: inputs/editorial-design.md L20–L39 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


## Rejected/deferred X concepts — no CLI verbs

These stable IDs exist only to preserve route/reviewer rationale. They are not API records, do not have canonical verbs and are not commands to implement.


### CLI-X-001 — Semantic intent / embeddings · Later · X

**Canonical verb:** none; rejected concept, not a command.

**Reason:** No embeddings, semantic model or new retrieval index; exact/lexical navigation remains the bounded fallback.

**Status:** Rejected from this phase; not a CLI command.

**Sources:** [S80LIMITS: supplements/80-repository-synthesis-draft.md L16–L26 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROADMAP: supplements/80-repository-synthesis-draft.md L356–L415 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)


### CLI-X-002 — Runtime behavior / traces · Later · X

**Canonical verb:** none; rejected concept, not a command.

**Reason:** Static published edges are not runtime traces or dynamic behavior.

**Status:** Rejected from this phase; not a CLI command.

**Sources:** [S80LIMITS: supplements/80-repository-synthesis-draft.md L16–L26 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROADMAP: supplements/80-repository-synthesis-draft.md L356–L415 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)


### CLI-X-003 — Change risk / technical-debt prediction · Later · X

**Canonical verb:** none; rejected concept, not a command.

**Reason:** No calibrated defect probability, safe-change claim or debt-hour score is supported by graph metrics.

**Status:** Rejected from this phase; not a CLI command.

**Sources:** [S80LIMITS: supplements/80-repository-synthesis-draft.md L16–L26 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROADMAP: supplements/80-repository-synthesis-draft.md L356–L415 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)


### CLI-X-004 — Affected-test selection / execution coverage · Later · X

**Canonical verb:** none; rejected concept, not a command.

**Reason:** Test-to-code and execution-coverage facts are absent or unverified; structural test candidates are not coverage.

**Status:** Rejected from this phase; not a CLI command.

**Sources:** [S80LIMITS: supplements/80-repository-synthesis-draft.md L16–L26 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROADMAP: supplements/80-repository-synthesis-draft.md L356–L415 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)


### CLI-X-005 — Reindex or republish workspace · Later · X

**Canonical verb:** none; rejected concept, not a command.

**Reason:** Mutation, source editing, extraction and reindexing are outside read-only navigation.

**Status:** Rejected from this phase; no query verb exists.

**Sources:** [S80LIMITS: supplements/80-repository-synthesis-draft.md L16–L26 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROADMAP: supplements/80-repository-synthesis-draft.md L356–L415 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)


### CLI-X-006 — Watcher / HTTP / MCP lifecycle · Later · X

**Canonical verb:** none; rejected concept, not a command.

**Reason:** Background watcher, HTTP/MCP implementation and service lifecycle are not required or authorized here.

**Status:** Rejected from this phase; no query verb exists.

**Sources:** [S80LIMITS: supplements/80-repository-synthesis-draft.md L16–L26 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROADMAP: supplements/80-repository-synthesis-draft.md L356–L415 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)
