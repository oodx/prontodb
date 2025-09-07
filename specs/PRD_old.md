# ProntoDB — Product Requirements Document (Refined)

**ProntoDB** is a **single-binary, RSB-style, string-only CLI** built on top of **SQLite** (system `libsqlite3`).  
Addressing model: **`project.namespace.key__context`** with configurable delimiter (default `.`).  
Supports **TTL-enabled namespaces**, **stream meta-directives**, **simple auth**, **XDG+ paths**, and optional **filesystem mirroring** for `grep`/`rg` workflows.  
Bias: tiny, deterministic, embed-friendly; not a server like CockroachDB.

---

## 0. Vision & Tenets
- **Purpose:** Fast, composable KV store for multi-agent and CLI workflows.
- **RSB discipline:**
  - `stdout = data`, `stderr = status`
  - Uses RSB function ordinality and macros (`bootstrap!`, `pre_dispatch!`, `dispatch!`, `args!`)
  - No `clap`; string parsing only
- **No daemon, no REPL** — always a one-shot CLI (future: optional server stub)
- **SQLite WAL** enabled by default for concurrency (`journal_mode=WAL`, `synchronous=NORMAL`, `busy_timeout=5000ms`)
- **Config via XDG+ paths** (`$XDG_CONFIG_HOME`, `$XDG_DATA_HOME`, `$XDG_LIB_HOME`; defaults to `~/.local/etc|data|lib/odx/prontodb`). Flags override config.
- **Security:** default admin `admin / pronto!` (override via `PRONTO_ADMIN_PASS`)
- **Non-goals:** distributed clustering, full SQL query API, custom crypto primitives (delegate to `age`, `openssl`, `sha256sum`)

---

## 1. Architecture
- **Engine:** system `libsqlite3`
- **Schema:** per-(project,namespace) tables
  - Standard ns:
    ```sql
    CREATE TABLE ns_<project>_<namespace>(
      k TEXT PRIMARY KEY,
      v BLOB NOT NULL
    );
    ```
  - TTL ns (auto-created via `admin create-cache`):
    ```sql
    CREATE TABLE ns_<project>_<namespace>__ttl(
      k TEXT PRIMARY KEY,
      v BLOB NOT NULL,
      created_at INTEGER NOT NULL,
      ttl_sec INTEGER NOT NULL
    );
    ```
- **System tables:**
  - `sys_namespaces(project TEXT, namespace TEXT, kind CHECK(kind IN ('std','ttl')), table_name, delim, PRIMARY KEY(project,namespace))`
  - `sys_caches(project TEXT, namespace TEXT, timeout_sec INTEGER, evict_on_read INTEGER DEFAULT 1, max_items NULL, PRIMARY KEY(project,namespace))`
  - `sec_users`, `sec_api_keys`, `sec_sessions`

---

## 2. Paths & Config
- **Config:** `~/.local/etc/odx/prontodb/pronto.conf`
  - Keys:
    - `ns_delim="."` (default delimiter)
    - `security.required=true|false`
    - `busy_timeout_ms=5000`
- **Data:** `~/.local/data/odx/prontodb/`
- **Binaries:** `~/.local/bin/prontodb` (symlink)
- **Libs:** `~/.local/lib/odx/prontodb/`
- **Env shims:**
  - `PRONTO_DB=/path/to/db` → alias for `-d`
  - `PRONTO_SECURITY=false` → disables required auth (same as config)

---

## 3. Addressing & Keys
- **Canonical form:** `project.namespace.key__context`
- **Alternative flags:** `-p <project> -n <namespace> <key>`
- **Delimiter:** default `.`; override with `--ns-delim <char>`
- **Context suffix:** `__ctx` reserved for overlays (e.g. `protocols.sleeping.wind_down__tom`)
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
- `namespaces -p <PROJECT>`
- `nss`
- `admin create-cache <project.namespace> timeout=SECONDS [evict_on_read=1; max_items=N; ...]`
- `admin set-cache <project.namespace> key=val;...`
- `admin drop-cache <project.namespace>`

**Streams**
- `stream`
  - Requires auth preamble unless disabled
  - Auth order enforced:
    - `meta:sec:pass=...; meta:sec:user=...;`
    - OR `meta:sec:apikey=...;`
  - Namespace:
    - `meta:path=project.namespace;`
    - OR `meta:project=...; meta:namespace=...;`
    - Alias supported: `meta:ns=...;`
  - Other directives: `meta:delim=.|:`, `meta:ttl=SECONDS` (only valid in TTL ns)
  - Data tokens: `key=value; key__ctx=value;`
  - One transaction per namespace

**Exit codes**
- `0` → success (operation ok, value found)
- `2` → key not found / expired (non-fatal miss)
- other non-zero → error

---

## 5. Security & Auth
- Default admin seeded: `admin / pronto!` (override with `PRONTO_ADMIN_PASS`)
- Auth commands (v0.2):
  ```bash
  prontodb admin user=sam;pass=123;pref1=x;
  prontodb proto auth pass=123; user=sam;
  prontodb proto auth sec=myapikey;
  ```
- Sessions & API keys: v0.2 feature
- Hashing/crypto: delegate to `age`, `openssl`, `sha256sum`

---

## 6. Testing
- `test.rs` — MVP integration (happy path)
- `test-tdd.rs` — spec-driven (future tests marked `#[ignore]`)
- Tests run under isolated `$HOME` with XDG layout
- `bin/test.sh` helper script

---

## 7. Roadmap
- **v0.1** — core engine, CRUD, TTL ns, auth, backup, stream, JSON/b64, env shims, exit-code 2 for misses, key validation everywhere
- **v0.2** — TSV import/export, stream grammar polish, discovery commands, cache admin polish
- **v0.3** — export-fs/import-fs (directory mapping), per-namespace eviction (`max_items`), sessions & API keys, compression (zstd), encryption (via age/openssl), HTTP/gRPC server stub

---

