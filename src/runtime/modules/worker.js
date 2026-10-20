  // ─── A Web Worker ───────────────────────────────────────────────────
  //
  // `worker css = spawn("/workers/css.js") { on message(m) { … } }`: a
  // script of the project's running beside the page, held like a socket —
  // owned by the scope that opened it, and stopped when it leaves.
  //
  // `css.request(method, params)` sends `{ id, method, params }` and is a
  // promise of the answer the worker posts back as `{ id, result }`, or the
  // failure it posts as `{ id, error }`. A request not answered in its
  // `timeout` fails; with `supersede: true`, a newer request of the same
  // method fails the older one, and the worker is told `{ cancel: id }`.
  // `css.send(value)` posts and expects nothing. What the worker posts
  // with no `id` — diagnostics, progress — is a message: `on message`
  // hears it, and `css.last(kind)` is the last of a kind (its `type`, or
  // its `method`). `css.pending` is how many requests are out.
  //
  // A worker that crashes (an error nothing in it caught) is started again
  // `restart` times, its requests failed; past that it is `error`.

  const REQUEST_MS = 10000;

  function worker(url, options) {
    const opts = options || {};
    const state = signal("open");
    const failure = signal(null);
    // How it stopped, as a socket's close says: a `match`'s `closed(c)`.
    const closure = signal(null);
    const pending = signal(0);
    const latest = signal(new Map());
    const waiting = new Map();
    let next = 1;
    let restarts = opts.restart === undefined ? 0 : Number(opts.restart) || 0;
    let running = null;
    let alive = true;

    const settle = (id, ok, value) => {
      const call = waiting.get(id);
      if (!call) return;
      waiting.delete(id);
      clearTimeout(call.timer);
      pending.set(waiting.size);
      if (ok) call.resolve(value);
      else call.reject(value);
    };
    const failAll = (why) => {
      for (const id of [...waiting.keys()]) settle(id, false, why);
    };

    const start = () => {
      // The worker's script is a file of the site: under its base path.
      const given = String(typeof url === "function" ? url() : url);
      const address = given.startsWith("/") && !given.startsWith("//") ? _basePath + given : given;
      try {
        running = new Worker(address, opts.type === "module" ? { type: "module" } : undefined);
      } catch (e) {
        failure.set(e);
        state.set("error");
        return;
      }
      state.set("open");
      running.onmessage = (event) => {
        const data = event.data;
        if (data && typeof data === "object" && data.id !== undefined && waiting.has(data.id)) {
          if (data.error !== undefined) {
            const err = data.error;
            settle(data.id, false, err instanceof Error ? err : Object.assign(new Error(err && err.message ? err.message : String(err)), { data: err }));
          } else {
            settle(data.id, true, data.result);
          }
          return;
        }
        const kind = data && typeof data === "object" ? (data.type || data.method) : null;
        if (kind) latest.update((held) => new Map(held).set(kind, data));
        latest.update((held) => new Map(held).set(null, data));
        if (opts.onMessage) opts.onMessage(data);
      };
      running.onerror = (event) => {
        if (event && event.preventDefault) event.preventDefault();
        const why = new Error((event && event.message) || "The worker stopped");
        failure.set(why);
        if (running) running.terminate();
        running = null;
        failAll(why);
        if (alive && restarts > 0) {
          restarts -= 1;
          start();
          if (opts.onRestart) opts.onRestart(why);
          return;
        }
        state.set("error");
        if (opts.onError) opts.onError(why);
      };
      running.onmessageerror = () => {
        failure.set(new Error("A message from the worker could not be read"));
      };
    };
    start();

    const post = (value) => {
      if (!running) throw new Error(state() === "closed" ? "The worker was stopped" : "The worker is not running");
      running.postMessage(value);
    };

    const handle = {
      state,
      error: failure,
      closure,
      pending,
      /// Ask the worker; a promise of its answer.
      request(method, params, how) {
        const rules = how || {};
        if (rules.supersede) {
          for (const [id, call] of [...waiting]) {
            if (call.method !== method) continue;
            settle(id, false, Object.assign(new Error("Superseded by a newer request"), { superseded: true }));
            try { post({ cancel: id }); } catch (e) { /* it is down */ }
          }
        }
        const id = next++;
        return new Promise((resolve, reject) => {
          const ms = rules.timeout || opts.timeout || REQUEST_MS;
          const timer = setTimeout(() => settle(id, false, Object.assign(new Error(`The worker did not answer \`${method}\` in ${ms} ms`), { timeout: true })), ms);
          waiting.set(id, { resolve, reject, timer, method });
          pending.set(waiting.size);
          try {
            post({ id, method, params: params === undefined ? null : params });
          } catch (e) {
            settle(id, false, e);
          }
        });
      },
      /// Post a value, expecting nothing back.
      send(value) {
        post(value);
      },
      /// The last message the worker posted, or the last of a kind.
      last(kind) {
        const held = latest();
        return held.has(kind === undefined ? null : kind) ? held.get(kind === undefined ? null : kind) : null;
      },
      /// Start it again: what was asked of the old one fails.
      restart() {
        if (running) running.terminate();
        running = null;
        failAll(new Error("The worker was restarted"));
        if (alive) start();
      },
      close() {
        alive = false;
        if (running) running.terminate();
        running = null;
        failAll(new Error("The worker was stopped"));
        closure.set({ code: 1000, reason: "stopped" });
        state.set("closed");
      },
    };
    // The scope that opened it stops it.
    onCleanup(() => handle.close());
    return handle;
  }
