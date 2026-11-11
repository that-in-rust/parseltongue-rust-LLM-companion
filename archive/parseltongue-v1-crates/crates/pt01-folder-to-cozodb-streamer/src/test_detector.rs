//! Test detection module implementing REQ-V090-003.0
//! 
//! Language-specific test detection following TDD principles with performance contracts

use std::path::Path;

/// Entity classification for code vs test separation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityClass {
    Code,
    Test,
}

/// Test detection interface with dependency injection for testability
pub trait TestDetector: Send + Sync {
    /// Detect if file contains test code based on language and patterns
    /// 
    /// # Performance Contract
    /// - Completes in <20μs per file
    /// - Language detection cached for repeated operations
    /// - Parse errors default to "non-test" classification
    fn detect_test_from_path_and_name(&self, file_path: &Path, content: &str) -> EntityClass;
}

/// Default implementation of test detection
pub struct DefaultTestDetector;

impl Default for DefaultTestDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultTestDetector {
    pub fn new() -> Self {
        Self
    }
}

impl TestDetector for DefaultTestDetector {
    fn detect_test_from_path_and_name(&self, file_path: &Path, content: &str) -> EntityClass {
        // REQ-V090-003.0: Language-specific test detection
        
        let path_str = file_path.to_string_lossy();
        
        // Rust test detection
        if path_str.ends_with(".rs") {
            return self.detect_rust_test(file_path, content);
        }
        
        // Go test detection
        if path_str.ends_with(".go") {
            return self.detect_go_test(file_path, content);
        }
        
        // JavaScript/TypeScript test detection
        if path_str.ends_with(".js") || path_str.ends_with(".ts") || 
           path_str.ends_with(".jsx") || path_str.ends_with(".tsx") {
            return self.detect_javascript_test(file_path, content);
        }
        
        // Python test detection
        if path_str.ends_with(".py") {
            return self.detect_python_test(file_path, content);
        }
        
        // Java test detection
        if path_str.ends_with(".java") {
            return self.detect_java_test(file_path, content);
        }
        
        // Unknown file extensions are treated as non-test files
        EntityClass::Code
    }
}

impl DefaultTestDetector {
    /// REQ-V090-003.1: Rust test detection
    ///
    /// Detects tests via:
    /// - `#[test]`, `#[tokio::test]`, `#[cfg(test)]` attributes
    /// - `tests/` directory
    /// - `_test.rs` filename pattern
    ///
    /// Special cases:
    /// - main.rs and lib.rs are ALWAYS Code (even if they contain #[cfg(test)] modules)
    fn detect_rust_test(&self, file_path: &Path, content: &str) -> EntityClass {
        let path_str = file_path.to_string_lossy();

        // Priority: main.rs and lib.rs are ALWAYS production code
        // They may contain #[cfg(test)] modules, but the file itself is Code
        if path_str.ends_with("main.rs") || path_str.ends_with("lib.rs") {
            return EntityClass::Code;
        }

        // Check directory pattern: tests/ directory
        if path_str.contains("tests/") {
            return EntityClass::Test;
        }

        // Check filename pattern: _test.rs
        if path_str.ends_with("_test.rs") {
            return EntityClass::Test;
        }

        // Check content for test attributes
        if content.contains("#[test]") ||
           content.contains("#[tokio::test]") ||
           content.contains("#[cfg(test)]") {
            return EntityClass::Test;
        }

        EntityClass::Code
    }
    
    /// REQ-V090-003.2: Go test detection
    /// 
    /// Detects tests via:
    /// - `*_test.go` files
    /// - Functions starting with `Test`, `Example`, `Benchmark`
    fn detect_go_test(&self, file_path: &Path, content: &str) -> EntityClass {
        let path_str = file_path.to_string_lossy();
        
        // Check filename pattern: *_test.go
        if path_str.ends_with("_test.go") {
            return EntityClass::Test;
        }
        
        // Check content for test functions
        if content.contains("func Test") || 
           content.contains("func Example") || 
           content.contains("func Benchmark") {
            return EntityClass::Test;
        }
        
        EntityClass::Code
    }
    
    /// REQ-V090-003.3: JavaScript/TypeScript test detection
    /// 
    /// Detects tests via:
    /// - `*.test.js`, `*.spec.ts`, `*.test.tsx`, `*.spec.jsx` files
    /// - `__tests__/` directory
    /// - Common test patterns in content
    fn detect_javascript_test(&self, file_path: &Path, content: &str) -> EntityClass {
        let path_str = file_path.to_string_lossy();
        
        // Check filename patterns
        if path_str.ends_with(".test.js") || 
           path_str.ends_with(".test.ts") || 
           path_str.ends_with(".test.tsx") || 
           path_str.ends_with(".spec.js") || 
           path_str.ends_with(".spec.ts") || 
           path_str.ends_with(".spec.jsx") {
            return EntityClass::Test;
        }
        
        // Check directory pattern: __tests__/
        if path_str.contains("__tests__/") {
            return EntityClass::Test;
        }
        
        // Check content for test patterns
        if content.contains("test(") || 
           content.contains("it(") || 
           content.contains("describe(") || 
           content.contains("expect(") {
            return EntityClass::Test;
        }
        
        EntityClass::Code
    }
    
    /// REQ-V090-003.4: Python test detection
    /// 
    /// Detects tests via:
    /// - `test_*.py`, `*_test.py` files
    /// - Functions/classes starting with `test_`, `Test`
    fn detect_python_test(&self, file_path: &Path, content: &str) -> EntityClass {
        let path_str = file_path.to_string_lossy();
        
        // Check filename patterns
        if path_str.ends_with("_test.py") || 
           path_str.starts_with("test_") && path_str.ends_with(".py") {
            return EntityClass::Test;
        }
        
        // Check content for test patterns
        if content.contains("def test_") || 
           content.contains("class Test") || 
           content.contains("@pytest") || 
           content.contains("unittest") {
            return EntityClass::Test;
        }
        
        EntityClass::Code
    }
    
    /// REQ-V090-003.5: Java test detection
    /// 
    /// Detects tests via:
    /// - `*Test.java`, `*Tests.java` files
    /// - Methods with `@Test` annotation
    fn detect_java_test(&self, file_path: &Path, content: &str) -> EntityClass {
        let path_str = file_path.to_string_lossy();
        
        // Check filename patterns
        if path_str.ends_with("Test.java") || 
           path_str.ends_with("Tests.java") {
            return EntityClass::Test;
        }
        
        // Check content for JUnit annotations
        if content.contains("@Test") || 
           content.contains("@BeforeEach") || 
           content.contains("@AfterEach") || 
           content.contains("junit") {
            return EntityClass::Test;
        }
        
        EntityClass::Code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test contract: Rust test detection
    #[test]
    fn test_rust_test_detection_contract() {
        let detector = DefaultTestDetector::new();
        
        // Test cases: (file_path, content, expected_result)
        let test_cases = vec![
            ("src/lib.rs", "fn normal_function() { }", EntityClass::Code),
            ("tests/integration_test.rs", "#[test] fn test_integration() { }", EntityClass::Test),
            ("src/module_test.rs", "#[tokio::test] async fn test_async() { }", EntityClass::Test),
            ("src/benches/bench.rs", "#[test] fn test_unit() { }", EntityClass::Test),
            ("src/main.rs", "fn main() { println!(\"hello\"); }", EntityClass::Code),
        ];
        
        for (file_path, content, expected) in test_cases {
            let result = detector.detect_test_from_path_and_name(
                Path::new(file_path), 
                content
            );
            assert_eq!(result, expected, 
                      "Rust file {} with content should be {:?}", file_path, expected);
        }
    }
    
    /// Test contract: Go test detection
    #[test]
    fn test_go_test_detection_contract() {
        let detector = DefaultTestDetector::new();
        
        let test_cases = vec![
            ("main.go", "func main() { }", EntityClass::Code),
            ("main_test.go", "func TestMain(t *testing.T) { }", EntityClass::Test),
            ("integration_test.go", "func ExampleUsage() { }", EntityClass::Test),
            ("benchmark_test.go", "func BenchmarkAlgorithm(b *testing.B) { }", EntityClass::Test),
            ("utils.go", "func helper() { }", EntityClass::Code),
        ];
        
        for (file_path, content, expected) in test_cases {
            let result = detector.detect_test_from_path_and_name(
                Path::new(file_path), 
                content
            );
            assert_eq!(result, expected,
                      "Go file {} should be {:?}", file_path, expected);
        }
    }
    
    /// Test contract: JavaScript/TypeScript test detection
    #[test]
    fn test_javascript_test_detection_contract() {
        let detector = DefaultTestDetector::new();
        
        let test_cases = vec![
            ("src/app.js", "function runApp() { }", EntityClass::Code),
            ("src/app.test.js", "test('app runs', () => { })", EntityClass::Test),
            ("utils.spec.ts", "describe('utils', () => { })", EntityClass::Test),
            ("__tests__/helper.js", "expect(true).toBe(true)", EntityClass::Test),
            ("components/Button.tsx", "export Button = () => {}", EntityClass::Code),
        ];
        
        for (file_path, content, expected) in test_cases {
            let result = detector.detect_test_from_path_and_name(
                Path::new(file_path), 
                content
            );
            assert_eq!(result, expected,
                      "JS/TS file {} should be {:?}", file_path, expected);
        }
    }

    /// Test contract: Unknown file extensions
    #[test]
    fn test_unknown_file_extensions_contract() {
        let detector = DefaultTestDetector::new();

        let result = detector.detect_test_from_path_and_name(
            Path::new("unknown.xyz"),
            "some content"
        );

        // Unknown extensions should be treated as non-test files
        assert_eq!(result, EntityClass::Code);
    }

    /// RED Test: main.rs with test module should still be classified as CODE
    ///
    /// Bug: Files containing #[cfg(test)] anywhere are classified as TEST,
    /// even if they contain production code like main()
    #[test]
    fn test_main_with_test_module_is_code() {
        let detector = DefaultTestDetector::new();

        // Realistic content: main.rs with both main() and test module
        let content = r#"
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        assert!(true);
    }
}
"#;

        let result = detector.detect_test_from_path_and_name(
            Path::new("src/main.rs"),
            content
        );

        // main.rs should ALWAYS be classified as CODE, even with test module
        assert_eq!(result, EntityClass::Code,
            "main.rs should be CODE even when it contains #[cfg(test)] module");
    }
}
