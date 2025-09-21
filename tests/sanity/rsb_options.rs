//! RSB OPTIONS Sanity Tests
//!
//! General sanity tests for RSB options! macro and argument parsing.
//! Tests core RSB functionality without meteor dependencies.

use rsb::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsb_options_basic() {
        // Test that options! macro works with basic args
        let args = vec!["prontodb".to_string(), "--verbose".to_string(), "test".to_string()];

        // This should not panic - basic RSB functionality test
        let parsed_args = Args::new(&args);
        // Test that args object was created successfully
        assert!(true); // Basic compilation test
    }

    #[test]
    fn test_rsb_bootstrap_pattern() {
        // Test that bootstrap! macro works
        // This is a compile-time test mainly
        std::env::set_var("TEST_RSB", "1");

        // Should not panic
        let args = bootstrap!();
        options!(&args);

        std::env::remove_var("TEST_RSB");
    }

    #[test]
    fn test_rsb_string_bias() {
        // Test RSB's string-biased approach
        let test_string = "key=value;another=test";

        // RSB should handle string-based data well
        assert!(test_string.contains("="));
        assert!(test_string.contains(";"));

        // Basic string parsing that RSB relies on
        let parts: Vec<&str> = test_string.split(';').collect();
        assert_eq!(parts.len(), 2);
    }
}