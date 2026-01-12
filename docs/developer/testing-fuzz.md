# Fuzz Testing Guide

Comprehensive guide to fuzz testing HEDL parsers and converters.

---

## Overview

HEDL uses comprehensive fuzz testing to discover edge cases, security vulnerabilities, and parser robustness issues. Our fuzz testing infrastructure is built on **cargo-fuzz** (libFuzzer) and covers critical parsing paths across multiple crates.

### Why Fuzz Testing?

Fuzz testing automatically generates malformed, edge-case, and adversarial inputs to find:
- **Security vulnerabilities** (buffer overflows, DoS, injection attacks)
- **Parser crashes** (panics, stack overflows)
- **Spec violations** (incorrect parsing behavior)
- **Memory safety issues** (use-after-free, memory leaks)

---

## Quick Start

### Prerequisites

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Install nightly Rust (required by libFuzzer)
rustup install nightly
```

### Running Fuzz Tests

```bash
# Run a specific fuzz target (hedl-core example)
cd crates/hedl-core
cargo +nightly fuzz run fuzz_parse

# Run with custom timeout (5 minutes)
cargo +nightly fuzz run fuzz_parse -- -max_total_time=300

# Run until specific number of executions
cargo +nightly fuzz run fuzz_parse -- -runs=1000000
```

###Continuous Fuzzing (Recommended)

```bash
# Run indefinitely (stop with Ctrl+C)
cargo +nightly fuzz run fuzz_parse

# Run all targets in sequence
for target in $(cargo +nightly fuzz list); do
    echo "Fuzzing: $target"
    cargo +nightly fuzz run $target -- -max_total_time=300
done
```

---

## Fuzz Target Catalog

### hedl-core (7 targets)

**Location**: `crates/hedl-core/fuzz/fuzz_targets/`

1. **fuzz_parse.rs** - Main HEDL parser
   - **Purpose**: General parsing robustness
   - **Inputs**: Arbitrary HEDL documents
   - **Finds**: Syntax errors, panics, hangs
   - **Runtime**: ~1M exec/sec

2. **fuzz_limits.rs** - Security limits enforcement
   - **Purpose**: Verify limit checking under adversarial input
   - **Inputs**: Documents designed to exceed limits
   - **Finds**: DoS vectors, limit bypasses
   - **Runtime**: ~800K exec/sec

3. **fuzz_references.rs** - Reference resolution
   - **Purpose**: Test forward refs, circular refs, missing refs
   - **Inputs**: Complex reference graphs
   - **Finds**: Reference resolution bugs, infinite loops
   - **Runtime**: ~600K exec/sec

4. **fuzz_nest.rs** - NEST hierarchy parsing
   - **Purpose**: Deep nesting, orphan rows, parent-child relationships
   - **Inputs**: Nested structures with varying depths
   - **Finds**: Stack overflows, hierarchy bugs
   - **Runtime**: ~700K exec/sec

5. **fuzz_matrix.rs** - CSV matrix parsing
   - **Purpose**: Quoted strings, escaping, ditto, field counts
   - **Inputs**: Malformed CSV rows
   - **Finds**: Quote handling bugs, field misalignment
   - **Runtime**: ~1.2M exec/sec

6. **fuzz_value_inference.rs** - Type inference
   - **Purpose**: Number parsing, bool parsing, tensor literals
   - **Inputs**: Ambiguous value strings
   - **Finds**: Type confusion, parsing errors
   - **Runtime**: ~1.5M exec/sec

7. **fuzz_block_strings.rs** - Multi-line block strings
   - **Purpose**: Triple-quote handling, newlines, truncation
   - **Inputs**: Malformed block strings
   - **Finds**: Buffer issues, newline handling bugs
   - **Runtime**: ~900K exec/sec

### hedl-stream (3 targets)

**Location**: `crates/hedl-stream/fuzz/fuzz_targets/`

1. **fuzz_streaming_parse.rs** - Streaming parser
   - **Purpose**: Incremental parsing, event generation
   - **Inputs**: Arbitrary HEDL streams
   - **Finds**: Streaming bugs, memory leaks
   - **Runtime**: ~800K exec/sec

2. **fuzz_streaming_limits.rs** - Streaming limits
   - **Purpose**: Timeout enforcement, resource limits
   - **Inputs**: Large documents, slow inputs
   - **Finds**: Timeout bypasses, resource exhaustion
   - **Runtime**: ~500K exec/sec

3. **fuzz_large_documents.rs** - Large document handling
   - **Purpose**: Memory efficiency, streaming correctness
   - **Inputs**: 10MB+ documents
   - **Finds**: Memory leaks, performance issues
   - **Runtime**: ~50K exec/sec

### hedl-json (3 targets)

**Location**: `crates/hedl-json/fuzz/fuzz_targets/`

1. **fuzz_json_conversion.rs** - HEDL ↔ JSON conversion
   - **Purpose**: Roundtrip correctness
   - **Inputs**: Complex JSON structures
   - **Finds**: Conversion bugs, data loss
   - **Runtime**: ~600K exec/sec

2. **fuzz_json_schema.rs** - JSON Schema generation
   - **Purpose**: Schema generation correctness
   - **Inputs**: Various HEDL documents
   - **Finds**: Schema bugs, missing constraints
   - **Runtime**: ~400K exec/sec

3. **fuzz_partial_parsing.rs** - Error tolerance modes
   - **Purpose**: Partial parsing, error recovery
   - **Inputs**: Malformed documents
   - **Finds**: Recovery bugs, crashes
   - **Runtime**: ~500K exec/sec

### hedl-xml (2 targets)

**Location**: `crates/hedl-xml/fuzz/fuzz_targets/`

1. **fuzz_xml_conversion.rs** - HEDL ↔ XML conversion
   - **Purpose**: Roundtrip correctness, entity handling
   - **Inputs**: Complex XML structures
   - **Finds**: Conversion bugs, entity issues
   - **Runtime**: ~300K exec/sec

2. **fuzz_xml_namespaces.rs** - XML namespace handling
   - **Purpose**: Namespace preservation, conflicts
   - **Inputs**: Documents with namespaces
   - **Finds**: Namespace bugs
   - **Runtime**: ~250K exec/sec

---

## Corpus Management

### Corpus Location

Fuzzing corpora are stored in `fuzz/corpus/<target>/`:

```
crates/hedl-core/fuzz/corpus/
├── fuzz_parse/
│   ├── seed1.hedl      # Manually added seed
│   ├── crash-abc123    # Crash-inducing input
│   └── ...
├── fuzz_limits/
└── ...
```

### Adding Seed Inputs

```bash
# Add a seed input to a specific target
cd crates/hedl-core
echo "%VERSION: 1.0\n---\nkey: value" > fuzz/corpus/fuzz_parse/basic.hedl

# Minimize corpus (remove redundant inputs)
cargo +nightly fuzz cmin fuzz_parse
```

### Crash Triage

When a crash is found:

1. **Reproduce**: `cargo +nightly fuzz run <target> fuzz/artifacts/<target>/crash-<hash>`
2. **Minimize**: `cargo +nightly fuzz tmin <target> fuzz/artifacts/<target>/crash-<hash>`
3. **Debug**: Add crash input to unit tests, fix bug
4. **Verify**: `cargo test` and re-run fuzzer

---

## Fuzzing Best Practices

### Recommended Fuzzing Schedule

**Daily** (automated):
- Run all targets for 5 minutes each (~40 minutes total)
- Check for new crashes
- Update corpus

**Weekly** (extended):
- Run critical targets (parse, limits) for 1 hour each
- Analyze coverage reports
- Add edge case seeds

**Monthly** (deep):
- 24-hour fuzzing campaign on all targets
- Corpus minimization
- Coverage analysis and improvement

### CI Integration

```yaml
# .github/workflows/fuzz.yml
name: Fuzzing
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - name: Run fuzz tests
        run: |
          for crate in hedl-core hedl-stream hedl-json hedl-xml; do
            cd crates/$crate
            for target in $(cargo +nightly fuzz list); do
              cargo +nightly fuzz run $target -- -max_total_time=300 -verbosity=0
            done
            cd ../..
          done
      - name: Upload crashes
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-artifacts
          path: '**/fuzz/artifacts'
```

---

## Writing New Fuzz Targets

### Template

```rust
// crates/hedl-core/fuzz/fuzz_targets/fuzz_my_feature.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use hedl_core::parse;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string (ignore invalid UTF-8)
    if let Ok(s) = std::str::from_utf8(data) {
        // Call the function under test
        let _ = parse(s.as_bytes());
        // Crashes, panics, or assertion failures are automatically caught
    }
});
```

### Add to Cargo.toml

```toml
# crates/hedl-core/fuzz/Cargo.toml
[[bin]]
name = "fuzz_my_feature"
path = "fuzz_targets/fuzz_my_feature.rs"
test = false
doc = false
```

### Best Practices for Targets

1. **Focus**: Test one feature or code path per target
2. **Performance**: Aim for >100K exec/sec
3. **Determinism**: Ensure same input always produces same behavior
4. **Timeout**: Add timeout checks for long-running operations
5. **Seeds**: Provide diverse seed inputs covering edge cases

---

## Coverage Analysis

### Generate Coverage Report

```bash
cd crates/hedl-core
cargo +nightly fuzz coverage fuzz_parse

# View HTML report
open fuzz/coverage/fuzz_parse/index.html
```

### Interpreting Coverage

- **Target**: >90% line coverage for critical parsers
- **Red zones**: Uncovered error paths (add seeds)
- **Hot paths**: Frequently executed code (optimize)

---

## Troubleshooting

### Slow Execution (<10K exec/sec)

**Causes**:
- Heavy computation in target
- Disk I/O in target
- Debug build

**Solutions**:
- Profile with `cargo flamegraph`
- Remove I/O operations
- Ensure release build: `CARGO_PROFILE_RELEASE_DEBUG=true cargo +nightly fuzz run`

### Out of Memory

**Causes**:
- Memory leaks
- Large allocations
- Corpus size

**Solutions**:
- Add memory limits: `cargo +nightly fuzz run <target> -- -rss_limit_mb=2048`
- Fix memory leaks
- Minimize corpus

### No New Findings

**Causes**:
- Good coverage
- Insufficient diversity
- Short runtime

**Solutions**:
- Add structured fuzzing (grammar-based)
- Increase runtime
- Add domain-specific seeds
- Use dictionary: `cargo +nightly fuzz run <target> -- -dict=fuzz/hedl.dict`

---

## Security Reporting

If fuzzing discovers a security vulnerability:

1. **Do NOT** commit the crash input to the repository
2. **Do NOT** open a public issue
3. **Email** security@dweve.com with:
   - Crash input (minimized)
   - Fuzz target name
   - Steps to reproduce
   - Impact assessment

---

## Performance Metrics

**Historical Performance** (AMD Ryzen 9 5900X, 32GB RAM):

| Target | Exec/sec | Coverage | Crashes Found |
|--------|----------|----------|---------------|
| fuzz_parse | 1.2M | 87% | 23 (all fixed) |
| fuzz_limits | 850K | 92% | 7 (all fixed) |
| fuzz_references | 620K | 89% | 12 (all fixed) |
| fuzz_nest | 730K | 85% | 5 (all fixed) |
| fuzz_matrix | 1.4M | 94% | 18 (all fixed) |

---

## References

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing best practices](https://github.com/google/fuzzing/blob/master/docs/good-fuzz-target.md)
- HEDL SPEC Section 14: Security Considerations

---

**Maintained by**: HEDL Security Team
**Questions**: Open an issue or email security@dweve.com
