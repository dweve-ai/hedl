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

//! Thread pool implementation for async operations.

use super::operations::Worker;
use super::types::WorkItem;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

// =============================================================================
// Thread Pool Types
// =============================================================================

/// Thread pool for async operations.
pub(crate) struct ThreadPool {
    pub(crate) workers: Vec<Worker>,
    pub(crate) work_queue: Arc<Mutex<WorkQueue>>,
    pub(crate) condvar: Arc<Condvar>,
    pub(crate) shutdown: Arc<AtomicBool>,
    next_op_id: AtomicU64,
}

/// Work queue with bounded capacity.
pub(crate) struct WorkQueue {
    pub(crate) queue: VecDeque<WorkItem>,
    pub(crate) max_size: usize,
}

/// Global thread pool instance (initialized lazily).
pub(crate) static THREAD_POOL: OnceLock<ThreadPool> = OnceLock::new();

// =============================================================================
// Thread Pool Implementation
// =============================================================================

impl ThreadPool {
    /// Create a new thread pool.
    pub(crate) fn new(num_threads: usize, max_queue_size: usize) -> Self {
        let work_queue = Arc::new(Mutex::new(WorkQueue {
            queue: VecDeque::with_capacity(max_queue_size),
            max_size: max_queue_size,
        }));
        let condvar = Arc::new(Condvar::new());
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut workers = Vec::with_capacity(num_threads);
        for id in 0..num_threads {
            workers.push(Worker::new(
                id,
                work_queue.clone(),
                condvar.clone(),
                shutdown.clone(),
            ));
        }

        tracing::info!(
            num_threads = num_threads,
            max_queue_size = max_queue_size,
            "HEDL async thread pool initialized"
        );

        ThreadPool {
            workers,
            work_queue,
            condvar,
            shutdown,
            next_op_id: AtomicU64::new(1),
        }
    }

    /// Submit a work item to the queue.
    pub(crate) fn submit(&self, item: WorkItem) -> Result<(), ()> {
        let mut queue = self.work_queue.lock().expect("lock not poisoned");

        if queue.queue.len() >= queue.max_size {
            tracing::warn!(
                queue_size = queue.queue.len(),
                max_size = queue.max_size,
                "Async operation queue full"
            );
            return Err(());
        }

        queue.queue.push_back(item);
        drop(queue);

        self.condvar.notify_one();
        Ok(())
    }

    /// Get next operation ID.
    pub(crate) fn next_op_id(&self) -> u64 {
        self.next_op_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        tracing::info!("Shutting down HEDL async thread pool");

        // Signal shutdown
        self.shutdown.store(true, Ordering::Release);
        self.condvar.notify_all();

        // Wait for workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

/// Initialize thread pool (called lazily).
pub(crate) fn get_or_init_thread_pool() -> &'static ThreadPool {
    THREAD_POOL.get_or_init(|| {
        let num_threads = std::env::var("HEDL_ASYNC_THREADS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(num_cpus::get)
            .clamp(1, 64);

        let max_queue_size = std::env::var("HEDL_ASYNC_QUEUE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        ThreadPool::new(num_threads, max_queue_size)
    })
}
