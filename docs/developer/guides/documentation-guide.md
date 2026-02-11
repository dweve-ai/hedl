# Documentation Guide: Writing Words That Help

Documentation is not an afterthought. It's the bridge between your code and everyone who will ever use it. Write it poorly, and that bridge collapses. Users struggle. Questions flood in. Adoption stalls. Write it well, and the bridge holds. Users succeed. Your code spreads. You sleep better.

This guide teaches you to write documentation that actually helps. Not documentation that merely exists. Not walls of text that users scroll past. Documentation that answers questions before they're asked, that guides users to success, that makes complex things feel simple.

```
╔═══════════════════════════════════════════════════════════════════╗
║                    THE DOCUMENTATION TRUTH                        ║
╠═══════════════════════════════════════════════════════════════════╣
║                                                                   ║
║   Code without documentation:                                    ║
║   └── Only the author can use it (and often, not even them)     ║
║                                                                   ║
║   Code with bad documentation:                                   ║
║   └── Users read it, get confused, give up                      ║
║                                                                   ║
║   Code with good documentation:                                  ║
║   └── Users find answers, succeed quickly, recommend it         ║
║                                                                   ║
║   The difference between a library that's "technically good"     ║
║   and a library people actually use is often documentation.     ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
```

---

## The Four Kinds of Documentation

Not all documentation serves the same purpose. Understanding what kind you're writing helps you write it well.

```
┌─────────────────────────────────────────────────────────────────┐
│                    DOCUMENTATION TYPES                          │
│                                                                 │
│  TYPE          │ PURPOSE            │ STYLE                    │
│  ──────────────┼────────────────────┼─────────────────────────  │
│  Tutorial      │ Teach by doing     │ Step-by-step, guided     │
│  How-To        │ Solve a problem    │ Goal-oriented, practical │
│  Concept       │ Explain why        │ Explanatory, thoughtful  │
│  Reference     │ Look up facts      │ Precise, complete        │
│                                                                 │
│  Different goals need different approaches.                    │
│  Don't mix them carelessly.                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Tutorials: Learning by Doing

Tutorials take newcomers by the hand. They say "do this, then this, then this" and the reader learns by following along.

**Good tutorial characteristics:**
- Linear progression from start to finish
- Working code at each step
- Celebration of small victories
- Clear prerequisites upfront

### How-To Guides: Solving Problems

How-to guides answer "how do I X?" They assume the reader knows the basics and needs to accomplish a specific task.

**Good how-to characteristics:**
- Clear goal in the title
- Minimal context, maximum action
- Complete solution (not just hints)
- Troubleshooting for common issues

### Concepts: Understanding Why

Concept documentation explains the ideas behind the code. It answers "why does it work this way?" and "what problem does this solve?"

**Good concept characteristics:**
- Background and motivation
- Design decisions explained
- Trade-offs discussed
- Connections to other concepts

### Reference: Looking Up Facts

Reference documentation is like a dictionary. Complete, precise, and useful when you know what you're looking for.

**Good reference characteristics:**
- Comprehensive coverage
- Consistent format
- Accurate to the code
- Easy to search and scan

---

## API Documentation: Rustdoc Done Right

Every public item needs documentation. Here's how to write it well.

### The Complete Function Doc

```rust
/// Parses a HEDL document from UTF-8 bytes.
///
/// This function performs complete parsing including header directives,
/// body parsing, and reference resolution. For streaming parsing of
/// large documents, use [`StreamingParser`] instead.
///
/// # Arguments
///
/// * `input` - UTF-8 encoded HEDL document bytes. Must include the
///   required headers (`%V:2.0`, `%NULL:~`, `%QUOTE:"`) and separator
///   (`---`).
///
/// # Returns
///
/// A fully parsed [`Document`] with all references resolved.
///
/// # Errors
///
/// Returns [`HedlError`] if:
/// - Input is not valid UTF-8 (`HedlErrorKind::Syntax`)
/// - Required headers are missing (`HedlErrorKind::Syntax`)
/// - Document has syntax errors (`HedlErrorKind::Syntax`)
/// - References don't resolve (`HedlErrorKind::Reference`)
/// - Resource limits exceeded (`HedlErrorKind::Security`)
///
/// # Panics
///
/// This function does not panic. All errors are returned as `Result::Err`.
///
/// # Examples
///
/// Basic parsing:
///
/// ```
/// use hedl_core::parse;
///
/// let input = br#"%V:2.0
/// %NULL:~
/// %QUOTE:"
/// ---
/// name: Alice
/// age: 30
/// "#;
///
/// let doc = parse(input)?;
/// assert_eq!(doc.root.len(), 2);
/// # Ok::<(), hedl_core::HedlError>(())
/// ```
///
/// Parsing with error handling:
///
/// ```
/// use hedl_core::{parse, HedlErrorKind};
///
/// let result = parse(b"invalid content");
///
/// match result {
///     Ok(doc) => println!("Parsed {} keys", doc.root.len()),
///     Err(e) => eprintln!("Parse failed: {} at line {}", e.message, e.line),
/// }
/// ```
///
/// # See Also
///
/// - [`parse_with_limits`]: Parsing with custom resource limits
/// - [`StreamingParser`]: For documents too large to fit in memory
pub fn parse(input: &[u8]) -> Result<Document, HedlError> {
    // Implementation
}
```

### The Minimal Function Doc

Not every function needs a novel. Simple functions need simple docs:

```rust
/// Returns the number of root keys in the document.
pub fn key_count(&self) -> usize {
    self.root.len()
}

/// Returns `true` if the document has no root keys.
pub fn is_empty(&self) -> bool {
    self.root.is_empty()
}
```

### Module Documentation

Module docs explain what the module contains and how to use it:

```rust
//! JSON conversion for HEDL documents.
//!
//! This module provides bidirectional conversion between HEDL and JSON,
//! enabling interoperability with JSON-based tools and workflows.
//!
//! # Quick Start
//!
//! Convert HEDL to JSON:
//!
//! ```
//! use hedl_core::parse;
//! use hedl_json::hedl_to_json;
//!
//! let doc = parse(br#"%V:2.0
//! %NULL:~
//! %QUOTE:"
//! ---
//! name: Alice
//! "#)?;
//!
//! let json = hedl_to_json(&doc)?;
//! println!("{}", json);
//! // {"name": "Alice"}
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Convert JSON to HEDL:
//!
//! ```
//! use hedl_json::json_to_hedl;
//!
//! let json = r#"{"name": "Alice", "age": 30}"#;
//! let doc = json_to_hedl(json)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Conversion Rules
//!
//! | JSON Type | HEDL Type |
//! |-----------|-----------|
//! | `null`    | `~` (null) |
//! | `true`/`false` | `true`/`false` |
//! | number (integer) | integer |
//! | number (float) | float |
//! | string | quoted or unquoted string |
//! | array | list or tensor |
//! | object | nested object |
//!
//! # Performance
//!
//! Conversion is efficient: O(n) where n is the number of JSON nodes.
//! For very large documents (> 100 MB), consider streaming conversion.
```

---

## Writing Tips

### Be Concise

Say what needs saying. Stop.

```rust
/// Parses HEDL from bytes.
pub fn parse(input: &[u8]) -> Result<Document, HedlError>

// Not this:
/// This function takes a byte slice as input, which should contain
/// a valid HEDL document encoded in UTF-8 format, and attempts to
/// parse that input into a Document structure that represents the
/// parsed content in a structured form...
```

### Lead with What Matters

First sentence should be the most important. Users often read only that.

```rust
/// Returns the document's version number.  // Key fact first
///
/// The version is extracted from the `%V:` directive in the header.
/// If no version directive is present, returns `(1, 0)` as the default.
pub fn version(&self) -> (u32, u32)
```

### Show, Don't Tell

Examples are worth more than descriptions:

```rust
/// Formats a reference as a string.
///
/// # Examples
///
/// ```
/// use hedl_core::Reference;
///
/// // Qualified reference
/// let r = Reference::qualified("User", "alice");
/// assert_eq!(r.to_ref_string(), "@User:alice");
///
/// // Unqualified reference
/// let r = Reference::local("alice");
/// assert_eq!(r.to_ref_string(), "@alice");
/// ```
pub fn to_ref_string(&self) -> String
```

### Document Errors Completely

Users need to know what can go wrong:

```rust
/// # Errors
///
/// Returns `HedlError` with kind:
///
/// - `Syntax`: Input is not valid UTF-8 or has syntax errors
/// - `Schema`: Type is used but not defined
/// - `Reference`: Reference target not found
/// - `Shape`: Matrix row doesn't match schema
/// - `Security`: File too large or nesting too deep
```

### Link Generously

Connect related items:

```rust
/// Parses with custom options.
///
/// For default parsing, use [`parse`]. For streaming large documents,
/// use [`StreamingParser`].
///
/// # See Also
///
/// - [`ParseOptions`]: All available parsing options
/// - [`Limits`]: Resource limit configuration
```

---

## Examples That Teach

Examples should be more than syntax demos. They should teach patterns.

### Bad Example: Just Shows Syntax

```rust
/// # Example
///
/// ```
/// let x = Foo::new();
/// x.do_thing();
/// ```
```

### Good Example: Shows a Real Use Case

```rust
/// # Example: Validating User Input
///
/// Parse user-provided HEDL and validate it before processing:
///
/// ```
/// use hedl_core::{parse, HedlErrorKind};
///
/// fn process_user_input(input: &[u8]) -> Result<String, String> {
///     match parse(input) {
///         Ok(doc) => {
///             // Successfully parsed, extract data
///             let name = doc.root.get("name")
///                 .and_then(|item| item.as_string())
///                 .unwrap_or("Anonymous");
///             Ok(format!("Hello, {}!", name))
///         }
///         Err(e) => {
///             // Failed to parse, return helpful error
///             Err(format!("Invalid input at line {}: {}", e.line, e.message))
///         }
///     }
/// }
/// ```
```

### Multiple Examples for Different Scenarios

```rust
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// let config = Config::default();
/// let result = process(&config)?;
/// ```
///
/// ## Custom Configuration
///
/// ```
/// let config = Config::builder()
///     .timeout(Duration::from_secs(30))
///     .retries(3)
///     .build();
/// let result = process(&config)?;
/// ```
///
/// ## Error Handling
///
/// ```
/// match process(&config) {
///     Ok(result) => println!("Success: {:?}", result),
///     Err(e) if e.is_timeout() => eprintln!("Timed out, retry later"),
///     Err(e) => eprintln!("Failed: {}", e),
/// }
/// ```
```

---

## User Guides: The Human Touch

API docs explain functions. User guides explain journeys.

### Structure of a Good Guide

```markdown
# Parsing Large Files

When files are too large to fit in memory, streaming parsing lets you
process them piece by piece.

## When to Use Streaming

Use streaming when:
- Files exceed available RAM
- You're processing many files concurrently
- You only need part of the document

## Quick Start

[Working example that shows the complete flow]

## Step-by-Step Explanation

1. Create the parser
2. Process events
3. Handle completion

## Common Patterns

### Counting Rows
### Extracting Specific Values
### Building Indexes

## Troubleshooting

### "Out of Memory" Errors
### "Buffer Too Small" Errors

## See Also

- [Parsing Basics](parsing-basics.md)
- [StreamingParser API Reference](api/streaming.md)
```

---

## Maintaining Documentation

Documentation rots. Code changes but docs don't. Fight this.

### Doc Tests: Documentation That Runs

Rust's doc tests ensure examples stay correct:

```rust
/// Parses a value from string.
///
/// ```
/// use hedl_core::parse_value;
///
/// let value = parse_value("42")?;
/// assert_eq!(value, Value::Int(42));
/// # Ok::<(), hedl_core::HedlError>(())
/// ```
```

When the API changes and this example breaks, the test suite fails. You're forced to update the docs.

### Review Docs in PRs

Code review should include documentation review:
- Did docstrings get updated?
- Do examples still work?
- Are new public items documented?

### Scheduled Doc Audits

Periodically review docs against actual behavior:
- Run all doc tests: `cargo test --doc`
- Build docs: `cargo doc --no-deps --open`
- Click through and verify accuracy

---

## The Documentation Checklist

Before shipping, verify:

```
┌─────────────────────────────────────────────────────────────────┐
│                    DOCUMENTATION CHECKLIST                      │
│                                                                 │
│  PUBLIC ITEMS                                                   │
│  □ Every public function has a doc comment                     │
│  □ Every public type has a doc comment                         │
│  □ Every public module has module-level docs                   │
│                                                                 │
│  CONTENT                                                        │
│  □ First sentence describes the item clearly                   │
│  □ Parameters are documented                                   │
│  □ Return values are documented                                │
│  □ Errors are documented with their causes                     │
│  □ At least one example is included                            │
│                                                                 │
│  QUALITY                                                        │
│  □ Examples compile and run                                    │
│  □ Links resolve to valid targets                              │
│  □ No outdated information                                     │
│  □ No jargon without explanation                               │
│                                                                 │
│  TESTING                                                        │
│  □ Doc tests pass: cargo test --doc                           │
│  □ Docs build without warnings: cargo doc                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## The Golden Rule

Write the documentation you wish you had when you were learning. Explain what confused you. Show the examples you searched for. Answer the questions you asked.

Your future users (and your future self) will thank you.

---

## Related Documentation

- **[Code Style Guide](code-style.md)**: How to write the code being documented
- **[API Design Guidelines](api-design.md)**: How to design APIs worth documenting
- **[Contributing Guide](../contributing.md)**: How to contribute documentation
