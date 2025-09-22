use std::fs;
use std::path::PathBuf;

use hub::error_ext::anyhow;
use rusqlite::Connection;

use crate::lib::core::crud::{
    CapabilityMap, CrudContext, CrudDomain, CrudError, CrudHooks, CrudMetadata, CrudObjectKind,
    CrudOutcome, CrudResource, CrudResult, CrudVerb,
};

use super::utils::{SqliteConnectionConfig, SqlitePathResolver};

/// Adapter responsible for SQLite database-level operations (open, backup, restore).
pub struct SqliteBaseAdapter<H: CrudHooks = ()> {
    config: SqliteConnectionConfig,
    hooks: H,
}

impl SqliteBaseAdapter {
    pub fn new(config: SqliteConnectionConfig) -> Self {
        Self { config, hooks: () }
    }
}

impl<H: CrudHooks> SqliteBaseAdapter<H> {
    pub fn with_hooks(config: SqliteConnectionConfig, hooks: H) -> Self {
        Self { config, hooks }
    }

    pub fn config(&self) -> &SqliteConnectionConfig {
        &self.config
    }

    fn config_from_ctx(&self, ctx: &CrudContext) -> SqliteConnectionConfig {
        if let Some(path) = ctx.option("database_path") {
            self.config.clone().with_database_path(path)
        } else {
            self.config.clone()
        }
    }

    fn ensure_connection(&self, config: &SqliteConnectionConfig, verb: CrudVerb) -> CrudResult<()> {
        let path = config.database_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;
        }

        let conn = Connection::open_with_flags(path, SqlitePathResolver::flags_for(config))
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;

        if !config.read_only && config.journal_wal {
            conn.pragma_update(None, "journal_mode", &"WAL")
                .map_err(|err| {
                    CrudError::internal(
                        self.domain(),
                        self.object_kind(),
                        verb,
                        anyhow::Error::new(err),
                    )
                })?;
        }

        Ok(())
    }

    fn file_metadata(&self, path: &PathBuf, verb: CrudVerb) -> CrudResult<CrudMetadata> {
        let meta = fs::metadata(path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                CrudError::not_found(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    format!("database file not found: {}", path.display()),
                )
            } else {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            }
        })?;

        let mut metadata = CrudMetadata::new();
        metadata.insert("path", path.display().to_string());
        metadata.insert("size_bytes", meta.len() as i64);
        if let Ok(modified) = meta.modified() {
            metadata.insert("last_modified", format!("{:?}", modified));
        }
        Ok(metadata)
    }

    fn resolve_target(ctx: &CrudContext, key: &str, verb: CrudVerb) -> Result<PathBuf, CrudError> {
        ctx.option(key)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                CrudError::invalid_input(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Base,
                    verb,
                    format!("missing required option: --{}", key.replace('_', "-")),
                )
            })
    }
}

impl<H: CrudHooks> CrudResource for SqliteBaseAdapter<H> {
    type Hooks = H;

    fn domain(&self) -> CrudDomain {
        CrudDomain::Sqlite
    }

    fn object_kind(&self) -> CrudObjectKind {
        CrudObjectKind::Base
    }

    fn hooks(&self) -> &<Self as CrudResource>::Hooks {
        &self.hooks
    }

    fn capabilities(&self) -> CapabilityMap {
        let mut map = CapabilityMap::new();
        map.allow(CrudObjectKind::Base, CrudVerb::Create);
        map.allow(CrudObjectKind::Base, CrudVerb::Read);
        map.allow(CrudObjectKind::Base, CrudVerb::Backup);
        map.allow(CrudObjectKind::Base, CrudVerb::Restore);
        map
    }

    fn create(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let verb = CrudVerb::Create;
        let config = self.config_from_ctx(&ctx);
        self.ensure_connection(&config, verb)?;

        let metadata = self.file_metadata(&config.database_path().to_path_buf(), verb)?;
        Ok(CrudOutcome::success(self.domain(), self.object_kind(), verb).with_metadata(metadata))
    }

    fn read(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let verb = CrudVerb::Read;
        let config = self.config_from_ctx(&ctx);
        let metadata = self.file_metadata(&config.database_path().to_path_buf(), verb)?;
        Ok(CrudOutcome::success(self.domain(), self.object_kind(), verb).with_metadata(metadata))
    }

    fn backup(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let verb = CrudVerb::Backup;
        let config = self.config_from_ctx(&ctx);
        let source_path = config.database_path().to_path_buf();
        let target_path = Self::resolve_target(&ctx, "target_path", verb)?;

        self.file_metadata(&source_path, verb)?;

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;
        }

        fs::copy(&source_path, &target_path).map_err(|err| {
            CrudError::internal(
                self.domain(),
                self.object_kind(),
                verb,
                anyhow::Error::new(err),
            )
        })?;

        let mut metadata = self.file_metadata(&source_path, verb)?;
        metadata.insert("backup_path", target_path.display().to_string());

        Ok(
            CrudOutcome::success(self.domain(), self.object_kind(), verb)
                .with_metadata(metadata)
                .with_payload(target_path.display().to_string()),
        )
    }

    fn restore(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let verb = CrudVerb::Restore;
        let config = self.config_from_ctx(&ctx);
        let dest_path = config.database_path().to_path_buf();
        let source_path = Self::resolve_target(&ctx, "source_path", verb)?;

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;
        }

        fs::metadata(&source_path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                CrudError::not_found(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    format!("restore source not found: {}", source_path.display()),
                )
            } else {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            }
        })?;

        fs::copy(&source_path, &dest_path).map_err(|err| {
            CrudError::internal(
                self.domain(),
                self.object_kind(),
                verb,
                anyhow::Error::new(err),
            )
        })?;

        let mut metadata = CrudMetadata::new();
        metadata.insert("restored_from", source_path.display().to_string());
        metadata.insert("path", dest_path.display().to_string());

        Ok(
            CrudOutcome::success(self.domain(), self.object_kind(), verb)
                .with_metadata(metadata)
                .with_payload(dest_path.display().to_string()),
        )
    }
}
