use super::capability::CapabilityMap;
use super::context::CrudContext;
use super::error::{CrudError, CrudResult};
use super::outcome::CrudOutcome;
use super::types::{CrudDomain, CrudObjectKind, CrudVerb};

/// No-op hook implementation; adapters can implement their own to add behaviour.
pub trait CrudHooks {
    fn before(&self, _verb: CrudVerb, _ctx: &CrudContext) -> CrudResult<()> {
        Ok(())
    }

    fn after(&self, _verb: CrudVerb, _ctx: &CrudContext, _outcome: &CrudOutcome) -> CrudResult<()> {
        Ok(())
    }

    fn on_error(&self, _verb: CrudVerb, _ctx: &CrudContext, _error: &CrudError) {}
}

impl CrudHooks for () {}

/// Primary trait every CRUD adapter must implement.
pub trait CrudResource {
    type Hooks: CrudHooks;

    fn domain(&self) -> CrudDomain;
    fn object_kind(&self) -> CrudObjectKind;
    fn hooks(&self) -> &Self::Hooks;

    fn capabilities(&self) -> CapabilityMap;

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

    fn list(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::List,
        ))
    }

    fn find(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Find,
        ))
    }

    fn backup(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Backup,
        ))
    }

    fn restore(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Restore,
        ))
    }

    fn alias(&self, _ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::unsupported(
            self.domain(),
            self.object_kind(),
            CrudVerb::Alias,
        ))
    }

    fn invalid(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        Err(CrudError::invalid_input(
            self.domain(),
            self.object_kind(),
            CrudVerb::Invalid,
            format!(
                "invalid verb requested for context: domain={} object={} identifiers={:?}",
                ctx.domain, ctx.object, ctx.identifiers
            ),
        ))
    }

    fn dispatch(&self, verb: CrudVerb, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        self.hooks().before(verb, &ctx)?;
        let result = match verb {
            CrudVerb::Create => self.create(ctx.clone()),
            CrudVerb::Read => self.read(ctx.clone()),
            CrudVerb::Update => self.update(ctx.clone()),
            CrudVerb::Delete => self.delete(ctx.clone()),
            CrudVerb::List => self.list(ctx.clone()),
            CrudVerb::Find => self.find(ctx.clone()),
            CrudVerb::Backup => self.backup(ctx.clone()),
            CrudVerb::Restore => self.restore(ctx.clone()),
            CrudVerb::Alias => self.alias(ctx.clone()),
            CrudVerb::Invalid => self.invalid(ctx.clone()),
        };

        match result {
            Ok(outcome) => {
                self.hooks().after(verb, &ctx, &outcome)?;
                Ok(outcome)
            }
            Err(error) => {
                self.hooks().on_error(verb, &ctx, &error);
                Err(error)
            }
        }
    }
}
