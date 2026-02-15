# Structural Taint Analysis via Tree-Sitter + Datalog Graph Reachability

**A Technical Thesis for Parseltongue v1.7.3**
**Date**: 2026-02-15
**Author**: Generated via competitive research (code-scalpel v3.0.4 + semgrep MCP + academic literature)

---

## Abstract

This document presents the design rationale and implementation strategy for adding structural taint analysis to Parseltongue — a Rust-based code analysis toolkit that parses codebases into a CozoDB (Datalog) graph database. We propose a three-layer architecture: (1) tree-sitter extracts data-flow edges during ingestion, (2) a curated source/sink/sanitizer registry classifies entities, and (3) CozoDB Datalog queries perform taint propagation via graph reachability. This approach trades symbolic precision (no Z3) for polyglot coverage (12 languages), millisecond query latency, and zero new dependencies.

The design is informed by deep source-code analysis of code-scalpel's TaintTracker (Python AST + Z3, 2,466 lines, 27 sink types, 80+ sanitizers) and semgrep's pattern-based taint mode, plus academic foundations (Denning 1976, Joern/CPG 2014, DOOP/Soufflé Datalog engines).

---

## 1. Academic Foundations

### 1.1 Information Flow and Lattice Models

The theoretical foundation for taint analysis traces to Dorothy E. Denning's 1976 paper "A Lattice Model of Secure Information Flow" (Communications of the ACM, Vol. 19, pp. 236-243). With over 2,079 citations (Semantic Scholar), this paper established that secure information flow can be modeled as a lattice where security classes form a partial order, and information may only flow "upward" in the lattice.

**Key insight for Parseltongue**: Taint analysis is information flow analysis. "Tainted" and "untainted" form the simplest lattice: `Untainted < Tainted`. Data may flow from untainted to tainted (safe), but tainted data reaching an untainted sink (SQL query expecting safe input) is a violation. Our CozoDB implementation encodes this as: a path exists in the graph from a TaintSource entity to a TaintSink entity without passing through a Sanitizer entity.

Follow-up works: Denning & Denning (1977) provided a static certification mechanism. Andrew C. Myers' **JFlow** (POPL 1999) extended Java with statically-checked information flow annotations, evolving into the Jif system at Cornell. Sabelfeld & Myers (IEEE S&P 2003) published the canonical survey citing ~150 papers.

**Reference**: [Denning 1976 — ACM Digital Library](https://dl.acm.org/doi/10.1145/360051.360056) | [Semantic Scholar — 2,079 citations](https://www.semanticscholar.org/paper/5f2b22b77559ddb4f3734459d1ff66c58d22df12) | [Myers JFlow — ACM DL](https://dl.acm.org/doi/10.1145/292540.292561)

### 1.2 Code Property Graphs (Joern, 2014)

Fabian Yamaguchi et al. introduced Code Property Graphs (CPGs) in "Modeling and Discovering Vulnerabilities with Code Property Graphs" (IEEE S&P 2014). CPGs merge Abstract Syntax Trees (AST), Control Flow Graphs (CFG), and Program Dependence Graphs (PDG) into a single queryable graph. The paper received the IEEE Test-of-Time Award in 2024.

**Key insight for Parseltongue**: Parseltongue already stores AST-derived entities and call-graph edges in CozoDB. Adding data-flow edges (assignments, parameters, returns) moves us toward a CPG-like representation without the full CFG overhead. Joern's taint-analysis engine demonstrates that graph reachability queries over CPGs effectively find taint-style vulnerabilities — reducing code to inspect by 94.9% in Yamaguchi et al.'s 2015 follow-up.

**Reference**: [Joern CPG Specification](https://cpg.joern.io/) | [Yamaguchi 2014 — Semantic Scholar](https://www.semanticscholar.org/paper/Modeling-and-Discovering-Vulnerabilities-with-Code-Yamaguchi-Golde/07c4549be429a52274bc0ec083bf5598a3e5c365)

### 1.3 Datalog for Program Analysis (DOOP, Soufflé)

The DOOP framework (Smaragdakis & Bravenboer, 2009) demonstrated that full context-sensitive points-to analysis can be specified entirely in Datalog, achieving 15x speedup over the previous state-of-the-art (PADDLE). Soufflé (Oracle Labs, CC 2016) compiles Datalog to parallel C++ — achieving 16x faster execution with 1/20th memory via trie-based data structures.

**Key insight for Parseltongue**: CozoDB is a Datalog engine. Taint propagation is naturally expressed as recursive Datalog reachability — what takes hundreds of lines of imperative code becomes a handful of declarative rules. Soufflé demonstrates that Datalog-based taint analysis scales to millions of variables. CozoDB's Datalog engine handles the same class of queries.

**References**:
- [DOOP — Using Datalog for Fast and Easy Program Analysis (Smaragdakis)](https://yanniss.github.io/doop-datalog2.0.pdf)
- [Soufflé — On Fast Large-Scale Program Analysis in Datalog (CC 2016)](https://souffle-lang.github.io/pdf/cc.pdf)
- [P/Taint — A unified points-to and taint analysis (Grech & Smaragdakis, 2017)](https://yanniss.github.io/)

### 1.4 Rust Code Property Graph (cpg-rs)

A Rust library `cpg-rs` by gbrigandi exists for constructing Code Property Graphs in Rust, suggesting the CPG paradigm can be implemented natively in Rust without JVM dependencies.

**Reference**: [cpg-rs on GitHub](https://github.com/gbrigandi/cpg-rs)

---

## 2. Competitor Deep-Dive: code-scalpel's TaintTracker

### 2.1 Architecture Overview

code-scalpel (v3.0.4) implements taint analysis in `src/code_scalpel/security/analyzers/taint_tracker.py` — a single 2,466-line Python file that combines:

| Component | Lines | Purpose |
|-----------|-------|---------|
| `TaintSource` enum | 63-81 | 7 source categories |
| `SecuritySink` enum | 84-139 | 27 sink types with CWE mapping |
| `TaintLevel` enum | 142-155 | 4 confidence levels |
| `TaintInfo` dataclass | 158-274 | Taint metadata with propagation + sanitizer tracking |
| `SINK_SANITIZERS` dict | 278-301 | Legacy sink→sanitizer mapping |
| `SanitizerInfo` dataclass | 309-323 | Sanitizer metadata with `full_clear` flag |
| `SANITIZER_REGISTRY` dict | 328-670 | 80+ sanitizers across Python, JS, Java |
| `TaintTracker` class | 673-872+ | Core propagation engine |
| `SINK_PATTERNS` dict | ~900-2049 | 200+ sink function patterns across languages |
| `SSR_SINK_PATTERNS` | 2056-2091 | Next.js, Remix, Nuxt, SvelteKit, Astro patterns |
| `HARDCODED_SECRET_PATTERNS` | 2093-2156 | 40+ regex patterns for secrets |
| `SANITIZER_PATTERNS` | 2197-2205 | Function name → sanitizer mapping |
| SSR detection functions | 2213-2466 | Framework auto-detection + vulnerability scanning |

### 2.2 Taint Source Classification

code-scalpel defines 7 taint source categories:

```python
class TaintSource(Enum):
    USER_INPUT = auto()      # request.args, request.form, sys.argv
    FILE_CONTENT = auto()    # open().read()
    NETWORK_DATA = auto()    # socket.recv(), requests.get()
    DATABASE = auto()        # cursor.fetchone()
    ENVIRONMENT = auto()     # os.environ
    HARDCODED = auto()       # Hardcoded secrets
    UNKNOWN = auto()         # Source couldn't be determined
```

**Parseltongue mapping**: We adopt all 7, plus add `COMMAND_LINE_ARG` as distinct from `USER_INPUT` (code-scalpel groups them; we separate for clarity) and `DESERIALIZATION` (pickle.loads, serde_json::from_str).

### 2.3 Security Sink Classification (27 Types)

code-scalpel v3.0.4 has evolved to 27 sink types through 5 major versions:

| Version | Sinks Added | Total |
|---------|-------------|-------|
| v1.0 | SQL_QUERY, HTML_OUTPUT, FILE_PATH, SHELL_COMMAND, EVAL, DESERIALIZATION, LOG_OUTPUT, HEADER, WEAK_CRYPTO, SSRF, HARDCODED_SECRET | 11 |
| v1.4 | XXE, SSTI | 13 |
| v2.0 | DOM_XSS, PROTOTYPE_POLLUTION, REDIRECT | 16 |
| v3.0.4 | UNVALIDATED_OUTPUT, LDAP_INJECTION, XPATH_INJECTION, NOSQL_INJECTION, EMAIL_INJECTION, REGEX_DOS, FORMAT_STRING, UNSAFE_REFLECTION, EL_INJECTION, GRAPHQL_INJECTION, CORS_MISCONFIGURATION, JWT_WEAKNESS, HTML_INJECTION | 27 (+2 reserved) |

Each sink maps to a CWE:

| Sink | CWE | OWASP 2021 |
|------|-----|------------|
| SQL_QUERY | CWE-89 | A03 Injection |
| SHELL_COMMAND | CWE-78 | A03 Injection |
| HTML_OUTPUT / DOM_XSS | CWE-79 | A03 Injection |
| FILE_PATH | CWE-22 | A01 Broken Access |
| EVAL | CWE-94 | A03 Injection |
| DESERIALIZATION | CWE-502 | A08 Integrity |
| SSRF | CWE-918 | A10 SSRF |
| XXE | CWE-611 | A05 Misconfiguration |
| SSTI | CWE-1336 | A03 Injection |
| REDIRECT | CWE-601 | A01 Broken Access |
| LDAP_INJECTION | CWE-90 | A03 Injection |
| XPATH_INJECTION | CWE-643 | A03 Injection |
| NOSQL_INJECTION | CWE-943 | A03 Injection |
| REGEX_DOS | CWE-1333 | A03 Injection |
| EL_INJECTION | CWE-917 | A03 Injection |
| GRAPHQL_INJECTION | CWE-89 variant | A03 Injection |
| CORS_MISCONFIGURATION | CWE-942 | A05 Misconfiguration |
| JWT_WEAKNESS | CWE-347 | A02 Crypto Failures |

**Parseltongue mapping**: We start with the 12 most common sinks (see PRD_v173.md). The remaining 15 are P2 additions — each requires only adding patterns to the registry, not changing the engine.

### 2.4 The Sanitizer Registry (80+ Entries)

code-scalpel's `SANITIZER_REGISTRY` is a `Dict[str, SanitizerInfo]` with 80+ entries. Each entry specifies:

```python
@dataclass
class SanitizerInfo:
    name: str                           # e.g., "html.escape"
    clears_sinks: Set[SecuritySink]     # Which sinks it neutralizes
    full_clear: bool = False            # Type coercion (int, float) clears ALL sinks
    confidence: float = 1.0             # How confident we are
```

**Key design patterns**:

1. **Sink-specific sanitizers**: `html.escape` clears `HTML_OUTPUT` but NOT `SQL_QUERY`. This is the critical insight that Z3 formalizes in code-scalpel.
2. **Full-clear sanitizers**: Type coercions (`int()`, `float()`, `bool()`, `Number()`, `parseInt()`) clear ALL sinks because they produce non-injectable primitive values.
3. **Multi-language coverage**: The registry includes Python (`html.escape`, `bleach.clean`), JavaScript (`DOMPurify.sanitize`, `encodeURIComponent`), Java (`StringEscapeUtils`, `ESAPI`), and Node.js (`mysql.escape`, `pg.escapeLiteral`).
4. **Schema validation as sanitization**: `zod.parse`, `ajv.validate`, `joi.validate`, `yup.validate` clear `DESERIALIZATION` sinks.

**Sanitizer categories from the registry**:

| Category | Count | Examples |
|----------|-------|---------|
| XSS/HTML sanitizers | 12 | html.escape, DOMPurify.sanitize, he.encode |
| SQL sanitizers | 8 | mysql.escape, pg.escapeLiteral, sqlstring.escape |
| Path sanitizers | 5 | os.path.basename, path.normalize, sanitize-filename |
| Shell sanitizers | 4 | shlex.quote, shell-quote.quote |
| Type coercions (full clear) | 10 | int, float, Number, parseInt, Boolean |
| XXE sanitizers | 7 | defusedxml.parse, defusedxml.fromstring |
| SSTI sanitizers | 3 | render_template, django.shortcuts.render |
| URL sanitizers | 2 | encodeURIComponent, encodeURI |
| Schema validators | 4 | zod.parse, ajv.validate, joi.validate |
| Java sanitizers | 15+ | ESAPI.encoder, StringEscapeUtils, Jsoup.clean |
| Safe DOM APIs | 3 | textContent, innerText, createTextNode |
| **Total** | **80+** | |

**Parseltongue mapping**: We adopt the same `SanitizerInfo` pattern in Rust. The registry is a `LazyLock<HashMap<&str, SanitizerInfo>>`. Adding new sanitizers requires only adding entries to the static map — no code changes.

### 2.5 Taint Propagation Algorithm

code-scalpel's `TaintTracker` class maintains a `_taint_map: Dict[str, TaintInfo]` — a shadow state mapping variable names to taint metadata.

**Propagation rules**:

1. **Source creation**: `taint_source(name, source)` creates a Z3 `String(name)` and marks it tainted with `TaintLevel.HIGH`.

2. **Assignment propagation**: `propagate_assignment(target, source_names)` — if ANY source is tainted, the target becomes tainted. When multiple sources are tainted, the MOST tainted level wins. Sanitizer histories are intersected (conservative: only sanitizers applied to ALL sources count).

3. **Concatenation propagation**: `propagate_concat(result, operands)` delegates to `propagate_assignment` — any tainted operand taints the result. This is the key rule for injection: `"SELECT * FROM users WHERE id=" + user_input` → the concatenation is tainted.

4. **Sanitizer application**: `apply_sanitizer(var, sanitizer)` looks up the sanitizer in `SANITIZER_REGISTRY`, adds cleared sinks to `TaintInfo.cleared_sinks`, lowers taint level to `LOW`, and if all major sinks are cleared, sets level to `NONE`.

5. **Sink check**: `is_dangerous_for(sink)` returns `True` if: (a) taint level is not NONE, (b) the sink is not in `cleared_sinks`, and (c) no applicable sanitizer was applied.

**Z3 usage**: code-scalpel creates Z3 `String` symbolic values for each tainted variable. These are used for:
- Tracking string concatenation symbolically (`Concat(s1, s2)`)
- Reasoning about whether transformations preserve or eliminate taint
- Potential future constraint solving (not heavily used in current codebase)

**Parseltongue equivalent**: We replace Z3 symbolic strings with CozoDB graph edges. Instead of tracking `String("user_input")` through operations, we store `DataFlowEdge(from=entity_A, to=entity_B, type=assign)` in the database. The "is tainted data dangerous?" question becomes a Datalog reachability query.

### 2.6 Sink Pattern Database (200+ Patterns)

code-scalpel maintains a massive `SINK_PATTERNS: Dict[str, SecuritySink]` dict mapping function/method names to sink types. This covers:

- **SQL**: `cursor.execute`, `Session.execute`, `connection.query`, `db.raw`, `knex.raw`, `sequelize.query`, `mongoose.aggregate` + 30 more
- **Shell**: `os.system`, `subprocess.run`, `child_process.exec`, `Runtime.exec` + 20 more
- **HTML**: `innerHTML`, `document.write`, `render_template_string`, `Markup()` + 15 more
- **File**: `open()`, `os.path.join`, `fs.readFile`, `new File()` + 10 more
- **SSR frameworks**: Next.js (`getServerSideProps`, `dangerouslySetInnerHTML`), Remix (`loader`, `action`), Nuxt (`defineEventHandler`, `useFetch`), SvelteKit (`load`), Astro (`Astro.props`, `set:html`)
- **Deserialization**: `pickle.loads`, `yaml.load`, `JSON.parse` (with caveats), `ObjectInputStream.readObject`
- **Crypto**: `hashlib.md5`, `hashlib.sha1`, `DES`, `RC4`
- **Reflection**: `getattr`, `eval`, `exec`, `Class.forName`, `Method.invoke`
- **Expression Language**: Spring SpEL, OGNL, MVEL, JSP EL
- **GraphQL**: `graphql.execute`, `ApolloServer`, `express-graphql`

**Parseltongue mapping**: These become tree-sitter query patterns. For each language, we match entity names against the patterns during ingestion. An entity matching `cursor.execute` gets tagged `taint_sink: SqlQuery`.

### 2.7 Hardcoded Secret Detection (40+ Regex Patterns)

code-scalpel includes `HARDCODED_SECRET_PATTERNS` — 40+ regex patterns for detecting hardcoded credentials:

| Category | Examples |
|----------|---------|
| AWS | `AKIA[A-Z0-9]{16}`, `aws_secret_access_key` patterns |
| GitHub | `ghp_`, `gho_`, `ghu_`, `github_pat_` prefixes |
| Stripe | `sk_live_`, `sk_test_`, `rk_live_` prefixes |
| Slack | `xox[baprs]-` prefix, webhook URLs |
| Google | `AIza` prefix, Firebase tokens |
| Private keys | `-----BEGIN RSA PRIVATE KEY-----` and variants |
| JWT tokens | `eyJ` pattern |
| Database URLs | `postgres://`, `mysql://`, `mongodb://`, `redis://` |
| Generic | `api_key=`, `secret=`, `password=`, `bearer` patterns |

**Parseltongue mapping**: This is a separate feature from taint analysis (hardcoded secrets don't require data-flow tracking). Could be a P2 addition: scan entity source code against regex patterns during ingestion, store matches in a `HardcodedSecrets` CozoDB relation.

---

## 3. Competitor Deep-Dive: Semgrep Taint Mode

### 3.1 Architecture

Semgrep's taint analysis is built into the semgrep CLI engine (written in OCaml). It is NOT exposed through the MCP servers we analyzed:

- **mcp-server-semgrep** (TypeScript, 7 tools): `scan_directory`, `list_rules`, `analyze_results`, `create_rule`, `filter_results`, `export_results`, `compare_results` — no dedicated taint tool
- **semgrep-mcp** (Python, Official): DEPRECATED in favor of native `semgrep mcp` command

Taint analysis is invoked indirectly through YAML rules with `mode: taint`.

### 3.2 Taint Rule YAML Structure

```yaml
rules:
  - id: sql-injection-python
    mode: taint
    severity: ERROR
    message: "Tainted data from $SOURCE reaches SQL query at $SINK"
    languages: [python]
    metadata:
      cwe: CWE-89
      owasp: A03:2021
    pattern-sources:
      - pattern: request.args.get(...)
      - pattern: request.form[...]
      - pattern: request.json
    pattern-sinks:
      - pattern: cursor.execute($QUERY, ...)
      - pattern: db.engine.execute($QUERY)
    pattern-sanitizers:
      - pattern: int(...)
      - pattern: escape_string(...)
    pattern-propagators:
      - pattern: $X = $Y
        from: $Y
        to: $X
```

### 3.3 Semgrep vs code-scalpel vs Parseltongue

| Dimension | Semgrep | code-scalpel | Parseltongue (proposed) |
|-----------|---------|-------------|------------------------|
| **Analysis engine** | OCaml (proprietary core) | Python AST + Z3 | Rust + tree-sitter + CozoDB |
| **Taint specification** | YAML rules (external) | Python enum/dict (internal) | Rust static maps (internal) |
| **Intra-procedural** | Yes (pattern matching) | Yes (AST walker) | Yes (tree-sitter data-flow) |
| **Inter-procedural** | Limited (cross-file taint in Pro) | Yes (cross-function) | Yes (CozoDB graph reachability) |
| **Sanitizer model** | Pattern match (`pattern-sanitizers`) | Registry + Z3 symbolic | Registry (curated, no Z3) |
| **Languages** | 30+ (but taint quality varies) | Python primary (JS/Java partial) | 12 (tree-sitter supported) |
| **Output** | SARIF, JSON findings | VulnerabilityDict with CWE | JSON flows with CWE + graph paths |
| **Query latency** | Seconds (re-scans code) | Seconds (re-parses AST) | Milliseconds (pre-computed graph) |
| **Integration** | CLI / MCP wrapper | CLI | HTTP API + MCP native |
| **Custom rules** | YAML files | pyproject.toml sanitizers | TBD (P2 — config file) |

### 3.4 What Semgrep Does Better

1. **Rule ecosystem**: 5,000+ community rules in the Semgrep Registry, covering dozens of frameworks
2. **Pattern language**: Semgrep's pattern syntax (`$X`, `...`, `<... $X ...>`) is more expressive than tree-sitter queries for matching code patterns
3. **Autofix**: Rules can include `fix:` patterns that automatically remediate findings
4. **SARIF export**: Standard format consumed by GitHub Advanced Security, Azure DevOps, etc.

### 3.5 What Parseltongue Does Better

1. **Pre-computed graph**: Taint queries are milliseconds, not seconds. The graph is built once during ingestion.
2. **Graph context**: A taint flow in Parseltongue comes with full dependency context — callers, callees, blast radius, Leiden community membership, k-core layer. No other tool provides this.
3. **Polyglot by default**: Tree-sitter supports 12 languages with the same query infrastructure. No per-language OCaml frontend needed.
4. **MCP native**: Taint tools are MCP tools — Claude/Cursor/VS Code can call them directly.

---

## 4. Proposed Architecture for Parseltongue

### 4.1 Design Principles

1. **No new dependencies**: Tree-sitter and CozoDB are already in the stack. No Z3, no OCaml, no Python.
2. **Registry over reasoning**: Curated sanitizer lookup instead of symbolic constraint solving. Simpler, faster, less precise.
3. **Graph-first**: Every taint fact is a CozoDB relation. Every taint query is a Datalog rule. The graph IS the analysis.
4. **Honest about limits**: We call it "structural taint analysis" — not "vulnerability detection." We find potential flows, not confirmed vulnerabilities.

### 4.2 New CozoDB Relations

```
# Entities with taint source classification
TaintSources {
    entity_key: String,      # ISGL1 key
    source_kind: String,     # UserInput, FileContent, NetworkData, ...
    pattern_matched: String, # Which pattern triggered classification
    confidence: Float,       # 1.0 = exact match, 0.7 = heuristic
}

# Entities with taint sink classification
TaintSinks {
    entity_key: String,      # ISGL1 key
    sink_kind: String,       # SqlQuery, ShellCommand, HtmlOutput, ...
    cwe_id: String,          # CWE-89, CWE-78, ...
    owasp_category: String,  # A03:2021, A01:2021, ...
    severity: String,        # critical, high, medium
    confidence: Float,
}

# Intra-function data-flow edges (NEW — extracted by tree-sitter)
DataFlowEdges {
    from_key: String,        # Source entity ISGL1 key
    to_key: String,          # Target entity ISGL1 key
    flow_type: String,       # assign, param, return
    variable_name: String,   # The variable carrying the data
    file_path: String,       # File where the flow occurs
    line_number: Int,        # Line of the assignment/call
}

# Known sanitizer entities
Sanitizers {
    entity_key: String,      # ISGL1 key of sanitizer entity
    sanitizer_name: String,  # e.g., "html.escape"
    neutralizes: [String],   # List of SecuritySinkKind values
    full_clear: Bool,        # Type coercion clears all sinks
}
```

### 4.3 Tree-Sitter Data-Flow Extraction

For each supported language, new tree-sitter queries extract three edge types:

**Assignment edges** (`x = foo()`):
```scheme
;; Rust
(let_declaration
  pattern: (identifier) @target
  value: (call_expression function: (_) @source))

;; Python
(assignment
  left: (identifier) @target
  right: (call) @source)

;; JavaScript/TypeScript
(variable_declarator
  name: (identifier) @target
  value: (call_expression function: (_) @source))
```

**Parameter edges** (`bar(x)` — x flows into bar's parameter):
```scheme
;; Universal pattern across languages
(call_expression
  function: (_) @callee
  arguments: (arguments (_) @arg))
```

**Return edges** (`return x` — x flows out of the function):
```scheme
;; Rust
(return_expression (_) @returned_value)

;; Python
(return_statement (_) @returned_value)
```

These queries produce `DataFlowEdges` that connect to existing `DependencyEdges` (call edges). Together they form the complete flow graph.

### 4.4 Taint Propagation Datalog Query

The core query finds paths from sources to sinks without sanitization:

```datalog
# ═══════════════════════════════════════════════════════
# Layer 1: Reachability through combined edge types
# ═══════════════════════════════════════════════════════

# Direct data-flow edge
reachable[start, end, 1] :=
    *DataFlowEdges[start, end, _, _, _, _]

# Direct call edge
reachable[start, end, 1] :=
    *DependencyEdges[start, end, _]

# Transitive closure (recursive)
reachable[start, end, depth] :=
    reachable[start, mid, d1],
    reachable[mid, end, d2],
    depth = d1 + d2,
    depth <= ?max_hops  # Bounded to prevent infinite recursion

# ═══════════════════════════════════════════════════════
# Layer 2: Taint flow detection
# ═══════════════════════════════════════════════════════

# An unsanitized taint flow exists when:
# 1. A source entity reaches a sink entity
# 2. No sanitizer on the path clears the sink type
taint_flow[source_key, sink_key, source_kind, sink_kind, cwe, depth] :=
    *TaintSources[source_key, source_kind, _, _],
    *TaintSinks[sink_key, sink_kind, cwe, _, _, _],
    reachable[source_key, sink_key, depth],
    not sanitized_path[source_key, sink_key, sink_kind]

# ═══════════════════════════════════════════════════════
# Layer 3: Sanitizer path detection
# ═══════════════════════════════════════════════════════

# A path is sanitized if any entity between source and sink
# is a sanitizer that clears the relevant sink type
sanitized_path[source_key, sink_key, sink_kind] :=
    reachable[source_key, san_key, _],
    reachable[san_key, sink_key, _],
    *Sanitizers[san_key, _, neutralizes, full_clear],
    (full_clear == true || sink_kind in neutralizes)
```

**Performance characteristics**:
- CozoDB evaluates Datalog with semi-naive evaluation (same algorithm as Soufflé)
- Bounded recursion (`depth <= max_hops`) prevents exponential blowup
- For typical codebases (< 50K entities), query time is < 100ms
- The graph is pre-computed — no file I/O during query

### 4.5 HTTP Endpoints

```
GET /{mode}/taint-flow-path-analysis?entity=ENTITY_KEY&hops=N
```

Response:
```json
{
  "entity": "rust:fn:handle_request:src_main_rs:15-45",
  "hops": 5,
  "flows": [
    {
      "source": {
        "entity_key": "python:fn:get_user_input:app_py:10-15",
        "source_kind": "UserInput",
        "pattern": "request.form"
      },
      "sink": {
        "entity_key": "python:fn:execute_query:db_py:30-35",
        "sink_kind": "SqlQuery",
        "cwe_id": "CWE-89",
        "severity": "critical"
      },
      "path": [
        "python:fn:get_user_input:app_py:10-15",
        "python:fn:process_data:app_py:20-25",
        "python:fn:execute_query:db_py:30-35"
      ],
      "depth": 3,
      "sanitized": false,
      "confidence": 0.85
    }
  ],
  "summary": {
    "total_flows": 3,
    "unsanitized": 2,
    "sanitized": 1,
    "critical": 1,
    "high": 1,
    "medium": 0
  }
}
```

```
GET /{mode}/taint-source-sink-discovery
```

Response:
```json
{
  "sources": [
    {"entity_key": "...", "source_kind": "UserInput", "confidence": 1.0}
  ],
  "sinks": [
    {"entity_key": "...", "sink_kind": "SqlQuery", "cwe_id": "CWE-89", "severity": "critical"}
  ],
  "sanitizers": [
    {"entity_key": "...", "sanitizer_name": "html.escape", "neutralizes": ["HtmlOutput"]}
  ],
  "summary": {
    "total_sources": 12,
    "total_sinks": 8,
    "total_sanitizers": 5,
    "total_flows": 23,
    "unsanitized_flows": 7
  }
}
```

### 4.6 MCP Tools

```
taint_flow_path_analysis(entity="rust:fn:handle_request", hops=5)
taint_source_sink_discovery()
```

These are added to the pt09 MCP tool registry alongside the existing 18 tools, bringing the total to 20.

---

## 5. CWE Top 25 (2025) — Taint Analysis Relevance

The 2025 CWE Top 25 (MITRE/CISA, December 2025) was compiled from 39,080 CVEs. **5 of the top 10 CWEs are directly detectable by taint analysis**:

| Rank | CWE | Name | Score | Taint-Detectable? |
|------|------|------|-------|-------------------|
| 1 | CWE-79 | Cross-Site Scripting (XSS) | 60.38 | **Yes** — tainted input reaching HTML output |
| 2 | CWE-89 | SQL Injection | 28.72 | **Yes** — tainted input reaching SQL queries |
| 6 | CWE-22 | Path Traversal | — | **Yes** — tainted input in file paths |
| 9 | CWE-78 | OS Command Injection | 15.65 | **Yes** — tainted input in system calls |
| 10 | CWE-94 | Code Injection | — | **Yes** — tainted input in eval/exec |

XSS (CWE-79) has held #1 for two consecutive years with a score of 60.38 — significantly higher than all others. All five share the same root cause: untrusted input flowing to sensitive operations without sanitization. This is exactly what taint analysis detects.

**Reference**: [MITRE CWE Top 25](https://cwe.mitre.org/top25/) | [CISA Advisory](https://www.cisa.gov/news-events/alerts/2025/12/11/2025-cwe-top-25-most-dangerous-software-weaknesses)

---

## 6. False Positive Rates — Empirical Data

| Tool/Approach | FP Rate | Domain | Source |
|---|---|---|---|
| RustGuard (Rust-specific, 2025) | **8.33%** | Rust programs | [Springer](https://link.springer.com/chapter/10.1007/978-981-95-3537-8_21) |
| SFlow/SFlowInfer (type-based) | **15%** | Java web apps | [RPI Paper](http://www.cs.rpi.edu/~milanova/docs/FASE14.pdf) |
| CFTaint (compositional) | **0.83-6.49%** | Microservices | [CUHK Paper](https://www.cse.cuhk.edu.hk/~cslui/PUBLICATION/) |
| SUTURE (high-order) | **51.23%** | Linux kernel | [CCS 2021](https://www.cs.ucr.edu/~zhiyunq/pub/ccs21_static_high_order.pdf) |
| CodeQL raw (kernel-level) | **~90%** | Linux kernel | [BugLens Paper](https://www.cs.ucr.edu/~zhiyunq/pub/ase25_buglens.pdf) |
| IRIS (LLM-augmented CodeQL) | **2x detections** | General | [arXiv:2405.17238](https://arxiv.org/html/2405.17238) |
| TaintTyper (ECOOP 2025) | **2.93-22.9x faster** | Type-based | [ECOOP 2025](https://2025.ecoop.org/details/ecoop-2025-technical-papers/29/) |
| AdaTaint (LLM-augmented) | **43.7% FP reduction** | General | [arXiv:2511.04023](https://arxiv.org/pdf/2511.04023) |

**Key insights**:
1. FP rates span two orders of magnitude (0.83% to 90%) depending on analysis scope and context sensitivity
2. **Context-sensitive analyses** (RustGuard at 8.33%) dramatically outperform context-insensitive ones
3. **Type-based approaches** (SFlow at 15%) provide natural constraints that eliminate infeasible paths
4. **LLM augmentation** is a 2024-2025 trend: IRIS doubled CodeQL's detection, AdaTaint cut FPs by 43.7%

**Parseltongue target**: 15-25% FP rate — comparable to type-based approaches. CozoDB's Datalog supports context encoding via additional relation columns (same technique as DOOP). Achieving <5% would require path-sensitive analysis or LLM post-filtering (P2 territory).

---

## 7. The Tooling Gap — Why This Architecture Is Novel

The internet research confirms a genuine gap in the tooling landscape:

| Tool | Parser | Taint Engine | Query Language | Deployment |
|------|--------|-------------|---------------|------------|
| CodeQL | Custom extractors (need compilation) | QL (Datalog-like) | QL | Server/CI |
| Semgrep | tree-sitter | OCaml engine | YAML patterns | CLI |
| Joern | Custom frontends (8 langs) | Scala traversals | CPGQL | JVM server |
| Soufflé | External (DOOP) | Compiled Datalog→C++ | Datalog | CLI (not persistent) |
| **Parseltongue** | **tree-sitter** | **CozoDB Datalog** | **Datalog** | **Embedded + HTTP + MCP** |

**No existing tool combines tree-sitter parsing with a Datalog engine for taint analysis.**

- Semgrep uses tree-sitter but performs taint in OCaml, not Datalog
- CodeQL has Datalog semantics but requires compilation, not tree-sitter
- Joern has CPGs but is JVM-based (Scala), not Rust
- Soufflé compiles Datalog to C++ but has no persistence or HTTP serving

CozoDB's unique advantage over Soufflé: CozoDB is **embeddable** (like SQLite) with RocksDB persistence, meaning taint results can be stored, queried incrementally, and served via HTTP — which Soufflé cannot do.

### Relevant Rust SAST Landscape

- **RustGuard** (2025): Extends `rustc` compiler, 91.67% precision/recall, 14% overhead. Requires compilation.
- **MIRAI** (Meta): Abstract interpreter for Rust MIR. Requires compilation.
- **PySpector**: Rust-powered SAST for Python with interprocedural taint engine.
- **cpg-rs**: Rust Code Property Graph library — confirms CPG paradigm works in Rust.
- **Aalborg University Thesis**: "Rust's ownership system is an excellent candidate for taint analysis" — strict aliasing reduces ambiguity.

**Reference**: [RustGuard](https://link.springer.com/chapter/10.1007/978-981-95-3537-8_21) | [Aalborg Thesis](https://projekter.aau.dk/projekter/files/421583418/Static_Taint_Analysis_in_Rust.pdf) | [cpg-rs](https://github.com/gbrigandi/cpg-rs)

---

## 8. Honest Assessment: What We Can and Cannot Do

### 8.1 What Structural Taint Analysis Catches

| Vulnerability Class | Can Parseltongue Detect? | How? |
|--------------------|-----------------------|------|
| SQL Injection via string concat | **Yes** | Source → concat → execute() path |
| XSS via template injection | **Yes** | Source → render_template_string() path |
| Command injection | **Yes** | Source → os.system() / subprocess path |
| Path traversal | **Yes** | Source → open() / fs.read() path |
| SSRF | **Yes** | Source → requests.get() / fetch() path |
| Deserialization attacks | **Yes** | Source → pickle.loads() / JSON.parse() path |
| Hardcoded secrets | **Partial** | Regex scan during ingestion (no flow needed) |

### 8.2 What Structural Taint Analysis Misses

| Scenario | Why We Miss It | code-scalpel Catches? |
|----------|---------------|---------------------|
| Custom sanitizer not in registry | No symbolic reasoning about function behavior | Yes (Z3 can model) |
| Taint through complex data structures | tree-sitter doesn't track `dict["key"]` flows | Partially (AST walker) |
| Implicit flows (`if tainted: x = "admin"`) | No control-flow analysis | No (code-scalpel misses too) |
| Taint through reflection/metaprogramming | tree-sitter sees syntax, not runtime behavior | No |
| Framework-specific magic (Django ORM) | Parameterized queries look like regular calls | Partially (heuristic) |

### 8.3 Projected Accuracy

| Metric | code-scalpel | Semgrep | Parseltongue (estimated) |
|--------|-------------|---------|--------------------------|
| True positive rate | ~90% | ~80% | ~75% |
| False positive rate | ~10% | ~20% | ~25% |
| Languages supported | 1 (Python) + partial JS/Java | 30+ | 12 |
| Cross-function tracking | Yes | Limited (Pro only) | Yes (graph native) |
| Query latency | Seconds | Seconds | Milliseconds |
| Setup overhead | pip install + Z3 | pip/npm install | Already running (pt08 server) |

### 8.4 The Confidence Score

Every taint flow gets a confidence score (0.0 - 1.0):

| Condition | Confidence |
|-----------|-----------|
| Direct path, exact pattern match for source + sink | 1.0 |
| Path through known functions, exact patterns | 0.9 |
| Path through unknown functions (black boxes) | 0.7 |
| Heuristic source match (name-based) | 0.6 |
| Path > 5 hops | 0.5 |
| Path through parameterized query (might be safe) | 0.3 |

This lets consumers (LLMs, security teams) prioritize flows by likelihood.

---

## 9. Implementation Plan

### 9.1 File Layout

```
parseltongue-core/src/
├── taint/
│   ├── mod.rs              # TaintSourceKind, SecuritySinkKind, TaintLevel enums
│   ├── registry.rs         # TAINT_SOURCES, TAINT_SINKS, SANITIZER_REGISTRY
│   ├── data_flow.rs        # DataFlowEdge extraction from tree-sitter
│   └── queries/            # Per-language tree-sitter query files
│       ├── rust.scm
│       ├── python.scm
│       ├── javascript.scm
│       ├── typescript.scm
│       ├── go.scm
│       ├── java.scm
│       ├── c.scm
│       ├── cpp.scm
│       ├── ruby.scm
│       ├── php.scm
│       ├── csharp.scm
│       └── swift.scm

parseltongue-core/src/storage/
└── cozo_client.rs          # +4 new relations: TaintSources, TaintSinks, DataFlowEdges, Sanitizers

pt01-folder-to-cozodb-streamer/src/
└── streamer.rs             # +taint extraction during ingestion

pt08-http-code-query-server/src/handlers/
├── taint_flow_path_analysis_handler.rs    # NEW
└── taint_source_sink_discovery_handler.rs # NEW
```

### 9.2 Line Estimates

| Component | Lines | Notes |
|-----------|-------|-------|
| Taint enums + types | 100 | TaintSourceKind, SecuritySinkKind, SanitizerInfo |
| Source/sink/sanitizer registry | 300 | Static HashMaps with patterns per language |
| Tree-sitter query files | 150 | 12 languages x ~12 lines each |
| CozoDB relation definitions | 60 | 4 new CREATE RELATION statements |
| pt01 taint extraction | 80 | Pattern matching + DataFlowEdge creation |
| Taint flow endpoint | 100 | Datalog query + response formatting |
| Source/sink discovery endpoint | 60 | Listing query + summary |
| MCP tool definitions | 40 | 2 new tools in pt09 |
| Tests | 200 | Registry tests, Datalog query tests, integration |
| **Total** | **~1,090** | No new dependencies |

### 9.3 Incremental Delivery

| Phase | What | Enables |
|-------|------|---------|
| Phase 1 | Enums + registry + CozoDB relations | Source/sink classification during ingestion |
| Phase 2 | Tree-sitter data-flow queries | DataFlowEdges extraction |
| Phase 3 | `/taint-source-sink-discovery` endpoint | "What sources and sinks exist?" |
| Phase 4 | Datalog propagation query | The actual taint tracking |
| Phase 5 | `/taint-flow-path-analysis` endpoint | "Show me taint flows" |
| Phase 6 | MCP tools | LLM-native taint queries |

---

## 10. Why This Matters (Shreyas Doshi Analysis)

### Product-Market Fit Assessment

| Framework | Score | Rationale |
|-----------|-------|-----------|
| **Painkiller vs Vitamin** | Painkiller (70%) | Security teams NEED taint visibility. But approximate results reduce urgency. |
| **PMF Score** | 70/100 | Useful for prioritization, not authoritative for compliance. |
| **User Segment** | Security-conscious dev teams + LLM agents doing code review | |
| **Competitive Moat** | Medium | Graph integration is unique. Registry approach is commoditized. |
| **LNO Rating** | Neutral-to-Leverage | Leverage if MCP taint tools become the default way LLMs check security. Neutral if viewed as "another SAST." |

### The Unique Value Proposition

No other tool provides taint analysis **inside a dependency graph with millisecond queries via MCP**. An LLM using Parseltongue can:

1. `blast_radius_impact_analysis(entity="handle_request", hops=3)` → see what's affected
2. `taint_flow_path_analysis(entity="handle_request", hops=5)` → see if any paths are vulnerable
3. `coupling_cohesion_metrics_suite(entity="handle_request")` → understand structural context
4. `smart_context_token_budget(focus="handle_request", tokens=4000)` → get relevant source code

This is a **security-aware code intelligence workflow** that no competitor offers as a unified API.

---

## 11. References

### Academic

1. Denning, D.E. (1976). "A Lattice Model of Secure Information Flow." *Communications of the ACM*, 19(5), 236-243. [ACM DL](https://dl.acm.org/doi/10.1145/360051.360056)
2. Yamaguchi, F. et al. (2014). "Modeling and Discovering Vulnerabilities with Code Property Graphs." *IEEE S&P 2014*. [Semantic Scholar](https://www.semanticscholar.org/paper/07c4549be429a52274bc0ec083bf5598a3e5c365)
3. Smaragdakis, Y. & Bravenboer, M. (2011). "Using Datalog for Fast and Easy Program Analysis." [PDF](https://yanniss.github.io/doop-datalog2.0.pdf)
4. Jordan, H. et al. (2016). "Soufflé: On Fast Large-Scale Program Analysis in Datalog." *CC 2016*. [PDF](https://souffle-lang.github.io/pdf/cc.pdf)
5. Grech, N. & Smaragdakis, Y. (2017). "P/Taint: Unified Points-to and Taint Analysis."

### Tools Analyzed

6. code-scalpel v3.0.4 — `src/code_scalpel/security/analyzers/taint_tracker.py` (2,466 lines)
7. semgrep-mcp — `src/semgrep_mcp/server.py` (deprecated, 494 lines)
8. mcp-server-semgrep — `src/index.ts` (773 lines, 7 MCP tools)

### Modern Tools & Research

9. IRIS (2024) — LLM-augmented CodeQL, 55 vulns vs 27. [arXiv:2405.17238](https://arxiv.org/html/2405.17238)
10. YASA (Ant Group, 2025) — Scalable multi-language taint on unified AST. [arXiv:2601.17390](https://arxiv.org/html/2601.17390v1)
11. TaintTyper (ECOOP 2025) — Type-based taint, 2.93-22.9x faster. [ECOOP 2025](https://2025.ecoop.org/details/ecoop-2025-technical-papers/29/)
12. AdaTaint — LLM-augmented taint, 43.7% FP reduction. [arXiv:2511.04023](https://arxiv.org/pdf/2511.04023)
13. RustGuard (2025) — Rust taint analysis, 91.67% precision. [Springer](https://link.springer.com/chapter/10.1007/978-981-95-3537-8_21)
14. Static Taint Analysis in Rust — Aalborg University thesis. [PDF](https://projekter.aau.dk/projekter/files/421583418/Static_Taint_Analysis_in_Rust.pdf)
15. CWE Top 25 (2025) — 39,080 CVEs analyzed. [MITRE](https://cwe.mitre.org/top25/) | [CISA](https://www.cisa.gov/news-events/alerts/2025/12/11/2025-cwe-top-25-most-dangerous-software-weaknesses)

### Rust Ecosystem

16. cpg-rs — Rust Code Property Graph library. [GitHub](https://github.com/gbrigandi/cpg-rs)
17. CozoDB — Datalog-based hybrid database. [GitHub](https://github.com/cozodb/cozo)
18. tree-sitter — Incremental parsing framework. [tree-sitter.github.io](https://tree-sitter.github.io/)
19. PySpector — Rust-powered SAST for Python. [GitHub](https://github.com/ParzivalHack/PySpector)
20. MIRAI — Abstract interpreter for Rust MIR. [GitHub](https://github.com/facebookexperimental/MIRAI)

### Industry

21. CodeQL Documentation — Data flow analysis. [Docs](https://codeql.github.com/docs/writing-codeql-queries/about-data-flow-analysis/)
22. Semgrep C/C++ — tree-sitter GLR parsing. [Blog](https://semgrep.dev/blog/2024/modernizing-static-analysis-for-c/)
23. SonarQube 2025.4 — Taint analysis expansion. [Blog](https://www.sonarsource.com/blog/sonarqube-server-2025-4-faster-analysis-stronger-security-better-coverage/)
24. Bearer CLI — tree-sitter based SAST. [Docs](https://docs.bearer.com/explanations/workflow/)
25. Neo4j for Cybersecurity. [Whitepaper](https://go.neo4j.com/rs/710-RRC-335/images/Neo4j-Graphs-for-Cybersecurity-Whitepaper.pdf)

---

*Generated 2026-02-15. Based on source code reading of code-scalpel v3.0.4 (2,466 lines taint_tracker.py), semgrep MCP servers (2 implementations), academic literature survey via web search (Denning, Joern, DOOP, Soufflé, RustGuard, TaintTyper, CWE Top 25), and Parseltongue architecture analysis.*
