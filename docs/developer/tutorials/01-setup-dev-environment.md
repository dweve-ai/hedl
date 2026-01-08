# Tutorial 1: Setup Development Environment

Learn how to set up a complete HEDL development environment from scratch.

## Overview

This tutorial walks you through setting up everything you need to develop HEDL:

- Install the Rust toolchain
- Clone and build the HEDL workspace
- Configure your IDE
- Run tests and benchmarks
- Verify your setup

**Time**: ~15 minutes

## Prerequisites

- A computer running Linux, macOS, or Windows (WSL2 recommended)
- Command line access
- Internet connection for downloading dependencies

## Step 1: Install Rust

### Linux and macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts and choose the default installation.

### Windows

Download and run [rustup-init.exe](https://rustup.rs/) from the official Rust website.

### Verify Installation

```bash
rustc --version
cargo --version
```

You should see output like:
```
rustc 1.75.0 (82e1608df 2024-12-21)
cargo 1.75.0 (1d8b05cdd 2024-11-20)
```

## Step 2: Install Development Tools

### Required Tools

```bash
# Install rustfmt (code formatter)
rustup component add rustfmt

# Install clippy (linter)
rustup component add clippy

# Install rust-analyzer (LSP for IDE support)
rustup component add rust-analyzer
```

### Optional but Recommended

```bash
# Install cargo-watch (auto-rebuild on file changes)
cargo install cargo-watch

# Install cargo-nextest (faster test runner)
cargo install cargo-nextest

# Install cargo-tarpaulin (code coverage)
cargo install cargo-tarpaulin

# Install cargo-audit (security auditing)
cargo install cargo-audit

# Install cargo-fuzz (fuzz testing)
cargo install cargo-fuzz
```

## Step 3: Clone HEDL Repository

```bash
# Clone the repository
git clone https://github.com/dweve-ai/hedl.git
cd hedl

# Check the workspace structure
ls -la
```

You should see:
```
crates/       # All HEDL crates
docs/         # Documentation
tests/        # Integration tests
fixtures/     # Test data
Cargo.toml    # Workspace configuration
```

## Step 4: Build the Workspace

### Full Build

```bash
# Build all crates in release mode
cargo build --all --release
```

This will take a few minutes on the first run as it downloads and compiles dependencies.

### Verify Build

```bash
# Build should complete without errors
echo $?  # Should output: 0
```

### Quick Build (Debug Mode)

For development, use debug mode (faster compile times):

```bash
cargo build --all
```

## Step 5: Run Tests

### Run All Tests

```bash
cargo test --all
```

This runs all unit tests, integration tests, and doc tests across all crates.

### Run Tests for a Specific Crate

```bash
# Test just the core parser
cargo test -p hedl-core

# Test JSON conversion
cargo test -p hedl-json
```

### Run Specific Test

```bash
# Run a specific test by name
cargo test --all test_parse_simple_value
```

### Expected Output

```
running 1250 tests
test hedl_core::tests::test_parse_simple_value ... ok
test hedl_json::tests::test_round_trip ... ok
...
test result: ok. 1250 passed; 0 failed; 0 ignored; 0 measured
```

## Step 6: Configure Your IDE

### Visual Studio Code

1. **Install Extensions**:
   - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
   - [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) (debugging)
   - [crates](https://marketplace.visualstudio.com/items?itemName=serayuzgur.crates) (dependency management)

2. **Configure Settings** (`.vscode/settings.json`):
   ```json
   {
     "rust-analyzer.check.command": "clippy",
     "rust-analyzer.cargo.allFeatures": true,
     "editor.formatOnSave": true,
     "files.watcherExclude": {
       "**/target/**": true
     }
   }
   ```

### IntelliJ IDEA / CLion

1. Install the [Rust plugin](https://plugins.jetbrains.com/plugin/8182-rust)
2. Open the `hedl` directory as a project
3. Enable Rust support in settings

### Vim / Neovim

1. Install [rust.vim](https://github.com/rust-lang/rust.vim)
2. Configure LSP with [coc.nvim](https://github.com/neoclide/coc.nvim) or native LSP
3. Add to `.vimrc` / `init.vim`:
   ```vim
   let g:rustfmt_autosave = 1
   ```

### Emacs

1. Install [rust-mode](https://github.com/rust-lang/rust-mode)
2. Configure LSP with [lsp-mode](https://github.com/emacs-lsp/lsp-mode)

## Step 7: Run Benchmarks

```bash
# Run all benchmarks (takes a few minutes)
cargo bench --all

# Run specific benchmark suite
cargo bench -p hedl-bench --bench parsing

# View benchmark results
ls -la crates/hedl-bench/target/
```

Benchmark reports are generated in:
- HTML: `target/criterion/report/index.html`
- JSON: `target/criterion/*/base/estimates.json`
- Markdown: `crates/hedl-bench/target/*_report.md`

## Step 8: Verify Your Setup

Run this complete verification script:

```bash
#!/bin/bash
set -e

echo "=== HEDL Development Environment Verification ==="

# 1. Check Rust installation
echo "✓ Checking Rust..."
rustc --version
cargo --version

# 2. Check code formatting
echo "✓ Checking code formatting..."
cargo fmt --all -- --check

# 3. Check linter
echo "✓ Running clippy..."
cargo clippy --all -- -D warnings

# 4. Build all crates
echo "✓ Building workspace..."
cargo build --all

# 5. Run tests
echo "✓ Running tests..."
cargo test --all

# 6. Check for security issues
echo "✓ Security audit..."
cargo audit

echo ""
echo "=== Setup Complete! ==="
echo "You're ready to develop HEDL!"
```

Save this as `verify_setup.sh`, make it executable, and run it:

```bash
chmod +x verify_setup.sh
./verify_setup.sh
```

## Development Workflow

### Quick Commands

```bash
# Format code before committing
cargo fmt --all

# Check for common mistakes
cargo clippy --all

# Run tests on file changes
cargo watch -x test

# Build documentation
cargo doc --all --no-deps --open

# Clean build artifacts
cargo clean
```

### Pre-Commit Checks

Before committing, always run:

```bash
cargo fmt --all
cargo clippy --all -- -D warnings
cargo test --all
```

## Common Issues and Solutions

### Issue: Slow Initial Build

**Solution**: Rust compiles dependencies from source. The first build takes 5-10 minutes. Subsequent builds are much faster due to caching.

### Issue: `cargo test` Fails with "Too Many Open Files"

**Solution**: Increase file descriptor limit:
```bash
# macOS/Linux
ulimit -n 4096

# Or add to ~/.bashrc / ~/.zshrc
echo "ulimit -n 4096" >> ~/.bashrc
```

### Issue: Out of Disk Space

**Solution**: Clean build artifacts:
```bash
cargo clean
rm -rf ~/.cargo/registry/cache
```

### Issue: Compilation Errors After Git Pull

**Solution**: Clean and rebuild:
```bash
cargo clean
cargo build --all
```

### Issue: LSP Not Working in IDE

**Solution**:
1. Ensure `rust-analyzer` is installed: `rustup component add rust-analyzer`
2. Restart your IDE
3. Check IDE settings for Rust support

## Next Steps

Now that your environment is set up:

1. **Explore the codebase**: Browse `crates/hedl-core/src/` to understand the parser
2. **Read documentation**: Run `cargo doc --all --open`
3. **Try examples**: Run `cargo run --example quick_start`
4. **Next tutorial**: [Adding Your First Feature](02-first-feature.md)

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [HEDL Architecture Guide](../architecture.md)

---

**Congratulations!** You now have a fully functional HEDL development environment.
