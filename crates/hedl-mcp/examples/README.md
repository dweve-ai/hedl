# hedl-mcp Examples

## Schema Macro Usage

The `schema_macro_usage.rs` example demonstrates the complete schema macro system for defining JSON schemas with minimal boilerplate.

### Running the Example

```bash
cargo run --example schema_macro_usage -p hedl-mcp
```

### What It Demonstrates

1. **Basic Type Macros**
   - `schema_string!` - String types with optional pattern validation
   - `schema_bool!` - Boolean types with optional defaults
   - `schema_integer!` - Integer types with optional ranges and defaults
   - `schema_enum!` - Enumerated string types
   - `schema_string_array!` - Array of strings

2. **Composite Macros**
   - `schema_options!` - Nested option objects
   - `tool_schema!` - Complete tool schemas with properties and required fields

3. **Domain-Specific Macros**
   - `hedl_content_arg!` - Standard HEDL document argument
   - `path_arg!` - File/directory path argument
   - `format_arg!` - Format enumeration argument
   - `ditto_arg!` - Ditto optimization flag

4. **Multi-Field Macros**
   - `validation_args!` - Strict/lint validation flags
   - `pagination_args!` - Limit/offset pagination
   - `file_write_args!` - Validate/format/backup file operations
   - `convert_to_options!` - Conversion output options
   - `convert_from_options!` - Conversion input options

5. **Real-World Tool Schemas**
   - Conversion tools (to/from formats)
   - Validation tools (with strict mode)
   - Stream parsing tools (with pagination)
   - File operation tools (with backup)

### Example Output

The example generates formatted JSON schemas for all macro patterns, showing:
- Complete schema structure
- Property definitions
- Required fields
- Default values
- Type constraints
- Pattern validation

### Code Size Comparison

**Before Macros**: ~1,500 characters per tool schema (manual definition)
**After Macros**: ~600 characters per tool schema (60% reduction)

See [SCHEMA_MACROS.md](../SCHEMA_MACROS.md) for complete documentation.
