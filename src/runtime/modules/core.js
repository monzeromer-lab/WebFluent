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
    live(target);
    target.addEventListener(event, fn, options);
    onCleanup(() => target.removeEventListener(event, fn, options));
  }

  // `on key("ctrl+k") { }`: a keydown whose key and modifiers match the
  // spelling — `mod` (⌘ on Apple's systems, Ctrl elsewhere), `ctrl`,
  // `shift`, `alt`, `meta`/`cmd`, then the key name as `KeyboardEvent.key`
  // spells it (`k`, `Enter`, `Escape`, `ArrowDown`). A chord is its steps
  // with a space between, `mod+k mod+s`: the second within a second and a
  // half of the first.
  const KEY_NAMES = { esc: "escape", return: "enter", space: " ", up: "arrowup", down: "arrowdown", left: "arrowleft", right: "arrowright", plus: "+" };
  const CHORD_MS = 1500;
  function _apple() {
    if (typeof navigator === "undefined") return false;
    const p = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || navigator.userAgent || "";
    return /mac|iphone|ipad|ipod/i.test(p);
  }
  function _steps(spelling) {
    return String(spelling).trim().split(/\s+/);
  }
  // One step, `mod+shift+p`, as what it asks of an event.
  function _step(spelling) {
    const parts = String(spelling).toLowerCase().split("+").map((p) => p.trim());
    let key = parts.pop();
    // `mod++` and `ctrl++` name the plus key.
    if (key === "" && parts[parts.length - 1] === "") { parts.pop(); key = "+"; }
    const has = (m) => parts.includes(m);
    const mod = has("mod");
    return {
      key: KEY_NAMES[key] || key,
      ctrl: has("ctrl") || (mod && !_apple()),
      shift: has("shift"),
      alt: has("alt"),
      meta: has("meta") || has("cmd") || (mod && _apple()),
    };
  }
  function _stepIs(e, step) {
    // A letter or digit is also its place on the keyboard, when the
    // layout gave no Latin letter or digit there: on a Mac ⌥⌘P sends `π`,
    // and an Arabic layout sends `ح`, with `KeyP` as the code. A layout
    // that does give one is read by it, so Ctrl+Q on AZERTY (code `KeyA`)
    // is `mod+q`, never `mod+a` too.
    const pressed = (e.key || "").toLowerCase();
    if (pressed !== step.key) {
      const code = /^[a-z]$/.test(step.key) ? `key${step.key}` : /^[0-9]$/.test(step.key) ? `digit${step.key}` : null;
      if (!code || /^[a-z0-9]$/.test(pressed) || (e.code || "").toLowerCase() !== code) return false;
    }
    return !!e.ctrlKey === step.ctrl && !!e.shiftKey === step.shift && !!e.altKey === step.alt && !!e.metaKey === step.meta;
  }
  // The keys that went down, newest last, so a chord's later step can see
  // its earlier ones; a modifier alone is not a step.
  const _pressed = [];
  let _hearingKeys = false;
  function _hearKeys() {
    if (_hearingKeys || typeof document === "undefined" || !document.addEventListener) return;
    _hearingKeys = true;
    document.addEventListener("keydown", (e) => {
      if (/^(control|shift|alt|meta|os)$/i.test(e.key || "")) return;
      _pressed.push({ e, at: Date.now() });
      if (_pressed.length > 8) _pressed.shift();
    }, true);
  }
  _hearKeys();
  function keyIs(e, spelling) {
    const steps = _steps(spelling).map(_step);
    if (!_stepIs(e, steps[steps.length - 1])) return false;
    if (steps.length === 1) return true;
    // The earlier steps are the keys pressed just before this one.
    let at = -1;
    for (let i = _pressed.length - 1; i >= 0; i--) if (_pressed[i].e === e) { at = i; break; }
    if (at < steps.length - 1) return false;
    for (let k = 1; k < steps.length; k++) {
      const was = _pressed[at - k];
      const next = _pressed[at - k + 1];
      if (!_stepIs(was.e, steps[steps.length - 1 - k]) || next.at - was.at > CHORD_MS) return false;
    }
    return true;
  }
  // The first steps of the chords a page listens for: the key after one is
  // the chord's, not a shortcut of its own.
  const _chordStarts = [];
  function _afterChordStart(e) {
    let at = -1;
    for (let i = _pressed.length - 1; i >= 0; i--) if (_pressed[i].e === e) { at = i; break; }
    if (at < 1) return false;
    const was = _pressed[at - 1];
    if (_pressed[at].at - was.at > CHORD_MS) return false;
    return _chordStarts.some((step) => _stepIs(was.e, step));
  }
  // How a combination is written for the reader on this system: `⌘⇧P` on
  // a Mac, `Ctrl+Shift+P` elsewhere; a chord's steps with a space between.
  const KEY_SHOWN = { arrowup: "↑", arrowdown: "↓", arrowleft: "←", arrowright: "→", escape: "Esc", enter: "Enter", " ": "Space", backspace: "Backspace", delete: "Delete", tab: "Tab", pageup: "Page Up", pagedown: "Page Down", home: "Home", end: "End" };
  function _keyShown(key) {
    if (KEY_SHOWN[key]) return KEY_SHOWN[key];
    return key.length === 1 ? key.toUpperCase() : key.charAt(0).toUpperCase() + key.slice(1);
  }
  function keyLabel(spelling) {
    const apple = _apple();
    return _steps(spelling).map((text) => {
      const s = _step(text);
      const key = _keyShown(s.key);
      if (apple) return (s.ctrl ? "⌃" : "") + (s.alt ? "⌥" : "") + (s.shift ? "⇧" : "") + (s.meta ? "⌘" : "") + key;
      return [s.ctrl && "Ctrl", s.alt && "Alt", s.shift && "Shift", s.meta && "Meta", key].filter(Boolean).join("+");
    }).join(" ");
  }
  // The same, as `aria-keyshortcuts` writes it: `Meta+Shift+P`.
  function keyAria(spelling) {
    return _steps(spelling).map((text) => {
      const s = _step(text);
      const key = s.key === " " ? "Space" : _keyShown(s.key).replace(/^[↑↓←→]$/, (a) => ({ "↑": "ArrowUp", "↓": "ArrowDown", "←": "ArrowLeft", "→": "ArrowRight" })[a]).replace(/^Esc$/, "Escape");
      return [s.ctrl && "Control", s.alt && "Alt", s.shift && "Shift", s.meta && "Meta", key].filter(Boolean).join("+");
    }).join(" ");
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
    _hearKeys();
    const steps = _steps(spelling);
    const first = steps[0];
    const plain = !/(^|\+)(mod|ctrl|control|alt|option|meta|cmd|command)\+/i.test(first)
      && !/^(escape|esc)$/i.test(first.split("+").pop());
    const page = target === document || target === window;
    if (steps.length > 1) {
      const start = _step(first);
      _chordStarts.push(start);
      onCleanup(() => {
        const i = _chordStarts.indexOf(start);
        if (i >= 0) _chordStarts.splice(i, 1);
      });
    }
    listen(target, "keydown", (e) => {
      if (page && plain && _typing(e)) return;
      if (steps.length === 1 && _afterChordStart(e)) return;
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
  // And an app's own, which its config names for frames (`meta.frame`):
  // the build says which, once, before anything is drawn.
  function allowSchemes(schemes) {
    for (const s of schemes) if (!WF_SCHEMES.includes(s)) WF_SCHEMES.push(s);
  }

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

  /// `url` where a picture is drawn — an `<img>`'s `src` — and "" where it
  /// may not be. Beside what `safeUrl` passes, a picture carried in the
  /// address (`data:image/png;base64,…`, `data:image/svg+xml,…`): an img
  /// never runs what it shows. Any other `data:` is refused here too, and
  /// a link, a frame or a form never comes here. The twin of
  /// `codegen::url::guard_picture`.
  function safePicture(url) {
    const kept = safeUrl(url);
    if (kept || url == null) return kept;
    const cleaned = String(url).replace(/[\u0000-\u0020\u007f-\u009f\s]/g, "").toLowerCase();
    return /^data:image\/[a-z0-9+.-]+[;,]/.test(cleaned) ? String(url) : "";
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
  // An attribute as it is written: a value that reads state is kept in
  // step with it, and `null` or `false` is no attribute.
  function plainAttr(el, k, v) {
    if (typeof v === "function") {
      effect(() => {
        const val = v();
        if (val == null || val === false) unsetAttr(el, k);
        else setAttr(el, k, val);
      });
    } else if (v != null && v !== false) {
      el.setAttribute(k, v);
    }
  }
  // A binding that runs again with the value it had writes nothing: the
  // browser takes any write as a change, and styles and lays out again
  // whatever it may touch. A list handed new items each time (a store's
  // derived list) runs every row's bindings, mostly to the same values.
  function setAttr(el, k, val) {
    const s = String(val);
    if (el.getAttribute(k) !== s) el.setAttribute(k, s);
  }
  function unsetAttr(el, k) {
    if (el.hasAttribute(k)) el.removeAttribute(k);
  }
  function setText(node, val) {
    const s = String(val);
    if (node.textContent !== s) node.textContent = s;
  }
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
        // What keeps the element current, or hands it out, holds it: a
        // pre-rendered page keeps its own element in this one's place
        // only when nothing does (hydrate).
        if (_hyd && (typeof v === "function" || k === "ref" || k === "checked" || k === "value")) _hyd.live.add(el);
        if (k.startsWith("attr:")) {
          // An author's attribute spelled like a key of the runtime's own
          // — `html`, `markdown` — is that attribute, never markup: the
          // compiler writes it so (`attr_key`).
          plainAttr(el, k.slice(5), v);
        } else if (k === "ref" && v && typeof v === "object") {
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
            effect(() => {
              const c = String(v());
              if (el.className !== c) el.className = c;
            });
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
          // A value the program sets is told to what follows the control's
          // text — a field's count of what is left — so the count follows a
          // clear or a reset too, not only typing (`el._wfOnValue`).
          const put = tag === "option"
            ? (val) => { el.value = val; el._wfValue = val; }
            : (val) => {
                el.value = val;
                if (el._wfOnValue) el._wfOnValue();
              };
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
            if (val == null) unsetAttr(el, k);
            else setAttr(el, k, val);
          };
          if (typeof v === "function") effect(() => set(v()));
          else set(v);
        } else {
          plainAttr(el, k, v);
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
          // The caller's own, never what every object inherits: a prop
          // named `toString` left to its default is the default, not
          // Object's function.
          const v = given && Object.prototype.hasOwnProperty.call(given, key) ? given[key] : undefined;
          return v === undefined && defaults && Object.prototype.hasOwnProperty.call(defaults, key) ? defaults[key] : v;
        },
      });
    }
    return out;
  }

  // The element what a component's call rendered stands for: the node
  // itself, or its fragment's first element — what a handler, a drag, an
  // attribute or a lifetime written on the call lands on. `null` when it
  // rendered no element.
  function _rootOf(node) {
    if (!node) return null;
    const isFragment = node.nodeType === 11 || node.tagName === "#DOCUMENT-FRAGMENT";
    return isFragment ? [...node.childNodes].find((n) => n.nodeType === 1) || null : node;
  }

  // What a component's call writes on it — a handler, motion markers,
  // attributes — is the root element's. A component whose body opens with
  // an `if`, a `match` or a slot has a root that region draws, and draws
  // again: `apply` runs on the element it holds now and on each one it
  // draws later, inside the scope of what drew it, so what `apply` set up
  // goes when that element does.
  function _eachRoot(drawn, apply) {
    const isFragment = drawn && (drawn.nodeType === 11 || drawn.tagName === "#DOCUMENT-FRAGMENT");
    const nodes = isFragment ? [...drawn.childNodes] : [].concat(drawn).flat();
    for (let i = 0; i < nodes.length; i++) {
      const n = nodes[i];
      if (!n) continue;
      if (n.__wfEnd) {
        (n.__wfDrawn || (n.__wfDrawn = [])).push((next) => _eachRoot(next, apply));
        const held = [];
        for (let m = n.nextSibling; m && m !== n.__wfEnd; m = m.nextSibling) held.push(m);
        _eachRoot(held, apply);
        return;
      }
      if (n.nodeType === 1) { apply(n); return; }
    }
  }

  // A region drew `nodes` between its markers: what follows its root
  // hears of the new one.
  function _drawnIn(start, nodes) {
    if (start.__wfDrawn) for (const fn of start.__wfDrawn) fn(nodes);
    return nodes;
  }

  // Listen on the root element of what a component returned: its fragment's
  // first element. A component that renders several roots gets the handler on
  // the first, which is where a caller expects it.
  function onRoot(frag, event, handler) {
    _eachRoot(frag, (root) => {
      live(root);
      root.addEventListener(event, handler);
    });
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
    _eachRoot(frag, (root) => _mark(root, attrs));
  }
  function _mark(root, attrs) {
    for (const [k, v] of Object.entries(attrs)) {
      root.setAttribute(k, v);
      if (TIMING[k]) root.style[TIMING[k]] = v;
      if (k === "data-wf-animate") root.classList.add("wf-animate-" + v);
    }
  }

  // Attributes written on a component's call — `aria-label`, `data-*`,
  // `id`, `hidden` — land on its root element, as a built-in's do. An ARIA
  // state keeps `false` as the string; any other attribute given `false`
  // or nothing is taken off.
  function rootAttrs(frag, attrs) {
    _eachRoot(frag, (root) => _rootAttrs(root, attrs));
  }
  function _rootAttrs(root, attrs) {
    for (const [k, v] of Object.entries(attrs)) {
      if (k === "class" || typeof v === "function") live(root);
      // `class:` adds to the root's own classes, as on a built-in.
      if (k === "class") {
        classes(root, typeof v === "function" ? v : () => v);
        continue;
      }
      // As the same attribute written on an element: `true` is "true",
      // and `false` no attribute but an ARIA state's.
      const set = (val) => {
        if (val == null || (val === false && !k.startsWith("aria-"))) unsetAttr(root, k);
        else setAttr(root, k, val);
      };
      if (typeof v === "function") effect(() => set(v()));
      else set(v);
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
    const node = live(document.createTextNode(""));
    parent.appendChild(node);
    effect(() => setText(node, fn()));
    return node;
  }

  function text(fn) {
    if (typeof fn === "function") {
      const node = live(document.createTextNode(""));
      effect(() => setText(node, fn()));
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
  // A node drawn over a pre-rendered page may have been matched to the
  // painted one, which is the one in the page (hydrate).
  function removeNodes(nodes) {
    for (const drawn of nodes) {
      const n = inPage(drawn);
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

  // ─── A pre-rendered page, taken over ────────────────
  //
  // The session the hydrate module opens while a pre-rendered page is
  // drawn again (null otherwise), and whether a page was taken over rather
  // than replaced.
  let _hyd = null;
  let _tookOver = false;

  /// Something keeps hold of `node` — an effect, a listener, a widget, a
  /// list — so it is the node the reader must end up with, not the painted
  /// one in its place. Outside a takeover it does nothing.
  function live(node, ...more) {
    if (_hyd) {
      if (node && typeof node === "object") _hyd.live.add(node);
      for (const n of more) if (n && typeof n === "object") _hyd.live.add(n);
    }
    return node;
  }

  /// The node in the page for `node`: the painted one it was matched to,
  /// when a pre-rendered page was taken over, or `node` itself.
  function inPage(node) {
    return (node && node.__wfAt) || node;
  }
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
    live(el);
    let prev = [];
    effect(() => {
      const next = classNames(get());
      for (const c of prev) if (!next.includes(c)) el.classList.remove(c);
      for (const c of next) if (!el.classList.contains(c)) el.classList.add(c);
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
    // A page drawn over a pre-rendered one takes it over now that it is
    // whole: its route, and the app around it.
    if (_hyd && typeof takeOver === "function") takeOver();
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
