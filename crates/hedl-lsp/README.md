# hedl-lsp

A language server for HEDL that brings IDE features to any editor supporting LSP.

## Why This Exists

HEDL files can grow complex. Schemas reference each other. Entities link across documents. Aliases expand in unexpected places. Without tooling, you end up constantly jumping between sections, double-checking entity names, and wondering if that reference you typed actually exists.

This language server watches your HEDL files as you edit them. It parses continuously, tracks every schema, entity, and alias, and feeds that understanding back to your editor. The result: errors appear as you type, completions suggest valid references, and navigation works across your entire workspace.

## Getting Started

Install the binary:

```bash
cargo install hedl-lsp
```

Or build from source:

```bash
cd crates/hedl-lsp
cargo build --release
# Binary at target/release/hedl-lsp
```

### VS Code

The official HEDL extension bundles this server automatically. Install it from the marketplace and you're done.

For manual setup, add to `settings.json`:

```json
{
  "hedl.server.path": "/path/to/hedl-lsp",
  "hedl.server.trace": "verbose"
}
```

### Neovim

```lua
require('lspconfig').hedl.setup{
  cmd = { 'hedl-lsp' },
  filetypes = { 'hedl' },
  root_dir = function(fname)
    return require('lspconfig.util').find_git_ancestor(fname) or vim.fn.getcwd()
  end,
  settings = {}
}

vim.cmd([[autocmd BufRead,BufNewFile *.hedl set filetype=hedl]])
```

### Emacs

```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(hedl-mode . "hedl"))

(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "hedl-lsp")
                  :major-modes '(hedl-mode)
                  :server-id 'hedl-lsp))

(add-hook 'hedl-mode-hook #'lsp)
```

### Sublime Text

Add to LSP settings:

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

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "hedl"
scope = "source.hedl"
file-types = ["hedl"]
language-server = { command = "hedl-lsp" }
```

## How It Helps

### Diagnostics

Parse errors show up immediately with line and column information. Lint warnings highlight unused aliases, unresolved references, and style issues. You see problems while typing, not after running a separate validation step.

```hedl
users: @User[id, name  # Error: unexpected end of line, expected ']'
```

### Completion

Type `@` and see all defined types. Type `@User:` and see all User entities. Start a line in the header and get directive suggestions. The server understands context and offers relevant options.

```hedl
author: @User:|  # Shows: @User:alice, @User:bob, @User:carol
```

### Hover

Hover over any reference to see its definition, location, and field values. Hover over aliases to see their expanded values. Hover over directives to see documentation and usage examples.

### Navigation

Ctrl+Click (or Cmd+Click) on a reference jumps to its definition. Find All References shows every place an entity is used. Document symbols give you an outline of schemas, aliases, and entities. Workspace symbols let you search across all open files.

### Rename

Press F2 on any symbol to rename it safely. The server finds all usages, checks for conflicts, and updates everything atomically. Works for entity IDs, type names, aliases, and field names.

### Formatting

Format your document to canonical HEDL style: consistent spacing, normalized indentation, sorted header directives.

## Under the Hood

The server debounces keystrokes (200ms) to avoid reparsing on every character. A content hash skips parsing when nothing actually changed. Parsed documents live in a cache with LRU eviction at 1000 documents. Reference lookups use hash maps for O(1) performance even with thousands of entities.

Documents up to 500 MB are supported. All string operations respect UTF-8 boundaries to handle emoji and international text safely.

## Dependencies

The server builds on `tower-lsp` for protocol handling and `tokio` for async runtime. It uses `hedl-core` for parsing, `hedl-lint` for diagnostics, and `hedl-c14n` for formatting.

## License

Apache-2.0
