# Release Process

Step-by-step guide to releasing new versions of HEDL.

## Pre-Release Checklist

- [ ] All tests pass: `cargo test --all`
- [ ] Benchmarks run: `cargo bench --all`
- [ ] Clippy clean: `cargo clippy --all -- -D warnings`
- [ ] Docs build: `cargo doc --all --no-deps`
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml

## Version Numbering

Follow Semantic Versioning (SemVer):

- **Major (X.0.0)**: Breaking changes
- **Minor (0.X.0)**: New features, backward compatible
- **Patch (0.0.X)**: Bug fixes

## Release Steps

### 1. Update Version

```bash
# Edit Cargo.toml
version = "1.2.3"

# Verify
cargo check --all
```

### 2. Update CHANGELOG

```markdown
## [1.2.3] - 2025-01-06

### Added
- New feature X

### Fixed
- Bug in Y

### Changed
- Improved Z
```

### 3. Commit and Tag

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: Release v1.2.3"
git tag -a v1.2.3 -m "Release v1.2.3"
```

### 4. Publish to crates.io

```bash
cargo publish -p hedl-core
cargo publish -p hedl-json
# ... publish in dependency order
cargo publish -p hedl
```

### 5. Push to GitHub

```bash
git push origin main --tags
```

### 6. Create GitHub Release

1. Go to https://github.com/dweve-ai/hedl/releases/new
2. Select tag v1.2.3
3. Title: "HEDL v1.2.3"
4. Body: Copy from CHANGELOG.md
5. Publish release

## Related

- [Contributing Guide](../contributing.md)
- [API Design](api-design.md)
