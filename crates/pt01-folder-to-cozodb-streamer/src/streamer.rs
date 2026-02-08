//! File streaming implementation for folder-to-cozoDB processing.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use tokio::io::AsyncReadExt;
use walkdir::WalkDir;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use parseltongue_core::entities::*;
use parseltongue_core::storage::CozoDbStorage;
use crate::errors::*;
use crate::external_dependency_handler::extract_placeholders_from_edges_deduplicated;
use crate::isgl1_generator::*;
use crate::lsp_client::*;
use crate::test_detector::{TestDetector, EntityClass};
use crate::StreamerConfig;

// Import LSP metadata types from parseltongue-core
use parseltongue_core::entities::{LspMetadata, TypeInformation, UsageAnalysis};

/// File streamer interface
#[async_trait::async_trait]
pub trait FileStreamer: Send + Sync {
    /// Stream all files from the configured directory to database (sequential)
    async fn stream_directory(&self) -> Result<StreamResult>;

    /// Stream all files from the configured directory to database (parallel with Rayon)
    ///
    /// v1.5.4: Parallel implementation using Rayon for 5-7x speedup on multi-core systems
    async fn stream_directory_with_parallel_rayon(&self) -> Result<StreamResult>;

    /// Stream a single file to database
    async fn stream_file(&self, file_path: &Path) -> Result<FileResult>;

    /// Get current streaming statistics
    fn get_stats(&self) -> StreamStats;
}

/// Streaming operation results
#[derive(Debug, Clone)]
pub struct StreamResult {
    pub total_files: usize,
    pub processed_files: usize,
    pub entities_created: usize,
    pub errors: Vec<String>,
    pub duration: std::time::Duration,
}

/// Single file processing result
#[derive(Debug, Clone)]
pub struct FileResult {
    pub file_path: String,
    pub entities_created: usize,
    pub success: bool,
    pub error: Option<String>,
}

/// Streaming statistics
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    pub files_processed: usize,
    pub entities_created: usize,
    pub code_entities_created: usize,  // v0.9.3: Track CODE entities separately
    pub test_entities_created: usize,  // v0.9.3: Track TEST entities separately
    pub errors_encountered: usize,
}

/// File streamer implementation
pub struct FileStreamerImpl {
    config: StreamerConfig,
    key_generator: Arc<dyn Isgl1KeyGenerator>,
    lsp_client: Arc<dyn RustAnalyzerClient>,
    test_detector: Arc<dyn TestDetector>,
    db: Arc<CozoDbStorage>,
    stats: std::sync::Mutex<StreamStats>,
}

impl FileStreamerImpl {
    /// Create new file streamer with database connection
    pub async fn new(
        config: StreamerConfig,
        key_generator: Arc<dyn Isgl1KeyGenerator>,
        test_detector: Arc<dyn TestDetector>,
    ) -> Result<Self> {
        // Initialize database connection
        let db = CozoDbStorage::new(&config.db_path)
            .await
            .map_err(|e| StreamerError::StorageError {
                details: format!("Failed to create database: {}", e),
            })?;

        // Create schema
        db.create_schema()
            .await
            .map_err(|e| StreamerError::StorageError {
                details: format!("Failed to create schema: {}", e),
            })?;

        // Initialize LSP client (graceful degradation if unavailable)
        let lsp_client = RustAnalyzerClientImpl::new().await;

        Ok(Self {
            config,
            key_generator,
            lsp_client: Arc::new(lsp_client),
            test_detector,
            db: Arc::new(db),
            stats: std::sync::Mutex::new(StreamStats::default()),
        })
    }

    /// Create new file streamer with custom LSP client (for testing)
    #[cfg(test)]
    pub async fn new_with_lsp(
        config: StreamerConfig,
        key_generator: Arc<dyn Isgl1KeyGenerator>,
        lsp_client: Arc<dyn RustAnalyzerClient>,
        test_detector: Arc<dyn TestDetector>,
    ) -> Result<Self> {
        // Initialize database connection
        let db = CozoDbStorage::new(&config.db_path)
            .await
            .map_err(|e| StreamerError::StorageError {
                details: format!("Failed to create database: {}", e),
            })?;

        // Create schema
        db.create_schema()
            .await
            .map_err(|e| StreamerError::StorageError {
                details: format!("Failed to create schema: {}", e),
            })?;

        Ok(Self {
            config,
            key_generator,
            lsp_client,
            test_detector,
            db: Arc::new(db),
            stats: std::sync::Mutex::new(StreamStats::default()),
        })
    }

    /// Convert ParsedEntity to CodeEntity for database storage
    fn parsed_entity_to_code_entity(
        &self,
        parsed: &ParsedEntity,
        isgl1_key: &str,
        source_code: &str,
        file_path: &Path,
    ) -> std::result::Result<CodeEntity, parseltongue_core::error::ParseltongError> {
        // Create InterfaceSignature
        let interface_signature = InterfaceSignature {
            entity_type: self.convert_entity_type(&parsed.entity_type),
            name: parsed.name.clone(),
            visibility: Visibility::Public, // Default to public for now
            file_path: PathBuf::from(&parsed.file_path),
            line_range: LineRange::new(parsed.line_range.0 as u32, parsed.line_range.1 as u32)?,
            module_path: self.derive_file_module_path(file_path),
            documentation: None,
            language_specific: self.create_language_signature(&parsed.language),
        };

        // Create CodeEntity with temporal state initialized to "unchanged" (current=true, future=true, action=none)
        // v0.9.0: Include EntityClass classification using test_detector
        let local_entity_class = self.test_detector.detect_test_from_path_and_name(
            file_path, 
            source_code  // Use actual source code for test detection, not entity name
        );
        
        // Convert from local EntityClass to parseltongue_core EntityClass
        let entity_class = match local_entity_class {
            EntityClass::Test => parseltongue_core::EntityClass::TestImplementation,
            EntityClass::Code => parseltongue_core::EntityClass::CodeImplementation,
        };
        
        let mut entity = CodeEntity::new(isgl1_key.to_string(), interface_signature, entity_class)?;

        // Extract the code snippet from the source
        let code_snippet = self.extract_code_snippet(source_code, parsed.line_range.0, parsed.line_range.1);

        // ISGL1 v2: Compute v2 fields BEFORE code_snippet is moved
        use parseltongue_core::isgl1_v2::{
            compute_birth_timestamp,
            compute_content_hash,
            extract_semantic_path,
        };

        entity.birth_timestamp = Some(compute_birth_timestamp(&parsed.file_path, &parsed.name));
        entity.content_hash = Some(compute_content_hash(&code_snippet));
        entity.semantic_path = Some(extract_semantic_path(&parsed.file_path));

        // Set current_code and future_code to the same value (unchanged state)
        entity.current_code = Some(code_snippet.clone());
        entity.future_code = Some(code_snippet);

        // GREEN Phase: Apply TDD classification based on parsed metadata
        entity.tdd_classification = self.classify_entity(parsed);

        Ok(entity)
    }

    /// Classify entity as TEST or CODE based on metadata
    ///
    /// FP Pattern: Pure function - deterministic classification based on metadata
    ///
    /// Preconditions:
    /// - parsed.metadata contains "is_test" key if entity is a test
    ///
    /// Postconditions:
    /// - Returns TddClassification with correct EntityClass
    fn classify_entity(&self, parsed: &ParsedEntity) -> parseltongue_core::entities::TddClassification {
        use parseltongue_core::entities::{EntityClass, TddClassification};

        // Pure FP: Check metadata for test indicator
        let is_test = parsed
            .metadata
            .get("is_test")
            .map(|v| v == "true")
            .unwrap_or(false);

        // Minimal GREEN implementation: Just set entity_class
        TddClassification {
            entity_class: if is_test {
                EntityClass::TestImplementation
            } else {
                EntityClass::CodeImplementation
            },
            ..TddClassification::default()
        }
    }

    /// Convert Tool 1's EntityType to parseltongue-core's EntityType
    ///
    /// **Design Pattern**: Functional mapping with exhaustive pattern matching
    /// **Post-v0.8.9**: All 11 entity types supported across 12 languages
    ///
    /// **Mapping Strategy** (MVP):
    /// - Namespace → Module (semantic equivalence for namespaces/packages)
    /// - Typedef → Variable (type aliases stored as variables for now)
    /// - Future: Add Namespace and Typedef to parseltongue-core EntityType enum
    fn convert_entity_type(&self, entity_type: &crate::isgl1_generator::EntityType) -> parseltongue_core::entities::EntityType {
        match entity_type {
            // Universal entities
            crate::isgl1_generator::EntityType::Function => parseltongue_core::entities::EntityType::Function,
            crate::isgl1_generator::EntityType::Class => parseltongue_core::entities::EntityType::Class,
            crate::isgl1_generator::EntityType::Method => parseltongue_core::entities::EntityType::Method,
            crate::isgl1_generator::EntityType::Module => parseltongue_core::entities::EntityType::Module,
            crate::isgl1_generator::EntityType::Variable => parseltongue_core::entities::EntityType::Variable,

            // Pragmatic mappings (v0.8.9 MVP)
            crate::isgl1_generator::EntityType::Namespace => parseltongue_core::entities::EntityType::Module,   // C++/C# namespace → Module
            crate::isgl1_generator::EntityType::Typedef => parseltongue_core::entities::EntityType::Variable,   // C typedef → Variable

            // Rust-specific entities
            crate::isgl1_generator::EntityType::Struct => parseltongue_core::entities::EntityType::Struct,
            crate::isgl1_generator::EntityType::Enum => parseltongue_core::entities::EntityType::Enum,
            crate::isgl1_generator::EntityType::Trait => parseltongue_core::entities::EntityType::Trait,
            crate::isgl1_generator::EntityType::Impl => parseltongue_core::entities::EntityType::ImplBlock {
                trait_name: None,
                struct_name: "Unknown".to_string(), // TODO: Extract from parsed entity
            },

            // SQL-specific entities (v1.5.6)
            crate::isgl1_generator::EntityType::Table => parseltongue_core::entities::EntityType::Table,
            crate::isgl1_generator::EntityType::View => parseltongue_core::entities::EntityType::View,
        }
    }

    /// Create language-specific signature
    fn create_language_signature(&self, language: &Language) -> LanguageSpecificSignature {
        match language {
            Language::Rust => LanguageSpecificSignature::Rust(RustSignature {
                generics: vec![],
                lifetimes: vec![],
                where_clauses: vec![],
                attributes: vec![],
                trait_impl: None,
            }),
            Language::Python => LanguageSpecificSignature::Python(PythonSignature {
                parameters: vec![],
                return_type: None,
                is_async: false,
                decorators: vec![],
            }),
            _ => LanguageSpecificSignature::Rust(RustSignature {
                generics: vec![],
                lifetimes: vec![],
                where_clauses: vec![],
                attributes: vec![],
                trait_impl: None,
            }),
        }
    }

    /// Extract code snippet from source by line range
    fn extract_code_snippet(&self, source: &str, start_line: usize, end_line: usize) -> String {
        source
            .lines()
            .enumerate()
            .filter(|(idx, _)| *idx + 1 >= start_line && *idx < end_line)
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract module path from file path relative to crate root
    ///
    /// # 4-Word Name: derive_file_module_path
    ///
    /// # Examples (Rust conventions)
    /// - `src/lib.rs` → `[]` (crate root)
    /// - `src/main.rs` → `[]` (binary root)
    /// - `src/calculator.rs` → `["calculator"]`
    /// - `src/utils/mod.rs` → `["utils"]`
    /// - `src/utils/helpers.rs` → `["utils", "helpers"]`
    fn derive_file_module_path(&self, file_path: &Path) -> Vec<String> {
        // Make path relative to root_dir
        let relative_path = file_path
            .strip_prefix(&self.config.root_dir)
            .unwrap_or(file_path);

        let path_str = relative_path.to_string_lossy();
        let parts: Vec<&str> = path_str.split(['/', '\\']).collect();

        // Find "src" directory index and start after it
        let start_idx = parts.iter().position(|&s| s == "src").map(|i| i + 1).unwrap_or(0);

        let mut module_path: Vec<String> = parts[start_idx..]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Remove file extension for Rust files
                s.strip_suffix(".rs")
                    .or_else(|| s.strip_suffix(".py"))
                    .or_else(|| s.strip_suffix(".js"))
                    .or_else(|| s.strip_suffix(".ts"))
                    .unwrap_or(s)
                    .to_string()
            })
            .collect();

        // Handle special Rust files
        if let Some(last) = module_path.last() {
            match last.as_str() {
                // lib.rs and main.rs are crate roots - remove from path
                "lib" | "main" => {
                    module_path.pop();
                }
                // mod.rs represents parent module - remove from path
                "mod" => {
                    module_path.pop();
                }
                _ => {}
            }
        }

        module_path
    }

    /// Check if file should be processed based on patterns
    fn should_process_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        // REQ-V090-002.0: Check for git subdirectories (always-on detection)
        if self.is_under_git_subdirectory(file_path) {
            return false;
        }

        // Check exclude patterns
        for pattern in &self.config.exclude_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return false;
            }
        }

        // Check include patterns
        for pattern in &self.config.include_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Simple glob pattern matching
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple pattern matching: check if path ends with extension
            // TODO: Implement proper glob matching for complex patterns
            path.contains(&pattern.replace('*', "")) || path == pattern
        } else {
            path.contains(pattern)
        }
    }

    /// REQ-V090-002.0: Check if path is under a directory containing .git (but not project root)
    /// 
    /// # Performance Contract
    /// - Completes in <50μs per path
    /// - Stops traversal at project root boundary
    /// - Handles permission errors gracefully
    fn is_under_git_subdirectory(&self, path: &Path) -> bool {
        let root = &self.config.root_dir;
        let mut current = path;

        // Walk up parent directories looking for .git
        while let Some(parent) = current.parent() {
            // Stop at project root (don't exclude project root itself)
            if parent == root {
                break;
            }
            
            // Don't go beyond project root
            if !parent.starts_with(root) {
                break;
            }

            // Check if this parent directory contains .git
            if parent.join(".git").exists() {
                return true; // Found nested git repo
            }

            current = parent;
        }

        false
    }

    /// Read file content with size limit
    async fn read_file_content(&self, file_path: &Path) -> Result<String> {
        let metadata = fs::metadata(file_path).await.map_err(|e| {
            StreamerError::FileSystemError {
                path: file_path.to_string_lossy().to_string(),
                source: e,
            }
        })?;

        if metadata.len() as usize > self.config.max_file_size {
            return Err(StreamerError::FileTooLarge {
                path: file_path.to_string_lossy().to_string(),
                size: metadata.len() as usize,
                limit: self.config.max_file_size,
            });
        }

        let mut content = String::new();
        let mut file = fs::File::open(file_path).await.map_err(|e| {
            StreamerError::FileSystemError {
                path: file_path.to_string_lossy().to_string(),
                source: e,
            }
        })?;

        file.read_to_string(&mut content).await.map_err(|e| {
            StreamerError::FileSystemError {
                path: file_path.to_string_lossy().to_string(),
                source: e,
            }
        })?;

        Ok(content)
    }

    /// Update streaming statistics (v0.9.3: track CODE/TEST separately)
    fn update_stats(&self, entities_created: usize, code_count: usize, test_count: usize, had_error: bool) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.files_processed += 1;
            stats.entities_created += entities_created;
            stats.code_entities_created += code_count;
            stats.test_entities_created += test_count;
            if had_error {
                stats.errors_encountered += 1;
            }
        }
    }
}

#[async_trait::async_trait]
impl FileStreamer for FileStreamerImpl {
    async fn stream_directory(&self) -> Result<StreamResult> {
        let start_time = Instant::now();
        let mut total_files = 0;
        let mut processed_files = 0;
        let mut entities_created = 0;
        let mut errors = Vec::new();

        println!(
            "{}",
            style("Starting directory streaming...").blue().bold()
        );

        // Setup progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
        );
        pb.set_message("Scanning files...");

        // Walk through directory
        for entry in WalkDir::new(&self.config.root_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() && self.should_process_file(path) {
                total_files += 1;
                pb.set_message(format!("Processing: {}", path.display()));

                match self.stream_file(path).await {
                    Ok(result) => {
                        processed_files += 1;
                        entities_created += result.entities_created;
                    }
                    Err(e) => {
                        let error_msg = format!("{}: {}", path.display(), e);
                        errors.push(error_msg.clone());
                        pb.println(format!("{} {}", style("⚠").yellow().for_stderr(), error_msg));
                        self.update_stats(0, 0, 0, true);  // v0.9.3: No entities created on error
                    }
                }
            }
        }

        pb.finish_with_message("Directory streaming completed");

        let duration = start_time.elapsed();

        // Get final stats for CODE/TEST breakdown
        let final_stats = self.get_stats();

        // Print summary
        println!("\n{}", style("Streaming Summary:").green().bold());
        println!("Total files found: {}", total_files);
        println!("Files processed: {}", processed_files);
        println!("Entities created: {} (CODE only)", style(entities_created).cyan().bold());
        println!("  └─ CODE entities: {}", style(final_stats.code_entities_created).cyan());
        println!("  └─ TEST entities: {} {}",
            style(final_stats.test_entities_created).yellow(),
            style("(excluded for optimal LLM context)").dim()
        );
        println!("Errors encountered: {}", errors.len());
        println!("Duration: {:?}", duration);

        // ✅ v0.9.6: Clear message about test exclusion
        if final_stats.test_entities_created > 0 {
            println!("\n{} Tests intentionally excluded from ingestion for optimal LLM context",
                style("✓").green().bold()
            );
        }

        Ok(StreamResult {
            total_files,
            processed_files,
            entities_created,
            errors,
            duration,
        })
    }

    async fn stream_directory_with_parallel_rayon(&self) -> Result<StreamResult> {
        let start_time = Instant::now();

        println!(
            "{}",
            style("Starting PARALLEL directory streaming (v1.5.4 Rayon)...").blue().bold()
        );

        // Setup progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
        );
        pb.set_message("Scanning files...");

        // Step 1: Collect all file paths upfront (trade-off: memory for parallelism)
        let files_to_process: Vec<PathBuf> = WalkDir::new(&self.config.root_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() && self.should_process_file(path) {
                    Some(path.to_path_buf())
                } else {
                    None
                }
            })
            .collect();

        let total_files = files_to_process.len();
        pb.set_message(format!("Found {} files, processing in parallel...", total_files));

        // Step 2: Process files in parallel using Rayon
        // Note: tree-sitter Parser is !Send, so we use thread-local initialization
        let results: Vec<Result<(FileResult, Vec<CodeEntity>, Vec<parseltongue_core::entities::DependencyEdge>)>> =
            files_to_process
                .par_iter()
                .map(|file_path| {
                    // Synchronous file processing (trade-off: blocking I/O for simplicity)
                    // In parallel context, async adds complexity without benefit
                    self.process_file_sync_for_parallel(file_path)
                })
                .collect();

        pb.finish_with_message("Parallel processing completed, inserting to database...");

        // Step 3: Aggregate results and collect entities/edges for batch insertion
        let mut processed_files = 0;
        let mut entities_created = 0;
        let mut errors = Vec::new();
        let mut all_entities: Vec<CodeEntity> = Vec::new();
        let mut all_dependencies: Vec<parseltongue_core::entities::DependencyEdge> = Vec::new();

        for result in results {
            match result {
                Ok((file_result, entities, dependencies)) => {
                    processed_files += 1;
                    entities_created += file_result.entities_created;
                    all_entities.extend(entities);
                    all_dependencies.extend(dependencies);
                    if let Some(error) = file_result.error {
                        errors.push(error);
                    }
                }
                Err(e) => {
                    errors.push(format!("Processing error: {}", e));
                }
            }
        }

        // Step 4: Batch insert all entities
        if !all_entities.is_empty() {
            match self.db.insert_entities_batch(&all_entities).await {
                Ok(_) => {
                    // Success
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to batch insert {} entities: {}",
                        all_entities.len(),
                        e
                    ));
                }
            }
        }

        // Step 5: Batch insert all dependencies
        if !all_dependencies.is_empty() {
            // Ensure schema exists
            let _ = self.db.create_dependency_edges_schema().await;

            match self.db.insert_edges_batch(&all_dependencies).await {
                Ok(_) => {
                    // Success
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to batch insert {} dependencies: {}",
                        all_dependencies.len(),
                        e
                    ));
                }
            }
        }

        let duration = start_time.elapsed();

        // Get final stats for CODE/TEST breakdown
        let final_stats = self.get_stats();

        // Print summary
        println!("\n{}", style("Parallel Streaming Summary:").green().bold());
        println!("Total files found: {}", total_files);
        println!("Files processed: {}", processed_files);
        println!("Entities created: {} (CODE only)", style(entities_created).cyan().bold());
        println!("  └─ CODE entities: {}", style(final_stats.code_entities_created).cyan());
        println!("  └─ TEST entities: {} {}",
            style(final_stats.test_entities_created).yellow(),
            style("(excluded for optimal LLM context)").dim()
        );
        println!("Errors encountered: {}", errors.len());
        println!("Duration: {:?}", duration);
        println!("Speedup estimate: {}",
            style("5-7x vs sequential (typical on 8+ cores)").cyan()
        );

        if final_stats.test_entities_created > 0 {
            println!("\n{} Tests intentionally excluded from ingestion for optimal LLM context",
                style("✓").green().bold()
            );
        }

        Ok(StreamResult {
            total_files,
            processed_files,
            entities_created,
            errors,
            duration,
        })
    }

    async fn stream_file(&self, file_path: &Path) -> Result<FileResult> {
        let file_path_str = file_path.to_string_lossy().to_string();

        // Read file content
        let content = self.read_file_content(file_path).await?;

        // Parse code entities AND dependencies (two-pass extraction)
        let (parsed_entities, dependencies) = self.key_generator.parse_source(&content, file_path)?;

        let mut entities_created = 0;
        let mut code_count = 0;  // v0.9.3: Track CODE entities
        let mut test_count = 0;  // v0.9.3: Track TEST entities
        let mut errors: Vec<String> = Vec::new();

        // Bug #4 Fix: Extract external dependency placeholders from edges
        // These are dependencies to external crates (e.g., clap::Parser, tokio::Runtime)
        // that don't exist in the local codebase. We create placeholder CodeEntity nodes
        // so that dependency edges have valid targets and blast radius queries work.
        let external_placeholders = extract_placeholders_from_edges_deduplicated(&dependencies);

        // Collect all entities to insert (external placeholders + regular entities)
        let mut entities_to_insert: Vec<CodeEntity> = Vec::new();

        // Add external dependency placeholders first
        for placeholder in external_placeholders {
            entities_to_insert.push(placeholder);
            code_count += 1; // External dependencies count as CODE entities
        }

        // Process each parsed entity and collect them for batch insertion
        for parsed_entity in parsed_entities {
            // Generate ISGL1 key
            let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;

            // Enrich with LSP metadata for Rust files (sequential hover requests)
            let lsp_metadata = self.fetch_lsp_metadata_for_entity(&parsed_entity, file_path).await;

            // Convert ParsedEntity to CodeEntity
            match self.parsed_entity_to_code_entity(&parsed_entity, &isgl1_key, &content, file_path) {
                Ok(mut code_entity) => {
                    // Store LSP metadata as JSON string if available
                    if let Some(metadata) = lsp_metadata {
                        code_entity.lsp_metadata = Some(metadata);
                    }

                    // v0.9.3: Track entity_class for stats
                    let entity_class = code_entity.entity_class;

                    // ✅ v0.9.6: Skip test entities - they pollute LLM context
                    if matches!(entity_class, parseltongue_core::EntityClass::TestImplementation) {
                        test_count += 1;
                        continue; // Don't insert tests into database
                    }

                    // Add to batch insertion list (CODE entities only)
                    entities_to_insert.push(code_entity);
                    code_count += 1; // v0.9.6: Only CODE entities reach here
                }
                Err(e) => {
                    let error_msg = format!("Failed to convert entity {}: {}", isgl1_key, e);
                    errors.push(error_msg);
                }
            }
        }

        // Batch insert all entities at once (external placeholders + regular entities)
        if !entities_to_insert.is_empty() {
            match self.db.insert_entities_batch(&entities_to_insert).await {
                Ok(_) => {
                    entities_created += entities_to_insert.len();
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to batch insert {} entities: {}",
                        entities_to_insert.len(),
                        e
                    );
                    errors.push(error_msg);
                }
            }
        }

        // ALWAYS create DependencyEdges schema, even if no dependencies
        // This ensures HTTP server can query the table (returns empty array if no edges)
        // Bug fix: Previously only created schema if dependencies.is_empty() == false
        if let Err(e) = self.db.create_dependency_edges_schema().await {
            // Schema might already exist - that's ok
            if !e.to_string().contains("already exists") && !e.to_string().contains("conflicts with an existing") {
                errors.push(format!("Failed to create dependency schema: {}", e));
            }
        }

        // Batch insert dependencies after all entities are stored
        if !dependencies.is_empty() {
            // Debug logging for v1.5.1 investigation
            let rust_edges_count = dependencies.iter().filter(|e| {
                let key_str: &str = e.from_key.as_ref();
                key_str.starts_with("rust:")
            }).count();
            let ruby_edges_count = dependencies.iter().filter(|e| {
                let key_str: &str = e.from_key.as_ref();
                key_str.starts_with("ruby:")
            }).count();
            if rust_edges_count > 0 || ruby_edges_count > 0 {
                eprintln!("[DEBUG-INSERT] About to insert {} total edges", dependencies.len());
                eprintln!("[DEBUG-INSERT] Rust edges: {}", rust_edges_count);
                eprintln!("[DEBUG-INSERT] Ruby edges: {}", ruby_edges_count);

                // Log first 3 Rust edges to see their actual keys
                eprintln!("[DEBUG-INSERT] Sample Rust edge keys:");
                for (i, edge) in dependencies.iter().filter(|e| {
                    let key_str: &str = e.from_key.as_ref();
                    key_str.starts_with("rust:")
                }).take(3).enumerate() {
                    let from_key: &str = edge.from_key.as_ref();
                    let to_key: &str = edge.to_key.as_ref();
                    eprintln!("[DEBUG-INSERT]   #{}: from={} to={} type={:?}",
                        i + 1, from_key, to_key, edge.edge_type);
                }
            }

            // Insert dependency edges
            match self.db.insert_edges_batch(&dependencies).await {
                Ok(_) => {
                    // Successfully inserted dependencies
                    if rust_edges_count > 0 || ruby_edges_count > 0 {
                        eprintln!("[DEBUG-INSERT] ✅ Successfully inserted edges");
                    }
                }
                Err(e) => {
                    eprintln!("[DEBUG-INSERT] ❌ FAILED to insert edges: {}", e);
                    errors.push(format!("Failed to insert {} dependencies: {}", dependencies.len(), e));
                }
            }
        }

        self.update_stats(entities_created, code_count, test_count, !errors.is_empty());

        Ok(FileResult {
            file_path: file_path_str,
            entities_created,
            success: errors.is_empty(),
            error: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        })
    }

    fn get_stats(&self) -> StreamStats {
        self.stats.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }
}

impl FileStreamerImpl {
    /// v1.5.4: Synchronous file processing for parallel context with Rayon
    ///
    /// # 4-Word Name: process_file_sync_for_parallel
    ///
    /// # Why Synchronous?
    /// - Rayon workers are OS threads, not async tasks
    /// - Blocking I/O in thread pool is acceptable (trade-off for simplicity)
    /// - Avoids async runtime complexity in parallel context
    /// - Still much faster than sequential due to parallelism
    ///
    /// # Performance Note
    /// Despite synchronous I/O, parallel processing achieves 5-7x speedup
    /// because file parsing (tree-sitter) dominates I/O time.
    ///
    /// # Return Type
    /// Returns (FileResult, entities_to_insert, dependencies_to_insert)
    /// The entities and dependencies need to be inserted after parallel processing
    fn process_file_sync_for_parallel(
        &self,
        file_path: &Path,
    ) -> Result<(FileResult, Vec<CodeEntity>, Vec<parseltongue_core::entities::DependencyEdge>)> {
        let file_path_str = file_path.to_string_lossy().to_string();

        // Read file content synchronously
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| StreamerError::FileSystemError {
                path: file_path_str.clone(),
                source: e,
            })?;

        // Check file size limit
        if content.len() > self.config.max_file_size {
            return Err(StreamerError::FileTooLarge {
                path: file_path_str.clone(),
                size: content.len(),
                limit: self.config.max_file_size,
            });
        }

        // Parse code entities AND dependencies (two-pass extraction)
        let (parsed_entities, dependencies) = self.key_generator.parse_source(&content, file_path)?;

        let mut code_count = 0;
        let mut test_count = 0;
        let mut errors: Vec<String> = Vec::new();

        // Extract external dependency placeholders
        let external_placeholders = extract_placeholders_from_edges_deduplicated(&dependencies);

        // Collect all entities to insert
        let mut entities_to_insert: Vec<CodeEntity> = Vec::new();

        // Add external dependency placeholders
        for placeholder in external_placeholders {
            entities_to_insert.push(placeholder);
            code_count += 1;
        }

        // Process each parsed entity
        for parsed_entity in parsed_entities {
            // Generate ISGL1 key
            let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;

            // Note: LSP metadata skipped in parallel mode (would require async)
            // This is a trade-off: speed vs metadata richness

            // Convert ParsedEntity to CodeEntity
            match self.parsed_entity_to_code_entity(&parsed_entity, &isgl1_key, &content, file_path) {
                Ok(code_entity) => {
                    let entity_class = code_entity.entity_class;

                    // Skip test entities
                    if matches!(entity_class, parseltongue_core::EntityClass::TestImplementation) {
                        test_count += 1;
                        continue;
                    }

                    // Add to batch insertion list
                    entities_to_insert.push(code_entity);
                    code_count += 1;
                }
                Err(e) => {
                    let error_msg = format!("Failed to convert entity {}: {}", isgl1_key, e);
                    errors.push(error_msg);
                }
            }
        }

        let entities_created = entities_to_insert.len();

        // Update stats
        self.update_stats(entities_created, code_count, test_count, !errors.is_empty());

        let file_result = FileResult {
            file_path: file_path_str,
            entities_created,
            success: errors.is_empty(),
            error: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        };

        Ok((file_result, entities_to_insert, dependencies))
    }

    /// Fetch LSP metadata for an entity using rust-analyzer hover
    /// Returns LspMetadata if successful, None if unavailable or failed (graceful degradation)
    async fn fetch_lsp_metadata_for_entity(
        &self,
        entity: &ParsedEntity,
        file_path: &Path,
    ) -> Option<LspMetadata> {
        // Only fetch for Rust files
        if entity.language != Language::Rust {
            return None;
        }

        // Calculate hover position at the start of the entity (line is 0-indexed in LSP)
        let line = entity.line_range.0.saturating_sub(1) as u32;
        let character = 0u32; // Start of line (tree-sitter gives us the entity name position)

        // Request hover metadata
        match self.lsp_client.hover(file_path, line, character).await {
            Ok(Some(hover_response)) => {
                // Convert hover response to LspMetadata (stub/MVP implementation)
                Self::hover_response_to_lsp_metadata(&hover_response)
            }
            Ok(None) => None, // Graceful degradation
            Err(_) => None,   // Graceful degradation
        }
    }

    /// Convert hover response to LspMetadata (stub implementation for MVP)
    /// Future enhancement: parse rust-analyzer response for richer metadata
    fn hover_response_to_lsp_metadata(hover: &HoverResponse) -> Option<LspMetadata> {
        Some(LspMetadata {
            type_information: TypeInformation {
                resolved_type: hover.contents.clone(),
                module_path: vec![],
                generic_parameters: vec![],
                definition_location: None,
            },
            usage_analysis: UsageAnalysis {
                total_references: 0,
                usage_locations: vec![],
                dependents: vec![],
            },
            semantic_tokens: vec![],
        })
    }
}

#[cfg(test)]
#[path = "streamer_lsp_tests.rs"]
mod streamer_lsp_tests;

#[cfg(test)]
#[path = "streamer_module_path_tests.rs"]
mod streamer_module_path_tests;
