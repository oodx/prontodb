// Test to validate Krex's fix for XDG path isolation issue
use prontodb::{CursorManager, Storage, XdgPaths};
use prontodb::api::{SetValueConfig, set_value_with_cursor_and_manager};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_krex_fix_cursor_integration() {
    println!("=== TESTING KREX'S XDG PATH FIX ===");
    
    // Setup test environment with custom XDG paths
    let temp_dir = TempDir::new().unwrap();
    let mut paths = XdgPaths::new();
    paths.data_dir = temp_dir.path().to_path_buf();
    paths.db_path = temp_dir.path().join("main.db");
    
    let cursor_manager = CursorManager::from_xdg(paths);
    let db_path = temp_dir.path().join("test.db");
    
    // Create database file
    let storage = Storage::open(&db_path).unwrap();
    drop(storage);
    println!("✓ Database created at: {:?}", db_path);
    
    // Create cursor with meta context using the SAME cursor manager
    cursor_manager.set_cursor_with_meta(
        "testcursor",
        db_path.clone(),
        "alice",
        Some("testorg".to_string()),
        None,
        None,
    );
    println!("✓ Cursor created with meta context");
    
    // Store value using the enhanced API with EXPLICIT cursor manager
    let config = SetValueConfig {
        project: Some("myproject"),
        namespace: Some("settings"),
        key_or_path: "theme",
        value: "dark",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("testcursor"),
        user: "alice",
        database: "test",
    };
    
    println!("--- TESTING STORAGE WITH KREX'S FIX ---");
    set_value_with_cursor_and_manager(config, &cursor_manager).unwrap();
    println!("✓ Storage operation completed");
    
    // Verify the value was stored correctly
    let storage = Storage::open(&db_path).unwrap();
    
    use prontodb::addressing::Address;
    let meta_addr = Address {
        project: "testorg.myproject".to_string(),
        namespace: "settings".to_string(),
        key: "theme".to_string(),
        context: None,
    };
    
    let stored_value = storage.get(&meta_addr).unwrap();
    println!("Meta-prefixed storage result: {:?}", stored_value);
    
    // This should now pass!
    assert_eq!(stored_value, Some("dark".to_string()));
    println!("✅ KREX'S FIX SUCCESSFUL - Meta namespace storage working!");
}