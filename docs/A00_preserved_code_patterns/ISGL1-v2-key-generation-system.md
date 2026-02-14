# ISGL1 v2 Key Generation System

**What it is**: The identity system that gives every code entity (function, struct, class, module, etc.) a unique, stable key across all 15 supported languages. "ISGL1" = Inter-System Graph Layer 1.

**Why it exists**: LLMs and graph algorithms need a stable way to refer to code entities. Line numbers change when you edit a file. ISGL1 v2 uses birth timestamps derived from deterministic hashes, so keys survive refactors.

**Where it lives**:
- Core functions: `crates/parseltongue-core/src/isgl1_v2.rs`
- Key generator: `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs`
- Entity types: `crates/parseltongue-core/src/entities.rs`

---

## 1. The Key Format

```
{language}:{type}:{sanitized_name}:{semantic_path}:T{birth_timestamp}
```

Real examples:
```
rust:fn:handle_auth:__src_auth:T1706284800
python:class:UserService:__app_services_user:T1689012345
csharp:fn:GetAsync__lt__T__gt__:__Controllers_ApiController:T1701234567
```

Five colon-separated segments:
1. **language** — `rust`, `python`, `javascript`, `typescript`, `java`, `c`, `cpp`, `go`, `ruby`, `php`, `csharp`, `swift`, `kotlin`, `scala`, `sql`
2. **type** — `fn`, `method`, `class`, `struct`, `enum`, `trait`, `impl`, `mod`, `namespace`, `typedef`, `var`, `const`, `test`, `table`, `view`
3. **sanitized_name** — entity name with generic syntax escaped (v2.1)
4. **semantic_path** — file path without extension, prefixed with `__`
5. **birth_timestamp** — `T` prefix + deterministic Unix timestamp

---

## 2. Key Generation Pipeline

The key is assembled in `Isgl1KeyGeneratorImpl::format_key()`:

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:176-212

fn format_key(&self, entity: &ParsedEntity) -> String {
    use parseltongue_core::isgl1_v2::{
        compute_birth_timestamp,
        extract_semantic_path,
        sanitize_entity_name_for_isgl1,  // v2.1: Add sanitization
    };

    let type_str = match entity.entity_type {
        EntityType::Function => "fn",
        EntityType::Class => "class",
        EntityType::Method => "method",
        EntityType::Struct => "struct",
        EntityType::Enum => "enum",
        EntityType::Trait => "trait",
        EntityType::Impl => "impl",
        EntityType::Module => "mod",
        EntityType::Namespace => "namespace",
        EntityType::Typedef => "typedef",
        EntityType::Variable => "var",
        EntityType::Table => "table",  // v1.5.6: SQL table
        EntityType::View => "view",    // v1.5.6: SQL view
    };

    // ISGL1 v2.1: Sanitize entity name to handle generic types
    let sanitized_name = sanitize_entity_name_for_isgl1(&entity.name);
    let semantic_path = extract_semantic_path(&entity.file_path);
    let birth_timestamp = compute_birth_timestamp(&entity.file_path, &entity.name);

    format!(
        "{}:{}:{}:{}:T{}",
        entity.language,
        type_str,
        sanitized_name,
        semantic_path,
        birth_timestamp
    )
}
```

Three helper functions feed into `format!`. Each one below.

---

## 3. Semantic Path Extraction

Converts a file path into a CozoDB-safe path segment.

```rust
// crates/parseltongue-core/src/isgl1_v2.rs:127-141

pub fn extract_semantic_path(file_path: &str) -> String {
    // Remove file extension
    let without_ext = if let Some(pos) = file_path.rfind('.') {
        &file_path[..pos]
    } else {
        file_path
    };

    // Replace path separators and special chars with underscore
    let sanitized = without_ext
        .replace(['/', '\\', '-', '.'], "_");

    // Add leading underscores (ISGL1 convention)
    format!("__{}", sanitized)
}
```

Examples:
```
"src/auth.rs"           -> "__src_auth"
"crates/core/lib.py"    -> "__crates_core_lib"
"src\\models\\user.cs"  -> "__src_models_user"     (Windows backslashes)
```

---

## 4. Birth Timestamp Computation

Generates a deterministic Unix timestamp from `file_path + entity_name`. Same entity always gets the same timestamp, even across full re-indexes.

```rust
// crates/parseltongue-core/src/isgl1_v2.rs:162-176

pub fn compute_birth_timestamp(file_path: &str, entity_name: &str) -> i64 {
    // Create deterministic hash from file + entity name
    let mut hasher = DefaultHasher::new();
    file_path.hash(&mut hasher);
    entity_name.hash(&mut hasher);
    let hash = hasher.finish();

    // Convert hash to reasonable timestamp range
    // Use modulo to keep it within recent years (2020-2030 range)
    let base_timestamp = 1577836800; // 2020-01-01 00:00:00 UTC
    let range = 315360000; // ~10 years in seconds
    let offset = (hash % range as u64) as i64;

    base_timestamp + offset
}
```

**Why deterministic**: `DefaultHasher` with the same two inputs always produces the same `u64`. The modulo maps it to 2020-2030 epoch range. No randomness, no clock dependency.

**Why timestamps and not line numbers**: Line numbers shift when you add/remove code above an entity. Birth timestamps stay stable. This is the entire point of ISGL1 v2 — it solved the incremental indexing false positive problem where every key changed on every re-index.

---

## 5. Entity Name Sanitization (v2.1)

Generic type syntax (`<`, `>`, `,`, `[`, `]`, `{`, `}`) breaks CozoDB query parsing. v2.1 replaces these with double-underscore tokens.

```rust
// crates/parseltongue-core/src/isgl1_v2.rs:44-75

pub fn sanitize_entity_name_for_isgl1(name: &str) -> String {
    let mut result = String::with_capacity(name.len() * 2);
    let chars: Vec<char> = name.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Handle comma followed by space as single unit
        if ch == ',' && i + 1 < chars.len() && chars[i + 1] == ' ' {
            result.push_str("__c__");
            i += 2; // Skip both comma and space
            continue;
        }

        match ch {
            ' ' => result.push('_'),
            '<' => result.push_str("__lt__"),
            '>' => result.push_str("__gt__"),
            ',' => result.push_str("__c__"),
            '[' => result.push_str("__lb__"),
            ']' => result.push_str("__rb__"),
            '{' => result.push_str("__lc__"),
            '}' => result.push_str("__rc__"),
            _ => result.push(ch),
        }

        i += 1;
    }

    result
}
```

Replacement table:

| Character | Token | Mnemonic |
|-----------|-------|----------|
| `<` | `__lt__` | less than |
| `>` | `__gt__` | greater than |
| `, ` | `__c__` | comma (eats trailing space) |
| `,` | `__c__` | comma (no space) |
| `[` | `__lb__` | left bracket |
| `]` | `__rb__` | right bracket |
| `{` | `__lc__` | left curly |
| `}` | `__rc__` | right curly |
| ` ` | `_` | space to single underscore |

Examples:
```
"List<string>"                  -> "List__lt__string__gt__"
"Dictionary<string, object>"    -> "Dictionary__lt__string__c__object__gt__"
"int[]"                         -> "int__lb____rb__"
```

**Why this matters**: C#, C++, Java, and TypeScript all have generic types. Without sanitization, a key like `csharp:fn:GetAsync<T>:__path:T123` would break CozoDB's Datalog parser because `<` and `>` are query operators.

---

## 6. Content Hash (SHA-256)

Used for change detection during incremental reindexing. Not part of the key itself, but stored alongside the entity.

```rust
// crates/parseltongue-core/src/isgl1_v2.rs:200-205

pub fn compute_content_hash(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
```

Returns a 64-character lowercase hex string. Whitespace-sensitive (formatting changes produce different hashes).

---

## 7. Three-Priority Entity Matching (Incremental Reindex)

When re-indexing a file, new entities must be matched against old entities to preserve their ISGL1 keys (and thus their graph edges). Three priority levels:

```rust
// crates/parseltongue-core/src/isgl1_v2.rs:301-335

pub fn match_entity_with_old_index(
    new_entity: &EntityCandidate,
    old_entities: &[OldEntity],
) -> EntityMatchResult {
    // Priority 1: Try hash match first (most reliable)
    // Check content hash + name + file to ensure same entity
    if let Some(matched) = old_entities.iter().find(|old| {
        old.content_hash == new_entity.content_hash
            && old.name == new_entity.name
            && old.file_path == new_entity.file_path
    }) {
        return EntityMatchResult::ContentMatch {
            old_key: matched.key.clone(),
        };
    }

    // Priority 2: Try position match (same name, file, approximate position)
    // Only executed if hash match fails (content changed)
    if let Some(matched) = old_entities.iter().find(|old| {
        old.name == new_entity.name
            && old.file_path == new_entity.file_path
            && is_within_position_tolerance(
                old.line_range,
                new_entity.line_range,
                POSITION_TOLERANCE_LINES,
            )
    }) {
        return EntityMatchResult::PositionMatch {
            old_key: matched.key.clone(),
        };
    }

    // Priority 3: No match found - treat as new entity
    EntityMatchResult::NewEntity
}
```

The three outcomes:

```rust
// crates/parseltongue-core/src/isgl1_v2.rs:241-248

pub enum EntityMatchResult {
    /// Priority 1: Hash matched (content unchanged)
    ContentMatch { old_key: String },
    /// Priority 2: Name/position matched (content changed)
    PositionMatch { old_key: String },
    /// Priority 3: New entity (will get new birth timestamp)
    NewEntity,
}
```

Position tolerance is +/-10 lines:

```rust
// crates/parseltongue-core/src/isgl1_v2.rs:254
pub const POSITION_TOLERANCE_LINES: i32 = 10;
```

The matching data structures:

```rust
// crates/parseltongue-core/src/isgl1_v2.rs:211-232

pub struct EntityCandidate {
    pub name: String,
    pub entity_type: EntityType,
    pub file_path: String,
    pub line_range: (usize, usize),
    pub content_hash: String,
    pub code: String,
}

pub struct OldEntity {
    pub key: String,              // ISGL1 v2 key with birth timestamp
    pub name: String,
    pub file_path: String,
    pub line_range: (usize, usize),
    pub content_hash: String,
}
```

---

## 8. Entity Types

Two EntityType enums exist. The core version (in `entities.rs`) is the source of truth for storage. The pt01 version (in `isgl1_generator.rs`) is the extraction-side enum.

### Core EntityType (parseltongue-core)

```rust
// crates/parseltongue-core/src/entities.rs:101-123

pub enum EntityType {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Interface,
    Module,
    ImplBlock {
        trait_name: Option<String>,
        struct_name: String,
    },
    Macro,
    ProcMacro,
    TestFunction,
    Class,
    Variable,
    Constant,
    Table,      // SQL CREATE TABLE
    View,       // SQL CREATE VIEW
}
```

### pt01 EntityType (extraction side)

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:94-122

pub enum EntityType {
    Function,
    Class,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Namespace,
    Typedef,
    Variable,
    Table,
    View,
}
```

The pt01 version is simpler (no ImplBlock inner fields, no Macro/ProcMacro/TestFunction/Constant). Mapping between them happens in `map_query_entity_type()`.

---

## 9. The Isgl1Key Newtype

Type-safe wrapper to prevent mixing ISGL1 keys with regular strings.

```rust
// crates/parseltongue-core/src/entities.rs:987-1027

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Isgl1Key(String);

impl Isgl1Key {
    /// Creates new ISGL1 key, validating non-empty
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        if key.is_empty() {
            Err(ParseltongError::InvalidIsgl1Key {
                key,
                reason: "ISGL1 key cannot be empty".to_string(),
            })
        } else {
            Ok(Self(key))
        }
    }

    /// Creates key without validation (for trusted sources like database reads)
    pub fn new_unchecked(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// Returns key as string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the Isgl1Key and returns the inner String
    pub fn into_inner(self) -> String {
        self.0
    }
}
```

`#[repr(transparent)]` means the newtype has zero runtime cost - same memory layout as a bare `String`.

---

## 10. Dependency Edges

Edges connect two ISGL1 keys with a relationship type.

```rust
// crates/parseltongue-core/src/entities.rs:1054-1062

pub enum EdgeType {
    /// Function call relationship (A calls B)
    Calls,
    /// Usage relationship (A uses B's type/interface)
    Uses,
    /// Trait implementation (A implements trait B)
    Implements,
}
```

```rust
// crates/parseltongue-core/src/entities.rs:1126-1136

pub struct DependencyEdge {
    /// Source entity ISGL1 key
    pub from_key: Isgl1Key,
    /// Target entity ISGL1 key
    pub to_key: Isgl1Key,
    /// Type of dependency relationship
    pub edge_type: EdgeType,
    /// Source code location where relationship occurs (optional)
    pub source_location: Option<String>,
}
```

Builder pattern for ergonomic construction:

```rust
let edge = DependencyEdge::builder()
    .from_key("rust:fn:main:__src_main:T1706284800")
    .to_key("rust:fn:helper:__src_utils:T1689012345")
    .edge_type(EdgeType::Calls)
    .source_location("src/main.rs:5")
    .build()
    .unwrap();
```

---

## 11. Thread-Local Parser Architecture

Parsing is parallelized via Rayon. Each thread gets its own parser and extractor instance via `thread_local!`, eliminating all mutex contention.

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:40-54

thread_local! {
    static THREAD_PARSERS: std::cell::RefCell<HashMap<Language, Parser>> =
        std::cell::RefCell::new(HashMap::new());
}

thread_local! {
    static THREAD_EXTRACTOR: std::cell::RefCell<Option<QueryBasedExtractor>> =
        std::cell::RefCell::new(None);
}
```

The generator struct itself has zero shared state:

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:141-143

pub struct Isgl1KeyGeneratorImpl {
    // Phase 5: All state moved to thread_local! storage
}
```

Parser creation per thread, amortized across files:

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:273-319

fn get_thread_local_parser_instance(&self, language: Language)
    -> std::result::Result<(), StreamerError>
{
    THREAD_PARSERS.with(|parsers| {
        let mut parsers = parsers.borrow_mut();

        // Return early if parser already exists for this language
        if parsers.contains_key(&language) {
            return Ok(());
        }

        // Create new parser for this language
        let mut parser = Parser::new();

        let ts_lang: tree_sitter::Language = match language {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            Language::Java => tree_sitter_java::LANGUAGE.into(),
            Language::C => tree_sitter_c::LANGUAGE.into(),
            Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Language::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            Language::Php => tree_sitter_php::LANGUAGE_PHP.into(),
            Language::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Language::Swift => tree_sitter_swift::LANGUAGE.into(),
            Language::Scala => tree_sitter_scala::LANGUAGE.into(),
            _ => {
                return Err(StreamerError::ParsingError {
                    file: "".to_string(),
                    reason: format!("Unsupported language: {:?}", language),
                });
            }
        };

        parser.set_language(&ts_lang)
            .map_err(|e| StreamerError::ParsingError {
                file: "".to_string(),
                reason: format!("Failed to set parser language: {}", e),
            })?;

        parsers.insert(language, parser);
        Ok(())
    })
}
```

---

## 12. Entity Extraction Pipeline

Two-pass hybrid approach:

1. **Pass 1** (all 12 languages): `QueryBasedExtractor` uses `.scm` tree-sitter query files for declarative entity extraction.
2. **Pass 2** (Rust only): Post-processing enriches entities with `#[test]`/`#[tokio::test]` attribute metadata.

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:490-543

fn extract_entities(
    &self,
    _tree: &Tree,
    source: &str,
    file_path: &Path,
    language: Language,
    entities: &mut Vec<ParsedEntity>,
    dependencies: &mut Vec<DependencyEdge>,
    extraction_warnings: &mut Vec<String>,
) {
    // Phase 5.1: Use thread-local QueryBasedExtractor (no mutex!)
    if let Err(e) = self.get_thread_extractor_instance_safely() {
        extraction_warnings.push(format!(
            "[EXTRACT_FAIL] {}: Failed to initialize thread extractor: {}",
            file_path.display(), e
        ));
        return;
    }

    THREAD_EXTRACTOR.with(|extractor_cell| {
        let mut extractor_opt = extractor_cell.borrow_mut();
        let extractor = extractor_opt.as_mut().expect("Extractor should be initialized");

        match extractor.parse_source(source, file_path, language) {
            Ok((query_entities, query_deps)) => {
                // Convert QueryBasedExtractor entities to pt01 ParsedEntity format
                for query_entity in query_entities {
                    entities.push(ParsedEntity {
                        entity_type: self.map_query_entity_type(&query_entity.entity_type),
                        name: query_entity.name,
                        language: query_entity.language,
                        line_range: query_entity.line_range,
                        file_path: query_entity.file_path,
                        metadata: query_entity.metadata,
                    });
                }

                // Rust-specific attribute parsing
                if language == Language::Rust {
                    self.enrich_rust_entities_with_attributes(entities, source);
                }

                // Dependency edges from query-based extraction
                dependencies.extend(query_deps);
            }
            Err(e) => {
                extraction_warnings.push(format!(
                    "[EXTRACT_FAIL] {}: QueryBasedExtractor failed for {:?}: {}",
                    file_path.display(), language, e
                ));
            }
        }
    });
}
```

The Rust attribute enrichment scans source lines for `#[test]`, `#[tokio::test]`, `#[async_test]` and marks matching entities with `metadata["is_test"] = "true"`:

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:423-473

fn enrich_rust_entities_with_attributes(
    &self,
    entities: &mut [ParsedEntity],
    source: &str,
) {
    let lines: Vec<&str> = source.lines().collect();

    // Build map of entity start lines for O(1) lookup
    let mut entity_lines: std::collections::HashMap<usize, &mut ParsedEntity> =
        std::collections::HashMap::new();

    for entity in entities.iter_mut() {
        entity_lines.insert(entity.line_range.0, entity);
    }

    // Scan source for attributes and match to entities
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed == "#[test]" || trimmed == "#[tokio::test]" || trimmed == "#[async_test]" {
            for next_idx in (idx + 1)..lines.len() {
                let next_line = lines[next_idx].trim();

                if next_line.starts_with("#[") {
                    continue;
                }

                if next_line.starts_with("fn ")
                    || next_line.starts_with("async fn ")
                    || next_line.starts_with("pub fn ")
                    || next_line.starts_with("pub async fn ")
                {
                    let entity_line = next_idx + 1;
                    if let Some(entity) = entity_lines.get_mut(&entity_line) {
                        entity.metadata.insert("is_test".to_string(), "true".to_string());
                    }
                    break;
                }

                if !next_line.is_empty() {
                    break;
                }
            }
        }
    }
}
```

---

## 13. The ParsedEntity Struct

Intermediate representation between tree-sitter extraction and CozoDB storage.

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:76-84

pub struct ParsedEntity {
    pub entity_type: EntityType,
    pub name: String,
    pub language: Language,
    pub line_range: (usize, usize),
    pub file_path: String,
    pub metadata: HashMap<String, String>,
}
```

---

## 14. The Isgl1KeyGenerator Trait

Public interface that pt01 streamer calls.

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:57-73

pub trait Isgl1KeyGenerator: Send + Sync {
    /// Generate ISGL1 key from parsed code entity
    fn generate_key(&self, entity: &ParsedEntity) -> Result<String>;

    /// Parse source code into structured entities AND dependency edges
    ///
    /// Returns (entities, dependencies, warnings)
    fn parse_source(&self, source: &str, file_path: &Path)
        -> Result<(Vec<ParsedEntity>, Vec<DependencyEdge>, Vec<String>)>;

    /// Get supported language for file extension
    fn get_language_type(&self, file_path: &Path) -> Result<Language>;
}
```

Factory creates `Arc<dyn Isgl1KeyGenerator>` for shared ownership across threads:

```rust
// crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:547-555

pub struct Isgl1KeyGeneratorFactory;

impl Isgl1KeyGeneratorFactory {
    pub fn new() -> Arc<dyn Isgl1KeyGenerator> {
        Arc::new(Isgl1KeyGeneratorImpl::new())
    }
}
```

---

## 15. CodeEntity (Storage Side)

The full entity stored in CozoDB. ISGL1 v2 added three fields: `birth_timestamp`, `content_hash`, `semantic_path`.

```rust
// crates/parseltongue-core/src/entities.rs:479-515

pub struct CodeEntity {
    pub isgl1_key: String,
    pub temporal_state: TemporalState,
    pub interface_signature: InterfaceSignature,
    pub current_code: Option<String>,
    pub future_code: Option<String>,
    pub tdd_classification: TddClassification,
    pub lsp_metadata: Option<LspMetadata>,
    pub metadata: EntityMetadata,
    pub entity_class: EntityClass,

    // ISGL1 v2: Stable entity identity fields
    pub birth_timestamp: Option<i64>,
    pub content_hash: Option<String>,
    pub semantic_path: Option<String>,
}
```

Constructor with v2 fields:

```rust
// crates/parseltongue-core/src/entities.rs:679-692

pub fn new_with_v2_fields(
    isgl1_key: String,
    interface_signature: InterfaceSignature,
    entity_class: EntityClass,
    birth_timestamp: i64,
    content_hash: String,
    semantic_path: String,
) -> Result<Self> {
    let mut entity = Self::new(isgl1_key, interface_signature, entity_class)?;
    entity.birth_timestamp = Some(birth_timestamp);
    entity.content_hash = Some(content_hash);
    entity.semantic_path = Some(semantic_path);
    Ok(entity)
}
```

---

## 16. Slim Graph Snapshot Types (v1.7.3)

For serialization to `.ptgraph` files, entities are stripped to 9 essential fields:

```rust
// crates/parseltongue-core/src/entities.rs:1809-1836

pub struct SlimEntityGraphSnapshot {
    pub isgl1_key: String,
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub entity_type: String,
    pub entity_class: String,
    pub language: String,
    pub root_subfolder_l1: String,
    pub root_subfolder_l2: String,
}
```

Edges reduce to 3 fields:

```rust
// crates/parseltongue-core/src/entities.rs:1844-1853

pub struct SlimEdgeGraphSnapshot {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: String,
}
```

Container wraps them with metadata:

```rust
// crates/parseltongue-core/src/entities.rs:1862-1883

pub struct PtGraphSnapshotContainer {
    pub version: String,
    pub generated_at: String,
    pub source_directory: String,
    pub entity_count: usize,
    pub edge_count: usize,
    pub entities: Vec<SlimEntityGraphSnapshot>,
    pub edges: Vec<SlimEdgeGraphSnapshot>,
}
```

---

## 17. Language Detection

File extension to language mapping, used by `get_language_type()`:

```rust
// crates/parseltongue-core/src/entities.rs:34-51

pub fn file_extensions(&self) -> Vec<&'static str> {
    match self {
        Language::Rust => vec!["rs"],
        Language::JavaScript => vec!["js", "jsx"],
        Language::TypeScript => vec!["ts", "tsx"],
        Language::Python => vec!["py"],
        Language::Java => vec!["java"],
        Language::C => vec!["c", "h"],
        Language::Cpp => vec!["cpp", "cc", "cxx", "hpp"],
        Language::Go => vec!["go"],
        Language::Ruby => vec!["rb"],
        Language::Php => vec!["php"],
        Language::CSharp => vec!["cs"],
        Language::Swift => vec!["swift"],
        Language::Kotlin => vec!["kt", "kts"],
        Language::Scala => vec!["scala", "sc"],
        Language::Sql => vec!["sql"],
    }
}
```

---

## 18. The Full Pipeline (End to End)

```
File on disk (e.g., src/auth.rs)
    |
    v
Language::from_file_path() -> Language::Rust
    |
    v
THREAD_PARSERS: get or create tree-sitter Parser for Rust
    |
    v
parser.parse(source) -> tree_sitter::Tree
    |
    v
THREAD_EXTRACTOR: QueryBasedExtractor.parse_source()
    -> Vec<ParsedEntity> + Vec<DependencyEdge>
    |
    v
[Rust only] enrich_rust_entities_with_attributes()
    -> adds metadata["is_test"] = "true" for #[test] fns
    |
    v
For each ParsedEntity:
    sanitize_entity_name_for_isgl1(entity.name)
    extract_semantic_path(entity.file_path)
    compute_birth_timestamp(entity.file_path, entity.name)
    -> format!("{}:{}:{}:{}:T{}", lang, type, name, path, ts)
    |
    v
ISGL1 v2 key: "rust:fn:handle_auth:__src_auth:T1706284800"
    |
    v
Stored in CozoDB as CodeEntity with:
    .isgl1_key = the generated key
    .birth_timestamp = Some(1706284800)
    .content_hash = Some(sha256 of code)
    .semantic_path = Some("__src_auth")
```

---

## 19. Version History

| Version | Change |
|---------|--------|
| v1 (original) | Line-number keys: `rust:fn:main:__src_main:10-50` |
| v2.0 | Birth timestamp keys: `rust:fn:main:__src_main:T1706284800` |
| v2.1 | Sanitized entity names for generic types (C#, C++, Java, TS) |
| v1.5.6 | Added SQL entity types: `Table`, `View` |
| Phase 5 | Thread-local parsers + extractors (zero mutex contention) |
| v0.8.9 | QueryBasedExtractor replaced manual tree-walking for 11/12 languages |
| v0.9.0 | Rust attribute enrichment (`#[test]` detection) |
| v1.7.3 | Slim snapshot types for `.ptgraph` serialization |
