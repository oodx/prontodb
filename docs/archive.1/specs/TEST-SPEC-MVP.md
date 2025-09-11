# ProntoDB — TEST-SPEC-MVP.md (Reorganized for Practical Development)

This document reorganizes the test plan to match our actual MVP development roadmap. 
Each test case references a PRD section, states **Given/When/Then**, and tags milestone: **[M0]**, **[M1]**, **[M2]**, or **[later]**.

Conventions:
- `PRONTODB_BIN` environment variable points to the binary under test (defaults to `./target/debug/prontodb`).
- Each test runs under an **isolated HOME** (`$TMPDIR/.prontodb_test_*`) to avoid cross‑contamination.
- Exit codes: `0=OK`, `2=MISS` (not found/expired), `!=0 && !=2 = ERROR`.

---

## Milestone 0a: Foundation & Basic Infrastructure [M0a]

### 0a.1 help command shows usage [M0a]
- Given no other arguments
- When `prontodb help` or `prontodb --help`
- Then shows usage information and exits 0

### 0a.2 unknown command shows error [M0a]  
- Given unknown command `badcommand`
- When `prontodb badcommand`
- Then exits non-zero (not 2), stderr explains unknown command

### 0a.3 no arguments shows error [M0a]
- Given no arguments
- When `prontodb` with no args
- Then exits non-zero, suggests help

### 0a.4 dot address parsing works [M0a] - INDEPENDENT
- Given canonical address format
- When parsing `project.namespace.key`
- Then Address { project: "project", namespace: "namespace", key: "key", context: None }

### 0a.5 flag address parsing works [M0a] - INDEPENDENT  
- Given flag-based addressing
- When parsing `-p proj -n ns key`
- Then Address { project: "proj", namespace: "ns", key: "key", context: None }

### 0a.6 custom delimiter parsing [M0a] - INDEPENDENT
- Given `--ns-delim '|'`  
- When parsing `proj|ns|key` with delimiter `|`
- Then Address { project: "proj", namespace: "ns", key: "key", context: None }

### 0a.7 delimiter override integration [M0a] - AFTER BASIC ADDRESSING
- Given working dot/flag addressing
- When `prontodb --ns-delim '|' set proj|ns|key value`
- Then parses with pipe delimiter instead of default dot

### 0a.8 dispatcher integration [M0a] - AFTER ADDRESSING
- Given working address parsing (dot, flag, delimiter override)
- When `prontodb set -p proj -n ns key value`
- Then dispatcher correctly routes with parsed address

### 0a.9 table infrastructure [M0a] - AFTER ADDRESSING
- Given table infrastructure module
- When querying table schema and performing basic operations
- Then table query helpers work correctly with SQLite backend

### 0a.10 project infrastructure [M0a] - AFTER TABLES  
- Given project infrastructure module
- When managing project-to-table mappings and metadata
- Then project operations work with addressing

### 0a.11 namespace infrastructure [M0a] - AFTER PROJECTS  
- Given namespace infrastructure module
- When resolving namespace within project hierarchy
- Then namespace resolution works with project structure

### 0a.12 addressing + project/namespace/table integration [M0a] - CRITICAL
- Given addressing, tables, projects, and namespace modules working
- When resolving `project.namespace.key` to actual table operations
- Then addressing correctly maps through project→namespace→table hierarchy

### 0a.13 key set operation [M0a] - AFTER INTEGRATION
- Given complete addressing + project/namespace/table integration
- When `prontodb set project.namespace.key value`
- Then stores value using proper project→namespace→table resolution

### 0a.14 key get operation [M0a] - AFTER SET
- Given set operation working and stored value
- When `prontodb get project.namespace.key`
- Then retrieves value correctly, exits 0

### 0a.15 key delete operation [M0a] - AFTER GET
- Given set/get working with stored value
- When `prontodb del project.namespace.key`
- Then removes key, subsequent get returns MISS

### 0a.16 keys listing operations [M0a] - AFTER DEL
- Given multiple keys stored in namespace
- When `prontodb keys project.namespace` and `prontodb scan project.namespace`
- Then lists keys and key-value pairs respectively

### 0a.17 get missing key returns MISS [M0a] - AFTER CRUD
- Given working key CRUD operations
- When `prontodb get missing.namespace.key`
- Then exits 2 (MISS), empty stdout

---

## Milestone 0b: Admin System & Database Management [M0b]

### 0b.1 admin sub-dispatcher routing [M0b]
- Given admin command with subcommand
- When `prontodb admin help`, `prontodb admin create-db test.db`
- Then routes to admin subcommand handlers correctly

### 0b.2 basic SQLite CRUD integration [M0b]
- Given database file management system
- When `prontodb admin create-db test.db`, `prontodb admin switch-db test.db`
- Then creates new database and switches context

### 0b.3 database switching works [M0b]  
- Given multiple database files: `main.db`, `test.db`
- When `prontodb admin switch-db test.db` then `prontodb set key value`
- Then stores value in test.db, not main.db

### 0b.4 database rename works [M0b]
- Given existing database `old.db` 
- When `prontodb admin rename-db old.db new.db`
- Then renames file and updates internal references

### 0b.5 database delete works [M0b]
- Given existing database `temp.db`
- When `prontodb admin delete-db temp.db`
- Then removes database file safely

### 0b.6 admin table operations [M0b] - POWER USER
- Given working database system
- When `prontodb admin create-table`, `admin drop-table`, `admin list-tables`
- Then table management works correctly (power user operations)

### 0b.7 admin project operations [M0b] - POWER USER
- Given working database system
- When `prontodb admin create-project`, `admin drop-project`, `admin list-projects`
- Then project management works correctly (power user operations)

### 0b.8 admin namespace operations [M0b] - POWER USER
- Given working database system  
- When `prontodb admin create-namespace`, `admin drop-namespace`, `admin list-namespaces`
- Then namespace management works correctly (power user operations)

---

## Milestone 0c: Multi-Instance & Config Management [M0c]

### 0c.1 --select flag sets sticky database [M0c]
- Given clean system
- When `prontodb --select path/to/custom.db set key value`
- Then creates .prontorc file with database selection and uses custom.db

### 0c.2 .prontorc persistence works [M0c]
- Given existing .prontorc with database selection
- When `prontodb set key value` (without --select)
- Then uses database from .prontorc file

### 0c.3 --config flag overrides database [M0c]
- Given .prontorc points to default.db
- When `prontodb --config other.db get key`
- Then uses other.db, ignoring .prontorc for this command

### 0c.4 multi-instance isolation [M0c]
- Given two different .prontorc files in different directories
- When commands run from each directory
- Then each uses its own database context without interference

### 0c.5 basic set/get/del operations [M0c]
- Given working multi-instance system
- When `prontodb set test.ns.key value` then `prontodb get test.ns.key`
- Then returns value correctly using proper database context
---

## Milestone 1: Core KV Operations [M1]

### 1.1 set/get basic string [M1]
- Given clean database
- When `prontodb set test.namespace.key myvalue` then `prontodb get test.namespace.key`
- Then second command returns `myvalue` and exits 0

### 1.2 get missing key returns MISS [M1]
- Given clean database
- When `prontodb get missing.key`
- Then exits 2, empty stdout, stderr indicates not found

### 1.3 delete removes key [M1]  
- Given key exists: `prontodb set test.ns.key value`
- When `prontodb del test.ns.key` then `prontodb get test.ns.key`
- Then delete exits 0, get exits 2 (MISS)

### 1.4 canonical addressing `project.namespace.key` [M1]
- Given value set via full path: `prontodb set myproj.myns.mykey myvalue`
- When get via full path: `prontodb get myproj.myns.mykey`
- Then returns `myvalue`

### 1.5 flag addressing `-p/-n` [M1]
- Given value set via flags: `prontodb set -p myproj -n myns mykey myvalue`
- When get via flags: `prontodb get -p myproj -n myns mykey`  
- Then returns `myvalue`

### 1.6 delimiter override `--ns-delim` [M1]
- Given `--ns-delim '|'`
- When `prontodb --ns-delim '|' set proj|ns|key value` then `prontodb --ns-delim '|' get proj|ns|key`
- Then returns `value`

### 1.7 key validation: delimiter not allowed in key [M1]
- Given active delimiter `.`
- When `prontodb set -p proj -n ns "bad.key" value`
- Then exits non-zero (not 2), stderr explains delimiter restriction

### 1.8 context suffix `__ctx` reserved [M1]
- Given `prontodb set test.ns.key__context value`
- When `prontodb get test.ns.key__context`
- Then returns specific context value

### 1.9 keys command lists keys [M1]
- Given multiple keys set in namespace
- When `prontodb keys -p proj -n ns`
- Then lists keys in that namespace

### 1.10 scan command shows key-value pairs [M1]
- Given multiple keys set in namespace  
- When `prontodb scan -p proj -n ns`
- Then shows key=value pairs

### 1.11 keys/scan with prefix filtering [M1]
- Given keys: `test1`, `test2`, `other`
- When `prontodb keys -p proj -n ns test`
- Then returns only `test1`, `test2`

---

## Milestone 2: TTL & Caches [M2]

### 2.1 create TTL namespace [M2]
- Given clean system
- When `prontodb admin create-cache proj.ns timeout=5`
- Then creates TTL-enabled namespace with 5s default

### 2.2 TTL expiration works [M2]
- Given TTL namespace with timeout=2
- When `prontodb set proj.ns.key value`, wait 3 seconds, `prontodb get proj.ns.key`
- Then get returns MISS (exit 2)

### 2.3 explicit `--ttl` only in TTL namespaces [M2]
- Given standard namespace `proj.std`
- When `prontodb set proj.std.key value --ttl 10`
- Then exits non-zero (not 2), error explains TTL not allowed

### 2.4 explicit `--ttl` works in TTL namespaces [M2]
- Given TTL namespace `proj.cache`
- When `prontodb set proj.cache.key value --ttl 10`
- Then accepts and applies 10s TTL

---

## Milestone 3: Streams & Auth [M2]

### 3.1 stream requires auth by default [M2]
- Given default security settings
- When `echo "meta:path=proj.ns; key=value;" | prontodb stream`
- Then exits non-zero (not 2), requires authentication

### 3.2 disable security via config [M2]
- Given config file with `security.required=false`
- When `echo "meta:path=proj.ns; key=value;" | prontodb stream`
- Then succeeds, stores key-value

### 3.3 stream auth preamble order enforced [M2]
- Given enabled security
- When `echo "meta:sec:user=admin; meta:sec:pass=pronto!; meta:path=proj.ns; key=value;" | prontodb stream`
- Then error (pass must come before user)

### 3.4 stream auth preamble correct order [M2]
- Given enabled security  
- When `echo "meta:sec:pass=pronto!; meta:sec:user=admin; meta:path=proj.ns; key=value;" | prontodb stream`
- Then succeeds, stores key-value

### 3.5 stream transaction boundary per namespace [M2]
- Given stream with mixed namespaces
- When multiple operations for same namespace in one stream
- Then all operations for namespace are atomic

---

## Milestone 4: Discovery & Admin [M2]

### 4.1 projects command lists projects [M2]
- Given data in multiple projects
- When `prontodb projects`
- Then lists all project names

### 4.2 namespaces command lists namespaces [M2]
- Given data in project with multiple namespaces
- When `prontodb namespaces -p myproject`
- Then lists namespaces in that project

### 4.3 nss command lists all namespaces [M2]
- Given data across projects and namespaces
- When `prontodb nss`
- Then lists all namespaces across all projects

### 4.4 admin set-cache updates TTL parameters [M2]
- Given existing TTL namespace
- When `prontodb admin set-cache proj.ns timeout=10`
- Then updates default TTL to 10 seconds

### 4.5 admin drop-cache removes TTL namespace [M2]
- Given existing TTL namespace
- When `prontodb admin drop-cache proj.ns`
- Then removes TTL settings, namespace becomes standard

---

## Deferred to Later: Lifecycle & Advanced Features [later]

### L.1 JSON canonicalization [later]
- Given JSON with mixed spacing: `prontodb set key '{"b":1, "a":2}'`
- When `prontodb get key --json`  
- Then returns canonical JSON: `{"a":2,"b":1}`

### L.2 install seeds system and default admin [later]
- Given fresh system
- When `prontodb install`
- Then creates XDG dirs, database, seeds admin user `admin/pronto!`

### L.3 uninstall cleanup [later]
- Given installed system
- When `prontodb uninstall` vs `prontodb uninstall --purge`
- Then first keeps data, second removes everything

### L.4 backup/restore [later]
- Given data present
- When `prontodb backup --out file.tar.gz`
- Then creates valid backup archive

### L.5 import/export TSV [later]
- Given data in namespace
- When `prontodb export proj.ns --format tsv` then restore to clean system
- Then roundtrip preserves all data

### L.6 ls alias with --stream format [later]
- Given key-value data
- When `prontodb ls --stream`
- Then returns `meta:path=...; key=val; ...` format

### L.7 include-expired inspection [later]
- Given expired TTL key
- When `prontodb get key --include-expired`
- Then shows value with metadata, exits 0

---

## Test Priority by Milestone

### Milestone 0 Tests (Foundation)
**Must pass before any KV operations**:
- 0.1, 0.2, 0.3 (basic CLI)
- 0.4, 0.5 (argument parsing)

### Milestone 1 Tests (Core KV)
**Must pass for basic functionality**:
- 1.1, 1.2, 1.3 (set/get/del)
- 1.4, 1.5, 1.6, 1.7 (addressing)
- 1.9, 1.10, 1.11 (keys/scan)

### Milestone 2 Tests (Advanced Features)
**Nice to have for full v0.1**:
- 2.x (TTL features)
- 3.x (streaming)
- 4.x (discovery/admin)

### Later Tests
**Post-v0.1 features**:
- L.x (lifecycle, import/export, advanced features)

---

## Exit Code Reference
- **0**: Success
- **2**: Key not found (MISS) - for get operations on missing/expired keys
- **1**: Error - for invalid commands, bad arguments, system errors
- **Other non-zero**: Other specific errors

## Test Environment Setup
- Use `PRONTO_DB` environment variable to point to isolated test database
- Each test should use `tempfile::TempDir` for isolation
- Clean database state between tests
- Use `assert_cmd` crate for CLI testing
