---
chapter: 1
kind: "tutorial"
route: "/learn/01-getting-started"
from: ["/docs/guide/getting-started"]
group: "tour"
blurb: "Install the compiler, make a project, run it, change it and see the change — then set up your editor, or WebFluent Studio, and see what a build writes."
description: "Install wf, check the install, create a first project, run the dev server and make a change. Then set up the language server and read the build."
learned: ["Install `wf` and check it with `wf --version`", "Create a project with `wf init` and run it with `wf serve`", "Change a page and read the error the dev server draws over it", "Write a two-page site: an `app` with a `Router`, and pages that name their own `path:`", "Say what `wf build` writes to `build/`"]
---
# 1. Getting started

This chapter installs `wf`, the WebFluent compiler, and runs a first
project in your browser. In about ten minutes you create the project, change
one line and see the page follow. The rest of the chapter covers optional
set-up: your editor, the other project templates, a site written from
scratch, and what a build writes.

## Before you start

You need:

- a terminal: macOS or Linux with `curl` and `bash`, or Windows with
  PowerShell (or, on any system, Rust's `cargo`);
- a web browser;
- a text editor. Any editor works; [Set up your editor](#set-up-your-editor)
  adds checks as you type, and [WebFluent Studio](#webfluent-studio) is an
  app made for WebFluent.

## Install

WebFluent is one binary, `wf`. Run the line for your machine. Each one
leaves `wf` on your `PATH`.

### macOS and Linux

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
```

The script fetches the binary for your platform from the latest release.
You do not need Rust. It covers Apple Silicon and Intel Macs and x86_64
Linux.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex
```

> [!IMPORTANT]
> On Windows, type `webfluent` wherever this guide types `wf`.

Windows already has a `wf`: `C:\Windows\System32\WF.msc`, the Windows
Firewall console. System32 comes first on `PATH`, and `.MSC` is one of the
extensions Windows tries for a bare name, so typing `wf` opens the firewall
instead of WebFluent. For that reason the release zip, `install.ps1` and the
`.msi` install the compiler as `webfluent.exe` beside `wf.exe`, and the
`.msi` puts their folder on `PATH`. `webfluent build`, `webfluent serve` and
`wf.exe build` all work. After `cargo install webfluent`, use `wf.exe`.

### With Rust

```bash
cargo install webfluent
```

This builds `wf` from crates.io. It works everywhere Rust does, and it is
how to install on Linux arm64, which has no prebuilt binary yet.

### Packages

Each [release](https://github.com/monzeromer-lab/WebFluent/releases) also
carries a `.deb`, an `.msi` and plain tarballs, for a machine where piping
a script to a shell is not an option.

## Check the install

Ask `wf` for its version:

```bash
wf --version
```

**You see** the release you installed:

```text
wf 6.0.0
```

If the shell says it cannot find `wf`, open a new terminal. The installer
adds its directory to your `PATH` in your shell's startup file, and the
terminal you ran it in read that file before the change.

## Create a project

Make a project called `hello` and move into it:

```bash
wf init hello
cd hello
```

**You see:**

```text
Created new WebFluent project: hello (template: spa)
  AGENTS.md is the language reference, for you and your coding agent
```

`wf init` writes a small application so you have something real to read:
four pages (a dashboard, a task list, a profile and settings), three
components and two stores, with tests under `tests/`.

## Run the dev server

Start the dev server:

```bash
wf serve
```

**You see:**

```text
Building hello...
  4 pages, 3 components, 2 stores
  Output: ./build/
Build complete.
Starting dev server at http://localhost:3000/
```

Open `http://localhost:3000` in your browser. **You see** the app's
**Dashboard** page: a navigation bar, three task counts and a list of recent
tasks.

Leave the server running. It rebuilds when you save a file, reloads the page
when a build lands, and, when a build fails, draws the error over the page
until a save fixes it.

## Make a change

1. Open `src/pages/Home.wf` in your editor.
2. Find the line `Text("Welcome to your dashboard.").muted` and change the
   text to `Text("Welcome back.").muted`.
3. Save the file.

**You see** the browser reload by itself, and the dashboard now reads
**Welcome back.** under its heading.

To see what an error looks like, change `.muted` to `.mutd` and save. The
page shows error `E103` over it, with the file, the line and the hint
“Did you mean `.muted`?”.
Change it back, save, and the page returns.

You have a working project. Stop the server with Ctrl+C when you are done.
Run `wf build` to write the finished site to `build/`, ready for any static
host ([Deploying](../recipes/index.md)).

The rest of this chapter is optional set-up. To keep learning, go to the
[tutorial](02-tutorial-shelf.md).

## Set up your editor

The language server, `wf-lsp`, gives your editor the same checks `wf build`
runs, as you type: diagnostics, completion of props, flags and cases, hover
with types and documentation, signature help, go to definition, references,
rename, formatting, inlay types, semantic highlighting and quick fixes.

`wf-lsp` is a second binary. Get it from the same
[release](https://github.com/monzeromer-lab/WebFluent/releases) as `wf` —
`wf-lsp-<version>-<arch>-<os>.tar.gz` — and put it on your `PATH`, or build
it from a clone of the repository:

```bash
cargo install --path crates/wf-lsp
```

Then connect your editor:

- **VS Code** — the extension, [webfluent-vscode](https://github.com/monzeromer-lab/webfluent-vscode), brings highlighting and
  the language server. Until it is on the Marketplace, install the `.vsix`
  from its latest release with **Extensions: Install from VSIX**.
- **Zed** — the extension, [webfluent-zed](https://github.com/monzeromer-lab/webfluent-zed), brings Tree-sitter highlighting,
  the outline, snippets and the language server (found on `PATH`, or
  downloaded from the latest release). Install it with
  **zed: install dev extension**, pointed at a clone of it.
- **Neovim, Helix, Emacs, or any other editor with LSP** — run `wf-lsp`
  over stdio for the `webfluent` file type (`*.wf`, `*.wfx`).
  [Editors](../reference/cli.md#editors) has the configuration for each.

### WebFluent Studio

[WebFluent Studio](https://github.com/monzeromer-lab/webfluent-studio) is
an app made for writing WebFluent. It puts a code editor, a live preview, a
visual designer, Git and GitHub, this guide and an agent in one window. It
does not use `wf-lsp`: the compiler runs inside the app, so it needs no
`wf` on your `PATH`, no terminal and no dev server. A project built in
Studio is byte-identical to the same project built with `wf build`, so you
can move between Studio and the command line at any time.

- **The preview** rebuilds as you type, and the build log and console link
  back to the source.
- **Diagnostics** have the compiler's own message and its fix, one click
  away; **Explain** opens the chapter of this guide for that code.
- **Design mode** edits the page on a canvas and writes the `.wf` you would
  have written.
- **The agent** builds its own change before you see it, then you accept or
  reject it one hunk at a time.

Studio is a separate, closed-source app at version 0.1, for Linux, macOS
and Windows; Android follows. Its builds are published on
[its releases page](https://github.com/monzeromer-lab/webfluent-studio/releases).
You do not need it to use WebFluent: `wf` and any editor are the whole
toolchain.

## Change how `wf` is installed

Both installers read two environment variables:

| Variable | Effect | Example |
|---|---|---|
| `WF_INSTALL_DIR` | Installs the binary somewhere other than `~/.webfluent/bin` | `WF_INSTALL_DIR=/opt/wf/bin` |
| `WF_VERSION` | Installs that release instead of the latest | `WF_VERSION=v6.0.0` |

Running the installer again upgrades `wf` in place.

To build from a clone instead — for an unreleased change, or to work on the
compiler itself — run:

```bash
git clone https://github.com/monzeromer-lab/WebFluent.git
cd WebFluent && cargo install --path .
```

## Choose a template

`wf init` takes a template with `-t`:

| Template | Makes |
|---|---|
| `-t spa` (default) | An interactive application, routed in the browser |
| `-t static` | A marketing site, every page pre-rendered to HTML |
| `-t pdf` | A PDF document |
| `-t slides` | A PDF slide deck |

`--locales en,ar` starts an `spa` or `static` project translated: its text
is in `src/translations/`, one file per language, and the first locale is
the default ([Internationalisation](../recipes/translate-a-site.md#start-a-project-translated)).

If you are unsure between `spa` and `static`, start with `static`. Every
page is real HTML before any script runs, which is better for search and for
the first paint, and the page is still fully interactive once the script
loads. [Static or single-page](../about/static-and-spa.md) explains the
difference.

## Write a site from scratch

To see the smallest whole site, replace every file in `src/` except
`theme.wf` with the two below, and delete the starter's tests under
`tests/`, which test the pages you removed. Keep `src/theme.wf`: the
project's `webfluent.app.json` names the themes it declares (`Day` and
`Night`), and the build stops if they are gone.

`src/App.wf` is the shell every page shares, with the router in it:

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

`src/pages/Home.wf` holds the pages, and each page names its own route:

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

Save, and the dev server shows a navigation bar with **Home** and
**About**. **Click me** counts your clicks. Here is what each part does:

- `app { … }` is the shell: what every page sits inside. `Router` is where
  the current page renders. There is no route table; each `page` names its
  own `path:`.
- `page Home(path: "/", title: …, description: …)` declares a page. `title`
  and `description` become the `<title>`, the meta description, the Open
  Graph and Twitter cards and the JSON-LD. If you leave them out, the build
  warns you.
- `state count = 0` is a signal. `Text("… {count} …")` reads it, so the text
  updates when it changes.
- In `Button("Click me").primary { on click { … } }`, the label is the one
  positional argument, `.primary` is a flag (a case of the button's `tone`
  prop), and the block holds the click handler.
- `Container`, `Heading`, `Text`, `Paragraph`, `Navbar` and `Link` are
  built-ins. The [components reference](../reference/components.md) lists
  all of them.

> [!NOTE]
> **Coming from React?** There is no route table and no `import`. Each
> `page` names its own `path:`, and every file under `src/` is part of one
> program, so `App.wf` places pages and components it never imports. See
> [Coming from React](../coming-from/react.md).

## What a build produces

`wf build` writes the finished site to `build/`. A static build with two
pages and a `path: "*"` page writes files like these:

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

Do not edit anything in `build/`: the next build writes it again. Keep it
out of version control; the `.gitignore` that `wf init` writes already
does. The build copies `public/` to the root of `build/` as it is.

`app.js` carries only the parts of the runtime the program reaches. The
runtime is a set of feature modules — `each` for a `for`, `router` for a
second page, `net` for a `resource`, `icons` for an `Icon` — and the build
works out which ones the code it has written calls, then keeps only the
functions of those that the calls reach, with their names shortened. A
page of static text is an `app.js` of 1.9 kB gzipped. `wf build --stats`
shows what a build weighs, and [Performance](../about/performance.md)
explains how to read it.

## The project, in brief

A project's files are laid out like this:

```text
AGENTS.md             the language reference, for you and a coding agent
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

The compiler reads every `.wf` or `.wfx` file under `src/` and merges them
into one program, so a page in one file uses a component from another
without importing it. The folders are a convention, not a rule.
[Project structure](../reference/project-structure.md) covers naming, organisation
and the config, and the [configuration reference](../reference/configuration.md)
lists every key.

The config needs only a name:

```json
{ "name": "hello" }
```

## Common mistakes

A flag with a typo. A flag is checked against the element that takes it,
so `.mutd` is an error, not a style that quietly does nothing:

```wf expect=E103
page Home(path: "/", title: "Hello", description: "A first site.") {
    Heading("Hello").h1
    Text("Welcome back.").mutd
}
```

Use one of the flags the message lists, here `.muted`
([E103](../errors/E103.md)).

An `app` with no `Router`. The shell is drawn, but no page ever is:

```wf expect=R03
app {
    Navbar {
        Navbar.Brand { Text("Hello").heading }
    }
}

page Home(path: "/", title: "Hello", description: "A first site.") {
    Heading("Hello, world").h1
}
```

Put `Router` in the `app` where the current page should appear
([R03](../errors/R03.md)).

A link to a page you have not written yet. The compiler checks every
`to:` against the pages' paths:

```wf expect=R01
app {
    Navbar {
        Navbar.Links {
            Link("Home", to: "/")
            Link("About", to: "/about")
        }
    }
    Router
}

page Home(path: "/", title: "Hello", description: "A first site.") {
    Heading("Hello, world").h1
}
```

Add the page, `page About(path: "/about", …)`, or fix the address
([R01](../errors/R01.md)).

## Next

[Tutorial: Shelf](02-tutorial-shelf.md): build a reading-list site, start to finish.
