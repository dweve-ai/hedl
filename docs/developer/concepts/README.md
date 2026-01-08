# HEDL Concepts

Understanding-oriented explanations of key concepts in HEDL.

## Overview

These guides explain **why** things work the way they do, providing deep understanding of HEDL's design principles and architectural decisions.

## Available Concepts

### Architecture

- **[Parser Architecture](parser-architecture.md)**
  - Lexical analysis design
  - Recursive descent parsing
  - Two-pass reference resolution
  - Indentation-based grammar

### Design Patterns

- **[Zero-Copy Optimizations](zero-copy-optimizations.md)**
  - String slice usage
  - Pre-allocation strategies
  - Performance vs safety trade-offs

- **[AST Design](ast-design.md)**
  - Hierarchical structure
  - Typed values
  - Traversal patterns

- **[Error Handling](error-handling.md)**
  - Type-safe errors
  - Location tracking
  - Recovery strategies

## Learning Path

### For New Contributors

1. Start with [Parser Architecture](parser-architecture.md) to understand the parsing pipeline
2. Study the `hedl-core` source code for data model details
3. Advanced: [Zero-Copy Optimizations](zero-copy-optimizations.md)

### For Integration Developers

1. Review the API documentation in `hedl-core`
2. [Parser Architecture](parser-architecture.md) - Understand parsing behavior
3. Check examples in the repository

## Concept vs Tutorial vs How-To

- **Tutorials**: Learning-oriented, step-by-step practice
- **How-To Guides**: Task-oriented, solve specific problems
- **Concepts**: Understanding-oriented, explain why things work
- **Reference**: Information-oriented, technical specifications

## Design Philosophy

HEDL's design follows these principles:

1. **Token Efficiency**: Minimize structural overhead for LLM contexts (35-60% reduction vs JSON)
2. **Type Safety**: Catch errors at parse time with struct definitions
3. **Developer Ergonomics**: Human-readable, easy to write and maintain
4. **Performance**: Fast parsing optimized for both small and large documents
5. **Modularity**: Clean separation of concerns across 19 crates

## Related Documentation

- Check the main project documentation in `/docs`
- See API documentation via `cargo doc --open`
- Review source code in `crates/hedl-core`

---

**Understanding concepts** helps you make better design decisions and write better code.
