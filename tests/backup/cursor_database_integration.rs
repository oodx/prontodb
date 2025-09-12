// Cursor system integration tests with multiple databases for SP-6
// Tests cursor directory isolation and database-scoped cursor functionality

use prontodb::xdg::test_utils::TestXdg;

/// Test cursor directory isolation between different databases
#[test]
fn test_cursor_directory_database_scoping() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    // Verify cursor directories are database-scoped
    let dev_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("dev");
    let prod_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("prod");
    let staging_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("staging");
    let test_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("test");
    
    // Each database should have its own cursor directory
    assert_ne!(dev_cursor_dir, prod_cursor_dir, "Dev and prod should have different cursor directories");
    assert_ne!(dev_cursor_dir, staging_cursor_dir, "Dev and staging should have different cursor directories");
    assert_ne!(dev_cursor_dir, test_cursor_dir, "Dev and test should have different cursor directories");
    assert_ne!(prod_cursor_dir, staging_cursor_dir, "Prod and staging should have different cursor directories");
    assert_ne!(prod_cursor_dir, test_cursor_dir, "Prod and test should have different cursor directories");
    assert_ne!(staging_cursor_dir, test_cursor_dir, "Staging and test should have different cursor directories");
    
    // Verify paths follow expected pattern
    assert!(dev_cursor_dir.to_string_lossy().contains("/dev/cursors"), "Dev cursor dir should be in dev database directory");
    assert!(prod_cursor_dir.to_string_lossy().contains("/prod/cursors"), "Prod cursor dir should be in prod database directory");
    assert!(staging_cursor_dir.to_string_lossy().contains("/staging/cursors"), "Staging cursor dir should be in staging database directory");
    assert!(test_cursor_dir.to_string_lossy().contains("/test/cursors"), "Test cursor dir should be in test database directory");
}

/// Test cursor directory creation and structure for multiple databases
#[test]
fn test_cursor_directory_creation() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    let database_names = ["app-dev", "app-staging", "app-prod", "cache-db", "logs-db"];
    
    for db_name in &database_names {
        let cursor_dir = test_xdg.paths.get_cursor_dir_with_name(db_name);
        let db_dir = test_xdg.paths.get_database_dir(db_name);
        
        // Cursor directory should be inside the database directory
        assert!(cursor_dir.starts_with(&db_dir), 
                "Cursor directory for {} should be inside its database directory", db_name);
        
        // Should be able to create the cursor directory
        std::fs::create_dir_all(&cursor_dir)
            .expect(&format!("Should create cursor directory for {}", db_name));
        
        assert!(cursor_dir.exists(), "Cursor directory should exist for {}", db_name);
        assert!(cursor_dir.is_dir(), "Cursor path should be a directory for {}", db_name);
    }
}

/// Test that cursor storage isolation works with multiple databases
#[test] 
fn test_cursor_storage_isolation() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    // Create databases for different applications
    let app1_db_path = test_xdg.paths.get_db_path_with_name("app1");
    let app2_db_path = test_xdg.paths.get_db_path_with_name("app2");
    let shared_db_path = test_xdg.paths.get_db_path_with_name("shared");
    
    let app1_storage = prontodb::Storage::open(&app1_db_path).expect("Should create app1 database");
    let app2_storage = prontodb::Storage::open(&app2_db_path).expect("Should create app2 database");
    let shared_storage = prontodb::Storage::open(&shared_db_path).expect("Should create shared database");
    
    // Verify each has its own cursor directory
    let app1_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("app1");
    let app2_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("app2");
    let shared_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("shared");
    
    assert_ne!(app1_cursor_dir, app2_cursor_dir, "App1 and app2 should have different cursor directories");
    assert_ne!(app1_cursor_dir, shared_cursor_dir, "App1 and shared should have different cursor directories");
    assert_ne!(app2_cursor_dir, shared_cursor_dir, "App2 and shared should have different cursor directories");
    
    // Store some data in each database to ensure they're isolated
    let test_addr = prontodb::Address::from_parts(
        Some("test".to_string()),
        Some("cursor".to_string()),
        "isolation".to_string(),
        None,
    );
    
    app1_storage.set(&test_addr, "app1-data", None).expect("Should set in app1");
    app2_storage.set(&test_addr, "app2-data", None).expect("Should set in app2");
    shared_storage.set(&test_addr, "shared-data", None).expect("Should set in shared");
    
    // Verify isolation
    let app1_result = app1_storage.get(&test_addr).expect("Should get from app1");
    let app2_result = app2_storage.get(&test_addr).expect("Should get from app2");
    let shared_result = shared_storage.get(&test_addr).expect("Should get from shared");
    
    assert_eq!(app1_result.unwrap(), "app1-data", "App1 should have app1 data");
    assert_eq!(app2_result.unwrap(), "app2-data", "App2 should have app2 data");
    assert_eq!(shared_result.unwrap(), "shared-data", "Shared should have shared data");
}

/// Test backwards compatibility - default cursor behavior still works
#[test]
fn test_default_cursor_compatibility() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    // Test that the default cursor directory method still works (uses "main" database)
    let default_cursor_dir = test_xdg.paths.get_cursor_dir();
    let main_cursor_dir = test_xdg.paths.get_cursor_dir_with_name("main");
    
    // Default should be same as main-scoped
    assert_eq!(default_cursor_dir, main_cursor_dir, 
               "Default cursor directory should be same as main database cursor directory");
    
    // Should follow expected structure
    assert!(default_cursor_dir.to_string_lossy().contains("/main/cursors"), 
            "Default cursor directory should be in main database directory");
}

/// Test cursor directory paths for various database name formats
#[test]
fn test_cursor_directory_name_variations() {
    let test_xdg = TestXdg::new().expect("create xdg");
    
    let database_names = vec![
        "simple",
        "with-dashes",
        "with_underscores", 
        "with.dots",
        "123numbers",
        "MixedCase",
        "very-long-database-name-with-many-parts",
        "a", // single character
    ];
    
    for db_name in &database_names {
        let cursor_dir = test_xdg.paths.get_cursor_dir_with_name(db_name);
        let db_dir = test_xdg.paths.get_database_dir(db_name);
        
        // Cursor directory should be properly scoped to the database
        assert!(cursor_dir.starts_with(&db_dir), 
                "Cursor directory should be inside database directory for '{}'", db_name);
        
        assert!(cursor_dir.to_string_lossy().contains(&format!("/{}/cursors", db_name)),
                "Cursor directory should contain correct database name path for '{}'", db_name);
        
        // Should be able to create the directory structure
        std::fs::create_dir_all(&cursor_dir)
            .expect(&format!("Should create cursor directory for '{}'", db_name));
        
        assert!(cursor_dir.exists(), "Cursor directory should exist for '{}'", db_name);
    }
}

// Note: Tests for actual cursor operations with CLI --database flag integration
// are pending the fix of global flag parsing in main.rs. The core cursor directory
// isolation and database-scoped cursor functionality is tested above.