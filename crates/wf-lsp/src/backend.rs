//! The server: holds the open documents, assembles the project a request
//! concerns, and answers.
//!
//! Every request rebuilds the project from the open buffers and the disk
//! cache — a few milliseconds for a project of dozens of files, and the only
//! way an edit in one file is seen at once by every other file's diagnostics.
//! Diagnostics are pushed: a change to any file re-reports every open file of
//! the same project, since a renamed component changes what its callers say.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use webfluent::parser::Program;

use crate::code_actions::{extract_component_action, provide_code_actions};
use crate::completion::provide_completions;
use crate::definition::find_definition;
use crate::diagnostics::project_diagnostics;
use crate::hover::provide_hover;
use crate::project::{FileCache, OpenText, Project};
use crate::rename::{prepare_rename, rename};
use crate::symbols::{document_symbols, workspace_symbols};

/// An open editor buffer.
struct OpenDocument {
    text: Arc<str>,
    /// The last parse of this buffer that succeeded.
    last_valid: Option<Arc<Program>>,
}

pub struct Backend {
    client: Client,
    documents: DashMap<PathBuf, OpenDocument>,
    cache: FileCache,
    hierarchical_symbols: AtomicBool,
    /// Whether the client lets the server ask to hear of file changes.
    watch_files: AtomicBool,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            cache: FileCache::default(),
            hierarchical_symbols: AtomicBool::new(true),
            watch_files: AtomicBool::new(false),
        }
    }

    fn path_of(uri: &Url) -> PathBuf {
        uri.to_file_path()
            .unwrap_or_else(|_| PathBuf::from(uri.path()))
    }

    /// The project `uri` belongs to, with every open buffer's text.
    fn project(&self, uri: &Url) -> Project {
        let open = |path: &Path| -> Option<OpenText> {
            let doc = self.documents.get(path)?;
            Some(OpenText {
                text: doc.text.clone(),
                last_valid: doc.last_valid.clone(),
            })
        };
        Project::load(uri, &open, &self.cache)
    }

    /// The project and the index of `uri` in it.
    fn locate(&self, uri: &Url) -> Option<(Project, usize)> {
        let project = self.project(uri);
        let ix = project.file_index(uri)?;
        Some((project, ix))
    }

    /// Records the buffer's text and re-reports the project's open files.
    async fn changed(&self, uri: Url, text: String) {
        let path = Self::path_of(&uri);
        let text: Arc<str> = text.into();
        let previous = self
            .documents
            .get(&path)
            .and_then(|doc| doc.last_valid.clone());
        self.documents.insert(
            path.clone(),
            OpenDocument {
                text: text.clone(),
                last_valid: previous,
            },
        );

        let project = self.project(&uri);

        // Remember the parse that succeeded, for the next edit that does not.
        if let Some(file) = project.file(&uri)
            && let Ok(program) = &file.parsed
            && let Some(mut doc) = self.documents.get_mut(&path)
        {
            doc.last_valid = Some(Arc::new(program.clone()));
        }

        self.publish(&project).await;
    }

    async fn publish(&self, project: &Project) {
        let per_file = project_diagnostics(project);
        for (file, diagnostics) in project.files.iter().zip(per_file) {
            if file.open {
                self.client
                    .publish_diagnostics(file.uri.clone(), diagnostics, None)
                    .await;
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let hierarchical = params
            .capabilities
            .text_document
            .as_ref()
            .and_then(|t| t.document_symbol.as_ref())
            .and_then(|s| s.hierarchical_document_symbol_support)
            .unwrap_or(false);
        self.hierarchical_symbols
            .store(hierarchical, Ordering::Relaxed);
        let watch = params
            .capabilities
            .workspace
            .as_ref()
            .and_then(|w| w.did_change_watched_files.as_ref())
            .and_then(|d| d.dynamic_registration)
            .unwrap_or(false);
        self.watch_files.store(watch, Ordering::Relaxed);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(false),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    // `.` for members and sub-components, `:` for `on:`; a
                    // space is not a trigger — it opened the menu after every
                    // word.
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        ":".to_string(),
                        "(".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![
                            CodeActionKind::QUICKFIX,
                            CodeActionKind::REFACTOR_EXTRACT,
                        ]),
                        ..Default::default()
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
                format!("WebFluent language server v{}", env!("CARGO_PKG_VERSION")),
            )
            .await;
        // Every editor, not only the one whose extension says so, tells the
        // server when a file a finding depends on changes on disk: a source,
        // a Markdown page, a script, a stylesheet, the config (`lints`,
        // `env`, `i18n`), a translation, a `.env`, or a file in `public/`
        // a page names.
        if self.watch_files.load(Ordering::Relaxed) {
            let watcher = |glob: &str| FileSystemWatcher {
                glob_pattern: GlobPattern::String(glob.to_string()),
                kind: None,
            };
            let options = DidChangeWatchedFilesRegistrationOptions {
                watchers: vec![
                    watcher("**/*.{wf,wfx,md,js,css,json}"),
                    watcher("**/.env"),
                    watcher("**/public/**"),
                ],
            };
            let _ = self
                .client
                .register_capability(vec![Registration {
                    id: "webfluent-files".to_string(),
                    method: "workspace/didChangeWatchedFiles".to_string(),
                    register_options: serde_json::to_value(options).ok(),
                }])
                .await;
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.changed(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            self.changed(params.text_document.uri, change.text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // The disk now agrees with the buffer; other files of the project
        // that read it from the disk are re-reported.
        let project = self.project(&params.text_document.uri);
        self.publish(&project).await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        // A script, a stylesheet or a closed `.wf` file changed on disk: what
        // the open files say may have changed with it — a call to a function
        // a script no longer declares, a class a sheet no longer defines.
        // Each open file's project is reported once.
        let open: Vec<PathBuf> = self.documents.iter().map(|d| d.key().clone()).collect();
        let mut done: Vec<Option<PathBuf>> = Vec::new();
        for path in open {
            let Ok(uri) = Url::from_file_path(&path) else {
                continue;
            };
            let project = self.project(&uri);
            if done.contains(&project.root) && project.root.is_some() {
                continue;
            }
            done.push(project.root.clone());
            self.publish(&project).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&Self::path_of(&uri));
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let items = match self.locate(&uri) {
            Some((project, ix)) => {
                provide_completions(&project, ix, params.text_document_position.position)
            }
            None => Vec::new(),
        };
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        Ok(self.locate(&uri).and_then(|(project, ix)| {
            provide_hover(&project, ix, params.text_document_position_params.position)
        }))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let hierarchical = self.hierarchical_symbols.load(Ordering::Relaxed);
        Ok(self
            .locate(&params.text_document.uri)
            .map(|(project, ix)| document_symbols(&project, ix, hierarchical)))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        // The project of any open document; they are all the same project
        // in the usual case of one window per project.
        let Some(uri) = self
            .documents
            .iter()
            .next()
            .and_then(|entry| Url::from_file_path(entry.key()).ok())
        else {
            return Ok(Some(Vec::new()));
        };
        let project = self.project(&uri);
        Ok(Some(workspace_symbols(&project, &params.query)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        Ok(self.locate(&uri).and_then(|(project, ix)| {
            find_definition(&project, ix, params.text_document_position_params.position)
        }))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        Ok(self
            .locate(&params.text_document.uri)
            .and_then(|(project, ix)| prepare_rename(&project, ix, params.position)))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let position = params.text_document_position.position;
        let Some((project, ix)) = self.locate(&params.text_document_position.text_document.uri)
        else {
            return Ok(None);
        };
        match rename(&project, ix, position, &params.new_name) {
            Ok(edit) => Ok(Some(edit)),
            Err(message) => Err(tower_lsp::jsonrpc::Error {
                code: tower_lsp::jsonrpc::ErrorCode::InvalidRequest,
                message: message.into(),
                data: None,
            }),
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.clone();
        let mut actions = provide_code_actions(&uri, params.clone());
        if let Some((project, ix)) = self.locate(&uri) {
            actions.extend(extract_component_action(&project, ix, params.range));
        }
        Ok(Some(actions))
    }
}
