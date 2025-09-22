//! Forge-inspired CRUD+ primitives for ProntoDB.
//! MODULE_SPEC: this orchestrator wires the trait, types, errors, and helpers.

mod capability;
mod context;
mod error;
mod metadata;
mod outcome;
mod traits;
mod types;

pub use capability::{CapabilityEntry, CapabilityMap};
pub use context::CrudContext;
pub use error::{CrudError, CrudErrorKind, CrudResult};
pub use metadata::{CrudMetadata, MetadataValue};
pub use outcome::{CrudOutcome, CrudStatus};
pub use traits::{CrudHooks, CrudResource};
pub use types::{CrudDomain, CrudObjectKind, CrudVerb};
