### External libraries

A script from another origin is listed in `meta.scripts` and called from your own scripts.

The compiler never fetches anything, so a library on another origin is a URL
in the config, used from your own scripts:

```json
{ "meta": {
    "scripts": [
      "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js",
      { "src": "https://esm.example.com/confetti.mjs", "module": true, "as": "Confetti" },
      { "src": "https://cdn.example.com/lib.js", "globals": ["Lib"] },
      { "src": "https://www.googletagmanager.com/gtag/js?id=G-XXXXXXX", "load": "after" }
    ],
    "integrity": { "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js": "sha384-…" }
} }
```

Each is linked `defer` in the order listed, before the project's scripts; a
module is imported by a generated `externals.js` and its exports put on
`window.<as>`; `globals` are names `.wf` may call directly, each taking and giving `Json`, and a
class among them is constructed when called. `"async": true` loads one
nothing waits for (analytics, a tag manager) with `<script async>`, so it
never holds up the page's code; `"load": "after"` goes further and has no
tag at all — the runtime asks for it once the page has loaded and drawn its
first frame, so it never counts against how soon the page is ready.
`"load": "consent"` waits further still: no tag, and nothing asked for
until the reader accepts in a `ConsentBanner` (then on every visit after).
None of these can be a module or have `globals` (`E111`). Their origins join
`script-src`, and one without an integrity hash draws a warning.

