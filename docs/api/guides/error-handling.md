# Error Handling Guide

Comprehensive error handling strategies for HEDL across all API surfaces.

## Error Philosophy

HEDL follows these error handling principles:

1. **Explicit over implicit**: All errors are explicit and typed
2. **Recoverable when possible**: Distinguish recoverable from fatal errors
3. **Context-rich**: Errors include location, kind, and helpful messages
4. **Language-appropriate**: Error handling matches language idioms

## Rust Error Handling

### Basic Result Pattern

```rust
use hedl::{parse, HedlError};

fn load_config(path: &str) -> Result<Config, HedlError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| HedlError::new(hedl::HedlErrorKind::IO, e.to_string(), 0))?;
    let doc = parse(&content)?;
    Config::from_document(&doc)
}

match load_config("config.hedl") {
    Ok(config) => println!("Loaded: {:?}", config),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Pattern Matching on Error Kind

```rust
use hedl::{parse, HedlErrorKind};

match parse(input) {
    Ok(doc) => process(doc),
    Err(e) => match e.kind {
        HedlErrorKind::Syntax => {
            eprintln!("Syntax error at line {}: {}", e.line, e.message);
            show_error_context(input, e.line);
        }
        HedlErrorKind::Reference => {
            eprintln!("Reference error, trying lenient mode...");
            match hedl::parse_lenient(input) {
                Ok(doc) => process(doc),
                Err(e2) => fatal_error(e2),
            }
        }
        HedlErrorKind::Security => {
            eprintln!("Document exceeds limits: {}", e.message);
            suggest_limit_increase(&e);
        }
        _ => fatal_error(e),
    }
}
```

### Error Context Chain

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("HEDL error: {0}")]
    Hedl(#[from] HedlError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation failed: {0}")]
    Validation(String),
}

fn load_with_context(path: &str) -> Result<Document, AppError> {
    std::fs::read_to_string(path)
        .map_err(|e| AppError::Io(e))
        .and_then(|content| {
            parse(&content)
                .map_err(|e| AppError::Hedl(e))
        })
}
```

### Custom Error Extensions

```rust
pub trait HedlErrorExt {
    fn with_context(self, context: impl Into<String>) -> Self;
    fn is_recoverable(&self) -> bool;
}

impl HedlErrorExt for HedlError {
    fn with_context(mut self, context: impl Into<String>) -> Self {
        self.message = format!("{}: {}", context.into(), self.message);
        self
    }

    fn is_recoverable(&self) -> bool {
        matches!(self.kind,
            HedlErrorKind::Reference |
            HedlErrorKind::Semantic
        )
    }
}

// Usage
let doc = parse(input)
    .map_err(|e| e.with_context("Failed to parse user config"))?;
```

## FFI Error Handling

### Error Code Pattern

```c
#include "hedl.h"

int process_hedl(const char* input) {
    HedlDocument* doc = NULL;
    char* json = NULL;
    int result = 0;

    HedlErrorCode code = hedl_parse(input, -1, 0, &doc);
    if (code != HEDL_OK) {
        const char* error = hedl_get_last_error();
        fprintf(stderr, "Parse failed (%d): %s\n", code, error);
        result = -1;
        goto cleanup;
    }

    code = hedl_to_json(doc, 0, &json);
    if (code != HEDL_OK) {
        const char* error = hedl_get_last_error();
        fprintf(stderr, "JSON conversion failed (%d): %s\n", code, error);
        result = -2;
        goto cleanup;
    }

    printf("Success: %s\n", json);

cleanup:
    if (json) hedl_free_string(json);
    if (doc) hedl_free_document(doc);
    return result;
}
```

### C++ Exception Wrapper

```cpp
#include <stdexcept>
#include <string>
#include "hedl.h"

class HedlException : public std::runtime_error {
public:
    HedlException(HedlErrorCode code, const std::string& msg)
        : std::runtime_error(msg), code_(code) {}

    HedlErrorCode code() const { return code_; }

private:
    HedlErrorCode code_;
};

class HedlDocument {
public:
    HedlDocument(const std::string& input) {
        HedlDocument* raw = nullptr;
        HedlErrorCode code = hedl_parse(input.c_str(), -1, 0, &raw);

        if (code != HEDL_OK) {
            std::string err = hedl_get_last_error();
            throw HedlException(code, err);
        }

        doc_.reset(raw);
    }

    std::string to_json() const {
        char* raw_json = nullptr;
        HedlErrorCode code = hedl_to_json(doc_.get(), 0, &raw_json);

        if (code != HEDL_OK) {
            std::string err = hedl_get_last_error();
            throw HedlException(code, err);
        }

        std::unique_ptr<char, decltype(&hedl_free_string)> json(
            raw_json, hedl_free_string
        );

        return std::string(json.get());
    }

private:
    std::unique_ptr<HedlDocument, decltype(&hedl_free_document)> doc_;
};

// Usage with try-catch
try {
    HedlDocument doc(input);
    std::string json = doc.to_json();
    std::cout << json << std::endl;
} catch (const HedlException& e) {
    std::cerr << "HEDL error (" << e.code() << "): "
              << e.what() << std::endl;
}
```

## JavaScript/WASM Error Handling

### Try-Catch Pattern

```typescript
import { parse, HedlError } from 'hedl-wasm';

function safeParseHedl(input: string) {
    try {
        const doc = parse(input);
        return { success: true, doc };
    } catch (error) {
        if (error instanceof HedlError) {
            return {
                success: false,
                error: {
                    kind: error.kind,
                    message: error.message,
                    line: error.line,
                }
            };
        }
        return {
            success: false,
            error: { message: 'Unknown error' }
        };
    }
}
```

### Promise-Based Error Handling

```typescript
async function loadAndParseHedl(url: string): Promise<any> {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const text = await response.text();
        const doc = parse(text);
        return doc;

    } catch (error) {
        if (error instanceof HedlError) {
            console.error(`HEDL parse error at line ${error.line}: ${error.message}`);
        } else {
            console.error('Failed to load HEDL:', error);
        }
        throw error;
    }
}
```

### Result Type Pattern

```typescript
type Result<T, E = Error> =
    | { ok: true; value: T }
    | { ok: false; error: E };

function parseHedl(input: string): Result<any, HedlError> {
    try {
        const doc = parse(input);
        return { ok: true, value: doc };
    } catch (error) {
        if (error instanceof HedlError) {
            return { ok: false, error };
        }
        throw error; // Re-throw unexpected errors
    }
}

// Usage
const result = parseHedl(input);
if (result.ok) {
    console.log('Parsed:', result.value);
} else {
    console.error('Error:', result.error.message);
}
```

## Error Recovery Strategies

### Lenient Parsing

```rust
use hedl::{parse, parse_lenient};

fn parse_with_fallback(input: &str) -> Result<Document, HedlError> {
    match parse(input) {
        Ok(doc) => Ok(doc),
        Err(e) if e.kind == HedlErrorKind::Reference => {
            eprintln!("Warning: Using lenient mode due to unresolved references");
            parse_lenient(input)
        }
        Err(e) => Err(e),
    }
}
```

### Partial Parsing

```rust
fn parse_best_effort(input: &str) -> (Option<Document>, Vec<HedlError>) {
    let mut errors = Vec::new();

    match parse(input) {
        Ok(doc) => (Some(doc), errors),
        Err(e) => {
            errors.push(e);
            // Try parsing individual sections
            // (Implementation depends on requirements)
            (None, errors)
        }
    }
}
```

### Validation with Warnings

```rust
use hedl::{parse, lint};
use hedl::lint::{Diagnostic, Severity};

fn parse_with_validation(input: &str) -> Result<(Document, Vec<Diagnostic>), HedlError> {
    let doc = parse(input)?;
    let diagnostics = lint(&doc);

    let errors: Vec<_> = diagnostics.iter()
        .filter(|d| d.severity() == Severity::Error)
        .collect();

    if !errors.is_empty() {
        return Err(HedlError::semantic(
            format!("Validation failed: {} errors", errors.len()),
            0
        ));
    }

    Ok((doc, diagnostics))
}
```

## User-Friendly Error Messages

### Context Display

```rust
fn show_error_with_context(input: &str, error: &HedlError) {
    eprintln!("Error at line {}: {}", error.line, error.message);

    let lines: Vec<&str> = input.lines().collect();
    if error.line > 0 && error.line <= lines.len() {
        let line_num = error.line;
        let line = lines[line_num - 1];

        eprintln!("{:4} | {}", line_num, line);
    }
}
```

### Suggestions

```rust
fn suggest_fix(error: &HedlError) -> Option<String> {
    match &error.kind {
        HedlErrorKind::Syntax if error.message.contains("colon") => {
            Some("Did you forget a ':' after the key?".to_string())
        }
        HedlErrorKind::Reference => {
            Some(format!("Reference not found. Did you define it? Details: {}", error.message))
        }
        HedlErrorKind::Security if error.message.contains("depth") => {
            Some("Try reducing nesting depth or increasing max_nest_depth limit".to_string())
        }
        _ => None,
    }
}
```

## Logging and Monitoring

### Structured Logging

```rust
use tracing::{error, warn, info};

fn parse_with_logging(input: &str) -> Result<Document, HedlError> {
    info!("Parsing HEDL document");

    match parse(input) {
        Ok(doc) => {
            info!("Successfully parsed document with {} items", doc.root.len());
            Ok(doc)
        }
        Err(e) => {
            error!(
                error_kind = ?e.kind,
                line = e.line,
                column = ?e.column,
                message = %e.message,
                "Failed to parse HEDL document"
            );
            Err(e)
        }
    }
}
```

### Error Metrics

```rust
use prometheus::{IntCounter, register_int_counter};

lazy_static! {
    static ref PARSE_ERRORS: IntCounter =
        register_int_counter!("hedl_parse_errors_total", "Total parse errors").unwrap();
}

fn parse_with_metrics(input: &str) -> Result<Document, HedlError> {
    match parse(input) {
        Ok(doc) => Ok(doc),
        Err(e) => {
            PARSE_ERRORS.inc();
            Err(e)
        }
    }
}
```

## Best Practices

### Do's

1. **Check error codes** immediately after FFI calls
2. **Use error context** to provide meaningful messages
3. **Log errors** with structured data for debugging
4. **Distinguish** recoverable from fatal errors
5. **Clean up resources** on error paths

### Don'ts

1. **Don't ignore errors** - handle or propagate
2. **Don't use panic** in library code
3. **Don't leak memory** on error paths
4. **Don't use generic error messages** - be specific
5. **Don't hide root causes** - preserve error chains

## Testing Error Paths

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_error() {
        let result = parse("invalid");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err.kind, HedlErrorKind::Syntax));
    }

    #[test]
    fn test_reference_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nuser: @User:missing";
        let result = parse(input);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err.kind, HedlErrorKind::Reference));
    }

    #[test]
    fn test_resource_limit() {
        let options = ParseOptions {
            limits: Limits {
                max_indent_depth: 2,
                ..Default::default()
            },
            ..Default::default()
        };

        let input = "%VERSION: 1.0\n---\na:\n  b:\n    c:\n      d: too deep";
        let result = parse_with_limits(input.as_bytes(), options);
        assert!(result.is_err());
    }
}
```

## See Also

- [Errors Reference](../errors.md)
- [Rust Best Practices](rust-best-practices.md)
- [Thread Safety Guide](thread-safety.md)
- [Memory Management Guide](memory-management.md)
