"use strict";
// WebFluent Runtime v1.0
// Fine-grained reactivity + DOM helpers + Router + Store + Fetch

const WF = (() => {
  // ─── Reactivity ──────────────────────────────────────
  let currentEffect = null;

  function signal(value) {
    const subs = new Set();
    const get = () => {
      if (currentEffect) subs.add(currentEffect);
      return value;
    };
    const set = (v) => {
      if (typeof v === "function") v = v(value);
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

  function effect(fn) {
    const run = () => {
      const prev = currentEffect;
      currentEffect = run;
      try { fn(); } finally { currentEffect = prev; }
    };
    run();
    return run;
  }

  function computed(fn) {
    const s = signal(undefined);
    effect(() => s.set(fn()));
    return s;
  }

  // ─── DOM Helpers ─────────────────────────────────────
  function h(tag, attrs, ...children) {
    const el = document.createElement(tag);
    // A select's value only takes once its options exist, so it is applied
    // after the children; set before them it was silently ignored.
    let selectValue;
    let iconName;
    if (attrs) {
      for (const [k, v] of Object.entries(attrs)) {
        if (k === "value" && tag === "select") {
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
          if (typeof v === "function") {
            effect(() => { el.value = v(); });
          } else {
            el.value = v;
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
        } else if (k === "data-icon") {
          // The glyph is drawn after the children: an icon button carries
          // data-icon and a .wf-icon child, and used to draw the glyph twice.
          iconName = typeof v === "function" ? v() : v;
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
    if (iconName !== undefined) {
      const drawn = [...el.children].some((c) => String(c.className || "").split(" ").includes("wf-icon"));
      if (!drawn) _renderIcon(el, iconName);
    }
    if (typeof selectValue === "function") {
      effect(() => { const v = selectValue(); if (v != null) el.value = v; });
    } else if (selectValue != null) {
      el.value = selectValue;
    }
    if (tag === "img" && !el.hasAttribute("loading")) _imageDefaults(el);
    return el;
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

  // ─── Animation helpers ──────────────────────────────
  const ANIM_REVERSE = {
    fadeIn: "fadeOut", fadeOut: "fadeIn",
    slideUp: "slideDown", slideDown: "slideUp",
    slideLeft: "slideRight", slideRight: "slideLeft",
    scaleIn: "scaleOut", scaleOut: "scaleIn",
    bounce: "fadeOut", shake: "fadeOut", pulse: "fadeOut",
  };

  function animateIn(el, name, duration, delay) {
    if (!name) return Promise.resolve();
    const cls = "wf-animate-" + name;
    if (duration) el.style.animationDuration = duration;
    if (delay) el.style.animationDelay = delay;
    el.classList.add(cls);
    return new Promise(resolve => {
      const done = () => { el.classList.remove(cls); el.style.animationDuration = ""; el.style.animationDelay = ""; resolve(); };
      el.addEventListener("animationend", done, { once: true });
      // Fallback timeout
      setTimeout(done, (parseInt(duration) || 300) + (parseInt(delay) || 0) + 100);
    });
  }

  function animateOut(el, name, duration) {
    if (!name) return Promise.resolve();
    const cls = "wf-animate-" + name;
    if (duration) el.style.animationDuration = duration;
    el.classList.add(cls);
    return new Promise(resolve => {
      const done = () => { el.classList.remove(cls); el.style.animationDuration = ""; resolve(); };
      el.addEventListener("animationend", done, { once: true });
      setTimeout(done, (parseInt(duration) || 300) + 100);
    });
  }

  function animateEl(target, name, duration) {
    const el = typeof target === "string" ? document.querySelector(`[data-ref="${target}"]`) : target;
    if (!el) return;
    return animateIn(el, name, duration);
  }

  function replayAnimation(el, name, duration) {
    // Remove then re-add the animation class to restart it
    const cls = "wf-animate-" + name;
    el.classList.remove(cls);
    // Force reflow to reset animation
    void el.offsetWidth;
    el.classList.add(cls);
    if (duration) el.style.animationDuration = duration;
  }

  // ─── Conditional rendering ───────────────────────────
  function removeNodes(nodes) {
    for (const n of nodes) {
      if (n && n.parentNode) n.parentNode.removeChild(n);
    }
  }

  function condRender(parent, condFn, thenFn, elseFn, animConfig) {
    const marker = document.createComment("wf-if");
    parent.appendChild(marker);
    let currentNodes = [];
    let lastShow = undefined;
    let pendingRemoval = null; // Track in-progress exit animations

    // Only track the condition signal — not signals read during rendering
    effect(() => {
      const show = !!condFn();
      if (show === lastShow) return;
      lastShow = show;

      // Cancel any pending removal animation
      if (pendingRemoval) {
        removeNodes(pendingRemoval);
        pendingRemoval = null;
      }

      // Remove old nodes
      const toRemove = [...currentNodes];
      currentNodes = [];

      if (animConfig && animConfig.exit && toRemove.length) {
        pendingRemoval = toRemove;
        const exitName = animConfig.exit;
        const promises = toRemove.map(n =>
          n instanceof Element ? animateOut(n, exitName, animConfig.duration) : Promise.resolve()
        );
        Promise.all(promises).then(() => {
          // Only remove if this is still the pending removal (not cancelled by a new toggle)
          if (pendingRemoval === toRemove) {
            removeNodes(toRemove);
            pendingRemoval = null;
          }
        });
      } else {
        removeNodes(toRemove);
      }

      // Add new nodes (untracked so rendering doesn't subscribe this effect to state signals)
      const renderFn = show ? thenFn : elseFn;
      if (renderFn) {
        const prev = currentEffect;
        currentEffect = null; // Untrack: don't subscribe to signals during render
        try {
          const result = renderFn();
          // Collect actual child nodes — DocumentFragments lose children when appended
          let nodes;
          if (result instanceof DocumentFragment) {
            nodes = [...result.childNodes];
          } else {
            nodes = [].concat(result).flat().filter(n => n instanceof Node);
          }
          currentNodes = nodes.slice();
          const frag = document.createDocumentFragment();
          for (const n of nodes) frag.appendChild(n);
          if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
          if (animConfig && animConfig.enter) {
            nodes.forEach(n => { if (n instanceof Element) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay); });
          }
        } finally {
          currentEffect = prev;
        }
      }
    });
  }

  // ─── List rendering ─────────────────────────────────
  function listRender(parent, listFn, itemFn, animConfig) {
    const marker = document.createComment("wf-for");
    parent.appendChild(marker);
    let currentNodes = [];

    effect(() => {
      const items = listFn(); // Track the list signal

      // Remove old
      if (animConfig && animConfig.exit && currentNodes.length) {
        const toRemove = [...currentNodes];
        toRemove.forEach((n, i) => {
          if (n instanceof Element) {
            animateOut(n, animConfig.exit, animConfig.duration).then(() => { if (n.parentNode) n.parentNode.removeChild(n); });
          } else {
            if (n.parentNode) n.parentNode.removeChild(n);
          }
        });
      } else {
        removeNodes(currentNodes);
      }
      currentNodes = [];

      // Render items untracked
      const prev = currentEffect;
      currentEffect = null;
      try {
        const frag = document.createDocumentFragment();
        if (items && items.length) {
          items.forEach((item, index) => {
            const result = itemFn(item, index);
            let nodes;
            if (result instanceof DocumentFragment) {
              nodes = [...result.childNodes];
            } else {
              nodes = [].concat(result).flat().filter(n => n instanceof Node);
            }
            for (const n of nodes) {
              frag.appendChild(n);
              currentNodes.push(n);
              if (animConfig && animConfig.enter && n instanceof Element) {
                const delay = animConfig.stagger ? (parseInt(animConfig.stagger) * index) + "ms" : animConfig.delay;
                animateIn(n, animConfig.enter, animConfig.duration, delay);
              }
            }
          });
        }
        if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
      } finally {
        currentEffect = prev;
      }
    });
  }

  // ─── Show/Hide ───────────────────────────────────────
  function showRender(parent, condFn, contentFn, animConfig) {
    const wrapper = document.createElement("div");
    wrapper.style.display = "contents";
    const nodes = [].concat(contentFn()).flat();
    for (const n of nodes) {
      if (n instanceof Node) wrapper.appendChild(n);
    }
    parent.appendChild(wrapper);

    if (animConfig) {
      effect(() => {
        if (condFn()) {
          wrapper.style.display = "contents";
          if (animConfig.enter) {
            for (const n of wrapper.children) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay);
          }
        } else {
          if (animConfig.exit) {
            const promises = [...wrapper.children].map(n => animateOut(n, animConfig.exit, animConfig.duration));
            Promise.all(promises).then(() => { wrapper.style.display = "none"; });
          } else {
            wrapper.style.display = "none";
          }
        }
      });
    } else {
      effect(() => {
        wrapper.style.display = condFn() ? "contents" : "none";
      });
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

  // Mark `el` as the current page's link while the route matches `href`:
  // `.active` for the stylesheet and `aria-current="page"` for assistive
  // technology, so the two cannot disagree. With `prefix`, a link to a section
  // root also matches the routes beneath it.
  function activeLink(el, href, prefix) {
    const target = String(href).replace(/\/$/, "") || "/";
    effect(() => {
      const path = pathSignal()().replace(/\/$/, "") || "/";
      const on = path === target || (prefix && target !== "/" && path.startsWith(target + "/"));
      if (on) {
        el.classList.add("active");
        el.setAttribute("aria-current", "page");
      } else {
        el.classList.remove("active");
        el.removeAttribute("aria-current");
      }
    });
  }

  function createRouter(routes, container) {
    // Check for SPA redirect from 404.html (?p=/path)
    const urlParams = new URLSearchParams(window.location.search);
    const redirectPath = urlParams.get("p");
    if (redirectPath) {
      window.history.replaceState(null, "", _basePath + redirectPath);
    }

    const initialPath = _stripBase(window.location.pathname);
    const currentPath = pathSignal();
    currentPath.set(initialPath);

    function matchRoute(path) {
      for (const route of routes) {
        const params = matchPath(route.path, path);
        if (params !== null) return { route, params };
      }
      // Try wildcard
      const wild = routes.find(r => r.path === "*");
      if (wild) return { route: wild, params: {} };
      return null;
    }

    function matchPath(pattern, path) {
      if (pattern === path) return {};
      const patternParts = pattern.split("/").filter(Boolean);
      const pathParts = path.split("/").filter(Boolean);
      if (patternParts.length !== pathParts.length) return null;

      const params = {};
      for (let i = 0; i < patternParts.length; i++) {
        if (patternParts[i].startsWith(":")) {
          params[patternParts[i].slice(1)] = pathParts[i];
        } else if (patternParts[i] !== pathParts[i]) {
          return null;
        }
      }
      return params;
    }

    // Whether the route change came from the back/forward buttons, whose
    // scroll position the browser restores itself.
    let fromHistory = false;
    let rendered = false;

    function render() {
      const path = currentPath(); // Only subscribe to path changes
      const match = matchRoute(path);
      if (!match) {
        container.innerHTML = "";
        return;
      }

      const draw = (renderFn) => {
        // The page arrived after the reader had already moved on.
        if (currentPath() !== path) return;
        container.innerHTML = "";
        _newPage();
        // The tab, the history entry and a screen reader all read the title;
        // a single-page app used to keep the entry page's title on every route.
        if (match.route.title) document.title = match.route.title;
        // Untrack: don't subscribe the router effect to signals read during page render
        const prev = currentEffect;
        currentEffect = null;
        try {
          const el = renderFn(match.params);
          if (el instanceof Node) container.appendChild(el);
        } finally {
          currentEffect = prev;
        }

        // A full page load lands the reader at the top with focus on the
        // document; a route change used to leave focus on a link that no
        // longer existed, the scroll wherever it was, and say nothing. It now
        // does what the page load does: focus moves to the new page's heading
        // (or the main landmark when it has none), the viewport returns to the
        // top, and the title is announced.
        if (rendered) {
          settleOnNewPage(container, !fromHistory);
        }
        rendered = true;
        fromHistory = false;
      };

      // A route names its page's render function directly, or names a page
      // that lives in its own chunk and is fetched the first time it shows.
      if (match.route.render) draw(match.route.render);
      else loadPage(match.route.page, draw);
    }

    window.addEventListener("popstate", () => {
      fromHistory = true;
      currentPath.set(_stripBase(window.location.pathname));
    });

    effect(render);

    routerInstance = {
      navigate: (path) => {
        window.history.pushState(null, "", _basePath + path);
        currentPath.set(path);
      },
      currentPath,
      back: () => window.history.back(),
      forward: () => window.history.forward(),
    };

    return routerInstance;
  }

  let _ssgMode = false;
  function setSsgMode(enabled) { _ssgMode = enabled; }
  function setBasePath(path) {
    _basePath = path.replace(/\/$/, "");
    // A link created before the base path was known compared against the
    // unstripped location; re-derive it now that stripping is possible.
    if (_pathSignal) _pathSignal.set(_stripBase(window.location.pathname));
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

  // ─── Pages (route chunks) ────────────────────────────
  //
  // A build writes each page as `pages/<Name>.js`, which registers itself
  // here when it runs; app.js holds the runtime, the stores and the
  // components every page shares. A page's HTML links its own chunk beside
  // app.js, so a static build loads the two in parallel; a route change in a
  // single-page build fetches the chunk the first time the route shows.
  const pages = {};
  const waiting = {};

  function definePage(name, renderFn) {
    pages[name] = renderFn;
    const callbacks = waiting[name];
    delete waiting[name];
    if (callbacks) for (const cb of callbacks) cb(renderFn);
  }

  function loadPage(name, cb) {
    if (pages[name]) {
      cb(pages[name]);
      return;
    }
    (waiting[name] = waiting[name] || []).push(cb);
    // Already linked by the page's HTML, or already requested: it will
    // register itself when it runs.
    if (document.querySelector('script[data-wf-page="' + name + '"]')) return;
    const script = document.createElement("script");
    script.src = _basePath + "/pages/" + name + ".js";
    script.async = true;
    script.setAttribute("data-wf-page", name);
    script.onerror = () => console.error("WebFluent: could not load the page chunk for " + name);
    (document.head || document.body).appendChild(script);
  }

  // The classes an expression names, kept in step with it: what it named
  // last time and no longer does is taken off, what it names now is added,
  // and the element's other classes are left alone.
  function classes(el, get) {
    let prev = [];
    effect(() => {
      const next = String(get() || "").split(/\s+/).filter(Boolean);
      for (const c of prev) if (!next.includes(c)) el.classList.remove(c);
      for (const c of next) el.classList.add(c);
      prev = next;
    });
  }

  function getParams() {
    return routerInstance ? routerInstance._currentParams || {} : {};
  }

  // ─── Store ───────────────────────────────────────────
  function createStore(definition) {
    const store = {};
    const states = {};

    // Create signals for each state
    if (definition.state) {
      for (const [key, val] of Object.entries(definition.state)) {
        const s = signal(typeof val === "function" ? val() : val);
        states[key] = s;
        __reg(key, s); // expose for WF.__debug.state()
        Object.defineProperty(store, key, {
          get: () => s(),
          set: (v) => s.set(v),
        });
      }
    }

    // Bind actions before the derived values: a computed runs as soon as it
    // is created, and a derived value that calls one of the store's own
    // actions used to find it missing.
    if (definition.actions) {
      for (const [key, fn] of Object.entries(definition.actions)) {
        store[key] = (...args) => fn(store, ...args);
      }
    }

    // Create computed for derived
    if (definition.derived) {
      for (const [key, fn] of Object.entries(definition.derived)) {
        const c = computed(() => fn(store));
        Object.defineProperty(store, key, { get: () => c() });
      }
    }

    return store;
  }

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

  // ─── Toast ───────────────────────────────────────────
  //
  // A live region only announces changes made *after* it is in the document, so
  // the container is created once up front rather than lazily on the first
  // toast — otherwise the first notification, the one most worth hearing, is
  // the one that is silently dropped.
  let toastContainer = null;

  function _toastContainer() {
    if (!toastContainer) {
      toastContainer = document.createElement("div");
      toastContainer.className = "wf-toast-container";
      // `polite` waits for a pause rather than interrupting; not atomic, so a
      // second toast announces itself rather than re-reading the whole stack.
      toastContainer.setAttribute("role", "status");
      toastContainer.setAttribute("aria-live", "polite");
      toastContainer.setAttribute("aria-atomic", "false");
      document.body.appendChild(toastContainer);
    }
    return toastContainer;
  }

  function showToast(message, variant, duration) {
    const container = _toastContainer();
    const toast = document.createElement("div");
    toast.className = `wf-toast wf-toast--${variant || "info"}`;
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(() => { toast.classList.add("wf-toast--exit"); setTimeout(() => toast.remove(), 300); }, duration || 3000);
  }

  // ─── Dialogs, popups and tablists ────────────────────

  /// Drive a `<dialog>` from a boolean signal.
  ///
  /// `showModal()` is what buys the focus trap, the inert background, Escape to
  /// close and `aria-modal`. The browser can close the dialog without us — via
  /// Escape or the backdrop — so the `close` event writes back to the signal;
  /// without that the state says "open" while the screen says otherwise, and the
  /// next toggle appears to do nothing.
  function bindDialog(el, openSignal) {
    effect(() => {
      const shouldBeOpen = openSignal();
      if (shouldBeOpen && !el.open) {
        if (el.showModal) el.showModal();
        else el.setAttribute("open", "");
      } else if (!shouldBeOpen && el.open) {
        if (el.close) el.close();
        else el.removeAttribute("open");
      }
    });
    el.addEventListener("close", () => {
      if (openSignal()) openSignal.set(false);
    });
  }

  /// Close a popup on Escape or an outside click, returning focus to its trigger.
  ///
  /// A keyboard user who opens a menu must be able to leave it without tabbing
  /// through every item, and must land back where they were.
  function bindPopup(root, trigger, openSignal) {
    document.addEventListener("click", (e) => {
      if (!root.contains(e.target)) openSignal.set(false);
    });
    root.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && openSignal()) {
        openSignal.set(false);
        if (trigger && trigger.focus) trigger.focus();
        if (e.stopPropagation) e.stopPropagation();
      }
    });
  }

  /// Wire a menu button and its `role="menu"` list to the ARIA menu pattern.
  ///
  /// The items are menuitems that the arrow keys move between (Home and End
  /// jump to the ends), Enter and Space activate, Escape closes with focus
  /// back on the button, and Tab leaves and closes. Opening from the
  /// keyboard puts focus on the first item; a pointer keeps focus on the
  /// button. `bindPopup` supplies the outside click and Escape.
  function menu(root, trigger, list, openSignal) {
    bindPopup(root, trigger, openSignal);
    const items = () =>
      Array.from(list.children).filter((el) => {
        const cls = el.className || "";
        if (cls.includes("divider") || cls.includes("separator")) {
          el.setAttribute("role", "separator");
          return false;
        }
        if (!el.hasAttribute("role")) el.setAttribute("role", "menuitem");
        if (!el.hasAttribute("tabindex")) el.setAttribute("tabindex", "-1");
        return true;
      });
    const focusItem = (i) => {
      const all = items();
      if (!all.length) return;
      const target = all[((i % all.length) + all.length) % all.length];
      if (target.focus) target.focus();
    };
    trigger.addEventListener("keydown", (e) => {
      if (e.key === "ArrowDown" || e.key === "ArrowUp" || e.key === "Enter" || e.key === " ") {
        if (e.preventDefault) e.preventDefault();
        openSignal.set(true);
        setTimeout(() => focusItem(e.key === "ArrowUp" ? -1 : 0), 0);
      }
    });
    list.addEventListener("keydown", (e) => {
      const all = items();
      const current = all.indexOf(document.activeElement);
      let consumed = true;
      if (e.key === "ArrowDown") focusItem(current + 1);
      else if (e.key === "ArrowUp") focusItem(current - 1);
      else if (e.key === "Home") focusItem(0);
      else if (e.key === "End") focusItem(-1);
      else if ((e.key === "Enter" || e.key === " ") && current >= 0) all[current].click();
      else if (e.key === "Tab") openSignal.set(false), (consumed = false);
      else consumed = false;
      if (consumed && e.preventDefault) e.preventDefault();
    });
    // Choosing an item closes the menu; the reader lands back on the button.
    list.addEventListener("click", (e) => {
      if (items().some((item) => item.contains(e.target))) {
        openSignal.set(false);
        if (trigger.focus) trigger.focus();
      }
    });
    items();
  }

  /// Arrow-key navigation for a `role="tablist"`.
  ///
  /// The WAI-ARIA pattern puts only the selected tab in the tab order and moves
  /// between tabs with the arrow keys, so Tab leaves the widget rather than
  /// walking through every tab in it.
  function tablist(nav, activeSignal) {
    nav.addEventListener("keydown", (e) => {
      const tabs = nav.querySelectorAll("button");
      if (!tabs.length) return;
      const current = activeSignal();
      let next = null;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") next = (current + 1) % tabs.length;
      else if (e.key === "ArrowLeft" || e.key === "ArrowUp") next = (current - 1 + tabs.length) % tabs.length;
      else if (e.key === "Home") next = 0;
      else if (e.key === "End") next = tabs.length - 1;
      if (next === null) return;
      if (e.preventDefault) e.preventDefault();
      activeSignal.set(next);
      if (tabs[next] && tabs[next].focus) tabs[next].focus();
    });
  }

  /// Move the reader to a page the router just rendered into `container`.
  function settleOnNewPage(container, resetScroll) {
    const heading = container.querySelector && container.querySelector("h1");
    const target = heading || container;
    if (target && target.setAttribute && !target.hasAttribute("tabindex")) {
      target.setAttribute("tabindex", "-1");
    }
    if (target && target.focus) {
      try { target.focus({ preventScroll: true }); } catch (_) { target.focus(); }
    }
    if (resetScroll && typeof window.scrollTo === "function") {
      window.scrollTo(0, 0);
    }
    announce(document.title || "");
  }

  // ─── Announcements ───────────────────────────────────
  //
  // One polite live region for what changed without a focus change to say
  // so: the page a route change landed on. Created up front, since a live
  // region only announces changes made after it is in the document.
  let announcer = null;

  function announce(text) {
    if (!announcer) {
      announcer = document.createElement("div");
      announcer.className = "wf-visually-hidden";
      announcer.setAttribute("role", "status");
      announcer.setAttribute("aria-live", "polite");
      announcer.setAttribute("aria-atomic", "true");
      document.body.appendChild(announcer);
    }
    // Cleared first, so the same title twice is still read twice.
    announcer.textContent = "";
    setTimeout(() => { announcer.textContent = text; }, 50);
  }

  // ─── Form fields ─────────────────────────────────────
  //
  // `Input(label: …)`, `Select(label: …)`: the label has to be a <label>
  // pointing at the control for a screen reader to read it and a click on
  // it to focus the field. `hint:` is a description the control refers to;
  // `error:` is a message that, while it is not empty, is announced, refers
  // to the control, and marks it invalid — so a validation failure reaches
  // a reader who cannot see the red text.
  let fieldSeq = 0;

  function field(control, opts) {
    const wrapper = h("div", { className: "wf-field" });
    if (!control.id) control.id = "wf-field-" + (++fieldSeq);
    const described = [];
    const existing = control.getAttribute("aria-describedby");
    if (existing) described.push(existing);

    if (opts.label != null) {
      wrapper.appendChild(
        h("label", { className: "wf-label", for: control.id }, opts.label),
      );
    }
    wrapper.appendChild(control);

    if (opts.hint != null) {
      const id = control.id + "-hint";
      wrapper.appendChild(h("p", { className: "wf-field__hint", id }, opts.hint));
      described.push(id);
    }
    if (opts.error !== undefined) {
      const id = control.id + "-error";
      const message = h("p", { className: "wf-field__error", id, role: "alert" });
      wrapper.appendChild(message);
      described.push(id);
      const apply = (value) => {
        const has = value != null && value !== false && value !== "";
        message.textContent = has ? String(value) : "";
        if (has) message.removeAttribute("hidden");
        else message.setAttribute("hidden", "");
        control.setAttribute("aria-invalid", has ? "true" : "false");
      };
      if (typeof opts.error === "function") effect(() => apply(opts.error()));
      else apply(opts.error);
    }
    if (described.length) control.setAttribute("aria-describedby", described.join(" "));
    return wrapper;
  }

  /// Wire a tooltip: the tip describes the trigger, and can be reached and
  /// dismissed without a pointer.
  ///
  /// `role="tooltip"` alone announces nothing — the element the reader is on
  /// has to refer to it — so `aria-describedby` goes on the first focusable
  /// thing inside the wrapper, and on the wrapper itself (made focusable)
  /// when there is none. The stylesheet shows the tip on focus as well as on
  /// hover; Escape hides it until the pointer or focus leaves and returns
  /// (WCAG 1.4.13: content on hover or focus must be dismissible).
  function tooltip(root, tip) {
    if (!root || !tip) return;
    const focusable = root.querySelector(
      "button, a[href], input, select, textarea, summary, [tabindex]",
    );
    const trigger = focusable || root;
    if (!focusable && !root.hasAttribute("tabindex")) root.setAttribute("tabindex", "0");
    const existing = trigger.getAttribute("aria-describedby");
    trigger.setAttribute("aria-describedby", existing ? `${existing} ${tip.id}` : tip.id);
    root.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        root.setAttribute("data-dismissed", "");
        e.stopPropagation();
      }
    });
    const restore = () => root.removeAttribute("data-dismissed");
    root.addEventListener("mouseleave", restore);
    root.addEventListener("focusout", (e) => {
      if (!root.contains(e.relatedTarget)) restore();
    });
  }

  /// An engine string a project may translate: the key is looked up in the
  /// project's messages when it has i18n, else the English is used.
  function _label(key, fallback) {
    if (i18nInstance) {
      const text = i18nInstance.t(key);
      if (text !== key) return text;
    }
    return fallback;
  }

  /// Wire a carousel: the slides in `root`'s track, the controls, the rotation.
  ///
  /// The WAI-ARIA carousel pattern: the root is a region a reader can name,
  /// each slide a group announced as "n of N", the slides off screen hidden
  /// from assistive technology and from the tab order. Rotation that a
  /// reader cannot stop fails WCAG 2.2.2, so autoplay has a pause button,
  /// pauses while the pointer or focus is on it, does not start at all for
  /// a reader who asked for reduced motion, and stops while the tab is
  /// hidden. While it is not rotating, the track is a polite live region,
  /// so a change made by the controls is read.
  function carousel(root, options) {
    const opts = options || {};
    const track = root.querySelector(".wf-carousel__track");
    if (!track) return null;
    const slides = Array.from(track.children).filter(
      (el) => el.classList && el.classList.contains("wf-carousel__slide"),
    );
    const count = slides.length;

    root.setAttribute("role", "region");
    root.setAttribute("aria-roledescription", _label("wf.carousel", "carousel"));
    if (!root.hasAttribute("aria-label") && !root.hasAttribute("aria-labelledby")) {
      root.setAttribute("aria-label", opts.label || _label("wf.carousel", "carousel"));
    }
    slides.forEach((slide, i) => {
      slide.setAttribute("role", "group");
      slide.setAttribute("aria-roledescription", _label("wf.carousel.slide", "slide"));
      if (!slide.hasAttribute("aria-label")) {
        slide.setAttribute("aria-label", `${i + 1} ${_label("wf.carousel.of", "of")} ${count}`);
      }
    });

    const index = signal(0);
    const show = (i) => index.set(((i % count) + count) % count);

    effect(() => {
      const current = index();
      track.style.transform = `translateX(-${current * 100}%)`;
      slides.forEach((slide, i) => {
        const shown = i === current;
        slide.setAttribute("aria-hidden", shown ? "false" : "true");
        if (shown) slide.removeAttribute("inert");
        else slide.setAttribute("inert", "");
      });
    });

    if (count < 2) return { show, index };

    const reduced =
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const autoplay = !!opts.autoplay && !reduced;
    let timer = null;
    let paused = false; // by the reader, with the button
    let pointerOver = false;
    let focused = false;

    const nav = h("div", { className: "wf-carousel__nav" });
    let playButton = null;

    const running = () =>
      autoplay && !paused && !pointerOver && !focused && !(document.hidden === true);

    function sync() {
      if (running()) {
        if (timer === null) {
          timer = setInterval(() => show(index() + 1), opts.interval || 5000);
        }
        track.setAttribute("aria-live", "off");
      } else {
        if (timer !== null) {
          clearInterval(timer);
          timer = null;
        }
        track.setAttribute("aria-live", "polite");
      }
      if (playButton) {
        playButton.setAttribute("aria-pressed", paused ? "true" : "false");
        playButton.setAttribute(
          "aria-label",
          paused
            ? _label("wf.carousel.play", "Start automatic slide rotation")
            : _label("wf.carousel.pause", "Stop automatic slide rotation"),
        );
        playButton.textContent = paused ? "\u25B6" : "\u275A\u275A";
      }
    }

    if (autoplay) {
      playButton = h("button", {
        className: "wf-carousel__control wf-carousel__play",
        type: "button",
        "on:click": () => {
          paused = !paused;
          sync();
        },
      });
      nav.appendChild(playButton);
      root.addEventListener("mouseenter", () => { pointerOver = true; sync(); });
      root.addEventListener("mouseleave", () => { pointerOver = false; sync(); });
      root.addEventListener("focusin", () => { focused = true; sync(); });
      root.addEventListener("focusout", (e) => {
        if (!root.contains(e.relatedTarget)) { focused = false; sync(); }
      });
      document.addEventListener("visibilitychange", sync);
    }

    nav.appendChild(
      h("button", {
        className: "wf-carousel__control wf-carousel__prev",
        type: "button",
        "aria-label": _label("wf.carousel.previous", "Previous slide"),
        "on:click": () => show(index() - 1),
      }, ["\u2039"]),
    );
    const dots = h("div", { className: "wf-carousel__dots" });
    slides.forEach((_, i) => {
      dots.appendChild(
        h("button", {
          className: () => (index() === i ? "wf-carousel__dot active" : "wf-carousel__dot"),
          type: "button",
          "aria-label": `${_label("wf.carousel.goto", "Go to slide")} ${i + 1}`,
          "aria-current": () => (index() === i ? "true" : null),
          "on:click": () => show(i),
        }),
      );
    });
    nav.appendChild(dots);
    nav.appendChild(
      h("button", {
        className: "wf-carousel__control wf-carousel__next",
        type: "button",
        "aria-label": _label("wf.carousel.next", "Next slide"),
        "on:click": () => show(index() + 1),
      }, ["\u203A"]),
    );
    root.appendChild(nav);
    sync();
    return { show, index };
  }

  /// The `<main>` landmark inside `container`, created if it is not there.
  ///
  /// A page needs exactly one main landmark and the skip link needs something to
  /// jump to, whether the page has a router (which supplies its own) or mounts
  /// straight into `#app`. In an SSG build the static paint already contains it,
  /// so this finds that one rather than nesting a second inside it.
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
  function offCanvas(panel, toggle, scrim) {
    if (!panel || !toggle) return;
    let open = false;

    const apply = () => {
      panel.setAttribute("data-open", open ? "true" : "false");
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
      if (scrim) {
        if (open) scrim.removeAttribute("hidden");
        else scrim.setAttribute("hidden", "");
      }
    };

    const set = (next, restoreFocus) => {
      open = next;
      apply();
      if (!next && restoreFocus && toggle.focus) toggle.focus();
    };

    toggle.addEventListener("click", () => set(!open, false));
    if (scrim) scrim.addEventListener("click", () => set(false, true));
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && open) set(false, true);
    });

    apply();
    return { open: () => set(true, false), close: () => set(false, false) };
  }

  // ─── Mount ───────────────────────────────────────────
  function mount(renderFn, container) {
    _newPage();
    const el = renderFn();
    if (el instanceof Node) {
      container.innerHTML = "";
      container.appendChild(el);
    }
  }

  // ─── Hydrate (SSG) ─────────────────────────────────
  function hydrate(renderFn, container) {
    // The server paint is the FIRST paint — real HTML, instantly visible and
    // crawlable — and the client render replaces it.
    //
    // This used to keep the pre-rendered DOM and call renderFn purely for its
    // side effects, discarding the nodes it built. But effects and event
    // listeners are bound to the nodes the render CREATES, so they were all
    // attached to detached elements: reactive text never updated what you could
    // see, and buttons on any pre-rendered page did nothing at all. Reusing the
    // server's DOM needs node matching this runtime does not have, so until it
    // does, replacing it is the behaviour that is actually correct.
    mount(renderFn, container);
  }

  // ─── i18n ────────────────────────────────────────────
  const RTL_LOCALES = new Set(["ar", "he", "fa", "ur"]);
  let i18nInstance = null;

  function createI18n(defaultLocale, translations) {
    const locale = signal(defaultLocale);
    const dir = signal(RTL_LOCALES.has(defaultLocale) ? "rtl" : "ltr");

    function t(key, params) {
      const currentLocale = locale();
      const messages = translations[currentLocale] || translations[defaultLocale] || {};
      let text = messages[key];
      // Fallback to default locale
      if (text === undefined && currentLocale !== defaultLocale) {
        const fallback = translations[defaultLocale] || {};
        text = fallback[key];
      }
      // Fallback to key itself
      if (text === undefined) return key;
      // Interpolate {placeholder} tokens
      if (params && text.includes("{")) {
        for (const [k, v] of Object.entries(params)) {
          text = text.replace(new RegExp("\\{" + k + "\\}", "g"), String(v));
        }
      }
      return text;
    }

    function setLocale(newLocale) {
      locale.set(newLocale);
      const newDir = RTL_LOCALES.has(newLocale) ? "rtl" : "ltr";
      dir.set(newDir);
      document.documentElement.setAttribute("lang", newLocale);
      document.documentElement.setAttribute("dir", newDir);
    }

    i18nInstance = { t, locale, dir, setLocale };
    return i18nInstance;
  }

  // ─── Icon System ────────────────────────────────────
  // Built-in SVG icons for common UI needs
  const _ICONS = {
    close: '<path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
    menu: '<path d="M3 12h18M3 6h18M3 18h18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
    search: '<circle cx="11" cy="11" r="8" fill="none" stroke="currentColor" stroke-width="2"/><path d="M21 21l-4.35-4.35" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
    home: '<path d="M3 12l9-9 9 9M5 10v10a1 1 0 001 1h3v-5h6v5h3a1 1 0 001-1V10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    user: '<path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2M12 11a4 4 0 100-8 4 4 0 000 8z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
    settings: '<circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" stroke-width="2"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 11-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 11-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 11-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 110-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 112.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 114 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 112.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 110 4h-.09a1.65 1.65 0 00-1.51 1z" fill="none" stroke="currentColor" stroke-width="2"/>',
    check: '<polyline points="20 6 9 17 4 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    "chevron-down": '<polyline points="6 9 12 15 18 9" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    "chevron-right": '<polyline points="9 18 15 12 9 6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    "chevron-left": '<polyline points="15 18 9 12 15 6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    plus: '<line x1="12" y1="5" x2="12" y2="19" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><line x1="5" y1="12" x2="19" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
    minus: '<line x1="5" y1="12" x2="19" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
    edit: '<path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" fill="none" stroke="currentColor" stroke-width="2"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" fill="none" stroke="currentColor" stroke-width="2"/>',
    trash: '<polyline points="3 6 5 6 21 6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
    star: '<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>',
    heart: '<path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z" fill="none" stroke="currentColor" stroke-width="2"/>',
    mail: '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" fill="none" stroke="currentColor" stroke-width="2"/><polyline points="22,6 12,13 2,6" fill="none" stroke="currentColor" stroke-width="2"/>',
    bell: '<path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9M13.73 21a2 2 0 01-3.46 0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    download: '<path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    upload: '<path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    eye: '<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" fill="none" stroke="currentColor" stroke-width="2"/><circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" stroke-width="2"/>',
    link: '<path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
    calendar: '<rect x="3" y="4" width="18" height="18" rx="2" ry="2" fill="none" stroke="currentColor" stroke-width="2"/><line x1="16" y1="2" x2="16" y2="6" stroke="currentColor" stroke-width="2"/><line x1="8" y1="2" x2="8" y2="6" stroke="currentColor" stroke-width="2"/><line x1="3" y1="10" x2="21" y2="10" stroke="currentColor" stroke-width="2"/>',
    filter: '<polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>',
    info: '<circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="2"/><line x1="12" y1="16" x2="12" y2="12" stroke="currentColor" stroke-width="2"/><line x1="12" y1="8" x2="12.01" y2="8" stroke="currentColor" stroke-width="2"/>',
    warning: '<path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" fill="none" stroke="currentColor" stroke-width="2"/><line x1="12" y1="9" x2="12" y2="13" stroke="currentColor" stroke-width="2"/><line x1="12" y1="17" x2="12.01" y2="17" stroke="currentColor" stroke-width="2"/>',
    "arrow-left": '<line x1="19" y1="12" x2="5" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><polyline points="12 19 5 12 12 5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    "arrow-right": '<line x1="5" y1="12" x2="19" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><polyline points="12 5 19 12 12 19" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    logout: '<path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>',
    copy: '<rect x="9" y="9" width="13" height="13" rx="2" ry="2" fill="none" stroke="currentColor" stroke-width="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" fill="none" stroke="currentColor" stroke-width="2"/>',
  };

  function _renderIcon(el, name) {
    const svgData = _ICONS[name];
    if (svgData) {
      el.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24" fill="none">${svgData}</svg>`;
    } else {
      // Fallback: render name as text
      el.textContent = name;
    }
  }

  // ─── Exports ─────────────────────────────────────────
  // ─── Studio debug hook (Slice 3 §1.4) ────────────────
  // Ground truth for the studio's e2e assertions and selection sync. tree/query*/
  // dispatch introspect the rendered DOM (via data-wf-node from Slice 2) and need
  // no cooperation from generated code; state() reads a registry that studio-mode
  // codegen populates through __reg.
  const __signals = new Map();
  function __reg(name, sig) {
    __signals.set(name, sig);
    return sig;
  }

  function __directText(el) {
    let t = "";
    for (const n of el.childNodes) {
      if (n.nodeType === 3) t += n.textContent;
    }
    return t.trim();
  }

  const __debug = {
    /// Snapshot of registered reactive signals (page/store state + derived).
    /// `scope` optionally filters by an id prefix.
    state(scope) {
      const out = {};
      for (const [name, sig] of __signals) {
        if (scope && !name.startsWith(scope)) continue;
        try {
          out[name] = typeof sig === "function" ? sig() : sig;
        } catch (_e) {
          out[name] = undefined;
        }
      }
      return out;
    },
    /// The rendered node tree keyed by data-wf-node.
    tree(root) {
      const start = root || document.getElementById("app") || document.body;
      const walk = (el) => {
        const nodes = [];
        for (const child of el.children) {
          nodes.push({
            id: child.getAttribute("data-wf-node") || null,
            tag: child.tagName.toLowerCase(),
            text: __directText(child),
            children: walk(child),
          });
        }
        return nodes;
      };
      return walk(start);
    },
    /// Synthesize a user event on the node with the given id. Returns whether the
    /// node was found.
    dispatch(nodeId, event, payload) {
      const el = document.querySelector('[data-wf-node="' + nodeId + '"]');
      if (!el) return false;
      if (event === "click") {
        el.click();
        return true;
      }
      if (event === "input" || event === "change") {
        if (payload != null && "value" in el) el.value = payload;
        el.dispatchEvent(new Event(event, { bubbles: true }));
        return true;
      }
      el.dispatchEvent(new Event(event, { bubbles: true }));
      return true;
    },
    /// Node ids whose text contains `text` (for assertions).
    queryText(text) {
      const out = [];
      for (const el of document.querySelectorAll("[data-wf-node]")) {
        if (el.textContent && el.textContent.includes(text)) {
          out.push(el.getAttribute("data-wf-node"));
        }
      }
      return out;
    },
    /// Node ids carrying the given ARIA/role attribute.
    queryRole(role) {
      return Array.from(document.querySelectorAll('[data-wf-node][role="' + role + '"]'))
        .map((el) => el.getAttribute("data-wf-node"));
    },
  };

  return {
    signal, effect, computed,
    h, text, reactiveText, appendChildren, onRoot, props,
    condRender, listRender, showRender,
    animateIn, animateOut, animateEl, replayAnimation,
    createRouter, navigate, getParams, activeLink, definePage, loadPage, classes,
    createStore,
    createI18n,
    wfFetch, showToast,
    mount, hydrate, setSsgMode, setBasePath,
    bindDialog, bindPopup, tablist, mainOf, offCanvas, announce, carousel, tooltip, menu, field,
    __debug, __reg,
    get _basePath() { return _basePath; },
    i18n: null,
  };
})();
