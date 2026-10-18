  // ─── Fetch ───────────────────────────────────────────
  function wfFetch(url, options, callbacks) {
    const container = document.createDocumentFragment();
    const wrapper = document.createElement("div");
    wrapper.style.display = "contents";

    const loading = signal(true);
    const error = signal(null);
    const data = signal(null);

    // A screen reader is told the region is still filling, and then told once —
    // politely — when it has. Without this a fetch completes in silence and the
    // user has no way to know the content arrived.
    wrapper.setAttribute("aria-live", "polite");
    wrapper.setAttribute("aria-busy", "true");
    effect(() => { wrapper.setAttribute("aria-busy", loading() ? "true" : "false"); });

    // Show loading
    if (callbacks.loading) {
      const loadingEl = document.createElement("div");
      loadingEl.style.display = "contents";
      const nodes = [].concat(callbacks.loading()).flat();
      for (const n of nodes) { if (n instanceof Node) loadingEl.appendChild(n); }
      wrapper.appendChild(loadingEl);
      effect(() => { loadingEl.style.display = loading() ? "contents" : "none"; });
    }

    // Success container
    const successEl = document.createElement("div");
    successEl.style.display = "contents";
    wrapper.appendChild(successEl);

    // Error container
    const errorEl = document.createElement("div");
    errorEl.style.display = "contents";
    wrapper.appendChild(errorEl);

    const resolvedUrl = typeof url === "function" ? url() : url;

    const doFetch = () => {
      const fetchUrl = typeof url === "function" ? url() : url;
      loading.set(true);
      error.set(null);

      const fetchOpts = {};
      if (options) {
        if (options.method) fetchOpts.method = options.method;
        if (options.headers) fetchOpts.headers = options.headers;
        if (options.body) {
          fetchOpts.body = JSON.stringify(typeof options.body === "function" ? options.body() : options.body);
          fetchOpts.headers = { "Content-Type": "application/json", ...(fetchOpts.headers || {}) };
        }
      }

      fetch(fetchUrl, fetchOpts)
        .then(r => { if (!r.ok) throw new Error(`HTTP ${r.status}`); return r.json(); })
        .then(d => {
          data.set(d);
          loading.set(false);
          if (callbacks.success) {
            successEl.innerHTML = "";
            const nodes = [].concat(callbacks.success(d)).flat();
            for (const n of nodes) { if (n instanceof Node) successEl.appendChild(n); }
          }
        })
        .catch(e => {
          error.set(e);
          loading.set(false);
          if (callbacks.error) {
            errorEl.innerHTML = "";
            const nodes = [].concat(callbacks.error(e)).flat();
            for (const n of nodes) { if (n instanceof Node) errorEl.appendChild(n); }
          }
        });
    };

    doFetch();

    return wrapper;
  }

  // ─── Resource ────────────────────────────────────────
  //
  // A request declared once and read anywhere. `state()` is "loading",
  // "ready" or "error"; `data()` and `error()` hold what arrived; `reload()`
  // asks again. A URL that reads state is followed: the request is made again
  // when it changes, and an answer to an older request is ignored. Asking
  // again for what is already shown is a refresh: it keeps "ready" and the
  // data while it runs, and `refreshing()` says it is running.
  // `let r = await fetch(url, opts)` in an action or a handler: the parsed
  // JSON body, or a thrown `Error` on a failed response — the request a
  // `resource` makes, made once. The browser's own is `window.fetch`.
  /// What a mutation shows before the server has answered, and takes back
  /// when it refuses.
  ///
  /// The change is made at once, and what was there is kept; if the request
  /// that follows fails, `rollback()` puts it back.
  let _undo = null;
  function optimistic(holder, change) {
    const before = holder();
    holder.set(change(before));
    const undo = { rollback: () => holder.set(before) };
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

  /// `await fetch(…)` in an action: one request, through the engine, so it
  /// gets the timeout, the retries and the typed errors every other
  /// request gets.
  function request(url, options) {
    return send(url, requestOptions(options));
  }

  /// The options a hand-written `fetch(…)` passes, as the engine takes them.
  function requestOptions(options) {
    const given = options || {};
    return {
      method: given.method,
      headers: given.headers,
      body: given.body,
      as: given.as,
      timeout: given.timeout,
      retry: given.retry,
      cache: given.cache || { kind: "none" },
      credentials: given.credentials,
      mode: given.mode,
      redirect: given.redirect,
      referrerPolicy: given.referrerPolicy,
      signal: given.signal,
      shape: given.shape,
    };
  }

  function fetchOptions(options) {
    const opts = {};
    if (options) {
      if (options.method) opts.method = options.method;
      if (options.headers) opts.headers = options.headers;
      if (options.body) {
        opts.body = JSON.stringify(typeof options.body === "function" ? options.body() : options.body);
        opts.headers = { "Content-Type": "application/json", ...(opts.headers || {}) };
      }
    }
    return opts;
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

  /// `resource rows = fetch("/api/rows")`, and `resource rows =
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
  /// next answer clears it — so a page can say the data is stale.
  /// `pending()` is true while any request of it runs, either kind.
  function resource(url, options) {
    const state = signal("loading");
    const data = signal(null);
    const error = signal(null);
    const refreshing = signal(false);
    const pending = signal(false);
    const opts = requestOptions(options);
    const argsOf =
      options && typeof options.args === "function"
        ? options.args
        : () => (options && options.args) || {};
    let generation = 0;
    let controller = null;
    // `paginate: .page` — the pages gathered so far, and whether the last
    // one had anything in it. `by` names the argument that is the page
    // number; `set` moves the state it came from, when it came from one.
    const paging = options && options.paginate;
    const items = signal([]);
    const hasMore = signal(true);
    let extraPage = null;
    // A next page asked for and not yet answered: another is not asked for
    // on top of it, which would abandon it.
    let gathering = false;
    const pageArgs = () => {
      const args = argsOf();
      if (paging && extraPage != null) args[paging.by] = extraPage;
      return args;
    };
    // Which request this is, the page `paginate` moves left out: what has
    // arrived for it may stay on the page while it is asked for again.
    const requestOf = () => {
      if (options && options.call) {
        const args = { ...argsOf() };
        if (paging) delete args[paging.by];
        try { return JSON.stringify(args); } catch (e) { return String(Math.random()); }
      }
      const address = typeof url === "function" ? url() : url;
      return (opts.method || "GET").toUpperCase() + " " + address;
    };
    // The request whose answer `data` holds, once one has arrived.
    let shown;
    let arrived = false;
    const settle = () => { refreshing.set(false); pending.set(false); gathering = false; };

    const load = () => {
      const gen = ++generation;
      if (controller) controller.abort();
      controller = typeof AbortController !== "undefined" ? new AbortController() : null;
      const signalOf = controller ? controller.signal : undefined;
      const request = requestOf();
      const again = arrived && request === shown;
      if (again) {
        refreshing.set(true);
      } else {
        refreshing.set(false);
        state.set("loading");
        error.set(null);
      }
      pending.set(true);
      // An endpoint is a call; an address is a request.
      const args = options && options.call ? pageArgs() : null;
      const made = options && options.call
        ? url({ ...args, signal: signalOf })
        : send(typeof url === "function" ? url() : url, { ...opts, signal: signalOf });
      made.then(
        (value) => {
          if (gen !== generation) return;
          arrived = true;
          shown = request;
          data.set(reconcile(data(), value));
          if (paging) {
            const page = Number(args && args[paging.by]) || 1;
            const got = Array.isArray(value) ? value : (value && Array.isArray(value.items) ? value.items : []);
            items.set(page <= 1 ? reconcile(items(), got) : [...items(), ...got]);
            hasMore.set(got.length > 0);
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
          if (!again) state.set("error");
        },
      );
    };

    // An address or an argument that reads state asks again when it changes.
    if (options && options.call) {
      effect(() => { argsOf(); load(); });
    } else if (typeof url === "function") {
      effect(() => { url(); load(); });
    } else {
      load();
    }
    // Leaving takes the request with it.
    onCleanup(() => { generation += 1; if (controller) controller.abort(); });

    // What the page asked for: another look when it comes back into focus,
    // when the connection returns, or every so often.
    const on = options && options.on;
    if (on && typeof window !== "undefined" && window.addEventListener) {
      if (on.focus) listen(window, "focus", () => load());
      if (on.reconnect) listen(window, "online", () => load());
      if (on.interval) every(on.interval, () => load());
    }

    return {
      state,
      data,
      error,
      refreshing,
      pending,
      items,
      hasMore,
      /// The next page, gathered onto `items`.
      loadMore() {
        if (!paging || !hasMore() || state() === "loading" || gathering) return;
        const next = (Number(pageArgs()[paging.by]) || 1) + 1;
        gathering = true;
        if (paging.set) paging.set(next);
        else { extraPage = next; load(); }
      },
      reload: load,
      /// Drop what was cached for it, and ask again.
      invalidate() {
        if (options && options.call && url.invalidate) url.invalidate(argsOf());
        else invalidate(cacheKey(typeof url === "function" ? url() : url, opts));
        load();
      },
      cancel() {
        generation += 1;
        if (controller) controller.abort();
        settle();
      },
    };
  }
