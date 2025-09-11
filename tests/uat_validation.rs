// UAT validation test to ensure the updated UAT script works correctly
// Tests key meta namespace commands used in the UAT ceremony

use prontodb::{CursorManager, Storage, XdgPaths};
use std::process::Command;
use tempfile::TempDir;

struct UATTestEnvironment {
    temp_dir: TempDir,
    binary_path: String,
}

impl UATTestEnvironment {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        
        // Get the current directory and construct absolute path to binary
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let binary_path = current_dir.join("target/debug/prontodb");
        
        // Ensure binary exists
        let output = Command::new("cargo")
            .args(&["build"])
            .current_dir(&current_dir)
            .output()
            .expect("Failed to build binary");
        
        if !output.status.success() {
            panic!("Failed to build binary for UAT validation");
        }
        
        if !binary_path.exists() {
            panic!("Binary not found at {:?}", binary_path);
        }
        
        Self {
            temp_dir,
            binary_path: binary_path.to_string_lossy().to_string(),
        }
    }
    
    fn run_command(&self, args: &[&str]) -> (String, String, i32) {
        let output = Command::new(&self.binary_path)
            .args(args)
            .current_dir(self.temp_dir.path())
            .output()
            .expect("Failed to execute command");
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        
        (stdout, stderr, exit_code)
    }
}

#[test]
fn test_uat_meta_namespace_commands() {
    let env = UATTestEnvironment::new();
    
    println!("=== UAT Meta Namespace Commands Validation ===");
    
    // Test 1: Create meta cursor
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "cursor", "set", "uat_org1", "./uat_test.db", "--meta", "testorg1"
    ]);
    println!("✓ Meta cursor creation: exit_code={}, output={}", exit_code, stdout.trim());
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("meta context 'testorg1'"));
    
    // Test 2: List cursors (should show meta context)
    let (stdout, _stderr, exit_code) = env.run_command(&["cursor", "list"]);
    println!("✓ Cursor listing: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("[meta: testorg1]"));
    
    // Test 3: Store data with transparent addressing
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_org1", "set", "myapp.config.theme", "dark"
    ]);
    println!("✓ Transparent addressing SET: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    
    // Test 4: Retrieve data with meta context
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_org1", "get", "myapp.config.theme"
    ]);
    println!("✓ Meta context GET: exit_code={}, value={}", exit_code, stdout.trim());
    assert_eq!(exit_code, 0);
    assert_eq!(stdout.trim(), "dark");
    
    // Test 5: List keys with meta context
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_org1", "keys", "myapp.config"
    ]);
    println!("✓ Meta context KEYS: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("theme"));
    
    // Test 6: Scan pairs with meta context
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_org1", "scan", "myapp.config"
    ]);
    println!("✓ Meta context SCAN: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("theme") && stdout.contains("dark"));
    
    // Test 7: Create second org for isolation test
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "cursor", "set", "uat_org2", "./uat_test2.db", "--meta", "testorg2"
    ]);
    println!("✓ Second meta cursor: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    
    // Test 8: Store same key in different org
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_org2", "set", "myapp.config.theme", "light"
    ]);
    println!("✓ Isolation SET: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    
    // Test 9: Verify isolation (org1 should still see 'dark')
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_org1", "get", "myapp.config.theme"
    ]);
    println!("✓ Isolation verification org1: exit_code={}, value={}", exit_code, stdout.trim());
    assert_eq!(exit_code, 0);
    assert_eq!(stdout.trim(), "dark");
    
    // Test 10: Verify isolation (org2 should see 'light')
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_org2", "get", "myapp.config.theme"
    ]);
    println!("✓ Isolation verification org2: exit_code={}, value={}", exit_code, stdout.trim());
    assert_eq!(exit_code, 0);
    assert_eq!(stdout.trim(), "light");
    
    // Test 11: Test fallback compatibility
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "cursor", "set", "uat_legacy", "./uat_test.db"
    ]);
    println!("✓ Legacy cursor creation: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_legacy", "set", "legacy.data.value", "old_format"
    ]);
    println!("✓ Legacy data storage: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    
    // Test meta namespace isolation (org1 should NOT access legacy data)
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_org1", "get", "legacy.data.value"
    ]);
    println!("✓ Isolation verification: exit_code={}, value={}", exit_code, stdout.trim());
    assert_eq!(exit_code, 2); // MISS - meta context cannot access root namespace
    assert_eq!(stdout.trim(), "");
    
    // Test that legacy cursor can still access its own data
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "--cursor", "uat_legacy", "get", "legacy.data.value"
    ]);
    println!("✓ Legacy access verification: exit_code={}, value={}", exit_code, stdout.trim());
    assert_eq!(exit_code, 0);
    assert_eq!(stdout.trim(), "old_format");
    
    println!("🎉 All UAT meta namespace commands validated successfully!");
}

#[test]
fn test_uat_backward_compatibility() {
    let env = UATTestEnvironment::new();
    
    println!("=== UAT Backward Compatibility Validation ===");
    
    // Test that all existing commands still work exactly as before
    
    // Basic operations without cursors
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "-p", "test_project", "-n", "test_namespace", "set", "test_key", "test_value"
    ]);
    println!("✓ Basic SET: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "-p", "test_project", "-n", "test_namespace", "get", "test_key"
    ]);
    println!("✓ Basic GET: exit_code={}, value={}", exit_code, stdout.trim());
    assert_eq!(exit_code, 0);
    assert_eq!(stdout.trim(), "test_value");
    
    // Full path addressing
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "set", "test_project.test_namespace.path_key", "path_value"
    ]);
    println!("✓ Path addressing SET: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "get", "test_project.test_namespace.path_key"
    ]);
    println!("✓ Path addressing GET: exit_code={}, value={}", exit_code, stdout.trim());
    assert_eq!(exit_code, 0);
    assert_eq!(stdout.trim(), "path_value");
    
    // Keys and scan operations
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "-p", "test_project", "-n", "test_namespace", "keys"
    ]);
    println!("✓ KEYS operation: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("test_key") && stdout.contains("path_key"));
    
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "-p", "test_project", "-n", "test_namespace", "scan"
    ]);
    println!("✓ SCAN operation: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("test_value") && stdout.contains("path_value"));
    
    // Delete operations
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "-p", "test_project", "-n", "test_namespace", "del", "test_key"
    ]);
    println!("✓ DELETE operation: exit_code={}", exit_code);
    assert_eq!(exit_code, 0);
    
    let (stdout, _stderr, exit_code) = env.run_command(&[
        "-p", "test_project", "-n", "test_namespace", "get", "test_key"
    ]);
    println!("✓ DELETE verification: exit_code={}", exit_code);
    assert_eq!(exit_code, 2); // Should be MISS (exit code 2)
    
    println!("🎉 All backward compatibility tests passed!");
}

#[test]
fn test_uat_error_conditions() {
    let env = UATTestEnvironment::new();
    
    println!("=== UAT Error Conditions Validation ===");
    
    // Test proper error codes for various scenarios
    
    // Non-existent key
    let (_stdout, _stderr, exit_code) = env.run_command(&[
        "get", "nonexistent.project.key"
    ]);
    println!("✓ Non-existent key: exit_code={}", exit_code);
    assert_eq!(exit_code, 2); // MISS
    
    // Invalid command
    let (_stdout, _stderr, exit_code) = env.run_command(&[
        "invalid_command"
    ]);
    println!("✓ Invalid command: exit_code={}", exit_code);
    assert_eq!(exit_code, 1); // ERROR
    
    // Missing arguments
    let (_stdout, _stderr, exit_code) = env.run_command(&[
        "set"
    ]);
    println!("✓ Missing arguments: exit_code={}", exit_code);
    assert_eq!(exit_code, 1); // ERROR
    
    // Help should work
    let (_stdout, _stderr, exit_code) = env.run_command(&[
        "help"
    ]);
    println!("✓ Help command: exit_code={}", exit_code);
    assert_eq!(exit_code, 0); // SUCCESS
    
    println!("🎉 All error condition tests passed!");
}