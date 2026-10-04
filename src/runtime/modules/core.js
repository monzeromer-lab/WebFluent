  // ─── Reactivity ──────────────────────────────────────
  let currentEffect = null;

  function signal(value) {
    const subs = new Set();
    const get = () => {
      if (currentEffect) subs.add(currentEffect);
      return value;
    };
    // A signal holds any value, a function included: `set(fn)` stores
    // `fn`. It used to call it as an updater, so a state holding a lambda
    // held what the lambda returned. `update(fn)` is the updater.
    const set = (v) => {
      if (v !== value) {
        value = v;
        for (const fn of [...subs]) fn();
      }
    };
    get.set = set;
    get.update = (fn) => set(fn(value));
    get.subscribe = (fn) => { subs.add(fn); return () => subs.delete(fn); };
    return get;
  }

  // Whether the page is served by `wf serve`, whose dev script says so
  // before the bundle runs: what only a developer should see — a list item
  // that threw, a route nothing answers, a response of the wrong shape — is
  // shown on the page there, and only logged anywhere else.
  function devMode() {
    return typeof window !== "undefined" && window.__WF_DEV__ === true;
  }

  // ─── Ownership ───────────────────────────────────────
  // What a page, a branch, a list item or a slot creates — effects, timers,
  // listeners — belongs to the scope it was created in, and is disposed of
  // when that scope's nodes leave. `scoped(fn)` runs `fn` in a fresh scope
  // and returns its result with the scope's disposer; `onCleanup(fn)` adds
  // to the scope at hand.
  let currentScope = null;

  function onCleanup(fn) {
    if (currentScope) currentScope.push(fn);
    return fn;
  }

  function scoped(fn) {
    const prev = currentScope;
    const scope = [];
    currentScope = scope;
    try {
      return [fn(), () => { for (const f of scope.splice(0)) f(); }];
    } finally {
      currentScope = prev;
    }
  }

  // An effect runs at once and again when a signal it read changes; what
  // it returns is its cleanup, run before the next run and on disposal.
  //
  // An effect that writes what it reads runs itself again from inside its
  // own run. One that settles does so once or twice; one that never does
  // used to recurse until the stack overflowed and the page died. Past
  // `MAX_REENTRY` nested runs it stops, and under `wf serve` says so.
  const MAX_REENTRY = 100;
  function effect(fn) {
    let cleanup = null;
    let dead = false;
    let depth = 0;
    let warned = false;
    const run = () => {
      if (dead) return;
      if (depth >= MAX_REENTRY) {
        if (!warned && devMode()) {
          warned = true;
          console.warn("WF: an effect changes something it reads, every time it runs, so it never settles; it was stopped after " + MAX_REENTRY + " runs. Guard the write, or move it to an action.");
        }
        return;
      }
      if (typeof cleanup === "function") { const c = cleanup; cleanup = null; c(); }
      const prev = currentEffect;
      currentEffect = run;
      depth++;
      try { cleanup = fn(); } finally { currentEffect = prev; depth--; }
    };
    run.dispose = () => {
      dead = true;
      if (typeof cleanup === "function") { const c = cleanup; cleanup = null; c(); }
    };
    onCleanup(run.dispose);
    run();
    return run;
  }

  // `every(ms) { }` and `after(ms) { }`: timers that stop with their scope.
  function every(ms, fn) {
    const id = setInterval(fn, ms);
    onCleanup(() => clearInterval(id));
    return id;
  }
  function after(ms, fn) {
    const id = setTimeout(fn, ms);
    onCleanup(() => clearTimeout(id));
    return id;
  }

  // A listener that leaves with its scope.
  function listen(target, event, fn, options) {
    target.addEventListener(event, fn, options);
    onCleanup(() => target.removeEventListener(event, fn, options));
  }

  // `on key("ctrl+k") { }`: a keydown whose key and modifiers match the
  // spelling — `ctrl`, `shift`, `alt`, `meta`/`cmd`, then the key name as
  // `KeyboardEvent.key` spells it (`k`, `Enter`, `Escape`, `ArrowDown`).
  function keyIs(e, spelling) {
    const parts = String(spelling).toLowerCase().split("+").map((p) => p.trim());
    const key = parts.pop();
    const names = { esc: "escape", return: "enter", space: " ", up: "arrowup", down: "arrowdown", left: "arrowleft", right: "arrowright", plus: "+" };
    const wanted = names[key] || key;
    if ((e.key || "").toLowerCase() !== wanted) return false;
    const want = { ctrl: parts.includes("ctrl"), shift: parts.includes("shift"), alt: parts.includes("alt"), meta: parts.includes("meta") || parts.includes("cmd") };
    return !!e.ctrlKey === want.ctrl && !!e.shiftKey === want.shift && !!e.altKey === want.alt && !!e.metaKey === want.meta;
  }
  // On the page, a key with no modifier — `n`, `/`, `?` — is the reader
  // typing when the focus is in a field, not a shortcut; one with `ctrl`,
  // `alt` or `meta`, and `Escape`, still is. On an element it is that
  // element's, and always answers.
  function _typing(e) {
    const t = e.target;
    if (!t || t.nodeType !== 1) return false;
    return t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName);
  }
  function onKey(target, spelling, fn) {
    const plain = !/(^|\+)(ctrl|control|alt|option|meta|cmd|command)\+/i.test(spelling)
      && !/^(escape|esc)$/i.test(spelling.split("+").pop());
    const page = target === document || target === window;
    listen(target, "keydown", (e) => {
      if (page && plain && _typing(e)) return;
      if (keyIs(e, spelling)) fn(e);
    });
  }

  // `ref: name`: a handle on an element, usable as the element itself —
  // `name.focus()`, `name.value` — once it is drawn.
  function ref() {
    const box = { el: null };
    return new Proxy(box, {
      get(t, k) {
        if (k === "current" || k === "el") return t.el;
        const v = t.el ? t.el[k] : undefined;
        return typeof v === "function" ? v.bind(t.el) : v;
      },
      set(t, k, v) {
        if (k === "current" || k === "el") { t.el = v; return true; }
        if (t.el) { t.el[k] = v; return true; }
        return false;
      },
    });
  }

  function computed(fn) {
    const s = signal(undefined);
    effect(() => s.set(fn()));
    return s;
  }

  /// An attribute holding JSON, as the prop it stands for.
  ///
  /// A `Map` or a list prop of a published custom element arrives as text —
  /// an attribute is always text — so the element reads it back. Text that
  /// is not JSON is left as text, which is what a page that wrote a plain
  /// string meant.
  function jsonAttr(raw) {
    if (raw == null) return undefined;
    try {
      return JSON.parse(raw);
    } catch (e) {
      return raw;
    }
  }

  // ─── Where a browser may be pointed ──────────────────
  //
  // The twin of `codegen::url`: a literal is checked where it is written,
  // and a value that only exists at run time is checked here. The rule is
  // an allow-list of schemes, so a scheme nobody has thought of is refused
  // by default.
  const WF_SCHEMES = ["http", "https", "mailto", "tel", "sms", "ftp"];

  /// `url` where a browser may follow it, and "" where it may not.
  ///
  /// The control characters and whitespace a browser ignores come out
  /// first, so `java\tscript:` and ` JavaScript:` are `javascript` here
  /// too.
  function safeUrl(url) {
    if (url == null) return "";
    const text = String(url);
    const cleaned = text.replace(/[\u0000-\u0020\u007f-\u009f\s]/g, "");
    const at = cleaned.indexOf(":");
    if (at < 0) return text;
    const scheme = cleaned.slice(0, at);
    if (!/^[a-zA-Z][a-zA-Z0-9+.-]*$/.test(scheme)) return text;
    return WF_SCHEMES.includes(scheme.toLowerCase()) ? text : "";
  }

  // ─── DOM Helpers ─────────────────────────────────────
  //
  // An attribute whose meaning belongs to a feature — `markdown`, the
  // `highlight` pair, `data-icon` — is handled by a hook that feature
  // registers, so a build that uses none of them carries none of them.
  // A late hook runs after the children, where a drawn glyph belongs.
  const attrHooks = {};
  const lateHooks = {};
  function attrHook(name, fn) { attrHooks[name] = fn; }
  function lateHook(name, fn) { lateHooks[name] = fn; }
  // The studio's signal registry, set by the debug module when it is built in.
  let __regHook = null;
  // Set by the offline module when the config asks for `sync`: what the
  // request engine hands a write that failed for want of a network, and how
  // many such writes wait. Empty in a build without it.
  let offlineWrites = null;
  let offlineQueued = null;
  // The project's translations, set by `locales()` when i18n is built in.
  // `format` and a field's labels read it without depending on i18n.
  let i18nInstance = null;

  function el(tag, attrs, ...children) {
    const el = document.createElement(tag);
    // A select's value only takes once its options exist, so it is applied
    // after the children; set before them it was silently ignored.
    let selectValue;
    let iconName;
    if (attrs) {
      for (const [k, v] of Object.entries(attrs)) {
        if (k === "ref" && v && typeof v === "object") {
          v.current = el;
        } else if (attrHooks[k]) {
          // An attribute a feature module owns — `markdown`, `highlight`.
          // `el` knows the name only because the module registered it.
          attrHooks[k](el, v);
        } else if (k === "value" && tag === "select") {
          selectValue = v;
        } else if (k.startsWith("on:")) {
          el.addEventListener(k.slice(3), v);
        } else if (k === "className" || k === "class") {
          if (typeof v === "function") {
            effect(() => { el.className = v(); });
          } else {
            el.className = v;
          }
        } else if (k === "style" && typeof v === "object") {
          Object.assign(el.style, v);
        } else if (k === "checked") {
          if (typeof v === "function") {
            effect(() => { el.checked = v(); });
          } else {
            el.checked = v;
          }
        } else if (k === "value") {
          // An option keeps the value it was given — a number, a case —
          // beside the string the DOM makes of it, so a bound select
          // writes back what the program wrote (`bound`, below).
          const put = tag === "option" ? (val) => { el.value = val; el._wfValue = val; } : (val) => { el.value = val; };
          if (typeof v === "function") {
            effect(() => put(v()));
          } else {
            put(v);
          }
        } else if (k === "disabled" || k === "multiple" || k === "required" || k === "readOnly") {
          if (typeof v === "function") {
            effect(() => { el[k] = !!v(); });
          } else {
            el[k] = !!v;
          }
        } else if (k === "min" || k === "max" || k === "step") {
          if (typeof v === "function") {
            effect(() => { el[k] = String(v()); });
          } else {
            el[k] = String(v);
          }
        } else if (k === "data-wf-delay" || k === "data-wf-duration" || k === "data-wf-easing") {
          // The element's own timing for the animation class it carries.
          el.setAttribute(k, v);
          el.style[TIMING[k]] = v;
        } else if (k === "data-icon") {
          // The glyph is drawn after the children: an icon button carries
          // data-icon and a .wf-icon child, and used to draw the glyph twice.
          iconName = v;
        } else if (k.startsWith("aria-")) {
          // An ARIA state is a string: aria-pressed="false" means "not pressed",
          // while a missing attribute means "not a toggle". So false is kept.
          const set = (val) => {
            if (val == null) el.removeAttribute(k);
            else el.setAttribute(k, String(val));
          };
          if (typeof v === "function") effect(() => set(v()));
          else set(v);
        } else if (typeof v === "function") {
          effect(() => {
            const val = v();
            if (val == null || val === false) el.removeAttribute(k);
            else el.setAttribute(k, val);
          });
        } else if (v != null && v !== false) {
          el.setAttribute(k, v);
        }
      }
    }
    appendChildren(el, children);
    if (iconName !== undefined && lateHooks["data-icon"]) lateHooks["data-icon"](el, iconName);
    // A select ignores a value it has no option for. The options may be
    // passed as children here, or appended by the statements that follow
    // this call; when the assignment does not take, try again once they are.
    const applySelectValue = (v) => {
      if (v == null) return;
      el.value = v;
      if (el.value !== String(v)) {
        queueMicrotask(() => {
          el.value = v;
        });
      }
    };
    if (typeof selectValue === "function") {
      effect(() => applySelectValue(selectValue()));
    } else {
      applySelectValue(selectValue);
    }
    if (tag === "img" && !el.hasAttribute("loading")) _imageDefaults(el);
    return el;
  }

  // What a bound control holds, typed as the program wrote it: a number
  // field's number (or `null` when it is empty), a select's option value
  // as the option was given it, anything else the string it reports. A
  // `.number` input used to store "3", so `qty + 1` was "31".
  function bound(t) {
    if (t.tagName === "SELECT") {
      const o = t.selectedOptions[0];
      return o ? ("_wfValue" in o ? o._wfValue : o.value) : null;
    }
    if (t.type === "number" || t.type === "range") {
      return t.value === "" || !Number.isFinite(t.valueAsNumber) ? null : t.valueAsNumber;
    }
    return t.value;
  }

  // ─── Images ──────────────────────────────────────────
  //
  // The first image a page creates is, near enough always, the one at the
  // top of it — the hero, the product shot — and the one Largest
  // Contentful Paint waits for. Lazy-loading it defers exactly the request
  // that matters; the rest of the page's images are what lazy loading is
  // for. The count restarts with every page the router draws.
  let imagesThisPage = 0;

  function _imageDefaults(el) {
    if (imagesThisPage === 0) {
      el.setAttribute("loading", "eager");
      el.setAttribute("fetchpriority", "high");
    } else {
      el.setAttribute("loading", "lazy");
    }
    imagesThisPage += 1;
  }

  function _newPage() {
    imagesThisPage = 0;
  }

  // A component's props: what the caller gave, with declared defaults for
  // whatever it left out. Reads go through getters, so a prop the caller
  // passed as a getter over state stays live inside the component.
  function props(given, defaults) {
    const out = {};
    const keys = new Set([...Object.keys(given || {}), ...Object.keys(defaults || {})]);
    for (const key of keys) {
      Object.defineProperty(out, key, {
        enumerable: true,
        get() {
          const v = given ? given[key] : undefined;
          return v === undefined ? defaults[key] : v;
        },
      });
    }
    return out;
  }

  // Listen on the root element of what a component returned: its fragment's
  // first element. A component that renders several roots gets the handler on
  // the first, which is where a caller expects it.
  function onRoot(frag, event, handler) {
    const isFragment = frag.nodeType === 11 || frag.tagName === "#DOCUMENT-FRAGMENT";
    const root = isFragment ? [...frag.childNodes].find((n) => n.nodeType === 1) : frag;
    if (root) root.addEventListener(event, handler);
  }

  // The style property each timing marker sets.
  const TIMING = {
    "data-wf-delay": "animationDelay",
    "data-wf-duration": "animationDuration",
    "data-wf-easing": "animationTimingFunction",
  };

  // Motion asked of a component at its call site lands on its root
  // element: `data-wf-exit`, `data-wf-delay`, `data-wf-duration`,
  // `data-wf-easing`.
  function mark(frag, attrs) {
    const isFragment = frag.nodeType === 11 || frag.tagName === "#DOCUMENT-FRAGMENT";
    const root = isFragment ? [...frag.childNodes].find((n) => n.nodeType === 1) : frag;
    if (!root) return;
    for (const [k, v] of Object.entries(attrs)) {
      root.setAttribute(k, v);
      if (TIMING[k]) root.style[TIMING[k]] = v;
      if (k === "data-wf-animate") root.classList.add("wf-animate-" + v);
    }
  }

  function appendChildren(el, children) {
    for (const child of children.flat(Infinity)) {
      if (child == null || child === false) continue;
      if (typeof child === "string" || typeof child === "number") {
        el.appendChild(document.createTextNode(String(child)));
      } else if (child instanceof Node) {
        el.appendChild(child);
      } else if (typeof child === "function") {
        reactiveText(el, child);
      }
    }
  }

  function reactiveText(parent, fn) {
    const node = document.createTextNode("");
    parent.appendChild(node);
    effect(() => { node.textContent = String(fn()); });
    return node;
  }

  function text(fn) {
    if (typeof fn === "function") {
      const node = document.createTextNode("");
      effect(() => { node.textContent = String(fn()); });
      return node;
    }
    return document.createTextNode(String(fn));
  }

  // ─── Enum cases ──────────────────────────────────────
  // A bare case is its name; a case with a payload is the name followed by
  // the payload, `["failed", "boom"]`. `caseOf` reads the name of either,
  // and `payload` the payload where the case is the one asked for: the one
  // value of a single payload, the list of a longer one, null otherwise.
  function caseOf(v) {
    return Array.isArray(v) ? v[0] : v;
  }
  function payload(v, name) {
    if (!Array.isArray(v) || v[0] !== name) return null;
    return v.length === 2 ? v[1] : v.slice(1);
  }

  // ─── Conditional rendering ───────────────────────────
  function removeNodes(nodes) {
    for (const n of nodes) {
      if (n && n.parentNode) n.parentNode.removeChild(n);
    }
  }

  // ─── Router ──────────────────────────────────────────
  let routerInstance = null;

  // ─── Router ──────────────────────────────────────────
  // Base path for deployment (set via WF.setBasePath or config)
  let _basePath = "";

  function _stripBase(fullPath) {
    if (_basePath && fullPath.startsWith(_basePath)) {
      const stripped = fullPath.slice(_basePath.length);
      return stripped || "/";
    }
    return fullPath;
  }

  // The current route, base path stripped, shared by the router and by every
  // link that wants to know whether it points at the page being shown. Created
  // on first use so a link built before the router (the app shell's navbar)
  // subscribes to the same signal the router later drives.
  let _pathSignal = null;
  function pathSignal() {
    if (!_pathSignal) _pathSignal = signal(_stripBase(window.location.pathname));
    return _pathSignal;
  }

  let _ssgMode = false;
  function setSsgMode(enabled) { _ssgMode = enabled; }
  function setBasePath(path) {
    _basePath = path.replace(/\/$/, "");
    // A link created before the base path was known compared against the
    // unstripped location; re-derive it now that stripping is possible.
    if (_pathSignal) _pathSignal.set(_stripBase(window.location.pathname));
  }

  /// Where the reader was sent: the element the address's `#fragment`
  /// names, scrolled into view. A pre-rendered page is replaced when its
  /// script takes over, and the browser's own jump to the fragment — made
  /// while the HTML was parsed — is lost with the nodes it jumped to; a
  /// route change scrolled to the top even when the link named a section.
  /// Returns whether there was somewhere to go.
  function landOnHash() {
    if (typeof window === "undefined" || typeof document === "undefined") return false;
    const hash = window.location && window.location.hash;
    if (!hash || hash.length < 2) return false;
    let id = hash.slice(1);
    try { id = decodeURIComponent(id); } catch (e) { /* as written */ }
    const target = document.getElementById && document.getElementById(id);
    if (!target || typeof target.scrollIntoView !== "function") return false;
    target.scrollIntoView();
    return true;
  }

  function navigate(path) {
    if (_ssgMode) {
      // SSG: full page load to the pre-rendered HTML file
      window.location.href = _basePath + path;
    } else if (routerInstance) {
      routerInstance.navigate(path);
    } else {
      window.location.href = path;
    }
  }

  // What a `class:` names: a string, `{ "is-on": cond }` (each key while its
  // value is true), or a list of either, null and `false` skipped.
  function classNames(v, out = []) {
    if (v == null || v === false) return out;
    if (Array.isArray(v)) { for (const x of v) classNames(x, out); return out; }
    if (typeof v === "object") {
      for (const k of Object.keys(v)) if (v[k]) classNames(k, out);
      return out;
    }
    for (const c of String(v).split(/\s+/)) if (c && !out.includes(c)) out.push(c);
    return out;
  }

  function classes(el, get) {
    let prev = [];
    effect(() => {
      const next = classNames(get());
      for (const c of prev) if (!next.includes(c)) el.classList.remove(c);
      for (const c of next) el.classList.add(c);
      prev = next;
    });
  }

  // ─── Events a component declares ─────────────────────
  //
  // `emit toggle(id)` inside a component calls the handler its caller passed
  // as `on: { toggle: … }`; a caller that passed none is not an error.
  function emit(props, name, ...args) {
    const handler = props && props.on && props.on[name];
    if (typeof handler === "function") return handler(...args);
    return undefined;
  }

  function mainOf(container) {
    if (!container) return container;
    if (container.tagName === "MAIN") return container;
    const existing = container.querySelector && container.querySelector("main");
    if (existing) return existing;
    const el = document.createElement("main");
    el.id = "wf-main";
    container.appendChild(el);
    return el;
  }

  /// Wire an off-canvas panel to a toggle button.
  ///
  /// Below the breakpoint the panel slides over the page and the toggle is the
  /// only way to reach it — so the button carries `aria-expanded`, Escape closes
  /// it, focus returns to the button, and a scrim catches the click outside.
  /// Above the breakpoint the CSS ignores all of it and the panel is just a
  /// column.
  // ─── Theme ───────────────────────────────────────────
  // `theme` is `"light"`, `"dark"` or `"system"`: what the reader chose,
  // kept in storage and written to `<html data-theme>`, which the sheet's
  // dark rules read; `"system"` leaves it to `prefers-color-scheme`.
  let _theme = null;
  function themeSignal() {
    if (!_theme) {
      let chosen = "system";
      try { chosen = window.localStorage.getItem("wf:theme") || "system"; } catch (e) { /* no storage */ }
      _theme = signal(chosen);
      applyTheme(chosen);
    }
    return _theme;
  }
  function applyTheme(chosen) {
    const root = document.documentElement;
    if (chosen === "light" || chosen === "dark") root.setAttribute("data-theme", chosen);
    else root.removeAttribute("data-theme");
  }
  function theme() {
    return themeSignal()();
  }
  function setTheme(chosen) {
    chosen = chosen === "light" || chosen === "dark" ? chosen : "system";
    applyTheme(chosen);
    try {
      if (chosen === "system") window.localStorage.removeItem("wf:theme");
      else window.localStorage.setItem("wf:theme", chosen);
    } catch (e) { /* no storage */ }
    themeSignal().set(chosen);
  }

  // ─── Rendered ────────────────────────────────────────
  // A plain script that drives the DOM itself — `querySelectorAll(".tilt")`
  // — has to know when there is a DOM to drive: the page is drawn after it
  // runs, drawn again over a pre-rendered one, and drawn anew on every
  // route. `wf:render` on `document` says so, once per drawing: the first
  // load's mount and its router's paint are one event, not two.
  let renderPending = null;
  function drawn(detail) {
    if (typeof document === "undefined" || typeof CustomEvent !== "function") return;
    const first = renderPending === null;
    renderPending = detail;
    if (!first) return;
    queueMicrotask(() => {
      const d = renderPending;
      renderPending = null;
      if (typeof document.dispatchEvent === "function") {
        document.dispatchEvent(new CustomEvent("wf:render", { detail: d }));
      }
    });
  }

  // ─── Mount ───────────────────────────────────────────
  function mount(renderFn, container) {
    themeSignal();
    _newPage();
    const el = renderFn();
    if (el instanceof Node) {
      container.innerHTML = "";
      container.appendChild(el);
    }
    const path = typeof location !== "undefined" ? location.pathname : "/";
    if (renderPending === null) drawn({ route: path, params: {} });
  }
