# CR-codex-v200-implications.md
# Phase 4: Competitor Findings → V200 Contract Implications
# Date: 2026-02-19
# Sources: CR07 (Codex analysis), CR05 (Droid decompilation), ZAI-PRD-contracts-02-FINAL

---

## 0. Governing Insight

> **Codex and Droid confirm that V200's architecture direction is correct —
> 8-crate clean DAG, MCP-first, graph-computed intelligence. What they reveal
> is WHERE to invest deeper: structured MCP responses (R4), deterministic
> quality scoring (graph-reasoning), and agent-aware context ranking (get_context).**

---

## 1. Contract-by-Contract Impact Map

### 1.1 Architecture Contracts (Crate Structure)

```
+==============================================================================+
|  V200 CRATE                    COMPETITOR EVIDENCE              IMPLICATION  |
+==============================================================================+
|                                                                              |
|  rust-llm-core-foundation      Codex: 15,901 entities across   CONFIRMED    |
|  (EntityKey + contracts)        44 crates with zero circular    Key format   |
|                                 deps. Their key format is       must handle  |
|                                 `rust:fn:name:file:hash`.       Codex-scale  |
|                                 15K+ entities = stress test     (15K+) with  |
|                                 for any key scheme.             zero         |
|                                                                 collisions   |
|                                 Codex entity: `rust:enum:       |||          |
|                                 SandboxCommand:____codex_rs_    delimiter    |
|                                 cli_src_main:T1881323720`       handles this |
|                                 → Long keys with path+hash.    → CF-P1-A    |
|                                                                 probe must   |
|                                                                 include      |
|                                                                 15K+ entity  |
|                                                                 corpus       |
|  --------------------------------------------------------------------------  |
|  rust-llm-tree-extractor       Codex: 5 languages detected     CONFIRMED    |
|  (12-language parsing)          (Rust, TS, JS, Py, C).          V200 covers  |
|                                 Codex ingestion: 60.96%         all 5 + 7    |
|                                 coverage (854/1401 files).      more. Must   |
|                                 Key gap: 85 integration         handle test  |
|                                 test files failed parsing.      files better |
|                                                                 than Codex's |
|                                 Droid: embedded ripgrep for     60% coverage |
|                                 text search only. Zero AST.     → TE-P4-C   |
|                                 This is V200's advantage.       degrade      |
|                                                                 visibility   |
|  --------------------------------------------------------------------------  |
|  rust-llm-rust-semantics       Codex: 1,400 `expect()` calls   NEW SIGNAL   |
|  (rust-analyzer bridge)         = unhandled panics. RA would    V200's RA    |
|                                 resolve all of these to real    bridge can   |
|                                 types. 14,324 Rust entities     detect       |
|                                 = heavy RA workload.            expect()-    |
|                                                                 as-tech-debt |
|                                 Codex's God Object pattern      pattern.     |
|                                 (CBO=1124) would be detected    → RS-P3-E   |
|                                 as coupling anomaly by RA       resource     |
|                                 type resolution.                 envelope    |
|                                                                 must handle  |
|                                                                 14K+ entity  |
|                                                                 codebases    |
|  --------------------------------------------------------------------------  |
|  rust-llm-cross-boundaries     Codex: 5 languages in one       CONFIRMED    |
|  (cross-language edges)         repo (Rust↔C via bubblewrap,    FFI/C edges  |
|                                 Rust↔TS via codex-cli,          are real in  |
|                                 Rust↔Py via SDK).               production   |
|                                 386 C entities vendored.        codebases.   |
|                                                                              |
|                                 Droid: Rust↔JS boundary         CB-P5-A     |
|                                 (Bun runtime bridge).           precision    |
|                                 Not analyzable (closed).        probe should |
|                                                                 include      |
|                                 Both confirm: real-world        Codex-style  |
|                                 codebases mix languages.        vendor/C     |
|                                                                 pattern      |
|  --------------------------------------------------------------------------  |
|  rust-llm-graph-reasoning      Codex: zero circular deps       CONFIRMED    |
|  (Ascent Datalog)               confirms SCC detection works.   + EXPANDED   |
|                                 God Object rule would flag      |
|                                 codex.rs (CBO=1124).            Datalog rule |
|                                 Dead code rule would find       `god_object` |
|                                 unreachable entities.           validated by |
|                                                                 real-world   |
|                                 Droid ByteRank: 8/11 cats       finding.     |
|                                 replaceable by Datalog rules:   |
|                                 • dead_code → dead_code(f)      ByteRank     |
|                                 • god_object → god_object(f)    replacement  |
|                                 • circular_dep → SCC rule       = NEW market |
|                                 • coupling → cbo(m)             for GR-P6    |
|                                 • testing → untested_pub_fn     rules.       |
|                                 • reachability → reachable()    |
|                                 • layer_violation → boundary    Phase 1 rule |
|                                 • unsafe_chain → unsafe_chain   selection    |
|                                                                 confirmed:   |
|                                                                 SCC, dead    |
|                                                                 code, god    |
|                                                                 object,      |
|                                                                 reachable    |
|  --------------------------------------------------------------------------  |
|  rust-llm-store-runtime        Codex: 136,130 edges in graph.  STRESS TEST  |
|  (graph persistence)            Leiden: 9,887 communities.      V200 store   |
|                                 PageRank: 15K+ entities.        must handle  |
|                                                                 136K+ edges  |
|                                 This is the scale V200 must     without      |
|                                 handle for real-world repos.    degradation. |
|                                                                 → SR-P2-A   |
|                                 Codex k-core max: 33 layers.    bounded      |
|                                 V200 has 8 crates → shallower   query guard  |
|                                 by design. But store must       must handle  |
|                                 support deep k-core queries.    k=33 depth   |
|  --------------------------------------------------------------------------  |
|  rust-llm-interface-gateway    Codex: MCP server + client.      CONFIRMED    |
|  (HTTP/MCP transport)           Approval via sampling/           MCP-first   |
|                                 createMessage protocol.          is correct. |
|                                                                              |
|                                 Codex SSE streaming in           V200 R4     |
|                                 k-core innermost ring (33).      (XML-tagged |
|                                 Streaming is architectural       responses)  |
|                                 bedrock for them.                confirmed   |
|                                                                 as needed.  |
|                                 Droid: MCP client only.                      |
|                                 Cannot be consumed by other      V200 as MCP |
|                                 agents. V200 avoids this         server can  |
|                                 limitation by being MCP-first.   serve BOTH  |
|                                                                 Codex AND   |
|                                                                 Droid.      |
|  --------------------------------------------------------------------------  |
|  rust-llm-test-harness         Codex: 85 test files failed      V200 test   |
|  (contract gates)               ingestion. Coverage gaps in      harness     |
|                                 network-proxy and rmcp-client.   must NOT    |
|                                                                 have this   |
|                                 V200's G4 (path normalization)   gap. G4     |
|                                 would prevent Codex's coverage   directly    |
|                                 counting problem.                prevents    |
|                                                                 the Codex   |
|                                                                 coverage    |
|                                                                 failure.    |
+==============================================================================+
```

---

## 2. Non-Negotiable Gate Validation

```
+==============================================================================+
|  GATE   COMPETITOR EVIDENCE                      STATUS                      |
+==============================================================================+
|                                                                              |
|  G1     Codex entity keys contain file hashes     REINFORCED                 |
|  Slim   (e.g., :T1881323720). V200's |||          Codex keys prove that      |
|  Types  delimiter must handle 15K+ entities        discriminators are needed  |
|         without collision. Codex's key length      for real codebases.        |
|         averages ~80 chars. V200 must be           V200's discriminator       |
|         comparable or shorter.                     field is validated.        |
|                                                                              |
|  G2     Codex: ToolOrchestrator::run() has        REINFORCED                 |
|  Single CBO=28 and blast radius of 21.9%.         Single getter contract     |
|  Getter If V200 had multiple read paths, a         prevents the blast-radius |
|         change to any one would cascade like       problem Codex has with     |
|         Codex's orchestrator. G2 prevents this.    ToolOrchestrator::run().   |
|                                                                              |
|  G3     Codex: 554 files failed parsing.          REINFORCED                 |
|  FS     Missing files returned silently (no        V200's explicit error      |
|  Read   explicit error contract in Codex graph).   contract for missing/      |
|         V200 must NEVER silently fail on reads.    moved files prevents the   |
|                                                    silent coverage gaps.      |
|                                                                              |
|  G4     Codex: ingestion coverage 60.96%.         REINFORCED                 |
|  Path   Path variants likely contribute to         Codex's coverage issues    |
|  Norm   undercounting. Codex stores paths with     validate G4 as essential.  |
|         `./` prefix variations in graph.           V200 normalizes first.     |
|                                                                              |
+==============================================================================+

ALL 4 GATES REINFORCED BY COMPETITOR EVIDENCE.
No gates need to be changed. Competitors validate the need for each one.
```

---

## 3. Promoted Requirements (R1-R8) Validation

```
+==============================================================================+
|  REQ   COMPETITOR EVIDENCE                      IMPACT                       |
+==============================================================================+
|                                                                              |
|  R1    Codex: 44 crates, each with own           CONFIRMED                   |
|  Route namespace. V200's route prefix nesting     Multi-project support       |
|  Prefix allows analyzing multiple repos             validated. V200 could      |
|        (e.g., Codex on /codex/ + Droid on         analyze BOTH competitors    |
|        /droid/) simultaneously.                    from one server instance.   |
|                                                                              |
|  R2    Codex: runs on port 7780 for this          CONFIRMED                   |
|  Auto  session. V200 must auto-assign ports        Port file discovery is      |
|  Port  to avoid conflicts when analyzing           essential for multi-repo    |
|        multiple repos concurrently.                analysis sessions.          |
|                                                                              |
|  R3    N/A — no direct competitor evidence.        NO CHANGE                   |
|  Shut                                                                         |
|  down                                                                         |
|                                                                              |
|  R4    Codex: MCP server uses structured          EXPANDED                    |
|  XML   JSON-RPC responses with thread IDs.         V200 XML-tagged responses  |
|  Tags  Droid: structured handoff JSON with         should include:            |
|        salientSummary, verification,               • provenance (where data   |
|        tests, discoveredIssues.                      came from)               |
|                                                    • confidence (how certain  |
|        BOTH competitors have structured              the analysis is)         |
|        response formats for agents.                • verification (what to    |
|        V200's R4 must match or exceed.               check to confirm)        |
|                                                                              |
|        Droid's handoff format is the gold          ADD to GW-P7-J probe:     |
|        standard for agent-consumable output.       handoff-style fields in    |
|                                                    XML-tagged responses.      |
|                                                                              |
|  R5    Codex: project slug would be "codex"       CONFIRMED                   |
|  Slug  for this analysis session. V200's slug      Slug-in-URL makes multi-   |
|        in URL makes it self-describing.            project analysis clear.     |
|                                                                              |
|  R6    N/A — extends R2/R5.                        NO CHANGE                   |
|  Slug                                                                         |
|  Port                                                                         |
|                                                                              |
|  R7    Codex: 15,901 entities. Smart context      CRITICAL                    |
|  Token must rank by token cost. Without real        Token counts are the       |
|  Count token counts per entity, get_context         foundation of get_context  |
|        can't budget properly. Codex's scale         ranking. Codex-scale       |
|        (15K entities) makes this non-optional.      repos make this P0.        |
|                                                                              |
|  R8    Codex: data flow edges not in current      CONFIRMED                   |
|  Data  Parseltongue graph. V200's R8 adds          ToolOrchestrator::run()    |
|  Flow  assign/param/return flow edges that          data flow would reveal     |
|        would reveal HOW Codex's god object          which parameters carry     |
|        propagates data through 1,124 deps.          taint through the hub.     |
|                                                                              |
+==============================================================================+

R4 IS THE ONE REQUIREMENT THAT NEEDS EXPANSION based on competitor evidence.
All others are confirmed as-is.
```

---

## 4. The get_context Killer Feature — Competitor Validation

```
+==============================================================================+
|                 get_context vs COMPETITOR APPROACHES                          |
+==============================================================================+
|                                                                              |
|  TOOL             CONTEXT METHOD              TOKEN EFFICIENCY               |
|  ===============  ========================    ======================         |
|  Codex            view_file → raw content     0% optimization                |
|                   (full file, no ranking)      (sends everything)             |
|                                                                              |
|  Droid            view_file → raw content     0% optimization                |
|                   grep_tool → text matches     (text search, no graph)        |
|                   ByteRank → LLM evaluation   ~5K tokens/check               |
|                                                                              |
|  V200 get_context Ranked entities by 8        99% reduction                  |
|                   signals within token         (2-5K vs 500K)                 |
|                   budget:                                                     |
|                   • Blast Radius (0.30)                                       |
|                   • SCC Membership (0.20)                                     |
|                   • Leiden Community (0.10)                                    |
|                   • PageRank (0.10)                                           |
|                   • CK Metrics (0.10)                                         |
|                   • Cross-Language (0.10)                                     |
|                   • K-Core (0.05)                                             |
|                   • Entropy (0.05)                                            |
|                                                                              |
+==============================================================================+

VALIDATION FROM CODEX ANALYSIS:

  If get_context were called with entity="ToolOrchestrator::run" tokens=4096:

  1. Blast Radius (0.30): 3,489 entities at 2 hops.
     get_context would RANK these by importance, not dump all 3,489.

  2. SCC Membership (0.20): Codex has zero SCCs > size 1.
     get_context would skip SCC co-inclusion (nothing to co-include).

  3. Leiden Community (0.10): Community containing ToolOrchestrator
     would get 50% of community budget.

  4. PageRank (0.10): Top entities are unresolved stdlib.
     get_context would filter to RESOLVED entities only.

  5. CK Metrics (0.10): CBO=28 → high coupling → include more
     of ToolOrchestrator's dependencies in context.

  6. K-Core (0.05): codex.rs is in k-core 33 (innermost).
     get_context would boost entities in same core layer.

  RESULT: Instead of sending the entire codex.rs (~6000 lines)
  to the LLM, get_context would send the ~20 most architecturally
  relevant entities (signatures + key bodies) within 4096 tokens.

  THIS IS V200's MOAT. Neither Codex nor Droid can do this.
```

---

## 5. ByteRank Replacement: Datalog Rules as Deterministic Alternative

```
+==============================================================================+
|  BYTERANK CATEGORY        DROID METHOD          V200 ALTERNATIVE             |
+==============================================================================+
|                                                                              |
|  code_modularization      LLM reads code,       Leiden communities +         |
|                           judges modularity      CBO/LCOM/RFC/WMC metrics   |
|                           ~5K tokens/check       O(1) graph lookup           |
|                                                  → GR-P6 rule: cbo(m)       |
|                                                                              |
|  cyclomatic_complexity    LLM reads function,    Shannon entropy +           |
|                           estimates complexity    branch count from AST      |
|                           Non-deterministic       Deterministic              |
|                                                  → /entropy-complexity       |
|                                                                              |
|  dead_code_detection      LLM reads imports,     Datalog: dead_code(f)      |
|                           guesses unused          Graph reachability from    |
|                           Misses transitive       entry points. Catches      |
|                                                   transitive dead code.      |
|                                                  → GR-P6 Phase 1 rule       |
|                                                                              |
|  duplicate_detection      LLM compares files     Entity similarity in       |
|                           Expensive, slow         graph (shared callees)     |
|                                                  → DEFERRED to v210         |
|                                                                              |
|  naming_consistency       LLM reads names,       Entity naming pattern      |
|                           checks conventions      analysis (regex on keys)   |
|                                                  → DEFERRED to v210         |
|                                                                              |
|  tech_debt_tracking       LLM looks for TODOs    SQALE scoring (ISO 25010)  |
|                           and code smells         CBO + LCOM + RFC + WMC    |
|                                                  → /technical-debt-sqale    |
|                                                                              |
|  test_coverage            LLM checks test files  untested_pub_fn(f) rule    |
|                           exist                   Test entity → code entity  |
|                                                   mapping in graph           |
|                                                  → GR-P6 Phase 2 rule       |
|                                                                              |
|  dependency_health        LLM reviews deps       SCC + PageRank + k-core    |
|                           Misses transitive       Full transitive analysis   |
|                                                  → /strongly-connected +    |
|                                                    /centrality-measures +    |
|                                                    /kcore-decomposition     |
|                                                                              |
+==============================================================================+

COVERAGE: 8 of 11 ByteRank categories have V200 alternatives.
REMAINING 3: deployment_frequency, dev_environment, observability
→ These are OPERATIONAL metrics, not code metrics. Outside V200 scope.

COMPETITIVE CLAIM:
┌──────────────────────────────────────────────────────────────────────┐
│ "V200 provides deterministic code quality scoring that replaces     │
│  LLM-evaluated metrics. Same categories as Factory Droid's          │
│  ByteRank, but faster (milliseconds vs seconds), cheaper (zero      │
│  LLM cost), and reproducible (same input = same output)."          │
│                                                                      │
│  This is a LEGITIMATE competitive claim backed by graph analysis.   │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 6. MCP Integration Strategy — Informed by Competitors

```
+==============================================================================+
|                V200 MCP POSITIONING (post-competitor analysis)                |
+==============================================================================+
|                                                                              |
|  CODEX MCP:                                                                  |
|  ┌────────────────────────────────────────┐                                 |
|  │ SERVER: Exposes tools to other agents   │                                 |
|  │ CLIENT: Consumes tools from servers     │                                 |
|  │ APPROVAL: sampling/createMessage        │                                 |
|  │ TRANSPORT: stdio + streamable HTTP      │                                 |
|  └────────────────────────────────────────┘                                 |
|                                                                              |
|  DROID MCP:                                                                  |
|  ┌────────────────────────────────────────┐                                 |
|  │ CLIENT ONLY: Consumes from Braintrust,  │                                 |
|  │ Fireflies, custom servers               │                                 |
|  │ Cannot be consumed by other agents      │                                 |
|  └────────────────────────────────────────┘                                 |
|                                                                              |
|  V200 MCP:                                                                   |
|  ┌────────────────────────────────────────┐                                 |
|  │ SERVER: Exposes analysis tools          │                                 |
|  │ • get_context (killer feature)          │                                 |
|  │ • blast_radius                          │                                 |
|  │ • coupling_metrics                      │                                 |
|  │ • scc_analysis                          │                                 |
|  │ • entity_search                         │                                 |
|  │ • forward/reverse callees               │                                 |
|  │                                         │                                 |
|  │ NOT A CLIENT: V200 provides data,       │                                 |
|  │ not consumes tools. This is correct.    │                                 |
|  └────────────────────────────────────────┘                                 |
|                                                                              |
|  INTEGRATION TOPOLOGY:                                                       |
|                                                                              |
|       ┌───────────────┐                                                      |
|       │  V200 (MCP    │                                                      |
|       │  server)      │                                                      |
|       │               │                                                      |
|       │  get_context   │                                                      |
|       │  blast_radius  │                                                      |
|       │  scc_analysis  │                                                      |
|       │  coupling      │                                                      |
|       └───────┬───────┘                                                      |
|               │ MCP                                                           |
|       ┌───────┼───────┐                                                      |
|       │               │                                                      |
|       ▼               ▼                                                      |
|  ┌─────────┐    ┌─────────┐                                                 |
|  │ CODEX   │    │ DROID   │                                                 |
|  │ (server │    │ (client │                                                 |
|  │ +client)│    │  only)  │                                                 |
|  │         │    │         │                                                 |
|  │ Can     │    │ Cannot  │                                                 |
|  │ RELAY   │    │ relay   │                                                 |
|  │ V200    │    │ V200    │                                                 |
|  │ data to │    │ data    │                                                 |
|  │ other   │    │         │                                                 |
|  │ agents  │    │         │                                                 |
|  └─────────┘    └─────────┘                                                 |
|                                                                              |
|  V200 ADVANTAGE: Serving both competitors from one MCP server.               |
|  CODEX BONUS: Codex can relay V200 data to downstream agents.                |
|  DROID LIMIT: Droid consumes but cannot re-serve.                            |
|                                                                              |
+==============================================================================+

LNO ALIGNMENT:
• Phase 1 (LEVERAGE): HTTP-only server with 22 endpoints → CONFIRMED
  Codex analysis was done entirely via HTTP. HTTP works.
• Phase 2 (NEUTRAL): MCP stdio transport → CONFIRMED
  Codex uses stdio for MCP. Match this.
• Phase 3 (OVERHEAD): Streamable HTTP MCP → DEFER
  Codex's SSE is in k-core 33 but V200 doesn't need it yet.
```

---

## 7. Architecture Anti-Pattern Warning: God Object

```
+==============================================================================+
|             WARNING FROM CODEX: THE GOD OBJECT TRAP                          |
+==============================================================================+
|                                                                              |
|  CODEX's codex.rs:                                                           |
|  • CBO = 1,124 (couples to 1,124 other entities)                            |
|  • LCOM = 1.0 (zero internal cohesion)                                       |
|  • SQALE debt = 14 hours remediation                                         |
|  • Health grade = F                                                          |
|  • Blast radius of ToolOrchestrator::run() = 21.9% of codebase             |
|                                                                              |
|  HOW THIS HAPPENED:                                                          |
|  codex.rs is the "main agent loop" that touches every subsystem:             |
|  sandbox, MCP, tools, approval, config, streaming, session, auth.           |
|  It grew organically as features were added.                                 |
|                                                                              |
|  V200 MUST PREVENT THIS:                                                     |
|  ┌──────────────────────────────────────────────────────────────────┐        |
|  │                                                                  │        |
|  │  V200 has 8 crates. The equivalent of codex.rs would be         │        |
|  │  rust-llm-interface-gateway becoming a God crate that            │        |
|  │  imports from all 7 others.                                      │        |
|  │                                                                  │        |
|  │  PREVENTION:                                                     │        |
|  │  1. G2 (single getter contract) prevents store-runtime          │        |
|  │     from becoming a dependency magnet                            │        |
|  │  2. Each crate has a defined PUBLIC INTERFACE (Section 2.3)     │        |
|  │  3. interface-gateway depends on 5 crates → watch CBO           │        |
|  │  4. If gateway CBO exceeds 50, split into sub-modules           │        |
|  │                                                                  │        |
|  │  MONITORING:                                                     │        |
|  │  Run V200's own coupling-cohesion-metrics on itself:             │        |
|  │  curl localhost:7777/coupling-cohesion-metrics-suite?            │        |
|  │    entity=rust:module:interface_gateway                           │        |
|  │  If CBO > 50 → refactor before shipping.                        │        |
|  │                                                                  │        |
|  └──────────────────────────────────────────────────────────────────┘        |
|                                                                              |
|  CODEX GOD OBJECT PROGRESSION (reconstructed from graph):                    |
|                                                                              |
|  codex.rs CBO growth probably looked like:                                   |
|  v0.1:  CBO ~50   (basic agent loop)                                        |
|  v0.5:  CBO ~200  (+ sandbox, + tools)                                      |
|  v0.8:  CBO ~500  (+ MCP, + multi-agent)                                    |
|  v1.0:  CBO 1124  (+ streaming, + config, + auth, + hooks)                  |
|                                                                              |
|  Each feature added ~100-200 CBO. By the time they noticed,                 |
|  it was too expensive to refactor. V200 monitors from day one.              |
|                                                                              |
+==============================================================================+
```

---

## 8. Datalog Rule Priority — Confirmed by Competitor Evidence

```
+==============================================================================+
|  RULE               PHASE   COMPETITOR EVIDENCE              PRIORITY        |
+==============================================================================+
|                                                                              |
|  SCC / circular_dep  P1     Codex: 0 cycles across 44       CONFIRMED P1    |
|                              crates. V200 detects when       Existing repos  |
|                              codebases are NOT this clean.   need this.      |
|                                                                              |
|  dead_code           P1     Codex: 60.96% coverage means    CONFIRMED P1    |
|                              39% of files may contain        ByteRank's      |
|                              dead code. V200 finds it        dead_code is    |
|                              via reachability.               LLM; V200 is    |
|                                                              graph.          |
|                                                                              |
|  god_object          P1     Codex: codex.rs CBO=1124.       CONFIRMED P1    |
|                              V200's god_object rule would    POSTER CHILD    |
|                              flag this immediately.          for the rule.   |
|                              chatwidget.rs CBO=1075.                         |
|                              msg_processor CBO=929.                          |
|                              3 god objects in one codebase.                   |
|                                                                              |
|  reachable           P1     Codex: blast radius of          CONFIRMED P1    |
|                              ToolOrchestrator::run() =       Reachability    |
|                              3,489 entities (21.9%).         underlies       |
|                              Reachability is the engine.     blast radius.   |
|                                                                              |
|  unsafe_chain        P1     Codex: bubblewrap C code has    CONFIRMED P1    |
|                              `acquire_privs`, `drop_all_     Real unsafe     |
|                              caps` — security-critical       chains in       |
|                              functions. V200 traces these.   production.     |
|                                                                              |
|  taint_analysis      P2     Droid: risk assessment per      CONFIRMED P2    |
|                              shell command (low/med/high).   Taint traces    |
|                              V200 provides GRAPH-based       data from user  |
|                              taint from user input to        input to        |
|                              security-sensitive sinks.       dangerous ops.  |
|                                                                              |
|  layer_violation     P2     Codex: clean 44-crate DAG       CONFIRMED P2    |
|                              with zero circular deps.        Detects when    |
|                              V200 detects VIOLATIONS of      crate boundaries|
|                              this discipline.                are breached.   |
|                                                                              |
|  untested_pub_fn     P2     Codex: 85 test files failed     CONFIRMED P2    |
|                              ingestion. V200 finds public    Coverage gaps   |
|                              functions without test callers. are real.       |
|                                                                              |
+==============================================================================+

PHASE 1 RULES (5): SCC, dead_code, god_object, reachable, unsafe_chain
ALL 5 CONFIRMED by competitor evidence. No changes needed to Q9 recommendation.
```

---

## 9. New Implications (Not in Original PRD)

### 9.1 Structured Agent Response Format (NEW — from Droid handoffs)

```
WHEN V200 MCP tool returns analysis results
THEN SHALL include structured metadata:
  provenance: { algorithm, parameters, timestamp }
  confidence: { score: f64, basis: string }
  verification: { commands: [string], expected: [string] }
AND SHALL format for agent consumption (not human reading)
```

**Why**: Droid's handoff JSON proves that agents need structured metadata, not just raw data. V200's MCP responses should be self-verifiable.

**Contract link**: Extends R4 (XML-tagged responses) with agent-specific fields.

### 9.2 Multi-Project Concurrent Analysis (NEW — from Codex+Droid research)

```
WHEN V200 server runs with multiple project slugs
THEN SHALL support concurrent analysis of competing codebases
AND SHALL enable cross-project comparison queries
```

**Why**: This CR07 research required running Parseltongue on Codex (port 7780) while the main server runs on 7777. Multi-project is a real workflow.

**Contract link**: Extends R1 (route prefix nesting) + R5 (project slug).

### 9.3 CBO Monitoring Threshold Alert (NEW — from God Object finding)

```
WHEN V200 ingests a codebase and computes CBO metrics
THEN SHALL flag entities with CBO > threshold (default: 50)
AND SHALL include these in /complexity-hotspots-ranking-view
AND SHALL expose as god_object Datalog rule output
```

**Why**: Codex's CBO=1124 god object was likely not detected until it was too expensive to fix. V200 should warn early.

**Contract link**: New probe for GR-P6 (graph-reasoning).

---

## 10. Priority Matrix (Updated with Competitor Evidence)

```
+==============================================================================+
|                  V200 PRIORITY MATRIX (POST-COMPETITOR)                       |
+==============================================================================+
|                                                                              |
|  P0 — MUST SHIP (blocked by competitor reality)                              |
|  ================================================                            |
|  • get_context MCP tool with 8 ranking signals (Section 4)                  |
|  • Deterministic quality scoring (ByteRank replacement) (Section 5)         |
|  • R7 token count at ingest (for get_context budgeting)                     |
|  • R4 XML-tagged responses with provenance/confidence (Section 9.1)         |
|  • Phase 1 Datalog rules: SCC, dead_code, god_object, reachable,           |
|    unsafe_chain (Section 8)                                                  |
|  • G1-G4 gates (all reinforced by competitor evidence) (Section 2)          |
|                                                                              |
|  P1 — SHOULD SHIP (competitive advantage)                                    |
|  ============================================                                |
|  • MCP stdio transport (Codex uses stdio for MCP)                           |
|  • R8 data-flow edges (reveals taint paths through god objects)             |
|  • Multi-project slug support (R1 + R5, validated by research workflow)     |
|  • Cross-language edge detection (Codex has Rust↔C↔TS↔Py in one repo)     |
|                                                                              |
|  P2 — NICE TO HAVE (differentiation polish)                                  |
|  =============================================                               |
|  • Streamable HTTP MCP (Codex SSE in k-core 33, but HTTP works)            |
|  • Phase 2 Datalog rules: taint, layer_violation, untested_pub_fn          |
|  • CBO monitoring threshold alerts (Section 9.3)                            |
|  • Agent handoff format documentation for Codex/Droid integration           |
|                                                                              |
|  P3 — DEFER (no competitor urgency)                                          |
|  ====================================                                        |
|  • TypeLayouts, ClosureCaptures as separate endpoints                       |
|  • Duplicate detection (ByteRank category, low priority)                    |
|  • Naming consistency analysis (ByteRank category, low priority)            |
|  • context-packer (deferred to v210 per PRD)                                |
|                                                                              |
+==============================================================================+
```

---

## 11. One-Liner Takeaways

1. **All 4 non-negotiable gates (G1-G4) are reinforced by competitor evidence.** No changes needed.
2. **R4 (XML-tagged responses) needs expansion** to include provenance/confidence/verification fields.
3. **get_context is V200's moat.** Neither Codex nor Droid has ranked, token-budgeted context.
4. **8 of 11 ByteRank categories are replaceable** by V200's graph algorithms. Legitimate competitive claim.
5. **Phase 1 Datalog rules (5) are all confirmed** by real-world findings in Codex's codebase.
6. **God Object warning is V200's poster child use case.** codex.rs CBO=1124 is the demo.
7. **V200 as MCP server serves BOTH Codex and Droid.** Neither can replace V200's role.
8. **HTTP-first (Phase 1) is correct.** This entire research was done via HTTP on port 7780.
9. **Multi-project support is a real workflow need.** Running analysis on multiple repos simultaneously.
10. **V200's 8-crate architecture is shallower than Codex's 44-crate DAG by design.** Monitor CBO to stay clean.

---

*Generated 2026-02-19. Phase 4: V200 Contract Implications from Competitor Analysis.*
*Sources: ZAI-PRD-contracts-02-FINAL.md (970 lines), CR07/CR-codex-*.md (2,504 lines), CR05/factory-droid/ (decompiled binary).*
*Parseltongue server: port 7780, 15,901 entities, 136,130 edges.*
