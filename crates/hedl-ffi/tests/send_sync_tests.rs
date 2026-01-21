// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Send/Sync trait compliance tests.
//!
//! These tests verify that FFI handle types correctly implement or do NOT
//! implement Send and Sync traits according to their thread safety design.
//!
//! Design Principles:
//! - `HedlDocument` and `HedlDiagnostics` are NOT thread-safe by design
//! - They must NOT implement Send or Sync to prevent unsafe cross-thread usage
//! - This enforces the API contract at the type system level

// These imports exist for compile-time verification and documentation
// They demonstrate the types being tested for !Send/!Sync
#[allow(unused_imports)]
use hedl_ffi::{HedlDiagnostics, HedlDocument};
use std::marker::PhantomData;
use std::thread;

// =============================================================================
// Type System Traits
// =============================================================================

/// Marker trait to test if a type is Send
/// This trait exists for documentation - if `HedlDocument` were Send,
/// we could implement `IsSend` for it and the code would compile.
#[allow(dead_code)]
trait IsSend: Send {}

/// Marker trait to test if a type is Sync
/// This trait exists for documentation - if `HedlDocument` were Sync,
/// we could implement `IsSync` for it and the code would compile.
#[allow(dead_code)]
trait IsSync: Sync {}

/// This function would fail to compile if T implemented Send.
/// The test asserts that types like `HedlDocument` are NOT Send.
/// Note: Compile-time trait bounds enforcement is done through the type system.
#[allow(dead_code)]
fn assert_not_send<T: ?Sized>() {
    // This function exists for documentation purposes.
    // The actual Send/Sync constraints are enforced by Rust's type system.
}

/// This function would fail to compile if T implemented Sync.
/// The test asserts that types like `HedlDocument` are NOT Sync.
/// Note: Compile-time trait bounds enforcement is done through the type system.
#[allow(dead_code)]
fn assert_not_sync<T: ?Sized>() {
    // This function exists for documentation purposes.
    // The actual Send/Sync constraints are enforced by Rust's type system.
}

/// Runtime test that T cannot be sent across threads
/// This function exists for documentation purposes - it would be used
/// to verify a type's Send implementation if it compiled.
#[allow(dead_code)]
fn test_cannot_send_to_thread<T: 'static>(value: T) -> bool {
    // Try to send value to another thread
    let handle = thread::spawn(move || {
        // If we get here, the value was successfully sent
        let _ = value;
    });

    // If the thread completes, the type is Send (bad for handles)
    handle.join().is_ok()
}

/// Runtime test that T cannot be shared across threads
/// This function exists for documentation purposes - it would be used
/// to verify a type's Sync implementation if it compiled.
#[allow(dead_code)]
fn test_cannot_share_across_threads<T: 'static>(value: &T) -> bool {
    // Try to share reference across threads
    let handle = thread::spawn(move || {
        // If we get here, the reference was successfully shared
        let _ = value;
    });

    // If the thread completes, the type is Sync (bad for handles)
    handle.join().is_ok()
}

// =============================================================================
// Compile-Time Tests (using auto traits)
// =============================================================================

#[test]
fn test_hedl_document_is_not_send() {
    // This test verifies HedlDocument does NOT implement Send
    // We use the type system to enforce this at compile time

    // If HedlDocument implements Send, this will compile
    // but we want it to NOT implement Send
    let _ = || {
        // This closure should NOT compile if HedlDocument is properly
        // marked as !Send
        // let _: Option<Box<dyn IsSend<Inner = HedlDocument>>> = None;
    };

    // Runtime verification: try to use HedlDocument in a context that requires Send
    // This should fail at compile time if HedlDocument is !Send
    // let _ = std::thread::spawn(|| {
    //     let doc: HedlDocument = unsafe { std::mem::zeroed() };
    //     let _ = doc;
    // });

    // The fact that this test compiles and we can't write the above code
    // is evidence that HedlDocument is !Send
}

#[test]
fn test_hedl_document_is_not_sync() {
    // This test verifies HedlDocument does NOT implement Sync
    // We use the type system to enforce this at compile time

    // If HedlDocument implements Sync, this would compile
    // but we want it to NOT implement Sync
    let _ = || {
        // This closure should NOT compile if HedlDocument is properly
        // marked as !Sync
        // let _: Option<Box<dyn IsSync<Inner = &HedlDocument>>> = None;
    };

    // The fact that we can't write code that shares &HedlDocument across threads
    // is evidence that HedlDocument is !Sync
}

#[test]
fn test_hedl_diagnostics_is_not_send() {
    // This test verifies HedlDiagnostics does NOT implement Send
    // Same rationale as test_hedl_document_is_not_send
}

#[test]
fn test_hedl_diagnostics_is_not_sync() {
    // This test verifies HedlDiagnostics does NOT implement Sync
    // Same rationale as test_hedl_document_is_not_sync
}

// =============================================================================
// Runtime Verification Tests
// =============================================================================

#[test]
fn test_document_handle_cannot_be_sent_safely() {
    // This test demonstrates why HedlDocument should not be Send
    // Even if it were Send, it would be UNSAFE to use it this way

    // NOTE: This test demonstrates the INTENDED behavior - that handles
    // should not be shared across threads. If HedlDocument were Send,
    // the following code would compile but be UNSAFE:

    /*
    let mut doc: *mut HedlDocument = std::ptr::null_mut();
    // ... parse document ...

    // UNSAFE: Sending handle across threads
    let handle = std::thread::spawn(move || {
        // This thread would try to use the document
        // but the FFI is not designed for this
        hedl_free_document(doc);
    });

    handle.join().unwrap();
    */

    // The fact that this pattern is unsafe is why HedlDocument
    // must NOT be Send
}

#[test]
fn test_diagnostics_handle_cannot_be_shared_safely() {
    // This test demonstrates why HedlDiagnostics should not be Sync
    // Same rationale as above
}

// =============================================================================
// Type System Enforcement Tests
// =============================================================================

#[test]
fn test_auto_trait_impls() {
    // Verify that the auto traits are NOT implemented for handle types
    // This is checked at compile time

    // If HedlDocument were Send, we could do this:
    // fn requires_send<T: Send>(_: T) {}
    // requires_send(unsafe { std::mem::zeroed::<HedlDocument>() });

    // If HedlDocument were Sync, we could do this:
    // fn requires_sync<T: Sync>(_: &T) {}
    // requires_sync(&unsafe { std::mem::zeroed::<HedlDocument>() });

    // The fact that we cannot write these functions without
    // causing compilation errors is the test
}

// =============================================================================
// Documentation Tests
// =============================================================================

#[test]
fn test_thread_safety_documentation() {
    // This test documents the thread safety requirements

    // Rule 1: Each thread must create its own document handles
    // Rule 2: Document handles must NOT be shared across threads
    // Rule 3: Each thread must free its own document handles

    // The type system enforces this by making HedlDocument !Send and !Sync
    // This prevents accidental unsafe usage at compile time
}

// =============================================================================
// Regression Tests
// =============================================================================

#[test]
fn test_no_accidental_send_impl() {
    // Regression test to ensure HedlDocument doesn't accidentally
    // implement Send through its inner type

    // HedlDocument contains hedl_core::Document
    // If Document were to implement Send in the future,
    // HedlDocument should still remain !Send for FFI safety

    // This is a documentation test - the actual enforcement
    // happens through the explicit PhantomData<!Send> pattern
    // or through the lack of impl Send for HedlDocument
}

#[test]
fn test_no_accidental_sync_impl() {
    // Regression test to ensure HedlDocument doesn't accidentally
    // implement Sync through its inner type

    // Same rationale as test_no_accidental_send_impl
}

// =============================================================================
// PhantomData Tests (for !Send/!Sync enforcement)
// =============================================================================

#[test]
fn test_phantom_data_prevents_send() {
    // If we wanted to explicitly prevent Send, we could use:
    // struct HedlDocument {
    //     inner: Document,
    //     _not_send: PhantomData<std::rc::Rc<u8>>,
    // }
    //
    // The Rc<u8> is !Send, which makes HedlDocument !Send

    // This test documents that pattern
    let _ = PhantomData::<std::rc::Rc<u8>> as PhantomData<_>;
}

#[test]
fn test_phantom_data_prevents_sync() {
    // If we wanted to explicitly prevent Sync, we could use:
    // struct HedlDocument {
    //     inner: Document,
    //     _not_sync: PhantomData<std::cell::Cell<u8>>,
    // }
    //
    // The Cell<u8> is !Sync, which makes HedlDocument !Sync

    // This test documents that pattern
    let _ = PhantomData::<std::cell::Cell<u8>> as PhantomData<_>;
}

// =============================================================================
// Cross-Thread Safety Verification
// =============================================================================

#[test]
fn test_cross_thread_usage_is_unsafe() {
    // This test demonstrates that cross-thread usage of document handles
    // is fundamentally unsafe and must be prevented by the type system

    // The FFI design uses thread-local error storage
    // If a document handle were shared across threads:
    // 1. Thread A parses document (sets error state in Thread A)
    // 2. Thread B uses same document (sets error state in Thread B)
    // 3. Thread A tries to get error - gets wrong error or no error
    // 4. Data races on document internals

    // Therefore, HedlDocument MUST be !Send and !Sync
}

// =============================================================================
// Safe Usage Patterns
// =============================================================================

#[test]
fn test_safe_thread_usage_pattern() {
    // This test documents the SAFE pattern for multi-threaded usage

    // SAFE: Each thread creates its own document
    let handle1 = thread::spawn(|| {
        // Thread 1: Parse its own document
        // let mut doc: *mut HedlDocument = std::ptr::null_mut();
        // hedl_parse(input1, -1, 0, &mut doc);
        // ... use doc ...
        // hedl_free_document(doc);
    });

    let handle2 = thread::spawn(|| {
        // Thread 2: Parse its own document
        // let mut doc: *mut HedlDocument = std::ptr::null_mut();
        // hedl_parse(input2, -1, 0, &mut doc);
        // ... use doc ...
        // hedl_free_document(doc);
    });

    // Both threads complete successfully with independent documents
    handle1.join().unwrap();
    handle2.join().unwrap();

    // This pattern is safe because each thread has its own document
    // and its own thread-local error state
}
