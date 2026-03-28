use zed_extension_api::{self as zed, Result};
use std::fs;

struct WebFluentExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for WebFluentExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Try to find wf-lsp in PATH first
        if let Some(path) = worktree.which("wf-lsp") {
            return Ok(zed::Command {
                command: path,
                args: vec![],
                env: Default::default(),
            });
        }

        // Try common install locations
        let home = std::env::var("HOME").unwrap_or_default();
        let cargo_bin = format!("{}/.cargo/bin/wf-lsp", home);
        if fs::metadata(&cargo_bin).is_ok() {
            return Ok(zed::Command {
                command: cargo_bin,
                args: vec![],
                env: Default::default(),
            });
        }

        // Try the project's target directory
        let release_path = "target/release/wf-lsp".to_string();
        if fs::metadata(&release_path).is_ok() {
            return Ok(zed::Command {
                command: release_path,
                args: vec![],
                env: Default::default(),
            });
        }

        Err("wf-lsp not found. Install with: cargo install --path crates/wf-lsp".into())
    }
}

zed::register_extension!(WebFluentExtension);
