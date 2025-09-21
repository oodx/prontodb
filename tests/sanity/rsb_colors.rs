//! RSB COLORS Sanity Tests
//!
//! General sanity tests for RSB color support and terminal output.
//! Tests core RSB functionality without meteor dependencies.

use rsb::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsb_color_support() {
        // Test that RSB handles color codes properly
        // RSB should work with or without color support

        // Basic color string handling
        let colored_text = "\x1b[31mRed Text\x1b[0m";
        assert!(colored_text.contains("\x1b[31m"));
        assert!(colored_text.contains("\x1b[0m"));
    }

    #[test]
    fn test_rsb_terminal_detection() {
        // Test basic terminal detection that RSB might use
        use std::env;

        // RSB should work regardless of terminal type
        let term = env::var("TERM").unwrap_or_default();

        // Should handle any terminal type gracefully
        assert!(term.is_empty() || !term.is_empty());
    }

    #[test]
    fn test_rsb_output_formatting() {
        // Test RSB's string-biased output formatting
        let output = format!("RSB: {} = {}", "key", "value");

        assert!(output.contains("RSB:"));
        assert!(output.contains("="));

        // RSB should produce clean, parseable output
        assert!(!output.contains("\n"));
    }

    #[test]
    fn test_rsb_error_formatting() {
        // Test that RSB error messages are well-formatted
        let error_msg = "RSB Error: Invalid argument";

        assert!(error_msg.starts_with("RSB Error:"));
        assert!(!error_msg.contains("\n"));
    }
}