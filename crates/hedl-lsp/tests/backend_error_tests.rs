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

//! Backend LSP protocol error path tests.
//!
//! This module specifically targets the backend.rs error paths to increase
//! coverage from 36% to 70%+. It tests LSP protocol edge cases, error handling,
//! and resource management.

use hedl_lsp::HedlLanguageServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, LanguageServer};

// Helper macro to create test server inline
macro_rules! test_server {
    () => {{
        let (service, _socket) = LspService::new(|client| HedlLanguageServer::new(client));
        service
    }};
    ($max_cache:expr, $max_doc_size:expr) => {{
        let (service, _socket) = LspService::new(move |client| {
            HedlLanguageServer::with_config(client, $max_cache, $max_doc_size)
        });
        service
    }};
}

// ============================================================================
// Initialize/Shutdown Tests
// ============================================================================

#[tokio::test]
async fn test_initialize_capabilities() {
    let (service, _socket) = LspService::new(|client| HedlLanguageServer::new(client));
    let server = service.inner();
    let params = InitializeParams::default();

    let result = server.initialize(params).await;
    assert!(result.is_ok());

    let init_result = result.unwrap();
    assert!(init_result.capabilities.text_document_sync.is_some());
    assert!(init_result.capabilities.completion_provider.is_some());
    assert!(init_result.capabilities.hover_provider.is_some());
    assert!(init_result.capabilities.definition_provider.is_some());
    assert!(init_result.capabilities.references_provider.is_some());
    assert!(init_result.capabilities.document_symbol_provider.is_some());
    assert!(init_result.capabilities.workspace_symbol_provider.is_some());
}

#[tokio::test]
async fn test_initialized_notification() {
    let service = test_server!(); let server = service.inner();
    let params = InitializedParams {};

    // Should not panic
    server.initialized(params).await;
}

#[tokio::test]
async fn test_shutdown() {
    let service = test_server!(); let server = service.inner();

    let result = server.shutdown().await;
    assert!(result.is_ok());
}

// ============================================================================
// Document Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_did_open_oversized_document() {
    let service = test_server!(10, 100); let server = service.inner(); // Small size limit

    let uri = Url::parse("file:///test.hedl").unwrap();
    let large_content = "x".repeat(200); // Exceeds limit

    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: large_content,
        },
    };

    // Should handle gracefully (show message to client, but not panic)
    server.did_open(params).await;

    // Document should not be in cache
    assert!(server.cache_statistics().current_size == 0);
}

#[tokio::test]
async fn test_did_open_valid_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\n".to_string(),
        },
    };

    server.did_open(params).await;

    // Document should be in cache
    assert_eq!(server.cache_statistics().current_size, 1);
}

#[tokio::test]
async fn test_did_change_nonexistent_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///nonexistent.hedl").unwrap();

    let params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "new content".to_string(),
        }],
    };

    // Should handle gracefully (creates new document)
    server.did_change(params).await;
}

#[tokio::test]
async fn test_did_change_empty_changes() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document first
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\n".to_string(),
        },
    }).await;

    let params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 2,
        },
        content_changes: vec![], // Empty changes
    };

    // Should handle gracefully
    server.did_change(params).await;
}

#[tokio::test]
async fn test_did_save_with_text() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    let params = DidSaveTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        text: Some("%VERSION: 1.0\n---\nUser: u1: \"Alice\"".to_string()),
    };

    server.did_save(params).await;

    // Document should be in cache
    assert_eq!(server.cache_statistics().current_size, 1);
}

#[tokio::test]
async fn test_did_save_without_text() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    let params = DidSaveTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        text: None, // No text provided
    };

    // Should handle gracefully
    server.did_save(params).await;
}

#[tokio::test]
async fn test_did_close() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\n".to_string(),
        },
    }).await;

    assert_eq!(server.cache_statistics().current_size, 1);

    // Close document
    server.did_close(DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
    }).await;

    // Document should be removed
    assert_eq!(server.cache_statistics().current_size, 0);
}

// ============================================================================
// Completion Tests
// ============================================================================

#[tokio::test]
async fn test_completion_nonexistent_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///nonexistent.hedl").unwrap();

    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 0, character: 0 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: None,
    };

    let result = server.completion(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_completion_valid_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n%STRUCT: User: [id]\n---\n".to_string(),
        },
    }).await;

    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 0, character: 0 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: None,
    };

    let result = server.completion(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

// ============================================================================
// Hover Tests
// ============================================================================

#[tokio::test]
async fn test_hover_nonexistent_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///nonexistent.hedl").unwrap();

    let params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 0, character: 0 },
        },
        work_done_progress_params: Default::default(),
    };

    let result = server.hover(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_hover_valid_position() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\n".to_string(),
        },
    }).await;

    let params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 0, character: 5 },
        },
        work_done_progress_params: Default::default(),
    };

    let result = server.hover(params).await;
    assert!(result.is_ok());
}

// ============================================================================
// Go to Definition Tests
// ============================================================================

#[tokio::test]
async fn test_goto_definition_nonexistent_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///nonexistent.hedl").unwrap();

    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 0, character: 0 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let result = server.goto_definition(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_goto_definition_no_reference_at_position() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\n".to_string(),
        },
    }).await;

    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 1, character: 0 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let result = server.goto_definition(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_goto_definition_qualified_reference() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document with qualified reference
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n%STRUCT: User: [id]\n---\nUser: u1: \"Alice\"\nPost: p1: @User:u1".to_string(),
        },
    }).await;

    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 4, character: 12 }, // On @User:u1
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let result = server.goto_definition(params).await;
    assert!(result.is_ok());
    // May or may not find definition depending on exact position
}

#[tokio::test]
async fn test_goto_definition_unqualified_reference() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document with unqualified reference
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n%STRUCT: User: [id]\n---\nUser: u1: \"Alice\"\nPost: p1: @u1".to_string(),
        },
    }).await;

    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 4, character: 11 }, // On @u1
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let result = server.goto_definition(params).await;
    assert!(result.is_ok());
}

// ============================================================================
// References Tests
// ============================================================================

#[tokio::test]
async fn test_references_nonexistent_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///nonexistent.hedl").unwrap();

    let params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 0, character: 0 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: ReferenceContext {
            include_declaration: false,
        },
    };

    let result = server.references(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_references_with_declaration() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n%STRUCT: User: [id]\n---\nUser: u1: \"Alice\"\nPost: p1: @User:u1\nPost: p2: @User:u1".to_string(),
        },
    }).await;

    let params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 4, character: 12 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: ReferenceContext {
            include_declaration: true, // Include definition
        },
    };

    let result = server.references(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_references_without_declaration() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n%STRUCT: User: [id]\n---\nUser: u1: \"Alice\"\nPost: p1: @User:u1".to_string(),
        },
    }).await;

    let params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 4, character: 12 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: ReferenceContext {
            include_declaration: false, // Exclude definition
        },
    };

    let result = server.references(params).await;
    assert!(result.is_ok());
}

// ============================================================================
// Document Symbols Tests
// ============================================================================

#[tokio::test]
async fn test_document_symbol_nonexistent() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///nonexistent.hedl").unwrap();

    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let result = server.document_symbol(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_document_symbol_valid() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n%STRUCT: User: [id]\n---\nUser: u1: \"Alice\"".to_string(),
        },
    }).await;

    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let result = server.document_symbol(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

// ============================================================================
// Workspace Symbols Tests
// ============================================================================

#[tokio::test]
async fn test_workspace_symbol_empty_workspace() {
    let service = test_server!(); let server = service.inner();

    let params = WorkspaceSymbolParams {
        query: "test".to_string(),
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let result = server.symbol(params).await;
    assert!(result.is_ok());
    let symbols = result.unwrap().unwrap();
    assert_eq!(symbols.len(), 0);
}

#[tokio::test]
async fn test_workspace_symbol_with_documents() {
    let service = test_server!(); let server = service.inner();

    // Open multiple documents
    for i in 0..3 {
        let uri = Url::parse(&format!("file:///test{}.hedl", i)).unwrap();
        server.did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "hedl".to_string(),
                version: 1,
                text: format!("%VERSION: 1.0\n---\nEntity{}: e{}: \"test\"", i, i),
            },
        }).await;
    }

    let params = WorkspaceSymbolParams {
        query: "Entity".to_string(),
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let result = server.symbol(params).await;
    assert!(result.is_ok());
    let _symbols = result.unwrap().unwrap();
    // Symbols might be empty depending on when analysis completes
}

// ============================================================================
// Formatting Tests
// ============================================================================

#[tokio::test]
async fn test_formatting_nonexistent_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///nonexistent.hedl").unwrap();

    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        options: FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            ..Default::default()
        },
        work_done_progress_params: Default::default(),
    };

    let result = server.formatting(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_formatting_valid_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\nUser: u1:    \"Alice\"   ".to_string(),
        },
    }).await;

    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        options: FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            ..Default::default()
        },
        work_done_progress_params: Default::default(),
    };

    let result = server.formatting(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_formatting_invalid_document() {
    let service = test_server!(); let server = service.inner();
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Open document with parse errors
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "@@@invalid content@@@".to_string(),
        },
    }).await;

    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        options: FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            ..Default::default()
        },
        work_done_progress_params: Default::default(),
    };

    let result = server.formatting(params).await;
    assert!(result.is_ok());
    // Should return None for invalid document
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_cache_size_configuration() {
    let service = test_server!(5, 1024 * 1024); let server = service.inner();

    assert_eq!(server.max_cache_size(), 5);

    // Runtime update
    server.set_max_cache_size(10);
    assert_eq!(server.max_cache_size(), 10);
}

#[tokio::test]
async fn test_document_size_configuration() {
    let service = test_server!(100, 1000); let server = service.inner();

    assert_eq!(server.max_document_size(), 1000);

    // Runtime update
    server.set_max_document_size(2000);
    assert_eq!(server.max_document_size(), 2000);
}

#[tokio::test]
async fn test_cache_statistics() {
    let service = test_server!(); let server = service.inner();

    let stats = server.cache_statistics();
    assert_eq!(stats.current_size, 0);
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);

    // Open a document
    let uri = Url::parse("file:///test.hedl").unwrap();
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\n".to_string(),
        },
    }).await;

    let stats = server.cache_statistics();
    assert_eq!(stats.current_size, 1);
}
