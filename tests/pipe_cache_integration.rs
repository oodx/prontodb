//! Integration tests for revolutionary pipe cache system
//! Tests zero data loss guarantees and recovery workflows

use std::process::{Command, Stdio};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper to run prontodb with piped input
fn run_with_pipe(args: &[&str], input: &str) -> (String, String, bool) {
    let mut child = Command::new("./target/debug/prontodb")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn prontodb");
    
    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }
    
    let output = child.wait_with_output().expect("Failed to read output");
    
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success()
    )
}

/// Helper to run prontodb without pipe
fn run_command(args: &[&str]) -> (String, String, bool) {
    let output = Command::new("./target/debug/prontodb")
        .args(args)
        .output()
        .expect("Failed to execute prontodb");
    
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success()
    )
}

#[test]
fn test_pipe_cache_invalid_address() {
    // Test that invalid addresses with piped content get cached
    let test_content = "This is important data that must not be lost!";
    let (stdout, stderr, success) = run_with_pipe(
        &["set", "invalid...address"],
        test_content
    );
    
    // Should fail but with helpful cache message
    assert!(!success, "Command should fail for invalid address");
    assert!(stderr.contains("Invalid address"), "Should report invalid address");
    assert!(stderr.contains("pipe.cache."), "Should mention cache key");
    assert!(stderr.contains("content cached"), "Should confirm caching");
    
    // Extract cache key from error message - it's after "cached as: "
    let cache_key = stderr
        .lines()
        .find(|line| line.contains("cached as:"))
        .and_then(|line| {
            line.split("cached as:").nth(1)
        })
        .map(|s| s.trim())
        .expect("Should find cache key in error message");
    
    // Verify we can retrieve the cached content
    let (stdout, _, success) = run_command(&["get", cache_key]);
    assert!(success, "Should successfully retrieve cached content");
    assert_eq!(stdout.trim(), test_content, "Cached content should match original");
}

#[test]
fn test_pipe_cache_with_valid_address() {
    // Test that valid addresses still need value parameter
    // The piped content is ignored for valid addresses
    let (_, stderr, success) = run_with_pipe(
        &["set", "test.pipe.normal"],
        "ignored content"
    );
    
    assert!(!success, "Should fail without value for valid address");
    assert!(stderr.contains("Usage:"), "Should show usage for valid address");
    
    // Now test with proper value parameter (no pipe)
    let (_, _, success) = run_command(&["set", "test.pipe.normal", "stored value"]);
    assert!(success, "Should succeed with proper parameters");
    
    // Verify stored normally
    let (stdout, _, success) = run_command(&["get", "test.pipe.normal"]);
    assert!(success, "Should retrieve normally stored content");
    assert_eq!(stdout.trim(), "stored value");
}

#[test]
fn test_pipe_cache_empty_input() {
    // Test that empty pipe input doesn't create cache
    let (_, stderr, success) = run_with_pipe(
        &["set", "invalid.address"],
        ""
    );
    
    assert!(!success, "Should fail for invalid address");
    assert!(stderr.contains("Invalid address"));
    assert!(!stderr.contains("pipe.cache."), "Should not cache empty input");
}

#[test]
fn test_pipe_cache_unique_keys() {
    // Test that multiple failures generate unique cache keys
    let content1 = "First failure content";
    let content2 = "Second failure content";
    
    let (_, stderr1, _) = run_with_pipe(
        &["set", "bad.addr1"],
        content1
    );
    
    let (_, stderr2, _) = run_with_pipe(
        &["set", "bad.addr2"],
        content2
    );
    
    // Extract cache keys
    let extract_key = |stderr: &str| -> String {
        stderr
            .lines()
            .find(|line| line.contains("cached as:"))
            .and_then(|line| {
                line.split("cached as:").nth(1)
            })
            .map(|s| s.trim())
            .unwrap_or("")
            .to_string()
    };
    
    let key1 = extract_key(&stderr1);
    let key2 = extract_key(&stderr2);
    
    assert!(!key1.is_empty(), "Should have first cache key");
    assert!(!key2.is_empty(), "Should have second cache key");
    assert_ne!(key1, key2, "Cache keys should be unique");
    
    // Verify both are retrievable
    let (stdout1, _, _) = run_command(&["get", &key1]);
    let (stdout2, _, _) = run_command(&["get", &key2]);
    
    assert_eq!(stdout1.trim(), content1);
    assert_eq!(stdout2.trim(), content2);
}

#[test]
fn test_pipe_cache_special_characters() {
    // Test caching with addresses containing special characters
    let test_content = "Content with special chars: !@#$%^&*()";
    let (_, stderr, _) = run_with_pipe(
        &["set", "bad/address\\with..special///chars"],
        test_content
    );
    
    assert!(stderr.contains("pipe.cache."), "Should cache despite special chars");
    
    // Extract and verify
    let cache_key = stderr
        .lines()
        .find(|line| line.contains("cached as:"))
        .and_then(|line| {
            line.split("cached as:").nth(1)
        })
        .map(|s| s.trim())
        .unwrap();
    
    let (stdout, _, success) = run_command(&["get", cache_key]);
    assert!(success);
    assert_eq!(stdout.trim(), test_content);
}

#[test]
fn test_pipe_cache_large_content() {
    // Test caching large content
    let large_content = "x".repeat(10000); // 10KB of data
    let (_, stderr, _) = run_with_pipe(
        &["set", "invalid.large"],
        &large_content
    );
    
    assert!(stderr.contains("pipe.cache."), "Should cache large content");
    
    let cache_key = stderr
        .lines()
        .find(|line| line.contains("cached as:"))
        .and_then(|line| {
            line.split("cached as:").nth(1)
        })
        .map(|s| s.trim())
        .unwrap();
    
    let (stdout, _, success) = run_command(&["get", cache_key]);
    assert!(success, "Should retrieve large cached content");
    assert_eq!(stdout.trim(), large_content);
}

#[test]
fn test_pipe_cache_multiline_content() {
    // Test caching multiline content
    let multiline = "Line 1\nLine 2\nLine 3\nWith special: 日本語";
    let (_, stderr, _) = run_with_pipe(
        &["set", "bad.multiline"],
        multiline
    );
    
    assert!(stderr.contains("pipe.cache."));
    
    let cache_key = stderr
        .lines()
        .find(|line| line.contains("cached as:"))
        .and_then(|line| {
            line.split("cached as:").nth(1)
        })
        .map(|s| s.trim())
        .unwrap();
    
    let (stdout, _, _) = run_command(&["get", cache_key]);
    assert_eq!(stdout.trim(), multiline);
}

#[test]
fn test_pipe_cache_key_format() {
    // Test that cache keys follow expected format
    let test_content = "Test for key format";
    let (_, stderr, _) = run_with_pipe(
        &["set", "invalid.test"],
        test_content
    );
    
    let cache_key = stderr
        .lines()
        .find(|line| line.contains("cached as:"))
        .and_then(|line| {
            line.split("cached as:").nth(1)
        })
        .map(|s| s.trim())
        .unwrap();
    
    // Verify format: pipe.cache.{timestamp}_{hash}_{sanitized_address}
    assert!(cache_key.starts_with("pipe.cache."));
    let parts: Vec<&str> = cache_key["pipe.cache.".len()..].split('_').collect();
    assert_eq!(parts.len(), 3, "Should have timestamp_hash_address format");
    
    // Verify timestamp is numeric
    assert!(parts[0].parse::<u64>().is_ok(), "First part should be timestamp");
    
    // Verify hash is 8 chars
    assert_eq!(parts[1].len(), 8, "Hash should be 8 characters");
    
    // Verify address is sanitized
    assert!(parts[2].contains("invalid"), "Should contain original address parts");
}