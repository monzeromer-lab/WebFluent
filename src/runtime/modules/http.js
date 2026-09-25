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
    return e;
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
  const _httpCache = new Map();

  function cacheEntry(key) {
    let entry = _httpCache.get(key);
    if (!entry) {
      entry = { value: undefined, at: 0, etag: null, modified: null, inflight: null };
      _httpCache.set(key, entry);
    }
    return entry;
  }

  /// Forget what was cached: everything, one key, or every key that starts
  /// with a prefix — which is how one endpoint's cache is dropped.
  function invalidate(prefix) {
    if (prefix == null) {
      _httpCache.clear();
      return;
    }
    for (const key of [..._httpCache.keys()]) {
      if (key === prefix || key.startsWith(prefix)) _httpCache.delete(key);
    }
  }

  /// One request, with everything the options ask for.
  ///
  /// `options`: `method`, `headers`, `body`, `query`, `as`, `timeout`,
  /// `retry`, `cache`, `signal`, `credentials`, `mode`, `redirect`,
  /// `referrerPolicy`, `progress`, `onRequest`, `onResponse`, `onError`,
  /// and `key` when the caller names the cache entry itself.
  function send(url, options) {
    const opts = options || {};
    const key = opts.key != null ? opts.key : cacheKey(url, opts);
    const policy = opts.cache;
    const fresh = policy && policy.kind !== "none";
    const entry = fresh ? cacheEntry(key) : null;

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
    const running = run(url, opts, entry);
    if (entry) {
      entry.inflight = running.finally(() => { entry.inflight = null; });
    }
    return running;
  }

  /// What a request is known by: the method, the URL and the body.
  function cacheKey(url, opts) {
    const method = (opts.method || "GET").toUpperCase();
    const body = opts.body == null ? "" : typeof opts.body === "string" ? opts.body : JSON.stringify(opts.body);
    return `${method} ${url} ${body}`;
  }

  async function run(url, opts, entry) {
    const policy = opts.retry || { times: 0 };
    const times = policy.times || 0;
    let attempt = 0;
    for (;;) {
      try {
        const value = await once(url, opts, entry);
        if (entry) {
          entry.value = value;
          entry.at = Date.now();
        }
        return value;
      } catch (error) {
        const wrapped = error && error.kind ? error : netError("network", String(error && error.message || error));
        if (attempt < times && retriable(policy, wrapped)) {
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
        throw wrapped;
      }
    }
  }

  /// One attempt: the request as the browser makes it.
  async function once(url, opts, entry) {
    if (typeof navigator !== "undefined" && navigator.onLine === false) {
      throw netError("offline", "You are offline");
    }
    const controller = typeof AbortController !== "undefined" ? new AbortController() : null;
    let timedOut = false;
    let timer = null;
    if (opts.timeout && controller) {
      timer = setTimeout(() => { timedOut = true; controller.abort(); }, opts.timeout);
    }
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
    if (opts.onRequest) request = (await opts.onRequest(request)) || request;

    let response;
    try {
      // An upload only reports its progress through XHR, so a request that
      // is watched goes that way and every other through `fetch`.
      response = opts.progress && body != null
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
      if (timedOut) throw netError("timeout", "The server took too long");
      if (e && (e.name === "AbortError" || e.kind === "aborted")) {
        throw netError("aborted", "The request was cancelled");
      }
      throw netError("offline", (e && e.message) || "The request could not be made");
    }
    if (timer) clearTimeout(timer);

    const heads = headerMap(response.headers);
    if (opts.onResponse) await opts.onResponse({ status: response.status, headers: heads, url: request.url });

    // Nothing changed: what is held is still current.
    if (response.status === 304 && entry && entry.value !== undefined) {
      entry.at = Date.now();
      return entry.value;
    }
    if (entry) {
      entry.etag = heads.etag || null;
      entry.modified = heads["last-modified"] || null;
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
    if (opts.withHeaders) return { value, headers: heads, status: response.status };
    return value;
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

  /// A response read line by line, as it arrives: `Stream<T>` on an
  /// endpoint, and what a log viewer or a token stream is made of.
  async function* lines(url, opts) {
    const response = await fetch(url, {
      method: (opts && opts.method) || "GET",
      headers: (opts && opts.headers) || {},
      body: opts && opts.body,
      signal: opts && opts.signal,
    });
    if (!response.body) return;
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let held = "";
    for (;;) {
      const { value, done } = await reader.read();
      if (done) break;
      held += decoder.decode(value, { stream: true });
      const parts = held.split("\n");
      held = parts.pop();
      for (const line of parts) {
        if (!line.trim()) continue;
        try {
          yield JSON.parse(line);
        } catch (e) {
          yield line;
        }
      }
    }
    if (held.trim()) {
      try {
        yield JSON.parse(held);
      } catch (e) {
        yield held;
      }
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
