# HEDL Conformance Test Suite

Comprehensive test suite for SPEC 1.0 compliance validation.

---

## Overview

The HEDL Conformance Test Suite verifies complete compliance with the **HEDL 1.0 Specification** (SPEC.md). These tests are derived from **Appendix B: Conformance Test Suite** and ensure that all SPEC requirements are correctly implemented.

### Purpose

- **Validate SPEC compliance**: Ensure parser behavior matches specification
- **Prevent regressions**: Catch spec violations in CI/CD
- **Enable certification**: Third-party implementations can verify conformance
- **Document behavior**: Tests serve as executable specification examples

---

## Quick Start

### Running Conformance Tests

```bash
# Run all conformance tests
cargo test --package hedl-core conformance

# Run specific conformance test category
cargo test --package hedl-core conformance::syntax
cargo test --package hedl-core conformance::schema
cargo test --package hedl-core conformance::data

# Run with detailed output
cargo test --package hedl-core conformance -- --nocapture

# Run specific test
cargo test --package hedl-core test_odd_indentation_error
```

### Conformance Report

```bash
# Generate conformance report
cargo test --package hedl-core conformance -- --format json > conformance-report.json

# Count passing tests
cargo test --package hedl-core conformance 2>&1 | grep -E "test result: ok"
```

---

## Test Organization

### Location

**File**: `crates/hedl-core/tests/conformance_tests.rs`
**Modules**:
- B.1: Syntax Validation (18 tests)
- B.2: Schema Validation (5 tests)
- B.3: Data Validation (12 tests)
- B.4: Reference Validation (7 tests)
- B.5: Parsing Correctness (10 tests)
- B.6: Edge Cases & Truncation (11 tests)
- B.7: Full Test Document (1 test)

**Total**: 82 conformance tests (run `cargo test --package hedl-core --test conformance_tests` to verify)

---

## Test Categories

### B.1: Syntax Validation (18 tests)

**Purpose**: Verify lexical and structural SPEC rules

**Tests**:
1. **test_odd_indentation_error** - SPEC § 5.1
   - Requirement: Indentation must be multiple of 2 spaces
   - Input: `---\na:\n   b: 1  # 3 spaces`
   - Expected: `SyntaxError`

2. **test_tab_indentation_error** - SPEC § 5.1
   - Requirement: Tabs not allowed for indentation
   - Input: `---\na:\n\tb: 1`
   - Expected: `SyntaxError`

3. **test_missing_separator_error** - SPEC § 4.1
   - Requirement: `---` separator required between header and body
   - Input: `%VERSION: 1.0\na: 1`
   - Expected: `SyntaxError`

4. **test_multiple_separators_error** - SPEC § 4.1
   - Requirement: Only one `---` separator allowed
   - Input: `---\na: 1\n---\nb: 2`
   - Expected: `SyntaxError`

5. **test_missing_space_after_colon_error** - SPEC § 5.3
   - Requirement: Space required after `:` in key-value pairs
   - Input: `a:1`
   - Expected: `SyntaxError`

6. **test_valid_id_uppercase_ok** - SPEC § 6.1 (Updated)
   - Requirement: IDs can contain uppercase (e.g., `SKU-4020`)
   - Input: `| SKU-4020, test`
   - Expected: Success

7. **test_invalid_reference_starts_digit_error** - SPEC § 6.1
   - Requirement: IDs cannot start with digit
   - Input: `| 123User, test`
   - Expected: `SemanticError`

8. **test_control_character_error** - SPEC § 8.1
   - Requirement: Control characters not allowed (except tab in strings)
   - Input: `a: test\x01value`
   - Expected: `SyntaxError`

9. **test_bare_cr_error** - SPEC § 5.2
   - Requirement: Line endings must be LF or CRLF, not bare CR
   - Input: `%VERSION: 1.0\r---\r`
   - Expected: `SyntaxError`

10-18. **Additional syntax tests**: Block strings, escaping, whitespace, etc.

---

### B.2: Schema Validation (5 tests)

**Purpose**: Verify schema and type system rules

**Tests**:
1. **test_unknown_type_error** - SPEC § 7.1
   - Requirement: Types must be defined before use
   - Input: `data: @UnknownType`
   - Expected: `SchemaError`

2. **test_schema_mismatch_error** - SPEC § 7.2
   - Requirement: Schema must match %STRUCT definition
   - Input: `%STRUCT: User: [id,name,email]\nusers: @User[id, name]`
   - Expected: `SchemaError`

3. **test_duplicate_struct_different_columns_error** - SPEC § 7.1
   - Requirement: Cannot redefine struct with different schema
   - Input: `%STRUCT: User: [id,name]\n%STRUCT: User: [id, email]`
   - Expected: `SchemaError`

4. **test_nest_undefined_type_error** - SPEC § 7.4
   - Requirement: %NEST child type must be defined
   - Input: `%NEST: User > UndefinedType`
   - Expected: `SchemaError`

5. **test_duplicate_struct_identical_columns_ok** - SPEC § 7.1
   - Requirement: Idempotent struct definitions allowed
   - Input: `%STRUCT: User: [id,name]\n%STRUCT: User: [id,name]`
   - Expected: Success

---

### B.3: Data Validation (12 tests)

**Purpose**: Verify data correctness and semantic rules

**Tests**:
1. **test_shape_mismatch_error** - SPEC § 9.1
   - Requirement: Row field count must match schema
   - Input: `%STRUCT: User: [id,name,email]\n| u1, Alice  # Missing email`
   - Expected: `ShapeError`

2. **test_first_row_ditto_error** - SPEC § 9.3
   - Requirement: Cannot use ditto (`^`) in first row
   - Input: `| x, ^`
   - Expected: `SemanticError`

3. **test_orphan_child_row_error** - SPEC § 9.5
   - Requirement: Child rows require %NEST directive
   - Input: Parent row followed by indented child without %NEST
   - Expected: `OrphanRowError`

4. **test_duplicate_id_collision_error** - SPEC § 6.2
   - Requirement: IDs must be unique within type
   - Input: `| u1, Alice\n| u1, Bob`
   - Expected: `CollisionError`

5. **test_different_id_across_types_ok** - SPEC § 6.2
   - Requirement: Same ID allowed in different types
   - Input: User type with id="admin", Role type with id="admin"
   - Expected: Success

6. **test_invalid_id_type_number_error** - SPEC § 6.1
   - Requirement: IDs must be strings, not numbers
   - Input: `| 123, test`
   - Expected: `SemanticError`

7. **test_ditto_in_id_column_error** - SPEC § 9.3
   - Requirement: Ditto not allowed in ID column
   - Input: `| a, 1\n| ^, 2`
   - Expected: `SemanticError`

8. **test_null_in_id_column_error** - SPEC § 6.1
   - Requirement: IDs cannot be null
   - Input: `| ~, test`
   - Expected: `SemanticError`

9-12. **Additional data tests**: ID formats, valid/invalid patterns

---

### B.4: Reference Validation (7 tests)

**Purpose**: Verify reference resolution rules

**Tests**:
1. **test_forward_reference_ok** - SPEC § 10.1
   - Requirement: Forward references allowed
   - Input: `| t1, @t2\n| t2, ~`
   - Expected: Success

2. **test_missing_reference_error** - SPEC § 10.1
   - Requirement: References must resolve
   - Input: `| t1, @missing`
   - Expected: `ReferenceError`

3. **test_self_reference_ok** - SPEC § 10.1
   - Requirement: Self-references allowed
   - Input: `| t1, @t1`
   - Expected: Success

4. **test_circular_reference_ok** - SPEC § 10.1
   - Requirement: Circular references allowed
   - Input: `| t1, @t2\n| t2, @t1`
   - Expected: Success

5. **test_qualified_reference_ok** - SPEC § 10.2
   - Requirement: Qualified references (`@Type:id`) supported
   - Input: `| p1, @User:u1`
   - Expected: Success

6. **test_unqualified_reference_scoped_to_current_type** - SPEC § 10.2-10.3
   - Requirement: Unqualified refs in matrix search current type only
   - Input: Same ID in User and Role, Post references @admin
   - Expected: `ReferenceError` (not found in Post)

7. **test_ambiguous_unqualified_reference_error** - SPEC § 10.3.1
   - Requirement: Unqualified refs in key-value must be unambiguous
   - Input: Same ID in User and Role, key-value uses @admin
   - Expected: `ReferenceError` (ambiguous)

---

### B.5: Parsing Correctness (10 tests)

**Purpose**: Verify parsing algorithm correctness

**Tests**:
1. **test_ditto_scoping** - SPEC § 9.3
   - Verifies ditto doesn't cross list boundaries

2. **test_child_attachment** - SPEC § 7.4
   - Verifies NEST child-parent attachment

3. **test_alias_expansion** - SPEC § 3.2
   - Verifies alias substitution

4. **test_hash_in_quoted_field** - SPEC § 9.2
   - Verifies `#` is data within quotes

5. **test_matrix_row_comment_stripped** - SPEC § 5.4
   - Verifies comments removed before CSV parse

6. **test_quoted_string_escaping** - SPEC § 8.1
   - Verifies `""` → `"` escaping

7. **test_number_inference** - SPEC § 8.3
   - Verifies int vs float inference

8. **test_tensor_literal** - SPEC § 8.4
   - Verifies tensor parsing

9. **test_at_and_dollar_in_strings** - SPEC § 8.1
   - Verifies `@` and `$` are literal mid-string

10. **test_elastic_alignment** - SPEC § 9.2
   - Verifies extra whitespace for visual alignment

---

### B.6: Edge Cases & Truncation (11 tests)

**Purpose**: Verify edge case handling

**Tests**:
1. **test_empty_document_ok** - SPEC § 4.1
2. **test_empty_matrix_list_ok** - SPEC § 9.1
3. **test_object_start_with_comment** - SPEC § 5.4
4. **test_empty_alias** - SPEC § 3.2
5. **test_whitespace_preservation** - SPEC § 8.1
6. **test_boolean_case_sensitivity** - SPEC § 8.3
7. **test_expression_nested_call** - SPEC § 8.5
8. **test_unclosed_quote_error** - SPEC § 8.1
9. **test_tab_in_quoted_string_ok** - SPEC § 8.1
10. **test_crlf_line_endings_ok** - SPEC § 5.2
11. **test_spec_14_5_truncated_object_detected** - SPEC § 14.5

---

### B.7: Full Test Document (1 test)

**Purpose**: Complete integration test

**Test**: **test_conformance_document**
- Parses full conformance document from SPEC Appendix B
- Verifies:
  - 4 test rows with correct values
  - Alias expansion (`%true` → `true`)
  - Reference values (`@t1`, `@t2`)
  - Ditto values (row 4 copies row 3)
  - NEST children (2 rows under parents)
  - Tensor literals (`[1,2,3]`, `[[1,2],[3,4]]`)

---

## Verification Workflow

### 1. Running Tests

```bash
# Full conformance suite
cargo test --package hedl-core conformance

# Expected output:
# test conformance_tests::test_odd_indentation_error ... ok
# test conformance_tests::test_tab_indentation_error ... ok
# ...
# test result: ok. 82 passed; 0 failed
```

### 2. Verifying SPEC Compliance

Each test includes SPEC section references:

```rust
/// B.1.1: Odd indentation -> Syntax Error (SPEC § 5.1)
#[test]
fn test_odd_indentation_error() {
    let doc = "%VERSION: 1.0\n---\na:\n   b: 1\n"; // 3 spaces
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}
```

### 3. Adding New Tests

When SPEC is updated:

1. Add test to appropriate section (B.1-B.7)
2. Reference SPEC section in doc comment
3. Use descriptive test name
4. Verify expected error kind matches SPEC
5. Run full suite to ensure no regressions

---

## Conformance Certification

### Third-Party Implementations

To certify SPEC compliance:

1. **Implement parser** following SPEC.md
2. **Run conformance suite** against implementation
3. **Pass all conformance tests**
4. **Report results** with:
   - Implementation name and version
   - Test results (JSON format)
   - SPEC version targeted
   - Any known limitations

### Example Certificate

```
HEDL Conformance Certificate

Implementation: hedl-rs v1.0.0
SPEC Version: 1.0

Results:
Run `cargo test --package hedl-core --test conformance_tests` for current counts.

Total: All conformance tests passed
```

---

## CI Integration

### GitHub Actions

```yaml
# .github/workflows/conformance.yml
name: SPEC Conformance

on: [push, pull_request]

jobs:
  conformance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run conformance tests
        run: cargo test --package hedl-core conformance
      - name: Generate conformance report
        run: cargo test --package hedl-core conformance -- --format json > conformance-report.json
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: conformance-report
          path: conformance-report.json
```

---

## Troubleshooting

### Test Failure

1. **Read error message**: Error kind should match expected
2. **Check SPEC**: Verify expected behavior in SPEC.md
3. **Minimal reproduction**: Isolate failing input
4. **Debug parser**: Add logging to parser code
5. **Compare with reference**: Check hedl-rs implementation

### Adding Missing Tests

If SPEC section lacks test:

1. **Identify requirement**: Find normative SPEC language ("must", "shall")
2. **Create test case**: Minimal input demonstrating behavior
3. **Verify both paths**: Test success and failure cases
4. **Document**: Reference SPEC section in comment

---

## References

- **SPEC.md**: Full specification (Sections 1-19, Appendices A-H)
- **Appendix B**: Conformance Test Suite (source of these tests)
- **hedl-core/tests/conformance_tests.rs**: Test implementation
- **Section 14**: Security considerations
- **Section 12**: Error handling requirements

---

## Maintenance

**Review Schedule**:
- **On SPEC updates**: Update tests within 1 week
- **Quarterly**: Review test coverage vs SPEC
- **Annually**: Audit for missing edge cases

**Contact**:
- Issues: GitHub issues
- SPEC clarifications: spec@dweve.com
- Certification: conformance@dweve.com
