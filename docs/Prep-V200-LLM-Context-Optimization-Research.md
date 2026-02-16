# Prep-V200: LLM Context Optimization Research

**Date**: 2026-02-16
**Purpose**: Deep research for `rust-llm-context` -- the killer crate of Parseltongue v2.0.0. This document examines how existing tools select code context for LLMs, what the academic literature says, how architectural graph signals can be used for ranking, token budgeting algorithms, context quality measurement, and API design.

**Thesis**: Nobody offers architecturally-aware code context selection as an embeddable library. `rust-llm-context` fills that gap.

---

## Table of Contents

1. [How Existing Tools Select Code Context](#1-how-existing-tools-select-code-context)
2. [Academic Research on Code Context for LLMs](#2-academic-research-on-code-context-for-llms)
3. [Architectural Signals for Ranking (Our Unique Advantage)](#3-architectural-signals-for-ranking)
4. [Token Budgeting Algorithms](#4-token-budgeting-algorithms)
5. [How to Measure Context Quality](#5-how-to-measure-context-quality)
6. [API Design for the Crate](#6-api-design-for-the-crate)
7. [Synthesis: What rust-llm-context Must Do](#7-synthesis)

---

## 1. How Existing Tools Select Code Context

### 1.1 Aider: Repo-Map via Tree-Sitter + PageRank

**Source**: [Repository map | aider](https://aider.chat/docs/repomap.html), [Building a better repository map with tree-sitter | aider](https://aider.chat/2023/10/22/repomap.html)

Aider originally used universal ctags but migrated to tree-sitter for richer symbol extraction. Its current pipeline:

1. **Tree-sitter parsing**: Parses all source files into ASTs. Extracts definitions (functions, classes, types) and references (where those symbols are used elsewhere).
2. **Graph construction**: Builds a graph where each source file is a node. Edges connect files that have symbol-level dependencies (file A references a symbol defined in file B).
3. **PageRank ranking**: Applies the PageRank algorithm to this file-dependency graph. Files that are referenced frequently by other important files score higher. The ranking is personalized -- files already in the chat or recently mentioned get a bias boost.
4. **Token-budget-aware selection**: Uses binary search to find the highest-ranked content that fits within the configured token budget (`--map-tokens`, default 1024 tokens). The budget expands dynamically when no files have been added to the chat.
5. **Output format**: A compact textual "repo map" showing file paths and key function/class signatures -- not full implementations, just the structural skeleton.

**Key characteristics**:
- Eschews semantic embeddings entirely. Relies 100% on structural relationships (symbol definitions and references).
- Achieves 4.3-6.5% context utilization efficiency (per a 2025 exploratory study on code retrieval in coding agents), the highest among tested tools.
- The map is **signature-level**, not implementation-level. This is a deliberate choice: showing `fn process_buffer(input: &[u8]) -> Result<Output>` is more token-efficient than showing the full body.
- **Limitation**: File-level granularity. The graph nodes are files, not individual entities. This means a 500-line file with one important function and 20 irrelevant functions gets included or excluded as a unit.
- **Limitation**: No architectural understanding. PageRank captures "what is referenced often" but not "what is architecturally coupled" or "what is in the same module cluster." A utility function used everywhere gets high PageRank but may not be relevant to a specific task.

**What rust-llm-context can do better**: Entity-level granularity instead of file-level. Graph signals beyond PageRank (SCC membership, Leiden community, k-core coreness, blast radius). Cross-language awareness.

---

### 1.2 Cursor: Embeddings + Turbopuffer + AST Chunking

**Source**: [How Cursor Indexes Codebases Fast](https://read.engineerscodex.com/p/how-cursor-indexes-codebases-fast), [How Cursor works -- Deep dive](https://bitpeak.com/how-cursor-works-deep-dive-into-vibe-coding/), [How GitHub Copilot Handles Multi-File Context Internally](https://dzone.com/articles/github-copilot-multi-file-context-internal-architecture)

Cursor's context selection is server-side and embedding-based:

1. **File hashing and sync**: On project open, Cursor computes a Merkle tree of file hashes. Only changed files are uploaded. Files in `.gitignore`/`.cursorignore` are excluded. Every few minutes, the Merkle tree is diffed to detect changes.
2. **AST-aware chunking**: Files are split into semantic chunks using tree-sitter AST parsing. The splitter traverses the AST depth-first, splitting at function/class boundaries. Sibling nodes are merged into larger chunks to avoid excessive fragmentation. Chunks target 100-250 tokens each.
3. **Embedding generation**: Chunks are embedded using OpenAI's embedding API or a custom embedding model, producing 512-dimensional vectors. Embeddings are cached in AWS by chunk content hash for fast re-indexing.
4. **Vector storage**: Embeddings are stored in Turbopuffer, a specialized high-dimensional vector search engine. Each user project gets its own namespace. Metadata includes obfuscated file paths and line ranges.
5. **Query-time retrieval**: When the user prompts, a query embedding is computed. Turbopuffer performs nearest-neighbor search. The server returns file paths and line ranges. The client reads the actual code locally and sends it back to the server for the LLM.
6. **Multi-source context assembly**: Cursor combines vector search results with current file prefix/suffix, open tab contents, symbol resolution via language server/AST traversal, and imported module definitions.

**Key characteristics**:
- Server-dependent architecture. Requires uploading code (even if obfuscated).
- Semantic similarity is the primary ranking signal -- "what looks similar to the query."
- 2026 update: Context-Aware Engine v3.2 introduced dependency tracking to check if changes in one file break dependencies in another.
- **Limitation**: Semantic similarity is not architectural relevance. Two functions can be semantically similar (they both handle HTTP requests) without being architecturally connected (one is in the auth module, the other in payments).
- **Limitation**: No graph-level understanding. Cursor does not know about strongly connected components, coupling metrics, or community structure.

**What rust-llm-context can do better**: Local-first (no server uploads). Architectural signals instead of or in addition to semantic similarity. Understanding of module boundaries and coupling.

---

### 1.3 Continue.dev: RAG Pipeline with Context Providers

**Source**: [Context Providers | Continue](https://docs.continue.dev/customization/context-providers), [How to Build Custom Code RAG - Continue](https://docs.continue.dev/guides/custom-code-rag), [Codebase Indexing | continuedev/continue | DeepWiki](https://deepwiki.com/continuedev/continue/3.4-context-providers)

Continue.dev is an open-source AI code assistant with a modular "context provider" architecture:

1. **Codebase indexing**: Uses three complementary systems:
   - Embedding-based vector search (chunked by tree-sitter AST boundaries, stored in local vector DB)
   - Tree-sitter AST parsing for structural symbol extraction
   - Ripgrep for fast text search
2. **Context provider system**: Modular plugins (three types: "normal", "query", "submenu") that supply context items from diverse sources. Built-in providers include: open files, git diff, terminal output, codebase search, documentation sites. Custom providers can integrate with any backend (internal docs, wikis, vector databases).
3. **DocsContextProvider**: The most sophisticated built-in provider. Crawls documentation sites, chunks content, generates embeddings, stores in a vector database, and enables semantic search over docs.
4. **Custom RAG integration**: Users can build custom RAG providers that connect to their own vector databases and retrieval servers. Supports incremental re-indexing for production use.

**Key characteristics**:
- Open-source and extensible via the context provider plugin system.
- Combines keyword search (ripgrep), structural search (tree-sitter), and semantic search (embeddings) -- but these are independent sources, not fused into a unified ranking.
- **Limitation**: No architectural understanding. The "context" is whatever is textually or semantically similar, not what is architecturally relevant.
- **Limitation**: The RAG pipeline follows standard chunk-embed-retrieve patterns. No graph-based reasoning about code structure.

**What rust-llm-context can do better**: Fuse structural and architectural signals into a single ranking. Provide a context provider that Continue.dev could consume via MCP.

---

### 1.4 Claude Code: Agentic Exploration via Glob/Grep/Read

**Source**: [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works), [Claude Code CHANGELOG](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md)

Claude Code takes a fundamentally different approach -- it does not pre-index the codebase. Instead, it explores on demand:

1. **Agentic loop**: Claude Code operates in three phases: gather context, take action, verify results. These phases blend together dynamically.
2. **Context gathering tools**: The agent uses `glob` (find files by pattern), `grep` (search file contents), and `read` (read specific files) to explore the codebase. It navigates the code the way a developer would -- walking directories, following imports, reading relevant files.
3. **Tree-sitter integration**: Claude Code uses tree-sitter (compiled to WASM) internally for parsing code during sessions. A memory leak fix for tree-sitter parse trees confirms this usage.
4. **MCP extensibility**: External tools can provide additional context via MCP servers. Projects like CodeRLM add tree-sitter-backed symbol indexes for more precise, structured code exploration.
5. **CLAUDE.md context**: Project-level context files provide persistent instructions and architectural knowledge.

**Key characteristics**:
- No pre-built index. Every session starts from scratch (modulo CLAUDE.md and cached context).
- The LLM itself decides what to read -- it acts as a search agent.
- Flexible but token-expensive: the exploration process itself consumes tokens.
- **Limitation**: The agent must "discover" the architecture every session. There is no persistent architectural model.
- **Limitation**: Context selection quality depends entirely on the LLM's ability to navigate. No algorithmic ranking, no graph signals, no guaranteed coverage of architecturally relevant code.

**What rust-llm-context can do better**: Provide a pre-computed architectural model that Claude Code could query via MCP. Instead of "explore and hope," Claude Code could ask "what are the 4K most relevant tokens for changing function X?" and get a ranked, budgeted answer.

---

### 1.5 GitHub Copilot: Neighboring Tabs + Jaccard/Embedding Similarity

**Source**: [copilot-explorer | Hacky repo to see what the Copilot extension sends to the server](https://thakkarparth007.github.io/copilot-explorer/posts/copilot-internals.html), [How Copilot Chat uses context - Visual Studio](https://learn.microsoft.com/en-us/visualstudio/ide/copilot-context-overview), [How GitHub Copilot Handles Multi-File Context Internally](https://dzone.com/articles/github-copilot-multi-file-context-internal-architecture)

Copilot's context selection has evolved through multiple generations:

1. **Original approach (Jaccard similarity)**:
   - Queries the 20 most recently accessed files of the same language from VS Code.
   - Slices each file into sliding windows (60 lines, slide 1 line at a time).
   - Tokenizes and filters out common keywords (`if`, `for`, etc.).
   - Computes Jaccard similarity J(A,B) = |A intersection B| / |A union B| between the current file tokens and each window.
   - Takes the top-N windows with highest similarity.
2. **Priority queue system**: Each context snippet is assigned a priority based on: proximity to cursor, semantic similarity, symbol relevance, recency of access. A time-decay function deprioritizes stale snippets.
3. **Token budget management**: Carefully manages available token budget across prefix (code above cursor), suffix (code below, ~15% of budget via Fill-in-the-Middle), metadata, and neighboring snippets.
4. **Evolution to embeddings**: Copilot now includes "Embeddings Search" -- a local SQLite-backed embedding index. Code chunks (100-250 tokens each) are embedded into 512-dimensional vectors. At search time, query-to-chunk dot product similarity is computed via linear scan (500-2000ms for indexed workspaces).
5. **Instant semantic indexing (2025)**: Reduced indexing time from ~5 minutes to seconds for instant context awareness on repo open.

**Key characteristics**:
- Primarily designed for autocomplete, not chat or agentic editing.
- Same-language filter means it misses cross-language context entirely.
- Multiple signals fused: Jaccard, embeddings, symbol resolution, recency.
- **Limitation**: No architectural understanding. Copilot does not know that a function is tightly coupled to another function in a different file via a dependency chain.
- **Limitation**: Tab-centric. Only considers files the developer has open, not the architecturally relevant files they should have open.

**What rust-llm-context can do better**: Proactively identify architecturally relevant code regardless of what tabs are open. Cross-language awareness. Entity-level rather than file-level context.

---

### 1.6 Cody (Sourcegraph): Code Graph + Search, Replacing Embeddings

**Source**: [How Cody understands your codebase](https://sourcegraph.com/blog/how-cody-understands-your-codebase), [AI-assisted Coding with Cody: Lessons from Context Retrieval](https://arxiv.org/html/2408.05344v1), [Cody Context - Sourcegraph docs](https://sourcegraph.com/docs/cody/core-concepts/context)

Cody is the tool most similar in spirit to what rust-llm-context aims to be, thanks to Sourcegraph's SCIP (code intelligence protocol) and code graph infrastructure:

1. **SCIP-powered code graph**: Sourcegraph's SCIP protocol provides compiler-accurate cross-repository code navigation (go-to-definition, find-references) using Protobuf-based indexing. SCIP replaced the older LSIF format with human-readable symbol IDs and 10-20% smaller index files.
2. **Two-stage retrieval pipeline** (modeled on recommender systems):
   - **Retrieval stage**: Multiple context sources queried in parallel -- local IDE context, Sourcegraph code search, code graph (SCIP symbol relationships). Optimizes for recall.
   - **Ranking stage**: Uses a Repo-level Semantic Graph (RSG) with an "Expand and Refine" method -- graph expansion followed by link prediction to rank context items by relevance. Optimizes for precision.
3. **Evolution away from embeddings**: Cody Enterprise dropped embeddings in favor of Sourcegraph's native search engine. Reasons: security (no code sent to OpenAI for embedding), scalability (search handles 100K+ repos better than vector DB), and simplicity (zero additional config).
4. **Context assembly**: For chat, merges local IDE context (open files, recent tabs) with remote Sourcegraph search results into a global ranking. Takes the top N snippets that fit the token budget. For autocomplete, uses local-only context (active file, open tabs, recently closed tabs) for speed.
5. **@-mentions for explicit context**: Users can explicitly reference repos, files, or docs via @-mentions.

**Key characteristics**:
- Most sophisticated context pipeline of any tool surveyed.
- SCIP provides genuine code graph (symbol-level relationships, not file-level).
- Replaced embeddings with search -- a significant architectural signal that embeddings alone are insufficient.
- Achieves up to 30% autocomplete acceptance rate via "bin packing" optimization of LLM context.
- **Limitation**: SCIP is navigation-focused (go-to-def, find-refs), not architecture-focused. It does not capture coupling metrics, community structure, or architectural layers.
- **Limitation**: Enterprise-only for full capability. The code graph features require Sourcegraph server infrastructure.

**What rust-llm-context can do better**: Architectural signals beyond navigation (SCC, PageRank, Leiden communities, coupling/cohesion). Embeddable as a library, not requiring server infrastructure. Local-first.

---

### 1.7 Amazon Q Developer: Workspace-Local Indexing

**Source**: [Amazon Q Developer's new context features](https://aws.amazon.com/blogs/devops/amazon-q-developers-new-context-features/), [Adding workspace context to Amazon Q Developer chat](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/workspace-context.html)

Amazon Q Developer uses a straightforward workspace-local approach:

1. **Local indexing**: On first `@workspace` trigger, indexes all programming and configuration files. Takes 5-20 minutes for workspaces up to 200MB. Index is persisted to disk, auto-refreshed after 24 hours.
2. **Incremental updates**: After initial indexing, the index is incrementally updated when files are closed or tabs switch.
3. **Memory-aware**: Indexing stops at a hard size limit or when available system memory reaches a minimum threshold. Individual files over 10MB are ignored.
4. **Context transparency**: Since 2025, shows exactly which files were used to formulate each answer.
5. **Project rules**: Markdown files in `.amazonq/rules` enforce coding standards across all developers sharing the project.
6. **MCP support**: The Amazon Q CLI supports MCP for integrating additional tools and data sources.

**Key characteristics**:
- Simple, workspace-local. No cross-repo intelligence.
- 20GB per-customization limit means large codebases must be split.
- **Limitation**: Single-repository only. Cannot aggregate context across multiple repos.
- **Limitation**: No disclosed graph-based or architectural understanding. Standard retrieval approach.

**What rust-llm-context can do better**: Cross-repository and cross-language context. Architectural understanding. Faster indexing via Rust performance.

---

### 1.8 Augment Code: Deep Semantic Indexing + Context Engine

**Source**: [Context Engine | Augment Code](https://www.augmentcode.com/context-engine), [A real-time index for your codebase](https://www.augmentcode.com/blog/a-real-time-index-for-your-codebase-secure-personal-scalable), [Augment's Context Engine is now available for any AI coding agent](https://www.augmentcode.com/blog/context-engine-mcp-now-live)

Augment Code represents the most advanced commercial approach to code context:

1. **Deep semantic indexing**: Not just tokens or grep -- Augment semantically indexes and maps code, understanding relationships between hundreds of thousands of files. Uses custom embedding models (not generic APIs) tailored for code.
2. **Semantic dependency graph**: Indexes the codebase to build a map of dependencies and relationships. Focuses on "context quality" beyond simple RAG.
3. **Real-time personal index**: Processes thousands of files per second. Branch switches handled almost instantly. Each user gets a personal, real-time index.
4. **Context Lineage**: Indexes commit history, summarizing diffs with a lightweight LLM step. Retrieves relevant commits on demand for evolution-aware intelligence.
5. **Infrastructure**: Heavy use of Google Cloud (PubSub, BigTable, AI Hypercomputer). Custom inference stack for GPU-optimized embedding.
6. **MCP integration (February 2026)**: Context Engine MCP brings its semantic search to every MCP-compatible agent. In benchmarks, adding Context Engine improved agent performance by 70%+ across Claude Code, Cursor, and Codex.

**Key characteristics**:
- The closest commercial competitor to what rust-llm-context aims to do.
- Claims 450K-file monorepo indexing in 27 minutes, incremental updates in <20 seconds.
- Context windows up to 200K tokens with architectural awareness across files, services, and layers.
- **Limitation**: Cloud-dependent. Requires Augment's infrastructure (Google Cloud, Turbopuffer).
- **Limitation**: Proprietary. Cannot be embedded as a library in other tools.
- **Limitation**: No published details on what "architectural awareness" means in terms of specific algorithms (SCC? PageRank? Community detection?).

**What rust-llm-context can do better**: Open-source and embeddable. Specific, named algorithms (Tarjan SCC, Leiden community detection, k-core decomposition, PageRank, Shannon entropy, CK metrics). Local-first, zero cloud dependency. Transparent ranking -- you can see exactly why each entity was included.

---

### 1.9 Amp (Sourcegraph): Multi-Agent Architectural Isolation

**Source**: [An Exploratory Study of Code Retrieval Techniques in Coding Agents (Preprints.org, October 2025)](https://www.preprints.org/manuscript/202510.0924), [Ampcode -- A Better Architecture for Coding Agents](https://www.abilshr.com/writing/rebuilding-ampcode)

Amp, built by Sourcegraph, uses a multi-agent architecture with context isolation:

1. **Two-layer delegation**: Main agent identifies retrieval needs and invokes specialized sub-agents (Librarian, Search Agent, Task Runner), each with isolated contexts and dedicated tool access.
2. **Context isolation**: Each sub-agent has its own context window. This prevents pollution between agents and keeps each context focused.
3. **Progressive refinement**: Sub-agents perform broad initial searches that narrow to targeted pattern matching and validation.
4. **`look_at` tool**: Sends files to a separate model with its own context window, returning only requested information. The main agent never processes the full file.

**Key characteristics**:
- Token consumption: 8,500 to 117,000 tokens across tested scenarios -- over an order of magnitude variation, yet all tasks completed.
- Demonstrates that architectural isolation (keeping sub-agent contexts small and focused) is as effective as graph-based ranking (Aider) for efficiency.
- **Limitation**: Architectural isolation is a process optimization, not a knowledge representation. Amp does not build a persistent model of the codebase's architecture.

---

### Summary: Comparative Analysis of Context Selection Approaches

```
Tool            Primary Signal           Granularity   Architecture?  Embeddable?  Local?
-----------     ----------------------   -----------   -------------  ----------   ------
Aider           PageRank on file graph   File-level    No             No (Python)  Yes
Cursor          Embedding similarity     Chunk-level   No             No           No
Continue.dev    RAG (embed+search)       Chunk-level   No             No (TS)      Yes
Claude Code     Agentic exploration      File-level    No             No           Yes
Copilot         Jaccard + embeddings     Window-level  No             No           No
Cody            Code graph + search      Symbol-level  Partial (nav)  No           No
Amazon Q        Local index              File-level    No             No           Yes
Augment Code    Semantic dep graph       Chunk-level   Partial        No           No
Amp             Multi-agent isolation    File-level    No             No           Yes
-----------     ----------------------   -----------   -------------  ----------   ------
rust-llm-context  Graph algorithms       Entity-level  YES            YES          YES
```

**The gap is clear**: No existing tool combines architectural understanding with entity-level granularity in an embeddable, local-first library.

---

## 2. Academic Research on Code Context for LLMs

### 2.1 Repository-Level Code Generation

The field of repository-level code generation has exploded since 2023. Key milestones:

**RepoCoder** (EMNLP 2023): Proposed iterative retrieval-and-generation for repository-level code completion using a similarity-based retriever and pre-trained LM. Foundational work that established the paradigm of retrieving context from the repo to augment generation. ([Paper](https://arxiv.org/abs/2303.12570))

**RepoAgent** (EMNLP 2024): Framework for automated repository-level documentation generation. Uses a three-stage pipeline: Global Structure Analysis (AST parsing + Jedi reference extraction), Documentation Generation, and Documentation Update. Outperformed human-authored documentation in blind preference tests. ([Paper](https://arxiv.org/abs/2402.16667))

**CodePlan** (FSE 2024, Microsoft): Frames repository-level coding as a planning problem. Uses incremental dependency analysis and change may-impact analysis to derive a multi-step chain-of-edits. Key insight: spatial context (arrangement and relationships of code blocks within a codebase) extracted from the dependency graph enables context-aware code modifications. ([Paper](https://dl.acm.org/doi/10.1145/3643757))

**DraCo** (ACL 2024): Dataflow-Guided Retrieval Augmentation for Repository-Level Code Completion. Uses dataflow analysis to guide which code to retrieve as context.

**RLCoder** (ICSE 2025): Uses reinforcement learning to implement repo-level code completion based on RAG methods. The RL agent learns which context to retrieve.

**InlineCoder** (January 2026): Enhances repository understanding by inlining the unfinished function into its call graph. Bidirectional: upstream inlining embeds the function into its callers for usage context; downstream retrieval integrates callees for dependency context. ([Paper](https://arxiv.org/abs/2601.00376))

**Key curated paper lists**:
- [allanj/repo-level-codegen-papers](https://github.com/allanj/repo-level-codegen-papers) -- IDE cross-file information for LLMs
- [YerbaPage/Awesome-Repo-Level-Code-Generation](https://github.com/YerbaPage/Awesome-Repo-Level-Code-Generation) -- must-read papers on repo-level code generation

---

### 2.2 Graph-Based Code Context

The most directly relevant research for rust-llm-context uses graph representations of code for context retrieval:

**GraphCoder** (2024): Uses Code Context Graphs (CCG) that model control-flow, data-flow, and control-dependence between code statements. Employs a coarse-to-fine retrieval process that significantly improved code completion accuracy while reducing retrieval time and storage. Statement-level granularity. ([Paper](https://www.semanticscholar.org/paper/793bbb3c70d4bac3908bd8f2ecc309f1bfdf5f52))

**CodexGraph** (NAACL 2025, NUS/Alibaba): Integrates LLM agents with graph database interfaces extracted from code repositories. Uses a task-agnostic schema to create a universal interface. The primary LLM agent writes natural language queries; a translation agent converts them to graph queries. Achieved 27.9% exact match on CrossCodeEval Lite (Python) and 22.96% Pass@1 on SWE-bench Lite. ([Paper](https://arxiv.org/abs/2408.03910))

**GraphCodeAgent** (2025): Constructs two interconnected graphs -- a Requirement Graph (RG) to model requirement relations between code snippets, and a Structural-Semantic Code Graph (SSCG) to capture code dependencies. An LLM-powered agent performs multi-hop reasoning to retrieve all context including implicit code and multi-hop related snippets. Achieved 43.81% relative improvement with GPT-4o over baselines on DevEval. ([Paper](https://arxiv.org/html/2504.10046))

**Knowledge Graph-Based Repository-Level Code Generation** (May 2025): Transforms code repositories into knowledge graphs capturing structural and relational information. Three-step: (1) build KG from repo, (2) create hybrid search index for sub-graph retrieval, (3) LLM generates code from retrieved sub-graph context. Achieved 36.36% pass@1 with Claude 3.5 Sonnet on EvoCodeBench. ([Paper](https://arxiv.org/html/2505.14394v1))

**Key insight from survey literature**: "Vector-based retrieval offers efficiency but limited structural insight, while graph-based methods better capture dependencies and consistency." ([Retrieval-Augmented Code Generation Survey](https://arxiv.org/html/2510.04905v1))

**Relevance to rust-llm-context**: This research validates our core thesis -- graph-based code context (using dependency graphs, call graphs, knowledge graphs) outperforms similarity-based retrieval. But no existing academic work combines ALL of our graph algorithms (SCC, PageRank, k-core, Leiden, entropy, CK metrics) into a unified ranking. Each paper uses one or two graph signals. rust-llm-context would fuse all of them.

---

### 2.3 SWE-Bench and Code Context Benchmarking

**SWE-bench** (ICLR 2024): Established the baseline for repository-level task evaluation. BM25 retrieval was the initial context strategy -- in ~40% of instances, BM25 retrieves a superset of oracle files for 27K-token context, but in ~50% of instances it retrieves none. ([Paper](https://arxiv.org/pdf/2310.06770))

**SWE-Agent** (NeurIPS 2024): Introduced Agent-Computer Interface (ACI) -- custom tools for agents to interactively explore code rather than relying on static retrieval. GPT-4 Turbo solved 12.47% of SWE-bench test tasks vs. 3.8% for non-interactive RAG. ([Paper](https://proceedings.neurips.cc/paper_files/paper/2024/file/5a7c947568c1b1328ccc5230172e1e7c-Paper-Conference.pdf))

**SWE-Search** (ICLR 2025): Uses Monte Carlo Tree Search for codebase navigation. Action space organized as a two-tier hierarchy (action types: Search, Plan, Edit; specific actions: tool invocations, code modifications). Balances exploitation of high-reward actions with exploration of less-visited states. ([Paper](https://proceedings.iclr.cc/paper_files/paper/2025/file/a1e6783e4d739196cad3336f12d402bf-Paper-Conference.pdf))

**SWE-Pruner** (2025): Self-adaptive context pruning for coding agents. Achieves 39% token reduction on SWE-Bench Verified with Claude Sonnet 4.5 while maintaining or improving success rate (64% vs. 62% baseline). Demonstrates that token-level pruning (LLMLingua2) disrupts code syntax, while coarse-grained retrieval (RAG) misses fine-grained details. SWE-Pruner's line-level pruning hits the sweet spot. ([Paper](https://www.researchgate.net/publication/400071923_SWE-Pruner_Self-Adaptive_Context_Pruning_for_Coding_Agents))

**ContextBench** (February 2026): Benchmark specifically for evaluating context retrieval quality in coding agents. Uses expert-curated "Gold Contexts" -- standard sets of repository artifacts that expert developers identify as necessary to resolve a given issue. Key finding: **sophisticated scaffolding does not necessarily lead to better context retrieval performance**. Block-level F1 scores below 0.45, line-level F1 below 0.35 across state-of-the-art LLMs. Higher recall does not indicate better performance when it comes with excessive noise. ([Paper](https://arxiv.org/html/2602.05892v2))

**Relevance to rust-llm-context**: The ContextBench finding that "more complex retrieval scaffolds do not consistently outperform a simple baseline" is a warning against over-engineering. But it also validates the opportunity -- the best F1 scores are still below 0.45, meaning there is massive room for improvement. Architectural signals (which no existing agent uses) could bridge this gap.

---

### 2.4 Token-Efficient Code Representation

**"Function Signature May Be All That Is Needed"** (ACM TOSEM): Experiments show that function signatures generate, on average, 9.2% more high-quality comments than full code when used as input to code summarization models. Signatures are also dramatically more token-efficient. This validates the hierarchical approach: signatures first, bodies only if budget allows. ([Paper](https://dl.acm.org/doi/10.1145/3652156))

**LongCodeZip** (2025): Code-specific context compression framework with a dual-stage strategy: (1) coarse-grained compression identifies and ranks function-level chunks using conditional perplexity; (2) fine-grained compression segments retained functions into blocks and selects an optimal subset under adaptive token budget. Achieves up to 5.6x compression ratio without degrading task performance. ([Paper](https://arxiv.org/html/2510.00446v1))

**Context Rot** (Chroma Research, 2025): Performance varies significantly as input length changes, even on simple tasks. Models experience 23% performance degradation when context utilization exceeds 85% of maximum capacity. Longer context does not mean better results -- it often means worse results due to distraction and "needle in a haystack" failures. ([Research](https://research.trychroma.com/context-rot))

**JetBrains Research** (NeurIPS 2025 Workshop): Two main approaches for managing coding agent context: LLM summarization (another model generates short summaries) and observation masking (older, less important information is hidden). Both preserve important context; summarization theoretically allows infinite scaling of turns. ([Blog](https://blog.jetbrains.com/research/2025/12/efficient-context-management/))

**Relevance to rust-llm-context**: These findings directly inform our hierarchical token budgeting strategy: (1) Always include type signatures and function headers -- they are the highest-value tokens. (2) Never fill more than ~80% of context capacity. (3) Use compression (function-level ranking by importance) to fit more relevant code. (4) Summaries of less-important entities are better than full implementations of them.

---

### 2.5 The "Static Analysis + RAG" Paradigm

**STALL+** (2024): Demonstrated that combining RAG with static analysis integration strategies (file-level dependencies, post-processing) substantially outperforms RAG alone for repository-level code completion. "It is the first time that static analysis integration has been shown to be superior to RAG in LLM-based repository-level code completion." ([Paper](https://mingwei-liu.github.io/assets/pdf/arxiv2024STALL.pdf))

This is perhaps the single most important finding for rust-llm-context's positioning: **static analysis (our graph algorithms) is proven superior to embeddings-based RAG for code context selection**. We are not adding graph signals to an embedding pipeline. We are building a fundamentally different approach that academic research shows is better.

---

## 3. Architectural Signals for Ranking

This is our unique advantage. No existing tool uses these signals for code context selection. Each signal answers a different question about relevance.

### 3.1 Blast Radius (Dependency Graph Distance)

**What it measures**: How many hops away an entity is from the focus entity in the dependency graph.

**Why it matters for context**: If you are changing function A, and function B is 1-hop away (A calls B or B calls A), then B is almost certainly relevant context. At 2 hops, probably relevant. At 3+ hops, diminishing returns.

**Ranking algorithm**:
```
score(entity) = 1.0 / (1.0 + distance_from_focus)
```
- 0 hops (the focus entity itself): score = 1.0
- 1 hop (direct callers/callees): score = 0.5
- 2 hops: score = 0.33
- 3 hops: score = 0.25

**Unique advantage**: Aider uses PageRank (global importance) but not blast radius (local relevance). A utility function used everywhere gets high PageRank but may be irrelevant to a specific task. Blast radius captures task-specific relevance.

---

### 3.2 SCC Membership (Strongly Connected Components)

**What it measures**: Groups of entities where every entity can reach every other entity via dependency chains. Computed via Tarjan's algorithm.

**Why it matters for context**: If entities A and B are in the same SCC, they are mutually dependent. Changing one without seeing the other risks breaking the circular dependency contract. **You must include the entire SCC or risk the LLM making inconsistent changes.**

**Ranking algorithm**:
```
if focus_entity is in SCC of size > 1:
    include ALL entities in that SCC (mandatory)
    score(scc_member) = 1.0  # same priority as focus entity
```

**Unique advantage**: No existing tool considers SCCs. This is a hard architectural constraint that no amount of semantic similarity can capture. Two functions in the same SCC might have completely different names, parameters, and documentation -- but they are architecturally inseparable.

---

### 3.3 Leiden Community (Module Cluster Membership)

**What it measures**: Groups of entities that are more densely connected to each other than to the rest of the codebase. Detected via the Leiden algorithm (successor to Louvain).

**Why it matters for context**: Entities in the same Leiden community are in the same "architectural module" even if they are in different files or directories. Including entities from the same community as the focus entity provides module-level context.

**Ranking algorithm**:
```
if entity is in same Leiden community as focus:
    score(entity) += 0.3  # community cohesion bonus
```

**Unique advantage**: No existing tool uses community detection for context selection. This captures implicit module boundaries that may not align with file/directory structure.

---

### 3.4 PageRank Score (Architectural Pillars)

**What it measures**: Global importance of an entity in the dependency graph. Entities that are depended upon by many other important entities score higher.

**Why it matters for context**: High-PageRank entities are "architectural pillars" -- the core interfaces, base classes, and utility functions that the entire codebase relies on. An LLM should generally know about these pillars, especially when working on code that depends on them.

**Ranking algorithm**:
```
score(entity) += pagerank(entity) * weight_global_importance
```

**Existing usage**: Aider uses PageRank at file-level granularity. rust-llm-context would use it at entity-level granularity, combined with other signals.

---

### 3.5 K-Core Coreness (Structural Importance)

**What it measures**: The maximum k for which an entity is part of a k-core subgraph (a subgraph where every node has at least k connections). Higher coreness means the entity is part of a more densely connected core.

**Why it matters for context**: High-coreness entities are in the structural core of the codebase -- the tightly interconnected backbone. Low-coreness entities are on the periphery. When token budget is tight, core entities should be prioritized.

**Ranking algorithm**:
```
score(entity) += coreness(entity) / max_coreness * weight_structural
```

**Unique advantage**: K-core decomposition has never been used for code context selection. It provides a different view than PageRank: PageRank measures "important because others depend on it" while k-core measures "important because it's deeply interconnected."

---

### 3.6 Coupling/Cohesion Metrics (CBO, LCOM, RFC, WMC)

**What they measure**: The Chidamber-Kemerer (CK) metrics suite:
- **CBO** (Coupling Between Objects): Number of distinct classes/modules an entity is coupled to
- **LCOM** (Lack of Cohesion in Methods): How cohesive an entity's methods are (higher = less cohesive = more context needed)
- **RFC** (Response For a Class): Number of methods that can be executed in response to a message received
- **WMC** (Weighted Methods per Class): Sum of complexities of methods in an entity

**Why they matter for context**: High CBO means the entity touches many other entities -- more context needed to understand it. High LCOM means the entity is internally fragmented -- more of its internals need to be visible. High RFC means many methods could be triggered -- more callee context needed.

**Ranking algorithm**:
```
context_need(entity) = normalize(CBO * w_cbo + LCOM * w_lcom + RFC * w_rfc + WMC * w_wmc)
// Higher context_need = allocate more token budget to this entity's context
```

**Unique advantage**: CK metrics are well-established in software engineering but have never been applied to LLM context allocation.

---

### 3.7 Cross-Language Edges (Multi-Language Context)

**What they measure**: Connections between entities in different programming languages -- FFI boundaries, WASM exports, gRPC contracts, REST API calls, message queue producers/consumers, PyO3 bindings.

**Why they matter for context**: When changing a Rust FFI function, the LLM needs to see the C header that declares it. When modifying a gRPC service definition, the LLM needs to see both the server implementation (Go) and the client (TypeScript). No existing tool handles this.

**Ranking algorithm**:
```
if focus_entity has cross_language_edge to entity:
    score(entity) += 0.8  # almost as important as direct dependency
    // Cross-language edges represent API contracts -- breaking them is costly
```

**Unique advantage**: Zero competitors offer cross-language edge detection. This is a blue-ocean signal.

---

### 3.8 Entropy Score (Complexity = Needs More Context)

**What it measures**: Shannon entropy of an entity's structure -- how complex and unpredictable its code is.

**Why it matters for context**: High-entropy entities are complex and hard to understand. An LLM modifying a high-entropy function needs MORE context around it (its callers, its callees, its type definitions) to avoid introducing bugs. Low-entropy entities (simple getters, constants) need less context.

**Ranking algorithm**:
```
context_allocation(entity) *= 1.0 + entropy(entity) / max_entropy
// High entropy = allocate up to 2x the normal token budget for this entity's neighborhood
```

**Unique advantage**: Entropy as a context allocation signal has not been explored in any existing tool or academic paper.

---

### 3.9 Unified Scoring Formula

Combining all signals into a single ranking score for each candidate entity:

```
score(entity, focus, task) =
    w_blast   * blast_radius_score(entity, focus)        // local relevance
  + w_scc     * scc_membership_score(entity, focus)      // mandatory co-inclusion
  + w_leiden  * leiden_community_score(entity, focus)     // module cohesion
  + w_pr      * pagerank_score(entity)                    // global importance
  + w_kcore   * kcore_score(entity)                       // structural centrality
  + w_ck      * coupling_need_score(entity)               // complexity-driven need
  + w_cross   * cross_language_score(entity, focus)       // API contract relevance
  + w_entropy * entropy_allocation_factor(entity)         // complexity amplifier
```

Default weights would be tuned empirically, but a reasonable starting point:
```
w_blast   = 0.30  // local relevance is the strongest signal
w_scc     = 0.20  // mandatory co-inclusion is critical
w_leiden  = 0.10  // module cohesion is useful but softer
w_pr      = 0.10  // global importance as tiebreaker
w_kcore   = 0.05  // structural depth as tiebreaker
w_ck      = 0.10  // complexity-driven allocation
w_cross   = 0.10  // cross-language contracts
w_entropy = 0.05  // complexity amplifier
```

---

## 4. Token Budgeting Algorithms

### 4.1 Greedy Allocation (Highest Rank First)

**Algorithm**:
```
1. Rank all candidate entities by unified score (Section 3.9)
2. For each entity in descending score order:
   a. Estimate token cost of including this entity
   b. If cost fits within remaining budget: include, decrement budget
   c. If cost exceeds remaining budget: try next entity (may be smaller)
3. Stop when budget exhausted or no more entities fit
```

**Pros**: Simple, fast, deterministic. Always includes the most relevant entities.
**Cons**: Can be greedy-suboptimal. A large high-scoring entity might crowd out several smaller entities whose combined value exceeds it.

**Research support**: Aider uses a variant of this (PageRank-ranked, binary search for budget fit). The Token-Budget-Aware LLM Reasoning (TALE) paper demonstrates greedy search with feasibility checking can find near-optimal budgets.

---

### 4.2 Proportional Allocation (Budget Per Community/Module)

**Algorithm**:
```
1. Identify Leiden communities containing relevant entities
2. Allocate token budget proportionally:
   - Community containing focus entity: 50% of budget
   - Adjacent communities (1-hop): 30% of budget
   - High-PageRank pillars from other communities: 15% of budget
   - Reserve: 5% for cross-language edges and SCC completions
3. Within each community's budget, apply greedy allocation (4.1)
```

**Pros**: Ensures diverse coverage across architectural modules. Prevents one large, densely-connected module from consuming the entire budget.
**Cons**: More complex. Requires community detection as a prerequisite. May under-allocate to the most relevant community if proportions are wrong.

**Research support**: Dynamic allocation based on information density (DAST paper, 2025) validates that proportional allocation based on content characteristics outperforms uniform allocation. Princeton NLP Group found priority-based context allocation improves task completion by 33%.

---

### 4.3 Hierarchical Allocation (Signatures First, Then Bodies)

**Algorithm**:
```
Phase 1: SIGNATURES (always included)
  For all ranked entities that fit in 40% of budget:
    Include: function signature, type definition, trait/interface declaration
    Format: "fn process_buffer(input: &[u8]) -> Result<Output>"

Phase 2: BODIES (if budget allows)
  For highest-ranked entities in remaining 40% of budget:
    Include: full implementation body
    Priority: focus entity body first, then 1-hop neighbors

Phase 3: ANNOTATIONS (remaining budget)
  For remaining budget:
    Include: relationship annotations
    Format: "// Called by: handle_request, validate_input"
            "// Calls: parse_header, write_response"
            "// Same SCC as: decode_message, encode_response"
```

**Pros**: Maximizes coverage at any budget. Even at 1K tokens, the LLM sees the structural skeleton. At 8K tokens, it sees key implementations. At 32K tokens, it sees everything relevant.
**Cons**: The LLM may need full bodies to understand complex logic. Signatures alone may not capture side effects or invariants.

**Research support**: The "Function Signature May Be All That Is Needed" paper (ACM TOSEM) directly validates Phase 1. Aider's repo map has been using this approach (signatures only) since inception and achieves 4.3-6.5% utilization efficiency. SWE-Pruner (2025) demonstrates that line-level selection (not all-or-nothing) is the sweet spot.

---

### 4.4 Adaptive Compression

**Algorithm**:
```
1. Start with hierarchical allocation (4.3)
2. Measure information density of each included entity:
   - Simple getters/setters: compress to signature only
   - Complex business logic: keep full body
   - Well-documented functions: include doc comment + signature
   - Undocumented complex functions: include full body + callee signatures
3. Re-allocate freed tokens to next-highest-ranked entities
4. Iterate until stable
```

**Pros**: Maximizes information density per token. A 4K token budget contains more architectural knowledge than a naive 16K dump.
**Cons**: Requires an information density metric. The compression decisions themselves consume computation.

**Research support**: LongCodeZip (2025) uses conditional perplexity to rank function-level chunks for compression, achieving 5.6x compression without performance degradation. DAST (2025) uses perplexity for importance and attention for global relevance in dynamic soft token allocation.

---

### 4.5 Recommended Approach: Hybrid (Hierarchical + Proportional)

Combine 4.2 and 4.3:

```
1. Compute unified ranking scores (Section 3.9) for all entities
2. Identify Leiden communities; allocate budget proportionally (4.2)
3. Within each community's allocation, use hierarchical phases (4.3):
   Phase 1: Signatures of all relevant entities
   Phase 2: Bodies of highest-ranked entities
   Phase 3: Relationship annotations for context
4. Cross-community: Always complete SCCs (mandatory)
5. Cross-language: Always include cross-language edge targets
6. Global: Include top PageRank entities as "architectural context"
7. Cap at 80% of stated budget (Context Rot research: >85% degrades performance)
```

---

## 5. How to Measure Context Quality

### 5.1 Task Completion Rate

The gold standard: given a task (e.g., "fix this bug," "add this feature"), does providing context from rust-llm-context lead to higher task completion rates than alternative context strategies?

**Benchmark design**:
```
For each task in benchmark suite:
  1. Provide LLM with context from Strategy A (e.g., rust-llm-context)
  2. Provide same LLM with context from Strategy B (e.g., Aider repo-map)
  3. Provide same LLM with context from Strategy C (e.g., raw file dump)
  4. Measure: Did the LLM produce a correct solution? (test suite validation)
  5. Measure: How many tokens were consumed?
```

**Existing benchmarks to leverage**:
- **ContextBench** (February 2026): Expert-curated Gold Contexts for SWE-bench tasks. Can compare our context selection against Gold Contexts using F1 score. ([Paper](https://arxiv.org/html/2602.05892v2))
- **SWE-bench Verified**: 500 human-verified tasks with ground-truth patches.
- **CrossCodeEval**: Multi-language code completion requiring cross-file context.
- **DevEval**: Emphasizes complex dependencies (GraphCodeAgent achieved 43.81% improvement here).

---

### 5.2 Token Efficiency (Results Per Token)

**Metric**: Task completion rate divided by tokens consumed.

```
efficiency = (tasks_completed / total_tasks) / (avg_tokens_consumed / 1000)
```

Higher is better. A strategy that completes 60% of tasks with 4K tokens is more efficient than one that completes 65% with 32K tokens.

**Research baseline**: Aider achieves 4.3-6.5% utilization efficiency. Amp achieves comparable efficiency via architectural isolation. Most other tools have lower efficiency due to over-provisioning.

---

### 5.3 Precision and Recall Against Gold Contexts

Using ContextBench's Gold Contexts:

```
Precision = |selected_entities intersection gold_entities| / |selected_entities|
Recall    = |selected_entities intersection gold_entities| / |gold_entities|
F1        = 2 * precision * recall / (precision + recall)
```

**Current baselines** (from ContextBench): Block-level F1 below 0.45, line-level F1 below 0.35 across state-of-the-art LLMs and agents. Beating these would be a significant result.

---

### 5.4 Architectural Consistency Score

A novel metric specific to rust-llm-context:

```
For each task requiring multi-entity changes:
  1. Did the context include ALL entities in the same SCC? (SCC coverage)
  2. Did the context include all cross-language edge targets? (cross-lang coverage)
  3. Did the context include the blast radius neighbors? (blast coverage)
  4. Score = weighted average of coverages
```

This measures whether the context would have prevented the LLM from making architecturally inconsistent changes -- something no existing benchmark measures.

---

### 5.5 A/B Testing Protocol

For production evaluation:

```
1. Deploy rust-llm-context as MCP server
2. Connect to Claude Code / Cursor / Aider via MCP
3. Run same tasks with and without rust-llm-context
4. Measure:
   - Task completion time
   - Number of LLM iterations needed
   - Token cost per task
   - Developer satisfaction (subjective)
```

Augment Code reported 70%+ improvement in agent performance when adding their Context Engine via MCP. This is the bar to beat.

---

## 6. API Design for the Crate

### 6.1 Core Input/Output Contract

```rust
/// The primary entry point for context extraction.
pub struct ContextRequest {
    /// Path to the codebase root (or pre-built FactSet)
    pub codebase: CodebaseSource,

    /// The entity or entities that are the focus of the task
    /// e.g., "rust:fn:process_buffer" or "typescript:class:PaymentService"
    pub focus: Vec<EntityKey>,

    /// Maximum number of tokens in the output
    pub token_budget: usize,

    /// Optional: natural language task description for additional ranking
    pub task_description: Option<String>,

    /// Optional: which budgeting strategy to use
    pub strategy: BudgetStrategy,

    /// Optional: weight overrides for ranking signals
    pub weights: Option<RankingWeights>,
}

pub enum CodebaseSource {
    /// Parse from filesystem path
    Path(PathBuf),
    /// Use pre-built analysis (from rust-llm-graph or similar)
    FactSet(FactSet),
    /// Use pre-built analysis from file
    FactSetPath(PathBuf),
}

pub enum BudgetStrategy {
    /// Simple greedy: highest-ranked entities first
    Greedy,
    /// Proportional allocation across Leiden communities
    Proportional,
    /// Signatures first, then bodies, then annotations
    Hierarchical,
    /// Hybrid of proportional + hierarchical (recommended)
    Adaptive,
}
```

```rust
/// The output: a ranked, budgeted, structured context package.
pub struct ContextResponse {
    /// The selected entities, in ranked order
    pub entities: Vec<ContextEntity>,

    /// Total tokens consumed
    pub tokens_used: usize,

    /// Token budget remaining
    pub tokens_remaining: usize,

    /// Why each entity was included (for transparency)
    pub ranking_explanation: Vec<RankingExplanation>,

    /// Relationship annotations
    pub relationships: Vec<Relationship>,

    /// Metadata about the context selection
    pub metadata: ContextMetadata,
}

pub struct ContextEntity {
    /// The entity key (e.g., "rust:fn:process_buffer")
    pub key: EntityKey,

    /// What level of detail is included
    pub detail: DetailLevel,

    /// The actual code content
    pub content: String,

    /// File path and line range
    pub location: SourceLocation,

    /// The entity's ranking score
    pub score: f64,
}

pub enum DetailLevel {
    /// Just the signature: "fn process_buffer(input: &[u8]) -> Result<Output>"
    Signature,
    /// Signature + doc comment
    SignatureWithDocs,
    /// Full implementation body
    FullBody,
    /// Compressed summary (for low-priority entities)
    Summary(String),
}

pub struct RankingExplanation {
    pub entity: EntityKey,
    pub score: f64,
    pub reasons: Vec<RankingReason>,
}

pub enum RankingReason {
    BlastRadius { hops: u32 },
    SameSCC { scc_size: usize },
    SameLeidenCommunity { community_id: usize },
    HighPageRank { rank: f64, percentile: f64 },
    HighKCoreness { coreness: u32 },
    HighCoupling { cbo: f64 },
    CrossLanguageEdge { edge_type: String },
    HighEntropy { entropy: f64 },
    DirectlyReferenced,
    TaskDescriptionMatch { relevance: f64 },
}
```

### 6.2 Simple API (One Function Call)

For users who want the simplest possible interface:

```rust
use rust_llm_context::extract;

// One function call. That's it.
let context = extract("src/", "fn main", 4096)?;

// The context is a structured package ready for LLM consumption
println!("{}", context.to_markdown());
println!("{}", context.to_json());
println!("{}", context.to_prompt_text());
```

This wraps the full pipeline: parse codebase, build graph, compute algorithms, rank entities, allocate budget, produce output.

### 6.3 Builder API (Fine-Grained Control)

For users who need more control:

```rust
use rust_llm_context::{ContextBuilder, BudgetStrategy, RankingWeights};

let context = ContextBuilder::new("src/")
    .focus("rust:fn:process_buffer")
    .focus("rust:fn:handle_request")  // multiple focus entities
    .token_budget(8192)
    .strategy(BudgetStrategy::Adaptive)
    .weights(RankingWeights {
        blast_radius: 0.35,
        scc_membership: 0.25,
        cross_language: 0.15,  // boost cross-lang for polyglot project
        ..Default::default()
    })
    .include_relationship_annotations(true)
    .include_ranking_explanations(true)
    .exclude_test_files(true)
    .build()?;
```

### 6.4 Streaming/Incremental API

For integration with file watchers or long-running servers:

```rust
use rust_llm_context::IncrementalContext;

// Initialize once
let mut ctx = IncrementalContext::new("src/")?;

// On file change, update incrementally
ctx.file_changed("src/handlers/payment.rs")?;

// Query remains fast because the graph is maintained
let context = ctx.extract("fn process_payment", 4096)?;
```

### 6.5 Output Formats

```rust
impl ContextResponse {
    /// Markdown format suitable for LLM prompts
    pub fn to_markdown(&self) -> String;

    /// JSON format for programmatic consumption
    pub fn to_json(&self) -> serde_json::Value;

    /// Plain text with code blocks and annotations
    pub fn to_prompt_text(&self) -> String;

    /// Token count of each format
    pub fn token_count(&self, tokenizer: &Tokenizer) -> usize;

    /// Structured format preserving entity boundaries
    pub fn to_structured(&self) -> Vec<ContextChunk>;
}
```

### 6.6 MCP Integration

rust-llm-context should be exposable as an MCP tool:

```json
{
  "name": "get_code_context",
  "description": "Get the optimal code context for a task within a token budget. Uses architectural graph analysis (dependency graph, SCC, PageRank, community detection) to select the most relevant code entities.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "focus": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Entity keys to focus on (e.g., 'rust:fn:main', 'ts:class:PaymentService')"
      },
      "token_budget": {
        "type": "integer",
        "description": "Maximum tokens in the output (default: 4096)"
      },
      "task": {
        "type": "string",
        "description": "Natural language description of the task for additional relevance ranking"
      },
      "strategy": {
        "type": "string",
        "enum": ["greedy", "proportional", "hierarchical", "adaptive"],
        "description": "Token budgeting strategy (default: adaptive)"
      }
    },
    "required": ["focus"]
  }
}
```

This MCP tool would allow Claude Code, Cursor, or any MCP-compatible agent to query for architecturally-optimal code context without embedding rust-llm-context directly.

### 6.7 Crate Dependencies (Minimal)

```toml
[dependencies]
# Core
tree-sitter = "0.24"            # Parsing (12 language grammars)
serde = { version = "1", features = ["derive"] }  # Serialization

# Graph algorithms (all pure Rust, no C dependencies)
petgraph = "0.6"                # Graph data structures
# OR hand-rolled graph algorithms for zero-dep purity

# Optional
tiktoken-rs = { version = "0.6", optional = true }  # OpenAI tokenizer
tokenizers = { version = "0.20", optional = true }    # HuggingFace tokenizer
serde_json = { version = "1", optional = true }       # JSON output

[features]
default = ["json"]
json = ["dep:serde_json"]
tiktoken = ["dep:tiktoken-rs"]
tokenizers = ["dep:tokenizers"]
```

Key design principle: **zero C/C++ dependencies** in the core path. No CozoDB (RocksDB), no system-level libs. Pure Rust for maximum portability (including WASM compilation for browser/edge use).

---

## 7. Synthesis: What rust-llm-context Must Do {#7-synthesis}

### 7.1 The Competitive Landscape in One Sentence

Every existing tool selects code context using **textual similarity** (embeddings, Jaccard, keyword search) or **navigational structure** (go-to-definition, find-references) -- but **none** uses architectural graph analysis to understand what code is structurally relevant to a task.

### 7.2 The Academic Validation

Research proves that:
- Graph-based code context outperforms similarity-based retrieval (STALL+, GraphCoder, CodexGraph, GraphCodeAgent)
- Function signatures are more token-efficient than full implementations (ACM TOSEM)
- Context quality degrades above 85% capacity (Context Rot)
- Line-level pruning beats both token-level and file-level granularity (SWE-Pruner)
- Current best F1 for context retrieval is below 0.45 (ContextBench) -- massive room for improvement
- Sophisticated scaffolding does not beat simple baselines unless the underlying signals are right

### 7.3 The Unique Value Proposition

rust-llm-context is the first tool to combine:

1. **Eight architectural ranking signals** (blast radius, SCC, Leiden, PageRank, k-core, CK metrics, cross-language edges, entropy) into a unified scoring formula
2. **Entity-level granularity** (not file-level, not chunk-level)
3. **Hierarchical token budgeting** (signatures first, then bodies, then annotations)
4. **Embeddable as a Rust library** (`cargo add rust-llm-context`)
5. **MCP-native** for integration with any AI coding tool
6. **Local-first, zero cloud dependency**
7. **Cross-language awareness** (12 languages, cross-language edges)
8. **Transparent ranking** (every entity comes with an explanation of why it was included)

### 7.4 The One-Line Pitch

> "Your LLM gets 4K tokens of architecturally-relevant code instead of 400K tokens of everything. 99% fewer tokens. Better results."

### 7.5 The Minimum Viable Implementation

For v2.0.0, the minimum viable rust-llm-context needs:

1. **Parse** a codebase into entities and edges (tree-sitter, 12 languages)
2. **Compute** at minimum: blast radius, PageRank, SCC detection
3. **Rank** entities using a unified scoring formula
4. **Budget** tokens using the hierarchical approach (signatures first, then bodies)
5. **Output** structured context in markdown and JSON formats
6. **Expose** as both a Rust library function and an MCP tool

Advanced features (Leiden communities, k-core, CK metrics, entropy, cross-language edges, adaptive compression) can be added incrementally in v2.1+. The core value -- architecturally-aware context selection -- ships with blast radius + PageRank + SCC.

---

## Sources

### Tool Documentation
- [Repository map | aider](https://aider.chat/docs/repomap.html)
- [Building a better repository map with tree-sitter | aider](https://aider.chat/2023/10/22/repomap.html)
- [How Cursor Indexes Codebases Fast - Engineer's Codex](https://read.engineerscodex.com/p/how-cursor-indexes-codebases-fast)
- [How Cursor works -- Deep dive into vibe coding - BitPeak](https://bitpeak.com/how-cursor-works-deep-dive-into-vibe-coding/)
- [Context Providers | Continue](https://docs.continue.dev/customization/context-providers)
- [How to Build Custom Code RAG - Continue](https://docs.continue.dev/guides/custom-code-rag)
- [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works)
- [How Copilot Chat uses context - Microsoft Learn](https://learn.microsoft.com/en-us/visualstudio/ide/copilot-context-overview)
- [copilot-explorer: Copilot internals](https://thakkarparth007.github.io/copilot-explorer/posts/copilot-internals.html)
- [How GitHub Copilot Handles Multi-File Context Internally - DZone](https://dzone.com/articles/github-copilot-multi-file-context-internal-architecture)
- [How Cody understands your codebase - Sourcegraph Blog](https://sourcegraph.com/blog/how-cody-understands-your-codebase)
- [AI-assisted Coding with Cody: Context Retrieval and Evaluation](https://arxiv.org/html/2408.05344v1)
- [Cody Context - Sourcegraph docs](https://sourcegraph.com/docs/cody/core-concepts/context)
- [SCIP Code Intelligence Protocol - Sourcegraph](https://sourcegraph.com/blog/announcing-scip)
- [Amazon Q Developer's new context features - AWS](https://aws.amazon.com/blogs/devops/amazon-q-developers-new-context-features/)
- [Context Engine | Augment Code](https://www.augmentcode.com/context-engine)
- [Augment's Context Engine MCP](https://www.augmentcode.com/blog/context-engine-mcp-now-live)
- [Ampcode -- A Better Architecture for Coding Agents](https://www.abilshr.com/writing/rebuilding-ampcode)

### Academic Papers
- [RepoCoder: Repository-Level Code Completion (EMNLP 2023)](https://arxiv.org/abs/2303.12570)
- [RepoAgent: LLM-Powered Repository-level Documentation (EMNLP 2024)](https://arxiv.org/abs/2402.16667)
- [CodePlan: Repository-Level Coding using LLMs and Planning (FSE 2024)](https://dl.acm.org/doi/10.1145/3643757)
- [GraphCoder: Code Context Graph-based Retrieval (2024)](https://www.semanticscholar.org/paper/793bbb3c70d4bac3908bd8f2ecc309f1bfdf5f52)
- [CodexGraph: LLMs and Code Repositories via Graph Databases (NAACL 2025)](https://arxiv.org/abs/2408.03910)
- [GraphCodeAgent: Dual Graph-Guided LLM Agent (2025)](https://arxiv.org/html/2504.10046)
- [Knowledge Graph Based Repository-Level Code Generation (2025)](https://arxiv.org/html/2505.14394v1)
- [InlineCoder: Context Inlining via Call Graphs (2026)](https://arxiv.org/abs/2601.00376)
- [STALL+: Static Analysis + LLM for Code Completion (2024)](https://mingwei-liu.github.io/assets/pdf/arxiv2024STALL.pdf)
- [Retrieval-Augmented Code Generation Survey (2025)](https://arxiv.org/html/2510.04905v1)
- [SWE-bench (ICLR 2024)](https://arxiv.org/pdf/2310.06770)
- [SWE-Agent (NeurIPS 2024)](https://proceedings.neurips.cc/paper_files/paper/2024/file/5a7c947568c1b1328ccc5230172e1e7c-Paper-Conference.pdf)
- [SWE-Search (ICLR 2025)](https://proceedings.iclr.cc/paper_files/paper/2025/file/a1e6783e4d739196cad3336f12d402bf-Paper-Conference.pdf)
- [SWE-Pruner: Self-Adaptive Context Pruning (2025)](https://www.researchgate.net/publication/400071923_SWE-Pruner_Self-Adaptive_Context_Pruning_for_Coding_Agents)
- [ContextBench: Benchmark for Context Retrieval (February 2026)](https://arxiv.org/html/2602.05892v2)
- [Function Signature May Be All That Is Needed (ACM TOSEM)](https://dl.acm.org/doi/10.1145/3652156)
- [LongCodeZip: Long Context Compression for Code LLMs (2025)](https://arxiv.org/html/2510.00446v1)
- [Context Rot: How Increasing Input Tokens Impacts LLM Performance (Chroma Research)](https://research.trychroma.com/context-rot)
- [Token-Budget-Aware LLM Reasoning (TALE, 2024)](https://arxiv.org/abs/2412.18547)
- [Efficient Context Management for Coding Agents (JetBrains/NeurIPS 2025)](https://blog.jetbrains.com/research/2025/12/efficient-context-management/)
- [An Exploratory Study of Code Retrieval in Coding Agents (October 2025)](https://www.preprints.org/manuscript/202510.0924)

### MCP and Ecosystem
- [Model Context Protocol Specification (November 2025)](https://modelcontextprotocol.io/specification/2025-11-25)
- [A Year of MCP: From Internal Experiment to Industry Standard](https://www.pento.ai/blog/a-year-of-mcp-2025-review)
- [MCP's Next Phase: Inside the November 2025 Specification](https://medium.com/@dave-patten/mcps-next-phase-inside-the-november-2025-specification-49f298502b03)

### Curated Paper Lists
- [allanj/repo-level-codegen-papers](https://github.com/allanj/repo-level-codegen-papers)
- [YerbaPage/Awesome-Repo-Level-Code-Generation](https://github.com/YerbaPage/Awesome-Repo-Level-Code-Generation)
- [codefuse-ai/Awesome-Code-LLM](https://github.com/codefuse-ai/Awesome-Code-LLM)
