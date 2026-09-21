# 1. Getting started

By the end of this chapter you will have installed the compiler, built a
two-page site, changed it with the dev server watching, and know where every
file of a project lives.

## Install

WebFluent is one binary, `wf`.

```bash
# Linux and macOS
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex

# From source, with Rust installed
git clone https://github.com/monzeromer-lab/WebFluent.git
cd WebFluent && cargo install --path .
```

Check it:

```bash
wf --version
```

The language server for your editor is a second binary, `wf-lsp` — see
[Tooling](18-tooling.md#editors) for Zed and VS Code.

## A first project

```bash
wf init hello
cd hello
wf serve
```

`wf init` writes a small application (a dashboard, a task list, a settings
page, a store, a theme) so you have something to read; `wf serve` builds it,
opens it at `http://localhost:3000`, rebuilds when you save, reloads the page
when a build lands, and — when a build fails — draws the error over the page
until the next save fixes it.

Templates: `wf init name -t spa` (the default, an interactive app), `-t static`
(a marketing site, pre-rendered), `-t pdf` (a document), `-t slides` (a deck).

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

`src/pages/Home.wf` — a page that owns its route:

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

- `app { … }` is the shell: what every page sits inside. `Router` is where the
  current page renders. There is no route table — each `page` names its own
  `path:`.
- `page Home(path: "/", title: …, description: …)` declares a page. `title` and
  `description` become the `<title>`, the meta description, the Open Graph and
  Twitter cards and the JSON-LD; leave them out and the build tells you.
- `state count = 0` is a signal. `Text("… {count} …")` reads it, so the text
  updates when it changes.
- `Button("Click me").primary { on click { … } }` — the label is the one
  positional argument, `.primary` is a flag (a case of the button's `tone`
  prop), and the block holds the click handler.
- `Container`, `Heading`, `Text`, `Paragraph`, `Navbar`, `Link` are built-ins;
  [chapter 19](19-components-reference.md) lists all of them.

Save, and the dev server shows it. `wf build` writes the finished site to
`build/`.

## What a build produces

```text
build/
  index.html          the page, pre-rendered (with build.ssg) or the app shell
  about/index.html    one directory per route in a static build
  app.js              the runtime, stores and components
  pages/Home.js       each page in its own chunk, loaded when its route shows
  pages/Home.css      the styles only that page reaches
  styles.css          the shared sheet: your tokens, the built-ins you used, your stylesheets
  sitemap.xml, robots.txt
  *.gz                pre-compressed copies a host can serve as-is
```

Nothing in `build/` is hand-editable; it is regenerated on every build.

## Project layout

```text
webfluent.app.json    the config (below)
src/
  App.wf              the app shell (read first; optional)
  pages/*.wf          pages — any file name, any nesting
  components/*.wf     components
  stores/*.wf         stores
  theme.wf            a theme, if you declare one
  translations/*.json translation files, with "i18n" in the config
  *.css               stylesheets, bundled into styles.css
  *.md                Markdown pages (chapter 16)
public/               copied to build/ as-is (images, fonts, favicon)
tests/*.wf            `test "…" { }` declarations for `wf test`
```

The compiler reads every `.wf` (or `.wfx`) under `src/` and merges their
declarations into one program: a page in one file can use a component from
another without importing anything. Names are global to the project, so two
declarations of the same name is an error.

## The config file

`webfluent.app.json` — every key is optional except `name`:

```json
{
  "name": "hello",
  "version": "0.1.0",
  "build": {
    "output": "build",
    "ssg": true,
    "split": true,
    "minify": true,
    "compress": true,
    "csp": true,
    "base_path": "",
    "output_type": "spa"
  },
  "dev": { "port": 3000, "hot_reload": true },
  "meta": {
    "title": "Hello",
    "description": "A first WebFluent site.",
    "lang": "en",
    "site_url": "https://example.com",
    "site_name": "Hello",
    "image": "/og.png",
    "favicon": "/favicon.svg",
    "sitemap": true,
    "fonts": ["https://fonts.googleapis.com/css2?family=Inter&display=swap"],
    "stylesheets": ["/base.css"]
  },
  "theme": { "name": "Brand", "dark": "Night", "builtin": "full", "tokens": {} },
  "i18n": { "default_locale": "en", "locales": ["en", "ar"], "dir": "src/translations" },
  "env": { "API_URL": "https://api.example.com" }
}
```

| Key | Meaning |
|---|---|
| `build.ssg` | Pre-render every page to HTML at build time (chapter 16). Off, the build is a single-page app with one `index.html`. |
| `build.split` | One JS and CSS chunk per page, loaded when the route shows. |
| `build.minify`, `build.compress` | Minify the bundle and the sheet; write `.gz` copies beside them. |
| `build.csp` | Emit the strict content-security policy the output satisfies (`script-src 'self'`, no inline script or style). |
| `build.base_path` | Serve from a sub-path, such as `/docs` on GitHub Pages. |
| `build.output_type` | `spa`, `pdf` or `slides` (chapter 17). |
| `meta.*` | The site-wide title, description, language, canonical URL, sharing image, sitemap, web fonts and extra stylesheets (chapter 16). |
| `theme.name`, `theme.dark` | Which `theme` declaration the build uses, and which one stands in for dark mode (chapter 12). `theme.builtin`: `full` ships the baseline design of the built-ins, `structural` only their layout and mechanics. |
| `i18n` | Translation files and locales (chapter 15). |
| `env` | Values read in code as `env.API_URL` (chapter 10). |

## Next

[Language basics](02-language-basics.md) — the shape of a file, and the two
ways to write blocks.
