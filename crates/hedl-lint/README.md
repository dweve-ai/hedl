# hedl-lint

**Production-grade linting for HEDL documents -catch errors, enforce best practices, and improve code quality before deployment.**

Valid syntax isn't enough. Unused schemas clutter headers. Empty lists waste space. Unqualified references in key-value contexts lose type information. ID fields should follow conventions. Code reviews catch these issues too late. Automated linting enforces standards consistently across teams and prevents common mistakes from reaching production.

`hedl-lint` provides comprehensive linting with 4 configurable rules covering naming conventions, schema usage, reference quality, and structural best practices. Integrates seamlessly with `hedl-cli`, LSP, and CI/CD pipelines. Configurable severity levels (Hint/Warning/Error) with per-rule error escalation. Custom rule support via trait system. Security-hardened with recursion and diagnostic limits.

## What's Implemented

Comprehensive linting with configuration and security:

1. **4 Lint Rules**: ID naming, unused schemas, empty lists, unqualified references
2. **Three Severity Levels**: Hint (informational), Warning (should fix), Error (must fix)
3. **Configurable Rules**: Enable/disable individual rules, escalate to error via config
4. **Rule-Level Escalation**: Promote individual rule diagnostics to error via config
5. **Custom Rules**: LintRule trait for user-defined lint checks
6. **Security Limits**: Max recursion depth (1000), max diagnostics (10,000)
7. **Line Number Tracking**: Diagnostics support optional line numbers
8. **CLI Integration**: Used by `hedl lint` command with multiple output formats
9. **IDE Integration**: Powers diagnostics in hedl-lsp for real-time feedback
10. **Performance**: O(n) single pass through document, minimal overhead

## Installation

```toml
[dependencies]
hedl-lint = "2.0"
```

## Basic Usage

### Lint with Default Rules

```rust
use hedl_core::parse;
use hedl_lint::lint;

let doc = parse(br#"
%V:2.0
%S:User:[id, name, email]
%S:Product:[id, title, price]
---
users: @User
 | alice, Alice, alice@example.com
 | bob, Bob, bob@example.com
products: @Item
 | item1, Widget, 9.99
"#)?;

let diagnostics = lint(&doc);

for diag in &diagnostics {
    println!("{}", diag);
}
```

**Output**:
```
[unused-schema] warning: Schema 'Product' is defined but never used
```

### Custom Configuration

```rust
use hedl_lint::{lint_with_config, LintConfig, Severity};

let mut config = LintConfig::default();
config.disable_rule("id-naming");
config.set_rule_error("unused-schema");
config.min_severity = Severity::Warning;

let diagnostics = lint_with_config(&doc, config);
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
let mut config = LintConfig::default();
// Enable by default, or disable:
config.disable_rule("id-naming");
```

### 2. unused-schema (Warning, enabled)

Detects %STRUCT definitions that are never referenced:

```hedl
%V:2.0
%S:User:[id, name, email]
%S:Product:[id, title, price]
%S:Order:[id, customer, total]
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
let mut config = LintConfig::default();
// Escalate to error for strict validation:
config.set_rule_error("unused-schema");
```

### 3. empty-list (Hint, enabled)

Flags matrix lists with schema but zero rows:

```hedl
%V:2.0
%S:User:[id, name, age]
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
let mut config = LintConfig::default();
// Escalate to error for strict validation:
config.set_rule_error("empty-list");
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
let mut config = LintConfig::default();
// Enabled by default with Warning severity
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

### LintConfig

```rust
use hedl_lint::{LintConfig, Severity};

let mut config = LintConfig::default();

// Enable/disable individual rules
config.enable_rule("id-naming");
config.disable_rule("empty-list");

// Escalate specific rules to error
config.set_rule_error("unused-schema");

// Set minimum severity to report
config.min_severity = Severity::Warning;

// Set maximum diagnostics to collect (default: 10,000)
config.max_diagnostics = 5000;
```

### RuleConfig

Rule configuration is stored in a HashMap and has two fields:
- `enabled: bool` - Whether the rule is active
- `error: bool` - Whether to escalate all diagnostics from this rule to Error severity

**Example**:
```rust
use hedl_lint::rules::RuleConfig;

let rule_config = RuleConfig {
    enabled: true,
    error: true,  // Escalate this rule's diagnostics to Error
};
```

### Per-Rule Escalation

The `set_rule_error` method enables a rule and escalates both Hint and Warning diagnostics to Error:

```rust
let mut config = LintConfig::default();
config.set_rule_error("unused-schema");

// Now all unused-schema diagnostics become errors (fail fast)
```

## Custom Rules

Implement the `LintRule` trait for custom checks:

```rust
use hedl_lint::{LintRule, Diagnostic, DiagnosticKind};
use hedl_core::Document;

pub struct RequireDescriptionRule;

impl LintRule for RequireDescriptionRule {
    fn id(&self) -> &str {
        "require-description"
    }

    fn description(&self) -> &str {
        "Check that document has a description field"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check if document has 'description' field
        if !doc.root.contains_key("description") {
            diagnostics.push(
                Diagnostic::warning(
                    DiagnosticKind::Custom("require-description".to_string()),
                    "Document should have a 'description' field",
                    "require-description"
                ).with_line(1)
            );
        }

        diagnostics
    }
}

// Use custom rule
use hedl_lint::{LintRunner, LintConfig};

let mut runner = LintRunner::new(LintConfig::default());
runner.add_rule(Box::new(RequireDescriptionRule));

let diagnostics = runner.run(&doc);
```

### LintRule Trait

```rust
pub trait LintRule: Send + Sync {
    /// Unique rule identifier (kebab-case)
    fn id(&self) -> &str;

    /// Rule description
    fn description(&self) -> &str;

    /// Check document and return diagnostics
    fn check(&self, doc: &Document) -> Vec<Diagnostic>;
}
```

## Security Limits

### Recursion Depth Limit

Protection against deeply nested structures:

```rust
const MAX_RECURSION_DEPTH: usize = 1000;

// Linting document > 1000 levels deep:
// Warning diagnostic added, traversal stops at that depth
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
// Warning diagnostic added, further diagnostics suppressed
```

**Prevents**:
- Memory exhaustion from severely malformed input
- Unbounded diagnostic generation
- DoS attacks via pathological documents

## Diagnostic Structure

The `lint` and `lint_with_config` functions return `Vec<Diagnostic>` directly (no Result wrapper), as linting does not fail on malformed documents:

```rust
use hedl_lint::lint;

let diagnostics = lint(&doc);

for diag in &diagnostics {
    println!("{}", diag);
}
```

### Diagnostic Fields

Diagnostics are accessed through getter methods:

```rust
let diag = &diagnostics[0];
println!("Severity: {:?}", diag.severity());
println!("Kind: {:?}", diag.kind());
println!("Message: {}", diag.message());
println!("Rule ID: {}", diag.rule_id());
if let Some(line) = diag.line() {
    println!("Line: {}", line);
}
if let Some(suggestion) = diag.suggestion() {
    println!("Suggestion: {}", suggestion);
}
```

### DiagnosticKind Enum

The `DiagnosticKind` enum identifies the type of lint issue:

```rust
pub enum DiagnosticKind {
    /// ID naming convention violation
    IdNaming,
    /// Unused schema definition
    UnusedSchema,
    /// Empty matrix list
    EmptyList,
    /// Unqualified reference in Key-Value context
    UnqualifiedKvReference,
    /// Custom rule violation
    Custom(String),
}
```

**Example**:
```rust
use hedl_lint::{Diagnostic, DiagnosticKind, Severity};

let diag = Diagnostic::warning(
    DiagnosticKind::UnusedSchema,
    "Schema 'Product' defined but never used",
    "unused-schema"
).with_line(5);

println!("{}", diag);
// Output: line 5: [unused-schema] warning: Schema 'Product' defined but never used
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

- `hedl-core` 2.0 - Core HEDL data structures and parsing
- `thiserror` 1.0 - Error type definitions

## License

Apache-2.0
