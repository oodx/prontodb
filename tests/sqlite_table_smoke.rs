use std::fs;

use hub::data_ext::serde_json::{self as serde_json, Value as JsonValue};
use prontodb::lib::adpt::sqlite::{SqliteBaseAdapter, SqliteConnectionConfig, SqliteTableAdapter};
use prontodb::lib::core::crud::{
    CrudContext, CrudDomain, CrudErrorKind, CrudObjectKind, CrudResource, CrudStatus, CrudVerb,
};
use rusqlite::{Connection, OpenFlags};
use tempfile::tempdir;

fn ctx_with_table(path: &str, table: &str, verb: CrudVerb) -> CrudContext {
    let mut ctx = CrudContext::new(CrudDomain::Sqlite, CrudObjectKind::Table, verb);
    ctx.options.insert("database_path".into(), path.to_string());
    ctx.options.insert("table".into(), table.to_string());
    ctx
}

fn open_connection(path: &str) -> Connection {
    let mut flags = OpenFlags::SQLITE_OPEN_READ_WRITE;
    flags.insert(OpenFlags::SQLITE_OPEN_NO_MUTEX);
    Connection::open_with_flags(path, flags).expect("open sqlite connection")
}

fn seed_rows(path: &str) {
    let conn = open_connection(path);
    conn.execute(
        "INSERT INTO specimen(id, value, blob) VALUES(1, 'alpha', x'414243')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO specimen(id, value, blob) VALUES(2, 'beta', x'444546')",
        [],
    )
    .unwrap();
}

#[test]
fn create_read_delete_table_roundtrip() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("table_roundtrip.sqlite");
    let adapter = SqliteTableAdapter::new(SqliteConnectionConfig::default());
    let base = SqliteBaseAdapter::new(SqliteConnectionConfig::default());

    // ensure database exists
    let base_ctx = CrudContext::new(CrudDomain::Sqlite, CrudObjectKind::Base, CrudVerb::Create)
        .with_option("database_path", db_path.to_str().unwrap());
    base.dispatch(CrudVerb::Create, base_ctx).unwrap();

    // create table
    let mut create_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Create);
    create_ctx.options.insert(
        "schema_sql".into(),
        "CREATE TABLE specimen(id INTEGER PRIMARY KEY, value TEXT)".into(),
    );
    let create_outcome = adapter
        .dispatch(CrudVerb::Create, create_ctx)
        .expect("create table succeeds");
    assert_eq!(create_outcome.status, CrudStatus::Success);

    // read schema
    let read_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Read);
    let read_outcome = adapter
        .dispatch(CrudVerb::Read, read_ctx)
        .expect("read table metadata succeeds");
    assert_eq!(read_outcome.status, CrudStatus::Success);

    // find (metadata summary)
    let find_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Find);
    let find_outcome = adapter
        .dispatch(CrudVerb::Find, find_ctx)
        .expect("find table succeeds");
    assert_eq!(find_outcome.status, CrudStatus::Success);

    // list tables
    let mut list_ctx = CrudContext::new(CrudDomain::Sqlite, CrudObjectKind::Table, CrudVerb::List);
    list_ctx
        .options
        .insert("database_path".into(), db_path.to_str().unwrap().into());
    let list_outcome = adapter
        .dispatch(CrudVerb::List, list_ctx)
        .expect("list tables succeeds");
    assert_eq!(list_outcome.status, CrudStatus::Success);

    // delete table
    let delete_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Delete);
    let delete_outcome = adapter
        .dispatch(CrudVerb::Delete, delete_ctx)
        .expect("delete table succeeds");
    assert_eq!(delete_outcome.status, CrudStatus::Success);

    // read after delete should error
    let read_ctx_missing = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Read);
    let err = adapter
        .dispatch(CrudVerb::Read, read_ctx_missing)
        .expect_err("read should fail after drop");
    assert_eq!(err.kind, CrudErrorKind::NotFound);
}

#[test]
fn create_without_schema_errors() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("table_no_schema.sqlite");
    let adapter = SqliteTableAdapter::new(SqliteConnectionConfig::default());

    let ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Create);
    let err = adapter
        .dispatch(CrudVerb::Create, ctx)
        .expect_err("create without schema should fail");
    assert_eq!(err.kind, CrudErrorKind::InvalidInput);
}

#[test]
fn table_update_adds_column() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("table_update.sqlite");
    let adapter = SqliteTableAdapter::new(SqliteConnectionConfig::default());
    let base = SqliteBaseAdapter::new(SqliteConnectionConfig::default());

    let base_ctx = CrudContext::new(CrudDomain::Sqlite, CrudObjectKind::Base, CrudVerb::Create)
        .with_option("database_path", db_path.to_str().unwrap());
    base.dispatch(CrudVerb::Create, base_ctx).unwrap();

    let mut create_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Create);
    create_ctx.options.insert(
        "schema_sql".into(),
        "CREATE TABLE specimen(id INTEGER PRIMARY KEY, label TEXT)".into(),
    );
    adapter
        .dispatch(CrudVerb::Create, create_ctx)
        .expect("create table succeeds");

    let mut update_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Update);
    update_ctx.options.insert(
        "update_sql".into(),
        "ALTER TABLE specimen ADD COLUMN description TEXT DEFAULT 'n/a'".into(),
    );
    let outcome = adapter
        .dispatch(CrudVerb::Update, update_ctx)
        .expect("update executes");
    assert_eq!(outcome.status, CrudStatus::Success);

    let read_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Read);
    let read_outcome = adapter
        .dispatch(CrudVerb::Read, read_ctx)
        .expect("read succeeds after update");
    let columns_meta = read_outcome
        .metadata
        .get("columns")
        .expect("columns metadata present");
    let column_summary = match columns_meta {
        prontodb::lib::core::crud::MetadataValue::List(values) => values.join(";"),
        other => panic!("unexpected metadata variant: {:?}", other),
    };
    assert!(
        column_summary.contains("description"),
        "updated schema should include new column"
    );
}

#[test]
fn table_backup_and_restore_roundtrip() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("table_backup.sqlite");
    let backup_path = temp.path().join("specimen_backup.json");
    let adapter = SqliteTableAdapter::new(SqliteConnectionConfig::default());
    let base = SqliteBaseAdapter::new(SqliteConnectionConfig::default());

    let base_ctx = CrudContext::new(CrudDomain::Sqlite, CrudObjectKind::Base, CrudVerb::Create)
        .with_option("database_path", db_path.to_str().unwrap());
    base.dispatch(CrudVerb::Create, base_ctx).unwrap();

    let mut create_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Create);
    create_ctx.options.insert(
        "schema_sql".into(),
        "CREATE TABLE specimen(id INTEGER PRIMARY KEY, value TEXT, blob BLOB)".into(),
    );
    adapter
        .dispatch(CrudVerb::Create, create_ctx)
        .expect("create table succeeds");

    seed_rows(db_path.to_str().unwrap());

    let mut backup_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Backup);
    backup_ctx
        .options
        .insert("target_path".into(), backup_path.to_str().unwrap().into());
    let backup_outcome = adapter
        .dispatch(CrudVerb::Backup, backup_ctx)
        .expect("table backup succeeds");
    assert_eq!(backup_outcome.status, CrudStatus::Success);
    assert!(backup_path.exists(), "backup file should exist");

    let backup_doc: JsonValue = serde_json::from_str(&fs::read_to_string(&backup_path).unwrap())
        .expect("backup JSON parse");
    assert_eq!(backup_doc["table"], "specimen");
    assert_eq!(backup_doc["rows"].as_array().unwrap().len(), 2);

    // drop table using adapter delete to ensure restore recreates it
    let delete_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Delete);
    adapter
        .dispatch(CrudVerb::Delete, delete_ctx)
        .expect("delete succeeds");

    let mut restore_ctx = ctx_with_table(db_path.to_str().unwrap(), "specimen", CrudVerb::Restore);
    restore_ctx
        .options
        .insert("source_path".into(), backup_path.to_str().unwrap().into());
    let restore_outcome = adapter
        .dispatch(CrudVerb::Restore, restore_ctx)
        .expect("restore succeeds");
    assert_eq!(restore_outcome.status, CrudStatus::Success);

    let conn = open_connection(db_path.to_str().unwrap());
    let row_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM specimen", [], |row| row.get(0))
        .unwrap();
    assert_eq!(row_count, 2);
    let value: String = conn
        .query_row("SELECT value FROM specimen WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(value, "alpha");
}
