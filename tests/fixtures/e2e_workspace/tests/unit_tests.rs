//! Unit tests for sample codebase
//! E2E Test: Should be EXCLUDED from indexing (TEST entities)

use sample_codebase::calculator;

#[test]
fn test_add() {
    assert_eq!(calculator::add(2, 3), 5);
}

#[test]
fn test_subtract() {
    assert_eq!(calculator::subtract(5, 3), 2);
}
