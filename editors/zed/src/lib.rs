use zed_extension_api::{self as zed, Result};

struct WebFluentExtension;

impl zed::Extension for WebFluentExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // 1. Check if user configured a custom binary path in Zed settings
        if let Ok(lsp_settings) = zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            if let Some(binary) = lsp_settings.binary {
                if let Some(path) = binary.path {
                    return Ok(zed::Command {
                        command: path,
                        args: binary.arguments.unwrap_or_default(),
                        env: binary.env.unwrap_or_default().into_iter().collect(),
                    });
                }
            }
        }

        // 2. Search for wf-lsp in PATH
        let path = worktree
            .which("wf-lsp")
            .ok_or_else(|| {
                "wf-lsp not found in PATH.\n\
                 Install with: cargo install --path crates/wf-lsp\n\
                 Or specify path in Zed settings.json: lsp.wf-lsp.binary.path"
                    .to_string()
            })?;

        Ok(zed::Command {
            command: path,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(WebFluentExtension);
