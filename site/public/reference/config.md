# Configuration

> Every key of webfluent.app.json, its type and default: project, build, media, PDF, slides, dev, motion, meta, theme, i18n, env, offline and lints.

The page: https://webfluent.monzeromer.dev/reference/config

This chapter is the reference for `webfluent.app.json`, the file beside
`src/` that holds a project's settings. Look up a key here to find its
type, its default and what it changes.

Every key is optional except `name`, so the smallest config is one line:

```json
{ "name": "my-site" }
```

A key the build does not know draws the `E112` warning, with the key it
most likely meant — `"defaultLocale"` says to write `default_locale` —
instead of being ignored (since **4.1**).

## Reading and changing the config

`wf config` lists every setting and edits the file without reformatting it
([Command line](https://webfluent.monzeromer.dev/reference/cli.md#wf-config)):

- `wf config schema` lists every setting with its type, the values it
  takes, its default and what it does. `--json` prints the same for a tool;
  an editor's settings window is built from it.
- `wf config get PATH` prints what the project sets at a dotted path, or the
  default.
- `wf config set PATH VALUE` checks the value against the setting's type
  before it writes anything. It replaces a value in place, and adds a new
  key at the end of its section, in the section's own style.
- `wf config unset PATH` removes a setting, so its default applies, and
  removes a section it leaves empty.

```bash
wf config set build.base_path /Docs
wf config set build.budget.app.js "40 kB"
wf config get dev.port
wf config unset build.base_path
```

Under a map such as `build.budget`, the rest of the path is one key, so
`build.budget.app.js` names the `app.js` entry.

## The defaults

This block is what a config holding only a name means. The compiler's test
suite checks it against the compiler's own defaults. `i18n` and `offline`
are absent unless you add them.

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
    "clean_urls": null,
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
      "output_filename": null,
      "fonts": [],
      "system_fonts": true
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
      "output_filename": null,
      "fonts": [],
      "system_fonts": true
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
    "owner": "organization",
    "same_as": [],
    "job_title": "",
    "image": "",
    "image_alt": "",
    "sitemap": true,
    "fonts": [],
    "preload": [],
    "stylesheets": [],
    "scripts": [],
    "connect": [],
    "img": [],
    "frame": [],
    "wasm": false,
    "style_elements": false
  },
  "motion": { "duration": null, "easing": null },
  "env": {},
  "lints": {},
  "public_env": []
}
```
<!-- /defaults -->

`wf init` writes a shorter file than this one, and turns `build.csp` on.

## The project

The top-level keys that describe the project itself.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | string | — | The only required key. The site's name, the PDF's file name when none is given, and the fallback for `meta.title`. |
| `version` | string | `"0.1.0"` | The project's version, as you number it. Nothing in the build reads it. |
| `author` | string | `""` | Who wrote it. Nothing in the build reads it. |

## `build`

How the project is compiled, and what the build writes.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `output` | string | `"./build"` | Where the build is written, relative to the project and inside it: a path that climbs out (`../site`, `/var/www`) or a link to somewhere else is refused unless the command line says `--allow-output-outside` ([CLI](https://webfluent.monzeromer.dev/reference/cli.md#wf-build)). |
| `output_type` | `spa`, `static`, `pdf`, `slides`, `elements` | `"spa"` | What the project builds: `spa` is a website; `static` is a website whose every page is pre-rendered, as `ssg: true` builds it; `pdf` a document; `slides` a deck; `elements` custom elements ([PDF and slides](https://webfluent.monzeromer.dev/learn/17-pdf-and-slides.md), [custom elements](https://webfluent.monzeromer.dev/recipes/publish-custom-elements.md)). |
| `ssg` | boolean | `false` | Pre-renders every page to its own HTML file, so it is read before any script runs ([Static sites and SPAs](https://webfluent.monzeromer.dev/about/static-or-single-page.md)). |
| `base_path` | string | `""` | The sub-path the site is served under, such as `"/docs"`. Every link and asset is prefixed with it. |
| `clean_urls` | `file`, `directory` | unset | How a route's address avoids a redirect on a host that adds a trailing slash. `"file"` also writes `contact.html` beside `contact/index.html`; `"directory"` names every route `/contact/` in canonical links, the sitemap and links to pages ([Content](https://webfluent.monzeromer.dev/recipes.md)). |
| `split` | boolean | `true` | Writes one script and stylesheet chunk per page, loaded when its route shows. Off, every page ships in `app.js`. |
| `minify` | boolean | `true` | Strips comments and whitespace from the scripts and stylesheets. Off, the output is readable and larger. |
| `sourcemap` | boolean | `false` | Writes a source map (version 3) beside the bundle and each page's chunk — `app.js.map`, `pages/Home.js.map` — so a stack trace from the running site reads back to the `.wf` line each statement came from. The scripts are otherwise unchanged. |
| `compress` | boolean | `true` | Writes a gzipped `.gz` copy beside every text output over a kilobyte, for hosts that serve one. |
| `runtime` | `auto`, `full` | `"auto"` | `auto` ships only the runtime modules the program reaches; `full` ships every one, for a hand-written script that calls `WF` in ways the build cannot see ([JavaScript interop](https://webfluent.monzeromer.dev/recipes.md)). |
| `csp` | boolean | `false` (`true` in a project `wf init` makes) | Emits a Content-Security-Policy and a `_headers` file, and checks the output against the policy ([Security](https://webfluent.monzeromer.dev/about/security.md#ship-a-content-security-policy)). |
| `budget` | map of file to size | `{}` | A gzipped size each named output should stay under: `{ "app.js": "40 kB" }`. A build over it warns and goes on. |
| `elements` | list of component names | `[]` | With `output_type: "elements"`, the components to publish as custom elements. |
| `media` | object | below | The image pipeline. |
| `pdf`, `slides` | object | below | The paper outputs. |

### `build.media`

The image pipeline: how the build resizes and re-encodes the pictures a
program names ([Media](https://webfluent.monzeromer.dev/recipes.md)).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `formats` | list of `webp`, `png`, `jpeg` | `["webp"]` | The formats every `image` is written in, best first, as well as its original. |
| `widths` | list of integers | `[480, 960, 1440, 1920]` | The widths written, where smaller than the original. |
| `quality` | integer, 1–100 | `78` | The encoding quality. WebP and JPEG are lossy at it. A file no smaller than the original is not written. |
| `pipeline` | boolean | `true` | `false` copies images as `public/` is copied, without resizing. |

### `build.pdf`

The page and type of a PDF build ([PDF and slides](https://webfluent.monzeromer.dev/learn/17-pdf-and-slides.md)).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `page_size` | string | `"A4"` | `A4`, `A3`, `A5`, `A6`, `Letter`, `Legal`, `Tabloid`, `Executive`, or two lengths (`"210mm 297mm"`). A `Document(size:)` overrides it. |
| `margins` | object of numbers | `72` each | `top`, `bottom`, `left`, `right`, in points (72 to the inch). The header and footer are drawn in them. |
| `default_font` | string | `"Helvetica"` | The document's family when its stylesheets name none: a standard name (`Helvetica`, `Times`, `Courier` — Liberation, built in) or a family under `fonts/`. |
| `default_font_size` | number | `12` | In points; `rem` in a style is this size. |
| `output_filename` | string | the project's name | The file written. |
| `fonts` | list of paths | `[]` | Font files or directories, read as well as the project's `fonts/`, `src/fonts/` and `public/fonts/`, which are always read. |
| `system_fonts` | boolean | `true` | Whether this machine's fonts may stand in for a family or a character nothing in the project has. The build names each one it uses. |

### `build.slides`

The page and type of a slide deck ([PDF and slides](https://webfluent.monzeromer.dev/learn/17-pdf-and-slides.md)).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `size` | string | `"16:9"` | `16:9` (960 × 540 pt), `4:3`, `A4-landscape`, or `"WIDTHxHEIGHT"` in points. |
| `width`, `height` | number | `null` | Explicit points, overriding `size`. |
| `margin` | number | `60` | The space around each slide's content, in points. What runs past it is clipped. |
| `default_font` | string | `"Helvetica"` | The family a slide's text is set in when nothing says otherwise. |
| `default_font_size` | number | `24` | The size a slide's text is set at, in points. |
| `show_slide_numbers` | boolean | `false` | Draws `n / total` in the bottom-right corner. |
| `footer_text` | string | `null` | Text in the bottom-left corner of every slide. |
| `background_color` | string | `null` | A full-bleed colour for every slide; a slide's own style overrides it. |
| `chrome_color` | string | `null` | The colour of the numbers and the footer. Unset, it flips between dark and light on each slide's background. |
| `output_filename` | string | the project's name | The file written. |
| `fonts`, `system_fonts` | list of paths, boolean | `[]`, `true` | As for `build.pdf`. |

## `dev`

The development server, `wf serve` ([Command line](https://webfluent.monzeromer.dev/reference/cli.md#wf-serve)).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `port` | integer | `3000` | The port `wf serve` listens on. |
| `hot_reload` | boolean | `true` | Rebuilds on a save, reloads open tabs, and draws a failed build's error over the page. |

## `motion`

What an animation does when the element says nothing
([Motion](https://webfluent.monzeromer.dev/recipes.md)).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `duration` | string | unset | How long an animation runs: `"180ms"`. |
| `easing` | string | unset | How it is paced: a CSS timing function, or a token such as `"$ease-standard"`. |

## `meta`

What every page's head carries: its fallbacks, its search and sharing tags,
and the other origins the content security policy allows.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `title` | string | the project's name | The title a page falls back to when it declares none. |
| `description` | string | `""` | The description a page falls back to when it declares none. |
| `lang` | language tag | `"en"` | The document's language: `en`, `ar`, `pt-BR`. A page written in another language says so with its own `lang:`. |
| `site_url` | string | `""` | The site's absolute address, such as `https://example.com`. Without it, canonical links, the sitemap's URLs and absolute sharing URLs are left out rather than guessed. |
| `site_name` | string | `""` | The name a link preview shows, and the site's owner in the structured data. |
| `owner` | `organization`, `person` | `"organization"` | Whether the site belongs to an organisation or a person: the JSON-LD node the site is published by ([Content](https://webfluent.monzeromer.dev/recipes.md)). |
| `same_as` | list of URLs | `[]` | The owner's profiles elsewhere, as its `sameAs`. |
| `job_title` | string | `""` | A person's `jobTitle`. |
| `owner_details` | map | `{}` | More about the owner, as schema.org properties merged into its node as written: `alternateName`, `email`, `worksFor`, `alumniOf`, `knowsAbout`, `logo`… The node's `@type`, `@id`, `name`, `url`, `jobTitle` and `sameAs` come from the settings above; naming one here is `E111`. |
| `image` | file | `""` | The sharing image for pages that set none. For an image in `public/`, its size is read into `og:image:width` and `og:image:height`. |
| `image_alt` | string | `""` | What that image shows, as `og:image:alt` and `twitter:image:alt`. A page's `image_alt:` wins. |
| `favicon` | file | `""` | The icon a browser tab shows: a file in `public/` — an SVG scales to every size — or a URL. |
| `touch_icon` | file | `""` | The icon a phone puts on its home screen: a 180×180 PNG in `public/`. iOS reads this, not an SVG `favicon`. |
| `sitemap` | boolean | `true` | Writes `sitemap.xml` and `robots.txt`. |
| `fonts` | list of URLs | `[]` | Web-font stylesheets to link, each with a `preconnect`. Their origins join the policy. |
| `preload` | list of paths | `[]` | Files on this site to fetch with the page, ahead of the stylesheets: a font a project `.css` file's `@font-face` names, the first screen's picture. `as` comes from the extension (a font is also `crossorigin`). A file on another origin is `E111` ([Content](https://webfluent.monzeromer.dev/recipes.md)). |
| `stylesheets` | list of paths or URLs | `[]` | Extra stylesheets to link ahead of `styles.css`: a file in `public/` or a URL. |
| `scripts` | list of scripts | `[]` | Libraries to load before the project's own scripts (below). Their origins join the policy ([JavaScript interop](https://webfluent.monzeromer.dev/recipes/library-from-a-cdn.md)). |
| `integrity` | map of URL to hash | `{}` | Subresource-integrity hashes of external assets, by URL. |
| `connect` | list of origins | `[]` | Origins a page may send requests to besides its own — an API elsewhere, an analytics endpoint — as the policy's `connect-src`: `"https://api.example.com"`, `"https://*.google-analytics.com"`, or an app's own scheme alone, `"ipc:"`. |
| `img` | list of origins | `[]` | Origins a page may load images from besides its own and `data:` — a CDN, a tag's tracking pixel — as the policy's `img-src`: `"https://images.example.com"`, `"https://www.googletagmanager.com"`. |
| `frame` | list of origins | `[]` | Origins a page may embed in a frame besides its own — a video player, a map — as the policy's `frame-src`: `"https://www.youtube-nocookie.com"`, or an app's own scheme alone, `"wfpreview:"`. |
| `wasm` | boolean | `false` | The pages compile WebAssembly: `script-src` gains `'wasm-unsafe-eval'`, which allows that and not `eval`. |
| `style_elements` | boolean | `false` | A script on the page writes `<style>` elements (an editor, a chart library): `style-src` gains `'unsafe-inline'`, for styles only. |

An entry of `connect`, `img` or `frame` that has a path, or that is a
keyword, is `E111`. [Security](https://webfluent.monzeromer.dev/about/security.md#allow-requests-images-and-frames-from-other-origins)
covers these settings and `wasm` and `style_elements`.

An entry of `meta.scripts` takes one of these forms:

| Form | Meaning |
|---|---|
| `"https://…"` or a file in `public/` | A classic script, linked `defer`. |
| `{ "src": "…", "module": true, "as": "Name" }` | An ES module, whose exports are put on `window.Name`. |
| `{ "src": "…", "globals": ["Chart"] }` | A script whose names `.wf` may call directly. |
| `{ "src": "…", "async": true }` | A script nothing waits for — analytics — loaded without holding up the page's code. |
| `{ "src": "…", "load": "after" }` | A script the page asks for only once it has loaded and painted. |

## `theme`

Which theme the site uses, and token values set over it
([Design tokens](https://webfluent.monzeromer.dev/reference/tokens.md)).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | string | unset | Which `theme` declaration is the site's. A project with exactly one need not name it; with none, the baseline applies. |
| `dark` | string | unset | The `theme` declaration that stands in for dark mode: when the reader prefers dark, or `setTheme("dark")` asks ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md#dark-mode)). |
| `builtin` | `full`, `structural` | `"full"` | `full` ships the built-ins' designed look; `structural` ships only their layout and mechanics, for a design of your own. |
| `tokens` | map of token to value | `{}` | Token values applied over the theme, for a pipeline to supply — a colour a deploy injects. |

## `i18n`

The project's translations. Absent, the project has one language.

```json
{ "i18n": { "default_locale": "en", "locales": ["en", "ar"], "dir": "src/translations" } }
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `default_locale` | language tag | `"en"` | The locale a page opens in, and the one a message falls back to when the current locale has no translation of it. |
| `locales` | list of language tags | `["en"]` | Every locale; one JSON file each, under `dir`. |
| `dir` | string | `"src/translations"` | The folder the translation files are in. |
| `literals` | boolean | `false` | Translates by the words themselves: each string a reader sees is its own key. |

See also: [Internationalisation](https://webfluent.monzeromer.dev/recipes.md).

## `env` and `public_env`

Values the program reads as `env.NAME`, and which of them may reach the
browser.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `env` | map | `{}` | Values read as `env.NAME`, fixed at build time. A `.env` file and the shell's variables add to them. |
| `public_env` | list of names | `[]` | The `env` names a page may read — and so ships to the browser — beyond those beginning `PUBLIC_`. |

See also: [Environments](https://webfluent.monzeromer.dev/recipes.md).

## `offline`

The service worker that stores the site for when the network is gone. Added
in **4.1**. Absent, no service worker is written.

```json
{ "offline": { "precache": ["/", "/docs/*"], "fallback": "/offline", "cache": { "/api/*": "network-first" }, "sync": true } }
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `precache` | list of globs | `["/"]` | The routes a first visit stores; `"*"` for every route. |
| `fallback` | string | `null` | A page's path, shown for a route that was not stored. |
| `cache` | map of glob to strategy | `{}` | How paths the build did not write are fetched: `network-first`, `cache-first`, `stale-while-revalidate` or `network-only`. |
| `sync` | boolean | `false` | Keeps writes made offline and sends them when the connection returns. |

See also: [Offline](https://webfluent.monzeromer.dev/recipes.md).

## `lints`

What a finding counts as, by its code or its family: `"off"`, `"warn"` or
`"error"`. A code's setting wins over its family's.

```json
{ "lints": { "A11": "off", "U": "error", "R01": "warn" } }
```

You can turn a warning off or make it an error. You can lower an error only
when it cannot ship a broken page:

- `R01` — a link to a route that is missing;
- `C02` — a prop nothing reads;
- `S04` — two pages on one route;
- `I01`, `I02`, `I04` — a message that shows its key or a placeholder.

Asking to lower any other error is itself refused, and so is a key that is
not a code or a family. To accept one finding on one line instead, write a
`// wf-allow(CODE)` comment; the same rule applies. Every code is listed in
[Diagnostics](https://webfluent.monzeromer.dev/errors.md).

## Next

[Diagnostics](https://webfluent.monzeromer.dev/errors.md) — every error and warning the compiler
reports.
