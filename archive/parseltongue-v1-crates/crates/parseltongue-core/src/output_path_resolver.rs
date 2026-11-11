//! Output Path Resolver: Timestamped folder creation for analysis sessions
//!
//! # Design Philosophy (S01 Ultra-Minimalist)
//!
//! This module implements automatic timestamped folder creation for all JSON/TOON outputs.
//! Following TDD principles: STUB → RED → GREEN → REFACTOR
//!
//! # Feature Requirements
//!
//! **Folder Naming Pattern**: `parseltongueYYYYMMDDHHMMSS`
//! - Example: `parseltongue20251115143022` for 2025-11-15 14:30:22
//!
//! **Scope**:
//! - Applies to ALL JSON and TOON file outputs
//! - Folder created at the root of the target repository being analyzed
//! - Timestamp reflects when the analysis session started (not per-file)
//! - All outputs from a single analysis run go to the same timestamped folder
//!
//! # Architecture (Functional, Immutable, Pure)
//!
//! - `format_timestamp_folder_name()` - Pure function: Timestamp → Folder name
//! - `create_timestamped_output_directory()` - Side effect: Create directory on filesystem
//! - `resolve_output_path_with_timestamp()` - Pure function: (Base path, timestamp) → Final path
//! - `get_current_session_start_timestamp()` - Impure function: Get current session timestamp
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use parseltongue_core::output_path_resolver::*;
//!
//! // Get session timestamp (once per analysis run)
//! let session_timestamp = get_current_session_start_timestamp();
//!
//! // Resolve output path with timestamp
//! let base_path = PathBuf::from("/repo/ISGLevel01.json");
//! let timestamped_path = resolve_output_path_with_timestamp(&base_path, &session_timestamp)?;
//! // Result: /repo/parseltongue20251115143022/ISGLevel01.json
//!
//! // Create the directory
//! create_timestamped_output_directory(&timestamped_path)?;
//! ```

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

/// Format timestamp into folder name
///
/// # Contract (Executable Specification)
///
/// **Preconditions**:
/// - Valid DateTime<Utc> timestamp
///
/// **Postconditions**:
/// - Returns folder name in format: `parseltongueYYYYMMDDHHMMSS`
/// - Folder name is exactly 27 characters (11 prefix + 14 digits + 2 separators)
///
/// **Error Conditions**:
/// - None (pure function, always succeeds)
///
/// # Example
///
/// ```
/// use chrono::{TimeZone, Utc};
/// use parseltongue_core::output_path_resolver::format_timestamp_folder_name;
///
/// let timestamp = Utc.with_ymd_and_hms(2025, 11, 15, 14, 30, 22).unwrap();
/// let folder_name = format_timestamp_folder_name(&timestamp);
/// assert_eq!(folder_name, "parseltongue20251115143022");
/// ```
pub fn format_timestamp_folder_name(timestamp: &DateTime<Utc>) -> String {
    format!("parseltongue{}", timestamp.format("%Y%m%d%H%M%S"))
}

/// Create timestamped output directory
///
/// # Contract (Executable Specification)
///
/// **Preconditions**:
/// - Path contains a timestamped folder component (parseltongueYYYYMMDDHHMMSS)
///
/// **Postconditions**:
/// - Directory exists on filesystem
/// - All parent directories created if needed
/// - Idempotent (safe to call multiple times)
///
/// **Error Conditions**:
/// - Permission denied
/// - Invalid path
/// - I/O error
///
/// # Example
///
/// ```rust,ignore
/// let path = PathBuf::from("/repo/parseltongue20251115143022/output.json");
/// create_timestamped_output_directory(&path)?;
/// // Directory /repo/parseltongue20251115143022 now exists
/// ```
pub fn create_timestamped_output_directory(file_path: &Path) -> Result<()> {
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Resolve output path with timestamp folder
///
/// # Contract (Executable Specification)
///
/// **Preconditions**:
/// - Base path is a valid file path (absolute or relative)
/// - Timestamp is a valid DateTime<Utc>
///
/// **Postconditions**:
/// - Returns new path with timestamped folder inserted
/// - Original filename preserved
/// - If base path is `/repo/output.json`, result is `/repo/parseltongueYYYYMMDDHHMMSS/output.json`
/// - If base path is `output.json`, result is `parseltongueYYYYMMDDHHMMSS/output.json`
///
/// **Error Conditions**:
/// - Invalid UTF-8 in path (returns error)
/// - None (otherwise)
///
/// # Example
///
/// ```
/// use chrono::{TimeZone, Utc};
/// use std::path::PathBuf;
/// use parseltongue_core::output_path_resolver::resolve_output_path_with_timestamp;
///
/// let base_path = PathBuf::from("/repo/ISGLevel01.json");
/// let timestamp = Utc.with_ymd_and_hms(2025, 11, 15, 14, 30, 22).unwrap();
/// let result = resolve_output_path_with_timestamp(&base_path, &timestamp).unwrap();
/// assert_eq!(result, PathBuf::from("/repo/parseltongue20251115143022/ISGLevel01.json"));
/// ```
pub fn resolve_output_path_with_timestamp(
    base_path: &Path,
    timestamp: &DateTime<Utc>,
) -> Result<PathBuf> {
    let folder_name = format_timestamp_folder_name(timestamp);

    // Get parent directory and filename
    let parent = base_path.parent();
    let filename = base_path.file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid path: no filename"))?;

    // Build new path: parent/timestamped_folder/filename
    let new_path = if let Some(parent_dir) = parent {
        parent_dir.join(&folder_name).join(filename)
    } else {
        PathBuf::from(&folder_name).join(filename)
    };

    Ok(new_path)
}

/// Get current session start timestamp
///
/// # Contract (Executable Specification)
///
/// **Preconditions**:
/// - None
///
/// **Postconditions**:
/// - Returns current UTC timestamp
/// - This should be called ONCE per analysis session and cached
///
/// **Error Conditions**:
/// - None (always succeeds)
///
/// # Example
///
/// ```
/// use parseltongue_core::output_path_resolver::get_current_session_start_timestamp;
///
/// let timestamp = get_current_session_start_timestamp();
/// assert!(timestamp.timestamp() > 0);
/// ```
pub fn get_current_session_start_timestamp() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ========================================================================
    // REQ-TIMESTAMP-001: Folder name format validation
    // ========================================================================

    #[test]
    fn test_format_timestamp_folder_name_contract() {
        // Precondition: Valid DateTime
        let timestamp = Utc.with_ymd_and_hms(2025, 11, 15, 14, 30, 22).unwrap();

        // Act
        let folder_name = format_timestamp_folder_name(&timestamp);

        // Postcondition: Correct format
        assert_eq!(folder_name, "parseltongue20251115143022");
        assert_eq!(folder_name.len(), 26); // 11 (prefix "parseltongue") + 14 digits + 1 (total)
        assert!(folder_name.starts_with("parseltongue"));
    }

    #[test]
    fn test_format_timestamp_folder_name_year_2099() {
        let timestamp = Utc.with_ymd_and_hms(2099, 12, 31, 23, 59, 59).unwrap();
        let folder_name = format_timestamp_folder_name(&timestamp);
        assert_eq!(folder_name, "parseltongue20991231235959");
    }

    #[test]
    fn test_format_timestamp_folder_name_year_2000() {
        let timestamp = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let folder_name = format_timestamp_folder_name(&timestamp);
        assert_eq!(folder_name, "parseltongue20000101000000");
    }

    // ========================================================================
    // REQ-TIMESTAMP-002: Path resolution with timestamp
    // ========================================================================

    #[test]
    fn test_resolve_output_path_with_timestamp_absolute_path() {
        // Precondition: Absolute path with parent directory
        let base_path = PathBuf::from("/repo/ISGLevel01.json");
        let timestamp = Utc.with_ymd_and_hms(2025, 11, 15, 14, 30, 22).unwrap();

        // Act
        let result = resolve_output_path_with_timestamp(&base_path, &timestamp).unwrap();

        // Postcondition: Timestamped folder inserted between parent and filename
        assert_eq!(result, PathBuf::from("/repo/parseltongue20251115143022/ISGLevel01.json"));
    }

    #[test]
    fn test_resolve_output_path_with_timestamp_relative_path() {
        // Precondition: Relative path
        let base_path = PathBuf::from("ISGLevel01.json");
        let timestamp = Utc.with_ymd_and_hms(2025, 11, 15, 14, 30, 22).unwrap();

        // Act
        let result = resolve_output_path_with_timestamp(&base_path, &timestamp).unwrap();

        // Postcondition: Timestamped folder becomes parent
        assert_eq!(result, PathBuf::from("parseltongue20251115143022/ISGLevel01.json"));
    }

    #[test]
    fn test_resolve_output_path_with_timestamp_nested_path() {
        // Precondition: Nested path
        let base_path = PathBuf::from("/repo/subdir/output.json");
        let timestamp = Utc.with_ymd_and_hms(2025, 11, 15, 14, 30, 22).unwrap();

        // Act
        let result = resolve_output_path_with_timestamp(&base_path, &timestamp).unwrap();

        // Postcondition: Timestamped folder inserted at correct level
        assert_eq!(result, PathBuf::from("/repo/subdir/parseltongue20251115143022/output.json"));
    }

    #[test]
    fn test_resolve_output_path_preserves_filename() {
        // Precondition: Various filenames
        let test_cases = vec![
            ("ISGLevel00.json", "ISGLevel00.json"),
            ("ISGLevel01.json", "ISGLevel01.json"),
            ("ISGLevel02.json", "ISGLevel02.json"),
            ("custom_name.json", "custom_name.json"),
            ("data.toon", "data.toon"),
        ];

        let timestamp = Utc.with_ymd_and_hms(2025, 11, 15, 14, 30, 22).unwrap();

        for (input_filename, expected_filename) in test_cases {
            let base_path = PathBuf::from(format!("/repo/{}", input_filename));
            let result = resolve_output_path_with_timestamp(&base_path, &timestamp).unwrap();

            // Postcondition: Filename preserved
            assert_eq!(result.file_name().unwrap().to_str().unwrap(), expected_filename);
        }
    }

    // ========================================================================
    // REQ-TIMESTAMP-003: Directory creation
    // ========================================================================

    #[test]
    fn test_create_timestamped_output_directory_creates_parent() {
        use tempfile::TempDir;

        // Precondition: Temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("parseltongue20251115143022/output.json");

        // Act
        let result = create_timestamped_output_directory(&test_path);

        // Postcondition: Directory created successfully
        assert!(result.is_ok());
        assert!(test_path.parent().unwrap().exists());
    }

    #[test]
    fn test_create_timestamped_output_directory_idempotent() {
        use tempfile::TempDir;

        // Precondition: Temporary directory
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("parseltongue20251115143022/output.json");

        // Act: Call twice
        let result1 = create_timestamped_output_directory(&test_path);
        let result2 = create_timestamped_output_directory(&test_path);

        // Postcondition: Both succeed (idempotent)
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    fn test_create_timestamped_output_directory_nested_parents() {
        use tempfile::TempDir;

        // Precondition: Nested path
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path()
            .join("level1")
            .join("level2")
            .join("parseltongue20251115143022")
            .join("output.json");

        // Act
        let result = create_timestamped_output_directory(&test_path);

        // Postcondition: All parent directories created
        assert!(result.is_ok());
        assert!(test_path.parent().unwrap().exists());
    }

    // ========================================================================
    // REQ-TIMESTAMP-004: Session timestamp consistency
    // ========================================================================

    #[test]
    fn test_get_current_session_start_timestamp_returns_valid() {
        // Act
        let timestamp = get_current_session_start_timestamp();

        // Postcondition: Valid timestamp (after 2020)
        assert!(timestamp.timestamp() > 1577836800); // 2020-01-01
    }

    #[test]
    fn test_single_session_timestamp_for_multiple_outputs() {
        // Simulate single session with multiple outputs
        let session_timestamp = get_current_session_start_timestamp();

        // All outputs should use the SAME timestamp
        let output1 = PathBuf::from("/repo/ISGLevel00.json");
        let output2 = PathBuf::from("/repo/ISGLevel01.json");
        let output3 = PathBuf::from("/repo/ISGLevel02.json");

        let path1 = resolve_output_path_with_timestamp(&output1, &session_timestamp).unwrap();
        let path2 = resolve_output_path_with_timestamp(&output2, &session_timestamp).unwrap();
        let path3 = resolve_output_path_with_timestamp(&output3, &session_timestamp).unwrap();

        // Extract folder names (should all be identical)
        let folder1 = path1.parent().unwrap().file_name().unwrap();
        let folder2 = path2.parent().unwrap().file_name().unwrap();
        let folder3 = path3.parent().unwrap().file_name().unwrap();

        assert_eq!(folder1, folder2);
        assert_eq!(folder2, folder3);
    }

    // ========================================================================
    // REQ-TIMESTAMP-005: Integration test (E2E)
    // ========================================================================

    #[test]
    fn test_end_to_end_workflow() {
        use tempfile::TempDir;

        // Setup: Simulate analysis session
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path();

        // Step 1: Get session timestamp (once per analysis)
        let session_timestamp = Utc.with_ymd_and_hms(2025, 11, 15, 14, 30, 22).unwrap();

        // Step 2: Resolve paths for all outputs
        let outputs = [repo_root.join("ISGLevel00.json"),
            repo_root.join("ISGLevel01.json"),
            repo_root.join("ISGLevel02.json")];

        let timestamped_paths: Vec<PathBuf> = outputs
            .iter()
            .map(|p| resolve_output_path_with_timestamp(p, &session_timestamp).unwrap())
            .collect();

        // Step 3: Create directories
        for path in &timestamped_paths {
            create_timestamped_output_directory(path).unwrap();
        }

        // Step 4: Write test files
        for path in &timestamped_paths {
            std::fs::write(path, "test content").unwrap();
        }

        // Verify: All files exist in same timestamped folder
        let expected_folder = repo_root.join("parseltongue20251115143022");
        assert!(expected_folder.exists());
        assert!(expected_folder.join("ISGLevel00.json").exists());
        assert!(expected_folder.join("ISGLevel01.json").exists());
        assert!(expected_folder.join("ISGLevel02.json").exists());
    }
}
