//! Path filtering for file watcher events
//!
//! # 4-Word Naming: path_filter_pattern_matcher
//!
//! This module provides pattern matching capabilities to filter out
//! irrelevant file system events (build artifacts, dependencies, etc.)
//!
//! ## Requirements Implemented:
//! - REQ-FILEWATCHER-006: Default ignore patterns
//! - REQ-FILEWATCHER-007: Custom ignore patterns
//! - REQ-FILEWATCHER-008: Pattern matching performance
//! - REQ-FILEWATCHER-009: Hidden files filtering
//! - REQ-FILEWATCHER-010: Symlink handling

use glob::Pattern;
use std::path::Path;

use super::watcher_types::{
    FileWatcherErrorType,
    DEFAULT_IGNORE_PATTERNS_LIST,
};

// =============================================================================
// Path Filter Configuration
// =============================================================================

/// Path filter configuration and compiled patterns
///
/// # 4-Word Name: PathFilterConfigurationStruct
///
/// Contains compiled glob patterns for efficient path filtering.
/// Patterns are compiled once and reused for all filtering operations.
#[derive(Debug, Clone)]
pub struct PathFilterConfigurationStruct {
    /// Compiled glob patterns for ignore matching
    compiled_patterns_list_value: Vec<Pattern>,
    /// Original pattern strings (for debugging)
    pattern_strings_list_value: Vec<String>,
}

impl Default for PathFilterConfigurationStruct {
    fn default() -> Self {
        Self::create_with_default_patterns().unwrap_or_else(|_| Self {
            compiled_patterns_list_value: Vec::new(),
            pattern_strings_list_value: Vec::new(),
        })
    }
}

impl PathFilterConfigurationStruct {
    /// Create filter with default patterns only
    ///
    /// # 4-Word Name: create_with_default_patterns
    pub fn create_with_default_patterns() -> Result<Self, FileWatcherErrorType> {
        compile_ignore_patterns_list(DEFAULT_IGNORE_PATTERNS_LIST)
    }

    /// Create filter with custom patterns (additive to defaults)
    ///
    /// # 4-Word Name: create_with_custom_patterns
    pub fn create_with_custom_patterns(
        custom_patterns: &[String],
    ) -> Result<Self, FileWatcherErrorType> {
        let mut all_patterns: Vec<&str> = DEFAULT_IGNORE_PATTERNS_LIST.to_vec();
        let custom_refs: Vec<&str> = custom_patterns.iter().map(|s| s.as_str()).collect();
        all_patterns.extend(custom_refs);

        compile_ignore_patterns_list(&all_patterns)
    }

    /// Check if path should be ignored
    ///
    /// # 4-Word Name: should_ignore_path_check
    pub fn should_ignore_path_check(&self, path: &Path) -> bool {
        filter_path_against_patterns(path, &self.compiled_patterns_list_value)
    }

    /// Get pattern count for debugging
    ///
    /// # 4-Word Name: get_pattern_count_value
    pub fn get_pattern_count_value(&self) -> usize {
        self.compiled_patterns_list_value.len()
    }

    /// Get pattern strings for debugging
    ///
    /// # 4-Word Name: get_pattern_strings_list
    pub fn get_pattern_strings_list(&self) -> &[String] {
        &self.pattern_strings_list_value
    }
}

// =============================================================================
// Core Filtering Functions
// =============================================================================

/// Filter path against compiled patterns
///
/// # 4-Word Name: filter_path_against_patterns
///
/// Returns `true` if the path should be filtered (ignored),
/// `false` if the path should be passed through.
///
/// ## Performance Contract
/// - Pattern matching completes in < 100 microseconds per path
/// - Uses early exit on first match (short-circuit)
///
/// ## Example
/// ```rust,ignore
/// let patterns = compile_ignore_patterns_list(&["**/target/**"])?;
/// let should_filter = filter_path_against_patterns(
///     Path::new("/project/target/debug/binary"),
///     &patterns,
/// );
/// assert!(should_filter); // true - should be ignored
/// ```
pub fn filter_path_against_patterns(path: &Path, patterns: &[Pattern]) -> bool {
    let path_str = path.to_string_lossy();

    // Early exit on first match (short-circuit optimization)
    for pattern in patterns {
        if pattern.matches(&path_str) {
            return true;
        }
    }

    false
}

/// Compile ignore patterns list into Pattern objects
///
/// # 4-Word Name: compile_ignore_patterns_list
///
/// Takes a slice of pattern strings and compiles them into
/// glob Pattern objects for efficient matching.
///
/// ## Error Handling
/// - Invalid patterns are logged and skipped (not fatal)
/// - Returns error only if all patterns are invalid
///
/// ## Example
/// ```rust,ignore
/// let patterns = compile_ignore_patterns_list(&[
///     "**/target/**",
///     "**/node_modules/**",
/// ])?;
/// ```
pub fn compile_ignore_patterns_list(
    patterns: &[&str],
) -> Result<PathFilterConfigurationStruct, FileWatcherErrorType> {
    let mut compiled = Vec::with_capacity(patterns.len());
    let mut strings = Vec::with_capacity(patterns.len());
    let mut invalid_count = 0;

    for pattern_str in patterns {
        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                compiled.push(pattern);
                strings.push(pattern_str.to_string());
            }
            Err(e) => {
                // Log warning but continue with valid patterns
                eprintln!(
                    "Warning: Invalid glob pattern '{}': {}",
                    pattern_str, e
                );
                invalid_count += 1;
            }
        }
    }

    // Only error if ALL patterns are invalid
    if compiled.is_empty() && !patterns.is_empty() {
        return Err(FileWatcherErrorType::InvalidGlobPatternError(
            format!("All {} patterns were invalid", invalid_count)
        ));
    }

    Ok(PathFilterConfigurationStruct {
        compiled_patterns_list_value: compiled,
        pattern_strings_list_value: strings,
    })
}

/// Check if path is a hidden file
///
/// # 4-Word Name: is_hidden_file_path
///
/// Returns true if the file or any directory in the path starts with '.'
pub fn is_hidden_file_path(path: &Path) -> bool {
    path.components().any(|component| {
        component.as_os_str()
            .to_string_lossy()
            .starts_with('.')
    })
}

/// Check if path is a source code file
///
/// # 4-Word Name: is_source_code_file
///
/// Returns true if the file has a recognized source code extension.
pub fn is_source_code_file(path: &Path) -> bool {
    let source_extensions = [
        "rs", "py", "js", "ts", "tsx", "jsx",
        "go", "java", "c", "cpp", "h", "hpp",
        "rb", "php", "cs", "swift",
    ];

    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| source_extensions.contains(&ext))
        .unwrap_or(false)
}

/// Determine if path should pass through filters
///
/// # 4-Word Name: should_path_pass_through
///
/// Combines pattern matching with special cases:
/// - Hidden source files pass through (.hidden.rs)
/// - Config files in .github/ pass through
/// - Hidden files without source extension filtered
pub fn should_path_pass_through(path: &Path, filter: &PathFilterConfigurationStruct) -> bool {
    // First check against ignore patterns
    if filter.should_ignore_path_check(path) {
        return false;
    }

    // Check for hidden files with special handling
    if is_hidden_file_path(path) {
        // Allow hidden files with source extensions
        if is_source_code_file(path) {
            return true;
        }

        // Allow .github workflow files
        let path_str = path.to_string_lossy();
        if path_str.contains(".github/") {
            return true;
        }

        // Allow specific config files
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let allowed_configs = [
            ".eslintrc", ".prettierrc", ".editorconfig",
            ".eslintrc.js", ".eslintrc.json", ".prettierrc.js",
        ];
        if allowed_configs.iter().any(|c| filename.starts_with(c)) {
            return true;
        }

        // Filter other hidden files
        return false;
    }

    // Non-hidden, non-ignored paths pass through
    true
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // =========================================================================
    // PathFilterConfigurationStruct Tests
    // =========================================================================

    /// Test default filter creation
    #[test]
    fn test_default_filter_creation() {
        let filter = PathFilterConfigurationStruct::default();
        assert!(filter.get_pattern_count_value() > 0);
    }

    /// Test custom patterns are additive
    #[test]
    fn test_custom_patterns_additive() {
        let default_filter = PathFilterConfigurationStruct::create_with_default_patterns()
            .unwrap();
        let custom_filter = PathFilterConfigurationStruct::create_with_custom_patterns(
            &["**/custom/**".to_string()],
        ).unwrap();

        assert!(custom_filter.get_pattern_count_value() > default_filter.get_pattern_count_value());
    }

    // =========================================================================
    // REQ-FILEWATCHER-006: Default Ignore Patterns Tests
    // =========================================================================

    /// REQ-FILEWATCHER-006.1: target/ directory events filtered
    #[test]
    fn test_target_directory_filtered() {
        let filter = PathFilterConfigurationStruct::default();
        let path = PathBuf::from("/project/target/debug/binary");

        assert!(filter.should_ignore_path_check(&path));
    }

    /// REQ-FILEWATCHER-006.1: node_modules/ directory events filtered
    #[test]
    fn test_node_modules_directory_filtered() {
        let filter = PathFilterConfigurationStruct::default();
        let path = PathBuf::from("/project/node_modules/lodash/index.js");

        assert!(filter.should_ignore_path_check(&path));
    }

    /// REQ-FILEWATCHER-006.1: .git/ directory events filtered
    #[test]
    fn test_git_directory_filtered() {
        let filter = PathFilterConfigurationStruct::default();
        let path = PathBuf::from("/project/.git/objects/abc123");

        assert!(filter.should_ignore_path_check(&path));
    }

    /// REQ-FILEWATCHER-006.2: Lock files filtered
    #[test]
    fn test_cargo_lock_filtered() {
        let filter = PathFilterConfigurationStruct::default();

        assert!(filter.should_ignore_path_check(Path::new("/project/Cargo.lock")));
        assert!(filter.should_ignore_path_check(Path::new("/project/package-lock.json")));
        assert!(filter.should_ignore_path_check(Path::new("/project/yarn.lock")));
    }

    /// REQ-FILEWATCHER-006.2: Swap files filtered
    #[test]
    fn test_swap_files_filtered() {
        let filter = PathFilterConfigurationStruct::default();

        assert!(filter.should_ignore_path_check(Path::new("/project/src/main.rs.swp")));
        assert!(filter.should_ignore_path_check(Path::new("/project/src/.main.rs.swo")));
        assert!(filter.should_ignore_path_check(Path::new("/project/src/main.rs~")));
    }

    /// REQ-FILEWATCHER-006.3: Source files pass through
    #[test]
    fn test_source_files_pass_through() {
        let filter = PathFilterConfigurationStruct::default();

        assert!(!filter.should_ignore_path_check(Path::new("/project/src/main.rs")));
        assert!(!filter.should_ignore_path_check(Path::new("/project/src/lib/utils.py")));
        assert!(!filter.should_ignore_path_check(Path::new("/project/app/index.ts")));
    }

    // =========================================================================
    // REQ-FILEWATCHER-007: Custom Ignore Patterns Tests
    // =========================================================================

    /// REQ-FILEWATCHER-007.1: Custom patterns filter correctly
    #[test]
    fn test_custom_patterns_filter() {
        let filter = PathFilterConfigurationStruct::create_with_custom_patterns(
            &["**/generated/**".to_string()],
        ).unwrap();

        assert!(filter.should_ignore_path_check(Path::new("/project/generated/code.rs")));
    }

    /// REQ-FILEWATCHER-007.2: Invalid patterns are skipped
    #[test]
    fn test_invalid_patterns_skipped() {
        // Create filter with mix of valid and invalid patterns
        let result = compile_ignore_patterns_list(&[
            "**/valid/**",
            "[invalid",  // Invalid glob syntax
            "**/also_valid/**",
        ]);

        // Should succeed with valid patterns
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.get_pattern_count_value(), 2);
    }

    // =========================================================================
    // REQ-FILEWATCHER-008: Pattern Matching Performance Tests
    // =========================================================================

    /// REQ-FILEWATCHER-008.1: Pattern matching is fast
    #[test]
    fn test_pattern_matching_performance() {
        let filter = PathFilterConfigurationStruct::default();
        let test_paths: Vec<PathBuf> = (0..1000)
            .map(|i| PathBuf::from(format!("/project/src/module_{}/file_{}.rs", i % 100, i)))
            .collect();

        let start = std::time::Instant::now();
        for path in &test_paths {
            let _ = filter.should_ignore_path_check(path);
        }
        let elapsed = start.elapsed();

        let per_path = elapsed / test_paths.len() as u32;
        assert!(
            per_path.as_micros() < 100,
            "Pattern match too slow: {:?} per path",
            per_path
        );
    }

    // =========================================================================
    // REQ-FILEWATCHER-009: Hidden Files Filtering Tests
    // =========================================================================

    /// REQ-FILEWATCHER-009.1: Hidden file detection
    #[test]
    fn test_hidden_file_detection() {
        assert!(is_hidden_file_path(Path::new("/project/.hidden")));
        assert!(is_hidden_file_path(Path::new("/project/.config/file")));
        assert!(!is_hidden_file_path(Path::new("/project/visible")));
    }

    /// REQ-FILEWATCHER-009.2: Hidden source files pass through
    #[test]
    fn test_hidden_source_files_pass() {
        let filter = PathFilterConfigurationStruct::default();
        let path = Path::new("/project/.hidden.rs");

        assert!(should_path_pass_through(path, &filter));
    }

    /// REQ-FILEWATCHER-009.2: .github/ files pass through
    #[test]
    fn test_github_files_pass() {
        let filter = PathFilterConfigurationStruct::default();
        let path = Path::new("/project/.github/workflows/ci.yml");

        assert!(should_path_pass_through(path, &filter));
    }

    /// Test .env files are filtered
    #[test]
    fn test_env_files_filtered() {
        let filter = PathFilterConfigurationStruct::default();

        assert!(filter.should_ignore_path_check(Path::new("/project/.env")));
        assert!(filter.should_ignore_path_check(Path::new("/project/.env.local")));
    }

    // =========================================================================
    // Source Code File Detection Tests
    // =========================================================================

    /// Test source code file detection
    #[test]
    fn test_source_code_file_detection() {
        assert!(is_source_code_file(Path::new("main.rs")));
        assert!(is_source_code_file(Path::new("app.py")));
        assert!(is_source_code_file(Path::new("index.tsx")));
        assert!(!is_source_code_file(Path::new("data.json")));
        assert!(!is_source_code_file(Path::new("README.md")));
    }

    // =========================================================================
    // compile_ignore_patterns_list Tests
    // =========================================================================

    /// Test empty patterns list
    #[test]
    fn test_empty_patterns_list() {
        let result = compile_ignore_patterns_list(&[]);
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.get_pattern_count_value(), 0);
    }

    /// Test all invalid patterns returns error
    #[test]
    fn test_all_invalid_patterns_error() {
        let result = compile_ignore_patterns_list(&[
            "[invalid1",
            "[invalid2",
        ]);

        assert!(result.is_err());
        if let Err(FileWatcherErrorType::InvalidGlobPatternError(msg)) = result {
            assert!(msg.contains("invalid"));
        } else {
            panic!("Expected InvalidGlobPatternError");
        }
    }

    // =========================================================================
    // should_path_pass_through Tests
    // =========================================================================

    /// Test complete pass-through logic
    #[test]
    fn test_path_pass_through_logic() {
        let filter = PathFilterConfigurationStruct::default();

        // Source files pass
        assert!(should_path_pass_through(Path::new("/project/src/main.rs"), &filter));

        // target/ filtered
        assert!(!should_path_pass_through(Path::new("/project/target/debug/bin"), &filter));

        // Hidden non-source filtered
        assert!(!should_path_pass_through(Path::new("/project/.hidden"), &filter));

        // Hidden source passes
        assert!(should_path_pass_through(Path::new("/project/.hidden.rs"), &filter));
    }
}
