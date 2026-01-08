"""
HEDL (Hierarchical Entity Data Language) Python Bindings

A token-efficient data format for LLM context optimization.

Example:
    >>> import hedl
    >>> doc = hedl.parse('%VERSION: 1.0\\n---\\nkey: value')
    >>> print(doc.version)
    (1, 0)
    >>> json_str = doc.to_json()
"""

from .core import (
    Document,
    Diagnostics,
    HedlError,
    parse,
    validate,
    from_json,
    from_yaml,
    from_xml,
    from_parquet,
)

from .lib import get_library_path, load_library

__version__ = "1.0.0"
__all__ = [
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
