# MCP Server Usage Tutorial

This tutorial demonstrates how to use the HEDL Model Context Protocol (MCP) server for AI/LLM integration.

> **Note**: The HEDL MCP server is under active development. Features described in this tutorial are implemented but may undergo API changes in future releases.

## What is MCP?

The Model Context Protocol (MCP) is a standardized interface that allows AI/LLM systems to interact with external data sources and tools. The HEDL MCP server provides:

- Read HEDL files from disk
- Query entities by type and ID
- Validate HEDL documents
- Optimize JSON to HEDL for token efficiency
- Get token statistics and comparisons

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
wget https://github.com/dweve/hedl/releases/download/v0.1.0/hedl-mcp
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
      "args": ["--root", "/path/to/hedl/files"]
    }
  }
}
```

## LLM Integration

### Claude with MCP

Configure Claude to use the HEDL MCP server:

```json
{
  "mcpServers": {
    "hedl": {
      "command": "hedl-mcp",
      "args": ["--directory", "/path/to/hedl/files"],
      "env": {
        "HEDL_LOG_LEVEL": "info"
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

### Security

```json
{
  "api_key": "your-secret-key",
  "allowed_ips": ["127.0.0.1", "192.168.1.0/24"],
  "rate_limit_per_minute": 100,
  "max_concurrent_connections": 10
}
```

### Caching

```json
{
  "enable_cache": true,
  "cache_ttl_seconds": 300,
  "cache_max_entries": 1000
}
```

### Logging

```bash
export HEDL_LOG_LEVEL=debug
export HEDL_LOG_FILE=/var/log/hedl-mcp.log

hedl-mcp --config config.json
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

## Monitoring

### Health Check

```bash
curl http://localhost:3000/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

### Metrics

```bash
curl http://localhost:3000/metrics
```

Response:
```json
{
  "requests_total": 1234,
  "requests_failed": 5,
  "cache_hits": 890,
  "cache_misses": 344,
  "average_response_time_ms": 15.6
}
```

## Next Steps

- **[MCP API Reference](../mcp-api.md)** - Complete MCP documentation
- **[Examples](../examples.md)** - More integration examples
- **[GitHub](https://github.com/dweve/hedl)** - Source code and issues

## Resources

- **[MCP Specification](https://spec.modelcontextprotocol.io/)** - MCP standard
- **[HEDL Documentation](https://hedl.dev/docs)** - Full documentation
- **[Community](https://github.com/dweve/hedl/discussions)** - Discussions and support
