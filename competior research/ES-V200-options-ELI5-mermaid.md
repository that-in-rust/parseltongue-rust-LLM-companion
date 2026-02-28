# ES-V200 Options (ELI5) with Mermaid
Date: 2026-02-27
Source option memo:
- `/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/competior research/ES-V200-cocoindex-options-01.md`

## Big Picture (ELI5)
Think of Parseltongue like a smart city map for code.
- We want the map to be correct.
- We want it to load fast.
- We want humans and AI to trust every road shown.

```mermaid
flowchart TB
    A["Codebase"] --> B["Ingest carefully"]
    B --> C["Store proof and confidence"]
    C --> D["Answer queries safely"]
    D --> E["Human and AI trust results"]
```

## Option 1: Zero-Config Workspace Boot
ELI5: App should find the right folder automatically, so users do not trip on setup.

```mermaid
flowchart TB
    A["Start Parseltongue"] --> B["Find '.parseltongue' marker"]
    B --> C["Else find '.git' root"]
    C --> D["Else use current folder"]
    D --> E["Save resolved root in run ledger"]
```

## Option 2: Freshness Contract + Singleflight Reindex
ELI5: User can pick "fresh now" or "fast enough now"; system clearly says data freshness.

```mermaid
flowchart TB
    A["Query arrives"] --> B["Choose freshness mode"]
    B --> C["If needed, refresh index"]
    C --> D["Use single lock for refresh"]
    D --> E["Return result with index state"]
```

## Option 3: Two-Layer Response Envelope
ELI5: First show tiny answer card; expand only when user asks for full details.

```mermaid
flowchart TB
    A["User asks question"] --> B["Return locator view"]
    B --> C["Key path span confidence"]
    C --> D["Need more details?"]
    D --> E["Return expanded body on opt in"]
```

## Option 4: Evidence Search Sidecar
ELI5: Semantic search can suggest clues, but clues are not truth until verified.

```mermaid
flowchart TB
    A["Vector search hit"] --> B["Tag as evidence"]
    B --> C["Map to canonical entity"]
    C --> D["Run conflict checks"]
    D --> E["Promote only if safe"]
```

## Option 5: Capability Manifest Endpoint
ELI5: System should openly say what it is good at, partial at, or weak at.

```mermaid
flowchart TB
    A["Collect runtime capabilities"] --> B["Language tier coverage"]
    B --> C["Tool availability"]
    C --> D["Degrade reasons"]
    D --> E["Expose profile endpoint"]
```

## Option 6: MCP Setup Command
ELI5: One command should connect Parseltongue to MCP clients safely.

```mermaid
flowchart TB
    A["Run 'parseltongue setup'"] --> B["Detect MCP clients"]
    B --> C["Show dry run diff"]
    C --> D["Write config deterministically"]
    D --> E["Log what changed"]
```

## Path Choices (ELI5)
Pick a path based on what matters right now.

```mermaid
flowchart TB
    A["What matters most now?"] --> B["Ship trust fastest"]
    A --> C["Improve semantic discovery"]
    A --> D["Boost onboarding first"]
    B --> E["Path A"]
    C --> F["Path B"]
    D --> G["Path C"]
```

## Path A (Trust and Operator Moat)

```mermaid
flowchart TB
    A["Path A"] --> B["Option 1"]
    A --> C["Option 2"]
    A --> D["Option 3"]
    A --> E["Option 6"]
```

## Path B (Semantic Discovery Expansion)

```mermaid
flowchart TB
    A["Path B"] --> B["Option 1"]
    A --> C["Option 2"]
    A --> D["Option 4"]
    A --> E["Option 5"]
```

## Path C (Platform Handshake First)

```mermaid
flowchart TB
    A["Path C"] --> B["Option 1"]
    A --> C["Option 3"]
    A --> D["Option 6"]
```

## Guardrails (Do Not Copy Blindly)

```mermaid
flowchart TB
    A["Similarity score"] --> B["Useful for ranking"]
    B --> C["Not equal to truth"]
    C --> D["Never mark verified without deterministic evidence"]
```
