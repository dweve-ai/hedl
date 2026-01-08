"""
Common test fixtures loader for HEDL Python bindings.

This module provides access to shared test fixtures stored in the
bindings/common/fixtures directory, eliminating test data duplication
across language bindings.
"""

import json
import os
from pathlib import Path
from typing import Dict, Any, Optional


class Fixtures:
    """
    Loads and provides access to common HEDL test fixtures.

    All fixtures are loaded from bindings/common/fixtures directory
    to ensure consistency across language bindings.
    """

    def __init__(self):
        """Initialize the fixtures loader and load manifest."""
        # Path to common fixtures directory
        # From bindings/python/tests/fixtures.py -> bindings/common/fixtures
        self._fixtures_dir = Path(__file__).parent.parent.parent / "common" / "fixtures"

        # Load manifest
        manifest_path = self._fixtures_dir / "manifest.json"
        with open(manifest_path, 'r', encoding='utf-8') as f:
            self._manifest = json.load(f)

    def _read_file(self, filename: str, binary: bool = False) -> str | bytes:
        """
        Read a fixture file.

        Args:
            filename: Name of the file to read
            binary: If True, read as binary, otherwise as text

        Returns:
            File contents as string or bytes
        """
        filepath = self._fixtures_dir / filename
        mode = 'rb' if binary else 'r'
        encoding = None if binary else 'utf-8'

        with open(filepath, mode, encoding=encoding) as f:
            return f.read()

    # Basic fixtures
    @property
    def basic_hedl(self) -> str:
        """Get basic HEDL sample document."""
        return self._read_file(self._manifest["fixtures"]["basic"]["files"]["hedl"])

    @property
    def basic_json(self) -> str:
        """Get basic JSON sample document."""
        return self._read_file(self._manifest["fixtures"]["basic"]["files"]["json"])

    @property
    def basic_yaml(self) -> str:
        """Get basic YAML sample document."""
        return self._read_file(self._manifest["fixtures"]["basic"]["files"]["yaml"])

    @property
    def basic_xml(self) -> str:
        """Get basic XML sample document."""
        return self._read_file(self._manifest["fixtures"]["basic"]["files"]["xml"])

    # Type-specific fixtures
    @property
    def scalars_hedl(self) -> str:
        """Get HEDL document with various scalar types."""
        return self._read_file(self._manifest["fixtures"]["scalars"]["files"]["hedl"])

    @property
    def nested_hedl(self) -> str:
        """Get HEDL document with nested structures."""
        return self._read_file(self._manifest["fixtures"]["nested"]["files"]["hedl"])

    @property
    def lists_hedl(self) -> str:
        """Get HEDL document with lists and arrays."""
        return self._read_file(self._manifest["fixtures"]["lists"]["files"]["hedl"])

    # Performance fixtures
    @property
    def large_hedl(self) -> str:
        """Get large HEDL document for performance testing."""
        return self._read_file(self._manifest["fixtures"]["large"]["files"]["hedl"])

    # Error fixtures
    @property
    def error_invalid_syntax(self) -> str:
        """Get invalid HEDL syntax for error testing."""
        return self._read_file(self._manifest["errors"]["invalid_syntax"]["file"])

    @property
    def error_malformed(self) -> str:
        """Get malformed HEDL document for error testing."""
        return self._read_file(self._manifest["errors"]["malformed"]["file"])

    # Utility methods
    def get_fixture(self, category: str, name: str, format: str = "hedl") -> str:
        """
        Get a specific fixture by category and name.

        Args:
            category: Fixture category ("basic", "scalars", etc.)
            name: Fixture name (same as category for most)
            format: File format ("hedl", "json", "yaml", "xml")

        Returns:
            Fixture content as string

        Example:
            >>> fixtures = Fixtures()
            >>> hedl = fixtures.get_fixture("basic", "basic", "hedl")
        """
        if category in self._manifest["fixtures"]:
            files = self._manifest["fixtures"][category]["files"]
            if format in files:
                return self._read_file(files[format])

        raise ValueError(f"Fixture not found: category={category}, format={format}")

    def get_error_fixture(self, error_type: str) -> str:
        """
        Get an error fixture by type.

        Args:
            error_type: Type of error ("invalid_syntax", "malformed")

        Returns:
            Error fixture content
        """
        if error_type in self._manifest["errors"]:
            return self._read_file(self._manifest["errors"][error_type]["file"])

        raise ValueError(f"Error fixture not found: {error_type}")


# Global fixtures instance for convenient access
fixtures = Fixtures()


# Legacy constants for backward compatibility
SAMPLE_HEDL = fixtures.basic_hedl
SAMPLE_JSON = fixtures.basic_json
SAMPLE_YAML = fixtures.basic_yaml
SAMPLE_XML = fixtures.basic_xml
