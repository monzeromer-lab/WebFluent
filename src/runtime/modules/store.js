  // ─── Store ───────────────────────────────────────────
  //
  // A store is **built on first read**. Nothing is constructed at boot, so
  // the order stores are declared in cannot matter — a `derived` that
  // reads a store declared further down the file used to read
  // `undefined` — and a store nothing reads costs nothing, in time or in
  // bytes.
  //
  // What a call site holds is a proxy over the instance. Every read and
  // write goes through it, so `Cart.items` builds the store the first time
  // anything asks and is the plain property afterwards.

  /// Every `.route` store, by the function that drops what it built.
  const routeScoped = new Set();

  /// What the devtools watch, when something installs it. Nothing is
  /// recorded while it is null, so a production build pays nothing for it.
  let storeWatcher = null;
  function watchStores(fn) {
    storeWatcher = fn;
    return () => { storeWatcher = null; };
  }

  /// Every store that has been built, by name — what the devtools list,
  /// snapshot and restore.
  const builtStores = new Map();

  function store(name, define, options) {
    const opts = options || {};
    let instance = null;
    const build = () => instance || (instance = assembleStore(name, define(), opts));
    if (opts.scope === "route") {
      routeScoped.add(() => {
        if (instance) disposeStore(name, instance);
        instance = null;
      });
    }
    const proxy = new Proxy(Object.create(null), {
      get: (_, key) => build()[key],
      set: (_, key, value) => { build()[key] = value; return true; },
      has: (_, key) => key in build(),
      deleteProperty: (_, key) => delete build()[key],
      ownKeys: () => Reflect.ownKeys(build()),
      getOwnPropertyDescriptor: (_, key) => {
        const d = Object.getOwnPropertyDescriptor(build(), key);
        return d && { ...d, configurable: true };
      },
    });
    // `store X(eager: true)`: built at boot, for the rare store whose
    // set-up has to happen whether or not a page reads it.
    if (opts.eager) build();
    return proxy;
  }

  /// Drop every `.route` store. The router calls this when the route
  /// changes: the next read of one builds it again, empty.
  function dropRouteStores() {
    for (const drop of routeScoped) drop();
  }

  function disposeStore(name, instance) {
    for (const off of instance.__off) off();
    builtStores.delete(name);
  }

  // ─── Building one ────────────────────────────────────

  function assembleStore(name, definition, opts) {
    const store = {};
    const states = {};
    // Everything to undo when a `.route` store's route leaves.
    const off = [];
    Object.defineProperty(store, "__off", { value: off, enumerable: false });

    const kept = definition.persist || {};
    if (definition.state) {
      for (const [key, val] of Object.entries(definition.state)) {
        const initial = typeof val === "function" ? val() : val;
        const policy = kept[key];
        // A store that is this tab's own leaves nothing behind after it,
        // so what it keeps goes in the tab's storage unless it says
        // otherwise.
        const s = policy
          ? persist(name + "." + key, initial, {
              ...policy,
              in: policy.in || (opts.scope === "session" ? "session" : "local"),
              off,
            })
          : signal(initial);
        states[key] = s;
        if (__regHook) __regHook(key, s); // expose for WF.__debug.state()
        Object.defineProperty(store, key, {
          get: () => s(),
          set: (v) => s.set(v),
          enumerable: true,
        });
      }
    }

    // Bind actions before the derived values: a computed runs as soon as it
    // is created, and a derived value that calls one of the store's own
    // actions used to find it missing.
    if (definition.actions) {
      for (const [key, fn] of Object.entries(definition.actions)) {
        const report = (args, before) => {
          if (storeWatcher) {
            storeWatcher({ store: name, action: key, args, before, after: snap(states) });
          }
        };
        // An async action carries `pending`: true while a call runs.
        if (fn.constructor && fn.constructor.name === "AsyncFunction") {
          const pending = signal(false);
          store[key] = async (...args) => {
            const before = storeWatcher ? snap(states) : null;
            pending.set(true);
            try { return await fn(store, ...args); } finally {
              pending.set(false);
              report(args, before);
            }
          };
          store[key].pending = pending;
        } else {
          store[key] = (...args) => {
            const before = storeWatcher ? snap(states) : null;
            try { return fn(store, ...args); } finally { report(args, before); }
          };
        }
      }
    }

    // Create computed for derived
    if (definition.derived) {
      for (const [key, fn] of Object.entries(definition.derived)) {
        const c = computed(() => fn(store));
        Object.defineProperty(store, key, { get: () => c(), enumerable: true });
      }
    }

    builtStores.set(name, { states, store });
    if (storeWatcher) storeWatcher({ store: name, action: null, args: [], before: null, after: snap(states) });
    return store;
  }

  /// What a store's state holds, as plain values.
  function snap(states) {
    const out = {};
    for (const [key, s] of Object.entries(states)) out[key] = s();
    return out;
  }

  // ─── Looking at them ─────────────────────────────────

  /// What every store that has been built is holding — the devtools' tree,
  /// and what time travel puts back.
  function storeSnapshot(name) {
    if (name) {
      const held = builtStores.get(name);
      return held ? snap(held.states) : null;
    }
    const out = {};
    for (const [key, held] of builtStores) out[key] = snap(held.states);
    return out;
  }

  /// Put a store back to a snapshot — one step of time travel. A name it
  /// has not built is ignored: a store nothing has read has no state to
  /// travel to.
  function restoreStore(name, values) {
    const held = builtStores.get(name);
    if (!held) return false;
    for (const [key, value] of Object.entries(values || {})) {
      if (held.states[key]) held.states[key].set(value);
    }
    return true;
  }
