# HEDL CLI Fuzz Testing

Comprehensive fuzz testing suite for the HEDL command-line interface using cargo-fuzz and libFuzzer.

## Overview

This directory contains fuzz targets that test the robustness, security, and stability of HEDL CLI operations. Fuzzing helps discover:

- **Crashes and panics** from malformed inputs
- **Memory safety issues** (buffer overflows, use-after-free, etc.)
- **Edge cases** in parsing, conversion, and formatting
- **Performance issues** with deeply nested or oversized structures
- **Security vulnerabilities** in file I/O and input validation

## Prerequisites

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

Note: Fuzzing requires a nightly Rust toolchain.

## Fuzz Targets

### Core Parsing

**`fuzz_parse`** - Tests HEDL parser with arbitrary input
- Malformed HEDL syntax
- UTF-8 edge cases
- Deeply nested structures
- Extremely large inputs

```bash
cargo fuzz run fuzz_parse
```

### Format Command

**`fuzz_format`** - Tests format command operations
- Format with/without ditto markers
- Format with count hints
- Canonicalization edge cases
- File I/O error handling

```bash
cargo fuzz run fuzz_format
```

### Conversion Commands

**`fuzz_convert`** - Tests all conversion operations
- HEDL ↔ JSON roundtrips
- HEDL ↔ YAML roundtrips
- HEDL ↔ XML roundtrips
- HEDL → CSV conversions
- Edge cases in type conversions

```bash
cargo fuzz run fuzz_convert
```

**`fuzz_json_roundtrip`** - Focused JSON conversion testing
- HEDL → JSON → HEDL stability
- Metadata preservation
- Edge cases in JSON serialization

```bash
cargo fuzz run fuzz_json_roundtrip
```

**`fuzz_yaml_roundtrip`** - Focused YAML conversion testing
- HEDL → YAML → HEDL stability
- YAML format variations

```bash
cargo fuzz run fuzz_yaml_roundtrip
```

**`fuzz_xml_roundtrip`** - Focused XML conversion testing
- HEDL → XML → HEDL stability
- XML format variations

```bash
cargo fuzz run fuzz_xml_roundtrip
```

### Stats Command

**`fuzz_stats`** - Tests statistics generation
- Size calculation accuracy
- Token estimation edge cases
- Conversion to all formats for comparison
- Formatting edge cases

```bash
cargo fuzz run fuzz_stats
```

### Lint Command

**`fuzz_lint`** - Tests linting functionality
- Diagnostic generation
- JSON serialization of diagnostics
- Severity classification
- Edge cases in lint rules

```bash
cargo fuzz run fuzz_lint
```

## Running Fuzz Tests

### Quick Test (10 seconds)

```bash
cargo fuzz run fuzz_parse -- -max_total_time=10
```

### Extended Test (1 hour)

```bash
cargo fuzz run fuzz_convert -- -max_total_time=3600
```

### Run All Targets (Sequential)

```bash
for target in fuzz_parse fuzz_format fuzz_convert fuzz_stats fuzz_lint \
              fuzz_json_roundtrip fuzz_yaml_roundtrip fuzz_xml_roundtrip; do
    echo "Running $target..."
    cargo fuzz run "$target" -- -max_total_time=60
done
```

### With Custom Options

```bash
# Use multiple workers (parallel fuzzing)
cargo fuzz run fuzz_parse -- -workers=4

# Limit memory usage
cargo fuzz run fuzz_parse -- -rss_limit_mb=2048

# Minimize corpus
cargo fuzz run fuzz_parse -- -merge=1
```

## Security Features Tested

### Input Size Limits

All fuzz targets enforce a 100 KB size limit to prevent timeout and resource exhaustion:

```rust
if data.len() > 100_000 {
    return;
}
```

This mirrors the production CLI's 100 MB limit (scaled for fuzzing efficiency).

### File I/O Safety

Format fuzzer tests file operations with temporary files to ensure:
- No path traversal vulnerabilities
- Proper error handling for I/O failures
- Safe handling of special characters in paths

### Error Handling

All fuzz targets verify that operations **never panic**, only return errors:

```rust
// Should never panic, only return Result
let _ = parse(text.as_bytes());
```

### Type Safety

Conversion fuzzers test type conversion edge cases:
- Invalid type names (CSV conversion)
- Type mismatches between formats
- Null/undefined handling
- Numeric overflow/underflow

## Corpus Management

### View Corpus

```bash
ls fuzz/corpus/fuzz_parse/
```

### Add Interesting Inputs

```bash
echo -e "%VERSION: 1.0\n%STRUCT: Team: [id,name]\n---\nteams: @Team\n  |t1,TeamA" > fuzz/corpus/fuzz_parse/example.hedl
```

### Minimize Corpus

```bash
cargo fuzz cmin fuzz_parse
```

### Merge Corpora

```bash
cargo fuzz run fuzz_parse -- -merge=1
```

## Analyzing Crashes

### Reproduce a Crash

```bash
cargo fuzz run fuzz_parse fuzz/artifacts/fuzz_parse/crash-<hash>
```

### Debug a Crash

```bash
# Run with debugger
rust-lldb target/x86_64-unknown-linux-gnu/release/fuzz_parse \
    fuzz/artifacts/fuzz_parse/crash-<hash>
```

### Generate Coverage Report

```bash
cargo fuzz coverage fuzz_parse
```

## Continuous Integration

Add to CI pipeline:

```yaml
- name: Fuzz Testing (Quick)
  run: |
    cd crates/hedl-cli/fuzz
    for target in fuzz_parse fuzz_format fuzz_convert fuzz_stats fuzz_lint; do
      cargo fuzz run "$target" -- -max_total_time=60 -rss_limit_mb=2048
    done
```

## Performance Considerations

### Timeout Protection

All fuzz targets enforce time limits to prevent fuzzer hanging:
- 100 KB input size limit
- Conversion operations timeout naturally via size limits

### Memory Safety

Fuzz targets run with AddressSanitizer (ASan) enabled to detect:
- Heap buffer overflows
- Stack buffer overflows
- Use-after-free
- Memory leaks
- Double free

### CPU Efficiency

Fuzz targets are optimized for fuzzing performance:
- Early returns for invalid UTF-8
- Size limits prevent exponential complexity
- Minimal allocations in hot paths

## Best Practices

### Regular Fuzzing

Run fuzz tests regularly:
- **Daily**: Quick 1-minute runs of all targets
- **Weekly**: Extended 1-hour runs of critical targets
- **Pre-release**: 24-hour continuous fuzzing

### Corpus Seeding

Seed corpus with real-world examples:
- Valid HEDL documents from tests
- Edge cases discovered manually
- Examples from documentation

### Coverage Tracking

Monitor code coverage improvements:

```bash
cargo fuzz coverage fuzz_parse
genhtml -o coverage/ fuzz/coverage/fuzz_parse/coverage.profdata
```

### Issue Tracking

When crashes are found:
1. Reproduce the crash
2. Create a minimal test case
3. File an issue with the crash input
4. Add to regression test suite once fixed

## Troubleshooting

### "Address sanitizer failed to allocate"

Increase memory limit:

```bash
cargo fuzz run fuzz_parse -- -rss_limit_mb=4096
```

### "Timeout" errors

Reduce input complexity or add early returns for known slow paths.

### "No new coverage" messages

This is normal after initial fuzzing. Consider:
- Switching to a different target
- Merging corpora from multiple runs
- Adding seed inputs for uncovered code paths

## References

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing strategies](https://google.github.io/clusterfuzz/)
- [HEDL security policy](../../SECURITY.md)

## License

Same as parent project (MIT OR Apache-2.0).
