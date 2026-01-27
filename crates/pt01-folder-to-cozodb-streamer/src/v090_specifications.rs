//! TDD-First Executable Specifications for v0.9.0 Enhancement
//!
//! Following S01-README-MOSTIMP.md and S06-design101-tdd-architecture-principles.md
//! All specifications written as executable contracts with measurable outcomes

/// # Executable Specification: Exclusion Patterns Feature
/// 
/// ## Requirement ID: REQ-V090-001.0
/// 
/// ### Acceptance Criteria
/// 
/// 1. WHEN I run `pt01-folder-to-cozodb-streamer <dir> -e ".ref" -e "archive"`
///    THEN the system SHALL exclude all files/directories containing ".ref" or "archive" from analysis
/// 
/// 2. WHEN I specify multiple exclusion patterns with `-e` flag
///    THEN the system SHALL apply ALL patterns (logical OR) for filtering
/// 
/// 3. WHEN no exclusion patterns are specified
///    THEN the system SHALL use default patterns only (target, node_modules, .git, etc.)
/// 
/// ### Performance Contract
/// 
/// - Pattern matching SHALL complete in <10μs per file path
/// - Memory overhead SHALL be <1KB for 100 exclusion patterns
/// 
/// ### Error Conditions
/// 
/// - Invalid patterns SHALL be ignored with warning
/// - Empty patterns SHALL be ignored
/// 
#[cfg(test)]
mod exclusion_patterns_tests {
    use super::*;
    use std::path::PathBuf;
    
    /// Test contract: Single exclusion pattern works
    #[test]
    fn test_single_exclusion_pattern_contract() {
        // Given: Exclusion pattern ".ref"
        let patterns = vec![".ref".to_string()];
        
        // When: Checking path ".ref/tool-semgrep"
        let path = PathBuf::from(".ref/tool-semgrep");
        
        // Then: Should be excluded
        assert!(should_exclude_path(&path, &patterns));
        
        // And: Normal source code should not be excluded
        let src_path = PathBuf::from("src/main.rs");
        assert!(!should_exclude_path(&src_path, &patterns));
    }
    
    /// Test contract: Multiple exclusion patterns work
    #[test]
    fn test_multiple_exclusion_patterns_contract() {
        // Given: Multiple exclusion patterns
        let patterns = vec![".ref".to_string(), "archive".to_string(), "tmp".to_string()];
        
        // When: Checking various paths
        let ref_path = PathBuf::from(".ref/tool-semgrep");
        let archive_path = PathBuf::from("archive/old-project");
        let tmp_path = PathBuf::from("tmp/cache");
        let src_path = PathBuf::from("src/main.rs");
        
        // Then: Correct exclusion decisions
        assert!(should_exclude_path(&ref_path, &patterns));      // .ref excluded
        assert!(should_exclude_path(&archive_path, &patterns));  // archive excluded
        assert!(should_exclude_path(&tmp_path, &patterns));      // tmp excluded
        assert!(!should_exclude_path(&src_path, &patterns));     // src included
    }
    
    /// Performance contract: Pattern matching speed
    #[test]
    fn test_pattern_matching_performance_contract() {
        // Given: 100 exclusion patterns
        let patterns: Vec<String> = (0..100)
            .map(|i| format!("pattern_{}", i))
            .collect();
        
        let test_path = PathBuf::from("pattern_42/some_file.rs");
        
        // When: Matching pattern
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            should_exclude_path(&test_path, &patterns);
        }
        let elapsed = start.elapsed();
        
        // Then: Performance contract satisfied (<10μs per check)
        let avg_time_per_check = elapsed / 1000;
        assert!(avg_time_per_check < std::time::Duration::from_micros(10),
                "Pattern matching took {:?}, expected <10μs", avg_time_per_check);
    }
}

/// # Executable Specification: Git Subfolder Detection
/// 
/// ## Requirement ID: REQ-V090-002.0
/// 
/// ### Acceptance Criteria
/// 
/// 1. WHEN a directory contains `.git/` subdirectory
///    THEN the system SHALL exclude it UNLESS it's the project root
/// 
/// 2. WHEN analyzing nested git repositories
///    THEN the system SHALL prevent analyzing external tool dependencies
/// 
/// 3. WHEN project root itself contains `.git/`
///    THEN the system SHALL NOT exclude the project root
/// 
/// ### Performance Contract
/// 
/// - Git detection SHALL complete in <50μs per path
/// - Directory traversal SHALL stop at project root boundary
/// 
/// ### Error Conditions
/// 
/// - Permission errors accessing `.git` SHALL be treated as "not a git repo"
/// - Symlink loops SHALL be handled gracefully
/// 
#[cfg(test)]
mod git_subfolder_tests {
    use super::*;
    use std::path::PathBuf;
    
    /// Test contract: Nested git repository detection
    #[test]
    fn test_nested_git_repo_detection_contract() {
        // Given: Create a temporary directory structure for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();
        
        // Create nested structure with .git
        let nested_dir = project_root.join(".ref").join("tool-semgrep");
        std::fs::create_dir_all(&nested_dir).unwrap();
        let nested_git = nested_dir.join(".git");
        std::fs::create_dir(&nested_git).unwrap();
        
        let test_file = nested_dir.join("some_file.rs");
        
        // When: Checking if test_file is under git subdirectory
        let is_nested = is_under_git_subdirectory(&test_file, project_root);
        
        // Then: Should detect as nested git repo
        assert!(is_nested, "Should detect nested git repository");
    }
    
    /// Test contract: Project root git is not excluded
    #[test]
    fn test_project_root_git_not_excluded_contract() {
        // Given: Create a temporary directory structure for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();
        
        // Create project root .git
        let project_git = project_root.join(".git");
        std::fs::create_dir(&project_git).unwrap();
        
        let test_file = project_root.join("src/main.rs");
        
        // When: Checking project root file
        let is_nested = is_under_git_subdirectory(&test_file, project_root);
        
        // Then: Should NOT be considered nested (it's the main repo)
        assert!(!is_nested, "Project root .git should not be excluded");
    }
    
    /// Performance contract: Git detection speed
    #[test]
    fn test_git_detection_performance_contract() {
        // Given: Deep path structure
        let project_root = PathBuf::from("/project");
        let deep_path = PathBuf::from("/project/a/b/c/d/e/f/g/h/i/j/file.rs");
        
        // When: Checking for nested git
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            is_under_git_subdirectory(&deep_path, &project_root);
        }
        let elapsed = start.elapsed();
        
        // Then: Performance contract satisfied (<50μs per check)
        let avg_time_per_check = elapsed / 1000;
        assert!(avg_time_per_check < std::time::Duration::from_micros(50),
                "Git detection took {:?}, expected <50μs", avg_time_per_check);
    }
}

/// # Executable Specification: Test Detection by Language
/// 
/// ## Requirement ID: REQ-V090-003.0
/// 
/// ### Acceptance Criteria
/// 
/// 1. WHEN processing Rust files
///    THEN the system SHALL detect tests via `#[test]`, `#[tokio::test]`, `tests/` directory, `_test.rs`
/// 
/// 2. WHEN processing Go files
///    THEN the system SHALL detect tests via `*_test.go` files and `Test*` functions
/// 
/// 3. WHEN processing JavaScript/TypeScript files
///    THEN the system SHALL detect tests via `*.test.js`, `*.spec.ts`, `__tests__/` directory
/// 
/// 4. WHEN processing Python files
///    THEN the system SHALL detect tests via `test_*.py`, `*_test.py`, `test_*` functions/classes
/// 
/// 5. WHEN processing Java files
///    THEN the system SHALL detect tests via `*Test.java`, `*Tests.java`, `@Test` methods
/// 
/// ### Performance Contract
/// 
/// - Test detection SHALL complete in <20μs per file
/// - Language detection SHALL be cached for repeated operations
/// 
/// ### Error Conditions
/// 
/// - Unknown file extensions SHALL be treated as non-test files
/// - Parse errors SHALL default to "non-test" classification
/// 
#[cfg(test)]
mod test_detection_tests {
    use super::*;
    use std::path::PathBuf;
    
    /// Test contract: Rust test detection
    #[test]
    fn test_rust_test_detection_contract() {
        // Given: Rust test patterns
        let test_cases = vec![
            ("src/lib.rs", "fn normal_function() { }", false),
            ("tests/integration_test.rs", "#[test] fn test_integration() { }", true),
            ("src/module_test.rs", "#[tokio::test] async fn test_async() { }", true),
            ("src/benches/bench.rs", "#[test] fn test_unit() { }", true),
        ];
        
        for (file_path, content, expected_is_test) in test_cases {
            // When: Detecting test status
            let is_test = detect_test_from_content(&PathBuf::from(file_path), content);
            
            // Then: Correct classification
            assert_eq!(is_test, expected_is_test, 
                      "File {} with content should be test: {}", file_path, expected_is_test);
        }
    }
    
    /// Test contract: Go test detection
    #[test]
    fn test_go_test_detection_contract() {
        // Given: Go test patterns
        let test_cases = vec![
            ("main.go", "func main() { }", false),
            ("main_test.go", "func TestMain(t *testing.T) { }", true),
            ("integration_test.go", "func ExampleUsage() { }", true),
            ("benchmark_test.go", "func BenchmarkAlgorithm(b *testing.B) { }", true),
        ];
        
        for (file_path, content, expected_is_test) in test_cases {
            // When: Detecting test status
            let is_test = detect_test_from_content(&PathBuf::from(file_path), content);
            
            // Then: Correct classification
            assert_eq!(is_test, expected_is_test,
                      "Go file {} should be test: {}", file_path, expected_is_test);
        }
    }
    
    /// Test contract: JavaScript/TypeScript test detection
    #[test]
    fn test_javascript_test_detection_contract() {
        // Given: JS/TS test patterns
        let test_cases = vec![
            ("src/app.js", "function runApp() { }", false),
            ("src/app.test.js", "test('app runs', () => { })", true),
            ("utils.spec.ts", "describe('utils', () => { })", true),
            ("__tests__/helper.js", "expect(true).toBe(true)", true),
        ];
        
        for (file_path, content, expected_is_test) in test_cases {
            // When: Detecting test status
            let is_test = detect_test_from_content(&PathBuf::from(file_path), content);
            
            // Then: Correct classification
            assert_eq!(is_test, expected_is_test,
                      "JS/TS file {} should be test: {}", file_path, expected_is_test);
        }
    }
}

// Test helper functions (only compiled for tests)
#[cfg(test)]
fn should_exclude_path(path: &std::path::Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    
    // Check each pattern for substring match
    for pattern in patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
fn is_under_git_subdirectory(path: &std::path::Path, project_root: &std::path::Path) -> bool {
    let mut current = path;
    
    // Walk up parent directories looking for .git
    while let Some(parent) = current.parent() {
        // Stop at project root (don't exclude project root itself)
        if parent == project_root {
            break;
        }
        
        // Don't go beyond project root
        if !parent.starts_with(project_root) {
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

#[cfg(test)]
fn detect_test_from_content(path: &std::path::Path, content: &str) -> bool {
    use crate::test_detector::{TestDetector, DefaultTestDetector, EntityClass};
    
    let detector = DefaultTestDetector::new();
    let result = detector.detect_test_from_path_and_name(path, content);
    
    matches!(result, EntityClass::Test)
}
