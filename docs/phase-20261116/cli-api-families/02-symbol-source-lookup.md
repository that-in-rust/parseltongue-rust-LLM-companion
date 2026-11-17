# Symbol and source lookup

One search operation with explicit matching modes, position overlap and bounded source reads.

**Status:** every API entry is proposed only; none is shipped. X feasibility items are separately listed as rejected concepts with no CLI verb. Priority and feasibility classes are defined in [`CLI-query-backlog.md`](../CLI-query-backlog.md).

<a id="family-evidence"></a>

## Shared evidence profile

Candidate IDs and paths must come from published node/source facts. Exact/prefix are explicit matching modes; fuzzy lexical ranking selects candidates only and is never source truth.

<a id="family-bounds"></a>

## Shared bounds and completeness profile

Return ambiguity and match reasons; cap candidates/bytes; exact mode never silently falls back to fuzzy; source bytes must match the stored digest. Position overlap is not compiler binding or scope-variable resolution.


### One search verb, three explicit modes

`CLI-SYM-001` is the only symbol search operation: `--match exact|prefix|fuzzy`. Former IDs `CLI-SYM-002` (prefix) and `CLI-SYM-003` (fuzzy rank) are documented aliases to this operation, not separate verbs. Exact mode never falls through to fuzzy matching.


<a id="cli-sym-001"></a>

### CLI-SYM-001 — `query-search-symbol-candidates` · P0 · D

**Job:** Search existing items with explicit exact, prefix or fuzzy matching modes; fuzzy lexical ranking is heuristic selection only.

**Inputs:** Query text; --match exact|prefix|fuzzy; kind/path filters; candidate cap.

**Outputs:** Real candidate IDs, match mode/reasons, deterministic ordering and ambiguity state.

**Evidence:** Candidate IDs and paths must come from published node/source facts. Exact/prefix are explicit matching modes; fuzzy lexical ranking selects candidates only and is never source truth.

**Bounds/completeness:** Return ambiguity and match reasons; cap candidates/bytes; exact mode never silently falls back to fuzzy; source bytes must match the stored digest. Position overlap is not compiler binding or scope-variable resolution.

**Dependencies:** Published node names/qualified paths and IDs; versioned lexical scorer for fuzzy mode.

**Sources:** [S80SEARCH: supplements/80-repository-synthesis-draft.md L157–L164 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7FLOW L9–L50](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L9-L50); [FNODE L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FSOURCE L85–L102](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-table-store-traits/src/lib.rs#L85-L102)


<a id="cli-sym-004"></a>

### CLI-SYM-004 — `query-compare-symbol-candidates` · P1 · D

**Job:** Show why several candidate cards differ so the caller can choose or reject them.

**Inputs:** Candidate IDs from one result page.

**Outputs:** Side-by-side path, kind, owner facts, match reasons and known/unknown fields.

**Evidence:** Candidate IDs and paths must come from published node/source facts. Exact/prefix are explicit matching modes; fuzzy lexical ranking selects candidates only and is never source truth.

**Bounds/completeness:** Return ambiguity and match reasons; cap candidates/bytes; exact mode never silently falls back to fuzzy; source bytes must match the stored digest. Position overlap is not compiler binding or scope-variable resolution.

**Dependencies:** Stable item IDs; capability schema.

**Sources:** [S80SEARCH: supplements/80-repository-synthesis-draft.md L157–L164 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7FLOW L9–L50](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L9-L50); [FNODE L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FSOURCE L85–L102](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-table-store-traits/src/lib.rs#L85-L102)


<a id="cli-sym-005"></a>

### CLI-SYM-005 — `query-preview-symbol-card` · P0 · D

**Job:** Preview one bounded candidate neighborhood before fetching full source.

**Inputs:** Item ID; relation/depth/byte limits.

**Outputs:** Candidate card with match reasons, path, available signature and selected neighbors.

**Evidence:** Candidate IDs and paths must come from published node/source facts. Exact/prefix are explicit matching modes; fuzzy lexical ranking selects candidates only and is never source truth.

**Bounds/completeness:** Return ambiguity and match reasons; cap candidates/bytes; exact mode never silently falls back to fuzzy; source bytes must match the stored digest. Position overlap is not compiler binding or scope-variable resolution.

**Dependencies:** Typed relations, optional signature fields, evidence handles.

**Sources:** [S80SEARCH: supplements/80-repository-synthesis-draft.md L157–L164 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7FLOW L9–L50](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L9-L50); [FNODE L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FSOURCE L85–L102](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-table-store-traits/src/lib.rs#L85-L102)


<a id="cli-sym-006"></a>

### CLI-SYM-006 — `query-outline-file-symbols` · P1 · E

**Job:** List items associated with one exact source path in stable source order.

**Inputs:** Relative source path; nesting/item cap.

**Outputs:** IDs and source ranges grouped by that path; unresolved associations remain visible.

**Evidence:** Candidate IDs and paths must come from published node/source facts. Exact/prefix are explicit matching modes; fuzzy lexical ranking selects candidates only and is never source truth.

**Bounds/completeness:** Return ambiguity and match reasons; cap candidates/bytes; exact mode never silently falls back to fuzzy; source bytes must match the stored digest. Position overlap is not compiler binding or scope-variable resolution.

**Dependencies:** Node source-relative path and source locations.

**Sources:** [S80SEARCH: supplements/80-repository-synthesis-draft.md L157–L164 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7FLOW L9–L50](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L9-L50); [FNODE L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FSOURCE L85–L102](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-table-store-traits/src/lib.rs#L85-L102)


<a id="cli-sym-007"></a>

### CLI-SYM-007 — `query-read-source-window` · P0 · E

**Job:** Read an exact stored source range or bounded context window by digest.

**Inputs:** Item/site ID or path+byte range; context/byte cap; expected digest.

**Outputs:** Exact bytes plus digest verification and historical/current-source label.

**Evidence:** Candidate IDs and paths must come from published node/source facts. Exact/prefix are explicit matching modes; fuzzy lexical ranking selects candidates only and is never source truth.

**Bounds/completeness:** Return ambiguity and match reasons; cap candidates/bytes; exact mode never silently falls back to fuzzy; source bytes must match the stored digest. Position overlap is not compiler binding or scope-variable resolution.

**Dependencies:** Source manifest digest and digest-addressed blob read.

**Sources:** [S80SEARCH: supplements/80-repository-synthesis-draft.md L157–L164 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7FLOW L9–L50](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L9-L50); [FNODE L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FSOURCE L85–L102](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-table-store-traits/src/lib.rs#L85-L102)


<a id="cli-sym-008"></a>

### CLI-SYM-008 — `query-search-source-literals` · P2 · D

**Job:** Find literal text occurrences by scanning available digest-checked source bytes.

**Inputs:** Literal; explicit paths; max files/bytes/time.

**Outputs:** Exact file/byte matches; no binding or semantic claim.

**Evidence:** Candidate IDs and paths must come from published node/source facts. Exact/prefix are explicit matching modes; fuzzy lexical ranking selects candidates only and is never source truth.

**Bounds/completeness:** Return ambiguity and match reasons; cap candidates/bytes; exact mode never silently falls back to fuzzy; source bytes must match the stored digest. Position overlap is not compiler binding or scope-variable resolution.

**Dependencies:** Source blob reads; bounded scan; no new index/parser.

**Sources:** [S80SEARCH: supplements/80-repository-synthesis-draft.md L157–L164 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7FLOW L9–L50](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L9-L50); [FNODE L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FSOURCE L85–L102](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-table-store-traits/src/lib.rs#L85-L102)


<a id="cli-sym-009"></a>

### CLI-SYM-009 — `query-find-source-overlaps` · P1 · E

**Job:** Find stored item/site spans overlapping an exact source position or byte range without resolving compiler bindings.

**Inputs:** Relative path; half-open byte range or byte position; source digest; row cap.

**Outputs:** Overlapping stored item/site IDs, span ranges and exact overlap evidence; no scope-variable binding claim.

**Evidence:** Candidate IDs and paths must come from published node/source facts. Exact/prefix are explicit matching modes; fuzzy lexical ranking selects candidates only and is never source truth.

**Bounds/completeness:** Return ambiguity and match reasons; cap candidates/bytes; exact mode never silently falls back to fuzzy; source bytes must match the stored digest. Position overlap is not compiler binding or scope-variable resolution.

**Dependencies:** Stored node/site source ranges and digest identity.

**Sources:** [S80SEARCH: supplements/80-repository-synthesis-draft.md L157–L164 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7FLOW L9–L50](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L9-L50); [FNODE L207–L241](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs#L207-L241); [FSOURCE L85–L102](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/9fc6e7a16e4f8776a1d03ab668d3d219bf943c7b/crates/parseltongue-table-store-traits/src/lib.rs#L85-L102)
