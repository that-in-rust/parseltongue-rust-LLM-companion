//! Library root for sample codebase
//! E2E Test Target: 2 functions (init, get_version)

pub mod calculator;
pub mod utils;

/// Initialize the library
pub fn init() {
    println!("Initializing sample_codebase v{}", get_version());
}

/// Get the current version
pub fn get_version() -> &'static str {
    "0.1.0"
}
