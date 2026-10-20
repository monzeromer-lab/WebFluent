  // ─── Keeping a value across visits ───────────────────
  //
  // `persist` at the top of a page, a component or a store: the one
  // implementation, so what a store keeps and what a page keeps cannot
  // drift. A program that persists nothing carries none of this.

  // Where a value can be kept: the browser's two, and any the project
  // gives itself — `in: .app` reads `window.wfStorage.app`, which a script
  // of the project's defines (a desktop app's store, an extension's). One
  // is `{ get(name), set(name, text), remove?(name), watch?(name, fn) }`:
  // `get` may answer at once or with a promise; `watch` hears another
  // writer (another window) and hands back how to stop.
  function _browserArea(area) {
    return {
      get: (name) => area().getItem(name),
      set: (name, text) => area().setItem(name, text),
      remove: (name) => area().removeItem(name),
    };
  }
  const _local = _browserArea(() => window.localStorage);
  const _session = _browserArea(() => window.sessionStorage);
  let _warnedArea = null;
  function _storage(place) {
    if (!place || place === "local") return _local;
    if (place === "session") return _session;
    const given = typeof window !== "undefined" && window.wfStorage && window.wfStorage[place];
    if (given && typeof given.get === "function" && typeof given.set === "function") return given;
    if (_warnedArea !== place && devMode()) {
      _warnedArea = place;
      console.warn(`WF: \`persist … { in: .${place} }\` has no \`window.wfStorage.${place}\` with get and set; it is kept in localStorage until a script of the project gives one.`);
    }
    return _local;
  }

  /// `persist name = value`: a signal whose value is written where the
  /// policy says, read back on the next visit, brought forward from what
  /// an older build of the site left, and — unless it says otherwise —
  /// kept in step with the site's other tabs.
  ///
  /// The policy is `{ in, version, sync, migrate, key, off }`; with none of
  /// it, the value is kept in `localStorage` under `wf:<name>`, as it always
  /// was. `key` is a function, read as state: when what it reads changes —
  /// the project that is open — the value is the one kept under the new
  /// key, and writes go there. `off` is where the listeners' undo goes for
  /// a caller that manages its own lifetime — a store's; without one it
  /// goes to the enclosing scope, which is what a page wants.
  function persist(base, initial, policy) {
    const rules = policy || {};
    const version = rules.version || 0;
    const store = () => _storage(rules.in);
    const decode = (text) => (text === null || text === undefined ? initial : forward(JSON.parse(text), initial, version, rules.migrate));
    // No key yet — no project open — is nowhere: the value lives in
    // memory until there is one, and nothing is read or written.
    const nameFor = () => {
      if (typeof rules.key !== "function") return "wf:" + base;
      const part = rules.key();
      return part === null || part === undefined ? null : "wf:" + base + ":" + String(part);
    };
    const later = (fn) => {
      if (rules.off) rules.off.push(fn);
      else onCleanup(fn);
    };

    const s = signal(initial);
    const set = s.set;
    let name = null;
    // Which read is the latest: an answer that comes back after the key
    // moved on, or after the reader wrote, is not taken.
    let reading = 0;
    let unwatch = null;

    const load = (to) => {
      const ticket = ++reading;
      if (to === null) {
        set(initial);
        return;
      }
      let got;
      try {
        got = store().get(to);
      } catch (e) {
        got = null;
      }
      const take = (text) => {
        if (ticket !== reading) return;
        try {
          set(decode(text));
        } catch (e) {
          set(initial);
        }
      };
      if (got && typeof got.then === "function") {
        // Until it answers, the value is what it was declared as.
        set(initial);
        got.then(take, () => take(null));
      } else {
        take(got);
      }
    };

    const watchName = (to) => {
      if (unwatch) { unwatch(); unwatch = null; }
      if (rules.sync === false || to === null) return;
      const area = store();
      // Another writer — another tab, another window — wrote it: take
      // what it wrote, without writing it back.
      const heard = (text) => {
        reading++;
        try {
          set(decode(text));
        } catch (err) { /* written in a shape this build cannot read */ }
      };
      if (area === _local && typeof window.addEventListener === "function") {
        const listen = (e) => {
          if (e.key === to) heard(e.newValue);
        };
        window.addEventListener("storage", listen);
        unwatch = () => window.removeEventListener("storage", listen);
      } else if (area !== _session && typeof area.watch === "function") {
        const undo = area.watch(to, heard);
        unwatch = typeof undo === "function" ? undo : null;
      }
    };

    const moveTo = (to) => {
      if (to === name) return;
      name = to;
      load(to);
      watchName(to);
    };

    const follow = () => {
      // The key is state: follow it. The effect reads only the key. A
      // store's outlives the page that first read it, so it belongs to no
      // page's scope; its undo goes with the store's.
      const scope = currentScope;
      if (rules.off) currentScope = null;
      let watcher;
      try {
        watcher = effect(() => {
          const to = nameFor();
          // Untracked: what loading reads is not the key's.
          const was = currentEffect;
          currentEffect = null;
          try { moveTo(to); } finally { currentEffect = was; }
        });
      } finally {
        currentScope = scope;
      }
      if (rules.off && watcher && watcher.dispose) rules.off.push(watcher.dispose);
    };
    if (typeof rules.key !== "function") moveTo(nameFor());
    // A store's key may read a state declared after this one: the store
    // starts following once every state exists.
    else if (Array.isArray(rules.start)) rules.start.push(follow);
    else follow();
    later(() => { if (unwatch) unwatch(); });

    s.set = (v) => {
      reading++;
      set(v);
      if (name === null) return;
      try {
        store().set(name, JSON.stringify(version ? { "wf:v": version, "wf:d": s() } : s()));
      } catch (e) { /* full or blocked */ }
    };
    s.update = (fn) => s.set(fn(s()));
    return s;
  }

  /// What was stored, brought forward to the version this build declares.
  ///
  /// A value written before the declaration had a version is version 1's;
  /// one written by a newer build is left alone and the initial value used,
  /// since this build cannot know what it means. A missing step is the same
  /// answer, for the same reason.
  function forward(stored, initial, version, migrate) {
    if (!version) return stored;
    const envelope = stored && typeof stored === "object" && "wf:v" in stored;
    let at = envelope ? stored["wf:v"] : 1;
    let value = envelope ? stored["wf:d"] : stored;
    if (at > version) return initial;
    while (at < version) {
      const step = migrate && migrate[at + 1];
      if (!step) return initial;
      value = step(value);
      at += 1;
    }
    return value;
  }
