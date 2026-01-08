"""
Core HEDL Python bindings.

THREAD SAFETY WARNING:
=====================
These bindings are NOT thread-safe. Document and Diagnostics objects must
not be accessed concurrently from multiple threads. The underlying FFI
library does not perform any internal locking. If you need concurrent
access to HEDL documents, you must:

1. Use separate Document instances per thread, OR
2. Implement your own synchronization (threading.Lock, etc.) around all
   Document and Diagnostics method calls

Concurrent access without proper synchronization may result in:
- Memory corruption
- Use-after-free errors
- Segmentation faults
- Undefined behavior

Python's GIL (Global Interpreter Lock) does NOT protect against these
issues because FFI calls release the GIL during execution.

RESOURCE LIMITS:
===============
The HEDL_MAX_OUTPUT_SIZE environment variable controls the maximum size of
output from conversion operations (to_json, to_yaml, to_xml, etc.).

Default: 100 MB (conservative, may be too restrictive for many use cases)
Recommended for data processing: 500 MB - 1 GB
For large datasets: 1 GB - 5 GB

Set before importing hedl:

    # In your shell (before running Python)
    export HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB

    # Or in Python (must be set BEFORE import)
    import os
    os.environ['HEDL_MAX_OUTPUT_SIZE'] = '1073741824'  # 1 GB
    import hedl

Use Cases:
- Small configs: 10-50 MB (default may suffice)
- Medium datasets: 100-500 MB (set to 524288000 for 500 MB)
- Large datasets: 500 MB - 5 GB (set to 1073741824+ for 1 GB+)
- No practical limit: set to a very high value like 10737418240 (10 GB)

When the limit is exceeded, operations will raise HedlError with code
HEDL_ERR_ALLOC and a message suggesting to increase HEDL_MAX_OUTPUT_SIZE.
"""

import ctypes
import os
from typing import Optional, Tuple, List, Union
from .lib import load_library
from .errors import (
    format_standardized_error,
    format_parse_error,
    format_conversion_error,
    format_resource_error,
    HEDL_OK,
    HEDL_ERR_NULL_PTR,
    HEDL_ERR_INVALID_UTF8,
    HEDL_ERR_PARSE,
    HEDL_ERR_CANONICALIZE,
    HEDL_ERR_JSON,
    HEDL_ERR_ALLOC,
    HEDL_ERR_YAML,
    HEDL_ERR_XML,
    HEDL_ERR_CSV,
    HEDL_ERR_PARQUET,
    HEDL_ERR_LINT,
    HEDL_ERR_NEO4J,
)


# Resource limits
# Default is 100MB, which may be too restrictive for many real-world scenarios.
# Recommended: 500MB-1GB for data processing, higher for large datasets.
# Set HEDL_MAX_OUTPUT_SIZE environment variable before importing to customize.
MAX_OUTPUT_SIZE = int(os.getenv('HEDL_MAX_OUTPUT_SIZE', '104857600'))  # 100MB default

# Severity levels
SEVERITY_HINT = 0
SEVERITY_WARNING = 1
SEVERITY_ERROR = 2


class HedlError(Exception):
    """
    Exception raised for HEDL operations.

    All HEDL error messages follow the standard format:
    [Component] Operation failed: {reason}. Expected: {expected}.

    Attributes:
        message: Formatted error message
        code: HEDL error code (e.g., HEDL_ERR_PARSE)
        context: Additional context dictionary
    """

    def __init__(self, message: str, code: int = HEDL_ERR_PARSE, context: Optional[dict] = None):
        self.message = message
        self.code = code
        self.context = context or {}
        super().__init__(message)

    @classmethod
    def from_lib(cls, code: int, operation: str = "HEDL operation", context: Optional[str] = None) -> "HedlError":
        """
        Create error from library error code using standard formatting.

        Args:
            code: HEDL error code from FFI layer
            operation: Operation that failed (e.g., "Parse HEDL document")
            context: Optional context (e.g., "config.hedl, 1024 bytes")

        Returns:
            HedlError with standardized message format.
        """
        lib = load_library()
        error_msg = lib.hedl_get_last_error()
        if error_msg:
            ffi_detail = error_msg.decode("utf-8")
        else:
            ffi_detail = f"Unknown error (code {code})"

        # Use standardized error formatting
        message = format_standardized_error(code, ffi_detail, operation, context)

        return cls(message, code, {"operation": operation, "context": context, "ffi_detail": ffi_detail})


def _check_output_size(output: Union[str, bytes], operation: str) -> None:
    """
    Check if output size exceeds MAX_OUTPUT_SIZE limit.

    Args:
        output: Output string or bytes to check
        operation: Operation name for error message

    Raises:
        HedlError: If output exceeds size limit
    """
    if isinstance(output, str):
        output_size = len(output.encode('utf-8'))
    else:
        output_size = len(output)

    if output_size > MAX_OUTPUT_SIZE:
        actual_mb = output_size / 1048576
        limit_mb = MAX_OUTPUT_SIZE / 1048576
        detail = f"Output size ({actual_mb:.2f}MB) exceeds limit ({limit_mb:.2f}MB)"
        requirement = "Increase HEDL_MAX_OUTPUT_SIZE environment variable or reduce document size"
        raise HedlError(
            format_resource_error(operation, detail, requirement),
            HEDL_ERR_ALLOC
        )


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
    """

    def __init__(self, ptr):
        self._lib = load_library()
        self._ptr = ptr
        self._closed = False

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
        return False

    def close(self) -> None:
        """Free the diagnostics handle."""
        if not self._closed and self._ptr:
            self._lib.hedl_free_diagnostics(self._ptr)
            self._closed = True

    def __del__(self):
        self.close()

    def __len__(self) -> int:
        """Return the number of diagnostics."""
        if self._closed:
            return 0
        count = self._lib.hedl_diagnostics_count(self._ptr)
        return max(0, count)

    def __iter__(self):
        """Iterate over (message, severity) tuples."""
        for i in range(len(self)):
            yield self.get(i)

    def get(self, index: int) -> Tuple[str, int]:
        """
        Get diagnostic at index.

        Args:
            index: Diagnostic index (0-based).

        Returns:
            Tuple of (message, severity).

        Raises:
            IndexError: If index is out of range.
            HedlError: If retrieval fails.
        """
        if self._closed:
            raise HedlError(
                format_resource_error(
                    "Access diagnostics",
                    "Diagnostics already closed",
                    "Diagnostics must be open before access"
                ),
                HEDL_ERR_NULL_PTR
            )

        if index < 0 or index >= len(self):
            raise IndexError(f"Diagnostic index {index} out of range")

        msg_ptr = ctypes.c_char_p()
        result = self._lib.hedl_diagnostics_get(
            self._ptr, index, ctypes.byref(msg_ptr)
        )

        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Get diagnostic message", f"index {index}")

        message = msg_ptr.value.decode("utf-8") if msg_ptr.value else ""
        self._lib.hedl_free_string(msg_ptr)

        severity = self._lib.hedl_diagnostics_severity(self._ptr, index)
        return (message, severity)

    @property
    def errors(self) -> List[str]:
        """Get all error messages."""
        return [msg for msg, sev in self if sev == SEVERITY_ERROR]

    @property
    def warnings(self) -> List[str]:
        """Get all warning messages."""
        return [msg for msg, sev in self if sev == SEVERITY_WARNING]

    @property
    def hints(self) -> List[str]:
        """Get all hint messages."""
        return [msg for msg, sev in self if sev == SEVERITY_HINT]


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

    def __init__(self, ptr):
        self._lib = load_library()
        self._ptr = ptr
        self._closed = False

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
        return False

    def close(self) -> None:
        """Free the document handle."""
        if not self._closed and self._ptr:
            self._lib.hedl_free_document(self._ptr)
            self._closed = True

    def __del__(self):
        self.close()

    def _check_closed(self) -> None:
        if self._closed:
            raise HedlError(
                format_resource_error(
                    "Access document",
                    "Document already closed",
                    "Document must be open before operations"
                ),
                HEDL_ERR_NULL_PTR
            )

    @property
    def version(self) -> Tuple[int, int]:
        """
        Get the HEDL version as (major, minor) tuple.

        Example:
            >>> doc.version
            (1, 0)
        """
        self._check_closed()
        major = ctypes.c_int()
        minor = ctypes.c_int()
        result = self._lib.hedl_get_version(
            self._ptr, ctypes.byref(major), ctypes.byref(minor)
        )
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Get HEDL version")
        return (major.value, minor.value)

    @property
    def schema_count(self) -> int:
        """Get the number of schema definitions."""
        self._check_closed()
        count = self._lib.hedl_schema_count(self._ptr)
        if count < 0:
            raise HedlError.from_lib(count, "Get schema count")
        return count

    @property
    def alias_count(self) -> int:
        """Get the number of alias definitions."""
        self._check_closed()
        count = self._lib.hedl_alias_count(self._ptr)
        if count < 0:
            raise HedlError.from_lib(count, "Get alias count")
        return count

    @property
    def root_item_count(self) -> int:
        """Get the number of root items."""
        self._check_closed()
        count = self._lib.hedl_root_item_count(self._ptr)
        if count < 0:
            raise HedlError.from_lib(count, "Get root item count")
        return count

    def canonicalize(self) -> str:
        """
        Convert to canonical HEDL form.

        Returns:
            Canonicalized HEDL string.

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit.
        """
        self._check_closed()
        out_ptr = ctypes.c_char_p()
        result = self._lib.hedl_canonicalize(self._ptr, ctypes.byref(out_ptr))
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Canonicalize HEDL document")
        output = out_ptr.value.decode("utf-8") if out_ptr.value else ""
        self._lib.hedl_free_string(out_ptr)
        _check_output_size(output, "Canonicalize HEDL document")
        return output

    def to_json(self, include_metadata: bool = False, pretty: bool = True) -> str:
        """
        Convert to JSON.

        Args:
            include_metadata: Include __type__ and __schema__ fields.
            pretty: Pretty-print the JSON (currently always pretty).

        Returns:
            JSON string.

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit.
        """
        self._check_closed()
        out_ptr = ctypes.c_char_p()
        result = self._lib.hedl_to_json(
            self._ptr, 1 if include_metadata else 0, ctypes.byref(out_ptr)
        )
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Convert HEDL to JSON")
        output = out_ptr.value.decode("utf-8") if out_ptr.value else ""
        self._lib.hedl_free_string(out_ptr)
        _check_output_size(output, "Convert HEDL to JSON")
        return output

    def to_yaml(self, include_metadata: bool = False) -> str:
        """
        Convert to YAML.

        Args:
            include_metadata: Include type metadata in output.

        Returns:
            YAML string.

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit.
        """
        self._check_closed()
        out_ptr = ctypes.c_char_p()
        result = self._lib.hedl_to_yaml(
            self._ptr, 1 if include_metadata else 0, ctypes.byref(out_ptr)
        )
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Convert HEDL to YAML")
        output = out_ptr.value.decode("utf-8") if out_ptr.value else ""
        self._lib.hedl_free_string(out_ptr)
        _check_output_size(output, "Convert HEDL to YAML")
        return output

    def to_xml(self) -> str:
        """
        Convert to XML.

        Returns:
            XML string.

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit.
        """
        self._check_closed()
        out_ptr = ctypes.c_char_p()
        result = self._lib.hedl_to_xml(self._ptr, ctypes.byref(out_ptr))
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Convert HEDL to XML")
        output = out_ptr.value.decode("utf-8") if out_ptr.value else ""
        self._lib.hedl_free_string(out_ptr)
        _check_output_size(output, "Convert HEDL to XML")
        return output

    def to_csv(self) -> str:
        """
        Convert to CSV.

        Note: Only works for documents with matrix lists.

        Returns:
            CSV string.

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit.
        """
        self._check_closed()
        out_ptr = ctypes.c_char_p()
        result = self._lib.hedl_to_csv(self._ptr, ctypes.byref(out_ptr))
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Convert HEDL to CSV")
        output = out_ptr.value.decode("utf-8") if out_ptr.value else ""
        self._lib.hedl_free_string(out_ptr)
        _check_output_size(output, "Convert HEDL to CSV")
        return output

    def to_parquet(self) -> bytes:
        """
        Convert to Parquet format.

        Note: Only works for documents with matrix lists.

        Returns:
            Parquet file contents as bytes.

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit.
        """
        self._check_closed()
        data_ptr = ctypes.POINTER(ctypes.c_uint8)()
        data_len = ctypes.c_size_t()
        result = self._lib.hedl_to_parquet(
            self._ptr, ctypes.byref(data_ptr), ctypes.byref(data_len)
        )
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Convert HEDL to Parquet")

        # Copy bytes before freeing
        output = bytes(data_ptr[:data_len.value])
        self._lib.hedl_free_bytes(data_ptr, data_len)
        _check_output_size(output, "Convert HEDL to Parquet")
        return output

    def to_cypher(self, use_merge: bool = True) -> str:
        """
        Convert to Neo4j Cypher queries.

        Args:
            use_merge: Use MERGE (idempotent) instead of CREATE.

        Returns:
            Cypher query string.

        Raises:
            HedlError: If output size exceeds HEDL_MAX_OUTPUT_SIZE limit.
        """
        self._check_closed()
        out_ptr = ctypes.c_char_p()
        result = self._lib.hedl_to_neo4j_cypher(
            self._ptr, 1 if use_merge else 0, ctypes.byref(out_ptr)
        )
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Convert HEDL to Neo4j Cypher")
        output = out_ptr.value.decode("utf-8") if out_ptr.value else ""
        self._lib.hedl_free_string(out_ptr)
        _check_output_size(output, "Convert HEDL to Neo4j Cypher")
        return output

    def lint(self) -> Diagnostics:
        """
        Run linting on the document.

        Returns:
            Diagnostics object with lint results.

        Example:
            >>> with doc.lint() as diag:
            ...     for msg, severity in diag:
            ...         print(msg)
        """
        self._check_closed()
        diag_ptr = self._lib.HedlDiagnosticsPtr()
        result = self._lib.hedl_lint(self._ptr, ctypes.byref(diag_ptr))
        if result != HEDL_OK:
            raise HedlError.from_lib(result, "Lint HEDL document")
        return Diagnostics(diag_ptr)


def parse(content: Union[str, bytes], strict: bool = True) -> Document:
    """
    Parse HEDL content into a Document.

    Args:
        content: HEDL content as string or bytes.
        strict: Enable strict reference validation.

    Returns:
        Parsed Document object.

    Raises:
        HedlError: If parsing fails.

    Example:
        >>> doc = hedl.parse('%VERSION: 1.0\\n---\\nkey: value')
        >>> print(doc.version)
        (1, 0)
    """
    lib = load_library()

    if isinstance(content, str):
        content = content.encode("utf-8")

    doc_ptr = lib.HedlDocumentPtr()
    result = lib.hedl_parse(
        content, len(content), 1 if strict else 0, ctypes.byref(doc_ptr)
    )

    if result != HEDL_OK:
        input_info = f"{len(content)} bytes"
        raise HedlError.from_lib(result, "Parse HEDL document", input_info)

    return Document(doc_ptr)


def validate(content: Union[str, bytes], strict: bool = True) -> bool:
    """
    Validate HEDL content without creating a document.

    Args:
        content: HEDL content as string or bytes.
        strict: Enable strict reference validation.

    Returns:
        True if valid, False otherwise.

    Example:
        >>> hedl.validate('%VERSION: 1.0\\n---\\nkey: value')
        True
        >>> hedl.validate('invalid content')
        False
    """
    lib = load_library()

    if isinstance(content, str):
        content = content.encode("utf-8")

    result = lib.hedl_validate(content, len(content), 1 if strict else 0)
    return result == HEDL_OK


def from_json(content: Union[str, bytes]) -> Document:
    """
    Parse JSON content into a HEDL Document.

    Args:
        content: JSON content as string or bytes.

    Returns:
        Parsed Document object.

    Raises:
        HedlError: If parsing fails.

    Example:
        >>> doc = hedl.from_json('{"key": "value"}')
        >>> print(doc.to_json())
    """
    lib = load_library()

    if isinstance(content, str):
        content = content.encode("utf-8")

    doc_ptr = lib.HedlDocumentPtr()
    result = lib.hedl_from_json(content, len(content), ctypes.byref(doc_ptr))

    if result != HEDL_OK:
        input_info = f"{len(content)} bytes of JSON"
        raise HedlError.from_lib(result, "Parse JSON to HEDL", input_info)

    return Document(doc_ptr)


def from_yaml(content: Union[str, bytes]) -> Document:
    """
    Parse YAML content into a HEDL Document.

    Args:
        content: YAML content as string or bytes.

    Returns:
        Parsed Document object.

    Raises:
        HedlError: If parsing fails.
    """
    lib = load_library()

    if isinstance(content, str):
        content = content.encode("utf-8")

    doc_ptr = lib.HedlDocumentPtr()
    result = lib.hedl_from_yaml(content, len(content), ctypes.byref(doc_ptr))

    if result != HEDL_OK:
        input_info = f"{len(content)} bytes of YAML"
        raise HedlError.from_lib(result, "Parse YAML to HEDL", input_info)

    return Document(doc_ptr)


def from_xml(content: Union[str, bytes]) -> Document:
    """
    Parse XML content into a HEDL Document.

    Args:
        content: XML content as string or bytes.

    Returns:
        Parsed Document object.

    Raises:
        HedlError: If parsing fails.
    """
    lib = load_library()

    if isinstance(content, str):
        content = content.encode("utf-8")

    doc_ptr = lib.HedlDocumentPtr()
    result = lib.hedl_from_xml(content, len(content), ctypes.byref(doc_ptr))

    if result != HEDL_OK:
        input_info = f"{len(content)} bytes of XML"
        raise HedlError.from_lib(result, "Parse XML to HEDL", input_info)

    return Document(doc_ptr)


def from_parquet(content: bytes) -> Document:
    """
    Parse Parquet content into a HEDL Document.

    Args:
        content: Parquet file contents as bytes.

    Returns:
        Parsed Document object.

    Raises:
        HedlError: If parsing fails.
    """
    lib = load_library()

    # Create a ctypes array from bytes
    data_array = (ctypes.c_uint8 * len(content))(*content)

    doc_ptr = lib.HedlDocumentPtr()
    result = lib.hedl_from_parquet(data_array, len(content), ctypes.byref(doc_ptr))

    if result != HEDL_OK:
        input_info = f"{len(content)} bytes of Parquet data"
        raise HedlError.from_lib(result, "Parse Parquet to HEDL", input_info)

    return Document(doc_ptr)
