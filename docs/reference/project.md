# Project structure

> Where files go, how they merge into one program, naming in a global namespace, wf generate, what to commit, and how to grow a project.

The page: https://webfluent.monzeromer.dev/reference/project

A WebFluent project is a config file and a folder of sources that the
compiler reads as one program. The folders are a convention for you and
your team; the compiler does not care where a declaration lives. This
chapter explains how a project is laid out and read, then shows you how to
add files, choose what to commit and keep a growing project in order.

In this chapter:

- the conventional layout, and which files are required;
- how the compiler merges every file into one program, and what that means
  for names;
- what happens to the files that are not `.wf`;
- how to generate starter files, what to commit, and how to organise a
  project as it grows.

## The layout

This is the layout `wf init` starts from, with every kind of file a project
can hold:

```text
my-site/
├── AGENTS.md               the language reference, for a coding agent (`wf init` writes it)
├── webfluent.app.json      the config (see Configuration)
├── .env                    build-time values, not committed (see Environments)
├── src/
│   ├── App.wf              the app shell and the Router
│   ├── theme.wf            themes
│   ├── types.wf            shared types, enums, constants, data
│   ├── api.wf              services
│   ├── pages/              one page per file
│   ├── components/         components
│   ├── stores/             stores
│   ├── styles.css          your own stylesheets, anywhere under src/
│   ├── tilt.js             your own scripts, anywhere under src/
│   ├── translations/       en.json, ar.json (with "i18n" in the config)
│   ├── posts.json          data files a `data` declaration reads
│   └── about.md            Markdown pages
├── public/                 copied to the build's root: favicon, fonts, images
├── tests/                  `test` declarations, and __snapshots__/
└── build/                  what `wf build` writes (not committed)
```

Only two things are required: `webfluent.app.json` and one `.wf` or `.wfx`
file under `src/`. Everything else is convention. The config is covered in
[Configuration](https://webfluent.monzeromer.dev/reference/config.md), and `.env` in
[Environments and secrets](https://webfluent.monzeromer.dev/recipes/environments-and-secrets.md).

## One program, with no imports

The compiler reads every `.wf` and `.wfx` file under `src/` and merges them
into a single program. A page uses a component declared in another file, and
a component uses a store, by name. There is nothing to import.

For example, a component declared in `src/components/PriceCard.wf` and a
page in `src/pages/Pricing.wf` work together with no import between them.
Here they are in one block:

```wf
// In src/components/PriceCard.wf
component PriceCard(_ plan: String, price: Number) {
    Card {
        Text(plan).bold
        Text(format(price, .currency))
    }
}

// In src/pages/Pricing.wf
page Pricing(path: "/pricing", title: "Pricing", description: "Plans and prices.") {
    Heading("Pricing").h1
    PriceCard("Team", price: 49)
}
```

The compiler reads `App.wf` (or `App.wfx`) first, then the rest in path
order. The order matters for nothing but the order of equal things in the
output.

`use Store` at the top of a page or component says that it reads that
store. It documents the dependency for the reader and the editor; it does
not scope anything, and a page reads a store by name with or without it.

## Names across a project

Because every file is one program, the namespace is global. Two
components, two pages, two stores, two types or two constants with one name
is an `E102` error, naming both places.

The conventions that keep names apart:

- **Declared names are PascalCase**: pages, components, stores, types,
  enums, themes, services and animations.
- **Constants are UPPER_SNAKE** by convention: `API`, `PAGE_SIZE`.
- **A page's name is its identity, not its title.** Add a suffix when the
  name would collide with a component: `page PostPage` beside
  `component Post`.
- **A component that one page uses** can live in that page's file.

## Files that are not `.wf`

The build reads other files too. What each becomes:

| File | Where | Becomes |
|---|---|---|
| `*.css` | anywhere under `src/` | bundled into `styles.css`, in path order, after the built-ins' rules |
| `*.js` | anywhere under `src/` | copied to `js/` as written and linked on every page before the compiled code; its top-level names are in scope ([Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)) |
| `*.md` | under `src/` | a page ([Content](https://webfluent.monzeromer.dev/recipes/markdown-pages.md)) |
| `*.json` | named by a `data` declaration | a constant, read at build time |
| images | named by an `image` declaration | resized, hashed files and a `<picture>` ([Add a picture](https://webfluent.monzeromer.dev/recipes/add-a-picture.md)) |
| `translations/*.json` | the `i18n.dir` | the locale tables ([Translate a site](https://webfluent.monzeromer.dev/recipes/translate-a-site.md)) |
| anything | `public/` | copied as it is to the build's root: `public/favicon.svg` is `/favicon.svg` |

## Generate a starter file

To add a page, a component or a store with the boilerplate written for
you, run `wf generate`:

```bash
wf generate page Pricing
wf generate component PriceCard
wf generate store Cart
```

Each command writes a starter declaration in the conventional folder:

| Command | Writes |
|---|---|
| `wf generate page Pricing` | `src/pages/Pricing.wf`, with `path: "/pricing"`, a title, a description and an `h1` |
| `wf generate component PriceCard` | `src/components/PriceCard.wf` |
| `wf generate store Cart` | `src/stores/cart.wf` |

In a project written in `.wfx`, the files are `.wfx`. `wf generate` never
overwrites a file: when one exists, it says so and stops. Add `-d DIR` to
generate into a project other than the current directory.

## Choose what to commit

Commit the sources and the config:

- `webfluent.app.json`
- `src/`
- `public/`
- `tests/`, with its `__snapshots__/`

Leave out what the build writes and what is private. Add these lines to
`.gitignore`:

```text
build/
.wf-cache/
.wf-sizes.json
.env
```

`.wf-cache/` holds resized images between builds. `.wf-sizes.json` records
what the last build weighed, for `wf build --stats` to compare against.
`.env` holds build-time values that may be private.

## Organise a growing project

As a project grows, these habits keep it readable:

- **Group by feature once folders get long.** A `src/billing/` folder that
  holds the billing pages, components, store and types reads better than
  four folders that each hold a slice of it. The compiler does not mind
  either way.
- **Put each service in one place.** Write one `api` per backend, in its
  own file, with the types its endpoints return beside it.
- **Keep shared types together**, in a `types.wf` per area, so the
  editor's go-to-definition lands somewhere predictable.
- **Document as you declare.** A `///` comment above a component, a prop,
  an event or a type shows on hover and in `wf docs`, the gallery of
  everything the project declares.
- **Hold every file to the formatter.** `wf fmt --check` in CI keeps every
  file in one layout ([Testing](https://webfluent.monzeromer.dev/learn/16-testing.md#run-the-checks-in-ci)).

## Quick reference

| Topic | Rule |
|---|---|
| Required files | `webfluent.app.json` and one `.wf` or `.wfx` file under `src/` |
| Reading order | `App.wf` first, then the rest in path order |
| Imports | none: every declaration is in scope by name |
| `use Store` | documents a dependency; scopes nothing |
| Duplicate names | `E102`, naming both places |
| `public/` | copied to the build's root as it is |
| Starter files | `wf generate page`, `component` or `store`, then the name |
| Not committed | `build/`, `.wf-cache/`, `.wf-sizes.json`, `.env` |

## Next

- [Static or single-page](https://webfluent.monzeromer.dev/about/static-or-single-page.md): choose how the project
  builds.
- [CLI](https://webfluent.monzeromer.dev/reference/cli.md): every `wf` command.
