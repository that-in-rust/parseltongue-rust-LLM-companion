# Derived structure and analytics

Path rollups with leaf drilldown and explicitly parameterized graph measures.

**Status:** every API entry is proposed only; none is shipped. X feasibility items are separately listed as rejected concepts with no CLI verb. Priority and feasibility classes are defined in [`CLI-query-backlog.md`](../CLI-query-backlog.md).

<a id="family-evidence"></a>

## Shared evidence profile

Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

<a id="family-bounds"></a>

## Shared bounds and completeness profile

Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.


<a id="cli-ana-001"></a>

### CLI-ANA-001 — `query-summarize-file-graph` · P1 · D

**Job:** Summarize recorded items and relation-site counts grouped by exact source-relative file path.

**Inputs:** Snapshot; path scope; relation allowlist.

**Outputs:** Per-file descriptive counts with leaf IDs and unknown coverage.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Node paths and edge sites; same-generation rows.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-002"></a>

### CLI-ANA-002 — `query-summarize-folder-graph` · P1 · D

**Job:** Roll up file-path groups by explicit relative-path prefixes.

**Inputs:** Snapshot; folder depth; relation filters; cap.

**Outputs:** Folder counts plus provenance to contributing file leaves.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Derived path-prefix rollup only; not a module-to-file relation.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-003"></a>

### CLI-ANA-003 — `query-drilldown-rollup-leaves` · P1 · D

**Job:** Drill a file/folder aggregate back to the exact contributing items and sites.

**Inputs:** Rollup ID/path; leaf/result cap.

**Outputs:** Contributor list, counts reconciled to the aggregate, truncation state.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** File/folder rollup and same-generation source paths.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-004"></a>

### CLI-ANA-004 — `query-count-relation-motifs` · P2 · D

**Job:** Count explicitly defined typed edge/path motifs in a selected subgraph.

**Inputs:** Motif definition/version; scope; relation allowlist.

**Outputs:** Exact counts and witness sample sites per motif.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Published edges; motif formula supplied and versioned.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-005"></a>

### CLI-ANA-005 — `query-measure-degree-distribution` · P2 · D

**Job:** Report directed in/out degree distributions by relation filter.

**Inputs:** Snapshot/scope; direction; relation list; bins or exact counts.

**Outputs:** Formula-labelled counts, isolates only within declared scope.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Derived from published edges; row and scope completeness.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-006"></a>

### CLI-ANA-006 — `query-find-strong-components` · P2 · D

**Job:** Compute strongly connected components over an explicitly directed relation set.

**Inputs:** Snapshot/scope; relations; node cap.

**Outputs:** Component IDs, members, algorithm version and seed if relevant.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Derived graph algorithm; not source-authored architecture.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-007"></a>

### CLI-ANA-007 — `query-list-cyclic-components` · P2 · D

**Job:** List SCCs or self-loops that satisfy an explicit cycle definition.

**Inputs:** Component/result cap; relation filters.

**Outputs:** Witness edges/sites and complete/partial scan status.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Strong-components operation; no runtime cycle claim.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-008"></a>

### CLI-ANA-008 — `query-rank-graph-centrality` · P2 · D

**Job:** Rank graph nodes by one named deterministic centrality algorithm.

**Inputs:** Algorithm/version; direction/relations; seed; scope/top-N.

**Outputs:** Scores, parameters, tie-breaks and source IDs.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Structural prioritization only; not “importance,” debt or break probability.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-009"></a>

### CLI-ANA-009 — `query-rank-graph-communities` · P2 · D

**Job:** Partition a selected graph with a named/versioned seeded algorithm.

**Inputs:** Algorithm/version/seed; relation set; scope.

**Outputs:** Partition membership and algorithm metadata, labelled heuristic.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Community labels describe the projection only, not semantic modules.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-010"></a>

### CLI-ANA-010 — `query-measure-kcore-decomposition` · P2 · D

**Job:** Compute k-core numbers for a declared graph projection.

**Inputs:** Direction/undirected projection; relation set; scope.

**Outputs:** Per-item core number and formula/version.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Descriptive graph layering only; not architecture tiers.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-011"></a>

### CLI-ANA-011 — `query-measure-graph-entropy` · P2 · D

**Job:** Compute a named entropy formula over an explicit graph distribution.

**Inputs:** Distribution (e.g. degree); binning/formula version; scope.

**Outputs:** Input counts, formula, units and result; no complexity label.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Derived statistic; no inference about code complexity or quality.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-012"></a>

### CLI-ANA-012 — `query-measure-subgraph-coupling` · P2 · D

**Job:** Count internal/cross-boundary relation sites for a declared partition.

**Inputs:** Partition/path boundary; relation allowlist; denominator policy.

**Outputs:** Numerators/denominators and witness sites; no borrowed CK metric names.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** Derived counts; caller-supplied partition or path prefixes.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)


<a id="cli-ana-013"></a>

### CLI-ANA-013 — `query-evaluate-supplied-rules` · P2 · D

**Job:** Evaluate caller-supplied predicates against available graph facts and return witnesses.

**Inputs:** Versioned rule JSON; subject IDs; rule limits.

**Outputs:** Satisfied, contradicted or unknown per rule with source evidence.

**Evidence:** Emit formula, algorithm/version, direction, relation allowlist, parameters and seed; metrics are descriptive derived views, not architecture truth, debt, safety or defect probability.

**Bounds/completeness:** Recompute only within an explicit compatible scope; deterministic tie-breaks; unknown inputs are not zero; return formula metadata and bounded rows with leaf drilldown.

**Dependencies:** No persisted rule database; missing facts yield unknown; no unsupported architecture assertion.

**Sources:** [S80METRICS: supplements/80-repository-synthesis-draft.md L182–L186 (supplied bundle)](https://app.devin.ai/attachments/93cae10f-5664-448d-aecd-b1febfcba3fb/review-bundle.tar.gz); [S80ROLL L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RAGG L7–L68](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793853341-rust-interface-graph-for-10-million-loc-systems-programming-and-aggregation-level.md#L7-L68); [RREADME L228–L305](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/README.md#L228-L305); [R7TRUTH L100–L108](https://github.com/that-in-rust/parseltongue-rust-LLM-companion/blob/fac71702f179c88990914ccd098e0bd52e7e7131/docs/devin-research/1793879701-parseltongue-as-grep-replacement-cpu-only-seven-event-retrieval-protocol-over-the-rust-graph.md#L100-L108)
