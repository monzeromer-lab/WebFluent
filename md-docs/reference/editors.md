---
chapter: 42
kind: "reference"
route: "/reference/editors"
group: "reference"
blurb: "The language server, wf-lsp, and what the VS Code and Zed extensions and the Tree-sitter grammars each support."
description: "Editor support for WebFluent: what wf-lsp answers, what the VS Code and Zed extensions and the Tree-sitter grammars support, and how to install each."
---
# Editors

This page is the reference for WebFluent's editor support: the language
server, `wf-lsp`, and the three repositories built around it. It lists
what each one supports and how to install it.

| | Repository | Highlighting | Language server | Snippets |
|---|---|---|---|---|
| VS Code | [webfluent-vscode](https://github.com/monzeromer-lab/webfluent-vscode) | TextMate grammar, `.wf` and `.wfx` | runs `wf-lsp`, downloaded and checked if it is not installed | no |
| Zed | [webfluent-zed](https://github.com/monzeromer-lab/webfluent-zed) | Tree-sitter, `.wf` and `.wfx` | runs `wf-lsp`, downloaded and checked if it is not installed | yes |
| Any editor with Tree-sitter | [tree-sitter-webfluent](https://github.com/monzeromer-lab/tree-sitter-webfluent) | the two grammars | — | — |
| Any editor with LSP | [`crates/wf-lsp`](https://github.com/monzeromer-lab/WebFluent/tree/master/crates/wf-lsp) | semantic tokens from the server | `wf-lsp` over standard input and output | — |

Every finding, completion and quick fix comes from `wf-lsp`, which is the
compiler itself: an editor reports what `wf check` reports.

## The language server: `wf-lsp`

`wf-lsp` runs the compiler's checks as you type and answers the editor's
questions about the code. Keep it at the version of your `wf`;
`wf-lsp --version` prints it.

To install it, download `wf-lsp-<version>-<arch>-<os>` from a
[release](https://github.com/monzeromer-lab/WebFluent/releases) and put it
on your `PATH`, or build it from a clone of the repository:

```bash
cargo install --path crates/wf-lsp
```

Releases are built for x86_64 Linux, macOS (Apple Silicon and Intel) and
x86_64 Windows. Both extensions download the server themselves when it is
not on your `PATH`.

### What it answers

| Request | What you get |
|---|---|
| Diagnostics | Every check `wf build` runs, for `.wf`, `.wfx` and Markdown pages, with the code, the place and the fixes. They run again when a file they depend on changes: a translation, a `.env`, a script, a stylesheet, the config or a file in `public/` |
| Completion | Components and their props, flags and parts, enum cases, events, `$` tokens, `match` arms, slots, record fields and the methods of each type, a service's endpoints, the keywords each place allows, translation keys and lint codes. In a PDF or slides project it offers only what paper can draw |
| Hover | A component's documentation and signature, a name's inferred type, a field, an endpoint, a case, a keyword or a built-in function |
| Signature help | What a call takes, and which argument the cursor is on |
| Go to definition, references, highlights | For every declaration — a component, a store, a type, a case, a field, a `const`, a `data` file, an animation, an endpoint — and inside a string's `{…}` splices |
| Rename | Across the project, after the editor asks whether the name can be renamed |
| Formatting | `wf fmt`, for the file or a range |
| Folding, inlay hints | Blocks fold; an inlay hint shows the type of a `state` or `derived` value written without one |
| Semantic tokens | A name is coloured by what it resolves to: a component, a type, a store, a prop or a state |
| Document and workspace symbols | The outline, and a search over the project's declarations |
| Code actions | The fixes the compiler attaches to a finding (quick fix), all of a file's fixes at once (fix all), and **Extract component** |

The [command line reference](cli.md#wf-lsp) lists each of these in full.

### How it finds the project

The project root is the nearest folder above the open file that holds
`webfluent.app.json`. The server keeps the analysed project between
requests and analyses it again only after a change, so a hover answers in
about a tenth of a millisecond. It asks the editor to tell it when a
`.wf`, `.wfx`, `.md`, `.js`, `.css` or `.json` file, a `.env` or a file in
`public/` changes on disk.

## VS Code

The extension highlights `.wf` and `.wfx` files with a TextMate grammar and
runs `wf-lsp` for everything else. It needs VS Code 1.82 or later.

To install it, download `webfluent-vscode-<version>.vsix` from the
extension's [latest release](https://github.com/monzeromer-lab/webfluent-vscode/releases/latest),
then run **Extensions: Install from VSIX** and pick the file. It is not on
the Marketplace yet.

The extension looks for the server in this order:

1. the `webfluent.server.path` setting;
2. `wf-lsp` in an absolute directory on your `PATH`;
3. `target/release` or `target/debug` in the workspace, only when
   `webfluent.server.useWorkspaceBuild` is on in your user settings and
   the workspace is trusted;
4. the `wf-lsp` of the WebFluent release this version of the extension
   pins, downloaded for your platform and checked against that release's
   `SHA256SUMS`. A download that does not match is refused.

| Setting | Default | What it does |
|---|---|---|
| `webfluent.server.path` | `""` | The `wf-lsp` to run. A workspace's value counts only once you trust the workspace |
| `webfluent.server.useWorkspaceBuild` | `false` | Runs a `wf-lsp` built in the workspace's `target/`, for work on the compiler. Read from user settings only |
| `webfluent.server.download` | `true` | Downloads the pinned release's `wf-lsp` when none is found. Off: highlighting only |

This user setting runs a server you installed yourself:

```json
{
  "webfluent.server.path": "/home/me/.cargo/bin/wf-lsp"
}
```

## Zed

The extension highlights `.wf` and `.wfx` files with the Tree-sitter
grammars and runs `wf-lsp`. From the grammars it also has bracket
matching, auto-indent, folding, the outline panel and breadcrumbs, Vim text
objects, and highlighting of ` ```wf ` fences in Markdown. Its snippets
cover pages, components, stores, state, actions, control flow, resources,
style rules, the common built-ins, `persist`, `validate`, `on key`, `api`
and `test`.

It is not in Zed's extension registry yet. To install it, clone
[webfluent-zed](https://github.com/monzeromer-lab/webfluent-zed), run
**zed: install dev extension** and pick the clone. Zed compiles the
extension and the grammars the first time, which takes about a minute.

The extension looks for the server in this order:

1. `lsp.wf-lsp.binary.path` in your **user** `settings.json`;
2. `wf-lsp` on your `PATH`;
3. the WebFluent release this version of the extension pins, downloaded on
   first use and checked against the release's `SHA256SUMS`;
4. a copy it downloaded and checked before, when the release cannot be
   reached.

This user setting runs a server you installed yourself:

```json
{
  "lsp": {
    "wf-lsp": {
      "binary": { "path": "/home/me/.cargo/bin/wf-lsp", "arguments": [] }
    }
  }
}
```

A project's own `.zed/settings.json` cannot choose the program: when it
says anything about `lsp.wf-lsp.binary`, every `binary` setting is ignored
for that project, and a `binary.path` must be absolute and outside the
project. A project may still set `lsp.wf-lsp.settings`.

## Tree-sitter grammars

[tree-sitter-webfluent](https://github.com/monzeromer-lab/tree-sitter-webfluent)
holds two grammars, generated from one source:

| Grammar | Files | Folder |
|---|---|---|
| `webfluent` | `.wf`, blocks in braces | `webfluent/` |
| `webfluentx` | `.wfx`, blocks by indentation | `webfluentx/` |

The generated `src/parser.c` files are committed, so an editor builds the
parser without the Tree-sitter command. The Zed extension pins a commit of
the repository, and the grammars are tested against every sample in this
guide.

## Other editors

Any editor with LSP support runs `wf-lsp` over standard input and output
for the `webfluent` file type (`*.wf`, `*.wfx`), with `webfluent.app.json`
as the root marker. The command line reference has the configuration for
[Neovim](cli.md#neovim), [Helix](cli.md#helix) and [Emacs](cli.md#emacs).
For highlighting in an editor that reads Tree-sitter, use the grammars
above.

[WebFluent Studio](../learn/01-getting-started.md#webfluent-studio) is an
editor of its own, with the language server, a live preview and a designer
in one window.

## Next

[Library](library.md) — the compiler as a Rust crate, a Node package and a
browser module.
