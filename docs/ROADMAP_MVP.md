# ProntoDB MVP Roadmap - Pragmatic Implementation

Based on TEST-SPEC.md requirements, focusing on functional MVP delivery.

## Core Object Model

ProntoDB is built on **6 core objects** that represent the fundamental domain:

### 1. **Database** (db_crud.rs)
- **Represents**: A SQLite database file
- **Operations**: create, switch, rename, delete databases
- **Relationships**: Contains Tables, Projects, and Namespaces

### 2. **Table** (tables.rs) 
- **Represents**: A SQLite table within a database
- **Operations**: schema management, query helpers (insert/select/update/delete)
- **Relationships**: Belongs to Database, used by Projects and Namespaces

### 3. **Project** (projects.rs)
- **Represents**: A logical project grouping within a database
- **Operations**: project CRUD, project-to-table mapping, project metadata
- **Relationships**: Belongs to Database, contains Namespaces, maps to Tables

### 4. **Namespace** (namespace.rs)
- **Represents**: A logical namespace within a project
- **Operations**: namespace-to-table mapping, namespace metadata
- **Relationships**: Belongs to Project, maps to Tables, contains Keys

### 5. **Key** (keys.rs)
- **Represents**: A key-value pair within a namespace
- **Operations**: set, get, delete, scan, list operations
- **Relationships**: Lives in Namespace, identified by Address

### 6. **Address** (addressing.rs)
- **Represents**: A parsed path identifier (`project.namespace.key`)
- **Operations**: parsing, validation, flag resolution
- **Relationships**: Identifies Keys within Project.Namespace hierarchy

## Core Object Relationships

```
Database (db_crud.rs)
├── contains Tables (tables.rs)
├── contains Projects (projects.rs)
├── contains Namespaces (namespace.rs)
└── selected by Config (.prontorc)

Address (addressing.rs) → identifies → Key (keys.rs)
                                      └── lives in → Namespace
                                                     └── belongs to → Project
                                                                     └── maps to → Table
                                                                                   └── exists in → Database

Hierarchy: Database → Project → Namespace → Key
           Database → Table (used by Project/Namespace)
```

## Everything Else is Infrastructure

All other components are **helpers, integrations, or APIs** built on these core objects:

### Infrastructure Layers:
- **Dispatcher** - Command routing API over core objects
- **Storage** - SQLite connection wrapper used by Database/Table objects  
- **Admin Commands** - Power user API over Database/Table/Namespace objects
- **Lifecycle** - Install/backup operations orchestrating core objects
- **Config** - Multi-instance support (.prontorc) for Database selection
- **XDG** - Path management helper for Database/Config location
- **Auth/Streams/TTL** - Advanced features layered on core object operations

### Data Flow:
1. **Address** parsing → **Project** resolution → **Namespace** resolution → **Table** location → **Key** operations  
2. **Database** selection (via Config) → **Table** access → **Project** operations → **Namespace** operations → **Key** CRUD
3. **Admin Commands** → direct manipulation of **Database/Table/Project/Namespace** objects
4. **Lifecycle Operations** → orchestration across all 6 core objects

## Development Philosophy: Isolated CRUD First → Integration Later

**Core Principle**: Build each object with complete CRUD lifecycle in isolation, then add integration layers.

### Systematic Construction Approach:
1. **Isolated CRUD Operations**: Each core object gets complete create/read/update/delete operations
2. **Direct Dispatcher Exposure**: Simple command routing for testing (no admin dispatch complexity initially) 
3. **Dependency-Respecting Build Order**: Database → Tables → Projects → Namespaces → Keys
4. **Complete Lifecycle Validation**: Full CRUD testing before advancing to next object
5. **Integration Phases**: Only after all pieces verified independently

## Phase 1: Foundation & Dispatch (Basic Working System)
**Goal**: Working dispatcher with XDG paths, flag parsing, addressing foundation

### Core Infrastructure (✅ Completed):
- ✅ **XDG Paths** (xdg.rs) - Path management 
- ✅ **Dispatcher** (dispatcher.rs) - Command routing with flag parsing
- ✅ **Storage** (storage.rs) - SQLite connection wrapper
- ✅ **Addressing** (addressing.rs) - Path identifier parsing foundation

### Missing Foundation Elements:
- **Flag parsing enhancement** - Core flags for all CRUD operations
- **Address validation** - Basic project.namespace.key structure validation
- **Direct CRUD dispatcher routes** - Each object gets direct commands for testing

## Phase 2: Object CRUD (Isolated Operations)
**Goal**: Each core object has complete, tested CRUD lifecycle with direct dispatcher access

### Dependency-Respecting Build Order:

#### Step 1: **Database CRUD** (db_crud.rs)
- **Direct Commands**: `prontodb db-create`, `db-switch`, `db-rename`, `db-delete`
- **Operations**: create/switch/rename/delete databases
- **Multi-Instance Support**: 
  - Database switching with `--select` flag and sticky cursor files
  - Per-user config files: `--user=name` uses `.prontorc-name` instead of base `.prontorc`
  - Configuration management with .prontorc file family
- **Dependencies**: Uses Storage, XDG (already complete)

#### Step 2: **Table CRUD** (tables.rs) 
- **Direct Commands**: `prontodb table-create`, `table-drop`, `table-list`, `table-describe`
- **Operations**: create/drop/list/describe tables + schema management, query helpers
- **Dependencies**: Requires Database CRUD (from Step 1)

#### Step 3: **Project CRUD** (projects.rs)
- **Direct Commands**: `prontodb project-create`, `project-drop`, `project-list`, `project-describe`
- **Operations**: project CRUD, project-to-table mapping, project metadata
- **Dependencies**: Requires Database + Table CRUD (Steps 1-2)

#### Step 4: **Namespace CRUD** (namespace.rs)
- **Direct Commands**: `prontodb ns-create`, `ns-drop`, `ns-list`, `ns-describe`  
- **Operations**: namespace-to-table mapping, namespace metadata within project
- **Dependencies**: Requires Database + Table + Project CRUD (Steps 1-3)

#### Step 5: **Key CRUD** (keys.rs)
- **Direct Commands**: `prontodb key-set`, `key-get`, `key-del`, `key-scan`, `key-list`
- **Operations**: set/get/del/scan/keys operations using full object hierarchy
- **Dependencies**: Requires ALL previous objects (Steps 1-4)

#### Step 6: **Address Integration** (addressing.rs enhancement)
- **Enhanced Addressing**: Full project.namespace.key resolution with all objects
- **Flag Addressing**: Complete --project/--namespace/--key support  
- **Dot Addressing**: project.namespace.key parsing and validation
- **Delimiter Overrides**: Custom delimiters for addressing
- **Dependencies**: Requires all objects for full resolution (Steps 1-5)

### Validation Strategy:
- **Direct Dispatcher Commands**: Each CRUD operation exposed through simple commands for testing
- **Complete Lifecycle Testing**: Full create/read/update/delete validation before moving to next object  
- **No Integration Complexity**: Build each object in isolation until all pieces work independently
- **Regression Prevention**: Each step must be solid before building dependencies

## Phase 3: Integration & User Commands
**Goal**: Transform isolated CRUD operations into integrated user experience

### Step 1: **User Command Integration**
- **Traditional Commands**: `prontodb set/get/del` using full object resolution
- **Address Resolution**: project.namespace.key → Database → Project → Namespace → Table → Key operations
- **Flag Integration**: `--project`, `--namespace`, `--key` flags working with all objects

### Step 2: **Lifecycle Operations** (lifecycle.rs)
- **Commands**: `prontodb install`, `uninstall`, `backup`, `restore`  
- **Operations**: Install/uninstall/backup/restore using all 6 core objects
- **Integration**: Orchestrates across Database/Table/Project/Namespace/Key operations

### Step 3: **Admin Interface** (Optional Enhancement)
- **Admin Dispatch**: `prontodb admin <cmd>` for power user access
- **Direct Object Manipulation**: Access to isolated CRUD operations for debugging
- **UAT Interface**: User Acceptance Testing layer for complex scenarios

## Implementation Benefits
1. **Isolated Testing**: Each object proven before integration complexity
2. **Clear Dependencies**: Build order respects object relationships  
3. **Direct Access**: Simple dispatcher commands for development/debugging
4. **Regression Prevention**: Complete CRUD lifecycle validation before advancement
5. **Systematic Construction**: No shortcuts, proper foundation building
6. **Parallel Development**: Objects can be built independently after dependencies satisfied

## Success Criteria by Phase:

### Phase 1 Success:
- ✅ All foundation infrastructure working (XDG, Storage, Dispatcher, Addressing)
- ✅ Flag parsing and basic address validation operational
- ✅ Direct CRUD dispatcher routing established

### Phase 2 Success (Per Object):
- ✅ Each object has complete CRUD operations working in isolation
- ✅ Direct dispatcher commands for testing each object independently  
- ✅ Full lifecycle validation (create → read → update → delete → verification)
- ✅ Clean dependencies respected (no forward references)

### Phase 3 Success:
- ✅ `prontodb set project.ns.key value` stores correctly using full hierarchy
- ✅ `prontodb get project.ns.key` retrieves correctly using address resolution
- ✅ `prontodb get missing.key` returns MISS (exit 2) with proper error handling
- ✅ All integration working without regression to isolated CRUD operations

## Future Milestones (After Core Foundation Complete)

### Enhanced Features Phase:
**Goal**: Build on solid foundation with advanced capabilities

#### Multi-Instance & Config Management
- **Config Management**: .prontorc file management and discovery  
- **Database Selection**: --select/--config flags with sticky cursor files
- **User Isolation**: --user=name flag uses .prontorc-name files instead of base .prontorc

#### Advanced Operations  
- **Scanning & Listing**: keys/scan commands with prefix filtering
- **JSON Support**: --json flag for structured output
- **Import/Export**: TSV format support with roundtrip validation

#### TTL & Caching
- **TTL Namespace Support**: create-cache command with expiration logic
- **Cache Management**: set-cache/drop-cache admin commands  
- **Metadata Systems**: sys_namespaces for TTL tracking

#### Stream Processing
- **Token Parser**: Stream command processing
- **Auth Integration**: meta:sec:pass/user preamble handling  
- **Transaction Boundaries**: Multi-operation stream support
- **Security Enforcement**: Stream-level security validation

#### Concurrency & Performance
- **Busy Timeout**: Configuration for multi-writer scenarios
- **WAL Mode**: Write-Ahead Logging for better concurrency
- **Multi-Writer Safety**: Proper locking and coordination

## Milestone 1: Complete v0.1 Feature Set
**Goal**: All [v0.1] tests passing

### Tasks:
1. **Complete KV Operations**
   - keys/scan commands
   - Prefix filtering
   - JSON support (--json flag)

2. **TTL Namespace Support**
   - create-cache command
   - TTL expiration logic
   - sys_namespaces metadata

3. **Stream Processing**
   - Token parser
   - Auth preamble (meta:sec:pass/user)
   - Transaction boundaries
   - Security enforcement

4. **Discovery Commands**
   - projects/namespaces/nss listing
   - Basic admin commands

5. **Concurrency**
   - busy_timeout configuration
   - WAL mode setup
   - Multi-writer safety

### Test Coverage Target:
- All [v0.1] tests from TEST-SPEC.md

## Milestone 2: Polish & v0.2 Features
**Goal**: Import/export, enhanced streaming

### Tasks:
1. **Import/Export**
   - TSV format support
   - Roundtrip validation

2. **Enhanced Streaming**
   - ls --stream format
   - meta:ns alias support
   - TTL validation in streams

3. **Admin Enhancements**
   - set-cache/drop-cache commands
   - Namespace management

### Test Coverage Target:
- All [v0.2] tests

## Next Immediate Actions

### Current Status:
- ✅ **ROADMAP_MVP.md Updated** - Systematic construction approach defined
- ✅ **Foundation Infrastructure** - XDG, Storage, Dispatcher, Addressing basics complete
- 🔧 **Ready for Phase 2** - Begin Database CRUD (Step 1) implementation

### Next Steps:
1. **Start Database CRUD (db_crud.rs)**: 
   - `prontodb db-create`, `db-switch`, `db-rename`, `db-delete` commands
   - Sticky cursor files for database selection persistence
   - `.prontorc-name` support for per-user configurations

2. **Complete Isolated CRUD Sequence**:
   - Database → Tables → Projects → Namespaces → Keys (dependency order)
   - Each with direct dispatcher commands for testing
   - Full lifecycle validation before advancing

3. **Integration Phase**: 
   - Traditional user commands (`set/get/del`) using full object resolution
   - Address resolution through complete object hierarchy
   - Lifecycle operations orchestrating across all 6 core objects

## Success Metrics

- **Phase 1**: Foundation infrastructure with direct CRUD dispatcher routing
- **Phase 2**: All 6 core objects with complete isolated CRUD operations  
- **Phase 3**: Full integration with traditional user commands working
- **Future**: Enhanced features (TTL, streaming, multi-instance) on solid foundation