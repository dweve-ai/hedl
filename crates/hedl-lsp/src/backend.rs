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

//! LSP backend implementation.
//!
//! # Performance Optimization
//!
//! This backend implements three key optimizations to reduce CPU usage:
//!
//! 1. **Debouncing**: Parse requests are delayed by 200ms to batch multiple
//!    keystrokes together. This reduces parse operations by ~90% during typing.
//!
//! 2. **Dirty Tracking**: A content hash and dirty flag prevent redundant
//!    parsing when document content hasn't actually changed.
//!
//! 3. **Caching**: Parsed `AnalyzedDocument` is cached and reused for LSP
//!    queries (hover, completion, symbols) without blocking.
//!
//! See OPTIMIZATION.md for detailed performance analysis and trade-offs.

use crate::analysis::AnalyzedDocument;
use crate::completion::get_completions;
use crate::constants::{DEBOUNCE_MS, POSITION_ZERO};
use crate::document_manager::{CacheStatistics, DocumentManager};
use crate::hover::get_hover;
use crate::symbols::{get_document_symbols, get_workspace_symbols};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::{debug, info, warn, error};

/// HEDL Language Server backend.
///
/// This backend handles LSP protocol implementation and delegates document
/// management to the DocumentManager. This provides a clean separation of
/// concerns between protocol handling and document lifecycle management.
pub struct HedlLanguageServer {
    /// LSP client connection.
    client: Client,
    /// Document manager for storage and caching.
    document_manager: Arc<DocumentManager>,
    /// Debounce channels: URI -> sender for triggering analysis.
    debounce_channels: DashMap<Url, mpsc::UnboundedSender<()>>,
}

impl HedlLanguageServer {
    /// Create a new language server with default configuration.
    ///
    /// Default settings:
    /// - Max cache size: 1000 documents
    /// - Max document size: 500 MB
    pub fn new(client: Client) -> Self {
        use crate::constants::{DEFAULT_MAX_CACHE_SIZE, DEFAULT_MAX_DOCUMENT_SIZE};
        Self::with_config(client, DEFAULT_MAX_CACHE_SIZE, DEFAULT_MAX_DOCUMENT_SIZE)
    }

    /// Create a new language server with custom cache size.
    ///
    /// # Deprecated
    ///
    /// Use `with_config()` instead to configure both cache size and document size limit.
    #[deprecated(since = "0.1.0", note = "Use `with_config()` instead")]
    pub fn with_max_cache_size(client: Client, max_cache_size: usize) -> Self {
        use crate::constants::DEFAULT_MAX_DOCUMENT_SIZE;
        Self::with_config(client, max_cache_size, DEFAULT_MAX_DOCUMENT_SIZE)
    }

    /// Create a new language server with custom configuration.
    ///
    /// # Parameters
    ///
    /// - `client`: LSP client connection
    /// - `max_cache_size`: Maximum number of documents to cache (default: 1000)
    /// - `max_document_size`: Maximum document size in bytes (default: 500 MB)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hedl_lsp::HedlLanguageServer;
    /// use tower_lsp::Client;
    ///
    /// fn create_server(client: Client) -> HedlLanguageServer {
    ///     // Allow up to 2000 documents, each up to 1 GB
    ///     HedlLanguageServer::with_config(
    ///         client,
    ///         2000,  // max documents
    ///         1024 * 1024 * 1024  // 1 GB per document
    ///     )
    /// }
    /// ```
    pub fn with_config(
        client: Client,
        max_cache_size: usize,
        max_document_size: usize,
    ) -> Self {
        Self {
            client,
            document_manager: Arc::new(DocumentManager::new(max_cache_size, max_document_size)),
            debounce_channels: DashMap::new(),
        }
    }

    /// Get current cache statistics.
    pub fn cache_statistics(&self) -> CacheStatistics {
        self.document_manager.statistics()
    }

    /// Update maximum cache size (can be called during runtime).
    pub fn set_max_cache_size(&self, new_max: usize) {
        self.document_manager.set_max_cache_size(new_max);
    }

    /// Get current maximum cache size.
    pub fn max_cache_size(&self) -> usize {
        self.document_manager.max_cache_size()
    }

    /// Update maximum document size (can be called during runtime).
    pub fn set_max_document_size(&self, new_max: usize) {
        self.document_manager.set_max_document_size(new_max);
    }

    /// Get current maximum document size.
    pub fn max_document_size(&self) -> usize {
        self.document_manager.max_document_size()
    }

    /// Analyze a document if dirty.
    ///
    /// # Error Handling
    ///
    /// - Missing document: Returns immediately (document may have been closed)
    /// - Analysis errors: Captured and published as diagnostics
    /// - Diagnostic publishing: Logged on success/failure
    async fn analyze_if_dirty(&self, uri: &Url) {
        if !self.document_manager.is_dirty(uri) {
            debug!("Document {} is clean, skipping analysis", uri);
            return;
        }

        let state_arc = match self.document_manager.get_state(uri) {
            Some(state) => state,
            None => {
                warn!("Cannot analyze non-existent document: {} (may have been closed/evicted)", uri);
                return;
            }
        };

        let content = {
            let state = state_arc.lock();
            state.rope.to_string()
        };

        debug!("Starting analysis for dirty document: {} ({} bytes)", uri, content.len());
        let analysis = Arc::new(AnalyzedDocument::analyze(&content));

        // Log analysis results
        if !analysis.errors.is_empty() {
            debug!(
                "Analysis found {} parse errors in {}",
                analysis.errors.len(),
                uri
            );
        }
        if !analysis.lint_diagnostics.is_empty() {
            debug!(
                "Analysis found {} lint diagnostics in {}",
                analysis.lint_diagnostics.len(),
                uri
            );
        }

        // Update state
        self.document_manager.update_analysis(uri, Arc::clone(&analysis));

        // Send diagnostics
        let diagnostics = analysis.to_lsp_diagnostics();
        debug!(
            "Publishing {} diagnostics for {}",
            diagnostics.len(),
            uri
        );
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    /// Start debounced analysis for a document.
    fn schedule_analysis(&self, uri: Url) {
        // Get or create debounce channel
        let tx = if let Some(entry) = self.debounce_channels.get(&uri) {
            entry.clone()
        } else {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let uri_clone = uri.clone();
            let client = self.client.clone();
            let document_manager = Arc::clone(&self.document_manager);

            // Spawn debounce task
            tokio::spawn(async move {
                while rx.recv().await.is_some() {
                    // Wait for debounce period
                    sleep(Duration::from_millis(DEBOUNCE_MS)).await;

                    // Drain any additional signals during debounce
                    while rx.try_recv().is_ok() {}

                    // Perform analysis if dirty
                    if !document_manager.is_dirty(&uri_clone) {
                        continue;
                    }

                    let state_arc = match document_manager.get_state(&uri_clone) {
                        Some(state) => state,
                        None => continue,
                    };

                    let content = {
                        let state = state_arc.lock();
                        state.rope.to_string()
                    };

                    debug!("Debounced analysis for: {}", uri_clone);
                    let analysis = Arc::new(AnalyzedDocument::analyze(&content));

                    // Update state
                    document_manager.update_analysis(&uri_clone, Arc::clone(&analysis));

                    // Send diagnostics
                    let diagnostics = analysis.to_lsp_diagnostics();
                    client
                        .publish_diagnostics(uri_clone.clone(), diagnostics, None)
                        .await;
                }
            });

            self.debounce_channels.insert(uri.clone(), tx.clone());
            tx
        };

        // Trigger analysis
        let _ = tx.send(());
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for HedlLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        info!("HEDL Language Server initializing");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "@".to_string(),
                        ":".to_string(),
                        "%".to_string(),
                        "$".to_string(),
                        "|".to_string(),
                    ]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: Default::default(),
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::OPERATOR,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::DECLARATION,
                                ],
                            },
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "hedl-lsp".to_string(),
                version: Some(crate::VERSION.to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        info!("HEDL Language Server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        info!("HEDL Language Server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = &params.text_document.uri;
        let content_len = params.text_document.text.len();
        let line_count = params.text_document.text.lines().count();

        info!(
            "Document opened: {} ({} bytes, {} lines)",
            uri, content_len, line_count
        );

        // Memory management: Check document size limit
        use crate::constants::BYTES_PER_MEGABYTE;
        let max_size = self.document_manager.max_document_size();
        if content_len > max_size {
            error!(
                "Document size limit exceeded on open: {} has {} bytes > {} bytes maximum",
                uri, content_len, max_size
            );
            self.client
                .show_message(
                    MessageType::ERROR,
                    format!(
                        "Document too large: {} bytes exceeds maximum of {} bytes ({} MB)",
                        content_len,
                        max_size,
                        max_size / BYTES_PER_MEGABYTE
                    ),
                )
                .await;
            return;
        }

        // For did_open, analyze immediately (no debounce) to provide instant feedback
        if self.document_manager.insert_or_update(uri, &params.text_document.text) {
            debug!("Document {} successfully registered, starting immediate analysis", uri);
            self.analyze_if_dirty(uri).await;
        } else {
            error!("Failed to register document {} (size validation failed)", uri);
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri;
        debug!("Document change event received for: {}", uri);

        if let Some(change) = params.content_changes.into_iter().last() {
            debug!(
                "Processing change for {}: {} bytes, {} lines",
                uri,
                change.text.len(),
                change.text.lines().count()
            );

            // Update content and schedule debounced analysis
            if self.document_manager.insert_or_update(uri, &change.text) {
                debug!("Document {} updated, scheduling debounced analysis", uri);
                self.schedule_analysis(uri.clone());
            } else {
                warn!(
                    "Failed to update document {} (size limit exceeded: {} bytes)",
                    uri,
                    change.text.len()
                );
            }
        } else {
            warn!("Document change event for {} had no content changes", uri);
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("Document saved: {}", params.text_document.uri);
        if let Some(text) = params.text {
            // For did_save, analyze immediately to ensure diagnostics are current
            self.document_manager.insert_or_update(&params.text_document.uri, &text);
            self.analyze_if_dirty(&params.text_document.uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        debug!("Document closed: {}", params.text_document.uri);
        self.document_manager.remove(&params.text_document.uri);
        self.debounce_channels.remove(&params.text_document.uri);
        // Clear diagnostics
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        debug!(
            "Completion request for {} at {}:{}",
            uri, position.line, position.character
        );

        if let Some((content, analysis)) = self.document_manager.get(uri) {
            let items = get_completions(analysis.as_ref(), &content, position);
            debug!(
                "Providing {} completion items for {} at {}:{}",
                items.len(),
                uri,
                position.line,
                position.character
            );
            return Ok(Some(CompletionResponse::Array(items)));
        }

        debug!(
            "No completion available for {} (document not found in cache)",
            uri
        );
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some((content, analysis)) = self.document_manager.get(uri) {
            return Ok(get_hover(analysis.as_ref(), &content, position));
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some((_content, analysis)) = self.document_manager.get(uri) {
            // Try using the enhanced reference index v2 first for O(1) precise lookups
            if let Some((ref_str, _loc)) = analysis.reference_index_v2.find_reference_at(position) {
                // Parse the reference to extract type and ID
                let ref_content = ref_str.strip_prefix('@').unwrap_or(ref_str);

                if let Some(colon_pos) = ref_content.find(':') {
                    // Qualified reference: @Type:id
                    let type_name = &ref_content[..colon_pos];
                    let id = &ref_content[colon_pos + 1..];

                    if let Some(def_loc) =
                        analysis.reference_index_v2.find_definition(type_name, id)
                    {
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                            uri: uri.clone(),
                            range: def_loc.to_range(),
                        })));
                    }
                } else {
                    // Unqualified reference: @id - search across all types
                    let id = ref_content;
                    for type_name in analysis.entities.keys() {
                        if let Some(def_loc) =
                            analysis.reference_index_v2.find_definition(type_name, id)
                        {
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: def_loc.to_range(),
                            })));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some((_content, analysis)) = self.document_manager.get(uri) {
            // Use enhanced reference index v2 for O(1) precise lookups
            if let Some((ref_str, _loc)) = analysis.reference_index_v2.find_reference_at(position) {
                let mut locations = Vec::new();

                // Get all references using the v2 index with precise character positions
                let ref_locations = analysis.reference_index_v2.find_references(ref_str);

                for ref_loc in ref_locations {
                    locations.push(Location {
                        uri: uri.clone(),
                        range: ref_loc.to_range(),
                    });
                }

                // If this is a qualified reference, also include the definition if requested
                if params.context.include_declaration {
                    let ref_content = ref_str.strip_prefix('@').unwrap_or(ref_str);

                    if let Some(colon_pos) = ref_content.find(':') {
                        // Qualified reference: @Type:id
                        let type_name = &ref_content[..colon_pos];
                        let id = &ref_content[colon_pos + 1..];

                        if let Some(def_loc) =
                            analysis.reference_index_v2.find_definition(type_name, id)
                        {
                            locations.push(Location {
                                uri: uri.clone(),
                                range: def_loc.to_range(),
                            });
                        }
                    } else {
                        // Unqualified reference: @id - find definition across all types
                        let id = ref_content;
                        for type_name in analysis.entities.keys() {
                            if let Some(def_loc) =
                                analysis.reference_index_v2.find_definition(type_name, id)
                            {
                                locations.push(Location {
                                    uri: uri.clone(),
                                    range: def_loc.to_range(),
                                });
                                break; // Only include the first definition found
                            }
                        }
                    }
                }

                return Ok(Some(locations));
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        if let Some((content, analysis)) = self.document_manager.get(uri) {
            let symbols = get_document_symbols(analysis.as_ref(), &content);
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }

        Ok(None)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let mut all_symbols = Vec::new();

        self.document_manager.for_each(|uri, state_arc| {
            let analysis = {
                let state = state_arc.lock();
                Arc::clone(&state.analysis)
            };

            let mut symbols = get_workspace_symbols(analysis.as_ref(), &params.query);

            // Fix URIs
            for sym in &mut symbols {
                sym.location.uri = uri.clone();
            }

            all_symbols.extend(symbols);
        });

        Ok(Some(all_symbols))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        debug!("Document formatting request for: {}", uri);

        if let Some((content, analysis)) = self.document_manager.get(uri) {
            if let Some(doc) = &analysis.document {
                debug!("Attempting to canonicalize document: {}", uri);

                // Canonicalize the document
                let config = hedl_c14n::CanonicalConfig::default();
                match hedl_c14n::canonicalize_with_config(doc, &config) {
                    Ok(formatted) => {
                        if formatted != content {
                            let line_count = content.lines().count();
                            let formatted_lines = formatted.lines().count();
                            debug!(
                                "Formatting {} changes {} lines to {} lines",
                                uri, line_count, formatted_lines
                            );
                            return Ok(Some(vec![TextEdit {
                                range: Range {
                                    start: Position {
                                        line: POSITION_ZERO,
                                        character: POSITION_ZERO,
                                    },
                                    end: Position {
                                        line: line_count as u32,
                                        character: POSITION_ZERO,
                                    },
                                },
                                new_text: formatted,
                            }]));
                        } else {
                            debug!("Document {} is already formatted correctly", uri);
                        }
                    }
                    Err(e) => {
                        error!(
                            "Canonicalization failed for {}: {} (line {})",
                            uri, e.message, e.line
                        );
                    }
                }
            } else {
                warn!(
                    "Cannot format {}: document failed to parse (has {} errors)",
                    uri,
                    analysis.errors.len()
                );
            }
        } else {
            warn!("Cannot format non-existent document: {}", uri);
        }

        Ok(None)
    }
}
