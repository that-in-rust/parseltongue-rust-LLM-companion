# itoa reference dataset summary

**Status:** LLM_PARSED_DRAFT_NOT_FROZEN; authored by Devin from pinned source; no independent compiler/oracle verification has frozen it. Symbolic identities; never production input.

The complete proposed candidate row set is preserved beside a separate eligible local-only subset and blocker index; it is not whole-source coverage. Known omissions include operators, body types, literals, and repeated same-line sites. The five expected HLD outputs are derived from eligible rows under the current six collapseable kinds (calls, uses_type, implements, reexports, derives, reads); contains/imports are forbidden at every hop. OD-9 remains an alternate proposal, not a revocation. OD-2/OD-7/OD-8 assumptions remain explicit blockers.

The file/folder roll-ups are partial diagnostics with status BLOCKED; 4 expected HLD edges are excluded pending source/target mapping. Builtin targets use BUILTIN_BUCKET_MAPPING_NOT_IMPLEMENTED; exclusions are listed in both roll-up outputs.

OD-9 did not suppress collapse derivation. The eligible subset currently yields 0 collapsed rows; this is an observed candidate result, not a completeness claim.

```text
candidate_nodes: 73
eligible_nodes: 26
blocked_nodes: 47
candidate_edges: 368
eligible_edges: 29
blocked_edges: 339
expected_interface_nodes: 15
expected_dependency_edges: 11
expected_direct_edges: 11
expected_collapsed_edges: 0
blocked_collapsible_candidate_edges: 265
excluded_rollup_edges: 4
source commit: af77385d0daf4d0e949e81f2588be2e44f69f086
source digests and byte sizes: [('src/lib.rs', 16904, 'e962338e1886873aa7c3399ec89aa5378ccbfed712b9690f043628079390a34f'), ('src/u128_ext.rs', 695, '498cc94f13805e551248fd82f47cc318f51d953a5002a9f8698858cd12fbae3c')]
macro invocation paths blocking roll-ups (not item assignments): ['src/lib.rs']
```
