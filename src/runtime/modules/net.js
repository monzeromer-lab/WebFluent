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
  // when it changes, and an answer to an older request is ignored.
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

  /// `resource rows = fetch("/api/rows")`, and `resource rows =
  /// Backend.rows(page: n)`.
  ///
  /// `url` is an address — a string, or a function of one that follows
  /// state — or a function that makes the request itself, which is what an
  /// `api` endpoint is. Either way the request is abandoned when the scope
  /// leaves or the address changes, so a superseded answer never lands and
  /// a page that is gone is never waited for.
  function resource(url, options) {
    const state = signal("loading");
    const data = signal(null);
    const error = signal(null);
    const opts = requestOptions(options);
    const argsOf =
      options && typeof options.args === "function"
        ? options.args
        : () => (options && options.args) || {};
    let generation = 0;
    let controller = null;

    const load = () => {
      const gen = ++generation;
      if (controller) controller.abort();
      controller = typeof AbortController !== "undefined" ? new AbortController() : null;
      const signalOf = controller ? controller.signal : undefined;
      state.set("loading");
      error.set(null);
      // An endpoint is a call; an address is a request.
      const made = options && options.call
        ? url({ ...argsOf(), signal: signalOf })
        : send(typeof url === "function" ? url() : url, { ...opts, signal: signalOf });
      made.then(
        (value) => {
          if (gen !== generation) return;
          data.set(value);
          state.set("ready");
        },
        (e) => {
          if (gen !== generation || (e && e.kind === "aborted")) return;
          error.set(e && e.kind ? e : netError("network", String((e && e.message) || e)));
          state.set("error");
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
      },
    };
  }
