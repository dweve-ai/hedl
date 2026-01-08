<?php
/**
 * HEDL (Hierarchical Entity Data Language) PHP Bindings
 *
 * A token-efficient data format optimized for LLM context windows.
 *
 * Requirements:
 * - PHP 8.0+ with FFI extension enabled
 * - libhedl_ffi shared library
 *
 * Usage:
 *   $doc = Hedl::parse('%VERSION: 1.0\n---\nkey: value');
 *   echo $doc->toJson();
 *   $doc->close();
 *
 * THREAD SAFETY WARNING:
 * =====================
 * These bindings are NOT thread-safe. Document and Diagnostics objects must
 * not be accessed concurrently from multiple threads. The underlying FFI
 * library does not perform any internal locking. If you need concurrent
 * access to HEDL documents, you must:
 *
 * 1. Use separate Document instances per thread, OR
 * 2. Implement your own synchronization (mutexes, locks) around all
 *    Document and Diagnostics method calls
 *
 * Concurrent access without proper synchronization may result in:
 * - Memory corruption
 * - Use-after-free errors
 * - Segmentation faults
 * - Undefined behavior
 *
 * PHP's typical request-per-process model means this is rarely an issue,
 * but be aware if using threading extensions like pthreads or parallel.
 *
 * RESOURCE LIMITS:
 * ===============
 * The HEDL_MAX_OUTPUT_SIZE environment variable controls the maximum size of
 * output from conversion operations (toJson, toYaml, toXml, etc.).
 *
 * Default: 100 MB (conservative, may be too restrictive for many use cases)
 * Recommended for data processing: 500 MB - 1 GB
 * For large datasets: 1 GB - 5 GB
 *
 * Set before loading the library:
 *
 *   // In your shell or Apache/PHP-FPM config
 *   export HEDL_MAX_OUTPUT_SIZE=1073741824  // 1 GB
 *
 *   // Or in PHP (must be set BEFORE use)
 *   putenv('HEDL_MAX_OUTPUT_SIZE=1073741824');  // 1 GB
 *   use Dweve\Hedl\Hedl;
 *
 * Use Cases:
 * - Small configs: 10-50 MB (default may suffice)
 * - Medium datasets: 100-500 MB (set to 524288000 for 500 MB)
 * - Large datasets: 500 MB - 5 GB (set to 1073741824+ for 1 GB+)
 * - No practical limit: set to a very high value like 10737418240 (10 GB)
 *
 * When the limit is exceeded, operations will throw HedlException with code
 * ErrorCode::ALLOC and a message suggesting to increase HEDL_MAX_OUTPUT_SIZE.
 */

namespace Dweve\Hedl;

use FFI;
use RuntimeException;

// Resource limits
// Default is 100MB, which may be too restrictive for many real-world scenarios.
// Recommended: 500MB-1GB for data processing, higher for large datasets.
// Set HEDL_MAX_OUTPUT_SIZE environment variable before loading to customize.
define('HEDL_MAX_OUTPUT_SIZE', (int)(getenv('HEDL_MAX_OUTPUT_SIZE') ?: '104857600')); // 100MB default

/**
 * Error codes from the HEDL library.
 */
class ErrorCode
{
    public const OK = 0;
    public const NULL_PTR = -1;
    public const INVALID_UTF8 = -2;
    public const PARSE = -3;
    public const CANONICALIZE = -4;
    public const JSON = -5;
    public const ALLOC = -6;
    public const YAML = -7;
    public const XML = -8;
    public const CSV = -9;
    public const PARQUET = -10;
    public const LINT = -11;
    public const NEO4J = -12;
}

/**
 * Severity levels for diagnostics.
 */
class Severity
{
    public const HINT = 0;
    public const WARNING = 1;
    public const ERROR = 2;
}

/**
 * Exception thrown by HEDL operations.
 */
class HedlException extends RuntimeException
{
    public function __construct(
        string $message,
        private readonly int $hedlCode = ErrorCode::PARSE,
    ) {
        parent::__construct($message, 0);
    }

    public function getHedlCode(): int
    {
        return $this->hedlCode;
    }
}

/**
 * Check if output size exceeds the limit.
 */
function checkOutputSize(string $output): void
{
    $outputSize = strlen($output);
    if ($outputSize > HEDL_MAX_OUTPUT_SIZE) {
        $actualMb = $outputSize / 1048576.0;
        $limitMb = HEDL_MAX_OUTPUT_SIZE / 1048576.0;
        throw new HedlException(
            sprintf("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                    $actualMb, $limitMb),
            ErrorCode::ALLOC
        );
    }
}

/**
 * FFI library loader.
 */
class Library
{
    private static ?FFI $ffi = null;
    private static ?string $libPath = null;

    private const HEADER = <<<'HEADER'
    typedef struct HedlDocument HedlDocument;
    typedef struct HedlDiagnostics HedlDiagnostics;

    const char* hedl_get_last_error(void);
    void hedl_free_string(char* s);
    void hedl_free_document(HedlDocument* doc);
    void hedl_free_diagnostics(HedlDiagnostics* diag);
    void hedl_free_bytes(uint8_t* data, size_t len);

    int hedl_parse(const char* input, int input_len, int strict, HedlDocument** out_doc);
    int hedl_validate(const char* input, int input_len, int strict);

    int hedl_get_version(const HedlDocument* doc, int* major, int* minor);
    int hedl_schema_count(const HedlDocument* doc);
    int hedl_alias_count(const HedlDocument* doc);
    int hedl_root_item_count(const HedlDocument* doc);

    int hedl_canonicalize(const HedlDocument* doc, char** out_str);

    int hedl_to_json(const HedlDocument* doc, int include_metadata, char** out_str);
    int hedl_from_json(const char* json, int json_len, HedlDocument** out_doc);

    int hedl_to_yaml(const HedlDocument* doc, int include_metadata, char** out_str);
    int hedl_from_yaml(const char* yaml, int yaml_len, HedlDocument** out_doc);

    int hedl_to_xml(const HedlDocument* doc, char** out_str);
    int hedl_from_xml(const char* xml, int xml_len, HedlDocument** out_doc);

    int hedl_to_csv(const HedlDocument* doc, char** out_str);

    int hedl_to_parquet(const HedlDocument* doc, uint8_t** out_data, size_t* out_len);
    int hedl_from_parquet(const uint8_t* data, size_t len, HedlDocument** out_doc);

    int hedl_to_neo4j_cypher(const HedlDocument* doc, int use_merge, char** out_str);

    int hedl_lint(const HedlDocument* doc, HedlDiagnostics** out_diag);
    int hedl_diagnostics_count(const HedlDiagnostics* diag);
    int hedl_diagnostics_get(const HedlDiagnostics* diag, int index, char** out_str);
    int hedl_diagnostics_severity(const HedlDiagnostics* diag, int index);
HEADER;

    /**
     * Set the library path.
     */
    public static function setLibraryPath(string $path): void
    {
        self::$libPath = $path;
        self::$ffi = null; // Force reload
    }

    /**
     * Get the FFI instance.
     */
    public static function get(): FFI
    {
        if (self::$ffi === null) {
            $libPath = self::findLibrary();
            self::$ffi = FFI::cdef(self::HEADER, $libPath);
        }
        return self::$ffi;
    }

    /**
     * Find the library path.
     */
    private static function findLibrary(): string
    {
        if (self::$libPath !== null) {
            return self::$libPath;
        }

        $envPath = getenv('HEDL_LIB_PATH');
        if ($envPath !== false && file_exists($envPath)) {
            return $envPath;
        }

        // Platform-specific library name
        $libName = match (PHP_OS_FAMILY) {
            'Darwin' => 'libhedl_ffi.dylib',
            'Windows' => 'hedl_ffi.dll',
            default => 'libhedl_ffi.so',
        };

        // Check common paths
        $paths = [
            __DIR__ . '/' . $libName,
            __DIR__ . '/../../../target/release/' . $libName,
            '/usr/local/lib/' . $libName,
            '/usr/lib/' . $libName,
        ];

        foreach ($paths as $path) {
            if (file_exists($path)) {
                return $path;
            }
        }

        // Return name for system search
        return $libName;
    }

    /**
     * Get the last error message from the library.
     */
    public static function getLastError(): ?string
    {
        $ffi = self::get();
        $err = $ffi->hedl_get_last_error();
        return $err !== null ? FFI::string($err) : null;
    }

    /**
     * Create an exception from the last error.
     */
    public static function createException(int $code): HedlException
    {
        $message = self::getLastError() ?? "HEDL error code {$code}";
        return new HedlException($message, $code);
    }
}

/**
 * Lint diagnostics container.
 *
 * Provides access to diagnostic messages (errors, warnings, hints) produced
 * by linting a HEDL document. Diagnostics are indexed starting from 0.
 *
 * This class manages native resources and should be explicitly closed when
 * no longer needed, or rely on automatic cleanup via __destruct.
 *
 * Example:
 *   $doc = Hedl::parse($content);
 *   $diag = $doc->lint();
 *   foreach ($diag->errors() as $error) {
 *       echo "Error: $error\n";
 *   }
 *   $diag->close();
 *   $doc->close();
 */
class Diagnostics
{
    private bool $closed = false;

    public function __construct(
        private readonly FFI\CData $ptr,
    ) {
    }

    public function __destruct()
    {
        $this->close();
    }

    /**
     * Free resources.
     */
    public function close(): void
    {
        if (!$this->closed) {
            Library::get()->hedl_free_diagnostics($this->ptr);
            $this->closed = true;
        }
    }

    /**
     * Get the number of diagnostics.
     */
    public function count(): int
    {
        if ($this->closed) {
            return 0;
        }
        $count = Library::get()->hedl_diagnostics_count($this->ptr);
        return max(0, $count);
    }

    /**
     * Get diagnostic at index.
     *
     * @return array{message: string, severity: int}
     */
    public function get(int $index): array
    {
        if ($this->closed) {
            throw new HedlException('Diagnostics already closed', ErrorCode::NULL_PTR);
        }

        $ffi = Library::get();
        $msgPtr = $ffi->new('char*');
        $result = $ffi->hedl_diagnostics_get($this->ptr, $index, FFI::addr($msgPtr));

        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }

        $message = FFI::string($msgPtr);
        $ffi->hedl_free_string($msgPtr);

        $severity = $ffi->hedl_diagnostics_severity($this->ptr, $index);

        return [
            'message' => $message,
            'severity' => $severity,
        ];
    }

    /**
     * Get all diagnostics.
     *
     * @return array<array{message: string, severity: int}>
     */
    public function all(): array
    {
        $result = [];
        for ($i = 0; $i < $this->count(); $i++) {
            $result[] = $this->get($i);
        }
        return $result;
    }

    /**
     * Get all error messages.
     *
     * @return string[]
     */
    public function errors(): array
    {
        return array_column(
            array_filter($this->all(), fn($d) => $d['severity'] === Severity::ERROR),
            'message'
        );
    }

    /**
     * Get all warning messages.
     *
     * @return string[]
     */
    public function warnings(): array
    {
        return array_column(
            array_filter($this->all(), fn($d) => $d['severity'] === Severity::WARNING),
            'message'
        );
    }

    /**
     * Get all hint messages.
     *
     * @return string[]
     */
    public function hints(): array
    {
        return array_column(
            array_filter($this->all(), fn($d) => $d['severity'] === Severity::HINT),
            'message'
        );
    }
}

/**
 * A parsed HEDL document.
 *
 * Represents a parsed HEDL document with access to metadata (version, schema
 * count, etc.) and conversion methods to various formats (JSON, YAML, XML,
 * CSV, Parquet, Cypher).
 *
 * This class manages native resources and should be explicitly closed when
 * no longer needed, or rely on automatic cleanup via __destruct.
 *
 * WARNING: Not thread-safe. See file-level documentation for details.
 *
 * Example:
 *   $doc = Hedl::parse('%VERSION: 1.0\n---\nkey: value');
 *   list($major, $minor) = $doc->version();
 *   echo $doc->toJson();
 *   $doc->close();
 */
class Document
{
    private bool $closed = false;

    public function __construct(
        private readonly FFI\CData $ptr,
    ) {
    }

    public function __destruct()
    {
        $this->close();
    }

    /**
     * Free resources.
     */
    public function close(): void
    {
        if (!$this->closed) {
            Library::get()->hedl_free_document($this->ptr);
            $this->closed = true;
        }
    }

    private function checkClosed(): void
    {
        if ($this->closed) {
            throw new HedlException('Document already closed', ErrorCode::NULL_PTR);
        }
    }

    /**
     * Get the HEDL version.
     *
     * @return array{int, int} [major, minor]
     */
    public function version(): array
    {
        $this->checkClosed();
        $ffi = Library::get();
        $major = $ffi->new('int');
        $minor = $ffi->new('int');
        $result = $ffi->hedl_get_version($this->ptr, FFI::addr($major), FFI::addr($minor));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        return [$major->cdata, $minor->cdata];
    }

    /**
     * Get the number of schema definitions.
     */
    public function schemaCount(): int
    {
        $this->checkClosed();
        $count = Library::get()->hedl_schema_count($this->ptr);
        if ($count < 0) {
            throw Library::createException($count);
        }
        return $count;
    }

    /**
     * Get the number of alias definitions.
     */
    public function aliasCount(): int
    {
        $this->checkClosed();
        $count = Library::get()->hedl_alias_count($this->ptr);
        if ($count < 0) {
            throw Library::createException($count);
        }
        return $count;
    }

    /**
     * Get the number of root items.
     */
    public function rootItemCount(): int
    {
        $this->checkClosed();
        $count = Library::get()->hedl_root_item_count($this->ptr);
        if ($count < 0) {
            throw Library::createException($count);
        }
        return $count;
    }

    /**
     * Convert to canonical HEDL form.
     */
    public function canonicalize(): string
    {
        $this->checkClosed();
        $ffi = Library::get();
        $outPtr = $ffi->new('char*');
        $result = $ffi->hedl_canonicalize($this->ptr, FFI::addr($outPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        $output = FFI::string($outPtr);
        $ffi->hedl_free_string($outPtr);
        checkOutputSize($output);
        return $output;
    }

    /**
     * Convert to JSON.
     */
    public function toJson(bool $includeMetadata = false): string
    {
        $this->checkClosed();
        $ffi = Library::get();
        $outPtr = $ffi->new('char*');
        $result = $ffi->hedl_to_json($this->ptr, $includeMetadata ? 1 : 0, FFI::addr($outPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        $output = FFI::string($outPtr);
        $ffi->hedl_free_string($outPtr);
        checkOutputSize($output);
        return $output;
    }

    /**
     * Convert to YAML.
     */
    public function toYaml(bool $includeMetadata = false): string
    {
        $this->checkClosed();
        $ffi = Library::get();
        $outPtr = $ffi->new('char*');
        $result = $ffi->hedl_to_yaml($this->ptr, $includeMetadata ? 1 : 0, FFI::addr($outPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        $output = FFI::string($outPtr);
        $ffi->hedl_free_string($outPtr);
        checkOutputSize($output);
        return $output;
    }

    /**
     * Convert to XML.
     */
    public function toXml(): string
    {
        $this->checkClosed();
        $ffi = Library::get();
        $outPtr = $ffi->new('char*');
        $result = $ffi->hedl_to_xml($this->ptr, FFI::addr($outPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        $output = FFI::string($outPtr);
        $ffi->hedl_free_string($outPtr);
        checkOutputSize($output);
        return $output;
    }

    /**
     * Convert to CSV.
     */
    public function toCsv(): string
    {
        $this->checkClosed();
        $ffi = Library::get();
        $outPtr = $ffi->new('char*');
        $result = $ffi->hedl_to_csv($this->ptr, FFI::addr($outPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        $output = FFI::string($outPtr);
        $ffi->hedl_free_string($outPtr);
        checkOutputSize($output);
        return $output;
    }

    /**
     * Convert to Neo4j Cypher queries.
     *
     * @param bool $useMerge Use MERGE (idempotent) instead of CREATE statements.
     * @return string Cypher query string.
     */
    public function toCypher(bool $useMerge = true): string
    {
        $this->checkClosed();
        $ffi = Library::get();
        $outPtr = $ffi->new('char*');
        $result = $ffi->hedl_to_neo4j_cypher($this->ptr, $useMerge ? 1 : 0, FFI::addr($outPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        $output = FFI::string($outPtr);
        $ffi->hedl_free_string($outPtr);
        checkOutputSize($output);
        return $output;
    }

    /**
     * Convert to Parquet format.
     *
     * Note: Only works for documents with matrix lists. The Parquet output
     * represents tabular data extracted from the HEDL document structure.
     *
     * @return string Binary Parquet file contents.
     * @throws HedlException If conversion fails.
     */
    public function toParquet(): string
    {
        $this->checkClosed();
        $ffi = Library::get();
        $dataPtr = $ffi->new('uint8_t*');
        $dataLen = $ffi->new('size_t');
        $result = $ffi->hedl_to_parquet($this->ptr, FFI::addr($dataPtr), FFI::addr($dataLen));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }

        // Copy binary data before freeing
        $length = $dataLen->cdata;
        $output = FFI::string($dataPtr, $length);
        $ffi->hedl_free_bytes($dataPtr, $dataLen->cdata);
        checkOutputSize($output);
        return $output;
    }

    /**
     * Run linting on the document.
     *
     * @return Diagnostics Diagnostic messages from linting.
     * @throws HedlException If linting fails.
     */
    public function lint(): Diagnostics
    {
        $this->checkClosed();
        $ffi = Library::get();
        $diagPtr = $ffi->new('HedlDiagnostics*');
        $result = $ffi->hedl_lint($this->ptr, FFI::addr($diagPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        return new Diagnostics($diagPtr);
    }
}

/**
 * Async wrapper for Document operations using AMPHP.
 *
 * Provides Promise-based async methods for HEDL document conversions.
 * This class wraps a Document and provides non-blocking operations.
 *
 * IMPORTANT: AMPHP promises are not truly asynchronous but cooperative:
 * - The underlying FFI operations are still blocking CPU-bound operations
 * - Promises allow better code structure and timeout handling
 * - Use for I/O-bound operations combined with HEDL processing
 *
 * Example:
 *   $doc = Hedl::parse($content);
 *   $asyncDoc = new AsyncDocument($doc);
 *   $json = Amp\call(function() use ($asyncDoc) {
 *       return yield $asyncDoc->toJsonAsync();
 *   });
 *   $doc->close();
 *
 * Requires: amphp/amp >= 2.6.0
 */
class AsyncDocument
{
    public function __construct(private readonly Document $document)
    {
        self::checkAmpInstalled();
    }

    /**
     * Check if AMPHP is installed and available.
     */
    private static function checkAmpInstalled(): void
    {
        if (!class_exists('Amp\Promise')) {
            throw new RuntimeException(
                'AMPHP is not installed. Install with: composer require amphp/amp'
            );
        }
    }

    /**
     * Get the underlying Document instance.
     */
    public function document(): Document
    {
        return $this->document;
    }

    /**
     * Asynchronously convert to canonical HEDL form.
     *
     * @return object Promise<string> Promise resolving to canonical HEDL
     */
    public function canonicalizeAsync(): object
    {
        return $this->executeAsync(
            fn() => $this->document->canonicalize(),
            'canonicalize'
        );
    }

    /**
     * Asynchronously convert to JSON.
     *
     * @return object Promise<string> Promise resolving to JSON string
     */
    public function toJsonAsync(bool $includeMetadata = false): object
    {
        return $this->executeAsync(
            fn() => $this->document->toJson($includeMetadata),
            'toJson'
        );
    }

    /**
     * Asynchronously convert to YAML.
     *
     * @return Promise<string> Promise resolving to YAML string
     */
    public function toYamlAsync(bool $includeMetadata = false): object
    {
        return $this->executeAsync(
            fn() => $this->document->toYaml($includeMetadata),
            'toYaml'
        );
    }

    /**
     * Asynchronously convert to XML.
     *
     * @return Promise<string> Promise resolving to XML string
     */
    public function toXmlAsync(): object
    {
        return $this->executeAsync(
            fn() => $this->document->toXml(),
            'toXml'
        );
    }

    /**
     * Asynchronously convert to CSV.
     *
     * @return Promise<string> Promise resolving to CSV string
     */
    public function toCsvAsync(): object
    {
        return $this->executeAsync(
            fn() => $this->document->toCsv(),
            'toCsv'
        );
    }

    /**
     * Asynchronously convert to Neo4j Cypher queries.
     *
     * @return Promise<string> Promise resolving to Cypher query string
     */
    public function toCypherAsync(bool $useMerge = true): object
    {
        return $this->executeAsync(
            fn() => $this->document->toCypher($useMerge),
            'toCypher'
        );
    }

    /**
     * Asynchronously convert to Parquet format.
     *
     * @return Promise<string> Promise resolving to binary Parquet data
     */
    public function toParquetAsync(): object
    {
        return $this->executeAsync(
            fn() => $this->document->toParquet(),
            'toParquet'
        );
    }

    /**
     * Asynchronously run linting on the document.
     *
     * @return Promise<Diagnostics> Promise resolving to Diagnostics
     */
    public function lintAsync(): object
    {
        return $this->executeAsync(
            fn() => $this->document->lint(),
            'lint'
        );
    }

    /**
     * Execute a callback asynchronously using cooperative multitasking.
     *
     * @template T
     * @param callable(): T $callback
     * @param string $operationName
     * @return object Promise<T> (Amp\Promise)
     */
    private function executeAsync(callable $callback, string $operationName): object
    {
        self::checkAmpInstalled();

        $deferred = new \Amp\Deferred();

        try {
            $result = $callback();
            $deferred->resolve($result);
        } catch (Throwable $e) {
            $deferred->fail($e);
        }

        return $deferred->promise();
    }
}

/**
 * Main HEDL class with static factory methods.
 *
 * Provides static methods for parsing HEDL content and converting from
 * various formats (JSON, YAML, XML, Parquet) into HEDL documents.
 *
 * All methods return Document instances that manage native resources.
 * Remember to call close() on returned documents when done.
 *
 * Example:
 *   // Parse HEDL
 *   $doc = Hedl::parse($hedlContent);
 *   echo $doc->toJson();
 *   $doc->close();
 *
 *   // Convert from JSON
 *   $doc = Hedl::fromJson($jsonContent);
 *   echo $doc->canonicalize();
 *   $doc->close();
 */
class Hedl
{
    /**
     * Parse HEDL content into a Document.
     */
    public static function parse(string $content, bool $strict = true): Document
    {
        $ffi = Library::get();
        $docPtr = $ffi->new('HedlDocument*');
        $result = $ffi->hedl_parse($content, strlen($content), $strict ? 1 : 0, FFI::addr($docPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        return new Document($docPtr);
    }

    /**
     * Validate HEDL content without creating a document.
     */
    public static function validate(string $content, bool $strict = true): bool
    {
        $ffi = Library::get();
        $result = $ffi->hedl_validate($content, strlen($content), $strict ? 1 : 0);
        return $result === ErrorCode::OK;
    }

    /**
     * Parse JSON content into a HEDL Document.
     */
    public static function fromJson(string $content): Document
    {
        $ffi = Library::get();
        $docPtr = $ffi->new('HedlDocument*');
        $result = $ffi->hedl_from_json($content, strlen($content), FFI::addr($docPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        return new Document($docPtr);
    }

    /**
     * Parse YAML content into a HEDL Document.
     */
    public static function fromYaml(string $content): Document
    {
        $ffi = Library::get();
        $docPtr = $ffi->new('HedlDocument*');
        $result = $ffi->hedl_from_yaml($content, strlen($content), FFI::addr($docPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        return new Document($docPtr);
    }

    /**
     * Parse XML content into a HEDL Document.
     *
     * @param string $content XML content to parse.
     * @return Document Parsed HEDL document.
     * @throws HedlException If parsing fails.
     */
    public static function fromXml(string $content): Document
    {
        $ffi = Library::get();
        $docPtr = $ffi->new('HedlDocument*');
        $result = $ffi->hedl_from_xml($content, strlen($content), FFI::addr($docPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        return new Document($docPtr);
    }

    /**
     * Parse Parquet binary data into a HEDL Document.
     *
     * @param string $data Binary Parquet file contents.
     * @return Document Parsed HEDL document.
     * @throws HedlException If parsing fails.
     */
    public static function fromParquet(string $data): Document
    {
        $ffi = Library::get();
        $docPtr = $ffi->new('HedlDocument*');

        // Create a buffer for binary data
        $buffer = $ffi->new("uint8_t[" . strlen($data) . "]", false);
        FFI::memcpy($buffer, $data, strlen($data));

        $result = $ffi->hedl_from_parquet($buffer, strlen($data), FFI::addr($docPtr));
        if ($result !== ErrorCode::OK) {
            throw Library::createException($result);
        }
        return new Document($docPtr);
    }

    /**
     * Set the library path.
     *
     * @param string $path Path to the libhedl_ffi shared library.
     */
    public static function setLibraryPath(string $path): void
    {
        Library::setLibraryPath($path);
    }

    // Async API (AMPHP-based)
    // IMPORTANT: These methods use cooperative multitasking, not true async.
    // The underlying FFI operations remain blocking. Use these for better code
    // structure, error handling, and timeout integration with other async I/O.

    /**
     * Asynchronously parse HEDL content into a Document.
     *
     * Returns a Promise that resolves to a Document instance.
     * Useful for integrating HEDL parsing into async workflows.
     *
     * Example:
     *   $promise = Hedl::parseAsync($hedlContent);
     *   $doc = Amp\call(function() use ($promise) {
     *       return yield $promise;
     *   });
     *
     * @return Promise<Document> Promise resolving to Document
     */
    public static function parseAsync(string $content, bool $strict = true): object
    {
        $deferred = new Deferred();

        try {
            $doc = self::parse($content, $strict);
            $deferred->resolve($doc);
        } catch (Throwable $e) {
            $deferred->fail($e);
        }

        return $deferred->promise();
    }

    /**
     * Asynchronously validate HEDL content.
     *
     * Returns a Promise that resolves to true if valid, false otherwise.
     *
     * @return Promise<bool> Promise resolving to validation result
     */
    public static function validateAsync(string $content, bool $strict = true): object
    {
        $deferred = new Deferred();

        try {
            $result = self::validate($content, $strict);
            $deferred->resolve($result);
        } catch (Throwable $e) {
            $deferred->fail($e);
        }

        return $deferred->promise();
    }

    /**
     * Asynchronously parse JSON content into a HEDL Document.
     *
     * @return Promise<Document> Promise resolving to Document
     */
    public static function fromJsonAsync(string $content): object
    {
        $deferred = new Deferred();

        try {
            $doc = self::fromJson($content);
            $deferred->resolve($doc);
        } catch (Throwable $e) {
            $deferred->fail($e);
        }

        return $deferred->promise();
    }

    /**
     * Asynchronously parse YAML content into a HEDL Document.
     *
     * @return Promise<Document> Promise resolving to Document
     */
    public static function fromYamlAsync(string $content): object
    {
        $deferred = new Deferred();

        try {
            $doc = self::fromYaml($content);
            $deferred->resolve($doc);
        } catch (Throwable $e) {
            $deferred->fail($e);
        }

        return $deferred->promise();
    }

    /**
     * Asynchronously parse XML content into a HEDL Document.
     *
     * @return Promise<Document> Promise resolving to Document
     */
    public static function fromXmlAsync(string $content): object
    {
        $deferred = new Deferred();

        try {
            $doc = self::fromXml($content);
            $deferred->resolve($doc);
        } catch (Throwable $e) {
            $deferred->fail($e);
        }

        return $deferred->promise();
    }

    /**
     * Asynchronously parse Parquet binary data into a HEDL Document.
     *
     * @return Promise<Document> Promise resolving to Document
     */
    public static function fromParquetAsync(string $data): object
    {
        $deferred = new Deferred();

        try {
            $doc = self::fromParquet($data);
            $deferred->resolve($doc);
        } catch (Throwable $e) {
            $deferred->fail($e);
        }

        return $deferred->promise();
    }

    /**
     * Wrap a Document in an AsyncDocument for async operations.
     *
     * This allows convenient use of async conversion methods on an existing
     * Document instance.
     *
     * Example:
     *   $doc = Hedl::parse($content);
     *   $asyncDoc = Hedl::wrapAsync($doc);
     *   $json = Amp\call(function() use ($asyncDoc) {
     *       return yield $asyncDoc->toJsonAsync();
     *   });
     *
     * @return AsyncDocument Wrapped document for async operations
     */
    public static function wrapAsync(Document $document): AsyncDocument
    {
        return new AsyncDocument($document);
    }
}
