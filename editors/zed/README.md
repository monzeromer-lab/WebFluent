# WebFluent for Zed

Language support for [WebFluent](https://github.com/monzeromer-lab/WebFluent)
`.wf` files in the [Zed](https://zed.dev) editor.

- Syntax highlighting, bracket matching and rainbow brackets, auto-indent,
  code folding, outline panel and breadcrumbs, Vim text objects — all from the
  Tree-sitter grammar in `../tree-sitter-webfluent`.
- Diagnostics, completions, hover documentation, go to definition, document
  symbols and quick fixes from `wf-lsp`, the WebFluent language server.
- Snippets for pages, components, stores, resources, style rules and the
  common built-ins. Type `page`, `resource`, `card`, … and accept the completion.
- ```` ```wf ```` fences in Markdown are highlighted as WebFluent.

## Layout

```
editors/zed/
  extension.toml          manifest: grammar, language server, snippets
  Cargo.toml, src/        the Rust part, compiled to WebAssembly by Zed
  languages/webfluent/
    config.toml           comments, brackets, indentation, overrides
    highlights.scm        syntax highlighting
    brackets.scm          bracket pairs
    outline.scm           outline panel entries
    indents.scm           auto-indent ranges
    injections.scm        embedded languages (comments)
    overrides.scm         scopes that config.toml refers to
    textobjects.scm       Vim-mode text objects
  snippets/webfluent.json
```

The grammar is not vendored here. `extension.toml` points at this repository
and a commit, and Zed clones it and compiles
`editors/tree-sitter-webfluent/src/parser.c` itself.

## The language server

The extension looks for `wf-lsp` in this order and uses the first it finds:

1. `lsp.wf-lsp.binary.path` in your Zed `settings.json`.
2. A `wf-lsp` on your `PATH` — `cargo install --path crates/wf-lsp` from a
   checkout, or `cargo install --git https://github.com/monzeromer-lab/WebFluent wf-lsp`.
3. The latest [GitHub release](https://github.com/monzeromer-lab/WebFluent/releases),
   which it downloads on first use (the status shows in Zed's bottom bar) —
   and again, into a new `wf-lsp-<version>/` directory, when a newer release
   is out: restart Zed after a WebFluent release and the server is the new
   one. Extension 1.1 was cut with wf-lsp 2.2.0.
4. A copy it downloaded before, when the release lookup fails (offline, or
   rate-limited), so the editor keeps working without the network.

To pin a binary, or pass it arguments or environment:

```json
{
  "lsp": {
    "wf-lsp": {
      "binary": {
        "path": "/home/me/.cargo/bin/wf-lsp",
        "arguments": [],
        "env": {}
      }
    }
  }
}
```

Anything under `lsp.wf-lsp.settings` is forwarded to the server as its
workspace configuration.

## Developing the extension

You need a Rust toolchain installed through `rustup` (Zed adds the
`wasm32-wasip2` target itself) and, for the grammar, `git` on your `PATH`.

1. In Zed, open the command palette and run `zed: install dev extension`,
   then pick this directory (`editors/zed`).
2. Open any `.wf` file. Zed compiles the Rust crate and the grammar the first
   time, which takes a minute; after that it is instant.
3. After changing a query, a snippet or `config.toml`, run
   `zed: reload extensions`. After changing the Rust code, reinstall the dev
   extension.

Zed writes `target/`, `grammars/` and `extension.wasm` into this directory
while doing so; they are ignored by git.

### Changing the grammar

The grammar Zed uses is whatever `[grammars.webfluent] rev` in
`extension.toml` names, so a grammar change reaches the extension in two
steps: commit the grammar, then point `rev` at that commit and commit the
manifest. From the repository root:

```bash
just grammar-test          # regenerate, run the corpus, parse every .wf in the repo
git commit -am "grammar: …"
just zed-pin-grammar       # rewrites rev = <HEAD> in extension.toml
git commit -am "zed: pin the grammar"
```

To try an uncommitted grammar in Zed before that, point the manifest at your
working copy for the duration — `repository = "file:///path/to/WebFluent"`
still needs `rev` to be a commit that exists locally, so commit the grammar
on a branch first — and change it back before committing the manifest.

`just zed-check` builds the Rust crate for `wasm32-wasip2`, validates every
query against the grammar, and checks that the modifier vocabulary in
`highlights.scm` matches the compiler's.

### Troubleshooting

- **No highlighting**: open `zed: open log`. A query that names a node the
  grammar does not have fails there with the file and line.
- **The server does not start**: the error in the log names every place the
  extension looked. Installing `wf-lsp` on the `PATH` is the quickest fix.
- **`--foreground`**: launching Zed from a terminal with `zed --foreground`
  prints the extension's own output.

## Publishing

Zed's extension registry is the
[zed-industries/extensions](https://github.com/zed-industries/extensions)
repository. Publishing means adding this directory as a git submodule there
and an entry to its `extensions.toml`; the registry builds the extension from
the pinned commit. See Zed's
[developing extensions](https://zed.dev/docs/extensions/developing-extensions)
guide for the current procedure.
