// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Arena allocation for expression parsing.
//!
//! **WARNING: Not recommended for production use**
//!
//! Benchmarks show arena allocation is 70-300% SLOWER than standard heap
//! allocation for HEDL expression parsing because:
//! - Vec/String still allocate their buffers on the heap (not in arena)
//! - Arena only stores the struct metadata (24 bytes for Vec)
//! - Modern allocators (jemalloc/mimalloc) are highly optimized
//! - Indirection overhead outweighs allocation savings
//!
//! This module exists for experimentation and to demonstrate why arena
//! allocation doesn't help for this use case.

use bumpalo::Bump;

/// Arena allocator for expression parsing.
pub struct ExpressionArena {
    bump: Bump,
}

impl ExpressionArena {
    /// Create a new expression arena.
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
        }
    }

    /// Create a new arena with a specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
        }
    }

    /// Allocate a value in the arena and return a reference to it.
    ///
    /// **Note**: If T contains heap-allocated data (Vec, String, etc.),
    /// only the struct itself is arena-allocated. The buffer is still
    /// allocated on the heap.
    pub fn alloc<T>(&mut self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    /// Reset the arena, freeing all allocations.
    ///
    /// This is useful for reusing the same arena across multiple parse operations.
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Get the number of bytes currently allocated in the arena.
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Default for ExpressionArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_allocation() {
        let mut arena = ExpressionArena::new();
        let x = arena.alloc(42);
        assert_eq!(*x, 42);

        // Separate scope to avoid double mut borrow
        let y = arena.alloc(100);
        assert_eq!(*y, 100);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = ExpressionArena::new();

        // Allocate some values
        for i in 0..100 {
            let _ = arena.alloc(i);
        }

        let before = arena.allocated_bytes();
        assert!(before > 0);

        arena.reset();

        // After reset, allocated_bytes may not be 0 because bumpalo
        // keeps capacity for reuse. Just verify it doesn't grow.
        let after = arena.allocated_bytes();
        assert!(after <= before);
    }

    #[test]
    fn test_vec_allocation_only_stores_struct() {
        let mut arena = ExpressionArena::new();

        // Allocate a large Vec - only the Vec struct (24 bytes) is in arena
        let large_vec: Vec<u8> = vec![0; 1024 * 1024]; // 1 MB of heap data
        let _ = arena.alloc(large_vec);

        // Arena size should be tiny (just the Vec struct), not 1 MB
        assert!(arena.allocated_bytes() < 1024);
    }

    #[test]
    fn test_with_capacity() {
        // Arena with capacity pre-allocates, so allocated_bytes may be non-zero
        let arena = ExpressionArena::with_capacity(4096);
        // Don't assert exact value, just that we can create it
        let _ = arena.allocated_bytes();

        // Can allocate within capacity
        let mut arena = ExpressionArena::with_capacity(4096);
        for i in 0..100 {
            let _ = arena.alloc(i);
        }
        assert!(arena.allocated_bytes() > 0);
    }
}
