//! The project a document belongs to, assembled the way `wf build` assembles it.
//!
//! A WebFluent program is every `.wf` file under a project's `src/`, merged
//! into one declaration list: `App.wf` routes to pages declared in
//! `pages/`, which call components declared in `components/` and read stores
//! declared in `stores/`. A server that looked at one file at a time reported
//! every one of those references as unknown. This module finds the project
//! root (the directory holding `webfluent.app.json`), reads every source file
//! — the editor's unsaved text for open buffers, the disk for the rest — and
//! merges them in the compiler's order, remembering which file each
//! declaration came from so a diagnostic or a definition can be routed back.
//!
//! A `.wf` file outside any project's `src/` (a template, a scratch file) is a
//! project of one.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use dashmap::DashMap;
use tower_lsp::lsp_types::Url;
use webfluent::config::project::{ProjectConfig, ThemeConfig};
use webfluent::error::WebFluentError;
use webfluent::lexer::Lexer;
use webfluent::parser::{Declaration, Parser, Program};

use crate::line_index::LineIndex;

/// The editor's text for an open file, and the last parse of it that
/// succeeded — what the server reads from while the text mid-edit does not
/// parse. Its spans are a little stale by then; its names are not.
pub struct OpenText {
    pub text: Arc<str>,
    pub last_valid: Option<Arc<Program>>,
}

/// One source file, parsed.
pub struct SourceFile {
    pub uri: Url,
    pub path: PathBuf,
    pub source: Arc<str>,
    pub index: LineIndex,
    /// The file's own parse of its current text.
    pub parsed: Result<Program, WebFluentError>,
    /// Whether the text came from an open editor buffer rather than the disk.
    pub open: bool,
    /// Whether this file's declarations in the merged program come from an
    /// earlier parse, because the current text does not parse.
    pub stale: bool,
}

/// A project: its files, and their declarations merged into one program.
pub struct Project {
    /// The directory holding `webfluent.app.json`, when there is one.
    pub root: Option<PathBuf>,
    pub theme: ThemeConfig,
    pub files: Vec<SourceFile>,
    /// Every file's declarations, in build order.
    pub program: Program,
    /// The index into `files` of the file each declaration in `program` came
    /// from, parallel to `program.declarations`.
    pub decl_file: Vec<usize>,
}

/// Parsed disk files, keyed by path, reused while the file is unchanged.
#[derive(Default)]
pub struct FileCache {
    entries: DashMap<PathBuf, Arc<CachedFile>>,
}

struct CachedFile {
    modified: Option<SystemTime>,
    len: u64,
    source: Arc<str>,
}

impl FileCache {
    fn read(&self, path: &Path) -> Option<Arc<str>> {
        let meta = fs::metadata(path).ok()?;
        let modified = meta.modified().ok();
        let len = meta.len();
        if let Some(hit) = self.entries.get(path)
            && hit.modified == modified
            && hit.len == len
        {
            return Some(hit.source.clone());
        }
        let source: Arc<str> = fs::read_to_string(path).ok()?.into();
        self.entries.insert(
            path.to_path_buf(),
            Arc::new(CachedFile {
                modified,
                len,
                source: source.clone(),
            }),
        );
        Some(source)
    }
}

impl Project {
    /// The project containing `uri`. `open_text` supplies the editor's text
    /// for files that are open, which takes precedence over the disk.
    pub fn load(
        uri: &Url,
        open_text: &dyn Fn(&Path) -> Option<OpenText>,
        cache: &FileCache,
    ) -> Project {
        let path = uri
            .to_file_path()
            .unwrap_or_else(|_| PathBuf::from(uri.path()));

        let root = find_root(&path);
        let (theme, paths) = match &root {
            Some(root) if path.starts_with(root.join("src")) => {
                let theme = ProjectConfig::load(root)
                    .map(|config| config.theme)
                    .unwrap_or_default();
                let mut paths = source_files(&root.join("src"));
                if !paths.iter().any(|p| p == &path) {
                    paths.push(path.clone());
                }
                (theme, paths)
            }
            _ => (ThemeConfig::default(), vec![path.clone()]),
        };

        let mut files = Vec::new();
        let mut fallbacks: Vec<Option<Arc<Program>>> = Vec::new();
        for file_path in paths.into_iter() {
            let (source, open, last_valid): (Arc<str>, bool, Option<Arc<Program>>) =
                match open_text(&file_path) {
                    Some(open) => (open.text, true, open.last_valid),
                    None => match cache.read(&file_path) {
                        Some(text) => (text, false, None),
                        None => continue,
                    },
                };
            let file_uri = Url::from_file_path(&file_path).unwrap_or_else(|_| uri.clone());
            let label = root
                .as_ref()
                .and_then(|root| file_path.strip_prefix(root).ok())
                .unwrap_or(&file_path)
                .to_string_lossy()
                .to_string();
            let parsed = parse(&source, &label);
            let stale = parsed.is_err() && last_valid.is_some();
            fallbacks.push(last_valid);
            files.push(SourceFile {
                uri: file_uri,
                path: file_path,
                index: LineIndex::new(&source),
                source,
                parsed,
                open,
                stale,
            });
        }

        let mut declarations = Vec::new();
        let mut decl_file = Vec::new();
        for (file_ix, file) in files.iter().enumerate() {
            let program: Option<&Program> = match &file.parsed {
                Ok(program) => Some(program),
                Err(_) => fallbacks[file_ix].as_deref(),
            };
            if let Some(program) = program {
                for decl in &program.declarations {
                    declarations.push(decl.clone());
                    decl_file.push(file_ix);
                }
            }
        }

        Project {
            root,
            theme,
            files,
            program: Program { declarations },
            decl_file,
        }
    }

    /// A project of one in-memory file — what tests build, and what a
    /// document with no path on disk gets.
    pub fn single(uri: Url, source: &str) -> Project {
        let source: Arc<str> = source.into();
        let parsed = parse(&source, "test.wf");
        let file = SourceFile {
            uri,
            path: PathBuf::new(),
            index: LineIndex::new(&source),
            source,
            parsed,
            open: true,
            stale: false,
        };
        let declarations = file
            .parsed
            .as_ref()
            .map(|p| p.declarations.clone())
            .unwrap_or_default();
        let decl_file = vec![0; declarations.len()];
        Project {
            root: None,
            theme: ThemeConfig::default(),
            files: vec![file],
            program: Program { declarations },
            decl_file,
        }
    }

    /// The file `uri` names, if it is part of this project.
    pub fn file_index(&self, uri: &Url) -> Option<usize> {
        self.files.iter().position(|file| &file.uri == uri)
    }

    pub fn file(&self, uri: &Url) -> Option<&SourceFile> {
        self.file_index(uri).map(|ix| &self.files[ix])
    }

    /// The merged program's declarations that came from `file_ix`, with their
    /// indices into `program.declarations`.
    pub fn declarations_of(&self, file_ix: usize) -> impl Iterator<Item = (usize, &Declaration)> {
        self.decl_file
            .iter()
            .enumerate()
            .filter(move |(_, f)| **f == file_ix)
            .map(move |(i, _)| (i, &self.program.declarations[i]))
    }

    /// The file a merged declaration lives in.
    pub fn file_of_declaration(&self, decl_ix: usize) -> &SourceFile {
        &self.files[self.decl_file[decl_ix]]
    }

    /// The label the compiler would print for a file: its path relative to
    /// the project root.
    pub fn label_of(&self, file_ix: usize) -> String {
        let path = &self.files[file_ix].path;
        self.root
            .as_ref()
            .and_then(|root| path.strip_prefix(root).ok())
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }
}

fn parse(source: &str, label: &str) -> Result<Program, WebFluentError> {
    let tokens = Lexer::new(source, label).tokenize()?;
    Parser::new(tokens, label).parse()
}

/// The nearest ancestor of `path` that holds a `webfluent.app.json`.
fn find_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .skip(1)
        .find(|dir| dir.join("webfluent.app.json").is_file())
        .map(Path::to_path_buf)
}

/// Every `.wf` under `dir`, in the order `wf build` reads them: `App.wf`
/// first, then the rest depth-first, alphabetically.
fn source_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let app = dir.join("App.wf");
    if app.is_file() {
        files.push(app);
    }
    walk(dir, dir, &mut files);
    files
}

fn walk(dir: &Path, top: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            walk(&path, top, files);
        } else if path.extension().is_some_and(|ext| ext == "wf") {
            if dir == top && path.file_name().is_some_and(|n| n == "App.wf") {
                continue;
            }
            files.push(path);
        }
    }
}
