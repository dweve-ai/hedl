"""
Type stubs for HEDL (Hierarchical Entity Data Language) Python Bindings.

This file provides complete type annotations for the HEDL Python API,
enabling static type checking with mypy and IDE autocompletion.

Example:
    >>> import hedl
    >>> doc: hedl.Document = hedl.parse('%VERSION: 1.0\\n---\\nkey: value')
    >>> version: tuple[int, int] = doc.version
    >>> json_str: str = doc.to_json()
"""

from typing import Optional, Union

# Version information
__version__: str

# Public API exports
__all__: list[str]

# Re-export types and functions from core module
from .core import (
    Document as Document,
    Diagnostics as Diagnostics,
    HedlError as HedlError,
    parse as parse,
    validate as validate,
    from_json as from_json,
    from_yaml as from_yaml,
    from_xml as from_xml,
    from_parquet as from_parquet,
    HEDL_OK as HEDL_OK,
    HEDL_ERR_NULL_PTR as HEDL_ERR_NULL_PTR,
    HEDL_ERR_INVALID_UTF8 as HEDL_ERR_INVALID_UTF8,
    HEDL_ERR_PARSE as HEDL_ERR_PARSE,
    HEDL_ERR_CANONICALIZE as HEDL_ERR_CANONICALIZE,
    HEDL_ERR_JSON as HEDL_ERR_JSON,
    HEDL_ERR_ALLOC as HEDL_ERR_ALLOC,
    HEDL_ERR_YAML as HEDL_ERR_YAML,
    HEDL_ERR_XML as HEDL_ERR_XML,
    HEDL_ERR_CSV as HEDL_ERR_CSV,
    HEDL_ERR_PARQUET as HEDL_ERR_PARQUET,
    HEDL_ERR_LINT as HEDL_ERR_LINT,
    HEDL_ERR_NEO4J as HEDL_ERR_NEO4J,
    SEVERITY_HINT as SEVERITY_HINT,
    SEVERITY_WARNING as SEVERITY_WARNING,
    SEVERITY_ERROR as SEVERITY_ERROR,
)

# Re-export lib utilities
from .lib import (
    get_library_path as get_library_path,
    load_library as load_library,
)
