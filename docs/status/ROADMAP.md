ProntoDB Roadmap (Consolidated)

Status: MVP-first delivery using a single `kv` table schema; advanced features follow.

Guiding Principles
- Ship a pragmatic MVP (core CRUD + discovery + TTL create) quickly.
- Keep storage simple for v0.1 (single `kv` table keyed by project/namespace/key/context).
- Enforce clean CLI behavior and exit codes; TDD where feasible.
- Defer streams/auth/lifecycle/import-export to later milestones.

Milestones

M0 — Foundation & Dispatch
- CLI basics: help/unknown/no-args behaviors
- Addressing: dot and flags; custom delimiter; key validation
- Dispatcher: route set/get/del/keys/scan/projects/namespaces
- XDG paths and DB path resolution

M1 — Core KV Operations
- `set`/`get`/`del` wired to SQLite storage (kv table)
- `keys`/`scan` with optional prefix filtering
- Exit codes: 0 success; 2 miss/expired for get; others = error

M2 — TTL Namespaces (Subset)
- `admin create-cache project.namespace timeout=SEC` (records in sys_namespaces)
- Enforce TTL rules:
  - Default TTL applied in TTL namespaces when `--ttl` not provided
  - Reject `--ttl` in non-TTL namespaces
- Lazy expiry on get (delete expired, return MISS)

M3 — Discovery
- `projects` lists distinct projects
- `namespaces -p <project>` lists namespaces for project
- `nss` optional aggregate (may follow after M1/M2)

Deferred (Post-MVP)
- Streams & Auth preamble enforcement
- Lifecycle: install/uninstall/backup
- Import/export TSV; JSON canonicalization; `--b64`
- Filesystem mirror; eviction policies; sessions/API keys
- Per-namespace physical tables (migrate from single-table once stable)

Implementation Notes
- Storage: keep current `kv` + `sys_namespaces` schema; WAL + busy_timeout enabled.
- Validation: disallow active delimiter in key; support `__context` suffix.
- Docs reflect MVP storage choice; table-per-namespace moved to backlog.

Success Criteria
- Core CRUD + discovery + TTL-create pass integration tests under isolated XDG env.
- Clear, minimal CLI UX and accurate exit codes.

