# Dependency Injection Patterns

> Configuration management and trait-based abstractions in HEDL

## Overview

> **Note**: This document describes both current and planned dependency injection patterns in HEDL. While configuration structs and builder patterns are fully implemented, the trait-based `FormatAdapter` and `ParsingStrategy` abstractions represent the intended roadmap for future extensibility.

HEDL uses dependency injection patterns to achieve loose coupling, testability, and configurability.

## DI Approaches in HEDL

### 1. Trait-Based Abstraction

HEDL defines traits for core abstractions, allowing different implementations:

```rust
// Core abstraction
pub trait FormatAdapter {
    fn to_format(&self, doc: &Document) -> Result<String>;
    fn from_format(&self, input: &str) -> Result<Document>;
}

// Concrete implementations
pub struct JsonAdapter {
    config: ToJsonConfig,
}

impl FormatAdapter for JsonAdapter {
    fn to_format(&self, doc: &Document) -> Result<String> {
        to_json(doc, &self.config)
    }

    fn from_format(&self, input: &str) -> Result<Document> {
        from_json(input, &FromJsonConfig::default())
    }
}

// Consumer uses trait
pub fn convert<A: FormatAdapter>(
    input: &[u8],
    adapter: &A,
) -> Result<String> {
    let doc = parse(input)?;
    adapter.to_format(&doc)
}
```

**Benefits**:
- Polymorphism without runtime overhead
- Testable with mock implementations
- Zero-cost abstraction via monomorphization

### 2. Configuration Structs

Configuration is injected via dedicated structs:

```rust
// Actual structure from hedl-core/src/parser.rs
pub struct ParseOptions {
    pub limits: Limits,
    pub strict_refs: bool,
}

// Limits structure from hedl-core/src/limits.rs
pub struct Limits {
    pub max_file_size: usize,
    pub max_line_length: usize,
    pub max_indent_depth: usize,
    pub max_nodes: usize,
    pub max_aliases: usize,
    pub max_columns: usize,
    pub max_nest_depth: usize,
    pub max_block_string_size: usize,
    pub max_object_keys: usize,
    pub max_total_keys: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            strict_refs: true,
        }
    }
}

// Usage: inject configuration
pub fn parse(input: &[u8]) -> HedlResult<Document> {
    parse_with_limits(input, ParseOptions::default())
}

pub fn parse_with_limits(input: &[u8], options: ParseOptions) -> HedlResult<Document> {
    // Use options throughout parsing
    if input.len() > options.limits.max_file_size {
        return Err(HedlError::security("File size limit exceeded"));
    }
    // ...
}
```

**Benefits**:
- Explicit dependencies
- Testable with custom configurations
- Type-safe configuration

### 3. Builder Pattern

Complex configuration uses builders:

```rust
pub struct ToJsonConfig {
    indent: usize,
    sort_keys: bool,
    schema_generation: bool,
    pretty: bool,
}

impl ToJsonConfig {
    pub fn builder() -> ToJsonConfigBuilder {
        ToJsonConfigBuilder::default()
    }
}

pub struct ToJsonConfigBuilder {
    indent: Option<usize>,
    sort_keys: Option<bool>,
    schema_generation: Option<bool>,
    pretty: Option<bool>,
}

impl ToJsonConfigBuilder {
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = Some(indent);
        self
    }

    pub fn sort_keys(mut self, sort: bool) -> Self {
        self.sort_keys = Some(sort);
        self
    }

    pub fn schema_generation(mut self, gen: bool) -> Self {
        self.schema_generation = Some(gen);
        self
    }

    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = Some(pretty);
        self
    }

    pub fn build(self) -> ToJsonConfig {
        ToJsonConfig {
            indent: self.indent.unwrap_or(2),
            sort_keys: self.sort_keys.unwrap_or(false),
            schema_generation: self.schema_generation.unwrap_or(false),
            pretty: self.pretty.unwrap_or(true),
        }
    }
}

// Usage
let config = ToJsonConfig {
    include_metadata: true,
    flatten_lists: false,
    include_children: true,
};

let json = hedl_json::to_json(&doc, &config)?;
```

**Benefits**:
- Fluent API
- Optional parameters
- Compile-time validation

### 4. Function Parameter Injection

Simple cases use direct parameter injection:

```rust
pub fn canonicalize(
    doc: &Document,
    config: &C14nConfig,
) -> Result<String> {
    // Config injected as parameter
    let indent = " ".repeat(config.indent);
    // ...
}

// Usage
let config = C14nConfig {
    indent: 4,
    sort_keys: true,
    quote_style: QuoteStyle::Minimal,
};

let canonical = canonicalize(&doc, &config)?;
```

## Core Abstractions

### Format Adapter Trait

```rust
/// Trait for bidirectional format conversion
pub trait FormatAdapter: Send + Sync {
    /// Convert HEDL document to format-specific representation
    fn to_format(&self, doc: &Document, config: &dyn Any) -> Result<String>;

    /// Convert format-specific representation to HEDL document
    fn from_format(&self, input: &str, config: &dyn Any) -> Result<Document>;

    /// Format name for identification
    fn name(&self) -> &str;
}

// Implementation example
pub struct JsonFormatAdapter;

impl FormatAdapter for JsonFormatAdapter {
    fn to_format(&self, doc: &Document, config: &dyn Any) -> Result<String> {
        let config = config.downcast_ref::<ToJsonConfig>()
            .ok_or_else(|| HedlError::conversion("Invalid config type"))?;
        to_json(doc, config)
    }

    fn from_format(&self, input: &str, config: &dyn Any) -> Result<Document> {
        let config = config.downcast_ref::<FromJsonConfig>()
            .ok_or_else(|| HedlError::conversion("Invalid config type"))?;
        from_json(input, config)
    }

    fn name(&self) -> &str {
        "json"
    }
}
```

### Lint Rule Trait

```rust
/// Trait for lint rules
pub trait LintRule: Send + Sync {
    /// Rule name/identifier
    fn name(&self) -> &str;

    /// Rule description
    fn description(&self) -> &str;

    /// Check document and return diagnostics
    fn check(&self, doc: &Document) -> Vec<Diagnostic>;

    /// Severity level
    fn severity(&self) -> Severity {
        Severity::Warning
    }
}

// Implementation example
pub struct DeepNestingRule {
    max_depth: usize,
}

impl LintRule for DeepNestingRule {
    fn name(&self) -> &str {
        "deep-nesting"
    }

    fn description(&self) -> &str {
        "Detects deeply nested structures"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        // Traverse root items to find deeply nested structures
        self.check_item_depth(&doc.root, 0, &mut diagnostics);
        diagnostics
    }
}

impl DeepNestingRule {
    fn check_item_depth(&self, items: &BTreeMap<String, Item>, depth: usize, diagnostics: &mut Vec<Diagnostic>) {
        if depth > self.max_depth {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!("Nesting depth {} exceeds {}", depth, self.max_depth),
                position: 0,
            });
            return;
        }

        for item in items.values() {
            match item {
                Item::Object(children) => {
                    self.check_item_depth(children, depth + 1, diagnostics);
                }
                Item::List(matrix) => {
                    for node in &matrix.rows {
                        // Check nested children in matrix rows
                        for children in node.children.values() {
                            // Matrix children follow NEST relationships
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }
}

// Lint engine with injected rules
pub struct LintEngine {
    rules: Vec<Box<dyn LintRule>>,
}

impl LintEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule<R: LintRule + 'static>(mut self, rule: R) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn run(&self, doc: &Document) -> Vec<Diagnostic> {
        self.rules.iter()
            .flat_map(|rule| rule.check(doc))
            .collect()
    }
}

// Usage
let engine = LintEngine::new()
    .add_rule(DeepNestingRule { max_depth: 10 })
    .add_rule(LargeDocumentRule { max_nodes: 1000 });

let diagnostics = engine.run(&doc);
```

### Parser Strategy Trait

```rust
/// Trait for different parsing strategies
pub trait ParsingStrategy: Send + Sync {
    /// Parse input with given options
    fn parse(&self, input: &str, options: &ParseOptions) -> Result<Document>;

    /// Strategy name
    fn name(&self) -> &str;
}

// Standard synchronous parser
pub struct SyncParser;

impl ParsingStrategy for SyncParser {
    fn parse(&self, input: &str, options: &ParseOptions) -> Result<Document> {
        hedl_core::parse_with_limits(input.as_bytes(), options.clone())
    }

    fn name(&self) -> &str {
        "sync"
    }
}

// Streaming parser for large files
pub struct StreamingParser;

impl ParsingStrategy for StreamingParser {
    fn parse(&self, input: &str, options: &ParseOptions) -> Result<Document> {
        // Use streaming implementation for large inputs
        hedl_core::parse_with_limits(input.as_bytes(), options.clone())
    }

    fn name(&self) -> &str {
        "streaming"
    }
}

// Parser selector with injected strategy
pub struct Parser {
    strategy: Box<dyn ParsingStrategy>,
}

impl Parser {
    pub fn new<S: ParsingStrategy + 'static>(strategy: S) -> Self {
        Self {
            strategy: Box::new(strategy),
        }
    }

    pub fn parse(&self, input: &str, options: &ParseOptions) -> Result<Document> {
        self.strategy.parse(input, options)
    }
}

// Usage
let parser = Parser::new(SyncParser);
let doc = parser.parse(input.as_bytes(), &ParseOptions::default())?;

// Or with streaming
let streaming_parser = Parser::new(StreamingParser);
let doc = streaming_parser.parse(large_input.as_bytes(), &ParseOptions::default())?;
```

## Configuration Patterns

### Hierarchical Configuration

```rust
/// System-wide defaults
pub mod defaults {
    pub const MAX_FILE_SIZE: usize = 1_000_000_000;
    pub const MAX_TOTAL_KEYS: usize = 10_000_000;
    pub const MAX_INDENT_DEPTH: usize = 50;
}

/// Environment-specific configuration
pub struct Environment {
    pub name: String,
    pub parse_options: ParseOptions,
    pub json_config: ToJsonConfig,
}

impl Environment {
    /// Production environment with strict limits
    pub fn production() -> Self {
        Self {
            name: "production".to_string(),
            parse_options: ParseOptions::builder()
                .max_file_size(defaults::MAX_FILE_SIZE)
                .max_total_keys(defaults::MAX_TOTAL_KEYS)
                .max_depth(defaults::MAX_INDENT_DEPTH)
                .strict(true)
                .build(),
            json_config: ToJsonConfig::builder()
                .pretty(false)  // Minified in production
                .schema_generation(false)
                .build(),
        }
    }

    /// Development environment with relaxed limits
    pub fn development() -> Self {
        Self {
            name: "development".to_string(),
            parse_options: ParseOptions::builder()
                .limits(Limits::unlimited())
                .strict(false)
                .build(),
            json_config: ToJsonConfig::builder()
                .pretty(true)
                .schema_generation(true)
                .build(),
        }
    }

    /// Test environment
    pub fn test() -> Self {
        Self {
            name: "test".to_string(),
            parse_options: ParseOptions::default(),
            json_config: ToJsonConfig::default(),
        }
    }
}

// Usage
let env = Environment::production();
let doc = parse(input, &env.parse_options)?;
let json = to_json(&doc, &env.json_config)?;
```

### Configuration Composition

```rust
/// Composable configuration traits
pub trait Configurable {
    type Config;
    fn config(&self) -> &Self::Config;
}

pub struct JsonConverter {
    to_config: ToJsonConfig,
    from_config: FromJsonConfig,
}

impl Configurable for JsonConverter {
    type Config = (ToJsonConfig, FromJsonConfig);

    fn config(&self) -> &Self::Config {
        (&self.to_config, &self.from_config)
    }
}

impl JsonConverter {
    pub fn new(to_config: ToJsonConfig, from_config: FromJsonConfig) -> Self {
        Self { to_config, from_config }
    }

    pub fn convert(&self, input: &str) -> Result<String> {
        let doc = from_json(input, &self.from_config)?;
        to_json(&doc, &self.to_config)
    }
}

// Usage: inject both configurations
let converter = JsonConverter::new(
    ToJsonConfig::builder().pretty(true).build(),
    FromJsonConfig::default(),
);

let output = converter.convert(input)?;
```

## Testing with DI

### Mock Implementations

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Mock format adapter for testing
    struct MockAdapter {
        to_result: Result<String>,
        from_result: Result<Document>,
    }

    impl FormatAdapter for MockAdapter {
        fn to_format(&self, _doc: &Document, _config: &dyn Any) -> Result<String> {
            self.to_result.clone()
        }

        fn from_format(&self, _input: &str, _config: &dyn Any) -> Result<Document> {
            self.from_result.clone()
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn test_conversion_error_handling() {
        let mock = MockAdapter {
            to_result: Err(HedlError::syntax("test error", 1)),
            from_result: Ok(Document::default()),
        };

        let result = convert("test", &mock);
        assert!(result.is_err());
    }
}
```

### Test Configurations

```rust
#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// Create test configuration with minimal limits for fast tests
    pub fn test_parse_options() -> ParseOptions {
        let limits = Limits {
            max_file_size: 1_000_000,
            max_total_keys: 10_000,
            max_indent_depth: 20,
            max_nodes: 10_000,
            ..Limits::default()
        };
        ParseOptions::builder()
            .limits(limits)
            .strict(true)
            .build()
    }

    /// Create test JSON config
    pub fn test_json_config() -> ToJsonConfig {
        ToJsonConfig::builder()
            .pretty(true)
            .sort_keys(true)  // Deterministic output
            .build()
    }
}

// Usage in tests
#[test]
fn test_parse_with_limits() {
    use test_helpers::*;

    let doc = parse(input.as_bytes(), &test_parse_options()).unwrap();
    assert_eq!(doc.root.len(), 3);
}
```

### Dependency Injection in Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    struct TestContext {
        parser: Parser,
        json_adapter: JsonAdapter,
        lint_engine: LintEngine,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                parser: Parser::new(SyncParser),
                json_adapter: JsonAdapter {
                    config: test_helpers::test_json_config(),
                },
                lint_engine: LintEngine::new()
                    .add_rule(DeepNestingRule { max_depth: 10 }),
            }
        }

        fn run_pipeline(&self, input: &str) -> Result<String> {
            // Parse
            let doc = self.parser.parse(input.as_bytes(), &test_parse_options())?;

            // Lint
            let diagnostics = self.lint_engine.run(&doc);
            if diagnostics.iter().any(|d| d.severity == Severity::Error) {
                return Err(HedlError::syntax("Validation failed", 0));
            }

            // Convert
            hedl_json::to_json(&doc, &Default::default())
        }
    }

    #[test]
    fn test_full_pipeline() {
        let ctx = TestContext::new();
        let result = ctx.run_pipeline("%VERSION: 1.0\n---\nkey: value");
        assert!(result.is_ok());
    }
}
```

## Advanced Patterns

### Strategy Pattern with Runtime Selection

```rust
pub enum ParserType {
    Sync,
    Streaming,
    Parallel,
}

pub fn create_parser(parser_type: ParserType) -> Box<dyn ParsingStrategy> {
    match parser_type {
        ParserType::Sync => Box::new(SyncParser),
        ParserType::Streaming => Box::new(StreamingParser),
        ParserType::Parallel => Box::new(ParallelParser),
    }
}

// Usage
let parser_type = if input.len() > 100_000_000 {
    ParserType::Streaming
} else {
    ParserType::Sync
};

let parser = Parser::new_boxed(create_parser(parser_type));
let doc = parser.parse(input, &ParseOptions::default())?;
```

### Decorator Pattern for Middleware

```rust
/// Middleware trait for parser decoration
pub trait ParserMiddleware: Send + Sync {
    fn before_parse(&self, input: &str) -> Result<()>;
    fn after_parse(&self, doc: &Document) -> Result<()>;
}

pub struct LoggingMiddleware;

impl ParserMiddleware for LoggingMiddleware {
    fn before_parse(&self, input: &str) -> Result<()> {
        println!("Parsing {} bytes", input.len());
        Ok(())
    }

    fn after_parse(&self, doc: &Document) -> Result<()> {
        println!("Parsed {} nodes", doc.nodes().len());
        Ok(())
    }
}

pub struct ValidationMiddleware {
    strict: bool,
}

impl ParserMiddleware for ValidationMiddleware {
    fn before_parse(&self, _input: &str) -> Result<()> {
        Ok(())
    }

    fn after_parse(&self, doc: &Document) -> Result<()> {
        if self.strict {
            validate(doc)?;
        }
        Ok(())
    }
}

pub struct DecoratedParser {
    inner: Box<dyn ParsingStrategy>,
    middleware: Vec<Box<dyn ParserMiddleware>>,
}

impl DecoratedParser {
    pub fn new<S: ParsingStrategy + 'static>(strategy: S) -> Self {
        Self {
            inner: Box::new(strategy),
            middleware: Vec::new(),
        }
    }

    pub fn with_middleware<M: ParserMiddleware + 'static>(
        mut self,
        middleware: M,
    ) -> Self {
        self.middleware.push(Box::new(middleware));
        self
    }

    pub fn parse(&self, input: &str, options: &ParseOptions) -> Result<Document> {
        // Run before hooks
        for mw in &self.middleware {
            mw.before_parse(input)?;
        }

        // Parse
        let doc = self.inner.parse(input, options)?;

        // Run after hooks
        for mw in &self.middleware {
            mw.after_parse(&doc)?;
        }

        Ok(doc)
    }
}

// Usage
let parser = DecoratedParser::new(SyncParser)
    .with_middleware(LoggingMiddleware)
    .with_middleware(ValidationMiddleware { strict: true });

let doc = parser.parse(input.as_bytes(), &ParseOptions::default())?;
```

## Best Practices

### 1. Prefer Compile-Time Injection

Use generics and traits for zero-cost abstraction:

```rust
// ✅ Good: Compile-time polymorphism
pub fn convert<A: FormatAdapter>(input: &[u8], adapter: &A) -> Result<String> {
    let doc = parse(input, &ParseOptions::default())?;
    adapter.to_format(&doc, &Default::default())
}

// ❌ Avoid: Runtime polymorphism unless necessary
pub fn convert_dyn(input: &[u8], adapter: &dyn FormatAdapter) -> Result<String> {
    let doc = parse(input, &ParseOptions::default())?;
    adapter.to_format(&doc, &Default::default())
}
```

### 2. Use Configuration Structs

Explicit configuration over implicit globals:

```rust
// ✅ Good: Explicit configuration
pub fn parse_with_limits(input: &[u8], options: ParseOptions) -> HedlResult<Document> {
    // ...
}

// ❌ Avoid: Global configuration
static mut PARSE_OPTIONS: ParseOptions = /* ... */;
pub fn parse(input: &[u8]) -> Result<Document> {
    unsafe { parse_with_limits(input, PARSE_OPTIONS.clone()) }
}
```

### 3. Provide Sensible Defaults

Use `Default` trait for common configurations:

```rust
impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            strict_refs: true,
        }
    }
}

// Usage: easy to use with defaults
let doc = parse(input)?;

// Or customize as needed
let options = ParseOptions::builder()
    .strict(false)
    .build();
let doc = parse_with_limits(input.as_bytes(), options)?;
```

### 4. Make Dependencies Explicit

Function signatures should reveal dependencies:

```rust
// ✅ Good: Explicit dependencies
pub fn lint_document(
    doc: &Document,
    rules: &[Box<dyn LintRule>],
    config: &LintConfig,
) -> Vec<Diagnostic> {
    // ...
}

// ❌ Avoid: Hidden dependencies
pub fn lint_document(doc: &Document) -> Vec<Diagnostic> {
    // Uses implicit global rules
}
```

## Related Documentation

- [Layered Architecture](layered-architecture.md) - Layer abstractions
- [Plugin Architecture](plugin-architecture.md) - Extension points
- [Testing Strategy](../infrastructure/testing.md) - Testing patterns
- [Module Structure](module-structure.md) - Crate organization

---

