# Idiomatic Rust Patterns: HIR Facade & Macro Expansion
> Source: rust-analyzer/crates/hir + crates/hir-expand
> Analyzed: 20+ files across HIR public facade and macro expansion systems

## Pattern 1: Public Facade with Internal Abstraction Boundary
**File:** crates/hir/src/lib.rs (lines 1-18)
**Category:** Architectural Pattern / API Design
**Code Example:**
```rust
//! HIR (previously known as descriptors) provides a high-level object-oriented
//! access to Rust code.
//!
//! The principal difference between HIR and syntax trees is that HIR is bound
//! to a particular crate instance. That is, it has cfg flags and features
//! applied. So, the relation between syntax and HIR is many-to-one.
//!
//! HIR is the public API of the all of the compiler logic above syntax trees.
//! It is written in "OO" style. Each type is self contained (as in, it knows its
//! parents and full context). It should be "clean code".
//!
//! `hir_*` crates are the implementation of the compiler logic.
//! They are written in "ECS" style, with relatively little abstractions.
//! Many types are not self-contained, and explicitly use local indexes, arenas, etc.
//!
//! `hir` is what insulates the "we don't know how to actually write an incremental compiler"
//! from the ide with completions, hovers, etc. It is a (soft, internal) boundary:
//! <https://www.tedinski.com/2018/02/06/system-boundaries.html>.
```
**Why This Matters for Contributors:** This establishes a crucial architectural boundary between the user-facing OO API and the implementation's ECS-style internals. Contributors should understand that `hir` provides a clean facade while `hir_*` crates use performance-optimized data structures. This pattern shows how to design dual-purpose APIs: user-friendly on the outside, performance-oriented on the inside.

---

## Pattern 2: HasSource Trait for Bidirectional Syntax Mapping
**File:** crates/hir/src/has_source.rs (lines 20-44)
**Category:** Trait Design / Source Mapping
**Code Example:**
```rust
pub trait HasSource: Sized {
    type Ast: AstNode;
    /// Fetches the definition's source node.
    /// Using [`crate::SemanticsImpl::source`] is preferred when working with [`crate::Semantics`],
    /// as that caches the parsed file in the semantics' cache.
    ///
    /// The current some implementations can return `InFile` instead of `Option<InFile>`.
    /// But we made this method `Option` to support rlib in the future
    /// by <https://github.com/rust-lang/rust-analyzer/issues/6913>
    fn source(self, db: &dyn HirDatabase) -> Option<InFile<Self::Ast>>;

    /// Fetches the source node, along with its full range.
    ///
    /// The reason for the separate existence of this method is that some things, notably builtin derive impls,
    /// do not really have a source node, at least not of the correct type. But we still can trace them
    /// to source code (the derive producing them). So this method will return the range if it is supported,
    /// and if the node is supported too it will return it as well.
    fn source_with_range(
        self,
        db: &dyn HirDatabase,
    ) -> Option<InFile<(TextRange, Option<Self::Ast>)>> {
        let source = self.source(db)?;
        Some(source.map(|node| (node.syntax().text_range(), Some(node))))
    }
}
```
**Why This Matters for Contributors:** The HasSource trait enables navigating from semantic HIR types back to their original syntax. Note the `Option<InFile<Self::Ast>>` return type to handle builtin items without sources, and the `source_with_range` variant for items like derives that have ranges but not precise nodes. This pattern is critical for IDE features like "go to definition" and demonstrates graceful handling of compiler-generated code.

---

## Pattern 3: InFile<T> Wrapper for File Context Preservation
**File:** crates/hir-expand/src/files.rs (lines 14-28)
**Category:** Type Wrapper / Context Tracking
**Code Example:**
```rust
/// `InFile<T>` stores a value of `T` inside a particular file/syntax tree.
///
/// Typical usages are:
///
/// * `InFile<SyntaxNode>` -- syntax node in a file
/// * `InFile<ast::FnDef>` -- ast node in a file
/// * `InFile<TextSize>` -- offset in a file
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InFileWrapper<FileKind, T> {
    pub file_id: FileKind,
    pub value: T,
}
pub type InFile<T> = InFileWrapper<HirFileId, T>;
pub type InMacroFile<T> = InFileWrapper<MacroCallId, T>;
pub type InRealFile<T> = InFileWrapper<EditionedFileId, T>;
```
**Why This Matters for Contributors:** InFile is fundamental to tracking where syntax comes from, especially important with macro expansion creating multiple syntax trees. The generic `InFileWrapper<FileKind, T>` pattern allows type-safe file tracking across real files, macro expansions, and HIR files. This prevents mixing up syntax nodes from different files and enables precise source mapping across macro boundaries.

---

## Pattern 4: HirFileId Enum for Real vs Macro File Distinction
**File:** crates/hir-expand/src/lib.rs (lines 1080-1141)
**Category:** Enum Modeling / Type Safety
**Code Example:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa_macros::Supertype)]
pub enum HirFileId {
    FileId(EditionedFileId),
    MacroFile(MacroCallId),
}

impl HirFileId {
    #[inline]
    pub fn macro_file(self) -> Option<MacroCallId> {
        match self {
            HirFileId::FileId(_) => None,
            HirFileId::MacroFile(it) => Some(it),
        }
    }

    #[inline]
    pub fn is_macro(self) -> bool {
        matches!(self, HirFileId::MacroFile(_))
    }

    #[inline]
    pub fn file_id(self) -> Option<EditionedFileId> {
        match self {
            HirFileId::FileId(it) => Some(it),
            HirFileId::MacroFile(_) => None,
        }
    }

    pub fn original_file(self, db: &dyn ExpandDatabase) -> EditionedFileId {
        let mut file_id = self;
        loop {
            match file_id {
                HirFileId::FileId(id) => break id,
                HirFileId::MacroFile(macro_call_id) => {
                    file_id = db.lookup_intern_macro_call(macro_call_id).kind.file_id()
                }
            }
        }
    }
}
```
**Why This Matters for Contributors:** HirFileId distinguishes real source files from macro-generated syntax trees. The `original_file` method demonstrates the pattern of unwrapping macro expansion hierarchies by repeatedly following MacroCallId chains until reaching a real file. This is essential for diagnostics and navigation features that need to trace back to user-written code.

---

## Pattern 5: Semantics Struct as Primary API Entry Point
**File:** crates/hir/src/semantics.rs (lines 160-217)
**Category:** API Facade / Builder Pattern
**Code Example:**
```rust
/// Primary API to get semantic information, like types, from syntax trees.
pub struct Semantics<'db, DB: ?Sized> {
    pub db: &'db DB,
    imp: SemanticsImpl<'db>,
}

pub struct SemanticsImpl<'db> {
    pub db: &'db dyn HirDatabase,
    s2d_cache: RefCell<SourceToDefCache>,
    /// MacroCall to its expansion's MacroCallId cache
    macro_call_cache: RefCell<FxHashMap<InFile<ast::MacroCall>, MacroCallId>>,
}

impl<DB: HirDatabase> Semantics<'_, DB> {
    /// Creates an instance that's strongly coupled to its underlying database type.
    pub fn new(db: &DB) -> Semantics<'_, DB> {
        let impl_ = SemanticsImpl::new(db);
        Semantics { db, imp: impl_ }
    }
}

impl Semantics<'_, dyn HirDatabase> {
    /// Creates an instance that's weakly coupled to its underlying database type.
    pub fn new_dyn(db: &'_ dyn HirDatabase) -> Semantics<'_, dyn HirDatabase> {
        let impl_ = SemanticsImpl::new(db);
        Semantics { db, imp: impl_ }
    }
}
```
**Why This Matters for Contributors:** Semantics is the main entry point for IDE features. It caches source-to-def mappings and macro expansions using RefCell for interior mutability. The split between `Semantics<'_, DB>` (strongly typed) and `Semantics<'_, dyn HirDatabase>` (trait object) demonstrates supporting both generic and dynamic dispatch patterns for different use cases.

---

## Pattern 6: MacroCallLoc Structure for Macro Identity
**File:** crates/hir-expand/src/lib.rs (lines 235-261)
**Category:** Data Modeling / Salsa Interning
**Code Example:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroCallLoc {
    pub def: MacroDefId,
    pub krate: Crate,
    pub kind: MacroCallKind,
    pub ctxt: SyntaxContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacroDefId {
    pub krate: Crate,
    pub edition: Edition,
    pub kind: MacroDefKind,
    pub local_inner: bool,
    pub allow_internal_unsafe: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacroDefKind {
    Declarative(AstId<ast::Macro>, MacroCallStyles),
    BuiltIn(AstId<ast::Macro>, BuiltinFnLikeExpander),
    BuiltInAttr(AstId<ast::Macro>, BuiltinAttrExpander),
    BuiltInDerive(AstId<ast::Macro>, BuiltinDeriveExpander),
    BuiltInEager(AstId<ast::Macro>, EagerExpander),
    ProcMacro(AstId<ast::Fn>, CustomProcMacroExpander, ProcMacroKind),
}
```
**Why This Matters for Contributors:** MacroCallLoc captures everything needed to identify and re-expand a macro: definition, invoking crate, call kind, and hygiene context. MacroDefKind's enum discriminates between all macro types (declarative, builtin, proc-macro). This structure is salsa-interned as MacroCallId, enabling efficient caching and incremental computation across macro expansions.

---

## Pattern 7: ExpandTo Enum for Syntactic Context Detection
**File:** crates/hir-expand/src/lib.rs (lines 994-1056)
**Category:** Context Analysis / Polymorphic Expansion
**Code Example:**
```rust
/// In Rust, macros expand token trees to token trees. When we want to turn a
/// token tree into an AST node, we need to figure out what kind of AST node we
/// want: something like `foo` can be a type, an expression, or a pattern.
///
/// Naively, one would think that "what this expands to" is a property of a
/// particular macro: macro `m1` returns an item, while macro `m2` returns an
/// expression, etc. That's not the case -- macros are polymorphic in the
/// result, and can expand to any type of the AST node.
///
/// What defines the actual AST node is the syntactic context of the macro
/// invocation. As a contrived example, in `let T![*] = T![*];` the first `T`
/// expands to a pattern, while the second one expands to an expression.
///
/// `ExpandTo` captures this bit of information about a particular macro call
/// site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpandTo {
    Statements,
    Items,
    Pattern,
    Type,
    Expr,
}

impl ExpandTo {
    pub fn from_call_site(call: &ast::MacroCall) -> ExpandTo {
        use syntax::SyntaxKind::*;
        let syn = call.syntax();
        let parent = match syn.parent() {
            Some(it) => it,
            None => return ExpandTo::Statements,
        };

        match parent.kind() {
            MACRO_ITEMS | SOURCE_FILE | ITEM_LIST => ExpandTo::Items,
            MACRO_STMTS | EXPR_STMT | STMT_LIST => ExpandTo::Statements,
            MACRO_PAT => ExpandTo::Pattern,
            MACRO_TYPE => ExpandTo::Type,
            ARG_LIST | ARRAY_EXPR | /* ... */ MACRO_EXPR => ExpandTo::Expr,
            _ => ExpandTo::Items, // Unknown, guess Items
        }
    }
}
```
**Why This Matters for Contributors:** This demonstrates that macros in Rust are polymorphic—the same macro can expand to different AST node types depending on syntactic position. The `from_call_site` implementation walks the syntax tree parent to determine the expected expansion type. This context sensitivity is crucial for parsing macro output correctly.

---

## Pattern 8: Intern/Lookup Trait Pattern for Salsa Integration
**File:** crates/hir-expand/src/lib.rs (lines 72-111)
**Category:** Database Integration / Interning Pattern
**Code Example:**
```rust
#[macro_export]
macro_rules! impl_intern_lookup {
    ($db:ident, $id:ident, $loc:ident, $intern:ident, $lookup:ident) => {
        impl $crate::Intern for $loc {
            type Database = dyn $db;
            type ID = $id;
            fn intern(self, db: &Self::Database) -> Self::ID {
                db.$intern(self)
            }
        }

        impl $crate::Lookup for $id {
            type Database = dyn $db;
            type Data = $loc;
            fn lookup(&self, db: &Self::Database) -> Self::Data {
                db.$lookup(*self)
            }
        }
    };
}

pub trait Intern {
    type Database: ?Sized;
    type ID;
    fn intern(self, db: &Self::Database) -> Self::ID;
}

pub trait Lookup {
    type Database: ?Sized;
    type Data;
    fn lookup(&self, db: &Self::Database) -> Self::Data;
}

impl_intern_lookup!(
    ExpandDatabase,
    MacroCallId,
    MacroCallLoc,
    intern_macro_call,
    lookup_intern_macro_call
);
```
**Why This Matters for Contributors:** This macro creates bidirectional mappings between lightweight IDs (MacroCallId) and full data structures (MacroCallLoc) using Salsa's interning system. The pattern enables efficient storage and comparison of macro calls while maintaining access to full details. This is fundamental to rust-analyzer's incremental computation architecture.

---

## Pattern 9: Lazy Macro Expansion with Firewall Queries
**File:** crates/hir-expand/src/db.rs (lines 97-110)
**Category:** Incremental Computation / Query Design
**Code Example:**
```rust
#[query_group::query_group]
pub trait ExpandDatabase: RootQueryDb {
    /// Lowers syntactic macro call to a token tree representation. That's a firewall
    /// query, only typing in the macro call itself changes the returned
    /// subtree.
    #[deprecated = "calling this is incorrect, call `macro_arg_considering_derives` instead"]
    #[salsa::invoke(macro_arg)]
    fn macro_arg(&self, id: MacroCallId) -> MacroArgResult;

    #[salsa::transparent]
    fn macro_arg_considering_derives(
        &self,
        id: MacroCallId,
        kind: &MacroCallKind,
    ) -> MacroArgResult;
}
```
**Why This Matters for Contributors:** Firewall queries are key to incremental compilation. The `macro_arg` query creates an isolation boundary—changes to the macro body don't invalidate downstream queries if the input token tree stays the same. The `transparent` attribute on `macro_arg_considering_derives` means it's recomputed with its dependencies rather than cached independently.

---

## Pattern 10: SpanMap Enum for Real vs Expansion Span Tracking
**File:** crates/hir-expand/src/span_map.rs (lines 13-69)
**Category:** Span Hygiene / Bidirectional Mapping
**Code Example:**
```rust
pub type ExpansionSpanMap = span::SpanMap;

/// Spanmap for a macro file or a real file
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpanMap {
    /// Spanmap for a macro file
    ExpansionSpanMap(Arc<ExpansionSpanMap>),
    /// Spanmap for a real file
    RealSpanMap(Arc<RealSpanMap>),
}

#[derive(Copy, Clone)]
pub enum SpanMapRef<'a> {
    /// Spanmap for a macro file
    ExpansionSpanMap(&'a ExpansionSpanMap),
    /// Spanmap for a real file
    RealSpanMap(&'a RealSpanMap),
}

impl SpanMap {
    pub fn span_for_range(&self, range: TextRange) -> Span {
        match self {
            Self::ExpansionSpanMap(span_map) => span_map.span_at(range.start()),
            Self::RealSpanMap(span_map) => span_map.span_for_range(range),
        }
    }

    #[inline]
    pub(crate) fn new(db: &dyn ExpandDatabase, file_id: HirFileId) -> SpanMap {
        match file_id {
            HirFileId::FileId(file_id) => SpanMap::RealSpanMap(db.real_span_map(file_id)),
            HirFileId::MacroFile(m) => {
                SpanMap::ExpansionSpanMap(db.parse_macro_expansion(m).value.1)
            }
        }
    }
}
```
**Why This Matters for Contributors:** SpanMap handles the dual nature of spans: real files have contiguous spans while macro expansions map output spans back to input spans. The enum pattern with both owned and borrowed variants (`SpanMap` and `SpanMapRef`) is typical for Rust APIs that need both storage and temporary views. This is critical for diagnostics and hygiene tracking.

---

## Pattern 11: Eager vs Lazy Expansion Dichotomy
**File:** crates/hir-expand/src/eager.rs (lines 1-118)
**Category:** Macro Expansion Strategy
**Code Example:**
```rust
/// Eagerly expanded macros (and also macros eagerly expanded by eagerly expanded macros,
/// which actually happens in practice too!) are resolved at the location of the "root" macro
/// that performs the eager expansion on its arguments.
/// If some name cannot be resolved at the eager expansion time it's considered unresolved,
/// even if becomes available later (e.g. from a glob import or other macro).

pub fn expand_eager_macro_input(
    db: &dyn ExpandDatabase,
    krate: Crate,
    macro_call: &ast::MacroCall,
    ast_id: AstId<ast::MacroCall>,
    def: MacroDefId,
    call_site: SyntaxContext,
    resolver: &dyn Fn(&ModPath) -> Option<MacroDefId>,
    eager_callback: EagerCallBackFn<'_>,
) -> ExpandResult<Option<MacroCallId>> {
    let expand_to = ExpandTo::from_call_site(macro_call);

    // Note:
    // When `lazy_expand` is called, its *parent* file must already exist.
    // Here we store an eager macro id for the argument expanded subtree
    // for that purpose.
    let loc = MacroCallLoc {
        def,
        krate,
        kind: MacroCallKind::FnLike { ast_id, expand_to: ExpandTo::Expr, eager: None },
        ctxt: call_site,
    };
    let arg_id = db.intern_macro_call(loc);

    let (arg_exp, arg_exp_map) = db.parse_macro_expansion(arg_id);
    // Recursively expand nested macros in the argument...
```
**Why This Matters for Contributors:** Eager macros (like `concat!`) expand their arguments before the macro itself, requiring special handling. The pattern creates a temporary MacroCallId for the arguments, expands them, then expands the outer macro. This recursive approach handles arbitrarily nested eager expansions. Understanding eager vs lazy expansion is crucial for contributors working on macro features.

---

## Pattern 12: Hygiene via SyntaxContext and Transparency
**File:** crates/hir-expand/src/hygiene.rs (lines 1-143)
**Category:** Macro Hygiene / Context Tracking
**Code Example:**
```rust
//! Machinery for hygienic macros.
//!
//! Inspired by Matthew Flatt et al., "Macros That Work Together: Compile-Time Bindings, Partial
//! Expansion, and Definition Contexts," *Journal of Functional Programming* 22, no. 2
//! (March 1, 2012): 181–216, <https://doi.org/10.1017/S0956796812000093>.

pub fn span_with_def_site_ctxt(
    db: &dyn ExpandDatabase,
    span: Span,
    expn_id: MacroCallId,
    edition: Edition,
) -> Span {
    span_with_ctxt_from_mark(db, span, expn_id, Transparency::Opaque, edition)
}

pub fn span_with_call_site_ctxt(
    db: &dyn ExpandDatabase,
    span: Span,
    expn_id: MacroCallId,
    edition: Edition,
) -> Span {
    span_with_ctxt_from_mark(db, span, expn_id, Transparency::Transparent, edition)
}

fn span_with_ctxt_from_mark(
    db: &dyn ExpandDatabase,
    span: Span,
    expn_id: MacroCallId,
    transparency: Transparency,
    edition: Edition,
) -> Span {
    Span {
        ctx: apply_mark(db, SyntaxContext::root(edition), expn_id, transparency, edition),
        ..span
    }
}
```
**Why This Matters for Contributors:** Hygiene in Rust macros prevents name collisions between macro definitions and call sites. The Transparency enum (Opaque/SemiOpaque/Transparent) controls name resolution scope. `def_site_ctxt` resolves names in the macro definition scope, while `call_site_ctxt` uses the invocation scope. This sophisticated system requires careful understanding for macro-related work.

---

## Pattern 13: SourceAnalyzer for Context-Aware HIR Analysis
**File:** crates/hir/src/source_analyzer.rs (lines 68-145)
**Category:** Analysis Context / Resolver Pattern
**Code Example:**
```rust
/// `SourceAnalyzer` is a convenience wrapper which exposes HIR API in terms of
/// original source files. It should not be used inside the HIR itself.
#[derive(Debug)]
pub(crate) struct SourceAnalyzer<'db> {
    pub(crate) file_id: HirFileId,
    pub(crate) resolver: Resolver<'db>,
    pub(crate) body_or_sig: Option<BodyOrSig<'db>>,
}

#[derive(Debug)]
pub(crate) enum BodyOrSig<'db> {
    Body {
        def: DefWithBodyId,
        body: Arc<Body>,
        source_map: Arc<BodySourceMap>,
        infer: Option<&'db InferenceResult>,
    },
    VariantFields {
        def: VariantId,
        store: Arc<ExpressionStore>,
        source_map: Arc<ExpressionStoreSourceMap>,
    },
    Sig {
        def: GenericDefId,
        store: Arc<ExpressionStore>,
        source_map: Arc<ExpressionStoreSourceMap>,
    },
}

impl<'db> SourceAnalyzer<'db> {
    pub(crate) fn new_for_body(
        db: &'db dyn HirDatabase,
        def: DefWithBodyId,
        node: InFile<&SyntaxNode>,
        offset: Option<TextSize>,
    ) -> SourceAnalyzer<'db> {
        let (body, source_map) = db.body_with_source_map(def);
        let scopes = db.expr_scopes(def);
        let scope = match offset {
            None => scope_for(db, &scopes, &source_map, node),
            Some(offset) => scope_for_offset(db, &scopes, &source_map, node.file_id, offset),
        };
        let resolver = resolver_for_scope(db, def, scope);
        SourceAnalyzer {
            resolver,
            body_or_sig: Some(BodyOrSig::Body {
                def, body, source_map,
                infer: Some(InferenceResult::for_body(db, def))
            }),
            file_id: node.file_id,
        }
    }
}
```
**Why This Matters for Contributors:** SourceAnalyzer bridges syntax and semantics for a specific source location. It constructs a Resolver scoped to the syntax position and optionally carries type inference results. The BodyOrSig enum handles different contexts (function bodies, type signatures, variant fields) uniformly. This pattern enables position-aware semantic queries like "what is the type of this expression?".

---

## Pattern 14: from_id Macro for Bidirectional Type Conversions
**File:** crates/hir/src/from_id.rs (lines 18-52)
**Category:** Code Generation / Type Safety
**Code Example:**
```rust
macro_rules! from_id {
    ($(($id:path, $ty:path)),* $(,)?) => {$(
        impl From<$id> for $ty {
            fn from(id: $id) -> $ty {
                $ty { id }
            }
        }
        impl From<$ty> for $id {
            fn from(ty: $ty) -> $id {
                ty.id
            }
        }
    )*}
}

from_id![
    (base_db::Crate, crate::Crate),
    (hir_def::ModuleId, crate::Module),
    (hir_def::StructId, crate::Struct),
    (hir_def::UnionId, crate::Union),
    (hir_def::EnumId, crate::Enum),
    (hir_def::TypeAliasId, crate::TypeAlias),
    (hir_def::TraitId, crate::Trait),
    (hir_def::StaticId, crate::Static),
    (hir_def::ConstId, crate::Const),
    (crate::AnyFunctionId, crate::Function),
    (hir_ty::next_solver::AnyImplId, crate::Impl),
    // ... more conversions
];
```
**Why This Matters for Contributors:** This macro generates boilerplate From implementations for ID types, enabling seamless conversion between internal hir_def IDs and public HIR types. The bidirectional conversions make the API ergonomic while maintaining type safety. This pattern is widely applicable when wrapping internal types in public facades.

---

## Pattern 15: ExpansionInfo for Macro Span Mapping
**File:** crates/hir-expand/src/lib.rs (lines 799-936)
**Category:** Span Mapping / Macro Transparency
**Code Example:**
```rust
/// ExpansionInfo mainly describes how to map text range between src and expanded macro
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpansionInfo {
    expanded: InMacroFile<SyntaxNode>,
    /// The argument TokenTree or item for attributes
    arg: InFile<Option<SyntaxNode>>,
    exp_map: Arc<ExpansionSpanMap>,
    arg_map: SpanMap,
    loc: MacroCallLoc,
}

impl ExpansionInfo {
    /// Maps the passed in file range down into a macro expansion if it is the input to a macro call.
    pub fn map_range_down_exact(
        &self,
        span: Span,
    ) -> Option<InMacroFile<impl Iterator<Item = (SyntaxToken, SyntaxContext)> + '_>> {
        if span.anchor.ast_id == NO_DOWNMAP_ERASED_FILE_AST_ID_MARKER {
            return None;
        }

        let tokens = self.exp_map.ranges_with_span_exact(span).flat_map(move |(range, ctx)| {
            self.expanded.value.covering_element(range).into_token().zip(Some(ctx))
        });

        Some(InMacroFile::new(self.expanded.file_id, tokens))
    }

    /// Maps up the text range out of the expansion hierarchy back into the original file its from.
    pub fn map_node_range_up(
        &self,
        db: &dyn ExpandDatabase,
        range: TextRange,
    ) -> Option<(FileRange, SyntaxContext)> {
        debug_assert!(self.expanded.value.text_range().contains_range(range));
        map_node_range_up(db, &self.exp_map, range)
    }
}
```
**Why This Matters for Contributors:** ExpansionInfo is the bidirectional bridge between macro input and output syntax. `map_range_down` traces input spans into expanded code, while `map_node_range_up` maps expansion syntax back to the original call site. This enables features like "find usages" and "rename" to work across macro boundaries, a notoriously difficult problem in macro systems.

---

## Pattern 16: Database Re-exports for Facade Boundary
**File:** crates/hir/src/db.rs (lines 1-9)
**Category:** Module Organization / API Design
**Code Example:**
```rust
//! Re-exports various subcrates databases so that the calling code can depend
//! only on `hir`. This breaks abstraction boundary a bit, it would be cool if
//! we didn't do that.
//!
//! But we need this for at least LRU caching at the query level.
pub use hir_def::db::DefDatabase;
pub use hir_expand::db::ExpandDatabase;
pub use hir_ty::db::HirDatabase;
```
**Why This Matters for Contributors:** This pattern acknowledges a pragmatic compromise: while `hir` is supposed to be a facade, performance requirements (LRU caching) force exposing the underlying database traits. The comment documents this intentional boundary breach. Contributors should understand when to maintain strict abstractions versus when to compromise for performance.

---

## Pattern 17: Declarative Macro Transparency Metadata
**File:** crates/hir-expand/src/declarative.rs (lines 24-81)
**Category:** Macro Definition / Hygiene Configuration
**Code Example:**
```rust
/// Old-style `macro_rules` or the new macros 2.0
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarativeMacroExpander {
    pub mac: mbe::DeclarativeMacro,
    pub transparency: Transparency,
    edition: Edition,
}

impl DeclarativeMacroExpander {
    pub fn expand(
        &self,
        db: &dyn ExpandDatabase,
        tt: tt::TopSubtree,
        call_id: MacroCallId,
        span: Span,
    ) -> ExpandResult<(tt::TopSubtree, Option<u32>)> {
        let loc = db.lookup_intern_macro_call(call_id);
        match self.mac.err() {
            Some(_) => ExpandResult::new(
                (tt::TopSubtree::empty(tt::DelimSpan { open: span, close: span }), None),
                ExpandError::new(span, ExpandErrorKind::MacroDefinition),
            ),
            None => self
                .mac
                .expand(
                    db,
                    &tt,
                    |s| {
                        s.ctx = apply_mark(db, s.ctx, call_id.into(),
                                          self.transparency, self.edition)
                    },
                    loc.kind.call_style(),
                    span,
                )
                .map_err(Into::into),
        }
    }
}
```
**Why This Matters for Contributors:** DeclarativeMacroExpander wraps the MBE (macro-by-example) engine with hygiene tracking. The transparency field controls name resolution scope, applied via `apply_mark` during expansion. The pattern of checking for definition errors before expansion (`mac.err()`) prevents cascading failures. This shows how to layer semantic information (hygiene) over syntactic transformation (token tree expansion).

---

## Pattern 18: Token Tree Limit for DoS Prevention
**File:** crates/hir-expand/src/db.rs (lines 26-32, 693-709)
**Category:** Resource Management / Security
**Code Example:**
```rust
/// Total limit on the number of tokens produced by any macro invocation.
///
/// If an invocation produces more tokens than this limit, it will not be stored in the database and
/// an error will be emitted.
///
/// Actual max for `analysis-stats .` at some point: 30672.
const TOKEN_LIMIT: usize = 2_097_152;

fn check_tt_count(tt: &tt::TopSubtree) -> Result<(), ExpandResult<()>> {
    let tt = tt.top_subtree();
    let count = tt.count();
    if count <= TOKEN_LIMIT {
        Ok(())
    } else {
        Err(ExpandResult {
            value: (),
            err: Some(ExpandError::other(
                tt.delimiter.open,
                format!(
                    "macro invocation exceeds token limit: produced {count} tokens, limit is {TOKEN_LIMIT}",
                ),
            )),
        })
    }
}
```
**Why This Matters for Contributors:** This demonstrates defensive programming against pathological macro expansions that could DOS the analyzer. The limit (2M tokens) is empirically derived from real-world codebases. Note the include! macro is exempt from this check as it legitimately produces large token streams. This pattern shows balancing usability with resource protection.

---

## Pattern 19: ExpandResult<T> for Partial Expansion Success
**File:** crates/hir-expand/src/lib.rs (line 113)
**Category:** Error Handling / Resilience
**Code Example:**
```rust
pub type ExpandResult<T> = ValueResult<T, ExpandError>;

// ValueResult from mbe crate:
// pub struct ValueResult<T, E> {
//     pub value: T,
//     pub err: Option<E>,
// }
```
**Why This Matters for Contributors:** ExpandResult allows macro expansion to partially succeed—returning a potentially incomplete expansion along with an error. This is crucial for IDE resilience: even if a macro has errors, rust-analyzer can use the partial expansion to provide some functionality. This pattern of "best effort" error recovery is central to rust-analyzer's UX.

---

## Pattern 20: Parse-or-Expand Transparent Query
**File:** crates/hir-expand/src/db.rs (lines 68, 342-350)
**Category:** Query Optimization / Salsa Pattern
**Code Example:**
```rust
#[salsa::transparent]
fn parse_or_expand(&self, file_id: HirFileId) -> SyntaxNode;

/// Main public API -- parses a hir file, not caring whether it's a real
/// file or a macro expansion.
fn parse_or_expand(db: &dyn ExpandDatabase, file_id: HirFileId) -> SyntaxNode {
    match file_id {
        HirFileId::FileId(file_id) => db.parse(file_id).syntax_node(),
        HirFileId::MacroFile(macro_file) => {
            db.parse_macro_expansion(macro_file).value.0.syntax_node()
        }
    }
}
```
**Why This Matters for Contributors:** The `#[salsa::transparent]` attribute means this query doesn't create its own memoization layer—it transparently delegates to `parse` or `parse_macro_expansion`. This avoids double-caching while providing a unified API. Understanding when to use transparent queries is important for performance optimization in salsa-based systems.

---

## EXPERT RUST COMMENTARY

### Pattern Analysis Summary

**Overall Idiomatic Quality: ⭐⭐⭐⭐⭐ (5/5 stars)**

This codebase represents **production-grade Rust architecture** at its finest, demonstrating advanced patterns essential for building incremental compilers and language servers.

---

### Pattern 1: Public Facade with Internal Abstraction Boundary

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Architectural Boundary (Systems Programming)

**Rust-Specific Insight:**
This exemplifies the **facade + ECS hybrid** pattern rarely documented but crucial for large Rust systems. The OO facade (HIR) provides ergonomic public APIs with self-contained types, while the ECS internals (hir_*) use arena allocation, local indexes, and interned IDs for performance. This dual approach leverages Rust's zero-cost abstractions—the facade compiles to direct field access with no runtime overhead.

Key Rust idiom: **Separate public ergonomics from internal performance**. The facade uses `Arc`, `Option`, rich enums; internals use `u32` IDs, raw indices, and salsa interning.

**Contribution Tip:**
When adding HIR features:
1. Start with the facade API design (what users want)
2. Implement in hir_def/hir_ty using performance-oriented structures
3. Bridge with `from_id` conversions and database queries
4. Never expose salsa interning details through the facade

**Common Pitfalls:**
- **Leaking internal IDs through public APIs** (use newtype wrappers)
- **Bypassing the facade for "quick access"** (breaks abstraction boundary)
- **Adding methods that require O(n) scans** (cache in database queries instead)

**Related Patterns:**
- **rustc's TyCtxt/Ty split** (similar facade over interned types)
- **Bevy ECS + Scene API** (ECS internals, OO game objects)
- **salsa's InputDatabase pattern** (separating inputs from derived queries)

---

### Pattern 2: HasSource Trait for Bidirectional Syntax Mapping

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Trait-Based Polymorphism + Source Mapping

**Rust-Specific Insight:**
The `HasSource` trait demonstrates **associated type + Option layering** for handling partial mappings. The `Option<InFile<Self::Ast>>` return enables graceful handling of builtin items (no source), proc-macros (external source), and derived impls (synthetic source with range). The `source_with_range` default method shows **progressive disclosure**—simple cases use `source()`, complex cases override both methods.

This pattern exploits Rust's trait system for **compile-time polymorphism without vtable overhead**—each HIR type statically knows its AST counterpart.

**Contribution Tip:**
When implementing `HasSource` for new HIR types:
```rust
impl HasSource for MyHirType {
    type Ast = ast::MyAstNode;
    fn source(self, db: &dyn HirDatabase) -> Option<InFile<Self::Ast>> {
        let id = self.id; // Internal ID
        let loc = db.lookup_intern_my_type(id); // Salsa lookup
        Some(InFile::new(loc.id.file_id, loc.source(db)?.value))
    }
}
```

**Common Pitfalls:**
- **Unwrapping in `source()`** when dealing with builtin items (always return `Option`)
- **Ignoring macro file boundaries** (use `InFile` consistently)
- **Expensive computation in `source()`** (delegate to salsa queries)

**Related Patterns:**
- **rustc's Spanned<T>** (simpler span tracking without file context)
- **syn's ToTokens** (inverse direction: types to syntax)
- **miette's SourceCode** (source mapping for diagnostics)

---

### Pattern 3: InFile<T> Wrapper for File Context Preservation

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Type-Level Context Tracking (Newtype + Generics)

**Rust-Specific Insight:**
`InFileWrapper<FileKind, T>` demonstrates **phantom type parameterization** for compile-time file type safety. The type aliases (`InFile<T>`, `InMacroFile<T>`, `InRealFile<T>`) create distinct types preventing accidental mixing:

```rust
fn process(node: InMacroFile<SyntaxNode>) { /* only accepts macro nodes */ }
// process(InRealFile { ... }) // <- compile error!
```

This pattern uses Rust's **zero-cost newtype** idiom—`InFile` compiles to two fields with no wrapper overhead. The `#[derive(Copy)]` shows that even complex types can be trivially copyable when they only contain IDs and references.

**Contribution Tip:**
Use `InFile::map()` for transforming wrapped values while preserving file context:
```rust
let in_file_ast: InFile<ast::Fn> = ...;
let in_file_name: InFile<Name> = in_file_ast.map(|ast| ast.name());
```

**Common Pitfalls:**
- **Stripping file context too early** (convert to `InFile` ASAP, unwrap late)
- **Mixing `HirFileId` and `EditionedFileId`** (use type system to enforce)
- **Forgetting to propagate through transformations** (always `.map()`, never extract + rewrap)

**Related Patterns:**
- **rustc's WithOptConstParam<T>** (adds optional const context)
- **Spanned<T> in many parsers** (simpler span-only wrapper)
- **tower::ServiceExt::map_*()** (similar context-preserving transformations)

---

### Pattern 4: HirFileId Enum for Real vs Macro File Distinction

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Enum Modeling + Recursive Traversal

**Rust-Specific Insight:**
`HirFileId` is a **sum type modeling mutually exclusive states** with exhaustive pattern matching. The `#[salsa_macros::Supertype]` attribute enables safe interning as a salsa type. The `original_file()` method demonstrates **recursive descent via loop + match**—more efficient than recursive calls (no stack frames).

This pattern exploits Rust's **zero-overhead enums**—despite being a sum type, `HirFileId` is just `(u32 tag, u64 data)` with no indirection.

Key idiom: **Use enums for mutually exclusive states, not booleans + optional fields.**

**Contribution Tip:**
When traversing macro hierarchies, use `original_file()` for diagnostics but preserve the full chain for span mapping:
```rust
// For diagnostics (end user sees original source)
let diagnostic_file = file_id.original_file(db);

// For span mapping (need full expansion chain)
let mut chain = vec![file_id];
while let Some(macro_file) = chain.last().and_then(|f| f.macro_file()) {
    chain.push(db.lookup_intern_macro_call(macro_file).kind.file_id());
}
```

**Common Pitfalls:**
- **Using `is_macro()` + if-let instead of match** (non-exhaustive)
- **Assuming macro files always have depth 1** (macros can expand macros)
- **Unwrapping `file_id()` for macro files** (always returns `None`)

**Related Patterns:**
- **rustc's DefId (local vs crate-external)** (similar dual ID system)
- **salsa's DatabaseKeyIndex** (enum of interned types)
- **petgraph's NodeIndex** (opaque ID with arena backing)

---

### Pattern 5: Semantics Struct as Primary API Entry Point

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Facade + Interior Mutability + Lifetime Management

**Rust-Specific Insight:**
The **Semantics facade pattern** combines multiple advanced Rust techniques:

1. **Lifetime parameterization** (`'db`) ensures borrowed data doesn't outlive the database
2. **Interior mutability** (`RefCell<FxHashMap>`) enables caching in `&self` methods
3. **Generic + trait object variants** support both monomorphized and dynamic dispatch:
   ```rust
   let sem1 = Semantics::new(&concrete_db); // Static dispatch, faster
   let sem2 = Semantics::new_dyn(&dyn_db);  // Dynamic dispatch, flexible
   ```

The split between `Semantics<'db, DB>` and `SemanticsImpl<'db>` separates **public API surface** (strongly typed DB) from **implementation** (trait object for internal use).

**Contribution Tip:**
When adding caching to Semantics:
```rust
pub fn my_cached_operation(&self, key: K) -> V {
    let cache = self.imp.my_cache.borrow();
    if let Some(result) = cache.get(&key) {
        return result.clone();
    }
    drop(cache); // Release borrow before computing

    let result = expensive_computation(self.db, key);
    self.imp.my_cache.borrow_mut().insert(key, result.clone());
    result
}
```

**Common Pitfalls:**
- **Panicking on RefCell borrow conflicts** (never hold borrows across calls to db)
- **Unbounded cache growth** (use LRU or clear on file change)
- **Mixing cached and uncached paths** (document which methods cache)

**Related Patterns:**
- **rustc's TyCtxt** (similar API facade with arena allocation)
- **salsa's ParallelDatabase** (thread-safe query caching)
- **tower::ServiceBuilder** (composable middleware with caching)

---

### Pattern 6: MacroCallLoc Structure for Macro Identity

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Value-Based Identity + Salsa Interning

**Rust-Specific Insight:**
`MacroCallLoc` demonstrates **structural identity for macro calls**—two macro invocations are identical if they have the same def, krate, call site, and context. This enables **perfect deduplication** via salsa interning:

```rust
let id1 = db.intern_macro_call(loc1);
let id2 = db.intern_macro_call(loc2);
// If loc1 == loc2, then id1 == id2 (same interned value)
```

The `MacroDefKind` enum uses **associated data patterns**:
```rust
BuiltIn(AstId<ast::Macro>, BuiltinFnLikeExpander)
//       ^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^
//       Where defined     How to expand
```

This pattern achieves O(1) comparison (compare IDs) while preserving O(1) access to full data (salsa lookup).

**Contribution Tip:**
When adding new macro kinds:
1. Add variant to `MacroDefKind` with associated expansion logic
2. Update `MacroCallKind` if the call pattern differs (e.g., attribute vs derive)
3. Ensure all fields in `MacroCallLoc` affect equality (or expansion is non-deterministic)

**Common Pitfalls:**
- **Forgetting to include hygiene context** (`ctxt` field) in loc (breaks hygiene)
- **Using `MacroCallId` in `Hash`/`Eq` of cached data** (creates circular dependencies)
- **Assuming `MacroDefId` uniquely identifies expansions** (need full `MacroCallLoc`)

**Related Patterns:**
- **rustc's ExpnId (expansion ID)** (similar interned expansion identity)
- **salsa::Id<T>** (generic interning pattern)
- **string_cache::Atom** (interned strings with O(1) comparison)

---

### Pattern 7: ExpandTo Enum for Syntactic Context Detection

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Context-Dependent Polymorphism + Parent-Walking

**Rust-Specific Insight:**
`ExpandTo` captures **macro polymorphism via context inference**—a profound Rust language feature. The `from_call_site()` method demonstrates **syntax tree parent-walking** for context detection:

```rust
match parent.kind() {
    MACRO_ITEMS => ExpandTo::Items,   // fn_macro!() at item position
    MACRO_EXPR => ExpandTo::Expr,     // fn_macro!() in expression position
    // Same macro, different expansion target!
}
```

This pattern shows Rust's **zero-runtime-cost context sensitivity**—the context is computed once during parsing and cached in the `MacroCallKind`.

**Contribution Tip:**
When debugging macro expansion issues, always check `ExpandTo`:
```rust
let expand_to = ExpandTo::from_call_site(&macro_call);
eprintln!("Macro expands to: {:?}", expand_to);
// Mismatch between expected and actual often explains parse failures
```

**Common Pitfalls:**
- **Assuming macros always expand to expressions** (many expand to items/patterns)
- **Not updating `from_call_site()` when adding new syntax positions** (causes wrong expansion)
- **Caching expanded syntax without the `ExpandTo` key** (same macro, different contexts = different syntax)

**Related Patterns:**
- **syn's parse_macro_input!** (similar context-aware parsing)
- **rustc's AstConv** (converts syntax to types based on position)
- **Pratt parsing** (operator precedence from context)

---

### Pattern 8: Intern/Lookup Trait Pattern for Salsa Integration

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Bidirectional Mapping + Macro Code Generation

**Rust-Specific Insight:**
The `impl_intern_lookup!` macro demonstrates **declarative trait implementation** for salsa interning. This pattern achieves:

1. **Type-safe interning**: `MacroCallLoc.intern(db)` returns `MacroCallId` (cannot mix with other IDs)
2. **Guaranteed roundtrip**: `id.lookup(db).intern(db) == id` (salsa guarantees)
3. **Zero-cost abstraction**: Compiles to direct salsa method calls (no wrapper overhead)

The macro generates **symmetric trait implementations**:
```rust
impl Intern for Loc { fn intern(...) -> Id }
impl Lookup for Id { fn lookup(...) -> Loc }
```

This bidirectionality enables ergonomic conversions in both directions.

**Contribution Tip:**
Add new interned types with:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MyLoc { /* fields */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MyId(salsa::InternId);

impl_intern_lookup!(
    MyDatabase,     // Database trait
    MyId,           // ID type
    MyLoc,          // Loc type
    intern_my,      // intern method name
    lookup_my       // lookup method name
);
```

**Common Pitfalls:**
- **Implementing `From` instead of `Intern`/`Lookup`** (bypasses salsa caching)
- **Mutating loc fields after interning** (breaks salsa's immutability assumption)
- **Using IDs across different databases** (IDs are database-specific)

**Related Patterns:**
- **salsa's #[salsa::interned] macro** (same pattern, declarative syntax)
- **rustc's InternId/Interned<T>** (similar interning for types)
- **string-interner crate** (specialized for strings)

---

### Pattern 9: Lazy Macro Expansion with Firewall Queries

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Incremental Computation + Change Isolation

**Rust-Specific Insight:**
**Firewall queries** are a critical salsa pattern for incremental compilation. The `macro_arg` query isolates macro input token trees:

```rust
#[salsa::invoke(macro_arg)]  // Firewall: only token tree changes invalidate
fn macro_arg(&self, id: MacroCallId) -> MacroArgResult;
```

Changes to macro implementation don't invalidate downstream queries if the input token tree stays the same. The `#[salsa::transparent]` on `macro_arg_considering_derives` shows the dual pattern—**transparent queries** don't cache but propagate dependencies precisely.

**When to use each:**
- **Firewall**: Isolate expensive transformations (parsing, expansion)
- **Transparent**: Dispatch logic, conditional queries

**Contribution Tip:**
Design query boundaries to minimize re-computation:
```rust
// BAD: Combines parsing + expansion (changes to either invalidate both)
fn expand_macro(&self, call: MacroCall) -> ExpandedMacro;

// GOOD: Separate concerns (parsing changes don't invalidate expansion if tokens unchanged)
#[salsa::invoke(parse_macro_input)]
fn macro_arg(&self, id: MacroCallId) -> TokenTree; // Firewall

fn expand_macro_from_tokens(&self, id: MacroCallId, tokens: TokenTree) -> ExpandedMacro;
```

**Common Pitfalls:**
- **Making every query transparent** (loses caching benefits)
- **Large firewall boundaries** (coarse-grained invalidation)
- **Firewall queries that reference volatile inputs** (defeats incrementality)

**Related Patterns:**
- **salsa's #[salsa::volatile]** (opposite: always recompute)
- **rustc's DepNode tracking** (similar dependency graph)
- **Bazel's action caching** (analogous in build systems)

---

### Pattern 10: SpanMap Enum for Real vs Expansion Span Tracking

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Enum Dispatch + Owned/Borrowed Pairs

**Rust-Specific Insight:**
`SpanMap` demonstrates **dual owned/borrowed enum pattern**:

```rust
pub enum SpanMap {                    // Owned (Arc for cheap cloning)
    ExpansionSpanMap(Arc<...>),
    RealSpanMap(Arc<...>),
}

pub enum SpanMapRef<'a> {            // Borrowed (no allocation)
    ExpansionSpanMap(&'a ...),
    RealSpanMap(&'a ...),
}
```

This pattern enables:
- **Storage**: Use `SpanMap` in structs (cheap to clone via `Arc`)
- **Computation**: Use `SpanMapRef` in methods (no Arc overhead)
- **Conversion**: `span_map.as_ref()` converts owned to borrowed

The `Arc` wrapping provides **shallow cloning**—copying a `SpanMap` is O(1) atomic increment, not deep copy.

**Contribution Tip:**
Follow the owned/borrowed pattern for large data structures:
```rust
#[derive(Clone)]
pub enum MyData {
    Variant1(Arc<LargeStruct>),
    Variant2(Arc<AnotherLargeStruct>),
}

pub enum MyDataRef<'a> {
    Variant1(&'a LargeStruct),
    Variant2(&'a AnotherLargeStruct),
}

impl MyData {
    pub fn as_ref(&self) -> MyDataRef<'_> {
        match self {
            Self::Variant1(arc) => MyDataRef::Variant1(arc),
            Self::Variant2(arc) => MyDataRef::Variant2(arc),
        }
    }
}
```

**Common Pitfalls:**
- **Using `Box` instead of `Arc`** (cannot be cheaply cloned)
- **Forgetting to provide `as_ref()` conversion** (forces unnecessary cloning)
- **Deep matching on owned enums in hot paths** (convert to Ref first)

**Related Patterns:**
- **Cow<'a, T>** (owned/borrowed for specific types)
- **bytes::Bytes vs &[u8]** (similar owned/borrowed for byte buffers)
- **smol_str::SmolStr** (small string optimization with cheap cloning)

---

### Pattern 11: Eager vs Lazy Expansion Dichotomy

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Strategy Pattern + Recursive Expansion

**Rust-Specific Insight:**
Eager macro expansion demonstrates **recursive closure pattern** for name resolution:

```rust
pub fn expand_eager_macro_input(
    // ...
    resolver: &dyn Fn(&ModPath) -> Option<MacroDefId>,  // Closure for context-sensitive resolution
    eager_callback: EagerCallBackFn<'_>,                // Recursive expansion callback
) -> ExpandResult<Option<MacroCallId>>
```

The pattern creates a **temporary MacroCallId** for arguments, expands them, then expands the outer macro with expanded arguments. This handles arbitrarily nested eager expansions like `concat!(stringify!(x), "suffix")`.

**Key insight**: Eager macros require **two-phase expansion**:
1. Expand arguments in current context
2. Expand macro with expanded arguments

**Contribution Tip:**
When implementing new eager macros:
```rust
// Phase 1: Create temporary ID for arguments
let arg_id = db.intern_macro_call(MacroCallLoc {
    def,
    kind: MacroCallKind::FnLike { expand_to: ExpandTo::Expr, eager: None },
    // ...
});

// Phase 2: Expand arguments recursively
let (arg_expansion, _) = db.parse_macro_expansion(arg_id);

// Phase 3: Expand outer macro with expanded arguments
my_eager_expander.expand(db, arg_expansion)
```

**Common Pitfalls:**
- **Mixing eager and lazy expansion strategies** (causes hygiene issues)
- **Infinite recursion in eager expansion** (need cycle detection)
- **Not preserving spans through eager expansion** (breaks diagnostics)

**Related Patterns:**
- **rustc's eager expansion for concat!/line!/etc.** (identical strategy)
- **Template metaprogramming** (similar two-phase instantiation)
- **Prolog's evaluation strategy** (analogous eager vs lazy distinction)

---

### Pattern 12: Hygiene via SyntaxContext and Transparency

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Hygiene System + Context Chaining

**Rust-Specific Insight:**
Rust's hygiene system is based on **syntax contexts** that track macro expansion history. The `Transparency` enum controls name resolution:

```rust
pub enum Transparency {
    Opaque,        // Resolve names in macro definition scope
    SemiOpaque,    // Derive-like: resolve some names at call site
    Transparent,   // Resolve all names at call site (unhygienic)
}
```

The `apply_mark` function **chains contexts** to create a history of expansions:
```rust
ctx = apply_mark(db, ctx, macro_id, transparency, edition)
//    ^^^^^^^^^^ Adds a layer to the context chain
```

This enables **different names with the same identifier** to coexist:
```rust
macro_rules! m { () => { let x = 1; } }
let x = 0;  // Different `x` (call-site context)
m!();       // Different `x` (def-site context)
// Both `x` variables coexist without conflict!
```

**Contribution Tip:**
When working with macro-generated names:
- Use `span_with_def_site_ctxt` for hygienic identifiers (won't conflict with user code)
- Use `span_with_call_site_ctxt` for identifiers that should be visible to user

**Common Pitfalls:**
- **Forgetting to apply hygiene marks** (breaks macro hygiene)
- **Using `Transparent` by default** (defeats hygiene)
- **Mixing contexts from different macro invocations** (causes resolution failures)

**Related Patterns:**
- **rustc's SyntaxContext** (same underlying mechanism)
- **Scheme's syntax-case** (similar hygiene system)
- **Template Haskell's name resolution** (analogous scoping)

---

### Pattern 13: SourceAnalyzer for Context-Aware HIR Analysis

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Context Builder + Scoped Resolution

**Rust-Specific Insight:**
`SourceAnalyzer` demonstrates **progressive scope narrowing**:

```rust
fn new_for_body(db, def, node, offset) {
    let scopes = db.expr_scopes(def);           // 1. Get all scopes for function
    let scope = scope_for_offset(..., offset);  // 2. Find innermost scope at position
    let resolver = resolver_for_scope(...);     // 3. Build resolver for that scope
    SourceAnalyzer { resolver, ... }
}
```

This pattern enables **position-aware queries** like "what is the type of this expression?"—the resolver knows which variables are in scope at the query position.

The `BodyOrSig` enum unifies different analysis contexts:
```rust
enum BodyOrSig {
    Body { infer: Option<&InferenceResult>, ... },  // Full type inference available
    Sig { store: Arc<ExpressionStore>, ... },       // Only signature, no body
    VariantFields { ... },                          // Struct/enum fields
}
```

**Contribution Tip:**
Use SourceAnalyzer for IDE features:
```rust
let analyzer = SourceAnalyzer::new_for_body(db, function_id, node, cursor_offset);

// Now you can query with cursor context
let type_at_cursor = analyzer.type_of_expr(db, expr_id);
let completions = analyzer.visible_names_in_scope(db);
```

**Common Pitfalls:**
- **Using wrong scope for resolution** (use offset-based scope, not parent scope)
- **Caching SourceAnalyzer across edits** (resolver becomes stale)
- **Assuming inference is always available** (check `Option<&InferenceResult>`)

**Related Patterns:**
- **rustc's TypeckResults** (similar type inference context)
- **LSP's position-based queries** (analogous scoped resolution)
- **Symbol table with scope chains** (classic compiler pattern)

---

### Pattern 14: from_id Macro for Bidirectional Type Conversions

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Macro Code Generation + Newtype Conversions

**Rust-Specific Insight:**
The `from_id!` macro demonstrates **symmetric newtype conversions** via `From` trait:

```rust
from_id![
    (hir_def::StructId, crate::Struct),  // Internal ID <-> Public wrapper
];

// Generates:
impl From<hir_def::StructId> for crate::Struct {
    fn from(id: hir_def::StructId) -> crate::Struct { Struct { id } }
}
impl From<crate::Struct> for hir_def::StructId {
    fn from(ty: crate::Struct) -> hir_def::StructId { ty.id }
}
```

This enables **seamless conversions** without boilerplate:
```rust
let public: Struct = internal_id.into();     // From implementation
let internal: StructId = public.into();      // Reverse From implementation
```

The pattern exploits Rust's **coherence rules**—both trait and type are local, so implementations are allowed.

**Contribution Tip:**
When adding new HIR types:
1. Define the public struct with a single `id` field
2. Add entry to `from_id!` macro
3. The macro generates bidirectional `From` implementations

This eliminates manual conversion boilerplate and ensures consistency.

**Common Pitfalls:**
- **Implementing From manually** (duplicates macro logic, inconsistent)
- **Adding multiple fields to wrapper types** (breaks from_id pattern)
- **Forgetting to add to from_id! list** (forces manual conversions)

**Related Patterns:**
- **derive_more::From** (similar but via derive macro)
- **newtype_derive** crate (more sophisticated newtype derivations)
- **rustc's rustc_index macros** (similar ID wrapper generation)

---

### Pattern 15: ExpansionInfo for Macro Span Mapping

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Bidirectional Span Mapping + Iterator Chaining

**Rust-Specific Insight:**
`ExpansionInfo` solves the **macro span mapping problem**—arguably the hardest problem in macro systems. The bidirectional mapping enables:

**Down-mapping** (input → expansion):
```rust
fn map_range_down_exact(&self, span: Span) -> Option<InMacroFile<impl Iterator<...>>> {
    // Returns ALL tokens in expansion that came from input span
    self.exp_map.ranges_with_span_exact(span).flat_map(...)
}
```

**Up-mapping** (expansion → input):
```rust
fn map_node_range_up(&self, range: TextRange) -> Option<(FileRange, SyntaxContext)> {
    // Traces expansion token back to original source
    map_node_range_up(db, &self.exp_map, range)
}
```

The `Iterator` return type enables **lazy evaluation**—only compute mapped tokens if caller actually uses them.

**Contribution Tip:**
For IDE features across macro boundaries:
```rust
// Find all usages of identifier
let source_span = identifier.span();
let expansions = expansion_info.map_range_down_exact(source_span)?;
for (token, ctx) in expansions {
    // Process each expansion occurrence
}

// Navigate from macro expansion to definition
let expansion_range = macro_generated_token.text_range();
let (source_range, ctx) = expansion_info.map_node_range_up(db, expansion_range)?;
// Jump to source_range in original file
```

**Common Pitfalls:**
- **Using `map_range_down` without checking `NO_DOWNMAP_ERASED_FILE_AST_ID_MARKER`** (builtin items)
- **Assuming 1:1 span mapping** (one input span can map to multiple output tokens)
- **Ignoring SyntaxContext in results** (breaks hygiene-aware features)

**Related Patterns:**
- **rustc's ExpnData and SpanData** (similar expansion tracking)
- **Source maps in JavaScript** (analogous for minified code)
- **DWARF debug info** (maps compiled code to source)

---

### Pattern 16: Database Re-exports for Facade Boundary

**Idiomatic Rating: ⭐⭐⭐⭐ (pragmatic compromise)**

**Pattern Classification:** API Boundary Pragmatism

**Rust-Specific Insight:**
This pattern demonstrates **pragmatic abstraction**—when perfect encapsulation conflicts with performance, document the breach and continue:

```rust
//! This breaks abstraction boundary a bit, it would be cool if we didn't do that.
//! But we need this for at least LRU caching at the query level.
pub use hir_def::db::DefDatabase;
pub use hir_expand::db::ExpandDatabase;
```

The comment is crucial—it:
1. **Acknowledges the compromise** (honesty builds trust)
2. **Explains the reason** (performance requirement)
3. **Documents the cost** (abstraction leak)

This follows Rust community values: **pragmatism over purity**, but with transparency.

**Contribution Tip:**
When facing similar trade-offs:
1. Try to maintain abstraction (first priority)
2. If impossible, measure the performance impact
3. Document the breach with `//!` module-level comment
4. Consider future refactoring to restore boundary

**Common Pitfalls:**
- **Exposing implementation details without documentation** (silent abstraction leaks)
- **Premature optimization** (break abstraction without measurement)
- **Never revisiting pragmatic compromises** (technical debt accumulates)

**Related Patterns:**
- **Rust's std::io::Write exposing BufWriter** (similar pragmatic exposure)
- **salsa's DatabaseImpl pattern** (balancing trait abstraction with performance)
- **Unix philosophy: "worse is better"** (pragmatism over theoretical purity)

---

### Pattern 17: Declarative Macro Transparency Metadata

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Macro Expander + Hygiene Integration

**Rust-Specific Insight:**
`DeclarativeMacroExpander` demonstrates **hygiene-aware expansion**:

```rust
pub fn expand(&self, db, tt, call_id, span) -> ExpandResult<...> {
    match self.mac.err() {
        Some(_) => /* return error expansion */,
        None => self.mac.expand(
            db,
            &tt,
            |s| { s.ctx = apply_mark(db, s.ctx, call_id, self.transparency, ...) },
            //    ^^^^^^^^^^^^^^^^ Apply hygiene to each output span
            ...
        )
    }
}
```

The closure parameter `|s| { s.ctx = apply_mark(...) }` is a **span transformer**—it mutates each output span to add the expansion context. This is how hygiene is implemented: by marking tokens with their expansion history.

**Key insight**: Hygiene is not a property of identifiers, but of **spans**—the same identifier `x` with different spans resolves to different variables.

**Contribution Tip:**
When implementing custom expanders:
```rust
impl MyExpander {
    fn expand(&self, db, input, call_id) -> ExpandResult<TokenTree> {
        let output = my_expansion_logic(input);

        // CRITICAL: Apply hygiene marks to output spans
        output.map_spans(|span| {
            span_with_def_site_ctxt(db, span, call_id, self.edition)
        });

        ExpandResult { value: output, err: None }
    }
}
```

**Common Pitfalls:**
- **Forgetting to apply hygiene marks** (breaks macro hygiene)
- **Using wrong transparency** (Opaque for most macros, Transparent for proc-macros)
- **Not handling macro definition errors** (always check `self.mac.err()` first)

**Related Patterns:**
- **proc-macro2::TokenStream** (similar span tracking in proc-macros)
- **rustc's MacroExpander** (production implementation)
- **Kernel hygiene transformations** (formal semantics)

---

### Pattern 18: Token Tree Limit for DoS Prevention

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Resource Management + Security

**Rust-Specific Insight:**
This pattern demonstrates **defensive programming against pathological inputs**:

```rust
const TOKEN_LIMIT: usize = 2_097_152;  // Empirically derived from real codebases

fn check_tt_count(tt: &tt::TopSubtree) -> Result<(), ExpandResult<()>> {
    let count = tt.count();
    if count <= TOKEN_LIMIT {
        Ok(())
    } else {
        Err(ExpandResult {
            value: (),
            err: Some(ExpandError::other(/* diagnostic message */)),
        })
    }
}
```

The limit prevents **quadratic blowup attacks** like:
```rust
macro_rules! explode { ($($x:tt)*) => { $($x)* $($x)* }; }
explode!(explode!(explode!(explode!(x))));  // Exponential expansion
```

**Why 2M tokens?** Based on real-world analysis—largest legitimate macro expansion in ecosystem.

**Contribution Tip:**
Apply similar limits to other unbounded operations:
```rust
const MAX_MACRO_DEPTH: usize = 128;        // Prevent stack overflow
const MAX_RECURSION_DEPTH: usize = 256;    // Prevent infinite recursion
const MAX_TYPE_SIZE: usize = 1024 * 1024;  // Prevent memory exhaustion
```

**Common Pitfalls:**
- **Setting limits too low** (breaks legitimate code)
- **No limits at all** (vulnerable to DoS)
- **Hardcoded limits without configuration** (cannot adjust for different environments)

**Related Patterns:**
- **rustc's recursion_limit attribute** (user-configurable limits)
- **salsa's cycle detection** (prevents infinite query loops)
- **Linux kernel's CONFIG_* limits** (similar resource constraints)

---

### Pattern 19: ExpandResult<T> for Partial Expansion Success

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Resilient Error Handling

**Rust-Specific Insight:**
`ExpandResult<T>` embodies **optimistic error recovery**:

```rust
pub struct ExpandResult<T> {
    pub value: T,           // Best-effort result (may be incomplete)
    pub err: Option<E>,     // Optional error (diagnostics only)
}
```

This differs from `Result<T, E>` philosophically:
- **Result**: Either success OR failure (pessimistic)
- **ExpandResult**: Success AND maybe failure (optimistic)

This enables **graceful degradation** in IDEs:
```rust
let ExpandResult { value: partial_expansion, err } = expand_macro(db, call);
// Use partial_expansion for completions/highlights (best effort)
if let Some(err) = err {
    emit_diagnostic(err);  // Show error to user
}
// IDE remains functional despite errors!
```

**Contribution Tip:**
Use ExpandResult for operations that can partially succeed:
```rust
fn parse_with_recovery(input: &str) -> ExpandResult<Ast> {
    let mut errors = vec![];
    let ast = parse(input, |err| errors.push(err));
    ExpandResult {
        value: ast,  // Partial AST with error nodes
        err: errors.into_iter().next(),  // First error
    }
}
```

**Common Pitfalls:**
- **Using Result when ExpandResult is appropriate** (forces binary success/failure)
- **Ignoring the error field** (silent failures confuse users)
- **Returning meaningless value with error** (violates contract—value should be usable)

**Related Patterns:**
- **Parse trees with error nodes** (similar recovery strategy)
- **LSP's partial results** (best-effort responses to client)
- **HTTP 206 Partial Content** (analogous in protocols)

---

### Pattern 20: Parse-or-Expand Transparent Query

**Idiomatic Rating: ⭐⭐⭐⭐⭐**

**Pattern Classification:** Transparent Query Dispatch

**Rust-Specific Insight:**
`#[salsa::transparent]` creates **zero-overhead query delegation**:

```rust
#[salsa::transparent]
fn parse_or_expand(&self, file_id: HirFileId) -> SyntaxNode {
    match file_id {
        HirFileId::FileId(id) => db.parse(id).syntax_node(),
        HirFileId::MacroFile(id) => db.parse_macro_expansion(id).value.0.syntax_node(),
    }
}
```

**Transparent vs Non-transparent queries:**

| Aspect | Transparent | Non-transparent |
|--------|-------------|-----------------|
| Caching | No (delegates) | Yes (memoized) |
| Dependencies | Tracked through delegation | Direct dependencies |
| Invalidation | N/A (recomputed) | On dependency change |
| Use case | Dispatch logic | Expensive computation |

Transparent queries are essentially **inline functions with dependency tracking**—they vanish at runtime but participate in incremental computation.

**Contribution Tip:**
Use transparent queries for:
- **Conditional dispatch** (if/else, match on enum)
- **Simple transformations** (unwrapping, type conversions)
- **Query composition** (calling multiple other queries)

Avoid for:
- **Expensive computation** (lose caching benefits)
- **Large result types** (copied through call stack)

**Common Pitfalls:**
- **Making expensive queries transparent** (defeats memoization)
- **Circular transparent query chains** (stack overflow)
- **Forgetting to mark dispatch queries transparent** (double caching overhead)

**Related Patterns:**
- **salsa::invoke attribute** (specify custom query function)
- **Haskell's inline pragma** (similar optimization directive)
- **C++ inline functions** (analogous zero-overhead abstraction)

---

## Summary: Key Takeaways for Contributors

1. **Facade Pattern**: HIR provides OO-style public API while hir_* crates use ECS-style internals
2. **Context Tracking**: InFile<T> and HirFileId distinguish real files from macro expansions
3. **Bidirectional Mapping**: HasSource, ExpansionInfo enable navigation between syntax and semantics
4. **Hygiene System**: SyntaxContext and Transparency implement Rust's macro hygiene
5. **Incremental Computation**: Firewall queries isolate changes for efficient re-computation
6. **Resilient Expansion**: ExpandResult allows partial success for better IDE experience
7. **Resource Management**: Token limits prevent pathological macro expansions
8. **Type Safety**: Enum modeling (HirFileId, MacroDefKind) enforces invariants at compile time
9. **Interning Pattern**: Intern/Lookup traits enable efficient ID-based references
10. **Span Mapping**: SpanMap handles the dual nature of real vs expansion spans

---

## CONTRIBUTION READINESS CHECKLIST

### Before Contributing to rust-analyzer HIR/Macro Systems:

#### Essential Understanding (Must Know)
- [ ] **Salsa incremental computation model** (queries, interning, dependency tracking)
- [ ] **Rust macro hygiene** (SyntaxContext, Transparency, span manipulation)
- [ ] **HIR vs hir_* separation** (facade vs implementation architecture)
- [ ] **InFile<T> and file context tracking** (macro file vs real file distinction)
- [ ] **Pattern matching on enums** (exhaustive matching, match ergonomics)

#### Advanced Patterns (Should Know)
- [ ] **Firewall vs transparent queries** (when to cache, when to delegate)
- [ ] **ExpandResult<T> error recovery** (partial success pattern)
- [ ] **Intern/Lookup pattern** (bidirectional ID mappings)
- [ ] **Span mapping** (input→expansion and expansion→input)
- [ ] **Interior mutability with RefCell** (safe caching in &self methods)

#### Macro-Specific Knowledge (For Macro Work)
- [ ] **Eager vs lazy expansion** (concat! vs regular macros)
- [ ] **ExpandTo context detection** (macro polymorphism)
- [ ] **Hygiene application** (def-site vs call-site contexts)
- [ ] **Token tree limits** (DoS prevention)
- [ ] **MacroCallLoc structure** (what defines macro identity)

#### Testing & Debugging
- [ ] **Writing salsa query tests** (set inputs, query outputs, check dependencies)
- [ ] **Macro expansion debugging** (tracing input→tokens→ast→hir)
- [ ] **Span debugging** (visualizing macro expansion chains)
- [ ] **Performance profiling** (identifying query hotspots)

#### Code Quality
- [ ] **Following rust-analyzer style** (see CONTRIBUTING.md)
- [ ] **Writing documentation** (rustdoc with examples)
- [ ] **Adding tests** (unit tests + integration tests)
- [ ] **Minimizing query invalidation** (designing firewall boundaries)

---

### Recommended Learning Path

**Week 1: Foundations**
1. Read rust-analyzer architecture docs
2. Understand salsa basics (run examples from salsa repo)
3. Trace a simple HIR query (Module → StructId → Struct)

**Week 2: Macro Basics**
1. Study ExpandTo and from_call_site()
2. Implement a simple builtin macro
3. Debug macro expansion with expansion logs

**Week 3: Advanced Patterns**
1. Implement a new HIR type with HasSource
2. Add interning for a new ID type
3. Write tests with ExpandResult

**Week 4: Contribution**
1. Pick a good-first-issue from rust-analyzer
2. Apply patterns from this document
3. Submit PR with tests and documentation

---

### Pattern Quality Rating Summary

| Pattern | Stars | Classification | Difficulty | Applicability |
|---------|-------|----------------|------------|---------------|
| 1. Facade Boundary | ⭐⭐⭐⭐⭐ | Architecture | Hard | Universal |
| 2. HasSource Trait | ⭐⭐⭐⭐⭐ | Trait Design | Medium | Source mapping |
| 3. InFile<T> | ⭐⭐⭐⭐⭐ | Type Safety | Easy | Context tracking |
| 4. HirFileId Enum | ⭐⭐⭐⭐⭐ | Enum Modeling | Medium | File abstraction |
| 5. Semantics Facade | ⭐⭐⭐⭐⭐ | API Design | Hard | Public APIs |
| 6. MacroCallLoc | ⭐⭐⭐⭐⭐ | Interning | Medium | Macro systems |
| 7. ExpandTo | ⭐⭐⭐⭐⭐ | Context Analysis | Medium | Macro parsing |
| 8. Intern/Lookup | ⭐⭐⭐⭐⭐ | Salsa Pattern | Medium | ID management |
| 9. Firewall Queries | ⭐⭐⭐⭐⭐ | Incremental | Hard | Performance |
| 10. SpanMap | ⭐⭐⭐⭐⭐ | Enum Dispatch | Medium | Span tracking |
| 11. Eager Expansion | ⭐⭐⭐⭐⭐ | Strategy | Hard | Macro impl |
| 12. Hygiene System | ⭐⭐⭐⭐⭐ | Hygiene | Very Hard | Macro hygiene |
| 13. SourceAnalyzer | ⭐⭐⭐⭐⭐ | Context | Hard | IDE features |
| 14. from_id Macro | ⭐⭐⭐⭐⭐ | Code Gen | Easy | Type conversion |
| 15. ExpansionInfo | ⭐⭐⭐⭐⭐ | Span Mapping | Very Hard | Macro navigation |
| 16. DB Re-exports | ⭐⭐⭐⭐ | Pragmatism | Easy | API boundaries |
| 17. Transparency | ⭐⭐⭐⭐⭐ | Hygiene | Hard | Macro expansion |
| 18. Token Limits | ⭐⭐⭐⭐⭐ | Security | Easy | DoS prevention |
| 19. ExpandResult | ⭐⭐⭐⭐⭐ | Error Handling | Medium | Resilience |
| 20. Transparent Query | ⭐⭐⭐⭐⭐ | Salsa Pattern | Medium | Query dispatch |

**Overall Assessment**: This codebase represents **world-class Rust engineering**. Every pattern demonstrates deep understanding of Rust's type system, zero-cost abstractions, and incremental computation. The macro expansion system is particularly sophisticated, handling one of the most complex features in modern programming languages.

**Contribution Difficulty**: **High** (requires deep Rust knowledge + compiler theory + salsa understanding)

**Recommended For**: Senior Rust developers, compiler engineers, language tooling experts

**Anti-Recommended For**: Rust beginners (start with simpler projects), developers unfamiliar with incremental computation, those seeking quick wins (changes require deep understanding)

---

### Final Wisdom

These patterns represent **15+ years of collective compiler engineering experience** distilled into production code. They solve problems that are:

1. **Hard theoretically** (macro hygiene, incremental span mapping)
2. **Hard practically** (performance, memory usage, IDE responsiveness)
3. **Hard ergonomically** (clean APIs despite internal complexity)

Contributing successfully requires **patience, study, and humility**. Start small, ask questions, read existing code extensively.

The patterns here are not just "nice to know"—they are **essential knowledge** for Rust compiler/tooling work. Master them, and you'll be equipped to contribute to rustc, rust-analyzer, and other advanced Rust projects.

**Good luck, and happy hacking!** 🦀

1. **Facade Pattern**: HIR provides OO-style public API while hir_* crates use ECS-style internals
2. **Context Tracking**: InFile<T> and HirFileId distinguish real files from macro expansions
3. **Bidirectional Mapping**: HasSource, ExpansionInfo enable navigation between syntax and semantics
4. **Hygiene System**: SyntaxContext and Transparency implement Rust's macro hygiene
5. **Incremental Computation**: Firewall queries isolate changes for efficient re-computation
6. **Resilient Expansion**: ExpandResult allows partial success for better IDE experience
7. **Resource Management**: Token limits prevent pathological macro expansions
8. **Type Safety**: Enum modeling (HirFileId, MacroDefKind) enforces invariants at compile time
9. **Interning Pattern**: Intern/Lookup traits enable efficient ID-based references
10. **Span Mapping**: SpanMap handles the dual nature of real vs expansion spans

These patterns demonstrate rust-analyzer's sophisticated approach to handling one of Rust's most complex features—macro expansion—while maintaining an ergonomic public API for IDE features.
