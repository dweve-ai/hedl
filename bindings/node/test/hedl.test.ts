/**
 * Tests for HEDL Node.js bindings.
 */

import * as hedl from '../src/index';
import { fixtures } from './fixtures';

// Use shared fixtures from common/fixtures directory
const SAMPLE_HEDL = fixtures.basicHedl;
const SAMPLE_JSON = fixtures.basicJson;
const SAMPLE_YAML = fixtures.basicYaml;
const SAMPLE_XML = fixtures.basicXml;

describe('hedl.parse', () => {
  test('parses valid HEDL content', () => {
    const doc = hedl.parse(SAMPLE_HEDL);
    expect(doc).toBeInstanceOf(hedl.Document);
    doc.close();
  });

  test('throws on invalid HEDL content', () => {
    expect(() => hedl.parse(fixtures.errorInvalidSyntax)).toThrow(hedl.HedlError);
  });
});

describe('hedl.validate', () => {
  test('returns true for valid content', () => {
    expect(hedl.validate(SAMPLE_HEDL)).toBe(true);
  });

  test('returns false for invalid content', () => {
    expect(hedl.validate(fixtures.errorInvalidSyntax)).toBe(false);
  });
});

describe('Document properties', () => {
  let doc: hedl.Document;

  beforeEach(() => {
    doc = hedl.parse(SAMPLE_HEDL);
  });

  afterEach(() => {
    doc.close();
  });

  test('version returns [major, minor]', () => {
    expect(doc.version).toEqual([1, 0]);
  });

  test('schemaCount returns number', () => {
    expect(doc.schemaCount).toBe(1);
  });

  test('rootItemCount returns number', () => {
    expect(doc.rootItemCount).toBeGreaterThanOrEqual(1);
  });
});

describe('Document conversions', () => {
  let doc: hedl.Document;

  beforeEach(() => {
    doc = hedl.parse(SAMPLE_HEDL);
  });

  afterEach(() => {
    doc.close();
  });

  test('canonicalize returns HEDL string', () => {
    const canonical = doc.canonicalize();
    expect(typeof canonical).toBe('string');
    expect(canonical).toContain('%VERSION');
  });

  test('toJson returns JSON string', () => {
    const json = doc.toJson();
    expect(typeof json).toBe('string');
    expect(json).toContain('users');
    JSON.parse(json); // Should not throw
  });

  test('toJson with metadata', () => {
    const json = doc.toJson(true);
    expect(typeof json).toBe('string');
  });

  test('toYaml returns YAML string', () => {
    const yaml = doc.toYaml();
    expect(typeof yaml).toBe('string');
  });

  test('toXml returns XML string', () => {
    const xml = doc.toXml();
    expect(typeof xml).toBe('string');
    expect(xml).toContain('<');
  });

  test('toCsv returns CSV string', () => {
    const csv = doc.toCsv();
    expect(typeof csv).toBe('string');
  });

  test('toCypher returns Cypher string', () => {
    const cypher = doc.toCypher();
    expect(typeof cypher).toBe('string');
  });

  test('toParquet returns Buffer', () => {
    const parquet = doc.toParquet();
    expect(Buffer.isBuffer(parquet)).toBe(true);
    expect(parquet.length).toBeGreaterThan(0);
  });
});

describe('Format conversion', () => {
  test('fromJson parses JSON', () => {
    const doc = hedl.fromJson(SAMPLE_JSON);
    expect(doc).toBeInstanceOf(hedl.Document);
    const hedlStr = doc.canonicalize();
    expect(hedlStr).toContain('users');
    doc.close();
  });

  test('fromYaml parses YAML', () => {
    const doc = hedl.fromYaml(SAMPLE_YAML);
    expect(doc).toBeInstanceOf(hedl.Document);
    doc.close();
  });

  test('fromXml parses XML', () => {
    const doc = hedl.fromXml(SAMPLE_XML);
    expect(doc).toBeInstanceOf(hedl.Document);
    doc.close();
  });
});

describe('Linting', () => {
  test('lint returns Diagnostics', () => {
    const doc = hedl.parse(SAMPLE_HEDL);
    const diag = doc.lint();
    expect(diag).toBeInstanceOf(hedl.Diagnostics);
    expect(diag.length).toBeGreaterThanOrEqual(0);
    diag.close();
    doc.close();
  });

  test('Diagnostics.all returns array', () => {
    const doc = hedl.parse(SAMPLE_HEDL);
    const diag = doc.lint();
    const items = diag.all();
    expect(Array.isArray(items)).toBe(true);
    diag.close();
    doc.close();
  });
});

describe('Memory management', () => {
  test('close can be called multiple times', () => {
    const doc = hedl.parse(SAMPLE_HEDL);
    doc.close();
    doc.close(); // Should not throw
  });

  test('throws when using closed document', () => {
    const doc = hedl.parse(SAMPLE_HEDL);
    doc.close();
    expect(() => doc.toJson()).toThrow(hedl.HedlError);
  });
});
