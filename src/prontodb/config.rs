// ProntoDB Configuration Module - RSB Compliant
// Handles configuration loading following RSB three-tier ordinality

use rsb::prelude::*;
use std::fs;
use std::collections::HashMap;

// =============================================================================
// PUBLIC API - User-facing configuration functions (do_*)
// =============================================================================

/// Initialize configuration system with defaults
pub fn do_init_config() -> i32 {
    validate_config_environment();
    
    match _helper_ensure_config_structure() {
        Ok(_) => {
            okay!("Configuration system initialized");
            0
        }
        Err(e) => {
            fatal!("Failed to initialize configuration: {}", e);
            1
        }
    }
}

/// Load configuration file with validation
pub fn do_load_config(config_path: Option<&str>) -> i32 {
    validate!(!config_path.unwrap_or("").is_empty() || config_path.is_none(), 
             "Configuration path cannot be empty");
    
    match _helper_load_config(config_path) {
        Ok(_) => {
            okay!("Configuration loaded successfully");
            0
        }
        Err(e) => {
            fatal!("Failed to load configuration: {}", e);
            1
        }
    }
}

/// Display current configuration values
pub fn do_show_config() -> i32 {
    match _helper_get_all_config() {
        Ok(config_map) => {
            for (key, value) in config_map {
                echo!("{}={}", key, value);
            }
            0
        }
        Err(e) => {
            error!("Failed to retrieve configuration: {}", e);
            1
        }
    }
}

// =============================================================================
// HELPER TIER - Business logic, assumes valid inputs (_helper_*)
// =============================================================================

/// Ensure configuration directory structure exists
pub fn _helper_ensure_config_structure() -> Result<(), String> {
    let home = param!("HOME");
    require_var!("HOME");
    
    let config_dir = _get_config_dir(&home);
    let data_dir = _get_data_dir(&home);
    
    super::utils::__blind_faith_create_dirs(&config_dir)?;
    super::utils::__blind_faith_create_dirs(&data_dir)?;
    
    let config_file = format!("{}/pronto.conf", config_dir);
    if !test!(-f &config_file) {
        _helper_create_default_config(&config_file)?;
    }
    
    Ok(())
}

/// Load configuration from file or defaults
pub fn _helper_load_config(config_path: Option<&str>) -> Result<(), String> {
    let config_file = match config_path {
        Some(path) => path.to_string(),
        None => {
            let home = param!("HOME");
            let config_dir = _get_config_dir(&home);
            format!("{}/pronto.conf", config_dir)
        }
    };
    
    if test!(-f &config_file) {
        let content = __blind_faith_read_file(&config_file)?;
        _helper_parse_and_apply_config(&content)?;
    } else if config_path.is_some() {
        return Err(format!("Configuration file not found: {}", config_file));
    }
    
    // Apply environment variable overrides
    _helper_apply_env_overrides()?;
    
    Ok(())
}

/// Get database path with precedence: ENV > flag > config > default
pub fn _helper_get_db_path() -> String {
    // 1. Environment variable (highest precedence)
    let db_path = param!("PRONTO_DB", default: "");
    if !db_path.is_empty() {
        return db_path;
    }
    
    // 2. Configuration file value
    let config_db = param!("PRONTO_CONFIG_DB_PATH", default: "");
    if !config_db.is_empty() {
        return config_db;
    }
    
    // 3. Default XDG path
    let home = param!("HOME");
    let data_dir = _get_data_dir(&home);
    format!("{}/pronto.db", data_dir)
}

/// Get default admin credentials
pub fn _helper_get_admin_credentials() -> (String, String) {
    let username = param!("PRONTO_ADMIN_USER", default: "admin");
    let password = param!("PRONTO_ADMIN_PASS", default: "pronto!");
    
    (username, password)
}

/// Check if security is required
pub fn _helper_is_security_required() -> bool {
    // Environment override (PRONTO_SECURITY=false disables)
    let security_env = param!("PRONTO_SECURITY", default: "");
    if !security_env.is_empty() {
        return security_env.to_lowercase() != "false";
    }
    
    // Configuration file value
    let security_config = param!("PRONTO_CONFIG_SECURITY_REQUIRED", default: "");
    if !security_config.is_empty() {
        return security_config.to_lowercase() != "false";
    }
    
    // Default: security required
    true
}

/// Get namespace delimiter
pub fn _helper_get_namespace_delimiter() -> String {
    // Environment override
    let delim_env = param!("PRONTO_NS_DELIM", default: "");
    if !delim_env.is_empty() {
        return delim_env;
    }
    
    // Configuration value
    let delim_config = param!("PRONTO_CONFIG_NS_DELIM", default: "");
    if !delim_config.is_empty() {
        return delim_config;
    }
    
    // Default
    ".".to_string()
}

/// Get SQLite busy timeout in milliseconds
pub fn _helper_get_busy_timeout_ms() -> u32 {
    let timeout_str = param!("PRONTO_BUSY_TIMEOUT_MS", default: "5000");
    timeout_str.parse::<u32>().unwrap_or(5000)
}

/// Get all configuration as map
pub fn _helper_get_all_config() -> Result<HashMap<String, String>, String> {
    let mut config = HashMap::new();
    
    config.insert("database_path".to_string(), _helper_get_db_path());
    config.insert("namespace_delimiter".to_string(), _helper_get_namespace_delimiter());
    config.insert("security_required".to_string(), _helper_is_security_required().to_string());
    config.insert("busy_timeout_ms".to_string(), _helper_get_busy_timeout_ms().to_string());
    
    let (admin_user, _admin_pass) = _helper_get_admin_credentials();
    config.insert("admin_username".to_string(), admin_user);
    config.insert("admin_password".to_string(), "[REDACTED]".to_string());
    
    Ok(config)
}

/// Create default configuration file
pub fn _helper_create_default_config(config_file: &str) -> Result<(), String> {
    let default_config = r#"# ProntoDB Configuration
# XDG+ compliant configuration file

# Namespace delimiter (default: ".")
ns_delim="."

# Security settings
security.required=true

# Database settings
busy_timeout_ms=5000

# Path settings (leave empty to use XDG defaults)
# database_path=""
"#;
    
    __blind_faith_write_file(config_file, default_config)?;
    info!("Created default configuration: {}", config_file);
    
    Ok(())
}

/// Parse configuration content and apply to environment
pub fn _helper_parse_and_apply_config(content: &str) -> Result<(), String> {
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse key=value pairs
        if let Some((key, value)) = _helper_parse_config_line(line) {
            _helper_apply_config_value(&key, &value)?;
        }
    }
    
    Ok(())
}

/// Apply environment variable overrides
pub fn _helper_apply_env_overrides() -> Result<(), String> {
    // Map environment variables to internal configuration
    let env_mappings = vec![
        ("PRONTO_DB", "PRONTO_CONFIG_DB_PATH"),
        ("PRONTO_SECURITY", "PRONTO_CONFIG_SECURITY_REQUIRED"),
        ("PRONTO_NS_DELIM", "PRONTO_CONFIG_NS_DELIM"),
        ("PRONTO_BUSY_TIMEOUT_MS", "PRONTO_CONFIG_BUSY_TIMEOUT_MS"),
    ];
    
    for (env_var, config_var) in env_mappings {
        let value = param!(env_var, default: "");
        if !value.is_empty() {
            set_var(config_var, &value);
        }
    }
    
    Ok(())
}

// =============================================================================
// BLIND FAITH TIER - Low-level operations (__blind_faith_*)
// =============================================================================

/// Parse a single configuration line
pub fn _helper_parse_config_line(line: &str) -> Option<(String, String)> {
    if let Some(eq_pos) = line.find('=') {
        let key = line[..eq_pos].trim();
        let value = line[eq_pos + 1..].trim();
        
        // Remove quotes if present
        let clean_value = if (value.starts_with('"') && value.ends_with('"')) ||
                             (value.starts_with('\'') && value.ends_with('\'')) {
            &value[1..value.len() - 1]
        } else {
            value
        };
        
        Some((key.to_string(), clean_value.to_string()))
    } else {
        None
    }
}

/// Apply a single configuration value
pub fn _helper_apply_config_value(key: &str, value: &str) -> Result<(), String> {
    match key {
        "ns_delim" => set_var("PRONTO_CONFIG_NS_DELIM", value),
        "security.required" => set_var("PRONTO_CONFIG_SECURITY_REQUIRED", value),
        "busy_timeout_ms" => set_var("PRONTO_CONFIG_BUSY_TIMEOUT_MS", value),
        "database_path" => {
            if !value.is_empty() {
                set_var("PRONTO_CONFIG_DB_PATH", value);
            }
        },
        _ => {
            // Ignore unknown configuration keys with warning
            warn!("Unknown configuration key: {}", key);
        }
    }
    
    Ok(())
}

// =============================================================================
// UTILITY FUNCTIONS - Path management
// =============================================================================

fn _get_config_dir(home: &str) -> String {
    // Use XDG_CONFIG_HOME if set, otherwise default
    let xdg_config = param!("XDG_CONFIG_HOME", default: &format!("{}/.local/etc", home));
    format!("{}/odx/prontodb", xdg_config)
}

fn _get_data_dir(home: &str) -> String {
    // Use XDG_DATA_HOME if set, otherwise default  
    let xdg_data = param!("XDG_DATA_HOME", default: &format!("{}/.local/data", home));
    format!("{}/odx/prontodb", xdg_data)
}

fn validate_config_environment() {
    require_var!("HOME");
    
    // Validate that critical paths are accessible
    let home = param!("HOME");
    if home.is_empty() {
        fatal!("HOME environment variable is empty");
    }
    
    if !test!(-d &home) {
        fatal!("HOME directory does not exist: {}", home);
    }
}

// =============================================================================
// BLIND FAITH SYSTEM OPERATIONS - File I/O
// =============================================================================

fn __blind_faith_read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path, e))
}

fn __blind_faith_write_file(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content)
        .map_err(|e| format!("Failed to write file {}: {}", path, e))
}