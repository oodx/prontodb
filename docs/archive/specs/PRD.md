
# ProntoDB — Product Requirements Document (Refined + Roadmap Details)

**ProntoDB** is a **single-binary, RSB-style, string-only CLI** built on top of **SQLite** (system `libsqlite3`).  
Addressing model: **`project.namespace.key__context`** with configurable delimiter (default `.`).  
Supports **TTL-enabled namespaces**, **stream meta-directives**, **simple auth**, **XDG+ paths**, and optional **filesystem mirroring** for `grep`/`rg` workflows.  
Bias: tiny, deterministic, embed‑friendly; not a server like CockroachDB.

---

## 0. Vision & Tenets
- **Purpose:** Fast, composable KV store for multi‑agent and CLI workflows.
- **RSB discipline:** `stdout = data`, `stderr = status`.  
  - Uses RSB function ordinality & macros: `bootstrap!`, `pre_dispatch!`, `dispatch!`, `args!`  
  - No `clap`; string parsing only.
- **No daemon, no REPL** — one‑shot CLI (future optional server stub).  
- **SQLite WAL** enabled by default (`journal_mode=WAL`, `synchronous=NORMAL`, `busy_timeout=5000ms`).  
- **Config via XDG+** (`$XDG_CONFIG_HOME`, `$XDG_DATA_HOME`, `$XDG_LIB_HOME`; defaults to `~/.local/etc|data|lib/odx/prontodb`). Flags override.  
- **Security:** default admin `admin/pronto!`, overridable via `PRONTO_ADMIN_PASS`.  
- **Non‑goals:** distributed clustering, full SQL API, custom crypto primitives (delegate to `age`, `openssl`, `sha256sum`).  

---

## 1. Architecture
- **Engine:** system `libsqlite3`
- **Schema:** per‑(project,namespace) tables
  - Standard ns: `ns_<project>_<namespace>(k TEXT PRIMARY KEY, v BLOB NOT NULL)`
  - TTL ns (via `admin create-cache`): `ns_<project>_<namespace>__ttl(k TEXT PRIMARY KEY, v BLOB NOT NULL, created_at INTEGER NOT NULL, ttl_sec INTEGER NOT NULL)`
- **System tables:**
  - `sys_namespaces(project TEXT, namespace TEXT, kind CHECK(kind IN ('std','ttl')), table_name, delim, PRIMARY KEY(project,namespace))`
  - `sys_caches(project TEXT, namespace TEXT, timeout_sec INTEGER, evict_on_read INTEGER DEFAULT 1, max_items NULL, PRIMARY KEY(project,namespace))`
  - `sec_users`, `sec_api_keys`, `sec_sessions`

---

## 2. Paths & Config
- **Config:** `~/.local/etc/odx/prontodb/pronto.conf`
  - `ns_delim="."` (default)
  - `security.required=true|false`
  - `busy_timeout_ms=5000`
- **Data:** `~/.local/data/odx/prontodb/`
- **Binaries:** `~/.local/bin/prontodb`
- **Libs:** `~/.local/lib/odx/prontodb/`
- **Env shims:**
  - `PRONTO_DB=/path/to/db` → alias for `-d`
  - `PRONTO_SECURITY=false` → sets `security.required=false`

---

## 3. Addressing & Keys
- **Canonical form:** `project.namespace.key__context`
- **Flags alternative:** `-p <PROJECT> -n <NAMESPACE> <key>`
- **Delimiter:** default `.`; override with `--ns-delim <char>`
- **Flag sugar:** `--user bob` or `--context foo` append `__<value>` automatically
- **Context suffix:** `__ctx` reserved for overlays
- **Validation:** keys may not contain the active delimiter; enforced in all codepaths

---

## 4. CLI Surface (MVP)
```
prontodb [-p PROJECT] [-n NAMESPACE] [-d DB] [--ns-delim C] [--json] [--b64] <command> ...
```

**Lifecycle**
- `install`
- `uninstall [--purge]`
- `backup [--out PATH] [--age-recipient X | --age-identity Y]`

**Core KV**
- `set <k|project.namespace.key[__ctx]> <v|-> [--ttl SECONDS]`
- `get <k|project.namespace.key[__ctx]> [--include-expired]`
- `del <k|project.namespace.key[__ctx]>`
- `keys [prefix] | ls [prefix] [--stream]`
- `scan [prefix] [--json] [--stream] [--include-expired]`

**Namespace mgmt**
- `projects`
- `namespaces -p <PROJECT>` (with metadata)
- `nss`
- `admin create-cache <project.namespace> timeout=SECONDS [evict_on_read=1; max_items=N; ...]`
- `admin set-cache <project.namespace> key=val;...`
- `admin drop-cache <project.namespace>`

**Streams**
- `stream`
  - Auth preamble required unless disabled
  - Auth order: `meta:sec:pass=...; meta:sec:user=...;` (pass→user) or `meta:sec:apikey=...;`
  - Namespace: `meta:path=project.namespace;` or `meta:project=...; meta:namespace=...;`
  - Alias supported: `meta:ns=...;`
  - Directives: `meta:delim=.|:`, `meta:ttl=SECONDS` (only valid in TTL ns)
  - Data tokens: `key=value; key__ctx=value;`
  - One transaction per namespace

**Exit codes**
- `0` → success (operation ok, value found)
- `2` → key not found / expired (non‑fatal miss)
- other non‑zero → error

---

## 5. Security & Auth
- Default admin seeded: `admin / pronto!` (override with `PRONTO_ADMIN_PASS`)
- Example usage:
  ```bash
  prontodb admin user=sam;pass=123;pref1=x;
  prontodb proto auth pass=123; user=sam;
  prontodb proto auth sec=myapikey;
  ```
- Sessions & API keys: v0.2 feature
- Hashing/crypto via system tools (`age`, `openssl`, `sha256sum`)

---

## 6. Testing & Ops
- `test.rs` — MVP integration
- `test-tdd.rs` — spec-driven (future features `#[ignore]`)
- Unit tests: delimiter parsing, TTL expiry & include‑expired, preamble order, key validation
- Integration: install → projects → namespaces → set/get/del → scan/ls → backup → uninstall
- WAL smoke test (concurrent writers)
- Helper: `bin/test.sh` compiles & runs both harnesses under isolated `$HOME`

---

## 7. Roadmap

### v0.1 (MVP)
- Core engine, per‑(project,namespace) schema
- CRUD (`set/get/del`), `keys/ls`, `scan`
- TTL namespaces (`__ttl`)
- Auth (default admin), env shims, exit code 2 for misses
- Backup (tar.gz + optional `age` encryption)
- Streams with auth preamble (no REPL)
- JSON canonicalization & `--b64`
- Key validation (no delimiter in key)

### v0.2
- TSV `export` / `import`
- Stream grammar polish (`meta:path`, `meta:ns` alias, `--stream` for `keys/scan`)
- Discovery (`projects`, `namespaces`, metadata)
- Cache admin (create/set/drop)

### v0.3
- Filesystem mirror: `export-fs` / `import-fs`
  - `proj.ns.key` → `proj/ns/key.json`
  - `proj.ns.key__ctx` → `proj/ns/ctx/key.json`
- Per‑namespace eviction (`max_items`)
- Session keys & API key auth
- Compression (zstd) integration for backups
- Encryption via `age`/`openssl` wrappers
- HTTP/gRPC server stub (optional, non‑core)
