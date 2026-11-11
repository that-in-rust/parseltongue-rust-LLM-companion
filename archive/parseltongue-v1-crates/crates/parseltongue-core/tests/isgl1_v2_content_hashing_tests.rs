// ISGL1 v2 Cycle 2: Content Hashing Tests
//
// These tests verify SHA-256 content hashing for entity change detection.
// Content hashing enables incremental indexing by detecting when entity code changes.

use parseltongue_core::isgl1_v2::compute_content_hash;

/// Test 1: Basic SHA-256 hashing
///
/// Verifies that compute_content_hash produces valid SHA-256 output:
/// - 64-character hex string
/// - All characters are valid hexadecimal digits
#[test]
fn test_compute_content_hash_sha256() {
    let code = "fn main() { println!(\"Hello\"); }";
    let hash = compute_content_hash(code);

    // SHA-256 produces 64-character hex string
    assert_eq!(hash.len(), 64, "SHA-256 hash must be 64 hex characters");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash must contain only hexadecimal characters"
    );
}

/// Test 2: Deterministic hashing (same input = same output)
///
/// Critical for incremental indexing: the same code must always produce
/// the same hash, enabling reliable change detection.
#[test]
fn test_content_hash_deterministic() {
    let code = "fn calculate() -> i32 { 42 }";
    let hash1 = compute_content_hash(code);
    let hash2 = compute_content_hash(code);

    assert_eq!(
        hash1, hash2,
        "Same code must produce identical hash (deterministic)"
    );
}

/// Test 3: Different content = different hash
///
/// Ensures hash sensitivity: even small code changes produce different hashes.
#[test]
fn test_content_hash_differs_by_content() {
    let code1 = "fn main() { println!(\"Hello\"); }";
    let code2 = "fn main() { println!(\"World\"); }";

    let hash1 = compute_content_hash(code1);
    let hash2 = compute_content_hash(code2);

    assert_ne!(
        hash1, hash2,
        "Different code must produce different hashes"
    );
}

/// Test 4: Whitespace sensitivity (formatting matters for change detection)
///
/// Intentionally whitespace-sensitive to detect formatting changes.
/// This ensures we catch ALL changes, including style refactoring.
#[test]
fn test_content_hash_whitespace_sensitive() {
    let code1 = "fn main(){println!(\"test\");}";
    let code2 = "fn main() {\n    println!(\"test\");\n}";

    let hash1 = compute_content_hash(code1);
    let hash2 = compute_content_hash(code2);

    // Different formatting = different hash (intentional for precise change detection)
    assert_ne!(
        hash1, hash2,
        "Different whitespace/formatting must produce different hashes"
    );
}
