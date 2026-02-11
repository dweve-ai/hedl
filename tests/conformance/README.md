# HEDL Conformance Test Suite

This directory contains test fixtures based on the HEDL Specification.

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
- `nested_hierarchy.hedl` - NEST directive with parent-child
- `tensor_literals.hedl` - Tensor/array literals
- `aliases.hedl` - Alias expansion
- `all_value_types.hedl` - All scalar value types
- `count_hints.hedl` - Count hint directives
- `count_directives.hedl` - %C count directives (total and distribution)
- `inline_children.hedl` - Inline child syntax @Type#N:|...|...
- `list_literals.hedl` - List literals (a, b, c)

### Invalid Tests (Expected Errors)
- `odd_indentation.hedl` - SyntaxError (3 spaces)
- `tab_indentation.hedl` - SyntaxError (tab character)
- `missing_separator.hedl` - SyntaxError (no ---)
- `missing_colon_space.hedl` - SyntaxError (a:1 not a: 1)
- `shape_mismatch.hedl` - ShapeError (wrong column count)
- `ditto_first_row.hedl` - SemanticError (pre-v2.0: ^ in first row)
- `ditto_id_column.hedl` - SemanticError (pre-v2.0: ^ in ID column)
- `ditto_not_allowed.hedl` - SemanticError (v2.0: ditto operator not allowed)
- `null_id_column.hedl` - SemanticError (~ in ID column)
- `duplicate_id.hedl` - CollisionError (same ID twice)
- `unresolved_reference.hedl` - ReferenceError (strict mode)
- `unclosed_quote.hedl` - SyntaxError (truncated)

## Version-Specific Features

### Ditto Operator (`^`)
The ditto operator was supported in HEDL v1.2 but is **NOT allowed in v2.0 and later**. Documents using `^` with `%V:2.0` or higher will fail validation.

- **pre-v2.0**: Ditto allowed for repeating values in matrix lists (with restrictions)
- **v2.0+**: Ditto not supported; explicit values required

## Running Tests

All implementations must pass these tests to claim conformance with their supported HEDL version.
