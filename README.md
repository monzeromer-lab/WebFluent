[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/monzeromer-lab/WebFluent)

# WebFluent

**One binary in, a website out.** Write `.wf`, get HTML, CSS and JavaScript — no framework, no `node_modules`, no build config. The same source also compiles to PDF documents and slide decks.

**[Docs](https://monzeromer-lab.github.io/WebFluent)** · **[Tutorial](https://monzeromer-lab.github.io/WebFluent/docs/guide/tutorial)** · **[Components](https://monzeromer-lab.github.io/WebFluent/docs/reference)** · **[Guide source](md-docs/)**

```wf
store Todos {
    state items = []

    action add(title: String) {
        items = items.concat([{ id: uuid(), title: title, done: false }])
    }
}

page Home(path: "/", title: "Todos") {
    use Todos
    state draft = ""

    Container {
        Heading("Todos").h1
        Row(gap: .sm) {
            Input(bind: draft, placeholder: "What needs doing?").text
            Button("Add").primary { on click { Todos.add(draft)  draft = "" } }
        }
        for t in Todos.items by t.id {
            Card { Text(t.title) }
        }
    }
}

app { Router }
```

That is the whole app — routing, reactivity, styling and the components included.

## Try it

**macOS and Linux** — the binary for your platform, no Rust needed:

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
```

<sub>**Windows:** `irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex` · **With Rust** (everywhere Rust runs, and the way to install on Linux arm64): `cargo install webfluent` · A `.deb`, `.msi` and tarballs are attached to each [release](https://github.com/monzeromer-lab/WebFluent/releases).</sub>

```bash
wf --version
wf init my-app        # -t spa (default) · static · pdf · slides
cd my-app
wf serve              # http://localhost:3000, rebuilds on save
```

`wf build` writes the finished site to `build/` — static files any host serves. [Deploying](md-docs/29-deploying.md) has GitHub Pages, Netlify, Vercel, Cloudflare and nginx.

<sub>If the shell cannot find `wf`, open a new terminal — the installer adds its directory to your `PATH` in your shell's startup file. `WF_INSTALL_DIR` installs elsewhere, `WF_VERSION` pins a release.</sub>

## What you get

- **Reactivity without a framework** — signals update the exact DOM node that changed. No virtual DOM, no diffing.
- **50+ components, already designed** — layout, navigation, tables, forms, modals, media. Styled by design tokens you can retheme in ten lines.
- **A compiler that actually checks your work** — types, props, flags, slots and events, plus lints for accessibility, SEO and dead code. A typo is an error with a line number, never a silent no-op.
- **Ships small** — the build keeps only the runtime modules your program reaches, splits a chunk per page, and precompresses everything. A page of static text ships under 4 kB of script, gzipped.
- **Four targets, one language** — SPA, pre-rendered static site, PDF, or a slide deck, by one config flag.
- **Batteries included** — routing, stores, forms and validation, `fetch` as a typed `api`, i18n with automatic RTL, animations, dark mode.
- **Editor support** — a language server for [Zed](editors/zed) and [VS Code](editors/vscode): completion, hover, go to definition, rename, quick fixes.

## A taste

<details>
<summary><b>Components, props and flags</b></summary>

```wf
/// A person, at a glance.
component UserCard(_ name: String, role: String, active: Bool = true) {
    Card.elevated {
        Row(align: .center, gap: .md) {
            Avatar(initials: "U").primary
            Stack {
                Text(name).bold
                Text(role).muted
            }
            if active { Badge("Active").success }
        }
    }
}

UserCard("Monzer", role: "Developer")
```

A `.flag` sets a boolean prop or picks an enum case. A flag the component does not declare is a compile error, with the ones it does take listed.
</details>

<details>
<summary><b>Data fetching</b></summary>

```wf
resource users = fetch("/api/users")

match users {
    loading      { Spinner }
    error(err)   { Alert("Failed to load").danger }
    ready(users) { for u in users by u.id { UserCard(u.name, role: u.role) } }
}
```

Describe a whole service once with `api`, and every call site is typed, cached, retried and cancellable.
</details>

<details>
<summary><b>Styling</b></summary>

```wf
Button("Save").primary.lg.rounded

Button("Custom") {
    style {
        background: $surface
        padding: $xl
        &:hover { background: $primary }
        @md { padding: $lg }
    }
}
```

`$token` is a design token, `{expr}` reads state. Any `.css` file under `src/` is bundled too.
</details>

<details>
<summary><b>Braces or indentation</b></summary>

A `.wf` file writes blocks in braces; a `.wfx` file writes them by indentation. Same grammar, same output — `wf fmt --to wfx` switches a project either way.

```wfx
page Home(path: "/", title: "Home")
    state count = 0
    Container
        Heading("Welcome").h1
        Button("+1").primary
            on click
                count = count + 1
```
</details>

<details>
<summary><b>PDFs and slides</b></summary>

```wf
page Deck(path: "/", title: "Q1 Review") {
    Presentation {
        TitleSlide("Q1 Review", subtitle: "March 2026")
        Slide {
            Heading("Highlights").h1
            List { Text("Revenue grew 15%")  Text("Expanded to 5 markets") }
        }
        SectionSlide("Q2 Plan").primary
    }
}
```

Set `"output_type": "slides"` (or `"pdf"`) and `wf build` writes the PDF. Raw PDF 1.7 bytes, no external dependencies — and interactive components are rejected at compile time rather than silently dropped.
</details>

## CLI

```
wf init <name> [-t spa|static|pdf|slides]   Create a project
wf build [-d DIR] [--stats]                 Compile; --stats prints what it weighs
wf serve [-d DIR]                           Dev server with live reload
wf generate page|component|store <name>     Scaffold a file
wf fmt [path] [--check] [--to wfx|wf]       Format, or switch layout
wf test [path] [--update]                   Run the project's tests, in a browser when one acts
wf verify [path] [--json] [--budget MS]     Visit every route in a headless browser
wf docs [-d DIR] [-o OUT]                   Write a component gallery
wf render <tpl> [--data f.json] [-f pdf]    Use WebFluent as a template engine
wf audit [path] [--json]                    What the project trusts
wf registry|types [--json]                  The registry, and what a project declares
wf migrate [path] [--check] [--wfx]         WebFluent 2 or 3 → 4
```

## Documentation

- **[The documentation](https://monzeromer-lab.github.io/WebFluent)** — built with WebFluent itself: getting started, a tutorial that ends in a deployed site, the language, and deploying
- **[Reference](https://monzeromer-lab.github.io/WebFluent/docs/reference)** — every component, [CLI command](md-docs/37-cli.md), [config key](md-docs/38-configuration.md), [diagnostic](md-docs/39-diagnostics.md) and [keyword](md-docs/43-grammar.md)
- **[Troubleshooting](md-docs/44-troubleshooting.md)** and **[upgrading](md-docs/45-upgrading.md)**
- **[The guide's source](md-docs/)** — the same chapters as Markdown; every code block in them is compiled by the test suite

## How it works

```
.wf source → Lexer → Parser → Type checker → Linters → Code generator → HTML + CSS + JS
                                                     → PDF validator   → PDF 1.7
```

The compiler is Rust. The generated JavaScript is a small signal-based runtime with no dependencies. PDF output is written byte by byte with Base14 font metrics.

To build the compiler from a clone — for an unreleased change, or to work on it:

```bash
git clone https://github.com/monzeromer-lab/WebFluent.git
cd WebFluent && cargo install --path .
```

Contributions welcome — see [the spec](spec/SPEC.md) for what the language promises.

## License

[GPL-3.0](LICENSE)
