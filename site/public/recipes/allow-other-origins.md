# Allow other origins in the security policy

> Widen the content security policy a build ships: requests, images and frames from other origins, an app's own schemes, WebAssembly, and pinned files.

The page: https://webfluent.monzeromer.dev/recipes/allow-other-origins

With `"build": { "csp": true }` (on for a project `wf init` creates), every
page carries a Content-Security-Policy that is `'self'` throughout. A page
that talks to an API on another domain, loads images from a CDN or embeds
a video names those origins in `webfluent.app.json`, and the build widens
the policy by exactly them. This config loads Google's tag manager after the
page and allows what it sends and draws:

```json
{
  "meta": {
    "scripts": [{ "src": "https://www.googletagmanager.com/gtag/js?id=G-XXXXXXX", "load": "after" }],
    "connect": ["https://*.google-analytics.com", "https://*.analytics.google.com",
                "https://www.googletagmanager.com"],
    "img": ["https://*.google-analytics.com", "https://www.googletagmanager.com"]
  }
}
```

## How it works

- **Requests.** A page may send requests only to its own origin. Each
  entry of `meta.connect` — an origin such as `https://api.example.com`, or
  `https://*.example.com` for its subdomains — joins `connect-src`.
- **Images.** A page may load images from its own origin and `data:`,
  unless `meta.img` names others, which join `img-src`. Name a CDN there,
  or the pixels a tag requests (`https://www.googletagmanager.com/a`,
  `/td`), which a tag assistant otherwise reports as blocked.
- **Frames.** `meta.frame` names the origins a page may embed — a video
  player (`https://www.youtube-nocookie.com`), a map — and they join
  `frame-src`.
- **Fonts, stylesheets and scripts** need nothing here: the origins
  `meta.fonts`, `meta.stylesheets` and `meta.scripts` declare are added to
  the policy already, and your own scripts under `src/` are same-origin.
- An entry with a path, or a keyword such as `'self'`, is `E111`.

A tag manager's own set-up — the `dataLayer` and the `gtag('config', …)`
call a vendor gives you as an inline `<script>` — goes in a plain script
under `src/`, which the policy already allows: the build refuses an inline
script ([Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)).

## Variations

- **An app's own scheme.** An app that serves its pages itself — a desktop
  shell — has schemes of its own. `meta.connect` and `meta.frame` take one
  as a scheme alone: `"ipc:"` for the channel a shell's pages call it on,
  `"wfpreview:"` for a frame the app fills from its own protocol. A scheme
  named in `meta.frame` is also one a frame's `src` may use: the address is
  kept, where any other unknown scheme is emptied. The web's schemes are
  refused: `https:` alone would allow every origin, and `data:`, `blob:`
  and `javascript:` run whatever a page writes
  ([Make a desktop or phone app with Tauri](https://webfluent.monzeromer.dev/recipes/tauri-app.md)).
- **WebAssembly.** A page that compiles WebAssembly — a parser, an image
  codec — sets `meta.wasm: true`, and `script-src` gains
  `'wasm-unsafe-eval'`. WebAssembly compiles; `eval` is still refused.
- **Style elements.** A page whose scripts write `<style>` elements — an
  editor such as CodeMirror, a chart library — sets
  `meta.style_elements: true`, and `style-src` gains `'unsafe-inline'`, for
  styles only.
- **Pin another origin's files.** A stylesheet or a font from another
  origin is code that origin can change after you read it. Declare its hash
  in `meta.integrity`, keyed by its URL:

  ```json
  {
    "meta": {
      "stylesheets": ["https://cdn.example.com/brand.css"],
      "integrity": { "https://cdn.example.com/brand.css": "sha384-…" }
    }
  }
  ```

  The build emits `integrity` and `crossorigin="anonymous"` on the tag, and
  warns (`D05`) about a declared external asset that has no hash. It never
  fetches the file to work the hash out, so a build does not depend on the
  network. A Google Fonts stylesheet is exempt: Google writes it per
  browser, so its bytes differ between readers and no hash can match. A
  library is pinned the same way
  ([Load a library from a CDN](https://webfluent.monzeromer.dev/recipes/library-from-a-cdn.md)).

[Security](https://webfluent.monzeromer.dev/about/security.md#the-content-security-policy) explains the
policy itself, and how the build holds every page to it.
