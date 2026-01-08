# HEDL How-To Guides

Practical, goal-oriented guides for accomplishing specific tasks with HEDL. Each guide provides step-by-step instructions for solving real-world problems.

## What are How-To Guides?

How-to guides are **problem-oriented** and focus on accomplishing specific goals. Unlike tutorials (which teach), how-to guides assume you know the basics and want to solve a particular problem.

## Quick Navigation

### Format Conversion
**[Convert Between Formats](convert-formats.md)** - Recipes for converting data between JSON, YAML, XML, CSV, Parquet, and HEDL

### Data Validation
**[Validate Documents](validate-documents.md)** - Ensure data quality and catch errors early

### Error Handling
**[Handle Errors](handle-errors.md)** - Deal with validation errors, malformed data, and conversion issues

### Performance Optimization
**[Optimize Performance](optimize-performance.md)** - Speed up processing, reduce memory usage, and improve throughput

---

## Guide Index

### 1. [Convert Between Formats](convert-formats.md)

Learn how to convert data between different formats effectively.

**Topics covered:**
- JSON ↔ HEDL conversion
- YAML ↔ HEDL conversion
- XML ↔ HEDL conversion
- CSV ↔ HEDL conversion
- Parquet ↔ HEDL conversion
- Multi-step conversions (CSV → HEDL → Parquet)
- Preserving metadata during conversion
- Handling schema differences

**When to use this guide:**
- Converting API responses to HEDL
- Migrating data between formats
- Creating format-agnostic workflows
- Integrating HEDL into existing systems

---

### 2. [Validate Documents](validate-documents.md)

Ensure your HEDL documents are correct and high-quality.

**Topics covered:**
- Syntax validation
- Schema validation
- Reference integrity checking
- Custom validation rules
- Batch validation
- Validation in CI/CD pipelines
- Pre-commit validation hooks
- Validation error reporting

**When to use this guide:**
- Quality assurance workflows
- Data ingestion pipelines
- Pre-deployment checks
- Code review processes

---

### 3. [Handle Errors](handle-errors.md)

Deal with errors gracefully and debug issues effectively.

**Topics covered:**
- Understanding error messages
- Fixing syntax errors
- Resolving validation failures
- Handling conversion errors
- Debugging reference errors
- Working with malformed data
- Error recovery strategies
- Logging and monitoring

**When to use this guide:**
- Troubleshooting validation failures
- Debugging conversion issues
- Building robust error handling
- Creating production-ready workflows

---

### 4. [Optimize Performance](optimize-performance.md)

Improve processing speed, reduce memory usage, and scale efficiently.

**Topics covered:**
- Profiling and benchmarking
- Memory optimization techniques
- Streaming for large files
- Parallel processing
- Caching strategies
- Format-specific optimizations
- Batch processing efficiency
- Resource limit tuning

**When to use this guide:**
- Processing large datasets
- Optimizing production workflows
- Reducing infrastructure costs
- Scaling HEDL operations

---

## How to Use These Guides

### 1. Identify Your Goal

Start with what you want to accomplish:
- "I need to convert JSON to HEDL" → [Convert Formats](convert-formats.md)
- "My validation is failing" → [Handle Errors](handle-errors.md)
- "Processing is too slow" → [Optimize Performance](optimize-performance.md)

### 2. Find the Relevant Section

Each guide is organized by specific tasks. Use the table of contents to jump to the section that matches your need.

### 3. Follow the Steps

Each task includes:
- **Goal:** What you'll accomplish
- **Prerequisites:** What you need before starting
- **Steps:** Specific instructions
- **Example:** Working code/commands
- **Variations:** Alternative approaches
- **Troubleshooting:** Common issues

### 4. Adapt to Your Needs

The examples are templates. Modify them for your specific use case.

## Common Task Quick Reference

### "I want to..."

| Goal | Guide | Section |
|------|-------|---------|
| Convert JSON to HEDL | [Convert Formats](convert-formats.md) | JSON → HEDL |
| Convert HEDL to Parquet | [Convert Formats](convert-formats.md) | HEDL → Parquet |
| Validate a file | [Validate Documents](validate-documents.md) | Basic Validation |
| Fix a validation error | [Handle Errors](handle-errors.md) | Validation Errors |
| Process files faster | [Optimize Performance](optimize-performance.md) | Parallel Processing |
| Handle large files | [Optimize Performance](optimize-performance.md) | Streaming |
| Validate in CI/CD | [Validate Documents](validate-documents.md) | CI/CD Integration |
| Debug a conversion | [Handle Errors](handle-errors.md) | Conversion Errors |

## Complementary Documentation

- **Learning HEDL?** Start with the [Tutorials](../tutorials/)
- **Understanding concepts?** Read the [Concepts](../concepts/) guides
- **Need reference info?** Check the [Reference](../reference/) documentation
- **Have a question?** See the [FAQ](../faq.md)
- **Something broken?** Try [Troubleshooting](../troubleshooting.md)

## Contributing

Have a useful recipe or workflow? We welcome contributions!

1. Create a new section in the relevant guide
2. Follow the existing format:
   - Clear goal statement
   - Prerequisite list
   - Step-by-step instructions
   - Working example
   - Troubleshooting tips
3. Submit a pull request

## Quick Access by Format

### Working with JSON
- [Convert JSON to HEDL](convert-formats.md#json-to-hedl)
- [Convert HEDL to JSON](convert-formats.md#hedl-to-json)
- [Handle JSON errors](handle-errors.md#json-parse-error)

### Working with CSV
- [Import CSV data](convert-formats.md#csv-to-hedl)
- [Export to CSV](convert-formats.md#hedl-to-csv)
- [Handle CSV encoding issues](handle-errors.md#csv-encoding-error)

### Working with Parquet
- [Convert to Parquet](convert-formats.md#hedl-to-parquet)
- [Import from Parquet](convert-formats.md#parquet-to-hedl)
- [Optimize Parquet output](optimize-performance.md#parquet-optimization)

### Working with XML
- [Convert XML to HEDL](convert-formats.md#xml-to-hedl)
- [Generate XML from HEDL](convert-formats.md#hedl-to-xml)
- [Handle XML namespaces](handle-errors.md#conversion-errors)

### Working with YAML
- [Convert YAML to HEDL](convert-formats.md#yaml-to-hedl)
- [Generate YAML](convert-formats.md#hedl-to-yaml)

---

**Ready to solve a problem?** Choose a guide above and get started!
