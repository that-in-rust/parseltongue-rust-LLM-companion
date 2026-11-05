# Lessons from LegacyCode Strata (decompiled) and clarity-cli for the Rust Interface Graph

Companion to the other notes in `devin-research/`. Sources: `strata-0.22.5-macos-arm64.dmg` from https://legacycode.com/download/strata, extracted on Linux (7-Zip 24.09), analysed via `Info.plist`, bundled resources, `strings` on the Swift binary (Swift symbols are stripped; protocol names and string literals survive); and a clone of https://github.com/LegacyCodeHQ/clarity-cli (Go, Apache-2.0) at HEAD on 2026-11-05. Everything below is from direct inspection unless marked as inference.

---

## 1. What Strata actually is

Not a code-analysis tool. Strata is a **native SwiftUI macOS diagram-spec editor with an MCP server**. Tagline on the product page: "The visual spec your coding agent builds from."

Facts from the bundle:

- Swift app (`libswiftCompatibilitySpan.dylib`, SwiftUI), GRDB/SQLite for storage, Sparkle updater, PostHog analytics, Keychain and "passport.legacycode.com" session/subscription flows (protocols `SessionStore`, `SubscriptionAccessServicing`, `SubscriptionCheckoutFlowing`). No tree-sitter, no LLM client, no code parser inside the app (`strings | grep -ic tree.sitter` = 0).
- Bundles a 40 MB **D2** binary (`Resources/bin/d2`, also looks in `/opt/homebrew/bin/d2`). All diagrams are D2 source compiled by d2; the app renders the SVG in a WebView (`SVGCanvas.html/js/css`).
- SQLite schema: `project`, `diagram_group`, `diagram(kit, layout_engine, current_version)`, `version(diagram_id, source, message, label, author, parent_id, render_ok, render_log)`. Every edit is an **immutable version row of the full source**; `render_ok=0` versions are refused ("Invalid D2 never lands").
- **Kits** = constrained DSLs compiled to D2: `freeform` (raw D2), `user_flow`, `discovery_tree`, plus `markdown` documents. Kit sources support `@direction`, `@status { path: status }` and "typed end nodes (success / drop-off / error) become the test oracle".
- **MCP server** on `127.0.0.1:7337/mcp`, loopback only. Tools (from binary strings): `get_active_context`, `list_projects`, `create_project`, `list_groups`, `create_group`, `rename_group`, `delete_group`, `list_artifacts`, `create_artifact`, `get_artifact`, `update_artifact`, `rename_artifact`, `duplicate_artifact`, `move_artifact`, `delete_artifact`, `validate_artifact`, `describe_kit`, `plan_first_artifact`, `list_versions`, `get_version`, `diff_versions`, `restore_version`, `get_artifact_export`, `export_artifacts_zip`, `open`, and a sidebar disclosure verb.
- A long embedded system prompt tells the agent when to offer a Strata artifact, to call `describe_kit` before authoring, to `validate_artifact` until green, to keep diagrams to "the ~8-15 most important nodes", and (in `plan_first_artifact`) to inspect `git log --author` to draw the developer's own recent work first.

### Design patterns worth copying

| Strata pattern | Why it matters for us |
|---|---|
| **Compile-before-commit.** A version is accepted only if the source compiles and renders. | Our graph store should refuse a model-emitted graph fragment that fails structural validation (unknown IDs, bad spans, dangling edges). Model output never lands unvalidated. |
| **`describe_kit` / `validate_artifact` pair.** The agent fetches the grammar, drafts, validates, gets `diagnostics[{severity,message,line,node}]`, repeats. | Exactly the loop we need for a small model emitting graph fragments under a grammar: structured diagnostics are the training signal and the runtime guardrail. |
| **Opaque `version_handle` as a write precondition** (optimistic concurrency; `conflict` returns current source to rebase onto). | Incremental updates from several agents editing the same crate need the same mechanism on the graph. |
| **Constrained kits over raw D2.** "Never fall back to freeform because you're unsure." | Our LLM projection (the Rust skeleton) is a constrained kit; raw JSON triples are the freeform fallback, not the default. |
| **Immutable full-source snapshots + `diff_versions`.** | Graph diffs between two revisions are the PR Change Witness product from docs-lossfunk/03; store whole snapshots keyed by content hash and diff them, don't try to maintain a mutable graph. |
| **Small, true artefacts.** The prompt caps diagrams at ~15 nodes. | Reinforces the layered projection: never show an agent the whole graph. |
| **Loopback MCP, SQLite, no cloud.** | Same posture Parseltongue already has (local HTTP on 7777). Switching the agent-facing surface from 26 REST endpoints to a small MCP tool set with precise descriptions is the cheaper lesson. |

What Strata does **not** teach: anything about extracting structure from code. Its graphs are hand- or agent-authored. The "spec first, build against it" workflow is the interesting product idea: a Rust skeleton graph authored *before* the code exists is a spec the agent builds against, and the extracted graph from the finished code is the check. That is the "graph first or code first" decision from the 10M-LOC note, and Strata has chosen "spec first".

---

## 2. What clarity-cli actually is

A **file-granularity** dependency-impact tool in Go, tree-sitter based (go-tree-sitter), 18 languages including Rust. From its README: "Clarity works at file granularity. It recovers coupling shape, not runtime behavior or full API contracts," and it explicitly does not provide "full symbol-level call graphs" or "API contract verification".

### Architecture (depgraph/)
- `moduleapi.Provider` per language: `Name, Extensions, Maturity, NewResolver, IsTestFile`. Clean plug-in boundary with a declared **maturity level** per language.
- Rust support (`depgraph/languages/rust/`, ~2.9k LOC non-test): `use`/`mod`/`extern crate` extraction (fast byte scanner with a tree-sitter fallback), a project import resolver that understands `crate::`/`self::`/`super::`, `mod.rs` vs `foo.rs` ownership, `pub use` re-export hops, custom `[lib]`/`[[bin]] path`, workspace members and `[workspace.dependencies] path`, Cargo manifest parsing, and a "phantom" test node split using `#[cfg(test)]` regions.
- Evidence model (`dependency_evidence.go`): every file edge carries `DependencyEvidence{Symbol, Kind, Relationship, ReferenceFile/Line, DeclarationFile/Line, Confidence high|medium|low}` with a cross-language relationship taxonomy: `resolved-dependency, module-declaration, import, type-import, re-export, call, type-reference, symbol-reference, inheritance, ...`. Unknown precision degrades to medium-confidence `resolved-dependency` rather than a guess.
- Rust evidence extraction (`parser_rust_evidence.go`) is **regex over lines**: a declaration regex at brace depth 0, identifiers followed by `(` are "call", capitalised identifiers are "type-reference". No scoping, no receiver types, no macro awareness. This is exactly the quality ceiling of a parser-only approach and a fair picture of the baseline we must beat.
- `cycles`: SCCs over the file graph, exact minimum break sets for <=16 internal edges, greedy otherwise, every set re-verified by removal; `--explain` prints the symbol/line evidence per edge; `--include-kind/--exclude-kind` filters rebuild the graph by relationship.
- `show` surface (docs/show-surface-v2.md): four orthogonal questions, **Snapshot** (working tree, commit, range) x **Anchor** (paths, `--between a,b`, `--all`, `--module`) x **Lens** (`--reach up|down|both --depth n --prune`, `--collapse` modules) x **Render** (dot, mermaid, URL, live watch).
- Rust evidence-adapter capabilities listed in docs/cycles.md: "module declarations, imports, re-exports, calls, and type references".

### Design patterns worth copying

| Clarity pattern | Why it matters for us |
|---|---|
| **Evidence on every edge with confidence and a reference/declaration line pair.** | Matches the `Observation/Assertion` records in docs-lossfunk/02. Adopt their field names where they fit; it lowers the cost of interop. |
| **Relationship taxonomy as filter keys** (`--include-kind call`). | Our per-relation scoring and per-relation views need the same vocabulary; theirs is already cross-language. |
| **Snapshot x Anchor x Lens x Render grammar.** | Better query API than 26 ad-hoc endpoints. Our agent tools should be: snapshot (rev), anchor (item/module/crate/`--between`), lens (`up|down|both`, depth, collapse-private), render (skeleton, JSONL, mermaid). |
| **Default anchor = the diff.** `clarity show` with no args shows the uncommitted change's neighbourhood. | The highest-value default for an agent loop: "what did I just touch, and who depends on it". |
| **Verified break sets for cycles.** | Cheap, exact, and concretely actionable; trivial to port to item-level graphs. |
| **Declared maturity per language, phantom test nodes, `#[cfg(test)]` region detection.** | Honest UX about precision; we should expose per-relation confidence the same way. |
| **Rust module-resolution edge cases encoded as tests** (`mod.rs` vs `foo.rs` ownership, re-export hops, `[[bin]] path`, workspace deps). | Free fixture list for our deterministic baseline and for the held-out test set. |

### Where clarity stops, and where we start
Clarity resolves **files**, not items. It cannot answer "who calls `Store::open`", "who implements `Service`", or "what is this crate's public surface". Its Rust `call` evidence is a regex. That is the gap: item-level identity, visibility-aware public surface, trait `IMPLEMENTS`, and resolved `CALLS`. Everything else (evidence model, CLI grammar, cycle analysis, MCP posture) is already solved well enough to borrow rather than reinvent.

---

## 3. Concrete takeaways for Parseltongue

1. **Agent surface:** replace or wrap the 26 REST endpoints with ~8 MCP tools following Clarity's Snapshot/Anchor/Lens/Render grammar, with Strata-grade tool descriptions (precise preconditions, what is and is not an error).
2. **Validation loop:** every model-emitted graph fragment goes through `validate_fragment` returning structured diagnostics; only valid fragments are committed, as immutable snapshots keyed by content hash. This doubles as the reward signal for training.
3. **Evidence schema:** adopt `relationship`, `confidence`, `reference_{file,line}`, `declaration_{file,line}` field names; add `cfg`, `provenance`, `status` on top.
4. **Baseline:** Clarity's Rust adapter is a credible, open, tree-sitter-plus-resolver baseline. Run it on the held-out crates alongside Parseltongue's own extractor; it is likely stronger on file-level resolution than ours and weaker than ours at item level. Measure.
5. **Product posture:** Strata shows the market LegacyCode is in: local, versioned, MCP-native artefacts for agents, with a paid native app. Their open-source CLI is the funnel. The item-level Rust graph is complementary to both, not competitive; it could be a Clarity "provider" or a Strata "kit" as easily as a standalone tool.
6. **Spec-first option:** a Rust skeleton authored before code exists is a spec; the extracted skeleton from finished code is the verification. Strata already sells this loop for diagrams; nobody sells it for Rust interfaces.
