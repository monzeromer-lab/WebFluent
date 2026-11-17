  // ─── HTTP ────────────────────────────────────────────
  //
  // One request engine, under `api` declarations, `resource`, and `await
  // fetch(…)` in an action. What it adds over the browser's `fetch` is
  // everything a page actually needs and had to hand-write: a timeout, an
  // abort that follows the scope, retries with backoff, a cache with
  // revalidation, request dedupe, progress, and errors that say what went
  // wrong in a shape `match` can read.
  //
  // An error is an array whose first item names the case — `.offline`,
  // `.timeout`, `.aborted`, `.parse`, `.status(code, body)` — so `match`
  // reads it as an enum, and which also carries `message`, `status`,
  // `body` and `headers` as fields, so `e.message` reads as a record.
  function netError(kind, message, extra) {
    const parts = kind === "status" ? ["status", extra.status, extra.body] : [kind];
    const e = parts;
    e.kind = kind;
    e.message = message;
    e.status = extra && extra.status != null ? extra.status : 0;
    e.body = extra ? extra.body : null;
    e.headers = (extra && extra.headers) || {};
    // A timeout's length, and where a response stopped fitting its type.
    if (extra && extra.after != null) e.after = extra.after;
    if (extra && extra.at != null) { e.at = extra.at; e.detail = extra.detail; }
    // A write kept to send when the connection returns.
    if (extra && extra.queued) e.queued = true;
    return e;
  }

  /// A request's failure as an `HttpError`: `.offline(queued)`, `.timeout(after)`,
  /// `.cancelled`, `.transport(message)`, `.decode(at, message)` or
  /// `.status(code, problem)`, each with `.message` for showing.
  function httpError(error) {
    const kind = error && error.kind;
    const message = (error && error.message) || String(error);
    let parts;
    if (kind === "offline") parts = ["offline", !!error.queued];
    else if (kind === "timeout") parts = ["timeout", error.after != null ? error.after : null];
    else if (kind === "aborted") parts = ["cancelled"];
    else if (kind === "parse") parts = ["decode", error.at || "$", error.detail || message];
    else if (kind === "status") {
      // A problem is the body when the server says it is one (RFC 9457).
      const type = String((error.headers && error.headers["content-type"]) || "");
      const problem = /problem\+json/i.test(type) && error.body && typeof error.body === "object" ? error.body : null;
      parts = ["status", error.status, problem];
    } else parts = ["transport", message];
    Object.defineProperty(parts, "message", { value: message, enumerable: false });
    Object.defineProperty(parts, "httpError", { value: true, enumerable: false });
    return parts;
  }

  /// A response's headers as a plain map, lower-cased, so `r.headers.link`
  /// reads whatever the server sent however it capitalised it.
  function headerMap(headers) {
    const out = {};
    if (!headers || typeof headers.forEach !== "function") return out;
    headers.forEach((value, key) => { out[String(key).toLowerCase()] = value; });
    return out;
  }

  /// What a value should be sent as, and the content type it needs.
  function bodyOf(value) {
    if (value == null) return { body: undefined, type: null };
    if (typeof FormData !== "undefined" && value instanceof FormData) return { body: value, type: null };
    if (typeof Blob !== "undefined" && value instanceof Blob) return { body: value, type: null };
    if (typeof ArrayBuffer !== "undefined" && value instanceof ArrayBuffer) return { body: value, type: null };
    if (typeof ReadableStream !== "undefined" && value instanceof ReadableStream) return { body: value, type: null };
    if (typeof URLSearchParams !== "undefined" && value instanceof URLSearchParams) return { body: value, type: null };
    if (typeof value === "string") return { body: value, type: "text/plain;charset=UTF-8" };
    return { body: JSON.stringify(value), type: "application/json" };
  }

  /// The response, read the way the endpoint said to read it.
  async function readBody(response, as) {
    if (response.status === 204 || response.status === 205) return null;
    switch (as) {
      case "text": return response.text();
      case "blob": return response.blob();
      case "file": return response.blob();
      case "arrayBuffer": return response.arrayBuffer();
      case "none": return null;
      case "response": return response;
      default: {
        const text = await response.text();
        if (!text) return null;
        try {
          return JSON.parse(text);
        } catch (e) {
          throw netError("parse", "The response was not JSON", { body: text });
        }
      }
    }
  }

  /// How long to wait before the next try: exponential, with jitter, and
  /// never longer than the server's own `Retry-After`.
  function backoffDelay(policy, attempt, headers) {
    const after = headers && headers["retry-after"];
    if (after) {
      const seconds = Number(after);
      if (!Number.isNaN(seconds)) return seconds * 1000;
      const at = Date.parse(after);
      if (!Number.isNaN(at)) return Math.max(0, at - Date.now());
    }
    const base = policy.delay || 300;
    const wait = Math.min(base * Math.pow(2, attempt), policy.max || 30000);
    return policy.jitter === false ? wait : wait * (0.5 + Math.random() / 2);
  }

  /// Whether a request may be sent again without changing anything beyond
  /// what sending it once does: the idempotent methods (RFC 9110 §9.2.2),
  /// or one the engine knows never left — the browser said it was offline
  /// before it was sent. A `POST` that timed out may have been taken.
  function resendable(method, error, idempotent) {
    return /^(GET|HEAD|OPTIONS|TRACE|PUT|DELETE)$/.test(method) || !!idempotent || !!(error && error.notSent);
  }

  /// Whether this error is one the policy retries.
  function retriable(policy, error) {
    const on = policy.on || ["network", "timeout", "status5xx", "status429"];
    if (error.kind === "aborted") return false;
    if (on.includes(error.kind)) return true;
    if (error.kind === "status") {
      if (on.includes("status" + error.status)) return true;
      if (on.includes("status5xx") && error.status >= 500) return true;
      if (on.includes("status4xx") && error.status >= 400 && error.status < 500) return true;
    }
    return false;
  }

  // ─── The cache ──
  //
  // One entry per key, shared by every reader: two resources on one URL
  // make one request, and a stale entry is served at once while the
  // revalidation runs behind it.
  //
  // It holds at most `CACHE_MAX` entries: a read moves an entry to the back,
  // and the one read least recently goes first (never one in flight). A
  // request that fails leaves nothing behind.
  //
  // What a reader is allowed to see depends on who it is, so the entry is
  // known by the credentials too: a request that sends another
  // `Authorization` (or another token header) has an entry of its own, and
  // an entry filled through a service's `on request` hook is not served once
  // the hook sends other credentials — after a logout, or as another user.
  // A cookie cannot be read, so a logout should also call `invalidate()` on
  // the endpoints whose answers are one reader's.
  const _httpCache = new Map();
  const CACHE_MAX = 256;
  // The credentials each `on request` hook sent last, by hook.
  const _hookAuth = new WeakMap();

  function cacheEntry(key) {
    let entry = _httpCache.get(key);
    if (entry) {
      _httpCache.delete(key);
      _httpCache.set(key, entry);
      return entry;
    }
    entry = { value: undefined, at: 0, etag: null, modified: null, inflight: null, auth: "" };
    _httpCache.set(key, entry);
    if (_httpCache.size > CACHE_MAX) {
      for (const [k, held] of _httpCache) {
        if (_httpCache.size <= CACHE_MAX) break;
        if (held === entry || held.inflight) continue;
        _httpCache.delete(k);
      }
    }
    return entry;
  }

  /// The cache held to `CACHE_MAX` entries: the one read least recently
  /// goes first, never `keep` or one in flight.
  function trimCache(keep) {
    for (const [k, held] of _httpCache) {
      if (_httpCache.size <= CACHE_MAX) break;
      if (held === keep || held.inflight) continue;
      _httpCache.delete(k);
    }
  }

  /// `Cache-Control`, as a map of its directives.
  function cacheControl(value) {
    const out = {};
    for (const part of String(value || "").split(",")) {
      const [name, arg] = part.split("=");
      const key = name.trim().toLowerCase();
      if (key) out[key] = arg == null ? true : arg.trim().replace(/^"|"$/g, "");
    }
    return out;
  }

  /// What the server allows of an answer it sent (RFC 9111): whether it may
  /// be kept, for how long it is fresh, and how long it may be served stale
  /// while it is asked again. With no caching headers nothing is kept.
  function keepFor(entry, heads) {
    const cc = cacheControl(heads["cache-control"]);
    if (cc["no-store"]) {
      entry.keep = false;
      return;
    }
    let fresh = null;
    if (cc["max-age"] != null && !Number.isNaN(Number(cc["max-age"]))) {
      fresh = Number(cc["max-age"]) * 1000;
    } else if (heads.expires) {
      const expires = Date.parse(heads.expires);
      const date = heads.date ? Date.parse(heads.date) : Date.now();
      fresh = Number.isNaN(expires) ? 0 : Math.max(0, expires - (Number.isNaN(date) ? Date.now() : date));
    }
    if (fresh != null && heads.age && !Number.isNaN(Number(heads.age))) {
      fresh = Math.max(0, fresh - Number(heads.age) * 1000);
    }
    // `no-cache`: kept, and asked again before every use.
    if (cc["no-cache"]) fresh = 0;
    if (fresh == null) {
      entry.keep = false;
      return;
    }
    entry.keep = true;
    entry.fresh = fresh;
    const swr = Number(cc["stale-while-revalidate"]);
    entry.swr = cc["no-cache"] || Number.isNaN(swr) ? 0 : swr * 1000;
  }

  /// The request headers an answer varies by, with the values it was asked
  /// with: another value is another answer. `*`: it is never reused.
  function varyOf(heads, requestHeaders) {
    const vary = heads.vary;
    if (!vary) return null;
    if (vary.trim() === "*") return "*";
    const sent = {};
    for (const [k, v] of Object.entries(requestHeaders || {})) sent[k.toLowerCase()] = String(v);
    return vary.split(",").map((n) => n.trim().toLowerCase()).filter(Boolean)
      .map((name) => [name, sent[name] == null ? null : sent[name]]);
  }

  /// Whether a held answer is one for a request with these headers.
  function varyMatches(entry, requestHeaders) {
    if (!entry.vary) return true;
    if (entry.vary === "*") return false;
    const sent = {};
    for (const [k, v] of Object.entries(requestHeaders || {})) sent[k.toLowerCase()] = String(v);
    return entry.vary.every(([name, value]) => (sent[name] == null ? null : sent[name]) === value);
  }

  /// A short fingerprint of the credentials headers carry: "" for none.
  function authOf(headers) {
    const found = _credentialHeaders(headers);
    if (!found.length) return "";
    found.sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
    let hash = 0x811c9dc5;
    for (const ch of JSON.stringify(found)) {
      hash ^= ch.charCodeAt(0);
      hash = Math.imul(hash, 0x01000193) >>> 0;
    }
    return hash.toString(36);
  }

  /// The headers a call was given, each read now.
  function headersNow(given) {
    const out = {};
    for (const [k, v] of Object.entries(given || {})) {
      const value = typeof v === "function" ? v() : v;
      if (value != null) out[k] = value;
    }
    return out;
  }

  /// Forget what was cached: everything, one key, or every key that starts
  /// with a prefix — which is how one endpoint's cache is dropped.
  ///
  /// What is on a page hears it too: a resource whose request the prefix
  /// names asks again, so `Backend.users.invalidate()` after a write
  /// redraws the list.
  function invalidate(prefix) {
    if (prefix == null) {
      _httpCache.clear();
    } else {
      for (const key of [..._httpCache.keys()]) {
        if (key === prefix || key.startsWith(prefix)) _httpCache.delete(key);
      }
    }
    for (const hear of [..._invalidated]) hear(prefix);
  }
  const _invalidated = new Set();

  /// What the cache holds under keys that start with `prefix`, changed by
  /// `change`; what puts it back is returned.
  function updateCached(prefix, change) {
    const before = [];
    for (const [key, entry] of _httpCache) {
      if (!key.startsWith(prefix) || entry.value === undefined) continue;
      before.push([entry, entry.value]);
      entry.value = change(entry.value);
    }
    return () => { for (const [entry, value] of before) entry.value = value; };
  }
  /// Hear every `invalidate`, until the function it returns is called.
  function onInvalidate(hear) {
    _invalidated.add(hear);
    return () => _invalidated.delete(hear);
  }

  /// One request, with everything the options ask for.
  ///
  /// `options`: `method`, `headers`, `body`, `query`, `as`, `timeout`,
  /// `retry`, `cache`, `signal`, `credentials`, `mode`, `redirect`,
  /// `referrerPolicy`, `progress`, `onRequest`, `onResponse`, `onError`,
  /// and `key` when the caller names the cache entry itself.
  function send(url, options) {
    const opts = options || {};
    const policy = opts.cache;
    const fresh = policy && policy.kind !== "none";
    let key = opts.key != null ? opts.key : cacheKey(url, opts);
    if (fresh) {
      const auth = authOf(headersNow(opts.headers));
      if (auth) key += " \u0001" + auth;
    }
    const entry = fresh ? cacheEntry(key) : null;

    // Filled through a hook that has sent other credentials since: not this
    // reader's to see.
    if (entry && entry.value !== undefined && opts.onRequest && _hookAuth.has(opts.onRequest)
        && _hookAuth.get(opts.onRequest) !== entry.auth) {
      entry.value = undefined;
      entry.etag = null;
      entry.modified = null;
    }
    // A reader of a key already in flight joins it rather than asking again.
    if (entry && entry.inflight) return entry.inflight;
    if (entry && entry.value !== undefined) {
      const age = Date.now() - entry.at;
      const ttl = policy.kind === "forever" ? Infinity : (policy.ttl || 0);
      if (age < ttl) return Promise.resolve(entry.value);
      // Stale: hand back what there is and revalidate behind it.
      if (policy.kind === "swr") {
        const revalidating = run(url, opts, entry).then(
          (value) => value,
          () => entry.value,
        );
        entry.inflight = revalidating.finally(() => { entry.inflight = null; });
        return Promise.resolve(entry.value);
      }
    }
    // A read the contract says nothing of caching: what the server said of
    // it (RFC 9111, a private cache). An answer it allowed to be kept is
    // served while fresh, served stale and revalidated behind within
    // `stale-while-revalidate`, and asked again with its `ETag` after.
    // Two readers of one address make one request either way; each keeps
    // its own abort, and the request stops when the last reader has gone.
    if (!entry && !opts.progress && /^(GET|HEAD)$/.test((opts.method || "GET").toUpperCase())) {
      const now = headersNow(opts.headers);
      const httpKey = key + " \u0002" + authOf(now) + " " + (opts.as || "");
      const held = policy && policy.kind === "none" ? null : _httpCache.get(httpKey);
      if (held && held.value !== undefined && varyMatches(held, now)) {
        _httpCache.delete(httpKey);
        _httpCache.set(httpKey, held);
        const age = Date.now() - held.at;
        if (age < held.fresh) return Promise.resolve(held.value);
        if (age < held.fresh + held.swr) {
          if (!held.inflight) {
            held.inflight = run(url, { ...opts, signal: undefined }, held)
              .catch(() => held.value)
              .finally(() => { held.inflight = null; });
          }
          return Promise.resolve(held.value);
        }
      }
      const candidate = policy && policy.kind === "none"
        ? null
        : held || { value: undefined, at: 0, etag: null, modified: null, inflight: null, auth: "", http: true, key: httpKey };
      return joinRead(key + " \u0001" + (opts.as || "") + " " + JSON.stringify(now), url, opts, candidate);
    }
    const running = run(url, opts, entry);
    if (entry) {
      // `finally` makes a second promise, which rejects when the request
      // does. A joiner reads the failure through it; with no joiner,
      // nothing would, and the failure — which the caller handles on
      // `running` — would surface again as an unhandled rejection.
      const joined = running.finally(() => {
        entry.inflight = null;
        // Nothing arrived: the entry holds nothing, and goes.
        if (entry.value === undefined && _httpCache.get(key) === entry) _httpCache.delete(key);
      });
      joined.catch(() => {});
      entry.inflight = joined;
    }
    return running;
  }

  const _readsInFlight = new Map();
  function joinRead(key, url, opts, entry) {
    let shared = _readsInFlight.get(key);
    if (!shared) {
      const controller = typeof AbortController !== "undefined" ? new AbortController() : null;
      shared = { readers: 0, controller };
      shared.promise = run(url, { ...opts, signal: controller ? controller.signal : undefined }, entry || null);
      shared.promise.catch(() => {});
      const gone = () => { if (_readsInFlight.get(key) === shared) _readsInFlight.delete(key); };
      shared.promise.then(gone, gone);
      _readsInFlight.set(key, shared);
    }
    shared.readers += 1;
    const mine = opts.signal;
    if (!mine) return shared.promise;
    return new Promise((resolve, reject) => {
      let left = false;
      const leave = () => {
        if (left) return;
        left = true;
        shared.readers -= 1;
        if (shared.readers === 0) {
          if (_readsInFlight.get(key) === shared) _readsInFlight.delete(key);
          if (shared.controller) shared.controller.abort();
        }
        reject(netError("aborted", "The request was cancelled"));
      };
      if (mine.aborted) { leave(); return; }
      mine.addEventListener("abort", leave, { once: true });
      shared.promise.then(
        (v) => { left = true; mine.removeEventListener("abort", leave); resolve(v); },
        (e) => { left = true; mine.removeEventListener("abort", leave); reject(e); },
      );
    });
  }

  /// What a request is known by: the method, the URL and the body.
  function cacheKey(url, opts) {
    const method = (opts.method || "GET").toUpperCase();
    const body = opts.body == null ? "" : typeof opts.body === "string" ? opts.body : JSON.stringify(opts.body);
    return `${method} ${url} ${body}`;
  }
  /// How many entries the cache holds (for tests and the studio).
  send.cacheSize = () => _httpCache.size;

  async function run(url, opts, entry) {
    const policy = opts.retry || { times: 0 };
    const times = policy.times || 0;
    let attempt = 0;
    for (;;) {
      try {
        const value = await once(url, opts, entry);
        // What the program declared it would get, held to what came — in
        // every build: a value typed `User` is a `User`, and a server that
        // does not keep its contract fails here, as `.decode(at, message)`.
        if (opts.shape) {
          const fault = shapeFault(opts.withHeaders && value ? value.value : value, opts.shape, "$");
          if (fault) throw netError("parse", "The response of " + url + " is not the type the program declares: " + fault.at + " " + fault.message, { body: value, at: fault.at, detail: fault.message });
        }
        if (entry && entry.http) {
          // What the server said may be kept, and nothing else.
          if (entry.keep) {
            entry.value = value;
            entry.at = Date.now();
            _httpCache.delete(entry.key);
            _httpCache.set(entry.key, entry);
            trimCache(entry);
          } else {
            _httpCache.delete(entry.key);
          }
        } else if (entry) {
          entry.value = value;
          entry.at = Date.now();
        }
        return value;
      } catch (error) {
        const wrapped = error && error.kind ? error : netError("network", String(error && error.message || error));
        // A failure's body, held to the type its status declares.
        if (wrapped.kind === "status" && opts.errorShapes) {
          // `on 4XX`: a class of statuses, keyed by its first digit.
          const shape = opts.errorShapes[wrapped.status] || opts.errorShapes[Math.floor(wrapped.status / 100)];
          const wrong = shape && shapeProblem(wrapped.body, shape, "$");
          if (wrong) throw netError("parse", "The " + wrapped.status + " body of " + url + " is not the type the program declares: " + wrong, { body: wrapped.body });
        }
        const method = (opts.method || "GET").toUpperCase();
        if (attempt < times && retriable(policy, wrapped) && resendable(method, wrapped, opts.idempotencyKey)) {
          const wait = backoffDelay(policy, attempt, wrapped.headers);
          attempt += 1;
          await new Promise((done) => setTimeout(done, wait));
          continue;
        }
        if (opts.onError) {
          // A hook may answer the error — by refreshing a token, say — and
          // ask for one more try.
          const answer = await opts.onError(wrapped);
          if (answer === "retry" && attempt <= times + 1) {
            attempt += 1;
            continue;
          }
        }
        // A write made with no network, where the config asks for `sync`: it
        // is kept and sent later. Only one that is safe to send twice: a
        // `PUT` or a `DELETE`, or one its contract marks `idempotent`, with
        // its key. A `POST` whose answer was lost may have been taken, and
        // sent again would be a second order. The call says what happened:
        // it failed `.offline(queued: true)` — no answer came, and the
        // write waits to be sent.
        if (wrapped.kind === "offline" && offlineWrites && wrapped.request
            && (/^(PUT|DELETE)$/.test(wrapped.request.method) || opts.idempotencyKey)) {
          const kept = offlineWrites(wrapped.request);
          let held = false;
          if (kept) {
            try { await kept; held = true; } catch (e) { held = false; }
          }
          if (held) throw netError("offline", "Kept, to send when the connection returns", { queued: true });
        }
        throw wrapped;
      }
    }
  }

  /// One attempt: the request as the browser makes it, through the
  /// service's middleware when it has any.
  async function once(url, opts, entry) {
    const controller = typeof AbortController !== "undefined" ? new AbortController() : null;
    // The caller's own signal aborts this one too, so a scope leaving takes
    // the request with it.
    if (opts.signal && controller) {
      if (opts.signal.aborted) controller.abort();
      else opts.signal.addEventListener("abort", () => controller.abort(), { once: true });
    }

    const { body, type } = bodyOf(typeof opts.body === "function" ? opts.body() : opts.body);
    const headers = { ...(opts.headers || {}) };
    for (const [k, v] of Object.entries(headers)) {
      if (typeof v === "function") headers[k] = v();
      if (headers[k] == null) delete headers[k];
    }
    if (type && !Object.keys(headers).some((k) => k.toLowerCase() === "content-type")) {
      headers["Content-Type"] = type;
    }
    // What is already held, offered back to the server.
    if (entry && entry.value !== undefined) {
      if (entry.etag) headers["If-None-Match"] = entry.etag;
      else if (entry.modified) headers["If-Modified-Since"] = entry.modified;
    }
    if (opts.idempotencyKey) headers["Idempotency-Key"] = opts.idempotencyKey;

    let request = {
      url,
      method: (opts.method || "GET").toUpperCase(),
      headers,
      body,
      credentials: opts.credentials,
      mode: opts.mode,
      redirect: opts.redirect,
      referrerPolicy: opts.referrerPolicy,
      signal: controller ? controller.signal : undefined,
    };
    // What the call itself sent, before a hook added to it: what a write
    // kept offline may store.
    const given = { ...headers };
    if (opts.onRequest) request = (await opts.onRequest(request)) || request;

    // Held for other credentials than these: not offered back, not served.
    const auth = authOf(request.headers);
    if (opts.onRequest) _hookAuth.set(opts.onRequest, auth);
    if (entry && entry.value !== undefined && entry.auth !== auth) {
      entry.value = undefined;
      entry.etag = null;
      entry.modified = null;
      if (request.headers) {
        delete request.headers["If-None-Match"];
        delete request.headers["If-Modified-Since"];
      }
    }

    const transmit = (request) => sendOnce(request, opts, entry, controller, given, auth);
    const steps = opts.middleware;
    if (!steps || !steps.length) return transmit(request);
    // The chain: each step around the next, the last the network. What a
    // step is handed is the request; what it returns is a result.
    const network = (req) => transmit({ ...request, ...req }).then(
      (value) => ["ok", value],
      (error) => ["err", toHttpError(error)],
    );
    const chain = steps.reduceRight(
      (next, step) => (req) => Promise.resolve()
        .then(() => step(req, next))
        .catch((e) => ["err", toHttpError(e)]),
      network,
    );
    const answered = await chain({
      url: request.url,
      method: request.method,
      headers: { ...request.headers },
      body: request.body,
    });
    if (answered === "ok") return undefined;
    if (Array.isArray(answered) && answered[0] === "ok") return answered[1];
    throw fromHttpError(Array.isArray(answered) ? answered[1] : answered);
  }

  /// A failure as the chain hands it on: an `HttpError`, carrying what the
  /// engine knew of it.
  function toHttpError(error) {
    if (Array.isArray(error) && error.httpError) return error;
    const failure = error && error.kind ? error : netError("transport", String((error && error.message) || error));
    const http = httpError(failure);
    Object.defineProperty(http, "cause", { value: failure, enumerable: false });
    return http;
  }

  /// An `HttpError` a middleware returned, as the engine throws it: the
  /// failure it came from, or one made of its case.
  function fromHttpError(http) {
    if (http && http.cause) return http.cause;
    const kind = Array.isArray(http) ? http[0] : String(http);
    const message = (http && http.message) || kind;
    switch (kind) {
      case "offline": return netError("offline", message);
      case "timeout": return netError("timeout", message, { after: http[1] });
      case "cancelled": return netError("aborted", message);
      case "decode": return netError("parse", message, { at: http[1], detail: http[2] });
      case "status": return netError("status", "HTTP " + http[1], { status: http[1], body: http[2] });
      default: return netError("transport", message);
    }
  }

  /// The request sent once — by a step of the chain, or straight — and its
  /// answer read: its value, or the failure thrown.
  async function sendOnce(request, opts, entry, controller, given, auth) {
    // A header a step wrote as a function — `bearer(token)` — is read now,
    // so a token refreshed between two sends is the one the second carries.
    const headers = {};
    for (const [k, v] of Object.entries(request.headers || {})) {
      const value = typeof v === "function" ? v() : v;
      if (value != null) headers[k] = String(value);
    }
    request = { ...request, headers };
    let timedOut = false;
    let timer = null;
    if (opts.timeout && controller) {
      timer = setTimeout(() => { timedOut = true; controller.abort(); }, opts.timeout);
    }

    // After the request is made up, so an offline write can be kept whole.
    // Only a request that failed for want of a network may be kept: the
    // browser says it is offline, or a request to this site's own server
    // could not be made. One refused by another origin — a CORS refusal, a
    // closed port — fails as it always did.
    const offlineError = (message, unreachable) => {
      if (timer) clearTimeout(timer);
      const error = netError("offline", message);
      if (unreachable) {
        error.request = {
          url: request.url,
          method: request.method,
          given,
          hooked: !!opts.onRequest,
          api: opts.api || null,
          credentials: request.credentials,
          body: request.body,
          // The reads a success makes stale, by their cache keys: applied
          // when the write is finally sent.
          stale: opts.stale ? opts.stale() : [],
        };
      }
      return error;
    };
    if (typeof navigator !== "undefined" && navigator.onLine === false) {
      const never = offlineError("You are offline", true);
      // Nothing was sent: safe to send again, whatever the method.
      never.notSent = true;
      throw never;
    }

    let response;
    try {
      // An upload only reports its progress through XHR, so a request that
      // is watched goes that way and every other through `fetch`.
      response = opts.progress && request.body != null
        ? await xhrSend(request, opts.progress, controller)
        : await fetch(request.url, {
            method: request.method,
            headers: request.headers,
            body: request.body,
            credentials: request.credentials,
            mode: request.mode,
            redirect: request.redirect,
            referrerPolicy: request.referrerPolicy,
            signal: request.signal,
          });
    } catch (e) {
      if (timer) clearTimeout(timer);
      if (timedOut) throw netError("timeout", "The server took too long", { after: opts.timeout });
      if (e && (e.name === "AbortError" || e.kind === "aborted")) {
        throw netError("aborted", "The request was cancelled");
      }
      const browserOffline = typeof navigator !== "undefined" && navigator.onLine === false;
      throw offlineError((e && e.message) || "The request could not be made", browserOffline || sameOrigin(request.url));
    }
    if (timer) clearTimeout(timer);
    if (entry) entry.auth = auth;

    const heads = headerMap(response.headers);
    if (opts.onResponse) await opts.onResponse({ status: response.status, headers: heads, url: request.url });

    // Nothing changed: what is held is still current, for as long as the
    // answer now says.
    if (response.status === 304 && entry && entry.value !== undefined) {
      entry.at = Date.now();
      if (!opts.cache) keepFor(entry, heads);
      return entry.value;
    }
    if (entry) {
      entry.etag = heads.etag || null;
      entry.modified = heads["last-modified"] || null;
      entry.vary = varyOf(heads, request.headers);
      if (!opts.cache) keepFor(entry, heads);
    }

    if (!response.ok) {
      let decoded = null;
      try {
        decoded = await readBody(response, opts.errorAs || "json");
      } catch (e) {
        decoded = null;
      }
      throw netError("status", `HTTP ${response.status}`, {
        status: response.status,
        body: decoded,
        headers: heads,
      });
    }

    const value = await readBody(response, opts.as);
    // The headers come back with the value when the caller asked for them:
    // pagination lives in `Link`, and nowhere else.
    if (opts.withHeaders) return { value, headers: heads, status: response.status, url: request.url };
    return value;
  }

  // ─── Middleware ──
  //
  // What every request of a service passes through, in the order its
  // `middleware [ … ]` names them: each a function of the request and the
  // rest of the chain, returning what the chain returns — `["ok", value]`
  // or `["err", httpError]`. A step reads what it adds when the request
  // reaches it, so one that sends the request again — `refreshOn401`,
  // written before `bearer` — sends it through `bearer` again, with the
  // token it just refreshed.
  const mw = {
    /// `bearer(Session.token)`: `Authorization: Bearer …`, when there is one.
    bearer: (read) => (req, next) => {
      const token = read();
      return next(token ? { ...req, headers: { ...req.headers, Authorization: "Bearer " + token } } : req);
    },
    /// `header("X-Client", "web")`.
    header: (name, read) => (req, next) => {
      const value = read();
      return next(value == null ? req : { ...req, headers: { ...req.headers, [name]: String(value) } });
    },
    /// A fresh `X-Request-Id`, the same on every try of one request.
    requestId: (req, next) => {
      const id = (typeof crypto !== "undefined" && crypto.randomUUID) ? crypto.randomUUID() : String(Date.now()) + Math.random().toString(36).slice(2);
      return next({ ...req, headers: { ...req.headers, "X-Request-Id": id } });
    },
    /// Each request and how it ended, in the console.
    log: async (req, next) => {
      const answer = await next(req);
      console.log(req.method, req.url, Array.isArray(answer) ? answer[0] : answer);
      return answer;
    },
    /// `refreshOn401(Session.refresh)`: when the server says the
    /// credentials expired, refresh them — once, for every request that
    /// heard it at the same time, since a running action is joined — and
    /// send the request again.
    refreshOn401: (refresh) => async (req, next) => {
      const first = await next(req);
      if (Array.isArray(first) && first[0] === "err" && Array.isArray(first[1])
          && first[1][0] === "status" && first[1][1] === 401) {
        await refresh();
        return next(req);
      }
      return first;
    },
  };

  /// Whether a URL is this site's own.
  function sameOrigin(url) {
    try {
      const here = typeof window !== "undefined" && window.location ? window.location : null;
      if (!here || !here.origin) return !/^[a-z][a-z0-9+.-]*:|^\/\//i.test(String(url));
      return new URL(url, here.href).origin === here.origin;
    } catch (e) {
      return false;
    }
  }

  /// A request through `XMLHttpRequest`, for the one thing `fetch` cannot
  /// do: tell you how far an upload has got.
  function xhrSend(request, progress, controller) {
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();
      xhr.open(request.method, request.url, true);
      if (request.credentials === "include") xhr.withCredentials = true;
      for (const [k, v] of Object.entries(request.headers || {})) xhr.setRequestHeader(k, v);
      xhr.responseType = "text";
      if (xhr.upload && progress) {
        xhr.upload.onprogress = (e) => {
          if (e.lengthComputable) progress.set(e.loaded / e.total);
        };
      }
      xhr.onload = () => {
        if (progress) progress.set(1);
        const raw = xhr.getAllResponseHeaders();
        const heads = new Map();
        for (const line of raw.trim().split(/[\r\n]+/)) {
          const at = line.indexOf(":");
          if (at > 0) heads.set(line.slice(0, at).trim().toLowerCase(), line.slice(at + 1).trim());
        }
        resolve({
          ok: xhr.status >= 200 && xhr.status < 300,
          status: xhr.status,
          headers: { forEach: (fn) => heads.forEach((v, k) => fn(v, k)) },
          text: () => Promise.resolve(xhr.responseText),
          json: () => Promise.resolve(JSON.parse(xhr.responseText)),
          blob: () => Promise.resolve(new Blob([xhr.response])),
          arrayBuffer: () => Promise.resolve(new TextEncoder().encode(xhr.responseText).buffer),
        });
      };
      xhr.onerror = () => reject(netError("offline", "The request could not be made"));
      xhr.onabort = () => reject(netError("aborted", "The request was cancelled"));
      if (controller) {
        controller.signal.addEventListener("abort", () => xhr.abort(), { once: true });
      }
      xhr.send(request.body);
    });
  }

  /// A `streamed` endpoint's answer, a line at a time as it arrives: each
  /// line JSON (or its text), held to the type of the list's item.
  ///
  /// The request goes as any other — middleware, timeout, retries — and is
  /// read as it comes rather than whole; a consumer that stops early
  /// cancels the rest.
  ///
  /// Every module shares one closure, so this is not called `lines`, the
  /// name of the helper that splits a text: wherever both were built in,
  /// the later declaration won.
  async function* streamResponse(url, opts, itemShape) {
    const response = await run(url, { ...opts, as: "response", shape: null, cache: { kind: "none" } }, null);
    let index = 0;
    const take = (text) => {
      let value = text;
      try { value = JSON.parse(text); } catch (e) { /* a line of text */ }
      if (itemShape) {
        const fault = shapeFault(value, itemShape, "$[" + index + "]");
        if (fault) throw netError("parse", "Line " + (index + 1) + " of " + url + " is not the type the program declares: " + fault.at + " " + fault.message, { body: value, at: fault.at, detail: fault.message });
      }
      index += 1;
      return value;
    };
    if (!response || !response.body || typeof response.body.getReader !== "function") {
      const text = response && typeof response.text === "function" ? await response.text() : "";
      for (const line of text.split("\n")) if (line.trim()) yield take(line.trim());
      return;
    }
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let held = "";
    let finished = false;
    try {
      for (;;) {
        let chunk;
        try {
          chunk = await reader.read();
        } catch (e) {
          if (e && (e.name === "AbortError" || e.kind === "aborted")) throw netError("aborted", "The request was cancelled");
          throw netError("transport", (e && e.message) || "The answer stopped arriving");
        }
        if (chunk.done) break;
        held += decoder.decode(chunk.value, { stream: true });
        const parts = held.split("\n");
        held = parts.pop();
        for (const line of parts) if (line.trim()) yield take(line.trim());
      }
      held += decoder.decode();
      if (held.trim()) yield take(held.trim());
      finished = true;
    } finally {
      if (!finished) reader.cancel().catch(() => {});
    }
  }

  /// A short-lived request that must outlive the page: analytics on
  /// unload, which `fetch` cannot promise to deliver.
  function beacon(url, data) {
    const body = typeof data === "string" ? data : JSON.stringify(data);
    if (typeof navigator !== "undefined" && navigator.sendBeacon) {
      return navigator.sendBeacon(url, new Blob([body], { type: "application/json" }));
    }
    try {
      fetch(url, { method: "POST", body, keepalive: true, headers: { "Content-Type": "application/json" } });
      return true;
    } catch (e) {
      return false;
    }
  }
