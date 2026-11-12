  // ─── The browser as values ───────────────────────────
  // `viewport` is the window's size with the breakpoints as booleans
  // (`viewport.md` from 768px up); `query` the URL's search parameters as
  // a map; `hash` the fragment without its `#`. Each is a signal read once
  // and kept current, so a condition on one follows the browser.
  let _viewport = null;
  // The breakpoints are the theme's: the stylesheet writes them onto the
  // root as custom properties, so what JavaScript calls `md` and what a
  // media query calls `md` are the same number, and a design that moves a
  // breakpoint moves both.
  function breakpointWidths() {
    const out = { sm: 640, md: 768, lg: 1024, xl: 1280 };
    try {
      if (typeof getComputedStyle !== "function" || !document.documentElement) return out;
      const root = getComputedStyle(document.documentElement);
      if (!root || typeof root.getPropertyValue !== "function") return out;
      for (const step of Object.keys(out)) {
        const width = parseFloat(String(root.getPropertyValue("--screen-" + step) || "").trim());
        if (!Number.isNaN(width)) out[step] = width;
      }
    } catch (e) {
      // No computed style to read: the baseline steps stand.
    }
    return out;
  }

  /// `viewport.width`, `.height`, and a flag per breakpoint.
  ///
  /// A breakpoint follows `matchMedia`, which fires once when the answer
  /// changes — where a `resize` listener fired on every pixel of a drag and
  /// re-rendered the page each time.
  function viewport() {
    if (!_viewport) {
      const widths = breakpointWidths();
      // The width is the answer; `matchMedia` is only how we hear that it
      // changed. A page that asks `viewport.md` and a rule that asks
      // `@media (min-width: …)` then agree, whatever a host's `matchMedia`
      // says.
      const read = () => {
        const held = {
          width: window.innerWidth || 0,
          height: window.innerHeight || 0,
        };
        for (const [step, at] of Object.entries(widths)) {
          held[step] = held.width >= at;
        }
        return held;
      };
      if (typeof window.matchMedia === "function") {
        for (const at of Object.values(widths)) {
          const query = window.matchMedia(`(min-width: ${at}px)`);
          const listen = query && query.addEventListener
            ? (fn) => query.addEventListener("change", fn)
            : query && query.addListener
              ? (fn) => query.addListener(fn)
              : null;
          if (listen) listen(() => _viewport.set(read()));
        }
      }
      _viewport = signal(read());
      // The width and the height still need the window; a size that only
      // crosses no breakpoint changes those and nothing else.
      window.addEventListener("resize", () => {
        const held = _viewport();
        const now = read();
        if (held.width !== now.width || held.height !== now.height) _viewport.set(now);
      });
    }
    return _viewport();
  }
  let _query = null;
  let _hash = null;
  function _readQuery() {
    const out = {};
    for (const [k, v] of new URLSearchParams(window.location.search)) out[k] = v;
    return out;
  }
  function _watchLocation() {
    const refresh = () => {
      if (_query) _query.set(_readQuery());
      if (_hash) _hash.set(window.location.hash.replace(/^#/, ""));
    };
    window.addEventListener("popstate", refresh);
    window.addEventListener("hashchange", refresh);
    const h = window.history;
    const push = h.pushState.bind(h);
    h.pushState = (...args) => { push(...args); refresh(); };
    const replace = h.replaceState.bind(h);
    h.replaceState = (...args) => { replace(...args); refresh(); };
  }
  // ─── The network ─────────────────────────────────────
  //
  // `network.online`, `.effectiveType`, `.saveData`, `.downlink` — a live
  // value, so a page can say what it does on a slow line or none at all.
  let _network = null;
  function network() {
    if (!_network) {
      const read = () => {
        const c = (typeof navigator !== "undefined" && (navigator.connection || navigator.mozConnection)) || {};
        return {
          online: typeof navigator === "undefined" ? true : navigator.onLine !== false,
          effectiveType: c.effectiveType || "4g",
          saveData: !!c.saveData,
          downlink: typeof c.downlink === "number" ? c.downlink : 10,
        };
      };
      _network = signal(read());
      const update = () => _network.set(read());
      if (typeof window !== "undefined" && window.addEventListener) {
        window.addEventListener("online", update);
        window.addEventListener("offline", update);
      }
      const c = (typeof navigator !== "undefined" && navigator.connection) || null;
      if (c && c.addEventListener) c.addEventListener("change", update);
    }
    // `queued`: the writes kept for when the connection returns, which the
    // offline module counts when the config asks for `sync`.
    return Object.assign({}, _network(), {
      get queued() { return offlineQueued ? offlineQueued() : 0; },
    });
  }

  // ─── The moment ──────────────────────────────────────
  //
  // `now` is a `DateTime` that keeps itself current: every minute by
  // default, or as often as a page asks — `now(every: 1.seconds)`. The
  // cadence is the fastest one that something still reading it asked for:
  // each reader is an effect, remembered with what its last run asked, and
  // one that was disposed of or no longer reads `now` is forgotten at the
  // next tick. With no reader left the timer stops; a later read starts it.
  let _now = null;
  let _nowEvery = null;
  let _nowTimer = null;
  const _nowReaders = new Map();
  const _nowValue = () => new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
  function _nowArm() {
    const listening = _now._subs;
    let want = null;
    for (const [reader, asked] of _nowReaders) {
      if (!listening.has(reader)) { _nowReaders.delete(reader); continue; }
      if (want === null || asked.every < want) want = asked.every;
    }
    if (want === _nowEvery && (_nowTimer !== null || want === null)) return;
    if (_nowTimer !== null) clearInterval(_nowTimer);
    _nowTimer = null;
    _nowEvery = want;
    if (want !== null && typeof setInterval !== "undefined") {
      _nowTimer = setInterval(() => {
        _now.set(_nowValue());
        _nowArm();
      }, want);
    }
  }
  function now(options) {
    const every = options && options.every ? Number(options.every) : 60000;
    if (!_now) _now = signal(_nowValue());
    const value = _now();
    const reader = currentEffect;
    if (reader) {
      // A run's reads replace the last run's: `deps` is new each run.
      const asked = _nowReaders.get(reader);
      if (!asked || asked.run !== reader.deps) _nowReaders.set(reader, { run: reader.deps, every });
      else if (every < asked.every) asked.every = every;
      _nowArm();
    }
    return value;
  }

  function query() {
    if (!_query) {
      const first = !_hash;
      _query = signal(_readQuery());
      if (first) _watchLocation();
    }
    return _query();
  }
  function hash() {
    if (!_hash) {
      const first = !_query;
      _hash = signal(window.location.hash.replace(/^#/, ""));
      if (first) _watchLocation();
    }
    return _hash();
  }

  // Mark `el` as the current page's link while the route matches `href`:
  // `.active` for the stylesheet and `aria-current="page"` for assistive
  // technology, so the two cannot disagree. With `prefix`, a link to a section
  // root also matches the routes beneath it.
  // The route part of a `to`: what is before its `?query` or `#hash`.
