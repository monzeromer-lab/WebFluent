# 2. Getting started

<!--
route: guide/getting-started
group: start
blurb: Install the compiler and your editor's support, make a project, and see what a build writes.
description: Install wf and the language server, create a first project, run the dev server, and learn what a build produces.
-->

By the end of this chapter you will have installed the compiler, set up your
editor, built a two-page site, changed it with the dev server watching, and
know what every file the build writes is for.

## Install

WebFluent is one binary, `wf`. Run the line for your machine; each one leaves
`wf` on your `PATH`.

### macOS and Linux

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
```

Fetches the binary for your platform from the latest release — no Rust
needed. It covers Apple Silicon and Intel Macs and x86_64 Linux.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex
```

### With Rust

```bash
cargo install webfluent
```

Builds from crates.io. This works everywhere Rust does, and it is how to
install on Linux arm64, which has no prebuilt binary yet.

### Packages

Each [release](https://github.com/monzeromer-lab/WebFluent/releases) also
carries a `.deb`, an `.msi` and plain tarballs, for a machine where piping a
script to a shell is not an option.

### Check it

```bash
wf --version
```

If the shell cannot find `wf`, open a new terminal: the installer adds its
directory to your `PATH` in your shell's startup file, which the session you
ran it in had already read.

Both installers read two environment variables: `WF_INSTALL_DIR` puts the
binary somewhere other than `~/.webfluent/bin`, and `WF_VERSION` pins a
release (`WF_VERSION=v4.0.1`) rather than taking the latest. Running the
installer again upgrades in place.

To build from a clone instead — for an unreleased change, or to work on the
compiler itself:

```bash
git clone https://github.com/monzeromer-lab/WebFluent.git
cd WebFluent && cargo install --path .
```

## Set up your editor

The language server, `wf-lsp`, gives your editor the same checks `wf build`
runs, as you type: diagnostics, completion of props, flags and cases, hover
with types and docs, go to definition, rename and quick fixes.

It is a second binary. Get it from the same
[release](https://github.com/monzeromer-lab/WebFluent/releases) as `wf` —
`wf-lsp-<version>-<arch>-<os>.tar.gz` — and put it on your `PATH`, or build
it from a clone:

```bash
cargo install --path crates/wf-lsp
```

- **VS Code** — the extension in `editors/vscode` brings highlighting and the
  language server. Until it is on the Marketplace, run `npm ci && npm run
  package` in that folder and install the `.vsix` it writes with
  *Extensions: Install from VSIX*.
- **Zed** — the extension in `editors/zed` brings Tree-sitter highlighting,
  the outline, snippets and the language server (found on `PATH`, or
  downloaded from the latest release). Install it with
  *zed: install dev extension* pointed at that folder.
- **Neovim, Helix, Emacs, anything else with LSP** — run `wf-lsp` over stdio
  for the `webfluent` file type (`*.wf`, `*.wfx`). [Editors](37-cli.md#editors)
  has configuration for each.

## A first project

```bash
wf init hello
cd hello
wf serve
```

`wf init` writes a small application — four pages (a dashboard, a task list,
a profile and settings), three components and two stores — so you have
something real to read. `wf serve` builds it, serves it at
`http://localhost:3000`, rebuilds when you save, reloads the page when a build
lands, and — when a build fails — draws the error over the page until the
next save fixes it.

`wf init` takes a template:

| Template | Makes |
|---|---|
| `-t spa` (default) | An interactive application, routed in the browser |
| `-t static` | A marketing site, every page pre-rendered to HTML |
| `-t pdf` | A PDF document |
| `-t slides` | A PDF slide deck |

Not sure between `spa` and `static`? Start with `static`: every page is real
HTML before any script runs, which is better for search and for a first
paint, and it is still fully interactive once the script loads.
[Static or single-page](26-static-and-spa.md) explains the difference.

## The smallest site

Replace `src/` with two files.

`src/App.wf` — the shell every page shares, and the router:

```wf
app {
    Navbar {
        Navbar.Brand { Text("Hello").heading }
        Navbar.Links {
            Link("Home", to: "/")
            Link("About", to: "/about")
        }
    }
    Router
}
```

`src/pages/Home.wf` — pages, each owning its route:

```wf
page Home(path: "/", title: "Hello", description: "A first WebFluent site.") {
    state count = 0
    Container {
        Heading("Hello, world").h1
        Text("You have clicked {count} times.")
        Button("Click me").primary { on click { count = count + 1 } }
    }
}

page About(path: "/about", title: "About", description: "What this site is for.") {
    Container {
        Heading("About").h1
        Paragraph("Two pages, one shell, no framework.")
    }
}
```

Read it top to bottom:

- `app { … }` is the shell: what every page sits inside. `Router` is where
  the current page renders. There is no route table — each `page` names its
  own `path:`.
- `page Home(path: "/", title: …, description: …)` declares a page. `title`
  and `description` become the `<title>`, the meta description, the Open Graph
  and Twitter cards and the JSON-LD; leave them out and the build warns you.
- `state count = 0` is a signal. `Text("… {count} …")` reads it, so the text
  updates when it changes.
- `Button("Click me").primary { on click { … } }` — the label is the one
  positional argument, `.primary` is a flag (a case of the button's `tone`
  prop), and the block holds the click handler.
- `Container`, `Heading`, `Text`, `Paragraph`, `Navbar` and `Link` are
  built-ins; the [components reference](36-components-reference.md) lists
  all of them.

Save, and the dev server shows the change. Run `wf build` to write the
finished site to `build/`.

## What a build produces

```text
build/
  index.html          the page, pre-rendered (static) or the app shell (spa)
  about/index.html    one directory per route in a static build
  404.html            the `path: "*"` page, which a host serves for unknown paths
  app.js              the runtime this program needs, its stores and components
  pages/Home.js       each page in its own chunk, loaded when its route shows
  pages/Home.css      the styles only that page reaches
  styles.css          tokens, the built-ins you used, your stylesheets
  sitemap.xml         every indexable route (needs meta.site_url)
  robots.txt
  _headers            the security headers (build.csp), for hosts that read the file
  sw.js               the service worker, when the config names "offline"
  *.gz                pre-compressed copies a host can serve as they are
```

Nothing in `build/` is for editing: it is written again on every build, so
keep it out of version control. `public/` is copied to its root as it is.

`app.js` carries only the parts of the runtime the program reaches. The
runtime is a set of feature modules — `each` for a `for`, `router` for a
second page, `net` for a `resource`, `icons` for an `Icon` — and the build
works out which ones the code it just wrote calls. A page of static text
carries 3 of its 37 modules — an `app.js` of 3.7 kB gzipped. `wf build
--stats` shows what a build weighs; [Performance](31-performance.md) reads it.

## The project, in brief

```text
webfluent.app.json    the config
src/
  App.wf              the app shell (optional)
  pages/*.wf          pages
  components/*.wf     components
  stores/*.wf         stores
  *.css               your own stylesheets, bundled into styles.css
  translations/*.json translation files, with "i18n" in the config
public/               copied to the build's root (images, fonts, favicon)
tests/*.wf            `test "…" { }` declarations for `wf test`
```

Every `.wf` or `.wfx` file under `src/` is read and merged into one program:
a page in one file uses a component from another without importing it. The
folders are convention, not rules. [Project structure](25-project-structure.md)
covers naming, organisation and the config; the
[configuration reference](38-configuration.md) lists every key.

The config needs only a name:

```json
{ "name": "hello" }
```

## Next

[Tutorial: a reading list, start to finish](03-tutorial.md).
