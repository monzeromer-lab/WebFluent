# wf-wasm

The WebFluent compiler as WebAssembly, for a playground that compiles in the
browser with nothing sent to a server.

```bash
cargo install wasm-bindgen-cli --version 0.2.130   # once
crates/wf-wasm/build.sh                            # writes crates/wf-wasm/pkg/
python3 -m http.server 8765 --directory crates/wf-wasm   # then open /demo.html
```

`demo.html` compiles a counter page as you type, shows the page, every file
the build wrote, and the findings.

## API

Every function takes and returns JSON text.

| Function | Returns |
|---|---|
| `compile(files, config)` | `{ ok, files: { path: text }, diagnostics, error, lines }` — `files` in is `{ "src/App.wf": "…", … }`, `config` the `webfluent.app.json` text or `null` |
| `check(files, config)` | the same, nothing built: `wf check` |
| `explain(code)` | `{ code, severity, title, summary, docs }` or `null` |
| `version()` | the compiler's version |

`diagnostics` are the objects `wf check --format json` lists, from the same
pipeline (`diagnostics::check::check_project`). Paths in them are under
`/wf-project/`, where the project seems to sit.

The Rust side is `webfluent::memory::compile`, which runs natively too and is
what the tests exercise.

## Size

The `wasm-release` profile (`opt-level = "z"`, LTO, one codegen unit,
`panic = "abort"`), names and producers sections removed:

| | raw | gzip -9 |
|---|---|---|
| `wf_wasm_bg.wasm` | 4.4 MB | 1.48 MB |
| `wf_wasm.js` glue | 10 KB | — |

A plain `--release` build with the PDF engine in is 17 MB / 6.5 MB.
`wasm-opt -Oz` (binaryen) runs in `build.sh` when it is on the path; it was
not measured here.

## What is not here

- **PDF and slides.** Built without the `pdf` feature: a project with
  `output_type: "pdf"` or `"slides"` reports that this build has no PDF
  engine (its checks still run). The engine itself is pure Rust and compiles
  to wasm; what it costs is the 2.4 MB of built-in Liberation fonts plus
  krilla/usvg/rustybuzz code, and with no system fonts any other family must
  be handed in as bytes. Turning `pdf` on in `Cargo.toml` is all it takes to
  try it.
- **Lossy WebP.** libwebp is C; without `native-media` the image pipeline
  writes no WebP and the original picture serves.
- **Time.** `ago(date)` measures against the `Date.now()` the entry point
  hands in; a build reports a duration of zero.
- **`env`.** There is no process environment; `.env` and the config's `env`
  are read as usual.
