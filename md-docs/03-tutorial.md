# 3. Tutorial

<!--
route: guide/tutorial
group: start
blurb: Build a small, complete site — data, components, a store, a form, a theme, a test — and put it on the web.
description: A step-by-step tutorial: build a reading-list site with WebFluent, test it, and deploy it to GitHub Pages.
-->

In this chapter you build **Shelf**, a small site that lists books worth
reading and lets a reader keep the ones they want. Along the way you use
most of what a real site needs: typed data read at build time, a reusable
component, a page per book, a store the reader's choices are kept in, a
validated form, a theme with a dark mode, a test that clicks, and a deploy.

Each step says what to write and why, and links the chapter that covers the
idea in depth. It takes about half an hour.

You need `wf` installed ([Getting started](02-getting-started.md)).

## Step 1: a project

```bash
wf init shelf -t static
cd shelf
```

The `static` template pre-renders every page to HTML — right for a site
people find through search. Delete what the template put in `src/`; you will
write every file yourself. Replace `webfluent.app.json` with:

```json
{
  "name": "shelf",
  "build": { "ssg": true, "csp": true },
  "theme": { "name": "Paper", "dark": "Ink" },
  "meta": {
    "title": "Shelf",
    "description": "Books worth reading, and the ones you want to.",
    "lang": "en"
  }
}
```

`ssg` pre-renders; `csp` ships a Content-Security-Policy and makes the build
check its own output against it; `theme` names the themes you will declare in
step 7. [Configuration](38-configuration.md) lists every key.

## Step 2: the data

The books are data, not code. Put them in `src/books.json`:

```json
[
  { "slug": "notes-on-the-analytical-engine", "title": "Notes on the Analytical Engine",
    "author": "Ada Lovelace", "year": 1843, "pages": 66,
    "summary": "The first published algorithm, and a view of what a machine might do beyond arithmetic.",
    "tags": ["computing", "history"] },
  { "slug": "the-design-of-everyday-things", "title": "The Design of Everyday Things",
    "author": "Don Norman", "year": 1988, "pages": 368,
    "summary": "Why doors confuse us, and what that says about everything else we make.",
    "tags": ["design"] },
  { "slug": "a-pattern-language", "title": "A Pattern Language",
    "author": "Christopher Alexander", "year": 1977, "pages": 1171,
    "summary": "Two hundred and fifty-three patterns for towns, buildings and rooms people want to live in.",
    "tags": ["design", "architecture"] }
]
```

Then describe it, in `src/types.wf`:

```wf
/// One book on the list.
type Book {
    slug: String
    title: String
    author: String
    year: Number
    pages: Number
    summary: String
    tags: [String] = []
}

data books: [Book] = "books.json"
```

`type Book` is a record the type checker holds every use to: `book.titel`
is an error where you write it. `data books` reads the file **at build time**
and makes it a constant every page can read — nothing is fetched when the
page loads. ([Types](13-types.md), [Data](17-data.md#data-a-file-at-build-time).)

## Step 3: a component

Every book is shown the same way, so it is a component. `src/components/BookCard.wf`:

```wf
// … reads what the other steps declare
/// A book at a glance, with the button that puts it on the shelf.
component BookCard(_ book: Book) {
    use Shelf
    Card.outlined {
        Stack(gap: .sm) {
            Link(book.title, to: "/books/{book.slug}")
            Text("{book.author} · {book.year}").muted.sm
            Text(book.summary)
            Row(gap: .xs) {
                for tag in book.tags { Tag(tag) }
            }
            Button(if Shelf.saved.includes(book.slug) { "On your shelf" } else { "Want to read" }).sm {
                on click { Shelf.toggle(book.slug) }
            }
        }
    }
}
```

- `_ book: Book` is the one prop a caller may pass without its name:
  `BookCard(book)`.
- `Card.outlined`, `Stack(gap: .sm)`, `Text(…).muted.sm` are built-ins with
  flags; the compiler knows each one's props and refuses a flag it does not
  take.
- `use Shelf` reads a store you write next. The button's label is an `if`
  used as a value, and it follows the store: when the reader clicks, the
  label changes without anything else re-rendering.

([Components](11-components.md), [Elements](07-elements.md).)

## Step 4: the reader's shelf

What the reader keeps must survive a reload and be the same on every page —
a store with a `persist`ed value. `src/stores/Shelf.wf`:

```wf
/// The books the reader wants to read, kept across visits.
store Shelf {
    persist saved: [String] = []
    derived count = saved.length

    action toggle(slug: String) {
        if saved.includes(slug) {
            saved = saved.filter(s => s != slug)
        } else {
            saved = [...saved, slug]
        }
    }
}
```

`persist` is a `state` kept in the browser's storage and read back on the next
visit; a change in another tab arrives here too. `derived count` recomputes
whenever `saved` changes. ([Stores](12-stores.md).)

## Step 5: pages

The shell every page shares, `src/App.wf`:

```wf
// … reads what the other steps declare
app {
    Navbar {
        Navbar.Brand { Link("Shelf", to: "/") }
        Navbar.Links {
            Link("Books", to: "/")
            Link("Your shelf ({Shelf.count})", to: "/shelf")
            Link("Suggest a book", to: "/suggest")
        }
    }
    Router
    Footer { Text("Built with WebFluent").muted.sm }
}
```

The home page lists the books with a search box, `src/pages/Home.wf`:

```wf
// … reads what the other steps declare
page Home(path: "/", title: "Shelf", description: "Books worth reading, and the ones you want to.") {
    state q = ""
    derived needle = q.trim().toLowerCase()
    derived found = books.filter(b => b.title.toLowerCase().includes(needle) || b.author.toLowerCase().includes(needle))

    Container {
        Stack(gap: .lg) {
            Heading("Books worth reading").h1
            Input(bind: q, label: "Search by title or author").search
            Text("{found.length} of {books.length} books").muted.sm
            Grid(columns: { base: 1, md: 2 }, gap: .md) {
                for book in found by book.slug { BookCard(book) }
            }
            if found.length == 0 { Text("Nothing matches “{q}”.").muted }
        }
    }
}
```

- `bind: q` keeps the input and the state in step both ways.
- `found` is derived from `q`, so typing narrows the list as you go.
- `for … by book.slug` is a keyed loop: a card that stays in the list keeps
  its element, rather than being rebuilt.
- `Grid(columns: { base: 1, md: 2 })` is one column on a phone and two from
  the `md` breakpoint — written in the first paint, no script needed.

A page per book, `src/pages/BookPage.wf`:

```wf
// … reads what the other steps declare
page BookPage(path: "/books/:slug", slug: String, title: "Book", description: "One book on the shelf.", type: "article", paths: books.map(b => b.slug)) {
    derived book = books.find(b => b.slug == slug)
    head {
        meta(property: "og:title", content: book?.title ?? "Book")
    }
    Container {
        if let b = book {
            Breadcrumb {
                Breadcrumb.Item(to: "/") { Text("Books") }
                Breadcrumb.Item { Text(b.title) }
            }
            Heading(b.title).h1
            Text("{b.author}, {b.year} · {b.pages} pages").muted
            Paragraph(b.summary)
        } else {
            Heading("No such book").h1
            Link("Back to the list", to: "/")
        }
    }
}
```

- `:slug` in the path is a typed parameter of the page.
- `paths:` tells the static build which values to render: one HTML file per
  book, `books/a-pattern-language/index.html` and so on.
- `books.find(…)` may find nothing, so `book` is a `Book?`. `if let b = book`
  binds it only when it is there — reading `book.title` without it is a
  compile error (`T04`).
- `head { … }` adds tags to this page's `<head>`, here the title a shared link
  previews with.

The shelf itself, `src/pages/ShelfPage.wf`:

```wf
// … reads what the other steps declare
page ShelfPage(path: "/shelf", title: "Your shelf", description: "The books you want to read.") {
    use Shelf
    derived mine = books.filter(b => Shelf.saved.includes(b.slug))
    Container {
        Stack(gap: .lg) {
            Heading("Your shelf").h1
            if mine.length == 0 {
                Text("Nothing yet. Pick something from the list.").muted
                Link("Browse the books", to: "/")
            } else {
                for book in mine by book.slug { BookCard(book) }
            }
        }
    }
}
```

And a page for addresses nobody claims, `src/pages/NotFound.wf`:

```wf
page NotFound(path: "*", title: "Not found", description: "Nothing lives at this address.", noindex: true) {
    Container {
        Heading("Nothing lives here").h1
        Link("Back to the books", to: "/")
    }
}
```

Run `wf serve` and open `http://localhost:3000`: search, open a book, put two
on your shelf, reload — they are still there. ([Pages and routing](06-pages-and-routing.md).)

## Step 6: a form

Readers suggest books. `src/pages/Suggest.wf`:

```wf
page Suggest(path: "/suggest", title: "Suggest a book", description: "Tell us what belongs on the shelf.") {
    state title = ""
    state author = ""
    state why = ""
    state sent = false

    validate title { required "Which book?" }
    validate author { required "Who wrote it?" }
    validate why { maxLength(280) "Keep it under 280 characters" }

    Container {
        Stack(gap: .lg) {
            Heading("Suggest a book").h1
            if sent { Alert("Thank you — we will take a look.").success }
            Form(bind: form) {
                on submit {
                    sent = true
                    form.reset()
                }
                Input(bind: title, label: "Title")
                Input(bind: author, label: "Author")
                Textarea(bind: why, label: "Why it belongs here", rows: 4, maxLength: 280)
                Button("Send the suggestion", type: .submit).primary
            }
        }
    }
}
```

A `validate` block sits beside the state it guards, and the control bound to
that state shows its messages — accessibly, with `aria-invalid` and the
message announced. A submit that fails moves focus to the first field with a
problem and does not run `on submit`. ([Forms](16-forms.md).)

A real site would send the suggestion somewhere. With a backend, describe it
once and call it:

```wf
api Suggestions(base: env.PUBLIC_API ?? "/api") {
    post suggest(body: Map)
}
…
on submit {
    await Suggestions.suggest(body: { title: title, author: author, why: why })
    sent = true
    form.reset()
}
```

([Data and APIs](17-data.md), [Environments](30-environments.md).)

## Step 7: a look of its own

A theme changes the design tokens every built-in is drawn with. `src/theme.wf`:

```wf
theme Paper {
    color-primary: #6D28D9
    font-family: Charter, Georgia, serif
    radius-md: 10px
}

theme Ink {
    color-background: #111014
    color-surface: #1A1820
    color-text: #ECEAF2
    color-text-muted: #A8A3B5
    color-border: #2B2833
}
```

The config named `Paper` as the theme and `Ink` as the dark one, so a reader
whose system is dark gets `Ink` — before the first paint, with no flash. A
token you do not name keeps its baseline value. ([Styling](15-styling.md).)

## Step 8: a test

`tests/BookCard.wf`:

```wf
// … reads what the other steps declare
test "a card shows who wrote the book, and when" {
    BookCard(Book(slug: "x", title: "X", author: "Ada Lovelace", year: 1843, pages: 66, summary: "Notes."))
    expect "Ada Lovelace · 1843"
    expect "Want to read"
}

test "a book goes on the shelf, and comes off again" {
    BookCard(Book(slug: "x", title: "X", author: "Ada Lovelace", year: 1843, pages: 66, summary: "Notes."))
    click "Want to read"
    expect "On your shelf"
    click "On your shelf"
    expect "Want to read"
}
```

```bash
wf test
```

The first test renders the card and reads what it shows; it also writes a
snapshot the next run compares against. The second **clicks**, so it runs in
a headless Chrome — the same compiler and runtime a reader gets. Buttons are
found by their text, as a reader finds them. ([Testing](24-testing.md).)

## Step 9: build and check

```bash
wf build
wf verify
```

`wf build` writes `build/` — a folder of HTML, CSS and JavaScript any static
host can serve — and prints every warning: a missing `alt`, a page without a
description, a state nothing reads. `wf verify` loads every page in a real
browser and fails on a script error, a failed request or a blank page.

## Step 10: deploy

Any static host serves `build/`. For GitHub Pages, a project site lives under
`https://<you>.github.io/shelf/`, so tell the build its sub-path and its
address — add to `webfluent.app.json`:

```json
{
  "build": { "ssg": true, "csp": true, "base_path": "/shelf" },
  "meta": { "site_url": "https://you.github.io/shelf" }
}
```

`base_path` prefixes every link and asset; `site_url` turns on canonical
links, the sitemap and absolute Open Graph URLs. Then add
`.github/workflows/pages.yml`:

```yaml
name: Deploy
on:
  push:
    branches: [main]
permissions:
  contents: read
  pages: write
  id-token: write
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v4
      - run: curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
      - run: ~/.webfluent/bin/wf build
      - uses: actions/upload-pages-artifact@v3
        with:
          path: build
      - id: deployment
        uses: actions/deploy-pages@v4
```

Push, and in the repository's *Settings → Pages* choose *GitHub Actions* as
the source. [Deploying](29-deploying.md) covers Netlify, Vercel, Cloudflare
Pages, nginx and the rest.

## The whole program

Every file above, as one listing — this is exactly what the steps built:

```wf
type Book {
    slug: String
    title: String
    author: String
    year: Number
    pages: Number
    summary: String
    tags: [String] = []
}

data books: [Book] = "books.json"

store Shelf {
    persist saved: [String] = []
    derived count = saved.length

    action toggle(slug: String) {
        if saved.includes(slug) {
            saved = saved.filter(s => s != slug)
        } else {
            saved = [...saved, slug]
        }
    }
}

component BookCard(_ book: Book) {
    use Shelf
    Card.outlined {
        Stack(gap: .sm) {
            Link(book.title, to: "/books/{book.slug}")
            Text("{book.author} · {book.year}").muted.sm
            Text(book.summary)
            Row(gap: .xs) {
                for tag in book.tags { Tag(tag) }
            }
            Button(if Shelf.saved.includes(book.slug) { "On your shelf" } else { "Want to read" }).sm {
                on click { Shelf.toggle(book.slug) }
            }
        }
    }
}

app {
    Navbar {
        Navbar.Brand { Link("Shelf", to: "/") }
        Navbar.Links {
            Link("Books", to: "/")
            Link("Your shelf ({Shelf.count})", to: "/shelf")
            Link("Suggest a book", to: "/suggest")
        }
    }
    Router
    Footer { Text("Built with WebFluent").muted.sm }
}

page Home(path: "/", title: "Shelf", description: "Books worth reading, and the ones you want to.") {
    state q = ""
    derived needle = q.trim().toLowerCase()
    derived found = books.filter(b => b.title.toLowerCase().includes(needle) || b.author.toLowerCase().includes(needle))

    Container {
        Stack(gap: .lg) {
            Heading("Books worth reading").h1
            Input(bind: q, label: "Search by title or author").search
            Text("{found.length} of {books.length} books").muted.sm
            Grid(columns: { base: 1, md: 2 }, gap: .md) {
                for book in found by book.slug { BookCard(book) }
            }
            if found.length == 0 { Text("Nothing matches “{q}”.").muted }
        }
    }
}

page BookPage(path: "/books/:slug", slug: String, title: "Book", description: "One book on the shelf.", type: "article", paths: books.map(b => b.slug)) {
    derived book = books.find(b => b.slug == slug)
    head {
        meta(property: "og:title", content: book?.title ?? "Book")
    }
    Container {
        if let b = book {
            Breadcrumb {
                Breadcrumb.Item(to: "/") { Text("Books") }
                Breadcrumb.Item { Text(b.title) }
            }
            Heading(b.title).h1
            Text("{b.author}, {b.year} · {b.pages} pages").muted
            Paragraph(b.summary)
        } else {
            Heading("No such book").h1
            Link("Back to the list", to: "/")
        }
    }
}

page ShelfPage(path: "/shelf", title: "Your shelf", description: "The books you want to read.") {
    use Shelf
    derived mine = books.filter(b => Shelf.saved.includes(b.slug))
    Container {
        Stack(gap: .lg) {
            Heading("Your shelf").h1
            if mine.length == 0 {
                Text("Nothing yet. Pick something from the list.").muted
                Link("Browse the books", to: "/")
            } else {
                for book in mine by book.slug { BookCard(book) }
            }
        }
    }
}

page Suggest(path: "/suggest", title: "Suggest a book", description: "Tell us what belongs on the shelf.") {
    state title = ""
    state author = ""
    state why = ""
    state sent = false

    validate title { required "Which book?" }
    validate author { required "Who wrote it?" }
    validate why { maxLength(280) "Keep it under 280 characters" }

    Container {
        Stack(gap: .lg) {
            Heading("Suggest a book").h1
            if sent { Alert("Thank you — we will take a look.").success }
            Form(bind: form) {
                on submit {
                    sent = true
                    form.reset()
                }
                Input(bind: title, label: "Title")
                Input(bind: author, label: "Author")
                Textarea(bind: why, label: "Why it belongs here", rows: 4, maxLength: 280)
                Button("Send the suggestion", type: .submit).primary
            }
        }
    }
}

page NotFound(path: "*", title: "Not found", description: "Nothing lives at this address.", noindex: true) {
    Container {
        Heading("Nothing lives here").h1
        Link("Back to the books", to: "/")
    }
}

theme Paper {
    color-primary: #6D28D9
    font-family: Charter, Georgia, serif
    radius-md: 10px
}

theme Ink {
    color-background: #111014
    color-surface: #1A1820
    color-text: #ECEAF2
    color-text-muted: #A8A3B5
    color-border: #2B2833
}

test "a card shows who wrote the book, and when" {
    BookCard(Book(slug: "x", title: "X", author: "Ada Lovelace", year: 1843, pages: 66, summary: "Notes."))
    expect "Ada Lovelace · 1843"
    expect "Want to read"
}
```

## Where to go from here

- Read the language properly, from [Language basics](05-language-basics.md).
- Talk to a real server: [Data and APIs](17-data.md).
- Make it work offline: [Offline](19-offline.md).
- Translate it: [Internationalisation](21-i18n.md).

## Next

[Coming from React, Vue or Svelte](04-coming-from.md).
