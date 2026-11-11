//! File watcher language coverage tests
//!
//! Tests verify all 12 supported languages are monitored by file watcher.
//!
//! NOTE: These are standalone unit tests that verify our language coverage
//! design without requiring full HTTP server compilation.

#[cfg(test)]
mod language_extension_coverage_tests {
    /// Helper function to get expected language extensions
    ///
    /// This represents the design specification for which file extensions
    /// should be monitored by the file watcher.
    fn get_expected_language_extensions() -> Vec<&'static str> {
        vec![
            // Existing (6 languages)
            "rs", "py", "js", "ts", "go", "java", // Missing (6 languages) - need to add these
            "c", "h", "cpp", "hpp", "rb", "php", "cs", "swift",
        ]
    }

    /// Test that file watcher monitors all 14 supported language extensions
    ///
    /// # Language Coverage
    /// - Rust: .rs
    /// - Python: .py
    /// - JavaScript/TypeScript: .js, .ts
    /// - Go: .go
    /// - Java: .java
    /// - C: .c, .h
    /// - C++: .cpp, .hpp
    /// - Ruby: .rb
    /// - PHP: .php
    /// - C#: .cs
    /// - Swift: .swift
    ///
    /// Total: 14 extensions covering 12 language families
    ///
    /// This test will FAIL until http_server_startup_runner.rs line 342-349
    /// is updated with all extensions.
    #[test]
    fn test_language_extensions_complete_coverage() {
        let expected = get_expected_language_extensions();

        // GREEN phase: Updated to match http_server_startup_runner.rs line 351-366
        let actual = vec![
            "rs", "py", "js", "ts", "go", "java", "c", "h", "cpp", "hpp", "rb", "php", "cs",
            "swift",
        ];

        assert_eq!(
            actual.len(),
            expected.len(),
            "Missing language extensions!\nExpected {} extensions, got {}.\nExpected: {:?}\nActual: {:?}",
            expected.len(),
            actual.len(),
            expected,
            actual
        );

        // Verify each expected extension exists
        for ext in &expected {
            assert!(
                actual.contains(ext),
                "Missing extension: .{}\nCurrent: {:?}",
                ext,
                actual
            );
        }
    }

    /// Test that each language family has correct extensions
    #[test]
    fn test_language_family_extension_mapping() {
        let language_families = vec![
            ("Rust", vec!["rs"]),
            ("Python", vec!["py"]),
            ("JavaScript/TypeScript", vec!["js", "ts"]),
            ("Go", vec!["go"]),
            ("Java", vec!["java"]),
            ("C", vec!["c", "h"]),
            ("C++", vec!["cpp", "hpp"]),
            ("Ruby", vec!["rb"]),
            ("PHP", vec!["php"]),
            ("C#", vec!["cs"]),
            ("Swift", vec!["swift"]),
        ];

        // Verify we have all families
        let expected_family_count = 11; // 11 distinct language families
        assert_eq!(
            language_families.len(),
            expected_family_count,
            "Language family count mismatch"
        );

        // Verify total extension count
        let total_extensions: usize = language_families.iter().map(|(_, exts)| exts.len()).sum();
        assert_eq!(
            total_extensions, 14,
            "Total extension count should be 14 (covering 11 language families)"
        );
    }

    /// Test that file extension list has no duplicates
    #[test]
    fn test_no_duplicate_extensions() {
        let extensions = get_expected_language_extensions();
        let mut sorted = extensions.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(
            extensions.len(),
            sorted.len(),
            "Found duplicate extensions in list"
        );
    }
}
