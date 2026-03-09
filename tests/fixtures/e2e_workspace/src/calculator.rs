//! Calculator module - primary E2E test target for modification
//! E2E Test Target: 3 functions (add, subtract, multiply)
//! # Version: INITIAL

use crate::utils::helpers::validate_input;

/// Add two numbers
pub fn add(a: i32, b: i32) -> i32 {
    validate_input(a);
    validate_input(b);
    a + b
}

/// Subtract two numbers
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Multiply two numbers
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
