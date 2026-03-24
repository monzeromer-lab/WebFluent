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
  function condRender(parent, condFn, thenFn, elseFn, animConfig) {
    const marker = document.createComment("wf-if");
    parent.appendChild(marker);
    let currentNodes = [];
    let animating = false;

    effect(() => {
      const show = condFn();
      if (animating) return;

      const removeOld = () => {
        if (currentNodes.length === 0) return Promise.resolve();
        if (animConfig && animConfig.exit) {
          animating = true;
          const exitName = animConfig.exit;
          const promises = currentNodes.map(n => n instanceof Element ? animateOut(n, exitName, animConfig.duration) : Promise.resolve());
          return Promise.all(promises).then(() => {
            for (const n of currentNodes) n.remove();
            currentNodes = [];
            animating = false;
          });
        }
        for (const n of currentNodes) n.remove();
        currentNodes = [];
        return Promise.resolve();
      };

      const addNew = (renderFn) => {
        const frag = document.createDocumentFragment();
        const nodes = [].concat(renderFn()).flat().filter(n => n instanceof Node);
        for (const n of nodes) { frag.appendChild(n); currentNodes.push(n); }
        marker.parentNode.insertBefore(frag, marker.nextSibling);
        if (animConfig && animConfig.enter) {
          nodes.forEach(n => { if (n instanceof Element) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay); });
        }
      };

      removeOld().then(() => {
        if (show && thenFn) addNew(thenFn);
        else if (!show && elseFn) addNew(elseFn);
      });
    });
  }

  // ─── List rendering ─────────────────────────────────
  function listRender(parent, listFn, itemFn, animConfig) {
    const marker = document.createComment("wf-for");
    parent.appendChild(marker);
    let currentNodes = [];

    effect(() => {
      // Remove old
      if (animConfig && animConfig.exit && currentNodes.length) {
        const exitName = animConfig.exit;
        currentNodes.forEach((n, i) => {
          if (n instanceof Element) {
            const delay = animConfig.stagger ? (parseInt(animConfig.stagger) * i) + "ms" : undefined;
            animateOut(n, exitName, animConfig.duration).then(() => n.remove());
          } else {
            n.remove();
          }
        });
      } else {
        for (const n of currentNodes) n.remove();
      }
      currentNodes = [];

      const items = listFn();
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
      marker.parentNode.insertBefore(frag, marker.nextSibling);
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

  function createRouter(routes, container) {
    const currentPath = signal(window.location.pathname);
    let currentCleanup = null;

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
      const path = currentPath();
      const match = matchRoute(path);
      container.innerHTML = "";

      if (match) {
        const el = match.route.render(match.params);
        if (el instanceof Node) container.appendChild(el);
      }
    }

    window.addEventListener("popstate", () => {
      currentPath.set(window.location.pathname);
    });

    effect(render);

    routerInstance = {
      navigate: (path) => {
        window.history.pushState(null, "", path);
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
  const _e2 = WF.h("code", { className: "wf-code wf-code--block" }, _code());
  _e1.appendChild(_e2);
  _e0.appendChild(_e1);
  _frag.appendChild(_e0);
  return _frag;
}

function Component_FeatureCard({ title, description }) {
  const _frag = document.createDocumentFragment();
  const _e3 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-scaleIn" });
  const _e4 = WF.h("div", { className: "wf-card__body" });
  const _e5 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, _title());
  _e4.appendChild(_e5);
  const _e6 = WF.h("div", { className: "wf-spacer" });
  _e4.appendChild(_e6);
  const _e7 = WF.h("p", { className: "wf-text wf-text--muted" }, _description());
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
  const _e136 = WF.h("p", { className: "wf-text wf-text--muted" }, "WebFluent has a token-based design system. Every built-in component uses design tokens for colors, spacing, typography, and more. Change the entire look with a single config update.");
  _e133.appendChild(_e136);
  const _e137 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e137);
  const _e138 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Variant Modifiers");
  _e133.appendChild(_e138);
  const _e139 = WF.h("p", { className: "wf-text" }, "Apply common styles with modifier keywords on any component.");
  _e133.appendChild(_e139);
  const _e140 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e140);
  const _e141 = WF.h("table", { className: "wf-table" });
  const _e142 = WF.h("thead", {});
  const _e143 = WF.h("td", {}, "Category");
  _e142.appendChild(_e143);
  const _e144 = WF.h("td", {}, "Modifiers");
  _e142.appendChild(_e144);
  _e141.appendChild(_e142);
  const _e145 = WF.h("tr", {});
  const _e146 = WF.h("td", {}, "Size");
  _e145.appendChild(_e146);
  const _e147 = WF.h("td", {}, "small, medium (default), large");
  _e145.appendChild(_e147);
  _e141.appendChild(_e145);
  const _e148 = WF.h("tr", {});
  const _e149 = WF.h("td", {}, "Color");
  _e148.appendChild(_e149);
  const _e150 = WF.h("td", {}, "primary, secondary, success, danger, warning, info");
  _e148.appendChild(_e150);
  _e141.appendChild(_e148);
  const _e151 = WF.h("tr", {});
  const _e152 = WF.h("td", {}, "Shape");
  _e151.appendChild(_e152);
  const _e153 = WF.h("td", {}, "rounded, pill, square");
  _e151.appendChild(_e153);
  _e141.appendChild(_e151);
  const _e154 = WF.h("tr", {});
  const _e155 = WF.h("td", {}, "Elevation");
  _e154.appendChild(_e155);
  const _e156 = WF.h("td", {}, "flat, elevated, outlined");
  _e154.appendChild(_e156);
  _e141.appendChild(_e154);
  const _e157 = WF.h("tr", {});
  const _e158 = WF.h("td", {}, "Width");
  _e157.appendChild(_e158);
  const _e159 = WF.h("td", {}, "full (100%), fit (fit-content)");
  _e157.appendChild(_e159);
  _e141.appendChild(_e157);
  const _e160 = WF.h("tr", {});
  const _e161 = WF.h("td", {}, "Text");
  _e160.appendChild(_e161);
  const _e162 = WF.h("td", {}, "bold, italic, underline, uppercase, lowercase");
  _e160.appendChild(_e162);
  _e141.appendChild(_e160);
  const _e163 = WF.h("tr", {});
  const _e164 = WF.h("td", {}, "Alignment");
  _e163.appendChild(_e164);
  const _e165 = WF.h("td", {}, "left, center, right");
  _e163.appendChild(_e165);
  _e141.appendChild(_e163);
  _e133.appendChild(_e141);
  const _e166 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e166);
  const _e167 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e168 = WF.h("div", { className: "wf-card__body" });
  const _e169 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Save\", primary, large, rounded)\nText(\"Warning!\", danger, bold, uppercase)\nCard(elevated, outlined) { Text(\"Content\") }\nInput(text, full, rounded)");
  _e168.appendChild(_e169);
  _e167.appendChild(_e168);
  _e133.appendChild(_e167);
  const _e170 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e170);
  const _e171 = WF.h("hr", { className: "wf-divider" });
  _e133.appendChild(_e171);
  const _e172 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e172);
  const _e173 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Style Blocks");
  _e133.appendChild(_e173);
  const _e174 = WF.h("p", { className: "wf-text" }, "For custom styling beyond modifiers, use a style block inside any component.");
  _e133.appendChild(_e174);
  const _e175 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e175);
  const _e176 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e177 = WF.h("div", { className: "wf-card__body" });
  const _e178 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Custom\") {\n    style {\n        background: \"#8B5CF6\"\n        color: \"#FFFFFF\"\n        padding: xl\n        radius: lg\n        shadow: md\n    }\n}");
  _e177.appendChild(_e178);
  _e176.appendChild(_e177);
  _e133.appendChild(_e176);
  const _e179 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e179);
  const _e180 = WF.h("p", { className: "wf-text wf-text--muted" }, "Token references: Use token names directly — background: primary references the color-primary token. padding: md references spacing-md.");
  _e133.appendChild(_e180);
  const _e181 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e181);
  const _e182 = WF.h("hr", { className: "wf-divider" });
  _e133.appendChild(_e182);
  const _e183 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e183);
  const _e184 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Design Tokens");
  _e133.appendChild(_e184);
  const _e185 = WF.h("p", { className: "wf-text" }, "All styling is built on tokens — named values defined as CSS custom properties.");
  _e133.appendChild(_e185);
  const _e186 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e186);
  const _e187 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Colors");
  _e133.appendChild(_e187);
  const _e188 = WF.h("table", { className: "wf-table" });
  const _e189 = WF.h("thead", {});
  const _e190 = WF.h("td", {}, "Token");
  _e189.appendChild(_e190);
  const _e191 = WF.h("td", {}, "Default Value");
  _e189.appendChild(_e191);
  _e188.appendChild(_e189);
  const _e192 = WF.h("tr", {});
  const _e193 = WF.h("td", {}, "color-primary");
  _e192.appendChild(_e193);
  const _e194 = WF.h("td", {}, "#3B82F6");
  _e192.appendChild(_e194);
  _e188.appendChild(_e192);
  const _e195 = WF.h("tr", {});
  const _e196 = WF.h("td", {}, "color-secondary");
  _e195.appendChild(_e196);
  const _e197 = WF.h("td", {}, "#64748B");
  _e195.appendChild(_e197);
  _e188.appendChild(_e195);
  const _e198 = WF.h("tr", {});
  const _e199 = WF.h("td", {}, "color-success");
  _e198.appendChild(_e199);
  const _e200 = WF.h("td", {}, "#22C55E");
  _e198.appendChild(_e200);
  _e188.appendChild(_e198);
  const _e201 = WF.h("tr", {});
  const _e202 = WF.h("td", {}, "color-danger");
  _e201.appendChild(_e202);
  const _e203 = WF.h("td", {}, "#EF4444");
  _e201.appendChild(_e203);
  _e188.appendChild(_e201);
  const _e204 = WF.h("tr", {});
  const _e205 = WF.h("td", {}, "color-warning");
  _e204.appendChild(_e205);
  const _e206 = WF.h("td", {}, "#F59E0B");
  _e204.appendChild(_e206);
  _e188.appendChild(_e204);
  const _e207 = WF.h("tr", {});
  const _e208 = WF.h("td", {}, "color-info");
  _e207.appendChild(_e208);
  const _e209 = WF.h("td", {}, "#06B6D4");
  _e207.appendChild(_e209);
  _e188.appendChild(_e207);
  const _e210 = WF.h("tr", {});
  const _e211 = WF.h("td", {}, "color-background");
  _e210.appendChild(_e211);
  const _e212 = WF.h("td", {}, "#FFFFFF");
  _e210.appendChild(_e212);
  _e188.appendChild(_e210);
  const _e213 = WF.h("tr", {});
  const _e214 = WF.h("td", {}, "color-surface");
  _e213.appendChild(_e214);
  const _e215 = WF.h("td", {}, "#F8FAFC");
  _e213.appendChild(_e215);
  _e188.appendChild(_e213);
  const _e216 = WF.h("tr", {});
  const _e217 = WF.h("td", {}, "color-text");
  _e216.appendChild(_e217);
  const _e218 = WF.h("td", {}, "#0F172A");
  _e216.appendChild(_e218);
  _e188.appendChild(_e216);
  const _e219 = WF.h("tr", {});
  const _e220 = WF.h("td", {}, "color-text-muted");
  _e219.appendChild(_e220);
  const _e221 = WF.h("td", {}, "#64748B");
  _e219.appendChild(_e221);
  _e188.appendChild(_e219);
  const _e222 = WF.h("tr", {});
  const _e223 = WF.h("td", {}, "color-border");
  _e222.appendChild(_e223);
  const _e224 = WF.h("td", {}, "#E2E8F0");
  _e222.appendChild(_e224);
  _e188.appendChild(_e222);
  _e133.appendChild(_e188);
  const _e225 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e225);
  const _e226 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Spacing, Radius, Shadows");
  _e133.appendChild(_e226);
  const _e227 = WF.h("table", { className: "wf-table" });
  const _e228 = WF.h("thead", {});
  const _e229 = WF.h("td", {}, "Token");
  _e228.appendChild(_e229);
  const _e230 = WF.h("td", {}, "Value");
  _e228.appendChild(_e230);
  _e227.appendChild(_e228);
  const _e231 = WF.h("tr", {});
  const _e232 = WF.h("td", {}, "spacing-xs / sm / md / lg / xl");
  _e231.appendChild(_e232);
  const _e233 = WF.h("td", {}, "0.25rem / 0.5rem / 1rem / 1.5rem / 2rem");
  _e231.appendChild(_e233);
  _e227.appendChild(_e231);
  const _e234 = WF.h("tr", {});
  const _e235 = WF.h("td", {}, "radius-sm / md / lg / full");
  _e234.appendChild(_e235);
  const _e236 = WF.h("td", {}, "0.25rem / 0.5rem / 1rem / 9999px");
  _e234.appendChild(_e236);
  _e227.appendChild(_e234);
  const _e237 = WF.h("tr", {});
  const _e238 = WF.h("td", {}, "shadow-sm / md / lg / xl");
  _e237.appendChild(_e238);
  const _e239 = WF.h("td", {}, "Increasing depth shadows");
  _e237.appendChild(_e239);
  _e227.appendChild(_e237);
  _e133.appendChild(_e227);
  const _e240 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e240);
  const _e241 = WF.h("hr", { className: "wf-divider" });
  _e133.appendChild(_e241);
  const _e242 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e242);
  const _e243 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Themes");
  _e133.appendChild(_e243);
  const _e244 = WF.h("p", { className: "wf-text" }, "WebFluent ships with 4 built-in themes. Set the theme in webfluent.app.json.");
  _e133.appendChild(_e244);
  const _e245 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e245);
  const _e246 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e247 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e248 = WF.h("div", { className: "wf-card__body" });
  const _e249 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "default");
  _e248.appendChild(_e249);
  const _e250 = WF.h("p", { className: "wf-text wf-text--muted" }, "Clean, modern light theme.");
  _e248.appendChild(_e250);
  _e247.appendChild(_e248);
  _e246.appendChild(_e247);
  const _e251 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e252 = WF.h("div", { className: "wf-card__body" });
  const _e253 = WF.h("span", { className: "wf-badge wf-badge--secondary" }, "dark");
  _e252.appendChild(_e253);
  const _e254 = WF.h("p", { className: "wf-text wf-text--muted" }, "Dark mode with muted tones.");
  _e252.appendChild(_e254);
  _e251.appendChild(_e252);
  _e246.appendChild(_e251);
  const _e255 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e256 = WF.h("div", { className: "wf-card__body" });
  const _e257 = WF.h("span", { className: "wf-badge" }, "minimal");
  _e256.appendChild(_e257);
  const _e258 = WF.h("p", { className: "wf-text wf-text--muted" }, "Ultra-minimal, no shadows or radii.");
  _e256.appendChild(_e258);
  _e255.appendChild(_e256);
  _e246.appendChild(_e255);
  const _e259 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e260 = WF.h("div", { className: "wf-card__body" });
  const _e261 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "brutalist");
  _e260.appendChild(_e261);
  const _e262 = WF.h("p", { className: "wf-text wf-text--muted" }, "Bold, monospace, hard shadows.");
  _e260.appendChild(_e262);
  _e259.appendChild(_e260);
  _e246.appendChild(_e259);
  _e133.appendChild(_e246);
  const _e263 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e263);
  const _e264 = WF.h("p", { className: "wf-text wf-text--bold" }, "Override any token in your config:");
  _e133.appendChild(_e264);
  const _e265 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e266 = WF.h("div", { className: "wf-card__body" });
  const _e267 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"theme\": {\n    \"name\": \"default\",\n    \"tokens\": {\n      \"color-primary\": \"#8B5CF6\",\n      \"font-family\": \"Poppins, sans-serif\",\n      \"radius-md\": \"1rem\"\n    }\n  }\n}");
  _e266.appendChild(_e267);
  _e265.appendChild(_e266);
  _e133.appendChild(_e265);
  const _e268 = WF.h("div", { className: "wf-spacer" });
  _e133.appendChild(_e268);
  _root.appendChild(_e133);
  return _root;
}

function Page_Guide(params) {
  const _root = document.createDocumentFragment();
  const _e269 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e270 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e270);
  const _e271 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Language Guide");
  _e269.appendChild(_e271);
  const _e272 = WF.h("p", { className: "wf-text wf-text--muted" }, "Learn the core concepts of WebFluent.");
  _e269.appendChild(_e272);
  const _e273 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e273);
  const _e274 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Pages");
  _e269.appendChild(_e274);
  const _e275 = WF.h("p", { className: "wf-text" }, "Pages are top-level route targets. Each page defines a URL path and contains the UI tree for that route.");
  _e269.appendChild(_e275);
  const _e276 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e276);
  const _e277 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e278 = WF.h("div", { className: "wf-card__body" });
  const _e279 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\", title: \"Home\") {\n    Container {\n        Heading(\"Welcome\", h1)\n        Text(\"This is the home page.\")\n    }\n}");
  _e278.appendChild(_e279);
  _e277.appendChild(_e278);
  _e269.appendChild(_e277);
  const _e280 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e280);
  const _e281 = WF.h("p", { className: "wf-text wf-text--bold" }, "Page attributes:");
  _e269.appendChild(_e281);
  const _e282 = WF.h("table", { className: "wf-table" });
  const _e283 = WF.h("thead", {});
  const _e284 = WF.h("td", {}, "Attribute");
  _e283.appendChild(_e284);
  const _e285 = WF.h("td", {}, "Type");
  _e283.appendChild(_e285);
  const _e286 = WF.h("td", {}, "Description");
  _e283.appendChild(_e286);
  _e282.appendChild(_e283);
  const _e287 = WF.h("tr", {});
  const _e288 = WF.h("td", {}, "path");
  _e287.appendChild(_e288);
  const _e289 = WF.h("td", {}, "String");
  _e287.appendChild(_e289);
  const _e290 = WF.h("td", {}, "URL route for this page (required)");
  _e287.appendChild(_e290);
  _e282.appendChild(_e287);
  const _e291 = WF.h("tr", {});
  const _e292 = WF.h("td", {}, "title");
  _e291.appendChild(_e292);
  const _e293 = WF.h("td", {}, "String");
  _e291.appendChild(_e293);
  const _e294 = WF.h("td", {}, "Document title");
  _e291.appendChild(_e294);
  _e282.appendChild(_e291);
  const _e295 = WF.h("tr", {});
  const _e296 = WF.h("td", {}, "guard");
  _e295.appendChild(_e296);
  const _e297 = WF.h("td", {}, "Expression");
  _e295.appendChild(_e297);
  const _e298 = WF.h("td", {}, "Navigation guard — redirects if false");
  _e295.appendChild(_e298);
  _e282.appendChild(_e295);
  const _e299 = WF.h("tr", {});
  const _e300 = WF.h("td", {}, "redirect");
  _e299.appendChild(_e300);
  const _e301 = WF.h("td", {}, "String");
  _e299.appendChild(_e301);
  const _e302 = WF.h("td", {}, "Redirect target when guard fails");
  _e299.appendChild(_e302);
  _e282.appendChild(_e299);
  _e269.appendChild(_e282);
  const _e303 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e303);
  const _e304 = WF.h("hr", { className: "wf-divider" });
  _e269.appendChild(_e304);
  const _e305 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e305);
  const _e306 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Components");
  _e269.appendChild(_e306);
  const _e307 = WF.h("p", { className: "wf-text" }, "Reusable UI blocks that accept props and can have internal state.");
  _e269.appendChild(_e307);
  const _e308 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e308);
  const _e309 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e310 = WF.h("div", { className: "wf-card__body" });
  const _e311 = WF.h("code", { className: "wf-code wf-code--block" }, "Component UserCard (name: String, role: String, active: Bool = true) {\n    Card(elevated) {\n        Row(align: center, gap: md) {\n            Avatar(initials: \"U\", primary)\n            Stack {\n                Text(name, bold)\n                Text(role, muted)\n            }\n            if active {\n                Badge(\"Active\", success)\n            }\n        }\n    }\n}\n\n// Usage\nUserCard(name: \"Monzer\", role: \"Developer\")");
  _e310.appendChild(_e311);
  _e309.appendChild(_e310);
  _e269.appendChild(_e309);
  const _e312 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e312);
  const _e313 = WF.h("p", { className: "wf-text wf-text--muted" }, "Props support types: String, Number, Bool, List, Map. Optional props use ?, defaults use =.");
  _e269.appendChild(_e313);
  const _e314 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e314);
  const _e315 = WF.h("hr", { className: "wf-divider" });
  _e269.appendChild(_e315);
  const _e316 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e316);
  const _e317 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "State and Reactivity");
  _e269.appendChild(_e317);
  const _e318 = WF.h("p", { className: "wf-text" }, "State is declared with the state keyword. It is reactive — any UI that reads it updates automatically when it changes.");
  _e269.appendChild(_e318);
  const _e319 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e319);
  const _e320 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e321 = WF.h("div", { className: "wf-card__body" });
  const _e322 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Counter (path: \"/counter\") {\n    state count = 0\n\n    Container {\n        Text(\"Count: {count}\")\n        Button(\"+1\", primary) { count = count + 1 }\n        Button(\"-1\") { count = count - 1 }\n    }\n}");
  _e321.appendChild(_e322);
  _e320.appendChild(_e321);
  _e269.appendChild(_e320);
  const _e323 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e323);
  const _e324 = WF.h("p", { className: "wf-text wf-text--bold" }, "Derived state:");
  _e269.appendChild(_e324);
  const _e325 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e326 = WF.h("div", { className: "wf-card__body" });
  const _e327 = WF.h("code", { className: "wf-code wf-code--block" }, "state items = [{name: \"A\", price: 3}, {name: \"B\", price: 2}]\nderived total = items.map(i => i.price).sum()\nderived isEmpty = items.length == 0");
  _e326.appendChild(_e327);
  _e325.appendChild(_e326);
  _e269.appendChild(_e325);
  const _e328 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e328);
  const _e329 = WF.h("hr", { className: "wf-divider" });
  _e269.appendChild(_e329);
  const _e330 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e330);
  const _e331 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Events");
  _e269.appendChild(_e331);
  const _e332 = WF.h("p", { className: "wf-text" }, "Event handlers are declared with on:event or via shorthand blocks on buttons.");
  _e269.appendChild(_e332);
  const _e333 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e333);
  const _e334 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e335 = WF.h("div", { className: "wf-card__body" });
  const _e336 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Submit\") {\n    on:click {\n        submitForm()\n    }\n}\n\nInput(text, placeholder: \"Search...\") {\n    on:input {\n        searchQuery = value\n    }\n    on:keydown {\n        if key == \"Enter\" {\n            performSearch()\n        }\n    }\n}\n\n// Shorthand: block on Button defaults to on:click\nButton(\"Save\") { save() }");
  _e335.appendChild(_e336);
  _e334.appendChild(_e335);
  _e269.appendChild(_e334);
  const _e337 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e337);
  const _e338 = WF.h("p", { className: "wf-text wf-text--muted" }, "Supported events: on:click, on:submit, on:input, on:change, on:focus, on:blur, on:keydown, on:keyup, on:mouseover, on:mouseout, on:mount, on:unmount");
  _e269.appendChild(_e338);
  const _e339 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e339);
  const _e340 = WF.h("hr", { className: "wf-divider" });
  _e269.appendChild(_e340);
  const _e341 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e341);
  const _e342 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Control Flow");
  _e269.appendChild(_e342);
  const _e343 = WF.h("p", { className: "wf-text wf-text--bold" }, "Conditionals:");
  _e269.appendChild(_e343);
  const _e344 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e345 = WF.h("div", { className: "wf-card__body" });
  const _e346 = WF.h("code", { className: "wf-code wf-code--block" }, "if isLoggedIn {\n    Text(\"Welcome back!\")\n} else if isGuest {\n    Text(\"Hello, guest\")\n} else {\n    Button(\"Log In\") { navigate(\"/login\") }\n}");
  _e345.appendChild(_e346);
  _e344.appendChild(_e345);
  _e269.appendChild(_e344);
  const _e347 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e347);
  const _e348 = WF.h("p", { className: "wf-text wf-text--bold" }, "Loops:");
  _e269.appendChild(_e348);
  const _e349 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e350 = WF.h("div", { className: "wf-card__body" });
  const _e351 = WF.h("code", { className: "wf-code wf-code--block" }, "for user in users {\n    UserCard(name: user.name, role: user.role)\n}\n\n// With index\nfor item, index in items {\n    Text(\"{index + 1}. {item}\")\n}");
  _e350.appendChild(_e351);
  _e349.appendChild(_e350);
  _e269.appendChild(_e349);
  const _e352 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e352);
  const _e353 = WF.h("p", { className: "wf-text wf-text--bold" }, "Show/Hide (keeps element in DOM, toggles visibility):");
  _e269.appendChild(_e353);
  const _e354 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e355 = WF.h("div", { className: "wf-card__body" });
  const _e356 = WF.h("code", { className: "wf-code wf-code--block" }, "show isExpanded {\n    Card { Text(\"Expanded content\") }\n}");
  _e355.appendChild(_e356);
  _e354.appendChild(_e355);
  _e269.appendChild(_e354);
  const _e357 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e357);
  const _e358 = WF.h("hr", { className: "wf-divider" });
  _e269.appendChild(_e358);
  const _e359 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e359);
  const _e360 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Stores");
  _e269.appendChild(_e360);
  const _e361 = WF.h("p", { className: "wf-text" }, "Stores hold shared state accessible from any page or component.");
  _e269.appendChild(_e361);
  const _e362 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e362);
  const _e363 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e364 = WF.h("div", { className: "wf-card__body" });
  const _e365 = WF.h("code", { className: "wf-code wf-code--block" }, "Store CartStore {\n    state items = []\n\n    derived total = items.map(i => i.price * i.quantity).sum()\n    derived count = items.length\n\n    action addItem(product: Map) {\n        items.push({ id: product.id, name: product.name, price: product.price, quantity: 1 })\n    }\n\n    action removeItem(id: Number) {\n        items = items.filter(i => i.id != id)\n    }\n}\n\n// Usage in a page\nPage Cart (path: \"/cart\") {\n    use CartStore\n\n    Text(\"Total: ${CartStore.total}\")\n    Button(\"Clear\") { CartStore.clear() }\n}");
  _e364.appendChild(_e365);
  _e363.appendChild(_e364);
  _e269.appendChild(_e363);
  const _e366 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e366);
  const _e367 = WF.h("hr", { className: "wf-divider" });
  _e269.appendChild(_e367);
  const _e368 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e368);
  const _e369 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Routing");
  _e269.appendChild(_e369);
  const _e370 = WF.h("p", { className: "wf-text" }, "SPA routing is declared in the App file.");
  _e269.appendChild(_e370);
  const _e371 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e371);
  const _e372 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e373 = WF.h("div", { className: "wf-card__body" });
  const _e374 = WF.h("code", { className: "wf-code wf-code--block" }, "App {\n    Navbar {\n        Navbar.Brand { Text(\"My App\", heading) }\n        Navbar.Links {\n            Link(to: \"/\") { Text(\"Home\") }\n            Link(to: \"/about\") { Text(\"About\") }\n        }\n    }\n\n    Router {\n        Route(path: \"/\", page: Home)\n        Route(path: \"/about\", page: About)\n        Route(path: \"/user/:id\", page: UserProfile)\n        Route(path: \"*\", page: NotFound)\n    }\n}\n\n// Programmatic navigation\nButton(\"Go Home\") { navigate(\"/\") }\n\n// Dynamic routes access params\nPage UserProfile (path: \"/user/:id\") {\n    Text(\"User ID: {params.id}\")\n}");
  _e373.appendChild(_e374);
  _e372.appendChild(_e373);
  _e269.appendChild(_e372);
  const _e375 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e375);
  const _e376 = WF.h("hr", { className: "wf-divider" });
  _e269.appendChild(_e376);
  const _e377 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e377);
  const _e378 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Fetching");
  _e269.appendChild(_e378);
  const _e379 = WF.h("p", { className: "wf-text" }, "Built-in async data loading with automatic loading, error, and success states.");
  _e269.appendChild(_e379);
  const _e380 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e380);
  const _e381 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e382 = WF.h("div", { className: "wf-card__body" });
  const _e383 = WF.h("code", { className: "wf-code wf-code--block" }, "fetch users from \"/api/users\" {\n    loading {\n        Spinner()\n    }\n    error (err) {\n        Alert(\"Failed to load users\", danger)\n    }\n    success {\n        for user in users {\n            UserCard(name: user.name, role: user.role)\n        }\n    }\n}\n\n// With options\nfetch result from \"/api/submit\" (method: \"POST\", body: { name: name, email: email }) {\n    success {\n        Alert(\"Saved!\", success)\n    }\n}");
  _e382.appendChild(_e383);
  _e381.appendChild(_e382);
  _e269.appendChild(_e381);
  const _e384 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e384);
  const _e385 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e386 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/components"); } }, "Components Reference");
  _e385.appendChild(_e386);
  const _e387 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e385.appendChild(_e387);
  _e269.appendChild(_e385);
  const _e388 = WF.h("div", { className: "wf-spacer" });
  _e269.appendChild(_e388);
  _root.appendChild(_e269);
  return _root;
}

function Page_Animation(params) {
  const _root = document.createDocumentFragment();
  const _e389 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e390 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e390);
  const _e391 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Animation System");
  _e389.appendChild(_e391);
  const _e392 = WF.h("p", { className: "wf-text wf-text--muted" }, "Declarative animations built into the language — no CSS keyframes to write, no JS animation libraries.");
  _e389.appendChild(_e392);
  const _e393 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e393);
  const _e394 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Animation Modifiers");
  _e389.appendChild(_e394);
  const _e395 = WF.h("p", { className: "wf-text" }, "Apply a built-in animation to any component. The animation plays when the element mounts.");
  _e389.appendChild(_e395);
  const _e396 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e396);
  const _e397 = WF.h("table", { className: "wf-table" });
  const _e398 = WF.h("thead", {});
  const _e399 = WF.h("td", {}, "Modifier");
  _e398.appendChild(_e399);
  const _e400 = WF.h("td", {}, "Effect");
  _e398.appendChild(_e400);
  _e397.appendChild(_e398);
  const _e401 = WF.h("tr", {});
  const _e402 = WF.h("td", {}, "fadeIn / fadeOut");
  _e401.appendChild(_e402);
  const _e403 = WF.h("td", {}, "Fade opacity in or out");
  _e401.appendChild(_e403);
  _e397.appendChild(_e401);
  const _e404 = WF.h("tr", {});
  const _e405 = WF.h("td", {}, "slideUp / slideDown");
  _e404.appendChild(_e405);
  const _e406 = WF.h("td", {}, "Slide from below or above with fade");
  _e404.appendChild(_e406);
  _e397.appendChild(_e404);
  const _e407 = WF.h("tr", {});
  const _e408 = WF.h("td", {}, "slideLeft / slideRight");
  _e407.appendChild(_e408);
  const _e409 = WF.h("td", {}, "Slide from right or left with fade");
  _e407.appendChild(_e409);
  _e397.appendChild(_e407);
  const _e410 = WF.h("tr", {});
  const _e411 = WF.h("td", {}, "scaleIn / scaleOut");
  _e410.appendChild(_e411);
  const _e412 = WF.h("td", {}, "Scale up from 90% or down to 90%");
  _e410.appendChild(_e412);
  _e397.appendChild(_e410);
  const _e413 = WF.h("tr", {});
  const _e414 = WF.h("td", {}, "bounce");
  _e413.appendChild(_e414);
  const _e415 = WF.h("td", {}, "Bouncy entrance animation");
  _e413.appendChild(_e415);
  _e397.appendChild(_e413);
  const _e416 = WF.h("tr", {});
  const _e417 = WF.h("td", {}, "shake");
  _e416.appendChild(_e417);
  const _e418 = WF.h("td", {}, "Horizontal shake");
  _e416.appendChild(_e418);
  _e397.appendChild(_e416);
  const _e419 = WF.h("tr", {});
  const _e420 = WF.h("td", {}, "pulse");
  _e419.appendChild(_e420);
  const _e421 = WF.h("td", {}, "Gentle scale pulse (infinite)");
  _e419.appendChild(_e421);
  _e397.appendChild(_e419);
  const _e422 = WF.h("tr", {});
  const _e423 = WF.h("td", {}, "spin");
  _e422.appendChild(_e423);
  const _e424 = WF.h("td", {}, "360-degree rotation (infinite)");
  _e422.appendChild(_e424);
  _e397.appendChild(_e422);
  _e389.appendChild(_e397);
  const _e425 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e425);
  const _e426 = WF.h("p", { className: "wf-text wf-text--muted" }, "Speed variants: fast (150ms), default (300ms), slow (500ms)");
  _e389.appendChild(_e426);
  const _e427 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e427);
  const _e428 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e429 = WF.h("div", { className: "wf-card__body" });
  const _e430 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn) { Text(\"Fades in\") }\nHeading(\"Title\", h1, slideUp, slow)\nButton(\"Click\", primary, bounce)\nSpinner(pulse)");
  _e429.appendChild(_e430);
  _e428.appendChild(_e429);
  _e389.appendChild(_e428);
  const _e431 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e431);
  const _e432 = WF.h("hr", { className: "wf-divider" });
  _e389.appendChild(_e432);
  const _e433 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e433);
  const _e434 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Control Flow Animations");
  _e389.appendChild(_e434);
  const _e435 = WF.h("p", { className: "wf-text" }, "Add enter and exit animations to if, for, and show blocks.");
  _e389.appendChild(_e435);
  const _e436 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e436);
  const _e437 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e438 = WF.h("div", { className: "wf-card__body" });
  const _e439 = WF.h("code", { className: "wf-code wf-code--block" }, "// Conditional — animate in and out\nif showMessage, animate(fadeIn, fadeOut) {\n    Alert(\"Saved!\", success)\n}\n\n// List — stagger each item\nfor product in products, animate(slideUp, fadeOut, stagger: \"50ms\") {\n    Card { Text(product.name) }\n}\n\n// Show/hide — keeps in DOM\nshow isOpen, animate(scaleIn, scaleOut) {\n    Modal(title: \"Confirm\") { Text(\"Are you sure?\") }\n}");
  _e438.appendChild(_e439);
  _e437.appendChild(_e438);
  _e389.appendChild(_e437);
  const _e440 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e440);
  const _e441 = WF.h("p", { className: "wf-text wf-text--muted" }, "If only one animation is given, the exit auto-reverses: fadeIn exits with fadeOut, slideUp exits with slideDown.");
  _e389.appendChild(_e441);
  const _e442 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e442);
  const _e443 = WF.h("hr", { className: "wf-divider" });
  _e389.appendChild(_e443);
  const _e444 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e444);
  const _e445 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Transition Blocks");
  _e389.appendChild(_e445);
  const _e446 = WF.h("p", { className: "wf-text" }, "Declare CSS transitions for smooth property changes on hover, focus, or state change.");
  _e389.appendChild(_e446);
  const _e447 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e447);
  const _e448 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e449 = WF.h("div", { className: "wf-card__body" });
  const _e450 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Hover me\", primary) {\n    transition {\n        background 200ms ease\n        transform 150ms spring\n    }\n}");
  _e449.appendChild(_e450);
  _e448.appendChild(_e449);
  _e389.appendChild(_e448);
  const _e451 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e451);
  const _e452 = WF.h("p", { className: "wf-text wf-text--muted" }, "Easing options: ease, linear, easeIn, easeOut, easeInOut, spring, bouncy, smooth");
  _e389.appendChild(_e452);
  const _e453 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e453);
  const _e454 = WF.h("hr", { className: "wf-divider" });
  _e389.appendChild(_e454);
  const _e455 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e455);
  const _e456 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Animation Design Tokens");
  _e389.appendChild(_e456);
  const _e457 = WF.h("p", { className: "wf-text" }, "Customize animation timing globally via config tokens.");
  _e389.appendChild(_e457);
  const _e458 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e458);
  const _e459 = WF.h("table", { className: "wf-table" });
  const _e460 = WF.h("thead", {});
  const _e461 = WF.h("td", {}, "Token");
  _e460.appendChild(_e461);
  const _e462 = WF.h("td", {}, "Default");
  _e460.appendChild(_e462);
  _e459.appendChild(_e460);
  const _e463 = WF.h("tr", {});
  const _e464 = WF.h("td", {}, "animation-duration-fast");
  _e463.appendChild(_e464);
  const _e465 = WF.h("td", {}, "150ms");
  _e463.appendChild(_e465);
  _e459.appendChild(_e463);
  const _e466 = WF.h("tr", {});
  const _e467 = WF.h("td", {}, "animation-duration-normal");
  _e466.appendChild(_e467);
  const _e468 = WF.h("td", {}, "300ms");
  _e466.appendChild(_e468);
  _e459.appendChild(_e466);
  const _e469 = WF.h("tr", {});
  const _e470 = WF.h("td", {}, "animation-duration-slow");
  _e469.appendChild(_e470);
  const _e471 = WF.h("td", {}, "500ms");
  _e469.appendChild(_e471);
  _e459.appendChild(_e469);
  const _e472 = WF.h("tr", {});
  const _e473 = WF.h("td", {}, "animation-easing-default");
  _e472.appendChild(_e473);
  const _e474 = WF.h("td", {}, "cubic-bezier(0.4, 0, 0.2, 1)");
  _e472.appendChild(_e474);
  _e459.appendChild(_e472);
  const _e475 = WF.h("tr", {});
  const _e476 = WF.h("td", {}, "animation-easing-bounce");
  _e475.appendChild(_e476);
  const _e477 = WF.h("td", {}, "cubic-bezier(0.68, -0.55, 0.265, 1.55)");
  _e475.appendChild(_e477);
  _e459.appendChild(_e475);
  const _e478 = WF.h("tr", {});
  const _e479 = WF.h("td", {}, "animation-easing-spring");
  _e478.appendChild(_e479);
  const _e480 = WF.h("td", {}, "cubic-bezier(0.175, 0.885, 0.32, 1.275)");
  _e478.appendChild(_e480);
  _e459.appendChild(_e478);
  _e389.appendChild(_e459);
  const _e481 = WF.h("div", { className: "wf-spacer" });
  _e389.appendChild(_e481);
  _root.appendChild(_e389);
  return _root;
}

function Page_I18n(params) {
  const _root = document.createDocumentFragment();
  const _e482 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e483 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e483);
  const _e484 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Internationalization (i18n)");
  _e482.appendChild(_e484);
  const _e485 = WF.h("p", { className: "wf-text wf-text--muted" }, "Built-in multi-language support with reactive locale switching and automatic RTL.");
  _e482.appendChild(_e485);
  const _e486 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e486);
  const _e487 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Setup");
  _e482.appendChild(_e487);
  const _e488 = WF.h("p", { className: "wf-text" }, "Create a JSON file per locale in your translations directory.");
  _e482.appendChild(_e488);
  const _e489 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e489);
  const _e490 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e491 = WF.h("div", { className: "wf-card__body" });
  const _e492 = WF.h("code", { className: "wf-code wf-code--block" }, "// src/translations/en.json\n{\n    \"greeting\": \"Hello, {name}!\",\n    \"nav.home\": \"Home\",\n    \"nav.about\": \"About\"\n}\n\n// src/translations/ar.json\n{\n    \"greeting\": \"!أهلاً، {name}\",\n    \"nav.home\": \"الرئيسية\",\n    \"nav.about\": \"حول\"\n}");
  _e491.appendChild(_e492);
  _e490.appendChild(_e491);
  _e482.appendChild(_e490);
  const _e493 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e493);
  const _e494 = WF.h("p", { className: "wf-text wf-text--bold" }, "Add i18n config to webfluent.app.json:");
  _e482.appendChild(_e494);
  const _e495 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e496 = WF.h("div", { className: "wf-card__body" });
  const _e497 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e496.appendChild(_e497);
  _e495.appendChild(_e496);
  _e482.appendChild(_e495);
  const _e498 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e498);
  const _e499 = WF.h("hr", { className: "wf-divider" });
  _e482.appendChild(_e499);
  const _e500 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e500);
  const _e501 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "The t() Function");
  _e482.appendChild(_e501);
  const _e502 = WF.h("p", { className: "wf-text" }, "Use t() to look up translated text. It is reactive — all t() calls update when the locale changes.");
  _e482.appendChild(_e502);
  const _e503 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e503);
  const _e504 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e505 = WF.h("div", { className: "wf-card__body" });
  const _e506 = WF.h("code", { className: "wf-code wf-code--block" }, "// Simple key lookup\nText(t(\"nav.home\"))\n\n// With interpolation\nText(t(\"greeting\", name: user.name))\n\n// In any component\nButton(t(\"actions.save\"), primary)\nHeading(t(\"page.title\"), h1)");
  _e505.appendChild(_e506);
  _e504.appendChild(_e505);
  _e482.appendChild(_e504);
  const _e507 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e507);
  const _e508 = WF.h("hr", { className: "wf-divider" });
  _e482.appendChild(_e508);
  const _e509 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e509);
  const _e510 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Locale Switching");
  _e482.appendChild(_e510);
  const _e511 = WF.h("p", { className: "wf-text" }, "Switch the locale at runtime with setLocale(). All translated text updates instantly.");
  _e482.appendChild(_e511);
  const _e512 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e512);
  const _e513 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e514 = WF.h("div", { className: "wf-card__body" });
  const _e515 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"English\") { setLocale(\"en\") }\nButton(\"العربية\") { setLocale(\"ar\") }\nButton(\"Espanol\") { setLocale(\"es\") }\n\n// Access current locale\nText(\"Current: {locale}\")\nText(\"Direction: {dir}\")");
  _e514.appendChild(_e515);
  _e513.appendChild(_e514);
  _e482.appendChild(_e513);
  const _e516 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e516);
  const _e517 = WF.h("hr", { className: "wf-divider" });
  _e482.appendChild(_e517);
  const _e518 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e518);
  const _e519 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "RTL Support");
  _e482.appendChild(_e519);
  const _e520 = WF.h("p", { className: "wf-text" }, "WebFluent automatically detects RTL locales and updates the document direction.");
  _e482.appendChild(_e520);
  const _e521 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e521);
  const _e522 = WF.h("table", { className: "wf-table" });
  const _e523 = WF.h("thead", {});
  const _e524 = WF.h("td", {}, "Locale");
  _e523.appendChild(_e524);
  const _e525 = WF.h("td", {}, "Direction");
  _e523.appendChild(_e525);
  _e522.appendChild(_e523);
  const _e526 = WF.h("tr", {});
  const _e527 = WF.h("td", {}, "ar (Arabic)");
  _e526.appendChild(_e527);
  const _e528 = WF.h("td", {}, "RTL");
  _e526.appendChild(_e528);
  _e522.appendChild(_e526);
  const _e529 = WF.h("tr", {});
  const _e530 = WF.h("td", {}, "he (Hebrew)");
  _e529.appendChild(_e530);
  const _e531 = WF.h("td", {}, "RTL");
  _e529.appendChild(_e531);
  _e522.appendChild(_e529);
  const _e532 = WF.h("tr", {});
  const _e533 = WF.h("td", {}, "fa (Farsi)");
  _e532.appendChild(_e533);
  const _e534 = WF.h("td", {}, "RTL");
  _e532.appendChild(_e534);
  _e522.appendChild(_e532);
  const _e535 = WF.h("tr", {});
  const _e536 = WF.h("td", {}, "ur (Urdu)");
  _e535.appendChild(_e536);
  const _e537 = WF.h("td", {}, "RTL");
  _e535.appendChild(_e537);
  _e522.appendChild(_e535);
  const _e538 = WF.h("tr", {});
  const _e539 = WF.h("td", {}, "All others");
  _e538.appendChild(_e539);
  const _e540 = WF.h("td", {}, "LTR");
  _e538.appendChild(_e540);
  _e522.appendChild(_e538);
  _e482.appendChild(_e522);
  const _e541 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e541);
  const _e542 = WF.h("p", { className: "wf-text wf-text--muted" }, "When setLocale(\"ar\") is called, the HTML element gets dir=\"rtl\" and lang=\"ar\" automatically.");
  _e482.appendChild(_e542);
  const _e543 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e543);
  const _e544 = WF.h("hr", { className: "wf-divider" });
  _e482.appendChild(_e544);
  const _e545 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e545);
  const _e546 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Fallback Behavior");
  _e482.appendChild(_e546);
  const _e547 = WF.h("p", { className: "wf-text" }, "If a key is missing in the current locale:");
  _e482.appendChild(_e547);
  const _e548 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e549 = WF.h("p", { className: "wf-text" }, "1. Falls back to the defaultLocale translation");
  _e548.appendChild(_e549);
  const _e550 = WF.h("p", { className: "wf-text" }, "2. If still missing, returns the key itself (e.g., \"nav.home\")");
  _e548.appendChild(_e550);
  _e482.appendChild(_e548);
  const _e551 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e551);
  const _e552 = WF.h("hr", { className: "wf-divider" });
  _e482.appendChild(_e552);
  const _e553 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e553);
  const _e554 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "SSG + i18n");
  _e482.appendChild(_e554);
  const _e555 = WF.h("p", { className: "wf-text wf-text--muted" }, "When both SSG and i18n are enabled, pages are pre-rendered with the default locale text. After JavaScript loads, locale switching works normally.");
  _e482.appendChild(_e555);
  const _e556 = WF.h("div", { className: "wf-spacer" });
  _e482.appendChild(_e556);
  _root.appendChild(_e482);
  return _root;
}

function Page_GettingStarted(params) {
  const _root = document.createDocumentFragment();
  const _e557 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e558 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e558);
  const _e559 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Getting Started");
  _e557.appendChild(_e559);
  const _e560 = WF.h("p", { className: "wf-text wf-text--muted" }, "Get up and running with WebFluent in under a minute.");
  _e557.appendChild(_e560);
  const _e561 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e561);
  const _e562 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Installation");
  _e557.appendChild(_e562);
  const _e563 = WF.h("p", { className: "wf-text" }, "Build from source. You need Rust installed.");
  _e557.appendChild(_e563);
  const _e564 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e564);
  const _e565 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e566 = WF.h("div", { className: "wf-card__body" });
  const _e567 = WF.h("code", { className: "wf-code wf-code--block" }, "git clone https://github.com/user/webfluent.git\ncd webfluent\ncargo build --release");
  _e566.appendChild(_e567);
  _e565.appendChild(_e566);
  _e557.appendChild(_e565);
  const _e568 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e568);
  const _e569 = WF.h("p", { className: "wf-text wf-text--muted" }, "The binary is at target/release/webfluent. Add it to your PATH.");
  _e557.appendChild(_e569);
  const _e570 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e570);
  const _e571 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Create a Project");
  _e557.appendChild(_e571);
  const _e572 = WF.h("p", { className: "wf-text" }, "WebFluent comes with two starter templates.");
  _e557.appendChild(_e572);
  const _e573 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e573);
  const _e574 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e575 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e576 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e577 = WF.h("div", { className: "wf-card__body" });
  const _e578 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "SPA");
  _e577.appendChild(_e578);
  const _e579 = WF.h("div", { className: "wf-spacer" });
  _e577.appendChild(_e579);
  const _e580 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Interactive App");
  _e577.appendChild(_e580);
  const _e581 = WF.h("p", { className: "wf-text wf-text--muted" }, "Dashboard with routing, stores, forms, modals, animations. Full CRUD task manager.");
  _e577.appendChild(_e581);
  const _e582 = WF.h("div", { className: "wf-spacer" });
  _e577.appendChild(_e582);
  const _e583 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-app --template spa");
  _e577.appendChild(_e583);
  _e576.appendChild(_e577);
  _e575.appendChild(_e576);
  _e574.appendChild(_e575);
  const _e584 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e585 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e586 = WF.h("div", { className: "wf-card__body" });
  const _e587 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Static");
  _e586.appendChild(_e587);
  const _e588 = WF.h("div", { className: "wf-spacer" });
  _e586.appendChild(_e588);
  const _e589 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Static Site");
  _e586.appendChild(_e589);
  const _e590 = WF.h("p", { className: "wf-text wf-text--muted" }, "Marketing site with SSG, i18n, blog cards, contact form. Pre-rendered HTML for instant loads.");
  _e586.appendChild(_e590);
  const _e591 = WF.h("div", { className: "wf-spacer" });
  _e586.appendChild(_e591);
  const _e592 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-site --template static");
  _e586.appendChild(_e592);
  _e585.appendChild(_e586);
  _e584.appendChild(_e585);
  _e574.appendChild(_e584);
  _e557.appendChild(_e574);
  const _e593 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e593);
  const _e594 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Build and Serve");
  _e557.appendChild(_e594);
  const _e595 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e596 = WF.h("div", { className: "wf-card__body" });
  const _e597 = WF.h("code", { className: "wf-code wf-code--block" }, "cd my-app\nwf build\nwf serve");
  _e596.appendChild(_e597);
  _e595.appendChild(_e596);
  _e557.appendChild(_e595);
  const _e598 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e598);
  const _e599 = WF.h("p", { className: "wf-text wf-text--muted" }, "Open http://localhost:3000 in your browser.");
  _e557.appendChild(_e599);
  const _e600 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e600);
  const _e601 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Project Structure");
  _e557.appendChild(_e601);
  const _e602 = WF.h("p", { className: "wf-text" }, "A WebFluent project has this structure:");
  _e557.appendChild(_e602);
  const _e603 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e603);
  const _e604 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e605 = WF.h("div", { className: "wf-card__body" });
  const _e606 = WF.h("code", { className: "wf-code wf-code--block" }, "my-app/\n├── webfluent.app.json       # Project config\n├── src/\n│   ├── App.wf               # Root app (router, layout)\n│   ├── pages/               # Page files\n│   │   ├── Home.wf\n│   │   └── About.wf\n│   ├── components/           # Reusable components\n│   │   └── Header.wf\n│   ├── stores/               # Shared state\n│   │   └── auth.wf\n│   └── translations/         # i18n JSON files\n│       ├── en.json\n│       └── ar.json\n├── public/                   # Static assets\n└── build/                    # Compiled output");
  _e605.appendChild(_e606);
  _e604.appendChild(_e605);
  _e557.appendChild(_e604);
  const _e607 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e607);
  const _e608 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Build Output");
  _e557.appendChild(_e608);
  const _e609 = WF.h("p", { className: "wf-text" }, "The compiler generates three files:");
  _e557.appendChild(_e609);
  const _e610 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e610);
  const _e611 = WF.h("table", { className: "wf-table" });
  const _e612 = WF.h("thead", {});
  const _e613 = WF.h("td", {}, "File");
  _e612.appendChild(_e613);
  const _e614 = WF.h("td", {}, "Contents");
  _e612.appendChild(_e614);
  _e611.appendChild(_e612);
  const _e615 = WF.h("tr", {});
  const _e616 = WF.h("td", {}, "index.html");
  _e615.appendChild(_e616);
  const _e617 = WF.h("td", {}, "HTML shell (or pre-rendered pages with SSG)");
  _e615.appendChild(_e617);
  _e611.appendChild(_e615);
  const _e618 = WF.h("tr", {});
  const _e619 = WF.h("td", {}, "app.js");
  _e618.appendChild(_e619);
  const _e620 = WF.h("td", {}, "Reactive runtime + compiled pages and components");
  _e618.appendChild(_e620);
  _e611.appendChild(_e618);
  const _e621 = WF.h("tr", {});
  const _e622 = WF.h("td", {}, "styles.css");
  _e621.appendChild(_e622);
  const _e623 = WF.h("td", {}, "Design tokens as CSS custom properties + component styles");
  _e621.appendChild(_e623);
  _e611.appendChild(_e621);
  _e557.appendChild(_e611);
  const _e624 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e624);
  const _e625 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Next Steps");
  _e557.appendChild(_e625);
  const _e626 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e627 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/guide"); } }, "Read the Guide");
  _e626.appendChild(_e627);
  const _e628 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/components"); } }, "Browse Components");
  _e626.appendChild(_e628);
  const _e629 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/cli"); } }, "View CLI Reference");
  _e626.appendChild(_e629);
  _e557.appendChild(_e626);
  const _e630 = WF.h("div", { className: "wf-spacer" });
  _e557.appendChild(_e630);
  _root.appendChild(_e557);
  return _root;
}

function Page_Cli(params) {
  const _root = document.createDocumentFragment();
  const _e631 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e632 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e632);
  const _e633 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "CLI Reference");
  _e631.appendChild(_e633);
  const _e634 = WF.h("p", { className: "wf-text wf-text--muted" }, "The WebFluent command-line interface.");
  _e631.appendChild(_e634);
  const _e635 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e635);
  const _e636 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf init");
  _e631.appendChild(_e636);
  const _e637 = WF.h("p", { className: "wf-text" }, "Create a new WebFluent project.");
  _e631.appendChild(_e637);
  const _e638 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e638);
  const _e639 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e640 = WF.h("div", { className: "wf-card__body" });
  const _e641 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init <name> [--template spa|static]");
  _e640.appendChild(_e641);
  _e639.appendChild(_e640);
  _e631.appendChild(_e639);
  const _e642 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e642);
  const _e643 = WF.h("table", { className: "wf-table" });
  const _e644 = WF.h("thead", {});
  const _e645 = WF.h("td", {}, "Argument");
  _e644.appendChild(_e645);
  const _e646 = WF.h("td", {}, "Description");
  _e644.appendChild(_e646);
  _e643.appendChild(_e644);
  const _e647 = WF.h("tr", {});
  const _e648 = WF.h("td", {}, "name");
  _e647.appendChild(_e648);
  const _e649 = WF.h("td", {}, "Project name (creates a directory)");
  _e647.appendChild(_e649);
  _e643.appendChild(_e647);
  const _e650 = WF.h("tr", {});
  const _e651 = WF.h("td", {}, "--template, -t");
  _e650.appendChild(_e651);
  const _e652 = WF.h("td", {}, "Template: spa (default) or static");
  _e650.appendChild(_e652);
  _e643.appendChild(_e650);
  _e631.appendChild(_e643);
  const _e653 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e653);
  const _e654 = WF.h("p", { className: "wf-text wf-text--muted" }, "SPA template: interactive dashboard with routing, stores, forms, animations. Static template: marketing site with SSG, i18n, responsive grids.");
  _e631.appendChild(_e654);
  const _e655 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e655);
  const _e656 = WF.h("hr", { className: "wf-divider" });
  _e631.appendChild(_e656);
  const _e657 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e657);
  const _e658 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf build");
  _e631.appendChild(_e658);
  const _e659 = WF.h("p", { className: "wf-text" }, "Compile .wf files to HTML, CSS, and JavaScript.");
  _e631.appendChild(_e659);
  const _e660 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e660);
  const _e661 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e662 = WF.h("div", { className: "wf-card__body" });
  const _e663 = WF.h("code", { className: "wf-code wf-code--block" }, "wf build [--dir DIR]");
  _e662.appendChild(_e663);
  _e661.appendChild(_e662);
  _e631.appendChild(_e661);
  const _e664 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e664);
  const _e665 = WF.h("table", { className: "wf-table" });
  const _e666 = WF.h("thead", {});
  const _e667 = WF.h("td", {}, "Option");
  _e666.appendChild(_e667);
  const _e668 = WF.h("td", {}, "Description");
  _e666.appendChild(_e668);
  _e665.appendChild(_e666);
  const _e669 = WF.h("tr", {});
  const _e670 = WF.h("td", {}, "--dir, -d");
  _e669.appendChild(_e670);
  const _e671 = WF.h("td", {}, "Project directory (default: current directory)");
  _e669.appendChild(_e671);
  _e665.appendChild(_e669);
  _e631.appendChild(_e665);
  const _e672 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e672);
  const _e673 = WF.h("p", { className: "wf-text wf-text--muted" }, "The build pipeline: Lex all .wf files, parse to AST, run accessibility linter, generate HTML + CSS + JS, write to output directory.");
  _e631.appendChild(_e673);
  const _e674 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e674);
  const _e675 = WF.h("p", { className: "wf-text" }, "Output depends on config:");
  _e631.appendChild(_e675);
  const _e676 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e677 = WF.h("p", { className: "wf-text" }, "SPA mode (ssg: false): single index.html + app.js + styles.css");
  _e676.appendChild(_e677);
  const _e678 = WF.h("p", { className: "wf-text" }, "SSG mode (ssg: true): one HTML per page + app.js + styles.css");
  _e676.appendChild(_e678);
  _e631.appendChild(_e676);
  const _e679 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e679);
  const _e680 = WF.h("hr", { className: "wf-divider" });
  _e631.appendChild(_e680);
  const _e681 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e681);
  const _e682 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf serve");
  _e631.appendChild(_e682);
  const _e683 = WF.h("p", { className: "wf-text" }, "Start a development server that serves the built output.");
  _e631.appendChild(_e683);
  const _e684 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e684);
  const _e685 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e686 = WF.h("div", { className: "wf-card__body" });
  const _e687 = WF.h("code", { className: "wf-code wf-code--block" }, "wf serve [--dir DIR]");
  _e686.appendChild(_e687);
  _e685.appendChild(_e686);
  _e631.appendChild(_e685);
  const _e688 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e688);
  const _e689 = WF.h("p", { className: "wf-text wf-text--muted" }, "Serves files from the build directory. SPA fallback: all routes serve index.html so client-side routing works. Port is configured in webfluent.app.json (default: 3000).");
  _e631.appendChild(_e689);
  const _e690 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e690);
  const _e691 = WF.h("hr", { className: "wf-divider" });
  _e631.appendChild(_e691);
  const _e692 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e692);
  const _e693 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf generate");
  _e631.appendChild(_e693);
  const _e694 = WF.h("p", { className: "wf-text" }, "Scaffold a new page, component, or store.");
  _e631.appendChild(_e694);
  const _e695 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e695);
  const _e696 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e697 = WF.h("div", { className: "wf-card__body" });
  const _e698 = WF.h("code", { className: "wf-code wf-code--block" }, "wf generate <kind> <name> [--dir DIR]");
  _e697.appendChild(_e698);
  _e696.appendChild(_e697);
  _e631.appendChild(_e696);
  const _e699 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e699);
  const _e700 = WF.h("table", { className: "wf-table" });
  const _e701 = WF.h("thead", {});
  const _e702 = WF.h("td", {}, "Kind");
  _e701.appendChild(_e702);
  const _e703 = WF.h("td", {}, "Creates");
  _e701.appendChild(_e703);
  const _e704 = WF.h("td", {}, "Example");
  _e701.appendChild(_e704);
  _e700.appendChild(_e701);
  const _e705 = WF.h("tr", {});
  const _e706 = WF.h("td", {}, "page");
  _e705.appendChild(_e706);
  const _e707 = WF.h("td", {}, "src/pages/Name.wf");
  _e705.appendChild(_e707);
  const _e708 = WF.h("td", {}, "wf generate page About");
  _e705.appendChild(_e708);
  _e700.appendChild(_e705);
  const _e709 = WF.h("tr", {});
  const _e710 = WF.h("td", {}, "component");
  _e709.appendChild(_e710);
  const _e711 = WF.h("td", {}, "src/components/Name.wf");
  _e709.appendChild(_e711);
  const _e712 = WF.h("td", {}, "wf generate component Header");
  _e709.appendChild(_e712);
  _e700.appendChild(_e709);
  const _e713 = WF.h("tr", {});
  const _e714 = WF.h("td", {}, "store");
  _e713.appendChild(_e714);
  const _e715 = WF.h("td", {}, "src/stores/name.wf");
  _e713.appendChild(_e715);
  const _e716 = WF.h("td", {}, "wf generate store CartStore");
  _e713.appendChild(_e716);
  _e700.appendChild(_e713);
  _e631.appendChild(_e700);
  const _e717 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e717);
  const _e718 = WF.h("hr", { className: "wf-divider" });
  _e631.appendChild(_e718);
  const _e719 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e719);
  const _e720 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Configuration");
  _e631.appendChild(_e720);
  const _e721 = WF.h("p", { className: "wf-text" }, "All config is in webfluent.app.json at the project root.");
  _e631.appendChild(_e721);
  const _e722 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e722);
  const _e723 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e724 = WF.h("div", { className: "wf-card__body" });
  const _e725 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"name\": \"My App\",\n  \"version\": \"1.0.0\",\n  \"author\": \"Your Name\",\n  \"theme\": {\n    \"name\": \"default\",\n    \"mode\": \"light\",\n    \"tokens\": {\n      \"color-primary\": \"#6366F1\"\n    }\n  },\n  \"build\": {\n    \"output\": \"./build\",\n    \"minify\": true,\n    \"ssg\": false\n  },\n  \"dev\": {\n    \"port\": 3000\n  },\n  \"meta\": {\n    \"title\": \"My App\",\n    \"description\": \"Built with WebFluent\",\n    \"lang\": \"en\"\n  },\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e724.appendChild(_e725);
  _e723.appendChild(_e724);
  _e631.appendChild(_e723);
  const _e726 = WF.h("div", { className: "wf-spacer" });
  _e631.appendChild(_e726);
  _root.appendChild(_e631);
  return _root;
}

function Page_Home(params) {
  const _root = document.createDocumentFragment();
  const _e727 = WF.h("div", { className: "wf-container" });
  const _e728 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e728);
  const _e729 = WF.h("div", { className: "wf-stack wf-stack--gap-md" });
  const _e730 = WF.h("h2", { className: "wf-heading wf-heading--h1 wf-text--center wf-animate-slideUp" }, "The Web-First Language");
  _e729.appendChild(_e730);
  const _e731 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, "Build fully functional single-page applications and static sites that compile to clean HTML, CSS, and JavaScript. No frameworks. No dependencies. Just the web.");
  _e729.appendChild(_e731);
  const _e732 = WF.h("div", { className: "wf-spacer" });
  _e729.appendChild(_e732);
  const _e733 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e734 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, "Get Started");
  _e733.appendChild(_e734);
  const _e735 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { WF.navigate("/guide"); } }, "View Guide");
  _e733.appendChild(_e735);
  _e729.appendChild(_e733);
  _e727.appendChild(_e729);
  const _e736 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e736);
  const _e737 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e738 = WF.h("div", { className: "wf-card__body" });
  const _e739 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\") {\n    Container {\n        Heading(\"Hello, WebFluent!\", h1)\n        Text(\"Build for the web. Nothing else.\")\n\n        Button(\"Get Started\", primary, large) {\n            navigate(\"/docs\")\n        }\n    }\n}");
  _e738.appendChild(_e739);
  _e737.appendChild(_e738);
  _e727.appendChild(_e737);
  const _e740 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e740);
  const _e741 = WF.h("hr", { className: "wf-divider" });
  _e727.appendChild(_e741);
  const _e742 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e742);
  const _e743 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, "Why WebFluent?");
  _e727.appendChild(_e743);
  const _e744 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, "Everything you need to build for the web, built into the language.");
  _e727.appendChild(_e744);
  const _e745 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e745);
  const _e746 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e747 = Component_FeatureCard({ title: "Declarative Syntax", description: "No XML, no JSX, no templates. Write UI as readable declarations with curly braces and parentheses." });
  _e746.appendChild(_e747);
  const _e748 = Component_FeatureCard({ title: "50+ Built-in Components", description: "Navbar, Card, Modal, Form, Table, Tabs, and more. Every component has a default design out of the box." });
  _e746.appendChild(_e748);
  const _e749 = Component_FeatureCard({ title: "Signal-Based Reactivity", description: "Fine-grained DOM updates without a virtual DOM. When state changes, only the affected nodes update." });
  _e746.appendChild(_e749);
  _e727.appendChild(_e746);
  const _e750 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e750);
  const _e751 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e752 = Component_FeatureCard({ title: "Design System First", description: "Design tokens for colors, spacing, typography. Switch themes with a single config change." });
  _e751.appendChild(_e752);
  const _e753 = Component_FeatureCard({ title: "Animations Built In", description: "12 animations as modifiers: fadeIn, slideUp, bounce, shake. Enter/exit animations on conditionals and loops." });
  _e751.appendChild(_e753);
  const _e754 = Component_FeatureCard({ title: "i18n + RTL Support", description: "JSON translations with t() function. Reactive locale switching. Automatic RTL for Arabic, Hebrew, Farsi." });
  _e751.appendChild(_e754);
  _e727.appendChild(_e751);
  const _e755 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e755);
  const _e756 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e757 = Component_FeatureCard({ title: "Static Site Generation", description: "Pre-render pages at build time for instant content. JavaScript hydrates for interactivity." });
  _e756.appendChild(_e757);
  const _e758 = Component_FeatureCard({ title: "Accessibility Linting", description: "12 compile-time checks catch missing alt text, labels, headings. Warnings during build, never blocks." });
  _e756.appendChild(_e758);
  const _e759 = Component_FeatureCard({ title: "Zero Dependencies", description: "Compiles to vanilla HTML, CSS, and JS. No runtime framework. The output is pure web standards." });
  _e756.appendChild(_e759);
  _e727.appendChild(_e756);
  const _e760 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e760);
  const _e761 = WF.h("hr", { className: "wf-divider" });
  _e727.appendChild(_e761);
  const _e762 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e762);
  const _e763 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, "How It Works");
  _e727.appendChild(_e763);
  const _e764 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e764);
  const _e765 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e766 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e767 = WF.h("div", { className: "wf-card" });
  const _e768 = WF.h("div", { className: "wf-card__body" });
  const _e769 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "1. Write");
  _e768.appendChild(_e769);
  const _e770 = WF.h("p", { className: "wf-text wf-text--muted" }, "Write your UI in .wf files using the declarative syntax. Components, pages, stores, and styles — all in one language.");
  _e768.appendChild(_e770);
  _e767.appendChild(_e768);
  _e766.appendChild(_e767);
  _e765.appendChild(_e766);
  const _e771 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e772 = WF.h("div", { className: "wf-card" });
  const _e773 = WF.h("div", { className: "wf-card__body" });
  const _e774 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "2. Build");
  _e773.appendChild(_e774);
  const _e775 = WF.h("p", { className: "wf-text wf-text--muted" }, "The Rust compiler lexes, parses, lints for accessibility, and generates optimized HTML + CSS + JS output.");
  _e773.appendChild(_e775);
  _e772.appendChild(_e773);
  _e771.appendChild(_e772);
  _e765.appendChild(_e771);
  const _e776 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e777 = WF.h("div", { className: "wf-card" });
  const _e778 = WF.h("div", { className: "wf-card__body" });
  const _e779 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "3. Ship");
  _e778.appendChild(_e779);
  const _e780 = WF.h("p", { className: "wf-text wf-text--muted" }, "Deploy the static output anywhere — GitHub Pages, Netlify, Vercel, or any static hosting. No server required.");
  _e778.appendChild(_e780);
  _e777.appendChild(_e778);
  _e776.appendChild(_e777);
  _e765.appendChild(_e776);
  _e727.appendChild(_e765);
  const _e781 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e781);
  const _e782 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e783 = WF.h("div", { className: "wf-card__body" });
  const _e784 = WF.h("div", { className: "wf-row wf-row--center wf-row--between" });
  const _e785 = WF.h("div", { className: "wf-stack" });
  const _e786 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Ready to start?");
  _e785.appendChild(_e786);
  const _e787 = WF.h("p", { className: "wf-text wf-text--muted" }, "Create your first WebFluent project in seconds.");
  _e785.appendChild(_e787);
  _e784.appendChild(_e785);
  const _e788 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, "Get Started");
  _e784.appendChild(_e788);
  _e783.appendChild(_e784);
  _e782.appendChild(_e783);
  _e727.appendChild(_e782);
  const _e789 = WF.h("div", { className: "wf-spacer" });
  _e727.appendChild(_e789);
  _root.appendChild(_e727);
  return _root;
}

function Page_Components(params) {
  const _root = document.createDocumentFragment();
  const _e790 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e791 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e791);
  const _e792 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Components Reference");
  _e790.appendChild(_e792);
  const _e793 = WF.h("p", { className: "wf-text wf-text--muted" }, "WebFluent ships with 50+ built-in components across 8 categories.");
  _e790.appendChild(_e793);
  const _e794 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e794);
  const _e795 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Layout");
  _e790.appendChild(_e795);
  const _e796 = WF.h("table", { className: "wf-table" });
  const _e797 = WF.h("thead", {});
  const _e798 = WF.h("td", {}, "Component");
  _e797.appendChild(_e798);
  const _e799 = WF.h("td", {}, "HTML");
  _e797.appendChild(_e799);
  const _e800 = WF.h("td", {}, "Description");
  _e797.appendChild(_e800);
  _e796.appendChild(_e797);
  const _e801 = WF.h("tr", {});
  const _e802 = WF.h("td", {}, "Container");
  _e801.appendChild(_e802);
  const _e803 = WF.h("td", {}, "div");
  _e801.appendChild(_e803);
  const _e804 = WF.h("td", {}, "Centered max-width wrapper. Use fluid modifier for full-width.");
  _e801.appendChild(_e804);
  _e796.appendChild(_e801);
  const _e805 = WF.h("tr", {});
  const _e806 = WF.h("td", {}, "Row");
  _e805.appendChild(_e806);
  const _e807 = WF.h("td", {}, "div");
  _e805.appendChild(_e807);
  const _e808 = WF.h("td", {}, "Horizontal flex container. Accepts gap, align, justify.");
  _e805.appendChild(_e808);
  _e796.appendChild(_e805);
  const _e809 = WF.h("tr", {});
  const _e810 = WF.h("td", {}, "Column");
  _e809.appendChild(_e810);
  const _e811 = WF.h("td", {}, "div");
  _e809.appendChild(_e811);
  const _e812 = WF.h("td", {}, "Flex child with optional span (1-12 grid). Responsive: span: 12, md: 6, lg: 4.");
  _e809.appendChild(_e812);
  _e796.appendChild(_e809);
  const _e813 = WF.h("tr", {});
  const _e814 = WF.h("td", {}, "Grid");
  _e813.appendChild(_e814);
  const _e815 = WF.h("td", {}, "div");
  _e813.appendChild(_e815);
  const _e816 = WF.h("td", {}, "CSS Grid container. Set columns: N and gap.");
  _e813.appendChild(_e816);
  _e796.appendChild(_e813);
  const _e817 = WF.h("tr", {});
  const _e818 = WF.h("td", {}, "Stack");
  _e817.appendChild(_e818);
  const _e819 = WF.h("td", {}, "div");
  _e817.appendChild(_e819);
  const _e820 = WF.h("td", {}, "Vertical flex with consistent gap.");
  _e817.appendChild(_e820);
  _e796.appendChild(_e817);
  const _e821 = WF.h("tr", {});
  const _e822 = WF.h("td", {}, "Spacer");
  _e821.appendChild(_e822);
  const _e823 = WF.h("td", {}, "div");
  _e821.appendChild(_e823);
  const _e824 = WF.h("td", {}, "Empty space. Variants: sm, md (default), lg, xl.");
  _e821.appendChild(_e824);
  _e796.appendChild(_e821);
  const _e825 = WF.h("tr", {});
  const _e826 = WF.h("td", {}, "Divider");
  _e825.appendChild(_e826);
  const _e827 = WF.h("td", {}, "hr");
  _e825.appendChild(_e827);
  const _e828 = WF.h("td", {}, "Horizontal separator line.");
  _e825.appendChild(_e828);
  _e796.appendChild(_e825);
  _e790.appendChild(_e796);
  const _e829 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e829);
  const _e830 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e831 = WF.h("div", { className: "wf-card__body" });
  const _e832 = WF.h("code", { className: "wf-code wf-code--block" }, "Container {\n    Row(gap: md) {\n        Column(span: 6) { Text(\"Left\") }\n        Column(span: 6) { Text(\"Right\") }\n    }\n}\n\nGrid(columns: 3, gap: md) {\n    Card { Text(\"A\") }\n    Card { Text(\"B\") }\n    Card { Text(\"C\") }\n}");
  _e831.appendChild(_e832);
  _e830.appendChild(_e831);
  _e790.appendChild(_e830);
  const _e833 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e833);
  const _e834 = WF.h("hr", { className: "wf-divider" });
  _e790.appendChild(_e834);
  const _e835 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e835);
  const _e836 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Navigation");
  _e790.appendChild(_e836);
  const _e837 = WF.h("table", { className: "wf-table" });
  const _e838 = WF.h("thead", {});
  const _e839 = WF.h("td", {}, "Component");
  _e838.appendChild(_e839);
  const _e840 = WF.h("td", {}, "Description");
  _e838.appendChild(_e840);
  _e837.appendChild(_e838);
  const _e841 = WF.h("tr", {});
  const _e842 = WF.h("td", {}, "Navbar");
  _e841.appendChild(_e842);
  const _e843 = WF.h("td", {}, "Top navigation bar with Brand, Links, Actions sub-components.");
  _e841.appendChild(_e843);
  _e837.appendChild(_e841);
  const _e844 = WF.h("tr", {});
  const _e845 = WF.h("td", {}, "Sidebar");
  _e844.appendChild(_e845);
  const _e846 = WF.h("td", {}, "Side panel with Header, Item, Divider sub-components.");
  _e844.appendChild(_e846);
  _e837.appendChild(_e844);
  const _e847 = WF.h("tr", {});
  const _e848 = WF.h("td", {}, "Breadcrumb");
  _e847.appendChild(_e848);
  const _e849 = WF.h("td", {}, "Navigation trail with Item sub-components.");
  _e847.appendChild(_e849);
  _e837.appendChild(_e847);
  const _e850 = WF.h("tr", {});
  const _e851 = WF.h("td", {}, "Link");
  _e850.appendChild(_e851);
  const _e852 = WF.h("td", {}, "SPA navigation link. Uses to: attribute. Renders as <a>.");
  _e850.appendChild(_e852);
  _e837.appendChild(_e850);
  const _e853 = WF.h("tr", {});
  const _e854 = WF.h("td", {}, "Menu");
  _e853.appendChild(_e854);
  const _e855 = WF.h("td", {}, "Dropdown menu with trigger and items.");
  _e853.appendChild(_e855);
  _e837.appendChild(_e853);
  const _e856 = WF.h("tr", {});
  const _e857 = WF.h("td", {}, "Tabs / TabPage");
  _e856.appendChild(_e857);
  const _e858 = WF.h("td", {}, "Tabbed content panels.");
  _e856.appendChild(_e858);
  _e837.appendChild(_e856);
  _e790.appendChild(_e837);
  const _e859 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e859);
  const _e860 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e861 = WF.h("div", { className: "wf-card__body" });
  const _e862 = WF.h("code", { className: "wf-code wf-code--block" }, "Navbar {\n    Navbar.Brand { Text(\"My App\", heading) }\n    Navbar.Links {\n        Link(to: \"/\") { Text(\"Home\") }\n        Link(to: \"/about\") { Text(\"About\") }\n    }\n    Navbar.Actions {\n        Button(\"Sign In\", primary)\n    }\n}\n\nTabs {\n    TabPage(\"General\") { Text(\"General settings\") }\n    TabPage(\"Security\") { Text(\"Security settings\") }\n}");
  _e861.appendChild(_e862);
  _e860.appendChild(_e861);
  _e790.appendChild(_e860);
  const _e863 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e863);
  const _e864 = WF.h("hr", { className: "wf-divider" });
  _e790.appendChild(_e864);
  const _e865 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e865);
  const _e866 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Display");
  _e790.appendChild(_e866);
  const _e867 = WF.h("table", { className: "wf-table" });
  const _e868 = WF.h("thead", {});
  const _e869 = WF.h("td", {}, "Component");
  _e868.appendChild(_e869);
  const _e870 = WF.h("td", {}, "Description");
  _e868.appendChild(_e870);
  _e867.appendChild(_e868);
  const _e871 = WF.h("tr", {});
  const _e872 = WF.h("td", {}, "Card");
  _e871.appendChild(_e872);
  const _e873 = WF.h("td", {}, "Surface with optional Header, Body, Footer. Modifiers: elevated, outlined, flat.");
  _e871.appendChild(_e873);
  _e867.appendChild(_e871);
  const _e874 = WF.h("tr", {});
  const _e875 = WF.h("td", {}, "Table / Thead / Trow / Tcell");
  _e874.appendChild(_e875);
  const _e876 = WF.h("td", {}, "Data table with headers and rows.");
  _e874.appendChild(_e876);
  _e867.appendChild(_e874);
  const _e877 = WF.h("tr", {});
  const _e878 = WF.h("td", {}, "List / List.Item");
  _e877.appendChild(_e878);
  const _e879 = WF.h("td", {}, "Styled list with items.");
  _e877.appendChild(_e879);
  _e867.appendChild(_e877);
  const _e880 = WF.h("tr", {});
  const _e881 = WF.h("td", {}, "Badge");
  _e880.appendChild(_e881);
  const _e882 = WF.h("td", {}, "Small label. Color modifiers: primary, success, danger, warning.");
  _e880.appendChild(_e882);
  _e867.appendChild(_e880);
  const _e883 = WF.h("tr", {});
  const _e884 = WF.h("td", {}, "Avatar");
  _e883.appendChild(_e884);
  const _e885 = WF.h("td", {}, "User photo or initials. Use initials: or src: attribute.");
  _e883.appendChild(_e885);
  _e867.appendChild(_e883);
  const _e886 = WF.h("tr", {});
  const _e887 = WF.h("td", {}, "Tooltip");
  _e886.appendChild(_e887);
  const _e888 = WF.h("td", {}, "Hover popup with text: attribute.");
  _e886.appendChild(_e888);
  _e867.appendChild(_e886);
  const _e889 = WF.h("tr", {});
  const _e890 = WF.h("td", {}, "Tag");
  _e889.appendChild(_e890);
  const _e891 = WF.h("td", {}, "Removable label with on:remove event.");
  _e889.appendChild(_e891);
  _e867.appendChild(_e889);
  _e790.appendChild(_e867);
  const _e892 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e892);
  const _e893 = WF.h("hr", { className: "wf-divider" });
  _e790.appendChild(_e893);
  const _e894 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e894);
  const _e895 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Input");
  _e790.appendChild(_e895);
  const _e896 = WF.h("table", { className: "wf-table" });
  const _e897 = WF.h("thead", {});
  const _e898 = WF.h("td", {}, "Component");
  _e897.appendChild(_e898);
  const _e899 = WF.h("td", {}, "Description");
  _e897.appendChild(_e899);
  _e896.appendChild(_e897);
  const _e900 = WF.h("tr", {});
  const _e901 = WF.h("td", {}, "Input");
  _e900.appendChild(_e901);
  const _e902 = WF.h("td", {}, "Text input. Types: text, email, password, number, search, tel, url, date, time, color. Use bind: for two-way binding.");
  _e900.appendChild(_e902);
  _e896.appendChild(_e900);
  const _e903 = WF.h("tr", {});
  const _e904 = WF.h("td", {}, "Select / Option");
  _e903.appendChild(_e904);
  const _e905 = WF.h("td", {}, "Dropdown select with options.");
  _e903.appendChild(_e905);
  _e896.appendChild(_e903);
  const _e906 = WF.h("tr", {});
  const _e907 = WF.h("td", {}, "Checkbox");
  _e906.appendChild(_e907);
  const _e908 = WF.h("td", {}, "Toggle checkbox with label.");
  _e906.appendChild(_e908);
  _e896.appendChild(_e906);
  const _e909 = WF.h("tr", {});
  const _e910 = WF.h("td", {}, "Radio");
  _e909.appendChild(_e910);
  const _e911 = WF.h("td", {}, "Exclusive radio button with value and label.");
  _e909.appendChild(_e911);
  _e896.appendChild(_e909);
  const _e912 = WF.h("tr", {});
  const _e913 = WF.h("td", {}, "Switch");
  _e912.appendChild(_e913);
  const _e914 = WF.h("td", {}, "Toggle switch with label.");
  _e912.appendChild(_e914);
  _e896.appendChild(_e912);
  const _e915 = WF.h("tr", {});
  const _e916 = WF.h("td", {}, "Slider");
  _e915.appendChild(_e916);
  const _e917 = WF.h("td", {}, "Range slider with min, max, step, label.");
  _e915.appendChild(_e917);
  _e896.appendChild(_e915);
  const _e918 = WF.h("tr", {});
  const _e919 = WF.h("td", {}, "DatePicker");
  _e918.appendChild(_e919);
  const _e920 = WF.h("td", {}, "Date input with optional min/max.");
  _e918.appendChild(_e920);
  _e896.appendChild(_e918);
  const _e921 = WF.h("tr", {});
  const _e922 = WF.h("td", {}, "FileUpload");
  _e921.appendChild(_e922);
  const _e923 = WF.h("td", {}, "File picker with accept filter.");
  _e921.appendChild(_e923);
  _e896.appendChild(_e921);
  const _e924 = WF.h("tr", {});
  const _e925 = WF.h("td", {}, "Form");
  _e924.appendChild(_e925);
  const _e926 = WF.h("td", {}, "Form wrapper with on:submit handler.");
  _e924.appendChild(_e926);
  _e896.appendChild(_e924);
  _e790.appendChild(_e896);
  const _e927 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e927);
  const _e928 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e929 = WF.h("div", { className: "wf-card__body" });
  const _e930 = WF.h("code", { className: "wf-code wf-code--block" }, "state username = \"\"\nstate agreed = false\nstate theme = \"light\"\nstate volume = 50\n\nForm {\n    Input(text, bind: username, label: \"Username\", required: true)\n    Checkbox(bind: agreed, label: \"I agree to terms\")\n    Radio(bind: theme, value: \"light\", label: \"Light\")\n    Radio(bind: theme, value: \"dark\", label: \"Dark\")\n    Switch(bind: notifications, label: \"Notifications\")\n    Slider(bind: volume, min: 0, max: 100, label: \"Volume\")\n    Button(\"Save\", primary)\n    on:submit { save() }\n}");
  _e929.appendChild(_e930);
  _e928.appendChild(_e929);
  _e790.appendChild(_e928);
  const _e931 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e931);
  const _e932 = WF.h("hr", { className: "wf-divider" });
  _e790.appendChild(_e932);
  const _e933 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e933);
  const _e934 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Feedback");
  _e790.appendChild(_e934);
  const _e935 = WF.h("table", { className: "wf-table" });
  const _e936 = WF.h("thead", {});
  const _e937 = WF.h("td", {}, "Component");
  _e936.appendChild(_e937);
  const _e938 = WF.h("td", {}, "Description");
  _e936.appendChild(_e938);
  _e935.appendChild(_e936);
  const _e939 = WF.h("tr", {});
  const _e940 = WF.h("td", {}, "Alert");
  _e939.appendChild(_e940);
  const _e941 = WF.h("td", {}, "Static notification. Modifiers: success, danger, warning, info. Add dismissible.");
  _e939.appendChild(_e941);
  _e935.appendChild(_e939);
  const _e942 = WF.h("tr", {});
  const _e943 = WF.h("td", {}, "Toast");
  _e942.appendChild(_e943);
  const _e944 = WF.h("td", {}, "Temporary popup triggered from actions. Toast(\"Saved!\", success)");
  _e942.appendChild(_e944);
  _e935.appendChild(_e942);
  const _e945 = WF.h("tr", {});
  const _e946 = WF.h("td", {}, "Modal");
  _e945.appendChild(_e946);
  const _e947 = WF.h("td", {}, "Full dialog with overlay. Requires visible: state and title: attribute.");
  _e945.appendChild(_e947);
  _e935.appendChild(_e945);
  const _e948 = WF.h("tr", {});
  const _e949 = WF.h("td", {}, "Dialog");
  _e948.appendChild(_e949);
  const _e950 = WF.h("td", {}, "Lightweight dialog. Same API as Modal.");
  _e948.appendChild(_e950);
  _e935.appendChild(_e948);
  const _e951 = WF.h("tr", {});
  const _e952 = WF.h("td", {}, "Spinner");
  _e951.appendChild(_e952);
  const _e953 = WF.h("td", {}, "Loading indicator. Modifiers: large, primary.");
  _e951.appendChild(_e953);
  _e935.appendChild(_e951);
  const _e954 = WF.h("tr", {});
  const _e955 = WF.h("td", {}, "Progress");
  _e954.appendChild(_e955);
  const _e956 = WF.h("td", {}, "Progress bar with value and max attributes.");
  _e954.appendChild(_e956);
  _e935.appendChild(_e954);
  const _e957 = WF.h("tr", {});
  const _e958 = WF.h("td", {}, "Skeleton");
  _e957.appendChild(_e958);
  const _e959 = WF.h("td", {}, "Loading placeholder with shimmer animation.");
  _e957.appendChild(_e959);
  _e935.appendChild(_e957);
  _e790.appendChild(_e935);
  const _e960 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e960);
  const _e961 = WF.h("hr", { className: "wf-divider" });
  _e790.appendChild(_e961);
  const _e962 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e962);
  const _e963 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Actions");
  _e790.appendChild(_e963);
  const _e964 = WF.h("table", { className: "wf-table" });
  const _e965 = WF.h("thead", {});
  const _e966 = WF.h("td", {}, "Component");
  _e965.appendChild(_e966);
  const _e967 = WF.h("td", {}, "Description");
  _e965.appendChild(_e967);
  _e964.appendChild(_e965);
  const _e968 = WF.h("tr", {});
  const _e969 = WF.h("td", {}, "Button");
  _e968.appendChild(_e969);
  const _e970 = WF.h("td", {}, "Interactive button. First arg is text. Modifiers: primary, danger, small, large, full, rounded.");
  _e968.appendChild(_e970);
  _e964.appendChild(_e968);
  const _e971 = WF.h("tr", {});
  const _e972 = WF.h("td", {}, "IconButton");
  _e971.appendChild(_e972);
  const _e973 = WF.h("td", {}, "Icon-only button. Requires icon: and label: for accessibility.");
  _e971.appendChild(_e973);
  _e964.appendChild(_e971);
  const _e974 = WF.h("tr", {});
  const _e975 = WF.h("td", {}, "ButtonGroup");
  _e974.appendChild(_e975);
  const _e976 = WF.h("td", {}, "Group of buttons rendered as a connected strip.");
  _e974.appendChild(_e976);
  _e964.appendChild(_e974);
  const _e977 = WF.h("tr", {});
  const _e978 = WF.h("td", {}, "Dropdown");
  _e977.appendChild(_e978);
  const _e979 = WF.h("td", {}, "Button with dropdown menu. Uses label: attribute.");
  _e977.appendChild(_e979);
  _e964.appendChild(_e977);
  _e790.appendChild(_e964);
  const _e980 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e980);
  const _e981 = WF.h("hr", { className: "wf-divider" });
  _e790.appendChild(_e981);
  const _e982 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e982);
  const _e983 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Media and Typography");
  _e790.appendChild(_e983);
  const _e984 = WF.h("table", { className: "wf-table" });
  const _e985 = WF.h("thead", {});
  const _e986 = WF.h("td", {}, "Component");
  _e985.appendChild(_e986);
  const _e987 = WF.h("td", {}, "Description");
  _e985.appendChild(_e987);
  _e984.appendChild(_e985);
  const _e988 = WF.h("tr", {});
  const _e989 = WF.h("td", {}, "Image");
  _e988.appendChild(_e989);
  const _e990 = WF.h("td", {}, "Requires src: and alt: attributes.");
  _e988.appendChild(_e990);
  _e984.appendChild(_e988);
  const _e991 = WF.h("tr", {});
  const _e992 = WF.h("td", {}, "Video");
  _e991.appendChild(_e992);
  const _e993 = WF.h("td", {}, "Video player. Use controls: true.");
  _e991.appendChild(_e993);
  _e984.appendChild(_e991);
  const _e994 = WF.h("tr", {});
  const _e995 = WF.h("td", {}, "Icon");
  _e994.appendChild(_e995);
  const _e996 = WF.h("td", {}, "Icon from built-in set.");
  _e994.appendChild(_e996);
  _e984.appendChild(_e994);
  const _e997 = WF.h("tr", {});
  const _e998 = WF.h("td", {}, "Text");
  _e997.appendChild(_e998);
  const _e999 = WF.h("td", {}, "Paragraph text. Modifiers: bold, muted, primary, center, small, large.");
  _e997.appendChild(_e999);
  _e984.appendChild(_e997);
  const _e1000 = WF.h("tr", {});
  const _e1001 = WF.h("td", {}, "Heading");
  _e1000.appendChild(_e1001);
  const _e1002 = WF.h("td", {}, "Heading with level modifiers: h1, h2, h3, h4, h5, h6.");
  _e1000.appendChild(_e1002);
  _e984.appendChild(_e1000);
  const _e1003 = WF.h("tr", {});
  const _e1004 = WF.h("td", {}, "Code");
  _e1003.appendChild(_e1004);
  const _e1005 = WF.h("td", {}, "Code text. Use block modifier for block-level display.");
  _e1003.appendChild(_e1005);
  _e984.appendChild(_e1003);
  const _e1006 = WF.h("tr", {});
  const _e1007 = WF.h("td", {}, "Blockquote");
  _e1006.appendChild(_e1007);
  const _e1008 = WF.h("td", {}, "Block quotation with left border.");
  _e1006.appendChild(_e1008);
  _e984.appendChild(_e1006);
  _e790.appendChild(_e984);
  const _e1009 = WF.h("div", { className: "wf-spacer" });
  _e790.appendChild(_e1009);
  _root.appendChild(_e790);
  return _root;
}

function Page_Accessibility(params) {
  const _root = document.createDocumentFragment();
  const _e1010 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1011 = WF.h("div", { className: "wf-spacer" });
  _e1010.appendChild(_e1011);
  const _e1012 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Accessibility Linting");
  _e1010.appendChild(_e1012);
  const _e1013 = WF.h("p", { className: "wf-text wf-text--muted" }, "WebFluent checks your code for accessibility issues at compile time. Warnings are printed during build but never block compilation.");
  _e1010.appendChild(_e1013);
  const _e1014 = WF.h("div", { className: "wf-spacer" });
  _e1010.appendChild(_e1014);
  const _e1015 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "How It Works");
  _e1010.appendChild(_e1015);
  const _e1016 = WF.h("p", { className: "wf-text" }, "The linter runs automatically after parsing, before code generation. It walks the AST and checks each component against 12 rules.");
  _e1010.appendChild(_e1016);
  const _e1017 = WF.h("div", { className: "wf-spacer" });
  _e1010.appendChild(_e1017);
  const _e1018 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1019 = WF.h("div", { className: "wf-card__body" });
  const _e1020 = WF.h("code", { className: "wf-code wf-code--block" }, "$ wf build\nBuilding my-app...\n  Warning [A01]: Image missing \"alt\" attribute at src/pages/Home.wf:12:5\n    Add alt text: Image(src: \"...\", alt: \"Description of image\")\n  Warning [A03]: Input missing \"label\" attribute at src/pages/Form.wf:8:9\n    Add a label: Input(text, label: \"Username\")\n  3 pages, 2 components, 1 stores\n  Build complete with 2 accessibility warning(s).");
  _e1019.appendChild(_e1020);
  _e1018.appendChild(_e1019);
  _e1010.appendChild(_e1018);
  const _e1021 = WF.h("div", { className: "wf-spacer" });
  _e1010.appendChild(_e1021);
  const _e1022 = WF.h("hr", { className: "wf-divider" });
  _e1010.appendChild(_e1022);
  const _e1023 = WF.h("div", { className: "wf-spacer" });
  _e1010.appendChild(_e1023);
  const _e1024 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Lint Rules");
  _e1010.appendChild(_e1024);
  const _e1025 = WF.h("table", { className: "wf-table" });
  const _e1026 = WF.h("thead", {});
  const _e1027 = WF.h("td", {}, "Rule");
  _e1026.appendChild(_e1027);
  const _e1028 = WF.h("td", {}, "Component");
  _e1026.appendChild(_e1028);
  const _e1029 = WF.h("td", {}, "Check");
  _e1026.appendChild(_e1029);
  _e1025.appendChild(_e1026);
  const _e1030 = WF.h("tr", {});
  const _e1031 = WF.h("td", {}, "A01");
  _e1030.appendChild(_e1031);
  const _e1032 = WF.h("td", {}, "Image");
  _e1030.appendChild(_e1032);
  const _e1033 = WF.h("td", {}, "Must have alt attribute");
  _e1030.appendChild(_e1033);
  _e1025.appendChild(_e1030);
  const _e1034 = WF.h("tr", {});
  const _e1035 = WF.h("td", {}, "A02");
  _e1034.appendChild(_e1035);
  const _e1036 = WF.h("td", {}, "IconButton");
  _e1034.appendChild(_e1036);
  const _e1037 = WF.h("td", {}, "Must have label attribute (no visible text)");
  _e1034.appendChild(_e1037);
  _e1025.appendChild(_e1034);
  const _e1038 = WF.h("tr", {});
  const _e1039 = WF.h("td", {}, "A03");
  _e1038.appendChild(_e1039);
  const _e1040 = WF.h("td", {}, "Input");
  _e1038.appendChild(_e1040);
  const _e1041 = WF.h("td", {}, "Must have label or placeholder");
  _e1038.appendChild(_e1041);
  _e1025.appendChild(_e1038);
  const _e1042 = WF.h("tr", {});
  const _e1043 = WF.h("td", {}, "A04");
  _e1042.appendChild(_e1043);
  const _e1044 = WF.h("td", {}, "Checkbox, Radio, Switch, Slider");
  _e1042.appendChild(_e1044);
  const _e1045 = WF.h("td", {}, "Must have label attribute");
  _e1042.appendChild(_e1045);
  _e1025.appendChild(_e1042);
  const _e1046 = WF.h("tr", {});
  const _e1047 = WF.h("td", {}, "A05");
  _e1046.appendChild(_e1047);
  const _e1048 = WF.h("td", {}, "Button");
  _e1046.appendChild(_e1048);
  const _e1049 = WF.h("td", {}, "Must have text content");
  _e1046.appendChild(_e1049);
  _e1025.appendChild(_e1046);
  const _e1050 = WF.h("tr", {});
  const _e1051 = WF.h("td", {}, "A06");
  _e1050.appendChild(_e1051);
  const _e1052 = WF.h("td", {}, "Link");
  _e1050.appendChild(_e1052);
  const _e1053 = WF.h("td", {}, "Must have text content or children");
  _e1050.appendChild(_e1053);
  _e1025.appendChild(_e1050);
  const _e1054 = WF.h("tr", {});
  const _e1055 = WF.h("td", {}, "A07");
  _e1054.appendChild(_e1055);
  const _e1056 = WF.h("td", {}, "Heading");
  _e1054.appendChild(_e1056);
  const _e1057 = WF.h("td", {}, "Must not be empty");
  _e1054.appendChild(_e1057);
  _e1025.appendChild(_e1054);
  const _e1058 = WF.h("tr", {});
  const _e1059 = WF.h("td", {}, "A08");
  _e1058.appendChild(_e1059);
  const _e1060 = WF.h("td", {}, "Modal, Dialog");
  _e1058.appendChild(_e1060);
  const _e1061 = WF.h("td", {}, "Must have title attribute");
  _e1058.appendChild(_e1061);
  _e1025.appendChild(_e1058);
  const _e1062 = WF.h("tr", {});
  const _e1063 = WF.h("td", {}, "A09");
  _e1062.appendChild(_e1063);
  const _e1064 = WF.h("td", {}, "Video");
  _e1062.appendChild(_e1064);
  const _e1065 = WF.h("td", {}, "Must have controls attribute");
  _e1062.appendChild(_e1065);
  _e1025.appendChild(_e1062);
  const _e1066 = WF.h("tr", {});
  const _e1067 = WF.h("td", {}, "A10");
  _e1066.appendChild(_e1067);
  const _e1068 = WF.h("td", {}, "Table");
  _e1066.appendChild(_e1068);
  const _e1069 = WF.h("td", {}, "Must have Thead header row");
  _e1066.appendChild(_e1069);
  _e1025.appendChild(_e1066);
  const _e1070 = WF.h("tr", {});
  const _e1071 = WF.h("td", {}, "A11");
  _e1070.appendChild(_e1071);
  const _e1072 = WF.h("td", {}, "Heading");
  _e1070.appendChild(_e1072);
  const _e1073 = WF.h("td", {}, "Levels must not skip (h1 to h3)");
  _e1070.appendChild(_e1073);
  _e1025.appendChild(_e1070);
  const _e1074 = WF.h("tr", {});
  const _e1075 = WF.h("td", {}, "A12");
  _e1074.appendChild(_e1075);
  const _e1076 = WF.h("td", {}, "Page");
  _e1074.appendChild(_e1076);
  const _e1077 = WF.h("td", {}, "Must have exactly one h1");
  _e1074.appendChild(_e1077);
  _e1025.appendChild(_e1074);
  _e1010.appendChild(_e1025);
  const _e1078 = WF.h("div", { className: "wf-spacer" });
  _e1010.appendChild(_e1078);
  const _e1079 = WF.h("hr", { className: "wf-divider" });
  _e1010.appendChild(_e1079);
  const _e1080 = WF.h("div", { className: "wf-spacer" });
  _e1010.appendChild(_e1080);
  const _e1081 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Examples");
  _e1010.appendChild(_e1081);
  const _e1082 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1083 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1084 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1085 = WF.h("div", { className: "wf-card__body" });
  const _e1086 = WF.h("p", { className: "wf-text wf-text--danger wf-text--bold" }, "Bad (triggers warning)");
  _e1085.appendChild(_e1086);
  const _e1087 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\")\nIconButton(icon: \"close\")\nInput(text)\nCheckbox(bind: agreed)\nButton()");
  _e1085.appendChild(_e1087);
  _e1084.appendChild(_e1085);
  _e1083.appendChild(_e1084);
  _e1082.appendChild(_e1083);
  const _e1088 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1089 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1090 = WF.h("div", { className: "wf-card__body" });
  const _e1091 = WF.h("p", { className: "wf-text wf-text--success wf-text--bold" }, "Good (no warnings)");
  _e1090.appendChild(_e1091);
  const _e1092 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\", alt: \"Team photo\")\nIconButton(icon: \"close\", label: \"Close\")\nInput(text, label: \"Username\")\nCheckbox(bind: agreed, label: \"I agree\")\nButton(\"Save\")");
  _e1090.appendChild(_e1092);
  _e1089.appendChild(_e1090);
  _e1088.appendChild(_e1089);
  _e1082.appendChild(_e1088);
  _e1010.appendChild(_e1082);
  const _e1093 = WF.h("div", { className: "wf-spacer" });
  _e1010.appendChild(_e1093);
  _root.appendChild(_e1010);
  return _root;
}

(function() {
  const _app = document.getElementById('app');
  _app.innerHTML = '';
  const _e1094 = Component_NavBar({});
  _app.appendChild(_e1094);
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
  const _e1095 = Component_SiteFooter({});
  _app.appendChild(_e1095);
})();
