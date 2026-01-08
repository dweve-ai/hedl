# LSP API Reference

**Language Server Protocol implementation for IDE integration**

---

## Overview

The HEDL LSP server provides rich IDE integration for HEDL files, enabling features like autocomplete, hover information, go-to-definition, and real-time diagnostics in any LSP-compatible editor.

**Supported Editors**:
- VS Code
- Neovim
- Emacs (via lsp-mode or eglot)
- Vim (via vim-lsp or coc.nvim)
- Sublime Text
- Atom
- IntelliJ IDEA (via LSP plugin)

---

## Quick Start

### Installation

```bash
# Install from source
cargo install hedl-lsp

# Or build locally
cargo build --release --bin hedl-lsp
```

### Running the Server

```bash
# Run with stdio transport
hedl-lsp

# With debug logging
RUST_LOG=debug hedl-lsp

# With trace logging
RUST_LOG=trace hedl-lsp
```

---

## Features

### 1. Diagnostics

Real-time error and warning reporting.

**Capabilities**:
- Syntax error detection
- Reference validation
- Schema validation
- Linting rules (best practices)

**Example Diagnostics**:
```
Line 10: Undefined type 'Usr' (did you mean 'User'?)
Line 15: Unused alias 'old_value'
Line 20: Missing required field 'email' in User entity
```

**Severity Levels**:
- Error (red squiggle)
- Warning (yellow squiggle)
- Hint (gray squiggle)

---

### 2. Autocomplete

Context-aware code completion.

**Completion Types**:

**a) Entity Types**
```hedl
users: @U|  # Suggests: User, Post, Comment
```

**b) Entity IDs**
```hedl
author: @User:|  # Suggests: alice, bob, charlie
```

**c) Directives**
```hedl
%|  # Suggests: VERSION, STRUCT
```

**d) Field Names**
```hedl
%STRUCT: User: [id, |  # Suggests common field names
```

**e) Values**
```hedl
active: |  # Suggests: true, false, null
```

---

### 3. Hover Information

Display information when hovering over symbols.

**Entity Type Hover**:
```hedl
users: @User
       ^^^^
```
Shows:
```
Type: User
Schema: [id, name, email, role]
Instances: 3
```

**Entity ID Hover**:
```hedl
author: @User:alice
              ^^^^^
```
Shows:
```
Entity: alice (User)
Fields:
  name: Alice Smith
  email: alice@example.com
  role: admin
```

**Reference Hover**:
```hedl
@User:alice
```
Shows:
```
Reference to User:alice
Status: ✓ Resolved
Type: User
```

---

### 4. Go to Definition

Navigate to entity and type definitions.

**From Reference**:
```hedl
author: @User:alice
              ^^^^^ → Jump to line 15 where alice is defined
```

**From Type**:
```hedl
users: @User
        ^^^^ → Jump to %STRUCT: User definition
```

---

### 5. Find References

Find all usages of entities and types.

**Find Entity References**:
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | alice, Alice

posts: @Post
  | post1, @User:alice
```

**Results**:
```
Found 2 references to User:
  Line 3: users: @User
  Line 8: | post1, @User:alice
```

---

### 6. Document Symbols

Hierarchical outline view.

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title]
---
users: @User
  | alice, Alice
  | bob, Bob
posts: @Post
  | post1, Hello
```

**Symbol Tree**:
```
📄 document.hedl
  📋 Schemas
    🔷 User [id, name]
    🔷 Post [id, title]
  📦 Entities
    👤 User (2 instances)
      • alice
      • bob
    📝 Post (1 instance)
      • post1
```

---

### 7. Workspace Symbols

Search symbols across all open documents.

**Query**: "User"

**Results**:
```
📄 users.hedl
  🔷 User (schema)
  👤 alice (User)
  👤 bob (User)

📄 posts.hedl
  🔗 @User:alice (reference)
```

---

### 8. Semantic Highlighting

Type-aware syntax highlighting.

**Token Types**:
- Keyword: `%VERSION`, `%STRUCT`
- Type: `User`, `Post`
- Variable: Entity IDs
- String: String literals
- Number: Numeric literals
- Operator: `@`, `^`, `$`
- Comment: `# comment`

---

### 9. Document Formatting

Format HEDL to canonical form.

**Before**:
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
z: 3
a: 1
```

**After** (Format Document):
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
a: 1
z: 3
```

---

## Configuration

### Default Configuration

```json
{
    "maxDocuments": 1000,
    "maxDocumentSize": 524288000,  // 500 MB
    "debounceMs": 200,
    "enableDiagnostics": true,
    "enableCompletion": true,
    "enableHover": true,
    "enableFormatting": true
}
```

### Custom Configuration

Override defaults programmatically:

```rust
use hedl_lsp::HedlLanguageServer;

let server = HedlLanguageServer::with_config(
    client,
    2000,                      // max documents
    1024 * 1024 * 1024         // 1 GB max size
);
```

---

## Performance

### Optimization Features

**1. Debouncing (200ms)**
- Batches keystrokes together
- Reduces parse operations by ~90%
- Prevents UI lag during typing

**2. Dirty Tracking**
- Content hash-based change detection
- Prevents redundant parsing
- Only re-parses when content changes

**3. Caching**
- Parsed documents cached in memory
- Reused for LSP queries
- LRU eviction when cache is full

**4. Reference Index**
- O(1) hash map lookups
- Fast definition/reference finding
- Replaces O(n) linear search

---

### Memory Management

**Document Size Limit**: 500 MB per document (configurable)
- Prevents memory exhaustion
- Configurable via `--max-document-size`

**Open Document Limit**: 1000 documents (configurable)
- LRU eviction when limit reached
- Configurable via `--max-documents`

**UTF-8 Safety**:
- All string slicing is UTF-8 boundary aware
- Prevents panics on invalid indices

---

## Editor Integration

### VS Code

Create `.vscode/settings.json`:

```json
{
    "hedl.lsp.enabled": true,
    "hedl.lsp.maxDocumentSize": 524288000,
    "hedl.lsp.trace.server": "verbose"
}
```

Install extension or configure manually in `settings.json`:

```json
{
    "languageServerExample.trace.server": "verbose",
    "hedl": {
        "command": "hedl-lsp",
        "args": [],
        "filetypes": ["hedl"],
        "rootPatterns": [".git", ".hedl"]
    }
}
```

---

### Neovim

Using `nvim-lspconfig`:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define HEDL LSP
if not configs.hedl_lsp then
    configs.hedl_lsp = {
        default_config = {
            cmd = {'hedl-lsp'},
            filetypes = {'hedl'},
            root_dir = lspconfig.util.root_pattern('.git', '.hedl'),
            settings = {},
        },
    }
end

-- Setup LSP
lspconfig.hedl_lsp.setup{
    on_attach = function(client, bufnr)
        -- Enable completion
        vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

        -- Keybindings
        local opts = { noremap=true, silent=true, buffer=bufnr }
        vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
        vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
        vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
        vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, opts)
    end,
}
```

---

### Emacs

Using `lsp-mode`:

```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(hedl-mode . "hedl"))

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection "hedl-lsp")
  :major-modes '(hedl-mode)
  :server-id 'hedl-lsp))

(add-hook 'hedl-mode-hook #'lsp)
```

Or using `eglot`:

```elisp
(require 'eglot)

(add-to-list 'eglot-server-programs
             '(hedl-mode . ("hedl-lsp")))

(add-hook 'hedl-mode-hook #'eglot-ensure)
```

---

### Vim

Using `vim-lsp`:

```vim
if executable('hedl-lsp')
    augroup LspHedl
        autocmd!
        autocmd User lsp_setup call lsp#register_server({
            \ 'name': 'hedl-lsp',
            \ 'cmd': {server_info->['hedl-lsp']},
            \ 'whitelist': ['hedl'],
            \ })
    augroup END
endif
```

---

### Sublime Text

Create `HEDL.sublime-settings`:

```json
{
    "clients": {
        "hedl-lsp": {
            "enabled": true,
            "command": ["hedl-lsp"],
            "selector": "source.hedl"
        }
    }
}
```

---

## Programmatic Usage

### Basic Setup

```rust
use hedl_lsp::HedlLanguageServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        HedlLanguageServer::new(client)
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
```

---

### Custom Configuration

```rust
use hedl_lsp::HedlLanguageServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        HedlLanguageServer::with_config(
            client,
            2000,                    // max documents
            1024 * 1024 * 1024       // 1 GB max document size
        )
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
```

---

## LSP Protocol Support

### Implemented Methods

**Lifecycle**:
- `initialize`
- `initialized`
- `shutdown`
- `exit`

**Document Synchronization**:
- `textDocument/didOpen`
- `textDocument/didChange` (incremental)
- `textDocument/didClose`
- `textDocument/didSave`

**Language Features**:
- `textDocument/completion`
- `textDocument/hover`
- `textDocument/definition`
- `textDocument/references`
- `textDocument/documentSymbol`
- `textDocument/formatting`
- `textDocument/semanticTokens/full`
- `workspace/symbol`

**Diagnostics**:
- `textDocument/publishDiagnostics`

---

### Server Capabilities

```json
{
    "textDocumentSync": {
        "openClose": true,
        "change": 2,  // Incremental
        "save": true
    },
    "completionProvider": {
        "triggerCharacters": ["@", ":", "%", "|"]
    },
    "hoverProvider": true,
    "definitionProvider": true,
    "referencesProvider": true,
    "documentSymbolProvider": true,
    "workspaceSymbolProvider": true,
    "documentFormattingProvider": true,
    "semanticTokensProvider": {
        "legend": {
            "tokenTypes": ["keyword", "type", "variable", "string", "number", "operator", "comment"],
            "tokenModifiers": []
        },
        "full": true
    }
}
```

---

## Diagnostics Reference

### Error Codes

Parse errors use the `HedlErrorKind` variant names as strings:

| Code | Message | Severity |
|------|---------|----------|
| `Syntax` | Syntax error in HEDL document | Error |
| `Reference` | Unresolved or invalid reference | Error |
| `Schema` | Struct definition or usage error | Error |
| `Security` | Document exceeds resource limits | Error |

Lint diagnostics use rule IDs:

| Code | Message | Severity |
|------|---------|----------|
| `unused-alias` | Alias defined but never used | Warning |
| `unused-schema` | Schema defined but never used | Warning |
| `duplicate-id` | Duplicate ID within same type | Warning |
| `missing-type` | Entity missing type annotation | Hint |
| `deeply-nested` | Nesting depth exceeds threshold | Hint |

---

## Troubleshooting

### Server Not Starting

Check logs:
```bash
RUST_LOG=debug hedl-lsp 2> lsp.log
```

### Slow Performance

Increase cache size:
```rust
HedlLanguageServer::with_config(client, 5000, 1024 * 1024 * 1024)
```

### High Memory Usage

Reduce max documents:
```rust
HedlLanguageServer::with_config(client, 500, 524288000)
```

---

## Best Practices

### 1. Keep Documents Under 10 MB

Large documents may impact performance. Consider splitting into multiple files.

### 2. Use .hedl Extension

Editors auto-detect file type based on extension.

### 3. Enable Auto-Save

Get real-time diagnostics:
```json
{
    "files.autoSave": "afterDelay",
    "files.autoSaveDelay": 500
}
```

### 4. Configure Ignore Patterns

Exclude build artifacts:
```json
{
    "files.watcherExclude": {
        "**/target/**": true,
        "**/node_modules/**": true
    }
}
```

---

**Next**: [Cross-Language Examples](examples.md)
