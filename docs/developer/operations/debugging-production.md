# Production Debugging

Debug HEDL issues in production environments.

## Debug Symbols

Build with debug symbols:

```bash
cargo build --release
# Debug symbols in target/release/
```

## Crash Analysis

Use backtrace:

```bash
RUST_BACKTRACE=1 ./target/release/hedl parse file.hedl
```

## Performance Profiling

```bash
# Linux: perf
perf record ./target/release/hedl parse large.hedl
perf report

# macOS: Instruments
instruments -t "Time Profiler" ./target/release/hedl
```

## Logging

Enable debug logging:

```rust
use tracing::info;

info!("Parsing document of size {}", input.len());
```

Run with:
```bash
RUST_LOG=debug cargo run
```

## Related

- [Debug Parser](../how-to/debug-parser.md)
- [Profile Performance](../how-to/profile-performance.md)
