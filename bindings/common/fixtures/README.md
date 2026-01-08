# Common Test Fixtures

This directory contains shared test fixtures used across all HEDL language bindings to ensure consistency and eliminate duplication.

## Overview

The common fixtures approach provides:

- **Single Source of Truth**: All test data maintained in one location
- **Consistency**: Identical test cases across Python, Ruby, Node.js, Go, PHP, and C#
- **DRY Principle**: Eliminates duplication of test data across 6 language bindings
- **Easy Maintenance**: Update once, apply everywhere
- **Comprehensive Coverage**: Scalars, nested structures, lists, errors, and performance tests

## Fixture Files

### Basic Fixtures

- `sample_basic.hedl` - Basic HEDL document with struct and table data
- `sample_basic.json` - Equivalent JSON representation
- `sample_basic.yaml` - Equivalent YAML representation
- `sample_basic.xml` - Equivalent XML representation

### Type-Specific Fixtures

- `sample_scalars.hedl` - Various scalar types (string, int, float, bool, null)
- `sample_nested.hedl` - Nested structures with multiple struct types
- `sample_lists.hedl` - Lists and arrays with various nesting levels

### Performance Fixtures

- `sample_large.hedl` - Large document for performance and stress testing

### Error Fixtures

- `error_invalid_syntax.hedl` - Invalid HEDL syntax for error testing
- `error_malformed.hedl` - Malformed HEDL document with structural issues

## Manifest

The `manifest.json` file describes all available fixtures and their properties:

```json
{
  "fixtures": {
    "basic": {
      "description": "Basic HEDL document...",
      "files": {
        "hedl": "sample_basic.hedl",
        "json": "sample_basic.json",
        ...
      }
    },
    ...
  },
  "errors": {
    "invalid_syntax": {
      "description": "Invalid HEDL syntax...",
      "file": "error_invalid_syntax.hedl",
      "expected_error": true
    },
    ...
  }
}
```

## Usage by Language

### Python

```python
from fixtures import fixtures

# Load basic fixtures
hedl_content = fixtures.basic_hedl
json_content = fixtures.basic_json

# Load error fixtures
invalid_syntax = fixtures.error_invalid_syntax

# Use utility methods
hedl = fixtures.get_fixture("basic", "hedl")
error = fixtures.get_error_fixture("invalid_syntax")
```

### Ruby

```ruby
require_relative 'fixtures'

# Load basic fixtures
hedl_content = $hedl_fixtures.basic_hedl
json_content = $hedl_fixtures.basic_json

# Load error fixtures
invalid_syntax = $hedl_fixtures.error_invalid_syntax

# Use utility methods
hedl = $hedl_fixtures.get_fixture("basic", "basic", "hedl")
error = $hedl_fixtures.get_error_fixture("invalid_syntax")
```

### Node.js/TypeScript

```typescript
import { fixtures } from './fixtures';

// Load basic fixtures
const hedlContent = fixtures.basicHedl;
const jsonContent = fixtures.basicJson;

// Load error fixtures
const invalidSyntax = fixtures.errorInvalidSyntax;

// Use utility methods
const hedl = fixtures.getFixture("basic", "basic", "hedl");
const error = fixtures.getErrorFixture("invalid_syntax");
```

### Go

```go
import "github.com/dweve/hedl/bindings/go/hedl"

// Get global fixtures instance
fixtures := hedl.GetGlobalFixtures()

// Load basic fixtures
hedlContent, _ := fixtures.BasicHEDL()
jsonContent, _ := fixtures.BasicJSON()

// Load error fixtures
invalidSyntax, _ := fixtures.ErrorInvalidSyntax()

// Use utility methods
hedl, _ := fixtures.GetFixture("basic", "hedl")
error, _ := fixtures.GetErrorFixture("invalid_syntax")
```

### PHP

```php
use Dweve\Hedl\Tests\Fixtures;

$fixtures = new Fixtures();

// Load basic fixtures
$hedlContent = $fixtures->basicHedl();
$jsonContent = $fixtures->basicJson();

// Load error fixtures
$invalidSyntax = $fixtures->errorInvalidSyntax();

// Use utility methods
$hedl = $fixtures->getFixture("basic", "basic", "hedl");
$error = $fixtures->getErrorFixture("invalid_syntax");
```

### C#

```csharp
using Dweve.Hedl.Tests;

var fixtures = new Fixtures();

// Load basic fixtures
string hedlContent = fixtures.BasicHedl;
string jsonContent = fixtures.BasicJson;

// Load error fixtures
string invalidSyntax = fixtures.ErrorInvalidSyntax;

// Use utility methods
string hedl = fixtures.GetFixture("basic", "hedl");
string error = fixtures.GetErrorFixture("invalid_syntax");
```

## Adding New Fixtures

To add a new fixture:

1. Create the fixture file(s) in this directory
2. Update `manifest.json` with the new fixture entry
3. Language-specific loaders will automatically pick it up
4. Add accessor methods/properties to each language's fixture loader if needed

### Example: Adding a New Fixture

1. Create `sample_references.hedl`:
```hedl
%VERSION: 1.0
%STRUCT: Author: [id, name]
%STRUCT: Book: [isbn, title, author_ref]
---
authors: @Author
  | 1, Alice Smith
  | 2, Bob Jones
books: @Book
  | 978-1234, "The Book", &authors[0]
  | 978-5678, "Another Book", &authors[1]
```

2. Update `manifest.json`:
```json
{
  "fixtures": {
    ...
    "references": {
      "description": "HEDL document with references",
      "files": {
        "hedl": "sample_references.hedl"
      }
    }
  }
}
```

3. Add accessors to language-specific loaders:
   - Python: `def references_hedl(self)`
   - Ruby: `def references_hedl`
   - Node.js: `get referencesHedl(): string`
   - Go: `func (f *Fixtures) ReferencesHEDL() (string, error)`
   - PHP: `public function referencesHedl(): string`
   - C#: `public string ReferencesHedl`

## Testing

All language bindings have been updated to use these shared fixtures:

- `bindings/python/tests/test_hedl.py` - Uses `from fixtures import fixtures`
- `bindings/ruby/test/test_hedl.rb` - Uses `require_relative 'fixtures'`
- `bindings/node/test/hedl.test.ts` - Uses `import { fixtures } from './fixtures'`
- `bindings/go/hedl_test.go` - Uses `GetGlobalFixtures()`
- `bindings/php/tests/HedlTest.php` - Uses `new Fixtures()`
- `bindings/csharp/Hedl.Tests/HedlTests.cs` - Uses `new Fixtures()`

## Benefits

### Before (Duplicated)

Each binding had its own copy of test data:
- Python: 36 lines of duplicated strings
- Ruby: 32 lines of duplicated strings
- Node.js: 31 lines of duplicated strings
- Go: 31 lines of duplicated constants
- PHP: 28 lines of duplicated strings
- C#: 35 lines of duplicated strings

**Total duplication**: ~193 lines across 6 files

### After (Centralized)

- Fixture files: 7 files in `common/fixtures/`
- Manifest: 1 `manifest.json`
- Language loaders: 6 files (one per language)
- Test updates: 6 files updated to import fixtures

**Result**: Single source of truth, zero duplication

### Maintenance

**Before**: To update a test case, edit 6 separate files
**After**: Edit one fixture file, change propagates to all bindings

### Consistency

**Before**: Easy for test data to drift between languages
**After**: Impossible - all languages use identical fixtures

## Architecture

```
bindings/
├── common/
│   └── fixtures/
│       ├── README.md (this file)
│       ├── manifest.json (fixture metadata)
│       ├── sample_basic.hedl
│       ├── sample_basic.json
│       ├── sample_basic.yaml
│       ├── sample_basic.xml
│       ├── sample_scalars.hedl
│       ├── sample_nested.hedl
│       ├── sample_lists.hedl
│       ├── sample_large.hedl
│       ├── error_invalid_syntax.hedl
│       └── error_malformed.hedl
├── python/
│   └── tests/
│       ├── fixtures.py (Python loader)
│       └── test_hedl.py (uses fixtures)
├── ruby/
│   └── test/
│       ├── fixtures.rb (Ruby loader)
│       └── test_hedl.rb (uses fixtures)
├── node/
│   └── test/
│       ├── fixtures.ts (Node.js/TS loader)
│       └── hedl.test.ts (uses fixtures)
├── go/
│   ├── fixtures.go (Go loader)
│   └── hedl_test.go (uses fixtures)
├── php/
│   └── tests/
│       ├── Fixtures.php (PHP loader)
│       └── HedlTest.php (uses fixtures)
└── csharp/
    └── Hedl.Tests/
        ├── Fixtures.cs (C# loader)
        └── HedlTests.cs (uses fixtures)
```

## Design Principles

1. **DRY (Don't Repeat Yourself)**: Single source of truth for all test data
2. **Language Idiomatic**: Each loader follows language conventions
3. **Type Safe**: Strong typing where applicable (TypeScript, Go, C#, PHP)
4. **Documentation**: Comprehensive doc comments in all loaders
5. **Error Handling**: Proper error handling in all languages
6. **Extensibility**: Easy to add new fixtures without breaking changes
7. **Backward Compatibility**: Legacy constants provided for existing tests

## Future Enhancements

Potential improvements:

- Add more complex fixtures (circular references, deep nesting)
- Add fixtures for edge cases and boundary conditions
- Add fixtures for performance benchmarking
- Generate fixtures programmatically for property-based testing
- Add fixtures for internationalization/unicode testing
- Add fixtures for security testing (injection, overflow, etc.)

## Contributing

When adding new fixtures:

1. Follow existing naming conventions
2. Update manifest.json
3. Add documentation to this README
4. Add accessors to all 6 language loaders
5. Update at least one test in each language to use the new fixture
6. Ensure fixtures are valid HEDL (or intentionally invalid for error cases)

## License

Same as HEDL project - see root LICENSE file.
