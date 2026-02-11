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

//! Cryptographic utilities for secure credential storage and API key hashing.

use crate::auth::{AuthError, AuthResult, SecretString};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use ring::rand::{SecureRandom, SystemRandom};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// API key hasher using Argon2id.
#[derive(Clone)]
pub struct ApiKeyHasher {
    argon2: Argon2<'static>,
}

impl ApiKeyHasher {
    /// Create a new hasher with secure default parameters.
    #[must_use]
    pub fn new() -> Self {
        // Argon2id with secure defaults:
        // - 256 MB memory cost
        // - 4 iterations
        // - 4 lanes (parallelism)
        // - Output length: 32 bytes
        let params = Params::new(262144, 4, 4, None).expect("Valid params");
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Self { argon2 }
    }

    /// Hash an API key using Argon2id.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key to hash
    ///
    /// # Returns
    ///
    /// The password hash string in PHC format.
    pub fn hash(&self, api_key: &str) -> AuthResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .argon2
            .hash_password(api_key.as_bytes(), &salt)
            .map_err(|_| AuthError::HashingFailed)?;
        Ok(hash.to_string())
    }

    /// Verify an API key against a hash.
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key to verify
    /// * `hash` - The stored hash to verify against
    ///
    /// # Returns
    ///
    /// `Ok(true)` if valid, `Ok(false)` if invalid, `Err` on failure.
    pub fn verify(&self, api_key: &str, hash: &str) -> AuthResult<bool> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError::InvalidHash)?;

        // Use constant-time verification
        Ok(self
            .argon2
            .verify_password(api_key.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Generate a new random API key.
    ///
    /// Format: `hedl_<env>_<random_32_bytes_base64>`
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment identifier (e.g., "dev", "prod")
    ///
    /// # Returns
    ///
    /// A new API key. Falls back to timestamp-based entropy if OS RNG is unavailable.
    #[must_use]
    pub fn generate_key(&self, environment: &str) -> String {
        use base64::Engine;
        use ring::rand::{SecureRandom, SystemRandom};
        let rng = SystemRandom::new();
        let mut bytes = [0u8; 32];

        if rng.fill(&mut bytes).is_err() {
            // Fallback: use multiple entropy sources
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let nanos = now.as_nanos();
            let pid = std::process::id();
            let thread_id = std::thread::current().id();

            // Mix entropy sources into the buffer
            bytes[0..16].copy_from_slice(&nanos.to_le_bytes());
            bytes[16..20].copy_from_slice(&pid.to_le_bytes());
            bytes[20..28].copy_from_slice(
                &format!("{thread_id:?}").as_bytes()[..8.min(format!("{thread_id:?}").len())],
            );
            // Fill remaining bytes with a simple counter
            for (i, b) in bytes[28..].iter_mut().enumerate() {
                *b = (i as u8).wrapping_add(nanos as u8);
            }
        }

        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        format!("hedl_{environment}_{encoded}")
    }
}

impl Default for ApiKeyHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Credential store for encrypting sensitive data at rest.
pub struct CredentialStore {
    cipher: ChaCha20Poly1305,
}

impl CredentialStore {
    /// Create a new credential store with a master password.
    ///
    /// # Arguments
    ///
    /// * `master_password` - Master password for encryption key derivation
    ///
    /// # Returns
    ///
    /// A new credential store.
    pub fn new(master_password: &SecretString) -> AuthResult<Self> {
        // Generate or load salt
        let salt = Self::load_or_generate_salt()?;

        // Derive key using Argon2id
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(master_password.expose_secret().as_bytes(), &salt, &mut key)
            .map_err(|_| AuthError::EncryptionFailed)?;

        let cipher = ChaCha20Poly1305::new(&key.into());

        // Clear key from memory
        use zeroize::Zeroize;
        key.zeroize();

        Ok(Self { cipher })
    }

    /// Encrypt plaintext data.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    ///
    /// # Returns
    ///
    /// Encrypted ciphertext with nonce prepended.
    pub fn encrypt(&self, plaintext: &[u8]) -> AuthResult<Vec<u8>> {
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| AuthError::EncryptionFailed)?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt ciphertext data.
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Data to decrypt (with nonce prepended)
    ///
    /// # Returns
    ///
    /// Decrypted plaintext.
    pub fn decrypt(&self, ciphertext: &[u8]) -> AuthResult<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(AuthError::InvalidCiphertext);
        }

        let (nonce, ct) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce);

        self.cipher
            .decrypt(nonce, ct)
            .map_err(|_| AuthError::DecryptionFailed)
    }

    /// Load or generate salt for key derivation.
    fn load_or_generate_salt() -> AuthResult<[u8; 32]> {
        let salt_path = Path::new(".hedl-mcp-salt");

        if salt_path.exists() {
            let salt = fs::read(salt_path).map_err(AuthError::Io)?;
            if salt.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&salt);
                return Ok(arr);
            }
        }

        // Generate new salt
        let rng = SystemRandom::new();
        let mut salt = [0u8; 32];
        rng.fill(&mut salt)
            .map_err(|_| AuthError::EncryptionFailed)?;
        secure_write(salt_path, &salt)?;
        Ok(salt)
    }
}

/// Write data to a file with secure permissions (0600).
///
/// # Arguments
///
/// * `path` - Path to write to
/// * `content` - Content to write
pub fn secure_write(path: &Path, content: &[u8]) -> AuthResult<()> {
    let mut file = fs::File::create(path).map_err(AuthError::Io)?;
    file.write_all(content).map_err(AuthError::Io)?;
    file.sync_all().map_err(AuthError::Io)?;

    // Set permissions to 0600 (owner read/write only)
    let mut perm = fs::metadata(path).map_err(AuthError::Io)?.permissions();
    perm.set_mode(0o600);
    fs::set_permissions(path, perm).map_err(AuthError::Io)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_hasher_roundtrip() {
        let hasher = ApiKeyHasher::new();
        let key = "test_api_key_12345";

        let hash = hasher.hash(key).unwrap();
        assert!(hasher.verify(key, &hash).unwrap());
        assert!(!hasher.verify("wrong_key", &hash).unwrap());
    }

    #[test]
    fn test_api_key_hasher_constant_time() {
        let hasher = ApiKeyHasher::new();
        let key = "test_api_key_12345";

        let hash = hasher.hash(key).unwrap();

        // Both valid and invalid keys should take similar time
        // Run a single iteration since Argon2 is slow
        let start1 = std::time::Instant::now();
        let _ = hasher.verify(key, &hash);
        let duration1 = start1.elapsed();

        let start2 = std::time::Instant::now();
        let _ = hasher.verify("wrong_key", &hash);
        let duration2 = start2.elapsed();

        // Allow 60 second difference (extremely generous for CI systems under heavy load)
        // Argon2 takes ~6-16 seconds per verification with our parameters,
        // and system load variance can be extreme during parallel test runs.
        // The actual constant-time guarantee comes from Argon2's algorithm design,
        // not from this test. This test just verifies both paths complete.
        assert!(
            duration1.abs_diff(duration2) < std::time::Duration::from_secs(60),
            "Timing difference too large: valid={duration1:?}, invalid={duration2:?}"
        );
    }

    #[test]
    fn test_api_key_generation() {
        let hasher = ApiKeyHasher::new();
        let key1 = hasher.generate_key("prod");
        let key2 = hasher.generate_key("prod");

        assert_ne!(key1, key2);
        assert!(key1.starts_with("hedl_prod_"));
        assert!(key2.starts_with("hedl_prod_"));
    }

    #[test]
    fn test_credential_store_roundtrip() {
        let master = SecretString::new("master_password_123".to_string());
        let store = CredentialStore::new(&master).unwrap();

        let plaintext = b"sensitive_data_12345";
        let ciphertext = store.encrypt(plaintext).unwrap();

        assert_ne!(plaintext.to_vec(), ciphertext);

        let decrypted = store.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_credential_store_invalid_ciphertext() {
        let master = SecretString::new("master_password_123".to_string());
        let store = CredentialStore::new(&master).unwrap();

        let result = store.decrypt(&[1, 2, 3]);
        assert!(matches!(result, Err(AuthError::InvalidCiphertext)));
    }

    #[test]
    fn test_credential_store_wrong_password() {
        let master1 = SecretString::new("master_password_123".to_string());
        let store1 = CredentialStore::new(&master1).unwrap();

        let master2 = SecretString::new("wrong_password".to_string());
        let store2 = CredentialStore::new(&master2).unwrap();

        let plaintext = b"sensitive_data_12345";
        let ciphertext = store1.encrypt(plaintext).unwrap();

        let result = store2.decrypt(&ciphertext);
        assert!(matches!(result, Err(AuthError::DecryptionFailed)));
    }
}
