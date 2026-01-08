# How-To: Debug Parser Issues

Learn how to diagnose and fix parsing problems in HEDL.

## Goal

Successfully diagnose why HEDL text isn't parsing correctly and fix the issue.

## Prerequisites

- HEDL source code cloned
- Rust toolchain installed
- Basic understanding of parsers

## Common Scenarios

### Scenario 1: Parsing Fails with Generic Error

**Symptoms**: `parse()` returns error but message isn't helpful.

**Solution**: Enable detailed error reporting.

```rust
use hedl_core::{parse, parse_with_limits, ParseOptions};

let input = b"problematic: content";

// Basic parse (limited error info)
match parse(input) {
    Err(e) => eprintln!("Error: {}", e),
    Ok(_) => println!("Parsed successfully"),
}

// Detailed parse with custom limits
let options = ParseOptions::builder()
    .max_indent_depth(100)
    .strict_refs(true)  // Strict reference validation
    .build();

match parse_with_limits(input, options) {
    Err(e) => {
        eprintln!("Parse error: {:?}", e);
        eprintln!("Kind: {:?}", e.kind);
        eprintln!("Line: {}", e.line);
    }
    Ok(_) => println!("Success"),
}
```

### Scenario 2: Lexer vs Parser Error

**Problem**: Need to know if error is in tokenization or syntax.

```bash
# Test lexer separately
cargo test -p hedl-core lex::tests -- --nocapture

# Test with minimal input
cat > test.hedl << 'EOF'
%VERSION: 1.0
---
key: value
EOF

cargo run --bin hedl validate test.hedl --strict
```

**Debug lexer**:

```rust
use hedl_core::lex::{is_valid_key_token, parse_csv_row};

// Test token validation
let key = "my-key";
if !is_valid_key_token(key) {
    eprintln!("Invalid key: {}", key);
}

// Test CSV parsing
let row = "alice, 30, true";
match parse_csv_row(row) {
    Ok(fields) => println!("Fields: {:?}", fields),
    Err(e) => eprintln!("Lexer error: {:?}", e),
}
```

### Scenario 3: Indentation Issues

**Symptoms**: Nested structures not recognized.

**Debug steps**:

1. Check for tabs vs spaces:
```bash
cat -A problematic.hedl | grep '\^I'
```

2. Verify indentation is consistent (2 spaces):
```hedl
%VERSION: 1.0
---
parent:
  child1:
    nested: value
  child2:
    nested: value
```

3. Use the preprocessor directly:
```rust
use hedl_core::parse;

let input = "parent:\n\tchild: value";  // Tab character

// Try parsing - tabs will be reported during preprocessing
match parse(input.as_bytes()) {
    Ok(doc) => println!("Document parsed: {:?}", doc),
    Err(e) => eprintln!("Parse failed: {}", e),
}
```

### Scenario 4: Reference Resolution Fails

**Symptoms**: References like `@User:alice` not resolving.

**Debug**:

```rust
use hedl_core::{parse, Document, Value, Item};

let hedl = b"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
post:
  author: @User:alice";

match parse(hedl) {
    Ok(doc) => {
        // Check root structure
        println!("Root keys: {:?}", doc.root.keys().collect::<Vec<_>>());

        // Manually verify reference by traversing structure
        for (key, item) in &doc.root {
            println!("Key: {}", key);
            match item {
                Item::Scalar(Value::Reference(ref_val)) => {
                    println!("Found reference: {:?}", ref_val);
                }
                Item::Object(map) => {
                    for (k, v) in map {
                        if let Item::Scalar(Value::Reference(r)) = v {
                            println!("Found nested reference in {}: {:?}", k, r);
                        }
                    }
                }
                Item::List(matrix) => {
                    println!("Found matrix list with {} rows", matrix.rows.len());
                }
                _ => {}
            }
        }
    }
    Err(e) => eprintln!("Parse failed: {}", e),
}
```

## Advanced Debugging Techniques

### Enable Tracing

Add to `Cargo.toml`:
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

In code:
```rust
use tracing::{debug, info, warn, trace};

fn parse_with_tracing(input: &[u8]) -> Result<Document> {
    tracing_subscriber::fmt::init();

    info!("Starting parse, input length: {}", input.len());
    trace!("Input: {:?}", String::from_utf8_lossy(input));

    let result = parse(input);

    match &result {
        Ok(doc) => info!("Parse succeeded, nodes: {}", count_nodes(doc)),
        Err(e) => warn!("Parse failed: {}", e),
    }

    result
}
```

Run with:
```bash
RUST_LOG=debug cargo run --example my_example
```

### Use Debugger

**With VS Code**:

1. Create `.vscode/launch.json`:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug unit test",
            "cargo": {
                "args": ["test", "--no-run", "--lib", "-p", "hedl-core"],
                "filter": {
                    "name": "test_parse_simple",
                    "kind": "test"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

2. Set breakpoint in `crates/hedl-core/src/parser.rs`
3. Press F5 to debug

**With GDB**:

```bash
# Build with debug symbols
cargo build --package hedl-core

# Run with debugger
rust-gdb target/debug/deps/hedl_core-<hash>

(gdb) break hedl_core::parser::parse_node
(gdb) run
(gdb) backtrace
(gdb) print node
```

### Minimal Reproduction

Create smallest failing example:

```rust
#[test]
fn test_minimal_failure() {
    // Start with full failing input
    let full = b"complex: document\nwith: many\nlines: [...]";

    // Binary search to find minimal case
    // Remove half the content, see if still fails
    let half = b"complex: document\nwith: many";

    // Continue until minimal
    let minimal = b"with: many";

    assert!(parse(minimal).is_ok(), "Minimal case should parse");
}
```

## Debugging Checklist

- [ ] Verify input is valid UTF-8
- [ ] Check for consistent indentation (spaces, not tabs)
- [ ] Ensure all references have targets
- [ ] Verify schema columns match data
- [ ] Check for duplicate IDs
- [ ] Test with minimal input
- [ ] Enable detailed error reporting
- [ ] Use tracing for complex issues
- [ ] Check resource limits not exceeded

## Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Invalid UTF-8" | Non-UTF-8 bytes | Convert to UTF-8 or remove invalid bytes |
| "Max depth exceeded" | Too many nested levels | Increase limit or flatten structure |
| "Unexpected token" | Syntax error | Check HEDL syntax guide |
| "Dangling reference" | Reference to non-existent ID | Add target node or fix ID |
| "Schema mismatch" | Wrong number of columns | Match columns to schema |
| "Duplicate ID" | Same ID used twice | Use unique IDs |

## Performance Debugging

If parsing is slow:

```rust
use std::time::Instant;

let start = Instant::now();
let doc = parse(large_input)?;
let duration = start.elapsed();
println!("Parse took: {:?}", duration);

// Profile with flamegraph
// cargo install flamegraph
// cargo flamegraph --bin my_bin
```

## Verification

To confirm the fix works:

```bash
# Run specific test
cargo test -p hedl-core test_that_was_failing

# Run all parser tests
cargo test -p hedl-core parser::tests

# Test with your actual file
cargo run --bin hedl parse your_file.hedl
```

## Troubleshooting

**Problem**: Still can't find the issue

**Solution**: Ask for help with a minimal example:

```rust
// Post this to GitHub Discussions
#[test]
fn test_mysterious_failure() {
    let input = b"..."; // Minimal failing case
    let result = parse(input);
    assert!(result.is_ok(), "This should parse but doesn't: {:?}", result);
}
```

## Related

- [Parser Architecture](../concepts/parser-architecture.md)
- [Concepts Index](../concepts/README.md)
- API docs: `cargo doc --package hedl-core --open`
