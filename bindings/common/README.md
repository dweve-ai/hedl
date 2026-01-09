# HEDL Common Bindings Resources

This directory contains shared resources used across all HEDL language bindings.

## Contents

### `/fixtures` - Common Test Fixtures

Centralized test fixtures shared across all 6 language bindings (Python, Ruby, Node.js, Go, PHP, C#).

**Key Features:**
- Single source of truth for test data
- Eliminates duplication across bindings
- Ensures consistency across languages
- Easy to maintain and extend

**Documentation:**
- See [fixtures/README.md](fixtures/README.md) for detailed usage guide

**Quick Usage:**

```python
# Python
from fixtures import fixtures
hedl_content = fixtures.basic_hedl
```

```ruby
# Ruby
require_relative 'fixtures'
hedl_content = $hedl_fixtures.basic_hedl
```

```typescript
// Node.js/TypeScript
import { fixtures } from './fixtures';
const hedlContent = fixtures.basicHedl;
```

```go
// Go
fixtures := hedl.GetGlobalFixtures()
hedlContent, _ := fixtures.BasicHEDL()
```

```php
// PHP
use Dweve\Hedl\Tests\Fixtures;
$fixtures = new Fixtures();
$hedlContent = $fixtures->basicHedl();
```

```csharp
// C#
using Dweve.Hedl.Tests;
var fixtures = new Fixtures();
string hedlContent = fixtures.BasicHedl;
```

## Directory Structure

```
common/
├── README.md (this file)
└── fixtures/
    ├── README.md (fixtures documentation)
    ├── manifest.json (fixture metadata)
    ├── sample_basic.hedl
    ├── sample_basic.json
    ├── sample_basic.yaml
    ├── sample_basic.xml
    ├── sample_scalars.hedl
    ├── sample_nested.hedl
    ├── sample_lists.hedl
    ├── sample_large.hedl
    ├── error_invalid_syntax.hedl
    └── error_malformed.hedl
```

## Adding New Common Resources

When adding new shared resources:

1. Create a new subdirectory (e.g., `common/schemas/`)
2. Add language-specific loaders in each binding
3. Document in this README
4. Update binding tests to use the shared resources

## Design Principles

1. **DRY (Don't Repeat Yourself)** - Single source of truth
2. **Language Idiomatic** - Follow each language's conventions
3. **Well Documented** - Clear usage examples for all languages
4. **Type Safe** - Strong typing where applicable
5. **Extensible** - Easy to add new resources

## Benefits

- **Consistency**: Identical data across all bindings
- **Maintenance**: Update once, apply everywhere
- **Quality**: Centralized validation and testing
- **Efficiency**: Reduced code duplication
- **Collaboration**: Easier for multi-language teams

## Future Additions

Potential future common resources:

- `/schemas` - Common JSON/XML schemas for validation
- `/data` - Shared test datasets
- `/scripts` - Common build/test scripts
- `/benchmarks` - Shared benchmark data
- `/docs` - Common documentation templates

## Contributing

When adding common resources:

1. Ensure it's truly needed by multiple bindings
2. Create language-specific loaders for all 6 bindings
3. Write comprehensive documentation
4. Add usage examples for each language
5. Update this README
6. Run tests in all bindings

## Related Documentation

- [Fixtures README](fixtures/README.md) - Detailed fixtures documentation
