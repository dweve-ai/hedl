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

We'll add a function that counts all items (objects and lists) in a document using the visitor pattern.

### Example Usage

```rust
use hedl_core::{parse, visitor::NodeCollector};

let doc = parse(b"%V:2.0\n---\nuser:\n  name: Alice\n  profile:\n    bio: Developer")?;
let collector = NodeCollector::new();
let count = collector.collect(&doc).len();
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
    pub schema_versions: BTreeMap<String, SchemaVersion>,
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
    pub fields: SmallVec<[Value; 4]>,  // Stack-allocated for ≤4 fields
    pub children: Option<Box<BTreeMap<String, Vec<Node>>>>,  // Lazy allocation
    pub child_count: u16,  // Compact hint
}
```

### Find the Right Module

The `visitor` module in `hedl-core` provides traversal patterns:

```bash
ls crates/hedl-core/src/visitor/
```

The existing `NodeCollector` visitor collects nodes during traversal. For this tutorial, we'll demonstrate extending the `hedl-test` crate with a new counting utility that leverages the existing visitor infrastructure.

## Step 3: Implement the Function

### Open the File

```bash
code crates/hedl-test/src/counts.rs
# or
vim crates/hedl-test/src/counts.rs
```

### Add the Function

Add this code to extend the counts module:

```rust
use hedl_core::{Document, Item};

/// Counts the total number of items (objects and lists) in a document.
///
/// This counts all nested objects and matrix lists in the document.
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
/// use hedl_core::parse;
/// use hedl_test::count_items;
///
/// let doc = parse(b"%V:2.0\n---\nuser:\n  name: Alice\n  profile:\n    bio: Dev").unwrap();
/// let count = count_items(&doc);
/// assert_eq!(count, 2); // user object, profile object
/// ```
pub fn count_items(doc: &Document) -> usize {
    count_items_recursive(&doc.root)
}

fn count_items_recursive(items: &std::collections::BTreeMap<String, Item>) -> usize {
    let mut count = 0;
    for item in items.values() {
        match item {
            Item::Object(nested) => {
                count += 1; // Count this object
                count += count_items_recursive(nested); // Count nested items
            }
            Item::List(_) => {
                count += 1; // Count the list
            }
            Item::Scalar(_) => {
                // Don't count scalars
            }
        }
    }
    count
}
```

### Understanding the Code

1. **Documentation**:
   - `///` for public docs (appears in `cargo doc`)
   - Examples in docstrings are tested by `cargo test`

2. **Using Visitors**:
   - `NodeCollector` is already built in to traverse documents
   - Visitor pattern separates traversal from processing
   - Filter the results to count only what we want

3. **Testing**:
   - Doc comments with examples are automatically tested

## Step 4: Export the Function

Add to `crates/hedl-test/src/lib.rs`:

```rust
// In the public module section
pub use counts::count_items;
```

This allows users to import as:
```rust
use hedl_test::count_items;
```

## Step 5: Write Tests

### Add Unit Tests

Create a new test file `crates/hedl-test/tests/count_items_tests.rs`:

```rust
use hedl_core::parse;
use hedl_test::count_items;

#[test]
fn test_count_items_empty() {
    let doc = parse(b"%V:2.0\n---\n").unwrap();
    assert_eq!(count_items(&doc), 0); // No items
}

#[test]
fn test_count_items_scalar_only() {
    let doc = parse(b"%V:2.0\n---\nname: Alice\nage: 30").unwrap();
    assert_eq!(count_items(&doc), 0); // Scalars don't count
}

#[test]
fn test_count_items_nested() {
    let hedl = b"%V:2.0\n---\nuser:\n  name: Alice\n  profile:\n    bio: Developer";
    let doc = parse(hedl).unwrap();
    // user object + profile object = 2
    assert_eq!(count_items(&doc), 2);
}

#[test]
fn test_count_items_deep_nesting() {
    let hedl = b"%V:2.0\n---\na:\n  b:\n    c:\n      d:\n        e: value";
    let doc = parse(hedl).unwrap();
    // a + b + c + d objects = 4 (e is scalar)
    assert_eq!(count_items(&doc), 4);
}

#[test]
fn test_count_items_mixed() {
    let hedl = b"%V:2.0\n---\nparent:\n  child1:\n    nested: value\n  child2:\n    nested: value\nscalar: data";
    let doc = parse(hedl).unwrap();
    // parent + child1 + child2 = 3 (scalars don't count)
    assert_eq!(count_items(&doc), 3);
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
cargo test -p hedl-test count_items

# Run all hedl-test tests
cargo test -p hedl-test

# Run with verbose output
cargo test -p hedl-test count_items -- --nocapture
```

Expected output:
```
running 5 tests
test test_count_items_empty ... ok
test test_count_items_scalar_only ... ok
test test_count_items_nested ... ok
test test_count_items_deep_nesting ... ok
test test_count_items_mixed ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

## Step 7: Test the Docstring Example

```bash
# Docstring examples are tested with doc tests
cargo test --doc -p hedl-test count_items
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
cargo doc -p hedl-test --open
```

Verify your function appears in the documentation with proper formatting.

## Step 9: Add a Real-World Integration Test

Update `crates/hedl-test/tests/count_items_tests.rs` to add:

```rust
#[test]
fn test_count_items_real_world_example() {
    let hedl = r#"
root:
  users:
    alice:
      name: Alice Smith
      email: alice@example.com
    bob:
      name: Bob Jones
      email: bob@example.com
  admin:
    name: Admin User
    permissions:
      read: true
      write: true
      delete: false
"#;

    let doc = parse(hedl.as_bytes()).unwrap();

    // root + users + alice + bob + admin + permissions = 6 objects
    assert_eq!(count_items(&doc), 6);
}
```

Run it:
```bash
cargo test -p hedl-test --test count_items_tests
```

## Step 10: Commit Your Changes

```bash
# Stage the changes
git add crates/hedl-test/src/counts.rs
git add crates/hedl-test/src/lib.rs
git add crates/hedl-test/tests/count_items_tests.rs

# Commit with descriptive message
git commit -m "feat(test): Add count_items utility function

- Add count_items() to hedl-test crate for document analysis
- Counts all Object and List items in document tree
- Excludes scalar values from count
- Uses visitor pattern for clean separation of concerns
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
pub struct NodeCollector

// Constants: SCREAMING_SNAKE_CASE
const MAX_DEPTH: usize = 100;

// Modules: snake_case
mod visitor;
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
2. **Explore Existing Code**: Read through `hedl-core/src/parser/`
3. **Pick an Issue**: Find "good first issue" tags on GitHub
4. **Improve Documentation**: Add examples to existing functions

## Additional Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [HEDL Contributing Guide](../contributing.md)
- [HEDL Code Review Checklist](../guides/code-style.md)

---

**Great job!** You've successfully contributed to HEDL.
