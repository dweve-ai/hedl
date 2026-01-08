# Contributing to HEDL

Thank you for your interest in contributing to HEDL (Hierarchical Entity Data Language)! We welcome contributions from everyone and appreciate your help in making HEDL better.

This document provides guidelines and information to help you contribute effectively to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style and Standards](#code-style-and-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Commit Message Format](#commit-message-format)
- [Project Structure](#project-structure)
- [Documentation](#documentation)
- [Getting Help](#getting-help)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful, constructive, and professional in all interactions. We expect all contributors to:

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other community members

## Ways to Contribute

There are many ways to contribute to HEDL:

### Report Bugs

Found a bug? Please open an issue on GitHub with:
- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- HEDL version and Rust version
- Sample HEDL code that demonstrates the issue (if applicable)
- Any relevant error messages or stack traces

### Suggest Features

Have an idea for a new feature or enhancement? We'd love to hear it! Open an issue describing:
- The problem your feature would solve
- Your proposed solution
- Any alternative solutions you've considered
- Examples of how the feature would be used

### Improve Documentation

Documentation improvements are always welcome:
- Fix typos, grammar, or unclear explanations
- Add examples to existing documentation
- Write tutorials or guides
- Improve API documentation and code comments
- Translate documentation (if applicable)

### Write Tests

Help improve test coverage:
- Add test cases for existing functionality
- Write edge case tests
- Create property-based tests
- Add performance benchmarks
- Improve test documentation

### Submit Code

Ready to write code? Great! Look for issues labeled:
- `good first issue` - Good for newcomers
- `help wanted` - We need assistance with these
- `bug` - Bug fixes are always appreciated
- `enhancement` - Feature additions and improvements

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/hedl.git
   cd hedl
   ```
3. **Add the upstream repository**:
   ```bash
   git remote add upstream https://github.com/dweve-ai/hedl.git
   ```
4. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

HEDL is a Rust workspace project with multiple crates. Here's how to set up your development environment:

### Prerequisites

- Rust 1.70 or later (install via [rustup](https://rustup.rs/))
- Git
- A code editor (we recommend VS Code with rust-analyzer)

### Building the Project

Build all workspace crates:
```bash
cargo build
```

Build in release mode for performance testing:
```bash
cargo build --release
```

Build a specific crate:
```bash
cargo build -p hedl-core
```

### Running Tests

Run all tests:
```bash
cargo test
```

Run tests for a specific crate:
```bash
cargo test -p hedl-lex
```

Run tests with output:
```bash
cargo test -- --nocapture
```

Run integration tests:
```bash
cargo test --test '*'
```

Run conformance tests:
```bash
cargo test --test conformance
```

### Running Examples

The CLI provides various commands for working with HEDL:
```bash
# Validate a HEDL file
cargo run --bin hedl -- validate examples/users.hedl

# Convert HEDL to JSON
cargo run --bin hedl -- to-json examples/users.hedl

# Lint a HEDL file
cargo run --bin hedl -- lint examples/users.hedl

# Format a HEDL file
cargo run --bin hedl -- format examples/users.hedl
```

## Code Style and Standards

We follow standard Rust conventions and use automated tools to ensure consistency:

### Code Formatting

All code must be formatted with `rustfmt`:
```bash
cargo fmt
```

Check formatting without modifying files:
```bash
cargo fmt -- --check
```

### Linting

All code must pass `clippy` checks:
```bash
cargo clippy -- -D warnings
```

Run clippy on all workspace crates:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Code Quality Standards

- **Follow Rust idioms**: Use idiomatic Rust patterns and conventions
- **Type safety**: Leverage Rust's type system for correctness
- **Error handling**: Use `Result` and `?` operator; avoid panics in library code
- **Documentation**: Document all public APIs with doc comments
- **DRY principle**: Don't Repeat Yourself - extract common logic
- **Single Responsibility**: Each function/module should have one clear purpose
- **Performance**: Consider performance implications, especially in hot paths
- **Safety**: Avoid `unsafe` unless absolutely necessary (and document why)

## Testing Requirements

All contributions must include appropriate tests. Here are our testing standards:

### Unit Tests

- **Required**: Every public function must have unit tests
- **Coverage**: Test happy paths, edge cases, and error conditions
- **Location**: Place tests in the same file as the code using `#[cfg(test)]`
- **Naming**: Use descriptive test names that explain what is being tested

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_key_value_succeeds() {
        let input = "name: John";
        let result = parse_key_value(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_empty_input_returns_error() {
        let input = "";
        let result = parse_key_value(input);
        assert!(result.is_err());
    }
}
```

### Integration Tests

- Add integration tests in the `tests/` directory
- Test interactions between multiple crates
- Test end-to-end workflows
- Validate against the HEDL specification

### Test Execution Requirements

Before submitting a PR:
- **All tests must pass**: `cargo test` must succeed
- **No test warnings**: Fix any test-related warnings
- **Add tests for new features**: New functionality must include tests
- **Add tests for bug fixes**: Include regression tests
- **Performance tests**: Add benchmarks for performance-critical code

### Property-Based Testing

Consider using property-based testing for:
- Parser invariants
- Serialization round-trip properties
- Canonicalization determinism

### Conformance Tests

If your changes affect parsing or output:
- Add conformance tests to `tests/conformance/`
- Ensure all existing conformance tests still pass
- Update conformance documentation if needed

## Pull Request Process

### Before Submitting

1. **Sync with upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/master
   ```

2. **Run all checks**:
   ```bash
   cargo fmt -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   cargo test --all-features
   ```

3. **Update documentation**:
   - Update relevant README files
   - Update API documentation
   - Add examples if appropriate

4. **Commit your changes** following our [commit message format](#commit-message-format)

### Submitting the PR

1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Open a Pull Request** on GitHub with:
   - Clear title describing the change
   - Description of what changed and why
   - Reference to any related issues (e.g., "Fixes #123")
   - Screenshots or examples (if applicable)
   - Confirmation that tests pass

3. **Respond to feedback**:
   - Address reviewer comments promptly
   - Push additional commits to your branch
   - Mark conversations as resolved when addressed

### PR Review Process

- A maintainer will review your PR
- Reviews may request changes or ask questions
- Once approved, a maintainer will merge your PR
- Your contribution will be included in the next release

### PR Requirements

- All CI checks must pass
- At least one maintainer approval required
- No merge conflicts with the main branch
- Code follows style guidelines
- Tests included and passing
- Documentation updated

## Commit Message Format

We follow conventional commit format for clear, searchable history:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic changes)
- `refactor`: Code refactoring (no feature changes)
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks, dependency updates
- `ci`: CI/CD changes

### Scope

The crate or component affected (optional but recommended):
- `core`: hedl-core
- `lex`: hedl-lex
- `json`: hedl-json
- `cli`: hedl-cli
- `test`: testing infrastructure
- etc.

### Examples

```
feat(json): add support for custom serialization options

Implements configurable JSON output formatting with options for
indentation, field ordering, and null handling.

Closes #456
```

```
fix(lex): handle Unicode whitespace in string literals

Previously, non-ASCII whitespace characters were incorrectly
treated as string content. Now properly handles all Unicode
whitespace categories.

Fixes #789
```

```
docs(readme): add examples for matrix list syntax

Added comprehensive examples demonstrating matrix list usage
with various data types and nested structures.
```

```
test(parser): add edge case tests for nested references

Adds property-based tests for deeply nested reference chains
and circular reference detection.
```

### Commit Best Practices

- Use imperative mood ("add feature" not "added feature")
- Keep subject line under 72 characters
- Provide context in the body for non-trivial changes
- Reference issues and PRs when relevant
- Make atomic commits (one logical change per commit)

## Project Structure

HEDL is organized as a Cargo workspace with multiple crates:

```
hedl/
├── crates/
│   ├── hedl/           # Main library (re-exports all functionality)
│   ├── hedl-core/      # Core types and data model
│   ├── hedl-lex/       # Lexical analysis and tokenization
│   ├── hedl-row/       # Matrix row parsing
│   ├── hedl-tensor/    # Tensor literal parsing
│   ├── hedl-c14n/      # Canonicalization
│   ├── hedl-json/      # JSON conversion
│   ├── hedl-yaml/      # YAML conversion
│   ├── hedl-xml/       # XML conversion
│   ├── hedl-csv/       # CSV conversion
│   ├── hedl-parquet/   # Parquet conversion
│   ├── hedl-neo4j/     # Neo4j integration
│   ├── hedl-lint/      # Linting and diagnostics
│   ├── hedl-cli/       # Command-line interface
│   ├── hedl-ffi/       # C FFI bindings
│   └── hedl-test/      # Testing utilities
├── tests/              # Integration and conformance tests
├── bindings/           # Language bindings (Python, etc.)
├── docs/               # Documentation
└── examples/           # Example HEDL files
```

### Key Crates

- **hedl-core**: Core data structures (`Document`, `Object`, `Value`, etc.)
- **hedl-lex**: Lexer and parser implementation
- **hedl-c14n**: Canonical form generation
- **hedl-***: Format converters for various data formats

## Documentation

Good documentation is essential. Here's what we need:

### Code Documentation

- **Public APIs**: All public items must have doc comments
- **Examples**: Include usage examples in doc comments
- **Errors**: Document error conditions and return types
- **Safety**: Document any unsafe code and invariants
- **Complexity**: Note algorithmic complexity for non-trivial functions

Example:
```rust
/// Parses a HEDL document from a string.
///
/// # Arguments
///
/// * `input` - The HEDL document as a string slice
///
/// # Returns
///
/// Returns `Ok(Document)` on success or `Err(ParseError)` if parsing fails.
///
/// # Errors
///
/// Returns `ParseError` if:
/// - The document has invalid syntax
/// - Required directives are missing
/// - References cannot be resolved
///
/// # Examples
///
/// ```
/// use hedl::parse;
///
/// let doc = parse("%VERSION: 1.0\n---\nname: John")?;
/// assert_eq!(doc.version(), "1.0");
/// ```
pub fn parse(input: &str) -> Result<Document, ParseError> {
    // Implementation
}
```

### User Documentation

When adding features, update:
- The main README if it affects getting started
- Relevant guides in `docs/guides/`
- API documentation in `docs/api/`
- The specification (SPEC.md) if it affects the format

### Documentation Testing

Ensure documentation examples compile and run:
```bash
cargo test --doc
```

## Getting Help

We're here to help! Here are the best ways to get assistance:

### Questions and Discussions

- **GitHub Discussions**: For questions, ideas, and general discussion
- **GitHub Issues**: For bug reports and feature requests
- **Documentation**: Check `docs/` directory for guides and references

### Communication Channels

- **Issue Tracker**: https://github.com/dweve-ai/hedl/issues
- **Pull Requests**: https://github.com/dweve-ai/hedl/pulls
- **Specification**: See SPEC.md for format details

### Maintainers

The HEDL project is maintained by the Dweve AI team. Feel free to tag maintainers in issues or PRs when you need guidance.

### Response Times

We aim to:
- Acknowledge issues within 2-3 business days
- Review PRs within 1 week
- Provide meaningful feedback on all contributions

Please be patient - we're a small team and will respond as quickly as we can!

## Recognition

All contributors will be recognized:
- Listed in release notes for their contributions
- Added to contributors list
- Credited in relevant documentation

Thank you for contributing to HEDL! Your efforts help make data serialization more efficient and accessible for everyone.

---

**Happy Contributing!**

For more information, see:
- [HEDL Specification](SPEC.md)
- [Project Documentation](docs/)
- [Issue Tracker](https://github.com/dweve-ai/hedl/issues)
