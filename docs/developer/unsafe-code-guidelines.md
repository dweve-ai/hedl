# Unsafe Code Guidelines for HEDL

## Policy

**Default stance**: Prefer safe Rust. Unsafe code requires extraordinary justification.

**Acceptable reasons for unsafe**:
1. **Performance**: >2x speedup on benchmarks, critical path only
2. **FFI**: Required for C interop (hedl-ffi crate)
3. **Low-level optimization**: SIMD, cache optimization with measured benefit
4. **Unavoidable**: No safe alternative exists (rare in Rust 2024+)

**Unacceptable reasons**:
- Convenience or brevity
- "I know it's safe" without proof
- Premature optimization
- Working around borrow checker without analysis

## Submission Process

### 1. Before Writing Unsafe Code

- Exhaust all safe alternatives
- Write benchmark showing performance need (if applicable)
- Get architectural approval for necessity

### 2. When Implementing

- Follow template in `.plan/security/unsafe-audit-template.md`
- Write MIRI tests that exercise all unsafe code paths
- Add comprehensive safety comments in code
- Minimize unsafe surface area (smallest possible scope)

### 3. Before Merging

- Complete audit document
- Pass all MIRI checks locally
- Pass property-based tests
- Get security-focused code review
- Update unsafe code inventory

### 4. After Merging

- Monitor for issues
- Re-audit annually
- Track if safe alternatives become available

## MIRI Testing

### Local Testing

```bash
# Quick check
./scripts/miri-check.sh

# Specific test
cargo +nightly miri test test_name

# With verbose output
MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test -- --nocapture
```

### Understanding MIRI Flags

- `-Zmiri-strict-provenance`: Catches ptr-to-int-to-ptr casts (strict aliasing)
- `-Zmiri-symbolic-alignment-check`: Validates alignment requirements
- `-Zmiri-tag-raw-pointers`: Tracks pointer provenance (stacked borrows model)
- `-Zmiri-tree-borrows`: Experimental stricter aliasing model
- `-Zmiri-disable-isolation`: Allows filesystem/env access (test only)

### MIRI Limitations

MIRI cannot detect:
- Memory leaks (use Valgrind/AddressSanitizer)
- Performance issues (use profiling tools)
- Logic errors (use property-based testing)
- Concurrency bugs not involving UB (use Loom for this)

## Safety Documentation Standards

Every unsafe block must have a comment explaining:

```rust
// Safety: [One sentence summary of why this is safe]
//
// INVARIANTS:
// 1. [What must be true for this to be safe]
// 2. [Additional invariants]
//
// JUSTIFICATION:
// [Detailed reasoning about why UB cannot occur]
unsafe {
    // implementation
}
```

Example:

```rust
// Safety: Index is bounds-checked above, preventing out-of-bounds access
//
// INVARIANTS:
// 1. `idx < slice.len()` verified by bounds check
// 2. `slice` is valid for reads (derived from &[T])
// 3. Returned reference lifetime tied to slice lifetime
//
// JUSTIFICATION:
// - get_unchecked requires valid index (guaranteed by check)
// - No mutable aliasing (slice is shared reference)
// - Alignment guaranteed by slice type
unsafe {
    slice.get_unchecked(idx)
}
```

## Common Unsafe Patterns - What to Avoid

### ❌ Unchecked Indexing Without Verification

```rust
// BAD - trusting assumption without verification
unsafe { slice.get_unchecked(idx) }
```

### ✅ Bounds-Checked Then Unchecked

```rust
// GOOD - verify, then optimize
if idx < slice.len() {
    unsafe { slice.get_unchecked(idx) }
} else {
    panic!("index out of bounds");
}
```

### ❌ Raw Pointer Arithmetic Without Bounds

```rust
// BAD - can overflow or escape allocation
unsafe { ptr.add(offset) }
```

### ✅ Validated Pointer Arithmetic

```rust
// GOOD - check arithmetic and bounds
let new_offset = offset.checked_add(delta)?;
if new_offset <= allocation_size {
    unsafe { ptr.add(new_offset) }
}
```

### ❌ Transmute Without Size/Alignment Check

```rust
// BAD - UB if sizes differ or alignment wrong
unsafe { std::mem::transmute::<A, B>(value) }
```

### ✅ Transmute With Compile-Time Validation

```rust
// GOOD - static assertions prove safety
const _: () = assert!(std::mem::size_of::<A>() == std::mem::size_of::<B>());
const _: () = assert!(std::mem::align_of::<A>() >= std::mem::align_of::<B>());
unsafe { std::mem::transmute::<A, B>(value) }
```

## Unsafe Code Inventory

| File | Lines | Justification | Last Audit | Next Audit |
|------|-------|---------------|------------|------------|
| _No unsafe code currently_ | | | | |

This table must be updated whenever unsafe code is added.

## Annual Re-Audit

All unsafe code must be re-audited annually to check:
1. Are safe alternatives now available?
2. Have underlying assumptions changed?
3. Are performance characteristics still valid?
4. Have new UB patterns been discovered?

Schedule: Every January during security review cycle.
