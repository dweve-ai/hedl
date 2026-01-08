# HEDL PHP Bindings

PHP bindings for HEDL (Hierarchical Entity Data Language) using PHP's FFI extension.

## Requirements

- PHP 8.0+ with FFI extension enabled
- `libhedl_ffi` shared library
- For async API: `amphp/amp >= 2.6.0` (optional)

### Enable FFI

In `php.ini`:
```ini
extension=ffi
ffi.enable=true
```

## Installation

Copy `Hedl.php` to your project and require it:

```php
require_once 'path/to/Hedl.php';

use Dweve\Hedl\Hedl;
```

Or use Composer (coming soon):
```bash
composer require dweve/hedl
```

## Quick Start

```php
<?php
require_once 'Hedl.php';

use Dweve\Hedl\Hedl;

// Parse HEDL content
$doc = Hedl::parse('
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
');

// Get document info
[$major, $minor] = $doc->version();
echo "Version: {$major}.{$minor}\n";
echo "Schemas: {$doc->schemaCount()}\n";

// Convert to JSON
$json = $doc->toJson();
echo $json;

// Convert to other formats
$yaml = $doc->toYaml();
$xml = $doc->toXml();
$csv = $doc->toCsv();
$cypher = $doc->toCypher();

// Clean up
$doc->close();
```

## API Reference

### Static Methods

| Method | Description |
|--------|-------------|
| `Hedl::parse($content, $strict)` | Parse HEDL string |
| `Hedl::validate($content, $strict)` | Validate without document |
| `Hedl::fromJson($content)` | Parse JSON to HEDL |
| `Hedl::fromYaml($content)` | Parse YAML to HEDL |
| `Hedl::fromXml($content)` | Parse XML to HEDL |
| `Hedl::setLibraryPath($path)` | Set library path |

### Document Methods

| Method | Description |
|--------|-------------|
| `version()` | Get [major, minor] array |
| `schemaCount()` | Get schema count |
| `aliasCount()` | Get alias count |
| `rootItemCount()` | Get root item count |
| `canonicalize()` | Convert to canonical HEDL |
| `toJson($includeMetadata)` | Convert to JSON |
| `toYaml($includeMetadata)` | Convert to YAML |
| `toXml()` | Convert to XML |
| `toCsv()` | Convert to CSV |
| `toCypher($useMerge)` | Convert to Neo4j Cypher |
| `lint()` | Run linting |
| `close()` | Free resources |

### Diagnostics

```php
$diag = $doc->lint();
echo "Issues: " . $diag->count() . "\n";

foreach ($diag->all() as $item) {
    echo "[{$item['severity']}] {$item['message']}\n";
}

$errors = $diag->errors();
$warnings = $diag->warnings();
$hints = $diag->hints();

$diag->close();
```

### Error Handling

```php
use Dweve\Hedl\HedlException;

try {
    $doc = Hedl::parse('invalid content');
} catch (HedlException $e) {
    echo "Error: " . $e->getMessage() . "\n";
    echo "Code: " . $e->getHedlCode() . "\n";
}
```

## Async API (AMPHP)

The HEDL PHP bindings include an async API using [AMPHP](https://amphp.org/), a popular non-blocking concurrency framework for PHP.

### Installation

```bash
composer require amphp/amp
```

### Static Async Methods

Parse and validation operations can be called asynchronously:

```php
<?php
require_once 'Hedl.php';

use Dweve\Hedl\Hedl;
use Amp\Loop;

// Async parsing
Loop::run(function() {
    $promise = Hedl::parseAsync('%VERSION: 1.0\n---\nkey: value');
    $doc = yield $promise;
    echo $doc->toJson();
    $doc->close();
});
```

**Available static async methods:**

| Method | Returns |
|--------|---------|
| `Hedl::parseAsync($content, $strict)` | `Promise<Document>` |
| `Hedl::validateAsync($content, $strict)` | `Promise<bool>` |
| `Hedl::fromJsonAsync($content)` | `Promise<Document>` |
| `Hedl::fromYamlAsync($content)` | `Promise<Document>` |
| `Hedl::fromXmlAsync($content)` | `Promise<Document>` |
| `Hedl::fromParquetAsync($data)` | `Promise<Document>` |

### Document Async Methods

Use `AsyncDocument` to call conversion operations asynchronously:

```php
<?php
require_once 'Hedl.php';

use Dweve\Hedl\Hedl;
use Amp\Loop;

Loop::run(function() {
    $doc = Hedl::parse('%VERSION: 1.0\n---\nusers: [{id: 1, name: Alice}]');
    $asyncDoc = Hedl::wrapAsync($doc);

    // Async conversions
    $json = yield $asyncDoc->toJsonAsync();
    $yaml = yield $asyncDoc->toYamlAsync();
    $xml = yield $asyncDoc->toXmlAsync();

    echo $json;

    $doc->close();
});
```

**Available async conversion methods:**

| Method | Returns |
|--------|---------|
| `canonicalizeAsync()` | `Promise<string>` |
| `toJsonAsync($includeMetadata)` | `Promise<string>` |
| `toYamlAsync($includeMetadata)` | `Promise<string>` |
| `toXmlAsync()` | `Promise<string>` |
| `toCsvAsync()` | `Promise<string>` |
| `toCypherAsync($useMerge)` | `Promise<string>` |
| `toParquetAsync()` | `Promise<string>` |
| `lintAsync()` | `Promise<Diagnostics>` |

### Async Integration Example

Combine HEDL processing with other async I/O operations:

```php
<?php
require_once 'Hedl.php';

use Dweve\Hedl\Hedl;
use Amp\Loop;
use Amp\Http\Client\HttpClientBuilder;

Loop::run(function() {
    $client = HttpClientBuilder::buildClient();

    // Fetch data from multiple sources concurrently
    $promises = [
        $client->request('GET', 'https://example.com/data1.json'),
        $client->request('GET', 'https://example.com/data2.json'),
    ];

    $responses = yield $promises;

    // Process each response with HEDL
    foreach ($responses as $response) {
        $json = yield $response->getBody();
        $doc = yield Hedl::fromJsonAsync($json);
        echo $doc->toJson();
        $doc->close();
    }
});
```

### Error Handling with Async

```php
<?php
require_once 'Hedl.php';

use Dweve\Hedl\Hedl;
use Dweve\Hedl\HedlException;
use Amp\Loop;

Loop::run(function() {
    try {
        $doc = yield Hedl::parseAsync('invalid {{{');
    } catch (HedlException $e) {
        echo "Parse error: " . $e->getMessage() . "\n";
    }
});
```

### Important Async Considerations

The AMPHP async API uses **cooperative multitasking**, not true OS-level concurrency:

1. **Blocking Operations**: HEDL parsing and conversion are CPU-bound and blocking. The async API primarily benefits when combined with truly async I/O (network requests, file operations).

2. **Resource Management**: Always close documents when done to free native resources:
   ```php
   $asyncDoc = Hedl::wrapAsync($doc);
   try {
       $result = yield $asyncDoc->toJsonAsync();
   } finally {
       $doc->close();
   }
   ```

3. **Promise Composition**: Use `Amp\Promise\all()` or `Amp\Promise\any()` to coordinate multiple operations:
   ```php
   $promises = [
       Hedl::parseAsync($content1),
       Hedl::parseAsync($content2),
   ];
   $docs = yield Amp\Promise\all($promises);
   ```

4. **Timeout Support**: The async API integrates with AMPHP's timeout mechanisms:
   ```php
   $doc = yield Amp\Promise\timeout(
       Hedl::parseAsync($content),
       5000  // 5 second timeout
   );
   ```

## Environment Variables

| Variable | Description | Default | Recommended |
|----------|-------------|---------|-------------|
| `HEDL_LIB_PATH` | Path to the HEDL shared library | Auto-detected | - |
| `HEDL_MAX_OUTPUT_SIZE` | Maximum output size in bytes for conversions | 100 MB | 500 MB - 1 GB |

### Resource Limits

The `HEDL_MAX_OUTPUT_SIZE` environment variable controls the maximum size of output from conversion operations (`toJson()`, `toYaml()`, `toXml()`, etc.). The default of 100 MB is conservative and may be too restrictive for many real-world data processing scenarios.

**Setting the limit:**

```bash
# In your shell or Apache/PHP-FPM environment
export HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB

# For Apache, add to your VirtualHost or .htaccess:
SetEnv HEDL_MAX_OUTPUT_SIZE 1073741824

# For PHP-FPM, add to pool configuration:
env[HEDL_MAX_OUTPUT_SIZE] = 1073741824

# Or in PHP (must be set BEFORE use)
putenv('HEDL_MAX_OUTPUT_SIZE=1073741824');  // 1 GB
use Dweve\Hedl\Hedl;
```

**Recommended values:**

- **Small configs (10-50 MB)**: Default 100 MB is usually sufficient
- **Medium datasets (100-500 MB)**: Set to `524288000` (500 MB)
- **Large datasets (500 MB - 5 GB)**: Set to `1073741824` or higher (1 GB+)
- **Very large datasets**: Set to `5368709120` (5 GB) or `10737418240` (10 GB)
- **No practical limit**: Set to a very high value appropriate for your system

**Error handling:**

When the output size exceeds the limit, a `HedlException` will be thrown:

```php
use Dweve\Hedl\HedlException;
use Dweve\Hedl\ErrorCode;

try {
    $largeOutput = $doc->toJson();
} catch (HedlException $e) {
    if ($e->getHedlCode() === ErrorCode::ALLOC) {
        echo "Output too large: {$e->getMessage()}\n";
        echo "Increase HEDL_MAX_OUTPUT_SIZE environment variable\n";
    }
}
```

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
