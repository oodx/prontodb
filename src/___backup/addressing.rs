// Addressing module for parsing canonical paths
// Handles project.namespace.key__context format

#![allow(dead_code)]  // Some functions are used via pub api

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AddressContext {
    KeyAccess,   // get/set/del operations - traditional namespace.key  
    Discovery,   // keys/scan operations - project.namespace discovery
    Auto,        // Attempt intelligent detection
}

#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    pub project: String,
    pub namespace: String,
    pub key: String,
    pub context: Option<String>,
}

impl Address {
    // Context-aware parsing for resolving 2-part addressing ambiguity
    pub fn parse_with_context(path: &str, delimiter: &str, context: AddressContext) -> Result<Self, String> {
        // Handle context suffix if present
        let (base_path, ctx_suffix) = if let Some(idx) = path.rfind("__") {
            (&path[..idx], Some(path[idx+2..].to_string()))
        } else {
            (path, None)
        };

        let parts: Vec<&str> = base_path.split(delimiter).collect();
        
        match parts.len() {
            0 => Err("Empty path".to_string()),
            1 => {
                // Single part: treat as key in default project/namespace
                Ok(Address {
                    project: "default".to_string(),
                    namespace: "default".to_string(),
                    key: parts[0].to_string(),
                    context: ctx_suffix,
                })
            }
            2 => {
                // Two parts: context determines interpretation
                match context {
                    AddressContext::KeyAccess => {
                        // Traditional: namespace.key -> default.namespace.key
                        Ok(Address {
                            project: "default".to_string(),
                            namespace: parts[0].to_string(),
                            key: parts[1].to_string(),
                            context: ctx_suffix,
                        })
                    }
                    AddressContext::Discovery => {
                        // Discovery: project.namespace -> project.namespace.""
                        Ok(Address {
                            project: parts[0].to_string(),
                            namespace: parts[1].to_string(),
                            key: "".to_string(),
                            context: ctx_suffix,
                        })
                    }
                    AddressContext::Auto => {
                        // Default to KeyAccess for backward compatibility
                        Ok(Address {
                            project: "default".to_string(),
                            namespace: parts[0].to_string(),
                            key: parts[1].to_string(),
                            context: ctx_suffix,
                        })
                    }
                }
            }
            3 => {
                // Three parts: project.namespace.key (unambiguous)
                Ok(Address {
                    project: parts[0].to_string(),
                    namespace: parts[1].to_string(),
                    key: parts[2].to_string(),
                    context: ctx_suffix,
                })
            }
            _ => Err("Too many parts in path".to_string()),
        }
    }

    // Parse a canonical path with given delimiter (backward compatibility)
    pub fn parse(path: &str, delimiter: &str) -> Result<Self, String> {
        // Default to KeyAccess for existing callers
        Self::parse_with_context(path, delimiter, AddressContext::KeyAccess)
    }

    // Build address from components
    pub fn from_parts(
        project: Option<String>,
        namespace: Option<String>,
        key: String,
        context: Option<String>,
    ) -> Self {
        Address {
            project: project.unwrap_or_else(|| "default".to_string()),
            namespace: namespace.unwrap_or_else(|| "default".to_string()),
            key,
            context,
        }
    }

    // Validate that key doesn't contain delimiter
    pub fn validate_key(&self, delimiter: &str) -> Result<(), String> {
        if self.key.contains(delimiter) {
            Err(format!(
                "Key '{}' cannot contain delimiter '{}'",
                self.key, delimiter
            ))
        } else {
            Ok(())
        }
    }

    // Convert back to canonical path
    pub fn to_path(&self, delimiter: &str) -> String {
        let base = format!(
            "{}{}{}{}{}",
            self.project, delimiter, self.namespace, delimiter, self.key
        );
        
        if let Some(ctx) = &self.context {
            format!("{}__{}", base, ctx)
        } else {
            base
        }
    }

    // Get the storage key (for database)
    pub fn storage_key(&self) -> String {
        if let Some(ctx) = &self.context {
            format!("{}.{}.{}__{}", self.project, self.namespace, self.key, ctx)
        } else {
            format!("{}.{}.{}", self.project, self.namespace, self.key)
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_path("."))
    }
}

// Helper to parse address from various sources
pub fn parse_address(
    path: Option<&str>,
    project: Option<String>,
    namespace: Option<String>,
    key: Option<&str>,
    delimiter: &str,
) -> Result<Address, String> {
    if let Some(path) = path {
        // Full path provided
        Address::parse(path, delimiter)
    } else if let Some(key) = key {
        // Build from parts
        Ok(Address::from_parts(project, namespace, key.to_string(), None))
    } else {
        Err("No address specified".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_path() {
        let addr = Address::parse("proj.ns.key", ".").unwrap();
        assert_eq!(addr.project, "proj");
        assert_eq!(addr.namespace, "ns");
        assert_eq!(addr.key, "key");
        assert_eq!(addr.context, None);
    }

    #[test]
    fn test_parse_with_context() {
        let addr = Address::parse("proj.ns.key__ctx", ".").unwrap();
        assert_eq!(addr.project, "proj");
        assert_eq!(addr.namespace, "ns");
        assert_eq!(addr.key, "key");
        assert_eq!(addr.context, Some("ctx".to_string()));
    }

    #[test]
    fn test_parse_partial_paths() {
        let addr = Address::parse("key", ".").unwrap();
        assert_eq!(addr.project, "default");
        assert_eq!(addr.namespace, "default");
        assert_eq!(addr.key, "key");

        let addr = Address::parse("ns.key", ".").unwrap();
        assert_eq!(addr.project, "default");
        assert_eq!(addr.namespace, "ns");
        assert_eq!(addr.key, "key");
    }

    #[test]
    fn test_custom_delimiter() {
        let addr = Address::parse("proj|ns|key", "|").unwrap();
        assert_eq!(addr.project, "proj");
        assert_eq!(addr.namespace, "ns");
        assert_eq!(addr.key, "key");
    }

    #[test]
    fn test_validate_key() {
        let addr = Address::from_parts(None, None, "bad.key".to_string(), None);
        assert!(addr.validate_key(".").is_err());

        let addr = Address::from_parts(None, None, "good_key".to_string(), None);
        assert!(addr.validate_key(".").is_ok());
    }
}