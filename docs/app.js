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
        } else if (k === "disabled") {
          if (typeof v === "function") {
            effect(() => { el.disabled = v(); });
          } else {
            el.disabled = v;
          }
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

    // Only track the condition signal — not signals read during rendering
    effect(() => {
      const show = !!condFn();
      if (show === lastShow) return;
      lastShow = show;

      // Remove old nodes
      if (animConfig && animConfig.exit && currentNodes.length) {
        const exitName = animConfig.exit;
        const toRemove = [...currentNodes];
        currentNodes = [];
        const promises = toRemove.map(n => n instanceof Element ? animateOut(n, exitName, animConfig.duration) : Promise.resolve());
        Promise.all(promises).then(() => removeNodes(toRemove));
      } else {
        removeNodes(currentNodes);
        currentNodes = [];
      }

      // Add new nodes (untracked so rendering doesn't subscribe this effect to state signals)
      const renderFn = show ? thenFn : elseFn;
      if (renderFn) {
        const prev = currentEffect;
        currentEffect = null; // Untrack: don't subscribe to signals during render
        try {
          const frag = document.createDocumentFragment();
          const result = renderFn();
          const nodes = [].concat(result).flat().filter(n => n instanceof Node);
          for (const n of nodes) { frag.appendChild(n); currentNodes.push(n); }
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
            const nodes = [].concat(itemFn(item, index)).flat();
            for (const n of nodes) {
              if (n instanceof Node) {
                frag.appendChild(n);
                currentNodes.push(n);
                if (animConfig && animConfig.enter && n instanceof Element) {
                  const delay = animConfig.stagger ? (parseInt(animConfig.stagger) * index) + "ms" : animConfig.delay;
                  animateIn(n, animConfig.enter, animConfig.duration, delay);
                }
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
  // Detect base path for GitHub Pages (e.g., /WebFluent)
  const _basePath = (function() {
    const base = document.querySelector("base");
    if (base) return base.getAttribute("href").replace(/\/$/, "");
    // Auto-detect: if script src is relative (../app.js), we're in a subdir
    const scripts = document.querySelectorAll("script[src]");
    for (const s of scripts) {
      const src = s.getAttribute("src");
      if (src && src.includes("app.js")) {
        const depth = (src.match(/\.\.\//g) || []).length;
        if (depth > 0) {
          const segs = location.pathname.split("/").filter(Boolean);
          return "/" + segs.slice(0, segs.length - depth).join("/");
        }
      }
    }
    return "";
  })();

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

  function navigate(path) {
    if (routerInstance) routerInstance.navigate(path);
    else window.location.href = path;
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

  // ─── Exports ─────────────────────────────────────────
  return {
    signal, effect, computed,
    h, text, reactiveText, appendChildren,
    condRender, listRender, showRender,
    animateIn, animateOut, animateEl,
    createRouter, navigate, getParams,
    createStore,
    createI18n,
    wfFetch, showToast,
    mount, hydrate,
    i18n: null,
  };
})();


function Component_CodeBlock({ code }) {
  const _frag = document.createDocumentFragment();
  const _e0 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1 = WF.h("div", { className: "wf-card__body" });
  const _e2 = WF.h("code", { className: "wf-code wf-code--block" }, code);
  _e1.appendChild(_e2);
  _e0.appendChild(_e1);
  _frag.appendChild(_e0);
  return _frag;
}

function Component_FeatureCard({ title, description }) {
  const _frag = document.createDocumentFragment();
  const _e3 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-scaleIn" });
  const _e4 = WF.h("div", { className: "wf-card__body" });
  const _e5 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, title);
  _e4.appendChild(_e5);
  const _e6 = WF.h("div", { className: "wf-spacer" });
  _e4.appendChild(_e6);
  const _e7 = WF.h("p", { className: "wf-text wf-text--muted" }, description);
  _e4.appendChild(_e7);
  _e3.appendChild(_e4);
  _frag.appendChild(_e3);
  return _frag;
}

function Component_SiteFooter() {
  const _frag = document.createDocumentFragment();
  const _e8 = WF.h("hr", { className: "wf-divider" });
  _frag.appendChild(_e8);
  const _e9 = WF.h("div", { className: "wf-container" });
  const _e10 = WF.h("div", { className: "wf-spacer" });
  _e9.appendChild(_e10);
  const _e11 = WF.h("div", { className: "wf-row wf-row--center wf-row--between" });
  const _e12 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "WebFluent — The Web-First Language");
  _e11.appendChild(_e12);
  const _e13 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e14 = WF.h("a", { className: "wf-link", href: "/", "on:click": (e) => { e.preventDefault(); WF.navigate("/"); } });
  const _e15 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Home");
  _e14.appendChild(_e15);
  _e13.appendChild(_e14);
  const _e16 = WF.h("a", { className: "wf-link", href: "/getting-started", "on:click": (e) => { e.preventDefault(); WF.navigate("/getting-started"); } });
  const _e17 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Docs");
  _e16.appendChild(_e17);
  _e13.appendChild(_e16);
  _e11.appendChild(_e13);
  _e9.appendChild(_e11);
  const _e18 = WF.h("div", { className: "wf-spacer" });
  _e9.appendChild(_e18);
  _frag.appendChild(_e9);
  return _frag;
}

function Component_NavBar() {
  const _frag = document.createDocumentFragment();
  const _e19 = WF.h("nav", { className: "wf-navbar" });
  const _e20 = WF.h("div", { className: "wf-navbar__brand" });
  const _e21 = WF.h("p", { className: "wf-text wf-text--heading" }, "WebFluent");
  _e20.appendChild(_e21);
  _e19.appendChild(_e20);
  const _e22 = WF.h("div", { className: "wf-navbar__links" });
  const _e23 = WF.h("a", { className: "wf-link", href: "/", "on:click": (e) => { e.preventDefault(); WF.navigate("/"); } });
  const _e24 = WF.h("p", { className: "wf-text" }, "Home");
  _e23.appendChild(_e24);
  _e22.appendChild(_e23);
  const _e25 = WF.h("a", { className: "wf-link", href: "/getting-started", "on:click": (e) => { e.preventDefault(); WF.navigate("/getting-started"); } });
  const _e26 = WF.h("p", { className: "wf-text" }, "Get Started");
  _e25.appendChild(_e26);
  _e22.appendChild(_e25);
  const _e27 = WF.h("a", { className: "wf-link", href: "/guide", "on:click": (e) => { e.preventDefault(); WF.navigate("/guide"); } });
  const _e28 = WF.h("p", { className: "wf-text" }, "Guide");
  _e27.appendChild(_e28);
  _e22.appendChild(_e27);
  const _e29 = WF.h("a", { className: "wf-link", href: "/components", "on:click": (e) => { e.preventDefault(); WF.navigate("/components"); } });
  const _e30 = WF.h("p", { className: "wf-text" }, "Components");
  _e29.appendChild(_e30);
  _e22.appendChild(_e29);
  const _e31 = WF.h("a", { className: "wf-link", href: "/styling", "on:click": (e) => { e.preventDefault(); WF.navigate("/styling"); } });
  const _e32 = WF.h("p", { className: "wf-text" }, "Styling");
  _e31.appendChild(_e32);
  _e22.appendChild(_e31);
  const _e33 = WF.h("a", { className: "wf-link", href: "/animation", "on:click": (e) => { e.preventDefault(); WF.navigate("/animation"); } });
  const _e34 = WF.h("p", { className: "wf-text" }, "Animation");
  _e33.appendChild(_e34);
  _e22.appendChild(_e33);
  const _e35 = WF.h("a", { className: "wf-link", href: "/i18n", "on:click": (e) => { e.preventDefault(); WF.navigate("/i18n"); } });
  const _e36 = WF.h("p", { className: "wf-text" }, "i18n");
  _e35.appendChild(_e36);
  _e22.appendChild(_e35);
  const _e37 = WF.h("a", { className: "wf-link", href: "/ssg", "on:click": (e) => { e.preventDefault(); WF.navigate("/ssg"); } });
  const _e38 = WF.h("p", { className: "wf-text" }, "SSG");
  _e37.appendChild(_e38);
  _e22.appendChild(_e37);
  const _e39 = WF.h("a", { className: "wf-link", href: "/accessibility", "on:click": (e) => { e.preventDefault(); WF.navigate("/accessibility"); } });
  const _e40 = WF.h("p", { className: "wf-text" }, "Accessibility");
  _e39.appendChild(_e40);
  _e22.appendChild(_e39);
  const _e41 = WF.h("a", { className: "wf-link", href: "/cli", "on:click": (e) => { e.preventDefault(); WF.navigate("/cli"); } });
  const _e42 = WF.h("p", { className: "wf-text" }, "CLI");
  _e41.appendChild(_e42);
  _e22.appendChild(_e41);
  _e19.appendChild(_e22);
  _frag.appendChild(_e19);
  return _frag;
}

function Page_Ssg(params) {
  const _root = document.createDocumentFragment();
  const _e43 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e44 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e44);
  const _e45 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Static Site Generation (SSG)");
  _e43.appendChild(_e45);
  const _e46 = WF.h("p", { className: "wf-text wf-text--muted" }, "Pre-render pages to HTML at build time for instant content visibility. JavaScript hydrates the page for interactivity.");
  _e43.appendChild(_e46);
  const _e47 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e47);
  const _e48 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Enable SSG");
  _e43.appendChild(_e48);
  const _e49 = WF.h("p", { className: "wf-text" }, "One config flag is all you need.");
  _e43.appendChild(_e49);
  const _e50 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e50);
  const _e51 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e52 = WF.h("div", { className: "wf-card__body" });
  const _e53 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"build\": {\n    \"ssg\": true\n  }\n}");
  _e52.appendChild(_e53);
  _e51.appendChild(_e52);
  _e43.appendChild(_e51);
  const _e54 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e54);
  const _e55 = WF.h("hr", { className: "wf-divider" });
  _e43.appendChild(_e55);
  const _e56 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e56);
  const _e57 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "How It Works");
  _e43.appendChild(_e57);
  const _e58 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e59 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e60 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e61 = WF.h("div", { className: "wf-card__body" });
  const _e62 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "1. Build");
  _e61.appendChild(_e62);
  const _e63 = WF.h("p", { className: "wf-text wf-text--muted" }, "The compiler walks the AST for each page and generates static HTML from the component tree.");
  _e61.appendChild(_e63);
  _e60.appendChild(_e61);
  _e59.appendChild(_e60);
  _e58.appendChild(_e59);
  const _e64 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e65 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e66 = WF.h("div", { className: "wf-card__body" });
  const _e67 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "2. Serve");
  _e66.appendChild(_e67);
  const _e68 = WF.h("p", { className: "wf-text wf-text--muted" }, "The browser loads pre-rendered HTML. Content is visible immediately — no blank white screen.");
  _e66.appendChild(_e68);
  _e65.appendChild(_e66);
  _e64.appendChild(_e65);
  _e58.appendChild(_e64);
  const _e69 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e70 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e71 = WF.h("div", { className: "wf-card__body" });
  const _e72 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "3. Hydrate");
  _e71.appendChild(_e72);
  const _e73 = WF.h("p", { className: "wf-text wf-text--muted" }, "JavaScript runs and hydrates the page: attaches events, initializes state, fills dynamic content.");
  _e71.appendChild(_e73);
  _e70.appendChild(_e71);
  _e69.appendChild(_e70);
  _e58.appendChild(_e69);
  _e43.appendChild(_e58);
  const _e74 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e74);
  const _e75 = WF.h("hr", { className: "wf-divider" });
  _e43.appendChild(_e75);
  const _e76 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e76);
  const _e77 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Build Output");
  _e43.appendChild(_e77);
  const _e78 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e79 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e80 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e81 = WF.h("div", { className: "wf-card__body" });
  const _e82 = WF.h("p", { className: "wf-text wf-text--bold" }, "SPA (default)");
  _e81.appendChild(_e82);
  const _e83 = WF.h("code", { className: "wf-code wf-code--block" }, "build/\n├── index.html       # Empty shell\n├── app.js\n└── styles.css");
  _e81.appendChild(_e83);
  _e80.appendChild(_e81);
  _e79.appendChild(_e80);
  _e78.appendChild(_e79);
  const _e84 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e85 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e86 = WF.h("div", { className: "wf-card__body" });
  const _e87 = WF.h("p", { className: "wf-text wf-text--bold" }, "SSG mode");
  _e86.appendChild(_e87);
  const _e88 = WF.h("code", { className: "wf-code wf-code--block" }, "build/\n├── index.html       # Pre-rendered /\n├── about/\n│   └── index.html   # Pre-rendered /about\n├── blog/\n│   └── index.html   # Pre-rendered /blog\n├── app.js\n└── styles.css");
  _e86.appendChild(_e88);
  _e85.appendChild(_e86);
  _e84.appendChild(_e85);
  _e78.appendChild(_e84);
  _e43.appendChild(_e78);
  const _e89 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e89);
  const _e90 = WF.h("hr", { className: "wf-divider" });
  _e43.appendChild(_e90);
  const _e91 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e91);
  const _e92 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "What Gets Pre-Rendered");
  _e43.appendChild(_e92);
  const _e93 = WF.h("table", { className: "wf-table" });
  const _e94 = WF.h("thead", {});
  const _e95 = WF.h("td", {}, "Element");
  _e94.appendChild(_e95);
  const _e96 = WF.h("td", {}, "SSG Behavior");
  _e94.appendChild(_e96);
  _e93.appendChild(_e94);
  const _e97 = WF.h("tr", {});
  const _e98 = WF.h("td", {}, "Static text, headings, components");
  _e97.appendChild(_e98);
  const _e99 = WF.h("td", {}, "Fully rendered to HTML");
  _e97.appendChild(_e99);
  _e93.appendChild(_e97);
  const _e100 = WF.h("tr", {});
  const _e101 = WF.h("td", {}, "Container, Row, Column, Card, etc.");
  _e100.appendChild(_e101);
  const _e102 = WF.h("td", {}, "Full HTML with classes");
  _e100.appendChild(_e102);
  _e93.appendChild(_e100);
  const _e103 = WF.h("tr", {});
  const _e104 = WF.h("td", {}, "Modifiers (primary, large, etc.)");
  _e103.appendChild(_e104);
  const _e105 = WF.h("td", {}, "CSS classes applied");
  _e103.appendChild(_e105);
  _e93.appendChild(_e103);
  const _e106 = WF.h("tr", {});
  const _e107 = WF.h("td", {}, "Animation modifiers (fadeIn, etc.)");
  _e106.appendChild(_e107);
  const _e108 = WF.h("td", {}, "Animation classes applied");
  _e106.appendChild(_e108);
  _e93.appendChild(_e106);
  const _e109 = WF.h("tr", {});
  const _e110 = WF.h("td", {}, "t() i18n calls");
  _e109.appendChild(_e110);
  const _e111 = WF.h("td", {}, "Default locale text rendered");
  _e109.appendChild(_e111);
  _e93.appendChild(_e109);
  const _e112 = WF.h("tr", {});
  const _e113 = WF.h("td", {}, "State-dependent text");
  _e112.appendChild(_e113);
  const _e114 = WF.h("td", {}, "Empty placeholder (filled by JS)");
  _e112.appendChild(_e114);
  _e93.appendChild(_e112);
  const _e115 = WF.h("tr", {});
  const _e116 = WF.h("td", {}, "if / for blocks");
  _e115.appendChild(_e116);
  const _e117 = WF.h("td", {}, "Comment placeholder (filled by JS)");
  _e115.appendChild(_e117);
  _e93.appendChild(_e115);
  const _e118 = WF.h("tr", {});
  const _e119 = WF.h("td", {}, "show blocks");
  _e118.appendChild(_e119);
  const _e120 = WF.h("td", {}, "Rendered but hidden (display:none)");
  _e118.appendChild(_e120);
  _e93.appendChild(_e118);
  const _e121 = WF.h("tr", {});
  const _e122 = WF.h("td", {}, "fetch blocks");
  _e121.appendChild(_e122);
  const _e123 = WF.h("td", {}, "Loading block if present, else placeholder");
  _e121.appendChild(_e123);
  _e93.appendChild(_e121);
  const _e124 = WF.h("tr", {});
  const _e125 = WF.h("td", {}, "Event handlers");
  _e124.appendChild(_e125);
  const _e126 = WF.h("td", {}, "Attached during hydration");
  _e124.appendChild(_e126);
  _e93.appendChild(_e124);
  _e43.appendChild(_e93);
  const _e127 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e127);
  const _e128 = WF.h("hr", { className: "wf-divider" });
  _e43.appendChild(_e128);
  const _e129 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e129);
  const _e130 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Dynamic Routes");
  _e43.appendChild(_e130);
  const _e131 = WF.h("p", { className: "wf-text wf-text--muted" }, "Pages with :param segments (e.g., /user/:id) cannot be pre-rendered — they fall back to client-side rendering.");
  _e43.appendChild(_e131);
  const _e132 = WF.h("div", { className: "wf-spacer" });
  _e43.appendChild(_e132);
  _root.appendChild(_e43);
  return _root;
}

function Page_Styling(params) {
  const _root = document.createDocumentFragment();
  const _e133 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e134 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e134);
  const _e135 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Design System & Styling");
  _e133.appendChild(_e135);
  const _e136 = WF.h("p", { className: "wf-text wf-text--muted" }, "Token-based design system. Every component uses design tokens for colors, spacing, typography. Change the entire look with a config update.");
  _e133.appendChild(_e136);
  const _e137 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e137);
  const _e138 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Variant Modifiers");
  _e133.appendChild(_e138);
  const _e139 = WF.h("p", { className: "wf-text" }, "Apply common styles with modifier keywords.");
  _e133.appendChild(_e139);
  const _e140 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e140);
  const _e141 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e142 = WF.h("div", { className: "wf-card__header" });
  const _e143 = WF.h("p", { className: "wf-text wf-text--bold" }, "Size Modifiers");
  _e142.appendChild(_e143);
  _e141.appendChild(_e142);
  const _e144 = WF.h("div", { className: "wf-card__body" });
  const _e145 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e146 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Small");
  _e145.appendChild(_e146);
  const _e147 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Medium");
  _e145.appendChild(_e147);
  const _e148 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large" }, "Large");
  _e145.appendChild(_e148);
  _e144.appendChild(_e145);
  _e141.appendChild(_e144);
  _e133.appendChild(_e141);
  const _e149 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e149);
  const _e150 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e151 = WF.h("div", { className: "wf-card__header" });
  const _e152 = WF.h("p", { className: "wf-text wf-text--bold" }, "Color Modifiers");
  _e151.appendChild(_e152);
  _e150.appendChild(_e151);
  const _e153 = WF.h("div", { className: "wf-card__body" });
  const _e154 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e155 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e154.appendChild(_e155);
  const _e156 = WF.h("button", { className: "wf-btn wf-btn--secondary" }, "Secondary");
  _e154.appendChild(_e156);
  const _e157 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e154.appendChild(_e157);
  const _e158 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e154.appendChild(_e158);
  const _e159 = WF.h("button", { className: "wf-btn wf-btn--warning" }, "Warning");
  _e154.appendChild(_e159);
  const _e160 = WF.h("button", { className: "wf-btn wf-btn--info" }, "Info");
  _e154.appendChild(_e160);
  _e153.appendChild(_e154);
  const _e161 = WF.h("div", { className: "wf-spacer" });
  _e153.appendChild(_e161);
  const _e162 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e163 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Primary");
  _e162.appendChild(_e163);
  const _e164 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Success");
  _e162.appendChild(_e164);
  const _e165 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Danger");
  _e162.appendChild(_e165);
  const _e166 = WF.h("span", { className: "wf-badge wf-badge--warning" }, "Warning");
  _e162.appendChild(_e166);
  _e153.appendChild(_e162);
  _e150.appendChild(_e153);
  _e133.appendChild(_e150);
  const _e167 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e167);
  const _e168 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e169 = WF.h("div", { className: "wf-card__header" });
  const _e170 = WF.h("p", { className: "wf-text wf-text--bold" }, "Shape and Elevation");
  _e169.appendChild(_e170);
  _e168.appendChild(_e169);
  const _e171 = WF.h("div", { className: "wf-card__body" });
  const _e172 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e173 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Default");
  _e172.appendChild(_e173);
  const _e174 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--rounded" }, "Rounded");
  _e172.appendChild(_e174);
  const _e175 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--full" }, "Full Width");
  _e172.appendChild(_e175);
  _e171.appendChild(_e172);
  const _e176 = WF.h("div", { className: "wf-spacer" });
  _e171.appendChild(_e176);
  const _e177 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e178 = WF.h("div", { className: "wf-card" });
  const _e179 = WF.h("div", { className: "wf-card__body" });
  const _e180 = WF.h("p", { className: "wf-text" }, "Default");
  _e179.appendChild(_e180);
  _e178.appendChild(_e179);
  _e177.appendChild(_e178);
  const _e181 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e182 = WF.h("div", { className: "wf-card__body" });
  const _e183 = WF.h("p", { className: "wf-text" }, "Elevated");
  _e182.appendChild(_e183);
  _e181.appendChild(_e182);
  _e177.appendChild(_e181);
  const _e184 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e185 = WF.h("div", { className: "wf-card__body" });
  const _e186 = WF.h("p", { className: "wf-text" }, "Outlined");
  _e185.appendChild(_e186);
  _e184.appendChild(_e185);
  _e177.appendChild(_e184);
  _e171.appendChild(_e177);
  _e168.appendChild(_e171);
  _e133.appendChild(_e168);
  const _e187 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e187);
  const _e188 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e189 = WF.h("div", { className: "wf-card__header" });
  const _e190 = WF.h("p", { className: "wf-text wf-text--bold" }, "Text Modifiers");
  _e189.appendChild(_e190);
  _e188.appendChild(_e189);
  const _e191 = WF.h("div", { className: "wf-card__body" });
  const _e192 = WF.h("p", { className: "wf-text wf-text--bold" }, "Bold text.");
  _e191.appendChild(_e192);
  const _e193 = WF.h("p", { className: "wf-text wf-text--italic" }, "Italic text.");
  _e191.appendChild(_e193);
  const _e194 = WF.h("p", { className: "wf-text wf-text--uppercase" }, "Uppercase text.");
  _e191.appendChild(_e194);
  const _e195 = WF.h("p", { className: "wf-text wf-text--muted" }, "Muted text.");
  _e191.appendChild(_e195);
  const _e196 = WF.h("p", { className: "wf-text wf-text--primary" }, "Primary colored text.");
  _e191.appendChild(_e196);
  const _e197 = WF.h("p", { className: "wf-text wf-text--small" }, "Small text.");
  _e191.appendChild(_e197);
  const _e198 = WF.h("p", { className: "wf-text wf-text--large" }, "Large text.");
  _e191.appendChild(_e198);
  _e188.appendChild(_e191);
  _e133.appendChild(_e188);
  const _e199 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e199);
  const _e200 = WF.h("hr", { className: "wf-divider" });
  _e133.appendChild(_e200);
  const _e201 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e201);
  const _e202 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Design Tokens");
  _e133.appendChild(_e202);
  const _e203 = WF.h("p", { className: "wf-text" }, "All styling is built on tokens — CSS custom properties. Override any token in your config.");
  _e133.appendChild(_e203);
  const _e204 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e204);
  const _e205 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e206 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e207 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e208 = WF.h("div", { className: "wf-card__header" });
  const _e209 = WF.h("p", { className: "wf-text wf-text--bold" }, "Colors");
  _e208.appendChild(_e209);
  _e207.appendChild(_e208);
  const _e210 = WF.h("div", { className: "wf-card__body" });
  const _e211 = WF.h("table", { className: "wf-table" });
  const _e212 = WF.h("thead", {});
  const _e213 = WF.h("td", {}, "Token");
  _e212.appendChild(_e213);
  const _e214 = WF.h("td", {}, "Value");
  _e212.appendChild(_e214);
  _e211.appendChild(_e212);
  const _e215 = WF.h("tr", {});
  const _e216 = WF.h("td", {}, "color-primary");
  _e215.appendChild(_e216);
  const _e217 = WF.h("td", {}, "#3B82F6");
  _e215.appendChild(_e217);
  _e211.appendChild(_e215);
  const _e218 = WF.h("tr", {});
  const _e219 = WF.h("td", {}, "color-success");
  _e218.appendChild(_e219);
  const _e220 = WF.h("td", {}, "#22C55E");
  _e218.appendChild(_e220);
  _e211.appendChild(_e218);
  const _e221 = WF.h("tr", {});
  const _e222 = WF.h("td", {}, "color-danger");
  _e221.appendChild(_e222);
  const _e223 = WF.h("td", {}, "#EF4444");
  _e221.appendChild(_e223);
  _e211.appendChild(_e221);
  const _e224 = WF.h("tr", {});
  const _e225 = WF.h("td", {}, "color-warning");
  _e224.appendChild(_e225);
  const _e226 = WF.h("td", {}, "#F59E0B");
  _e224.appendChild(_e226);
  _e211.appendChild(_e224);
  const _e227 = WF.h("tr", {});
  const _e228 = WF.h("td", {}, "color-text");
  _e227.appendChild(_e228);
  const _e229 = WF.h("td", {}, "#0F172A");
  _e227.appendChild(_e229);
  _e211.appendChild(_e227);
  const _e230 = WF.h("tr", {});
  const _e231 = WF.h("td", {}, "color-border");
  _e230.appendChild(_e231);
  const _e232 = WF.h("td", {}, "#E2E8F0");
  _e230.appendChild(_e232);
  _e211.appendChild(_e230);
  _e210.appendChild(_e211);
  _e207.appendChild(_e210);
  _e206.appendChild(_e207);
  _e205.appendChild(_e206);
  const _e233 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e234 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e235 = WF.h("div", { className: "wf-card__header" });
  const _e236 = WF.h("p", { className: "wf-text wf-text--bold" }, "Spacing and Radius");
  _e235.appendChild(_e236);
  _e234.appendChild(_e235);
  const _e237 = WF.h("div", { className: "wf-card__body" });
  const _e238 = WF.h("table", { className: "wf-table" });
  const _e239 = WF.h("thead", {});
  const _e240 = WF.h("td", {}, "Token");
  _e239.appendChild(_e240);
  const _e241 = WF.h("td", {}, "Value");
  _e239.appendChild(_e241);
  _e238.appendChild(_e239);
  const _e242 = WF.h("tr", {});
  const _e243 = WF.h("td", {}, "spacing-xs");
  _e242.appendChild(_e243);
  const _e244 = WF.h("td", {}, "0.25rem");
  _e242.appendChild(_e244);
  _e238.appendChild(_e242);
  const _e245 = WF.h("tr", {});
  const _e246 = WF.h("td", {}, "spacing-sm");
  _e245.appendChild(_e246);
  const _e247 = WF.h("td", {}, "0.5rem");
  _e245.appendChild(_e247);
  _e238.appendChild(_e245);
  const _e248 = WF.h("tr", {});
  const _e249 = WF.h("td", {}, "spacing-md");
  _e248.appendChild(_e249);
  const _e250 = WF.h("td", {}, "1rem");
  _e248.appendChild(_e250);
  _e238.appendChild(_e248);
  const _e251 = WF.h("tr", {});
  const _e252 = WF.h("td", {}, "spacing-lg");
  _e251.appendChild(_e252);
  const _e253 = WF.h("td", {}, "1.5rem");
  _e251.appendChild(_e253);
  _e238.appendChild(_e251);
  const _e254 = WF.h("tr", {});
  const _e255 = WF.h("td", {}, "radius-md");
  _e254.appendChild(_e255);
  const _e256 = WF.h("td", {}, "0.5rem");
  _e254.appendChild(_e256);
  _e238.appendChild(_e254);
  const _e257 = WF.h("tr", {});
  const _e258 = WF.h("td", {}, "radius-full");
  _e257.appendChild(_e258);
  const _e259 = WF.h("td", {}, "9999px");
  _e257.appendChild(_e259);
  _e238.appendChild(_e257);
  _e237.appendChild(_e238);
  _e234.appendChild(_e237);
  _e233.appendChild(_e234);
  _e205.appendChild(_e233);
  _e133.appendChild(_e205);
  const _e260 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e260);
  const _e261 = WF.h("hr", { className: "wf-divider" });
  _e133.appendChild(_e261);
  const _e262 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e262);
  const _e263 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Themes");
  _e133.appendChild(_e263);
  const _e264 = WF.h("p", { className: "wf-text" }, "4 built-in themes. Set in webfluent.app.json.");
  _e133.appendChild(_e264);
  const _e265 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e265);
  const _e266 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e267 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e268 = WF.h("div", { className: "wf-card__body" });
  const _e269 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "default");
  _e268.appendChild(_e269);
  const _e270 = WF.h("div", { className: "wf-spacer" });
  _e268.appendChild(_e270);
  const _e271 = WF.h("p", { className: "wf-text wf-text--muted" }, "Clean, modern light theme.");
  _e268.appendChild(_e271);
  _e267.appendChild(_e268);
  _e266.appendChild(_e267);
  const _e272 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e273 = WF.h("div", { className: "wf-card__body" });
  const _e274 = WF.h("span", { className: "wf-badge wf-badge--secondary" }, "dark");
  _e273.appendChild(_e274);
  const _e275 = WF.h("div", { className: "wf-spacer" });
  _e273.appendChild(_e275);
  const _e276 = WF.h("p", { className: "wf-text wf-text--muted" }, "Dark backgrounds, muted light text.");
  _e273.appendChild(_e276);
  _e272.appendChild(_e273);
  _e266.appendChild(_e272);
  const _e277 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e278 = WF.h("div", { className: "wf-card__body" });
  const _e279 = WF.h("span", { className: "wf-badge" }, "minimal");
  _e278.appendChild(_e279);
  const _e280 = WF.h("div", { className: "wf-spacer" });
  _e278.appendChild(_e280);
  const _e281 = WF.h("p", { className: "wf-text wf-text--muted" }, "Black and white. No shadows or radii.");
  _e278.appendChild(_e281);
  _e277.appendChild(_e278);
  _e266.appendChild(_e277);
  const _e282 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e283 = WF.h("div", { className: "wf-card__body" });
  const _e284 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "brutalist");
  _e283.appendChild(_e284);
  const _e285 = WF.h("div", { className: "wf-spacer" });
  _e283.appendChild(_e285);
  const _e286 = WF.h("p", { className: "wf-text wf-text--muted" }, "Monospace, hard shadows, bold.");
  _e283.appendChild(_e286);
  _e282.appendChild(_e283);
  _e266.appendChild(_e282);
  _e133.appendChild(_e266);
  const _e287 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e287);
  _root.appendChild(_e133);
  return _root;
}

function Page_Guide(params) {
  const _root = document.createDocumentFragment();
  const _e288 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e289 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e289);
  const _e290 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Language Guide");
  _e288.appendChild(_e290);
  const _e291 = WF.h("p", { className: "wf-text wf-text--muted" }, "Learn the core concepts of WebFluent.");
  _e288.appendChild(_e291);
  const _e292 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e292);
  const _e293 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Pages");
  _e288.appendChild(_e293);
  const _e294 = WF.h("p", { className: "wf-text" }, "Pages are top-level route targets. Each page defines a URL path and contains the UI tree for that route.");
  _e288.appendChild(_e294);
  const _e295 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e295);
  const _e296 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e297 = WF.h("div", { className: "wf-card__body" });
  const _e298 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\", title: \"Home\") {\n    Container {\n        Heading(\"Welcome\", h1)\n        Text(\"This is the home page.\")\n    }\n}");
  _e297.appendChild(_e298);
  _e296.appendChild(_e297);
  _e288.appendChild(_e296);
  const _e299 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e299);
  const _e300 = WF.h("p", { className: "wf-text wf-text--bold" }, "Page attributes:");
  _e288.appendChild(_e300);
  const _e301 = WF.h("table", { className: "wf-table" });
  const _e302 = WF.h("thead", {});
  const _e303 = WF.h("td", {}, "Attribute");
  _e302.appendChild(_e303);
  const _e304 = WF.h("td", {}, "Type");
  _e302.appendChild(_e304);
  const _e305 = WF.h("td", {}, "Description");
  _e302.appendChild(_e305);
  _e301.appendChild(_e302);
  const _e306 = WF.h("tr", {});
  const _e307 = WF.h("td", {}, "path");
  _e306.appendChild(_e307);
  const _e308 = WF.h("td", {}, "String");
  _e306.appendChild(_e308);
  const _e309 = WF.h("td", {}, "URL route for this page (required)");
  _e306.appendChild(_e309);
  _e301.appendChild(_e306);
  const _e310 = WF.h("tr", {});
  const _e311 = WF.h("td", {}, "title");
  _e310.appendChild(_e311);
  const _e312 = WF.h("td", {}, "String");
  _e310.appendChild(_e312);
  const _e313 = WF.h("td", {}, "Document title");
  _e310.appendChild(_e313);
  _e301.appendChild(_e310);
  const _e314 = WF.h("tr", {});
  const _e315 = WF.h("td", {}, "guard");
  _e314.appendChild(_e315);
  const _e316 = WF.h("td", {}, "Expression");
  _e314.appendChild(_e316);
  const _e317 = WF.h("td", {}, "Navigation guard — redirects if false");
  _e314.appendChild(_e317);
  _e301.appendChild(_e314);
  const _e318 = WF.h("tr", {});
  const _e319 = WF.h("td", {}, "redirect");
  _e318.appendChild(_e319);
  const _e320 = WF.h("td", {}, "String");
  _e318.appendChild(_e320);
  const _e321 = WF.h("td", {}, "Redirect target when guard fails");
  _e318.appendChild(_e321);
  _e301.appendChild(_e318);
  _e288.appendChild(_e301);
  const _e322 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e322);
  const _e323 = WF.h("hr", { className: "wf-divider" });
  _e288.appendChild(_e323);
  const _e324 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e324);
  const _e325 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Components");
  _e288.appendChild(_e325);
  const _e326 = WF.h("p", { className: "wf-text" }, "Reusable UI blocks that accept props and can have internal state.");
  _e288.appendChild(_e326);
  const _e327 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e327);
  const _e328 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e329 = WF.h("div", { className: "wf-card__body" });
  const _e330 = WF.h("code", { className: "wf-code wf-code--block" }, "Component UserCard (name: String, role: String, active: Bool = true) {\n    Card(elevated) {\n        Row(align: center, gap: md) {\n            Avatar(initials: \"U\", primary)\n            Stack {\n                Text(name, bold)\n                Text(role, muted)\n            }\n            if active {\n                Badge(\"Active\", success)\n            }\n        }\n    }\n}\n\n// Usage\nUserCard(name: \"Monzer\", role: \"Developer\")");
  _e329.appendChild(_e330);
  _e328.appendChild(_e329);
  _e288.appendChild(_e328);
  const _e331 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e331);
  const _e332 = WF.h("p", { className: "wf-text wf-text--muted" }, "Props support types: String, Number, Bool, List, Map. Optional props use ?, defaults use =.");
  _e288.appendChild(_e332);
  const _e333 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e333);
  const _e334 = WF.h("hr", { className: "wf-divider" });
  _e288.appendChild(_e334);
  const _e335 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e335);
  const _e336 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "State and Reactivity");
  _e288.appendChild(_e336);
  const _e337 = WF.h("p", { className: "wf-text" }, "State is declared with the state keyword. It is reactive — any UI that reads it updates automatically when it changes.");
  _e288.appendChild(_e337);
  const _e338 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e338);
  const _e339 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e340 = WF.h("div", { className: "wf-card__body" });
  const _e341 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Counter (path: \"/counter\") {\n    state count = 0\n\n    Container {\n        Text(\"Count: {count}\")\n        Button(\"+1\", primary) { count = count + 1 }\n        Button(\"-1\") { count = count - 1 }\n    }\n}");
  _e340.appendChild(_e341);
  _e339.appendChild(_e340);
  _e288.appendChild(_e339);
  const _e342 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e342);
  const _e343 = WF.h("p", { className: "wf-text wf-text--bold" }, "Derived state:");
  _e288.appendChild(_e343);
  const _e344 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e345 = WF.h("div", { className: "wf-card__body" });
  const _e346 = WF.h("code", { className: "wf-code wf-code--block" }, "state items = [{name: \"A\", price: 3}, {name: \"B\", price: 2}]\nderived total = items.map(i => i.price).sum()\nderived isEmpty = items.length == 0");
  _e345.appendChild(_e346);
  _e344.appendChild(_e345);
  _e288.appendChild(_e344);
  const _e347 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e347);
  const _e348 = WF.h("hr", { className: "wf-divider" });
  _e288.appendChild(_e348);
  const _e349 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e349);
  const _e350 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Events");
  _e288.appendChild(_e350);
  const _e351 = WF.h("p", { className: "wf-text" }, "Event handlers are declared with on:event or via shorthand blocks on buttons.");
  _e288.appendChild(_e351);
  const _e352 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e352);
  const _e353 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e354 = WF.h("div", { className: "wf-card__body" });
  const _e355 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Submit\") {\n    on:click {\n        submitForm()\n    }\n}\n\nInput(text, placeholder: \"Search...\") {\n    on:input {\n        searchQuery = value\n    }\n    on:keydown {\n        if key == \"Enter\" {\n            performSearch()\n        }\n    }\n}\n\n// Shorthand: block on Button defaults to on:click\nButton(\"Save\") { save() }");
  _e354.appendChild(_e355);
  _e353.appendChild(_e354);
  _e288.appendChild(_e353);
  const _e356 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e356);
  const _e357 = WF.h("p", { className: "wf-text wf-text--muted" }, "Supported events: on:click, on:submit, on:input, on:change, on:focus, on:blur, on:keydown, on:keyup, on:mouseover, on:mouseout, on:mount, on:unmount");
  _e288.appendChild(_e357);
  const _e358 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e358);
  const _e359 = WF.h("hr", { className: "wf-divider" });
  _e288.appendChild(_e359);
  const _e360 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e360);
  const _e361 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Control Flow");
  _e288.appendChild(_e361);
  const _e362 = WF.h("p", { className: "wf-text wf-text--bold" }, "Conditionals:");
  _e288.appendChild(_e362);
  const _e363 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e364 = WF.h("div", { className: "wf-card__body" });
  const _e365 = WF.h("code", { className: "wf-code wf-code--block" }, "if isLoggedIn {\n    Text(\"Welcome back!\")\n} else if isGuest {\n    Text(\"Hello, guest\")\n} else {\n    Button(\"Log In\") { navigate(\"/login\") }\n}");
  _e364.appendChild(_e365);
  _e363.appendChild(_e364);
  _e288.appendChild(_e363);
  const _e366 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e366);
  const _e367 = WF.h("p", { className: "wf-text wf-text--bold" }, "Loops:");
  _e288.appendChild(_e367);
  const _e368 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e369 = WF.h("div", { className: "wf-card__body" });
  const _e370 = WF.h("code", { className: "wf-code wf-code--block" }, "for user in users {\n    UserCard(name: user.name, role: user.role)\n}\n\n// With index\nfor item, index in items {\n    Text(\"{index + 1}. {item}\")\n}");
  _e369.appendChild(_e370);
  _e368.appendChild(_e369);
  _e288.appendChild(_e368);
  const _e371 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e371);
  const _e372 = WF.h("p", { className: "wf-text wf-text--bold" }, "Show/Hide (keeps element in DOM, toggles visibility):");
  _e288.appendChild(_e372);
  const _e373 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e374 = WF.h("div", { className: "wf-card__body" });
  const _e375 = WF.h("code", { className: "wf-code wf-code--block" }, "show isExpanded {\n    Card { Text(\"Expanded content\") }\n}");
  _e374.appendChild(_e375);
  _e373.appendChild(_e374);
  _e288.appendChild(_e373);
  const _e376 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e376);
  const _e377 = WF.h("hr", { className: "wf-divider" });
  _e288.appendChild(_e377);
  const _e378 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e378);
  const _e379 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Stores");
  _e288.appendChild(_e379);
  const _e380 = WF.h("p", { className: "wf-text" }, "Stores hold shared state accessible from any page or component.");
  _e288.appendChild(_e380);
  const _e381 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e381);
  const _e382 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e383 = WF.h("div", { className: "wf-card__body" });
  const _e384 = WF.h("code", { className: "wf-code wf-code--block" }, "Store CartStore {\n    state items = []\n\n    derived total = items.map(i => i.price * i.quantity).sum()\n    derived count = items.length\n\n    action addItem(product: Map) {\n        items.push({ id: product.id, name: product.name, price: product.price, quantity: 1 })\n    }\n\n    action removeItem(id: Number) {\n        items = items.filter(i => i.id != id)\n    }\n}\n\n// Usage in a page\nPage Cart (path: \"/cart\") {\n    use CartStore\n\n    Text(\"Total: ${CartStore.total}\")\n    Button(\"Clear\") { CartStore.clear() }\n}");
  _e383.appendChild(_e384);
  _e382.appendChild(_e383);
  _e288.appendChild(_e382);
  const _e385 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e385);
  const _e386 = WF.h("hr", { className: "wf-divider" });
  _e288.appendChild(_e386);
  const _e387 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e387);
  const _e388 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Routing");
  _e288.appendChild(_e388);
  const _e389 = WF.h("p", { className: "wf-text" }, "SPA routing is declared in the App file.");
  _e288.appendChild(_e389);
  const _e390 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e390);
  const _e391 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e392 = WF.h("div", { className: "wf-card__body" });
  const _e393 = WF.h("code", { className: "wf-code wf-code--block" }, "App {\n    Navbar {\n        Navbar.Brand { Text(\"My App\", heading) }\n        Navbar.Links {\n            Link(to: \"/\") { Text(\"Home\") }\n            Link(to: \"/about\") { Text(\"About\") }\n        }\n    }\n\n    Router {\n        Route(path: \"/\", page: Home)\n        Route(path: \"/about\", page: About)\n        Route(path: \"/user/:id\", page: UserProfile)\n        Route(path: \"*\", page: NotFound)\n    }\n}\n\n// Programmatic navigation\nButton(\"Go Home\") { navigate(\"/\") }\n\n// Dynamic routes access params\nPage UserProfile (path: \"/user/:id\") {\n    Text(\"User ID: {params.id}\")\n}");
  _e392.appendChild(_e393);
  _e391.appendChild(_e392);
  _e288.appendChild(_e391);
  const _e394 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e394);
  const _e395 = WF.h("hr", { className: "wf-divider" });
  _e288.appendChild(_e395);
  const _e396 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e396);
  const _e397 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Fetching");
  _e288.appendChild(_e397);
  const _e398 = WF.h("p", { className: "wf-text" }, "Built-in async data loading with automatic loading, error, and success states.");
  _e288.appendChild(_e398);
  const _e399 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e399);
  const _e400 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e401 = WF.h("div", { className: "wf-card__body" });
  const _e402 = WF.h("code", { className: "wf-code wf-code--block" }, "fetch users from \"/api/users\" {\n    loading {\n        Spinner()\n    }\n    error (err) {\n        Alert(\"Failed to load users\", danger)\n    }\n    success {\n        for user in users {\n            UserCard(name: user.name, role: user.role)\n        }\n    }\n}\n\n// With options\nfetch result from \"/api/submit\" (method: \"POST\", body: { name: name, email: email }) {\n    success {\n        Alert(\"Saved!\", success)\n    }\n}");
  _e401.appendChild(_e402);
  _e400.appendChild(_e401);
  _e288.appendChild(_e400);
  const _e403 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e403);
  const _e404 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e405 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/components"); } }, "Components Reference");
  _e404.appendChild(_e405);
  const _e406 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e404.appendChild(_e406);
  _e288.appendChild(_e404);
  const _e407 = WF.h("div", { className: "wf-spacer" });
  _e288.appendChild(_e407);
  _root.appendChild(_e288);
  return _root;
}

function Page_Animation(params) {
  const _showCard = WF.signal(false);
  const _items = WF.signal(["Item A", "Item B", "Item C"]);
  const _root = document.createDocumentFragment();
  const _e408 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e409 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e409);
  const _e410 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Animation System");
  _e408.appendChild(_e410);
  const _e411 = WF.h("p", { className: "wf-text wf-text--muted" }, "Declarative animations built into the language. No CSS keyframes to write.");
  _e408.appendChild(_e411);
  const _e412 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e412);
  const _e413 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Mount Animations");
  _e408.appendChild(_e413);
  const _e414 = WF.h("p", { className: "wf-text" }, "Add an animation modifier to any component. It plays when the element appears.");
  _e408.appendChild(_e414);
  const _e415 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e415);
  const _e416 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e417 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn" });
  const _e418 = WF.h("div", { className: "wf-card__body" });
  const _e419 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "fadeIn");
  _e418.appendChild(_e419);
  const _e420 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Fades from transparent");
  _e418.appendChild(_e420);
  _e417.appendChild(_e418);
  _e416.appendChild(_e417);
  const _e421 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideUp" });
  const _e422 = WF.h("div", { className: "wf-card__body" });
  const _e423 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideUp");
  _e422.appendChild(_e423);
  const _e424 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from below");
  _e422.appendChild(_e424);
  _e421.appendChild(_e422);
  _e416.appendChild(_e421);
  const _e425 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-scaleIn" });
  const _e426 = WF.h("div", { className: "wf-card__body" });
  const _e427 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "scaleIn");
  _e426.appendChild(_e427);
  const _e428 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Scales from 90%");
  _e426.appendChild(_e428);
  _e425.appendChild(_e426);
  _e416.appendChild(_e425);
  const _e429 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideDown" });
  const _e430 = WF.h("div", { className: "wf-card__body" });
  const _e431 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideDown");
  _e430.appendChild(_e431);
  const _e432 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from above");
  _e430.appendChild(_e432);
  _e429.appendChild(_e430);
  _e416.appendChild(_e429);
  const _e433 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideLeft" });
  const _e434 = WF.h("div", { className: "wf-card__body" });
  const _e435 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideLeft");
  _e434.appendChild(_e435);
  const _e436 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from right");
  _e434.appendChild(_e436);
  _e433.appendChild(_e434);
  _e416.appendChild(_e433);
  const _e437 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-bounce" });
  const _e438 = WF.h("div", { className: "wf-card__body" });
  const _e439 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "bounce");
  _e438.appendChild(_e439);
  const _e440 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Bouncy entrance");
  _e438.appendChild(_e440);
  _e437.appendChild(_e438);
  _e416.appendChild(_e437);
  _e408.appendChild(_e416);
  const _e441 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e441);
  const _e442 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e443 = WF.h("div", { className: "wf-card__body" });
  const _e444 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn) { ... }\nHeading(\"Title\", h1, slideUp)\nButton(\"Click\", primary, bounce)");
  _e443.appendChild(_e444);
  _e442.appendChild(_e443);
  _e408.appendChild(_e442);
  const _e445 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e445);
  const _e446 = WF.h("hr", { className: "wf-divider" });
  _e408.appendChild(_e446);
  const _e447 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e447);
  const _e448 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Live: Conditional Animation");
  _e408.appendChild(_e448);
  const _e449 = WF.h("p", { className: "wf-text" }, "Toggle the switch to see enter/exit animations on the card below.");
  _e408.appendChild(_e449);
  const _e450 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e450);
  const _e451 = WF.h("label", { className: "wf-switch" });
  const _e452 = WF.h("input", { type: "checkbox", checked: () => _showCard(), "on:change": () => _showCard.set(!_showCard()) });
  _e451.appendChild(_e452);
  const _e453 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e451.appendChild(_e453);
  _e451.appendChild(WF.text("Show animated card"));
  _e408.appendChild(_e451);
  const _e454 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e454);
  WF.condRender(_e408,
    () => _showCard(),
    () => {
      const _e455 = document.createDocumentFragment();
      const _e456 = WF.h("div", { className: "wf-card wf-card--elevated" });
      const _e457 = WF.h("div", { className: "wf-card__body" });
      const _e458 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Animated!");
      _e457.appendChild(_e458);
      const _e459 = WF.h("div", { className: "wf-spacer" });
      _e457.appendChild(_e459);
      const _e460 = WF.h("p", { className: "wf-text" }, "This card scales in and fades out.");
      _e457.appendChild(_e460);
      const _e461 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Controlled by: if showCard, animate(scaleIn, fadeOut)");
      _e457.appendChild(_e461);
      _e456.appendChild(_e457);
      _e455.appendChild(_e456);
      return _e455;
    },
    null,
    { enter: "scaleIn", exit: "fadeOut" }
  );
  const _e462 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e462);
  const _e463 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e464 = WF.h("div", { className: "wf-card__body" });
  const _e465 = WF.h("code", { className: "wf-code wf-code--block" }, "if showCard, animate(scaleIn, fadeOut) {\n    Card(elevated) {\n        Text(\"Animated content\")\n    }\n}");
  _e464.appendChild(_e465);
  _e463.appendChild(_e464);
  _e408.appendChild(_e463);
  const _e466 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e466);
  const _e467 = WF.h("hr", { className: "wf-divider" });
  _e408.appendChild(_e467);
  const _e468 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e468);
  const _e469 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Speed Variants");
  _e408.appendChild(_e469);
  const _e470 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e470);
  const _e471 = WF.h("table", { className: "wf-table" });
  const _e472 = WF.h("thead", {});
  const _e473 = WF.h("td", {}, "Modifier");
  _e472.appendChild(_e473);
  const _e474 = WF.h("td", {}, "Duration");
  _e472.appendChild(_e474);
  _e471.appendChild(_e472);
  const _e475 = WF.h("tr", {});
  const _e476 = WF.h("td", {}, "fast");
  _e475.appendChild(_e476);
  const _e477 = WF.h("td", {}, "150ms");
  _e475.appendChild(_e477);
  _e471.appendChild(_e475);
  const _e478 = WF.h("tr", {});
  const _e479 = WF.h("td", {}, "(default)");
  _e478.appendChild(_e479);
  const _e480 = WF.h("td", {}, "300ms");
  _e478.appendChild(_e480);
  _e471.appendChild(_e478);
  const _e481 = WF.h("tr", {});
  const _e482 = WF.h("td", {}, "slow");
  _e481.appendChild(_e482);
  const _e483 = WF.h("td", {}, "500ms");
  _e481.appendChild(_e483);
  _e471.appendChild(_e481);
  _e408.appendChild(_e471);
  const _e484 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e484);
  const _e485 = WF.h("hr", { className: "wf-divider" });
  _e408.appendChild(_e485);
  const _e486 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e486);
  const _e487 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "All 12 Animations");
  _e408.appendChild(_e487);
  const _e488 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e488);
  const _e489 = WF.h("table", { className: "wf-table" });
  const _e490 = WF.h("thead", {});
  const _e491 = WF.h("td", {}, "Name");
  _e490.appendChild(_e491);
  const _e492 = WF.h("td", {}, "Effect");
  _e490.appendChild(_e492);
  _e489.appendChild(_e490);
  const _e493 = WF.h("tr", {});
  const _e494 = WF.h("td", {}, "fadeIn / fadeOut");
  _e493.appendChild(_e494);
  const _e495 = WF.h("td", {}, "Opacity fade");
  _e493.appendChild(_e495);
  _e489.appendChild(_e493);
  const _e496 = WF.h("tr", {});
  const _e497 = WF.h("td", {}, "slideUp / slideDown");
  _e496.appendChild(_e497);
  const _e498 = WF.h("td", {}, "Vertical slide with fade");
  _e496.appendChild(_e498);
  _e489.appendChild(_e496);
  const _e499 = WF.h("tr", {});
  const _e500 = WF.h("td", {}, "slideLeft / slideRight");
  _e499.appendChild(_e500);
  const _e501 = WF.h("td", {}, "Horizontal slide with fade");
  _e499.appendChild(_e501);
  _e489.appendChild(_e499);
  const _e502 = WF.h("tr", {});
  const _e503 = WF.h("td", {}, "scaleIn / scaleOut");
  _e502.appendChild(_e503);
  const _e504 = WF.h("td", {}, "Scale from/to 90%");
  _e502.appendChild(_e504);
  _e489.appendChild(_e502);
  const _e505 = WF.h("tr", {});
  const _e506 = WF.h("td", {}, "bounce");
  _e505.appendChild(_e506);
  const _e507 = WF.h("td", {}, "Bouncy entrance");
  _e505.appendChild(_e507);
  _e489.appendChild(_e505);
  const _e508 = WF.h("tr", {});
  const _e509 = WF.h("td", {}, "shake");
  _e508.appendChild(_e509);
  const _e510 = WF.h("td", {}, "Horizontal shake");
  _e508.appendChild(_e510);
  _e489.appendChild(_e508);
  const _e511 = WF.h("tr", {});
  const _e512 = WF.h("td", {}, "pulse");
  _e511.appendChild(_e512);
  const _e513 = WF.h("td", {}, "Gentle scale pulse (infinite)");
  _e511.appendChild(_e513);
  _e489.appendChild(_e511);
  const _e514 = WF.h("tr", {});
  const _e515 = WF.h("td", {}, "spin");
  _e514.appendChild(_e515);
  const _e516 = WF.h("td", {}, "360-degree rotation (infinite)");
  _e514.appendChild(_e516);
  _e489.appendChild(_e514);
  _e408.appendChild(_e489);
  const _e517 = WF.h("div", { className: "wf-spacer" });
  _e408.appendChild(_e517);
  _root.appendChild(_e408);
  return _root;
}

function Page_I18n(params) {
  const _root = document.createDocumentFragment();
  const _e518 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e519 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e519);
  const _e520 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Internationalization (i18n)");
  _e518.appendChild(_e520);
  const _e521 = WF.h("p", { className: "wf-text wf-text--muted" }, "Built-in multi-language support with reactive locale switching and automatic RTL.");
  _e518.appendChild(_e521);
  const _e522 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e522);
  const _e523 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Setup");
  _e518.appendChild(_e523);
  const _e524 = WF.h("p", { className: "wf-text" }, "Create a JSON file per locale in your translations directory.");
  _e518.appendChild(_e524);
  const _e525 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e525);
  const _e526 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e527 = WF.h("div", { className: "wf-card__body" });
  const _e528 = WF.h("code", { className: "wf-code wf-code--block" }, "// src/translations/en.json\n{\n    \"greeting\": \"Hello, {name}!\",\n    \"nav.home\": \"Home\",\n    \"nav.about\": \"About\"\n}\n\n// src/translations/ar.json\n{\n    \"greeting\": \"!أهلاً، {name}\",\n    \"nav.home\": \"الرئيسية\",\n    \"nav.about\": \"حول\"\n}");
  _e527.appendChild(_e528);
  _e526.appendChild(_e527);
  _e518.appendChild(_e526);
  const _e529 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e529);
  const _e530 = WF.h("p", { className: "wf-text wf-text--bold" }, "Add i18n config to webfluent.app.json:");
  _e518.appendChild(_e530);
  const _e531 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e532 = WF.h("div", { className: "wf-card__body" });
  const _e533 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e532.appendChild(_e533);
  _e531.appendChild(_e532);
  _e518.appendChild(_e531);
  const _e534 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e534);
  const _e535 = WF.h("hr", { className: "wf-divider" });
  _e518.appendChild(_e535);
  const _e536 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e536);
  const _e537 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "The t() Function");
  _e518.appendChild(_e537);
  const _e538 = WF.h("p", { className: "wf-text" }, "Use t() to look up translated text. It is reactive — all t() calls update when the locale changes.");
  _e518.appendChild(_e538);
  const _e539 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e539);
  const _e540 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e541 = WF.h("div", { className: "wf-card__body" });
  const _e542 = WF.h("code", { className: "wf-code wf-code--block" }, "// Simple key lookup\nText(t(\"nav.home\"))\n\n// With interpolation\nText(t(\"greeting\", name: user.name))\n\n// In any component\nButton(t(\"actions.save\"), primary)\nHeading(t(\"page.title\"), h1)");
  _e541.appendChild(_e542);
  _e540.appendChild(_e541);
  _e518.appendChild(_e540);
  const _e543 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e543);
  const _e544 = WF.h("hr", { className: "wf-divider" });
  _e518.appendChild(_e544);
  const _e545 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e545);
  const _e546 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Locale Switching");
  _e518.appendChild(_e546);
  const _e547 = WF.h("p", { className: "wf-text" }, "Switch the locale at runtime with setLocale(). All translated text updates instantly.");
  _e518.appendChild(_e547);
  const _e548 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e548);
  const _e549 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e550 = WF.h("div", { className: "wf-card__body" });
  const _e551 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"English\") { setLocale(\"en\") }\nButton(\"العربية\") { setLocale(\"ar\") }\nButton(\"Espanol\") { setLocale(\"es\") }\n\n// Access current locale\nText(\"Current: {locale}\")\nText(\"Direction: {dir}\")");
  _e550.appendChild(_e551);
  _e549.appendChild(_e550);
  _e518.appendChild(_e549);
  const _e552 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e552);
  const _e553 = WF.h("hr", { className: "wf-divider" });
  _e518.appendChild(_e553);
  const _e554 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e554);
  const _e555 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "RTL Support");
  _e518.appendChild(_e555);
  const _e556 = WF.h("p", { className: "wf-text" }, "WebFluent automatically detects RTL locales and updates the document direction.");
  _e518.appendChild(_e556);
  const _e557 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e557);
  const _e558 = WF.h("table", { className: "wf-table" });
  const _e559 = WF.h("thead", {});
  const _e560 = WF.h("td", {}, "Locale");
  _e559.appendChild(_e560);
  const _e561 = WF.h("td", {}, "Direction");
  _e559.appendChild(_e561);
  _e558.appendChild(_e559);
  const _e562 = WF.h("tr", {});
  const _e563 = WF.h("td", {}, "ar (Arabic)");
  _e562.appendChild(_e563);
  const _e564 = WF.h("td", {}, "RTL");
  _e562.appendChild(_e564);
  _e558.appendChild(_e562);
  const _e565 = WF.h("tr", {});
  const _e566 = WF.h("td", {}, "he (Hebrew)");
  _e565.appendChild(_e566);
  const _e567 = WF.h("td", {}, "RTL");
  _e565.appendChild(_e567);
  _e558.appendChild(_e565);
  const _e568 = WF.h("tr", {});
  const _e569 = WF.h("td", {}, "fa (Farsi)");
  _e568.appendChild(_e569);
  const _e570 = WF.h("td", {}, "RTL");
  _e568.appendChild(_e570);
  _e558.appendChild(_e568);
  const _e571 = WF.h("tr", {});
  const _e572 = WF.h("td", {}, "ur (Urdu)");
  _e571.appendChild(_e572);
  const _e573 = WF.h("td", {}, "RTL");
  _e571.appendChild(_e573);
  _e558.appendChild(_e571);
  const _e574 = WF.h("tr", {});
  const _e575 = WF.h("td", {}, "All others");
  _e574.appendChild(_e575);
  const _e576 = WF.h("td", {}, "LTR");
  _e574.appendChild(_e576);
  _e558.appendChild(_e574);
  _e518.appendChild(_e558);
  const _e577 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e577);
  const _e578 = WF.h("p", { className: "wf-text wf-text--muted" }, "When setLocale(\"ar\") is called, the HTML element gets dir=\"rtl\" and lang=\"ar\" automatically.");
  _e518.appendChild(_e578);
  const _e579 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e579);
  const _e580 = WF.h("hr", { className: "wf-divider" });
  _e518.appendChild(_e580);
  const _e581 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e581);
  const _e582 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Fallback Behavior");
  _e518.appendChild(_e582);
  const _e583 = WF.h("p", { className: "wf-text" }, "If a key is missing in the current locale:");
  _e518.appendChild(_e583);
  const _e584 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e585 = WF.h("p", { className: "wf-text" }, "1. Falls back to the defaultLocale translation");
  _e584.appendChild(_e585);
  const _e586 = WF.h("p", { className: "wf-text" }, "2. If still missing, returns the key itself (e.g., \"nav.home\")");
  _e584.appendChild(_e586);
  _e518.appendChild(_e584);
  const _e587 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e587);
  const _e588 = WF.h("hr", { className: "wf-divider" });
  _e518.appendChild(_e588);
  const _e589 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e589);
  const _e590 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "SSG + i18n");
  _e518.appendChild(_e590);
  const _e591 = WF.h("p", { className: "wf-text wf-text--muted" }, "When both SSG and i18n are enabled, pages are pre-rendered with the default locale text. After JavaScript loads, locale switching works normally.");
  _e518.appendChild(_e591);
  const _e592 = WF.h("div", { className: "wf-spacer" });
  _e518.appendChild(_e592);
  _root.appendChild(_e518);
  return _root;
}

function Page_GettingStarted(params) {
  const _root = document.createDocumentFragment();
  const _e593 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e594 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e594);
  const _e595 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Getting Started");
  _e593.appendChild(_e595);
  const _e596 = WF.h("p", { className: "wf-text wf-text--muted" }, "Get up and running with WebFluent in under a minute.");
  _e593.appendChild(_e596);
  const _e597 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e597);
  const _e598 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Install");
  _e593.appendChild(_e598);
  const _e599 = WF.h("p", { className: "wf-text" }, "Build from source (requires Rust):");
  _e593.appendChild(_e599);
  const _e600 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e600);
  const _e601 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e602 = WF.h("div", { className: "wf-card__body" });
  const _e603 = WF.h("code", { className: "wf-code wf-code--block" }, "git clone https://github.com/user/webfluent.git\ncd webfluent\ncargo build --release");
  _e602.appendChild(_e603);
  _e601.appendChild(_e602);
  _e593.appendChild(_e601);
  const _e604 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e604);
  const _e605 = WF.h("p", { className: "wf-text wf-text--muted" }, "The binary is at target/release/wf. Add it to your PATH.");
  _e593.appendChild(_e605);
  const _e606 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e606);
  const _e607 = WF.h("hr", { className: "wf-divider" });
  _e593.appendChild(_e607);
  const _e608 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e608);
  const _e609 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Create a Project");
  _e593.appendChild(_e609);
  const _e610 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e610);
  const _e611 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e612 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e613 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e614 = WF.h("div", { className: "wf-card__body" });
  const _e615 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "SPA");
  _e614.appendChild(_e615);
  const _e616 = WF.h("div", { className: "wf-spacer" });
  _e614.appendChild(_e616);
  const _e617 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Interactive App");
  _e614.appendChild(_e617);
  const _e618 = WF.h("p", { className: "wf-text wf-text--muted" }, "Dashboard with routing, stores, forms, modals, animations.");
  _e614.appendChild(_e618);
  const _e619 = WF.h("div", { className: "wf-spacer" });
  _e614.appendChild(_e619);
  const _e620 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-app --template spa");
  _e614.appendChild(_e620);
  _e613.appendChild(_e614);
  _e612.appendChild(_e613);
  _e611.appendChild(_e612);
  const _e621 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e622 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e623 = WF.h("div", { className: "wf-card__body" });
  const _e624 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Static");
  _e623.appendChild(_e624);
  const _e625 = WF.h("div", { className: "wf-spacer" });
  _e623.appendChild(_e625);
  const _e626 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Static Site");
  _e623.appendChild(_e626);
  const _e627 = WF.h("p", { className: "wf-text wf-text--muted" }, "Marketing site with SSG, i18n, blog, contact form.");
  _e623.appendChild(_e627);
  const _e628 = WF.h("div", { className: "wf-spacer" });
  _e623.appendChild(_e628);
  const _e629 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-site --template static");
  _e623.appendChild(_e629);
  _e622.appendChild(_e623);
  _e621.appendChild(_e622);
  _e611.appendChild(_e621);
  _e593.appendChild(_e611);
  const _e630 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e630);
  const _e631 = WF.h("hr", { className: "wf-divider" });
  _e593.appendChild(_e631);
  const _e632 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e632);
  const _e633 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Build and Serve");
  _e593.appendChild(_e633);
  const _e634 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e635 = WF.h("div", { className: "wf-card__body" });
  const _e636 = WF.h("code", { className: "wf-code wf-code--block" }, "cd my-app\nwf build\nwf serve");
  _e635.appendChild(_e636);
  _e634.appendChild(_e635);
  _e593.appendChild(_e634);
  const _e637 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e637);
  const _e638 = WF.h("p", { className: "wf-text wf-text--muted" }, "Open http://localhost:3000 in your browser.");
  _e593.appendChild(_e638);
  const _e639 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e639);
  const _e640 = WF.h("hr", { className: "wf-divider" });
  _e593.appendChild(_e640);
  const _e641 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e641);
  const _e642 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Project Structure");
  _e593.appendChild(_e642);
  const _e643 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e644 = WF.h("div", { className: "wf-card__body" });
  const _e645 = WF.h("code", { className: "wf-code wf-code--block" }, "my-app/\n+-- webfluent.app.json       # Config\n+-- src/\n|   +-- App.wf               # Root (router, layout)\n|   +-- pages/\n|   +-- components/\n|   +-- stores/\n|   +-- translations/\n+-- public/\n+-- build/");
  _e644.appendChild(_e645);
  _e643.appendChild(_e644);
  _e593.appendChild(_e643);
  const _e646 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e646);
  const _e647 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e648 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/guide"); } }, "Read the Guide");
  _e647.appendChild(_e648);
  const _e649 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/components"); } }, "Browse Components");
  _e647.appendChild(_e649);
  _e593.appendChild(_e647);
  const _e650 = WF.h("div", { className: "wf-spacer" });
  _e593.appendChild(_e650);
  _root.appendChild(_e593);
  return _root;
}

function Page_Cli(params) {
  const _root = document.createDocumentFragment();
  const _e651 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e652 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e652);
  const _e653 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "CLI Reference");
  _e651.appendChild(_e653);
  const _e654 = WF.h("p", { className: "wf-text wf-text--muted" }, "The WebFluent command-line interface.");
  _e651.appendChild(_e654);
  const _e655 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e655);
  const _e656 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf init");
  _e651.appendChild(_e656);
  const _e657 = WF.h("p", { className: "wf-text" }, "Create a new WebFluent project.");
  _e651.appendChild(_e657);
  const _e658 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e658);
  const _e659 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e660 = WF.h("div", { className: "wf-card__body" });
  const _e661 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init <name> [--template spa|static]");
  _e660.appendChild(_e661);
  _e659.appendChild(_e660);
  _e651.appendChild(_e659);
  const _e662 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e662);
  const _e663 = WF.h("table", { className: "wf-table" });
  const _e664 = WF.h("thead", {});
  const _e665 = WF.h("td", {}, "Argument");
  _e664.appendChild(_e665);
  const _e666 = WF.h("td", {}, "Description");
  _e664.appendChild(_e666);
  _e663.appendChild(_e664);
  const _e667 = WF.h("tr", {});
  const _e668 = WF.h("td", {}, "name");
  _e667.appendChild(_e668);
  const _e669 = WF.h("td", {}, "Project name (creates a directory)");
  _e667.appendChild(_e669);
  _e663.appendChild(_e667);
  const _e670 = WF.h("tr", {});
  const _e671 = WF.h("td", {}, "--template, -t");
  _e670.appendChild(_e671);
  const _e672 = WF.h("td", {}, "Template: spa (default) or static");
  _e670.appendChild(_e672);
  _e663.appendChild(_e670);
  _e651.appendChild(_e663);
  const _e673 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e673);
  const _e674 = WF.h("p", { className: "wf-text wf-text--muted" }, "SPA template: interactive dashboard with routing, stores, forms, animations. Static template: marketing site with SSG, i18n, responsive grids.");
  _e651.appendChild(_e674);
  const _e675 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e675);
  const _e676 = WF.h("hr", { className: "wf-divider" });
  _e651.appendChild(_e676);
  const _e677 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e677);
  const _e678 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf build");
  _e651.appendChild(_e678);
  const _e679 = WF.h("p", { className: "wf-text" }, "Compile .wf files to HTML, CSS, and JavaScript.");
  _e651.appendChild(_e679);
  const _e680 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e680);
  const _e681 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e682 = WF.h("div", { className: "wf-card__body" });
  const _e683 = WF.h("code", { className: "wf-code wf-code--block" }, "wf build [--dir DIR]");
  _e682.appendChild(_e683);
  _e681.appendChild(_e682);
  _e651.appendChild(_e681);
  const _e684 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e684);
  const _e685 = WF.h("table", { className: "wf-table" });
  const _e686 = WF.h("thead", {});
  const _e687 = WF.h("td", {}, "Option");
  _e686.appendChild(_e687);
  const _e688 = WF.h("td", {}, "Description");
  _e686.appendChild(_e688);
  _e685.appendChild(_e686);
  const _e689 = WF.h("tr", {});
  const _e690 = WF.h("td", {}, "--dir, -d");
  _e689.appendChild(_e690);
  const _e691 = WF.h("td", {}, "Project directory (default: current directory)");
  _e689.appendChild(_e691);
  _e685.appendChild(_e689);
  _e651.appendChild(_e685);
  const _e692 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e692);
  const _e693 = WF.h("p", { className: "wf-text wf-text--muted" }, "The build pipeline: Lex all .wf files, parse to AST, run accessibility linter, generate HTML + CSS + JS, write to output directory.");
  _e651.appendChild(_e693);
  const _e694 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e694);
  const _e695 = WF.h("p", { className: "wf-text" }, "Output depends on config:");
  _e651.appendChild(_e695);
  const _e696 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e697 = WF.h("p", { className: "wf-text" }, "SPA mode (ssg: false): single index.html + app.js + styles.css");
  _e696.appendChild(_e697);
  const _e698 = WF.h("p", { className: "wf-text" }, "SSG mode (ssg: true): one HTML per page + app.js + styles.css");
  _e696.appendChild(_e698);
  _e651.appendChild(_e696);
  const _e699 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e699);
  const _e700 = WF.h("hr", { className: "wf-divider" });
  _e651.appendChild(_e700);
  const _e701 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e701);
  const _e702 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf serve");
  _e651.appendChild(_e702);
  const _e703 = WF.h("p", { className: "wf-text" }, "Start a development server that serves the built output.");
  _e651.appendChild(_e703);
  const _e704 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e704);
  const _e705 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e706 = WF.h("div", { className: "wf-card__body" });
  const _e707 = WF.h("code", { className: "wf-code wf-code--block" }, "wf serve [--dir DIR]");
  _e706.appendChild(_e707);
  _e705.appendChild(_e706);
  _e651.appendChild(_e705);
  const _e708 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e708);
  const _e709 = WF.h("p", { className: "wf-text wf-text--muted" }, "Serves files from the build directory. SPA fallback: all routes serve index.html so client-side routing works. Port is configured in webfluent.app.json (default: 3000).");
  _e651.appendChild(_e709);
  const _e710 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e710);
  const _e711 = WF.h("hr", { className: "wf-divider" });
  _e651.appendChild(_e711);
  const _e712 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e712);
  const _e713 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf generate");
  _e651.appendChild(_e713);
  const _e714 = WF.h("p", { className: "wf-text" }, "Scaffold a new page, component, or store.");
  _e651.appendChild(_e714);
  const _e715 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e715);
  const _e716 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e717 = WF.h("div", { className: "wf-card__body" });
  const _e718 = WF.h("code", { className: "wf-code wf-code--block" }, "wf generate <kind> <name> [--dir DIR]");
  _e717.appendChild(_e718);
  _e716.appendChild(_e717);
  _e651.appendChild(_e716);
  const _e719 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e719);
  const _e720 = WF.h("table", { className: "wf-table" });
  const _e721 = WF.h("thead", {});
  const _e722 = WF.h("td", {}, "Kind");
  _e721.appendChild(_e722);
  const _e723 = WF.h("td", {}, "Creates");
  _e721.appendChild(_e723);
  const _e724 = WF.h("td", {}, "Example");
  _e721.appendChild(_e724);
  _e720.appendChild(_e721);
  const _e725 = WF.h("tr", {});
  const _e726 = WF.h("td", {}, "page");
  _e725.appendChild(_e726);
  const _e727 = WF.h("td", {}, "src/pages/Name.wf");
  _e725.appendChild(_e727);
  const _e728 = WF.h("td", {}, "wf generate page About");
  _e725.appendChild(_e728);
  _e720.appendChild(_e725);
  const _e729 = WF.h("tr", {});
  const _e730 = WF.h("td", {}, "component");
  _e729.appendChild(_e730);
  const _e731 = WF.h("td", {}, "src/components/Name.wf");
  _e729.appendChild(_e731);
  const _e732 = WF.h("td", {}, "wf generate component Header");
  _e729.appendChild(_e732);
  _e720.appendChild(_e729);
  const _e733 = WF.h("tr", {});
  const _e734 = WF.h("td", {}, "store");
  _e733.appendChild(_e734);
  const _e735 = WF.h("td", {}, "src/stores/name.wf");
  _e733.appendChild(_e735);
  const _e736 = WF.h("td", {}, "wf generate store CartStore");
  _e733.appendChild(_e736);
  _e720.appendChild(_e733);
  _e651.appendChild(_e720);
  const _e737 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e737);
  const _e738 = WF.h("hr", { className: "wf-divider" });
  _e651.appendChild(_e738);
  const _e739 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e739);
  const _e740 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Configuration");
  _e651.appendChild(_e740);
  const _e741 = WF.h("p", { className: "wf-text" }, "All config is in webfluent.app.json at the project root.");
  _e651.appendChild(_e741);
  const _e742 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e742);
  const _e743 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e744 = WF.h("div", { className: "wf-card__body" });
  const _e745 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"name\": \"My App\",\n  \"version\": \"1.0.0\",\n  \"author\": \"Your Name\",\n  \"theme\": {\n    \"name\": \"default\",\n    \"mode\": \"light\",\n    \"tokens\": {\n      \"color-primary\": \"#6366F1\"\n    }\n  },\n  \"build\": {\n    \"output\": \"./build\",\n    \"minify\": true,\n    \"ssg\": false\n  },\n  \"dev\": {\n    \"port\": 3000\n  },\n  \"meta\": {\n    \"title\": \"My App\",\n    \"description\": \"Built with WebFluent\",\n    \"lang\": \"en\"\n  },\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e744.appendChild(_e745);
  _e743.appendChild(_e744);
  _e651.appendChild(_e743);
  const _e746 = WF.h("div", { className: "wf-spacer" });
  _e651.appendChild(_e746);
  _root.appendChild(_e651);
  return _root;
}

function Page_Home(params) {
  const _counter = WF.signal(0);
  const _taskInput = WF.signal("");
  const _tasks = WF.signal(["Learn WebFluent", "Build something cool"]);
  const _showDemo = WF.signal(false);
  const _root = document.createDocumentFragment();
  const _e747 = WF.h("div", { className: "wf-container" });
  const _e748 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e748);
  const _e749 = WF.h("h2", { className: "wf-heading wf-heading--h1 wf-text--center wf-animate-slideUp" }, "The Web-First Language");
  _e747.appendChild(_e749);
  const _e750 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e750);
  const _e751 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, "A programming language that compiles to HTML, CSS, and JavaScript.");
  _e747.appendChild(_e751);
  const _e752 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, "Built-in components, reactivity, routing, i18n, animations, and SSG.");
  _e747.appendChild(_e752);
  const _e753 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e753);
  const _e754 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e755 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, "Get Started");
  _e754.appendChild(_e755);
  const _e756 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { WF.navigate("/guide"); } }, "View Guide");
  _e754.appendChild(_e756);
  _e747.appendChild(_e754);
  const _e757 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e757);
  const _e758 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e759 = WF.h("div", { className: "wf-card__body" });
  const _e760 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\") {\n    Container {\n        Heading(\"Hello, WebFluent!\", h1)\n        Text(\"Build for the web. Nothing else.\")\n\n        Button(\"Get Started\", primary, large) {\n            navigate(\"/docs\")\n        }\n    }\n}");
  _e759.appendChild(_e760);
  _e758.appendChild(_e759);
  _e747.appendChild(_e758);
  const _e761 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e761);
  const _e762 = WF.h("hr", { className: "wf-divider" });
  _e747.appendChild(_e762);
  const _e763 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e763);
  const _e764 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, "Try It Live");
  _e747.appendChild(_e764);
  const _e765 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, "These are real WebFluent components running in your browser.");
  _e747.appendChild(_e765);
  const _e766 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e766);
  const _e767 = WF.h("div", { className: "wf-grid wf-grid--gap-lg", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e768 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e769 = WF.h("div", { className: "wf-card__header" });
  const _e770 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Reactive Counter");
  _e769.appendChild(_e770);
  _e768.appendChild(_e769);
  const _e771 = WF.h("div", { className: "wf-card__body" });
  const _e772 = WF.h("div", { className: "wf-row wf-row--center wf-row--gap-md" });
  const _e773 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { _counter.set((_counter() - 1)); } }, "-");
  _e772.appendChild(_e773);
  const _e774 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-heading--primary" }, `${_counter()}`);
  _e772.appendChild(_e774);
  const _e775 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { _counter.set((_counter() + 1)); } }, "+");
  _e772.appendChild(_e775);
  _e771.appendChild(_e772);
  const _e776 = WF.h("div", { className: "wf-spacer" });
  _e771.appendChild(_e776);
  const _e777 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Click the buttons. The number updates instantly.");
  _e771.appendChild(_e777);
  _e768.appendChild(_e771);
  _e767.appendChild(_e768);
  const _e778 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e779 = WF.h("div", { className: "wf-card__header" });
  const _e780 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Two-Way Binding");
  _e779.appendChild(_e780);
  _e778.appendChild(_e779);
  const _e781 = WF.h("div", { className: "wf-card__body" });
  const _e782 = WF.h("input", { className: "wf-input", value: () => _taskInput(), "on:input": (e) => _taskInput.set(e.target.value), placeholder: "Type something here...", label: "Live input", type: "text" });
  _e781.appendChild(_e782);
  const _e783 = WF.h("div", { className: "wf-spacer" });
  _e781.appendChild(_e783);
  WF.condRender(_e781,
    () => (_taskInput() !== ""),
    () => {
      const _e784 = document.createDocumentFragment();
      const _e785 = WF.h("div", { className: "wf-alert wf-alert--info" }, `You typed: ${_taskInput()}`);
      _e784.appendChild(_e785);
      return _e784;
    },
    null,
    null
  );
  const _e786 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "The alert updates as you type.");
  _e781.appendChild(_e786);
  _e778.appendChild(_e781);
  _e767.appendChild(_e778);
  const _e787 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e788 = WF.h("div", { className: "wf-card__header" });
  const _e789 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Conditional Rendering");
  _e788.appendChild(_e789);
  _e787.appendChild(_e788);
  const _e790 = WF.h("div", { className: "wf-card__body" });
  const _e791 = WF.h("label", { className: "wf-switch" });
  const _e792 = WF.h("input", { type: "checkbox", checked: () => _showDemo(), "on:change": () => _showDemo.set(!_showDemo()) });
  _e791.appendChild(_e792);
  const _e793 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e791.appendChild(_e793);
  _e791.appendChild(WF.text("Toggle content"));
  _e790.appendChild(_e791);
  const _e794 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e794);
  WF.condRender(_e790,
    () => _showDemo(),
    () => {
      const _e795 = document.createDocumentFragment();
      const _e796 = WF.h("div", { className: "wf-card wf-card--outlined" });
      const _e797 = WF.h("div", { className: "wf-card__body" });
      const _e798 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Visible!");
      _e797.appendChild(_e798);
      const _e799 = WF.h("div", { className: "wf-spacer" });
      _e797.appendChild(_e799);
      const _e800 = WF.h("p", { className: "wf-text" }, "This card animates in/out when you toggle the switch.");
      _e797.appendChild(_e800);
      _e796.appendChild(_e797);
      _e795.appendChild(_e796);
      return _e795;
    },
    null,
    { enter: "slideUp", exit: "fadeOut" }
  );
  _e787.appendChild(_e790);
  _e767.appendChild(_e787);
  const _e801 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e802 = WF.h("div", { className: "wf-card__header" });
  const _e803 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Components");
  _e802.appendChild(_e803);
  _e801.appendChild(_e802);
  const _e804 = WF.h("div", { className: "wf-card__body" });
  const _e805 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e806 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e807 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e806.appendChild(_e807);
  const _e808 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e806.appendChild(_e808);
  const _e809 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e806.appendChild(_e809);
  _e805.appendChild(_e806);
  const _e810 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e811 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "New");
  _e810.appendChild(_e811);
  const _e812 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Sale");
  _e810.appendChild(_e812);
  const _e813 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Active");
  _e810.appendChild(_e813);
  const _e814 = WF.h("span", { className: "wf-tag" }, "WebFluent");
  _e810.appendChild(_e814);
  _e805.appendChild(_e810);
  const _e815 = WF.h("progress", { className: "wf-progress", value: 72, max: 100 });
  _e805.appendChild(_e815);
  const _e816 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Button variants, badges, tags, and progress bar.");
  _e805.appendChild(_e816);
  _e804.appendChild(_e805);
  _e801.appendChild(_e804);
  _e767.appendChild(_e801);
  _e747.appendChild(_e767);
  const _e817 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e817);
  const _e818 = WF.h("hr", { className: "wf-divider" });
  _e747.appendChild(_e818);
  const _e819 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e819);
  const _e820 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, "Why WebFluent?");
  _e747.appendChild(_e820);
  const _e821 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, "Everything you need, built into the language.");
  _e747.appendChild(_e821);
  const _e822 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e822);
  const _e823 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e824 = Component_FeatureCard({ title: "Declarative Syntax", description: "No XML, no JSX. Write UI as readable declarations with curly braces and parentheses." });
  _e823.appendChild(_e824);
  const _e825 = Component_FeatureCard({ title: "50+ Components", description: "Navbar, Card, Modal, Form, Table, Tabs, and more. Every component has a default design." });
  _e823.appendChild(_e825);
  const _e826 = Component_FeatureCard({ title: "Signal Reactivity", description: "Fine-grained DOM updates without a virtual DOM. Only affected nodes update." });
  _e823.appendChild(_e826);
  const _e827 = Component_FeatureCard({ title: "Design System", description: "Design tokens for colors, spacing, typography. 4 themes. Switch with one config line." });
  _e823.appendChild(_e827);
  const _e828 = Component_FeatureCard({ title: "Animations", description: "12 built-in animations as modifiers. Enter/exit on conditionals and loops with stagger." });
  _e823.appendChild(_e828);
  const _e829 = Component_FeatureCard({ title: "i18n + RTL", description: "JSON translations, t() function, reactive locale switching, automatic RTL direction." });
  _e823.appendChild(_e829);
  const _e830 = Component_FeatureCard({ title: "SSG", description: "Pre-render pages at build time. Instant content, then JS hydrates for interactivity." });
  _e823.appendChild(_e830);
  const _e831 = Component_FeatureCard({ title: "A11y Linting", description: "12 compile-time checks for missing alt text, labels, headings. Never blocks the build." });
  _e823.appendChild(_e831);
  const _e832 = Component_FeatureCard({ title: "Zero Dependencies", description: "Compiles to vanilla HTML, CSS, JS. No runtime framework. Pure web standards output." });
  _e823.appendChild(_e832);
  _e747.appendChild(_e823);
  const _e833 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e833);
  const _e834 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e835 = WF.h("div", { className: "wf-card__body" });
  const _e836 = WF.h("div", { className: "wf-row wf-row--center wf-row--between" });
  const _e837 = WF.h("div", { className: "wf-stack" });
  const _e838 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Ready to build?");
  _e837.appendChild(_e838);
  const _e839 = WF.h("p", { className: "wf-text wf-text--muted" }, "Create your first project in seconds.");
  _e837.appendChild(_e839);
  _e836.appendChild(_e837);
  const _e840 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, "Get Started");
  _e836.appendChild(_e840);
  _e835.appendChild(_e836);
  _e834.appendChild(_e835);
  _e747.appendChild(_e834);
  const _e841 = WF.h("div", { className: "wf-spacer" });
  _e747.appendChild(_e841);
  _root.appendChild(_e747);
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
  const _root = document.createDocumentFragment();
  const _e842 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e843 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e843);
  const _e844 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Components Reference");
  _e842.appendChild(_e844);
  const _e845 = WF.h("p", { className: "wf-text wf-text--muted" }, "50+ built-in components. Below are live interactive examples you can play with.");
  _e842.appendChild(_e845);
  const _e846 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e846);
  const _e847 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Buttons");
  _e842.appendChild(_e847);
  const _e848 = WF.h("p", { className: "wf-text" }, "Buttons support size, color, and shape modifiers.");
  _e842.appendChild(_e848);
  const _e849 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e849);
  const _e850 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e851 = WF.h("div", { className: "wf-card__body" });
  const _e852 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e853 = WF.h("button", { className: "wf-btn" }, "Default");
  _e852.appendChild(_e853);
  const _e854 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e852.appendChild(_e854);
  const _e855 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e852.appendChild(_e855);
  const _e856 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e852.appendChild(_e856);
  const _e857 = WF.h("button", { className: "wf-btn wf-btn--warning" }, "Warning");
  _e852.appendChild(_e857);
  const _e858 = WF.h("button", { className: "wf-btn wf-btn--info" }, "Info");
  _e852.appendChild(_e858);
  _e851.appendChild(_e852);
  const _e859 = WF.h("div", { className: "wf-spacer" });
  _e851.appendChild(_e859);
  const _e860 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e861 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Small");
  _e860.appendChild(_e861);
  const _e862 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Medium");
  _e860.appendChild(_e862);
  const _e863 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large" }, "Large");
  _e860.appendChild(_e863);
  const _e864 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--rounded" }, "Rounded");
  _e860.appendChild(_e864);
  const _e865 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--full" }, "Full Width");
  _e860.appendChild(_e865);
  _e851.appendChild(_e860);
  _e850.appendChild(_e851);
  _e842.appendChild(_e850);
  const _e866 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e866);
  const _e867 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e868 = WF.h("div", { className: "wf-card__body" });
  const _e869 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Primary\", primary)\nButton(\"Large\", primary, large)\nButton(\"Rounded\", success, rounded)\nButton(\"Full Width\", danger, full)");
  _e868.appendChild(_e869);
  _e867.appendChild(_e868);
  _e842.appendChild(_e867);
  const _e870 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e870);
  const _e871 = WF.h("hr", { className: "wf-divider" });
  _e842.appendChild(_e871);
  const _e872 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e872);
  const _e873 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Cards");
  _e842.appendChild(_e873);
  const _e874 = WF.h("p", { className: "wf-text" }, "Cards are surfaces for grouping content. They support Header, Body, and Footer sub-components.");
  _e842.appendChild(_e874);
  const _e875 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e875);
  const _e876 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e877 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e878 = WF.h("div", { className: "wf-card" });
  const _e879 = WF.h("div", { className: "wf-card__header" });
  const _e880 = WF.h("p", { className: "wf-text wf-text--bold" }, "Default Card");
  _e879.appendChild(_e880);
  _e878.appendChild(_e879);
  const _e881 = WF.h("div", { className: "wf-card__body" });
  const _e882 = WF.h("p", { className: "wf-text wf-text--muted" }, "Basic card with header and body.");
  _e881.appendChild(_e882);
  _e878.appendChild(_e881);
  const _e883 = WF.h("div", { className: "wf-card__footer" });
  const _e884 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Action");
  console.log("clicked");
  _e883.appendChild(_e884);
  _e878.appendChild(_e883);
  _e877.appendChild(_e878);
  _e876.appendChild(_e877);
  const _e885 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e886 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e887 = WF.h("div", { className: "wf-card__header" });
  const _e888 = WF.h("p", { className: "wf-text wf-text--bold" }, "Elevated");
  _e887.appendChild(_e888);
  _e886.appendChild(_e887);
  const _e889 = WF.h("div", { className: "wf-card__body" });
  const _e890 = WF.h("p", { className: "wf-text wf-text--muted" }, "Card with shadow elevation.");
  _e889.appendChild(_e890);
  _e886.appendChild(_e889);
  _e885.appendChild(_e886);
  _e876.appendChild(_e885);
  const _e891 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e892 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e893 = WF.h("div", { className: "wf-card__header" });
  const _e894 = WF.h("p", { className: "wf-text wf-text--bold" }, "Outlined");
  _e893.appendChild(_e894);
  _e892.appendChild(_e893);
  const _e895 = WF.h("div", { className: "wf-card__body" });
  const _e896 = WF.h("p", { className: "wf-text wf-text--muted" }, "Card with border only.");
  _e895.appendChild(_e896);
  _e892.appendChild(_e895);
  _e891.appendChild(_e892);
  _e876.appendChild(_e891);
  _e842.appendChild(_e876);
  const _e897 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e897);
  const _e898 = WF.h("hr", { className: "wf-divider" });
  _e842.appendChild(_e898);
  const _e899 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e899);
  const _e900 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Form Controls");
  _e842.appendChild(_e900);
  const _e901 = WF.h("p", { className: "wf-text" }, "All form inputs support two-way binding with the bind: attribute.");
  _e842.appendChild(_e901);
  const _e902 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e902);
  const _e903 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e904 = WF.h("div", { className: "wf-card__body" });
  const _e905 = WF.h("div", { className: "wf-stack wf-stack--gap-md" });
  const _e906 = WF.h("input", { className: "wf-input", value: () => _inputVal(), "on:input": (e) => _inputVal.set(e.target.value), label: "Text Input", placeholder: "Type here...", type: "text" });
  _e905.appendChild(_e906);
  WF.condRender(_e905,
    () => (_inputVal() !== ""),
    () => {
      const _e907 = document.createDocumentFragment();
      const _e908 = WF.h("p", { className: "wf-text wf-text--primary wf-text--bold" }, `You typed: ${_inputVal()}`);
      _e907.appendChild(_e908);
      return _e907;
    },
    null,
    null
  );
  const _e909 = WF.h("hr", { className: "wf-divider" });
  _e905.appendChild(_e909);
  const _e910 = WF.h("select", { className: "wf-select", value: () => _selectVal(), "on:input": (e) => _selectVal.set(e.target.value), label: "Select" });
  const _e911 = WF.h("option", {}, "opt1");
  _e910.appendChild(_e911);
  const _e912 = WF.h("option", {}, "opt2");
  _e910.appendChild(_e912);
  const _e913 = WF.h("option", {}, "opt3");
  _e910.appendChild(_e913);
  _e905.appendChild(_e910);
  const _e914 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, `Selected: ${_selectVal()}`);
  _e905.appendChild(_e914);
  const _e915 = WF.h("hr", { className: "wf-divider" });
  _e905.appendChild(_e915);
  const _e916 = WF.h("label", { className: "wf-checkbox" });
  const _e917 = WF.h("input", { type: "checkbox", checked: () => _checkVal(), "on:change": () => _checkVal.set(!_checkVal()) });
  _e916.appendChild(_e917);
  _e916.appendChild(WF.text("I agree to the terms"));
  _e905.appendChild(_e916);
  const _e918 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, `Checked: ${_checkVal()}`);
  _e905.appendChild(_e918);
  const _e919 = WF.h("hr", { className: "wf-divider" });
  _e905.appendChild(_e919);
  const _e920 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e921 = WF.h("label", { className: "wf-radio" });
  const _e922 = WF.h("input", { type: "radio", checked: () => _radioVal() === "a", "on:change": () => _radioVal.set("a") });
  _e921.appendChild(_e922);
  _e921.appendChild(WF.text("Option A"));
  _e920.appendChild(_e921);
  const _e923 = WF.h("label", { className: "wf-radio" });
  const _e924 = WF.h("input", { type: "radio", checked: () => _radioVal() === "b", "on:change": () => _radioVal.set("b") });
  _e923.appendChild(_e924);
  _e923.appendChild(WF.text("Option B"));
  _e920.appendChild(_e923);
  const _e925 = WF.h("label", { className: "wf-radio" });
  const _e926 = WF.h("input", { type: "radio", checked: () => _radioVal() === "c", "on:change": () => _radioVal.set("c") });
  _e925.appendChild(_e926);
  _e925.appendChild(WF.text("Option C"));
  _e920.appendChild(_e925);
  _e905.appendChild(_e920);
  const _e927 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, `Selected: ${_radioVal()}`);
  _e905.appendChild(_e927);
  const _e928 = WF.h("hr", { className: "wf-divider" });
  _e905.appendChild(_e928);
  const _e929 = WF.h("label", { className: "wf-switch" });
  const _e930 = WF.h("input", { type: "checkbox", checked: () => _switchVal(), "on:change": () => _switchVal.set(!_switchVal()) });
  _e929.appendChild(_e930);
  const _e931 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e929.appendChild(_e931);
  _e929.appendChild(WF.text("Dark Mode"));
  _e905.appendChild(_e929);
  const _e932 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, `Enabled: ${_switchVal()}`);
  _e905.appendChild(_e932);
  const _e933 = WF.h("hr", { className: "wf-divider" });
  _e905.appendChild(_e933);
  const _e934 = WF.h("input", { className: "wf-slider", value: () => _sliderVal(), "on:input": (e) => _sliderVal.set(e.target.value), min: 0, max: 100, step: 1, label: "Volume" });
  _e905.appendChild(_e934);
  const _e935 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, `Value: ${_sliderVal()}`);
  _e905.appendChild(_e935);
  _e904.appendChild(_e905);
  _e903.appendChild(_e904);
  _e842.appendChild(_e903);
  const _e936 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e936);
  const _e937 = WF.h("hr", { className: "wf-divider" });
  _e842.appendChild(_e937);
  const _e938 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e938);
  const _e939 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Feedback");
  _e842.appendChild(_e939);
  const _e940 = WF.h("p", { className: "wf-text" }, "Alerts, modals, progress bars, and loading indicators.");
  _e842.appendChild(_e940);
  const _e941 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e941);
  const _e942 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e943 = WF.h("div", { className: "wf-card__body" });
  const _e944 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e945 = WF.h("div", { className: "wf-alert wf-alert--success" }, "This is a success alert.");
  _e944.appendChild(_e945);
  const _e946 = WF.h("div", { className: "wf-alert wf-alert--warning" }, "This is a warning alert.");
  _e944.appendChild(_e946);
  const _e947 = WF.h("div", { className: "wf-alert wf-alert--danger" }, "This is a danger alert.");
  _e944.appendChild(_e947);
  const _e948 = WF.h("div", { className: "wf-alert wf-alert--info" }, "This is an info alert.");
  _e944.appendChild(_e948);
  _e943.appendChild(_e944);
  const _e949 = WF.h("div", { className: "wf-spacer" });
  _e943.appendChild(_e949);
  const _e950 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e951 = WF.h("div", { className: "wf-spinner" });
  _e950.appendChild(_e951);
  const _e952 = WF.h("div", { className: "wf-spinner wf-spinner--large wf-spinner--primary" });
  _e950.appendChild(_e952);
  const _e953 = WF.h("progress", { className: "wf-progress", value: _sliderVal(), max: 100 });
  _e950.appendChild(_e953);
  _e943.appendChild(_e950);
  const _e954 = WF.h("div", { className: "wf-spacer" });
  _e943.appendChild(_e954);
  const _e955 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { _activeModal.set(true); } }, "Open Modal");
  _e943.appendChild(_e955);
  _e942.appendChild(_e943);
  _e842.appendChild(_e942);
  const _e956 = WF.h("div", { className: "wf-modal" });
  const _e957 = WF.h("div", { className: "wf-modal__content" });
  const _e958 = WF.h("div", { className: "wf-modal__header" }, WF.h("h3", {}, "Example Modal"));
  _e957.appendChild(_e958);
  const _e959 = WF.h("div", { className: "wf-modal__body" });
  const _e960 = WF.h("p", { className: "wf-text" }, "This is a real modal dialog. It was triggered by clicking the button.");
  _e959.appendChild(_e960);
  const _e961 = WF.h("div", { className: "wf-spacer" });
  _e959.appendChild(_e961);
  const _e962 = WF.h("p", { className: "wf-text wf-text--muted" }, "The modal is controlled by a state variable.");
  _e959.appendChild(_e962);
  _e957.appendChild(_e959);
  const _e963 = WF.h("div", { className: "wf-modal__footer" });
  const _e964 = WF.h("button", { className: "wf-btn", "on:click": (e) => { _activeModal.set(false); } }, "Close");
  _e963.appendChild(_e964);
  const _e965 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { _activeModal.set(false); } }, "Confirm");
  _e963.appendChild(_e965);
  _e957.appendChild(_e963);
  _e956.appendChild(_e957);
  WF.effect(() => { _e956.className = _activeModal() ? 'wf-modal open' : 'wf-modal'; });
  _e842.appendChild(_e956);
  const _e966 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e966);
  const _e967 = WF.h("hr", { className: "wf-divider" });
  _e842.appendChild(_e967);
  const _e968 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e968);
  const _e969 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Display");
  _e842.appendChild(_e969);
  const _e970 = WF.h("p", { className: "wf-text" }, "Tables, badges, avatars, tags, and tooltips.");
  _e842.appendChild(_e970);
  const _e971 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e971);
  const _e972 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e973 = WF.h("div", { className: "wf-card__body" });
  const _e974 = WF.h("table", { className: "wf-table" });
  const _e975 = WF.h("thead", {});
  const _e976 = WF.h("td", {}, "Name");
  _e975.appendChild(_e976);
  const _e977 = WF.h("td", {}, "Role");
  _e975.appendChild(_e977);
  const _e978 = WF.h("td", {}, "Status");
  _e975.appendChild(_e978);
  _e974.appendChild(_e975);
  const _e979 = WF.h("tr", {});
  const _e980 = WF.h("td", {}, "Monzer Omer");
  _e979.appendChild(_e980);
  const _e981 = WF.h("td", {}, "Creator");
  _e979.appendChild(_e981);
  const _e982 = WF.h("td", {}, "Active");
  _e979.appendChild(_e982);
  _e974.appendChild(_e979);
  const _e983 = WF.h("tr", {});
  const _e984 = WF.h("td", {}, "Sara Ali");
  _e983.appendChild(_e984);
  const _e985 = WF.h("td", {}, "Designer");
  _e983.appendChild(_e985);
  const _e986 = WF.h("td", {}, "Active");
  _e983.appendChild(_e986);
  _e974.appendChild(_e983);
  const _e987 = WF.h("tr", {});
  const _e988 = WF.h("td", {}, "Omar Hassan");
  _e987.appendChild(_e988);
  const _e989 = WF.h("td", {}, "Developer");
  _e987.appendChild(_e989);
  const _e990 = WF.h("td", {}, "Away");
  _e987.appendChild(_e990);
  _e974.appendChild(_e987);
  _e973.appendChild(_e974);
  const _e991 = WF.h("div", { className: "wf-spacer" });
  _e973.appendChild(_e991);
  const _e992 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e993 = WF.h("div", { className: "wf-avatar wf-avatar--primary", initials: "MO" });
  _e992.appendChild(_e993);
  const _e994 = WF.h("div", { className: "wf-avatar wf-avatar--success", initials: "SA" });
  _e992.appendChild(_e994);
  const _e995 = WF.h("div", { className: "wf-avatar wf-avatar--info", initials: "OH" });
  _e992.appendChild(_e995);
  const _e996 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Admin");
  _e992.appendChild(_e996);
  const _e997 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Online");
  _e992.appendChild(_e997);
  const _e998 = WF.h("span", { className: "wf-tag" }, "WebFluent");
  _e992.appendChild(_e998);
  const _e999 = WF.h("span", { className: "wf-tag" }, "Rust");
  _e992.appendChild(_e999);
  const _e1000 = WF.h("span", { className: "wf-tag" }, "Open Source");
  _e992.appendChild(_e1000);
  _e973.appendChild(_e992);
  _e972.appendChild(_e973);
  _e842.appendChild(_e972);
  const _e1001 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1001);
  const _e1002 = WF.h("hr", { className: "wf-divider" });
  _e842.appendChild(_e1002);
  const _e1003 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1003);
  const _e1004 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Layout");
  _e842.appendChild(_e1004);
  const _e1005 = WF.h("p", { className: "wf-text" }, "Container, Row, Column, Grid, Stack, Spacer, Divider.");
  _e842.appendChild(_e1005);
  const _e1006 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1006);
  const _e1007 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1008 = WF.h("div", { className: "wf-card__body" });
  const _e1009 = WF.h("p", { className: "wf-text wf-text--bold" }, "Grid with 3 columns:");
  _e1008.appendChild(_e1009);
  const _e1010 = WF.h("div", { className: "wf-spacer" });
  _e1008.appendChild(_e1010);
  const _e1011 = WF.h("div", { className: "wf-grid wf-grid--gap-sm", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1012 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1013 = WF.h("div", { className: "wf-card__body" });
  const _e1014 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 1");
  _e1013.appendChild(_e1014);
  _e1012.appendChild(_e1013);
  _e1011.appendChild(_e1012);
  const _e1015 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1016 = WF.h("div", { className: "wf-card__body" });
  const _e1017 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 2");
  _e1016.appendChild(_e1017);
  _e1015.appendChild(_e1016);
  _e1011.appendChild(_e1015);
  const _e1018 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1019 = WF.h("div", { className: "wf-card__body" });
  const _e1020 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 3");
  _e1019.appendChild(_e1020);
  _e1018.appendChild(_e1019);
  _e1011.appendChild(_e1018);
  _e1008.appendChild(_e1011);
  const _e1021 = WF.h("div", { className: "wf-spacer" });
  _e1008.appendChild(_e1021);
  const _e1022 = WF.h("p", { className: "wf-text wf-text--bold" }, "Row with Columns (6/6 split):");
  _e1008.appendChild(_e1022);
  const _e1023 = WF.h("div", { className: "wf-spacer" });
  _e1008.appendChild(_e1023);
  const _e1024 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1025 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1026 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1027 = WF.h("div", { className: "wf-card__body" });
  const _e1028 = WF.h("p", { className: "wf-text wf-text--center" }, "Left Half");
  _e1027.appendChild(_e1028);
  _e1026.appendChild(_e1027);
  _e1025.appendChild(_e1026);
  _e1024.appendChild(_e1025);
  const _e1029 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1030 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1031 = WF.h("div", { className: "wf-card__body" });
  const _e1032 = WF.h("p", { className: "wf-text wf-text--center" }, "Right Half");
  _e1031.appendChild(_e1032);
  _e1030.appendChild(_e1031);
  _e1029.appendChild(_e1030);
  _e1024.appendChild(_e1029);
  _e1008.appendChild(_e1024);
  const _e1033 = WF.h("div", { className: "wf-spacer" });
  _e1008.appendChild(_e1033);
  const _e1034 = WF.h("p", { className: "wf-text wf-text--bold" }, "Stack (vertical):");
  _e1008.appendChild(_e1034);
  const _e1035 = WF.h("div", { className: "wf-spacer" });
  _e1008.appendChild(_e1035);
  const _e1036 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1037 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1038 = WF.h("div", { className: "wf-card__body" });
  const _e1039 = WF.h("p", { className: "wf-text" }, "Item 1");
  _e1038.appendChild(_e1039);
  _e1037.appendChild(_e1038);
  _e1036.appendChild(_e1037);
  const _e1040 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1041 = WF.h("div", { className: "wf-card__body" });
  const _e1042 = WF.h("p", { className: "wf-text" }, "Item 2");
  _e1041.appendChild(_e1042);
  _e1040.appendChild(_e1041);
  _e1036.appendChild(_e1040);
  const _e1043 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1044 = WF.h("div", { className: "wf-card__body" });
  const _e1045 = WF.h("p", { className: "wf-text" }, "Item 3");
  _e1044.appendChild(_e1045);
  _e1043.appendChild(_e1044);
  _e1036.appendChild(_e1043);
  _e1008.appendChild(_e1036);
  _e1007.appendChild(_e1008);
  _e842.appendChild(_e1007);
  const _e1046 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1046);
  const _e1047 = WF.h("hr", { className: "wf-divider" });
  _e842.appendChild(_e1047);
  const _e1048 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1048);
  const _e1049 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Navigation");
  _e842.appendChild(_e1049);
  const _e1050 = WF.h("p", { className: "wf-text" }, "Tabs let you switch between content panels.");
  _e842.appendChild(_e1050);
  const _e1051 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1051);
  const _e1052 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1053 = WF.h("div", { className: "wf-card__body" });
  const _e1054 = WF.h("div", { className: "wf-tabs" });
  const _e1055 = WF.h("div", { className: "wf-tabs__nav" });
  const _e1056 = WF.signal(0);
  const _e1057 = WF.h("button", { className: () => _e1056() === 0 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1056.set(0) }, "Profile");
  _e1055.appendChild(_e1057);
  const _e1058 = WF.h("button", { className: () => _e1056() === 1 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1056.set(1) }, "Settings");
  _e1055.appendChild(_e1058);
  const _e1059 = WF.h("button", { className: () => _e1056() === 2 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1056.set(2) }, "About");
  _e1055.appendChild(_e1059);
  _e1054.appendChild(_e1055);
  const _e1060 = WF.h("div", { className: "wf-tab-page" });
  const _e1061 = WF.h("div", { className: "wf-spacer" });
  _e1060.appendChild(_e1061);
  const _e1062 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1063 = WF.h("div", { className: "wf-avatar wf-avatar--primary wf-avatar--large", initials: "MO" });
  _e1062.appendChild(_e1063);
  const _e1064 = WF.h("div", { className: "wf-stack" });
  const _e1065 = WF.h("p", { className: "wf-text wf-text--bold" }, "Monzer Omer");
  _e1064.appendChild(_e1065);
  const _e1066 = WF.h("p", { className: "wf-text wf-text--muted" }, "Creator of WebFluent");
  _e1064.appendChild(_e1066);
  _e1062.appendChild(_e1064);
  _e1060.appendChild(_e1062);
  WF.effect(() => { _e1060.style.display = _e1056() === 0 ? 'block' : 'none'; });
  _e1054.appendChild(_e1060);
  const _e1067 = WF.h("div", { className: "wf-tab-page" });
  const _e1068 = WF.h("div", { className: "wf-spacer" });
  _e1067.appendChild(_e1068);
  const _e1069 = WF.h("label", { className: "wf-switch" });
  const _e1070 = WF.h("input", { type: "checkbox", checked: () => _switchVal(), "on:change": () => _switchVal.set(!_switchVal()) });
  _e1069.appendChild(_e1070);
  const _e1071 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1069.appendChild(_e1071);
  _e1069.appendChild(WF.text("Enable notifications"));
  _e1067.appendChild(_e1069);
  const _e1072 = WF.h("div", { className: "wf-spacer" });
  _e1067.appendChild(_e1072);
  const _e1073 = WF.h("input", { className: "wf-slider", value: () => _sliderVal(), "on:input": (e) => _sliderVal.set(e.target.value), min: 0, max: 100, label: "Volume" });
  _e1067.appendChild(_e1073);
  WF.effect(() => { _e1067.style.display = _e1056() === 1 ? 'block' : 'none'; });
  _e1054.appendChild(_e1067);
  const _e1074 = WF.h("div", { className: "wf-tab-page" });
  const _e1075 = WF.h("div", { className: "wf-spacer" });
  _e1074.appendChild(_e1075);
  const _e1076 = WF.h("p", { className: "wf-text" }, "WebFluent is a web-first programming language.");
  _e1074.appendChild(_e1076);
  const _e1077 = WF.h("p", { className: "wf-text wf-text--muted" }, "It compiles to HTML, CSS, and JavaScript.");
  _e1074.appendChild(_e1077);
  WF.effect(() => { _e1074.style.display = _e1056() === 2 ? 'block' : 'none'; });
  _e1054.appendChild(_e1074);
  _e1053.appendChild(_e1054);
  _e1052.appendChild(_e1053);
  _e842.appendChild(_e1052);
  const _e1078 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1078);
  const _e1079 = WF.h("hr", { className: "wf-divider" });
  _e842.appendChild(_e1079);
  const _e1080 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1080);
  const _e1081 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Typography");
  _e842.appendChild(_e1081);
  const _e1082 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1082);
  const _e1083 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1084 = WF.h("div", { className: "wf-card__body" });
  const _e1085 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Heading h2");
  _e1084.appendChild(_e1085);
  const _e1086 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Heading h2");
  _e1084.appendChild(_e1086);
  const _e1087 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Heading h3");
  _e1084.appendChild(_e1087);
  const _e1088 = WF.h("div", { className: "wf-spacer" });
  _e1084.appendChild(_e1088);
  const _e1089 = WF.h("p", { className: "wf-text" }, "Normal text paragraph.");
  _e1084.appendChild(_e1089);
  const _e1090 = WF.h("p", { className: "wf-text wf-text--bold" }, "Bold text.");
  _e1084.appendChild(_e1090);
  const _e1091 = WF.h("p", { className: "wf-text wf-text--muted" }, "Muted text.");
  _e1084.appendChild(_e1091);
  const _e1092 = WF.h("p", { className: "wf-text wf-text--primary" }, "Primary colored.");
  _e1084.appendChild(_e1092);
  const _e1093 = WF.h("p", { className: "wf-text wf-text--danger" }, "Danger colored.");
  _e1084.appendChild(_e1093);
  const _e1094 = WF.h("p", { className: "wf-text wf-text--small" }, "Small text.");
  _e1084.appendChild(_e1094);
  const _e1095 = WF.h("p", { className: "wf-text wf-text--uppercase" }, "Uppercase.");
  _e1084.appendChild(_e1095);
  const _e1096 = WF.h("p", { className: "wf-text wf-text--center" }, "Centered text.");
  _e1084.appendChild(_e1096);
  const _e1097 = WF.h("div", { className: "wf-spacer" });
  _e1084.appendChild(_e1097);
  const _e1098 = WF.h("blockquote", { className: "wf-blockquote" }, "The best way to predict the future is to create it.");
  _e1084.appendChild(_e1098);
  const _e1099 = WF.h("div", { className: "wf-spacer" });
  _e1084.appendChild(_e1099);
  const _e1100 = WF.h("code", { className: "wf-code" }, "const greeting = \"Hello, WebFluent!\";");
  _e1084.appendChild(_e1100);
  _e1083.appendChild(_e1084);
  _e842.appendChild(_e1083);
  const _e1101 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1101);
  const _e1102 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1103 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e1102.appendChild(_e1103);
  const _e1104 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/animation"); } }, "Animation System");
  _e1102.appendChild(_e1104);
  _e842.appendChild(_e1102);
  const _e1105 = WF.h("div", { className: "wf-spacer" });
  _e842.appendChild(_e1105);
  _root.appendChild(_e842);
  return _root;
}

function Page_Accessibility(params) {
  const _root = document.createDocumentFragment();
  const _e1106 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1107 = WF.h("div", { className: "wf-spacer" });
  _e1106.appendChild(_e1107);
  const _e1108 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Accessibility Linting");
  _e1106.appendChild(_e1108);
  const _e1109 = WF.h("p", { className: "wf-text wf-text--muted" }, "WebFluent checks your code for accessibility issues at compile time. Warnings are printed during build but never block compilation.");
  _e1106.appendChild(_e1109);
  const _e1110 = WF.h("div", { className: "wf-spacer" });
  _e1106.appendChild(_e1110);
  const _e1111 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "How It Works");
  _e1106.appendChild(_e1111);
  const _e1112 = WF.h("p", { className: "wf-text" }, "The linter runs automatically after parsing, before code generation. It walks the AST and checks each component against 12 rules.");
  _e1106.appendChild(_e1112);
  const _e1113 = WF.h("div", { className: "wf-spacer" });
  _e1106.appendChild(_e1113);
  const _e1114 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1115 = WF.h("div", { className: "wf-card__body" });
  const _e1116 = WF.h("code", { className: "wf-code wf-code--block" }, "$ wf build\nBuilding my-app...\n  Warning [A01]: Image missing \"alt\" attribute at src/pages/Home.wf:12:5\n    Add alt text: Image(src: \"...\", alt: \"Description of image\")\n  Warning [A03]: Input missing \"label\" attribute at src/pages/Form.wf:8:9\n    Add a label: Input(text, label: \"Username\")\n  3 pages, 2 components, 1 stores\n  Build complete with 2 accessibility warning(s).");
  _e1115.appendChild(_e1116);
  _e1114.appendChild(_e1115);
  _e1106.appendChild(_e1114);
  const _e1117 = WF.h("div", { className: "wf-spacer" });
  _e1106.appendChild(_e1117);
  const _e1118 = WF.h("hr", { className: "wf-divider" });
  _e1106.appendChild(_e1118);
  const _e1119 = WF.h("div", { className: "wf-spacer" });
  _e1106.appendChild(_e1119);
  const _e1120 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Lint Rules");
  _e1106.appendChild(_e1120);
  const _e1121 = WF.h("table", { className: "wf-table" });
  const _e1122 = WF.h("thead", {});
  const _e1123 = WF.h("td", {}, "Rule");
  _e1122.appendChild(_e1123);
  const _e1124 = WF.h("td", {}, "Component");
  _e1122.appendChild(_e1124);
  const _e1125 = WF.h("td", {}, "Check");
  _e1122.appendChild(_e1125);
  _e1121.appendChild(_e1122);
  const _e1126 = WF.h("tr", {});
  const _e1127 = WF.h("td", {}, "A01");
  _e1126.appendChild(_e1127);
  const _e1128 = WF.h("td", {}, "Image");
  _e1126.appendChild(_e1128);
  const _e1129 = WF.h("td", {}, "Must have alt attribute");
  _e1126.appendChild(_e1129);
  _e1121.appendChild(_e1126);
  const _e1130 = WF.h("tr", {});
  const _e1131 = WF.h("td", {}, "A02");
  _e1130.appendChild(_e1131);
  const _e1132 = WF.h("td", {}, "IconButton");
  _e1130.appendChild(_e1132);
  const _e1133 = WF.h("td", {}, "Must have label attribute (no visible text)");
  _e1130.appendChild(_e1133);
  _e1121.appendChild(_e1130);
  const _e1134 = WF.h("tr", {});
  const _e1135 = WF.h("td", {}, "A03");
  _e1134.appendChild(_e1135);
  const _e1136 = WF.h("td", {}, "Input");
  _e1134.appendChild(_e1136);
  const _e1137 = WF.h("td", {}, "Must have label or placeholder");
  _e1134.appendChild(_e1137);
  _e1121.appendChild(_e1134);
  const _e1138 = WF.h("tr", {});
  const _e1139 = WF.h("td", {}, "A04");
  _e1138.appendChild(_e1139);
  const _e1140 = WF.h("td", {}, "Checkbox, Radio, Switch, Slider");
  _e1138.appendChild(_e1140);
  const _e1141 = WF.h("td", {}, "Must have label attribute");
  _e1138.appendChild(_e1141);
  _e1121.appendChild(_e1138);
  const _e1142 = WF.h("tr", {});
  const _e1143 = WF.h("td", {}, "A05");
  _e1142.appendChild(_e1143);
  const _e1144 = WF.h("td", {}, "Button");
  _e1142.appendChild(_e1144);
  const _e1145 = WF.h("td", {}, "Must have text content");
  _e1142.appendChild(_e1145);
  _e1121.appendChild(_e1142);
  const _e1146 = WF.h("tr", {});
  const _e1147 = WF.h("td", {}, "A06");
  _e1146.appendChild(_e1147);
  const _e1148 = WF.h("td", {}, "Link");
  _e1146.appendChild(_e1148);
  const _e1149 = WF.h("td", {}, "Must have text content or children");
  _e1146.appendChild(_e1149);
  _e1121.appendChild(_e1146);
  const _e1150 = WF.h("tr", {});
  const _e1151 = WF.h("td", {}, "A07");
  _e1150.appendChild(_e1151);
  const _e1152 = WF.h("td", {}, "Heading");
  _e1150.appendChild(_e1152);
  const _e1153 = WF.h("td", {}, "Must not be empty");
  _e1150.appendChild(_e1153);
  _e1121.appendChild(_e1150);
  const _e1154 = WF.h("tr", {});
  const _e1155 = WF.h("td", {}, "A08");
  _e1154.appendChild(_e1155);
  const _e1156 = WF.h("td", {}, "Modal, Dialog");
  _e1154.appendChild(_e1156);
  const _e1157 = WF.h("td", {}, "Must have title attribute");
  _e1154.appendChild(_e1157);
  _e1121.appendChild(_e1154);
  const _e1158 = WF.h("tr", {});
  const _e1159 = WF.h("td", {}, "A09");
  _e1158.appendChild(_e1159);
  const _e1160 = WF.h("td", {}, "Video");
  _e1158.appendChild(_e1160);
  const _e1161 = WF.h("td", {}, "Must have controls attribute");
  _e1158.appendChild(_e1161);
  _e1121.appendChild(_e1158);
  const _e1162 = WF.h("tr", {});
  const _e1163 = WF.h("td", {}, "A10");
  _e1162.appendChild(_e1163);
  const _e1164 = WF.h("td", {}, "Table");
  _e1162.appendChild(_e1164);
  const _e1165 = WF.h("td", {}, "Must have Thead header row");
  _e1162.appendChild(_e1165);
  _e1121.appendChild(_e1162);
  const _e1166 = WF.h("tr", {});
  const _e1167 = WF.h("td", {}, "A11");
  _e1166.appendChild(_e1167);
  const _e1168 = WF.h("td", {}, "Heading");
  _e1166.appendChild(_e1168);
  const _e1169 = WF.h("td", {}, "Levels must not skip (h1 to h3)");
  _e1166.appendChild(_e1169);
  _e1121.appendChild(_e1166);
  const _e1170 = WF.h("tr", {});
  const _e1171 = WF.h("td", {}, "A12");
  _e1170.appendChild(_e1171);
  const _e1172 = WF.h("td", {}, "Page");
  _e1170.appendChild(_e1172);
  const _e1173 = WF.h("td", {}, "Must have exactly one h1");
  _e1170.appendChild(_e1173);
  _e1121.appendChild(_e1170);
  _e1106.appendChild(_e1121);
  const _e1174 = WF.h("div", { className: "wf-spacer" });
  _e1106.appendChild(_e1174);
  const _e1175 = WF.h("hr", { className: "wf-divider" });
  _e1106.appendChild(_e1175);
  const _e1176 = WF.h("div", { className: "wf-spacer" });
  _e1106.appendChild(_e1176);
  const _e1177 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Examples");
  _e1106.appendChild(_e1177);
  const _e1178 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1179 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1180 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1181 = WF.h("div", { className: "wf-card__body" });
  const _e1182 = WF.h("p", { className: "wf-text wf-text--danger wf-text--bold" }, "Bad (triggers warning)");
  _e1181.appendChild(_e1182);
  const _e1183 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\")\nIconButton(icon: \"close\")\nInput(text)\nCheckbox(bind: agreed)\nButton()");
  _e1181.appendChild(_e1183);
  _e1180.appendChild(_e1181);
  _e1179.appendChild(_e1180);
  _e1178.appendChild(_e1179);
  const _e1184 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1185 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1186 = WF.h("div", { className: "wf-card__body" });
  const _e1187 = WF.h("p", { className: "wf-text wf-text--success wf-text--bold" }, "Good (no warnings)");
  _e1186.appendChild(_e1187);
  const _e1188 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\", alt: \"Team photo\")\nIconButton(icon: \"close\", label: \"Close\")\nInput(text, label: \"Username\")\nCheckbox(bind: agreed, label: \"I agree\")\nButton(\"Save\")");
  _e1186.appendChild(_e1188);
  _e1185.appendChild(_e1186);
  _e1184.appendChild(_e1185);
  _e1178.appendChild(_e1184);
  _e1106.appendChild(_e1178);
  const _e1189 = WF.h("div", { className: "wf-spacer" });
  _e1106.appendChild(_e1189);
  _root.appendChild(_e1106);
  return _root;
}

(function() {
  const _app = document.getElementById('app');
  _app.innerHTML = '';
  const _e1190 = Component_NavBar({});
  _app.appendChild(_e1190);
  const _routerEl = document.createElement('div');
  _routerEl.id = 'wf-router';
  _routerEl.style.flex = '1';
  _app.appendChild(_routerEl);
  const _routes = [
    { path: "/", render: (params) => Page_Home(params) },
    { path: "/getting-started", render: (params) => Page_GettingStarted(params) },
    { path: "/guide", render: (params) => Page_Guide(params) },
    { path: "/components", render: (params) => Page_Components(params) },
    { path: "/styling", render: (params) => Page_Styling(params) },
    { path: "/animation", render: (params) => Page_Animation(params) },
    { path: "/i18n", render: (params) => Page_I18n(params) },
    { path: "/ssg", render: (params) => Page_Ssg(params) },
    { path: "/accessibility", render: (params) => Page_Accessibility(params) },
    { path: "/cli", render: (params) => Page_Cli(params) },
  ];
  WF.createRouter(_routes, _routerEl);
  const _e1191 = Component_SiteFooter({});
  _app.appendChild(_e1191);
})();
