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
