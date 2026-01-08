# How-To Guides

Practical, task-oriented guides for common development tasks.

## Overview

These guides answer the question "How do I...?" with step-by-step instructions for specific tasks.

## Available Guides

### Debugging & Troubleshooting

- **[Debug Parser Issues](debug-parser.md)**
  - Diagnose parsing failures
  - Use tracing and logging
  - Understand error messages
  - Debug lexer and parser separately

### Performance

- **[Profile Performance](profile-performance.md)**
  - Use cargo-flamegraph
  - Analyze criterion benchmarks
  - Identify bottlenecks
  - Compare performance across versions

### Development

- **[Add Benchmarks](add-benchmarks.md)**
  - Create new benchmark suites
  - Use criterion effectively
  - Generate performance reports
  - Track regression


## Guide Structure

Each guide follows this format:

1. **Goal**: What you want to accomplish
2. **Prerequisites**: What you need before starting
3. **Steps**: Numbered, actionable instructions
4. **Verification**: How to confirm success
5. **Troubleshooting**: Common issues and solutions
6. **Related**: Links to relevant documentation

## Quick Reference

| Task | Guide | Time |
|------|-------|------|
| Fix parsing error | [Debug Parser](debug-parser.md) | 10 min |
| Find performance bottleneck | [Profile Performance](profile-performance.md) | 20 min |
| Add performance test | [Add Benchmarks](add-benchmarks.md) | 15 min |
| Create C bindings | [Write FFI Bindings](write-ffi-bindings.md) | 30 min |

## Contributing Guides

Have a useful technique? Add a how-to guide:

1. Create a new `.md` file in `docs/developer/how-to/`
2. Follow the standard guide structure
3. Include working code examples
4. Add verification steps
5. Submit a pull request

## Related Documentation

- [Tutorials](../tutorials/README.md) - Learning-oriented lessons
- [Concepts](../concepts/README.md) - Understanding-oriented explanations
- [Reference](../reference/README.md) - Information-oriented specifications

---

**Need help?** Ask in [GitHub Discussions](https://github.com/dweve-ai/hedl/discussions)
