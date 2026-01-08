# Contributing to HEDL

Guide for contributing to the HEDL project. We welcome all contributions!

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Workflow](#development-workflow)
3. [Code Standards](#code-standards)
4. [Testing Requirements](#testing-requirements)
5. [Documentation](#documentation)
6. [Pull Request Process](#pull-request-process)
7. [Commit Guidelines](#commit-guidelines)
8. [Review Process](#review-process)
9. [Community Guidelines](#community-guidelines)

---

## Getting Started

### Prerequisites

Before contributing, ensure you have:

1. **Rust toolchain** (1.70+)
   ```bash
   rustup update
   ```

2. **Development environment** set up
   - See [Getting Started Guide](getting-started.md)

3. **Repository forked and cloned**
   ```bash
   git clone https://github.com/YOUR-USERNAME/hedl.git
   cd hedl
   git remote add upstream https://github.com/dweve-ai/hedl.git
   ```

### Finding Work

**Good First Issues**:
- Look for issues tagged `good first issue`
- Check `help wanted` label for needed contributions
- Browse documentation gaps and TODOs

**Areas of Contribution**:
- **Core features**: Parser improvements, new directives
- **Format converters**: New format support, optimization
- **Documentation**: Examples, guides, tutorials
- **Testing**: Test coverage, edge cases, fuzz tests
- **Performance**: Optimization, benchmarks
- **Tooling**: IDE support, CLI enhancements

### Asking Questions

- **Discussions**: Use [GitHub Discussions](https://github.com/dweve-ai/hedl/discussions) for questions
- **Issues**: For bug reports and feature requests
- **Discord**: Real-time chat (if available)

---

## Development Workflow

### 1. Create a Branch

```bash
# Update main
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/bug-description
```

**Branch Naming**:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation
- `perf/` - Performance improvements
- `refactor/` - Code refactoring
- `test/` - Test additions

### 2. Make Changes

**Follow TDD**:
```bash
# 1. Write failing test
cargo test test_new_feature  # Should fail

# 2. Implement feature
# Edit code...

# 3. Make test pass
cargo test test_new_feature  # Should pass

# 4. Refactor
# Clean up code...

# 5. Verify
cargo test --all
```

**Incremental Commits**:
```bash
# Commit logical units
git add src/parser.rs tests/parser_tests.rs
git commit -m "feat: add support for new directive"

# Continue working
git add src/validator.rs tests/validator_tests.rs
git commit -m "feat: validate new directive"
```

### 3. Keep Updated

```bash
# Regularly sync with upstream
git fetch upstream
git rebase upstream/main

# Resolve conflicts if needed
# Edit conflicted files
git add .
git rebase --continue
```

### 4. Run Checks

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all -- -D warnings

# Run tests
cargo test --all

# Run benchmarks (if relevant)
cargo bench --all

# Check documentation
cargo doc --all --no-deps
```

### 5. Submit Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create PR on GitHub
# Fill out PR template
# Link related issues
```

---

## Code Standards

### Rust Style

**Follow Rust conventions**:
```rust
// Good: Snake case for functions and variables
fn parse_value(input: &str) -> Value { }
let user_name = "Alice";

// Good: Pascal case for types
struct ParseOptions { }
enum ValueType { }

// Good: SCREAMING_SNAKE_CASE for constants
const MAX_DEPTH: usize = 100;

// Good: Documentation comments
/// Parses a HEDL value from text.
///
/// # Arguments
///
/// * `input` - The text to parse
///
/// # Errors
///
/// Returns error if input is invalid
pub fn parse_value(input: &str) -> Result<Value> {
    // ...
}
```

### Code Organization

```rust
// Module structure
mod parser {
    // Private implementation
    fn parse_internal(input: &str) -> Result<Value> { }

    // Public API
    pub fn parse(input: &str) -> Result<Value> {
        parse_internal(input)
    }
}

// Clear imports
use std::collections::HashMap;
use std::io::Read;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::document::Document;
use crate::error::HedlError;
```

### Error Handling

```rust
// Use Result for fallible operations
pub fn parse(input: &str) -> Result<Document, HedlError> {
    // Explicit error handling
    let value = parse_value(input)
        .map_err(|e| HedlError::syntax(format!("Invalid value: {}", e), 1))?;

    Ok(Document { /* ... */ })
}

// Use custom error types
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid syntax at line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Performance Considerations

```rust
// Prefer borrowing over cloning
fn process(input: &str) -> Result<String> {  // Borrow
    // ...
}

// Pre-allocate when size is known
let mut result = Vec::with_capacity(items.len());

// Use iterators over loops
items.iter()
    .filter(|item| item.is_valid())
    .map(|item| process(item))
    .collect()

// Avoid unnecessary allocations
fn extract_key(line: &str) -> &str {  // Return slice, not String
    line.split(':').next().unwrap()
}
```

### Safety

```rust
// Document unsafe code
/// SAFETY: input must be valid UTF-8 and aligned
unsafe fn parse_unchecked(input: *const u8, len: usize) -> &str {
    std::str::from_utf8_unchecked(std::slice::from_raw_parts(input, len))
}

// Prefer safe alternatives
fn parse_safe(input: &[u8]) -> Result<&str> {
    std::str::from_utf8(input)
        .map_err(|e| ParseError::InvalidUtf8(e))
}
```

---

## Testing Requirements

### Test Coverage

**All new code must have tests**:
- Unit tests for functions
- Integration tests for features
- Property tests for invariants
- Edge case tests
- Error path tests

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_value() {
        // Arrange
        let input = "42";

        // Act
        let result = parse_value(input);

        // Assert
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_parse_invalid_value() {
        let result = parse_value("@");
        assert!(result.is_err());
    }

    // Property-based tests
    proptest! {
        #[test]
        fn test_parse_never_panics(s in ".*") {
            let _ = parse_value(&s);  // Should not panic
        }
    }
}
```

### Running Tests

```bash
# All tests
cargo test --all

# With output
cargo test -- --nocapture

# Specific test
cargo test test_parse_simple

# Coverage
cargo tarpaulin --all --out Html
```

---

## Documentation

### Code Documentation

**Every public item must be documented**:

```rust
/// Parses a HEDL document from UTF-8 bytes.
///
/// This function performs a complete parse of the input,
/// including header directives, body parsing, and reference
/// resolution.
///
/// # Arguments
///
/// * `input` - The HEDL document as UTF-8 bytes
///
/// # Returns
///
/// Returns the parsed `Document` on success.
///
/// # Errors
///
/// Returns `HedlError` if:
/// - Input is not valid UTF-8
/// - Syntax is invalid
/// - Indentation is incorrect
/// - References are unresolved (in strict mode)
/// - Resource limits are exceeded
///
/// # Examples
///
/// ```
/// use hedl_core::parse;
///
/// let input = b"%VERSION: 1.0\n---\nname: Alice\nage: 30";
/// let doc = parse(input).unwrap();
/// assert_eq!(doc.root.len(), 2);
/// ```
///
/// # Panics
///
/// This function does not panic under normal circumstances.
///
/// # Performance
///
/// Time complexity: O(n) where n is input length
/// Space complexity: O(n) for AST storage
pub fn parse(input: &[u8]) -> Result<Document, HedlError> {
    // Implementation
}
```

### Module Documentation

```rust
//! Parser module for HEDL documents.
//!
//! This module provides the core parsing functionality,
//! converting HEDL text into an abstract syntax tree (AST).
//!
//! # Examples
//!
//! ```
//! use hedl_core::parse;
//!
//! let input = b"%VERSION: 1.0\n---\nname: Alice";
//! let doc = parse(input).unwrap();
//! assert_eq!(doc.version, (1, 0));
//! ```
//!
//! # Architecture
//!
//! The parser uses a multi-stage pipeline:
//! 1. UTF-8 validation and line splitting
//! 2. Header parsing (directives, schemas)
//! 3. Body parsing (items, values)
//! 4. Reference collection and resolution
//! 5. Validation
```

### User Documentation

For user-facing features, add documentation:

```markdown
## New Feature: JSONPath Support

HEDL now supports JSONPath queries for extracting specific data from documents.

### Usage

Use the `hedl-json` crate to perform queries:

```rust
use hedl_json::jsonpath::query;

let results = query(&doc, "$.users[*].name", &config)?;
```
```

---

## Pull Request Process

### PR Checklist

Before submitting:

- [ ] Code follows style guidelines
- [ ] All tests pass (`cargo test --all`)
- [ ] New tests added for changes
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy --all`)
- [ ] Documentation updated
- [ ] Changelog updated (if applicable)
- [ ] Benchmarks run (for performance changes)

### PR Template

```markdown
## Description

Brief description of changes.

## Motivation

Why is this change needed?

## Changes

- Detailed list of changes
- Breaking changes highlighted

## Testing

How was this tested?

## Checklist

- [ ] Tests pass
- [ ] Documentation updated
- [ ] No breaking changes (or documented)

## Related Issues

Fixes #123
Related to #456
```

### PR Size

**Keep PRs focused and reviewable**:
- Small PRs (< 500 lines) preferred
- Single concern per PR
- Split large features into multiple PRs
- Use draft PRs for work-in-progress

---

## Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting (no code change)
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Test additions
- `chore`: Build/tooling changes

**Examples**:

```
feat(parser): add support for custom directives

Implement plugin system for user-defined directives.
Directives can be registered with custom handlers.

Breaking change: ParseOptions API changed
```

```
fix(json): handle null values in arrays

Previously, null values in arrays caused panic.
Now correctly serialized as JSON null.

Fixes #123
```

```
perf(lexer): optimize whitespace scanning with SIMD

Use SIMD instructions for 2x faster whitespace detection
on x86_64 platforms.

Benchmark results:
- Before: 1.2 ms
- After: 0.6 ms
```

### Atomic Commits

**Each commit should be self-contained**:

```bash
# Good: Logical units
git commit -m "feat: add parser for new directive"
git commit -m "test: add tests for new directive"
git commit -m "docs: document new directive"

# Bad: Everything at once
git commit -m "Add feature with tests and docs"
```

---

## Review Process

### For Contributors

**Responding to feedback**:
- Address all comments
- Ask for clarification if needed
- Update code and push changes
- Mark conversations as resolved
- Be patient and respectful

**Making changes**:
```bash
# Make requested changes
# Edit files...

# Amend if fixing review comments
git add .
git commit --amend
git push --force-with-lease

# Or add new commit if substantial changes
git commit -m "Address review comments"
git push
```

### For Reviewers

**Review checklist**:
- [ ] Code is correct
- [ ] Tests are comprehensive
- [ ] Documentation is clear
- [ ] Style is consistent
- [ ] No security issues
- [ ] Performance is acceptable

**Review comments**:
```markdown
# Good: Specific, actionable, kind
Consider using `BTreeMap` instead of `HashMap` for
deterministic iteration order. This helps with testing.

# Good: Suggests alternative
This could be simplified using the `?` operator:
\`\`\`rust
let value = parse_value(input)?;
\`\`\`

# Good: Asks questions
Why is this allocation necessary here? Could we use
a string slice instead?

# Bad: Vague
This doesn't look right.

# Bad: Demands without explanation
Change this.
```

---

## Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive environment.

**Expected behavior**:
- Be respectful and inclusive
- Accept constructive criticism
- Focus on what's best for the project
- Show empathy

**Unacceptable behavior**:
- Harassment or discrimination
- Trolling or insulting comments
- Personal attacks
- Unwelcome sexual attention

### Communication

**GitHub Issues**:
- Bug reports: Include reproduction steps
- Feature requests: Explain use case
- Questions: Check existing issues first

**Pull Requests**:
- Describe changes clearly
- Link related issues
- Respond to feedback promptly

**Discussions**:
- Ask questions
- Share ideas
- Help others

---

## Getting Help

### Resources

- **[Getting Started Guide](getting-started.md)**: Setup and basics
- **[Concepts](concepts/README.md)**: Understanding core concepts
- **[How-To Guides](how-to/README.md)**: Practical task guides
- API documentation: `cargo doc --workspace --open`

### Contact

- **GitHub Issues**: Bug reports and features
- **GitHub Discussions**: Questions and ideas
- **Email**: opensource@dweve.com

---

## Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Changelog

Thank you for contributing to HEDL!

---

**Ready to contribute?** Check out [good first issues](https://github.com/dweve-ai/hedl/labels/good%20first%20issue)!
