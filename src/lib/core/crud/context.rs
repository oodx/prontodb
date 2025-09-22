use std::collections::BTreeMap;

use super::{CrudDomain, CrudObjectKind, CrudVerb};

/// Normalised data passed into each CRUD+ operation.
#[derive(Clone, Debug)]
pub struct CrudContext {
    /// Domain in which the adapter operates (sqlite, filesystem, ...).
    pub domain: CrudDomain,
    /// Object kind targeted by the operation (table, record, ...).
    pub object: CrudObjectKind,
    /// Verb being executed.
    pub verb: CrudVerb,
    /// Identifiers supplied by callers (table name, key, etc.).
    pub identifiers: BTreeMap<String, String>,
    /// Arbitrary options (CLI flags, environment derived configuration).
    pub options: BTreeMap<String, String>,
}

impl CrudContext {
    pub fn new(domain: CrudDomain, object: CrudObjectKind, verb: CrudVerb) -> Self {
        Self {
            domain,
            object,
            verb,
            identifiers: BTreeMap::default(),
            options: BTreeMap::default(),
        }
    }

    pub fn with_identifier<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.identifiers.insert(key.into(), value.into());
        self
    }

    pub fn with_option<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.options.insert(key.into(), value.into());
        self
    }

    pub fn identifier(&self, key: &str) -> Option<&str> {
        self.identifiers.get(key).map(|s| s.as_str())
    }

    pub fn option(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(|s| s.as_str())
    }
}
