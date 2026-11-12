//! A burst of edits is checked once it pauses, and what is reported is
//! what the last edit says — not one full check of the project per key.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn send(stdin: &mut impl Write, message: serde_json::Value) {
    let body = message.to_string();
    write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
    stdin.flush().unwrap();
}

/// Every message the server writes, as it writes it.
fn messages(stdout: impl Read + Send + 'static) -> mpsc::Receiver<serde_json::Value> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut length = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    return;
                }
                let line = line.trim();
                if line.is_empty() {
                    break;
                }
                if let Some(n) = line.strip_prefix("Content-Length: ") {
                    length = n.parse().unwrap();
                }
            }
            let mut body = vec![0; length];
            if reader.read_exact(&mut body).is_err() {
                return;
            }
            if tx.send(serde_json::from_slice(&body).unwrap()).is_err() {
                return;
            }
        }
    });
    rx
}

#[test]
fn the_file_cache_forgets_the_projects_nothing_is_open_in() {
    use wf_lsp::project::{FileCache, Project};
    let base = std::env::temp_dir().join(format!("wf-lsp-cache-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let mut uris = Vec::new();
    for name in ["one", "two"] {
        let dir = base.join(name);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "c" }"#).unwrap();
        std::fs::write(dir.join("src/App.wf"), "app { Router }\n").unwrap();
        std::fs::write(
            dir.join("src/Home.wf"),
            "page Home(path: \"/\", title: \"H\") { Text(\"x\") }\n",
        )
        .unwrap();
        uris.push((
            dir.clone(),
            lsp_types::Url::from_file_path(dir.join("src/App.wf")).unwrap(),
        ));
    }
    let cache = FileCache::default();
    for (_, uri) in &uris {
        Project::load_with(uri, &webfluent::vfs::FsVfs, &cache);
    }
    let both = cache.len();
    assert!(both >= 4, "{both}");
    cache.retain_under(&[uris[0].0.clone()]);
    assert_eq!(cache.len(), both / 2);
    cache.retain_under(&[]);
    assert!(cache.is_empty());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn a_burst_of_edits_is_checked_once_and_reports_the_last() {
    let dir = std::env::temp_dir().join(format!("wf-lsp-debounce-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "d" }"#).unwrap();
    let file = dir.join("src/App.wf");
    let good = "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") { Heading(\"Home\").h1 }\n";
    std::fs::write(&file, good).unwrap();
    let uri = format!("file://{}", file.display());

    let mut server = Command::new(env!("CARGO_BIN_EXE_wf-lsp"))
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("wf-lsp runs");
    let mut stdin = server.stdin.take().unwrap();
    let rx = messages(server.stdout.take().unwrap());

    send(
        &mut stdin,
        serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "capabilities": {} } }),
    );
    let until = Instant::now() + Duration::from_secs(20);
    while rx
        .recv_timeout(Duration::from_secs(20))
        .map(|m| m["id"] != 1)
        .unwrap_or(false)
    {
        assert!(Instant::now() < until, "no answer to initialize");
    }
    send(
        &mut stdin,
        serde_json::json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
    );
    send(
        &mut stdin,
        serde_json::json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": { "uri": uri, "languageId": "webfluent", "version": 1, "text": good } } }),
    );
    // Forty keystrokes at once, the last of which names a component that
    // does not exist.
    let edits = 40;
    for v in 0..edits {
        let text = if v + 1 == edits {
            good.replace("Heading(\"Home\").h1", "Nope(\"Home\")")
        } else {
            good.replace("Home\").h1", &format!("Home{v}\").h1"))
        };
        send(
            &mut stdin,
            serde_json::json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
                "textDocument": { "uri": uri, "version": v + 2 },
                "contentChanges": [{ "text": text }] } }),
        );
    }
    let mut published = Vec::new();
    let quiet_for = Duration::from_millis(1500);
    while let Ok(message) = rx.recv_timeout(quiet_for) {
        if message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri.as_str()
        {
            published.push(message["params"]["diagnostics"].clone());
        }
    }
    let _ = server.kill();
    let _ = server.wait();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(!published.is_empty(), "nothing was reported");
    // The open, and a handful of checks for the burst: never one per edit.
    assert!(
        published.len() <= 1 + edits / 4,
        "{} reports for {edits} edits",
        published.len()
    );
    let last = published.last().unwrap().to_string();
    assert!(
        last.contains("Nope"),
        "the last report is the last edit's: {last}"
    );
}
