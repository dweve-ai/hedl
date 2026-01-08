// Dweve HEDL - Hierarchical Entity Data Language
// TypeScript definitions for hedl-wasm

// ============================================================================
// JSON Type Definitions
// ============================================================================

/**
 * Represents a JSON primitive value.
 */
export type JsonPrimitive = string | number | boolean | null;

/**
 * Represents a JSON array (recursive).
 */
export type JsonArray = JsonValue[];

/**
 * Represents a JSON object (recursive).
 */
export type JsonObject = { [key: string]: JsonValue };

/**
 * Represents any valid JSON value.
 * This is a recursive type that can represent any JSON structure.
 */
export type JsonValue = JsonPrimitive | JsonObject | JsonArray;

// ============================================================================
// Initialization
// ============================================================================

/**
 * Initialize the WASM module. Call this before using any other functions.
 */
export function init(): Promise<void>;

/**
 * Get the HEDL library version.
 */
export function version(): string;

/**
 * Set the maximum input size in bytes.
 *
 * This controls the maximum size of HEDL/JSON input strings that can be processed.
 * Default is 500 MB (524,288,000 bytes). Set to a higher value if you need to process larger documents.
 *
 * @param size - Maximum input size in bytes
 *
 * @example
 * ```typescript
 * import { setMaxInputSize } from 'hedl-wasm';
 *
 * // Allow processing up to 1 GB documents
 * setMaxInputSize(1024 * 1024 * 1024);
 * ```
 */
export function setMaxInputSize(size: number): void;

/**
 * Get the current maximum input size in bytes.
 *
 * @returns Current maximum input size setting (default: 524,288,000 bytes = 500 MB)
 *
 * @example
 * ```typescript
 * import { getMaxInputSize } from 'hedl-wasm';
 *
 * const currentLimit = getMaxInputSize();
 * console.log(`Current limit: ${currentLimit / (1024 * 1024)} MB`);
 * ```
 */
export function getMaxInputSize(): number;

/**
 * Parse a HEDL string and return a document object.
 * @param input - HEDL document string
 * @throws Error if parsing fails or input exceeds the configured maximum size (default: 500 MB)
 */
export function parse(input: string): HedlDocument;

/**
 * Convert HEDL string to JSON string.
 * @param hedl - HEDL document string
 * @param pretty - Whether to pretty-print (default: true)
 * @throws Error if parsing fails or input exceeds the configured maximum size (default: 500 MB)
 */
export function toJson(hedl: string, pretty?: boolean): string;

/**
 * Convert JSON string to HEDL string.
 * @param json - JSON string to convert
 * @param useDitto - Enable ditto optimization (default: true)
 * @throws Error if conversion fails or input exceeds the configured maximum size (default: 500 MB)
 */
export function fromJson(json: string, useDitto?: boolean): string;

/**
 * Format HEDL to canonical form.
 * @param hedl - HEDL document string
 * @param useDitto - Enable ditto optimization (default: true)
 * @throws Error if formatting fails or input exceeds the configured maximum size (default: 500 MB)
 */
export function format(hedl: string, useDitto?: boolean): string;

/**
 * Validate HEDL and return detailed diagnostics.
 * @param hedl - HEDL document string
 * @param runLint - Run linting rules (default: true)
 * @returns Validation result with any errors/warnings. Input size errors are reported in the errors array.
 */
export function validate(hedl: string, runLint?: boolean): ValidationResult;

/**
 * Get token usage statistics for a HEDL document.
 * @param hedl - HEDL document string
 * @throws Error if parsing fails or input exceeds the configured maximum size (default: 500 MB)
 */
export function getStats(hedl: string): TokenStats;

/**
 * Compare token counts between HEDL and JSON.
 * @param hedl - HEDL document string
 * @param json - JSON string
 */
export function compareTokens(hedl: string, json: string): TokenComparison;

/**
 * Parsed HEDL document.
 */
export class HedlDocument {
  /** HEDL version (e.g., "1.0") */
  readonly version: string;

  /** Number of schema definitions */
  readonly schemaCount: number;

  /** Number of alias definitions */
  readonly aliasCount: number;

  /** Number of nest relationships */
  readonly nestCount: number;

  /** Number of root items */
  readonly rootItemCount: number;

  /** Get all schema type names */
  getSchemaNames(): string[];

  /** Get schema columns for a type */
  getSchema(typeName: string): string[] | undefined;

  /**
   * Get all aliases as a map.
   * @returns Map of alias names to their resolved values
   */
  getAliases(): Record<string, string>;

  /**
   * Get all nest relationships as a map.
   * @returns Map of parent types to their child types
   */
  getNests(): Record<string, string>;

  /**
   * Convert to JSON object.
   * @returns The document as a structured JSON value
   */
  toJson(): JsonValue;

  /**
   * Convert to JSON string.
   * @param pretty - Whether to pretty-print the JSON (default: true)
   * @returns JSON string representation of the document
   */
  toJsonString(pretty?: boolean): string;

  /** Convert to canonical HEDL string */
  toHedl(useDitto?: boolean): string;

  /** Count entities by type */
  countEntities(): Record<string, number>;

  /** Query entities by type and/or ID */
  query(typeName?: string, id?: string): EntityResult[];
}

/**
 * Validation result.
 */
export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

/**
 * Validation error.
 */
export interface ValidationError {
  line: number;
  message: string;
  type: string;
}

/**
 * Validation warning.
 */
export interface ValidationWarning {
  line: number;
  message: string;
  rule: string;
}

/**
 * Token usage statistics.
 */
export interface TokenStats {
  hedlBytes: number;
  hedlTokens: number;
  hedlLines: number;
  jsonBytes: number;
  jsonTokens: number;
  savingsPercent: number;
  tokensSaved: number;
}

/**
 * Token comparison result.
 */
export interface TokenComparison {
  hedl: {
    bytes: number;
    tokens: number;
    lines: number;
  };
  json: {
    bytes: number;
    tokens: number;
  };
  savings: {
    percent: number;
    tokens: number;
  };
}

/**
 * Entity query result.
 */
export interface EntityResult {
  /** Entity type name */
  type: string;
  /** Entity ID */
  id: string;
  /** Entity field values mapped by field name */
  fields: Record<string, JsonValue>;
}
