  // ─── A service, described once ───────────────────────
  //
  // An `api` declaration compiles to one call of this: the settings the
  // service is reached with, and the endpoints it has. What comes back is
  // an object of callable endpoints — `Backend.users(page: 2)` — each of
  // which also carries `.invalidate()`, `.prefetch(args)`, `.progress` and
  // `.key(args)`, so a page can say what it wants without writing a client.
  function api(config) {
    const client = {};
    if (config.name && config.onRequest) _requestHooks.set(config.name, config.onRequest);
    const base = () => {
      const b = typeof config.base === "function" ? config.base() : config.base;
      return (b || "").replace(/\/+$/, "");
    };
    for (const [name, endpoint] of Object.entries(config.endpoints || {})) {
      client[name] = makeEndpoint(config, base, name, endpoint, client);
    }
    // The whole service's cache, for a mutation that changes everything.
    client.invalidate = () => invalidate(config.name ? config.name + " " : null);
    // What a group of it is reached through.
    Object.defineProperty(client, "$reach", { value: { config, base }, enumerable: false });
    return client;
  }

  /// `api Backend.users { … }`: a group of a service's endpoints, reached
  /// through it — its address, its middleware, its cache — as
  /// `Backend.users.list(…)`.
  function group(client, name, spec) {
    const { config, base } = client.$reach;
    const members = {};
    for (const [ep, endpoint] of Object.entries((spec && spec.endpoints) || {})) {
      members[ep] = makeEndpoint(config, base, name + "." + ep, endpoint, members);
    }
    // The group's cache, every endpoint of it.
    members.invalidate = () => invalidate((config.name || "") + " " + name + ".");
    client[name] = members;
    return members;
  }

  /// A key no other call has: what an `Idempotency-Key` carries.
  function oneKey() {
    if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") return crypto.randomUUID();
    return Date.now().toString(36) + "-" + Math.random().toString(36).slice(2) + Math.random().toString(36).slice(2);
  }

  /// The path with its `{name}` parts filled in (and 6.x's `:name`), and
  /// what is left over as the query string.
  function fillPath(path, args) {
    const used = new Set();
    const fill = (whole, name) => {
      if (!(name in args)) return whole;
      used.add(name);
      return encodeURIComponent(String(args[name]));
    };
    const filled = String(path)
      .replace(/\{([A-Za-z_][A-Za-z0-9_]*)\}/g, fill)
      .replace(/:([A-Za-z_][A-Za-z0-9_]*)/g, fill);
    return { path: filled, used };
  }

  /// The names a path takes its parts from.
  function pathNames(path) {
    const names = [];
    String(path).replace(/\{([A-Za-z_][A-Za-z0-9_]*)\}|:([A-Za-z_][A-Za-z0-9_]*)/g, (whole, a, b) => {
      names.push(a || b);
      return whole;
    });
    return names;
  }

  function queryString(args, used, skip) {
    const parts = [];
    for (const [key, value] of Object.entries(args || {})) {
      if (used.has(key) || skip.has(key) || value == null) continue;
      if (Array.isArray(value)) {
        for (const item of value) parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(item))}`);
      } else {
        parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
      }
    }
    return parts.length ? "?" + parts.join("&") : "";
  }

  function makeEndpoint(config, base, name, endpoint, siblings) {
    const progress = signal(0);
    const method = (endpoint.method || "GET").toUpperCase();
    // What is a parameter of the call rather than of the request.
    const perCall = new Set(["body", "file", "headers", "retry", "cache", "on", "refresh", "placeholder", "timeout", "signal", "as", "$address"]);
    // The parameter a file arrives in is the one the endpoint names — `post
    // avatar(image: File)` sends `image` — and never part of the query.
    const fileField = endpoint.fileField || "file";
    perCall.add(fileField);
    const fileOf = (given) => given[fileField] != null ? given[fileField] : given.file;

    // Where a parameter goes that is not in the path, the body or the
    // query: `in header "X-Tenant"`, `in cookie`.
    const placed = endpoint.locations || {};
    for (const p of Object.keys(placed)) perCall.add(p);
    const addressOf = (args) => {
      const given = args || {};
      // The next page of a `pages .link` read: the address the server
      // named, as it named it.
      if (given.$address) return given.$address;
      const { path, used } = fillPath(endpoint.path, given);
      const url = /^[a-z]+:\/\//i.test(path) ? path : base() + "/" + path.replace(/^\/+/, "");
      // A query parameter the server names otherwise: `in query "user-id"`.
      const renamed = {};
      for (const [p, where] of Object.entries(placed)) {
        if (where[0] === "query" && given[p] != null) renamed[where[1]] = given[p];
      }
      const query = queryString(given, used, perCall);
      const more = queryString(renamed, new Set(), new Set());
      return url + (more ? (query ? query + "&" + more.slice(1) : more) : query);
    };
    /// The headers a call's own parameters put on it.
    const placedHeaders = (given) => {
      const out = {};
      for (const [p, where] of Object.entries(placed)) {
        if (given[p] == null) continue;
        if (where[0] === "header") out[where[1]] = String(given[p]);
        else if (where[0] === "cookie" && typeof document !== "undefined") {
          document.cookie = encodeURIComponent(where[1] || p) + "=" + encodeURIComponent(String(given[p])) + "; path=/";
        }
      }
      return out;
    };

    const optionsOf = (args) => {
      const given = args || {};
      let body = given.body;
      // A file is sent as a form, which is what a server expects of one.
      const file = fileOf(given);
      if (file != null && typeof FormData !== "undefined") {
        const form = new FormData();
        form.append(fileField, file);
        for (const [k, v] of Object.entries(body || {})) form.append(k, v);
        body = form;
      }
      return {
        method,
        body,
        headers: { ...(config.headers || {}), ...placedHeaders(given), ...(given.headers || {}) },
        as: given.as || endpoint.as || "json",
        errorAs: endpoint.errorAs || "json",
        timeout: given.timeout != null ? given.timeout : config.timeout != null ? config.timeout : 30000,
        // An endpoint the contract marks `idempotent` may be sent again —
        // retried, replayed — and says so with one key per call, the same
        // on every try, so the server does what it asks once.
        idempotencyKey: endpoint.idempotent ? oneKey() : undefined,
        middleware: config.middleware,
        retry: given.retry || config.retry,
        cache: given.cache || endpoint.cache || (method === "GET" ? config.cache : { kind: "none" }),
        credentials: config.credentials,
        mode: config.mode,
        redirect: config.redirect,
        referrerPolicy: config.referrerPolicy,
        signal: given.signal,
        withHeaders: endpoint.withHeaders,
        progress: fileOf(given) != null || endpoint.progress ? progress : null,
        onRequest: config.onRequest,
        api: config.name || null,
        onResponse: config.onResponse,
        onError: config.onError,
        shape: endpoint.shape,
        errorShapes: endpoint.errorShapes,
        key: (config.name || "") + " " + name + " " + addressOf(given),
        // What a success makes stale, by cache key: a write kept offline
        // carries it, to apply once it is sent.
        stale: endpoint.invalidates && siblings
          ? () => endpoint.invalidates(given).map(([read, readArgs]) =>
              siblings[read] && siblings[read].staleKey ? siblings[read].staleKey(readArgs || undefined) : null).filter(Boolean)
          : null,
      };
    };

    // A path parameter with no value would ask for something else —
    // `users/` for `users/:id`, the whole collection. So would `.` or `..`,
    // which the browser takes as this folder and the one above:
    // `users/..` is the collection's parent. The call is not made: it ends
    // as aborted, which a resource over it reads as still loading, until the
    // value arrives and the address changes.
    const unfilled = (args) => {
      const given = args || {};
      for (const n of pathNames(endpoint.path)) {
        const v = given[n];
        if (v == null || v === "") return [n, "is empty"];
        if (String(v) === "." || String(v) === "..") return [n, "is `" + v + "`"];
      }
      return null;
    };
    // `streamed`: the answer a line at a time, each line an item of the
    // list it declares.
    const itemShape = endpoint.shape && endpoint.shape[0] === "l" ? endpoint.shape[1] : null;
    const lines = (args) => {
      // A fake answers the whole list; it arrives a line at a time.
      if (fakesOn()) {
        return (async function* () {
          const all = await faked(args);
          for (const line of all || []) yield line;
        })();
      }
      const empty = unfilled(args);
      if (empty) {
        return (async function* () {
          throw netError("aborted", "`" + empty[0] + "` " + empty[1] + ", so " + endpoint.path + " was not requested");
        })();
      }
      return streamResponse(addressOf(args), optionsOf(args), itemShape);
    };
    // On a page a test opens (or `wf verify --fakes`), nothing is sent:
    // the fake the test names answers, and every call is counted, so a test
    // can `expect called` it. A call no fake answers is the test's failure,
    // said by name.
    const whole = (config.name ? config.name + "." : "") + name;
    const faked = (args) => {
      const calls = window.__wfCalls || (window.__wfCalls = []);
      calls.push(whole);
      const answer = window.__wfFakes[whole];
      if (!answer) {
        const message = "No fake answers " + whole + ": name one that does in the test's `fakes:`";
        console.error(message);
        return Promise.reject(netError("transport", message));
      }
      return Promise.resolve()
        .then(() => answer(args || {}))
        .then((given) => {
          if (given === "ok") return undefined;
          if (Array.isArray(given) && given[0] === "ok") {
            return endpoint.withHeaders ? { value: given[1], headers: {}, status: 200, url: addressOf(args) } : given[1];
          }
          // What the fake said went wrong is the endpoint's error as it
          // stands: nothing to read from a status.
          throw { $faked: true, error: Array.isArray(given) ? given[1] : given };
        });
    };
    const fakesOn = () => typeof window !== "undefined" && window.__wfFakes != null;
    const raw = (args) => {
      if (fakesOn() && !endpoint.streamed) {
        const sent = faked(args);
        if (!endpoint.invalidates || !siblings) return sent;
        return sent.then((value) => {
          for (const [read, readArgs] of endpoint.invalidates(args || {})) {
            if (siblings[read] && siblings[read].invalidate) siblings[read].invalidate(readArgs || undefined);
          }
          return value;
        });
      }
      const empty = unfilled(args);
      if (empty) {
        return Promise.reject(netError("aborted", "`" + empty[0] + "` " + empty[1] + ", so " + endpoint.path + " was not requested"));
      }
      // A streamed answer awaited whole: every line, gathered.
      if (endpoint.streamed) {
        return (async () => {
          const all = [];
          for await (const line of lines(args)) all.push(line);
          return all;
        })();
      }
      const sent = send(addressOf(args), optionsOf(args));
      // `invalidates [list, byId(id)]`: what a success makes stale asks
      // again, wherever it is shown.
      if (!endpoint.invalidates || !siblings) return sent;
      return sent.then((value) => {
        for (const [read, readArgs] of endpoint.invalidates(args || {})) {
          if (siblings[read] && siblings[read].invalidate) siblings[read].invalidate(readArgs || undefined);
        }
        return value;
      });
    };
    // An endpoint that declares a result (`-> User ! CreateUserError`,
    // `-> User!`) never throws: what comes back is `.ok(value)` or
    // `.err(error)`. A resource reads it through `raw`, as it always has.
    const result = endpoint.result;
    const outcome = (made) => result
      ? made.then(
          (value) => (result.unit ? "ok" : ["ok", value]),
          (error) => ["err", failureOf(error, result)],
        )
      : made;
    // A streamed call is two things: awaited, every line as one result;
    // walked with `for await`, each line as it arrives. Nothing is asked
    // until one of them is wanted.
    const call = endpoint.streamed
      ? (args) => {
          let whole = null;
          const once = () => whole || (whole = outcome(raw(args)));
          return {
            then: (done, failed) => once().then(done, failed),
            catch: (failed) => once().catch(failed),
            finally: (after) => once().finally(after),
            [Symbol.asyncIterator]: () => lines(args)[Symbol.asyncIterator](),
          };
        }
      : (args) => outcome(raw(args));
    call.raw = raw;
    /// How a read of it pages, and whether it answers a line at a time.
    if (endpoint.pages) call.pages = endpoint.pages;
    if (endpoint.streamed) {
      call.streamed = true;
      /// Each line to `onLine` as it arrives; done when the answer is.
      call.stream = async (args, onLine) => {
        for await (const line of lines(args)) onLine(line);
      };
    }
    // What a read of it holds when it fails: what `.err` would.
    call.failure = (error) => (result ? failureOf(error, result) : error && error.$faked ? error.error : httpError(error));
    call.progress = progress;
    call.key = (args) => optionsOf(args).key;
    call.url = addressOf;
    /// What its cache is known by: one call's, or every call's.
    call.staleKey = (args) => (args ? optionsOf(args).key : (config.name || "") + " " + name + " ");
    /// Forget what this endpoint cached — every argument of it.
    call.invalidate = (args) => invalidate(call.staleKey(args));
    /// The reads of it shown on the page, which `update` changes in place.
    call.reads = new Set();
    /// `Backend.todos.list.update(list => …)`: what every read of it shows,
    /// and what the cache holds of it, changed at once — before the server
    /// has answered — and what undoes it, returned.
    call.update = (change) => {
      const undone = [];
      for (const read of call.reads) {
        const before = read.data();
        if (before == null) continue;
        read.data.set(change(before));
        undone.push(() => read.data.set(before));
      }
      // A paged read caches its pages, not the items it shows: the change
      // is of the items, so only what is shown takes it.
      if (!endpoint.pages) undone.push(updateCached((config.name || "") + " " + name + " ", change));
      let done = false;
      return () => {
        if (done) return;
        done = true;
        for (const undo of undone.reverse()) undo();
      };
    };
    /// Ask now for what will be wanted soon.
    call.prefetch = (args) => send(addressOf(args), optionsOf(args)).catch(() => null);
    /// Every line of a streamed response, as it arrives.
    call.lines = lines;
    return call;
  }

  /// What a request that failed is, as its endpoint declares it: the case
  /// of its error enum for a status the enum names, with the body; its
  /// `http` case for anything else; or the `HttpError` itself, for an
  /// endpoint that declares no enum (`-> User!`).
  function failureOf(error, result) {
    if (error && error.$faked) return error.error;
    const cases = result.cases || {};
    if (error && error.kind === "status") {
      // A status a case names, else the class it is in (`on 4XX`).
      const named = cases[error.status] != null ? cases[error.status] : cases[Math.floor(error.status / 100)];
      if (named != null) return [named, error.body];
    }
    const http = httpError(error);
    return result.enum ? ["http", http] : http;
  }

  // ─── A socket ────────────────────────────────────────
  //
  // `socket chat = ws("wss://…")`: a connection that reconnects with
  // backoff, keeps itself alive with a heartbeat, holds what was sent
  // while it was down, and closes when the scope that opened it leaves —
  // so a route change cannot leak one.
  function ws(url, options) {
    const opts = options || {};
    const state = signal("connecting");
    const messages = signal([]);
    const closure = signal(null);
    const failure = signal(null);
    const waiting = [];
    const keep = keepOf(opts);
    let socket = null;
    let retry = null;
    let attempt = 0;
    let heart = null;
    let alive = true;
    let pong = true;

    const open = () => {
      if (!alive) return;
      const address = typeof url === "function" ? url() : url;
      state.set(socket ? "connecting" : "connecting");
      try {
        socket = opts.protocols ? new WebSocket(address, opts.protocols) : new WebSocket(address);
      } catch (e) {
        failure.set(e);
        state.set("error");
        return;
      }
      if (opts.binaryType) socket.binaryType = opts.binaryType;
      socket.onopen = () => {
        attempt = 0;
        state.set("open");
        while (waiting.length) socket.send(waiting.shift());
        // `on open { }`, as a peer and a channel already had it — and each
        // time a reconnection opens the line again.
        if (opts.onOpen) opts.onOpen();
        if (opts.heartbeat) {
          pong = true;
          heart = setInterval(() => {
            // A missed answer means the line is dead however open it looks.
            if (!pong) { socket.close(4000, "no heartbeat"); return; }
            pong = false;
            try { socket.send(opts.ping || "ping"); } catch (e) { /* closing */ }
          }, opts.heartbeat);
        }
      };
      socket.onmessage = (event) => {
        pong = true;
        let data = event.data;
        if (typeof data === "string") {
          if (data === (opts.pong || "pong")) return;
          try { data = JSON.parse(data); } catch (e) { /* a plain string */ }
        }
        messages.update((held) => appendKept(held, data, keep));
        if (opts.onMessage) opts.onMessage(data);
      };
      socket.onerror = (e) => { failure.set(netError("network", "The connection failed")); };
      socket.onclose = (event) => {
        if (heart) { clearInterval(heart); heart = null; }
        closure.set({ code: event.code, reason: event.reason });
        if (!alive || opts.reconnect === false || event.code === 1000) {
          state.set("closed");
          return;
        }
        state.set("connecting");
        const wait = Math.min((opts.delay || 500) * Math.pow(2, attempt), opts.max || 30000);
        attempt += 1;
        retry = setTimeout(() => { retry = null; open(); }, wait * (0.5 + Math.random() / 2));
      };
    };
    open();

    const handle = {
      state,
      messages,
      closure,
      error: failure,
      /// Send a value; a value sent while the line is down waits for it.
      send(value) {
        const text = typeof value === "string" || value instanceof ArrayBuffer ? value : JSON.stringify(value);
        if (socket && socket.readyState === 1) socket.send(text);
        else if (opts.queue !== false) waiting.push(text);
      },
      /// The last message, or the last of a kind when the messages say what
      /// kind they are.
      last(kind) {
        const held = messages();
        for (let i = held.length - 1; i >= 0; i--) {
          if (kind == null || (held[i] && held[i].type === kind)) return held[i];
        }
        return null;
      },
      close(code, reason) {
        const wasAlive = alive;
        alive = false;
        if (heart) { clearInterval(heart); heart = null; }
        if (retry !== null) { clearTimeout(retry); retry = null; }
        waiting.length = 0;
        if (socket) {
          // Its handlers go now: a socket closed while it was still
          // connecting answers late, or never, and would hold them — and
          // the page they reach — until it did.
          const s = socket;
          socket = null;
          s.onopen = null;
          s.onmessage = null;
          s.onerror = null;
          s.onclose = null;
          try { s.close(code || 1000, reason || ""); } catch (e) { /* already closed */ }
          if (wasAlive) closure.set({ code: code || 1000, reason: reason || "" });
        }
        state.set("closed");
      },
    };
    // The scope that opened it closes it.
    onCleanup(() => handle.close(1000, "scope left"));
    return handle;
  }

  // ─── Server-sent events ──────────────────────────────
  //
  // `stream ticks = sse("/events", events: ["price"])`: the browser
  // reconnects on its own and remembers where it was, and the last message
  // of each named event is there to read.
  function sse(url, options) {
    const opts = options || {};
    const state = signal("connecting");
    const messages = signal([]);
    const latest = signal({});
    const failure = signal(null);
    const keep = keepOf(opts);
    let source = null;

    const open = () => {
      const address = typeof url === "function" ? url() : url;
      source = new EventSource(address, { withCredentials: !!opts.credentials });
      source.onopen = () => state.set("open");
      source.onerror = () => {
        // The browser retries by itself; `closed` is only for a source that
        // said it was done.
        state.set(source && source.readyState === 2 ? "closed" : "connecting");
        failure.set(netError("network", "The stream dropped"));
      };
      const take = (name) => (event) => {
        let data = event.data;
        try { data = JSON.parse(data); } catch (e) { /* a plain string */ }
        const message = { event: name, data, id: event.lastEventId };
        messages.update((held) => appendKept(held, message, keep));
        latest.update((held) => ({ ...held, [name]: data }));
        if (opts.onMessage) opts.onMessage(message);
      };
      source.onmessage = take("message");
      for (const name of opts.events || []) source.addEventListener(name, take(name));
    };
    open();

    const handle = {
      state,
      messages,
      error: failure,
      /// The last thing that arrived under a name.
      last: (name) => latest()[name == null ? "message" : name] ?? null,
      close() {
        if (source) source.close();
        state.set("closed");
      },
    };
    onCleanup(() => handle.close());
    return handle;
  }

  // ─── Every tab of this origin ────────────────────────
  //
  // `channel cart = broadcast("cart")`: what one tab posts, the others
  // hear — which is how a store stays in step across tabs.
  function broadcast(name, options) {
    const opts = options || {};
    const messages = signal(null);
    let channel = null;
    if (typeof BroadcastChannel !== "undefined") {
      channel = new BroadcastChannel(name);
      channel.onmessage = (event) => {
        messages.set(event.data);
        if (opts.onMessage) opts.onMessage(event.data);
      };
    }
    if (opts.onOpen) opts.onOpen();
    {
    }
    const handle = {
      state: signal(channel ? "open" : "closed"),
      closure: signal(null),
      error: signal(null),
      last: messages,
      post(value) {
        if (channel) channel.postMessage(value);
      },
      close() {
        if (channel) channel.close();
        channel = null;
      },
    };
    onCleanup(() => handle.close());
    return handle;
  }
