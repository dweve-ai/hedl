# Getting Started with HEDL Development

This guide will help you set up your development environment and start contributing to HEDL.

## Prerequisites

### Required Software

1. **Rust** (version 1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```

2. **Git**
   ```bash
   # Verify installation
   git --version
   ```

3. **Cargo Tools** (optional but recommended)
   ```bash
   # Formatting
   rustup component add rustfmt

   # Linting
   rustup component add clippy

   # Code coverage
   cargo install cargo-tarpaulin

   # Benchmarking
   cargo install cargo-criterion
   ```

### Platform-Specific Requirements

#### Linux
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora
sudo dnf install gcc pkg-config openssl-devel
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if needed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Windows
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- Or use [WSL2](https://docs.microsoft.com/en-us/windows/wsl/install) with Linux instructions

## Repository Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR-USERNAME/hedl.git
cd hedl

# Add upstream remote
git remote add upstream https://github.com/dweve-ai/hedl.git

# Verify remotes
git remote -v
```

### 2. Build the Project

```bash
# Build all crates in release mode
cargo build --release --all

# Build specific crate
cargo build -p hedl-core

# Build with all features
cargo build --all-features
```

Expected output:
```
   Compiling hedl-core v1.0.0
   Compiling hedl-c14n v1.0.0
   ...
   Finished release [optimized] target(s) in 2m 34s
```

### 3. Run Tests

```bash
# Run all tests
cargo test --all

# Run tests for specific crate
cargo test -p hedl-core

# Run with verbose output
cargo test --all -- --nocapture

# Run specific test
cargo test test_parse_simple_document
```

### 4. Verify Installation

```bash
# Run the CLI tool
cargo run --bin hedl -- --version

# Validate a sample file
echo '%VERSION: 1.0
---
name: Alice
age: 30' > /tmp/test.hedl
cargo run --bin hedl -- validate /tmp/test.hedl

# Convert to JSON
cargo run --bin hedl -- to-json /tmp/test.hedl --pretty

# Run linter
cargo run --bin hedl -- lint bindings/common/fixtures/sample_basic.hedl
```

## Development Environment

### IDE Setup

#### VS Code (Recommended)

Install these extensions:
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - Rust language support
- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) - Debugging support
- [crates](https://marketplace.visualstudio.com/items?itemName=serayuzgur.crates) - Dependency management
- [Better TOML](https://marketplace.visualstudio.com/items?itemName=bungcip.better-toml) - TOML syntax

Workspace settings (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.allFeatures": true,
  "editor.formatOnSave": true,
  "editor.rulers": [100]
}
```

#### IntelliJ IDEA / CLion

Install the [Rust plugin](https://www.jetbrains.com/rust/):
1. Go to Settings → Plugins
2. Search for "Rust"
3. Install and restart

#### Vim / Neovim

Use [coc-rust-analyzer](https://github.com/fannheyward/coc-rust-analyzer):
```vim
" In your .vimrc / init.vim
Plug 'neoclide/coc.nvim', {'branch': 'release'}
:CocInstall coc-rust-analyzer
```

### Project Structure

```
hedl/
├── Cargo.toml              # Workspace configuration
├── Cargo.lock              # Dependency lock file
├── crates/                 # All crates
│   ├── hedl/               # Main library facade
│   ├── hedl-core/          # Core parser
│   ├── hedl-c14n/          # Canonicalization
│   ├── hedl-json/          # JSON conversion
│   ├── hedl-yaml/          # YAML conversion
│   ├── hedl-xml/           # XML conversion
│   ├── hedl-csv/           # CSV conversion
│   ├── hedl-toon/          # TOON format
│   ├── hedl-parquet/       # Parquet conversion
│   ├── hedl-neo4j/         # Neo4j Cypher
│   ├── hedl-lint/          # Linting
│   ├── hedl-cli/           # CLI tool
│   ├── hedl-ffi/           # C bindings
│   ├── hedl-wasm/          # WASM bindings
│   ├── hedl-lsp/           # Language server
│   ├── hedl-mcp/           # Model Context Protocol
│   ├── hedl-stream/        # Streaming parser
│   ├── hedl-test/          # Test utilities
│   └── hedl-bench/         # Benchmarks
├── docs/                   # Documentation
├── tests/                  # Integration tests
├── bindings/               # Language bindings
│   └── common/fixtures/    # Test fixtures (sample HEDL files)
├── SPEC.md                 # Language specification
├── CONTRIBUTING.md         # Contribution guide
└── README.md               # Project README
```

## Basic Workflows

### Making Changes

1. **Create a branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes**
   - Edit code in `crates/`
   - Add tests in `tests/` or crate-specific test files
   - Update documentation if needed

3. **Format and lint**
   ```bash
   cargo fmt --all
   cargo clippy --all -- -D warnings
   ```

4. **Run tests**
   ```bash
   cargo test --all
   ```

5. **Commit changes**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

6. **Push and create PR**
   ```bash
   git push origin feature/my-feature
   # Then create PR on GitHub
   ```

### Testing Your Changes

```bash
# Unit tests
cargo test -p hedl-core

# Integration tests
cargo test --test integration_tests

# Doc tests
cargo test --doc

# Specific test
cargo test test_name

# With backtrace on failure
RUST_BACKTRACE=1 cargo test

# Show test output
cargo test -- --nocapture
```

### Running Benchmarks

```bash
# All benchmarks
cargo bench --all

# Specific benchmark
cargo bench -p hedl-bench

# Generate reports
cargo bench --bench parsing -- --save-baseline before
# Make changes...
cargo bench --bench parsing -- --baseline before
```

### Building Documentation

```bash
# Build docs for all crates
cargo doc --all --no-deps

# Open in browser
cargo doc --all --no-deps --open

# Include private items
cargo doc --all --no-deps --document-private-items
```

## Common Development Tasks

### Adding a New Feature

1. Identify the appropriate crate(s)
2. Write failing tests first (TDD)
3. Implement the feature
4. Update documentation
5. Add examples if applicable
6. Run full test suite

### Fixing a Bug

1. Write a failing test that reproduces the bug
2. Fix the bug
3. Verify the test passes
4. Add regression test if needed
5. Update changelog

### Optimizing Performance

1. Write a benchmark
2. Profile the code (`cargo flamegraph` or `perf`)
3. Identify bottlenecks
4. Optimize
5. Verify benchmark improvements
6. Ensure tests still pass

### Adding Tests

```rust
// Unit test in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input = "name: Alice";
        let result = parse(input);
        assert!(result.is_ok());
    }
}

// Integration test in tests/ directory
use hedl::{parse, to_json};

#[test]
fn test_round_trip() {
    let hedl = r#"
%VERSION: 1.0
---
name: Alice
"#;
    let doc = parse(hedl).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("Alice"));
}
```

## Debugging

### Using rust-analyzer

Set breakpoints in VS Code and use F5 to debug.

### Using LLDB/GDB

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-lldb target/debug/hedl-cli
# or
rust-gdb target/debug/hedl-cli
```

### Print Debugging

```rust
// Debug print
dbg!(&value);

// Pretty print
println!("{:#?}", document);

// Conditional compilation
#[cfg(debug_assertions)]
println!("Debug: {:?}", value);
```

### Logging

```rust
use tracing::{debug, info, warn, error};

info!("Parsing document");
debug!("Current position: {}", pos);
```

## Performance Profiling

### CPU Profiling

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench parsing

# Open flamegraph.svg in browser
```

### Memory Profiling

```bash
# Using valgrind
cargo build --release
valgrind --tool=massif target/release/hedl-cli parse large-file.hedl

# Using heaptrack (Linux)
heaptrack target/release/hedl-cli parse large-file.hedl
```

### Benchmark Comparison

```bash
# Create baseline
cargo bench --bench parsing -- --save-baseline master

# Make changes...

# Compare
cargo bench --bench parsing -- --baseline master
```

## Continuous Integration

The project uses GitHub Actions for CI/CD:

- **Tests**: Run on every commit and PR
- **Linting**: `cargo fmt` and `cargo clippy` checks
- **Coverage**: Code coverage reports
- **Benchmarks**: Performance regression detection

View CI results in the [Actions tab](https://github.com/dweve-ai/hedl/actions).

## Getting Help

### Documentation
- [Concepts](concepts/README.md)
- [How-To Guides](how-to/README.md)
- [Contributing Guide](contributing.md)
- API docs: `cargo doc --workspace --open`

### Community
- [GitHub Discussions](https://github.com/dweve-ai/hedl/discussions)
- [Issue Tracker](https://github.com/dweve-ai/hedl/issues)

### Tips
- Read existing code to understand patterns
- Start with `good first issue` labels
- Ask questions in discussions
- Review merged PRs for examples

## Next Steps

Now that you have your environment set up:

1. **Explore the codebase** - Browse `crates/` directory
2. **Understand the architecture** - See [Concepts](concepts/README.md)
3. **Learn by doing** - Check [How-To Guides](how-to/README.md)
4. **Make your first contribution** - Read [Contributing Guide](contributing.md)

Happy coding!
