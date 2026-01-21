# Property-Based Tests for hedl-core

This directory contains comprehensive property-based tests using proptest to validate invariants across thousands of generated inputs.

## Overview

**Total Properties**: 114+ individual property tests
**Total Test Cases**: 100,000+ generated per full test run (1000 cases × 100+ properties)
**Coverage**: All major hedl-core functionality including parsing, validation, NEST hierarchies, and error handling

## Test Modules

### Phase 1-2: Core Functionality (Existing)

#### `value_inference.rs` (15 properties)
Tests value type inference determinism and correctness:
- Integer roundtripping (range: -1M to 1M)
- Float parsing and precision (within epsilon)
- Boolean roundtripping
- String roundtripping
- Null value handling
- Valid key name acceptance
- Leading zero handling
- Inference determinism (same input same type)

#### `references.rs` (15 properties)
Tests reference resolution consistency:
- Valid ID acceptance
- Qualified reference parsing
- Multiple unique IDs registration
- Self-references
- Reference resolution determinism
- Cross-type references
- Circular references (allowed)
- Many-to-one references
- Nested reference resolution
- Duplicate ID detection (error case)
- Unresolved reference detection (error case)
- Forward references

#### `ditto.rs` (12 properties)
Tests ditto marker expansion correctness:
- Ditto copies all scalar types (int, string, bool, null, reference)
- Ditto in first row produces error
- Ditto chain propagation
- Partial ditto (selective column copying)
- Multiple dittos in same row
- Ditto copies previous row only
- Float value copying
- Long ditto chains (2-20 rows)
- Mixed type copying

### Phase 1-6: New Comprehensive Coverage

#### `roundtrip.rs` (18 properties)
Tests parse/serialize preservation:
- Parse determinism (same bytes same result)
- Structure preservation (nesting, types)
- Value preservation (all scalar types maintain values)
- Schema preservation (type names, field names exact)
- Version preservation
- Empty documents
- Single-character keys
- Long keys (100+ chars)
- Mixed value types
- Large documents (1000+ nodes)
- Empty lists

#### `errors.rs` (16 properties)
Tests error handling consistency:
- Error determinism (same malformed input same error)
- No panics on any input
- Error message quality (non-empty, informative)
- Missing VERSION header detection
- Malformed VERSION detection
- Duplicate key detection
- Invalid indentation handling
- Unresolved reference errors
- Duplicate ID errors
- Ditto first row errors
- Undefined struct errors
- Field count mismatch handling
- Limit violation errors (deep nesting, long lines)

#### `boundaries.rs` (22 properties)
Tests boundary conditions and limits:
- i64 MIN/MAX values
- Float boundary values (MIN_POSITIVE, EPSILON, etc.)
- Zero values (int and float)
- Empty strings
- Single-space strings
- Moderate nesting (1-10 levels)
- Wide schemas (50+ columns)
- Many object keys (100+ keys)
- Empty objects
- Empty lists
- Single-row lists
- Maximum column count (100)
- Long IDs (100+ chars)
- Many aliases (100+ aliases)
- Unicode boundaries
- NEST depth boundaries
- NEST with no children

#### `nest.rs` (8 properties)
Tests NEST hierarchy semantics:
- NEST relationship definition
- NEST requires both types declared
- Multiple NEST relationships
- NEST stored in document
- STRUCT and NEST together
- Same parent different children
- Long type names in NEST
- NEST parsing determinism

#### `block_strings.rs` (11 properties)
Tests block string handling:
- Line count preservation
- Single-line block strings
- Multi-line block strings
- Empty lines handling
- Leading spaces in content
- Trailing spaces in content
- Very long block strings (200+ lines)
- Special characters
- Unicode content
- Empty block strings
- Tabs and mixed line endings

#### `expressions.rs` (12 properties)
Tests expression and reference handling:
- Local references parse correctly
- Qualified references parse correctly
- Hyphenated IDs work
- Underscored IDs work
- Multiple references in same row
- Cross-type references
- Null references (valid)
- Very long reference IDs
- Reference parsing determinism
- Expression syntax recognition

## Invariants Tested

### Parsing
- Parse determinism: Same input always produces same output
- Never panic on any input
- Valid documents always parse
- Whitespace normalized correctly

### Type System
- Type inference is deterministic
- Type precedence respected (null > bool > number > string)
- Roundtrip preserves types
- Value types maintained exactly

### References
- Resolution is deterministic
- Self-references allowed
- Forward references allowed
- Circular references allowed
- Duplicate IDs detected
- Unresolved references detected

### NEST Hierarchies
- NEST relationships defined correctly
- Schema propagation correct
- Both types must be declared
- Multiple relationships supported

### Security
- All limits enforced (depth, width, count)
- Invalid UTF-8 handled gracefully
- Malformed input produces clear errors
- No panic on malformed input

### Error Handling
- Same error for same malformed input
- All errors have non-empty messages
- Error messages are user-facing (no internal panics)
- Deterministic error production

## Running Property Tests

```bash
# Run all property tests (1000 cases per property)
cargo test --package hedl-core --test property_tests

# Run specific module
cargo test --package hedl-core --test property_tests roundtrip

# Run with more cases for thorough testing
PROPTEST_CASES=10000 cargo test --package hedl-core --test property_tests

# Run with specific seed for reproducibility
PROPTEST_SEED=12345 cargo test --package hedl-core --test property_tests

# Run in release mode for faster execution
cargo test --package hedl-core --test property_tests --release
```

## Test Configuration

All property tests use:
- **Cases**: 1000 per property (configurable via PROPTEST_CASES)
- **Shrinking**: Automatic test case minimization on failure
- **Determinism**: Reproducible with PROPTEST_SEED
- **Timeout**: 2 minutes default per test

## Adding New Properties

1. Choose appropriate module or create new one
2. Use `proptest!` macro with descriptive test name
3. Configure test cases: `#![proptest_config(ProptestConfig::with_cases(1000))]`
4. Document the property being tested in doc comments
5. Use `prop_assert!` or `prop_assert_eq!` for assertions
6. Test with `prop_assume!` to filter out invalid inputs
7. Add property summary to this README

## Property Testing Guidelines

### Good Properties
- Test invariants, not implementations
- Generate wide range of inputs
- Use shrinking to find minimal failing cases
- Document what invariant is being tested
- Keep properties simple and focused

### Avoid
- Testing implementation details
- Over-constraining inputs (use prop_assume sparingly)
- Complex multi-step properties (split into separate tests)
- Asserting on string formatting (test structure instead)

## Coverage Goals

- [x] Parsing: 18+ properties
- [x] Type System: 15+ properties
- [x] References: 27+ properties (existing + new)
- [x] NEST: 8+ properties
- [x] Block Strings: 11+ properties
- [x] Expressions: 12+ properties
- [x] Errors: 16+ properties
- [x] Boundaries: 22+ properties

**Total**: 114+ properties covering all major functionality

## Future Enhancements

1. **Phase 7: Unicode Properties** - Comprehensive UTF-8 validation, grapheme clusters, normalization
2. **Phase 8: Timeout Properties** - Parser timeout enforcement and accuracy
3. **Phase 9: Fixture Integration** - Use hedl-test fixtures as seed corpus
4. **Phase 10: Performance Properties** - Parsing is O(n), no unbounded memory growth

5. **Custom Shrinking Strategies** - Implement shrinking for complex document structures
6. **Stateful Testing** - Test incremental parsing with stateful properties
7. **Model-Based Testing** - Build reference model and compare behaviors
8. **Differential Testing** - Compare with other HEDL parsers
9. **Concurrent Properties** - Test thread-safety if applicable

## Related Documentation

- [Property-Based Testing Guide](../../../docs/testing-property.md) (if exists)
- [Test Coverage Expansion Plan](../../../.plan/testing/hedl-core-property-test-coverage/PLAN.md)
- [proptest Documentation](https://proptest-rs.github.io/proptest/)

## Maintenance

- Run property tests in CI on every commit
- Monitor for flaky tests (use PROPTEST_SEED to reproduce)
- Update this README when adding new properties
- Review and expand coverage quarterly
- Keep test execution time under 5 minutes in CI
