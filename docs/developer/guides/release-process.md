# Release Process: Shipping with Confidence

A release is a promise. When you publish version 1.2.3, you're telling users: "This version works. This version is stable. You can depend on this." Breaking that promise erodes trust. Users who've been burned by bad releases become cautious, then skeptical, then they leave.

This guide ensures every HEDL release is worthy of trust. It's a checklist, yes, but more importantly, it's a philosophy: measure twice, cut once, and never ship on Friday.

```
╔═══════════════════════════════════════════════════════════════════╗
║                    THE RELEASE COVENANT                           ║
╠═══════════════════════════════════════════════════════════════════╣
║                                                                   ║
║   Every release promises:                                        ║
║                                                                   ║
║   ✓ All tests pass                                               ║
║   ✓ No new warnings                                              ║
║   ✓ Documentation is accurate                                    ║
║   ✓ Version number follows SemVer                                ║
║   ✓ CHANGELOG explains what changed                              ║
║   ✓ No regressions from previous version                        ║
║                                                                   ║
║   Break any of these, and you break trust.                       ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
```

---

## Semantic Versioning: The Version Contract

HEDL follows Semantic Versioning (SemVer). This isn't just a numbering scheme. It's a contract with users about what changes they can expect.

```
┌─────────────────────────────────────────────────────────────────┐
│                    VERSION ANATOMY                              │
│                                                                 │
│                     1  .  2  .  3                               │
│                     │     │     │                               │
│                     │     │     └── PATCH: Bug fixes only       │
│                     │     │         No new features             │
│                     │     │         No breaking changes         │
│                     │     │         Safe to upgrade always      │
│                     │     │                                     │
│                     │     └── MINOR: New features               │
│                     │         Backward compatible               │
│                     │         Existing code keeps working       │
│                     │         Safe to upgrade usually           │
│                     │                                           │
│                     └── MAJOR: Breaking changes                 │
│                         Existing code may break                 │
│                         Migration may be needed                 │
│                         Upgrade with caution                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### What Triggers Each Version Type

**PATCH (1.2.3 → 1.2.4)**
- Bug fixes that don't change documented behavior
- Performance improvements with no API changes
- Documentation corrections
- Security patches

**MINOR (1.2.3 → 2.0.0)**
- New functions, types, or modules
- New optional parameters with defaults
- Deprecations (not removals)
- Non-breaking behavioral improvements

**MAJOR (1.2.3 → 2.0.0)**
- Removed public APIs
- Changed function signatures
- Changed type definitions
- Behavioral changes that break existing code

---

## The Pre-Release Checklist

Before any release, every item on this list must be verified. No exceptions. No shortcuts.

```
┌─────────────────────────────────────────────────────────────────┐
│                    PRE-RELEASE CHECKLIST                        │
│                                                                 │
│  CODE QUALITY                                                   │
│  □ All tests pass                                              │
│    cargo test --all --all-features                             │
│                                                                 │
│  □ No clippy warnings                                          │
│    cargo clippy --all --all-features -- -D warnings            │
│                                                                 │
│  □ Code is formatted                                           │
│    cargo fmt --all -- --check                                  │
│                                                                 │
│  □ Benchmarks run successfully                                 │
│    cargo bench --all                                           │
│                                                                 │
│  DOCUMENTATION                                                  │
│  □ Docs build without warnings                                 │
│    cargo doc --all --no-deps                                   │
│                                                                 │
│  □ Doc tests pass                                              │
│    cargo test --doc --all                                      │
│                                                                 │
│  □ CHANGELOG.md updated                                        │
│                                                                 │
│  □ README.md accurate                                          │
│                                                                 │
│  VERSION                                                        │
│  □ Version bumped in all Cargo.toml files                      │
│                                                                 │
│  □ Version follows SemVer appropriately                        │
│                                                                 │
│  □ Internal dependencies updated                               │
│                                                                 │
│  SECURITY                                                       │
│  □ No known vulnerabilities                                    │
│    cargo audit                                                 │
│                                                                 │
│  □ Dependencies up to date                                     │
│    cargo outdated                                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## The Release Process

### Step 1: Verify Everything Passes

Run the full verification suite:

```bash
# Format check
cargo fmt --all -- --check

# Clippy with maximum strictness
cargo clippy --all --all-features -- -D warnings

# All tests
cargo test --all --all-features

# Doc tests specifically
cargo test --doc --all

# Build docs
cargo doc --all --no-deps

# Security audit
cargo audit

# Benchmarks (ensure no regressions)
cargo bench --all
```

If any command fails, stop. Fix the issue before proceeding.

### Step 2: Update the Version

Edit the version in the root `Cargo.toml`:

```toml
[package]
name = "hedl"
version = "1.2.3"  # Update this
```

For workspace crates, update each crate's version and internal dependencies:

```toml
# In crates/hedl-json/Cargo.toml
[package]
name = "hedl-json"
version = "1.2.3"  # Match the release version

[dependencies]
hedl-core = { version = "1.2.3", path = "../hedl-core" }
```

Verify the changes compile:

```bash
cargo check --all
```

### Step 3: Update the CHANGELOG

Add an entry at the top of `CHANGELOG.md`:

```markdown
## [1.2.3] - 2025-01-15

### Added
- New `parse_streaming` function for memory-efficient parsing of large files
- Support for `%SV:` schema versioning directive
- JSON output now supports `pretty` option

### Fixed
- Reference resolution correctly handles circular references (#123)
- Parser no longer panics on deeply nested documents (#145)
- Fixed memory leak in streaming parser (#167)

### Changed
- Improved error messages for schema mismatches
- Reduced memory usage by 15% for typical documents
- Updated `serde` dependency to 1.0.195

### Deprecated
- `parse_legacy` is deprecated; use `parse_with_options` instead

### Security
- Fixed potential DoS via deeply nested structures (CVE-2025-XXXX)
```

Follow the [Keep a Changelog](https://keepachangelog.com/) format:
- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Features that will be removed
- **Removed**: Features removed in this release
- **Fixed**: Bug fixes
- **Security**: Security-related fixes

### Step 4: Commit and Tag

Create the release commit:

```bash
# Stage the version changes
git add -A

# Commit with conventional format
git commit -m "chore: release v1.2.3

- Added streaming parser
- Fixed circular reference handling
- Improved error messages

See CHANGELOG.md for full details."

# Create an annotated tag
git tag -a v1.2.3 -m "Release v1.2.3"
```

### Step 5: Publish to crates.io

Publish crates in dependency order (dependencies before dependents):

```bash
# Core crates first
cargo publish -p hedl-core
sleep 30  # Wait for crates.io to index

# Then crates that depend on hedl-core
cargo publish -p hedl-c14n
cargo publish -p hedl-lint
cargo publish -p hedl-stream
sleep 30

# Format converters
cargo publish -p hedl-json
cargo publish -p hedl-yaml
cargo publish -p hedl-xml
cargo publish -p hedl-csv
cargo publish -p hedl-parquet
cargo publish -p hedl-toon
cargo publish -p hedl-neo4j
sleep 30

# Higher-level crates
cargo publish -p hedl-ffi
cargo publish -p hedl-wasm
sleep 30

# Tools
cargo publish -p hedl-cli
cargo publish -p hedl-lsp
cargo publish -p hedl-mcp
sleep 30

# Finally, the umbrella crate
cargo publish -p hedl

# Test utilities last
cargo publish -p hedl-test
cargo publish -p hedl-bench
```

The `sleep` commands give crates.io time to index each crate before publishing dependents.

### Step 6: Push to GitHub

```bash
# Push the commit
git push origin main

# Push the tag
git push origin v1.2.3
```

### Step 7: Create GitHub Release

1. Go to https://github.com/your-org/hedl/releases/new
2. Select the tag `v1.2.3`
3. Title: `HEDL v1.2.3`
4. Description: Copy the relevant section from CHANGELOG.md
5. Check "Set as the latest release"
6. Click "Publish release"

---

## Handling Release Problems

### A Test Fails After Publishing

If you discover a problem after publishing:

1. **Don't panic**. Take a breath.
2. **Assess severity**. Is it a minor bug or a critical issue?
3. **For minor issues**: Fix and release a patch (1.2.4)
4. **For critical issues**: Yank the broken version:
   ```bash
   cargo yank --vers 1.2.3 hedl
   ```
   Then fix and release a new patch.

### Version Mismatch Between Crates

If crates publish with mismatched versions:

1. Yank the mismatched crates
2. Fix the version numbers
3. Publish again with correct versions

### crates.io Publish Fails

Common causes:
- **"already exists"**: Version already published. Bump version.
- **"dependency not found"**: Publish dependencies first. Wait longer.
- **"not logged in"**: Run `cargo login` with your token.

---

## Automation with CI

For teams, automate the release process with GitHub Actions:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run tests
        run: cargo test --all --all-features

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all --all-features -- -D warnings

      - name: Build docs
        run: cargo doc --all --no-deps

  publish:
    needs: verify
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cargo publish -p hedl-core
          sleep 30
          # ... rest of publish commands

  github-release:
    needs: publish
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Extract changelog
        id: changelog
        run: |
          # Extract the section for this version from CHANGELOG.md
          VERSION=${GITHUB_REF#refs/tags/v}
          # ... extraction logic

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          body: ${{ steps.changelog.outputs.content }}
```

---

## The Release Rhythm

Good projects have a predictable release cadence:

```
┌─────────────────────────────────────────────────────────────────┐
│                    SUGGESTED RELEASE RHYTHM                     │
│                                                                 │
│  PATCH releases (1.2.x)                                         │
│  └── As needed for bug fixes                                   │
│      No waiting, ship when ready                               │
│                                                                 │
│  MINOR releases (1.x.0)                                         │
│  └── Monthly or when significant features accumulate            │
│      Bundle multiple features for user convenience              │
│                                                                 │
│  MAJOR releases (x.0.0)                                         │
│  └── Rare, only when breaking changes provide clear value      │
│      Communicate well in advance                               │
│      Provide migration guides                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Final Thoughts

A release is the moment your code meets the world. Make it count. Verify thoroughly. Document clearly. Ship confidently.

And never, ever ship on Friday afternoon.

---

## Related Documentation

- **[Contributing Guide](../contributing.md)**: How contributions lead to releases
- **[API Design Guidelines](api-design.md)**: How API design affects versioning
- **[Testing](../testing.md)**: How testing ensures release quality
