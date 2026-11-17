# Compatible snapshot comparison

One conditional diff operation with section modes; left/right identities may differ.

**Status:** every API entry is proposed only; none is shipped. X feasibility items are separately listed as rejected concepts with no CLI verb. Priority and feasibility classes are defined in [`CLI-query-backlog.md`](../CLI-query-backlog.md).

<a id="family-evidence"></a>

## Shared evidence profile

Each left/right input must be internally consistent: all tables for that side share its own generation digest and attempt. Left and right are expected to have different identities; compare their parser/config/unit/source dimensions explicitly.

<a id="family-bounds"></a>

## Shared bounds and completeness profile

Never mix generations within either side. Do not require left and right generation/attempt equality; report parser/config differences as compatibility dimensions or typed incompatibility under the chosen policy, and preserve identity ambiguity.


### Left/right identity rule

Left and right snapshots normally have different generation/attempt IDs. Require consistency among all tables within each side; do not require equality between sides. Report parser, compiler/configuration, unit and source-digest differences as explicit comparison dimensions or typed incompatibilities under the requested policy.


<a id="cli-snp-001"></a>

### CLI-SNP-001 — `query-compare-published-snapshots` · P2 · C

**Job:** Compare selected symbols, relations or source hashes across two caller-supplied frozen snapshots; the snapshots may differ in generation and attempt.

**Inputs:** Left and right frozen stores/manifests; section=symbols|relations|source-hashes; identity policy; optional compatibility dimensions.

**Outputs:** Section diff, per-side identities, parser/config/unit/source differences, and ambiguous identity matches.

**Evidence:** Each left/right input must be internally consistent: all tables for that side share its own generation digest and attempt. Left and right are expected to have different identities; compare their parser/config/unit/source dimensions explicitly.

**Bounds/completeness:** Never mix generations within either side. Do not require left and right generation/attempt equality; report parser/config differences as compatibility dimensions or typed incompatibility under the chosen policy, and preserve identity ambiguity.

**Dependencies:** Each side is internally consistent; left/right generation and attempt may differ; two-snapshot read path remains conditional.

**Sources:** [S80SNAP: supplements/80-repository-synthesis-draft.md L181–L182 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FGEN L18–L29](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/stage-pipeline-row-types.rs#L18-L29); [FHEADER L11–L23](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L11-L23)


<a id="cli-snp-004"></a>

### CLI-SNP-004 — `query-map-snapshot-identities` · P2 · C

**Job:** Map candidate item identities between snapshots without forcing ambiguous name matches.

**Inputs:** Two frozen snapshots with independently consistent headers; strict identity keys and explicit ambiguity policy.

**Outputs:** One-to-one, split/merge ambiguous and unmatched identity states across the two generations.

**Evidence:** Each left/right input must be internally consistent: all tables for that side share its own generation digest and attempt. Left and right are expected to have different identities; compare their parser/config/unit/source dimensions explicitly.

**Bounds/completeness:** Never mix generations within either side. Do not require left and right generation/attempt equality; report parser/config differences as compatibility dimensions or typed incompatibility under the chosen policy, and preserve identity ambiguity.

**Dependencies:** Stable identity fields plus explicit ambiguity policy.

**Sources:** [S80SNAP: supplements/80-repository-synthesis-draft.md L181–L182 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FGEN L18–L29](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/stage-pipeline-row-types.rs#L18-L29); [FHEADER L11–L23](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L11-L23)


<a id="cli-snp-005"></a>

### CLI-SNP-005 — `query-report-snapshot-compatibility` · P1 · C

**Job:** Validate each side internally and report cross-snapshot parser/config/unit/source differences before diffing.

**Inputs:** Left and right snapshot headers/manifests; selected compatibility policy.

**Outputs:** Per-side consistency plus explicit compatibility dimensions; different generation/attempt IDs alone are not a mismatch.

**Evidence:** Each left/right input must be internally consistent: all tables for that side share its own generation digest and attempt. Left and right are expected to have different identities; compare their parser/config/unit/source dimensions explicitly.

**Bounds/completeness:** Never mix generations within either side. Do not require left and right generation/attempt equality; report parser/config differences as compatibility dimensions or typed incompatibility under the chosen policy, and preserve identity ambiguity.

**Dependencies:** Published header and unit manifest; multi-generation read path absent.

**Sources:** [S80SNAP: supplements/80-repository-synthesis-draft.md L181–L182 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [FGEN L18–L29](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/stage-pipeline-row-types.rs#L18-L29); [FHEADER L11–L23](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs#L11-L23)
