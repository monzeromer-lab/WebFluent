# Changelog

## 0.3.0

- Highlighting for WebFluent 4: `api` services and their endpoints,
  `socket`, `stream` and `channel`, `validate`, `test` and its steps,
  `const`, `image`, `data`, `try`/`catch`, the eleven scalar types, date,
  time, money and duration literals, raw and block strings, and a string
  inside a splice. Every built-in component is recognised.
- The language server is downloaded from the latest release when it is not
  on `PATH`, and kept for the next start.
- `wf-lsp.exe` is found on Windows, where only `wf-lsp` was looked for.
- The extension ships as one bundle. The package before this one left the
  language client out and could not start.
- New settings: `webfluent.server.path`, `webfluent.server.download`.
- Needs VS Code 1.82 or later.

## 0.2.0

- `.wfx` files, and the `wf-lsp` language client.
