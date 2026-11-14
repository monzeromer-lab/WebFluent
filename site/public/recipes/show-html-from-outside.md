# Show HTML from outside the project

> Show HTML that arrived from outside the project, such as a CMS body, through sanitize and Unsafe.Html, the one door for markup.

The page: https://webfluent.monzeromer.dev/recipes/show-html-from-outside

Every string an element shows goes in as text, so `"<p>"` in your data is
shown as `<p>`. Sometimes the content really is HTML: a CMS body, the
output of a Markdown renderer of your own, an email. This page fetches a
post and shows its HTML body as markup:

```wf
type Post { title: String, body: String }

api Backend(base: "/api") {
    get post(slug: String) at "posts/:slug" -> Post
}

page PostPage(path: "/p/:slug", title: "Post", description: "One post.", slug: String) {
    resource post = Backend.post(slug: slug)
    match post {
        ready(p) {
            Heading(p.title).h1
            Unsafe.Html(sanitize(p.body))
        }
        else { Skeleton }
    }
}
```

## How it works

- `sanitize(html)` keeps only what an allow-list names: a fixed set of
  elements and, for each, a fixed set of attributes. It keeps no
  `<script>`, `<style>`, `<iframe>` or `<form>`, and no `on*` attribute,
  and every `href` and `src` goes through the same scheme check as every
  other URL. A dropped element keeps its text.
- `Unsafe.Html(markup)` puts the result in as markup. The build prints a
  `V03` at each one, saying whether its markup went through `sanitize`,
  and `wf audit` lists them all.
- `sanitize` runs at build time as well as in the browser, from the same
  allow-list, so a pre-rendered page shows the sanitised body at once.

> [!CAUTION]
> Use `Unsafe.Html(raw)` without `sanitize` only for markup **your project
> produced**. Anything that arrived from outside goes through `sanitize`.

[Security](https://webfluent.monzeromer.dev/about/security.md#one-door-for-markup) says why markup has
one door, and what the allow-list keeps out.

## Variations

- **Markup a project script inserts**: a script under `src/` that sets
  `innerHTML` with markup from outside the project passes it through
  `WF.sanitize(…)` first ([Call WF from a script](https://webfluent.monzeromer.dev/recipes/call-wf-from-a-script.md)).
- **Markdown instead of HTML**: `Markdown(text)` escapes any HTML in the
  text and needs no `Unsafe` ([Render Markdown](https://webfluent.monzeromer.dev/recipes/render-markdown.md)).
