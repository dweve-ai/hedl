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

//! API Key authentication implementation.

use crate::auth::{crypto::ApiKeyHasher, AuthError, ClientMetadata};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Information about an API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    /// Unique key identifier.
    pub key_id: String,

    /// Client ID this key belongs to.
    pub client_id: String,

    /// Scopes granted to this key.
    pub scopes: Vec<String>,

    /// When the key was created.
    pub created_at: DateTime<Utc>,

    /// When the key expires (optional).
    pub expires_at: Option<DateTime<Utc>>,

    /// Whether the key is revoked.
    pub revoked: bool,
}

/// Trait for API key storage backends.
#[async_trait]
pub trait ApiKeyStore: Send + Sync {
    /// Validate an API key and return associated client metadata.
    async fn validate(&self, key: &str) -> Result<ClientMetadata, AuthError>;

    /// Create a new API key for a client.
    async fn create(&self, client_id: &str, scopes: Vec<String>) -> Result<String, AuthError>;

    /// Revoke an API key.
    async fn revoke(&self, key: &str) -> Result<(), AuthError>;

    /// List all keys for a client.
    async fn list_for_client(&self, client_id: &str) -> Result<Vec<ApiKeyInfo>, AuthError>;
}

/// In-memory API key store for development and testing.
pub struct InMemoryApiKeyStore {
    /// Map from key hash to client metadata.
    keys: Arc<DashMap<String, ClientMetadata>>,

    /// Map from key ID to key hash (for revocation).
    key_ids: Arc<DashMap<String, String>>,

    /// Map from client ID to list of key IDs.
    client_keys: Arc<DashMap<String, Vec<String>>>,

    /// API key hasher.
    hasher: ApiKeyHasher,
}

impl InMemoryApiKeyStore {
    /// Create a new in-memory API key store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            keys: Arc::new(DashMap::new()),
            key_ids: Arc::new(DashMap::new()),
            client_keys: Arc::new(DashMap::new()),
            hasher: ApiKeyHasher::new(),
        }
    }

    /// Create a new API key and return it.
    pub fn create_key_sync(
        &self,
        client_id: &str,
        scopes: Vec<String>,
    ) -> Result<String, AuthError> {
        let key = self.hasher.generate_key("dev");
        let hash = self.hasher.hash(&key)?;

        let metadata = ClientMetadata {
            client_id: client_id.to_string(),
            scopes: scopes.clone(),
            rate_limit: None,
            created_at: Utc::now(),
            expires_at: None,
            metadata: None,
        };

        let key_id = Self::generate_key_id();

        // Store the key
        self.keys.insert(hash.clone(), metadata);
        self.key_ids.insert(key_id.clone(), hash);
        self.client_keys
            .entry(client_id.to_string())
            .or_default()
            .push(key_id.clone());

        Ok(key)
    }

    /// Revoke an API key synchronously.
    pub fn revoke_sync(&self, key: &str) -> Result<(), AuthError> {
        // Find the hash that matches this key
        let mut found_hash: Option<String> = None;
        for entry in self.keys.iter() {
            let stored_hash = entry.key();
            if self.hasher.verify(key, stored_hash).unwrap_or(false) {
                found_hash = Some(stored_hash.clone());
                break;
            }
        }

        if let Some(hash) = found_hash {
            // Get metadata before removing
            if let Some((_, metadata)) = self.keys.remove(&hash) {
                // Remove from key_ids (we need to find the key_id first)
                self.key_ids.retain(|_, v| v != &hash);

                // Remove from client_keys
                if let Some(mut keys) = self.client_keys.get_mut(&metadata.client_id) {
                    keys.retain(|k| {
                        if let Some(kv) = self.key_ids.get(k) {
                            kv.value() != &hash
                        } else {
                            false
                        }
                    });
                }
            }
        }

        Ok(())
    }

    /// Generate a unique key ID.
    ///
    /// Falls back to timestamp-based ID if OS RNG is unavailable.
    fn generate_key_id() -> String {
        use ring::rand::{SecureRandom, SystemRandom};
        let rng = SystemRandom::new();
        let mut bytes = [0u8; 16];
        if let Ok(()) = rng.fill(&mut bytes) {
            hex::encode(bytes)
        } else {
            // Fallback: use timestamp + process ID for uniqueness
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{:016x}{:08x}", now.as_nanos(), std::process::id())
        }
    }
}

impl Default for InMemoryApiKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ApiKeyStore for InMemoryApiKeyStore {
    async fn validate(&self, key: &str) -> Result<ClientMetadata, AuthError> {
        // Iterate over all stored hashes and verify against each one
        // This is necessary because Argon2 uses random salts
        for entry in self.keys.iter() {
            let stored_hash = entry.key();
            if self.hasher.verify(key, stored_hash).unwrap_or(false) {
                return Ok(entry.value().clone());
            }
        }

        Err(AuthError::InvalidCredentials)
    }

    async fn create(&self, client_id: &str, scopes: Vec<String>) -> Result<String, AuthError> {
        self.create_key_sync(client_id, scopes)
    }

    async fn revoke(&self, key: &str) -> Result<(), AuthError> {
        self.revoke_sync(key)
    }

    async fn list_for_client(&self, client_id: &str) -> Result<Vec<ApiKeyInfo>, AuthError> {
        let key_ids = self
            .client_keys
            .get(client_id)
            .map(|kv| kv.clone())
            .unwrap_or_default();

        let mut infos = Vec::new();
        for key_id in &key_ids {
            if let Some(entry) = self.key_ids.get(key_id) {
                let hash = entry.value();
                if let Some(entry) = self.keys.get(hash) {
                    let metadata = entry.value();
                    infos.push(ApiKeyInfo {
                        key_id: key_id.to_string(),
                        client_id: metadata.client_id.to_string(),
                        scopes: metadata.scopes.iter().map(|s| s.to_string()).collect(),
                        created_at: metadata.created_at,
                        expires_at: metadata.expires_at,
                        revoked: false,
                    });
                }
            }
        }

        Ok(infos)
    }
}

/// API Key authentication.
pub struct ApiKeyAuth {
    /// Storage backend for API keys.
    key_store: Arc<dyn ApiKeyStore>,

    /// Optional key prefix for identification (e.g., "hedl_").
    key_prefix: Option<String>,
}

impl ApiKeyAuth {
    /// Create a new API Key authentication handler.
    ///
    /// # Arguments
    ///
    /// * `key_store` - Storage backend for API keys
    /// * `key_prefix` - Optional key prefix for identification
    pub fn new(key_store: Arc<dyn ApiKeyStore>, key_prefix: Option<String>) -> Self {
        Self {
            key_store,
            key_prefix,
        }
    }

    /// Authenticate a request using an API key.
    ///
    /// # Arguments
    ///
    /// * `key` - The API key from the request
    ///
    /// # Returns
    ///
    /// Client metadata if authentication succeeds.
    pub async fn authenticate(&self, key: &str) -> Result<ClientMetadata, AuthError> {
        // Check key prefix if configured
        if let Some(prefix) = &self.key_prefix {
            if !key.starts_with(prefix) {
                return Err(AuthError::InvalidCredentials);
            }
        }

        // Validate against key store
        self.key_store.validate(key).await
    }

    /// Create a new API key for a client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client identifier
    /// * `scopes` - Scopes to grant
    ///
    /// # Returns
    ///
    /// The new API key.
    pub async fn create_key(
        &self,
        client_id: &str,
        scopes: Vec<String>,
    ) -> Result<String, AuthError> {
        self.key_store.create(client_id, scopes).await
    }

    /// Revoke an API key.
    ///
    /// # Arguments
    ///
    /// * `key` - The API key to revoke
    pub async fn revoke_key(&self, key: &str) -> Result<(), AuthError> {
        self.key_store.revoke(key).await
    }

    /// List all keys for a client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client identifier
    ///
    /// # Returns
    ///
    /// List of API key information.
    pub async fn list_keys(&self, client_id: &str) -> Result<Vec<ApiKeyInfo>, AuthError> {
        self.key_store.list_for_client(client_id).await
    }
}

/// File-based API key store for single-server deployments.
pub struct FileApiKeyStore {
    /// Path to the keys file.
    path: std::path::PathBuf,

    /// In-memory cache of keys.
    cache: Arc<DashMap<String, ClientMetadata>>,

    /// API key hasher.
    hasher: ApiKeyHasher,
}

impl FileApiKeyStore {
    /// Create a new file-based API key store.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the keys file (JSON format)
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AuthError> {
        let path_buf = path.as_ref().to_path_buf();
        let cache = Arc::new(DashMap::new());
        let hasher = ApiKeyHasher::new();

        let store = Self {
            path: path_buf.clone(),
            cache,
            hasher,
        };

        // Load existing keys if file exists
        if path_buf.exists() {
            store.load()?;
        }

        Ok(store)
    }

    /// Load keys from file.
    fn load(&self) -> Result<(), AuthError> {
        let content = std::fs::read_to_string(&self.path).map_err(AuthError::Io)?;
        let keys: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(obj) = keys.as_object() {
            for (key_hash, metadata) in obj {
                if let Ok(metadata) = serde_json::from_value::<ClientMetadata>(metadata.clone()) {
                    self.cache.insert(key_hash.clone(), metadata);
                }
            }
        }

        Ok(())
    }

    /// Save keys to file.
    fn save(&self) -> Result<(), AuthError> {
        let mut obj = serde_json::Map::new();

        for entry in self.cache.iter() {
            let key = entry.key();
            let value = entry.value();
            if let Ok(metadata) = serde_json::to_value(value) {
                obj.insert(key.clone(), metadata);
            }
        }

        let content = serde_json::to_string_pretty(&obj).map_err(AuthError::Json)?;
        std::fs::write(&self.path, content).map_err(AuthError::Io)?;

        Ok(())
    }
}

#[async_trait]
impl ApiKeyStore for FileApiKeyStore {
    async fn validate(&self, key: &str) -> Result<ClientMetadata, AuthError> {
        let hash = self.hasher.hash(key)?;

        self.cache
            .get(&hash)
            .map(|kv| kv.clone())
            .ok_or(AuthError::InvalidCredentials)
    }

    async fn create(&self, client_id: &str, scopes: Vec<String>) -> Result<String, AuthError> {
        let hasher = ApiKeyHasher::new();
        let key = hasher.generate_key("prod");
        let hash = self.hasher.hash(&key)?;

        let metadata = ClientMetadata {
            client_id: client_id.to_string(),
            scopes,
            rate_limit: None,
            created_at: Utc::now(),
            expires_at: None,
            metadata: None,
        };

        self.cache.insert(hash.clone(), metadata);
        self.save()?;

        Ok(key)
    }

    async fn revoke(&self, key: &str) -> Result<(), AuthError> {
        let hash = self.hasher.hash(key)?;
        self.cache.remove(&hash);
        self.save()?;
        Ok(())
    }

    async fn list_for_client(&self, client_id: &str) -> Result<Vec<ApiKeyInfo>, AuthError> {
        let mut infos = Vec::new();

        for entry in self.cache.iter() {
            let metadata = entry.value();
            if metadata.client_id == client_id {
                infos.push(ApiKeyInfo {
                    key_id: "file-store".to_string(),
                    client_id: metadata.client_id.clone(),
                    scopes: metadata.scopes.clone(),
                    created_at: metadata.created_at,
                    expires_at: metadata.expires_at,
                    revoked: false,
                });
            }
        }

        Ok(infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_store_create_and_validate() {
        let store = InMemoryApiKeyStore::new();
        let scopes = vec!["read".to_string(), "write".to_string()];

        let key = store.create("client-1", scopes.clone()).await.unwrap();

        let metadata = store.validate(&key).await.unwrap();
        assert_eq!(metadata.client_id, "client-1");
        assert_eq!(metadata.scopes, scopes);
    }

    #[tokio::test]
    async fn test_in_memory_store_invalid_key() {
        let store = InMemoryApiKeyStore::new();

        let result = store.validate("invalid_key").await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_in_memory_store_revoke() {
        let store = InMemoryApiKeyStore::new();
        let key = store.create("client-1", vec![]).await.unwrap();

        // Key should work initially
        assert!(store.validate(&key).await.is_ok());

        // Revoke the key
        store.revoke(&key).await.unwrap();

        // Key should no longer work
        assert!(matches!(
            store.validate(&key).await,
            Err(AuthError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn test_api_key_auth_with_prefix() {
        let store = Arc::new(InMemoryApiKeyStore::new());
        let auth = ApiKeyAuth::new(store, Some("hedl_".to_string()));

        let key = auth
            .create_key("client-1", vec!["read".to_string()])
            .await
            .unwrap();

        // Valid key with prefix
        assert!(auth.authenticate(&key).await.is_ok());

        // Invalid key without prefix
        assert!(matches!(
            auth.authenticate("wrong_key").await,
            Err(AuthError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn test_in_memory_store_list_for_client() {
        let store = InMemoryApiKeyStore::new();

        store
            .create("client-1", vec!["read".to_string()])
            .await
            .unwrap();
        store
            .create("client-1", vec!["write".to_string()])
            .await
            .unwrap();
        store
            .create("client-2", vec!["read".to_string()])
            .await
            .unwrap();

        let client1_keys = store.list_for_client("client-1").await.unwrap();
        assert_eq!(client1_keys.len(), 2);

        let client2_keys = store.list_for_client("client-2").await.unwrap();
        assert_eq!(client2_keys.len(), 1);
    }

    #[test]
    fn test_api_key_hasher_different_keys() {
        let hasher = ApiKeyHasher::new();
        let hash1 = hasher.hash("key1").unwrap();
        let hash2 = hasher.hash("key2").unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_api_key_hasher_same_key_verifies() {
        // With Argon2, hashes use random salts so they won't be equal,
        // but the same key should verify against both hashes
        let hasher = ApiKeyHasher::new();
        let hash1 = hasher.hash("key1").unwrap();
        let hash2 = hasher.hash("key1").unwrap();

        // Hashes should be different (different salts)
        assert_ne!(hash1, hash2);

        // But the same key should verify against both
        assert!(hasher.verify("key1", &hash1).unwrap());
        assert!(hasher.verify("key1", &hash2).unwrap());

        // And a different key should not verify
        assert!(!hasher.verify("key2", &hash1).unwrap());
    }
}
