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

//! Reentrancy detection for FFI callback functions.
//!
//! This module provides runtime detection of callback reentrancy, which occurs
//! when an FFI function is called from within a callback function. This is
//! unsafe and can cause:
//!
//! - `RefCell` panics due to overlapping borrows of thread-local storage
//! - Use-after-free bugs if callback invalidates data
//! - Deadlock in nested synchronization primitives
//!
//! # Design
//!
//! Uses thread-local storage to track when we're inside a callback execution.
//! Any FFI function call while the callback flag is set is rejected with an
//! error code instead of panicking.

use std::cell::Cell;
use std::os::raw::c_int;

// =============================================================================
// Reentrancy Detection
// =============================================================================

thread_local! {
    /// Flag indicating whether we're currently executing a callback.
    ///
    /// This is per-thread to ensure thread-safe callback execution.
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
}

/// Error code for reentrant FFI calls.
pub const HEDL_ERR_REENTRANT_CALL: c_int = -14;

/// Enter a callback context.
///
/// This function must be called at the start of any callback function before
/// invoking user code. Returns an error if we're already in a callback (nested
/// callbacks are not supported).
///
/// # Returns
///
/// * `Ok(())` - Successfully entered callback context
/// * `Err(HEDL_ERR_REENTRANT_CALL)` - Already in a callback (reentrancy detected)
///
/// # Safety
///
/// This function is safe to call from any thread. Each thread maintains its
/// own callback state.
pub fn enter_callback() -> Result<(), c_int> {
    IN_CALLBACK.with(|flag| {
        if flag.get() {
            // Already in a callback - reentrancy detected
            Err(HEDL_ERR_REENTRANT_CALL)
        } else {
            flag.set(true);
            Ok(())
        }
    })
}

/// Exit a callback context.
///
/// This function must be called after callback execution completes, even if
/// the callback panicked or returned an error.
///
/// # Safety
///
/// This function must only be called after `enter_callback()` returned `Ok(())`.
/// Calling it without having entered a callback is a bug but will not cause
/// undefined behavior.
pub fn exit_callback() {
    IN_CALLBACK.with(|flag| {
        flag.set(false);
    });
}

/// Check if we're currently in a callback.
///
/// This function does not modify the callback state and can be used to query
/// whether reentrancy detection is active.
///
/// # Returns
///
/// `true` if currently in a callback, `false` otherwise.
#[must_use]
pub fn in_callback() -> bool {
    IN_CALLBACK.with(std::cell::Cell::get)
}

// =============================================================================
// Reentrancy Guard
// =============================================================================

/// RAII guard for automatic callback context management.
///
/// This guard automatically exits the callback context when dropped, ensuring
/// cleanup even if the callback panics.
///
/// # Examples
///
/// ```rust,ignore
/// use hedl_ffi::reentrancy::ReentrancyGuard;
/// use std::ffi::c_void;
///
/// unsafe extern "C" fn my_callback(data: *const i8, len: usize, user_data: *mut c_void) {
///     let _guard = match ReentrancyGuard::enter() {
///         Ok(guard) => guard,
///         Err(_) => return, // Reentrancy detected
///     };
///
///     // ... callback implementation ...
/// }
/// ```
#[derive(Debug)]
pub struct ReentrancyGuard;

impl ReentrancyGuard {
    /// Enter a callback context and create a guard.
    ///
    /// # Returns
    ///
    /// * `Ok(ReentrancyGuard)` - Successfully entered callback context
    /// * `Err(HEDL_ERR_REENTRANT_CALL)` - Already in a callback
    pub fn enter() -> Result<Self, c_int> {
        enter_callback()?;
        Ok(Self)
    }
}

impl Drop for ReentrancyGuard {
    fn drop(&mut self) {
        exit_callback();
    }
}

// =============================================================================
// FFI Function Reentrancy Check
// =============================================================================

/// Check if an FFI function is being called reentrantly.
///
/// This function should be called at the start of all FFI functions that
/// should NOT be called from within callbacks. It returns an error code if
/// reentrancy is detected, allowing the function to return early.
///
/// # Returns
///
/// * `None` - Safe to proceed (not in a callback)
/// * `Some(HEDL_ERR_REENTRANT_CALL)` - In a callback, should return error
///
/// # Examples
///
/// ```rust,ignore
/// use std::ffi::c_int;
/// use hedl_ffi::reentrancy::check_ffi_reentrancy;
///
/// #[no_mangle]
/// pub unsafe extern "C" fn hedl_some_function(arg: i32) -> c_int {
///     // Check for reentrancy
///     if let Some(err) = check_ffi_reentrancy() {
///         return err;
///     }
///
///     // ... rest of function ...
///     0
/// }
/// ```
#[must_use]
pub fn check_ffi_reentrancy() -> Option<c_int> {
    if in_callback() {
        Some(HEDL_ERR_REENTRANT_CALL)
    } else {
        None
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_enter_exit_callback() {
        assert!(!in_callback());

        assert!(enter_callback().is_ok());
        assert!(in_callback());

        exit_callback();
        assert!(!in_callback());
    }

    #[test]
    fn test_reentrancy_detection() {
        assert!(!in_callback());

        // First entry should succeed
        assert!(enter_callback().is_ok());
        assert!(in_callback());

        // Second entry should fail
        let result = enter_callback();
        assert_eq!(result, Err(HEDL_ERR_REENTRANT_CALL));

        // Still in callback
        assert!(in_callback());

        // Exit and verify
        exit_callback();
        assert!(!in_callback());

        // Should be able to enter again
        assert!(enter_callback().is_ok());
        exit_callback();
    }

    #[test]
    fn test_guard_automatic_cleanup() {
        assert!(!in_callback());

        {
            let _guard = ReentrancyGuard::enter().unwrap();
            assert!(in_callback());
            // Guard exits automatically when dropped
        }

        assert!(!in_callback());
    }

    #[test]
    fn test_guard_reentrancy_detection() {
        let _guard1 = ReentrancyGuard::enter().unwrap();
        assert!(in_callback());

        // Try to create another guard while in callback
        let guard2 = ReentrancyGuard::enter();
        assert!(guard2.is_err());
        assert_eq!(guard2.unwrap_err(), HEDL_ERR_REENTRANT_CALL);

        // First guard is still active
        assert!(in_callback());

        drop(_guard1);
        assert!(!in_callback());
    }

    #[test]
    fn test_check_ffi_reentrancy() {
        // Not in callback - should return None
        assert!(check_ffi_reentrancy().is_none());

        // Enter callback
        let _guard = ReentrancyGuard::enter().unwrap();

        // In callback - should return error code
        assert_eq!(check_ffi_reentrancy(), Some(HEDL_ERR_REENTRANT_CALL));
    }

    #[test]
    fn test_callback_isolation_across_threads() {
        use std::sync::{Arc, Barrier};
        const NUM_THREADS: usize = 4;

        let barrier = Arc::new(Barrier::new(NUM_THREADS));
        let mut handles = vec![];

        for i in 0..NUM_THREADS {
            let barrier_clone = barrier.clone();
            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Each thread should be able to enter callback independently
                assert!(!in_callback());
                assert!(enter_callback().is_ok());
                assert!(in_callback());

                // Other threads should not affect this thread's callback state
                exit_callback();
                assert!(!in_callback());

                i
            });

            handles.push(handle);
        }

        // All threads should complete successfully
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_nested_callback_guard_failure() {
        // Simulate callback trying to call FFI function
        let outer_guard = ReentrancyGuard::enter().unwrap();

        // Try to call FFI function from within callback
        let result = check_ffi_reentrancy();
        assert!(result.is_some());

        // After outer guard drops, we should be able to call FFI again
        drop(outer_guard);
        assert!(check_ffi_reentrancy().is_none());
    }

    #[test]
    fn test_multiple_callbacks_in_sequence() {
        // Test that we can enter and exit callbacks multiple times
        for _ in 0..10 {
            let guard = ReentrancyGuard::enter().unwrap();
            assert!(in_callback());
            drop(guard);
            assert!(!in_callback());
        }
    }

    #[test]
    fn test_callback_state_persistence() {
        // Test that callback state persists across multiple operations
        assert!(enter_callback().is_ok());

        // Perform some operations while in callback
        assert!(in_callback());
        assert!(enter_callback().is_err()); // Can't re-enter
        assert!(in_callback());

        // Exit and verify state clears
        exit_callback();
        assert!(!in_callback());
        assert!(enter_callback().is_ok()); // Can re-enter now
        exit_callback();
    }
}
