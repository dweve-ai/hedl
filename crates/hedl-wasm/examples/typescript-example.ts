// Example TypeScript usage demonstrating improved type safety
// This file showcases the JsonValue type system and type-safe API usage

import {
  init,
  parse,
  toJson,
  fromJson,
  getStats,
  JsonValue,
  JsonObject,
  HedlDocument,
  EntityResult
} from '../hedl.d';

async function main() {
  // Initialize WASM module
  await init();

  // Example HEDL document
  const hedl = `
%VERSION: 1.0
%STRUCT: User: [id, name, email, age]
%STRUCT: Post: [id, title, content]
%NEST: User > Post
---
users: @User
  | alice | Alice Smith | alice@example.com | 30
    | post1 | Hello World | My first post
    | post2 | TypeScript | Love the type safety
  | bob | Bob Jones | bob@example.com | 25
    | post3 | WASM rocks | WebAssembly is amazing
`;

  // Parse with type safety
  const doc: HedlDocument = parse(hedl);

  // ============================================================================
  // Document Metadata (with type inference)
  // ============================================================================

  console.log('HEDL Version:', doc.version);  // Type: string
  console.log('Schema Count:', doc.schemaCount);  // Type: number
  console.log('Root Items:', doc.rootItemCount);  // Type: number

  // Get schema names (Type: string[])
  const schemas: string[] = doc.getSchemaNames();
  console.log('Schemas:', schemas);

  // Get specific schema (Type: string[] | undefined)
  const userSchema = doc.getSchema('User');
  if (userSchema) {
    console.log('User columns:', userSchema);
  }

  // ============================================================================
  // JSON Conversion with JsonValue Type Safety
  // ============================================================================

  // Convert to JSON with explicit JsonValue type (NO MORE 'any'!)
  const jsonValue: JsonValue = doc.toJson();

  // Type-safe JSON operations
  if (typeof jsonValue === 'object' && jsonValue !== null && !Array.isArray(jsonValue)) {
    const jsonObj = jsonValue as JsonObject;

    // Access nested data with type safety
    const usersData = jsonObj.users;
    console.log('Users data:', usersData);

    // TypeScript knows this is JsonValue, not 'any'
    if (Array.isArray(usersData)) {
      console.log('Number of users:', usersData.length);
    }
  }

  // Convert to JSON string
  const jsonString: string = doc.toJsonString(true);
  console.log('JSON output:\n', jsonString);

  // ============================================================================
  // Entity Queries with Type-Safe Fields
  // ============================================================================

  // Query all users (Type: EntityResult[])
  const allUsers: EntityResult[] = doc.query('User');

  for (const user of allUsers) {
    // EntityResult has typed fields now (JsonValue instead of any)
    console.log('User:', user.type, user.id);

    // Access fields with type safety
    const name = user.fields.name;  // Type: JsonValue (not any!)
    const email = user.fields.email;  // Type: JsonValue

    // Type-safe value checking
    if (typeof name === 'string') {
      console.log('  Name:', name);
    }
    if (typeof email === 'string') {
      console.log('  Email:', email);
    }
  }

  // Query specific user
  const alice: EntityResult[] = doc.query('User', 'alice');
  if (alice.length > 0) {
    const aliceData = alice[0];
    console.log('Alice found:', aliceData.fields);
  }

  // ============================================================================
  // Count Entities with Type Safety
  // ============================================================================

  // Type: Record<string, number>
  const counts: Record<string, number> = doc.countEntities();
  console.log('Entity counts:', counts);
  console.log('  Users:', counts['User']);
  console.log('  Posts:', counts['Post']);

  // ============================================================================
  // Token Statistics
  // ============================================================================

  const stats = getStats(hedl);
  console.log('\nToken Statistics:');
  console.log('  HEDL tokens:', stats.hedlTokens);
  console.log('  JSON tokens:', stats.jsonTokens);
  console.log('  Savings:', stats.savingsPercent + '%');
  console.log('  Tokens saved:', stats.tokensSaved);

  // ============================================================================
  // Round-trip Conversion
  // ============================================================================

  // JSON → HEDL → JSON (all type-safe)
  const jsonInput = '{"name": "Test", "value": 42}';
  const hedlOutput: string = fromJson(jsonInput, true);
  console.log('\nJSON to HEDL:\n', hedlOutput);

  const backToJson: string = toJson(hedlOutput, true);
  console.log('\nBack to JSON:\n', backToJson);

  // ============================================================================
  // Type-Safe Helper Functions
  // ============================================================================

  // Extract string value from JsonValue
  function getStringField(entity: EntityResult, field: string): string | null {
    const value = entity.fields[field];
    return typeof value === 'string' ? value : null;
  }

  // Extract number value from JsonValue
  function getNumberField(entity: EntityResult, field: string): number | null {
    const value = entity.fields[field];
    return typeof value === 'number' ? value : null;
  }

  // Extract object value from JsonValue
  function getObjectField(entity: EntityResult, field: string): JsonObject | null {
    const value = entity.fields[field];
    return (typeof value === 'object' && value !== null && !Array.isArray(value))
      ? value as JsonObject
      : null;
  }

  // Use type-safe helpers
  for (const user of allUsers) {
    const name = getStringField(user, 'name');
    const age = getNumberField(user, 'age');

    if (name && age !== null) {
      console.log(`${name} is ${age} years old`);
    }
  }

  // ============================================================================
  // Type Guards for JsonValue
  // ============================================================================

  function isJsonObject(value: JsonValue): value is JsonObject {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
  }

  function isJsonArray(value: JsonValue): value is JsonValue[] {
    return Array.isArray(value);
  }

  function isJsonPrimitive(value: JsonValue): value is string | number | boolean | null {
    return value === null ||
           typeof value === 'string' ||
           typeof value === 'number' ||
           typeof value === 'boolean';
  }

  // Use type guards
  const jsonData = doc.toJson();
  if (isJsonObject(jsonData)) {
    console.log('Root is an object');
    for (const [key, value] of Object.entries(jsonData)) {
      console.log(`  ${key}: ${typeof value}`);
    }
  }
}

// ============================================================================
// Compile-Time Type Checking Examples
// ============================================================================

// These examples demonstrate TypeScript's compile-time type checking
// Uncomment to see type errors:

/*
// ❌ Type error: Cannot assign 'any' to 'JsonValue'
const doc = parse(hedl);
const badJson: any = doc.toJson();  // Discouraged, use JsonValue instead

// ❌ Type error: fields is Record<string, JsonValue>, not Record<string, any>
const entity: EntityResult = {
  type: 'User',
  id: 'test',
  fields: { name: 123 as any }  // Type error without 'as any'
};

// ✅ Correct: Explicit JsonValue typing
const entity: EntityResult = {
  type: 'User',
  id: 'test',
  fields: {
    name: 'Alice' as JsonValue,  // Explicitly typed as JsonValue
    age: 30 as JsonValue
  }
};
*/

// Run the example
main().catch(console.error);

// ============================================================================
// Key Benefits of JsonValue Type System:
// ============================================================================
//
// 1. NO MORE 'any' TYPES
//    - All JSON operations are type-safe
//    - TypeScript can catch errors at compile time
//
// 2. BETTER IDE SUPPORT
//    - Autocomplete works correctly
//    - Inline documentation via JSDoc
//    - Type hints for all values
//
// 3. REFACTORING SAFETY
//    - Type changes propagate correctly
//    - Breaking changes caught early
//    - Easier to maintain code
//
// 4. SELF-DOCUMENTING
//    - Type definitions explain structure
//    - Clear contracts between code
//    - Reduced need for comments
//
// 5. INDUSTRY STANDARD
//    - Follows TypeScript best practices
//    - Compatible with other JSON libraries
//    - Professional-grade type system
//
// ============================================================================
