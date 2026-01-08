"""
Standard error message formatting for HEDL Python bindings.

This module provides utilities for creating consistent, actionable error messages
that follow the HEDL standard format:

    [Component] Operation failed: {reason}. Expected: {expected}.
"""

from typing import Optional, Dict


# Error code constants (from FFI)
HEDL_OK = 0
HEDL_ERR_NULL_PTR = -1
HEDL_ERR_INVALID_UTF8 = -2
HEDL_ERR_PARSE = -3
HEDL_ERR_CANONICALIZE = -4
HEDL_ERR_JSON = -5
HEDL_ERR_ALLOC = -6
HEDL_ERR_YAML = -7
HEDL_ERR_XML = -8
HEDL_ERR_CSV = -9
HEDL_ERR_PARQUET = -10
HEDL_ERR_LINT = -11
HEDL_ERR_NEO4J = -12


# Error code to component mapping
_ERROR_COMPONENTS: Dict[int, str] = {
    HEDL_ERR_NULL_PTR: "Resource",
    HEDL_ERR_INVALID_UTF8: "Encoding",
    HEDL_ERR_PARSE: "Parser",
    HEDL_ERR_CANONICALIZE: "Canonicalizer",
    HEDL_ERR_JSON: "Converter",
    HEDL_ERR_ALLOC: "Resource",
    HEDL_ERR_YAML: "Converter",
    HEDL_ERR_XML: "Converter",
    HEDL_ERR_CSV: "Converter",
    HEDL_ERR_PARQUET: "Converter",
    HEDL_ERR_LINT: "Linter",
    HEDL_ERR_NEO4J: "Converter",
}


# Error code to expected behavior mapping
_ERROR_EXPECTATIONS: Dict[int, str] = {
    HEDL_ERR_NULL_PTR: "Document or resource must be open before operations",
    HEDL_ERR_INVALID_UTF8: "Valid UTF-8 encoded input",
    HEDL_ERR_PARSE: "Valid HEDL syntax without unclosed strings or invalid characters",
    HEDL_ERR_CANONICALIZE: "HEDL document without circular references or undefined symbols",
    HEDL_ERR_JSON: "Valid JSON structure compatible with HEDL data types",
    HEDL_ERR_ALLOC: "Increase HEDL_MAX_OUTPUT_SIZE environment variable or reduce document size",
    HEDL_ERR_YAML: "Valid YAML structure compatible with HEDL data types",
    HEDL_ERR_XML: "Valid XML structure with proper element names and attributes",
    HEDL_ERR_CSV: "HEDL document containing matrix lists for tabular data",
    HEDL_ERR_PARQUET: "HEDL document with matrix lists and consistent column types",
    HEDL_ERR_LINT: "Valid HEDL document structure for analysis",
    HEDL_ERR_NEO4J: "HEDL document with graph-compatible structure (nodes and relationships)",
}


def format_error(component: str, operation: str, reason: str, expected: str) -> str:
    """
    Format an error message according to HEDL standard format.

    Args:
        component: The component that failed (Parser, Converter, etc.)
        operation: The operation that failed (parsing, conversion, etc.)
        reason: Detailed explanation of what went wrong
        expected: What was expected or how to fix the issue

    Returns:
        Formatted error message.

    Example:
        >>> format_error("Parser", "Parse HEDL document", "Unexpected character '}' at line 5",
        ...              "Valid HEDL syntax")
        "[Parser] Parse HEDL document failed: Unexpected character '}' at line 5. Expected: Valid HEDL syntax."
    """
    return f"[{component}] {operation} failed: {reason}. Expected: {expected}."


def get_component_for_error_code(code: int) -> str:
    """Get the component name for an error code."""
    return _ERROR_COMPONENTS.get(code, "HEDL")


def get_expectation_for_error_code(code: int) -> str:
    """Get the expected behavior for an error code."""
    return _ERROR_EXPECTATIONS.get(code, "Valid input and proper usage")


def format_parse_error(detail: str, context: Optional[str] = None) -> str:
    """
    Format a parse error message.

    Args:
        detail: Specific parse error detail from FFI
        context: Optional context (file name, input size, etc.)

    Returns:
        Formatted parse error message.
    """
    operation = "Parse HEDL document"
    if context:
        operation += f" ({context})"

    return format_error(
        component="Parser",
        operation=operation,
        reason=detail,
        expected="Valid HEDL syntax without unclosed strings or invalid characters"
    )


def format_conversion_error(format_name: str, detail: str, to_hedl: bool = False) -> str:
    """
    Format a conversion error message.

    Args:
        format_name: Name of the format (JSON, YAML, XML, etc.)
        detail: Specific conversion error detail
        to_hedl: If True, converting FROM format TO HEDL; if False, vice versa

    Returns:
        Formatted conversion error message.
    """
    if to_hedl:
        operation = f"Convert {format_name} to HEDL"
        expected = f"Valid {format_name} structure compatible with HEDL"
    else:
        operation = f"Convert HEDL to {format_name}"
        expected = f"HEDL document compatible with {format_name} format"

    return format_error(
        component="Converter",
        operation=operation,
        reason=detail,
        expected=expected
    )


def format_validation_error(detail: str, requirement: str) -> str:
    """
    Format a validation error message.

    Args:
        detail: Specific validation error detail
        requirement: What is required for validation

    Returns:
        Formatted validation error message.
    """
    return format_error(
        component="Validator",
        operation="Validate HEDL document",
        reason=detail,
        expected=f"{requirement}. See documentation for details"
    )


def format_resource_error(operation: str, detail: str, requirement: str) -> str:
    """
    Format a resource error message (memory, document access, etc.).

    Args:
        operation: The operation that failed
        detail: What went wrong
        requirement: What is required

    Returns:
        Formatted resource error message.
    """
    return format_error(
        component="Resource",
        operation=operation,
        reason=detail,
        expected=requirement
    )


def format_ffi_error(function: str, detail: str) -> str:
    """
    Format an FFI call error message.

    Args:
        function: The FFI function that failed
        detail: What went wrong

    Returns:
        Formatted FFI error message.
    """
    return format_error(
        component="FFI",
        operation=f"Call to '{function}'",
        reason=detail,
        expected="Valid FFI library setup and compatible data types"
    )


def format_encoding_error(operation: str, detail: str) -> str:
    """
    Format an encoding error message.

    Args:
        operation: The encoding operation that failed
        detail: What went wrong

    Returns:
        Formatted encoding error message.
    """
    return format_error(
        component="Encoding",
        operation=operation,
        reason=detail,
        expected="Valid UTF-8 encoded input"
    )


def format_standardized_error(
    code: int,
    ffi_detail: str,
    operation: Optional[str] = None,
    context: Optional[str] = None
) -> str:
    """
    Format a standardized error message from FFI error code and detail.

    This is the primary function for creating error messages from FFI layer errors.
    It automatically determines the component and expected behavior based on the
    error code.

    Args:
        code: HEDL error code (e.g., HEDL_ERR_PARSE)
        ffi_detail: Error detail from FFI layer
        operation: Optional operation description (auto-generated if not provided)
        context: Optional context information

    Returns:
        Formatted error message following HEDL standard.

    Example:
        >>> format_standardized_error(
        ...     HEDL_ERR_PARSE,
        ...     "Unexpected character '}' at line 5",
        ...     operation="Parse configuration file",
        ...     context="config.hedl, 1024 bytes"
        ... )
        "[Parser] Parse configuration file failed: Unexpected character '}' at line 5. Expected: Valid HEDL syntax without unclosed strings or invalid characters."
    """
    component = get_component_for_error_code(code)
    expected = get_expectation_for_error_code(code)

    # Auto-generate operation if not provided
    if operation is None:
        operation = _get_default_operation(code)

    # Add context to operation if provided
    if context:
        operation = f"{operation} ({context})"

    return format_error(component, operation, ffi_detail, expected)


def _get_default_operation(code: int) -> str:
    """Get default operation name for an error code."""
    operations = {
        HEDL_ERR_NULL_PTR: "Access resource",
        HEDL_ERR_INVALID_UTF8: "Decode input",
        HEDL_ERR_PARSE: "Parse HEDL document",
        HEDL_ERR_CANONICALIZE: "Canonicalize document",
        HEDL_ERR_JSON: "Convert JSON",
        HEDL_ERR_ALLOC: "Allocate memory",
        HEDL_ERR_YAML: "Convert YAML",
        HEDL_ERR_XML: "Convert XML",
        HEDL_ERR_CSV: "Convert CSV",
        HEDL_ERR_PARQUET: "Convert Parquet",
        HEDL_ERR_LINT: "Lint document",
        HEDL_ERR_NEO4J: "Convert to Cypher",
    }
    return operations.get(code, "HEDL operation")
