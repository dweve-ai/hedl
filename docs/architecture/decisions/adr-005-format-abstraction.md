# ADR-005: Format Abstraction Layer

**Status**: Proposed (Future Work)

**Date**: 2026-01-06

**Context**: Extensible format conversion architecture

---

## Context

HEDL supports multiple data formats (JSON, YAML, XML, CSV, Parquet, Neo4j). We need an architecture that allows:
- Easy addition of new formats
- Consistent conversion APIs
- Optional format dependencies
- Third-party format implementations

**Current Implementation Note**: HEDL currently provides format conversion through separate crates with individual APIs. This ADR documents the planned unified trait-based abstraction for future implementation.

## Decision Drivers

1. **Extensibility**: Easy to add new formats
2. **Consistency**: Uniform API across formats
3. **Optional Dependencies**: Users choose formats
4. **Performance**: Minimal abstraction overhead
5. **Type Safety**: Compile-time format validation

## Considered Options

### Option 1: Trait-Based Abstraction (CHOSEN)

**Approach**: Define `FormatAdapter` trait

```rust
pub trait FormatAdapter {
    fn to_format(&self, doc: &Document) -> Result<String>;
    fn from_format(&self, input: &str) -> Result<Document>;
    fn name(&self) -> &str;
    fn extensions(&self) -> &[&str];
}
```

**Pros**:
- Zero-cost abstraction (monomorphization)
- Type-safe at compile time
- Easy to implement new formats
- No runtime overhead

**Cons**:
- Requires trait implementation
- Cannot load formats dynamically

### Option 2: Enum-Based Dispatch

**Approach**: Enum of all formats

```rust
pub enum Format {
    Json,
    Yaml,
    Xml,
    // ...
}

impl Format {
    pub fn to_format(&self, doc: &Document) -> Result<String> {
        match self {
            Format::Json => hedl_json::to_json(doc),
            Format::Yaml => hedl_yaml::to_yaml(doc),
            // ...
        }
    }
}
```

**Pros**:
- Simple implementation
- Explicit format enumeration
- Pattern matching support

**Cons**:
- Not extensible (closed set)
- Cannot add third-party formats
- Tight coupling to all formats

### Option 3: Dynamic Plugin System

**Approach**: Runtime format loading

```rust
pub struct FormatRegistry {
    formats: HashMap<String, Box<dyn FormatAdapter>>,
}

impl FormatRegistry {
    pub fn register(&mut self, adapter: Box<dyn FormatAdapter>) {
        self.formats.insert(adapter.name().to_string(), adapter);
    }
}
```

**Pros**:
- Dynamic format loading
- Runtime extensibility
- Third-party plugins

**Cons**:
- Runtime overhead (vtable dispatch)
- Complex plugin infrastructure
- Type safety lost at runtime

## Decision

**Chosen**: Option 1 - Trait-Based Abstraction + Static Registry (Planned for Future Implementation)

Hybrid approach: trait-based with optional static registry for convenience.

**Current Status (v1.0.0)**: The codebase currently uses separate crates with individual conversion functions (e.g., `hedl_json::to_json()`, `hedl_json::from_json()`). This ADR documents the planned migration to a unified trait-based system in a future release. All trait-based code examples marked "Planned" are not yet implemented.

## Rationale

### Trait-Based Core (PLANNED - Not Yet Implemented)

Provides zero-cost abstraction:

```rust
// PLANNED FUTURE IMPLEMENTATION - Generic function
pub fn convert<A: FormatAdapter>(
    input: &str,
    adapter: &A,
) -> Result<String> {
    let doc = parse(input)?;
    adapter.to_format(&doc)  // Monomorphized - no vtable
}

// Usage
let json_output = convert(input, &JsonAdapter)?;  // Static dispatch
```

### Static Registry for Convenience (PLANNED - Not Yet Implemented)

Optional registry for runtime format selection:

```rust
// PLANNED FUTURE IMPLEMENTATION
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn FormatAdapter>>,
}

impl AdapterRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(JsonAdapter));
        registry.register(Box::new(YamlAdapter));
        registry
    }

    pub fn convert(
        &self,
        input: &str,
        from: &str,
        to: &str,
    ) -> Result<String> {
        let from_adapter = self.get(from)?;
        let to_adapter = self.get(to)?;

        let doc = from_adapter.from_format(input)?;
        to_adapter.to_format(&doc)
    }
}
```

## Implementation

### Current Format Conversion (CURRENT v1.0.0 Implementation)

Each format crate provides direct conversion functions:

```rust
// hedl-json crate
pub fn to_json(doc: &Document, config: &ToJsonConfig) -> Result<String, String>;
pub fn from_json(json: &str, config: &FromJsonConfig) -> Result<Document, JsonConversionError>;

// hedl-yaml crate
pub fn to_yaml(doc: &Document, config: &ToYamlConfig) -> Result<String, YamlError>;
pub fn from_yaml(yaml: &str, config: &FromYamlConfig) -> Result<Document, YamlError>;

// hedl-xml crate
pub fn to_xml(doc: &Document, config: &ToXmlConfig) -> Result<String, String>;
pub fn from_xml(xml: &str, config: &FromXmlConfig) -> Result<Document, String>;

// etc.
```

### Planned Trait-Based Abstraction (PLANNED - Not Yet Implemented)

Future unified interface:

```rust
// PLANNED FUTURE IMPLEMENTATION
pub trait FormatAdapter: Send + Sync {
    /// Convert HEDL document to format
    fn to_format(
        &self,
        doc: &Document,
        config: &dyn std::any::Any,
    ) -> Result<String>;

    /// Convert format to HEDL document
    fn from_format(
        &self,
        input: &str,
        config: &dyn std::any::Any,
    ) -> Result<Document>;

    /// Format identifier
    fn name(&self) -> &str;

    /// Supported file extensions
    fn extensions(&self) -> &[&str];

    /// MIME type
    fn mime_type(&self) -> &str;

    /// Format-specific validation
    fn validate(&self, doc: &Document) -> Result<()> {
        Ok(())  // Default: no validation
    }
}
```

### Format Implementation (PLANNED - Not Yet Implemented)

```rust
// PLANNED FUTURE IMPLEMENTATION
pub struct JsonAdapter;

impl FormatAdapter for JsonAdapter {
    fn to_format(
        &self,
        doc: &Document,
        config: &dyn std::any::Any,
    ) -> Result<String> {
        let config = config
            .downcast_ref::<ToJsonConfig>()
            .unwrap_or(&ToJsonConfig::default());

        hedl_json::to_json(doc, config)
    }

    fn from_format(
        &self,
        input: &str,
        config: &dyn std::any::Any,
    ) -> Result<Document> {
        let config = config
            .downcast_ref::<FromJsonConfig>()
            .unwrap_or(&FromJsonConfig::default());

        hedl_json::from_json(input, config)
    }

    fn name(&self) -> &str {
        "json"
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }

    fn mime_type(&self) -> &str {
        "application/json"
    }

    fn validate(&self, doc: &Document) -> Result<()> {
        // JSON-specific validation
        self.check_json_compatible(doc)
    }
}
```

### Registry Implementation (PLANNED - Not Yet Implemented)

```rust
// PLANNED FUTURE IMPLEMENTATION
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn FormatAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        registry.register(Box::new(JsonAdapter));

        #[cfg(feature = "yaml")]
        registry.register(Box::new(YamlAdapter));

        #[cfg(feature = "xml")]
        registry.register(Box::new(XmlAdapter));

        registry
    }

    pub fn register(&mut self, adapter: Box<dyn FormatAdapter>) {
        self.adapters.insert(adapter.name().to_string(), adapter);
    }

    pub fn get(&self, name: &str) -> Result<&dyn FormatAdapter> {
        self.adapters
            .get(name)
            .map(|a| &**a)
            .ok_or_else(|| HedlError::conversion(format!("Unknown format: {}", name)))
    }

    pub fn get_by_extension(&self, ext: &str) -> Result<&dyn FormatAdapter> {
        self.adapters
            .values()
            .find(|a| a.extensions().contains(&ext))
            .map(|a| &**a)
            .ok_or_else(|| HedlError::conversion(format!("Unknown extension: {}", ext)))
    }

    pub fn list(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }
}
```

## Consequences

### Positive

1. **Zero-Cost**: Trait monomorphization (no vtable for static dispatch)
2. **Extensible**: Third parties can implement trait
3. **Type-Safe**: Compile-time validation
4. **Optional**: Registry only used when needed
5. **Flexible**: Static or dynamic dispatch

### Negative

1. **Trait Complexity**: Understanding trait implementation
2. **Type Erasure**: `dyn Any` for config (in registry)
3. **Learning Curve**: Trait-based design

## Usage Patterns

### Static Dispatch (Zero-Cost) (PLANNED - Not Yet Implemented)

```rust
// PLANNED FUTURE IMPLEMENTATION - Direct usage - monomorphized
let json = JsonAdapter;
let output = json.to_format(&doc, &ToJsonConfig::default())?;
```

### Dynamic Dispatch (Flexible) (PLANNED - Not Yet Implemented)

```rust
// PLANNED FUTURE IMPLEMENTATION - Registry usage - vtable dispatch
let registry = AdapterRegistry::with_defaults();
let output = registry.convert(input, "hedl", "json")?;
```

### CLI Integration (PLANNED - Not Yet Implemented)

```rust
// PLANNED FUTURE IMPLEMENTATION
let registry = AdapterRegistry::with_defaults();

let from_ext = input_path.extension().unwrap();
let to_ext = output_path.extension().unwrap();

let from_adapter = registry.get_by_extension(from_ext)?;
let to_adapter = registry.get_by_extension(to_ext)?;

let doc = from_adapter.from_format(&input_content, &Default::default())?;
let output = to_adapter.to_format(&doc, &Default::default())?;
```

## Related ADRs

- [ADR-001: Workspace Structure](adr-001-workspace-structure.md) - Format crate organization

## References

- Rust trait objects: https://doc.rust-lang.org/book/ch17-02-trait-objects.html
- Zero-cost abstractions: https://boats.gitlab.io/blog/post/zero-cost-abstractions/

---

