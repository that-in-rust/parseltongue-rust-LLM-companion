# rust-analyzer Idiomatic Patterns Analysis State

## TDD Session State: 2026-02-20

### Current Phase: Complete - Documentation Generated

### Analysis Completed:
- **Domain:** crates/ide (IDE features: goto def, hover, completion entry points)
- **Files Analyzed:** 15+ source files
- **Patterns Identified:** 22 comprehensive patterns
- **Output:** `/Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-idiomatic-patterns/04-ide-features-patterns.md`

### Files Read and Analyzed:

1. **lib.rs** - Core Analysis/AnalysisHost architecture, cancellation patterns
2. **fixture.rs** - Test infrastructure with $0 markers
3. **markup.rs** - Rich text markdown output
4. **goto_definition.rs** - Token descent, IdentClass classification
5. **hover.rs** - HoverResult with actions, config patterns
6. **navigation_target.rs** - ToNav/TryToNav traits, UpmappingResult
7. **references.rs** - Reference search with categories
8. **signature_help.rs** - Active parameter tracking
9. **call_hierarchy.rs** - Bidirectional call graph traversal
10. **syntax_highlighting.rs** - Semantic highlighting with tags/modifiers
11. **inlay_hints.rs** - Incremental rendering with range pruning
12. **runnables.rs** - Test/binary discovery with macro handling

### Patterns Documented (22 Total):

#### Architecture Patterns (1-4):
1. **AnalysisHost/Analysis Snapshot Pattern** - Mutable host, immutable snapshots
2. **with_db Cancellation Wrapper** - Salsa unwinding-based cancellation
3. **FilePosition/FileRange Parameters** - Standardized position types
4. **RangeInfo Return Type** - Results enriched with triggering range

#### Token & Name Resolution (5-7):
5. **Token Classification with pick_best_token** - Ranked token selection
6. **Semantics-Based Token Descent** - Macro-aware analysis
7. **IdentClass Pattern for Name Resolution** - Unified name classification

#### Navigation & Results (8-10):
8. **NavigationTarget Builder Pattern** - Structured navigation with full/focus ranges
9. **ToNav/TryToNav Trait Pattern** - Polymorphic navigation target creation
10. **Test Fixture Infrastructure** - Annotated test helpers ($0 markers)

#### Rich Output (11-13):
11. **Markup Generation for Rich Text** - Markdown output type safety
12. **Hover Result with Actions** - Actionable information display
13. **Config Structs for Feature Customization** - Grouped configuration

#### Advanced Features (14-19):
14. **Signature Help with Active Parameter** - String building with range tracking
15. **Call Hierarchy with Bidirectional Search** - Graph traversal patterns
16. **Reference Search with Category Metadata** - Categorized reference results
17. **Syntax Highlighting with Layered Tags** - Tag + modifier system
18. **Inlay Hints with Incremental Rendering** - Range-based tree pruning
19. **Runnable Discovery with Test Detection** - Macro-aware test finding

#### Cross-Cutting Concerns (20-22):
20. **UpmappingResult for Macro Call/Definition Sites** - Dual-site navigation
21. **Module Feature Organization Pattern** - Flat feature module structure
22. **Edition-Aware Parsing** - Multi-edition Rust support

### Key Insights:

1. **Cancellation is Fundamental**: Every public Analysis method uses `with_db` wrapper. Salsa uses panic unwinding for cancellation. All feature functions must be UnwindSafe.

2. **Macro Awareness Everywhere**: Token descent (`descend_into_macros`), UpmappingResult dual sites, and special handling for macro-generated items are pervasive patterns.

3. **Position-Based APIs**: FilePosition for point queries, FileRange for range queries, RangeInfo for results. This standardization makes the API predictable.

4. **Semantic Classification**: IdentClass, NameClass, NameRefClass unify name resolution. All features use these abstractions rather than raw syntax.

5. **Test Infrastructure**: Fixture module with $0 markers, annotations, multi-file support makes testing IDE features straightforward and readable.

6. **Config-Driven Features**: Complex features take Config structs enabling customization without API breakage.

7. **Rich Results**: Results combine information (Markup, NavigationTarget) with actions (HoverAction, RunnableKind) for interactive editor UX.

### Context Notes:

- **File Structure**: crates/ide contains 30+ feature modules as siblings, each implementing one IDE capability (goto-def, hover, completion, etc.)

- **Database Access**: Features receive &RootDatabase from Analysis.with_db, never mutate it

- **Edition Handling**: Rust 2015/2018/2021/2024 differences require edition-aware parsing and keyword recognition

- **Performance**: Inlay hints and syntax highlighting use range-based pruning to only compute visible results

- **Macro Complexity**: Macro expansions create dual navigation sites (call site vs definition site), handled by UpmappingResult

### Technical Debt Identified:

- Some features have TODO comments about improving edition handling
- Documentation could be more explicit about cancellation requirements
- Some navigation functions have FIXME notes about improving macro site selection

### Dependencies:
- **hir**: Semantic analysis (types, name resolution, etc.)
- **ide_db**: Common utilities (RootDatabase, FileId, Semantics, etc.)
- **syntax**: Syntax tree (AST, SyntaxNode, tokens)
- **test_fixture**: Test infrastructure
- **salsa**: Incremental computation with cancellation

### Next Steps (if continuing analysis):

1. Analyze crates/ide-completion for completion-specific patterns
2. Analyze crates/ide-assists for code action patterns
3. Analyze crates/ide-diagnostics for diagnostic patterns
4. Create cross-feature comparison document
5. Extract reusable trait patterns for new feature implementation

### Performance/Metrics:
- **Files analyzed**: 15+ Rust source files
- **Lines of code examined**: ~5000+ lines
- **Documentation generated**: 22 comprehensive patterns with examples
- **Output size**: ~1200 lines of markdown

---

## Pattern Quality Assessment:

### Coverage:
- ✅ Feature module organization
- ✅ Analysis/AnalysisHost architecture
- ✅ Cancellation-aware computation
- ✅ FilePosition/FileRange usage
- ✅ Test infrastructure (fixtures)
- ✅ Markup generation
- ✅ Navigation target patterns
- ✅ Token descent for macros
- ✅ Config struct patterns
- ✅ Rich result types with actions

### Completeness:
All major architectural patterns in crates/ide have been documented with:
- Real code examples from the codebase
- Clear explanations of why they matter
- Guidance for contributors
- Context about when to use each pattern

### Actionability:
Each pattern includes sufficient detail for contributors to:
- Recognize when the pattern applies
- Implement features following established conventions
- Understand the rationale behind design choices
- Avoid common pitfalls
