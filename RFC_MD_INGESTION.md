# RFC: Markdown Document Ingestion System for ProntoDB

**RFC ID:** MD-INGEST-001  
**Status:** Draft  
**Author:** ProntoDB Development Team  
**Date:** 2025-09-10  

## Abstract

This RFC proposes a markdown document ingestion system for ProntoDB that virtualizes large documents by splitting them into section-based key-value storage, enabling biblical-style addressing (e.g., `BashFX 1:10`), partial retrieval, collaborative editing, and seamless document reassembly.

---

## 1. Problem Statement

### Current Challenge
- Large technical documents are monolithic and difficult to manage
- Agent collaboration on documents requires full document loading
- No standard way to reference specific sections programmatically
- Version management of large documents is cumbersome

### Existing Solution Analysis
The sandbox demonstrates a sophisticated file-based approach using:
- H1-based document partitioning
- Hierarchical directory structure (`doc/version/N-name/content.md`)
- Metadata-driven reassembly
- Biblical-style addressing (`doc@version#section`)

### ProntoDB Opportunity
Transform the file-based approach into a KV-based system that provides:
- **Instant section access** without file I/O overhead
- **Multi-agent concurrent editing** with section-level isolation
- **Version control** at the section granularity
- **Unified addressing** through ProntoDB's existing namespace system

---

## 2. Requirements

### Functional Requirements

**FR1: Document Ingestion**
- `prontodb ingest <markdown-file> <project> <namespace>` command
- Automatic H1-based section detection and splitting
- Support for generic markdown (not just BASHFX-specific patterns)
- Preserve document hierarchy and section ordering

**FR2: Biblical-Style Addressing**
- Support `project.namespace.section` addressing
- Enable numeric addressing: `bashfx.v3.1`, `bashfx.v3.2`
- Enable semantic addressing: `bashfx.v3.philosophy`, `bashfx.v3.structure`
- Support range queries: `bashfx.v3.1-3` (sections 1 through 3)

**FR3: Section Retrieval**
- `prontodb get <project> <namespace> <section>` - retrieve specific section
- `prontodb scan <project> <namespace>` - list all sections
- Support both numeric and semantic section references
- Maintain section metadata (order, title, description)

**FR4: Document Reassembly**
- `prontodb assemble <project> <namespace>` - reconstruct full document
- `prontodb assemble <project> <namespace> <range>` - partial assembly
- Preserve original formatting and section spacing
- Maintain heading hierarchy and numbering

**FR5: Version Management**
- Multiple document versions per project
- Default namespace resolution (latest version)
- Version comparison and diff capabilities

### Non-Functional Requirements

**NFR1: Performance**
- Section retrieval: <50ms per section
- Full document assembly: <500ms for documents up to 100 sections
- Ingestion: <2s for documents up to 1MB

**NFR2: Storage Efficiency**
- Minimal metadata overhead per section
- Efficient encoding of section ordering
- Support for compressed large sections

**NFR3: Compatibility**
- Work with existing ProntoDB addressing patterns
- Maintain backward compatibility with current KV operations
- Support standard markdown formatting

---

## 3. Design Overview

### Architecture Mapping

```
Current Sandbox System    →    ProntoDB Structure
doc/bashfx               →    PROJECT: bashfx
version/v3.0             →    NAMESPACE: v3.0  
N-name/1-philosophy      →    KEY: 1 (with metadata)
partN.md content         →    VALUE: {"content": "...", "meta": {...}}
```

### Addressing Scheme

```
Biblical Reference       ProntoDB Command
BashFX 1:10             prontodb get bashfx v3.0 1:10
BashFX 2:3:4            prontodb get bashfx v3.0 2:3:4
BashFX 1                prontodb get bashfx v3.0 1
BashFX v3.0#philosophy  prontodb get bashfx v3.0 philosophy
```

### Value Structure

```json
{
  "content": "# Part I: Philosophy\n\nThis section covers...",
  "metadata": {
    "order": 1,
    "title": "Philosophy", 
    "semantic_name": "philosophy",
    "subsection_count": 5,
    "word_count": 1250,
    "ingestion_date": "2025-09-10T12:00:00Z",
    "source_lines": "1-245"
  }
}
```

---

## 4. Detailed Design

### 4.1 Command Interface

#### Ingestion Command
```bash
prontodb ingest [OPTIONS] <file> <project> <namespace>

OPTIONS:
  --section-pattern <regex>    # H1 detection pattern (default: "^# ")
  --preserve-numbering         # Keep existing section numbers
  --auto-semantic             # Generate semantic names from headings
  --dry-run                   # Show what would be ingested
  --force                     # Overwrite existing sections

EXAMPLES:
  prontodb ingest BASHFX-v3.md bashfx v3.0
  prontodb ingest --section-pattern "^## " doc.md myproject v1.0
  prontodb ingest --dry-run large-spec.md specs latest
```

#### Retrieval Commands
```bash
# Section retrieval
prontodb get <project> <namespace> <section>
prontodb get bashfx v3.0 1                    # Numeric
prontodb get bashfx v3.0 philosophy           # Semantic
prontodb get bashfx v3.0 1.3                  # Subsection

# Section listing
prontodb sections <project> <namespace>        # List all sections
prontodb sections --format table bashfx v3.0  # Tabular format
prontodb sections --metadata bashfx v3.0      # Include metadata

# Document assembly
prontodb assemble <project> <namespace> [range]
prontodb assemble bashfx v3.0                 # Full document
prontodb assemble bashfx v3.0 1-3            # Sections 1-3
prontodb assemble bashfx v3.0 philosophy,structure  # Named sections
```

#### Management Commands
```bash
# Version management
prontodb versions <project>                   # List versions
prontodb set-default <project> <namespace>    # Set default version

# Section management
prontodb section-info <project> <namespace> <section>  # Metadata
prontodb move-section <project> <from-ns> <to-ns> <section>
prontodb delete-section <project> <namespace> <section>
```

### 4.2 Parsing Algorithm

```rust
struct DocumentParser {
    section_pattern: Regex,
    preserve_numbering: bool,
    auto_semantic: bool,
}

impl DocumentParser {
    fn parse_document(&self, content: &str) -> Vec<DocumentSection> {
        let lines: Vec<&str> = content.lines().collect();
        let mut sections = Vec::new();
        let mut current_section = None;
        let mut section_counter = 1;
        
        for (line_num, line) in lines.iter().enumerate() {
            if self.section_pattern.is_match(line) {
                // Save previous section
                if let Some(section) = current_section.take() {
                    sections.push(section);
                }
                
                // Start new section
                current_section = Some(DocumentSection {
                    order: section_counter,
                    title: self.extract_title(line),
                    semantic_name: self.generate_semantic_name(line),
                    content: String::new(),
                    start_line: line_num,
                    subsections: Vec::new(),
                });
                
                section_counter += 1;
            }
            
            // Accumulate content
            if let Some(ref mut section) = current_section {
                section.content.push_str(line);
                section.content.push('\n');
            }
        }
        
        sections
    }
}
```

### 4.3 Storage Schema

#### Section Keys
```
Format: <order>[:<subsection>]
Examples: 
  "1" - Section 1
  "1:1" - Section 1, Subsection 1  
  "1:2:3" - Section 1, Subsection 2, Sub-subsection 3
```

#### Semantic Name Mapping
```
Additional keys for semantic access:
  "philosophy" -> "1"
  "structure" -> "2"
  "implementation" -> "3"
```

#### Metadata Storage
```
Special keys:
  "_meta" - Document-level metadata
  "_order" - Section ordering information
  "_index" - Semantic name -> numeric mapping
```

### 4.4 Assembly Algorithm

```rust
pub fn assemble_document(
    project: &str, 
    namespace: &str, 
    range: Option<&str>
) -> Result<String, Error> {
    // 1. Get section ordering from metadata
    let ordering = get_section_ordering(project, namespace)?;
    
    // 2. Filter sections based on range
    let sections_to_include = match range {
        Some(r) => filter_sections_by_range(&ordering, r)?,
        None => ordering,
    };
    
    // 3. Retrieve and assemble sections in order
    let mut assembled = String::new();
    for section_key in sections_to_include {
        let section_data = api::get_value(project, namespace, &section_key)?;
        let section: DocumentSection = serde_json::from_str(&section_data)?;
        
        assembled.push_str(&section.content);
        assembled.push_str("\n\n");  // Section spacing
    }
    
    Ok(assembled)
}
```

---

## 5. Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Add `ingest` command to dispatcher
- [ ] Implement markdown parsing with H1 detection
- [ ] Create section storage format and metadata schema
- [ ] Basic section retrieval functionality

### Phase 2: Advanced Addressing (Week 3)
- [ ] Semantic name mapping and resolution
- [ ] Subsection support (1.1, 1.2.3 style addressing)
- [ ] Range query support
- [ ] Section listing and metadata display

### Phase 3: Document Assembly (Week 4)
- [ ] Full document reconstruction
- [ ] Partial assembly with range support
- [ ] Formatting preservation and spacing
- [ ] Output format options (markdown, plain text)

### Phase 4: Management Features (Week 5)
- [ ] Version management commands
- [ ] Section move/copy/delete operations
- [ ] Document diff and comparison
- [ ] Import/export with external formats

### Phase 5: Optimization & Production (Week 6)
- [ ] Performance optimization for large documents
- [ ] Compression for large sections
- [ ] Extensive testing and validation
- [ ] Documentation and examples

---

## 6. API Examples

### Ingestion Workflow
```bash
# Ingest a large specification document
prontodb ingest BASHFX-v3.md bashfx v3.0

# Show what was created
prontodb sections bashfx v3.0
# Output:
# 1    philosophy     Philosophy and Design Principles
# 2    structure      Core Architecture Components  
# 3    implementation Implementation Guidelines
# 4    advanced       Advanced Features
# 5    examples       Usage Examples
```

### Section Access
```bash
# Get specific section by number
prontodb get bashfx v3.0 1
# Output: Full "Philosophy" section content

# Get by semantic name
prontodb get bashfx v3.0 philosophy  
# Output: Same as above

# Get subsection (biblical style!)
prontodb get bashfx v3.0 1:2
# Output: Philosophy subsection 2 content

# Get sub-subsection 
prontodb get bashfx v3.0 1:2:3
# Output: Philosophy subsection 2, part 3 content
```

### Document Assembly
```bash
# Reassemble full document
prontodb assemble bashfx v3.0 > BASHFX-v3-reassembled.md

# Partial assembly - first 3 sections  
prontodb assemble bashfx v3.0 1-3 > BASHFX-intro.md

# Assembly with biblical ranges
prontodb assemble bashfx v3.0 1:1-1:5 > BASHFX-philosophy-details.md

# Assembly by section names
prontodb assemble bashfx v3.0 philosophy,structure > BASHFX-overview.md
```

### Agent Collaboration
```bash
# Agent 1: Work on philosophy section
prontodb --user agent1 get bashfx v3.0 philosophy > section1.md
# ... edit section1.md ...
prontodb --user agent1 ingest section1.md bashfx v3.0-agent1-draft

# Agent 2: Work on structure section  
prontodb --user agent2 get bashfx v3.0 structure > section2.md
# ... edit section2.md ...
prontodb --user agent2 ingest section2.md bashfx v3.0-agent2-draft

# Orchestrator: Merge changes
prontodb assemble bashfx v3.0-agent1-draft philosophy > merged.md
prontodb assemble bashfx v3.0-agent2-draft structure >> merged.md
prontodb ingest merged.md bashfx v3.1
```

---

## 7. Benefits

### For Agent Workflows
- **Fine-grained access**: Agents can work on specific document sections
- **Parallel editing**: Multiple agents can edit different sections simultaneously
- **Version tracking**: Section-level version history and rollback
- **Reference linking**: Precise section references in agent communications

### For Document Management
- **Storage efficiency**: No duplicate content across versions
- **Fast retrieval**: Instant section access without parsing full documents
- **Flexible addressing**: Both numeric and semantic section references
- **Scalability**: Handle documents with hundreds of sections efficiently

### For Development Workflows  
- **Modular documentation**: Break large specs into manageable pieces
- **Collaborative authoring**: Multiple contributors without merge conflicts
- **Content reuse**: Reference sections across different document versions
- **Automated assembly**: CI/CD integration for document building

---

## 8. Risks and Mitigations

### Risk 1: Document Structure Variations
- **Risk**: Documents may not follow H1-based partitioning
- **Mitigation**: Configurable section detection patterns, manual section definition

### Risk 2: Large Section Storage
- **Risk**: Very large sections may impact performance
- **Mitigation**: Compression for large sections, size warnings, automatic subsection detection

### Risk 3: Complex Reassembly Logic
- **Risk**: Document assembly may not preserve original formatting
- **Mitigation**: Extensive testing with real documents, format preservation options

### Risk 4: Addressing Conflicts
- **Risk**: Numeric and semantic names may conflict
- **Mitigation**: Clear precedence rules, validation during ingestion, explicit disambiguation

---

## 9. Future Enhancements

### Advanced Features
- **Rich media support**: Images, tables, code blocks as separate entities
- **Cross-references**: Automatic link resolution between sections
- **Template system**: Reusable document structures and patterns
- **Search integration**: Full-text search within ingested documents

### Integration Possibilities
- **Git integration**: Sync with Git-based documentation workflows
- **Export formats**: PDF, HTML, LaTeX generation from assembled documents
- **API access**: RESTful API for external document management systems
- **IDE plugins**: Editor integration for section-based document editing

---

## 10. Conclusion

The markdown ingestion system transforms ProntoDB from a simple key-value store into a powerful document virtualization platform. By building on the proven patterns from the sandbox implementation, this system provides the foundation for sophisticated agent-based document workflows while maintaining the simplicity and performance characteristics that make ProntoDB valuable for infrastructure automation.

The biblical-style addressing (`BashFX 1:10`) combined with ProntoDB's existing multi-user, multi-cursor architecture creates a unique platform for collaborative technical documentation that scales from individual use to enterprise-wide knowledge management systems.

---

## References

- Sandbox Implementation Analysis: `.eggs/egg.1.sandbox-markdown-processing.txt`
- ProntoDB Architecture: `src/api.rs`, `src/addressing.rs`
- Biblical Addressing Examples: `sandbox/lore/` scripts
- BASHFX Document Structure: `sandbox/BASHFX-v3.md`