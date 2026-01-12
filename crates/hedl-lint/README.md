# hedl-lint

**Production-grade linting for HEDL documents—catch errors, enforce best practices, and improve code quality before deployment.**

Valid syntax isn't enough. Unused schemas clutter headers. Empty lists waste space. Unqualified references in key-value contexts lose type information. ID fields should follow conventions. Code reviews catch these issues too late. Automated linting enforces standards consistently across teams and prevents common mistakes from reaching production.

`hedl-lint` provides comprehensive linting with 5 configurable rules covering naming conventions, schema usage, reference quality, and structural best practices. Integrates seamlessly with `hedl-cli`, LSP, and CI/CD pipelines. Configurable severity levels (Hint/Warning/Error) with rule-specific enable/disable. Custom rule support via trait system. Security-hardened with recursion and diagnostic limits.

## What's Implemented

Comprehensive linting with configuration and security:

1. **5 Lint Rules**: ID naming, unused schemas, empty lists, unqualified references, unused aliases
2. **Three Severity Levels**: Hint (informational), Warning (should fix), Error (must fix)
3. **Configurable Rules**: Enable/disable individual rules, set severity per rule
4. **Severity Escalation**: Promote hints to warnings, warnings to errors via config
5. **Custom Rules**: LintRule trait for user-defined lint checks
6. **Security Limits**: Max recursion depth (1000), max diagnostics (10,000)
7. **Line Number Tracking**: Every diagnostic includes source line number
8. **CLI Integration**: Used by `hedl lint` command with multiple output formats
9. **IDE Integration**: Powers diagnostics in hedl-lsp for real-time feedback
10. **Performance**: O(n) single pass through document, minimal overhead

## Installation

```toml
[dependencies]
hedl-lint = "1.0"
```

## Basic Usage

### Lint with Default Rules

```rust
use hedl_core::parse;
use hedl_lint::lint;

let doc = parse(br#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
%STRUCT: Product: [id, title, price]
---
users: @User
  | alice, Alice, alice@example.com
  | bob, Bob, bob@example.com
products: @Item
  | item1, Widget, 9.99
"#)?;

let diagnostics = lint(&doc)?;

for diag in diagnostics {
    println!("[{}] Line {}: {}",
        diag.severity, diag.line, diag.message);
}
```

**Output**:
```
[Warning] Line 3: Unused schema definition: 'Product' is defined but never used
[Hint] Line 8: Consider using qualified reference '@Item:item1' instead of unqualified 'item1' for better type safety
```

### Custom Configuration

```rust
use hedl_lint::{lint_with_config, LintConfig, RuleConfig, Severity};

let config = LintConfig::builder()
    .rule("id-naming", RuleConfig::enabled(Severity::Hint))
    .rule("unused-schema", RuleConfig::enabled(Severity::Warning))
    .rule("empty-list", RuleConfig::enabled(Severity::Hint))
    .rule("unqualified-kv-ref", RuleConfig::enabled(Severity::Warning))
    .rule("unused-alias", RuleConfig::disabled())  // Disabled by default
    .escalate_hints_to_warnings(false)
    .escalate_warnings_to_errors(false)
    .build();

let diagnostics = lint_with_config(&doc, &config)?;
```

## Lint Rules

### 1. id-naming (Hint, enabled)

Checks ID field naming conventions for consistency:

```hedl
# Good: kebab-case IDs
users: @User[id, name]
  | alice-smith, Alice Smith
  | bob-jones, Bob Jones

# Warning: inconsistent casing
users: @User[id, name]
  | AliceSmith, Alice Smith    # Hint: prefer kebab-case
  | bob_jones, Bob Jones        # Hint: prefer kebab-case
```

**Checks**:
- Consistent naming style (kebab-case, snake_case, camelCase)
- No mixed naming styles within document
- Descriptive IDs (warns on generic like "item1", "id1")

**Severity**: Hint (informational, doesn't fail builds)

**Configuration**:
```rust
.rule("id-naming", RuleConfig::enabled(Severity::Hint))
// Or disable entirely:
.rule("id-naming", RuleConfig::disabled())
```

### 2. unused-schema (Warning, enabled)

Detects %STRUCT definitions that are never referenced:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
%STRUCT: Product: [id, title, price]
%STRUCT: Order: [id, customer, total]
---
users: @User
  | alice, Alice, alice@example.com

# Warning: Product and Order schemas defined but unused
```

**Impact**:
- Clutters header with unnecessary declarations
- Confuses readers about expected document structure
- Increases file size and parsing overhead

**Severity**: Warning (should be fixed)

**Fix**: Remove unused %STRUCT declarations or add corresponding entity lists

**Configuration**:
```rust
.rule("unused-schema", RuleConfig::enabled(Severity::Warning))
// Escalate to error for strict validation:
.rule("unused-schema", RuleConfig::enabled(Severity::Error))
```

### 3. empty-list (Hint, enabled)

Flags matrix lists with schema but zero rows:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User[0]
  # No rows - empty list

# Hint: Empty list 'users' defined but contains no data
```

**When This Occurs**:
- Placeholder lists during development
- Filtered results with no matches
- Template documents

**Why It Matters**:
- Wastes bytes in production data
- May indicate incomplete data export
- Confuses document purpose

**Severity**: Hint (may be intentional)

**Configuration**:
```rust
.rule("empty-list", RuleConfig::enabled(Severity::Hint))
// Escalate to warning for production:
.rule("empty-list", RuleConfig::enabled(Severity::Warning))
```

### 4. unqualified-kv-ref (Warning, enabled)

Warns about unqualified references in key-value context:

```hedl
# Bad: unqualified reference loses type information
config:
  admin: @alice              # Warning: prefer @User:alice

# Good: qualified reference preserves type
config:
  admin: @User:alice         # Type information explicit
```

**Why Qualified is Better**:
- Explicit type information aids tooling (LSP, validation)
- Self-documenting references
- Enables static analysis
- Prevents ambiguity when multiple entity types have same ID

**Severity**: Warning (impacts code quality)

**Configuration**:
```rust
.rule("unqualified-kv-ref", RuleConfig::enabled(Severity::Warning))
```

### 5. unused-alias (Warning, disabled by default)

Detects %ALIAS definitions that are never used:

```hedl
%VERSION: 1.0
%ALIAS: api_url: https://api.example.com
%ALIAS: db_host: localhost
---
config:
  endpoint: $api_url       # api_url used
  # db_host never used

# Warning: Alias 'db_host' defined but never referenced
```

**Why Disabled By Default**:
- Aliases may be used in external templates
- Configuration files often have optional aliases
- Less critical than unused schemas

**Enable When**: Strict validation desired, no external template system

**Configuration**:
```rust
.rule("unused-alias", RuleConfig::enabled(Severity::Warning))
```

## Severity Levels

### Hint

**Purpose**: Informational suggestions for improvements

**Examples**:
- ID naming conventions
- Empty lists (may be intentional)
- Stylistic preferences

**Behavior**:
- Does NOT fail CI/CD builds
- Shown in IDE with blue squiggle
- Optional to fix

### Warning

**Purpose**: Should be fixed but not blocking

**Examples**:
- Unused schemas
- Unqualified references in key-value context
- Unused aliases

**Behavior**:
- Fails strict validation modes
- Shown in IDE with yellow squiggle
- Should be addressed before merge

### Error

**Purpose**: Must be fixed, blocks deployment

**Examples**:
- Schema violations (wrong column count)
- Unresolved references (when escalated)
- Critical structural issues

**Behavior**:
- Always fails validation
- Shown in IDE with red squiggle
- Blocks CI/CD pipeline

## Configuration System

### LintConfig Builder

```rust
use hedl_lint::{LintConfig, RuleConfig, Severity};

let config = LintConfig::builder()
    // Enable/disable individual rules
    .rule("id-naming", RuleConfig::enabled(Severity::Hint))
    .rule("unused-schema", RuleConfig::enabled(Severity::Error))  // Escalated
    .rule("empty-list", RuleConfig::disabled())
    .rule("unqualified-kv-ref", RuleConfig::enabled(Severity::Warning))
    .rule("unused-alias", RuleConfig::enabled(Severity::Warning))

    // Global severity escalation
    .escalate_hints_to_warnings(true)   // Promote all hints to warnings
    .escalate_warnings_to_errors(true)  // Promote all warnings to errors

    .build();
```

### RuleConfig Options

**Enabled with Severity**:
```rust
RuleConfig::enabled(Severity::Hint)
RuleConfig::enabled(Severity::Warning)
RuleConfig::enabled(Severity::Error)
```

**Disabled**:
```rust
RuleConfig::disabled()
```

### Severity Escalation

**escalate_hints_to_warnings** (default: false)
- Promotes all Hint diagnostics to Warning
- Useful for stricter CI/CD validation
- Makes informational suggestions more prominent

**escalate_warnings_to_errors** (default: false)
- Promotes all Warning diagnostics to Error
- Useful for zero-warnings policy
- Blocks builds on any quality issues

**Example**:
```rust
let strict_config = LintConfig::builder()
    .escalate_hints_to_warnings(true)
    .escalate_warnings_to_errors(true)
    .build();

// Now ALL diagnostics become errors (fail fast)
```

## Custom Rules

Implement the `LintRule` trait for custom checks:

```rust
use hedl_lint::{LintRule, Diagnostic, Severity};
use hedl_core::Document;

pub struct RequireDescriptionRule;

impl LintRule for RequireDescriptionRule {
    fn name(&self) -> &str {
        "require-description"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check if document has 'description' field
        if !doc.fields.contains_key("description") {
            diagnostics.push(Diagnostic {
                line: 1,
                severity: Severity::Warning,
                rule: self.name().to_string(),
                message: "Document should have a 'description' field".to_string(),
            });
        }

        diagnostics
    }
}

// Use custom rule
use hedl_lint::{lint_with_rules, LintConfig};

let custom_rules: Vec<Box<dyn LintRule>> = vec![
    Box::new(RequireDescriptionRule),
];

let config = LintConfig::default();
let diagnostics = lint_with_rules(&doc, &config, &custom_rules)?;
```

### LintRule Trait

```rust
pub trait LintRule {
    /// Unique rule identifier (kebab-case)
    fn name(&self) -> &str;

    /// Check document and return diagnostics
    fn check(&self, doc: &Document) -> Vec<Diagnostic>;

    /// Optional: default severity (override in config)
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
}
```

## Security Limits

### Recursion Depth Limit

Protection against deeply nested structures:

```rust
const MAX_RECURSION_DEPTH: usize = 1000;

// Linting document > 1000 levels deep:
// Error: LintError::MaxRecursionExceeded { depth: 1001, max: 1000 }
```

**Prevents**:
- Stack overflow from malicious input
- Infinite recursion bugs
- Runaway memory consumption

### Diagnostic Count Limit

Protection against diagnostic explosion:

```rust
const MAX_DIAGNOSTICS: usize = 10_000;

// If linting generates > 10,000 diagnostics:
// Error: LintError::TooManyDiagnostics { count: 10001, max: 10000 }
```

**Prevents**:
- Memory exhaustion from severely malformed input
- Unbounded diagnostic generation
- DoS attacks via pathological documents

## Error Handling

```rust
use hedl_lint::{lint, LintError};

match lint(&doc) {
    Ok(diagnostics) => {
        for diag in diagnostics {
            println!("{:?}", diag);
        }
    }
    Err(LintError::MaxRecursionExceeded { depth, max }) => {
        eprintln!("Document nesting too deep: {} (max: {})", depth, max);
    }
    Err(LintError::TooManyDiagnostics { count, max }) => {
        eprintln!("Too many issues: {} (max: {})", count, max);
    }
    Err(e) => {
        eprintln!("Lint error: {}", e);
    }
}
```

### Error Types

- `MaxRecursionExceeded` - Nesting depth exceeds 1000 levels
- `TooManyDiagnostics` - Generated diagnostics exceed 10,000
- `Io(std::io::Error)` - I/O failures
- `InvalidRule(String)` - Unknown rule name in configuration

## Diagnostic Structure

```rust
pub struct Diagnostic {
    pub line: usize,              // Source line number (1-indexed)
    pub severity: Severity,       // Hint / Warning / Error
    pub rule: String,             // Rule name (e.g., "unused-schema")
    pub message: String,          // Human-readable description
}
```

**Example**:
```rust
Diagnostic {
    line: 5,
    severity: Severity::Warning,
    rule: "unused-schema".to_string(),
    message: "Schema 'Product' defined but never used".to_string(),
}
```

## CLI Integration

The `hedl-cli` crate uses `hedl-lint` for the `lint` command:

```bash
# Lint with default rules
hedl lint document.hedl

# JSON output for tooling
hedl lint --format json document.hedl

# Escalate warnings to errors
hedl lint --strict document.hedl

# Disable specific rules
hedl lint --disable unused-alias document.hedl
```

**Exit Codes**:
- 0: No errors (hints and warnings allowed)
- 1: One or more errors found
- 2: Internal linter error

## IDE Integration

The `hedl-lsp` crate uses `hedl-lint` for real-time diagnostics:

**Features**:
- Live linting as you type (200ms debounce)
- Squiggly underlines (blue/yellow/red for hint/warning/error)
- Hover messages with full diagnostic text
- Quick fixes (where applicable)

**Performance**: Incremental linting on document change, cached between edits.

## Use Cases

**CI/CD Validation**: Run `hedl lint` in CI pipelines to enforce code quality standards. Fail builds on warnings in strict mode.

**Pre-Commit Hooks**: Add linting to git pre-commit hooks to catch issues before they reach code review. Faster feedback loop.

**Code Review Automation**: Reduce human review burden by catching common issues automatically. Let reviewers focus on logic, not style.

**IDE Integration**: Real-time feedback while editing prevents issues from being introduced. Fix problems immediately at creation time.

**Migration Tooling**: Run linting on legacy HEDL to identify quality issues during modernization projects. Generate technical debt inventory.

**Documentation Generation**: Use lint diagnostics to generate quality reports for stakeholders. Track improvement over time with metrics.

## What This Crate Doesn't Do

**Schema Validation**: Linting checks best practices, not structural validity. For schema validation (column count, type checking), use hedl-core's validator.

**Auto-Fixing**: Diagnostics identify problems but don't automatically fix them. For formatting, use `hedl-c14n` or `hedl format`.

**Performance Profiling**: Linting focuses on code quality, not runtime performance. For performance analysis, use hedl-bench.

**Security Scanning**: While security-hardened, this isn't a security scanner. For vulnerability detection in dependencies, use cargo-audit.

## Performance Characteristics

**Time Complexity**: O(n) where n = total entities + fields. Single linear pass through document.

**Space Complexity**: O(d) where d = diagnostic count. Typically <100 diagnostics per document.

**Caching**: Lint results can be cached by document hash for repeated checks (implemented in hedl-lsp).

**Overhead**: Minimal (<5% of parse time). Suitable for interactive use in IDEs.

## Dependencies

- `hedl-core` 1.0 - Core HEDL data structures and parsing
- `thiserror` 1.0 - Error type definitions

## License

Apache-2.0
