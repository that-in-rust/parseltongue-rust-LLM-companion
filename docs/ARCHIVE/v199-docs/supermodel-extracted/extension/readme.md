# Supermodel: Shared Vision for Human-Agent collaboration

**‼️This is a Beta Release ‼️**

Managing code you didn't write used to be an enterprise problem, now with AI it's everyone's problem.

Supermodel helps by creating an interactive model of any codebase allowing you to visually collaborate with autonomous coding agents to review architecture, understand complex software, plan changes and ensure your vision aligns with what your agents are going to build before they touch the code.

Stay in control of your codebase.
- prevent architectural drift
- understand your system at every level
- avoid compounding errors that can quickly make hours or days of token outputs entirely unusable

## Features

- **Interactive Graph Model**: 
    - Understand any codebase visually with an interactive graph.
    - Real time view of your system architecture so you can prevent drift and uncover hidden couplings.
    - Filter views by domain, node type and more
    - Node properties panel provides useful context such as imports, dependencies, and detailed explanations for every part of your system.
-  **MCP Server**:  
    - Allows agents to stub out features/changes in the graph for visual review before touching the code.
    - When generating code, stub nodes act as a "to-do list" for agents, grouped as features within the context of your full code graph.
    - Agents can query Supermodel for critical context when generating code to save tokens, reduce mistakes, and persist system knowledge across agents and sessions.

## Getting Started

1. Open a project workspace in VS Code.
2. Click the Supermodel icon in the activity bar.
3. Configure the MCP Server in your favorite agentic development tool for full functionality including context and stubbing features.
4. Create a new project by selecting the workspace you want to process and initiating Supermodel generation.

## Data & Privacy

Your code is not stored on our Azure servers after being processed to generate your Supermodel. 

We also maintain a zero data retention (ZDR) policy with our AI providers. 

We currently use models hosted via OpenRouter. 

Eventually you will need to bring your own API Keys or pay per usage, however during the beta we don't want to be tied down to hosts or model providers while we're optimizing the analysis.

## Configuration

If you need to configure Supermodel to include or exclude any specific files or types, configure the extension through VS Code settings (search for "Supermodel"):

- **`supermodel.includeGlobs`**: File patterns to include when scanning (default: `["**/*.{ts,js,jsx,py}"]`)
- **`supermodel.excludeGlobs`**: File patterns to exclude when scanning (default: `node_modules`, `dist`, etc.)

## Status

This extension is currently in active development. Features and functionality may change without notice.

---

_For development documentation and technical details, see the internal development docs._

_For careers, feedback or other requests, please contact abe@supermodel.software_