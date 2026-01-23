// Minimal test codebase - AFTER state
// Changes from BEFORE:
// 1. REMOVED: helper_to_remove function
// 2. MODIFIED: function_to_modify (different implementation)
// 3. ADDED: new_function
// 4. UNCHANGED: Config struct and methods
// 5. MODIFIED: caller_function (now calls new_function instead)

/// Configuration struct - UNCHANGED
pub struct Config {
    pub name: String,
    pub value: i32,
}

impl Config {
    /// Creates new config - UNCHANGED
    pub fn new(name: &str, value: i32) -> Self {
        Config {
            name: name.to_string(),
            value,
        }
    }

    /// Method that will be UNCHANGED
    pub fn display(&self) -> String {
        format!("{}: {}", self.name, self.value)
    }
}

// NOTE: helper_to_remove is REMOVED

/// Function that was MODIFIED - different implementation
pub fn function_to_modify(input: &str) -> String {
    // Changed from "Original:" to "Modified:"
    // Added extra processing
    let processed = input.to_uppercase();
    format!("Modified: {}", processed)
}

/// NEW function added in AFTER state
pub fn new_function(a: i32, b: i32) -> i32 {
    a + b
}

/// Function that was calling helper - MODIFIED to call new_function instead
/// This demonstrates how code evolves when dependencies change
pub fn caller_function(x: i32) -> i32 {
    new_function(x, 10)
}

/// Another function that uses Config - UNCHANGED
pub fn use_config(config: &Config) -> String {
    config.display()
}

/// Public API function - unchanged
pub fn main_api_function(name: &str, value: i32) -> String {
    let config = Config::new(name, value);
    use_config(&config)
}
