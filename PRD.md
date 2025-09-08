ProntoDB — Product Requirements (MVP Consolidated)

Overview
- Single-binary, CLI key–value store on SQLite.
- Addressing: `project.namespace.key__context`; custom delimiter (default `.`).
- XDG paths; WAL mode; clear exit codes; no daemon.
- MVP storage: single `kv` table keyed by project/namespace/key/context (+ TTL via expires_at).

Tenets
- stdout = data, stderr = status; simple flags, no clap.
- Deterministic, small surface; one-shot commands.
- Defer complex features until core CRUD is stable.

CLI Surface (MVP)
```
prontodb [-p PROJECT] [-n NAMESPACE] [-d DB] [--ns-delim C] <command> ...
```

Core Commands
- set <k|project.namespace.key[__ctx]> <value> [--ttl SECONDS]
- get <k|project.namespace.key[__ctx]>
- del <k|project.namespace.key[__ctx]>
- keys [prefix]
- scan [prefix]

Discovery
- projects
- namespaces -p <PROJECT>

Admin (TTL subset)
- admin create-cache <project.namespace> timeout=SECONDS

Exit Codes
- 0 = success; 2 = not found/expired (MISS); other non-zero = error.

Validation
- Active delimiter cannot appear in key (reject in CLI before write).
- `__context` suffix allowed; included in addressing, stored as context column.
- TTL rules:
  - If namespace is TTL and `--ttl` omitted, apply default TTL.
  - If namespace is not TTL and `--ttl` provided, reject with error.

Paths & Config
- XDG: data `~/.local/data/odx/prontodb/pronto.db`, config `~/.local/etc/odx/prontodb/pronto.conf`.
- `PRONTO_DB` overrides DB path.

Non-Goals for MVP
- Streams/auth preamble; lifecycle (install/uninstall/backup); import/export TSV; JSON canonicalization; `--b64`.
- Per-namespace physical tables (planned post-MVP via migration).

Success Criteria
- Core CRUD + discovery + TTL-create pass tests in isolated env; correct exit codes; clear errors.

