  // ─── Resource ────────────────────────────────────────
  //
  // A request declared once and read anywhere. What it holds is one value,
  // `read()`: `"loading"`, `["failed", e]`, or `["loaded", value,
  // refreshing, stale]` — and `value()`, `isLoading()`, `isLoaded()`,
  // `isFailed()`, `isRefreshing()` and `error()` are read from it. `reload()`
  // asks again. A URL that reads state is followed: the request is made again
  // when it changes, and an answer to an older request is ignored. Asking
  // again for what is already shown is a refresh: it stays loaded, with the
  // value, while it runs.
  // `let r = await fetch(url, opts)` in an action or a handler: the parsed
  // JSON body, or a thrown `Error` on a failed response — the request a
  // `resource` makes, made once. The browser's own is `window.fetch`.
  /// What a mutation shows before the server has answered, and takes back
  /// when it refuses.
  ///
  /// The change is made at once, and what was there is kept; if the request
  /// that follows fails, `rollback()` puts it back.
  let _undo = null;
  // `holder` is a state's signal, or `{ get, set }` over a place the
  // compiler cannot hand as one (a store's state, `Todos.items`).
  function optimistic(holder, change) {
    const get = typeof holder === "function" ? holder : holder.get;
    const set = typeof holder === "function" ? holder.set : holder.set;
    const before = get();
    set(change(before));
    const undo = { rollback: () => set(before) };
    // Inside an action, the rollback is the action's to do: if anything
    // after this throws, what was shown is taken back.
    if (_undo) _undo.push(undo);
    return undo;
  }

  /// An action that showed something before the server agreed: what it
  /// showed is taken back if it throws.
  async function attempt(fn) {
    const outer = _undo;
    const mine = [];
    _undo = mine;
    try {
      return await fn();
    } catch (e) {
      for (const undo of mine.reverse()) undo.rollback();
      throw e;
    } finally {
      _undo = outer;
    }
  }

  /// One request through the engine — what `http.get(…)` and the other verbs
  /// send, wrapped in a result by `httpCall` — so it
  /// gets the timeout, the retries and the typed errors every other
  /// request gets.
  function request(url, options) {
    return send(url, requestOptions(options));
  }

  /// `http.post(url, body: draft)`: a request of the built-in service, its
  /// answer `.ok(value)`, or `.err(error)` with an `HttpError`.
  function httpCall(method, url, options) {
    return send(url, requestOptions({ ...(options || {}), method })).then(
      (value) => ["ok", value],
      (error) => ["err", httpError(error)],
    );
  }

  /// The options a hand-written `fetch(…)` passes, as the engine takes them.
  function requestOptions(options) {
    const given = options || {};
    return {
      method: given.method,
      headers: given.headers,
      body: given.body,
      as: given.as,
      // Thirty seconds unless the call says otherwise: a request with no
      // limit waits for ever on a server that never answers.
      timeout: given.timeout != null ? given.timeout : 30000,
      retry: given.retry,
      // What the server says of caching, unless the call overrides it.
      cache: given.cache,
      credentials: given.credentials,
      mode: given.mode,
      redirect: given.redirect,
      referrerPolicy: given.referrerPolicy,
      signal: given.signal,
      shape: given.shape,
    };
  }

  /// What arrived, sharing what it has in common with what was there.
  ///
  /// A part of `next` equal to the part of `prev` it stands for is that
  /// part, the same object: an answer that changed nothing is `prev`
  /// itself, which a signal does not pass on, and one that changed a row is
  /// a new list whose other rows are the rows that were drawn — so a keyed
  /// list keeps their nodes and a branch over them does not draw again. An
  /// item of a list stands for the item with the same `id`, or for an equal
  /// one, or else for the one in its place. Only plain JSON is shared; a
  /// `Blob`, a `File` or a class instance is taken as it came.
  function reconcile(prev, next) {
    if (prev === next || !plainJson(prev) || !plainJson(next)) return next;
    if (Array.isArray(prev) !== Array.isArray(next)) return next;
    let same;
    let out;
    if (Array.isArray(next)) {
      same = prev.length === next.length;
      const byId = new Map();
      let byValue = null;
      for (const p of prev) if (hasId(p)) byId.set(p.id, p);
      out = next.map((n, i) => {
        let was;
        if (hasId(n)) {
          was = byId.get(n.id);
        } else if (plainJson(n)) {
          if (!byValue) {
            byValue = new Map();
            for (const p of prev) if (plainJson(p) && !hasId(p)) byValue.set(JSON.stringify(p), p);
          }
          was = byValue.get(JSON.stringify(n));
        }
        if (was === undefined) was = prev[i];
        const r = was === undefined ? n : reconcile(was, n);
        if (r !== prev[i]) same = false;
        return r;
      });
    } else {
      const keys = Object.keys(next);
      same = keys.length === Object.keys(prev).length;
      out = {};
      for (const k of keys) {
        const had = Object.prototype.hasOwnProperty.call(prev, k);
        const r = had ? reconcile(prev[k], next[k]) : next[k];
        if (!had || r !== prev[k]) same = false;
        out[k] = r;
      }
    }
    return same ? prev : out;
  }
  function plainJson(v) {
    if (v === null || typeof v !== "object") return false;
    if (Array.isArray(v)) return true;
    const proto = Object.getPrototypeOf(v);
    return proto === Object.prototype || proto === null;
  }
  function hasId(v) {
    return plainJson(v) && !Array.isArray(v) && v.id != null;
  }

  /// `resource rows: [Row] = http.get("/api/rows")`, and `resource rows =
  /// Backend.rows(page: n)`.
  ///
  /// `url` is an address — a string, or a function of one that follows
  /// state — or a function that makes the request itself, which is what an
  /// `api` endpoint is. Either way the request is abandoned when the scope
  /// leaves or the address changes, so a superseded answer never lands and
  /// a page that is gone is never waited for.
  ///
  /// A request is a first load, or a refresh. A first load — nothing has
  /// arrived yet, or the request changed (an address or an argument that
  /// reads state now names something else, so what is shown would be
  /// wrong) — is "loading" until it answers, and "error" if it fails. A
  /// refresh — the same request asked again, by `reload()`, `invalidate()`,
  /// `on:` or the next page of `paginate` — keeps "ready" and the data
  /// while it runs, with `refreshing()` true; what it brings replaces the
  /// data in place, sharing what did not change (`reconcile`). A refresh
  /// that fails keeps the data and "ready", and sets `error()` — until the
  /// next answer clears it — so a page can say the data is stale. A first
  /// load for another request drops what the last one brought: `data()` is
  /// null (and a paged resource's `items()` empty) until it answers, and
  /// stays so if it fails.
  /// `pending()` is true while any request of it runs, either kind.
  /// The arm an editor's preview holds every resource in — the page's root
  /// carrying `data-wf-resource="loading"` or `"error"` — so a page can be
  /// designed as it loads and as it fails without breaking its requests.
  /// A signal, so the page follows the attribute as it comes and goes.
  let heldArm = null;
  function armHeld() {
    if (typeof document === "undefined" || !document.documentElement) return null;
    if (!heldArm) {
      const root = document.documentElement;
      const read = () => {
        const v = root.getAttribute("data-wf-resource");
        return v === "loading" || v === "error" ? v : null;
      };
      heldArm = signal(read());
      if (typeof MutationObserver === "function") {
        new MutationObserver(() => heldArm.set(read()))
          .observe(root, { attributes: true, attributeFilter: ["data-wf-resource"] });
      }
    }
    return heldArm();
  }

  function resource(url, options) {
    // An endpoint that answers with a result is read here as it was: its
    // value, or its failure thrown.
    if (options && options.call && url && url.raw) url = Object.assign(url.raw, url);
    const real = signal("loading");
    const data = signal(null);
    const failed = signal(null);
    // What the page reads: the request's own state and error, unless the
    // preview holds the arm, when an error it has not met reads as one a
    // server gives.
    const state = () => armHeld() || real();
    state.set = (v) => real.set(v);
    const error = () => armHeld() === "error" ? failed() || netError("status", "503 Service Unavailable", { status: 503 }) : failed();
    error.set = (v) => failed.set(v);
    // What a failure is to the page: the endpoint's error enum, or an
    // `HttpError` — what `.err` holds for the same call.
    const failureOf = (e) => e == null ? null : options && options.call && url.failure ? url.failure(e) : httpError(e);
    // `placeholder: .previous`: another request keeps the last value shown,
    // refreshing, until its own answer arrives.
    const keepPrevious = options && options.placeholder === "previous";
    const refreshing = signal(false);
    const pending = signal(false);
    const opts = requestOptions(options);
    // A body and headers are read on every load: one that reads state
    // makes another request when it changes.
    const bodyNow = () => typeof opts.body === "function" ? opts.body() : opts.body;
    const headersOf = () => headersNow(typeof opts.headers === "function" ? opts.headers() : opts.headers);
    const argsOf =
      options && typeof options.args === "function"
        ? options.args
        : () => (options && options.args) || {};
    let generation = 0;
    let controller = null;
    // `pages` on the endpoint: a read gathered a page at a time. Its value
    // is every page's items, in order; each page knows what asks for the
    // one after it — a number, a cursor, the address its `Link` names — or
    // that there is none. A refresh asks again for every page gathered, in
    // order, and shows them together.
    const paging = options && options.call && url.pages ? url.pages : null;
    // `streamed`: a read that grows a line at a time.
    const streaming = !!(options && options.call && url.streamed && url.stream);
    const items = signal([]);
    const hasMore = signal(false);
    let pages = [];
    // A next page asked for and not yet answered: another is not asked for
    // on top of it.
    let gathering = false;
    let more = false;
    const linkNext = (got) => {
      const header = got && got.headers && got.headers.link;
      if (!header) return null;
      for (const part of String(header).split(/,(?=\s*<)/)) {
        // `<…?page=3>; rel="next"` (RFC 8288): a rel may name several.
        const m = /^<([^>]*)>(.*)$/.exec(part.trim());
        const rel = m && /;\s*rel\s*=\s*(?:"([^"]*)"|([^\s;]+))/i.exec(m[2]);
        if (!rel || !(rel[1] || rel[2]).toLowerCase().split(/\s+/).includes("next")) continue;
        try {
          const here = typeof location !== "undefined" && location.href ? location.href : "http://localhost/";
          return new URL(m[1], new URL(got.url || "", here)).href;
        } catch (e) {
          return m[1];
        }
      }
      return null;
    };
    /// One page: the one asked with `at` (the first when `at` is null).
    const askPage = (base, at, signalOf) => {
      const args = { ...base, signal: signalOf };
      if (at != null) {
        if (paging.style === "link") args.$address = at;
        else args[paging.param] = at;
      }
      return url(args).then((got) => {
        const answer = paging.style === "link" ? got && got.value : got;
        const held = paging.items ? paging.items(answer) : answer;
        const list = Array.isArray(held) ? held : [];
        let next = null;
        if (paging.style === "number") {
          const n = at != null ? Number(at) : Number(base[paging.param]) || 1;
          next = list.length > 0 ? n + 1 : null;
        } else if (paging.style === "cursor") {
          const c = paging.next ? paging.next(answer) : null;
          next = c == null || c === "" ? null : c;
        } else {
          next = linkNext(got);
        }
        return { items: list, next };
      });
    };
    /// The pages a load asks for: the next one, or the first and as many
    /// after it as were gathered; what they hold, together.
    const gatherPages = async (base, signalOf, mode) => {
      if (mode === "more") {
        const last = pages[pages.length - 1];
        const page = await askPage(base, last ? last.next : null, signalOf);
        const now = pages.concat([page]);
        return { now, all: now.flatMap((p) => p.items) };
      }
      const count = mode === "refresh" ? Math.max(1, pages.length) : 1;
      const now = [];
      let at = null;
      for (let i = 0; i < count; i++) {
        const page = await askPage(base, at, signalOf);
        now.push(page);
        if (page.next == null) break;
        at = page.next;
      }
      return { now, all: now.flatMap((p) => p.items) };
    };
    // Which request this is: what has arrived for it may stay on the page
    // while it is asked for again.
    const requestOf = () => {
      if (options && options.call) {
        try { return JSON.stringify({ ...argsOf() }); } catch (e) { return String(Math.random()); }
      }
      const address = typeof url === "function" ? url() : url;
      return cacheKey(address, { method: opts.method, body: bodyNow() }) + " " + JSON.stringify(headersOf());
    };
    // The request whose answer `data` holds, once one has arrived.
    let shown;
    let arrived = false;
    const settle = () => { refreshing.set(false); pending.set(false); gathering = false; more = false; };

    const load = () => {
      const gen = ++generation;
      if (controller) controller.abort();
      controller = typeof AbortController !== "undefined" ? new AbortController() : null;
      const signalOf = controller ? controller.signal : undefined;
      const request = requestOf();
      const again = arrived && (request === shown || keepPrevious);
      const mode = more && again ? "more" : again && request === shown ? "refresh" : "first";
      if (again) {
        refreshing.set(true);
      } else {
        // Another request: what the last one brought is not its answer,
        // so it goes — a derived value over `data` read the old answer
        // while the new one loaded, and still read it if that failed.
        refreshing.set(false);
        state.set("loading");
        error.set(null);
        if (arrived) {
          arrived = false;
          data.set(null);
          if (paging) {
            items.set([]);
            hasMore.set(false);
            pages = [];
          }
        }
      }
      pending.set(true);
      // An endpoint is a call; an address is a request.
      // Asked again — `reload()`, a refresh — goes to the server whatever a
      // kept answer's freshness, sending what it holds to revalidate.
      const args = options && options.call ? (again ? { ...argsOf(), $ask: true } : argsOf()) : null;
      let made;
      if (paging) {
        made = gatherPages(args, signalOf, mode).then(({ now, all }) => {
          if (gen === generation) pages = now;
          return all;
        });
      } else if (streaming) {
        // A first load is `.loaded` from its first line, the list growing as
        // the rest arrive; a refresh gathers them, and shows them whole.
        const lines = [];
        made = url.stream({ ...args, signal: signalOf }, (line) => {
          if (gen !== generation) return;
          lines.push(line);
          if (again) return;
          arrived = true;
          shown = request;
          data.set(lines.slice());
          if (real() !== "ready") {
            error.set(null);
            state.set("ready");
          }
        }).then(() => lines);
      } else {
        made = options && options.call
          ? url({ ...args, signal: signalOf })
          : send(typeof url === "function" ? url() : url, { ...opts, body: bodyNow(), headers: headersOf(), signal: signalOf, ask: again });
      }
      made.then(
        (value) => {
          if (gen !== generation) return;
          arrived = true;
          shown = request;
          data.set(reconcile(data(), value));
          if (paging) {
            items.set(data());
            hasMore.set(pages.length > 0 && pages[pages.length - 1].next != null);
          }
          error.set(null);
          settle();
          state.set("ready");
        },
        (e) => {
          if (gen !== generation) return;
          settle();
          // Not asked (a path parameter with no value yet) or abandoned:
          // nothing to say, and a first load stays "loading".
          if (e && e.kind === "aborted") return;
          error.set(e && e.kind ? e : netError("network", String((e && e.message) || e)));
          // A stream that failed after its first lines keeps them.
          if (!again && !arrived) state.set("error");
        },
      );
    };

    // An address or an argument that reads state asks again when it changes.
    if (options && options.call) {
      effect(() => { argsOf(); load(); });
    } else if (typeof url === "function" || typeof opts.body === "function" || typeof opts.headers === "function") {
      effect(() => { requestOf(); load(); });
    } else {
      load();
    }
    // Leaving takes the request with it.
    onCleanup(() => { generation += 1; if (controller) controller.abort(); });
    // A read of an endpoint is one its `update` reaches.
    if (options && options.call && url.reads) {
      const read = { data };
      url.reads.add(read);
      onCleanup(() => url.reads.delete(read));
    }

    // What the cache knows this request by; an `invalidate` that names it —
    // the endpoint's, the service's, everything — asks again.
    const keyNow = () => options && options.call && url.key
      ? url.key(argsOf())
      : cacheKey(typeof url === "function" ? url() : url, { method: opts.method, body: bodyNow() });
    let invalidating = false;
    onCleanup(onInvalidate((prefix) => {
      if (invalidating) return;
      // Heard inside whatever called `invalidate`, which may be an effect:
      // what this reads is not that effect's to follow.
      const outer = currentEffect;
      currentEffect = null;
      try {
        const key = keyNow();
        if (prefix == null || key === prefix || key.startsWith(prefix)) load();
      } finally {
        currentEffect = outer;
      }
    }));

    // `refresh:` — another look when the page comes back into focus, when
    // the connection returns (unless it says otherwise), or every so often.
    const on = (options && options.refresh) || { reconnect: true };
    if (typeof window !== "undefined" && window.addEventListener) {
      if (on.focus) listen(window, "focus", () => load());
      if (on.reconnect) listen(window, "online", () => load());
      if (on.interval) every(on.interval, () => load());
    }

    const read = () => {
      const now = state();
      if (now === "loading") return "loading";
      if (now === "error") return ["failed", failureOf(error())];
      return ["loaded", data(), refreshing(), failureOf(error())];
    };
    return {
      read,
      value: () => (state() === "ready" ? data() : null),
      isLoading: () => state() === "loading",
      isLoaded: () => state() === "ready",
      isFailed: () => state() === "error",
      isRefreshing: () => state() === "ready" && refreshing(),
      error: () => failureOf(error()),
      state,
      data,
      refreshing,
      pending,
      items,
      hasMore,
      /// The next page, gathered onto what is shown.
      loadMore() {
        if (!paging || !hasMore() || state() !== "ready" || pending() || gathering) return;
        gathering = true;
        more = true;
        load();
      },
      reload: load,
      /// Drop what was cached for it, and ask again.
      invalidate() {
        invalidating = true;
        try { invalidate(keyNow()); } finally { invalidating = false; }
        load();
      },
      cancel() {
        generation += 1;
        if (controller) controller.abort();
        settle();
      },
    };
  }
