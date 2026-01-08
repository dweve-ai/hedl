# Tutorial 2: Adding Your First Feature

Learn how to add a simple feature to HEDL, from design to pull request.

## Overview

In this tutorial, you'll add a utility function to `hedl-core` that counts the number of nodes in a HEDL document. This teaches you:

- How to navigate the codebase
- Where to add new functionality
- How to write comprehensive tests
- How to follow HEDL coding conventions
- How to submit a pull request

**Time**: ~30 minutes

## Prerequisites

- Completed [Tutorial 1: Setup Development Environment](01-setup-dev-environment.md)
- Basic understanding of Rust syntax
- Familiarity with Git

## The Feature: Node Counter

We'll add a function that counts all nodes in a document, including nested nodes.

### Example Usage

```rust
use hedl_core::{parse, traverse::count_items};

let doc = parse(b"%VERSION: 1.0\n---\nuser:\n  name: Alice\n  profile:\n    bio: Developer")?;
let count = count_items(&doc);
// count = 2 (user object, profile object - scalars not counted)
```

## Step 1: Create a Feature Branch

```bash
cd hedl
git checkout -b add-node-counter
git branch  # Verify you're on the new branch
```

## Step 2: Understand the Code Structure

### Explore the Document Structure

```bash
# Open the core library
cat crates/hedl-core/src/lib.rs
```

Key types:
```rust
pub struct Document {
    pub version: (u32, u32),
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    pub root: BTreeMap<String, Item>,
}

pub enum Item {
    Scalar(Value),
    Object(BTreeMap<String, Item>),
    List(MatrixList),
}

pub struct MatrixList {
    pub type_name: String,
    pub schema: Vec<String>,
    pub rows: Vec<Node>,
    pub count_hint: Option<usize>,
}

pub struct Node {
    pub type_name: String,
    pub id: String,
    pub fields: Vec<Value>,
    pub children: BTreeMap<String, Vec<Node>>,
    pub child_count: Option<usize>,
}
```

### Find the Right Module

The `traverse` module handles tree walking:

```bash
cat crates/hedl-core/src/traverse.rs
```

Note: This tutorial demonstrates how you would add a new feature. The actual `count_nodes` function doesn't exist yet - you'll be creating it as practice.

## Step 3: Implement the Function

### Open the File

```bash
code crates/hedl-core/src/traverse.rs
# or
vim crates/hedl-core/src/traverse.rs
```

### Add the Function

Add this code after the existing `traverse` function:

```rust
/// Counts the total number of items (objects and lists) in a document.
///
/// This recursively counts all nested objects and matrix lists in the document.
/// Scalar values are not counted.
///
/// # Arguments
///
/// * `doc` - The document to count items in
///
/// # Returns
///
/// The total number of objects and lists in the document tree.
///
/// # Examples
///
/// ```
/// use hedl_core::{parse, traverse::count_items};
///
/// let doc = parse(b"user:\n  name: Alice\n  profile:\n    bio: Dev").unwrap();
/// let count = count_items(&doc);
/// assert_eq!(count, 2); // user object, profile object
/// ```
pub fn count_items(doc: &Document) -> usize {
    let mut count = 0;
    for (_key, item) in &doc.root {
        count += count_items_recursive(item);
    }
    count
}

/// Recursive helper to count items
fn count_items_recursive(item: &Item) -> usize {
    match item {
        Item::Scalar(_) => 0,
        Item::Object(map) => {
            let mut count = 1; // Count this object
            for (_key, child_item) in map {
                count += count_items_recursive(child_item);
            }
            count
        }
        Item::List(matrix) => {
            1 + matrix.rows.len() // Count the list plus all rows as nodes
        }
    }
}
```

### Understanding the Code

1. **Documentation**:
   - `///` for public docs (appears in `cargo doc`)
   - Examples in docstrings are tested by `cargo test`

2. **Recursion**:
   - Count objects and lists, not scalars
   - Recursively traverse nested objects
   - Matrix list rows are counted as nodes

3. **Pattern Matching**:
   - `match` on `Item` enum variants
   - Handle `Scalar`, `Object`, and `List` cases differently

## Step 4: Export the Function

Add to `crates/hedl-core/src/lib.rs`:

```rust
// Find the existing re-exports section
pub use traverse::{traverse, DocumentVisitor, StatsCollector, VisitorContext};

// Add count_items
pub use traverse::{count_items, traverse, DocumentVisitor, StatsCollector, VisitorContext};
```

## Step 5: Write Tests

### Add Unit Tests

At the end of `crates/hedl-core/src/traverse.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_count_items_empty() {
        let doc = parse(b"%VERSION: 1.0\n---\n").unwrap();
        assert_eq!(count_items(&doc), 0); // No items
    }

    #[test]
    fn test_count_items_scalar_only() {
        let doc = parse(b"%VERSION: 1.0\n---\nname: Alice\nage: 30").unwrap();
        assert_eq!(count_items(&doc), 0); // Scalars don't count
    }

    #[test]
    fn test_count_items_nested() {
        let hedl = b"%VERSION: 1.0\n---\nuser:\n  name: Alice\n  profile:\n    bio: Developer";
        let doc = parse(hedl).unwrap();
        // user object + profile object = 2
        assert_eq!(count_items(&doc), 2);
    }

    #[test]
    fn test_count_items_with_matrix() {
        let hedl = b"%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n| u1, Alice\n| u2, Bob";
        let doc = parse(hedl).unwrap();
        // users list + 2 rows = 3
        assert_eq!(count_items(&doc), 3);
    }

    #[test]
    fn test_count_items_deep_nesting() {
        let hedl = b"%VERSION: 1.0\n---\na:\n  b:\n    c:\n      d:\n        e: value";
        let doc = parse(hedl).unwrap();
        // a + b + c + d objects = 4 (e is scalar)
        assert_eq!(count_items(&doc), 4);
    }

    #[test]
    fn test_count_items_mixed() {
        let hedl = b"%VERSION: 1.0\n---\nparent:\n child1:\n  nested: value\n child2:\n  nested: value\n scalar: data";
        let doc = parse(hedl).unwrap();
        // parent + child1 + child2 = 3 (scalars don't count)
        assert_eq!(count_items(&doc), 3);
    }
}
```

### Test Categories

1. **Edge Cases**: Empty document
2. **Simple Cases**: Flat structure
3. **Nested Cases**: Tree structures
4. **Mixed Cases**: Nodes + matrix lists
5. **Deep Nesting**: Stress test recursion
6. **Multiple Children**: Branching trees

## Step 6: Run the Tests

```bash
# Run just the new tests
cargo test -p hedl-core count_items

# Run all traverse module tests
cargo test -p hedl-core traverse::tests

# Run all hedl-core tests
cargo test -p hedl-core
```

Expected output:
```
running 6 tests
test traverse::tests::test_count_items_empty ... ok
test traverse::tests::test_count_items_scalar_only ... ok
test traverse::tests::test_count_items_nested ... ok
test traverse::tests::test_count_items_with_matrix ... ok
test traverse::tests::test_count_items_deep_nesting ... ok
test traverse::tests::test_count_items_mixed ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured
```

## Step 7: Test the Docstring Example

```bash
# Docstring examples are tested with doc tests
cargo test --doc -p hedl-core count_items
```

This runs the example in the `///` documentation.

## Step 8: Check Code Quality

### Format the Code

```bash
cargo fmt --all
```

### Run Clippy (Linter)

```bash
cargo clippy -p hedl-core -- -D warnings
```

Fix any warnings that appear.

### Build Documentation

```bash
cargo doc -p hedl-core --open
```

Verify your function appears in the documentation with proper formatting.

## Step 9: Add an Integration Test

Create `crates/hedl-core/tests/integration/node_counting.rs`:

```rust
use hedl_core::{parse, traverse::count_items};

#[test]
fn test_count_items_real_world_example() {
    let hedl = r#"%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com

admin:
  name: Admin User
  permissions:
    read: true
    write: true
    delete: false
"#;

    let doc = parse(hedl.as_bytes()).unwrap();

    // users (list + 2 rows) + admin + permissions = 5
    assert_eq!(count_items(&doc), 5);
}
```

Run it:
```bash
cargo test -p hedl-core --test integration
```

## Step 10: Commit Your Changes

```bash
# Stage the changes
git add crates/hedl-core/src/traverse.rs
git add crates/hedl-core/src/lib.rs
git add crates/hedl-core/tests/integration/node_counting.rs

# Commit with descriptive message
git commit -m "feat(core): Add count_items function for document analysis

- Add count_items() to traverse module
- Recursively counts all Object and List items in document tree
- Excludes scalar values from count
- Add comprehensive unit tests covering edge cases
- Add integration test with real-world example
- Add documentation with usage examples"
```

### Commit Message Format

HEDL uses [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `test`: Adding tests
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `chore`: Maintenance tasks

## Step 11: Push and Create Pull Request

```bash
# Push to your fork
git push origin add-node-counter
```

Then on GitHub:

1. Go to https://github.com/dweve-ai/hedl
2. Click "New Pull Request"
3. Select your branch
4. Fill in the template:

```markdown
## Description

Adds a `count_items()` function to the traverse module for counting all objects and lists in a document tree.

## Motivation

Useful for:
- Document statistics
- Memory estimation
- Progress tracking during parsing
- Test assertions

## Changes

- Add `count_items()` and helper `count_items_recursive()`
- Export from `hedl-core` public API
- Add 6 unit tests covering edge cases
- Add integration test with real-world example
- Add documentation with examples

## Testing

- [x] Unit tests pass
- [x] Integration tests pass
- [x] Docstring examples tested
- [x] Clippy passes with no warnings
- [x] Code formatted with rustfmt

## Checklist

- [x] Code follows project style guidelines
- [x] Documentation added for new functionality
- [x] Tests added for new functionality
- [x] All tests pass locally
- [x] No clippy warnings
```

## Step 12: Respond to Review Feedback

Reviewers might ask for changes:

```bash
# Make requested changes
vim crates/hedl-core/src/traverse.rs

# Test again
cargo test -p hedl-core

# Commit the changes
git add -u
git commit -m "refactor: Address review feedback

- Improve documentation clarity
- Add example for deeply nested structures"

# Push the update
git push origin add-node-counter
```

## Code Style Guidelines

### Naming Conventions

```rust
// Functions: snake_case
pub fn count_items(doc: &Document) -> usize

// Types: PascalCase
pub struct DocumentVisitor

// Constants: SCREAMING_SNAKE_CASE
const MAX_DEPTH: usize = 100;

// Modules: snake_case
mod traverse;
```

### Documentation Style

```rust
/// Brief one-line description.
///
/// Longer description explaining behavior, edge cases,
/// and important details.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function returns an error (if applicable)
///
/// # Examples
///
/// ```
/// use hedl_core::traverse::count_items;
/// // Example code here
/// ```
pub fn count_items(doc: &Document) -> usize {
    // Implementation
}
```

### Error Handling

```rust
// Use Result for fallible operations
pub fn parse(input: &[u8]) -> Result<Document, HedlError> {
    // ...
}

// Use Option for nullable values
pub fn get_attribute(&self, key: &str) -> Option<&Value> {
    self.attributes.get(key)
}

// Propagate errors with ?
let doc = parse(input)?;
```

## Common Mistakes to Avoid

### 1. Forgetting to Export

```rust
// ❌ Function exists but not exported
// src/traverse.rs has count_items but lib.rs doesn't export it

// ✅ Export from lib.rs
pub use traverse::count_items;
```

### 2. Missing Documentation

```rust
// ❌ No documentation
pub fn count_items(doc: &Document) -> usize {

// ✅ Documented
/// Counts items in document.
pub fn count_items(doc: &Document) -> usize {
```

### 3. Insufficient Testing

```rust
// ❌ Only happy path
#[test]
fn test_count_items() {
    assert_eq!(count_items(&doc), 5);
}

// ✅ Edge cases covered
#[test]
fn test_count_items_empty() { }

#[test]
fn test_count_items_nested() { }

#[test]
fn test_count_items_with_matrix() { }
```

## Next Steps

Congratulations! You've added your first feature. Next:

1. **More Complex Features**: Try [Adding Format Support](03-adding-format-support.md)
2. **Explore Existing Code**: Read through `hedl-core/src/parser.rs`
3. **Pick an Issue**: Find "good first issue" tags on GitHub
4. **Improve Documentation**: Add examples to existing functions

## Additional Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [HEDL Contributing Guide](../contributing.md)
- [HEDL Code Review Checklist](../guides/code-style.md)

---

**Great job!** You've successfully contributed to HEDL.
