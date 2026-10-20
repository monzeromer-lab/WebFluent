//! W8: `worker name = spawn(…)`, held like a socket, on a page or in a
//! store — and a store's connections, which it now opens. What the
//! runtime does is `tests/js/worker.test.mjs` and `tests/browser/worker.mjs`.

mod common;

use common::{page, spa_generated};

fn d01(src: &str, files: &[&str]) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!("wf-worker-d01-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for f in files {
        let at = dir.join("public").join(f);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, "self.onmessage = () => {};").unwrap();
    }
    std::fs::create_dir_all(dir.join("public")).unwrap();
    let (program, out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
    let found =
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(source.clone()),
            dir: Some(&dir),
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics
        .into_iter()
        .filter(|d| d.code == "D01")
        .map(|d| d.message)
        .collect();
    let _ = std::fs::remove_dir_all(&dir);
    found
}

#[test]
fn a_page_s_worker_is_a_connection_it_owns() {
    let src = page(
        "state said = \"\"\n    worker lang = spawn(\"/workers/lang.js\", restart: 1) {\n        on message(m) { said = m.text }\n    }\n    Text(\"{lang.pending}\")\n    match lang {\n        open { Text(\"open\") }\n        error(e) { Text(e.message) }\n        else { Text(\"closed\") }\n    }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains(
            "const _lang = WF.worker(\"/workers/lang.js\", { restart: 1, onMessage: (m) => {"
        ),
        "{js}"
    );
    assert!(js.contains("_lang.pending()"), "pending is a signal: {js}");
}

#[test]
fn a_store_opens_its_connections_and_reads_them_in_its_own_terms() {
    let src = "store Lang {\n    state said = \"\"\n    worker lang = spawn(\"/workers/lang.js\") {\n        on message(m) { said = m.text }\n    }\n    socket chat = ws(\"wss://example.com/chat\")\n    derived busy = lang.pending > 0\n    action ask(text: String) {\n        said = await lang.request(\"upper\", { text: text })\n    }\n}\npage P(path: \"/\", title: \"T\") {\n    Text(\"{Lang.lang.pending} {Lang.chat.state}\")\n}\n";
    let js = spa_generated(src);
    assert!(js.contains("connections: {"), "{js}");
    assert!(
        js.contains("lang: (store) => WF.worker(\"/workers/lang.js\", {"),
        "{js}"
    );
    assert!(
        js.contains("store.said = m.text;"),
        "its handler writes the store: {js}"
    );
    assert!(
        js.contains("chat: (store) => WF.ws(\"wss://example.com/chat\", {"),
        "a socket in a store is opened too; it used to be dropped: {js}"
    );
    assert!(js.contains("(store.lang.pending() > 0)"), "{js}");
    assert!(js.contains("await store.lang.request(\"upper\""), "{js}");
    assert!(js.contains("Lang.lang.pending()"), "{js}");
    assert!(js.contains("Lang.chat.state()"), "{js}");
}

#[test]
fn a_worker_s_script_must_be_a_file_of_public() {
    let src = page("worker lang = spawn(\"/workers/lang.js\")\n    Text(\"x\")");
    assert_eq!(d01(&src, &["workers/lang.js"]), Vec::<String>::new());
    let missing = d01(&src, &[]);
    assert_eq!(missing.len(), 1, "{missing:?}");
    assert!(missing[0].contains("never starts"), "{missing:?}");
    let elsewhere = page("worker lang = spawn(\"https://cdn.example/lang.js\")\n    Text(\"x\")");
    assert!(
        d01(&elsewhere, &[]).is_empty(),
        "another origin's is not ours to check"
    );
}

#[test]
fn spawn_is_how_a_worker_opens() {
    let src = page("worker lang = ws(\"/x\")\n    Text(\"x\")");
    let err = match webfluent::parse_source(&src, "t.wf") {
        Ok(_) => String::new(),
        Err(e) => e.to_string(),
    };
    assert!(err.contains("A `worker` opens with `spawn(…)`"), "{err}");
}
