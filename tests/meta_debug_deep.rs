// Deep debug test to track storage operation step by step
use prontodb::{CursorManager, Storage, XdgPaths};
use prontodb::api::{SetValueConfig};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn debug_storage_step_by_step() {
    println!("=== DEEP DEBUG: Storage Step by Step ===");
    
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let mut paths = XdgPaths::new();
    paths.data_dir = temp_dir.path().to_path_buf();
    paths.db_path = temp_dir.path().join("main.db");
    
    let cursor_manager = CursorManager::from_xdg(paths);
    let db_path = temp_dir.path().join("test.db");
    
    // Create database file
    let storage = Storage::open(&db_path).unwrap();
    drop(storage);
    println!("✓ Database file created at: {:?}", db_path);
    
    // Create cursor with meta context
    cursor_manager.set_cursor_with_meta(
        "testcursor",
        db_path.clone(),
        "alice",
        Some("testorg".to_string()),
        None,
        None,
    );
    println!("✓ Cursor created with meta context");
    
    // Verify cursor exists and has correct meta context
    let cursor_data = cursor_manager.get_cursor("testcursor", "alice").unwrap();
    println!("Cursor meta_context: {:?}", cursor_data.meta_context);
    println!("Cursor database_path: {:?}", cursor_data.database_path);
    assert_eq!(cursor_data.meta_context, Some("testorg".to_string()));
    assert_eq!(cursor_data.database_path, db_path);
    
    // Try to store a value using the meta cursor
    println!("--- ATTEMPTING STORAGE ---");
    let config = SetValueConfig {
        project: Some("myproject"),
        namespace: Some("settings"),
        key_or_path: "theme",
        value: "dark",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("testcursor"),
        user: "alice",
        database: "test", // Should be ignored in favor of cursor path
    };
    
    // Store the value
    match prontodb::api::set_value_with_cursor(config) {
        Ok(()) => println!("✓ set_value_with_cursor completed successfully"),
        Err(e) => {
            println!("✗ set_value_with_cursor failed: {}", e);
            panic!("Storage operation failed: {}", e);
        }
    }
    
    // Now inspect the database directly to see what was actually stored
    println!("--- DIRECT DATABASE INSPECTION ---");
    let storage = Storage::open(&db_path).unwrap();
    
    // Check various possible storage locations
    use prontodb::addressing::Address;
    
    // 1. Check if stored with meta-prefixed project
    let meta_addr = Address {
        project: "testorg.myproject".to_string(),
        namespace: "settings".to_string(),
        key: "theme".to_string(),
        context: None,
    };
    let meta_result = storage.get(&meta_addr).unwrap();
    println!("Meta-prefixed storage (testorg.myproject.settings.theme): {:?}", meta_result);
    
    // 2. Check if stored with direct project
    let direct_addr = Address {
        project: "myproject".to_string(),
        namespace: "settings".to_string(),
        key: "theme".to_string(),
        context: None,
    };
    let direct_result = storage.get(&direct_addr).unwrap();
    println!("Direct storage (myproject.settings.theme): {:?}", direct_result);
    
    // 3. Try to list all projects to see what's actually there
    let projects = storage.list_projects().unwrap();
    println!("All projects in database: {:?}", projects);
    
    // 4. If no projects, try to see if any keys exist at all
    if projects.is_empty() {
        println!("No projects found - database appears empty");
        
        // Check if the database file itself exists and has content
        let metadata = std::fs::metadata(&db_path).unwrap();
        println!("Database file size: {} bytes", metadata.len());
        
        // Try to open storage again and verify it's working
        let test_addr = Address {
            project: "test".to_string(),
            namespace: "test".to_string(),
            key: "test".to_string(),
            context: None,
        };
        storage.set(&test_addr, "test_value", None).unwrap();
        let retrieved = storage.get(&test_addr).unwrap();
        println!("Manual storage test result: {:?}", retrieved);
        
        if retrieved == Some("test_value".to_string()) {
            println!("✓ Direct storage operations work");
        } else {
            println!("✗ Direct storage operations don't work");
        }
    }
    
    // The test should pass if meta-prefixed storage worked
    assert_eq!(meta_result, Some("dark".to_string()), "Value should be stored with meta prefix");
}