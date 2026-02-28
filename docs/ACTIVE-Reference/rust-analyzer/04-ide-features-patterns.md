# Idiomatic Rust Patterns: IDE Features Layer
> Source: rust-analyzer/crates/ide
> Purpose: Patterns to guide contributions to rust-analyzer

## Pattern 1: AnalysisHost/Analysis Snapshot Pattern
**File:** `crates/ide/src/lib.rs`
**Category:** Cancellation-Aware Architecture
**Code Example:**
```rust
/// `AnalysisHost` stores the current state of the world.
#[derive(Debug)]
pub struct AnalysisHost {
    db: RootDatabase,
}

impl AnalysisHost {
    /// Returns a snapshot of the current state, which you can query for
    /// semantic information.
    pub fn analysis(&self) -> Analysis {
        Analysis { db: self.db.clone() }
    }

    /// Applies changes to the current state of the world. If there are
    /// outstanding snapshots, they will be canceled.
    pub fn apply_change(&mut self, change: ChangeWithProcMacros) {
        self.db.apply_change(change);
    }
}

/// Analysis is a snapshot of a world state at a moment in time. It is the main
/// entry point for asking semantic information about the world. When the world
/// state is advanced using `AnalysisHost::apply_change` method, all existing
/// `Analysis` are canceled (most method return `Err(Canceled)`).
#[derive(Debug)]
pub struct Analysis {
    db: RootDatabase,
}

pub type Cancellable<T> = Result<T, Cancelled>;
```
**Why This Matters for Contributors:** This is the fundamental pattern for implementing IDE features in rust-analyzer. The AnalysisHost owns the mutable database, while Analysis provides immutable snapshots. When the user edits code, AnalysisHost applies changes and cancels all pending queries. Every IDE feature method returns `Cancellable<T>` to handle graceful cancellation. Contributors implementing new features must use `self.with_db(|db| ...)` pattern which wraps queries in cancellation handlers.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Concurrency - Snapshot Isolation + Cancellation
**Rust-Specific Insight:** This pattern demonstrates Rust's ownership system at architectural scale. Clone on RootDatabase creates a cheap Salsa snapshot (Arc-based incremental computation), not a deep copy. The cancellation via panic unwinding leverages Rust's panic safety guarantees - Salsa catches the unwind at API boundaries, converting it to `Result<T, Cancelled>`. This is only safe because Rust guarantees that unwinding either completes successfully or aborts, never leaving inconsistent state.
**Contribution Tip:** When adding new IDE features, ALWAYS wrap database queries in `with_db`. Never access `self.db` directly from Analysis methods. The `UnwindSafe` bound is critical - avoid capturing `Rc`/`RefCell` in closures. Test cancellation behavior by triggering rapid file edits during long-running operations.
**Common Pitfalls:**
- Forgetting `with_db` wrapper → cancellation won't work, queries see inconsistent state
- Capturing non-UnwindSafe types → compilation error or runtime panic
- Holding mutable state across `with_db` → violates panic safety
- Not handling `Cancelled` in calling code → propagate with `?` or convert to appropriate error
**Related Patterns in Ecosystem:**
- Salsa query framework (incremental computation)
- Tower's `Service` pattern (snapshottable request handlers)
- `crossbeam-epoch` (similar snapshot semantics for concurrent data structures)
- MVCC in databases (snapshot isolation inspiration)

---

## Pattern 2: with_db Cancellation Wrapper
**File:** `crates/ide/src/lib.rs`
**Category:** Cancellation-Aware Computation
**Code Example:**
```rust
impl Analysis {
    /// Performs an operation on the database that may be canceled.
    ///
    /// rust-analyzer needs to be able to answer semantic questions about the
    /// code while the code is being modified. A common problem is that a
    /// long-running query is being calculated when a new change arrives.
    ///
    /// We can't just apply the change immediately: this will cause the pending
    /// query to see inconsistent state (it will observe an absence of
    /// repeatable read). So what we do is we **cancel** all pending queries
    /// before applying the change.
    ///
    /// Salsa implements cancellation by unwinding with a special value and
    /// catching it on the API boundary.
    fn with_db<F, T>(&self, f: F) -> Cancellable<T>
    where
        F: FnOnce(&RootDatabase) -> T + std::panic::UnwindSafe,
    {
        hir::attach_db_allow_change(&self.db, || Cancelled::catch(|| f(&self.db)))
    }

    pub fn goto_definition(
        &self,
        position: FilePosition,
        config: &GotoDefinitionConfig<'_>,
    ) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>> {
        self.with_db(|db| goto_definition::goto_definition(db, position, config))
    }
}
```
**Why This Matters for Contributors:** Every public method in Analysis that queries the database MUST use `with_db` wrapper. This ensures that salsa's cancellation mechanism works correctly. The closure passed to `with_db` must be `UnwindSafe` because cancellation is implemented via panic unwinding. Never call database methods directly on `self.db` from Analysis methods - always go through `with_db`.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Error Handling - Structured Panic Recovery
**Rust-Specific Insight:** This is one of the rare legitimate uses of panic-as-control-flow in Rust. Salsa uses a special sentinel panic value for cancellation, catching it with `std::panic::catch_unwind`. The `UnwindSafe` bound is Rust's way of ensuring the closure won't leave shared state inconsistent if unwound. Note that `Cancelled::catch` wraps the database call, converting the special panic to `Err(Cancelled)`. This pattern requires deep understanding of Rust's panic mechanics: panics unwind the stack, running destructors (RAII cleanup), but can be caught at strategic boundaries.
**Contribution Tip:** To understand if a type is `UnwindSafe`, check: Does it contain `Rc<RefCell<T>>`? (not UnwindSafe). Does it contain `Arc<Mutex<T>>`? (is UnwindSafe via `std::panic::AssertUnwindSafe` if needed). Use `std::panic::AssertUnwindSafe` wrapper ONLY if you can prove the closure maintains invariants despite unwinding.
**Common Pitfalls:**
- Using `&mut` references inside the closure → not UnwindSafe by default
- Capturing `Rc<RefCell<T>>` → violates UnwindSafe, indicates potential state corruption
- Forgetting to propagate `Cancellable<T>` → swallowing cancellation signal
- Using bare `unwrap()` on Cancellable → panic instead of graceful cancellation
**Related Patterns in Ecosystem:**
- `std::panic::catch_unwind` for FFI boundaries
- Tokio's `JoinHandle` (propagates panics across tasks)
- `rayon` parallel iterators (panic in one thread cancels others)
- Embedded `no_std` panic handlers (abort vs unwind strategies)

---

## Pattern 3: FilePosition/FileRange Parameters
**File:** `crates/ide/src/lib.rs`
**Category:** Position-Based Feature APIs
**Code Example:**
```rust
/// Returns the definitions from the symbol at `position`.
pub fn goto_definition(
    &self,
    position: FilePosition,
    config: &GotoDefinitionConfig<'_>,
) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>> {
    self.with_db(|db| goto_definition::goto_definition(db, position, config))
}

/// Returns a short text describing element at position.
pub fn hover(
    &self,
    config: &HoverConfig<'_>,
    range: FileRange,
) -> Cancellable<Option<RangeInfo<HoverResult>>> {
    self.with_db(|db| hover::hover(db, range, config))
}

// From ide_db:
pub struct FilePosition {
    pub file_id: FileId,
    pub offset: TextSize,
}

pub struct FileRange {
    pub file_id: FileId,
    pub range: TextRange,
}
```
**Why This Matters for Contributors:** IDE features take either `FilePosition` (point queries like goto-definition) or `FileRange` (range queries like hover on selection). This standardization makes APIs predictable. Features that operate on a single point use FilePosition, features that can work on ranges use FileRange. Return types often wrap results in `RangeInfo<T>` which includes the text range the result applies to - crucial for editor highlighting.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** API Design - Newtype Pattern for Domain Modeling
**Rust-Specific Insight:** These are textbook newtypes that provide type safety over raw tuples. `FilePosition` prevents mixing up `(FileId, usize)` with `(usize, FileId)`. The pattern follows Rust's zero-cost abstraction principle: these structs have `#[repr(transparent)]` semantics (though not explicitly marked), so they compile to the same machine code as raw fields. The use of `TextSize` (from rowan) instead of `usize` adds another layer of type safety - you can't accidentally use byte indices where character indices are expected.
**Contribution Tip:** When designing new IDE APIs, follow this pattern: create domain-specific parameter types instead of accepting multiple primitives. This makes call sites self-documenting: `goto_definition(position, config)` vs `goto_definition(file_id, offset, config)`. The destructuring pattern `FilePosition { file_id, offset }` in function signatures is idiomatic and improves readability.
**Common Pitfalls:**
- Using raw `(FileId, TextSize)` tuples → loses type safety, breaks API evolution
- Confusing `TextSize` (UTF-8 byte offset) with character/line positions
- Not handling empty ranges in FileRange → can cause assertion failures
- Forgetting to normalize ranges (start <= end) → undefined behavior in range operations
**Related Patterns in Ecosystem:**
- `std::ops::Range` (similar range abstraction)
- LSP `Position` and `Range` types (protocol-level equivalents)
- `tower::Service` request types (similar domain parameter wrapping)
- Game engines use similar patterns for world positions (Vec2/Vec3 wrappers)

---

## Pattern 4: RangeInfo Return Type
**File:** `crates/ide/src/lib.rs`
**Category:** Result Enrichment Pattern
**Code Example:**
```rust
/// Info associated with a text range.
#[derive(Debug, UpmapFromRaFixture)]
pub struct RangeInfo<T> {
    pub range: TextRange,
    pub info: T,
}

impl<T> RangeInfo<T> {
    pub fn new(range: TextRange, info: T) -> RangeInfo<T> {
        RangeInfo { range, info }
    }
}

// Usage example from goto_definition:
pub(crate) fn goto_definition(
    db: &RootDatabase,
    FilePosition { file_id, offset }: FilePosition,
    config: &GotoDefinitionConfig<'_>,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    // ... token finding logic ...
    let navs = navs.into_iter().unique().collect();

    Some(RangeInfo::new(original_token.text_range(), navs))
}
```
**Why This Matters for Contributors:** Many IDE features return `Option<RangeInfo<T>>` to communicate both the result AND which text range triggered it. This allows editors to highlight the relevant portion of code. For example, goto-definition highlights the identifier you clicked, while returning the destination. The pattern is: find the triggering token/node, compute the result, then wrap both in RangeInfo. This is essential for features like hover, goto-definition, references.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Data Structures - Result Enrichment / Product Type
**Rust-Specific Insight:** `RangeInfo<T>` is a generic product type (struct with two fields) that enriches any result with spatial metadata. The generic parameter `T` makes it reusable across all IDE features - classic Rust polymorphism. The `#[derive(UpmapFromRaFixture)]` is a custom proc macro for test infrastructure, showing how rust-analyzer extends the type system with domain-specific derives. This pattern is superior to returning `(TextRange, T)` tuples because it's self-documenting and allows adding fields without breaking callers.
**Contribution Tip:** Always return `RangeInfo<T>` when your feature result should highlight a specific source location. Set `range` to the token that triggered the feature (what user clicked), not the entire node. For example, in `struct Foo { bar: i32 }`, clicking `bar` should highlight just "bar", not the entire field definition. Use `RangeInfo::new(token.text_range(), result)` as the standard construction pattern.
**Common Pitfalls:**
- Setting range to entire item instead of triggering token → over-highlighting
- Using different range in RangeInfo vs actual feature target → confusing UX
- Not handling macro expansions → range might be in generated code, not user code
- Returning range from expanded macro instead of call site → user sees wrong highlight
**Related Patterns in Ecosystem:**
- `std::ops::Range` with associated data (similar concept)
- LSP's `LocationLink` (range + target location)
- Compiler error spans (annotating results with source locations)
- Database query results with rowid metadata

---

## Pattern 5: Token Classification with pick_best_token
**File:** `crates/ide/src/goto_definition.rs` (lines 51-66)
**Category:** Syntax Token Selection
**Code Example:**
```rust
use ide_db::helpers::pick_best_token;

pub(crate) fn goto_definition(
    db: &RootDatabase,
    FilePosition { file_id, offset }: FilePosition,
    config: &GotoDefinitionConfig<'_>,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    let sema = &Semantics::new(db);
    let file = sema.parse_guess_edition(file_id).syntax().clone();
    let edition = sema.attach_first_edition(file_id).edition(db);
    let original_token = pick_best_token(file.token_at_offset(offset), |kind| match kind {
        IDENT
        | INT_NUMBER
        | LIFETIME_IDENT
        | T![self]
        | T![super]
        | T![crate]
        | T![Self]
        | COMMENT => 4,
        // index and prefix ops
        T!['['] | T![']'] | T![?] | T![*] | T![-] | T![!] => 3,
        kind if kind.is_keyword(edition) => 2,
        T!['('] | T![')'] => 2,
        kind if kind.is_trivia() => 0,
        _ => 1,
    })?;
    // ... rest of implementation
}
```
**Why This Matters for Contributors:** At any file offset, there can be 0-2 tokens (cursor between tokens). `pick_best_token` uses a ranking function to choose the most semantically relevant token. Higher scores = more relevant. Pattern: rank identifiers highest, then operators, then keywords, then punctuation, trivia gets 0 (filtered out). This ensures features work predictably when cursor is between tokens. Every feature that needs a token at a position should use this helper.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Algorithms - Heuristic Selection with Closure Customization
**Rust-Specific Insight:** This demonstrates Rust's strength in zero-cost abstraction through closures. The ranking function `|kind| match kind { ... }` is a closure that captures nothing, so it compiles to a function pointer with no allocation. The pattern `kind if kind.is_keyword(edition)` uses match guards for conditional ranking - idiomatic pattern matching. The `Option` return allows `pick_best_token` to reject all tokens (return None) when no token scores > 0, handling trivia-only cases gracefully.
**Contribution Tip:** When calling `pick_best_token`, tailor the ranking function to your feature's needs. For symbol rename, identifiers should rank highest. For syntax-aware features, operators might rank higher. Use edition-aware checks like `kind.is_keyword(edition)` to handle reserved words correctly across Rust editions. Test at token boundaries explicitly: `let x = fo$0o;` should pick "foo", `let x = $0foo;` should also pick "foo".
**Common Pitfalls:**
- Forgetting to rank trivia (whitespace/comments) as 0 → features trigger on whitespace
- Not handling edition in keyword detection → wrong behavior in edition 2015 vs 2021
- Assuming there's always a token at offset → must handle `None` return
- Not testing cursor between tokens → features fail in common editor scenarios
**Related Patterns in Ecosystem:**
- Fuzzy matching in editors (skim, fzf) - similar scoring functions
- `Iterator::max_by_key` (finding best element by score)
- Game AI decision trees (scoring actions to pick best)
- Compiler precedence parsing (operator ranking)

---

## Pattern 6: Semantics-Based Token Descent
**File:** `crates/ide/src/goto_definition.rs` (lines 90-96)
**Category:** Macro-Aware Analysis
**Code Example:**
```rust
pub(crate) fn goto_definition(
    db: &RootDatabase,
    FilePosition { file_id, offset }: FilePosition,
    config: &GotoDefinitionConfig<'_>,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    let sema = &Semantics::new(db);
    // ... find original_token ...

    let tokens = sema.descend_into_macros_no_opaque(original_token.clone(), false);
    let mut navs = Vec::new();
    for token in tokens {
        // ... handle each descended token ...
        let parent = token.value.parent()?;

        let Some(ident_class) = IdentClass::classify_node(sema, &parent) else { continue };
        navs.extend(ident_class.definitions().into_iter().flat_map(|(def, _)| {
            // ... convert definitions to navigation targets ...
        }));
    }
    let navs = navs.into_iter().unique().collect();

    Some(RangeInfo::new(original_token.text_range(), navs))
}
```
**Why This Matters for Contributors:** rust-analyzer must work inside macro expansions. Given a token in user-written code, `descend_into_macros` finds all corresponding tokens in macro expansions. The pattern is: (1) Get surface token from user code, (2) Descend into expansions getting all mapped tokens, (3) Analyze each descended token using semantic helpers like IdentClass::classify_node, (4) Collect results and deduplicate. This ensures features work identically inside and outside macros.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Algorithms - Tree Traversal with Hygiene Awareness
**Rust-Specific Insight:** Macro descent is the most complex part of rust-analyzer's design. Macros create a parallel AST (expanded tree) that must be mapped back to source tokens. The `descend_into_macros_no_opaque` method uses Salsa's incrementally computed expansion mappings. The `no_opaque` variant skips builtin macros like `format_args!` which don't have user-editable expansions. The `Vec<Token>` return handles ambiguous cases where one source token maps to multiple expansion tokens (macro repetitions). Deduplication with `.unique()` is critical because different expansion paths may yield the same semantic definition.
**Contribution Tip:** Always use `descend_into_macros` when implementing IDE features - it's the difference between working only on raw code vs working everywhere. Use the `no_opaque` variant unless you specifically need to analyze builtin macro expansions. Iterate over ALL descended tokens, not just the first - macros can generate multiple occurrences. Use `.unique()` or `FxHashSet` to deduplicate final results, as multiple expansion paths can lead to the same definition.
**Common Pitfalls:**
- Only processing first descended token → features break in macro repetitions
- Not deduplicating results → duplicate entries in goto-definition, references
- Using `descend_into_macros` instead of `no_opaque` → performance hit from builtin macros
- Forgetting to use original_token for RangeInfo → highlights wrong location
- Not handling macro hygiene → features see internal macro identifiers
**Related Patterns in Ecosystem:**
- Syn's macro parsing (lower-level AST manipulation)
- Procedural macro hygiene rules (similar mapping concepts)
- Source maps in compilers (mapping generated code to source)
- Debugger line tables (similar location mapping)

---

## Pattern 7: IdentClass Pattern for Name Resolution
**File:** `crates/ide/src/goto_definition.rs` (lines 134-145)
**Category:** Unified Name Classification
**Code Example:**
```rust
// From goto_definition:
let Some(ident_class) = IdentClass::classify_node(sema, &parent) else { continue };
navs.extend(ident_class.definitions().into_iter().flat_map(|(def, _)| {
    if let Definition::ExternCrateDecl(crate_def) = def {
        return crate_def
            .resolved_crate(db)
            .map(|it| it.root_module(db).to_nav(db))
            .into_iter()
            .flatten()
            .collect();
    }
    try_filter_trait_item_definition(sema, &def).unwrap_or_else(|| def_to_nav(sema, def))
}));

// From references:
match name_like {
    ast::NameLike::NameRef(name_ref) => {
        match NameRefClass::classify(sema, &name_ref)? {
            NameRefClass::Definition(def, _) => def,
            NameRefClass::FieldShorthand { local_ref, field_ref, .. } => {
                // handle shorthand
            }
        }
    }
    ast::NameLike::Name(name) => {
        match NameClass::classify(sema, &name)? {
            NameClass::Definition(it) | NameClass::ConstReference(it) => it,
            NameClass::PatFieldShorthand { local_def: _, field_ref, .. } => {
                Definition::Field(field_ref)
            }
        }
    }
    ast::NameLike::Lifetime(lifetime) => {
        // handle lifetimes
    }
}
```
**Why This Matters for Contributors:** `IdentClass`, `NameClass`, and `NameRefClass` (from ide_db::defs) are the core abstractions for resolving names to definitions. Use IdentClass::classify_node for nodes that could be either name or name-ref. Use NameClass for declarations (fn foo), use NameRefClass for references (calling foo). These return Definition enum which unifies all name kinds (functions, types, locals, etc). Pattern: classify node → get Definition → use Definition APIs for further analysis. This abstraction is critical for implementing any feature that needs name resolution.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Type System - Sum Type (Enum) for Polymorphism
**Rust-Specific Insight:** The `Definition` enum is a perfect example of Rust's algebraic data types replacing OOP inheritance. Instead of a trait `Definable` with `impl` for each type, we have one enum with variants for each definition kind. This enables exhaustive pattern matching - the compiler enforces that you handle all cases. The classify methods use Option for fallible classification, composing with `?` for early returns. FieldShorthand handling shows Rust-specific syntax awareness (struct initialization shorthand `Foo { x }` where `x` means `x: x`).
**Contribution Tip:** When you need to resolve a name, match on the AST node type (NameRef vs Name vs Lifetime) and use the corresponding classifier. The returned `Definition` can be matched exhaustively to handle each kind specifically, or used polymorphically through trait methods (Definition::name(), Definition::module(), etc). For field shorthand, both local_ref and field_ref are valid targets - decide based on context which one your feature needs.
**Common Pitfalls:**
- Not handling FieldShorthand cases → features fail on `Struct { x }` syntax
- Using wrong classifier for node type → classification fails, returns None
- Forgetting `Some(def)` filter → processing None and panicking
- Not handling all Definition variants in exhaustive match → compilation error (good!)
- Assuming Definition::name() is infallible → some definitions have no name (anonymous lifetimes)
**Related Patterns in Ecosystem:**
- Compiler HIR (similar definition abstraction)
- LSP SymbolKind (protocol-level equivalent)
- Syn's Item enum (parsing-level abstraction)
- Database ORM type unions (similar polymorphic data)

---

## Pattern 8: NavigationTarget Builder Pattern
**File:** `crates/ide/src/navigation_target.rs`
**Category:** Structured Navigation Result
**Code Example:**
```rust
/// `NavigationTarget` represents an element in the editor's UI which you can
/// click on to navigate to a particular piece of code.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NavigationTarget {
    pub file_id: FileId,
    /// Range which encompasses the whole element.
    /// Should include body, doc comments, attributes, etc.
    pub full_range: TextRange,
    /// A "most interesting" range within the `full_range`.
    /// Typically, `full_range` is the whole syntax node, including doc
    /// comments, and `focus_range` is the range of the identifier.
    pub focus_range: Option<TextRange>,
    pub name: Symbol,
    pub kind: Option<SymbolKind>,
    pub container_name: Option<Symbol>,
    pub description: Option<String>,
    pub docs: Option<Documentation<'static>>,
    pub alias: Option<Symbol>,
}

impl NavigationTarget {
    pub fn focus_or_full_range(&self) -> TextRange {
        self.focus_range.unwrap_or(self.full_range)
    }

    pub(crate) fn from_named(
        db: &RootDatabase,
        InFile { file_id, value }: InFile<&dyn ast::HasName>,
        kind: SymbolKind,
    ) -> UpmappingResult<NavigationTarget> {
        let name = value.name().map(|it| Symbol::intern(&it.text()))
            .unwrap_or_else(|| sym::underscore);

        orig_range_with_focus(db, file_id, value.syntax(), value.name()).map(
            |(FileRange { file_id, range: full_range }, focus_range)| {
                NavigationTarget::from_syntax(file_id, name.clone(), focus_range, full_range, kind)
            },
        )
    }
}
```
**Why This Matters for Contributors:** NavigationTarget is the standard way to represent "things you can navigate to" in rust-analyzer. Key insight: separate full_range (entire item including docs/attrs) from focus_range (just the name). When user clicks, cursor goes to focus_range; when highlighting, highlight full_range. Container_name provides breadcrumbs (Foo::bar). Pattern for building: use from_named for AST nodes with names, or from_syntax for custom construction. The UpmappingResult wrapper handles macro call sites vs definition sites.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Data Structures - Rich Domain Model
**Rust-Specific Insight:** NavigationTarget uses `Symbol` (interned strings) instead of `String` for names - critical optimization in rust-analyzer where the same names ("new", "clone", "self") appear millions of times. Interning gives `O(1)` equality checks and minimal memory footprint. The `Option<TextRange>` for focus_range is idiomatic: None means "no specific focus point, use full_range". The `InFile<&dyn ast::HasName>` parameter demonstrates trait objects for polymorphism - any AST node with a name can use this helper. Hash/Eq/PartialEq derives enable using NavigationTarget as HashMap keys for deduplication.
**Contribution Tip:** Use `from_named` for most cases - it handles the common pattern of extracting name and computing ranges. Set focus_range to the identifier token, full_range to the entire item including attributes and documentation. For breadcrumbs, compute container_name by walking up the module tree (impl blocks, modules, traits). Use `Symbol::intern` for names, not `String::from`, to benefit from string interning.
**Common Pitfalls:**
- Using String instead of Symbol → memory waste and slow comparisons
- Setting focus_range == full_range → cursor jumps to item start, not name
- Not including docs/attrs in full_range → incomplete navigation context
- Forgetting to set container_name → missing breadcrumbs in UI
- Not handling UpmappingResult → macro navigation goes to wrong location
**Related Patterns in Ecosystem:**
- LSP Location and LocationLink (protocol equivalents)
- Compiler Span with expansion info (similar rich location data)
- Database RowId with denormalized metadata (similar enrichment)
- IDE symbol trees (similar hierarchical representation)

---

## Pattern 9: ToNav/TryToNav Trait Pattern
**File:** `crates/ide/src/navigation_target.rs` (lines 117-138, 431-471)
**Category:** Polymorphic Navigation
**Code Example:**
```rust
pub(crate) trait ToNav {
    fn to_nav(&self, db: &RootDatabase) -> UpmappingResult<NavigationTarget>;
}

pub trait TryToNav {
    fn try_to_nav(
        &self,
        sema: &Semantics<'_, RootDatabase>,
    ) -> Option<UpmappingResult<NavigationTarget>>;
}

impl<T: TryToNav, U: TryToNav> TryToNav for Either<T, U> {
    fn try_to_nav(&self, sema: &Semantics<'_, RootDatabase>)
        -> Option<UpmappingResult<NavigationTarget>> {
        match self {
            Either::Left(it) => it.try_to_nav(sema),
            Either::Right(it) => it.try_to_nav(sema),
        }
    }
}

// Generic implementation for common HIR types:
impl<D> TryToNav for D
where
    D: HasSource
        + ToNavFromAst
        + Copy
        + HasDocs
        + for<'db> HirDisplay<'db>
        + HasCrate
        + hir::HasName,
    D::Ast: ast::HasName,
{
    fn try_to_nav(&self, sema: &Semantics<'_, RootDatabase>)
        -> Option<UpmappingResult<NavigationTarget>> {
        let db = sema.db;
        let src = self.source_with_range(db)?;
        Some(
            NavigationTarget::from_named_with_range(/*...*/)
            .map(|mut res| {
                res.docs = self.docs(db).map(Documentation::into_owned);
                res.description = Some(self.display(db, /*...*/).to_string());
                res.container_name = self.container_name(db);
                res
            }),
        )
    }
}
```
**Why This Matters for Contributors:** ToNav/TryToNav provide polymorphic navigation target creation. ToNav is infallible (modules always have nav), TryToNav can fail (macros might not have source). The generic blanket implementation shows how to add navigation for new HIR types: implement ToNavFromAst marker trait with KIND constant, get HasSource+HasDocs+HirDisplay automatically implemented. This eliminates boilerplate when adding new navigable items. Pattern: use TryToNav in feature code, implement ToNavFromAst for new HIR types.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Trait System - Blanket Implementation + Marker Traits
**Rust-Specific Insight:** This demonstrates advanced trait-driven design. The blanket impl `impl<D> TryToNav for D where D: HasSource + ToNavFromAst + ...` automatically implements TryToNav for ANY type that satisfies the bounds. ToNavFromAst is a marker trait carrying only the KIND constant - a pattern for injecting type-specific constants into generic code. The `for<'db> HirDisplay<'db>` syntax is a higher-ranked trait bound (HRTB), allowing display implementation that works for any database lifetime. The `Either<T, U>` impl shows trait composition for sum types.
**Contribution Tip:** To add navigation for a new HIR type (say, `Constant`), implement: (1) `HasSource` (map to AST node), (2) `ToNavFromAst` with `const KIND = SymbolKind::Const`, (3) Get TryToNav automatically via blanket impl. Don't manually implement TryToNav unless you need custom logic. The trait bounds (HasDocs, HirDisplay, HasName) ensure your type has all necessary capabilities.
**Common Pitfalls:**
- Forgetting to implement marker trait ToNavFromAst → blanket impl doesn't apply
- Wrong SymbolKind constant → incorrect icons in editor
- Not implementing HasSource → blanket impl can't apply (trait bound unsatisfied)
- Manual TryToNav impl when blanket would work → code duplication
- Missing Copy bound on HIR type → can't use in trait impl (HIR types are cheap Copy)
**Related Patterns in Ecosystem:**
- Serde's Serialize/Deserialize blanket impls
- Std's From/Into automatic reciprocal implementation
- Iterator trait with associated type IntoIter
- Async trait with Send bounds for runtime compatibility

---

## Pattern 10: Test Fixture Infrastructure
**File:** `crates/ide/src/fixture.rs`
**Category:** Test Scaffolding Pattern
**Code Example:**
```rust
/// Creates analysis for a single file.
pub(crate) fn file(#[rust_analyzer::rust_fixture] ra_fixture: &str) -> (Analysis, FileId) {
    let mut host = AnalysisHost::default();
    let change_fixture = ChangeFixture::parse(ra_fixture);
    host.db.enable_proc_attr_macros();
    host.db.apply_change(change_fixture.change);
    (host.analysis(), change_fixture.files[0].file_id())
}

/// Creates analysis from a multi-file fixture, returns positions marked with $0.
pub(crate) fn position(
    #[rust_analyzer::rust_fixture] ra_fixture: &str,
) -> (Analysis, FilePosition) {
    let mut host = AnalysisHost::default();
    let change_fixture = ChangeFixture::parse(ra_fixture);
    host.db.enable_proc_attr_macros();
    host.db.apply_change(change_fixture.change);
    let (file_id, range_or_offset) = change_fixture.file_position.expect("expected a marker ($0)");
    let offset = range_or_offset.expect_offset();
    (host.analysis(), FilePosition { file_id: file_id.file_id(), offset })
}

// Usage in tests:
#[test]
fn test_goto_definition() {
    let (analysis, position) = fixture::position(
        r#"
struct Foo;
fn bar() {
    let x = F$0oo;
}
        "#,
    );

    let result = analysis.goto_definition(position, &config).unwrap();
    // assertions...
}
```
**Why This Matters for Contributors:** The fixture module provides test helpers that parse annotated test code. `$0` marks cursor position, `$0....$0` marks ranges, `^...^` marks annotations. Pattern: use fixture::position for point-based tests, fixture::range for range tests, fixture::annotations for multi-point tests. The fixtures support multi-file projects with `//- /path/to/file.rs` syntax. Every new feature should have tests using these fixtures - they make tests concise and readable while handling all Analysis setup boilerplate.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Testing - Domain-Specific Language for Test Data
**Rust-Specific Insight:** The `#[rust_analyzer::rust_fixture]` attribute is a custom proc-macro that transforms string literals into parsed test data at compile time. This is a powerful Rust pattern: using proc-macros to create embedded DSLs. The fixture parser handles both inline annotations (`$0`) and multi-file syntax (`//- /lib.rs`), creating a complete test workspace. The `.expect("expected a marker ($0)")` provides clear error messages when tests are malformed - idiomatic use of Option/Result for validation. ChangeFixture internally uses Salsa's change application, ensuring tests use the same code path as production.
**Contribution Tip:** Write tests using fixtures for clarity. Structure: (1) Create fixture string with code and markers, (2) Call appropriate fixture function (position/range/annotations), (3) Call feature under test, (4) Assert on result. Use multi-file fixtures when testing cross-file features: `//- /lib.rs`, `//- /foo.rs`, etc. Use `$0` for cursor, `$0...$0` for ranges, `^` comments for multiple points. Always test macro cases and edge cases (empty files, EOF).
**Common Pitfalls:**
- Forgetting to mark cursor with $0 → expect() panics with clear message
- Using wrong fixture function (range vs position) → type mismatch at compile time
- Not testing multi-file scenarios → features break on cross-crate references
- Hardcoding FileIds instead of using fixture → brittle tests
- Not testing macro expansions → features fail in real code with macros
**Related Patterns in Ecosystem:**
- Datatest-stable (directory-based test discovery)
- Insta snapshot testing (similar test data management)
- Proptest strategies (generative test data)
- Syn's quote! macro (similar embedded DSL for code generation)

---

## Pattern 11: Markup Generation for Rich Text
**File:** `crates/ide/src/markup.rs`
**Category:** Rich Text Output
**Code Example:**
```rust
/// Markdown formatting.
/// Sometimes, we want to display a "rich text" in the UI. At the moment, we use
/// markdown for this purpose.
#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct Markup {
    text: String,
}

impl From<Markup> for String {
    fn from(markup: Markup) -> Self {
        markup.text
    }
}

impl From<String> for Markup {
    fn from(text: String) -> Self {
        Markup { text }
    }
}

impl Markup {
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn fenced_block(contents: impl fmt::Display) -> Markup {
        format!("```rust\n{contents}\n```").into()
    }

    pub fn fenced_block_text(contents: impl fmt::Display) -> Markup {
        format!("```text\n{contents}\n```").into()
    }
}

// Usage in hover:
let markup = Markup::fenced_block(type_display);
HoverResult { markup, actions: vec![] }
```
**Why This Matters for Contributors:** Markup is a newtype wrapper around String that semantically represents markdown content destined for editor display. It provides builders for common markdown patterns (fenced code blocks). Pattern: use Markup for any user-facing rich text (hover tooltips, signature help). Use fenced_block for Rust code, fenced_block_text for plain text. This type safety prevents accidentally mixing markdown with plain strings. When implementing features with formatted output, always return Markup, not raw String.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★☆ (4/5)
**Pattern Classification:** Type System - Newtype for Semantic Distinction
**Rust-Specific Insight:** Markup is a simple but effective newtype pattern. Wrapping String prevents confusion between raw text and markdown-formatted text at compile time. The bidirectional From impls (String → Markup, Markup → String) provide ergonomic conversions. The builders (`fenced_block`) use `impl fmt::Display` parameters - a Rust idiom that accepts String, &str, and any Display type without explicit conversion. The pattern prevents security issues (markdown injection) by forcing explicit conversion to Markup.
**Contribution Tip:** Use Markup::fenced_block for all code snippets in hover/completion. The language tag ("rust"/"text") enables syntax highlighting in editors. Build complex markdown with format! or format_to! macro, then convert to Markup. For plain text that might contain markdown special chars, use fenced_block_text or escape manually. Consider adding builders for other markdown patterns (lists, headers) if needed frequently.
**Common Pitfalls:**
- Returning raw String instead of Markup → type error (good!) forces correction
- Not escaping user input in markdown → markdown injection, rendering issues
- Using wrong language tag → incorrect syntax highlighting in editor
- Forgetting to newline-terminate fenced blocks → malformed markdown
- Mixing Markup and String in collections → type mismatch prevents this
**Related Patterns in Ecosystem:**
- Html/Css newtype wrappers in web frameworks (similar safety)
- Serde's untagged enum for flexible input (similar conversion patterns)
- Diesel's SqlIdentifier (similar domain-specific string type)
- Comrak/pulldown-cmark (markdown parsing/rendering crates)

---

## Pattern 12: Hover Result with Actions
**File:** `crates/ide/src/hover.rs` (lines 79-123)
**Category:** Actionable Information Display
**Code Example:**
```rust
#[derive(Debug, Clone, Hash, PartialEq, Eq, UpmapFromRaFixture)]
pub enum HoverAction {
    Runnable(Runnable),
    Implementation(FilePosition),
    Reference(FilePosition),
    GoToType(Vec<HoverGotoTypeData>),
}

/// Contains the results when hovering over an item
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, UpmapFromRaFixture)]
pub struct HoverResult {
    pub markup: Markup,
    pub actions: Vec<HoverAction>,
}

// Building HoverResult in hover_for_definition:
HoverResult {
    markup: render::process_markup(sema.db, def, &markup, range_map, config),
    actions: [
        show_fn_references_action(sema, def),
        show_implementations_action(sema, def),
        runnable_action(sema, def, file_id),
        goto_type_action_for_def(sema, def, &notable_traits, subst_types, edition),
    ]
    .into_iter()
    .flatten()
    .collect(),
}

fn show_implementations_action(
    sema: &Semantics<'_, RootDatabase>,
    def: Definition,
) -> Option<HoverAction> {
    fn to_action(nav_target: NavigationTarget) -> HoverAction {
        HoverAction::Implementation(FilePosition {
            file_id: nav_target.file_id,
            offset: nav_target.focus_or_full_range().start(),
        })
    }

    let adt = match def {
        Definition::Trait(it) => {
            return it.try_to_nav(sema).map(UpmappingResult::call_site).map(to_action);
        }
        Definition::Adt(it) => Some(it),
        Definition::SelfType(it) => it.self_ty(sema.db).as_adt(),
        _ => None,
    }?;
    adt.try_to_nav(sema).map(UpmappingResult::call_site).map(to_action)
}
```
**Why This Matters for Contributors:** HoverResult combines information display (Markup) with actionable items (HoverAction). Each action becomes a clickable link in the editor. Pattern: (1) Render the primary information into Markup, (2) Generate context-appropriate actions (go to implementation, find references, run test), (3) Combine into HoverResult. Action generation functions return Option<HoverAction> and get flattened - only applicable actions appear. This pattern makes hover tooltips interactive. When adding hover support for new items, always consider what actions make sense.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Data Structures - Command Pattern with Enum Dispatch
**Rust-Specific Insight:** HoverAction enum represents the Command pattern in Rust - each variant is a different action type with associated data. The pattern `[fn1(), fn2(), fn3()].into_iter().flatten().collect()` is idiomatic for filtering and collecting Option results - None values disappear via flatten(). The enum approach is superior to trait objects here because actions are a closed set (editors need to handle all cases). The UpmapFromRaFixture derive shows extensibility through proc-macros. Hash/PartialEq/Eq derives enable deduplication of actions.
**Contribution Tip:** When adding actions, create Option-returning helper functions for each action type. Use pattern matching on Definition to determine which actions apply. Always flatten the array of Options to filter out inapplicable actions. For new action types, add an enum variant and handle it in all client code (LSP server, UI layers). Test that actions appear/disappear correctly based on cursor position and definition type.
**Common Pitfalls:**
- Returning Some(action) for inapplicable cases → wrong actions shown to user
- Not deduplicating actions → duplicate "Go to implementation" links
- Missing Hash/Eq derives → can't deduplicate in HashSet
- Not handling new enum variants in LSP conversion → compilation error (good!)
- Creating actions with wrong FilePosition → action navigates to wrong location
**Related Patterns in Ecosystem:**
- LSP CodeAction (protocol-level equivalent)
- UI event handlers (similar command dispatch)
- Redux actions in web frameworks (similar enum-based commands)
- Game engine input handling (similar action abstraction)

---

## Pattern 13: Config Structs for Feature Customization
**File:** `crates/ide/src/hover.rs` (lines 35-48)
**Category:** Feature Configuration
**Code Example:**
```rust
#[derive(Clone, Debug)]
pub struct HoverConfig<'a> {
    pub links_in_hover: bool,
    pub memory_layout: Option<MemoryLayoutHoverConfig>,
    pub documentation: bool,
    pub keywords: bool,
    pub format: HoverDocFormat,
    pub max_trait_assoc_items_count: Option<usize>,
    pub max_fields_count: Option<usize>,
    pub max_enum_variants_count: Option<usize>,
    pub max_subst_ty_len: SubstTyLen,
    pub show_drop_glue: bool,
    pub minicore: MiniCore<'a>,
}

pub(crate) fn hover(
    db: &RootDatabase,
    frange @ FileRange { file_id, range }: FileRange,
    config: &HoverConfig<'_>,
) -> Option<RangeInfo<HoverResult>> {
    // Use config throughout implementation
    if let HoverDocFormat::PlainText = config.format {
        res.info.markup = remove_markdown(res.info.markup.as_str()).into();
    }
    // ...
}
```
**Why This Matters for Contributors:** Complex IDE features take a Config struct instead of many boolean parameters. This pattern: (1) Groups related options, (2) Allows adding options without breaking API, (3) Makes call sites cleaner, (4) Enables partial configuration with defaults. Pattern: create FooConfig struct, take `&FooConfig` parameter, use config fields to customize behavior. Config structs should derive Clone and have descriptive field names. This is essential for features with user-configurable behavior (hover, inlay hints, completions, diagnostics).

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** API Design - Configuration Object Pattern
**Rust-Specific Insight:** This pattern uses struct borrowing (`&HoverConfig`) instead of owned values for efficiency - config is read-only during feature execution. The lifetime parameter `<'a>` on HoverConfig allows it to borrow data (MiniCore) from the caller without cloning. Using Option<T> for optional settings (max_trait_assoc_items_count) is idiomatic - None means "no limit", Some(n) means "limit to n". Clone derive enables cheap copying when needed (configs are small, mostly Copy types). This pattern prevents "boolean blindness" - compare `hover(db, range, true, false, 10, 5)` vs `hover(db, range, &config)`.
**Contribution Tip:** When creating config structs, use: (1) bool for binary choices, (2) Option<T> for optional settings with None = "off", (3) enums for multi-choice (HoverDocFormat), (4) numeric types for limits/thresholds. Take config by reference (&Config) to avoid copying. Add #[derive(Clone, Debug)] for testability. Document each field's meaning and default value. Group related settings (memory_layout_config subgroup).
**Common Pitfalls:**
- Taking config by value → unnecessary cloning on every call
- Using bool for multi-choice settings → add enum variant later → breaking change
- Not documenting defaults → users confused about None vs Some behavior
- Adding required fields to existing config → breaks all callers (use Option<T>)
- No Default impl → callers must set all fields (painful for many options)
**Related Patterns in Ecosystem:**
- Builder pattern (alternative for complex configuration)
- Clap's Args struct (CLI argument parsing with similar pattern)
- Serde's Deserializer config (similar extensible options)
- Tower middleware configuration (similar layered config)

---

## Pattern 14: Signature Help with Active Parameter
**File:** `crates/ide/src/signature_help.rs` (lines 28-70)
**Category:** Contextual Parameter Information
**Code Example:**
```rust
/// Contains information about an item signature as seen from a use site.
/// This includes the "active parameter", which is the parameter whose value
/// is currently being edited.
#[derive(Debug)]
pub struct SignatureHelp {
    pub doc: Option<Documentation<'static>>,
    pub signature: String,
    pub active_parameter: Option<usize>,
    parameters: Vec<TextRange>,
}

impl SignatureHelp {
    pub fn parameter_labels(&self) -> impl Iterator<Item = &str> + '_ {
        self.parameters.iter().map(move |&it| &self.signature[it])
    }

    pub fn parameter_ranges(&self) -> &[TextRange] {
        &self.parameters
    }

    fn push_call_param(&mut self, param: &str) {
        self.push_param("(", param);
    }

    fn push_param(&mut self, opening_delim: &str, param: &str) {
        if !self.signature.ends_with(opening_delim) {
            self.signature.push_str(", ");
        }
        let start = TextSize::of(&self.signature);
        self.signature.push_str(param);
        let end = TextSize::of(&self.signature);
        self.parameters.push(TextRange::new(start, end))
    }
}

// Usage in signature_help_for_call:
let mut res = SignatureHelp {
    doc: None,
    signature: String::new(),
    parameters: vec![],
    active_parameter
};
format_to!(res.signature, "fn {}", func.name(db).display(db, edition));
// ... build signature string while tracking parameter ranges ...
for param in fn_params {
    res.push_call_param(&param.display(db, edition).to_string());
}
```
**Why This Matters for Contributors:** SignatureHelp demonstrates a builder pattern for incremental string construction while tracking structured metadata. The key insight: build signature as a String, but record TextRange for each parameter within that string. This allows editors to highlight the active parameter. Pattern: (1) Create SignatureHelp with empty signature, (2) Build signature incrementally with push_param methods that track ranges, (3) Calculate active parameter index from cursor position. This pattern applies to any feature that generates formatted text with internal structure.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Data Structures - Coordinated State Builder
**Rust-Specific Insight:** This pattern maintains an invariant: parameters Vec contains ranges that partition the signature String. The `TextSize::of(&self.signature)` pattern uses rowan's TextSize abstraction (newtype over u32) instead of raw usize - provides bounds checking and prevents mixing byte/char indices. The push_param method uses checked string mutation - it modifies signature and synchronously updates parameters to maintain the invariant. The Iterator return type `impl Iterator<Item = &str>` is zero-cost: no allocation, just yields slices over signature String.
**Contribution Tip:** Use this pattern when you need structured text output. Build the string and metadata together in the same method calls - this maintains invariants. Use TextSize for all text positions (consistent with rowan). Calculate ranges as you build, don't try to parse afterwards. For display, use parameter_labels() iterator to avoid cloning. Test with edge cases: no parameters, many parameters, parameters with special chars.
**Common Pitfalls:**
- Building string first, then computing ranges → off-by-one errors, complexity
- Using usize instead of TextSize → type mismatches with rowan APIs
- Not handling comma separators correctly → wrong parameter ranges
- Pushing parameters in wrong order → active parameter highlights wrong param
- Forgetting to update parameters Vec → ranges become stale, wrong highlighting
**Related Patterns in Ecosystem:**
- Rope data structures (similar coordinated text + metadata)
- Syntax highlighting with ranges (similar range tracking)
- LSP ParameterInformation (protocol equivalent)
- Regex capture groups (similar substring indexing)

---

## Pattern 15: Call Hierarchy with Bidirectional Search
**File:** `crates/ide/src/call_hierarchy.rs`
**Category:** Graph Traversal Pattern
**Code Example:**
```rust
#[derive(Debug, Clone)]
pub struct CallItem {
    pub target: NavigationTarget,
    pub ranges: Vec<FileRange>,
}

pub(crate) fn incoming_calls(
    db: &RootDatabase,
    config: &CallHierarchyConfig<'_>,
    FilePosition { file_id, offset }: FilePosition,
) -> Option<Vec<CallItem>> {
    let sema = &Semantics::new(db);
    let file = sema.parse_guess_edition(file_id);
    let file = file.syntax();
    let mut calls = CallLocations::default();

    let references = sema
        .find_nodes_at_offset_with_descend(file, offset)
        .filter_map(move |node| match node {
            ast::NameLike::NameRef(name_ref) => match NameRefClass::classify(sema, &name_ref)? {
                NameRefClass::Definition(def @ Definition::Function(_), _) => Some(def),
                _ => None,
            },
            ast::NameLike::Name(name) => match NameClass::classify(sema, &name)? {
                NameClass::Definition(def @ Definition::Function(_)) => Some(def),
                _ => None,
            },
            ast::NameLike::Lifetime(_) => None,
        })
        .flat_map(|func| func.usages(sema).all());

    for (_, references) in references {
        let references = references.iter()
            .filter_map(|FileReference { name, .. }| name.as_name_ref());
        for name in references {
            // This target is the containing function
            let def_nav = sema.ancestors_with_macros(name.syntax().clone()).find_map(|node| {
                let def = ast::Fn::cast(node).and_then(|fn_| sema.to_def(&fn_))?;
                def.try_to_nav(sema).map(|nav| (def, nav))
            });

            if let Some((def, nav)) = def_nav {
                if config.exclude_tests && def.is_test(db) {
                    continue;
                }
                let range = sema.original_range(name.syntax());
                calls.add(nav.call_site, range.into_file_id(db));
            }
        }
    }

    Some(calls.into_items())
}
```
**Why This Matters for Contributors:** Call hierarchy requires bidirectional analysis: incoming calls (who calls me?) and outgoing calls (what do I call?). The pattern: (1) Find target function at cursor, (2) For incoming: use usages() to find all references, walk up syntax tree to find containing function, (3) For outgoing: walk down syntax tree finding CallExpr/MethodCallExpr, resolve to target. CallItem groups multiple call sites to the same target. This demonstrates how to combine name resolution, reference search, and syntax tree walking for complex queries.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Algorithms - Bidirectional Graph Traversal
**Rust-Specific Insight:** This pattern combines multiple iterator chains for complex filtering. The `filter_map` + `flat_map` chain is idiomatic for transforming and flattening in one pass. The `sema.ancestors_with_macros()` method walks up the syntax tree efficiently using iterator adaptors - no allocations until find_map consumes. The CallLocations type (not shown) uses HashMap to deduplicate - multiple calls to same function from one caller are grouped. The pattern `.filter(|_| !config.exclude_tests)` shows compile-time optimization of conditionals via config.
**Contribution Tip:** For incoming calls: (1) Use Definition::usages() for reference search, (2) Filter to NameRef nodes only (not declarations), (3) Walk up syntax tree with ancestors_with_macros, (4) Find first enclosing Fn, (5) Group by containing function. For outgoing calls: walk child nodes finding CallExpr, resolve each to target definition. Always use sema methods (ancestors_with_macros, original_range) for macro awareness. Deduplicate call sites pointing to same target.
**Common Pitfalls:**
- Using ancestors() instead of ancestors_with_macros() → breaks in macro expansions
- Not deduplicating calls → duplicate entries when function called multiple times
- Including the function itself in incoming calls → should only show callers
- Not handling method calls separately → missing method call hierarchy
- Forgetting config.exclude_tests → showing test functions when user disabled them
**Related Patterns in Ecosystem:**
- Tree-sitter query cursors (similar tree walking)
- Compiler's def-use chains (similar call graph analysis)
- LSP CallHierarchy (protocol equivalent)
- Static analysis tools (similar reference tracking)

---

## Pattern 16: Reference Search with Category Metadata
**File:** `crates/ide/src/references.rs` (lines 45-68)
**Category:** Categorized Reference Results
**Code Example:**
```rust
/// Result of a reference search operation.
#[derive(Debug, Clone, UpmapFromRaFixture)]
pub struct ReferenceSearchResult {
    /// Information about the declaration site of the searched item.
    pub declaration: Option<Declaration>,
    /// All references found, grouped by file.
    /// The map key is the file ID, and the value is a vector of (range, category) pairs.
    pub references: IntMap<FileId, Vec<(TextRange, ReferenceCategory)>>,
}

#[derive(Debug, Clone, UpmapFromRaFixture)]
pub struct Declaration {
    pub nav: NavigationTarget,
    pub is_mut: bool,
}

// Building the result:
pub(crate) fn find_all_refs(
    sema: &Semantics<'_, RootDatabase>,
    position: FilePosition,
    config: &FindAllRefsConfig<'_>,
) -> Option<Vec<ReferenceSearchResult>> {
    let make_searcher = |literal_search: bool| {
        move |def: Definition| {
            let mut usages = def.usages(sema)
                .set_scope(config.search_scope.as_ref())
                .include_self_refs()
                .all();

            let mut references: IntMap<FileId, Vec<(TextRange, ReferenceCategory)>> = usages
                .into_iter()
                .map(|(file_id, refs)| {
                    (
                        file_id.file_id(sema.db),
                        refs.into_iter()
                            .map(|file_ref| (file_ref.range, file_ref.category))
                            .unique()
                            .collect(),
                    )
                })
                .collect();

            ReferenceSearchResult { declaration, references }
        }
    };
    // ...
}
```
**Why This Matters for Contributors:** Reference search returns categorized results: reads vs writes, imports, etc. References are grouped by file for efficient editor display. Declaration is separated from references, providing context about what's being referenced. The pattern: (1) Use Definition::usages() for search, (2) Map results to (TextRange, ReferenceCategory) pairs, (3) Group by file using IntMap for performance, (4) Include declaration info. ReferenceCategory allows editors to show different icons for reads/writes. This structure is essential for "find all references" feature.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Data Structures - Hierarchical Grouping with Metadata
**Rust-Specific Insight:** IntMap is a specialized HashMap using FileId's internal integer as key directly - faster than hashing because FileId is a newtype over u32. The pattern `map().collect()` for grouping is idiomatic - transforming iterator of (FileId, Vec<Ref>) into IntMap. The `.unique()` call uses itertools to deduplicate - important because macro expansions can create duplicate references. ReferenceCategory is an enum (Read/Write/Import/etc) providing O(1) categorization checks. Separating declaration from references allows showing "1 definition, 42 references" in UI.
**Contribution Tip:** Use Definition::usages() as starting point, configure with set_scope() for workspace vs file search. Always call include_self_refs() to include the declaration. Map each FileReference to (range, category), then group by file_id. Use IntMap instead of HashMap for FileId keys (performance). Deduplicate with .unique() after mapping. Calculate declaration info separately (nav + is_mut flag for locals).
**Common Pitfalls:**
- Not deduplicating references → duplicate entries from macro expansions
- Using HashMap instead of IntMap → slower performance for FileId keys
- Forgetting include_self_refs() → declaration not included in results
- Not categorizing references → can't distinguish reads from writes
- Not grouping by file → inefficient editor display, slow UI updates
**Related Patterns in Ecosystem:**
- LSP DocumentSymbol hierarchy (similar grouping)
- Compiler's def-use chains (similar reference tracking)
- Database query grouping (GROUP BY analogy)
- Ripgrep's grouped output (similar file-based grouping)

---

## Pattern 17: Syntax Highlighting with Layered Tags
**File:** `crates/ide/src/syntax_highlighting.rs` (lines 39-67)
**Category:** Semantic Syntax Coloring
**Code Example:**
```rust
#[derive(Debug, Clone, Copy)]
pub struct HlRange {
    pub range: TextRange,
    pub highlight: Highlight,
    pub binding_hash: Option<u64>,
}

#[derive(Copy, Clone, Debug)]
pub struct HighlightConfig<'a> {
    pub strings: bool,
    pub comments: bool,
    pub punctuation: bool,
    pub specialize_punctuation: bool,
    pub operator: bool,
    pub specialize_operator: bool,
    pub inject_doc_comment: bool,
    pub macro_bang: bool,
    pub syntactic_name_ref_highlighting: bool,
    pub minicore: MiniCore<'a>,
}

pub(crate) fn highlight(
    db: &RootDatabase,
    config: &HighlightConfig<'_>,
    file_id: FileId,
    range_to_highlight: Option<TextRange>,
) -> Vec<HlRange> {
    // Returns a Vec of highlighted ranges with semantic tags
    // Each HlRange has: text range, Highlight (tag + modifiers), binding_hash
}

// From tags.rs:
pub struct Highlight {
    pub tag: HlTag,
    pub mods: HlMods,
}

pub enum HlTag {
    Symbol(SymbolKind),
    BuiltinType,
    Keyword,
    Operator(HlOperator),
    Punctuation(HlPunct),
    // ... more variants
}
```
**Why This Matters for Contributors:** Syntax highlighting uses a two-layer system: HlTag (what kind of thing) + HlMods (modifiers like mutable, reference). This allows fine-grained control: "mutable local variable" = Symbol(Local) + mutable modifier. Binding_hash allows semantic rainbow highlighting (same color for related uses). Pattern: walk syntax tree, classify each token semantically using hir, emit HlRange with appropriate tag+mods. Config allows disabling features (comments, strings) for performance. This architecture separates semantic analysis from color choice - clients map tags to colors.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Data Structures - Tagged Union with Bitflag Modifiers
**Rust-Specific Insight:** The Highlight struct combines an enum (HlTag) with bitflags (HlMods) - a powerful Rust pattern for extensible categorization. HlTag uses nested enums (Symbol(SymbolKind)) for hierarchical classification. HlMods uses bitflags crate (not shown) allowing combinations like `mutable | reference | unsafe`. The binding_hash Option<u64> enables semantic highlighting - hash of the binding definition, so all uses of same variable get same hash. This is an example of tagging data (ranges) with computed metadata for downstream use.
**Contribution Tip:** When adding semantic highlighting, classify each token into (tag, mods, binding_hash) triple. Use Symbol(kind) for user-defined items, BuiltinType for primitives, Keyword for reserved words. Set mods based on context: mutable for &mut, unsafe for unsafe blocks, etc. Calculate binding_hash only for local bindings (not for global items). Return Vec<HlRange> sorted by range start for efficient editor consumption. Test that highlighting respects config flags.
**Common Pitfalls:**
- Returning unsorted HlRanges → editor rendering bugs
- Wrong tag for item (using Keyword for type names) → incorrect colors
- Not setting modifiers → missing visual distinction (mut vs immut)
- Same binding_hash for unrelated items → wrong rainbow grouping
- Highlighting syntax inside strings/comments when disabled → performance hit
**Related Patterns in Ecosystem:**
- LSP SemanticTokens (protocol equivalent)
- Tree-sitter highlighting queries (similar tree-based highlighting)
- Compiler HIR representation (similar semantic categorization)
- Syntax highlighting libraries (similar tag + modifier approach)

---

## Pattern 18: Inlay Hints with Incremental Rendering
**File:** `crates/ide/src/inlay_hints.rs` (lines 84-123)
**Category:** Virtual Text Generation
**Code Example:**
```rust
pub(crate) fn inlay_hints(
    db: &RootDatabase,
    file_id: FileId,
    range_limit: Option<TextRange>,
    config: &InlayHintsConfig<'_>,
) -> Vec<InlayHint> {
    let _p = tracing::info_span!("inlay_hints").entered();
    let sema = Semantics::new(db);
    let file_id = sema.attach_first_edition(file_id);
    let file = sema.parse(file_id);
    let file = file.syntax();

    let mut acc = Vec::new();
    let scope = sema.scope(file)?;
    let famous_defs = FamousDefs(&sema, scope.krate());
    let display_target = famous_defs.1.to_display_target(sema.db);

    let ctx = &mut InlayHintCtx::default();
    let mut hints = |event| {
        if let Some(node) = handle_event(ctx, event) {
            hints(&mut acc, ctx, &famous_defs, config, file_id, display_target, node);
        }
    };

    let mut preorder = file.preorder();
    while let Some(event) = preorder.next() {
        if matches!((&event, range_limit), (WalkEvent::Enter(node), Some(range))
            if range.intersect(node.text_range()).is_none())
        {
            preorder.skip_subtree();
            continue;
        }
        hints(event);
    }

    if let Some(range_limit) = range_limit {
        acc.retain(|hint| range_limit.contains_range(hint.range));
    }
    acc
}
```
**Why This Matters for Contributors:** Inlay hints use a preorder tree walk with range-based pruning for performance. When range_limit is provided (visible editor region), subtrees outside the range are skipped entirely. The ctx tracks state across tree traversal (lifetime stacks). Pattern: (1) Walk syntax tree with WalkEvent (Enter/Leave), (2) On Enter: check if in range, skip subtree if not, (3) Maintain context stack for nested scopes, (4) Generate hints for relevant nodes, (5) Filter results to range. This architecture allows fast incremental re-rendering when scrolling - only visible hints are computed.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Algorithms - Incremental Tree Traversal with Pruning
**Rust-Specific Insight:** The preorder() walk returns an iterator of WalkEvent::Enter/Leave pairs - a depth-first traversal that allows stateful processing (Enter = push context, Leave = pop context). The `preorder.skip_subtree()` optimization is critical: when a node is outside range_limit, skip its entire subtree. This uses Iterator's ability to maintain state (the PreorderIter knows what to skip). The closure `|event| { ... }` captures `&mut acc` - safe because closures can capture mutable references, and we guarantee non-overlapping access. The final `acc.retain()` post-filters, catching edge cases.
**Contribution Tip:** Use preorder walk for stateful tree traversal. Check range intersection on Enter events and skip_subtree() when outside visible range. Maintain context in a separate struct (InlayHintCtx) that's updated on Enter/Leave. Generate hints in the event handler, appending to accumulator. Use range_limit.contains_range() for final filtering. Profile with large files to verify pruning works - should be sub-millisecond for visible region only.
**Common Pitfalls:**
- Not using skip_subtree() → process entire file even for small visible region
- Wrong range intersection check → skip nodes that should be processed
- Not maintaining context stack → wrong hints in nested scopes (lifetimes)
- Forgetting final retain() → hints outside range visible to user
- Using recursive traversal → stack overflow on deeply nested code
**Related Patterns in Ecosystem:**
- Tree-sitter tree cursors (similar stateful tree walking)
- DOM traversal with range selection (similar pruning)
- Rowan's SyntaxNode iteration (underlying implementation)
- Virtual scrolling in UIs (similar visibility-based rendering)

---

## Pattern 19: Runnable Discovery with Test Detection
**File:** `crates/ide/src/runnables.rs` (lines 129-189)
**Category:** Executable Test/Binary Detection
**Code Example:**
```rust
#[derive(Debug, Clone, Hash, PartialEq, Eq, UpmapFromRaFixture)]
pub struct Runnable {
    pub use_name_in_title: bool,
    pub nav: NavigationTarget,
    pub kind: RunnableKind,
    pub cfg: Option<CfgExpr>,
    pub update_test: UpdateTest,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RunnableKind {
    TestMod { path: String },
    Test { test_id: TestId, attr: TestAttr },
    Bench { test_id: TestId },
    DocTest { test_id: TestId },
    Bin,
}

pub(crate) fn runnables(db: &RootDatabase, file_id: FileId) -> Vec<Runnable> {
    let sema = Semantics::new(db);
    let mut res = Vec::new();
    let mut in_macro_expansion = FxIndexMap::<hir::HirFileId, Vec<Runnable>>::default();

    let mut add_opt = |runnable: Option<Runnable>, def| {
        if let Some(runnable) = runnable.filter(|runnable| runnable.nav.file_id == file_id) {
            if let Some(def) = def {
                let file_id = match def {
                    Definition::Module(it) => it.declaration_source_range(db).map(|src| src.file_id),
                    Definition::Function(it) => it.source(db).map(|src| src.file_id),
                    _ => None,
                };
                if let Some(file_id) = file_id.filter(|file| file.macro_file().is_some()) {
                    in_macro_expansion.entry(file_id).or_default().push(runnable);
                    return;
                }
            }
            res.push(runnable);
        }
    };

    visit_file_defs(&sema, file_id, &mut |def| {
        let runnable = match def {
            Definition::Module(it) => runnable_mod(&sema, it),
            Definition::Function(it) => runnable_fn(&sema, it),
            Definition::SelfType(impl_) => runnable_impl(&sema, &impl_),
            _ => None,
        };
        add_opt(runnable.or_else(|| module_def_doctest(&sema, def)), Some(def));
    });

    res.sort_by(cmp_runnables);
    res
}
```
**Why This Matters for Contributors:** Runnable discovery walks all definitions in a file, detecting tests, benchmarks, binaries, and doctests. Key challenges: (1) Tests generated by macros need special handling - they're tracked separately and marked with use_name_in_title to disambiguate, (2) Filtering ensures only runnables in the target file appear, (3) Sorting provides consistent ordering. Pattern: use visit_file_defs to find all definitions, classify each as runnable or not, handle macro-generated runnables specially, sort results. This shows how to implement file-wide analysis features that discover specific item types.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Algorithms - Visitor Pattern with Classification
**Rust-Specific Insight:** The `visit_file_defs` function is a visitor pattern using closures - pass `&mut |def| { ... }` to process each definition. FxIndexMap (rustc's hash map) preserves insertion order while allowing deduplication - important for stable runnable ordering. The pattern `.filter(|runnable| runnable.nav.file_id == file_id)` ensures runnables from macro expansions in other files don't appear. The add_opt closure demonstrates Rust's closure capture - it captures `&mut res` and `&mut in_macro_expansion` safely. The cfg: Option<CfgExpr> field allows conditional runnables (#[cfg(test)]).
**Contribution Tip:** Use visit_file_defs for file-wide definition discovery. Match on Definition variants to classify items (Module → runnable_mod, Function → runnable_fn). Check for macro expansion with `.source().filter(|src| src.file_id.macro_file().is_some())` and handle separately. Always filter to target file_id. Sort results with custom comparator for stable ordering. Include cfg expressions from attributes for conditional compilation awareness.
**Common Pitfalls:**
- Not filtering by file_id → runnables from macros in other files shown
- Not handling macro-generated tests → tests invisible to user
- Missing use_name_in_title for macro tests → ambiguous test names in UI
- Not sorting results → unstable ordering, annoying UX
- Forgetting doctest detection → module_def_doctest cases missed
**Related Patterns in Ecosystem:**
- Cargo's test discovery (similar binary/test detection)
- LSP CodeLens (protocol equivalent for runnable indicators)
- Pytest test collection (similar hierarchical test discovery)
- JUnit test runners (similar test classification)

---

## Pattern 20: UpmappingResult for Macro Call/Definition Sites
**File:** `crates/ide/src/navigation_target.rs` (lines 880-916)
**Category:** Macro-Aware Navigation
**Code Example:**
```rust
#[derive(Debug)]
pub struct UpmappingResult<T> {
    /// The macro call site.
    pub call_site: T,
    /// The macro definition site, if relevant.
    pub def_site: Option<T>,
}

impl<T> UpmappingResult<T> {
    pub fn call_site(self) -> T {
        self.call_site
    }

    pub fn collect<FI: FromIterator<T>>(self) -> FI {
        FI::from_iter(self)
    }
}

impl<T> IntoIterator for UpmappingResult<T> {
    type Item = T;
    type IntoIter = <ArrayVec<T, 2> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.def_site
            .into_iter()
            .chain(Some(self.call_site))
            .collect::<ArrayVec<_, 2>>()
            .into_iter()
    }
}

// Usage in navigation:
fn orig_range_with_focus_r(
    db: &RootDatabase,
    hir_file: HirFileId,
    value: TextRange,
    focus_range: Option<TextRange>,
) -> UpmappingResult<(FileRange, Option<TextRange>)> {
    // ... complex logic to determine call_site and def_site ...

    UpmappingResult {
        call_site: (call_site_range, call_site_focus),
        def_site: def_site.map(|(def_site_range, def_site_focus)| {
            (def_site_range, def_site_focus)
        }),
    }
}
```
**Why This Matters for Contributors:** When navigating inside macro expansions, there are two relevant locations: where the macro was called (call_site) and where it's defined (def_site). UpmappingResult captures both, allowing features to show either or both. Pattern: (1) Primary result goes in call_site, (2) Optional alternative goes in def_site, (3) Implement IntoIterator to yield both, (4) Provide call_site() method for when only primary matters. This abstraction is crucial for goto-definition inside macros - you want to jump to the call site but also know the definition. Features should preserve UpmappingResult through their pipeline.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Data Structures - Multi-Result with Primary/Secondary
**Rust-Specific Insight:** UpmappingResult is a specialized Either/Or type optimized for macro navigation. The IntoIterator impl is clever: it uses ArrayVec<T, 2> (stack-allocated, no heap) to yield def_site first, then call_site. This ordering (def before call) matches typical user expectation. The `collect<FI: FromIterator<T>>()` method uses generic return type to allow collecting into Vec, Option, or any FromIterator type. The call_site() consumer is a convenience for features that don't care about def_site. This pattern avoids heap allocation for the common 1-2 item case.
**Contribution Tip:** When features need navigation in macro context, use UpmappingResult to preserve both locations. Map over UpmappingResult maintaining the structure: `nav.map(|n| transform(n))`. Use call_site() when you only care about primary location (most cases). Use into_iter() or collect() when you want to show both options to user (LSP can show multiple locations). Test with code inside macros to ensure both locations are computed correctly.
**Common Pitfalls:**
- Extracting call_site early and losing def_site → can't provide full navigation options
- Reversing def_site/call_site order → confusing UX (user sees definition first)
- Not using ArrayVec → heap allocation for small fixed-size result
- Forgetting to handle None def_site → panics when expecting def_site in all cases
- Not propagating UpmappingResult through feature pipeline → macro info lost
**Related Patterns in Ecosystem:**
- Either<L, R> type (similar sum type)
- Compiler's Span with ExpnData (similar macro location tracking)
- LSP LocationLink (can represent both source and target)
- Source maps in transpilers (similar multi-location mapping)

---

## Pattern 21: Module Feature Organization Pattern
**File:** `crates/ide/src/lib.rs` (lines 18-59)
**Category:** Crate Structure
**Code Example:**
```rust
#[cfg(test)]
mod fixture;

mod markup;
mod navigation_target;

// Each IDE feature gets its own module
mod annotations;
mod call_hierarchy;
mod child_modules;
mod doc_links;
mod expand_macro;
mod extend_selection;
mod fetch_crates;
mod file_structure;
mod folding_ranges;
mod goto_declaration;
mod goto_definition;
mod goto_implementation;
mod goto_type_definition;
mod highlight_related;
mod hover;
mod inlay_hints;
// ... 30+ more feature modules

// Re-export public types from features
pub use crate::{
    annotations::{Annotation, AnnotationConfig, AnnotationKind, AnnotationLocation},
    call_hierarchy::{CallHierarchyConfig, CallItem},
    expand_macro::ExpandedMacro,
    file_structure::{FileStructureConfig, StructureNode, StructureNodeKind},
    folding_ranges::{Fold, FoldKind},
    // ...
};
```
**Why This Matters for Contributors:** rust-analyzer's IDE features are organized as siblings under crates/ide. Each feature is a separate module with:
- Public types re-exported from lib.rs
- Internal implementation kept private
- Config types for customization
- Tests in same module or tests submodule

Pattern for adding new features:
1. Create mod new_feature.rs
2. Declare in lib.rs (mod new_feature;)
3. Add method to impl Analysis
4. Re-export public types
5. Add tests using fixture module

This flat structure keeps features isolated and discoverable. Contributors can understand one feature without understanding others.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Architecture - Flat Module Organization with Facade
**Rust-Specific Insight:** This demonstrates Rust's module system at scale. The crates/ide/src/lib.rs acts as a facade, re-exporting selected types from internal modules. Private modules (mod goto_definition;) keep implementations hidden, public re-exports (pub use crate::goto_definition::GotoDefinitionConfig;) provide clean API surface. The pattern avoids deep nesting (no goto_definition/mod.rs with submodules) - all features are top-level siblings. Conditional compilation (#[cfg(test)] mod fixture;) keeps test infrastructure separate. This is how large Rust projects scale: flat, feature-focused modules with facade-based public API.
**Contribution Tip:** To add a new IDE feature: (1) Create src/my_feature.rs with pub(crate) functions and public types, (2) Add `mod my_feature;` to lib.rs, (3) Add impl Analysis method delegating to my_feature::my_feature(), (4) Re-export public types in lib.rs pub use block, (5) Add tests in my_feature.rs using #[cfg(test)] or separate tests/ file. Follow naming: module name = feature name (snake_case), main function same as module, config type = FeatureConfig.
**Common Pitfalls:**
- Creating nested module hierarchies → hard to navigate, circular dependency risks
- Not re-exporting types → internal paths leak to public API (breaking changes)
- Putting all features in lib.rs → monolithic file, merge conflicts
- Making feature modules public → exposes internals, locks implementation
- Not using pub(crate) for internal helpers → pollutes public API
**Related Patterns in Ecosystem:**
- Tower's middleware modules (similar flat feature organization)
- Tokio's runtime/task/sync modules (similar facade pattern)
- Serde's de/ser modules with re-exports (similar API surface control)
- Linux kernel's flat driver organization (similar feature isolation)

---

## Pattern 22: Edition-Aware Parsing
**File:** `crates/ide/src/goto_definition.rs` (lines 48-50), `crates/ide/src/hover.rs` (lines 136-138)
**Category:** Language Edition Handling
**Code Example:**
```rust
pub(crate) fn goto_definition(
    db: &RootDatabase,
    FilePosition { file_id, offset }: FilePosition,
    config: &GotoDefinitionConfig<'_>,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    let sema = &Semantics::new(db);
    let file = sema.parse_guess_edition(file_id).syntax().clone();
    let edition = sema.attach_first_edition(file_id).edition(db);
    let original_token = pick_best_token(file.token_at_offset(offset), |kind| match kind {
        // ...
        kind if kind.is_keyword(edition) => 2,
        // ...
    })?;
    // ...
}

fn hover_offset(
    sema: &Semantics<'_, RootDatabase>,
    FilePosition { file_id, offset }: FilePosition,
    file: SyntaxNode,
    config: &HoverConfig<'_>,
    edition: Edition,
    display_target: DisplayTarget,
) -> Option<RangeInfo<HoverResult>> {
    let original_token = pick_best_token(file.token_at_offset(offset), |kind| match kind {
        // ...
        kind if kind.is_keyword(edition) => 2,
        // ...
    })?;
    // ...
}
```
**Why This Matters for Contributors:** Rust has multiple editions (2015, 2018, 2021, 2024) with different keyword sets and syntax rules. Features must parse and analyze code edition-correctly. Pattern:
1. Get edition: `let edition = sema.attach_first_edition(file_id).edition(db);`
2. Use edition in parsing: `sema.parse_guess_edition(file_id)`
3. Pass edition to helpers: `kind.is_keyword(edition)`
4. Use edition in display: `name.display(db, edition)`

Edition affects keyword recognition, async/await syntax, try blocks, etc. Always retrieve and pass edition when parsing or formatting. This ensures rust-analyzer works correctly with all Rust editions.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★ (5/5)
**Pattern Classification:** Language Evolution - Edition-Aware Parsing
**Rust-Specific Insight:** Rust editions (2015, 2018, 2021, 2024) are the language's stability mechanism - breaking changes only occur across editions. The pattern `sema.attach_first_edition(file_id).edition(db)` retrieves the edition from Cargo.toml metadata via Salsa queries. The `parse_guess_edition` method uses heuristics when edition is unknown (detached files). Edition affects parsing (try becomes keyword in 2018+, async/await in 2018+), but also display - `display(db, edition)` formats types according to edition rules. This is critical: rust-analyzer must handle all editions simultaneously in one workspace.
**Contribution Tip:** Always fetch edition at the start of feature implementation: `let edition = sema.attach_first_edition(file_id).edition(db);`. Pass edition to: (1) token classification (is_keyword(edition)), (2) parsing (parse_guess_edition), (3) formatting/display (name.display(db, edition)), (4) syntax validation. Test features against all editions - create fixtures with `//- /lib.rs edition:2021` syntax. Handle edition-specific syntax differences explicitly.
**Common Pitfalls:**
- Assuming 2021 syntax everywhere → breaks on legacy codebases
- Not passing edition to is_keyword() → wrong token classification
- Using hardcoded edition in tests → features break on other editions
- Forgetting edition in display → wrong syntax shown to user
- Not testing with multiple editions → silent bugs in less common editions
**Related Patterns in Ecosystem:**
- C++11/14/17/20 standard versions (similar edition mechanism)
- Python 2 vs 3 compatibility (similar language evolution)
- JavaScript "use strict" (similar opt-in language mode)
- Compiler's edition flags (-C edition=2021)

---

## Summary: IDE Features Layer Patterns

This document covered 22 core patterns for implementing IDE features in rust-analyzer. These patterns form the foundation for all language-aware editor functionality.

### Pattern Categories:

**Architectural (5 patterns):**
- Pattern 1: AnalysisHost/Analysis Snapshot - Cancellable query architecture
- Pattern 2: with_db Cancellation Wrapper - Panic-based cancellation
- Pattern 21: Module Feature Organization - Flat feature structure
- Pattern 22: Edition-Aware Parsing - Multi-edition support
- Pattern 20: UpmappingResult - Macro call/definition site handling

**Data Modeling (7 patterns):**
- Pattern 3: FilePosition/FileRange - Position abstractions
- Pattern 4: RangeInfo - Result enrichment
- Pattern 8: NavigationTarget - Rich navigation metadata
- Pattern 11: Markup - Type-safe markdown
- Pattern 13: Config Structs - Feature configuration
- Pattern 14: SignatureHelp - Structured text with metadata
- Pattern 17: Syntax Highlighting - Tagged unions with modifiers

**Name Resolution (2 patterns):**
- Pattern 7: IdentClass - Unified name classification
- Pattern 9: ToNav/TryToNav - Polymorphic navigation

**Tree Traversal (3 patterns):**
- Pattern 5: pick_best_token - Heuristic selection
- Pattern 6: Semantics-Based Descent - Macro-aware traversal
- Pattern 18: Inlay Hints - Incremental tree walk with pruning

**Complex Features (5 patterns):**
- Pattern 12: Hover with Actions - Actionable tooltips
- Pattern 15: Call Hierarchy - Bidirectional graph traversal
- Pattern 16: Reference Search - Categorized results
- Pattern 19: Runnable Discovery - Visitor-based classification
- Pattern 10: Test Fixture Infrastructure - DSL for test data

### Contribution Readiness Checklist

When implementing a new IDE feature, ensure you:

**✅ Architecture:**
- [ ] Wrap all database queries in `with_db(|db| ...)`
- [ ] Return `Cancellable<T>` from public Analysis methods
- [ ] Handle edition via `sema.attach_first_edition(file_id).edition(db)`
- [ ] Use `descend_into_macros` for macro-aware token processing

**✅ Data Modeling:**
- [ ] Use FilePosition for point queries, FileRange for range queries
- [ ] Return `Option<RangeInfo<T>>` for range-annotated results
- [ ] Build NavigationTarget with `from_named()` or `from_syntax()`
- [ ] Use Symbol (interned strings) instead of String for names
- [ ] Create FooConfig struct for features with >3 configuration options
- [ ] Return Markup for user-facing rich text

**✅ Name Resolution:**
- [ ] Use IdentClass/NameClass/NameRefClass for name classification
- [ ] Handle FieldShorthand cases explicitly
- [ ] Match exhaustively on Definition variants
- [ ] Implement ToNavFromAst marker trait for new navigable HIR types

**✅ Tree Traversal:**
- [ ] Use `pick_best_token()` for cursor position token selection
- [ ] Pass edition to token ranking functions
- [ ] Process ALL descended tokens from macro expansion
- [ ] Deduplicate results with `.unique()` or HashSet
- [ ] Use preorder walk with `skip_subtree()` for range-limited features

**✅ Testing:**
- [ ] Write tests using fixture::position() or fixture::range()
- [ ] Test with `$0` cursor markers and `//- /file.rs` multi-file syntax
- [ ] Test inside macro expansions
- [ ] Test with multiple Rust editions (2015, 2018, 2021, 2024)
- [ ] Test edge cases: empty files, EOF, special syntax

**✅ Performance:**
- [ ] Use IntMap for FileId-keyed collections
- [ ] Skip irrelevant subtrees in tree walks
- [ ] Group results by file for efficient display
- [ ] Use zero-allocation iterators where possible
- [ ] Consider config flags to disable expensive features

**✅ UX:**
- [ ] Set focus_range to identifier, full_range to entire item
- [ ] Provide container_name for breadcrumbs
- [ ] Include docs in NavigationTarget
- [ ] Generate appropriate HoverActions for item types
- [ ] Handle UpmappingResult for macro navigation

### Key Rust Idioms Demonstrated:

1. **Ownership for Architecture:** AnalysisHost owns mutable state, Analysis provides immutable snapshots (Arc-based sharing)
2. **Newtypes for Safety:** FilePosition, TextSize, Symbol provide type safety over primitives
3. **Enums for Polymorphism:** Definition, HoverAction, ReferenceCategory use sum types instead of inheritance
4. **Traits for Extensibility:** ToNav/TryToNav with blanket impls eliminate boilerplate
5. **Closures for Customization:** pick_best_token uses closures for zero-cost ranking functions
6. **Iterators for Composition:** filter_map, flat_map, unique chain transformations efficiently
7. **Options for Fallibility:** Option<T> for nullable values, Result<T, E> for errors
8. **Generics for Reuse:** RangeInfo<T>, UpmappingResult<T> work with any payload
9. **Proc-macros for DSLs:** #[rust_analyzer::rust_fixture] creates embedded test DSL
10. **Zero-Cost Abstractions:** All patterns compile to efficient machine code

### Common Anti-Patterns to Avoid:

❌ Accessing `self.db` directly instead of using `with_db`
❌ Using String instead of Symbol for frequently-used names
❌ Returning raw tuples instead of named structs (FilePosition, RangeInfo)
❌ Only processing first descended token from macros
❌ Not deduplicating results from macro expansions
❌ Using HashMap instead of IntMap for FileId keys
❌ Forgetting to filter results to target file_id
❌ Not testing with multiple Rust editions
❌ Ignoring UpmappingResult and losing macro navigation info
❌ Deep module nesting instead of flat feature organization

---
