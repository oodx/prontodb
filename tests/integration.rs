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