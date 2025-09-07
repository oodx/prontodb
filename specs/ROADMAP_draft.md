# ProntoDB — Roadmap & Milestones

This roadmap lays out the staged evolution of ProntoDB, aligned with the Refined PRD. Versions are incremental (v0.1 → v0.3), with a separate backlog for possible future expansions.

---

## Milestone v0.1 — **MVP (Core Engine)**
**Goal:** Deliver a minimal, working single‑binary KV store for CLI and multi‑agent workflows.

**Scope (must‑have):**
- **Engine & storage:**
  - SQLite via system `libsqlite3`
  - `PRAGMA journal_mode=WAL`, `synchronous=NORMAL`, `busy_timeout=5000ms`
- **System setup:**
  - `install` / `uninstall [--purge]`
  - Default XDG+ paths:
    - Config: `~/.local/etc/odx/prontodb/pronto.conf`
    - Data: `~/.local/data/odx/prontodb/`
    - Libs: `~/.local/lib/odx/prontodb/`
    - Bin: `~/.local/bin/prontodb`
  - Env var shims: `PRONTO_DB`, `PRONTO_SECURITY`
  - Default admin `admin/pronto!` seeded on install (override via env)
- **Schema:**
  - Per‑(project,namespace) tables
  - TTL namespaces via `admin create-cache`
  - System tables: `sys_namespaces`, `sys_caches`, `sec_users`, `sec_api_keys`, `sec_sessions`
- **Core KV commands:** `set`, `get`, `del`, `keys|ls`, `scan`
- **Streams:** one transaction per namespace, auth preamble required (pass→user order or apikey)
- **Exit codes:** 0=success, 2=not found/expired, other=error
- **Testing:** integration harness (`test.rs`), TDD spec (`test-tdd.rs`), `bin/test.sh`

**Estimate:** ~15 story points

---

## Milestone v0.2 — **Quality & Features**
**Goal:** Strengthen usability and polish around data access & administration.

**Scope:**
- **Import/export:**
  - `export <project.namespace>` → TSV (base64 encoded values)
  - `import <project.namespace>` → TSV ingest
- **Stream grammar polish:**
  - Support `meta:ns=` alias for `meta:path=`
  - Enforce TTL flags only in TTL namespaces (error code 2 if misused)
  - Support `--stream` flag for `keys` and `scan` commands
- **Discovery commands:** `projects`, `namespaces`, `nss` — include metadata output
- **Cache admin:** `admin create-cache`, `admin set-cache`, `admin drop-cache`
- **Testing:** add TDD coverage for TTL expiry, delimiter validation, stream preamble order

**Estimate:** ~13 story points

---

## Milestone v0.3 — **Expansion & Integrations**
**Goal:** Broaden usability and start bridging CLI with external tools.

**Scope:**
- **Filesystem mirror:**
  - `export-fs` / `import-fs` to map `project.namespace.key__ctx` → `project/namespace/context/key.json`
  - `--overwrite` flag, conflict resolution TBD
- **Eviction policies:** implement `timeout_sec`, `evict_on_read`, `max_items`
- **Sessions & API keys:**
  - `proto auth` flows for issuing and using session keys
- **Advanced features:**
  - Compression (zstd) for stored values (optional)
  - Encryption via `age` or `openssl`
- **Server stub:** optional HTTP/gRPC endpoints (`GET/PUT/DELETE /ns/key`) for remote use

**Estimate:** ~10 story points

---

## Backlog / Future Considerations
- **Single‑table schema mode** (alternative to per‑namespace tables)
- **Per‑user quotas, resource isolation**
- **Indexing & secondary queries** (beyond key prefix scan)
- **Multi‑node / replication layer** (out of scope until later)
- **Plugin API** for custom storage backends or auth providers
- **Observability hooks:** metrics, tracing, audit logs
- **Policy DSL** for namespace access rules

---

## Delivery Strategy
- Tight TDD feedback loops (unit + integration harnesses)
- Keep bundle size lean (single binary + XDG paths)
- Iterative, test‑driven increments
- Each milestone produces a shippable CLI with docs & test suite

---

