# Publish a page in each language

> Have search engines find each language: write each page once per language at its own address, with lang: and translates: joining them.

The page: https://webfluent.monzeromer.dev/recipes/site-in-two-languages

Have a search engine find your site in each of its languages. `t()` and
`setLocale` translate a page in the browser, at one address, and a search
engine indexes what that address's HTML says, which is the default locale.
A site that wants each language found writes each page once per language,
at an address of its own, and says how the pages are joined.

This site is written in Arabic, with its home page in English at `/en`.
The site's language, its address and a static build go in
`webfluent.app.json`. Without `meta.lang` the site's pages are taken to be
in English, and without `meta.site_url` there is no absolute address for
the alternates:

```json
{
    "build": { "ssg": true },
    "meta": { "lang": "ar", "site_url": "https://example.com" }
}
```

Then the pages:

```wf
page Home(path: "/", title: "منذر عمر", description: "مهندس خلفيات.") {
    Heading("مرحباً").h1
}

page HomeEn(path: "/en", title: "Monzer Omer", description: "Engineer.",
            lang: "en", translates: Home) {
    Heading("Hello").h1
}
```

## How it works

- **`lang:`** is the language the page is written in. A page without one
  is in the site's (`meta.lang`, or `default_locale`). The page is written
  `<html lang="en">`, with `dir="rtl"` for a language written right to
  left, and its `og:locale` and its JSON-LD's `inLanguage` say the same.
  The router sets `lang` and `dir` on `<html>` when it shows the page, and
  puts back the site's (or the locale the reader chose) on a page without
  one.
- **`translates:`** names the page this one is a translation of. Pages
  joined this way — `HomeFr` may translate `Home` too — each list all of
  them as `hreflang` alternates, at their own addresses and canonical to
  themselves, with `x-default` for the one in the site's language, and
  each names the others' languages as `og:locale:alternate`.
- **A `:param` page's translation is matched by its parameters:**
  `/en/posts/:slug` for `/posts/:slug`, filled with the same `slug`.

A `translates:` that names no page, or two translations of one page in one
language, is `R05`.
