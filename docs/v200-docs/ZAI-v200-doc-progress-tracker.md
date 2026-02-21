# V200 Documentation Analysis Progress Tracker

**Created**: 2026-02-17
**Purpose**: Track document reading progress, key findings, and cross-references for v200-docs analysis

---

## Documents Analyzed

| # | Document | Status | Key Findings |
|---|----------|--------|--------------|
| 1 | ES-V200-Decision-log-01.md | ✅ Complete | Defer rust-llm-context-packer to v210; Promote lifecycle + companion bundle (8 items) |
| 2 | ES-V200-Dependency-Graph-Contract-Hardening.md | ✅ Complete | Method: Dependency Graph Contract Hardening with pass order, probe requirements |
| 3 | ES-V200-Hashing-Risks-v01.md | ✅ Complete | 8-crate architecture, risk matrix, non-negotiable gates (G1-G4), pass ledger |
| 4 | ES-V200-User-Journey-01.md | ✅ Complete | Tauri app journey, 8 journey steps, 21 acceptance criteria |
| 5 | ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md | ✅ Complete | Tauri as visual CLI launcher (not replacement), 3 user modes |
| 6 | PRD-v200.md | ✅ Complete | 2 requirements: No backward compat, No old code deletion |
| 7 | Prep-V200-Competitive-Deep-Dive.md | ✅ Complete | CodeQL/Semgrep/SCIP/SonarQube analysis, unique differentiators |
| 8 | Prep-V200-Compiled-Research-Best-Ideas.md | ✅ Complete | 18 derived rules, priority matrix (H/M/L), 8 leverage items |
| 9 | Prep-V200-Cross-Language-Detection-Heuristics.md | ✅ Complete | 5 detection patterns (FFI/WASM/PyO3/HTTP/MQ), confidence scoring |
| 10 | Prep-V200-Datalog-Ascent-Rule-Patterns.md | ✅ Complete | Ascent crate deep dive, 18 derived rules, base relations |
| 11 | Prep-V200-Key-Format-Design.md | ✅ Complete | ISGL1 problems, 4 candidates, recommendation: D+B hybrid |
| 12 | Prep-V200-LLM-Context-Optimization-Research.md | ✅ Complete | 8 architectural ranking signals, 5 token budgeting algorithms |
| 13 | Prep-V200-MCP-Protocol-Integration.md | ✅ Complete | MCP spec, rmcp SDK, 20+ tools mapping, MCP-first strategy |
| 14 | Prep-V200-Max-Adoption-Architecture-Strategy.md | ✅ Complete | 4 options (A-D), problem-shaped crates, protocol + ecosystem |
| 15 | Prep-V200-Rust-Analyzer-API-Surface.md | ✅ Complete | 20 API methods, version pinning, feature flag strategy |
| 16 | Prep-V200-Tree-Sitter-Query-Patterns.md | ✅ Complete | Query syntax, per-language patterns, 9 implementation phases |
| 17 | Prep-V200-isgl1-ambiguity-risk-table.md | ✅ Complete | 18 open questions across 9 areas, risk reassessment |
| 18 | v200-doc-index-01.md | ✅ Complete | Curated index of 17 primary docs |

---

## Cross-References Between Documents

### Core Architecture References
- **PRD-v200.md** → All other docs (defines clean break)
- **ES-V200-Hashing-Risks-v01.md** ↔ **ES-V200-Dependency-Graph-Contract-Hardening.md** (pass method)
- **ES-V200-Decision-log-01.md** → **ES-V200-Hashing-Risks-v01.md** (scope decisions)

### Key Format Chain
- **Prep-V200-Key-Format-Design.md** → **Prep-V200-isgl1-ambiguity-risk-table.md** (refines analysis)
- **Prep-V200-Key-Format-Design.md** → **ES-V200-Hashing-Risks-v01.md** (EntityKey struct)

### Integration Chain
- **Prep-V200-MCP-Protocol-Integration.md** → **ES-V200-User-Journey-01.md** (MCP integration)
- **Prep-V200-Tree-Sitter-Query-Patterns.md** → **Prep-V200-Cross-Language-Detection-Heuristics.md** (extraction)
- **Prep-V200-Rust-Analyzer-API-Surface.md** → **Prep-V200-Compiled-Research-Best-Ideas.md** (semantic depth)

### User Experience Chain
- **ES-V200-User-Journey-01.md** → **ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md** (reframes Tauri role)
- **Prep-V200-LLM-Context-Optimization-Research.md** → **ES-V200-User-Journey-01.md** (context selection)

---

## Key Contracts Summary

### Data Contracts
1. **EntityKey Format**: `{lang}|||{kind}|||{scope}|||{name}|||{file_path}|||{discriminator}`
2. **CrossLangEdge**: source, target, pattern, confidence [0.0-1.0]
3. **FactSet**: Typed collection of entities, edges, attributes

### API Contracts (22 HTTP + 20+ MCP tools)
- Core: health-check, statistics, help
- Entity: list-all, detail, search-fuzzy
- Graph: edges-list, reverse-callers, forward-callees
- Analysis: blast-radius, cycles, hotspots, clusters
- Advanced: smart-context, SCC, SQALE, k-core, centrality, entropy, coupling, Leiden

### Behavior Contracts
1. **G1 Slim Types**: Entity/storage schema stays canonical, minimal, deterministic
2. **G2 Single Getter**: All read paths go through one storage getter contract
3. **G3 Filesystem Read**: Detail view returns current disk lines with explicit error contract
4. **G4 Path Normalization**: Coverage treats ./path, path, absolute as one file

---

## Gaps and Open Questions

### From Document Analysis
1. **Scope extraction depth** - How much effort for 12 languages?
2. **External entity keys** - Format for std/numpy entities?
3. **rust-analyzer DefId mapping** - Reconciliation with tree-sitter keys?
4. **Cross-language matching heuristics** - Confidence thresholds per pattern?

### Deferred to v210 (from Decision Log)
- rust-llm-context-packer crate
- Token-minimization strategy optimization

---

## Next Steps
1. Synthesize findings into ZAI-PRD-contracts-01.md
2. Ensure all contracts have source document references
3. Validate cross-references between contracts

---

*Generated: 2026-02-17*
