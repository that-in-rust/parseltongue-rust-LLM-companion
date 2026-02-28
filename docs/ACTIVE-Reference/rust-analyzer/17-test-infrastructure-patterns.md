# Idiomatic Rust Patterns: Test Infrastructure
> Source: rust-analyzer testing crates and patterns

## Pattern 1: Fixture Mini-DSL with //- Markers
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/test-fixture/src/lib.rs
**Category:** Fixture System
**Code Example:**
```rust
/// A trait for setting up test databases from fixture strings.
///
/// Fixtures are strings containing Rust source code with optional metadata that describe
/// a project setup. This is the primary way to write tests for rust-analyzer without
/// having to depend on the entire sysroot.
///
/// # Fixture Syntax
///
/// ## Basic Structure
///
/// A fixture without metadata is parsed into a single source file (`/main.rs`).
/// Metadata is added after a `//-` comment prefix.
///
/// ```text
/// //- /main.rs
/// fn main() {
///     println!("Hello");
/// }
/// ```
///
/// ## File Metadata
///
/// Each file can have the following metadata after `//-`:
///
/// - **Path** (required): Must start with `/`, e.g., `/main.rs`, `/lib.rs`, `/foo/bar.rs`
/// - **`crate:<name>`**: Defines a new crate with this file as its root
/// - **`deps:<crate1>,<crate2>`**: Dependencies (requires `crate:`)
/// - **`edition:<year>`**: Rust edition (2015, 2018, 2021, 2024). Defaults to current.
/// - **`cfg:<key>=<value>,<flag>`**: Configuration options
/// - **`env:<KEY>=<value>`**: Environment variables
/// - **`crate-attr:<attr>`**: Crate-level attributes
/// - **`new_source_root:local|library`**: Starts a new source root
/// - **`library`**: Marks crate as external library (not workspace member)
///
/// ## Global Meta (must appear at the top, in order)
///
/// - **`//- toolchain: nightly|stable`**: Sets the Rust toolchain (default: stable)
/// - **`//- target_data_layout: <layout>`**: LLVM data layout string
/// - **`//- proc_macros: <name1>,<name2>`**: Enables predefined test proc macros
/// - **`//- minicore: <flag1>, <flag2>`**: Includes subset of libcore
///
/// ## Cursor Markers
///
/// Use `$0` to mark cursor position(s) in the fixture:
/// - Single `$0`: marks a position (use with [`with_position`])
/// - Two `$0` markers: marks a range (use with [`with_range`])
/// - Escape as `\$0` if you need a literal `$0`
pub trait WithFixture: Default + ExpandDatabase + SourceDatabase + 'static {
    #[track_caller]
    fn with_single_file(
        #[rust_analyzer::rust_fixture] ra_fixture: &str,
    ) -> (Self, EditionedFileId) {
        let mut db = Self::default();
        let fixture = ChangeFixture::parse(ra_fixture);
        fixture.change.apply(&mut db);
        assert_eq!(fixture.files.len(), 1, "Multiple file found in the fixture");
        let file = EditionedFileId::from_span_guess_origin(&db, fixture.files[0]);
        (db, file)
    }

    #[track_caller]
    fn with_position(#[rust_analyzer::rust_fixture] ra_fixture: &str) -> (Self, FilePosition) {
        let (db, file_id, range_or_offset) = Self::with_range_or_offset(ra_fixture);
        let offset = range_or_offset.expect_offset();
        (db, FilePosition { file_id, offset })
    }
}
```
**Why This Matters for Contributors:** This mini-DSL allows writing complex multi-crate test setups as simple strings without creating actual files. The `//-` marker syntax is intuitive and the metadata system supports every aspect of a real Rust project (editions, dependencies, proc macros, cfg flags). This pattern enables testing cross-crate features, name resolution, and type inference without heavy test infrastructure. The `$0` cursor marker pattern is brilliant for IDE feature testing.

---

## Expert Commentary: Pattern 1

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Domain-Specific Language (DSL) for Test Fixtures + Embedded Language Design

**Rust-Specific Insight:**
This mini-DSL showcases Rust's strength in creating type-safe, zero-cost abstractions for testing. The `//-` marker syntax leverages Rust's comment system ingeniously—fixtures are valid Rust comments, so they can be embedded in doc tests. The `$0` cursor marker pattern exploits Rust's lack of string interpolation by making markers explicitly visible in source. The trait-based `WithFixture` API uses Rust's powerful trait system to add fixture parsing to any database type implementing the required traits.

The metadata system (`crate:`, `deps:`, `edition:`, etc.) mirrors Cargo.toml's structure, creating cognitive consistency. The cursor marker escape sequence (`\$0`) demonstrates attention to edge cases. The `#[track_caller]` usage on all constructors is exemplary—tests show exactly which line failed, not the fixture parser internals.

**Contribution Tip:**
When adding new metadata directives, update the trait documentation comprehensively. Add parse error tests for malformed directives. Consider adding a "strict mode" that rejects unknown metadata to catch typos. For cursor markers, add support for named markers (`$foo`, `$bar`) to support tests with multiple points of interest.

**Common Pitfalls:**
- **Forgetting to escape `$0`** in string literals within fixtures leads to confusing parser errors
- **Metadata order matters** for global meta (toolchain, target_data_layout, proc_macros, minicore must be at top)
- **Implicit file paths** can be surprising—single file defaults to `/main.rs`, not `/lib.rs`
- **Edition inheritance** behavior across crates in a single fixture can be non-obvious

**Related Patterns in Ecosystem:**
- **insta crate**: Snapshot testing with similar inline expectations but different DSL design
- **proptest**: Property-based testing where fixtures are generated rather than declared
- **rstest**: Parameterized tests with fixture attributes, but for runtime data not code structure
- **datatest-stable**: Directory-based test discovery, complementary to inline fixtures
- **trybuild**: Compile-fail tests using actual source files rather than strings

---

## Pattern 2: MiniCore - Minimal Standard Library Stub
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/test-utils/src/minicore.rs
**Category:** MiniCore System
**Code Example:**
```rust
//! This is a fixture we use for tests that need lang items.
//!
//! We want to include the minimal subset of core for each test, so this file
//! supports "conditional compilation". Tests use the following syntax to include minicore:
//!
//!  //- minicore: flag1, flag2
//!
//! We then strip all the code marked with other flags.
//!
//! Available flags:
//!     sized:
//!     copy: clone
//!     clone: sized
//!     option: panic
//!     result:
//!     iterator: option
//!     fn: sized, tuple
//!     future: pin
//!     deref: sized

impl MiniCore {
    pub const RAW_SOURCE: &'static str = include_str!("./minicore.rs");

    /// Strips parts of minicore.rs which are flagged by inactive flags.
    ///
    /// This is probably over-engineered to support flags dependencies.
    pub fn source_code(mut self, raw_source: &str) -> String {
        let mut buf = String::new();
        let mut lines = raw_source.split_inclusive('\n');

        let mut implications = Vec::new();

        // Parse `//!` preamble and extract flags and dependencies.
        // ... [parsing logic that builds dependency graph]

        // Fixed point loop to compute transitive closure of flags.
        loop {
            let mut changed = false;
            for &(u, v) in &implications {
                if self.has_flag(u) && !self.has_flag(v) {
                    self.activated_flags.push(v.to_owned());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // Strip regions based on active/inactive flags
        for line in lines {
            let trimmed = line.trim();
            if let Some(region) = trimmed.strip_prefix("// region:") {
                active_regions.push(region);
                continue;
            }

            let mut keep = true;
            for &region in &active_regions {
                keep &= self.has_flag(region);
            }

            if keep {
                buf.push_str(line);
            }
        }

        buf
    }
}

// Example usage in minicore.rs:
pub mod marker {
    // region:sized
    #[lang = "sized"]
    pub trait Sized: MetaSized {}
    // endregion:sized

    // region:copy
    #[lang = "copy"]
    pub trait Copy: Clone {}
    // endregion:copy
}
```
**Why This Matters for Contributors:** MiniCore solves a critical problem: tests need lang items like `Sized`, `Copy`, etc., but including the full standard library is slow and obscures test intent. The region-based conditional compilation with dependency tracking (e.g., `copy: clone` means activating `copy` automatically activates `clone`) creates a minimal, fast-compiling stdlib stub. The fixed-point algorithm for transitive dependencies is elegant and ensures consistency. Tests specify only what they need (e.g., `//- minicore: iterator, option`) and get exactly the minimal set of traits/types required.

---

## Expert Commentary: Pattern 2

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Conditional Compilation at Test Runtime + Dependency Graph Resolution

**Rust-Specific Insight:**
MiniCore is a masterclass in pragmatic engineering. The problem is fundamental: tests need `#[lang = "sized"]` and friends, but linking libstd adds 1000+ files and slows tests 10-100x. The solution—a manually curated "micro-stdlib" with region-based conditional compilation—is brilliant.

The dependency tracking (`copy: clone` means `copy` implies `clone`) uses a fixed-point algorithm to compute transitive closure. This is the same algorithm used in dataflow analysis and constraint solving. The `// region:flag` markers are processed at test runtime, not compile time, which is unusual but correct—it lets one `minicore.rs` file support all flag combinations without combinatorial explosion.

The pattern demonstrates Rust's ability to embed sophisticated algorithms in test infrastructure. The string processing (splitting on `// region:`, tracking active regions) is clean and efficient. Using `include_str!("./minicore.rs")` embeds the source at compile time, avoiding filesystem I/O during tests.

**Contribution Tip:**
When adding new traits/types to minicore.rs, define minimal dependencies (`iterator: option` not `iterator: option, fn, sized`). Document why each dependency exists. Add tests verifying flag transitivity (`activating X should activate Y`). Consider generating a dependency graph visualization for documentation.

**Common Pitfalls:**
- **Circular dependencies** in flag implications will cause the fixed-point loop to terminate, but implications will be incomplete
- **Forgetting `// region:` prefix** means code always includes, breaking minimal subset goal
- **Mismatched region/endregion** pairs cause silent inclusion/exclusion bugs
- **Assuming minicore matches std exactly**—it's intentionally minimal, some trait methods are stubs

**Related Patterns in Ecosystem:**
- **no_std compatibility testing**: Similar need for minimal stdlib in embedded contexts
- **feature flag combinations**: Cargo features have similar transitive dependencies but at compile time
- **conditional compilation with cfg**: `#[cfg(feature = "...")]` is the standard approach, but doesn't support runtime selection
- **cfg-if crate**: Compile-time conditional blocks, complementary but different use case
- **libcore/liballoc**: The real minimal Rust—minicore is an even smaller subset for tests

---

## Pattern 3: Cursor Marker Extraction ($0 Pattern)
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/test-utils/src/lib.rs
**Category:** Test Helpers
**Code Example:**
```rust
pub const CURSOR_MARKER: &str = "$0";
pub const ESCAPED_CURSOR_MARKER: &str = "\\$0";

/// Returns the offset of the first occurrence of `$0` marker and the copy of `text`
/// without the marker.
fn try_extract_offset(text: &str) -> Option<(TextSize, String)> {
    let cursor_pos = text.find(CURSOR_MARKER)?;
    let mut new_text = String::with_capacity(text.len() - CURSOR_MARKER.len());
    new_text.push_str(&text[..cursor_pos]);
    new_text.push_str(&text[cursor_pos + CURSOR_MARKER.len()..]);
    let cursor_pos = TextSize::from(cursor_pos as u32);
    Some((cursor_pos, new_text))
}

/// Returns `TextRange` between the first two markers `$0...$0` and the copy
/// of `text` without both of these markers.
fn try_extract_range(text: &str) -> Option<(TextRange, String)> {
    let (start, text) = try_extract_offset(text)?;
    let (end, text) = try_extract_offset(&text)?;
    Some((TextRange::new(start, end), text))
}

#[derive(Clone, Copy, Debug)]
pub enum RangeOrOffset {
    Range(TextRange),
    Offset(TextSize),
}

impl RangeOrOffset {
    pub fn expect_offset(self) -> TextSize {
        match self {
            RangeOrOffset::Offset(it) => it,
            RangeOrOffset::Range(_) => panic!("expected an offset but got a range instead"),
        }
    }
    pub fn expect_range(self) -> TextRange {
        match self {
            RangeOrOffset::Range(it) => it,
            RangeOrOffset::Offset(_) => panic!("expected a range but got an offset"),
        }
    }
}

/// Extracts `TextRange` or `TextSize` depending on the amount of `$0` markers
/// found in `text`.
pub fn extract_range_or_offset(text: &str) -> (RangeOrOffset, String) {
    if let Some((range, text)) = try_extract_range(text) {
        return (RangeOrOffset::Range(range), text);
    }
    let (offset, text) = extract_offset(text);
    (RangeOrOffset::Offset(offset), text)
}
```
**Why This Matters for Contributors:** The `$0` marker pattern is essential for IDE testing. Instead of computing byte offsets manually, tests embed `$0` directly in source code where the cursor should be. One marker = cursor position, two markers = selection range. The `RangeOrOffset` enum handles both cases uniformly. The escape sequence `\$0` allows literal `$0` in tests. This makes IDE feature tests (completions, goto definition, refactorings) incredibly readable and maintainable compared to coordinate-based approaches.

---

## Expert Commentary: Pattern 3

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Textual Marker Pattern + Sum Type for Multiple Modes

**Rust-Specific Insight:**
The `$0` marker pattern exemplifies Rust's "make invalid states unrepresentable" philosophy through the `RangeOrOffset` enum. Instead of using `Option<TextRange>` and `Option<TextSize>` (4 states, 2 valid), the enum has exactly 2 states, both valid. The `expect_offset()` and `expect_range()` methods panic with clear messages rather than silently misinterpreting data.

The extraction logic is carefully designed: `try_extract_range` calls `try_extract_offset` twice, naturally handling the two-marker case. The `TextSize::from(cursor_pos as u32)` conversion is an unusual cast—rust-analyzer uses `u32` for text sizes to save memory (files >4GB are unsupported, which is reasonable for source code).

The escape sequence `\$0` → `$0` handling would typically use `replace()` but here it's implicit in the marker search—the literal `\$` in source means `find("$0")` won't match. This is subtle but correct.

**Contribution Tip:**
Add `try_extract_offsets()` for tests needing 3+ markers. Consider named markers (`$cursor`, `$start`, `$end`) for complex scenarios. Add validation that markers are at valid UTF-8 boundaries (currently assumed). Provide helper to insert markers programmatically (useful for generated tests).

**Common Pitfalls:**
- **Forgetting to escape `$0`** in test strings that should contain literal `$0` (e.g., testing bash scripts)
- **Off-by-one errors** when manually computing positions—the abstraction prevents this but fixture creation can still err
- **UTF-8 boundary assumptions**—inserting `$0` mid-codepoint would cause panics elsewhere
- **Mixing range and offset modes**—calling `expect_range()` when fixture has one marker panics, but error message is clear

**Related Patterns in Ecosystem:**
- **Language Server Protocol**: Uses offset/range for edits, similar dual representation
- **pest parser**: Uses span tracking similar to TextRange for parse errors
- **codespan-reporting**: Diagnostic library with span/position handling
- **tree-sitter**: Parser with byte offset node positions
- **ropey**: Rope data structure for text editors with offset/position conversions

---

## Pattern 4: Annotation Extraction (//^^^ Pattern)
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/test-utils/src/lib.rs
**Category:** Test Helpers
**Code Example:**
```rust
/// Extracts `//^^^ some text` annotations.
///
/// A run of `^^^` can be arbitrary long and points to the corresponding range
/// in the line above.
///
/// The `// ^file text` syntax can be used to attach `text` to the entirety of
/// the file.
///
/// Multiline string values are supported:
///
/// // ^^^ first line
/// //   | second line
///
/// Trailing whitespace is sometimes desired but usually stripped by the editor
/// if at the end of a line, or incorrectly sized if followed by another
/// annotation. In those cases the annotation can be explicitly ended with the
/// `$` character.
///
/// // ^^^ trailing-ws-wanted  $
///
/// Annotations point to the last line that actually was long enough for the
/// range, not counting annotations themselves. So overlapping annotations are
/// possible:
/// ```text
/// // stuff        other stuff
/// // ^^ 'st'
/// // ^^^^^ 'stuff'
/// //              ^^^^^^^^^^^ 'other stuff'
/// ```
pub fn extract_annotations(text: &str) -> Vec<(TextRange, String)> {
    let mut res = Vec::new();
    // map from line length to beginning of last line that had that length
    let mut line_start_map = BTreeMap::new();
    let mut line_start: TextSize = 0.into();
    let mut prev_line_annotations: Vec<(TextSize, usize)> = Vec::new();

    for line in text.split_inclusive('\n') {
        let mut this_line_annotations = Vec::new();
        let line_length = if let Some((prefix, suffix)) = line.split_once("//") {
            let ss_len = TextSize::of("//");
            let annotation_offset = TextSize::of(prefix) + ss_len;
            for annotation in extract_line_annotations(suffix.trim_end_matches('\n')) {
                match annotation {
                    LineAnnotation::Annotation { mut range, content, file } => {
                        range += annotation_offset;
                        this_line_annotations.push((range.end(), res.len()));
                        let range = if file {
                            TextRange::up_to(TextSize::of(text))
                        } else {
                            let line_start = line_start_map.range(range.end()..).next().unwrap();
                            range + line_start.1
                        };
                        res.push((range, content));
                    }
                    LineAnnotation::Continuation { mut offset, content } => {
                        offset += annotation_offset;
                        let &(_, idx) = prev_line_annotations
                            .iter()
                            .find(|&&(off, _idx)| off == offset)
                            .unwrap();
                        res[idx].1.push('\n');
                        res[idx].1.push_str(&content);
                        res[idx].1.push('\n');
                    }
                }
            }
            annotation_offset
        } else {
            TextSize::of(line)
        };

        line_start_map = line_start_map.split_off(&line_length);
        line_start_map.insert(line_length, line_start);
        line_start += TextSize::of(line);
        prev_line_annotations = this_line_annotations;
    }

    res
}

#[test]
fn test_extract_annotations_1() {
    let text = stdx::trim_indent(
        r#"
fn main() {
    let (x,     y) = (9, 2);
       //^ def  ^ def
    zoo + 1
} //^^^ type:
  //  | i32

// ^file
    "#,
    );
    let res = extract_annotations(&text)
        .into_iter()
        .map(|(range, ann)| (&text[range], ann))
        .collect::<Vec<_>>();

    assert_eq!(
        res[..3],
        [("x", "def".into()), ("y", "def".into()), ("zoo", "type:\ni32\n".into())]
    );
    assert_eq!(res[3].0.len(), 115); // ^file annotation
}
```
**Why This Matters for Contributors:** The `//^^^` annotation pattern allows tests to embed expected results directly under the code being tested. The caret characters point upward to specific ranges in the previous line. This is perfect for type inference tests (`// ^^^ type: i32`), error messages, diagnostics, etc. The multiline support with `|` continuation and the `^file` whole-file annotation make it versatile. The `BTreeMap` tracking of line lengths enables overlapping annotations, crucial for complex scenarios.

---

## Expert Commentary: Pattern 4

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Visual Alignment Pattern + Stateful Line Parsing + Complex DSL

**Rust-Specific Insight:**
The `//^^^` annotation pattern is remarkably sophisticated. The key insight: use `BTreeMap<line_length, line_start>` to map from caret position back to which line the annotation targets. This handles overlapping annotations cleanly—multiple `//^^^` lines can point to different ranges in the same source line by varying the caret count.

The multiline support (`| continuation`) allows complex multi-line expected values while maintaining visual alignment. The trailing `$` delimiter for preserving whitespace shows attention to edge cases (editors strip trailing spaces, breaking tests).

The algorithm tracks "last line that had length >= annotation start position" using `BTreeMap::range()`. This is a clever use of Rust's ordered map to binary-search for the target line. The `split_off()` call prunes the map of lines too short to be targets, keeping search space small.

The `^file` annotation for whole-file assertions is a nice special case that shows API consistency—same annotation syntax, different semantic meaning based on keyword.

**Contribution Tip:**
Add column-based markers (`//^^^ [1:5] annotation`) for disambiguating overlapping ranges. Consider graphical output showing which code range each annotation targets (helpful when tests fail). Add validation that carets align with actual code positions. Support inline annotations (`x + y // <- type: i32`) as syntactic sugar.

**Common Pitfalls:**
- **Annotation-only lines** (no code above) silently succeed with wrong range
- **Misaligned carets**—visual alignment doesn't match column position due to tabs/Unicode
- **Forgetting continuation pipe**—multiline annotations missing `|` get concatenated without newlines
- **Overlapping annotations with identical ranges**—later overwrites earlier without warning

**Related Patterns in Ecosystem:**
- **rustdoc's `# fn main() { ... }`**: Hidden lines in doc tests, similar annotation concept
- **compiletest-rs**: Rustc's test harness with `//~ ERROR` annotations
- **mdbook's `# hidden`**: Hiding lines in code examples
- **proptest's `#[derive(Arbitrary)]`**: Generated test cases with shrinking, different approach to test data
- **ui_test crate**: Compiler testing framework with similar annotation expectations

---

## Pattern 5: TestDB with Salsa Integration
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/hir-def/src/test_db.rs
**Category:** Test Database
**Code Example:**
```rust
#[salsa_macros::db]
pub(crate) struct TestDB {
    storage: salsa::Storage<Self>,
    files: Arc<base_db::Files>,
    crates_map: Arc<CratesMap>,
    events: Arc<Mutex<Option<Vec<salsa::Event>>>>,
    nonce: Nonce,
}

impl Default for TestDB {
    fn default() -> Self {
        let events = <Arc<Mutex<Option<Vec<salsa::Event>>>>>::default();
        let mut this = Self {
            storage: salsa::Storage::new(Some(Box::new({
                let events = events.clone();
                move |event| {
                    let mut events = events.lock().unwrap();
                    if let Some(events) = &mut *events {
                        events.push(event);
                    }
                }
            }))),
            events,
            files: Default::default(),
            crates_map: Default::default(),
            nonce: Nonce::new(),
        };
        this.set_expand_proc_attr_macros_with_durability(true, Durability::HIGH);
        this.set_all_crates(Arc::new(Box::new([])));
        _ = base_db::LibraryRoots::builder(Default::default())
            .durability(Durability::MEDIUM)
            .new(&this);
        _ = base_db::LocalRoots::builder(Default::default())
            .durability(Durability::MEDIUM)
            .new(&this);
        CrateGraphBuilder::default().set_in_db(&mut this);
        this
    }
}

impl TestDB {
    pub(crate) fn log(&self, f: impl FnOnce()) -> Vec<salsa::Event> {
        *self.events.lock().unwrap() = Some(Vec::new());
        f();
        self.events.lock().unwrap().take().unwrap()
    }

    pub(crate) fn log_executed(&self, f: impl FnOnce()) -> Vec<String> {
        let events = self.log(f);
        events
            .into_iter()
            .filter_map(|e| match e.kind {
                salsa::EventKind::WillExecute { database_key } => {
                    let ingredient = (self as &dyn salsa::Database)
                        .ingredient_debug_name(database_key.ingredient_index());
                    Some(ingredient.to_string())
                }
                _ => None,
            })
            .collect()
    }
}

#[salsa_macros::db]
impl SourceDatabase for TestDB {
    fn file_text(&self, file_id: base_db::FileId) -> FileText {
        self.files.file_text(file_id)
    }
    // ... other database methods
}
```
**Why This Matters for Contributors:** TestDB demonstrates how to create a minimal database implementation for testing a Salsa-based query system. The event logging mechanism (`log()` and `log_executed()`) is crucial for testing incremental computation—you can verify which queries re-execute after changes. The `nonce` field ensures unique database instances. The initialization pattern (setting durabilities, building crate graphs) shows the proper sequence for setting up a test database. The `Arc<Mutex<Option<Vec<salsa::Event>>>>` pattern allows conditional event collection without overhead when not logging.

---

## Expert Commentary: Pattern 5

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Salsa Integration Pattern + Test Event Logging + Nonce-based Isolation

**Rust-Specific Insight:**
This TestDB implementation showcases advanced patterns for testing incremental computation systems. The `Arc<Mutex<Option<Vec<salsa::Event>>>>` triple-wrapping is intentional: `Arc` for shared ownership, `Mutex` for interior mutability, `Option` to distinguish "not logging" from "logging empty." This zero-cost abstraction means event collection has no overhead when disabled.

The closure-based event callback (`Box::new(move |event| { ... })`) captures the shared `events` Arc, creating a feedback loop where Salsa reports events back to the database. This is safe because the mutex prevents deadlocks (events are pushed, not queried during execution).

The `Nonce` field ensures each TestDB instance is unique, preventing Salsa from reusing cached results across unrelated tests. The `Durability::HIGH`/`MEDIUM` settings pre-configure stability—`HIGH` means "this will never change during this test."

The `log_executed()` helper filtering for `WillExecute` events is crucial—it ignores `DidValidateMemoizedValue` and other noise, focusing on actual recomputation.

**Contribution Tip:**
Add `assert_not_executed()` helper to verify memoization works. Create `diff_executions()` to compare two event logs. Consider adding event count assertions (`assert_max_executions(10)`). Add helpers for common assertion patterns like "only these queries executed."

**Common Pitfalls:**
- **Forgetting to call `log()`** before the operation—events are only collected when `Option` is `Some`
- **Nested `log()` calls** will overwrite the inner log's events
- **Comparing query names as strings**—fragile to refactoring, consider enum-based filtering
- **Not setting durabilities**—tests may fail intermittently as Salsa recomputes unexpectedly

**Related Patterns in Ecosystem:**
- **criterion's event sampling**: Similar logging pattern for benchmark statistics
- **tracing subscriber**: General-purpose event collection for debugging
- **tokio-console**: Runtime introspection using similar event callback pattern
- **cargo's fingerprinting**: Incremental compilation tracking, similar goals
- **salsa examples**: Official salsa repo has simpler TestDB examples for reference

---

## Pattern 6: expect_test for Snapshot Testing
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/hir-def/src/macro_expansion_tests/mbe.rs
**Category:** Snapshot Testing
**Code Example:**
```rust
use expect_test::expect;

#[test]
fn token_mapping_smoke_test() {
    check(
        r#"
macro_rules! f {
    ( struct $ident:ident ) => {
        struct $ident {
            map: ::std::collections::HashSet<()>,
        }
    };
}

// +spans+syntaxctxt
f!(struct MyTraitMap2);
"#,
        expect![[r#"
macro_rules! f {
    ( struct $ident:ident ) => {
        struct $ident {
            map: ::std::collections::HashSet<()>,
        }
    };
}

struct#0:MacroRules[BE8F, 0]@58..64#17408# MyTraitMap2#0:MacroCall[BE8F, 0]@31..42#ROOT2024# {#0:MacroRules[BE8F, 0]@72..73#17408#
    map#0:MacroRules[BE8F, 0]@86..89#17408#:#0:MacroRules[BE8F, 0]@89..90#17408# #0:MacroRules[BE8F, 0]@89..90#17408#::#0:MacroRules[BE8F, 0]@91..93#17408#std#0:MacroRules[BE8F, 0]@93..96#17408#::#0:MacroRules[BE8F, 0]@96..98#17408#collections#0:MacroRules[BE8F, 0]@98..109#17408#::#0:MacroRules[BE8F, 0]@109..111#17408#HashSet#0:MacroRules[BE8F, 0]@111..118#17408#<#0:MacroRules[BE8F, 0]@118..119#17408#(#0:MacroRules[BE8F, 0]@119..120#17408#)#0:MacroRules[BE8F, 0]@120..121#17408#>#0:MacroRules[BE8F, 0]@121..122#17408#,#0:MacroRules[BE8F, 0]@122..123#17408#
}#0:MacroRules[BE8F, 0]@132..133#17408#
"#]],
    );
}
```
**Why This Matters for Contributors:** The `expect_test` crate provides snapshot testing with in-source expectations. Instead of maintaining separate `.snap` files, expected output is embedded in test code using the `expect![[...]]` macro. Running tests with `UPDATE_EXPECT=1` automatically updates expectations when intentional changes occur. This pattern is perfect for compiler/macro expansion tests where output is complex and verbose. The double-bracket syntax `expect![[...]]` supports multi-line raw strings. The workflow is smoother than traditional snapshot testing—no jumping between files, and git diffs show both code and expectation changes together.

---

## Expert Commentary: Pattern 6

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** In-Source Snapshot Testing + Blessed Workflow + Macro-Based DSL

**Rust-Specific Insight:**
The `expect_test` crate revolutionizes snapshot testing in Rust. Traditional snapshot testing (à la Jest) requires separate `.snap` files, breaking locality and complicating reviews. The `expect![[...]]` macro embeds expectations inline using a double-bracket syntax that's visually distinct and supports raw multi-line strings.

The workflow is superior: write `expect![[""]]`, run tests, they fail, run `UPDATE_EXPECT=1 cargo test`, expectations auto-update. This is possible because proc macros can rewrite source files—the macro tracks source locations via `Span` and updates the file on disk.

The pattern works beautifully for complex output like macro expansions with span information (`#0:MacroRules[BE8F, 0]@58..64#17408#`). Git diffs show both code and expectation changes together, improving review quality. The `expect` macro is actually minimal—just wraps the string and provides `assert_eq!` behavior.

**Contribution Tip:**
Use `expect_test` for any test with complex, structured output (ASTs, type inference results, macro expansions). Group related expectations with `expect_file!` for very large outputs. Add `--features=update-expect` guard to prevent accidental updates in CI. Consider `expect_test::expect_snapshot!` for prettier formatting.

**Common Pitfalls:**
- **Forgetting `UPDATE_EXPECT=1`** and manually copying output into expectations
- **Committing auto-updated expectations** without reviewing—always diff before commit
- **Non-deterministic output** breaks the pattern—tests flake as expectations thrash
- **Too-large expectations** make tests unreadable—consider factoring or using `expect_file!`

**Related Patterns in Ecosystem:**
- **insta crate**: Alternative snapshot testing with separate `.snap` files but better diffing
- **k9**: Snapshot testing with colored diffs and inline expectations
- **assert_cmd**: Snapshot testing for CLI output
- **goldentests**: Directory-based golden file testing
- **similar**: Diffing library used by many snapshot test crates

---

## Pattern 7: Type Annotation Testing (//^^^ type: Pattern)
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/hir-ty/src/tests.rs
**Category:** Type Inference Testing
**Code Example:**
```rust
#[track_caller]
fn check_types(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
    check_impl(ra_fixture, false, true, false)
}

#[track_caller]
fn check_impl(
    #[rust_analyzer::rust_fixture] ra_fixture: &str,
    allow_none: bool,
    only_types: bool,
    display_source: bool,
) {
    let _tracing = setup_tracing();
    let (db, files) = TestDB::with_many_files(ra_fixture);

    crate::attach_db(&db, || {
        let mut had_annotations = false;
        let mut types = FxHashMap::default();

        // Extract annotations from fixture
        for (file_id, annotations) in db.extract_annotations() {
            for (range, expected) in annotations {
                let file_range = FileRange { file_id, range };
                if only_types {
                    types.insert(file_range, expected);
                } else if expected.starts_with("type: ") {
                    types.insert(file_range, expected.trim_start_matches("type: ").to_owned());
                }
                had_annotations = true;
            }
        }
        assert!(had_annotations || allow_none, "no `//^` annotations found");

        // Check inferred types match annotations
        for (def, krate) in defs {
            let display_target = DisplayTarget::from_crate(&db, krate);
            let (body, body_source_map) = db.body_with_source_map(def);
            let inference_result = InferenceResult::for_body(&db, def);

            for (expr, ty) in inference_result.type_of_expr.iter() {
                let ty = ty.as_ref();
                let node = match expr_node(&body_source_map, expr, &db) {
                    Some(value) => value,
                    None => continue,
                };
                let range = node.as_ref().original_file_range_rooted(&db);
                if let Some(expected) = types.remove(&range) {
                    let actual = ty.display_test(&db, display_target).to_string();
                    assert_eq!(actual, expected, "type annotation differs at {:#?}", range.range);
                }
            }
        }

        // Ensure all annotations were checked
        if !types.is_empty() {
            let mut buf = String::new();
            format_to!(buf, "Unchecked type annotations:\n");
            for t in types {
                format_to!(buf, "{:?}: type {}\n", t.0.range, t.1);
            }
            panic!("{}", buf);
        }
    });
}

// Example test usage:
#[test]
fn example_type_test() {
    check_types(
        r#"
fn foo() {
    let x = 42;
      //^ i32
    let y = "hello";
      //^ &str
}
        "#,
    );
}
```
**Why This Matters for Contributors:** This pattern enables comprehensive type inference testing. The `check_types()` helper extracts `//^ type: T` annotations, runs type inference, and verifies that the inferred type of each annotated expression matches the annotation. The bidirectional check (annotation → inferred type AND ensuring no unchecked annotations remain) prevents both false negatives and test drift. The `display_test()` method produces normalized type representations suitable for comparison. This pattern is used extensively across hundreds of type inference tests in rust-analyzer.

---

## Expert Commentary: Pattern 7

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Bidirectional Test Validation + Type Display Normalization + Comprehensive Coverage

**Rust-Specific Insight:**
The `check_types()` pattern demonstrates industrial-strength type system testing. The bidirectional check is critical: (1) for each annotation, verify inferred type matches; (2) at end, assert all annotations were checked. This prevents test drift where code changes but annotations don't update.

The `display_test()` method is key—it normalizes type representations for comparison. Without this, trivial formatting differences (spaces, parentheses) would cause spurious failures. The `DisplayTarget` captures crate context, ensuring types like `Option<T>` display consistently whether from std or a local definition.

The `FxHashMap` for tracking unchecked annotations is intentional—insertion order doesn't matter, but lookups are O(1). The `remove()` on successful check is elegant: remaining keys at the end are unchecked annotations.

The `attach_db()` pattern wraps the test in a thread-local context for better error messages and allows the Salsa database to be queried during panics for debugging.

**Contribution Tip:**
Add `check_types_no_diagnostics()` variant that also asserts no errors. Create `check_type_at_offset()` for targeted single-expression checks. Support regex patterns in annotations (`//^ i32|u32`) for platform-dependent types. Add `check_infer_full()` that generates annotations for all expressions, useful for bulk test creation.

**Common Pitfalls:**
- **Annotations on non-expressions**—pointing at whitespace or statements silently skips checks
- **Macro-generated code**—annotations in macro output may not align correctly with expansions
- **Generic type display**—display may vary based on inference state, use normalized forms
- **Multiple expressions at same range**—only the last is checked, others are silently skipped

**Related Patterns in Ecosystem:**
- **ui_test annotations**: Similar `//~ ERROR` syntax for compiler diagnostics
- **compiletest-rs**: Rustc's test framework with similar annotation expectations
- **trybuild**: Compile-fail testing with stderr snapshot comparison
- **insta**: General snapshot testing that could be adapted for type checking
- **chalk's test framework**: Trait solver testing with similar annotation patterns

---

## Pattern 8: Assert Linear - Performance Regression Detection
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/test-utils/src/assert_linear.rs
**Category:** Performance Testing
**Code Example:**
```rust
/// Checks that a set of measurements looks like a linear function rather than
/// like a quadratic function. Algorithm:
///
/// 1. Linearly scale input to be in [0; 1)
/// 2. Using linear regression, compute the best linear function approximating
///    the input.
/// 3. Compute RMSE and maximal absolute error.
/// 4. Check that errors are within tolerances and that the constant term is not
///    too negative.
///
/// We might get false positives on a VM, but never false negatives. So, if the
/// first round fails, we repeat the ordeal three more times and fail only if
/// every time there's a fault.
#[derive(Default)]
pub struct AssertLinear {
    rounds: Vec<Round>,
}

impl AssertLinear {
    pub fn next_round(&mut self) -> bool {
        if let Some(round) = self.rounds.last_mut() {
            round.finish();
        }
        if self.rounds.iter().any(|it| it.linear) || self.rounds.len() == 4 {
            return false;
        }
        self.rounds.push(Round::default());
        true
    }

    pub fn sample(&mut self, x: f64, y: f64) {
        self.rounds.last_mut().unwrap().samples.push((x, y));
    }
}

impl Drop for AssertLinear {
    fn drop(&mut self) {
        assert!(!self.rounds.is_empty());
        if self.rounds.iter().all(|it| !it.linear) {
            for round in &self.rounds {
                eprintln!("\n{}", round.plot);
            }
            panic!("Doesn't look linear!");
        }
    }
}

impl Round {
    fn finish(&mut self) {
        let (mut xs, mut ys): (Vec<_>, Vec<_>) = self.samples.iter().copied().unzip();
        normalize(&mut xs);
        normalize(&mut ys);

        // Linear regression: finding a and b to fit y = a + b*x.
        let mean_x = mean(&xs);
        let mean_y = mean(&ys);

        let b = {
            let mut num = 0.0;
            let mut denom = 0.0;
            for (x, y) in xy.clone() {
                num += (x - mean_x) * (y - mean_y);
                denom += (x - mean_x).powi(2);
            }
            num / denom
        };

        let a = mean_y - b * mean_x;

        let mut se = 0.0;
        let mut max_error = 0.0f64;
        for (x, y) in xy {
            let y_pred = a + b * x;
            se += (y - y_pred).powi(2);
            max_error = max_error.max((y_pred - y).abs());
        }

        let rmse = (se / xs.len() as f64).sqrt();
        self.linear = rmse < 0.05 && max_error < 0.1 && a > -0.1;
    }
}

// Example usage:
#[test]
fn parse_is_linear() {
    if skip_slow_tests() { return; }

    let mut al = AssertLinear::default();
    while al.next_round() {
        for size in [10, 100, 1000, 10000] {
            let input = generate_input(size);
            let start = Instant::now();
            parse(&input);
            let elapsed = start.elapsed().as_millis() as f64;
            al.sample(size as f64, elapsed);
        }
    }
}
```
**Why This Matters for Contributors:** AssertLinear detects algorithmic complexity regressions by verifying that performance scales linearly rather than quadratically. It uses proper statistical methods (linear regression, RMSE, max error) rather than naive timing checks. The multi-round retry logic (up to 4 rounds) handles VM jitter and other environmental noise—tests must fail consistently to actually fail. The `Drop` implementation provides diagnostic output (regression plot) on failure. This is far superior to simple timing assertions and catches O(n²) bugs before they reach production. The normalization step makes the thresholds (RMSE < 0.05) work across different scales.

---

## Expert Commentary: Pattern 8

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Statistical Performance Testing + Multi-Round Flake Resistance + Linear Regression

**Rust-Specific Insight:**
AssertLinear is a masterpiece of performance testing. It solves the fundamental problem: naive timing assertions (`elapsed < 100ms`) are brittle and don't detect algorithmic complexity. This pattern uses statistical rigor—linear regression, RMSE, max error—to distinguish O(n) from O(n²) behavior.

The multi-round retry (up to 4 attempts) handles VM jitter and environmental noise brilliantly. The test must fail *consistently* to actually fail, eliminating false positives from GC pauses or CPU throttling. The `Drop` implementation prints diagnostic plots on failure, providing actionable data for debugging.

The normalization step is critical—it scales inputs to [0,1) so the thresholds (RMSE < 0.05, max error < 0.1) are meaningful regardless of input scale. The `a > -0.1` check ensures the linear fit doesn't have a large negative constant term (which would indicate O(n²) with small constant factor appearing linear at small n).

The pattern integrates with `skip_slow_tests()` to run only in comprehensive CI, not during rapid development.

**Contribution Tip:**
Add `AssertQuadratic` for tests that should be O(n²) (some dynamic programming). Create `AssertLogLinear` for O(n log n) algorithms. Add environment variable to force-fail for local testing (`FORCE_PERF_TESTS=1`). Export regression data in machine-readable format for trends over time.

**Common Pitfalls:**
- **Too few sample sizes**—need 4+ data points for meaningful regression
- **Non-representative sizes**—samples like [10, 11, 12, 13] won't reveal complexity differences
- **Warmup effects**—first iteration may be slower, skewing results
- **Multicore interference**—parallel test runners can introduce noise, run sequentially

**Related Patterns in Ecosystem:**
- **criterion**: Full-featured benchmarking with statistical analysis and regression detection
- **iai**: Cachegrind-based deterministic benchmarking, different approach to noise elimination
- **proptest's ![]`: Benchmarking variants of property tests
- **divan**: Modern benchmarking with statistical rigor, similar goals
- **codspeed**: Continuous benchmarking platform for detecting regressions in CI

---

## Pattern 9: Benchmark Fixture Generators
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/test-utils/src/bench_fixture.rs
**Category:** Benchmark Helpers
**Code Example:**
```rust
//! Generates large snippets of Rust code for usage in the benchmarks.

use std::fs;
use stdx::format_to;
use crate::project_root;

pub fn big_struct() -> String {
    let n = 1_000;
    big_struct_n(n)
}

pub fn big_struct_n(n: u32) -> String {
    let mut buf = "pub struct RegisterBlock {".to_owned();
    for i in 0..n {
        format_to!(buf, "  /// Doc comment for {}.\n", i);
        format_to!(buf, "  pub s{}: S{},\n", i, i);
    }
    buf.push_str("}\n\n");
    for i in 0..n {
        format_to!(
            buf,
            "

#[repr(transparent)]
struct S{} {{
    field: u32,
}}",
            i
        );
    }

    buf
}

pub fn glorious_old_parser() -> String {
    let path = project_root().join("bench_data/glorious_old_parser");
    fs::read_to_string(path).unwrap()
}

pub fn numerous_macro_rules() -> String {
    let path = project_root().join("bench_data/numerous_macro_rules");
    fs::read_to_string(path).unwrap()
}

// Example benchmark usage:
#[test]
fn benchmark_foo() {
    if skip_slow_tests() { return; }

    let data = bench_fixture::big_struct();
    let analysis = setup_analysis(&data);

    let hash = {
        let _b = bench("struct_parsing");
        compute_some_result(analysis)
    };
    assert_eq!(hash, 92); // Sanity check that real work was done
}
```
**Why This Matters for Contributors:** Benchmark fixtures provide deterministic, scalable input for performance tests. `big_struct_n()` generates parameterized test data, crucial for `AssertLinear` tests. The hybrid approach (generate some fixtures, load others from `bench_data/`) balances flexibility and realism. Loading real-world code samples like `glorious_old_parser` (actual parser code from an old Rust project) ensures benchmarks reflect realistic workloads, not just synthetic patterns. The sanity check pattern (returning/asserting a hash) prevents "optimizing" benchmarks into no-ops.

---

## Expert Commentary: Pattern 9

**Idiomatic Rating: ⭐⭐⭐⭐ (4/5)**

**Pattern Classification:** Parameterized Fixture Generation + Hybrid Synthetic/Real Data

**Rust-Specific Insight:**
Benchmark fixture generators balance two needs: parameterized synthetic data for scalability testing and real-world code samples for realistic behavior. The `big_struct_n(n)` function generates structs with `n` fields, enabling precise control over input size for `AssertLinear` tests. The `format_to!` macro (from `stdx`) provides efficient string building without allocations-per-append.

The pattern of returning a hash/checksum from benchmarks (`assert_eq!(hash, 92)`) is crucial—it prevents the compiler from optimizing away the actual work. Without this, DCE (dead code elimination) could turn expensive operations into no-ops, making benchmarks meaningless.

The hybrid approach (generate some fixtures, load others from `bench_data/`) is pragmatic. Generated fixtures are deterministic and parameterized; real fixtures (`glorious_old_parser`) capture emergent complexity patterns not easily synthesized. The `project_root()` helper abstracts workspace structure, making tests robust to directory layout.

**Contribution Tip:**
Add more real-world samples across different coding styles (async-heavy, macro-heavy, deeply nested). Parameterize fixture generators beyond size (e.g., `big_struct_with_generics(n, generic_count)`). Cache generated fixtures to disk for very large sizes to avoid regeneration overhead. Add metadata to fixtures (LOC, complexity metrics) for analysis.

**Common Pitfalls:**
- **Generated code too uniform**—doesn't exercise realistic code paths with varied patterns
- **Missing hash assertions**—benchmarks become no-ops via dead code elimination
- **Huge fixtures in-memory**—large generated strings can OOM, stream or use tempfiles instead
- **Stale real fixtures**—`glorious_old_parser` may not reflect modern Rust syntax

**Related Patterns in Ecosystem:**
- **cargo-fuzz corpus**: Saved inputs that triggered interesting behavior, similar archival pattern
- **proptest seed files**: Generated test cases saved for regression testing
- **criterion's baselines**: Stored benchmark results for historical comparison
- **libfuzzer-sys test cases**: Minimized inputs that found bugs, real-world test data
- **compiletest fixtures**: Rustc test suite's large corpus of test Rust programs

---

## Pattern 10: check_assist Test Helper Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-assists/src/tests.rs
**Category:** IDE Assist Testing
**Code Example:**
```rust
pub(crate) const TEST_CONFIG: AssistConfig = AssistConfig {
    snippet_cap: SnippetCap::new(true),
    allowed: None,
    insert_use: InsertUseConfig {
        granularity: ImportGranularity::Crate,
        prefix_kind: hir::PrefixKind::Plain,
        enforce_granularity: true,
        group: true,
        skip_glob_imports: true,
    },
    prefer_no_std: false,
    prefer_prelude: true,
    assist_emit_must_use: false,
    // ... more config
};

#[track_caller]
pub(crate) fn check_assist(
    assist: Handler,
    #[rust_analyzer::rust_fixture] ra_fixture_before: &str,
    #[rust_analyzer::rust_fixture] ra_fixture_after: &str,
) {
    let ra_fixture_after = trim_indent(ra_fixture_after);
    check(assist, ra_fixture_before, ExpectedResult::After(&ra_fixture_after), None);
}

#[track_caller]
pub(crate) fn check_assist_with_config(
    assist: Handler,
    config: AssistConfig,
    #[rust_analyzer::rust_fixture] ra_fixture_before: &str,
    #[rust_analyzer::rust_fixture] ra_fixture_after: &str,
) {
    let ra_fixture_after = trim_indent(ra_fixture_after);
    check_with_config(
        config,
        assist,
        ra_fixture_before,
        ExpectedResult::After(&ra_fixture_after),
        None,
    );
}

// Example usage:
#[test]
fn test_extract_variable() {
    check_assist(
        extract_variable,
        r#"
fn main() {
    let x = 1 $0+ 2;
}
        "#,
        r#"
fn main() {
    let $0var_name = 1 + 2;
    let x = var_name;
}
        "#,
    );
}
```
**Why This Matters for Contributors:** The `check_assist` family of helpers provides a clean DSL for testing IDE code actions. The before/after fixture pattern with `$0` cursor markers makes tests readable—you see exactly what code transforms into what. The parameterized config variants (`check_assist_with_config`, `check_assist_no_snippet_cap`, `check_assist_import_one`) test different IDE settings without duplicating test logic. The `#[track_caller]` attribute provides better error messages by pointing to the actual test, not the helper. The `trim_indent()` call allows nicely formatted multi-line strings in tests.

---

## Expert Commentary: Pattern 10

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** DSL for IDE Transformations + Before/After Snapshot + Parameterized Configuration

**Rust-Specific Insight:**
The `check_assist` family creates a declarative DSL for testing IDE code actions. The before/after pattern with `$0` cursor markers is instantly readable—reviewers see exactly what transformation happens. The `#[track_caller]` on all helpers is exemplary: test failures point to the actual test line, not into `check_assist` internals.

The parameterized variants (`check_assist_with_config`, `check_assist_no_snippet_cap`, `check_assist_import_one`) demonstrate the "don't repeat yourself" principle—one implementation, multiple test scenarios. The `TEST_CONFIG` constant provides sensible defaults, and variants override specific fields. The `trim_indent()` call allows writing multi-line fixtures with proper indentation in source, improving readability.

The `ExpectedResult::After` enum (not shown but implied) likely also supports `ExpectedResult::NotApplicable` for testing when assists shouldn't trigger, making the API complete. The pattern scales to hundreds of tests without duplication.

**Contribution Tip:**
Add `check_assist_multiple()` for assists that can be applied multiple times. Create `check_assist_selections()` for range-based refactorings. Add `check_assist_equivalent()` for comparing AST structure rather than text (whitespace-independent). Consider adding negative tests (`check_assist_not_applicable()`) as first-class citizens.

**Common Pitfalls:**
- **Inconsistent whitespace** in expected output—use `trim_indent()` consistently
- **Cursor position in output**—some refactorings should move cursor, test both
- **Config mismatch**—forgetting which config variant is used can lead to confusing failures
- **Non-idempotent assists**—applying twice should be no-op or well-defined, test this

**Related Patterns in Ecosystem:**
- **codemod testing**: Similar before/after transformation testing in refactoring tools
- **LSP test utilities**: Language server testing with similar fixture patterns
- **refactorium testing**: IntelliJ refactoring test framework, similar approach
- **rope manipulation tests**: Text editing transformation tests with similar patterns
- **tree-sitter query tests**: Pattern matching transformations with similar validation

---

## Pattern 11: Module Navigation Test Helpers
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/hir-def/src/test_db.rs
**Category:** Test Database Helpers
**Code Example:**
```rust
impl TestDB {
    pub(crate) fn module_for_file(&self, file_id: FileId) -> ModuleId {
        for &krate in self.relevant_crates(file_id).iter() {
            let crate_def_map = crate_def_map(self, krate);
            for (local_id, data) in crate_def_map.modules() {
                if data.origin.file_id().map(|file_id| file_id.file_id(self)) == Some(file_id) {
                    return local_id;
                }
            }
        }
        panic!("Can't find module for file")
    }

    pub(crate) fn module_at_position(&self, position: FilePosition) -> ModuleId {
        let file_module = self.module_for_file(position.file_id.file_id(self));
        let mut def_map = file_module.def_map(self);
        let module = self.mod_at_position(def_map, position);

        def_map = match self.block_at_position(def_map, position) {
            Some(it) => it,
            None => return module,
        };
        loop {
            let new_map = self.block_at_position(def_map, position);
            match new_map {
                Some(new_block) if !std::ptr::eq(&new_block, &def_map) => {
                    def_map = new_block;
                }
                _ => {
                    // FIXME: handle `mod` inside block expression
                    return def_map.root;
                }
            }
        }
    }

    /// Finds the smallest/innermost module in `def_map` containing `position`.
    fn mod_at_position(&self, def_map: &DefMap, position: FilePosition) -> ModuleId {
        let mut size = None;
        let mut res = def_map.root;
        for (module, data) in def_map.modules() {
            let src = data.definition_source(self);
            let Some(file_id) = src.file_id.file_id() else {
                continue;
            };
            if file_id.file_id(self) != position.file_id.file_id(self) {
                continue;
            }

            let range = match src.value {
                ModuleSource::SourceFile(it) => it.syntax().text_range(),
                ModuleSource::Module(it) => it.syntax().text_range(),
                ModuleSource::BlockExpr(it) => it.syntax().text_range(),
            };

            if !range.contains(position.offset) {
                continue;
            }

            let new_size = match size {
                None => range.len(),
                Some(size) => {
                    if range.len() < size {
                        range.len()
                    } else {
                        size
                    }
                }
            };

            if size != Some(new_size) {
                size = Some(new_size);
                res = module;
            }
        }

        res
    }
}
```
**Why This Matters for Contributors:** These test helpers solve the problem of locating semantic information from textual positions in tests. `module_for_file()` maps a FileId to the ModuleId that owns it by searching through crate def maps—essential for setting up test contexts. `module_at_position()` handles the tricky case of inline modules and block expressions by iteratively narrowing down through nested DefMaps. The `mod_at_position()` helper finds the smallest containing module by tracking text ranges and comparing sizes. These utilities abstract away the complexity of rust-analyzer's multi-layer module system, letting tests focus on the actual behavior being tested.

---

## Expert Commentary: Pattern 11

**Idiomatic Rating: ⭐⭐⭐⭐ (4/5)**

**Pattern Classification:** Semantic Navigation Helpers + Multi-Layer DefMap Traversal + Smallest-Range Selection

**Rust-Specific Insight:**
These helpers solve a deceptively hard problem: mapping from textual positions (file + offset) to semantic constructs (ModuleId). The `module_for_file()` search through crate def maps is necessary because rust-analyzer's module system is multi-layered—crates, source roots, and modules all interact.

The `module_at_position()` logic handles nested inline modules and block expressions via iterative DefMap traversal. The `std::ptr::eq(&new_block, &def_map)` pointer comparison detects when we've stopped finding deeper blocks, preventing infinite loops. This is idiomatic Rust—use pointer equality for referential checks when structural equality isn't appropriate.

The `mod_at_position()` smallest-range selection algorithm is elegant: iterate all modules, filter to those containing the position, track the smallest range. This handles edge cases like nested inline modules automatically. The `Option` tracking of `size` allows distinguishing "no match" from "found match."

**Contribution Tip:**
Add caching for repeated position queries in the same test. Create `file_at_position()` helper for getting FileId from positions across source roots. Add `item_at_position()` for finding the nearest item (function, struct, etc.). Consider adding error recovery for positions outside any module.

**Common Pitfalls:**
- **Off-by-one in ranges**—`contains()` uses exclusive end, ensure consistency
- **Block expressions**—forgetting that `{ }` can define scopes with their own DefMaps
- **Macro-generated modules**—positions in macro expansions may not have clear ModuleIds
- **Multiple files per module**—`mod.rs` vs `mod_name.rs` can cause confusion

**Related Patterns in Ecosystem:**
- **tree-sitter node_at_position**: Similar smallest-enclosing-node algorithm
- **LSP's textDocument/documentSymbol**: Hierarchical symbol navigation
- **rowan's covering_element**: CST navigation by position
- **ra_ap_syntax's covering_element**: rust-analyzer's own CST traversal
- **syn's buffer/cursor**: Token stream navigation in proc macros

---

## Pattern 12: Salsa Event Logging for Incremental Computation Tests
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/hir-ty/src/test_db.rs
**Category:** Incremental Testing
**Code Example:**
```rust
#[salsa_macros::db]
pub(crate) struct TestDB {
    storage: salsa::Storage<Self>,
    files: Arc<base_db::Files>,
    crates_map: Arc<CratesMap>,
    events: Arc<Mutex<Option<Vec<salsa::Event>>>>,
    nonce: Nonce,
}

impl Default for TestDB {
    fn default() -> Self {
        let events = <Arc<Mutex<Option<Vec<salsa::Event>>>>>::default();
        let mut this = Self {
            storage: salsa::Storage::new(Some(Box::new({
                let events = events.clone();
                move |event| {
                    let mut events = events.lock().unwrap();
                    if let Some(events) = &mut *events {
                        events.push(event);
                    }
                }
            }))),
            events,
            // ... other fields
        };
        // ... initialization
        this
    }
}

impl TestDB {
    pub(crate) fn log(&self, f: impl FnOnce()) -> Vec<salsa::Event> {
        *self.events.lock().unwrap() = Some(Vec::new());
        f();
        self.events.lock().unwrap().take().unwrap()
    }

    pub(crate) fn log_executed(&self, f: impl FnOnce()) -> (Vec<String>, Vec<salsa::Event>) {
        let events = self.log(f);
        let executed = events
            .iter()
            .filter_map(|e| match e.kind {
                salsa::EventKind::WillExecute { database_key } => {
                    let ingredient = (self as &dyn salsa::Database)
                        .ingredient_debug_name(database_key.ingredient_index());
                    Some(ingredient.to_string())
                }
                _ => None,
            })
            .collect();
        (executed, events)
    }
}

// Example test usage:
#[test]
fn incremental_recomputation() {
    let (mut db, file_id) = TestDB::with_single_file("fn foo() {}");

    // Initial computation
    let (executed, _) = db.log_executed(|| {
        let _ = db.infer(file_id);
    });
    assert!(executed.contains(&"InferQuery".to_string()));

    // Modify file
    db.set_file_text(file_id, "fn foo() { let x = 1; }");

    // Check what recomputes
    let (executed, _) = db.log_executed(|| {
        let _ = db.infer(file_id);
    });
    assert!(executed.contains(&"InferQuery".to_string()));
    assert!(!executed.contains(&"CrateDefMapQuery".to_string())); // Shouldn't recompute
}
```
**Why This Matters for Contributors:** Salsa event logging is critical for testing incremental computation correctness. The `Arc<Mutex<Option<Vec<salsa::Event>>>>` pattern allows zero-overhead event collection (the `Option` is `None` normally, `Some(vec)` only during logging). The `log_executed()` helper extracts which queries actually executed, enabling assertions like "changing this file should NOT recompute that query." This tests that Salsa's memoization and dependency tracking work correctly. The pattern prevents subtle performance bugs where queries recompute unnecessarily, which would slow down the IDE.

---

## Expert Commentary: Pattern 12

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Zero-Overhead Event Collection + Incremental Computation Validation + Closure-Based Callback

**Rust-Specific Insight:**
The Salsa event logging pattern demonstrates advanced interior mutability design. The `Arc<Mutex<Option<Vec<salsa::Event>>>>` type is precise: `Arc` for shared ownership across database and callback, `Mutex` for interior mutability, `Option` as a flag for "currently logging" vs. "not logging." When `None`, event collection has zero overhead—no allocations, no mutex contention.

The closure-based callback (`move |event| { ... }`) is stored in a `Box` and passed to Salsa's storage. This creates a feedback loop: Salsa calls the closure, which locks the mutex and pushes events, which the test later retrieves. The design prevents deadlocks because event pushing never queries the database.

The `log_executed()` helper filtering for `WillExecute` events is essential—it ignores verification and other internal events, focusing on actual query execution. The return of `Vec<String>` with query names makes assertions simple: `assert!(executed.contains(&"InferQuery".to_string()))`.

The pattern enables precise incremental computation testing: modify input, check which queries recompute. This catches bugs where queries recompute unnecessarily (performance) or don't recompute when needed (correctness).

**Contribution Tip:**
Add `assert_not_executed()` for negative assertions. Create `diff_executions()` to compare logs between runs. Add event count limits to detect unexpected recomputation storms. Consider structured logging (enums not strings) for type-safe event filtering.

**Common Pitfalls:**
- **Forgetting to enable logging**—`log()` must wrap the operation, not called afterward
- **Nested logging calls**—inner `log()` overwrites outer, use unique logging contexts
- **String-based query matching**—fragile to renames, consider query type IDs
- **Event ordering assumptions**—Salsa may execute queries in any order, only check presence

**Related Patterns in Ecosystem:**
- **tracing-subscriber**: General event collection with similar filtering patterns
- **tokio-console**: Task execution tracking with similar callback patterns
- **criterion's event hooks**: Benchmark event collection for custom analysis
- **perf-event**: Low-level performance counter sampling, different approach
- **cargo's build event stream**: Similar event logging for build process introspection

---

## Pattern 13: #[rust_analyzer::rust_fixture] Attribute Macro
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/test-fixture/src/lib.rs
**Category:** Test Annotation
**Code Example:**
```rust
pub trait WithFixture: Default + ExpandDatabase + SourceDatabase + 'static {
    #[track_caller]
    fn with_single_file(
        #[rust_analyzer::rust_fixture] ra_fixture: &str,
    ) -> (Self, EditionedFileId) {
        let mut db = Self::default();
        let fixture = ChangeFixture::parse(ra_fixture);
        fixture.change.apply(&mut db);
        assert_eq!(fixture.files.len(), 1, "Multiple file found in the fixture");
        let file = EditionedFileId::from_span_guess_origin(&db, fixture.files[0]);
        (db, file)
    }

    #[track_caller]
    fn with_position(#[rust_analyzer::rust_fixture] ra_fixture: &str) -> (Self, FilePosition) {
        let (db, file_id, range_or_offset) = Self::with_range_or_offset(ra_fixture);
        let offset = range_or_offset.expect_offset();
        (db, FilePosition { file_id, offset })
    }
}

// Example usage in tests:
fn check(#[rust_analyzer::rust_fixture] ra_fixture: &str) {
    let (db, files) = TestDB::with_many_files(ra_fixture);
    // ... test implementation
}
```
**Why This Matters for Contributors:** The `#[rust_analyzer::rust_fixture]` attribute serves as documentation and potentially enables tooling support for fixture strings. It marks parameters that contain test fixtures in the mini-DSL format, making it immediately clear which strings are interpreted specially. This is a form of "semantic markup" for test code—even without proc macro expansion, the attribute communicates intent. It could enable IDE features like syntax highlighting inside fixture strings, fixture validation, or autocomplete for metadata directives. The pattern shows how custom attributes can document domain-specific languages embedded in strings.

---

## Expert Commentary: Pattern 13

**Idiomatic Rating: ⭐⭐⭐⭐ (4/5)**

**Pattern Classification:** Semantic Annotation + Documentation Pattern + Tooling Hook

**Rust-Specific Insight:**
The `#[rust_analyzer::rust_fixture]` attribute is a clever use of custom attributes for documentation and potential tooling. Unlike most attributes that affect code generation or behavior, this one primarily serves as semantic markup—it signals "this parameter contains a fixture in our mini-DSL."

The attribute enables several benefits: (1) Documentation—readers immediately know which strings are fixtures; (2) Tooling potential—IDEs could provide fixture-aware features like syntax highlighting inside strings, validation, or autocomplete for metadata directives; (3) Future extensibility—the attribute could be extended with parameters like `#[rust_analyzer::rust_fixture(strict)]` to enable validation modes.

The pattern demonstrates Rust's attribute system flexibility. Even without proc macro implementation, the attribute compiles (assuming it's declared somewhere) and provides value through documentation. This is "design for the future"—laying groundwork for potential tooling enhancements.

**Contribution Tip:**
Implement proc macro validation for fixture syntax at compile time. Add IDE support for syntax highlighting inside fixture strings. Create `rust_fixture_validate!` macro that parses fixtures at compile time for early error detection. Consider adding `#[rust_fixture(minicore = "option")]` parameter validation.

**Common Pitfalls:**
- **Attribute without declaration**—need `#[allow(unused_attributes)]` or actual proc macro crate
- **Over-reliance on tooling**—attribute is documentation, not enforcement without implementation
- **Inconsistent usage**—forgetting attribute on some fixture parameters breaks grep-ability
- **No runtime effect**—developers may assume attribute validates input, but it's passive

**Related Patterns in Ecosystem:**
- **serde's `#[serde(rename)]`**: Attribute-based configuration with proc macro implementation
- **tracing's `#[instrument]`**: Attribute for behavior modification, similar extensibility
- **diesel's `#[table_name]`**: Domain-specific attribute for ORM schema
- **rocket's `#[get]` routes**: Attribute-driven routing DSL
- **sqlx's `#[sqlx::test]`**: Test-specific attributes for setup/teardown

---

## Pattern 14: Normalized Test Configuration Constants
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/ide-assists/src/tests.rs
**Category:** Test Configuration
**Code Example:**
```rust
pub(crate) const TEST_CONFIG: AssistConfig = AssistConfig {
    snippet_cap: SnippetCap::new(true),
    allowed: None,
    insert_use: InsertUseConfig {
        granularity: ImportGranularity::Crate,
        prefix_kind: hir::PrefixKind::Plain,
        enforce_granularity: true,
        group: true,
        skip_glob_imports: true,
    },
    prefer_no_std: false,
    prefer_prelude: true,
    prefer_absolute: false,
    assist_emit_must_use: false,
    term_search_fuel: 400,
    term_search_borrowck: true,
    code_action_grouping: true,
    expr_fill_default: ExprFillDefaultMode::Todo,
    prefer_self_ty: false,
    show_rename_conflicts: true,
};

pub(crate) const TEST_CONFIG_NO_SNIPPET_CAP: AssistConfig = AssistConfig {
    snippet_cap: None,
    allowed: None,
    insert_use: InsertUseConfig {
        granularity: ImportGranularity::Crate,
        prefix_kind: hir::PrefixKind::Plain,
        enforce_granularity: true,
        group: true,
        skip_glob_imports: true,
    },
    // ... same as TEST_CONFIG except snippet_cap
    prefer_no_std: false,
    prefer_prelude: true,
    // ... all other fields
};

pub(crate) const TEST_CONFIG_IMPORT_ONE: AssistConfig = AssistConfig {
    snippet_cap: SnippetCap::new(true),
    allowed: None,
    insert_use: InsertUseConfig {
        granularity: ImportGranularity::One,  // Only difference
        prefix_kind: hir::PrefixKind::Plain,
        enforce_granularity: true,
        group: true,
        skip_glob_imports: true,
    },
    // ... rest same as TEST_CONFIG
};
```
**Why This Matters for Contributors:** Centralized test configuration constants ensure consistency across test suites. Instead of each test constructing an `AssistConfig` with potentially different defaults, they use shared constants like `TEST_CONFIG`. The pattern of creating variants (`TEST_CONFIG_NO_SNIPPET_CAP`, `TEST_CONFIG_IMPORT_ONE`) that differ in one dimension makes it clear what's being tested. This is superior to builder patterns or partial config structs because it's explicit, const-evaluable, and grep-able. When default behavior needs to change, updating the constant fixes all tests at once. The repetition is intentional—it makes each variant's full configuration obvious.

---

## Expert Commentary: Pattern 14

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Centralized Configuration Constants + Explicit Variants + Const Evaluation

**Rust-Specific Insight:**
The normalized configuration constant pattern is deceptively simple but architecturally critical. By defining `TEST_CONFIG` as a `const`, all tests share the same baseline behavior. Variants like `TEST_CONFIG_NO_SNIPPET_CAP` differ in exactly one dimension, making the difference explicit and searchable.

The pattern prevents a common anti-pattern: tests constructing configs ad-hoc with `AssistConfig { snippet_cap: ..., ..Default::default() }`. This silently breaks when new fields are added to `AssistConfig`—the `Default` impl might have different values than tests expect. Explicit constants make changes visible and intentional.

The repetition is intentional and valuable. Each variant shows its *complete* configuration, not just differences from a base. This improves readability—no mental merging of partial configs. It also makes refactoring safer—changing `TEST_CONFIG` won't silently affect `TEST_CONFIG_IMPORT_ONE` unless explicitly updated.

The `const` qualifier enables compile-time evaluation, ensuring zero runtime overhead. These aren't just convenient—they're free.

**Contribution Tip:**
Add `_BASE` suffix to the main config (`TEST_CONFIG_BASE`) to make variants' relationship clearer. Document why each variant exists in comments. Create a test that asserts variants differ only in expected dimensions. Consider macros for generating variants if the list grows large.

**Common Pitfalls:**
- **Using `Default::default()` in tests**—bypasses the normalization, creates inconsistency
- **Forgetting to update variants**—when fields are added, variants may get stale defaults
- **Too many variants**—if every test needs its own config, the pattern breaks down
- **Incomplete documentation**—variants should explain *why* the difference matters

**Related Patterns in Ecosystem:**
- **lazy_static!**: Runtime-initialized global configs, different trade-offs
- **config crate**: Layered configuration loading, more dynamic approach
- **Default trait**: Rust's standard default values, but less explicit
- **Builder pattern**: More flexible but higher overhead than const configs
- **feature flags**: Compile-time configuration, complementary pattern

---

## Pattern 15: with_single_file() Database Builder Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/test-fixture/src/lib.rs
**Category:** Database Construction
**Code Example:**
```rust
pub trait WithFixture: Default + ExpandDatabase + SourceDatabase + 'static {
    #[track_caller]
    fn with_single_file(
        #[rust_analyzer::rust_fixture] ra_fixture: &str,
    ) -> (Self, EditionedFileId) {
        let mut db = Self::default();
        let fixture = ChangeFixture::parse(ra_fixture);
        fixture.change.apply(&mut db);
        assert_eq!(fixture.files.len(), 1, "Multiple file found in the fixture");
        let file = EditionedFileId::from_span_guess_origin(&db, fixture.files[0]);
        (db, file)
    }

    #[track_caller]
    fn with_many_files(
        #[rust_analyzer::rust_fixture] ra_fixture: &str,
    ) -> (Self, Vec<EditionedFileId>) {
        let mut db = Self::default();
        let fixture = ChangeFixture::parse(ra_fixture);
        fixture.change.apply(&mut db);
        assert!(fixture.file_position.is_none());
        let files = fixture
            .files
            .into_iter()
            .map(|file| EditionedFileId::from_span_guess_origin(&db, file))
            .collect();
        (db, files)
    }

    #[track_caller]
    fn with_position(#[rust_analyzer::rust_fixture] ra_fixture: &str) -> (Self, FilePosition) {
        let (db, file_id, range_or_offset) = Self::with_range_or_offset(ra_fixture);
        let offset = range_or_offset.expect_offset();
        (db, FilePosition { file_id, offset })
    }

    #[track_caller]
    fn with_range(#[rust_analyzer::rust_fixture] ra_fixture: &str) -> (Self, FileRange) {
        let (db, file_id, range_or_offset) = Self::with_range_or_offset(ra_fixture);
        let range = range_or_offset.expect_range();
        (db, FileRange { file_id, range })
    }
}

impl<DB: ExpandDatabase + SourceDatabase + Default + 'static> WithFixture for DB {}

// Example usage:
fn test_completion() {
    let (db, position) = TestDB::with_position(
        r#"
fn main() {
    let x = 42;
    x.$0
}
        "#,
    );
    let completions = get_completions(&db, position);
    // ... assertions
}
```
**Why This Matters for Contributors:** The `with_*` family of database constructors provides a uniform interface for setting up test databases from fixtures. The generic implementation (`impl<DB: ...> WithFixture for DB`) means any database type implementing the required traits gets these methods for free. The pattern handles different scenarios: single file (`with_single_file`), multiple files (`with_many_files`), cursor position (`with_position`), text range (`with_range`). The `#[track_caller]` ensures errors point to the test, not the constructor. The assertions (e.g., "exactly one file") provide clear error messages when fixtures are malformed. This is a textbook example of using traits to add convenience methods without coupling.

---

## Expert Commentary: Pattern 15

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:** Generic Trait Implementation + Builder API + Named Constructor Pattern

**Rust-Specific Insight:**
The `WithFixture` trait demonstrates Rust's trait system at its best. The generic implementation (`impl<DB: ExpandDatabase + SourceDatabase + Default + 'static> WithFixture for DB {}`) means any database type gets these methods for free. This is "blanket implementation" in action—adding capabilities without inheritance.

The family of `with_*` constructors provides a clean API surface: `with_single_file()` returns `(DB, FileId)`, `with_position()` returns `(DB, FilePosition)`, `with_range()` returns `(DB, FileRange)`. Each constructor parses the same fixture format but extracts different information, reducing duplication while maintaining type safety.

The `#[track_caller]` on all methods is essential—when assertions fail (like "Multiple file found in fixture"), the error points to the test invocation, not into `with_single_file()`. This makes debugging trivial.

The pattern scales beautifully: new database types automatically gain fixture support by implementing the prerequisite traits. Tests read naturally: `let (db, pos) = TestDB::with_position(fixture)` is clear and concise.

**Contribution Tip:**
Add `with_files_and_positions()` for multi-cursor scenarios. Create `with_diagnostics()` that parses expected diagnostics from annotations. Add builder methods like `.with_config()` for customizing fixture parsing. Consider adding `with_project()` for multi-crate setups.

**Common Pitfalls:**
- **Wrong constructor**—using `with_single_file()` when fixture has multiple files causes assertion failure
- **Missing `$0` markers**—`with_position()` panics if no cursor marker in fixture
- **Fixture parsing errors**—malformed metadata silently ignored or causes cryptic failures
- **Type inference issues**—sometimes need `TestDB::with_position(...)` instead of `.with_position(...)`

**Related Patterns in Ecosystem:**
- **Default trait + builder**: Common pattern for test setup, but less flexible
- **test-case crate**: Parameterized test generation, complementary approach
- **rstest fixtures**: Runtime fixture injection, different philosophy
- **proptest strategies**: Generated test data, orthogonal to declarative fixtures
- **criterion's benchmark groups**: Similar API family for different contexts

---

## Summary: Test Infrastructure Patterns from rust-analyzer

These 15 patterns represent a comprehensive, industrial-strength testing framework extracted from one of Rust's most sophisticated projects. Together, they demonstrate:

### Key Architectural Insights

1. **Multi-Layer DSL Design**: The fixture system (Pattern 1) + MiniCore (Pattern 2) + cursor markers (Pattern 3) + annotations (Pattern 4) form a complete embedded language for test specification. Each layer is independently useful but composes beautifully.

2. **Zero-Cost Abstractions in Testing**: Event logging (Patterns 5, 12), database builders (Patterns 11, 15), and test helpers (Pattern 10) provide rich functionality with zero runtime overhead when not used.

3. **Statistical Rigor**: AssertLinear (Pattern 8) brings academic-grade statistical analysis to performance testing, far beyond naive timing assertions.

4. **Bidirectional Validation**: Type checking tests (Pattern 7) validate both that code matches expectations AND that expectations are comprehensive, preventing test drift.

5. **Trait-Based Extensibility**: Multiple patterns (1, 15) use blanket trait implementations to provide functionality to any compatible type, demonstrating Rust's trait system excellence.

### Innovation Highlights

- **In-source snapshot testing** (Pattern 6): `expect_test` eliminates separate `.snap` files, a genuine improvement over prior art
- **Region-based conditional compilation** (Pattern 2): Novel approach to creating minimal standard library subsets
- **Visual alignment annotations** (Pattern 4): The `//^^^` pattern is intuitive and handles complex scenarios elegantly
- **Multi-round flake resistance** (Pattern 8): Statistical approach to performance testing that handles environmental noise

### Patterns by Maturity Level

**Production-Ready (Use Immediately):**
- Patterns 1, 3, 6, 7, 10, 14, 15: Battle-tested, comprehensive, ready for adoption

**Advanced (Requires Deep Understanding):**
- Patterns 2, 4, 5, 8, 11, 12: Powerful but need careful integration and testing expertise

**Specialized (Domain-Specific):**
- Patterns 9, 13: Valuable in specific contexts (benchmarking, documentation)

## Contribution Readiness Checklist

Use this checklist when contributing tests to rust-analyzer or implementing similar patterns in your project:

### Fixture Design (Patterns 1, 2, 9, 13)
- [ ] Fixtures use `//- /path` metadata for multi-file scenarios
- [ ] `minicore` flags specified minimally (only required traits)
- [ ] Global metadata (toolchain, proc_macros) placed at fixture top
- [ ] Escape sequences (`\$0`) used for literal markers in test data
- [ ] `#[rust_analyzer::rust_fixture]` attribute marks fixture parameters

### Cursor & Annotation Markers (Patterns 3, 4)
- [ ] `$0` markers placed at semantically meaningful positions
- [ ] Two `$0` markers for range-based operations (refactorings, selections)
- [ ] `//^^^` annotations align visually with target code
- [ ] Multiline annotations use `|` continuation correctly
- [ ] Trailing `$` used when preserving whitespace matters

### Database & State Management (Patterns 5, 11, 12, 15)
- [ ] `with_*` constructors chosen appropriately (`with_position` vs `with_range`)
- [ ] `#[track_caller]` on all test helpers for better error messages
- [ ] Salsa durabilities set correctly (HIGH for test-stable data)
- [ ] Event logging used to validate incremental recomputation
- [ ] `Nonce` or equivalent ensures test isolation

### Test Assertions (Patterns 7, 8, 10)
- [ ] Type annotations bidirectionally validated (check + ensure all checked)
- [ ] Performance tests use `AssertLinear` or equivalent statistical approach
- [ ] IDE assists tested with clear before/after fixtures
- [ ] Test configs use normalized constants, not ad-hoc construction
- [ ] Benchmarks include sanity checks (hashes) to prevent optimization away

### Documentation & Maintainability (Patterns 6, 13, 14)
- [ ] `expect_test` used for complex output (prefer over manual string matching)
- [ ] Test config variants documented with rationale
- [ ] Fixture metadata comprehensively documented in trait docs
- [ ] Helper functions annotated with purpose and usage examples
- [ ] Slow tests gated with `skip_slow_tests()` or equivalent

### Error Handling & Debugging
- [ ] Assertion messages include context (ranges, file paths, etc.)
- [ ] Panics provide diagnostic output (plots, event logs, etc.)
- [ ] Test failures point to actual test location (via `#[track_caller]`)
- [ ] Fixture parse errors give actionable messages
- [ ] Unchecked annotations cause test failures (prevent drift)

### Performance & Scalability
- [ ] Event logging uses `Option<Vec<_>>` for zero-cost when disabled
- [ ] Large fixtures generated programmatically or loaded from files
- [ ] Statistical tests run multiple rounds to handle environmental noise
- [ ] Benchmark fixtures balanced between synthetic and real-world samples
- [ ] Tests don't duplicate expensive setup (use database builders)

---

**Total Pattern Count:** 15 core patterns
**Idiomatic Rating Average:** 4.8/5 stars
**Production Readiness:** ⭐⭐⭐⭐⭐ (Extensively battle-tested in rust-analyzer)
**Learning Curve:** Moderate to Advanced (layered complexity, start with Patterns 1, 3, 6)
**Ecosystem Impact:** High (patterns applicable to any Rust project with complex testing needs)

These patterns represent the state-of-the-art in Rust testing infrastructure. They demonstrate that with careful design, test code can be as elegant, type-safe, and maintainable as production code—an embodiment of Rust's philosophy applied to the testing domain.

---
