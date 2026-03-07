# Idiomatic Rust Patterns: IDE Diagnostics
> Source: rust-analyzer/crates/ide-diagnostics
>
> **Purpose**: Diagnostic reporting system for Rust code analysis - structured diagnostics with fixes, severity mapping, experimental gating, and lint attribute handling.

## Pattern 1: Diagnostic Handler Registration Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 388-470)
**Category:** Diagnostic Registration & Dispatch

**Code Example:**
```rust
// Central diagnostic dispatch using exhaustive match on AnyDiagnostic enum
for diag in diags {
    let d = match diag {
        AnyDiagnostic::AwaitOutsideOfAsync(d) => handlers::await_outside_of_async::await_outside_of_async(&ctx, &d),
        AnyDiagnostic::CastToUnsized(d) => handlers::invalid_cast::cast_to_unsized(&ctx, &d),
        AnyDiagnostic::ExpectedFunction(d) => handlers::expected_function::expected_function(&ctx, &d),
        AnyDiagnostic::InactiveCode(d) => match handlers::inactive_code::inactive_code(&ctx, &d) {
            Some(it) => it,
            None => continue,  // Handler can opt out of emitting diagnostic
        }
        AnyDiagnostic::IncoherentImpl(d) => handlers::incoherent_impl::incoherent_impl(&ctx, &d),
        AnyDiagnostic::IncorrectCase(d) => handlers::incorrect_case::incorrect_case(&ctx, &d),
        // ... 40+ diagnostic handlers
    };
    res.push(d)
}
```

**Why This Matters for Contributors:**
- **Exhaustive Matching**: Adding new diagnostics requires updating the central match
- **Optional Emission**: Handlers return `Option<Diagnostic>` to conditionally suppress diagnostics
- **Uniform Context**: All handlers receive `&DiagnosticsContext` with semantic info
- **Type-Safe Dispatch**: Compiler ensures all diagnostic variants are handled

---

### 🔍 Expert Rust Commentary: Pattern 1

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Algebraic Dispatch + Type-Safe Extensibility

**Rust-Specific Insight:**
This is a textbook example of leveraging Rust's exhaustive match checking for compile-time guarantees in plugin architectures. The `AnyDiagnostic` enum acts as a type-safe registry where adding a new diagnostic variant causes compilation failure until the dispatch match is updated. This is superior to dynamic dispatch (trait objects) or hash-map lookups because:

1. **Zero Runtime Cost**: The match compiles to a jump table or series of comparisons
2. **Impossible to Forget**: Cannot ship code with unhandled diagnostic types
3. **Refactoring Safety**: Renaming a variant updates all match sites automatically

The `Option<Diagnostic>` return type from handlers is elegant—it distinguishes "diagnostic doesn't apply" from "diagnostic suppressed by config" without needing a separate enum. This follows the Rust idiom of using `Option` for optional results rather than sentinel values.

**Contribution Tip:**
When adding a new diagnostic:
1. Define the variant in `AnyDiagnostic` enum
2. Compiler will error at the dispatch match—add your handler there
3. Use `None` return to conditionally suppress (e.g., for `inactive_code` in macro contexts)
4. The type system prevents you from forgetting any integration point

**Common Pitfalls:**
- **Forgetting `continue` on `None`**: Without it, you'll try to push `None` and get a type error. The pattern `match handlers::foo(...) { Some(it) => it, None => continue }` is correct.
- **Breaking exhaustiveness**: Adding `_ => unreachable!()` defeats the safety benefit. Keep matches exhaustive.
- **Handler side effects**: Handlers should be pure—don't mutate context, just return diagnostics.

**Related Patterns in Ecosystem:**
- **rustc's `DiagnosticBuilder`**: Similar dispatch but with more ceremony (session-based context)
- **Clippy's lint registration**: Uses declarative macros + static arrays, less type-safe
- **Tower middleware**: Uses trait objects for dynamic dispatch, trading compile-time safety for runtime flexibility

**Architectural Note:**
This pattern scales well to 100+ diagnostic types because the match is optimized by LLVM. The linear appearance doesn't reflect runtime cost—expect O(1) jump table dispatch in release builds.

---

## Pattern 2: Builder Pattern for Diagnostic Construction
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 152-221)
**Category:** Diagnostic Creation

**Code Example:**
```rust
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub range: FileRange,
    pub severity: Severity,
    pub unused: bool,
    pub experimental: bool,
    pub fixes: Option<Vec<Assist>>,
    pub main_node: Option<InFile<SyntaxNodePtr>>,
}

impl Diagnostic {
    fn new(code: DiagnosticCode, message: impl Into<String>, range: impl Into<FileRange>) -> Diagnostic {
        Diagnostic {
            code,
            message: message.into(),
            range: range.into(),
            severity: match code {
                DiagnosticCode::RustcHardError(_) | DiagnosticCode::SyntaxError => Severity::Error,
                DiagnosticCode::RustcLint(_) => Severity::Warning,
                DiagnosticCode::Clippy(_) => Severity::WeakWarning,
                DiagnosticCode::Ra(_, s) => s,
            },
            unused: false,
            experimental: true,
            fixes: None,
            main_node: None,
        }
    }

    fn stable(mut self) -> Diagnostic {
        self.experimental = false;
        self
    }

    fn with_main_node(mut self, main_node: InFile<SyntaxNodePtr>) -> Diagnostic {
        self.main_node = Some(main_node);
        self
    }

    fn with_fixes(mut self, fixes: Option<Vec<Assist>>) -> Diagnostic {
        self.fixes = fixes;
        self
    }

    fn with_unused(mut self, unused: bool) -> Diagnostic {
        self.unused = unused;
        self
    }
}
```

**Why This Matters for Contributors:**
- **Fluent API**: Chain methods like `.stable().with_fixes(fixes).with_unused(true)`
- **Default Severity**: Automatically derived from diagnostic code type
- **Experimental by Default**: New diagnostics are experimental until `.stable()` called
- **Type-Driven Construction**: `new()` requires code, message, and range; everything else optional

---

### 🔍 Expert Rust Commentary: Pattern 2

**Idiomatic Rating:** ⭐⭐⭐⭐ (4/5)

**Pattern Classification:** Consuming Builder + Type-State Lite

**Rust-Specific Insight:**
This builder deviates from the typical Rust builder pattern in a pragmatic way: it consumes `self` in each method rather than borrowing, and doesn't use the type-state pattern to enforce required fields. This is appropriate because:

1. **Diagnostic construction is leaf operation**: Once built, you never need the builder again
2. **Defaults are sensible**: `experimental: true` is a safe default—better to have false negatives initially
3. **Performance**: Consuming avoids clone on final build step

The automatic severity derivation from `DiagnosticCode` is clever—it centralizes the rustc/clippy/ra severity mapping in one place. Notice how the code type determines severity, making it impossible to create a `RustcHardError` with `Severity::Warning`.

**Key Design Choice**: `experimental: true` as the default means new diagnostics must opt-in to stability via `.stable()`. This is **fail-safe design**—better to under-report than spam users with false positives.

**Contribution Tip:**
The builder chain is intentionally minimal:
```rust
Diagnostic::new(code, message, range)
    .stable()                    // Only if thoroughly tested
    .with_fixes(fixes)           // Only if fixes available
    .with_unused(true)           // Only for dead code diagnostics
    .with_main_node(node)        // For macro filtering
```

Don't add builder methods "just in case"—keep the API minimal. If you need complex initialization, consider a separate constructor like `new_with_syntax_node_ptr()`.

**Common Pitfalls:**
- **Forgetting `.stable()`**: Your diagnostic will be hidden by default if users have `disable_experimental: true`
- **Wrong severity for code type**: The match in `new()` will override your manual setting
- **Not setting `main_node`**: Required for macro-aware filtering to work

**Related Patterns in Ecosystem:**
- **`proc_macro::Diagnostic`**: Similar but uses borrows (different lifetime constraints)
- **`miette::Diagnostic`**: Richer builder with source code snippets, labels, help text
- **`codespan-reporting::Diagnostic`**: More complex builder with multi-label support

**Why Not Full Type-State?**
Type-state builders (where `Builder<Init> -> Builder<WithCode> -> Builder<WithRange>`) would be overkill here because:
1. All fields have sensible defaults or are derived
2. Only 3 fields are truly required, passed to `new()`
3. The error messages from missing `new()` args are clear enough

---

## Pattern 3: Enum-Based Diagnostic Code System
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 111-150)
**Category:** Diagnostic Classification

**Code Example:**
```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    RustcHardError(&'static str),  // E0123
    SyntaxError,
    RustcLint(&'static str),       // unused_variables
    Clippy(&'static str),          // clippy::needless_borrow
    Ra(&'static str, Severity),    // rust-analyzer custom diagnostic
}

impl DiagnosticCode {
    pub fn url(&self) -> String {
        match self {
            DiagnosticCode::RustcHardError(e) => {
                format!("https://doc.rust-lang.org/stable/error_codes/{e}.html")
            }
            DiagnosticCode::SyntaxError => {
                String::from("https://doc.rust-lang.org/stable/reference/")
            }
            DiagnosticCode::RustcLint(e) => {
                format!("https://doc.rust-lang.org/rustc/?search={e}")
            }
            DiagnosticCode::Clippy(e) => {
                format!("https://rust-lang.github.io/rust-clippy/master/#/{e}")
            }
            DiagnosticCode::Ra(e, _) => {
                format!("https://rust-analyzer.github.io/book/diagnostics.html#{e}")
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticCode::RustcHardError(r)
            | DiagnosticCode::RustcLint(r)
            | DiagnosticCode::Clippy(r)
            | DiagnosticCode::Ra(r, _) => r,
            DiagnosticCode::SyntaxError => "syntax-error",
        }
    }
}
```

**Why This Matters for Contributors:**
- **Documentation Links**: Each code type maps to appropriate documentation URL
- **Severity Embedding**: `Ra` variant stores severity, others derive it
- **Uniform Access**: `as_str()` provides consistent string representation
- **Type Safety**: Distinguish rustc errors, lints, clippy, and custom diagnostics at compile time

---

### 🔍 Expert Rust Commentary: Pattern 3

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Tagged Union + Smart Constructors

**Rust-Specific Insight:**
This is a masterclass in using enums to encode invariants. The `DiagnosticCode` enum achieves three goals simultaneously:

1. **Type-Safe Classification**: Impossible to create a Clippy diagnostic with an error code like `E0425`
2. **Documentation Link Generation**: Each variant knows how to construct its own docs URL
3. **Severity Embedding**: The `Ra` variant includes severity as data, while others derive it

The `&'static str` usage is intentional—diagnostic codes are compile-time constants, so no allocation needed. This is zero-cost abstraction at its finest.

**Clever Detail**: The `url()` method uses match-based dispatch to generate appropriate doc links. This demonstrates **data-driven behavior**—the enum variant determines not just what data is stored, but what operations are valid.

**Contribution Tip:**
When adding a new diagnostic code category (unlikely but possible):
1. Add a new enum variant with appropriate payload
2. Update `url()` to generate correct doc link
3. Update `as_str()` to extract the string representation
4. Update severity derivation in `Diagnostic::new()`

The pattern enforces consistency: you can't add a code type without also defining its documentation and severity semantics.

**Common Pitfalls:**
- **Dynamic strings**: Don't use `String` for codes—they're always static in rust-analyzer. `&'static str` prevents allocations.
- **Wrong variant**: Using `RustcLint` for hard errors will give incorrect severity. The type system prevents mixing compile errors and lints.
- **URL construction**: Ensure new URLs follow the established pattern (HTTPS, official rust-lang domains)

**Related Patterns in Ecosystem:**
- **rustc's `DiagnosticId`**: More complex with `Error(ErrorCode)` and `Lint(LintId)` wrappers
- **Clippy's lint naming**: Flat strings, no type safety
- **IntelliJ Rust**: Uses string-based inspection IDs, less type-safe

**Design Insight:**
The enum approach is superior to trait-based abstraction because:
1. **Closed Set**: Diagnostic types are known at compile time
2. **No Virtual Dispatch**: `url()` and `as_str()` compile to direct calls
3. **Exhaustiveness**: Adding a new variant requires updating all match sites

**Trivia**: The `Ra` variant name stands for "Rust-Analyzer"—custom diagnostics not from rustc/clippy.

---

## Pattern 4: DiagnosticsContext Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 279-286)
**Category:** Context Passing

**Code Example:**
```rust
struct DiagnosticsContext<'a> {
    config: &'a DiagnosticsConfig,
    sema: Semantics<'a, RootDatabase>,
    resolve: &'a AssistResolveStrategy,
    edition: Edition,
    display_target: DisplayTarget,
    is_nightly: bool,
}

// Used in handler signature:
pub(crate) fn unresolved_ident(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedIdent,
) -> Diagnostic {
    let mut range = ctx.sema.diagnostics_display_range(d.node.map(...));
    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0425"),
        "no such value in this scope",
        range
    )
}
```

**Why This Matters for Contributors:**
- **Centralized Context**: All handlers receive same context object
- **Semantic Access**: `sema` field provides type resolution, name lookup, etc.
- **Configuration**: Access to disabled diagnostics, fix preferences
- **Edition Awareness**: Diagnostic behavior can vary by Rust edition
- **Nightly Detection**: Some diagnostics only available on nightly

---

### 🔍 Expert Rust Commentary: Pattern 4

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Context Object + Semantic Analysis Facade

**Rust-Specific Insight:**
The `DiagnosticsContext` is a textbook implementation of the **Context Object pattern** with Rust-specific lifetime optimization. Notice the `<'a>` lifetime parameter—all borrowed data (`config`, `resolve`) shares the same lifetime, simplifying the signature and enabling borrowck-friendly handler composition.

The `Semantics<'a, RootDatabase>` field is particularly clever: it's a wrapper around the salsa database that provides:
1. **Scope-based name resolution**: `scope()` returns items visible at a syntax node
2. **Type inference access**: Through `sema`, handlers can query inferred types
3. **Macro expansion**: `diagnostics_display_range()` maps macro-expanded spans back to source

**Key Design Choice**: All handlers receive `&DiagnosticsContext` rather than individual fields. This:
- **Prevents signature churn**: Adding new context fields doesn't break 40+ handlers
- **Enforces uniform access**: All handlers see the same semantic info
- **Enables future extension**: Can add fields without breaking API

**Contribution Tip:**
When writing handlers:
```rust
pub(crate) fn my_diagnostic(
    ctx: &DiagnosticsContext<'_>,  // Accept the context
    d: &hir::MyDiagnostic,
) -> Diagnostic {
    // Access semantics
    let ty = ctx.sema.type_of_expr(&expr)?;

    // Check edition-specific behavior
    if ctx.edition >= Edition::Edition2021 {
        // Different diagnostic message for 2021+
    }

    // Respect nightly-only features
    if !ctx.is_nightly {
        return None; // Suppress unstable diagnostic
    }
}
```

**Common Pitfalls:**
- **Storing `Semantics`**: Don't cache `ctx.sema` in local variables across calls—it's cheaply cloned
- **Ignoring edition**: Many diagnostics behave differently in different editions (e.g., 2018 vs 2021 closure capture rules)
- **Expensive sema queries**: `scope()` and `type_of_expr()` are cached by salsa, but still avoid redundant calls

**Related Patterns in Ecosystem:**
- **rustc's `TypeckResults`**: Similar semantic context but more specialized
- **Clippy's `LateContext`**: Heavier, includes HIR maps and type context
- **IntelliJ's `PsiElement` + `JavaResolveResult`**: Similar semantic facade in Java

**Performance Note:**
The `Semantics` wrapper provides memoization through salsa's query system. Calling `ctx.sema.type_of_expr()` on the same expression twice hits a cache, making repeated queries cheap.

**Architectural Insight:**
The context pattern decouples diagnostic handlers from the database layer. Handlers don't need to know about salsa queries—they just call methods on `Semantics`, which translates them to cached queries.

---

## Pattern 5: Fix Generation Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/handlers/missing_fields.rs (lines 54-180)
**Category:** Quick Fix Creation

**Code Example:**
```rust
fn fixes(ctx: &DiagnosticsContext<'_>, d: &hir::MissingFields) -> Option<Vec<Assist>> {
    // Early return if unfixable
    if d.missed_fields.iter().any(|it| it.as_tuple_index().is_some()) {
        return None;
    }

    let root = ctx.sema.db.parse_or_expand(d.file);
    let current_module = ctx.sema.scope(d.field_list_parent.to_node(&root).syntax())
        .map(|it| it.module());

    let build_text_edit = |new_syntax: &SyntaxNode, old_syntax| {
        let edit = {
            let old_range = ctx.sema.original_range_opt(old_syntax)?;
            let mut builder = TextEdit::builder();
            if d.file.is_macro() {
                // Can't use diff in macros, replace entire node
                builder.replace(old_range.range, new_syntax.to_string());
            } else {
                diff(old_syntax, new_syntax).into_text_edit(&mut builder);
            }
            builder.finish()
        };
        Some(vec![fix(
            "fill_missing_fields",
            "Fill struct fields",
            SourceChange::from_text_edit(range.file_id.file_id(ctx.sema.db), edit),
            range.range,
        )])
    };

    match &d.field_list_parent.to_node(&root) {
        Either::Left(field_list_parent) => {
            // Generate expressions for each missing field
            let old_field_list = field_list_parent.record_expr_field_list()?;
            let new_field_list = old_field_list.clone_for_update();
            for (f, ty) in missing_fields.iter() {
                let field_expr = generate_fill_expr(ty);
                let field = make::record_expr_field(
                    make::name_ref(&f.name(ctx.sema.db).display_no_db(ctx.edition).to_smolstr()),
                    field_expr,
                );
                new_field_list.add_field(field.clone_for_update());
            }
            build_text_edit(new_field_list.syntax(), old_field_list.syntax())
        }
        Either::Right(field_list_parent) => { /* pattern case */ }
    }
}
```

**Why This Matters for Contributors:**
- **Optional Fixes**: Return `None` if no fix available
- **Syntax Trees**: Use `clone_for_update()` pattern for mutable syntax manipulation
- **Macro Awareness**: Different strategies for macro vs. normal code
- **Diff-Based Edits**: Use `diff()` for minimal changes in non-macro code
- **Type-Aware Generation**: Generate appropriate expressions based on field types

---

### 🔍 Expert Rust Commentary: Pattern 5

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Optional Computation + Mutable Syntax Tree

**Rust-Specific Insight:**
This pattern showcases **rust-analyzer's mutable syntax tree API**, which is unique in the Rust ecosystem. The key insight is `clone_for_update()`:

```rust
let old_field_list = field_list_parent.record_expr_field_list()?;
let new_field_list = old_field_list.clone_for_update();  // Mutable copy
new_field_list.add_field(field.clone_for_update());      // Mutate in place
diff(old_syntax, new_syntax).into_text_edit(&mut builder); // Minimal diff
```

This is **immutable original + mutable clone** pattern: the original tree remains untouched (required for salsa caching), but the clone can be freely mutated. The `diff()` function then computes minimal text edits.

**Macro Handling Subtlety:**
The code has two edit strategies:
1. **Non-macro code**: Use `diff()` for minimal, precise edits
2. **Macro code**: Replace entire node because macro spans are unreliable

This is **defensive programming** for macro contexts where fine-grained edits could corrupt the expansion.

**Contribution Tip:**
When generating fixes:
```rust
fn fixes(ctx: &DiagnosticsContext<'_>, d: &hir::MyDiag) -> Option<Vec<Assist>> {
    // Early return if unfixable
    if some_condition_makes_it_unfixable { return None; }

    // Use type information to validate
    if !expected_ty.could_coerce_to(actual_ty) { return None; }

    // Clone for mutation
    let new_syntax = old_syntax.clone_for_update();

    // Apply changes
    new_syntax.add_something(...);

    // Generate edit
    if d.file.is_macro() {
        builder.replace(range, new_syntax.to_string()); // Full replace
    } else {
        diff(old, new).into_text_edit(&mut builder);    // Minimal diff
    }

    Some(vec![fix("id", "description", source_change, range)])
}
```

**Common Pitfalls:**
- **Mutating original tree**: Causes salsa cache invalidation. Always use `clone_for_update()`.
- **Not checking macro context**: Fine-grained edits in macros produce broken code
- **Type-unsafe generation**: Use semantic info (`ctx.sema.type_of_expr()`) to generate correct expressions
- **Empty Vec instead of None**: Return `None` if no fixes, not `Some(vec![])`

**Related Patterns in Ecosystem:**
- **rustc's suggestion API**: Uses span-based text replacement, less sophisticated
- **Clippy's `span_lint_and_sugg()`**: Similar but doesn't use syntax trees, just text ranges
- **IntelliJ's intention actions**: Uses similar mutable PSI trees

**Performance Insight:**
The `diff()` algorithm is critical for editor responsiveness. It uses a tree-diff algorithm (similar to React's virtual DOM diffing) to find minimal changes, reducing the amount of text the editor must re-lex and re-parse.

**Type-Aware Expression Generation:**
The `generate_fill_expr(ty)` function is remarkable—it generates appropriate default values based on type:
- `bool` → `false`
- `Option<T>` → `None`
- `Vec<T>` → `vec![]`
- `String` → `String::new()`
- Custom types → `Default::default()` or `todo!()`

This is **type-directed code generation**, a powerful technique for assists.

---

## Pattern 6: Lint Attribute Handling Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 600-738)
**Category:** Severity Calculation

**Code Example:**
```rust
fn handle_lints(
    sema: &Semantics<'_, RootDatabase>,
    file_id: FileId,
    krate: hir::Crate,
    diagnostics: &mut [(InFile<SyntaxNode>, &mut Diagnostic)],
    edition: Edition,
) {
    for (node, diag) in diagnostics {
        let lint = match diag.code {
            DiagnosticCode::RustcLint(lint) => RUSTC_LINTS[lint].lint,
            DiagnosticCode::Clippy(lint) => CLIPPY_LINTS[lint].lint,
            _ => panic!("non-lint passed to `handle_lints()`"),
        };

        let default_severity = default_lint_severity(lint, edition);
        if !(default_severity == Severity::Allow && diag.severity == Severity::WeakWarning) {
            diag.severity = default_severity;
        }

        let mut diag_severity = lint_severity_at(sema, file_id, krate, node, &lint_groups(&diag.code, edition));

        if let outline_diag_severity @ Some(_) =
            find_outline_mod_lint_severity(sema, file_id, krate, node, diag, edition)
        {
            diag_severity = outline_diag_severity;
        }

        if let Some(diag_severity) = diag_severity {
            diag.severity = diag_severity;
        }
    }
}

fn lint_severity_at(
    sema: &Semantics<'_, RootDatabase>,
    file_id: FileId,
    krate: hir::Crate,
    node: &InFile<SyntaxNode>,
    lint_groups: &LintGroups,
) -> Option<Severity> {
    node.value
        .ancestors()
        .filter_map(ast::AnyHasAttrs::cast)
        .find_map(|ancestor| {
            lint_attrs(sema, file_id, krate, ancestor)
                .find_map(|(lint, severity)| lint_groups.contains(&lint).then_some(severity))
        })
        .or_else(|| {
            lint_severity_at(sema, file_id, krate, &sema.find_parent_file(node.file_id)?, lint_groups)
        })
}

fn lint_attrs(
    sema: &Semantics<'_, RootDatabase>,
    file_id: FileId,
    krate: hir::Crate,
    ancestor: ast::AnyHasAttrs,
) -> impl Iterator<Item = (SmolStr, Severity)> {
    sema.lint_attrs(file_id, krate, ancestor).rev().map(|(lint_attr, lint)| {
        let severity = match lint_attr {
            hir::LintAttr::Allow | hir::LintAttr::Expect => Severity::Allow,
            hir::LintAttr::Warn => Severity::Warning,
            hir::LintAttr::Deny | hir::LintAttr::Forbid => Severity::Error,
        };
        (lint, severity)
    })
}
```

**Why This Matters for Contributors:**
- **Attribute Hierarchy**: Walk syntax tree ancestors to find closest `#[allow]`/`#[warn]`/`#[deny]`
- **Edition-Dependent Defaults**: Lint severity can change based on Rust edition
- **Lint Groups**: Support `warnings`, `nonstandard_style`, `clippy::all` etc.
- **Outline Modules**: Special handling for module-level attributes in separate files
- **Recursive Resolution**: Traverse file boundaries for module attributes

---

### 🔍 Expert Rust Commentary: Pattern 6

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Tree-Walking Scope Resolution + Attribute Hierarchy

**Rust-Specific Insight:**
This is one of the most sophisticated patterns in rust-analyzer. It implements **Rust's lint attribute scoping rules** accurately:

1. **Ancestor Walking**: `node.value.ancestors().filter_map(ast::AnyHasAttrs::cast)` finds all attribute-bearing ancestors
2. **Closest Wins**: `find_map()` returns the first (closest) attribute that matches the lint
3. **File Boundaries**: Recursively climbs to parent modules via `sema.find_parent_file()`
4. **Lint Groups**: Handles `#[allow(warnings)]` affecting multiple lints

**Critical Detail**: The attributes are processed in **reverse** (`rev()`) because `sema.lint_attrs()` returns outer-to-inner order, but inner attributes take precedence.

**Edition-Dependent Behavior:**
```rust
let default_severity = default_lint_severity(lint, edition);
```
This is crucial: some lints change default severity across editions. For example:
- `bare_trait_objects`: warn in 2021+, allow in 2018
- `ellipsis_inclusive_range_patterns`: warn in 2021+, allow in 2018

**Contribution Tip:**
Understanding lint severity resolution:
```rust
// Priority order (highest to lowest):
1. #[forbid(lint)]     -> Severity::Error (cannot be overridden)
2. #[deny(lint)]       -> Severity::Error
3. #[warn(lint)]       -> Severity::Warning
4. #[allow(lint)]      -> Severity::Allow
5. Edition default     -> Varies by lint and edition
6. Global default      -> Typically Allow or Warning
```

**Outline Module Handling:**
The `find_outline_mod_lint_severity()` call is subtle: when a module is in a separate file (`mod foo;`), the module-level attributes in `foo.rs` affect diagnostics in that file. This requires special handling because the file tree and semantic tree don't align.

**Common Pitfalls:**
- **Forgetting lint groups**: `#[allow(warnings)]` should suppress all warnings, not just individual lints
- **Edition handling**: Not checking edition can cause incorrect default severities
- **Forbid vs Deny**: `#[forbid]` cannot be overridden by inner attributes, `#[deny]` can
- **Expect attributes**: `#[expect(lint)]` is special—it allows the lint but warns if it's not triggered

**Related Patterns in Ecosystem:**
- **rustc's lint levels**: Similar but uses a `LintLevelMap` for efficiency
- **Clippy's lint configuration**: Uses TOML files instead of attributes for project-wide settings
- **ESLint's inline config**: Similar `/* eslint-disable */` comments but less structured

**Performance Consideration:**
Walking ancestors for every diagnostic seems expensive, but:
1. Most diagnostics don't have attribute overrides (fast path)
2. Ancestor iteration is lazy and usually stops at the first match
3. The `lint_attrs()` query is salsa-cached

**Architectural Insight:**
This pattern demonstrates **policy separation**: the core diagnostic system doesn't know about lint attributes—this layer applies policy (severity adjustment) after diagnostics are generated. This keeps the diagnostic handlers simple and focuses complexity in one place.

---

## Pattern 7: Experimental Diagnostic Gating
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 186-204, 474-477)
**Category:** Feature Gating

**Code Example:**
```rust
impl Diagnostic {
    fn new(...) -> Diagnostic {
        Diagnostic {
            code,
            message: message.into(),
            range: range.into(),
            severity: /* ... */,
            unused: false,
            experimental: true,  // New diagnostics are experimental by default
            fixes: None,
            main_node: None,
        }
    }

    fn stable(mut self) -> Diagnostic {
        self.experimental = false;
        self
    }
}

// Filter out experimental diagnostics if disabled in config
res.retain(|d| {
    !(ctx.config.disabled.contains(d.code.as_str())
        || ctx.config.disable_experimental && d.experimental)
});

// Usage in handler:
Diagnostic::new_with_syntax_node_ptr(ctx, code, message, node)
    .stable()  // Mark as stable
    .with_fixes(fixes)
```

**Why This Matters for Contributors:**
- **Safe Rollout**: New diagnostics start experimental, promoted to stable after testing
- **User Control**: Users can disable experimental diagnostics via config
- **Default Safe**: All diagnostics are experimental unless explicitly marked `.stable()`
- **Per-Diagnostic Gating**: Each diagnostic individually marked, not feature-flag based

---

### 🔍 Expert Rust Commentary: Pattern 7

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Feature Gating + Progressive Disclosure

**Rust-Specific Insight:**
This pattern implements **fail-safe defaults** with opt-in stability—a critical pattern for evolving diagnostics in a production tool. The design philosophy:

1. **New diagnostics are experimental by default** (`experimental: true` in constructor)
2. **Must explicitly call `.stable()`** to promote to stable
3. **Users control exposure** via `disable_experimental` config
4. **No feature flags needed** per-diagnostic gating instead of compile-time features

This is superior to feature flags because:
- **Fine-grained control**: Users can disable specific diagnostics, not categories
- **Runtime flexibility**: No recompilation needed to toggle diagnostics
- **A/B testing**: Can ship experimental features to subset of users

**Critical Safety Property:**
```rust
res.retain(|d| {
    !(ctx.config.disabled.contains(d.code.as_str())
        || ctx.config.disable_experimental && d.experimental)
});
```
This filter runs **after** handlers run, meaning experimental diagnostics still execute their logic (warming caches) but are simply not shown. This enables gradual testing without changing user experience.

**Contribution Tip:**
Lifecycle of a new diagnostic:
```rust
// Phase 1: Initial implementation (experimental, may have false positives)
Diagnostic::new(code, message, range)
    .with_fixes(fixes)
    // .experimental is true by default

// Phase 2: After testing, promote to stable
Diagnostic::new(code, message, range)
    .stable()  // Users with disable_experimental still see it
    .with_fixes(fixes)
```

The `.stable()` call should happen after:
1. Manual testing across diverse codebases
2. Dogfooding on rust-analyzer itself
3. Community feedback from experimental users
4. False positive rate is acceptably low

**Common Pitfalls:**
- **Premature `.stable()`**: Marking diagnostic stable too early leads to user complaints
- **Forgetting to remove `.stable()`**: If testing reveals issues, remove `.stable()` in a patch
- **Config check in handler**: Don't check `ctx.config.disable_experimental` in handlers—the filter handles it
- **Empty diagnostics**: Experimental diagnostics that never fire are dead weight; remove or fix them

**Related Patterns in Ecosystem:**
- **rustc's `-Z` flags**: Unstable features gated by compiler flags (more coarse-grained)
- **Clippy's lint levels**: No experimental concept, all lints are stable or removed
- **TypeScript's `--strict` flag**: Similar progressive disclosure but less granular

**User Experience Insight:**
The pattern respects the **principle of least surprise**: new rust-analyzer releases won't suddenly spam users with new diagnostics. Users opt-in to experimental features explicitly, making the tool feel stable and reliable.

**Telemetry Opportunity:**
Experimental diagnostics could be instrumented to report:
- How often they fire
- False positive rate (user dismisses immediately)
- Fix acceptance rate

This data would inform stabilization decisions.

**Design Tradeoff:**
The current approach (runtime flag) means diagnostic code always runs, even if filtered. An alternative would be compile-time features, but that would require separate builds for experimental/stable, which is impractical for an IDE.

---

## Pattern 8: Handler Module Structure Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 31-86)
**Category:** Code Organization

**Code Example:**
```rust
mod handlers {
    pub(crate) mod await_outside_of_async;
    pub(crate) mod bad_rtn;
    pub(crate) mod break_outside_of_loop;
    // ... 40+ handlers, one per diagnostic type
    pub(crate) mod type_mismatch;
    pub(crate) mod typed_hole;
    pub(crate) mod unused_variables;

    // The handlers below are unusual, they implement the diagnostics as well.
    pub(crate) mod field_shorthand;
    pub(crate) mod json_is_not_rust;
    pub(crate) mod unlinked_file;
    pub(crate) mod useless_braces;
}

// Each handler module follows this structure:
pub(crate) fn type_mismatch(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::TypeMismatch<'_>,
) -> Diagnostic {
    // ... diagnostic creation logic
}

fn fixes(ctx: &DiagnosticsContext<'_>, d: &hir::TypeMismatch<'_>) -> Option<Vec<Assist>> {
    // ... fix generation logic
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;
    // ... test cases
}
```

**Why This Matters for Contributors:**
- **One File Per Diagnostic**: Each diagnostic gets its own module
- **Standard Interface**: Public handler function + private `fixes()` helper
- **Co-Located Tests**: Tests in same file as implementation
- **Two Categories**: Most handlers consume HIR diagnostics; some (field_shorthand) analyze AST directly
- **Documentation Comments**: Each handler has `// Diagnostic: diagnostic-name` comment

---

### 🔍 Expert Rust Commentary: Pattern 8

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Module-per-Feature + Convention-over-Configuration

**Rust-Specific Insight:**
This organizational pattern leverages Rust's module system for **physical isolation with logical cohesion**. Each diagnostic gets its own file, enforcing:

1. **Single Responsibility**: One file = one diagnostic type
2. **Co-located Tests**: Tests live with implementation, reducing cognitive distance
3. **Standard Interface**: All handlers follow `pub(crate) fn handler(ctx, diag) -> Diagnostic`
4. **Private Helpers**: `fixes()` is module-private, keeping the public API clean

**Rust Module System Advantage:**
Unlike languages with header files, Rust modules naturally enforce encapsulation:
```rust
mod handlers {
    pub(crate) mod await_outside_of_async;  // Only lib.rs can import
}

// In await_outside_of_async.rs:
pub(crate) fn await_outside_of_async(...) { ... }  // Visible to parent
fn fixes(...) { ... }                               // Module-private
```

The `pub(crate)` visibility is perfect here: handlers are internal implementation details but need to be called from `lib.rs`.

**Two Categories of Handlers:**
The comment highlights an important distinction:
- **HIR-based handlers** (majority): Consume diagnostics from semantic analysis (`hir` crate)
- **AST-based handlers** (minority): Directly analyze syntax (e.g., `field_shorthand`, `useless_braces`)

AST-based handlers are used for style lints that don't require type information.

**Contribution Tip:**
Creating a new handler:
```rust
// 1. Create handlers/my_diagnostic.rs
pub(crate) fn my_diagnostic(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::MyDiagnostic,
) -> Diagnostic {
    Diagnostic::new(code, message, range)
        .with_fixes(fixes(ctx, d))
}

fn fixes(ctx: &DiagnosticsContext<'_>, d: &hir::MyDiagnostic) -> Option<Vec<Assist>> {
    // Fix generation logic
    Some(vec![...])
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn test_my_diagnostic() {
        check_diagnostics(r#"
            fn foo() {
                // code that triggers diagnostic
                //^^^ error: my diagnostic message
            }
        "#);
    }
}

// 2. Add to lib.rs:
mod handlers {
    // ...
    pub(crate) mod my_diagnostic;
}

// 3. Add to dispatch match:
AnyDiagnostic::MyDiagnostic(d) => handlers::my_diagnostic::my_diagnostic(&ctx, &d),
```

**Common Pitfalls:**
- **Exposing internal helpers**: Keep `fixes()` and other helpers private (no `pub` keyword)
- **Cross-handler dependencies**: Handlers should not call each other; if sharing logic, extract to `lib.rs` or a `utils` module
- **Missing diagnostic comment**: Each handler file should have `// Diagnostic: diagnostic-name` at the top for documentation
- **Tests in wrong location**: Tests must be in the same file as the handler for clarity

**Related Patterns in Ecosystem:**
- **rustc's error codes**: Each error has a dedicated markdown file in error_codes directory
- **Clippy's lint modules**: Similar one-file-per-lint structure
- **IntelliJ inspections**: Each inspection is a Java class, but tests are often separate

**Architectural Benefit:**
This structure scales well to 100+ diagnostics because:
1. **No merge conflicts**: Different diagnostics edited in different files
2. **Easy to navigate**: File name matches diagnostic name
3. **Self-documenting**: The module list in `lib.rs` is a diagnostic catalog
4. **Easy to remove**: Deleting a diagnostic means deleting one file and one match arm

**Testing Philosophy:**
Co-located tests mean you never lose track of test coverage. If a handler exists, its tests are right there—no hunting through a separate `tests/` directory.

---

## Pattern 9: Display Range Adjustment Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 758-769)
**Category:** User Experience

**Code Example:**
```rust
fn adjusted_display_range<N: AstNode>(
    ctx: &DiagnosticsContext<'_>,
    diag_ptr: InFile<AstPtr<N>>,
    adj: &dyn Fn(N) -> Option<TextRange>,
) -> FileRange {
    let source_file = ctx.sema.parse_or_expand(diag_ptr.file_id);
    let node = diag_ptr.value.to_node(&source_file);
    let hir::FileRange { file_id, range } = diag_ptr
        .with_value(adj(node).unwrap_or_else(|| diag_ptr.value.text_range()))
        .original_node_file_range_rooted(ctx.sema.db);
    ide_db::FileRange { file_id: file_id.file_id(ctx.sema.db), range }
}

// Usage in type_mismatch handler:
let display_range = adjusted_display_range(ctx, d.expr_or_pat, &|node| {
    let Either::Left(expr) = node else { return None };
    let salient_token_range = match expr {
        ast::Expr::IfExpr(it) => it.if_token()?.text_range(),
        ast::Expr::LoopExpr(it) => it.loop_token()?.text_range(),
        ast::Expr::ForExpr(it) => it.for_token()?.text_range(),
        ast::Expr::WhileExpr(it) => it.while_token()?.text_range(),
        ast::Expr::BlockExpr(it) => it.stmt_list()?.r_curly_token()?.text_range(),
        ast::Expr::MatchExpr(it) => it.match_token()?.text_range(),
        ast::Expr::MethodCallExpr(it) => it.name_ref()?.ident_token()?.text_range(),
        ast::Expr::FieldExpr(it) => it.name_ref()?.ident_token()?.text_range(),
        _ => return None,
    };
    Some(salient_token_range)
});
```

**Why This Matters for Contributors:**
- **Precise Highlighting**: Instead of underlining entire `if { ... }`, highlight just `if` keyword
- **Closure Customization**: Each handler provides custom range adjustment logic
- **Fallback Default**: If adjustment returns `None`, use full node range
- **Macro Mapping**: `original_node_file_range_rooted()` maps expanded code back to source
- **Better UX**: Small, specific highlights are easier to understand than large ranges

---

### 🔍 Expert Rust Commentary: Pattern 9

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Range Adjustment + UX Optimization

**Rust-Specific Insight:**
This pattern addresses a critical UX problem: **generic diagnostics on large syntax nodes are confusing**. Consider:

```rust
// Without adjustment: entire expression highlighted
let x: u32 = if condition { "string" } else { "other" };
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// With adjustment: just the 'if' keyword highlighted
let x: u32 = if condition { "string" } else { "other" };
//           ^^
```

The `adjusted_display_range` function takes a **closure** that returns the "salient token" for each expression type. This is **data-driven UX tuning**—different expression types have different salient ranges.

**Macro Mapping Subtlety:**
The `original_node_file_range_rooted()` call is critical for macros:
```rust
macro_rules! make_fn {
    () => { fn foo() -> u32 { "string" } };
}
make_fn!(); // Macro invocation

// Without mapping: diagnostic points into expansion (invisible to user)
// With mapping: diagnostic points to macro invocation site
```

**Contribution Tip:**
When adding salient token logic:
```rust
let display_range = adjusted_display_range(ctx, ptr, &|node| {
    let expr = node.as_expr()?;
    Some(match expr {
        // Highlight the keyword for control flow
        ast::Expr::IfExpr(it) => it.if_token()?.text_range(),
        ast::Expr::WhileExpr(it) => it.while_token()?.text_range(),
        ast::Expr::LoopExpr(it) => it.loop_token()?.text_range(),

        // Highlight the operator for operations
        ast::Expr::BinExpr(it) => it.op_token()?.text_range(),

        // Highlight the callee for calls
        ast::Expr::CallExpr(it) => it.expr()?.syntax().text_range(),
        ast::Expr::MethodCallExpr(it) => it.name_ref()?.text_range(),

        // Fallback to full range for complex expressions
        _ => return None,
    })
});
```

**Fallback Behavior:**
If the closure returns `None`, the full node range is used. This is important for:
1. **Unknown expression types**: New syntax forms don't break diagnostics
2. **Missing tokens**: Incomplete/invalid syntax gracefully falls back
3. **Literal expressions**: Highlighting the entire `"string"` is fine

**Common Pitfalls:**
- **Too aggressive highlighting**: Highlighting just one token can be confusing if it's not obvious what's wrong
- **Missing fallback**: Always return `None` for unknown cases, don't panic
- **Inconsistent choices**: Be consistent across similar expression types (all loops highlight keywords)
- **Macro positions**: Don't assume tokens exist; macro expansions can have weird spans

**Related Patterns in Ecosystem:**
- **rustc's diagnostic spans**: Uses primary/secondary spans but doesn't adjust within expressions
- **Clippy's span selection**: More aggressive highlighting (often the entire problematic expression)
- **IntelliJ's error highlighting**: Similar salient token approach for Java

**Cognitive Science Insight:**
Research shows that **small, precise highlights** are easier to understand than large ones because:
1. **Faster eye movement**: User's eyes go directly to the problem
2. **Less visual noise**: Doesn't obscure surrounding code
3. **Clearer causality**: Highlighting `if` for type mismatch suggests "the if expression has the wrong type"

**Performance Note:**
The closure is called once per diagnostic, and token extraction is cheap (just following pointers in the syntax tree). This optimization has negligible cost but significant UX benefit.

**Design Tradeoff:**
The pattern couples diagnostic presentation to AST structure. If AST structure changes, salient token logic must be updated. But this is acceptable because AST is stable and the coupling is localized to one function.

---

## Pattern 10: Macro-Aware Diagnostic Filtering
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/lib.rs (lines 525-557)
**Category:** Macro Handling

**Code Example:**
```rust
fn handle_diag_from_macros(
    sema: &Semantics<'_, RootDatabase>,
    diag: &mut Diagnostic,
    node: &InFile<SyntaxNode>,
) -> bool {
    let Some(macro_file) = node.file_id.macro_file() else { return true };
    let span_map = sema.db.expansion_span_map(macro_file);
    let mut spans = span_map.spans_for_range(node.text_range());

    if spans.any(|span| {
        span.ctx.outer_expn(sema.db).is_some_and(|expansion| {
            let macro_call = sema.db.lookup_intern_macro_call(expansion.into());
            // Don't show diagnostics for non-local macros
            !Crate::from(macro_call.def.krate).origin(sema.db).is_local()
                || !macro_call.def.kind.is_declarative()
        })
    }) {
        // Disable suggestions for external macros
        diag.fixes = None;

        // Only report certain lints in external macros
        if let DiagnosticCode::RustcLint(lint) = diag.code
            && !LINTS_TO_REPORT_IN_EXTERNAL_MACROS.contains(lint)
        {
            return false;
        }
    };
    true
}

// Usage:
res.retain_mut(|diag| {
    if let Some(node) = diag.main_node
        .map(|ptr| ptr.map(|node| node.to_node(&ctx.sema.parse_or_expand(ptr.file_id))))
    {
        handle_diag_from_macros(&ctx.sema, diag, &node)
    } else {
        true
    }
});
```

**Why This Matters for Contributors:**
- **External Macro Suppression**: Don't report most lints in external library macros
- **Fix Disabling**: Never suggest edits to external macro code
- **Local vs. External**: Distinguish workspace macros from dependency macros
- **Declarative vs. Proc**: Different rules for `macro_rules!` vs. proc macros
- **Whitelist Approach**: Some lints (empty set currently) allowed in external macros

---

### 🔍 Expert Rust Commentary: Pattern 10

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Macro Hygiene + External Code Filtering

**Rust-Specific Insight:**
This pattern implements a critical policy: **don't report most lints in code you don't control**. The reasoning is subtle but important:

1. **External macros** (from dependencies): User can't fix them, diagnostics are noise
2. **Local macros** (in workspace): User owns them, diagnostics are helpful
3. **Procedural macros**: Generated code is often intentionally non-idiomatic

**Macro Origin Detection:**
```rust
let macro_call = sema.db.lookup_intern_macro_call(expansion.into());
!Crate::from(macro_call.def.krate).origin(sema.db).is_local()
    || !macro_call.def.kind.is_declarative()
```

This checks:
- **Crate origin**: Is the macro defined in the local workspace?
- **Macro kind**: Is it a `macro_rules!` (declarative) or proc macro?

**Fix Disabling:**
```rust
if some_external_macro {
    diag.fixes = None;  // Disable all quick fixes
}
```

This is critical: suggesting edits to external macro expansions would:
1. **Fail to apply**: Edits target the expansion, which isn't in source
2. **Confuse users**: "Apply fix" does nothing, frustrating UX
3. **Break builds**: Edits to generated code get overwritten on next build

**Whitelist Approach:**
```rust
if let DiagnosticCode::RustcLint(lint) = diag.code
    && !LINTS_TO_REPORT_IN_EXTERNAL_MACROS.contains(lint)
{
    return false;  // Suppress diagnostic
}
```

Currently `LINTS_TO_REPORT_IN_EXTERNAL_MACROS` is empty, but the infrastructure exists for exceptions. Potential candidates:
- `unused_must_use`: Still relevant in external macros
- `deprecated`: User needs to know about deprecated calls, even in macros

**Contribution Tip:**
When deciding if a diagnostic should appear in external macros:
```rust
// Questions to ask:
1. Can the user fix it? (If no, suppress)
2. Is it about the macro invocation, not the expansion? (If yes, show)
3. Does it indicate a real bug the user needs to know about? (If yes, show)
4. Is it a style lint? (If yes, suppress)

// Examples:
- "unused variable" in external macro → suppress (macro author's problem)
- "type mismatch" at macro call site → show (user's arguments wrong)
- "missing field" in macro expansion → suppress (macro needs fixing)
```

**Common Pitfalls:**
- **Assuming all macro code is external**: Must check `is_local()` to distinguish workspace macros
- **Suppressing errors**: Only suppress *lints* in external macros, not hard errors
- **Not setting `main_node`**: Filtering requires the diagnostic to have `main_node` set
- **Proc macro assumptions**: Declarative and proc macros need different handling

**Related Patterns in Ecosystem:**
- **rustc's `#[allow_internal_unsafe]`**: Allows unsafe in macro expansions, similar trust model
- **Clippy's macro handling**: Also suppresses lints in external macros
- **ESLint's generated code**: Uses `/* eslint-disable */` comments, less automatic

**Edge Cases:**
1. **Macro calls in macro expansions**: The code walks the expansion tree to find the original macro
2. **Nested workspace macros**: Local macros calling local macros should still report diagnostics
3. **Mixed content**: If a macro expansion contains both user code and generated code, diagnostics may be noisy

**Design Insight:**
This is an example of **context-aware analysis**: the same diagnostic is shown or hidden based on code provenance, not just content. This respects the **principle of actionable feedback**—only show diagnostics the user can act on.

**Future Enhancement:**
Could add a config option: `show_diagnostics_in_external_macros: bool` for users debugging library macros. Most users would keep this false, but library developers might enable it.

---

## Pattern 11: Type-Mismatch Fix Strategies
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/handlers/type_mismatch.rs (lines 59-326)
**Category:** Complex Fix Generation

**Code Example:**
```rust
fn fixes(ctx: &DiagnosticsContext<'_>, d: &hir::TypeMismatch<'_>) -> Option<Vec<Assist>> {
    let mut fixes = Vec::new();

    if let Some(expr_ptr) = d.expr_or_pat.value.cast::<ast::Expr>() {
        let expr_ptr = &InFile { file_id: d.expr_or_pat.file_id, value: expr_ptr };
        add_reference(ctx, d, expr_ptr, &mut fixes);
        add_missing_ok_or_some(ctx, d, expr_ptr, &mut fixes);
        remove_unnecessary_wrapper(ctx, d, expr_ptr, &mut fixes);
        remove_semicolon(ctx, d, expr_ptr, &mut fixes);
        str_ref_to_owned(ctx, d, expr_ptr, &mut fixes);
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn add_reference(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::TypeMismatch<'_>,
    expr_ptr: &InFile<AstPtr<ast::Expr>>,
    acc: &mut Vec<Assist>,
) -> Option<()> {
    let range = ctx.sema.diagnostics_display_range((*expr_ptr).map(|it| it.into()));

    let (_, mutability) = d.expected.as_reference()?;
    let actual_with_ref = d.actual.add_reference(mutability);
    if !actual_with_ref.could_coerce_to(ctx.sema.db, &d.expected) {
        return None;
    }

    let ampersands = format!("&{}", mutability.as_keyword_for_ref());
    let edit = TextEdit::insert(range.range.start(), ampersands);
    let source_change = SourceChange::from_text_edit(range.file_id, edit);
    acc.push(fix("add_reference_here", "Add reference here", source_change, range.range));
    Some(())
}

fn add_missing_ok_or_some(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::TypeMismatch<'_>,
    expr_ptr: &InFile<AstPtr<ast::Expr>>,
    acc: &mut Vec<Assist>,
) -> Option<()> {
    let expected_adt = d.expected.as_adt()?;
    let expected_enum = expected_adt.as_enum()?;

    let famous_defs = FamousDefs(&ctx.sema, scope.krate());
    if Some(expected_enum) != core_result && Some(expected_enum) != core_option {
        return None;
    }

    let variant_name = if Some(expected_enum) == core_result { "Ok" } else { "Some" };
    let wrapped_actual_ty = expected_adt.ty_with_args(ctx.sema.db, std::iter::once(d.actual.clone()));

    if !d.expected.could_unify_with(ctx.sema.db, &wrapped_actual_ty) {
        return None;
    }

    let mut builder = TextEdit::builder();
    builder.insert(expr.syntax().text_range().start(), format!("{variant_name}("));
    builder.insert(expr.syntax().text_range().end(), ")".to_owned());
    // ...
}
```

**Why This Matters for Contributors:**
- **Multiple Strategies**: Try different fix approaches for same diagnostic
- **Type-Guided Fixes**: Use semantic type information to determine applicability
- **Accumulator Pattern**: Each strategy pushes to shared `Vec<Assist>`
- **Early Returns**: Use `Option<()>` to bail if strategy doesn't apply
- **Standard Library Detection**: `FamousDefs` identifies `Result`/`Option` types

---

### 🔍 Expert Rust Commentary: Pattern 11

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Strategy Accumulation + Type-Guided Fixing

**Rust-Specific Insight:**
This is the most sophisticated fix generation pattern in rust-analyzer. It implements **multiple independent fix strategies** for a single diagnostic, each using semantic type information to determine applicability:

```rust
fn fixes(...) -> Option<Vec<Assist>> {
    let mut fixes = Vec::new();

    // Try all strategies
    add_reference(ctx, d, expr_ptr, &mut fixes);
    add_missing_ok_or_some(ctx, d, expr_ptr, &mut fixes);
    remove_unnecessary_wrapper(ctx, d, expr_ptr, &mut fixes);
    str_ref_to_owned(ctx, d, expr_ptr, &mut fixes);

    if fixes.is_empty() { None } else { Some(fixes) }
}
```

Each strategy:
1. **Checks applicability** using type information
2. **Returns early** (`Option<()>`) if not applicable
3. **Pushes to accumulator** if successful

**Type-Driven Applicability:**
```rust
fn add_reference(...) -> Option<()> {
    let (_, mutability) = d.expected.as_reference()?;  // Must expect reference
    let actual_with_ref = d.actual.add_reference(mutability);
    if !actual_with_ref.could_coerce_to(ctx.sema.db, &d.expected) {
        return None;  // Adding reference wouldn't fix it
    }
    // Generate the fix
}
```

This uses **hypothetical type checking**: "If I added a `&`, would the type check?" This is far more sophisticated than pattern matching on AST.

**Famous Defs Pattern:**
```rust
let famous_defs = FamousDefs(&ctx.sema, scope.krate());
if Some(expected_enum) == core_result && Some(expected_enum) == core_option {
    // Special handling for Result/Option
}
```

`FamousDefs` is a utility for identifying standard library types. This is critical because:
- Can't rely on name matching (`Result` could be shadowed)
- Must use semantic identity (DefId comparison)
- Handles re-exports (`std::result::Result` vs `core::result::Result`)

**Contribution Tip:**
Adding a new fix strategy:
```rust
fn my_fix_strategy(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::TypeMismatch<'_>,
    expr_ptr: &InFile<AstPtr<ast::Expr>>,
    acc: &mut Vec<Assist>,
) -> Option<()> {
    // 1. Check if strategy applies
    let expected_ty = d.expected.some_property()?;

    // 2. Use type system to validate
    let hypothetical = d.actual.apply_transformation();
    if !hypothetical.could_coerce_to(ctx.sema.db, &d.expected) {
        return None;
    }

    // 3. Generate edit
    let range = ctx.sema.diagnostics_display_range(*expr_ptr);
    let edit = TextEdit::insert(range.start(), "prefix");
    let source_change = SourceChange::from_text_edit(range.file_id, edit);

    // 4. Add to accumulator
    acc.push(fix("strategy_id", "Fix description", source_change, range.range));

    Some(())  // Signal success
}
```

**Common Pitfalls:**
- **Not checking coercion**: Must verify the fix actually solves the type error
- **Wrong accumulator pattern**: Return `Option<()>`, not `Option<Assist>`. Push to `acc`.
- **Missing early returns**: Check applicability early to avoid wasted computation
- **Forgetting `None` fallback**: If all strategies fail, return `None` not `Some(vec![])`

**Related Patterns in Ecosystem:**
- **rustc's suggestions**: Multiple suggestions shown sequentially, not all at once
- **IntelliJ's intention actions**: Similar multi-strategy approach
- **Clippy's `span_lint_and_sugg()`**: Usually single fix, less sophisticated

**Type System Integration:**
The key methods:
- `could_coerce_to()`: Can type A coerce to type B?
- `could_unify_with()`: Can types A and B unify (bidirectional)?
- `as_reference()`: Is this a reference type? What mutability?
- `as_adt()`: Is this an ADT (struct/enum/union)?

These are **semantic predicates**, not syntactic—they work through type aliases, infer types, etc.

**UX Consideration:**
Showing multiple fixes is powerful but can overwhelm:
```rust
// User sees:
💡 Add reference here (&)
💡 Wrap in Ok()
💡 Call .to_string()
```

Strategies are ordered by likelihood. The first fix is usually the right one, but having options is valuable.

**Performance Note:**
The type queries (`could_coerce_to`, etc.) are **salsa-cached**, so trying multiple strategies is cheap if the same types are queried.

---

## Pattern 12: Syntax-Based Diagnostic Handlers
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/handlers/field_shorthand.rs
**Category:** AST-Level Diagnostics

**Code Example:**
```rust
pub(crate) fn field_shorthand(
    db: &RootDatabase,
    acc: &mut Vec<Diagnostic>,
    file_id: EditionedFileId,
    node: &SyntaxNode,
) {
    match_ast! {
        match node {
            ast::RecordExpr(it) => check_expr_field_shorthand(db, acc, file_id, it),
            ast::RecordPat(it) => check_pat_field_shorthand(db, acc, file_id, it),
            _ => ()
        }
    };
}

fn check_expr_field_shorthand(
    db: &RootDatabase,
    acc: &mut Vec<Diagnostic>,
    file_id: EditionedFileId,
    record_expr: ast::RecordExpr,
) {
    let record_field_list = match record_expr.record_expr_field_list() {
        Some(it) => it,
        None => return,
    };
    for record_field in record_field_list.fields() {
        let (name_ref, expr) = match record_field.name_ref().zip(record_field.expr()) {
            Some(it) => it,
            None => continue,
        };

        let field_name = name_ref.syntax().text().to_string();
        let field_expr = expr.syntax().text().to_string();
        if field_name != field_expr || name_ref.as_tuple_field().is_some() {
            continue;
        }

        let mut edit_builder = TextEdit::builder();
        edit_builder.delete(record_field.syntax().text_range());
        edit_builder.insert(record_field.syntax().text_range().start(), field_name);
        let edit = edit_builder.finish();

        acc.push(
            Diagnostic::new(
                DiagnosticCode::Clippy("redundant_field_names"),
                "Shorthand struct initialization",
                FileRange { file_id: vfs_file_id, range: field_range },
            )
            .with_fixes(Some(vec![fix(
                "use_expr_field_shorthand",
                "Use struct shorthand initialization",
                SourceChange::from_text_edit(vfs_file_id, edit),
                field_range,
            )])),
        );
    }
}
```

**Why This Matters for Contributors:**
- **Direct AST Analysis**: Some diagnostics work on syntax, not HIR semantics
- **Visitor Pattern**: Called for each syntax node during file traversal
- **Match AST**: Use `match_ast!` macro to dispatch on node type
- **Direct Accumulation**: Push diagnostics directly to `acc`, not returned
- **Clippy Compatibility**: Implement Clippy-equivalent lints in rust-analyzer

---

### 🔍 Expert Rust Commentary: Pattern 12

**Idiomatic Rating:** ⭐⭐⭐⭐ (4/5)

**Pattern Classification:** Visitor Pattern + Syntax-Only Analysis

**Rust-Specific Insight:**
This pattern represents the **alternative to semantic diagnostics**: analyze syntax directly without type information. This is appropriate for **style lints** where semantics don't matter:

```rust
pub(crate) fn field_shorthand(
    db: &RootDatabase,
    acc: &mut Vec<Diagnostic>,  // Direct accumulation
    file_id: EditionedFileId,
    node: &SyntaxNode,          // Called for every node
) {
    match_ast! {
        match node {
            ast::RecordExpr(it) => check_expr_field_shorthand(...),
            ast::RecordPat(it) => check_pat_field_shorthand(...),
            _ => ()  // Ignore other node types
        }
    }
}
```

**Match AST Macro:**
The `match_ast!` macro is rust-analyzer's DSL for type-safe AST dispatch:
```rust
match_ast! {
    match node {
        ast::Foo(it) => {
            // 'it' is typed as ast::Foo
            it.method();
        },
        ast::Bar(it) => {
            // 'it' is typed as ast::Bar
        },
        _ => ()
    }
}
```

This is safer than manual downcasting because the macro ensures correct type extraction.

**Direct Accumulation:**
Unlike semantic handlers that return `Diagnostic`, these push directly to `acc`:
```rust
acc.push(
    Diagnostic::new(code, message, range)
        .with_fixes(Some(vec![fix(...)]))
);
```

This is the **visitor pattern**: the framework calls the handler for each node, and the handler accumulates diagnostics.

**Contribution Tip:**
When to use syntax-based handlers:
```rust
// Use syntax-based for:
✅ Style lints (field shorthand, useless braces)
✅ Naming conventions (doesn't require type info)
✅ Simple pattern matching (tuple index checks)
✅ Performance (avoid semantic queries for simple checks)

// Use semantic handlers for:
✅ Type errors
✅ Name resolution
✅ Trait bounds
✅ Lifetime issues
```

**Performance Consideration:**
Syntax handlers run on **every syntax node** during file traversal. They must be fast:
```rust
fn check_expr_field_shorthand(...) {
    // Early exit patterns
    let field_list = match record_expr.record_expr_field_list() {
        Some(it) => it,
        None => return,  // No fields, skip
    };

    for record_field in field_list.fields() {
        // Check each field cheaply
        if field_name != field_expr { continue; }
        // ...
    }
}
```

**Common Pitfalls:**
- **Expensive operations**: Don't call semantic queries (type resolution) in syntax handlers
- **Wrong signature**: Must accept `db`, `acc`, `file_id`, `node` exactly
- **Not registered**: Must be called from the syntax visitor in `lib.rs`
- **Side effects**: Don't modify the database or cache state

**Related Patterns in Ecosystem:**
- **Clippy's early lints**: Many Clippy lints are syntax-only for performance
- **rustfmt**: Pure syntax-based analysis
- **syn crate visitors**: Similar visitor pattern for proc macros

**Clippy Equivalence:**
The diagnostic code `DiagnosticCode::Clippy("redundant_field_names")` indicates this implements a Clippy lint natively in rust-analyzer. This provides:
1. **IDE integration**: Inline diagnostics as you type
2. **Quick fixes**: Clippy doesn't provide fixes for all lints
3. **Performance**: No need to run separate Clippy process

**Design Tradeoff:**
Syntax-based handlers are fast but limited:
- **Can't distinguish types**: `{ x: x }` might be intentional if `x` shadows an outer binding
- **No false positive suppression**: Semantic info would allow smarter checks
- **Limited to AST structure**: Can't analyze control flow, data flow, etc.

But for style lints, this tradeoff is acceptable—the rules are purely syntactic.

---

## Pattern 13: Unused Variable Renaming Fix
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/handlers/unused_variables.rs (lines 64-91)
**Category:** Naming Convention Fixes

**Code Example:**
```rust
fn fixes(
    db: &RootDatabase,
    var_name: Name,
    name_range: TextRange,
    diagnostic_range: FileRange,
    is_in_macro: bool,
    is_shorthand_field: bool,
    edition: Edition,
) -> Option<Vec<Assist>> {
    if is_in_macro {
        return None;  // Can't suggest fixes in macros
    }

    let name = var_name.display(db, edition).to_smolstr();
    let name = name.strip_prefix("r#").unwrap_or(&name);

    // Handle struct field shorthand: `field` -> `field: _field`
    let new_name = if is_shorthand_field {
        format!("{name}: _{name}")
    } else {
        format!("_{name}")
    };

    Some(vec![Assist {
        id: AssistId::quick_fix("unscore_unused_variable_name"),
        label: Label::new(format!("Rename unused {name} to {new_name}")),
        group: None,
        target: diagnostic_range.range,
        source_change: Some(SourceChange::from_text_edit(
            diagnostic_range.file_id,
            TextEdit::replace(name_range, new_name),
        )),
        command: None,
    }])
}
```

**Why This Matters for Contributors:**
- **Macro Guard**: Never suggest renames in macro-expanded code
- **Raw Identifier Handling**: Strip `r#` prefix for cleaner suggestions
- **Shorthand Special Case**: `{ field }` becomes `{ field: _field }`, not `{ _field }`
- **Single Edit**: Simple text replacement, no rename refactoring needed
- **Convention Following**: Prefix with `_` is Rust convention for unused variables

---

### 🔍 Expert Rust Commentary: Pattern 13

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Convention-Based Renaming + Shorthand Handling

**Rust-Specific Insight:**
This pattern implements the **Rust naming convention for unused variables**: prefix with underscore. But it has a subtle twist for struct field shorthand:

```rust
// Simple case:
let unused = 42;      // Fix: let _unused = 42;

// Shorthand case:
let Foo { field } = value;      // Fix: let Foo { field: _field } = value;
NOT:
let Foo { _field } = value;     // Wrong! This tries to match a field named '_field'
```

**Shorthand Semantics:**
The fix correctly handles the expansion:
```rust
if is_shorthand_field {
    format!("{name}: _{name}")  // Expand shorthand
} else {
    format!("_{name}")          // Simple prefix
}
```

This demonstrates understanding of **Rust's pattern matching rules**: in shorthand, the identifier binds the variable but matches the field name. Renaming the variable requires expanding to `field: _field`.

**Raw Identifier Handling:**
```rust
let name = var_name.display(db, edition).to_smolstr();
let name = name.strip_prefix("r#").unwrap_or(&name);
```

This handles raw identifiers like `r#match`:
```rust
let r#match = get_match();  // Fix: let _match = get_match();
NOT:
let _r#match = get_match(); // Wrong! Underscore goes after r#, not before
```

**Contribution Tip:**
Understanding the fix applicability:
```rust
if is_in_macro {
    return None;  // CRITICAL: Never suggest renames in macros
}

// Reasons:
1. Macro-expanded variables might be used in other expansions
2. The rename would target the expansion, not the source
3. User can't see or edit the expanded code
```

**Common Pitfalls:**
- **Macro context**: Must check `is_in_macro` first, or fixes break
- **Shorthand detection**: Must detect shorthand to expand properly
- **Raw identifiers**: Must strip `r#` to avoid `_r#name` syntax error
- **Full rename**: This is NOT a full rename refactoring—it's a simple text replacement

**Related Patterns in Ecosystem:**
- **rustc's suggestion**: Similar `_` prefix suggestion but doesn't provide quick fix
- **Clippy's unused warnings**: Reports the issue but delegates fix to rust-analyzer
- **ESLint's no-unused-vars**: JavaScript version but simpler (no shorthand subtlety)

**Why Not Full Rename?**
The pattern uses `TextEdit::replace()` instead of the full rename refactoring (`def.rename()`) because:

1. **Performance**: Simple edit is instant, full rename analyzes all usages
2. **Semantics**: Variable is unused, so no usages to update
3. **Simplicity**: Edge cases (shadowing, captures) don't matter for unused variables

**Edge Case:**
What if the variable becomes used after renaming?
```rust
let _unused = 42;  // Now has underscore
dbg!(_unused);     // Still works! Underscore doesn't make it forbidden
```

This is intentional—the underscore is a *convention* signaling intent, not a semantic restriction (unlike in some languages where `_` discards the value).

**UX Insight:**
The fix message is specific:
```rust
format!("Rename unused {name} to {new_name}")
```

Not just "Rename"—it explains WHY (unused) and shows the BEFORE/AFTER clearly. This helps users understand the convention.

**Design Note:**
The single edit approach means this fix is **always safe to apply**—it can't break code since the variable is by definition unused. This makes it a good candidate for automatic application in batch mode.

---

## Pattern 14: Inactive Code Diagnostic (Conditional Compilation)
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/handlers/inactive_code.rs
**Category:** CFG-Based Diagnostics

**Code Example:**
```rust
pub(crate) fn inactive_code(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::InactiveCode,
) -> Option<Diagnostic> {
    // Don't propagate inactive code diagnostics from within macros
    if d.node.file_id.is_macro() {
        return None;
    }

    let inactive = DnfExpr::new(&d.cfg).why_inactive(&d.opts);
    let mut message = "code is inactive due to #[cfg] directives".to_owned();

    if let Some(inactive) = inactive {
        let inactive_reasons = inactive.to_string();
        if !inactive_reasons.is_empty() {
            format_to!(message, ": {}", inactive);
        }
    }

    let res = Diagnostic::new(
        DiagnosticCode::Ra("inactive-code", Severity::WeakWarning),
        message,
        ctx.sema.diagnostics_display_range(d.node),
    )
    .stable()
    .with_unused(true);  // Grays out the code in editor

    Some(res)
}
```

**Why This Matters for Contributors:**
- **Macro Suppression**: Don't show inactive code diagnostics inside macro expansions
- **Reason Explanation**: Show why code is inactive (e.g., "feature = 'std' is disabled")
- **DNF Expression**: Use Disjunctive Normal Form to simplify cfg conditions
- **Unused Styling**: `.with_unused(true)` causes editor to gray out code
- **Weak Warning**: Non-intrusive severity for informational diagnostic

---

### 🔍 Expert Rust Commentary: Pattern 14

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** CFG Analysis + DNF Simplification

**Rust-Specific Insight:**
This pattern implements **conditional compilation awareness**—one of Rust's most powerful features for cross-platform code. The key insight is using **Disjunctive Normal Form (DNF)** to explain why code is inactive:

```rust
let inactive = DnfExpr::new(&d.cfg).why_inactive(&d.opts);

// Example:
#[cfg(all(unix, not(target_os = "macos")))]
// If on macOS, DNF explains: "unix is true, but target_os = 'macos' conflicts with not(target_os = 'macos')"
```

**DNF Simplification:**
DNF converts complex cfg expressions to readable form:
```rust
// Input: #[cfg(any(all(feature="a", feature="b"), all(feature="c", not(feature="d"))))]
// DNF:   (feature="a" AND feature="b") OR (feature="c" AND NOT feature="d")
// Message: "feature 'a' is disabled" (showing the first failing clause)
```

This is **logical inference for developers**: the diagnostic explains the root cause, not just "this code is inactive."

**Macro Suppression:**
```rust
if d.node.file_id.is_macro() {
    return None;  // Don't show in macro expansions
}
```

Critical: Macro expansions often have platform-specific branches that expand differently on different targets. Showing "inactive code" in expansions would:
1. **Confuse users**: They didn't write the code
2. **Create noise**: Standard library macros are full of `#[cfg]`
3. **Miss the point**: The issue is with the macro invocation args, not the expansion

**Unused Styling:**
```rust
.with_unused(true)  // Grays out the code in editor
```

This uses the same visual treatment as dead code, which is semantically correct: inactive code is by definition unreachable and thus "unused."

**Contribution Tip:**
Understanding CFG diagnostics:
```rust
// Good diagnostic messages:
✅ "code is inactive: feature 'std' is disabled"
✅ "code is inactive: target_os = 'windows' but current target is 'linux'"
✅ "code is inactive: all(unix, target_arch = 'wasm32') is impossible"

// Bad messages:
❌ "code is inactive" (no explanation)
❌ "cfg condition not met" (too vague)
```

The DNF explanation is what makes this diagnostic actionable.

**Common Pitfalls:**
- **Not handling "never" cfgs**: `#[cfg(any())]` is always false, special case it
- **Showing in macro expansions**: Breaks user mental model (they didn't write the code)
- **Wrong severity**: This must be WeakWarning, not Error (code is valid, just disabled)
- **Missing `.with_unused(true)`**: Visual styling is critical for UX

**Related Patterns in Ecosystem:**
- **rustc's dead code analysis**: Similar visual treatment but doesn't explain CFG reasons
- **Clippy's cfg warnings**: Warns about impossible cfgs but less user-friendly
- **C/C++ `#ifdef` graying**: Similar editor feature but no explanation

**Configuration Complexity:**
Rust's `cfg` is Turing-complete (with `cfg_attr`), making analysis hard:
```rust
#[cfg_attr(feature = "a", cfg(feature = "b"))]  // Conditional cfg!
#[cfg(all(not(any(windows, unix)), target_env = "gnu"))]  // Impossible?
```

The DNF simplification is essential for human comprehension of complex conditions.

**UX Insight:**
The diagnostic is **informational, not actionable** in most cases. Users usually don't want to enable the feature—they want to know why the code is grayed out. The message:
```rust
format_to!(message, ": {}", inactive);  // Append explanation
```

Provides context without demanding action.

**Future Enhancement:**
Could add a quick fix to toggle the feature in `Cargo.toml`:
```rust
// Quick fix: "Enable feature 'std' in Cargo.toml"
// Edit: [dependencies]
//       my-crate = { version = "1.0", features = ["std"] }
```

But this is complex because:
1. Features can be in multiple dependency specs
2. Might be a dev/build dependency
3. Might be enabled via workspace inheritance

---

## Pattern 15: Term Search for Typed Holes
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/handlers/typed_hole.rs (lines 44-101)
**Category:** AI-Assisted Code Generation

**Code Example:**
```rust
fn fixes(ctx: &DiagnosticsContext<'_>, d: &hir::TypedHole<'_>) -> Option<Vec<Assist>> {
    let db = ctx.sema.db;
    let root = db.parse_or_expand(d.expr.file_id);
    let scope = ctx.sema.scope(d.expr.value.to_node(&root).syntax())?;

    let term_search_ctx = TermSearchCtx {
        sema: &ctx.sema,
        scope: &scope,
        goal: d.expected.clone(),
        config: TermSearchConfig {
            fuel: ctx.config.term_search_fuel,
            enable_borrowcheck: ctx.config.term_search_borrowck,
            ..Default::default()
        },
    };
    let paths = term_search(&term_search_ctx);

    let assists: Vec<Assist> = d.expected
        .is_unknown()
        .not()
        .then(|| "todo!()".to_owned())
        .into_iter()
        .chain(paths.into_iter().filter_map(|path| {
            path.gen_source_code(
                &scope,
                &mut formatter,
                FindPathConfig {
                    prefer_no_std: ctx.config.prefer_no_std,
                    prefer_prelude: ctx.config.prefer_prelude,
                    prefer_absolute: ctx.config.prefer_absolute,
                    allow_unstable: ctx.is_nightly,
                },
                ctx.display_target,
            )
            .ok()
        }))
        .unique()
        .map(|code| Assist {
            id: AssistId::quick_fix("typed-hole"),
            label: Label::new(format!("Replace `_` with `{code}`")),
            group: Some(GroupLabel("Replace `_` with a term".to_owned())),
            target: original_range.range,
            source_change: Some(SourceChange::from_text_edit(...)),
            command: None,
        })
        .collect();

    if !assists.is_empty() { Some(assists) } else { None }
}
```

**Why This Matters for Contributors:**
- **Type-Directed Search**: Find expressions matching expected type
- **Fuel Limiting**: `term_search_fuel` config limits computation cost
- **Borrow Checking**: Optional `enable_borrowcheck` for soundness
- **Multiple Suggestions**: Return all valid expressions as separate assists
- **Grouped Fixes**: All suggestions grouped under "Replace `_` with a term"
- **Fallback to `todo!()`**: Always offer placeholder if search finds nothing

---

### 🔍 Expert Rust Commentary: Pattern 15

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Type-Directed Synthesis + Bounded Search

**Rust-Specific Insight:**
This is **AI-assisted code completion on steroids**. The term search algorithm finds expressions that match an expected type by:

1. **Scope analysis**: What values are in scope?
2. **Type matching**: Which can coerce to the target type?
3. **Composition**: Can we combine values (function calls, field access, etc.) to produce the target?
4. **Borrow checking**: Is the generated expression sound?

**Fuel Limiting:**
```rust
config: TermSearchConfig {
    fuel: ctx.config.term_search_fuel,  // Computational budget
    enable_borrowcheck: ctx.config.term_search_borrowck,
    ..Default::default()
},
```

The `fuel` parameter prevents runaway search. Term search is **exponential in theory** (combinatorial explosion of expressions), so:
- Low fuel (default: ~1000): Fast, finds simple expressions
- High fuel: Slower, finds complex multi-step transformations

**Borrow Checking:**
```rust
enable_borrowcheck: ctx.config.term_search_borrowck
```

This is subtle: without borrow checking, the search might generate:
```rust
let mut x = vec![1, 2, 3];
x.push(4);
x  // Error: x moved into push()
```

With borrow checking enabled, the search validates each generated expression, ensuring soundness at the cost of performance.

**Contribution Tip:**
Understanding what term search finds:
```rust
fn foo(x: &str) {
    let _: String = _;  // What can fill this hole?
}

// Term search tries:
1. Variables in scope: None (no String in scope)
2. Constructors: String::new() ✓
3. Conversions: x.to_string() ✓, x.to_owned() ✓
4. Complex: format!("{}", x) ✓

// User sees all as suggestions
```

**Grouped Fixes:**
```rust
group: Some(GroupLabel("Replace `_` with a term".to_owned()))
```

This groups all term search results under one menu item, preventing suggestion spam. Users expand the group to see all options.

**Fallback to `todo!()`:**
```rust
d.expected.is_unknown()
    .not()
    .then(|| "todo!()".to_owned())
    .into_iter()
    .chain(paths.into_iter()...)
```

If the type is unknown (inference failed), don't suggest `todo!()`—it's not helpful. But if the type is known, always offer `todo!()` as a last resort.

**Common Pitfalls:**
- **Too high fuel**: Causes editor lag during typing
- **Disabled borrow checking**: Generates unsound code that doesn't compile
- **No uniqueness**: Duplicate suggestions annoy users (`.unique()` fixes this)
- **Complex suggestions first**: Sort by simplicity (String::new() before format!("{}", x))

**Related Patterns in Ecosystem:**
- **Agda's Agsy**: Academic auto-programming system, similar goals
- **Idris's proof search**: Type-directed term synthesis for theorem proving
- **IntelliJ's live templates**: Less sophisticated, no type search
- **GitHub Copilot**: ML-based, different approach but similar UX

**Type-Directed Synthesis Research:**
This implements ideas from **program synthesis** research:
1. **Type-driven**: Use types as specifications
2. **Enumerative**: Try all combinations up to a bound
3. **Pruning**: Use type checking to reject invalid candidates early

**Performance Profile:**
```rust
// Fast searches (fuel ~100):
- Simple constructors (String::new())
- Direct conversions (.to_string())
- Field access (struct.field)

// Slow searches (fuel ~10000):
- Multi-step conversions (x.iter().collect())
- Generic function calls (Some(x.into()))
- Complex nesting (Ok(x.to_owned()))
```

**UX Tradeoff:**
More suggestions = more helpful but also more overwhelming. The current approach:
1. **Cap results**: Don't show 100+ suggestions
2. **Group under menu**: Prevents visual clutter
3. **Sort by likelihood**: Simple suggestions first

**Future Enhancement:**
Machine learning could rank suggestions by:
1. **Frequency in similar contexts**: What do other developers choose?
2. **Idiomatic patterns**: Prefer `.to_owned()` over `.clone().into()` for &str → String
3. **User history**: Learn per-user preferences

But the current approach is deterministic and predictable, which is valuable.

---

## Pattern 16: Test Infrastructure Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/tests.rs (lines 187-281)
**Category:** Testing

**Code Example:**
```rust
#[track_caller]
pub(crate) fn check_diagnostics(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
    let mut config = DiagnosticsConfig::test_sample();
    config.disabled.insert("inactive-code".to_owned());
    check_diagnostics_with_config(config, ra_fixture)
}

#[track_caller]
pub(crate) fn check_diagnostics_with_config(
    config: DiagnosticsConfig,
    #[rust_analyzer::rust_fixture] ra_fixture: &str,
) {
    let (db, files) = RootDatabase::with_many_files(ra_fixture);
    let mut annotations = files
        .iter()
        .copied()
        .flat_map(|file_id| {
            super::full_diagnostics(&db, &config, &AssistResolveStrategy::All, file_id.file_id(&db))
                .into_iter()
                .map(|d| {
                    let mut annotation = String::new();
                    if let Some(fixes) = &d.fixes {
                        annotation.push_str("💡 ")
                    }
                    annotation.push_str(match d.severity {
                        Severity::Error => "error",
                        Severity::WeakWarning => "weak",
                        Severity::Warning => "warn",
                        Severity::Allow => "allow",
                    });
                    annotation.push_str(": ");
                    annotation.push_str(&d.message);
                    (d.range, annotation)
                })
        })
        .into_group_map();

    for file_id in files {
        let file_id = file_id.file_id(&db);
        let mut actual = annotations.remove(&file_id).unwrap_or_default();
        let mut expected = extract_annotations(db.file_text(file_id).text(&db));

        expected.sort_by_key(|(range, s)| (range.start(), s.clone()));
        actual.sort_by_key(|(range, s)| (range.start(), s.clone()));
        actual.dedup();

        if expected != actual {
            let fneg = expected.iter().filter(|x| !actual.contains(x)).collect::<Vec<_>>();
            let fpos = actual.iter().filter(|x| !expected.contains(x)).collect::<Vec<_>>();
            panic!("Diagnostic test failed.\nFalse negatives: {fneg:?}\nFalse positives: {fpos:?}");
        }
    }
}
```

**Why This Matters for Contributors:**
- **Annotation-Based Testing**: Write expected diagnostics as comments in test code
- **Format**: `//^^^ 💡 error: message here` where `💡` indicates fix available
- **Multi-File Support**: Test diagnostics across multiple files
- **False Positive/Negative Reporting**: Clear distinction in test failures
- **Config Customization**: `check_diagnostics_with_disabled()` for testing specific configs
- **Deduplication**: Handles duplicate diagnostics from macro expansions

---

### 🔍 Expert Rust Commentary: Pattern 16

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Annotation-Based Testing + Golden Master

**Rust-Specific Insight:**
This is **literate testing** for diagnostics: expected diagnostics are embedded as comments in the test code. The infrastructure then:

1. **Extracts annotations**: Parse `//^^^` comments to find expected diagnostics
2. **Runs diagnostics**: Invoke the diagnostic system on the test code
3. **Compares**: Match actual diagnostics against annotations
4. **Reports diffs**: Show false positives and false negatives clearly

**Annotation Format:**
```rust
fn test() {
    let x: u32 = "string";
    //           ^^^^^^^^ 💡 error: mismatched types
}

// Parsed as:
Annotation {
    range: [13..21],  // "string" span
    message: "💡 error: mismatched types"
}
```

**Severity Encoding:**
```rust
annotation.push_str(match d.severity {
    Severity::Error => "error",
    Severity::WeakWarning => "weak",
    Severity::Warning => "warn",
    Severity::Allow => "allow",
});
```

This allows testing severity calculation, not just presence of diagnostics.

**Fix Indicator:**
```rust
if let Some(fixes) = &d.fixes {
    annotation.push_str("💡 ")  // Lightbulb = fix available
}
```

The 💡 emoji indicates a quick fix exists. This is **self-documenting test assertions**: reading the test shows what fixes are expected.

**Contribution Tip:**
Writing diagnostic tests:
```rust
#[test]
fn test_my_diagnostic() {
    check_diagnostics(r#"
        fn foo() {
            let x = unknown();
            //      ^^^^^^^ error: cannot find function `unknown` in this scope

            let y: u32 = "string";
            //           ^^^^^^^^ 💡 error: mismatched types

            #[allow(my_lint)]
            let _unused = 42;  // No diagnostic here due to allow
        }
    "#);
}
```

**Multi-File Testing:**
```rust
check_diagnostics(r#"
//- /main.rs
mod foo;

//- /foo.rs
pub fn bar() {}
    //^^^ warn: unused function
"#);
```

The `//- /path` syntax creates multi-file test fixtures, essential for testing cross-file diagnostics.

**Deduplication:**
```rust
actual.dedup();  // Remove duplicate diagnostics
```

Macro expansions can generate the same diagnostic multiple times (once per expansion site). Deduplication prevents false positives in tests.

**Error Reporting:**
```rust
if expected != actual {
    let fneg = expected.iter().filter(|x| !actual.contains(x)).collect::<Vec<_>>();
    let fpos = actual.iter().filter(|x| !expected.contains(x)).collect::<Vec<_>>();
    panic!("Diagnostic test failed.\nFalse negatives: {fneg:?}\nFalse positives: {fpos:?}");
}
```

This is **actionable test failure output**:
- **False negatives**: Expected diagnostics that didn't appear (need to fix detection)
- **False positives**: Unexpected diagnostics (need to suppress or fix annotation)

**Common Pitfalls:**
- **Whitespace sensitivity**: Annotations use column positions, be careful with tabs vs spaces
- **Multiple diagnostics on one line**: Order them left-to-right in the annotation
- **Macro expansions**: Test files with macros can have surprising diagnostic positions
- **Missing `💡`**: If a fix exists but annotation lacks 💡, test fails

**Related Patterns in Ecosystem:**
- **rustc's `ui` tests**: Similar annotation format, gold standard in Rust testing
- **Clippy's test harness**: Uses same infrastructure
- **Jest's snapshot testing**: JavaScript equivalent but less precise (text-based)
- **trybuild**: Tests compile failures with similar annotation style

**Track Caller:**
```rust
#[track_caller]
pub(crate) fn check_diagnostics(...) { ... }
```

This is critical: when a test fails, the error points to the **test invocation** line, not inside the helper. This makes failures easy to locate.

**Config Customization:**
```rust
let mut config = DiagnosticsConfig::test_sample();
config.disabled.insert("inactive-code".to_owned());  // Disable noisy diagnostics
```

Tests can customize diagnostic configuration to focus on specific diagnostics. Default disables `inactive-code` because it's very noisy in tests.

**Performance Note:**
The test infrastructure is **fast** because:
1. Single-file tests don't touch the filesystem
2. Database queries are cached within each test
3. No compilation or external processes

This enables hundreds of diagnostic tests to run in seconds.

**Design Philosophy:**
This implements **specification by example**: the test cases ARE the specification. Adding a new diagnostic requires adding test cases showing when it fires, which serves as both:
1. **Regression test**: Ensures the diagnostic keeps working
2. **Documentation**: Shows developers what the diagnostic does

---

## Pattern 17: Rename-Based Fixes for Case Violations
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-diagnostics/src/handlers/incorrect_case.rs (lines 36-57)
**Category:** Refactoring-Based Fixes

**Code Example:**
```rust
fn fixes(ctx: &DiagnosticsContext<'_>, d: &hir::IncorrectCase) -> Option<Vec<Assist>> {
    let root = ctx.sema.db.parse_or_expand(d.file);
    let name_node = d.ident.to_node(&root);
    let def = NameClass::classify(&ctx.sema, &name_node)?.defined()?;

    let name_node = InFile::new(d.file, name_node.syntax());
    let frange = name_node.original_file_range_rooted(ctx.sema.db);

    let label = format!("Rename to {}", d.suggested_text);
    let mut res = unresolved_fix("change_case", &label, frange.range);

    if ctx.resolve.should_resolve(&res.id) {
        let source_change = def.rename(
            &ctx.sema,
            &d.suggested_text,
            RenameDefinition::Yes,
            &ctx.config.rename_config(),
        );
        res.source_change = Some(source_change.ok().unwrap_or_default());
    }

    Some(vec![res])
}
```

**Why This Matters for Contributors:**
- **Semantic Rename**: Use full rename refactoring, not simple text replacement
- **Updates All References**: Renaming `NonSnakeCase` to `non_snake_case` updates usages
- **Lazy Resolution**: Only compute rename if `should_resolve()` returns true
- **Error Handling**: `unwrap_or_default()` provides empty change if rename fails
- **NameClass Classification**: Determines what kind of definition (local, function, etc.)

---

### 🔍 Expert Rust Commentary: Pattern 17

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5)

**Pattern Classification:** Semantic Rename + Lazy Resolution

**Rust-Specific Insight:**
This pattern demonstrates the **difference between text replacement and semantic refactoring**. For case violations, rust-analyzer uses full rename refactoring, not simple string replacement:

```rust
let source_change = def.rename(
    &ctx.sema,
    &d.suggested_text,
    RenameDefinition::Yes,
    &ctx.config.rename_config(),
);
```

This is critical because renaming `NonSnakeCase` to `non_snake_case` must:
1. **Find all references**: Across files, modules, even in documentation
2. **Handle shadowing**: Ensure the new name doesn't conflict
3. **Update imports**: `use foo::NonSnakeCase` → `use foo::non_snake_case`
4. **Preserve semantics**: Don't break code

**Lazy Resolution:**
```rust
if ctx.resolve.should_resolve(&res.id) {
    // Only compute rename if needed
    let source_change = def.rename(...);
    res.source_change = Some(source_change.ok().unwrap_or_default());
}
```

This is **performance optimization**: the diagnostic shows "Rename to foo_bar" immediately, but the actual rename computation (expensive) only happens when the user hovers over the lightbulb or applies the fix.

**NameClass Classification:**
```rust
let def = NameClass::classify(&ctx.sema, &name_node)?.defined()?;
```

`NameClass` determines what kind of definition this is:
- `NameClass::Definition`: A definition (e.g., `fn foo()`)
- `NameClass::ConstReference`: A const or static
- `NameClass::PatFieldShorthand`: Special case for `Foo { field }` patterns

This is necessary because different definition types have different rename constraints.

**Contribution Tip:**
Understanding rename semantics:
```rust
fn example() {
    let NonSnakeCase = 42;  // Violates naming convention

    // After rename to non_snake_case:
    let non_snake_case = 42;
    //  ^^^^^^^^^^^^^^ Updates definition

    dbg!(non_snake_case);
    //   ^^^^^^^^^^^^^^ Updates reference
}
```

**Error Handling:**
```rust
res.source_change = Some(source_change.ok().unwrap_or_default());
```

If rename fails (e.g., would create a conflict), the fix has an empty `SourceChange`, which makes it a no-op. This is **graceful degradation**: the diagnostic still appears, but the fix doesn't break the code.

**Common Pitfalls:**
- **Using text replacement**: Would miss references in other files
- **Not checking conflicts**: Renaming `x` to `y` when `y` exists would break code
- **Ignoring imports**: Must update `use` statements
- **Macro expansions**: Rename might fail in macro-generated code

**Related Patterns in Ecosystem:**
- **rustc's error messages**: Suggest the fix but don't apply it
- **Clippy**: Reports the violation but doesn't provide fixes
- **IntelliJ's inspections**: Similar semantic rename approach
- **VSCode's rename refactoring**: Same underlying LSP `textDocument/rename`

**Rename Configuration:**
```rust
&ctx.config.rename_config()
```

This includes:
- **Search in comments**: Whether to rename in doc comments
- **Search in strings**: Whether to rename in string literals
- **Rename in macro defs**: Whether to rename in macro definitions

These are user preferences that must be respected.

**Design Choice: Unresolved Fix:**
```rust
let mut res = unresolved_fix("change_case", &label, frange.range);
```

The fix is initially "unresolved"—it lacks the actual edit. This is an optimization:
1. Diagnostics are computed for the whole file immediately (for responsive UI)
2. Edits are computed on-demand (when user interacts)

This makes the editor feel fast even with many diagnostics.

**Edge Case: Macro Definitions:**
If the name is defined in a macro definition, renaming might:
1. **Fail entirely**: Can't rename macro template parameters
2. **Affect multiple expansions**: One rename affects all expansion sites
3. **Break macro hygiene**: Rename might introduce name conflicts

The `unwrap_or_default()` handles these by producing an empty change.

**UX Insight:**
The fix message is precise:
```rust
format!("Rename to {}", d.suggested_text)
```

It shows the exact new name, not just "Fix naming violation". This is important because:
- **snake_case** vs **SCREAMING_SNAKE_CASE** vs **UpperCamelCase** depend on item type
- User might disagree with the suggestion
- Clarity prevents accidental applications

**Performance Note:**
The rename operation uses **salsa queries** internally, so:
1. Name resolution is cached
2. Syntax tree is already in memory
3. Most renames complete in <100ms

This makes the fix feel instant even on large codebases.

---

## Summary for Contributors

### Adding a New Diagnostic

1. **Define HIR diagnostic** in `hir` crate (or use syntax-based approach)
2. **Create handler module** in `src/handlers/your_diagnostic.rs`
3. **Implement handler function** with signature `pub(crate) fn your_diagnostic(ctx: &DiagnosticsContext<'_>, d: &hir::YourDiagnostic) -> Diagnostic`
4. **Add to dispatch** in `lib.rs` `semantic_diagnostics()` match statement
5. **Implement fixes** in private `fixes()` function if applicable
6. **Write tests** in handler module using `check_diagnostics()` infrastructure
7. **Mark stability** with `.stable()` or leave experimental

### Fix Generation Guidelines

- Return `Option<Vec<Assist>>` from `fixes()` function
- Return `None` if no fix applicable (don't return empty `Vec`)
- Use `TextEdit::builder()` for simple text edits
- Use `clone_for_update()` + mutable syntax tree for complex edits
- Check for macro context and bail if fixes can't be applied
- Use type information (`could_coerce_to()`, `could_unify_with()`) to validate applicability
- Group related fixes under `GroupLabel`

### Testing Best Practices

- Use `check_diagnostics()` for standard tests
- Use `check_fix()` to test specific fix application
- Use `//^^^` annotation to mark expected diagnostic range
- Include severity: `error`, `warn`, `weak`, or `allow`
- Add `💡` to annotation if fix expected
- Use `#[track_caller]` on test helpers for better error messages
- Test with different Rust editions if behavior varies

### Performance Considerations

- Early return from handlers if diagnostic doesn't apply
- Use `Option<Diagnostic>` return type to suppress diagnostics conditionally
- Don't compute expensive fixes if `ctx.resolve.should_resolve()` is false
- Filter experimental diagnostics early in pipeline
- Batch syntax tree traversals where possible

### Key Files to Know

- `lib.rs`: Central registration, lint handling, macro filtering
- `handlers/*.rs`: One file per diagnostic type
- `tests.rs`: Test infrastructure
- Handler modules follow standard structure: public handler + private fixes + tests

---

## 📊 Expert Summary: IDE Diagnostics Architecture

### Overall Architecture Rating: ⭐⭐⭐⭐⭐ (5/5)

The rust-analyzer diagnostics system represents **production-grade IDE infrastructure** with several standout qualities:

### Architectural Strengths

**1. Type-Safe Extensibility**
- Exhaustive match-based dispatch prevents missing diagnostic handlers
- Compiler enforces that all diagnostic types are handled
- Adding new diagnostics causes compile failures at all integration points

**2. Layered Sophistication**
```
┌─────────────────────────────────────┐
│  Syntax-Based Diagnostics (Fast)   │  ← Style lints, simple patterns
├─────────────────────────────────────┤
│  Semantic Diagnostics (Accurate)   │  ← Type errors, name resolution
├─────────────────────────────────────┤
│  Attribute Processing (Policy)      │  ← #[allow], edition defaults
├─────────────────────────────────────┤
│  Macro Filtering (Context-Aware)   │  ← External macro suppression
├─────────────────────────────────────┤
│  Fix Generation (Multi-Strategy)    │  ← Type-guided transformations
└─────────────────────────────────────┘
```

**3. Performance-First Design**
- Lazy fix resolution (compute edits only on-demand)
- Salsa caching for semantic queries
- Syntax-based handlers for style lints
- Early filtering (experimental, disabled) before expensive operations

**4. Fail-Safe Defaults**
- New diagnostics experimental by default
- Macro context suppresses most lints
- Optional fix generation (return `None` if inapplicable)
- Graceful degradation on rename failures

### Key Design Patterns

**Pattern Family: Algebraic Dispatch**
- `AnyDiagnostic` enum as type-safe registry
- Match-based routing with exhaustiveness checking
- Zero runtime overhead (compiles to jump tables)

**Pattern Family: Semantic Analysis**
- `DiagnosticsContext` as unified semantic facade
- Type-guided fix generation (`could_coerce_to`, `could_unify_with`)
- `FamousDefs` for identifying standard library types

**Pattern Family: Progressive Disclosure**
- Experimental gating for safe feature rollout
- Lazy resolution for performance
- Multi-strategy fix generation for power users

**Pattern Family: Editor Integration**
- Display range adjustment for precise highlighting
- Annotation-based testing for specification
- Grouping and prioritization of fixes

### Comparison to Similar Systems

| Feature | rust-analyzer | rustc | Clippy | IntelliJ IDEA |
|---------|---------------|-------|--------|---------------|
| Type-safe dispatch | ✅ Enum exhaustiveness | ✅ Match exhaustive | ❌ Macro registration | ❌ Runtime lookup |
| Lazy fix resolution | ✅ On-demand | ❌ N/A (no fixes) | ❌ Eager | ✅ On-demand |
| Macro awareness | ✅ External suppression | ⚠️ Basic | ⚠️ Basic | ❌ Poor |
| Multi-strategy fixes | ✅ Accumulator pattern | ❌ N/A | ❌ Single fix | ✅ Intentions |
| Term search | ✅ Type-directed | ❌ N/A | ❌ N/A | ⚠️ Template-based |
| Annotation testing | ✅ Golden master | ✅ UI tests | ✅ UI tests | ❌ Separate |

### Critical Implementation Details

**1. Mutable Syntax Trees**
- `clone_for_update()` pattern for safe mutation
- `diff()` algorithm for minimal text edits
- Separate strategies for macro vs. non-macro code

**2. Lint Attribute Resolution**
- Ancestor walking to find closest `#[allow]`/`#[warn]`
- Edition-dependent default severities
- Lint group expansion (`warnings`, `clippy::all`)
- Outline module special handling

**3. Type System Integration**
- Hypothetical type checking for fix validation
- Coercion analysis (`could_coerce_to`)
- Unification checking (`could_unify_with`)
- ADT detection and manipulation

**4. Configuration Management**
- Per-diagnostic disabling
- Experimental feature gating
- Term search fuel limits
- Borrow checking toggles

### Learning Path for Contributors

**Beginner (Syntax-Based Diagnostics)**
1. Start with `handlers/field_shorthand.rs` - pure syntax analysis
2. Understand `match_ast!` macro for type-safe node dispatch
3. Practice with simple text edits (`TextEdit::builder()`)

**Intermediate (Semantic Diagnostics)**
1. Study `handlers/type_mismatch.rs` - type-guided fixes
2. Learn `DiagnosticsContext` usage patterns
3. Implement multi-strategy fix generation

**Advanced (Infrastructure)**
1. Understand `lib.rs` dispatch mechanism
2. Explore lint attribute resolution
3. Contribute to term search algorithms

### Common Contributor Mistakes

**❌ Anti-Pattern: Breaking Exhaustiveness**
```rust
AnyDiagnostic::MyDiag(d) => handlers::my_diag(&ctx, &d),
_ => unreachable!(),  // DON'T DO THIS - defeats type safety
```

**✅ Correct Pattern: Exhaustive Match**
```rust
AnyDiagnostic::MyDiag(d) => handlers::my_diag(&ctx, &d),
AnyDiagnostic::OtherDiag(d) => handlers::other_diag(&ctx, &d),
// Compiler ensures all variants handled
```

**❌ Anti-Pattern: Returning Empty Vec**
```rust
fn fixes(...) -> Option<Vec<Assist>> {
    Some(vec![])  // Wrong - use None
}
```

**✅ Correct Pattern: None for No Fixes**
```rust
fn fixes(...) -> Option<Vec<Assist>> {
    if fixes.is_empty() { None } else { Some(fixes) }
}
```

**❌ Anti-Pattern: Forgetting Macro Guards**
```rust
fn fixes(...) -> Option<Vec<Assist>> {
    // Generate fix without checking macro context
    Some(vec![...])
}
```

**✅ Correct Pattern: Macro Context Checks**
```rust
fn fixes(...) -> Option<Vec<Assist>> {
    if is_in_macro { return None; }  // Never suggest fixes in macros
    Some(vec![...])
}
```

### Performance Characteristics

**Diagnostic Computation**: O(n) in file size
- Syntax traversal: Linear in AST nodes
- Semantic queries: Cached by salsa
- Lint attribute resolution: O(depth) per diagnostic

**Fix Generation**: O(1) per diagnostic (lazy)
- Display-only mode: Just show lightbulb
- Resolution mode: Compute actual edits
- Amortized cost across many diagnostics

**Test Suite**: ~2-5 seconds for 100+ tests
- No filesystem I/O (in-memory fixtures)
- Salsa caching across tests
- Parallel test execution

---

## 🎯 Contribution Readiness Checklist

### Before Starting a Diagnostic PR

**Preparation**
- [ ] Read the **entire** `lib.rs` to understand dispatch flow
- [ ] Study **3+ existing handlers** similar to your diagnostic
- [ ] Identify if diagnostic is **semantic** (HIR-based) or **syntactic** (AST-based)
- [ ] Check if **rustc/Clippy** already has this diagnostic (maintain compatibility)
- [ ] Review **test infrastructure** (`tests.rs`) for annotation format

**Design Phase**
- [ ] Define **when diagnostic fires** with precision (avoid false positives)
- [ ] Determine **severity** (Error, Warning, WeakWarning, Allow)
- [ ] Plan **fix strategies** (if applicable) - multiple strategies preferred
- [ ] Consider **edition-specific** behavior
- [ ] Plan **macro handling** (suppress in external macros?)

**Implementation Phase**
- [ ] Create **handler module** `src/handlers/my_diagnostic.rs`
- [ ] Implement **public handler** function with correct signature
- [ ] Add **private `fixes()` helper** if quick fixes applicable
- [ ] Add to **dispatch match** in `lib.rs`
- [ ] Mark **experimental** (default) or `.stable()` if well-tested
- [ ] Set **`main_node`** for macro filtering support
- [ ] Use **`adjusted_display_range()`** for precise highlighting

**Fix Generation (If Applicable)**
- [ ] Return **`None`** if no fix available (not empty `Vec`)
- [ ] Check **macro context** - never suggest fixes in macros
- [ ] Use **type system** to validate fix applicability
- [ ] Implement **multiple strategies** (accumulator pattern)
- [ ] Use **`clone_for_update()`** for syntax tree mutation
- [ ] Use **`diff()`** for minimal edits (non-macro code)
- [ ] Test **all fix strategies** independently

**Testing Phase**
- [ ] Write **annotation-based tests** with `check_diagnostics()`
- [ ] Test **all code paths** that trigger the diagnostic
- [ ] Test **false positive prevention** (cases that shouldn't trigger)
- [ ] Test **fixes** with `check_fix()` for each strategy
- [ ] Test **multi-file** scenarios if relevant
- [ ] Test **macro contexts** (should suppress correctly)
- [ ] Test **edition differences** if applicable
- [ ] Use **`#[track_caller]`** on test helpers

**Documentation Phase**
- [ ] Add **`// Diagnostic: diagnostic-name`** comment at top of handler
- [ ] Document **when diagnostic fires** in module docs
- [ ] Document **fix strategies** and their applicability
- [ ] Add **examples** in test cases as documentation
- [ ] Update **book/diagnostics** page if diagnostic is stable

**Review Preparation**
- [ ] Run **`cargo test`** - all tests pass
- [ ] Run **`cargo clippy`** - no warnings
- [ ] Run **`cargo fmt`** - consistent formatting
- [ ] Test **manually** in editor on real code
- [ ] Check **performance** - no editor lag with diagnostic
- [ ] Verify **false positive rate** is low
- [ ] Self-review for **common pitfalls** (see above)

### Diagnostic Maturity Levels

**Level 1: Experimental** (Initial PR)
- Works on basic cases
- May have false positives
- Disabled by default (`experimental: true`)
- Gather feedback from early users

**Level 2: Beta** (After Dogfooding)
- Tested on rust-analyzer codebase
- Most false positives eliminated
- Still experimental, but refined
- Ready for wider testing

**Level 3: Stable** (Mark `.stable()`)
- Low false positive rate (<1%)
- Comprehensive test coverage
- Positive community feedback
- Fix strategies well-tested
- Enabled by default

**Level 4: Production** (In Release)
- Battle-tested across diverse codebases
- Performance validated
- Integration with other diagnostics verified
- Documentation complete

### Red Flags During Review

**🚩 Immediate Reject Signals**
- No tests or minimal test coverage
- Breaking exhaustiveness with `_ => unreachable!()`
- Suggesting fixes in macro expansions
- High false positive rate
- Expensive operations without caching
- Not handling `None` cases

**⚠️ Needs Improvement Signals**
- Only one test case
- No fix strategies when applicable
- Unclear diagnostic messages
- Missing edition handling
- Poor display range selection
- Not respecting lint attributes

**✅ High-Quality PR Signals**
- Multiple test cases covering edge cases
- Multi-strategy fix generation
- Clear, actionable messages
- Proper macro handling
- Edition-aware behavior
- Comprehensive documentation

### Next Steps for Aspiring Contributors

**1. Small Wins**
Start with enhancements to existing diagnostics:
- Improve an existing fix strategy
- Add missing test cases
- Better display range adjustment
- Clearer diagnostic messages

**2. Medium Complexity**
Implement a new syntax-based diagnostic:
- Style lints (like `field_shorthand`)
- Naming conventions
- Simple pattern detection

**3. Advanced Contributions**
Implement semantic diagnostics:
- Type-related errors
- Lifetime issues
- Trait bound violations

**4. Infrastructure Improvements**
Enhance the diagnostic system itself:
- Better term search heuristics
- Improved macro filtering
- Performance optimizations
- New testing utilities

### Resources for Deep Dives

**Essential Reading**
- `lib.rs` - Central dispatch and filtering
- `handlers/type_mismatch.rs` - Complex multi-strategy fixes
- `handlers/field_shorthand.rs` - Syntax-based diagnostic template
- `tests.rs` - Test infrastructure patterns

**Key Concepts to Master**
- Salsa query system (for semantic analysis)
- Rowan syntax trees (for AST manipulation)
- HIR diagnostic types (in `hir` crate)
- LSP diagnostic mapping (in `rust-analyzer` binary)

**Community Support**
- Zulip #rust-analyzer stream - Ask questions
- GitHub issues labeled `good-first-issue`
- Weekly triage meetings
- Mentor volunteers for new contributors

---

**Final Note**: The rust-analyzer diagnostic system is **mature, well-architected, and contributor-friendly**. The patterns documented here represent years of refinement by expert Rust developers. Study them deeply—they embody best practices for:

- Type-safe extensibility
- Performance-conscious design
- Editor integration UX
- Fail-safe defaults
- Progressive feature rollout

These patterns are applicable far beyond diagnostics—they're templates for building robust, high-performance Rust systems.

