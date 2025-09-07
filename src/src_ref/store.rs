
use rusqlite::{Connection, params, OptionalExtension};
use crate::common::{Config, Addr, now_epoch};
use std::path::Path;

#[derive(Clone)]
pub struct Store { pub conn: Connection, pub busy_ms: i64 }

pub enum NsKind { Std, Ttl }

impl Store {
    pub fn open(cfg: &Config) -> rusqlite::Result<Self> {
        let conn = Connection::open(&cfg.db_path)?;
        conn.pragma_update(None, "journal_mode", &"WAL")?;
        conn.pragma_update(None, "synchronous", &"NORMAL")?;
        conn.pragma_update(None, "busy_timeout", &cfg.busy_timeout_ms)?;
        Ok(Self { conn, busy_ms: cfg.busy_timeout_ms })
    }

    pub fn install(&self, etc: &Path, data: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(etc)?;
        std::fs::create_dir_all(data)?;
        self.conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS sys_namespaces(
            project TEXT NOT NULL, namespace TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('std','ttl')),
            table_name TEXT NOT NULL, delim TEXT DEFAULT '.',
            PRIMARY KEY(project, namespace)
        );
        CREATE TABLE IF NOT EXISTS sys_caches(
            project TEXT NOT NULL, namespace TEXT NOT NULL,
            timeout_sec INTEGER NOT NULL, evict_on_read INTEGER DEFAULT 1, max_items INTEGER NULL,
            PRIMARY KEY(project, namespace)
        );
        CREATE TABLE IF NOT EXISTS sec_users(user TEXT PRIMARY KEY, pass TEXT NOT NULL);
        "#)?;
        // seed admin
        let exists: Option<i64> = self.conn
            .query_row("SELECT 1 FROM sec_users WHERE user='admin' LIMIT 1", [], |r| r.get(0))
            .optional()?;
        if exists.is_none() {
            let pass = std::env::var("PRONTO_ADMIN_PASS").unwrap_or_else(|_| "pronto!".to_string());
            self.conn.execute("INSERT INTO sec_users(user, pass) VALUES('admin', ?1)", params![pass])?;
        }
        Ok(())
    }

    pub fn uninstall(&self, data: &Path, etc: &Path, purge: bool) -> std::io::Result<()> {
        if purge {
            let _ = std::fs::remove_file(data.join("prontodb.sqlite3"));
            let _ = std::fs::remove_dir_all(data);
        }
        let _ = std::fs::remove_dir_all(etc);
        Ok(())
    }

    fn table_name(project: &str, ns: &str, kind: &NsKind) -> String {
        match kind {
            NsKind::Std => format!("ns_{}_{}", project, ns),
            NsKind::Ttl => format!("ns_{}_{}__ttl", project, ns),
        }
    }

    pub fn ensure_ns(&self, project: &str, ns: &str, delim: char) -> rusqlite::Result<()> {
        let exists: Option<String> = self.conn
            .query_row("SELECT table_name FROM sys_namespaces WHERE project=?1 AND namespace=?2",
                params![project, ns], |r| r.get(0)).optional()?;
        if exists.is_none() {
            let tn = Self::table_name(project, ns, &NsKind::Std);
            self.conn.execute(&format!("CREATE TABLE IF NOT EXISTS {}(k TEXT PRIMARY KEY, v BLOB NOT NULL)", tn), [])?;
            self.conn.execute(
                "INSERT OR IGNORE INTO sys_namespaces(project, namespace, kind, table_name, delim) VALUES (?1, ?2, 'std', ?3, ?4)",
                params![project, ns, tn, delim.to_string()]
            )?;
        }
        Ok(())
    }

    pub fn create_cache(&self, project: &str, ns: &str, timeout: i64, delim: char) -> rusqlite::Result<()> {
        let tn = Self::table_name(project, ns, &NsKind::Ttl);
        self.conn.execute(&format!("CREATE TABLE IF NOT EXISTS {}(k TEXT PRIMARY KEY, v BLOB NOT NULL, created_at INTEGER NOT NULL, ttl_sec INTEGER NOT NULL)", tn), [])?;
        self.conn.execute(
            "INSERT OR REPLACE INTO sys_namespaces(project, namespace, kind, table_name, delim) VALUES (?1, ?2, 'ttl', ?3, ?4)",
            params![project, ns, tn, delim.to_string()]
        )?;
        self.conn.execute(
            "INSERT OR REPLACE INTO sys_caches(project, namespace, timeout_sec, evict_on_read, max_items) VALUES (?1, ?2, ?3, 1, NULL)",
            params![project, ns, timeout]
        )?;
        Ok(())
    }

    pub fn ns_kind(&self, project: &str, ns: &str) -> rusqlite::Result<NsKind> {
        let k: String = self.conn.query_row(
            "SELECT kind FROM sys_namespaces WHERE project=?1 AND namespace=?2",
            params![project, ns], |r| r.get(0)
        )?;
        Ok(match k.as_str() { "ttl" => NsKind::Ttl, _ => NsKind::Std })
    }

    pub fn set(&self, addr: &Addr, value: &[u8], ttl_override: Option<i64>) -> rusqlite::Result<()> {
        let kind = self.ns_kind(&addr.project, &addr.namespace)?;
        match kind {
            NsKind::Std => {
                if ttl_override.is_some() { return Err(rusqlite::Error::ExecuteReturnedResults); }
                let tn = Self::table_name(&addr.project, &addr.namespace, &kind);
                let key = if let Some(ctx)=&addr.ctx { format!("{}__{}", addr.key, ctx) } else { addr.key.clone() };
                self.conn.execute(&format!("INSERT OR REPLACE INTO {}(k, v) VALUES (?1, ?2)", tn), params![key, value])?;
            }
            NsKind::Ttl => {
                let tn = Self::table_name(&addr.project, &addr.namespace, &kind);
                let def_ttl: i64 = self.conn.query_row(
                    "SELECT timeout_sec FROM sys_caches WHERE project=?1 AND namespace=?2",
                    params![&addr.project, &addr.namespace], |r| r.get(0)).unwrap_or(60);
                let ttl = ttl_override.unwrap_or(def_ttl);
                let key = if let Some(ctx)=&addr.ctx { format!("{}__{}", addr.key, ctx) } else { addr.key.clone() };
                self.conn.execute(&format!("INSERT OR REPLACE INTO {}(k, v, created_at, ttl_sec) VALUES (?1, ?2, ?3, ?4)", tn),
                    params![key, value, now_epoch(), ttl])?;
            }
        }
        Ok(())
    }

    pub fn get(&self, addr: &Addr, include_expired: bool) -> rusqlite::Result<Option<Vec<u8>>> {
        let kind = self.ns_kind(&addr.project, &addr.namespace)?;
        match kind {
            NsKind::Std => {
                let tn = Self::table_name(&addr.project, &addr.namespace, &kind);
                self.conn.query_row(&format!("SELECT v FROM {} WHERE k=?1", tn), params![fullkey(addr)], |r| r.get(0)).optional()
            }
            NsKind::Ttl => {
                let tn = Self::table_name(&addr.project, &addr.namespace, &kind);
                let row: Option<(Vec<u8>, i64, i64)> = self.conn.query_row(
                    &format!("SELECT v, created_at, ttl_sec FROM {} WHERE k=?1", tn),
                    params![fullkey(addr)], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).optional()?;
                if let Some((v, created, ttl)) = row {
                    let expired = now_epoch() >= created + ttl;
                    if expired && !include_expired { return Ok(None); }
                    Ok(Some(v))
                } else { Ok(None) }
            }
        }
    }

    pub fn del(&self, addr: &Addr) -> rusqlite::Result<usize> {
        let kind = self.ns_kind(&addr.project, &addr.namespace)?;
        let tn = Self::table_name(&addr.project, &addr.namespace, &kind);
        let n = self.conn.execute(&format!("DELETE FROM {} WHERE k=?1", tn), params![fullkey(addr)])?;
        Ok(n)
    }

    pub fn keys(&self, project: &str, ns: &str) -> rusqlite::Result<Vec<String>> {
        let kind = self.ns_kind(project, ns)?;
        let tn = Self::table_name(project, ns, &kind);
        let mut stmt = self.conn.prepare(&format!("SELECT k FROM {} ORDER BY k ASC", tn))?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        let mut out = Vec::new();
        for k in rows { out.push(k?); }
        Ok(out)
    }
}

fn fullkey(addr: &Addr) -> String {
    match &addr.ctx { Some(c) => format!("{}__{}", addr.key, c), None => addr.key.clone() }
}
