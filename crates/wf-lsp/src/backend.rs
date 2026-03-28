use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use webfluent::lexer::Lexer;
use webfluent::linter::lint_accessibility;
use webfluent::parser::{Declaration, Program, Statement, Parser};

use crate::completion::provide_completions;
use crate::diagnostics::publish_diagnostics;
use crate::hover::provide_hover;

/// Per-document cached state.
pub struct DocumentState {
    pub source: String,
    pub program: Option<Program>,
}

/// The WebFluent LSP backend.
pub struct Backend {
    pub client: Client,
    pub documents: DashMap<String, DocumentState>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    /// Parse a document and cache results; publish diagnostics.
    async fn on_change(&self, uri: Url, text: String) {
        let uri_str = uri.to_string();
        let file = uri.path().to_string();

        let mut lexer = Lexer::new(&text, &file);
        let tokens = lexer.tokenize();

        match tokens {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens, &file);
                match parser.parse() {
                    Ok(program) => {
                        let a11y_warnings = lint_accessibility(&program);
                        self.documents.insert(
                            uri_str.clone(),
                            DocumentState {
                                source: text,
                                program: Some(program),
                            },
                        );
                        publish_diagnostics(&self.client, &uri, &[], &a11y_warnings).await;
                    }
                    Err(e) => {
                        self.documents.insert(
                            uri_str.clone(),
                            DocumentState {
                                source: text,
                                program: None,
                            },
                        );
                        publish_diagnostics(&self.client, &uri, &[e], &[]).await;
                    }
                }
            }
            Err(e) => {
                self.documents.insert(
                    uri_str.clone(),
                    DocumentState {
                        source: text,
                        program: None,
                    },
                );
                publish_diagnostics(&self.client, &uri, &[e], &[]).await;
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
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
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "WebFluent LSP server initialized")
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

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri_str = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let items = if let Some(doc) = self.documents.get(&uri_str) {
            provide_completions(&doc.source, position)
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
            Ok(provide_hover(&doc.source, position))
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri_str = params.text_document.uri.to_string();

        let symbols = if let Some(doc) = self.documents.get(&uri_str) {
            if let Some(program) = &doc.program {
                build_document_symbols(program)
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }
}

/// Build document symbols from a parsed program.
fn build_document_symbols(program: &Program) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    for decl in &program.declarations {
        match decl {
            Declaration::Page(page) => {
                symbols.push(make_symbol(
                    &page.name,
                    SymbolKind::CLASS,
                    "Page",
                ));
                collect_body_symbols(&page.body, &mut symbols);
            }
            Declaration::Component(comp) => {
                symbols.push(make_symbol(
                    &comp.name,
                    SymbolKind::CLASS,
                    "Component",
                ));
                collect_body_symbols(&comp.body, &mut symbols);
            }
            Declaration::Store(store) => {
                symbols.push(make_symbol(
                    &store.name,
                    SymbolKind::MODULE,
                    "Store",
                ));
                collect_body_symbols(&store.body, &mut symbols);
            }
            Declaration::App(_app) => {
                symbols.push(make_symbol("App", SymbolKind::CLASS, "App"));
            }
        }
    }

    symbols
}

fn collect_body_symbols(stmts: &[Statement], symbols: &mut Vec<SymbolInformation>) {
    for stmt in stmts {
        match stmt {
            Statement::State(s) => {
                symbols.push(make_symbol(
                    &s.name,
                    SymbolKind::VARIABLE,
                    "state",
                ));
            }
            Statement::Derived(d) => {
                symbols.push(make_symbol(
                    &d.name,
                    SymbolKind::VARIABLE,
                    "derived",
                ));
            }
            Statement::Action(a) => {
                symbols.push(make_symbol(
                    &a.name,
                    SymbolKind::FUNCTION,
                    "action",
                ));
            }
            _ => {}
        }
    }
}

#[allow(deprecated)]
fn make_symbol(name: &str, kind: SymbolKind, container: &str) -> SymbolInformation {
    SymbolInformation {
        name: name.to_string(),
        kind,
        tags: None,
        deprecated: None,
        location: Location {
            uri: Url::parse("file:///unknown").unwrap(),
            range: Range::default(),
        },
        container_name: Some(container.to_string()),
    }
}
