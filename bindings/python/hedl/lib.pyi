"""
Type stubs for HEDL library loading utilities.

Provides type annotations for FFI library discovery and loading.
"""

import ctypes
from pathlib import Path
from typing import Optional

def get_library_path() -> Optional[Path]:
    """
    Find the HEDL shared library.

    Searches in order:
    1. HEDL_LIB_PATH environment variable
    2. Same directory as this module
    3. System library paths
    4. Relative to workspace root (for development)

    Returns:
        Path to the library, or None if not found
    """
    ...

def load_library(path: Optional[Path] = None) -> ctypes.CDLL:
    """
    Load the HEDL shared library.

    Args:
        path: Optional explicit path to the library.
              If None, searches standard locations.

    Returns:
        Loaded CDLL instance

    Raises:
        OSError: If the library cannot be loaded
    """
    ...
