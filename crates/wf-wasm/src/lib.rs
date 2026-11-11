//! The WebFluent compiler in a browser.
//!
//! Built for `wasm32-unknown-unknown` and bound with `wasm-bindgen`, this is
//! what the docs' playground loads on its first edit: the real compiler,
//! with nothing sent to a server. Every function takes and returns JSON
//! text, so the page needs no generated types and the boundary costs one
//! string each way.
//!
//! ```js
//! import init, { compile, explain } from "./pkg/wf_wasm.js";
//! await init();
//! const out = JSON.parse(compile(JSON.stringify({
//!   "src/App.wf": "app { Router }",
//!   "src/pages/Home.wf": 'page Home(path: "/") { Text("Hi") }',
//! }), null));
//! out.files["index.html"]; out.diagnostics;
//! ```

use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
mod libc;

use wasm_bindgen::prelude::*;
use webfluent::memory::{self, Contents, Options};

/// The files a page hands over: each a string of text, or
/// `{ "base64": "…" }` for one that is not text (a picture, a font).
fn files_of(json: &str) -> Result<Vec<(String, Contents)>, String> {
    let map: BTreeMap<String, serde_json::Value> = serde_json::from_str(json)
        .map_err(|e| format!("the files are not a JSON object of path to contents: {e}"))?;
    map.into_iter()
        .map(|(path, value)| match value {
            serde_json::Value::String(text) => Ok((path, Contents::Text(text))),
            serde_json::Value::Object(o) => match o.get("base64").and_then(|b| b.as_str()) {
                Some(b64) => unbase64(b64)
                    .map(|bytes| (path.clone(), Contents::Bytes(bytes)))
                    .ok_or_else(|| format!("`{path}` is not valid base64")),
                None => Err(format!("`{path}` is an object without `base64`")),
            },
            _ => Err(format!("`{path}` is neither text nor {{ base64 }}")),
        })
        .collect()
}

fn unbase64(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\n' | b'\r' | b' ' => continue,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

fn build(files_json: &str, config_json: Option<String>, options: Options) -> String {
    let files = match files_of(files_json) {
        Ok(files) => files,
        Err(e) => {
            return serde_json::json!({
                "ok": false,
                "files": {},
                "diagnostics": [],
                "error": e,
                "lines": [],
            })
            .to_string();
        }
    };
    // There is no clock the compiler can read here; `ago(date)` measures
    // against the page's.
    webfluent::platform::set_unix_now((js_sys_now() / 1000.0) as i64);
    let out = memory::compile_files(files, config_json.as_deref(), &options);
    serde_json::to_string(&out).unwrap_or_else(|e| {
        format!(r#"{{"ok":false,"files":{{}},"diagnostics":[],"error":"{e}","lines":[]}}"#)
    })
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

/// A panic in the compiler is a bug in it; say what it was in the console
/// rather than leave the page with "unreachable".
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(|info| {
        console_error(&format!("webfluent: the compiler panicked: {info}"));
    }));
}

fn js_sys_now() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        date_now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}

/// Build a project: `files` is a JSON object of path → text (`src/App.wf`,
/// `src/pages/Home.wf`, `public/…`; a file that is not text as
/// `{ "base64": "…" }`), `config` the `webfluent.app.json` text or `null`.
/// Returns JSON: `{ ok, files: { path: text }, binary: { path: base64 },
/// diagnostics, error, lines }` — `diagnostics` exactly as
/// `wf check --format json` lists them.
#[wasm_bindgen]
pub fn compile(files: &str, config: Option<String>) -> String {
    build(files, config, Options::default())
}

/// Every finding, with nothing built — `wf check`. The same JSON as
/// [`compile`], with no files.
#[wasm_bindgen]
pub fn check(files: &str, config: Option<String>) -> String {
    build(
        files,
        config,
        Options {
            check_only: true,
            dev: false,
        },
    )
}

/// What `wf explain CODE` says, as JSON `{ code, severity, title, summary,
/// docs }`, or `null` for a code there is not.
#[wasm_bindgen]
pub fn explain(code: &str) -> String {
    memory::explain(code)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".to_string())
}

/// Take a font file (as it is, or zlib-compressed) as one of the faces a
/// PDF or a deck is set in by default — the Liberation faces, which this
/// build leaves out of its download. Returns the family, or `""` when the
/// bytes are not a font.
#[wasm_bindgen]
pub fn register_font(bytes: &[u8]) -> String {
    webfluent::paged::fonts::register_font(bytes.to_vec()).unwrap_or_default()
}

/// Whether this build renders PDFs and decks (the `pdf` feature).
#[wasm_bindgen]
pub fn has_pdf() -> bool {
    cfg!(feature = "pdf")
}

/// The compiler's version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_counter_compiles_through_the_entry_point() {
        let files = serde_json::json!({
            "src/App.wf": "app {\n    Router\n}\n",
            "src/pages/Home.wf": "page Home(path: \"/\", title: \"Counter\", description: \"Count.\") {\n    state count = 0\n    Heading(\"Counter\").h1\n    Button(\"+1\").primary { on click { count = count + 1 } }\n    Text(\"Count: {count}\")\n}\n",
        });
        let out: serde_json::Value =
            serde_json::from_str(&compile(&files.to_string(), None)).unwrap();
        assert_eq!(out["ok"], true, "{out:#}");
        assert!(out["files"]["index.html"].is_string());
        assert!(out["files"]["app.js"].is_string());
    }

    #[test]
    fn base64_reads_back_what_was_written() {
        let bytes: Vec<u8> = (0..=255).collect();
        let text = webfluent::media::probe::base64(&bytes);
        assert_eq!(unbase64(&text).unwrap(), bytes);
    }

    #[test]
    fn bad_input_is_an_error_not_a_panic() {
        let out: serde_json::Value = serde_json::from_str(&compile("[1]", None)).unwrap();
        assert_eq!(out["ok"], false);
        assert_eq!(explain("nope"), "null");
    }
}
