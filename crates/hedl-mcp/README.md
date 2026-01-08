# hedl-mcp

Model Context Protocol server for HEDL, enabling AI/LLM integration.

## Installation

```bash
cargo install hedl-mcp
```

## Usage

```bash
hedl-mcp --stdio
```

## MCP Tools

The server provides these tools to LLMs:

### hedl_parse
Parse and validate a HEDL document.

### hedl_to_json
Convert HEDL to JSON format.

### hedl_from_json
Convert JSON to HEDL format.

### hedl_validate
Validate HEDL syntax and semantics.

### hedl_format
Canonicalize a HEDL document.

### hedl_lint
Run linting rules on a document.

### hedl_query
Query a HEDL document with path expressions.

## Claude Desktop Integration

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "hedl": {
      "command": "hedl-mcp",
      "args": ["--stdio"]
    }
  }
}
```

## Features

- **Caching** - Parsed documents are cached for performance
- **Rate limiting** - Configurable request limits
- **Streaming** - Large document support

## License

Apache-2.0
