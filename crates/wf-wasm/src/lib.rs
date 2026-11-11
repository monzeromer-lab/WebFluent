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

use wasm_bindgen::prelude::*;
use webfluent::memory::{self, Options};

fn build(files_json: &str, config_json: Option<String>, options: Options) -> String {
    let files: BTreeMap<String, String> = match serde_json::from_str(files_json) {
        Ok(files) => files,
        Err(e) => {
            return serde_json::json!({
                "ok": false,
                "files": {},
                "diagnostics": [],
                "error": format!("the files are not a JSON object of path to text: {e}"),
                "lines": [],
            })
            .to_string();
        }
    };
    // There is no clock the compiler can read here; `ago(date)` measures
    // against the page's.
    webfluent::platform::set_unix_now((js_sys_now() / 1000.0) as i64);
    let out = memory::compile(&files, config_json.as_deref(), &options);
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
/// `src/pages/Home.wf`, `public/…`), `config` the `webfluent.app.json` text
/// or `null`. Returns JSON: `{ ok, files: { path: text }, diagnostics,
/// error, lines }` — `diagnostics` exactly as `wf check --format json`
/// lists them.
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
    fn bad_input_is_an_error_not_a_panic() {
        let out: serde_json::Value = serde_json::from_str(&compile("[1]", None)).unwrap();
        assert_eq!(out["ok"], false);
        assert_eq!(explain("nope"), "null");
    }
}
