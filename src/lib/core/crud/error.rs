use std::error::Error as StdError;
use std::fmt;

use hub::error_ext::anyhow::{self, Error};

use super::{CrudDomain, CrudObjectKind, CrudVerb};

/// Classified error kinds for CRUD operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CrudErrorKind {
    Unsupported,
    InvalidInput,
    Conflict,
    NotFound,
    Internal,
}

/// Error wrapper carrying domain/object/verb context.
#[derive(Debug)]
pub struct CrudError {
    pub kind: CrudErrorKind,
    pub domain: CrudDomain,
    pub object: CrudObjectKind,
    pub verb: CrudVerb,
    source: Error,
}

pub type CrudResult<T> = Result<T, CrudError>;

impl CrudError {
    pub fn new(
        kind: CrudErrorKind,
        domain: CrudDomain,
        object: CrudObjectKind,
        verb: CrudVerb,
        source: Error,
    ) -> Self {
        Self {
            kind,
            domain,
            object,
            verb,
            source,
        }
    }

    pub fn unsupported(domain: CrudDomain, object: CrudObjectKind, verb: CrudVerb) -> Self {
        let message = format!("{} {} does not support verb {}", domain, object, verb);
        Self::new(
            CrudErrorKind::Unsupported,
            domain,
            object,
            verb,
            anyhow::anyhow!(message),
        )
    }

    pub fn invalid_input<S: Into<String>>(
        domain: CrudDomain,
        object: CrudObjectKind,
        verb: CrudVerb,
        message: S,
    ) -> Self {
        Self::new(
            CrudErrorKind::InvalidInput,
            domain,
            object,
            verb,
            anyhow::anyhow!(message.into()),
        )
    }

    pub fn conflict<S: Into<String>>(
        domain: CrudDomain,
        object: CrudObjectKind,
        verb: CrudVerb,
        message: S,
    ) -> Self {
        Self::new(
            CrudErrorKind::Conflict,
            domain,
            object,
            verb,
            anyhow::anyhow!(message.into()),
        )
    }

    pub fn not_found<S: Into<String>>(
        domain: CrudDomain,
        object: CrudObjectKind,
        verb: CrudVerb,
        message: S,
    ) -> Self {
        Self::new(
            CrudErrorKind::NotFound,
            domain,
            object,
            verb,
            anyhow::anyhow!(message.into()),
        )
    }

    pub fn internal(
        domain: CrudDomain,
        object: CrudObjectKind,
        verb: CrudVerb,
        source: Error,
    ) -> Self {
        Self::new(CrudErrorKind::Internal, domain, object, verb, source)
    }

    pub fn source(&self) -> &Error {
        &self.source
    }
}

impl fmt::Display for CrudError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} error during {} {}: {}",
            match self.kind {
                CrudErrorKind::Unsupported => "Unsupported",
                CrudErrorKind::InvalidInput => "Invalid input",
                CrudErrorKind::Conflict => "Conflict",
                CrudErrorKind::NotFound => "Not found",
                CrudErrorKind::Internal => "Internal",
            },
            self.domain,
            self.object,
            self.source
        )
    }
}

impl StdError for CrudError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.source.as_ref())
    }
}

impl From<Error> for CrudError {
    fn from(source: Error) -> Self {
        // Fallback when context is missing; mark as internal/unknown meta.
        CrudError {
            kind: CrudErrorKind::Internal,
            domain: CrudDomain::Sqlite,
            object: CrudObjectKind::Container,
            verb: CrudVerb::Invalid,
            source,
        }
    }
}
