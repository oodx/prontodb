// RSB-compliant configuration tests - string-first testing approach
// Tests the public API functions and helper functions, not internal types

use rsb::prelude::*;
use tempfile::TempDir;

#[test]
fn test_config_default_values() {
    // RSB Pattern: Test string-first public interfaces
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    
    // Test the actual RSB helper functions that are exposed
    assert_eq!(prontodb::config::_helper_get_namespace_delimiter(), ".");
    assert_eq!(prontodb::config::_helper_get_busy_timeout_ms(), 5000);
    assert_eq!(prontodb::config::_helper_is_security_required(), true);
    
    let (admin_user, admin_pass) = prontodb::config::_helper_get_admin_credentials();
    assert_eq!(admin_user, "admin");
    assert_eq!(admin_pass, "pronto!");
    
    unset_var("HOME");
}

#[test]
fn test_config_initialization_succeeds() {
    // RSB Pattern: Test the public do_* functions
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    
    // Test the public RSB interface, not internal types
    let result = prontodb::config::do_init_config();
    assert_eq!(result, 0, "Configuration initialization should succeed");
    
    let result = prontodb::config::do_show_config();
    assert_eq!(result, 0, "Configuration display should succeed");
    
    unset_var("HOME");
}

#[test]
fn test_config_db_path_environment_override() {
    // RSB Pattern: Test param!() macro behavior
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    set_var("PRONTO_DB", "/custom/path/db.sqlite");
    
    let db_path = prontodb::config::_helper_get_db_path();
    assert_eq!(db_path, "/custom/path/db.sqlite");
    
    unset_var("PRONTO_DB");
    unset_var("HOME");
}

#[test]
fn test_config_security_environment_override() {
    // RSB Pattern: Use set_var/param! instead of env::set_var
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    set_var("PRONTO_SECURITY", "false");
    
    assert_eq!(prontodb::config::_helper_is_security_required(), false);
    
    unset_var("PRONTO_SECURITY");
    unset_var("HOME");
}

#[test]
fn test_config_admin_password_override() {
    // RSB Pattern: Test environment variable override
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    let custom_pass = "super_secret_123";
    set_var("PRONTO_ADMIN_PASS", custom_pass);
    
    let (_admin_user, admin_pass) = prontodb::config::_helper_get_admin_credentials();
    assert_eq!(admin_pass, custom_pass);
    
    unset_var("PRONTO_ADMIN_PASS");
    unset_var("HOME");
}

#[test]
fn test_config_namespace_delimiter_override() {
    // RSB Pattern: Test parameter expansion with custom delimiter
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    set_var("PRONTO_NS_DELIM", ":");
    
    assert_eq!(prontodb::config::_helper_get_namespace_delimiter(), ":");
    
    unset_var("PRONTO_NS_DELIM");
    unset_var("HOME");
}

#[test]
fn test_config_busy_timeout_override() {
    // RSB Pattern: Test numeric parameter with validation
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    set_var("PRONTO_BUSY_TIMEOUT_MS", "8000");
    
    assert_eq!(prontodb::config::_helper_get_busy_timeout_ms(), 8000);
    
    unset_var("PRONTO_BUSY_TIMEOUT_MS");
    unset_var("HOME");
}

#[test]
fn test_config_load_with_custom_path() {
    // RSB Pattern: String-based configuration file testing
    let temp_dir = TempDir::new().unwrap();
    let custom_config = temp_dir.path().join("custom.conf");
    
    // Create custom config file with RSB file operations
    let config_content = r#"ns_delim="|"
security.required=false
busy_timeout_ms=10000"#;
    
    write_file(custom_config.to_str().unwrap(), config_content);
    set_var("HOME", temp_dir.path().to_str().unwrap());
    
    // Test loading from custom path
    let result = prontodb::config::do_load_config(Some(custom_config.to_str().unwrap()));
    assert_eq!(result, 0, "Custom config loading should succeed");
    
    // Verify values were loaded (RSB environment should be updated)
    assert_eq!(prontodb::config::_helper_get_namespace_delimiter(), "|");
    assert_eq!(prontodb::config::_helper_is_security_required(), false);
    assert_eq!(prontodb::config::_helper_get_busy_timeout_ms(), 10000);
    
    unset_var("HOME");
}

#[test]
fn test_config_all_values_retrieval() {
    // RSB Pattern: Test the string-first API for retrieving all config
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    
    let config_map = prontodb::config::_helper_get_all_config().unwrap();
    
    // Verify expected keys exist in the configuration map
    assert!(config_map.contains_key("database_path"));
    assert!(config_map.contains_key("namespace_delimiter"));
    assert!(config_map.contains_key("security_required"));
    assert!(config_map.contains_key("busy_timeout_ms"));
    assert!(config_map.contains_key("admin_username"));
    assert!(config_map.contains_key("admin_password"));
    
    // Verify default values
    assert_eq!(config_map.get("namespace_delimiter").unwrap(), ".");
    assert_eq!(config_map.get("security_required").unwrap(), "true");
    assert_eq!(config_map.get("busy_timeout_ms").unwrap(), "5000");
    assert_eq!(config_map.get("admin_password").unwrap(), "[REDACTED]");
    
    unset_var("HOME");
}

#[test]
fn test_config_load_invalid_file_fails() {
    // RSB Pattern: Test error conditions with string-first API
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    
    let nonexistent_file = temp_dir.path().join("nonexistent.conf");
    
    // Should fail gracefully when file doesn't exist
    let result = prontodb::config::do_load_config(Some(nonexistent_file.to_str().unwrap()));
    assert_ne!(result, 0, "Loading nonexistent config should fail");
    
    unset_var("HOME");
}

#[test]
fn test_config_structure_creation() {
    // RSB Pattern: Test the helper function for directory creation
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    
    let result = prontodb::config::_helper_ensure_config_structure();
    assert!(result.is_ok(), "Config structure creation should succeed");
    
    // Verify the expected directory structure was created
    let config_dir = temp_dir.path().join(".local").join("etc").join("odx").join("prontodb");
    let data_dir = temp_dir.path().join(".local").join("data").join("odx").join("prontodb");
    
    assert!(test!(-d config_dir.to_str().unwrap()), "Config directory should exist");
    assert!(test!(-d data_dir.to_str().unwrap()), "Data directory should exist");
    
    // Verify default config file was created
    let config_file = config_dir.join("pronto.conf");
    assert!(test!(-f config_file.to_str().unwrap()), "Default config file should exist");
    
    unset_var("HOME");
}