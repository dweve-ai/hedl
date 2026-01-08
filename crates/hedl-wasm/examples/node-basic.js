#!/usr/bin/env node

/**
 * Basic Node.js example for hedl-wasm
 *
 * Demonstrates:
 * - File I/O with HEDL
 * - Parsing and conversion
 * - Error handling
 * - Statistics reporting
 */

const fs = require('fs');
const path = require('path');
const { parse, toJson, fromJson, format, validate, getStats, version } = require('hedl-wasm');

// Sample HEDL document
const sampleHedl = `%VERSION: 1.0
%STRUCT: Product[id,name,price,stock]
%STRUCT: Category[id,name]
%NEST: Category > Product
%ALIAS: %inStock: "true"
---
categories: @Category
  |electronics,Electronics
    |p001,Laptop,999.99,25
    |p002,Mouse,24.99,150
    |p003,Keyboard,79.99,75
  |books,Books
    |p004,HEDL Guide,29.99,100
    |p005,Data Science,49.99,50

settings:
  currency: "USD"
  taxRate: 0.08
  inStock: %inStock
`;

function main() {
  console.log('='.repeat(60));
  console.log(`HEDL WASM Node.js Example - v${version()}`);
  console.log('='.repeat(60));
  console.log();

  // 1. Parse HEDL
  console.log('1. Parsing HEDL...');
  let doc;
  try {
    doc = parse(sampleHedl);
    console.log(`   ✓ Parsed successfully`);
    console.log(`   - Version: ${doc.version}`);
    console.log(`   - Schemas: ${doc.schemaCount}`);
    console.log(`   - Root items: ${doc.rootItemCount}`);
    console.log();
  } catch (e) {
    console.error(`   ✗ Parse failed: ${e.message}`);
    process.exit(1);
  }

  // 2. Validate
  console.log('2. Validating HEDL...');
  const validation = validate(sampleHedl, true);
  if (validation.valid) {
    console.log(`   ✓ Validation passed`);
    if (validation.warnings.length > 0) {
      console.log(`   - Warnings: ${validation.warnings.length}`);
      validation.warnings.forEach(w => {
        console.log(`     Line ${w.line}: ${w.message}`);
      });
    }
  } else {
    console.log(`   ✗ Validation failed`);
    validation.errors.forEach(e => {
      console.error(`     Line ${e.line}: ${e.message} (${e.type})`);
    });
  }
  console.log();

  // 3. Convert to JSON
  console.log('3. Converting to JSON...');
  try {
    const json = doc.toJsonString(true);
    console.log(`   ✓ Converted to JSON (${json.length} bytes)`);
    console.log();
    console.log('   First 200 characters:');
    console.log('   ' + json.substring(0, 200).replace(/\n/g, '\n   ') + '...');
    console.log();
  } catch (e) {
    console.error(`   ✗ Conversion failed: ${e.message}`);
  }

  // 4. Query entities
  console.log('4. Querying entities...');
  const products = doc.query('Product');
  console.log(`   ✓ Found ${products.length} products`);
  products.forEach(p => {
    const name = p.fields.name || 'unknown';
    const price = p.fields.price || 0;
    console.log(`     - ${p.id}: ${name} ($${price})`);
  });
  console.log();

  // 5. Count entities
  console.log('5. Counting entities...');
  const counts = doc.countEntities();
  for (const [type, count] of Object.entries(counts)) {
    console.log(`   - ${type}: ${count}`);
  }
  console.log();

  // 6. Get statistics
  console.log('6. Token statistics...');
  const stats = getStats(sampleHedl);
  console.log(`   HEDL:    ${stats.hedlTokens} tokens (${stats.hedlBytes} bytes, ${stats.hedlLines} lines)`);
  console.log(`   JSON:    ${stats.jsonTokens} tokens (${stats.jsonBytes} bytes)`);
  console.log(`   Savings: ${stats.tokensSaved} tokens (${stats.savingsPercent}%)`);
  console.log();

  // 7. Format HEDL
  console.log('7. Formatting HEDL...');
  try {
    const formatted = format(sampleHedl, true);
    console.log(`   ✓ Formatted (${formatted.length} bytes)`);
    console.log();
  } catch (e) {
    console.error(`   ✗ Format failed: ${e.message}`);
  }

  // 8. Round-trip conversion
  console.log('8. Round-trip conversion (HEDL → JSON → HEDL)...');
  try {
    const jsonStr = doc.toJsonString(false);
    const backToHedl = fromJson(jsonStr, true);
    const reparsed = parse(backToHedl);

    console.log(`   ✓ Round-trip successful`);
    console.log(`   - Original schemas: ${doc.schemaCount}`);
    console.log(`   - Reparsed schemas: ${reparsed.schemaCount}`);
    console.log();
  } catch (e) {
    console.error(`   ✗ Round-trip failed: ${e.message}`);
  }

  // 9. File operations
  console.log('9. File operations...');

  // Write HEDL to file
  const hedlPath = path.join(__dirname, 'output-sample.hedl');
  fs.writeFileSync(hedlPath, sampleHedl, 'utf8');
  console.log(`   ✓ Written to ${hedlPath}`);

  // Write JSON to file
  const jsonPath = path.join(__dirname, 'output-sample.json');
  const jsonStr = doc.toJsonString(true);
  fs.writeFileSync(jsonPath, jsonStr, 'utf8');
  console.log(`   ✓ Written to ${jsonPath}`);

  // Read back and verify
  const readHedl = fs.readFileSync(hedlPath, 'utf8');
  const readDoc = parse(readHedl);
  console.log(`   ✓ Read back and verified (${readDoc.rootItemCount} items)`);
  console.log();

  // 10. Schema inspection
  console.log('10. Schema inspection...');
  const schemas = doc.getSchemaNames();
  schemas.forEach(name => {
    const schema = doc.getSchema(name);
    console.log(`   ${name}: [${schema.join(', ')}]`);
  });
  console.log();

  console.log('='.repeat(60));
  console.log('All operations completed successfully!');
  console.log('='.repeat(60));
}

// Error handling
try {
  main();
} catch (e) {
  console.error('Fatal error:', e.message);
  console.error(e.stack);
  process.exit(1);
}
