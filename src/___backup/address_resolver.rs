// Address resolution module - handles all addressing scenarios
// Determines whether to use 3-layer or 4-layer addressing based on input

use crate::addressing::Address;
use crate::addressing4::Address4;

#[derive(Debug, Clone)]
pub enum ResolvedAddress {
    ThreeLayer(Address),
    FourLayer(Address4),
}

impl ResolvedAddress {
    pub fn to_display_key(&self) -> String {
        match self {
            ResolvedAddress::ThreeLayer(addr) => {
                if let Some(ctx) = &addr.context {
                    format!("{}.{}.{}__{}", addr.project, addr.namespace, addr.key, ctx)
                } else {
                    format!("{}.{}.{}", addr.project, addr.namespace, addr.key)
                }
            }
            ResolvedAddress::FourLayer(addr) => addr.to_storage_key(),
        }
    }
}

pub struct AddressResolver {
    pub meta_context: Option<String>,  // --meta flag value
    pub project_context: Option<String>,  // -p flag value
    pub namespace_context: Option<String>,  // -n flag value
}

impl AddressResolver {
    pub fn new() -> Self {
        AddressResolver {
            meta_context: None,
            project_context: None,
            namespace_context: None,
        }
    }

    pub fn with_meta(mut self, meta: Option<String>) -> Self {
        self.meta_context = meta;
        self
    }

    pub fn with_project(mut self, project: Option<String>) -> Self {
        self.project_context = project;
        self
    }

    pub fn with_namespace(mut self, namespace: Option<String>) -> Self {
        self.namespace_context = namespace;
        self
    }

    /// Resolve an address string to either 3-layer or 4-layer addressing
    pub fn resolve_address(&self, address_str: &str) -> Result<ResolvedAddress, String> {
        let dot_count = address_str.matches('.').count();
        
        // Determine if we need 4-layer addressing
        let needs_meta = self.meta_context.is_some() || dot_count >= 3;
        
        if needs_meta {
            // Use 4-layer addressing
            let address = if dot_count >= 3 {
                // Full 4-dot address provided - parse as-is
                Address4::parse(address_str)?
            } else if let Some(meta) = &self.meta_context {
                // 3-dot or less with --meta flag
                self.build_4layer_with_meta(address_str, meta)?
            } else {
                // Edge case: 3+ dots but no meta context - use default meta
                match Address4::parse(address_str) {
                    Ok(addr) => addr,
                    Err(_) => {
                        // If 4-dot parsing fails, try building with default meta
                        self.build_4layer_with_meta(address_str, "default")?
                    }
                }
            };
            
            Ok(ResolvedAddress::FourLayer(address))
        } else {
            // Use traditional 3-layer addressing
            let address = self.build_3layer(address_str)?;
            Ok(ResolvedAddress::ThreeLayer(address))
        }
    }

    fn build_4layer_with_meta(&self, address_str: &str, meta: &str) -> Result<Address4, String> {
        let parts: Vec<&str> = address_str.split('.').collect();
        
        // Handle context suffix (key__context)
        let (key_part, context) = if let Some(last) = parts.last() {
            if let Some(idx) = last.rfind("__") {
                (&last[..idx], Some(last[idx+2..].to_string()))
            } else {
                (*last, None)
            }
        } else {
            return Err("Empty address".to_string());
        };

        let mut address = match parts.len() {
            1 => {
                // Single part: use contexts or defaults
                let project = self.project_context.as_deref().unwrap_or("default");
                let namespace = self.namespace_context.as_deref().unwrap_or("default");
                Address4::new(meta, project, namespace, key_part)
            }
            2 => {
                // Two parts: namespace.key
                let project = self.project_context.as_deref().unwrap_or("default");
                Address4::new(meta, project, parts[0], key_part)
            }
            3 => {
                // Three parts: project.namespace.key
                Address4::new(meta, parts[0], parts[1], key_part)
            }
            _ => {
                return Err(format!("Invalid address format: {}", address_str));
            }
        };

        if let Some(ctx) = context {
            address.context = Some(ctx);
        }

        Ok(address)
    }

    fn build_3layer(&self, address_str: &str) -> Result<Address, String> {
        let parts: Vec<&str> = address_str.split('.').collect();
        
        // Handle context suffix (key__context)
        let (key_part, context) = if let Some(last) = parts.last() {
            if let Some(idx) = last.rfind("__") {
                (&last[..idx], Some(last[idx+2..].to_string()))
            } else {
                (*last, None)
            }
        } else {
            return Err("Empty address".to_string());
        };

        let mut address = match parts.len() {
            1 => {
                // Single part: use contexts or defaults
                let project = self.project_context.as_deref().unwrap_or("default");
                let namespace = self.namespace_context.as_deref().unwrap_or("default");
                Address {
                    project: project.to_string(),
                    namespace: namespace.to_string(),
                    key: key_part.to_string(),
                    context: None,
                }
            }
            2 => {
                // Two parts: could be namespace.key or project.namespace based on context
                if self.project_context.is_some() {
                    // Have project context, treat as namespace.key
                    Address {
                        project: self.project_context.as_ref().unwrap().clone(),
                        namespace: parts[0].to_string(),
                        key: key_part.to_string(),
                        context: None,
                    }
                } else {
                    // No project context, treat as namespace.key with default project
                    Address {
                        project: "default".to_string(),
                        namespace: parts[0].to_string(),
                        key: key_part.to_string(),
                        context: None,
                    }
                }
            }
            3 => {
                // Three parts: project.namespace.key
                Address {
                    project: parts[0].to_string(),
                    namespace: parts[1].to_string(),
                    key: key_part.to_string(),
                    context: None,
                }
            }
            _ => {
                return Err(format!("Invalid address format: {}", address_str));
            }
        };

        if let Some(ctx) = context {
            address.context = Some(ctx);
        }

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_4layer_full_address() {
        let resolver = AddressResolver::new();
        let result = resolver.resolve_address("keeper.app.config.key").unwrap();
        
        if let ResolvedAddress::FourLayer(addr) = result {
            assert_eq!(addr.meta, "keeper");
            assert_eq!(addr.project, "app");
            assert_eq!(addr.namespace, "config");
            assert_eq!(addr.key, "key");
        } else {
            panic!("Expected 4-layer address");
        }
    }

    #[test]
    fn test_3layer_with_meta_flag() {
        let resolver = AddressResolver::new().with_meta(Some("keeper".to_string()));
        let result = resolver.resolve_address("app.config.key").unwrap();
        
        if let ResolvedAddress::FourLayer(addr) = result {
            assert_eq!(addr.meta, "keeper");
            assert_eq!(addr.project, "app");
            assert_eq!(addr.namespace, "config");
            assert_eq!(addr.key, "key");
        } else {
            panic!("Expected 4-layer address with meta");
        }
    }

    #[test]
    fn test_traditional_3layer() {
        let resolver = AddressResolver::new();
        let result = resolver.resolve_address("app.config.key").unwrap();
        
        if let ResolvedAddress::ThreeLayer(addr) = result {
            assert_eq!(addr.project, "app");
            assert_eq!(addr.namespace, "config");
            assert_eq!(addr.key, "key");
        } else {
            panic!("Expected 3-layer address");
        }
    }

    #[test]
    fn test_context_suffix() {
        let resolver = AddressResolver::new().with_meta(Some("keeper".to_string()));
        let result = resolver.resolve_address("app.config.key__prod").unwrap();
        
        if let ResolvedAddress::FourLayer(addr) = result {
            assert_eq!(addr.key, "key");
            assert_eq!(addr.context, Some("prod".to_string()));
        } else {
            panic!("Expected 4-layer address with context");
        }
    }
}