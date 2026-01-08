"""
Production-grade type-safe usage examples for HEDL Python bindings.

This module demonstrates best practices for using HEDL with comprehensive
type annotations, proper error handling, and resource management.

Run mypy to validate types:
    mypy examples/typed_usage.py --strict
"""

import os
from pathlib import Path
from typing import Optional, Union, cast

import hedl


# =============================================================================
# Type-Safe Parsing and Validation
# =============================================================================


def parse_hedl_strict(content: Union[str, bytes]) -> hedl.Document:
    """
    Parse HEDL content with strict validation and type checking.

    Args:
        content: HEDL content as string or bytes

    Returns:
        Parsed Document instance

    Raises:
        hedl.HedlError: If parsing fails
    """
    try:
        doc: hedl.Document = hedl.parse(content, strict=True)
        return doc
    except hedl.HedlError as e:
        print(f"Parse error [{e.code}]: {e.message}")
        if e.context:
            print(f"Context: {e.context}")
        raise


def validate_hedl_content(content: Union[str, bytes]) -> bool:
    """
    Validate HEDL content without creating a document.

    Args:
        content: HEDL content to validate

    Returns:
        True if valid, False otherwise
    """
    is_valid: bool = hedl.validate(content, strict=True)
    return is_valid


# =============================================================================
# Context Manager Pattern (Recommended)
# =============================================================================


def process_hedl_with_context(hedl_content: str) -> dict[str, int]:
    """
    Process HEDL document using context manager for automatic cleanup.

    Args:
        hedl_content: HEDL document as string

    Returns:
        Dictionary with document statistics
    """
    stats: dict[str, int] = {}

    with hedl.parse(hedl_content) as doc:
        version: tuple[int, int] = doc.version
        stats["version_major"] = version[0]
        stats["version_minor"] = version[1]
        stats["schema_count"] = doc.schema_count
        stats["alias_count"] = doc.alias_count
        stats["root_item_count"] = doc.root_item_count

    return stats


# =============================================================================
# Format Conversions with Type Safety
# =============================================================================


def convert_to_json(doc: hedl.Document, pretty: bool = True) -> str:
    """
    Convert HEDL document to JSON with type annotations.

    Args:
        doc: HEDL document
        pretty: Enable pretty printing

    Returns:
        JSON string representation
    """
    json_output: str = doc.to_json(include_metadata=False, pretty=pretty)
    return json_output


def convert_to_parquet(doc: hedl.Document) -> bytes:
    """
    Convert HEDL document to Parquet format.

    Args:
        doc: HEDL document with matrix list structure

    Returns:
        Parquet file contents as bytes

    Raises:
        hedl.HedlError: If document structure is incompatible
    """
    try:
        parquet_data: bytes = doc.to_parquet()
        return parquet_data
    except hedl.HedlError as e:
        if e.code == hedl.HEDL_ERR_CSV:
            print("Document must contain matrix lists for Parquet conversion")
        raise


def convert_all_formats(doc: hedl.Document) -> dict[str, Union[str, bytes]]:
    """
    Convert document to all supported formats.

    Args:
        doc: HEDL document

    Returns:
        Dictionary mapping format names to converted content
    """
    formats: dict[str, Union[str, bytes]] = {
        "canonical": doc.canonicalize(),
        "json": doc.to_json(),
        "yaml": doc.to_yaml(),
        "xml": doc.to_xml(),
    }

    # Optional formats that may not work for all documents
    try:
        formats["csv"] = doc.to_csv()
    except hedl.HedlError:
        formats["csv"] = ""

    try:
        formats["parquet"] = doc.to_parquet()
    except hedl.HedlError:
        formats["parquet"] = b""

    try:
        formats["cypher"] = doc.to_cypher(use_merge=True)
    except hedl.HedlError:
        formats["cypher"] = ""

    return formats


# =============================================================================
# Linting and Diagnostics
# =============================================================================


def lint_document(doc: hedl.Document) -> tuple[list[str], list[str], list[str]]:
    """
    Lint HEDL document and categorize diagnostics.

    Args:
        doc: HEDL document to lint

    Returns:
        Tuple of (errors, warnings, hints)
    """
    with doc.lint() as diag:
        errors: list[str] = diag.errors
        warnings: list[str] = diag.warnings
        hints: list[str] = diag.hints

        # Iterate over all diagnostics
        for message, severity in diag:
            severity_str: str
            if severity == hedl.SEVERITY_ERROR:
                severity_str = "ERROR"
            elif severity == hedl.SEVERITY_WARNING:
                severity_str = "WARNING"
            elif severity == hedl.SEVERITY_HINT:
                severity_str = "HINT"
            else:
                severity_str = "UNKNOWN"

            print(f"[{severity_str}] {message}")

    return (errors, warnings, hints)


def validate_with_diagnostics(content: str) -> bool:
    """
    Validate and lint HEDL content, reporting all issues.

    Args:
        content: HEDL content to validate

    Returns:
        True if no errors found, False otherwise
    """
    if not hedl.validate(content):
        print("Validation failed - invalid HEDL syntax")
        return False

    with hedl.parse(content) as doc:
        errors, warnings, hints = lint_document(doc)

        print(f"Linting results: {len(errors)} errors, "
              f"{len(warnings)} warnings, {len(hints)} hints")

        return len(errors) == 0


# =============================================================================
# Format Parsing with Type Safety
# =============================================================================


def parse_json_to_hedl(json_content: Union[str, bytes]) -> hedl.Document:
    """
    Parse JSON content into HEDL document.

    Args:
        json_content: JSON content as string or bytes

    Returns:
        HEDL Document

    Raises:
        hedl.HedlError: If parsing fails
    """
    doc: hedl.Document = hedl.from_json(json_content)
    return doc


def parse_yaml_to_hedl(yaml_content: Union[str, bytes]) -> hedl.Document:
    """
    Parse YAML content into HEDL document.

    Args:
        yaml_content: YAML content as string or bytes

    Returns:
        HEDL Document
    """
    doc: hedl.Document = hedl.from_yaml(yaml_content)
    return doc


def parse_xml_to_hedl(xml_content: Union[str, bytes]) -> hedl.Document:
    """
    Parse XML content into HEDL document.

    Args:
        xml_content: XML content as string or bytes

    Returns:
        HEDL Document
    """
    doc: hedl.Document = hedl.from_xml(xml_content)
    return doc


def parse_parquet_to_hedl(parquet_data: bytes) -> hedl.Document:
    """
    Parse Parquet file into HEDL document.

    Args:
        parquet_data: Parquet file contents as bytes

    Returns:
        HEDL Document
    """
    doc: hedl.Document = hedl.from_parquet(parquet_data)
    return doc


# =============================================================================
# Resource Management and Error Handling
# =============================================================================


def safe_document_processing(hedl_content: str) -> Optional[str]:
    """
    Demonstrate robust error handling and resource cleanup.

    Args:
        hedl_content: HEDL document content

    Returns:
        JSON output if successful, None otherwise
    """
    doc: Optional[hedl.Document] = None

    try:
        # Parse document
        doc = hedl.parse(hedl_content, strict=True)

        # Check version compatibility
        major, minor = doc.version
        if major != 1:
            print(f"Unsupported version: {major}.{minor}")
            return None

        # Convert to JSON
        json_output: str = doc.to_json(include_metadata=True)
        return json_output

    except hedl.HedlError as e:
        # Handle specific error codes
        error_handlers: dict[int, str] = {
            hedl.HEDL_ERR_PARSE: "Syntax error in HEDL",
            hedl.HEDL_ERR_INVALID_UTF8: "Invalid UTF-8 encoding",
            hedl.HEDL_ERR_JSON: "JSON conversion failed",
            hedl.HEDL_ERR_ALLOC: "Memory allocation error - output too large",
        }

        error_msg: str = error_handlers.get(e.code, f"Unknown error: {e.code}")
        print(f"Error: {error_msg}")

        if e.code == hedl.HEDL_ERR_ALLOC:
            print("Suggestion: Increase HEDL_MAX_OUTPUT_SIZE environment variable")

        return None

    finally:
        # Explicit cleanup (context manager preferred)
        if doc is not None:
            doc.close()


# =============================================================================
# Environment Configuration
# =============================================================================


def configure_output_limit(size_mb: int) -> None:
    """
    Configure maximum output size for HEDL operations.

    Must be called before importing hedl module for first time.

    Args:
        size_mb: Maximum output size in megabytes
    """
    size_bytes: int = size_mb * 1024 * 1024
    os.environ['HEDL_MAX_OUTPUT_SIZE'] = str(size_bytes)
    print(f"Configured HEDL_MAX_OUTPUT_SIZE to {size_mb}MB")


# =============================================================================
# Example Usage
# =============================================================================


def main() -> None:
    """Demonstrate type-safe HEDL usage."""

    # Configure for large datasets (must be done before import in production)
    # configure_output_limit(500)  # 500 MB

    # Sample HEDL content
    hedl_content: str = """%VERSION: 1.0
---
person:
  name: "John Doe"
  age: 30
  email: "john@example.com"
"""

    print("=== Type-Safe HEDL Processing ===\n")

    # Validate content
    print("1. Validation:")
    is_valid: bool = validate_hedl_content(hedl_content)
    print(f"   Valid: {is_valid}\n")

    # Parse and get statistics
    print("2. Document Statistics:")
    stats: dict[str, int] = process_hedl_with_context(hedl_content)
    for key, value in stats.items():
        print(f"   {key}: {value}")
    print()

    # Convert to formats
    print("3. Format Conversions:")
    with hedl.parse(hedl_content) as doc:
        json_output: str = convert_to_json(doc)
        print(f"   JSON length: {len(json_output)} chars")

        yaml_output: str = doc.to_yaml()
        print(f"   YAML length: {len(yaml_output)} chars")

        xml_output: str = doc.to_xml()
        print(f"   XML length: {len(xml_output)} chars")
    print()

    # Lint document
    print("4. Linting:")
    with hedl.parse(hedl_content) as doc:
        errors, warnings, hints = lint_document(doc)
        print(f"   Errors: {len(errors)}")
        print(f"   Warnings: {len(warnings)}")
        print(f"   Hints: {len(hints)}")
    print()

    # Round-trip conversion
    print("5. Round-trip Conversion:")
    with hedl.parse(hedl_content) as doc:
        json_str: str = doc.to_json(include_metadata=True)

    # Parse JSON back to HEDL
    with hedl.from_json(json_str) as doc2:
        version: tuple[int, int] = doc2.version
        print(f"   Restored version: {version}")
        canonical: str = doc2.canonicalize()
        print(f"   Canonical length: {len(canonical)} chars")


if __name__ == "__main__":
    main()
