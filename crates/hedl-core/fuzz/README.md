# HEDL Core Fuzzing Suite

Comprehensive fuzz testing infrastructure for the HEDL core parser using `cargo-fuzz` and libFuzzer.

## Overview

This fuzzing suite targets security-critical paths in the HEDL parser to identify:

- Memory safety violations (buffer overflows, use-after-free)
- Panics and crashes from unexpected input
- Integer overflow/underflow vulnerabilities
- Resource exhaustion (DoS) vulnerabilities
- Logic errors in parsing and validation
- Reference resolution edge cases

## Fuzz Targets

### 1. `fuzz_parse` - General Parser Fuzzing

Tests the full parsing pipeline with arbitrary input:

- Preprocessing and line splitting
- Header parsing (VERSION, TYPE, ALIAS, NEST)
- Body parsing (objects, lists, matrix rows)
- Reference resolution
- Error handling

**Run:**
```bash
cargo fuzz run fuzz_parse
```

**Focus Areas:**
- Input validation and UTF-8 handling
- Memory allocation patterns
- Recursive parsing structures
- Error propagation

### 2. `fuzz_limits` - Security Limit Enforcement

Tests that all security limits are properly enforced:

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

**Run:**
```bash
cargo fuzz run fuzz_limits
```

**Attack Scenarios Tested:**
- File size bombs
- Line length bombs
- Deep nesting bombs
- Node count bombs
- Wide schema bombs
- Block string bombs
- Key count bombs

### 3. `fuzz_references` - Reference Resolution

Tests the reference resolution system with complex graphs:

- Qualified references (`@Type:id`)
- Unqualified references (`@id`)
- Ambiguous unqualified references
- Forward references
- Self-references
- Circular reference graphs
- Missing references
- Type registry index consistency

**Run:**
```bash
cargo fuzz run fuzz_references
```

**Reference Patterns Tested:**
- Simple references
- Qualified references
- Ambiguous references
- Missing references
- Self-references
- Circular graphs
- Complex interconnected graphs
- Type collisions

### 4. `fuzz_nest_depth` - NEST Hierarchy Depth

Specifically targets NEST hierarchy depth limit enforcement:

- Deep linear NEST chains (A→B→C→D→...)
- Deep wide NEST trees (multiple children per level)
- Recursive NEST (if allowed)
- Mixed depth structures
- Boundary condition testing

**Run:**
```bash
cargo fuzz run fuzz_nest_depth
```

**Security Importance:**
- Prevents stack overflow from recursive parsing
- Prevents excessive memory allocation
- Prevents CPU exhaustion from deep traversal

## Quick Start

### Prerequisites

Install `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

### Running Fuzzers

Run a specific fuzzer:

```bash
cd crates/hedl-core
cargo fuzz run fuzz_parse
```

Run with time limit:

```bash
cargo fuzz run fuzz_parse -- -max_total_time=300  # 5 minutes
```

Run with input size limit:

```bash
cargo fuzz run fuzz_limits -- -max_len=10000  # 10KB max input
```

Run with multiple cores:

```bash
cargo fuzz run fuzz_parse -- -jobs=8
```

Run with memory limit:

```bash
cargo fuzz run fuzz_limits -- -rss_limit_mb=512
```

### Adding Seed Inputs

Add example HEDL documents to guide the fuzzer:

```bash
mkdir -p fuzz/corpus/fuzz_parse
echo "VERSION 1.0" > fuzz/corpus/fuzz_parse/simple.hedl
```

### Viewing Crashes

When the fuzzer finds a crash, it saves the input to `fuzz/artifacts/`:

```bash
ls -la fuzz/artifacts/fuzz_parse/
cat fuzz/artifacts/fuzz_parse/crash-<hash>
```

### Minimizing Crash Cases

Minimize a crash to the smallest reproducing input:

```bash
cargo fuzz cmin fuzz_parse
cargo fuzz tmin fuzz_parse fuzz/artifacts/fuzz_parse/crash-<hash>
```

## Advanced Usage

### Using Different Sanitizers

AddressSanitizer (default):
```bash
cargo fuzz run fuzz_parse --sanitizer=address
```

MemorySanitizer (requires nightly and instrumented stdlib):
```bash
cargo fuzz run fuzz_parse --sanitizer=memory
```

UndefinedBehaviorSanitizer:
```bash
cargo fuzz run fuzz_parse --sanitizer=undefined
```

### Coverage-Guided Fuzzing

Generate coverage report:

```bash
cargo fuzz coverage fuzz_parse
```

View coverage:

```bash
llvm-cov show target/x86_64-unknown-linux-gnu/release/fuzz_parse \
    -instr-profile=fuzz/coverage/fuzz_parse/coverage.profdata \
    -format=html > coverage.html
```

### Dictionary-Based Fuzzing

Create a dictionary of HEDL keywords to guide fuzzing:

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

## Continuous Fuzzing

### Local Continuous Fuzzing

Run all fuzzers in parallel:

```bash
#!/bin/bash
for target in fuzz_parse fuzz_limits fuzz_references fuzz_nest_depth; do
    cargo fuzz run $target -- -max_total_time=3600 -jobs=2 &
done
wait
```

### CI Integration

Add to GitHub Actions workflow:

```yaml
- name: Install cargo-fuzz
  run: cargo install cargo-fuzz

- name: Run fuzz tests (smoke test)
  run: |
    cd crates/hedl-core
    cargo fuzz run fuzz_parse -- -max_total_time=60 -runs=10000
    cargo fuzz run fuzz_limits -- -max_total_time=60 -runs=10000
```

## Performance Benchmarking

Measure fuzzing performance:

```bash
cargo fuzz run fuzz_parse -- -max_total_time=300 -print_final_stats=1
```

Key metrics:
- `exec/s`: Executions per second (higher is better)
- `cov`: Coverage (higher is better)
- `corp`: Corpus size (interesting inputs found)

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
- Profile the parser for bottlenecks
- Reduce max input length for faster iterations
- Use more cores: `-jobs=8`

### Reproducing Crashes

To reproduce a crash manually:

```rust
// In hedl-core/src/lib.rs tests
#[test]
fn reproduce_fuzz_crash() {
    let input = include_bytes!("../fuzz/artifacts/fuzz_parse/crash-<hash>");
    let _ = parse(input);
}
```

## Best Practices

1. **Run Regularly**: Fuzz for at least 1 hour per target weekly
2. **Monitor Coverage**: Aim for >80% line coverage
3. **Minimize Crashes**: Always minimize crash cases before filing bugs
4. **Update Corpus**: Add interesting valid inputs to corpus
5. **Version Artifacts**: Keep crash artifacts in version control (optional)

## Security Impact

Fuzzing has discovered and prevented:

- Stack overflow from deeply nested structures
- Integer overflow in node counters
- Memory exhaustion from unbounded allocations
- Panic from malformed UTF-8
- Reference resolution infinite loops

## References

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [HEDL Specification](../../docs/SPEC.md)

## License

Same as hedl-core (Apache-2.0).
