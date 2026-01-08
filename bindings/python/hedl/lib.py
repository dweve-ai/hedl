"""
Library loading utilities for HEDL FFI bindings.
"""

import ctypes
import os
import sys
from pathlib import Path
from typing import Optional

# Global library instance
_lib: Optional[ctypes.CDLL] = None


def get_library_path() -> Optional[Path]:
    """
    Find the HEDL shared library.

    Searches in order:
    1. HEDL_LIB_PATH environment variable
    2. Same directory as this module
    3. System library paths
    4. Relative to workspace root (for development)

    Returns:
        Path to the library, or None if not found.
    """
    # Platform-specific library name
    if sys.platform == "darwin":
        lib_name = "libhedl_ffi.dylib"
    elif sys.platform == "win32":
        lib_name = "hedl_ffi.dll"
    else:
        lib_name = "libhedl_ffi.so"

    # Check environment variable
    env_path = os.environ.get("HEDL_LIB_PATH")
    if env_path:
        path = Path(env_path)
        if path.is_file():
            return path
        elif path.is_dir():
            lib_path = path / lib_name
            if lib_path.exists():
                return lib_path

    # Check same directory as module
    module_dir = Path(__file__).parent
    local_lib = module_dir / lib_name
    if local_lib.exists():
        return local_lib

    # Check common build locations (for development)
    workspace_root = module_dir.parent.parent.parent
    for build_type in ["release", "debug"]:
        dev_lib = workspace_root / "target" / build_type / lib_name
        if dev_lib.exists():
            return dev_lib

    # Try system path (ctypes will search LD_LIBRARY_PATH, etc.)
    return None


def load_library(path: Optional[Path] = None) -> ctypes.CDLL:
    """
    Load the HEDL shared library.

    Args:
        path: Optional explicit path to the library.
              If None, searches standard locations.

    Returns:
        Loaded CDLL instance.

    Raises:
        OSError: If the library cannot be loaded.
    """
    global _lib

    if _lib is not None:
        return _lib

    if path is None:
        path = get_library_path()

    if path is not None:
        _lib = ctypes.CDLL(str(path))
    else:
        # Try loading by name (relies on system library paths)
        if sys.platform == "darwin":
            _lib = ctypes.CDLL("libhedl_ffi.dylib")
        elif sys.platform == "win32":
            _lib = ctypes.CDLL("hedl_ffi.dll")
        else:
            _lib = ctypes.CDLL("libhedl_ffi.so")

    _setup_function_signatures(_lib)
    return _lib


def _setup_function_signatures(lib: ctypes.CDLL) -> None:
    """Configure ctypes function signatures for type safety."""

    # Opaque pointer types
    class HedlDocument(ctypes.Structure):
        pass

    class HedlDiagnostics(ctypes.Structure):
        pass

    HedlDocumentPtr = ctypes.POINTER(HedlDocument)
    HedlDiagnosticsPtr = ctypes.POINTER(HedlDiagnostics)

    # Store types on library for access
    lib.HedlDocument = HedlDocument
    lib.HedlDiagnostics = HedlDiagnostics
    lib.HedlDocumentPtr = HedlDocumentPtr
    lib.HedlDiagnosticsPtr = HedlDiagnosticsPtr

    # Error handling
    lib.hedl_get_last_error.argtypes = []
    lib.hedl_get_last_error.restype = ctypes.c_char_p

    # Memory management
    lib.hedl_free_string.argtypes = [ctypes.c_char_p]
    lib.hedl_free_string.restype = None

    lib.hedl_free_document.argtypes = [HedlDocumentPtr]
    lib.hedl_free_document.restype = None

    lib.hedl_free_diagnostics.argtypes = [HedlDiagnosticsPtr]
    lib.hedl_free_diagnostics.restype = None

    lib.hedl_free_bytes.argtypes = [ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
    lib.hedl_free_bytes.restype = None

    # Parsing
    lib.hedl_parse.argtypes = [
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_int,
        ctypes.POINTER(HedlDocumentPtr)
    ]
    lib.hedl_parse.restype = ctypes.c_int

    lib.hedl_validate.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.c_int]
    lib.hedl_validate.restype = ctypes.c_int

    # Document info
    lib.hedl_get_version.argtypes = [
        HedlDocumentPtr,
        ctypes.POINTER(ctypes.c_int),
        ctypes.POINTER(ctypes.c_int)
    ]
    lib.hedl_get_version.restype = ctypes.c_int

    lib.hedl_schema_count.argtypes = [HedlDocumentPtr]
    lib.hedl_schema_count.restype = ctypes.c_int

    lib.hedl_alias_count.argtypes = [HedlDocumentPtr]
    lib.hedl_alias_count.restype = ctypes.c_int

    lib.hedl_root_item_count.argtypes = [HedlDocumentPtr]
    lib.hedl_root_item_count.restype = ctypes.c_int

    # Canonicalization
    lib.hedl_canonicalize.argtypes = [HedlDocumentPtr, ctypes.POINTER(ctypes.c_char_p)]
    lib.hedl_canonicalize.restype = ctypes.c_int

    # JSON
    lib.hedl_to_json.argtypes = [
        HedlDocumentPtr,
        ctypes.c_int,
        ctypes.POINTER(ctypes.c_char_p)
    ]
    lib.hedl_to_json.restype = ctypes.c_int

    lib.hedl_from_json.argtypes = [
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.POINTER(HedlDocumentPtr)
    ]
    lib.hedl_from_json.restype = ctypes.c_int

    # YAML
    lib.hedl_to_yaml.argtypes = [
        HedlDocumentPtr,
        ctypes.c_int,
        ctypes.POINTER(ctypes.c_char_p)
    ]
    lib.hedl_to_yaml.restype = ctypes.c_int

    lib.hedl_from_yaml.argtypes = [
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.POINTER(HedlDocumentPtr)
    ]
    lib.hedl_from_yaml.restype = ctypes.c_int

    # XML
    lib.hedl_to_xml.argtypes = [HedlDocumentPtr, ctypes.POINTER(ctypes.c_char_p)]
    lib.hedl_to_xml.restype = ctypes.c_int

    lib.hedl_from_xml.argtypes = [
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.POINTER(HedlDocumentPtr)
    ]
    lib.hedl_from_xml.restype = ctypes.c_int

    # CSV
    lib.hedl_to_csv.argtypes = [HedlDocumentPtr, ctypes.POINTER(ctypes.c_char_p)]
    lib.hedl_to_csv.restype = ctypes.c_int

    # Parquet
    lib.hedl_to_parquet.argtypes = [
        HedlDocumentPtr,
        ctypes.POINTER(ctypes.POINTER(ctypes.c_uint8)),
        ctypes.POINTER(ctypes.c_size_t)
    ]
    lib.hedl_to_parquet.restype = ctypes.c_int

    lib.hedl_from_parquet.argtypes = [
        ctypes.POINTER(ctypes.c_uint8),
        ctypes.c_size_t,
        ctypes.POINTER(HedlDocumentPtr)
    ]
    lib.hedl_from_parquet.restype = ctypes.c_int

    # Neo4j
    lib.hedl_to_neo4j_cypher.argtypes = [
        HedlDocumentPtr,
        ctypes.c_int,
        ctypes.POINTER(ctypes.c_char_p)
    ]
    lib.hedl_to_neo4j_cypher.restype = ctypes.c_int

    # Linting
    lib.hedl_lint.argtypes = [HedlDocumentPtr, ctypes.POINTER(HedlDiagnosticsPtr)]
    lib.hedl_lint.restype = ctypes.c_int

    lib.hedl_diagnostics_count.argtypes = [HedlDiagnosticsPtr]
    lib.hedl_diagnostics_count.restype = ctypes.c_int

    lib.hedl_diagnostics_get.argtypes = [
        HedlDiagnosticsPtr,
        ctypes.c_int,
        ctypes.POINTER(ctypes.c_char_p)
    ]
    lib.hedl_diagnostics_get.restype = ctypes.c_int

    lib.hedl_diagnostics_severity.argtypes = [HedlDiagnosticsPtr, ctypes.c_int]
    lib.hedl_diagnostics_severity.restype = ctypes.c_int
