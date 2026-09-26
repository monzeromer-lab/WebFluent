# 25. Project structure

<!--
route: guide/project-structure
group: shipping
blurb: Where files go, how the compiler reads them, how names work across a project, and how a project grows without getting lost.
description: Where files go, how they merge into one program, naming in a global namespace, wf generate, what to commit, and growing a project.
-->

## The layout

```text
my-site/
├── webfluent.app.json      the config (chapter 38)
├── .env                    build-time values, not committed (chapter 30)
├── src/
│   ├── App.wf              the app shell and the Router
│   ├── theme.wf            themes
│   ├── types.wf            shared types, enums, constants, data
│   ├── api.wf              services
│   ├── pages/              one page per file
│   ├── components/         components
│   ├── stores/             stores
│   ├── styles.css          your own stylesheets, anywhere under src/
│   ├── translations/       en.json, ar.json (with "i18n" in the config)
│   ├── posts.json          data files a `data` declaration reads
│   └── about.md            Markdown pages
├── public/                 copied to the build's root: favicon, fonts, images
├── tests/                  `test` declarations, and __snapshots__/
└── build/                  what `wf build` writes (not committed)
```

Only `webfluent.app.json` and one `.wf` or `.wfx` file under `src/` are
required. The folders are convention: the compiler does not care where a
declaration lives.

## One program, no imports

Every `.wf` and `.wfx` file under `src/` is read and merged into a single
program. A page uses a component declared in another file, and a component a
store, by name, with nothing to import. `App.wf` is read first; the rest in
path order, which matters for nothing but the order of equal things in the
output.

That makes the namespace global: two components, two pages, two stores, two
types or two constants with one name is an error, naming both places.

- **Names that declare** — pages, components, stores, types, enums, themes,
  services, externals, animations — are PascalCase.
- **Constants** are conventionally UPPER_SNAKE: `API`, `PAGE_SIZE`.
- **A page's name is its identity, not its title.** Suffix it when it would
  collide with a component: `page PostPage` beside `component Post`.
- **A component used by one page** can live in that page's file.

`use Store` at the top of a page or component says it reads that store. It
is documentation for the reader and the editor; it scopes nothing.

## Files that are not `.wf`

| File | Where | Becomes |
|---|---|---|
| `*.css` | anywhere under `src/` | bundled into `styles.css`, in path order, after the built-ins' rules |
| `*.md` | under `src/` | a page ([Content](27-content.md#md-pages)) |
| `*.json` | named by a `data` declaration | a constant, read at build time |
| images | named by an `image` declaration | resized, hashed files and a `<picture>` ([Media](28-media.md)) |
| `translations/*.json` | the `i18n.dir` | the locale tables |
| anything | `public/` | copied as it is to the build's root: `public/favicon.svg` is `/favicon.svg` |

## Starting files: `wf generate`

```bash
wf generate page Pricing
wf generate component PriceCard
wf generate store Cart
```

Each writes a starter declaration in the conventional folder —
`src/pages/Pricing.wf` with `path: "/pricing"`, a title, a description and an
`h1`; `src/components/PriceCard.wf`; `src/stores/Cart.wf` — in `.wfx` when
the project is written that way. It never overwrites a file.

## What to commit

Commit `webfluent.app.json`, `src/`, `public/`, `tests/` (with
`__snapshots__/`). Do not commit:

```text
build/
.wf-cache/
.wf-sizes.json
.env
```

`.wf-cache/` holds resized images between builds; `.wf-sizes.json` what the
last build weighed, for `wf build --stats` to compare against.

## As a project grows

- **Group by feature once folders get long.** `src/billing/` holding the
  billing pages, components, store and types reads better than four folders
  each holding a slice of it. The compiler does not mind either way.
- **Put a service in one place.** One `api` per backend, in its own file,
  with the types its endpoints return beside it.
- **Keep shared types together**, in a `types.wf` per area, so the editor's
  go-to-definition lands somewhere predictable.
- **Write docs as you declare.** A `///` comment above a component, a prop,
  an event or a type is shown on hover and in `wf docs`, the gallery of
  everything the project declares.
- **Hold it to the formatter.** `wf fmt --check` in CI keeps every file in one
  layout.

## Next

[Static or single-page](26-static-and-spa.md).
