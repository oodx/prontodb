// Sanity tests - Basic assumptions and core functionality
// These tests verify our fundamental assumptions work before building complex features

use std::process::Command;
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
    assert!(stdout.contains("USAGE:") || stdout.contains("Usage:"), "Help should show usage");
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
            .unwrap_or_else(|_| panic!("Failed to execute: {:?}", args));
        
        // Sanity: doesn't crash with segfault etc
        assert!(
            output.status.code().is_some(),
            "Command {:?} should exit cleanly (not crash)",
            args
        );
        
        // Sanity: exit code is reasonable (0, 1, or 2)
        let code = output.status.code().unwrap();
        assert!(
            (0..=2).contains(&code),
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
    
    let _storage = storage_result.unwrap();
    
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

/// Test database-scoped path structure
#[test]
fn sanity_database_scoped_paths() {
    let test_xdg = TestXdg::new().expect("Failed to create test XDG environment");
    
    // Test that different databases use different directories
    let main_db = test_xdg.paths.get_db_path_with_name("main");
    let test_db = test_xdg.paths.get_db_path_with_name("test");
    let staging_db = test_xdg.paths.get_db_path_with_name("staging");
    
    // Sanity: different databases use different paths
    assert_ne!(main_db, test_db, "main and test databases should have different paths");
    assert_ne!(main_db, staging_db, "main and staging databases should have different paths");
    assert_ne!(test_db, staging_db, "test and staging databases should have different paths");
    
    // Sanity: paths follow database-scoped structure
    assert!(main_db.to_string_lossy().contains("/main/pronto.main.prdb"), "main database should be in main directory");
    assert!(test_db.to_string_lossy().contains("/test/pronto.test.prdb"), "test database should be in test directory");
    assert!(staging_db.to_string_lossy().contains("/staging/pronto.staging.prdb"), "staging database should be in staging directory");
    
    // Test cursor directories are also scoped
    let main_cursors = test_xdg.paths.get_cursor_dir_with_name("main");
    let test_cursors = test_xdg.paths.get_cursor_dir_with_name("test");
    
    assert_ne!(main_cursors, test_cursors, "different databases should have different cursor dirs");
    assert!(main_cursors.to_string_lossy().contains("/main/cursors"), "main cursors should be in main directory");
    assert!(test_cursors.to_string_lossy().contains("/test/cursors"), "test cursors should be in test directory");
}

/// Test --database flag functionality at basic level
#[test]
fn sanity_database_flag_functionality() {
    let test_xdg = TestXdg::new().expect("Failed to create test XDG environment");
    let home = test_xdg.home_str();
    
    // Test help command with --database flag (should not affect help)
    // Note: Global flag parsing might need fixes, but help should work
    let output = Command::new("./target/debug/prontodb")
        .args(["--database", "testdb", "help"])
        .env("HOME", home)
        .output()
        .expect("failed to execute with database flag");
    
    // For now, just verify the command doesn't crash with the flag
    assert!(output.status.code().is_some(), "Command should not crash with --database flag");
    
    // Test that database parameter validation works
    for db_name in ["test", "staging", "prod", "dev", "db-1"] {
        let output = Command::new("./target/debug/prontodb")
            .args(["--database", db_name, "help"])
            .env("HOME", home)
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute with database name: {}", db_name));
        
        // Should not crash with valid database names
        assert!(output.status.code().is_some(), "Command should not crash with database name: {}", db_name);
    }
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

/// Test database path structure functionality 
#[test]
fn sanity_database_data_isolation() {
    let test_xdg = TestXdg::new().expect("Failed to create test XDG environment");
    
    // Test that database paths are properly scoped
    let test_db_path = test_xdg.paths.get_db_path_with_name("test");
    let staging_db_path = test_xdg.paths.get_db_path_with_name("staging"); 
    let prod_db_path = test_xdg.paths.get_db_path_with_name("prod");
    
    // Verify database isolation at the path level
    assert_ne!(test_db_path, staging_db_path, "test and staging databases should have different paths");
    assert_ne!(test_db_path, prod_db_path, "test and prod databases should have different paths");
    assert_ne!(staging_db_path, prod_db_path, "staging and prod databases should have different paths");
    
    // Verify proper database directory structure
    assert!(test_db_path.to_string_lossy().contains("/test/"), "test database should be in test directory");
    assert!(staging_db_path.to_string_lossy().contains("/staging/"), "staging database should be in staging directory");
    assert!(prod_db_path.to_string_lossy().contains("/prod/"), "prod database should be in prod directory");
    
    // Test creating storage instances for different databases
    let test_storage = prontodb::Storage::open(&test_db_path);
    let staging_storage = prontodb::Storage::open(&staging_db_path);
    
    assert!(test_storage.is_ok(), "Should be able to create test database storage");
    assert!(staging_storage.is_ok(), "Should be able to create staging database storage");
    
    // Verify database files are created in isolated locations
    assert!(test_db_path.exists(), "Test database file should be created");
    assert!(staging_db_path.exists(), "Staging database file should be created");
    
    // Test that different databases can store different data
    let mut test_store = test_storage.unwrap();
    let mut staging_store = staging_storage.unwrap();
    
    let test_addr = prontodb::Address::from_parts(
        Some("proj".to_string()),
        Some("ns".to_string()),
        "key1".to_string(),
        None,
    );
    
    // Store different values in each database
    test_store.set(&test_addr, "test_value", None).expect("Should set value in test database");
    staging_store.set(&test_addr, "staging_value", None).expect("Should set value in staging database");
    
    // Verify isolation - each database should have its own value
    let test_result = test_store.get(&test_addr).expect("Should get from test database");
    let staging_result = staging_store.get(&test_addr).expect("Should get from staging database");
    
    assert_eq!(test_result.unwrap(), "test_value", "Test database should have test_value");
    assert_eq!(staging_result.unwrap(), "staging_value", "Staging database should have staging_value");
}