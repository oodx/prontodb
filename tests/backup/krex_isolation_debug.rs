// Krex Iron Gate Isolation Failure Debug Test
// Mathematical precision test to identify the exact isolation failure point

use prontodb::{api::*, cursor::CursorManager, xdg::XdgPaths};
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;

struct IsolationTestHarness {
    temp_dir: TempDir,
    cursor_manager: CursorManager,
    paths: XdgPaths,
}

impl IsolationTestHarness {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Create isolated XDG paths
        let mut paths = XdgPaths::new();
        paths.data_dir = temp_dir.path().join("data");
        paths.db_path = paths.data_dir.join("main/pronto.main.prdb");
        
        let cursor_manager = CursorManager::from_xdg(paths.clone());
        
        Self {
            temp_dir,
            cursor_manager,
            paths,
        }
    }
    
    fn debug_cursor_state(&self, cursor_name: &str, user: &str) {
        match self.cursor_manager.get_cursor(cursor_name, user) {
            Ok(cursor_data) => {
                println!("✓ Cursor {}: db_path={:?}, meta_context={:?}", 
                    cursor_name, 
                    cursor_data.database_path,
                    cursor_data.meta_context
                );
            }
            Err(e) => {
                println!("✗ Cursor {} not found: {}", cursor_name, e);
            }
        }
    }
    
    fn debug_storage_path(&self, cursor_name: &str, user: &str) {
        // Use the cursor manager to check cursor data directly
        match self.cursor_manager.get_cursor(cursor_name, user) {
            Ok(cursor_data) => {
                println!("✓ Cursor data for {}: db_path={:?}, meta_context={:?}", 
                    cursor_name, cursor_data.database_path, cursor_data.meta_context);
            }
            Err(e) => {
                println!("✗ Failed to get cursor data for {}: {}", cursor_name, e);
            }
        }
    }
}

#[test]
fn test_krex_isolation_failure_reproduction() {
    let harness = IsolationTestHarness::new();
    
    println!("=== KREX IRON GATE ISOLATION FAILURE DEBUG ===");
    
    // Create two database files in the temp directory
    let db1_path = harness.temp_dir.path().join("uat_test.db");
    let db2_path = harness.temp_dir.path().join("uat_test2.db");
    
    println!("Database paths:");
    println!("  DB1: {:?}", db1_path);
    println!("  DB2: {:?}", db2_path);
    
    // Create the cursors with meta contexts
    harness.cursor_manager.set_cursor_with_meta(
        "uat_org1",
        db1_path.clone(),
        "default",
        Some("testorg1".to_string()),
        None,
        None,
    );
    
    harness.cursor_manager.set_cursor_with_meta(
        "uat_org2", 
        db2_path.clone(),
        "default",
        Some("testorg2".to_string()),
        None,
        None,
    );
    
    println!("\n--- CURSOR STATE VERIFICATION ---");
    harness.debug_cursor_state("uat_org1", "default");
    harness.debug_cursor_state("uat_org2", "default");
    
    println!("\n--- STORAGE PATH VERIFICATION ---");
    harness.debug_storage_path("uat_org1", "default");
    harness.debug_storage_path("uat_org2", "default");
    
    // Store values in each organization
    let set_config1 = SetValueConfig {
        project: None,
        namespace: None,
        key_or_path: "myapp.config.theme",
        value: "dark",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("uat_org1"),
        user: "default", 
        database: "main",
        meta_context_override: None,
    };
    
    let set_config2 = SetValueConfig {
        project: None,
        namespace: None,
        key_or_path: "myapp.config.theme",
        value: "light",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("uat_org2"),
        user: "default",
        database: "main", 
        meta_context_override: None,
    };
    
    println!("\n--- STORING VALUES ---");
    set_value_with_cursor_and_manager(set_config1, &harness.cursor_manager)
        .expect("Failed to set value in org1");
    println!("✓ Stored 'dark' in uat_org1");
    
    set_value_with_cursor_and_manager(set_config2, &harness.cursor_manager)
        .expect("Failed to set value in org2");
    println!("✓ Stored 'light' in uat_org2");
    
    println!("\n--- RETRIEVAL VERIFICATION (THE CRITICAL TEST) ---");
    
    // Retrieve from org1 - should get "dark"
    let org1_value = get_value_with_cursor_and_manager(
        None, None,
        "myapp.config.theme",
        ".",
        Some("uat_org1"),
        "default",
        "main",
        &harness.cursor_manager,
    ).expect("Failed to get value from org1");
    
    println!("ORG1 Retrieved: {:?}", org1_value);
    
    // Retrieve from org2 - should get "light"
    let org2_value = get_value_with_cursor_and_manager(
        None, None,
        "myapp.config.theme", 
        ".",
        Some("uat_org2"),
        "default",
        "main",
        &harness.cursor_manager,
    ).expect("Failed to get value from org2");
    
    println!("ORG2 Retrieved: {:?}", org2_value);
    
    println!("\n--- ISOLATION ANALYSIS ---");
    
    // Check if databases actually exist
    println!("DB1 exists: {}", db1_path.exists());
    println!("DB2 exists: {}", db2_path.exists());
    
    if db1_path.exists() && db2_path.exists() {
        let db1_size = fs::metadata(&db1_path).unwrap().len();
        let db2_size = fs::metadata(&db2_path).unwrap().len();
        println!("DB1 size: {} bytes", db1_size);
        println!("DB2 size: {} bytes", db2_size);
        
        if db1_size == db2_size {
            println!("🚨 CRITICAL: Database files are same size - potential file sharing");
        }
    }
    
    // Test the isolation requirement
    assert_eq!(org1_value, Some("dark".to_string()), "ORG1 should return 'dark'");
    assert_eq!(org2_value, Some("light".to_string()), "ORG2 should return 'light' - ISOLATION FAILURE");
    
    println!("\n🎯 If this test fails on the second assertion, we have reproduced the isolation failure");
}