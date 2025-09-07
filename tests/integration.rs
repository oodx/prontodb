// Integration tests based on TEST-SPEC.md
// Following TDD protocol: RED -> GREEN -> REFACTOR -> COMMIT

use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::io::Write;

fn bin() -> String {
    env::var("PRONTODB_BIN").unwrap_or_else(|_| "./target/debug/prontodb".to_string())
}

fn run(home: &str, args: &[&str]) -> (i32, String, String) {
    let output = Command::new(bin())
        .args(args)
        .env("HOME", home)
        .output()
        .unwrap();
    
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string()
    )
}

fn test_home(tag: &str) -> String {
    let home = format!(
        "{}/.prontodb_test_{}_{}",
        std::env::temp_dir().display(),
        tag,
        std::process::id()
    );
    fs::create_dir_all(&home).unwrap();
    home
}

// =============================================================================
// RED PHASE: First failing tests from TEST-SPEC.md
// =============================================================================

#[test]
fn test_rsb_framework_available_and_builds() {
    // TEST-SPEC 0.0: RSB framework is available and builds without test [v0.1]
    // This test verifies the basic RSB integration works
    let home = test_home("rsb_basic");
    
    // Just running help should work if RSB bootstrap and dispatch work
    let (code, _stdout, stderr) = run(&home, &["--help"]);
    // Should not crash with RSB framework errors
    assert_ne!(stderr.contains("RSB"), true, "Should not have RSB framework errors");
    
    // Any command should at least dispatch without RSB failures
    let (code, _stdout, stderr) = run(&home, &["keys"]);
    assert_ne!(stderr.contains("bootstrap"), true, "RSB bootstrap should work");
    assert_ne!(stderr.contains("dispatch"), true, "RSB dispatch should work");
}

#[test]
fn test_install_creates_system_tables() {
    // TEST-SPEC 0.1: install seeds system tables and default admin [v0.1]
    let home = test_home("install");
    
    let (code, _stdout, stderr) = run(&home, &["install"]);
    
    // This will fail until we implement RSB bootstrap
    assert_eq!(code, 0, "Install failed: {}", stderr);
}

#[test]
fn test_canonical_addressing() {
    // TEST-SPEC 1.1: canonical addressing `project.namespace.key__ctx` [v0.1]
    let home = test_home("addressing");
    run(&home, &["install"]);
    
    run(&home, &["set", "kb.recipes.pasta__italian", "marinara"]);
    let (code, stdout, stderr) = run(&home, &["get", "kb.recipes.pasta__italian"]);
    
    assert_eq!(code, 0, "Get failed: {}", stderr);
    assert_eq!(stdout.trim(), "marinara");
}

#[test]
fn test_set_get_basic_string() {
    // TEST-SPEC 2.1: set/get basic string [v0.1]
    let home = test_home("basic_kv");
    run(&home, &["install"]);
    
    let (code, _stdout, stderr) = run(&home, &["set", "test.basic.key", "value"]);
    assert_eq!(code, 0, "Set failed: {}", stderr);
    
    let (code, stdout, stderr) = run(&home, &["get", "test.basic.key"]);
    assert_eq!(code, 0, "Get failed: {}", stderr);
    assert_eq!(stdout.trim(), "value");
}

#[test]
fn test_delete_removes_key() {
    // TEST-SPEC 2.3: delete removes key [v0.1]
    let home = test_home("delete");
    run(&home, &["install"]);
    
    run(&home, &["set", "test.delete.key", "value"]);
    let (code, _stdout, stderr) = run(&home, &["del", "test.delete.key"]);
    assert_eq!(code, 0, "Delete failed: {}", stderr);
    
    // After `del`, `get` returns MISS (exit 2, empty stdout)
    let (code, stdout, _stderr) = run(&home, &["get", "test.delete.key"]);
    assert_eq!(code, 2, "Should return exit code 2 for missing key");
    assert_eq!(stdout.trim(), "", "Should have empty stdout for miss");
}

#[test]
fn test_flag_addressing_project_namespace() {
    // TEST-SPEC 1.2: flag addressing `-p/-n` [v0.1] 
    let home = test_home("flag_addressing");
    run(&home, &["install"]);
    
    // Set value using flag addressing
    let (code, _stdout, stderr) = run(&home, &["set", "-p", "myproject", "-n", "config", "debug_level", "verbose"]);
    assert_eq!(code, 0, "Set with flags failed: {}", stderr);
    
    // Get value using flag addressing
    let (code, stdout, stderr) = run(&home, &["get", "-p", "myproject", "-n", "config", "debug_level"]);
    assert_eq!(code, 0, "Get with flags failed: {}", stderr);
    assert_eq!(stdout.trim(), "verbose", "Flag addressing should return set value");
    
    // Verify canonical addressing also works for the same key
    let (code, stdout, stderr) = run(&home, &["get", "myproject.config.debug_level"]);
    assert_eq!(code, 0, "Canonical get failed: {}", stderr);
    assert_eq!(stdout.trim(), "verbose", "Canonical addressing should return same value");
}