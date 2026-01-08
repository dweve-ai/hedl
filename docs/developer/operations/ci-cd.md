# CI/CD Pipeline

Overview of HEDL's continuous integration and deployment setup.

## GitHub Actions Workflows

### Main CI Workflow

File: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]
  workflow_dispatch:

jobs:
  test:
    name: Tests
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Run tests
        run: cargo test --workspace --all-features

      - name: Run doc tests
        run: cargo test --workspace --doc

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Run clippy
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings

  fmt:
    name: Formatting
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

  msrv:
    name: Minimum Supported Rust Version
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust 1.70
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.70"

      - name: Check MSRV
        run: cargo check --workspace --all-features
```

### Benchmark Workflow

File: `.github/workflows/benchmarks.yml`

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v2

      - name: Run benchmarks
        run: cargo bench --all -- --save-baseline pr

      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: target/criterion/
```

### Coverage Workflow

File: `.github/workflows/coverage.yml`

```yaml
name: Coverage

on:
  push:
    branches: [main]

jobs:
  coverage:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v2

      - name: Run tarpaulin
        uses: actions-rs/tarpaulin@v0.1
        with:
          args: '--all --out Lcov'

      - name: Upload to codecov
        uses: codecov/codecov-action@v2
```

## Local CI Simulation

Run CI checks locally before pushing:

```bash
# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Tests
cargo test --workspace --all-features

# Documentation
cargo doc --workspace --all-features --no-deps

# MSRV check
cargo +1.70 check --workspace --all-features
```

## Related

- [Testing Guide](../testing.md)
- [Benchmarking Guide](../benchmarking.md)
- [Release Process](../guides/release-process.md)
