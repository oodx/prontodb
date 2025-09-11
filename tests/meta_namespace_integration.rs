// Integration tests for meta namespace functionality
// Tests the complete meta namespace workflow from cursor creation to CRUD operations

use prontodb::{CursorManager, Storage, XdgPaths};
use prontodb::api::{SetValueConfig, set_value_with_cursor_and_manager, get_value_with_cursor_and_manager, delete_value_with_cursor_and_manager, list_keys_with_cursor_and_manager, scan_pairs_with_cursor_and_manager};
use std::path::PathBuf;
use tempfile::TempDir;

// Test utilities
struct TestEnvironment {
    temp_dir: TempDir,
    paths: XdgPaths,
    cursor_manager: CursorManager,
}

impl TestEnvironment {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let mut paths = XdgPaths::new();
        paths.data_dir = temp_dir.path().to_path_buf();
        paths.db_path = temp_dir.path().join("main.db");
        
        let cursor_manager = CursorManager::from_xdg(paths.clone());
        
        Self {
            temp_dir,
            paths,
            cursor_manager,
        }
    }
    
    fn create_test_database(&self, name: &str) -> PathBuf {
        let db_path = self.temp_dir.path().join(format!("{}.db", name));
        let storage = Storage::open(&db_path).unwrap();
        drop(storage); // Close the database
        db_path
    }
}

#[test]
fn test_meta_namespace_crud_operations() {
    let env = TestEnvironment::new();
    
    // Create a test database and cursor with meta context
    let db_path = env.create_test_database("work");
    env.cursor_manager.set_cursor_with_meta(
        "work",
        db_path,
        "alice",
        Some("company_engineering".to_string()),
        None,
        None,
    );
    
    // Test SET operation with meta context
    let config = SetValueConfig {
        project: Some("bashfx"),
        namespace: Some("config"),
        key_or_path: "debug",
        value: "true",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("work"),
        user: "alice",
        database: "main",
        meta_context_override: None,
    };
    
    set_value_with_cursor_and_manager(config, &env.cursor_manager).unwrap();
    
    // Test GET operation with meta context (should find the meta-prefixed key)
    let value = get_value_with_cursor_and_manager(
        Some("bashfx"),
        Some("config"),
        "debug",
        ".",
        Some("work"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    assert_eq!(value, Some("true".to_string()));
    
    // Test LIST operation with meta context
    let keys = list_keys_with_cursor_and_manager(
        "bashfx",
        "config",
        None,
        Some("work"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    assert!(keys.contains(&"debug".to_string()));
    
    // Test SCAN operation with meta context
    let pairs = scan_pairs_with_cursor_and_manager(
        "bashfx",
        "config",
        None,
        Some("work"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    assert!(pairs.contains(&("debug".to_string(), "true".to_string())));
    
    // Test DELETE operation with meta context
    delete_value_with_cursor_and_manager(
        Some("bashfx"),
        Some("config"),
        "debug",
        ".",
        Some("work"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    // Verify the key is deleted
    let value_after_delete = get_value_with_cursor_and_manager(
        Some("bashfx"),
        Some("config"),
        "debug",
        ".",
        Some("work"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    assert_eq!(value_after_delete, None);
}

#[test]
fn test_meta_namespace_fallback_compatibility() {
    let env = TestEnvironment::new();
    
    // Create a cursor without meta context
    let db_path = env.create_test_database("legacy");
    env.cursor_manager.set_cursor("legacy", db_path, "alice");
    
    // Store a value using the legacy cursor
    let config = SetValueConfig {
        project: Some("project1"),
        namespace: Some("ns1"),
        key_or_path: "key1",
        value: "legacy_value",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("legacy"),
        user: "alice",
        database: "main",
        meta_context_override: None,
    };
    
    set_value_with_cursor_and_manager(config, &env.cursor_manager).unwrap();
    
    // Now create a cursor with meta context pointing to the same database
    env.cursor_manager.set_cursor_with_meta(
        "meta_enabled",
        env.temp_dir.path().join("legacy.db"),
        "alice",
        Some("org".to_string()),
        None,
        None,
    );
    
    // GET operation with meta cursor should NOT fallback to direct key for security isolation
    let value = get_value_with_cursor_and_manager(
        Some("project1"),
        Some("ns1"),
        "key1",
        ".",
        Some("meta_enabled"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    // Meta cursor cannot access root namespace data - this ensures complete isolation
    assert_eq!(value, None);
}

#[test]
fn test_meta_namespace_isolation() {
    let env = TestEnvironment::new();
    
    // Create two cursors with different meta contexts pointing to the same database
    let db_path = env.create_test_database("shared");
    
    env.cursor_manager.set_cursor_with_meta(
        "org1",
        db_path.clone(),
        "alice",
        Some("organization1".to_string()),
        None,
        None,
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "org2",
        db_path,
        "alice",
        Some("organization2".to_string()),
        None,
        None,
    );
    
    // Store the same key in both contexts
    let config1 = SetValueConfig {
        project: Some("project"),
        namespace: Some("config"),
        key_or_path: "setting",
        value: "org1_value",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("org1"),
        user: "alice",
        database: "main",
        meta_context_override: None,
    };
    
    let config2 = SetValueConfig {
        project: Some("project"),
        namespace: Some("config"),
        key_or_path: "setting",
        value: "org2_value",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("org2"),
        user: "alice",
        database: "main",
        meta_context_override: None,
    };
    
    set_value_with_cursor_and_manager(config1, &env.cursor_manager).unwrap();
    set_value_with_cursor_and_manager(config2, &env.cursor_manager).unwrap();
    
    // Verify isolation - each cursor should see its own value
    let value1 = get_value_with_cursor_and_manager(
        Some("project"),
        Some("config"),
        "setting",
        ".",
        Some("org1"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    let value2 = get_value_with_cursor_and_manager(
        Some("project"),
        Some("config"),
        "setting",
        ".",
        Some("org2"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    assert_eq!(value1, Some("org1_value".to_string()));
    assert_eq!(value2, Some("org2_value".to_string()));
    
    // Verify keys are isolated in listing
    let keys1 = list_keys_with_cursor_and_manager(
        "project",
        "config",
        None,
        Some("org1"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    let keys2 = list_keys_with_cursor_and_manager(
        "project",
        "config", 
        None,
        Some("org2"),
        "alice",
        "main",
        &env.cursor_manager,
    ).unwrap();
    
    assert!(keys1.contains(&"setting".to_string()));
    assert!(keys2.contains(&"setting".to_string()));
    // Both should see the same key name but they're actually different stored keys
}

#[test]
fn test_meta_namespace_transparent_addressing() {
    let env = TestEnvironment::new();
    
    // Create cursor with meta context
    let db_path = env.create_test_database("transparent");
    env.cursor_manager.set_cursor_with_meta(
        "work",
        db_path,
        "alice",
        Some("company".to_string()),
        None,
        None,
    );
    
    // User types 3-layer address
    let config = SetValueConfig {
        project: Some("myapp"),
        namespace: Some("settings"),
        key_or_path: "theme",
        value: "dark",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("work"),
        user: "alice",
        database: "transparent", // This should match but cursor path takes precedence
        meta_context_override: None,
    };
    
    set_value_with_cursor_and_manager(config, &env.cursor_manager).unwrap();
    
    // Verify data is actually stored with meta-prefixed project by checking storage directly
    let storage = Storage::open(&env.temp_dir.path().join("transparent.db")).unwrap();
    
    // The key should be stored with project "company.myapp", namespace "settings", key "theme"
    use prontodb::addressing::{Address, AddressContext};
    let stored_addr = Address {
        project: "company.myapp".to_string(),
        namespace: "settings".to_string(),
        key: "theme".to_string(),
        context: None,
    };
    let stored_value = storage.get(&stored_addr).unwrap();
    
    assert_eq!(stored_value, Some("dark".to_string()));
    
    // But the 3-layer address should not exist directly
    let direct_addr = Address {
        project: "myapp".to_string(),
        namespace: "settings".to_string(),
        key: "theme".to_string(),
        context: None,
    };
    let direct_value = storage.get(&direct_addr).unwrap();
    assert_eq!(direct_value, None);
}

#[test]
fn test_meta_namespace_error_handling() {
    let env = TestEnvironment::new();
    
    // Test cursor with meta context that doesn't exist
    let result = get_value_with_cursor_and_manager(
        Some("project"),
        Some("config"),
        "key",
        ".",
        Some("nonexistent_cursor"),
        "alice",
        "main",
        &env.cursor_manager,
    );
    
    // Should fallback gracefully (no cursor found = use default database)
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None); // Key doesn't exist in default database
}