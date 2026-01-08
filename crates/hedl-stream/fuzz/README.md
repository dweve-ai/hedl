# Fuzz Testing for hedl-stream

This directory contains fuzz testing infrastructure for the `hedl-stream` crate using `cargo-fuzz` and `libfuzzer-sys`.

## Overview

Fuzz testing validates the parser's robustness against arbitrary, potentially malformed input. The fuzzer:

- Tests all parsing code paths with random input
- Detects memory safety issues (use-after-free, buffer overflows, etc.)
- Validates resource limit enforcement
- Ensures no panic paths exist for invalid input
- Checks UTF-8 handling

## Installation

First, install `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

Note: Fuzzing requires a nightly Rust compiler:

```bash
rustup default nightly
# or for this directory only
rustup override set nightly
```

## Running the Fuzzer

### Basic Usage

```bash
cd crates/hedl-stream
cargo fuzz run fuzz_streaming_parser
```

This will run indefinitely, generating random inputs and testing the parser.

### With Options

```bash
# Limit maximum input size to 100KB
cargo fuzz run fuzz_streaming_parser -- -max_len=100000

# Run for a specific duration (600 seconds = 10 minutes)
cargo fuzz run fuzz_streaming_parser -- -max_total_time=600

# Use multiple cores
cargo fuzz run fuzz_streaming_parser -- -jobs=8

# Minimize test cases
cargo fuzz run fuzz_streaming_parser -- -minimize_crash=1
```

### Using a Corpus

Fuzz testing works best with a corpus of interesting inputs:

```bash
# Create a corpus directory
mkdir -p fuzz/corpus/fuzz_streaming_parser

# Add sample HEDL files
cp ../../../examples/*.hedl fuzz/corpus/fuzz_streaming_parser/

# Run with corpus
cargo fuzz run fuzz_streaming_parser
```

## Crashes and Artifacts

When the fuzzer finds a crash, it saves the input to `fuzz/artifacts/`:

```bash
# Reproduce a crash
cargo fuzz run fuzz_streaming_parser fuzz/artifacts/fuzz_streaming_parser/crash-<hash>

# Debug with gdb
cargo fuzz run --debug-assertions fuzz_streaming_parser fuzz/artifacts/...
```

## Coverage

To see code coverage from fuzzing:

```bash
# Generate coverage report
cargo fuzz coverage fuzz_streaming_parser

# View with llvm-cov
llvm-cov show target/*/release/fuzz_streaming_parser \
    --format=html \
    --instr-profile=fuzz/coverage/fuzz_streaming_parser/coverage.profdata \
    > coverage.html
```

## Continuous Fuzzing

For production, consider:

- **OSS-Fuzz**: Continuous fuzzing service for open source projects
- **ClusterFuzz**: Google's scalable fuzzing infrastructure
- **CI Integration**: Run fuzzing for 10-60 minutes in CI/CD

Example GitHub Actions integration:

```yaml
- name: Fuzz Test
  run: |
    cargo install cargo-fuzz
    cd crates/hedl-stream
    cargo fuzz run fuzz_streaming_parser -- -max_total_time=600
```

## Targets

### fuzz_streaming_parser

**File:** `fuzz_targets/fuzz_streaming_parser.rs`

**Tests:**
- Header parsing (VERSION, STRUCT, ALIAS, NEST directives)
- Document body parsing (lists, nodes, scalars)
- Indentation handling and nesting
- Comment stripping
- Value inference (numbers, booleans, references, etc.)
- Error handling for all malformed inputs

**Expected Behavior:**
- No panics (except documented `expect()` calls)
- All errors returned as `Result::Err`
- Resource limits enforced (max_line_length, max_indent_depth)
- Invalid UTF-8 handled gracefully

## Performance

Typical fuzzing performance:
- **Speed:** ~50,000 executions/second on modern CPU
- **Coverage:** Reaches 95%+ code coverage within minutes
- **Corpus Size:** Stabilizes at ~100-500 interesting inputs

## Troubleshooting

### Out of Memory

Reduce maximum input size:
```bash
cargo fuzz run fuzz_streaming_parser -- -rss_limit_mb=2048 -max_len=10000
```

### Slow Execution

The fuzzer may slow down with very large inputs. Limit input size:
```bash
cargo fuzz run fuzz_streaming_parser -- -max_len=50000
```

### Timeout

Some inputs may cause slow parsing. Add timeout:
```bash
cargo fuzz run fuzz_streaming_parser -- -timeout=10
```

## Resources

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing best practices](https://google.github.io/oss-fuzz/getting-started/new-project-guide/)

## Reporting Issues

If the fuzzer finds a crash:

1. Save the crashing input from `fuzz/artifacts/`
2. Reproduce with the provided input
3. File an issue with:
   - The crashing input
   - Stack trace
   - Rust version
   - OS and architecture

## License

Same as parent crate (Apache-2.0)
