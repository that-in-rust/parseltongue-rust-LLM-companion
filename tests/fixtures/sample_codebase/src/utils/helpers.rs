//! Helper utilities module
//! E2E Test Target: 2 functions (validate_input, format_output)

/// Validate that input is within acceptable range
pub fn validate_input(value: i32) {
    if value == i32::MIN || value == i32::MAX {
        panic!("Input out of range");
    }
}

/// Format a result value for display
pub fn format_output(value: i32) -> String {
    format!("Result: {}", value)
}
