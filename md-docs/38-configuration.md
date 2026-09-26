# 38. Configuration

<!--
route: guide/config
group: reference
blurb: Every key of webfluent.app.json, its default, and what it changes.
description: Every key of webfluent.app.json with its default: project, build, media, PDF, slides, dev, motion, meta, theme, i18n, env and offline.
-->

A project's settings live in `webfluent.app.json` beside `src/`. Every key
is optional except `name`, so the smallest config is:

```json
{ "name": "my-site" }
```

A key the build does not know is reported with a warning — and the key it
most likely meant, so `"defaultLocale"` says to write `default_locale` —
rather than silently ignored (**4.1**).

## The defaults

This is what a config with only a name means. `i18n` and `offline` are
absent unless you add them.

<!-- defaults -->
```json
{
  "name": "my-site",
  "version": "0.1.0",
  "author": "",
  "theme": {
    "name": null,
    "tokens": {},
    "builtin": "full",
    "dark": null
  },
  "build": {
    "elements": [],
    "output": "./build",
    "minify": true,
    "sourcemap": false,
    "ssg": false,
    "base_path": "",
    "csp": false,
    "split": true,
    "compress": true,
    "budget": {},
    "runtime": "auto",
    "media": {
      "formats": ["webp"],
      "widths": [480, 960, 1440, 1920],
      "quality": 78,
      "pipeline": true
    },
    "output_type": "spa",
    "pdf": {
      "page_size": "A4",
      "margins": { "top": 72.0, "bottom": 72.0, "left": 72.0, "right": 72.0 },
      "default_font": "Helvetica",
      "default_font_size": 12.0,
      "output_filename": null
    },
    "slides": {
      "size": "16:9",
      "width": null,
      "height": null,
      "default_font": "Helvetica",
      "default_font_size": 24.0,
      "margin": 60.0,
      "show_slide_numbers": false,
      "footer_text": null,
      "background_color": null,
      "chrome_color": null,
      "output_filename": null
    }
  },
  "dev": { "port": 3000, "hot_reload": true },
  "meta": {
    "integrity": {},
    "title": "my-site",
    "description": "",
    "favicon": "",
    "touch_icon": "",
    "lang": "en",
    "site_url": "",
    "site_name": "",
    "image": "",
    "sitemap": true,
    "fonts": [],
    "stylesheets": []
  },
  "motion": { "duration": null, "easing": null },
  "env": {},
  "public_env": []
}
```
<!-- /defaults -->

`wf init` writes a shorter file and turns `build.csp` on.

## The project

| Key | Default | Meaning |
|---|---|---|
| `name` | — | The only required key. The site's name, the PDF's file name when none is given, and `meta.title`'s fallback. |
| `version`, `author` | `"0.1.0"`, `""` | Yours; nothing in the build reads them. |

## `build`

| Key | Default | Meaning |
|---|---|---|
| `output` | `"./build"` | Where the build is written, relative to the project. |
| `output_type` | `"spa"` | `spa` (a website), `pdf`, `slides` or `elements` ([chapter 33](33-pdf-and-slides.md)). |
| `ssg` | `false` | Pre-render every page to its own HTML ([chapter 26](26-static-and-spa.md)). |
| `base_path` | `""` | The sub-path the site is served under, such as `"/docs"`. Every link and asset is prefixed. |
| `split` | `true` | One script and stylesheet chunk per page, loaded when its route shows. |
| `minify` | `true` | Strip comments and whitespace from the scripts and stylesheets. |
| `sourcemap` | `false` | Write a source map beside the bundle. |
| `compress` | `true` | Write a `.gz` beside every text output over a kilobyte. |
| `runtime` | `"auto"` | `auto` ships the runtime modules the program reaches; `full` ships every one, for a hand-written script that calls `WF` ([chapter 32](32-javascript-interop.md)). |
| `csp` | `false` (`true` in a project `wf init` makes) | Emit a Content-Security-Policy and `_headers`, and check the output against the policy ([chapter 23](23-security.md#content-security-policy)). |
| `budget` | `{}` | A gzipped size each named output should stay under: `{ "app.js": "40 kB" }`. Over it, the build warns. |
| `elements` | `[]` | With `output_type: "elements"`, the components to publish as custom elements. |
| `media` | below | The image pipeline. |
| `pdf`, `slides` | below | The paper outputs. |

### `build.media`

| Key | Default | Meaning |
|---|---|---|
| `formats` | `["webp"]` | The formats every `image` is written in, as well as its original. |
| `widths` | `[480, 960, 1440, 1920]` | The widths written, where smaller than the original. |
| `quality` | `78` | The encoding quality, 1–100. |
| `pipeline` | `true` | `false` copies images as `public/` does, without resizing. |

### `build.pdf`

| Key | Default | Meaning |
|---|---|---|
| `page_size` | `"A4"` | `A4`, `A3`, `A5`, `Letter` or `Legal`. |
| `margins` | `72` each | `top`, `bottom`, `left`, `right`, in points (72 to the inch). |
| `default_font` | `"Helvetica"` | `Helvetica`, `Times-Roman` or `Courier`, or their `-Bold` forms — standard PDF fonts, which need no embedding. |
| `default_font_size` | `12` | In points. |
| `output_filename` | the project's name | The file written. |

### `build.slides`

| Key | Default | Meaning |
|---|---|---|
| `size` | `"16:9"` | `16:9` (960 × 540 pt), `4:3`, `A4-landscape`, or `"WIDTHxHEIGHT"` in points. |
| `width`, `height` | `null` | Explicit points, overriding `size`. |
| `margin` | `60` | The space around each slide's content; what runs past it is clipped. |
| `default_font`, `default_font_size` | `"Helvetica"`, `24` | The deck's text. |
| `show_slide_numbers` | `false` | `n / total` in the bottom corner. |
| `footer_text` | `null` | Text in the bottom-left of every slide. |
| `background_color` | `null` | A full-bleed colour for every slide. |
| `chrome_color` | `null` | The colour of the numbers and footer; unset, it flips between dark and light on the slide's background. |
| `output_filename` | the project's name | The file written. |

## `dev`

| Key | Default | Meaning |
|---|---|---|
| `port` | `3000` | The port `wf serve` listens on. |
| `hot_reload` | `true` | Rebuild on a save, reload open tabs, and draw a failed build's error over the page. |

## `motion`

| Key | Default | Meaning |
|---|---|---|
| `duration` | unset | How long an animation runs when the element does not say: `"180ms"`. |
| `easing` | unset | How it is paced: a CSS timing function, or a token such as `"$ease-standard"`. |

## `meta`

| Key | Default | Meaning |
|---|---|---|
| `title`, `description` | the name, `""` | What a page falls back to when it declares none. |
| `lang` | `"en"` | The document's language. |
| `site_url` | `""` | The absolute address. Without it, canonical links, the sitemap's URLs and absolute sharing URLs are left out rather than guessed. |
| `site_name` | `""` | The name a link preview shows. |
| `image` | `""` | The sharing image for pages that set none. |
| `favicon` | `""` | The icon a tab shows: a file in `public/` — an SVG scales to every size — or a URL. |
| `touch_icon` | `""` | The icon a phone puts on its home screen: a 180×180 PNG in `public/`. iOS reads this, not an SVG `favicon`. |
| `sitemap` | `true` | Write `sitemap.xml` and `robots.txt`. |
| `fonts` | `[]` | Web-font stylesheet URLs to link, each with a `preconnect`; their origins join the policy. |
| `stylesheets` | `[]` | Extra stylesheets to link ahead of `styles.css`: a file in `public/` or a URL. |
| `integrity` | `{}` | Subresource-integrity hashes of external assets, by URL. |

## `theme`

| Key | Default | Meaning |
|---|---|---|
| `name` | unset | Which `theme` declaration is the site's. A project with exactly one need not name it. |
| `dark` | unset | The theme that stands in for dark mode ([chapter 15](15-styling.md#dark-mode)). |
| `builtin` | `"full"` | `full` ships the built-ins' designed look; `structural` only their layout and mechanics. |
| `tokens` | `{}` | Token values on top of the theme, for a pipeline to supply. |

## `i18n`

```json
{ "i18n": { "default_locale": "en", "locales": ["en", "ar"], "dir": "src/translations" } }
```

| Key | Default | Meaning |
|---|---|---|
| `default_locale` | `"en"` | The locale a page opens in. |
| `locales` | `["en"]` | Every locale; one JSON file each. |
| `dir` | `"src/translations"` | Where the files are. |

[Chapter 21](21-i18n.md).

## `env` and `public_env`

| Key | Default | Meaning |
|---|---|---|
| `env` | `{}` | Values read as `env.NAME`, fixed at build time. A `.env` file and the shell add to them. |
| `public_env` | `[]` | Names a page may read beyond those beginning `PUBLIC_`. |

[Chapter 30](30-environments.md).

## `offline`

**4.1.** Absent, no service worker is written.

```json
{ "offline": { "precache": ["/", "/docs/*"], "fallback": "/offline", "cache": { "/api/*": "network-first" }, "sync": true } }
```

| Key | Default | Meaning |
|---|---|---|
| `precache` | `["/"]` | The routes a first visit stores, as globs; `"*"` for every route. |
| `fallback` | `null` | A page's path, shown for a route that was not stored. |
| `cache` | `{}` | How paths the build did not write are fetched: `network-first`, `cache-first`, `stale-while-revalidate` or `network-only`. |
| `sync` | `false` | Keep writes made offline and send them later. |

[Chapter 19](19-offline.md).

## Next

[Diagnostics](39-diagnostics.md).
