# Build a statically generated notebook

> An intermediate WebFluent tutorial: a static notes site with data files, a page per note, Markdown, search, tags and saved notes, in about an hour.

The page: https://webfluent.monzeromer.dev/recipes/build-a-notebook-site

**What you will build.** A notebook kept in public — notes from a garden, a
kitchen and a few trips — built as a static site. The notes are a JSON file
the build reads. Every page, including one page per note, is written out
as HTML that a search engine can read. The one thing a reader changes, the
notes they save for later, stays in their browser. The finished project is
[`examples/tutorials/medium-field-notes`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/medium-field-notes).

**Time:** about an hour.

## Before you start

You need `wf`, Chrome and a copy of the repository
(see [Before you start](https://webfluent.monzeromer.dev/learn/03-tutorial-todos.md#before-you-start) in the Todos tutorial). This tutorial shows
the part of each file that teaches something new, so work in the finished
project and read each file as its step comes up:

```bash
cd WebFluent/examples/tutorials/medium-field-notes
wf serve
```

The Todos tutorial explains state, stores and components; this one
assumes them.

## Step 1: Create a static project and its look

A new project of this kind starts from the `static` template:

```bash
wf init field-notes -t static
cd field-notes
```

The project's `webfluent.app.json`:

```json
{
  "name": "field-notes",
  "version": "1.0.0",
  "author": "Field Notes",
  "theme": { "name": "Meadow", "dark": "Dusk" },
  "build": {
    "output": "./build",
    "ssg": true,
    "csp": true
  },
  "meta": {
    "title": "Field Notes",
    "description": "Notes from a small garden, a busy kitchen and the roads between them.",
    "lang": "en",
    "site_url": "https://field-notes.example.com",
    "site_name": "Field Notes",
    "favicon": "/favicon.svg",
    "image": "/card.png",
    "sitemap": true
  }
}
```

- `"ssg": true` pre-renders every page to its own `index.html`. When the
  page loads, the script takes it over in place — *hydration*
  ([Static sites and SPAs](https://webfluent.monzeromer.dev/about/static-or-single-page.md)).
- `site_url` lets the build write what search engines and link previews
  read: a canonical address, Open Graph tags, JSON-LD, `sitemap.xml` and
  `robots.txt` ([Content](https://webfluent.monzeromer.dev/recipes.md)).

The dark theme names only the tokens that change:

```wf
// src/theme.wf
/// The dark theme: only the tokens that change.
theme Dusk {
    color-primary: #86C9A6
    color-secondary: #E8A086
    color-background: #121614
    color-surface: #1A201D
    color-text: #E9E5DA
    color-text-muted: #A6AFA9
    color-border: #2B3430
    paper-raised: #171C19
    tag-bg: #213129
    tag-text: #A9DCC1
}
```

The light theme, `Meadow`, sets the same names. A theme may declare tokens
of its own, such as `paper-raised` and `tag-bg`. The project's stylesheet,
`src/styles.css`, reads every colour as a token (`var(--tag-bg)`), so the
dark theme needs no rule of its own ([Design tokens](https://webfluent.monzeromer.dev/reference/tokens.md)).

**Check it:** with `wf serve` running, set your system to dark mode. The
page turns dark green and cream without a reload.

## Step 2: Type the data

The notes are data, and a `type` says what each one holds:

```wf
// src/types.wf
/// One note in the notebook.
type Note {
    slug: String
    title: String
    date: Date
    tags: [String]
    summary: String
    minutes: Number
    body: String
}

/// Every note, read from `notes.json` when the site is built.
data notes: [Note] = "notes.json"
```

- `data` reads a JSON file at build time. The result is a constant, typed
  by the annotation, inlined into the bundle and seeded into every
  pre-rendered page ([Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md)).
- The checker holds the program to `Note`: a page that reads `note.titel`
  is an error where it is written.
- `date` is a `Date`: a plain `"2026-03-08"` string the language knows how
  to format. `{note.date:.date(long)}` shows `March 8, 2026`.

`src/notes.json` holds seven notes, each with a Markdown `body`.

**Check it:** `wf check` prints `No problems in field-notes.` If one note
in `notes.json` has a field that `Note` does not declare, the check reports
`T05` at the `data` line and names the field; if a note leaves out a field
`Note` requires, it reports `C01`.

## Step 3: Write the shell and a layout

The `app` declaration wraps every page. The current page is drawn where
`Router` stands, inside `<main>`, so the skip link each page carries jumps
past the navigation ([Pages and routing](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md)). The
start of `src/App.wf`:

```wf
// src/App.wf
app {
    use Saved

    Navbar(class: "site-nav") {
        Navbar.Brand {
            Link("Field Notes", to: "/", class: "brand")
        }
        Navbar.Links {
            Link("Notes", to: "/")
            Link("Tags", to: "/tags")
            Link(to: "/saved", class: "saved-link") {
                Text("Saved")
                if Saved.count > 0 {
                    Badge("{Saved.count}").pill
                }
            }
            Link("About", to: "/about")
        }
        Navbar.Actions {
            ThemeToggle
        }
    }

    Router(transition: .fade, duration: "160ms")
```

- The link whose `to:` is the page shown is marked `aria-current="page"`.
- `ThemeToggle` is two icon buttons that call `setTheme("dark")` and
  `setTheme("light")`. The reader's choice is kept across visits.
- `Router(transition: .fade, duration: "160ms")` fades one page out and
  the next in.

Each page names the column it is set in with `layout:` — a component whose
default slot is the page:

```wf
// src/components/Sheet.wf
/// The column a page is set in, and the frame every page names with
/// `layout: Sheet`. A narrow sheet is for reading; a wide one holds a grid.
component Sheet(wide: Bool = false) {
    slot
    Container(class: ["sheet", if wide { "sheet--wide" }]) {
        children
    }
}
```

**Check it:** the navbar shows **Notes**, **Tags**, **Saved** and
**About**. Click **Tags**: the page fades over, and the **Tags** link is
marked as the current page.

## Step 4: Build a card

A card shows a note at a glance:

```wf
// src/components/NoteCard.wf
/// A note at a glance: when it was written, its title, a summary, its tags
/// and the button that saves it.
component NoteCard(
    _ note: Note,
    /// Keeps the card in the page but out of sight — what a filter does.
    hidden: Bool = false
) {
    Card(class: "note-card", hidden: hidden) {
        Stack(gap: .sm) {
            Row(justify: .between, align: .center, gap: .sm) {
                Text("{note.date:.date(long)} · {note.minutes} min read", class: "note-meta").sm
                SaveButton(note.slug, title: note.title)
            }
            Link(to: "/notes/{note.slug}", class: "note-card__link") {
                Heading(note.title).h2
            }
            Text(note.summary, class: "note-card__summary")
            TagList(note.tags)
        }
    }
}
```

- A `///` comment above a component or a prop is its documentation. Your
  editor shows it when you hover a call ([Components](https://webfluent.monzeromer.dev/learn/10-components.md)).
- A splice may say how to show its value: `{note.date:.date(long)}` is
  `format(note.date, .date, "long")`, written where it is read
  ([Expressions](https://webfluent.monzeromer.dev/reference/expressions.md)).
- `TagList` is a row of `#tag` links to `/?tag=…`.

**Check it:** the front page shows a card per note, each with a line such
as `March 8, 2026 · 4 min read`, a title, a summary and its tags.

## Step 5: Search and filter the front page

The front page lists every note, narrowed by a search box and by the tag in
the address. `NoteFinder` works the list out:

```wf
// src/components/NoteFinder.wf
    state q = ""

    derived needle = q.trim().toLowerCase()
    derived newest = all.sortBy(n => n.date).reverse()
    derived shown = newest.filter(n =>
        (tag == "" || n.tags.includes(tag))
        && (needle == ""
            || n.title.toLowerCase().includes(needle)
            || n.summary.toLowerCase().includes(needle)))
    derived visible = shown.map(n => n.slug)
    derived allTags = all.flatMap(n => n.tags).unique().sort()
```

`q` is bound to the search box, and everything else follows it. `sortBy`,
`flatMap` and `unique` are list helpers the language brings. Each also runs
at build time, so the pre-rendered page shows the sorted list
([State and reactivity](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md)).

A static site adds one idea here. Its HTML is written once, before any
reader arrives, so the HTML cannot depend on what only the browser knows,
such as the address's query or what is in storage. So the finder draws
every note and hides the ones that do not match:

```wf
// src/components/NoteFinder.wf
            // Every note is drawn, and the ones that do not match are hidden.
            // A pre-rendered page cannot know the URL's query, so a list
            // filtered by it would be missing from the HTML; this way the
            // page carries every note, and the filter applies once it is live.
            for note in newest by note.slug {
                NoteCard(note, hidden: !visible.includes(note.slug))
            }
```

The HTML then holds every note, for a reader without script and for a
search engine, and the filter applies the moment the page is live. The
front page hands the finder the address's tag:

```wf
// src/pages/Home.wf
        // `query.tag` is the URL's `?tag=…`; it is missing when the URL has
        // none, and `?? ""` turns that into the finder's "every note".
        NoteFinder(notes, tag: query.tag ?? "")
```

**Check it:** the front page reads `7 of 7 notes`. Type `tomato` in the
search box, and the count and the cards narrow as you type. Open
`/?tag=garden`, and only the notes tagged `garden` show. Run `wf build` and
open `build/index.html` in a text editor: all seven notes are in it.

If a filtered list is empty in the built HTML, you probably filtered the
`for` itself by `query`; hide the cards instead, as above.

## Step 6: Give each note a page

Each note gets a page at `/notes/<slug>`:

```wf
// src/pages/NotePage.wf
page NotePage(path: "/notes/:slug", slug: String,
    title: "{notes.find(n => n.slug == slug)?.title ?? slug} — Field Notes",
    description: "{notes.find(n => n.slug == slug)?.summary ?? slug}",
    type: "article",
    layout: Sheet,
    paths: notes.map(n => n.slug)) {

    derived note = notes.find(n => n.slug == slug)
```

- `:slug` in the path is a typed parameter of the page.
- `paths:` names the values a static build renders the page for: one
  `notes/<slug>/index.html` per note, each listed in the sitemap
  ([Content](https://webfluent.monzeromer.dev/recipes.md)).
- `title:` and `description:` may be worked out from the parameters, so
  each note's search result shows its own title and summary.

The body of the page is the note's Markdown:

```wf
// src/pages/NotePage.wf
            Markdown(n.body)
```

`Markdown` renders headings, lists, quotes, emphasis and links, at build
time and live. It escapes the text first, so HTML in a note is shown, not
run ([Elements](https://webfluent.monzeromer.dev/learn/06-elements.md)). Below the note, `related` lists the three
newest notes that share a tag with it — a `derived` value like any other.

**Check it:** click a note's title. Its page shows the whole note, and the
browser tab shows its title followed by `— Field Notes`. After `wf build`,
`build/notes/` holds seven folders, one per note.

## Step 7: List the tags, and write a page in Markdown

The Tags page groups the notes under each tag:

```wf
// src/pages/Tags.wf
        Grid(columns: { base: 1, md: 2 }, gap: .md, align: .start) {
            for tag in allTags by tag {
                derived filed = notes.filter(n => n.tags.includes(tag)).sortBy(n => n.date).reverse()
```

- A layout prop takes one value per breakpoint: one column on a phone, two
  from `md` up. It compiles to a media query, so the first paint is already
  right ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md)).
- A `derived` value inside a `for` is worked out for each item.

The About page is not a `.wf` file at all. `src/about.md` is a page: its
front matter names its path, title, description and layout, and the rest
is its Markdown:

```markdown
---
path: /about
title: About — Field Notes
description: Who keeps this notebook, why it exists, and how the site is built.
layout: Sheet
---
# About
```

**Check it:** `/tags` shows one card per tag, in one column on a narrow
window and two on a wide one. `/about` shows the About page in the same
column as the others.

## Step 8: Keep the reader's saved notes

A store keeps the notes this reader saves:

```wf
// src/stores/Saved.wf
/// The notes this reader has saved for later, kept in their browser.
store Saved {
    persist slugs: [String] = []

    derived count = slugs.length
```

`persist` keeps the list in the browser across visits. There is no
account, and nothing is sent anywhere ([Stores](https://webfluent.monzeromer.dev/learn/11-stores.md)).

The button that saves a note says whether it is pressed, the way assistive
technology reads it:

```wf
// src/components/SaveButton.wf
/// Saves a note for later, or takes it off the saved list.
component SaveButton(_ slug: String, title: String) {
    use Saved

    Button(if Saved.slugs.includes(slug) { "Saved" } else { "Save" },
        aria-pressed: Saved.slugs.includes(slug),
        aria-label: if Saved.slugs.includes(slug) { "Saved: {title}" } else { "Save: {title}" },
        class: "save-button").sm.outlined {
        on click { Saved.toggle(slug) }
    }
}
```

`aria-pressed` follows the state, and `styles.css` styles the pressed
button with `[aria-pressed="true"]`, so the look and the meaning are one
attribute ([Accessibility](https://webfluent.monzeromer.dev/about/accessibility.md)). The Saved page lists
`notes.filter(n => Saved.slugs.includes(n.slug))`, and the navbar's badge
shows `Saved.count`.

**Check it:** click **Save** on a card. It reads **Saved**, and the navbar
shows a `1` badge beside **Saved**. Reload the page: the note is still
saved, and `/saved` lists it.

## Step 9: Test it, and run the checks a static site needs

A test reads the project's `data` file, as any page does
([Testing](https://webfluent.monzeromer.dev/learn/16-testing.md)):

```wf
// tests/NoteFinder.wf
test "a search that matches nothing says so, and clears" {
    NoteFinder(notes)
    type "zeppelin" into "Search the notes"
    expect "Nothing matches"
    expect "0 of 7 notes"
    click "Clear the search"
    expect "7 of 7 notes"
    expect not "Nothing matches"
}
```

The project has six tests. Run them, check with warnings counted as
failures, build, and load the result in a browser:

```bash
wf test
wf check --deny-warnings
wf build
wf verify
```

`wf verify` loads every route of the built site in a headless Chrome —
each note's page at its `paths:` value — and fails on an error, a missing
file or a page that draws nothing ([Command line](https://webfluent.monzeromer.dev/reference/cli.md)).

**Check it:** `wf test` reports `6 passed`, `wf build` reports `SSG:
pre-rendered static pages`, and `wf verify` ends with `0 problem(s)`.
`build/` is ready for any static host.

## What you built

- A static build, with canonical addresses, a sitemap and link previews
  ([Content](https://webfluent.monzeromer.dev/recipes.md)).
- Typed data read from a JSON file at build time ([Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md)).
- A shell, and a layout every page names
  ([Pages and routing](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md)).
- A page per note from one `:slug` page and `paths:`, and a page written in
  Markdown ([Content](https://webfluent.monzeromer.dev/recipes.md)).
- A search and a filter that work in pre-rendered HTML
  ([Static sites and SPAs](https://webfluent.monzeromer.dev/about/static-or-single-page.md)).
- A reader's saved list, with a pressed state assistive technology reads
  ([Accessibility](https://webfluent.monzeromer.dev/about/accessibility.md)).
