# Research: Dependency Extraction Tools Per Language
## Parseltongue V2.0 — Maximum-Depth Graph Extraction Stack

**Date**: 2026-02-22
**Context**: This document surveys the best available tools (as of early 2026) for extracting dependency graphs to maximum depth for each of Parseltongue's 7 target languages. The goal is to identify the "rust-analyzer equivalent" per language — meaning tools that go beyond syntactic tree-sitter parsing into semantic analysis: type resolution, call graphs, module graph, and framework-specific patterns.

**Scope**: Rust, C, C++, TypeScript, JavaScript, Ruby, Ruby on Rails.

---

## Table of Contents

1. [Rust](#1-rust)
2. [C](#2-c)
3. [C++](#3-c-1)
4. [TypeScript](#4-typescript)
5. [JavaScript](#5-javascript)
6. [Ruby](#6-ruby)
7. [Ruby on Rails](#7-ruby-on-rails)
8. [Cross-Cutting Concerns](#8-cross-cutting-concerns)
9. [Summary Comparison Table](#9-summary-comparison-table)
10. [Recommended Extraction Stack Per Language](#10-recommended-extraction-stack-per-language)

---

## 1. Rust

### Baseline Reference: rust-analyzer

rust-analyzer is the production-grade language server and IDE backend for Rust. It is the gold standard against which every other language's tooling is measured in this document.

#### Primary Extraction Tool

**rust-analyzer** — the official Rust language server, maintained by the rust-lang organization. It implements a salsa-based incremental computation engine that provides full semantic analysis of Rust code including macro expansion, trait resolution, type inference, and call graph construction.

- Repository: https://github.com/rust-lang/rust-analyzer
- LSP protocol: standard LSP 3.17 plus custom `rust-analyzer/` extensions
- Crate surface: `ra_ap_ide`, `ra_ap_hir`, `ra_ap_rust-analyzer` on crates.io

#### How to Drive It from Rust

**Option A — Spawn as subprocess over stdio (JSON-RPC/LSP)**

The most stable integration approach. Spawn `rust-analyzer` as a child process, communicate over stdin/stdout using JSON-RPC, and use standard LSP methods plus custom rust-analyzer extensions.

```rust
use std::process::{Command, Stdio};
use std::io::{BufReader, BufWriter};

// Spawn rust-analyzer as a language server
let mut child = Command::new("rust-analyzer")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Failed to spawn rust-analyzer");

let stdin = BufWriter::new(child.stdin.take().unwrap());
let stdout = BufReader::new(child.stdout.take().unwrap());

// Use lsp-types crate for typed request/response structs
// Use tower-lsp or lsp-client-rs for async framing
```

Useful crates for the client side:
- `lsp-types` — typed LSP request/response structs (crates.io)
- `tower-lsp` — LSP server/client implementation built on Tower
- `lsp-client-rs` — lightweight async LSP client for driving external servers

**LSP methods relevant to graph extraction:**

```
// Initialize the server
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}

// textDocument/references — all uses of a symbol across the project
{"jsonrpc":"2.0","id":2,"method":"textDocument/references","params":{...}}

// callHierarchy/prepareCallHierarchy — resolve a position to a call hierarchy item
{"jsonrpc":"2.0","id":3,"method":"textDocument/prepareCallHierarchy","params":{...}}

// callHierarchy/incomingCalls — who calls this function?
{"jsonrpc":"2.0","id":4,"method":"callHierarchy/incomingCalls","params":{...}}

// callHierarchy/outgoingCalls — what does this function call?
{"jsonrpc":"2.0","id":5,"method":"callHierarchy/outgoingCalls","params":{...}}

// workspace/symbol — list all symbols in the workspace
{"jsonrpc":"2.0","id":6,"method":"workspace/symbol","params":{"query":""}}
```

**Option B — Link directly against `ra_ap_ide` (in-process)**

This avoids subprocess overhead but couples you to rust-analyzer's internal APIs, which change between releases. The `ra_ap_ide` crate provides:

- `Analysis::call_hierarchy()` — compute call hierarchy candidates
- `Analysis::incoming_calls()` — compute callers for a function
- `Analysis::outgoing_calls()` — compute callees for a function
- `Analysis::find_all_refs()` — cross-crate reference resolution

```rust
// Cargo.toml
[dependencies]
ra_ap_ide = "0.0.270"       # version tracks rust-analyzer nightly
ra_ap_ide_db = "0.0.270"
ra_ap_hir = "0.0.270"
```

Note: `ra_ap_*` crates are published automatically from the rust-analyzer repository. They are considered internal APIs — expect breaking changes on every release. The subprocess approach is preferred for production use.

**Option C — cargo-call-stack for static LLVM IR analysis**

`cargo-call-stack` derives the call graph from LLVM IR (the `.ll` files emitted by `rustc --emit=llvm-ir`). This works without LSP but requires a release build with LTO enabled.

```bash
cargo install cargo-call-stack
cargo call-stack --bin my_binary 2>/dev/null | dot -Tsvg > call-graph.svg
```

This approach captures indirect calls (fn pointers, trait objects) and is used for embedded / stack analysis. It does not resolve trait method dispatch as accurately as rust-analyzer because LLVM's type system is less expressive than Rust's.

#### What rust-analyzer Provides Beyond tree-sitter

| Capability | tree-sitter | rust-analyzer |
|---|---|---|
| Macro expansion | No (syntactic stubs) | Full expansion via HIR |
| Trait method resolution | No | Yes — resolves `impl Trait for T` |
| Type inference across files | No | Yes — salsa-based incremental |
| Proc-macro evaluation | No | Yes (limited, via proc-macro server) |
| Cross-crate symbol lookup | No | Yes — reads compiled metadata |
| Call graph (callee/caller) | No | Yes — via callHierarchy LSP |
| `use` path resolution | Syntactic only | Fully resolved to canonical paths |
| FFI boundary detection | No | Partial — `extern "C"` detection |

#### Maturity / Reliability

Production-grade. rust-analyzer is the official Rust IDE backend, used by VS Code, Helix, Zed, Emacs, and all major editors. Actively maintained by the rust-lang organization. Stable enough for CI automation.

#### Key Limitations

- Subprocess startup cost: 2-10 seconds for large workspaces on first indexing
- `ra_ap_*` crate APIs change weekly (track nightly, not semver)
- Proc-macro expansion requires the proc-macro server to compile macros, which can fail in cross-compilation scenarios
- Call hierarchy completeness: indirect calls through trait objects are approximated, not exact
- Does not provide a structured JSON dump of the entire call graph — requires driving the LSP request per symbol

#### Additional Rust-Specific Tools

**cargo-geiger** — detects unsafe code in the dependency tree. Not a call graph tool, but useful for marking FFI boundaries in the entity graph.

```bash
cargo install cargo-geiger
cargo geiger 2>/dev/null
```

**bindgen** — generates Rust FFI bindings from C headers. When parseltongue detects `extern "C"` blocks in Rust code, cross-referencing them against `bindgen`-generated types reveals C-to-Rust boundary edges.

**cbindgen** — inverse of bindgen; generates C headers from Rust `pub extern "C"` functions. Useful for Rust-to-C boundary detection.

---

## 2. C

### Primary Extraction Tool: clangd + compile_commands.json

The C/C++ ecosystem does not have a single unified IDE backend with the same status as rust-analyzer. The closest equivalent is **clangd**, the LLVM project's language server for C/C++/Objective-C, backed by **libclang** and **libTooling**.

The essential prerequisite for any accurate C analysis is a **compilation database** (`compile_commands.json`), which records the exact compiler flags, include paths, and defines used to build each source file.

#### Primary Extraction Tool

**clangd** — LLVM's language server for C and C++.

- Repository: https://github.com/clangd/clangd
- Protocol: standard LSP 3.17
- Requires: `compile_commands.json` in the project root (or a `compile_flags.txt` for simple projects)

#### Generating compile_commands.json

**Bear** is the standard tool for projects that do not natively produce a compilation database:

```bash
# Install Bear (Linux/macOS)
# macOS: brew install bear
# Ubuntu: apt-get install bear

# Intercept any build system's compiler invocations
bear -- make

# Output: compile_commands.json in the current directory
```

Bear intercepts compiler calls using dynamic library preloading on Unix systems. It produces a JSON array where each entry contains:
- `file` — absolute path to the source file
- `command` or `arguments` — the full compiler invocation
- `directory` — the working directory for the compilation

For **CMake** projects, native support is built in:

```bash
cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=ON -B build .
# Produces: build/compile_commands.json
```

#### How to Drive clangd from Rust

Spawn clangd as a subprocess over stdio, same pattern as rust-analyzer:

```rust
use std::process::{Command, Stdio};

let mut clangd = Command::new("clangd")
    .arg("--compile-commands-dir=/path/to/project")
    .arg("--background-index")        // indexes in background for workspace/symbol
    .arg("--clang-tidy")              // enables static analysis checks
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("failed to spawn clangd");
```

Relevant LSP requests for graph extraction:

```
// Call hierarchy (requires clangd 12+)
textDocument/prepareCallHierarchy
callHierarchy/incomingCalls
callHierarchy/outgoingCalls

// All references to a symbol
textDocument/references

// Jump to definition (cross-file)
textDocument/definition

// Document symbols (all symbols in one file)
textDocument/documentSymbol

// Workspace symbols (all symbols across all files)
workspace/symbol
```

**Important limitation**: clangd's call hierarchy implementation reports incompleteness for large projects. It may miss references that exist in files not yet indexed. The `--background-index` flag helps but results are not guaranteed to be complete until indexing finishes.

#### Alternative: GNU cflow (static C call graph)

**cflow** is a simpler, older tool that generates call graphs from C source files without needing a compiler invocation. It works purely on source text and does not require a compilation database.

```bash
# Install: apt-get install cflow  or  brew install cflow
cflow src/*.c --format=posix > callgraph.txt

# Cross-reference mode (caller/callee pairs)
cflow --format=posix --xref src/*.c
```

cflow output is text-only. To drive it from Rust:

```rust
use std::process::Command;

let output = Command::new("cflow")
    .args(&["--format=posix", "--xref"])
    .args(source_files)
    .output()
    .expect("failed to run cflow");

let stdout = String::from_utf8(output.stdout).unwrap();
// Parse the indented tree or cross-reference pairs
```

cflow does not perform type resolution or handle preprocessor macros reliably. It is best used as a first-pass approximation for projects without a build system.

#### Alternative: libclang via Rust bindings (clang-sys)

The `clang-sys` crate provides Rust bindings to libclang. This allows parsing a C translation unit programmatically and walking its AST:

```toml
[dependencies]
clang = "2.0"    # safe clang bindings crate (wraps clang-sys)
```

```rust
use clang::{Clang, Index, TranslationUnit};

let clang = Clang::new().unwrap();
let index = Index::new(&clang, false, false);

// Parse a single file with compile flags
let tu = index.parser("src/main.c")
    .arguments(&["-I/usr/include", "-DDEBUG=1"])
    .parse()
    .expect("failed to parse");

// Walk the AST cursor tree
let entity = tu.get_entity();
entity.visit_children(|cursor, _parent| {
    // CursorKind::CallExpr — a function call
    // CursorKind::FunctionDecl — a function declaration
    // cursor.get_reference() — resolves to the definition
    clang::EntityVisitResult::Recurse
});
```

This approach gives you full AST access including:
- Resolved include graph (header inclusions)
- Function declarations and definitions
- Call expressions resolved to their target declarations
- Type information (resolved typedef chains, struct layouts)
- Preprocessor macro expansion visibility (where macros expand to)

The `clang` crate (safe wrapper) and `clang-sys` (unsafe bindings) are both available on crates.io.

#### What clangd / libclang Provides Beyond tree-sitter

| Capability | tree-sitter | clangd / libclang |
|---|---|---|
| Preprocessor macro expansion | No | Yes (libclang expands at parse time) |
| `#include` graph resolution | Syntactic only | Fully resolved to file paths |
| Type resolution across headers | No | Yes — typedef chains, struct members |
| Call graph (inferred) | No | Yes — via libclang AST walk or LSP |
| `compile_commands.json` aware | No | Yes — correct flags per translation unit |
| Cross-TU analysis | No | Partial (clangd indexes cross-TU) |
| Static analyzer checks | No | Yes (clangd --clang-tidy) |

#### Maturity / Reliability

clangd: Production-grade. Ships with LLVM, used by CLion, VS Code C/C++ extension, Vim, Emacs. Version 18+ (2024) is stable.

cflow: Mature but limited. Good for simple C codebases. Not suitable for C++ or macro-heavy code.

libclang (clang-sys / clang crate): Stable C API, considered the public API surface of Clang. The Rust bindings are maintained but not as actively updated as libclang itself.

#### Key Limitations

- Requires compilation database for accurate analysis; without it, include paths and defines are wrong
- Cross-translation-unit (cross-TU) analysis in clangd is probabilistic — it uses a background index but cannot guarantee completeness
- Indirect calls through function pointers are not resolved statically
- Template instantiation in C++ (see section 3) adds significant complexity
- clangd call hierarchy completeness caveat: known issue — results may be incomplete for large codebases

---

## 3. C++

### Primary Extraction Tool: clangd + libclang + include-what-you-use

C++ analysis uses the same clangd/libclang infrastructure as C, but with additional complexity from templates, namespaces, virtual dispatch, and the ODR (One Definition Rule). All tools from section 2 apply; this section documents C++-specific additions.

#### Primary Extraction Tool

**clangd** (same as C) — fully supports C++ including:
- Template instantiation tracking
- Virtual method resolution (partial — static dispatch is resolved, dynamic dispatch is approximated)
- Namespace and `using` resolution
- Lambda capture analysis
- `auto` type deduction

#### compile_commands.json for C++

Same generation process as C (Bear, CMake). For C++ projects using other build systems:

```bash
# Ninja
ninja -C build -t compdb cxx cc > compile_commands.json

# Meson
meson setup build
cd build && meson compile --compile-commands

# Bazel (requires third-party tool)
# https://github.com/hedronvision/bazel-compile-commands-extractor
bazel run @hedron_compile_commands//:refresh_all
```

#### include-what-you-use (IWYU)

IWYU analyzes `#include` directives in C/C++ source files to determine which headers are actually needed versus which are over-included. For Parseltongue, this provides an accurate **header dependency graph**: which source files depend on which headers, and through which symbols.

- Repository: https://github.com/include-what-you-use/include-what-you-use
- Latest (as of early 2026): IWYU 0.25, compatible with LLVM/Clang 21
- CMake integration: `CMAKE_CXX_INCLUDE_WHAT_YOU_USE`

```bash
# Run IWYU on a project
iwyu_tool.py -p compile_commands.json -- --mapping_file=my_project.imp 2>&1 | tee iwyu_output.txt
```

IWYU output can be parsed programmatically. Each block shows:
```
file.cpp should add these lines:
#include <header.h>  // for SomeType

file.cpp should remove these lines:
- #include <unused_header.h>

The full include-list for file.cpp:
#include <header.h>
```

Drive from Rust via subprocess:

```rust
let output = Command::new("iwyu_tool.py")
    .args(&["-p", "compile_commands.json"])
    .output()
    .expect("IWYU failed");
// Parse stdout for the structured include analysis
```

#### Doxygen for C/C++ Call Graph (XML Output)

Doxygen generates call graphs and caller graphs using Graphviz. More usefully for programmatic consumption, Doxygen can output **XML** containing the full parsed documentation, including callee/caller relationships.

```bash
# Doxyfile configuration for call graph extraction
doxygen -g Doxyfile
# Edit Doxyfile:
#   CALL_GRAPH = YES
#   CALLER_GRAPH = YES
#   GENERATE_XML = YES
#   XML_OUTPUT = doxygen_xml/
#   EXTRACT_ALL = YES
#   HAVE_DOT = YES

doxygen Doxyfile
```

The XML output in `doxygen_xml/` contains structured data about every function including its callers and callees. This can be parsed without Graphviz. The format is well-documented at https://www.doxygen.nl/manual/xml.html.

Parse from Rust using the `quick-xml` or `roxmltree` crates.

#### Virtual Dispatch — The Unsolvable Problem

C++ virtual method calls (`v->method()`) cannot be statically resolved to a single target without runtime type information (RTTI). The best static tools can do is compute a **call graph approximation** using Class Hierarchy Analysis (CHA) or Rapid Type Analysis (RTA):

- CHA: assumes any subclass of the declared type might be the actual receiver
- RTA: narrows based on which concrete types are actually instantiated

Neither clangd nor libclang performs RTA by default. For whole-program virtual dispatch analysis, academic tools (LLVM-based, such as PHASAR) are required.

**PHASAR** — a LLVM-based static analysis framework that supports whole-program call graph construction with pointer analysis:
- Repository: https://github.com/secure-software-engineering/phasar
- Approach: operates on LLVM IR (bitcode files from clang)
- Maturity: research-grade, not production-hardened for arbitrary codebases

#### What C++ Tools Provide Beyond tree-sitter

| Capability | tree-sitter | clangd + IWYU |
|---|---|---|
| Template instantiation | No | Partial (clangd resolves instantiations) |
| Virtual dispatch resolution | No | Partial (CHA approximation) |
| Header dependency graph | Include text only | Resolved via IWYU + libclang |
| Namespace resolution | Syntactic only | Full resolution |
| `auto` type deduction | No | Yes (clangd) |
| RAII / destructor calls | No | Yes (libclang AST) |
| Macro expansion in C++ | No | Yes (libclang) |

#### Maturity / Reliability

clangd for C++: Production-grade. All major C++ IDEs use it.
IWYU: Mature and actively maintained. IWYU 0.25 supports Clang 21.
Doxygen XML: Mature. Over 20 years of active use. XML output is reliable.
PHASAR: Research-grade. Useful for specific security analysis scenarios.

#### Key Limitations

- Virtual dispatch is fundamentally undecidable statically
- Template instantiations can be large; clangd may time out on deep template hierarchies
- Header-only libraries (e.g., Boost, Eigen) make call graph attribution to a "file" complex
- ODR violations in large codebases confuse cross-TU analysis
- IWYU's mapping files require manual curation for complex projects

---

## 4. TypeScript

### Primary Extraction Tool: TypeScript Compiler API (tsc) + tsserver/typescript-language-server

TypeScript's compiler is written in TypeScript and exposes a rich programmatic API. This compiler API is the authoritative source for type-checked dependency information.

#### Primary Extraction Tool

**TypeScript Compiler API** — the TypeScript compiler (`tsc`) is itself a library (`typescript` npm package) that exposes:
- Full typed AST (with resolved types, not just syntactic nodes)
- Module resolution (follows `import` statements, `tsconfig.json` paths, barrel files)
- Type checker (`ts.TypeChecker`) for resolving types across files
- Symbol table for all exported/imported symbols
- Call hierarchy (via tsserver)

**ts-morph** — a higher-level wrapper around the TypeScript Compiler API that significantly simplifies programmatic analysis:
- Repository: https://github.com/dsherret/ts-morph
- npm: `ts-morph`
- Provides: `Project`, `SourceFile`, `Node`, `Symbol`, `Type` abstractions

**typescript-language-server** — an LSP wrapper around tsserver:
- Repository: https://github.com/typescript-language-server/typescript-language-server
- This is the standard LSP for TypeScript used by Neovim, Helix, and most non-VS Code editors

#### 2026 Architecture Note: TypeScript 7 Native Port

Microsoft is porting the TypeScript compiler to Go (project: `typescript-go`, published as `@typescript/native-preview`). As of early 2026:

- TypeScript 7 targets early 2026 release
- TypeScript 7 uses standard LSP protocol instead of the custom tsserver protocol
- The native port is 10x faster (VSCode codebase: 89 seconds → 8.74 seconds)
- The `typescript-language-server` npm package wraps tsserver for LSP compatibility
- Plan: TypeScript 6.0 (bridge release with deprecations) then TypeScript 7.0 (native, LSP-native)

For Parseltongue, this means: today use tsserver/typescript-language-server; by late 2026, migrate to TypeScript 7 LSP directly.

#### How to Drive from Rust

**Option A — Spawn typescript-language-server as subprocess (recommended)**

```rust
use std::process::{Command, Stdio};

let mut ts_server = Command::new("typescript-language-server")
    .arg("--stdio")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("typescript-language-server not installed");

// Send LSP initialize then drive with standard LSP requests
```

Relevant LSP methods for TypeScript:
```
textDocument/definition           — follow import to definition
textDocument/references           — find all uses of an exported symbol
textDocument/prepareCallHierarchy — resolve position to call hierarchy item
callHierarchy/incomingCalls       — who calls this function
callHierarchy/outgoingCalls       — what does this function call
workspace/symbol                  — all exported symbols in project
textDocument/documentSymbol       — all symbols in one file
```

**Option B — Spawn Node.js subprocess running ts-morph script**

Write a Node.js/TypeScript analysis script using ts-morph, invoke it from Rust as a subprocess, and consume its JSON output on stdout:

```typescript
// analysis-worker.ts — run this from Rust via Node.js subprocess
import { Project, SyntaxKind } from "ts-morph";
import * as path from "path";

const project = new Project({
    tsConfigFilePath: process.argv[2],  // path to tsconfig.json
});

const entities: any[] = [];
const edges: any[] = [];

for (const sourceFile of project.getSourceFiles()) {
    // Public exports
    for (const exportedSymbol of sourceFile.getExportedDeclarations()) {
        entities.push({
            key: `ts:export:${sourceFile.getFilePath()}:${exportedSymbol[0]}`,
            name: exportedSymbol[0],
            file: sourceFile.getFilePath(),
        });
    }

    // Import edges
    for (const importDecl of sourceFile.getImportDeclarations()) {
        const moduleSpecifier = importDecl.getModuleSpecifierSourceFile();
        if (moduleSpecifier) {
            edges.push({
                from: sourceFile.getFilePath(),
                to: moduleSpecifier.getFilePath(),
                kind: "imports",
            });
        }
    }

    // Function calls (syntactic)
    sourceFile.forEachDescendant((node) => {
        if (node.getKind() === SyntaxKind.CallExpression) {
            // ts.TypeChecker.getSymbolAtLocation() resolves the callee
            const callExpr = node.asKindOrThrow(SyntaxKind.CallExpression);
            const callee = callExpr.getExpression().getSymbol();
            if (callee) {
                const decls = callee.getDeclarations();
                decls.forEach(d => {
                    edges.push({
                        from: sourceFile.getFilePath(),
                        to: d.getSourceFile().getFilePath(),
                        callee: callee.getName(),
                        kind: "calls",
                    });
                });
            }
        }
    });
}

console.log(JSON.stringify({ entities, edges }, null, 2));
```

Invoke from Rust:

```rust
use std::process::Command;
use serde_json::Value;

let output = Command::new("node")
    .args(&[
        "--loader", "ts-node/esm",
        "analysis-worker.ts",
        "/path/to/project/tsconfig.json",
    ])
    .output()
    .expect("node subprocess failed");

let result: Value = serde_json::from_slice(&output.stdout)
    .expect("invalid JSON from ts-morph worker");
```

#### Barrel File Resolution

Barrel files (index.ts files that re-export from multiple sub-modules) are fully resolved by the TypeScript compiler API. When ts-morph loads a project, it follows:

```typescript
// barrel: components/index.ts
export { Button } from './Button';
export { Modal } from './Modal';

// consumer.ts
import { Button } from './components';
// TypeChecker.getSymbolAtLocation() resolves Button to components/Button.tsx
```

ts-morph resolves through barrel re-exports to the canonical definition. This is the key advantage over tree-sitter, which can only see the barrel file text, not resolve the chain.

#### dependency-cruiser for Module-Level Graph

**dependency-cruiser** provides module-level (file-to-file) dependency graphs for TypeScript, JavaScript, CoffeeScript, and other variants. It has a **programmatic JavaScript API** that can be invoked from a Node.js subprocess:

```javascript
// dep-analysis.js
const { cruise } = require('dependency-cruiser');

const result = cruise(
    ["src"],
    {
        outputType: "json",
        tsConfig: { fileName: "tsconfig.json" },
        includeOnly: "src",
    }
);

// result.output is the full dependency graph as JSON
process.stdout.write(JSON.stringify(result.output));
```

dependency-cruiser operates at the **module graph** level (file-to-file), not the symbol/function level. It uses the TypeScript compiler for resolution under the hood, so barrel files are followed correctly.

Compared to madge:
- dependency-cruiser: more robust, supports rule validation, better HTML reports, understands tsconfig paths aliases, supports barrel files, used in CI pipelines
- madge: simpler API, fewer features, popular for quick visualizations

#### What TypeScript Tools Provide Beyond tree-sitter

| Capability | tree-sitter | TypeScript Compiler API |
|---|---|---|
| Type resolution across files | No | Full — TypeChecker.getTypeAtLocation() |
| Import path resolution | Syntactic only | Full — follows tsconfig paths, node_modules |
| Barrel file traversal | No | Yes — follows re-exports to source |
| Generic type instantiation | No | Yes — TypeChecker resolves instantiations |
| Interface / type alias resolution | No | Yes |
| Export surface extraction | Syntactic only | Full — all exported declarations |
| Call graph (callee resolution) | No | Yes — via TypeChecker.getSymbolAtLocation() |
| Declaration merging | No | Yes |
| Conditional types | No | Yes |
| `satisfies` / `infer` patterns | No | Yes |

#### Maturity / Reliability

TypeScript Compiler API: Production-grade, maintained by Microsoft. Used internally by all TypeScript tooling including VS Code.

ts-morph: Mature, actively maintained. Version 22+ (2025). Used in production at many organizations.

typescript-language-server: Production-grade. Standard for non-VS Code editors.

dependency-cruiser: Mature, widely adopted in CI pipelines.

#### Key Limitations

- TypeScript Compiler API startup cost: loading a large project (1M+ LOC) takes 10-30 seconds for the first full type-check
- Dynamic imports (`import()`) are resolved syntactically but type information at the call site is approximated
- JavaScript files in a TypeScript project with `allowJs: true` may have incomplete type info
- TypeScript 7 (Go-native port) will break the current TypeScript Compiler API surface — migration path not yet documented as of early 2026
- Circular imports (common in large TypeScript projects) do not cause analysis failure but can cause incorrect ordering in symbol lookup

---

## 5. JavaScript

### Primary Extraction Tool: TypeScript Compiler API (with allowJs) + dependency-cruiser + OXC

JavaScript analysis is a subset of TypeScript analysis. The TypeScript compiler fully supports JavaScript files when `allowJs: true` is set in tsconfig.json, providing type inference (even without type annotations) through JSDoc and flow-type inference.

#### Primary Extraction Tool

**TypeScript Compiler API with `allowJs: true`** — the same compiler API used for TypeScript works on JavaScript files. Type information is inferred rather than declared:
- Return type inference from function bodies
- JSDoc type annotations (e.g., `@param {string} name`)
- Control flow analysis

**Acorn** — the JavaScript parser underlying Node.js's own parsing. Fast, correct, spec-compliant ES2025 parser.
- npm: `acorn`
- Used by: ESLint, webpack, Rollup, dependency-cruiser

**Babel Parser** — alternative JS/JSX parser with plugin architecture. Supports all proposal-stage syntax:
- npm: `@babel/parser`
- Advantage over Acorn: supports TypeScript, JSX, Flow, decorators out of the box
- Used by: Jest, Babel, Metro (React Native)

#### OXC — Rust-Native JavaScript/TypeScript Parser

**OXC** (Oxidation Compiler) is a collection of high-performance JavaScript and TypeScript tools written entirely in Rust. As of 2025-2026:

- **oxc_parser**: fastest conformant JS/TS parser written in Rust, 50-100x faster than Acorn
- **oxc_resolver**: Rust port of webpack/enhanced-resolve, 28x faster for module resolution
- **oxc_semantic**: semantic analysis (scope analysis, symbol resolution) on top of the AST
- **oxlint**: linter, 50-100x faster than ESLint, v1.0 released August 2025
- **Biome v2**: separate project, but similar architecture — includes type-aware linting (June 2025) without requiring the TypeScript compiler

OXC is the most relevant Rust-native option for Parseltongue to integrate directly:

```toml
[dependencies]
oxc_parser = "0.70"       # check crates.io for latest
oxc_resolver = "3.0"      # module resolution
oxc_semantic = "0.70"     # scope/symbol analysis
```

```rust
use oxc_parser::{Parser, ParserReturn};
use oxc_allocator::Allocator;
use oxc_span::SourceType;

let allocator = Allocator::default();
let source_type = SourceType::from_path("app.js").unwrap();
let source_text = std::fs::read_to_string("app.js").unwrap();

let ParserReturn { program, errors, .. } = Parser::new(
    &allocator,
    &source_text,
    source_type,
).parse();

// program is a fully-parsed AST
// Walk program.body to extract imports, exports, function declarations
```

For module resolution from Rust:

```rust
use oxc_resolver::{ResolverGeneric, ResolverOptions};

let resolver = ResolverGeneric::new(ResolverOptions {
    extensions: vec!["js".into(), "mjs".into(), "cjs".into()],
    ..Default::default()
});

// Resolve "react" from /project/src/app.js
let resolved = resolver.resolve("/project/src", "react").unwrap();
// Returns the absolute path to the resolved module
```

`oxc_resolver` is a first-class Rust crate — no subprocess needed for module resolution.

#### dependency-cruiser for JavaScript

dependency-cruiser supports JavaScript projects equally well:

```bash
# CLI invocation
depcruise --output-type json src/ > deps.json

# Or driven via Node.js API (see TypeScript section)
```

dependency-cruiser uses Acorn by default for JavaScript (when not TypeScript). It supports:
- CommonJS `require()` resolution
- ES module `import` resolution
- AMD `define()` patterns (legacy)
- Dynamic `import()` (syntactic extraction, not type-checked)

#### madge

**madge** generates module dependency graphs for JavaScript/Node.js projects. It is simpler than dependency-cruiser:

```bash
# Install
npm install -g madge

# Generate dependency graph
madge --json src/index.js > deps.json

# Find circular dependencies
madge --circular src/

# For TypeScript projects
madge --ts-config tsconfig.json src/index.ts
```

madge is popular for quick visualizations but has known limitations:
- Incomplete support for path aliases (tsconfig `paths`)
- Weaker support for complex CommonJS patterns
- No rule validation (dependency-cruiser is better for CI gates)

#### What JavaScript Tools Provide Beyond tree-sitter

| Capability | tree-sitter | TypeScript API (allowJs) + OXC |
|---|---|---|
| Module resolution (CommonJS) | Syntactic only | Full (oxc_resolver / TypeChecker) |
| Module resolution (ESM) | Syntactic only | Full |
| JSDoc type extraction | No | Yes (TypeChecker) |
| Dynamic require() approximation | No | Partial (static string args only) |
| Scope analysis | No | Yes (oxc_semantic) |
| Symbol table | No | Yes (TypeChecker / oxc_semantic) |
| Circular dependency detection | No | Yes (dependency-cruiser, madge) |

#### Maturity / Reliability

OXC: Rapidly maturing. v1.0 of oxlint released August 2025. The resolver is production-grade, used by Rolldown (Vite's new bundler). oxc_semantic is stable for scope analysis.

dependency-cruiser: Production-grade. Widely used in enterprise TypeScript/JavaScript monorepos.

madge: Mature but not feature-complete for complex projects.

#### Key Limitations

- JavaScript without TypeScript cannot have guaranteed type resolution — everything is inferred or approximate
- Dynamic requires (`require(path_variable)`) are impossible to resolve statically
- CommonJS `module.exports = { ... }` patterns require heuristic detection of exports
- OXC's semantic analysis does not yet reach TypeScript Compiler API-level type checking accuracy; it is complementary (speed) rather than a complete replacement
- Biome v2's type-aware linting covers ~85% of typescript-eslint checks without the compiler — sufficient for many use cases but not complete

---

## 6. Ruby

### Primary Extraction Tool: ruby-lsp + Prism parser + Sorbet (optional)

Ruby is a highly dynamic language. The ceiling of static analysis is lower than for Rust or TypeScript. The key constraint: **without type annotations (Sorbet/RBS), callee resolution is often impossible statically** because Ruby's method dispatch is determined at runtime.

#### Primary Extraction Tool

**ruby-lsp** — Shopify's opinionated Ruby language server. As of 2025, this is the recommended tool over solargraph for new projects.

- Repository: https://github.com/Shopify/ruby-lsp
- Protocol: standard LSP
- Backed by: **Prism** (the new Ruby parser, standard library in Ruby 3.3+)
- Extension model: third-party addons (e.g., `ruby-lsp-rails` for Rails-specific patterns)
- ruby-lsp-rails: https://github.com/Shopify/ruby-lsp-rails

**solargraph** — the previous standard Ruby language server. Still maintained and used, but receiving fewer updates.
- Repository: https://github.com/castwide/solargraph
- Backed by: the `parser` gem (whitequark/parser)
- Stronger documentation inference from YARD comments

#### Prism Parser

**Prism** is the new official Ruby parser, added as a standard library in Ruby 3.3 and built into CRuby:
- Repository: https://github.com/ruby/prism
- Gem: `prism` on RubyGems
- Ruby 3.4+: `Prism::Translation::Parser` provides compatibility with the `parser` gem API
- The `parser` gem (whitequark/parser) only supports Ruby syntax up to 3.3. For Ruby 3.4+, Prism is required.
- RuboCop has switched to Prism for Ruby 3.4+ syntax support

For Parseltongue, this means: use Prism for Ruby 3.4+ codebases, use the `parser` gem for Ruby 3.3 and below.

**Prism Rust bindings**: Prism exposes a C API, making it possible to call from Rust via `prism-sys` (FFI bindings). However, invoking a Ruby subprocess is simpler and more maintainable.

#### How to Drive ruby-lsp from Rust

Spawn ruby-lsp as a subprocess over stdio:

```rust
use std::process::{Command, Stdio};

let mut ruby_lsp = Command::new("ruby-lsp")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .env("BUNDLE_GEMFILE", "/path/to/project/Gemfile")
    .spawn()
    .expect("ruby-lsp not installed");

// Send LSP initialize, then:
// textDocument/documentSymbol — symbols in one file
// workspace/symbol — all symbols in project
// textDocument/references — usages of a symbol
// textDocument/definition — jump to definition
```

**Note**: ruby-lsp uses Prism internally and provides good symbol resolution. However, call hierarchy (`callHierarchy/incomingCalls`) is partially implemented as of early 2026. Check the ruby-lsp releases page for current status.

#### Alternative: Parse with Prism via Ruby subprocess

For direct AST access without the LSP overhead:

```ruby
# prism_analysis.rb — invoke from Rust via ruby subprocess
require 'prism'
require 'json'

result = {entities: [], edges: []}

Dir.glob(ARGV[0] + '/**/*.rb').each do |file|
  source = File.read(file)
  parse_result = Prism.parse(source)
  next unless parse_result.success?

  # Walk the AST
  parse_result.value.accept(Prism::Visitor.new do
    def visit_def_node(node)
      result[:entities] << {
        kind: 'method',
        name: node.name.to_s,
        file: file,
        line: node.location.start_line,
      }
      super
    end

    def visit_call_node(node)
      result[:edges] << {
        kind: 'call',
        callee: node.name.to_s,
        file: file,
        line: node.location.start_line,
      }
      super
    end
  end)
end

puts JSON.generate(result)
```

Invoke from Rust:

```rust
let output = Command::new("ruby")
    .args(&["prism_analysis.rb", "/path/to/project"])
    .output()
    .expect("ruby subprocess failed");
let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
```

**Critical limitation of this approach**: method call nodes in Prism (`visit_call_node`) capture the method name as a string but **cannot resolve which class/module defines the method** without runtime type information. You know that `foo.bar` calls a method named `bar`, but not which class `foo` is an instance of, unless Sorbet type annotations are present.

#### RuboCop AST for Pattern-Based Extraction

**rubocop-ast** provides a pattern-matching DSL on top of the parser gem (or Prism via translation layer):

```ruby
require 'rubocop-ast'
require 'parser/current'

source = Parser::CurrentRuby.parse(File.read('app.rb'))

# NodePattern matching
require_def = RuboCop::AST::NodePattern.new('(send nil? :require _)')
source.each_node do |node|
  if require_def.match(node)
    required = node.arguments.first.value
    puts "requires: #{required}"
  end
end
```

RuboCop AST is good for pattern-matching specific Ruby idioms (DSL method calls, specific require patterns, class/module definitions) without building a full scope analysis engine.

#### Sorbet for Type-Annotated Codebases

**Sorbet** is a fast, powerful static type checker for Ruby developed at Stripe and now open-source. It provides accurate type resolution where type annotations (Sorbet `sig` blocks or RBS inline comments) are present.

- Repository: https://github.com/sorbet/sorbet
- RBS inline comments support: added April 2025 (see railsatscale.com post)
- LSP mode: `srb --lsp`

Sorbet provides an LSP and can be interrogated programmatically. **Spoom** is the official Shopify toolbox for driving Sorbet programmatically:

```ruby
# Using spoom to connect to Sorbet LSP
require 'spoom'

lsp = Spoom::LSP::Client.new(
  Spoom::Config::SORBET_PATH,
  "--lsp",
  "--no-config"
)

lsp.open("/path/to/project")

# Hover: get type at position
type_info = lsp.hover("app/models/user.rb", line: 10, column: 5)

# References: find all uses of a symbol
refs = lsp.references("app/models/user.rb", line: 10, column: 5)
```

For Parseltongue, driving Sorbet's LSP via a Ruby subprocess running Spoom is the cleanest integration path for typed Ruby codebases.

**If Sorbet is not available** (most Ruby codebases are not typed):
- Callee resolution is heuristic-based (match method name to class definitions)
- `attr_reader`, `attr_accessor`, `delegate` — require pattern recognition
- Duck typing means resolution is never certain

#### What Ruby Tools Provide Beyond tree-sitter

| Capability | tree-sitter | ruby-lsp / Prism / Sorbet |
|---|---|---|
| Method resolution (untyped) | No | Heuristic (name-based) |
| Method resolution (typed + Sorbet) | No | Full type-accurate resolution |
| `require` / `require_relative` graph | Syntactic only | Resolved paths (ruby-lsp) |
| Module/class hierarchy | Syntactic only | Partial (ruby-lsp uses Prism index) |
| Block / yield analysis | No | Partial |
| `send` / `method_missing` | No | Not resolvable statically |
| Metaprogramming (`define_method`) | No | Not resolvable statically |
| YARD doc types | Syntactic only | solargraph infers from YARD |

#### Maturity / Reliability

ruby-lsp: Actively developed by Shopify, recommended for new projects as of 2025. Prism-backed for Ruby 3.4+.

solargraph: Mature, stable, good YARD documentation inference. Less active development in 2025.

Prism: Official Ruby parser as of Ruby 3.3+. Production-grade.

Sorbet: Production-grade, used at Stripe and Shopify. Requires significant upfront annotation work.

Spoom: Maintained by Shopify. Good programmatic interface to Sorbet.

#### Key Limitations

- Ruby's dynamic nature means that without Sorbet/RBS annotations, callee resolution is fundamentally approximate
- `method_missing` and `respond_to?` patterns are opaque to static analysis
- `require` resolution requires knowing the load path ($LOAD_PATH), which is set at runtime
- Metaprogramming (dynamically defined methods) is invisible to all static analyzers
- ruby-lsp's call hierarchy is incomplete as of early 2026

---

## 7. Ruby on Rails

### Extraction Stack: ruby-lsp-rails + railroady + rails-erd + annotate + Prism

Rails imposes conventions on top of Ruby that enable additional static extraction: ActiveRecord models follow naming conventions, controllers follow REST conventions, and routes are declared in `config/routes.rb`. These conventions make more information extractable than in arbitrary Ruby.

#### Primary Extraction Tool: ruby-lsp-rails

**ruby-lsp-rails** is the official Rails addon for ruby-lsp, developed by Shopify:
- Repository: https://github.com/Shopify/ruby-lsp-rails
- Provides: Rails-specific completions, route awareness, ActiveRecord association navigation, controller action linking

From a dependency graph perspective, ruby-lsp-rails provides:
- Navigation from controller actions to corresponding views
- ActiveRecord model-to-table association resolution
- Route-to-controller-action resolution

Drive via ruby-lsp LSP (same as section 6).

#### Rails Routes Extraction

The `rails routes` command outputs all registered routes. As of Rails 7+, it supports structured output:

```bash
# Human-readable (default)
bundle exec rails routes

# JSON output (merged in rails/rails via PR #37136, available Rails 6.1+)
bundle exec rails routes --format=json
```

The JSON format provides: verb, path pattern, controller, action, name (route helper name), and constraints.

For programmatic extraction from a Rust subprocess:

```rust
use std::process::Command;
use serde_json::Value;

let output = Command::new("bundle")
    .args(&["exec", "rails", "routes", "--format=json"])
    .current_dir("/path/to/rails/project")
    .output()
    .expect("bundle exec rails routes failed");

if output.status.success() {
    let routes: Value = serde_json::from_slice(&output.stdout)
        .expect("invalid JSON from rails routes");
    // routes is an array of {verb, path, controller, action, name}
}
```

**Programmatic access without rails CLI**: you can also load routes in a Ruby subprocess:

```ruby
# list_routes.rb
require_relative 'config/environment'

routes_data = Rails.application.routes.routes.map do |route|
  {
    verb: route.verb,
    path: route.path.spec.to_s,
    controller: route.defaults[:controller],
    action: route.defaults[:action],
    name: route.name,
  }
rescue => e
  nil
end.compact

puts routes_data.to_json
```

#### ActiveRecord Association Extraction

ActiveRecord associations (`has_many`, `belongs_to`, `has_one`, `has_and_belongs_to_many`, `has_many :through`) define the domain model relationships.

**Two approaches**:

**Static (Prism/parser gem AST)**: Parse model files and look for `send` nodes named `has_many`, `belongs_to`, etc. Does not require loading Rails — works without a database connection.

```ruby
# Extract associations statically using Prism
require 'prism'
require 'json'

ASSOCIATION_METHODS = %w[has_many belongs_to has_one has_and_belongs_to_many has_many through]

associations = []
Dir.glob('app/models/**/*.rb').each do |file|
  parse_result = Prism.parse(File.read(file))
  next unless parse_result.success?

  parse_result.value.each_node do |node|
    next unless node.is_a?(Prism::CallNode)
    next unless ASSOCIATION_METHODS.include?(node.name.to_s)

    associations << {
      file: file,
      type: node.name.to_s,
      target: node.arguments&.arguments&.first&.then { |a|
        a.is_a?(Prism::SymbolNode) ? a.value : nil
      },
    }
  end
end

puts associations.to_json
```

**Runtime (Rails reflection API)**: More accurate, but requires loading the full Rails environment and a database connection:

```ruby
# Extract associations via ActiveRecord reflection (requires Rails boot)
require_relative 'config/environment'

model_data = []
ActiveRecord::Base.descendants.each do |model|
  next unless model.respond_to?(:reflect_on_all_associations)

  model.reflect_on_all_associations.each do |reflection|
    model_data << {
      model: model.name,
      association_type: reflection.macro.to_s,
      associated_model: reflection.class_name,
      name: reflection.name.to_s,
    }
  end
end

puts model_data.to_json
```

The runtime approach is accurate but couples extraction to a running database. For Parseltongue's static analysis focus, the Prism-based static approach is preferred as a first pass.

#### Zeitwerk Autoloading — Module Resolution

**Zeitwerk** is Rails 6+'s code loader. It maps file paths to constant names using naming conventions:
- `app/models/user.rb` → `User`
- `app/models/admin/profile.rb` → `Admin::Profile`

For Parseltongue, this means: given a file path in a Rails project, the constant name is deterministically derivable from the file path relative to the autoload roots.

Zeitwerk's autoload mapping can be extracted statically:

```ruby
# zeitwerk_map.rb
require_relative 'config/environment'

loader = Zeitwerk::Registry.loaders.first  # Rails typically has one main loader
constantizable = {}

loader.autoload_paths.each do |path|
  Dir.glob("#{path}/**/*.rb").each do |file|
    relative = Pathname.new(file).relative_path_from(path).to_s.sub(/\.rb$/, '')
    constant_name = relative.split('/').map { |part|
      part.split('_').map(&:capitalize).join
    }.join('::')
    constantizable[file] = constant_name
  end
end

puts constantizable.to_json
```

This Zeitwerk mapping, combined with the Prism AST extraction, allows Parseltongue to resolve `User` to `app/models/user.rb` without runtime execution.

#### railroady — UML/DOT Diagrams for Rails

**railroady** generates UML class diagrams for Rails models and controllers:

```bash
gem install railroady

# Generate model diagram in DOT format
railroady -M | dot -Tsvg -o models.svg

# Generate controller diagram
railroady -C | dot -Tsvg -o controllers.svg

# Output DOT format for programmatic parsing
railroady -M > models.dot
```

The DOT output is parseable programmatically. Nodes are models/controllers; edges are associations/routes. railroady is simpler than rails-erd but requires less infrastructure.

```rust
// Parse railroady DOT output from Rust
let output = Command::new("railroady")
    .arg("-M")
    .current_dir("/path/to/rails/project")
    .output()
    .expect("railroady failed");

let dot_output = String::from_utf8(output.stdout).unwrap();
// Parse DOT format using petgraph's dot parser or a regex-based approach
```

#### rails-erd — Programmatic ActiveRecord ERD

**rails-erd** (voormedia/rails-erd) generates Entity-Relationship Diagrams from ActiveRecord models:

- Repository: https://github.com/voormedia/rails-erd
- Programmatic API: `RailsERD::Domain.generate`

```ruby
# rails_erd_extract.rb
require_relative 'config/environment'
require 'rails_erd/domain'

domain = RailsERD::Domain.generate

entities = domain.entities.map { |e| { name: e.name, file: e.model&.name } }
relationships = domain.relationships.map { |r|
  {
    from: r.source.name,
    to: r.destination.name,
    type: r.indirect? ? 'indirect' : 'direct',
  }
}

puts({ entities: entities, relationships: relationships }.to_json)
```

rails-erd requires ActiveRecord models to be loaded (Rails environment must be started).

#### annotate gem — Schema Comments in Model Files

**annotate** (annotate_models gem) adds schema information as comments to ActiveRecord model files. For Parseltongue, the schema annotations are a secondary signal: they confirm which columns exist on a model, which can be correlated with associations.

```bash
bundle exec annotate --models --show-indexes
```

After running annotate, model files contain comments like:

```ruby
# == Schema Information
#
# Table name: users
#
#  id         :bigint           not null, primary key
#  email      :string(255)      not null
#  name       :string(255)
#  created_at :datetime         not null
#  updated_at :datetime         not null
```

These comments can be parsed by Prism's comment extraction, giving Parseltongue table-to-model mappings without a database connection.

#### What Rails-Specific Tools Provide Beyond tree-sitter

| Capability | tree-sitter | Rails-specific tools |
|---|---|---|
| Route graph (verb + path + action) | No | rails routes --format=json |
| ActiveRecord association graph | No | Prism static + Rails reflection |
| Model-to-table mapping | No | annotate + schema.rb parsing |
| Zeitwerk constant resolution | No | Zeitwerk loader map |
| Controller-to-view linking | No | ruby-lsp-rails |
| ERD (entity-relationship) | No | rails-erd programmatic API |
| UML model/controller diagram | No | railroady DOT output |
| Concern/Mixin tracking | No | Prism `include`/`extend` extraction |

#### Maturity / Reliability

ruby-lsp-rails: Actively maintained by Shopify, increasingly feature-complete.
railroady: Maintained, works for Rails 3-7+. Simple DOT output.
rails-erd: Less actively maintained (last major update 2020), but still functional.
annotate (annotate_models): Widely used, actively maintained.
Prism for static Rails extraction: Stable for Ruby 3.3+ syntax.

#### Key Limitations

- Association extraction via Rails reflection requires booting the full Rails environment (database, environment variables, all gems loaded)
- Polymorphic associations (`belongs_to :commentable, polymorphic: true`) cannot be fully resolved statically — the target model is determined at runtime
- STI (Single Table Inheritance) complicates the model-to-table mapping
- `routes.rb` supports complex routing DSLs (nested resources, concerns, constraints) — full JSON extraction handles most patterns but edge cases exist
- Zeitwerk's autoload paths can be customized, so the deterministic file-to-constant mapping only works for standard Rails directory layouts
- rails-erd requires ActiveRecord model loading, which requires a valid database configuration

---

## 8. Cross-Cutting Concerns

### 8.1 Compilation Database Pattern

The `compile_commands.json` compilation database is the standard way to tell analysis tools how to compile each source file. The pattern:

```
[
  {
    "directory": "/project",
    "file": "/project/src/main.c",
    "command": "clang -I/project/include -DDEBUG main.c -o main",
    "arguments": ["clang", "-I/project/include", "-DDEBUG", "main.c", "-o", "main"]
  }
]
```

| Build System | How to Generate |
|---|---|
| CMake | `cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=ON` |
| Ninja | `ninja -t compdb > compile_commands.json` |
| Make (arbitrary) | `bear -- make` |
| Meson | `meson setup build; cd build; meson compile` |
| Bazel | `bazel run @hedron_compile_commands//:refresh_all` |
| Cargo (Rust) | Not needed; rust-analyzer reads Cargo.toml |

For Parseltongue, when indexing a C or C++ project, detecting whether `compile_commands.json` exists (or can be generated) should be a prerequisite check before launching clangd.

### 8.2 LSP as Universal Driver

All language servers described in this document implement the Language Server Protocol (LSP) over JSON-RPC. From Rust, the driver pattern is always:

1. Spawn the language server as a child process (stdin/stdout pipes)
2. Send `initialize` request with workspace root
3. Send `initialized` notification
4. Open virtual documents via `textDocument/didOpen`
5. Send analysis requests (`workspace/symbol`, `callHierarchy/*`, `textDocument/references`)
6. Collect JSON responses
7. Send `shutdown` and `exit`

The `lsp-types` crate provides typed Rust structs for all standard LSP messages. The `tower-lsp` crate provides async server/client infrastructure. The `lsp-client-rs` crate provides a minimal async client for driving external language servers.

```rust
// Minimal LSP initialization message
let init_params = serde_json::json!({
    "processId": std::process::id(),
    "clientInfo": { "name": "parseltongue", "version": "2.0.0" },
    "rootUri": format!("file://{}", workspace_root),
    "capabilities": {
        "workspace": {
            "symbol": { "resolveSupport": { "properties": ["location"] } }
        },
        "textDocument": {
            "callHierarchy": { "dynamicRegistration": false }
        }
    }
});
```

### 8.3 Cross-Language Boundary Detection

Parseltongue v2.0 needs to detect edges that cross language boundaries. The key boundary types:

**Rust FFI → C**:
- Detect `extern "C"` blocks in Rust source (tree-sitter or rust-analyzer)
- Match `#[link(name = "libfoo")]` attributes to C library names
- Cross-reference with `compile_commands.json` to find the C source being linked
- Tools: `bindgen` (reads C headers, generates Rust types), `cbindgen` (reads Rust pub extern C, generates C headers)

**C → Rust**:
- Rust functions with `pub extern "C"` and `#[no_mangle]` are callable from C
- C source files that include headers generated by `cbindgen` are identified via IWYU analysis

**TypeScript → JavaScript**:
- TypeScript can import `.js` files when `allowJs: true` or via `import type { ... } from './file.js'`
- TypeScript compiler API's `getResolvedModuleWithFailedLookupLocations()` shows cross-type-boundary imports
- dependency-cruiser tracks these transparently

**JavaScript → TypeScript (via .d.ts files)**:
- `.d.ts` declaration files provide type information for JS libraries
- TypeScript Compiler API resolves `@types/*` packages automatically

**Ruby → C extensions (native gems)**:
- Ruby gems with C extensions have `.so` / `.bundle` native code
- Detection: check `gem specification --local <gemname>` for `extensions` field
- Static analysis across this boundary is not feasible without decompilation

### 8.4 Language-Server Tools without Full LSP

Several tools expose programmatic APIs without running as an LSP server:

| Tool | Language | Interface | Notes |
|---|---|---|---|
| `ra_ap_ide` | Rust | Rust library (in-process) | Unstable API, tracks ra nightly |
| `ts-morph` | TypeScript | Node.js library | Stable, wraps tsc compiler API |
| `dependency-cruiser` | TS/JS | Node.js library + CLI | Stable, JSON output |
| `oxc_parser` | JS/TS | Rust library (in-process) | Production-grade parser |
| `oxc_resolver` | JS/TS | Rust library (in-process) | Production-grade resolver |
| `Prism` | Ruby | Ruby library + C API | Official Ruby 3.3+ |
| `Spoom` | Ruby | Ruby library (Sorbet LSP client) | Requires Sorbet |
| `RailsERD::Domain` | Rails | Ruby library (in-process) | Requires Rails env |
| `libclang` (clang-sys) | C/C++ | Rust FFI bindings | Stable C API |
| `cflow` | C | CLI subprocess | Simple, text output |

---

## 9. Summary Comparison Table

| Language | Primary Tool | Invocation from Rust | Type Resolution | Call Graph | Module Graph | Framework Extras | Maturity |
|---|---|---|---|---|---|---|---|
| **Rust** | rust-analyzer | LSP subprocess or `ra_ap_ide` (in-process) | Full (salsa-based, cross-crate) | Full (incomingCalls/outgoingCalls) | Full (`use` path resolution) | Macro expansion, cargo workspace | Production |
| **C** | clangd + compile_commands.json | LSP subprocess | Full (libclang semantic) | Partial (LSP callHierarchy, incomplete for large projects) | Full (#include graph via libclang) | N/A | Production |
| **C++** | clangd + IWYU + Doxygen | LSP subprocess + CLI subprocess | Full (templates partial) | Partial (virtual dispatch approximated) | Full (header graph via IWYU) | N/A (template meta-programming opaque) | Production |
| **TypeScript** | TypeScript Compiler API / ts-morph | Node.js subprocess (ts-morph script) or LSP (typescript-language-server) | Full (TypeChecker, barrel files) | Full (TypeChecker.getSymbolAtLocation) | Full (tsconfig paths, barrel re-exports) | React component detection (heuristic) | Production |
| **JavaScript** | TypeScript Compiler API (allowJs) + OXC | Node.js subprocess or Rust-native (oxc_parser, oxc_resolver) | Inferred/heuristic (no annotations) | Heuristic (name-based, no type info) | Full module resolution (oxc_resolver) | N/A (framework-specific heuristics) | Production (OXC v1.0) |
| **Ruby** | ruby-lsp (Prism-backed) + Sorbet (if typed) | LSP subprocess + Ruby subprocess | Heuristic (untyped) / Full (Sorbet) | Heuristic (untyped) / Full (Sorbet LSP) | require/require_relative graph (partial) | YARD docs (solargraph) | Production (ruby-lsp) |
| **Ruby on Rails** | ruby-lsp-rails + railroady + rails-erd + Prism | Ruby subprocess + CLI subprocess | Heuristic (untyped) + ActiveRecord reflection | N/A (focus on model/route/association graphs) | Zeitwerk constant map | Routes, associations, ERD, annotate schema | Production (individual tools) |

---

## 10. Recommended Extraction Stack Per Language

This section specifies the concrete extraction stack Parseltongue v2.0 should implement for each language, ordered by priority.

### 10.1 Rust — Recommended Stack

**Tier 1 (always run):**
1. `rust-analyzer` — spawn as LSP subprocess; use `workspace/symbol` for all entities, `callHierarchy/outgoingCalls` per function for call graph construction
2. tree-sitter-rust — fast syntactic pre-pass for file inventory and basic structure

**Tier 2 (when call graph depth is needed):**
3. `cargo-call-stack` — LLVM IR-based static call stack analysis for embedded/safety-critical paths

**Tier 3 (FFI boundary detection):**
4. `bindgen` — run on C headers linked by the project; import the generated Rust types into the graph
5. `cbindgen` — run on Rust project; emit C headers, then parse with libclang

**Invocation pattern**: spawn rust-analyzer with `--no-rustc-wrapper` flag; send `initialize`, wait for `window/workDoneProgress` to complete (indexing done), then batch `workspace/symbol` and `callHierarchy/*` requests.

### 10.2 C — Recommended Stack

**Tier 1 (always run):**
1. Generate `compile_commands.json` — detect CMake (check for `CMakeLists.txt`), Makefile (offer Bear), or hand-written flags
2. `clangd` — spawn as LSP subprocess with `--compile-commands-dir`; use `workspace/symbol` and `callHierarchy/*`

**Tier 2 (when compilation database unavailable):**
3. `cflow` — subprocess; text output parsed for caller/callee pairs; no type resolution

**Tier 3 (header dependency graph):**
4. `libclang` via `clang-sys` (in-process) — walk AST per translation unit; extract `#include` edges and function call edges with full type resolution

**Invocation pattern**: `clangd` provides the most complete analysis. Fall back to `cflow` if no compilation database is found.

### 10.3 C++ — Recommended Stack

**Tier 1 (always run):**
1. Generate `compile_commands.json` (same as C)
2. `clangd` — LSP subprocess; all standard LSP methods including `callHierarchy/*`

**Tier 2 (header dependency analysis):**
3. `include-what-you-use` (IWYU) — subprocess; parse output for per-file include requirements

**Tier 3 (documentation-grade call graph):**
4. `doxygen --generate-xml` — subprocess; parse `doxygen_xml/` for callee/caller relationships

**Tier 4 (virtual dispatch — research-grade):**
5. `phasar` (LLVM-based) — for projects where virtual dispatch accuracy is critical; significant setup cost

**Invocation pattern**: identical to C but with C++ flags. IWYU adds a second pass. Doxygen XML is a supplementary source, not the primary.

### 10.4 TypeScript — Recommended Stack

**Tier 1 (always run):**
1. `typescript-language-server` — LSP subprocess; `workspace/symbol`, `callHierarchy/*`, `textDocument/references`
2. ts-morph Node.js worker script — for bulk export/import edge extraction without LSP overhead per symbol

**Tier 2 (module-level graph):**
3. `dependency-cruiser` — Node.js subprocess or Node.js API call; provides the complete module dependency graph including path alias resolution

**Tier 3 (TypeScript 7 transition, late 2026):**
4. `@typescript/native-preview` (typescript-go) — replace tsserver with native LSP when TypeScript 7 stabilizes

**Invocation pattern**: run dependency-cruiser first (fast, gives file-level graph), then use typescript-language-server for symbol-level resolution. The ts-morph worker script fills gaps for bulk export enumeration.

### 10.5 JavaScript — Recommended Stack

**Tier 1 (always run, Rust-native):**
1. `oxc_parser` crate — parse all JS files in-process; fastest available JS parser in Rust
2. `oxc_resolver` crate — resolve all import specifiers in-process; no subprocess needed

**Tier 2 (type-inferred analysis):**
3. TypeScript Compiler API with `allowJs: true` — Node.js subprocess; provides JSDoc type inference and cross-file symbol resolution

**Tier 3 (module-level graph):**
4. `dependency-cruiser` — same as TypeScript tier 2; handles CommonJS and ESM

**Invocation pattern**: OXC handles the in-process fast path (parse + resolve). The TypeScript compiler API subprocess adds type information. dependency-cruiser validates the module graph with rules.

### 10.6 Ruby — Recommended Stack

**Tier 1 (always run):**
1. Prism parser via Ruby subprocess — extract all class/module/method definitions and `require`/`require_relative` edges statically; no Rails boot needed
2. `ruby-lsp` — LSP subprocess; `workspace/symbol`, `textDocument/references`, `textDocument/definition`

**Tier 2 (type-annotated codebases):**
3. Sorbet LSP (`srb --lsp`) via Spoom — when `sorbet/` directory detected in project; provides type-accurate callee resolution

**Tier 3 (documentation inference):**
4. solargraph — LSP subprocess; better YARD documentation inference than ruby-lsp; run in parallel or as fallback

**Invocation pattern**: Prism subprocess runs first for structural extraction (fast, no Ruby runtime needed). ruby-lsp subprocess adds symbol resolution. Detect Sorbet presence by checking for `sorbet/config` or `Gemfile` containing `sorbet` gem.

### 10.7 Ruby on Rails — Recommended Stack

**Tier 1 (always run, no Rails boot):**
1. Prism subprocess — extract model definitions, controller actions, concern inclusions, `has_many`/`belongs_to` method calls (static, approximate)
2. Zeitwerk file-to-constant mapping — compute from directory structure without booting Rails
3. `schema.rb` parsing — parse `db/schema.rb` with tree-sitter or Prism; extract table-to-column mappings

**Tier 2 (requires Rails environment boot):**
4. `rails routes --format=json` — subprocess; complete route graph with verb/path/controller/action
5. `ActiveRecord::Base.descendants` + `reflect_on_all_associations` — Ruby subprocess loading Rails environment; complete association graph

**Tier 3 (visualization-grade):**
6. `railroady -M` — subprocess; DOT output for model diagram (parseable for additional validation)
7. `rails-erd` (`RailsERD::Domain.generate`) — Ruby subprocess; ERD with indirect relationships

**Tier 4 (metadata enrichment):**
8. annotate schema comments — parse existing schema annotations in model files with Prism (if annotate has been run in the project)
9. `ruby-lsp-rails` — LSP addon; controller-to-view navigation, route awareness

**Invocation pattern**: Tier 1 (static analysis) always runs without Rails boot — safe for CI, no database needed. Tier 2 requires `RAILS_ENV=development bundle exec ruby` — offer as opt-in or detect availability. Tier 3+ are for full analysis mode.

---

## References

- [rust-analyzer LSP Integration](https://rust-lang.github.io/rust-analyzer/rust_analyzer/)
- [ra_ap_ide crate documentation](https://docs.rs/ra_ap_ide)
- [rust-analyzer call hierarchy source](https://github.com/rust-lang/rust-analyzer/blob/master/crates/ide/src/call_hierarchy.rs)
- [Bear compilation database generator](https://github.com/rizsotto/Bear)
- [clangd-mcp-server (LSP client example in Rust-adjacent pattern)](https://github.com/felipeerias/clangd-mcp-server)
- [include-what-you-use](https://include-what-you-use.org/)
- [ts-morph GitHub](https://github.com/dsherret/ts-morph)
- [typescript-language-server](https://github.com/typescript-language-server/typescript-language-server)
- [TypeScript 7 native port (typescript-go)](https://github.com/microsoft/typescript-go)
- [TypeScript 7 progress blog post (December 2025)](https://devblogs.microsoft.com/typescript/progress-on-typescript-7-december-2025/)
- [dependency-cruiser API documentation](https://github.com/sverweij/dependency-cruiser/blob/main/doc/api.md)
- [OXC project](https://oxc.rs/)
- [oxc-resolver crate](https://crates.io/crates/oxc_resolver)
- [Prism Ruby parser](https://github.com/ruby/prism)
- [RuboCop AST](https://github.com/rubocop/rubocop-ast)
- [ruby-lsp (Shopify)](https://github.com/Shopify/ruby-lsp)
- [ruby-lsp-rails (Shopify)](https://github.com/Shopify/ruby-lsp-rails)
- [Sorbet type checker](https://sorbet.org/)
- [Spoom toolbox for Sorbet](https://github.com/Shopify/spoom)
- [railroady gem](https://github.com/preston/railroady)
- [rails-erd gem](https://github.com/voormedia/rails-erd)
- [annotate_models gem](https://github.com/ctran/annotate_models)
- [zeitwerk code loader](https://github.com/fxn/zeitwerk)
- [cargo-call-stack](https://github.com/japaric/cargo-call-stack)
- [cargo-geiger](https://github.com/geiger-rs/cargo-geiger)
- [tower-lsp crate](https://github.com/ebkalderon/tower-lsp)
- [lsp-types crate](https://docs.rs/lsp-types)
- [tree-sitter-stack-graphs (name resolution at scale)](https://crates.io/crates/tree-sitter-stack-graphs)
