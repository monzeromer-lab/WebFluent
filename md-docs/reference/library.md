---
chapter: 53
kind: "reference"
route: "/reference/library"
group: "reference"
blurb: "The compiler as a library: the Rust crate's documented API, the Node package, and wf.js in a browser."
description: "The compiler as a library: the webfluent crate (Template, build, memory, vfs, diagnostics, config, errors, semver), the Node package and wf.js."
---
# 53. Library

This page is the reference for the compiler used as a library: the
`webfluent` crate in a Rust program, the `webfluent` package in Node, and
`wf.js` in a browser. Each one compiles the same language as `wf`, with the
same findings.

| | What it is | Use it to |
|---|---|---|
| [Rust](#rust-the-webfluent-crate) | the compiler itself, `webfluent` on crates.io | render templates in a server, build a site from a program, write an editor or a tool |
| [Node](#node-the-webfluent-package) | a wrapper that runs `wf render` | render templates from a Node server |
| [Browser](#browser-wf-js) | the compiler built for WebAssembly | compile and preview a project in a page, with no server |

This Rust program renders a template to HTML:

```rust
// Cargo.toml: webfluent = { version = "6", default-features = false }
//             serde_json = "1"
use webfluent::Template;

fn main() -> webfluent::Result<()> {
    let tpl = Template::from_str(
        r#"page Hello(path: "/", title: "Hello") { Heading("Hello, {name}!").h1 }"#,
    )?;
    let html = tpl.render_html(&serde_json::json!({ "name": "Ada" }))?;
    println!("{html}");
    Ok(())
}
```

## Rust: the `webfluent` crate

The crate is the compiler: `wf`, the language server and the WebAssembly
build are all made from it. Item-level documentation — every type, field
and method — is on [docs.rs/webfluent](https://docs.rs/webfluent); this
section says which parts exist, what each is for, and what semver covers.

### Add it

```toml
[dependencies]
webfluent = { version = "6", default-features = false }
```

`default-features = false` leaves out the `wf` command's own dependencies
(its argument parser and the development server). A program that only
renders templates or builds projects needs neither.

| Feature | Default | What it adds |
|---|---|---|
| `cli` | on | the `wf` command's dependencies: `clap`, `tiny_http`, `mimalloc` |
| `webp`, `pdf`, `bundled-fonts` | off | for a WebAssembly build only: lossy WebP, the PDF engine, the Liberation faces. A native build has all three whatever the features say |
| `internals` | off | the compiler's own modules, outside semver (see [What semver covers](#what-semver-covers)) |

The crate's minimum Rust version is the workspace's `rust-version`, and
`webfluent::VERSION` is the compiler's version as `wf --version` prints it.

### The documented API

| Module or item | What it is for |
|---|---|
| [`Template`](https://docs.rs/webfluent/latest/webfluent/struct.Template.html), `template` | Render `.wf` templates with JSON data to HTML, an HTML fragment, a PDF or a slide deck |
| `build` | `wf build` and `wf check` as function calls |
| `memory` | Build a project held in memory: no disk read, none written |
| `vfs`, `outfs` | Where a build reads a project from, and where it writes |
| `diagnostics`, [`Diagnostic`](https://docs.rs/webfluent/latest/webfluent/diagnostics/struct.Diagnostic.html) | Findings, their codes, and the JSON, SARIF and GitHub formats |
| `config` | `webfluent.app.json`, loaded and typed |
| `syntax`, `parse_source` | The parser, the tree (`syntax::parser::ast`), the formatter (`syntax::fmt`) and the `.wfx` layout (`syntax::layout`) |
| `init` | `wf init`: a new project from a starter |
| `studio`, `compile_studio` | The compile an editor's live preview runs |
| [`WebFluentError`](https://docs.rs/webfluent/latest/webfluent/enum.WebFluentError.html), `Result` | The error every fallible call returns |

### Templates

`Template` parses and checks once, is `Send + Sync`, and is cheap to clone,
so a server loads its templates at start-up and renders from any thread.
Data is any `serde::Serialize` value. This program renders one page of a
directory of templates to PDF:

```rust
use serde::Serialize;
use webfluent::Template;

#[derive(Serialize)]
struct Receipt {
    number: u32,
    total: f64,
}

fn main() -> webfluent::Result<()> {
    let templates = Template::from_dir("templates")?;
    let pdf = templates
        .page("Receipt")?
        .with_lang("en")
        .render_pdf(&Receipt { number: 7, total: 12.5 })?;
    std::fs::write("receipt.pdf", pdf)?;
    Ok(())
}
```

| Load | Configure | Render |
|---|---|---|
| `from_str`, `from_file`, `from_files`, `from_dir`, `from_sources` | `page`, `with_theme`, `with_tokens`, `with_lang`, `with_pdf`, `with_slides` | `render_html`, `render_html_fragment`, `render_html_parts` (CSS and body apart), `render_pdf`, `render_slides`, and `render_pdf_report` with the engine's notes |

A template is the static part of the language. [Server
rendering](../recipes/index.md) shows what one may use, and how
to render from a web server.

### Builds

`build::build` is `wf build`, and `build::check` is `wf check`. Both read
the project through a `Vfs` and return a `BuildReport`: every line the
command prints, every finding as a `Diagnostic`, every page's route, and
where the site was written. A build that stops returns a `BuildFailure`
holding the report so far and the error:

```rust
use std::path::Path;
use webfluent::build::{BuildOptions, build};
use webfluent::vfs::FsVfs;

fn main() {
    let options = BuildOptions {
        deny_warnings: true,
        ..BuildOptions::new(Path::new("my-site"), &FsVfs)
    };
    match build(&options) {
        Ok(report) => println!("{} pages in {}", report.pages, report.output_dir.display()),
        Err(failure) => {
            for d in failure.report.errors() {
                eprintln!("{}:{}:{}: {} [{}]", d.file, d.line, d.column, d.message, d.code);
            }
            eprintln!("{}", failure.error);
            std::process::exit(1);
        }
    }
}
```

Each field of `BuildOptions` is one of `wf build`'s flags: `output_dir`,
`allow_output_outside`, `stats`, `format`, `deny_warnings`, `dev` (a `wf
serve` build), `cache` (`--cache`), `studio` (`--studio`) and `on_line`, a
callback for each line as the build prints it.

### Read a project through a `Vfs`

Everything that reads a project reads it through `vfs::Vfs`:

- `FsVfs` is the disk, and what every command uses.
- `OverlayVfs::over_disk()` is the disk with an editor's unsaved buffers
  over it. `set` lays text over a path, `set_bytes` a picture or a font,
  and `delete` hides a file as though it were gone.

This function checks a project as an editor holds it, before the file is
saved:

```rust
use std::path::Path;
use webfluent::build::{BuildOptions, check};
use webfluent::vfs::OverlayVfs;

fn findings_while_editing(dir: &Path, unsaved: &str) -> usize {
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(dir.join("src/pages/Home.wf"), unsaved);
    match check(&BuildOptions::new(dir, &vfs)) {
        Ok(report) => report.diagnostics.len(),
        Err(failure) => failure.report.diagnostics.len(),
    }
}

fn main() {
    let n = findings_while_editing(Path::new("my-site"), "page Home(path: \"/\") { }");
    println!("{n} findings");
}
```

What a build writes goes to `outfs`: the disk, or memory inside
`outfs::capture`. An overlay changes what the project says, not where its
output lands.

### Build from memory

`memory::compile_files` builds a project given as a map of path to contents
and returns what it wrote as a map, with every finding. It reads no disk and
writes none, which is how the browser build works, and it runs natively too:

```rust
use webfluent::memory::{Contents, Options, compile_files};

fn main() {
    let files = [(
        "src/pages/Home.wf".to_string(),
        Contents::Text(
            r#"page Home(path: "/", title: "Home", description: "The front page.") { Heading("Hello").h1 }"#.into(),
        ),
    )];
    let out = compile_files(files, None, &Options::default());
    for d in &out.diagnostics {
        println!("{} {}", d.code, d.message);
    }
    if out.ok {
        println!("{} bytes of HTML", out.files["index.html"].len());
    }
}
```

`files` holds text files, and `binary` the pictures and PDFs. `None` as the
config uses the one in the map, or a default. `Options { check_only: true,
.. }` is `wf check`; `memory::compile` takes text only; `memory::explain`
is `wf explain`.

### Findings

A `Diagnostic` is one finding: its `code`, `severity`, `message`, `file`,
`line` and `column` (1-based, counted in characters), `hint`, the other
places it concerns (`related`) and the edits that fix it (`fixes`). It is
the same value `wf check --format json` prints, and the language server
sends. `diagnostics::format::{json, sarif, github}` write a list as those
formats; `diagnostics::codes` is the registry of every code, and
`docs_url()` its page.

### Configuration

`config::ProjectConfig::load(dir)` reads `webfluent.app.json` with every
default filled in, and `load_via(vfs, dir)` reads it through a `Vfs`. Its
fields are the file's keys ([Configuration](configuration.md)).

### Errors

Every fallible call returns `webfluent::Result<T>`, whose error is
`WebFluentError`:

| Case | When |
|---|---|
| `LexerError`, `ParseError` | the source cannot be read; holds the finding |
| `Diagnostics` | the checks found errors; holds every finding |
| `ConfigError` | the config is missing, is not JSON, or sends the output out of the project |
| `IoError` | a file cannot be read or written |
| `NotFound` | what was asked for is not there: a file, a page, the text an edit names |
| `CodegenError`, `EditError` | code generation failed, or a structured edit cannot apply |

`IoError`, `ConfigError`, `CodegenError`, `EditError` and `NotFound` each
hold a message and, where there is one, the error beneath it, which
`std::error::Error::source` returns. `WebFluentError::io(message)` and
`io_with(message, cause)` make one. The enum is `#[non_exhaustive]`, so a
`match` on it needs a `_` arm:

```rust
use webfluent::{Template, WebFluentError};

fn main() {
    match Template::from_file("templates/missing.wf") {
        Ok(_) => println!("loaded"),
        Err(WebFluentError::IoError(e)) => eprintln!("cannot read it: {}", e.message()),
        Err(WebFluentError::Diagnostics(found)) => eprintln!("{} findings", found.len()),
        Err(other) => eprintln!("{other}"),
    }
}
```

### What semver covers

The modules and items in [The documented API](#the-documented-api) are
covered: a change that breaks them waits for a major version. A minor
version may add a case to a `#[non_exhaustive]` enum (`WebFluentError`, the
tree's `Declaration`, `Expr` and `StatementKind`, `OutputType`,
`memory::Contents`, …), which is why a `match` on one needs a `_` arm.

The rest of the compiler — the checker, the code generator, the runtime,
the PDF engine, the registry and the tools behind the other commands — is
the crate's own. The `internals` feature makes it public again for a tool
that needs it, hidden from the docs and outside semver: anything there may
change in any release. A crate that turns it on pins an exact version:

```toml
[dependencies]
webfluent = { version = "=6.0.0", default-features = false, features = ["internals"] }
```

## Node: the `webfluent` package

The package renders templates from Node.js. It is a wrapper around `wf
render`: each render runs `wf` once and sends the data on standard input,
so `wf` must be installed too. Its source is
[`bindings/node/`](https://github.com/monzeromer-lab/WebFluent/tree/master/bindings/node),
and its version is the compiler's it was released with.

```bash
npm install webfluent
```

If npm does not find it, install it from a clone of the repository:

```bash
npm install ./WebFluent/bindings/node
```

This script renders one page of a directory of templates in a request
handler, without blocking the event loop:

```js
const http = require("node:http");
const { Template } = require("webfluent");

const receipt = Template.fromDir("templates").page("Receipt").withLang("en");

http.createServer(async (req, res) => {
  const html = await receipt.renderHtmlAsync({ number: 7, total: 12.5 });
  res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
  res.end(html);
}).listen(8080);
```

| Method | What it does |
|---|---|
| `Template.fromString(source)`, `fromFile(path)`, `fromDir(path)` | Load a template; a directory shares its components |
| `page(name)`, `withLang(lang)`, `withTheme(name)`, `withTokens({ … })` | Choose the page, the language, the theme and design tokens over it |
| `renderHtml(data)`, `renderHtmlFragment(data)` | A whole document, or the body only, as a string |
| `renderPdf(data)`, `renderSlides(data)` | A PDF, or a slide deck, as a `Buffer` |
| `renderHtmlAsync`, `renderHtmlFragmentAsync`, `renderPdfAsync`, `renderSlidesAsync` | The same, as a `Promise`: use these in a server |

The package finds `wf` in `WF_BIN`, then in the absolute directories on
`PATH` (never the current directory), then in `~/.webfluent/bin` and
`~/.cargo/bin`. It needs Node 18 or later, and ships its TypeScript
types (`index.d.ts`).

## Browser: `wf.js`

The compiler also builds for WebAssembly
([`crates/wf-wasm`](https://github.com/monzeromer-lab/WebFluent/tree/master/crates/wf-wasm)).
A page imports `wf.js` and compiles a project with nothing sent to a
server: sites and apps, every check `wf check` makes, pictures sized to
WebP, PDFs and slide decks. This module compiles a page and shows it in a
frame:

```js
import { compile, preview, previewConfig } from "./wf.js";

const home = `page Home(path: "/", title: "Home", description: "The front page.") {
    Heading("Hello").h1
}`;

const out = await compile(
  { "src/pages/Home.wf": home },
  previewConfig({ name: "Playground" }),
);
for (const d of out.diagnostics) console.log(d.code, d.message);
if (out.ok) document.querySelector("iframe").src = await preview(out);
```

| Export | What it does |
|---|---|
| `compile(files, config)` | Builds `files` (path → text, or a `Uint8Array` for a picture or a font) with `config`, the text or object of `webfluent.app.json`. Resolves to `{ ok, files, binary, diagnostics, error, lines }`; `diagnostics` is what `wf check --format json` lists |
| `check(files, config)` | Every finding, with nothing built |
| `explain(code)` | `{ code, severity, title, summary, docs }`, or `null` |
| `version()` | The compiler's version |
| `preview(out, route?)` | Hands a build to a service worker (`preview-sw.js`) that serves it under `preview/`, as a host would, and resolves to the address for an `<iframe>` |
| `previewConfig(config)` | `config` with `build.base_path` set to where `preview` serves, so the pages ask for their files there |
| `registerFont(bytes)` | A face a document may be set in or fall back to (TTF or OTF, as it is, zlib or gzip); resolves to its family |
| `preload({ document })` | Starts loading before the first call; `document: true` loads the PDF build too |

The `<iframe>` needs `sandbox="allow-scripts allow-same-origin"` for the
service worker to answer it. Give the preview an origin of its own when the
page around it holds anything worth protecting.

### The two builds

`crates/wf-wasm/build.sh` writes two builds, and `wf.js` loads each only
when a project needs it. Sizes are of the 6.0 build, compressed with
`gzip -9`:

| File | Raw | Gzipped | Loaded |
|---|---|---|---|
| `pkg/wf_wasm_bg.wasm`, the compiler with WebP | 4.8 MB | 1.6 MB | on the first `compile`, `check` or `preload` |
| `pkg-pdf/wf_wasm_pdf_bg.wasm`, the same with the PDF engine | 8.4 MB | 2.7 MB | the first time a config's `output_type` is `"pdf"` or `"slides"` |
| `pkg-pdf/fonts/*.ttf.z`, the twelve Liberation faces | 2.4 MB | already compressed | with the PDF build |
| `pkg-pdf/fonts/NotoSansArabic.ttf.gz` | 455 kB | already compressed | the first time a document's text has Arabic in it |

A browser has no machine fonts to fall back on: a document in a script the
Liberation faces and the Arabic face lack needs a face given to
`registerFont`. `wf test`, `wf verify`, `wf serve` and `wf fmt` are not in
the browser build.

To build both, install `wasm-bindgen-cli` 0.2.130, the
`wasm32-unknown-unknown` target and a clang that targets wasm32 (libwebp is
C), then run:

```bash
crates/wf-wasm/build.sh
```

## Next

[Editors](editors.md) — the language server, and the extensions for VS
Code and Zed.
