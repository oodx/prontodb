// Multi-database integration tests for SP-6
// Tests database isolation and core database-scoped functionality

use prontodb::xdg::test_utils::TestXdg;

/// Test comprehensive database isolation across all storage operations
#[test]
fn test_complete_database_isolation() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    // Create separate storage instances for different databases
    let dev_db_path = test_xdg.paths.get_db_path_with_name("dev");
    let staging_db_path = test_xdg.paths.get_db_path_with_name("staging");
    let prod_db_path = test_xdg.paths.get_db_path_with_name("prod");
    
    let dev_storage = prontodb::Storage::open(&dev_db_path).expect("Should create dev database");
    let staging_storage = prontodb::Storage::open(&staging_db_path).expect("Should create staging database");
    let prod_storage = prontodb::Storage::open(&prod_db_path).expect("Should create prod database");
    
    // Create the same address for testing isolation
    let test_addr = prontodb::Address::from_parts(
        Some("app".to_string()),
        Some("config".to_string()),
        "db_host".to_string(),
        None,
    );
    
    // Set different values for the same address in different databases
    dev_storage.set(&test_addr, "dev-db.local", None).expect("Should set in dev database");
    staging_storage.set(&test_addr, "staging-db.local", None).expect("Should set in staging database");
    prod_storage.set(&test_addr, "prod-db.local", None).expect("Should set in prod database");
    
    // Verify complete isolation - each database has its own value
    let dev_result = dev_storage.get(&test_addr).expect("Should get from dev database");
    assert_eq!(dev_result.unwrap(), "dev-db.local", "Dev database should have dev value");
    
    let staging_result = staging_storage.get(&test_addr).expect("Should get from staging database");
    assert_eq!(staging_result.unwrap(), "staging-db.local", "Staging database should have staging value");
    
    let prod_result = prod_storage.get(&test_addr).expect("Should get from prod database");
    assert_eq!(prod_result.unwrap(), "prod-db.local", "Prod database should have prod value");
    
    // Delete from one database and verify others are unaffected
    dev_storage.delete(&test_addr).expect("Should delete from dev database");
    
    // Dev should be missing the key
    let dev_result = dev_storage.get(&test_addr).expect("Should query dev database");
    assert!(dev_result.is_none(), "Dev database should not have deleted key");
    
    // Other databases should still have their values
    let staging_result = staging_storage.get(&test_addr).expect("Should get from staging database");
    assert_eq!(staging_result.unwrap(), "staging-db.local", "Staging should still have its value");
    
    let prod_result = prod_storage.get(&test_addr).expect("Should get from prod database");
    assert_eq!(prod_result.unwrap(), "prod-db.local", "Prod should still have its value");
}

/// Test database path isolation and basic functionality
#[test] 
fn test_database_path_isolation() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    // Test that different databases use different paths
    let db1_path = test_xdg.paths.get_db_path_with_name("database1");
    let db2_path = test_xdg.paths.get_db_path_with_name("database2");
    let db3_path = test_xdg.paths.get_db_path_with_name("database3");
    
    // Verify isolation
    assert_ne!(db1_path, db2_path, "database1 and database2 should have different paths");
    assert_ne!(db1_path, db3_path, "database1 and database3 should have different paths"); 
    assert_ne!(db2_path, db3_path, "database2 and database3 should have different paths");
    
    // Verify path structure
    assert!(db1_path.to_string_lossy().contains("/database1/"), "database1 path should contain database1 directory");
    assert!(db2_path.to_string_lossy().contains("/database2/"), "database2 path should contain database2 directory");
    assert!(db3_path.to_string_lossy().contains("/database3/"), "database3 path should contain database3 directory");
    
    // Test that we can create storage for each
    let _storage1 = prontodb::Storage::open(&db1_path).expect("Should create database1 storage");
    let _storage2 = prontodb::Storage::open(&db2_path).expect("Should create database2 storage");
    let _storage3 = prontodb::Storage::open(&db3_path).expect("Should create database3 storage");
    
    // Verify database files exist in separate locations
    assert!(db1_path.exists(), "database1 file should exist");
    assert!(db2_path.exists(), "database2 file should exist");
    assert!(db3_path.exists(), "database3 file should exist");
}

/// Test cursor directory isolation for different databases
#[test]
fn test_cursor_directory_database_scoping() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    // Verify cursor directories are database-scoped
    let dev_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("dev");
    let prod_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("prod");
    let staging_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("staging");
    
    // Each database should have its own cursor directory
    assert_ne!(dev_cursor_dir, prod_cursor_dir, "Dev and prod should have different cursor directories");
    assert_ne!(dev_cursor_dir, staging_cursor_dir, "Dev and staging should have different cursor directories");
    assert_ne!(prod_cursor_dir, staging_cursor_dir, "Prod and staging should have different cursor directories");
    
    // Verify paths follow expected pattern
    assert!(dev_cursor_dir.to_string_lossy().contains("/dev/cursors"), "Dev cursor dir should be in dev database directory");
    assert!(prod_cursor_dir.to_string_lossy().contains("/prod/cursors"), "Prod cursor dir should be in prod database directory");
    assert!(staging_cursor_dir.to_string_lossy().contains("/staging/cursors"), "Staging cursor dir should be in staging database directory");
}

/// Test multiple database instances with different data sets
#[test]
fn test_multi_database_data_sets() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    // Create databases for different environments
    let environments = ["development", "testing", "staging", "production"];
    let mut databases = std::collections::HashMap::new();
    
    // Initialize each environment database
    for env in &environments {
        let db_path = test_xdg.paths.get_db_path_with_name(env);
        let storage = prontodb::Storage::open(&db_path)
            .expect(&format!("Should create {} database", env));
        databases.insert(*env, storage);
    }
    
    // Set environment-specific configurations
    for (env, storage) in &databases {
        let config_addr = prontodb::Address::from_parts(
            Some("app".to_string()),
            Some("config".to_string()),
            "environment".to_string(),
            None,
        );
        
        let debug_addr = prontodb::Address::from_parts(
            Some("app".to_string()),
            Some("config".to_string()),
            "debug_mode".to_string(),
            None,
        );
        
        // Set environment name
        storage.set(&config_addr, env, None)
            .expect(&format!("Should set environment for {}", env));
        
        // Set debug mode (only for dev/test)
        let debug_value = if *env == "development" || *env == "testing" { "true" } else { "false" };
        storage.set(&debug_addr, debug_value, None)
            .expect(&format!("Should set debug mode for {}", env));
    }
    
    // Verify each environment has correct configuration
    for (env, storage) in &databases {
        let config_addr = prontodb::Address::from_parts(
            Some("app".to_string()),
            Some("config".to_string()),
            "environment".to_string(),
            None,
        );
        
        let debug_addr = prontodb::Address::from_parts(
            Some("app".to_string()),
            Some("config".to_string()),
            "debug_mode".to_string(),
            None,
        );
        
        // Check environment name
        let env_result = storage.get(&config_addr).expect("Should get environment");
        assert_eq!(env_result.unwrap(), *env, "Environment should match database name");
        
        // Check debug mode
        let debug_result = storage.get(&debug_addr).expect("Should get debug mode");
        let expected_debug = if *env == "development" || *env == "testing" { "true" } else { "false" };
        assert_eq!(debug_result.unwrap(), expected_debug, 
                   "Debug mode should be correct for {}", env);
    }
}

/// Test that database names with various formats work correctly
#[test]
fn test_database_name_variations() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    // Test various valid database names
    let valid_names = vec![
        "simple",
        "with-dashes", 
        "with_underscores",
        "with.dots",
        "123numbers",
        "MixedCase",
        "long_database_name_with_many_parts",
        "a", // single character
    ];
    
    for name in &valid_names {
        let db_path = test_xdg.paths.get_db_path_with_name(name);
        
        // Should be able to create storage
        let storage = prontodb::Storage::open(&db_path)
            .expect(&format!("Should create storage for database '{}'", name));
        
        // Should be able to store and retrieve data
        let test_addr = prontodb::Address::from_parts(
            Some("test".to_string()),
            Some("ns".to_string()),
            "key".to_string(),
            None,
        );
        
        storage.set(&test_addr, &format!("value-{}", name), None)
            .expect(&format!("Should set value in database '{}'", name));
        
        let result = storage.get(&test_addr)
            .expect(&format!("Should get value from database '{}'", name));
        assert_eq!(result.unwrap(), format!("value-{}", name), 
                   "Should retrieve correct value from database '{}'", name);
        
        // Database file should exist in correct location
        assert!(db_path.exists(), "Database file should exist for '{}'", name);
        assert!(db_path.to_string_lossy().contains(&format!("/{}/", name)), 
                "Database path should contain database name directory for '{}'", name);
    }
}

// Note: Additional tests for CLI --database flag integration, backup/restore across databases,
// and cursor integration with multiple databases are in cursor_database_integration.rs
// These require the global flag parsing to be fixed in main.rs