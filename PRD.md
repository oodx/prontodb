# ProntoDB — Product Requirements Document (PRD)

## Purpose

ProntoDB is a **lightweight, file-based key–value database** built on SQLite and orchestrated with the **RSB architecture**. It is designed for small tool ecosystems where CLI-first, string-based interaction is prioritized over embedding. It emphasizes **namespaces, streams, and JSON support**, with minimal dependencies and RSB-aligned conventions.

---

## Architecture

* **Engine:** SQLite (system `libsqlite3` preferred; optional static bundle).
* **Interface:** Pure string-based CLI, no REPL.
* **Framework:** RSB (`bootstrap!`, `pre_dispatch!`, `dispatch!`, `Args` methods).
* **Discipline:**

  * **stdout = data**
  * **stderr = logs/status**
* **Schema:**

  * Default model: single table `kv(ns TEXT, k TEXT, v BLOB, PRIMARY KEY(ns,k))`.
  * Option: one table per namespace (user preference; natural separation).
  * System tables auto-initialized: `sys`, `sec`.

---

## Features

### Core Commands

* `init` → create schema and system tables.
* `set <key> <val|->` → insert/replace (`-` = read from stdin).
* `get <key>` → fetch value.
* `del <key>` → delete key.
* `keys [prefix]` → list keys in namespace.
* `scan [prefix]` → list `key<TAB>value` rows.
* `nss` → list namespaces.
* `export [ns]` → dump namespace as TSV (`key<TAB>base64(value)`).
* `import [ns]` → load TSV from stdin.
* `stream` → consume KV streams with meta-directives.
* `backup` → archive DB/data to file.
* `install` / `uninstall` → create/remove XDG-compliant directories and links.

### Namespace Management

* Hierarchical namespaces supported (`a:b:c` by default).
* Configurable delimiter (default `:`; can override via flag or `pronto.conf`).
* Namespace handling:

  * CLI `-n` option sets active namespace.
  * Qualified keys (e.g. `a:b:key`) override active namespace.

### JSON Support

* `--json` flag on `set/get/scan/stream`.
* On `set`, validates and canonicalizes JSON before storage.
* On `get/scan`, pretty-prints JSON or emits structured JSON array.

### Streaming & Meta-Directives

* **Input format:** `token; token; ...`.
* Tokens:

  * `meta:ns=value` → switch active namespace.
  * `meta:delim=value` → set namespace delimiter.
  * `meta:sec:user=u; meta:sec:secret=p;` → authenticate before stream accepted.
  * `key=value` → insert key/value under current namespace.
* Transactions: all stream operations wrapped atomically.
* Extensible: new `meta:*` directives can be added (TTL, compression toggles, etc).

### Security Layer

* **System table `sec`** manages users and API keys.
* Example: `pronto admin user=sam;pass=123;pref1=x;pref2=y`.
* Auth methods:

  * Username + password.
  * API key (`sec:apikey`).
* Session keys: generated via external tools (e.g. `ssh-keygen`, `md5sum`) invoked in shell; ProntoDB does not embed crypto.
* Security features configurable/disable-able via `pronto.conf`.

### Configuration & Paths

* Config file: `$XDG_CONFIG_HOME/odx/prontodb/pronto.conf` (or fallback `~/.local/etc/odx/prontodb/pronto.conf`).
* Data files: `$XDG_DATA_HOME/odx/prontodb` (or fallback `~/.local/data/odx/prontodb`).
* Binaries: `$XDG_LIB_HOME/odx/prontodb/bin` symlinked into `~/.local/bin/odx/`.
* Configurable base path for multi-setup or testing.

### Backup & Install

* `prontodb install` → creates config/data/bin dirs under XDG base paths.
* `prontodb uninstall` → removes ProntoDB dirs and symlinks.
* `prontodb backup` → dumps DB + config into a tarball.

---

## Optional / Backlog Features

* **TTL/Expiry:** support `set --ttl 60s`; purge expired keys on scan/read.
* **Compression (zstd):** optional transparent per-value compression.
* **Encryption (AES-GCM):** passphrase-based per-value encryption.
* **Server mode:** optional micro-HTTP layer (`GET/PUT/DELETE /ns/key`).

---

## Non-Goals

* No REPL shell (always one-shot CLI commands).
* No embedded crypto (all crypto delegated to shell tools if needed).
* No heavyweight server; keep binary small and snappy.

---

## Deliverables

1. **Cargo.toml** with RSB + rusqlite, JSON enabled by default.
2. **main.rs** RSB-compliant CLI, dispatch-based.
3. **System table bootstrapping** (`sys`, `sec`).
4. **Config loader** for pronto.conf, flag override support.
5. **Streaming parser** with meta-directives.
6. **Install/uninstall + backup commands**.
7. Documentation of commands + examples.
