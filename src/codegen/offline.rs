//! The service worker a build writes when the config names `offline`.
//!
//! What it stores is decided here, from what the build wrote; how it answers
//! is the template below, with those decisions filled in as JSON. The page's
//! side — registering it, the update flow, writes kept for later — is the
//! runtime's `offline` module.

use serde_json::json;

/// What goes into one `sw.js`.
pub struct Worker<'a> {
    /// Changes whenever anything the build wrote changes.
    pub version: &'a str,
    /// `build.base_path`: `""` or `"/my-site"`.
    pub base: &'a str,
    /// Every file stored at install, as site paths under `base`.
    pub precache: Vec<String>,
    /// A stored route and the file that answers it: `"/docs"` →
    /// `"/docs/index.html"`, both under `base`.
    pub routes: Vec<(String, String)>,
    /// A single-page build's shell, which renders any route it is given.
    pub shell: Option<String>,
    /// The route shown for one that is not stored, when the network is
    /// gone, under `base`: the navigation is sent there, so the page's router
    /// renders that page rather than the one it could not fetch.
    pub fallback: Option<String>,
    /// `(glob, strategy)` for paths the build did not write.
    pub policies: Vec<(String, String)>,
    /// Whether writes kept by the page are sent from here, by Background Sync.
    pub sync: bool,
}

/// A glob over site paths as a regular expression: `*` is anything,
/// `/docs/*` everything under `/docs/`.
pub fn glob_regex(glob: &str) -> String {
    let mut out = String::from("^");
    for c in glob.chars() {
        match c {
            '*' => out.push_str(".*"),
            c if "\\.+?()[]{}|^$".contains(c) => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push('$');
    out
}

/// Whether a route falls under one of the globs `offline.precache` names.
pub fn route_matches(globs: &[String], route: &str) -> bool {
    globs.iter().any(|g| {
        regex::Regex::new(&glob_regex(g))
            .map(|r| r.is_match(route))
            .unwrap_or(false)
    })
}

/// A version for everything the build wrote: FNV-1a over each file's path
/// and bytes, in path order, so it moves when any byte does and not
/// otherwise.
pub fn version_of(files: &[(String, Vec<u8>)]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for b in bytes {
            hash ^= u64::from(*b);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    let mut sorted: Vec<&(String, Vec<u8>)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (path, bytes) in sorted {
        eat(path.as_bytes());
        eat(&[0]);
        eat(bytes);
        eat(&[0]);
    }
    format!("{hash:016x}")
}

pub fn service_worker(w: &Worker) -> String {
    let config = json!({
        "version": w.version,
        "base": w.base,
        "precache": w.precache,
        "routes": w.routes.iter().map(|(r, f)| json!([r, f])).collect::<Vec<_>>(),
        "shell": w.shell,
        "fallback": w.fallback,
        "policies": w.policies
            .iter()
            .map(|(g, s)| json!([glob_regex(g), s]))
            .collect::<Vec<_>>(),
        "sync": w.sync,
    });
    TEMPLATE.replace("__CONFIG__", &config.to_string())
}

const TEMPLATE: &str = r#"// Written by `wf build` because the config names `offline`. Edit the
// config, not this file: the next build writes it again.
"use strict";
const C = __CONFIG__;
const STORE = `wf-${C.version}`;
const RUNTIME = `wf-runtime-${C.version}`;
const PRECACHED = new Set(C.precache);
const ROUTES = new Map(C.routes);
const POLICIES = C.policies.map(([pattern, strategy]) => [new RegExp(pattern), strategy]);

// The path a route is known by: no base, no trailing slash, no index.html.
function routeOf(pathname) {
  let p = pathname;
  if (C.base && p.startsWith(C.base)) p = p.slice(C.base.length) || "/";
  if (p.endsWith("/index.html")) p = p.slice(0, -"index.html".length);
  if (p.length > 1 && p.endsWith("/")) p = p.slice(0, -1);
  return p || "/";
}

self.addEventListener("install", (event) => {
  // Fetched past the HTTP cache, so a new version stores new files.
  event.waitUntil(
    caches.open(STORE).then((cache) =>
      cache.addAll(C.precache.map((url) => new Request(url, { cache: "reload" })))),
  );
});

self.addEventListener("activate", (event) => {
  // The stores of every version before this one go.
  event.waitUntil(
    caches.keys()
      .then((keys) => Promise.all(
        keys.filter((k) => k.startsWith("wf-") && k !== STORE && k !== RUNTIME).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

// A new version waits until the page says to take over: `update.apply()`.
self.addEventListener("message", (event) => {
  if (event.data && event.data.type === "wf:activate") self.skipWaiting();
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  // A write goes to the network; with `sync` the page keeps one that fails.
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  if (request.mode === "navigate") {
    event.respondWith(navigate(request, url.pathname));
    return;
  }
  if (PRECACHED.has(url.pathname)) {
    event.respondWith(stored(request));
    return;
  }
  const route = routeOf(url.pathname);
  const policy = POLICIES.find(([pattern]) => pattern.test(route));
  if (policy) event.respondWith(STRATEGIES[policy[1]](request));
});

// The network first, for a page that may have changed; what is stored when
// it is gone; the fallback when not even that is. The fallback is a
// redirect, not its HTML under this address: the page's router renders by
// address, and would try to draw the route it has no chunk for.
async function navigate(request, pathname) {
  try {
    return await fetch(request);
  } catch (e) {
    const cache = await caches.open(STORE);
    const route = routeOf(pathname);
    const file = ROUTES.get(route);
    if (file) {
      const hit = await cache.match(file);
      if (hit) return hit;
    }
    if (C.fallback && route !== routeOf(C.fallback)) {
      return Response.redirect(new URL(C.fallback, self.location.origin).href, 302);
    }
    if (C.shell) {
      const hit = await cache.match(C.shell);
      if (hit) return hit;
    }
    return Response.error();
  }
}

async function stored(request) {
  const cache = await caches.open(STORE);
  return (await cache.match(request, { ignoreSearch: true })) || fetch(request);
}

async function keep(request, response) {
  if (response && response.ok) {
    const cache = await caches.open(RUNTIME);
    await cache.put(request, response.clone());
  }
  return response;
}

const STRATEGIES = {
  "network-first": async (request) => {
    try {
      return await keep(request, await fetch(request));
    } catch (e) {
      return (await caches.match(request)) || Response.error();
    }
  },
  "cache-first": async (request) =>
    (await caches.match(request)) || keep(request, await fetch(request)),
  "stale-while-revalidate": async (request) => {
    const cached = await caches.match(request);
    const fresh = fetch(request).then((r) => keep(request, r)).catch(() => cached);
    return cached || fresh;
  },
  "network-only": (request) => fetch(request),
};

// ─── Writes kept while offline (`sync`) ──
//
// The page keeps them in IndexedDB; with Background Sync the browser wakes
// this worker when the connection returns, page open or not, and they are
// sent oldest first. A server that answers has the write; one that is down,
// busy or asking to wait keeps it, and the browser tries again later.
if (C.sync) {
  const openWrites = () => new Promise((resolve, reject) => {
    const open = indexedDB.open("wf-offline", 1);
    open.onupgradeneeded = () => open.result.createObjectStore("writes", { keyPath: "id", autoIncrement: true });
    open.onerror = () => reject(open.error);
    open.onsuccess = () => resolve(open.result);
  });
  const inStore = (db, mode, work) => new Promise((resolve, reject) => {
    const tx = db.transaction("writes", mode);
    let out;
    tx.oncomplete = () => resolve(out);
    tx.onerror = () => reject(tx.error);
    work(tx.objectStore("writes"), (v) => { out = v; });
  });
  // Under the lock the page takes too, so a write is never sent by both.
  const sendWrites = () => (self.navigator.locks
    ? self.navigator.locks.request("wf-writes", sendAll)
    : sendAll());
  const sendAll = async () => {
    const db = await openWrites();
    try {
      const writes = await inStore(db, "readonly", (s, done) => {
        const r = s.getAll();
        r.onsuccess = () => done(r.result || []);
      });
      for (const w of writes) {
        const response = await fetch(w.url, { method: w.method, headers: w.headers, body: w.body });
        if (response.status >= 500 || response.status === 408 || response.status === 429) {
          throw new Error(`the server answered ${response.status}; trying again later`);
        }
        await inStore(db, "readwrite", (s) => { s.delete(w.id); });
      }
    } finally {
      db.close();
      const pages = await self.clients.matchAll();
      for (const page of pages) page.postMessage({ type: "wf:writes" });
    }
  };
  self.addEventListener("sync", (event) => {
    if (event.tag === "wf-writes") event.waitUntil(sendWrites());
  });
}
"#;
