//! Core interfaces for Parseltongue.
//!
//! Following steering docs principle: "Dependency Injection for Testability"
//! with trait-based design that enables comprehensive testing.

use crate::entities::*;
use crate::error::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

/// Core trait for all Parseltongue tools
///
/// Every tool in the 6-tool pipeline implements this interface,
/// enabling uniform execution and testing patterns.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool with given input
    async fn execute(&self, input: ToolInput) -> Result<ToolOutput>;

    /// Validate input before execution
    fn validate_input(&self, input: &ToolInput) -> Result<()>;

    /// Get tool metadata
    fn metadata(&self) -> ToolMetadata;
}

/// Tool input types
#[derive(Debug, Clone)]
pub enum ToolInput {
    /// Index a folder (Tool 1)
    IndexFolder {
        path: PathBuf,
        language_filter: Option<Language>,
    },

    /// Apply temporal changes (Tool 2)
    ApplyTemporalChanges {
        changes: Vec<TemporalChange>,
    },

    /// Generate context (Tool 3)
    GenerateContext {
        query: ContextQuery,
    },

    /// Validate code (Tool 4)
    ValidateCode {
        validation_level: ValidationLevel,
    },

    /// Write code changes (Tool 5)
    WriteChanges {
        changes: Vec<CodeChange>,
        validate_before_write: bool,
    },

    /// Reset temporal state (Tool 6)
    ResetTemporalState {
        reindex_path: Option<PathBuf>,
    },
}

/// Tool output types
#[derive(Debug, Clone)]
pub enum ToolOutput {
    /// Indexing completed
    IndexingComplete {
        entities_count: usize,
        duration_ms: u64,
    },

    /// Temporal changes applied
    TemporalChangesApplied {
        changes_count: usize,
        affected_entities: Vec<String>,
    },

    /// Context generated
    ContextGenerated {
        context: CodeGraphContext,
        token_count: usize,
    },

    /// Validation results
    ValidationResults {
        level: ValidationLevel,
        results: Vec<ValidationResult>,
        success: bool,
    },

    /// Code write results
    WriteResults {
        files_written: Vec<PathBuf>,
        files_modified: Vec<PathBuf>,
        files_deleted: Vec<PathBuf>,
    },

    /// Reset completed
    ResetComplete {
        entities_reset: usize,
        reindexed: bool,
    },
}

/// Tool metadata
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    /// Tool identifier
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Tool description
    pub description: String,
    /// Supported input types
    pub supported_inputs: Vec<String>,
    /// Tool capabilities
    pub capabilities: ToolCapabilities,
}

/// Tool capabilities
#[derive(Debug, Clone)]
pub struct ToolCapabilities {
    /// Supports async execution
    pub async_execution: bool,
    /// Supports parallel processing
    pub parallel_processing: bool,
    /// Supports incremental processing
    pub incremental_processing: bool,
    /// Requires network access
    pub requires_network: bool,
    /// Maximum supported input size
    pub max_input_size: Option<usize>,
}

/// Repository interface for data access operations
///
/// Following dependency injection principle for testability
/// and enabling different storage backends.
#[async_trait]
pub trait CodeGraphRepository: Send + Sync {
    /// Store a code entity
    async fn store_entity(&mut self, entity: CodeEntity) -> Result<()>;

    /// Retrieve an entity by ISGL1 key
    async fn get_entity(&self, isgl1_key: &str) -> Result<Option<CodeEntity>>;

    /// Update an entity
    async fn update_entity(&mut self, entity: CodeEntity) -> Result<()>;

    /// Delete an entity
    async fn delete_entity(&mut self, isgl1_key: &str) -> Result<()>;

    /// Query entities with temporal filters
    async fn query_entities(&self, query: &TemporalQuery) -> Result<Vec<CodeEntity>>;

    /// Get all entities that will change
    async fn get_changed_entities(&self) -> Result<Vec<CodeEntity>>;

    /// Reset temporal state (Tool 6 operation)
    async fn reset_temporal_state(&mut self) -> Result<()>;
}

/// Temporal query for entity retrieval
#[derive(Debug, Clone)]
pub struct TemporalQuery {
    /// Base entities to start from
    pub base_entities: Vec<String>,
    /// Hop depth for dependency analysis
    pub hop_depth: u32,
    /// Include future changes only
    pub future_only: bool,
    /// Entity type filter
    pub entity_type_filter: Option<EntityType>,
    /// Language filter
    pub language_filter: Option<Language>,
}

/// Language parser interface
///
/// Enables different parsing strategies and testing with mock parsers
#[async_trait]
pub trait LanguageParser: Send + Sync {
    /// Parse a file and extract entities
    async fn parse_file(&self, file_path: &PathBuf) -> Result<Vec<InterfaceChunk>>;

    /// Extract interfaces from source code
    async fn extract_interfaces(&self, code: &str, language: Language) -> Result<Vec<InterfaceChunk>>;

    /// Detect language from content
    fn detect_language(&self, content: &str) -> Option<Language>;

    /// Get supported languages
    fn supported_languages(&self) -> Vec<Language>;
}

/// Interface chunk from parsing
#[derive(Debug, Clone)]
pub struct InterfaceChunk {
    /// ISGL1 key
    pub isgl1_key: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Entity name
    pub name: String,
    /// Interface signature
    pub signature: InterfaceSignature,
    /// Source code
    pub source_code: String,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// LSP client interface for enhanced validation
///
/// Enables testing with mock LSP implementations
#[async_trait]
pub trait LspClient: Send + Sync {
    /// Start the LSP server
    async fn start_server(&mut self) -> Result<()>;

    /// Get semantic tokens for a file
    async fn get_semantic_tokens(&self, file_path: &str) -> Result<Vec<SemanticToken>>;

    /// Get type information for a position
    async fn get_type_info(&self, position: &Position) -> Result<TypeInformation>;

    /// Get usage analysis for an entity
    async fn get_usage_analysis(&self, isgl1_key: &str) -> Result<UsageAnalysis>;

    /// Get implementation locations
    async fn get_implementations(&self, position: &Position) -> Result<Vec<Location>>;

    /// Shutdown the LSP server
    async fn shutdown_server(&mut self) -> Result<()>;

    /// Health check
    async fn health_check(&self) -> Result<HealthStatus>;
}

/// Position in code
#[derive(Debug, Clone)]
pub struct Position {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
}

/// Health status
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy { reason: String },
    Unknown,
}

/// LLM client interface for reasoning operations
///
/// Enables testing with mock LLM responses
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send request to LLM
    async fn send_request(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Validate response format
    fn validate_response(&self, response: &LlmResponse, request: &LlmRequest) -> Result<()>;

    /// Get rate limit status
    async fn get_rate_limit_status(&self) -> Result<RateLimitStatus>;

    /// Estimate token count
    fn estimate_tokens(&self, content: &str) -> usize;
}

/// LLM request
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub request_id: Uuid,
    pub context: CodeGraphContext,
    pub task: TaskSpecification,
    pub constraints: RequestConstraints,
}

/// LLM response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub request_id: Uuid,
    pub reasoning: String,
    pub proposed_changes: Vec<ProposedChange>,
    pub confidence_score: f64,
    pub validation_status: ValidationStatus,
}

/// Task specification for LLM
#[derive(Debug, Clone)]
pub struct TaskSpecification {
    pub task_type: TaskType,
    pub instruction: String,
    pub success_criteria: SuccessCriteria,
}

/// Task types
#[derive(Debug, Clone)]
pub enum TaskType {
    ChangeReasoning,
    ValidationCheck,
    ContextGeneration,
    DependencyAnalysis,
}

/// Success criteria
#[derive(Debug, Clone)]
pub struct SuccessCriteria {
    pub min_confidence: f64,
    pub max_duration: std::time::Duration,
    pub validation_rules: Vec<ValidationRule>,
}

/// Validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field: String,
    pub constraint: String,
}

/// Request constraints
#[derive(Debug, Clone)]
pub struct RequestConstraints {
    pub max_tokens: usize,
    pub temperature: f64,
    pub min_confidence: f64,
}

/// Proposed change from LLM
#[derive(Debug, Clone)]
pub struct ProposedChange {
    pub target_entity: String,
    pub change_type: TemporalAction,
    pub new_content: String,
    pub justification: String,
    pub affected_dependencies: Vec<String>,
}

/// Validation status
#[derive(Debug, Clone)]
pub enum ValidationStatus {
    Valid,
    Invalid { errors: Vec<ValidationError> },
    Unknown,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

/// Rate limit status
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub requests_remaining: u32,
    pub reset_time: std::time::SystemTime,
    pub limit: u32,
}

/// Context query interface
///
/// Enables different context generation strategies
#[async_trait]
pub trait ContextGenerator: Send + Sync {
    /// Generate context from entities
    async fn generate_context(&self, entities: Vec<CodeEntity>, query: &ContextQuery) -> Result<CodeGraphContext>;

    /// Optimize context for token limits
    fn optimize_context(&self, context: &mut CodeGraphContext, token_limit: usize) -> Result<()>;

    /// Calculate token count
    fn estimate_tokens(&self, context: &CodeGraphContext) -> usize;
}

/// Context query parameters
#[derive(Debug, Clone)]
pub struct ContextQuery {
    pub base_entities: Vec<String>,
    pub hop_depth: u32,
    pub change_type: ChangeType,
    pub size_limit: usize,
    pub optimization_strategy: OptimizationStrategy,
}

/// Change type
#[derive(Debug, Clone)]
pub enum ChangeType {
    Edit,
    Create,
    Delete,
    Refactor,
}

/// Optimization strategy
#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    BlastRadius,
    RelevanceBased,
    SizeBased,
    PrioritizeTests,
}

/// CodeGraph context for LLM
#[derive(Debug, Clone)]
pub struct CodeGraphContext {
    pub version: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub token_count: usize,
    pub entities: Vec<ContextEntity>,
    pub relationships: Vec<ContextRelationship>,
    pub optimization_info: OptimizationInfo,
}

/// Context entity
#[derive(Debug, Clone)]
pub struct ContextEntity {
    pub isgl1_key: String,
    pub interface_signature: InterfaceSignature,
    pub tdd_classification: TddClassification,
    pub lsp_metadata: Option<LspMetadata>,
    pub relevance_score: f64,
    pub dependency_level: u32,
}

/// Context relationship
#[derive(Debug, Clone)]
pub struct ContextRelationship {
    pub dependent: String,
    pub dependency: String,
    pub relationship_type: String,
    pub strength: f64,
}

/// Optimization information
#[derive(Debug, Clone)]
pub struct OptimizationInfo {
    pub excluded_entities: Vec<String>,
    pub truncation_applied: bool,
    pub prioritization_strategy: String,
}

/// Temporal change
#[derive(Debug, Clone)]
pub struct TemporalChange {
    pub isgl1_key: String,
    pub action: TemporalAction,
    pub future_code: Option<String>,
    pub updated_signature: Option<InterfaceSignature>,
}

/// Validation level
#[derive(Debug, Clone)]
pub enum ValidationLevel {
    Syntax,
    Build,
    Test,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub entity: String,
    pub level: ValidationLevel,
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
}

/// Code change
#[derive(Debug, Clone)]
pub struct CodeChange {
    pub file_path: PathBuf,
    pub entity_name: String,
    pub change_type: TemporalAction,
    pub old_content: Option<String>,
    pub new_content: String,
    pub line_range: LineRange,
}

/// Performance monitoring interface
///
/// Following steering docs principle: "Performance Claims Must Be Test-Validated"
#[async_trait]
pub trait PerformanceMonitor: Send + Sync {
    /// Record operation start
    async fn start_operation(&self, operation_id: &str, operation_type: &str);

    /// Record operation completion
    async fn complete_operation(
        &self,
        operation_id: &str,
        duration: std::time::Duration,
        success: bool,
    );

    /// Get performance metrics
    async fn get_metrics(&self, operation_type: &str) -> Result<PerformanceMetrics>;

    /// Check performance contracts
    async fn check_contracts(&self) -> Result<Vec<PerformanceViolation>>;
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_count: u64,
    pub total_duration: std::time::Duration,
    pub average_duration: std::time::Duration,
    pub success_rate: f64,
    pub min_duration: std::time::Duration,
    pub max_duration: std::time::Duration,
}

/// Performance violation
#[derive(Debug, Clone)]
pub struct PerformanceViolation {
    pub operation_type: String,
    pub constraint: String,
    pub actual_value: String,
    pub expected_value: String,
}

/// Mock implementations for testing
#[cfg(feature = "test-utils")]
pub mod mocks {
    use super::*;

    /// Mock tool implementation
    #[derive(Debug)]
    pub struct MockTool {
        pub metadata: ToolMetadata,
        pub execute_result: Option<Result<ToolOutput>>,
        pub should_fail: bool,
    }

    impl MockTool {
        pub fn new(name: &str) -> Self {
            Self {
                metadata: ToolMetadata {
                    id: format!("mock-{}", name),
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    description: format!("Mock implementation of {}", name),
                    supported_inputs: vec![],
                    capabilities: ToolCapabilities {
                        async_execution: true,
                        parallel_processing: false,
                        incremental_processing: false,
                        requires_network: false,
                        max_input_size: None,
                    },
                },
                execute_result: None,
                should_fail: false,
            }
        }

        pub fn with_execute_result(mut self, result: Result<ToolOutput>) -> Self {
            self.execute_result = Some(result);
            self
        }

        pub fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        async fn execute(&self, _input: ToolInput) -> Result<ToolOutput> {
            if self.should_fail {
                return Err(crate::error::ParseltongError::ConfigurationError {
                    details: "Mock tool configured to fail".to_string(),
                });
            }

            match &self.execute_result {
                Some(Ok(output)) => Ok(output.clone()),
                Some(Err(e)) => Err((*e).clone()),
                None => Ok(ToolOutput::IndexingComplete {
                    entities_count: 0,
                    duration_ms: 0,
                }),
            }
        }

        fn validate_input(&self, _input: &ToolInput) -> Result<()> {
            if self.should_fail {
                return Err(crate::error::ParseltongError::ValidationError {
                    field: "input".to_string(),
                    expected: "valid input".to_string(),
                    actual: "mock failure".to_string(),
                });
            }
            Ok(())
        }

        fn metadata(&self) -> ToolMetadata {
            self.metadata.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata_creation() {
        let metadata = ToolMetadata {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            supported_inputs: vec!["test".to_string()],
            capabilities: ToolCapabilities {
                async_execution: true,
                parallel_processing: false,
                incremental_processing: true,
                requires_network: false,
                max_input_size: Some(1024),
            },
        };

        assert_eq!(metadata.id, "test-tool");
        assert!(metadata.capabilities.async_execution);
        assert!(!metadata.capabilities.parallel_processing);
    }

    #[test]
    fn temporal_query_creation() {
        let query = TemporalQuery {
            base_entities: vec!["test.rs-test-function".to_string()],
            hop_depth: 2,
            future_only: true,
            entity_type_filter: Some(EntityType::Function),
            language_filter: Some(Language::Rust),
        };

        assert_eq!(query.base_entities.len(), 1);
        assert_eq!(query.hop_depth, 2);
        assert!(query.future_only);
    }
}