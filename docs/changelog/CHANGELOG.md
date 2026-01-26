# Changelog

All notable changes to Parseltongue will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **TypeScript import tracking**: Fixed dependency graph not capturing JS/TS import relationships. The TypeScript dependency query (`dependency_queries/typescript.scm`) was missing proper `@reference.*` captures needed by `process_dependency_match()` to extract module names. Added comprehensive patterns for:
  - ES module imports (`import x from 'module'`)
  - Named imports (`import { foo, bar } from 'module'`)
  - Type imports (`import type { Foo } from 'module'`)
  - Namespace imports (`import * as ns from 'module'`)
  - Dynamic imports (`await import('./module')`)
  - CommonJS requires (`require('module')`)
  - Class inheritance (`extends`)
  - Interface inheritance (`extends`)
  - Interface implementation (`implements`)

- **TypeScript grammar patterns**: Fixed tree-sitter query patterns for TypeScript class declarations:
  - Use `type_identifier` instead of `identifier` for class names (TypeScript AST difference)
  - Handle `member_expression` in extends clause for patterns like `React.Component`

## [1.3.0] - 2026-01-26

### Added

- D09-D12 Parseltongue architecture and design documents
- D08 Principal Rust Engineer Interview documentation

## [1.2.0] - 2026-01-23

### Added

- Dependency query infrastructure (v0.9.0 feature)
- Support for 12 languages in dependency extraction
- `DependencyEdges` table in CozoDB storage
- Edge types: `Calls`, `Uses`, `Implements`

### Changed

- Query extractor now returns `(Vec<ParsedEntity>, Vec<DependencyEdge>)` tuple

## [1.1.0] - 2026-01-20

### Added

- Query-based entity extraction using tree-sitter `.scm` files
- Support for 12 languages: Rust, Python, C, C++, Ruby, JavaScript, TypeScript, Go, Java, PHP, C#, Swift
- HTTP query server with 15 analysis endpoints
- Circular dependency detection
- Forward/reverse call graph queries

### Performance

- <20ms per 1K LOC parsing (release mode)
- 67% code reduction vs imperative extractors

## [1.0.0] - 2026-01-15

### Added

- Initial release
- ISGL1 key format for code entity identification
- CozoDB storage backend with RocksDB persistence
- File streaming and indexing (pt01)
- Basic entity extraction for Rust and Python

[Unreleased]: https://github.com/that-in-rust/parseltongue-dependency-graph-generator/compare/v1.3.0...HEAD
[1.3.0]: https://github.com/that-in-rust/parseltongue-dependency-graph-generator/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/that-in-rust/parseltongue-dependency-graph-generator/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/that-in-rust/parseltongue-dependency-graph-generator/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/that-in-rust/parseltongue-dependency-graph-generator/releases/tag/v1.0.0
