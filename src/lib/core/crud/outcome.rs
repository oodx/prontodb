use super::metadata::{CrudMetadata, MetadataValue};
use super::{CrudDomain, CrudObjectKind, CrudVerb};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CrudStatus {
    Success,
    NoOp,
    Skipped,
}

/// Result envelope returned by CRUD adapters.
#[derive(Clone, Debug)]
pub struct CrudOutcome {
    pub domain: CrudDomain,
    pub object: CrudObjectKind,
    pub verb: CrudVerb,
    pub status: CrudStatus,
    pub metadata: CrudMetadata,
    pub payload: Option<MetadataValue>,
}

impl CrudOutcome {
    pub fn success(domain: CrudDomain, object: CrudObjectKind, verb: CrudVerb) -> Self {
        Self {
            domain,
            object,
            verb,
            status: CrudStatus::Success,
            metadata: CrudMetadata::new(),
            payload: None,
        }
    }

    pub fn with_metadata(mut self, metadata: CrudMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_payload<V>(mut self, payload: V) -> Self
    where
        V: Into<MetadataValue>,
    {
        self.payload = Some(payload.into());
        self
    }
}
