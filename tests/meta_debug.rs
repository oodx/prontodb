// Debug test to systematically validate meta namespace assumptions
use prontodb::{CursorManager, Storage, XdgPaths};
use prontodb::api::SetValueConfig;
use std::path::PathBuf;
use tempfile::TempDir;

struct SimpleTestEnv {
    temp_dir: TempDir,
    cursor_manager: CursorManager,
}

impl SimpleTestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let mut paths = XdgPaths::new();
        paths.data_dir = temp_dir.path().to_path_buf();
        paths.db_path = temp_dir.path().join("main.db");
        
        let cursor_manager = CursorManager::from_xdg(paths);
        Self { temp_dir, cursor_manager }
    }
    
    fn create_shared_db(&self) -> PathBuf {
        let db_path = self.temp_dir.path().join("shared.db");
        // Initialize database
        let storage = Storage::open(&db_path).unwrap();
        drop(storage);
        db_path
    }
}

#[test]
fn debug_step1_basic_cursor_creation() {
    println!("=== STEP 1: Basic Cursor Creation ===");
    let env = SimpleTestEnv::new();
    let db_path = env.create_shared_db();
    
    // Create two cursors with different meta contexts pointing to same DB
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
        db_path.clone(),
        "alice",
        Some("organization2".to_string()),
        None,
        None,
    );
    
    // Verify cursors were created with correct meta contexts
    let cursor1 = env.cursor_manager.get_cursor("org1", "alice").unwrap();
    let cursor2 = env.cursor_manager.get_cursor("org2", "alice").unwrap();
    
    println!("Cursor1 meta_context: {:?}", cursor1.meta_context);
    println!("Cursor2 meta_context: {:?}", cursor2.meta_context);
    println!("Cursor1 database_path: {:?}", cursor1.database_path);
    println!("Cursor2 database_path: {:?}", cursor2.database_path);
    
    assert_eq!(cursor1.meta_context, Some("organization1".to_string()));
    assert_eq!(cursor2.meta_context, Some("organization2".to_string()));
    assert_eq!(cursor1.database_path, cursor2.database_path); // Same DB file
    
    println!("✓ Step 1 passed: Cursors created correctly with different meta contexts");
}

#[test]
fn debug_step2_storage_operation() {
    println!("=== STEP 2: Single Storage Operation ===");
    let env = SimpleTestEnv::new();
    let db_path = env.create_shared_db();
    
    env.cursor_manager.set_cursor_with_meta(
        "org1",
        db_path,
        "alice", 
        Some("organization1".to_string()),
        None,
        None,
    );
    
    // Store one value using org1 cursor
    let config = SetValueConfig {
        project: Some("project"),
        namespace: Some("config"),
        key_or_path: "setting",
        value: "org1_value",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("org1"),
        user: "alice",
        database: "shared", // Database name parameter (should be overridden by cursor)
    };
    
    println!("Storing value using org1 cursor...");
    prontodb::api::set_value_with_cursor_and_manager(config, &env.cursor_manager).unwrap();
    println!("✓ Storage successful");
    
    // Try to retrieve the value
    let retrieved = prontodb::api::get_value_with_cursor_and_manager(
        Some("project"),
        Some("config"),
        "setting",
        ".",
        Some("org1"),
        "alice",
        "shared",
        &env.cursor_manager,
    ).unwrap();
    
    println!("Retrieved value: {:?}", retrieved);
    assert_eq!(retrieved, Some("org1_value".to_string()));
    
    println!("✓ Step 2 passed: Single storage and retrieval works");
}

#[test]
fn debug_step3_isolation_validation() {
    println!("=== STEP 3: Isolation Validation ===");
    let env = SimpleTestEnv::new();
    let db_path = env.create_shared_db();
    
    // Create cursors
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
    
    // Store value using org1
    let config1 = SetValueConfig {
        project: Some("project"),
        namespace: Some("config"),
        key_or_path: "setting",
        value: "org1_value",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("org1"),
        user: "alice",
        database: "shared",
    };
    prontodb::api::set_value_with_cursor_and_manager(config1, &env.cursor_manager).unwrap();
    println!("✓ Stored org1_value using org1 cursor");
    
    // Store value using org2 (same key path)
    let config2 = SetValueConfig {
        project: Some("project"),
        namespace: Some("config"),
        key_or_path: "setting",
        value: "org2_value",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("org2"),
        user: "alice", 
        database: "shared",
    };
    prontodb::api::set_value_with_cursor_and_manager(config2, &env.cursor_manager).unwrap();
    println!("✓ Stored org2_value using org2 cursor");
    
    // Verify each cursor sees its own value
    let value1 = prontodb::api::get_value_with_cursor_and_manager(
        Some("project"),
        Some("config"), 
        "setting",
        ".",
        Some("org1"),
        "alice",
        "shared",
        &env.cursor_manager,
    ).unwrap();
    
    let value2 = prontodb::api::get_value_with_cursor_and_manager(
        Some("project"),
        Some("config"),
        "setting", 
        ".",
        Some("org2"),
        "alice",
        "shared",
        &env.cursor_manager,
    ).unwrap();
    
    println!("Value retrieved via org1 cursor: {:?}", value1);
    println!("Value retrieved via org2 cursor: {:?}", value2);
    
    // This is where the original test was failing
    assert_eq!(value1, Some("org1_value".to_string()));
    assert_eq!(value2, Some("org2_value".to_string()));
    
    println!("✓ Step 3 passed: Complete isolation verified");
}

#[test]
fn debug_step4_storage_inspection() {
    println!("=== STEP 4: Direct Storage Inspection ===");
    let env = SimpleTestEnv::new();
    let db_path = env.create_shared_db();
    
    env.cursor_manager.set_cursor_with_meta(
        "org1",
        db_path.clone(),
        "alice",
        Some("organization1".to_string()),
        None,
        None,
    );
    
    // Store a value
    let config = SetValueConfig {
        project: Some("myproject"),
        namespace: Some("settings"),
        key_or_path: "theme",
        value: "dark",
        ns_delim: ".",
        ttl_flag: None,
        cursor_name: Some("org1"),
        user: "alice",
        database: "shared",
    };
    prontodb::api::set_value_with_cursor_and_manager(config, &env.cursor_manager).unwrap();
    println!("✓ Stored value via meta cursor");
    
    // Inspect storage directly
    let storage = Storage::open(&db_path).unwrap();
    
    // Test 1: Check if stored with meta-prefixed project name
    use prontodb::addressing::Address;
    let meta_addr = Address {
        project: "organization1.myproject".to_string(),
        namespace: "settings".to_string(),
        key: "theme".to_string(),
        context: None,
    };
    let meta_value = storage.get(&meta_addr).unwrap();
    println!("Meta-prefixed address result: {:?}", meta_value);
    
    // Test 2: Check if direct project name has no value
    let direct_addr = Address {
        project: "myproject".to_string(),
        namespace: "settings".to_string(), 
        key: "theme".to_string(),
        context: None,
    };
    let direct_value = storage.get(&direct_addr).unwrap();
    println!("Direct address result: {:?}", direct_value);
    
    assert_eq!(meta_value, Some("dark".to_string()));
    assert_eq!(direct_value, None);
    
    println!("✓ Step 4 passed: Storage uses meta-prefixed project names correctly");
}