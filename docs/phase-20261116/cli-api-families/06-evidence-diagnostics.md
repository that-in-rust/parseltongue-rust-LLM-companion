# Evidence and diagnostics

Site evidence, unknowns, availability, completeness, truncation and query plans.

**Status:** every API entry is proposed only; none is shipped. X feasibility items are separately listed as rejected concepts with no CLI verb. Priority and feasibility classes are defined in [`CLI-query-backlog.md`](../CLI-query-backlog.md).

<a id="family-evidence"></a>

## Shared evidence profile

Return source-backed row identity, resolution/coverage and evidence IDs; unavailable fields are typed and distinct from empty results. Targetless unresolved sites remain scoped unknowns, not target-attributed callers.

<a id="family-bounds"></a>

## Shared bounds and completeness profile

Report scan/page/semantic completeness separately; truncation and continuation are explicit in the same result envelope; valid partial/empty results differ from malformed/unsupported/operational failures.


<a id="cli-evd-001"></a>

### CLI-EVD-001 — `query-inspect-site-evidence` · P0 · E

**Job:** Fetch the source/evidence envelope for one returned edge or occurrence.

**Inputs:** Evidence/site ID; source-byte option.

**Outputs:** Site identity, range/digest, relation and resolution evidence.

**Evidence:** Return source-backed row identity, resolution/coverage and evidence IDs; unavailable fields are typed and distinct from empty results. Targetless unresolved sites remain scoped unknowns, not target-attributed callers.

**Bounds/completeness:** Report scan/page/semantic completeness separately; truncation and continuation are explicit in the same result envelope; valid partial/empty results differ from malformed/unsupported/operational failures.

**Dependencies:** Published edge-evidence rows and digest-checked source read.

**Sources:** [S80REQ: supplements/80-repository-synthesis-draft.md L34–L43 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [PHASESCOPE L20–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L20-L39); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301)


<a id="cli-evd-002"></a>

### CLI-EVD-002 — `query-list-unresolved-sites` · P1 · E

**Job:** List unresolved, proposed or gap sites by typed reason.

**Inputs:** Snapshot; reason/path/relation filters; cap.

**Outputs:** Site rows with typed reason and source anchors.

**Evidence:** Return source-backed row identity, resolution/coverage and evidence IDs; unavailable fields are typed and distinct from empty results. Targetless unresolved sites remain scoped unknowns, not target-attributed callers.

**Bounds/completeness:** Report scan/page/semantic completeness separately; truncation and continuation are explicit in the same result envelope; valid partial/empty results differ from malformed/unsupported/operational failures.

**Dependencies:** Stored resolution/coverage fields; do not merge states.

**Sources:** [S80REQ: supplements/80-repository-synthesis-draft.md L34–L43 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [PHASESCOPE L20–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L20-L39); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301)


<a id="cli-evd-003"></a>

### CLI-EVD-003 — `query-explain-result-completeness` · P0 · D

**Job:** Explain whether a result scan, page and semantic resolution are complete.

**Inputs:** Self-contained caller-held result/cursor receipt JSON.

**Outputs:** Separate scan/page/semantic/source-scope states, unknown totals, omitted records and a safe self-contained next cursor.

**Evidence:** Return source-backed row identity, resolution/coverage and evidence IDs; unavailable fields are typed and distinct from empty results. Targetless unresolved sites remain scoped unknowns, not target-attributed callers.

**Bounds/completeness:** Report scan/page/semantic completeness separately; truncation and continuation are explicit in the same result envelope; valid partial/empty results differ from malformed/unsupported/operational failures.

**Dependencies:** Query execution metadata and published coverage facts.

**Sources:** [S80REQ: supplements/80-repository-synthesis-draft.md L34–L43 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [PHASESCOPE L20–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L20-L39); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301)


<a id="cli-evd-004"></a>

### CLI-EVD-004 — `query-explain-fact-availability` · P0 · D

**Job:** Explain why one requested field or relation is present, absent, unsupported or unknown.

**Inputs:** Operation/field/item ID.

**Outputs:** Capability status, missing inputs, exclusions and field-level reason.

**Evidence:** Return source-backed row identity, resolution/coverage and evidence IDs; unavailable fields are typed and distinct from empty results. Targetless unresolved sites remain scoped unknowns, not target-attributed callers.

**Bounds/completeness:** Report scan/page/semantic completeness separately; truncation and continuation are explicit in the same result envelope; valid partial/empty results differ from malformed/unsupported/operational failures.

**Dependencies:** Capability registry plus coverage manifest; no producer change.

**Sources:** [S80REQ: supplements/80-repository-synthesis-draft.md L34–L43 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [PHASESCOPE L20–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L20-L39); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301)


<a id="cli-evd-006"></a>

### CLI-EVD-006 — `query-estimate-result-budget` · P2 · D

**Job:** Estimate output bytes or tokens before expansion without claiming measured savings.

**Inputs:** Candidate IDs; byte cap or named tokenizer/version.

**Outputs:** Per-item estimates and selection method; token count labelled exact/estimated.

**Evidence:** Return source-backed row identity, resolution/coverage and evidence IDs; unavailable fields are typed and distinct from empty results. Targetless unresolved sites remain scoped unknowns, not target-attributed callers.

**Bounds/completeness:** Report scan/page/semantic completeness separately; truncation and continuation are explicit in the same result envelope; valid partial/empty results differ from malformed/unsupported/operational failures.

**Dependencies:** Candidate cards and explicit tokenizer; no invented token metric.

**Sources:** [S80REQ: supplements/80-repository-synthesis-draft.md L34–L43 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [PHASESCOPE L20–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L20-L39); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301)


<a id="cli-evd-007"></a>

### CLI-EVD-007 — `query-report-query-plan` · P2 · D

**Job:** Explain normalized filters, relation selection, algorithm and scope used by one query.

**Inputs:** Self-contained caller-held request/result receipt JSON.

**Outputs:** Normalized filters, relation selection, algorithm and scope used; no stored session lookup or SQL exposure.

**Evidence:** Return source-backed row identity, resolution/coverage and evidence IDs; unavailable fields are typed and distinct from empty results. Targetless unresolved sites remain scoped unknowns, not target-attributed callers.

**Bounds/completeness:** Report scan/page/semantic completeness separately; truncation and continuation are explicit in the same result envelope; valid partial/empty results differ from malformed/unsupported/operational failures.

**Dependencies:** Registered query plan and diagnostic envelope.

**Sources:** [S80REQ: supplements/80-repository-synthesis-draft.md L34–L43 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [PHASESCOPE L20–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L20-L39); [FEDGE L261–L301](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L261-L301)
