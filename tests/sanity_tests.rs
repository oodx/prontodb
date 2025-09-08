// Sanity tests - Basic assumptions and core functionality
// These tests verify our fundamental assumptions work before building complex features

use std::process::Command;
use tempfile::TempDir;
use prontodb::xdg::test_utils::TestXdg;

/// Test basic binary can be executed
#[test]
fn sanity_binary_exists_and_runs() {
    let output = Command::new("./target/debug/prontodb")
        .arg("help")
        .output()
        .expect("failed to execute prontodb binary");
    
    // Basic sanity: binary runs without crashing
    assert!(output.status.success(), "Binary should execute successfully");
    
    // Basic sanity: produces some output
    assert!(!output.stdout.is_empty(), "Help should produce output");
    
    // Basic sanity: help mentions expected content
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("prontodb"), "Help should mention prontodb");
    assert!(stdout.contains("Usage:"), "Help should show usage");
}

/// Test XDG path isolation works correctly
#[test]
fn sanity_xdg_isolation_works() {
    let test_xdg = TestXdg::new().expect("Failed to create test XDG environment");
    
    // Sanity: directories are created
    assert!(test_xdg.paths.data_dir.exists(), "Data directory should be created");
    assert!(test_xdg.paths.config_dir.exists(), "Config directory should be created");
    
    // Sanity: paths are isolated (not in real home)
    let home = std::env::var("HOME").unwrap_or_default();
    assert!(!test_xdg.home_str().starts_with(&home), "Test paths should be isolated from real HOME");
    
    // Sanity: can get string paths for env vars
    assert!(!test_xdg.home_str().is_empty(), "Should provide home path string");
    assert!(!test_xdg.db_path_str().is_empty(), "Should provide db path string");
}

/// Test environment variable overrides work
#[test]
fn sanity_env_var_overrides() {
    let test_xdg = TestXdg::new().expect("Failed to create test XDG environment");
    let custom_db = test_xdg.temp_dir.path().join("custom_test.db");
    
    // Set custom database path
    std::env::set_var("PRONTO_DB", &custom_db);
    
    // Sanity: override is respected
    assert_eq!(test_xdg.paths.get_db_path(), custom_db);
    
    // Cleanup
    std::env::remove_var("PRONTO_DB");
}

/// Test basic command parsing doesn't crash
#[test]
fn sanity_command_parsing_basic() {
    // Test various command patterns don't cause panics
    
    let test_cases = vec![
        vec!["help"],
        vec!["--help"],
        vec!["-h"],
        vec!["nonexistent"],
        vec!["set"],  // incomplete, should error gracefully
        vec!["get"],  // incomplete, should error gracefully
    ];
    
    for args in test_cases {
        let output = Command::new("./target/debug/prontodb")
            .args(&args)
            .output()
            .expect(&format!("Failed to execute: {:?}", args));
        
        // Sanity: doesn't crash with segfault etc
        assert!(
            output.status.code().is_some(),
            "Command {:?} should exit cleanly (not crash)",
            args
        );
        
        // Sanity: exit code is reasonable (0, 1, or 2)
        let code = output.status.code().unwrap();
        assert!(
            code >= 0 && code <= 2,
            "Command {:?} should use standard exit codes, got {}",
            args, code
        );
    }
}

/// Test storage module basic assumptions
#[test]
fn sanity_storage_module_basic() {
    let test_xdg = TestXdg::new().expect("Failed to create test XDG environment");
    
    // Sanity: can create storage instance without panic
    let storage_result = prontodb::Storage::open(&test_xdg.paths.db_path);
    assert!(storage_result.is_ok(), "Should be able to create storage instance");
    
    let storage = storage_result.unwrap();
    
    // Sanity: database file is created
    assert!(test_xdg.paths.db_path.exists(), "Database file should be created");
    
    // Sanity: can create basic address without panic
    let addr = prontodb::Address::from_parts(
        Some("test".to_string()),
        Some("ns".to_string()),
        "key".to_string(),
        None,
    );
    
    // Sanity: address has expected values
    assert_eq!(addr.project, "test");
    assert_eq!(addr.namespace, "ns");
    assert_eq!(addr.key, "key");
    assert_eq!(addr.context, None);
}

/// Test addressing module basic assumptions
#[test]
fn sanity_addressing_parsing_basic() {
    // Test basic path parsing doesn't panic
    
    let test_cases = vec![
        ("simple", "."),
        ("ns.key", "."),
        ("proj.ns.key", "."),
        ("proj.ns.key__ctx", "."),
        ("a|b|c", "|"),
        ("single", "."),
    ];
    
    for (path, delim) in test_cases {
        let result = prontodb::Address::parse(path, delim);
        
        // Sanity: parsing doesn't panic
        assert!(
            result.is_ok(),
            "Parsing '{}' with delimiter '{}' should not fail",
            path, delim
        );
        
        let addr = result.unwrap();
        
        // Sanity: parsed address has non-empty components
        assert!(!addr.project.is_empty(), "Project should not be empty");
        assert!(!addr.namespace.is_empty(), "Namespace should not be empty");
        assert!(!addr.key.is_empty(), "Key should not be empty");
    }
}

/// Test exit codes match expectations
#[test]
fn sanity_exit_codes_reasonable() {
    let test_xdg = TestXdg::new().expect("Failed to create test XDG environment");
    
    // Test help command (should exit 0)
    let output = Command::new("./target/debug/prontodb")
        .arg("help")
        .output()
        .expect("failed to execute");
    
    assert_eq!(output.status.code(), Some(0), "Help should exit with code 0");
    
    // Test unknown command (should exit non-zero, not 2)
    let output = Command::new("./target/debug/prontodb")
        .arg("totally_unknown_command")
        .output()
        .expect("failed to execute");
    
    let code = output.status.code().unwrap();
    assert!(code != 0, "Unknown command should not exit 0");
    assert!(code != 2, "Unknown command should not exit 2 (MISS is for missing keys)");
    
    // Test incomplete command
    let output = Command::new("./target/debug/prontodb")
        .arg("set")  // Missing required args
        .env("HOME", test_xdg.home_str())
        .output()
        .expect("failed to execute");
    
    let code = output.status.code().unwrap();
    assert!(code != 0, "Incomplete command should not exit 0");
    assert!(code != 2, "Incomplete command should not use MISS code");
}

/// Test multi-instance isolation
#[test]
fn sanity_multi_instance_isolation() {
    // Clear XDG environment variables to ensure true isolation for test
    let _xdg_data_home = std::env::var("XDG_DATA_HOME");
    let _xdg_config_home = std::env::var("XDG_CONFIG_HOME");
    let _xdg_cache_home = std::env::var("XDG_CACHE_HOME");
    
    std::env::remove_var("XDG_DATA_HOME");
    std::env::remove_var("XDG_CONFIG_HOME");
    std::env::remove_var("XDG_CACHE_HOME");
    
    // Create two isolated test environments
    let test_xdg1 = TestXdg::new().expect("Failed to create test XDG environment 1");
    let test_xdg2 = TestXdg::new().expect("Failed to create test XDG environment 2");
    
    // Restore environment variables
    if let Ok(val) = _xdg_data_home { std::env::set_var("XDG_DATA_HOME", val); }
    if let Ok(val) = _xdg_config_home { std::env::set_var("XDG_CONFIG_HOME", val); }  
    if let Ok(val) = _xdg_cache_home { std::env::set_var("XDG_CACHE_HOME", val); }
    
    // Sanity: they have different temp directories  
    assert_ne!(
        test_xdg1.home_str(),
        test_xdg2.home_str(),
        "Different instances should have different home directories"
    );
    
    // Debug: print the paths to understand what's happening
    println!("XDG1 home: {}", test_xdg1.home_str());
    println!("XDG1 db_path: {}", test_xdg1.db_path_str());
    println!("XDG2 home: {}", test_xdg2.home_str());
    println!("XDG2 db_path: {}", test_xdg2.db_path_str());
    
    // Sanity: they have different database paths derived from different homes
    assert_ne!(
        test_xdg1.db_path_str(),
        test_xdg2.db_path_str(),
        "Different instances should have different database paths"
    );
    
    // Sanity: can create storage in both without conflict
    let storage1 = prontodb::Storage::open(&test_xdg1.paths.db_path);
    let storage2 = prontodb::Storage::open(&test_xdg2.paths.db_path);
    
    assert!(storage1.is_ok(), "Should create storage in instance 1");
    assert!(storage2.is_ok(), "Should create storage in instance 2");
    
    // Sanity: both database files exist independently
    assert!(test_xdg1.paths.db_path.exists());
    assert!(test_xdg2.paths.db_path.exists());
}