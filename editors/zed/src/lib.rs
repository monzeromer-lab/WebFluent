use zed_extension_api::{self as zed, Result};

struct WebFluentExtension;

impl zed::Extension for WebFluentExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Find wf-lsp in PATH (installed via cargo install --path crates/wf-lsp)
        let path = worktree
            .which("wf-lsp")
            .ok_or_else(|| "wf-lsp not found in PATH. Install with: cargo install --path crates/wf-lsp".to_string())?;

        Ok(zed::Command {
            command: path,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(WebFluentExtension);
