//! The dev server and `wf verify`, run as a developer runs them
//! (`spec/DIAGNOSTICS_PLAN.md`, Part E): what `wf serve` answers and how it
//! builds, and what `wf verify` visits and refuses in a real browser.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn project(name: &str, config: &str, app: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-sv-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    std::fs::write(dir.join("src/App.wf"), app).unwrap();
    dir
}

fn wf(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("wf runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// `GET path` with `headers`: the status code and the body.
fn get(port: u16, path: &str, headers: &str) -> Option<(u16, String)> {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .ok()?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{headers}Connection: close\r\n\r\n"
    )
    .ok()?;
    let mut reply = Vec::new();
    stream.read_to_end(&mut reply).ok()?;
    let reply = String::from_utf8_lossy(&reply).to_string();
    let status = reply.split_whitespace().nth(1)?.parse().ok()?;
    let body = reply
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    Some((status, body))
}

/// A port nothing is listening on.
fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// A `wf serve` that stops when the test does, passing or not.
struct Serving(Option<Child>);

impl Serving {
    /// Stop it, and hand back what it printed to standard error.
    fn stop(&mut self) -> String {
        let Some(mut child) = self.0.take() else {
            return String::new();
        };
        let _ = child.kill();
        let _ = child.wait();
        let mut err = String::new();
        if let Some(mut pipe) = child.stderr.take() {
            let _ = pipe.read_to_string(&mut err);
        }
        err
    }
}

impl Drop for Serving {
    fn drop(&mut self) {
        self.stop();
    }
}

#[test]
fn the_dev_server_answers_only_a_navigation_with_the_shell_and_builds_for_development() {
    let port = free_port();
    let dir = project(
        "serve",
        &format!(r#"{{ "name": "s", "dev": {{ "port": {port}, "hot_reload": true }} }}"#),
        "app { Router }\ntype User { id: String, name: String }\npage Home(path: \"/\", title: \"H\", description: \"D\") {\n    resource users: [User] = fetch(\"/api/users\")\n    Heading(\"Home\").h1\n    match users { ready(list) { Text(\"{list.length}\") } else { Text(\"…\") } }\n}\n",
    );
    let child = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("serve")
        .current_dir(&dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("wf serve starts");
    let mut serving = Serving(Some(child));
    let started = Instant::now();
    while get(port, "/__wf/status", "").is_none_or(|(s, _)| s != 200) {
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "wf serve never answered"
        );
        std::thread::sleep(Duration::from_millis(200));
    }

    // A navigation gets the page, with the script that says it is dev first.
    let (status, body) = get(port, "/anywhere", "Sec-Fetch-Mode: navigate\r\n").unwrap();
    assert_eq!(status, 200);
    let head = body.find("<head>").expect("a document");
    assert!(
        body[head..].starts_with("<head><script src=\"/__wf/mode.js\"></script>"),
        "{body}"
    );
    let (status, mode) = get(port, "/__wf/mode.js", "").unwrap();
    assert_eq!((status, mode.trim()), (200, "window.__WF_DEV__ = true;"));

    // A fetch, a file: a 404, not the shell.
    let (status, body) = get(port, "/api/users", "Sec-Fetch-Mode: cors\r\n").unwrap();
    assert_eq!(status, 404, "{body}");
    assert!(!body.contains("<html"), "{body}");
    assert_eq!(get(port, "/missing.json", "").unwrap().0, 404);
    assert_eq!(
        get(port, "/favicon.ico", "Sec-Fetch-Mode: no-cors\r\n")
            .unwrap()
            .0,
        204
    );
    // What the build wrote is still served.
    assert_eq!(
        get(port, "/app.js", "Sec-Fetch-Mode: no-cors\r\n")
            .unwrap()
            .0,
        200
    );

    // A development build: the resource carries its type's shape.
    let page = std::fs::read_to_string(dir.join("build/pages/Home.js")).unwrap();
    assert!(
        page.replace(' ', "").contains(r#"shape:["l",["r","User""#),
        "{page}"
    );

    let err = serving.stop();
    assert!(
        err.contains("404 GET /api/users"),
        "the terminal says which: {err}"
    );

    // `wf build` writes for a host: no shapes.
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    let page = std::fs::read_to_string(dir.join("build/pages/Home.js")).unwrap();
    assert!(!page.replace(' ', "").contains("shape:["), "{page}");
}

/// `wf verify` with these arguments, or `None` where the machine has no
/// browser to run it in.
fn verify(dir: &Path, args: &[&str]) -> Option<(i32, String, String)> {
    let mut all = vec!["verify"];
    all.extend_from_slice(args);
    let out = wf(dir, &all);
    if out.2.contains("no Chrome or Chromium") {
        // CI has a browser; a skip there would be a gap nobody reads.
        assert!(
            std::env::var_os("WF_REQUIRE_CHROME").is_none(),
            "WF_REQUIRE_CHROME is set and there is no browser: {}",
            out.2
        );
        eprintln!("\n  SKIPPED: no Chrome or Chromium for `wf verify`.\n");
        return None;
    }
    Some(out)
}

#[test]
fn verify_visits_param_routes_and_a_returning_visitor() {
    let dir = project(
        "verify",
        r#"{ "name": "v" }"#,
        "app { Router }\ntype Item { n: Number }\npage Home(path: \"/\", title: \"H\", description: \"D\") {\n    persist items: [Item] = [Item(n: 1)]\n    Heading(\"Home {items.map(i => i.n.toFixed(1)).join(\\\", \\\")}\").h1\n}\npage User(path: \"/user/:id\", title: \"U\", description: \"D\", id: String) { Heading(\"User {id}\").h1 }\n",
    );
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    let Some((code, out, err)) = verify(&dir, &["--json", "--returning-visitor"]) else {
        return;
    };
    assert_eq!(code, 0, "{out}\n{err}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let urls: Vec<&str> = v["routes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["url"].as_str().unwrap())
        .collect();
    assert_eq!(
        urls,
        vec!["/", "/user/1", "/ (returning)", "/user/1 (returning)"]
    );

    // What a returning reader holds is of a shape this build cannot read —
    // as when a build changed the shape with no new `version:` (D04): the
    // returning visit shows the page it would break.
    std::fs::write(
        dir.join(".wf-cache/persist-values.previous.json"),
        r#"{ "wf:Home.items": ["a"] }"#,
    )
    .unwrap();
    let (code, out, _) = verify(&dir, &["--json", "--returning-visitor"]).unwrap();
    assert_ne!(code, 0, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let returning = v["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["url"] == "/ (returning)")
        .unwrap();
    let errors = returning["errors"].to_string();
    assert!(errors.contains("uncaught"), "{errors}");
    assert!(errors.contains("the router drew nothing"), "{errors}");
    // The first visit, with nothing stored, is fine.
    let first = v["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["url"] == "/")
        .unwrap();
    assert_eq!(first["errors"], serde_json::json!([]), "{first}");
}

#[test]
fn verify_fails_a_fetch_the_build_has_no_file_for() {
    let dir = project(
        "verify-fetch",
        r#"{ "name": "v" }"#,
        "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") {\n    resource rows = fetch(\"/api/rows\")\n    Heading(\"Home\").h1\n    match rows { ready(r) { Text(\"rows\") } error(e) { Text(\"failed\") } else { Text(\"…\") } }\n}\n",
    );
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    let Some((code, out, _)) = verify(&dir, &["--json"]) else {
        return;
    };
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("404") && out.contains("/api/rows"), "{out}");
}
