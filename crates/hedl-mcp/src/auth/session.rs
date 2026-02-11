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

//! Session management for authenticated clients.

use crate::auth::{AuthError, ClientMetadata, SessionId};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Session configuration.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum session duration before re-authentication required.
    pub max_duration: Duration,

    /// Idle timeout (no requests).
    pub idle_timeout: Duration,

    /// Maximum requests per session.
    pub max_requests: Option<u64>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_duration: Duration::from_secs(24 * 60 * 60), // 24 hours
            idle_timeout: Duration::from_secs(60 * 60),      // 1 hour
            max_requests: None,
        }
    }
}

/// Active session for an authenticated client.
#[derive(Clone)]
pub struct Session {
    /// Unique session identifier.
    pub id: SessionId,

    /// Client metadata from authentication.
    pub client_metadata: ClientMetadata,

    /// When the session was created.
    pub created_at: Instant,

    /// Last activity timestamp.
    pub last_active: Arc<AtomicInstant>,

    /// Request counter.
    pub request_count: Arc<AtomicU64>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("client_id", &self.client_metadata.client_id)
            .field("created_at", &self.created_at)
            .field(
                "request_count",
                &self
                    .request_count
                    .load(std::sync::atomic::Ordering::Relaxed),
            )
            .finish()
    }
}

/// Wrapper around Instant for use in atomic operations.
pub struct AtomicInstant {
    inner: std::sync::RwLock<Instant>,
}

impl AtomicInstant {
    fn new(instant: Instant) -> Self {
        Self {
            inner: std::sync::RwLock::new(instant),
        }
    }

    fn store(&self, instant: Instant) {
        *self.inner.write().expect("lock not poisoned") = instant;
    }

    fn load(&self) -> Instant {
        *self.inner.read().expect("lock not poisoned")
    }
}

impl Clone for AtomicInstant {
    fn clone(&self) -> Self {
        Self::new(self.load())
    }
}

/// Session manager for tracking and validating sessions.
pub struct SessionRegistry {
    /// Active sessions.
    sessions: Arc<DashMap<SessionId, Session>>,

    /// Session configuration.
    config: SessionConfig,
}

impl SessionRegistry {
    /// Create a new session manager.
    ///
    /// # Arguments
    ///
    /// * `config` - Session configuration
    #[must_use]
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Create a new session for an authenticated client.
    ///
    /// # Arguments
    ///
    /// * `client_metadata` - Client metadata from authentication
    ///
    /// # Returns
    ///
    /// The new session.
    #[must_use]
    pub fn create_session(&self, client_metadata: ClientMetadata) -> Session {
        let session = Session {
            id: SessionId::new(),
            client_metadata,
            created_at: Instant::now(),
            last_active: Arc::new(AtomicInstant::new(Instant::now())),
            request_count: Arc::new(AtomicU64::new(0)),
        };

        self.sessions.insert(session.id.clone(), session.clone());
        session
    }

    /// Validate a session and update activity.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to validate
    ///
    /// # Returns
    ///
    /// The session if valid, error otherwise.
    pub fn validate_session(&self, session_id: &SessionId) -> Result<Session, AuthError> {
        // Extract data we need and drop the reference to avoid deadlock
        let (session_clone, created_at, last_active_elapsed, request_count) = {
            let session = self
                .sessions
                .get(session_id)
                .ok_or(AuthError::SessionNotFound)?;
            (
                session.clone(),
                session.created_at,
                session.last_active.load().elapsed(),
                session.request_count.load(Ordering::Relaxed),
            )
        };
        // Reference is now dropped, safe to call remove()

        // Check max duration
        if created_at.elapsed() > self.config.max_duration {
            self.sessions.remove(session_id);
            return Err(AuthError::SessionExpired);
        }

        // Check idle timeout
        if last_active_elapsed > self.config.idle_timeout {
            self.sessions.remove(session_id);
            return Err(AuthError::SessionIdle);
        }

        // Check request limit
        if let Some(max_requests) = self.config.max_requests {
            if request_count >= max_requests {
                self.sessions.remove(session_id);
                return Err(AuthError::SessionRequestLimitExceeded);
            }
        }

        // Update last active and request count
        session_clone.last_active.store(Instant::now());
        session_clone.request_count.fetch_add(1, Ordering::Relaxed);

        Ok(session_clone)
    }

    /// End a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to end
    #[must_use]
    pub fn end_session(&self, session_id: &SessionId) -> Option<Session> {
        self.sessions.remove(session_id).map(|(_, session)| session)
    }

    /// Cleanup expired sessions.
    ///
    /// Returns the number of sessions removed.
    #[must_use]
    pub fn cleanup_expired(&self) -> usize {
        let initial_count = self.sessions.len();
        let now = Instant::now();

        self.sessions.retain(|_, session| {
            let duration_valid = now.duration_since(session.created_at) < self.config.max_duration;
            let idle_valid =
                now.duration_since(session.last_active.load()) < self.config.idle_timeout;
            duration_valid && idle_valid
        });

        initial_count - self.sessions.len()
    }

    /// Get the number of active sessions.
    #[must_use]
    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get all active sessions.
    #[must_use]
    pub fn get_all_sessions(&self) -> Vec<Session> {
        self.sessions.iter().map(|entry| entry.clone()).collect()
    }
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_create_and_validate() {
        let manager = SessionRegistry::new(SessionConfig::default());

        let metadata = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec!["read".to_string()],
            ..Default::default()
        };

        let session = manager.create_session(metadata);
        assert_eq!(manager.active_session_count(), 1);

        let validated = manager.validate_session(&session.id).unwrap();
        assert_eq!(validated.id, session.id);
    }

    #[test]
    fn test_session_idle_timeout() {
        let config = SessionConfig {
            idle_timeout: Duration::from_millis(100),
            ..Default::default()
        };

        let manager = SessionRegistry::new(config);

        let metadata = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec![],
            ..Default::default()
        };

        let session = manager.create_session(metadata);

        // Wait for idle timeout
        std::thread::sleep(Duration::from_millis(150));

        let result = manager.validate_session(&session.id);
        assert!(matches!(result, Err(AuthError::SessionIdle)));
    }

    #[test]
    fn test_session_max_duration() {
        let config = SessionConfig {
            max_duration: Duration::from_millis(100),
            ..Default::default()
        };

        let manager = SessionRegistry::new(config);

        let metadata = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec![],
            ..Default::default()
        };

        let session = manager.create_session(metadata);

        // Wait for max duration
        std::thread::sleep(Duration::from_millis(150));

        let result = manager.validate_session(&session.id);
        assert!(matches!(result, Err(AuthError::SessionExpired)));
    }

    #[test]
    fn test_session_request_limit() {
        let config = SessionConfig {
            max_requests: Some(5),
            ..Default::default()
        };

        let manager = SessionRegistry::new(config);

        let metadata = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec![],
            ..Default::default()
        };

        let session = manager.create_session(metadata);

        // Use 5 requests
        for _ in 0..5 {
            manager.validate_session(&session.id).unwrap();
        }

        // 6th request should fail
        let result = manager.validate_session(&session.id);
        assert!(matches!(
            result,
            Err(AuthError::SessionRequestLimitExceeded)
        ));
    }

    #[test]
    fn test_session_end() {
        let manager = SessionRegistry::new(SessionConfig::default());

        let metadata = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec![],
            ..Default::default()
        };

        let session = manager.create_session(metadata);
        assert_eq!(manager.active_session_count(), 1);

        manager.end_session(&session.id);
        assert_eq!(manager.active_session_count(), 0);

        let result = manager.validate_session(&session.id);
        assert!(matches!(result, Err(AuthError::SessionNotFound)));
    }

    #[test]
    fn test_session_cleanup() {
        let config = SessionConfig {
            idle_timeout: Duration::from_millis(100),
            ..Default::default()
        };

        let manager = SessionRegistry::new(config);

        let metadata = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec![],
            ..Default::default()
        };

        manager.create_session(metadata.clone());
        manager.create_session(metadata);

        std::thread::sleep(Duration::from_millis(150));

        let removed = manager.cleanup_expired();
        assert_eq!(removed, 2);
        assert_eq!(manager.active_session_count(), 0);
    }
}
