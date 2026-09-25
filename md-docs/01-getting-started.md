# 1. Getting started

By the end of this chapter you will have installed the compiler, built a
two-page site, changed it with the dev server watching, and know where every
file of a project lives.

## Install

WebFluent is one binary, `wf`. Run the line for your machine; each one
leaves `wf` on your `PATH`.

### macOS and Linux

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
```

Fetches the binary for your platform from the latest release — no Rust
needed. Covers Apple Silicon and Intel Macs, and x86_64 Linux.

### Windows

```bash
irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex
```

### With Rust

```bash
cargo install webfluent
```

Builds from crates.io. This is the one that works everywhere Rust does,
and it is how to install on Linux arm64, which has no prebuilt binary
yet.

### Check it

```bash
wf --version
```

If the shell cannot find `wf`, open a new terminal: the installer adds
its directory to your `PATH` in your shell's startup file, which the
session you ran it in has already read.

Both installers take two environment variables — `WF_INSTALL_DIR` to put
the binary somewhere other than `~/.webfluent/bin`, and `WF_VERSION`
(`v3.2.1`) to pin a release rather than take the latest.

To build from a clone instead, for an unreleased change or to work on
the compiler itself:

```bash
git clone https://github.com/monzeromer-lab/WebFluent.git
cd WebFluent && cargo install --path .
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
  [chapter 20](20-components-reference.md) lists all of them.

Save, and the dev server shows it. `wf build` writes the finished site to
`build/`.

## What a build produces

```text
build/
  index.html          the page, pre-rendered (with build.ssg) or the app shell
  about/index.html    one directory per route in a static build
  app.js              the runtime this program needs, stores and components
  pages/Home.js       each page in its own chunk, loaded when its route shows
  pages/Home.css      the styles only that page reaches
  styles.css          the shared sheet: your tokens, the built-ins you used, your stylesheets
  sitemap.xml, robots.txt
  *.gz                pre-compressed copies a host can serve as-is
```

Nothing in `build/` is hand-editable; it is regenerated on every build.

`app.js` carries only the parts of the runtime the program reaches. The
runtime is a set of feature modules — `each` for a `for`, `router` for a
second page, `net` for a `resource`, `icons` for an `Icon` — and the build
works out which ones the code it just wrote calls into. A page of static
text carries 19 kB of runtime where the whole of it is 107 kB.

`wf build --stats` shows it:

```text
  Runtime: 4 of 28 modules, 24.9 kB of 107.2 kB (before minifying)
    kept     core pages store icons
    left out motion helpers format form text when match slot each show …
```

The one thing a build cannot see is a name assembled at run time — an icon
whose name arrives from an API, or `WF.*` called from a hand-written
`<script>`. `"build": { "runtime": "full" }` ships every module for that.

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

`webfluent.app.json` — every key is optional except `name`. What is shown
here is the default, so a key you never write behaves as it reads:

```json
{
  "name": "hello",
  "version": "1.0.0",
  "author": "",
  "build": {
    "output": "./build",
    "output_type": "spa",
    "ssg": false,
    "split": true,
    "minify": true,
    "sourcemap": false,
    "compress": true,
    "runtime": "auto",
    "csp": false,
    "base_path": "",
    "budget": {},
    "elements": [],
    "media": { "formats": ["webp"], "widths": [480, 960, 1440, 1920], "quality": 78, "pipeline": true }
  },
  "dev": { "port": 3000, "hot_reload": true },
  "motion": { "duration": null, "easing": null },
  "meta": {
    "title": "", "description": "", "lang": "en",
    "site_url": "", "site_name": "", "image": "", "favicon": "",
    "sitemap": true, "fonts": [], "stylesheets": [], "integrity": {}
  },
  "theme": { "name": null, "dark": null, "builtin": "full", "tokens": {} },
  "i18n": { "default_locale": "en", "locales": ["en"], "dir": "src/translations" },
  "env": {},
  "public_env": []
}
```

### The project

| Key | Default | Meaning |
|---|---|---|
| `name` | — | The only required key. The site's name, and the PDF's filename where none is given. |
| `version`, `author` | `"1.0.0"`, `""` | Yours to use; nothing in the build reads them. |

### `build`

| Key | Default | Meaning |
|---|---|---|
| `output` | `"./build"` | Where the build is written, relative to the project. |
| `output_type` | `"spa"` | `spa`, `pdf`, `slides` or `elements` ([chapter 17](17-outputs.md)). |
| `ssg` | `false` | Pre-render every page to its own HTML ([chapter 17](17-outputs.md)). Off, the build is one shell. |
| `split` | `true` | One JS and CSS chunk per page, loaded when its route shows. |
| `minify` | `true` | Strip comments and whitespace from the bundle and the sheet. |
| `sourcemap` | `false` | Write a source map beside the bundle. |
| `compress` | `true` | Write a `.gz` beside every text output over a kilobyte, for a host that serves it. |
| `runtime` | `"auto"` | `auto` ships the runtime modules the program reaches; `full` ships all of them, for a program the scan cannot see (a `WF.*` call in a hand-written script). |
| `csp` | `false` for an existing project, `true` for one `wf init` writes | Emit the Content-Security-Policy, and hold the output to it ([chapter 19](19-security.md#content-security-policy)). |
| `base_path` | `""` | Serve from a sub-path, such as `/docs` on GitHub Pages. |
| `budget` | `{}` | A gzipped size each named output should stay under: `{ "app.js": "40 kB" }`. Over it the build warns; it never fails. |
| `elements` | `[]` | With `output_type: "elements"`, the components to publish as custom elements ([chapter 8](08-components.md#publishing-yours)). |
| `media` | see above | The image pipeline: the `formats` and `widths` to write, the `quality` to encode at, and `pipeline: false` to copy files as `public/` always did ([chapter 16](16-content.md)). |
| `pdf`, `slides` | see [chapter 17](17-outputs.md) | Page size, margins, fonts, and for a deck the slide size, chrome and colours. |

### `dev`, `motion`

| Key | Default | Meaning |
|---|---|---|
| `dev.port` | `3000` | The port `wf serve` listens on. |
| `dev.hot_reload` | `true` | Rebuild on a save, reload every open tab, and draw a failed build's error over the page. |
| `motion.duration` | unset | How long an animation runs when the element does not say: `"180ms"`. It is the `animation-duration-normal` token ([chapter 13](13-motion.md#the-defaults)). |
| `motion.easing` | unset | How it is paced: a CSS timing function, or `"$ease-standard"` to take a token's value. |

### `meta`

| Key | Default | Meaning |
|---|---|---|
| `title`, `description` | `""` | What a page falls back to when it declares none. |
| `lang` | `"en"` | The document's language. An RTL locale sets `dir="rtl"` on its own. |
| `site_url` | `""` | The site's absolute address. **Without it the tags that need one are left out rather than guessed at** — a wrong canonical URL costs more than a missing one ([chapter 16](16-content.md)). |
| `site_name`, `image` | `""` | The name and the sharing image a link preview shows. |
| `favicon` | `""` | The icon a tab shows. |
| `sitemap` | `true` | Write `sitemap.xml` and `robots.txt`. |
| `fonts` | `[]` | Web-font stylesheets to link, each with a `preconnect` to its origin. |
| `stylesheets` | `[]` | Extra stylesheets to link ahead of `styles.css` — a file in `public/`, or a URL. For what no element-level `style { }` can say. |
| `integrity` | `{}` | The subresource-integrity hash of an external asset, by its URL ([chapter 19](19-security.md#another-origins-files)). |

### `theme`, `i18n`, `env`

| Key | Default | Meaning |
|---|---|---|
| `theme.name` | unset | Which `theme` declaration to use. A project with exactly one need not name it. |
| `theme.dark` | unset | The theme that stands in under `prefers-color-scheme: dark`, and when the reader asks ([chapter 12](12-styling.md)). |
| `theme.builtin` | `"full"` | `full` ships the baseline design of the built-ins; `structural` only their layout and mechanics, for a project supplying its own. |
| `theme.tokens` | `{}` | Token overrides on top of the theme, for a value a machine supplies. |
| `i18n.default_locale`, `i18n.locales`, `i18n.dir` | `"en"`, `["en"]`, `"src/translations"` | Translations ([chapter 15](15-i18n.md)). |
| `env` | `{}` | Values read in code as `env.NAME`, fixed at build time. |
| `public_env` | `[]` | The `env` names a page may read, beyond every name beginning `PUBLIC_`. **Anything a page reads is in the bundle**, so this list is the difference between a setting and a leak ([chapter 19](19-security.md#env-and-what-ends-up-in-the-bundle)). |

## Next

[Language basics](02-language-basics.md) — the shape of a file, and the two
ways to write blocks.
