use parseltongue_core::entities::{DependencyEdge, Language};
use parseltongue_core::query_extractor::{ParsedEntity, QueryBasedExtractor};
use std::path::Path;

/// Load a source file from a T-folder in test-fixtures/
pub fn load_fixture_source_file(t_folder: &str, filename: &str) -> String {
    // Path from crates/parseltongue-core/ to test-fixtures/ is ../../test-fixtures/
    let path = format!("../../test-fixtures/{}/{}", t_folder, filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Missing fixture file: {}", path))
}

/// Detect language from filename extension
pub fn detect_language_from_filename(filename: &str) -> Language {
    Language::from_file_path(Path::new(filename))
        .unwrap_or_else(|| panic!("Unsupported language for fixture file: {}", filename))
}

/// Parse a fixture file and return entities + edges
pub fn parse_fixture_extract_results(
    t_folder: &str,
    filename: &str,
) -> (Vec<ParsedEntity>, Vec<DependencyEdge>) {
    let code = load_fixture_source_file(t_folder, filename);
    let language = detect_language_from_filename(filename);
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    extractor
        .parse_source(&code, Path::new(filename), language)
        .expect("Failed to parse fixture file")
}

/// Parse all fixture files in a T-folder, return combined entities + edges
pub fn validate_fixture_extraction_results(
    t_folder: &str,
    filenames: &[&str],
) -> Vec<(String, Vec<ParsedEntity>, Vec<DependencyEdge>)> {
    filenames
        .iter()
        .map(|filename| {
            let (entities, edges) = parse_fixture_extract_results(t_folder, filename);
            (filename.to_string(), entities, edges)
        })
        .collect()
}
