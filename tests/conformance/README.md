# HEDL Conformance Test Suite

This directory contains test fixtures based on HEDL Specification v1.0.0 Appendix B.

## Structure

- `valid/` - Documents that MUST parse successfully
- `invalid/` - Documents that MUST produce specific errors

## Test Categories

### Valid Tests
- `minimal.hedl` - Minimum valid document
- `simple_object.hedl` - Basic key-value pairs
- `nested_object.hedl` - Nested object structure
- `matrix_list.hedl` - Basic matrix list with schema
- `references.hedl` - Forward references within type
- `type_scoped_ids.hedl` - Same ID in different type namespaces
- `ditto.hedl` - Ditto operator usage
- `nested_hierarchy.hedl` - NEST directive with parent-child
- `tensor_literals.hedl` - Tensor/array literals
- `aliases.hedl` - Alias expansion
- `all_value_types.hedl` - All scalar value types

### Invalid Tests (Expected Errors)
- `odd_indentation.hedl` - SyntaxError (3 spaces)
- `tab_indentation.hedl` - SyntaxError (tab character)
- `missing_separator.hedl` - SyntaxError (no ---)
- `missing_colon_space.hedl` - SyntaxError (a:1 not a: 1)
- `shape_mismatch.hedl` - ShapeError (wrong column count)
- `ditto_first_row.hedl` - SemanticError (^ in first row)
- `ditto_id_column.hedl` - SemanticError (^ in ID column)
- `null_id_column.hedl` - SemanticError (~ in ID column)
- `duplicate_id.hedl` - CollisionError (same ID twice)
- `unresolved_reference.hedl` - ReferenceError (strict mode)
- `unclosed_quote.hedl` - SyntaxError (truncated)

## Running Tests

All implementations must pass these tests to claim HEDL 1.0 compliance.
