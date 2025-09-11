# Semantic Layer Architecture: MarkdownDB Example

## Overview

This document demonstrates how to build sophisticated semantic layers on top of ProntoDB's simple 3-layer KV foundation. MarkdownDB serves as an example of how to create specialized tools that leverage ProntoDB's addressing system while maintaining clean architectural separation.

## Architecture Principles

### 1. **Foundation Layer: ProntoDB**
- **Role**: Dead-simple KV store with 3-layer addressing (`project.namespace.key`)
- **Enhanced Cursors**: Store database path + optional meta-context prefix
- **Multi-Agent**: Isolation via `--user` parameter
- **Stays Simple**: No business logic, just reliable storage

### 2. **Semantic Layer: MarkdownDB**
- **Role**: Sophisticated document management with biblical addressing
- **Storage Backend**: Uses ProntoDB for all persistence
- **Address Translation**: Maps document addresses to ProntoDB keys
- **Complex Features**: Context chains, cross-references, metadata

## MarkdownDB Implementation Example

### Biblical Addressing System

```rust
// MarkdownDB addresses: BashFX 1:10 (document:chapter:verse)
// Maps to ProntoDB: bashfx.v1.verse_10

struct DocumentAddress {
    document: String,    // "bashfx"  
    chapter: u32,        // 1
    verse: u32,          // 10
}

impl DocumentAddress {
    fn to_prontodb_key(&self) -> String {
        format!("{}.v{}.verse_{}", self.document, self.chapter, self.verse)
    }
    
    fn from_biblical(addr: &str) -> Option<Self> {
        // Parse "BashFX 1:10" -> DocumentAddress
        let parts: Vec<&str> = addr.split_whitespace().collect();
        if parts.len() != 2 { return None; }
        
        let document = parts[0].to_lowercase();
        let chapter_verse: Vec<&str> = parts[1].split(':').collect();
        if chapter_verse.len() != 2 { return None; }
        
        Some(DocumentAddress {
            document,
            chapter: chapter_verse[0].parse().ok()?,
            verse: chapter_verse[1].parse().ok()?,
        })
    }
}
```

### MarkdownDB Command Layer

```rust
// markdowndb ingest document.md --name="BashFX" --chapter=1
fn do_ingest(args: Args) -> i32 {
    let file_path = args.get_or(1, "");
    let doc_name = args.get_flag_or("--name", "");
    let chapter = args.get_flag_or("--chapter", "1").parse::<u32>().unwrap_or(1);
    
    if file_path.is_empty() || doc_name.is_empty() {
        error!("Usage: markdowndb ingest <file> --name=<doc> --chapter=<n>");
        return 1;
    }
    
    _ingest_markdown_document(file_path, doc_name, chapter)
}

fn _ingest_markdown_document(file_path: &str, doc_name: &str, chapter: u32) -> i32 {
    let content = match read_file(file_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to read {}: {}", file_path, e);
            return 1;
        }
    };
    
    let verses = __parse_markdown_verses(&content);
    
    for (verse_num, verse_content) in verses.iter().enumerate() {
        let addr = DocumentAddress {
            document: doc_name.to_lowercase(),
            chapter,
            verse: (verse_num + 1) as u32,
        };
        
        let prontodb_key = addr.to_prontodb_key();
        
        // Store in ProntoDB using enhanced cursor with meta-context
        let result = shell!(
            "prontodb set {} '{}' --user=markdowndb", 
            prontodb_key, 
            verse_content.replace("'", "\\'")
        );
        
        if result.status != 0 {
            error!("Failed to store verse {}", verse_num + 1);
            return 1;
        }
    }
    
    // Store document metadata
    let meta_key = format!("{}.meta.info", doc_name.to_lowercase());
    let metadata = json!({
        "document": doc_name,
        "chapter": chapter,
        "verse_count": verses.len(),
        "ingested_at": date!(epoch),
        "source_file": file_path
    }).to_string();
    
    shell!("prontodb set {} '{}' --user=markdowndb", meta_key, metadata);
    
    okay!("Ingested {} verses from {} as {}", verses.len(), file_path, doc_name);
    0
}

fn __parse_markdown_verses(content: &str) -> Vec<String> {
    // Split markdown into logical verses (paragraphs, code blocks, etc.)
    let mut verses = Vec::new();
    let mut current_verse = String::new();
    
    for line in content.lines() {
        if line.trim().is_empty() {
            if !current_verse.trim().is_empty() {
                verses.push(current_verse.trim().to_string());
                current_verse.clear();
            }
        } else {
            current_verse.push_str(line);
            current_verse.push('\n');
        }
    }
    
    if !current_verse.trim().is_empty() {
        verses.push(current_verse.trim().to_string());
    }
    
    verses
}
```

### MarkdownDB Query Layer

```rust
// markdowndb get "BashFX 1:10"  
fn do_get(args: Args) -> i32 {
    let biblical_addr = args.get_or(1, "");
    if biblical_addr.is_empty() {
        error!("Usage: markdowndb get <Document Chapter:Verse>");
        return 1;
    }
    
    match DocumentAddress::from_biblical(biblical_addr) {
        Some(addr) => {
            let prontodb_key = addr.to_prontodb_key();
            let result = shell!("prontodb get {} --user=markdowndb", prontodb_key);
            
            if result.status == 0 && !result.output.trim().is_empty() {
                println!("{} {}:{}", addr.document.to_uppercase(), addr.chapter, addr.verse);
                println!("---");
                println!("{}", result.output.trim());
                0
            } else {
                error!("Verse not found: {}", biblical_addr);
                1
            }
        },
        None => {
            error!("Invalid address format. Use: Document Chapter:Verse");
            1
        }
    }
}

// markdowndb search "error handling" --doc=bashfx
fn do_search(args: Args) -> i32 {
    let query = args.get_or(1, "");
    let doc_filter = args.get_flag("--doc");
    
    if query.is_empty() {
        error!("Usage: markdowndb search <query> [--doc=<name>]");
        return 1;
    }
    
    // Get all keys matching pattern
    let pattern = if let Some(doc) = doc_filter {
        format!("{}.v*", doc.to_lowercase())
    } else {
        "*.v*".to_string()
    };
    
    let result = shell!("prontodb keys '{}' --user=markdowndb", pattern);
    if result.status != 0 {
        error!("Failed to query ProntoDB");
        return 1;
    }
    
    let mut matches = Vec::new();
    
    for key in result.output.lines() {
        let key = key.trim();
        if key.is_empty() { continue; }
        
        // Get content from ProntoDB
        let content_result = shell!("prontodb get {} --user=markdowndb", key);
        if content_result.status == 0 {
            let content = content_result.output.trim();
            if content.to_lowercase().contains(&query.to_lowercase()) {
                matches.push((key.to_string(), content.to_string()));
            }
        }
    }
    
    if matches.is_empty() {
        info!("No matches found for: {}", query);
        return 0;
    }
    
    info!("Found {} matches for '{}':", matches.len(), query);
    for (key, content) in matches {
        // Convert key back to biblical address  
        if let Some(addr) = __key_to_biblical(key) {
            println!("\n{}", addr);
            println!("---");
            println!("{}", content);
        }
    }
    
    0
}

fn __key_to_biblical(key: String) -> Option<String> {
    // Convert "bashfx.v1.verse_10" back to "BashFX 1:10"
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 3 { return None; }
    
    let doc = parts[0].to_uppercase();
    let chapter = parts[1].strip_prefix("v")?.parse::<u32>().ok()?;
    let verse = parts[2].strip_prefix("verse_")?.parse::<u32>().ok()?;
    
    Some(format!("{} {}:{}", doc, chapter, verse))
}
```

## Enhanced Cursor Integration

### Multi-Project Context Management

```rust
// Enhanced cursor stores database path + meta-context
// markdowndb cursor set work /path/to/work.db --meta=company_engineering
fn do_cursor_set(args: Args) -> i32 {
    let cursor_name = args.get_or(1, "");
    let db_path = args.get_or(2, "");
    let meta_context = args.get_flag("--meta");
    
    if cursor_name.is_empty() || db_path.is_empty() {
        error!("Usage: markdowndb cursor set <name> <db_path> [--meta=<context>]");
        return 1;
    }
    
    // Store enhanced cursor info in ProntoDB
    let cursor_key = format!("_cursors.{}.config", cursor_name);
    let cursor_config = json!({
        "db_path": db_path,
        "meta_context": meta_context,
        "created_at": date!(epoch)
    }).to_string();
    
    // Use default ProntoDB (not --user=markdowndb for cursor storage)
    shell!("prontodb set {} '{}'", cursor_key, cursor_config);
    
    okay!("Cursor '{}' configured with database: {}", cursor_name, db_path);
    if let Some(meta) = meta_context {
        info!("Meta-context: {}", meta);
    }
    
    0
}

// markdowndb --cursor=work get "BashFX 1:10"
// Automatically uses enhanced cursor with meta-context
fn resolve_cursor_context(cursor_name: &str) -> Option<(String, Option<String>)> {
    let cursor_key = format!("_cursors.{}.config", cursor_name);
    let result = shell!("prontodb get {}", cursor_key);
    
    if result.status != 0 { return None; }
    
    // Parse cursor configuration
    let config: serde_json::Value = serde_json::from_str(&result.output).ok()?;
    let db_path = config.get("db_path")?.as_str()?.to_string();
    let meta_context = config.get("meta_context").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    Some((db_path, meta_context))
}
```

## Cross-Agent Coordination Example

### Agent Namespace Isolation

```rust
// Different agents use same MarkdownDB but with isolated namespaces
// Agent "alice": markdowndb --user=alice ingest specs.md --name="ProjectSpecs"  
// Agent "bob":   markdowndb --user=bob ingest notes.md --name="MeetingNotes"

// Storage in ProntoDB:
// alice: projectspecs.v1.verse_1 = "Requirements overview..."  
// bob:   meetingnotes.v1.verse_1 = "Discussed architecture..."

fn do_list_documents(args: Args) -> i32 {
    let user_context = get_var("USER").unwrap_or("default".to_string());
    
    info!("Documents for user context: {}", user_context);
    
    // Query ProntoDB for this user's documents
    let result = shell!("prontodb keys '*.meta.info' --user={}", user_context);
    if result.status != 0 {
        error!("Failed to query documents");
        return 1;
    }
    
    for key in result.output.lines() {
        let key = key.trim();
        if key.is_empty() { continue; }
        
        let doc_name = key.split('.').next().unwrap_or("unknown");
        
        // Get metadata
        let meta_result = shell!("prontodb get {} --user={}", key, user_context);
        if meta_result.status == 0 {
            let meta: serde_json::Value = serde_json::from_str(&meta_result.output).unwrap_or_default();
            let verse_count = meta.get("verse_count").and_then(|v| v.as_u64()).unwrap_or(0);
            let ingested_at = meta.get("ingested_at").and_then(|v| v.as_str()).unwrap_or("unknown");
            
            println!("{}: {} verses (ingested: {})", doc_name.to_uppercase(), verse_count, ingested_at);
        }
    }
    
    0
}
```

## Benefits of This Architecture

### 1. **Clean Separation**
- ProntoDB stays simple and reliable
- MarkdownDB handles all complexity 
- Clear interface boundary via shell commands

### 2. **Enhanced Cursor Power** 
- 4-layer storage capability via meta-contexts
- 3-layer user interface maintains simplicity
- Transparent context switching

### 3. **Multi-Agent Ready**
- Namespace isolation via `--user` parameter
- Cross-agent workflows through shared keys
- No coordination complexity in ProntoDB layer

### 4. **Composability**
- Other semantic layers can be built (TaskDB, RuleDB, EventDB)
- All use same ProntoDB foundation
- Consistent addressing and multi-agent patterns

## Implementation Plan

1. **Phase 1**: Implement enhanced cursors in ProntoDB core
2. **Phase 2**: Build MarkdownDB as separate crate depending on ProntoDB
3. **Phase 3**: Create additional semantic layer examples (TaskDB, ConfigDB)
4. **Phase 4**: Document cross-layer integration patterns

This architecture enables unlimited sophistication while keeping the foundation rock-solid and simple.