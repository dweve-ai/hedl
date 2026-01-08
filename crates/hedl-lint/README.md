# hedl-lint

Linting and best practices validation for HEDL documents.

## Installation

```toml
[dependencies]
hedl-lint = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_lint::{lint, LintConfig, Severity};

let doc = parse(hedl.as_bytes())?;
let diagnostics = lint(&doc)?;

for diag in diagnostics {
    match diag.severity {
        Severity::Error => eprintln!("Error: {}", diag.message),
        Severity::Warning => eprintln!("Warning: {}", diag.message),
        Severity::Info => println!("Info: {}", diag.message),
    }
}
```

## Rules

- **Naming conventions** - Check key and type naming
- **Unused references** - Detect dangling references
- **Schema consistency** - Validate matrix schemas
- **Best practices** - Suggest improvements

## CLI Integration

Used by `hedl lint` command:

```bash
hedl lint document.hedl
hedl lint --format json document.hedl
```

## License

Apache-2.0
