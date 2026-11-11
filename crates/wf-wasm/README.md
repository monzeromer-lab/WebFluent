# wf-wasm

The WebFluent compiler as WebAssembly: sites, apps, pictures, PDFs and slide
decks built in the browser, with nothing sent to a server.

```bash
cargo install wasm-bindgen-cli --version 0.2.130          # once
crates/wf-wasm/build.sh                                   # pkg/, pkg-pdf/
python3 -m http.server 8765 --directory crates/wf-wasm    # open /demo.html
```

`build.sh` needs the `wasm32-unknown-unknown` target and a clang that
targets wasm32, with its `llvm-ar` (for libwebp; set `CC_wasm32_unknown_unknown`
and `AR_wasm32_unknown_unknown` to pick others). It runs `wasm-opt -Oz` when
one is on the path.

`demo.html` has four examples: a site of two pages, a picture sized to WebP,
a PDF report with Arabic in it, and a slide deck.

## Using it

Import `wf.js`; it loads what a project needs, when it needs it.

```js
import { compile, preview, previewConfig, explain, preload } from "./wf.js";

preload();                                   // on the first edit, or sooner
const out = await compile(
  { "src/App.wf": "…", "src/pages/Home.wf": "…", "public/hero.png": pngBytes },
  previewConfig({ name: "Playground" }),     // or any webfluent.app.json text
);
out.ok; out.diagnostics;                     // as `wf check --format json`
out.files["index.html"];                     // text the build wrote
out.binary["Report.pdf"];                    // Uint8Array: pictures, a PDF
iframe.src = await preview(out);             // a site, served as a host would
```

| | |
|---|---|
| `compile(files, config)` | `files`: path → text or bytes (`Uint8Array`). Resolves to `{ ok, files, binary, diagnostics, error, lines }` |
| `check(files, config)` | every finding, nothing built (`wf check`) |
| `preview(out, route?)` | publishes the build to a service worker (`preview-sw.js`) that serves it under `preview/` — every page, chunk, stylesheet and picture, routes included — and resolves to the address for an iframe. Build with `previewConfig(config)` so the pages ask for their files there. The iframe needs `sandbox="allow-scripts allow-same-origin"`; give the preview an origin of its own if the page around it holds anything worth protecting |
| `explain(code)` | `{ code, severity, title, summary, docs }` or `null` |
| `registerFont(bytes)` | a face documents may be set in or fall back to (TTF/OTF, as it is, zlib or gzip); resolves to its family |
| `preload({ document })` | start loading before the first call |

### What is loaded, and when

| | raw | gzip -9 | loaded |
|---|---|---|---|
| `pkg/wf_wasm_bg.wasm` — the compiler, with WebP | 4.65 MB | 1.60 MB | on the first `compile` (or `preload`) |
| `pkg-pdf/wf_wasm_pdf_bg.wasm` — the same with the PDF engine | 8.29 MB | 2.70 MB | the first time a config says `output_type: "pdf"` or `"slides"` |
| `pkg-pdf/fonts/*.ttf.z` — the 12 Liberation faces | 2.37 MB (already compressed) | — | with the PDF build |
| `pkg-pdf/fonts/NotoSansArabic.ttf.gz` | 455 KB | — | the first time a document's text has Arabic in it |

Timings in Chrome: a two-page site compiles in about 30–60 ms; a PDF in
about 120–240 ms; a 1600×900 picture's first build about 2 s (four WebP
widths), later builds reuse the encodings (about 250 ms) — `memory::compile`
keeps `.wf-cache/media` between builds, up to 64 MB.

### The lower level

`pkg/wf_wasm.js` itself exports `compile(filesJson, configJson)`,
`check`, `explain`, `register_font(bytes)`, `has_pdf()` and `version()`,
JSON text in and out; a file that is not text is `{ "base64": "…" }` in
`filesJson`, and `binary` comes back base64. The Rust side is
`webfluent::memory::compile_files`, which runs natively too and is what
the tests exercise.

## How each part works in a browser

- **No disk.** The project is read from a map (`memory::compile_files`
  over an `OverlayVfs` with nothing beneath it); what a build writes goes
  to memory (`outfs::capture`) and comes back as the result.
- **No threads, no clock, no environment.** `platform` runs the checks one
  after another, reports a build's duration as zero, takes `Date.now()`
  for `ago(date)`, and reads no process environment (`.env` and the
  config's `env` are read as usual).
- **WebP is C.** `libc/` is a C library just large enough for libwebp:
  freestanding headers, `shim.c` (sort, search, a stdio that writes
  nowhere) and `src/libc.rs` (the allocator is Rust's, the maths the `libm`
  crate's).
- **Fonts.** The PDF build leaves the Liberation faces out of its download
  (`bundled-fonts` off) and `wf.js` hands them to `register_font`; a face
  registered there stands where a built-in one does. A browser has no
  machine fonts to fall back on, so a script the Liberation faces lack needs
  a face registered for it — Arabic is shipped; for another, register one
  (`register_font` takes a TTF/OTF as it is, zlib or gzip) or put it in the
  project's `fonts/` as bytes.
