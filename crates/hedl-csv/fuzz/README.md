# Fuzz Testing for hedl-csv

This directory contains fuzz testing infrastructure for the `hedl-csv` crate using `cargo-fuzz` (libFuzzer).

## Overview

Fuzz testing automatically generates random inputs to discover bugs, crashes, and security vulnerabilities. The hedl-csv fuzzer focuses on:

- **Security**: DoS protection, row limits, memory exhaustion
- **Robustness**: Handling malformed CSV, invalid UTF-8, edge cases
- **Correctness**: Type inference, parsing logic, error handling

## Setup

### Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

### Nightly Rust Required

Fuzzing requires nightly Rust:

```bash
rustup install nightly
```

## Running Fuzzers

### Basic Usage

From the `hedl-csv` directory:

```bash
# Run the main CSV parsing fuzzer
cargo +nightly fuzz run fuzz_from_csv

# Run the value parsing fuzzer
cargo +nightly fuzz run fuzz_parse_value
```

### Advanced Options

```bash
# Limit input length (faster iteration)
cargo +nightly fuzz run fuzz_from_csv -- -max_len=10000

# Run for specific time (seconds)
cargo +nightly fuzz run fuzz_from_csv -- -max_total_time=300

# Use multiple CPU cores
cargo +nightly fuzz run fuzz_from_csv -- -jobs=8

# Run with address sanitizer (detect memory bugs)
cargo +nightly fuzz run fuzz_from_csv --sanitizer=address

# Run with memory sanitizer (detect uninitialized reads)
cargo +nightly fuzz run fuzz_from_csv --sanitizer=memory
```

## Fuzz Targets

### fuzz_from_csv

Tests the complete CSV parsing pipeline:

- Various delimiters (comma, tab, semicolon)
- Header/no-header modes
- Whitespace trimming
- Row count limits
- Multi-field schemas
- UTF-8 validation

**Location**: `fuzz_targets/fuzz_from_csv.rs`

### fuzz_parse_value

Tests individual field value parsing and type inference:

- Null detection
- Boolean parsing
- Integer/float parsing
- Reference syntax
- Expression syntax
- Tensor literals
- String fallback

**Location**: `fuzz_targets/fuzz_parse_value.rs`

## Corpus

Fuzzing builds a corpus of interesting inputs in `fuzz/corpus/<target_name>/`. You can:

- **Add seeds**: Place sample CSV files in the corpus directory
- **Share corpus**: Corpus can be version controlled to improve coverage
- **Minimize**: Run `cargo fuzz cmin <target>` to minimize corpus

### Example Seeds

Create `fuzz/corpus/fuzz_from_csv/` with files like:

```csv
# edge_cases.csv
id,value
1,@ref
2,$(expr)
3,true
4,NaN
5,[1,2,3]
```

## Artifacts

When the fuzzer finds a crash, it saves the input to `fuzz/artifacts/<target_name>/`. You can:

- **Reproduce**: `cargo +nightly fuzz run <target> <artifact_file>`
- **Debug**: Use the artifact in tests or debugger
- **Report**: Include artifacts in bug reports

## Continuous Fuzzing

### Integration with CI

Add to `.github/workflows/fuzz.yml`:

```yaml
name: Fuzz Testing
on:
  schedule:
    - cron: '0 0 * * *'  # Daily
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
      - run: cargo install cargo-fuzz
      - run: cd crates/hedl-csv && cargo +nightly fuzz run fuzz_from_csv -- -max_total_time=600
```

### OSS-Fuzz Integration

For continuous fuzzing at scale, consider [OSS-Fuzz](https://github.com/google/oss-fuzz):

1. Submit project to OSS-Fuzz
2. OSS-Fuzz runs fuzzers 24/7
3. Automatic bug reports via GitHub issues

## Performance

- **Speed**: LibFuzzer achieves ~10,000 executions/second
- **Coverage**: Use `cargo +nightly fuzz coverage <target>` to measure
- **Memory**: Fuzzer limited to 2GB RSS by default

## Best Practices

1. **Run regularly**: Fuzz for at least 1 hour before releases
2. **Fix quickly**: Address crashes immediately (likely security bugs)
3. **Expand corpus**: Add real-world CSV files as seeds
4. **Monitor coverage**: Ensure new code is fuzzed
5. **Use sanitizers**: Rotate between ASAN, MSAN, UBSAN

## Troubleshooting

### Fuzzer Crashes Immediately

Check for:
- Missing dependencies
- Incorrect nightly version
- Sanitizer configuration issues

### Low Exec/s

- Reduce `-max_len`
- Simplify fuzz target
- Use `-jobs=1` to measure baseline

### No New Coverage

- Add diverse seed inputs
- Check if code paths are reachable
- Consider dictionary-based fuzzing

## References

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz project](https://github.com/rust-fuzz)

## Security Disclosure

If fuzzing discovers security vulnerabilities, please:

1. **Do not** commit artifacts to version control
2. Report via security contact (see main README)
3. Wait for patch before disclosure
