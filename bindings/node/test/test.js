/**
 * Simple tests for HEDL Node.js bindings
 */

// Set library path before requiring hedl
process.env.HEDL_LIB_PATH = require('path').join(__dirname, '..', '..', '..', 'target', 'release', 'libhedl_ffi.so');

const hedl = require('../dist/index.js');

const SAMPLE_HEDL = `%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
`;

const SAMPLE_JSON = '{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}';

const SAMPLE_YAML = `users:
  - id: 1
    name: Alice
  - id: 2
    name: Bob
`;

const SAMPLE_XML = `<?xml version="1.0"?>
<root>
  <users>
    <item><id>1</id><name>Alice</name></item>
    <item><id>2</id><name>Bob</name></item>
  </users>
</root>
`;

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    console.log(`✓ ${name}`);
    passed++;
  } catch (err) {
    console.log(`✗ ${name}: ${err.message}`);
    failed++;
  }
}

function assertEqual(actual, expected, message) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function assertTrue(value, message) {
  if (!value) {
    throw new Error(message || 'Expected true');
  }
}

function assertFalse(value, message) {
  if (value) {
    throw new Error(message || 'Expected false');
  }
}

function assertThrows(fn, message) {
  try {
    fn();
    throw new Error(message || 'Expected exception');
  } catch (err) {
    if (err.message === (message || 'Expected exception')) {
      throw err;
    }
    // Expected exception was thrown
  }
}

async function asyncTest(name, fn) {
  try {
    await fn();
    console.log(`✓ ${name}`);
    passed++;
  } catch (err) {
    console.log(`✗ ${name}: ${err.message}`);
    failed++;
  }
}

// Tests
test('parse - valid content', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  assertTrue(doc instanceof hedl.Document);
  doc.close();
});

test('parse - invalid content throws', () => {
  assertThrows(() => hedl.parse('invalid {{{{'), 'Expected parse to throw');
});

test('validate - valid content', () => {
  assertTrue(hedl.validate(SAMPLE_HEDL), 'Expected valid content to pass');
});

test('validate - invalid content', () => {
  assertFalse(hedl.validate('invalid {{{{'), 'Expected invalid content to fail');
});

test('version', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  assertEqual(doc.version, [1, 0], 'Version should be [1, 0]');
  doc.close();
});

test('schemaCount', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  assertEqual(doc.schemaCount, 1, 'Schema count should be 1');
  doc.close();
});

test('canonicalize', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  const canonical = doc.canonicalize();
  assertTrue(canonical.length > 0, 'Expected non-empty canonical');
  assertTrue(canonical.includes('%VERSION'), 'Expected %VERSION in output');
  doc.close();
});

test('toJson', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  const json = doc.toJson();
  assertTrue(json.length > 0, 'Expected non-empty JSON');
  JSON.parse(json); // Should not throw
  doc.close();
});

test('toYaml', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  const yaml = doc.toYaml();
  assertTrue(yaml.length > 0, 'Expected non-empty YAML');
  doc.close();
});

test('toXml', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  const xml = doc.toXml();
  assertTrue(xml.length > 0, 'Expected non-empty XML');
  assertTrue(xml.includes('<'), 'Expected XML tags');
  doc.close();
});

test('toCsv', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  const csv = doc.toCsv();
  assertTrue(csv.length > 0, 'Expected non-empty CSV');
  doc.close();
});

test('toCypher', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  const cypher = doc.toCypher();
  assertTrue(cypher.length > 0, 'Expected non-empty Cypher');
  doc.close();
});

test('toParquet', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  const data = doc.toParquet();
  assertTrue(Buffer.isBuffer(data), 'Expected Buffer');
  assertTrue(data.length > 0, 'Expected non-empty Parquet');
  doc.close();
});

test('fromJson', () => {
  const doc = hedl.fromJson(SAMPLE_JSON);
  assertTrue(doc instanceof hedl.Document);
  const canonical = doc.canonicalize();
  assertTrue(canonical.length > 0, 'Expected non-empty canonical');
  doc.close();
});

test('fromYaml', () => {
  const doc = hedl.fromYaml(SAMPLE_YAML);
  assertTrue(doc instanceof hedl.Document);
  doc.close();
});

test('fromXml', () => {
  const doc = hedl.fromXml(SAMPLE_XML);
  assertTrue(doc instanceof hedl.Document);
  doc.close();
});

test('lint', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  const diag = doc.lint();
  assertTrue(diag instanceof hedl.Diagnostics);
  assertTrue(diag.length >= 0, 'Expected count >= 0');
  diag.close();
  doc.close();
});

test('double close', () => {
  const doc = hedl.parse(SAMPLE_HEDL);
  doc.close();
  doc.close(); // Should not throw
});

// Async tests
async function runAsyncTests() {
  console.log('\n--- Async API Tests ---\n');

  await asyncTest('parseAsync - valid content', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    assertTrue(doc instanceof hedl.Document);
    doc.close();
  });

  await asyncTest('parseAsync - invalid content throws', async () => {
    try {
      await hedl.parseAsync('invalid {{{{');
      throw new Error('Expected error to be thrown');
    } catch (err) {
      if (!(err instanceof hedl.HedlError)) {
        throw err;
      }
    }
  });

  await asyncTest('validateAsync - valid content', async () => {
    const result = await hedl.validateAsync(SAMPLE_HEDL);
    assertTrue(result, 'Expected valid content to pass');
  });

  await asyncTest('validateAsync - invalid content', async () => {
    const result = await hedl.validateAsync('invalid {{{{');
    assertFalse(result, 'Expected invalid content to fail');
  });

  await asyncTest('fromJsonAsync', async () => {
    const doc = await hedl.fromJsonAsync(SAMPLE_JSON);
    assertTrue(doc instanceof hedl.Document);
    doc.close();
  });

  await asyncTest('fromYamlAsync', async () => {
    const doc = await hedl.fromYamlAsync(SAMPLE_YAML);
    assertTrue(doc instanceof hedl.Document);
    doc.close();
  });

  await asyncTest('fromXmlAsync', async () => {
    const doc = await hedl.fromXmlAsync(SAMPLE_XML);
    assertTrue(doc instanceof hedl.Document);
    doc.close();
  });

  await asyncTest('Document.canonicalizeAsync', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const canonical = await doc.canonicalizeAsync();
    assertTrue(canonical.length > 0, 'Expected non-empty canonical');
    assertTrue(canonical.includes('%VERSION'), 'Expected %VERSION in output');
    doc.close();
  });

  await asyncTest('Document.toJsonAsync', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const json = await doc.toJsonAsync();
    assertTrue(json.length > 0, 'Expected non-empty JSON');
    JSON.parse(json); // Should not throw
    doc.close();
  });

  await asyncTest('Document.toJsonAsync with metadata', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const json = await doc.toJsonAsync(true);
    assertTrue(json.length > 0, 'Expected non-empty JSON');
    doc.close();
  });

  await asyncTest('Document.toYamlAsync', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const yaml = await doc.toYamlAsync();
    assertTrue(yaml.length > 0, 'Expected non-empty YAML');
    doc.close();
  });

  await asyncTest('Document.toXmlAsync', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const xml = await doc.toXmlAsync();
    assertTrue(xml.length > 0, 'Expected non-empty XML');
    assertTrue(xml.includes('<'), 'Expected XML tags');
    doc.close();
  });

  await asyncTest('Document.toCsvAsync', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const csv = await doc.toCsvAsync();
    assertTrue(csv.length > 0, 'Expected non-empty CSV');
    doc.close();
  });

  await asyncTest('Document.toCypherAsync', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const cypher = await doc.toCypherAsync();
    assertTrue(cypher.length > 0, 'Expected non-empty Cypher');
    doc.close();
  });

  await asyncTest('Document.toParquetAsync', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const data = await doc.toParquetAsync();
    assertTrue(Buffer.isBuffer(data), 'Expected Buffer');
    assertTrue(data.length > 0, 'Expected non-empty Parquet');
    doc.close();
  });

  await asyncTest('Document.lintAsync', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const diag = await doc.lintAsync();
    assertTrue(diag instanceof hedl.Diagnostics);
    assertTrue(diag.length >= 0, 'Expected count >= 0');
    diag.close();
    doc.close();
  });

  await asyncTest('fromParquetAsync', async () => {
    const doc1 = await hedl.parseAsync(SAMPLE_HEDL);
    const parquetData = await doc1.toParquetAsync();
    doc1.close();

    const doc2 = await hedl.fromParquetAsync(parquetData);
    assertTrue(doc2 instanceof hedl.Document);
    doc2.close();
  });

  await asyncTest('Async chaining', async () => {
    const doc = await hedl.parseAsync(SAMPLE_HEDL);
    const json = await doc.toJsonAsync();
    const doc2 = await hedl.fromJsonAsync(json);
    const yaml = await doc2.toYamlAsync();
    assertTrue(yaml.length > 0, 'Expected non-empty YAML');
    doc.close();
    doc2.close();
  });
}

// Run async tests and then summarize
(async () => {
  try {
    await runAsyncTests();
  } catch (err) {
    console.error('Error running async tests:', err);
  }

  // Summary
  console.log();
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  process.exit(failed > 0 ? 1 : 0);
})();
