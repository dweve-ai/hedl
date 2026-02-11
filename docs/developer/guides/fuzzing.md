# Fuzz Testing Guide

Security bugs hide in edge cases. The parser that handles 10,000 valid documents might crash on the 10,001st malformed one. Memory safety violations, integer overflows, resource exhaustion, injection vulnerabilities - these lurk in code paths you never thought to test.

Fuzz testing finds them. Feed random, malformed, adversarial input to your code for hours or days, and watch it discover bugs you never imagined. HEDL's fuzzing infrastructure uses cargo-fuzz and libFuzzer to test every crate that handles untrusted input.

This guide covers setting up fuzzers, running them effectively, and integrating fuzzing into your workflow.

## Prerequisites

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

Fuzzing requires nightly Rust:

```bash
rustup install nightly
# Or for a specific directory
rustup override set nightly
```

## Fuzz Targets by Crate

### hedl-core

The core parser has four fuzz targets covering security-critical paths:

| Target | Focus | Key Scenarios |
|--------|-------|---------------|
| `fuzz_parse` | General parsing | UTF-8 handling, recursive structures, error propagation |
| `fuzz_limits` | DoS protection | File size bombs, line length bombs, nesting bombs |
| `fuzz_references` | Reference resolution | Circular graphs, missing references, type collisions |
| `fuzz_nest_depth` | NEST hierarchy | Deep chains, wide trees, boundary conditions |

```bash
cd crates/hedl-core
cargo fuzz run fuzz_parse
cargo fuzz run fuzz_limits
cargo fuzz run fuzz_references
cargo fuzz run fuzz_nest_depth
```

**Security limits tested:**
- `max_file_size`: File size DoS protection
- `max_line_length`: Line length DoS protection
- `max_indent_depth`: Indentation depth DoS protection
- `max_nodes`: Node count DoS protection
- `max_aliases`: Alias count DoS protection
- `max_columns`: Schema column DoS protection
- `max_nest_depth`: NEST hierarchy DoS protection
- `max_block_string_size`: Block string DoS protection
- `max_object_keys`: Per-object key DoS protection
- `max_total_keys`: Total key DoS protection

### hedl-cli

The CLI has eight fuzz targets covering all commands:

| Target | Focus |
|--------|-------|
| `fuzz_parse` | HEDL parser with arbitrary input |
| `fuzz_format` | Format command with count hints and legacy ditto handling |
| `fuzz_convert` | All conversion operations (JSON, YAML, XML, CSV) |
| `fuzz_json_roundtrip` | HEDL to JSON to HEDL stability |
| `fuzz_yaml_roundtrip` | HEDL to YAML to HEDL stability |
| `fuzz_xml_roundtrip` | HEDL to XML to HEDL stability |
| `fuzz_stats` | Statistics generation and token estimation |
| `fuzz_lint` | Linting and diagnostic generation |

```bash
cd crates/hedl-cli
cargo fuzz run fuzz_convert
```

### hedl-stream

Streaming parser fuzzing:

| Target | Focus |
|--------|-------|
| `fuzz_streaming_parser` | Header parsing, body parsing, indentation, comments, value inference |

```bash
cd crates/hedl-stream
cargo fuzz run fuzz_streaming_parser
```

### hedl-json

JSON conversion fuzzing:

| Target | Focus |
|--------|-------|
| `fuzz_json_to_hedl` | Malformed JSON, deep nesting, large arrays, Unicode edge cases |
| `fuzz_jsonpath` | Malformed JSONPath, filter expressions, Unicode in selectors |

```bash
cd crates/hedl-json
cargo +nightly fuzz run fuzz_json_to_hedl
```

### hedl-csv

CSV parsing fuzzing:

| Target | Focus |
|--------|-------|
| `fuzz_from_csv` | Delimiters, headers, whitespace, row limits, UTF-8 |
| `fuzz_parse_value` | Type inference (null, bool, int, float, reference, expression, tensor) |

```bash
cd crates/hedl-csv
cargo +nightly fuzz run fuzz_from_csv
```

### hedl-xml

XML conversion fuzzing with security focus:

| Target | Focus |
|--------|-------|
| `fuzz_xml_to_hedl` | XXE attacks, billion laughs, DOCTYPE, deep nesting, malformed XML |
| `fuzz_hedl_to_xml` | XML escaping, deep structures, Unicode |

```bash
cd crates/hedl-xml
cargo +nightly fuzz run fuzz_xml_to_hedl
```

### hedl-yaml

YAML conversion fuzzing:

| Target | Focus |
|--------|-------|
| `fuzz_yaml_to_hedl` | YAML bombs, circular anchors, deep nesting, multi-document streams |
| `fuzz_hedl_to_yaml` | YAML escaping, key quoting, Unicode |

```bash
cd crates/hedl-yaml
cargo +nightly fuzz run fuzz_yaml_to_hedl
```

### hedl-neo4j

Cypher generation fuzzing (injection prevention):

| Target | Focus |
|--------|-------|
| `fuzz_hedl_to_cypher` | Cypher injection, property escaping, Unicode in labels |

```bash
cd crates/hedl-neo4j
cargo +nightly fuzz run fuzz_hedl_to_cypher
```

## Running Fuzzers

### Basic Usage

```bash
cd crates/hedl-core
cargo fuzz run fuzz_parse
```

### With Time Limit

```bash
# Run for 5 minutes
cargo fuzz run fuzz_parse -- -max_total_time=300

# Run for 1 hour
cargo fuzz run fuzz_limits -- -max_total_time=3600
```

### With Input Size Limit

```bash
# Limit to 10KB inputs (faster iteration)
cargo fuzz run fuzz_parse -- -max_len=10000
```

### Parallel Fuzzing

```bash
# Use 8 cores
cargo fuzz run fuzz_parse -- -jobs=8
```

### Memory Limit

```bash
cargo fuzz run fuzz_limits -- -rss_limit_mb=2048
```

### Run All Targets

```bash
for target in fuzz_parse fuzz_limits fuzz_references fuzz_nest_depth; do
    cargo fuzz run "$target" -- -max_total_time=60
done
```

## Sanitizers

### AddressSanitizer (Default)

Detects memory safety issues:

```bash
cargo fuzz run fuzz_parse --sanitizer=address
```

### MemorySanitizer

Detects uninitialized reads (requires instrumented stdlib):

```bash
cargo fuzz run fuzz_parse --sanitizer=memory
```

### UndefinedBehaviorSanitizer

```bash
cargo fuzz run fuzz_parse --sanitizer=undefined
```

## Corpus Management

### Adding Seed Inputs

```bash
mkdir -p fuzz/corpus/fuzz_parse
cp ../../../examples/*.hedl fuzz/corpus/fuzz_parse/
```

### Viewing Corpus

```bash
ls fuzz/corpus/fuzz_parse/
```

### Minimizing Corpus

```bash
cargo fuzz cmin fuzz_parse
```

### Dictionary-Based Fuzzing

Create a dictionary of HEDL keywords:

```bash
cat > fuzz/keywords.dict <<EOF
"VERSION"
"TYPE"
"ALIAS"
"NEST"
"@"
"|"
"\"\"\""
EOF

cargo fuzz run fuzz_parse -- -dict=fuzz/keywords.dict
```

## Handling Crashes

### Viewing Crashes

When the fuzzer finds a crash, it saves the input to `fuzz/artifacts/`:

```bash
ls -la fuzz/artifacts/fuzz_parse/
cat fuzz/artifacts/fuzz_parse/crash-<hash>
```

### Reproducing a Crash

```bash
cargo fuzz run fuzz_parse fuzz/artifacts/fuzz_parse/crash-<hash>
```

### Minimizing a Crash

```bash
cargo fuzz tmin fuzz_parse fuzz/artifacts/fuzz_parse/crash-<hash>
```

### Debugging a Crash

```bash
rust-lldb target/x86_64-unknown-linux-gnu/release/fuzz_parse \
    fuzz/artifacts/fuzz_parse/crash-<hash>
```

### Adding to Regression Tests

```rust
#[test]
fn reproduce_fuzz_crash() {
    let input = include_bytes!("../fuzz/artifacts/fuzz_parse/crash-<hash>");
    let _ = parse(input);
}
```

## Coverage

### Generate Coverage Report

```bash
cargo fuzz coverage fuzz_parse
```

### View Coverage

```bash
llvm-cov show target/x86_64-unknown-linux-gnu/release/fuzz_parse \
    -instr-profile=fuzz/coverage/fuzz_parse/coverage.profdata \
    -format=html > coverage.html
```

## Continuous Integration

### GitHub Actions

```yaml
- name: Install cargo-fuzz
  run: cargo install cargo-fuzz

- name: Run fuzz tests (smoke test)
  run: |
    cd crates/hedl-core
    cargo fuzz run fuzz_parse -- -max_total_time=60 -runs=10000
    cargo fuzz run fuzz_limits -- -max_total_time=60 -runs=10000
```

### Daily Fuzzing Workflow

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
      - run: |
          cd crates/hedl-core
          cargo fuzz run fuzz_parse -- -max_total_time=3600
```

### OSS-Fuzz Integration

For continuous fuzzing at scale, submit the project to [OSS-Fuzz](https://github.com/google/oss-fuzz). OSS-Fuzz runs fuzzers 24/7 and reports bugs automatically.

## Troubleshooting

### Fuzzer Not Finding Crashes

- Add seed inputs to `fuzz/corpus/`
- Use a dictionary for keyword-heavy formats
- Increase max input length: `-max_len=100000`
- Run longer: `-max_total_time=3600`

### Out of Memory

- Reduce input size: `-max_len=10000`
- Set RSS limit: `-rss_limit_mb=512`
- Reduce parallel jobs: `-jobs=1`

### Slow Fuzzing

- Check `exec/s` (should be >1000 for simple parsers)
- Reduce max input length for faster iterations
- Use more cores: `-jobs=8`

### Timeout Errors

- Reduce input complexity
- Add early returns for known slow paths
- Set timeout: `-- -timeout=10`

### "Address sanitizer failed to allocate"

Increase memory limit:

```bash
cargo fuzz run fuzz_parse -- -rss_limit_mb=4096
```

## Best Practices

### Regular Fuzzing Schedule

- **Daily**: Quick 1-minute runs of all targets (CI smoke test)
- **Weekly**: Extended 1-hour runs of critical targets
- **Pre-release**: 24-hour continuous fuzzing of all targets

### Corpus Seeding

Seed corpus with real-world examples:
- Valid HEDL documents from tests
- Edge cases discovered manually
- Examples from documentation
- Files from users (anonymized)

### Coverage Tracking

- Aim for >80% line coverage
- Monitor coverage trends over time
- Add seeds for uncovered code paths

### Issue Handling

When crashes are found:
1. Reproduce the crash
2. Minimize to smallest reproducing input
3. File an issue with the crash input
4. Add to regression test suite once fixed

### Security Disclosure

If fuzzing discovers security vulnerabilities:
1. Do not commit artifacts to version control
2. Report via security contact (see SECURITY.md)
3. Wait for patch before public disclosure

## Performance Metrics

| Crate | Target | Typical exec/s | Coverage |
|-------|--------|----------------|----------|
| hedl-core | fuzz_parse | 50,000+ | 95%+ |
| hedl-stream | fuzz_streaming_parser | 50,000+ | 95%+ |
| hedl-json | fuzz_json_to_hedl | 10,000+ | 90%+ |
| hedl-csv | fuzz_from_csv | 10,000+ | 90%+ |

Key metrics to watch:
- `exec/s`: Executions per second (higher is better)
- `cov`: Coverage (higher is better)
- `corp`: Corpus size (interesting inputs found)

## Security Impact

Fuzzing has discovered and prevented:
- Stack overflow from deeply nested structures
- Integer overflow in node counters
- Memory exhaustion from unbounded allocations
- Panic from malformed UTF-8
- Reference resolution infinite loops
- XXE and YAML bomb attacks
- Cypher injection vulnerabilities

## References

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [OSS-Fuzz](https://github.com/google/oss-fuzz)
- [Fuzzing best practices](https://google.github.io/clusterfuzz/)
