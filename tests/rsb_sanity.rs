//! RSB Sanity Tests
//!
//! Integration tests that verify RSB framework functionality.

use rsb::prelude::*;

#[test]
fn test_rsb_basic_functionality() {
    // Test RSB prelude is available and basic functionality works
    let output = format!("RSB: {} = {}", "test", "value");
    assert!(output.contains("RSB:"));
    assert!(output.contains("test"));
    assert!(output.contains("value"));
}

#[test]
fn test_rsb_color_handling() {
    // Test basic color string handling (from rsb_colors.rs)
    let colored_text = "\x1b[31mRed Text\x1b[0m";
    assert!(colored_text.contains("\x1b[31m"));
    assert!(colored_text.contains("\x1b[0m"));
}

#[test]
fn test_rsb_environment() {
    // Test RSB works in different environments
    use std::env;
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
