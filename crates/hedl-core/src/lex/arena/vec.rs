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

// Arena allocator requires unsafe for performance-critical pointer operations.
// Safety invariants are documented at each unsafe block.
#![allow(unsafe_code)]

//! Arena-backed vector for small, fixed-size collections.
//!
//! ArenaVec stores its contents directly in an arena, avoiding heap
//! allocation for small collections like node fields (typically 3-7 values).
//!
//! Unlike Vec<T> where the struct is arena-allocated but the buffer is
//! heap-allocated, ArenaVec stores the actual data in the arena.

use bumpalo::Bump;
use std::marker::PhantomData;
use std::ops::{Deref, Index};
use std::ptr::NonNull;
use std::slice;

/// Arena-backed vector (read-only after creation).
///
/// Stores its contents directly in an arena. This is efficient for small,
/// fixed-size collections that don't need to grow after creation.
///
/// # Performance Characteristics
///
/// - **Creation**: O(n) to copy n elements into arena
/// - **Access**: O(1) index access
/// - **Memory**: No heap allocation (data in arena)
/// - **Cache**: Better locality than heap-scattered Vec
///
/// # Limitations
///
/// - **Read-only**: Cannot push/pop after creation (use Vec for that)
/// - **Fixed-size**: Cannot grow (arena allocation is bump-only)
/// - **Lifetime-bounded**: Tied to arena lifetime
///
/// # Examples
///
/// ```ignore
/// use bumpalo::Bump;
/// use hedl_core::lex::arena::ArenaVec;
///
/// let arena = Bump::new();
/// let vec = ArenaVec::from_slice(&arena, &[1, 2, 3, 4, 5]);
///
/// assert_eq!(vec.len(), 5);
/// assert_eq!(vec[0], 1);
/// assert_eq!(vec[4], 5);
///
/// // Iterate
/// for (i, &val) in vec.iter().enumerate() {
///     assert_eq!(val, i as i32 + 1);
/// }
/// ```
pub struct ArenaVec<'arena, T> {
    /// Pointer to the first element
    ptr: NonNull<T>,
    /// Number of elements
    len: usize,
    /// Phantom data to track lifetime and ownership
    _marker: PhantomData<&'arena [T]>,
}

impl<'arena, T> ArenaVec<'arena, T> {
    /// Create an empty ArenaVec.
    ///
    /// This doesn't allocate anything in the arena.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use hedl_core::lex::arena::ArenaVec;
    ///
    /// let vec: ArenaVec<i32> = ArenaVec::empty();
    /// assert_eq!(vec.len(), 0);
    /// assert!(vec.is_empty());
    /// ```
    pub fn empty() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Create an ArenaVec from a slice.
    ///
    /// Copies the slice elements into the arena.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bumpalo::Bump;
    /// use hedl_core::lex::arena::ArenaVec;
    ///
    /// let arena = Bump::new();
    /// let data = [1, 2, 3, 4, 5];
    /// let vec = ArenaVec::from_slice(&arena, &data);
    ///
    /// assert_eq!(vec.len(), 5);
    /// assert_eq!(&vec[..], &data);
    /// ```
    pub fn from_slice(arena: &'arena Bump, slice: &[T]) -> Self
    where
        T: Copy,
    {
        if slice.is_empty() {
            return Self::empty();
        }

        // Allocate storage in arena and copy elements
        let arena_storage = arena.alloc_slice_copy(slice);

        Self {
            ptr: NonNull::new(arena_storage.as_mut_ptr()).expect("arena allocation is non-null"),
            len: slice.len(),
            _marker: PhantomData,
        }
    }

    /// Create an ArenaVec from an iterator.
    ///
    /// Collects the iterator into a temporary Vec, then copies to arena.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bumpalo::Bump;
    /// use hedl_core::lex::arena::ArenaVec;
    ///
    /// let arena = Bump::new();
    /// let vec = ArenaVec::from_iter(&arena, [1, 2, 3, 4, 5].iter().copied());
    ///
    /// assert_eq!(vec.len(), 5);
    /// ```
    pub fn from_iter(arena: &'arena Bump, iter: impl IntoIterator<Item = T>) -> Self
    where
        T: Copy,
    {
        let items: Vec<T> = iter.into_iter().collect();
        Self::from_slice(arena, &items)
    }

    /// Get the number of elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get a slice of all elements.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bumpalo::Bump;
    /// use hedl_core::lex::arena::ArenaVec;
    ///
    /// let arena = Bump::new();
    /// let vec = ArenaVec::from_slice(&arena, &[1, 2, 3]);
    ///
    /// let slice: &[i32] = vec.as_slice();
    /// assert_eq!(slice, &[1, 2, 3]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            &[]
        } else {
            // SAFETY: ptr and len were created from valid slice
            unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
        }
    }

    /// Get an iterator over the elements.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bumpalo::Bump;
    /// use hedl_core::lex::arena::ArenaVec;
    ///
    /// let arena = Bump::new();
    /// let vec = ArenaVec::from_slice(&arena, &[1, 2, 3]);
    ///
    /// let sum: i32 = vec.iter().sum();
    /// assert_eq!(sum, 6);
    /// ```
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Get a reference to an element by index.
    ///
    /// Returns None if index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bumpalo::Bump;
    /// use hedl_core::lex::arena::ArenaVec;
    ///
    /// let arena = Bump::new();
    /// let vec = ArenaVec::from_slice(&arena, &[10, 20, 30]);
    ///
    /// assert_eq!(vec.get(0), Some(&10));
    /// assert_eq!(vec.get(2), Some(&30));
    /// assert_eq!(vec.get(3), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    /// Get the first element.
    ///
    /// Returns None if the vector is empty.
    pub fn first(&self) -> Option<&T> {
        self.as_slice().first()
    }

    /// Get the last element.
    ///
    /// Returns None if the vector is empty.
    pub fn last(&self) -> Option<&T> {
        self.as_slice().last()
    }

    /// Convert to an owned Vec by cloning all elements.
    ///
    /// This is used when converting arena-backed temporary data to
    /// heap-allocated final document.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bumpalo::Bump;
    /// use hedl_core::lex::arena::ArenaVec;
    ///
    /// let arena = Bump::new();
    /// let vec = ArenaVec::from_slice(&arena, &[1, 2, 3]);
    ///
    /// let owned: Vec<i32> = vec.to_vec();
    /// assert_eq!(owned, vec![1, 2, 3]);
    /// ```
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.as_slice().to_vec()
    }
}

// Deref to slice for convenient access
impl<'arena, T> Deref for ArenaVec<'arena, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

// Index access
impl<'arena, T> Index<usize> for ArenaVec<'arena, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

// Clone (shallow - just copies via Copy since this type is Copy)
impl<'arena, T> Clone for ArenaVec<'arena, T> {
    fn clone(&self) -> Self {
        *self
    }
}

// Copy (for cheap copying of the vec reference)
impl<'arena, T> Copy for ArenaVec<'arena, T> {}

// Debug
impl<'arena, T: std::fmt::Debug> std::fmt::Debug for ArenaVec<'arena, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(f)
    }
}

// PartialEq
impl<'arena, T: PartialEq> PartialEq for ArenaVec<'arena, T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<'arena, T: Eq> Eq for ArenaVec<'arena, T> {}

// PartialEq with slice
impl<'arena, T: PartialEq> PartialEq<[T]> for ArenaVec<'arena, T> {
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<'arena, T: PartialEq> PartialEq<&[T]> for ArenaVec<'arena, T> {
    fn eq(&self, other: &&[T]) -> bool {
        self.as_slice() == *other
    }
}

impl<'arena, T: PartialEq> PartialEq<Vec<T>> for ArenaVec<'arena, T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

// IntoIterator
impl<'arena, T> IntoIterator for ArenaVec<'arena, T> {
    type Item = &'arena T;
    type IntoIter = slice::Iter<'arena, T>;

    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: We're creating an iterator with the same lifetime as the arena
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len).iter() }
    }
}

impl<'a, 'arena, T> IntoIterator for &'a ArenaVec<'arena, T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let vec: ArenaVec<'_, i32> = ArenaVec::empty();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        assert_eq!(vec.as_slice(), &[]);
    }

    #[test]
    fn test_from_slice() {
        let arena = Bump::new();
        let data = [1, 2, 3, 4, 5];
        let vec = ArenaVec::from_slice(&arena, &data);

        assert_eq!(vec.len(), 5);
        assert!(!vec.is_empty());
        assert_eq!(vec.as_slice(), &data);
    }

    #[test]
    fn test_from_empty_slice() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[] as &[i32]);

        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_from_iter() {
        let arena = Bump::new();
        let vec = ArenaVec::from_iter(&arena, [1, 2, 3, 4, 5].iter().copied());

        assert_eq!(vec.len(), 5);
        assert_eq!(vec.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_indexing() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[10, 20, 30, 40, 50]);

        assert_eq!(vec[0], 10);
        assert_eq!(vec[2], 30);
        assert_eq!(vec[4], 50);
    }

    #[test]
    fn test_get() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[10, 20, 30]);

        assert_eq!(vec.get(0), Some(&10));
        assert_eq!(vec.get(2), Some(&30));
        assert_eq!(vec.get(3), None);
    }

    #[test]
    fn test_first_last() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[10, 20, 30]);

        assert_eq!(vec.first(), Some(&10));
        assert_eq!(vec.last(), Some(&30));

        let empty: ArenaVec<'_, i32> = ArenaVec::empty();
        assert_eq!(empty.first(), None);
        assert_eq!(empty.last(), None);
    }

    #[test]
    fn test_iter() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[1, 2, 3, 4, 5]);

        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 15);

        let doubled: Vec<i32> = vec.iter().map(|x| x * 2).collect();
        assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_into_iter() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[1, 2, 3]);

        let sum: i32 = vec.into_iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_to_vec() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[1, 2, 3]);

        let owned = vec.to_vec();
        assert_eq!(owned, vec![1, 2, 3]);
    }

    #[test]
    fn test_clone_copy() {
        let arena = Bump::new();
        let vec1 = ArenaVec::from_slice(&arena, &[1, 2, 3]);

        let vec2 = vec1;
        let vec3 = vec1; // Copy

        assert_eq!(vec1, vec2);
        assert_eq!(vec1, vec3);
        assert_eq!(vec2, vec3);
    }

    #[test]
    fn test_equality() {
        let arena = Bump::new();
        let vec1 = ArenaVec::from_slice(&arena, &[1, 2, 3]);
        let vec2 = ArenaVec::from_slice(&arena, &[1, 2, 3]);
        let vec3 = ArenaVec::from_slice(&arena, &[1, 2, 4]);

        assert_eq!(vec1, vec2);
        assert_ne!(vec1, vec3);

        // Compare with slice
        assert_eq!(vec1, &[1, 2, 3][..]);
        assert_eq!(vec1.as_slice(), &[1, 2, 3]);

        // Compare with Vec
        assert_eq!(vec1, vec![1, 2, 3]);
        assert_ne!(vec1, vec![1, 2, 4]);
    }

    #[test]
    fn test_debug() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[1, 2, 3]);

        let debug = format!("{:?}", vec);
        assert_eq!(debug, "[1, 2, 3]");
    }

    #[test]
    fn test_deref() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[1, 2, 3, 4, 5]);

        // Can use slice methods via Deref
        assert!(vec.contains(&3));
        assert!(!vec.contains(&10));
        assert_eq!(vec.binary_search(&3), Ok(2));
    }

    #[test]
    fn test_strings() {
        let arena = Bump::new();
        let strings = vec!["hello", "world", "test"];
        let vec = ArenaVec::from_slice(&arena, &strings);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], "hello");
        assert_eq!(vec[1], "world");
        assert_eq!(vec[2], "test");
    }

    #[test]
    fn test_large_vec() {
        let arena = Bump::new();
        let data: Vec<i32> = (0..1000).collect();
        let vec = ArenaVec::from_slice(&arena, &data);

        assert_eq!(vec.len(), 1000);
        assert_eq!(vec[0], 0);
        assert_eq!(vec[999], 999);

        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 999 * 1000 / 2); // Sum of 0..1000
    }

    #[test]
    fn test_multiple_vecs_same_arena() {
        let arena = Bump::new();

        let vec1 = ArenaVec::from_slice(&arena, &[1, 2, 3]);
        let vec2 = ArenaVec::from_slice(&arena, &[4, 5, 6]);
        let vec3 = ArenaVec::from_slice(&arena, &[7, 8, 9]);

        assert_eq!(vec1.as_slice(), &[1, 2, 3]);
        assert_eq!(vec2.as_slice(), &[4, 5, 6]);
        assert_eq!(vec3.as_slice(), &[7, 8, 9]);
    }

    #[test]
    fn test_slice_operations() {
        let arena = Bump::new();
        let vec = ArenaVec::from_slice(&arena, &[1, 2, 3, 4, 5]);

        // Can use slice syntax via Deref
        let slice = vec.as_slice();
        assert_eq!(&slice[1..4], &[2, 3, 4]);
        assert_eq!(&slice[..3], &[1, 2, 3]);
        assert_eq!(&slice[2..], &[3, 4, 5]);
    }
}
