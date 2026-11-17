# Progressive context and conversation

Budget planning, candidate cards, composition, caller-owned unseen context and portable receipts.

**Status:** every API entry is proposed only; none is shipped. X feasibility items are separately listed as rejected concepts with no CLI verb. Priority and feasibility classes are defined in [`CLI-query-backlog.md`](../CLI-query-backlog.md).

<a id="family-evidence"></a>

## Shared evidence profile

Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

<a id="family-bounds"></a>

## Shared bounds and completeness profile

Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.


### Caller-held continuation

Pages and receipts are self-contained JSON passed back by the caller. A cursor is an option on a context operation, not a separate API, and no server-side session/result store is assumed.


<a id="cli-ctx-001"></a>

### CLI-CTX-001 — `query-plan-context-budget` · P1 · D

**Job:** Plan a reason-labelled candidate set under an explicit output budget.

**Inputs:** Focus IDs; intent; relation filters; bytes or named tokenizer budget.

**Outputs:** Ordered candidates, estimated/actual method and omitted counts.

**Evidence:** Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

**Bounds/completeness:** Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.

**Dependencies:** Candidate cards and budget model; ranking stays heuristic.

**Sources:** [S80CTX: supplements/80-repository-synthesis-draft.md L180–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7BUDGET L66–L82](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L66-L82); [S80WORK: supplements/80-repository-synthesis-draft.md L273–L305 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-ctx-002"></a>

### CLI-CTX-002 — `query-preview-context-candidates` · P1 · D

**Job:** Show compact candidate cards before source bodies are fetched.

**Inputs:** Focus IDs; candidate limit; preview-byte cap.

**Outputs:** Stable IDs, inclusion reasons, paths, evidence availability and next operations.

**Evidence:** Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

**Bounds/completeness:** Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.

**Dependencies:** Search, relation and coverage operations.

**Sources:** [S80CTX: supplements/80-repository-synthesis-draft.md L180–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7BUDGET L66–L82](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L66-L82); [S80WORK: supplements/80-repository-synthesis-draft.md L273–L305 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-ctx-003"></a>

### CLI-CTX-003 — `query-build-evidence-context` · P1 · D

**Job:** Compose source/signature/relationship/evidence items into one bounded bundle.

**Inputs:** Self-contained caller-held request/receipt JSON; focus IDs; intent/body policy; relation and byte/token budgets.

**Outputs:** Deduplicated evidence-backed items, reasons, omissions and a self-contained continuation receipt; no server session.

**Evidence:** Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

**Bounds/completeness:** Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.

**Dependencies:** Read-only composition of other operations; no generated facts.

**Sources:** [S80CTX: supplements/80-repository-synthesis-draft.md L180–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7BUDGET L66–L82](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L66-L82); [S80WORK: supplements/80-repository-synthesis-draft.md L273–L305 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-ctx-004"></a>

### CLI-CTX-004 — `query-expand-context-candidate` · P1 · D

**Job:** Expand one selected card to a deeper bounded neighborhood.

**Inputs:** Self-contained candidate-card receipt JSON; depth and byte/token budget; relation allowlist.

**Outputs:** Expanded graph/source items with per-item inclusion reason.

**Evidence:** Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

**Bounds/completeness:** Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.

**Dependencies:** Preview card data and snapshot-bound IDs carried in the caller-held receipt.

**Sources:** [S80CTX: supplements/80-repository-synthesis-draft.md L180–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7BUDGET L66–L82](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L66-L82); [S80WORK: supplements/80-repository-synthesis-draft.md L273–L305 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-ctx-005"></a>

### CLI-CTX-005 — `query-compose-query-sequence` · P2 · D

**Job:** Run a typed bounded sequence of existing operations with explicit data handoff.

**Inputs:** Versioned query recipe; IDs/filters; per-step and total bounds.

**Outputs:** Step results, dependencies, errors and final evidence envelope.

**Evidence:** Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

**Bounds/completeness:** Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.

**Dependencies:** Only registered operations; no unrestricted SQL or hidden mutation.

**Sources:** [S80CTX: supplements/80-repository-synthesis-draft.md L180–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7BUDGET L66–L82](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L66-L82); [S80WORK: supplements/80-repository-synthesis-draft.md L273–L305 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-ctx-006"></a>

### CLI-CTX-006 — `query-build-context-delta` · P2 · D

**Job:** Return unseen items relative to caller-supplied prior receipt IDs in the same snapshot.

**Inputs:** Caller-held prior receipt JSON with seen IDs, generation/query/filter/order identity; same snapshot required for an unseen delta.

**Outputs:** Newly selected items, already-seen counts and completeness; stale/mismatched receipt is typed, not silently reset.

**Evidence:** Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

**Bounds/completeness:** Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.

**Dependencies:** Caller supplies receipts; CLI stores no persistent learned memory.

**Sources:** [S80CTX: supplements/80-repository-synthesis-draft.md L180–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7BUDGET L66–L82](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L66-L82); [S80WORK: supplements/80-repository-synthesis-draft.md L273–L305 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-ctx-008"></a>

### CLI-CTX-008 — `query-export-portable-context` · P2 · D

**Job:** Emit a portable receipt describing selected IDs, query identity and source digests.

**Inputs:** Selected result IDs; snapshot; include-evidence option.

**Outputs:** JSON receipt plus schema/version and continuation metadata.

**Evidence:** Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

**Bounds/completeness:** Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.

**Dependencies:** Canonical JSON and source/evidence IDs; no copied LLM reasoning.

**Sources:** [S80CTX: supplements/80-repository-synthesis-draft.md L180–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7BUDGET L66–L82](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L66-L82); [S80WORK: supplements/80-repository-synthesis-draft.md L273–L305 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)


<a id="cli-ctx-009"></a>

### CLI-CTX-009 — `query-compare-context-receipts` · P2 · D

**Job:** Compare two caller-held receipts to show included, omitted and changed selections.

**Inputs:** Two caller-held context receipt JSON objects; explicit compatibility/diff policy.

**Outputs:** Added/removed/changed selections and reason deltas; left/right snapshot identities may differ and are reported.

**Evidence:** Every selected item carries a reason and source/evidence handle; caller focus/weights/seen receipts affect selection, not stored facts. Receipts and cursors are self-contained caller-held JSON; no server-side result/session state.

**Bounds/completeness:** Bind continuation to generation, query, filters, ordering and algorithm version; enforce whole-record limits; tokens require a named tokenizer or labelled estimate.

**Dependencies:** Receipt schema; snapshot compatibility.

**Sources:** [S80CTX: supplements/80-repository-synthesis-draft.md L180–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80RESPONSE: supplements/80-repository-synthesis-draft.md L254–L263 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [R7BUDGET L66–L82](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L66-L82); [S80WORK: supplements/80-repository-synthesis-draft.md L273–L305 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz)
