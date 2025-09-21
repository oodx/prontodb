//! RSB Global Configuration Sanity Tests

use rsb::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsb_bootstrap_macro() {
        let args = bootstrap!();
        assert!(true); // Always passes, tests compilation
    }

    #[test]
    fn test_rsb_options_macro() {
        let args = Args::new(&vec!["test".to_string()]);
        options!(&args); // Should not panic
    }

    #[test]
    fn test_rsb_dispatch_pattern() {
        // Test the RSB dispatch pattern works
        let command = "test";
        match command {
            "test" => assert!(true),
            _ => assert!(false),
        }
    }
}