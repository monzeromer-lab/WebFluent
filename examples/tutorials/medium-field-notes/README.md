# Field Notes

A personal notebook site — notes from a garden, a kitchen and a few trips —
built with WebFluent as a fully static site. There is no backend: the notes
are a JSON file read when the site is built, every page is pre-rendered to
HTML, and the one thing a reader can change (the notes they save for later)
is kept in their own browser.

It is the finished project of the second tutorial in the guide's
[Tutorials chapter](../../../md-docs/35-tutorials.md#medium-field-notes),
which builds it step by step.

## What it teaches

| Idea | Where |
|---|---|
| A static project: `ssg`, `csp`, `site_url` and the rest of `meta` | `webfluent.app.json` |
| A theme, a dark theme, and a toggle built on `setTheme` | `src/theme.wf`, `src/components/ThemeToggle.wf` |
| A typed `data` file, with a `Date` field | `src/types.wf`, `src/notes.json` |
| The app shell (`Navbar`, `Router`, `Footer`) and a `layout:` component | `src/App.wf`, `src/components/Sheet.wf` |
| Reusable components, props with defaults, `///` docs | `src/components/NoteCard.wf`, `TagList.wf`, `SaveButton.wf` |
| A filter that lives in the URL (`query.tag`) and a search box (`state`, `derived`), with an empty state | `src/components/NoteFinder.wf`, `src/pages/Home.wf` |
| What the static paint can and cannot know, and drawing a list so the HTML holds every note | `src/components/NoteFinder.wf` |
| A `:slug` route rendered once per note with `paths:`, a computed `title:`, `head { }` tags and `Markdown` | `src/pages/NotePage.wf` |
| A tags index, and a page written as a `.md` file with front matter | `src/pages/Tags.wf`, `src/about.md` |
| A store with a `persist`ed list, a Saved page and a count in the navbar | `src/stores/Saved.wf`, `src/pages/SavedPage.wf`, `src/App.wf` |
| A project stylesheet that reads the theme's tokens | `src/styles.css` |
| Tests that render, and tests that type and click in a browser | `tests/` |

## Run it

You need `wf` installed. From this folder:

```bash
wf serve      # http://localhost:3000 — rebuilds and reloads on save
wf build      # writes build/: one HTML file per page, plus sitemap.xml and robots.txt
wf test       # renders the components and clicks through them (needs Chrome)
wf verify     # loads every built page in headless Chrome (run after wf build)
```

`wf check --deny-warnings` runs every check without writing anything.

The tests that type or click run in a headless Chrome. If yours is not on
the `PATH`, point `WF_CHROME` at it:

```bash
WF_CHROME=/usr/bin/google-chrome wf test
```

## Layout

```
webfluent.app.json        theme, build, meta (site_url, favicon, sharing image)
public/                   favicon.svg and card.png, copied to the site's root
src/
├── App.wf                the shell: navbar, Router, footer
├── theme.wf              Meadow (light) and Dusk (dark)
├── types.wf              type Note, and `data notes` read from notes.json
├── notes.json            the notes themselves, each body in Markdown
├── about.md              the About page, written in Markdown
├── styles.css            the project's own stylesheet
├── components/           Sheet (the layout), NoteFinder, NoteCard, TagList, SaveButton, ThemeToggle
├── pages/                Home, NotePage, Tags, SavedPage, NotFound
└── stores/Saved.wf       the reader's saved notes, kept in their browser
tests/                    NoteCard.wf, NoteFinder.wf (and their snapshots)
```

## Adding a note

Add a record to `src/notes.json` — a unique `slug`, a `title`, a `date`
(`"2026-10-04"`), some `tags`, a one-line `summary`, the `minutes` it takes
to read and a Markdown `body` — and build. The note gets its own page at
`/notes/<slug>/`, a card on the front page, a place under each of its tags
and an entry in the sitemap. The type checker holds every record to
`type Note`, so a misspelt field is an error at build time, not a blank on
the page.
