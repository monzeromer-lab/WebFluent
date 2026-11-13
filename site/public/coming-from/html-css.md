# WebFluent for HTML and CSS authors

> Plain HTML and CSS in WebFluent: elements, links, images, forms, classes, flexbox and grid, stylesheets, media queries and scripts, side by side.

The page: https://webfluent.monzeromer.dev/coming-from/html-css

If you write HTML and CSS by hand, WebFluent keeps what you know and takes
away the repetition: elements come designed and accessible, a page's head
is written for you, and a layout is a `Row` or a `Grid` instead of a class
and a rule. Your CSS still works — any `.css` file under `src/` is part of
the build.

## The map

| Concept | HTML & CSS | WebFluent |
|---|---|---|
| A page | `index.html` with `<head>`, `<title>` and `<body>` | `page Home(path: "/", title: "Home", description: "…") { … }` |
| A heading | `<h1>Welcome</h1>` | `Heading("Welcome").h1` |
| A paragraph | `<p>Hello</p>` | `Text("Hello")` |
| A link | `<a href="/about">About</a>` | `Link("About", to: "/about")` |
| A picture | `<img src="/team.jpg" alt="The team">` | `Image(src: "/team.jpg", alt: "The team")` |
| A button | `<button class="btn btn-primary">Save</button>` | `Button("Save").primary` |
| A list | `<ul><li>One</li></ul>` | `List { List.Item { Text("One") } }` |
| A table | `<table><thead>…</thead><tbody>…</tbody></table>` | `Table { Table.Head { … } Table.Body { … } }` |
| A form field | `<label for="e">Email</label><input id="e" type="email">` | `Input(bind: email, label: "Email").email` |
| A row with flexbox | `<div class="row">` and `.row { display: flex; gap: 16px }` | `Row(gap: .md) { … }` |
| A grid | `.grid { display: grid; grid-template-columns: repeat(3, 1fr) }` | `Grid(columns: 3, gap: .md) { … }` |
| A responsive grid | a `@media` rule per breakpoint | `Grid(columns: { base: 1, md: 3 })` |
| Inline styles | `style="padding: 2rem"` | `style { padding: 2rem }` in the element's block |
| Hover and focus | `.card:hover { … }` | `style { &:hover { … } }` |
| A stylesheet | `<link rel="stylesheet" href="site.css">` | any `.css` file under `src/` |
| CSS variables | `--brand: #0F766E` and `var(--brand)` | a `theme` of design tokens, read as `$color-primary` |
| Dark mode | `@media (prefers-color-scheme: dark)` | a second theme, named as `"dark"` in the config |
| A script | `<script src="app.js"></script>` | a plain `.js` file under `src/`, its functions called by name |
| Meta tags | `<meta name="description">`, Open Graph tags | the page's `title:` and `description:`, and `meta` in the config |

## Things that work differently

**Elements are checked.** An `Image` with no `alt:` is a warning (`A01`), a
`Link` to a page that does not exist is an error (`R01`), and a flag the
element does not have (`Button("Save").primray`) stops the build. What you
would have caught with a validator and an accessibility audit, the
compiler catches on every build.

**Text is text.** Every string goes in as text, so a `<` in it is shown,
not parsed. Markup you did not write goes through `Unsafe.Html(sanitize(…))`,
which the build reports each time; inline `onclick` handlers do not compile
— a handler is `on click { … }` in the element's block.

**The head is written for you.** Each page's title, description, canonical
link, Open Graph and Twitter tags, structured data, a sitemap and
`robots.txt` come from the page and the config
([Describe pages to search engines and link previews](https://webfluent.monzeromer.dev/recipes/search-and-link-previews.md)).

**Your CSS is welcome, scoped by you.** A `.css` file under `src/` is bundled
after the built-ins' rules, so yours win; `class:` on any element adds your
classes. A `style { }` block is for the one element it is on.

**A site, not a pile of files.** Pages share a header and footer through
`app` (or a `layout:`), a value shared by pages is a `store`, and repeated
markup is a `component` with typed props.

## A component, ported

A card that shows a person, in HTML and CSS and in WebFluent.

```html
<div class="card">
  <img class="avatar" src="/ada.jpg" alt="Ada Lovelace">
  <div>
    <h3>Ada Lovelace</h3>
    <p class="muted">Analyst</p>
  </div>
</div>

<style>
  .card { display: flex; gap: 12px; align-items: center; padding: 16px;
          border: 1px solid #ddd; border-radius: 8px; }
  .avatar { width: 48px; height: 48px; border-radius: 50%; }
  .muted { color: #666; }
</style>
```

```wf
page Team(path: "/", title: "Team", description: "Who we are.") {
    Heading("Team").h1
    Person("Ada Lovelace", role: "Analyst", photo: "/ada.jpg")
}

component Person(_ name: String, role: String, photo: String) {
    Card.outlined {
        Row(gap: .sm, align: .center) {
            Avatar(src: photo, alt: name)
            Stack {
                Heading(name).h3
                Text(role).muted
            }
        }
    }
}
```

- **No class names to invent.** `Card.outlined`, `Avatar` and `.muted` come
  designed, and follow the theme.
- **One component, many people.** `Person` is placed with each person's
  props; the compiler checks every call.
