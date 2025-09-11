# ProntoDB — TEST-SPEC.md (Spec-as-Tests)

This document translates the **Refined PRD** into a structured, versioned **test plan**. It is the contract that drives TDD.
Each test case references a PRD section, states **Given/When/Then**, and tags milestone: **[v0.1]**, **[v0.2]**, **[v0.3]**, or **[backlog]**.

Conventions:
- `PRONTODB_BIN` environment variable points to the binary under test (defaults to `./target/debug/prontodb`).
- Each test runs under an **isolated HOME** (`$TMPDIR/.prontodb_test_*`) to avoid cross‑contamination.
- Exit codes: `0=OK`, `2=MISS` (not found/expired), `!=0 && !=2 = ERROR`.

---

## 0. Lifecycle & Setup

### 0.1 install seeds system tables and default admin [v0.1]
- Given a fresh HOME
- When `prontodb install` runs
- Then creates XDG dirs and DB; seeds admin user `admin/pronto!`

### 0.2 uninstall removes system artifacts; `--purge` removes data [v0.1]
- Given an installed system
- When `prontodb uninstall` → config/data remain; When `prontodb uninstall --purge` → all removed

### 0.3 backup produces snapshot; optional age encryption [v0.1]
- Given data present
- When `prontodb backup --out file.tar.gz [--age-recipient X | --age-identity Y]`
- Then file exists and is a valid tar.gz (encryption is passthrough to system tools)

---

## 1. Addressing, Keys, Delimiters

### 1.1 canonical addressing `project.namespace.key__ctx` [v0.1]
- Given a value set via full path
- When `get` via full path
- Then value equals

### 1.2 flag addressing `-p/-n` [v0.1]
- Given a value set via flags
- When `get` via flags
- Then value equals

### 1.3 delimiter override `--ns-delim` [v0.1]
- Given `--ns-delim '|'`
- When `set kb|recipes|g v` then `get` same path
- Then value equals

### 1.4 key validation: delimiter not allowed in key [v0.1]
- Given active delimiter `.`
- When `set -p kb -n recipes "bad.key" x`
- Then error (non‑zero, not 2) and stderr explains

### 1.5 context suffix `__ctx` reserved [v0.1]
- Given `p.n.k__x` written
- When `get p.n.k__x`
- Then returns specific overlay

---

## 2. Core KV

### 2.1 set/get basic string [v0.1]
- Set `p.n.k v`; get `p.n.k` → `v`

### 2.2 set/get JSON `--json` canonicalization [v0.1]
- Set JSON with mixed spacing; get `--json` prints canonical JSON

### 2.3 delete removes key [v0.1]
- After `del`, `get` returns MISS (exit 2, empty stdout)

### 2.4 keys and scan list entries [v0.1]
- `keys` yields keys; `scan --json` yields k/v documents

### 2.5 keys/scan with prefix [v0.1]
- Only keys with prefix returned

### 2.6 ls alias & `--stream` format (doc/data stream) [v0.2]
- `ls --stream` returns `meta:path=...; key=val; ...`

---

## 3. TTL Namespaces (Caches)

### 3.1 create TTL namespace [v0.1]
- `admin create-cache p.n timeout=2` creates `__ttl` table; metadata row in `sys_namespaces/sys_caches`

### 3.2 default write into TTL ns without explicit `--ttl` uses namespace default [v0.1]
- After create-cache with timeout=2, `set p.n.k X`; after 3s, `get` is MISS

### 3.3 explicit `--ttl` allowed only in TTL namespaces [v0.2]
- `set p.std_ns.k X --ttl 10` → error (non‑zero, not 2) with message
- `set p.ttl_ns.k X --ttl 10` → OK

### 3.4 include-expired inspection [v0.2]
- After expiry, `get --include-expired` shows value (and metadata if implemented), exit 0

### 3.5 eviction policy placeholders [v0.3]
- Coverage for `evict_on_read`, `max_items` (ignored until implemented; mark tests `#[ignore]`)

---

## 4. Streams

### 4.1 auth required by default [v0.1]
- `echo "meta:path=p.n; k=v;" | prontodb stream` → ERROR (non‑zero, not 2)

### 4.2 disable security via config/env [v0.1]
- Write `pronto.conf: security.required=false`; stream without auth → OK

### 4.3 preamble order enforced (pass→user) [v0.1]
- `meta:sec:user=admin; meta:sec:pass=pronto!;` → ERROR
- `meta:sec:pass=pronto!; meta:sec:user=admin;` → OK

### 4.4 alias `meta:ns=` equals `meta:path=` [v0.2]
- Both select the same namespace

### 4.5 `meta:ttl` only valid for TTL namespaces [v0.2]
- Stream into std ns with `meta:ttl=5` → error
- Stream into TTL ns with `meta:ttl=5` → OK

### 4.6 transaction boundary = one per namespace [v0.1]
- Interleaved tokens for same ns are atomic (basic smoke)

### 4.7 multi‑namespace stream semantics [v0.3]
- TBD rules; mark tests `#[ignore]`

---

## 5. Discovery & Admin

### 5.1 projects/namespaces/nss list content [v0.1]
- After writes, `projects` contains project; `namespaces -p P` contains NS; `nss` includes both

### 5.2 admin set/drop‑cache [v0.2]
- Update cache params and drop TTL namespace; verify metadata reflected

---

## 6. Export/Import/Backup

### 6.1 export TSV + import TSV roundtrip [v0.2]
- Export `p.n` to TSV; uninstall/purge; reinstall; import TSV; values restored

### 6.2 backup snapshot [v0.1]
- Produces tar.gz; optional `age` flags shell out

### 6.3 filesystem mirror export‑fs/import‑fs [v0.3]
- Verify directory layout mapping and roundtrip JSON

---

## 7. Security

### 7.1 default admin usable [v0.1]
- `meta:sec:pass=pronto!; meta:sec:user=admin;` authenticates

### 7.2 apikey flow placeholder [v0.3]
- `meta:sec:apikey=...` behavior reserved; tests `#[ignore]`

---

## 8. Concurrency & WAL

### 8.1 busy_timeout honored [v0.1]
- Set `busy_timeout_ms` small; simulate write lock; next writer waits/fails accordingly

### 8.2 multi‑process write smoke [v0.1]
- Two writers via `stream` on same ns; both complete; database consistent

---

## 9. Exit‑Code Semantics

### 9.1 get miss returns 2 [v0.1]
- Missing key and expired key → exit 2, empty stdout, stderr “not found/expired”

### 9.2 error codes for invalid flags / misuse [v0.1]
- Invalid delimiter in key; unknown command; reversed preamble → non‑zero (not 2)

---

## Tag Index
- [v0.1] MVP (core engine & flows)
- [v0.2] Quality & polish (import/export, stream grammar, discovery)
- [v0.3] Expansion (fs mirror, eviction, sessions/API, compression/encryption, HTTP/gRPC)
- [backlog] Post‑v0.3 ideas (plugins, quotas, DSL, observability, replication)
