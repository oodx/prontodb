use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use prontodb::cursor::{CursorManager, CursorData};
use prontodb::cursor_cache::CursorCache;
use prontodb::xdg::XdgPaths;

/// Test helper to create isolated test environment
struct UserIsolationTestEnv {
    _temp_dir: TempDir,
    xdg_paths: XdgPaths,
    cursor_manager: CursorManager,
    cursor_cache: CursorCache,
}

impl UserIsolationTestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_path_buf();
        
        // Set XDG environment variables to isolate this test
        std::env::set_var("XDG_DATA_HOME", temp_path.join("data"));
        std::env::set_var("XDG_CONFIG_HOME", temp_path.join("config"));
        std::env::set_var("XDG_CACHE_HOME", temp_path.join("cache"));
        std::env::set_var("HOME", &temp_path);
        
        // Create isolated XDG paths for testing
        let xdg_paths = XdgPaths::new(); // Will use the environment variables we just set
        
        let cursor_manager = CursorManager::new(); // Will use XDG paths from environment
        let cursor_cache = CursorCache::new(); // Will use XDG paths from environment
        
        Self {
            _temp_dir: temp_dir,
            xdg_paths,
            cursor_manager,
            cursor_cache,
        }
    }
}

#[test]
fn test_user_specific_cursor_creation_and_retrieval() {
    let env = UserIsolationTestEnv::new();
    
    // Create cursors for different users
    env.cursor_manager.set_cursor_with_meta(
        "work_cursor",
        PathBuf::from("/path/to/alice_work.db"),
        "alice",
        Some("alice_company".to_string()),
        Some("project1".to_string()),
        Some("config".to_string()),
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "work_cursor", // Same name, different user
        PathBuf::from("/path/to/bob_work.db"),
        "bob",
        Some("bob_company".to_string()),
        Some("project2".to_string()),
        Some("settings".to_string()),
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "dev_cursor",
        PathBuf::from("/path/to/alice_dev.db"),
        "alice",
        Some("alice_dev_org".to_string()),
        None,
        None,
    );
    
    // Test that users can only retrieve their own cursors
    let alice_work = env.cursor_manager.get_cursor("work_cursor", "alice").unwrap();
    assert_eq!(alice_work.database_path, PathBuf::from("/path/to/alice_work.db"));
    assert_eq!(alice_work.meta_context, Some("alice_company".to_string()));
    assert_eq!(alice_work.user, "alice");
    
    let bob_work = env.cursor_manager.get_cursor("work_cursor", "bob").unwrap();
    assert_eq!(bob_work.database_path, PathBuf::from("/path/to/bob_work.db"));
    assert_eq!(bob_work.meta_context, Some("bob_company".to_string()));
    assert_eq!(bob_work.user, "bob");
    
    // Test that users cannot access other users' cursors
    assert!(env.cursor_manager.get_cursor("work_cursor", "charlie").is_err());
    assert!(env.cursor_manager.get_cursor("dev_cursor", "bob").is_err());
    
    // Test that alice can access her dev_cursor but bob cannot
    let alice_dev = env.cursor_manager.get_cursor("dev_cursor", "alice").unwrap();
    assert_eq!(alice_dev.meta_context, Some("alice_dev_org".to_string()));
    assert!(env.cursor_manager.get_cursor("dev_cursor", "bob").is_err());
}

#[test]
fn test_cursor_listing_isolation() {
    let env = UserIsolationTestEnv::new();
    
    // Create cursors for multiple users
    env.cursor_manager.set_cursor_with_meta(
        "alice_cursor1",
        PathBuf::from("/alice1.db"),
        "alice",
        Some("alice_meta1".to_string()),
        None, None,
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "alice_cursor2", 
        PathBuf::from("/alice2.db"),
        "alice",
        None, // No meta context
        None, None,
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "bob_cursor1",
        PathBuf::from("/bob1.db"),
        "bob",
        Some("bob_meta1".to_string()),
        None, None,
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "charlie_cursor1",
        PathBuf::from("/charlie1.db"), 
        "charlie",
        Some("charlie_meta1".to_string()),
        None, None,
    );
    
    // Test that each user only sees their own cursors
    let alice_cursors = env.cursor_manager.list_cursors("alice").unwrap();
    assert_eq!(alice_cursors.len(), 2);
    assert!(alice_cursors.contains_key("alice_cursor1"));
    assert!(alice_cursors.contains_key("alice_cursor2"));
    assert!(!alice_cursors.contains_key("bob_cursor1"));
    assert!(!alice_cursors.contains_key("charlie_cursor1"));
    
    let bob_cursors = env.cursor_manager.list_cursors("bob").unwrap();
    assert_eq!(bob_cursors.len(), 1);
    assert!(bob_cursors.contains_key("bob_cursor1"));
    assert!(!bob_cursors.contains_key("alice_cursor1"));
    assert!(!bob_cursors.contains_key("alice_cursor2"));
    assert!(!bob_cursors.contains_key("charlie_cursor1"));
    
    let charlie_cursors = env.cursor_manager.list_cursors("charlie").unwrap();
    assert_eq!(charlie_cursors.len(), 1);
    assert!(charlie_cursors.contains_key("charlie_cursor1"));
    assert!(!charlie_cursors.contains_key("alice_cursor1"));
    assert!(!charlie_cursors.contains_key("bob_cursor1"));
    
    // Test that a user with no cursors gets empty list
    let david_cursors = env.cursor_manager.list_cursors("david").unwrap();
    assert_eq!(david_cursors.len(), 0);
}

#[test]
fn test_meta_namespace_integration_with_user_contexts() {
    let env = UserIsolationTestEnv::new();
    
    // Create cursors with same meta context but different users
    env.cursor_manager.set_cursor_with_meta(
        "org_cursor",
        PathBuf::from("/alice_org.db"),
        "alice",
        Some("shared_organization".to_string()),
        Some("shared_project".to_string()),
        Some("config".to_string()),
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "org_cursor", // Same name and meta context, different user
        PathBuf::from("/bob_org.db"),
        "bob", 
        Some("shared_organization".to_string()),
        Some("shared_project".to_string()),
        Some("config".to_string()),
    );
    
    // Verify each user gets their own cursor despite same meta context
    let alice_cursor = env.cursor_manager.get_cursor("org_cursor", "alice").unwrap();
    let bob_cursor = env.cursor_manager.get_cursor("org_cursor", "bob").unwrap();
    
    assert_eq!(alice_cursor.database_path, PathBuf::from("/alice_org.db"));
    assert_eq!(bob_cursor.database_path, PathBuf::from("/bob_org.db"));
    
    // Both should have same meta context but different database paths
    assert_eq!(alice_cursor.meta_context, Some("shared_organization".to_string()));
    assert_eq!(bob_cursor.meta_context, Some("shared_organization".to_string()));
    
    // Verify user isolation is maintained
    assert_eq!(alice_cursor.user, "alice");
    assert_eq!(bob_cursor.user, "bob");
}

#[test]
fn test_cursor_deletion_user_isolation() {
    let env = UserIsolationTestEnv::new();
    
    // Create cursors for different users with same name
    env.cursor_manager.set_cursor_with_meta(
        "shared_name",
        PathBuf::from("/alice_shared.db"),
        "alice",
        Some("alice_meta".to_string()),
        None, None,
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "shared_name",
        PathBuf::from("/bob_shared.db"),
        "bob",
        Some("bob_meta".to_string()),
        None, None,
    );
    
    // Verify both cursors exist
    assert!(env.cursor_manager.get_cursor("shared_name", "alice").is_ok());
    assert!(env.cursor_manager.get_cursor("shared_name", "bob").is_ok());
    
    // Delete alice's cursor
    let deleted = env.cursor_manager.delete_cursor("shared_name", "alice").unwrap();
    assert!(deleted);
    
    // Verify alice's cursor is gone but bob's remains
    assert!(env.cursor_manager.get_cursor("shared_name", "alice").is_err());
    assert!(env.cursor_manager.get_cursor("shared_name", "bob").is_ok());
    
    // Verify bob cannot delete alice's (already deleted) cursor
    let not_deleted = env.cursor_manager.delete_cursor("shared_name", "alice").unwrap();
    assert!(!not_deleted);
    
    // Bob can still delete his own cursor
    let bob_deleted = env.cursor_manager.delete_cursor("shared_name", "bob").unwrap();
    assert!(bob_deleted);
    assert!(env.cursor_manager.get_cursor("shared_name", "bob").is_err());
}

#[test]
fn test_default_user_isolation() {
    let env = UserIsolationTestEnv::new();
    
    // Create cursors for system default user and named users  
    env.cursor_manager.set_cursor_with_meta(
        "system_cursor",
        PathBuf::from("/system.db"),
        "default", // This is the internal system default, not a user named "default"
        Some("system_meta".to_string()),
        None, None,
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "alice_cursor", // Different name for alice
        PathBuf::from("/alice_system.db"),
        "alice",
        Some("alice_meta".to_string()),
        None, None,
    );
    
    // Verify the cursors exist
    let system_cursor = env.cursor_manager.get_cursor("system_cursor", "default").unwrap();
    let alice_cursor = env.cursor_manager.get_cursor("alice_cursor", "alice").unwrap();
    
    assert_eq!(system_cursor.database_path, PathBuf::from("/system.db"));
    assert_eq!(alice_cursor.database_path, PathBuf::from("/alice_system.db"));
    
    assert_eq!(system_cursor.meta_context, Some("system_meta".to_string()));
    assert_eq!(alice_cursor.meta_context, Some("alice_meta".to_string()));
    
    // Test listing isolation
    let system_cursors = env.cursor_manager.list_cursors("default").unwrap();
    let alice_cursors = env.cursor_manager.list_cursors("alice").unwrap();
    
    assert_eq!(system_cursors.len(), 1);
    assert_eq!(alice_cursors.len(), 1);
    assert!(system_cursors.contains_key("system_cursor"));
    assert!(alice_cursors.contains_key("alice_cursor"));
    
    // They should be completely different cursors
    assert_ne!(system_cursor.database_path, alice_cursor.database_path);
}

#[test]
fn test_cursor_file_naming_with_users() {
    let env = UserIsolationTestEnv::new();
    
    // Create cursors and verify file naming follows user suffix pattern
    env.cursor_manager.set_cursor_with_meta(
        "test_cursor",
        PathBuf::from("/test.db"),
        "alice",
        Some("test_meta".to_string()),
        None, None,
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "test_cursor",
        PathBuf::from("/test.db"),
        "default",
        Some("test_meta".to_string()),
        None, None,
    );
    
    // Check that cursor files are created with proper user suffixes
    // For database-scoped storage, files should be in: 
    // data_dir/test/cursors/test_cursor.alice.cursor
    // data_dir/test/cursors/test_cursor.cursor (for default user)
    
    let test_db_cursor_dir = env.xdg_paths.data_dir.join("test").join("cursors");
    
    let alice_cursor_file = test_db_cursor_dir.join("test_cursor.alice.cursor");
    let default_cursor_file = test_db_cursor_dir.join("test_cursor.cursor");
    
    assert!(alice_cursor_file.exists(), "Alice's cursor file should exist");
    assert!(default_cursor_file.exists(), "Default cursor file should exist");
    
    // Verify file contents have correct user
    let alice_content = fs::read_to_string(&alice_cursor_file).unwrap();
    let default_content = fs::read_to_string(&default_cursor_file).unwrap();
    
    let alice_data: CursorData = serde_json::from_str(&alice_content).unwrap();
    let default_data: CursorData = serde_json::from_str(&default_content).unwrap();
    
    assert_eq!(alice_data.user, "alice");
    assert_eq!(default_data.user, "default");
}

#[test]
fn test_cross_user_access_prevention() {
    let env = UserIsolationTestEnv::new();
    
    // Create sensitive cursors for different users
    env.cursor_manager.set_cursor_with_meta(
        "production_db",
        PathBuf::from("/secure/prod_alice.db"),
        "alice",
        Some("production_org".to_string()),
        Some("critical_project".to_string()),
        Some("prod_config".to_string()),
    );
    
    env.cursor_manager.set_cursor_with_meta(
        "dev_secrets",
        PathBuf::from("/secure/dev_bob.db"),
        "bob",
        Some("dev_org".to_string()),
        Some("secret_project".to_string()),
        Some("secrets".to_string()),
    );
    
    // Verify cross-user access is completely blocked
    assert!(env.cursor_manager.get_cursor("production_db", "bob").is_err());
    assert!(env.cursor_manager.get_cursor("production_db", "charlie").is_err());
    assert!(env.cursor_manager.get_cursor("production_db", "default").is_err());
    
    assert!(env.cursor_manager.get_cursor("dev_secrets", "alice").is_err());
    assert!(env.cursor_manager.get_cursor("dev_secrets", "charlie").is_err());
    assert!(env.cursor_manager.get_cursor("dev_secrets", "default").is_err());
    
    // Verify legitimate access still works
    assert!(env.cursor_manager.get_cursor("production_db", "alice").is_ok());
    assert!(env.cursor_manager.get_cursor("dev_secrets", "bob").is_ok());
    
    // Verify cross-user deletion attempts fail
    assert!(!env.cursor_manager.delete_cursor("production_db", "bob").unwrap());
    assert!(!env.cursor_manager.delete_cursor("dev_secrets", "alice").unwrap());
    
    // Verify cursors still exist after failed deletion attempts
    assert!(env.cursor_manager.get_cursor("production_db", "alice").is_ok());
    assert!(env.cursor_manager.get_cursor("dev_secrets", "bob").is_ok());
}