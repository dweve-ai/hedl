#!/usr/bin/env node

/**
 * HEDL File Converter
 *
 * Converts between HEDL and JSON formats
 *
 * Usage:
 *   node node-convert.js input.hedl output.json       # HEDL → JSON
 *   node node-convert.js input.json output.hedl       # JSON → HEDL
 *   node node-convert.js input.hedl --validate        # Validate only
 *   node node-convert.js input.hedl --stats           # Show statistics
 */

const fs = require('fs');
const path = require('path');
const { parse, fromJson, validate, getStats, version } = require('hedl-wasm');

function showHelp() {
  console.log(`
HEDL File Converter v${version()}

Usage:
  node node-convert.js <input> <output>           Convert file
  node node-convert.js <input> --validate         Validate HEDL
  node node-convert.js <input> --stats            Show statistics
  node node-convert.js <input> --format           Format HEDL
  node node-convert.js --help                     Show this help

Examples:
  node node-convert.js data.hedl data.json        HEDL → JSON
  node node-convert.js data.json data.hedl        JSON → HEDL
  node node-convert.js data.hedl --validate       Validate
  node node-convert.js data.hedl --stats          Statistics
`);
}

function detectFormat(filePath) {
  const ext = path.extname(filePath).toLowerCase();
  if (ext === '.hedl') return 'hedl';
  if (ext === '.json') return 'json';

  // Try to detect from content
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    if (content.trim().startsWith('{') || content.trim().startsWith('[')) {
      return 'json';
    }
    if (content.includes('%VERSION')) {
      return 'hedl';
    }
  } catch (e) {
    // Ignore
  }

  return null;
}

function validateFile(inputPath) {
  console.log(`Validating: ${inputPath}`);

  const content = fs.readFileSync(inputPath, 'utf8');
  const result = validate(content, true);

  if (result.valid) {
    console.log('✓ Validation passed');

    if (result.warnings.length > 0) {
      console.log(`\nWarnings (${result.warnings.length}):`);
      result.warnings.forEach(w => {
        console.log(`  Line ${w.line}: ${w.message} [${w.rule}]`);
      });
    } else {
      console.log('  No warnings');
    }

    process.exit(0);
  } else {
    console.log('✗ Validation failed');
    console.log(`\nErrors (${result.errors.length}):`);
    result.errors.forEach(e => {
      console.error(`  Line ${e.line}: ${e.message} (${e.type})`);
    });

    if (result.warnings.length > 0) {
      console.log(`\nWarnings (${result.warnings.length}):`);
      result.warnings.forEach(w => {
        console.log(`  Line ${w.line}: ${w.message} [${w.rule}]`);
      });
    }

    process.exit(1);
  }
}

function showStats(inputPath) {
  console.log(`Statistics for: ${inputPath}`);

  const content = fs.readFileSync(inputPath, 'utf8');
  const doc = parse(content);
  const stats = getStats(content);

  console.log('\nDocument:');
  console.log(`  Version:      ${doc.version}`);
  console.log(`  Schemas:      ${doc.schemaCount}`);
  console.log(`  Aliases:      ${doc.aliasCount}`);
  console.log(`  Nests:        ${doc.nestCount}`);
  console.log(`  Root items:   ${doc.rootItemCount}`);

  const entities = doc.countEntities();
  const totalEntities = Object.values(entities).reduce((a, b) => a + b, 0);
  console.log(`  Total entities: ${totalEntities}`);

  if (Object.keys(entities).length > 0) {
    console.log('\nEntities:');
    for (const [type, count] of Object.entries(entities)) {
      console.log(`  ${type}: ${count}`);
    }
  }

  console.log('\nTokens:');
  console.log(`  HEDL:    ${stats.hedlTokens} tokens (${stats.hedlBytes} bytes, ${stats.hedlLines} lines)`);
  console.log(`  JSON:    ${stats.jsonTokens} tokens (${stats.jsonBytes} bytes)`);
  console.log(`  Savings: ${stats.tokensSaved} tokens (${stats.savingsPercent}%)`);

  process.exit(0);
}

function formatFile(inputPath, outputPath) {
  console.log(`Formatting: ${inputPath}`);

  const content = fs.readFileSync(inputPath, 'utf8');
  const formatted = require('hedl-wasm').format(content, true);

  if (outputPath) {
    fs.writeFileSync(outputPath, formatted, 'utf8');
    console.log(`✓ Formatted and saved to: ${outputPath}`);
  } else {
    console.log('\nFormatted output:');
    console.log(formatted);
  }

  process.exit(0);
}

function convertFile(inputPath, outputPath) {
  const inputFormat = detectFormat(inputPath);
  if (!inputFormat) {
    console.error(`Error: Could not detect format of ${inputPath}`);
    console.error('Use .hedl or .json extension, or specify format explicitly');
    process.exit(1);
  }

  const outputFormat = detectFormat(outputPath);
  if (!outputFormat) {
    console.error(`Error: Could not detect format of ${outputPath}`);
    console.error('Use .hedl or .json extension');
    process.exit(1);
  }

  console.log(`Converting: ${inputPath} (${inputFormat}) → ${outputPath} (${outputFormat})`);

  const content = fs.readFileSync(inputPath, 'utf8');

  try {
    let output;

    if (inputFormat === 'hedl' && outputFormat === 'json') {
      // HEDL → JSON
      const doc = parse(content);
      output = doc.toJsonString(true);
    } else if (inputFormat === 'json' && outputFormat === 'hedl') {
      // JSON → HEDL
      output = fromJson(content, true);
    } else if (inputFormat === outputFormat) {
      // Same format - just validate and format
      if (inputFormat === 'hedl') {
        output = require('hedl-wasm').format(content, true);
      } else {
        const json = JSON.parse(content);
        output = JSON.stringify(json, null, 2);
      }
    } else {
      console.error(`Error: Unsupported conversion ${inputFormat} → ${outputFormat}`);
      process.exit(1);
    }

    fs.writeFileSync(outputPath, output, 'utf8');
    console.log(`✓ Conversion successful`);
    console.log(`  Input:  ${content.length} bytes`);
    console.log(`  Output: ${output.length} bytes`);

    // Show token savings for HEDL → JSON
    if (inputFormat === 'hedl') {
      const stats = getStats(content);
      console.log(`  Token savings: ${stats.savingsPercent}%`);
    }

  } catch (e) {
    console.error(`\nConversion failed: ${e.message}`);
    process.exit(1);
  }
}

function main() {
  const args = process.argv.slice(2);

  if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
    showHelp();
    process.exit(0);
  }

  const inputPath = args[0];

  // Check if input file exists
  if (!fs.existsSync(inputPath)) {
    console.error(`Error: Input file not found: ${inputPath}`);
    process.exit(1);
  }

  // Special commands
  if (args.length === 2) {
    const command = args[1];

    if (command === '--validate' || command === '-v') {
      validateFile(inputPath);
      return;
    }

    if (command === '--stats' || command === '-s') {
      showStats(inputPath);
      return;
    }

    if (command === '--format' || command === '-f') {
      formatFile(inputPath);
      return;
    }
  }

  if (args.length === 3 && (args[1] === '--format' || args[1] === '-f')) {
    formatFile(inputPath, args[2]);
    return;
  }

  // Convert
  if (args.length < 2) {
    console.error('Error: Output file required');
    console.error('Usage: node node-convert.js <input> <output>');
    process.exit(1);
  }

  const outputPath = args[1];
  convertFile(inputPath, outputPath);
}

// Run
try {
  main();
} catch (e) {
  console.error('Fatal error:', e.message);
  if (process.env.DEBUG) {
    console.error(e.stack);
  }
  process.exit(1);
}
