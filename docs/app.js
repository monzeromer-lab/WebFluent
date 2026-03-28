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
    if (attrs) {
      for (const [k, v] of Object.entries(attrs)) {
        if (k.startsWith("on:")) {
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
          // Render icon as inline SVG or text emoji/symbol
          const iconName = typeof v === "function" ? v() : v;
          _renderIcon(el, iconName);
        } else if (typeof v === "function") {
          effect(() => { el.setAttribute(k, v()); });
        } else if (v != null && v !== false) {
          el.setAttribute(k, v);
        }
      }
    }
    appendChildren(el, children);
    return el;
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

  function createRouter(routes, container) {
    // Check for SPA redirect from 404.html (?p=/path)
    const urlParams = new URLSearchParams(window.location.search);
    const redirectPath = urlParams.get("p");
    if (redirectPath) {
      window.history.replaceState(null, "", _basePath + redirectPath);
    }

    const initialPath = _stripBase(window.location.pathname);
    const currentPath = signal(initialPath);

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

    function render() {
      const path = currentPath(); // Only subscribe to path changes
      const match = matchRoute(path);
      container.innerHTML = "";

      if (match) {
        // Untrack: don't subscribe the router effect to signals read during page render
        const prev = currentEffect;
        currentEffect = null;
        try {
          const el = match.route.render(match.params);
          if (el instanceof Node) container.appendChild(el);
        } finally {
          currentEffect = prev;
        }
      }
    }

    window.addEventListener("popstate", () => {
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
  function setBasePath(path) { _basePath = path.replace(/\/$/, ""); }

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
        Object.defineProperty(store, key, {
          get: () => s(),
          set: (v) => s.set(v),
        });
      }
    }

    // Create computed for derived
    if (definition.derived) {
      for (const [key, fn] of Object.entries(definition.derived)) {
        const c = computed(() => fn(store));
        Object.defineProperty(store, key, { get: () => c() });
      }
    }

    // Bind actions
    if (definition.actions) {
      for (const [key, fn] of Object.entries(definition.actions)) {
        store[key] = (...args) => fn(store, ...args);
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
  let toastContainer = null;

  function showToast(message, variant, duration) {
    if (!toastContainer) {
      toastContainer = document.createElement("div");
      toastContainer.className = "wf-toast-container";
      document.body.appendChild(toastContainer);
    }
    const toast = document.createElement("div");
    toast.className = `wf-toast wf-toast--${variant || "info"}`;
    toast.textContent = message;
    toastContainer.appendChild(toast);
    setTimeout(() => { toast.classList.add("wf-toast--exit"); setTimeout(() => toast.remove(), 300); }, duration || 3000);
  }

  // ─── Mount ───────────────────────────────────────────
  function mount(renderFn, container) {
    const el = renderFn();
    if (el instanceof Node) {
      container.innerHTML = "";
      container.appendChild(el);
    }
  }

  // ─── Hydrate (SSG) ─────────────────────────────────
  function hydrate(renderFn, container) {
    // If container already has pre-rendered content, keep it and
    // run the render function to initialize signals, effects, and events.
    // The render function builds DOM nodes that won't be inserted —
    // instead, the existing DOM is kept and JS takes over.
    if (container.children.length > 0) {
      // Run render to initialize all signals and effects
      renderFn();
      // The effects will find and update the existing DOM nodes
    } else {
      // No pre-rendered content — fall back to full mount
      mount(renderFn, container);
    }
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
  return {
    signal, effect, computed,
    h, text, reactiveText, appendChildren,
    condRender, listRender, showRender,
    animateIn, animateOut, animateEl, replayAnimation,
    createRouter, navigate, getParams,
    createStore,
    createI18n,
    wfFetch, showToast,
    mount, hydrate, setSsgMode, setBasePath,
    get _basePath() { return _basePath; },
    i18n: null,
  };
})();


WF.setBasePath("/WebFluent");
WF.setSsgMode(true);
WF.i18n = WF.createI18n(
  "en",
  {
    "ar": {
      "cta.subtitle": "أنشئ مشروعك الأول في ثوانٍ.",
      "cta.title": "مستعد للبناء؟",
      "demo.binding": "ربط ثنائي الاتجاه",
      "demo.binding.hint": "التنبيه يتحدث أثناء الكتابة.",
      "demo.binding.placeholder": "اكتب شيئاً هنا...",
      "demo.components": "المكونات",
      "demo.components.hint": "أزرار متنوعة، شارات، وسوم، وشريط تقدم.",
      "demo.conditional": "العرض الشرطي",
      "demo.conditional.text": "هذه البطاقة تتحرك عند تبديل المفتاح.",
      "demo.conditional.toggle": "تبديل المحتوى",
      "demo.counter": "عدّاد تفاعلي",
      "demo.counter.hint": "اضغط الأزرار. الرقم يتحدث فوراً.",
      "demo.subtitle": "هذه مكونات WebFluent حقيقية تعمل في متصفحك.",
      "demo.title": "جرّب مباشرة",
      "footer.built": "WebFluent — لغة الويب الأولى",
      "footer.docs": "التوثيق",
      "hero.cta": "ابدأ الآن",
      "hero.guide": "اقرأ الدليل",
      "hero.sub1": "لغة برمجة تُترجم إلى HTML و CSS و JavaScript.",
      "hero.sub2": "مكونات مدمجة، تفاعلية، توجيه، تدويل، حركات، وتوليد ثابت.",
      "hero.title": "لغة الويب الأولى",
      "nav.a11y": "إمكانية الوصول",
      "nav.animation": "الحركة",
      "nav.cli": "سطر الأوامر",
      "nav.components": "المكونات",
      "nav.guide": "الدليل",
      "nav.home": "الرئيسية",
      "nav.i18n": "التدويل",
      "nav.pdf": "PDF",
      "nav.section.features": "الميزات",
      "nav.section.intro": "مقدمة",
      "nav.section.tools": "الأدوات",
      "nav.ssg": "التوليد الثابت",
      "nav.start": "ابدأ",
      "nav.styling": "التصميم",
      "nav.template": "محرك القوالب",
      "tpl.subtitle": "استخدم WebFluent كمحرك قوالب من Rust و Node.js لتوليد HTML و PDF.",
      "tpl.title": "محرك القوالب",
      "why.a11y": "فحص إمكانية الوصول",
      "why.a11y.desc": "١٢ فحص وقت الترجمة لنص بديل مفقود وتسميات وعناوين. لا يعيق البناء.",
      "why.animation": "حركات",
      "why.animation.desc": "١٢ حركة مدمجة كمعدّلات. دخول/خروج على الشروط والحلقات مع تأخير.",
      "why.components": "أكثر من 50 مكوّن",
      "why.components.desc": "شريط تنقل، بطاقة، نافذة، نموذج، جدول، وألسنة. كل مكون بتصميم افتراضي.",
      "why.design": "نظام تصميم",
      "why.design.desc": "رموز تصميم للألوان والمسافات والخطوط. ٤ سمات. بدّل بسطر واحد.",
      "why.i18n": "تدويل + RTL",
      "why.i18n.desc": "ترجمات JSON، دالة t()، تبديل لغة تفاعلي، اتجاه RTL تلقائي.",
      "why.reactivity": "تفاعلية بالإشارات",
      "why.reactivity.desc": "تحديثات DOM دقيقة بدون DOM افتراضي. فقط العناصر المتأثرة تتحدث.",
      "why.ssg": "توليد ثابت",
      "why.ssg.desc": "عرض الصفحات مسبقاً وقت البناء. محتوى فوري، ثم JS يضيف التفاعلية.",
      "why.subtitle": "كل ما تحتاجه، مدمج في اللغة.",
      "why.syntax": "صياغة تصريحية",
      "why.syntax.desc": "لا XML، لا JSX. اكتب واجهة المستخدم بأقواس معقوفة وأقواس.",
      "why.title": "لماذا WebFluent؟",
      "why.zero": "بدون تبعيات",
      "why.zero.desc": "يُترجم إلى HTML وCSS وJS خالصة. بدون إطار عمل. معايير ويب صافية.",
    },
    "en": {
      "cta.subtitle": "Create your first project in seconds.",
      "cta.title": "Ready to build?",
      "demo.binding": "Two-Way Binding",
      "demo.binding.hint": "The alert updates as you type.",
      "demo.binding.placeholder": "Type something here...",
      "demo.components": "Components",
      "demo.components.hint": "Button variants, badges, tags, and progress bar.",
      "demo.conditional": "Conditional Rendering",
      "demo.conditional.text": "This card animates in/out when you toggle the switch.",
      "demo.conditional.toggle": "Toggle content",
      "demo.counter": "Reactive Counter",
      "demo.counter.hint": "Click the buttons. The number updates instantly.",
      "demo.subtitle": "These are real WebFluent components running in your browser.",
      "demo.title": "Try It Live",
      "footer.built": "WebFluent — The Web-First Language",
      "footer.docs": "Docs",
      "hero.cta": "Get Started",
      "hero.guide": "View Guide",
      "hero.sub1": "A programming language that compiles to HTML, CSS, and JavaScript.",
      "hero.sub2": "Built-in components, reactivity, routing, i18n, animations, and SSG.",
      "hero.title": "The Web-First Language",
      "nav.a11y": "Accessibility",
      "nav.animation": "Animation",
      "nav.cli": "CLI",
      "nav.components": "Components",
      "nav.guide": "Guide",
      "nav.home": "Home",
      "nav.i18n": "i18n",
      "nav.pdf": "PDF",
      "nav.section.features": "Features",
      "nav.section.intro": "Introduction",
      "nav.section.tools": "Tools",
      "nav.ssg": "SSG",
      "nav.start": "Get Started",
      "nav.styling": "Styling",
      "nav.template": "Template Engine",
      "tpl.subtitle": "Use WebFluent as a server-side template engine from Rust and Node.js.",
      "tpl.title": "Template Engine",
      "why.a11y": "A11y Linting",
      "why.a11y.desc": "12 compile-time checks for missing alt text, labels, headings. Never blocks the build.",
      "why.animation": "Animations",
      "why.animation.desc": "12 built-in animations as modifiers. Enter/exit on conditionals and loops with stagger.",
      "why.components": "50+ Components",
      "why.components.desc": "Navbar, Card, Modal, Form, Table, Tabs, and more. Every component has a default design.",
      "why.design": "Design System",
      "why.design.desc": "Design tokens for colors, spacing, typography. 4 themes. Switch with one config line.",
      "why.i18n": "i18n + RTL",
      "why.i18n.desc": "JSON translations, t() function, reactive locale switching, automatic RTL direction.",
      "why.reactivity": "Signal Reactivity",
      "why.reactivity.desc": "Fine-grained DOM updates without a virtual DOM. Only affected nodes update.",
      "why.ssg": "SSG",
      "why.ssg.desc": "Pre-render pages at build time. Instant content, then JS hydrates for interactivity.",
      "why.subtitle": "Everything you need, built into the language.",
      "why.syntax": "Declarative Syntax",
      "why.syntax.desc": "No XML, no JSX. Write UI as readable declarations with curly braces and parentheses.",
      "why.title": "Why WebFluent?",
      "why.zero": "Zero Dependencies",
      "why.zero.desc": "Compiles to vanilla HTML, CSS, JS. No runtime framework. Pure web standards output.",
    },
  }
);

function Component_DocSidebar() {
  const _frag = document.createDocumentFragment();
  const _e0 = WF.h("aside", { className: "wf-sidebar" });
  const _e1 = WF.h("div", { className: "wf-sidebar__header" });
  const _e2 = WF.h("a", { className: "wf-link", href: WF._basePath + "/" });
  const _e3 = WF.h("p", { className: "wf-text wf-text--heading" }, "WebFluent");
  _e2.appendChild(_e3);
  _e1.appendChild(_e2);
  _e0.appendChild(_e1);
  const _e4 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small wf-text--bold wf-text--uppercase" }, () => WF.i18n.t("nav.section.intro"));
  _e0.appendChild(_e4);
  const _e5 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/" });
  _e5.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "home" }));
  const _e6 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.home"));
  _e5.appendChild(_e6);
  _e0.appendChild(_e5);
  const _e7 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/getting-started" });
  _e7.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "arrow-right" }));
  const _e8 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.start"));
  _e7.appendChild(_e8);
  _e0.appendChild(_e7);
  const _e9 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/guide" });
  _e9.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "info" }));
  const _e10 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.guide"));
  _e9.appendChild(_e10);
  _e0.appendChild(_e9);
  _e0.appendChild(WF.h("div", { className: "wf-sidebar__divider" }));
  const _e11 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small wf-text--bold wf-text--uppercase" }, () => WF.i18n.t("nav.section.features"));
  _e0.appendChild(_e11);
  const _e12 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/components" });
  _e12.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "filter" }));
  const _e13 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.components"));
  _e12.appendChild(_e13);
  _e0.appendChild(_e12);
  const _e14 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/styling" });
  _e14.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "eye" }));
  const _e15 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.styling"));
  _e14.appendChild(_e15);
  _e0.appendChild(_e14);
  const _e16 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/animation" });
  _e16.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "star" }));
  const _e17 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.animation"));
  _e16.appendChild(_e17);
  _e0.appendChild(_e16);
  const _e18 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/i18n" });
  _e18.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "link" }));
  const _e19 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.i18n"));
  _e18.appendChild(_e19);
  _e0.appendChild(_e18);
  const _e20 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/ssg" });
  _e20.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "download" }));
  const _e21 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.ssg"));
  _e20.appendChild(_e21);
  _e0.appendChild(_e20);
  const _e22 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/pdf" });
  _e22.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "copy" }));
  const _e23 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.pdf"));
  _e22.appendChild(_e23);
  _e0.appendChild(_e22);
  const _e24 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/template-engine" });
  _e24.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "settings" }));
  const _e25 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.template"));
  _e24.appendChild(_e25);
  _e0.appendChild(_e24);
  _e0.appendChild(WF.h("div", { className: "wf-sidebar__divider" }));
  const _e26 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small wf-text--bold wf-text--uppercase" }, () => WF.i18n.t("nav.section.tools"));
  _e0.appendChild(_e26);
  const _e27 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/accessibility" });
  _e27.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "check" }));
  const _e28 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.a11y"));
  _e27.appendChild(_e28);
  _e0.appendChild(_e27);
  const _e29 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/cli" });
  _e29.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "chevron-right" }));
  const _e30 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.cli"));
  _e29.appendChild(_e30);
  _e0.appendChild(_e29);
  _frag.appendChild(_e0);
  return _frag;
}

function Component_CodeBlock({ code }) {
  const _frag = document.createDocumentFragment();
  const _e31 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e32 = WF.h("div", { className: "wf-card__body" });
  const _e33 = WF.h("code", { className: "wf-code wf-code--block" }, code);
  _e32.appendChild(_e33);
  _e31.appendChild(_e32);
  _frag.appendChild(_e31);
  return _frag;
}

function Component_FeatureCard({ title, description }) {
  const _frag = document.createDocumentFragment();
  const _e34 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-scaleIn" });
  const _e35 = WF.h("div", { className: "wf-card__body" });
  const _e36 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, title);
  _e35.appendChild(_e36);
  const _e37 = WF.h("div", { className: "wf-spacer" });
  _e35.appendChild(_e37);
  const _e38 = WF.h("p", { className: "wf-text wf-text--muted" }, description);
  _e35.appendChild(_e38);
  _e34.appendChild(_e35);
  _frag.appendChild(_e34);
  return _frag;
}

function Component_SiteFooter() {
  const _frag = document.createDocumentFragment();
  const _e39 = WF.h("hr", { className: "wf-divider" });
  _frag.appendChild(_e39);
  const _e40 = WF.h("div", { className: "wf-container" });
  const _e41 = WF.h("div", { className: "wf-spacer" });
  _e40.appendChild(_e41);
  const _e42 = WF.h("div", { className: "wf-row wf-row--center wf-row--between" });
  const _e43 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("footer.built"));
  _e42.appendChild(_e43);
  const _e44 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e45 = WF.h("a", { className: "wf-link", href: WF._basePath + "/" });
  const _e46 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("nav.home"));
  _e45.appendChild(_e46);
  _e44.appendChild(_e45);
  const _e47 = WF.h("a", { className: "wf-link", href: WF._basePath + "/getting-started" });
  const _e48 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("footer.docs"));
  _e47.appendChild(_e48);
  _e44.appendChild(_e47);
  _e42.appendChild(_e44);
  _e40.appendChild(_e42);
  const _e49 = WF.h("div", { className: "wf-spacer" });
  _e40.appendChild(_e49);
  _frag.appendChild(_e40);
  return _frag;
}

function Component_NavBar() {
  const _frag = document.createDocumentFragment();
  const _e50 = WF.h("nav", { className: "wf-navbar" });
  const _e51 = WF.h("div", { className: "wf-navbar__brand" });
  const _e52 = WF.h("a", { className: "wf-link", href: WF._basePath + "/" });
  const _e53 = WF.h("p", { className: "wf-text wf-text--heading" }, "WebFluent");
  _e52.appendChild(_e53);
  _e51.appendChild(_e52);
  _e50.appendChild(_e51);
  const _e54 = WF.h("div", { className: "wf-navbar__links" });
  _e50.appendChild(_e54);
  const _e55 = WF.h("div", { className: "wf-navbar__actions" });
  const _e56 = WF.h("button", { className: "wf-btn wf-btn--small", "on:click": (e) => { WF.i18n.setLocale("en"); } }, "EN");
  _e55.appendChild(_e56);
  const _e57 = WF.h("button", { className: "wf-btn wf-btn--small", "on:click": (e) => { WF.i18n.setLocale("ar"); } }, "AR");
  _e55.appendChild(_e57);
  _e50.appendChild(_e55);
  _frag.appendChild(_e50);
  return _frag;
}

function Page_Ssg(params) {
  const _root = document.createDocumentFragment();
  const _e58 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e59 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e59);
  const _e60 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Static Site Generation (SSG)");
  _e58.appendChild(_e60);
  const _e61 = WF.h("p", { className: "wf-text wf-text--muted" }, "Pre-render pages to HTML at build time for instant content visibility. JavaScript hydrates the page for interactivity.");
  _e58.appendChild(_e61);
  const _e62 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e62);
  const _e63 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Enable SSG");
  _e58.appendChild(_e63);
  const _e64 = WF.h("p", { className: "wf-text" }, "One config flag is all you need.");
  _e58.appendChild(_e64);
  const _e65 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e65);
  const _e66 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e67 = WF.h("div", { className: "wf-card__body" });
  const _e68 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"build\": {\n    \"ssg\": true\n  }\n}");
  _e67.appendChild(_e68);
  _e66.appendChild(_e67);
  _e58.appendChild(_e66);
  const _e69 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e69);
  const _e70 = WF.h("hr", { className: "wf-divider" });
  _e58.appendChild(_e70);
  const _e71 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e71);
  const _e72 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "How It Works");
  _e58.appendChild(_e72);
  const _e73 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e74 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e75 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e76 = WF.h("div", { className: "wf-card__body" });
  const _e77 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "1. Build");
  _e76.appendChild(_e77);
  const _e78 = WF.h("p", { className: "wf-text wf-text--muted" }, "The compiler walks the AST for each page and generates static HTML from the component tree.");
  _e76.appendChild(_e78);
  _e75.appendChild(_e76);
  _e74.appendChild(_e75);
  _e73.appendChild(_e74);
  const _e79 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e80 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e81 = WF.h("div", { className: "wf-card__body" });
  const _e82 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "2. Serve");
  _e81.appendChild(_e82);
  const _e83 = WF.h("p", { className: "wf-text wf-text--muted" }, "The browser loads pre-rendered HTML. Content is visible immediately — no blank white screen.");
  _e81.appendChild(_e83);
  _e80.appendChild(_e81);
  _e79.appendChild(_e80);
  _e73.appendChild(_e79);
  const _e84 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e85 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e86 = WF.h("div", { className: "wf-card__body" });
  const _e87 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "3. Hydrate");
  _e86.appendChild(_e87);
  const _e88 = WF.h("p", { className: "wf-text wf-text--muted" }, "JavaScript runs and hydrates the page: attaches events, initializes state, fills dynamic content.");
  _e86.appendChild(_e88);
  _e85.appendChild(_e86);
  _e84.appendChild(_e85);
  _e73.appendChild(_e84);
  _e58.appendChild(_e73);
  const _e89 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e89);
  const _e90 = WF.h("hr", { className: "wf-divider" });
  _e58.appendChild(_e90);
  const _e91 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e91);
  const _e92 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Build Output");
  _e58.appendChild(_e92);
  const _e93 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e94 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e95 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e96 = WF.h("div", { className: "wf-card__body" });
  const _e97 = WF.h("p", { className: "wf-text wf-text--bold" }, "SPA (default)");
  _e96.appendChild(_e97);
  const _e98 = WF.h("code", { className: "wf-code wf-code--block" }, "build/\n├── index.html       # Empty shell\n├── app.js\n└── styles.css");
  _e96.appendChild(_e98);
  _e95.appendChild(_e96);
  _e94.appendChild(_e95);
  _e93.appendChild(_e94);
  const _e99 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e100 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e101 = WF.h("div", { className: "wf-card__body" });
  const _e102 = WF.h("p", { className: "wf-text wf-text--bold" }, "SSG mode");
  _e101.appendChild(_e102);
  const _e103 = WF.h("code", { className: "wf-code wf-code--block" }, "build/\n├── index.html       # Pre-rendered /\n├── about/\n│   └── index.html   # Pre-rendered /about\n├── blog/\n│   └── index.html   # Pre-rendered /blog\n├── app.js\n└── styles.css");
  _e101.appendChild(_e103);
  _e100.appendChild(_e101);
  _e99.appendChild(_e100);
  _e93.appendChild(_e99);
  _e58.appendChild(_e93);
  const _e104 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e104);
  const _e105 = WF.h("hr", { className: "wf-divider" });
  _e58.appendChild(_e105);
  const _e106 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e106);
  const _e107 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "What Gets Pre-Rendered");
  _e58.appendChild(_e107);
  const _e108 = WF.h("table", { className: "wf-table" });
  const _e109 = WF.h("thead", {});
  const _e110 = WF.h("td", {}, "Element");
  _e109.appendChild(_e110);
  const _e111 = WF.h("td", {}, "SSG Behavior");
  _e109.appendChild(_e111);
  _e108.appendChild(_e109);
  const _e112 = WF.h("tr", {});
  const _e113 = WF.h("td", {}, "Static text, headings, components");
  _e112.appendChild(_e113);
  const _e114 = WF.h("td", {}, "Fully rendered to HTML");
  _e112.appendChild(_e114);
  _e108.appendChild(_e112);
  const _e115 = WF.h("tr", {});
  const _e116 = WF.h("td", {}, "Container, Row, Column, Card, etc.");
  _e115.appendChild(_e116);
  const _e117 = WF.h("td", {}, "Full HTML with classes");
  _e115.appendChild(_e117);
  _e108.appendChild(_e115);
  const _e118 = WF.h("tr", {});
  const _e119 = WF.h("td", {}, "Modifiers (primary, large, etc.)");
  _e118.appendChild(_e119);
  const _e120 = WF.h("td", {}, "CSS classes applied");
  _e118.appendChild(_e120);
  _e108.appendChild(_e118);
  const _e121 = WF.h("tr", {});
  const _e122 = WF.h("td", {}, "Animation modifiers (fadeIn, etc.)");
  _e121.appendChild(_e122);
  const _e123 = WF.h("td", {}, "Animation classes applied");
  _e121.appendChild(_e123);
  _e108.appendChild(_e121);
  const _e124 = WF.h("tr", {});
  const _e125 = WF.h("td", {}, "t() i18n calls");
  _e124.appendChild(_e125);
  const _e126 = WF.h("td", {}, "Default locale text rendered");
  _e124.appendChild(_e126);
  _e108.appendChild(_e124);
  const _e127 = WF.h("tr", {});
  const _e128 = WF.h("td", {}, "State-dependent text");
  _e127.appendChild(_e128);
  const _e129 = WF.h("td", {}, "Empty placeholder (filled by JS)");
  _e127.appendChild(_e129);
  _e108.appendChild(_e127);
  const _e130 = WF.h("tr", {});
  const _e131 = WF.h("td", {}, "if / for blocks");
  _e130.appendChild(_e131);
  const _e132 = WF.h("td", {}, "Comment placeholder (filled by JS)");
  _e130.appendChild(_e132);
  _e108.appendChild(_e130);
  const _e133 = WF.h("tr", {});
  const _e134 = WF.h("td", {}, "show blocks");
  _e133.appendChild(_e134);
  const _e135 = WF.h("td", {}, "Rendered but hidden (display:none)");
  _e133.appendChild(_e135);
  _e108.appendChild(_e133);
  const _e136 = WF.h("tr", {});
  const _e137 = WF.h("td", {}, "fetch blocks");
  _e136.appendChild(_e137);
  const _e138 = WF.h("td", {}, "Loading block if present, else placeholder");
  _e136.appendChild(_e138);
  _e108.appendChild(_e136);
  const _e139 = WF.h("tr", {});
  const _e140 = WF.h("td", {}, "Event handlers");
  _e139.appendChild(_e140);
  const _e141 = WF.h("td", {}, "Attached during hydration");
  _e139.appendChild(_e141);
  _e108.appendChild(_e139);
  _e58.appendChild(_e108);
  const _e142 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e142);
  const _e143 = WF.h("hr", { className: "wf-divider" });
  _e58.appendChild(_e143);
  const _e144 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e144);
  const _e145 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Dynamic Routes");
  _e58.appendChild(_e145);
  const _e146 = WF.h("p", { className: "wf-text wf-text--muted" }, "Pages with :param segments (e.g., /user/:id) cannot be pre-rendered — they fall back to client-side rendering.");
  _e58.appendChild(_e146);
  const _e147 = WF.h("div", { className: "wf-spacer" });
  _e58.appendChild(_e147);
  _root.appendChild(_e58);
  return _root;
}

function Page_NotFound(params) {
  const _root = document.createDocumentFragment();
  const _e148 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e149 = WF.h("div", { className: "wf-spacer" });
  _e148.appendChild(_e149);
  const _e150 = WF.h("div", { className: "wf-stack wf-stack--gap-md" });
  const _e151 = WF.h("h2", { className: "wf-heading wf-heading--h1 wf-text--center wf-heading--primary" }, "404");
  _e150.appendChild(_e151);
  const _e152 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, "Page Not Found");
  _e150.appendChild(_e152);
  const _e153 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, "The page you are looking for does not exist or has been moved.");
  _e150.appendChild(_e153);
  const _e154 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e154);
  const _e155 = WF.h("div", { className: "wf-row" });
  const _e156 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/"); } }, "Go Home");
  _e155.appendChild(_e156);
  _e150.appendChild(_e155);
  _e148.appendChild(_e150);
  const _e157 = WF.h("div", { className: "wf-spacer" });
  _e148.appendChild(_e157);
  _root.appendChild(_e148);
  return _root;
}

function Page_Styling(params) {
  const _root = document.createDocumentFragment();
  const _e158 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e159 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e159);
  const _e160 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Design System & Styling");
  _e158.appendChild(_e160);
  const _e161 = WF.h("p", { className: "wf-text wf-text--muted" }, "Token-based design system. Every component uses design tokens for colors, spacing, typography. Change the entire look with a config update.");
  _e158.appendChild(_e161);
  const _e162 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e162);
  const _e163 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Variant Modifiers");
  _e158.appendChild(_e163);
  const _e164 = WF.h("p", { className: "wf-text" }, "Apply common styles with modifier keywords.");
  _e158.appendChild(_e164);
  const _e165 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e165);
  const _e166 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e167 = WF.h("div", { className: "wf-card__header" });
  const _e168 = WF.h("p", { className: "wf-text wf-text--bold" }, "Size Modifiers");
  _e167.appendChild(_e168);
  _e166.appendChild(_e167);
  const _e169 = WF.h("div", { className: "wf-card__body" });
  const _e170 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e171 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Small");
  _e170.appendChild(_e171);
  const _e172 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Medium");
  _e170.appendChild(_e172);
  const _e173 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large" }, "Large");
  _e170.appendChild(_e173);
  _e169.appendChild(_e170);
  _e166.appendChild(_e169);
  _e158.appendChild(_e166);
  const _e174 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e174);
  const _e175 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e176 = WF.h("div", { className: "wf-card__header" });
  const _e177 = WF.h("p", { className: "wf-text wf-text--bold" }, "Color Modifiers");
  _e176.appendChild(_e177);
  _e175.appendChild(_e176);
  const _e178 = WF.h("div", { className: "wf-card__body" });
  const _e179 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e180 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e179.appendChild(_e180);
  const _e181 = WF.h("button", { className: "wf-btn wf-btn--secondary" }, "Secondary");
  _e179.appendChild(_e181);
  const _e182 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e179.appendChild(_e182);
  const _e183 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e179.appendChild(_e183);
  const _e184 = WF.h("button", { className: "wf-btn wf-btn--warning" }, "Warning");
  _e179.appendChild(_e184);
  const _e185 = WF.h("button", { className: "wf-btn wf-btn--info" }, "Info");
  _e179.appendChild(_e185);
  _e178.appendChild(_e179);
  const _e186 = WF.h("div", { className: "wf-spacer" });
  _e178.appendChild(_e186);
  const _e187 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e188 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Primary");
  _e187.appendChild(_e188);
  const _e189 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Success");
  _e187.appendChild(_e189);
  const _e190 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Danger");
  _e187.appendChild(_e190);
  const _e191 = WF.h("span", { className: "wf-badge wf-badge--warning" }, "Warning");
  _e187.appendChild(_e191);
  _e178.appendChild(_e187);
  _e175.appendChild(_e178);
  _e158.appendChild(_e175);
  const _e192 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e192);
  const _e193 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e194 = WF.h("div", { className: "wf-card__header" });
  const _e195 = WF.h("p", { className: "wf-text wf-text--bold" }, "Shape and Elevation");
  _e194.appendChild(_e195);
  _e193.appendChild(_e194);
  const _e196 = WF.h("div", { className: "wf-card__body" });
  const _e197 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e198 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Default");
  _e197.appendChild(_e198);
  const _e199 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--rounded" }, "Rounded");
  _e197.appendChild(_e199);
  const _e200 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--full" }, "Full Width");
  _e197.appendChild(_e200);
  _e196.appendChild(_e197);
  const _e201 = WF.h("div", { className: "wf-spacer" });
  _e196.appendChild(_e201);
  const _e202 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e203 = WF.h("div", { className: "wf-card" });
  const _e204 = WF.h("div", { className: "wf-card__body" });
  const _e205 = WF.h("p", { className: "wf-text" }, "Default");
  _e204.appendChild(_e205);
  _e203.appendChild(_e204);
  _e202.appendChild(_e203);
  const _e206 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e207 = WF.h("div", { className: "wf-card__body" });
  const _e208 = WF.h("p", { className: "wf-text" }, "Elevated");
  _e207.appendChild(_e208);
  _e206.appendChild(_e207);
  _e202.appendChild(_e206);
  const _e209 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e210 = WF.h("div", { className: "wf-card__body" });
  const _e211 = WF.h("p", { className: "wf-text" }, "Outlined");
  _e210.appendChild(_e211);
  _e209.appendChild(_e210);
  _e202.appendChild(_e209);
  _e196.appendChild(_e202);
  _e193.appendChild(_e196);
  _e158.appendChild(_e193);
  const _e212 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e212);
  const _e213 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e214 = WF.h("div", { className: "wf-card__header" });
  const _e215 = WF.h("p", { className: "wf-text wf-text--bold" }, "Text Modifiers");
  _e214.appendChild(_e215);
  _e213.appendChild(_e214);
  const _e216 = WF.h("div", { className: "wf-card__body" });
  const _e217 = WF.h("p", { className: "wf-text wf-text--bold" }, "Bold text.");
  _e216.appendChild(_e217);
  const _e218 = WF.h("p", { className: "wf-text wf-text--italic" }, "Italic text.");
  _e216.appendChild(_e218);
  const _e219 = WF.h("p", { className: "wf-text wf-text--uppercase" }, "Uppercase text.");
  _e216.appendChild(_e219);
  const _e220 = WF.h("p", { className: "wf-text wf-text--muted" }, "Muted text.");
  _e216.appendChild(_e220);
  const _e221 = WF.h("p", { className: "wf-text wf-text--primary" }, "Primary colored text.");
  _e216.appendChild(_e221);
  const _e222 = WF.h("p", { className: "wf-text wf-text--small" }, "Small text.");
  _e216.appendChild(_e222);
  const _e223 = WF.h("p", { className: "wf-text wf-text--large" }, "Large text.");
  _e216.appendChild(_e223);
  _e213.appendChild(_e216);
  _e158.appendChild(_e213);
  const _e224 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e224);
  const _e225 = WF.h("hr", { className: "wf-divider" });
  _e158.appendChild(_e225);
  const _e226 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e226);
  const _e227 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Design Tokens");
  _e158.appendChild(_e227);
  const _e228 = WF.h("p", { className: "wf-text" }, "All styling is built on tokens — CSS custom properties. Override any token in your config.");
  _e158.appendChild(_e228);
  const _e229 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e229);
  const _e230 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e231 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e232 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e233 = WF.h("div", { className: "wf-card__header" });
  const _e234 = WF.h("p", { className: "wf-text wf-text--bold" }, "Colors");
  _e233.appendChild(_e234);
  _e232.appendChild(_e233);
  const _e235 = WF.h("div", { className: "wf-card__body" });
  const _e236 = WF.h("table", { className: "wf-table" });
  const _e237 = WF.h("thead", {});
  const _e238 = WF.h("td", {}, "Token");
  _e237.appendChild(_e238);
  const _e239 = WF.h("td", {}, "Value");
  _e237.appendChild(_e239);
  _e236.appendChild(_e237);
  const _e240 = WF.h("tr", {});
  const _e241 = WF.h("td", {}, "color-primary");
  _e240.appendChild(_e241);
  const _e242 = WF.h("td", {}, "#3B82F6");
  _e240.appendChild(_e242);
  _e236.appendChild(_e240);
  const _e243 = WF.h("tr", {});
  const _e244 = WF.h("td", {}, "color-success");
  _e243.appendChild(_e244);
  const _e245 = WF.h("td", {}, "#22C55E");
  _e243.appendChild(_e245);
  _e236.appendChild(_e243);
  const _e246 = WF.h("tr", {});
  const _e247 = WF.h("td", {}, "color-danger");
  _e246.appendChild(_e247);
  const _e248 = WF.h("td", {}, "#EF4444");
  _e246.appendChild(_e248);
  _e236.appendChild(_e246);
  const _e249 = WF.h("tr", {});
  const _e250 = WF.h("td", {}, "color-warning");
  _e249.appendChild(_e250);
  const _e251 = WF.h("td", {}, "#F59E0B");
  _e249.appendChild(_e251);
  _e236.appendChild(_e249);
  const _e252 = WF.h("tr", {});
  const _e253 = WF.h("td", {}, "color-text");
  _e252.appendChild(_e253);
  const _e254 = WF.h("td", {}, "#0F172A");
  _e252.appendChild(_e254);
  _e236.appendChild(_e252);
  const _e255 = WF.h("tr", {});
  const _e256 = WF.h("td", {}, "color-border");
  _e255.appendChild(_e256);
  const _e257 = WF.h("td", {}, "#E2E8F0");
  _e255.appendChild(_e257);
  _e236.appendChild(_e255);
  _e235.appendChild(_e236);
  _e232.appendChild(_e235);
  _e231.appendChild(_e232);
  _e230.appendChild(_e231);
  const _e258 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e259 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e260 = WF.h("div", { className: "wf-card__header" });
  const _e261 = WF.h("p", { className: "wf-text wf-text--bold" }, "Spacing and Radius");
  _e260.appendChild(_e261);
  _e259.appendChild(_e260);
  const _e262 = WF.h("div", { className: "wf-card__body" });
  const _e263 = WF.h("table", { className: "wf-table" });
  const _e264 = WF.h("thead", {});
  const _e265 = WF.h("td", {}, "Token");
  _e264.appendChild(_e265);
  const _e266 = WF.h("td", {}, "Value");
  _e264.appendChild(_e266);
  _e263.appendChild(_e264);
  const _e267 = WF.h("tr", {});
  const _e268 = WF.h("td", {}, "spacing-xs");
  _e267.appendChild(_e268);
  const _e269 = WF.h("td", {}, "0.25rem");
  _e267.appendChild(_e269);
  _e263.appendChild(_e267);
  const _e270 = WF.h("tr", {});
  const _e271 = WF.h("td", {}, "spacing-sm");
  _e270.appendChild(_e271);
  const _e272 = WF.h("td", {}, "0.5rem");
  _e270.appendChild(_e272);
  _e263.appendChild(_e270);
  const _e273 = WF.h("tr", {});
  const _e274 = WF.h("td", {}, "spacing-md");
  _e273.appendChild(_e274);
  const _e275 = WF.h("td", {}, "1rem");
  _e273.appendChild(_e275);
  _e263.appendChild(_e273);
  const _e276 = WF.h("tr", {});
  const _e277 = WF.h("td", {}, "spacing-lg");
  _e276.appendChild(_e277);
  const _e278 = WF.h("td", {}, "1.5rem");
  _e276.appendChild(_e278);
  _e263.appendChild(_e276);
  const _e279 = WF.h("tr", {});
  const _e280 = WF.h("td", {}, "radius-md");
  _e279.appendChild(_e280);
  const _e281 = WF.h("td", {}, "0.5rem");
  _e279.appendChild(_e281);
  _e263.appendChild(_e279);
  const _e282 = WF.h("tr", {});
  const _e283 = WF.h("td", {}, "radius-full");
  _e282.appendChild(_e283);
  const _e284 = WF.h("td", {}, "9999px");
  _e282.appendChild(_e284);
  _e263.appendChild(_e282);
  _e262.appendChild(_e263);
  _e259.appendChild(_e262);
  _e258.appendChild(_e259);
  _e230.appendChild(_e258);
  _e158.appendChild(_e230);
  const _e285 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e285);
  const _e286 = WF.h("hr", { className: "wf-divider" });
  _e158.appendChild(_e286);
  const _e287 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e287);
  const _e288 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Themes");
  _e158.appendChild(_e288);
  const _e289 = WF.h("p", { className: "wf-text" }, "4 built-in themes. Set in webfluent.app.json. Each preview below shows the actual colors and style of that theme.");
  _e158.appendChild(_e289);
  const _e290 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e290);
  const _e291 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e292 = WF.h("div", { className: "wf-card" });
  const _e293 = WF.h("div", { className: "wf-card__body" });
  const _e294 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e295 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "default");
  _e294.appendChild(_e295);
  const _e296 = WF.h("p", { className: "wf-text wf-text--bold" }, "Default");
  _e294.appendChild(_e296);
  _e293.appendChild(_e294);
  const _e297 = WF.h("div", { className: "wf-spacer" });
  _e293.appendChild(_e297);
  const _e298 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Clean, modern light theme with blue primary.");
  _e293.appendChild(_e298);
  const _e299 = WF.h("div", { className: "wf-spacer" });
  _e293.appendChild(_e299);
  const _e300 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e301 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Primary");
  _e300.appendChild(_e301);
  const _e302 = WF.h("button", { className: "wf-btn wf-btn--success wf-btn--small" }, "Success");
  _e300.appendChild(_e302);
  const _e303 = WF.h("span", { className: "wf-badge wf-badge--info" }, "Tag");
  _e300.appendChild(_e303);
  _e293.appendChild(_e300);
  const _e304 = WF.h("div", { className: "wf-spacer" });
  _e293.appendChild(_e304);
  const _e305 = WF.h("progress", { className: "wf-progress wf-progress--primary", value: 65, max: 100 });
  _e293.appendChild(_e305);
  const _e306 = WF.h("div", { className: "wf-spacer" });
  _e293.appendChild(_e306);
  const _e307 = WF.h("code", { className: "wf-code wf-code--block" }, "\"theme\": { \"name\": \"default\" }");
  _e293.appendChild(_e307);
  _e292.appendChild(_e293);
  _e292.style.background = "#ffffff";
  _e292.style.border = "1px solid #E2E8F0";
  _e292.style.borderRadius = "0.75rem";
  _e291.appendChild(_e292);
  const _e308 = WF.h("div", { className: "wf-card" });
  const _e309 = WF.h("div", { className: "wf-card__body" });
  const _e310 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e311 = WF.h("span", { className: "wf-badge wf-badge--secondary" }, "dark");
  _e310.appendChild(_e311);
  const _e312 = WF.h("p", { className: "wf-text wf-text--bold" }, "Dark");
  _e310.appendChild(_e312);
  _e309.appendChild(_e310);
  const _e313 = WF.h("div", { className: "wf-spacer" });
  _e309.appendChild(_e313);
  const _e314 = WF.h("p", { className: "wf-text wf-text--small" }, "Dark backgrounds with light text and vibrant accents.");
  _e309.appendChild(_e314);
  const _e315 = WF.h("div", { className: "wf-spacer" });
  _e309.appendChild(_e315);
  const _e316 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e317 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Primary");
  _e316.appendChild(_e317);
  const _e318 = WF.h("button", { className: "wf-btn wf-btn--danger wf-btn--small" }, "Danger");
  _e316.appendChild(_e318);
  const _e319 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Live");
  _e316.appendChild(_e319);
  _e309.appendChild(_e316);
  const _e320 = WF.h("div", { className: "wf-spacer" });
  _e309.appendChild(_e320);
  const _e321 = WF.h("progress", { className: "wf-progress wf-progress--info", value: 80, max: 100 });
  _e309.appendChild(_e321);
  const _e322 = WF.h("div", { className: "wf-spacer" });
  _e309.appendChild(_e322);
  const _e323 = WF.h("code", { className: "wf-code wf-code--block" }, "\"theme\": { \"name\": \"dark\" }");
  _e309.appendChild(_e323);
  _e308.appendChild(_e309);
  _e308.style.background = "#0F172A";
  _e308.style.color = "#E2E8F0";
  _e308.style.border = "1px solid #334155";
  _e308.style.borderRadius = "0.75rem";
  _e291.appendChild(_e308);
  const _e324 = WF.h("div", { className: "wf-card" });
  const _e325 = WF.h("div", { className: "wf-card__body" });
  const _e326 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e327 = WF.h("span", { className: "wf-badge" }, "minimal");
  _e326.appendChild(_e327);
  const _e328 = WF.h("p", { className: "wf-text wf-text--bold" }, "Minimal");
  _e326.appendChild(_e328);
  _e325.appendChild(_e326);
  const _e329 = WF.h("div", { className: "wf-spacer" });
  _e325.appendChild(_e329);
  const _e330 = WF.h("p", { className: "wf-text wf-text--small" }, "Black and white. No shadows, no border-radius. Pure content.");
  _e325.appendChild(_e330);
  const _e331 = WF.h("div", { className: "wf-spacer" });
  _e325.appendChild(_e331);
  const _e332 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e333 = WF.h("button", { className: "wf-btn wf-btn--small" }, "Action");
  _e332.appendChild(_e333);
  const _e334 = WF.h("span", { className: "wf-badge" }, "Note");
  _e332.appendChild(_e334);
  _e325.appendChild(_e332);
  const _e335 = WF.h("div", { className: "wf-spacer" });
  _e325.appendChild(_e335);
  const _e336 = WF.h("progress", { className: "wf-progress", value: 50, max: 100 });
  _e325.appendChild(_e336);
  const _e337 = WF.h("div", { className: "wf-spacer" });
  _e325.appendChild(_e337);
  const _e338 = WF.h("code", { className: "wf-code wf-code--block" }, "\"theme\": { \"name\": \"minimal\" }");
  _e325.appendChild(_e338);
  _e324.appendChild(_e325);
  _e324.style.background = "#ffffff";
  _e324.style.border = "2px solid #000000";
  _e324.style.borderRadius = "0";
  _e291.appendChild(_e324);
  const _e339 = WF.h("div", { className: "wf-card" });
  const _e340 = WF.h("div", { className: "wf-card__body" });
  const _e341 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e342 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "brutalist");
  _e341.appendChild(_e342);
  const _e343 = WF.h("p", { className: "wf-text wf-text--bold" }, "Brutalist");
  _e341.appendChild(_e343);
  _e340.appendChild(_e341);
  const _e344 = WF.h("div", { className: "wf-spacer" });
  _e340.appendChild(_e344);
  const _e345 = WF.h("p", { className: "wf-text wf-text--small" }, "Monospace font, bold red primary, hard offset shadows.");
  _e340.appendChild(_e345);
  const _e346 = WF.h("div", { className: "wf-spacer" });
  _e340.appendChild(_e346);
  const _e347 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e348 = WF.h("button", { className: "wf-btn wf-btn--danger wf-btn--small" }, "Action");
  _e347.appendChild(_e348);
  const _e349 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Alert");
  _e347.appendChild(_e349);
  _e340.appendChild(_e347);
  const _e350 = WF.h("div", { className: "wf-spacer" });
  _e340.appendChild(_e350);
  const _e351 = WF.h("progress", { className: "wf-progress wf-progress--danger", value: 90, max: 100 });
  _e340.appendChild(_e351);
  const _e352 = WF.h("div", { className: "wf-spacer" });
  _e340.appendChild(_e352);
  const _e353 = WF.h("code", { className: "wf-code wf-code--block" }, "\"theme\": { \"name\": \"brutalist\" }");
  _e340.appendChild(_e353);
  _e339.appendChild(_e340);
  _e339.style.background = "#ffffff";
  _e339.style.border = "3px solid #000000";
  _e339.style.borderRadius = "0";
  _e339.style.boxShadow = "4px 4px 0 #000000";
  _e339.style.fontFamily = "monospace";
  _e291.appendChild(_e339);
  _e158.appendChild(_e291);
  const _e354 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e354);
  const _e355 = WF.h("hr", { className: "wf-divider" });
  _e158.appendChild(_e355);
  const _e356 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e356);
  const _e357 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Custom Tokens");
  _e158.appendChild(_e357);
  const _e358 = WF.h("p", { className: "wf-text" }, "Override any design token in your config to customize the theme.");
  _e158.appendChild(_e358);
  const _e359 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e359);
  const _e360 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e361 = WF.h("div", { className: "wf-card__body" });
  const _e362 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"theme\": {\n    \"name\": \"default\",\n    \"tokens\": {\n      \"color-primary\": \"#8B5CF6\",\n      \"color-secondary\": \"#EC4899\",\n      \"font-family\": \"Poppins, sans-serif\",\n      \"radius-md\": \"1rem\"\n    }\n  }\n}");
  _e361.appendChild(_e362);
  _e360.appendChild(_e361);
  _e158.appendChild(_e360);
  const _e363 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e363);
  const _e364 = WF.h("hr", { className: "wf-divider" });
  _e158.appendChild(_e364);
  const _e365 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e365);
  const _e366 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Style Blocks");
  _e158.appendChild(_e366);
  const _e367 = WF.h("p", { className: "wf-text" }, "Override styles on any component with inline style blocks.");
  _e158.appendChild(_e367);
  const _e368 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e368);
  const _e369 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e370 = WF.h("div", { className: "wf-card__body" });
  const _e371 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Custom\") {\n    style {\n        background: \"#8B5CF6\"\n        padding: xl\n        radius: lg\n    }\n}");
  _e370.appendChild(_e371);
  _e369.appendChild(_e370);
  _e158.appendChild(_e369);
  const _e372 = WF.h("div", { className: "wf-spacer" });
  _e158.appendChild(_e372);
  _root.appendChild(_e158);
  return _root;
}

function Page_TemplateEngine(params) {
  const _root = document.createDocumentFragment();
  const _e373 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e374 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e374);
  const _e375 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, () => WF.i18n.t("tpl.title"));
  _e373.appendChild(_e375);
  const _e376 = WF.h("p", { className: "wf-text wf-text--muted" }, () => WF.i18n.t("tpl.subtitle"));
  _e373.appendChild(_e376);
  const _e377 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e377);
  const _e378 = WF.h("div", { className: "wf-alert wf-alert--info" }, "WebFluent can be used as a server-side template engine from Rust and Node.js to render .wf templates into HTML or PDF with JSON data.");
  _e373.appendChild(_e378);
  const _e379 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e379);
  const _e380 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "CLI Usage");
  _e373.appendChild(_e380);
  const _e381 = WF.h("p", { className: "wf-text" }, "Render any .wf template with JSON data directly from the command line.");
  _e373.appendChild(_e381);
  const _e382 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e382);
  const _e383 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e384 = WF.h("div", { className: "wf-card__body" });
  const _e385 = WF.h("code", { className: "wf-code wf-code--block" }, "# Render to HTML\nwf render template.wf --data data.json --format html -o output.html\n\n# Render to HTML fragment (no <html> wrapper)\nwf render template.wf --data data.json --format fragment\n\n# Render to PDF\nwf render template.wf --data data.json --format pdf -o report.pdf\n\n# Pipe JSON from stdin\necho '{\"name\":\"Monzer\"}' | wf render template.wf --format html\n\n# With theme\nwf render template.wf --data data.json --format html --theme dark");
  _e384.appendChild(_e385);
  _e383.appendChild(_e384);
  _e373.appendChild(_e383);
  const _e386 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e386);
  const _e387 = WF.h("table", { className: "wf-table" });
  const _e388 = WF.h("thead", {});
  const _e389 = WF.h("td", {}, "Option");
  _e388.appendChild(_e389);
  const _e390 = WF.h("td", {}, "Description");
  _e388.appendChild(_e390);
  _e387.appendChild(_e388);
  const _e391 = WF.h("tr", {});
  const _e392 = WF.h("td", {}, "template");
  _e391.appendChild(_e392);
  const _e393 = WF.h("td", {}, "Path to the .wf template file");
  _e391.appendChild(_e393);
  _e387.appendChild(_e391);
  const _e394 = WF.h("tr", {});
  const _e395 = WF.h("td", {}, "--data");
  _e394.appendChild(_e395);
  const _e396 = WF.h("td", {}, "Path to JSON data file (reads stdin if omitted)");
  _e394.appendChild(_e396);
  _e387.appendChild(_e394);
  const _e397 = WF.h("tr", {});
  const _e398 = WF.h("td", {}, "--format, -f");
  _e397.appendChild(_e398);
  const _e399 = WF.h("td", {}, "Output format: html, fragment, or pdf");
  _e397.appendChild(_e399);
  _e387.appendChild(_e397);
  const _e400 = WF.h("tr", {});
  const _e401 = WF.h("td", {}, "--output, -o");
  _e400.appendChild(_e401);
  const _e402 = WF.h("td", {}, "Output file path (stdout if omitted)");
  _e400.appendChild(_e402);
  _e387.appendChild(_e400);
  const _e403 = WF.h("tr", {});
  const _e404 = WF.h("td", {}, "--theme");
  _e403.appendChild(_e404);
  const _e405 = WF.h("td", {}, "Theme name (default: \"default\")");
  _e403.appendChild(_e405);
  _e387.appendChild(_e403);
  _e373.appendChild(_e387);
  const _e406 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e406);
  const _e407 = WF.h("hr", { className: "wf-divider" });
  _e373.appendChild(_e407);
  const _e408 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e408);
  const _e409 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Template Syntax");
  _e373.appendChild(_e409);
  const _e410 = WF.h("p", { className: "wf-text" }, "Templates use standard .wf syntax. Data is passed as a JSON object — top-level keys become template variables.");
  _e373.appendChild(_e410);
  const _e411 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e411);
  const _e412 = WF.h("div", { className: "wf-grid wf-grid--gap-lg", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e413 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e414 = WF.h("div", { className: "wf-card__header" });
  const _e415 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Template");
  _e414.appendChild(_e415);
  const _e416 = WF.h("p", { className: "wf-text wf-text--bold" }, "invoice.wf");
  _e414.appendChild(_e416);
  _e413.appendChild(_e414);
  const _e417 = WF.h("div", { className: "wf-card__body" });
  const _e418 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Invoice (path: \"/\", title: \"Invoice\") {\n    Container {\n        Heading(\"Invoice #{number}\", h1)\n        Text(\"Customer: {customer.name}\")\n\n        Table {\n            Thead { Trow { Tcell(\"Item\") Tcell(\"Price\") } }\n            for item in items {\n                Trow {\n                    Tcell(item.name)\n                    Tcell(\"${item.price}\")\n                }\n            }\n        }\n\n        if paid {\n            Badge(\"PAID\", success)\n        } else {\n            Badge(\"UNPAID\", danger)\n        }\n    }\n}");
  _e417.appendChild(_e418);
  _e413.appendChild(_e417);
  _e412.appendChild(_e413);
  const _e419 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e420 = WF.h("div", { className: "wf-card__header" });
  const _e421 = WF.h("span", { className: "wf-badge wf-badge--info" }, "Data");
  _e420.appendChild(_e421);
  const _e422 = WF.h("p", { className: "wf-text wf-text--bold" }, "data.json");
  _e420.appendChild(_e422);
  _e419.appendChild(_e420);
  const _e423 = WF.h("div", { className: "wf-card__body" });
  const _e424 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"number\": \"INV-001\",\n  \"customer\": { \"name\": \"Acme Corp\" },\n  \"items\": [\n    { \"name\": \"Widget\", \"price\": 9.99 },\n    { \"name\": \"Gadget\", \"price\": 24.99 }\n  ],\n  \"paid\": true\n}");
  _e423.appendChild(_e424);
  _e419.appendChild(_e423);
  _e412.appendChild(_e419);
  _e373.appendChild(_e412);
  const _e425 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e425);
  const _e426 = WF.h("hr", { className: "wf-divider" });
  _e373.appendChild(_e426);
  const _e427 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e427);
  const _e428 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Rust API");
  _e373.appendChild(_e428);
  const _e429 = WF.h("p", { className: "wf-text" }, "Add WebFluent as a library dependency to use templates in your Rust application.");
  _e373.appendChild(_e429);
  const _e430 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e430);
  const _e431 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e432 = WF.h("div", { className: "wf-card__header" });
  const _e433 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Cargo.toml");
  _e432.appendChild(_e433);
  _e431.appendChild(_e432);
  const _e434 = WF.h("div", { className: "wf-card__body" });
  const _e435 = WF.h("code", { className: "wf-code wf-code--block" }, "[dependencies]\nwebfluent = \"0.2\"");
  _e434.appendChild(_e435);
  _e431.appendChild(_e434);
  _e373.appendChild(_e431);
  const _e436 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e436);
  const _e437 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e438 = WF.h("div", { className: "wf-card__header" });
  const _e439 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "main.rs");
  _e438.appendChild(_e439);
  _e437.appendChild(_e438);
  const _e440 = WF.h("div", { className: "wf-card__body" });
  const _e441 = WF.h("code", { className: "wf-code wf-code--block" }, "use webfluent::Template;\nuse serde_json::json;\n\nfn main() -> webfluent::Result<()> {\n    let tpl = Template::from_file(\"templates/invoice.wf\")?;\n\n    // HTML document (with embedded CSS)\n    let html = tpl.render_html(&json!({\n        \"number\": \"INV-001\",\n        \"customer\": { \"name\": \"Acme Corp\" },\n        \"items\": [{ \"name\": \"Widget\", \"price\": 9.99 }],\n        \"paid\": true\n    }))?;\n\n    // HTML fragment (no wrapper)\n    let fragment = tpl.render_html_fragment(&data)?;\n\n    // PDF bytes\n    let pdf_bytes = tpl.render_pdf(&data)?;\n    std::fs::write(\"invoice.pdf\", pdf_bytes)?;\n\n    // With custom theme\n    let dark = Template::from_file(\"invoice.wf\")?\n        .with_theme(\"dark\")\n        .with_tokens(&[(\"color-primary\", \"#8B5CF6\")]);\n    let html = dark.render_html(&data)?;\n\n    Ok(())\n}");
  _e440.appendChild(_e441);
  _e437.appendChild(_e440);
  _e373.appendChild(_e437);
  const _e442 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e442);
  const _e443 = WF.h("hr", { className: "wf-divider" });
  _e373.appendChild(_e443);
  const _e444 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e444);
  const _e445 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Node.js API");
  _e373.appendChild(_e445);
  const _e446 = WF.h("p", { className: "wf-text" }, "Use WebFluent templates in Express, Next.js, or any Node.js application.");
  _e373.appendChild(_e446);
  const _e447 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e447);
  const _e448 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e449 = WF.h("div", { className: "wf-card__header" });
  const _e450 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Install");
  _e449.appendChild(_e450);
  _e448.appendChild(_e449);
  const _e451 = WF.h("div", { className: "wf-card__body" });
  const _e452 = WF.h("code", { className: "wf-code wf-code--block" }, "npm install @aspect/webfluent");
  _e451.appendChild(_e452);
  _e448.appendChild(_e451);
  _e373.appendChild(_e448);
  const _e453 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e453);
  const _e454 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e455 = WF.h("div", { className: "wf-card__header" });
  const _e456 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Basic Usage");
  _e455.appendChild(_e456);
  _e454.appendChild(_e455);
  const _e457 = WF.h("div", { className: "wf-card__body" });
  const _e458 = WF.h("code", { className: "wf-code wf-code--block" }, "const { Template } = require('@aspect/webfluent');\n\nconst tpl = Template.fromFile('templates/invoice.wf');\n// or: Template.fromString('Container { Heading(\"Hello!\", h1) }');\n\n// Render to HTML\nconst html = tpl.renderHtml({ name: \"World\" });\n\n// Render to HTML fragment\nconst frag = tpl.renderHtmlFragment({ name: \"World\" });\n\n// Render to PDF (returns Buffer)\nconst pdf = tpl.renderPdf({ name: \"World\" });\nfs.writeFileSync('output.pdf', pdf);");
  _e457.appendChild(_e458);
  _e454.appendChild(_e457);
  _e373.appendChild(_e454);
  const _e459 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e459);
  const _e460 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e461 = WF.h("div", { className: "wf-card__header" });
  const _e462 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Express.js Example");
  _e461.appendChild(_e462);
  _e460.appendChild(_e461);
  const _e463 = WF.h("div", { className: "wf-card__body" });
  const _e464 = WF.h("code", { className: "wf-code wf-code--block" }, "const express = require('express');\nconst { Template } = require('@aspect/webfluent');\n\nconst app = express();\n\napp.get('/invoice/:id', async (req, res) => {\n    const invoice = await db.getInvoice(req.params.id);\n    const tpl = Template.fromFile('templates/invoice.wf');\n    res.send(tpl.renderHtml(invoice));\n});\n\napp.get('/invoice/:id/pdf', async (req, res) => {\n    const invoice = await db.getInvoice(req.params.id);\n    const tpl = Template.fromFile('templates/invoice.wf');\n    res.type('application/pdf').send(tpl.renderPdf(invoice));\n});");
  _e463.appendChild(_e464);
  _e460.appendChild(_e463);
  _e373.appendChild(_e460);
  const _e465 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e465);
  const _e466 = WF.h("hr", { className: "wf-divider" });
  _e373.appendChild(_e466);
  const _e467 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e467);
  const _e468 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Supported Features");
  _e373.appendChild(_e468);
  const _e469 = WF.h("p", { className: "wf-text" }, "Templates support the static, data-driven subset of WebFluent.");
  _e373.appendChild(_e469);
  const _e470 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e470);
  const _e471 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e472 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e473 = WF.h("div", { className: "wf-card__header" });
  const _e474 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Supported");
  _e473.appendChild(_e474);
  _e472.appendChild(_e473);
  const _e475 = WF.h("div", { className: "wf-card__body" });
  const _e476 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e477 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e478 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e477.appendChild(_e478);
  const _e479 = WF.h("p", { className: "wf-text" }, "All layout components");
  _e477.appendChild(_e479);
  _e476.appendChild(_e477);
  const _e480 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e481 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e480.appendChild(_e481);
  const _e482 = WF.h("p", { className: "wf-text" }, "Typography (Text, Heading, Code)");
  _e480.appendChild(_e482);
  _e476.appendChild(_e480);
  const _e483 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e484 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e483.appendChild(_e484);
  const _e485 = WF.h("p", { className: "wf-text" }, "Data display (Card, Table, List, Badge)");
  _e483.appendChild(_e485);
  _e476.appendChild(_e483);
  const _e486 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e487 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e486.appendChild(_e487);
  const _e488 = WF.h("p", { className: "wf-text" }, "for loops over data arrays");
  _e486.appendChild(_e488);
  _e476.appendChild(_e486);
  const _e489 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e490 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e489.appendChild(_e490);
  const _e491 = WF.h("p", { className: "wf-text" }, "if/else conditionals");
  _e489.appendChild(_e491);
  _e476.appendChild(_e489);
  const _e492 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e493 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e492.appendChild(_e493);
  const _e494 = WF.h("p", { className: "wf-text" }, "String interpolation {var}");
  _e492.appendChild(_e494);
  _e476.appendChild(_e492);
  const _e495 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e496 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e495.appendChild(_e496);
  const _e497 = WF.h("p", { className: "wf-text" }, "Nested access (user.name)");
  _e495.appendChild(_e497);
  _e476.appendChild(_e495);
  const _e498 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e499 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e498.appendChild(_e499);
  const _e500 = WF.h("p", { className: "wf-text" }, "Design tokens and themes");
  _e498.appendChild(_e500);
  _e476.appendChild(_e498);
  const _e501 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e502 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e501.appendChild(_e502);
  const _e503 = WF.h("p", { className: "wf-text" }, "Style blocks");
  _e501.appendChild(_e503);
  _e476.appendChild(_e501);
  const _e504 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e505 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e504.appendChild(_e505);
  const _e506 = WF.h("p", { className: "wf-text" }, "PDF components");
  _e504.appendChild(_e506);
  _e476.appendChild(_e504);
  _e475.appendChild(_e476);
  _e472.appendChild(_e475);
  _e471.appendChild(_e472);
  const _e507 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e508 = WF.h("div", { className: "wf-card__header" });
  const _e509 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Not Supported");
  _e508.appendChild(_e509);
  _e507.appendChild(_e508);
  const _e510 = WF.h("div", { className: "wf-card__body" });
  const _e511 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e512 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e513 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e512.appendChild(_e513);
  const _e514 = WF.h("p", { className: "wf-text" }, "state / derived / effect");
  _e512.appendChild(_e514);
  _e511.appendChild(_e512);
  const _e515 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e516 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e515.appendChild(_e516);
  const _e517 = WF.h("p", { className: "wf-text" }, "Events (on:click, on:submit)");
  _e515.appendChild(_e517);
  _e511.appendChild(_e515);
  const _e518 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e519 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e518.appendChild(_e519);
  const _e520 = WF.h("p", { className: "wf-text" }, "Navigation / Router");
  _e518.appendChild(_e520);
  _e511.appendChild(_e518);
  const _e521 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e522 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e521.appendChild(_e522);
  const _e523 = WF.h("p", { className: "wf-text" }, "Stores (shared state)");
  _e521.appendChild(_e523);
  _e511.appendChild(_e521);
  const _e524 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e525 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e524.appendChild(_e525);
  const _e526 = WF.h("p", { className: "wf-text" }, "fetch (data loading)");
  _e524.appendChild(_e526);
  _e511.appendChild(_e524);
  const _e527 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e528 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e527.appendChild(_e528);
  const _e529 = WF.h("p", { className: "wf-text" }, "Animations");
  _e527.appendChild(_e529);
  _e511.appendChild(_e527);
  const _e530 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e531 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e530.appendChild(_e531);
  const _e532 = WF.h("p", { className: "wf-text" }, "Toast (imperative)");
  _e530.appendChild(_e532);
  _e511.appendChild(_e530);
  _e510.appendChild(_e511);
  _e507.appendChild(_e510);
  _e471.appendChild(_e507);
  _e373.appendChild(_e471);
  const _e533 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e533);
  const _e534 = WF.h("hr", { className: "wf-divider" });
  _e373.appendChild(_e534);
  const _e535 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e535);
  const _e536 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Use Cases");
  _e373.appendChild(_e536);
  const _e537 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e537);
  const _e538 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e539 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e540 = WF.h("div", { className: "wf-card__body" });
  const _e541 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Server-Rendered Pages");
  _e540.appendChild(_e541);
  const _e542 = WF.h("p", { className: "wf-text wf-text--muted" }, "Generate HTML pages on the server with data from your database or API.");
  _e540.appendChild(_e542);
  _e539.appendChild(_e540);
  _e538.appendChild(_e539);
  const _e543 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e544 = WF.h("div", { className: "wf-card__body" });
  const _e545 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "PDF Reports");
  _e544.appendChild(_e545);
  const _e546 = WF.h("p", { className: "wf-text wf-text--muted" }, "Create invoices, receipts, and reports as PDF files from structured data.");
  _e544.appendChild(_e546);
  _e543.appendChild(_e544);
  _e538.appendChild(_e543);
  const _e547 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e548 = WF.h("div", { className: "wf-card__body" });
  const _e549 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Email Templates");
  _e548.appendChild(_e549);
  const _e550 = WF.h("p", { className: "wf-text wf-text--muted" }, "Render HTML emails with WebFluent components and your design system.");
  _e548.appendChild(_e550);
  _e547.appendChild(_e548);
  _e538.appendChild(_e547);
  _e373.appendChild(_e538);
  const _e551 = WF.h("div", { className: "wf-spacer" });
  _e373.appendChild(_e551);
  _root.appendChild(_e373);
  return _root;
}

function Page_Pdf(params) {
  const _root = document.createDocumentFragment();
  const _e552 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e553 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e553);
  const _e554 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "PDF Generation");
  _e552.appendChild(_e554);
  const _e555 = WF.h("p", { className: "wf-text wf-text--muted" }, "Generate PDF documents directly from .wf source files. No external dependencies — raw PDF 1.7 output.");
  _e552.appendChild(_e555);
  const _e556 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e556);
  const _e557 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Enable PDF Output");
  _e552.appendChild(_e557);
  const _e558 = WF.h("p", { className: "wf-text" }, "Set the output type to pdf in your project config.");
  _e552.appendChild(_e558);
  const _e559 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e559);
  const _e560 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e561 = WF.h("div", { className: "wf-card__body" });
  const _e562 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"build\": {\n    \"output_type\": \"pdf\",\n    \"pdf\": {\n      \"page_size\": \"A4\",\n      \"margins\": { \"top\": 72, \"bottom\": 72, \"left\": 72, \"right\": 72 },\n      \"default_font\": \"Helvetica\",\n      \"default_font_size\": 12,\n      \"output_filename\": \"report.pdf\"\n    }\n  }\n}");
  _e561.appendChild(_e562);
  _e560.appendChild(_e561);
  _e552.appendChild(_e560);
  const _e563 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e563);
  const _e564 = WF.h("hr", { className: "wf-divider" });
  _e552.appendChild(_e564);
  const _e565 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e565);
  const _e566 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Quick Start");
  _e552.appendChild(_e566);
  const _e567 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e568 = WF.h("div", { className: "wf-card__body" });
  const _e569 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-report --template pdf\ncd my-report\nwf build");
  _e568.appendChild(_e569);
  _e567.appendChild(_e568);
  _e552.appendChild(_e567);
  const _e570 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e570);
  const _e571 = WF.h("p", { className: "wf-text wf-text--muted" }, "This creates a sample PDF project and builds it to build/my-report.pdf.");
  _e552.appendChild(_e571);
  const _e572 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e572);
  const _e573 = WF.h("hr", { className: "wf-divider" });
  _e552.appendChild(_e573);
  const _e574 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e574);
  const _e575 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Document Structure");
  _e552.appendChild(_e575);
  const _e576 = WF.h("p", { className: "wf-text" }, "PDF documents use the same .wf syntax. Wrap content in a Document element with optional Header and Footer.");
  _e552.appendChild(_e576);
  const _e577 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e577);
  const _e578 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e579 = WF.h("div", { className: "wf-card__body" });
  const _e580 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Report (path: \"/\", title: \"Q1 Report\") {\n    Document(page_size: \"A4\") {\n        Header {\n            Text(\"Company Inc.\", muted, small, right)\n        }\n\n        Footer {\n            Text(\"Confidential\", muted, small, center)\n        }\n\n        Section {\n            Heading(\"Quarterly Report\", h1)\n            Text(\"Revenue grew 15% this quarter.\")\n\n            Table {\n                Thead {\n                    Trow {\n                        Tcell(\"Region\")\n                        Tcell(\"Revenue\")\n                    }\n                }\n                Tbody {\n                    Trow {\n                        Tcell(\"North America\")\n                        Tcell(\"$2.4M\")\n                    }\n                }\n            }\n\n            PageBreak()\n\n            Heading(\"Key Highlights\", h2)\n            List {\n                Text(\"Launched 3 new products\")\n                Text(\"Expanded to 5 new markets\")\n            }\n        }\n    }\n}");
  _e579.appendChild(_e580);
  _e578.appendChild(_e579);
  _e552.appendChild(_e578);
  const _e581 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e581);
  const _e582 = WF.h("hr", { className: "wf-divider" });
  _e552.appendChild(_e582);
  const _e583 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e583);
  const _e584 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Supported Components");
  _e552.appendChild(_e584);
  const _e585 = WF.h("p", { className: "wf-text" }, "These components render in PDF output:");
  _e552.appendChild(_e585);
  const _e586 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e586);
  const _e587 = WF.h("table", { className: "wf-table" });
  const _e588 = WF.h("thead", {});
  const _e589 = WF.h("td", {}, "Component");
  _e588.appendChild(_e589);
  const _e590 = WF.h("td", {}, "PDF Behavior");
  _e588.appendChild(_e590);
  _e587.appendChild(_e588);
  const _e591 = WF.h("tr", {});
  const _e592 = WF.h("td", {}, "Document");
  _e591.appendChild(_e592);
  const _e593 = WF.h("td", {}, "Root element. Sets page size via page_size arg.");
  _e591.appendChild(_e593);
  _e587.appendChild(_e591);
  const _e594 = WF.h("tr", {});
  const _e595 = WF.h("td", {}, "Header / Footer");
  _e594.appendChild(_e595);
  const _e596 = WF.h("td", {}, "Repeated on every page. Positioned in margins.");
  _e594.appendChild(_e596);
  _e587.appendChild(_e594);
  const _e597 = WF.h("tr", {});
  const _e598 = WF.h("td", {}, "Section");
  _e597.appendChild(_e598);
  const _e599 = WF.h("td", {}, "Groups content with spacing.");
  _e597.appendChild(_e599);
  _e587.appendChild(_e597);
  const _e600 = WF.h("tr", {});
  const _e601 = WF.h("td", {}, "Paragraph");
  _e600.appendChild(_e601);
  const _e602 = WF.h("td", {}, "Block of text with paragraph spacing.");
  _e600.appendChild(_e602);
  _e587.appendChild(_e600);
  const _e603 = WF.h("tr", {});
  const _e604 = WF.h("td", {}, "PageBreak");
  _e603.appendChild(_e604);
  const _e605 = WF.h("td", {}, "Forces a new page.");
  _e603.appendChild(_e605);
  _e587.appendChild(_e603);
  const _e606 = WF.h("tr", {});
  const _e607 = WF.h("td", {}, "Heading(text, h1..h6)");
  _e606.appendChild(_e607);
  const _e608 = WF.h("td", {}, "Bold heading. h1=28pt, h2=22pt, h3=18pt...");
  _e606.appendChild(_e608);
  _e587.appendChild(_e606);
  const _e609 = WF.h("tr", {});
  const _e610 = WF.h("td", {}, "Text(text)");
  _e609.appendChild(_e610);
  const _e611 = WF.h("td", {}, "Body text with word wrapping.");
  _e609.appendChild(_e611);
  _e587.appendChild(_e609);
  const _e612 = WF.h("tr", {});
  const _e613 = WF.h("td", {}, "Table / Thead / Tbody / Trow / Tcell");
  _e612.appendChild(_e613);
  const _e614 = WF.h("td", {}, "Gridded table with borders and header styling.");
  _e612.appendChild(_e614);
  _e587.appendChild(_e612);
  const _e615 = WF.h("tr", {});
  const _e616 = WF.h("td", {}, "List");
  _e615.appendChild(_e616);
  const _e617 = WF.h("td", {}, "Bulleted list. Add ordered modifier for numbered.");
  _e615.appendChild(_e617);
  _e587.appendChild(_e615);
  const _e618 = WF.h("tr", {});
  const _e619 = WF.h("td", {}, "Code(text, block)");
  _e618.appendChild(_e619);
  const _e620 = WF.h("td", {}, "Monospace code with gray background.");
  _e618.appendChild(_e620);
  _e587.appendChild(_e618);
  const _e621 = WF.h("tr", {});
  const _e622 = WF.h("td", {}, "Blockquote");
  _e621.appendChild(_e622);
  const _e623 = WF.h("td", {}, "Indented text with left bar.");
  _e621.appendChild(_e623);
  _e587.appendChild(_e621);
  const _e624 = WF.h("tr", {});
  const _e625 = WF.h("td", {}, "Divider");
  _e624.appendChild(_e625);
  const _e626 = WF.h("td", {}, "Horizontal line.");
  _e624.appendChild(_e626);
  _e587.appendChild(_e624);
  const _e627 = WF.h("tr", {});
  const _e628 = WF.h("td", {}, "Alert(text, variant)");
  _e627.appendChild(_e628);
  const _e629 = WF.h("td", {}, "Colored box with left accent bar.");
  _e627.appendChild(_e629);
  _e587.appendChild(_e627);
  const _e630 = WF.h("tr", {});
  const _e631 = WF.h("td", {}, "Badge / Tag");
  _e630.appendChild(_e631);
  const _e632 = WF.h("td", {}, "Colored pill with white text.");
  _e630.appendChild(_e632);
  _e587.appendChild(_e630);
  const _e633 = WF.h("tr", {});
  const _e634 = WF.h("td", {}, "Progress(value, max)");
  _e633.appendChild(_e634);
  const _e635 = WF.h("td", {}, "Horizontal bar.");
  _e633.appendChild(_e635);
  _e587.appendChild(_e633);
  const _e636 = WF.h("tr", {});
  const _e637 = WF.h("td", {}, "Card");
  _e636.appendChild(_e637);
  const _e638 = WF.h("td", {}, "Bordered box around children.");
  _e636.appendChild(_e638);
  _e587.appendChild(_e636);
  const _e639 = WF.h("tr", {});
  const _e640 = WF.h("td", {}, "Image(src)");
  _e639.appendChild(_e640);
  const _e641 = WF.h("td", {}, "Placeholder rectangle (JPEG planned).");
  _e639.appendChild(_e641);
  _e587.appendChild(_e639);
  const _e642 = WF.h("tr", {});
  const _e643 = WF.h("td", {}, "Spacer");
  _e642.appendChild(_e643);
  const _e644 = WF.h("td", {}, "Vertical space. Modifiers: sm, md, lg, xl.");
  _e642.appendChild(_e644);
  _e587.appendChild(_e642);
  _e552.appendChild(_e587);
  const _e645 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e645);
  const _e646 = WF.h("hr", { className: "wf-divider" });
  _e552.appendChild(_e646);
  const _e647 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e647);
  const _e648 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Rejected Components");
  _e552.appendChild(_e648);
  const _e649 = WF.h("p", { className: "wf-text" }, "Interactive and web-only components cause compile-time errors in PDF mode:");
  _e552.appendChild(_e649);
  const _e650 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e650);
  const _e651 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e652 = WF.h("div", { className: "wf-card__body" });
  const _e653 = WF.h("code", { className: "wf-code wf-code--block" }, "error[pdf]: 'Button' cannot be used in PDF output (Page Report)\n  — interactive elements are not supported in PDF\n\nerror[pdf]: 'Input' cannot be used in PDF output (Page Report)\n  — interactive elements are not supported in PDF");
  _e652.appendChild(_e653);
  _e651.appendChild(_e652);
  _e552.appendChild(_e651);
  const _e654 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e654);
  const _e655 = WF.h("p", { className: "wf-text wf-text--muted" }, "Rejected: Button, Input, Select, Checkbox, Switch, Slider, Form, Modal, Dialog, Toast, Router, Navbar, Sidebar, Tabs, Video, Carousel, and all event handlers.");
  _e552.appendChild(_e655);
  const _e656 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e656);
  const _e657 = WF.h("hr", { className: "wf-divider" });
  _e552.appendChild(_e657);
  const _e658 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e658);
  const _e659 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Page Sizes");
  _e552.appendChild(_e659);
  const _e660 = WF.h("table", { className: "wf-table" });
  const _e661 = WF.h("thead", {});
  const _e662 = WF.h("td", {}, "Value");
  _e661.appendChild(_e662);
  const _e663 = WF.h("td", {}, "Dimensions (points)");
  _e661.appendChild(_e663);
  const _e664 = WF.h("td", {}, "Dimensions (mm)");
  _e661.appendChild(_e664);
  _e660.appendChild(_e661);
  const _e665 = WF.h("tr", {});
  const _e666 = WF.h("td", {}, "A4");
  _e665.appendChild(_e666);
  const _e667 = WF.h("td", {}, "595 x 842");
  _e665.appendChild(_e667);
  const _e668 = WF.h("td", {}, "210 x 297");
  _e665.appendChild(_e668);
  _e660.appendChild(_e665);
  const _e669 = WF.h("tr", {});
  const _e670 = WF.h("td", {}, "A3");
  _e669.appendChild(_e670);
  const _e671 = WF.h("td", {}, "842 x 1191");
  _e669.appendChild(_e671);
  const _e672 = WF.h("td", {}, "297 x 420");
  _e669.appendChild(_e672);
  _e660.appendChild(_e669);
  const _e673 = WF.h("tr", {});
  const _e674 = WF.h("td", {}, "A5");
  _e673.appendChild(_e674);
  const _e675 = WF.h("td", {}, "420 x 595");
  _e673.appendChild(_e675);
  const _e676 = WF.h("td", {}, "148 x 210");
  _e673.appendChild(_e676);
  _e660.appendChild(_e673);
  const _e677 = WF.h("tr", {});
  const _e678 = WF.h("td", {}, "Letter");
  _e677.appendChild(_e678);
  const _e679 = WF.h("td", {}, "612 x 792");
  _e677.appendChild(_e679);
  const _e680 = WF.h("td", {}, "216 x 279");
  _e677.appendChild(_e680);
  _e660.appendChild(_e677);
  const _e681 = WF.h("tr", {});
  const _e682 = WF.h("td", {}, "Legal");
  _e681.appendChild(_e682);
  const _e683 = WF.h("td", {}, "612 x 1008");
  _e681.appendChild(_e683);
  const _e684 = WF.h("td", {}, "216 x 356");
  _e681.appendChild(_e684);
  _e660.appendChild(_e681);
  _e552.appendChild(_e660);
  const _e685 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e685);
  const _e686 = WF.h("hr", { className: "wf-divider" });
  _e552.appendChild(_e686);
  const _e687 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e687);
  const _e688 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Fonts");
  _e552.appendChild(_e688);
  const _e689 = WF.h("p", { className: "wf-text" }, "PDF output uses the 14 standard PDF base fonts. No embedding needed.");
  _e552.appendChild(_e689);
  const _e690 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e690);
  const _e691 = WF.h("table", { className: "wf-table" });
  const _e692 = WF.h("thead", {});
  const _e693 = WF.h("td", {}, "Font Family");
  _e692.appendChild(_e693);
  const _e694 = WF.h("td", {}, "Variants");
  _e692.appendChild(_e694);
  _e691.appendChild(_e692);
  const _e695 = WF.h("tr", {});
  const _e696 = WF.h("td", {}, "Helvetica");
  _e695.appendChild(_e696);
  const _e697 = WF.h("td", {}, "Regular, Bold, Oblique, BoldOblique");
  _e695.appendChild(_e697);
  _e691.appendChild(_e695);
  const _e698 = WF.h("tr", {});
  const _e699 = WF.h("td", {}, "Times");
  _e698.appendChild(_e699);
  const _e700 = WF.h("td", {}, "Roman, Bold, Italic, BoldItalic");
  _e698.appendChild(_e700);
  _e691.appendChild(_e698);
  const _e701 = WF.h("tr", {});
  const _e702 = WF.h("td", {}, "Courier");
  _e701.appendChild(_e702);
  const _e703 = WF.h("td", {}, "Regular, Bold, Oblique, BoldOblique");
  _e701.appendChild(_e703);
  _e691.appendChild(_e701);
  _e552.appendChild(_e691);
  const _e704 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e704);
  const _e705 = WF.h("p", { className: "wf-text wf-text--muted" }, "Set the default font in config or override per-element with style blocks:");
  _e552.appendChild(_e705);
  const _e706 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e706);
  const _e707 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e708 = WF.h("div", { className: "wf-card__body" });
  const _e709 = WF.h("code", { className: "wf-code wf-code--block" }, "Heading(\"Title\", h1) {\n    style {\n        font-family: \"Helvetica-Bold\"\n        color: \"#1a1a2e\"\n    }\n}");
  _e708.appendChild(_e709);
  _e707.appendChild(_e708);
  _e552.appendChild(_e707);
  const _e710 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e710);
  const _e711 = WF.h("hr", { className: "wf-divider" });
  _e552.appendChild(_e711);
  const _e712 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e712);
  const _e713 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Styling in PDF");
  _e552.appendChild(_e713);
  const _e714 = WF.h("p", { className: "wf-text" }, "Style blocks support these properties in PDF output:");
  _e552.appendChild(_e714);
  const _e715 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e715);
  const _e716 = WF.h("table", { className: "wf-table" });
  const _e717 = WF.h("thead", {});
  const _e718 = WF.h("td", {}, "Property");
  _e717.appendChild(_e718);
  const _e719 = WF.h("td", {}, "Values");
  _e717.appendChild(_e719);
  const _e720 = WF.h("td", {}, "Example");
  _e717.appendChild(_e720);
  _e716.appendChild(_e717);
  const _e721 = WF.h("tr", {});
  const _e722 = WF.h("td", {}, "font-size");
  _e721.appendChild(_e722);
  const _e723 = WF.h("td", {}, "Number (points)");
  _e721.appendChild(_e723);
  const _e724 = WF.h("td", {}, "font-size: 14");
  _e721.appendChild(_e724);
  _e716.appendChild(_e721);
  const _e725 = WF.h("tr", {});
  const _e726 = WF.h("td", {}, "font-family");
  _e725.appendChild(_e726);
  const _e727 = WF.h("td", {}, "Base14 font name");
  _e725.appendChild(_e727);
  const _e728 = WF.h("td", {}, "font-family: \"Courier\"");
  _e725.appendChild(_e728);
  _e716.appendChild(_e725);
  const _e729 = WF.h("tr", {});
  const _e730 = WF.h("td", {}, "color");
  _e729.appendChild(_e730);
  const _e731 = WF.h("td", {}, "Hex color");
  _e729.appendChild(_e731);
  const _e732 = WF.h("td", {}, "color: \"#333333\"");
  _e729.appendChild(_e732);
  _e716.appendChild(_e729);
  const _e733 = WF.h("tr", {});
  const _e734 = WF.h("td", {}, "text-align");
  _e733.appendChild(_e734);
  const _e735 = WF.h("td", {}, "left, center, right");
  _e733.appendChild(_e735);
  const _e736 = WF.h("td", {}, "text-align: \"center\"");
  _e733.appendChild(_e736);
  _e716.appendChild(_e733);
  _e552.appendChild(_e716);
  const _e737 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e737);
  const _e738 = WF.h("p", { className: "wf-text wf-text--muted" }, "Modifiers also work: bold, muted, primary, danger, success, warning, info, small, large, center, right.");
  _e552.appendChild(_e738);
  const _e739 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e739);
  const _e740 = WF.h("hr", { className: "wf-divider" });
  _e552.appendChild(_e740);
  const _e741 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e741);
  const _e742 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Auto Page Breaks");
  _e552.appendChild(_e742);
  const _e743 = WF.h("p", { className: "wf-text wf-text--muted" }, "Content automatically flows to a new page when it reaches the bottom margin. Headers and footers are rendered on every page, including auto-generated ones.");
  _e552.appendChild(_e743);
  const _e744 = WF.h("div", { className: "wf-spacer" });
  _e552.appendChild(_e744);
  _root.appendChild(_e552);
  return _root;
}

function Page_Guide(params) {
  const _root = document.createDocumentFragment();
  const _e745 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e746 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e746);
  const _e747 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Language Guide");
  _e745.appendChild(_e747);
  const _e748 = WF.h("p", { className: "wf-text wf-text--muted" }, "Learn the core concepts of WebFluent.");
  _e745.appendChild(_e748);
  const _e749 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e749);
  const _e750 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Pages");
  _e745.appendChild(_e750);
  const _e751 = WF.h("p", { className: "wf-text" }, "Pages are top-level route targets. Each page defines a URL path and contains the UI tree for that route.");
  _e745.appendChild(_e751);
  const _e752 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e752);
  const _e753 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e754 = WF.h("div", { className: "wf-card__body" });
  const _e755 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\", title: \"Home\") {\n    Container {\n        Heading(\"Welcome\", h1)\n        Text(\"This is the home page.\")\n    }\n}");
  _e754.appendChild(_e755);
  _e753.appendChild(_e754);
  _e745.appendChild(_e753);
  const _e756 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e756);
  const _e757 = WF.h("p", { className: "wf-text wf-text--bold" }, "Page attributes:");
  _e745.appendChild(_e757);
  const _e758 = WF.h("table", { className: "wf-table" });
  const _e759 = WF.h("thead", {});
  const _e760 = WF.h("td", {}, "Attribute");
  _e759.appendChild(_e760);
  const _e761 = WF.h("td", {}, "Type");
  _e759.appendChild(_e761);
  const _e762 = WF.h("td", {}, "Description");
  _e759.appendChild(_e762);
  _e758.appendChild(_e759);
  const _e763 = WF.h("tr", {});
  const _e764 = WF.h("td", {}, "path");
  _e763.appendChild(_e764);
  const _e765 = WF.h("td", {}, "String");
  _e763.appendChild(_e765);
  const _e766 = WF.h("td", {}, "URL route for this page (required)");
  _e763.appendChild(_e766);
  _e758.appendChild(_e763);
  const _e767 = WF.h("tr", {});
  const _e768 = WF.h("td", {}, "title");
  _e767.appendChild(_e768);
  const _e769 = WF.h("td", {}, "String");
  _e767.appendChild(_e769);
  const _e770 = WF.h("td", {}, "Document title");
  _e767.appendChild(_e770);
  _e758.appendChild(_e767);
  const _e771 = WF.h("tr", {});
  const _e772 = WF.h("td", {}, "guard");
  _e771.appendChild(_e772);
  const _e773 = WF.h("td", {}, "Expression");
  _e771.appendChild(_e773);
  const _e774 = WF.h("td", {}, "Navigation guard — redirects if false");
  _e771.appendChild(_e774);
  _e758.appendChild(_e771);
  const _e775 = WF.h("tr", {});
  const _e776 = WF.h("td", {}, "redirect");
  _e775.appendChild(_e776);
  const _e777 = WF.h("td", {}, "String");
  _e775.appendChild(_e777);
  const _e778 = WF.h("td", {}, "Redirect target when guard fails");
  _e775.appendChild(_e778);
  _e758.appendChild(_e775);
  _e745.appendChild(_e758);
  const _e779 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e779);
  const _e780 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e780);
  const _e781 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e781);
  const _e782 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Components");
  _e745.appendChild(_e782);
  const _e783 = WF.h("p", { className: "wf-text" }, "Reusable UI blocks that accept props and can have internal state.");
  _e745.appendChild(_e783);
  const _e784 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e784);
  const _e785 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e786 = WF.h("div", { className: "wf-card__body" });
  const _e787 = WF.h("code", { className: "wf-code wf-code--block" }, "Component UserCard (name: String, role: String, active: Bool = true) {\n    Card(elevated) {\n        Row(align: center, gap: md) {\n            Avatar(initials: \"U\", primary)\n            Stack {\n                Text(name, bold)\n                Text(role, muted)\n            }\n            if active {\n                Badge(\"Active\", success)\n            }\n        }\n    }\n}\n\n// Usage\nUserCard(name: \"Monzer\", role: \"Developer\")");
  _e786.appendChild(_e787);
  _e785.appendChild(_e786);
  _e745.appendChild(_e785);
  const _e788 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e788);
  const _e789 = WF.h("p", { className: "wf-text wf-text--muted" }, "Props support types: String, Number, Bool, List, Map. Optional props use ?, defaults use =.");
  _e745.appendChild(_e789);
  const _e790 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e790);
  const _e791 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e791);
  const _e792 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e792);
  const _e793 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "State and Reactivity");
  _e745.appendChild(_e793);
  const _e794 = WF.h("p", { className: "wf-text" }, "State is declared with the state keyword. It is reactive — any UI that reads it updates automatically when it changes.");
  _e745.appendChild(_e794);
  const _e795 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e795);
  const _e796 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e797 = WF.h("div", { className: "wf-card__body" });
  const _e798 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Counter (path: \"/counter\") {\n    state count = 0\n\n    Container {\n        Text(\"Count: {count}\")\n        Button(\"+1\", primary) { count = count + 1 }\n        Button(\"-1\") { count = count - 1 }\n    }\n}");
  _e797.appendChild(_e798);
  _e796.appendChild(_e797);
  _e745.appendChild(_e796);
  const _e799 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e799);
  const _e800 = WF.h("p", { className: "wf-text wf-text--bold" }, "Derived state:");
  _e745.appendChild(_e800);
  const _e801 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e802 = WF.h("div", { className: "wf-card__body" });
  const _e803 = WF.h("code", { className: "wf-code wf-code--block" }, "state items = [{name: \"A\", price: 3}, {name: \"B\", price: 2}]\nderived total = items.map(i => i.price).sum()\nderived isEmpty = items.length == 0");
  _e802.appendChild(_e803);
  _e801.appendChild(_e802);
  _e745.appendChild(_e801);
  const _e804 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e804);
  const _e805 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e805);
  const _e806 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e806);
  const _e807 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Events");
  _e745.appendChild(_e807);
  const _e808 = WF.h("p", { className: "wf-text" }, "Event handlers are declared with on:event or via shorthand blocks on buttons.");
  _e745.appendChild(_e808);
  const _e809 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e809);
  const _e810 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e811 = WF.h("div", { className: "wf-card__body" });
  const _e812 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Submit\") {\n    on:click {\n        submitForm()\n    }\n}\n\nInput(text, placeholder: \"Search...\") {\n    on:input {\n        searchQuery = value\n    }\n    on:keydown {\n        if key == \"Enter\" {\n            performSearch()\n        }\n    }\n}\n\n// Shorthand: block on Button defaults to on:click\nButton(\"Save\") { save() }");
  _e811.appendChild(_e812);
  _e810.appendChild(_e811);
  _e745.appendChild(_e810);
  const _e813 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e813);
  const _e814 = WF.h("p", { className: "wf-text wf-text--muted" }, "Supported events: on:click, on:submit, on:input, on:change, on:focus, on:blur, on:keydown, on:keyup, on:mouseover, on:mouseout, on:mount, on:unmount");
  _e745.appendChild(_e814);
  const _e815 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e815);
  const _e816 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e816);
  const _e817 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e817);
  const _e818 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Control Flow");
  _e745.appendChild(_e818);
  const _e819 = WF.h("p", { className: "wf-text wf-text--bold" }, "Conditionals:");
  _e745.appendChild(_e819);
  const _e820 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e821 = WF.h("div", { className: "wf-card__body" });
  const _e822 = WF.h("code", { className: "wf-code wf-code--block" }, "if isLoggedIn {\n    Text(\"Welcome back!\")\n} else if isGuest {\n    Text(\"Hello, guest\")\n} else {\n    Button(\"Log In\") { navigate(\"/login\") }\n}");
  _e821.appendChild(_e822);
  _e820.appendChild(_e821);
  _e745.appendChild(_e820);
  const _e823 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e823);
  const _e824 = WF.h("p", { className: "wf-text wf-text--bold" }, "Loops:");
  _e745.appendChild(_e824);
  const _e825 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e826 = WF.h("div", { className: "wf-card__body" });
  const _e827 = WF.h("code", { className: "wf-code wf-code--block" }, "for user in users {\n    UserCard(name: user.name, role: user.role)\n}\n\n// With index\nfor item, index in items {\n    Text(\"{index + 1}. {item}\")\n}");
  _e826.appendChild(_e827);
  _e825.appendChild(_e826);
  _e745.appendChild(_e825);
  const _e828 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e828);
  const _e829 = WF.h("p", { className: "wf-text wf-text--bold" }, "Show/Hide (keeps element in DOM, toggles visibility):");
  _e745.appendChild(_e829);
  const _e830 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e831 = WF.h("div", { className: "wf-card__body" });
  const _e832 = WF.h("code", { className: "wf-code wf-code--block" }, "show isExpanded {\n    Card { Text(\"Expanded content\") }\n}");
  _e831.appendChild(_e832);
  _e830.appendChild(_e831);
  _e745.appendChild(_e830);
  const _e833 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e833);
  const _e834 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e834);
  const _e835 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e835);
  const _e836 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Stores");
  _e745.appendChild(_e836);
  const _e837 = WF.h("p", { className: "wf-text" }, "Stores hold shared state accessible from any page or component.");
  _e745.appendChild(_e837);
  const _e838 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e838);
  const _e839 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e840 = WF.h("div", { className: "wf-card__body" });
  const _e841 = WF.h("code", { className: "wf-code wf-code--block" }, "Store CartStore {\n    state items = []\n\n    derived total = items.map(i => i.price * i.quantity).sum()\n    derived count = items.length\n\n    action addItem(product: Map) {\n        items.push({ id: product.id, name: product.name, price: product.price, quantity: 1 })\n    }\n\n    action removeItem(id: Number) {\n        items = items.filter(i => i.id != id)\n    }\n}\n\n// Usage in a page\nPage Cart (path: \"/cart\") {\n    use CartStore\n\n    Text(\"Total: ${CartStore.total}\")\n    Button(\"Clear\") { CartStore.clear() }\n}");
  _e840.appendChild(_e841);
  _e839.appendChild(_e840);
  _e745.appendChild(_e839);
  const _e842 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e842);
  const _e843 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e843);
  const _e844 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e844);
  const _e845 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Routing");
  _e745.appendChild(_e845);
  const _e846 = WF.h("p", { className: "wf-text" }, "SPA routing is declared in the App file.");
  _e745.appendChild(_e846);
  const _e847 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e847);
  const _e848 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e849 = WF.h("div", { className: "wf-card__body" });
  const _e850 = WF.h("code", { className: "wf-code wf-code--block" }, "App {\n    Navbar {\n        Navbar.Brand { Text(\"My App\", heading) }\n        Navbar.Links {\n            Link(to: \"/\") { Text(\"Home\") }\n            Link(to: \"/about\") { Text(\"About\") }\n        }\n    }\n\n    Router {\n        Route(path: \"/\", page: Home)\n        Route(path: \"/about\", page: About)\n        Route(path: \"/user/:id\", page: UserProfile)\n        Route(path: \"*\", page: NotFound)\n    }\n}\n\n// Programmatic navigation\nButton(\"Go Home\") { navigate(\"/\") }\n\n// Dynamic routes access params\nPage UserProfile (path: \"/user/:id\") {\n    Text(\"User ID: {params.id}\")\n}");
  _e849.appendChild(_e850);
  _e848.appendChild(_e849);
  _e745.appendChild(_e848);
  const _e851 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e851);
  const _e852 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e852);
  const _e853 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e853);
  const _e854 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Fetching");
  _e745.appendChild(_e854);
  const _e855 = WF.h("p", { className: "wf-text" }, "Built-in async data loading with automatic loading, error, and success states.");
  _e745.appendChild(_e855);
  const _e856 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e856);
  const _e857 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e858 = WF.h("div", { className: "wf-card__body" });
  const _e859 = WF.h("code", { className: "wf-code wf-code--block" }, "fetch users from \"/api/users\" {\n    loading {\n        Spinner()\n    }\n    error (err) {\n        Alert(\"Failed to load users\", danger)\n    }\n    success {\n        for user in users {\n            UserCard(name: user.name, role: user.role)\n        }\n    }\n}\n\n// With options\nfetch result from \"/api/submit\" (method: \"POST\", body: { name: name, email: email }) {\n    success {\n        Alert(\"Saved!\", success)\n    }\n}");
  _e858.appendChild(_e859);
  _e857.appendChild(_e858);
  _e745.appendChild(_e857);
  const _e860 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e860);
  const _e861 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e861);
  const _e862 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e862);
  const _e863 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Return Values");
  _e745.appendChild(_e863);
  const _e864 = WF.h("p", { className: "wf-text" }, "Store actions can return values using the return keyword.");
  _e745.appendChild(_e864);
  const _e865 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e865);
  const _e866 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e867 = WF.h("div", { className: "wf-card__body" });
  const _e868 = WF.h("code", { className: "wf-code wf-code--block" }, "Store AuthStore {\n    state accessToken = \"\"\n\n    action getHeaders() {\n        h = {}\n        h[\"Authorization\"] = \"Bearer \" + accessToken\n        return h\n    }\n}");
  _e867.appendChild(_e868);
  _e866.appendChild(_e867);
  _e745.appendChild(_e866);
  const _e869 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e869);
  const _e870 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e870);
  const _e871 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e871);
  const _e872 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Browser Globals");
  _e745.appendChild(_e872);
  const _e873 = WF.h("p", { className: "wf-text" }, "Standard browser APIs are available directly without any special syntax. They compile to their JavaScript equivalents.");
  _e745.appendChild(_e873);
  const _e874 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e874);
  const _e875 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e876 = WF.h("div", { className: "wf-card__body" });
  const _e877 = WF.h("code", { className: "wf-code wf-code--block" }, "// Storage\nlocalStorage.setItem(\"token\", tok)\nsessionStorage.getItem(\"key\")\n\n// Window & Document\nwindow.open(\"https://example.com\")\n\n// JSON\ndata = JSON.parse(responseText)\ntext = JSON.stringify(obj)\n\n// Console\nconsole.log(\"debug info\")\n\n// Timers\nsetTimeout(callback, 1000)");
  _e876.appendChild(_e877);
  _e875.appendChild(_e876);
  _e745.appendChild(_e875);
  const _e878 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e878);
  const _e879 = WF.h("p", { className: "wf-text wf-text--muted" }, "Available globals: window, document, console, localStorage, sessionStorage, JSON, Math, Date, setTimeout, setInterval, parseInt, parseFloat, Array, Object, Promise, Error, fetch, alert, confirm, prompt, and more.");
  _e745.appendChild(_e879);
  const _e880 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e880);
  const _e881 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e881);
  const _e882 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e882);
  const _e883 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Map Literals");
  _e745.appendChild(_e883);
  const _e884 = WF.h("p", { className: "wf-text" }, "Map literals support quoted string keys for HTTP headers and special field names. Reserved words also work as map keys.");
  _e745.appendChild(_e884);
  const _e885 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e885);
  const _e886 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e887 = WF.h("div", { className: "wf-card__body" });
  const _e888 = WF.h("code", { className: "wf-code wf-code--block" }, "// Quoted keys for headers\nheaders: { \"Content-Type\": \"application/json\", \"X-Api-Key\": apiKey }\n\n// Reserved words as keys\nbody: { action: \"create\", token: sessionToken, state: \"active\" }");
  _e887.appendChild(_e888);
  _e886.appendChild(_e887);
  _e745.appendChild(_e886);
  const _e889 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e889);
  const _e890 = WF.h("hr", { className: "wf-divider" });
  _e745.appendChild(_e890);
  const _e891 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e891);
  const _e892 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Operators");
  _e745.appendChild(_e892);
  const _e893 = WF.h("p", { className: "wf-text" }, "WebFluent supports all common comparison and logical operators.");
  _e745.appendChild(_e893);
  const _e894 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e894);
  const _e895 = WF.h("table", { className: "wf-table" });
  const _e896 = WF.h("thead", {});
  const _e897 = WF.h("td", {}, "Operator");
  _e896.appendChild(_e897);
  const _e898 = WF.h("td", {}, "Description");
  _e896.appendChild(_e898);
  _e895.appendChild(_e896);
  const _e899 = WF.h("tr", {});
  const _e900 = WF.h("td", {}, "==");
  _e899.appendChild(_e900);
  const _e901 = WF.h("td", {}, "Equal");
  _e899.appendChild(_e901);
  _e895.appendChild(_e899);
  const _e902 = WF.h("tr", {});
  const _e903 = WF.h("td", {}, "!=");
  _e902.appendChild(_e903);
  const _e904 = WF.h("td", {}, "Not equal");
  _e902.appendChild(_e904);
  _e895.appendChild(_e902);
  const _e905 = WF.h("tr", {});
  const _e906 = WF.h("td", {}, "!==");
  _e905.appendChild(_e906);
  const _e907 = WF.h("td", {}, "Strict not equal (alias for !=)");
  _e905.appendChild(_e907);
  _e895.appendChild(_e905);
  const _e908 = WF.h("tr", {});
  const _e909 = WF.h("td", {}, "< > <= >=");
  _e908.appendChild(_e909);
  const _e910 = WF.h("td", {}, "Comparison");
  _e908.appendChild(_e910);
  _e895.appendChild(_e908);
  const _e911 = WF.h("tr", {});
  const _e912 = WF.h("td", {}, "&& ||");
  _e911.appendChild(_e912);
  const _e913 = WF.h("td", {}, "Logical AND / OR");
  _e911.appendChild(_e913);
  _e895.appendChild(_e911);
  const _e914 = WF.h("tr", {});
  const _e915 = WF.h("td", {}, "!");
  _e914.appendChild(_e915);
  const _e916 = WF.h("td", {}, "Logical NOT");
  _e914.appendChild(_e916);
  _e895.appendChild(_e914);
  const _e917 = WF.h("tr", {});
  const _e918 = WF.h("td", {}, "+ - * / %");
  _e917.appendChild(_e918);
  const _e919 = WF.h("td", {}, "Arithmetic");
  _e917.appendChild(_e919);
  _e895.appendChild(_e917);
  _e745.appendChild(_e895);
  const _e920 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e920);
  const _e921 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e922 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/components"); } }, "Components Reference");
  _e921.appendChild(_e922);
  const _e923 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e921.appendChild(_e923);
  _e745.appendChild(_e921);
  const _e924 = WF.h("div", { className: "wf-spacer" });
  _e745.appendChild(_e924);
  _root.appendChild(_e745);
  return _root;
}

function Page_Animation(params) {
  const _showCard = WF.signal(false);
  const _root = document.createDocumentFragment();
  const _e925 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e926 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e926);
  const _e927 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Animation System");
  _e925.appendChild(_e927);
  const _e928 = WF.h("p", { className: "wf-text wf-text--muted" }, "Declarative animations built into the language. No CSS keyframes to write.");
  _e925.appendChild(_e928);
  const _e929 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e929);
  const _e930 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Mount Animations");
  _e925.appendChild(_e930);
  const _e931 = WF.h("p", { className: "wf-text" }, "Add an animation modifier to any component. It plays when the element appears. Hover each card to replay.");
  _e925.appendChild(_e931);
  const _e932 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e932);
  const _e933 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e934 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn"); } });
  const _e935 = WF.h("div", { className: "wf-card__body" });
  const _e936 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "fadeIn");
  _e935.appendChild(_e936);
  const _e937 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Fades from transparent");
  _e935.appendChild(_e937);
  _e934.appendChild(_e935);
  _e933.appendChild(_e934);
  const _e938 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideUp", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideUp"); } });
  const _e939 = WF.h("div", { className: "wf-card__body" });
  const _e940 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideUp");
  _e939.appendChild(_e940);
  const _e941 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from below");
  _e939.appendChild(_e941);
  _e938.appendChild(_e939);
  _e933.appendChild(_e938);
  const _e942 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-scaleIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "scaleIn"); } });
  const _e943 = WF.h("div", { className: "wf-card__body" });
  const _e944 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "scaleIn");
  _e943.appendChild(_e944);
  const _e945 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Scales from 90%");
  _e943.appendChild(_e945);
  _e942.appendChild(_e943);
  _e933.appendChild(_e942);
  const _e946 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideDown", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideDown"); } });
  const _e947 = WF.h("div", { className: "wf-card__body" });
  const _e948 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideDown");
  _e947.appendChild(_e948);
  const _e949 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from above");
  _e947.appendChild(_e949);
  _e946.appendChild(_e947);
  _e933.appendChild(_e946);
  const _e950 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideLeft", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideLeft"); } });
  const _e951 = WF.h("div", { className: "wf-card__body" });
  const _e952 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideLeft");
  _e951.appendChild(_e952);
  const _e953 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from right");
  _e951.appendChild(_e953);
  _e950.appendChild(_e951);
  _e933.appendChild(_e950);
  const _e954 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-bounce", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "bounce"); } });
  const _e955 = WF.h("div", { className: "wf-card__body" });
  const _e956 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "bounce");
  _e955.appendChild(_e956);
  const _e957 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Bouncy entrance");
  _e955.appendChild(_e957);
  _e954.appendChild(_e955);
  _e933.appendChild(_e954);
  const _e958 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-shake", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "shake"); } });
  const _e959 = WF.h("div", { className: "wf-card__body" });
  const _e960 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "shake");
  _e959.appendChild(_e960);
  const _e961 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Horizontal shake");
  _e959.appendChild(_e961);
  _e958.appendChild(_e959);
  _e933.appendChild(_e958);
  const _e962 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-pulse", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "pulse"); } });
  const _e963 = WF.h("div", { className: "wf-card__body" });
  const _e964 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "pulse");
  _e963.appendChild(_e964);
  const _e965 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Gentle scale pulse");
  _e963.appendChild(_e965);
  _e962.appendChild(_e963);
  _e933.appendChild(_e962);
  const _e966 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideRight", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideRight"); } });
  const _e967 = WF.h("div", { className: "wf-card__body" });
  const _e968 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideRight");
  _e967.appendChild(_e968);
  const _e969 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from left");
  _e967.appendChild(_e969);
  _e966.appendChild(_e967);
  _e933.appendChild(_e966);
  _e925.appendChild(_e933);
  const _e970 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e970);
  const _e971 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e972 = WF.h("div", { className: "wf-card__body" });
  const _e973 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn) { ... }\nHeading(\"Title\", h1, slideUp)\nButton(\"Click\", primary, bounce)");
  _e972.appendChild(_e973);
  _e971.appendChild(_e972);
  _e925.appendChild(_e971);
  const _e974 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e974);
  const _e975 = WF.h("hr", { className: "wf-divider" });
  _e925.appendChild(_e975);
  const _e976 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e976);
  const _e977 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Live: Conditional Animation");
  _e925.appendChild(_e977);
  const _e978 = WF.h("p", { className: "wf-text" }, "Toggle the switch to see enter/exit animations on the card below.");
  _e925.appendChild(_e978);
  const _e979 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e979);
  const _e980 = WF.h("label", { className: "wf-switch" });
  const _e981 = WF.h("input", { type: "checkbox", checked: () => _showCard(), "on:change": () => _showCard.set(!_showCard()) });
  _e980.appendChild(_e981);
  const _e982 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e980.appendChild(_e982);
  _e980.appendChild(WF.text("Show animated card"));
  _e925.appendChild(_e980);
  const _e983 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e983);
  WF.condRender(_e925,
    () => _showCard(),
    () => {
      const _e984 = document.createDocumentFragment();
      const _e985 = WF.h("div", { className: "wf-card wf-card--elevated" });
      const _e986 = WF.h("div", { className: "wf-card__body" });
      const _e987 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Animated!");
      _e986.appendChild(_e987);
      const _e988 = WF.h("div", { className: "wf-spacer" });
      _e986.appendChild(_e988);
      const _e989 = WF.h("p", { className: "wf-text" }, "This card scales in and fades out.");
      _e986.appendChild(_e989);
      const _e990 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Controlled by: if showCard, animate(scaleIn, fadeOut)");
      _e986.appendChild(_e990);
      _e985.appendChild(_e986);
      _e984.appendChild(_e985);
      return _e984;
    },
    null,
    { enter: "scaleIn", exit: "fadeOut" }
  );
  const _e991 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e991);
  const _e992 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e993 = WF.h("div", { className: "wf-card__body" });
  const _e994 = WF.h("code", { className: "wf-code wf-code--block" }, "if showCard, animate(scaleIn, fadeOut) {\n    Card(elevated) {\n        Text(\"Animated content\")\n    }\n}");
  _e993.appendChild(_e994);
  _e992.appendChild(_e993);
  _e925.appendChild(_e992);
  const _e995 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e995);
  const _e996 = WF.h("hr", { className: "wf-divider" });
  _e925.appendChild(_e996);
  const _e997 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e997);
  const _e998 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Speed Variants");
  _e925.appendChild(_e998);
  const _e999 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e999);
  const _e1000 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1001 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn", "150ms"); } });
  const _e1002 = WF.h("div", { className: "wf-card__body" });
  const _e1003 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "fast");
  _e1002.appendChild(_e1003);
  const _e1004 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "150ms");
  _e1002.appendChild(_e1004);
  const _e1005 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn, fast)");
  _e1002.appendChild(_e1005);
  _e1001.appendChild(_e1002);
  _e1000.appendChild(_e1001);
  const _e1006 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn"); } });
  const _e1007 = WF.h("div", { className: "wf-card__body" });
  const _e1008 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "default");
  _e1007.appendChild(_e1008);
  const _e1009 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "300ms");
  _e1007.appendChild(_e1009);
  const _e1010 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn)");
  _e1007.appendChild(_e1010);
  _e1006.appendChild(_e1007);
  _e1000.appendChild(_e1006);
  const _e1011 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn", "500ms"); } });
  const _e1012 = WF.h("div", { className: "wf-card__body" });
  const _e1013 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slow");
  _e1012.appendChild(_e1013);
  const _e1014 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "500ms");
  _e1012.appendChild(_e1014);
  const _e1015 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn, slow)");
  _e1012.appendChild(_e1015);
  _e1011.appendChild(_e1012);
  _e1000.appendChild(_e1011);
  _e925.appendChild(_e1000);
  const _e1016 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1016);
  const _e1017 = WF.h("hr", { className: "wf-divider" });
  _e925.appendChild(_e1017);
  const _e1018 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1018);
  const _e1019 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "All 12 Animations");
  _e925.appendChild(_e1019);
  const _e1020 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1020);
  const _e1021 = WF.h("table", { className: "wf-table" });
  const _e1022 = WF.h("thead", {});
  const _e1023 = WF.h("td", {}, "Name");
  _e1022.appendChild(_e1023);
  const _e1024 = WF.h("td", {}, "Effect");
  _e1022.appendChild(_e1024);
  const _e1025 = WF.h("td", {}, "Usage");
  _e1022.appendChild(_e1025);
  _e1021.appendChild(_e1022);
  const _e1026 = WF.h("tr", {});
  const _e1027 = WF.h("td", {}, "fadeIn / fadeOut");
  _e1026.appendChild(_e1027);
  const _e1028 = WF.h("td", {}, "Opacity fade");
  _e1026.appendChild(_e1028);
  const _e1029 = WF.h("td", {}, "Card(elevated, fadeIn)");
  _e1026.appendChild(_e1029);
  _e1021.appendChild(_e1026);
  const _e1030 = WF.h("tr", {});
  const _e1031 = WF.h("td", {}, "slideUp / slideDown");
  _e1030.appendChild(_e1031);
  const _e1032 = WF.h("td", {}, "Vertical slide + fade");
  _e1030.appendChild(_e1032);
  const _e1033 = WF.h("td", {}, "Heading(\"Hi\", h1, slideUp)");
  _e1030.appendChild(_e1033);
  _e1021.appendChild(_e1030);
  const _e1034 = WF.h("tr", {});
  const _e1035 = WF.h("td", {}, "slideLeft / slideRight");
  _e1034.appendChild(_e1035);
  const _e1036 = WF.h("td", {}, "Horizontal slide + fade");
  _e1034.appendChild(_e1036);
  const _e1037 = WF.h("td", {}, "Text(\"Hello\", slideLeft)");
  _e1034.appendChild(_e1037);
  _e1021.appendChild(_e1034);
  const _e1038 = WF.h("tr", {});
  const _e1039 = WF.h("td", {}, "scaleIn / scaleOut");
  _e1038.appendChild(_e1039);
  const _e1040 = WF.h("td", {}, "Scale from/to 90%");
  _e1038.appendChild(_e1040);
  const _e1041 = WF.h("td", {}, "Badge(\"New\", scaleIn)");
  _e1038.appendChild(_e1041);
  _e1021.appendChild(_e1038);
  const _e1042 = WF.h("tr", {});
  const _e1043 = WF.h("td", {}, "bounce");
  _e1042.appendChild(_e1043);
  const _e1044 = WF.h("td", {}, "Bouncy entrance");
  _e1042.appendChild(_e1044);
  const _e1045 = WF.h("td", {}, "Button(\"Go\", bounce)");
  _e1042.appendChild(_e1045);
  _e1021.appendChild(_e1042);
  const _e1046 = WF.h("tr", {});
  const _e1047 = WF.h("td", {}, "shake");
  _e1046.appendChild(_e1047);
  const _e1048 = WF.h("td", {}, "Horizontal shake");
  _e1046.appendChild(_e1048);
  const _e1049 = WF.h("td", {}, "Alert(\"Error!\", shake)");
  _e1046.appendChild(_e1049);
  _e1021.appendChild(_e1046);
  const _e1050 = WF.h("tr", {});
  const _e1051 = WF.h("td", {}, "pulse");
  _e1050.appendChild(_e1051);
  const _e1052 = WF.h("td", {}, "Scale pulse (infinite)");
  _e1050.appendChild(_e1052);
  const _e1053 = WF.h("td", {}, "Badge(\"Live\", pulse)");
  _e1050.appendChild(_e1053);
  _e1021.appendChild(_e1050);
  const _e1054 = WF.h("tr", {});
  const _e1055 = WF.h("td", {}, "spin");
  _e1054.appendChild(_e1055);
  const _e1056 = WF.h("td", {}, "360-degree rotation");
  _e1054.appendChild(_e1056);
  const _e1057 = WF.h("td", {}, "Spinner(spin)");
  _e1054.appendChild(_e1057);
  _e1021.appendChild(_e1054);
  _e925.appendChild(_e1021);
  const _e1058 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1058);
  const _e1059 = WF.h("hr", { className: "wf-divider" });
  _e925.appendChild(_e1059);
  const _e1060 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1060);
  const _e1061 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Conditional Animations");
  _e925.appendChild(_e1061);
  const _e1062 = WF.h("p", { className: "wf-text" }, "Attach enter and exit animations to if blocks.");
  _e925.appendChild(_e1062);
  const _e1063 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1063);
  const _e1064 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1065 = WF.h("div", { className: "wf-card__body" });
  const _e1066 = WF.h("code", { className: "wf-code wf-code--block" }, "if visible, animate(slideUp, fadeOut) {\n    Card { Text(\"Appears with slideUp, exits with fadeOut\") }\n}\n\nif expanded, animate(scaleIn, scaleOut) {\n    Text(\"Scales in and out\")\n}");
  _e1065.appendChild(_e1066);
  _e1064.appendChild(_e1065);
  _e925.appendChild(_e1064);
  const _e1067 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1067);
  const _e1068 = WF.h("hr", { className: "wf-divider" });
  _e925.appendChild(_e1068);
  const _e1069 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1069);
  const _e1070 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "List Stagger");
  _e925.appendChild(_e1070);
  const _e1071 = WF.h("p", { className: "wf-text" }, "Animate list items with staggered delays.");
  _e925.appendChild(_e1071);
  const _e1072 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1072);
  const _e1073 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1074 = WF.h("div", { className: "wf-card__body" });
  const _e1075 = WF.h("code", { className: "wf-code wf-code--block" }, "for item in items, animate(slideUp, fadeOut, stagger: \"50ms\") {\n    Card { Text(item.name) }\n}");
  _e1074.appendChild(_e1075);
  _e1073.appendChild(_e1074);
  _e925.appendChild(_e1073);
  const _e1076 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1076);
  const _e1077 = WF.h("hr", { className: "wf-divider" });
  _e925.appendChild(_e1077);
  const _e1078 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1078);
  const _e1079 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Transition Blocks");
  _e925.appendChild(_e1079);
  const _e1080 = WF.h("p", { className: "wf-text" }, "Smooth CSS transitions on property changes.");
  _e925.appendChild(_e1080);
  const _e1081 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1081);
  const _e1082 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1083 = WF.h("div", { className: "wf-card__body" });
  const _e1084 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Hover me\") {\n    transition {\n        background 200ms ease\n        transform 150ms spring\n    }\n}");
  _e1083.appendChild(_e1084);
  _e1082.appendChild(_e1083);
  _e925.appendChild(_e1082);
  const _e1085 = WF.h("div", { className: "wf-spacer" });
  _e925.appendChild(_e1085);
  _root.appendChild(_e925);
  return _root;
}

function Page_I18n(params) {
  const _root = document.createDocumentFragment();
  const _e1086 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1087 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1087);
  const _e1088 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Internationalization (i18n)");
  _e1086.appendChild(_e1088);
  const _e1089 = WF.h("p", { className: "wf-text wf-text--muted" }, "Built-in multi-language support with reactive locale switching and automatic RTL.");
  _e1086.appendChild(_e1089);
  const _e1090 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1090);
  const _e1091 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Setup");
  _e1086.appendChild(_e1091);
  const _e1092 = WF.h("p", { className: "wf-text" }, "Create a JSON file per locale in your translations directory.");
  _e1086.appendChild(_e1092);
  const _e1093 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1093);
  const _e1094 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1095 = WF.h("div", { className: "wf-card__body" });
  const _e1096 = WF.h("code", { className: "wf-code wf-code--block" }, "// src/translations/en.json\n{\n    \"greeting\": \"Hello, {name}!\",\n    \"nav.home\": \"Home\",\n    \"nav.about\": \"About\"\n}\n\n// src/translations/ar.json\n{\n    \"greeting\": \"!أهلاً، {name}\",\n    \"nav.home\": \"الرئيسية\",\n    \"nav.about\": \"حول\"\n}");
  _e1095.appendChild(_e1096);
  _e1094.appendChild(_e1095);
  _e1086.appendChild(_e1094);
  const _e1097 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1097);
  const _e1098 = WF.h("p", { className: "wf-text wf-text--bold" }, "Add i18n config to webfluent.app.json:");
  _e1086.appendChild(_e1098);
  const _e1099 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1100 = WF.h("div", { className: "wf-card__body" });
  const _e1101 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e1100.appendChild(_e1101);
  _e1099.appendChild(_e1100);
  _e1086.appendChild(_e1099);
  const _e1102 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1102);
  const _e1103 = WF.h("hr", { className: "wf-divider" });
  _e1086.appendChild(_e1103);
  const _e1104 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1104);
  const _e1105 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "The t() Function");
  _e1086.appendChild(_e1105);
  const _e1106 = WF.h("p", { className: "wf-text" }, "Use t() to look up translated text. It is reactive — all t() calls update when the locale changes.");
  _e1086.appendChild(_e1106);
  const _e1107 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1107);
  const _e1108 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1109 = WF.h("div", { className: "wf-card__body" });
  const _e1110 = WF.h("code", { className: "wf-code wf-code--block" }, "// Simple key lookup\nText(t(\"nav.home\"))\n\n// With interpolation\nText(t(\"greeting\", name: user.name))\n\n// In any component\nButton(t(\"actions.save\"), primary)\nHeading(t(\"page.title\"), h1)");
  _e1109.appendChild(_e1110);
  _e1108.appendChild(_e1109);
  _e1086.appendChild(_e1108);
  const _e1111 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1111);
  const _e1112 = WF.h("hr", { className: "wf-divider" });
  _e1086.appendChild(_e1112);
  const _e1113 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1113);
  const _e1114 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Locale Switching");
  _e1086.appendChild(_e1114);
  const _e1115 = WF.h("p", { className: "wf-text" }, "Switch the locale at runtime with setLocale(). All translated text updates instantly.");
  _e1086.appendChild(_e1115);
  const _e1116 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1116);
  const _e1117 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1118 = WF.h("div", { className: "wf-card__body" });
  const _e1119 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"English\") { setLocale(\"en\") }\nButton(\"العربية\") { setLocale(\"ar\") }\nButton(\"Espanol\") { setLocale(\"es\") }\n\n// Access current locale\nText(\"Current: {locale}\")\nText(\"Direction: {dir}\")");
  _e1118.appendChild(_e1119);
  _e1117.appendChild(_e1118);
  _e1086.appendChild(_e1117);
  const _e1120 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1120);
  const _e1121 = WF.h("hr", { className: "wf-divider" });
  _e1086.appendChild(_e1121);
  const _e1122 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1122);
  const _e1123 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "RTL Support");
  _e1086.appendChild(_e1123);
  const _e1124 = WF.h("p", { className: "wf-text" }, "WebFluent automatically detects RTL locales and updates the document direction.");
  _e1086.appendChild(_e1124);
  const _e1125 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1125);
  const _e1126 = WF.h("table", { className: "wf-table" });
  const _e1127 = WF.h("thead", {});
  const _e1128 = WF.h("td", {}, "Locale");
  _e1127.appendChild(_e1128);
  const _e1129 = WF.h("td", {}, "Direction");
  _e1127.appendChild(_e1129);
  _e1126.appendChild(_e1127);
  const _e1130 = WF.h("tr", {});
  const _e1131 = WF.h("td", {}, "ar (Arabic)");
  _e1130.appendChild(_e1131);
  const _e1132 = WF.h("td", {}, "RTL");
  _e1130.appendChild(_e1132);
  _e1126.appendChild(_e1130);
  const _e1133 = WF.h("tr", {});
  const _e1134 = WF.h("td", {}, "he (Hebrew)");
  _e1133.appendChild(_e1134);
  const _e1135 = WF.h("td", {}, "RTL");
  _e1133.appendChild(_e1135);
  _e1126.appendChild(_e1133);
  const _e1136 = WF.h("tr", {});
  const _e1137 = WF.h("td", {}, "fa (Farsi)");
  _e1136.appendChild(_e1137);
  const _e1138 = WF.h("td", {}, "RTL");
  _e1136.appendChild(_e1138);
  _e1126.appendChild(_e1136);
  const _e1139 = WF.h("tr", {});
  const _e1140 = WF.h("td", {}, "ur (Urdu)");
  _e1139.appendChild(_e1140);
  const _e1141 = WF.h("td", {}, "RTL");
  _e1139.appendChild(_e1141);
  _e1126.appendChild(_e1139);
  const _e1142 = WF.h("tr", {});
  const _e1143 = WF.h("td", {}, "All others");
  _e1142.appendChild(_e1143);
  const _e1144 = WF.h("td", {}, "LTR");
  _e1142.appendChild(_e1144);
  _e1126.appendChild(_e1142);
  _e1086.appendChild(_e1126);
  const _e1145 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1145);
  const _e1146 = WF.h("p", { className: "wf-text wf-text--muted" }, "When setLocale(\"ar\") is called, the HTML element gets dir=\"rtl\" and lang=\"ar\" automatically.");
  _e1086.appendChild(_e1146);
  const _e1147 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1147);
  const _e1148 = WF.h("hr", { className: "wf-divider" });
  _e1086.appendChild(_e1148);
  const _e1149 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1149);
  const _e1150 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Fallback Behavior");
  _e1086.appendChild(_e1150);
  const _e1151 = WF.h("p", { className: "wf-text" }, "If a key is missing in the current locale:");
  _e1086.appendChild(_e1151);
  const _e1152 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1153 = WF.h("p", { className: "wf-text" }, "1. Falls back to the defaultLocale translation");
  _e1152.appendChild(_e1153);
  const _e1154 = WF.h("p", { className: "wf-text" }, "2. If still missing, returns the key itself (e.g., \"nav.home\")");
  _e1152.appendChild(_e1154);
  _e1086.appendChild(_e1152);
  const _e1155 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1155);
  const _e1156 = WF.h("hr", { className: "wf-divider" });
  _e1086.appendChild(_e1156);
  const _e1157 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1157);
  const _e1158 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "SSG + i18n");
  _e1086.appendChild(_e1158);
  const _e1159 = WF.h("p", { className: "wf-text wf-text--muted" }, "When both SSG and i18n are enabled, pages are pre-rendered with the default locale text. After JavaScript loads, locale switching works normally.");
  _e1086.appendChild(_e1159);
  const _e1160 = WF.h("div", { className: "wf-spacer" });
  _e1086.appendChild(_e1160);
  _root.appendChild(_e1086);
  return _root;
}

function Page_GettingStarted(params) {
  const _root = document.createDocumentFragment();
  const _e1161 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1162 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1162);
  const _e1163 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Getting Started");
  _e1161.appendChild(_e1163);
  const _e1164 = WF.h("p", { className: "wf-text wf-text--muted" }, "Get up and running with WebFluent in under a minute.");
  _e1161.appendChild(_e1164);
  const _e1165 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1165);
  const _e1166 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Install");
  _e1161.appendChild(_e1166);
  const _e1167 = WF.h("p", { className: "wf-text" }, "Build from source (requires Rust):");
  _e1161.appendChild(_e1167);
  const _e1168 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1168);
  const _e1169 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1170 = WF.h("div", { className: "wf-card__body" });
  const _e1171 = WF.h("code", { className: "wf-code wf-code--block" }, "git clone https://github.com/user/webfluent.git\ncd webfluent\ncargo build --release");
  _e1170.appendChild(_e1171);
  _e1169.appendChild(_e1170);
  _e1161.appendChild(_e1169);
  const _e1172 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1172);
  const _e1173 = WF.h("p", { className: "wf-text wf-text--muted" }, "The binary is at target/release/wf. Add it to your PATH.");
  _e1161.appendChild(_e1173);
  const _e1174 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1174);
  const _e1175 = WF.h("hr", { className: "wf-divider" });
  _e1161.appendChild(_e1175);
  const _e1176 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1176);
  const _e1177 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Create a Project");
  _e1161.appendChild(_e1177);
  const _e1178 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1178);
  const _e1179 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1180 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1181 = WF.h("div", { className: "wf-card__body" });
  const _e1182 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "SPA");
  _e1181.appendChild(_e1182);
  const _e1183 = WF.h("div", { className: "wf-spacer" });
  _e1181.appendChild(_e1183);
  const _e1184 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Interactive App");
  _e1181.appendChild(_e1184);
  const _e1185 = WF.h("p", { className: "wf-text wf-text--muted" }, "Dashboard with routing, stores, forms, modals, animations.");
  _e1181.appendChild(_e1185);
  const _e1186 = WF.h("div", { className: "wf-spacer" });
  _e1181.appendChild(_e1186);
  const _e1187 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-app -t spa");
  _e1181.appendChild(_e1187);
  _e1180.appendChild(_e1181);
  _e1179.appendChild(_e1180);
  const _e1188 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1189 = WF.h("div", { className: "wf-card__body" });
  const _e1190 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Static");
  _e1189.appendChild(_e1190);
  const _e1191 = WF.h("div", { className: "wf-spacer" });
  _e1189.appendChild(_e1191);
  const _e1192 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Static Site");
  _e1189.appendChild(_e1192);
  const _e1193 = WF.h("p", { className: "wf-text wf-text--muted" }, "Marketing site with SSG, i18n, blog, contact form.");
  _e1189.appendChild(_e1193);
  const _e1194 = WF.h("div", { className: "wf-spacer" });
  _e1189.appendChild(_e1194);
  const _e1195 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-site -t static");
  _e1189.appendChild(_e1195);
  _e1188.appendChild(_e1189);
  _e1179.appendChild(_e1188);
  const _e1196 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1197 = WF.h("div", { className: "wf-card__body" });
  const _e1198 = WF.h("span", { className: "wf-badge wf-badge--info" }, "PDF");
  _e1197.appendChild(_e1198);
  const _e1199 = WF.h("div", { className: "wf-spacer" });
  _e1197.appendChild(_e1199);
  const _e1200 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "PDF Document");
  _e1197.appendChild(_e1200);
  const _e1201 = WF.h("p", { className: "wf-text wf-text--muted" }, "Reports, invoices, docs. Tables, code blocks, auto page breaks.");
  _e1197.appendChild(_e1201);
  const _e1202 = WF.h("div", { className: "wf-spacer" });
  _e1197.appendChild(_e1202);
  const _e1203 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-report -t pdf");
  _e1197.appendChild(_e1203);
  _e1196.appendChild(_e1197);
  _e1179.appendChild(_e1196);
  _e1161.appendChild(_e1179);
  const _e1204 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1204);
  const _e1205 = WF.h("hr", { className: "wf-divider" });
  _e1161.appendChild(_e1205);
  const _e1206 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1206);
  const _e1207 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Build and Serve");
  _e1161.appendChild(_e1207);
  const _e1208 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1209 = WF.h("div", { className: "wf-card__body" });
  const _e1210 = WF.h("code", { className: "wf-code wf-code--block" }, "cd my-app\nwf build\nwf serve");
  _e1209.appendChild(_e1210);
  _e1208.appendChild(_e1209);
  _e1161.appendChild(_e1208);
  const _e1211 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1211);
  const _e1212 = WF.h("p", { className: "wf-text wf-text--muted" }, "Open http://localhost:3000 in your browser.");
  _e1161.appendChild(_e1212);
  const _e1213 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1213);
  const _e1214 = WF.h("hr", { className: "wf-divider" });
  _e1161.appendChild(_e1214);
  const _e1215 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1215);
  const _e1216 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Project Structure");
  _e1161.appendChild(_e1216);
  const _e1217 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1218 = WF.h("div", { className: "wf-card__body" });
  const _e1219 = WF.h("code", { className: "wf-code wf-code--block" }, "my-app/\n+-- webfluent.app.json       # Config\n+-- src/\n|   +-- App.wf               # Root (router, layout)\n|   +-- pages/\n|   +-- components/\n|   +-- stores/\n|   +-- translations/\n+-- public/\n+-- build/");
  _e1218.appendChild(_e1219);
  _e1217.appendChild(_e1218);
  _e1161.appendChild(_e1217);
  const _e1220 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1220);
  const _e1221 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1222 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/guide"); } }, "Read the Guide");
  _e1221.appendChild(_e1222);
  const _e1223 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/components"); } }, "Browse Components");
  _e1221.appendChild(_e1223);
  _e1161.appendChild(_e1221);
  const _e1224 = WF.h("div", { className: "wf-spacer" });
  _e1161.appendChild(_e1224);
  _root.appendChild(_e1161);
  return _root;
}

function Page_Cli(params) {
  const _root = document.createDocumentFragment();
  const _e1225 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1226 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1226);
  const _e1227 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "CLI Reference");
  _e1225.appendChild(_e1227);
  const _e1228 = WF.h("p", { className: "wf-text wf-text--muted" }, "The WebFluent command-line interface.");
  _e1225.appendChild(_e1228);
  const _e1229 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1229);
  const _e1230 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf init");
  _e1225.appendChild(_e1230);
  const _e1231 = WF.h("p", { className: "wf-text" }, "Create a new WebFluent project.");
  _e1225.appendChild(_e1231);
  const _e1232 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1232);
  const _e1233 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1234 = WF.h("div", { className: "wf-card__body" });
  const _e1235 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init <name> [--template spa|static|pdf]");
  _e1234.appendChild(_e1235);
  _e1233.appendChild(_e1234);
  _e1225.appendChild(_e1233);
  const _e1236 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1236);
  const _e1237 = WF.h("table", { className: "wf-table" });
  const _e1238 = WF.h("thead", {});
  const _e1239 = WF.h("td", {}, "Argument");
  _e1238.appendChild(_e1239);
  const _e1240 = WF.h("td", {}, "Description");
  _e1238.appendChild(_e1240);
  _e1237.appendChild(_e1238);
  const _e1241 = WF.h("tr", {});
  const _e1242 = WF.h("td", {}, "name");
  _e1241.appendChild(_e1242);
  const _e1243 = WF.h("td", {}, "Project name (creates a directory)");
  _e1241.appendChild(_e1243);
  _e1237.appendChild(_e1241);
  const _e1244 = WF.h("tr", {});
  const _e1245 = WF.h("td", {}, "--template, -t");
  _e1244.appendChild(_e1245);
  const _e1246 = WF.h("td", {}, "Template: spa (default), static, or pdf");
  _e1244.appendChild(_e1246);
  _e1237.appendChild(_e1244);
  _e1225.appendChild(_e1237);
  const _e1247 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1247);
  const _e1248 = WF.h("p", { className: "wf-text wf-text--muted" }, "SPA: interactive app with routing and state. Static: SSG site with i18n. PDF: document generation with tables, headings, and auto page breaks.");
  _e1225.appendChild(_e1248);
  const _e1249 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1249);
  const _e1250 = WF.h("hr", { className: "wf-divider" });
  _e1225.appendChild(_e1250);
  const _e1251 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1251);
  const _e1252 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf build");
  _e1225.appendChild(_e1252);
  const _e1253 = WF.h("p", { className: "wf-text" }, "Compile .wf files to HTML, CSS, and JavaScript.");
  _e1225.appendChild(_e1253);
  const _e1254 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1254);
  const _e1255 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1256 = WF.h("div", { className: "wf-card__body" });
  const _e1257 = WF.h("code", { className: "wf-code wf-code--block" }, "wf build [--dir DIR]");
  _e1256.appendChild(_e1257);
  _e1255.appendChild(_e1256);
  _e1225.appendChild(_e1255);
  const _e1258 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1258);
  const _e1259 = WF.h("table", { className: "wf-table" });
  const _e1260 = WF.h("thead", {});
  const _e1261 = WF.h("td", {}, "Option");
  _e1260.appendChild(_e1261);
  const _e1262 = WF.h("td", {}, "Description");
  _e1260.appendChild(_e1262);
  _e1259.appendChild(_e1260);
  const _e1263 = WF.h("tr", {});
  const _e1264 = WF.h("td", {}, "--dir, -d");
  _e1263.appendChild(_e1264);
  const _e1265 = WF.h("td", {}, "Project directory (default: current directory)");
  _e1263.appendChild(_e1265);
  _e1259.appendChild(_e1263);
  _e1225.appendChild(_e1259);
  const _e1266 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1266);
  const _e1267 = WF.h("p", { className: "wf-text wf-text--muted" }, "The build pipeline: Lex all .wf files, parse to AST, run accessibility linter, generate HTML + CSS + JS, write to output directory.");
  _e1225.appendChild(_e1267);
  const _e1268 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1268);
  const _e1269 = WF.h("p", { className: "wf-text" }, "Output depends on config:");
  _e1225.appendChild(_e1269);
  const _e1270 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1271 = WF.h("p", { className: "wf-text" }, "SPA mode (default): single index.html + app.js + styles.css");
  _e1270.appendChild(_e1271);
  const _e1272 = WF.h("p", { className: "wf-text" }, "SSG mode (ssg: true): one HTML per page + app.js + styles.css");
  _e1270.appendChild(_e1272);
  const _e1273 = WF.h("p", { className: "wf-text" }, "PDF mode (output_type: pdf): a single .pdf file");
  _e1270.appendChild(_e1273);
  _e1225.appendChild(_e1270);
  const _e1274 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1274);
  const _e1275 = WF.h("hr", { className: "wf-divider" });
  _e1225.appendChild(_e1275);
  const _e1276 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1276);
  const _e1277 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf serve");
  _e1225.appendChild(_e1277);
  const _e1278 = WF.h("p", { className: "wf-text" }, "Start a development server that serves the built output.");
  _e1225.appendChild(_e1278);
  const _e1279 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1279);
  const _e1280 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1281 = WF.h("div", { className: "wf-card__body" });
  const _e1282 = WF.h("code", { className: "wf-code wf-code--block" }, "wf serve [--dir DIR]");
  _e1281.appendChild(_e1282);
  _e1280.appendChild(_e1281);
  _e1225.appendChild(_e1280);
  const _e1283 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1283);
  const _e1284 = WF.h("p", { className: "wf-text wf-text--muted" }, "Serves files from the build directory. SPA fallback: all routes serve index.html so client-side routing works. Port is configured in webfluent.app.json (default: 3000).");
  _e1225.appendChild(_e1284);
  const _e1285 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1285);
  const _e1286 = WF.h("hr", { className: "wf-divider" });
  _e1225.appendChild(_e1286);
  const _e1287 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1287);
  const _e1288 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf generate");
  _e1225.appendChild(_e1288);
  const _e1289 = WF.h("p", { className: "wf-text" }, "Scaffold a new page, component, or store.");
  _e1225.appendChild(_e1289);
  const _e1290 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1290);
  const _e1291 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1292 = WF.h("div", { className: "wf-card__body" });
  const _e1293 = WF.h("code", { className: "wf-code wf-code--block" }, "wf generate <kind> <name> [--dir DIR]");
  _e1292.appendChild(_e1293);
  _e1291.appendChild(_e1292);
  _e1225.appendChild(_e1291);
  const _e1294 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1294);
  const _e1295 = WF.h("table", { className: "wf-table" });
  const _e1296 = WF.h("thead", {});
  const _e1297 = WF.h("td", {}, "Kind");
  _e1296.appendChild(_e1297);
  const _e1298 = WF.h("td", {}, "Creates");
  _e1296.appendChild(_e1298);
  const _e1299 = WF.h("td", {}, "Example");
  _e1296.appendChild(_e1299);
  _e1295.appendChild(_e1296);
  const _e1300 = WF.h("tr", {});
  const _e1301 = WF.h("td", {}, "page");
  _e1300.appendChild(_e1301);
  const _e1302 = WF.h("td", {}, "src/pages/Name.wf");
  _e1300.appendChild(_e1302);
  const _e1303 = WF.h("td", {}, "wf generate page About");
  _e1300.appendChild(_e1303);
  _e1295.appendChild(_e1300);
  const _e1304 = WF.h("tr", {});
  const _e1305 = WF.h("td", {}, "component");
  _e1304.appendChild(_e1305);
  const _e1306 = WF.h("td", {}, "src/components/Name.wf");
  _e1304.appendChild(_e1306);
  const _e1307 = WF.h("td", {}, "wf generate component Header");
  _e1304.appendChild(_e1307);
  _e1295.appendChild(_e1304);
  const _e1308 = WF.h("tr", {});
  const _e1309 = WF.h("td", {}, "store");
  _e1308.appendChild(_e1309);
  const _e1310 = WF.h("td", {}, "src/stores/name.wf");
  _e1308.appendChild(_e1310);
  const _e1311 = WF.h("td", {}, "wf generate store CartStore");
  _e1308.appendChild(_e1311);
  _e1295.appendChild(_e1308);
  _e1225.appendChild(_e1295);
  const _e1312 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1312);
  const _e1313 = WF.h("hr", { className: "wf-divider" });
  _e1225.appendChild(_e1313);
  const _e1314 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1314);
  const _e1315 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Configuration");
  _e1225.appendChild(_e1315);
  const _e1316 = WF.h("p", { className: "wf-text" }, "All config is in webfluent.app.json at the project root.");
  _e1225.appendChild(_e1316);
  const _e1317 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1317);
  const _e1318 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1319 = WF.h("div", { className: "wf-card__body" });
  const _e1320 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"name\": \"My App\",\n  \"version\": \"1.0.0\",\n  \"author\": \"Your Name\",\n  \"theme\": {\n    \"name\": \"default\",\n    \"mode\": \"light\",\n    \"tokens\": { \"color-primary\": \"#6366F1\" }\n  },\n  \"build\": {\n    \"output\": \"./build\",\n    \"minify\": true,\n    \"ssg\": false,\n    \"output_type\": \"spa\",\n    \"pdf\": {\n      \"page_size\": \"A4\",\n      \"default_font\": \"Helvetica\",\n      \"output_filename\": \"report.pdf\"\n    }\n  },\n  \"dev\": { \"port\": 3000 },\n  \"meta\": {\n    \"title\": \"My App\",\n    \"description\": \"Built with WebFluent\",\n    \"lang\": \"en\"\n  },\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e1319.appendChild(_e1320);
  _e1318.appendChild(_e1319);
  _e1225.appendChild(_e1318);
  const _e1321 = WF.h("div", { className: "wf-spacer" });
  _e1225.appendChild(_e1321);
  _root.appendChild(_e1225);
  return _root;
}

function Page_Home(params) {
  const _counter = WF.signal(0);
  const _taskInput = WF.signal("");
  const _showDemo = WF.signal(false);
  const _root = document.createDocumentFragment();
  const _e1322 = WF.h("div", { className: "wf-container" });
  const _e1323 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1323);
  const _e1324 = WF.h("h2", { className: "wf-heading wf-heading--h1 wf-text--center wf-animate-slideUp" }, () => WF.i18n.t("hero.title"));
  _e1322.appendChild(_e1324);
  const _e1325 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1325);
  const _e1326 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, () => WF.i18n.t("hero.sub1"));
  _e1322.appendChild(_e1326);
  const _e1327 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, () => WF.i18n.t("hero.sub2"));
  _e1322.appendChild(_e1327);
  const _e1328 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1328);
  const _e1329 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1330 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, () => WF.i18n.t("hero.cta"));
  _e1329.appendChild(_e1330);
  const _e1331 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { WF.navigate("/guide"); } }, () => WF.i18n.t("hero.guide"));
  _e1329.appendChild(_e1331);
  _e1322.appendChild(_e1329);
  const _e1332 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1332);
  const _e1333 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1334 = WF.h("div", { className: "wf-card__body" });
  const _e1335 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\") {\n    Container {\n        Heading(\"Hello, WebFluent!\", h1)\n        Text(\"Build for the web. Nothing else.\")\n\n        Button(\"Get Started\", primary, large) {\n            navigate(\"/docs\")\n        }\n    }\n}");
  _e1334.appendChild(_e1335);
  _e1333.appendChild(_e1334);
  _e1322.appendChild(_e1333);
  const _e1336 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1336);
  const _e1337 = WF.h("hr", { className: "wf-divider" });
  _e1322.appendChild(_e1337);
  const _e1338 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1338);
  const _e1339 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, () => WF.i18n.t("demo.title"));
  _e1322.appendChild(_e1339);
  const _e1340 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, () => WF.i18n.t("demo.subtitle"));
  _e1322.appendChild(_e1340);
  const _e1341 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1341);
  const _e1342 = WF.h("div", { className: "wf-grid wf-grid--gap-lg", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e1343 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1344 = WF.h("div", { className: "wf-card__header" });
  const _e1345 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("demo.counter"));
  _e1344.appendChild(_e1345);
  _e1343.appendChild(_e1344);
  const _e1346 = WF.h("div", { className: "wf-card__body" });
  const _e1347 = WF.h("div", { className: "wf-row wf-row--center wf-row--gap-md" });
  const _e1348 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { _counter.set((_counter() - 1)); } }, "-");
  _e1347.appendChild(_e1348);
  const _e1349 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-heading--primary" }, () => `${_counter()}`);
  _e1347.appendChild(_e1349);
  const _e1350 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { _counter.set((_counter() + 1)); } }, "+");
  _e1347.appendChild(_e1350);
  _e1346.appendChild(_e1347);
  const _e1351 = WF.h("div", { className: "wf-spacer" });
  _e1346.appendChild(_e1351);
  const _e1352 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.counter.hint"));
  _e1346.appendChild(_e1352);
  _e1343.appendChild(_e1346);
  _e1342.appendChild(_e1343);
  const _e1353 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1354 = WF.h("div", { className: "wf-card__header" });
  const _e1355 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("demo.binding"));
  _e1354.appendChild(_e1355);
  _e1353.appendChild(_e1354);
  const _e1356 = WF.h("div", { className: "wf-card__body" });
  const _e1357 = WF.h("input", { className: "wf-input", value: () => _taskInput(), "on:input": (e) => _taskInput.set(e.target.value), placeholder: WF.i18n.t("demo.binding.placeholder"), label: "Input", type: "text" });
  _e1356.appendChild(_e1357);
  const _e1358 = WF.h("div", { className: "wf-spacer" });
  _e1356.appendChild(_e1358);
  WF.condRender(_e1356,
    () => (_taskInput() !== ""),
    () => {
      const _e1359 = document.createDocumentFragment();
      const _e1360 = WF.h("div", { className: "wf-alert wf-alert--info" }, () => `You typed: ${_taskInput()}`);
      _e1359.appendChild(_e1360);
      return _e1359;
    },
    null,
    null
  );
  const _e1361 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.binding.hint"));
  _e1356.appendChild(_e1361);
  _e1353.appendChild(_e1356);
  _e1342.appendChild(_e1353);
  const _e1362 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1363 = WF.h("div", { className: "wf-card__header" });
  const _e1364 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("demo.conditional"));
  _e1363.appendChild(_e1364);
  _e1362.appendChild(_e1363);
  const _e1365 = WF.h("div", { className: "wf-card__body" });
  const _e1366 = WF.h("label", { className: "wf-switch" });
  const _e1367 = WF.h("input", { type: "checkbox", checked: () => _showDemo(), "on:change": () => _showDemo.set(!_showDemo()) });
  _e1366.appendChild(_e1367);
  const _e1368 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1366.appendChild(_e1368);
  _e1366.appendChild(WF.text(WF.i18n.t("demo.conditional.toggle")));
  _e1365.appendChild(_e1366);
  const _e1369 = WF.h("div", { className: "wf-spacer" });
  _e1365.appendChild(_e1369);
  WF.condRender(_e1365,
    () => _showDemo(),
    () => {
      const _e1370 = document.createDocumentFragment();
      const _e1371 = WF.h("div", { className: "wf-card wf-card--outlined" });
      const _e1372 = WF.h("div", { className: "wf-card__body" });
      const _e1373 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Visible!");
      _e1372.appendChild(_e1373);
      const _e1374 = WF.h("div", { className: "wf-spacer" });
      _e1372.appendChild(_e1374);
      const _e1375 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("demo.conditional.text"));
      _e1372.appendChild(_e1375);
      _e1371.appendChild(_e1372);
      _e1370.appendChild(_e1371);
      return _e1370;
    },
    null,
    { enter: "slideUp", exit: "fadeOut" }
  );
  _e1362.appendChild(_e1365);
  _e1342.appendChild(_e1362);
  const _e1376 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1377 = WF.h("div", { className: "wf-card__header" });
  const _e1378 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("demo.components"));
  _e1377.appendChild(_e1378);
  _e1376.appendChild(_e1377);
  const _e1379 = WF.h("div", { className: "wf-card__body" });
  const _e1380 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1381 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1382 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e1381.appendChild(_e1382);
  const _e1383 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e1381.appendChild(_e1383);
  const _e1384 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e1381.appendChild(_e1384);
  _e1380.appendChild(_e1381);
  const _e1385 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1386 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "New");
  _e1385.appendChild(_e1386);
  const _e1387 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Sale");
  _e1385.appendChild(_e1387);
  const _e1388 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Active");
  _e1385.appendChild(_e1388);
  const _e1389 = WF.h("span", { className: "wf-tag" }, "WebFluent");
  _e1385.appendChild(_e1389);
  _e1380.appendChild(_e1385);
  const _e1390 = WF.h("progress", { className: "wf-progress", value: 72, max: 100 });
  _e1380.appendChild(_e1390);
  const _e1391 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.components.hint"));
  _e1380.appendChild(_e1391);
  _e1379.appendChild(_e1380);
  _e1376.appendChild(_e1379);
  _e1342.appendChild(_e1376);
  _e1322.appendChild(_e1342);
  const _e1392 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1392);
  const _e1393 = WF.h("hr", { className: "wf-divider" });
  _e1322.appendChild(_e1393);
  const _e1394 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1394);
  const _e1395 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, () => WF.i18n.t("why.title"));
  _e1322.appendChild(_e1395);
  const _e1396 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, () => WF.i18n.t("why.subtitle"));
  _e1322.appendChild(_e1396);
  const _e1397 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1397);
  const _e1398 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1399 = Component_FeatureCard({ title: WF.i18n.t("why.syntax"), description: WF.i18n.t("why.syntax.desc") });
  _e1398.appendChild(_e1399);
  const _e1400 = Component_FeatureCard({ title: WF.i18n.t("why.components"), description: WF.i18n.t("why.components.desc") });
  _e1398.appendChild(_e1400);
  const _e1401 = Component_FeatureCard({ title: WF.i18n.t("why.reactivity"), description: WF.i18n.t("why.reactivity.desc") });
  _e1398.appendChild(_e1401);
  const _e1402 = Component_FeatureCard({ title: WF.i18n.t("why.design"), description: WF.i18n.t("why.design.desc") });
  _e1398.appendChild(_e1402);
  const _e1403 = Component_FeatureCard({ title: WF.i18n.t("why.animation"), description: WF.i18n.t("why.animation.desc") });
  _e1398.appendChild(_e1403);
  const _e1404 = Component_FeatureCard({ title: WF.i18n.t("why.i18n"), description: WF.i18n.t("why.i18n.desc") });
  _e1398.appendChild(_e1404);
  const _e1405 = Component_FeatureCard({ title: WF.i18n.t("why.ssg"), description: WF.i18n.t("why.ssg.desc") });
  _e1398.appendChild(_e1405);
  const _e1406 = Component_FeatureCard({ title: WF.i18n.t("why.a11y"), description: WF.i18n.t("why.a11y.desc") });
  _e1398.appendChild(_e1406);
  const _e1407 = Component_FeatureCard({ title: WF.i18n.t("why.zero"), description: WF.i18n.t("why.zero.desc") });
  _e1398.appendChild(_e1407);
  _e1322.appendChild(_e1398);
  const _e1408 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1408);
  const _e1409 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1410 = WF.h("div", { className: "wf-card__body" });
  const _e1411 = WF.h("div", { className: "wf-row wf-row--center wf-row--between" });
  const _e1412 = WF.h("div", { className: "wf-stack" });
  const _e1413 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("cta.title"));
  _e1412.appendChild(_e1413);
  const _e1414 = WF.h("p", { className: "wf-text wf-text--muted" }, () => WF.i18n.t("cta.subtitle"));
  _e1412.appendChild(_e1414);
  _e1411.appendChild(_e1412);
  const _e1415 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, () => WF.i18n.t("hero.cta"));
  _e1411.appendChild(_e1415);
  _e1410.appendChild(_e1411);
  _e1409.appendChild(_e1410);
  _e1322.appendChild(_e1409);
  const _e1416 = WF.h("div", { className: "wf-spacer" });
  _e1322.appendChild(_e1416);
  _root.appendChild(_e1322);
  return _root;
}

function Page_Components(params) {
  const _activeModal = WF.signal(false);
  const _alertVisible = WF.signal(true);
  const _switchVal = WF.signal(false);
  const _sliderVal = WF.signal(50);
  const _selectVal = WF.signal("opt1");
  const _inputVal = WF.signal("");
  const _checkVal = WF.signal(false);
  const _radioVal = WF.signal("a");
  const _tabActive = WF.signal("preview");
  const _dateVal = WF.signal("");
  const _root = document.createDocumentFragment();
  const _e1417 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1418 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1418);
  const _e1419 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Components Reference");
  _e1417.appendChild(_e1419);
  const _e1420 = WF.h("p", { className: "wf-text wf-text--muted" }, "50+ built-in components. Below are live interactive examples you can play with.");
  _e1417.appendChild(_e1420);
  const _e1421 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1421);
  const _e1422 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Buttons");
  _e1417.appendChild(_e1422);
  const _e1423 = WF.h("p", { className: "wf-text" }, "Buttons support size, color, and shape modifiers.");
  _e1417.appendChild(_e1423);
  const _e1424 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1424);
  const _e1425 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1426 = WF.h("div", { className: "wf-card__body" });
  const _e1427 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1428 = WF.h("button", { className: "wf-btn" }, "Default");
  _e1427.appendChild(_e1428);
  const _e1429 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e1427.appendChild(_e1429);
  const _e1430 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e1427.appendChild(_e1430);
  const _e1431 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e1427.appendChild(_e1431);
  const _e1432 = WF.h("button", { className: "wf-btn wf-btn--warning" }, "Warning");
  _e1427.appendChild(_e1432);
  const _e1433 = WF.h("button", { className: "wf-btn wf-btn--info" }, "Info");
  _e1427.appendChild(_e1433);
  _e1426.appendChild(_e1427);
  const _e1434 = WF.h("div", { className: "wf-spacer" });
  _e1426.appendChild(_e1434);
  const _e1435 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1436 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Small");
  _e1435.appendChild(_e1436);
  const _e1437 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Medium");
  _e1435.appendChild(_e1437);
  const _e1438 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large" }, "Large");
  _e1435.appendChild(_e1438);
  const _e1439 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--rounded" }, "Rounded");
  _e1435.appendChild(_e1439);
  const _e1440 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--full" }, "Full Width");
  _e1435.appendChild(_e1440);
  _e1426.appendChild(_e1435);
  _e1425.appendChild(_e1426);
  _e1417.appendChild(_e1425);
  const _e1441 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1441);
  const _e1442 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1443 = WF.h("div", { className: "wf-card__body" });
  const _e1444 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Primary\", primary)\nButton(\"Large\", primary, large)\nButton(\"Rounded\", success, rounded)\nButton(\"Full Width\", danger, full)");
  _e1443.appendChild(_e1444);
  _e1442.appendChild(_e1443);
  _e1417.appendChild(_e1442);
  const _e1445 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1445);
  const _e1446 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1446);
  const _e1447 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1447);
  const _e1448 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Cards");
  _e1417.appendChild(_e1448);
  const _e1449 = WF.h("p", { className: "wf-text" }, "Cards are surfaces for grouping content. They support Header, Body, and Footer sub-components.");
  _e1417.appendChild(_e1449);
  const _e1450 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1450);
  const _e1451 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1452 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e1453 = WF.h("div", { className: "wf-card" });
  const _e1454 = WF.h("div", { className: "wf-card__header" });
  const _e1455 = WF.h("p", { className: "wf-text wf-text--bold" }, "Default Card");
  _e1454.appendChild(_e1455);
  _e1453.appendChild(_e1454);
  const _e1456 = WF.h("div", { className: "wf-card__body" });
  const _e1457 = WF.h("p", { className: "wf-text wf-text--muted" }, "Basic card with header and body.");
  _e1456.appendChild(_e1457);
  _e1453.appendChild(_e1456);
  const _e1458 = WF.h("div", { className: "wf-card__footer" });
  const _e1459 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Action");
  console.log("clicked");
  _e1458.appendChild(_e1459);
  _e1453.appendChild(_e1458);
  _e1452.appendChild(_e1453);
  _e1451.appendChild(_e1452);
  const _e1460 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e1461 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1462 = WF.h("div", { className: "wf-card__header" });
  const _e1463 = WF.h("p", { className: "wf-text wf-text--bold" }, "Elevated");
  _e1462.appendChild(_e1463);
  _e1461.appendChild(_e1462);
  const _e1464 = WF.h("div", { className: "wf-card__body" });
  const _e1465 = WF.h("p", { className: "wf-text wf-text--muted" }, "Card with shadow elevation.");
  _e1464.appendChild(_e1465);
  _e1461.appendChild(_e1464);
  _e1460.appendChild(_e1461);
  _e1451.appendChild(_e1460);
  const _e1466 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e1467 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1468 = WF.h("div", { className: "wf-card__header" });
  const _e1469 = WF.h("p", { className: "wf-text wf-text--bold" }, "Outlined");
  _e1468.appendChild(_e1469);
  _e1467.appendChild(_e1468);
  const _e1470 = WF.h("div", { className: "wf-card__body" });
  const _e1471 = WF.h("p", { className: "wf-text wf-text--muted" }, "Card with border only.");
  _e1470.appendChild(_e1471);
  _e1467.appendChild(_e1470);
  _e1466.appendChild(_e1467);
  _e1451.appendChild(_e1466);
  _e1417.appendChild(_e1451);
  const _e1472 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1472);
  const _e1473 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1473);
  const _e1474 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1474);
  const _e1475 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Form Controls");
  _e1417.appendChild(_e1475);
  const _e1476 = WF.h("p", { className: "wf-text" }, "All form inputs support two-way binding with the bind: attribute.");
  _e1417.appendChild(_e1476);
  const _e1477 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1477);
  const _e1478 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1479 = WF.h("div", { className: "wf-card__body" });
  const _e1480 = WF.h("div", { className: "wf-stack wf-stack--gap-md" });
  const _e1481 = WF.h("input", { className: "wf-input", value: () => _inputVal(), "on:input": (e) => _inputVal.set(e.target.value), label: "Text Input", placeholder: "Type here...", type: "text" });
  _e1480.appendChild(_e1481);
  WF.condRender(_e1480,
    () => (_inputVal() !== ""),
    () => {
      const _e1482 = document.createDocumentFragment();
      const _e1483 = WF.h("p", { className: "wf-text wf-text--primary wf-text--bold" }, () => `You typed: ${_inputVal()}`);
      _e1482.appendChild(_e1483);
      return _e1482;
    },
    null,
    null
  );
  const _e1484 = WF.h("hr", { className: "wf-divider" });
  _e1480.appendChild(_e1484);
  const _e1485 = WF.h("select", { className: "wf-select", value: () => _selectVal(), "on:input": (e) => _selectVal.set(e.target.value), label: "Select" });
  const _e1486 = WF.h("option", {}, "opt1");
  _e1485.appendChild(_e1486);
  const _e1487 = WF.h("option", {}, "opt2");
  _e1485.appendChild(_e1487);
  const _e1488 = WF.h("option", {}, "opt3");
  _e1485.appendChild(_e1488);
  _e1480.appendChild(_e1485);
  const _e1489 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Selected: ${_selectVal()}`);
  _e1480.appendChild(_e1489);
  const _e1490 = WF.h("hr", { className: "wf-divider" });
  _e1480.appendChild(_e1490);
  const _e1491 = WF.h("label", { className: "wf-checkbox" });
  const _e1492 = WF.h("input", { type: "checkbox", checked: () => _checkVal(), "on:change": () => _checkVal.set(!_checkVal()) });
  _e1491.appendChild(_e1492);
  _e1491.appendChild(WF.text("I agree to the terms"));
  _e1480.appendChild(_e1491);
  const _e1493 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Checked: ${_checkVal()}`);
  _e1480.appendChild(_e1493);
  const _e1494 = WF.h("hr", { className: "wf-divider" });
  _e1480.appendChild(_e1494);
  const _e1495 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e1496 = WF.h("label", { className: "wf-radio" });
  const _e1497 = WF.h("input", { type: "radio", checked: () => _radioVal() === "a", "on:change": () => _radioVal.set("a") });
  _e1496.appendChild(_e1497);
  _e1496.appendChild(WF.text("Option A"));
  _e1495.appendChild(_e1496);
  const _e1498 = WF.h("label", { className: "wf-radio" });
  const _e1499 = WF.h("input", { type: "radio", checked: () => _radioVal() === "b", "on:change": () => _radioVal.set("b") });
  _e1498.appendChild(_e1499);
  _e1498.appendChild(WF.text("Option B"));
  _e1495.appendChild(_e1498);
  const _e1500 = WF.h("label", { className: "wf-radio" });
  const _e1501 = WF.h("input", { type: "radio", checked: () => _radioVal() === "c", "on:change": () => _radioVal.set("c") });
  _e1500.appendChild(_e1501);
  _e1500.appendChild(WF.text("Option C"));
  _e1495.appendChild(_e1500);
  _e1480.appendChild(_e1495);
  const _e1502 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Selected: ${_radioVal()}`);
  _e1480.appendChild(_e1502);
  const _e1503 = WF.h("hr", { className: "wf-divider" });
  _e1480.appendChild(_e1503);
  const _e1504 = WF.h("label", { className: "wf-switch" });
  const _e1505 = WF.h("input", { type: "checkbox", checked: () => _switchVal(), "on:change": () => _switchVal.set(!_switchVal()) });
  _e1504.appendChild(_e1505);
  const _e1506 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1504.appendChild(_e1506);
  _e1504.appendChild(WF.text("Dark Mode"));
  _e1480.appendChild(_e1504);
  const _e1507 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Enabled: ${_switchVal()}`);
  _e1480.appendChild(_e1507);
  const _e1508 = WF.h("hr", { className: "wf-divider" });
  _e1480.appendChild(_e1508);
  const _e1509 = WF.h("div", { className: "wf-slider" });
  const _e1510 = WF.h("label", { className: "wf-form-label" }, "Volume");
  _e1509.appendChild(_e1510);
  const _e1511 = WF.h("input", { type: "range", min: 0, max: 100, step: 1, value: () => _sliderVal(), "on:input": (e) => _sliderVal.set(Number(e.target.value)) });
  _e1509.appendChild(_e1511);
  const _e1512 = WF.h("span", { className: "wf-slider__value" }, () => String(_sliderVal()));
  _e1509.appendChild(_e1512);
  _e1480.appendChild(_e1509);
  const _e1513 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Value: ${_sliderVal()}`);
  _e1480.appendChild(_e1513);
  const _e1514 = WF.h("hr", { className: "wf-divider" });
  _e1480.appendChild(_e1514);
  const _e1516 = WF.h("div", { className: "wf-form-group" });
  const _e1517 = WF.h("label", { className: "wf-form-label" }, "Pick a Date");
  _e1516.appendChild(_e1517);
  const _e1518 = WF.h("input", { type: "date", className: "wf-input", value: () => _dateVal(), "on:change": (e) => _dateVal.set(e.target.value) });
  _e1516.appendChild(_e1518);
  const _e1515 = _e1516;
  _e1480.appendChild(_e1515);
  const _e1519 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Selected: ${_dateVal()}`);
  _e1480.appendChild(_e1519);
  const _e1520 = WF.h("hr", { className: "wf-divider" });
  _e1480.appendChild(_e1520);
  const _e1522 = WF.h("div", { className: "wf-file-upload" });
  const _e1523 = WF.h("label", { className: "wf-form-label" }, "Upload Image");
  _e1522.appendChild(_e1523);
  const _e1524 = WF.h("input", { type: "file", className: "wf-input", accept: "image/*" });
  _e1522.appendChild(_e1524);
  const _e1521 = _e1522;
  _e1480.appendChild(_e1521);
  const _e1526 = WF.h("div", { className: "wf-file-upload" });
  const _e1527 = WF.h("label", { className: "wf-form-label" }, "Documents");
  _e1526.appendChild(_e1527);
  const _e1528 = WF.h("input", { type: "file", className: "wf-input", accept: ".pdf,.doc" });
  _e1526.appendChild(_e1528);
  const _e1525 = _e1526;
  _e1480.appendChild(_e1525);
  _e1479.appendChild(_e1480);
  _e1478.appendChild(_e1479);
  _e1417.appendChild(_e1478);
  const _e1529 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1529);
  const _e1530 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1530);
  const _e1531 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1531);
  const _e1532 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Feedback");
  _e1417.appendChild(_e1532);
  const _e1533 = WF.h("p", { className: "wf-text" }, "Alerts, modals, progress bars, and loading indicators.");
  _e1417.appendChild(_e1533);
  const _e1534 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1534);
  const _e1535 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1536 = WF.h("div", { className: "wf-card__body" });
  const _e1537 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1538 = WF.h("div", { className: "wf-alert wf-alert--success" }, "This is a success alert.");
  _e1537.appendChild(_e1538);
  const _e1539 = WF.h("div", { className: "wf-alert wf-alert--warning" }, "This is a warning alert.");
  _e1537.appendChild(_e1539);
  const _e1540 = WF.h("div", { className: "wf-alert wf-alert--danger" }, "This is a danger alert.");
  _e1537.appendChild(_e1540);
  const _e1541 = WF.h("div", { className: "wf-alert wf-alert--info" }, "This is an info alert.");
  _e1537.appendChild(_e1541);
  _e1536.appendChild(_e1537);
  const _e1542 = WF.h("div", { className: "wf-spacer" });
  _e1536.appendChild(_e1542);
  const _e1543 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1544 = WF.h("div", { className: "wf-spinner" });
  _e1543.appendChild(_e1544);
  const _e1545 = WF.h("div", { className: "wf-spinner wf-spinner--large wf-spinner--primary" });
  _e1543.appendChild(_e1545);
  const _e1546 = WF.h("progress", { className: "wf-progress", value: _sliderVal(), max: 100 });
  _e1543.appendChild(_e1546);
  _e1536.appendChild(_e1543);
  const _e1547 = WF.h("div", { className: "wf-spacer" });
  _e1536.appendChild(_e1547);
  const _e1548 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { _activeModal.set(true); } }, "Open Modal");
  _e1536.appendChild(_e1548);
  _e1535.appendChild(_e1536);
  _e1417.appendChild(_e1535);
  const _e1549 = WF.h("div", { className: "wf-modal" });
  const _e1550 = WF.h("div", { className: "wf-modal__content" });
  const _e1551 = WF.h("div", { className: "wf-modal__header" }, WF.h("h3", {}, "Example Modal"));
  _e1550.appendChild(_e1551);
  const _e1552 = WF.h("div", { className: "wf-modal__body" });
  const _e1553 = WF.h("p", { className: "wf-text" }, "This is a real modal dialog. It was triggered by clicking the button.");
  _e1552.appendChild(_e1553);
  const _e1554 = WF.h("div", { className: "wf-spacer" });
  _e1552.appendChild(_e1554);
  const _e1555 = WF.h("p", { className: "wf-text wf-text--muted" }, "The modal is controlled by a state variable.");
  _e1552.appendChild(_e1555);
  _e1550.appendChild(_e1552);
  const _e1556 = WF.h("div", { className: "wf-modal__footer" });
  const _e1557 = WF.h("button", { className: "wf-btn", "on:click": (e) => { _activeModal.set(false); } }, "Close");
  _e1556.appendChild(_e1557);
  const _e1558 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { _activeModal.set(false); } }, "Confirm");
  _e1556.appendChild(_e1558);
  _e1550.appendChild(_e1556);
  _e1549.appendChild(_e1550);
  WF.effect(() => { _e1549.className = _activeModal() ? 'wf-modal open' : 'wf-modal'; });
  _e1417.appendChild(_e1549);
  const _e1559 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1559);
  const _e1560 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1560);
  const _e1561 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1561);
  const _e1562 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Display");
  _e1417.appendChild(_e1562);
  const _e1563 = WF.h("p", { className: "wf-text" }, "Tables, badges, avatars, tags, and tooltips.");
  _e1417.appendChild(_e1563);
  const _e1564 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1564);
  const _e1565 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1566 = WF.h("div", { className: "wf-card__body" });
  const _e1567 = WF.h("table", { className: "wf-table" });
  const _e1568 = WF.h("thead", {});
  const _e1569 = WF.h("td", {}, "Name");
  _e1568.appendChild(_e1569);
  const _e1570 = WF.h("td", {}, "Role");
  _e1568.appendChild(_e1570);
  const _e1571 = WF.h("td", {}, "Status");
  _e1568.appendChild(_e1571);
  _e1567.appendChild(_e1568);
  const _e1572 = WF.h("tr", {});
  const _e1573 = WF.h("td", {}, "Monzer Omer");
  _e1572.appendChild(_e1573);
  const _e1574 = WF.h("td", {}, "Creator");
  _e1572.appendChild(_e1574);
  const _e1575 = WF.h("td", {}, "Active");
  _e1572.appendChild(_e1575);
  _e1567.appendChild(_e1572);
  const _e1576 = WF.h("tr", {});
  const _e1577 = WF.h("td", {}, "Sara Ali");
  _e1576.appendChild(_e1577);
  const _e1578 = WF.h("td", {}, "Designer");
  _e1576.appendChild(_e1578);
  const _e1579 = WF.h("td", {}, "Active");
  _e1576.appendChild(_e1579);
  _e1567.appendChild(_e1576);
  const _e1580 = WF.h("tr", {});
  const _e1581 = WF.h("td", {}, "Omar Hassan");
  _e1580.appendChild(_e1581);
  const _e1582 = WF.h("td", {}, "Developer");
  _e1580.appendChild(_e1582);
  const _e1583 = WF.h("td", {}, "Away");
  _e1580.appendChild(_e1583);
  _e1567.appendChild(_e1580);
  _e1566.appendChild(_e1567);
  const _e1584 = WF.h("div", { className: "wf-spacer" });
  _e1566.appendChild(_e1584);
  const _e1585 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1586 = WF.h("div", { className: "wf-avatar wf-avatar--primary" }, "MO");
  _e1585.appendChild(_e1586);
  const _e1587 = WF.h("div", { className: "wf-avatar" }, "SA");
  _e1585.appendChild(_e1587);
  const _e1588 = WF.h("div", { className: "wf-avatar" }, "OH");
  _e1585.appendChild(_e1588);
  const _e1589 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Admin");
  _e1585.appendChild(_e1589);
  const _e1590 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Online");
  _e1585.appendChild(_e1590);
  const _e1591 = WF.h("span", { className: "wf-tag" }, "WebFluent");
  _e1585.appendChild(_e1591);
  const _e1592 = WF.h("span", { className: "wf-tag" }, "Rust");
  _e1585.appendChild(_e1592);
  const _e1593 = WF.h("span", { className: "wf-tag" }, "Open Source");
  _e1585.appendChild(_e1593);
  _e1566.appendChild(_e1585);
  _e1565.appendChild(_e1566);
  _e1417.appendChild(_e1565);
  const _e1594 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1594);
  const _e1595 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1595);
  const _e1596 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1596);
  const _e1597 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Layout");
  _e1417.appendChild(_e1597);
  const _e1598 = WF.h("p", { className: "wf-text" }, "Container, Row, Column, Grid, Stack, Spacer, Divider.");
  _e1417.appendChild(_e1598);
  const _e1599 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1599);
  const _e1600 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1601 = WF.h("div", { className: "wf-card__body" });
  const _e1602 = WF.h("p", { className: "wf-text wf-text--bold" }, "Grid with 3 columns:");
  _e1601.appendChild(_e1602);
  const _e1603 = WF.h("div", { className: "wf-spacer" });
  _e1601.appendChild(_e1603);
  const _e1604 = WF.h("div", { className: "wf-grid wf-grid--gap-sm", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1605 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1606 = WF.h("div", { className: "wf-card__body" });
  const _e1607 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 1");
  _e1606.appendChild(_e1607);
  _e1605.appendChild(_e1606);
  _e1604.appendChild(_e1605);
  const _e1608 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1609 = WF.h("div", { className: "wf-card__body" });
  const _e1610 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 2");
  _e1609.appendChild(_e1610);
  _e1608.appendChild(_e1609);
  _e1604.appendChild(_e1608);
  const _e1611 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1612 = WF.h("div", { className: "wf-card__body" });
  const _e1613 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 3");
  _e1612.appendChild(_e1613);
  _e1611.appendChild(_e1612);
  _e1604.appendChild(_e1611);
  _e1601.appendChild(_e1604);
  const _e1614 = WF.h("div", { className: "wf-spacer" });
  _e1601.appendChild(_e1614);
  const _e1615 = WF.h("p", { className: "wf-text wf-text--bold" }, "Row with Columns (6/6 split):");
  _e1601.appendChild(_e1615);
  const _e1616 = WF.h("div", { className: "wf-spacer" });
  _e1601.appendChild(_e1616);
  const _e1617 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1618 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1619 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1620 = WF.h("div", { className: "wf-card__body" });
  const _e1621 = WF.h("p", { className: "wf-text wf-text--center" }, "Left Half");
  _e1620.appendChild(_e1621);
  _e1619.appendChild(_e1620);
  _e1618.appendChild(_e1619);
  _e1617.appendChild(_e1618);
  const _e1622 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1623 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1624 = WF.h("div", { className: "wf-card__body" });
  const _e1625 = WF.h("p", { className: "wf-text wf-text--center" }, "Right Half");
  _e1624.appendChild(_e1625);
  _e1623.appendChild(_e1624);
  _e1622.appendChild(_e1623);
  _e1617.appendChild(_e1622);
  _e1601.appendChild(_e1617);
  const _e1626 = WF.h("div", { className: "wf-spacer" });
  _e1601.appendChild(_e1626);
  const _e1627 = WF.h("p", { className: "wf-text wf-text--bold" }, "Stack (vertical):");
  _e1601.appendChild(_e1627);
  const _e1628 = WF.h("div", { className: "wf-spacer" });
  _e1601.appendChild(_e1628);
  const _e1629 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1630 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1631 = WF.h("div", { className: "wf-card__body" });
  const _e1632 = WF.h("p", { className: "wf-text" }, "Item 1");
  _e1631.appendChild(_e1632);
  _e1630.appendChild(_e1631);
  _e1629.appendChild(_e1630);
  const _e1633 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1634 = WF.h("div", { className: "wf-card__body" });
  const _e1635 = WF.h("p", { className: "wf-text" }, "Item 2");
  _e1634.appendChild(_e1635);
  _e1633.appendChild(_e1634);
  _e1629.appendChild(_e1633);
  const _e1636 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1637 = WF.h("div", { className: "wf-card__body" });
  const _e1638 = WF.h("p", { className: "wf-text" }, "Item 3");
  _e1637.appendChild(_e1638);
  _e1636.appendChild(_e1637);
  _e1629.appendChild(_e1636);
  _e1601.appendChild(_e1629);
  _e1600.appendChild(_e1601);
  _e1417.appendChild(_e1600);
  const _e1639 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1639);
  const _e1640 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1640);
  const _e1641 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1641);
  const _e1642 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Icons & Icon Buttons");
  _e1417.appendChild(_e1642);
  const _e1643 = WF.h("p", { className: "wf-text" }, "30 built-in SVG icons. Use Icon for display, IconButton for clickable actions.");
  _e1417.appendChild(_e1643);
  const _e1644 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1644);
  const _e1645 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1646 = WF.h("div", { className: "wf-card__body" });
  const _e1647 = WF.h("p", { className: "wf-text wf-text--bold" }, "Available Icons:");
  _e1646.appendChild(_e1647);
  const _e1648 = WF.h("div", { className: "wf-spacer" });
  _e1646.appendChild(_e1648);
  const _e1649 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1650 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1651 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1652 = WF.h("i", { className: "wf-icon" }, "home");
  _e1651.appendChild(_e1652);
  const _e1653 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "home");
  _e1651.appendChild(_e1653);
  _e1650.appendChild(_e1651);
  const _e1654 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1655 = WF.h("i", { className: "wf-icon" }, "search");
  _e1654.appendChild(_e1655);
  const _e1656 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "search");
  _e1654.appendChild(_e1656);
  _e1650.appendChild(_e1654);
  const _e1657 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1658 = WF.h("i", { className: "wf-icon" }, "user");
  _e1657.appendChild(_e1658);
  const _e1659 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "user");
  _e1657.appendChild(_e1659);
  _e1650.appendChild(_e1657);
  const _e1660 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1661 = WF.h("i", { className: "wf-icon" }, "settings");
  _e1660.appendChild(_e1661);
  const _e1662 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "settings");
  _e1660.appendChild(_e1662);
  _e1650.appendChild(_e1660);
  const _e1663 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1664 = WF.h("i", { className: "wf-icon" }, "mail");
  _e1663.appendChild(_e1664);
  const _e1665 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "mail");
  _e1663.appendChild(_e1665);
  _e1650.appendChild(_e1663);
  const _e1666 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1667 = WF.h("i", { className: "wf-icon" }, "bell");
  _e1666.appendChild(_e1667);
  const _e1668 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "bell");
  _e1666.appendChild(_e1668);
  _e1650.appendChild(_e1666);
  _e1649.appendChild(_e1650);
  const _e1669 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1670 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1671 = WF.h("i", { className: "wf-icon" }, "edit");
  _e1670.appendChild(_e1671);
  const _e1672 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "edit");
  _e1670.appendChild(_e1672);
  _e1669.appendChild(_e1670);
  const _e1673 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1674 = WF.h("i", { className: "wf-icon" }, "trash");
  _e1673.appendChild(_e1674);
  const _e1675 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "trash");
  _e1673.appendChild(_e1675);
  _e1669.appendChild(_e1673);
  const _e1676 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1677 = WF.h("i", { className: "wf-icon" }, "plus");
  _e1676.appendChild(_e1677);
  const _e1678 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "plus");
  _e1676.appendChild(_e1678);
  _e1669.appendChild(_e1676);
  const _e1679 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1680 = WF.h("i", { className: "wf-icon" }, "check");
  _e1679.appendChild(_e1680);
  const _e1681 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "check");
  _e1679.appendChild(_e1681);
  _e1669.appendChild(_e1679);
  const _e1682 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1683 = WF.h("i", { className: "wf-icon" }, "close");
  _e1682.appendChild(_e1683);
  const _e1684 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "close");
  _e1682.appendChild(_e1684);
  _e1669.appendChild(_e1682);
  const _e1685 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1686 = WF.h("i", { className: "wf-icon" }, "copy");
  _e1685.appendChild(_e1686);
  const _e1687 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "copy");
  _e1685.appendChild(_e1687);
  _e1669.appendChild(_e1685);
  _e1649.appendChild(_e1669);
  const _e1688 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1689 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1690 = WF.h("i", { className: "wf-icon" }, "star");
  _e1689.appendChild(_e1690);
  const _e1691 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "star");
  _e1689.appendChild(_e1691);
  _e1688.appendChild(_e1689);
  const _e1692 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1693 = WF.h("i", { className: "wf-icon" }, "heart");
  _e1692.appendChild(_e1693);
  const _e1694 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "heart");
  _e1692.appendChild(_e1694);
  _e1688.appendChild(_e1692);
  const _e1695 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1696 = WF.h("i", { className: "wf-icon" }, "eye");
  _e1695.appendChild(_e1696);
  const _e1697 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "eye");
  _e1695.appendChild(_e1697);
  _e1688.appendChild(_e1695);
  const _e1698 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1699 = WF.h("i", { className: "wf-icon" }, "download");
  _e1698.appendChild(_e1699);
  const _e1700 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "download");
  _e1698.appendChild(_e1700);
  _e1688.appendChild(_e1698);
  const _e1701 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1702 = WF.h("i", { className: "wf-icon" }, "upload");
  _e1701.appendChild(_e1702);
  const _e1703 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "upload");
  _e1701.appendChild(_e1703);
  _e1688.appendChild(_e1701);
  const _e1704 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1705 = WF.h("i", { className: "wf-icon" }, "link");
  _e1704.appendChild(_e1705);
  const _e1706 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "link");
  _e1704.appendChild(_e1706);
  _e1688.appendChild(_e1704);
  _e1649.appendChild(_e1688);
  const _e1707 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1708 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1709 = WF.h("i", { className: "wf-icon" }, "calendar");
  _e1708.appendChild(_e1709);
  const _e1710 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "calendar");
  _e1708.appendChild(_e1710);
  _e1707.appendChild(_e1708);
  const _e1711 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1712 = WF.h("i", { className: "wf-icon" }, "filter");
  _e1711.appendChild(_e1712);
  const _e1713 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "filter");
  _e1711.appendChild(_e1713);
  _e1707.appendChild(_e1711);
  const _e1714 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1715 = WF.h("i", { className: "wf-icon" }, "info");
  _e1714.appendChild(_e1715);
  const _e1716 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "info");
  _e1714.appendChild(_e1716);
  _e1707.appendChild(_e1714);
  const _e1717 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1718 = WF.h("i", { className: "wf-icon" }, "warning");
  _e1717.appendChild(_e1718);
  const _e1719 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "warning");
  _e1717.appendChild(_e1719);
  _e1707.appendChild(_e1717);
  const _e1720 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1721 = WF.h("i", { className: "wf-icon" }, "logout");
  _e1720.appendChild(_e1721);
  const _e1722 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "logout");
  _e1720.appendChild(_e1722);
  _e1707.appendChild(_e1720);
  const _e1723 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1724 = WF.h("i", { className: "wf-icon" }, "menu");
  _e1723.appendChild(_e1724);
  const _e1725 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "menu");
  _e1723.appendChild(_e1725);
  _e1707.appendChild(_e1723);
  _e1649.appendChild(_e1707);
  _e1646.appendChild(_e1649);
  const _e1726 = WF.h("div", { className: "wf-spacer" });
  _e1646.appendChild(_e1726);
  const _e1727 = WF.h("p", { className: "wf-text wf-text--bold" }, "Icon Buttons:");
  _e1646.appendChild(_e1727);
  const _e1728 = WF.h("div", { className: "wf-spacer" });
  _e1646.appendChild(_e1728);
  const _e1729 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1730 = WF.h("button", { className: "wf-icon-btn", "data-icon": "edit", "aria-label": "Edit", title: "Edit" }, WF.h("span", { className: "wf-icon", "data-icon": "edit" }));
  _e1729.appendChild(_e1730);
  const _e1731 = WF.h("button", { className: "wf-icon-btn wf-icon-btn--danger", "data-icon": "trash", "aria-label": "Delete", title: "Delete" }, WF.h("span", { className: "wf-icon", "data-icon": "trash" }));
  _e1729.appendChild(_e1731);
  const _e1732 = WF.h("button", { className: "wf-icon-btn wf-icon-btn--primary", "data-icon": "plus", "aria-label": "Add", title: "Add" }, WF.h("span", { className: "wf-icon", "data-icon": "plus" }));
  _e1729.appendChild(_e1732);
  const _e1733 = WF.h("button", { className: "wf-icon-btn wf-icon-btn--large", "data-icon": "search", "aria-label": "Search", title: "Search" }, WF.h("span", { className: "wf-icon", "data-icon": "search" }));
  _e1729.appendChild(_e1733);
  const _e1734 = WF.h("button", { className: "wf-icon-btn wf-icon-btn--small", "data-icon": "close", "aria-label": "Close", title: "Close" }, WF.h("span", { className: "wf-icon", "data-icon": "close" }));
  _e1729.appendChild(_e1734);
  _e1646.appendChild(_e1729);
  _e1645.appendChild(_e1646);
  _e1417.appendChild(_e1645);
  const _e1735 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1735);
  const _e1736 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1737 = WF.h("div", { className: "wf-card__body" });
  const _e1738 = WF.h("code", { className: "wf-code wf-code--block" }, "Icon(\"home\")\nIcon(\"search\", large, primary)\nIconButton(icon: \"edit\", label: \"Edit\")\nIconButton(icon: \"trash\", label: \"Delete\", danger)");
  _e1737.appendChild(_e1738);
  _e1736.appendChild(_e1737);
  _e1417.appendChild(_e1736);
  const _e1739 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1739);
  const _e1740 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1740);
  const _e1741 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1741);
  const _e1742 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Tooltips");
  _e1417.appendChild(_e1742);
  const _e1743 = WF.h("p", { className: "wf-text" }, "Wrap any element in a Tooltip to show text on hover.");
  _e1417.appendChild(_e1743);
  const _e1744 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1744);
  const _e1745 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1746 = WF.h("div", { className: "wf-card__body" });
  const _e1747 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1748 = WF.h("div", { className: "wf-tooltip" });
  const _e1749 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Hover me");
  _e1748.appendChild(_e1749);
  const _e1750 = WF.h("span", { className: "wf-tooltip__text", role: "tooltip" }, "This is a primary button");
  _e1748.appendChild(_e1750);
  _e1747.appendChild(_e1748);
  const _e1751 = WF.h("div", { className: "wf-tooltip" });
  const _e1752 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Delete");
  _e1751.appendChild(_e1752);
  const _e1753 = WF.h("span", { className: "wf-tooltip__text", role: "tooltip" }, "Deletes the item permanently");
  _e1751.appendChild(_e1753);
  _e1747.appendChild(_e1751);
  const _e1754 = WF.h("div", { className: "wf-tooltip" });
  const _e1755 = WF.h("div", { className: "wf-avatar wf-avatar--primary" }, "MO");
  _e1754.appendChild(_e1755);
  const _e1756 = WF.h("span", { className: "wf-tooltip__text", role: "tooltip" }, "User profile picture");
  _e1754.appendChild(_e1756);
  _e1747.appendChild(_e1754);
  _e1746.appendChild(_e1747);
  _e1745.appendChild(_e1746);
  _e1417.appendChild(_e1745);
  const _e1757 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1757);
  const _e1758 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1759 = WF.h("div", { className: "wf-card__body" });
  const _e1760 = WF.h("code", { className: "wf-code wf-code--block" }, "Tooltip(text: \"Help text\") {\n    Button(\"Hover me\", primary)\n}");
  _e1759.appendChild(_e1760);
  _e1758.appendChild(_e1759);
  _e1417.appendChild(_e1758);
  const _e1761 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1761);
  const _e1762 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1762);
  const _e1763 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1763);
  const _e1764 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Sidebar");
  _e1417.appendChild(_e1764);
  const _e1765 = WF.h("p", { className: "wf-text" }, "Sidebar navigation with header, items, and dividers. Items support icons and links.");
  _e1417.appendChild(_e1765);
  const _e1766 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1766);
  const _e1767 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1768 = WF.h("div", { className: "wf-card__body" });
  const _e1769 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e1770 = WF.h("aside", { className: "wf-sidebar" });
  const _e1771 = WF.h("div", { className: "wf-sidebar__header" });
  const _e1772 = WF.h("p", { className: "wf-text wf-text--heading" }, "My App");
  _e1771.appendChild(_e1772);
  _e1770.appendChild(_e1771);
  const _e1773 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/" });
  _e1773.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "home" }));
  const _e1774 = WF.h("p", { className: "wf-text" }, "Dashboard");
  _e1773.appendChild(_e1774);
  _e1770.appendChild(_e1773);
  const _e1775 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/components" });
  _e1775.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "settings" }));
  const _e1776 = WF.h("p", { className: "wf-text" }, "Settings");
  _e1775.appendChild(_e1776);
  _e1770.appendChild(_e1775);
  const _e1777 = WF.h("div", { className: "wf-sidebar__item" });
  _e1777.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "user" }));
  const _e1778 = WF.h("p", { className: "wf-text" }, "Profile");
  _e1777.appendChild(_e1778);
  _e1770.appendChild(_e1777);
  _e1770.appendChild(WF.h("div", { className: "wf-sidebar__divider" }));
  const _e1779 = WF.h("div", { className: "wf-sidebar__item" });
  _e1779.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "logout" }));
  const _e1780 = WF.h("p", { className: "wf-text" }, "Logout");
  _e1779.appendChild(_e1780);
  _e1770.appendChild(_e1779);
  _e1769.appendChild(_e1770);
  const _e1781 = WF.h("div", { className: "wf-stack" });
  const _e1782 = WF.h("p", { className: "wf-text wf-text--muted" }, "The sidebar renders with proper structure:");
  _e1781.appendChild(_e1782);
  const _e1783 = WF.h("p", { className: "wf-text wf-text--small" }, "Sidebar.Header for the title");
  _e1781.appendChild(_e1783);
  const _e1784 = WF.h("p", { className: "wf-text wf-text--small" }, "Sidebar.Item with to: and icon: props");
  _e1781.appendChild(_e1784);
  const _e1785 = WF.h("p", { className: "wf-text wf-text--small" }, "Sidebar.Divider for separation");
  _e1781.appendChild(_e1785);
  _e1769.appendChild(_e1781);
  _e1768.appendChild(_e1769);
  _e1767.appendChild(_e1768);
  _e1417.appendChild(_e1767);
  const _e1786 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1786);
  const _e1787 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1788 = WF.h("div", { className: "wf-card__body" });
  const _e1789 = WF.h("code", { className: "wf-code wf-code--block" }, "Sidebar {\n    Sidebar.Header { Text(\"My App\", heading) }\n    Sidebar.Item(to: \"/\", icon: \"home\") { Text(\"Dashboard\") }\n    Sidebar.Item(icon: \"settings\") { Text(\"Settings\") }\n    Sidebar.Divider()\n    Sidebar.Item(icon: \"logout\") { Text(\"Logout\") }\n}");
  _e1788.appendChild(_e1789);
  _e1787.appendChild(_e1788);
  _e1417.appendChild(_e1787);
  const _e1790 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1790);
  const _e1791 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1791);
  const _e1792 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1792);
  const _e1793 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Breadcrumb");
  _e1417.appendChild(_e1793);
  const _e1794 = WF.h("p", { className: "wf-text" }, "Show navigation hierarchy with automatic separators.");
  _e1417.appendChild(_e1794);
  const _e1795 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1795);
  const _e1796 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1797 = WF.h("div", { className: "wf-card__body" });
  const _e1798 = WF.h("nav", { className: "wf-breadcrumb", "aria-label": "breadcrumb" });
  const _e1799 = WF.h("a", { className: "wf-breadcrumb__item", href: WF._basePath + "/" });
  const _e1800 = WF.h("p", { className: "wf-text" }, "Home");
  _e1799.appendChild(_e1800);
  _e1798.appendChild(_e1799);
  const _e1801 = WF.h("a", { className: "wf-breadcrumb__item", href: WF._basePath + "/components" });
  const _e1802 = WF.h("p", { className: "wf-text" }, "Components");
  _e1801.appendChild(_e1802);
  _e1798.appendChild(_e1801);
  const _e1803 = WF.h("span", { className: "wf-breadcrumb__item" });
  const _e1804 = WF.h("p", { className: "wf-text" }, "Breadcrumb");
  _e1803.appendChild(_e1804);
  _e1798.appendChild(_e1803);
  _e1797.appendChild(_e1798);
  _e1796.appendChild(_e1797);
  _e1417.appendChild(_e1796);
  const _e1805 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1805);
  const _e1806 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1807 = WF.h("div", { className: "wf-card__body" });
  const _e1808 = WF.h("code", { className: "wf-code wf-code--block" }, "Breadcrumb {\n    Breadcrumb.Item(to: \"/\") { Text(\"Home\") }\n    Breadcrumb.Item(to: \"/docs\") { Text(\"Docs\") }\n    Breadcrumb.Item { Text(\"Current Page\") }\n}");
  _e1807.appendChild(_e1808);
  _e1806.appendChild(_e1807);
  _e1417.appendChild(_e1806);
  const _e1809 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1809);
  const _e1810 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1810);
  const _e1811 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1811);
  const _e1812 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Skeleton Loading");
  _e1417.appendChild(_e1812);
  const _e1813 = WF.h("p", { className: "wf-text" }, "Placeholder shapes that shimmer while content loads.");
  _e1417.appendChild(_e1813);
  const _e1814 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1814);
  const _e1815 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1816 = WF.h("div", { className: "wf-card__body" });
  const _e1817 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1818 = WF.h("div", { className: "wf-skeleton" });
  _e1818.style.height = "16px";
  _e1818.style.width = "80%";
  _e1817.appendChild(_e1818);
  const _e1819 = WF.h("div", { className: "wf-skeleton" });
  _e1819.style.height = "16px";
  _e1819.style.width = "60%";
  _e1817.appendChild(_e1819);
  const _e1820 = WF.h("div", { className: "wf-skeleton" });
  _e1820.style.height = "16px";
  _e1820.style.width = "40%";
  _e1817.appendChild(_e1820);
  const _e1821 = WF.h("div", { className: "wf-spacer" });
  _e1817.appendChild(_e1821);
  const _e1822 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1823 = WF.h("div", { className: "wf-skeleton" });
  _e1822.appendChild(_e1823);
  const _e1824 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1825 = WF.h("div", { className: "wf-skeleton" });
  _e1825.style.height = "14px";
  _e1825.style.width = "120px";
  _e1824.appendChild(_e1825);
  const _e1826 = WF.h("div", { className: "wf-skeleton" });
  _e1826.style.height = "12px";
  _e1826.style.width = "80px";
  _e1824.appendChild(_e1826);
  _e1822.appendChild(_e1824);
  _e1817.appendChild(_e1822);
  _e1816.appendChild(_e1817);
  _e1815.appendChild(_e1816);
  _e1417.appendChild(_e1815);
  const _e1827 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1827);
  const _e1828 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1829 = WF.h("div", { className: "wf-card__body" });
  const _e1830 = WF.h("code", { className: "wf-code wf-code--block" }, "Skeleton(height: \"16px\", width: \"80%\")\nSkeleton(circle, size: \"48px\")");
  _e1829.appendChild(_e1830);
  _e1828.appendChild(_e1829);
  _e1417.appendChild(_e1828);
  const _e1831 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1831);
  const _e1832 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1832);
  const _e1833 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1833);
  const _e1834 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Dropdown & Menu");
  _e1417.appendChild(_e1834);
  const _e1835 = WF.h("p", { className: "wf-text" }, "Click-to-toggle dropdown menus with auto-close on outside click.");
  _e1417.appendChild(_e1835);
  const _e1836 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1836);
  const _e1837 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1838 = WF.h("div", { className: "wf-card__body" });
  const _e1839 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e1841 = WF.signal(false);
  const _e1840 = WF.h("div", { className: () => _e1841() ? "wf-dropdown open" : "wf-dropdown" });
  const _e1842 = WF.h("button", { className: "wf-btn", "on:click": () => _e1841.set(!_e1841()) }, "Actions");
  _e1840.appendChild(_e1842);
  const _e1843 = WF.h("div", { className: "wf-dropdown__items" });
  const _e1844 = WF.h("li", { className: "wf-dropdown__item" });
  const _e1845 = WF.h("p", { className: "wf-text" }, "Edit");
  _e1844.appendChild(_e1845);
  _e1843.appendChild(_e1844);
  const _e1846 = WF.h("li", { className: "wf-dropdown__item" });
  const _e1847 = WF.h("p", { className: "wf-text" }, "Duplicate");
  _e1846.appendChild(_e1847);
  _e1843.appendChild(_e1846);
  const _e1848 = WF.h("div", { className: "wf-dropdown__divider" });
  _e1843.appendChild(_e1848);
  const _e1849 = WF.h("li", { className: "wf-dropdown__item" });
  const _e1850 = WF.h("p", { className: "wf-text" }, "Delete");
  _e1849.appendChild(_e1850);
  _e1843.appendChild(_e1849);
  _e1840.appendChild(_e1843);
  document.addEventListener('click', (e) => { if (!_e1840.contains(e.target)) _e1841.set(false); });
  _e1839.appendChild(_e1840);
  const _e1852 = WF.signal(false);
  const _e1851 = WF.h("div", { className: () => _e1852() ? "wf-menu open" : "wf-menu" });
  const _e1853 = WF.h("button", { className: "wf-btn", "on:click": () => _e1852.set(!_e1852()) }, "Options");
  _e1851.appendChild(_e1853);
  const _e1854 = WF.h("div", { className: "wf-menu__items" });
  const _e1855 = WF.h("li", { className: "wf-menu__item" });
  const _e1856 = WF.h("p", { className: "wf-text" }, "Profile");
  _e1855.appendChild(_e1856);
  _e1854.appendChild(_e1855);
  const _e1857 = WF.h("li", { className: "wf-menu__item" });
  const _e1858 = WF.h("p", { className: "wf-text" }, "Settings");
  _e1857.appendChild(_e1858);
  _e1854.appendChild(_e1857);
  const _e1859 = WF.h("div", { className: "wf-menu__divider" });
  _e1854.appendChild(_e1859);
  const _e1860 = WF.h("li", { className: "wf-menu__item" });
  const _e1861 = WF.h("p", { className: "wf-text" }, "Logout");
  _e1860.appendChild(_e1861);
  _e1854.appendChild(_e1860);
  _e1851.appendChild(_e1854);
  document.addEventListener('click', (e) => { if (!_e1851.contains(e.target)) _e1852.set(false); });
  _e1839.appendChild(_e1851);
  _e1838.appendChild(_e1839);
  _e1837.appendChild(_e1838);
  _e1417.appendChild(_e1837);
  const _e1862 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1862);
  const _e1863 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1864 = WF.h("div", { className: "wf-card__body" });
  const _e1865 = WF.h("code", { className: "wf-code wf-code--block" }, "Dropdown(label: \"Actions\") {\n    Dropdown.Item { Text(\"Edit\") }\n    Dropdown.Divider()\n    Dropdown.Item { Text(\"Delete\") }\n}");
  _e1864.appendChild(_e1865);
  _e1863.appendChild(_e1864);
  _e1417.appendChild(_e1863);
  const _e1866 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1866);
  const _e1867 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1867);
  const _e1868 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1868);
  const _e1869 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Navigation");
  _e1417.appendChild(_e1869);
  const _e1870 = WF.h("p", { className: "wf-text" }, "Tabs let you switch between content panels.");
  _e1417.appendChild(_e1870);
  const _e1871 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1871);
  const _e1872 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1873 = WF.h("div", { className: "wf-card__body" });
  const _e1874 = WF.h("div", { className: "wf-tabs" });
  const _e1875 = WF.h("div", { className: "wf-tabs__nav" });
  const _e1876 = WF.signal(0);
  const _e1877 = WF.h("button", { className: () => _e1876() === 0 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1876.set(0) }, "Profile");
  _e1875.appendChild(_e1877);
  const _e1878 = WF.h("button", { className: () => _e1876() === 1 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1876.set(1) }, "Settings");
  _e1875.appendChild(_e1878);
  const _e1879 = WF.h("button", { className: () => _e1876() === 2 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1876.set(2) }, "About");
  _e1875.appendChild(_e1879);
  _e1874.appendChild(_e1875);
  const _e1880 = WF.h("div", { className: "wf-tab-page" });
  const _e1881 = WF.h("div", { className: "wf-spacer" });
  _e1880.appendChild(_e1881);
  const _e1882 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1883 = WF.h("div", { className: "wf-avatar wf-avatar--primary wf-avatar--large" }, "MO");
  _e1882.appendChild(_e1883);
  const _e1884 = WF.h("div", { className: "wf-stack" });
  const _e1885 = WF.h("p", { className: "wf-text wf-text--bold" }, "Monzer Omer");
  _e1884.appendChild(_e1885);
  const _e1886 = WF.h("p", { className: "wf-text wf-text--muted" }, "Creator of WebFluent");
  _e1884.appendChild(_e1886);
  _e1882.appendChild(_e1884);
  _e1880.appendChild(_e1882);
  WF.effect(() => { _e1880.style.display = _e1876() === 0 ? 'block' : 'none'; });
  _e1874.appendChild(_e1880);
  const _e1887 = WF.h("div", { className: "wf-tab-page" });
  const _e1888 = WF.h("div", { className: "wf-spacer" });
  _e1887.appendChild(_e1888);
  const _e1889 = WF.h("label", { className: "wf-switch" });
  const _e1890 = WF.h("input", { type: "checkbox", checked: () => _switchVal(), "on:change": () => _switchVal.set(!_switchVal()) });
  _e1889.appendChild(_e1890);
  const _e1891 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1889.appendChild(_e1891);
  _e1889.appendChild(WF.text("Enable notifications"));
  _e1887.appendChild(_e1889);
  const _e1892 = WF.h("div", { className: "wf-spacer" });
  _e1887.appendChild(_e1892);
  const _e1893 = WF.h("div", { className: "wf-slider" });
  const _e1894 = WF.h("label", { className: "wf-form-label" }, "Volume");
  _e1893.appendChild(_e1894);
  const _e1895 = WF.h("input", { type: "range", min: 0, max: 100, step: 1, value: () => _sliderVal(), "on:input": (e) => _sliderVal.set(Number(e.target.value)) });
  _e1893.appendChild(_e1895);
  const _e1896 = WF.h("span", { className: "wf-slider__value" }, () => String(_sliderVal()));
  _e1893.appendChild(_e1896);
  _e1887.appendChild(_e1893);
  WF.effect(() => { _e1887.style.display = _e1876() === 1 ? 'block' : 'none'; });
  _e1874.appendChild(_e1887);
  const _e1897 = WF.h("div", { className: "wf-tab-page" });
  const _e1898 = WF.h("div", { className: "wf-spacer" });
  _e1897.appendChild(_e1898);
  const _e1899 = WF.h("p", { className: "wf-text" }, "WebFluent is a web-first programming language.");
  _e1897.appendChild(_e1899);
  const _e1900 = WF.h("p", { className: "wf-text wf-text--muted" }, "It compiles to HTML, CSS, and JavaScript.");
  _e1897.appendChild(_e1900);
  WF.effect(() => { _e1897.style.display = _e1876() === 2 ? 'block' : 'none'; });
  _e1874.appendChild(_e1897);
  _e1873.appendChild(_e1874);
  _e1872.appendChild(_e1873);
  _e1417.appendChild(_e1872);
  const _e1901 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1901);
  const _e1902 = WF.h("hr", { className: "wf-divider" });
  _e1417.appendChild(_e1902);
  const _e1903 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1903);
  const _e1904 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Typography");
  _e1417.appendChild(_e1904);
  const _e1905 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1905);
  const _e1906 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1907 = WF.h("div", { className: "wf-card__body" });
  const _e1908 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Heading h2");
  _e1907.appendChild(_e1908);
  const _e1909 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Heading h2");
  _e1907.appendChild(_e1909);
  const _e1910 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Heading h3");
  _e1907.appendChild(_e1910);
  const _e1911 = WF.h("div", { className: "wf-spacer" });
  _e1907.appendChild(_e1911);
  const _e1912 = WF.h("p", { className: "wf-text" }, "Normal text paragraph.");
  _e1907.appendChild(_e1912);
  const _e1913 = WF.h("p", { className: "wf-text wf-text--bold" }, "Bold text.");
  _e1907.appendChild(_e1913);
  const _e1914 = WF.h("p", { className: "wf-text wf-text--muted" }, "Muted text.");
  _e1907.appendChild(_e1914);
  const _e1915 = WF.h("p", { className: "wf-text wf-text--primary" }, "Primary colored.");
  _e1907.appendChild(_e1915);
  const _e1916 = WF.h("p", { className: "wf-text wf-text--danger" }, "Danger colored.");
  _e1907.appendChild(_e1916);
  const _e1917 = WF.h("p", { className: "wf-text wf-text--small" }, "Small text.");
  _e1907.appendChild(_e1917);
  const _e1918 = WF.h("p", { className: "wf-text wf-text--uppercase" }, "Uppercase.");
  _e1907.appendChild(_e1918);
  const _e1919 = WF.h("p", { className: "wf-text wf-text--center" }, "Centered text.");
  _e1907.appendChild(_e1919);
  const _e1920 = WF.h("div", { className: "wf-spacer" });
  _e1907.appendChild(_e1920);
  const _e1921 = WF.h("blockquote", { className: "wf-blockquote" }, "The best way to predict the future is to create it.");
  _e1907.appendChild(_e1921);
  const _e1922 = WF.h("div", { className: "wf-spacer" });
  _e1907.appendChild(_e1922);
  const _e1923 = WF.h("code", { className: "wf-code" }, "const greeting = \"Hello, WebFluent!\";");
  _e1907.appendChild(_e1923);
  _e1906.appendChild(_e1907);
  _e1417.appendChild(_e1906);
  const _e1924 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1924);
  const _e1925 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1926 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e1925.appendChild(_e1926);
  const _e1927 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/animation"); } }, "Animation System");
  _e1925.appendChild(_e1927);
  _e1417.appendChild(_e1925);
  const _e1928 = WF.h("div", { className: "wf-spacer" });
  _e1417.appendChild(_e1928);
  _root.appendChild(_e1417);
  return _root;
}

function Page_Accessibility(params) {
  const _root = document.createDocumentFragment();
  const _e1929 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1930 = WF.h("div", { className: "wf-spacer" });
  _e1929.appendChild(_e1930);
  const _e1931 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Accessibility Linting");
  _e1929.appendChild(_e1931);
  const _e1932 = WF.h("p", { className: "wf-text wf-text--muted" }, "WebFluent checks your code for accessibility issues at compile time. Warnings are printed during build but never block compilation.");
  _e1929.appendChild(_e1932);
  const _e1933 = WF.h("div", { className: "wf-spacer" });
  _e1929.appendChild(_e1933);
  const _e1934 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "How It Works");
  _e1929.appendChild(_e1934);
  const _e1935 = WF.h("p", { className: "wf-text" }, "The linter runs automatically after parsing, before code generation. It walks the AST and checks each component against 12 rules.");
  _e1929.appendChild(_e1935);
  const _e1936 = WF.h("div", { className: "wf-spacer" });
  _e1929.appendChild(_e1936);
  const _e1937 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1938 = WF.h("div", { className: "wf-card__body" });
  const _e1939 = WF.h("code", { className: "wf-code wf-code--block" }, "$ wf build\nBuilding my-app...\n  Warning [A01]: Image missing \"alt\" attribute at src/pages/Home.wf:12:5\n    Add alt text: Image(src: \"...\", alt: \"Description of image\")\n  Warning [A03]: Input missing \"label\" attribute at src/pages/Form.wf:8:9\n    Add a label: Input(text, label: \"Username\")\n  3 pages, 2 components, 1 stores\n  Build complete with 2 accessibility warning(s).");
  _e1938.appendChild(_e1939);
  _e1937.appendChild(_e1938);
  _e1929.appendChild(_e1937);
  const _e1940 = WF.h("div", { className: "wf-spacer" });
  _e1929.appendChild(_e1940);
  const _e1941 = WF.h("hr", { className: "wf-divider" });
  _e1929.appendChild(_e1941);
  const _e1942 = WF.h("div", { className: "wf-spacer" });
  _e1929.appendChild(_e1942);
  const _e1943 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Lint Rules");
  _e1929.appendChild(_e1943);
  const _e1944 = WF.h("table", { className: "wf-table" });
  const _e1945 = WF.h("thead", {});
  const _e1946 = WF.h("td", {}, "Rule");
  _e1945.appendChild(_e1946);
  const _e1947 = WF.h("td", {}, "Component");
  _e1945.appendChild(_e1947);
  const _e1948 = WF.h("td", {}, "Check");
  _e1945.appendChild(_e1948);
  _e1944.appendChild(_e1945);
  const _e1949 = WF.h("tr", {});
  const _e1950 = WF.h("td", {}, "A01");
  _e1949.appendChild(_e1950);
  const _e1951 = WF.h("td", {}, "Image");
  _e1949.appendChild(_e1951);
  const _e1952 = WF.h("td", {}, "Must have alt attribute");
  _e1949.appendChild(_e1952);
  _e1944.appendChild(_e1949);
  const _e1953 = WF.h("tr", {});
  const _e1954 = WF.h("td", {}, "A02");
  _e1953.appendChild(_e1954);
  const _e1955 = WF.h("td", {}, "IconButton");
  _e1953.appendChild(_e1955);
  const _e1956 = WF.h("td", {}, "Must have label attribute (no visible text)");
  _e1953.appendChild(_e1956);
  _e1944.appendChild(_e1953);
  const _e1957 = WF.h("tr", {});
  const _e1958 = WF.h("td", {}, "A03");
  _e1957.appendChild(_e1958);
  const _e1959 = WF.h("td", {}, "Input");
  _e1957.appendChild(_e1959);
  const _e1960 = WF.h("td", {}, "Must have label or placeholder");
  _e1957.appendChild(_e1960);
  _e1944.appendChild(_e1957);
  const _e1961 = WF.h("tr", {});
  const _e1962 = WF.h("td", {}, "A04");
  _e1961.appendChild(_e1962);
  const _e1963 = WF.h("td", {}, "Checkbox, Radio, Switch, Slider");
  _e1961.appendChild(_e1963);
  const _e1964 = WF.h("td", {}, "Must have label attribute");
  _e1961.appendChild(_e1964);
  _e1944.appendChild(_e1961);
  const _e1965 = WF.h("tr", {});
  const _e1966 = WF.h("td", {}, "A05");
  _e1965.appendChild(_e1966);
  const _e1967 = WF.h("td", {}, "Button");
  _e1965.appendChild(_e1967);
  const _e1968 = WF.h("td", {}, "Must have text content");
  _e1965.appendChild(_e1968);
  _e1944.appendChild(_e1965);
  const _e1969 = WF.h("tr", {});
  const _e1970 = WF.h("td", {}, "A06");
  _e1969.appendChild(_e1970);
  const _e1971 = WF.h("td", {}, "Link");
  _e1969.appendChild(_e1971);
  const _e1972 = WF.h("td", {}, "Must have text content or children");
  _e1969.appendChild(_e1972);
  _e1944.appendChild(_e1969);
  const _e1973 = WF.h("tr", {});
  const _e1974 = WF.h("td", {}, "A07");
  _e1973.appendChild(_e1974);
  const _e1975 = WF.h("td", {}, "Heading");
  _e1973.appendChild(_e1975);
  const _e1976 = WF.h("td", {}, "Must not be empty");
  _e1973.appendChild(_e1976);
  _e1944.appendChild(_e1973);
  const _e1977 = WF.h("tr", {});
  const _e1978 = WF.h("td", {}, "A08");
  _e1977.appendChild(_e1978);
  const _e1979 = WF.h("td", {}, "Modal, Dialog");
  _e1977.appendChild(_e1979);
  const _e1980 = WF.h("td", {}, "Must have title attribute");
  _e1977.appendChild(_e1980);
  _e1944.appendChild(_e1977);
  const _e1981 = WF.h("tr", {});
  const _e1982 = WF.h("td", {}, "A09");
  _e1981.appendChild(_e1982);
  const _e1983 = WF.h("td", {}, "Video");
  _e1981.appendChild(_e1983);
  const _e1984 = WF.h("td", {}, "Must have controls attribute");
  _e1981.appendChild(_e1984);
  _e1944.appendChild(_e1981);
  const _e1985 = WF.h("tr", {});
  const _e1986 = WF.h("td", {}, "A10");
  _e1985.appendChild(_e1986);
  const _e1987 = WF.h("td", {}, "Table");
  _e1985.appendChild(_e1987);
  const _e1988 = WF.h("td", {}, "Must have Thead header row");
  _e1985.appendChild(_e1988);
  _e1944.appendChild(_e1985);
  const _e1989 = WF.h("tr", {});
  const _e1990 = WF.h("td", {}, "A11");
  _e1989.appendChild(_e1990);
  const _e1991 = WF.h("td", {}, "Heading");
  _e1989.appendChild(_e1991);
  const _e1992 = WF.h("td", {}, "Levels must not skip (h1 to h3)");
  _e1989.appendChild(_e1992);
  _e1944.appendChild(_e1989);
  const _e1993 = WF.h("tr", {});
  const _e1994 = WF.h("td", {}, "A12");
  _e1993.appendChild(_e1994);
  const _e1995 = WF.h("td", {}, "Page");
  _e1993.appendChild(_e1995);
  const _e1996 = WF.h("td", {}, "Must have exactly one h1");
  _e1993.appendChild(_e1996);
  _e1944.appendChild(_e1993);
  _e1929.appendChild(_e1944);
  const _e1997 = WF.h("div", { className: "wf-spacer" });
  _e1929.appendChild(_e1997);
  const _e1998 = WF.h("hr", { className: "wf-divider" });
  _e1929.appendChild(_e1998);
  const _e1999 = WF.h("div", { className: "wf-spacer" });
  _e1929.appendChild(_e1999);
  const _e2000 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Examples");
  _e1929.appendChild(_e2000);
  const _e2001 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e2002 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e2003 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e2004 = WF.h("div", { className: "wf-card__body" });
  const _e2005 = WF.h("p", { className: "wf-text wf-text--danger wf-text--bold" }, "Bad (triggers warning)");
  _e2004.appendChild(_e2005);
  const _e2006 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\")\nIconButton(icon: \"close\")\nInput(text)\nCheckbox(bind: agreed)\nButton()");
  _e2004.appendChild(_e2006);
  _e2003.appendChild(_e2004);
  _e2002.appendChild(_e2003);
  _e2001.appendChild(_e2002);
  const _e2007 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e2008 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e2009 = WF.h("div", { className: "wf-card__body" });
  const _e2010 = WF.h("p", { className: "wf-text wf-text--success wf-text--bold" }, "Good (no warnings)");
  _e2009.appendChild(_e2010);
  const _e2011 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\", alt: \"Team photo\")\nIconButton(icon: \"close\", label: \"Close\")\nInput(text, label: \"Username\")\nCheckbox(bind: agreed, label: \"I agree\")\nButton(\"Save\")");
  _e2009.appendChild(_e2011);
  _e2008.appendChild(_e2009);
  _e2007.appendChild(_e2008);
  _e2001.appendChild(_e2007);
  _e1929.appendChild(_e2001);
  const _e2012 = WF.h("div", { className: "wf-spacer" });
  _e1929.appendChild(_e2012);
  _root.appendChild(_e1929);
  return _root;
}

(function() {
  const _app = document.getElementById('app');
  _app.innerHTML = '';
  const _e2013 = Component_NavBar({});
  _app.appendChild(_e2013);
  const _e2014 = WF.h("div", { className: "wf-row" });
  _app.appendChild(_e2014);
  const _e2015 = Component_DocSidebar({});
  _e2014.appendChild(_e2015);
  const _routerEl = document.createElement('div');
  _routerEl.id = 'wf-router';
  _routerEl.style.flex = '1';
  _e2014.appendChild(_routerEl);
  const _e2016 = Component_SiteFooter({});
  _app.appendChild(_e2016);
  const _routes = [
    { path: "/", render: (params) => Page_Home(params) },
    { path: "/getting-started", render: (params) => Page_GettingStarted(params) },
    { path: "/guide", render: (params) => Page_Guide(params) },
    { path: "/components", render: (params) => Page_Components(params) },
    { path: "/styling", render: (params) => Page_Styling(params) },
    { path: "/animation", render: (params) => Page_Animation(params) },
    { path: "/i18n", render: (params) => Page_I18n(params) },
    { path: "/ssg", render: (params) => Page_Ssg(params) },
    { path: "/pdf", render: (params) => Page_Pdf(params) },
    { path: "/template-engine", render: (params) => Page_TemplateEngine(params) },
    { path: "/accessibility", render: (params) => Page_Accessibility(params) },
    { path: "/cli", render: (params) => Page_Cli(params) },
    { path: "/404", render: (params) => Page_NotFound(params) },
  ];
  WF.createRouter(_routes, _routerEl);
})();
