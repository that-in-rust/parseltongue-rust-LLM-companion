//! Smart port selection with retry logic
//!
//! # 4-Word Naming: port_selection
//!
//! This module provides intelligent port selection that:
//! - Treats user-specified ports as preferences, not requirements
//! - Automatically tries next available port on conflict
//! - Eliminates race conditions by holding the listener
//! - Provides clear feedback for each port attempt
//!
//! ## Architecture Layers
//!
//! - **L1 Core**: `ValidatedPortNumber` newtype for type safety
//! - **L2 Standard**: `PortRangeIterator` for port enumeration
//! - **L3 External**: `tokio::net::TcpListener` for async binding
//!
//! ## Usage
//!
//! ```rust,no_run
//! use pt08_http_code_query_server::port_selection::find_and_bind_port_available;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // User specified preference, but system adapts if occupied
//! let listener = find_and_bind_port_available(Some(7777), 100).await?;
//! let actual_port = listener.local_addr()?.port();
//! println!("Running on http://localhost:{}", actual_port);
//! # Ok(())
//! # }
//! ```

use std::io::ErrorKind;
use thiserror::Error;
use tokio::net::TcpListener;

// ============================================================================
// L1 CORE: Type-Safe Port Wrapper
// ============================================================================

/// Validated port number (1-65535, with privilege checking)
///
/// # Four-Word Name: ValidatedPortNumber
///
/// # Preconditions
/// - Port value is between 1024 and 65535 (non-privileged)
/// - Port value is not zero
///
/// # Postconditions
/// - Contains a valid, non-privileged port number
/// - Can be safely used for TCP binding without root
///
/// # Error Conditions
/// - PortValidationError::PrivilegedPort if < 1024
/// - PortValidationError::ZeroPort if == 0
/// - PortValidationError::OutOfRange if > 65535
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ValidatedPortNumber {
    value: u16,
}

impl ValidatedPortNumber {
    /// Create a new validated port number
    ///
    /// # Four-Word Name: create_validated_port_number
    ///
    /// # Test Contracts
    /// ## Scenario 1: Valid Non-Privileged Port
    /// Given: Port 8080
    /// When: create_validated_port_number(8080) is called
    /// Then: Returns Ok(ValidatedPortNumber)
    ///
    /// # Scenario 2: Privileged Port Rejected
    /// Given: Port 80 (< 1024)
    /// When: create_validated_port_number(80) is called
    /// Then: Returns Err(PortValidationError::PrivilegedPort)
    ///
    /// # Scenario 3: Zero Port Rejected
    /// Given: Port 0
    /// When: create_validated_port_number(0) is called
    /// Then: Returns Err(PortValidationError::ZeroPort)
    pub fn new(port: u16) -> Result<Self, PortValidationError> {
        if port == 0 {
            return Err(PortValidationError::ZeroPort);
        }
        if port < 1024 {
            return Err(PortValidationError::PrivilegedPort { port });
        }
        Ok(Self { value: port })
    }

    /// Get the underlying port value
    ///
    /// # Four-Word Name: get_underlying_port_value
    pub fn value(&self) -> u16 {
        self.value
    }
}

impl From<ValidatedPortNumber> for u16 {
    fn from(port: ValidatedPortNumber) -> Self {
        port.value
    }
}

// ============================================================================
// L1 CORE: Port Validation Error
// ============================================================================

/// Port validation error types
///
/// # Four-Word Name: PortValidationError
#[derive(Error, Debug, PartialEq, Eq)]
pub enum PortValidationError {
    /// Port zero is invalid
    #[error("Port zero is not a valid TCP port")]
    ZeroPort,

    /// Port requires root privileges (< 1024)
    #[error("Port {port} requires root privileges (use port >= 1024)")]
    PrivilegedPort { port: u16 },

    /// Port out of valid range
    #[error("Port {port} is out of valid range (1-65535)")]
    OutOfRange { port: u16 },
}

// ============================================================================
// L2 STANDARD: Port Range Iterator
// ============================================================================

/// Iterator over a range of port numbers with validation
///
/// # Four-Word Name: PortRangeIterator
///
/// # Preconditions
/// - Start port is valid (>= 1024, != 0)
/// - End port > start port
///
/// # Postconditions
/// - Yields validated port numbers
/// - Stops at end (exclusive)
///
/// # Test Contracts
/// ## Scenario 1: Iterate Over Valid Range
/// Given: Range 7777..7780
/// When: Iterator is collected
/// Then: Yields [7777, 7778, 7779]
pub struct PortRangeIterator {
    current: u16,
    end: u16,
}

impl PortRangeIterator {
    /// Create a new port range iterator
    ///
    /// # Four-Word Name: create_port_range_iterator
    pub fn new(start: u16, end: u16) -> Self {
        Self {
            current: start,
            end,
        }
    }
}

impl Iterator for PortRangeIterator {
    type Item = ValidatedPortNumber;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.end {
            let port = self.current;
            self.current += 1;
            // Skip invalid ports by continuing the loop
            if let Ok(validated) = ValidatedPortNumber::new(port) {
                return Some(validated);
            }
            // Continue loop to try next port
        }
        None
    }
}

// ============================================================================
// L3 EXTERNAL: Port Selection Error
// ============================================================================

/// Port selection failure modes
///
/// # Four-Word Name: PortSelectionError
///
/// # Error Conditions
/// - RangeExhausted: All ports in range are occupied
/// - PermissionDenied: Binding failed due to permissions
/// - SystemError: OS-level bind failure
#[derive(Error, Debug)]
pub enum PortSelectionError {
    /// No available port in the requested range
    #[error("No available port in range {start}-{end}. Try using a different starting port.")]
    RangeExhausted { start: u16, end: u16 },

    /// Permission denied for port binding
    #[error("Permission denied for port {port}: {cause}")]
    PermissionDenied { port: u16, cause: String },

    /// System-level bind failure
    #[error("System error binding to port {port}: {cause}")]
    SystemError { port: u16, cause: String },
}

// ============================================================================
// L3 EXTERNAL: Smart Port Selector (Main API)
// ============================================================================

/// Find and bind to an available port with retry logic
///
/// # Four-Word Name: find_and_bind_port_available
///
/// This function eliminates the race condition present in the old
/// `find_available_port_number` function by holding the TcpListener
/// open and returning it directly to the caller.
///
/// # Preconditions
/// - preferred_port is None or a valid port number
/// - max_attempts is between 1 and 1000
/// - At least one port in the range should be available
///
/// # Postconditions
/// - Returns Ok(TcpListener) bound to an available port
/// - The listener is held open (not dropped)
/// - Port number is within [preferred, preferred + max_attempts)
/// - Progress is logged to stderr for each attempt
///
/// # Error Conditions
/// - PortSelectionError::RangeExhausted if no port available in range
/// - PortSelectionError::PermissionDenied if binding fails due to permissions
/// - PortSelectionError::SystemError for other OS-level failures
///
/// # Test Contracts
/// ## Scenario 1: REQ-PORT-001.0 - First Port Available
/// Given: Port 7777 is available
/// When: find_and_bind_port_available(Some(7777), 100) is called
/// Then: Returns Ok(listener) bound to 7777
/// And: Logs "Trying 7777... ✓"
///
/// ## Scenario 2: REQ-PORT-002.0 - Port Busy, Next Available
/// Given: Port 7777 is occupied, 7778 is available
/// When: find_and_bind_port_available(Some(7777), 100) is called
/// Then: Returns Ok(listener) bound to 7778
/// And: Logs "Trying 7777... in use, trying next..."
/// And: Logs "Trying 7778... ✓"
///
/// ## Scenario 3: REQ-PORT-003.0 - No Race Condition
/// Given: A port that becomes available during search
/// When: find_and_bind_port_available succeeds
/// Then: The TcpListener is actually bound (check-and-bind are atomic)
///
/// ## Scenario 4: REQ-PORT-005.0 - Range Exhaustion
/// Given: All ports in range are occupied
/// When: find_and_bind_port_available is called
/// Then: Returns Err(PortSelectionError::RangeExhausted)
pub async fn find_and_bind_port_available(
    preferred_port_option: Option<u16>,
    max_attempts_count: u16,
) -> Result<TcpListener, PortSelectionError> {
    // REQ-PORT-001.0: Default to 7777 if no preference
    let start = preferred_port_option.unwrap_or(7777);

    // Validate range
    if max_attempts_count == 0 {
        return Err(PortSelectionError::RangeExhausted {
            start,
            end: start,
        });
    }

    // REQ-PORT-004.0: Try each port in range, logging progress
    for port in start..start + max_attempts_count {
        // REQ-PORT-004.0: Log attempt to stderr
        eprint!("Trying {}...", port);

        // REQ-PORT-003.0: Atomic bind operation (no check-then-bind gap)
        match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(listener) => {
                // REQ-PORT-004.0: Log success
                eprintln!(" ✓");
                return Ok(listener);
            }
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                // REQ-PORT-002.0: Port is occupied, try next
                // REQ-PORT-004.0: Log that port is in use
                eprintln!(" in use, trying next...");
                continue;
            }
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                return Err(PortSelectionError::PermissionDenied {
                    port,
                    cause: e.to_string(),
                });
            }
            Err(e) => {
                return Err(PortSelectionError::SystemError {
                    port,
                    cause: e.to_string(),
                });
            }
        }
    }

    // REQ-PORT-005.0: Range exhausted
    Err(PortSelectionError::RangeExhausted {
        start,
        end: start + max_attempts_count,
    })
}

// ============================================================================
// TESTS: Unit tests for port selection
// ============================================================================

#[cfg(test)]
mod port_selection_unit_tests {
    use super::*;

    // -------------------------------------------------------------------------
    // L1 Core: ValidatedPortNumber Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_create_valid_port_number_success() {
        // Test contract: Valid non-privileged port succeeds
        let result = ValidatedPortNumber::new(8080);
        assert!(result.is_ok());
        let port = result.unwrap();
        assert_eq!(port.value(), 8080);
    }

    #[test]
    fn test_reject_privileged_port_80() {
        // Test contract: Privileged port (< 1024) is rejected
        let result = ValidatedPortNumber::new(80);
        assert_eq!(result, Err(PortValidationError::PrivilegedPort { port: 80 }));
    }

    #[test]
    fn test_reject_privileged_port_1023() {
        // Test contract: Port 1023 is still privileged
        let result = ValidatedPortNumber::new(1023);
        assert!(result.is_err());
    }

    #[test]
    fn test_accept_minimum_non_privileged_port() {
        // Test contract: Port 1024 is minimum non-privileged
        let result = ValidatedPortNumber::new(1024);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), 1024);
    }

    #[test]
    fn test_reject_zero_port() {
        // Test contract: Port zero is rejected
        let result = ValidatedPortNumber::new(0);
        assert_eq!(result, Err(PortValidationError::ZeroPort));
    }

    // -------------------------------------------------------------------------
    // L2 Standard: PortRangeIterator Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_port_range_iterator_yields_valid_ports() {
        // Test contract: Iterator yields validated ports
        let iterator = PortRangeIterator::new(7777, 7780);
        let ports: Vec<u16> = iterator.map(|p| p.value()).collect();
        assert_eq!(ports, vec![7777, 7778, 7779]);
    }

    #[test]
    fn test_port_range_iterator_empty_range() {
        // Test contract: Empty range yields nothing
        let iterator = PortRangeIterator::new(7777, 7777);
        let count = iterator.count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_port_range_iterator_skips_invalid_ports() {
        // Test contract: Iterator skips ports < 1024
        let iterator = PortRangeIterator::new(1022, 1026);
        let ports: Vec<u16> = iterator.map(|p| p.value()).collect();
        // 1022, 1023 are privileged (skipped), 1024, 1025 are valid
        assert_eq!(ports, vec![1024, 1025]);
    }
}

// ============================================================================
// TESTS: Integration tests for find_and_bind_port_available
// ============================================================================

#[cfg(test)]
mod port_selection_integration_tests {
    use super::*;

    // -------------------------------------------------------------------------
    // REQ-PORT-001.0: First Port Available
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_req_port_001_first_port_available() {
        // Test contract: When port 7777 is available, binds to 7777
        let result = find_and_bind_port_available(Some(9999), 10).await;
        assert!(result.is_ok(), "Should find available port");
        let listener = result.unwrap();
        let port = listener.local_addr().unwrap().port();
        assert_eq!(port, 9999, "Should bind to first available port");
    }

    #[tokio::test]
    async fn test_req_port_001_default_port_7777() {
        // Test contract: When no preference, defaults to 7777
        let result = find_and_bind_port_available(None, 10).await;
        assert!(result.is_ok());
        let listener = result.unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(port >= 7777, "Should use default 7777 or higher");
    }

    // -------------------------------------------------------------------------
    // REQ-PORT-002.0: Port Preference with Fallback
    // -------------------------------------------------------------------------


    // -------------------------------------------------------------------------
    // REQ-PORT-003.0: No Race Condition
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_req_port_003_no_race_condition() {
        // Test contract: Listener is actually bound (no check-then-bind gap)
        let result = find_and_bind_port_available(Some(9995), 10).await;
        assert!(result.is_ok());

        let listener = result.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Verify the listener is actually bound by trying to bind again
        let duplicate_attempt = TcpListener::bind(format!("0.0.0.0:{}", port)).await;
        assert!(duplicate_attempt.is_err(), "Port should actually be bound");
    }

    // -------------------------------------------------------------------------
    // REQ-PORT-005.0: Range Exhaustion
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_req_port_005_range_exhaustion() {
        // Test contract: Returns RangeExhausted when all ports occupied
        // Setup: Occupy all ports in a small range
        let mut guards = Vec::new();
        for p in 9500..9505 {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", p)).await.unwrap();
            guards.push(listener);
        }

        let result = find_and_bind_port_available(Some(9500), 5).await;
        assert!(matches!(result, Err(PortSelectionError::RangeExhausted { start: 9500, end: 9505 })));
    }

    #[tokio::test]
    async fn test_req_port_005_zero_attempts_returns_error() {
        // Test contract: Zero max_attempts returns error immediately
        let result = find_and_bind_port_available(Some(7777), 0).await;
        assert!(matches!(result, Err(PortSelectionError::RangeExhausted { start: 7777, end: 7777 })));
    }

    // -------------------------------------------------------------------------
    // Additional: Error Handling
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_privileged_port_error() {
        // Test contract: Privileged ports return PermissionDenied error
        // Note: This test may fail if running as root
        let result = find_and_bind_port_available(Some(80), 1).await;

        // Should either succeed (if root) or return PermissionDenied
        if let Err(e) = result {
            assert!(matches!(e, PortSelectionError::PermissionDenied { .. }));
        }
    }
}
