# Parseltongue v0.9.7 & Scope Document

**Date**: 2025-11-07
**Status**: Draft - Planning Phase
**Previous Version**: v0.9.6 (Test Exclusion + Single Binary)
**Target**: Feature enhancement release

===

# For v097
- [ ] Binary change : toon export should be a default part of the json export
   - [ ] working for each and every we're looking for each and every command which exports to JSON. As long as it exports to JSON automatically, it should export to .toon as well
- [ ] Agent Change: SEARCH ONLY GRAPH not the actual codebase
   - [ ] 0 Pure ISG-Native: Successfully eliminated grep fallback
      - [ ] Clear Rules: Strong guardrails against filesystem tools
   - [ ] Pyramid level strategy, you increase depth of the map as needed like a pyramid   - [ ] ⚠️ Gaps
- [ ] As soon ingestion happens a lot of visuals which insightful should appear from analytics of
   - Dependency Graph
   - Code Graph
   - TEST DATA WILL NOT BE INGESTED at the ingestion phase itself
- [ ] As soon as ingestion happens, we should have a parseltongue folder in root
   - [ ] Main folder with all cozoDB and json or toon files for the root git folder
      - Main folder will refresh each time codebase is committed
   - [ ] Subfolder with all gitclones and research documents with cozoDBs and research MD,txts, jsons, toons etc.
- [] reasoningBetterEnhancements
   - [ ] FIX current dependency graph, L01, L02 as base only for reading, and reset only when code is commited
   - 2 Stage system per commit
      - [ ] Stage 1: the protector of previous truth when micro-PRD was thought of 
      - [ ] Stage 2: the explorer of convergence with micro-PRD
         - [ ] Alway query Stage for what exists in dependency graphs in base
         - [ ] As soon as it makes a change in the code, it will reingest the whole dependency graph L01 and L02 in Stage 2 area
         - [ ] Dependency Graphs of Stage 1 & Stage 2 will be compared AND Stage 2 which is current codebase situation will be compiled, and tested by humans
            - If it successfully compiles and tests then it will be moved then make a commit and Stage 1 is reingested
            - If it fails then
               - make a json of failures
               - make a json of success
               - make a json of possible useful patterns
               - after this move the head of code back to Stage 1 commit and reingest it
            - go back to solving the problem from commit-base but with learnings of previous attempts

# For v098

- [ ] Clustering enhancements
   - [ ] Semantic Clustering, clustering based

# For v099

- [ ] Flow enhancements
   - [ ] Control Flow
   - [ ] Data Flow



# For v100

# long term backlog
- [ ] Next-Leap Ideas
   - [ ] Sub-agent driven LLM-meta-data for each interface
      - [ ] In natural language what does this function do
   - [ ] Sub-agent driven LLM-meta-data for each interface-edge-interface
      - [ ] In natural language this fn to fn interaction mean something
   - [ ] Sub-agent driven LLM-meta-data for each interface-edge-interface-edge-interface
      - [ ] In natural language this fn to fn interaction mean something
   - 



# PRD to implementation flow using Parseltongue

5 agents
- Agent01: parsentongue-ultrathink-isg-explorer-PRD-evaluator
- Agent02: parseltongue-ultrathink-isg-explorer-PRD-




``` mermaid
graph TD
   Agent[**Parseltongue Agent**] --> Ingestion[**Ingestion of Codebase** <br> Will exclude all test interfaces]
   Ingestion --> DependencyGraph[**LLMs reads Dependency Graph** <br> via expor to json or toon using pt02]
   DependencyGraph --> GraphType1[**Type 1 interface-relationship graph**: <br>Simplest form of e.g. fn-edge-fn]
   DependencyGraph --> GraphType2[**Type 2 interface-relationship graph**: <br> First interface -- First edge -- Second interface -- Second edge -- Third interface]

   PRD --> PRDjson[**LLM Reads PRD**]
   PRDjson -->ContextBase[**Combined Context with LLMs**: <br> LLMs combines the context of PRD, Dependency Graph 1, Dependency Graph2]
   GraphType1 -->ContextBase
   GraphType2 -->ContextBase

   ContextBase --> ContextAdequacyTest[**Does the LLM feel it needs more context?**]
   ContextAdequacyTest-- NO --> MicroPRDs[Time to break down PRD into N parts, how many micro PRDS]
   ContextAdequacyTest-- YES --> DomainTypes[API or Pattern or Deep research Requirements]
   DomainTypes --> APIOriginGitRepos[**APIs** <br> LLMs clone repos libraries which they are dependent on for APIs **]
   DomainTypes --> APIPatternGitRepos[**Use Patterns** <br> LLMs clone repos libraries which they are dependent on for Patterns of usage for specific APIs **]
   DomainTypes --> PredenceRequirements[**Similar Problems** <br> LLMs clone repos libraries which might be solving similar problems **]
   DomainTypes --> AbstractResearchRequirements[**Abstract Research** <br> LLMs assimilate research from the mathematical or physics or scientific papers or blogs or social media posts from the internet which do not have code precendence but the patterns match some how **]




```





### Rough Steps


- [ ] Steps of agent need to change
- Step01: PRD evolution to list of Micro-PRDs
- Compare PRD to ISG
- Break PRD into multiple parts
- Calcuate feasibility of each broken Micro-PRD
- Choose to analyze deeply each Micro-PRD






===

# Random Ideas 

## Architecture ideas

``` mermaid
graph TD
    A[Parseltongue v4.0] --> B[Semantic Clustering]
    A --> C[Multi-Flow Analysis]
    A --> D[Dynamic Context Selection]
    A --> E[Enhanced Visualization]
    
    B --> B1[Cluster Tools 01]
    B --> B2[Cluster Tools 02]
    B --> B3[Cluster Tools 03]
    B --> B4[Cluster Tools 04]
    
    C --> C1[Data Flow]
    C --> C2[Control Flow]

```

## Long notes


      - [ ] Strategy 5 (Semantic Search) - Documented but not implemented
         - [ ] Line 487: "Status: Future enhancement (clustering not yet implemented)"
         - [ ] No pt07-query-cluster tool available
         - [ ] No semantic cluster discovery



Okay here is the idea that I am having: Half the time we want to understand what is already there but in order for us to prove that the architecture might work we need inspiration from other open source libraries who might have done it. And there is always something somebody who has done. Generally in fact to build something we use help from very very fundamental libraries. The problem with all of it is we cannot. So we put them in subfolders but we index them and all that indexing cannot happen together. So what we need truly is a way to document all of these codebases together. There'll be two classes of codebases:
1. The GitHub repo and then it shouldn't have a lot of subfolders
2. A list of GitHub repos which will be in the same area but they will be inside  Think of 10 GitHub repos which we have just taken inside the reference folder which will also should be converted into CosaDB because we want to get their JSON and this is something that is often not considered. For example all the API calls you want or internals you want to know from using TreeCity then TreeCity library would be better being in a codebase as a sub. You get cloned. We won't touch it, we won't change it but we want to learn from it. We want to find its intricate details and this is a humongous hack. The problem is you're not ready to have this hack. And if you can have this hack then actually you can code far better because this is one and saying is whatever you want to search on the code you got damn better search it. This particular folder that makes sense. Oh these are my initial thoughts and then you can. I don't know what you can do with this information but think about it. We right now only in just one Git folder but this can be way beyond that.


===

# For v300 (long term backlog)


### 📊 Known Gaps (from Recommendation20251107.md)
1. **No Clustering Capability**
   - Current: Users navigate function-by-function or file-by-file
   - Needed: ISGL0.5 semantic grouping (3-20 functions per cluster)
   - Impact: 80-96% time savings in code exploration tasks

2. **No Flow Analysis**
   - Current: Static entity relationships only
   - Needed: Control flow, data flow, temporal flow
   - Impact: 2.3× relevance improvement, taint tracking, bottleneck detection

3. **No Visual Feedback**
   - Current: Text-only output
   - Needed: Terminal visualizations (bar charts, cycle warnings)
   - Note: pt07 binaries exist but not integrated into agent workflows

---

## v0.9.7 Scope Options

### Option A: Quick Win - Visual Integration (2 weeks)
**Goal**: Integrate existing pt07-* visual tools into agent workflows

**Deliverables**:
- Agent automatically invokes pt07-render-entity-count-bar-chart after ingestion
- Agent shows pt07-render-dependency-cycle-warning-list when cycles detected
- Add visual feedback to ultrathink agent workflows
- Update README with visual analytics examples

**Effort**: 2 weeks
**Risk**: Low (tools already exist)
**Value**: Moderate (improved UX, no new capabilities)

**Files to Modify**:
- `.claude/agents/parseltongue-ultrathink-isg-explorer.md` - add visual tool invocations
- `crates/pt07-visual-analytics-terminal/README.md` - update examples
- Root README.md - add Section 4: Visual Analytics

---

### Option B: Foundation - Clustering Phase 1 (4 weeks)
**Goal**: Implement ISGL0.5 semantic clustering (from Recommendation20251107.md Phase 1)

**Deliverables**:
- New crate: `pt09-semantic-clustering`
- pt09-cluster-by-louvain: Group related functions using Louvain algorithm
- pt09-cluster-query: Find which cluster(s) contain specific functions
- Add `clusters` table to CozoDB schema
- Update agent to use clustering in code exploration workflows

**Effort**: 4 weeks
**Risk**: Medium (new graph algorithms, schema changes)
**Value**: High (addresses #1 gap, 80% time savings)

**Files to Create**:
- `crates/pt09-semantic-clustering/Cargo.toml`
- `crates/pt09-semantic-clustering/src/lib.rs`
- `crates/pt09-semantic-clustering/src/algorithms/louvain.rs`
- `crates/pt09-semantic-clustering/src/bin/pt09_cluster_by_louvain.rs`
- `crates/pt09-semantic-clustering/src/bin/pt09_cluster_query.rs`

**Files to Modify**:
- `crates/parseltongue-core/src/schema.rs` - add clusters table
- `.claude/agents/parseltongue-ultrathink-isg-explorer.md` - clustering workflows
- `Cargo.toml` - add pt09 to workspace

---

### Option C: Foundation - Flow Analysis Phase 1 (4 weeks)
**Goal**: Implement control flow analysis (from Recommendation20251107.md Phase 2)

**Deliverables**:
- New crate: `pt10-flow-analysis`
- pt10-control-flow: Trace call chains (who calls whom)
- pt10-find-bottlenecks: Detect functions with high betweenness centrality
- Add `control_flow_edges` table to CozoDB schema
- Update agent to use flow analysis in debugging workflows

**Effort**: 4 weeks
**Risk**: Medium (complex graph traversal, performance concerns)
**Value**: High (addresses #2 gap partially, enables debugging workflows)

**Files to Create**:
- `crates/pt10-flow-analysis/Cargo.toml`
- `crates/pt10-flow-analysis/src/lib.rs`
- `crates/pt10-flow-analysis/src/control_flow.rs`
- `crates/pt10-flow-analysis/src/bin/pt10_control_flow.rs`
- `crates/pt10-flow-analysis/src/bin/pt10_find_bottlenecks.rs`

**Files to Modify**:
- `crates/parseltongue-core/src/schema.rs` - add control_flow_edges table
- `.claude/agents/parseltongue-ultrathink-isg-explorer.md` - flow workflows
- `Cargo.toml` - add pt10 to workspace

---

### Option D: Incremental - TOON Format Enhancements (1 week)
**Goal**: Improve existing TOON format with user-requested features

**Potential Enhancements**:
- TOON compression option (gzip) for large codebases
- TOON streaming mode for real-time ingestion
- TOON validation tool (detect format errors)
- TOON statistics (show token savings per file)

**Effort**: 1 week
**Risk**: Low (incremental improvement)
**Value**: Low-Medium (improves existing feature, no new capabilities)

**Files to Modify**:
- `crates/parseltongue-core/src/serializers/toon.rs` - add enhancements
- `crates/parseltongue-core/src/serializers/mod.rs` - export new features
- Add new bins if needed (pt11-toon-validate, pt11-toon-stats)

---

## Recommendation Matrix

| Option | Effort | Risk | Value | Aligns with Recommendation20251107.md |
|--------|--------|------|-------|--------------------------------------|
| A: Visual Integration | 2 weeks | Low | Medium | Partial (Phase 4: Visual Feedback) |
| B: Clustering Phase 1 | 4 weeks | Medium | High | YES (Phase 1: Clustering) |
| C: Flow Analysis Phase 1 | 4 weeks | Medium | High | YES (Phase 2: Control Flow) |
| D: TOON Enhancements | 1 week | Low | Low-Medium | No |

---

## My Recommendation: **Option B (Clustering Phase 1)**

**Rationale**:
1. **Highest Impact**: Addresses the #1 gap from Recommendation20251107.md research
2. **Foundation for Future**: Clustering is prerequisite for advanced flow analysis (Phase 3)
3. **Clear User Value**: 80% time savings in code exploration (proven in simulations)
4. **Manageable Scope**: 4 weeks for Phase 1, can iterate in v0.9.8, v0.9.9
5. **Follows ONE FEATURE PER INCREMENT**: Single major capability per release

**Follow-up Roadmap**:
- v0.9.7: Clustering Phase 1 (Louvain algorithm, basic queries)
- v0.9.8: Flow Analysis Phase 1 (Control flow, bottleneck detection)
- v0.9.9: Flow Analysis Phase 2 (Data flow, taint tracking)
- v0.10.0: Visual Integration + Multi-flow synthesis (Phase 4)

---

## Questions for User Decision

1. **Which option aligns with your priorities?**
   - Quick win (Option A: Visual)
   - Foundation (Option B: Clustering or Option C: Flow)
   - Incremental (Option D: TOON)

2. **Timeline constraints?**
   - Do you need v0.9.7 within 2 weeks, or can we take 4 weeks for foundation work?

3. **Feature dependencies?**
   - Are there external commitments (Agent Games 2025, demos) that require specific features?

4. **Breaking changes acceptable?**
   - Options B & C require CozoDB schema changes (migration needed)

---

## Current Working Tree Status

```
M .claude/.parseltongue/S01-README-MOSTIMP.md
M .claude/.parseltongue/S02-visual-summary-terminal-guide.md
M .claude/.parseltongue/S05-tone-style-guide.md
M .claude/.parseltongue/S06-design101-tdd-architecture-principles.md
M crates/pt01-folder-to-cozodb-streamer/Cargo.toml
D crates/pt01-folder-to-cozodb-streamer/src/main.rs
M crates/pt01-folder-to-cozodb-streamer/src/streamer.rs
D crates/pt02-llm-cozodb-to-context-writer/src/main.rs
M crates/pt03-llm-to-cozodb-writer/Cargo.toml
D crates/pt03-llm-to-cozodb-writer/src/main.rs
M crates/pt04-syntax-preflight-validator/Cargo.toml
D crates/pt04-syntax-preflight-validator/src/main.rs
M crates/pt05-llm-cozodb-to-diff-writer/Cargo.toml
D crates/pt05-llm-cozodb-to-diff-writer/src/main.rs
M crates/pt06-cozodb-make-future-code-current/Cargo.toml
D crates/pt06-cozodb-make-future-code-current/src/main.rs
M crates/pt07-visual-analytics-terminal/Cargo.toml
D crates/pt07-visual-analytics-terminal/src/bin/pt07_visual_analytics_terminal.rs
D crates/pt07-visual-analytics-terminal/src/bin/render_dependency_cycle_warning_list.rs
D crates/pt07-visual-analytics-terminal/src/bin/render_entity_count_bar_chart.rs
```

**Note**: Uncommitted changes above - should be resolved before starting v0.9.7 work.

---

## Next Steps

1. **User Decision**: Choose option A, B, C, or D (or propose Option E)
2. **Clean Working Tree**: Commit or stash current changes
3. **Create Feature Branch**: `git checkout -b feature/v0.9.7-<selected-option>`
4. **TDD Cycle**: STUB → RED → GREEN → REFACTOR
5. **Update Agent**: Modify `.claude/agents/parseltongue-ultrathink-isg-explorer.md` with new capabilities

---

**This document will be archived to `zzArchive202510/scopeDocs/` after v0.9.7 release.**
