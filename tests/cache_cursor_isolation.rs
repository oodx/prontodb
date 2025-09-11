use prontodb::cursor_cache::CursorCache;
use tempfile::TempDir;

/// Test helper for isolated cache cursor testing
struct CacheCursorTestEnv {
    _temp_dir: TempDir,
    cache: CursorCache,
}

impl CacheCursorTestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_path_buf();
        
        // Set XDG environment variables to isolate this test
        std::env::set_var("HOME", &temp_path);
        
        // Create isolated cursor cache using XDG isolation
        let cache_dir = temp_path.join(".local").join("etc").join("prontodb");
        let cache = CursorCache::from_dir(cache_dir);
        
        Self {
            _temp_dir: temp_dir,
            cache,
        }
    }
}

#[test]
fn test_cache_cursor_user_isolation() {
    let env = CacheCursorTestEnv::new();
    
    // Set cache cursors for different users
    env.cache.set_cursor("alice_db.db", Some("alice")).unwrap();
    env.cache.set_cursor("bob_db.db", Some("bob")).unwrap();
    env.cache.set_cursor("default_db.db", None).unwrap(); // Default user
    env.cache.set_cursor("charlie_db.db", Some("charlie")).unwrap();
    
    // Verify each user can only access their own cache cursor
    assert_eq!(env.cache.get_cursor(Some("alice")), Some("alice_db.db".to_string()));
    assert_eq!(env.cache.get_cursor(Some("bob")), Some("bob_db.db".to_string()));
    assert_eq!(env.cache.get_cursor(None), Some("default_db.db".to_string()));
    assert_eq!(env.cache.get_cursor(Some("charlie")), Some("charlie_db.db".to_string()));
    
    // Verify cross-user access returns None (not other users' cursors)
    assert_eq!(env.cache.get_cursor(Some("david")), None); // Non-existent user
    
    // Verify users cannot see other users' cache cursors through get_cursor
    // (Each user should only get their own cursor or None)
    assert_ne!(env.cache.get_cursor(Some("alice")), Some("bob_db.db".to_string()));
    assert_ne!(env.cache.get_cursor(Some("bob")), Some("alice_db.db".to_string()));
    assert_ne!(env.cache.get_cursor(Some("alice")), Some("default_db.db".to_string()));
}

#[test]
fn test_cache_cursor_clearing_isolation() {
    let env = CacheCursorTestEnv::new();
    
    // Set cursors for multiple users
    env.cache.set_cursor("alice_work.db", Some("alice")).unwrap();
    env.cache.set_cursor("bob_work.db", Some("bob")).unwrap();
    env.cache.set_cursor("default_work.db", None).unwrap();
    
    // Clear alice's cursor
    env.cache.clear_cursor(Some("alice")).unwrap();
    
    // Verify only alice's cursor is cleared
    assert_eq!(env.cache.get_cursor(Some("alice")), None);
    assert_eq!(env.cache.get_cursor(Some("bob")), Some("bob_work.db".to_string()));
    assert_eq!(env.cache.get_cursor(None), Some("default_work.db".to_string()));
    
    // Clear default cursor
    env.cache.clear_cursor(None).unwrap();
    
    // Verify only default cursor is cleared
    assert_eq!(env.cache.get_cursor(None), None);
    assert_eq!(env.cache.get_cursor(Some("bob")), Some("bob_work.db".to_string()));
}

#[test]
fn test_cache_cursor_overwriting_user_specific() {
    let env = CacheCursorTestEnv::new();
    
    // Set initial cursor for alice
    env.cache.set_cursor("alice_initial.db", Some("alice")).unwrap();
    assert_eq!(env.cache.get_cursor(Some("alice")), Some("alice_initial.db".to_string()));
    
    // Overwrite alice's cursor
    env.cache.set_cursor("alice_updated.db", Some("alice")).unwrap();
    assert_eq!(env.cache.get_cursor(Some("alice")), Some("alice_updated.db".to_string()));
    
    // Set cursor for bob (should not affect alice)
    env.cache.set_cursor("bob_db.db", Some("bob")).unwrap();
    assert_eq!(env.cache.get_cursor(Some("alice")), Some("alice_updated.db".to_string()));
    assert_eq!(env.cache.get_cursor(Some("bob")), Some("bob_db.db".to_string()));
}

#[test]
fn test_default_user_cache_isolation() {
    let env = CacheCursorTestEnv::new();
    
    // Set cursors for default user (None) and named user "systemuser"
    env.cache.set_cursor("system_default.db", None).unwrap();
    env.cache.set_cursor("user_specific.db", Some("systemuser")).unwrap();
    
    // Verify they are isolated (different cursors)
    assert_eq!(env.cache.get_cursor(None), Some("system_default.db".to_string()));
    assert_eq!(env.cache.get_cursor(Some("systemuser")), Some("user_specific.db".to_string()));
    
    // Clear one should not affect the other
    env.cache.clear_cursor(None).unwrap();
    assert_eq!(env.cache.get_cursor(None), None);
    assert_eq!(env.cache.get_cursor(Some("systemuser")), Some("user_specific.db".to_string()));
}

#[test]
fn test_cache_cursor_edge_cases() {
    let env = CacheCursorTestEnv::new();
    
    // Test empty username
    env.cache.set_cursor("empty_user.db", Some("")).unwrap();
    assert_eq!(env.cache.get_cursor(Some("")), Some("empty_user.db".to_string()));
    
    // Test username with special characters
    env.cache.set_cursor("special_user.db", Some("user@company.com")).unwrap();
    assert_eq!(env.cache.get_cursor(Some("user@company.com")), Some("special_user.db".to_string()));
    
    // Test long username
    let long_user = "a".repeat(100);
    env.cache.set_cursor("long_user.db", Some(&long_user)).unwrap();
    assert_eq!(env.cache.get_cursor(Some(&long_user)), Some("long_user.db".to_string()));
    
    // Verify isolation is maintained
    assert_eq!(env.cache.get_cursor(Some("")), Some("empty_user.db".to_string()));
    assert_eq!(env.cache.get_cursor(Some("user@company.com")), Some("special_user.db".to_string()));
    assert_eq!(env.cache.get_cursor(Some(&long_user)), Some("long_user.db".to_string()));
    assert_eq!(env.cache.get_cursor(Some("other")), None);
}

#[test] 
fn test_cache_list_all_cursors_admin_function() {
    let env = CacheCursorTestEnv::new();
    
    // Set cursors for multiple users
    env.cache.set_cursor("alice_db.db", Some("alice")).unwrap();
    env.cache.set_cursor("bob_db.db", Some("bob")).unwrap();
    env.cache.set_cursor("default_db.db", None).unwrap();
    
    // list_all_cursors should return all cursors (for admin purposes)
    let all_cursors = env.cache.list_all_cursors();
    
    // Verify all cursors are present
    assert!(all_cursors.iter().any(|(user, db)| user == "alice" && db == "alice_db.db"));
    assert!(all_cursors.iter().any(|(user, db)| user == "bob" && db == "bob_db.db"));
    assert!(all_cursors.iter().any(|(user, db)| user == "default" && db == "default_db.db"));
    
    // Should be exactly 3 cursors
    assert_eq!(all_cursors.len(), 3);
}