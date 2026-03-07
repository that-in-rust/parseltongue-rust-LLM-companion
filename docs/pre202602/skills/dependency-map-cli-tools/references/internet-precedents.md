# Internet Precedents: No-DB Codebase Mapping

This skill is based on established tooling patterns in open-source code navigation.

## 1) Pointer-first navigation is standard
- Language Server Protocol (LSP 3.17) defines project-wide symbol discovery (`workspace/symbol`) and call hierarchy APIs (`callHierarchy/incomingCalls`, `callHierarchy/outgoingCalls`).
- This supports a pointer-first loop: find symbol locations first, read exact spans second.
- Source: [LSP 3.17 Specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)

## 2) "Precise first, search fallback" works in practice
- Sourcegraph documents precise navigation using SCIP indexes and fallback to text search when precise data is missing.
- This validates a hybrid strategy for rough maps without full indexing.
- Source: [Sourcegraph precise code navigation](https://sourcegraph.com/docs/code_search/explanations/precise_code_navigation)

## 3) Concrete syntax trees and structural search are reliable primitives
- Tree-sitter provides fast incremental parsing into concrete syntax trees.
- ast-grep applies AST-based pattern search on top of tree-sitter, reducing regex-only noise.
- Sources:
  - [Tree-sitter docs](https://tree-sitter.github.io/tree-sitter/)
  - [ast-grep docs](https://ast-grep.github.io/)

## 4) ctags gives cross-language symbol spans cheaply
- Universal Ctags supports JSON output and fields such as line number and end line.
- This is enough to produce `file:start:end` pointers without a database.
- Source: [Universal Ctags options](https://docs.ctags.io/en/latest/man/ctags.1.html)

## 5) ripgrep is a strong lexical baseline
- ripgrep provides recursive line-oriented search with line numbers by default.
- For rough dependency extraction, lexical import/include scans are fast and portable.
- Source: [ripgrep guide](https://github.com/BurntSushi/ripgrep/blob/master/GUIDE.md)

## 6) Dependency graph rendering can stay simple
- Graphviz DOT and tools like dependency-cruiser show practical graph output workflows.
- For rough maps, generating DOT/Mermaid from extracted edges is usually sufficient.
- Sources:
  - [Graphviz DOT language](https://graphviz.org/doc/info/lang.html)
  - [dependency-cruiser](https://github.com/sverweij/dependency-cruiser)
