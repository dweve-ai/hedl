// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive authentication and authorization tests.

use hedl_mcp::auth::*;
use hedl_mcp::authorization::DefaultPolicy;
use std::sync::Arc;

#[tokio::test]
async fn test_api_key_lifecycle() {
    let store = Arc::new(InMemoryApiKeyStore::new());
    let auth = ApiKeyAuth::new(store.clone(), Some("hedl_".to_string()));

    // Create a key
    let key = auth
        .create_key("test-client", vec!["read".to_string(), "write".to_string()])
        .await
        .unwrap();
    assert!(key.starts_with("hedl_"));

    // Authenticate with the key
    let metadata = auth.authenticate(&key).await.unwrap();
    assert_eq!(metadata.client_id, "test-client");
    assert_eq!(metadata.scopes.len(), 2);

    // List keys for client
    let keys = auth.list_keys("test-client").await.unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].client_id, "test-client");

    // Revoke the key
    auth.revoke_key(&key).await.unwrap();

    // Key should no longer work
    let result = auth.authenticate(&key).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn test_api_key_prefix_validation() {
    let store = Arc::new(InMemoryApiKeyStore::new());
    let auth = ApiKeyAuth::new(store.clone(), Some("hedl_".to_string()));

    // Create a key
    let key = auth.create_key("test-client", vec![]).await.unwrap();

    // Key with wrong prefix should fail
    let wrong_key = format!("wrong_{}", &key[5..]);
    let result = auth.authenticate(&wrong_key).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));

    // Key without prefix should fail
    let result = auth.authenticate(&key[5..]).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn test_api_key_no_prefix() {
    let store = Arc::new(InMemoryApiKeyStore::new());
    let auth = ApiKeyAuth::new(store.clone(), None);

    let key = auth.create_key("test-client", vec![]).await.unwrap();

    // Should work without prefix requirement
    let result = auth.authenticate(&key).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_clients() {
    let store = Arc::new(InMemoryApiKeyStore::new());

    // Create keys for different clients
    let key1 = store
        .create("client-1", vec!["read".to_string()])
        .await
        .unwrap();
    let key2 = store
        .create("client-2", vec!["write".to_string()])
        .await
        .unwrap();

    // Validate keys return correct client metadata
    let metadata1 = store.validate(&key1).await.unwrap();
    assert_eq!(metadata1.client_id, "client-1");
    assert_eq!(metadata1.scopes, vec!["read"]);

    let metadata2 = store.validate(&key2).await.unwrap();
    assert_eq!(metadata2.client_id, "client-2");
    assert_eq!(metadata2.scopes, vec!["write"]);

    // List keys per client
    let client1_keys = store.list_for_client("client-1").await.unwrap();
    assert_eq!(client1_keys.len(), 1);

    let client2_keys = store.list_for_client("client-2").await.unwrap();
    assert_eq!(client2_keys.len(), 1);
}

#[test]
fn test_authorization_path_patterns() {
    use std::path::Path;

    let mut policy = AuthorizationPolicy::new();
    policy.add_rule(PolicyRule {
        subject: SubjectMatcher::Scope {
            scope: "admin".to_string(),
        },
        resource: AuthResource::path("data/*.hedl"),
        actions: vec![Action::Read, Action::Write],
        conditions: vec![],
    });

    let client = ClientMetadata {
        client_id: "admin-user".to_string(),
        scopes: vec!["admin".to_string()],
        ..Default::default()
    };

    // Should allow matching paths
    assert!(policy
        .check_path_read(&client, Path::new("data/test.hedl"))
        .is_ok());

    // Should deny non-matching paths
    assert!(policy
        .check_path_read(&client, Path::new("other/test.hedl"))
        .is_err());
}

#[test]
fn test_authorization_tool_access() {
    let mut policy = AuthorizationPolicy::new();
    policy.add_rule(PolicyRule {
        subject: SubjectMatcher::Scope {
            scope: "validator".to_string(),
        },
        resource: AuthResource::tool("hedl_validate"),
        actions: vec![Action::Execute],
        conditions: vec![],
    });

    let client = ClientMetadata {
        client_id: "test-client".to_string(),
        scopes: vec!["validator".to_string()],
        ..Default::default()
    };

    // Should allow hedl_validate
    assert!(policy.check_tool(&client, "hedl_validate").is_ok());

    // Should deny other tools
    assert!(policy.check_tool(&client, "hedl_write").is_err());
}

#[test]
fn test_authorization_multiple_rules() {
    let mut policy = AuthorizationPolicy::new();

    // Rule 1: Read access for readers
    policy.add_rule(PolicyRule {
        subject: SubjectMatcher::Scope {
            scope: "read".to_string(),
        },
        resource: AuthResource::All,
        actions: vec![Action::Read],
        conditions: vec![],
    });

    // Rule 2: Write access for writers
    policy.add_rule(PolicyRule {
        subject: SubjectMatcher::Scope {
            scope: "write".to_string(),
        },
        resource: AuthResource::All,
        actions: vec![Action::Write],
        conditions: vec![],
    });

    let reader = ClientMetadata {
        client_id: "reader".to_string(),
        scopes: vec!["read".to_string()],
        ..Default::default()
    };

    let writer = ClientMetadata {
        client_id: "writer".to_string(),
        scopes: vec!["write".to_string()],
        ..Default::default()
    };

    // Reader can read but not write
    assert!(policy
        .check(&reader, &AuthResource::All, &Action::Read)
        .is_ok());
    assert!(policy
        .check(&reader, &AuthResource::All, &Action::Write)
        .is_err());

    // Writer can write but not read
    assert!(policy
        .check(&writer, &AuthResource::All, &Action::Write)
        .is_ok());
    assert!(policy
        .check(&writer, &AuthResource::All, &Action::Read)
        .is_err());
}

#[test]
fn test_authorization_client_pattern_matching() {
    let pattern_matcher = SubjectMatcher::ClientPattern {
        pattern: "service-*".to_string(),
    };

    let matching_client = ClientMetadata {
        client_id: "service-auth".to_string(),
        scopes: vec![],
        ..Default::default()
    };

    let non_matching_client = ClientMetadata {
        client_id: "user-123".to_string(),
        scopes: vec![],
        ..Default::default()
    };

    assert!(pattern_matcher.matches(&matching_client));
    assert!(!pattern_matcher.matches(&non_matching_client));
}

#[test]
fn test_authorization_group_matching() {
    use serde_json::json;

    let group_matcher = SubjectMatcher::Group {
        name: "developers".to_string(),
    };

    let client_in_group = ClientMetadata {
        client_id: "user-1".to_string(),
        scopes: vec![],
        metadata: Some(json!({
            "groups": ["developers", "users"]
        })),
        ..Default::default()
    };

    let client_not_in_group = ClientMetadata {
        client_id: "user-2".to_string(),
        scopes: vec![],
        metadata: Some(json!({
            "groups": ["users"]
        })),
        ..Default::default()
    };

    assert!(group_matcher.matches(&client_in_group));
    assert!(!group_matcher.matches(&client_not_in_group));
}

#[test]
fn test_action_parsing() {
    assert_eq!(Action::from_str("read").unwrap(), Action::Read);
    assert_eq!(Action::from_str("WRITE").unwrap(), Action::Write);
    assert_eq!(Action::from_str("Execute").unwrap(), Action::Execute);
    assert_eq!(Action::from_str("validate").unwrap(), Action::Validate);
    assert_eq!(Action::from_str("convert").unwrap(), Action::Convert);
    assert_eq!(Action::from_str("delete").unwrap(), Action::Delete);

    assert!(Action::from_str("unknown").is_err());
    assert!(Action::from_str("").is_err());
}

#[test]
fn test_path_pattern_matching() {
    use std::path::Path;

    let pattern = PathPattern::new("data/**/*.hedl").unwrap();

    assert!(pattern.matches(Path::new("data/test.hedl")));
    assert!(pattern.matches(Path::new("data/sub/test.hedl")));
    assert!(pattern.matches(Path::new("data/deep/nested/test.hedl")));
    assert!(!pattern.matches(Path::new("data/test.json")));
    assert!(!pattern.matches(Path::new("other/test.hedl")));
}

#[test]
fn test_path_pattern_invalid() {
    let result = PathPattern::new("[invalid");
    assert!(result.is_err());
}

#[test]
fn test_resource_matching_tool() {
    let tool1 = AuthResource::tool("hedl_read");
    let tool2 = AuthResource::tool("hedl_write");
    let all = AuthResource::All;

    assert!(tool1.matches(&tool1));
    assert!(!tool1.matches(&tool2));
    assert!(all.matches(&tool1));
    assert!(all.matches(&tool2));
}

#[test]
fn test_resource_matching_entity_type() {
    let user_type = AuthResource::EntityType {
        name: "User".to_string(),
    };
    let product_type = AuthResource::EntityType {
        name: "Product".to_string(),
    };

    assert!(user_type.matches(&user_type));
    assert!(!user_type.matches(&product_type));
}

#[test]
fn test_condition_time_window_valid() {
    let condition = Condition::TimeWindow {
        start: "00:00:00".to_string(),
        end: "23:59:59".to_string(),
    };

    // Current time should be within this window
    assert!(condition.check().is_ok());
}

#[test]
fn test_condition_time_window_narrow() {
    // Create a narrow window that might not include current time
    let condition = Condition::TimeWindow {
        start: "00:00:00".to_string(),
        end: "00:00:01".to_string(),
    };

    // This might fail or pass depending on when test runs
    let result = condition.check();

    // If we're not in the window, should get Forbidden error
    if result.is_err() {
        assert!(matches!(result, Err(AuthError::Forbidden)));
    }
}

#[test]
fn test_condition_ip_whitelist() {
    let condition = Condition::IpWhitelist {
        ips: vec!["192.168.1.1".to_string(), "10.0.0.1".to_string()],
    };

    // IP-based checks are handled at network layer, so this should always pass
    assert!(condition.check().is_ok());
}

#[test]
fn test_condition_rate_limit() {
    let condition = Condition::RateLimit {
        requests: 100,
        window_seconds: 60,
    };

    // Rate limiting is handled separately, so this should always pass
    assert!(condition.check().is_ok());
}

#[test]
fn test_condition_custom() {
    use serde_json::json;

    let condition = Condition::Custom {
        name: "custom_check".to_string(),
        config: json!({ "key": "value" }),
    };

    // Custom conditions are evaluated by application, so this should always pass
    assert!(condition.check().is_ok());
}

#[test]
fn test_default_policy_deny() {
    let policy = AuthorizationPolicy::new();

    let client = ClientMetadata {
        client_id: "test".to_string(),
        scopes: vec![],
        ..Default::default()
    };

    // No rules, default deny
    assert!(policy
        .check(&client, &AuthResource::All, &Action::Read)
        .is_err());
}

#[test]
fn test_default_policy_allow() {
    let mut policy = AuthorizationPolicy::new();
    policy.default_policy = DefaultPolicy::Allow;

    let client = ClientMetadata {
        client_id: "test".to_string(),
        scopes: vec![],
        ..Default::default()
    };

    // No rules, default allow
    assert!(policy
        .check(&client, &AuthResource::All, &Action::Read)
        .is_ok());
}

#[tokio::test]
async fn test_session_lifecycle() {
    use std::time::Duration;

    let config = SessionConfig {
        max_duration: Duration::from_secs(3600),
        idle_timeout: Duration::from_secs(600),
        max_requests: Some(100),
    };

    let manager = SessionManager::new(config);

    // Create session
    let metadata = ClientMetadata {
        client_id: "test-client".to_string(),
        scopes: vec!["read".to_string()],
        ..Default::default()
    };

    let session = manager.create_session(metadata);
    assert_eq!(manager.active_session_count(), 1);

    // Validate session
    let validated = manager.validate_session(&session.id).unwrap();
    assert_eq!(validated.client_metadata.client_id, "test-client");

    // End session
    let ended = manager.end_session(&session.id);
    assert!(ended.is_some());
    assert_eq!(manager.active_session_count(), 0);

    // Session should no longer be valid
    let result = manager.validate_session(&session.id);
    assert!(matches!(result, Err(AuthError::SessionNotFound)));
}

#[tokio::test]
async fn test_session_request_limit() {
    use std::time::Duration;

    let config = SessionConfig {
        max_duration: Duration::from_secs(3600),
        idle_timeout: Duration::from_secs(600),
        max_requests: Some(3),
    };

    let manager = SessionManager::new(config);
    let metadata = ClientMetadata {
        client_id: "test".to_string(),
        scopes: vec![],
        ..Default::default()
    };

    let session = manager.create_session(metadata);

    // First 3 requests should succeed
    for _ in 0..3 {
        assert!(manager.validate_session(&session.id).is_ok());
    }

    // 4th request should fail (limit exceeded)
    let result = manager.validate_session(&session.id);
    assert!(matches!(
        result,
        Err(AuthError::SessionRequestLimitExceeded)
    ));
}

#[tokio::test]
async fn test_session_cleanup() {
    use std::time::Duration;
    use tokio::time::sleep;

    let config = SessionConfig {
        max_duration: Duration::from_millis(100),
        idle_timeout: Duration::from_millis(50),
        max_requests: None,
    };

    let manager = SessionManager::new(config);

    // Create a session
    let metadata = ClientMetadata {
        client_id: "test".to_string(),
        scopes: vec![],
        ..Default::default()
    };
    manager.create_session(metadata);

    assert_eq!(manager.active_session_count(), 1);

    // Wait for session to expire
    sleep(Duration::from_millis(150)).await;

    // Cleanup should remove expired session
    let removed = manager.cleanup_expired();
    assert_eq!(removed, 1);
    assert_eq!(manager.active_session_count(), 0);
}

#[test]
fn test_session_id_uniqueness() {
    let id1 = SessionId::new();
    let id2 = SessionId::new();

    assert_ne!(id1, id2);
}

#[test]
fn test_session_id_string_conversion() {
    let id = SessionId::new();
    let id_str = id.as_str();

    let parsed = SessionId::from_str(id_str);
    assert_eq!(id, parsed);
}

#[test]
fn test_client_metadata_serialization() {
    let metadata = ClientMetadata {
        client_id: "test-client".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
        ..Default::default()
    };

    let serialized = serde_json::to_value(&metadata).unwrap();
    let deserialized: ClientMetadata = serde_json::from_value(serialized).unwrap();

    assert_eq!(metadata.client_id, deserialized.client_id);
    assert_eq!(metadata.scopes, deserialized.scopes);
}
