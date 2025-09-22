use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use hub::data_ext::base64::{engine::general_purpose, Engine as _};
use hub::data_ext::serde_json::{self as serde_json};
use hub::error_ext::anyhow;
use hub::serde::{Deserialize, Serialize};
use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::OptionalExtension;
use rusqlite::{params_from_iter, Connection, Row, Transaction};

use crate::lib::core::crud::{
    CapabilityMap, CrudContext, CrudDomain, CrudError, CrudHooks, CrudMetadata, CrudObjectKind,
    CrudOutcome, CrudResource, CrudResult, CrudVerb, MetadataValue,
};

use super::utils::{SqliteConnectionConfig, SqlitePathResolver, SqliteRow, SqliteValue};

/// Adapter for SQLite table operations (schema + row group level).
pub struct SqliteTableAdapter<H: CrudHooks = ()> {
    config: SqliteConnectionConfig,
    hooks: H,
}

impl SqliteTableAdapter {
    pub fn new(config: SqliteConnectionConfig) -> Self {
        Self { config, hooks: () }
    }
}

impl<H: CrudHooks> SqliteTableAdapter<H> {
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

    fn connection(&self, ctx: &CrudContext, verb: CrudVerb) -> CrudResult<Connection> {
        let config = self.config_from_ctx(ctx);
        let flags = SqlitePathResolver::flags_for(&config);
        Connection::open_with_flags(config.database_path(), flags).map_err(|err| {
            CrudError::internal(
                self.domain(),
                self.object_kind(),
                verb,
                anyhow::Error::new(err),
            )
        })
    }

    fn table_name<'ctx>(&self, ctx: &'ctx CrudContext) -> Result<&'ctx str, CrudError> {
        ctx.identifier("table")
            .or_else(|| ctx.option("table"))
            .or_else(|| ctx.option("table_name"))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                CrudError::invalid_input(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    ctx.verb,
                    "missing table identifier (--table or --table-name)".to_string(),
                )
            })
    }

    fn schema_sql<'ctx>(&self, ctx: &'ctx CrudContext) -> Result<&'ctx str, CrudError> {
        ctx.option("schema_sql")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                CrudError::invalid_input(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    ctx.verb,
                    "missing schema definition (--schema-sql)".to_string(),
                )
            })
    }

    fn run_tx<F>(&self, conn: &mut Connection, verb: CrudVerb, f: F) -> CrudResult<CrudOutcome>
    where
        F: FnOnce(&Transaction<'_>) -> CrudResult<CrudOutcome>,
    {
        let tx = conn.transaction().map_err(|err| {
            CrudError::internal(
                self.domain(),
                self.object_kind(),
                verb,
                anyhow::Error::new(err),
            )
        })?;
        let outcome = f(&tx)?;
        tx.commit().map_err(|err| {
            CrudError::internal(
                self.domain(),
                self.object_kind(),
                verb,
                anyhow::Error::new(err),
            )
        })?;
        Ok(outcome)
    }

    fn pragma_table_info(
        &self,
        conn: &Connection,
        table: &str,
        verb: CrudVerb,
    ) -> CrudResult<Vec<BTreeMap<String, MetadataValue>>> {
        let pragma_sql = format!("PRAGMA table_info('{}')", table.replace("'", "''"));
        let mut stmt = conn.prepare(&pragma_sql).map_err(|err| {
            CrudError::internal(
                self.domain(),
                self.object_kind(),
                verb,
                anyhow::Error::new(err),
            )
        })?;
        let rows = stmt
            .query_map([], |row| Self::row_to_column_metadata(row))
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;
        let mut columns = Vec::new();
        for result in rows {
            columns.push(result.map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?);
        }
        Ok(columns)
    }

    fn row_to_column_metadata(row: &Row<'_>) -> rusqlite::Result<BTreeMap<String, MetadataValue>> {
        let cid: i64 = row.get("cid")?;
        let name: String = row.get("name")?;
        let data_type: String = row.get("type")?;
        let notnull: i64 = row.get("notnull")?;
        let dflt_value: Option<String> = row.get("dflt_value")?;
        let pk: i64 = row.get("pk")?;

        let mut column = BTreeMap::new();
        column.insert("cid".into(), MetadataValue::Integer(cid));
        column.insert("name".into(), MetadataValue::Text(name));
        column.insert("type".into(), MetadataValue::Text(data_type));
        column.insert("notnull".into(), MetadataValue::Boolean(notnull != 0));
        if let Some(default) = dflt_value {
            column.insert("default".into(), MetadataValue::Text(default));
        }
        column.insert("primary_key".into(), MetadataValue::Boolean(pk != 0));
        Ok(column)
    }

    fn ensure_table_exists(
        &self,
        tx: &Transaction<'_>,
        table: &str,
        verb: CrudVerb,
    ) -> CrudResult<()> {
        let mut stmt = tx
            .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1")
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;

        let exists = stmt
            .query_row([table], |row| row.get::<_, i64>(0))
            .optional()
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;

        if exists.is_some() {
            Ok(())
        } else {
            Err(CrudError::not_found(
                self.domain(),
                self.object_kind(),
                verb,
                format!("table not found: {}", table),
            ))
        }
    }

    fn table_schema_sql(
        &self,
        conn: &Connection,
        table: &str,
        verb: CrudVerb,
    ) -> CrudResult<String> {
        let mut stmt = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name=?1")
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;

        let schema = stmt
            .query_row([table], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;

        schema.ok_or_else(|| {
            CrudError::not_found(
                self.domain(),
                self.object_kind(),
                verb,
                format!("table not found: {}", table),
            )
        })
    }

    fn fetch_rows(
        &self,
        conn: &Connection,
        table: &str,
        verb: CrudVerb,
    ) -> CrudResult<Vec<SqliteRow>> {
        let mut stmt = conn
            .prepare(&format!("SELECT * FROM {}", Self::quote_identifier(table)))
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    verb,
                    anyhow::Error::new(err),
                )
            })?;

        let mut rows = stmt.query([]).map_err(|err| {
            CrudError::internal(
                self.domain(),
                self.object_kind(),
                verb,
                anyhow::Error::new(err),
            )
        })?;

        let mut entries = Vec::new();
        while let Some(row) = rows.next().map_err(|err| {
            CrudError::internal(
                self.domain(),
                self.object_kind(),
                verb,
                anyhow::Error::new(err),
            )
        })? {
            let mut map = BTreeMap::new();
            let column_names = row.as_ref().column_names().to_vec();
            for (index, name) in column_names.into_iter().enumerate() {
                let value_ref = row.get_ref(index).map_err(|err| {
                    CrudError::internal(
                        self.domain(),
                        self.object_kind(),
                        verb,
                        anyhow::Error::new(err),
                    )
                })?;

                let value = SqliteValue::from_value_ref(value_ref).map_err(|err| {
                    CrudError::internal(self.domain(), self.object_kind(), verb, err)
                })?;

                map.insert(name.to_string(), value);
            }
            entries.push(map);
        }

        Ok(entries)
    }

    fn resolve_path(ctx: &CrudContext, key: &str, verb: CrudVerb) -> CrudResult<PathBuf> {
        ctx.option(key)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                CrudError::invalid_input(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    verb,
                    format!("missing required option: --{}", key.replace('_', "-")),
                )
            })
    }

    fn quote_identifier(value: &str) -> String {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    }
}

impl<H: CrudHooks> CrudResource for SqliteTableAdapter<H> {
    type Hooks = H;

    fn domain(&self) -> CrudDomain {
        CrudDomain::Sqlite
    }

    fn object_kind(&self) -> CrudObjectKind {
        CrudObjectKind::Table
    }

    fn hooks(&self) -> &<Self as CrudResource>::Hooks {
        &self.hooks
    }

    fn capabilities(&self) -> CapabilityMap {
        let mut map = CapabilityMap::new();
        map.allow(CrudObjectKind::Table, CrudVerb::Create);
        map.allow(CrudObjectKind::Table, CrudVerb::Read);
        map.allow(CrudObjectKind::Table, CrudVerb::Update);
        map.allow(CrudObjectKind::Table, CrudVerb::Delete);
        map.allow(CrudObjectKind::Table, CrudVerb::List);
        map.allow(CrudObjectKind::Table, CrudVerb::Find);
        map.allow(CrudObjectKind::Table, CrudVerb::Backup);
        map.allow(CrudObjectKind::Table, CrudVerb::Restore);
        map
    }

    fn create(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let table = self.table_name(&ctx)?.to_string();
        let schema_sql = self.schema_sql(&ctx)?.to_string();
        let mut conn = self.connection(&ctx, CrudVerb::Create)?;

        self.run_tx(&mut conn, CrudVerb::Create, |tx| {
            tx.execute_batch(&schema_sql).map_err(|err| {
                CrudError::invalid_input(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Create,
                    format!("failed to execute schema: {}", err),
                )
            })?;

            let metadata = CrudMetadata::new().with_entry("table", table.clone());
            Ok(
                CrudOutcome::success(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::Create)
                    .with_metadata(metadata)
                    .with_payload(table),
            )
        })
    }

    fn read(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let table = self.table_name(&ctx)?.to_string();
        let conn = self.connection(&ctx, CrudVerb::Read)?;
        let columns = self.pragma_table_info(&conn, &table, CrudVerb::Read)?;
        if columns.is_empty() {
            return Err(CrudError::not_found(
                CrudDomain::Sqlite,
                CrudObjectKind::Table,
                CrudVerb::Read,
                format!("table not found: {}", table),
            ));
        }

        let mut metadata = CrudMetadata::new();
        metadata.insert("table", table);
        let column_descriptions: Vec<String> = columns
            .into_iter()
            .map(|col| {
                col.into_iter()
                    .map(|(key, value)| format!("{}={:?}", key, value))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect();
        metadata.insert("columns", MetadataValue::from(column_descriptions));

        Ok(
            CrudOutcome::success(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::Read)
                .with_metadata(metadata),
        )
    }

    fn update(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let table = self.table_name(&ctx)?.to_string();
        let update_sql = ctx
            .option("update_sql")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                CrudError::invalid_input(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Update,
                    "missing update statements (--update-sql)".to_string(),
                )
            })?;

        let mut conn = self.connection(&ctx, CrudVerb::Update)?;

        self.run_tx(&mut conn, CrudVerb::Update, |tx| {
            self.ensure_table_exists(tx, &table, CrudVerb::Update)?;

            tx.execute_batch(update_sql).map_err(|err| {
                CrudError::invalid_input(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Update,
                    format!("failed to execute update SQL: {}", err),
                )
            })?;

            let metadata = CrudMetadata::new().with_entry("table", table.clone());

            Ok(
                CrudOutcome::success(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::Update)
                    .with_metadata(metadata)
                    .with_payload(table),
            )
        })
    }

    fn delete(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let table = self.table_name(&ctx)?.to_string();
        let mut conn = self.connection(&ctx, CrudVerb::Delete)?;

        self.run_tx(&mut conn, CrudVerb::Delete, |tx| {
            let exists: Result<String, _> = tx.query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                [table.as_str()],
                |row| row.get(0),
            );

            if exists.is_err() {
                return Err(CrudError::not_found(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Delete,
                    format!("table not found: {}", table),
                ));
            }

            tx.execute(&format!("DROP TABLE \"{}\"", table), [])
                .map_err(|err| {
                    CrudError::internal(
                        CrudDomain::Sqlite,
                        CrudObjectKind::Table,
                        CrudVerb::Delete,
                        anyhow::Error::new(err),
                    )
                })?;

            let metadata = CrudMetadata::new().with_entry("table", table.clone());
            Ok(
                CrudOutcome::success(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::Delete)
                    .with_metadata(metadata)
                    .with_payload(table),
            )
        })
    }

    fn list(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let conn = self.connection(&ctx, CrudVerb::List)?;
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            )
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    CrudVerb::List,
                    anyhow::Error::new(err),
                )
            })?;
        let tables = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    CrudVerb::List,
                    anyhow::Error::new(err),
                )
            })?;
        let mut names = Vec::new();
        for result in tables {
            names.push(result.map_err(|err| {
                CrudError::internal(
                    self.domain(),
                    self.object_kind(),
                    CrudVerb::List,
                    anyhow::Error::new(err),
                )
            })?);
        }

        let mut metadata = CrudMetadata::new();
        metadata.insert("tables", MetadataValue::List(names.clone()));

        Ok(
            CrudOutcome::success(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::List)
                .with_metadata(metadata)
                .with_payload(names.join(",")),
        )
    }

    fn find(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let table = self.table_name(&ctx)?.to_string();
        let conn = self.connection(&ctx, CrudVerb::Find)?;
        let columns = self.pragma_table_info(&conn, &table, CrudVerb::Find)?;

        if columns.is_empty() {
            return Err(CrudError::not_found(
                CrudDomain::Sqlite,
                CrudObjectKind::Table,
                CrudVerb::Find,
                format!("table not found: {}", table),
            ));
        }

        let mut metadata = CrudMetadata::new();
        metadata.insert("table", table.clone());
        metadata.insert("column_count", MetadataValue::Integer(columns.len() as i64));

        Ok(
            CrudOutcome::success(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::Find)
                .with_metadata(metadata)
                .with_payload(table),
        )
    }

    fn backup(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let table = self.table_name(&ctx)?.to_string();
        let target_path = Self::resolve_path(&ctx, "target_path", CrudVerb::Backup)?;
        let conn = self.connection(&ctx, CrudVerb::Backup)?;

        let schema_sql = self.table_schema_sql(&conn, &table, CrudVerb::Backup)?;
        let rows = self.fetch_rows(&conn, &table, CrudVerb::Backup)?;

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                CrudError::internal(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Backup,
                    anyhow::Error::new(err),
                )
            })?;
        }

        let payload = TableBackupFile {
            table: table.clone(),
            schema_sql: schema_sql.clone(),
            rows: rows.clone(),
        };

        let serialized = serde_json::to_vec_pretty(&payload).map_err(|err| {
            CrudError::internal(
                CrudDomain::Sqlite,
                CrudObjectKind::Table,
                CrudVerb::Backup,
                anyhow::Error::new(err),
            )
        })?;

        fs::write(&target_path, serialized).map_err(|err| {
            CrudError::internal(
                CrudDomain::Sqlite,
                CrudObjectKind::Table,
                CrudVerb::Backup,
                anyhow::Error::new(err),
            )
        })?;

        let mut metadata = CrudMetadata::new();
        metadata.insert("table", table.clone());
        metadata.insert("row_count", MetadataValue::Integer(rows.len() as i64));
        metadata.insert(
            "backup_path",
            MetadataValue::Text(target_path.display().to_string()),
        );

        Ok(
            CrudOutcome::success(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::Backup)
                .with_metadata(metadata)
                .with_payload(target_path.display().to_string()),
        )
    }

    fn restore(&self, ctx: CrudContext) -> CrudResult<CrudOutcome> {
        let table = self.table_name(&ctx)?.to_string();
        let source_path = Self::resolve_path(&ctx, "source_path", CrudVerb::Restore)?;
        let bytes = fs::read(&source_path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                CrudError::not_found(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Restore,
                    format!("restore source not found: {}", source_path.display()),
                )
            } else {
                CrudError::internal(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Restore,
                    anyhow::Error::new(err),
                )
            }
        })?;

        let backup: TableBackupFile = serde_json::from_slice(&bytes).map_err(|err| {
            CrudError::invalid_input(
                CrudDomain::Sqlite,
                CrudObjectKind::Table,
                CrudVerb::Restore,
                format!("invalid table backup payload: {}", err),
            )
        })?;

        if backup.table != table {
            return Err(CrudError::invalid_input(
                CrudDomain::Sqlite,
                CrudObjectKind::Table,
                CrudVerb::Restore,
                format!(
                    "backup targeted table '{}' but context requested '{}'",
                    backup.table, table
                ),
            ));
        }

        let mut conn = self.connection(&ctx, CrudVerb::Restore)?;

        self.run_tx(&mut conn, CrudVerb::Restore, |tx| {
            tx.execute(
                &format!("DROP TABLE IF EXISTS {}", Self::quote_identifier(&table)),
                [],
            )
            .map_err(|err| {
                CrudError::internal(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Restore,
                    anyhow::Error::new(err),
                )
            })?;

            tx.execute_batch(&backup.schema_sql).map_err(|err| {
                CrudError::invalid_input(
                    CrudDomain::Sqlite,
                    CrudObjectKind::Table,
                    CrudVerb::Restore,
                    format!("failed to apply schema: {}", err),
                )
            })?;

            let column_order = backup
                .rows
                .first()
                .map(|row| row.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();

            if !column_order.is_empty() {
                let insert_sql = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    Self::quote_identifier(&table),
                    column_order
                        .iter()
                        .map(|col| Self::quote_identifier(col))
                        .collect::<Vec<_>>()
                        .join(","),
                    vec!["?"; column_order.len()].join(",")
                );

                let mut stmt = tx.prepare(&insert_sql).map_err(|err| {
                    CrudError::internal(
                        CrudDomain::Sqlite,
                        CrudObjectKind::Table,
                        CrudVerb::Restore,
                        anyhow::Error::new(err),
                    )
                })?;

                for row in &backup.rows {
                    let mut values = Vec::with_capacity(column_order.len());
                    for column in &column_order {
                        let value = row.get(column).ok_or_else(|| {
                            CrudError::invalid_input(
                                CrudDomain::Sqlite,
                                CrudObjectKind::Table,
                                CrudVerb::Restore,
                                format!("row missing column '{}' required by insert order", column),
                            )
                        })?;

                        values.push(value.to_sql_value().map_err(|err| {
                            CrudError::invalid_input(
                                CrudDomain::Sqlite,
                                CrudObjectKind::Table,
                                CrudVerb::Restore,
                                format!("failed to decode value for column '{}': {}", column, err),
                            )
                        })?);
                    }

                    stmt.execute(params_from_iter(values)).map_err(|err| {
                        CrudError::internal(
                            CrudDomain::Sqlite,
                            CrudObjectKind::Table,
                            CrudVerb::Restore,
                            anyhow::Error::new(err),
                        )
                    })?;
                }
            }

            let mut metadata = CrudMetadata::new();
            metadata.insert("table", table.clone());
            metadata.insert(
                "source_path",
                MetadataValue::Text(source_path.display().to_string()),
            );
            metadata.insert(
                "row_count",
                MetadataValue::Integer(backup.rows.len() as i64),
            );

            Ok(
                CrudOutcome::success(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::Restore)
                    .with_metadata(metadata)
                    .with_payload(table.clone()),
            )
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "hub::serde")]
struct TableBackupFile {
    table: String,
    schema_sql: String,
    rows: Vec<SqliteRow>,
}
