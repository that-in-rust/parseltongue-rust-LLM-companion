# Idiomatic Rust Patterns: IDE Completion Engine
> Source: rust-analyzer/crates/ide-completion
> Purpose: Patterns for contributing completion features to rust-analyzer

## Pattern 1: Two-Phase Completion Architecture
**File:** crates/ide-completion/src/lib.rs:191-273
**Category:** Completion Engine Architecture
**Code Example:**
```rust
pub fn completions(
    db: &RootDatabase,
    config: &CompletionConfig<'_>,
    position: FilePosition,
    trigger_character: Option<char>,
) -> Option<Vec<CompletionItem>> {
    let (ctx, analysis) = &CompletionContext::new(db, position, config, trigger_character)?;
    let mut completions = Completions::default();

    // Phase 1: Context collection with speculative parsing
    // Phase 2: Completion generation based on context analysis
    match analysis {
        CompletionAnalysis::Name(name_ctx) => completions::complete_name(acc, ctx, name_ctx),
        CompletionAnalysis::NameRef(name_ref_ctx) => {
            completions::complete_name_ref(acc, ctx, name_ref_ctx)
        }
        CompletionAnalysis::Lifetime(lifetime_ctx) => {
            completions::lifetime::complete_label(acc, ctx, lifetime_ctx);
            completions::lifetime::complete_lifetime(acc, ctx, lifetime_ctx);
        }
        // ... other analysis types
    }

    Some(completions.into())
}
```
**Why This Matters for Contributors:** The two-phase design (collect context, then generate completions) separates concerns. Phase 1 handles the messy incomplete syntax parsing, while Phase 2 provides clean APIs for completion providers. When adding new completion types, you only need to focus on Phase 2 logic.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Architectural Separation of Concerns (L2 - Standard Library idioms with Option/Result)
**Rust-Specific Insight:** This pattern leverages Rust's enum dispatch (`CompletionAnalysis`) for zero-cost abstraction between completion types. The `Option<(CompletionContext, CompletionAnalysis)>` return type elegantly handles early failure without exceptions. The match exhaustiveness checking ensures all analysis types are handled.
**Contribution Tip:** When adding a new completion type, create a new `CompletionAnalysis` variant and implement the corresponding `complete_*` function. The compiler will guide you through all required match arms. Test with incomplete/invalid syntax from the start.
**Common Pitfalls:** Don't bypass Phase 1 context collection - it handles cursor position, scope analysis, and syntax validation. Avoid adding completion logic directly to Phase 1; keep it pure context gathering.
**Related Patterns in Ecosystem:** Similar to tower/axum middleware layering (collect request context → dispatch to handlers), LSP protocol request handling (parse → validate → process), and parser combinator two-phase design (tokenize → parse).

---

## Pattern 2: Speculative Parsing with Fake Identifiers
**File:** crates/ide-completion/src/context.rs:701-761
**Category:** Completion Context Analysis
**Code Example:**
```rust
const COMPLETION_MARKER: &str = "raCompletionMarker";

pub(crate) fn new(
    db: &'db RootDatabase,
    position @ FilePosition { file_id, offset }: FilePosition,
    config: &'db CompletionConfig<'db>,
    trigger_character: Option<char>,
) -> Option<(CompletionContext<'db>, CompletionAnalysis<'db>)> {
    let sema = Semantics::new(db);
    let editioned_file_id = sema.attach_first_edition(file_id);
    let original_file = sema.parse(editioned_file_id);

    // Insert a fake ident to get a valid parse tree. We will use this file
    // to determine context, though the original_file will be used for
    // actual completion.
    let file_with_fake_ident = {
        let (_, edition) = editioned_file_id.unpack(db);
        let parse = db.parse(editioned_file_id);
        parse.reparse(TextRange::empty(offset), COMPLETION_MARKER, edition).tree()
    };

    // always pick the token to the immediate left of the cursor
    let original_token = original_file.syntax().token_at_offset(offset).left_biased()?;

    // Analyze using fake ident for context, original for completion items
    // ...
}
```
**Why This Matters for Contributors:** Incomplete code produces broken ASTs. The fake identifier trick (`raCompletionMarker`) creates a valid AST for analysis while preserving the original for actual text edits. This is crucial for understanding cursor position context in incomplete expressions like `foo.ba$0`.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Speculative Execution with RAII Resource Management (Advanced Parser Pattern)
**Rust-Specific Insight:** The `reparse()` method demonstrates Rust's incremental compilation philosophy applied to IDE features. Using `TextRange::empty(offset)` for zero-width insertion is a clever application of Rust's half-open range semantics. The `left_biased()` token selection uses Rust's iterator adapters for precise cursor positioning.
**Contribution Tip:** When working with incomplete syntax, always use the fake-ident tree for semantic analysis but the original tree for text edits. The marker string must be a valid identifier that won't appear in real code. Add `cov_mark::hit!` calls to test that both trees are used correctly.
**Common Pitfalls:** Don't use the fake tree for generating completion text - you'll insert "raCompletionMarker" into user code. Don't assume the fake ident creates a valid parse in all contexts (e.g., inside string literals). Always check both `original_token` and fake tree token.
**Related Patterns in Ecosystem:** Similar to nom's error recovery parsing, tree-sitter's error node handling, and salsa's speculative query execution. Mirrors compiler incremental parsing (rustc's `parse_from_source_str`), and LSP's partial document synchronization.

---

## Pattern 3: Builder Pattern for CompletionItem Construction
**File:** crates/ide-completion/src/item.rs:481-688
**Category:** Item Builder Pattern
**Code Example:**
```rust
#[must_use]
#[derive(Clone)]
pub(crate) struct Builder {
    source_range: TextRange,
    imports_to_add: SmallVec<[LocatedImport; 1]>,
    trait_name: Option<SmolStr>,
    doc_aliases: Vec<SmolStr>,
    label: SmolStr,
    insert_text: Option<String>,
    is_snippet: bool,
    // ... more fields
    edition: Edition,
}

impl Builder {
    pub(crate) fn from_resolution(
        ctx: &CompletionContext<'_>,
        path_ctx: &PathCompletionCtx<'_>,
        local_name: hir::Name,
        resolution: hir::ScopeDef,
    ) -> Self {
        let doc_aliases = ctx.doc_aliases_in_scope(resolution);
        render_path_resolution(
            RenderContext::new(ctx).doc_aliases(doc_aliases),
            path_ctx,
            local_name,
            resolution,
        )
    }

    pub(crate) fn lookup_by(&mut self, lookup: impl Into<SmolStr>) -> &mut Builder {
        self.lookup = Some(lookup.into());
        self
    }

    pub(crate) fn detail(&mut self, detail: impl Into<String>) -> &mut Builder {
        self.set_detail(Some(detail))
    }

    pub(crate) fn with_relevance(
        &mut self,
        relevance: impl FnOnce(CompletionRelevance) -> CompletionRelevance,
    ) -> &mut Builder {
        self.relevance = relevance(mem::take(&mut self.relevance));
        self
    }

    pub(crate) fn build(self, db: &RootDatabase) -> CompletionItem {
        // Final assembly with import path display formatting
        // ...
    }
}
```
**Why This Matters for Contributors:** The builder pattern provides a fluent API for constructing completion items with many optional fields. The `#[must_use]` attribute ensures builders are actually built. The `with_relevance` closure pattern allows modifying relevance without fully replacing it.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Type-State Builder with Compile-Time Safety (Classic Rust API Design Pattern)
**Rust-Specific Insight:** The `#[must_use]` attribute is critical here - forgetting to call `.build()` is a compiler error, preventing silent bugs. The `SmallVec<[LocatedImport; 1]>` optimizes for the common single-import case with stack allocation. Using `SmolStr` (interned strings) reduces memory for repeated labels. The `mem::take` in `with_relevance` demonstrates zero-cost state transfer.
**Contribution Tip:** Always use the builder pattern for completion items - direct construction is error-prone. Chain builder methods for readability. Use `lookup_by()` when the completion text differs from the search key. The `from_resolution()` constructor handles the most common case (completing a HIR item).
**Common Pitfalls:** Don't clone builders unnecessarily - they're cheap but not free. Don't forget `#[must_use]` on custom builders. Don't mutate builder state after calling modification methods (they return `&mut self`, not `Self`). Watch for memory leaks if builders are created but never built.
**Related Patterns in Ecosystem:** Similar to derive_builder pattern, typed-builder crate, bon builder pattern, and axum's Router builder. Mirrors std's process::Command builder, tokio's runtime Builder, and serde_json's json! macro expansion.

---

## Pattern 4: Relevance Scoring System
**File:** crates/ide-completion/src/item.rs:149-359
**Category:** Relevance Scoring
**Code Example:**
```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct CompletionRelevance {
    pub exact_name_match: bool,
    pub type_match: Option<CompletionRelevanceTypeMatch>,
    pub is_local: bool,
    pub trait_: Option<CompletionRelevanceTraitInfo>,
    pub is_name_already_imported: bool,
    pub requires_import: bool,
    pub is_private_editable: bool,
    pub postfix_match: Option<CompletionRelevancePostfixMatch>,
    pub function: Option<CompletionRelevanceFn>,
    pub is_skipping_completion: bool,
}

impl CompletionRelevance {
    const BASE_SCORE: u32 = u32::MAX / 2;

    pub fn score(self) -> u32 {
        let mut score = Self::BASE_SCORE;

        // Incremental scoring based on various factors
        if self.is_local { score += 1; }
        if !self.is_private_editable { score += 1; }
        if self.exact_name_match { score += 20; }
        if self.requires_import { score -= 1; }

        match self.postfix_match {
            Some(CompletionRelevancePostfixMatch::Exact) => score += 100,
            Some(CompletionRelevancePostfixMatch::NonExact) => score -= 5,
            None => (),
        };

        score += match self.type_match {
            Some(CompletionRelevanceTypeMatch::Exact) => 18,
            Some(CompletionRelevanceTypeMatch::CouldUnify) => 5,
            None => 0,
        };

        // Function return type scoring
        if let Some(function) = self.function {
            let mut fn_score = match function.return_type {
                CompletionRelevanceReturnType::DirectConstructor => 15,
                CompletionRelevanceReturnType::Builder => 10,
                CompletionRelevanceReturnType::Constructor => 5,
                CompletionRelevanceReturnType::Other => 0u32,
            };
            // Adjust based on parameters
            score += fn_score;
        }

        score
    }
}
```
**Why This Matters for Contributors:** The scoring system prioritizes completions by usefulness. Each relevance factor has a carefully tuned weight. When adding new completion types, you set relevance fields and the scoring algorithm handles prioritization. The `BASE_SCORE` at `u32::MAX/2` allows both positive and negative adjustments.

### Rust Expert Commentary
**Idiomatic Rating:** 4/5 ★★★★☆
**Pattern Classification:** Declarative Scoring Algorithm with Type-Level Configuration (Performance-Critical Pattern)
**Rust-Specific Insight:** Using `Copy` trait for `CompletionRelevance` enables cheap stack-based scoring without allocations. The `Option<CompletionRelevanceTypeMatch>` enum pattern allows three states (exact/could-unify/no-match) more explicitly than booleans. Starting at `u32::MAX/2` prevents overflow while allowing symmetric positive/negative adjustments. The additive scoring approach is cache-friendly and branch-predictor-friendly.
**Contribution Tip:** When adding new relevance factors, use small integer weights (1-20 range for most factors, 100+ for extremely strong signals like postfix exact match). Test score ordering with real-world completion scenarios. Document the rationale for each weight. Use pattern matching on enums rather than boolean flags for clarity.
**Common Pitfalls:** Don't use floating-point scores (breaks LSP protocol sorting). Don't make weights too large (causes score clustering). Don't add boolean flags without corresponding score adjustments. Avoid complex branching in `score()` - keep it linear for performance.
**Related Patterns in Ecosystem:** Similar to fuzzy-finder scoring (fzf, skim), search ranking algorithms (Elasticsearch BM25), and compiler priority queues. Mirrors tokio's task priority, rayon's work-stealing weights, and LSP's completion item sorting protocol.

---

## Pattern 5: RenderContext for Completion Rendering
**File:** crates/ide-completion/src/render.rs:34-120
**Category:** Render Pattern
**Code Example:**
```rust
#[derive(Debug, Clone)]
pub(crate) struct RenderContext<'a> {
    completion: &'a CompletionContext<'a>,
    is_private_editable: bool,
    import_to_add: Option<LocatedImport>,
    doc_aliases: Vec<SmolStr>,
}

impl<'a> RenderContext<'a> {
    pub(crate) fn new(completion: &'a CompletionContext<'a>) -> RenderContext<'a> {
        RenderContext {
            completion,
            is_private_editable: false,
            import_to_add: None,
            doc_aliases: vec![],
        }
    }

    pub(crate) fn private_editable(mut self, private_editable: bool) -> Self {
        self.is_private_editable = private_editable;
        self
    }

    pub(crate) fn import_to_add(mut self, import_to_add: Option<LocatedImport>) -> Self {
        self.import_to_add = import_to_add;
        self
    }

    pub(crate) fn doc_aliases(mut self, doc_aliases: Vec<SmolStr>) -> Self {
        self.doc_aliases = doc_aliases;
        self
    }

    fn completion_relevance(&self) -> CompletionRelevance {
        CompletionRelevance {
            is_private_editable: self.is_private_editable,
            requires_import: self.import_to_add.is_some(),
            ..Default::default()
        }
    }
}
```
**Why This Matters for Contributors:** `RenderContext` encapsulates rendering state and provides builder methods for configuration. It separates rendering logic from completion logic and ensures consistent rendering across different completion types. The fluent API makes it easy to chain configuration calls.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Render Context with Lifetime-Parametric Builder (Zero-Cost Abstraction)
**Rust-Specific Insight:** The lifetime parameter `'a` ties `RenderContext` to `CompletionContext`, preventing use-after-free without runtime overhead. The `self`-consuming methods enable method chaining while preventing accidental reuse. The `completion_relevance()` method demonstrates derived state computation - relevance is calculated from context rather than stored.
**Contribution Tip:** Always construct `RenderContext` via `RenderContext::new()` for consistent initialization. Chain configuration methods before passing to render functions. The context holds references, so don't store it beyond the completion call. Use `private_editable()` when suggesting private items from editable crates.
**Common Pitfalls:** Don't store `RenderContext` in struct fields - the lifetime makes this difficult. Don't bypass `RenderContext` and access `CompletionContext` directly in render functions. Don't mutate context after passing to render functions (ownership is transferred).
**Related Patterns in Ecosystem:** Similar to askama template context, serde serialization context, and axum's extractors. Mirrors handlebars render context, tera template context, and tower middleware layer context pattern.

---

## Pattern 6: Completions Accumulator Pattern
**File:** crates/ide-completion/src/completions.rs:54-88
**Category:** Completion Collection
**Code Example:**
```rust
#[derive(Debug, Default)]
pub struct Completions {
    buf: Vec<CompletionItem>,
}

impl From<Completions> for Vec<CompletionItem> {
    fn from(val: Completions) -> Self {
        val.buf
    }
}

impl Builder {
    /// Convenience method to add a freshly created completion into accumulator
    pub(crate) fn add_to(self, acc: &mut Completions, db: &RootDatabase) {
        acc.add(self.build(db))
    }
}

impl Completions {
    fn add(&mut self, item: CompletionItem) {
        self.buf.push(item)
    }

    pub(crate) fn add_keyword(&mut self, ctx: &CompletionContext<'_>, keyword: &'static str) {
        let item = CompletionItem::new(
            CompletionItemKind::Keyword,
            ctx.source_range(),
            SmolStr::new_static(keyword),
            ctx.edition,
        );
        item.add_to(self, ctx.db);
    }

    pub(crate) fn add_path_resolution(
        &mut self,
        ctx: &CompletionContext<'_>,
        path_ctx: &PathCompletionCtx<'_>,
        local_name: hir::Name,
        resolution: hir::ScopeDef,
        doc_aliases: Vec<syntax::SmolStr>,
    ) {
        let is_private_editable = match ctx.def_is_visible(&resolution) {
            Visible::Yes => false,
            Visible::Editable => true,
            Visible::No => return,  // Early return for invisible items
        };
        self.add(
            render_path_resolution(
                RenderContext::new(ctx)
                    .private_editable(is_private_editable)
                    .doc_aliases(doc_aliases),
                path_ctx,
                local_name,
                resolution,
            )
            .build(ctx.db),
        );
    }
}
```
**Why This Matters for Contributors:** The accumulator pattern provides a clean API for collecting completions. Type-specific `add_*` methods encapsulate rendering logic and visibility checks. Early returns for invisible items prevent unnecessary work. This pattern makes completion providers simple: just call the appropriate `add_*` method.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Accumulator with Typed Entry Points (Collection Builder Pattern)
**Rust-Specific Insight:** The newtype wrapper around `Vec<CompletionItem>` prevents accidental direct mutation while enabling `From` conversion for zero-cost unwrapping. The `add_to()` method on `Builder` follows Rust's convention of consumers taking `self` (moves the builder). Early returns in `add_path_resolution()` demonstrate the "fail fast" idiom. The `&mut self` signature on `add` methods enables efficient in-place accumulation.
**Contribution Tip:** Always use type-specific `add_*` methods rather than `add()` directly - they handle visibility, rendering, and edge cases. Use early returns for invisible items to avoid expensive render operations. The accumulator owns the `Vec`, so prefer `&mut` references to avoid cloning.
**Common Pitfalls:** Don't call `add()` directly unless you've already performed visibility checks. Don't collect to `Vec` prematurely - let the accumulator own the collection. Don't forget to convert `Completions` to `Vec` at the boundary (the `From` impl handles this). Avoid holding multiple mutable references to the accumulator.
**Related Patterns in Ecosystem:** Similar to std::fmt::Formatter, io::Write, and std::io::BufWriter accumulation. Mirrors syn's fold pattern, quote's token accumulation, and proc-macro2's TokenStream builder pattern.

---

## Pattern 7: PathCompletionCtx for Path Context Analysis
**File:** crates/ide-completion/src/context.rs:68-105
**Category:** Completion Context
**Code Example:**
```rust
#[derive(Debug)]
pub(crate) struct PathCompletionCtx<'db> {
    /// If this is a call with () already there
    pub(crate) has_call_parens: bool,
    /// If this has a macro call bang !
    pub(crate) has_macro_bang: bool,
    /// The qualifier of the current path.
    pub(crate) qualified: Qualified<'db>,
    /// The parent of the path we are completing.
    pub(crate) parent: Option<ast::Path>,
    /// The path of which we are completing the segment
    pub(crate) path: ast::Path,
    /// The path in the original file
    pub(crate) original_path: Option<ast::Path>,
    pub(crate) kind: PathKind<'db>,
    /// Whether the path segment has type args or not.
    pub(crate) has_type_args: bool,
    /// Whether the qualifier comes from a use tree parent or not
    pub(crate) use_tree_parent: bool,
}

impl PathCompletionCtx<'_> {
    pub(crate) fn is_trivial_path(&self) -> bool {
        matches!(
            self,
            PathCompletionCtx {
                has_call_parens: false,
                has_macro_bang: false,
                qualified: Qualified::No,
                parent: None,
                has_type_args: false,
                ..
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PathKind<'db> {
    Expr { expr_ctx: PathExprCtx<'db> },
    Type { location: TypeLocation },
    Attr { attr_ctx: AttrCtx },
    Derive { existing_derives: ExistingDerives },
    Item { kind: ItemListKind },
    Pat { pat_ctx: PatternContext },
    Vis { has_in_token: bool },
    Use,
}
```
**Why This Matters for Contributors:** `PathCompletionCtx` captures all relevant information about the path being completed. The `is_trivial_path()` helper identifies simple unqualified paths, which get different completions. The `PathKind` enum distinguishes expression paths from type paths from attribute paths, etc. This rich context enables context-specific completion logic.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Rich Context Object with Enum Dispatch (Domain Modeling Excellence)
**Rust-Specific Insight:** The lifetime parameter `'db` ties the context to database references, preventing dangling pointers. The `PathKind` enum with associated data (e.g., `expr_ctx`, `attr_ctx`) demonstrates Rust's algebraic data types for exhaustive pattern matching. The `is_trivial_path()` uses struct pattern matching with `..` to ignore irrelevant fields. Boolean flags (`has_call_parens`, `has_macro_bang`) encode syntax state compactly.
**Contribution Tip:** Always check `PathKind` before generating completions - the same syntax has different semantics in different contexts. Use `is_trivial_path()` to optimize common cases. The `original_path` field preserves user syntax for text edits. Match on `PathKind` exhaustively to handle all cases.
**Common Pitfalls:** Don't assume paths are always expressions - they could be types, patterns, or attributes. Don't ignore `has_call_parens` when rendering functions - it determines whether to add `()`. Don't mutate `PathCompletionCtx` after creation - it's immutable context.
**Related Patterns in Ecosystem:** Similar to syn's parse context, rustc's resolution context, and salsa's query context. Mirrors tree-sitter's AST context, LSP's hover context, and cargo's dependency resolution context.

---

## Pattern 8: DotAccess for Field/Method Completion
**File:** crates/ide-completion/src/context.rs:395-427
**Category:** Dot Completion Context
**Code Example:**
```rust
#[derive(Debug)]
pub(crate) struct DotAccess<'db> {
    pub(crate) receiver: Option<ast::Expr>,
    pub(crate) receiver_ty: Option<TypeInfo<'db>>,
    pub(crate) kind: DotAccessKind,
    pub(crate) ctx: DotAccessExprCtx,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DotAccessKind {
    Field {
        /// True if the receiver is an integer and there is no ident
        /// after it yet like `0.$0`
        receiver_is_ambiguous_float_literal: bool,
    },
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DotAccessExprCtx {
    pub(crate) in_block_expr: bool,
    pub(crate) in_breakable: Option<BreakableKind>,
}

// Usage in completions/dot.rs
pub(crate) fn complete_dot(
    acc: &mut Completions,
    ctx: &CompletionContext<'_>,
    dot_access: &DotAccess<'_>,
) {
    let receiver_ty = match dot_access {
        DotAccess { receiver_ty: Some(receiver_ty), .. } => &receiver_ty.original,
        _ => return,
    };

    let has_parens = matches!(dot_access.kind, DotAccessKind::Method);

    complete_fields(acc, ctx, receiver_ty, /* ... */);
    complete_methods(ctx, receiver_ty, &ctx.traits_in_scope(), /* ... */);
}
```
**Why This Matters for Contributors:** `DotAccess` provides all information needed for field and method completions. The `receiver_is_ambiguous_float_literal` flag handles the tricky case where `0.` could be a float or a method call. The separation between `Field` and `Method` kind allows different completion logic (e.g., whether to add parens).

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Domain-Specific Context with Edge Case Handling (Precision Syntax Analysis)
**Rust-Specific Insight:** The `receiver_is_ambiguous_float_literal` flag demonstrates Rust's syntactic subtlety - `0.` could be `0.0` or `0.method()`. The `Option<ast::Expr>` receiver handles incomplete syntax where the receiver hasn't been parsed yet. The `TypeInfo` wrapper provides both original and adjusted types for autoref/autoderef completion. The `DotAccessExprCtx` separates expression context from access kind.
**Contribution Tip:** Always check `receiver_ty` before generating completions - incomplete code might not have a resolved type. Use `receiver_is_ambiguous_float_literal` to filter out method completions on numeric literals with trailing dots. The `kind` field determines completion behavior (fields vs methods).
**Common Pitfalls:** Don't assume `receiver` is always `Some` - incomplete syntax can have `None`. Don't suggest methods on float literals without checking the ambiguous flag. Don't ignore `DotAccessExprCtx` - it affects what completions are valid (e.g., `break` in breakable contexts).
**Related Patterns in Ecosystem:** Similar to rustc's autoderef resolution, method lookup algorithm, and type inference. Mirrors IntelliJ's completion context, VSCode's suggestion context, and tree-sitter's node context pattern.

---

## Pattern 9: Flyimport (Auto-import) Pattern
**File:** crates/ide-completion/src/completions/flyimport.rs:111-148
**Category:** Auto-import Completions
**Code Example:**
```rust
pub(crate) fn import_on_the_fly_path(
    acc: &mut Completions,
    ctx: &CompletionContext<'_>,
    path_ctx: &PathCompletionCtx<'_>,
) -> Option<()> {
    if !ctx.config.enable_imports_on_the_fly {
        return None;
    }

    let qualified = match path_ctx {
        PathCompletionCtx {
            kind:
                PathKind::Expr { .. }
                | PathKind::Type { .. }
                | PathKind::Attr { .. }
                | PathKind::Derive { .. }
                | PathKind::Item { .. }
                | PathKind::Pat { .. },
            qualified,
            ..
        } => qualified,
        _ => return None,
    };

    let potential_import_name = import_name(ctx);
    let qualifier = match qualified {
        Qualified::With { path, .. } => Some(path.clone()),
        _ => None,
    };

    let import_assets = import_assets_for_path(
        ctx,
        &potential_import_name,
        qualifier.clone()
    )?;

    import_on_the_fly(
        acc,
        ctx,
        path_ctx,
        import_assets,
        qualifier.map(|it| it.syntax().clone())
            .or_else(|| ctx.original_token.parent())?,
        potential_import_name,
    )
}
```
**Why This Matters for Contributors:** Auto-import is implemented as a separate completion pass that fuzzy-matches against all importable items. The feature respects LSP capabilities (checking `enable_imports_on_the_fly`). Import suggestions are added to the `imports_to_add` field of `CompletionItem`, which LSP clients can resolve lazily for performance.

### Rust Expert Commentary
**Idiomatic Rating:** 4/5 ★★★★☆
**Pattern Classification:** Lazy Import Resolution with Feature Gating (Performance Optimization Pattern)
**Rust-Specific Insight:** The early return pattern (`return None`) uses Rust's `Option` type for control flow, avoiding deep nesting. The `import_assets_for_path()` call demonstrates query-based architecture - imports are discovered on-demand. The `import_to_add` field uses LSP's resolution protocol to defer expensive import path calculations. Pattern matching with guards (`PathKind::Expr { .. } | ...`) shows exhaustive enum handling.
**Contribution Tip:** Always check `enable_imports_on_the_fly` before fuzzy-searching imports - it's expensive. Use `import_name()` to extract the partial name being typed. Let LSP clients resolve imports lazily via the `additionalTextEdits` field. Test with large dependency graphs to avoid performance regressions.
**Common Pitfalls:** Don't fuzzy-match all crates on every keystroke - use the partial name to filter. Don't resolve import paths eagerly - defer to LSP resolution. Don't forget to check `PathKind` - not all paths support auto-import (e.g., `Vis` paths). Avoid quadratic behavior when many items match.
**Related Patterns in Ecosystem:** Similar to cargo's fuzzy dependency resolution, LSP's completion resolve protocol, and IDE import optimization. Mirrors IntelliJ's auto-import, VSCode's import suggestion, and Eclipse's organize imports pattern.

---

## Pattern 10: Snippet Completion Pattern
**File:** crates/ide-completion/src/snippet.rs:118-164
**Category:** User-defined Snippets
**Code Example:**
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snippet {
    pub postfix_triggers: Box<[Box<str>]>,
    pub prefix_triggers: Box<[Box<str>]>,
    pub scope: SnippetScope,
    pub description: Option<Box<str>>,
    snippet: String,
    requires: Box<[ModPath]>,
}

impl Snippet {
    pub fn new(
        prefix_triggers: &[String],
        postfix_triggers: &[String],
        snippet: &[String],
        description: &str,
        requires: &[String],
        scope: SnippetScope,
    ) -> Option<Self> {
        if prefix_triggers.is_empty() && postfix_triggers.is_empty() {
            return None;
        }
        let (requires, snippet, description) =
            validate_snippet(snippet, description, requires)?;
        Some(Snippet {
            postfix_triggers: postfix_triggers
                .iter().map(String::as_str).map(Into::into).collect(),
            prefix_triggers: prefix_triggers
                .iter().map(String::as_str).map(Into::into).collect(),
            scope,
            snippet,
            description,
            requires,
        })
    }

    /// Returns [`None`] if the required items do not resolve.
    pub(crate) fn imports(&self, ctx: &CompletionContext<'_>) -> Option<Vec<LocatedImport>> {
        import_edits(ctx, &self.requires)
    }

    pub fn snippet(&self) -> String {
        self.snippet.replace("${receiver}", "$0")
    }

    pub fn postfix_snippet(&self, receiver: &str) -> String {
        self.snippet.replace("${receiver}", receiver)
    }
}
```
**Why This Matters for Contributors:** User snippets are first-class completions with optional imports (`requires` field). The `${receiver}` placeholder makes snippets work for both prefix (`fn $0`) and postfix (`foo.arc` → `Arc::new(foo)`) cases. Validation happens at creation time, and import resolution happens lazily only when needed.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** User-Extensible Snippet System with Import Dependencies (Plugin Architecture)
**Rust-Specific Insight:** The `Box<[Box<str>]>` type uses boxed slices for compact memory layout (no capacity field) and boxed strings for interning. The `Option<Self>` return from `new()` enforces validation at construction time, preventing invalid snippets. The `${receiver}` template variable is replaced with `$0` (cursor position) for prefix mode or actual receiver text for postfix mode. The `requires` field specifies import dependencies as `ModPath` for path resolution.
**Contribution Tip:** When creating snippets, always provide either prefix or postfix triggers (or both). Use `${receiver}` for postfix snippets to reference the expression before the dot. Specify `requires` for snippets that need imports (e.g., `Arc::new` requires `std::sync::Arc`). Test that imports resolve correctly.
**Common Pitfalls:** Don't forget to validate snippet syntax at creation time - invalid snippets fail silently. Don't use `${receiver}` in prefix-only snippets. Don't assume imports always resolve - check `imports()` returns `Some`. Avoid complex snippet logic - keep them simple string templates.
**Related Patterns in Ecosystem:** Similar to VSCode snippet syntax, Emacs yasnippet, and Vim UltiSnips. Mirrors TextMate snippet grammar, LSP snippet protocol, and IntelliJ live templates pattern.

---

## Pattern 11: Postfix Completion Pattern
**File:** crates/ide-completion/src/completions/postfix.rs:32-104
**Category:** Postfix Completions
**Code Example:**
```rust
pub(crate) fn complete_postfix(
    acc: &mut Completions,
    ctx: &CompletionContext<'_>,
    dot_access: &DotAccess<'_>,
) {
    if !ctx.config.enable_postfix_completions {
        return;
    }

    let (dot_receiver, receiver_ty, receiver_is_ambiguous_float_literal) =
        match dot_access {
            DotAccess { receiver_ty: Some(ty), receiver: Some(it), kind, .. } => (
                it,
                &ty.original,
                match *kind {
                    DotAccessKind::Field { receiver_is_ambiguous_float_literal } =>
                        receiver_is_ambiguous_float_literal,
                    DotAccessKind::Method => false,
                },
            ),
            _ => return,
        };

    let receiver_text = get_receiver_text(
        &ctx.sema,
        dot_receiver,
        receiver_is_ambiguous_float_literal
    );

    let cap = match ctx.config.snippet_cap {
        Some(it) => it,
        None => return,
    };

    let postfix_snippet = match build_postfix_snippet_builder(ctx, cap, dot_receiver) {
        Some(it) => it,
        None => return,
    };

    // Type-aware postfix completions
    if let Some(drop_trait) = ctx.famous_defs().core_ops_Drop()
        && receiver_ty.impls_trait(ctx.db, drop_trait, &[])
        && let Some(drop_fn) = ctx.famous_defs().core_mem_drop()
        && let Some(path) = ctx.module.find_path(ctx.db, ItemInNs::Values(drop_fn.into()), cfg)
    {
        cov_mark::hit!(postfix_drop_completion);
        let mut item = postfix_snippet(
            "drop",
            "fn drop(&mut self)",
            &format!("{path}($0{receiver_text})", path = path.display(ctx.db, ctx.edition)),
        );
        item.set_documentation(drop_fn.docs(ctx.db));
        item.add_to(acc, ctx.db);
    }

    postfix_snippet("ref", "&expr", &format!("&{receiver_text}")).add_to(acc, ctx.db);
    postfix_snippet("refm", "&mut expr", &format!("&mut {receiver_text}")).add_to(acc, ctx.db);
}
```
**Why This Matters for Contributors:** Postfix completions transform the expression before the dot. The pattern extracts receiver text from the AST, then builds snippets that wrap it. Type-aware completions (like `.drop`) check trait implementations before offering suggestions. This makes completions contextual and safe.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Type-Aware Template Expansion with Trait Resolution (Advanced IDE Feature)
**Rust-Specific Insight:** The `let Some(drop_trait) = ...` with chained `&&` conditions demonstrates Rust 1.65+ let-else pattern for early return on complex predicates. The `impls_trait()` call queries HIR for trait implementation, preventing invalid suggestions. The `cov_mark::hit!` macro enables test coverage verification for specific code paths. The `format_smolstr!` macro creates interned strings efficiently.
**Contribution Tip:** Always check trait implementations before suggesting trait-specific postfix completions. Use `get_receiver_text()` to extract the expression text, respecting float literal edge cases. Require `snippet_cap` to ensure LSP client supports snippets. Add `cov_mark::hit!` calls and corresponding tests.
**Common Pitfalls:** Don't suggest `.drop` on non-Drop types - check the trait. Don't ignore `receiver_is_ambiguous_float_literal` when extracting text. Don't forget to require `snippet_cap` - not all clients support snippets. Avoid suggesting postfix completions that change semantics unexpectedly.
**Related Patterns in Ecosystem:** Similar to IntelliJ's postfix completion, VSCode's user snippets, and ReSharper's live templates. Mirrors rustfmt's macro expansion, clippy's suggestion generation, and rustc's derive macro expansion pattern.

---

## Pattern 12: Test Infrastructure Pattern
**File:** crates/ide-completion/src/tests.rs:96-268
**Category:** Test Infrastructure
**Code Example:**
```rust
const TEST_CONFIG: CompletionConfig<'_> = CompletionConfig {
    enable_postfix_completions: true,
    enable_imports_on_the_fly: true,
    enable_self_on_the_fly: true,
    callable: Some(CallableSnippets::FillArguments),
    add_semicolon_to_unit: true,
    snippet_cap: SnippetCap::new(true),
    // ... more config
};

pub(crate) fn completion_list(#[rust_analyzer::rust_fixture] ra_fixture: &str) -> String {
    completion_list_with_config(TEST_CONFIG, ra_fixture, true, None)
}

#[track_caller]
pub(crate) fn check_edit(
    what: &str,
    #[rust_analyzer::rust_fixture] ra_fixture_before: &str,
    #[rust_analyzer::rust_fixture] ra_fixture_after: &str,
) {
    let ra_fixture_after = trim_indent(ra_fixture_after);
    let (db, position) = position(ra_fixture_before);
    let completions: Vec<CompletionItem> =
        hir::attach_db(&db, || crate::completions(&db, &config, position, None).unwrap());

    let Some((completion,)) = completions
        .iter()
        .filter(|it| it.lookup() == what)
        .collect_tuple()
    else {
        panic!("can't find {what:?} completion in {completions:#?}")
    };

    let mut actual = db.file_text(position.file_id).text(&db).to_string();

    let mut combined_edit = completion.text_edit.clone();
    resolve_completion_edits(&db, &config, position, completion.import_to_add.iter().cloned())
        .into_iter()
        .flatten()
        .for_each(|text_edit| {
            combined_edit.union(text_edit).expect(
                "Failed to apply completion resolve changes: change ranges overlap, but should not",
            )
        });

    combined_edit.apply(&mut actual);
    assert_eq_text!(&ra_fixture_after, &actual)
}

pub(crate) fn check(#[rust_analyzer::rust_fixture] ra_fixture: &str, expect: Expect) {
    let actual = completion_list(ra_fixture);
    expect.assert_eq(&actual);
}
```
**Why This Matters for Contributors:** The test infrastructure uses inline fixtures with `$0` marking cursor position. `check_edit` tests verify that selecting a completion produces the expected code. `check` tests verify the completion list contents. The `expect_test` crate enables snapshot testing where expected output lives in source code.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Inline Fixture Testing with Snapshot Assertions (IDE Testing Best Practice)
**Rust-Specific Insight:** The `#[rust_analyzer::rust_fixture]` procedural macro attribute parses inline Rust code at compile time for type-safe fixtures. The `$0` cursor marker convention avoids magic numbers for position encoding. The `track_caller` attribute provides accurate error locations when assertions fail. The `assert_eq_text!` macro from expect_test enables reviewable diffs in source code. The `union()` method on `TextEdit` composes import edits with completion edits safely.
**Contribution Tip:** Always write `check_edit` tests for completions that insert text - they verify the final code state. Use `check` tests with `expect![]` for verifying completion list contents and order. The `$0` marker indicates cursor position in fixtures. Update snapshots with `env UPDATE_EXPECT=1 cargo test` when output changes intentionally.
**Common Pitfalls:** Don't forget to test import resolution with `check_edit` - imports might fail to apply correctly. Don't assume completion order is stable without explicit ordering. Don't skip edge case tests (empty input, incomplete syntax, ambiguous contexts). Avoid hardcoding positions instead of using `$0`.
**Related Patterns in Ecosystem:** Similar to insta snapshot testing, cucumber inline fixtures, and jest's inline snapshots. Mirrors rustdoc's doc tests, trybuild's compile_fail tests, and compiletest_rs pattern from rustc.

---

## Pattern 13: CompletionConfig for Feature Flags
**File:** crates/ide-completion/src/config.rs:15-82
**Category:** Configuration
**Code Example:**
```rust
#[derive(Clone, Debug)]
pub struct CompletionConfig<'a> {
    pub enable_postfix_completions: bool,
    pub enable_imports_on_the_fly: bool,
    pub enable_self_on_the_fly: bool,
    pub enable_auto_iter: bool,
    pub enable_auto_await: bool,
    pub enable_private_editable: bool,
    pub enable_term_search: bool,
    pub term_search_fuel: u64,
    pub full_function_signatures: bool,
    pub callable: Option<CallableSnippets>,
    pub add_semicolon_to_unit: bool,
    pub snippet_cap: Option<SnippetCap>,
    pub insert_use: InsertUseConfig,
    pub prefer_no_std: bool,
    pub prefer_prelude: bool,
    pub prefer_absolute: bool,
    pub snippets: Vec<Snippet>,
    pub limit: Option<usize>,
    pub fields_to_resolve: CompletionFieldsToResolve,
    pub exclude_flyimport: Vec<(String, AutoImportExclusionType)>,
    pub exclude_traits: &'a [String],
}

impl CompletionConfig<'_> {
    pub fn postfix_snippets(&self) -> impl Iterator<Item = (&str, &Snippet)> {
        self.snippets
            .iter()
            .flat_map(|snip| snip.postfix_triggers.iter().map(move |trigger| (&**trigger, snip)))
    }

    pub fn find_path_config(&self, allow_unstable: bool) -> FindPathConfig {
        FindPathConfig {
            prefer_no_std: self.prefer_no_std,
            prefer_prelude: self.prefer_prelude,
            prefer_absolute: self.prefer_absolute,
            allow_unstable,
        }
    }
}
```
**Why This Matters for Contributors:** All completion features are controlled by `CompletionConfig`. New features should add a corresponding flag. The `snippet_cap` field uses the newtype pattern (`SnippetCap`) to statically ensure snippets are only created when LSP supports them. Helper methods like `find_path_config()` convert completion config to other system configs.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Central Configuration with Type-Safe Capabilities (Feature Flag Pattern)
**Rust-Specific Insight:** The `Option<SnippetCap>` newtype prevents accidental snippet usage when LSP doesn't support them - trying to use snippets requires unwrapping, forcing an explicit capability check. The lifetime parameter `'a` on `exclude_traits` avoids cloning string slices. The `Vec<Snippet>` stores user-defined snippets inline, avoiding indirection. The `find_path_config()` method demonstrates configuration transformation - converting IDE config to import resolution config.
**Contribution Tip:** When adding new features, add a boolean flag to `CompletionConfig` and check it early in the completion function. Use `snippet_cap` to gate snippet generation. The `limit` field caps completion count for performance - respect it. Test with all config combinations to avoid feature interaction bugs.
**Common Pitfalls:** Don't bypass config checks - they respect user preferences and LSP capabilities. Don't clone config unnecessarily - it's passed by reference. Don't forget to add new config fields to the test `TEST_CONFIG` constant. Avoid feature flags that default to expensive operations.
**Related Patterns in Ecosystem:** Similar to cargo's feature flags, rustc's target configuration, and clap's app configuration. Mirrors serde's deserialize config, tokio's runtime config, and axum's router config pattern.

---

## Pattern 14: Visibility Checking Pattern
**File:** crates/ide-completion/src/context.rs:523-595
**Category:** Visibility and Stability
**Code Example:**
```rust
#[derive(Debug)]
pub(crate) enum Visible {
    Yes,
    Editable,
    No,
}

impl CompletionContext<'_> {
    /// Checks if an item is visible and not `doc(hidden)` at the completion site.
    pub(crate) fn def_is_visible(&self, item: &ScopeDef) -> Visible {
        match item {
            ScopeDef::ModuleDef(def) => match def {
                hir::ModuleDef::Module(it) => self.is_visible(it),
                hir::ModuleDef::Function(it) => self.is_visible(it),
                hir::ModuleDef::Adt(it) => self.is_visible(it),
                // ... all module def types
                hir::ModuleDef::BuiltinType(_) => Visible::Yes,
            },
            ScopeDef::GenericParam(_)
            | ScopeDef::ImplSelfType(_)
            | ScopeDef::AdtSelfType(_)
            | ScopeDef::Local(_)
            | ScopeDef::Label(_)
            | ScopeDef::Unknown => Visible::Yes,
        }
    }

    /// Checks if an item is visible, not `doc(hidden)` and stable at the completion site.
    pub(crate) fn is_visible<I>(&self, item: &I) -> Visible
    where
        I: hir::HasVisibility + hir::HasAttrs + hir::HasCrate + Copy,
    {
        let vis = item.visibility(self.db);
        let attrs = item.attrs(self.db);
        self.is_visible_impl(&vis, &attrs, item.krate(self.db))
    }

    fn is_visible_impl(
        &self,
        vis: &hir::Visibility,
        attrs: &hir::AttrsWithOwner,
        defining_crate: hir::Crate,
    ) -> Visible {
        if !self.check_stability(Some(attrs)) {
            return Visible::No;
        }

        if !vis.is_visible_from(self.db, self.module.into()) {
            if !self.config.enable_private_editable {
                return Visible::No;
            }
            // If the definition location is editable, also show private items
            return if is_editable_crate(defining_crate, self.db) {
                Visible::Editable
            } else {
                Visible::No
            };
        }

        if self.is_doc_hidden(attrs, defining_crate) {
            Visible::No
        } else {
            Visible::Yes
        }
    }
}
```
**Why This Matters for Contributors:** The three-way visibility check (`Yes`/`Editable`/`No`) enables showing private items from editable crates while hiding them from dependencies. Stability checking respects nightly vs stable toolchains. The `doc(hidden)` check prevents suggesting internal APIs. This pattern should be used for all completion items.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Three-Valued Logic for Permission Modeling (Fine-Grained Visibility Control)
**Rust-Specific Insight:** The `Visible` enum uses three states instead of `Option<bool>` to distinguish "visible + public" from "visible + private-editable" from "invisible". The trait bound `where I: hir::HasVisibility + hir::HasAttrs + hir::HasCrate + Copy` enables generic visibility checking across all HIR items. The `is_visible_from()` method queries salsa's incremental database for visibility semantics. The early return on unstable items prevents suggesting nightly-only APIs on stable toolchains.
**Contribution Tip:** Always call `def_is_visible()` before adding completions to the accumulator. Use `Visible::Editable` to support "edit private items in local crates" workflow. The stability check requires checking attributes, so it's more expensive - use judiciously. Test with multi-crate workspaces to verify visibility rules.
**Common Pitfalls:** Don't skip visibility checks - suggesting invisible items confuses users. Don't ignore `Editable` state - it enables useful workflows. Don't bypass `doc(hidden)` checks - these items are intentionally hidden. Avoid calling visibility checks repeatedly on the same item - cache results.
**Related Patterns in Ecosystem:** Similar to rustc's privacy checker, cargo's feature resolution, and module visibility rules. Mirrors Java's protected/package-private visibility, C++'s friend declarations, and LSP's symbol visibility protocol.

---

## Pattern 15: Function Rendering with Smart Parameter Handling
**File:** crates/ide-completion/src/render/function.rs:50-182
**Category:** Function Completion Rendering
**Code Example:**
```rust
fn render(
    ctx @ RenderContext { completion, .. }: RenderContext<'_>,
    local_name: Option<hir::Name>,
    func: hir::Function,
    func_kind: FuncKind<'_>,
) -> Builder {
    let db = completion.db;
    let name = local_name.unwrap_or_else(|| func.name(db));

    let (call, escaped_call) = match &func_kind {
        FuncKind::Method(_, Some(receiver)) => (
            format_smolstr!("{}.{}", receiver, name.as_str()),
            format_smolstr!("{}.{}", receiver, name.display(ctx.db(), completion.edition)),
        ),
        _ => (name.as_str().to_smolstr(), name.display(db, completion.edition).to_smolstr()),
    };

    let has_self_param = func.self_param(db).is_some();
    let mut item = CompletionItem::new(
        CompletionItemKind::SymbolKind(if has_self_param {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        }),
        ctx.source_range(),
        call.clone(),
        completion.edition,
    );

    let ret_type = func.ret_type(db);

    // Determine whether to add call parens
    let complete_call_parens = cap
        .filter(|_| !has_call_parens)
        .and_then(|cap| Some((cap, params(ctx.completion, func, &func_kind, has_dot_receiver)?)));

    // Constructor detection for relevance
    let function = assoc_item
        .and_then(|assoc_item| assoc_item.implementing_ty(db))
        .map(|self_type| compute_return_type_match(db, &ctx, self_type, &ret_type))
        .map(|return_type| CompletionRelevanceFn {
            has_params: has_self_param || func.num_params(db) > 0,
            has_self_param,
            return_type,
        });

    item.set_relevance(CompletionRelevance {
        type_match: if has_call_parens || complete_call_parens.is_some() {
            compute_type_match(completion, &ret_type)
        } else {
            compute_type_match(completion, &func.ty(db))
        },
        exact_name_match: compute_exact_name_match(completion, &call),
        function,
        // ...
    });

    if let Some((cap, (self_param, params))) = complete_call_parens {
        add_call_parens(&mut item, completion, cap, call, escaped_call,
                       self_param, params, &ret_type);
    }

    item
}

fn compute_return_type_match(
    db: &dyn HirDatabase,
    ctx: &RenderContext<'_>,
    self_type: hir::Type<'_>,
    ret_type: &hir::Type<'_>,
) -> CompletionRelevanceReturnType {
    if match_types(ctx.completion, &self_type, ret_type).is_some() {
        // fn([..]) -> Self
        CompletionRelevanceReturnType::DirectConstructor
    } else if ret_type
        .type_arguments()
        .any(|ret_type_arg| match_types(ctx.completion, &self_type, &ret_type_arg).is_some())
    {
        // fn([..]) -> Result<Self, E>
        CompletionRelevanceReturnType::Constructor
    } else if ret_type
        .as_adt()
        .map(|adt| adt.name(db).as_str().ends_with("Builder"))
        .unwrap_or(false)
    {
        // fn([..]) -> FooBuilder
        CompletionRelevanceReturnType::Builder
    } else {
        CompletionRelevanceReturnType::Other
    }
}
```
**Why This Matters for Contributors:** Function rendering is complex because it handles methods vs functions, decides whether to add `()`, generates parameter snippets, and detects constructors. The constructor detection pattern (checking if return type is `Self` or `Result<Self>`) boosts relevance for builder patterns. The escaped name handling ensures raw identifiers render correctly.

### Rust Expert Commentary
**Idiomatic Rating:** 5/5 ★★★★★
**Pattern Classification:** Context-Sensitive Rendering with Heuristic Constructor Detection (Smart Code Generation)
**Rust-Specific Insight:** The `format_smolstr!` macro creates interned strings efficiently for method calls like `receiver.method`. The constructor detection uses `type_arguments()` to check if `Self` appears in generic positions (e.g., `Result<Self, E>`). The builder heuristic checks if the return type ADT name ends with "Builder" - a naming convention detection. The escaped name handling (`display(db, edition)`) ensures raw identifiers like `r#type` render correctly in completions.
**Contribution Tip:** Always use `compute_return_type_match()` to detect constructors and boost relevance. Handle both static functions (`Foo::new()`) and methods (`self.build()`). Use `has_call_parens` to avoid adding duplicate `()`. The `complete_call_parens` logic generates parameter snippets when LSP supports it.
**Common Pitfalls:** Don't add `()` when `has_call_parens` is true - the user already typed them. Don't assume all functions returning `Self` are constructors - check if they're associated functions. Don't forget escaped name handling - raw identifiers break without it. Avoid over-detecting builders (false positives on types ending in "Builder").
**Related Patterns in Ecosystem:** Similar to IntelliJ's method completion, VSCode's signature help, and LSP's completion item detail. Mirrors rustdoc's function rendering, rust-lang.org API docs, and clippy's lint suggestion formatting.

---

## Summary: IDE Completion Pattern Architecture

### Core Architectural Principles

**1. Separation of Concerns via Two-Phase Design**
The completion engine separates context analysis (Phase 1) from completion generation (Phase 2). This enables robust handling of incomplete syntax while maintaining clean APIs for completion providers.

**2. Type-Safe Configuration with Newtype Pattern**
Critical capabilities like `SnippetCap` use newtypes to prevent misuse at compile time. The `#[must_use]` attribute on builders ensures completions are actually built.

**3. Rich Context Objects with Lifetime Discipline**
Context types like `PathCompletionCtx` and `DotAccess` capture all relevant information with precise lifetime annotations, preventing use-after-free without runtime overhead.

**4. Declarative Relevance Scoring**
The additive scoring system uses small integer weights for cache-friendly, predictable ordering. The `Copy` trait on `CompletionRelevance` enables cheap scoring calculations.

**5. Three-Valued Visibility Logic**
The `Visible` enum distinguishes public items, editable private items, and invisible items, enabling sophisticated privacy-aware completions.

**6. Speculative Parsing with Fake Identifiers**
The `raCompletionMarker` technique creates valid ASTs from incomplete code for semantic analysis while preserving original syntax for text edits.

### Performance Characteristics

- **Memory Efficiency**: `SmolStr` for interned strings, `SmallVec<[LocatedImport; 1]>` for stack-allocated imports, boxed slices for compact storage
- **Lazy Resolution**: Import paths and snippet expansions computed on-demand via LSP resolution protocol
- **Early Exits**: Visibility checks and feature flags exit early to avoid expensive operations
- **Zero-Cost Abstractions**: Lifetime-parametric contexts, enum dispatch, and `Copy` types prevent runtime overhead

### Integration Points

**LSP Protocol Compliance**
- Completion items follow LSP 3.x protocol structure
- Additional text edits for imports use LSP's resolution mechanism
- Snippet syntax follows LSP snippet specification
- Capability negotiation via `SnippetCap` newtype

**HIR Integration**
- Semantic analysis via `hir::ScopeDef` resolution
- Trait implementation queries via `impls_trait()`
- Type inference via `TypeInfo` wrappers
- Module path finding via `find_path()`

**Salsa Query System**
- Incremental parsing with `db.parse()` and `reparse()`
- Visibility checking via `is_visible_from()`
- Stability attribute queries via `HasAttrs`
- Edition-aware parsing and rendering

### Testing Strategy

**Snapshot Testing with Inline Fixtures**
The `expect_test` crate enables reviewable test output in source code. The `$0` cursor marker provides type-safe position encoding.

**Coverage Verification**
The `cov_mark::hit!` macro enables testing specific code paths, especially edge cases like ambiguous float literals and trait-specific completions.

**Edit Verification**
The `check_edit` pattern verifies that completion selection produces correct final code, including import insertion and snippet expansion.

---

## Contribution Readiness Checklist

### Before Submitting Completion Features

#### 1. Context Analysis
- [ ] Identify which `CompletionAnalysis` variant your feature belongs to
- [ ] Add necessary fields to existing context types or create new ones
- [ ] Ensure lifetime parameters correctly tie context to database
- [ ] Test with incomplete/invalid syntax using `$0` fixtures

#### 2. Completion Generation
- [ ] Use appropriate `add_*` method on `Completions` accumulator
- [ ] Call `def_is_visible()` for all HIR items before suggesting
- [ ] Set relevance fields correctly (exact name match, type match, etc.)
- [ ] Handle raw identifiers with `display(db, edition)`

#### 3. Configuration
- [ ] Add feature flag to `CompletionConfig` if optional
- [ ] Check config early to avoid expensive operations
- [ ] Update `TEST_CONFIG` constant for testing
- [ ] Document config field purpose and default value

#### 4. Rendering
- [ ] Use `RenderContext` builder pattern for consistency
- [ ] Set `private_editable` when suggesting from editable crates
- [ ] Use `format_smolstr!` for efficient string construction
- [ ] Check `snippet_cap` before generating snippets

#### 5. Relevance Scoring
- [ ] Set `exact_name_match` for prefix matches
- [ ] Set `type_match` for type-aware completions
- [ ] Set `is_local` for local variables/items
- [ ] Add new relevance factors if needed (document weight rationale)

#### 6. Auto-Import Support
- [ ] Check `enable_imports_on_the_fly` config flag
- [ ] Use `import_to_add` field for import suggestions
- [ ] Let LSP clients resolve imports lazily
- [ ] Test with multi-crate workspaces

#### 7. Testing Requirements
- [ ] Write `check` test with `expect![]` for completion list
- [ ] Write `check_edit` tests for text insertion
- [ ] Add `cov_mark::hit!` for edge cases
- [ ] Test visibility, stability, and `doc(hidden)` filtering
- [ ] Verify performance with large dependency graphs

#### 8. Edge Cases to Handle
- [ ] Incomplete syntax (missing tokens, unfinished expressions)
- [ ] Ambiguous float literals (`0.` could be `0.0` or method call)
- [ ] Raw identifiers (`r#type`, `r#match`)
- [ ] Generic functions with complex bounds
- [ ] Trait methods with auto-deref/auto-ref
- [ ] Postfix completions on complex expressions

#### 9. Performance Considerations
- [ ] Early return for invisible/unstable items
- [ ] Avoid quadratic behavior in loops
- [ ] Use `SmolStr` for repeated string values
- [ ] Respect `limit` config for completion count
- [ ] Profile with `cargo bench` if adding expensive operations

#### 10. Documentation
- [ ] Add doc comments explaining feature purpose
- [ ] Document relevance scoring rationale
- [ ] Add examples to guide future contributors
- [ ] Update architecture docs if adding new patterns

### Common Mistakes to Avoid

**❌ Don't:**
- Skip visibility checks (use `def_is_visible()` always)
- Add completions directly to `Vec` (use `Completions` accumulator)
- Bypass `RenderContext` (it ensures consistent rendering)
- Ignore `has_call_parens` (causes duplicate parentheses)
- Generate snippets without checking `snippet_cap`
- Mutate context after passing to render functions
- Use floating-point relevance scores (LSP requires integers)
- Forget to test with incomplete/invalid syntax
- Hardcode positions instead of using `$0` marker
- Clone builders unnecessarily (they're cheap but not free)

**✅ Do:**
- Use type-specific `add_*` methods on accumulator
- Set multiple relevance fields for accurate scoring
- Test with multi-crate workspaces
- Add `cov_mark::hit!` for edge cases
- Update snapshot tests when output changes intentionally
- Profile performance with realistic dependency graphs
- Document why weights are chosen for relevance
- Handle raw identifiers with `display(db, edition)`
- Use early returns to avoid expensive operations
- Follow the builder pattern for `CompletionItem`

### Integration Testing Commands

```bash
# Run all completion tests
cargo test -p ide-completion

# Run specific test file
cargo test -p ide-completion --test completion_tests

# Update snapshot expectations
env UPDATE_EXPECT=1 cargo test -p ide-completion

# Check coverage for specific module
cargo llvm-cov --package ide-completion

# Profile completion performance
cargo bench -p ide-completion

# Verify no regressions with manual testing
code /path/to/rust-project  # Open in VSCode with rust-analyzer
# Type incomplete code and verify completions appear correctly
```

### Key Files to Study

**Core Architecture:**
- `crates/ide-completion/src/lib.rs` - Entry point, two-phase design
- `crates/ide-completion/src/context.rs` - Context analysis, speculative parsing
- `crates/ide-completion/src/completions.rs` - Accumulator pattern

**Rendering:**
- `crates/ide-completion/src/render.rs` - RenderContext pattern
- `crates/ide-completion/src/render/function.rs` - Function rendering, constructor detection
- `crates/ide-completion/src/item.rs` - Builder pattern, relevance scoring

**Feature Modules:**
- `crates/ide-completion/src/completions/flyimport.rs` - Auto-import pattern
- `crates/ide-completion/src/completions/postfix.rs` - Postfix completion, type-aware snippets
- `crates/ide-completion/src/snippet.rs` - User-defined snippets

**Testing:**
- `crates/ide-completion/src/tests.rs` - Test infrastructure, fixture patterns

### Advanced Contribution Patterns

**Adding a New Completion Type:**
1. Add variant to `CompletionAnalysis` enum
2. Create context type (e.g., `FooCompletionCtx`)
3. Add analysis logic in `CompletionContext::new()`
4. Implement `complete_foo()` function
5. Add tests with `check()` and `check_edit()`

**Adding Type-Aware Completion:**
1. Query `receiver_ty` from context
2. Use `impls_trait()` for trait checks
3. Set `type_match` relevance field
4. Test with multiple type scenarios

**Adding Configurable Feature:**
1. Add flag to `CompletionConfig`
2. Check flag early in completion function
3. Update `TEST_CONFIG` constant
4. Document config purpose and default

**Optimizing Performance:**
1. Profile with `cargo bench`
2. Add early returns for common cases
3. Use `limit` config to cap results
4. Avoid repeated HIR queries (cache in context)
5. Test with large workspaces

This documentation provides a comprehensive guide to contributing IDE completion features to rust-analyzer, covering architecture, patterns, testing, and common pitfalls. Follow the checklist to ensure high-quality contributions that integrate seamlessly with the existing codebase.

---
