# CR-v173-03: Competitor Feature Deep-Dive -- Code-Level Analysis

**Generated**: 2026-02-14
**Method**: Pass 1 of 3 -- cclsp LSP code reading + direct source analysis
**Pass 2**: Parseltongue graph validation (COMPLETE -- 7779 port, 19,431 entities, 144,137 edges)
**Pass 3**: Implementation specs + Shreyas Doshi product opinion (COMPLETE -- 2026-02-14T18:30:00Z, 16 specs + LNO priority matrix)

---

## 1. code-scalpel

### Feature 1.1: MCP Server with FastMCP + Tiered Licensing

**ELI5**: Code Scalpel runs as an MCP server that any AI tool (Claude Desktop, Cursor, etc.) can connect to. It has a 3-tier licensing system (Community/Pro/Enterprise) where higher tiers unlock more powerful security and analysis tools. The server uses JWT tokens to validate licenses, and environment variables can only downgrade your tier, never upgrade it -- preventing piracy.

**Key Code Path**:
```
src/code_scalpel/mcp/protocol.py:147 -> FastMCP("Code Scalpel") instantiation
  -> protocol.py:9 imports from mcp.server.fastmcp
  -> protocol.py:93 _get_current_tier() -> JWTLicenseValidator().validate()
  -> server.py:209 _get_current_tier() -> rank comparison -> min(requested, licensed)
  -> tools/analyze.py:143 @mcp.tool() decorator registers analyze_code
  -> tools/security.py:25 @mcp.tool() registers unified_sink_detect
  -> tools/policy.py:21 @mcp.tool() registers validate_paths
  -> tools/symbolic.py:36 @mcp.tool() registers symbolic_execute
  -> tools/extraction.py:33 @mcp.tool() registers extract_code
```

**What the code actually does**: The `mcp` singleton is created in `protocol.py:147` as `FastMCP("Code Scalpel")` with a massive instruction string describing all available tools. Tool modules in `mcp/tools/*.py` import this singleton and decorate async functions with `@mcp.tool()`. Each tool function calls `_get_current_tier()` which validates a JWT license file, determines the effective tier (Community/Pro/Enterprise), and passes tier-specific capabilities from `licensing/features.py` to the sync helper functions via `asyncio.to_thread()`. Every tool response is wrapped in a `ToolResponseEnvelope` from `mcp/contract.py` that includes metadata like tier, duration, and tool version. The server supports both stdio (default) and streamable-http transports.

**Parseltongue comparison**:
- **File entity**: `python:file:__code-scalpel_src_code_scalpel_mcp_server_py:1-1`
- **CBO (Coupling Between Objects)**: 483 (highest in entire corpus -- grade F, 14h tech debt)
- **Forward callees**: 483 (calls AST node types, Pydantic models, CallGraphBuilder, CrawlSummary, ContextualExtractionResult, etc.)
- **Reverse callers**: 0 inbound (pure leaf consumer -- imports everything, nothing imports it)
- **FastMCP reference**: `python:fn:FastMCP:unresolved-reference:0-0` called from 6 locations (greptimedb-mcp tests only)
- **Parseltongue has equivalent?**: Partial -- Parseltongue exposes 22 HTTP endpoints (not MCP). No tiered licensing, no JWT validation, no FastMCP. The HTTP server architecture is fundamentally different: Parseltongue pre-computes graph data in CozoDB vs. code-scalpel's on-demand parsing. Parseltongue's `/api-reference-documentation-help` endpoint serves a similar discovery role to code-scalpel's tool registry.
**Implementation spec**:
**Feature name**: `pt09-mcp-protocol-bridge-server` (new crate)
**Where**: New crate `crates/pt09-mcp-protocol-bridge-server/` + thin adapter layer over existing pt08 handlers
**CozoDB tables needed**: None new -- reuses existing `CodeGraph` and `DependencyEdges` relations
**Key dependencies**: `rmcp` (Rust MCP SDK), `tokio`, `serde_json`, `parseltongue-core`
**Architecture**: Bridge pattern -- pt09 wraps pt08's existing query logic (extracted to shared functions in `parseltongue-core`) and exposes them as MCP tools via stdio and streamable-HTTP transports. Each of the 24 existing HTTP endpoints becomes an MCP tool with the same parameters. No tiered licensing in v1 -- Parseltongue is open-source, so JWT/licensing is overhead. If licensing is needed later, add a `license-validation-gate-middleware` module.
**New MCP tools** (mapped 1:1 from HTTP endpoints):
  - `codebase_statistics_overview_summary` -> existing handler logic
  - `code_entities_search_fuzzy` -> existing handler logic
  - `blast_radius_impact_analysis` -> existing handler logic
  - (+ 21 more, one per endpoint)
**Transport**: stdio (primary, for Claude Desktop/Cursor) + streamable-HTTP (secondary, for web clients)
**Estimated LoC**: ~1,200 Rust (MCP server boilerplate + tool registration + transport setup)
**Effort**: Medium (2-3 weeks) -- most logic already exists in pt08 handlers; main work is MCP protocol plumbing
**Risk**: Low -- `rmcp` crate is maturing rapidly; stdio transport is well-tested in the MCP ecosystem. The four-word naming constraint means all tool names need careful design.

---

### Feature 1.2: Taint Analysis with Z3 Symbolic Strings

**ELI5**: Code Scalpel can track "dirty data" (like user input from a web form) as it flows through your code. If that dirty data reaches a dangerous operation (like a SQL query) without being cleaned first, the tool flags it as a security vulnerability with a specific CWE identifier. It uses Microsoft's Z3 theorem prover to represent data symbolically, so it can reason about code paths without actually running the code.

**Key Code Path**:
```
security/analyzers/taint_tracker.py:63 -> TaintSource enum (USER_INPUT, FILE_CONTENT, NETWORK_DATA, etc.)
  -> taint_tracker.py:84 -> SecuritySink enum (SQL_QUERY, HTML_OUTPUT, SHELL_COMMAND + 20 more)
  -> taint_tracker.py:142 -> TaintLevel enum (HIGH, MEDIUM, LOW, NONE)
  -> taint_tracker.py:158 -> TaintInfo dataclass (source, level, propagation_path, sanitizers_applied, cleared_sinks)
  -> taint_tracker.py:181 -> TaintInfo.propagate(through_var) creates new TaintInfo with extended path
  -> taint_tracker.py:201 -> TaintInfo.apply_sanitizer() checks SANITIZER_REGISTRY, clears specific sinks
  -> taint_tracker.py:248 -> TaintInfo.is_dangerous_for(sink) checks cleared_sinks + SINK_SANITIZERS
  -> taint_tracker.py:326 -> SANITIZER_REGISTRY dict (80+ sanitizers for Python, JS, Node.js)
  -> taint_tracker.py:673 -> TaintTracker class
  -> taint_tracker.py:704 -> taint_source(name, source) creates z3.String(name) + TaintInfo
  -> taint_tracker.py:769 -> propagate_assignment(target, source_names) merges taint from multiple sources
  -> taint_tracker.py:45 -> VulnerabilityDict TypedDict (type, cwe, sink, source, taint_path, severity, recommendation)
```

**What the code actually does**: `TaintTracker` maintains a `_taint_map: Dict[str, TaintInfo]` shadow state alongside the symbolic execution state. When user input is detected (e.g., `request.args`), `taint_source()` creates a Z3 `String` symbolic variable and a `TaintInfo` with `TaintLevel.HIGH`. As data flows through assignments and operations, `propagate_assignment()` merges taint from all source variables. When a sanitizer like `html.escape()` is detected, `apply_sanitizer()` looks it up in `SANITIZER_REGISTRY` (80+ entries covering Python, JavaScript, and Node.js ecosystems) and marks specific sinks as safe in `cleared_sinks`. The `is_dangerous_for(sink)` method checks if tainted data reaching a specific sink (e.g., `SQL_QUERY`) is still dangerous after sanitization. The tracker supports 25+ sink types including SQL injection (CWE-89), XSS (CWE-79), command injection (CWE-78), SSRF (CWE-918), prototype pollution (CWE-1321), and GraphQL injection.

**Parseltongue comparison**:
- **File entity**: `python:file:__code-scalpel_src_code_scalpel_security_analyzers_taint_tracker_py:1-1`
- **CBO**: 137 (grade F, 14h tech debt) -- calls 137 unique entities including AST node types (Call, Name, Attribute), 24 SecuritySink enums (SQL_QUERY, DOM_XSS, DESERIALIZATION, EL_INJECTION, EMAIL_INJECTION, etc.), and sanitizer infrastructure
- **Class entities indexed**: 9 classes (`TaintTracker`, `TaintSource`, `TaintInfo`, `TaintLevel`, `SecuritySink`, `SanitizerInfo`, `TaintedValue`, `Vulnerability`, `VulnerabilityDict`)
- **Reverse callers**: 58 total (3 non-test: `detect_ssr_vulnerabilities`, `security_analyzer.analyze`, `taint_tracker.fork`; 55 test callers)
- **K-core**: coreness=24 (densest core in entire corpus, alongside surgical_extractor.py and call_graph.py)
- **Parseltongue has equivalent?**: No -- Parseltongue has zero security analysis capabilities. No taint tracking, no vulnerability detection, no Z3 integration, no sanitizer registry. Parseltongue's nearest analog is dependency edge tracking in the graph (144,137 edges), which could theoretically model data flow if edges were tagged with taint propagation metadata. This is code-scalpel's deepest competitive moat (195 taint entities + 88 vulnerability entities = 283 security entities total).
**Implementation spec**:
**Feature name**: `security-taint-analysis-tracker` (new module in parseltongue-core)
**Where**: `parseltongue-core/src/security/taint_tracker.rs` + new endpoint in pt08 + new tree-sitter `.scm` queries
**CozoDB tables needed**:
  - New relation: `TaintSources { source_key: String, file_path: String, line: Int, variable: String, source_type: String, taint_level: String => language: String }`
  - New relation: `TaintSinks { sink_key: String, file_path: String, line: Int, operation: String, sink_type: String, cwe_id: String => language: String }`
  - New relation: `TaintFlows { source_key: String, sink_key: String => path_length: Int, sanitized: Bool, flow_path: String }`
**Tree-sitter queries**: New `.scm` patterns per language to detect sources (e.g., `request.args`, `process.env`, `stdin`) and sinks (e.g., `execute()`, `innerHTML`, `eval()`). Approximately 12 source patterns and 25 sink patterns across Python, JS, Java, Go.
**Graph integration**: Taint flows overlay on existing `DependencyEdges` -- reuse blast-radius traversal to find paths from source entities to sink entities. This is Parseltongue's unique advantage: blast-radius-aware taint tracking that no flat-file tool can replicate.
**Key dependencies**: No Z3 dependency (too heavy for Rust, immature bindings). Instead, use graph reachability on existing `DependencyEdges` to trace data flow. Simpler but leverages existing infrastructure.
**New HTTP endpoints**:
  - `/taint-flow-analysis-report?entity=X` (4-word: taint-flow-analysis-report)
  - `/vulnerability-detection-scan-results` (4-word: vulnerability-detection-scan-results)
**Estimated LoC**: ~3,000 Rust (taint source/sink detection via tree-sitter) + ~500 (CozoDB relations + queries) + ~400 (HTTP handlers) + ~600 (tree-sitter .scm patterns)
**Effort**: Large (6-8 weeks) -- tree-sitter query authoring for 12 languages is the bottleneck; taint propagation logic is graph traversal (existing infra)
**Risk**: High -- Z3-less approach means no symbolic reasoning about sanitizers; taint tracking is "structural" (follows call graph edges) not "semantic" (understands data transformations). This limits vulnerability detection accuracy compared to code-scalpel's Z3-backed approach. However, graph-based taint tracking is unique and defensible.

---

### Feature 1.3: Z3 Symbolic Execution Engine

**ELI5**: Code Scalpel can explore every possible path through a function without running it. For example, if a function has an `if/else`, it explores both branches and uses Z3 (a math solver) to figure out what input values would trigger each branch. This is useful for generating test cases automatically -- it tells you "if x=11, this branch executes; if x=0, that branch executes."

**Key Code Path**:
```
symbolic_execution_tools/engine.py:160 -> SymbolicAnalyzer class
  -> engine.py:233 -> analyze(code, language="python") entry point
  -> engine.py:252 -> checks cache (SHA256 of code + config)
  -> engine.py:275 -> _analyze_uncached()
  -> engine.py:278 -> TypeInferenceEngine() for Python type inference
  -> engine.py:279 -> ConstraintSolver(timeout_ms=2000) for Z3 solving
  -> engine.py:280 -> IRSymbolicInterpreter(max_loop_iterations=10)
  -> engine.py:291 -> PythonNormalizer().normalize(code) -> IR
  -> engine.py:293 -> JavaScriptNormalizer().normalize(code) -> IR
  -> engine.py:295 -> JavaNormalizer().normalize(code) -> IR
  -> engine.py:311-346 -> Extract function params, create z3 symbolic vars per type annotation
  -> engine.py:355 -> interpreter.execute(ir_module) -> terminal_states
  -> engine.py:379 -> _process_path() -> ConstraintSolver.solve(constraints, variables)
  -> engine.py:66 -> PathResult dataclass (path_id, status, constraints, variables, model, visited_lines)
  -> engine.py:107 -> AnalysisResult (paths, feasible_count, infeasible_count, total_paths)
```

**What the code actually does**: `SymbolicAnalyzer` is the top-level orchestration class. When `analyze()` is called, it first checks a unified cache (SHA256 of code + config). On cache miss, it creates three core components: `TypeInferenceEngine` (infers Z3 types from Python AST), `IRSymbolicInterpreter` (walks the IR with smart forking at branches, bounded loop unrolling at 10 iterations), and `ConstraintSolver` (wraps Z3 with a 2000ms timeout). The code is first normalized to a language-agnostic IR via `PythonNormalizer`, `JavaScriptNormalizer`, or `JavaNormalizer`. If a function definition is found, its parameters are converted to Z3 symbolic variables based on type annotations (Int, Real, Bool, String). The interpreter executes the IR, producing terminal states for each explored path. Each terminal state is passed through `ConstraintSolver.solve()` which checks satisfiability and extracts concrete Python values. Results are cached for future runs.

**Parseltongue comparison**:
- **File entity**: `python:file:__code-scalpel_src_code_scalpel_symbolic_execution_tools_engine_py:1-1`
- **CBO**: 91 (grade F) -- calls 91 unique entities including Z3 primitives (Bool, BoolRef, BoolSort, Int, IntSort, ExprRef), IR infrastructure (IRSymbolicInterpreter, ConstraintSolver, InferredType), and path analysis types (AnalysisResult, FEASIBLE, INFEASIBLE)
- **Class entity**: `python:class:SymbolicAnalyzer:____code_scalpel_src_code_scalpel_symbolic_execution_tools_engine:T1748822010`
- **Reverse callers**: 69 total (7 non-test: `cli._analyze_java`, `cli._analyze_javascript`, `symbolic_helpers._symbolic_execute_sync`, `server._symbolic_execute_sync`, `engine.create_analyzer`, `security_analyzer._verify_vulnerability`, `crewai.analyze_symbolic`)
- **Key insight**: SymbolicAnalyzer is called from both the MCP server path AND the CLI path AND the CrewAI agent integration -- 3 distinct entry points converging on the same engine.
- **Parseltongue has equivalent?**: No -- Parseltongue has no symbolic execution, no Z3 integration, no path exploration, no constraint solving. Parseltongue's `/smart-context-token-budget` endpoint returned 0 entities when queried with `focus=TaintTracker&tokens=5000`, confirming the smart context feature does not extend to security analysis. The nearest functional analog is Parseltongue's graph traversal algorithms (PageRank, K-core, Leiden) which explore "paths" in the dependency graph -- but these are structural paths, not execution paths.
**Implementation spec**:
**Feature name**: NOT RECOMMENDED for implementation
**Rationale**: Z3 symbolic execution requires an IR (Intermediate Representation) normalizer per language, a constraint solver, and path exploration engine. Code-scalpel invested ~5,000 LoC across `PythonNormalizer`, `JavaScriptNormalizer`, `JavaNormalizer`, `IRSymbolicInterpreter`, and `ConstraintSolver`. Replicating this in Rust would require:
  - `z3-sys` crate (C FFI bindings to Z3) -- compiles Z3 from source (~30min build, 500MB)
  - Custom IR definition + normalizers for each language
  - Path exploration with bounded loop unrolling
**Estimated LoC**: ~8,000-12,000 Rust (IR + normalizers + interpreter + Z3 bindings)
**Effort**: Very Large (12-16 weeks) -- this is a standalone project, not a feature
**Alternative**: Integrate with code-scalpel's symbolic execution via MCP tool delegation. If pt09 (MCP bridge) is built, Parseltongue could delegate symbolic execution requests to a running code-scalpel MCP instance and annotate results onto its graph. This gives graph-aware symbolic execution (e.g., "symbolically execute all functions in blast radius of entity X") without reimplementing Z3.
**Priority**: P3 (Ignore) -- code-scalpel has an unassailable lead here with 69 callers across 3 entry points. Build on their work via interop, not competition.

---

### Feature 1.4: Surgical Code Extraction (Token Budget)

**ELI5**: Instead of sending entire files to an LLM (which wastes tokens and money), Code Scalpel can surgically extract just one function or class from a file -- including its dependencies. It uses tiktoken to count exactly how many tokens the extracted code will cost, and can trim to fit a token budget. The idea is "feed the LLM 50 lines, not 5,000 lines."

**Key Code Path**:
```
surgery/surgical_extractor.py:62 -> count_tokens(text, model="gpt-4") with tiktoken
  -> surgical_extractor.py:103 -> ExtractionResult, CrossFileSymbol, ContextualExtraction dataclasses
  -> surgical_extractor.py:16 -> SurgicalExtractor(code) class
  -> entry: get_function("calculate_tax"), get_function_with_context("calculate_tax")
  -> get_method("Calculator", "add"), find_callers()
  -> surgical_extractor.py:57 -> resolve_file_path() for safe path resolution
  -> surgical_extractor.py:58 -> parse_python_code() from parsing/unified_parser.py
  -> mcp/tools/extraction.py:33 -> @mcp.tool() extract_code MCP tool
  -> mcp/tools/extraction.py:176 -> @mcp.tool() rename_symbol MCP tool
  -> mcp/tools/extraction.py:261 -> @mcp.tool() update_symbol MCP tool
```

**What the code actually does**: `SurgicalExtractor` takes raw source code and uses Python's `ast.parse()` to build an AST. The `get_function()` method walks the AST to find the function by name and extracts its complete source text. `get_function_with_context()` also resolves imports and dependencies. Token counting uses `tiktoken` when available (model-specific encodings for GPT-4, GPT-3.5, Claude) or falls back to `len(text) // 4`. The `CrossFileSymbol` dataclass tracks symbols resolved from external files with confidence scoring and depth metadata. Results are LRU-cached for a 2.8x speedup. The MCP tools `extract_code`, `rename_symbol`, and `update_symbol` expose this as a read-modify-write workflow: extract only what you need, modify it, write it back with backup and syntax validation.

**Parseltongue comparison**:
- **File entity**: `python:file:__code-scalpel_src_code_scalpel_surgery_surgical_extractor_py:1-1`
- **CBO**: 177 (grade F) -- calls 177 unique entities including AST manipulation (Assign, AsyncFor, AsyncFunctionDef, Call, ClassDef, AnnAssign), cross-file resolution types (ClosureAnalysisResult, ClosureVariable, ContextualExtraction, CrossFileResolution, CrossFileSymbol, CrossRepoImport)
- **Class entity**: `python:class:SurgicalExtractor:____code_scalpel_src_code_scalpel_surgery_surgical_extractor:T1707125572`
- **Reverse callers**: 143 total (31 non-test: `extraction_helpers._create_extractor`, `server._create_extractor`, `resources._extract_code_sync`, `server._extract_code_sync`, plus demo/example files)
- **K-core**: coreness=24 (densest core -- surgical_extractor.py is one of the most interconnected files in the entire corpus)
- **Parseltongue has equivalent?**: Yes, partial -- Parseltongue's `/smart-context-token-budget` endpoint serves a similar purpose (selecting entities within a token budget). However, it operates at the graph entity level, not at the AST node level. Code-scalpel extracts literal code text with dependencies; Parseltongue returns entity metadata (names, types, relationships). Parseltongue's 99% token reduction (50 tokens for stats vs. 388,670 for full entity list) demonstrates the same philosophy but through a different mechanism. Code-scalpel's LRU-cached extraction with tiktoken counting is more mature for the "feed minimal code to LLM" use case.
**Implementation spec**:
**Feature name**: `smart-context-source-extraction` (enhance existing module)
**Where**: Extend existing `/smart-context-token-budget` handler in `pt08-http-code-query-server/src/http_endpoint_handler_modules/smart_context_token_budget_handler.rs` + new source extraction in `parseltongue-core`
**CozoDB tables needed**: Extend `CodeGraph` with `source_text: String?` column (optional -- store raw source for entities during ingestion)
**Architecture**: Two-phase enhancement:
  1. **Phase 1 (ingestion)**: During pt01 ingestion, optionally store raw source text for each entity in `CodeGraph.source_text` (configurable via `--store-source` flag to control DB size). Tree-sitter already parses the full AST -- extracting node text is a one-liner.
  2. **Phase 2 (extraction)**: Extend `/smart-context-token-budget` to return actual source code (not just entity metadata) within the token budget. Add `tiktoken`-equivalent token counting via `tiktoken-rs` crate.
**Key dependencies**: `tiktoken-rs` (Rust bindings for OpenAI's tiktoken), existing tree-sitter infrastructure
**New HTTP endpoint enhancements**:
  - `/smart-context-token-budget?focus=X&tokens=N&include_source=true` -- returns source text within budget
  - `/surgical-extraction-entity-source?entity=X` (4-word: surgical-extraction-entity-source) -- returns raw source for a single entity with its dependency sources
**Estimated LoC**: ~800 Rust (source storage in pt01) + ~600 (token counting + budget allocation) + ~300 (HTTP handler changes)
**Effort**: Medium (2-3 weeks) -- tree-sitter already has the AST; main work is source text storage and token budget allocation
**Risk**: Low -- storage size increase is controllable via flag. `tiktoken-rs` is well-maintained. This directly competes with code-scalpel's SurgicalExtractor but leverages Parseltongue's graph for dependency-aware extraction (extract function X + all functions it calls within budget).

---

### Feature 1.5: Policy Engine (OPA/Rego + Cryptographic Verification)

**ELI5**: Code Scalpel can enforce coding standards using policies written in Rego (the same language used by Open Policy Agent). Policies are defined in YAML, and the engine verifies their integrity using cryptographic signatures to prevent tampering. If a policy check fails, the system defaults to "deny all" -- a fail-closed security model. Enterprise users can run HIPAA, SOC2, GDPR, and PCI-DSS compliance checks.

**Key Code Path**:
```
policy_engine/policy_engine.py:38 -> PolicyError exception
  -> policy_engine.py:44 -> PolicySeverity enum (CRITICAL, HIGH, MEDIUM, LOW, INFO)
  -> policy_engine.py:54 -> PolicyAction enum (DENY, WARN, AUDIT)
  -> policy_engine.py:62 -> Policy dataclass (name, description, rule, severity, action)
  -> policy_engine/crypto_verify.py -> cryptographic signature verification
  -> policy_engine/tamper_resistance.py -> tamper detection
  -> policy_engine/audit_log.py -> enterprise audit logging
  -> mcp/tools/policy.py:21 -> @mcp.tool() validate_paths (path accessibility checking)
  -> mcp/tools/policy.py:95 -> @mcp.tool() verify_policy_integrity (HMAC signature validation)
  -> mcp/tools/policy.py:171 -> @mcp.tool() code_policy_check (PEP8, ESLint, compliance)
```

**What the code actually does**: The `PolicyEngine` loads policies from YAML files, each containing a Rego rule, severity level, and action (DENY/WARN/AUDIT). The engine calls the OPA CLI (`shutil.which` to find it) to evaluate policies. The security model is "fail closed" -- any parsing, evaluation, or OPA CLI error results in DENY ALL. The `verify_policy_integrity` MCP tool checks cryptographic signatures against a manifest to detect tampered policy files. The `code_policy_check` tool runs tiered analysis: Community gets PEP8/ESLint basic style checks (max 100 files, 50 rules), Pro adds best practices and custom rules (1000 files, 200 rules), and Enterprise adds compliance standards (HIPAA, SOC2, GDPR, PCI-DSS) with PDF reports and audit logging.

**Parseltongue comparison**:
- **File entity**: `python:file:__code-scalpel_src_code_scalpel_policy_engine_policy_engine_py:1-1`
- **CBO**: 98 (grade F) -- calls 98 unique entities including OPA integration types (Policy, PolicyDecision, PolicyViolation, OverrideDecision, Operation), YAML/config handling (YAMLError, NamedTemporaryFile, TimeoutExpired, MULTILINE), and stdlib types
- **Class entities**: 2 resolved classes -- `python:class:PolicyEngine:____code_scalpel_src_code_scalpel_policy_engine_policy_engine:T1810149115` and `python:class:PolicyEngine:____code_scalpel_src_code_scalpel_security_analyzers_policy_engine:T1641188175` (two separate PolicyEngine implementations in different modules)
- **Reverse callers**: 35 total (3 non-test: `security_helpers._detect_custom_logging_rules`, `security_helpers._detect_policy_violations`, `unified_governance.__init__`)
- **Gemini CLI also has policy**: `typescript:fn:createPolicyEngineConfig` found in both `gemini-cli/packages/cli/` and `gemini-cli/packages/core/` with `PolicyEngineConfig` trait -- Google also invested in policy infrastructure for tool execution
- **Parseltongue has equivalent?**: No -- Parseltongue has no policy engine, no OPA/Rego integration, no compliance checking, no cryptographic verification. The closest concept is Parseltongue's `/technical-debt-sqale-scoring` endpoint which scores code health (ISO 25010), but this is read-only metrics, not enforcement. Policy enforcement would be a natural extension of graph queries (e.g., "deny if blast radius > 50 entities").
**Implementation spec**:
**Feature name**: `graph-policy-enforcement-engine` (new module in parseltongue-core)
**Where**: `parseltongue-core/src/policy/` + new endpoint in pt08
**CozoDB tables needed**:
  - New relation: `PolicyRules { rule_id: String => rule_name: String, rule_type: String, severity: String, action: String, datalog_query: String, description: String }`
  - New relation: `PolicyViolations { violation_id: String, rule_id: String, entity_key: String => timestamp: String, details: String }`
**Architecture**: Graph-native policy engine. Instead of OPA/Rego (external process), policies are expressed as CozoDB Datalog queries. This is Parseltongue's unique advantage -- policies like "no entity shall have CBO > 50" or "no circular dependency shall span more than 3 modules" are naturally expressible in Datalog over the existing graph. No external OPA CLI needed.
**Example policies (Datalog)**:
  - High coupling: `?[entity] := *CodeGraph{ISGL1_key: entity}, *DependencyEdges{from_key: entity}, count > 50`
  - Blast radius limit: `?[entity] := blast_radius(entity, 3) > 100`
  - Circular dep ban: `?[cycle] := *SCC{component: cycle}, len(cycle) > 1`
**Key dependencies**: None new -- CozoDB Datalog is the policy language. `serde_yaml` for policy definition files.
**New HTTP endpoints**:
  - `/policy-violation-detection-scan` (4-word: policy-violation-detection-scan)
  - `/policy-rules-management-list` (4-word: policy-rules-management-list)
**No OPA, no Rego, no cryptographic signing** -- those are enterprise overhead. Parseltongue's Datalog-native approach is simpler, faster, and leverages existing infrastructure.
**Estimated LoC**: ~1,500 Rust (policy engine + YAML loader + violation tracker) + ~300 (HTTP handlers)
**Effort**: Medium (3-4 weeks) -- Datalog queries are the natural fit; main work is policy YAML schema + violation reporting
**Risk**: Low -- CozoDB Datalog is already battle-tested for graph queries. Policy expressiveness is limited to graph properties (no source code pattern matching), but that is the correct scope for a graph database tool.

---

## 2. AiDex

### Feature 2.1: SQLite Code Index with WAL Mode

**ELI5**: AiDex creates a local SQLite database inside your project that stores every identifier, function, class, and type it finds. It uses WAL (Write-Ahead Logging) mode for fast concurrent reads during queries. Instead of grep-searching your entire codebase every time, the LLM can query the index in milliseconds. The database stores files, lines, items (terms), occurrences, methods, types, and even a task backlog.

**Key Code Path**:
```
src/db/database.ts:17 -> AiDexDatabase class wraps better-sqlite3
  -> database.ts:23 -> new Database(dbPath) constructor
  -> database.ts:28 -> pragma('journal_mode = WAL') for concurrent reads
  -> database.ts:29 -> pragma('foreign_keys = ON') for referential integrity
  -> database.ts:35 -> initSchema() reads schema.sql and executes it
  -> db/schema.sql:12 -> files table (path, hash, last_indexed)
  -> db/schema.sql:25 -> lines table (file_id, line_number, line_type, line_hash, modified)
  -> db/schema.sql:42 -> items table (term COLLATE NOCASE) for case-insensitive search
  -> db/schema.sql:52 -> occurrences table (item_id, file_id, line_id) junction table
  -> db/schema.sql:76 -> methods table (name, prototype, visibility, is_static, is_async)
  -> db/schema.sql:94 -> types table (name, kind: class/struct/interface/enum/type)
  -> db/schema.sql:109 -> dependencies table for cross-project linking
  -> db/schema.sql:133 -> tasks table (priority, status, tags) for task backlog
  -> database.ts:98 -> getStats() returns counts for all 7 entity types + file size
```

**What the code actually does**: `AiDexDatabase` wraps `better-sqlite3` with WAL mode and foreign keys enabled. The schema has 10 tables: `files` tracks the file tree with SHA256 hashes for change detection; `lines` stores individual lines with type classification (code/comment/struct/method/property/string) and per-line modification timestamps; `items` stores unique terms with case-insensitive collation; `occurrences` is a junction table linking items to their locations; `methods` stores function prototypes with visibility, static, and async flags; `types` stores classes/structs/interfaces/enums; `dependencies` enables cross-project linking; `project_files` categorizes all files by type (code/config/doc/asset/test); `tasks` and `task_log` implement a built-in task backlog with priority and status tracking. The `getStats()` method returns entity counts using COUNT(*) queries.

**Parseltongue comparison**:
- **File entity**: `typescript:file:__aidex_AiDex_src_db_database_ts:1-1`
- **CBO**: 20 (grade F -- borderline; threshold is 10) -- calls 20 unique entities
- **Class entity**: `typescript:class:AiDexDatabase:____aidex_AiDex_src_db_database:T1786728235` (CBO=0 at class level; edges tracked at file level)
- **Reverse callers**: 2 (`createDatabase` and `openDatabase` -- both in same file, factory functions wrapping the constructor)
- **Total AiDex entities**: 159 (vs. Parseltongue's ~287 Rust entities for its graph storage layer alone)
- **AiDex init.ts (indexing)**: CBO=66 (grade F) -- the indexing command is the most coupled file in AiDex
- **Parseltongue has equivalent?**: Yes, direct competitor -- Parseltongue uses CozoDB (graph database) with RocksDB backend instead of SQLite with WAL mode. Key differences: (1) AiDex stores entities in relational tables (files, lines, items, occurrences); Parseltongue stores entities as graph nodes with typed edges. (2) AiDex has 10 tables with inverted index semantics; Parseltongue has CozoDB relations with Datalog queries. (3) AiDex supports 7 entity types (files, lines, items, methods, types, tasks, dependencies); Parseltongue supports 9 entity types (fn, method, class, trait, typedef, enum, impl, struct, module). (4) Parseltongue's graph approach enables algorithms AiDex cannot: PageRank, Leiden community detection, K-core decomposition, SCC analysis -- none of which are possible with a flat SQLite index.
**Implementation spec**:
**Feature name**: NO NEW IMPLEMENTATION NEEDED -- Parseltongue already has a superior equivalent
**Where**: Existing `parseltongue-core/src/storage/cozo_client.rs` (CozoDB with RocksDB backend)
**Current state**: Parseltongue's `CodeGraph` relation (17 columns) + `DependencyEdges` relation (4 columns) + diagnostic relations already exceed AiDex's 10-table SQLite schema in both capability and query power. CozoDB's Datalog enables graph algorithms (PageRank, Leiden, K-core, SCC, betweenness centrality) that are impossible in SQLite without external graph libraries.
**Competitive advantage**: AiDex stores entities in flat relational tables (inverted index semantics). Parseltongue stores entities as graph nodes with typed edges, enabling transitive queries (blast radius, reverse callers N hops deep) in single Datalog expressions. The 44,890 entities and 385,560 edges in the current analysis DB demonstrate scale that SQLite WAL mode would struggle with for graph traversals.
**Gap to address**: AiDex's `tasks` table (built-in task backlog with priority/status) is a nice developer UX feature. Consider adding a `TaskBacklog` relation to CozoDB for parity if Tauri desktop app (v1.7.3 roadmap) needs task tracking.
**Effort**: None (existing) / Small (1 week for optional task backlog relation)

---

### Feature 2.2: Tree-sitter Multi-Language Parsing

**ELI5**: AiDex uses tree-sitter (the same parser used by Neovim and GitHub) to understand the structure of code in 11 programming languages. It creates a cached parser for each language and can parse files up to 1MB. The parser produces a concrete syntax tree that the extractor walks to find functions, classes, and other identifiers.

**Key Code Path**:
```
src/parser/tree-sitter.ts:5 -> import Parser from 'tree-sitter'
  -> tree-sitter.ts:8-17 -> imports for 11 language grammars (CSharp, TS, Rust, Python, C, Cpp, Java, Go, Php, Ruby)
  -> tree-sitter.ts:24 -> EXTENSION_MAP: 22 file extensions mapped to 11 languages
  -> tree-sitter.ts:50 -> parsers: Map<string, Parser> cached per-language
  -> tree-sitter.ts:55 -> getParser(language) creates and caches parser with setLanguage()
  -> tree-sitter.ts:130 -> PARSE_BUFFER_SIZE = 1MB (fixes tree-sitter 32KB limit)
  -> tree-sitter.ts:135 -> parse(sourceCode, language) returns Parser.Tree
  -> tree-sitter.ts:154 -> getParserForGrammar() handles tsx/jsx as virtual grammar keys
  -> tree-sitter.ts:177 -> parseFile(sourceCode, filePath) auto-detects language from extension
  -> parser/extractor.ts -> walks syntax tree to extract identifiers, methods, types
  -> parser/languages/*.ts -> per-language keyword filters (11 files)
```

**What the code actually does**: The tree-sitter module maintains a `Map<string, Parser>` of cached parsers. `getParser()` creates a parser on first use and caches it. The `EXTENSION_MAP` handles 22 file extensions including `.tsx`, `.jsx`, `.pyw`, `.rake`, and C++ variants (`.cc`, `.cxx`, `.hpp`). The buffer size is set to 1MB (1024*1024) to work around tree-sitter's default 32KB limit that caused "Invalid argument" errors. TSX and JSX get special handling via `getParserForGrammar()` which uses `TypeScript.tsx` grammar. JavaScript files are parsed with the TypeScript grammar since it's a superset. The `parseFile()` convenience function auto-detects language from file extension and returns a `Parser.Tree` or null for unsupported files.

**Parseltongue comparison**:
- **File entity**: `typescript:file:__aidex_AiDex_src_parser_tree-sitter_ts:1-1`
- **CBO**: 13 (grade F -- just above threshold of 10) -- calls 13 unique entities including parser setup, language detection, grammar resolution
- **Entities indexed**: 11 AiDex tree-sitter entities (`detectLanguage`, `getParser`, `getParserForGrammar`, `parseFile`, `isSupported`, `getSupportedExtensions`, `getGrammarKey`, etc.)
- **code-scalpel comparison**: code-scalpel has 30 tree-sitter entities with a `TreeSitterVisitor` pattern -- deeper visitor model vs. AiDex's utility-oriented approach
- **Parseltongue has equivalent?**: Yes, direct equivalent and stronger -- Parseltongue uses tree-sitter in Rust (via `tree-sitter` crate in `parseltongue-core`) to parse 12 languages (Rust, Python, JS, TS, Go, Java, C, C++, Ruby, PHP, C#, Swift). Key differences: (1) AiDex supports 11 languages with Node.js tree-sitter bindings; Parseltongue supports 12 with native Rust tree-sitter. (2) AiDex's 1MB buffer workaround addresses tree-sitter's 32KB limit; Parseltongue handles this at the Rust level with direct buffer management. (3) AiDex caches parsers in a `Map<string, Parser>`; Parseltongue creates parsers per-language in `parseltongue-core`. (4) AiDex extracts identifiers and methods; Parseltongue extracts full entity metadata (functions, classes, methods, traits, enums, structs) with dependency edges. This analysis corpus itself validates Parseltongue's parser: 19,431 entities extracted from 1,012 files across 6 languages.
**Implementation spec**:
**Feature name**: NO NEW IMPLEMENTATION NEEDED -- Parseltongue already has a superior equivalent
**Where**: Existing `parseltongue-core/` tree-sitter integration (Rust-native bindings for 12 languages)
**Current state**: Parseltongue parses 12 languages (Rust, Python, JS, TS, Go, Java, C, C++, Ruby, PHP, C#, Swift) vs. AiDex's 11 languages. Parseltongue uses Rust-native tree-sitter bindings (zero Node.js overhead) with direct buffer management. The current analysis DB (44,890 entities from 12 languages) validates production-grade parsing.
**Gap to address**: AiDex handles TSX/JSX with special grammar routing (`TypeScript.tsx`). Verify Parseltongue's tree-sitter handles `.tsx` and `.jsx` correctly via the existing `Language::from_file_path()` in `entities.rs` (it maps `tsx` to TypeScript, `jsx` to JavaScript). Also, AiDex's 1MB buffer workaround is a non-issue in Rust where tree-sitter has no 32KB default limit.
**Potential improvement**: Add Lua language support (for CodeCompanion.nvim analysis -- see Feature 4.1 gap). Tree-sitter has a `tree-sitter-lua` grammar. This would make Parseltongue support 13 languages.
**Effort**: None (existing) / Small (3-5 days for optional Lua support)

---

### Feature 2.3: Incremental Re-indexing with Hash-based Change Detection

**ELI5**: When you re-index a project, AiDex does not start from scratch. It computes a SHA256 hash of each file and compares it to the stored hash. If a file has not changed, it skips it entirely. It also respects .gitignore patterns and has 40+ built-in exclude patterns for node_modules, build output, and IDE files. Modified files detected during session start are automatically re-indexed.

**Key Code Path**:
```
src/commands/init.ts:16 -> shortHash(content) = SHA256 first 16 hex chars
  -> init.ts:172 -> init(params) main entry point
  -> init.ts:218 -> dbExists && !params.fresh = incremental mode
  -> init.ts:230 -> readGitignore(projectPath) parses .gitignore to glob patterns
  -> init.ts:231 -> DEFAULT_EXCLUDE array (40+ patterns: node_modules, build, dist, __pycache__, .git, etc.)
  -> init.ts:236 -> glob(pattern, { cwd, ignore: exclude }) finds source files
  -> init.ts:256 -> db.transaction(() => { for file of files: indexFile(...) })
  -> init.ts:259 -> indexFile() compares hash, skips unchanged files
  -> commands/session.ts:117 -> detectExternalChanges() finds modified/deleted/new files
  -> commands/session.ts:121 -> auto-reindex modified files via update()
```

**What the code actually does**: The `init()` function checks if an existing database exists; if so, it enters incremental mode. For each source file, `indexFile()` computes `shortHash()` (first 16 hex chars of SHA256) and compares it to the stored hash. Unchanged files are skipped (counted as `filesSkipped`). The function reads `.gitignore` and converts patterns to glob format (e.g., `foo/` becomes `**/foo/**`). All file operations happen inside a SQLite transaction for atomicity. The session system (`session.ts`) detects external changes by comparing file modification timestamps and hashes. When a new session starts (more than 5 minutes since last activity), `detectExternalChanges()` finds modified, deleted, and new files, then automatically re-indexes modified ones via `update()`.

**Parseltongue comparison**:
- **File entity**: `typescript:file:__aidex_AiDex_src_commands_init_ts:1-1`
- **CBO**: 66 (grade F) -- the indexing command is the most coupled file in AiDex, calling 66 unique entities for file hashing, glob scanning, .gitignore parsing, and incremental skip logic
- **Indexed entity**: `typescript:fn:indexFile:unresolved-reference:0-0` (1 unresolved reference -- the function is called but its definition was not resolved within the ingested code)
- **code-scalpel incremental**: 44 entities (`IncrementalIndex`, `IncrementalIndexer`, `IncrementalASTCache`, `IncrementalAnalyzer`)
- **Parseltongue has equivalent?**: Yes, stronger -- Parseltongue has an always-on file watcher (confirmed active: `file_watcher_active: true` in health check) that detects file changes in real-time and triggers re-ingestion. Key differences: (1) AiDex uses SHA256 hash comparison on explicit re-index command; Parseltongue uses filesystem events for automatic detection. (2) AiDex has 40+ built-in exclude patterns; Parseltongue respects .gitignore natively. (3) AiDex's incremental mode skips unchanged files in a batch re-index; Parseltongue's file watcher provides continuous incremental updates without explicit commands. (4) code-scalpel's incremental approach (44 entities) is more complex than AiDex's but still batch-oriented. Parseltongue's event-driven approach is architecturally superior for IDE/editor integration.
**Implementation spec**:
**Feature name**: NO NEW IMPLEMENTATION NEEDED -- Parseltongue's file watcher is architecturally superior
**Where**: Existing `pt08-http-code-query-server/src/file_watcher_integration_service.rs` + `incremental_reindex_core_logic.rs`
**Current state**: Parseltongue has an always-on file watcher (`file_watcher_active: true` confirmed in health check) that detects filesystem events in real-time and triggers incremental re-ingestion. This is event-driven (push model) vs. AiDex's hash-based polling (pull model). Parseltongue never needs to "start from scratch" or "compare hashes" -- changes are detected as they happen.
**Gap to address**: AiDex's 40+ built-in exclude patterns (node_modules, build, dist, __pycache__, .git, etc.) and .gitignore parsing are useful UX features. Verify Parseltongue's file watcher respects .gitignore and has sensible defaults. The `initial_scan.rs` module in pt08 likely handles this.
**Competitive advantage**: Parseltongue's file watcher + CozoDB incremental update is a stronger architecture than AiDex's batch-oriented SHA256 hash comparison. No explicit "re-index" command needed.
**Effort**: None (existing)

---

### Feature 2.4: Session Management with External Change Detection

**ELI5**: AiDex tracks when you start and end coding sessions. If you close your editor and come back later, it knows you are in a "new session" and automatically checks if anyone (or any tool) changed files while you were away. It re-indexes those files and tells you what changed. It also supports session notes that persist in the database.

**Key Code Path**:
```
src/commands/session.ts:22 -> SessionParams { path }
  -> session.ts:37 -> SessionResult { isNewSession, externalChanges, reindexed, note }
  -> session.ts:57 -> SESSION_TIMEOUT_MS = 5 * 60 * 1000 (5 minutes)
  -> session.ts:68 -> session(params) main function
  -> session.ts:97 -> isNewSession = now - lastActivity > SESSION_TIMEOUT_MS
  -> session.ts:104 -> archives previous session times in metadata table
  -> session.ts:118 -> detectExternalChanges(projectPath, queries) -> ChangedFile[]
  -> session.ts:121-128 -> auto-reindex: for change.reason === 'modified' -> update()
  -> session.ts:144 -> db.setMetadata(KEY_LAST_SESSION_END, now) as heartbeat
  -> session.ts:178 -> updateSessionHeartbeat() for periodic calls
```

**What the code actually does**: The session system uses the `metadata` table to store timestamps: `current_session_start`, `last_session_start`, `last_session_end`, and `session_note`. When `session()` is called, it checks if more than 5 minutes have elapsed since the last activity (either `last_session_end` or `current_session_start`). If yes, it starts a new session: archives previous session times, runs `detectExternalChanges()` to find files that were modified/deleted/added outside the session, and auto-reindexes modified files. Every call also updates the `last_session_end` timestamp as a heartbeat. The `updateSessionHeartbeat()` function can be called periodically to keep the session alive.

**Parseltongue comparison**:
- **File entity**: `typescript:file:__aidex_AiDex_src_commands_session_ts:1-1`
- **CBO**: 20 (grade F) -- session management calls 20 unique entities including timestamp comparison, metadata storage, external change detection
- **Entities indexed**: `detectExternalChanges` (resolved at `____aidex_AiDex_src_commands_session:T1678563409`), `formatDuration`, `formatSessionTime`, `getSessionInfo`, `getSessionNote`, `updateSessionHeartbeat`
- **code-scalpel session**: `SessionManager` class (`python:class:SessionManager:____code_scalpel_src_code_scalpel_mcp_helpers_session:T1829407042`) with audit logging, project root detection, persistent audit trail -- 26 entities total
- **Parseltongue has equivalent?**: No -- Parseltongue has no session management concept. The HTTP server is stateless (each request is independent, backed by CozoDB queries). The file watcher provides continuous monitoring but does not track "sessions" or "external changes since last activity." This is a gap for IDE integration scenarios where users expect the tool to know what changed while they were away. However, Parseltongue's always-on file watcher partially addresses this by automatically re-ingesting changed files without requiring explicit session start/end signals.
**Implementation spec**:
**Feature name**: `session-lifecycle-management-tracker` (new module in pt08)
**Where**: `pt08-http-code-query-server/src/session_lifecycle_management_tracker.rs` + new CozoDB relation
**CozoDB tables needed**:
  - New relation: `SessionLog { session_id: String => start_time: String, end_time: String?, heartbeat_time: String, session_note: String?, files_changed: Int, files_reindexed: Int }`
**Architecture**: Lightweight session tracking layered on the existing file watcher. When a client connects (first HTTP request after idle period), start a new session. Track heartbeats via periodic `/server-health-check-status` calls. When the file watcher detects changes, log them against the active session. Session notes stored in `SessionLog` relation.
**Key dependencies**: None new -- uses existing file watcher events + CozoDB storage
**New HTTP endpoints**:
  - `/session-activity-tracking-status` (4-word: session-activity-tracking-status) -- current session info + changes since last query
  - `/session-history-timeline-list` (4-word: session-history-timeline-list) -- list past sessions with change summaries
**Integration with Tauri**: The Tauri desktop app (v1.7.3 roadmap) would be the primary consumer -- show "Welcome back, 3 files changed while you were away" on app launch.
**Estimated LoC**: ~600 Rust (session state machine + CozoDB ops) + ~300 (HTTP handlers)
**Effort**: Small (1-2 weeks) -- simple state machine, most infrastructure exists
**Risk**: Low -- session detection is just timestamp comparison. Main design decision: session timeout threshold (AiDex uses 5 minutes; Parseltongue could use configurable `--session-timeout` flag).

---

### Feature 2.5: MCP Server Setup (22 Tools)

**ELI5**: AiDex exposes 22 tools to AI assistants via MCP. It uses the official MCP SDK with stdio transport (the AI spawns it as a subprocess). Tools cover search (query, files), signatures (get type/method info without reading whole files), project overview (summary, tree), cross-project linking, session tracking, task management, and even screenshot capture. The server registering pattern is simple: a `registerTools()` function returns tool definitions, and `handleToolCall()` dispatches by name.

**Key Code Path**:
```
src/index.ts:13 -> import { createServer } from './server/mcp-server.js'
  -> index.ts:94 -> createServer() -> server.start() with StdioServerTransport
  -> server/mcp-server.ts:5 -> import { Server } from '@modelcontextprotocol/sdk/server/index.js'
  -> server/mcp-server.ts:14 -> createServer() function
  -> server/mcp-server.ts:28 -> server.setRequestHandler(ListToolsRequestSchema, () => registerTools())
  -> server/mcp-server.ts:35 -> server.setRequestHandler(CallToolRequestSchema, (req) => handleToolCall(name, args))
  -> server/tools.ts -> registerTools() returns array of 22 tool definitions
  -> server/tools.ts -> handleToolCall() dispatches to command modules
  -> commands/init.ts, query.ts, signature.ts, session.ts, task.ts, etc.
```

**What the code actually does**: `createServer()` instantiates a `Server` from the MCP SDK with `capabilities: { tools: {} }`. It registers two request handlers: `ListToolsRequestSchema` returns the array of 22 tool definitions from `registerTools()`, and `CallToolRequestSchema` dispatches to `handleToolCall()` which routes by tool name to the appropriate command module. The server uses `StdioServerTransport` for subprocess communication. Graceful shutdown handlers for SIGTERM and SIGINT stop the viewer server and exit cleanly.

**Parseltongue comparison**:
- **File entity**: `typescript:file:__aidex_AiDex_src_server_mcp-server_ts:1-1`
- **CBO**: 6 (grade D -- the healthiest file in AiDex, well below threshold of 10)
- **Entity**: `typescript:fn:createServer:____aidex_AiDex_src_server_mcp_server:T1636640397`
- **Total AiDex entities**: 159 across 22 MCP tools (search, signatures, project overview, cross-project linking, session, tasks, screenshot)
- **Parseltongue has equivalent?**: Structural parallel but different protocol -- Parseltongue exposes 22 HTTP REST endpoints (exact same count as AiDex's 22 MCP tools). Key differences: (1) AiDex uses MCP SDK with stdio transport (subprocess model); Parseltongue uses Axum HTTP server (network model). (2) AiDex's tools are invoked by AI agents via MCP protocol; Parseltongue's endpoints are invoked via curl/HTTP clients. (3) AiDex has specialized tools (screenshot capture, task management, session notes) that Parseltongue lacks. (4) Parseltongue has graph analysis endpoints (PageRank, Leiden, K-core, SCC, SQALE, entropy, coupling/cohesion) that AiDex cannot offer with its SQLite backend. The 22-vs-22 tool count is a coincidence but reveals similar scope ambitions with fundamentally different architectures.
**Implementation spec**:
**Feature name**: `pt09-mcp-protocol-bridge-server` (same as Feature 1.1 -- single crate serves both needs)
**Where**: Same crate as Feature 1.1 spec. AiDex's 22 MCP tools map directly to Parseltongue's 24 HTTP endpoints. The pt09 bridge crate covers both code-scalpel MCP parity (Feature 1.1) and AiDex MCP parity (Feature 2.5) simultaneously.
**AiDex-specific tool mappings**:
  - AiDex `query` -> PT `/code-entities-search-fuzzy`
  - AiDex `signatures` -> PT `/code-entity-detail-view/{key}` (entity metadata includes interface signatures)
  - AiDex `summary` -> PT `/codebase-statistics-overview-summary`
  - AiDex `tree` -> PT `/folder-structure-discovery-tree`
  - AiDex `session` -> PT `/session-activity-tracking-status` (if Feature 2.4 spec is implemented)
  - AiDex `screenshot` -> NOT IMPLEMENTED (out of scope for a code analysis tool)
  - AiDex `cross_project` -> PT could support via multi-DB mode (load multiple `analysis.db` files)
**Gap**: AiDex's task management tools (`task_add`, `task_list`, `task_update`) have no PT equivalent. Low priority unless Tauri app needs built-in task tracking.
**Estimated LoC**: Included in Feature 1.1 estimate (~1,200 Rust total for pt09)
**Effort**: Included in Feature 1.1 (2-3 weeks)
**Risk**: Low -- same as Feature 1.1

---

## 3. Gemini CLI

### Feature 3.1: Event-Driven Tool Scheduler

**ELI5**: Gemini CLI has a sophisticated scheduler that manages tool execution with queuing, cancellation, confirmation prompts, and policy checks. When the LLM wants to run a tool, the scheduler validates it, checks security policies, optionally asks the user for confirmation, executes it, and sends results back. It handles batches of tool calls, queues new requests while processing, and supports abort signals for cancellation.

**Key Code Path**:
```
packages/core/src/scheduler/scheduler.ts:83 -> class Scheduler
  -> scheduler.ts:87 -> SchedulerStateManager for state tracking
  -> scheduler.ts:88 -> ToolExecutor for actual execution
  -> scheduler.ts:89 -> ToolModificationHandler for pre/post modification
  -> scheduler.ts:101 -> constructor(options: SchedulerOptions) with Config, MessageBus, Editor
  -> scheduler.ts:108 -> state = new SchedulerStateManager(messageBus, schedulerId, logToolCall)
  -> scheduler.ts:113 -> executor = new ToolExecutor(config)
  -> scheduler.ts:145 -> schedule(request, signal) -> Promise<CompletedToolCall[]>
  -> scheduler.ts:149 -> runInDevTraceSpan({ name: 'schedule' }) wraps in telemetry span
  -> scheduler.ts:155 -> if processing: _enqueueRequest(); else: _startBatch()
  -> scheduler.ts:239 -> _startBatch() -> resolve tools, validate, check policy, confirm, execute
  -> scheduler.ts:248 -> config.getToolRegistry().getTool(name) for resolution
  -> scheduler/state-manager.ts -> SchedulerStateManager for state transitions
  -> scheduler/confirmation.ts -> resolveConfirmation() for user approval
  -> scheduler/policy.ts -> checkPolicy(), updatePolicy() for security
  -> scheduler/tool-executor.ts -> ToolExecutor.execute() for actual invocation
  -> scheduler/types.ts -> CoreToolCallStatus enum (Queued, Validating, Executing, Success, Error, Cancelled)
```

**What the code actually does**: The `Scheduler` class is an event-driven orchestrator. It uses `SchedulerStateManager` to track tool call lifecycle through states: Queued -> Validating -> Executing -> Success/Error/Cancelled. When `schedule()` is called, it wraps execution in a telemetry span via `runInDevTraceSpan()`. If already processing, requests are queued with abort signal support. `_startBatch()` resolves each tool from the `ToolRegistry`, validates parameters, checks security policy via `checkPolicy()`, resolves confirmation with the user via `MessageBus`, and finally executes via `ToolExecutor`. The `cancelAll()` method drains the queue and cancels the active call. A `WeakSet` of MessageBus instances prevents duplicate listener registration across scheduler instances.

**Parseltongue comparison**:
- **File entity**: `typescript:file:__agent-client-protocol_gemini-cli_packages_core_src_scheduler_scheduler_ts:1-1`
- **CBO**: 92 (grade F) -- calls 92 unique entities including execution lifecycle types (Scheduled, Executing, Success, Cancel, Cancelled), policy enforcement (DENY, ASK_USER, ProceedOnce), error handling (TOOL_NOT_REGISTERED, INVALID_TOOL_PARAMS, UNHANDLED_EXCEPTION), and Promise-based async coordination
- **Class entity**: `typescript:class:Scheduler:____agent_client_protocol_gemini_cli_packages_core_src_scheduler_scheduler:T1620034677` (CBO=1 at class level)
- **Reverse callers**: 3 non-test callers (`nonInteractiveCli.runNonInteractive`, `agent-scheduler.scheduleAgentTools`, `useToolScheduler`)
- **Related entities**: `SchedulerStateManager`, `ToolExecutor`, `ToolModificationHandler`, `CoreToolCallStatus` enum, `CoreToolScheduler`, `checkPolicy`, `awaitConfirmation`
- **Parseltongue has equivalent?**: No -- Parseltongue has no tool scheduling, no execution lifecycle management, no confirmation/approval flows, no policy-gated execution. Parseltongue's HTTP server processes requests synchronously (query CozoDB, return JSON). The scheduler pattern is relevant to Parseltongue only if MCP server mode is added (P3 roadmap item from CR-v173-02) -- at which point tool scheduling becomes necessary for managing concurrent analysis requests.
**Implementation spec**:
**Feature name**: NOT RECOMMENDED as a standalone feature -- partially covered by pt09 MCP bridge
**Rationale**: The event-driven tool scheduler is a client-side orchestration pattern for LLM agents that invoke multiple tools. Parseltongue is a data provider, not an LLM agent orchestrator. The scheduler becomes relevant only if Parseltongue evolves into an autonomous code analysis agent.
**If MCP server (pt09) is built**: The MCP protocol itself handles tool call lifecycle (request -> response). The MCP SDK (`rmcp`) provides built-in request queuing and cancellation via the protocol's `notifications/cancelled` message. No custom scheduler needed for pt09.
**If autonomous agent mode is later desired**: Extract Gemini CLI's `Scheduler` + `SchedulerStateManager` + `ToolExecutor` pattern into a `pt10-autonomous-analysis-agent` crate. But this is a fundamentally different product direction (from "data provider" to "autonomous agent").
**Estimated LoC**: N/A (not recommended)
**Effort**: N/A
**Risk**: Building an agent orchestrator would dilute Parseltongue's focus on graph analysis. Let Claude Code, Gemini CLI, and Cursor handle orchestration -- Parseltongue should be the best tool they call, not a competing orchestrator.

---

### Feature 3.2: MCP Client Manager with Multi-Server Discovery

**ELI5**: Gemini CLI can connect to multiple MCP servers simultaneously. It manages the lifecycle of each connection (start, stop, discover tools, handle errors) and supports multiple authentication methods including Google OAuth, service account impersonation, and standard OAuth. It can also load/unload servers dynamically as extensions are enabled/disabled, and has admin-level allowlists/blocklists for controlling which servers are permitted.

**Key Code Path**:
```
packages/core/src/tools/mcp-client-manager.ts:28 -> class McpClientManager
  -> mcp-client-manager.ts:29 -> clients: Map<string, McpClient>
  -> mcp-client-manager.ts:31 -> allServerConfigs: Map<string, MCPServerConfig>
  -> mcp-client-manager.ts:37 -> discoveryState: MCPDiscoveryState (NOT_STARTED, IN_PROGRESS, COMPLETED)
  -> mcp-client-manager.ts:72 -> stopExtension(extension) disconnects all servers for an extension
  -> mcp-client-manager.ts:101 -> startExtension(extension) discovers all servers for an extension
  -> mcp-client-manager.ts:118 -> isBlockedBySettings(name) checks allowlist/excludelist
  -> tools/mcp-client.ts:7 -> Client from @modelcontextprotocol/sdk/client
  -> mcp-client.ts:14-18 -> SSEClientTransport, StdioClientTransport, StreamableHTTPClientTransport
  -> mcp-client.ts:39-42 -> GoogleCredentialProvider, ServiceAccountImpersonationProvider
  -> mcp-client.ts:48 -> MCPOAuthProvider, MCPOAuthTokenStorage
  -> mcp-client.ts:41 -> DiscoveredMCPTool for wrapping discovered tools
  -> mcp-client.ts:85 -> MCPServerStatus enum (DISCONNECTED, DISCONNECTING, CONNECTING, CONNECTED)
  -> mcp-client.ts:99 -> MCPDiscoveryState enum (NOT_STARTED, IN_PROGRESS, COMPLETED, FAILED)
```

**What the code actually does**: `McpClientManager` maintains a `Map<string, McpClient>` of active connections and a parallel `Map<string, MCPServerConfig>` of all configured servers. The `startExtension()` method iterates over an extension's `mcpServers` config, calling `maybeDiscoverMcpServer()` for each. `McpClient` supports three transport types: `StdioClientTransport` (subprocess), `SSEClientTransport` (Server-Sent Events), and `StreamableHTTPClientTransport` (HTTP streaming). Authentication is pluggable via `McpAuthProvider` interface with implementations for Google credentials, service account impersonation, and standard OAuth with PKCE. The `isBlockedBySettings()` method checks admin allowlists/excludelists. Tool, resource, and prompt list changes are listened for via `ToolListChangedNotificationSchema`, `ResourceListChangedNotificationSchema`, and `PromptListChangedNotificationSchema`.

**Parseltongue comparison**:
- **Entity**: `typescript:fn:McpClientManager:unresolved-reference:0-0` (unresolved -- the class definition was not fully indexed; only references to it were captured)
- **Reverse callers**: 0 for `McpClientManager` unresolved ref (the class is likely instantiated via factory methods not captured as direct calls)
- **Related entities**: `setupMcpClients` and `unsetupMcpClients` in AiDex (`____aidex_AiDex_src_commands_setup`), 3 transport types visible in gemini-cli (StdioClientTransport, SSEClientTransport, StreamableHTTPClientTransport)
- **MCP client infrastructure**: gemini-cli has 138 MCP-related entities (client-side), including `DiscoveredMCPTool`, `HybridTokenStorage`, `KeychainTokenStorage`, `MCPOAuthTokenStorage`, `ServiceAccountImpersonationProvider`, `XcodeMcpBridgeFixTransport` -- enterprise-grade auth and discovery
- **Parseltongue has equivalent?**: No -- Parseltongue is neither an MCP server nor an MCP client. It does not discover, connect to, or manage external tool servers. The multi-server discovery pattern is only relevant if Parseltongue adds MCP client mode (to consume tools from other MCP servers) or if Parseltongue becomes an MCP server that other clients discover. Currently, Parseltongue operates as a standalone HTTP server with no protocol-level interop with the MCP ecosystem.
**Implementation spec**:
**Feature name**: NOT RECOMMENDED for implementation
**Rationale**: MCP client management (multi-server discovery, 3 transports, Google OAuth, admin allowlists) is a client-side infrastructure concern for tools that consume MCP servers. Parseltongue is an MCP server (producer), not an MCP client (consumer). Gemini CLI's 138 MCP-related entities represent Google's investment in a universal MCP client -- Parseltongue should be discoverable by that client, not compete with it.
**What pt09 needs instead**: pt09 (MCP bridge) needs to be a well-behaved MCP server that clients like Gemini CLI, Claude Desktop, and Cursor can discover. This means:
  - Proper `initialize` handshake with capabilities declaration
  - `tools/list` returning all 24 tools with JSON Schema parameter descriptions
  - `tools/call` handling with structured error responses
  - stdio transport (spawned as subprocess) and optionally streamable-HTTP
**These are all covered by the pt09 spec in Feature 1.1** -- no separate MCP client infrastructure needed.
**Effort**: N/A (not recommended)

---

### Feature 3.3: OpenTelemetry Telemetry System

**ELI5**: Gemini CLI has a comprehensive observability system built on OpenTelemetry. It tracks traces (what happened when), metrics (how often, how fast), and logs (what went wrong). Data can be exported to Google Cloud Platform or to local files. It also includes specialized loggers for tool calls, API requests, API errors, flash fallbacks, and conversation events. There is even a memory monitor and a Clearcut logger (Google's internal logging system).

**Key Code Path**:
```
packages/core/src/telemetry/index.ts:7 -> TelemetryTarget enum (GCP, LOCAL)
  -> index.ts:16-20 -> exports: initializeTelemetry, shutdownTelemetry, flushTelemetry
  -> index.ts:33-50 -> exports: logToolCall, logApiRequest, logApiError, logApiResponse, logFlashFallback, etc.
  -> telemetry/sdk.ts:8-14 -> imports from @opentelemetry/api (diag, trace, context, metrics, propagation)
  -> sdk.ts:15-17 -> OTLPTraceExporter, OTLPLogExporter, OTLPMetricExporter (gRPC)
  -> sdk.ts:18-20 -> HTTP variants for trace, log, metric export
  -> sdk.ts:22 -> NodeSDK from @opentelemetry/sdk-node
  -> sdk.ts:42 -> ClearcutLogger for Google's internal logging
  -> sdk.ts:44-47 -> FileLogExporter, FileMetricExporter, FileSpanExporter for local export
  -> sdk.ts:49-52 -> GcpTraceExporter, GcpMetricExporter, GcpLogExporter for GCP export
  -> telemetry/metrics.ts -> initializeMetrics() for custom metric instruments
  -> telemetry/memory-monitor.ts -> MemoryMonitor with global monitoring
  -> telemetry/clearcut-logger/ -> Google Clearcut integration
  -> telemetry/activity-detector.ts, activity-monitor.ts -> user activity tracking
```

**What the code actually does**: The telemetry system initializes via `initializeTelemetry()` which sets up a `NodeSDK` with configurable exporters. For GCP targets, it uses gRPC-based OTLP exporters with gzip compression; for local targets, it uses file-based exporters. A `ClearcutLogger` handles Google's internal Clearcut logging format. The system logs 15+ event types including `ToolCallEvent`, `ApiRequestEvent`, `ApiErrorEvent`, `UserPromptEvent`, `FlashFallbackEvent`, `ConversationFinishedEvent`, and `ChatCompressionEvent`. `MemoryMonitor` tracks memory usage with `recordCurrentMemoryUsage()` and supports global monitoring start/stop. Activity detection monitors user interaction patterns. All logging goes through a `DiagLoggerAdapter` that routes to a `debugLogger`.

**Parseltongue comparison**:
- **File entities**: `typescript:file:__agent-client-protocol_gemini-cli_packages_core_src_telemetry_types_ts:1-1` (CBO=201, grade F), `typescript:file:__agent-client-protocol_gemini-cli_packages_core_src_telemetry_sdk_ts:1-1` (CBO=38, grade F), `typescript:file:__agent-client-protocol_gemini-cli_packages_core_src_telemetry_clearcut-logger_clearcut-logger_ts:1-1` (CBO=360, grade F -- #3 hotspot in entire corpus)
- **ClearcutLogger**: `typescript:class:ClearcutLogger:____agent_client_protocol_gemini_cli_packages_core_src_telemetry_clearcut_logger_clearcut_logger:T1638633643` (CBO=1 at class level, 360 at file level)
- **Event types indexed**: 25+ event classes (AgentFinishEvent, AgentStartEvent, ApiErrorEvent, ApiRequestEvent, ApiResponseEvent, ApprovalModeDurationEvent, ApprovalModeSwitchEvent, ContentRetryEvent, ConversationFinishedEvent, EditCorrectionEvent, EditStrategyEvent, EndSessionEvent, ExtensionDisableEvent, ExtensionEnableEvent, ExtensionInstallEvent, ExtensionUninstallEvent, ExtensionUpdateEvent, etc.)
- **telemetry/types.ts callees**: 201 entities including HTTP_STATUS_CODE, agent_id, agent_name, address -- Google's telemetry type system is massive
- **Total gemini-cli telemetry entities**: 375 (from CR-v173-02)
- **Parseltongue has equivalent?**: No -- Parseltongue has no telemetry, no observability, no OpenTelemetry integration. The HTTP server returns response times in some endpoints but does not export traces, metrics, or logs. This is a significant gap for production deployment: no way to monitor query latency, ingestion throughput, or error rates. The 375 telemetry entities in gemini-cli reflect Google's enterprise-grade observability investment. For Parseltongue's Tauri desktop app plans (v1.7.3 roadmap), basic telemetry would be needed to monitor performance and usage patterns.
**Implementation spec**:
**Feature name**: `observability-metrics-telemetry-exporter` (new module in pt08)
**Where**: `pt08-http-code-query-server/src/observability_metrics_telemetry_exporter.rs`
**Architecture**: Lightweight telemetry using `tracing` + `tracing-subscriber` (already in Rust ecosystem, zero new dependencies). NOT full OpenTelemetry (375 entities is enterprise overkill for a dev tool). Track:
  - Request latency per endpoint (histogram)
  - Ingestion throughput (entities/sec during re-index)
  - CozoDB query execution time (per Datalog query)
  - Error rates by endpoint
  - File watcher event counts
**Key dependencies**: `tracing` (already likely in dependency tree), `tracing-subscriber` with JSON formatting, `metrics` crate for counters/histograms
**New HTTP endpoints**:
  - `/observability-metrics-telemetry-export` (4-word: observability-metrics-telemetry-export) -- returns JSON metrics snapshot (Prometheus-compatible format)
**Export targets**: JSON to stdout (default), optional file export. No GCP, no Clearcut, no gRPC OTLP -- those are Google-scale concerns. If Tauri desktop app needs it, add a metrics dashboard panel that reads from this endpoint.
**Estimated LoC**: ~800 Rust (tracing middleware + metrics collection + export handler)
**Effort**: Small-Medium (1-2 weeks) -- `tracing` ecosystem is mature and well-documented
**Risk**: Low -- `tracing` is the de facto Rust observability standard. Main decision: what to measure (start with request latency + error rate, expand later).

---

### Feature 3.4: Multi-Model Config with Auto-Routing

**ELI5**: Gemini CLI supports multiple Gemini models (Flash, Pro, Auto) with automatic fallback routing. The config system manages tool registration, prompt registries, resource registries, model selection, and MCP server discovery. It can automatically switch between models based on availability and task complexity, with telemetry logging for each fallback decision.

**Key Code Path**:
```
packages/core/src/config/config.ts:16-20 -> ContentGenerator, createContentGeneratorConfig
  -> config.ts:22-37 -> ToolRegistry with 15+ built-in tools (LS, ReadFile, Grep, RipGrep, Glob, Edit, Shell, WriteFile, WebFetch, WebSearch, etc.)
  -> config.ts:52 -> tokenLimit from core/tokenLimits.js
  -> config.ts:57-64 -> DEFAULT_GEMINI_MODEL, DEFAULT_GEMINI_FLASH_MODEL, DEFAULT_GEMINI_MODEL_AUTO, resolveModel, isAutoModel, isPreviewModel
  -> config.ts:82 -> ModelAvailabilityService for checking model availability
  -> config.ts:83 -> ModelRouterService for auto-routing between models
  -> config.ts:89 -> ModelConfigService with DEFAULT_MODEL_CONFIGS
  -> config.ts:91 -> ContextManager for managing conversation context
  -> config/models.ts -> model name constants and resolution logic
  -> routing/modelRouterService.ts -> smart model selection
  -> availability/modelAvailabilityService.ts -> model health checks
  -> fallback/types.ts -> FallbackModelHandler, ValidationHandler
```

**What the code actually does**: The `Config` class is the central configuration hub. It initializes a `ToolRegistry` with 15+ built-in tools (LS, ReadFile, Grep, RipGrep, Glob, Edit, Shell, WriteFile, WebFetch, MemoryTool, WebSearch, AskUser, EnterPlanMode, ExitPlanMode, WriteTodos, ActivateSkill). It creates a `ContentGenerator` that wraps the Gemini API client with model-specific token limits. The `ModelRouterService` handles automatic model selection based on availability and task requirements, with `FlashFallbackEvent` telemetry. `ModelConfigService` manages per-model configurations with `DEFAULT_MODEL_CONFIGS`. The `ContextManager` handles conversation context compression and truncation. Model names support "auto" mode (`DEFAULT_GEMINI_MODEL_AUTO`) which dynamically selects between Flash and Pro based on the `ModelAvailabilityService`.

**Parseltongue comparison**:
- **File entity**: `typescript:file:__agent-client-protocol_gemini-cli_packages_core_src_config_config_ts:1-1`
- **CBO**: 343 (grade F -- #4 hotspot in entire corpus, 14h tech debt) -- calls 343 unique entities including model routing types (BANNER_TEXT_CAPACITY_ISSUES, CLASSIFIER_THRESHOLD, CONTEXT_COMPRESSION_THRESHOLD, ENABLE_ADMIN_CONTROLS, ENABLE_NUMERICAL_ROUTING), masking config (MASKING_PROTECTION_THRESHOLD, MASKING_PROTECT_LATEST_TURN, MASKING_PRUNABLE_THRESHOLD), and tool registry setup (Map, Name, Partial, PLAN, DEFAULT, AgentsRefreshed)
- **Related entities**: `ModelAvailabilityService`, `ModelRouterService`, `ModelConfigService`, `ContextManager`, `FallbackModelHandler` -- sophisticated model orchestration layer
- **Parseltongue has equivalent?**: No -- Parseltongue does not interact with LLMs directly. It is a code analysis tool that produces data for LLM consumption, not an LLM orchestrator. The multi-model config pattern is orthogonal to Parseltongue's purpose. However, Parseltongue's `/smart-context-token-budget` endpoint implicitly assumes a target model's token limit -- extending this with model-aware token budgets (GPT-4 vs. Claude vs. Gemini limits) would be a lightweight way to add model awareness without full routing.
**Implementation spec**:
**Feature name**: `token-budget-model-awareness` (enhance existing module)
**Where**: Extend existing `/smart-context-token-budget` handler + config in pt08
**Architecture**: Add a `--model` query parameter to `/smart-context-token-budget` that adjusts token limits per model family:
  - `gpt-4o`: 128K context, default 4K budget
  - `claude-sonnet`: 200K context, default 8K budget
  - `gemini-pro`: 1M context, default 32K budget
  - `gemini-flash`: 1M context, default 16K budget
  - Custom: `--tokens=N` overrides any model default
**Key dependencies**: `tiktoken-rs` (same as Feature 1.4 spec) for accurate per-model token counting
**New HTTP endpoint enhancement**:
  - `/smart-context-token-budget?focus=X&model=claude-sonnet&tokens=8000` -- model-aware budget
**CozoDB tables needed**: None -- model config is a static lookup table in Rust code
**Why NOT full multi-model routing**: Parseltongue does not call LLMs. It produces data for LLMs. Model awareness here means "format output optimally for the consuming model's context window" -- not "route requests to different models." This is a clean, bounded enhancement that respects Parseltongue's data-provider identity.
**Estimated LoC**: ~200 Rust (model config lookup + parameter parsing)
**Effort**: Small (2-3 days) -- trivial if `tiktoken-rs` is already added for Feature 1.4
**Risk**: Very low -- static config, no external API calls

---

## 4. CodeCompanion.nvim

### Feature 4.1: ACP Adapter Pattern (Agent Client Protocol)

**ELI5**: CodeCompanion.nvim can talk to both traditional HTTP-based LLM APIs and ACP-based agent tools (like Claude Code, Gemini CLI, Auggie CLI, Codex). The adapter factory auto-detects whether a configured adapter is HTTP or ACP type, and resolves it accordingly. ACP adapters have protocol methods for authentication, session management, and prompting that differ from simple HTTP request/response.

**Key Code Path**:
```
lua/codecompanion/adapters/init.lua:8 -> adapter_type(adapter) determines "http" or "acp"
  -> init.lua:21-26 -> checks config.adapters.acp[name] vs config.adapters.http[name]
  -> init.lua:37 -> M.resolve(adapter, opts) factory method
  -> init.lua:38-41 -> if acp: require("codecompanion.adapters.acp").resolve()
                        else: require("codecompanion.adapters.http").resolve()
  -> adapters/acp/init.lua:5 -> CodeCompanion.ACPAdapter class definition
  -> acp/init.lua:19-29 -> protocol interface: authenticate, new_session, load_session, prompt, agent_state, session_update
  -> acp/init.lua:35 -> Adapter.new(args) with setmetatable
  -> acp/init.lua:44 -> Adapter.extend(adapter, opts) loads from config or module
  -> acp/init.lua:78 -> Adapter.resolve(adapter, opts) resolves from string/table/function
  -> acp/init.lua:82-86 -> handles { name = "claude_code", model = "opus" } style config
```

**What the code actually does**: The adapter system uses a factory pattern with type detection. `adapter_type()` checks the adapter's `type` field, then looks up the name in `config.adapters.acp` and `config.adapters.http` to determine the type. The `CodeCompanion.ACPAdapter` class has a `protocol` table with methods: `authenticate` (connect to the agent), `new_session` (start a fresh conversation), `load_session` (resume a saved one), `prompt` (send messages to the agent), `agent_state` (TBD), and `session_update` (TBD). The `resolve()` method handles multiple input formats: a string name, a table like `{ name = "claude_code", model = "opus" }`, or a function. `extend()` merges user options with adapter defaults using `vim.tbl_deep_extend("force", ...)`. Backward compatibility with HTTP adapters is maintained via `set_model()`, `get_handler()`, and `call_handler()` forwarding methods.

**Parseltongue comparison**:
- **Entities indexed**: 0 Lua entities -- CodeCompanion.nvim is written in Lua, which Parseltongue does not parse. The only indexed entities are test stubs: `go:struct:ExampleStruct` (Go stub), `python:class:ExampleClass` (Python stub), and `python:method:compute` (Python stub) in `codecompanion.nvim/tests/stubs/stub.*`. Total: 5 entities from test fixtures.
- **Coverage gap**: Lua is not in Parseltongue's supported language list (Rust, Python, JS, TS, Go, Java, C, C++, Ruby, PHP, C#, Swift). This means CodeCompanion.nvim's entire codebase (adapters, tools, slash commands, orchestrator) is invisible to Parseltongue graph analysis.
- **Parseltongue has equivalent?**: No -- Parseltongue has no adapter/protocol abstraction layer. The ACP (Agent Client Protocol) adapter pattern is a client-side concern for tools that integrate with multiple LLM providers. Parseltongue is a data provider, not an LLM client. However, if Parseltongue adds MCP server mode, an adapter pattern could be useful for supporting both stdio and HTTP transports.
**Implementation spec**:
**Feature name**: NOT RECOMMENDED as a standalone feature -- covered by pt09 MCP bridge
**Rationale**: The ACP (Agent Client Protocol) adapter pattern is a client-side abstraction for tools that talk to multiple LLM backends. Parseltongue does not talk to LLMs -- it is talked to by LLMs. The adapter pattern is irrelevant.
**What IS relevant**: If pt09 (MCP bridge) is built, CodeCompanion.nvim could consume Parseltongue as an MCP server via its existing ACP adapter system. No work needed on Parseltongue's side -- CodeCompanion already supports arbitrary MCP servers.
**Lua language gap**: CodeCompanion.nvim's 0 indexed Lua entities is a real coverage gap. Adding `tree-sitter-lua` to `parseltongue-core` (as noted in Feature 2.2 spec) would enable Parseltongue to analyze CodeCompanion.nvim itself. This is a marketing opportunity: "Parseltongue can analyze the tools that use it."
**Effort**: N/A (not recommended) / Small (3-5 days for Lua tree-sitter support)

---

### Feature 4.2: Slash Commands (Context Injection)

**ELI5**: In CodeCompanion's chat buffer, users can type `/buffer`, `/file`, `/symbols`, etc. to inject context from their Neovim environment into the conversation. The slash command system resolves callbacks to Lua modules, supports both built-in and user-defined commands, and can load commands from file paths, module paths, or inline functions. Each command can have a provider (e.g., telescope, fzf) for interactive selection.

**Key Code Path**:
```
lua/codecompanion/interactions/chat/slash_commands/init.lua:8 -> resolve(callback) loads module
  -> init.lua:9 -> pcall(require, "codecompanion." .. callback) for built-in
  -> init.lua:16 -> pcall(require, callback) for user module path
  -> init.lua:24 -> loadfile(vim.fs.normalize(callback)) for file path
  -> init.lua:36 -> SlashCommands class
  -> init.lua:47 -> set_provider(SlashCommand, providers) selects UI provider
  -> init.lua:64 -> execute(item, chat) runs the selected slash command
  -> init.lua:69 -> if callback is function: call it directly
  -> init.lua:73 -> else: resolve module, check enabled(), create .new(), call :execute()
  -> init.lua:99 -> SlashCommands.context(chat, slash_command, opts) for external context injection
  -> slash_commands/builtin/ -> /buffer, /file, /fetch, /symbols, /help, /image, /quickfix, /terminal, /mode, /memory, /now
```

**What the code actually does**: `SlashCommands` uses a 3-layer resolution strategy for callbacks: (1) try `require("codecompanion." .. callback)` for built-in commands, (2) try `require(callback)` for user-provided module paths, (3) try `loadfile(vim.fs.normalize(callback))` for file paths. The `execute()` method first checks if the callback is a raw function (inline config), otherwise resolves the module, checks if the command is `enabled()` (e.g., some commands require LSP), creates a new instance with `Chat`, `config`, and `context`, and calls `:execute(self)`. The `set_provider()` method allows slash commands to use different UI backends (telescope, fzf, default). The `context()` static method allows external objects to inject context programmatically. Built-in commands include `/buffer` (current buffer content), `/file` (pick a file), `/symbols` (LSP symbols), `/fetch` (web page), `/image` (vision), `/terminal` (terminal output), and `/memory` (persistent notes).

**Parseltongue comparison**:
- **Entities indexed**: 0 Lua entities (CodeCompanion.nvim Lua code not parsed -- see Feature 4.1 note)
- **Parseltongue has equivalent?**: Partial conceptual overlap -- CodeCompanion's slash commands inject context (buffer content, file content, LSP symbols, web pages) into LLM conversations. Parseltongue's HTTP endpoints serve a similar "context injection" role: `/code-entities-search-fuzzy` provides symbol search, `/code-entity-detail-view/{key}` provides entity detail, `/smart-context-token-budget` provides budget-aware context. The difference is the injection mechanism: CodeCompanion injects into a chat buffer inside Neovim; Parseltongue returns JSON over HTTP for any client to consume. If Parseltongue adds MCP server mode, its endpoints could be exposed as MCP resources that slash command systems like CodeCompanion's could consume natively.
**Implementation spec**:
**Feature name**: MCP resource exposure (covered by pt09 MCP bridge)
**Where**: pt09 MCP bridge crate -- expose Parseltongue endpoints as both MCP tools AND MCP resources
**Architecture**: MCP distinguishes between "tools" (actions the LLM invokes) and "resources" (data the client can browse). Parseltongue's endpoints map naturally:
  - **MCP Tools**: Action-oriented endpoints (`blast_radius_impact_analysis`, `code_entities_search_fuzzy`, `circular_dependency_detection_scan`)
  - **MCP Resources**: Data-browsing endpoints (`codebase_statistics_overview_summary`, `code_entities_list_all`, `dependency_edges_list_all`)
  - **MCP Resource Templates**: Parameterized lookups (`code_entity_detail_view/{key}`, `coupling_cohesion_metrics_suite?entity={key}`)
**How CodeCompanion would consume this**: CodeCompanion's `/symbols` slash command could be backed by PT's `code_entities_search_fuzzy` MCP tool. The `/file` slash command could use PT's `code_entity_detail_view` for entity-level context instead of raw file content.
**Estimated LoC**: Included in pt09 estimate -- resource registration is ~200 additional lines on top of tool registration
**Effort**: Included in Feature 1.1 (2-3 weeks total)

---

### Feature 4.3: Tool System with Orchestrator

**ELI5**: When an LLM returns tool calls in its response, CodeCompanion's tool system extracts them, validates them against the registered tools, handles approval flows (ask user before dangerous operations), executes them, and sends results back. The orchestrator manages the lifecycle of tool execution including parallel execution, error handling, and virtual text status indicators in the chat buffer.

**Key Code Path**:
```
lua/codecompanion/interactions/chat/tools/init.lua:1 -> CodeCompanion.Tools class
  -> init.lua:17 -> Orchestrator = require("...tools.orchestrator")
  -> init.lua:18 -> approvals = require("...tools.approvals")
  -> init.lua:19 -> tool_filter = require("...tools.filter")
  -> init.lua:31 -> CONSTANTS { PREFIX = "@", STATUS_ERROR, STATUS_SUCCESS, PROCESSING_MSG }
  -> init.lua:51 -> _pattern(tool) creates "@{tool_name}" regex pattern
  -> init.lua:59 -> _handle_tool_error(tool, error_message) sends error back to LLM
  -> init.lua:95 -> _resolve_and_prepare_tool(tool, id) validates and loads tool module
  -> init.lua:97 -> tools_config[name] lookup from registered tools
  -> tools/orchestrator.lua -> manages execution lifecycle
  -> tools/approvals.lua -> user confirmation for dangerous tools
  -> tools/filter.lua -> filters tools based on adapter capabilities
  -> tools/builtin/ -> read_file, create_file, delete_file, insert_edit_into_file, cmd_runner, grep_search, file_search, etc.
  -> tools/runtime/ -> runtime execution environment
```

**What the code actually does**: The `Tools` class is the main coordinator. When the LLM returns a response with tool calls, `_resolve_and_prepare_tool()` looks up each tool name in `tools_config` (populated from the tool registry). It supports "hybrid tools" where an adapter's native tool is paired with a CodeCompanion client-side tool. The `Orchestrator` manages the execution pipeline: extract tool calls from the response, validate arguments, check approval requirements via `approvals.lua`, execute tools (potentially in parallel), collect results, and send them back to the LLM. The `filter.lua` module ensures only tools compatible with the current adapter are exposed. Built-in tools include file operations (`read_file`, `create_file`, `delete_file`, `insert_edit_into_file`), code analysis (`list_code_usages` via LSP, `grep_search`, `file_search`), execution (`cmd_runner`), and web (`web_search`, `fetch_webpage`). Virtual text status indicators (`PROCESSING_MSG`) appear in the chat buffer during execution.

**Parseltongue comparison**:
- **Entities indexed**: 0 Lua entities (CodeCompanion.nvim Lua code not parsed -- see Feature 4.1 note)
- **Parseltongue has equivalent?**: No -- Parseltongue has no tool execution orchestrator. It does not extract tool calls from LLM responses, validate them, manage approval flows, or execute tools in parallel. The orchestrator pattern is a client-side concern for LLM-integrated editors. Parseltongue is purely a data provider. However, the built-in tool patterns (read_file, create_file, grep_search, file_search, cmd_runner) overlap with Parseltongue's analysis capabilities -- an LLM using CodeCompanion could call Parseltongue's HTTP endpoints via `cmd_runner` (curl) as a workaround until native MCP integration exists.
**Implementation spec**:
**Feature name**: NOT RECOMMENDED for implementation
**Rationale**: The tool orchestrator pattern (extract tool calls from LLM responses, validate, approve, execute, return results) is a client-side concern. Parseltongue is on the server side of the tool call -- it receives requests and returns data. The orchestrator lives in the client (CodeCompanion, Claude Desktop, Cursor, Gemini CLI).
**What pt09 enables**: When pt09 (MCP bridge) is built, tool orchestrators like CodeCompanion's `Orchestrator` and Gemini CLI's `Scheduler` will be able to call Parseltongue tools natively. Parseltongue does not need to replicate the orchestrator -- it needs to be callable BY orchestrators.
**Built-in tool overlap**: CodeCompanion's `grep_search` and `file_search` built-in tools overlap with Parseltongue's `/code-entities-search-fuzzy`. An LLM using CodeCompanion could get better results by calling PT's fuzzy search (which searches a pre-built graph index) instead of CodeCompanion's grep (which does linear text search). This is a positioning opportunity for pt09.
**Effort**: N/A (not recommended)

---

### Feature 4.4: Multi-Adapter LLM Support

**ELI5**: CodeCompanion can talk to over 15 different LLM providers through a unified adapter interface. HTTP adapters handle traditional API-based providers (Anthropic, OpenAI, Ollama, Gemini, etc.), while ACP adapters handle agent-based tools (Claude Code, Gemini CLI). The factory pattern auto-detects the adapter type and resolves it with user-specified overrides for model selection.

**Key Code Path**:
```
lua/codecompanion/adapters/init.lua:37 -> M.resolve(adapter, opts) dispatches to acp or http
  -> init.lua:62 -> M.extend(adapter, opts) merges user config
  -> init.lua:82 -> M.set_model(args) switches model on an existing adapter
  -> adapters/http/ -> Anthropic, OpenAI, Copilot, Ollama, Gemini, Mistral, Deepseek, Azure OpenAI, GitHub Models, HuggingFace, XAI, Jina, Tavily, Novita, OpenAI-compatible
  -> adapters/acp/ -> Claude Code, Auggie CLI, Codex, Gemini CLI
  -> adapters/shared.lua -> map_roles() shared between http and acp adapters
```

**What the code actually does**: The adapter system provides a uniform interface for all LLM providers. The root `init.lua` acts as a factory that dispatches to `adapters/http` or `adapters/acp` based on `adapter_type()` detection. Both adapter types implement `resolve()`, `extend()`, `resolved()`, `make_safe()`, and `set_model()`. HTTP adapters define request parameters, headers, and response parsing handlers specific to each provider's API format. ACP adapters define command execution, protocol methods, and session management. `shared.lua` provides common utilities like `map_roles()` that translates CodeCompanion's role names to each provider's expected format. The `make_safe()` method strips functions and non-serializable data for safe persistence.

**Parseltongue comparison**:
- **Entities indexed**: 0 Lua entities (CodeCompanion.nvim Lua code not parsed -- see Feature 4.1 note)
- **Adapter count comparison**: CodeCompanion supports 15+ HTTP providers (Anthropic, OpenAI, Copilot, Ollama, Gemini, Mistral, Deepseek, Azure OpenAI, GitHub Models, HuggingFace, XAI, Jina, Tavily, Novita, OpenAI-compatible) + 4 ACP agents (Claude Code, Auggie CLI, Codex, Gemini CLI). Parseltongue supports 0 LLM providers -- it is model-agnostic by design, producing structured data that any LLM can consume via HTTP.
- **Parseltongue has equivalent?**: No, and intentionally not -- Parseltongue's value proposition is "model-agnostic code analysis data." Multi-adapter support is a consumer concern. The 99% token reduction (2-5K tokens vs. 500K raw dumps) means Parseltongue's output fits within any model's context window without model-specific optimizations. The adapter pattern becomes relevant only if Parseltongue adds an LLM-powered natural language query interface on top of its graph database.
**Implementation spec**:
**Feature name**: NOT RECOMMENDED for implementation
**Rationale**: Multi-adapter LLM support is the polar opposite of Parseltongue's value proposition. Parseltongue's power is being model-agnostic: it produces structured graph data (JSON) that any LLM can consume via HTTP or MCP. Adding adapter-specific formatting (Anthropic message format vs. OpenAI chat format vs. Gemini content format) would:
  1. Create coupling to rapidly-changing API formats
  2. Dilute the clean data-provider architecture
  3. Duplicate work already done by 15+ clients (CodeCompanion, Claude Desktop, Cursor, Continue, etc.)
**The 99% token reduction works for ALL models** -- Parseltongue's 50-token stats summary fits in GPT-4's 128K window AND Gemini's 1M window equally well. No per-model optimization needed.
**Effort**: N/A (not recommended -- intentionally excluded)

---

## 5. Notable Grep Servers

### Feature 5.1: ast-grep-mcp -- AST Structural Search

**ELI5**: ast-grep-mcp wraps the `ast-grep` CLI tool as an MCP server. Unlike regular grep which matches text, ast-grep matches code structure. You can write patterns like `class $NAME` to find all class definitions, or complex YAML rules to find nested patterns. It supports 25+ programming languages and returns results in either human-readable text or JSON format.

**Key Code Path**:
```
main.py:97 -> mcp = FastMCP("ast-grep")
  -> main.py:102 -> register_mcp_tools() defines 4 tools
  -> main.py:104 -> @mcp.tool() dump_syntax_tree(code, language, format)
     -> calls: ast-grep run --pattern <code> --lang <lang> --debug-query=<format>
  -> main.py:122 -> @mcp.tool() test_match_code_rule(code, yaml)
     -> calls: ast-grep scan --inline-rules <yaml> --json --stdin
  -> main.py:139 -> @mcp.tool() find_code(project_folder, pattern, language, max_results, output_format)
     -> calls: ast-grep run --pattern <pattern> --json <folder>
  -> main.py:207 -> @mcp.tool() find_code_by_rule(project_folder, yaml, max_results, output_format)
     -> calls: ast-grep scan --inline-rules <yaml> --json <folder>
  -> main.py:271 -> format_matches_as_text() converts JSON to file:start-end headers + match text
  -> main.py:298 -> get_supported_languages() returns 25+ languages + custom from sgconfig.yaml
  -> main.py:390 -> run_ast_grep(command, args) adds --config flag if CONFIG_PATH set
  -> main.py:17 -> SIGINT handler ignores for multi-session stability
```

**What the code actually does**: The server exposes 4 tools: `dump_syntax_tree` for inspecting code structure (CST, AST, or pattern format), `test_match_code_rule` for testing YAML rules against code snippets, `find_code` for pattern-based project search, and `find_code_by_rule` for complex YAML rule-based search. All tools delegate to `run_ast_grep()` which wraps `subprocess.run()` with the `ast-grep` CLI. The `format_matches_as_text()` function converts JSON matches to a compact `file:start-end\nmatch_text` format optimized for LLM consumption. Custom languages from `sgconfig.yaml` are auto-detected and added to the supported list. A SIGINT handler ignores interrupts for multi-session stability (Claude Code sends SIGINT when new sessions start). The server supports both stdio and SSE transports.

**Parseltongue comparison**:
- **Entities indexed**: Only 4 test fixture entities from `ast-grep-mcp/tests/fixtures/example.py` (`Calculator` class, `add` function, `hello` function, `multiply` method). The main `main.py` (which defines all 4 MCP tools) was not parsed -- its functions appear only as unresolved references (`dump_syntax_tree`, `find_code`, `find_code_by_rule`, `format_matches_as_text`, `run_ast_grep`)
- **Coverage gap**: ast-grep-mcp's `main.py` likely uses Python shebang or import patterns that triggered parse failures (documented in CR-v173-01 ingestion error analysis)
- **Parseltongue has equivalent?**: Partial overlap -- ast-grep searches code by AST structure (e.g., "find all class definitions matching pattern X"). Parseltongue's `/code-entities-search-fuzzy` searches by entity name, and `/code-entities-list-all` returns all parsed entities. The key difference: ast-grep matches structural patterns in source code (works on raw AST); Parseltongue matches entities in a pre-built graph database (works on indexed metadata). ast-grep can find arbitrary patterns ("find all functions with more than 5 parameters"); Parseltongue can find entity relationships ("find all callers of function X within 3 hops"). These are complementary, not competing capabilities. A Parseltongue + ast-grep combination would provide both structural search and graph analysis.
**Implementation spec**:
**Feature name**: `structural-pattern-search-query` (new endpoint in pt08, NOT a new crate)
**Where**: `pt08-http-code-query-server/src/http_endpoint_handler_modules/structural_pattern_search_handler.rs`
**Architecture**: Complement ast-grep, do not replace it. ast-grep excels at source-level structural patterns ("find all functions with >5 params"). Parseltongue excels at graph-level structural patterns ("find all entities with CBO >50 that are in a cycle"). Add an endpoint that combines both:
  - Accept a graph pattern (Datalog-expressible) and return matching entities
  - Accept an entity filter from graph query + delegate to ast-grep for source-level refinement (via shelling out to `sg` CLI if installed)
**CozoDB queries needed**: New Datalog patterns for structural search:
  - "All functions called by >10 callers": `?[entity] := *DependencyEdges{to_key: entity}, count > 10`
  - "All classes in cycles": `?[entity] := *SCC{members: ms}, entity in ms, len(ms) > 1`
  - "Hub entities (high fan-in + high fan-out)": combine forward + reverse edge counts
**New HTTP endpoint**:
  - `/structural-pattern-search-query?pattern=high_coupling&threshold=50` (4-word: structural-pattern-search-query)
**Key dependencies**: None new for graph patterns. Optional: `ast-grep` CLI detection via `which sg` for source-level refinement.
**Estimated LoC**: ~600 Rust (predefined pattern library + CozoDB query execution + optional ast-grep delegation)
**Effort**: Small-Medium (1-2 weeks) -- patterns are Datalog queries; the handler is thin
**Risk**: Low -- CozoDB Datalog is already proven for graph queries. ast-grep integration is optional and gracefully degrades if CLI not found.

---

### Feature 5.2: kp-ripgrep-mcp -- Obsidian-Aware Ripgrep

**ELI5**: kp-ripgrep-mcp is a specialized ripgrep wrapper designed for searching Obsidian vaults (Markdown-based knowledge bases). It understands Obsidian's frontmatter (YAML metadata), wiki-style links (`[[page]]`), and document structure (headings). You can search content only, frontmatter only, or both. Results include "smart context" -- automatically extracted headings and frontmatter properties around each match.

**Key Code Path**:
```
src/rgrep_mcp/server.py:9 -> mcp = FastMCP("rgrep-mcp")
  -> server.py:16 -> Config() loads from environment (OBSIDIAN_VAULT_PATH)
  -> server.py:19 -> RipgrepWrapper(config.vault_path) wraps rg CLI
  -> server.py:33 -> @mcp.tool() rg_search_notes(query, search_scope, case_sensitive, folder, max_results, smart_context)
     -> search_scope: "all" | "content_only" | "frontmatter_only"
     -> smart_context: includes frontmatter property names and content headings
  -> server.py:111 -> @mcp.tool() rg_search_links(link_type, url_pattern, title_pattern, folder, max_results)
     -> extracts wiki links, markdown links, external links
  -> ripgrep.py -> RipgrepWrapper class
     -> search_content(), search_content_only(), search_frontmatter_only()
     -> smart context extraction (headings, frontmatter keys)
  -> config.py -> Config class with OBSIDIAN_VAULT_PATH
```

**What the code actually does**: The server initializes by loading the `OBSIDIAN_VAULT_PATH` environment variable and creating a `RipgrepWrapper` instance that validates the vault exists and ripgrep works. `rg_search_notes()` supports 3 search scopes: `all` (full file), `content_only` (skip YAML frontmatter), and `frontmatter_only` (only YAML metadata). When `smart_context=True`, each match result includes extracted headings (section context) and frontmatter property names around the match. `rg_search_links()` extracts and filters wiki links (`[[page]]`), markdown links (`[text](url)`), and external URLs. Results are formatted as JSON with file path, line number, snippet, and smart context -- optimized for LLM consumption. The `max_results` parameter caps at 100 and is applied after ripgrep's per-file `--max-count` limit.

**Parseltongue comparison**:
- **File entity**: `python:file:__mcp-grep-servers_kp-ripgrep-mcp_src_rgrep_mcp_ripgrep_py:1-1`
- **CBO**: 76 (grade F, 14h tech debt -- HIGH_COUPLING, LOW_COHESION, HIGH_COMPLEXITY violations)
- **Forward callees**: 76 entities including Python stdlib (CalledProcessError, SubprocessError, JSONDecodeError, YAMLError, DOTALL, IGNORECASE), internal methods (`_add_smart_context`, `_build_rg_command`, `_filter_content_results`, `_filter_frontmatter_results`, `_get_content_heading_context`)
- **K-core**: coreness=24 (same densest core as code-scalpel's security modules -- surprising for a "thin wrapper" tool. This is because its 76 callees share Python builtins with the entire Python sub-graph, pulling it into the dense core)
- **Reverse callers**: 7 (all test files -- `test_max_results`, `test_path`, `test_recent_notes_fixes`, `test_scope_issues`, `test_smart_context`, `test_unicode`, `test_search`)
- **Class entity**: `python:class:RipgrepWrapper:____mcp_grep_servers_kp_ripgrep_mcp_src_rgrep_mcp_ripgrep:T1807368672`
- **Parseltongue has equivalent?**: No direct equivalent -- kp-ripgrep-mcp is specialized for Obsidian vault search (frontmatter awareness, wiki link extraction, heading context). Parseltongue's `/code-entities-search-fuzzy` searches code entities, not Markdown documents. The "smart context" concept (automatically including headings and frontmatter properties around matches) is conceptually similar to Parseltongue's graph context (including caller/callee relationships around a matched entity). Both tools solve "give the LLM relevant context around a match" but for different content types (Markdown vs. code graphs).
**Implementation spec**:
**Feature name**: NOT RECOMMENDED for implementation -- wrong domain
**Rationale**: kp-ripgrep-mcp is specialized for Obsidian Markdown vault search (frontmatter awareness, wiki link extraction, heading context). Parseltongue is a code analysis tool, not a knowledge base search tool. There is zero overlap in target use case.
**Conceptual takeaway**: The "smart context" concept (automatically including headings and frontmatter properties around matches) has a parallel in Parseltongue's graph context. Parseltongue's `/blast-radius-impact-analysis` already provides "structural smart context" -- entities related to a match within N hops. This is strictly more powerful than text-based heading extraction for code analysis use cases.
**If documentation search is ever needed**: Parseltongue could add a `DocGraph` relation alongside `CodeGraph` for indexing Markdown/RST documentation files with heading structure. But this is a different product direction and competes with tools like Obsidian, Notion, and dedicated knowledge management tools.
**Effort**: N/A (not recommended -- out of scope)

---

### Feature 5.3: mcp-server-semgrep -- SARIF Export + Scan Comparison

**ELI5**: mcp-server-semgrep wraps Semgrep (a static analysis tool) as an MCP server. It can scan directories for security vulnerabilities, create custom rules, filter and analyze results, export to SARIF format (a standard for static analysis results), and compare two scan results to see what changed. It auto-installs Semgrep via pip if not found, and supports Semgrep Pro rules with API tokens.

**Key Code Path**:
```
src/index.ts:22 -> class SemgrepServer
  -> index.ts:26 -> new Server({ name: 'mcp-server-semgrep', version: '1.0.0' })
  -> index.ts:116 -> setupToolHandlers() registers 7 tools:
     1. scan_directory(path, config) -> semgrep scan --json
     2. list_rules(language) -> available rule collections
     3. analyze_results(results_file) -> analyze JSON results
     4. create_rule(output_path, pattern, language, message, severity, id) -> YAML rule file
     5. filter_results(results_file, severity, rule_id, path_pattern, language, message_pattern) -> filtered results
     6. export_results(results_file, output_file, format: json|sarif|text) -> export in multiple formats
     7. compare_results(old_results, new_results) -> diff two scan results
  -> index.ts:47 -> checkSemgrepInstallation() tests 'semgrep --version'
  -> index.ts:56 -> installSemgrep() via pip3 if not found
  -> index.ts:81 -> validateAbsolutePath() with path traversal prevention
  -> index.ts:324 -> handleScanDirectory() supports SEMGREP_APP_TOKEN for Pro rules
  -> index.ts:259 -> compare_results tool diffs old vs new findings
```

**What the code actually does**: `SemgrepServer` creates an MCP `Server` and registers 7 tools via `setupToolHandlers()`. The `scan_directory` tool runs `semgrep scan --json` on a directory with configurable rulesets (e.g., `auto`, `p/security`, `p/ci`). It supports Pro rules via `SEMGREP_APP_TOKEN` environment variable. The `create_rule` tool generates YAML rule files with pattern, language, message, severity, and ID fields. The `filter_results` tool applies multi-criteria filtering (severity, rule_id, path regex, language, message regex) to existing scan results. The `export_results` tool converts JSON results to SARIF (Static Analysis Results Interchange Format) or plain text. The `compare_results` tool diffs two scan result files to show new, fixed, and unchanged findings. Path validation prevents directory traversal attacks by normalizing paths and checking they fall within the allowed base directory. The server auto-installs Semgrep via `pip3` if not found.

**Parseltongue comparison**:
- **File entity**: `python:file:__mcp-grep-servers_semgrep-mcp_src_semgrep_mcp_semgrep_py:1-1`
- **CBO**: 64 (grade F) -- the SemgrepContext class file calls 64 unique entities
- **Class entity**: `python:class:SemgrepContext:____mcp_grep_servers_semgrep_mcp_src_semgrep_mcp_semgrep:T1698802211`
- **Entities indexed**: 105 semgrep-related entities across both the semgrep-mcp wrapper AND code-scalpel's `SemgrepParser`/`SemgrepFinding` classes (code-scalpel also integrates semgrep for Java analysis). semgrep-mcp models include: `SemgrepScanResult`, `Finding`, `Rule`, `CodeWithLanguage`, `CodeFile`, `SemgrepContext`, `Location`, `Repository`, `Component`, `Guidance`, `Autofix`, `Autotriage`, `ReviewComment`, `SourcingPolicy`, `ExternalTicket`, `Assistant`
- **mcp-server-semgrep note**: The TypeScript version (`src/index.ts` with `SemgrepServer` class) was NOT parsed -- `SemgrepServer` returned 0 results. Only the Python semgrep-mcp wrapper was indexed.
- **Parseltongue has equivalent?**: No -- Parseltongue has no SAST (Static Application Security Testing) capabilities, no SARIF export, no scan comparison, no custom rule creation. Semgrep finds security vulnerabilities using pattern matching; Parseltongue finds structural relationships using graph algorithms. These are complementary: semgrep finds "this code pattern is dangerous" while Parseltongue finds "this entity has 483 dependencies and a blast radius of N hops." A combined approach (semgrep findings as node annotations in Parseltongue's graph) would enable queries like "show me all entities with CWE-89 findings that are reachable from user-facing APIs within 3 hops."
**Implementation spec**:
**Feature name**: `sarif-export-analysis-formatter` (new module in parseltongue-core) + `semgrep-annotation-graph-overlay` (optional)
**Where**: `parseltongue-core/src/export/sarif_formatter.rs` + optional annotation overlay in pt08
**Architecture**: Two distinct capabilities:
  1. **SARIF Export** (recommended): Export Parseltongue's analysis results (tech debt violations, circular dependencies, coupling hotspots, SCC cycles) in SARIF format (Static Analysis Results Interchange Format v2.1.0). This enables integration with GitHub Code Scanning, VS Code SARIF Viewer, and CI/CD pipelines.
  2. **Semgrep Annotation Overlay** (optional): If semgrep is installed, run `semgrep scan --json`, parse results, and store findings as annotations on graph entities. This enables queries like "show all entities with CWE-89 findings AND CBO > 50 AND blast radius > 20 hops" -- a query no other tool can answer.
**CozoDB tables needed** (for annotation overlay):
  - New relation: `SecurityFindings { finding_id: String, entity_key: String => tool: String, rule_id: String, cwe_id: String?, severity: String, message: String, file_path: String, line: Int }`
**New HTTP endpoints**:
  - `/sarif-export-analysis-formatter?categories=debt,cycles,coupling` (4-word: sarif-export-analysis-formatter) -- exports selected analysis categories as SARIF JSON
  - `/security-findings-annotation-overlay?entity=X` (4-word: security-findings-annotation-overlay) -- returns semgrep findings annotated on graph entities
**Key dependencies**: `serde_json` (already present) for SARIF schema serialization. Optional: semgrep CLI detection.
**Estimated LoC**: ~1,000 Rust (SARIF schema types + serialization) + ~500 (semgrep integration + CozoDB annotation storage)
**Effort**: Medium (2-3 weeks for SARIF export) + Medium (2-3 weeks for semgrep overlay, if desired)
**Risk**: Low for SARIF export (well-defined JSON schema). Medium for semgrep overlay (depends on semgrep CLI availability + output format stability).

---

---

## Final Summary Table

| # | Competitor | Feature | ELI5 (1-liner) | PT Has? | Effort | Shreyas: User Segment | Shreyas: Differentiation |
|---|-----------|---------|----------------|---------|--------|----------------------|------------------------|
| 1.1 | code-scalpel | MCP Server + Tiered Licensing | AI tools connect to code analysis via MCP protocol with license tiers | Partial (HTTP only) | Medium (2-3w) | Solo devs using Claude/Cursor for code understanding | **Leverage** -- 10x moat: only graph DB offers blast-radius-aware MCP tools. **Painkiller** (MCP is the new REST for AI tools). **Defensible** (graph queries cannot be replicated by flat-file MCP servers). |
| 1.2 | code-scalpel | Taint Analysis | Track dirty data from user input to dangerous operations, flag vulnerabilities | No | Large (6-8w) | Enterprise AppSec teams running SAST pipelines | **Leverage** -- 10x moat: graph-based taint tracking via DependencyEdges is unique. No other tool combines blast-radius with taint flow. **Painkiller** (security vulns are urgent). **Defensible** (requires deep graph infra + tree-sitter query authoring). |
| 1.3 | code-scalpel | Z3 Symbolic Execution | Explore every code path without running code, generate test inputs automatically | No | Very Large (12-16w) | Enterprise testing/verification teams at banks, aerospace | **Overhead** -- code-scalpel has unassailable 5K LoC lead with 3 entry points. Reimplementing Z3 in Rust is pure overhead. **Vitamin** (nice for formal verification, not urgent for most teams). **Commoditized** (Z3 is the same engine regardless of wrapper). |
| 1.4 | code-scalpel | Surgical Code Extraction | Extract one function with deps, count tokens, fit within LLM budget | Partial (smart-context) | Medium (2-3w) | Solo devs using Claude/Cursor for code understanding | **Leverage** -- existing smart-context endpoint + graph-aware extraction = unique value. Extract function X + all callees within token budget. **Painkiller** (token costs are real). **Defensible** (graph-based dependency resolution is the moat). |
| 1.5 | code-scalpel | Policy Engine (OPA/Rego) | Enforce coding standards with Rego rules, cryptographic integrity checks, compliance | No | Medium (3-4w) | Platform eng at Series B+ standardizing tooling | **Leverage** -- Datalog-native policies over graph are simpler AND more powerful than OPA for structural rules. **Painkiller** (compliance is mandatory for enterprise). **Defensible** (CozoDB Datalog policies are unique). |
| 2.1 | AiDex | SQLite Code Index | SQLite database storing every identifier, function, class for fast LLM queries | Yes (CozoDB) | None | Solo devs wanting fast code search | **Neutral** -- table stakes. PT's CozoDB is strictly superior (graph algorithms impossible in SQLite). **Vitamin** (incremental improvement over grep). **Defensible** (graph DB vs. relational DB is an architectural moat). |
| 2.2 | AiDex | Tree-sitter Parsing | Parse 11 languages to understand code structure for indexing | Yes (12 langs) | None | Open source maintainers reviewing PRs across polyglot repos | **Neutral** -- table stakes. Every code analysis tool uses tree-sitter. PT already wins (12 > 11 languages, Rust native > Node.js bindings). **Vitamin**. **Commoditized** (tree-sitter grammars are shared). |
| 2.3 | AiDex | Incremental Re-indexing | Skip unchanged files via SHA256 hash comparison, respect .gitignore | Yes (file watcher) | None | Devs with large monorepos (>10K files) | **Neutral** -- table stakes. PT's event-driven file watcher is architecturally superior to batch hash comparison. **Vitamin**. **Commoditized** (filesystem watching is a solved problem). |
| 2.4 | AiDex | Session Management | Track coding sessions, detect external changes, auto-reindex on return | No | Small (1-2w) | Solo devs using Parseltongue daily in IDE | **Neutral** -- nice UX, not differentiating. PT's file watcher partially substitutes. Session tracking matters for Tauri desktop app. **Vitamin** (convenience, not critical). **Commoditized** (timestamp comparison is trivial). |
| 2.5 | AiDex | MCP Server (22 tools) | AI assistants call 22 tools via MCP for search, signatures, project overview | Parallel (22 HTTP) | Medium (2-3w) | Solo devs using Claude/Cursor for code understanding | **Leverage** -- same as Feature 1.1. PT's 24 endpoints become 24 MCP tools with graph analysis capabilities no other MCP server has. **Painkiller**. **Defensible**. |
| 3.1 | Gemini CLI | Event-Driven Scheduler | Manage tool execution with queuing, cancellation, confirmation, policy checks | No | N/A | Platform eng building internal AI agent infra | **Overhead** -- client-side orchestration pattern irrelevant to data providers. MCP SDK handles tool call lifecycle. **Vitamin**. **Commoditized** (every agent framework has a scheduler). |
| 3.2 | Gemini CLI | MCP Client Manager | Connect to multiple MCP servers with auth, discovery, extension lifecycle | No | N/A | Platform eng at Google-scale enterprises | **Overhead** -- PT is an MCP server, not client. Google's 138 MCP entities are enterprise auth infra. **Vitamin**. **Commoditized** (MCP client SDKs are open source). |
| 3.3 | Gemini CLI | OpenTelemetry System | Traces, metrics, logs to GCP or local files, 15+ event types, memory monitor | No | Small (1-2w) | Platform eng deploying PT in production CI/CD | **Neutral** -- important for production but not differentiating. Use `tracing` crate, not full OTel. **Vitamin** (observability is hygiene, not a selling point). **Commoditized** (tracing is identical across Rust services). |
| 3.4 | Gemini CLI | Multi-Model Config | Auto-route between Flash/Pro models, model availability, context management | No | Small (2-3d) | Solo devs using multiple LLM providers | **Neutral** -- PT is model-agnostic by design. Add model-aware token budgets as lightweight enhancement. **Vitamin**. **Commoditized** (token limit lookup is a config table). |
| 4.1 | CodeCompanion.nvim | ACP Adapter Pattern | Factory auto-detects HTTP vs. ACP adapter type for LLM provider connections | No | N/A | Neovim users with multiple LLM subscriptions | **Overhead** -- client-side LLM routing, irrelevant to PT. PT should be callable BY adapters, not contain them. **Vitamin**. **Commoditized**. |
| 4.2 | CodeCompanion.nvim | Slash Commands | Type /buffer, /file, /symbols to inject context into LLM conversations | Partial (HTTP) | Included in 1.1 | Neovim power users who live in the terminal | **Neutral** -- PT's MCP tools would be consumable by slash commands. No work needed beyond pt09. **Vitamin** (editor integration is nice-to-have). **Commoditized** (any MCP server works). |
| 4.3 | CodeCompanion.nvim | Tool Orchestrator | Extract, validate, approve, execute tool calls from LLM responses | No | N/A | Neovim plugin developers building AI features | **Overhead** -- client-side orchestration, same rationale as Feature 3.1. **Vitamin**. **Commoditized**. |
| 4.4 | CodeCompanion.nvim | Multi-Adapter LLM | 15+ HTTP providers + 4 ACP agents with unified interface | No (intentionally) | N/A | Developers who switch between LLM providers | **Overhead** -- intentionally excluded. Model-agnostic JSON output is the correct strategy. **Vitamin**. **Commoditized** (adapter wrappers are interchangeable). |
| 5.1 | ast-grep-mcp | AST Structural Search | Search code by structure (class definitions, function patterns), not text | Complementary | Small (1-2w) | Devs doing large-scale refactors across codebases | **Leverage** -- combining graph patterns (PT) + AST patterns (ast-grep) is unique. Neither tool alone covers both. **Painkiller** (structural refactoring is painful with grep). **Defensible** (graph + AST integration requires deep architecture). |
| 5.2 | kp-ripgrep-mcp | Obsidian-Aware Ripgrep | Search Markdown knowledge bases with frontmatter awareness and wiki links | No | N/A | Knowledge workers using Obsidian for notes | **Overhead** -- wrong domain entirely. PT analyzes code, not Markdown. **Vitamin**. **Commoditized** (ripgrep wrapper with YAML parsing). |
| 5.3 | mcp-server-semgrep | SARIF Export + Comparison | Scan for security vulnerabilities, export SARIF, compare scan results over time | No | Medium (2-3w each) | Enterprise AppSec teams + CI/CD pipeline operators | **Leverage** -- SARIF export of PT's analysis results enables GitHub Code Scanning integration. Semgrep annotation overlay on graph is unique. **Painkiller** (CI/CD integration is required for adoption). **Defensible** (graph-annotated security findings are novel). |

---

## Priority Matrix (Shreyas Doshi LNO Framework)

### P0 -- Do Now (Leverage: high impact, leverages existing graph architecture)

These features leverage Parseltongue's existing CozoDB graph infrastructure for maximum impact with minimal new architecture. They are painkillers that solve urgent problems and create defensible moats.

| Priority | Feature | Crate/Module | Effort | Why P0 |
|----------|---------|-------------|--------|--------|
| P0-1 | **MCP Protocol Bridge** (1.1 + 2.5) | `pt09-mcp-protocol-bridge-server` | 2-3 weeks | MCP is the new REST for AI tools. Without MCP, Parseltongue is invisible to Claude Desktop, Cursor, Windsurf, and every MCP-enabled IDE. This is an existential gap. All 24 existing HTTP endpoints become MCP tools -- zero new analysis logic needed. The graph analysis tools (PageRank, Leiden, K-core, blast radius) become unique differentiators in the MCP ecosystem where competitors offer only flat search. |
| P0-2 | **Surgical Source Extraction** (1.4) | Extend `smart-context-token-budget` handler | 2-3 weeks | The existing `/smart-context-token-budget` endpoint returns entity metadata but not source code. Adding `--store-source` to pt01 ingestion and `?include_source=true` to the endpoint enables graph-aware code extraction: "extract function X + all its callees within 4K tokens." This directly competes with code-scalpel's SurgicalExtractor (CBO=177, 143 callers) but leverages Parseltongue's dependency graph for dependency-aware extraction. Requires `tiktoken-rs` for accurate token counting. |
| P0-3 | **SARIF Export** (5.3) | `parseltongue-core/src/export/sarif_formatter.rs` | 2-3 weeks | SARIF export enables GitHub Code Scanning integration, VS Code SARIF Viewer, and CI/CD pipeline adoption. Parseltongue already computes tech debt violations, circular dependencies, coupling hotspots, and SCC cycles -- SARIF is just a serialization format for these existing results. Without SARIF, Parseltongue's analysis results are locked inside HTTP JSON responses that CI/CD pipelines cannot consume natively. |

**P0 Total: 6-9 weeks, 3 features, ~3,000 LoC**

### P1 -- Plan Next (Leverage+: high impact, needs new infrastructure)

These features require new CozoDB relations and tree-sitter queries but create deep moats that no competitor can replicate without a graph database.

| Priority | Feature | Crate/Module | Effort | Why P1 |
|----------|---------|-------------|--------|--------|
| P1-1 | **Graph-Native Taint Analysis** (1.2) | `parseltongue-core/src/security/taint_tracker.rs` | 6-8 weeks | Graph-based taint tracking via `DependencyEdges` traversal is Parseltongue's deepest potential moat. No other tool combines blast-radius-aware impact analysis with security vulnerability detection. Requires new `TaintSources`, `TaintSinks`, and `TaintFlows` CozoDB relations + tree-sitter `.scm` patterns for 12 languages. The Z3-less approach (graph reachability instead of symbolic execution) is simpler but uniquely leverages existing infrastructure. code-scalpel's 283 security entities took years to build; PT's graph approach shortcuts this with structural (not semantic) taint tracking. |
| P1-2 | **Datalog Policy Engine** (1.5) | `parseltongue-core/src/policy/` | 3-4 weeks | CozoDB Datalog is a natural policy language for graph structural rules. "No entity shall have CBO > 50" is a one-line Datalog query. "No blast radius shall exceed 100 entities" reuses existing blast-radius traversal. This is simpler AND more powerful than OPA/Rego for structural policies because the graph data is already in CozoDB. Enterprise compliance (SOC2 coupling limits, HIPAA data flow restrictions) maps naturally to graph constraints. |
| P1-3 | **Structural Pattern Search** (5.1) | `pt08/.../structural_pattern_search_handler.rs` | 1-2 weeks | Expose predefined graph patterns ("hub entities", "God classes", "dead code") as a parameterized endpoint. Optional ast-grep CLI delegation for source-level refinement. Complements ast-grep rather than competing -- PT finds entities matching graph patterns, ast-grep finds entities matching AST patterns. The combination is unique. |

**P1 Total: 10-14 weeks, 3 features, ~6,000 LoC**

### P2 -- Explore (Neutral: medium impact, unclear demand)

Useful features that improve developer experience but do not create defensible moats. Implement if Tauri desktop app (v1.7.3 roadmap) needs them.

| Priority | Feature | Crate/Module | Effort | Why P2 |
|----------|---------|-------------|--------|--------|
| P2-1 | **Lightweight Telemetry** (3.3) | `pt08/.../observability_metrics_telemetry_exporter.rs` | 1-2 weeks | Production deployment hygiene. Use `tracing` crate for request latency, error rates, and ingestion throughput. NOT full OpenTelemetry (375 entities is overkill). Export as JSON to stdout or file. Needed for any serious CI/CD integration but not a selling point. |
| P2-2 | **Session Lifecycle Tracking** (2.4) | `pt08/.../session_lifecycle_management_tracker.rs` | 1-2 weeks | Nice UX for Tauri desktop app: "Welcome back, 3 files changed." Simple timestamp-based session detection layered on existing file watcher. Low value as standalone HTTP feature; high value integrated into desktop app. |
| P2-3 | **Model-Aware Token Budgets** (3.4) | Extend `smart-context-token-budget` handler | 2-3 days | Add `?model=claude-sonnet` parameter that adjusts token budget defaults per model family. Trivial if `tiktoken-rs` is already added for P0-2. Incremental improvement, not differentiating. |
| P2-4 | **Lua Language Support** (4.1) | `parseltongue-core` tree-sitter integration | 3-5 days | Add `tree-sitter-lua` grammar to support Neovim plugin analysis (CodeCompanion.nvim, lazy.nvim, etc.). Marketing angle: "Parseltongue can analyze the tools that use it." Low effort, modest demand. |
| P2-5 | **Semgrep Annotation Overlay** (5.3) | `pt08` + new `SecurityFindings` relation | 2-3 weeks | Store semgrep scan results as annotations on graph entities. Enables powerful queries ("CWE-89 findings within blast radius of user-facing APIs") but depends on semgrep CLI availability. Separate from SARIF export (P0-3) which is about exporting PT's own analysis. |

**P2 Total: 7-10 weeks, 5 features, ~3,000 LoC**

### P3 -- Ignore (Overhead: low impact or competitors have unassailable leads)

Do NOT implement these features. They are either out of scope for a data-provider tool, in a domain where competitors have insurmountable leads, or fundamentally misaligned with Parseltongue's architecture.

| Priority | Feature | Rationale for Ignoring |
|----------|---------|----------------------|
| P3-1 | **Z3 Symbolic Execution** (1.3) | code-scalpel has 5K+ LoC across 3 normalizers + IR interpreter + constraint solver with 69 callers across CLI+MCP+CrewAI. Replicating this in Rust requires compiling Z3 from source (30min build, 500MB), custom IR per language, and path exploration engine. 12-16 weeks effort for a feature where code-scalpel has an unassailable lead. Better strategy: delegate to code-scalpel via MCP tool interop from pt09. |
| P3-2 | **Event-Driven Tool Scheduler** (3.1) | Client-side orchestration pattern. Parseltongue receives tool calls, it does not orchestrate them. The MCP SDK (`rmcp`) handles request lifecycle. Building a scheduler would turn PT from a focused data provider into a competing agent framework -- a strategic mistake. |
| P3-3 | **MCP Client Manager** (3.2) | Parseltongue is an MCP server (producer), not an MCP client (consumer). Google's 138 MCP client entities represent enterprise-grade auth/discovery infra that is orthogonal to PT's purpose. Use the MCP SDK's built-in client transport if PT ever needs to delegate to other MCP servers. |
| P3-4 | **ACP Adapter Pattern** (4.1) | Client-side LLM provider routing. PT does not talk to LLMs; LLMs talk to PT. The adapter lives in CodeCompanion, Claude Desktop, Cursor -- not in the data source. |
| P3-5 | **Tool Orchestrator** (4.3) | Same rationale as P3-2. Client-side concern. PT should be the best tool that orchestrators call, not a competing orchestrator. |
| P3-6 | **Multi-Adapter LLM** (4.4) | Intentionally excluded. Model-agnostic JSON output is Parseltongue's correct strategy. The 99% token reduction (50 tokens for stats) works identically across GPT-4's 128K, Claude's 200K, and Gemini's 1M context windows. Adding per-model formatting creates coupling and dilutes the architecture. |
| P3-7 | **Obsidian-Aware Search** (5.2) | Wrong domain entirely. Parseltongue analyzes code dependency graphs, not Markdown knowledge bases. Zero user overlap between "code analysis for LLMs" and "Obsidian vault search." |

---

## Implementation Roadmap (Recommended Sequence)

```
Phase 1 (v1.7.3-v1.7.5): P0 features — MCP bridge + source extraction + SARIF
  Week 1-3:  pt09 MCP bridge (P0-1) — unlock Claude Desktop / Cursor adoption
  Week 3-5:  Source extraction + tiktoken (P0-2) — graph-aware code extraction
  Week 5-7:  SARIF export (P0-3) — CI/CD pipeline integration

Phase 2 (v1.8.x): P1 features — security + policy + structural search
  Week 8-10:  Structural pattern search (P1-3) — quick win, extends graph queries
  Week 10-14: Datalog policy engine (P1-2) — enterprise compliance
  Week 14-22: Graph-native taint analysis (P1-1) — deepest moat feature

Phase 3 (v1.9.x): P2 features — polish for Tauri desktop app
  As needed:  Telemetry, sessions, model-awareness, Lua support, semgrep overlay
```

**Total estimated effort for P0+P1**: 16-23 weeks, ~9,000 LoC
**Features that already exist and need no work**: 2.1 (CozoDB), 2.2 (tree-sitter), 2.3 (file watcher)
**Features intentionally excluded**: 1.3, 3.1, 3.2, 4.1, 4.3, 4.4, 5.2 (7 features = 0 LoC saved)

---

*Document generated by Agent 03 (Pass 3) on 2026-02-14. Analysis based on Parseltongue v1.6.5 (API v1.6.5, 24 endpoints, 44,890 entities, 385,560 edges) running on port 7777. See also: [CR-v173-04-oh-my-pi.md](CR-v173-04-oh-my-pi.md) for oh-my-pi competitive deep-dive.*
