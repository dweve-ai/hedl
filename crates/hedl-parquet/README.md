# hedl-parquet

**Bidirectional HEDL ↔ Apache Parquet conversion—columnar storage for analytics workloads with compression and efficient querying.**

Analytics systems need columnar storage. Parquet is the standard for big data workflows—Spark, Presto, Athena, BigQuery all consume it. But converting HEDL documents to Parquet shouldn't lose type information or structural semantics. Reading Parquet files back to HEDL for transformation and validation shouldn't require custom schemas.

`hedl-parquet` provides bidirectional conversion between HEDL and Apache Parquet via Arrow 57.0 integration. Export HEDL entity lists as columnar Parquet tables with automatic schema generation and type mapping. Choose compression algorithms (SNAPPY/GZIP/ZSTD/UNCOMPRESSED) for storage optimization. Import Parquet files back to HEDL with full schema preservation and metadata roundtrip. Security-hardened with decompression bomb protection and column count limits.

## What's Implemented

Comprehensive Parquet integration with Apache Arrow:

1. **Arrow 57.0 Integration**: Full Arrow RecordBatch support for columnar encoding
2. **Type Mapping**: All HEDL types → Arrow types (Int64, Float64, Utf8, Boolean, Null)
3. **Four Compression Modes**: SNAPPY (default), GZIP, ZSTD, UNCOMPRESSED
4. **Automatic Schema Generation**: HEDL %STRUCT → Arrow Schema with metadata
5. **Bidirectional Conversion**: HEDL → Parquet → HEDL with roundtrip fidelity
6. **Position Preservation**: Sequential processing guarantees insertion order maintained
7. **Security Hardening**: Decompression bomb detection (100 MB limit), max 1000 columns
8. **Metadata Preservation**: HEDL version, struct definitions, aliases stored in Parquet metadata
9. **Reference Encoding**: References serialized as strings (`@Type:id` format preserved)
10. **Batch Processing**: Write multiple entity lists as separate Parquet files

## Installation

```toml
[dependencies]
hedl-parquet = "1.0"
```

## Basic Usage

### HEDL → Parquet

Export HEDL entity list to Parquet file:

```rust
use hedl_core::parse;
use hedl_parquet::to_parquet;
use std::fs::File;

let doc = parse(br#"
%VERSION: 1.0
%STRUCT: User: [id, name, age, email, active]
---
users: @User
  | alice, Alice Smith, 30, alice@example.com, true
  | bob, Bob Jones, 25, bob@example.com, true
  | carol, Carol White, 35, carol@example.com, false
"#)?;

let file = File::create("users.parquet")?;
to_parquet(&doc, file)?;
```

**Generated Parquet**:
- **Schema**: `id: Utf8, name: Utf8, age: Int64, email: Utf8, active: Boolean`
- **Rows**: 3 records in columnar format
- **Compression**: SNAPPY (default)
- **Metadata**: Includes HEDL version and schema definitions

### Custom Configuration

```rust
use hedl_parquet::{to_parquet_with_config, ParquetConfig, Compression};

let config = ParquetConfig::builder()
    .compression(Compression::ZSTD)          // ZSTD compression
    .row_group_size(10000)                   // 10K rows per group
    .include_metadata(true)                  // Include HEDL metadata
    .max_columns(500)                        // Limit column count
    .build();

let file = File::create("users.parquet")?;
to_parquet_with_config(&doc, file, &config)?;
```

### Parquet → HEDL

Import Parquet file back to HEDL:

```rust
use hedl_parquet::from_parquet;
use std::fs::File;

let file = File::open("users.parquet")?;
let doc = from_parquet(file)?;

// Use HEDL's structured API
println!("Version: {}.{}", doc.version.0, doc.version.1);
for (type_name, entities) in &doc.entities {
    println!("{}: {} entities", type_name, entities.len());
}
```

## Compression Algorithms

### SNAPPY (Default)

Fast compression with moderate compression ratios:

```rust
.compression(Compression::SNAPPY)
```

**Characteristics**:
- Speed: Very fast (200-500 MB/s compression)
- Ratio: ~2-3x compression
- Use Case: Default choice for balanced performance

**When to Use**: General-purpose analytics, interactive queries, when read speed matters more than storage

### GZIP

High compression ratios at cost of speed:

```rust
.compression(Compression::GZIP)
```

**Characteristics**:
- Speed: Slower (20-50 MB/s compression)
- Ratio: ~4-10x compression
- Use Case: Archival storage, infrequently accessed data

**When to Use**: Long-term storage, cost-sensitive scenarios, infrequent reads

### ZSTD

Modern compression with excellent speed/ratio balance:

```rust
.compression(Compression::ZSTD)
```

**Characteristics**:
- Speed: Fast (100-300 MB/s compression)
- Ratio: ~3-6x compression (configurable levels)
- Use Case: Modern analytics pipelines

**When to Use**: Best overall choice for new systems, Spark/Presto workloads

### UNCOMPRESSED

No compression (raw columnar storage):

```rust
.compression(Compression::UNCOMPRESSED)
```

**Characteristics**:
- Speed: Fastest I/O
- Ratio: 1x (no compression)
- Use Case: In-memory analytics, temporary data

**When to Use**: Data already compressed at filesystem level, in-memory processing

## Type Mapping

HEDL types map to Arrow types with full fidelity:

```rust
// HEDL Value → Arrow Type
Value::Int(42)              → arrow::datatypes::DataType::Int64
Value::Float(3.14)          → arrow::datatypes::DataType::Float64
Value::String("alice")      → arrow::datatypes::DataType::Utf8
Value::Bool(true)           → arrow::datatypes::DataType::Boolean
Value::Null                 → null value in nullable column

// References serialized as strings
Value::Reference(qualified("User", "alice"))  → "@User:alice" (Utf8)
Value::Reference(local("item1"))             → "@item1" (Utf8)

// Expressions evaluated then converted
Value::Expression("$(1+2)") → 3 (Int64)
```

**Nullable Columns**: All columns nullable by default (Arrow schema)

**Type Inference**: When importing Parquet without HEDL metadata, types inferred from Arrow schema

## Schema Generation

Automatic Arrow schema from HEDL %STRUCT:

```hedl
%STRUCT: Product: [id, name, price, stock, discontinued]
```

**Generated Arrow Schema**:
```
Schema {
  fields: [
    Field { name: "id", data_type: Utf8, nullable: true },
    Field { name: "name", data_type: Utf8, nullable: true },
    Field { name: "price", data_type: Float64, nullable: true },
    Field { name: "stock", data_type: Int64, nullable: true },
    Field { name: "discontinued", data_type: Boolean, nullable: true },
  ],
  metadata: {
    "hedl.version": "1.0",
    "hedl.struct.Product": "[id, name, price, stock, discontinued]"
  }
}
```

**Metadata Included**:
- HEDL version (major.minor)
- Struct definitions (%STRUCT declarations)
- Aliases (%ALIAS declarations)
- Type information for roundtrip fidelity

## Configuration Reference

### ParquetConfig

```rust
use hedl_parquet::{ParquetConfig, Compression};

let config = ParquetConfig::builder()
    .compression(Compression::SNAPPY)        // Compression algorithm
    .row_group_size(1000)                    // Rows per row group (default: 1000)
    .include_metadata(true)                  // Include HEDL metadata (default: true)
    .max_columns(1000)                       // Maximum columns (default: 1000)
    .max_decompression_size(100 * 1024 * 1024)  // 100 MB bomb protection
    .preserve_order(true)                    // Maintain insertion order (default: true)
    .build();
```

### Configuration Options

**compression** (default: SNAPPY)
- Compression algorithm for storage
- Options: SNAPPY, GZIP, ZSTD, UNCOMPRESSED
- Trade-off: speed vs size

**row_group_size** (default: 1000)
- Rows per Parquet row group
- Affects query performance and memory usage
- Recommendation: 1000-10000 for most cases

**include_metadata** (default: true)
- Include HEDL metadata in Parquet schema
- Required for roundtrip fidelity
- Disable if targeting systems that don't handle custom metadata

**max_columns** (default: 1000)
- Maximum allowed columns per schema
- Security limit to prevent resource exhaustion
- Raise for wide schemas (>1000 columns)

**max_decompression_size** (default: 100 MB)
- Decompression bomb protection
- Prevents memory exhaustion from malicious compressed data
- Raise for legitimate large files

**preserve_order** (default: true)
- Maintain entity insertion order during conversion
- Disable for performance if order doesn't matter

## Security Features

### Decompression Bomb Protection

Prevents memory exhaustion from malicious compressed Parquet:

```rust
const MAX_DECOMPRESSION_SIZE: usize = 100 * 1024 * 1024;  // 100 MB

// Reading compressed file that decompresses to > 100 MB:
// Error: ParquetError::DecompressionBombDetected { size: 500MB, max: 100MB }
```

**Protection Against**:
- Zip bomb attacks (small compressed, huge decompressed)
- Memory exhaustion DoS
- Accidental processing of extremely large files

### Column Count Limit

Prevents resource exhaustion from wide schemas:

```rust
const MAX_COLUMNS: usize = 1000;

// Processing schema with > 1000 columns:
// Error: ParquetError::TooManyColumns { count: 1500, max: 1000 }
```

**Protection Against**:
- Memory allocation attacks
- Schema processing overhead
- Accidentally processing transposed data

### Integer Overflow Protection

All integer operations checked for overflow:

```rust
// Safe arithmetic on column counts, row counts, buffer sizes
// Panics prevented, returns error on overflow
```

## Position Preservation

Sequential processing guarantees insertion order:

```hedl
users: @User
  | alice, Alice       # Row 0
  | bob, Bob           # Row 1
  | carol, Carol       # Row 2
```

**Parquet Order**: Row 0, 1, 2 (same as HEDL)

**When Importing**: Rows reconstructed in same order

**Configuration**:
```rust
.preserve_order(true)  // Maintain order (default)
.preserve_order(false) // Allow reordering for optimization
```

## Batch Processing

Write multiple entity lists to separate Parquet files:

```rust
use hedl_parquet::{write_entity_list, ParquetConfig};

let config = ParquetConfig::default();

// Write each entity list to separate file
for (type_name, entities) in &doc.entities {
    let filename = format!("{}.parquet", type_name.to_lowercase());
    let file = File::create(&filename)?;
    write_entity_list(type_name, entities, file, &config)?;
}
```

**Benefits**:
- Independent file per entity type
- Parallel processing possible
- Selective loading (load only needed types)

## Error Handling

```rust
use hedl_parquet::{to_parquet, ParquetError};

match to_parquet(&doc, file) {
    Ok(()) => println!("Export successful"),
    Err(ParquetError::TooManyColumns { count, max }) => {
        eprintln!("Schema too wide: {} columns (max: {})", count, max);
    }
    Err(ParquetError::DecompressionBombDetected { size, max }) => {
        eprintln!("File too large after decompression: {} bytes (max: {})",
            size, max);
    }
    Err(ParquetError::UnsupportedType { type_name, field }) => {
        eprintln!("Unsupported type '{}' in field '{}'", type_name, field);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Error Types

- `TooManyColumns` - Schema exceeds column limit
- `DecompressionBombDetected` - Decompressed size exceeds limit
- `UnsupportedType` - Value type cannot be represented in Parquet
- `SchemaConflict` - Inconsistent types within column
- `Arrow(arrow::error::ArrowError)` - Arrow library errors
- `Io(std::io::Error)` - I/O failures

## Use Cases

**Data Warehousing**: Export HEDL entity lists to Parquet for loading into Snowflake, BigQuery, Redshift. Leverage columnar compression and query performance.

**Spark Pipelines**: Convert HEDL to Parquet for Apache Spark processing. Read from Spark, transform, write back to HEDL for downstream systems.

**Athena Queries**: Store HEDL exports as Parquet in S3, query with AWS Athena. Partition by date/type for cost-effective analytics.

**ML Feature Stores**: Export training data from HEDL to Parquet for ML pipelines. Read from Parquet, transform features, train models.

**Long-Term Archival**: Store HEDL data as compressed Parquet for archival. GZIP/ZSTD compression reduces storage costs significantly.

**Data Lake Integration**: Append HEDL data to Parquet-based data lakes. Maintain compatibility with ecosystem tools (Hive, Presto, Drill).

## What This Crate Doesn't Do

**Multi-File Datasets**: Writes single Parquet file per entity list. For partitioned datasets, use external partitioning logic.

**Nested Structures**: Flattens nested HEDL entities to flat Parquet tables. For complex nesting, use JSON export or custom schema mapping.

**Streaming Writes**: Buffers entire entity list before writing. For streaming, write in batches to multiple files.

**Predicate Pushdown**: No query optimization—reads entire file. For selective queries, use query engines (Spark, Presto) on generated Parquet.

**Schema Evolution**: No automatic schema migration. For evolving schemas, handle versioning externally.

## Performance Characteristics

**Write Performance**: 50-200 MB/s depending on compression (SNAPPY fastest, GZIP slowest)

**Read Performance**: 100-500 MB/s columnar reads (benefits from predicate/projection pushdown in query engines)

**Compression Ratios**:
- SNAPPY: 2-3x
- GZIP: 4-10x
- ZSTD: 3-6x
- UNCOMPRESSED: 1x

**Memory Usage**: O(row_group_size * column_count) during write, O(row_group_size) during read

## Dependencies

- `hedl-core` 1.0 - Core HEDL implementation
- `arrow` 57.0 - Apache Arrow columnar format
- `parquet` 57.0 - Apache Parquet file format
- `thiserror` 1.0 - Error type definitions

## License

Apache-2.0
