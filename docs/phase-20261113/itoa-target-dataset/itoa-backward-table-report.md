# itoa backward-table verification

Status: **BLOCKED**

Export integrity: BLOCKED
Frozen stage gate: BLOCKED (exact frozen-revision proof is not implemented; summary markers are not proof)
Stage summary scope: NOT_SUPPLIED
Source evidence: PRESENT; generation link: UNVERIFIED_GENERATION_LINK
Semantic target: BLOCKED_NOT_CHECKED (NOT_IMPLEMENTED_CANONICAL_TARGET_MAPPING)

No canonical mapping/full HLD comparison is implemented; overall verification stays BLOCKED.

## Backward ladder

- `hld-current-interface-nodes` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `hld-current-dependency-edges` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `hld-edge-evidence-paths` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `hld-file-level-rollup` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `hld-folder-level-rollup` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `itoa-full-graph-nodes.json` — export `LOCAL_REFERENCE_AVAILABLE`, marker `NOT_APPLICABLE_REFERENCE`, comparison `BLOCKED`; source path, byte range and draft classification are retained
- `itoa-full-graph-edges.json` — export `LOCAL_REFERENCE_AVAILABLE`, marker `NOT_APPLICABLE_REFERENCE`, comparison `BLOCKED`; relation, target classification and source evidence are retained
- `HldMaterializationInputRecord` — export `NOT_EXPORTED`, marker `NOT_APPLICABLE_TYPED_INTERFACE`, comparison `BLOCKED`; adapter accepts this typed record; no named upstream table is read directly
- `item-effective-visibility-records` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `edge-evidence-record-table` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `lowered-signature-type-table` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `impl-header-index-table` — export `EXPORTED_EMPTY`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `module-scope-binding-table` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `name-resolution-scope-records` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `path-resolution-site-records` — export `EXPORTED_ROWS`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `analysis-resolution-receipt-records` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `macro-expanded-item-table` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `lang-item-definition-table` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `syntax-namespace-input-scopes` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `syntax-namespace-input-items` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `syntax-namespace-input-imports` — export `EXPORTED_ROWS`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `syntax-namespace-input-paths` — export `EXPORTED_ROWS`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `syntax-namespace-input-macros` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `syntax-namespace-input-externs` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `legacy-macro-scope-records` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `legacy-macro-source-events` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `file-source-text-table` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `span-reference-text-table` — export `EXPORTED_ROWS`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `span-byte-ownership-ranges` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `field-signature-detail-table` — export `EXPORTED_ROWS`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `body-site-reference-table` — export `EXPORTED_ROWS`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `module-source-closure-requests` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `syntax-file-work-requests` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `file-source-content-records` — export `NOT_EXPORTED`, marker `NOT_APPLICABLE_CONTENT_STORE`, comparison `BLOCKED`; content bytes are stored by file_content_digest_full; digest verification is not checked here
- `crate-dependency-graph-table` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `workspace-config-state-records` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `oracle-edge-agreement-table` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `stage-run-generation-records` — export `NOT_EXPORTED`, marker `MISSING`, comparison `BLOCKED`; exact table name and generation/attempt envelope; payload comparison blocked without canonical mapping
- `workspace-generation-publication-manifest` — export `NOT_EXPORTED`, marker `CONTROL_EVIDENCE_NOT_VERIFIED`, comparison `BLOCKED`; workspace generation scope is unique
- `analysis-input-publication-manifest` — export `NOT_EXPORTED`, marker `CONTROL_EVIDENCE_NOT_VERIFIED`, comparison `BLOCKED`; analysis input generation scope is unique
- `frozen-stage-publication-records` — export `NOT_EXPORTED`, marker `CONTROL_EVIDENCE_NOT_VERIFIED`, comparison `BLOCKED`; revision identity must match frozen stage proof
- `frozen-stage-unit-memberships` — export `NOT_EXPORTED`, marker `CONTROL_EVIDENCE_NOT_VERIFIED`, comparison `BLOCKED`; unit membership belongs to the exact frozen revision
