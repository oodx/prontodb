// Name validation utilities for ProntoDB
// Provides consistent validation for usernames, database names, and other identifiers

/// Reserved words that cannot be used as names
const RESERVED_WORDS: &[&str] = &[
    "default", "pronto", "prontodb", "pdb", "main", "rust", "user", "name",
    "config", "cache", "data", "temp", "tmp", "system", "admin", "root",
    "database", "db", "storage", "cursor", "meta", "namespace", "project"
];

/// Validation error types
#[derive(Debug, PartialEq)]
pub enum ValidationError {
    Empty,
    Reserved(String),
    StartsWithNumber,
    InvalidCharacters,
    TooLong(usize),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Empty => write!(f, "Name cannot be empty"),
            ValidationError::Reserved(name) => write!(f, "'{}' is a reserved name. Please choose a different name", name),
            ValidationError::StartsWithNumber => write!(f, "Name cannot start with a number"),
            ValidationError::InvalidCharacters => write!(f, "Name must contain only alphanumeric characters (a-z, A-Z, 0-9)"),
            ValidationError::TooLong(max) => write!(f, "Name is too long. Maximum length is {} characters", max),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a name (username, database name, etc.)
/// 
/// Rules:
/// - Cannot be empty
/// - Cannot be a reserved word (case insensitive)
/// - Cannot start with a number
/// - Must contain only alphanumeric characters
/// - Must not exceed max_length (if specified)
pub fn validate_name(name: &str, max_length: Option<usize>) -> Result<(), ValidationError> {
    // Check for empty name
    if name.is_empty() {
        return Err(ValidationError::Empty);
    }
    
    // Check length limit
    if let Some(max_len) = max_length {
        if name.len() > max_len {
            return Err(ValidationError::TooLong(max_len));
        }
    }
    
    // Check for reserved words (case insensitive)
    if RESERVED_WORDS.iter().any(|&reserved| name.to_lowercase() == reserved) {
        return Err(ValidationError::Reserved(name.to_string()));
    }
    
    // Check if starts with number
    if name.chars().next().unwrap().is_ascii_digit() {
        return Err(ValidationError::StartsWithNumber);
    }
    
    // Check for alphanumeric only
    if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ValidationError::InvalidCharacters);
    }
    
    Ok(())
}

/// Validate a username (convenience wrapper with username-specific max length)
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    validate_name(username, Some(32)) // 32 char limit for usernames
}

/// Validate a database name (convenience wrapper with database-specific max length)
#[allow(dead_code)]  // Future feature for database name validation
pub fn validate_database_name(db_name: &str) -> Result<(), ValidationError> {
    validate_name(db_name, Some(64)) // 64 char limit for database names
}

/// Validate a project name (convenience wrapper)
pub fn validate_project_name(project_name: &str) -> Result<(), ValidationError> {
    validate_name(project_name, Some(64))
}

/// Validate a namespace name (convenience wrapper)
#[allow(dead_code)]  // Future feature for namespace validation
pub fn validate_namespace_name(namespace_name: &str) -> Result<(), ValidationError> {
    validate_name(namespace_name, Some(64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_name("alice", None).is_ok());
        assert!(validate_name("user123", None).is_ok());
        assert!(validate_name("myApp", None).is_ok());
        assert!(validate_name("a", None).is_ok());
        assert!(validate_name("A1B2C3", None).is_ok());
    }

    #[test]
    fn test_empty_name() {
        assert_eq!(validate_name("", None), Err(ValidationError::Empty));
    }

    #[test]
    fn test_reserved_words() {
        assert_eq!(validate_name("default", None), Err(ValidationError::Reserved("default".to_string())));
        assert_eq!(validate_name("DEFAULT", None), Err(ValidationError::Reserved("DEFAULT".to_string()))); // case insensitive
        assert_eq!(validate_name("prontodb", None), Err(ValidationError::Reserved("prontodb".to_string())));
        assert_eq!(validate_name("system", None), Err(ValidationError::Reserved("system".to_string())));
    }

    #[test]
    fn test_starts_with_number() {
        assert_eq!(validate_name("123user", None), Err(ValidationError::StartsWithNumber));
        assert_eq!(validate_name("0alice", None), Err(ValidationError::StartsWithNumber));
    }

    #[test]
    fn test_invalid_characters() {
        assert_eq!(validate_name("user-name", None), Err(ValidationError::InvalidCharacters));
        assert_eq!(validate_name("user_name", None), Err(ValidationError::InvalidCharacters));
        assert_eq!(validate_name("user@company", None), Err(ValidationError::InvalidCharacters));
        assert_eq!(validate_name("user name", None), Err(ValidationError::InvalidCharacters));
    }

    #[test]
    fn test_length_limits() {
        let long_name = "a".repeat(100);
        assert_eq!(validate_name(&long_name, Some(50)), Err(ValidationError::TooLong(50)));
        assert!(validate_name(&long_name, Some(150)).is_ok());
        assert!(validate_name(&long_name, None).is_ok()); // No limit
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("alice").is_ok());
        assert_eq!(validate_username("default"), Err(ValidationError::Reserved("default".to_string())));
        
        let long_username = "a".repeat(50);
        assert_eq!(validate_username(&long_username), Err(ValidationError::TooLong(32)));
    }

    #[test]
    fn test_database_name_validation() {
        assert!(validate_database_name("mydb").is_ok());
        assert_eq!(validate_database_name("main"), Err(ValidationError::Reserved("main".to_string())));
        
        let long_db_name = "a".repeat(100);
        assert_eq!(validate_database_name(&long_db_name), Err(ValidationError::TooLong(64)));
    }
}