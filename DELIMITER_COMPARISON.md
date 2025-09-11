# ProntoDB Delimiter Comparison: Dot vs Colon

## Current System (Dot Delimiter)

### Basic Usage
```bash
# Current working examples:
prontodb set myapp.config.debug true
prontodb get myapp.config.debug
prontodb del myapp.config.debug
prontodb keys myapp.config
prontodb scan myapp.config

# With flags (alternative syntax):
prontodb set -p myapp -n config debug true
prontodb get -p myapp -n config debug
```

### Multi-user and Cursors
```bash
# Current multi-user examples:
prontodb --user alice --cursor dev set myapp.config.host "dev.local"
prontodb --user bob --cursor prod set myapp.config.host "prod.com"

# Cursor management:
prontodb cursor set dev /path/to/dev.db
prontodb cursor set prod /path/to/prod.db
prontodb cursor list
```

### Current Limitations for Document Ingestion
```bash
# These BREAK with current dot system:
prontodb set bashfx.v3.0.section.1.2.3 content  ❌ (v3.0 breaks namespace)
prontodb set api.v2.1.endpoint.auth content      ❌ (v2.1 breaks namespace)
prontodb set document.v1.2.1.chapter.1.verse.10 content  ❌ (v1.2.1 breaks)

# Forced workarounds (ugly):
prontodb set bashfx.v3_0.section_1_2_3 content  😞 (loses readability)
prontodb set api.v2_1.endpoint_auth content      😞 (unnatural)
```

---

## Proposed System (Colon Delimiter)

### Basic Usage (Colon Replaces Dot)
```bash
# Equivalent functionality with colons:
prontodb set myapp:config:debug true
prontodb get myapp:config:debug  
prontodb del myapp:config:debug
prontodb keys myapp:config
prontodb scan myapp:config

# Flag syntax would also change:
prontodb set -p myapp -n config debug true  # Same
# OR new unified syntax:
prontodb set myapp:config:debug true         # Preferred
```

### Multi-user and Cursors
```bash
# Multi-user with colons:
prontodb --user alice --cursor dev set myapp:config:host "dev.local"  
prontodb --user bob --cursor prod set myapp:config:host "prod.com"

# Cursor management (same commands, different data):
prontodb cursor set dev /path/to/dev.db
prontodb cursor set prod /path/to/prod.db
prontodb cursor list
```

### Document Ingestion (NOW POSSIBLE!)
```bash
# Biblical addressing works perfectly:
prontodb set bashfx:v3.0:1:2:3 "Section content..."           ✅
prontodb set api:v2.1:auth:requirements:3.2.1 "Auth specs..."  ✅
prontodb set bible:kjv:genesis:1:1 "In the beginning..."       ✅

# Natural version names with dots:
prontodb set specs:v1.2.1:requirements:core "Core reqs..."     ✅
prontodb set system:release.2024.12:config:db.host "prod.db"   ✅

# Complex nested addressing:
prontodb get bashfx:v3.0:philosophy:core.principles:1.1.2      ✅
prontodb get specs:latest:api.design:section.2.3:examples      ✅
```

---

## Side-by-Side Examples

### Configuration Management
```bash
# CURRENT (Dot)                          # PROPOSED (Colon)
prontodb set myapp.config.db.host val    prontodb set myapp:config:db.host val
prontodb set myapp.config.api.key val    prontodb set myapp:config:api.key val
prontodb get myapp.config.db.host        prontodb get myapp:config:db.host
```

### Multi-Agent Workflows  
```bash
# CURRENT (Dot)                                           # PROPOSED (Colon)
prontodb --user worker1 set tasks.queue.item1 data       prontodb --user worker1 set tasks:queue:item1 data
prontodb --user monitor get system.health.status         prontodb --user monitor get system:health:status
prontodb --cursor prod get myapp.config.db.url           prontodb --cursor prod get myapp:config:db.url
```

### Cross-Agent Workflows from CROSS_AGENT_WORKFLOWS.md
```bash
# CURRENT (Dot)                                           # PROPOSED (Colon)
prontodb --user orchestrator --cursor main \             prontodb --user orchestrator --cursor main \
  set workflow.status "initializing"                       set workflow:status "initializing"

prontodb --user worker1 --cursor tasks \                 prontodb --user worker1 --cursor tasks \
  set queue.task_1 "pending"                               set queue:task_1 "pending"

prontodb --user monitor --cursor logs \                  prontodb --user monitor --cursor logs \
  set metrics.worker1.tasks_completed 42                   set metrics:worker1:tasks_completed 42
```

### Document Ingestion Examples
```bash
# CURRENT (BROKEN - Cannot handle versions with dots)    # PROPOSED (WORKS!)
prontodb ingest doc.md proj v1.0        ❌ BREAKS       prontodb ingest doc.md proj:v1.0        ✅

# After ingestion:
prontodb get proj.v1.0.section.1        ❌ BREAKS       prontodb get proj:v1.0:section:1        ✅
prontodb get proj.v1.0.1.2.3           ❌ BREAKS       prontodb get proj:v1.0:1:2:3           ✅

# Biblical addressing:
# BashFX 1:10 becomes:
prontodb get bashfx.v3.0.1.10          ❌ BREAKS       prontodb get bashfx:v3.0:1:10          ✅

# Complex subsections:
prontodb get doc.v2.1.chapter.1.2.3    ❌ BREAKS       prontodb get doc:v2.1:chapter:1.2.3    ✅
```

---

## Migration Impact Analysis

### What Changes
```bash
# API calls change from dots to colons:
OLD: prontodb set myapp.config.debug true
NEW: prontodb set myapp:config:debug true

# Flag-based syntax could remain the same:
UNCHANGED: prontodb set -p myapp -n config debug true
```

### What Stays the Same
```bash
# These remain identical:
prontodb help
prontodb version  
prontodb cursor list
prontodb cursor set name /path/to/db
prontodb --user alice --cursor dev [command]
prontodb backup --output /backups
```

### Current Data Migration
```bash
# Migration script would need to:
# 1. Export all current data
prontodb scan | while IFS='=' read -r key value; do
    # Convert key: myapp.config.debug -> myapp:config:debug
    new_key=$(echo "$key" | sed 's/\./:/g')
    echo "prontodb set $new_key \"$value\"" >> migration.sh
done

# 2. Re-import with new format
bash migration.sh
```

---

## Breaking Changes Summary

### Commands That Change
- All `set/get/del/keys/scan` commands use `:` instead of `.`
- Document ingestion becomes possible and natural
- Biblical addressing works perfectly

### Commands That Don't Change  
- `help`, `version`, `cursor`, `backup`, `admin`
- Global flags: `--user`, `--cursor`, `--database`
- Multi-user and cursor functionality (same behavior)

### New Capabilities Unlocked
- Document ingestion with complex versioning
- Biblical addressing: `document:version:chapter:verse`
- Natural version names: `v1.2.3`, `release.2024.12`
- Complex nested keys: `api.design:section.2.3`

---

## Recommendation

**The colon delimiter enables the document ingestion feature and creates a more consistent, biblical addressing system.** 

**Pros:**
- ✅ Document ingestion becomes possible
- ✅ Biblical addressing throughout
- ✅ Natural version names (v1.2.3)
- ✅ Consistent mental model
- ✅ More powerful addressing

**Cons:**  
- ❌ Breaking change for existing users
- ❌ Migration required for existing data
- ❌ Different from common CLI conventions (most use dots)

**Verdict:** Worth it for early-stage product (v0.5.0 → v0.6.0) to unlock document management capabilities and create architectural consistency.