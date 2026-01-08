# hedl-lsp

Language Server Protocol implementation for HEDL.

## Installation

```bash
cargo install hedl-lsp
```

## Features

- **Syntax highlighting** - Semantic tokens for editors
- **Diagnostics** - Real-time error detection
- **Auto-completion** - Context-aware suggestions
- **Hover information** - Type and documentation on hover
- **Go to definition** - Navigate to referenced items
- **Find references** - Find all usages of an item
- **Document symbols** - Outline view of document structure

## Editor Integration

### VS Code

Install the HEDL extension from the marketplace, which bundles hedl-lsp.

### Neovim (with nvim-lspconfig)

```lua
require('lspconfig').hedl.setup{
  cmd = { 'hedl-lsp' },
  filetypes = { 'hedl' },
}
```

### Other Editors

Configure your editor to run `hedl-lsp` as the language server for `.hedl` files.

## Running Manually

```bash
hedl-lsp --stdio
```

## License

Apache-2.0
