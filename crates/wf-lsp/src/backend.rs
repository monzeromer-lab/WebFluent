use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use webfluent::lexer::Lexer;
use webfluent::linter::{lint_accessibility, lint_contrast, lint_vocabulary, validate_semantics};
use webfluent::parser::{Parser, Program};
use webfluent::themes::resolve_tokens;

use crate::code_actions::provide_code_actions;
use crate::completion::provide_completions;
use crate::definition::find_definition;
use crate::diagnostics::publish_all_diagnostics;
use crate::hover::provide_hover;
use crate::line_index::LineIndex;
use crate::symbols::build_document_symbols;

/// Per-document cached state.
pub struct DocumentState {
    pub source: String,
    pub index: LineIndex,
    pub program: Option<Program>,
    /// Last successfully parsed AST snapshot, keeping hover/completions alive during typing.
    pub last_valid_program: Option<Program>,
}

/// The WebFluent LSP backend.
pub struct Backend {
    pub client: Client,
    pub documents: DashMap<String, DocumentState>,
    pub supports_hierarchical_symbols: Arc<AtomicBool>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            supports_hierarchical_symbols: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Parse a document and cache results; run all compiler linters and publish diagnostics.
    async fn on_change(&self, uri: Url, text: String) {
        let uri_str = uri.to_string();
        let file = uri.path().to_string();
        let index = LineIndex::new(&text);

        let mut lexer = Lexer::new(&text, &file);
        let tokens = lexer.tokenize();

        match tokens {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens, &file);
                match parser.parse() {
                    Ok(program) => {
                        let semantic_diagnostics = validate_semantics(&program, &file);
                        let vocab_warnings = lint_vocabulary(&program, &file);
                        let a11y_warnings = lint_accessibility(&program);
                        let resolved = resolve_tokens(&program, &Default::default()).unwrap_or_default();
                        let contrast_warnings = lint_contrast(&program, &resolved);

                        self.documents.insert(
                            uri_str.clone(),
                            DocumentState {
                                source: text.clone(),
                                index: index.clone(),
                                program: Some(program.clone()),
                                last_valid_program: Some(program),
                            },
                        );

                        publish_all_diagnostics(
                            &self.client,
                            &uri,
                            &text,
                            &index,
                            &[],
                            &semantic_diagnostics,
                            &a11y_warnings,
                            &vocab_warnings,
                            &contrast_warnings,
                        )
                        .await;
                    }
                    Err(e) => {
                        let previous_valid = self
                            .documents
                            .get(&uri_str)
                            .and_then(|doc| doc.last_valid_program.clone());

                        self.documents.insert(
                            uri_str.clone(),
                            DocumentState {
                                source: text.clone(),
                                index: index.clone(),
                                program: None,
                                last_valid_program: previous_valid,
                            },
                        );

                        publish_all_diagnostics(
                            &self.client,
                            &uri,
                            &text,
                            &index,
                            &[e],
                            &[],
                            &[],
                            &[],
                            &[],
                        )
                        .await;
                    }
                }
            }
            Err(e) => {
                let previous_valid = self
                    .documents
                    .get(&uri_str)
                    .and_then(|doc| doc.last_valid_program.clone());

                self.documents.insert(
                    uri_str.clone(),
                    DocumentState {
                        source: text.clone(),
                        index: index.clone(),
                        program: None,
                        last_valid_program: previous_valid,
                    },
                );

                publish_all_diagnostics(
                    &self.client,
                    &uri,
                    &text,
                    &index,
                    &[e],
                    &[],
                    &[],
                    &[],
                    &[],
                )
                .await;
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(text_doc) = &params.capabilities.text_document {
            if let Some(doc_sym) = &text_doc.document_symbol {
                let hierarchical = doc_sym.hierarchical_document_symbol_support.unwrap_or(false);
                self.supports_hierarchical_symbols
                    .store(hierarchical, Ordering::Relaxed);
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        ":".to_string(),
                        "(".to_string(),
                        " ".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("webfluent".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "wf-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("WebFluent LSP v{} initialized", env!("CARGO_PKG_VERSION")),
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.on_change(uri, text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            self.on_change(uri, change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri_str = params.text_document.uri.to_string();
        self.documents.remove(&uri_str);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri_str = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let items = if let Some(doc) = self.documents.get(&uri_str) {
            let prog = doc
                .program
                .as_ref()
                .or(doc.last_valid_program.as_ref());
            provide_completions(&doc.source, position, prog)
        } else {
            vec![]
        };

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri_str = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri_str) {
            let prog = doc
                .program
                .as_ref()
                .or(doc.last_valid_program.as_ref());
            Ok(provide_hover(&doc.source, position, prog))
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let uri_str = uri.to_string();

        if let Some(doc) = self.documents.get(&uri_str) {
            if let Some(program) = doc.program.as_ref().or(doc.last_valid_program.as_ref()) {
                let hierarchical = self.supports_hierarchical_symbols.load(Ordering::Relaxed);
                return Ok(Some(build_document_symbols(
                    program,
                    &doc.source,
                    &doc.index,
                    &uri,
                    hierarchical,
                )));
            }
        }

        Ok(Some(DocumentSymbolResponse::Flat(vec![])))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let uri_str = uri.to_string();
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri_str) {
            if let Some(program) = doc.program.as_ref().or(doc.last_valid_program.as_ref()) {
                return Ok(find_definition(program, &doc.source, &doc.index, &uri, position));
            }
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.clone();
        let actions = provide_code_actions(&uri, params);
        Ok(Some(actions))
    }
}
