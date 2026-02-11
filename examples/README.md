# HEDL Examples

This directory contains working examples that demonstrate HEDL's features and common use cases. Each file is a valid, parseable document you can run through the CLI or use as a starting point for your own data.

## Getting Started

The simplest way to explore these examples is to validate and inspect them:

```bash
# Validate an example
cargo run -p hedl-cli -- validate examples/basic.hedl

# See how HEDL parsed it
cargo run -p hedl-cli -- inspect examples/basic.hedl

# Convert to JSON to see the structure
cargo run -p hedl-cli -- to-json examples/basic.hedl
```

## Basic Examples

**basic.hedl** shows the minimum structure every HEDL document needs: the required header directives followed by simple key-value pairs. If you're new to HEDL, start here.

**scalars.hedl** demonstrates all the scalar types: strings (both quoted and unquoted), integers, floats with scientific notation, booleans, and null values. Useful as a reference when you're unsure about the syntax for a particular type.

**nested.hedl** shows how to build hierarchical structures using indentation. HEDL uses exactly one space per nesting level, which keeps things compact while remaining readable. This example has multiple levels of nesting to show the pattern.

## Working with Structured Data

**matrix_list.hedl** is where HEDL really shines. Instead of repeating field names for every record (like JSON does), you define a schema once and then list data in a compact, CSV-like format. This example shows how to define schemas with `%S:TypeName:[columns]` and then populate them with `|` rows.

**references.hedl** demonstrates HEDL's reference system. When you write `@User:u1`, you're creating a typed reference to another entity. This is invaluable for representing relationships without the ambiguity of foreign keys stored as plain strings.

**schema.hedl** combines schemas with nested relationships using `%N:Parent>Child` declarations. This lets you represent hierarchical data like organizations with departments, blog posts with comments, or orders with line items.

## Advanced Features

**tensors.hedl** covers tensor literals for ML and scientific computing workflows. You can represent vectors, matrices, and higher-dimensional arrays directly in HEDL, making it a natural choice for ML configuration and data pipelines.

**expressions.hedl** shows the expression syntax `$(...)` for embedding computations or placeholders. The parser treats these as opaque tokens -it doesn't evaluate them, just preserves them for your runtime to interpret. Useful for template values or deferred computation.

**aliases.hedl** demonstrates the alias system. Define a value once with `%A:key:value`, then reference it throughout your document with `$key`. Great for configuration files where the same URL, path, or constant appears in multiple places.

## Real-World Use Cases

**config.hedl** is a realistic application configuration file with database settings, cache configuration, API endpoints, and monitoring options. It shows how HEDL's features combine for practical configuration management.

**llm_context.hedl** demonstrates HEDL's primary use case: packing more information into fewer tokens when feeding context to language models. This example includes conversation history, few-shot examples, and system prompts in a format that uses roughly half the tokens of equivalent JSON.

**data_pipeline.hedl** shows an ML pipeline configuration with stages, transformations, validators, and model settings. The compact syntax makes it easy to see the entire pipeline structure at a glance.

**api_response.hedl** represents a typical REST API response with pagination, nested resources, and relational data. It shows how HEDL handles the kinds of structures you'd typically serialize as JSON.

**knowledge_graph.hedl** demonstrates graph-like data with concepts, relationships, and properties. HEDL's reference system makes it natural to represent knowledge graphs, entity relationships, and semantic networks.

## Feature Reference

| Feature | Where to find it |
|---------|------------------|
| Required headers | All files |
| Simple key-value pairs | basic.hedl, scalars.hedl |
| Nested objects | nested.hedl, config.hedl |
| Schemas and matrix lists | matrix_list.hedl, schema.hedl, references.hedl |
| Tensors | tensors.hedl |
| Entity references | references.hedl, schema.hedl, api_response.hedl |
| Expressions | expressions.hedl, config.hedl |
| Aliases | aliases.hedl |
| Graph relationships | knowledge_graph.hedl, schema.hedl |

## Syntax Quick Reference

Every HEDL document starts with a header section, separated from the body by `---`:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
# Your data goes here
```

The three header directives are required. They tell the parser the document version, what character represents null (typically `~`), and what character is used for quoting strings (typically `"`).

Indentation uses exactly one space per level. Matrix rows start with `|`. References use `@Type:id` syntax. Comments start with `#`. Expressions use `$(...)` for deferred evaluation. Aliases expand `$key` to the value defined in `%A:key:value`.

## Why HEDL Uses Fewer Tokens

HEDL achieves roughly 56% token reduction compared to JSON through several mechanisms:

Matrix lists eliminate the repetition of field names. In JSON, a list of 1000 users repeats `"id"`, `"name"`, `"email"` a thousand times each. In HEDL, you define the schema once and the data is just values.

Implicit structure removes closing brackets. Where JSON needs `}` and `]` to close every object and array, HEDL uses indentation, so the structure is inherent in the formatting.

Aliases reduce repetition of common values. If your configuration references the same API endpoint in ten places, you define it once and reference it by name.

For LLM applications where you're paying per token, this efficiency translates directly to cost savings and the ability to fit more context into limited windows.

## Further Reading

The [SPEC.md](../SPEC.md) file contains the complete HEDL specification with formal grammar and detailed semantics.

The [tests/conformance/](../tests/conformance/) directory has test cases that define expected parser behavior for edge cases and error conditions.
