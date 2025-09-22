use crate::lib::core::crud::{
    CapabilityMap, CrudContext, CrudDomain, CrudError, CrudHooks, CrudObjectKind, CrudOutcome,
    CrudResource, CrudResult, CrudVerb,
};

use super::utils::SqliteConnectionConfig;

/// Adapter for row-level operations within a SQLite table.
pub struct SqliteRecordAdapter<H: CrudHooks = ()> {
    config: SqliteConnectionConfig,
    hooks: H,
}

impl SqliteRecordAdapter {
    pub fn new(config: SqliteConnectionConfig) -> Self {
        Self { config, hooks: () }
    }
}

impl<H: CrudHooks> SqliteRecordAdapter<H> {
    pub fn with_hooks(config: SqliteConnectionConfig, hooks: H) -> Self {
        Self { config, hooks }
    }

    pub fn config(&self) -> &SqliteConnectionConfig {
        &self.config
    }
}

impl<H: CrudHooks> CrudResource for SqliteRecordAdapter<H> {
    type Hooks = H;

    fn domain(&self) -> CrudDomain {
        CrudDomain::Sqlite
    }

    fn object_kind(&self) -> CrudObjectKind {
        CrudObjectKind::Record
    }

    fn hooks(&self) -> &<Self as CrudResource>::Hooks {
        &self.hooks
    }

    fn capabilities(&self) -> CapabilityMap {
        CapabilityMap::new()
    }

    fn create(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Create,
        ))
    }

    fn read(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Read,
        ))
    }

    fn update(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Update,
        ))
    }

    fn delete(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Delete,
        ))
    }

    fn find(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Find,
        ))
    }
}
