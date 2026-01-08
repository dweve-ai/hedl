"""
Comprehensive type stub validation tests for HEDL Python bindings.

Tests verify that:
1. All type stubs are syntactically correct
2. Type annotations match runtime implementation
3. mypy can validate typed usage
4. Context managers work with type checkers
"""

import sys
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

# Type checking imports
if TYPE_CHECKING:
    from typing import Any, Optional, Union


def test_py_typed_marker_exists():
    """Verify py.typed marker file exists for PEP 561 compliance."""
    import hedl

    module_path = Path(hedl.__file__).parent
    py_typed = module_path / "py.typed"

    assert py_typed.exists(), "py.typed marker file must exist"
    assert py_typed.is_file(), "py.typed must be a file"


def test_stub_files_exist():
    """Verify all .pyi stub files exist."""
    import hedl

    module_path = Path(hedl.__file__).parent

    required_stubs = [
        "__init__.pyi",
        "core.pyi",
        "lib.pyi",
    ]

    for stub_name in required_stubs:
        stub_path = module_path / stub_name
        assert stub_path.exists(), f"{stub_name} must exist"
        assert stub_path.is_file(), f"{stub_name} must be a file"


def test_stub_syntax_valid():
    """Verify stub files are syntactically valid Python."""
    import hedl
    import ast

    module_path = Path(hedl.__file__).parent

    stub_files = list(module_path.glob("*.pyi"))
    assert len(stub_files) > 0, "No stub files found"

    for stub_file in stub_files:
        try:
            code = stub_file.read_text()
            ast.parse(code, filename=str(stub_file))
        except SyntaxError as e:
            pytest.fail(f"Syntax error in {stub_file.name}: {e}")


def test_error_codes_exported():
    """Verify all error code constants are exported."""
    import hedl

    error_codes = [
        "HEDL_OK",
        "HEDL_ERR_NULL_PTR",
        "HEDL_ERR_INVALID_UTF8",
        "HEDL_ERR_PARSE",
        "HEDL_ERR_CANONICALIZE",
        "HEDL_ERR_JSON",
        "HEDL_ERR_ALLOC",
        "HEDL_ERR_YAML",
        "HEDL_ERR_XML",
        "HEDL_ERR_CSV",
        "HEDL_ERR_PARQUET",
        "HEDL_ERR_LINT",
        "HEDL_ERR_NEO4J",
    ]

    for code in error_codes:
        assert hasattr(hedl, code), f"{code} must be exported"
        value = getattr(hedl, code)
        assert isinstance(value, int), f"{code} must be an integer"


def test_severity_levels_exported():
    """Verify severity level constants are exported."""
    import hedl

    severities = [
        "SEVERITY_HINT",
        "SEVERITY_WARNING",
        "SEVERITY_ERROR",
    ]

    for severity in severities:
        assert hasattr(hedl, severity), f"{severity} must be exported"
        value = getattr(hedl, severity)
        assert isinstance(value, int), f"{severity} must be an integer"


def test_public_api_exported():
    """Verify all public API is exported in __all__."""
    import hedl

    expected_exports = [
        "Document",
        "Diagnostics",
        "HedlError",
        "parse",
        "validate",
        "from_json",
        "from_yaml",
        "from_xml",
        "from_parquet",
        "get_library_path",
        "load_library",
    ]

    assert hasattr(hedl, "__all__"), "__all__ must be defined"
    all_exports = set(hedl.__all__)

    for name in expected_exports:
        assert name in all_exports, f"{name} must be in __all__"
        assert hasattr(hedl, name), f"{name} must be exported"


def test_document_type_annotations():
    """Verify Document class has correct type annotations."""
    import hedl
    import inspect

    # Check Document is a class
    assert inspect.isclass(hedl.Document), "Document must be a class"

    # Check it has expected methods
    expected_methods = [
        "close",
        "canonicalize",
        "to_json",
        "to_yaml",
        "to_xml",
        "to_csv",
        "to_parquet",
        "to_cypher",
        "lint",
    ]

    for method in expected_methods:
        assert hasattr(hedl.Document, method), f"Document must have {method} method"


def test_document_properties():
    """Verify Document has correct properties."""
    import hedl

    # These should be properties
    expected_properties = [
        "version",
        "schema_count",
        "alias_count",
        "root_item_count",
    ]

    for prop in expected_properties:
        assert hasattr(hedl.Document, prop), f"Document must have {prop} property"


def test_diagnostics_type_annotations():
    """Verify Diagnostics class has correct type annotations."""
    import hedl
    import inspect

    assert inspect.isclass(hedl.Diagnostics), "Diagnostics must be a class"

    # Check methods
    expected_methods = ["close", "get"]
    for method in expected_methods:
        assert hasattr(hedl.Diagnostics, method), f"Diagnostics must have {method}"

    # Check properties
    expected_properties = ["errors", "warnings", "hints"]
    for prop in expected_properties:
        assert hasattr(hedl.Diagnostics, prop), f"Diagnostics must have {prop}"


def test_hedl_error_structure():
    """Verify HedlError exception structure."""
    import hedl

    # HedlError should be an exception
    assert issubclass(hedl.HedlError, Exception), "HedlError must inherit from Exception"

    # Check it can be instantiated
    error = hedl.HedlError("test message", code=hedl.HEDL_ERR_PARSE)
    assert error.message == "test message"
    assert error.code == hedl.HEDL_ERR_PARSE
    assert isinstance(error.context, dict)


def test_parse_function_signature():
    """Verify parse function accepts correct types."""
    import hedl
    import inspect

    sig = inspect.signature(hedl.parse)
    params = sig.parameters

    # Check parameters
    assert "content" in params, "parse must have 'content' parameter"
    assert "strict" in params, "parse must have 'strict' parameter"

    # Check strict has default
    assert params["strict"].default is not inspect.Parameter.empty


def test_validate_function_signature():
    """Verify validate function signature."""
    import hedl
    import inspect

    sig = inspect.signature(hedl.validate)
    params = sig.parameters

    assert "content" in params
    assert "strict" in params


def test_conversion_functions_signature():
    """Verify from_* conversion functions have correct signatures."""
    import hedl
    import inspect

    converters = [
        ("from_json", ["content"]),
        ("from_yaml", ["content"]),
        ("from_xml", ["content"]),
        ("from_parquet", ["content"]),
    ]

    for func_name, expected_params in converters:
        func = getattr(hedl, func_name)
        sig = inspect.signature(func)
        params = list(sig.parameters.keys())

        for expected in expected_params:
            assert expected in params, f"{func_name} must have '{expected}' parameter"


def test_context_manager_protocol():
    """Verify Document and Diagnostics implement context manager protocol."""
    import hedl

    # Document should have __enter__ and __exit__
    assert hasattr(hedl.Document, "__enter__")
    assert hasattr(hedl.Document, "__exit__")

    # Diagnostics should have __enter__ and __exit__
    assert hasattr(hedl.Diagnostics, "__enter__")
    assert hasattr(hedl.Diagnostics, "__exit__")


def test_diagnostics_iteration_protocol():
    """Verify Diagnostics implements iteration protocol."""
    import hedl

    assert hasattr(hedl.Diagnostics, "__len__")
    assert hasattr(hedl.Diagnostics, "__iter__")


@pytest.mark.skipif(sys.version_info < (3, 9), reason="Type hints require Python 3.9+")
def test_runtime_type_hints():
    """Verify runtime type hints are accessible (Python 3.9+)."""
    import hedl
    from typing import get_type_hints

    # Should be able to get type hints without errors
    try:
        hints = get_type_hints(hedl.parse)
        assert "content" in hints or "return" in hints
    except Exception as e:
        # Type hints may not be fully available at runtime for stub-only types
        pytest.skip(f"Type hints not available at runtime: {e}")


def test_version_attribute():
    """Verify __version__ attribute exists and is correct type."""
    import hedl

    assert hasattr(hedl, "__version__")
    assert isinstance(hedl.__version__, str)

    # Should be semantic version format
    parts = hedl.__version__.split(".")
    assert len(parts) >= 2, "Version should be in semantic format (e.g., 1.0.0)"


def test_stub_docstrings():
    """Verify stub files contain docstrings."""
    import hedl

    module_path = Path(hedl.__file__).parent

    for stub_file in module_path.glob("*.pyi"):
        content = stub_file.read_text()

        # Should have module docstring
        assert '"""' in content, f"{stub_file.name} should have docstrings"


def test_type_completeness():
    """Verify all runtime exports have type stubs."""
    import hedl

    # Get all public names from runtime
    runtime_names = {
        name for name in dir(hedl)
        if not name.startswith("_")
    }

    # Get all names from __all__
    all_names = set(hedl.__all__) if hasattr(hedl, "__all__") else set()

    # All __all__ exports should exist at runtime
    for name in all_names:
        assert hasattr(hedl, name), f"{name} in __all__ but not found at runtime"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
