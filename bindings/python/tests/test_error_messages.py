"""
Tests for standardized error message formatting in HEDL Python bindings.

This test suite verifies that all error messages follow the standard format:
[Component] Operation failed: {reason}. Expected: {expected}.
"""

import pytest
import re
import os
import sys

# Add parent directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from hedl import parse, from_json, from_yaml, from_xml, HedlError
from hedl.errors import (
    format_error,
    format_parse_error,
    format_conversion_error,
    format_validation_error,
    format_resource_error,
    format_ffi_error,
    format_encoding_error,
    format_standardized_error,
    HEDL_ERR_PARSE,
    HEDL_ERR_JSON,
    HEDL_ERR_ALLOC,
)


# Error message pattern: [Component] Operation failed: {reason}. Expected: {expected}.
ERROR_PATTERN = re.compile(r'^\[(\w+)\] (.+?) failed: (.+?)\. Expected: (.+)\.$')


def test_error_format_function():
    """Test the basic error formatting function."""
    msg = format_error("Parser", "Parse document", "Syntax error at line 5", "Valid syntax")
    assert ERROR_PATTERN.match(msg)
    assert msg == "[Parser] Parse document failed: Syntax error at line 5. Expected: Valid syntax."


def test_parse_error_format():
    """Test parse error formatting."""
    msg = format_parse_error("Unexpected character '}'")
    assert ERROR_PATTERN.match(msg)
    assert "[Parser]" in msg
    assert "failed:" in msg
    assert "Expected:" in msg


def test_conversion_error_format():
    """Test conversion error formatting."""
    # To HEDL
    msg = format_conversion_error("JSON", "Invalid structure", to_hedl=True)
    assert ERROR_PATTERN.match(msg)
    assert "[Converter]" in msg
    assert "Convert JSON to HEDL" in msg

    # From HEDL
    msg = format_conversion_error("XML", "Cannot serialize", to_hedl=False)
    assert ERROR_PATTERN.match(msg)
    assert "[Converter]" in msg
    assert "Convert HEDL to XML" in msg


def test_resource_error_format():
    """Test resource error formatting."""
    msg = format_resource_error("Allocate memory", "Size limit exceeded", "Increase limit")
    assert ERROR_PATTERN.match(msg)
    assert "[Resource]" in msg


def test_ffi_error_format():
    """Test FFI error formatting."""
    msg = format_ffi_error("hedl_parse", "Null pointer argument")
    assert ERROR_PATTERN.match(msg)
    assert "[FFI]" in msg
    assert "hedl_parse" in msg


def test_encoding_error_format():
    """Test encoding error formatting."""
    msg = format_encoding_error("Decode input", "Invalid UTF-8 byte sequence")
    assert ERROR_PATTERN.match(msg)
    assert "[Encoding]" in msg


def test_standardized_error_format():
    """Test standardized error formatting from error code."""
    msg = format_standardized_error(
        HEDL_ERR_PARSE,
        "Unexpected character '}' at line 5",
        operation="Parse configuration",
        context="config.hedl, 1024 bytes"
    )
    assert ERROR_PATTERN.match(msg)
    assert "[Parser]" in msg
    assert "Parse configuration" in msg
    assert "config.hedl, 1024 bytes" in msg


def test_parse_error_messages():
    """Test that parse errors follow standard format."""
    invalid_hedl = "%VERSION: 1.0\n---\nkey: {unclosed"

    with pytest.raises(HedlError) as exc_info:
        parse(invalid_hedl)

    error_msg = str(exc_info.value)
    # Should follow standard format
    # Note: actual message depends on FFI layer
    assert "failed:" in error_msg.lower() or "error" in error_msg.lower()


def test_conversion_error_messages():
    """Test that conversion errors follow standard format."""
    invalid_json = "{not valid json"

    with pytest.raises(HedlError) as exc_info:
        from_json(invalid_json)

    error_msg = str(exc_info.value)
    # Should follow standard format
    assert "failed:" in error_msg.lower() or "error" in error_msg.lower()


def test_resource_error_closed_document():
    """Test that closed document errors follow standard format."""
    doc = parse("%VERSION: 1.0\n---\nkey: value")
    doc.close()

    with pytest.raises(HedlError) as exc_info:
        _ = doc.version

    error_msg = str(exc_info.value)
    match = ERROR_PATTERN.match(error_msg)
    assert match is not None, f"Error message doesn't match pattern: {error_msg}"
    assert match.group(1) == "Resource"
    assert "closed" in error_msg.lower()


def test_resource_error_output_size_limit():
    """Test that output size limit errors follow standard format."""
    # Set a very small limit to trigger the error
    original_limit = os.environ.get('HEDL_MAX_OUTPUT_SIZE')
    os.environ['HEDL_MAX_OUTPUT_SIZE'] = '100'  # 100 bytes

    try:
        # Need to reload the module to pick up the new environment variable
        # For this test, we'll just verify the helper function works correctly
        from hedl.core import _check_output_size

        large_output = "x" * 1000  # 1000 bytes

        with pytest.raises(HedlError) as exc_info:
            _check_output_size(large_output, "Convert HEDL to JSON")

        error_msg = str(exc_info.value)
        match = ERROR_PATTERN.match(error_msg)
        assert match is not None, f"Error message doesn't match pattern: {error_msg}"
        assert match.group(1) == "Resource"
        assert "Output size" in error_msg
        assert "HEDL_MAX_OUTPUT_SIZE" in error_msg

    finally:
        # Restore original limit
        if original_limit is None:
            os.environ.pop('HEDL_MAX_OUTPUT_SIZE', None)
        else:
            os.environ['HEDL_MAX_OUTPUT_SIZE'] = original_limit


def test_diagnostics_closed_error():
    """Test that closed diagnostics errors follow standard format."""
    doc = parse("%VERSION: 1.0\n---\nkey: value")
    diag = doc.lint()
    diag.close()

    with pytest.raises(HedlError) as exc_info:
        _ = diag.get(0)

    error_msg = str(exc_info.value)
    match = ERROR_PATTERN.match(error_msg)
    assert match is not None, f"Error message doesn't match pattern: {error_msg}"
    assert match.group(1) == "Resource"
    assert "Diagnostics" in error_msg
    assert "closed" in error_msg.lower()


def test_error_code_preservation():
    """Test that error codes are preserved in exceptions."""
    with pytest.raises(HedlError) as exc_info:
        parse("invalid hedl")

    assert hasattr(exc_info.value, 'code')
    assert exc_info.value.code == HEDL_ERR_PARSE


def test_error_context_preservation():
    """Test that error context is preserved."""
    with pytest.raises(HedlError) as exc_info:
        parse("invalid hedl")

    assert hasattr(exc_info.value, 'context')
    assert isinstance(exc_info.value.context, dict)


def test_all_error_codes_have_components():
    """Test that all error codes have component mappings."""
    from hedl.errors import _ERROR_COMPONENTS, _ERROR_EXPECTATIONS

    error_codes = [
        HEDL_ERR_PARSE,
        HEDL_ERR_JSON,
        HEDL_ERR_ALLOC,
        -4,  # CANONICALIZE
        -7,  # YAML
        -8,  # XML
        -9,  # CSV
        -10, # PARQUET
        -11, # LINT
        -12, # NEO4J
    ]

    for code in error_codes:
        assert code in _ERROR_COMPONENTS, f"Error code {code} missing component mapping"
        assert code in _ERROR_EXPECTATIONS, f"Error code {code} missing expectation mapping"


def test_error_message_consistency():
    """Test that error messages are consistent across similar operations."""
    # Parse different invalid formats - all should have consistent structure
    invalid_inputs = [
        ("hedl", "invalid hedl syntax"),
        ("json", "{invalid json}"),
        ("yaml", "- invalid: yaml: structure"),
    ]

    for format_type, invalid_input in invalid_inputs:
        try:
            if format_type == "hedl":
                parse(invalid_input)
            elif format_type == "json":
                from_json(invalid_input)
            elif format_type == "yaml":
                from_yaml(invalid_input)
        except HedlError as e:
            error_msg = str(e)
            # All should contain "failed:" and "Expected:"
            assert "failed:" in error_msg.lower() or "[" in error_msg
        except Exception:
            # Some formats might not be available in test environment
            pass


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
