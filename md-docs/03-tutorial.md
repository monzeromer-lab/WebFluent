# 3. Tutorial: Shelf

<!--
route: guide/tutorial
group: start
blurb: Build a small, complete site — data, components, a store, a form, a theme, a test — and put it on the web.
description: A step-by-step tutorial: build a reading-list site with WebFluent, test it, and deploy it to GitHub Pages.
-->

In this tutorial you build **Shelf**, a small site that lists books worth
reading and lets a reader keep the ones they want. Each step adds one piece,
and ends with what you should see.

## What you will build

Shelf has a home page that lists books with a search box, a page for each
book, a **Your shelf** page with the books the reader saved, a form to
suggest a book, a theme with a dark mode, a test that clicks, and a deploy
to GitHub Pages.

Along the way you use most of what a real site needs: typed data read at
build time, a reusable component, a page per book, a store that keeps the
reader's choices, a validated form, a theme, a test and a deploy. Each step
links the chapter that covers its idea in depth.

It takes about half an hour, plus the deploy.

## Before you start

You need:

- `wf` installed and on your `PATH` ([Getting started](02-getting-started.md)).
  On Windows, type `webfluent` where this tutorial types `wf`.
- Chrome or Chromium on the machine, for the test that clicks
  ([step 9](#step-9-test-the-book-card)) and for `wf verify`
  ([step 10](#step-10-build-and-verify-the-site)). If it is installed
  somewhere unusual, point the `WF_CHROME` environment variable at it.
- For the deploy, a GitHub account and a repository called `shelf`.

You do not need to know WebFluent yet.

## Step 1: Create the project

Start from the `static` template, which pre-renders every page to HTML —
right for a site people find through search:

```bash
wf init shelf -t static
cd shelf
```

You write every file yourself, so delete what the template put in `src/`
and its tests under `tests/` (they test the starter's pages). Then replace
`webfluent.app.json` with:

```json
{
  "name": "shelf",
  "build": { "ssg": true, "csp": true },
  "meta": {
    "title": "Shelf",
    "description": "Books worth reading, and the ones you want to.",
    "lang": "en"
  }
}
```

`ssg` pre-renders each page. `csp` ships a Content-Security-Policy and makes
the build check its own output against it. [Configuration](42-configuration.md)
lists every key.

**Check it:** run `wf check`. It prints `No .wf files found in src/` —
correct, because `src/` is empty until the next step.

## Step 2: Describe the books

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

Then describe the data's shape and read the file, in `src/types.wf`:

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

`type Book` is a record that the type checker holds every use to: writing
`book.titel` is an error where you write it. `data books` reads the file **at
build time** and makes it a constant every page can read, so nothing is
fetched when the page loads. ([Types](17-types.md),
[Fetching data](24-data.md#read-a-file-at-build-time-with-data).)

**Check it:** run `wf check`. It prints `No problems in shelf.`

## Step 3: Keep the reader's shelf in a store

What the reader keeps must survive a reload and be the same on every page,
so it goes in a store, with its value kept by `persist`. Create
`src/stores/Shelf.wf`:

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

`persist` is a `state` that is kept in the browser's storage and read back on
the next visit; a change made in another tab arrives here too. `derived
count` recomputes whenever `saved` changes. ([Stores](16-stores.md).)

**Check it:** run `wf check`. It prints two `U04` warnings:
`` `Shelf.count` is declared but never read `` and the same for
`Shelf.toggle`. Nothing uses the store yet; the next steps do, and the
warnings go away.

## Step 4: Show a book with a component

Every book is shown the same way, so it is a component. Create
`src/components/BookCard.wf`:

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
- `Card.outlined`, `Stack(gap: .sm)` and `Text(…).muted.sm` are built-ins
  with flags. The compiler knows each one's props and refuses a flag it does
  not take.
- `use Shelf` reads the store from step 3. The button's label is an `if`
  used as a value, and it follows the store: when the reader clicks, the
  label changes and nothing else re-renders.

([Components](15-components.md), [Elements](11-elements.md).)

**Check it:** run `wf check`. It reports one error:
`` error[R01]: `/books/{…}` is not a route ``. The card links to a book's
page, and no page answers that address yet. The compiler checks every link
against the pages you declare; the next step adds the page.

## Step 5: Add the home page and a page per book

The home page lists the books with a search box. Create
`src/pages/Home.wf`:

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

- `bind: q` keeps the input and the state in step, both ways.
- `found` is derived from `q`, so typing narrows the list as you go.
- `for … by book.slug` is a keyed loop: a card that stays in the list keeps
  its element instead of being rebuilt.
- `Grid(columns: { base: 1, md: 2 })` is one column on a phone and two from
  the `md` breakpoint. The pre-rendered HTML already has this layout, so no
  script is needed.

Then add a page for each book, `src/pages/BookPage.wf`:

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
- `books.find(…)` may find nothing, so `book` is a `Book?` (a `Book` or
  `null`). `if let b = book` binds it only when it is there. Reading
  `book.title` without that check is a compile error (`T04`).
- `head { … }` adds tags to this page's `<head>` — here, the title that a
  shared link previews with.

([Pages and routing](10-pages-and-routing.md).)

**Check it:** run `wf serve` and open `http://localhost:3000`. You see
**Books worth reading**, `3 of 3 books` and three cards. Type `norman` in
the search box: one card stays, and the count reads `1 of 3 books`. Click **Want to read** on a card and its
label changes to **On your shelf**. Click a book's title and its page opens,
with a breadcrumb back to the list. `wf check` now reports only the `U04`
warning about `Shelf.count`; step 6 uses it.

## Step 6: Add the shelf page and the site's shell

The shelf page shows the books the reader saved. Create
`src/pages/ShelfPage.wf`:

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

Add a page for addresses no other page claims, `src/pages/NotFound.wf`.
`path: "*"` matches anything, and the static build writes it as `404.html`:

```wf
page NotFound(path: "*", title: "Not found", description: "Nothing lives at this address.", noindex: true) {
    Container {
        Heading("Nothing lives here").h1
        Link("Back to the books", to: "/")
    }
}
```

Last, the shell every page shares, `src/App.wf`. `Router` is where the
current page is drawn:

```wf
// … reads what the other steps declare
app {
    Navbar {
        Navbar.Brand { Link("Shelf", to: "/") }
        Navbar.Links {
            Link("Books", to: "/")
            Link("Your shelf ({Shelf.count})", to: "/shelf")
        }
    }
    Router
    Footer { Text("Built with WebFluent").muted.sm }
}
```

**Check it:** `wf check` prints `No problems in shelf.` Every page now has a
navigation bar. Put two books on your shelf: the link reads
**Your shelf (2)**. Open it, then reload the page — the two books are still
there. Open `http://localhost:3000/nowhere` and you see **Nothing lives
here**.

## Step 7: Add a form to suggest a book

Readers can suggest books. Create `src/pages/Suggest.wf`:

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
that state shows its messages accessibly: the field gets `aria-invalid`, and
the message is announced. A submit that fails moves focus to the first field
with a problem and does not run `on submit`. ([Forms](23-forms.md).)

Then link the page from the shell. Add the third link to `src/App.wf`:

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

**Check it:** click **Suggest a book**, then **Send the suggestion** with
the form empty. The form does not send: **Which book?** appears under
**Title**, and **Who wrote it?** under **Author**. Fill in both and send
again. You see **Thank you — we will take a look.**, and the form is empty.

A real site sends the suggestion somewhere. With a backend, you describe the
service once and call it from the handler:

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

([Data and APIs](24-data.md), [Environments](35-environments.md).)

## Step 8: Give the site a theme

A theme changes the design tokens that every built-in is drawn with. Create
`src/theme.wf`:

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

Then name the themes in `webfluent.app.json` — `Paper` as the theme and
`Ink` as the dark one:

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

A reader whose system is set to dark mode gets `Ink`, before the first paint
and with no flash of the light theme. A token you do not name keeps its
baseline value. ([Styling](19-styling.md).)

**Check it:** the links and the **Send the suggestion** button are purple,
and the text is a serif face. Switch
your system to dark mode and the page turns dark. If the build stops with
``No `theme Paper` is declared in this project``, the config names a theme
that `src/theme.wf` does not declare: check the file name and the spelling.

## Step 9: Test the book card

Tests live in `tests/`. Create `tests/BookCard.wf`:

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

Run them:

```bash
wf test
```

The first test renders the card and reads what it shows. It also writes a
snapshot that the next run compares against. The second test **clicks**, so
it runs in a headless Chrome, with the same compiler and runtime a reader
gets. Buttons are found by their text, as a reader finds them.
([Testing](29-testing.md).)

**Check it:** the first run ends:

```text
  new   tests/BookCard.wf — a card shows who wrote the book, and when (snapshot written)
  ok    tests/BookCard.wf — a book goes on the shelf, and comes off again (in a browser)
2 passed, 0 failed, 1 snapshot(s) written
```

Run `wf test` again and the first line reads `ok`: the render matches its
snapshot.

If the second test says there is no browser, install Chrome or Chromium, or
set `WF_CHROME` to its path.

## Step 10: Build and verify the site

Stop the dev server, then build the site and load every page in a real
browser:

```bash
wf build
wf verify
```

`wf build` writes `build/`, a folder of HTML, CSS and JavaScript that any
static host can serve, and prints every warning, such as a missing `alt`, a
page without a description, or a state nothing reads. `wf verify` loads every
page in a real browser and fails on a script error, a failed request or a
blank page.

**Check it:** `wf build` ends with `Build complete.`, and `build/` holds
`index.html`, `shelf/index.html`, `suggest/index.html`, `404.html` and one
`books/<slug>/index.html` per book. `wf verify` lists six routes, each `ok`,
and ends with `6 page(s), 0 problem(s)`.

## Step 11: Deploy to GitHub Pages

Any static host serves `build/`. On GitHub Pages, a project site lives under
`https://<you>.github.io/shelf/`, so tell the build its sub-path and its
address. Add these keys to `webfluent.app.json`, keeping the others:

```json
{
  "build": { "ssg": true, "csp": true, "base_path": "/shelf" },
  "meta": { "site_url": "https://you.github.io/shelf" }
}
```

`base_path` prefixes every link and asset. `site_url` turns on canonical
links, the sitemap and absolute Open Graph URLs.

Then add a workflow that builds and publishes the site on every push to
`main`, `.github/workflows/pages.yml`:

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

Push the project to your `shelf` repository. In the repository's
**Settings → Pages**, choose **GitHub Actions** as the source.

**Check it:** the **Actions** tab shows the **Deploy** workflow passing, and
`https://<you>.github.io/shelf/` shows **Books worth reading**.
[Deploying](34-deploying.md) covers Netlify, Vercel, Cloudflare Pages,
nginx and other hosts.

## The whole program

Every `.wf` file above, as one listing. This is the program the steps built,
with the first of the two tests:

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

## What you built

Shelf is a whole site, and each part of it uses one idea of the language:

| You wrote | The idea | Read more |
|---|---|---|
| `type Book` and `data books` | Typed data read at build time | [Types](17-types.md), [Fetching data](24-data.md#read-a-file-at-build-time-with-data) |
| `store Shelf` with `persist` | Shared state kept across visits | [Stores](16-stores.md) |
| `component BookCard` | A reusable component with a positional prop | [Components](15-components.md) |
| `state`, `derived`, `bind:` and `for … by` on the home page | Reactivity and keyed lists | [State and reactivity](12-state-and-reactivity.md), [Control flow](14-control-flow.md) |
| `path: "/books/:slug"` with `paths:`, and `if let` | Route parameters, static paths and optional values | [Pages and routing](10-pages-and-routing.md) |
| `app { … Router … }` | The shell every page shares | [Pages and routing](10-pages-and-routing.md) |
| `validate` and `Form(bind: form)` | Validated forms | [Forms](23-forms.md) |
| `theme Paper` and `theme Ink` | Design tokens and dark mode | [Styling](19-styling.md) |
| `test "…" { … click … expect … }` | Tests that render and click | [Testing](29-testing.md) |
| `wf build`, `wf verify` and a workflow | Building, checking and deploying | [Deploying](34-deploying.md) |

To go further:

- Read the language properly, starting at [Language basics](09-language-basics.md).
- Talk to a real server: [Data and APIs](24-data.md).
- Make the site work offline: [Offline](26-offline.md).
- Translate it: [Internationalisation](28-i18n.md).
- Build a bigger project: [Tutorials](05-tutorial-todos.md).

## Next

[Coming from React, Vue or Svelte](04-coming-from.md).
