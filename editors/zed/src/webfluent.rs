//! The WebFluent extension for Zed.
//!
//! Everything static — the grammar, the language configuration, the queries,
//! the snippets — is declared in `extension.toml` and `languages/webfluent/`.
//! This crate exists for the two things a manifest cannot say: where the
//! language server is, and how its completions should be drawn.
//!
//! The server is found in this order, and the first hit wins:
//!
//! 1. `lsp.wf-lsp.binary.path` in the user's Zed settings.
//! 2. A `wf-lsp` on the `PATH` Zed was launched with (`cargo install --path
//!    crates/wf-lsp` puts it there).
//! 3. The latest GitHub release of WebFluent, downloaded into the extension's
//!    working directory — a new release is picked up on the next start.
//! 4. A copy this extension downloaded earlier, when the release cannot be
//!    looked up.

use std::fs;

use zed_extension_api::{
    self as zed,
    lsp::{Completion, CompletionKind, Symbol, SymbolKind},
    settings::LspSettings,
    CodeLabel, CodeLabelSpan, LanguageServerId, LanguageServerInstallationStatus, Result,
};

const SERVER_NAME: &str = "wf-lsp";
const GITHUB_REPO: &str = "monzeromer-lab/WebFluent";

struct WebFluentExtension {
    /// The binary a previous call found or downloaded. Re-checked on every
    /// call, because the user may have installed one on the `PATH` since.
    cached_binary_path: Option<String>,
}

impl WebFluentExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        // The latest release is what gets downloaded, so a new server ships
        // to every editor on its next start. When the lookup fails — no
        // network, a rate limit — a copy downloaded earlier still serves.
        let release = match zed::latest_github_release(
            GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        ) {
            Ok(release) => release,
            Err(error) => {
                if let Some(path) = downloaded_server() {
                    self.cached_binary_path = Some(path.clone());
                    return Ok(path);
                }
                return Err(install_hint(&format!(
                    "could not look up the latest release: {error}"
                )));
            }
        };

        let (os, arch) = zed::current_platform();
        let asset_name = format!(
            "{SERVER_NAME}-{version}-{arch}-{os}.{ext}",
            version = release.version,
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x86_64",
            },
            os = match os {
                zed::Os::Mac => "macos",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
            ext = match os {
                zed::Os::Mac | zed::Os::Linux => "tar.gz",
                zed::Os::Windows => "zip",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                install_hint(&format!(
                    "release {} has no asset named {asset_name}",
                    release.version
                ))
            })?;

        let version_dir = format!("{SERVER_NAME}-{}", release.version);
        let binary_path = format!(
            "{version_dir}/{SERVER_NAME}{}",
            if os == zed::Os::Windows { ".exe" } else { "" }
        );

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                match os {
                    zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
                    zed::Os::Windows => zed::DownloadedFileType::Zip,
                },
            )
            .map_err(|error| format!("failed to download {asset_name}: {error}"))?;

            zed::make_file_executable(&binary_path)?;

            // Older downloads are of no further use; keep the directory tidy.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    if entry.file_name().to_str() != Some(&version_dir) {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

/// The newest server this extension downloaded before, if any: the
/// `wf-lsp-<version>/wf-lsp` with the highest version in the working
/// directory.
fn downloaded_server() -> Option<String> {
    let (os, _) = zed::current_platform();
    let exe = if os == zed::Os::Windows { ".exe" } else { "" };
    let mut versions: Vec<(Vec<u32>, String)> = fs::read_dir(".")
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            let version = name.strip_prefix(&format!("{SERVER_NAME}-"))?;
            let path = format!("{name}/{SERVER_NAME}{exe}");
            fs::metadata(&path).ok()?.is_file().then_some(())?;
            let parts = version
                .trim_start_matches('v')
                .split('.')
                .map(|p| p.parse().unwrap_or(0))
                .collect();
            Some((parts, path))
        })
        .collect();
    versions.sort();
    versions.pop().map(|(_, path)| path)
}

/// An error that also says how to get past it, since the fix is one command.
fn install_hint(reason: &str) -> String {
    format!(
        "{SERVER_NAME} is not installed and could not be downloaded ({reason}).\n\
         Install it with `cargo install --git https://github.com/{GITHUB_REPO} wf-lsp`, \
         or point Zed at a binary in settings.json:\n\
         {{ \"lsp\": {{ \"{SERVER_NAME}\": {{ \"binary\": {{ \"path\": \"/path/to/{SERVER_NAME}\" }} }} }} }}"
    )
}

impl zed::Extension for WebFluentExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);

        let command = match settings.as_ref().and_then(|binary| binary.path.clone()) {
            Some(path) => path,
            None => self.language_server_binary_path(language_server_id, worktree)?,
        };

        let args = settings
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_default();

        // The server inherits the shell's environment (so a `PATH`, a proxy or
        // a locale set there reach it), with anything from settings on top.
        let mut env = worktree.shell_env();
        if let Some(extra) = settings.and_then(|binary| binary.env) {
            env.extend(extra);
        }

        Ok(zed::Command { command, args, env })
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        // Forwarded as-is from `lsp.wf-lsp.settings` in settings.json, so a
        // future server option needs no change here.
        Ok(
            LspSettings::for_worktree(language_server_id.as_ref(), worktree)
                .ok()
                .and_then(|settings| settings.settings),
        )
    }

    /// Draws a completion the way the same text would look in a buffer: a
    /// component in the component colour, a state variable after `state`,
    /// an action as a call. The `code` is parsed with the WebFluent grammar
    /// and highlighted by the theme, so the menu matches the editor.
    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        let name = completion.label.clone();
        let detail = completion.detail.as_deref().unwrap_or("");
        let kind = completion.kind?;

        let (code, name_range) = match kind {
            // Built-in and user components: `Card`, `UserCard`.
            CompletionKind::Class => (name.clone(), 0..name.len()),

            // `state count`, `derived total`, a prop, a store member.
            CompletionKind::Variable => {
                let prefix = if detail.contains("derived") {
                    "derived "
                } else if detail.contains("Prop") {
                    "prop "
                } else {
                    "state "
                };
                let code = format!("{prefix}{name}");
                (code, prefix.len()..prefix.len() + name.len())
            }

            // Actions and functions read as a call.
            CompletionKind::Function | CompletionKind::Method => {
                (format!("{name}()"), 0..name.len())
            }

            // Named arguments, style properties and map keys carry their colon.
            CompletionKind::Property | CompletionKind::Field => (format!("{name}:"), 0..name.len()),

            // Modifiers such as `primary` or `bold` are attributes in the
            // highlight query, and are drawn that way here too.
            CompletionKind::EnumMember | CompletionKind::Constant => {
                return Some(literal_label(&name, "attribute", detail));
            }

            CompletionKind::Keyword => return Some(literal_label(&name, "keyword", detail)),

            _ => return None,
        };

        let mut spans = vec![CodeLabelSpan::code_range(name_range)];
        spans.push(detail_span(detail));

        Some(CodeLabel {
            code,
            spans,
            filter_range: (0..name.len()).into(),
        })
    }

    fn label_for_symbol(
        &self,
        _language_server_id: &LanguageServerId,
        symbol: Symbol,
    ) -> Option<CodeLabel> {
        let highlight = match symbol.kind {
            SymbolKind::Class | SymbolKind::Module | SymbolKind::Namespace => "type",
            SymbolKind::Function | SymbolKind::Method => "function",
            SymbolKind::Variable | SymbolKind::Field | SymbolKind::Property => "variable",
            SymbolKind::Constant => "constant",
            _ => return None,
        };
        Some(CodeLabel {
            spans: vec![CodeLabelSpan::literal(
                symbol.name.clone(),
                Some(highlight.to_string()),
            )],
            filter_range: (0..symbol.name.len()).into(),
            code: String::new(),
        })
    }
}

/// A label drawn with one theme colour rather than parsed as code.
fn literal_label(name: &str, highlight: &str, detail: &str) -> CodeLabel {
    CodeLabel {
        spans: vec![
            CodeLabelSpan::literal(name.to_string(), Some(highlight.to_string())),
            detail_span(detail),
        ],
        filter_range: (0..name.len()).into(),
        code: String::new(),
    }
}

/// The server's one-line description, dimmed after the name.
fn detail_span(detail: &str) -> CodeLabelSpan {
    if detail.is_empty() {
        CodeLabelSpan::literal(String::new(), None)
    } else {
        CodeLabelSpan::literal(format!("  {detail}"), Some("comment".to_string()))
    }
}

zed::register_extension!(WebFluentExtension);
