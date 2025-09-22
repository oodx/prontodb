use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use hub::data_ext::base64::{engine::general_purpose, Engine as _};
use hub::error_ext::anyhow;
use hub::serde::{Deserialize, Serialize};
use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::OpenFlags;

use crate::lib::core::crud::CrudDomain;

/// Configuration for establishing SQLite connections for adapters.
#[derive(Clone, Debug)]
pub struct SqliteConnectionConfig {
    pub database_path: PathBuf,
    pub read_only: bool,
    pub journal_wal: bool,
}

impl SqliteConnectionConfig {
    pub fn new<P: AsRef<Path>>(database_path: P) -> Self {
        Self {
            database_path: database_path.as_ref().to_path_buf(),
            read_only: false,
            journal_wal: true,
        }
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_wal(mut self, wal: bool) -> Self {
        self.journal_wal = wal;
        self
    }

    pub fn with_database_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.database_path = path.as_ref().to_path_buf();
        self
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }
}

impl Default for SqliteConnectionConfig {
    fn default() -> Self {
        Self::new("prontodb.sqlite3")
    }
}

/// Resolves on-disk locations for SQLite databases leveraging RSB globals (future work).
#[derive(Default)]
pub struct SqlitePathResolver;

impl SqlitePathResolver {
    pub fn database_path_from_env(_domain: CrudDomain) -> PathBuf {
        // TODO: consume RSB host/global context to honour XDG + per-address layouts.
        PathBuf::from("prontodb.sqlite3")
    }

    pub fn flags_for(config: &SqliteConnectionConfig) -> OpenFlags {
        let mut flags = OpenFlags::SQLITE_OPEN_NO_MUTEX;
        if config.read_only {
            flags.insert(OpenFlags::SQLITE_OPEN_READ_ONLY);
        } else {
            flags.insert(OpenFlags::SQLITE_OPEN_READ_WRITE);
            flags.insert(OpenFlags::SQLITE_OPEN_CREATE);
        }
        flags
    }
}

/// JSON-serialisable representation of SQLite values used by table backups and record CRUD APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", crate = "hub::serde")]
pub enum SqliteValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(String),
}

impl SqliteValue {
    pub fn from_value_ref(value: ValueRef<'_>) -> Result<Self, anyhow::Error> {
        let encoded = match value {
            ValueRef::Null => SqliteValue::Null,
            ValueRef::Integer(v) => SqliteValue::Integer(v),
            ValueRef::Real(v) => SqliteValue::Real(v),
            ValueRef::Text(bytes) => SqliteValue::Text(String::from_utf8(bytes.to_vec())?),
            ValueRef::Blob(bytes) => SqliteValue::Blob(general_purpose::STANDARD.encode(bytes)),
        };
        Ok(encoded)
    }

    pub fn to_sql_value(&self) -> Result<SqlValue, anyhow::Error> {
        match self {
            SqliteValue::Null => Ok(SqlValue::Null),
            SqliteValue::Integer(v) => Ok(SqlValue::Integer(*v)),
            SqliteValue::Real(v) => Ok(SqlValue::Real(*v)),
            SqliteValue::Text(text) => Ok(SqlValue::Text(text.clone())),
            SqliteValue::Blob(encoded) => {
                let decoded = general_purpose::STANDARD.decode(encoded.as_bytes())?;
                Ok(SqlValue::Blob(decoded))
            }
        }
    }
}

pub type SqliteRow = BTreeMap<String, SqliteValue>;
