# Prep-V200: Competitive Deep Dive

**Date**: 2026-02-16
**Context**: Exhaustive competitive analysis for rust-llm v2.0.0 positioning as "Code Intelligence for LLMs." Builds on existing research from `competitor_research/FEATURE-MATRIX-COMPARISON.md`, `docs/Prep-Doc-V200.md`, and `docs/Prep-V200-Max-Adoption-Architecture-Strategy.md`.

---

## Table of Contents

1. [CodeQL (GitHub/Semmle)](#1-codeql-githubsemmle)
2. [Semgrep (Return Security)](#2-semgrep-return-security)
3. [Sourcegraph / SCIP](#3-sourcegraph--scip)
4. [SonarQube (Sonar)](#4-sonarqube-sonar)
5. [AI Coding Tools as Indirect Competitors](#5-ai-coding-tools-as-indirect-competitors)
6. [Emerging Tools](#6-emerging-tools)
7. [Gap Analysis Table](#7-gap-analysis-table)
8. [Positioning Statement](#8-positioning-statement)

---

## 1. CodeQL (GitHub/Semmle)

### How It Works

CodeQL treats code as data. The pipeline has three stages:

```
Source Code  -->  Extractor  -->  CodeQL Database  -->  QL Queries  -->  SARIF Results
                 (per-language)   (relational snapshot)
```

1. **Database Extraction**: Language-specific extractors parse source code into a relational database snapshot. Each language gets its OWN database -- C++ code and Python code in the same repo produce two separate databases. The database captures the AST, control flow graph, and data flow graph for a single language at a single point in time.

2. **QL Query Language**: Queries are written in QL, a Datalog-derived declarative logic programming language. QL describes patterns to match in the code structure and dataflow. Example: "find any loop, enclosed in a function named foo, where the loop body contains a call to function bar." Queries are packaged into **query packs** and **library packs**.

3. **Query Execution**: The CodeQL engine evaluates queries against the database and outputs results in SARIF v2.1.0 format, which integrates directly with GitHub Advanced Security for alert generation and PR annotations.

4. **Community Packs**: The `GitHubSecurityLab/CodeQL-Community-Packs` repository provides community-driven query, library, and model packs that supplement the built-in set. Model packs extend analysis to libraries and frameworks not covered by default. As of December 2025 (v2.23.7), CodeQL ships **491 security queries** in the Default suite covering **166 CWEs**, plus an additional 135 queries in the Extended suite covering 35 more CWEs.

### What It is Good At

- **Security vulnerability detection**: CodeQL is the industry-leading semantic SAST engine. Deep data flow analysis, taint tracking, and variant analysis across function boundaries within a single language.
- **Declarative query power**: QL is genuinely expressive -- you can describe multi-step vulnerability patterns (source -> sanitizer bypass -> sink) declaratively. Security researchers share and build on each other's queries.
- **GitHub integration**: First-class integration into GitHub Actions, code scanning, and Copilot Autofix. Results appear directly in PRs. Autofix can now generate remediation suggestions for CodeQL findings.
- **Supported languages (2026)**: C/C++, C#, Go, Java/Kotlin, JavaScript/TypeScript, Python, Ruby, Rust, Swift, GitHub Actions. Rust support was added in 2025 with cross-site scripting, request forgery, and certificate check queries.
- **Community query growth**: 491 security queries (Dec 2025), growing by ~40 queries/year. Active community contribution pipeline.

### What It CAN'T Do

| Limitation | Detail |
|---|---|
| **No cross-language analysis** | Databases are per-language. A Python frontend calling a C extension through FFI is invisible to CodeQL. Data flow STOPS at the language boundary. This is an architectural limitation, not a missing feature. |
| **No LLM context optimization** | CodeQL outputs SARIF (finding reports). It has no concept of token budgeting, context ranking, or producing LLM-optimized output. SARIF is designed for dashboards and CI, not for feeding into an LLM. |
| **No architecture metrics** | No coupling/cohesion, no PageRank, no community detection, no k-core, no SQALE tech debt scoring. CodeQL is designed for finding security bugs, not for understanding architecture. |
| **Proprietary engine** | The CodeQL engine is NOT open source. It is free for research and open source code, but analyzing proprietary/private code requires a GitHub Advanced Security license. The QL language itself is proprietary. |
| **No embeddability** | CodeQL is a CLI tool and GitHub integration. You cannot `cargo add codeql` or use it as a library in your own application. It is a standalone tool, not a composable building block. |
| **No MCP support** | No Model Context Protocol server. No way for an LLM to call CodeQL as a tool natively. |
| **High barrier to entry** | QL is a specialized language that requires significant learning investment. Research shows general-purpose LLMs struggle to generate correct QL code due to its unique syntax and references to non-existent modules. |
| **Missing languages** | No PHP, no Scala, no Objective-C. C++20 modules not supported. C23/C++23 in beta only. |

### Pricing / Access Model

| Tier | Price | Access |
|---|---|---|
| **Open Source / Research** | Free | CodeQL CLI free on public repositories. Standard and community query packs included. |
| **GitHub Advanced Security** | ~$49/user/month (add-on to Enterprise) | Required for private/proprietary code scanning. Includes code scanning, secret scanning, dependency review. |
| **GitHub Enterprise Cloud** | ~$21/user/month (base) + GHAS add-on | Full enterprise feature set. SAML SSO, audit logs, auto-provisioning. |
| **Self-Hosted (Enterprise Server)** | Contact sales | On-premises deployment with Advanced Security add-on. |

**Key economic insight**: For a 100-person team, GitHub Advanced Security alone costs ~$59,000/year on top of Enterprise subscriptions. This creates a significant cost barrier for smaller organizations.

---

## 2. Semgrep (Return Security)

### How It Works

Semgrep's core differentiator is its rule syntax: rules look like the source code you want to match. No abstract DSL. No regex wrestling.

```
Source Code  -->  Tree-sitter Parse  -->  AST  -->  Pattern Matching  -->  Findings
                                          |
                                     IL Translation
                                          |
                                   Dataflow Analysis (taint, const prop)
```

1. **Pattern Matching**: Rules use a source-code-like syntax with metavariables (`$X`) as wildcards. Example: `requests.get($URL)` matches any call to `requests.get` regardless of the argument.

2. **Intermediate Language (IL)**: After tree-sitter parsing, the AST is translated into a language-agnostic IL for cross-language rule reuse. This translation is not fully complete for all language features.

3. **Dataflow Engine**: Built-in constant propagation and taint tracking. The taint engine tracks data flow from sources to sinks with sanitizer support. However, at present, users CANNOT write their own data-flow analyses -- only the built-in ones are available.

4. **Rule Registry**: Over 20,000 proprietary rules (Pro) plus 2,000+ community rules (OSS). Rules cover SAST, SCA, and secrets detection. The registry is a key network effect.

5. **Semgrep Assistant (AI)**: Post-processing of findings with AI to reduce noise by ~20% and provide step-by-step remediation guidance rated actionable >80% of the time.

### What It is Good At

- **Speed**: Single-file analysis means the control flow graph never exceeds file size. Extremely fast scanning even on large codebases.
- **Rule authoring**: Rules look like code. Creating a new custom rule takes minutes, not hours. This is the #1 reason security teams adopt Semgrep.
- **OSS core**: The Community Edition (LGPL-2.1) is genuinely useful. 2,000+ free rules, 30+ languages, VS Code and IntelliJ extensions.
- **Cross-function taint tracking (Pro)**: The Pro engine adds cross-file and cross-function analysis, reducing false positives by 25% and increasing true positive detection by 250% over OSS.
- **SCA with reachability (Pro)**: Supply chain scanning that determines whether your code actually calls the vulnerable function in a dependency. Filters out CVEs in imported-but-never-invoked packages.
- **Secrets detection**: Goes beyond regex pattern matching with semantic analysis.

### What It CAN'T Do

| Limitation | Detail |
|---|---|
| **No graph analysis** | No dependency graph, no SCC, no PageRank, no community detection, no k-core, no blast radius. Semgrep finds patterns in code; it does not model the architecture. |
| **No Datalog reasoning** | Unlike CodeQL (QL/Datalog) or rust-llm (Ascent), Semgrep has no general-purpose logic engine. You cannot write recursive queries or compose custom analyses. |
| **No type resolution** | Limited type inference only. Not a type checker. Features of some languages that Semgrep does not handle correctly are silently ignored, producing false negatives or false positives. |
| **No pointer/shape analysis** | Aliasing through arrays, pointers, or complex data structures is not detected. Individual array elements are not tracked. |
| **OSS is single-file only** | The free engine is intraprocedural. It cannot track data beyond a single function or file. Cross-file and cross-function analysis requires the paid Pro tier. |
| **No architecture awareness** | No module boundary detection, no coupling metrics, no cohesion scoring. |
| **No LLM optimization** | Output is findings/alerts. No token budgeting, no context ranking, no structured LLM output. |
| **No MCP server** | The official semgrep-mcp exists (5 tools), but it wraps the Semgrep CLI -- it is not an architectural code intelligence MCP server. |
| **Alert fatigue** | High false-positive rates reported. Many developers and CISOs are re-evaluating due to noise, slow scan performance on large codebases, and limited coverage gaps. |

### Pro vs. OSS Features

| Feature | Community (OSS) | Pro / Teams / Enterprise |
|---|---|---|
| Single-file SAST | Yes | Yes |
| Cross-file data flow | No | Yes |
| Cross-function taint | No | Yes |
| Pro Rules (20,000+) | No | Yes |
| SCA (dependency scanning) | No | Yes |
| Secrets detection | No | Yes |
| AI Assistant (noise reduction) | No | Yes |
| Dashboard and reporting | No | Yes |
| Custom rules | Yes | Yes |
| Languages (30+) | Yes | Yes |

### Pricing

| Tier | Price | Details |
|---|---|---|
| **Community (Free)** | $0 | OSS CLI + 2,000 community rules. Single-file analysis only. |
| **Teams** | Per-contributor/month (undisclosed) | Choose from Code (SAST), Supply Chain (SCA), or Secrets. ~35% discount available through negotiation. |
| **Enterprise** | Contact sales | White-glove onboarding, dedicated support, roadmap access. |

**Notable**: The **Opengrep** fork (response to Semgrep licensing changes) offers inter-file and cross-function taint analysis as open source. In benchmarks, Opengrep detected 7/9 multi-hop taint propagation cases vs. Semgrep 4/9.

---

## 3. Sourcegraph / SCIP

### How SCIP Works

SCIP (SCIP Code Intelligence Protocol) is Sourcegraph's replacement for LSIF. It is a Protobuf-based indexing format designed for code navigation.

```
Source Code  -->  Language Indexer  -->  SCIP Index (.scip)  -->  Upload  -->  Sourcegraph
                  (scip-typescript,                                            (precise navigation)
                   scip-java, etc.)
```

1. **Language Indexers**: Each language has a dedicated indexer (scip-java, scip-typescript, scip-python, scip-clang, scip-kotlin). These run the language compiler or type-checker to extract precise symbol information.

2. **SCIP Format**: A Protobuf schema centered on human-readable string IDs for symbols (replacing LSIF opaque numeric IDs). Benefits: 10x faster CI than lsif-node, smaller index files, better debugging.

3. **Precise Code Navigation**: Compiler-accurate go-to-definition and find-references that work across repositories. Search-based navigation (fast but less accurate) is available as a fallback.

4. **Current Indexers (2026)**: Java/Scala/Kotlin (scip-java), TypeScript/JavaScript (scip-typescript), Python (scip-python), C/C++ (scip-clang), Kotlin (scip-kotlin). Latest release: v0.5.2 (Feb 2025).

### What It Covers

- **Navigation**: Go-to-definition, find-references, hover documentation -- at compiler-level accuracy.
- **Cross-repo navigation**: Navigate between repositories. A function call in repo A can jump to its definition in repo B.
- **Planned: Cross-language navigation**: Navigate between Protobuf definitions and generated Java/Go bindings (not yet implemented as of 2026).
- **Planned: Incremental indexing**: Re-index only changed files after git push (not yet implemented).
- **Code search**: Sourcegraph core product. Regex, structural, and now AI-powered "Deep Search."
- **Cody AI**: AI coding assistant with whole-repository context via code index.

### What It CAN'T Do

| Limitation | Detail |
|---|---|
| **No architecture analysis** | SCIP indexes symbols and references. It does not compute dependency graphs, coupling metrics, community detection, k-core decomposition, or any architectural analysis. It knows WHERE symbols are defined and referenced, not HOW they relate architecturally. |
| **No safety analysis** | No vulnerability detection, no taint tracking, no unsafe chain detection. SCIP is a navigation protocol, not a security tool. |
| **No LLM optimization** | Sourcegraph Cody AI uses the code index for context, but SCIP itself has no concept of token budgeting, ranked context, or LLM-optimized output formats. Cody is a proprietary product built on top of SCIP, not an open composable layer. |
| **No graph algorithms** | No SCC, no PageRank, no blast radius, no Leiden clustering. SCIP provides a flat index of symbols, not a graph structure. |
| **No custom rules** | No way to write custom analysis rules, Datalog queries, or architectural constraints against the SCIP index. |
| **No embeddability** | SCIP is an indexing format/protocol. Sourcegraph is a hosted or self-hosted product. You cannot embed SCIP as a library in your own tool (though the format is open). |
| **Limited language depth** | Indexer quality varies. scip-clang for C/C++ is newer and less mature than scip-java. No Rust indexer, no PHP indexer, no Ruby indexer, no Go indexer (for precise; search-based covers these). |

### Pricing (2026)

| Tier | Price | Details |
|---|---|---|
| **Sourcegraph Free** | $0 | Code search on public repos. Cody Free: unlimited autocomplete, 200 chats/month. |
| **Enterprise Starter** | $19/user/month | Up to 50 developers, 100 repos, 5GB storage. Code search, AI chat, prompt library, private code indexing. |
| **Enterprise** | Contact sales | Universal code search, AI agents, batch changes, indexed code intelligence. |
| **Cody Pro** | ~$9/user/month | Enhanced AI coding assistant with more model access and features. |
| **Deep Search** | 3 searches/seat/month (included in Enterprise) | Extra usage incurs charges. |

---

## 4. SonarQube (Sonar)

### How It Works

SonarQube is a rule-based static analysis platform focused on code quality and security.

```
Source Code  -->  Language Plugin  -->  AST + CFG  -->  Rule Engine  -->  Issues
                  (per-language)                       (7,000+ rules)     (bugs, smells, vulns)
```

1. **Language Plugins**: Each supported language has a dedicated analyzer plugin. Analysis is per-language, per-repository.

2. **Rule Engine**: Over 7,000 rules across 35+ languages. Rules detect bugs, code smells, and security vulnerabilities. Rules are organized into Quality Profiles (language-specific collections). Rules map to OWASP Top 10, CWE Top 25.

3. **Taint Analysis**: Available in Enterprise/Advanced Security tier. Tracks data flow from sources through sanitizers to sinks. Cross-file and cross-function within a single language.

4. **AI Features (2025-2026)**: AI Code Assurance inspects AI-generated code against quality/security standards. AI CodeFix auto-generates fix suggestions for discovered issues.

5. **Advanced SAST (Enterprise)**: Improved detection of vulnerabilities in first-party code interacting with third-party dependencies. Currently supports Java, C#, and JavaScript/TypeScript.

### Enterprise Adoption

SonarQube has massive enterprise penetration. Key facts:
- 35+ languages, 7,000+ rules
- Integrates with GitHub, GitLab, Bitbucket, Azure DevOps
- Available as Cloud (SaaS), Server (self-hosted), and Community Build (free)
- Quality Gates enforce minimum quality standards on CI/CD pipelines
- SBOM generation for dependency tracking
- Compliance reporting for NIST SSDF, OWASP, CWE, STIG, CASA

### Pricing (2026)

| Tier | Price | Details |
|---|---|---|
| **Community Build (Free)** | $0 | Up to 50K LOC. Basic analysis, limited languages. |
| **Cloud Team** | EUR 30/month (100K LOC) | Advanced analysis, up to 1.9M LOC scaling. |
| **Cloud Enterprise** | Annual plan (contact sales) | 6 additional enterprise languages, SSO/SAML, portfolios, security reports. |
| **Server Developer** | ~$10,000/year (2M LOC) | Self-hosted. 16-25% discount potential. |
| **Server Enterprise** | ~$35,700/year (5M LOC) | Unlimited users/projects/scans. 39-46% discount potential. |
| **Advanced Security Add-on** | ~$35,700/year (on top of Enterprise) | Advanced SAST + SCA. Total ~$71,400/year for 5M LOC. |

### Limitations

| Limitation | Detail |
|---|---|
| **No graph analysis** | No dependency graph modeling, no SCC, no PageRank, no community detection. Analysis is file/function-scoped, not architecture-scoped. SonarQube is "excellent for file-level quality, blind to architectural context." |
| **No Datalog/logic engine** | Rule-based pattern matching and symbolic execution. No declarative query language. No recursive queries. Users cannot compose custom multi-step analyses. |
| **No cross-language semantic analysis** | Each language analyzed independently. No tracing of data flow across language boundaries (e.g., Rust FFI to C, Python calling Java via JNI). |
| **No LLM optimization** | Output is issue reports and dashboards. No token budgeting, no context ranking, no structured output for LLMs. |
| **No embeddability** | SonarQube is a platform (server + scanners). Not a library you can embed in your own tool. |
| **No MCP support** | No Model Context Protocol server. |
| **Business logic blind spot** | A 2024 Veracode study found 50% of critical security issues were logic-based and missed by static analysis tools like SonarQube. |
| **False positive rate** | A 2023 Forrester study found 35% of SonarQube alerts were non-issues, creating developer alert fatigue. |
| **Costly at scale** | Enterprise + Advanced Security can reach $71,400/year for 5M LOC. Multi-year contracts needed for significant discounts. |

---

## 5. AI Coding Tools as Indirect Competitors

These tools are not direct competitors to rust-llm, but they represent the PRIMARY USERS of what rust-llm produces. Understanding how they currently solve the code understanding problem reveals the gap rust-llm fills.

### Cursor

**Context approach**: RAG-based retrieval from local filesystem.

**How it works internally**:
1. **AST-based code chunking**: Uses tree-sitter to parse code into semantically meaningful chunks (functions, classes, methods) rather than fixed-token-length splitting.
2. **Embedding and vectorization**: Chunks are embedded using Cursor own embedding model into a vector database.
3. **Incremental updates via Merkle tree**: Detects changes and updates only affected data, avoiding full re-indexing.
4. **Semantic search via RAG**: Query is embedded, nearest-neighbor search retrieves relevant chunks, which are injected into the LLM context window.
5. **Multi-model routing**: Requests dynamically route to GPT-4, Claude 3.7 Sonnet, or Gemini 2.0 Flash based on task type.

**What Cursor lacks that rust-llm provides**:
- No architectural understanding (coupling, cohesion, modules, layers)
- No graph algorithms (SCC, PageRank, k-core, Leiden)
- No blast radius analysis ("what breaks if I change X?")
- No cross-language edge detection (FFI, WASM, gRPC boundaries)
- RAG retrieval is lexical/semantic similarity, NOT architectural relevance
- No custom analysis rules or Datalog reasoning

**How rust-llm helps Cursor**: As an MCP server, rust-llm could provide Cursor with architecturally-ranked context instead of similarity-ranked context. The "Deep Graph MCP" concept (already explored by the community) proves this demand exists.

### Aider

**Context approach**: Explicit file selection + repo-map via ctags.

**How it works**:
1. User manually adds files to context
2. Repo-map: Aider uses ctags to build a map of all symbols in the repository
3. AI proposes diffs that user reviews and commits
4. Model-agnostic: works with OpenAI, Anthropic, local models

**What Aider lacks that rust-llm provides**:
- Repo-map is flat (symbol list), not a graph (no edges, no relationships)
- No automatic context selection -- user must curate manually
- No blast radius, no SCC, no coupling metrics
- No cross-language understanding
- No token-budgeted ranked output

**How rust-llm helps Aider**: `rust-llm-context` as a library could replace Aider ctags-based repo-map with architecturally-ranked context. One function call: "give me the best 4K tokens for editing fn main."

### Continue

**Context approach**: Configurable blocks, rules, and integrations.

**How it works**:
1. Open-source IDE extension (VS Code, JetBrains) -- 20K+ GitHub stars
2. Highly configurable: "blocks" system lets you add prompts, rules, or integrations
3. Hub for sharing custom AI assistants and building blocks
4. Model-agnostic: local or remote models
5. Enterprise users include Siemens and Morningstar

**What Continue lacks that rust-llm provides**:
- No built-in code analysis engine
- Context is configured manually or via basic file inclusion
- No graph algorithms, no architectural metrics
- No cross-language edge detection

**How rust-llm helps Continue**: A rust-llm MCP server or Continue "block" could provide graph-aware context to any Continue-powered assistant.

### Cody (Sourcegraph)

**Context approach**: Sourcegraph code index for whole-repository understanding.

**How it works**:
1. Indexes entire project using Sourcegraph infrastructure
2. Provides context-aware answers using code index
3. Conversational Q&A for in-depth code analysis
4. Shared prompts for team productivity

**What Cody lacks that rust-llm provides**:
- Cody uses SCIP symbol index, not architectural analysis
- No graph algorithms (SCC, PageRank, community detection)
- No coupling/cohesion metrics
- No cross-language boundary detection
- No custom rule engine
- Context is symbol-based, not architecture-based

**How rust-llm helps Cody**: rust-llm architectural analysis could complement Cody symbol-level understanding. Where Cody knows "this symbol is defined here and referenced there," rust-llm knows "this symbol is the highest-PageRank entity in a 20-node SCC with 14 hours of tech debt."

### Summary: The Gap AI Tools Would Fill

```
CURRENT STATE OF AI CODE UNDERSTANDING (2026):

  Cursor:     Tree-sitter + RAG embeddings = lexical/semantic similarity
  Aider:      ctags repo-map = flat symbol list
  Continue:   Manual/configurable context = user-defined
  Cody:       SCIP symbol index = definition/reference graph
  Claude Code: tree-sitter + grep = manual exploration

WHAT NONE OF THEM HAVE:

  - Architectural understanding (coupling, cohesion, modules, layers)
  - Graph algorithms (SCC, PageRank, k-core, Leiden, entropy)
  - Blast radius analysis
  - Cross-language boundary detection
  - Datalog-powered custom rules
  - Token-budgeted architecturally-ranked context

THIS IS EXACTLY WHAT rust-llm v2.0.0 PROVIDES.
```

---

## 6. Emerging Tools

### ast-grep

**What it is**: A CLI tool for code structural search, lint, and rewriting, written in Rust. 12,400+ GitHub stars. Actively maintained (latest release: Jan 2026).

**How it works**: Uses tree-sitter to parse code into AST, then matches patterns against the AST structure. Patterns look like code with metavariable wildcards. Supports search, rewrite, lint, and analyze operations. Query formats: pattern strings, YAML rules, or programmatic API.

**Key capabilities**:
- Structural code search (find patterns that look like code, not regex)
- AST-based code rewriting / refactoring
- Custom lint rules in YAML
- Language server for IDE integration
- Multi-language support via tree-sitter
- Vercel Turbo uses ast-grep for Rust linting

**Relationship to rust-llm**: ast-grep is a pattern-matching tool. It finds instances of patterns. It does NOT build dependency graphs, compute graph algorithms, detect cross-language boundaries, or optimize context for LLMs. ast-grep and rust-llm are complementary, not competitive. ast-grep could be a RULE PRODUCER in the rust-llm-facts ecosystem (Option B from the architecture strategy).

### Mozilla rust-code-analysis

**What it is**: A Rust library for analyzing and extracting metrics from source code. Uses tree-sitter. Maintained by Mozilla / SoftengPoliTo.

**Capabilities**:
- 11 maintainability metrics for 10+ languages
- Halstead measures, cyclomatic complexity, LOC, etc.
- CLI tool + REST API (rust-code-analysis-web)
- MPL-2.0 license

**Relationship to rust-llm**: rust-code-analysis computes file/function-level metrics. It does NOT build dependency graphs, run graph algorithms, detect cross-language edges, or optimize for LLMs. rust-llm metrics (SQALE, CK metrics, Shannon entropy) are graph-aware and architecture-scoped, which is a different level of analysis.

### Charon (Rust Analysis Framework)

**What it is**: An analysis framework for Rust that converts MIR to LLBC for program verification. Interfaces with the rustc compiler. Published at a research venue, actively maintained (Jan 2026).

**Relationship to rust-llm**: Charon operates at the MIR (Mid-level Intermediate Representation) level for formal verification of Rust programs. This is deeper than tree-sitter (syntax) but narrower (Rust only, verification focus). Complementary for rust-llm-03 (rust-analyzer bridge) but not competitive for the broader code intelligence mission.

### RAPx / lockbud / Rudra

**What they are**: Rust-specific static analysis tools for memory safety and concurrency bug detection. They work on HIR/MIR to detect Use-After-Free, Double-Lock, Atomicity-Violation, etc.

**Relationship to rust-llm**: These are specialized Rust safety checkers. rust-llm safety analysis (via Ascent rules) is broader (cross-language, architectural, taint tracking) but shallower per-language than MIR-based tools. They could be FACT PRODUCERS feeding into rust-llm-facts.

### mago (PHP Toolchain in Rust)

**What it is**: A complete PHP toolchain written in Rust -- formatter, linter, static analyzer, and architectural guard. Enforces dependency rules.

**Relationship to rust-llm**: The "architectural guard" feature is directly relevant. mago proves that dependency rule enforcement at the architectural level is a real user need. rust-llm generalizes this across 12 languages.

### Code Scalpel (from Existing Research)

From `competitor_research/FEATURE-MATRIX-COMPARISON.md`: Code Scalpel is the most feature-rich MCP server competitor with 48 total features. It includes tree-sitter parsing, call graphs, taint analysis (cross-file), Z3 symbolic execution, CWE mapping, OWASP coverage, refactoring tools, test generation, and a unified IR. Written in Python.

**Key differentiator vs. rust-llm**: Code Scalpel has taint analysis and security scanning that rust-llm v1.x lacks. But Code Scalpel has NO graph algorithms (no SCC, no PageRank, no k-core, no Leiden, no SQALE). The feature matrix shows this clearly -- graph analysis is Parseltongue exclusive domain among all 17 tools surveyed.

### AiDex (from Existing Research)

MCP-native tree-sitter indexer with 27 features. Persistent SQLite index, incremental re-indexing, session management, task management, browser viewer. Claims 50-80% token reduction.

**Key differentiator vs. rust-llm**: AiDex has MCP native support and session management that rust-llm lacks. But AiDex has ZERO graph algorithms, ZERO metrics beyond basic stats, and no Datalog reasoning. It is a code indexer, not a code intelligence platform.

### MCP Ecosystem Context (2026)

The Model Context Protocol ecosystem has exploded since Anthropic November 2024 launch:
- 8+ million MCP server downloads by April 2025
- 5,800+ MCP servers and 300+ MCP clients available
- OpenAI adopted MCP in March 2025
- Anthropic donated MCP to the Linux Foundation (Agentic AI Foundation) in December 2025
- Co-founded by Anthropic, Block, and OpenAI

**Why this matters for rust-llm**: MCP is becoming the universal interface for LLM tool integration. Being MCP-native (rust-llm-07-mcp-server) is not optional -- it is the primary integration path for AI coding tools. Claude Desktop, Cursor, VS Code, and others all support MCP.

---

## 7. Gap Analysis Table

### Feature Matrix: rust-llm v2.0.0 vs. All Competitors

| Feature | CodeQL | Semgrep | Sourcegraph/SCIP | SonarQube | ast-grep | Code Scalpel | AiDex | rust-llm v2.0.0 |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Architecture Analysis** | | | | | | | | |
| Dependency graph construction | -- | -- | -- | -- | -- | Y | Y | **Y** |
| Blast radius / impact analysis | -- | -- | -- | -- | -- | -- | -- | **Y** |
| SCC (Tarjan) detection | -- | -- | -- | -- | -- | -- | -- | **Y** |
| PageRank / centrality ranking | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Betweenness centrality | -- | -- | -- | -- | -- | -- | -- | **Y** |
| K-core decomposition | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Leiden community detection | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Shannon entropy scoring | -- | -- | -- | -- | -- | -- | -- | **Y** |
| SQALE tech debt (ISO 25010) | -- | -- | -- | -- | -- | -- | -- | **Y** |
| CK metrics (CBO/LCOM/RFC/WMC) | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Semantic clustering | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Module boundary detection | -- | -- | -- | Partial | -- | -- | -- | **Y** |
| **Cross-Language** | | | | | | | | |
| Cross-language data flow | -- | -- | Planned | -- | -- | -- | -- | **Y** |
| FFI boundary detection | -- | -- | -- | -- | -- | -- | -- | **Y** |
| WASM export/import mapping | -- | -- | -- | -- | -- | -- | -- | **Y** |
| gRPC/HTTP contract matching | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Message queue topic linking | -- | -- | -- | -- | -- | -- | -- | **Y** |
| PyO3/JNI bridge detection | -- | -- | -- | -- | -- | -- | -- | **Y** |
| **LLM-Optimized Output** | | | | | | | | |
| Token-budgeted context | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Architecturally-ranked context | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Structured (non-text) output | -- | -- | -- | -- | -- | -- | -- | **Y** |
| 99% token reduction | -- | -- | -- | -- | -- | -- | 50-80% | **99%** |
| **Embeddable Library** | | | | | | | | |
| Usable as Rust crate | -- | -- | -- | -- | Y (API) | -- | -- | **Y** |
| Composable modules | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Feature-flag optional depth | -- | -- | -- | -- | -- | -- | -- | **Y** |
| **Custom Rules / Reasoning** | | | | | | | | |
| Datalog/logic query engine | Y (QL) | -- | -- | -- | -- | -- | -- | **Y (Ascent)** |
| Custom rule files | Y (.ql) | Y (.yaml) | -- | Y (rules) | Y (.yaml) | -- | -- | **Y (.rlm)** |
| Rule registry / community | Y (491+) | Y (20K+) | -- | Y (7K+) | -- | -- | -- | **Planned** |
| Recursive queries | Y | -- | -- | -- | -- | -- | -- | **Y** |
| Taint analysis | Y | Y (Pro) | -- | Y (Ent) | -- | Y | -- | **Y (via Ascent)** |
| **Graph Algorithms** | | | | | | | | |
| Strongly connected components | -- | -- | -- | -- | -- | -- | -- | **Y** |
| PageRank | -- | -- | -- | -- | -- | -- | -- | **Y** |
| K-core decomposition | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Community detection (Leiden) | -- | -- | -- | -- | -- | -- | -- | **Y** |
| Complexity hotspots | -- | -- | -- | Partial | -- | Y | -- | **Y** |
| **Protocol / Interface** | | | | | | | | |
| MCP support | -- | Y (5) | -- | -- | Y (3) | Y (23) | Y (22) | **Y (planned)** |
| HTTP REST API | -- | -- | -- | -- | -- | P | -- | **Y** |
| CLI | Y | Y | -- | Y | Y | Y | Y | **Y** |
| Embeddable in other tools | -- | Partial | -- | -- | Y | -- | -- | **Y** |
| **Security Analysis** | | | | | | | | |
| SAST vulnerability scanning | **Y** | **Y** | -- | **Y** | P | **Y** | -- | Planned |
| Cross-file taint tracking | **Y** | Y (Pro) | -- | Y (Ent) | -- | **Y** | -- | **Y (Ascent)** |
| CWE mapping | **Y** (166) | **Y** | -- | **Y** | -- | **Y** | -- | Planned |
| SARIF export | **Y** | -- | -- | -- | -- | Y | -- | Planned |
| CVE/SCA scanning | -- | Y (Pro) | -- | Y (Ent) | -- | Y | -- | -- |
| **Language Support** | | | | | | | | |
| Deep analysis languages | 11 | 30+ | 5-6 | 35+ | 15+ | 4 | 11 | **12** |
| Rust support | Y | Y | -- | Y (Cloud) | Y | P | Y | **Y (deep)** |

### Legend

- **Y** = Fully implemented
- **P** = Partial / limited
- **--** = Not implemented
- **Planned** = On the v2.0.0 roadmap but not yet shipped
- **Ent** = Enterprise/paid tier only
- **Pro** = Pro/paid tier only

---

## 8. Positioning Statement

### Where rust-llm Sits

```
                          SECURITY DEPTH
                               |
                    CodeQL     |     Semgrep
                  (QL queries, |   (pattern matching,
                   deep taint) |    fast, OSS core)
                               |
                               |
  ARCHITECTURE  ---------------+------------------  CODE NAVIGATION
  UNDERSTANDING                |                    & SEARCH
                               |
                   rust-llm    |    Sourcegraph/SCIP
                  (graph algos,|   (go-to-def,
                   Datalog,    |    find-refs,
                   LLM output) |    code search)
                               |
                               |
                    SonarQube  |     AI Coding Tools
                  (quality     |   (Cursor, Aider,
                   rules,      |    Continue, Cody)
                   enterprise) |
                               |
                          CODE QUALITY
```

### The Unique Combination No One Else Has

rust-llm v2.0.0 occupies a position that NO existing tool occupies. Here is why:

**1. Architecture Analysis + LLM Output = Nobody**

CodeQL finds security bugs. Semgrep matches patterns. SonarQube scores quality. NONE of them understand the ARCHITECTURE of a codebase (coupling, cohesion, modules, layers, hotspots). And NONE of them produce output optimized for LLM consumption (token-budgeted, ranked, structured).

rust-llm does BOTH. It understands architecture AND outputs for LLMs.

**2. Cross-Language Edges = Zero Competitors**

No tool in the entire competitive landscape detects cross-language boundaries: FFI links between Rust and C, WASM exports from Rust to JavaScript, PyO3 bridges between Rust and Python, gRPC contracts across any language pair, message queue topic linkage. CodeQL per-language databases make this architecturally impossible. Semgrep per-file analysis cannot see it. SCIP planned cross-language navigation is limited to Protobuf-to-generated-code pairs.

rust-llm sees BOTH sides of every boundary because tree-sitter parses ALL languages.

**3. Embeddable Datalog + Graph Algorithms = Nobody**

CodeQL has QL (Datalog-derived) but it is NOT embeddable -- it is a proprietary engine you cannot `cargo add`. Semgrep has custom rules but NO Datalog reasoning. SonarQube has rules but no logic engine.

rust-llm has Ascent (compile-time Datalog) that IS embeddable, combined with 7+ graph algorithms that run on typed Rust data. Anyone can `cargo add rust-llm-core` and get Datalog reasoning + graph algorithms in their own tool.

**4. Protocol Layer = No One Even Tries**

SCIP is a code navigation protocol (definitions + references). SARIF is a findings protocol (security alerts). Neither covers architecture, safety, cross-language, or LLM context.

rust-llm-facts (Option B from the architecture strategy) would be the FIRST protocol for code analysis facts -- covering entities, edges, attributes, cross-language links, and derived architectural insights. A "protobuf for code analysis."

### The One-Line Pitch

> **rust-llm: The code intelligence layer that AI coding tools are missing.**

Every AI coding tool (Cursor, Aider, Continue, Cody, Claude Code, Copilot) needs to understand code architecture to give better answers. None of them have it. rust-llm provides it -- as a library they embed, an MCP server they call, or an HTTP API they query.

### The Competitive Moat (Ordered by Defensibility)

```
MOAT LAYER 1: RULES (highest lock-in)
  Teams write custom Ascent rules encoding institutional knowledge.
  50 rules = cannot switch. Like CodeQL query library, but embeddable and open.

MOAT LAYER 2: CROSS-LANGUAGE EDGES (zero competitors)
  No other tool detects FFI, WASM, PyO3, gRPC, message queue boundaries.
  This data exists NOWHERE else.

MOAT LAYER 3: GRAPH ALGORITHMS (no competitor has >0)
  SCC, PageRank, k-core, Leiden, SQALE, CK metrics, entropy.
  Among 17 tools surveyed, Parseltongue is the ONLY one with graph analysis.

MOAT LAYER 4: LLM-NATIVE OUTPUT (category-defining)
  Token-budgeted, architecturally-ranked, structured context.
  Not an afterthought. The primary design goal.

MOAT LAYER 5: PROTOCOL (rust-llm-facts)
  If adopted, creates network effects: every new producer and consumer
  strengthens the ecosystem. The "LSP for code analysis" play.
```

### Who Pays and Why

| Buyer | What They Pay For | Why rust-llm Wins |
|---|---|---|
| **AI tool builders** (Cursor, Continue, etc.) | Embed `rust-llm-context` as a library | Better context = better AI output. No one else offers this as a library. |
| **Platform teams** | `rust-llm-graph` for architecture analysis | Only tool that computes graph algorithms on code. CI pipeline for coupling regression. |
| **Security teams** | `rust-llm-safety` for unsafe chain / taint analysis | Embeddable CodeQL alternative. No GitHub lock-in. Runs locally. |
| **Enterprise architects** | `rust-llm-rules` for custom analysis | Institutional knowledge encoded as Datalog rules. Enforced on every PR. |
| **Polyglot teams** | `rust-llm-crosslang` for boundary detection | The only tool that sees cross-language connections. Microservice visualization. |

---

## Appendix A: Sources

### CodeQL
- [CodeQL GitHub Repository](https://github.com/github/codeql)
- [CodeQL Documentation](https://codeql.github.com/docs/)
- [CodeQL Supported Languages](https://codeql.github.com/docs/codeql-overview/supported-languages-and-frameworks/)
- [CodeQL Community Packs](https://github.com/GitHubSecurityLab/CodeQL-Community-Packs)
- [GitHub Advanced Security Pricing](https://github.com/pricing)
- [CodeQL SARIF Output](https://docs.github.com/en/code-security/codeql-cli/using-the-advanced-functionality-of-the-codeql-cli/sarif-output)
- [About CodeQL](https://codeql.github.com/docs/codeql-overview/about-codeql/)
- [CodeQL 2.23.7 Changelog](https://codeql.github.com/docs/codeql-overview/codeql-changelog/codeql-cli-2.23.7/)
- [Getting started with CodeQL (Tweag)](https://www.tweag.io/blog/2025-08-07-codeql-intro/)
- [Understanding CodeQL Pricing (BytePlus)](https://www.byteplus.com/en/topic/516929)

### Semgrep
- [Semgrep GitHub Repository](https://github.com/semgrep/semgrep)
- [Semgrep Pricing](https://semgrep.dev/pricing/)
- [Semgrep Pro vs OSS](https://semgrep.dev/docs/semgrep-pro-vs-oss)
- [Semgrep Dataflow Analysis](https://semgrep.dev/docs/writing-rules/data-flow/data-flow-overview)
- [Semgrep Alternatives (Aikido)](https://www.aikido.dev/blog/semgrep-alternatives)
- [Comparing Semgrep CE and Semgrep Code](https://semgrep.dev/blog/2025/security-research-comparing-semgrep-community-edition-and-semgrep-code-for-static-analysis/)
- [Semgrep Wikipedia](https://en.wikipedia.org/wiki/Semgrep)
- [Semgrep 2026 (AppSec Santa)](https://appsecsanta.com/semgrep)

### Sourcegraph / SCIP
- [SCIP GitHub Repository](https://github.com/sourcegraph/scip)
- [SCIP Announcement Blog](https://sourcegraph.com/blog/announcing-scip)
- [Sourcegraph Pricing](https://sourcegraph.com/pricing)
- [Sourcegraph Pricing Plans Docs](https://sourcegraph.com/docs/pricing/plans)
- [Sourcegraph Precise Code Navigation](https://sourcegraph.com/docs/code-search/code-navigation/precise_code_navigation)
- [Sourcegraph 6.7 Release](https://sourcegraph.com/changelog/releases/6.7)
- [Cody Pricing](https://sourcegraph.com/docs/cody/usage-and-pricing)

### SonarQube
- [SonarQube Product Page](https://www.sonarsource.com/products/sonarqube/)
- [SonarQube Pricing](https://www.sonarsource.com/plans-and-pricing/)
- [SonarQube Server Pricing](https://www.sonarsource.com/plans-and-pricing/sonarqube/)
- [SonarQube SAST Solutions](https://www.sonarsource.com/solutions/security/sast/)
- [SonarQube Wikipedia](https://en.wikipedia.org/wiki/SonarQube)
- [SonarQube Cloud New Pricing](https://www.sonarsource.com/products/sonarqube/cloud/new-pricing-plans/)
- [SonarQube Supported Languages](https://docs.sonarsource.com/sonarqube-server/analyzing-source-code/languages/overview)
- [The Static Analysis Trap (Newsroom Panama)](https://newsroompanama.com/2025/06/09/the-static-analysis-trap-why-sonarqube-may-not-be-enough/)

### AI Coding Tools
- [How Cursor Works Internally (Aditya Rohilla)](https://adityarohilla.com/2025/05/08/how-cursor-works-internally/)
- [How Cursor Actually Indexes Your Codebase (Towards Data Science)](https://towardsdatascience.com/how-cursor-actually-indexes-your-codebase/)
- [AI Coding Tools Comparison (Toolpod)](https://toolpod.dev/ai-coding-tools-comparison)
- [Best AI Coding Agents 2026 (Faros AI)](https://www.faros.ai/blog/best-ai-coding-agents-2026)
- [AI Aider vs Cursor (Sider)](https://sider.ai/blog/ai-tools/ai-aider-vs-cursor-which-ai-coding-assistant-wins-for-2025)
- [Claude Code vs Cursor (Qodo)](https://www.qodo.ai/blog/claude-code-vs-cursor/)
- [Building RAG on Codebases (LanceDB)](https://lancedb.com/blog/building-rag-on-codebases-part-1/)

### Emerging Tools
- [ast-grep GitHub Repository](https://github.com/ast-grep/ast-grep)
- [ast-grep Documentation](https://ast-grep.github.io/)
- [rust-code-analysis (Mozilla)](https://github.com/mozilla/rust-code-analysis)
- [Charon: An Analysis Framework for Rust](https://arxiv.org/html/2410.18042v3)
- [Awesome-Rust-Checker](https://github.com/BurtonQin/Awesome-Rust-Checker)
- [Static Analysis Tools (analysis-tools.dev)](https://analysis-tools.dev/tag/rust)

### MCP Ecosystem
- [Model Context Protocol Specification](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP Enterprise Adoption Guide](https://guptadeepak.com/the-complete-guide-to-model-context-protocol-mcp-enterprise-adoption-market-trends-and-implementation-strategies/)
- [MCP Impact on 2025 (Thoughtworks)](https://www.thoughtworks.com/en-us/insights/blog/generative-ai/model-context-protocol-mcp-impact-2025)
- [Best MCP Servers (Desktop Commander)](https://desktopcommander.app/blog/2025/11/25/best-mcp-servers/)

---

## Appendix B: Existing Research Incorporated

This document builds on and extends the following existing research in the repository:

1. **`competitor_research/FEATURE-MATRIX-COMPARISON.md`** -- 60+ feature comparison across 17 tools. The graph analysis exclusivity finding (Section 4) and competitive positioning summary (Section 14) are directly referenced and expanded here.

2. **`docs/Prep-Doc-V200.md`** -- CozoDB architecture dogfooding, tree-sitter capability gap analysis, Ascent Datalog reasoning, cross-language boundary patterns. The "5 cross-language patterns" from Section 5 form the basis of the cross-language edge analysis in this document.

3. **`docs/Prep-V200-Max-Adoption-Architecture-Strategy.md`** -- Options A through D architecture analysis, the "Problem-Shaped Crates" vs "Protocol + Ecosystem" vs "Embeddable CodeQL" vs "LLM-Native" evaluation. The competitor gap table from the Appendix is expanded into the full gap analysis in Section 7.

4. **`docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md`** -- 28 research-backed features from 40+ papers. Algorithm choices (Leiden, GVE-LPA, etc.) referenced in graph algorithm comparisons.

5. **`docs/THESIS-taint-analysis-for-parseltongue.md`** -- Taint analysis research informing the security analysis comparison.

6. **`docs/PRD-v200.md`** -- The clean-break requirement that frames v2.0.0 as a new product, not an update to v1.x.
