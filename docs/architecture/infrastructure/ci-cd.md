# CI/CD Architecture

> Continuous integration and deployment pipeline

## Overview

HEDL uses GitHub Actions for CI/CD, running tests, benchmarks, and quality checks on every commit.

## CI Pipeline

```yaml
name: CI

on:
  push:
    branches: [ master ]
  pull_request:
    branches: [ master ]
  workflow_dispatch:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --all-features
      - run: cargo test --workspace --doc

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo doc --workspace --all-features --no-deps

  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.70"
      - run: cargo check --workspace --all-features
```

## Quality Gates

### Pre-merge Checks

1. All tests pass
2. No clippy warnings
3. Code formatted correctly
4. No security vulnerabilities
5. Documentation builds

### Performance Checks

1. Benchmarks run successfully
2. No performance regressions > 10%
3. Memory usage within limits

## Deployment

### Crate Publishing

```bash
# Publish to crates.io
cargo publish -p hedl-core
cargo publish -p hedl
```

### Documentation Deployment

```bash
# Build and deploy docs
cargo doc --workspace --no-deps
```

---

