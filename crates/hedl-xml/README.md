# hedl-xml

**HEDL's XML ecosystem integration -bidirectional conversion, XSD schema validation, streaming, and async I/O.**

XML powers enterprise systems: SOAP APIs, configuration files, data interchange across legacy platforms, regulatory compliance documents. Your infrastructure depends on it. Your vendors require it. But XML's verbosity and lack of type safety create friction.

`hedl-xml` bridges HEDL's structured data model with XML's ubiquity. Convert between formats with configurable fidelity. Validate against XSD schemas with detailed error messages. Stream multi-gigabyte files without loading everything into memory. Use async I/O for concurrent processing with Tokio.

Part of the **HEDL format family** alongside `hedl-json`, `hedl-yaml`, `hedl-csv`, and `hedl-parquet` -bringing HEDL's efficiency and structure to every ecosystem you work in.

## What's Implemented

Based on 6,068 lines of Rust across 7 modules:

1. **Bidirectional Conversion**: HEDL ↔ XML with configurable formatting
2. **XSD Schema Validation**: Full XSD 1.0 validation with comprehensive error messages
3. **Schema Caching**: Thread-safe LRU cache for high-performance repeated validation
4. **Streaming Parser**: Process multi-gigabyte XML files with O(1) memory per element
5. **Async I/O**: Tokio-based async operations for concurrent processing (feature-gated)
6. **Security**: XXE prevention with entity policies, configurable recursion depth limits, and batch size controls

## Installation

```toml
[dependencies]
hedl-xml = "2.0"

# For async I/O support:
hedl-xml = { version = "2.0", features = ["async"] }
tokio = { version = "1", features = ["full"] }
```

## Bidirectional Conversion

### HEDL → XML: Export for Legacy Systems

Convert HEDL documents to XML when you need compatibility with existing enterprise systems:

```rust
use hedl_xml::{to_xml, ToXmlConfig};

let doc = hedl_core::parse(br#"
%S:User:[id, name, email]
---
users: @User
 | alice, Alice Smith, alice@example.com
 | bob, Bob Jones, bob@example.com
"#)?;

// Configure XML output
let config = ToXmlConfig {
    pretty: true,                       // Pretty-print with indentation
    indent: "  ".to_string(),           // 2-space indentation
    root_element: "hedl".to_string(),   // Root element name
    include_metadata: true,             // Add HEDL version metadata
    use_attributes: false,              // Use elements vs attributes
};

let xml = to_xml(&doc, &config)?;
```

Generated XML (3-5x larger than HEDL):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<hedl version="2.0">
  <users>
    <user>
      <id>alice</id>
      <name>Alice Smith</name>
      <email>alice@example.com</email>
    </user>
    <user>
      <id>bob</id>
      <name>Bob Jones</name>
      <email>bob@example.com</email>
    </user>
  </users>
</hedl>
```

**Size Overhead**: XML is typically 3-5x larger than HEDL due to verbose tag syntax. Use XML only at system boundaries where compatibility is required.

### XML → HEDL: Import from Enterprise Systems

Parse XML from SOAP APIs, configuration files, or data exports:

```rust
use hedl_xml::{from_xml, FromXmlConfig};

let xml = r#"<?xml version="1.0"?>
<system>
  <database>
    <host>localhost</host>
    <port>5432</port>
    <credentials>
      <username>admin</username>
      <password>secret</password>
    </credentials>
  </database>
  <replicas>3</replicas>
</system>"#;

let config = FromXmlConfig {
    default_type_name: "Item".to_string(),  // Default for inferred lists
    version: (1, 0),                         // HEDL version
    infer_lists: true,                       // Auto-detect repeated elements
    ..Default::default()                     // Use defaults for entity_policy, log_security_events
};

let hedl_doc = from_xml(xml, &config)?;
// Now use HEDL's structured API for querying, validation, transformation
```

**List Inference**: When `infer_lists: true`, repeated XML elements like `<user>...<user>...` automatically become HEDL matrix lists.

## XSD Schema Validation

Validate XML documents against XSD schemas with detailed, actionable error messages:

```rust
use hedl_xml::schema::SchemaValidator;

let schema_xsd = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="name" type="xs:string"/>
        <xs:element name="age" type="xs:integer"/>
        <xs:element name="email" type="xs:string"/>
      </xs:sequence>
      <xs:attribute name="id" type="xs:string" use="required"/>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

let validator = SchemaValidator::from_xsd(schema_xsd)?;

// Validate XML document
let xml = r#"<?xml version="1.0"?>
<person id="p1">
  <name>Alice</name>
  <age>30</age>
  <email>alice@example.com</email>
</person>"#;

validator.validate(xml)?;  // Returns Ok(()) if valid
```

### Schema Validation Features

**Comprehensive Validation**:
- Element structure validation (sequence, choice, all)
- Type validation (xs:string, xs:integer, xs:decimal, xs:boolean, custom types)
- Attribute validation (required, optional, fixed, default)
- Cardinality validation (minOccurs, maxOccurs, including unbounded)
- Namespace support (multiple namespaces, imports)

**Detailed Error Messages** with line numbers:

```rust
// Invalid XML - age is not an integer
let xml = r#"<?xml version="1.0"?>
<person id="p1">
  <name>Alice</name>
  <age>thirty</age>
  <email>alice@example.com</email>
</person>"#;

let result = validator.validate(xml);
// Error: "Type validation failed for 'age': expected xs:integer, found 'thirty' at line 4"
```

### Schema Caching: High-Performance Validation

For repeated validation operations, use the thread-safe LRU schema cache:

```rust
use hedl_xml::schema::SchemaCache;
use std::path::Path;

// Create cache with capacity for 100 schemas
let cache = SchemaCache::new(100);

// First load: parses and caches schema
let validator = cache.get_or_load(Path::new("api_schema.xsd"))?;
validator.validate(xml1)?;

// Subsequent loads: uses cached validator (no re-parsing)
let validator2 = cache.get_or_load(Path::new("api_schema.xsd"))?;
validator2.validate(xml2)?;

// Monitor cache performance
println!("Cache size: {}", cache.size());
```

**Performance**: Schema caching eliminates parsing overhead for repeated validations. Use in high-throughput services processing thousands of XML documents.

## Streaming: Process Multi-Gigabyte Files

For large XML files (hundreds of MB to several GB), use the streaming parser to process elements incrementally without loading the entire document into memory:

```rust
use hedl_xml::streaming::{from_xml_stream, StreamConfig};
use std::fs::File;

// Open large XML file (e.g., 5 GB database export)
let file = File::open("massive_export.xml")?;

let config = StreamConfig {
    buffer_size: 65536,              // 64 KB buffer (default)
    max_recursion_depth: 100,        // Max XML nesting depth
    max_batch_size: 1000,            // Batch size for list processing
    default_type_name: "Item".to_string(),
    version: (1, 0),
    infer_lists: true,
    ..Default::default()             // Use defaults for entity_policy and log_security_events
};

let mut count = 0;
for result in from_xml_stream(file, &config)? {
    match result {
        Ok(item) => {
            count += 1;
            // Process each item: validate, transform, write to database
            // Memory usage remains constant regardless of file size
        }
        Err(e) => {
            eprintln!("Parse error at item {}: {}", count, e);
        }
    }
}
println!("Processed {} items from multi-GB file", count);
```

**Memory Usage**: O(1) per element. A 5 GB XML file uses the same memory as a 5 MB file. Only the current element and buffer are in memory.

**Streaming vs Buffered**: Use streaming for files >100 MB. For smaller files, use `from_xml()` for simpler code.

## Async I/O with Tokio

Enable async support for non-blocking I/O and concurrent processing (requires `async` feature):

```rust
use hedl_xml::async_api::{from_xml_file_async, to_xml_file_async};
use hedl_xml::{FromXmlConfig, ToXmlConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read XML file asynchronously (doesn't block event loop)
    let doc = from_xml_file_async("input.xml", &FromXmlConfig::default()).await?;

    // Process document...

    // Write XML file asynchronously
    to_xml_file_async(&doc, "output.xml", &ToXmlConfig::default()).await?;

    Ok(())
}
```

### Concurrent Batch Processing

Process multiple XML files concurrently with automatic concurrency limiting:

```rust
use hedl_xml::async_api::from_xml_files_concurrent;
use hedl_xml::FromXmlConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        "export1.xml",
        "export2.xml",
        "export3.xml",
        "export4.xml",
    ];

    let config = FromXmlConfig::default();

    // Process 4 files with concurrency limit of 2
    let results = from_xml_files_concurrent(&files, &config, 2).await;

    for (path, result) in files.iter().zip(results.iter()) {
        match result {
            Ok(doc) => println!("{}: {} items", path, doc.root.len()),
            Err(e) => eprintln!("{}: error - {}", path, e),
        }
    }

    Ok(())
}
```

### Async Streaming for Large Files

Combine streaming with async I/O for maximum throughput:

```rust
use hedl_xml::async_api::from_xml_stream_async;
use hedl_xml::streaming::StreamConfig;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("large.xml").await?;
    let config = StreamConfig::default();

    let mut stream = from_xml_stream_async(file, &config).await?;

    let mut count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(item) => count += 1,
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    println!("Processed {} items", count);

    Ok(())
}
```

## Security Limits: DoS Protection

`hedl-xml` enforces resource limits to prevent denial-of-service attacks from malicious XML files:

### Recursion Depth Limit

**Default**: 100 levels
**Configurable**: Yes, via `StreamConfig::max_recursion_depth` (streaming API). Standard `from_xml()` uses fixed limit.
**Protection**: Prevents stack overflow from deeply nested XML structures

```xml
<!-- Malicious XML with 1000+ nested levels -->
<a><a><a>... (1000 levels deep) ...</a></a></a>
```

Error: `XML recursion depth exceeded (max: 100, found: 101)`

### Batch Size Limit (Streaming)

**Default**: 1,000 elements per batch
**Configurable**: Yes, via `StreamConfig::max_batch_size`
**Protection**: Controls memory usage when processing repeated elements in streams

For the standard (non-streaming) `from_xml()` and `to_xml()` APIs, limits are hardcoded and cannot be adjusted. Use the streaming API if you need custom batch size limits.

**Example with custom recursion limit**:

```rust
use hedl_xml::streaming::StreamConfig;

let config = StreamConfig {
    max_recursion_depth: 50,  // Stricter than default
    max_batch_size: 500,      // Process smaller batches
    ..Default::default()
};
```

**Note on String and List Size Limits**: The error types support reporting string length and list size violations, but the actual limits are enforced at the underlying quick-xml parser level (no individual XML element can exceed XML parser limits). These are not currently user-configurable in hedl-xml.

## Format Mapping

### HEDL → XML

| HEDL Type | XML Output | Notes |
|-----------|------------|-------|
| Scalars (null, bool, number, string) | Element with text content | `<val>42</val>` |
| Objects | Nested elements | `<config><name>test</name></config>` |
| Arrays (tensors) | `<item>` elements | `<tensor><item>1</item><item>2</item></tensor>` |
| References (`@User:alice`) | Element with `__hedl_type__="ref"` attribute | Distinguishes from strings starting with @ |
| Expressions (`$(x + 1)`) | Element with `$()` wrapped text | `<expr>$(x + 1)</expr>` |
| Matrix lists | Repeated elements | `<user>...<user>...` (singularized type name) |

### XML → HEDL

| XML Pattern | HEDL Result | Notes |
|-------------|-------------|-------|
| Elements with text | HEDL scalars | Type inference: "true" → Bool, "42" → Int, "3.14" → Float |
| Nested elements | HEDL objects | Hierarchical structure preserved |
| Repeated elements | HEDL matrix lists | When `infer_lists: true` |
| Element with `__hedl_type__="ref"` | HEDL reference | `@Type:id` format |
| Text matching `$(...)` pattern | HEDL expression | Parsed as computed value |
| Attributes | Object fields | `<item id="1"/>` → `{"id": 1}` |

**Key Conversion**: XML element names are converted to snake_case for HEDL compatibility: `UserPost` → `user_post`, `XMLData` → `xmldata`.

## Use Cases

**SOAP API Integration**: Parse SOAP XML responses into HEDL for structured querying. Generate SOAP XML requests from HEDL templates with validation.

**Configuration Migration**: Convert XML config files (Spring, Tomcat, etc.) to HEDL for LSP-assisted editing with validation. Export back to XML for runtime.

**Data Export/Import**: Stream large XML database exports into HEDL for transformation. Export HEDL to XML for compatibility with legacy ETL tools.

**Schema-First Development**: Define data contracts as XSD schemas. Validate XML payloads in real-time with detailed error reporting. Convert to HEDL for processing.

**Regulatory Compliance**: Parse XML from compliance systems (banking, healthcare, government). Validate against regulatory XSD schemas. Transform with HEDL's structured API.

**Multi-Format Pipelines**: Read XML from SOAP APIs, convert to HEDL, combine with JSON from REST APIs (`hedl-json`), export to CSV for reporting (`hedl-csv`) -all through HEDL's unified data model.

## What This Crate Doesn't Do

**Schema Preservation**: XML doesn't preserve HEDL's `%STRUCT`, `%NEST`, `%ALIAS` declarations (they're HEDL-specific). If you need schemas after round-tripping through XML, use XSD for validation or redefine HEDL schemas.

**Validation**: Converts formats, doesn't validate data. For HEDL schema validation, use `hedl-lint`. For XML schema validation, use `SchemaValidator` with XSD.

**Optimization**: Converts faithfully, not optimally. Verbose XML becomes verbose HEDL (3-5x size overhead). XML is inherently verbose -HEDL's efficiency comes from avoiding XML in the first place.

**XML Comments**: XML comments are discarded during parsing (standard XML processing behavior). Use HEDL comments in source `.hedl` files for preserved documentation.

## Dependencies

- `quick-xml` 0.31 - High-performance XML parsing and serialization
- `roxmltree` 0.20 - XSD schema parsing and validation
- `hedl-core` 2.0 - HEDL parsing and data model
- `parking_lot` 0.12 - High-performance RwLock for schema cache
- `tokio` 1.0 (optional) - Async I/O runtime (requires `async` feature)
- `thiserror` 1.0 - Error type definitions

## Performance Characteristics

**Conversion Speed**: HEDL → XML is serialization-bound (~50-100 MB/s). XML → HEDL is parsing-bound (~100-200 MB/s depending on complexity).

**Schema Validation**: XSD validation adds ~10-20% overhead vs parse-only. Schema caching eliminates re-parsing overhead for repeated validations.

**Streaming**: O(1) memory per element regardless of file size. Process 10 GB files with 100 MB RAM. Throughput: ~50-100 MB/s depending on element complexity.

**Async I/O**: Concurrent file processing scales linearly up to CPU core count. Use for I/O-bound workloads (network file systems, slow disks).

Detailed performance benchmarks are available in the HEDL repository benchmark suite.

## License

Apache-2.0
