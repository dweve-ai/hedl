/**
 * HEDL (Hierarchical Entity Data Language) Node.js Bindings
 *
 * A token-efficient data format for LLM context optimization.
 *
 * @example
 * ```typescript
 * import * as hedl from 'hedl';
 *
 * const doc = hedl.parse(`
 * %VERSION: 1.0
 * %STRUCT: User: [id, name]
 * ---
 * users: @User
 *   | alice, Alice Smith
 * `);
 *
 * console.log(doc.version); // [1, 0]
 * console.log(doc.toJson());
 * doc.close();
 * ```
 *
 * THREAD SAFETY WARNING:
 * =====================
 * These bindings are NOT thread-safe. Document and Diagnostics objects must
 * not be accessed concurrently from multiple threads/workers. The underlying
 * FFI library does not perform any internal locking. If you need concurrent
 * access to HEDL documents, you must:
 *
 * 1. Use separate Document instances per worker thread, OR
 * 2. Implement your own synchronization (mutexes, semaphores) around all
 *    Document and Diagnostics method calls
 *
 * Concurrent access without proper synchronization may result in:
 * - Memory corruption
 * - Use-after-free errors
 * - Segmentation faults
 * - Undefined behavior
 *
 * Node.js is single-threaded by default, so this is typically not an issue
 * unless you're using Worker threads or the cluster module with shared state.
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
 * Set before loading hedl:
 *
 *   // In your shell (before running Node.js)
 *   export HEDL_MAX_OUTPUT_SIZE=1073741824  // 1 GB
 *
 *   // Or in Node.js (must be set BEFORE import)
 *   process.env.HEDL_MAX_OUTPUT_SIZE = '1073741824';  // 1 GB
 *   import * as hedl from 'hedl';
 *
 * Use Cases:
 * - Small configs: 10-50 MB (default may suffice)
 * - Medium datasets: 100-500 MB (set to 524288000 for 500 MB)
 * - Large datasets: 500 MB - 5 GB (set to 1073741824+ for 1 GB+)
 * - No practical limit: set to a very high value like 10737418240 (10 GB)
 *
 * When the limit is exceeded, operations will throw HedlError with code
 * HEDL_ERR_ALLOC and a message suggesting to increase HEDL_MAX_OUTPUT_SIZE.
 */

import * as koffi from 'koffi';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

// Resource limits
// Default is 100MB, which may be too restrictive for many real-world scenarios.
// Recommended: 500MB-1GB for data processing, higher for large datasets.
// Set HEDL_MAX_OUTPUT_SIZE environment variable before importing to customize.
const MAX_OUTPUT_SIZE = parseInt(process.env.HEDL_MAX_OUTPUT_SIZE || '104857600', 10); // 100MB default

// Error codes
export const HEDL_OK = 0;
export const HEDL_ERR_NULL_PTR = -1;
export const HEDL_ERR_INVALID_UTF8 = -2;
export const HEDL_ERR_PARSE = -3;
export const HEDL_ERR_CANONICALIZE = -4;
export const HEDL_ERR_JSON = -5;
export const HEDL_ERR_ALLOC = -6;
export const HEDL_ERR_YAML = -7;
export const HEDL_ERR_XML = -8;
export const HEDL_ERR_CSV = -9;
export const HEDL_ERR_PARQUET = -10;
export const HEDL_ERR_LINT = -11;
export const HEDL_ERR_NEO4J = -12;

// Severity levels
export const SEVERITY_HINT = 0;
export const SEVERITY_WARNING = 1;
export const SEVERITY_ERROR = 2;

/**
 * Find the HEDL shared library.
 */
function findLibrary(): string {
  const platform = os.platform();
  let libName: string;

  if (platform === 'darwin') {
    libName = 'libhedl_ffi.dylib';
  } else if (platform === 'win32') {
    libName = 'hedl_ffi.dll';
  } else {
    libName = 'libhedl_ffi.so';
  }

  // Check environment variable
  const envPath = process.env.HEDL_LIB_PATH;
  if (envPath) {
    if (fs.existsSync(envPath)) {
      const stat = fs.statSync(envPath);
      if (stat.isFile()) return envPath;
      const libPath = path.join(envPath, libName);
      if (fs.existsSync(libPath)) return libPath;
    }
  }

  // Check module directory
  const moduleLib = path.join(__dirname, '..', libName);
  if (fs.existsSync(moduleLib)) return moduleLib;

  // Check development paths
  const devPaths = [
    path.join(__dirname, '..', '..', '..', 'target', 'release', libName),
    path.join(__dirname, '..', '..', '..', 'target', 'debug', libName),
  ];

  for (const devPath of devPaths) {
    if (fs.existsSync(devPath)) return devPath;
  }

  // Return name for system path search
  return libName;
}

// Load the library
let lib: koffi.IKoffiLib | null = null;
let funcs: {
  hedl_get_last_error: koffi.KoffiFunction;
  hedl_free_string: koffi.KoffiFunction;
  hedl_free_document: koffi.KoffiFunction;
  hedl_free_diagnostics: koffi.KoffiFunction;
  hedl_free_bytes: koffi.KoffiFunction;
  hedl_parse: koffi.KoffiFunction;
  hedl_validate: koffi.KoffiFunction;
  hedl_get_version: koffi.KoffiFunction;
  hedl_schema_count: koffi.KoffiFunction;
  hedl_alias_count: koffi.KoffiFunction;
  hedl_root_item_count: koffi.KoffiFunction;
  hedl_canonicalize: koffi.KoffiFunction;
  hedl_to_json: koffi.KoffiFunction;
  hedl_from_json: koffi.KoffiFunction;
  hedl_to_yaml: koffi.KoffiFunction;
  hedl_from_yaml: koffi.KoffiFunction;
  hedl_to_xml: koffi.KoffiFunction;
  hedl_from_xml: koffi.KoffiFunction;
  hedl_to_csv: koffi.KoffiFunction;
  hedl_to_parquet: koffi.KoffiFunction;
  hedl_from_parquet: koffi.KoffiFunction;
  hedl_to_neo4j_cypher: koffi.KoffiFunction;
  hedl_lint: koffi.KoffiFunction;
  hedl_diagnostics_count: koffi.KoffiFunction;
  hedl_diagnostics_get: koffi.KoffiFunction;
  hedl_diagnostics_severity: koffi.KoffiFunction;
} | null = null;

function getLib() {
  if (!lib || !funcs) {
    const libPath = findLibrary();
    lib = koffi.load(libPath);

    funcs = {
      hedl_get_last_error: lib.func('hedl_get_last_error', 'str', []),
      hedl_free_string: lib.func('hedl_free_string', 'void', ['void *']),
      hedl_free_document: lib.func('hedl_free_document', 'void', ['void *']),
      hedl_free_diagnostics: lib.func('hedl_free_diagnostics', 'void', ['void *']),
      hedl_free_bytes: lib.func('hedl_free_bytes', 'void', ['void *', 'size_t']),
      hedl_parse: lib.func('hedl_parse', 'int', ['str', 'int', 'int', '_Out_ void **']),
      hedl_validate: lib.func('hedl_validate', 'int', ['str', 'int', 'int']),
      hedl_get_version: lib.func('hedl_get_version', 'int', ['void *', '_Out_ int *', '_Out_ int *']),
      hedl_schema_count: lib.func('hedl_schema_count', 'int', ['void *']),
      hedl_alias_count: lib.func('hedl_alias_count', 'int', ['void *']),
      hedl_root_item_count: lib.func('hedl_root_item_count', 'int', ['void *']),
      hedl_canonicalize: lib.func('hedl_canonicalize', 'int', ['void *', '_Out_ void **']),
      hedl_to_json: lib.func('hedl_to_json', 'int', ['void *', 'int', '_Out_ void **']),
      hedl_from_json: lib.func('hedl_from_json', 'int', ['str', 'int', '_Out_ void **']),
      hedl_to_yaml: lib.func('hedl_to_yaml', 'int', ['void *', 'int', '_Out_ void **']),
      hedl_from_yaml: lib.func('hedl_from_yaml', 'int', ['str', 'int', '_Out_ void **']),
      hedl_to_xml: lib.func('hedl_to_xml', 'int', ['void *', '_Out_ void **']),
      hedl_from_xml: lib.func('hedl_from_xml', 'int', ['str', 'int', '_Out_ void **']),
      hedl_to_csv: lib.func('hedl_to_csv', 'int', ['void *', '_Out_ void **']),
      hedl_to_parquet: lib.func('hedl_to_parquet', 'int', ['void *', '_Out_ void **', '_Out_ size_t *']),
      hedl_from_parquet: lib.func('hedl_from_parquet', 'int', ['void *', 'size_t', '_Out_ void **']),
      hedl_to_neo4j_cypher: lib.func('hedl_to_neo4j_cypher', 'int', ['void *', 'int', '_Out_ void **']),
      hedl_lint: lib.func('hedl_lint', 'int', ['void *', '_Out_ void **']),
      hedl_diagnostics_count: lib.func('hedl_diagnostics_count', 'int', ['void *']),
      hedl_diagnostics_get: lib.func('hedl_diagnostics_get', 'int', ['void *', 'int', '_Out_ void **']),
      hedl_diagnostics_severity: lib.func('hedl_diagnostics_severity', 'int', ['void *', 'int']),
    };
  }
  return funcs;
}

/**
 * Error thrown by HEDL operations.
 */
export class HedlError extends Error {
  code: number;

  constructor(message: string, code: number = HEDL_ERR_PARSE) {
    super(message);
    this.name = 'HedlError';
    this.code = code;
  }

  static fromLib(code: number): HedlError {
    const lib = getLib();
    const errorMsg = lib.hedl_get_last_error();
    const message = errorMsg || `HEDL error code ${code}`;
    return new HedlError(message, code);
  }
}

/**
 * Check if output size exceeds the limit.
 */
function checkOutputSize(output: string | Buffer): void {
  const outputSize = typeof output === 'string' ? Buffer.byteLength(output, 'utf8') : output.length;
  if (outputSize > MAX_OUTPUT_SIZE) {
    const actualMb = (outputSize / 1048576).toFixed(2);
    const limitMb = (MAX_OUTPUT_SIZE / 1048576).toFixed(2);
    throw new HedlError(
      `Output size (${actualMb}MB) exceeds limit (${limitMb}MB). Set HEDL_MAX_OUTPUT_SIZE to increase.`,
      HEDL_ERR_ALLOC
    );
  }
}

/**
 * Lint diagnostics container.
 */
export class Diagnostics {
  private ptr: any;
  private closed: boolean = false;

  constructor(ptr: any) {
    this.ptr = ptr;
  }

  /**
   * Get the number of diagnostics.
   */
  get length(): number {
    if (this.closed) return 0;
    const count = getLib().hedl_diagnostics_count(this.ptr);
    return Math.max(0, count);
  }

  /**
   * Get diagnostic at index.
   */
  get(index: number): { message: string; severity: number } {
    if (this.closed) {
      throw new HedlError('Diagnostics already closed', HEDL_ERR_NULL_PTR);
    }

    if (index < 0 || index >= this.length) {
      throw new RangeError(`Diagnostic index ${index} out of range`);
    }

    const lib = getLib();
    const msgPtr: any[] = [null];
    const result = lib.hedl_diagnostics_get(this.ptr, index, msgPtr);

    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }

    const message = koffi.decode(msgPtr[0], 'char', -1);
    lib.hedl_free_string(msgPtr[0]);

    const severity = lib.hedl_diagnostics_severity(this.ptr, index);
    return { message, severity };
  }

  /**
   * Get all diagnostics.
   */
  all(): Array<{ message: string; severity: number }> {
    const results: Array<{ message: string; severity: number }> = [];
    for (let i = 0; i < this.length; i++) {
      results.push(this.get(i));
    }
    return results;
  }

  /**
   * Get all error messages.
   */
  get errors(): string[] {
    return this.all()
      .filter((d) => d.severity === SEVERITY_ERROR)
      .map((d) => d.message);
  }

  /**
   * Get all warning messages.
   */
  get warnings(): string[] {
    return this.all()
      .filter((d) => d.severity === SEVERITY_WARNING)
      .map((d) => d.message);
  }

  /**
   * Get all hint messages.
   */
  get hints(): string[] {
    return this.all()
      .filter((d) => d.severity === SEVERITY_HINT)
      .map((d) => d.message);
  }

  /**
   * Free resources.
   */
  close(): void {
    if (!this.closed && this.ptr) {
      getLib().hedl_free_diagnostics(this.ptr);
      this.closed = true;
    }
  }
}

/**
 * A parsed HEDL document.
 */
export class Document {
  private ptr: any;
  private closed: boolean = false;

  constructor(ptr: any) {
    this.ptr = ptr;
  }

  private checkClosed(): void {
    if (this.closed) {
      throw new HedlError('Document already closed', HEDL_ERR_NULL_PTR);
    }
  }

  /**
   * Get the HEDL version as [major, minor] tuple.
   */
  get version(): [number, number] {
    this.checkClosed();
    const lib = getLib();
    const major: number[] = [0];
    const minor: number[] = [0];
    const result = lib.hedl_get_version(this.ptr, major, minor);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    return [major[0], minor[0]];
  }

  /**
   * Get the number of schema definitions.
   */
  get schemaCount(): number {
    this.checkClosed();
    const count = getLib().hedl_schema_count(this.ptr);
    if (count < 0) throw HedlError.fromLib(count);
    return count;
  }

  /**
   * Get the number of alias definitions.
   */
  get aliasCount(): number {
    this.checkClosed();
    const count = getLib().hedl_alias_count(this.ptr);
    if (count < 0) throw HedlError.fromLib(count);
    return count;
  }

  /**
   * Get the number of root items.
   */
  get rootItemCount(): number {
    this.checkClosed();
    const count = getLib().hedl_root_item_count(this.ptr);
    if (count < 0) throw HedlError.fromLib(count);
    return count;
  }

  /**
   * Convert to canonical HEDL form.
   */
  canonicalize(): string {
    this.checkClosed();
    const lib = getLib();
    const outPtr: any[] = [null];
    const result = lib.hedl_canonicalize(this.ptr, outPtr);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    const output = koffi.decode(outPtr[0], 'char', -1);
    lib.hedl_free_string(outPtr[0]);
    checkOutputSize(output);
    return output;
  }

  /**
   * Convert to JSON.
   */
  toJson(includeMetadata: boolean = false): string {
    this.checkClosed();
    const lib = getLib();
    const outPtr: any[] = [null];
    const result = lib.hedl_to_json(this.ptr, includeMetadata ? 1 : 0, outPtr);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    const output = koffi.decode(outPtr[0], 'char', -1);
    lib.hedl_free_string(outPtr[0]);
    checkOutputSize(output);
    return output;
  }

  /**
   * Convert to YAML.
   */
  toYaml(includeMetadata: boolean = false): string {
    this.checkClosed();
    const lib = getLib();
    const outPtr: any[] = [null];
    const result = lib.hedl_to_yaml(this.ptr, includeMetadata ? 1 : 0, outPtr);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    const output = koffi.decode(outPtr[0], 'char', -1);
    lib.hedl_free_string(outPtr[0]);
    checkOutputSize(output);
    return output;
  }

  /**
   * Convert to XML.
   */
  toXml(): string {
    this.checkClosed();
    const lib = getLib();
    const outPtr: any[] = [null];
    const result = lib.hedl_to_xml(this.ptr, outPtr);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    const output = koffi.decode(outPtr[0], 'char', -1);
    lib.hedl_free_string(outPtr[0]);
    checkOutputSize(output);
    return output;
  }

  /**
   * Convert to CSV.
   */
  toCsv(): string {
    this.checkClosed();
    const lib = getLib();
    const outPtr: any[] = [null];
    const result = lib.hedl_to_csv(this.ptr, outPtr);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    const output = koffi.decode(outPtr[0], 'char', -1);
    lib.hedl_free_string(outPtr[0]);
    checkOutputSize(output);
    return output;
  }

  /**
   * Convert to Parquet.
   */
  toParquet(): Buffer {
    this.checkClosed();
    const lib = getLib();
    const dataPtr: any[] = [null];
    const lenPtr: bigint[] = [BigInt(0)];
    const result = lib.hedl_to_parquet(this.ptr, dataPtr, lenPtr);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    const len = Number(lenPtr[0]);
    const buffer = Buffer.from(koffi.decode(dataPtr[0], koffi.array('uint8', len)));
    lib.hedl_free_bytes(dataPtr[0], len);
    checkOutputSize(buffer);
    return buffer;
  }

  /**
   * Convert to Neo4j Cypher queries.
   */
  toCypher(useMerge: boolean = true): string {
    this.checkClosed();
    const lib = getLib();
    const outPtr: any[] = [null];
    const result = lib.hedl_to_neo4j_cypher(this.ptr, useMerge ? 1 : 0, outPtr);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    const output = koffi.decode(outPtr[0], 'char', -1);
    lib.hedl_free_string(outPtr[0]);
    checkOutputSize(output);
    return output;
  }

  /**
   * Run linting on the document.
   */
  lint(): Diagnostics {
    this.checkClosed();
    const lib = getLib();
    const diagPtr: any[] = [null];
    const result = lib.hedl_lint(this.ptr, diagPtr);
    if (result !== HEDL_OK) {
      throw HedlError.fromLib(result);
    }
    return new Diagnostics(diagPtr[0]);
  }

  /**
   * Free document resources.
   */
  close(): void {
    if (!this.closed && this.ptr) {
      getLib().hedl_free_document(this.ptr);
      this.closed = true;
    }
  }
}

/**
 * Parse HEDL content into a Document.
 *
 * @param content - HEDL content as string
 * @param strict - Enable strict reference validation
 * @returns Parsed Document object
 *
 * @example
 * ```typescript
 * const doc = hedl.parse('%VERSION: 1.0\n---\nkey: value');
 * console.log(doc.version); // [1, 0]
 * doc.close();
 * ```
 */
export function parse(content: string, strict: boolean = true): Document {
  const lib = getLib();
  const docPtr: any[] = [null];
  const result = lib.hedl_parse(content, content.length, strict ? 1 : 0, docPtr);
  if (result !== HEDL_OK) {
    throw HedlError.fromLib(result);
  }
  return new Document(docPtr[0]);
}

/**
 * Validate HEDL content without creating a document.
 *
 * @param content - HEDL content as string
 * @param strict - Enable strict reference validation
 * @returns true if valid, false otherwise
 */
export function validate(content: string, strict: boolean = true): boolean {
  const lib = getLib();
  const result = lib.hedl_validate(content, content.length, strict ? 1 : 0);
  return result === HEDL_OK;
}

/**
 * Parse JSON content into a HEDL Document.
 */
export function fromJson(content: string): Document {
  const lib = getLib();
  const docPtr: any[] = [null];
  const result = lib.hedl_from_json(content, content.length, docPtr);
  if (result !== HEDL_OK) {
    throw HedlError.fromLib(result);
  }
  return new Document(docPtr[0]);
}

/**
 * Parse YAML content into a HEDL Document.
 */
export function fromYaml(content: string): Document {
  const lib = getLib();
  const docPtr: any[] = [null];
  const result = lib.hedl_from_yaml(content, content.length, docPtr);
  if (result !== HEDL_OK) {
    throw HedlError.fromLib(result);
  }
  return new Document(docPtr[0]);
}

/**
 * Parse XML content into a HEDL Document.
 */
export function fromXml(content: string): Document {
  const lib = getLib();
  const docPtr: any[] = [null];
  const result = lib.hedl_from_xml(content, content.length, docPtr);
  if (result !== HEDL_OK) {
    throw HedlError.fromLib(result);
  }
  return new Document(docPtr[0]);
}

/**
 * Parse Parquet content into a HEDL Document.
 */
export function fromParquet(data: Buffer): Document {
  const lib = getLib();
  const docPtr: any[] = [null];
  const result = lib.hedl_from_parquet(data, data.length, docPtr);
  if (result !== HEDL_OK) {
    throw HedlError.fromLib(result);
  }
  return new Document(docPtr[0]);
}

/**
 * Async variant of parse() using setImmediate for non-blocking execution.
 *
 * Parse HEDL content into a Document asynchronously.
 * This prevents blocking the event loop for large documents.
 *
 * @param content - HEDL content as string
 * @param strict - Enable strict reference validation
 * @returns Promise that resolves to a Document
 *
 * @example
 * ```typescript
 * const doc = await hedl.parseAsync('%VERSION: 1.0\n---\nkey: value');
 * console.log(doc.version); // [1, 0]
 * doc.close();
 * ```
 */
export function parseAsync(content: string, strict: boolean = true): Promise<Document> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        const result = parse(content, strict);
        resolve(result);
      } catch (err) {
        reject(err);
      }
    });
  });
}

/**
 * Async variant of validate() using setImmediate for non-blocking execution.
 *
 * Validate HEDL content without creating a document, asynchronously.
 * This prevents blocking the event loop for large documents.
 *
 * @param content - HEDL content as string
 * @param strict - Enable strict reference validation
 * @returns Promise that resolves to a boolean
 *
 * @example
 * ```typescript
 * const isValid = await hedl.validateAsync('%VERSION: 1.0\n---\nkey: value');
 * console.log(isValid); // true
 * ```
 */
export function validateAsync(content: string, strict: boolean = true): Promise<boolean> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        const result = validate(content, strict);
        resolve(result);
      } catch (err) {
        reject(err);
      }
    });
  });
}

/**
 * Async variant of fromJson() using setImmediate for non-blocking execution.
 *
 * Parse JSON content into a HEDL Document asynchronously.
 * This prevents blocking the event loop for large documents.
 *
 * @param content - JSON content as string
 * @returns Promise that resolves to a Document
 *
 * @example
 * ```typescript
 * const doc = await hedl.fromJsonAsync('{"key": "value"}');
 * console.log(doc.toJson());
 * doc.close();
 * ```
 */
export function fromJsonAsync(content: string): Promise<Document> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        const result = fromJson(content);
        resolve(result);
      } catch (err) {
        reject(err);
      }
    });
  });
}

/**
 * Async variant of fromYaml() using setImmediate for non-blocking execution.
 *
 * Parse YAML content into a HEDL Document asynchronously.
 * This prevents blocking the event loop for large documents.
 *
 * @param content - YAML content as string
 * @returns Promise that resolves to a Document
 *
 * @example
 * ```typescript
 * const doc = await hedl.fromYamlAsync('key: value');
 * doc.close();
 * ```
 */
export function fromYamlAsync(content: string): Promise<Document> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        const result = fromYaml(content);
        resolve(result);
      } catch (err) {
        reject(err);
      }
    });
  });
}

/**
 * Async variant of fromXml() using setImmediate for non-blocking execution.
 *
 * Parse XML content into a HEDL Document asynchronously.
 * This prevents blocking the event loop for large documents.
 *
 * @param content - XML content as string
 * @returns Promise that resolves to a Document
 *
 * @example
 * ```typescript
 * const doc = await hedl.fromXmlAsync('<root><key>value</key></root>');
 * doc.close();
 * ```
 */
export function fromXmlAsync(content: string): Promise<Document> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        const result = fromXml(content);
        resolve(result);
      } catch (err) {
        reject(err);
      }
    });
  });
}

/**
 * Async variant of fromParquet() using setImmediate for non-blocking execution.
 *
 * Parse Parquet content into a HEDL Document asynchronously.
 * This prevents blocking the event loop for large documents.
 *
 * @param data - Parquet data as Buffer
 * @returns Promise that resolves to a Document
 *
 * @example
 * ```typescript
 * const buffer = fs.readFileSync('data.parquet');
 * const doc = await hedl.fromParquetAsync(buffer);
 * doc.close();
 * ```
 */
export function fromParquetAsync(data: Buffer): Promise<Document> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        const result = fromParquet(data);
        resolve(result);
      } catch (err) {
        reject(err);
      }
    });
  });
}

/**
 * Async conversion helper interface for Document methods.
 * Provides async variants of blocking conversion operations on a Document.
 */
export interface AsyncDocumentMethods {
  /**
   * Async variant of canonicalize().
   * Convert to canonical HEDL form asynchronously.
   */
  canonicalizeAsync(): Promise<string>;

  /**
   * Async variant of toJson().
   * Convert to JSON asynchronously.
   */
  toJsonAsync(includeMetadata?: boolean): Promise<string>;

  /**
   * Async variant of toYaml().
   * Convert to YAML asynchronously.
   */
  toYamlAsync(includeMetadata?: boolean): Promise<string>;

  /**
   * Async variant of toXml().
   * Convert to XML asynchronously.
   */
  toXmlAsync(): Promise<string>;

  /**
   * Async variant of toCsv().
   * Convert to CSV asynchronously.
   */
  toCsvAsync(): Promise<string>;

  /**
   * Async variant of toParquet().
   * Convert to Parquet asynchronously.
   */
  toParquetAsync(): Promise<Buffer>;

  /**
   * Async variant of toCypher().
   * Convert to Neo4j Cypher queries asynchronously.
   */
  toCypherAsync(useMerge?: boolean): Promise<string>;

  /**
   * Async variant of lint().
   * Run linting asynchronously.
   */
  lintAsync(): Promise<Diagnostics>;
}

/**
 * Extend Document class with async method signatures.
 */
declare global {
  interface Document extends AsyncDocumentMethods {}
}

/**
 * Add async methods to Document prototype.
 * This extends the Document class with async variants of all blocking operations.
 */
(Document.prototype as any).canonicalizeAsync = function (this: Document): Promise<string> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        resolve(this.canonicalize());
      } catch (err) {
        reject(err);
      }
    });
  });
};

(Document.prototype as any).toJsonAsync = function (
  this: Document,
  includeMetadata: boolean = false
): Promise<string> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        resolve(this.toJson(includeMetadata));
      } catch (err) {
        reject(err);
      }
    });
  });
};

(Document.prototype as any).toYamlAsync = function (
  this: Document,
  includeMetadata: boolean = false
): Promise<string> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        resolve(this.toYaml(includeMetadata));
      } catch (err) {
        reject(err);
      }
    });
  });
};

(Document.prototype as any).toXmlAsync = function (this: Document): Promise<string> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        resolve(this.toXml());
      } catch (err) {
        reject(err);
      }
    });
  });
};

(Document.prototype as any).toCsvAsync = function (this: Document): Promise<string> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        resolve(this.toCsv());
      } catch (err) {
        reject(err);
      }
    });
  });
};

(Document.prototype as any).toParquetAsync = function (this: Document): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        resolve(this.toParquet());
      } catch (err) {
        reject(err);
      }
    });
  });
};

(Document.prototype as any).toCypherAsync = function (
  this: Document,
  useMerge: boolean = true
): Promise<string> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        resolve(this.toCypher(useMerge));
      } catch (err) {
        reject(err);
      }
    });
  });
};

(Document.prototype as any).lintAsync = function (this: Document): Promise<Diagnostics> {
  return new Promise((resolve, reject) => {
    setImmediate(() => {
      try {
        resolve(this.lint());
      } catch (err) {
        reject(err);
      }
    });
  });
};
