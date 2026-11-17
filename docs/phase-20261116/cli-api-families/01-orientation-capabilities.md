# Orientation and capabilities

Read-only publication identity, capability discovery and source-coverage summaries.

**Status:** every API entry is proposed only; none is shipped. X feasibility items are separately listed as rejected concepts with no CLI verb. Priority and feasibility classes are defined in [`CLI-query-backlog.md`](../CLI-query-backlog.md).

<a id="family-evidence"></a>

## Shared evidence profile

Published generation header, manifest and typed coverage facts only; schema/readback existence is not evidence that a row is present or accepted.

<a id="family-bounds"></a>

## Shared bounds and completeness profile

Bind each answer to one generation + attempt; unavailable, no publication, empty table and unknown counts remain distinct.


<a id="cli-ori-001"></a>

### CLI-ORI-001 — `query-report-published-scope` · P0 · E

**Job:** Show whether one published generation can be read and which scope header it names.

**Inputs:** Store path; read-only open.

**Outputs:** Publication state, generation digest, attempt sequence and allowed table names; never producer acceptance.

**Evidence:** Published generation header, manifest and typed coverage facts only; schema/readback existence is not evidence that a row is present or accepted.

**Bounds/completeness:** Bind each answer to one generation + attempt; unavailable, no publication, empty table and unknown counts remain distinct.

**Dependencies:** Current published-scope readback; CLI exposes export, not a query registry.

**Sources:** [S80ORI: supplements/80-repository-synthesis-draft.md L153–L156 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [COVERAGE L1–L48](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/markdown-review-coverage.md#L1-L48); [FHEADER L11–L23](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L11-L23); [FEXPORT L66–L101](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L66-L101); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)


<a id="cli-ori-002"></a>

### CLI-ORI-002 — `query-discover-query-capabilities` · P0 · D

**Job:** Let an agent discover exact operations, versioned fields and unsupported capabilities before querying.

**Inputs:** Optional operation or requested field.

**Outputs:** Capability schema with supported, conditional and unavailable states.

**Evidence:** Published generation header, manifest and typed coverage facts only; schema/readback existence is not evidence that a row is present or accepted.

**Bounds/completeness:** Bind each answer to one generation + attempt; unavailable, no publication, empty table and unknown counts remain distinct.

**Dependencies:** Future query registry; no new stored fact.

**Sources:** [S80ORI: supplements/80-repository-synthesis-draft.md L153–L156 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [COVERAGE L1–L48](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/markdown-review-coverage.md#L1-L48); [FHEADER L11–L23](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L11-L23); [FEXPORT L66–L101](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L66-L101); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)


<a id="cli-ori-003"></a>

### CLI-ORI-003 — `query-inspect-generation-identity` · P0 · E

**Job:** Return the identity that must bind all follow-up reads.

**Inputs:** Store path; optional expected generation.

**Outputs:** Generation digest, attempt sequence and unit-manifest identity.

**Evidence:** Published generation header, manifest and typed coverage facts only; schema/readback existence is not evidence that a row is present or accepted.

**Bounds/completeness:** Bind each answer to one generation + attempt; unavailable, no publication, empty table and unknown counts remain distinct.

**Dependencies:** Publication header and mapped-unit manifest.

**Sources:** [S80ORI: supplements/80-repository-synthesis-draft.md L153–L156 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [COVERAGE L1–L48](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/markdown-review-coverage.md#L1-L48); [FHEADER L11–L23](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L11-L23); [FEXPORT L66–L101](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L66-L101); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)


<a id="cli-ori-004"></a>

### CLI-ORI-004 — `query-summarize-source-coverage` · P1 · D

**Job:** Summarize only recorded source-coverage and mapping-scope facts.

**Inputs:** Snapshot; path/build-unit filter; row cap.

**Outputs:** Counts and unknowns by typed coverage state with drilldown handles.

**Evidence:** Published generation header, manifest and typed coverage facts only; schema/readback existence is not evidence that a row is present or accepted.

**Bounds/completeness:** Bind each answer to one generation + attempt; unavailable, no publication, empty table and unknown counts remain distinct.

**Dependencies:** Published node/edge coverage fields; unit manifest.

**Sources:** [S80ORI: supplements/80-repository-synthesis-draft.md L153–L156 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [COVERAGE L1–L48](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/markdown-review-coverage.md#L1-L48); [FHEADER L11–L23](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L11-L23); [FEXPORT L66–L101](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L66-L101); [PHASE L14–L39](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/docs/phase-20261116/ReadMe.md#L14-L39)
