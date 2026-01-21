# hedl-lsp

**Production-grade Language Server Protocol implementation for HEDL—rich IDE integration with diagnostics, completion, navigation, and performance optimization.**

Writing HEDL without IDE support means no syntax validation, no autocomplete, no reference checking, no quick navigation. Modern development demands real-time feedback, intelligent suggestions, and seamless navigation. `hedl-lsp` brings HEDL to every LSP-compatible editor: VS Code, Neovim, Emacs, Sublime Text, and more.

This is a complete LSP server with 10 implemented features, performance optimizations (200ms debouncing, O(1) reference lookups, caching), and comprehensive error handling. Write HEDL with confidence—get instant diagnostics, context-aware completions, hover documentation, and cross-document navigation.

## What's Implemented

Complete LSP server with production-grade features:

1. **Real-Time Diagnostics**: Parse errors from hedl-core + lint warnings from hedl-lint
2. **Context-Aware Completion**: 7 completion contexts (header, references, matrix, keys, values)
3. **Hover Information**: Markdown documentation for directives, references, types, aliases
4. **Go to Definition**: O(1) precise navigation to entity and type definitions
5. **Find References**: O(1) lookup of all entity and type usages
6. **Document Symbols**: Hierarchical outline with schemas, aliases, nests, entities
7. **Workspace Symbols**: Case-insensitive search across all open documents
8. **Semantic Highlighting**: Token types and modifiers for syntax awareness
9. **Document Formatting**: Canonicalization via hedl-c14n
10. **Rename Refactoring**: Safe rename for entity IDs, types, aliases, and field names with conflict detection

## Installation

```bash
# From source
cargo install hedl-lsp

# Or build locally
cd crates/hedl-lsp
cargo build --release
```

Binary location: `target/release/hedl-lsp`

## Editor Integration

### VS Code

**Recommended**: Install the official HEDL extension from the VS Code marketplace, which bundles `hedl-lsp` and provides automatic configuration.

**Manual Setup**:
1. Install `hedl-lsp` binary
2. Add to `settings.json`:

```json
{
  "hedl.server.path": "/path/to/hedl-lsp",
  "hedl.server.trace": "verbose"
}
```

### Neovim (nvim-lspconfig)

```lua
require('lspconfig').hedl.setup{
  cmd = { 'hedl-lsp' },
  filetypes = { 'hedl' },
  root_dir = function(fname)
    return require('lspconfig.util').find_git_ancestor(fname) or vim.fn.getcwd()
  end,
  settings = {}
}
```

**Auto-start on .hedl files**:
```lua
vim.cmd([[autocmd BufRead,BufNewFile *.hedl set filetype=hedl]])
```

### Emacs (lsp-mode)

```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(hedl-mode . "hedl"))

(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "hedl-lsp")
                  :major-modes '(hedl-mode)
                  :server-id 'hedl-lsp))

(add-hook 'hedl-mode-hook #'lsp)
```

### Sublime Text (LSP Package)

Add to LSP settings (`Preferences: LSP Settings`):

```json
{
  "clients": {
    "hedl-lsp": {
      "enabled": true,
      "command": ["/path/to/hedl-lsp"],
      "selector": "source.hedl"
    }
  }
}
```

### Helix Editor

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "hedl"
scope = "source.hedl"
file-types = ["hedl"]
language-server = { command = "hedl-lsp" }
```

## Features in Detail

### Real-Time Diagnostics

Instant feedback as you type with parse errors and lint warnings:

**Parse Errors**:
```hedl
users: @User[id, name  # Missing closing bracket
  | alice, Alice Smith
```

**Diagnostic**:
```
Error at line 1, column 24:
  Parse error: unexpected end of line, expected ']'
```

**Lint Warnings**:
```hedl
%ALIAS: old_api: https://legacy.example.com  # Defined but unused
---
users: @User[id, name]
  | alice, Alice Smith
```

**Diagnostic**:
```
Warning at line 1:
  [unused-alias] Alias 'old_api' is defined but never used
```

**Severity Levels**:
- **Error** (red squiggles): Parse errors, schema violations
- **Warning** (yellow squiggles): Lint warnings (unused aliases, unresolved references)
- **Information** (blue squiggles): Lint suggestions (add count hints, use ditto)

### Context-Aware Completion

7 different completion contexts with intelligent filtering:

#### 1. Header Context - Directive Suggestions

**Trigger**: Start of line in header section

**Completions**:
```
%VERSION:    Document version (required)
%STRUCT:     Define entity schema
%ALIAS:      Define variable alias
%NEST:       Define parent-child relationship
```

#### 2. Reference Context - Type Names

**Trigger**: After `@` symbol

```hedl
author: @|  # Cursor here
```

**Completions** (from defined schemas):
```
@User
@Post
@Comment
@Organization
```

#### 3. Reference ID Context - Entity IDs

**Trigger**: After `@Type:`

```hedl
author: @User:|  # Cursor here
```

**Completions** (from parsed entities):
```
@User:alice
@User:bob
@User:carol
```

#### 4. List Type Context - Schema Types

**Trigger**: After `:` in list declaration

```hedl
users: |  # Cursor here
```

**Completions**:
```
@User[id, name, email]
@Post[id, title, content]
@Comment[id, text, author]
```

#### 5. Matrix Cell Context - Cell Values

**Trigger**: Inside matrix row (after `|`)

```hedl
users: @User[id, name, active]
  | alice, Alice, |  # Cursor here
```

**Completions**:
```
^        # Ditto (repeat previous value)
~        # Null
true     # Boolean
false    # Boolean
@User:   # Reference to User entities
```

#### 6. Key Context - Field Names

**Trigger**: Start of line in data section

**Completions** (common keys + inferred from types):
```
users       # List key for User type
posts       # List key for Post type
name        # Common field
description # Common field
config      # Common field
```

#### 7. Value Context - Aliases and References

**Trigger**: After `:` on data line

```hedl
api_url: |  # Cursor here
```

**Completions**:
```
$api_endpoint    # Alias expansion
$version         # Alias expansion
@User:alice      # Entity reference
@Organization:   # Organization references
```

**Performance**: Optimized context detection with cached `header_end_line` for O(1) lookups.

### Hover Information

Markdown-formatted documentation on mouse hover:

#### Hover on Directives

```hedl
%STRUCT: User: [id, name, email]
^^^^^^^
```

**Hover Output**:
```markdown
**%STRUCT Directive**

Define entity schema with field names.

**Syntax**: `%STRUCT: TypeName: [field1, field2, ...]`

**Example**:
%STRUCT: User: [id, name, email, created_at]
```

#### Hover on References

```hedl
author: @User:alice
        ^^^^^^^^^^^
```

**Hover Output**:
```markdown
**Entity Reference**

**Type**: User
**ID**: alice
**Status**: ✓ Found (line 15)

**Schema**: [id, name, email, created_at]
**Fields**: alice, Alice Smith, alice@example.com, 2024-01-15
```

If reference is unresolved:
```markdown
**Entity Reference**

**Type**: User
**ID**: unknown_user
**Status**: ⚠ Not found

The referenced entity does not exist in this document.
```

#### Hover on Aliases

```hedl
endpoint: $api_url
          ^^^^^^^^
```

**Hover Output**:
```markdown
**Alias Expansion**

**Name**: api_url
**Value**: https://api.example.com
**Defined at**: line 3
```

#### Hover on Types

```hedl
users: @User[id, name]
       ^^^^^
```

**Hover Output**:
```markdown
**Type: User**

**Schema**: [id, name, email, created_at]
**Entities**: 125

**Nested Children**: Post (via %NEST)
```

#### Hover on Special Tokens

```hedl
| alice, Alice, ^
              ^
```

**Hover Output**:
```markdown
**Ditto Operator (`^`)**

Repeats the value from the same column in the previous row.

**Example**:
| order1, @User:alice, pending
| order2, ^, shipped          # Repeats @User:alice
```

### Go to Definition

**Trigger**: Ctrl+Click (or Cmd+Click) on references

Jump to exact definition location with precise character ranges:

```hedl
# Line 15: Entity definition
users: @User[id, name]
  | alice, Alice Smith
  | bob, Bob Jones

# Line 42: Reference usage
author: @User:alice
        ^^^^^^^^^^^  # Ctrl+Click here
```

**Result**: Jumps to line 16, column 4 (the `alice` definition in matrix row)

**Features**:
- O(1) lookup via ReferenceIndex v2 (HashMap-based)
- Handles both qualified (`@Type:id`) and unqualified (`@id`) references
- Works across all entity types

### Find References

**Trigger**: Right-click → Find All References

Find all locations where an entity or type is referenced:

```hedl
# Definition at line 16
users: @User[id, name]
  | alice, Alice Smith
    ^^^^^  # Find References here

# Shows all usages:
```

**Result**:
```
Found 5 references to 'alice':
  - Line 16: Entity definition (users matrix)
  - Line 42: author: @User:alice
  - Line 58: reviewer: @User:alice
  - Line 73: owner: @User:alice
  - Line 91: created_by: @User:alice
```

**Options**:
- `include_declaration`: Include the definition location in results (default: true)

**Performance**: O(1) lookup via ReferenceIndex v2

### Document Symbols

**Trigger**: Ctrl+Shift+O (or Cmd+Shift+O) - "Go to Symbol in File"

Hierarchical outline of document structure:

```
📄 Document
├─ 📦 Header
│  ├─ 🏗 Schemas (3)
│  │  ├─ User [id, name, email, created_at]
│  │  ├─ Post [id, title, content, author]
│  │  └─ Comment [id, text, post, author]
│  ├─ 🔗 Aliases (2)
│  │  ├─ $api_url
│  │  └─ $version
│  └─ 🌳 Nests (1)
│     └─ Post > Comment
└─ 📊 Data
   ├─ 👥 users: @User (125 entities)
   │  ├─ alice
   │  ├─ bob
   │  └─ carol
   ├─ 📝 posts: @Post (48 entities)
   └─ 💬 comments: @Comment (312 entities)
```

**Symbol Types**:
- **Module**: Header container
- **Struct**: Type schemas
- **Variable**: Aliases (with `$` prefix)
- **Class**: Entity types
- **Function**: Nest relationships
- **Object**: Individual entities

### Workspace Symbols

**Trigger**: Ctrl+T (or Cmd+T) - "Go to Symbol in Workspace"

**Query**: Type to filter symbols across all open documents

```
Query: "user"

Results:
  User (schema) in config.hedl:5
  users (list) in data.hedl:12
  user_roles (alias) in settings.hedl:8
  user_count (field) in stats.hedl:23
```

**Features**:
- Case-insensitive search
- Matches schemas, entities, aliases across all documents
- Returns symbol with container name and location

### Semantic Highlighting

**Token Types**:
- **KEYWORD**: `%VERSION`, `%STRUCT`, `%ALIAS`, `%NEST`
- **TYPE**: Entity type names (`@User`, `@Post`)
- **VARIABLE**: Aliases (`$api_url`, `$version`)
- **STRING**: Quoted strings
- **NUMBER**: Integers, floats
- **COMMENT**: Full-line and inline comments
- **OPERATOR**: `@`, `$`, `|`, `^`, `~`

**Modifiers**:
- **DEFINITION**: Entity definitions, type declarations
- **DECLARATION**: Schema declarations, alias declarations

### Document Formatting

**Trigger**: Shift+Alt+F (or Shift+Option+F) - "Format Document"

Formats HEDL to canonical form using hedl-c14n:

**Before**:
```hedl
users:@User[id,name]
|alice,Alice Smith
   |bob,Bob Jones
```

**After**:
```hedl
users: @User[id, name]
  | alice, Alice Smith
  | bob, Bob Jones
```

**Features**:
- Normalizes whitespace
- Consistent indentation (2 spaces)
- Alphabetically sorts header directives
- Graceful handling of parse errors (returns original on failure)

### Rename Refactoring

**Trigger**: F2 (or editor's rename command) on a symbol

Safe, validated rename refactoring for HEDL symbols with conflict detection and cross-document support:

**Supported Symbol Types**:
- **Entity IDs**: Rename individual entities (e.g., `alice` → `alice_smith`)
- **Type Names**: Rename schema types (e.g., `User` → `Account`)
- **Alias Names**: Rename variable aliases (e.g., `api_url` → `api_endpoint`)
- **Field Names**: Rename schema fields (e.g., `email` → `email_address`)

**Features**:
- **Prepare Rename**: Shows what will be renamed before committing (with validation)
- **Conflict Detection**: Prevents duplicate names in scope
- **Cross-Document Support**: Rename across all open documents (workspace-wide)
- **Validation**: Syntax and semantic correctness checks
- **Case Similarity Warnings**: Detects names differing only in case

**Example**:
```hedl
# Rename entity 'alice' to 'alice_smith'
users: @User[id, name]
  | alice, Alice Smith           # Definition
  | bob, Bob Jones

# References to alice (all updated):
owner: @User:alice               # Line 1
author: @User:alice              # Line 2
created_by: @User:alice          # Line 3
```

**Result**: All 3 occurrences renamed to `alice_smith` in a single atomic operation.

## Performance Optimizations

### 1. Debouncing (200ms)

Batches keystrokes together to prevent excessive parsing:

```
User types: "users: @User[id, name]"

Without debouncing:
  - Parse after "u" (1st keystroke)
  - Parse after "us" (2nd keystroke)
  - Parse after "use" (3rd keystroke)
  ... 25 parses total (one per character)

With 200ms debouncing:
  - Wait 200ms after last keystroke
  - Parse once: "users: @User[id, name]"
  ... 1 parse total (~90% reduction)
```

**Impact**: Prevents stuttering during rapid typing, reduces CPU usage by ~90% during editing.

### 2. Dirty Tracking

Content hash-based change detection prevents redundant parsing:

```rust
// Only parse if content actually changed
if content_hash != previous_hash {
    parse_and_analyze(content);
}
```

**Impact**: Eliminates parsing when moving cursor without changes, saves ~30% of parse operations.

### 3. Caching

Parsed `AnalyzedDocument` cached in DocumentManager:

```rust
// Arc-wrapped for efficient concurrent access
document_cache: DashMap<Url, Arc<AnalyzedDocument>>
```

**Impact**: O(1) document retrieval for hover, completion, definition lookups.

### 4. Reference Index v2 (O(1) Lookups)

**Old Implementation**: O(n) linear search through all entities
**New Implementation**: HashMap-based O(1) lookups

```rust
pub struct ReferenceIndex {
    // O(1) definition lookup
    definitions: HashMap<(String, String), RefLocation>,
    // O(1) references lookup
    references: HashMap<String, Vec<RefLocation>>,
    // Position-based reverse lookup
    location_to_ref: HashMap<u32, Vec<(String, RefLocation)>>,
}
```

**Impact**: Go to Definition and Find References are instant even with thousands of entities.

### 5. Header Optimization

Caches header end line to eliminate O(n) scans on every completion:

```rust
// Single-pass header parsing, cached result
header_end_line: Option<usize>
```

**Impact**: Context detection for completion is O(1) instead of O(n).

## Memory Management

### Document Size Limits

**Per Document**: 500 MB (configurable)
**Max Documents**: 1000 with LRU eviction

```rust
pub struct DocumentManager {
    documents: DashMap<Url, Arc<Mutex<DocumentState>>>,
    cache_stats: Arc<Mutex<CacheStatistics>>,
    max_cache_size: Arc<parking_lot::RwLock<usize>>,      // 1000
    max_document_size: Arc<parking_lot::RwLock<usize>>,   // 500 MB
}
```

**Enforcement**: Checked on `did_open` and `did_change` events

**Error Message** (when exceeded):
```
Document size (520 MB) exceeds limit (500 MB)
```

### LRU Eviction

When cache is full (1000 documents), least-recently-used documents are evicted:

```rust
pub struct CacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub current_size: usize,
    pub max_size: usize,
}
```

**Cache Stats** (available via `statistics()` method):
```
CacheStatistics {
  hits: 15234,
  misses: 1523,
  evictions: 23,
  current_size: 847,
  max_size: 1000
}
```

### UTF-8 Safety

All string slicing is UTF-8 boundary-aware:

```rust
// Safe slicing functions
pub fn safe_slice_to(s: &str, to: usize) -> &str
pub fn safe_slice_from(s: &str, from: usize) -> &str
```

**Prevents**: Panics on multi-byte character boundaries (emoji, non-ASCII)

## Text Synchronization

**Sync Kind**: Full document sync

**Events Handled**:
1. `textDocument/didOpen` - Initial document analysis
2. `textDocument/didChange` - Re-analysis after edits
3. `textDocument/didSave` - Includes full text for immediate re-analysis
4. `textDocument/didClose` - Cache cleanup

**Save Behavior**: Includes text content in save notification for immediate re-analysis (ensures diagnostics update instantly).

## Capabilities Advertised

On LSP `initialize`:

```json
{
  "capabilities": {
    "textDocumentSync": {
      "openClose": true,
      "change": 1,
      "save": { "includeText": true }
    },
    "completionProvider": {
      "resolveProvider": false,
      "triggerCharacters": ["@", ":", "%", "$", "|"]
    },
    "hoverProvider": true,
    "definitionProvider": true,
    "referencesProvider": true,
    "documentSymbolProvider": true,
    "workspaceSymbolProvider": true,
    "documentFormattingProvider": true,
    "renameProvider": {
      "prepareProvider": true
    },
    "semanticTokensProvider": {
      "legend": {
        "tokenTypes": ["keyword", "type", "variable", "string", "number", "comment", "operator"],
        "tokenModifiers": ["definition", "declaration"]
      },
      "range": false,
      "full": true
    }
  }
}
```

## What This Crate Doesn't Implement

**Code Lens**: Not implemented—no actionable commands displayed inline.

**Code Actions**: No quick-fixes or refactoring actions (e.g., "Add schema declaration").

**Folding Ranges**: No code folding support for matrix lists or nested structures.

**Call Hierarchy**: Not applicable to HEDL's declarative data model.

**Linked Editing Range**: Simultaneous editing of related symbols not supported.

**Inlay Hints**: No inline type hints or parameter names displayed.

## Architecture

### Key Data Structures

**AnalyzedDocument**:
```rust
pub struct AnalyzedDocument {
    document: Option<Document>,              // Parsed AST
    errors: Vec<HedlError>,                  // Parse errors
    lint_diagnostics: Vec<Diagnostic>,       // Lint warnings
    entities: HashMap<String, HashMap<String, usize>>,  // type -> id -> line
    schemas: HashMap<String, (Vec<String>, usize)>,     // type -> (cols, line)
    aliases: HashMap<String, (String, usize)>,          // name -> (value, line)
    reference_index_v2: ReferenceIndex,      // O(1) reference lookup
    nests: HashMap<String, (String, usize)>, // parent -> (child, line)
    header_end_line: Option<usize>,          // Cached header boundary
}
```

**ReferenceIndex** (v2):
```rust
pub struct ReferenceIndex {
    // Definition locations: (type, id) -> location
    definitions: HashMap<(String, String), RefLocation>,
    // Reference locations: reference_string -> vec of locations
    references: HashMap<String, Vec<RefLocation>>,
    // Reverse mapping: location -> reference string
    location_to_ref: HashMap<u32, Vec<(String, RefLocation)>>,
}
```

### Testing Coverage

Comprehensive test suite in `tests.rs`:
- **Analysis**: Schema/alias/nest/entity extraction
- **Completion**: All 7 contexts (header, reference, matrix, key/value)
- **Hover**: Directives, references, types, aliases, special tokens
- **Symbols**: Document symbols, workspace symbols
- **Cache**: LRU eviction, statistics, hit tracking

## Use Cases

**HEDL Schema Development**: Write HEDL schemas with instant validation feedback, autocomplete for types and references, go-to-definition for entity lookups.

**Configuration File Editing**: Edit HEDL configuration files with real-time error checking, hover documentation for directives, format on save for consistency.

**Large Document Navigation**: Navigate large HEDL files with document symbols outline, workspace-wide entity search, find all references to entities.

**Multi-File Projects**: Work across multiple HEDL files with workspace symbols search, cross-document reference validation, consistent formatting.

**Learning HEDL**: Explore HEDL syntax with hover documentation on directives, context-aware completions showing available options, inline diagnostics explaining errors.

## Performance Characteristics

**Parse Performance**: ~100-200 MB/s for typical HEDL files

**Debouncing**: ~90% reduction in parse operations during typing

**Reference Lookups**: O(1) with ReferenceIndex v2 (previously O(n))

**Completion**: O(1) context detection with cached header end line

**Memory**: O(document_size + cache_size), with LRU eviction at 1000 documents

**Responsiveness**: 200ms debounce ensures smooth typing experience without stuttering

Detailed performance benchmarks are available in the HEDL repository benchmark suite.

## Dependencies

- `tower-lsp` 0.20 - LSP protocol implementation
- `tokio` 1.0 - Async runtime (features: full)
- `dashmap` 5.5 - Concurrent HashMap
- `parking_lot` 0.12 - High-performance mutexes
- `ropey` 1.6 - Rope for efficient document editing
- `tracing` 0.1 - Structured logging
- `tracing-subscriber` 0.3 - Logging configuration (features: env-filter)
- `hedl-core` (workspace) - HEDL parser
- `hedl` (workspace) - HEDL core library
- `hedl-lint` (workspace) - HEDL linter
- `hedl-c14n` (workspace) - HEDL canonicalization
- `serde` (workspace) - Serialization framework
- `serde_json` (workspace) - JSON support
- `thiserror` (workspace) - Error handling

## License

Apache-2.0
