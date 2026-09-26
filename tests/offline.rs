//! What `offline` in the config makes a build write, and refuse.
//!
//! How the worker behaves — storing, serving with the server gone, a write
//! kept and sent, the update flow — is `tests/browser/offline.mjs`, in a real
//! Chrome, since a service worker is nothing without one. This holds what
//! the build decides: what `sw.js` stores and routes, its version, where the
//! bundle registers it, and the configurations it refuses.

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str, config: &str, app: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-offline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    std::fs::write(dir.join("src/App.wf"), app).unwrap();
    dir
}

fn build(dir: &Path) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(dir)
        .output()
        .expect("run wf");
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// The JSON the worker was built with.
fn worker_config(dir: &Path) -> serde_json::Value {
    let sw = std::fs::read_to_string(dir.join("build/sw.js")).expect("sw.js");
    let start = sw.find("const C = ").unwrap() + "const C = ".len();
    let end = start + sw[start..].find(";\n").unwrap();
    serde_json::from_str(&sw[start..end]).unwrap()
}

const PAGES: &str = r#"
app { Router }
page Home(path: "/", title: "Home", description: "d") { Heading("Home").h1 }
page About(path: "/about", title: "About", description: "d") { Heading("About").h1 }
page Contact(path: "/contact", title: "Contact", description: "d") { Heading("Contact").h1 }
page Offline(path: "/offline", title: "Offline", description: "d") { Heading("Offline").h1 }
"#;

fn config(ssg: bool, offline: &str, base: &str) -> String {
    format!(
        r#"{{ "name": "t", "build": {{ "output": "build", "ssg": {ssg}, "base_path": "{base}" }}, "offline": {offline} }}"#
    )
}

#[test]
fn a_static_build_stores_the_shell_the_named_routes_and_the_fallback() {
    let dir = scratch(
        "ssg",
        &config(
            true,
            r#"{ "precache": ["/", "/about"], "fallback": "/offline", "cache": { "/api/*": "network-first" }, "sync": true }"#,
            "",
        ),
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    let c = worker_config(&dir);
    let mut stored: Vec<&str> = c["precache"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    stored.sort();
    assert_eq!(
        stored,
        [
            "/about/index.html",
            "/app.js",
            "/index.html",
            "/offline/index.html",
            "/pages/About.js",
            "/pages/Home.js",
            "/pages/Offline.js",
            "/styles.css"
        ],
        "a route the config does not name — /contact — is not stored"
    );
    assert_eq!(
        c["fallback"], "/offline",
        "the fallback is a route to send a navigation to"
    );
    assert_eq!(c["sync"], true);
    assert_eq!(
        c["policies"][0],
        serde_json::json!(["^/api/.*$", "network-first"])
    );
    let routes: Vec<String> = c["routes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[0].as_str().unwrap().to_string())
        .collect();
    assert!(
        routes.contains(&"/about".to_string()) && routes.contains(&"/offline".to_string()),
        "{routes:?}"
    );

    let bundle = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(
        bundle.contains("WF.offline({sync:true})"),
        "the bundle registers the worker"
    );
    assert!(
        bundle.contains("wf:activate"),
        "and carries the module that does it"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_single_page_build_stores_the_shell_and_answers_its_routes_with_it() {
    let dir = scratch(
        "spa",
        &config(false, r#"{ "precache": ["/about"] }"#, ""),
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    let c = worker_config(&dir);
    assert_eq!(c["shell"], "/index.html");
    assert!(
        c["precache"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "/pages/About.js")
    );
    assert!(
        !c["precache"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "/pages/Contact.js")
    );
    // A stored route is answered by the shell; the worker has to know which
    // those are, or it would send them to the fallback like the rest.
    assert_eq!(c["routes"], serde_json::json!([["/about", "/index.html"]]));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn every_path_is_under_the_base() {
    let dir = scratch(
        "base",
        &config(true, r#"{ "fallback": "/offline" }"#, "/site"),
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    let c = worker_config(&dir);
    assert_eq!(c["base"], "/site");
    for path in c["precache"].as_array().unwrap() {
        assert!(path.as_str().unwrap().starts_with("/site/"), "{path}");
    }
    assert_eq!(c["fallback"], "/site/offline");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_version_moves_with_the_build_and_only_then() {
    let dir = scratch("version", &config(true, "{}", ""), PAGES);
    assert!(build(&dir).0);
    let first = worker_config(&dir)["version"].as_str().unwrap().to_string();
    assert!(build(&dir).0);
    assert_eq!(
        worker_config(&dir)["version"],
        first.as_str(),
        "nothing changed, so neither does the version"
    );
    std::fs::write(
        dir.join("src/App.wf"),
        PAGES.replace("Heading(\"About\")", "Heading(\"About us\")"),
    )
    .unwrap();
    assert!(build(&dir).0);
    assert_ne!(
        worker_config(&dir)["version"],
        first.as_str(),
        "a changed page is a new version"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn without_offline_nothing_is_registered_or_written() {
    let dir = scratch(
        "none",
        r#"{ "name": "t", "build": { "output": "build" } }"#,
        PAGES,
    );
    assert!(build(&dir).0);
    assert!(!dir.join("build/sw.js").exists());
    let bundle = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(!bundle.contains("WF.offline"), "no registration");
    assert!(!bundle.contains("wf:activate"), "and not the module either");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn what_offline_cannot_mean_is_refused() {
    let dir = scratch(
        "refused",
        &config(
            true,
            r#"{ "fallback": "/gone", "cache": { "/api/*": "network-last" } }"#,
            "",
        ),
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(!ok);
    assert!(
        out.contains("`network-last`") && out.contains("`network-first`"),
        "{out}"
    );
    assert!(out.contains("`/gone`, which no page's `path` is"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn update_and_network_queued_are_values_a_page_reads() {
    let dir = scratch(
        "values",
        &config(true, r#"{ "sync": true }"#, ""),
        r#"
page Home(path: "/", title: "Home", description: "d") {
    Heading("Home").h1
    Text("waiting: {network.queued}")
    if update.available { Button("Update") { on click { update.apply() } } }
}
"#,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "the type checker knows both: {out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_host_is_told_never_to_cache_the_worker() {
    let dir = scratch(
        "headers",
        r#"{ "name": "t", "build": { "output": "build", "ssg": true, "csp": true, "base_path": "/site" }, "offline": {} }"#,
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    let headers = std::fs::read_to_string(dir.join("build/_headers")).unwrap();
    assert!(
        headers.contains("/site/sw.js\n  Cache-Control: no-cache"),
        "{headers}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
