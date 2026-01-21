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

# Error codes
HEDL_OK = 0
HEDL_ERR_NULL_PTR = -1
HEDL_ERR_INVALID_UTF8 = -2
HEDL_ERR_PARSE = -3
HEDL_ERR_CANONICALIZE = -4
HEDL_ERR_JSON = -5
HEDL_ERR_ALLOC = -6
HEDL_ERR_YAML = -7
HEDL_ERR_XML = -8
HEDL_ERR_CSV = -9
HEDL_ERR_PARQUET = -10
HEDL_ERR_LINT = -11
HEDL_ERR_NEO4J = -12
HEDL_ERR_TOON = -13
HEDL_ERR_REENTRANT_CALL = -14
HEDL_ERR_CANCELLED = -15
HEDL_ERR_QUEUE_FULL = -16
HEDL_ERR_INVALID_HANDLE = -17

# Core parsing functions
libhedl.hedl_parse.argtypes = [c_char_p, c_int, c_int, POINTER(c_void_p)]
libhedl.hedl_parse.restype = c_int

libhedl.hedl_validate.argtypes = [c_char_p, c_int, c_int]
libhedl.hedl_validate.restype = c_int

# Document information
libhedl.hedl_get_version.argtypes = [c_void_p, POINTER(c_int), POINTER(c_int)]
libhedl.hedl_get_version.restype = c_int

libhedl.hedl_schema_count.argtypes = [c_void_p]
libhedl.hedl_schema_count.restype = c_int

libhedl.hedl_alias_count.argtypes = [c_void_p]
libhedl.hedl_alias_count.restype = c_int

libhedl.hedl_root_item_count.argtypes = [c_void_p]
libhedl.hedl_root_item_count.restype = c_int

# Operations
libhedl.hedl_canonicalize.argtypes = [c_void_p, POINTER(c_char_p)]
libhedl.hedl_canonicalize.restype = c_int

libhedl.hedl_lint.argtypes = [c_void_p, POINTER(c_void_p)]
libhedl.hedl_lint.restype = c_int

# Conversion to other formats
libhedl.hedl_to_json.argtypes = [c_void_p, c_int, POINTER(c_char_p)]
libhedl.hedl_to_json.restype = c_int

libhedl.hedl_to_yaml.argtypes = [c_void_p, c_int, POINTER(c_char_p)]
libhedl.hedl_to_yaml.restype = c_int

libhedl.hedl_to_xml.argtypes = [c_void_p, POINTER(c_char_p)]
libhedl.hedl_to_xml.restype = c_int

libhedl.hedl_to_csv.argtypes = [c_void_p, POINTER(c_char_p)]
libhedl.hedl_to_csv.restype = c_int

libhedl.hedl_to_parquet.argtypes = [c_void_p, POINTER(POINTER(c_ubyte)), POINTER(c_size_t)]
libhedl.hedl_to_parquet.restype = c_int

libhedl.hedl_to_neo4j_cypher.argtypes = [c_void_p, c_int, POINTER(c_char_p)]
libhedl.hedl_to_neo4j_cypher.restype = c_int

libhedl.hedl_to_toon.argtypes = [c_void_p, POINTER(c_char_p)]
libhedl.hedl_to_toon.restype = c_int

# Conversion from other formats
libhedl.hedl_from_json.argtypes = [c_char_p, c_int, POINTER(c_void_p)]
libhedl.hedl_from_json.restype = c_int

libhedl.hedl_from_yaml.argtypes = [c_char_p, c_int, POINTER(c_void_p)]
libhedl.hedl_from_yaml.restype = c_int

libhedl.hedl_from_xml.argtypes = [c_char_p, c_int, POINTER(c_void_p)]
libhedl.hedl_from_xml.restype = c_int

libhedl.hedl_from_parquet.argtypes = [POINTER(c_ubyte), c_size_t, POINTER(c_void_p)]
libhedl.hedl_from_parquet.restype = c_int

libhedl.hedl_from_toon.argtypes = [c_char_p, c_int, POINTER(c_void_p)]
libhedl.hedl_from_toon.restype = c_int

# Error handling (thread-safe)
libhedl.hedl_get_last_error.argtypes = []
libhedl.hedl_get_last_error.restype = c_char_p

libhedl.hedl_get_last_error_threadsafe.argtypes = []
libhedl.hedl_get_last_error_threadsafe.restype = c_char_p

libhedl.hedl_clear_error_threadsafe.argtypes = []
libhedl.hedl_clear_error_threadsafe.restype = None

# Memory management
libhedl.hedl_free_document.argtypes = [c_void_p]
libhedl.hedl_free_document.restype = None

libhedl.hedl_free_string.argtypes = [c_char_p]
libhedl.hedl_free_string.restype = None

libhedl.hedl_free_bytes.argtypes = [POINTER(c_ubyte), c_size_t]
libhedl.hedl_free_bytes.restype = None

libhedl.hedl_free_diagnostics.argtypes = [c_void_p]
libhedl.hedl_free_diagnostics.restype = None

# Diagnostics functions
libhedl.hedl_diagnostics_count.argtypes = [c_void_p]
libhedl.hedl_diagnostics_count.restype = c_int

libhedl.hedl_diagnostics_get.argtypes = [c_void_p, c_int, POINTER(c_char_p)]
libhedl.hedl_diagnostics_get.restype = c_int

libhedl.hedl_diagnostics_severity.argtypes = [c_void_p, c_int]
libhedl.hedl_diagnostics_severity.restype = c_int
```

### Thread Safety

The HEDL FFI library uses **thread-local error storage**, which provides important guarantees for multi-threaded Python applications:

- Each thread maintains its own independent error state
- `hedl_get_last_error()` returns the error for the calling thread only
- Errors from one thread will never appear in another thread
- No locks or synchronization are required
- You must call error functions from the same thread that received the error code

Document handles (`HedlDocument*`) are **NOT thread-safe** and should not be shared between threads without external synchronization.

### Memory Safety

**Critical**: The `hedl_free_*` functions only accept pointers allocated by HEDL functions. Passing pointers from other sources will cause undefined behavior:

- **Safe**: Pointers returned by `hedl_parse`, `hedl_to_json`, `hedl_to_parquet`, etc.
- **Unsafe**: Pointers from `malloc`, stack-allocated variables, already-freed pointers, or other libraries
- **Safe**: NULL pointers (ignored by all free functions)

```python
# CORRECT: Free strings returned by HEDL
json_ptr = c_char_p()
libhedl.hedl_to_json(doc, 0, byref(json_ptr))
libhedl.hedl_free_string(json_ptr)  # Safe - allocated by hedl_to_json

# CORRECT: Free byte arrays with matching length
data_ptr = POINTER(c_ubyte)()
data_len = c_size_t()
libhedl.hedl_to_parquet(doc, byref(data_ptr), byref(data_len))
libhedl.hedl_free_bytes(data_ptr, data_len.value)  # Safe - allocated by hedl_to_parquet

# INCORRECT: Never free Python strings
my_string = b"test"
# libhedl.hedl_free_string(my_string)  # UNDEFINED BEHAVIOR!

# CORRECT: NULL is always safe
libhedl.hedl_free_string(None)  # No-op, safe
libhedl.hedl_free_document(None)  # No-op, safe
```

### Usage Examples

#### Basic Parsing and Conversion

```python
def parse_hedl(hedl_text: str) -> str:
    doc = c_void_p()
    code = libhedl.hedl_parse(
        hedl_text.encode('utf-8'),
        -1,  # Null-terminated string
        1,   # Strict mode
        byref(doc)
    )

    if code != HEDL_OK:
        error = libhedl.hedl_get_last_error()
        raise Exception(error.decode('utf-8') if error else "Unknown error")

    json_ptr = c_char_p()
    code = libhedl.hedl_to_json(doc, 0, byref(json_ptr))

    if code != HEDL_OK:
        libhedl.hedl_free_document(doc)
        error = libhedl.hedl_get_last_error()
        raise Exception(error.decode('utf-8') if error else "Conversion failed")

    json_str = json_ptr.value.decode('utf-8')

    libhedl.hedl_free_string(json_ptr)
    libhedl.hedl_free_document(doc)

    return json_str
```

#### Document Information

```python
def get_document_info(doc: c_void_p) -> dict:
    major = c_int()
    minor = c_int()
    libhedl.hedl_get_version(doc, byref(major), byref(minor))

    schema_count = libhedl.hedl_schema_count(doc)
    alias_count = libhedl.hedl_alias_count(doc)
    root_count = libhedl.hedl_root_item_count(doc)

    return {
        'version': f"{major.value}.{minor.value}",
        'schema_count': schema_count,
        'alias_count': alias_count,
        'root_item_count': root_count
    }
```

#### Validation and Linting

```python
def validate_hedl(hedl_text: str) -> bool:
    code = libhedl.hedl_validate(
        hedl_text.encode('utf-8'),
        -1,  # Null-terminated
        1    # Strict mode
    )
    return code == HEDL_OK

def lint_document(doc: c_void_p) -> list:
    diag = c_void_p()
    code = libhedl.hedl_lint(doc, byref(diag))

    if code != HEDL_OK:
        return []

    count = libhedl.hedl_diagnostics_count(diag)
    diagnostics = []

    for i in range(count):
        msg_ptr = c_char_p()
        libhedl.hedl_diagnostics_get(diag, i, byref(msg_ptr))

        severity = libhedl.hedl_diagnostics_severity(diag, i)

        diagnostics.append({
            'message': msg_ptr.value.decode('utf-8') if msg_ptr.value else '',
            'severity': severity
        })

    libhedl.hedl_free_diagnostics(diag)
    return diagnostics
```

#### Format Conversions

```python
def convert_json_to_yaml(json_text: str) -> str:
    # Parse JSON to HEDL
    doc = c_void_p()
    code = libhedl.hedl_from_json(json_text.encode('utf-8'), -1, byref(doc))

    if code != HEDL_OK:
        error = libhedl.hedl_get_last_error()
        raise Exception(error.decode('utf-8') if error else "JSON parse failed")

    # Convert HEDL to YAML
    yaml_ptr = c_char_p()
    code = libhedl.hedl_to_yaml(doc, 0, byref(yaml_ptr))

    if code != HEDL_OK:
        libhedl.hedl_free_document(doc)
        error = libhedl.hedl_get_last_error()
        raise Exception(error.decode('utf-8') if error else "YAML conversion failed")

    yaml_str = yaml_ptr.value.decode('utf-8')

    libhedl.hedl_free_string(yaml_ptr)
    libhedl.hedl_free_document(doc)

    return yaml_str
```

#### Working with Parquet

```python
def hedl_to_parquet_file(doc: c_void_p, output_path: str):
    data_ptr = POINTER(c_ubyte)()
    data_len = c_size_t()

    code = libhedl.hedl_to_parquet(doc, byref(data_ptr), byref(data_len))

    if code != HEDL_OK:
        error = libhedl.hedl_get_last_error()
        raise Exception(error.decode('utf-8') if error else "Parquet conversion failed")

    # Copy bytes to Python before freeing
    bytes_data = bytes(data_ptr[:data_len.value])

    # Free the C-allocated memory
    libhedl.hedl_free_bytes(data_ptr, data_len.value)

    # Write to file
    with open(output_path, 'wb') as f:
        f.write(bytes_data)

def parquet_file_to_hedl(input_path: str) -> c_void_p:
    with open(input_path, 'rb') as f:
        data = f.read()

    # Create a ctypes buffer
    buffer = (c_ubyte * len(data)).from_buffer_copy(data)

    doc = c_void_p()
    code = libhedl.hedl_from_parquet(buffer, len(data), byref(doc))

    if code != HEDL_OK:
        error = libhedl.hedl_get_last_error()
        raise Exception(error.decode('utf-8') if error else "Parquet parse failed")

    return doc
```

#### Thread-Safe Error Handling

```python
import threading

def worker_thread(hedl_text: str, results: list, index: int):
    """Each thread maintains its own error state."""
    doc = c_void_p()
    code = libhedl.hedl_parse(hedl_text.encode('utf-8'), -1, 1, byref(doc))

    if code != HEDL_OK:
        # Get error for THIS thread only
        error = libhedl.hedl_get_last_error_threadsafe()
        results[index] = {'error': error.decode('utf-8') if error else "Unknown"}
        return

    # Process document...
    json_ptr = c_char_p()
    code = libhedl.hedl_to_json(doc, 0, byref(json_ptr))

    if code == HEDL_OK:
        results[index] = {'json': json_ptr.value.decode('utf-8')}
        libhedl.hedl_free_string(json_ptr)
    else:
        error = libhedl.hedl_get_last_error_threadsafe()
        results[index] = {'error': error.decode('utf-8') if error else "Unknown"}

    libhedl.hedl_free_document(doc)

# Run multiple threads
inputs = [hedl_text1, hedl_text2, hedl_text3]
results = [None] * len(inputs)
threads = []

for i, text in enumerate(inputs):
    t = threading.Thread(target=worker_thread, args=(text, results, i))
    threads.append(t)
    t.start()

for t in threads:
    t.join()

# Each thread had independent error state
for i, result in enumerate(results):
    print(f"Thread {i}: {result}")
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
