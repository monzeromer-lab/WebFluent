//! What the service worker `offline` writes does with what it stores, run in
//! Node against a fake `caches`, `fetch` and `IndexedDB`: a site under a base
//! path keeps its stores, its runtime answers and its kept writes apart from
//! another site's on the same origin; an answer marked `no-store` or
//! `private`, asked for with credentials or varying by them is not kept; the
//! runtime store is bounded; and a kept write is sent without credentials.
//!
//! And Markdown's place markers, which the compiler's renderer, like the
//! runtime's, never reads from the text it is given.

use std::path::PathBuf;
use std::process::Command;

use webfluent::codegen::markdown;
use webfluent::codegen::offline::{Worker, service_worker};

fn has_node() -> bool {
    Command::new("node").arg("--version").output().is_ok()
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/runtime-fixes")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn worker(base: &str, sync: bool) -> String {
    service_worker(&Worker {
        version: "00000000000000aa",
        base,
        precache: vec![format!("{base}/"), format!("{base}/app.js")],
        routes: vec![("/".into(), format!("{base}/index.html"))],
        shell: None,
        fallback: None,
        policies: vec![("/api/*".into(), "network-first".into())],
        sync,
    })
}

/// The fakes the worker runs against, and the scenarios, in one script that
/// prints what it saw as JSON.
const HARNESS: &str = r#"
const fs = require("node:fs");
const vm = require("node:vm");
const source = fs.readFileSync(process.argv[2], "utf8");
const scenario = process.argv[3];
const ORIGIN = "https://site.example";

class FakeCache {
  constructor() { this.entries = []; }
  async put(req, res) {
    const url = typeof req === "string" ? new URL(req, ORIGIN).href : req.url;
    this.entries = this.entries.filter((e) => e.url !== url);
    this.entries.push({ url, res, req: new Request(url) });
  }
  async match(req) {
    const url = typeof req === "string" ? new URL(req, ORIGIN).href : req.url;
    const hit = this.entries.find((e) => e.url === url);
    return hit ? hit.res.clone() : undefined;
  }
  async keys() { return this.entries.map((e) => e.req); }
  async delete(req) {
    const url = typeof req === "string" ? new URL(req, ORIGIN).href : req.url;
    const before = this.entries.length;
    this.entries = this.entries.filter((e) => e.url !== url);
    return this.entries.length !== before;
  }
  async addAll() {}
}
const stores = new Map();
const caches = {
  async open(name) { if (!stores.has(name)) stores.set(name, new FakeCache()); return stores.get(name); },
  async keys() { return [...stores.keys()]; },
  async delete(name) { return stores.delete(name); },
  async match(req) {
    for (const c of stores.values()) { const hit = await c.match(req); if (hit) return hit; }
    return undefined;
  },
};

// IndexedDB, as much as the worker asks of it.
const dbs = new Map();
const later = (fn) => setTimeout(fn, 0);
const indexedDB = {
  open(name) {
    const r = {};
    later(() => {
      let db = dbs.get(name);
      const fresh = !db;
      if (!db) { db = { stores: new Map(), next: 1 }; dbs.set(name, db); }
      r.result = {
        createObjectStore: (s) => db.stores.set(s, new Map()),
        transaction: (s) => {
          const tx = {};
          const store = db.stores.get(s);
          tx.objectStore = () => ({
            getAll: () => { const q = {}; later(() => { q.result = [...store.values()]; q.onsuccess && q.onsuccess(); }); return q; },
            delete: (id) => store.delete(id),
          });
          later(() => later(() => tx.oncomplete && tx.oncomplete()));
          return tx;
        },
        close() {},
      };
      if (fresh && r.onupgradeneeded) r.onupgradeneeded();
      r.onsuccess && r.onsuccess();
    });
    return r;
  },
};
const seed = (name, writes) => {
  const store = new Map();
  writes.forEach((w, i) => store.set(i + 1, { ...w, id: i + 1 }));
  dbs.set(name, { stores: new Map([["writes", store]]), next: writes.length + 1 });
};

const fetched = [];
let answer = () => new Response("{}", { status: 200 });
const handlers = {};
const self = {
  location: { origin: ORIGIN },
  addEventListener: (type, fn) => { (handlers[type] = handlers[type] || []).push(fn); },
  clients: { claim: async () => {}, matchAll: async () => [] },
  skipWaiting() {},
  navigator: {},
};
const fetch = async (req, init) => {
  const url = typeof req === "string" ? req : req.url;
  fetched.push({ url, headers: init && init.headers ? init.headers : null, credentials: init && init.credentials });
  return answer(req);
};
vm.runInNewContext(source, { self, caches, fetch, indexedDB, Response, Request, Headers, URL, console, setTimeout, Promise, Map, Set, Object, Array, JSON, RegExp });

async function fire(type, event) {
  const waits = [];
  const ev = Object.assign({ waitUntil: (p) => waits.push(p), respondWith: (p) => waits.push(p) }, event);
  for (const fn of handlers[type] || []) fn(ev);
  return Promise.all(waits);
}
const get = (path, headers) => new Request(ORIGIN + path, { headers });
const out = {};

(async () => {
  if (scenario === "activate") {
    await (await caches.open("wf:/shop:0000000000000001")).put(ORIGIN + "/shop/app.js", new Response("old"));
    await (await caches.open("wf:/shop:runtime-0000000000000001")).put(ORIGIN + "/shop/api/a", new Response("old"));
    await (await caches.open("wf:/blog:0000000000000001")).put(ORIGIN + "/blog/app.js", new Response("blog"));
    await (await caches.open("wf-1111111111111111")).put(ORIGIN + "/app.js", new Response("root"));
    await (await caches.open("wf-2222222222222222")).put(ORIGIN + "/shop/app.js", new Response("shop before"));
    await (await caches.open("wf:/shop:00000000000000aa")).put(ORIGIN + "/shop/app.js", new Response("now"));
    await fire("activate", {});
    out.left = [...stores.keys()].sort();
  } else if (scenario === "keep") {
    const answers = {
      "/shop/api/plain": () => new Response("p"),
      "/shop/api/nostore": () => new Response("n", { headers: { "Cache-Control": "no-store" } }),
      "/shop/api/private": () => new Response("v", { headers: { "Cache-Control": "max-age=60, private" } }),
      "/shop/api/vary": () => new Response("c", { headers: { "Vary": "Accept, Cookie" } }),
      "/shop/api/auth": () => new Response("a"),
      "/shop/api/public": () => new Response("x", { headers: { "Cache-Control": "public, max-age=60" } }),
    };
    answer = (req) => answers[new URL(req.url).pathname]();
    for (const p of Object.keys(answers)) {
      const r = p.endsWith("auth") ? get(p, { Authorization: "Bearer x" }) : get(p);
      await fire("fetch", { request: r });
    }
    const runtime = await caches.open("wf:/shop:runtime-00000000000000aa");
    out.kept = (await runtime.keys()).map((r) => new URL(r.url).pathname).sort();
    // Bounded.
    answer = () => new Response("b");
    for (let i = 0; i < 260; i++) await fire("fetch", { request: get(`/shop/api/n${i}`) });
    const keys = await runtime.keys();
    out.size = keys.length;
    out.newest = new URL(keys[keys.length - 1].url).pathname;
    out.oldestLeft = new URL(keys[0].url).pathname;
    // Offline: another site's store is not this one's answer.
    await (await caches.open("wf:/blog:runtime-1")).put(ORIGIN + "/shop/api/other", new Response("not ours"));
    answer = () => { throw new TypeError("offline"); };
    const [res] = await fire("fetch", { request: get("/shop/api/other") });
    out.otherServed = res.type !== "error";
    const [mine] = await fire("fetch", { request: get("/shop/api/n259") });
    out.ownServed = mine.type !== "error" ? await mine.text() : null;
  } else if (scenario === "sync") {
    seed("wf-offline:/shop", [
      { url: "/shop/api/a", method: "POST", headers: { Authorization: "Bearer old", "X-Trace": "1", "Content-Type": "application/json" }, body: "{}" },
      { url: "/shop/api/b", method: "POST", headers: {}, body: "{}", api: "B" },
      { url: "/shop/api/c", method: "POST", headers: {}, body: "{}" },
    ]);
    seed("wf-offline", [{ url: "/root", method: "POST", headers: {}, body: "{}" }]);
    await fire("sync", { tag: "wf-writes" });
    out.afterOtherTag = fetched.length;
    await fire("sync", { tag: "wf-writes:/shop" });
    out.fetched = fetched;
    out.left = [...dbs.get("wf-offline:/shop").stores.get("writes").values()].map((w) => w.url);
    out.rootLeft = dbs.get("wf-offline").stores.get("writes").size;
  }
  console.log(JSON.stringify(out));
})().catch((e) => { console.error(e); process.exit(1); });
"#;

fn run(base: &str, sync: bool, scenario: &str) -> serde_json::Value {
    let dir = scratch(&format!("sw-{scenario}"));
    std::fs::write(dir.join("sw.js"), worker(base, sync)).unwrap();
    std::fs::write(dir.join("harness.js"), HARNESS).unwrap();
    let out = Command::new("node")
        .arg(dir.join("harness.js"))
        .arg(dir.join("sw.js"))
        .arg(scenario)
        .output()
        .expect("run node");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("the harness prints JSON")
}

#[test]
fn a_site_under_a_base_path_deletes_only_its_own_old_stores() {
    if !has_node() {
        return;
    }
    let out = run("/shop", false, "activate");
    let left: Vec<String> = serde_json::from_value(out["left"].clone()).unwrap();
    assert_eq!(
        left,
        vec![
            "wf-1111111111111111",
            "wf:/blog:0000000000000001",
            "wf:/shop:00000000000000aa",
        ],
        "another site's stores stay; this one's older ones (and its store from before names carried the base) go"
    );
}

#[test]
fn the_runtime_store_keeps_only_what_another_reader_may_see_and_only_so_many() {
    if !has_node() {
        return;
    }
    let out = run("/shop", false, "keep");
    let kept: Vec<String> = serde_json::from_value(out["kept"].clone()).unwrap();
    assert_eq!(kept, vec!["/shop/api/plain", "/shop/api/public"]);
    assert_eq!(out["size"], 200, "bounded");
    assert_eq!(out["newest"], "/shop/api/n259");
    assert_eq!(out["oldestLeft"], "/shop/api/n60");
    assert_eq!(
        out["otherServed"], false,
        "another site's store is not read"
    );
    assert_eq!(out["ownServed"], "b");
}

#[test]
fn kept_writes_are_this_sites_and_go_without_credentials() {
    if !has_node() {
        return;
    }
    let out = run("/shop", true, "sync");
    assert_eq!(
        out["afterOtherTag"], 0,
        "the root site's tag is not this one's"
    );
    let fetched = out["fetched"].as_array().unwrap();
    assert_eq!(fetched.len(), 1, "{fetched:?}");
    assert_eq!(fetched[0]["url"], "/shop/api/a");
    assert_eq!(
        fetched[0]["headers"],
        serde_json::json!({ "X-Trace": "1", "Content-Type": "application/json" }),
        "a credential kept by an older build is not sent"
    );
    let left: Vec<String> = serde_json::from_value(out["left"].clone()).unwrap();
    assert_eq!(
        left,
        vec!["/shop/api/b", "/shop/api/c"],
        "a write whose service adds credentials waits, in order, for the page"
    );
    assert_eq!(out["rootLeft"], 1, "the root site's writes are not sent");
}

#[test]
fn a_site_at_the_root_keeps_the_names_it_had() {
    let sw = worker("", true);
    assert!(sw.contains(r#"const NS = C.base ? `wf:${C.base}:` : "wf-";"#));
    assert!(sw.contains(r#"const WRITES_DB = C.base ? `wf-offline:${C.base}` : "wf-offline";"#));
    assert!(sw.contains(r#""base":"""#));
}

#[test]
fn markdown_never_reads_its_own_markers_from_the_text() {
    let html = markdown::render("`a` and \u{0}0\u{0}");
    assert_eq!(html.matches("<code>").count(), 1, "{html}");
    assert!(!html.contains('\u{0}'));
    let br = markdown::render("a\u{1}\nb");
    assert!(!br.contains("<br>"), "{br}");
}
