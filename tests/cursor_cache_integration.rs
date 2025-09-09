// Integration tests for global cursor caching system
// Tests the complete cursor cache implementation including command line integration

use prontodb::{api, cursor_cache::CursorCache};
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Test utility to set up isolated environment
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("etc").join("prontodb");
    fs::create_dir_all(&cache_dir).unwrap();
    
    (temp_dir, cache_dir)
}

#[test]
fn test_cursor_cache_basic_functionality() {
    let (_temp, cache_dir) = setup_test_env();
    let cache = CursorCache::from_dir(cache_dir);
    
    // Test setting and getting global cursor
    cache.set_cursor("staging", None).unwrap();
    assert_eq!(cache.get_cursor(None), Some("staging".to_string()));
    
    // Test setting and getting user-specific cursor
    cache.set_cursor("prod", Some("alice")).unwrap();
    assert_eq!(cache.get_cursor(Some("alice")), Some("prod".to_string()));
    
    // Test default user behavior
    cache.set_cursor("test", Some("default")).unwrap();
    assert_eq!(cache.get_cursor(Some("default")), Some("test".to_string()));
    assert_eq!(cache.get_cursor(None), Some("test".to_string())); // should be same as default
}

#[test]
fn test_cursor_cache_multi_user_isolation() {
    let (_temp, cache_dir) = setup_test_env();
    let cache = CursorCache::from_dir(cache_dir);
    
    // Set different cursors for different users
    cache.set_cursor("dev", Some("alice")).unwrap();
    cache.set_cursor("staging", Some("bob")).unwrap();  
    cache.set_cursor("prod", None).unwrap(); // global
    
    // Verify isolation
    assert_eq!(cache.get_cursor(Some("alice")), Some("dev".to_string()));
    assert_eq!(cache.get_cursor(Some("bob")), Some("staging".to_string()));
    assert_eq!(cache.get_cursor(None), Some("prod".to_string()));
    assert_eq!(cache.get_cursor(Some("charlie")), None); // no cursor set
}

#[test]
fn test_cursor_cache_file_persistence() {
    let (_temp, cache_dir) = setup_test_env();
    
    // Set cursors in one cache instance
    {
        let cache = CursorCache::from_dir(cache_dir.clone());
        cache.set_cursor("persistent", None).unwrap();
        cache.set_cursor("user_persistent", Some("alice")).unwrap();
    }
    
    // Verify persistence in new cache instance
    {
        let cache = CursorCache::from_dir(cache_dir.clone());
        assert_eq!(cache.get_cursor(None), Some("persistent".to_string()));
        assert_eq!(cache.get_cursor(Some("alice")), Some("user_persistent".to_string()));
    }
}

#[test]
fn test_cursor_cache_list_functionality() {
    let (_temp, cache_dir) = setup_test_env();
    let cache = CursorCache::from_dir(cache_dir);
    
    // Initially empty
    assert!(cache.list_all_cursors().is_empty());
    
    // Set multiple cursors
    cache.set_cursor("staging", None).unwrap();
    cache.set_cursor("dev", Some("alice")).unwrap();
    cache.set_cursor("prod", Some("bob")).unwrap();
    
    let cursors = cache.list_all_cursors();
    assert_eq!(cursors.len(), 3);
    
    // Verify contents
    assert!(cursors.contains(&("default".to_string(), "staging".to_string())));
    assert!(cursors.contains(&("alice".to_string(), "dev".to_string())));
    assert!(cursors.contains(&("bob".to_string(), "prod".to_string())));
}

#[test]
fn test_cursor_cache_clear_functionality() {
    let (_temp, cache_dir) = setup_test_env();
    let cache = CursorCache::from_dir(cache_dir);
    
    // Set cursors
    cache.set_cursor("staging", None).unwrap();
    cache.set_cursor("dev", Some("alice")).unwrap();
    
    // Verify they exist
    assert!(cache.has_cursor(None));
    assert!(cache.has_cursor(Some("alice")));
    
    // Clear one cursor
    cache.clear_cursor(Some("alice")).unwrap();
    assert!(cache.has_cursor(None));
    assert!(!cache.has_cursor(Some("alice")));
    
    // Clear global cursor
    cache.clear_cursor(None).unwrap();
    assert!(!cache.has_cursor(None));
}

#[test]
fn test_database_scoped_cursor_integration() {
    let (_temp, cache_dir) = setup_test_env();
    let cache = CursorCache::from_dir(cache_dir);
    
    // Test that cursor cache works with database-scoped operations
    cache.set_cursor("myapp", None).unwrap();
    cache.set_cursor("staging", Some("alice")).unwrap();
    
    // Simulate API calls checking cursor cache
    let global_db = cache.get_cursor(None).unwrap();
    let alice_db = cache.get_cursor(Some("alice")).unwrap();
    
    assert_eq!(global_db, "myapp");
    assert_eq!(alice_db, "staging");
    
    // Test that different users get different databases
    assert_ne!(
        cache.get_cursor(None),
        cache.get_cursor(Some("alice"))
    );
}

#[test]
fn test_cursor_cache_error_handling() {
    let (_temp, cache_dir) = setup_test_env();
    let cache = CursorCache::from_dir(cache_dir);
    
    // Test handling of nonexistent cursors
    assert_eq!(cache.get_cursor(Some("nonexistent")), None);
    assert!(!cache.has_cursor(Some("nonexistent")));
    
    // Test clearing nonexistent cursors (should not error)
    cache.clear_cursor(Some("nonexistent")).unwrap();
    
    // Test empty database names
    cache.set_cursor("", None).unwrap();
    assert_eq!(cache.get_cursor(None), None); // Empty should be treated as None
}

#[test]
fn test_cursor_cache_edge_cases() {
    let (_temp, cache_dir) = setup_test_env();
    let cache = CursorCache::from_dir(cache_dir);
    
    // Test whitespace handling
    cache.set_cursor("  spaced_db  \n", None).unwrap();
    assert_eq!(cache.get_cursor(None), Some("spaced_db".to_string()));
    
    // Test special characters in database names
    cache.set_cursor("my-app.db", Some("alice")).unwrap();
    assert_eq!(cache.get_cursor(Some("alice")), Some("my-app.db".to_string()));
    
    // Test long database names
    let long_name = "a".repeat(100);
    cache.set_cursor(&long_name, Some("bob")).unwrap();
    assert_eq!(cache.get_cursor(Some("bob")), Some(long_name));
}

#[test]
fn test_cursor_cache_concurrent_access() {
    let (_temp, cache_dir) = setup_test_env();
    
    // Test that multiple cache instances can access the same cache directory
    let cache1 = CursorCache::from_dir(cache_dir.clone());
    let cache2 = CursorCache::from_dir(cache_dir.clone());
    
    // Set cursor with first instance
    cache1.set_cursor("concurrent", None).unwrap();
    
    // Read with second instance
    assert_eq!(cache2.get_cursor(None), Some("concurrent".to_string()));
    
    // Modify with second instance
    cache2.set_cursor("modified", None).unwrap();
    
    // Verify with first instance
    assert_eq!(cache1.get_cursor(None), Some("modified".to_string()));
}

#[test]
fn test_cursor_cache_xdg_location() {
    // Test that CursorCache uses the correct XDG location
    let cache = CursorCache::new();
    
    // Should create cache directory in HOME/.local/etc/prontodb/
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let expected_dir = PathBuf::from(&home)
        .join(".local")
        .join("etc")
        .join("prontodb");
    
    // Setting a cursor should create the directory structure
    cache.set_cursor("test_xdg", None).unwrap();
    
    // The cache directory should exist
    assert!(expected_dir.exists());
    
    // The cursor file should exist
    let cursor_file = expected_dir.join("cursor");
    assert!(cursor_file.exists());
    
    // Clean up
    let _ = fs::remove_file(&cursor_file);
}