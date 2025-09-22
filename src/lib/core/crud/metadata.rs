use std::collections::BTreeMap;

/// Simple value bag for metadata returned by CRUD operations.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum MetadataValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<String>),
}

impl From<&str> for MetadataValue {
    fn from(value: &str) -> Self {
        MetadataValue::Text(value.to_string())
    }
}

impl From<String> for MetadataValue {
    fn from(value: String) -> Self {
        MetadataValue::Text(value)
    }
}

impl From<i64> for MetadataValue {
    fn from(value: i64) -> Self {
        MetadataValue::Integer(value)
    }
}

impl From<f64> for MetadataValue {
    fn from(value: f64) -> Self {
        MetadataValue::Float(value)
    }
}

impl From<bool> for MetadataValue {
    fn from(value: bool) -> Self {
        MetadataValue::Boolean(value)
    }
}

impl From<Vec<String>> for MetadataValue {
    fn from(value: Vec<String>) -> Self {
        MetadataValue::List(value)
    }
}

/// Structured metadata returned by adapters.
#[derive(Clone, Debug, Default)]
pub struct CrudMetadata {
    pub entries: BTreeMap<String, MetadataValue>,
}

impl CrudMetadata {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn with_entry<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<MetadataValue>,
    {
        self.entries.insert(key.into(), value.into());
        self
    }

    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<MetadataValue>,
    {
        self.entries.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.entries.get(key)
    }
}
