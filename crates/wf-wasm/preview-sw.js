// Serves a build the page compiled, as a host would serve it.
//
// `wf.js`'s `preview()` registers this worker and posts it the files a
// build wrote; the worker keeps them in Cache Storage (a worker that is
// stopped while idle forgets its memory, a cache it does not) and answers
// every request under its scope from them: `app.js`, a page's chunk, a
// picture. A navigation to a route is answered with the page a static build
// wrote for it (`about/index.html`, `about.html`), or the app's shell.

const CACHE = "wf-preview";
const scope = new URL(self.registration.scope);

const TYPES = {
  html: "text/html; charset=utf-8", js: "text/javascript; charset=utf-8",
  css: "text/css; charset=utf-8", json: "application/json", xml: "application/xml",
  txt: "text/plain; charset=utf-8", svg: "image/svg+xml", png: "image/png",
  jpg: "image/jpeg", jpeg: "image/jpeg", gif: "image/gif", webp: "image/webp",
  ico: "image/x-icon", pdf: "application/pdf", woff2: "font/woff2", woff: "font/woff",
  ttf: "font/ttf", otf: "font/otf", mp4: "video/mp4", webm: "video/webm",
  mp3: "audio/mpeg", vtt: "text/vtt",
};

const typeOf = (path) => TYPES[path.split(".").pop().toLowerCase()] ?? "application/octet-stream";

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (event) => event.waitUntil(self.clients.claim()));

self.addEventListener("message", (event) => {
  if (event.data?.type !== "wf-publish") return;
  const reply = event.ports[0];
  event.waitUntil((async () => {
    await caches.delete(CACHE);
    const cache = await caches.open(CACHE);
    await Promise.all(Object.entries(event.data.files).map(([path, body]) =>
      cache.put(new URL(path, scope).href, new Response(body, {
        headers: { "content-type": typeOf(path), "cache-control": "no-store" },
      }))));
    reply?.postMessage({ ok: true });
  })());
});

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (url.origin !== scope.origin || !url.pathname.startsWith(scope.pathname)) return;
  event.respondWith((async () => {
    const cache = await caches.open(CACHE);
    const path = url.pathname.slice(scope.pathname.length).replace(/\/$/, "");
    const tries = [path];
    if (event.request.mode === "navigate" || !/\.[a-z0-9]+$/i.test(path)) {
      tries.push(path ? `${path}/index.html` : "index.html", `${path}.html`);
    }
    for (const at of tries) {
      const hit = at !== "" && await cache.match(new URL(at, scope).href);
      if (hit) return hit;
    }
    // A route no page was written for: the single-page app's shell, whose
    // router draws it (or its 404).
    if (event.request.mode === "navigate") {
      const shell = await cache.match(new URL("index.html", scope).href);
      if (shell) return shell;
    }
    return new Response(`${path} is not in this build`, { status: 404 });
  })());
});
