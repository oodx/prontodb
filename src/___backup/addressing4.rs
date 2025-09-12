// 4-Layer Addressing module for meta.project.namespace.key format
// Supports transparent meta namespace isolation for multi-tenant systems

#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AddressContext {
    KeyAccess,   // get/set/del operations
    Discovery,   // keys/scan operations  
    Auto,        // Attempt intelligent detection
}

#[derive(Debug, Clone, PartialEq)]
pub struct Address4 {
    pub meta: String,        // Meta namespace (user/tenant/context)
    pub project: String,     // Project namespace
    pub namespace: String,   // Namespace within project
    pub key: String,         // The actual key
    pub context: Option<String>,  // Optional context suffix
}

impl Address4 {
    /// Parse a 4-dot address: meta.project.namespace.key
    pub fn parse(path: &str) -> Result<Self, String> {
        Self::parse_with_delimiter(path, ".")
    }

    /// Parse with custom delimiter
    pub fn parse_with_delimiter(path: &str, delimiter: &str) -> Result<Self, String> {
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
                // Single part: use defaults for all namespaces
                Ok(Address4 {
                    meta: "default".to_string(),
                    project: "default".to_string(),
                    namespace: "default".to_string(),
                    key: parts[0].to_string(),
                    context: ctx_suffix,
                })
            }
            2 => {
                // Two parts: namespace.key with default meta/project
                Ok(Address4 {
                    meta: "default".to_string(),
                    project: "default".to_string(),
                    namespace: parts[0].to_string(),
                    key: parts[1].to_string(),
                    context: ctx_suffix,
                })
            }
            3 => {
                // Three parts: project.namespace.key with default meta
                Ok(Address4 {
                    meta: "default".to_string(),
                    project: parts[0].to_string(),
                    namespace: parts[1].to_string(),
                    key: parts[2].to_string(),
                    context: ctx_suffix,
                })
            }
            4 => {
                // Four parts: meta.project.namespace.key (full address)
                Ok(Address4 {
                    meta: parts[0].to_string(),
                    project: parts[1].to_string(),
                    namespace: parts[2].to_string(),
                    key: parts[3].to_string(),
                    context: ctx_suffix,
                })
            }
            _ => {
                // More than 4 parts: join extras into key
                Ok(Address4 {
                    meta: parts[0].to_string(),
                    project: parts[1].to_string(),
                    namespace: parts[2].to_string(),
                    key: parts[3..].join(delimiter),
                    context: ctx_suffix,
                })
            }
        }
    }

    /// Convert to storage key format (flattened for database)
    pub fn to_storage_key(&self) -> String {
        if let Some(ctx) = &self.context {
            format!("{}.{}.{}.{}__{}",
                self.meta, self.project, self.namespace, self.key, ctx)
        } else {
            format!("{}.{}.{}.{}",
                self.meta, self.project, self.namespace, self.key)
        }
    }

    /// Get the canonical path without meta (for backward compatibility)
    pub fn to_canonical_path(&self) -> String {
        if let Some(ctx) = &self.context {
            format!("{}.{}.{}__{}",
                self.project, self.namespace, self.key, ctx)
        } else {
            format!("{}.{}.{}",
                self.project, self.namespace, self.key)
        }
    }

    /// Create from components
    pub fn new(meta: &str, project: &str, namespace: &str, key: &str) -> Self {
        Address4 {
            meta: meta.to_string(),
            project: project.to_string(),
            namespace: namespace.to_string(),
            key: key.to_string(),
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(meta: &str, project: &str, namespace: &str, key: &str, context: &str) -> Self {
        Address4 {
            meta: meta.to_string(),
            project: project.to_string(),
            namespace: namespace.to_string(),
            key: key.to_string(),
            context: Some(context.to_string()),
        }
    }
}

impl fmt::Display for Address4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_storage_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_4dot_address() {
        let addr = Address4::parse("keeper.pantheon.config.setting").unwrap();
        assert_eq!(addr.meta, "keeper");
        assert_eq!(addr.project, "pantheon");
        assert_eq!(addr.namespace, "config");
        assert_eq!(addr.key, "setting");
    }

    #[test]
    fn test_parse_3dot_defaults_meta() {
        let addr = Address4::parse("pantheon.config.setting").unwrap();
        assert_eq!(addr.meta, "default");
        assert_eq!(addr.project, "pantheon");
        assert_eq!(addr.namespace, "config");
        assert_eq!(addr.key, "setting");
    }

    #[test]
    fn test_parse_2dot_defaults() {
        let addr = Address4::parse("config.setting").unwrap();
        assert_eq!(addr.meta, "default");
        assert_eq!(addr.project, "default");
        assert_eq!(addr.namespace, "config");
        assert_eq!(addr.key, "setting");
    }

    #[test]
    fn test_parse_with_context() {
        let addr = Address4::parse("keeper.pantheon.config.api_key__prod").unwrap();
        assert_eq!(addr.meta, "keeper");
        assert_eq!(addr.project, "pantheon");
        assert_eq!(addr.namespace, "config");
        assert_eq!(addr.key, "api_key");
        assert_eq!(addr.context, Some("prod".to_string()));
    }

    #[test]
    fn test_storage_key_format() {
        let addr = Address4::new("lucas", "fx", "tools", "hammer");
        assert_eq!(addr.to_storage_key(), "lucas.fx.tools.hammer");
    }

    #[test]
    fn test_user_isolation() {
        let keeper_addr = Address4::parse("keeper.pantheon.data.secret").unwrap();
        let lucas_addr = Address4::parse("lucas.pantheon.data.secret").unwrap();
        
        // Same canonical path but different storage keys
        assert_ne!(keeper_addr.to_storage_key(), lucas_addr.to_storage_key());
        assert_eq!(keeper_addr.to_canonical_path(), lucas_addr.to_canonical_path());
    }
}