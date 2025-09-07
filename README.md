# ProntoDB

**ProntoDB** is a **single-binary, RSB-style, string-only key–value store CLI** built on top of **SQLite (system `libsqlite3`)**.
It acts as a **fast virtual wrapper for multi-agent workflows**, giving you hierarchical namespaces, TTL-aware caches, streaming inserts with meta-directives, and optional filesystem mirroring for `grep`/`rg`-friendly exploration — all with zero daemons and near-SQLite performance.

---

## ✨ Core Concepts

- **RSB Discipline**
  - Built with the [RSB architecture](./rsb-architecture.md).
  - Command macros: `bootstrap!`, `pre_dispatch!`, `dispatch!`, `args!`.
  - **stdout = data**, **stderr = status**.
  - **No `clap`**, no REPL, no daemons. One binary, one command at a time.

- **Hierarchical Namespaces**
  - Addressing: `project.namespace.key__context`.
  - `project` + `namespace` form a physical SQLite table.
  - `__context` suffix creates an overlay variant of a key.
  - Delimiter defaults to `.` but is configurable via `--ns-delim` or `pronto.conf`.

- **TTL Namespaces (Caches)**
  - Created via `admin create-cache`.
  - Keys include `created_at` + `ttl_sec`.
  - TTL enforcement only valid in TTL namespaces.
  - Lazy expiry on read/write; `--include-expired` flag for inspection.
  - Exit code `2` signals “miss/expired” without being a hard error.

- **Security**
  - Default admin user: `admin / pronto!` (override with `PRONTO_ADMIN_PASS`).
  - Streams require auth preamble unless disabled (`security.required=false` or `PRONTO_SECURITY=false`).
  - Auth preamble order enforced: `meta:sec:pass=...; meta:sec:user=...;` or `meta:sec:apikey=...;`.
  - Session tokens/API keys: v0.2+ feature.
  - Hashing/encryption delegated to system tools (`age`, `openssl`, `sha256sum`).

- **XDG+ Paths**
  - Config: `~/.local/etc/odx/prontodb/pronto.conf`
  - Data: `~/.local/data/odx/prontodb/`
  - Libs: `~/.local/lib/odx/prontodb/`
  - Bin: `~/.local/bin/prontodb`
  - Env shims:
    - `PRONTO_DB=/path/to/db` → alias for `-d`
    - `PRONTO_SECURITY=false` → disables required auth

- **Filesystem Mirror (future v0.3)**
  - `export-fs` / `import-fs` map keys to directories:
    - `kb.recipes.pasta` → `kb/recipes/pasta.json`
    - `kb.recipes.pasta__italian` → `kb/recipes/italian/pasta.json`

---

## 🛠 Features (MVP)

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
  - Auth order: `meta:sec:pass=...; meta:sec:user=...;` or `meta:sec:apikey=...;`
  - Namespace: `meta:path=project.namespace;` or `meta:project=...; meta:namespace=...;`
  - Alias: `meta:ns=...;`
  - Directives: `meta:delim=.|:`, `meta:ttl=SECONDS` (only valid in TTL ns)
  - Data tokens: `key=value; key__ctx=value;`
  - One transaction per namespace

**Exit codes**
- `0` → success
- `2` → key not found / expired (non-fatal miss)
- other non-zero → error

---

## 📦 Install

```bash
# build
cargo build --release

# link binary
mkdir -p ~/.local/bin
cp target/release/prontodb ~/.local/bin/prontodb

# initialize system tables + admin user
prontodb install
```

Config: `~/.local/etc/odx/prontodb/pronto.conf`

```ini
ns_delim = "."
security.required = true
busy_timeout_ms = 5000
```

---

## 📖 Usage Patterns

### Set & Get
```bash
prontodb -p kb -n recipes set pasta '{"s":"red"}' --json
prontodb -p kb -n recipes get pasta --json
```

### Namespaces & Keys
```bash
prontodb projects
prontodb namespaces -p kb
prontodb -p kb -n recipes keys
```

### TTL Namespace
```bash
prontodb admin create-cache kb.recipes timeout=60
prontodb -p kb -n recipes set temp_note "hello" --ttl 10
sleep 12
prontodb -p kb -n recipes get temp_note   # exit code 2, no stdout
```

### Streaming with Auth
```bash
echo "meta:sec:pass=pronto!; meta:sec:user=admin; meta:path=kb.recipes; note=hi;" | \
  prontodb stream
```

### Backup vs Export
```bash
# Full snapshot + config (optionally encrypted)
prontodb backup --out ./snap.tar.gz --age-recipient user@key

# Export a namespace to TSV
prontodb export kb.recipes > recipes.tsv

# Import TSV back
prontodb import kb.recipes < recipes.tsv
```

### Filesystem Mirror (v0.3)
```bash
prontodb export-fs --root ./topics --ns kb.recipes
# → ./topics/kb/recipes/pasta.json
# → ./topics/kb/recipes/italian/pasta.json
```

---

## 🔐 Security Examples
```bash
# Seeded admin (default: admin/pronto!)
prontodb proto auth pass=pronto!; user=admin;

# Add a new user
prontodb admin user=sam;pass=123;pref1=x;

# Request API key (v0.2+)
prontodb proto auth sec=myapikey;
```

---

## 🧪 Testing
- `test.rs` — MVP integration tests
- `test-tdd.rs` — TDD/spec-driven tests (future features `#[ignore]`)
- Tests run under isolated `$HOME` with XDG layout
- `bin/test.sh` helper script

---

## 🚀 Roadmap
See [Pronto Db Roadmap](./Pronto%20Db%20Roadmap).

- **v0.1** — Core engine, CRUD, TTL ns, auth, backup, stream, JSON/b64, env shims
- **v0.2** — TSV import/export, stream grammar polish, discovery, cache admin polish
- **v0.3** — export-fs/import-fs, eviction policies, sessions/API keys, compression, encryption, server stub
