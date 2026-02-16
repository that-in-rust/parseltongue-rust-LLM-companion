# Final Priority to add

- SARIF Export
- Taint Analysis
 

Backlog
- Structural Pattern Search - We need workflows first and 4 word names for structural pattern search

 Queries
 - What is Structural Pattern Search?
 





# Raw notes

  Add from CR-v173-03 (P0 — these are missing and should ship in v1.7.3):                                                                                                    
                                                                                               
  1. Surgical Source Extraction (P0-2) — extend /smart-context-token-budget to return actual source code within token budget. Graph-aware: "extract function X + all callees within 4K tokens." Needs tiktoken-rs. This is the feature that directly competes with
  code-scalpel's SurgicalExtractor.               
  2. SARIF Export (P0-3) — serialize Parseltongue's existing analysis results (tech debt, cycles, coupling, SCC) as SARIF JSON. Unlocks GitHub Code Scanning, VS Code SARIF Viewer, CI/CD pipelines. No new analysis logic — just a serialization format for what already
  exists.

  Add from CR-v173-03 (P1 — also v1.7.3 scope):

  3. Graph-Native Taint Analysis (P1-1) — taint tracking via DependencyEdges traversal. New CozoDB relations: TaintSources, TaintSinks, TaintFlows. New tree-sitter .scm patterns. No Z3.
  4. Datalog Policy Engine (P1-2) — policies expressed as CozoDB Datalog queries. "No entity shall have CBO > 50" is a one-liner. No OPA, no Rego.
  5. Structural Pattern Search (P1-3) — predefined graph patterns (hub entities, God classes, dead code) as a parameterized endpoint. Optional ast-grep delegation.

  Add from CR-v173-03 (P2 — also v1.7.3 if it's all in):

  6. Lightweight Telemetry — tracing crate, not full OpenTelemetry. Request latency, error rates, ingestion throughput.
  7. Session Lifecycle Tracking — timestamp-based session detection on top of file watcher. "3 files changed while you were away."
  8. Model-Aware Token Budgets — ?model=claude-sonnet parameter on smart-context endpoint.
  9. Lua Language Support — tree-sitter-lua for 13th language.
  10. Semgrep Annotation Overlay — store semgrep findings as annotations on graph entities.

  Expand "What We're NOT Building" with all 7 P3 items and their rationales (Z3, scheduler, MCP client, ACP adapter, tool orchestrator, multi-adapter LLM, Obsidian search).

  Add a condensed Competitive Priority Matrix — one table showing all 16 features, who has them, PT's position, and LNO rating. This is the strategic backbone.

  Update build order and acceptance criteria to include all the above.

  Should I proceed with writing all of this into the PRD?

❯ ELI5 the changes you are saying from CR-v173-03                                                                                                                                                                                                                             
                                                  
⏺ P0 (highest priority):                                                                                                                                                                                                                                                      
                                                            
  1. Surgical Source Extraction — Right now, /smart-context-token-budget tells the LLM "here are the entity names that fit in your budget." It doesn't give actual code. This change makes it return the real source code, and it's smart about it: "here's function X and the
   3 functions it calls, all fitting in 4,000 tokens." Code-scalpel already does this. We should too, but ours follows the dependency graph.
  2. SARIF Export — SARIF is a standard JSON format that GitHub, VS Code, and CI pipelines all understand for "here are problems in your code." We already compute tech debt scores, circular dependencies, coupling hotspots. This just packages those results in the format 
  GitHub Code Scanning already reads. No new analysis — just a different output format.        
                                                                                                                                                                             
  P1:                                                                                          
                                                                                                                                                                             
  3. Taint Analysis — Track "dirty data" (user input) flowing through the call graph to "dangerous operations" (SQL queries, shell commands). We already have the dependency edges. This walks them looking for paths from input sources to dangerous sinks. If it finds one
  without a sanitizer, it flags a security vulnerability.                                                                                                                    
  4. Policy Engine — Let users write rules like "no function should have more than 50 dependencies" or "no circular dependency allowed." The rules are CozoDB Datalog queries — the same query language we already use. If a rule is violated, report it.
  5. Structural Pattern Search — A new endpoint that answers canned questions: "show me God classes," "show me hub functions," "show me dead code." These are predefined graph queries with parameters.

  P2:

  6. Telemetry — Log how fast endpoints respond, how many errors happen, how long ingestion takes. Basic operational metrics.
  7. Session Tracking — Know when a user "comes back" after being away and tell them what changed.
  8. Model-Aware Budgets — Claude has 200K tokens, GPT-4 has 128K, Gemini has 1M. Let the smart-context endpoint adjust its budget based on which model is consuming the data.
  9. Lua Support — Parse Lua files (13th language). Needed to analyze Neovim plugins.
  10. Semgrep Overlay — If the user has Semgrep installed, run it, take its security findings, and attach them to our graph nodes. Then you can ask "show me security vulnerabilities within 3 hops of user-facing APIs."


===

⏺ Here are all 4 options, dead simple.                                                                                                                                                                                                                                        
                                                                                                                                                                                                                                                                            
  ---                                                                                                                                                                                                                                                                         
  Option A: Cram Everything Into CozoDB                                                                                                                                                                                                                                     
                                                                                                                                                                                                                                                                              
                           ONE DATABASE                                                                                                                                                                                                                                     
                      ┌─────────────────────┐                                                                                                                                
                      │       CozoDB        │                                                  
                      │                     │                                                                                                                                
                      │  CodeGraph (19 col) │ ← entities (existing)
                      │  DependencyEdges    │ ← "Calls"/"Uses" (existing)                                                                                                    
                      │  TypedCallEdges     │ ← NEW: call_kind, via_trait
                      │  TraitImpls         │ ← NEW: who implements what
                      │  SupertraitEdges    │ ← NEW: trait hierarchy
                      │  SemanticTypes      │ ← NEW: return types, params
                      │  TypeLayouts        │ ← NEW: size, padding
                      │  ClosureCaptures    │ ← NEW: what closures grab
                      │                     │
                      │  (5 existing +      │
                      │   6 new = 11 total) │
                      └─────────────────────┘
                                │
                           Datalog queries
                                │
                      ┌─────────────────────┐
                      │    pt08 HTTP API    │
                      └─────────────────────┘

  GOOD:  One database. One query language. Joins across everything.
         "Find edges where call_kind=TraitMethod AND from_key is in SCC"
         = one Datalog query.

  BAD:   CozoDB stores everything as String/Int/Float/Bool.
         "return_type" = "Result<User, AuthError>" is just a string.
         You CANNOT query "find functions where the error type is AuthError"
         because CozoDB doesn't understand type structure.
         You get: WHERE return_type LIKE '%AuthError%'  ← string matching, not type matching.

         CozoDB is DORMANT. Last release mid-2023. Issues piling up.
         Adding 6 more relations = betting harder on an abandoned project.

         Rust-only data in a 12-language schema. 11 of 12 languages
         leave the 6 new relations empty. Schema bloat.

  WHO PICKS THIS: Someone who wants simplicity and is OK with string-matching
                  for type queries. The "good enough" choice.

  ---
  Option B: Two CozoDBs (Syntax + Semantic)

          SYNTAX DATABASE                    SEMANTIC DATABASE
      ┌───────────────────┐              ┌───────────────────┐
      │   CozoDB #1       │              │   CozoDB #2       │
      │                   │              │                   │
      │  CodeGraph        │              │  TypedCallEdges   │
      │  DependencyEdges  │              │  TraitImpls       │
      │  TestEntities     │              │  SupertraitEdges  │
      │  FileWordCoverage │              │  SemanticTypes    │
      │  IgnoredFiles     │              │  TypeLayouts      │
      │                   │              │  ClosureCaptures  │
      │  ALL 12 languages │              │  RUST ONLY        │
      └────────┬──────────┘              └────────┬──────────┘
               │                                  │
               └──────────┬───────────────────────┘
                          │
                pt08 merges at HTTP layer
                          │
                ┌─────────────────────┐
                │    pt08 HTTP API    │
                └─────────────────────┘

  GOOD:  Clean separation. Syntax schema stays clean for all 12 languages.
         Semantic schema is Rust-optimized. Each DB has its own schema
         designed for its data shape.

  BAD:   TWO CozoDB instances. Double the memory for index overhead.
         Cross-DB joins are MANUAL (app code, not Datalog).
         "Blast radius where edges are TraitMethod" = query DB #1 for
         blast radius, query DB #2 for edge types, merge in Rust code.
         Loses Datalog's join power.

         STILL has the string-matching problem for type queries.
         CozoDB #2 is still CozoDB — same limitations, same dormancy risk.

  WHO PICKS THIS: Someone who wants clean separation but doesn't mind
                  CozoDB's type query limitations. Rare choice.

  ---
  Option C: Keep rust-analyzer Alive (Query Salsa Directly)

      ┌───────────────────┐          ┌──────────────────────────┐
      │   CozoDB          │          │   rust-analyzer           │
      │                   │          │   (salsa database)        │
      │  CodeGraph        │          │                          │
      │  DependencyEdges  │          │  FULL type resolution    │
      │  (existing 5)     │          │  FULL trait hierarchy    │
      │                   │          │  FULL closure analysis   │
      │  Graph traversal  │          │  FULL visibility         │
      │  (SCC, PageRank,  │          │  FULL generic bounds     │
      │   Leiden, etc.)   │          │  FULL type layouts       │
      │                   │          │                          │
      │                   │          │  Incremental updates     │
      │                   │          │  (salsa handles this     │
      │                   │          │   natively)              │
      └────────┬──────────┘          └────────┬─────────────────┘
               │                              │
               └──────────┬───────────────────┘
                          │
                pt08 queries BOTH
                          │
                ┌─────────────────────┐
                │    pt08 HTTP API    │
                └─────────────────────┘

  GOOD:  FULL FIDELITY. No lossy conversion to strings or flat tables.
         rust-analyzer already computed everything — why copy it into
         another database? Just ASK it.

         The compiler is the database.

         "What's the fully resolved return type of authenticate()?"
         → salsa already knows. Zero conversion. 100% accurate.

         "What does this closure capture?"
         → salsa already analyzed it. No approximation.

         Incremental updates are FREE. Edit a file, salsa recomputes
         only what changed. No re-ingestion pipeline.

  BAD:   rust-analyzer holds the ENTIRE Cargo workspace in memory.
         200-500MB for a medium project. ON TOP of CozoDB.
         pt08 becomes a memory hog.

         salsa's API is designed for IDE queries, not bulk analysis.
         "Give me ALL call edges with dispatch kinds" isn't a first-class
         operation. You'd iterate every function, every call site.

         rust-analyzer's internal API is UNSTABLE. It refactors regularly.
         You're pinning to a git commit and maintaining a soft fork.

         Snapshot export (pt02) can't serialize salsa's state.
         You lose portability.

  WHO PICKS THIS: Someone who wants 100% compiler fidelity and is willing
                  to pay the memory cost + API instability cost.
                  The "purist" choice.

  ---
  Option D: CozoDB + Custom Rust Store + Ascent

      ┌───────────────────┐          ┌──────────────────────────┐
      │   CozoDB          │          │  TypedAnalysisStore       │
      │   (UNCHANGED)     │          │  (pure Rust HashMaps)     │
      │                   │          │                          │
      │  CodeGraph        │          │  entity_metadata         │
      │  DependencyEdges  │          │    HashMap<Key, Info>    │
      │  (existing 5)     │          │    - return_type         │
      │                   │          │    - params              │
      │  Datalog queries  │          │    - visibility          │
      │  Graph algorithms │          │    - async/unsafe        │
      │                   │          │    - generic bounds      │
      │                   │          │    - closure captures    │
      │                   │          │    - type layouts        │
      │                   │          │                          │
      │                   │          │  typed_edges             │
      │                   │          │    HashMap<(From,To), Info>│
      │                   │          │    - call_kind           │
      │                   │          │    - via_trait            │
      │                   │          │    - receiver_type       │
      │                   │          │                          │
      │                   │          │  INDEXES (O(1) lookup):  │
      │                   │          │    by_return_type        │
      │                   │          │    by_visibility         │
      │                   │          │    by_trait_bound        │
      │                   │          │    by_call_kind          │
      │                   │          │    async_entities        │
      │                   │          │    unsafe_entities       │
      │                   │          │                          │
      │                   │          │  + Ascent (Datalog proc  │
      │                   │          │    macro for recursive   │
      │                   │          │    trait hierarchy)       │
      └────────┬──────────┘          └────────┬─────────────────┘
               │                              │
               └──────────┬───────────────────┘
                          │
                pt08 merges at HTTP layer
                          │
                ┌─────────────────────┐
                │    pt08 HTTP API    │
                └─────────────────────┘

  GOOD:  CozoDB untouched. Zero regression risk on 22 existing endpoints.
         Type queries are sub-microsecond (HashMap index lookup).
         Ascent gives native Datalog for trait hierarchy recursion.
         Pure Rust. No FFI. ~0.5MB dependency. MessagePack serializable.

  BAD:   Two data stores to keep in sync.
         Can't write ONE query that joins graph traversal + type filters.
         "Blast radius where edges are TraitMethod" still requires:
           1. CozoDB → blast radius entities
           2. TypedAnalysisStore → filter by call_kind
           3. Merge in Rust
         Manual index maintenance (insert = update all indexes).

  WHO PICKS THIS: Someone who wants incremental progress with low risk.
                  The "pragmatic" choice.

  ---
  NOW: Forget Pragmatism. What's Maximum Leverage?

  You said forget quick launch. You're the primary user. You want max leverage.

  Here's what I actually think:

  THE ANSWER IS NONE OF THE ABOVE.

  The answer is: OPTION E.

  Option E: DuckDB Replaces CozoDB Entirely

                        ONE DATABASE (replaces CozoDB)
                      ┌─────────────────────────────────────┐
                      │            DuckDB                    │
                      │                                     │
                      │  TABLES (SQL):                      │
                      │    entities        ← CodeGraph      │
                      │    edges           ← DependencyEdges│
                      │    typed_edges     ← call_kind etc  │
                      │    trait_impls     ← who impls what │
                      │    supertrait_edges                 │
                      │    semantic_types  ← return types   │
                      │    closure_captures                 │
                      │    type_layouts                     │
                      │                                     │
                      │  QUERY POWER:                       │
                      │    SQL for EVERYTHING               │
                      │    Recursive CTEs (trait hierarchy)  │
                      │    DuckPGQ (graph pattern matching)  │
                      │    JSON functions (nested data)      │
                      │    LIKE/REGEX (type search)          │
                      │    Columnar (fast analytics)         │
                      │    Vectorized execution              │
                      │    Parallel                          │
                      │                                     │
                      │  PERSISTENCE:                       │
                      │    In-memory (fast)                  │
                      │    File-backed (portable)            │
                      │    Parquet export (ecosystem)        │
                      │                                     │
                      │  MAINTAINED:                        │
                      │    EXTREMELY active (v1.4.4, 2026)  │
                      │    Massive community                │
                      │    Official Rust bindings            │
                      └─────────────────┬───────────────────┘
                                        │
                                    SQL queries
                                        │
                      ┌─────────────────────────────────────┐
                      │           pt08 HTTP API             │
                      │                                     │
                      │  Graph algos: KEEP in pure Rust     │
                      │  (Tarjan, Leiden, PageRank, etc.)   │
                      │  Fed by SQL SELECT instead of       │
                      │  CozoDB Datalog                     │
                      └─────────────────────────────────────┘

  Why Option E is Maximum Leverage

  WHAT YOU GET:

  1. ONE QUERY for typed blast radius:
     WITH RECURSIVE blast AS (
       SELECT to_key, 1 AS hops FROM typed_edges WHERE from_key = ?
       UNION
       SELECT e.to_key, b.hops + 1
       FROM blast b JOIN typed_edges e ON b.to_key = e.from_key
       WHERE b.hops < ?
     )
     SELECT b.*, t.call_kind, t.via_trait
     FROM blast b
     JOIN typed_edges t ON b.to_key = t.to_key
     WHERE t.call_kind = 'TraitMethod'

     ← CozoDB + sidecar Option D can't do this in one query.

  2. TYPE-AWARE SEARCH (not string matching):
     SELECT * FROM semantic_types
     WHERE return_type LIKE 'Result<%,AuthError>'
     AND visibility = 'pub'
     AND is_async = true

     ← Real SQL columns, real indexes, real query optimizer.

  3. GRAPH PATTERN MATCHING via DuckPGQ:
     SELECT * FROM GRAPH_TABLE(dependency_graph
       MATCH (a)-[e:typed_edges WHERE e.call_kind = 'TraitMethod']->(b)
       COLUMNS(a.name, b.name, e.via_trait)
     )

     ← Cypher-style graph queries ON TOP of relational data.

  4. ANALYTICS for free:
     SELECT call_kind, COUNT(*), AVG(hops)
     FROM typed_edges
     GROUP BY call_kind
     ORDER BY COUNT(*) DESC

     ← DuckDB is an analytical database. Aggregations are instant.

  5. PARQUET EXPORT:
     COPY entities TO 'entities.parquet' (FORMAT PARQUET);
     COPY typed_edges TO 'typed_edges.parquet' (FORMAT PARQUET);

     ← Your snapshot format becomes industry-standard Parquet.
     ← Any data tool (Python pandas, Polars, Spark) can read it.
     ← pt02 becomes trivial.

  6. FUTURE-PROOF:
     DuckDB is one of the most actively developed databases (2026).
     CozoDB is dormant since mid-2023.
     DuckDB has a massive ecosystem. CozoDB has... us.

  What You Lose

  MIGRATION COST:

  1. Rewrite 22 HTTP handlers from CozoDB Datalog → SQL
     - Not hard, but not trivial. ~2-3 days of focused work.
     - Every raw_query("?[x] := *CodeGraph{...}") becomes
       "SELECT x FROM entities WHERE ..."

  2. Graph algorithms stay in pure Rust (already are).
     - Currently fed by CozoDB query → Vec<(String, String, String)>
     - Would be fed by DuckDB query → Vec<(String, String, String)>
     - The interface is the same. The algorithms don't care.

  3. Binary size: +50MB from DuckDB C++ core via FFI.
     - CozoDB is ~15MB. DuckDB bundled is ~50MB+.
     - For a CLI tool, this matters. For a server, it doesn't.

  4. FFI boundary:
     - CozoDB is pure Rust. DuckDB is C++ with Rust bindings.
     - Slightly more friction for error handling, slightly more
       risk of FFI bugs. In practice, duckdb-rs is solid.

  The Honest Comparison

                      CozoDB          DuckDB
                      (current)       (Option E)
                      ─────────       ──────────
  Maintained?         DORMANT         EXTREMELY ACTIVE
  Type queries?       String blobs    SQL columns + indexes
  Graph queries?      Datalog (good)  DuckPGQ (good) + recursive CTE
  Recursive?          Datalog (10/10) CTE (7/10)
  Analytics?          Basic           Vectorized columnar (10/10)
  Ecosystem?          Tiny            Massive (Parquet, Arrow, etc.)
  Portability?        RocksDB/SQLite  File/Parquet/In-memory
  Binary size?        ~15MB           ~50MB+
  Pure Rust?          Yes             No (C++ FFI)
  Graph algos?        Built-in some   None (but ours are pure Rust anyway)
  Community?          ~dead           Thriving
  Risk?               Abandonment     None (backed by DuckDB Labs)

  My Honest Take

  If you're the primary user and you want maximum leverage for the LONG TERM:

  Option E (DuckDB) gives you the most powerful query engine, the best ecosystem integration, the most active maintenance, and eliminates the CozoDB dormancy risk.

  The migration cost is real (~2-3 days to rewrite handlers from Datalog to SQL). But you pay it once. Then every future feature — typed edges, trait hierarchy, type search, visibility audit, async chain tracing, Parquet export — is just another SQL table and another
  SQL query.

  Option D (CozoDB + Custom Store + Ascent) is the safe incremental choice. Zero migration risk, zero regression risk. But you end up with TWO data stores, manual index maintenance, and you're still betting on a dormant database for your core graph queries.

  The question is: do you want to pay the migration cost NOW and have a single powerful store forever, or avoid the migration and manage two stores indefinitely?

  For maximum leverage: Option E.

  ==

  100x SWE
Automated Pull Request Generation System with AI-Powered Code Intelligence

Project Overview
This system automates the entire pull request workflow by combining TypeScript AST parsing, hybrid search algorithms, and LangGraph orchestration to generate, validate, and test code changes with minimal manual intervention [memory:3][memory:4].

The architecture leverages vector embeddings, BM25 keyword indexing, and reciprocal rank fusion to intelligently retrieve relevant code files, while E2B sandboxes provide isolated testing environments with automated rollback capabilities.

System Architecture
Frontend
Next.js + TypeScript interface for monitoring PR generation, viewing logs, and managing GitHub integrations

Vercel
Backend
Node.js + Express API handling webhooks, authentication, and orchestrating the entire PR workflow

DigitalOcean
Worker
Redis-based queue processor executing long-running code generation and validation tasks asynchronously

DigitalOcean
Core Features
1. TypeScript AST Parsing
Parses TypeScript source code into Abstract Syntax Trees to understand code structure, dependencies, and relationships between functions, classes, and modules.

Tech Stack:
TypeScript Compiler API
ts-morph
AST Traversal
Use Cases:
▪
Extract function signatures and dependencies
▪
Identify import/export relationships
▪
Generate code skeletons for token efficiency
▪
Map cross-file references and call graphs
2. Hybrid Search & Retrieval
Combines BM25 keyword-based search with vector embeddings to retrieve the most relevant code files for any given task or issue.

Tech Stack:
BM25 Algorithm
Vector Embeddings
Reciprocal Rank Fusion
Pinecone/Vector DB
Use Cases:
▪
Find relevant files when user describes a feature
▪
Retrieve semantically similar code patterns
▪
Balance keyword matching with semantic understanding
▪
Reduce context window by selecting only relevant files
3. LangGraph Orchestration
Uses LangGraph to orchestrate multi-step validation workflows with parallel consistency checks, retry logic, and state management.

Tech Stack:
LangGraph
LangChain
State Machines
Parallel Execution
Use Cases:
▪
Validate code changes across multiple files simultaneously
▪
Check for breaking changes and type errors
▪
Coordinate between code generation and testing phases
▪
Implement retry strategies with exponential backoff
4. E2B Sandbox Testing
Executes generated code in isolated E2B sandbox environments to run tests, validate functionality, and ensure no regressions before creating PRs.

Tech Stack:
E2B SDK
Docker Containers
Automated Testing
Rollback Mechanisms
Use Cases:
▪
Run unit and integration tests in isolation
▪
Validate code changes don't break existing functionality
▪
Automatically rollback failed changes
▪
Capture test outputs and error logs for debugging
5. GitHub Integration
Deep integration with GitHub API for webhooks, OAuth authentication, repository cloning, and automated PR creation with detailed descriptions.

Tech Stack:
GitHub App API
Webhooks
OAuth 2.0
Octokit SDK
Use Cases:
▪
Listen to push events and issue comments
▪
Authenticate users and clone private repositories
▪
Create PRs with AI-generated code and descriptions
▪
Update PR status based on validation results
End-to-End Workflow
1
Trigger Event
User creates an issue or comments on a PR describing the desired code change

2
Webhook Reception
GitHub webhook fires to backend, validated via signature verification and queued in Redis

3
Code Indexing
Worker clones repository, parses all TypeScript files into ASTs, generates embeddings and BM25 index

4
Intelligent Retrieval
Hybrid search uses reciprocal rank fusion to find top-k most relevant files based on user request

5
Code Generation
LLM receives code skeletons (compressed ASTs) + context to generate proposed changes with minimal tokens

6
LangGraph Validation
Parallel validation checks: type consistency, breaking changes, cross-file dependencies

7
Sandbox Testing
E2B sandbox executes tests on generated code; rollback if tests fail

8
PR Creation
If all validations pass, create GitHub PR with AI-generated description and link to issue

Technology Stack
Backend
Node.js + Express
TypeScript
Prisma ORM
PostgreSQL
Redis + BullMQ
JWT Authentication
AI & ML
LangGraph
LangChain
OpenAI / Gemini API
Vector Embeddings
BM25 Search
Reciprocal Rank Fusion
Frontend
Next.js 14+
React
TypeScript
Tailwind CSS
Lucide Icons
DevOps & Testing
E2B Sandbox
Docker
DigitalOcean
Vercel
GitHub Actions
Daytona
Database Schema
User
id
String (UUID)
Primary key, unique user identifier
email
String
User email from GitHub OAuth
username
String
GitHub username
githubId
String
GitHub user ID for API calls
accessToken
String (encrypted)
GitHub OAuth token for repo access
createdAt
DateTime
Account creation timestamp
Repository
id
String (UUID)
Primary key
userId
String (FK)
Foreign key to User
repoName
String
Full repo name (owner/repo)
installationId
String
GitHub App installation ID
indexed
Boolean
Whether codebase has been indexed
lastIndexedAt
DateTime?
Last indexing timestamp
Session
id
String (UUID)
Primary key
userId
String (FK)
Foreign key to User
token
String
Session token hash
expiresAt
DateTime
Session expiration time
createdAt
DateTime
Session creation timestamp
CodeIndex
id
String (UUID)
Primary key
repoId
String (FK)
Foreign key to Repository
filePath
String
Relative path to file in repo
embedding
Vector
Vector embedding of file content
bm25Score
Float
BM25 relevance score
astHash
String
Hash of AST for change detection
API Endpoints
GET
/health
Health check endpoint returning server status

{ "status": "ok", "timestamp": "ISO-8601" }

POST
/github-webhook
Unified webhook handler for GitHub events (push, PR, issues)

Auth: GitHub Signature Verification

GET
/auth/login
Initiates GitHub OAuth flow

Redirects to GitHub authorization page

GET
/auth/callback
OAuth callback handling token exchange and session creation

JWT token + session cookie

POST
/auth/logout
Invalidates session and clears authentication

Auth: JWT Required

POST
/api/index-repo
Manually trigger repository indexing

Auth: JWT Required

GET
/api/repos
List all repositories for authenticated user

Auth: JWT Required

POST
/installation/created
Handle GitHub App installation events

Auth: GitHub Signature

Key Optimizations
Token Efficiency
Code skeleton compression reduces context by 70-80%
Only sends function signatures and type definitions to LLM
Hybrid search limits files to top-k most relevant
Incremental indexing only processes changed files
Performance
Redis queue prevents webhook timeouts
Parallel validation with LangGraph workers
Prisma connection pooling for database efficiency
Vector index caching for fast retrieval
Reliability
E2B sandbox isolation prevents code injection
Automated rollback on test failures
Exponential backoff retry logic
Webhook signature verification prevents spoofing
Scalability
Stateless backend enables horizontal scaling
Worker processes can scale independently
Database indexes on frequently queried fields
CDN deployment for frontend (Vercel)
User Journey Example
1
Install GitHub App
User installs the GitHub App on their repository, granting access to read code and create PRs

2
Create Issue
User creates an issue: "Add input validation to user registration endpoint"

3
Automatic Processing
System indexes the repository (if not already done), identifies relevant files using hybrid search, and generates validation code

4
Validation & Testing
LangGraph validates the generated code for type safety and cross-file consistency, then E2B sandbox runs all tests

5
PR Creation
System creates a PR with the validation code, detailed description, and links back to the original issue

6
Review & Merge
User reviews the AI-generated code, requests changes if needed, and merges when satisfied

Future Enhancements
Multi-language Support
Extend AST parsing to Python, Java, Go, and Rust codebases

Conversational Refinement
Allow users to iterate on generated code through chat interface

Cost Optimization
Implement caching layer for repeated queries and code patterns

Analytics Dashboard
Track PR success rates, token usage, and code generation metrics

Custom Validation Rules
Let users define project-specific linting and validation rules

Multi-agent Collaboration
Deploy specialized agents for testing, documentation, and refactoring

Built with TypeScript, LangGraph, and Next.js | Deployed on DigitalOcean & Vercel

