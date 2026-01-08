/**
 * Common test fixtures loader for HEDL Node.js bindings.
 *
 * This module provides access to shared test fixtures stored in the
 * bindings/common/fixtures directory, eliminating test data duplication
 * across language bindings.
 */

import * as fs from 'fs';
import * as path from 'path';

/**
 * Manifest structure for fixture definitions.
 */
interface FixtureManifest {
  fixtures: {
    [key: string]: {
      description: string;
      files: {
        [format: string]: string;
      };
    };
  };
  errors: {
    [key: string]: {
      description: string;
      file: string;
      expected_error: boolean;
    };
  };
}

/**
 * Loads and provides access to common HEDL test fixtures.
 *
 * All fixtures are loaded from bindings/common/fixtures directory
 * to ensure consistency across language bindings.
 */
export class Fixtures {
  private fixturesDir: string;
  private manifest: FixtureManifest;

  /**
   * Initialize the fixtures loader and load manifest.
   */
  constructor() {
    // Path to common fixtures directory
    this.fixturesDir = path.join(__dirname, '..', '..', '..', 'common', 'fixtures');

    // Load manifest
    const manifestPath = path.join(this.fixturesDir, 'manifest.json');
    const manifestContent = fs.readFileSync(manifestPath, 'utf-8');
    this.manifest = JSON.parse(manifestContent);
  }

  /**
   * Read a fixture file.
   *
   * @param filename - Name of the file to read
   * @param binary - If true, read as binary buffer
   * @returns File contents as string or Buffer
   */
  private readFile(filename: string, binary: boolean = false): string | Buffer {
    const filepath = path.join(this.fixturesDir, filename);
    if (binary) {
      return fs.readFileSync(filepath);
    } else {
      return fs.readFileSync(filepath, 'utf-8');
    }
  }

  // Basic fixtures

  /**
   * Get basic HEDL sample document.
   */
  get basicHedl(): string {
    return this.readFile(this.manifest.fixtures.basic.files.hedl) as string;
  }

  /**
   * Get basic JSON sample document.
   */
  get basicJson(): string {
    return this.readFile(this.manifest.fixtures.basic.files.json) as string;
  }

  /**
   * Get basic YAML sample document.
   */
  get basicYaml(): string {
    return this.readFile(this.manifest.fixtures.basic.files.yaml) as string;
  }

  /**
   * Get basic XML sample document.
   */
  get basicXml(): string {
    return this.readFile(this.manifest.fixtures.basic.files.xml) as string;
  }

  // Type-specific fixtures

  /**
   * Get HEDL document with various scalar types.
   */
  get scalarsHedl(): string {
    return this.readFile(this.manifest.fixtures.scalars.files.hedl) as string;
  }

  /**
   * Get HEDL document with nested structures.
   */
  get nestedHedl(): string {
    return this.readFile(this.manifest.fixtures.nested.files.hedl) as string;
  }

  /**
   * Get HEDL document with lists and arrays.
   */
  get listsHedl(): string {
    return this.readFile(this.manifest.fixtures.lists.files.hedl) as string;
  }

  // Performance fixtures

  /**
   * Get large HEDL document for performance testing.
   */
  get largeHedl(): string {
    return this.readFile(this.manifest.fixtures.large.files.hedl) as string;
  }

  // Error fixtures

  /**
   * Get invalid HEDL syntax for error testing.
   */
  get errorInvalidSyntax(): string {
    return this.readFile(this.manifest.errors.invalid_syntax.file) as string;
  }

  /**
   * Get malformed HEDL document for error testing.
   */
  get errorMalformed(): string {
    return this.readFile(this.manifest.errors.malformed.file) as string;
  }

  // Utility methods

  /**
   * Get a specific fixture by category and name.
   *
   * @param category - Fixture category ("basic", "scalars", etc.)
   * @param name - Fixture name (same as category for most)
   * @param format - File format ("hedl", "json", "yaml", "xml")
   * @returns Fixture content as string
   *
   * @example
   * ```typescript
   * const fixtures = new Fixtures();
   * const hedl = fixtures.getFixture("basic", "basic", "hedl");
   * ```
   */
  getFixture(category: string, name: string = category, format: string = 'hedl'): string {
    if (category in this.manifest.fixtures) {
      const files = this.manifest.fixtures[category].files;
      if (format in files) {
        return this.readFile(files[format]) as string;
      }
    }

    throw new Error(`Fixture not found: category=${category}, format=${format}`);
  }

  /**
   * Get an error fixture by type.
   *
   * @param errorType - Type of error ("invalid_syntax", "malformed")
   * @returns Error fixture content
   */
  getErrorFixture(errorType: string): string {
    if (errorType in this.manifest.errors) {
      return this.readFile(this.manifest.errors[errorType].file) as string;
    }

    throw new Error(`Error fixture not found: ${errorType}`);
  }
}

// Global fixtures instance for convenient access
export const fixtures = new Fixtures();

// Legacy constants for backward compatibility
export const SAMPLE_HEDL = fixtures.basicHedl;
export const SAMPLE_JSON = fixtures.basicJson;
export const SAMPLE_YAML = fixtures.basicYaml;
export const SAMPLE_XML = fixtures.basicXml;
