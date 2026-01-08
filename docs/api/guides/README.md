# API Guides

Comprehensive guides for using HEDL effectively across different scenarios and platforms.

## Best Practices

- **[Rust Best Practices](rust-best-practices.md)** - Idiomatic Rust patterns for HEDL
- **[Thread Safety](thread-safety.md)** - Concurrent programming with HEDL
- **[Memory Management](memory-management.md)** - Efficient memory usage patterns
- **[Error Handling](error-handling.md)** - Robust error handling strategies

## Guide Categories

### Language-Specific Guides

#### Rust
- [Rust Best Practices](rust-best-practices.md)
  - Type safety patterns
  - Zero-cost abstractions
  - Performance optimization
  - Async/await patterns

#### C/C++
- [Thread Safety](thread-safety.md)
  - FFI thread safety
  - Thread-local storage
  - Synchronization patterns

- [Memory Management](memory-management.md)
  - FFI memory ownership
  - Resource cleanup
  - RAII patterns (C++)

#### JavaScript/TypeScript
- [WASM Integration](../tutorials/03-wasm-browser.md)
  - Browser integration
  - React/Vue patterns
  - Performance optimization

### Cross-Cutting Concerns

#### Error Handling
- [Error Handling Guide](error-handling.md)
  - Error types and categories
  - Recovery strategies
  - Logging and monitoring
  - User-friendly error messages

#### Performance
- [Rust Best Practices](rust-best-practices.md#performance)
  - Parsing optimization
  - Serialization efficiency
  - Memory allocation strategies
  - Benchmark-driven development

#### Security
- Input validation
- Resource limits
- Denial-of-service prevention
- See [Errors](../errors.md) for security considerations

## Quick Links

### For Beginners
1. Start with [Tutorials](../tutorials/README.md)
2. Read language-specific best practices
3. Review [Error Handling](error-handling.md)

### For Production Use
1. Review [Thread Safety](thread-safety.md)
2. Implement [Memory Management](memory-management.md) patterns
3. Set up [Error Handling](error-handling.md)
4. Apply [Rust Best Practices](rust-best-practices.md)

### For High-Performance Applications
1. Read [Rust Best Practices](rust-best-practices.md#performance)
2. Apply [Memory Management](memory-management.md) optimization
3. Profile and benchmark your code
4. Review [Core Types](../reference/core-types.md) for zero-cost abstractions

## Contributing

Found an issue or have a suggestion? Please open an issue or pull request on GitHub.

## Support

- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share knowledge
- **Documentation**: Browse complete API documentation
