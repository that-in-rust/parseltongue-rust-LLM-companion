# Branch Differences Summary

## Files with Differences:

### 1. AGENTS.md
**Status**: `main` & `interview-docs` have the file, `apwbd20260122` does NOT have it

**Content**: 
- 309-line comprehensive guide for WARP (warp.dev)
- Contains build/test commands, architecture overview, naming conventions
- Documents the parseltongue project structure and development workflow
- Includes TDD process, error handling, CozoDB schema, workspace management

**Difference**: `apwbd20260122` branch is missing this entire file

---

### 2. docs/v2/D01_UserJourneys.md
**Status**: `main` & `interview-docs` have the file, `apwbd20260122` does NOT have it

**Content**:
- 447-line user journey documentation
- 5 user personas: AI-First Developer, AI Tool Builder, Platform Engineer, Tech Lead, OSS Maintainer
- Detailed flow diagrams using Mermaid
- Interface architecture and job-to-be-done mapping
- Practical examples of how different users interact with Parseltongue

**Difference**: `apwbd20260122` branch is missing this entire file

---

### 3. docs/v2/D02_MCP_Explained.md
**Status**: `main` & `interview-docs` have the file, `apwbd20260122` does NOT have it

**Content**:
- 574-line comprehensive MCP guide
- ELI15 explanation of Model Context Protocol
- Technical architecture, integration patterns
- Step-by-step guide for using Parseltongue as MCP with Claude Code
- Comparison of MCP vs Skills vs Agents

**Difference**: `apwbd20260122` branch is missing this entire file

---

### 4. docs/v2/D03_MCP_Industry_Research.md
**Status**: `main` & `interview-docs` have the file, `apwbd20260122` does NOT have it

**Content**:
- 1644-line industry research report
- MCP support analysis across top 10 AI coding tools
- Executive summary showing 7/10 tools support MCP
- Detailed configuration examples for each IDE
- Market adoption statistics and trends

**Difference**: `apwbd20260122` branch is missing this entire file

---

## Key Findings:

1. **`.stable/` folder**: ✅ IDENTICAL across all three branches
2. **Documentation files**: `main` and `interview-docs` have 4 additional documentation files that `apwbd20260122` lacks
3. **Missing content**: `apwbd20260122` is missing ~3,000+ lines of documentation

## Recommendation:
The `main` and `interview-docs` branches have more comprehensive documentation. Consider copying these files to `apwbd20260122` if you need consistency across all branches.

## Files Created:
- All diff files saved in `tmp/branch-diffs/` directory
- Each file shows the complete content that differs between branches
