use std::fs;
use std::io::Write;

use prontodb::lib::adpt::sqlite::{
    SqliteBaseAdapter, SqliteConnectionConfig, SqliteRecordAdapter, SqliteTableAdapter,
};
use prontodb::lib::core::crud::{
    CrudContext, CrudDomain, CrudErrorKind, CrudObjectKind, CrudResource, CrudStatus, CrudVerb,
    MetadataValue,
};
use rusqlite::{Connection, OpenFlags};
use tempfile::tempdir;

fn ctx_for_base(path: &str, verb: CrudVerb) -> CrudContext {
    let mut ctx = CrudContext::new(CrudDomain::Sqlite, CrudObjectKind::Base, verb);
    ctx.options.insert("database_path".into(), path.to_string());
    ctx
}

#[test]
fn base_create_creates_sqlite_file() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("base_create.sqlite");
    let adapter = SqliteBaseAdapter::new(SqliteConnectionConfig::default());

    let ctx = ctx_for_base(db_path.to_str().unwrap(), CrudVerb::Create);
    let outcome = adapter
        .dispatch(CrudVerb::Create, ctx)
        .expect("create should succeed");

    assert_eq!(outcome.status, CrudStatus::Success);
    assert!(
        db_path.exists(),
        "database file should exist after creation"
    );
}

#[test]
fn base_read_reports_metadata() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("base_read.sqlite");
    let adapter = SqliteBaseAdapter::new(SqliteConnectionConfig::default());

    let ctx = ctx_for_base(db_path.to_str().unwrap(), CrudVerb::Create);
    adapter.dispatch(CrudVerb::Create, ctx).unwrap();

    let read_ctx = ctx_for_base(db_path.to_str().unwrap(), CrudVerb::Read);
    let outcome = adapter
        .dispatch(CrudVerb::Read, read_ctx)
        .expect("read should succeed");

    let size = outcome
        .metadata
        .get("size_bytes")
        .expect("size metadata")
        .clone();

    match size {
        MetadataValue::Integer(value) => assert!(value > 0),
        other => panic!("unexpected metadata type: {:?}", other),
    }
}

#[test]
fn base_backup_writes_copy_to_target() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("base_backup.sqlite");
    let backup_path = temp.path().join("base_backup.sqlite.bak");
    let adapter = SqliteBaseAdapter::new(SqliteConnectionConfig::default());

    // create file and insert a row to ensure non-empty database
    let ctx = ctx_for_base(db_path.to_str().unwrap(), CrudVerb::Create);
    adapter.dispatch(CrudVerb::Create, ctx).unwrap();
    insert_sample_row(db_path.to_str().unwrap());

    let mut backup_ctx = ctx_for_base(db_path.to_str().unwrap(), CrudVerb::Backup);
    backup_ctx.options.insert(
        "target_path".into(),
        backup_path.to_str().unwrap().to_string(),
    );

    let outcome = adapter
        .dispatch(CrudVerb::Backup, backup_ctx)
        .expect("backup should succeed");

    assert_eq!(outcome.status, CrudStatus::Success);
    assert!(backup_path.exists(), "backup path should exist");
}

#[test]
fn base_restore_copies_from_source() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("base_restore.sqlite");
    let source_path = temp.path().join("restore_source.sqlite");
    let adapter = SqliteBaseAdapter::new(SqliteConnectionConfig::default());

    // Prepare source file with distinctive bytes
    let mut file = fs::File::create(&source_path).unwrap();
    writeln!(file, "custom-backup").unwrap();

    let mut restore_ctx = ctx_for_base(db_path.to_str().unwrap(), CrudVerb::Restore);
    restore_ctx.options.insert(
        "source_path".into(),
        source_path.to_str().unwrap().to_string(),
    );

    let outcome = adapter
        .dispatch(CrudVerb::Restore, restore_ctx)
        .expect("restore should succeed");

    assert_eq!(outcome.status, CrudStatus::Success);
    let restored = fs::read(&db_path).unwrap();
    assert!(
        restored.starts_with(b"custom-backup"),
        "destination should contain source bytes"
    );
}

#[test]
fn record_adapter_still_unsupported() {
    let adapter_record = SqliteRecordAdapter::new(SqliteConnectionConfig::default());

    let result_record = adapter_record.dispatch(
        CrudVerb::Find,
        CrudContext::new(CrudDomain::Sqlite, CrudObjectKind::Record, CrudVerb::Find),
    );
    assert!(result_record.is_err());
    assert_eq!(result_record.unwrap_err().kind, CrudErrorKind::Unsupported);
}

#[test]
fn table_adapter_advertises_supported_verbs() {
    let adapter_table = SqliteTableAdapter::new(SqliteConnectionConfig::default());
    let capabilities = adapter_table.capabilities();
    let object = CrudObjectKind::Table;

    for verb in [
        CrudVerb::Create,
        CrudVerb::Read,
        CrudVerb::Update,
        CrudVerb::Delete,
        CrudVerb::List,
        CrudVerb::Find,
        CrudVerb::Backup,
        CrudVerb::Restore,
    ] {
        assert!(
            capabilities.allows(&object, verb),
            "table adapter should support {:?}",
            verb
        );
    }

    for verb in [CrudVerb::Alias] {
        assert!(
            !capabilities.allows(&object, verb),
            "table adapter should not advertise {:?}",
            verb
        );
    }
}

fn insert_sample_row(path: &str) {
    let mut flags = OpenFlags::SQLITE_OPEN_READ_WRITE;
    flags.insert(OpenFlags::SQLITE_OPEN_NO_MUTEX);
    let conn = Connection::open_with_flags(path, flags).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS specimen(key TEXT PRIMARY KEY, value TEXT)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO specimen(key, value) VALUES('a', 'b')",
        [],
    )
    .unwrap();
}
