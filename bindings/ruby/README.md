# HEDL Ruby Bindings

Ruby bindings for HEDL (Hierarchical Entity Data Language) - a token-efficient data format optimized for LLM context windows.

## Installation

Add to your Gemfile:

```ruby
gem 'hedl'
```

Or install directly:

```bash
gem install hedl
```

**Prerequisites:** The HEDL shared library must be installed.

## Quick Start

```ruby
require 'hedl'

# Parse HEDL content
doc = Hedl.parse(<<~HEDL)
  %VERSION: 1.0
  %STRUCT: User: [id, name, email]
  ---
  users: @User
    | alice, Alice Smith, alice@example.com
    | bob, Bob Jones, bob@example.com
HEDL

# Get document info
puts doc.version.inspect  # [1, 0]
puts doc.schema_count     # 1

# Convert to JSON
puts doc.to_json

# Convert to other formats
yaml = doc.to_yaml
xml = doc.to_xml
csv = doc.to_csv
cypher = doc.to_cypher

# Clean up
doc.close
```

## API Reference

### Module Methods

| Method | Description |
|--------|-------------|
| `Hedl.parse(content, strict: true)` | Parse HEDL string |
| `Hedl.validate(content, strict: true)` | Validate without document |
| `Hedl.from_json(content)` | Parse JSON to HEDL |
| `Hedl.from_yaml(content)` | Parse YAML to HEDL |
| `Hedl.from_xml(content)` | Parse XML to HEDL |
| `Hedl.from_parquet(data)` | Parse Parquet to HEDL |

### Document Methods

| Method | Description |
|--------|-------------|
| `version` | Get [major, minor] array |
| `schema_count` | Get schema count |
| `alias_count` | Get alias count |
| `root_item_count` | Get root item count |
| `canonicalize` | Convert to canonical HEDL |
| `to_json(include_metadata: false)` | Convert to JSON |
| `to_yaml(include_metadata: false)` | Convert to YAML |
| `to_xml` | Convert to XML |
| `to_csv` | Convert to CSV |
| `to_parquet` | Convert to Parquet bytes |
| `to_cypher(use_merge: true)` | Convert to Neo4j Cypher |
| `lint` | Run linting |
| `close` | Free resources |
| `closed?` | Check if closed |

### Diagnostics

```ruby
diag = doc.lint
puts "Issues: #{diag.count}"

diag.each do |item|
  puts "[#{item[:severity]}] #{item[:message]}"
end

errors = diag.errors
warnings = diag.warnings
hints = diag.hints

diag.close
```

### Error Handling

```ruby
begin
  doc = Hedl.parse('invalid content')
rescue Hedl::Error => e
  puts "Error: #{e.message}"
  puts "Code: #{e.code}"
end
```

## Environment Variables

| Variable | Description | Default | Recommended |
|----------|-------------|---------|-------------|
| `HEDL_LIB_PATH` | Path to the HEDL shared library | Auto-detected | - |
| `HEDL_MAX_OUTPUT_SIZE` | Maximum output size in bytes for conversions | 100 MB | 500 MB - 1 GB |

### Resource Limits

The `HEDL_MAX_OUTPUT_SIZE` environment variable controls the maximum size of output from conversion operations (`to_json`, `to_yaml`, `to_xml`, etc.). The default of 100 MB is conservative and may be too restrictive for many real-world data processing scenarios.

**Setting the limit:**

```bash
# In your shell (before running Ruby)
export HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB

# Or in Ruby (must be set BEFORE require 'hedl')
ENV['HEDL_MAX_OUTPUT_SIZE'] = '1073741824'  # 1 GB
require 'hedl'
```

**Recommended values:**

- **Small configs (10-50 MB)**: Default 100 MB is usually sufficient
- **Medium datasets (100-500 MB)**: Set to `524288000` (500 MB)
- **Large datasets (500 MB - 5 GB)**: Set to `1073741824` or higher (1 GB+)
- **Very large datasets**: Set to `5368709120` (5 GB) or `10737418240` (10 GB)
- **No practical limit**: Set to a very high value appropriate for your system

**Error handling:**

When the output size exceeds the limit, a `Hedl::Error` will be raised:

```ruby
begin
  large_output = doc.to_json
rescue Hedl::Error => e
  if e.code == Hedl::ErrorCode::ALLOC
    puts "Output too large: #{e.message}"
    puts "Increase HEDL_MAX_OUTPUT_SIZE environment variable"
  end
end
```

## Development

```bash
# Build the library
cd /path/to/hedl
cargo build --release -p hedl-ffi

# Run tests
bundle exec rspec
```

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
