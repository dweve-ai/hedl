# MCP Server Usage Tutorial

This tutorial demonstrates how to use the HEDL Model Context Protocol (MCP) server for AI/LLM integration.

> **Note**: The HEDL MCP server is under active development. Features described in this tutorial are implemented but may undergo API changes in future releases.

## What is MCP?

The Model Context Protocol (MCP) is a standardized interface that allows AI/LLM systems to interact with external data sources and tools. The HEDL MCP server provides **10 tools**:

- `hedl_read`: Read and parse HEDL files from disk
- `hedl_query`: Query entities by type and/or ID
- `hedl_validate`: Validate HEDL with detailed diagnostics
- `hedl_optimize`: Convert JSON to optimized HEDL format
- `hedl_stats`: Get token usage statistics (HEDL vs JSON)
- `hedl_format`: Format HEDL to canonical form
- `hedl_write`: Write HEDL content to a file
- `hedl_convert_to`: Convert HEDL to json, yaml, csv, parquet, cypher
- `hedl_convert_from`: Convert json, yaml, csv, parquet to HEDL
- `hedl_stream`: Stream parse large HEDL documents with pagination

## Prerequisites

- Rust 1.70+ (for building from source)
- Understanding of JSON-RPC
- AI/LLM system supporting MCP (Claude Code, etc.)

## Installation

### From Source

```bash
git clone https://github.com/dweve/hedl.git
cd hedl
cargo build --release -p hedl-mcp

# Binary will be in target/release/
./target/release/hedl-mcp --help
```

### Pre-built Binary

Download from GitHub releases:

```bash
wget https://github.com/dweve/hedl/releases/download/v1.2.0/hedl-mcp
chmod +x hedl-mcp
```

## Quick Start

### Start the Server

The HEDL MCP server communicates via STDIO (standard input/output), following the MCP specification. It does not use HTTP ports by default.

```bash
# Run with default root directory (current directory)
hedl-mcp

# Run with a specific root directory for file operations
hedl-mcp --root /path/to/hedl/files
```

## Available Tools

The HEDL MCP server provides several tools for HEDL operations. All tools are called via the `tools/call` method.

### 1. hedl_read

Read and parse a HEDL file.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "hedl_read",
    "arguments": {
      "path": "users.hedl"
    }
  },
  "id": 1
}
```

### 2. hedl_query

Query entities by type and ID.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "hedl_query",
    "arguments": {
      "hedl": "%VERSION: 1.0\n...",
      "type_name": "User",
      "id": "alice"
    }
  },
  "id": 2
}
```

## Client Integration

### JavaScript/TypeScript Client (Node.js)

```typescript
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

async function main() {
    const transport = new StdioClientTransport({
        command: "hedl-mcp",
        args: ["--root", "./data"]
    });

    const client = new Client({
        name: "hedl-client",
        version: "1.0.0"
    }, {
        capabilities: {}
    });

    await client.connect(transport);

    // Call a tool
    const result = await client.callTool({
        name: "hedl_validate",
        arguments: {
            hedl: "%VERSION: 1.0\n---\nkey: value"
        }
    });

    console.log(result);
}

main();
```

## LLM Integration

### Claude with MCP

Configure Claude Desktop to use the HEDL MCP server by editing `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "hedl": {
      "command": "hedl-mcp",
      "args": ["--root", "/path/to/hedl/files"],
      "env": {
        "RUST_LOG": "hedl_mcp=info"
      }
    }
  }
}
```

Example prompt:

```
Using the HEDL MCP server, read the users.hedl file and show me all users with the "admin" role.
```

Claude will:
1. Call `hedl_read` to load users.hedl
2. Parse the response
3. Filter users by role
4. Present the results

### Custom LLM Integration

```python
from openai import OpenAI

client = OpenAI()
hedl_client = HedlMcpClient()

def chat_with_hedl_context(query, hedl_path):
    # Read HEDL file
    doc = hedl_client.read_file(hedl_path)

    # Convert to JSON for context
    context = json.dumps(doc, indent=2)

    # Send to LLM
    response = client.chat.completions.create(
        model="gpt-4",
        messages=[
            {"role": "system", "content": "You have access to HEDL data."},
            {"role": "user", "content": f"Context:\n{context}\n\nQuery: {query}"}
        ]
    )

    return response.choices[0].message.content

# Usage
answer = chat_with_hedl_context(
    "Which users have admin access?",
    "users.hedl"
)
print(answer)
```

## Advanced Configuration

The MCP server supports TOML-based configuration for advanced settings.

### Configuration File

```toml
# hedl-mcp.toml
enabled = true

[request]
max_total_size_bytes = 10485760      # 10 MB
max_param_size_bytes = 5242880       # 5 MB
max_array_elements = 10000
max_object_depth = 32

[response]
max_total_size_bytes = 50000000      # 50 MB
max_result_items = 100000
enable_streaming = true

[rate_limiting]
mode = "per_client"
default_burst = 200
default_per_second = 100
cleanup_interval_seconds = 300        # 5 minutes

[[rate_limiting.overrides]]
client_pattern = "premium-*"
burst = 1000
per_second = 500

[concurrency]
max_concurrent_requests = 100
max_concurrent_per_client = 10
max_concurrent_per_tool = 50
queue_timeout_ms = 5000              # 5 seconds

[timeouts]
default_timeout_ms = 30000           # 30 seconds

[timeouts.per_tool]
hedl_validate = 5000                 # 5 seconds
hedl_query = 10000                   # 10 seconds
hedl_convert_to = 60000              # 60 seconds
hedl_stream = 120000                 # 120 seconds
```

### Logging

```bash
# Set log level via environment variable
export RUST_LOG=hedl_mcp=debug

# Run the server
hedl-mcp --root /path/to/hedl/files
```

## Complete Example: Data Pipeline

```python
import json
from hedl_mcp_client import HedlMcpClient

class DataPipeline:
    def __init__(self):
        self.client = HedlMcpClient()

    def extract_users(self, path):
        """Extract users from HEDL file"""
        doc = self.client.read_file(path)
        return doc["content"]["root"]

    def transform_to_json(self, users):
        """Transform HEDL users to JSON"""
        return json.dumps(users, indent=2)

    def optimize_for_llm(self, json_data):
        """Optimize JSON for LLM context"""
        result = self.client.optimize(json_data)
        print(f"Token savings: {result['stats']['savings_percent']}%")
        return result["hedl"]

    def validate_output(self, hedl_content):
        """Validate the optimized HEDL"""
        result = self.client.validate(hedl_content)
        if not result["valid"]:
            raise Exception(f"Invalid HEDL: {result['diagnostics']}")

    def run(self, input_path):
        """Run the pipeline"""
        # Extract
        users = self.extract_users(input_path)
        print(f"Extracted {len(users)} users")

        # Transform
        json_data = self.transform_to_json(users)

        # Optimize
        optimized = self.optimize_for_llm(json_data)

        # Validate
        self.validate_output(optimized)

        return optimized

# Usage
pipeline = DataPipeline()
result = pipeline.run("users.hedl")
print("Pipeline result:", result)
```

## Protocol

The MCP server uses STDIO transport (JSON-RPC 2.0 over standard input/output). It does not expose HTTP endpoints.

### Supported Methods

| Method | Description |
|--------|-------------|
| `initialize` | Protocol handshake with capability negotiation |
| `initialized` | Notification after handshake completion |
| `shutdown` | Graceful server shutdown |
| `tools/list` | List available HEDL tools |
| `tools/call` | Execute a specific tool |
| `resources/list` | List available HEDL files |
| `resources/read` | Read HEDL file content |
| `ping` | Health check endpoint |

### Error Codes

| Code | Error | Description |
|------|-------|-------------|
| `-32700` | Json | Invalid JSON received |
| `-32600` | InvalidRequest | Invalid JSON-RPC request |
| `-32601` | ToolNotFound | Tool not found |
| `-32602` | InvalidArguments | Invalid tool arguments |
| `-32603` | ResourceNotFound | Resource not found |
| `-32001` | Parse | HEDL parsing error |
| `-32002` | Io | IO error |
| `-32003` | PathTraversal | Path traversal attempt blocked |
| `-32004` | FileNotFound | Requested file not found |
| `-32005` | ResourceLimit | Resource limit exceeded |

## Next Steps

- **[MCP API Reference](../mcp-api.md)** - Complete MCP documentation
- **[Examples](../examples.md)** - More integration examples
- **[GitHub](https://github.com/dweve/hedl)** - Source code and issues

## Resources

- **[MCP Specification](https://spec.modelcontextprotocol.io/)** - MCP standard
- **[HEDL Documentation](https://hedl.dev/docs)** - Full documentation
- **[Community](https://github.com/dweve/hedl/discussions)** - Discussions and support
