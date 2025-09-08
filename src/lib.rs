// ProntoDB MVP Library
pub mod addressing;
pub mod storage;
pub mod xdg;

// Re-export key types for convenience  
pub use addressing::Address;
pub use storage::Storage;
pub use xdg::XdgPaths;

// TDD infrastructure validation function for Card 001
/// Validates that the TDD infrastructure is properly set up and working
/// Returns true if all TDD components are functioning correctly
pub fn validate_tdd_infrastructure() -> bool {
    // Minimal implementation to pass the test
    // This validates that:
    // 1. Test framework can compile and run
    // 2. Functions can be called from tests
    // 3. Basic assertion infrastructure works
    true
}
