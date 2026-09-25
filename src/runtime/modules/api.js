  // ─── A service, described once ───────────────────────
  //
  // An `api` declaration compiles to one call of this: the settings the
  // service is reached with, and the endpoints it has. What comes back is
  // an object of callable endpoints — `Backend.users(page: 2)` — each of
  // which also carries `.invalidate()`, `.prefetch(args)`, `.progress` and
  // `.key(args)`, so a page can say what it wants without writing a client.
  function api(config) {
    const client = {};
    const base = () => {
      const b = typeof config.base === "function" ? config.base() : config.base;
      return (b || "").replace(/\/+$/, "");
    };
    for (const [name, endpoint] of Object.entries(config.endpoints || {})) {
      client[name] = makeEndpoint(config, base, name, endpoint);
    }
    // The whole service's cache, for a mutation that changes everything.
    client.invalidate = () => invalidate(config.name ? config.name + " " : null);
    return client;
  }

  /// The path with its `:name` parts filled in, and what is left over as
  /// the query string.
  function fillPath(path, args) {
    const used = new Set();
    const filled = String(path).replace(/:([A-Za-z_][A-Za-z0-9_]*)/g, (whole, name) => {
      if (!(name in args)) return whole;
      used.add(name);
      return encodeURIComponent(String(args[name]));
    });
    return { path: filled, used };
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

  function makeEndpoint(config, base, name, endpoint) {
    const progress = signal(0);
    const method = (endpoint.method || "GET").toUpperCase();
    // What is a parameter of the call rather than of the request.
    const perCall = new Set(["body", "file", "cache", "on", "timeout", "signal", "as"]);

    const addressOf = (args) => {
      const given = args || {};
      const { path, used } = fillPath(endpoint.path, given);
      const url = /^[a-z]+:\/\//i.test(path) ? path : base() + "/" + path.replace(/^\/+/, "");
      return url + queryString(given, used, perCall);
    };

    const optionsOf = (args) => {
      const given = args || {};
      let body = given.body;
      // A file is sent as a form, which is what a server expects of one.
      if (given.file != null && typeof FormData !== "undefined") {
        const form = new FormData();
        form.append(endpoint.fileField || "file", given.file);
        for (const [k, v] of Object.entries(body || {})) form.append(k, v);
        body = form;
      }
      return {
        method,
        body,
        headers: { ...(config.headers || {}), ...(given.headers || {}) },
        as: given.as || endpoint.as || "json",
        errorAs: endpoint.errorAs || "json",
        timeout: given.timeout != null ? given.timeout : config.timeout,
        retry: given.retry || config.retry,
        cache: given.cache || endpoint.cache || (method === "GET" ? config.cache : { kind: "none" }),
        credentials: config.credentials,
        mode: config.mode,
        redirect: config.redirect,
        referrerPolicy: config.referrerPolicy,
        signal: given.signal,
        withHeaders: endpoint.withHeaders,
        progress: given.file != null || endpoint.progress ? progress : null,
        onRequest: config.onRequest,
        onResponse: config.onResponse,
        onError: config.onError,
        key: (config.name || "") + " " + name + " " + addressOf(given),
      };
    };

    const call = (args) => send(addressOf(args), optionsOf(args));
    call.progress = progress;
    call.key = (args) => optionsOf(args).key;
    call.url = addressOf;
    /// Forget what this endpoint cached — every argument of it.
    call.invalidate = (args) =>
      invalidate(args ? optionsOf(args).key : (config.name || "") + " " + name + " ");
    /// Ask now for what will be wanted soon.
    call.prefetch = (args) => send(addressOf(args), optionsOf(args)).catch(() => null);
    /// Every line of a streamed response, as it arrives.
    call.lines = (args) => lines(addressOf(args), optionsOf(args));
    return call;
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
    let socket = null;
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
        messages.update((held) => [...held, data]);
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
        setTimeout(open, wait * (0.5 + Math.random() / 2));
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
        alive = false;
        if (heart) clearInterval(heart);
        if (socket) socket.close(code || 1000, reason || "");
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
        messages.update((held) => [...held, message]);
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
