"""
Type stubs for HEDL core module.

Provides comprehensive type annotations for all HEDL operations including
parsing, validation, conversion, and diagnostics.
"""

import ctypes
from typing import Optional, Union, Iterator, Any
from types import TracebackType

# Error codes
HEDL_OK: int
HEDL_ERR_NULL_PTR: int
HEDL_ERR_INVALID_UTF8: int
HEDL_ERR_PARSE: int
HEDL_ERR_CANONICALIZE: int
HEDL_ERR_JSON: int
HEDL_ERR_ALLOC: int
HEDL_ERR_YAML: int
HEDL_ERR_XML: int
HEDL_ERR_CSV: int
HEDL_ERR_PARQUET: int
HEDL_ERR_LINT: int
HEDL_ERR_NEO4J: int

# Severity levels
SEVERITY_HINT: int
SEVERITY_WARNING: int
SEVERITY_ERROR: int

# Resource limits
MAX_OUTPUT_SIZE: int

class HedlError(Exception):
    """
    Exception raised for HEDL operations.

    Attributes:
        message: Human-readable error message
        code: HEDL error code (one of HEDL_ERR_* constants)
        context: Additional error context dictionary
    """

    message: str
    code: int
    context: dict[str, Any]

    def __init__(
        self,
        message: str,
        code: int = HEDL_ERR_PARSE,
        context: Optional[dict[str, Any]] = None
    ) -> None: ...

    @classmethod
    def from_lib(
        cls,
        code: int,
        operation: str = "HEDL operation",
        input_info: Optional[str] = None
    ) -> HedlError:
        """
        Create error from library error code with context.

        Args:
            code: HEDL error code
            operation: Description of the operation that failed
            input_info: Information about the input that caused the error

        Returns:
            Configured HedlError instance
        """
        ...

class Diagnostics:
    """
    Lint diagnostics container.

    Provides access to diagnostic messages (errors, warnings, hints) produced
    by linting a HEDL document. Diagnostics are iterable and support indexing.

    This class manages native resources and should be used as a context manager
    or explicitly closed when no longer needed.

    WARNING: Not thread-safe. See module-level documentation for details.

    Example:
        >>> with doc.lint() as diag:
        ...     for msg, severity in diag:
        ...         print(f"[{severity}] {msg}")
        ...     errors = diag.errors
        ...     warnings = diag.warnings
    """

    _lib: ctypes.CDLL
    _ptr: Any
    _closed: bool

    def __init__(self, ptr: Any) -> None: ...

    def __enter__(self) -> Diagnostics: ...

    def __exit__(
        self,
        exc_type: Optional[type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType]
    ) -> bool: ...

    def close(self) -> None:
        """Free the diagnostics handle."""
        ...

    def __del__(self) -> None: ...

    def __len__(self) -> int:
        """Return the number of diagnostics."""
        ...

    def __iter__(self) -> Iterator[tuple[str, int]]:
        """Iterate over (message, severity) tuples."""
        ...

    def get(self, index: int) -> tuple[str, int]:
        """
        Get diagnostic at index.

        Args:
            index: Diagnostic index (0-based)

        Returns:
            Tuple of (message, severity)

        Raises:
            IndexError: If index is out of range
            HedlError: If retrieval fails
        """
        ...

    @property
    def errors(self) -> list[str]:
        """Get all error messages."""
        ...

    @property
    def warnings(self) -> list[str]:
        """Get all warning messages."""
        ...

    @property
    def hints(self) -> list[str]:
        """Get all hint messages."""
        ...

class Document:
    """
    A parsed HEDL document.

    Represents a parsed HEDL document with access to metadata (version, schema
    count, etc.) and conversion methods to various formats (JSON, YAML, XML,
    CSV, Parquet, Cypher).

    This class manages native resources and should be used as a context manager
    or explicitly closed when no longer needed.

    WARNING: Not thread-safe. See module-level documentation for details.

    Example:
        >>> with hedl.parse(content) as doc:
        ...     print(doc.version)  # (major, minor) tuple
        ...     json_str = doc.to_json()
        ...     parquet_bytes = doc.to_parquet()
    """

    _lib: ctypes.CDLL
    _ptr: Any
    _closed: bool

    def __init__(self, ptr: Any) -> None: ...

    def __enter__(self) -> Document: ...

    def __exit__(
        self,
        exc_type: Optional[type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType]
    ) -> bool: ...

    def close(self) -> None:
        """Free the document handle."""
        ...

    def __del__(self) -> None: ...

    def _check_closed(self) -> None: ...

    @property
    def version(self) -> tuple[int, int]:
        """
        Get the HEDL version as (major, minor) tuple.

        Example:
            >>> doc.version
            (1, 0)

        Raises:
            HedlError: If document is closed or operation fails
        """
        ...

    @property
    def schema_count(self) -> int:
        """
        Get the number of schema definitions.

        Raises:
            HedlError: If document is closed or operation fails
        """
        ...

    @property
    def alias_count(self) -> int:
        """
        Get the number of alias definitions.

        Raises:
            HedlError: If document is closed or operation fails
        """
        ...

    @property
    def root_item_count(self) -> int:
        """
        Get the number of root items.

        Raises:
            HedlError: If document is closed or operation fails
        """
        ...

    def canonicalize(self) -> str:
        """
        Convert to canonical HEDL form.

        Returns:
            Canonicalized HEDL string

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit
        """
        ...

    def to_json(self, include_metadata: bool = False, pretty: bool = True) -> str:
        """
        Convert to JSON.

        Args:
            include_metadata: Include __type__ and __schema__ fields
            pretty: Pretty-print the JSON (currently always pretty)

        Returns:
            JSON string

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit
        """
        ...

    def to_yaml(self, include_metadata: bool = False) -> str:
        """
        Convert to YAML.

        Args:
            include_metadata: Include type metadata in output

        Returns:
            YAML string

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit
        """
        ...

    def to_xml(self) -> str:
        """
        Convert to XML.

        Returns:
            XML string

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit
        """
        ...

    def to_csv(self) -> str:
        """
        Convert to CSV.

        Note: Only works for documents with matrix lists.

        Returns:
            CSV string

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit
                       or document structure is incompatible
        """
        ...

    def to_parquet(self) -> bytes:
        """
        Convert to Parquet format.

        Note: Only works for documents with matrix lists.

        Returns:
            Parquet file contents as bytes

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit
                       or document structure is incompatible
        """
        ...

    def to_cypher(self, use_merge: bool = True) -> str:
        """
        Convert to Neo4j Cypher queries.

        Args:
            use_merge: Use MERGE (idempotent) instead of CREATE

        Returns:
            Cypher query string

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit
        """
        ...

    def lint(self) -> Diagnostics:
        """
        Run linting on the document.

        Returns:
            Diagnostics object with lint results

        Example:
            >>> with doc.lint() as diag:
            ...     for msg, severity in diag:
            ...         print(msg)
        """
        ...

def parse(content: Union[str, bytes], strict: bool = True) -> Document:
    """
    Parse HEDL content into a Document.

    Args:
        content: HEDL content as string or bytes
        strict: Enable strict reference validation

    Returns:
        Parsed Document object

    Raises:
        HedlError: If parsing fails

    Example:
        >>> doc = hedl.parse('%VERSION: 1.0\\n---\\nkey: value')
        >>> print(doc.version)
        (1, 0)
    """
    ...

def validate(content: Union[str, bytes], strict: bool = True) -> bool:
    """
    Validate HEDL content without creating a document.

    Args:
        content: HEDL content as string or bytes
        strict: Enable strict reference validation

    Returns:
        True if valid, False otherwise

    Example:
        >>> hedl.validate('%VERSION: 1.0\\n---\\nkey: value')
        True
        >>> hedl.validate('invalid content')
        False
    """
    ...

def from_json(content: Union[str, bytes]) -> Document:
    """
    Parse JSON content into a HEDL Document.

    Args:
        content: JSON content as string or bytes

    Returns:
        Parsed Document object

    Raises:
        HedlError: If parsing fails

    Example:
        >>> doc = hedl.from_json('{"key": "value"}')
        >>> print(doc.to_json())
    """
    ...

def from_yaml(content: Union[str, bytes]) -> Document:
    """
    Parse YAML content into a HEDL Document.

    Args:
        content: YAML content as string or bytes

    Returns:
        Parsed Document object

    Raises:
        HedlError: If parsing fails
    """
    ...

def from_xml(content: Union[str, bytes]) -> Document:
    """
    Parse XML content into a HEDL Document.

    Args:
        content: XML content as string or bytes

    Returns:
        Parsed Document object

    Raises:
        HedlError: If parsing fails
    """
    ...

def from_parquet(content: bytes) -> Document:
    """
    Parse Parquet content into a HEDL Document.

    Args:
        content: Parquet file contents as bytes

    Returns:
        Parsed Document object

    Raises:
        HedlError: If parsing fails
    """
    ...
