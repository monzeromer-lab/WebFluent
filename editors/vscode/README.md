# WebFluent for VS Code

Highlighting for WebFluent's `.wf` and `.wfx` files, and the language server
that checks them as you type.

- **Diagnostics** — the compiler's own errors and warnings, with the line.
- **Completion** — elements, their props, flags, events and slots, from the
  compiler's registry.
- **Hover, go to definition, rename** across the project.
- **Quick fixes** and *Extract component*.

## The language server

The extension runs `wf-lsp`. It looks for it in this order:

1. the `webfluent.server.path` setting;
2. `wf-lsp` on your `PATH`;
3. `target/release` or `target/debug` in the workspace (the compiler's own
   checkout);
4. otherwise it downloads the latest release's `wf-lsp` for your platform
   once and keeps it.

Releases are built for x86_64 Linux, macOS (Apple Silicon and Intel) and
x86_64 Windows. Elsewhere, build it with `cargo build --release -p wf-lsp`
from a clone and point `webfluent.server.path` at it.

## Settings

| Setting | Default | |
|---|---|---|
| `webfluent.server.path` | `""` | The `wf-lsp` to run. |
| `webfluent.server.download` | `true` | Download `wf-lsp` when it is not found. Off: highlighting only. |

## Links

- [WebFluent](https://github.com/monzeromer-lab/WebFluent) — the language,
  and where to report a problem with this extension.
- [Documentation](https://monzeromer-lab.github.io/WebFluent)
