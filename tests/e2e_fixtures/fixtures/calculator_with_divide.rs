//! Calculator module - MODIFIED version with divide function added
//! E2E Test Target: 4 functions (add, subtract, multiply, divide)
//! # Version: MODIFIED

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

/// Divide two numbers (NEW function)
pub fn divide(a: i32, b: i32) -> Option<i32> {
    validate_input(a);
    validate_input(b);
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}
