# Python SDK Documentation

Python bindings for HEDL (if/when available).

## Status

Python bindings for HEDL are planned but not yet implemented. This page documents the intended API.

## Planned Installation

```bash
pip install hedl-python
```

## Planned API

### Basic Usage

```python
import hedl

# Parse HEDL
doc = hedl.parse(hedl_text)

# Convert to JSON
json_str = hedl.to_json(doc)

# Convert from JSON
doc = hedl.from_json(json_str)

# Validate
is_valid = hedl.validate(hedl_text)

# Lint
diagnostics = hedl.lint(doc)
for d in diagnostics:
    print(f"{d.severity}: {d.message}")
```

### Object-Oriented API

```python
from hedl import Document

# Parse
doc = Document.parse(hedl_text)

# Properties
print(doc.version)
print(doc.structs)
print(doc.root)

# Serialization
json_str = doc.to_json()
yaml_str = doc.to_yaml()
canonical = doc.canonicalize()

# Validation
errors = doc.validate()
diagnostics = doc.lint()
```

### Type Hints

```python
from typing import Dict, List, Any
from hedl import Document, Value, Item

def process_document(doc: Document) -> Dict[str, Any]:
    result: Dict[str, Any] = {}

    for item in doc.root:
        if isinstance(item, hedl.KeyValue):
            result[item.key] = item.value

    return result
```

## Alternative: FFI via ctypes

Until Python bindings are available, you can use the FFI library via ctypes:

```python
from ctypes import *

# Load library
libhedl = CDLL('./libhedl.so')

# Define functions
libhedl.hedl_parse.argtypes = [c_char_p, c_int, c_int, POINTER(c_void_p)]
libhedl.hedl_parse.restype = c_int

libhedl.hedl_to_json.argtypes = [c_void_p, c_int, POINTER(c_char_p)]
libhedl.hedl_to_json.restype = c_int

libhedl.hedl_free_document.argtypes = [c_void_p]
libhedl.hedl_free_string.argtypes = [c_char_p]

# Usage
def parse_hedl(hedl_text: str) -> str:
    doc = c_void_p()
    code = libhedl.hedl_parse(
        hedl_text.encode('utf-8'),
        -1,
        1,
        byref(doc)
    )

    if code != 0:
        error = libhedl.hedl_get_last_error()
        raise Exception(error.decode('utf-8'))

    json_ptr = c_char_p()
    code = libhedl.hedl_to_json(doc, 0, byref(json_ptr))

    if code != 0:
        libhedl.hedl_free_document(doc)
        raise Exception("Conversion failed")

    json_str = json_ptr.value.decode('utf-8')

    libhedl.hedl_free_string(json_ptr)
    libhedl.hedl_free_document(doc)

    return json_str
```

## Contributing

Interested in creating Python bindings? See:
- [FFI API Reference](../ffi-api.md)
- [Contributing Guide](../../developer/contributing.md)
- Consider using PyO3 or ctypes

## See Also

- [FFI API Reference](../ffi-api.md) - C interface
- [C/C++ SDK](c-cpp.md) - FFI examples
- [GitHub Issues](https://github.com/dweve/hedl/issues) - Request Python bindings
