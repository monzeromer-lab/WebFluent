  // ─── Keeping a value across visits ───────────────────
  //
  // `persist` at the top of a page, a component or a store: the one
  // implementation, so what a store keeps and what a page keeps cannot
  // drift. A program that persists nothing carries none of this.

  /// `persist name = value`: a signal whose value is written where the
  /// policy says, read back on the next visit, brought forward from what
  /// an older build of the site left, and — unless it says otherwise —
  /// kept in step with the site's other tabs.
  ///
  /// The policy is `{ in, version, sync, migrate, off }`; with none of it,
  /// the value is kept in `localStorage` under `wf:<key>`, as it always
  /// was. `off` is where the storage listener's undo goes for a caller
  /// that manages its own lifetime — a store's; without one it goes to the
  /// enclosing scope, which is what a page wants.
  function persist(key, initial, policy) {
    const rules = policy || {};
    const name = "wf:" + key;
    const area = () => (rules.in === "session" ? window.sessionStorage : window.localStorage);
    const version = rules.version || 0;

    let value = initial;
    try {
      const stored = area().getItem(name);
      if (stored !== null) value = forward(JSON.parse(stored), initial, version, rules.migrate);
    } catch (e) { /* storage blocked or unreadable: the initial value */ }

    const s = signal(value);
    const set = s.set;
    s.set = (v) => {
      set(v);
      try {
        area().setItem(name, JSON.stringify(version ? { "wf:v": version, "wf:d": s() } : s()));
      } catch (e) { /* full or blocked */ }
    };
    s.update = (fn) => s.set(fn(s()));

    // Another tab wrote it: take what it wrote, without writing it back.
    // `sync: false` on the declaration is how a value stays this tab's own.
    if (rules.sync !== false && rules.in !== "session" && typeof window.addEventListener === "function") {
      const listen = (e) => {
        if (e.key !== name) return;
        try {
          set(e.newValue === null ? initial : forward(JSON.parse(e.newValue), initial, version, rules.migrate));
        } catch (err) { /* another tab wrote something unreadable */ }
      };
      window.addEventListener("storage", listen);
      const undo = () => window.removeEventListener("storage", listen);
      if (rules.off) rules.off.push(undo);
      else onCleanup(undo);
    }
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
