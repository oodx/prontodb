ProntoDB Tasks (Story-Pointed)

Scale: XS=1, S=2, M=3, L=5, XL=8

M0 — Foundation & Dispatch
- Help/unknown/no-args behaviors (XS)
- Addressing parse + custom delimiter (done) (—)
- Key validation (delimiter restriction) integration (S)
- Dispatcher: route commands (S)
- XDG paths usage and DB resolution (XS)

M1 — Core KV
- Implement `set` wired to storage (S)
- Implement `get` with MISS=2 semantics (S)
- Implement `del` (XS)
- Implement `keys` with optional prefix (S)
- Implement `scan` with optional prefix (S)
- Basic discovery: `projects`, `namespaces -p` (S)
- Output/UX polish (help text, errors) (S)

M2 — TTL Subset
- `admin create-cache project.namespace timeout=SEC` (S)
- Enforce TTL rules in `set` (S)
- Lazy expiry on `get` (done in storage) (—)

Infra & Tests
- Add CLI integration tests for M1 (M)
- Add TTL tests for M2 rules (S)
- CI-friendly test harness using isolated HOME (XS)

Documentation
- Consolidate ROADMAP/PRD/TASKS (done) (—)
- Update README to match MVP storage model (S)

Backlog (Post-MVP)
- Streams/auth error path + full parser (L)
- Lifecycle: install/uninstall/backup (L)
- TSV import/export, JSON canonicalization, `--b64` (L)
- Filesystem mirror, eviction policies, sessions (XL)
- Per-namespace physical tables + migration (XL)

