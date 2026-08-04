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
        __reg(key, s); // expose for WF.__debug.state()
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
    h, text, reactiveText, appendChildren,
    condRender, listRender, showRender,
    animateIn, animateOut, animateEl, replayAnimation,
    createRouter, navigate, getParams,
    createStore,
    createI18n,
    wfFetch, showToast,
    mount, hydrate, setSsgMode, setBasePath,
    bindDialog, bindPopup, tablist, mainOf, offCanvas,
    __debug, __reg,
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

function Component_FeatureCard({ title, description }) {
  const _frag = document.createDocumentFragment();
  const _e0 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-scaleIn" });
  const _e1 = WF.h("div", { className: "wf-card__body" });
  const _e2 = WF.h("h2", { className: "wf-heading" }, title);
  _e1.appendChild(_e2);
  const _e3 = WF.h("div", { className: "wf-spacer" });
  _e1.appendChild(_e3);
  const _e4 = WF.h("p", { className: "wf-text wf-text--muted" }, description);
  _e1.appendChild(_e4);
  _e0.appendChild(_e1);
  _frag.appendChild(_e0);
  return _frag;
}

function Component_NavBar() {
  const _frag = document.createDocumentFragment();
  const _e5 = WF.h("nav", { className: "wf-navbar", "aria-label": "Main" });
  const _e6 = WF.h("div", { className: "wf-navbar__brand" });
  const _e7 = WF.h("a", { className: "wf-link", href: WF._basePath + "/" });
  const _e8 = WF.h("p", { className: "wf-text wf-text--heading" }, "WebFluent");
  _e7.appendChild(_e8);
  _e6.appendChild(_e7);
  _e5.appendChild(_e6);
  const _e9 = WF.h("div", { className: "wf-navbar__links" });
  _e5.appendChild(_e9);
  const _e10 = WF.h("div", { className: "wf-navbar__actions" });
  const _e11 = WF.h("button", { className: "wf-btn wf-btn--small", "on:click": (e) => { WF.i18n.setLocale("en"); } }, "EN");
  _e10.appendChild(_e11);
  const _e12 = WF.h("button", { className: "wf-btn wf-btn--small", "on:click": (e) => { WF.i18n.setLocale("ar"); } }, "AR");
  _e10.appendChild(_e12);
  _e5.appendChild(_e10);
  const _e13 = WF.h("button", { className: "wf-navbar__toggle",                          type: "button", "aria-label": "Menu", "aria-expanded": "false" }, "\u2630");
  _e5.appendChild(_e13);
  WF.offCanvas(_e5, _e13, null);
  _frag.appendChild(_e5);
  return _frag;
}

function Component_SiteFooter() {
  const _frag = document.createDocumentFragment();
  const _e14 = WF.h("hr", { className: "wf-divider" });
  _frag.appendChild(_e14);
  const _e15 = WF.h("div", { className: "wf-container" });
  const _e16 = WF.h("div", { className: "wf-spacer" });
  _e15.appendChild(_e16);
  const _e17 = WF.h("div", { className: "wf-row wf-row--center wf-row--between" });
  const _e18 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("footer.built"));
  _e17.appendChild(_e18);
  const _e19 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e20 = WF.h("a", { className: "wf-link", href: WF._basePath + "/" });
  const _e21 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("nav.home"));
  _e20.appendChild(_e21);
  _e19.appendChild(_e20);
  const _e22 = WF.h("a", { className: "wf-link", href: WF._basePath + "/getting-started" });
  const _e23 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("footer.docs"));
  _e22.appendChild(_e23);
  _e19.appendChild(_e22);
  _e17.appendChild(_e19);
  _e15.appendChild(_e17);
  const _e24 = WF.h("div", { className: "wf-spacer" });
  _e15.appendChild(_e24);
  _frag.appendChild(_e15);
  return _frag;
}

function Component_CodeBlock({ code }) {
  const _frag = document.createDocumentFragment();
  const _e25 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e26 = WF.h("div", { className: "wf-card__body" });
  const _e27 = WF.h("code", { className: "wf-code wf-code--block" }, code);
  _e26.appendChild(_e27);
  _e25.appendChild(_e26);
  _frag.appendChild(_e25);
  return _frag;
}

function Component_DocSidebar() {
  const _frag = document.createDocumentFragment();
  const _e28 = WF.h("aside", { className: "wf-sidebar", id: "wf-sidebar-28" });
  const _e29 = WF.h("div", { className: "wf-sidebar__header" });
  const _e30 = WF.h("a", { className: "wf-link", href: WF._basePath + "/" });
  const _e31 = WF.h("p", { className: "wf-text wf-text--heading" }, "WebFluent");
  _e30.appendChild(_e31);
  _e29.appendChild(_e30);
  _e28.appendChild(_e29);
  const _e32 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small wf-text--bold wf-text--uppercase" }, () => WF.i18n.t("nav.section.intro"));
  _e28.appendChild(_e32);
  const _e33 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/" });
  _e33.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "home" }));
  const _e34 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.home"));
  _e33.appendChild(_e34);
  _e28.appendChild(_e33);
  const _e35 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/getting-started" });
  _e35.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "arrow-right" }));
  const _e36 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.start"));
  _e35.appendChild(_e36);
  _e28.appendChild(_e35);
  const _e37 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/guide" });
  _e37.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "info" }));
  const _e38 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.guide"));
  _e37.appendChild(_e38);
  _e28.appendChild(_e37);
  _e28.appendChild(WF.h("div", { className: "wf-sidebar__divider" }));
  const _e39 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small wf-text--bold wf-text--uppercase" }, () => WF.i18n.t("nav.section.features"));
  _e28.appendChild(_e39);
  const _e40 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/components" });
  _e40.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "filter" }));
  const _e41 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.components"));
  _e40.appendChild(_e41);
  _e28.appendChild(_e40);
  const _e42 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/styling" });
  _e42.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "eye" }));
  const _e43 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.styling"));
  _e42.appendChild(_e43);
  _e28.appendChild(_e42);
  const _e44 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/animation" });
  _e44.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "star" }));
  const _e45 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.animation"));
  _e44.appendChild(_e45);
  _e28.appendChild(_e44);
  const _e46 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/i18n" });
  _e46.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "link" }));
  const _e47 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.i18n"));
  _e46.appendChild(_e47);
  _e28.appendChild(_e46);
  const _e48 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/ssg" });
  _e48.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "download" }));
  const _e49 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.ssg"));
  _e48.appendChild(_e49);
  _e28.appendChild(_e48);
  const _e50 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/pdf" });
  _e50.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "copy" }));
  const _e51 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.pdf"));
  _e50.appendChild(_e51);
  _e28.appendChild(_e50);
  const _e52 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/template-engine" });
  _e52.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "settings" }));
  const _e53 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.template"));
  _e52.appendChild(_e53);
  _e28.appendChild(_e52);
  _e28.appendChild(WF.h("div", { className: "wf-sidebar__divider" }));
  const _e54 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small wf-text--bold wf-text--uppercase" }, () => WF.i18n.t("nav.section.tools"));
  _e28.appendChild(_e54);
  const _e55 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/accessibility" });
  _e55.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "check" }));
  const _e56 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.a11y"));
  _e55.appendChild(_e56);
  _e28.appendChild(_e55);
  const _e57 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/cli" });
  _e57.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "chevron-right" }));
  const _e58 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.cli"));
  _e57.appendChild(_e58);
  _e28.appendChild(_e57);
  _frag.appendChild(_e28);
  const _e59 = WF.h("div", { className: "wf-sidebar__scrim", hidden: true });
  const _e60 = WF.h("button", { className: "wf-sidebar__toggle", type: "button", "aria-label": "Open navigation", "aria-expanded": "false", "aria-controls": "wf-sidebar-28" }, "\u2630");
  _frag.appendChild(_e59);
  _frag.appendChild(_e60);
  WF.offCanvas(_e28, _e60, _e59);
  return _frag;
}

function Page_Pdf(params) {
  const _root = document.createDocumentFragment();
  const _e61 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e62 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e62);
  const _e63 = WF.h("h1", { className: "wf-heading" }, "PDF Generation");
  _e61.appendChild(_e63);
  const _e64 = WF.h("p", { className: "wf-text wf-text--muted" }, "Generate PDF documents directly from .wf source files. No external dependencies — raw PDF 1.7 output.");
  _e61.appendChild(_e64);
  const _e65 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e65);
  const _e66 = WF.h("h2", { className: "wf-heading" }, "Enable PDF Output");
  _e61.appendChild(_e66);
  const _e67 = WF.h("p", { className: "wf-text" }, "Set the output type to pdf in your project config.");
  _e61.appendChild(_e67);
  const _e68 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e68);
  const _e69 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e70 = WF.h("div", { className: "wf-card__body" });
  const _e71 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"build\": {\n    \"output_type\": \"pdf\",\n    \"pdf\": {\n      \"page_size\": \"A4\",\n      \"margins\": { \"top\": 72, \"bottom\": 72, \"left\": 72, \"right\": 72 },\n      \"default_font\": \"Helvetica\",\n      \"default_font_size\": 12,\n      \"output_filename\": \"report.pdf\"\n    }\n  }\n}");
  _e70.appendChild(_e71);
  _e69.appendChild(_e70);
  _e61.appendChild(_e69);
  const _e72 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e72);
  const _e73 = WF.h("hr", { className: "wf-divider" });
  _e61.appendChild(_e73);
  const _e74 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e74);
  const _e75 = WF.h("h2", { className: "wf-heading" }, "Quick Start");
  _e61.appendChild(_e75);
  const _e76 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e77 = WF.h("div", { className: "wf-card__body" });
  const _e78 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-report --template pdf\ncd my-report\nwf build");
  _e77.appendChild(_e78);
  _e76.appendChild(_e77);
  _e61.appendChild(_e76);
  const _e79 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e79);
  const _e80 = WF.h("p", { className: "wf-text wf-text--muted" }, "This creates a sample PDF project and builds it to build/my-report.pdf.");
  _e61.appendChild(_e80);
  const _e81 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e81);
  const _e82 = WF.h("hr", { className: "wf-divider" });
  _e61.appendChild(_e82);
  const _e83 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e83);
  const _e84 = WF.h("h2", { className: "wf-heading" }, "Document Structure");
  _e61.appendChild(_e84);
  const _e85 = WF.h("p", { className: "wf-text" }, "PDF documents use the same .wf syntax. Wrap content in a Document element with optional Header and Footer.");
  _e61.appendChild(_e85);
  const _e86 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e86);
  const _e87 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e88 = WF.h("div", { className: "wf-card__body" });
  const _e89 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Report (path: \"/\", title: \"Q1 Report\") {\n    Document(page_size: \"A4\") {\n        Header {\n            Text(\"Company Inc.\", muted, small, right)\n        }\n\n        Footer {\n            Text(\"Confidential\", muted, small, center)\n        }\n\n        Section {\n            Heading(\"Quarterly Report\", h1)\n            Text(\"Revenue grew 15% this quarter.\")\n\n            Table {\n                Thead {\n                    Trow {\n                        Tcell(\"Region\")\n                        Tcell(\"Revenue\")\n                    }\n                }\n                Tbody {\n                    Trow {\n                        Tcell(\"North America\")\n                        Tcell(\"$2.4M\")\n                    }\n                }\n            }\n\n            PageBreak()\n\n            Heading(\"Key Highlights\", h2)\n            List {\n                Text(\"Launched 3 new products\")\n                Text(\"Expanded to 5 new markets\")\n            }\n        }\n    }\n}");
  _e88.appendChild(_e89);
  _e87.appendChild(_e88);
  _e61.appendChild(_e87);
  const _e90 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e90);
  const _e91 = WF.h("hr", { className: "wf-divider" });
  _e61.appendChild(_e91);
  const _e92 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e92);
  const _e93 = WF.h("h2", { className: "wf-heading" }, "Supported Components");
  _e61.appendChild(_e93);
  const _e94 = WF.h("p", { className: "wf-text" }, "These components render in PDF output:");
  _e61.appendChild(_e94);
  const _e95 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e95);
  const _e96 = WF.h("table", { className: "wf-table" });
  const _e97 = WF.h("thead", {});
  const _e98 = WF.h("td", {}, "Component");
  _e97.appendChild(_e98);
  const _e99 = WF.h("td", {}, "PDF Behavior");
  _e97.appendChild(_e99);
  _e96.appendChild(_e97);
  const _e100 = WF.h("tr", {});
  const _e101 = WF.h("td", {}, "Document");
  _e100.appendChild(_e101);
  const _e102 = WF.h("td", {}, "Root element. Sets page size via page_size arg.");
  _e100.appendChild(_e102);
  _e96.appendChild(_e100);
  const _e103 = WF.h("tr", {});
  const _e104 = WF.h("td", {}, "Header / Footer");
  _e103.appendChild(_e104);
  const _e105 = WF.h("td", {}, "Repeated on every page. Positioned in margins.");
  _e103.appendChild(_e105);
  _e96.appendChild(_e103);
  const _e106 = WF.h("tr", {});
  const _e107 = WF.h("td", {}, "Section");
  _e106.appendChild(_e107);
  const _e108 = WF.h("td", {}, "Groups content with spacing.");
  _e106.appendChild(_e108);
  _e96.appendChild(_e106);
  const _e109 = WF.h("tr", {});
  const _e110 = WF.h("td", {}, "Paragraph");
  _e109.appendChild(_e110);
  const _e111 = WF.h("td", {}, "Block of text with paragraph spacing.");
  _e109.appendChild(_e111);
  _e96.appendChild(_e109);
  const _e112 = WF.h("tr", {});
  const _e113 = WF.h("td", {}, "PageBreak");
  _e112.appendChild(_e113);
  const _e114 = WF.h("td", {}, "Forces a new page.");
  _e112.appendChild(_e114);
  _e96.appendChild(_e112);
  const _e115 = WF.h("tr", {});
  const _e116 = WF.h("td", {}, "Heading(text, h1..h6)");
  _e115.appendChild(_e116);
  const _e117 = WF.h("td", {}, "Bold heading. h1=28pt, h2=22pt, h3=18pt...");
  _e115.appendChild(_e117);
  _e96.appendChild(_e115);
  const _e118 = WF.h("tr", {});
  const _e119 = WF.h("td", {}, "Text(text)");
  _e118.appendChild(_e119);
  const _e120 = WF.h("td", {}, "Body text with word wrapping.");
  _e118.appendChild(_e120);
  _e96.appendChild(_e118);
  const _e121 = WF.h("tr", {});
  const _e122 = WF.h("td", {}, "Table / Thead / Tbody / Trow / Tcell");
  _e121.appendChild(_e122);
  const _e123 = WF.h("td", {}, "Gridded table with borders and header styling.");
  _e121.appendChild(_e123);
  _e96.appendChild(_e121);
  const _e124 = WF.h("tr", {});
  const _e125 = WF.h("td", {}, "List");
  _e124.appendChild(_e125);
  const _e126 = WF.h("td", {}, "Bulleted list. Add ordered modifier for numbered.");
  _e124.appendChild(_e126);
  _e96.appendChild(_e124);
  const _e127 = WF.h("tr", {});
  const _e128 = WF.h("td", {}, "Code(text, block)");
  _e127.appendChild(_e128);
  const _e129 = WF.h("td", {}, "Monospace code with gray background.");
  _e127.appendChild(_e129);
  _e96.appendChild(_e127);
  const _e130 = WF.h("tr", {});
  const _e131 = WF.h("td", {}, "Blockquote");
  _e130.appendChild(_e131);
  const _e132 = WF.h("td", {}, "Indented text with left bar.");
  _e130.appendChild(_e132);
  _e96.appendChild(_e130);
  const _e133 = WF.h("tr", {});
  const _e134 = WF.h("td", {}, "Divider");
  _e133.appendChild(_e134);
  const _e135 = WF.h("td", {}, "Horizontal line.");
  _e133.appendChild(_e135);
  _e96.appendChild(_e133);
  const _e136 = WF.h("tr", {});
  const _e137 = WF.h("td", {}, "Alert(text, variant)");
  _e136.appendChild(_e137);
  const _e138 = WF.h("td", {}, "Colored box with left accent bar.");
  _e136.appendChild(_e138);
  _e96.appendChild(_e136);
  const _e139 = WF.h("tr", {});
  const _e140 = WF.h("td", {}, "Badge / Tag");
  _e139.appendChild(_e140);
  const _e141 = WF.h("td", {}, "Colored pill with white text.");
  _e139.appendChild(_e141);
  _e96.appendChild(_e139);
  const _e142 = WF.h("tr", {});
  const _e143 = WF.h("td", {}, "Progress(value, max)");
  _e142.appendChild(_e143);
  const _e144 = WF.h("td", {}, "Horizontal bar.");
  _e142.appendChild(_e144);
  _e96.appendChild(_e142);
  const _e145 = WF.h("tr", {});
  const _e146 = WF.h("td", {}, "Card");
  _e145.appendChild(_e146);
  const _e147 = WF.h("td", {}, "Bordered box around children.");
  _e145.appendChild(_e147);
  _e96.appendChild(_e145);
  const _e148 = WF.h("tr", {});
  const _e149 = WF.h("td", {}, "Image(src)");
  _e148.appendChild(_e149);
  const _e150 = WF.h("td", {}, "Placeholder rectangle (JPEG planned).");
  _e148.appendChild(_e150);
  _e96.appendChild(_e148);
  const _e151 = WF.h("tr", {});
  const _e152 = WF.h("td", {}, "Spacer");
  _e151.appendChild(_e152);
  const _e153 = WF.h("td", {}, "Vertical space. Modifiers: sm, md, lg, xl.");
  _e151.appendChild(_e153);
  _e96.appendChild(_e151);
  _e61.appendChild(_e96);
  const _e154 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e154);
  const _e155 = WF.h("hr", { className: "wf-divider" });
  _e61.appendChild(_e155);
  const _e156 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e156);
  const _e157 = WF.h("h2", { className: "wf-heading" }, "Rejected Components");
  _e61.appendChild(_e157);
  const _e158 = WF.h("p", { className: "wf-text" }, "Interactive and web-only components cause compile-time errors in PDF mode:");
  _e61.appendChild(_e158);
  const _e159 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e159);
  const _e160 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e161 = WF.h("div", { className: "wf-card__body" });
  const _e162 = WF.h("code", { className: "wf-code wf-code--block" }, "error[pdf]: 'Button' cannot be used in PDF output (Page Report)\n  — interactive elements are not supported in PDF\n\nerror[pdf]: 'Input' cannot be used in PDF output (Page Report)\n  — interactive elements are not supported in PDF");
  _e161.appendChild(_e162);
  _e160.appendChild(_e161);
  _e61.appendChild(_e160);
  const _e163 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e163);
  const _e164 = WF.h("p", { className: "wf-text wf-text--muted" }, "Rejected: Button, Input, Select, Checkbox, Switch, Slider, Form, Modal, Dialog, Toast, Router, Navbar, Sidebar, Tabs, Video, Carousel, and all event handlers.");
  _e61.appendChild(_e164);
  const _e165 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e165);
  const _e166 = WF.h("hr", { className: "wf-divider" });
  _e61.appendChild(_e166);
  const _e167 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e167);
  const _e168 = WF.h("h2", { className: "wf-heading" }, "Page Sizes");
  _e61.appendChild(_e168);
  const _e169 = WF.h("table", { className: "wf-table" });
  const _e170 = WF.h("thead", {});
  const _e171 = WF.h("td", {}, "Value");
  _e170.appendChild(_e171);
  const _e172 = WF.h("td", {}, "Dimensions (points)");
  _e170.appendChild(_e172);
  const _e173 = WF.h("td", {}, "Dimensions (mm)");
  _e170.appendChild(_e173);
  _e169.appendChild(_e170);
  const _e174 = WF.h("tr", {});
  const _e175 = WF.h("td", {}, "A4");
  _e174.appendChild(_e175);
  const _e176 = WF.h("td", {}, "595 x 842");
  _e174.appendChild(_e176);
  const _e177 = WF.h("td", {}, "210 x 297");
  _e174.appendChild(_e177);
  _e169.appendChild(_e174);
  const _e178 = WF.h("tr", {});
  const _e179 = WF.h("td", {}, "A3");
  _e178.appendChild(_e179);
  const _e180 = WF.h("td", {}, "842 x 1191");
  _e178.appendChild(_e180);
  const _e181 = WF.h("td", {}, "297 x 420");
  _e178.appendChild(_e181);
  _e169.appendChild(_e178);
  const _e182 = WF.h("tr", {});
  const _e183 = WF.h("td", {}, "A5");
  _e182.appendChild(_e183);
  const _e184 = WF.h("td", {}, "420 x 595");
  _e182.appendChild(_e184);
  const _e185 = WF.h("td", {}, "148 x 210");
  _e182.appendChild(_e185);
  _e169.appendChild(_e182);
  const _e186 = WF.h("tr", {});
  const _e187 = WF.h("td", {}, "Letter");
  _e186.appendChild(_e187);
  const _e188 = WF.h("td", {}, "612 x 792");
  _e186.appendChild(_e188);
  const _e189 = WF.h("td", {}, "216 x 279");
  _e186.appendChild(_e189);
  _e169.appendChild(_e186);
  const _e190 = WF.h("tr", {});
  const _e191 = WF.h("td", {}, "Legal");
  _e190.appendChild(_e191);
  const _e192 = WF.h("td", {}, "612 x 1008");
  _e190.appendChild(_e192);
  const _e193 = WF.h("td", {}, "216 x 356");
  _e190.appendChild(_e193);
  _e169.appendChild(_e190);
  _e61.appendChild(_e169);
  const _e194 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e194);
  const _e195 = WF.h("hr", { className: "wf-divider" });
  _e61.appendChild(_e195);
  const _e196 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e196);
  const _e197 = WF.h("h2", { className: "wf-heading" }, "Fonts");
  _e61.appendChild(_e197);
  const _e198 = WF.h("p", { className: "wf-text" }, "PDF output uses the 14 standard PDF base fonts. No embedding needed.");
  _e61.appendChild(_e198);
  const _e199 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e199);
  const _e200 = WF.h("table", { className: "wf-table" });
  const _e201 = WF.h("thead", {});
  const _e202 = WF.h("td", {}, "Font Family");
  _e201.appendChild(_e202);
  const _e203 = WF.h("td", {}, "Variants");
  _e201.appendChild(_e203);
  _e200.appendChild(_e201);
  const _e204 = WF.h("tr", {});
  const _e205 = WF.h("td", {}, "Helvetica");
  _e204.appendChild(_e205);
  const _e206 = WF.h("td", {}, "Regular, Bold, Oblique, BoldOblique");
  _e204.appendChild(_e206);
  _e200.appendChild(_e204);
  const _e207 = WF.h("tr", {});
  const _e208 = WF.h("td", {}, "Times");
  _e207.appendChild(_e208);
  const _e209 = WF.h("td", {}, "Roman, Bold, Italic, BoldItalic");
  _e207.appendChild(_e209);
  _e200.appendChild(_e207);
  const _e210 = WF.h("tr", {});
  const _e211 = WF.h("td", {}, "Courier");
  _e210.appendChild(_e211);
  const _e212 = WF.h("td", {}, "Regular, Bold, Oblique, BoldOblique");
  _e210.appendChild(_e212);
  _e200.appendChild(_e210);
  _e61.appendChild(_e200);
  const _e213 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e213);
  const _e214 = WF.h("p", { className: "wf-text wf-text--muted" }, "Set the default font in config or override per-element with style blocks:");
  _e61.appendChild(_e214);
  const _e215 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e215);
  const _e216 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e217 = WF.h("div", { className: "wf-card__body" });
  const _e218 = WF.h("code", { className: "wf-code wf-code--block" }, "Heading(\"Title\", h1) {\n    style {\n        font-family: \"Helvetica-Bold\"\n        color: \"#1a1a2e\"\n    }\n}");
  _e217.appendChild(_e218);
  _e216.appendChild(_e217);
  _e61.appendChild(_e216);
  const _e219 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e219);
  const _e220 = WF.h("hr", { className: "wf-divider" });
  _e61.appendChild(_e220);
  const _e221 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e221);
  const _e222 = WF.h("h2", { className: "wf-heading" }, "Styling in PDF");
  _e61.appendChild(_e222);
  const _e223 = WF.h("p", { className: "wf-text" }, "Style blocks support these properties in PDF output:");
  _e61.appendChild(_e223);
  const _e224 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e224);
  const _e225 = WF.h("table", { className: "wf-table" });
  const _e226 = WF.h("thead", {});
  const _e227 = WF.h("td", {}, "Property");
  _e226.appendChild(_e227);
  const _e228 = WF.h("td", {}, "Values");
  _e226.appendChild(_e228);
  const _e229 = WF.h("td", {}, "Example");
  _e226.appendChild(_e229);
  _e225.appendChild(_e226);
  const _e230 = WF.h("tr", {});
  const _e231 = WF.h("td", {}, "font-size");
  _e230.appendChild(_e231);
  const _e232 = WF.h("td", {}, "Number (points)");
  _e230.appendChild(_e232);
  const _e233 = WF.h("td", {}, "font-size: 14");
  _e230.appendChild(_e233);
  _e225.appendChild(_e230);
  const _e234 = WF.h("tr", {});
  const _e235 = WF.h("td", {}, "font-family");
  _e234.appendChild(_e235);
  const _e236 = WF.h("td", {}, "Base14 font name");
  _e234.appendChild(_e236);
  const _e237 = WF.h("td", {}, "font-family: \"Courier\"");
  _e234.appendChild(_e237);
  _e225.appendChild(_e234);
  const _e238 = WF.h("tr", {});
  const _e239 = WF.h("td", {}, "color");
  _e238.appendChild(_e239);
  const _e240 = WF.h("td", {}, "Hex color");
  _e238.appendChild(_e240);
  const _e241 = WF.h("td", {}, "color: \"#333333\"");
  _e238.appendChild(_e241);
  _e225.appendChild(_e238);
  const _e242 = WF.h("tr", {});
  const _e243 = WF.h("td", {}, "text-align");
  _e242.appendChild(_e243);
  const _e244 = WF.h("td", {}, "left, center, right");
  _e242.appendChild(_e244);
  const _e245 = WF.h("td", {}, "text-align: \"center\"");
  _e242.appendChild(_e245);
  _e225.appendChild(_e242);
  _e61.appendChild(_e225);
  const _e246 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e246);
  const _e247 = WF.h("p", { className: "wf-text wf-text--muted" }, "Modifiers also work: bold, muted, primary, danger, success, warning, info, small, large, center, right.");
  _e61.appendChild(_e247);
  const _e248 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e248);
  const _e249 = WF.h("hr", { className: "wf-divider" });
  _e61.appendChild(_e249);
  const _e250 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e250);
  const _e251 = WF.h("h2", { className: "wf-heading" }, "Auto Page Breaks");
  _e61.appendChild(_e251);
  const _e252 = WF.h("p", { className: "wf-text wf-text--muted" }, "Content automatically flows to a new page when it reaches the bottom margin. Headers and footers are rendered on every page, including auto-generated ones.");
  _e61.appendChild(_e252);
  const _e253 = WF.h("div", { className: "wf-spacer" });
  _e61.appendChild(_e253);
  _root.appendChild(_e61);
  return _root;
}

function Page_Ssg(params) {
  const _root = document.createDocumentFragment();
  const _e254 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e255 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e255);
  const _e256 = WF.h("h1", { className: "wf-heading" }, "Static Site Generation (SSG)");
  _e254.appendChild(_e256);
  const _e257 = WF.h("p", { className: "wf-text wf-text--muted" }, "Pre-render pages to HTML at build time for instant content visibility. JavaScript hydrates the page for interactivity.");
  _e254.appendChild(_e257);
  const _e258 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e258);
  const _e259 = WF.h("h2", { className: "wf-heading" }, "Enable SSG");
  _e254.appendChild(_e259);
  const _e260 = WF.h("p", { className: "wf-text" }, "One config flag is all you need.");
  _e254.appendChild(_e260);
  const _e261 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e261);
  const _e262 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e263 = WF.h("div", { className: "wf-card__body" });
  const _e264 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"build\": {\n    \"ssg\": true\n  }\n}");
  _e263.appendChild(_e264);
  _e262.appendChild(_e263);
  _e254.appendChild(_e262);
  const _e265 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e265);
  const _e266 = WF.h("hr", { className: "wf-divider" });
  _e254.appendChild(_e266);
  const _e267 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e267);
  const _e268 = WF.h("h2", { className: "wf-heading" }, "How It Works");
  _e254.appendChild(_e268);
  const _e269 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e270 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e271 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e272 = WF.h("div", { className: "wf-card__body" });
  const _e273 = WF.h("h2", { className: "wf-heading" }, "1. Build");
  _e272.appendChild(_e273);
  const _e274 = WF.h("p", { className: "wf-text wf-text--muted" }, "The compiler walks the AST for each page and generates static HTML from the component tree.");
  _e272.appendChild(_e274);
  _e271.appendChild(_e272);
  _e270.appendChild(_e271);
  _e269.appendChild(_e270);
  const _e275 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e276 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e277 = WF.h("div", { className: "wf-card__body" });
  const _e278 = WF.h("h2", { className: "wf-heading" }, "2. Serve");
  _e277.appendChild(_e278);
  const _e279 = WF.h("p", { className: "wf-text wf-text--muted" }, "The browser loads pre-rendered HTML. Content is visible immediately — no blank white screen.");
  _e277.appendChild(_e279);
  _e276.appendChild(_e277);
  _e275.appendChild(_e276);
  _e269.appendChild(_e275);
  const _e280 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e281 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e282 = WF.h("div", { className: "wf-card__body" });
  const _e283 = WF.h("h2", { className: "wf-heading" }, "3. Hydrate");
  _e282.appendChild(_e283);
  const _e284 = WF.h("p", { className: "wf-text wf-text--muted" }, "JavaScript runs and hydrates the page: attaches events, initializes state, fills dynamic content.");
  _e282.appendChild(_e284);
  _e281.appendChild(_e282);
  _e280.appendChild(_e281);
  _e269.appendChild(_e280);
  _e254.appendChild(_e269);
  const _e285 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e285);
  const _e286 = WF.h("hr", { className: "wf-divider" });
  _e254.appendChild(_e286);
  const _e287 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e287);
  const _e288 = WF.h("h2", { className: "wf-heading" }, "Build Output");
  _e254.appendChild(_e288);
  const _e289 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e290 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e291 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e292 = WF.h("div", { className: "wf-card__body" });
  const _e293 = WF.h("p", { className: "wf-text wf-text--bold" }, "SPA (default)");
  _e292.appendChild(_e293);
  const _e294 = WF.h("code", { className: "wf-code wf-code--block" }, "build/\n├── index.html       # Empty shell\n├── app.js\n└── styles.css");
  _e292.appendChild(_e294);
  _e291.appendChild(_e292);
  _e290.appendChild(_e291);
  _e289.appendChild(_e290);
  const _e295 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e296 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e297 = WF.h("div", { className: "wf-card__body" });
  const _e298 = WF.h("p", { className: "wf-text wf-text--bold" }, "SSG mode");
  _e297.appendChild(_e298);
  const _e299 = WF.h("code", { className: "wf-code wf-code--block" }, "build/\n├── index.html       # Pre-rendered /\n├── about/\n│   └── index.html   # Pre-rendered /about\n├── blog/\n│   └── index.html   # Pre-rendered /blog\n├── app.js\n└── styles.css");
  _e297.appendChild(_e299);
  _e296.appendChild(_e297);
  _e295.appendChild(_e296);
  _e289.appendChild(_e295);
  _e254.appendChild(_e289);
  const _e300 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e300);
  const _e301 = WF.h("hr", { className: "wf-divider" });
  _e254.appendChild(_e301);
  const _e302 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e302);
  const _e303 = WF.h("h2", { className: "wf-heading" }, "What Gets Pre-Rendered");
  _e254.appendChild(_e303);
  const _e304 = WF.h("table", { className: "wf-table" });
  const _e305 = WF.h("thead", {});
  const _e306 = WF.h("td", {}, "Element");
  _e305.appendChild(_e306);
  const _e307 = WF.h("td", {}, "SSG Behavior");
  _e305.appendChild(_e307);
  _e304.appendChild(_e305);
  const _e308 = WF.h("tr", {});
  const _e309 = WF.h("td", {}, "Static text, headings, components");
  _e308.appendChild(_e309);
  const _e310 = WF.h("td", {}, "Fully rendered to HTML");
  _e308.appendChild(_e310);
  _e304.appendChild(_e308);
  const _e311 = WF.h("tr", {});
  const _e312 = WF.h("td", {}, "Container, Row, Column, Card, etc.");
  _e311.appendChild(_e312);
  const _e313 = WF.h("td", {}, "Full HTML with classes");
  _e311.appendChild(_e313);
  _e304.appendChild(_e311);
  const _e314 = WF.h("tr", {});
  const _e315 = WF.h("td", {}, "Modifiers (primary, large, etc.)");
  _e314.appendChild(_e315);
  const _e316 = WF.h("td", {}, "CSS classes applied");
  _e314.appendChild(_e316);
  _e304.appendChild(_e314);
  const _e317 = WF.h("tr", {});
  const _e318 = WF.h("td", {}, "Animation modifiers (fadeIn, etc.)");
  _e317.appendChild(_e318);
  const _e319 = WF.h("td", {}, "Animation classes applied");
  _e317.appendChild(_e319);
  _e304.appendChild(_e317);
  const _e320 = WF.h("tr", {});
  const _e321 = WF.h("td", {}, "t() i18n calls");
  _e320.appendChild(_e321);
  const _e322 = WF.h("td", {}, "Default locale text rendered");
  _e320.appendChild(_e322);
  _e304.appendChild(_e320);
  const _e323 = WF.h("tr", {});
  const _e324 = WF.h("td", {}, "State-dependent text");
  _e323.appendChild(_e324);
  const _e325 = WF.h("td", {}, "Empty placeholder (filled by JS)");
  _e323.appendChild(_e325);
  _e304.appendChild(_e323);
  const _e326 = WF.h("tr", {});
  const _e327 = WF.h("td", {}, "if / for blocks");
  _e326.appendChild(_e327);
  const _e328 = WF.h("td", {}, "Comment placeholder (filled by JS)");
  _e326.appendChild(_e328);
  _e304.appendChild(_e326);
  const _e329 = WF.h("tr", {});
  const _e330 = WF.h("td", {}, "show blocks");
  _e329.appendChild(_e330);
  const _e331 = WF.h("td", {}, "Rendered but hidden (display:none)");
  _e329.appendChild(_e331);
  _e304.appendChild(_e329);
  const _e332 = WF.h("tr", {});
  const _e333 = WF.h("td", {}, "fetch blocks");
  _e332.appendChild(_e333);
  const _e334 = WF.h("td", {}, "Loading block if present, else placeholder");
  _e332.appendChild(_e334);
  _e304.appendChild(_e332);
  const _e335 = WF.h("tr", {});
  const _e336 = WF.h("td", {}, "Event handlers");
  _e335.appendChild(_e336);
  const _e337 = WF.h("td", {}, "Attached during hydration");
  _e335.appendChild(_e337);
  _e304.appendChild(_e335);
  _e254.appendChild(_e304);
  const _e338 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e338);
  const _e339 = WF.h("hr", { className: "wf-divider" });
  _e254.appendChild(_e339);
  const _e340 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e340);
  const _e341 = WF.h("h2", { className: "wf-heading" }, "Dynamic Routes");
  _e254.appendChild(_e341);
  const _e342 = WF.h("p", { className: "wf-text wf-text--muted" }, "Pages with :param segments (e.g., /user/:id) cannot be pre-rendered — they fall back to client-side rendering.");
  _e254.appendChild(_e342);
  const _e343 = WF.h("div", { className: "wf-spacer" });
  _e254.appendChild(_e343);
  _root.appendChild(_e254);
  return _root;
}

function Page_Styling(params) {
  const _root = document.createDocumentFragment();
  const _e344 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e345 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e345);
  const _e346 = WF.h("h1", { className: "wf-heading" }, "Design System & Styling");
  _e344.appendChild(_e346);
  const _e347 = WF.h("p", { className: "wf-text wf-text--muted" }, "Token-based design system. Every component uses design tokens for colors, spacing, typography. Change the entire look with a config update.");
  _e344.appendChild(_e347);
  const _e348 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e348);
  const _e349 = WF.h("h2", { className: "wf-heading" }, "Variant Modifiers");
  _e344.appendChild(_e349);
  const _e350 = WF.h("p", { className: "wf-text" }, "Apply common styles with modifier keywords.");
  _e344.appendChild(_e350);
  const _e351 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e351);
  const _e352 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e353 = WF.h("div", { className: "wf-card__header" });
  const _e354 = WF.h("p", { className: "wf-text wf-text--bold" }, "Size Modifiers");
  _e353.appendChild(_e354);
  _e352.appendChild(_e353);
  const _e355 = WF.h("div", { className: "wf-card__body" });
  const _e356 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e357 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Small");
  _e356.appendChild(_e357);
  const _e358 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Medium");
  _e356.appendChild(_e358);
  const _e359 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large" }, "Large");
  _e356.appendChild(_e359);
  _e355.appendChild(_e356);
  _e352.appendChild(_e355);
  _e344.appendChild(_e352);
  const _e360 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e360);
  const _e361 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e362 = WF.h("div", { className: "wf-card__header" });
  const _e363 = WF.h("p", { className: "wf-text wf-text--bold" }, "Color Modifiers");
  _e362.appendChild(_e363);
  _e361.appendChild(_e362);
  const _e364 = WF.h("div", { className: "wf-card__body" });
  const _e365 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e366 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e365.appendChild(_e366);
  const _e367 = WF.h("button", { className: "wf-btn wf-btn--secondary" }, "Secondary");
  _e365.appendChild(_e367);
  const _e368 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e365.appendChild(_e368);
  const _e369 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e365.appendChild(_e369);
  const _e370 = WF.h("button", { className: "wf-btn wf-btn--warning" }, "Warning");
  _e365.appendChild(_e370);
  const _e371 = WF.h("button", { className: "wf-btn wf-btn--info" }, "Info");
  _e365.appendChild(_e371);
  _e364.appendChild(_e365);
  const _e372 = WF.h("div", { className: "wf-spacer" });
  _e364.appendChild(_e372);
  const _e373 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e374 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Primary");
  _e373.appendChild(_e374);
  const _e375 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Success");
  _e373.appendChild(_e375);
  const _e376 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Danger");
  _e373.appendChild(_e376);
  const _e377 = WF.h("span", { className: "wf-badge wf-badge--warning" }, "Warning");
  _e373.appendChild(_e377);
  _e364.appendChild(_e373);
  _e361.appendChild(_e364);
  _e344.appendChild(_e361);
  const _e378 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e378);
  const _e379 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e380 = WF.h("div", { className: "wf-card__header" });
  const _e381 = WF.h("p", { className: "wf-text wf-text--bold" }, "Shape and Elevation");
  _e380.appendChild(_e381);
  _e379.appendChild(_e380);
  const _e382 = WF.h("div", { className: "wf-card__body" });
  const _e383 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e384 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Default");
  _e383.appendChild(_e384);
  const _e385 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--rounded" }, "Rounded");
  _e383.appendChild(_e385);
  const _e386 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--full" }, "Full Width");
  _e383.appendChild(_e386);
  _e382.appendChild(_e383);
  const _e387 = WF.h("div", { className: "wf-spacer" });
  _e382.appendChild(_e387);
  const _e388 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e389 = WF.h("div", { className: "wf-card" });
  const _e390 = WF.h("div", { className: "wf-card__body" });
  const _e391 = WF.h("p", { className: "wf-text" }, "Default");
  _e390.appendChild(_e391);
  _e389.appendChild(_e390);
  _e388.appendChild(_e389);
  const _e392 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e393 = WF.h("div", { className: "wf-card__body" });
  const _e394 = WF.h("p", { className: "wf-text" }, "Elevated");
  _e393.appendChild(_e394);
  _e392.appendChild(_e393);
  _e388.appendChild(_e392);
  const _e395 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e396 = WF.h("div", { className: "wf-card__body" });
  const _e397 = WF.h("p", { className: "wf-text" }, "Outlined");
  _e396.appendChild(_e397);
  _e395.appendChild(_e396);
  _e388.appendChild(_e395);
  _e382.appendChild(_e388);
  _e379.appendChild(_e382);
  _e344.appendChild(_e379);
  const _e398 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e398);
  const _e399 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e400 = WF.h("div", { className: "wf-card__header" });
  const _e401 = WF.h("p", { className: "wf-text wf-text--bold" }, "Text Modifiers");
  _e400.appendChild(_e401);
  _e399.appendChild(_e400);
  const _e402 = WF.h("div", { className: "wf-card__body" });
  const _e403 = WF.h("p", { className: "wf-text wf-text--bold" }, "Bold text.");
  _e402.appendChild(_e403);
  const _e404 = WF.h("p", { className: "wf-text wf-text--italic" }, "Italic text.");
  _e402.appendChild(_e404);
  const _e405 = WF.h("p", { className: "wf-text wf-text--uppercase" }, "Uppercase text.");
  _e402.appendChild(_e405);
  const _e406 = WF.h("p", { className: "wf-text wf-text--muted" }, "Muted text.");
  _e402.appendChild(_e406);
  const _e407 = WF.h("p", { className: "wf-text wf-text--primary" }, "Primary colored text.");
  _e402.appendChild(_e407);
  const _e408 = WF.h("p", { className: "wf-text wf-text--small" }, "Small text.");
  _e402.appendChild(_e408);
  const _e409 = WF.h("p", { className: "wf-text wf-text--large" }, "Large text.");
  _e402.appendChild(_e409);
  _e399.appendChild(_e402);
  _e344.appendChild(_e399);
  const _e410 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e410);
  const _e411 = WF.h("hr", { className: "wf-divider" });
  _e344.appendChild(_e411);
  const _e412 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e412);
  const _e413 = WF.h("h2", { className: "wf-heading" }, "Design Tokens");
  _e344.appendChild(_e413);
  const _e414 = WF.h("p", { className: "wf-text" }, "All styling is built on tokens — CSS custom properties. Override any token in your config.");
  _e344.appendChild(_e414);
  const _e415 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e415);
  const _e416 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e417 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e418 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e419 = WF.h("div", { className: "wf-card__header" });
  const _e420 = WF.h("p", { className: "wf-text wf-text--bold" }, "Colors");
  _e419.appendChild(_e420);
  _e418.appendChild(_e419);
  const _e421 = WF.h("div", { className: "wf-card__body" });
  const _e422 = WF.h("table", { className: "wf-table" });
  const _e423 = WF.h("thead", {});
  const _e424 = WF.h("td", {}, "Token");
  _e423.appendChild(_e424);
  const _e425 = WF.h("td", {}, "Value");
  _e423.appendChild(_e425);
  _e422.appendChild(_e423);
  const _e426 = WF.h("tr", {});
  const _e427 = WF.h("td", {}, "color-primary");
  _e426.appendChild(_e427);
  const _e428 = WF.h("td", {}, "#3B82F6");
  _e426.appendChild(_e428);
  _e422.appendChild(_e426);
  const _e429 = WF.h("tr", {});
  const _e430 = WF.h("td", {}, "color-success");
  _e429.appendChild(_e430);
  const _e431 = WF.h("td", {}, "#22C55E");
  _e429.appendChild(_e431);
  _e422.appendChild(_e429);
  const _e432 = WF.h("tr", {});
  const _e433 = WF.h("td", {}, "color-danger");
  _e432.appendChild(_e433);
  const _e434 = WF.h("td", {}, "#EF4444");
  _e432.appendChild(_e434);
  _e422.appendChild(_e432);
  const _e435 = WF.h("tr", {});
  const _e436 = WF.h("td", {}, "color-warning");
  _e435.appendChild(_e436);
  const _e437 = WF.h("td", {}, "#F59E0B");
  _e435.appendChild(_e437);
  _e422.appendChild(_e435);
  const _e438 = WF.h("tr", {});
  const _e439 = WF.h("td", {}, "color-text");
  _e438.appendChild(_e439);
  const _e440 = WF.h("td", {}, "#0F172A");
  _e438.appendChild(_e440);
  _e422.appendChild(_e438);
  const _e441 = WF.h("tr", {});
  const _e442 = WF.h("td", {}, "color-border");
  _e441.appendChild(_e442);
  const _e443 = WF.h("td", {}, "#E2E8F0");
  _e441.appendChild(_e443);
  _e422.appendChild(_e441);
  _e421.appendChild(_e422);
  _e418.appendChild(_e421);
  _e417.appendChild(_e418);
  _e416.appendChild(_e417);
  const _e444 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e445 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e446 = WF.h("div", { className: "wf-card__header" });
  const _e447 = WF.h("p", { className: "wf-text wf-text--bold" }, "Spacing and Radius");
  _e446.appendChild(_e447);
  _e445.appendChild(_e446);
  const _e448 = WF.h("div", { className: "wf-card__body" });
  const _e449 = WF.h("table", { className: "wf-table" });
  const _e450 = WF.h("thead", {});
  const _e451 = WF.h("td", {}, "Token");
  _e450.appendChild(_e451);
  const _e452 = WF.h("td", {}, "Value");
  _e450.appendChild(_e452);
  _e449.appendChild(_e450);
  const _e453 = WF.h("tr", {});
  const _e454 = WF.h("td", {}, "spacing-xs");
  _e453.appendChild(_e454);
  const _e455 = WF.h("td", {}, "0.25rem");
  _e453.appendChild(_e455);
  _e449.appendChild(_e453);
  const _e456 = WF.h("tr", {});
  const _e457 = WF.h("td", {}, "spacing-sm");
  _e456.appendChild(_e457);
  const _e458 = WF.h("td", {}, "0.5rem");
  _e456.appendChild(_e458);
  _e449.appendChild(_e456);
  const _e459 = WF.h("tr", {});
  const _e460 = WF.h("td", {}, "spacing-md");
  _e459.appendChild(_e460);
  const _e461 = WF.h("td", {}, "1rem");
  _e459.appendChild(_e461);
  _e449.appendChild(_e459);
  const _e462 = WF.h("tr", {});
  const _e463 = WF.h("td", {}, "spacing-lg");
  _e462.appendChild(_e463);
  const _e464 = WF.h("td", {}, "1.5rem");
  _e462.appendChild(_e464);
  _e449.appendChild(_e462);
  const _e465 = WF.h("tr", {});
  const _e466 = WF.h("td", {}, "radius-md");
  _e465.appendChild(_e466);
  const _e467 = WF.h("td", {}, "0.5rem");
  _e465.appendChild(_e467);
  _e449.appendChild(_e465);
  const _e468 = WF.h("tr", {});
  const _e469 = WF.h("td", {}, "radius-full");
  _e468.appendChild(_e469);
  const _e470 = WF.h("td", {}, "9999px");
  _e468.appendChild(_e470);
  _e449.appendChild(_e468);
  _e448.appendChild(_e449);
  _e445.appendChild(_e448);
  _e444.appendChild(_e445);
  _e416.appendChild(_e444);
  _e344.appendChild(_e416);
  const _e471 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e471);
  const _e472 = WF.h("hr", { className: "wf-divider" });
  _e344.appendChild(_e472);
  const _e473 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e473);
  const _e474 = WF.h("h2", { className: "wf-heading" }, "Themes");
  _e344.appendChild(_e474);
  const _e475 = WF.h("p", { className: "wf-text" }, "A theme is written in WebFluent, in your own source, next to the code it dresses. Declare one and every built-in component follows it.");
  _e344.appendChild(_e475);
  const _e476 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e476);
  const _e477 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e478 = WF.h("div", { className: "wf-card__body" });
  const _e479 = WF.h("code", { className: "wf-code wf-code--block" }, "Theme Ledger {\n    token color-primary: \"#0F766E\"\n    token color-secondary: \"#134E4A\"\n    token font-family: \"'Söhne', system-ui, sans-serif\"\n    token radius-md: \"14px\"\n}");
  _e478.appendChild(_e479);
  _e477.appendChild(_e478);
  _e344.appendChild(_e477);
  const _e480 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e480);
  const _e481 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Every token you do not name keeps its baseline value, so a theme is only as large as the difference you want. Declare one theme and it is used automatically; declare several and name the one you want with \"theme\": { \"name\": \"Ledger\" } in webfluent.app.json.");
  _e344.appendChild(_e481);
  const _e482 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e482);
  const _e483 = WF.h("h3", { className: "wf-heading" }, "Example themes");
  _e344.appendChild(_e483);
  const _e484 = WF.h("p", { className: "wf-text" }, "Four starting points ship in examples/themes/. Copy one into your src/ and edit it — they are ordinary source files, not engine settings.");
  _e344.appendChild(_e484);
  const _e485 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e485);
  const _e486 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e487 = WF.h("div", { className: "wf-card" });
  const _e488 = WF.h("div", { className: "wf-card__body" });
  const _e489 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e490 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "baseline");
  _e489.appendChild(_e490);
  const _e491 = WF.h("p", { className: "wf-text wf-text--bold" }, "Baseline");
  _e489.appendChild(_e491);
  _e488.appendChild(_e489);
  const _e492 = WF.h("div", { className: "wf-spacer" });
  _e488.appendChild(_e492);
  const _e493 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "What you get with no theme declared: clean, modern, blue primary.");
  _e488.appendChild(_e493);
  const _e494 = WF.h("div", { className: "wf-spacer" });
  _e488.appendChild(_e494);
  const _e495 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e496 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Primary");
  _e495.appendChild(_e496);
  const _e497 = WF.h("button", { className: "wf-btn wf-btn--success wf-btn--small" }, "Success");
  _e495.appendChild(_e497);
  const _e498 = WF.h("span", { className: "wf-badge wf-badge--info" }, "Tag");
  _e495.appendChild(_e498);
  _e488.appendChild(_e495);
  const _e499 = WF.h("div", { className: "wf-spacer" });
  _e488.appendChild(_e499);
  const _e500 = WF.h("progress", { className: "wf-progress wf-progress--primary", value: 65, max: 100 });
  _e488.appendChild(_e500);
  const _e501 = WF.h("div", { className: "wf-spacer" });
  _e488.appendChild(_e501);
  const _e502 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "No declaration needed.");
  _e488.appendChild(_e502);
  _e487.appendChild(_e488);
  _e487.style.background = "#ffffff";
  _e487.style.border = "1px solid #E2E8F0";
  _e487.style.borderRadius = "0.75rem";
  _e486.appendChild(_e487);
  const _e503 = WF.h("div", { className: "wf-card" });
  const _e504 = WF.h("div", { className: "wf-card__body" });
  const _e505 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e506 = WF.h("span", { className: "wf-badge wf-badge--secondary" }, "dark");
  _e505.appendChild(_e506);
  const _e507 = WF.h("p", { className: "wf-text wf-text--bold" }, "Dark");
  _e505.appendChild(_e507);
  _e504.appendChild(_e505);
  const _e508 = WF.h("div", { className: "wf-spacer" });
  _e504.appendChild(_e508);
  const _e509 = WF.h("p", { className: "wf-text wf-text--small" }, "Dark backgrounds with light text and vibrant accents.");
  _e504.appendChild(_e509);
  const _e510 = WF.h("div", { className: "wf-spacer" });
  _e504.appendChild(_e510);
  const _e511 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e512 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Primary");
  _e511.appendChild(_e512);
  const _e513 = WF.h("button", { className: "wf-btn wf-btn--danger wf-btn--small" }, "Danger");
  _e511.appendChild(_e513);
  const _e514 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Live");
  _e511.appendChild(_e514);
  _e504.appendChild(_e511);
  const _e515 = WF.h("div", { className: "wf-spacer" });
  _e504.appendChild(_e515);
  const _e516 = WF.h("progress", { className: "wf-progress wf-progress--info", value: 80, max: 100 });
  _e504.appendChild(_e516);
  const _e517 = WF.h("div", { className: "wf-spacer" });
  _e504.appendChild(_e517);
  const _e518 = WF.h("code", { className: "wf-code wf-code--block" }, "examples/themes/dark.wf");
  _e504.appendChild(_e518);
  _e503.appendChild(_e504);
  _e503.style.background = "#0F172A";
  _e503.style.color = "#E2E8F0";
  _e503.style.border = "1px solid #334155";
  _e503.style.borderRadius = "0.75rem";
  _e486.appendChild(_e503);
  const _e519 = WF.h("div", { className: "wf-card" });
  const _e520 = WF.h("div", { className: "wf-card__body" });
  const _e521 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e522 = WF.h("span", { className: "wf-badge" }, "minimal");
  _e521.appendChild(_e522);
  const _e523 = WF.h("p", { className: "wf-text wf-text--bold" }, "Minimal");
  _e521.appendChild(_e523);
  _e520.appendChild(_e521);
  const _e524 = WF.h("div", { className: "wf-spacer" });
  _e520.appendChild(_e524);
  const _e525 = WF.h("p", { className: "wf-text wf-text--small" }, "Black and white. No shadows, no border-radius. Pure content.");
  _e520.appendChild(_e525);
  const _e526 = WF.h("div", { className: "wf-spacer" });
  _e520.appendChild(_e526);
  const _e527 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e528 = WF.h("button", { className: "wf-btn wf-btn--small" }, "Action");
  _e527.appendChild(_e528);
  const _e529 = WF.h("span", { className: "wf-badge" }, "Note");
  _e527.appendChild(_e529);
  _e520.appendChild(_e527);
  const _e530 = WF.h("div", { className: "wf-spacer" });
  _e520.appendChild(_e530);
  const _e531 = WF.h("progress", { className: "wf-progress", value: 50, max: 100 });
  _e520.appendChild(_e531);
  const _e532 = WF.h("div", { className: "wf-spacer" });
  _e520.appendChild(_e532);
  const _e533 = WF.h("code", { className: "wf-code wf-code--block" }, "examples/themes/minimal.wf");
  _e520.appendChild(_e533);
  _e519.appendChild(_e520);
  _e519.style.background = "#ffffff";
  _e519.style.border = "2px solid #000000";
  _e519.style.borderRadius = "0";
  _e486.appendChild(_e519);
  const _e534 = WF.h("div", { className: "wf-card" });
  const _e535 = WF.h("div", { className: "wf-card__body" });
  const _e536 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e537 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "brutalist");
  _e536.appendChild(_e537);
  const _e538 = WF.h("p", { className: "wf-text wf-text--bold" }, "Brutalist");
  _e536.appendChild(_e538);
  _e535.appendChild(_e536);
  const _e539 = WF.h("div", { className: "wf-spacer" });
  _e535.appendChild(_e539);
  const _e540 = WF.h("p", { className: "wf-text wf-text--small" }, "Monospace font, bold red primary, hard offset shadows.");
  _e535.appendChild(_e540);
  const _e541 = WF.h("div", { className: "wf-spacer" });
  _e535.appendChild(_e541);
  const _e542 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e543 = WF.h("button", { className: "wf-btn wf-btn--danger wf-btn--small" }, "Action");
  _e542.appendChild(_e543);
  const _e544 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Alert");
  _e542.appendChild(_e544);
  _e535.appendChild(_e542);
  const _e545 = WF.h("div", { className: "wf-spacer" });
  _e535.appendChild(_e545);
  const _e546 = WF.h("progress", { className: "wf-progress wf-progress--danger", value: 90, max: 100 });
  _e535.appendChild(_e546);
  const _e547 = WF.h("div", { className: "wf-spacer" });
  _e535.appendChild(_e547);
  const _e548 = WF.h("code", { className: "wf-code wf-code--block" }, "examples/themes/brutalist.wf");
  _e535.appendChild(_e548);
  _e534.appendChild(_e535);
  _e534.style.background = "#ffffff";
  _e534.style.border = "3px solid #000000";
  _e534.style.borderRadius = "0";
  _e534.style.boxShadow = "4px 4px 0 #000000";
  _e534.style.fontFamily = "monospace";
  _e486.appendChild(_e534);
  _e344.appendChild(_e486);
  const _e549 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e549);
  const _e550 = WF.h("hr", { className: "wf-divider" });
  _e344.appendChild(_e550);
  const _e551 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e551);
  const _e552 = WF.h("h2", { className: "wf-heading" }, "Custom Tokens");
  _e344.appendChild(_e552);
  const _e553 = WF.h("p", { className: "wf-text" }, "A theme covers the design. For values a machine supplies — a deploy pipeline injecting a brand colour — config tokens still apply, on top of whatever the theme set.");
  _e344.appendChild(_e553);
  const _e554 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e554);
  const _e555 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e556 = WF.h("div", { className: "wf-card__body" });
  const _e557 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"theme\": {\n    \"tokens\": {\n      \"color-primary\": \"#8B5CF6\"\n    }\n  }\n}");
  _e556.appendChild(_e557);
  _e555.appendChild(_e556);
  _e344.appendChild(_e555);
  const _e558 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e558);
  const _e559 = WF.h("hr", { className: "wf-divider" });
  _e344.appendChild(_e559);
  const _e560 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e560);
  const _e561 = WF.h("h2", { className: "wf-heading" }, "Style Blocks");
  _e344.appendChild(_e561);
  const _e562 = WF.h("p", { className: "wf-text" }, "Override styles on any component with inline style blocks.");
  _e344.appendChild(_e562);
  const _e563 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e563);
  const _e564 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e565 = WF.h("div", { className: "wf-card__body" });
  const _e566 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Custom\") {\n    style {\n        background: \"#8B5CF6\"\n        padding: xl\n        radius: lg\n    }\n}");
  _e565.appendChild(_e566);
  _e564.appendChild(_e565);
  _e344.appendChild(_e564);
  const _e567 = WF.h("div", { className: "wf-spacer" });
  _e344.appendChild(_e567);
  _root.appendChild(_e344);
  return _root;
}

function Page_Guide(params) {
  const _root = document.createDocumentFragment();
  const _e568 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e569 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e569);
  const _e570 = WF.h("h1", { className: "wf-heading" }, "Language Guide");
  _e568.appendChild(_e570);
  const _e571 = WF.h("p", { className: "wf-text wf-text--muted" }, "Learn the core concepts of WebFluent.");
  _e568.appendChild(_e571);
  const _e572 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e572);
  const _e573 = WF.h("h2", { className: "wf-heading" }, "Pages");
  _e568.appendChild(_e573);
  const _e574 = WF.h("p", { className: "wf-text" }, "Pages are top-level route targets. Each page defines a URL path and contains the UI tree for that route.");
  _e568.appendChild(_e574);
  const _e575 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e575);
  const _e576 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e577 = WF.h("div", { className: "wf-card__body" });
  const _e578 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\", title: \"Home\") {\n    Container {\n        Heading(\"Welcome\", h1)\n        Text(\"This is the home page.\")\n    }\n}");
  _e577.appendChild(_e578);
  _e576.appendChild(_e577);
  _e568.appendChild(_e576);
  const _e579 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e579);
  const _e580 = WF.h("p", { className: "wf-text wf-text--bold" }, "Page attributes:");
  _e568.appendChild(_e580);
  const _e581 = WF.h("table", { className: "wf-table" });
  const _e582 = WF.h("thead", {});
  const _e583 = WF.h("td", {}, "Attribute");
  _e582.appendChild(_e583);
  const _e584 = WF.h("td", {}, "Type");
  _e582.appendChild(_e584);
  const _e585 = WF.h("td", {}, "Description");
  _e582.appendChild(_e585);
  _e581.appendChild(_e582);
  const _e586 = WF.h("tr", {});
  const _e587 = WF.h("td", {}, "path");
  _e586.appendChild(_e587);
  const _e588 = WF.h("td", {}, "String");
  _e586.appendChild(_e588);
  const _e589 = WF.h("td", {}, "URL route for this page (required)");
  _e586.appendChild(_e589);
  _e581.appendChild(_e586);
  const _e590 = WF.h("tr", {});
  const _e591 = WF.h("td", {}, "title");
  _e590.appendChild(_e591);
  const _e592 = WF.h("td", {}, "String");
  _e590.appendChild(_e592);
  const _e593 = WF.h("td", {}, "Document title");
  _e590.appendChild(_e593);
  _e581.appendChild(_e590);
  const _e594 = WF.h("tr", {});
  const _e595 = WF.h("td", {}, "guard");
  _e594.appendChild(_e595);
  const _e596 = WF.h("td", {}, "Expression");
  _e594.appendChild(_e596);
  const _e597 = WF.h("td", {}, "Navigation guard — redirects if false");
  _e594.appendChild(_e597);
  _e581.appendChild(_e594);
  const _e598 = WF.h("tr", {});
  const _e599 = WF.h("td", {}, "redirect");
  _e598.appendChild(_e599);
  const _e600 = WF.h("td", {}, "String");
  _e598.appendChild(_e600);
  const _e601 = WF.h("td", {}, "Redirect target when guard fails");
  _e598.appendChild(_e601);
  _e581.appendChild(_e598);
  _e568.appendChild(_e581);
  const _e602 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e602);
  const _e603 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e603);
  const _e604 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e604);
  const _e605 = WF.h("h2", { className: "wf-heading" }, "Components");
  _e568.appendChild(_e605);
  const _e606 = WF.h("p", { className: "wf-text" }, "Reusable UI blocks that accept props and can have internal state.");
  _e568.appendChild(_e606);
  const _e607 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e607);
  const _e608 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e609 = WF.h("div", { className: "wf-card__body" });
  const _e610 = WF.h("code", { className: "wf-code wf-code--block" }, "Component UserCard (name: String, role: String, active: Bool = true) {\n    Card(elevated) {\n        Row(align: center, gap: md) {\n            Avatar(initials: \"U\", primary)\n            Stack {\n                Text(name, bold)\n                Text(role, muted)\n            }\n            if active {\n                Badge(\"Active\", success)\n            }\n        }\n    }\n}\n\n// Usage\nUserCard(name: \"Monzer\", role: \"Developer\")");
  _e609.appendChild(_e610);
  _e608.appendChild(_e609);
  _e568.appendChild(_e608);
  const _e611 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e611);
  const _e612 = WF.h("p", { className: "wf-text wf-text--muted" }, "Props support types: String, Number, Bool, List, Map. Optional props use ?, defaults use =.");
  _e568.appendChild(_e612);
  const _e613 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e613);
  const _e614 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e614);
  const _e615 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e615);
  const _e616 = WF.h("h2", { className: "wf-heading" }, "State and Reactivity");
  _e568.appendChild(_e616);
  const _e617 = WF.h("p", { className: "wf-text" }, "State is declared with the state keyword. It is reactive — any UI that reads it updates automatically when it changes.");
  _e568.appendChild(_e617);
  const _e618 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e618);
  const _e619 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e620 = WF.h("div", { className: "wf-card__body" });
  const _e621 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Counter (path: \"/counter\") {\n    state count = 0\n\n    Container {\n        Text(\"Count: {count}\")\n        Button(\"+1\", primary) { count = count + 1 }\n        Button(\"-1\") { count = count - 1 }\n    }\n}");
  _e620.appendChild(_e621);
  _e619.appendChild(_e620);
  _e568.appendChild(_e619);
  const _e622 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e622);
  const _e623 = WF.h("p", { className: "wf-text wf-text--bold" }, "Derived state:");
  _e568.appendChild(_e623);
  const _e624 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e625 = WF.h("div", { className: "wf-card__body" });
  const _e626 = WF.h("code", { className: "wf-code wf-code--block" }, "state items = [{name: \"A\", price: 3}, {name: \"B\", price: 2}]\nderived total = items.map(i => i.price).sum()\nderived isEmpty = items.length == 0");
  _e625.appendChild(_e626);
  _e624.appendChild(_e625);
  _e568.appendChild(_e624);
  const _e627 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e627);
  const _e628 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e628);
  const _e629 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e629);
  const _e630 = WF.h("h2", { className: "wf-heading" }, "Events");
  _e568.appendChild(_e630);
  const _e631 = WF.h("p", { className: "wf-text" }, "Event handlers are declared with on:event or via shorthand blocks on buttons.");
  _e568.appendChild(_e631);
  const _e632 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e632);
  const _e633 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e634 = WF.h("div", { className: "wf-card__body" });
  const _e635 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Submit\") {\n    on:click {\n        submitForm()\n    }\n}\n\nInput(text, placeholder: \"Search...\") {\n    on:input {\n        searchQuery = value\n    }\n    on:keydown {\n        if key == \"Enter\" {\n            performSearch()\n        }\n    }\n}\n\n// Shorthand: block on Button defaults to on:click\nButton(\"Save\") { save() }");
  _e634.appendChild(_e635);
  _e633.appendChild(_e634);
  _e568.appendChild(_e633);
  const _e636 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e636);
  const _e637 = WF.h("p", { className: "wf-text wf-text--muted" }, "Supported events: on:click, on:submit, on:input, on:change, on:focus, on:blur, on:keydown, on:keyup, on:mouseover, on:mouseout, on:mount, on:unmount");
  _e568.appendChild(_e637);
  const _e638 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e638);
  const _e639 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e639);
  const _e640 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e640);
  const _e641 = WF.h("h2", { className: "wf-heading" }, "Control Flow");
  _e568.appendChild(_e641);
  const _e642 = WF.h("p", { className: "wf-text wf-text--bold" }, "Conditionals:");
  _e568.appendChild(_e642);
  const _e643 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e644 = WF.h("div", { className: "wf-card__body" });
  const _e645 = WF.h("code", { className: "wf-code wf-code--block" }, "if isLoggedIn {\n    Text(\"Welcome back!\")\n} else if isGuest {\n    Text(\"Hello, guest\")\n} else {\n    Button(\"Log In\") { navigate(\"/login\") }\n}");
  _e644.appendChild(_e645);
  _e643.appendChild(_e644);
  _e568.appendChild(_e643);
  const _e646 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e646);
  const _e647 = WF.h("p", { className: "wf-text wf-text--bold" }, "Loops:");
  _e568.appendChild(_e647);
  const _e648 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e649 = WF.h("div", { className: "wf-card__body" });
  const _e650 = WF.h("code", { className: "wf-code wf-code--block" }, "for user in users {\n    UserCard(name: user.name, role: user.role)\n}\n\n// With index\nfor item, index in items {\n    Text(\"{index + 1}. {item}\")\n}");
  _e649.appendChild(_e650);
  _e648.appendChild(_e649);
  _e568.appendChild(_e648);
  const _e651 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e651);
  const _e652 = WF.h("p", { className: "wf-text wf-text--bold" }, "Show/Hide (keeps element in DOM, toggles visibility):");
  _e568.appendChild(_e652);
  const _e653 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e654 = WF.h("div", { className: "wf-card__body" });
  const _e655 = WF.h("code", { className: "wf-code wf-code--block" }, "show isExpanded {\n    Card { Text(\"Expanded content\") }\n}");
  _e654.appendChild(_e655);
  _e653.appendChild(_e654);
  _e568.appendChild(_e653);
  const _e656 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e656);
  const _e657 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e657);
  const _e658 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e658);
  const _e659 = WF.h("h2", { className: "wf-heading" }, "Stores");
  _e568.appendChild(_e659);
  const _e660 = WF.h("p", { className: "wf-text" }, "Stores hold shared state accessible from any page or component.");
  _e568.appendChild(_e660);
  const _e661 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e661);
  const _e662 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e663 = WF.h("div", { className: "wf-card__body" });
  const _e664 = WF.h("code", { className: "wf-code wf-code--block" }, "Store CartStore {\n    state items = []\n\n    derived total = items.map(i => i.price * i.quantity).sum()\n    derived count = items.length\n\n    action addItem(product: Map) {\n        items.push({ id: product.id, name: product.name, price: product.price, quantity: 1 })\n    }\n\n    action removeItem(id: Number) {\n        items = items.filter(i => i.id != id)\n    }\n}\n\n// Usage in a page\nPage Cart (path: \"/cart\") {\n    use CartStore\n\n    Text(\"Total: ${CartStore.total}\")\n    Button(\"Clear\") { CartStore.clear() }\n}");
  _e663.appendChild(_e664);
  _e662.appendChild(_e663);
  _e568.appendChild(_e662);
  const _e665 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e665);
  const _e666 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e666);
  const _e667 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e667);
  const _e668 = WF.h("h2", { className: "wf-heading" }, "Routing");
  _e568.appendChild(_e668);
  const _e669 = WF.h("p", { className: "wf-text" }, "SPA routing is declared in the App file.");
  _e568.appendChild(_e669);
  const _e670 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e670);
  const _e671 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e672 = WF.h("div", { className: "wf-card__body" });
  const _e673 = WF.h("code", { className: "wf-code wf-code--block" }, "App {\n    Navbar {\n        Navbar.Brand { Text(\"My App\", heading) }\n        Navbar.Links {\n            Link(to: \"/\") { Text(\"Home\") }\n            Link(to: \"/about\") { Text(\"About\") }\n        }\n    }\n\n    Router {\n        Route(path: \"/\", page: Home)\n        Route(path: \"/about\", page: About)\n        Route(path: \"/user/:id\", page: UserProfile)\n        Route(path: \"*\", page: NotFound)\n    }\n}\n\n// Programmatic navigation\nButton(\"Go Home\") { navigate(\"/\") }\n\n// Dynamic routes access params\nPage UserProfile (path: \"/user/:id\") {\n    Text(\"User ID: {params.id}\")\n}");
  _e672.appendChild(_e673);
  _e671.appendChild(_e672);
  _e568.appendChild(_e671);
  const _e674 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e674);
  const _e675 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e675);
  const _e676 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e676);
  const _e677 = WF.h("h2", { className: "wf-heading" }, "Data Fetching");
  _e568.appendChild(_e677);
  const _e678 = WF.h("p", { className: "wf-text" }, "Built-in async data loading with automatic loading, error, and success states.");
  _e568.appendChild(_e678);
  const _e679 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e679);
  const _e680 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e681 = WF.h("div", { className: "wf-card__body" });
  const _e682 = WF.h("code", { className: "wf-code wf-code--block" }, "fetch users from \"/api/users\" {\n    loading {\n        Spinner()\n    }\n    error (err) {\n        Alert(\"Failed to load users\", danger)\n    }\n    success {\n        for user in users {\n            UserCard(name: user.name, role: user.role)\n        }\n    }\n}\n\n// With options\nfetch result from \"/api/submit\" (method: \"POST\", body: { name: name, email: email }) {\n    success {\n        Alert(\"Saved!\", success)\n    }\n}");
  _e681.appendChild(_e682);
  _e680.appendChild(_e681);
  _e568.appendChild(_e680);
  const _e683 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e683);
  const _e684 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e684);
  const _e685 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e685);
  const _e686 = WF.h("h2", { className: "wf-heading" }, "Return Values");
  _e568.appendChild(_e686);
  const _e687 = WF.h("p", { className: "wf-text" }, "Store actions can return values using the return keyword.");
  _e568.appendChild(_e687);
  const _e688 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e688);
  const _e689 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e690 = WF.h("div", { className: "wf-card__body" });
  const _e691 = WF.h("code", { className: "wf-code wf-code--block" }, "Store AuthStore {\n    state accessToken = \"\"\n\n    action getHeaders() {\n        h = {}\n        h[\"Authorization\"] = \"Bearer \" + accessToken\n        return h\n    }\n}");
  _e690.appendChild(_e691);
  _e689.appendChild(_e690);
  _e568.appendChild(_e689);
  const _e692 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e692);
  const _e693 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e693);
  const _e694 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e694);
  const _e695 = WF.h("h2", { className: "wf-heading" }, "Browser Globals");
  _e568.appendChild(_e695);
  const _e696 = WF.h("p", { className: "wf-text" }, "Standard browser APIs are available directly without any special syntax. They compile to their JavaScript equivalents.");
  _e568.appendChild(_e696);
  const _e697 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e697);
  const _e698 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e699 = WF.h("div", { className: "wf-card__body" });
  const _e700 = WF.h("code", { className: "wf-code wf-code--block" }, "// Storage\nlocalStorage.setItem(\"token\", tok)\nsessionStorage.getItem(\"key\")\n\n// Window & Document\nwindow.open(\"https://example.com\")\n\n// JSON\ndata = JSON.parse(responseText)\ntext = JSON.stringify(obj)\n\n// Console\nconsole.log(\"debug info\")\n\n// Timers\nsetTimeout(callback, 1000)");
  _e699.appendChild(_e700);
  _e698.appendChild(_e699);
  _e568.appendChild(_e698);
  const _e701 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e701);
  const _e702 = WF.h("p", { className: "wf-text wf-text--muted" }, "Available globals: window, document, console, localStorage, sessionStorage, JSON, Math, Date, setTimeout, setInterval, parseInt, parseFloat, Array, Object, Promise, Error, fetch, alert, confirm, prompt, and more.");
  _e568.appendChild(_e702);
  const _e703 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e703);
  const _e704 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e704);
  const _e705 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e705);
  const _e706 = WF.h("h2", { className: "wf-heading" }, "Map Literals");
  _e568.appendChild(_e706);
  const _e707 = WF.h("p", { className: "wf-text" }, "Map literals support quoted string keys for HTTP headers and special field names. Reserved words also work as map keys.");
  _e568.appendChild(_e707);
  const _e708 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e708);
  const _e709 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e710 = WF.h("div", { className: "wf-card__body" });
  const _e711 = WF.h("code", { className: "wf-code wf-code--block" }, "// Quoted keys for headers\nheaders: { \"Content-Type\": \"application/json\", \"X-Api-Key\": apiKey }\n\n// Reserved words as keys\nbody: { action: \"create\", token: sessionToken, state: \"active\" }");
  _e710.appendChild(_e711);
  _e709.appendChild(_e710);
  _e568.appendChild(_e709);
  const _e712 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e712);
  const _e713 = WF.h("hr", { className: "wf-divider" });
  _e568.appendChild(_e713);
  const _e714 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e714);
  const _e715 = WF.h("h2", { className: "wf-heading" }, "Operators");
  _e568.appendChild(_e715);
  const _e716 = WF.h("p", { className: "wf-text" }, "WebFluent supports all common comparison and logical operators.");
  _e568.appendChild(_e716);
  const _e717 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e717);
  const _e718 = WF.h("table", { className: "wf-table" });
  const _e719 = WF.h("thead", {});
  const _e720 = WF.h("td", {}, "Operator");
  _e719.appendChild(_e720);
  const _e721 = WF.h("td", {}, "Description");
  _e719.appendChild(_e721);
  _e718.appendChild(_e719);
  const _e722 = WF.h("tr", {});
  const _e723 = WF.h("td", {}, "==");
  _e722.appendChild(_e723);
  const _e724 = WF.h("td", {}, "Equal");
  _e722.appendChild(_e724);
  _e718.appendChild(_e722);
  const _e725 = WF.h("tr", {});
  const _e726 = WF.h("td", {}, "!=");
  _e725.appendChild(_e726);
  const _e727 = WF.h("td", {}, "Not equal");
  _e725.appendChild(_e727);
  _e718.appendChild(_e725);
  const _e728 = WF.h("tr", {});
  const _e729 = WF.h("td", {}, "!==");
  _e728.appendChild(_e729);
  const _e730 = WF.h("td", {}, "Strict not equal (alias for !=)");
  _e728.appendChild(_e730);
  _e718.appendChild(_e728);
  const _e731 = WF.h("tr", {});
  const _e732 = WF.h("td", {}, "< > <= >=");
  _e731.appendChild(_e732);
  const _e733 = WF.h("td", {}, "Comparison");
  _e731.appendChild(_e733);
  _e718.appendChild(_e731);
  const _e734 = WF.h("tr", {});
  const _e735 = WF.h("td", {}, "&& ||");
  _e734.appendChild(_e735);
  const _e736 = WF.h("td", {}, "Logical AND / OR");
  _e734.appendChild(_e736);
  _e718.appendChild(_e734);
  const _e737 = WF.h("tr", {});
  const _e738 = WF.h("td", {}, "!");
  _e737.appendChild(_e738);
  const _e739 = WF.h("td", {}, "Logical NOT");
  _e737.appendChild(_e739);
  _e718.appendChild(_e737);
  const _e740 = WF.h("tr", {});
  const _e741 = WF.h("td", {}, "+ - * / %");
  _e740.appendChild(_e741);
  const _e742 = WF.h("td", {}, "Arithmetic");
  _e740.appendChild(_e742);
  _e718.appendChild(_e740);
  _e568.appendChild(_e718);
  const _e743 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e743);
  const _e744 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e745 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/components"); } }, "Components Reference");
  _e744.appendChild(_e745);
  const _e746 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e744.appendChild(_e746);
  _e568.appendChild(_e744);
  const _e747 = WF.h("div", { className: "wf-spacer" });
  _e568.appendChild(_e747);
  _root.appendChild(_e568);
  return _root;
}

function Page_NotFound(params) {
  const _root = document.createDocumentFragment();
  const _e748 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e749 = WF.h("div", { className: "wf-spacer" });
  _e748.appendChild(_e749);
  const _e750 = WF.h("div", { className: "wf-stack wf-stack--gap-md" });
  const _e751 = WF.h("h1", { className: "wf-heading wf-text--center wf-heading--primary" }, "404");
  _e750.appendChild(_e751);
  const _e752 = WF.h("h2", { className: "wf-heading wf-text--center" }, "Page Not Found");
  _e750.appendChild(_e752);
  const _e753 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, "The page you are looking for does not exist or has been moved.");
  _e750.appendChild(_e753);
  const _e754 = WF.h("div", { className: "wf-spacer" });
  _e750.appendChild(_e754);
  const _e755 = WF.h("div", { className: "wf-row" });
  const _e756 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/"); } }, "Go Home");
  _e755.appendChild(_e756);
  _e750.appendChild(_e755);
  _e748.appendChild(_e750);
  const _e757 = WF.h("div", { className: "wf-spacer" });
  _e748.appendChild(_e757);
  _root.appendChild(_e748);
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
  const _e758 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e759 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e759);
  const _e760 = WF.h("h1", { className: "wf-heading" }, "Components Reference");
  _e758.appendChild(_e760);
  const _e761 = WF.h("p", { className: "wf-text wf-text--muted" }, "50+ built-in components. Below are live interactive examples you can play with.");
  _e758.appendChild(_e761);
  const _e762 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e762);
  const _e763 = WF.h("h2", { className: "wf-heading" }, "Buttons");
  _e758.appendChild(_e763);
  const _e764 = WF.h("p", { className: "wf-text" }, "Buttons support size, color, and shape modifiers.");
  _e758.appendChild(_e764);
  const _e765 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e765);
  const _e766 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e767 = WF.h("div", { className: "wf-card__body" });
  const _e768 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e769 = WF.h("button", { className: "wf-btn" }, "Default");
  _e768.appendChild(_e769);
  const _e770 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e768.appendChild(_e770);
  const _e771 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e768.appendChild(_e771);
  const _e772 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e768.appendChild(_e772);
  const _e773 = WF.h("button", { className: "wf-btn wf-btn--warning" }, "Warning");
  _e768.appendChild(_e773);
  const _e774 = WF.h("button", { className: "wf-btn wf-btn--info" }, "Info");
  _e768.appendChild(_e774);
  _e767.appendChild(_e768);
  const _e775 = WF.h("div", { className: "wf-spacer" });
  _e767.appendChild(_e775);
  const _e776 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e777 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Small");
  _e776.appendChild(_e777);
  const _e778 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Medium");
  _e776.appendChild(_e778);
  const _e779 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large" }, "Large");
  _e776.appendChild(_e779);
  const _e780 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--rounded" }, "Rounded");
  _e776.appendChild(_e780);
  const _e781 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--full" }, "Full Width");
  _e776.appendChild(_e781);
  _e767.appendChild(_e776);
  _e766.appendChild(_e767);
  _e758.appendChild(_e766);
  const _e782 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e782);
  const _e783 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e784 = WF.h("div", { className: "wf-card__body" });
  const _e785 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Primary\", primary)\nButton(\"Large\", primary, large)\nButton(\"Rounded\", success, rounded)\nButton(\"Full Width\", danger, full)");
  _e784.appendChild(_e785);
  _e783.appendChild(_e784);
  _e758.appendChild(_e783);
  const _e786 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e786);
  const _e787 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e787);
  const _e788 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e788);
  const _e789 = WF.h("h2", { className: "wf-heading" }, "Cards");
  _e758.appendChild(_e789);
  const _e790 = WF.h("p", { className: "wf-text" }, "Cards are surfaces for grouping content. They support Header, Body, and Footer sub-components.");
  _e758.appendChild(_e790);
  const _e791 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e791);
  const _e792 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e793 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e794 = WF.h("div", { className: "wf-card" });
  const _e795 = WF.h("div", { className: "wf-card__header" });
  const _e796 = WF.h("p", { className: "wf-text wf-text--bold" }, "Default Card");
  _e795.appendChild(_e796);
  _e794.appendChild(_e795);
  const _e797 = WF.h("div", { className: "wf-card__body" });
  const _e798 = WF.h("p", { className: "wf-text wf-text--muted" }, "Basic card with header and body.");
  _e797.appendChild(_e798);
  _e794.appendChild(_e797);
  const _e799 = WF.h("div", { className: "wf-card__footer" });
  const _e800 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Action");
  console.log("clicked");
  _e799.appendChild(_e800);
  _e794.appendChild(_e799);
  _e793.appendChild(_e794);
  _e792.appendChild(_e793);
  const _e801 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e802 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e803 = WF.h("div", { className: "wf-card__header" });
  const _e804 = WF.h("p", { className: "wf-text wf-text--bold" }, "Elevated");
  _e803.appendChild(_e804);
  _e802.appendChild(_e803);
  const _e805 = WF.h("div", { className: "wf-card__body" });
  const _e806 = WF.h("p", { className: "wf-text wf-text--muted" }, "Card with shadow elevation.");
  _e805.appendChild(_e806);
  _e802.appendChild(_e805);
  _e801.appendChild(_e802);
  _e792.appendChild(_e801);
  const _e807 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e808 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e809 = WF.h("div", { className: "wf-card__header" });
  const _e810 = WF.h("p", { className: "wf-text wf-text--bold" }, "Outlined");
  _e809.appendChild(_e810);
  _e808.appendChild(_e809);
  const _e811 = WF.h("div", { className: "wf-card__body" });
  const _e812 = WF.h("p", { className: "wf-text wf-text--muted" }, "Card with border only.");
  _e811.appendChild(_e812);
  _e808.appendChild(_e811);
  _e807.appendChild(_e808);
  _e792.appendChild(_e807);
  _e758.appendChild(_e792);
  const _e813 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e813);
  const _e814 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e814);
  const _e815 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e815);
  const _e816 = WF.h("h2", { className: "wf-heading" }, "Form Controls");
  _e758.appendChild(_e816);
  const _e817 = WF.h("p", { className: "wf-text" }, "All form inputs support two-way binding with the bind: attribute.");
  _e758.appendChild(_e817);
  const _e818 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e818);
  const _e819 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e820 = WF.h("div", { className: "wf-card__body" });
  const _e821 = WF.h("div", { className: "wf-stack wf-stack--gap-md" });
  const _e822 = WF.h("input", { className: "wf-input", value: () => _inputVal(), "on:input": (e) => _inputVal.set(e.target.value), label: "Text Input", placeholder: "Type here...", type: "text" });
  _e821.appendChild(_e822);
  WF.condRender(_e821,
    () => (_inputVal() !== ""),
    () => {
      const _e823 = document.createDocumentFragment();
      const _e824 = WF.h("p", { className: "wf-text wf-text--primary wf-text--bold" }, () => `You typed: ${_inputVal()}`);
      _e823.appendChild(_e824);
      return _e823;
    },
    null,
    null
  );
  const _e825 = WF.h("hr", { className: "wf-divider" });
  _e821.appendChild(_e825);
  const _e826 = WF.h("select", { className: "wf-select", value: () => _selectVal(), "on:input": (e) => _selectVal.set(e.target.value), label: "Select" });
  const _e827 = WF.h("option", {}, "opt1");
  _e826.appendChild(_e827);
  const _e828 = WF.h("option", {}, "opt2");
  _e826.appendChild(_e828);
  const _e829 = WF.h("option", {}, "opt3");
  _e826.appendChild(_e829);
  _e821.appendChild(_e826);
  const _e830 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Selected: ${_selectVal()}`);
  _e821.appendChild(_e830);
  const _e831 = WF.h("hr", { className: "wf-divider" });
  _e821.appendChild(_e831);
  const _e832 = WF.h("label", { className: "wf-checkbox" });
  const _e833 = WF.h("input", { type: "checkbox", checked: () => _checkVal(), "on:change": () => _checkVal.set(!_checkVal()) });
  _e832.appendChild(_e833);
  _e832.appendChild(WF.text("I agree to the terms"));
  _e821.appendChild(_e832);
  const _e834 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Checked: ${_checkVal()}`);
  _e821.appendChild(_e834);
  const _e835 = WF.h("hr", { className: "wf-divider" });
  _e821.appendChild(_e835);
  const _e836 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e837 = WF.h("label", { className: "wf-radio" });
  const _e838 = WF.h("input", { type: "radio", checked: () => _radioVal() === "a", "on:change": () => _radioVal.set("a") });
  _e837.appendChild(_e838);
  _e837.appendChild(WF.text("Option A"));
  _e836.appendChild(_e837);
  const _e839 = WF.h("label", { className: "wf-radio" });
  const _e840 = WF.h("input", { type: "radio", checked: () => _radioVal() === "b", "on:change": () => _radioVal.set("b") });
  _e839.appendChild(_e840);
  _e839.appendChild(WF.text("Option B"));
  _e836.appendChild(_e839);
  const _e841 = WF.h("label", { className: "wf-radio" });
  const _e842 = WF.h("input", { type: "radio", checked: () => _radioVal() === "c", "on:change": () => _radioVal.set("c") });
  _e841.appendChild(_e842);
  _e841.appendChild(WF.text("Option C"));
  _e836.appendChild(_e841);
  _e821.appendChild(_e836);
  const _e843 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Selected: ${_radioVal()}`);
  _e821.appendChild(_e843);
  const _e844 = WF.h("hr", { className: "wf-divider" });
  _e821.appendChild(_e844);
  const _e845 = WF.h("label", { className: "wf-switch" });
  const _e846 = WF.h("input", { type: "checkbox", role: "switch",                  checked: () => _switchVal(), "aria-checked": () => _switchVal() ? "true" : "false",                  "on:change": () => _switchVal.set(!_switchVal()) });
  _e845.appendChild(_e846);
  const _e847 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e845.appendChild(_e847);
  _e845.appendChild(WF.text("Dark Mode"));
  _e821.appendChild(_e845);
  const _e848 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Enabled: ${_switchVal()}`);
  _e821.appendChild(_e848);
  const _e849 = WF.h("hr", { className: "wf-divider" });
  _e821.appendChild(_e849);
  const _e850 = WF.h("div", { className: "wf-slider" });
  const _e851 = WF.h("label", { className: "wf-form-label" }, "Volume");
  _e850.appendChild(_e851);
  const _e852 = WF.h("input", { type: "range", min: 0, max: 100, step: 1, value: () => _sliderVal(), "on:input": (e) => _sliderVal.set(Number(e.target.value)) });
  _e850.appendChild(_e852);
  const _e853 = WF.h("span", { className: "wf-slider__value" }, () => String(_sliderVal()));
  _e850.appendChild(_e853);
  _e821.appendChild(_e850);
  const _e854 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Value: ${_sliderVal()}`);
  _e821.appendChild(_e854);
  const _e855 = WF.h("hr", { className: "wf-divider" });
  _e821.appendChild(_e855);
  const _e857 = WF.h("div", { className: "wf-datepicker" });
  const _e858 = WF.h("label", { className: "wf-form-label" }, "Pick a Date");
  _e857.appendChild(_e858);
  const _e859 = WF.h("input", { type: "date", className: "wf-input", value: () => _dateVal(), "on:change": (e) => _dateVal.set(e.target.value) });
  _e857.appendChild(_e859);
  const _e856 = _e857;
  _e821.appendChild(_e856);
  const _e860 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Selected: ${_dateVal()}`);
  _e821.appendChild(_e860);
  const _e861 = WF.h("hr", { className: "wf-divider" });
  _e821.appendChild(_e861);
  const _e863 = WF.h("div", { className: "wf-file-upload" });
  const _e864 = WF.h("label", { className: "wf-form-label" }, "Upload Image");
  _e863.appendChild(_e864);
  const _e865 = WF.h("input", { type: "file", className: "wf-input", accept: "image/*" });
  _e863.appendChild(_e865);
  const _e862 = _e863;
  _e821.appendChild(_e862);
  const _e867 = WF.h("div", { className: "wf-file-upload" });
  const _e868 = WF.h("label", { className: "wf-form-label" }, "Documents");
  _e867.appendChild(_e868);
  const _e869 = WF.h("input", { type: "file", className: "wf-input", accept: ".pdf,.doc" });
  _e867.appendChild(_e869);
  const _e866 = _e867;
  _e821.appendChild(_e866);
  _e820.appendChild(_e821);
  _e819.appendChild(_e820);
  _e758.appendChild(_e819);
  const _e870 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e870);
  const _e871 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e871);
  const _e872 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e872);
  const _e873 = WF.h("h2", { className: "wf-heading" }, "Feedback");
  _e758.appendChild(_e873);
  const _e874 = WF.h("p", { className: "wf-text" }, "Alerts, modals, progress bars, and loading indicators.");
  _e758.appendChild(_e874);
  const _e875 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e875);
  const _e876 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e877 = WF.h("div", { className: "wf-card__body" });
  const _e878 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e879 = WF.h("div", { className: "wf-alert wf-alert--success", role: "status" }, "This is a success alert.");
  _e878.appendChild(_e879);
  const _e880 = WF.h("div", { className: "wf-alert wf-alert--warning", role: "alert" }, "This is a warning alert.");
  _e878.appendChild(_e880);
  const _e881 = WF.h("div", { className: "wf-alert wf-alert--danger", role: "alert" }, "This is a danger alert.");
  _e878.appendChild(_e881);
  const _e882 = WF.h("div", { className: "wf-alert wf-alert--info", role: "status" }, "This is an info alert.");
  _e878.appendChild(_e882);
  _e877.appendChild(_e878);
  const _e883 = WF.h("div", { className: "wf-spacer" });
  _e877.appendChild(_e883);
  const _e884 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e885 = WF.h("div", { className: "wf-spinner", role: "status" });
  _e884.appendChild(_e885);
  const _e886 = WF.h("div", { className: "wf-spinner wf-spinner--large wf-spinner--primary", role: "status" });
  _e884.appendChild(_e886);
  const _e887 = WF.h("progress", { className: "wf-progress", value: _sliderVal(), max: 100 });
  _e884.appendChild(_e887);
  _e877.appendChild(_e884);
  const _e888 = WF.h("div", { className: "wf-spacer" });
  _e877.appendChild(_e888);
  const _e889 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { _activeModal.set(true); } }, "Open Modal");
  _e877.appendChild(_e889);
  _e876.appendChild(_e877);
  _e758.appendChild(_e876);
  const _e890 = WF.h("dialog", { className: "wf-modal", "aria-labelledby": "wf-dlg-890" });
  const _e891 = WF.h("div", { className: "wf-modal__content" });
  const _e892 = WF.h("div", { className: "wf-modal__header" }, WF.h("h3", { id: "wf-dlg-890" }, "Example Modal"));
  _e891.appendChild(_e892);
  const _e893 = WF.h("div", { className: "wf-modal__body" });
  const _e894 = WF.h("p", { className: "wf-text" }, "This is a real modal dialog. It was triggered by clicking the button.");
  _e893.appendChild(_e894);
  const _e895 = WF.h("div", { className: "wf-spacer" });
  _e893.appendChild(_e895);
  const _e896 = WF.h("p", { className: "wf-text wf-text--muted" }, "The modal is controlled by a state variable.");
  _e893.appendChild(_e896);
  _e891.appendChild(_e893);
  const _e897 = WF.h("div", { className: "wf-modal__footer" });
  const _e898 = WF.h("button", { className: "wf-btn", "on:click": (e) => { _activeModal.set(false); } }, "Close");
  _e897.appendChild(_e898);
  const _e899 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { _activeModal.set(false); } }, "Confirm");
  _e897.appendChild(_e899);
  _e891.appendChild(_e897);
  _e890.appendChild(_e891);
  WF.bindDialog(_e890, _activeModal);
  _e758.appendChild(_e890);
  const _e900 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e900);
  const _e901 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e901);
  const _e902 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e902);
  const _e903 = WF.h("h2", { className: "wf-heading" }, "Data Display");
  _e758.appendChild(_e903);
  const _e904 = WF.h("p", { className: "wf-text" }, "Tables, badges, avatars, tags, and tooltips.");
  _e758.appendChild(_e904);
  const _e905 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e905);
  const _e906 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e907 = WF.h("div", { className: "wf-card__body" });
  const _e908 = WF.h("table", { className: "wf-table" });
  const _e909 = WF.h("thead", {});
  const _e910 = WF.h("td", {}, "Name");
  _e909.appendChild(_e910);
  const _e911 = WF.h("td", {}, "Role");
  _e909.appendChild(_e911);
  const _e912 = WF.h("td", {}, "Status");
  _e909.appendChild(_e912);
  _e908.appendChild(_e909);
  const _e913 = WF.h("tr", {});
  const _e914 = WF.h("td", {}, "Monzer Omer");
  _e913.appendChild(_e914);
  const _e915 = WF.h("td", {}, "Creator");
  _e913.appendChild(_e915);
  const _e916 = WF.h("td", {}, "Active");
  _e913.appendChild(_e916);
  _e908.appendChild(_e913);
  const _e917 = WF.h("tr", {});
  const _e918 = WF.h("td", {}, "Sara Ali");
  _e917.appendChild(_e918);
  const _e919 = WF.h("td", {}, "Designer");
  _e917.appendChild(_e919);
  const _e920 = WF.h("td", {}, "Active");
  _e917.appendChild(_e920);
  _e908.appendChild(_e917);
  const _e921 = WF.h("tr", {});
  const _e922 = WF.h("td", {}, "Omar Hassan");
  _e921.appendChild(_e922);
  const _e923 = WF.h("td", {}, "Developer");
  _e921.appendChild(_e923);
  const _e924 = WF.h("td", {}, "Away");
  _e921.appendChild(_e924);
  _e908.appendChild(_e921);
  _e907.appendChild(_e908);
  const _e925 = WF.h("div", { className: "wf-spacer" });
  _e907.appendChild(_e925);
  const _e926 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e927 = WF.h("div", { className: "wf-avatar wf-avatar--primary" }, "MO");
  _e926.appendChild(_e927);
  const _e928 = WF.h("div", { className: "wf-avatar" }, "SA");
  _e926.appendChild(_e928);
  const _e929 = WF.h("div", { className: "wf-avatar" }, "OH");
  _e926.appendChild(_e929);
  const _e930 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Admin");
  _e926.appendChild(_e930);
  const _e931 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Online");
  _e926.appendChild(_e931);
  const _e932 = WF.h("span", { className: "wf-tag" }, "WebFluent");
  _e926.appendChild(_e932);
  const _e933 = WF.h("span", { className: "wf-tag" }, "Rust");
  _e926.appendChild(_e933);
  const _e934 = WF.h("span", { className: "wf-tag" }, "Open Source");
  _e926.appendChild(_e934);
  _e907.appendChild(_e926);
  _e906.appendChild(_e907);
  _e758.appendChild(_e906);
  const _e935 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e935);
  const _e936 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e936);
  const _e937 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e937);
  const _e938 = WF.h("h2", { className: "wf-heading" }, "Layout");
  _e758.appendChild(_e938);
  const _e939 = WF.h("p", { className: "wf-text" }, "Container, Row, Column, Grid, Stack, Spacer, Divider.");
  _e758.appendChild(_e939);
  const _e940 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e940);
  const _e941 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e942 = WF.h("div", { className: "wf-card__body" });
  const _e943 = WF.h("p", { className: "wf-text wf-text--bold" }, "Grid with 3 columns:");
  _e942.appendChild(_e943);
  const _e944 = WF.h("div", { className: "wf-spacer" });
  _e942.appendChild(_e944);
  const _e945 = WF.h("div", { className: "wf-grid wf-grid--gap-sm", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e946 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e947 = WF.h("div", { className: "wf-card__body" });
  const _e948 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 1");
  _e947.appendChild(_e948);
  _e946.appendChild(_e947);
  _e945.appendChild(_e946);
  const _e949 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e950 = WF.h("div", { className: "wf-card__body" });
  const _e951 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 2");
  _e950.appendChild(_e951);
  _e949.appendChild(_e950);
  _e945.appendChild(_e949);
  const _e952 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e953 = WF.h("div", { className: "wf-card__body" });
  const _e954 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 3");
  _e953.appendChild(_e954);
  _e952.appendChild(_e953);
  _e945.appendChild(_e952);
  _e942.appendChild(_e945);
  const _e955 = WF.h("div", { className: "wf-spacer" });
  _e942.appendChild(_e955);
  const _e956 = WF.h("p", { className: "wf-text wf-text--bold" }, "Row with Columns (6/6 split):");
  _e942.appendChild(_e956);
  const _e957 = WF.h("div", { className: "wf-spacer" });
  _e942.appendChild(_e957);
  const _e958 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e959 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e960 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e961 = WF.h("div", { className: "wf-card__body" });
  const _e962 = WF.h("p", { className: "wf-text wf-text--center" }, "Left Half");
  _e961.appendChild(_e962);
  _e960.appendChild(_e961);
  _e959.appendChild(_e960);
  _e958.appendChild(_e959);
  const _e963 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e964 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e965 = WF.h("div", { className: "wf-card__body" });
  const _e966 = WF.h("p", { className: "wf-text wf-text--center" }, "Right Half");
  _e965.appendChild(_e966);
  _e964.appendChild(_e965);
  _e963.appendChild(_e964);
  _e958.appendChild(_e963);
  _e942.appendChild(_e958);
  const _e967 = WF.h("div", { className: "wf-spacer" });
  _e942.appendChild(_e967);
  const _e968 = WF.h("p", { className: "wf-text wf-text--bold" }, "Stack (vertical):");
  _e942.appendChild(_e968);
  const _e969 = WF.h("div", { className: "wf-spacer" });
  _e942.appendChild(_e969);
  const _e970 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e971 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e972 = WF.h("div", { className: "wf-card__body" });
  const _e973 = WF.h("p", { className: "wf-text" }, "Item 1");
  _e972.appendChild(_e973);
  _e971.appendChild(_e972);
  _e970.appendChild(_e971);
  const _e974 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e975 = WF.h("div", { className: "wf-card__body" });
  const _e976 = WF.h("p", { className: "wf-text" }, "Item 2");
  _e975.appendChild(_e976);
  _e974.appendChild(_e975);
  _e970.appendChild(_e974);
  const _e977 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e978 = WF.h("div", { className: "wf-card__body" });
  const _e979 = WF.h("p", { className: "wf-text" }, "Item 3");
  _e978.appendChild(_e979);
  _e977.appendChild(_e978);
  _e970.appendChild(_e977);
  _e942.appendChild(_e970);
  _e941.appendChild(_e942);
  _e758.appendChild(_e941);
  const _e980 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e980);
  const _e981 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e981);
  const _e982 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e982);
  const _e983 = WF.h("h2", { className: "wf-heading" }, "Icons & Icon Buttons");
  _e758.appendChild(_e983);
  const _e984 = WF.h("p", { className: "wf-text" }, "30 built-in SVG icons. Use Icon for display, IconButton for clickable actions.");
  _e758.appendChild(_e984);
  const _e985 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e985);
  const _e986 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e987 = WF.h("div", { className: "wf-card__body" });
  const _e988 = WF.h("p", { className: "wf-text wf-text--bold" }, "Available Icons:");
  _e987.appendChild(_e988);
  const _e989 = WF.h("div", { className: "wf-spacer" });
  _e987.appendChild(_e989);
  const _e990 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e991 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e992 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e993 = WF.h("i", { className: "wf-icon" }, "home");
  _e992.appendChild(_e993);
  const _e994 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "home");
  _e992.appendChild(_e994);
  _e991.appendChild(_e992);
  const _e995 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e996 = WF.h("i", { className: "wf-icon" }, "search");
  _e995.appendChild(_e996);
  const _e997 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "search");
  _e995.appendChild(_e997);
  _e991.appendChild(_e995);
  const _e998 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e999 = WF.h("i", { className: "wf-icon" }, "user");
  _e998.appendChild(_e999);
  const _e1000 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "user");
  _e998.appendChild(_e1000);
  _e991.appendChild(_e998);
  const _e1001 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1002 = WF.h("i", { className: "wf-icon" }, "settings");
  _e1001.appendChild(_e1002);
  const _e1003 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "settings");
  _e1001.appendChild(_e1003);
  _e991.appendChild(_e1001);
  const _e1004 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1005 = WF.h("i", { className: "wf-icon" }, "mail");
  _e1004.appendChild(_e1005);
  const _e1006 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "mail");
  _e1004.appendChild(_e1006);
  _e991.appendChild(_e1004);
  const _e1007 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1008 = WF.h("i", { className: "wf-icon" }, "bell");
  _e1007.appendChild(_e1008);
  const _e1009 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "bell");
  _e1007.appendChild(_e1009);
  _e991.appendChild(_e1007);
  _e990.appendChild(_e991);
  const _e1010 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1011 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1012 = WF.h("i", { className: "wf-icon" }, "edit");
  _e1011.appendChild(_e1012);
  const _e1013 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "edit");
  _e1011.appendChild(_e1013);
  _e1010.appendChild(_e1011);
  const _e1014 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1015 = WF.h("i", { className: "wf-icon" }, "trash");
  _e1014.appendChild(_e1015);
  const _e1016 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "trash");
  _e1014.appendChild(_e1016);
  _e1010.appendChild(_e1014);
  const _e1017 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1018 = WF.h("i", { className: "wf-icon" }, "plus");
  _e1017.appendChild(_e1018);
  const _e1019 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "plus");
  _e1017.appendChild(_e1019);
  _e1010.appendChild(_e1017);
  const _e1020 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1021 = WF.h("i", { className: "wf-icon" }, "check");
  _e1020.appendChild(_e1021);
  const _e1022 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "check");
  _e1020.appendChild(_e1022);
  _e1010.appendChild(_e1020);
  const _e1023 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1024 = WF.h("i", { className: "wf-icon" }, "close");
  _e1023.appendChild(_e1024);
  const _e1025 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "close");
  _e1023.appendChild(_e1025);
  _e1010.appendChild(_e1023);
  const _e1026 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1027 = WF.h("i", { className: "wf-icon" }, "copy");
  _e1026.appendChild(_e1027);
  const _e1028 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "copy");
  _e1026.appendChild(_e1028);
  _e1010.appendChild(_e1026);
  _e990.appendChild(_e1010);
  const _e1029 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1030 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1031 = WF.h("i", { className: "wf-icon" }, "star");
  _e1030.appendChild(_e1031);
  const _e1032 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "star");
  _e1030.appendChild(_e1032);
  _e1029.appendChild(_e1030);
  const _e1033 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1034 = WF.h("i", { className: "wf-icon" }, "heart");
  _e1033.appendChild(_e1034);
  const _e1035 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "heart");
  _e1033.appendChild(_e1035);
  _e1029.appendChild(_e1033);
  const _e1036 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1037 = WF.h("i", { className: "wf-icon" }, "eye");
  _e1036.appendChild(_e1037);
  const _e1038 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "eye");
  _e1036.appendChild(_e1038);
  _e1029.appendChild(_e1036);
  const _e1039 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1040 = WF.h("i", { className: "wf-icon" }, "download");
  _e1039.appendChild(_e1040);
  const _e1041 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "download");
  _e1039.appendChild(_e1041);
  _e1029.appendChild(_e1039);
  const _e1042 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1043 = WF.h("i", { className: "wf-icon" }, "upload");
  _e1042.appendChild(_e1043);
  const _e1044 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "upload");
  _e1042.appendChild(_e1044);
  _e1029.appendChild(_e1042);
  const _e1045 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1046 = WF.h("i", { className: "wf-icon" }, "link");
  _e1045.appendChild(_e1046);
  const _e1047 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "link");
  _e1045.appendChild(_e1047);
  _e1029.appendChild(_e1045);
  _e990.appendChild(_e1029);
  const _e1048 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1049 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1050 = WF.h("i", { className: "wf-icon" }, "calendar");
  _e1049.appendChild(_e1050);
  const _e1051 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "calendar");
  _e1049.appendChild(_e1051);
  _e1048.appendChild(_e1049);
  const _e1052 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1053 = WF.h("i", { className: "wf-icon" }, "filter");
  _e1052.appendChild(_e1053);
  const _e1054 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "filter");
  _e1052.appendChild(_e1054);
  _e1048.appendChild(_e1052);
  const _e1055 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1056 = WF.h("i", { className: "wf-icon" }, "info");
  _e1055.appendChild(_e1056);
  const _e1057 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "info");
  _e1055.appendChild(_e1057);
  _e1048.appendChild(_e1055);
  const _e1058 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1059 = WF.h("i", { className: "wf-icon" }, "warning");
  _e1058.appendChild(_e1059);
  const _e1060 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "warning");
  _e1058.appendChild(_e1060);
  _e1048.appendChild(_e1058);
  const _e1061 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1062 = WF.h("i", { className: "wf-icon" }, "logout");
  _e1061.appendChild(_e1062);
  const _e1063 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "logout");
  _e1061.appendChild(_e1063);
  _e1048.appendChild(_e1061);
  const _e1064 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e1065 = WF.h("i", { className: "wf-icon" }, "menu");
  _e1064.appendChild(_e1065);
  const _e1066 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "menu");
  _e1064.appendChild(_e1066);
  _e1048.appendChild(_e1064);
  _e990.appendChild(_e1048);
  _e987.appendChild(_e990);
  const _e1067 = WF.h("div", { className: "wf-spacer" });
  _e987.appendChild(_e1067);
  const _e1068 = WF.h("p", { className: "wf-text wf-text--bold" }, "Icon Buttons:");
  _e987.appendChild(_e1068);
  const _e1069 = WF.h("div", { className: "wf-spacer" });
  _e987.appendChild(_e1069);
  const _e1070 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1071 = WF.h("button", { className: "wf-icon-btn", "data-icon": "edit", "aria-label": "Edit", title: "Edit" }, WF.h("span", { className: "wf-icon", "data-icon": "edit" }));
  _e1070.appendChild(_e1071);
  const _e1072 = WF.h("button", { className: "wf-icon-btn wf-icon-btn--danger", "data-icon": "trash", "aria-label": "Delete", title: "Delete" }, WF.h("span", { className: "wf-icon", "data-icon": "trash" }));
  _e1070.appendChild(_e1072);
  const _e1073 = WF.h("button", { className: "wf-icon-btn wf-icon-btn--primary", "data-icon": "plus", "aria-label": "Add", title: "Add" }, WF.h("span", { className: "wf-icon", "data-icon": "plus" }));
  _e1070.appendChild(_e1073);
  const _e1074 = WF.h("button", { className: "wf-icon-btn wf-icon-btn--large", "data-icon": "search", "aria-label": "Search", title: "Search" }, WF.h("span", { className: "wf-icon", "data-icon": "search" }));
  _e1070.appendChild(_e1074);
  const _e1075 = WF.h("button", { className: "wf-icon-btn wf-icon-btn--small", "data-icon": "close", "aria-label": "Close", title: "Close" }, WF.h("span", { className: "wf-icon", "data-icon": "close" }));
  _e1070.appendChild(_e1075);
  _e987.appendChild(_e1070);
  _e986.appendChild(_e987);
  _e758.appendChild(_e986);
  const _e1076 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1076);
  const _e1077 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1078 = WF.h("div", { className: "wf-card__body" });
  const _e1079 = WF.h("code", { className: "wf-code wf-code--block" }, "Icon(\"home\")\nIcon(\"search\", large, primary)\nIconButton(icon: \"edit\", label: \"Edit\")\nIconButton(icon: \"trash\", label: \"Delete\", danger)");
  _e1078.appendChild(_e1079);
  _e1077.appendChild(_e1078);
  _e758.appendChild(_e1077);
  const _e1080 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1080);
  const _e1081 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e1081);
  const _e1082 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1082);
  const _e1083 = WF.h("h2", { className: "wf-heading" }, "Tooltips");
  _e758.appendChild(_e1083);
  const _e1084 = WF.h("p", { className: "wf-text" }, "Wrap any element in a Tooltip to show text on hover.");
  _e758.appendChild(_e1084);
  const _e1085 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1085);
  const _e1086 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1087 = WF.h("div", { className: "wf-card__body" });
  const _e1088 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1089 = WF.h("div", { className: "wf-tooltip", "aria-describedby": "wf-tip-1089" });
  const _e1090 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Hover me");
  _e1089.appendChild(_e1090);
  const _e1091 = WF.h("span", { className: "wf-tooltip__text", role: "tooltip", id: "wf-tip-1089" }, "This is a primary button");
  _e1089.appendChild(_e1091);
  _e1088.appendChild(_e1089);
  const _e1092 = WF.h("div", { className: "wf-tooltip", "aria-describedby": "wf-tip-1092" });
  const _e1093 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Delete");
  _e1092.appendChild(_e1093);
  const _e1094 = WF.h("span", { className: "wf-tooltip__text", role: "tooltip", id: "wf-tip-1092" }, "Deletes the item permanently");
  _e1092.appendChild(_e1094);
  _e1088.appendChild(_e1092);
  const _e1095 = WF.h("div", { className: "wf-tooltip", "aria-describedby": "wf-tip-1095" });
  const _e1096 = WF.h("div", { className: "wf-avatar wf-avatar--primary" }, "MO");
  _e1095.appendChild(_e1096);
  const _e1097 = WF.h("span", { className: "wf-tooltip__text", role: "tooltip", id: "wf-tip-1095" }, "User profile picture");
  _e1095.appendChild(_e1097);
  _e1088.appendChild(_e1095);
  _e1087.appendChild(_e1088);
  _e1086.appendChild(_e1087);
  _e758.appendChild(_e1086);
  const _e1098 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1098);
  const _e1099 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1100 = WF.h("div", { className: "wf-card__body" });
  const _e1101 = WF.h("code", { className: "wf-code wf-code--block" }, "Tooltip(text: \"Help text\") {\n    Button(\"Hover me\", primary)\n}");
  _e1100.appendChild(_e1101);
  _e1099.appendChild(_e1100);
  _e758.appendChild(_e1099);
  const _e1102 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1102);
  const _e1103 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e1103);
  const _e1104 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1104);
  const _e1105 = WF.h("h2", { className: "wf-heading" }, "Sidebar");
  _e758.appendChild(_e1105);
  const _e1106 = WF.h("p", { className: "wf-text" }, "Sidebar navigation with header, items, and dividers. Items support icons and links.");
  _e758.appendChild(_e1106);
  const _e1107 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1107);
  const _e1108 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1109 = WF.h("div", { className: "wf-card__body" });
  const _e1110 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e1111 = WF.h("aside", { className: "wf-sidebar", id: "wf-sidebar-1111" });
  const _e1112 = WF.h("div", { className: "wf-sidebar__header" });
  const _e1113 = WF.h("p", { className: "wf-text wf-text--heading" }, "My App");
  _e1112.appendChild(_e1113);
  _e1111.appendChild(_e1112);
  const _e1114 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/" });
  _e1114.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "home" }));
  const _e1115 = WF.h("p", { className: "wf-text" }, "Dashboard");
  _e1114.appendChild(_e1115);
  _e1111.appendChild(_e1114);
  const _e1116 = WF.h("a", { className: "wf-sidebar__item", href: WF._basePath +  "/components" });
  _e1116.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "settings" }));
  const _e1117 = WF.h("p", { className: "wf-text" }, "Settings");
  _e1116.appendChild(_e1117);
  _e1111.appendChild(_e1116);
  const _e1118 = WF.h("div", { className: "wf-sidebar__item" });
  _e1118.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "user" }));
  const _e1119 = WF.h("p", { className: "wf-text" }, "Profile");
  _e1118.appendChild(_e1119);
  _e1111.appendChild(_e1118);
  _e1111.appendChild(WF.h("div", { className: "wf-sidebar__divider" }));
  const _e1120 = WF.h("div", { className: "wf-sidebar__item" });
  _e1120.appendChild(WF.h("span", { className: "wf-icon", "data-icon": "logout" }));
  const _e1121 = WF.h("p", { className: "wf-text" }, "Logout");
  _e1120.appendChild(_e1121);
  _e1111.appendChild(_e1120);
  _e1110.appendChild(_e1111);
  const _e1122 = WF.h("div", { className: "wf-sidebar__scrim", hidden: true });
  const _e1123 = WF.h("button", { className: "wf-sidebar__toggle", type: "button", "aria-label": "Open navigation", "aria-expanded": "false", "aria-controls": "wf-sidebar-1111" }, "\u2630");
  _e1110.appendChild(_e1122);
  _e1110.appendChild(_e1123);
  WF.offCanvas(_e1111, _e1123, _e1122);
  const _e1124 = WF.h("div", { className: "wf-stack" });
  const _e1125 = WF.h("p", { className: "wf-text wf-text--muted" }, "The sidebar renders with proper structure:");
  _e1124.appendChild(_e1125);
  const _e1126 = WF.h("p", { className: "wf-text wf-text--small" }, "Sidebar.Header for the title");
  _e1124.appendChild(_e1126);
  const _e1127 = WF.h("p", { className: "wf-text wf-text--small" }, "Sidebar.Item with to: and icon: props");
  _e1124.appendChild(_e1127);
  const _e1128 = WF.h("p", { className: "wf-text wf-text--small" }, "Sidebar.Divider for separation");
  _e1124.appendChild(_e1128);
  _e1110.appendChild(_e1124);
  _e1109.appendChild(_e1110);
  _e1108.appendChild(_e1109);
  _e758.appendChild(_e1108);
  const _e1129 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1129);
  const _e1130 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1131 = WF.h("div", { className: "wf-card__body" });
  const _e1132 = WF.h("code", { className: "wf-code wf-code--block" }, "Sidebar {\n    Sidebar.Header { Text(\"My App\", heading) }\n    Sidebar.Item(to: \"/\", icon: \"home\") { Text(\"Dashboard\") }\n    Sidebar.Item(icon: \"settings\") { Text(\"Settings\") }\n    Sidebar.Divider()\n    Sidebar.Item(icon: \"logout\") { Text(\"Logout\") }\n}");
  _e1131.appendChild(_e1132);
  _e1130.appendChild(_e1131);
  _e758.appendChild(_e1130);
  const _e1133 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1133);
  const _e1134 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e1134);
  const _e1135 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1135);
  const _e1136 = WF.h("h2", { className: "wf-heading" }, "Breadcrumb");
  _e758.appendChild(_e1136);
  const _e1137 = WF.h("p", { className: "wf-text" }, "Show navigation hierarchy with automatic separators.");
  _e758.appendChild(_e1137);
  const _e1138 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1138);
  const _e1139 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1140 = WF.h("div", { className: "wf-card__body" });
  const _e1141 = WF.h("nav", { className: "wf-breadcrumb", "aria-label": "breadcrumb" });
  const _e1142 = WF.h("a", { className: "wf-breadcrumb__item", href: WF._basePath + "/" });
  const _e1143 = WF.h("p", { className: "wf-text" }, "Home");
  _e1142.appendChild(_e1143);
  _e1141.appendChild(_e1142);
  const _e1144 = WF.h("a", { className: "wf-breadcrumb__item", href: WF._basePath + "/components" });
  const _e1145 = WF.h("p", { className: "wf-text" }, "Components");
  _e1144.appendChild(_e1145);
  _e1141.appendChild(_e1144);
  const _e1146 = WF.h("span", { className: "wf-breadcrumb__item" });
  const _e1147 = WF.h("p", { className: "wf-text" }, "Breadcrumb");
  _e1146.appendChild(_e1147);
  _e1141.appendChild(_e1146);
  _e1140.appendChild(_e1141);
  _e1139.appendChild(_e1140);
  _e758.appendChild(_e1139);
  const _e1148 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1148);
  const _e1149 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1150 = WF.h("div", { className: "wf-card__body" });
  const _e1151 = WF.h("code", { className: "wf-code wf-code--block" }, "Breadcrumb {\n    Breadcrumb.Item(to: \"/\") { Text(\"Home\") }\n    Breadcrumb.Item(to: \"/docs\") { Text(\"Docs\") }\n    Breadcrumb.Item { Text(\"Current Page\") }\n}");
  _e1150.appendChild(_e1151);
  _e1149.appendChild(_e1150);
  _e758.appendChild(_e1149);
  const _e1152 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1152);
  const _e1153 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e1153);
  const _e1154 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1154);
  const _e1155 = WF.h("h2", { className: "wf-heading" }, "Skeleton Loading");
  _e758.appendChild(_e1155);
  const _e1156 = WF.h("p", { className: "wf-text" }, "Placeholder shapes that shimmer while content loads.");
  _e758.appendChild(_e1156);
  const _e1157 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1157);
  const _e1158 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1159 = WF.h("div", { className: "wf-card__body" });
  const _e1160 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1161 = WF.h("div", { className: "wf-skeleton" });
  _e1161.style.height = "16px";
  _e1161.style.width = "80%";
  _e1160.appendChild(_e1161);
  const _e1162 = WF.h("div", { className: "wf-skeleton" });
  _e1162.style.height = "16px";
  _e1162.style.width = "60%";
  _e1160.appendChild(_e1162);
  const _e1163 = WF.h("div", { className: "wf-skeleton" });
  _e1163.style.height = "16px";
  _e1163.style.width = "40%";
  _e1160.appendChild(_e1163);
  const _e1164 = WF.h("div", { className: "wf-spacer" });
  _e1160.appendChild(_e1164);
  const _e1165 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1166 = WF.h("div", { className: "wf-skeleton" });
  _e1165.appendChild(_e1166);
  const _e1167 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1168 = WF.h("div", { className: "wf-skeleton" });
  _e1168.style.height = "14px";
  _e1168.style.width = "120px";
  _e1167.appendChild(_e1168);
  const _e1169 = WF.h("div", { className: "wf-skeleton" });
  _e1169.style.height = "12px";
  _e1169.style.width = "80px";
  _e1167.appendChild(_e1169);
  _e1165.appendChild(_e1167);
  _e1160.appendChild(_e1165);
  _e1159.appendChild(_e1160);
  _e1158.appendChild(_e1159);
  _e758.appendChild(_e1158);
  const _e1170 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1170);
  const _e1171 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1172 = WF.h("div", { className: "wf-card__body" });
  const _e1173 = WF.h("code", { className: "wf-code wf-code--block" }, "Skeleton(height: \"16px\", width: \"80%\")\nSkeleton(circle, size: \"48px\")");
  _e1172.appendChild(_e1173);
  _e1171.appendChild(_e1172);
  _e758.appendChild(_e1171);
  const _e1174 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1174);
  const _e1175 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e1175);
  const _e1176 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1176);
  const _e1177 = WF.h("h2", { className: "wf-heading" }, "Dropdown & Menu");
  _e758.appendChild(_e1177);
  const _e1178 = WF.h("p", { className: "wf-text" }, "Click-to-toggle dropdown menus with auto-close on outside click.");
  _e758.appendChild(_e1178);
  const _e1179 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1179);
  const _e1180 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1181 = WF.h("div", { className: "wf-card__body" });
  const _e1182 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e1184 = WF.signal(false);
  const _e1183 = WF.h("div", { className: () => _e1184() ? "wf-dropdown open" : "wf-dropdown" });
  const _e1185 = WF.h("button", { className: "wf-btn", type: "button",              "aria-haspopup": "true", "aria-controls": "wf-menu-1183",              "aria-expanded": () => _e1184() ? "true" : "false",              "on:click": () => _e1184.set(!_e1184()) }, "Actions");
  _e1183.appendChild(_e1185);
  const _e1186 = WF.h("div", { className: "wf-dropdown__items", id: "wf-menu-1183", role: "menu" });
  const _e1187 = WF.h("li", { className: "wf-dropdown__item" });
  const _e1188 = WF.h("p", { className: "wf-text" }, "Edit");
  _e1187.appendChild(_e1188);
  _e1186.appendChild(_e1187);
  const _e1189 = WF.h("li", { className: "wf-dropdown__item" });
  const _e1190 = WF.h("p", { className: "wf-text" }, "Duplicate");
  _e1189.appendChild(_e1190);
  _e1186.appendChild(_e1189);
  const _e1191 = WF.h("div", { className: "wf-dropdown__divider" });
  _e1186.appendChild(_e1191);
  const _e1192 = WF.h("li", { className: "wf-dropdown__item" });
  const _e1193 = WF.h("p", { className: "wf-text" }, "Delete");
  _e1192.appendChild(_e1193);
  _e1186.appendChild(_e1192);
  _e1183.appendChild(_e1186);
  WF.bindPopup(_e1183, _e1185, _e1184);
  _e1182.appendChild(_e1183);
  const _e1195 = WF.signal(false);
  const _e1194 = WF.h("div", { className: () => _e1195() ? "wf-menu open" : "wf-menu" });
  const _e1196 = WF.h("button", { className: "wf-btn", type: "button",              "aria-haspopup": "true", "aria-controls": "wf-menu-1194",              "aria-expanded": () => _e1195() ? "true" : "false",              "on:click": () => _e1195.set(!_e1195()) }, "Options");
  _e1194.appendChild(_e1196);
  const _e1197 = WF.h("div", { className: "wf-menu__items", id: "wf-menu-1194", role: "menu" });
  const _e1198 = WF.h("li", { className: "wf-menu__item" });
  const _e1199 = WF.h("p", { className: "wf-text" }, "Profile");
  _e1198.appendChild(_e1199);
  _e1197.appendChild(_e1198);
  const _e1200 = WF.h("li", { className: "wf-menu__item" });
  const _e1201 = WF.h("p", { className: "wf-text" }, "Settings");
  _e1200.appendChild(_e1201);
  _e1197.appendChild(_e1200);
  const _e1202 = WF.h("div", { className: "wf-menu__divider" });
  _e1197.appendChild(_e1202);
  const _e1203 = WF.h("li", { className: "wf-menu__item" });
  const _e1204 = WF.h("p", { className: "wf-text" }, "Logout");
  _e1203.appendChild(_e1204);
  _e1197.appendChild(_e1203);
  _e1194.appendChild(_e1197);
  WF.bindPopup(_e1194, _e1196, _e1195);
  _e1182.appendChild(_e1194);
  _e1181.appendChild(_e1182);
  _e1180.appendChild(_e1181);
  _e758.appendChild(_e1180);
  const _e1205 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1205);
  const _e1206 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1207 = WF.h("div", { className: "wf-card__body" });
  const _e1208 = WF.h("code", { className: "wf-code wf-code--block" }, "Dropdown(label: \"Actions\") {\n    Dropdown.Item { Text(\"Edit\") }\n    Dropdown.Divider()\n    Dropdown.Item { Text(\"Delete\") }\n}");
  _e1207.appendChild(_e1208);
  _e1206.appendChild(_e1207);
  _e758.appendChild(_e1206);
  const _e1209 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1209);
  const _e1210 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e1210);
  const _e1211 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1211);
  const _e1212 = WF.h("h2", { className: "wf-heading" }, "Navigation");
  _e758.appendChild(_e1212);
  const _e1213 = WF.h("p", { className: "wf-text" }, "Tabs let you switch between content panels.");
  _e758.appendChild(_e1213);
  const _e1214 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1214);
  const _e1215 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1216 = WF.h("div", { className: "wf-card__body" });
  const _e1217 = WF.h("div", { className: "wf-tabs" });
  const _e1218 = WF.h("div", { className: "wf-tabs__nav", role: "tablist" });
  const _e1219 = WF.signal(0);
  const _e1220 = WF.h("button", { className: () => _e1219() === 0 ? "wf-tabs__tab active" : "wf-tabs__tab",                  role: "tab", type: "button", id: "wf-tab-1217-0",                  "aria-controls": "wf-tabpanel-1217-0",                  "aria-selected": () => _e1219() === 0 ? "true" : "false",                  tabindex: () => _e1219() === 0 ? 0 : -1,                  "on:click": () => _e1219.set(0) }, "Profile");
  _e1218.appendChild(_e1220);
  const _e1221 = WF.h("button", { className: () => _e1219() === 1 ? "wf-tabs__tab active" : "wf-tabs__tab",                  role: "tab", type: "button", id: "wf-tab-1217-1",                  "aria-controls": "wf-tabpanel-1217-1",                  "aria-selected": () => _e1219() === 1 ? "true" : "false",                  tabindex: () => _e1219() === 1 ? 0 : -1,                  "on:click": () => _e1219.set(1) }, "Settings");
  _e1218.appendChild(_e1221);
  const _e1222 = WF.h("button", { className: () => _e1219() === 2 ? "wf-tabs__tab active" : "wf-tabs__tab",                  role: "tab", type: "button", id: "wf-tab-1217-2",                  "aria-controls": "wf-tabpanel-1217-2",                  "aria-selected": () => _e1219() === 2 ? "true" : "false",                  tabindex: () => _e1219() === 2 ? 0 : -1,                  "on:click": () => _e1219.set(2) }, "About");
  _e1218.appendChild(_e1222);
  _e1217.appendChild(_e1218);
  const _e1223 = WF.h("div", { className: "wf-tab-page", role: "tabpanel",                  id: "wf-tabpanel-1217-0", "aria-labelledby": "wf-tab-1217-0", tabindex: 0 });
  const _e1224 = WF.h("div", { className: "wf-spacer" });
  _e1223.appendChild(_e1224);
  const _e1225 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1226 = WF.h("div", { className: "wf-avatar wf-avatar--primary wf-avatar--large" }, "MO");
  _e1225.appendChild(_e1226);
  const _e1227 = WF.h("div", { className: "wf-stack" });
  const _e1228 = WF.h("p", { className: "wf-text wf-text--bold" }, "Monzer Omer");
  _e1227.appendChild(_e1228);
  const _e1229 = WF.h("p", { className: "wf-text wf-text--muted" }, "Creator of WebFluent");
  _e1227.appendChild(_e1229);
  _e1225.appendChild(_e1227);
  _e1223.appendChild(_e1225);
  WF.effect(() => { _e1223.style.display = _e1219() === 0 ? 'block' : 'none'; });
  _e1217.appendChild(_e1223);
  const _e1230 = WF.h("div", { className: "wf-tab-page", role: "tabpanel",                  id: "wf-tabpanel-1217-1", "aria-labelledby": "wf-tab-1217-1", tabindex: 0 });
  const _e1231 = WF.h("div", { className: "wf-spacer" });
  _e1230.appendChild(_e1231);
  const _e1232 = WF.h("label", { className: "wf-switch" });
  const _e1233 = WF.h("input", { type: "checkbox", role: "switch",                  checked: () => _switchVal(), "aria-checked": () => _switchVal() ? "true" : "false",                  "on:change": () => _switchVal.set(!_switchVal()) });
  _e1232.appendChild(_e1233);
  const _e1234 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1232.appendChild(_e1234);
  _e1232.appendChild(WF.text("Enable notifications"));
  _e1230.appendChild(_e1232);
  const _e1235 = WF.h("div", { className: "wf-spacer" });
  _e1230.appendChild(_e1235);
  const _e1236 = WF.h("div", { className: "wf-slider" });
  const _e1237 = WF.h("label", { className: "wf-form-label" }, "Volume");
  _e1236.appendChild(_e1237);
  const _e1238 = WF.h("input", { type: "range", min: 0, max: 100, step: 1, value: () => _sliderVal(), "on:input": (e) => _sliderVal.set(Number(e.target.value)) });
  _e1236.appendChild(_e1238);
  const _e1239 = WF.h("span", { className: "wf-slider__value" }, () => String(_sliderVal()));
  _e1236.appendChild(_e1239);
  _e1230.appendChild(_e1236);
  WF.effect(() => { _e1230.style.display = _e1219() === 1 ? 'block' : 'none'; });
  _e1217.appendChild(_e1230);
  const _e1240 = WF.h("div", { className: "wf-tab-page", role: "tabpanel",                  id: "wf-tabpanel-1217-2", "aria-labelledby": "wf-tab-1217-2", tabindex: 0 });
  const _e1241 = WF.h("div", { className: "wf-spacer" });
  _e1240.appendChild(_e1241);
  const _e1242 = WF.h("p", { className: "wf-text" }, "WebFluent is a web-first programming language.");
  _e1240.appendChild(_e1242);
  const _e1243 = WF.h("p", { className: "wf-text wf-text--muted" }, "It compiles to HTML, CSS, and JavaScript.");
  _e1240.appendChild(_e1243);
  WF.effect(() => { _e1240.style.display = _e1219() === 2 ? 'block' : 'none'; });
  _e1217.appendChild(_e1240);
  WF.tablist(_e1218, _e1219);
  _e1216.appendChild(_e1217);
  _e1215.appendChild(_e1216);
  _e758.appendChild(_e1215);
  const _e1244 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1244);
  const _e1245 = WF.h("hr", { className: "wf-divider" });
  _e758.appendChild(_e1245);
  const _e1246 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1246);
  const _e1247 = WF.h("h2", { className: "wf-heading" }, "Typography");
  _e758.appendChild(_e1247);
  const _e1248 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1248);
  const _e1249 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1250 = WF.h("div", { className: "wf-card__body" });
  const _e1251 = WF.h("h2", { className: "wf-heading" }, "Heading h2");
  _e1250.appendChild(_e1251);
  const _e1252 = WF.h("h2", { className: "wf-heading" }, "Heading h2");
  _e1250.appendChild(_e1252);
  const _e1253 = WF.h("h3", { className: "wf-heading" }, "Heading h3");
  _e1250.appendChild(_e1253);
  const _e1254 = WF.h("div", { className: "wf-spacer" });
  _e1250.appendChild(_e1254);
  const _e1255 = WF.h("p", { className: "wf-text" }, "Normal text paragraph.");
  _e1250.appendChild(_e1255);
  const _e1256 = WF.h("p", { className: "wf-text wf-text--bold" }, "Bold text.");
  _e1250.appendChild(_e1256);
  const _e1257 = WF.h("p", { className: "wf-text wf-text--muted" }, "Muted text.");
  _e1250.appendChild(_e1257);
  const _e1258 = WF.h("p", { className: "wf-text wf-text--primary" }, "Primary colored.");
  _e1250.appendChild(_e1258);
  const _e1259 = WF.h("p", { className: "wf-text wf-text--danger" }, "Danger colored.");
  _e1250.appendChild(_e1259);
  const _e1260 = WF.h("p", { className: "wf-text wf-text--small" }, "Small text.");
  _e1250.appendChild(_e1260);
  const _e1261 = WF.h("p", { className: "wf-text wf-text--uppercase" }, "Uppercase.");
  _e1250.appendChild(_e1261);
  const _e1262 = WF.h("p", { className: "wf-text wf-text--center" }, "Centered text.");
  _e1250.appendChild(_e1262);
  const _e1263 = WF.h("div", { className: "wf-spacer" });
  _e1250.appendChild(_e1263);
  const _e1264 = WF.h("blockquote", { className: "wf-blockquote" }, "The best way to predict the future is to create it.");
  _e1250.appendChild(_e1264);
  const _e1265 = WF.h("div", { className: "wf-spacer" });
  _e1250.appendChild(_e1265);
  const _e1266 = WF.h("code", { className: "wf-code" }, "const greeting = \"Hello, WebFluent!\";");
  _e1250.appendChild(_e1266);
  _e1249.appendChild(_e1250);
  _e758.appendChild(_e1249);
  const _e1267 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1267);
  const _e1268 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1269 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e1268.appendChild(_e1269);
  const _e1270 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/animation"); } }, "Animation System");
  _e1268.appendChild(_e1270);
  _e758.appendChild(_e1268);
  const _e1271 = WF.h("div", { className: "wf-spacer" });
  _e758.appendChild(_e1271);
  _root.appendChild(_e758);
  return _root;
}

function Page_Cli(params) {
  const _root = document.createDocumentFragment();
  const _e1272 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1273 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1273);
  const _e1274 = WF.h("h1", { className: "wf-heading" }, "CLI Reference");
  _e1272.appendChild(_e1274);
  const _e1275 = WF.h("p", { className: "wf-text wf-text--muted" }, "The WebFluent command-line interface.");
  _e1272.appendChild(_e1275);
  const _e1276 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1276);
  const _e1277 = WF.h("h2", { className: "wf-heading" }, "wf init");
  _e1272.appendChild(_e1277);
  const _e1278 = WF.h("p", { className: "wf-text" }, "Create a new WebFluent project.");
  _e1272.appendChild(_e1278);
  const _e1279 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1279);
  const _e1280 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1281 = WF.h("div", { className: "wf-card__body" });
  const _e1282 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init <name> [--template spa|static|pdf]");
  _e1281.appendChild(_e1282);
  _e1280.appendChild(_e1281);
  _e1272.appendChild(_e1280);
  const _e1283 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1283);
  const _e1284 = WF.h("table", { className: "wf-table" });
  const _e1285 = WF.h("thead", {});
  const _e1286 = WF.h("td", {}, "Argument");
  _e1285.appendChild(_e1286);
  const _e1287 = WF.h("td", {}, "Description");
  _e1285.appendChild(_e1287);
  _e1284.appendChild(_e1285);
  const _e1288 = WF.h("tr", {});
  const _e1289 = WF.h("td", {}, "name");
  _e1288.appendChild(_e1289);
  const _e1290 = WF.h("td", {}, "Project name (creates a directory)");
  _e1288.appendChild(_e1290);
  _e1284.appendChild(_e1288);
  const _e1291 = WF.h("tr", {});
  const _e1292 = WF.h("td", {}, "--template, -t");
  _e1291.appendChild(_e1292);
  const _e1293 = WF.h("td", {}, "Template: spa (default), static, or pdf");
  _e1291.appendChild(_e1293);
  _e1284.appendChild(_e1291);
  _e1272.appendChild(_e1284);
  const _e1294 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1294);
  const _e1295 = WF.h("p", { className: "wf-text wf-text--muted" }, "SPA: interactive app with routing and state. Static: SSG site with i18n. PDF: document generation with tables, headings, and auto page breaks.");
  _e1272.appendChild(_e1295);
  const _e1296 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1296);
  const _e1297 = WF.h("hr", { className: "wf-divider" });
  _e1272.appendChild(_e1297);
  const _e1298 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1298);
  const _e1299 = WF.h("h2", { className: "wf-heading" }, "wf build");
  _e1272.appendChild(_e1299);
  const _e1300 = WF.h("p", { className: "wf-text" }, "Compile .wf files to HTML, CSS, and JavaScript.");
  _e1272.appendChild(_e1300);
  const _e1301 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1301);
  const _e1302 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1303 = WF.h("div", { className: "wf-card__body" });
  const _e1304 = WF.h("code", { className: "wf-code wf-code--block" }, "wf build [--dir DIR]");
  _e1303.appendChild(_e1304);
  _e1302.appendChild(_e1303);
  _e1272.appendChild(_e1302);
  const _e1305 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1305);
  const _e1306 = WF.h("table", { className: "wf-table" });
  const _e1307 = WF.h("thead", {});
  const _e1308 = WF.h("td", {}, "Option");
  _e1307.appendChild(_e1308);
  const _e1309 = WF.h("td", {}, "Description");
  _e1307.appendChild(_e1309);
  _e1306.appendChild(_e1307);
  const _e1310 = WF.h("tr", {});
  const _e1311 = WF.h("td", {}, "--dir, -d");
  _e1310.appendChild(_e1311);
  const _e1312 = WF.h("td", {}, "Project directory (default: current directory)");
  _e1310.appendChild(_e1312);
  _e1306.appendChild(_e1310);
  _e1272.appendChild(_e1306);
  const _e1313 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1313);
  const _e1314 = WF.h("p", { className: "wf-text wf-text--muted" }, "The build pipeline: Lex all .wf files, parse to AST, run accessibility linter, generate HTML + CSS + JS, write to output directory.");
  _e1272.appendChild(_e1314);
  const _e1315 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1315);
  const _e1316 = WF.h("p", { className: "wf-text" }, "Output depends on config:");
  _e1272.appendChild(_e1316);
  const _e1317 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1318 = WF.h("p", { className: "wf-text" }, "SPA mode (default): single index.html + app.js + styles.css");
  _e1317.appendChild(_e1318);
  const _e1319 = WF.h("p", { className: "wf-text" }, "SSG mode (ssg: true): one HTML per page + app.js + styles.css");
  _e1317.appendChild(_e1319);
  const _e1320 = WF.h("p", { className: "wf-text" }, "PDF mode (output_type: pdf): a single .pdf file");
  _e1317.appendChild(_e1320);
  _e1272.appendChild(_e1317);
  const _e1321 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1321);
  const _e1322 = WF.h("hr", { className: "wf-divider" });
  _e1272.appendChild(_e1322);
  const _e1323 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1323);
  const _e1324 = WF.h("h2", { className: "wf-heading" }, "wf serve");
  _e1272.appendChild(_e1324);
  const _e1325 = WF.h("p", { className: "wf-text" }, "Start a development server that serves the built output.");
  _e1272.appendChild(_e1325);
  const _e1326 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1326);
  const _e1327 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1328 = WF.h("div", { className: "wf-card__body" });
  const _e1329 = WF.h("code", { className: "wf-code wf-code--block" }, "wf serve [--dir DIR]");
  _e1328.appendChild(_e1329);
  _e1327.appendChild(_e1328);
  _e1272.appendChild(_e1327);
  const _e1330 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1330);
  const _e1331 = WF.h("p", { className: "wf-text wf-text--muted" }, "Serves files from the build directory. SPA fallback: all routes serve index.html so client-side routing works. Port is configured in webfluent.app.json (default: 3000).");
  _e1272.appendChild(_e1331);
  const _e1332 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1332);
  const _e1333 = WF.h("hr", { className: "wf-divider" });
  _e1272.appendChild(_e1333);
  const _e1334 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1334);
  const _e1335 = WF.h("h2", { className: "wf-heading" }, "wf generate");
  _e1272.appendChild(_e1335);
  const _e1336 = WF.h("p", { className: "wf-text" }, "Scaffold a new page, component, or store.");
  _e1272.appendChild(_e1336);
  const _e1337 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1337);
  const _e1338 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1339 = WF.h("div", { className: "wf-card__body" });
  const _e1340 = WF.h("code", { className: "wf-code wf-code--block" }, "wf generate <kind> <name> [--dir DIR]");
  _e1339.appendChild(_e1340);
  _e1338.appendChild(_e1339);
  _e1272.appendChild(_e1338);
  const _e1341 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1341);
  const _e1342 = WF.h("table", { className: "wf-table" });
  const _e1343 = WF.h("thead", {});
  const _e1344 = WF.h("td", {}, "Kind");
  _e1343.appendChild(_e1344);
  const _e1345 = WF.h("td", {}, "Creates");
  _e1343.appendChild(_e1345);
  const _e1346 = WF.h("td", {}, "Example");
  _e1343.appendChild(_e1346);
  _e1342.appendChild(_e1343);
  const _e1347 = WF.h("tr", {});
  const _e1348 = WF.h("td", {}, "page");
  _e1347.appendChild(_e1348);
  const _e1349 = WF.h("td", {}, "src/pages/Name.wf");
  _e1347.appendChild(_e1349);
  const _e1350 = WF.h("td", {}, "wf generate page About");
  _e1347.appendChild(_e1350);
  _e1342.appendChild(_e1347);
  const _e1351 = WF.h("tr", {});
  const _e1352 = WF.h("td", {}, "component");
  _e1351.appendChild(_e1352);
  const _e1353 = WF.h("td", {}, "src/components/Name.wf");
  _e1351.appendChild(_e1353);
  const _e1354 = WF.h("td", {}, "wf generate component Header");
  _e1351.appendChild(_e1354);
  _e1342.appendChild(_e1351);
  const _e1355 = WF.h("tr", {});
  const _e1356 = WF.h("td", {}, "store");
  _e1355.appendChild(_e1356);
  const _e1357 = WF.h("td", {}, "src/stores/name.wf");
  _e1355.appendChild(_e1357);
  const _e1358 = WF.h("td", {}, "wf generate store CartStore");
  _e1355.appendChild(_e1358);
  _e1342.appendChild(_e1355);
  _e1272.appendChild(_e1342);
  const _e1359 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1359);
  const _e1360 = WF.h("hr", { className: "wf-divider" });
  _e1272.appendChild(_e1360);
  const _e1361 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1361);
  const _e1362 = WF.h("h2", { className: "wf-heading" }, "Configuration");
  _e1272.appendChild(_e1362);
  const _e1363 = WF.h("p", { className: "wf-text" }, "All config is in webfluent.app.json at the project root.");
  _e1272.appendChild(_e1363);
  const _e1364 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1364);
  const _e1365 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1366 = WF.h("div", { className: "wf-card__body" });
  const _e1367 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"name\": \"My App\",\n  \"version\": \"1.0.0\",\n  \"author\": \"Your Name\",\n  \"theme\": {\n    \"name\": \"default\",\n    \"mode\": \"light\",\n    \"tokens\": { \"color-primary\": \"#6366F1\" }\n  },\n  \"build\": {\n    \"output\": \"./build\",\n    \"minify\": true,\n    \"ssg\": false,\n    \"output_type\": \"spa\",\n    \"pdf\": {\n      \"page_size\": \"A4\",\n      \"default_font\": \"Helvetica\",\n      \"output_filename\": \"report.pdf\"\n    }\n  },\n  \"dev\": { \"port\": 3000 },\n  \"meta\": {\n    \"title\": \"My App\",\n    \"description\": \"Built with WebFluent\",\n    \"lang\": \"en\"\n  },\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e1366.appendChild(_e1367);
  _e1365.appendChild(_e1366);
  _e1272.appendChild(_e1365);
  const _e1368 = WF.h("div", { className: "wf-spacer" });
  _e1272.appendChild(_e1368);
  _root.appendChild(_e1272);
  return _root;
}

function Page_I18n(params) {
  const _root = document.createDocumentFragment();
  const _e1369 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1370 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1370);
  const _e1371 = WF.h("h1", { className: "wf-heading" }, "Internationalization (i18n)");
  _e1369.appendChild(_e1371);
  const _e1372 = WF.h("p", { className: "wf-text wf-text--muted" }, "Built-in multi-language support with reactive locale switching and automatic RTL.");
  _e1369.appendChild(_e1372);
  const _e1373 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1373);
  const _e1374 = WF.h("h2", { className: "wf-heading" }, "Setup");
  _e1369.appendChild(_e1374);
  const _e1375 = WF.h("p", { className: "wf-text" }, "Create a JSON file per locale in your translations directory.");
  _e1369.appendChild(_e1375);
  const _e1376 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1376);
  const _e1377 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1378 = WF.h("div", { className: "wf-card__body" });
  const _e1379 = WF.h("code", { className: "wf-code wf-code--block" }, "// src/translations/en.json\n{\n    \"greeting\": \"Hello, {name}!\",\n    \"nav.home\": \"Home\",\n    \"nav.about\": \"About\"\n}\n\n// src/translations/ar.json\n{\n    \"greeting\": \"!أهلاً، {name}\",\n    \"nav.home\": \"الرئيسية\",\n    \"nav.about\": \"حول\"\n}");
  _e1378.appendChild(_e1379);
  _e1377.appendChild(_e1378);
  _e1369.appendChild(_e1377);
  const _e1380 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1380);
  const _e1381 = WF.h("p", { className: "wf-text wf-text--bold" }, "Add i18n config to webfluent.app.json:");
  _e1369.appendChild(_e1381);
  const _e1382 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1383 = WF.h("div", { className: "wf-card__body" });
  const _e1384 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e1383.appendChild(_e1384);
  _e1382.appendChild(_e1383);
  _e1369.appendChild(_e1382);
  const _e1385 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1385);
  const _e1386 = WF.h("hr", { className: "wf-divider" });
  _e1369.appendChild(_e1386);
  const _e1387 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1387);
  const _e1388 = WF.h("h2", { className: "wf-heading" }, "The t() Function");
  _e1369.appendChild(_e1388);
  const _e1389 = WF.h("p", { className: "wf-text" }, "Use t() to look up translated text. It is reactive — all t() calls update when the locale changes.");
  _e1369.appendChild(_e1389);
  const _e1390 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1390);
  const _e1391 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1392 = WF.h("div", { className: "wf-card__body" });
  const _e1393 = WF.h("code", { className: "wf-code wf-code--block" }, "// Simple key lookup\nText(t(\"nav.home\"))\n\n// With interpolation\nText(t(\"greeting\", name: user.name))\n\n// In any component\nButton(t(\"actions.save\"), primary)\nHeading(t(\"page.title\"), h1)");
  _e1392.appendChild(_e1393);
  _e1391.appendChild(_e1392);
  _e1369.appendChild(_e1391);
  const _e1394 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1394);
  const _e1395 = WF.h("hr", { className: "wf-divider" });
  _e1369.appendChild(_e1395);
  const _e1396 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1396);
  const _e1397 = WF.h("h2", { className: "wf-heading" }, "Locale Switching");
  _e1369.appendChild(_e1397);
  const _e1398 = WF.h("p", { className: "wf-text" }, "Switch the locale at runtime with setLocale(). All translated text updates instantly.");
  _e1369.appendChild(_e1398);
  const _e1399 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1399);
  const _e1400 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1401 = WF.h("div", { className: "wf-card__body" });
  const _e1402 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"English\") { setLocale(\"en\") }\nButton(\"العربية\") { setLocale(\"ar\") }\nButton(\"Espanol\") { setLocale(\"es\") }\n\n// Access current locale\nText(\"Current: {locale}\")\nText(\"Direction: {dir}\")");
  _e1401.appendChild(_e1402);
  _e1400.appendChild(_e1401);
  _e1369.appendChild(_e1400);
  const _e1403 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1403);
  const _e1404 = WF.h("hr", { className: "wf-divider" });
  _e1369.appendChild(_e1404);
  const _e1405 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1405);
  const _e1406 = WF.h("h2", { className: "wf-heading" }, "RTL Support");
  _e1369.appendChild(_e1406);
  const _e1407 = WF.h("p", { className: "wf-text" }, "WebFluent automatically detects RTL locales and updates the document direction.");
  _e1369.appendChild(_e1407);
  const _e1408 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1408);
  const _e1409 = WF.h("table", { className: "wf-table" });
  const _e1410 = WF.h("thead", {});
  const _e1411 = WF.h("td", {}, "Locale");
  _e1410.appendChild(_e1411);
  const _e1412 = WF.h("td", {}, "Direction");
  _e1410.appendChild(_e1412);
  _e1409.appendChild(_e1410);
  const _e1413 = WF.h("tr", {});
  const _e1414 = WF.h("td", {}, "ar (Arabic)");
  _e1413.appendChild(_e1414);
  const _e1415 = WF.h("td", {}, "RTL");
  _e1413.appendChild(_e1415);
  _e1409.appendChild(_e1413);
  const _e1416 = WF.h("tr", {});
  const _e1417 = WF.h("td", {}, "he (Hebrew)");
  _e1416.appendChild(_e1417);
  const _e1418 = WF.h("td", {}, "RTL");
  _e1416.appendChild(_e1418);
  _e1409.appendChild(_e1416);
  const _e1419 = WF.h("tr", {});
  const _e1420 = WF.h("td", {}, "fa (Farsi)");
  _e1419.appendChild(_e1420);
  const _e1421 = WF.h("td", {}, "RTL");
  _e1419.appendChild(_e1421);
  _e1409.appendChild(_e1419);
  const _e1422 = WF.h("tr", {});
  const _e1423 = WF.h("td", {}, "ur (Urdu)");
  _e1422.appendChild(_e1423);
  const _e1424 = WF.h("td", {}, "RTL");
  _e1422.appendChild(_e1424);
  _e1409.appendChild(_e1422);
  const _e1425 = WF.h("tr", {});
  const _e1426 = WF.h("td", {}, "All others");
  _e1425.appendChild(_e1426);
  const _e1427 = WF.h("td", {}, "LTR");
  _e1425.appendChild(_e1427);
  _e1409.appendChild(_e1425);
  _e1369.appendChild(_e1409);
  const _e1428 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1428);
  const _e1429 = WF.h("p", { className: "wf-text wf-text--muted" }, "When setLocale(\"ar\") is called, the HTML element gets dir=\"rtl\" and lang=\"ar\" automatically.");
  _e1369.appendChild(_e1429);
  const _e1430 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1430);
  const _e1431 = WF.h("hr", { className: "wf-divider" });
  _e1369.appendChild(_e1431);
  const _e1432 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1432);
  const _e1433 = WF.h("h2", { className: "wf-heading" }, "Fallback Behavior");
  _e1369.appendChild(_e1433);
  const _e1434 = WF.h("p", { className: "wf-text" }, "If a key is missing in the current locale:");
  _e1369.appendChild(_e1434);
  const _e1435 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1436 = WF.h("p", { className: "wf-text" }, "1. Falls back to the defaultLocale translation");
  _e1435.appendChild(_e1436);
  const _e1437 = WF.h("p", { className: "wf-text" }, "2. If still missing, returns the key itself (e.g., \"nav.home\")");
  _e1435.appendChild(_e1437);
  _e1369.appendChild(_e1435);
  const _e1438 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1438);
  const _e1439 = WF.h("hr", { className: "wf-divider" });
  _e1369.appendChild(_e1439);
  const _e1440 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1440);
  const _e1441 = WF.h("h2", { className: "wf-heading" }, "SSG + i18n");
  _e1369.appendChild(_e1441);
  const _e1442 = WF.h("p", { className: "wf-text wf-text--muted" }, "When both SSG and i18n are enabled, pages are pre-rendered with the default locale text. After JavaScript loads, locale switching works normally.");
  _e1369.appendChild(_e1442);
  const _e1443 = WF.h("div", { className: "wf-spacer" });
  _e1369.appendChild(_e1443);
  _root.appendChild(_e1369);
  return _root;
}

function Page_TemplateEngine(params) {
  const _root = document.createDocumentFragment();
  const _e1444 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1445 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1445);
  const _e1446 = WF.h("h1", { className: "wf-heading" }, () => WF.i18n.t("tpl.title"));
  _e1444.appendChild(_e1446);
  const _e1447 = WF.h("p", { className: "wf-text wf-text--muted" }, () => WF.i18n.t("tpl.subtitle"));
  _e1444.appendChild(_e1447);
  const _e1448 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1448);
  const _e1449 = WF.h("div", { className: "wf-alert wf-alert--info", role: "status" }, "WebFluent can be used as a server-side template engine from Rust and Node.js to render .wf templates into HTML or PDF with JSON data.");
  _e1444.appendChild(_e1449);
  const _e1450 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1450);
  const _e1451 = WF.h("h2", { className: "wf-heading" }, "CLI Usage");
  _e1444.appendChild(_e1451);
  const _e1452 = WF.h("p", { className: "wf-text" }, "Render any .wf template with JSON data directly from the command line.");
  _e1444.appendChild(_e1452);
  const _e1453 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1453);
  const _e1454 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1455 = WF.h("div", { className: "wf-card__body" });
  const _e1456 = WF.h("code", { className: "wf-code wf-code--block" }, "# Render to HTML\nwf render template.wf --data data.json --format html -o output.html\n\n# Render to HTML fragment (no <html> wrapper)\nwf render template.wf --data data.json --format fragment\n\n# Render to PDF\nwf render template.wf --data data.json --format pdf -o report.pdf\n\n# Pipe JSON from stdin\necho '{\"name\":\"Monzer\"}' | wf render template.wf --format html\n\n# With theme\nwf render template.wf --data data.json --format html --theme dark");
  _e1455.appendChild(_e1456);
  _e1454.appendChild(_e1455);
  _e1444.appendChild(_e1454);
  const _e1457 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1457);
  const _e1458 = WF.h("table", { className: "wf-table" });
  const _e1459 = WF.h("thead", {});
  const _e1460 = WF.h("td", {}, "Option");
  _e1459.appendChild(_e1460);
  const _e1461 = WF.h("td", {}, "Description");
  _e1459.appendChild(_e1461);
  _e1458.appendChild(_e1459);
  const _e1462 = WF.h("tr", {});
  const _e1463 = WF.h("td", {}, "template");
  _e1462.appendChild(_e1463);
  const _e1464 = WF.h("td", {}, "Path to the .wf template file");
  _e1462.appendChild(_e1464);
  _e1458.appendChild(_e1462);
  const _e1465 = WF.h("tr", {});
  const _e1466 = WF.h("td", {}, "--data");
  _e1465.appendChild(_e1466);
  const _e1467 = WF.h("td", {}, "Path to JSON data file (reads stdin if omitted)");
  _e1465.appendChild(_e1467);
  _e1458.appendChild(_e1465);
  const _e1468 = WF.h("tr", {});
  const _e1469 = WF.h("td", {}, "--format, -f");
  _e1468.appendChild(_e1469);
  const _e1470 = WF.h("td", {}, "Output format: html, fragment, or pdf");
  _e1468.appendChild(_e1470);
  _e1458.appendChild(_e1468);
  const _e1471 = WF.h("tr", {});
  const _e1472 = WF.h("td", {}, "--output, -o");
  _e1471.appendChild(_e1472);
  const _e1473 = WF.h("td", {}, "Output file path (stdout if omitted)");
  _e1471.appendChild(_e1473);
  _e1458.appendChild(_e1471);
  const _e1474 = WF.h("tr", {});
  const _e1475 = WF.h("td", {}, "--theme");
  _e1474.appendChild(_e1475);
  const _e1476 = WF.h("td", {}, "Theme name (default: \"default\")");
  _e1474.appendChild(_e1476);
  _e1458.appendChild(_e1474);
  _e1444.appendChild(_e1458);
  const _e1477 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1477);
  const _e1478 = WF.h("hr", { className: "wf-divider" });
  _e1444.appendChild(_e1478);
  const _e1479 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1479);
  const _e1480 = WF.h("h2", { className: "wf-heading" }, "Template Syntax");
  _e1444.appendChild(_e1480);
  const _e1481 = WF.h("p", { className: "wf-text" }, "Templates use standard .wf syntax. Data is passed as a JSON object — top-level keys become template variables.");
  _e1444.appendChild(_e1481);
  const _e1482 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1482);
  const _e1483 = WF.h("div", { className: "wf-grid wf-grid--gap-lg", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e1484 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1485 = WF.h("div", { className: "wf-card__header" });
  const _e1486 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Template");
  _e1485.appendChild(_e1486);
  const _e1487 = WF.h("p", { className: "wf-text wf-text--bold" }, "invoice.wf");
  _e1485.appendChild(_e1487);
  _e1484.appendChild(_e1485);
  const _e1488 = WF.h("div", { className: "wf-card__body" });
  const _e1489 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Invoice (path: \"/\", title: \"Invoice\") {\n    Container {\n        Heading(\"Invoice #{number}\", h1)\n        Text(\"Customer: {customer.name}\")\n\n        Table {\n            Thead { Trow { Tcell(\"Item\") Tcell(\"Price\") } }\n            for item in items {\n                Trow {\n                    Tcell(item.name)\n                    Tcell(\"${item.price}\")\n                }\n            }\n        }\n\n        if paid {\n            Badge(\"PAID\", success)\n        } else {\n            Badge(\"UNPAID\", danger)\n        }\n    }\n}");
  _e1488.appendChild(_e1489);
  _e1484.appendChild(_e1488);
  _e1483.appendChild(_e1484);
  const _e1490 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1491 = WF.h("div", { className: "wf-card__header" });
  const _e1492 = WF.h("span", { className: "wf-badge wf-badge--info" }, "Data");
  _e1491.appendChild(_e1492);
  const _e1493 = WF.h("p", { className: "wf-text wf-text--bold" }, "data.json");
  _e1491.appendChild(_e1493);
  _e1490.appendChild(_e1491);
  const _e1494 = WF.h("div", { className: "wf-card__body" });
  const _e1495 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"number\": \"INV-001\",\n  \"customer\": { \"name\": \"Acme Corp\" },\n  \"items\": [\n    { \"name\": \"Widget\", \"price\": 9.99 },\n    { \"name\": \"Gadget\", \"price\": 24.99 }\n  ],\n  \"paid\": true\n}");
  _e1494.appendChild(_e1495);
  _e1490.appendChild(_e1494);
  _e1483.appendChild(_e1490);
  _e1444.appendChild(_e1483);
  const _e1496 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1496);
  const _e1497 = WF.h("hr", { className: "wf-divider" });
  _e1444.appendChild(_e1497);
  const _e1498 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1498);
  const _e1499 = WF.h("h2", { className: "wf-heading" }, "Rust API");
  _e1444.appendChild(_e1499);
  const _e1500 = WF.h("p", { className: "wf-text" }, "Add WebFluent as a library dependency to use templates in your Rust application.");
  _e1444.appendChild(_e1500);
  const _e1501 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1501);
  const _e1502 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1503 = WF.h("div", { className: "wf-card__header" });
  const _e1504 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Cargo.toml");
  _e1503.appendChild(_e1504);
  _e1502.appendChild(_e1503);
  const _e1505 = WF.h("div", { className: "wf-card__body" });
  const _e1506 = WF.h("code", { className: "wf-code wf-code--block" }, "[dependencies]\nwebfluent = \"0.2\"");
  _e1505.appendChild(_e1506);
  _e1502.appendChild(_e1505);
  _e1444.appendChild(_e1502);
  const _e1507 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1507);
  const _e1508 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1509 = WF.h("div", { className: "wf-card__header" });
  const _e1510 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "main.rs");
  _e1509.appendChild(_e1510);
  _e1508.appendChild(_e1509);
  const _e1511 = WF.h("div", { className: "wf-card__body" });
  const _e1512 = WF.h("code", { className: "wf-code wf-code--block" }, "use webfluent::Template;\nuse serde_json::json;\n\nfn main() -> webfluent::Result<()> {\n    let tpl = Template::from_file(\"templates/invoice.wf\")?;\n\n    // HTML document (with embedded CSS)\n    let html = tpl.render_html(&json!({\n        \"number\": \"INV-001\",\n        \"customer\": { \"name\": \"Acme Corp\" },\n        \"items\": [{ \"name\": \"Widget\", \"price\": 9.99 }],\n        \"paid\": true\n    }))?;\n\n    // HTML fragment (no wrapper)\n    let fragment = tpl.render_html_fragment(&data)?;\n\n    // PDF bytes\n    let pdf_bytes = tpl.render_pdf(&data)?;\n    std::fs::write(\"invoice.pdf\", pdf_bytes)?;\n\n    // With custom theme\n    let dark = Template::from_file(\"invoice.wf\")?\n        .with_theme(\"dark\")\n        .with_tokens(&[(\"color-primary\", \"#8B5CF6\")]);\n    let html = dark.render_html(&data)?;\n\n    Ok(())\n}");
  _e1511.appendChild(_e1512);
  _e1508.appendChild(_e1511);
  _e1444.appendChild(_e1508);
  const _e1513 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1513);
  const _e1514 = WF.h("hr", { className: "wf-divider" });
  _e1444.appendChild(_e1514);
  const _e1515 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1515);
  const _e1516 = WF.h("h2", { className: "wf-heading" }, "Node.js API");
  _e1444.appendChild(_e1516);
  const _e1517 = WF.h("p", { className: "wf-text" }, "Use WebFluent templates in Express, Next.js, or any Node.js application.");
  _e1444.appendChild(_e1517);
  const _e1518 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1518);
  const _e1519 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1520 = WF.h("div", { className: "wf-card__header" });
  const _e1521 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Install");
  _e1520.appendChild(_e1521);
  _e1519.appendChild(_e1520);
  const _e1522 = WF.h("div", { className: "wf-card__body" });
  const _e1523 = WF.h("code", { className: "wf-code wf-code--block" }, "npm install @aspect/webfluent");
  _e1522.appendChild(_e1523);
  _e1519.appendChild(_e1522);
  _e1444.appendChild(_e1519);
  const _e1524 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1524);
  const _e1525 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1526 = WF.h("div", { className: "wf-card__header" });
  const _e1527 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Basic Usage");
  _e1526.appendChild(_e1527);
  _e1525.appendChild(_e1526);
  const _e1528 = WF.h("div", { className: "wf-card__body" });
  const _e1529 = WF.h("code", { className: "wf-code wf-code--block" }, "const { Template } = require('@aspect/webfluent');\n\nconst tpl = Template.fromFile('templates/invoice.wf');\n// or: Template.fromString('Container { Heading(\"Hello!\", h1) }');\n\n// Render to HTML\nconst html = tpl.renderHtml({ name: \"World\" });\n\n// Render to HTML fragment\nconst frag = tpl.renderHtmlFragment({ name: \"World\" });\n\n// Render to PDF (returns Buffer)\nconst pdf = tpl.renderPdf({ name: \"World\" });\nfs.writeFileSync('output.pdf', pdf);");
  _e1528.appendChild(_e1529);
  _e1525.appendChild(_e1528);
  _e1444.appendChild(_e1525);
  const _e1530 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1530);
  const _e1531 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1532 = WF.h("div", { className: "wf-card__header" });
  const _e1533 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Express.js Example");
  _e1532.appendChild(_e1533);
  _e1531.appendChild(_e1532);
  const _e1534 = WF.h("div", { className: "wf-card__body" });
  const _e1535 = WF.h("code", { className: "wf-code wf-code--block" }, "const express = require('express');\nconst { Template } = require('@aspect/webfluent');\n\nconst app = express();\n\napp.get('/invoice/:id', async (req, res) => {\n    const invoice = await db.getInvoice(req.params.id);\n    const tpl = Template.fromFile('templates/invoice.wf');\n    res.send(tpl.renderHtml(invoice));\n});\n\napp.get('/invoice/:id/pdf', async (req, res) => {\n    const invoice = await db.getInvoice(req.params.id);\n    const tpl = Template.fromFile('templates/invoice.wf');\n    res.type('application/pdf').send(tpl.renderPdf(invoice));\n});");
  _e1534.appendChild(_e1535);
  _e1531.appendChild(_e1534);
  _e1444.appendChild(_e1531);
  const _e1536 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1536);
  const _e1537 = WF.h("hr", { className: "wf-divider" });
  _e1444.appendChild(_e1537);
  const _e1538 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1538);
  const _e1539 = WF.h("h2", { className: "wf-heading" }, "Supported Features");
  _e1444.appendChild(_e1539);
  const _e1540 = WF.h("p", { className: "wf-text" }, "Templates support the static, data-driven subset of WebFluent.");
  _e1444.appendChild(_e1540);
  const _e1541 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1541);
  const _e1542 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e1543 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1544 = WF.h("div", { className: "wf-card__header" });
  const _e1545 = WF.h("h3", { className: "wf-heading" }, "Supported");
  _e1544.appendChild(_e1545);
  _e1543.appendChild(_e1544);
  const _e1546 = WF.h("div", { className: "wf-card__body" });
  const _e1547 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1548 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1549 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1548.appendChild(_e1549);
  const _e1550 = WF.h("p", { className: "wf-text" }, "All layout components");
  _e1548.appendChild(_e1550);
  _e1547.appendChild(_e1548);
  const _e1551 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1552 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1551.appendChild(_e1552);
  const _e1553 = WF.h("p", { className: "wf-text" }, "Typography (Text, Heading, Code)");
  _e1551.appendChild(_e1553);
  _e1547.appendChild(_e1551);
  const _e1554 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1555 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1554.appendChild(_e1555);
  const _e1556 = WF.h("p", { className: "wf-text" }, "Data display (Card, Table, List, Badge)");
  _e1554.appendChild(_e1556);
  _e1547.appendChild(_e1554);
  const _e1557 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1558 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1557.appendChild(_e1558);
  const _e1559 = WF.h("p", { className: "wf-text" }, "for loops over data arrays");
  _e1557.appendChild(_e1559);
  _e1547.appendChild(_e1557);
  const _e1560 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1561 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1560.appendChild(_e1561);
  const _e1562 = WF.h("p", { className: "wf-text" }, "if/else conditionals");
  _e1560.appendChild(_e1562);
  _e1547.appendChild(_e1560);
  const _e1563 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1564 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1563.appendChild(_e1564);
  const _e1565 = WF.h("p", { className: "wf-text" }, "String interpolation {var}");
  _e1563.appendChild(_e1565);
  _e1547.appendChild(_e1563);
  const _e1566 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1567 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1566.appendChild(_e1567);
  const _e1568 = WF.h("p", { className: "wf-text" }, "Nested access (user.name)");
  _e1566.appendChild(_e1568);
  _e1547.appendChild(_e1566);
  const _e1569 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1570 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1569.appendChild(_e1570);
  const _e1571 = WF.h("p", { className: "wf-text" }, "Design tokens and themes");
  _e1569.appendChild(_e1571);
  _e1547.appendChild(_e1569);
  const _e1572 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1573 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1572.appendChild(_e1573);
  const _e1574 = WF.h("p", { className: "wf-text" }, "Style blocks");
  _e1572.appendChild(_e1574);
  _e1547.appendChild(_e1572);
  const _e1575 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1576 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e1575.appendChild(_e1576);
  const _e1577 = WF.h("p", { className: "wf-text" }, "PDF components");
  _e1575.appendChild(_e1577);
  _e1547.appendChild(_e1575);
  _e1546.appendChild(_e1547);
  _e1543.appendChild(_e1546);
  _e1542.appendChild(_e1543);
  const _e1578 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1579 = WF.h("div", { className: "wf-card__header" });
  const _e1580 = WF.h("h3", { className: "wf-heading" }, "Not Supported");
  _e1579.appendChild(_e1580);
  _e1578.appendChild(_e1579);
  const _e1581 = WF.h("div", { className: "wf-card__body" });
  const _e1582 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1583 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1584 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e1583.appendChild(_e1584);
  const _e1585 = WF.h("p", { className: "wf-text" }, "state / derived / effect");
  _e1583.appendChild(_e1585);
  _e1582.appendChild(_e1583);
  const _e1586 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1587 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e1586.appendChild(_e1587);
  const _e1588 = WF.h("p", { className: "wf-text" }, "Events (on:click, on:submit)");
  _e1586.appendChild(_e1588);
  _e1582.appendChild(_e1586);
  const _e1589 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1590 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e1589.appendChild(_e1590);
  const _e1591 = WF.h("p", { className: "wf-text" }, "Navigation / Router");
  _e1589.appendChild(_e1591);
  _e1582.appendChild(_e1589);
  const _e1592 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1593 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e1592.appendChild(_e1593);
  const _e1594 = WF.h("p", { className: "wf-text" }, "Stores (shared state)");
  _e1592.appendChild(_e1594);
  _e1582.appendChild(_e1592);
  const _e1595 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1596 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e1595.appendChild(_e1596);
  const _e1597 = WF.h("p", { className: "wf-text" }, "fetch (data loading)");
  _e1595.appendChild(_e1597);
  _e1582.appendChild(_e1595);
  const _e1598 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1599 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e1598.appendChild(_e1599);
  const _e1600 = WF.h("p", { className: "wf-text" }, "Animations");
  _e1598.appendChild(_e1600);
  _e1582.appendChild(_e1598);
  const _e1601 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1602 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e1601.appendChild(_e1602);
  const _e1603 = WF.h("p", { className: "wf-text" }, "Toast (imperative)");
  _e1601.appendChild(_e1603);
  _e1582.appendChild(_e1601);
  _e1581.appendChild(_e1582);
  _e1578.appendChild(_e1581);
  _e1542.appendChild(_e1578);
  _e1444.appendChild(_e1542);
  const _e1604 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1604);
  const _e1605 = WF.h("hr", { className: "wf-divider" });
  _e1444.appendChild(_e1605);
  const _e1606 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1606);
  const _e1607 = WF.h("h2", { className: "wf-heading" }, "Use Cases");
  _e1444.appendChild(_e1607);
  const _e1608 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1608);
  const _e1609 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1610 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1611 = WF.h("div", { className: "wf-card__body" });
  const _e1612 = WF.h("h3", { className: "wf-heading" }, "Server-Rendered Pages");
  _e1611.appendChild(_e1612);
  const _e1613 = WF.h("p", { className: "wf-text wf-text--muted" }, "Generate HTML pages on the server with data from your database or API.");
  _e1611.appendChild(_e1613);
  _e1610.appendChild(_e1611);
  _e1609.appendChild(_e1610);
  const _e1614 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1615 = WF.h("div", { className: "wf-card__body" });
  const _e1616 = WF.h("h3", { className: "wf-heading" }, "PDF Reports");
  _e1615.appendChild(_e1616);
  const _e1617 = WF.h("p", { className: "wf-text wf-text--muted" }, "Create invoices, receipts, and reports as PDF files from structured data.");
  _e1615.appendChild(_e1617);
  _e1614.appendChild(_e1615);
  _e1609.appendChild(_e1614);
  const _e1618 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1619 = WF.h("div", { className: "wf-card__body" });
  const _e1620 = WF.h("h3", { className: "wf-heading" }, "Email Templates");
  _e1619.appendChild(_e1620);
  const _e1621 = WF.h("p", { className: "wf-text wf-text--muted" }, "Render HTML emails with WebFluent components and your design system.");
  _e1619.appendChild(_e1621);
  _e1618.appendChild(_e1619);
  _e1609.appendChild(_e1618);
  _e1444.appendChild(_e1609);
  const _e1622 = WF.h("div", { className: "wf-spacer" });
  _e1444.appendChild(_e1622);
  _root.appendChild(_e1444);
  return _root;
}

function Page_Accessibility(params) {
  const _root = document.createDocumentFragment();
  const _e1623 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1624 = WF.h("div", { className: "wf-spacer" });
  _e1623.appendChild(_e1624);
  const _e1625 = WF.h("h1", { className: "wf-heading" }, "Accessibility Linting");
  _e1623.appendChild(_e1625);
  const _e1626 = WF.h("p", { className: "wf-text wf-text--muted" }, "WebFluent checks your code for accessibility issues at compile time. Warnings are printed during build but never block compilation.");
  _e1623.appendChild(_e1626);
  const _e1627 = WF.h("div", { className: "wf-spacer" });
  _e1623.appendChild(_e1627);
  const _e1628 = WF.h("h2", { className: "wf-heading" }, "How It Works");
  _e1623.appendChild(_e1628);
  const _e1629 = WF.h("p", { className: "wf-text" }, "The linter runs automatically after parsing, before code generation. It walks the AST and checks each component against 12 rules.");
  _e1623.appendChild(_e1629);
  const _e1630 = WF.h("div", { className: "wf-spacer" });
  _e1623.appendChild(_e1630);
  const _e1631 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1632 = WF.h("div", { className: "wf-card__body" });
  const _e1633 = WF.h("code", { className: "wf-code wf-code--block" }, "$ wf build\nBuilding my-app...\n  Warning [A01]: Image missing \"alt\" attribute at src/pages/Home.wf:12:5\n    Add alt text: Image(src: \"...\", alt: \"Description of image\")\n  Warning [A03]: Input missing \"label\" attribute at src/pages/Form.wf:8:9\n    Add a label: Input(text, label: \"Username\")\n  3 pages, 2 components, 1 stores\n  Build complete with 2 accessibility warning(s).");
  _e1632.appendChild(_e1633);
  _e1631.appendChild(_e1632);
  _e1623.appendChild(_e1631);
  const _e1634 = WF.h("div", { className: "wf-spacer" });
  _e1623.appendChild(_e1634);
  const _e1635 = WF.h("hr", { className: "wf-divider" });
  _e1623.appendChild(_e1635);
  const _e1636 = WF.h("div", { className: "wf-spacer" });
  _e1623.appendChild(_e1636);
  const _e1637 = WF.h("h2", { className: "wf-heading" }, "Lint Rules");
  _e1623.appendChild(_e1637);
  const _e1638 = WF.h("table", { className: "wf-table" });
  const _e1639 = WF.h("thead", {});
  const _e1640 = WF.h("td", {}, "Rule");
  _e1639.appendChild(_e1640);
  const _e1641 = WF.h("td", {}, "Component");
  _e1639.appendChild(_e1641);
  const _e1642 = WF.h("td", {}, "Check");
  _e1639.appendChild(_e1642);
  _e1638.appendChild(_e1639);
  const _e1643 = WF.h("tr", {});
  const _e1644 = WF.h("td", {}, "A01");
  _e1643.appendChild(_e1644);
  const _e1645 = WF.h("td", {}, "Image");
  _e1643.appendChild(_e1645);
  const _e1646 = WF.h("td", {}, "Must have alt attribute");
  _e1643.appendChild(_e1646);
  _e1638.appendChild(_e1643);
  const _e1647 = WF.h("tr", {});
  const _e1648 = WF.h("td", {}, "A02");
  _e1647.appendChild(_e1648);
  const _e1649 = WF.h("td", {}, "IconButton");
  _e1647.appendChild(_e1649);
  const _e1650 = WF.h("td", {}, "Must have label attribute (no visible text)");
  _e1647.appendChild(_e1650);
  _e1638.appendChild(_e1647);
  const _e1651 = WF.h("tr", {});
  const _e1652 = WF.h("td", {}, "A03");
  _e1651.appendChild(_e1652);
  const _e1653 = WF.h("td", {}, "Input");
  _e1651.appendChild(_e1653);
  const _e1654 = WF.h("td", {}, "Must have label or placeholder");
  _e1651.appendChild(_e1654);
  _e1638.appendChild(_e1651);
  const _e1655 = WF.h("tr", {});
  const _e1656 = WF.h("td", {}, "A04");
  _e1655.appendChild(_e1656);
  const _e1657 = WF.h("td", {}, "Checkbox, Radio, Switch, Slider");
  _e1655.appendChild(_e1657);
  const _e1658 = WF.h("td", {}, "Must have label attribute");
  _e1655.appendChild(_e1658);
  _e1638.appendChild(_e1655);
  const _e1659 = WF.h("tr", {});
  const _e1660 = WF.h("td", {}, "A05");
  _e1659.appendChild(_e1660);
  const _e1661 = WF.h("td", {}, "Button");
  _e1659.appendChild(_e1661);
  const _e1662 = WF.h("td", {}, "Must have text content");
  _e1659.appendChild(_e1662);
  _e1638.appendChild(_e1659);
  const _e1663 = WF.h("tr", {});
  const _e1664 = WF.h("td", {}, "A06");
  _e1663.appendChild(_e1664);
  const _e1665 = WF.h("td", {}, "Link");
  _e1663.appendChild(_e1665);
  const _e1666 = WF.h("td", {}, "Must have text content or children");
  _e1663.appendChild(_e1666);
  _e1638.appendChild(_e1663);
  const _e1667 = WF.h("tr", {});
  const _e1668 = WF.h("td", {}, "A07");
  _e1667.appendChild(_e1668);
  const _e1669 = WF.h("td", {}, "Heading");
  _e1667.appendChild(_e1669);
  const _e1670 = WF.h("td", {}, "Must not be empty");
  _e1667.appendChild(_e1670);
  _e1638.appendChild(_e1667);
  const _e1671 = WF.h("tr", {});
  const _e1672 = WF.h("td", {}, "A08");
  _e1671.appendChild(_e1672);
  const _e1673 = WF.h("td", {}, "Modal, Dialog");
  _e1671.appendChild(_e1673);
  const _e1674 = WF.h("td", {}, "Must have title attribute");
  _e1671.appendChild(_e1674);
  _e1638.appendChild(_e1671);
  const _e1675 = WF.h("tr", {});
  const _e1676 = WF.h("td", {}, "A09");
  _e1675.appendChild(_e1676);
  const _e1677 = WF.h("td", {}, "Video");
  _e1675.appendChild(_e1677);
  const _e1678 = WF.h("td", {}, "Must have controls attribute");
  _e1675.appendChild(_e1678);
  _e1638.appendChild(_e1675);
  const _e1679 = WF.h("tr", {});
  const _e1680 = WF.h("td", {}, "A10");
  _e1679.appendChild(_e1680);
  const _e1681 = WF.h("td", {}, "Table");
  _e1679.appendChild(_e1681);
  const _e1682 = WF.h("td", {}, "Must have Thead header row");
  _e1679.appendChild(_e1682);
  _e1638.appendChild(_e1679);
  const _e1683 = WF.h("tr", {});
  const _e1684 = WF.h("td", {}, "A11");
  _e1683.appendChild(_e1684);
  const _e1685 = WF.h("td", {}, "Heading");
  _e1683.appendChild(_e1685);
  const _e1686 = WF.h("td", {}, "Levels must not skip (h1 to h3)");
  _e1683.appendChild(_e1686);
  _e1638.appendChild(_e1683);
  const _e1687 = WF.h("tr", {});
  const _e1688 = WF.h("td", {}, "A12");
  _e1687.appendChild(_e1688);
  const _e1689 = WF.h("td", {}, "Page");
  _e1687.appendChild(_e1689);
  const _e1690 = WF.h("td", {}, "Must have exactly one h1 (not for Presentation or Document)");
  _e1687.appendChild(_e1690);
  _e1638.appendChild(_e1687);
  const _e1691 = WF.h("tr", {});
  const _e1692 = WF.h("td", {}, "A13");
  _e1691.appendChild(_e1692);
  const _e1693 = WF.h("td", {}, "Theme");
  _e1691.appendChild(_e1693);
  const _e1694 = WF.h("td", {}, "Colour pairings must clear the WCAG AA contrast ratio");
  _e1691.appendChild(_e1694);
  _e1638.appendChild(_e1691);
  const _e1695 = WF.h("tr", {});
  const _e1696 = WF.h("td", {}, "S01");
  _e1695.appendChild(_e1696);
  const _e1697 = WF.h("td", {}, "Page");
  _e1695.appendChild(_e1697);
  const _e1698 = WF.h("td", {}, "Must have a title");
  _e1695.appendChild(_e1698);
  _e1638.appendChild(_e1695);
  const _e1699 = WF.h("tr", {});
  const _e1700 = WF.h("td", {}, "S02");
  _e1699.appendChild(_e1700);
  const _e1701 = WF.h("td", {}, "Page");
  _e1699.appendChild(_e1701);
  const _e1702 = WF.h("td", {}, "Should have a description, or its search snippet gets written for it");
  _e1699.appendChild(_e1702);
  _e1638.appendChild(_e1699);
  const _e1703 = WF.h("tr", {});
  const _e1704 = WF.h("td", {}, "S03");
  _e1703.appendChild(_e1704);
  const _e1705 = WF.h("td", {}, "Page");
  _e1703.appendChild(_e1705);
  const _e1706 = WF.h("td", {}, "A description over ~160 characters is truncated in results");
  _e1703.appendChild(_e1706);
  _e1638.appendChild(_e1703);
  const _e1707 = WF.h("tr", {});
  const _e1708 = WF.h("td", {}, "S04");
  _e1707.appendChild(_e1708);
  const _e1709 = WF.h("td", {}, "Page");
  _e1707.appendChild(_e1709);
  const _e1710 = WF.h("td", {}, "Two pages must not claim one route");
  _e1707.appendChild(_e1710);
  _e1638.appendChild(_e1707);
  const _e1711 = WF.h("tr", {});
  const _e1712 = WF.h("td", {}, "V01");
  _e1711.appendChild(_e1712);
  const _e1713 = WF.h("td", {}, "Any");
  _e1711.appendChild(_e1713);
  const _e1714 = WF.h("td", {}, "A bare word that resolves to nothing");
  _e1711.appendChild(_e1714);
  _e1638.appendChild(_e1711);
  const _e1715 = WF.h("tr", {});
  const _e1716 = WF.h("td", {}, "V02");
  _e1715.appendChild(_e1716);
  const _e1717 = WF.h("td", {}, "Any");
  _e1715.appendChild(_e1717);
  const _e1718 = WF.h("td", {}, "A real modifier whose class no stylesheet defines");
  _e1715.appendChild(_e1718);
  _e1638.appendChild(_e1715);
  _e1623.appendChild(_e1638);
  const _e1719 = WF.h("div", { className: "wf-spacer" });
  _e1623.appendChild(_e1719);
  const _e1720 = WF.h("hr", { className: "wf-divider" });
  _e1623.appendChild(_e1720);
  const _e1721 = WF.h("div", { className: "wf-spacer" });
  _e1623.appendChild(_e1721);
  const _e1722 = WF.h("h2", { className: "wf-heading" }, "Examples");
  _e1623.appendChild(_e1722);
  const _e1723 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1724 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1725 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1726 = WF.h("div", { className: "wf-card__body" });
  const _e1727 = WF.h("p", { className: "wf-text wf-text--danger wf-text--bold" }, "Bad (triggers warning)");
  _e1726.appendChild(_e1727);
  const _e1728 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\")\nIconButton(icon: \"close\")\nInput(text)\nCheckbox(bind: agreed)\nButton()");
  _e1726.appendChild(_e1728);
  _e1725.appendChild(_e1726);
  _e1724.appendChild(_e1725);
  _e1723.appendChild(_e1724);
  const _e1729 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1730 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1731 = WF.h("div", { className: "wf-card__body" });
  const _e1732 = WF.h("p", { className: "wf-text wf-text--success wf-text--bold" }, "Good (no warnings)");
  _e1731.appendChild(_e1732);
  const _e1733 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\", alt: \"Team photo\")\nIconButton(icon: \"close\", label: \"Close\")\nInput(text, label: \"Username\")\nCheckbox(bind: agreed, label: \"I agree\")\nButton(\"Save\")");
  _e1731.appendChild(_e1733);
  _e1730.appendChild(_e1731);
  _e1729.appendChild(_e1730);
  _e1723.appendChild(_e1729);
  _e1623.appendChild(_e1723);
  const _e1734 = WF.h("div", { className: "wf-spacer" });
  _e1623.appendChild(_e1734);
  _root.appendChild(_e1623);
  return _root;
}

function Page_Home(params) {
  const _counter = WF.signal(0);
  const _taskInput = WF.signal("");
  const _showDemo = WF.signal(false);
  const _root = document.createDocumentFragment();
  const _e1735 = WF.h("div", { className: "wf-container" });
  const _e1736 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1736);
  const _e1737 = WF.h("h1", { className: "wf-heading wf-text--center wf-animate-slideUp" }, () => WF.i18n.t("hero.title"));
  _e1735.appendChild(_e1737);
  const _e1738 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1738);
  const _e1739 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, () => WF.i18n.t("hero.sub1"));
  _e1735.appendChild(_e1739);
  const _e1740 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, () => WF.i18n.t("hero.sub2"));
  _e1735.appendChild(_e1740);
  const _e1741 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1741);
  const _e1742 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1743 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, () => WF.i18n.t("hero.cta"));
  _e1742.appendChild(_e1743);
  const _e1744 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { WF.navigate("/guide"); } }, () => WF.i18n.t("hero.guide"));
  _e1742.appendChild(_e1744);
  _e1735.appendChild(_e1742);
  const _e1745 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1745);
  const _e1746 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1747 = WF.h("div", { className: "wf-card__body" });
  const _e1748 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\") {\n    Container {\n        Heading(\"Hello, WebFluent!\", h1)\n        Text(\"Build for the web. Nothing else.\")\n\n        Button(\"Get Started\", primary, large) {\n            navigate(\"/docs\")\n        }\n    }\n}");
  _e1747.appendChild(_e1748);
  _e1746.appendChild(_e1747);
  _e1735.appendChild(_e1746);
  const _e1749 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1749);
  const _e1750 = WF.h("hr", { className: "wf-divider" });
  _e1735.appendChild(_e1750);
  const _e1751 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1751);
  const _e1752 = WF.h("h2", { className: "wf-heading wf-text--center" }, () => WF.i18n.t("demo.title"));
  _e1735.appendChild(_e1752);
  const _e1753 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, () => WF.i18n.t("demo.subtitle"));
  _e1735.appendChild(_e1753);
  const _e1754 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1754);
  const _e1755 = WF.h("div", { className: "wf-grid wf-grid--gap-lg", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e1756 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1757 = WF.h("div", { className: "wf-card__header" });
  const _e1758 = WF.h("h2", { className: "wf-heading" }, () => WF.i18n.t("demo.counter"));
  _e1757.appendChild(_e1758);
  _e1756.appendChild(_e1757);
  const _e1759 = WF.h("div", { className: "wf-card__body" });
  const _e1760 = WF.h("div", { className: "wf-row wf-row--center wf-row--gap-md" });
  const _e1761 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { _counter.set((_counter() - 1)); } }, "-");
  _e1760.appendChild(_e1761);
  const _e1762 = WF.h("h2", { className: "wf-heading wf-heading--primary" }, () => `${_counter()}`);
  _e1760.appendChild(_e1762);
  const _e1763 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { _counter.set((_counter() + 1)); } }, "+");
  _e1760.appendChild(_e1763);
  _e1759.appendChild(_e1760);
  const _e1764 = WF.h("div", { className: "wf-spacer" });
  _e1759.appendChild(_e1764);
  const _e1765 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.counter.hint"));
  _e1759.appendChild(_e1765);
  _e1756.appendChild(_e1759);
  _e1755.appendChild(_e1756);
  const _e1766 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1767 = WF.h("div", { className: "wf-card__header" });
  const _e1768 = WF.h("h2", { className: "wf-heading" }, () => WF.i18n.t("demo.binding"));
  _e1767.appendChild(_e1768);
  _e1766.appendChild(_e1767);
  const _e1769 = WF.h("div", { className: "wf-card__body" });
  const _e1770 = WF.h("input", { className: "wf-input", value: () => _taskInput(), "on:input": (e) => _taskInput.set(e.target.value), placeholder: WF.i18n.t("demo.binding.placeholder"), label: "Input", type: "text" });
  _e1769.appendChild(_e1770);
  const _e1771 = WF.h("div", { className: "wf-spacer" });
  _e1769.appendChild(_e1771);
  WF.condRender(_e1769,
    () => (_taskInput() !== ""),
    () => {
      const _e1772 = document.createDocumentFragment();
      const _e1773 = WF.h("div", { className: "wf-alert wf-alert--info", role: "status" }, () => `You typed: ${_taskInput()}`);
      _e1772.appendChild(_e1773);
      return _e1772;
    },
    null,
    null
  );
  const _e1774 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.binding.hint"));
  _e1769.appendChild(_e1774);
  _e1766.appendChild(_e1769);
  _e1755.appendChild(_e1766);
  const _e1775 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1776 = WF.h("div", { className: "wf-card__header" });
  const _e1777 = WF.h("h2", { className: "wf-heading" }, () => WF.i18n.t("demo.conditional"));
  _e1776.appendChild(_e1777);
  _e1775.appendChild(_e1776);
  const _e1778 = WF.h("div", { className: "wf-card__body" });
  const _e1779 = WF.h("label", { className: "wf-switch" });
  const _e1780 = WF.h("input", { type: "checkbox", role: "switch",                  checked: () => _showDemo(), "aria-checked": () => _showDemo() ? "true" : "false",                  "on:change": () => _showDemo.set(!_showDemo()) });
  _e1779.appendChild(_e1780);
  const _e1781 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1779.appendChild(_e1781);
  _e1779.appendChild(WF.text(WF.i18n.t("demo.conditional.toggle")));
  _e1778.appendChild(_e1779);
  const _e1782 = WF.h("div", { className: "wf-spacer" });
  _e1778.appendChild(_e1782);
  WF.condRender(_e1778,
    () => _showDemo(),
    () => {
      const _e1783 = document.createDocumentFragment();
      const _e1784 = WF.h("div", { className: "wf-card wf-card--outlined" });
      const _e1785 = WF.h("div", { className: "wf-card__body" });
      const _e1786 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Visible!");
      _e1785.appendChild(_e1786);
      const _e1787 = WF.h("div", { className: "wf-spacer" });
      _e1785.appendChild(_e1787);
      const _e1788 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("demo.conditional.text"));
      _e1785.appendChild(_e1788);
      _e1784.appendChild(_e1785);
      _e1783.appendChild(_e1784);
      return _e1783;
    },
    null,
    { enter: "slideUp", exit: "fadeOut" }
  );
  _e1775.appendChild(_e1778);
  _e1755.appendChild(_e1775);
  const _e1789 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1790 = WF.h("div", { className: "wf-card__header" });
  const _e1791 = WF.h("h2", { className: "wf-heading" }, () => WF.i18n.t("demo.components"));
  _e1790.appendChild(_e1791);
  _e1789.appendChild(_e1790);
  const _e1792 = WF.h("div", { className: "wf-card__body" });
  const _e1793 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1794 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1795 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e1794.appendChild(_e1795);
  const _e1796 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e1794.appendChild(_e1796);
  const _e1797 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e1794.appendChild(_e1797);
  _e1793.appendChild(_e1794);
  const _e1798 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1799 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "New");
  _e1798.appendChild(_e1799);
  const _e1800 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Sale");
  _e1798.appendChild(_e1800);
  const _e1801 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Active");
  _e1798.appendChild(_e1801);
  const _e1802 = WF.h("span", { className: "wf-tag" }, "WebFluent");
  _e1798.appendChild(_e1802);
  _e1793.appendChild(_e1798);
  const _e1803 = WF.h("progress", { className: "wf-progress", value: 72, max: 100 });
  _e1793.appendChild(_e1803);
  const _e1804 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.components.hint"));
  _e1793.appendChild(_e1804);
  _e1792.appendChild(_e1793);
  _e1789.appendChild(_e1792);
  _e1755.appendChild(_e1789);
  _e1735.appendChild(_e1755);
  const _e1805 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1805);
  const _e1806 = WF.h("hr", { className: "wf-divider" });
  _e1735.appendChild(_e1806);
  const _e1807 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1807);
  const _e1808 = WF.h("h2", { className: "wf-heading wf-text--center" }, () => WF.i18n.t("why.title"));
  _e1735.appendChild(_e1808);
  const _e1809 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, () => WF.i18n.t("why.subtitle"));
  _e1735.appendChild(_e1809);
  const _e1810 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1810);
  const _e1811 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1812 = Component_FeatureCard({ title: WF.i18n.t("why.syntax"), description: WF.i18n.t("why.syntax.desc") });
  _e1811.appendChild(_e1812);
  const _e1813 = Component_FeatureCard({ title: WF.i18n.t("why.components"), description: WF.i18n.t("why.components.desc") });
  _e1811.appendChild(_e1813);
  const _e1814 = Component_FeatureCard({ title: WF.i18n.t("why.reactivity"), description: WF.i18n.t("why.reactivity.desc") });
  _e1811.appendChild(_e1814);
  const _e1815 = Component_FeatureCard({ title: WF.i18n.t("why.design"), description: WF.i18n.t("why.design.desc") });
  _e1811.appendChild(_e1815);
  const _e1816 = Component_FeatureCard({ title: WF.i18n.t("why.animation"), description: WF.i18n.t("why.animation.desc") });
  _e1811.appendChild(_e1816);
  const _e1817 = Component_FeatureCard({ title: WF.i18n.t("why.i18n"), description: WF.i18n.t("why.i18n.desc") });
  _e1811.appendChild(_e1817);
  const _e1818 = Component_FeatureCard({ title: WF.i18n.t("why.ssg"), description: WF.i18n.t("why.ssg.desc") });
  _e1811.appendChild(_e1818);
  const _e1819 = Component_FeatureCard({ title: WF.i18n.t("why.a11y"), description: WF.i18n.t("why.a11y.desc") });
  _e1811.appendChild(_e1819);
  const _e1820 = Component_FeatureCard({ title: WF.i18n.t("why.zero"), description: WF.i18n.t("why.zero.desc") });
  _e1811.appendChild(_e1820);
  _e1735.appendChild(_e1811);
  const _e1821 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1821);
  const _e1822 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1823 = WF.h("div", { className: "wf-card__body" });
  const _e1824 = WF.h("div", { className: "wf-row wf-row--center wf-row--between" });
  const _e1825 = WF.h("div", { className: "wf-stack" });
  const _e1826 = WF.h("h2", { className: "wf-heading" }, () => WF.i18n.t("cta.title"));
  _e1825.appendChild(_e1826);
  const _e1827 = WF.h("p", { className: "wf-text wf-text--muted" }, () => WF.i18n.t("cta.subtitle"));
  _e1825.appendChild(_e1827);
  _e1824.appendChild(_e1825);
  const _e1828 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, () => WF.i18n.t("hero.cta"));
  _e1824.appendChild(_e1828);
  _e1823.appendChild(_e1824);
  _e1822.appendChild(_e1823);
  _e1735.appendChild(_e1822);
  const _e1829 = WF.h("div", { className: "wf-spacer" });
  _e1735.appendChild(_e1829);
  _root.appendChild(_e1735);
  return _root;
}

function Page_GettingStarted(params) {
  const _root = document.createDocumentFragment();
  const _e1830 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1831 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1831);
  const _e1832 = WF.h("h1", { className: "wf-heading" }, "Getting Started");
  _e1830.appendChild(_e1832);
  const _e1833 = WF.h("p", { className: "wf-text wf-text--muted" }, "Get up and running with WebFluent in under a minute.");
  _e1830.appendChild(_e1833);
  const _e1834 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1834);
  const _e1835 = WF.h("h2", { className: "wf-heading" }, "Install");
  _e1830.appendChild(_e1835);
  const _e1836 = WF.h("p", { className: "wf-text" }, "Build from source (requires Rust):");
  _e1830.appendChild(_e1836);
  const _e1837 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1837);
  const _e1838 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1839 = WF.h("div", { className: "wf-card__body" });
  const _e1840 = WF.h("code", { className: "wf-code wf-code--block" }, "git clone https://github.com/user/webfluent.git\ncd webfluent\ncargo build --release");
  _e1839.appendChild(_e1840);
  _e1838.appendChild(_e1839);
  _e1830.appendChild(_e1838);
  const _e1841 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1841);
  const _e1842 = WF.h("p", { className: "wf-text wf-text--muted" }, "The binary is at target/release/wf. Add it to your PATH.");
  _e1830.appendChild(_e1842);
  const _e1843 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1843);
  const _e1844 = WF.h("hr", { className: "wf-divider" });
  _e1830.appendChild(_e1844);
  const _e1845 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1845);
  const _e1846 = WF.h("h2", { className: "wf-heading" }, "Create a Project");
  _e1830.appendChild(_e1846);
  const _e1847 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1847);
  const _e1848 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1849 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1850 = WF.h("div", { className: "wf-card__body" });
  const _e1851 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "SPA");
  _e1850.appendChild(_e1851);
  const _e1852 = WF.h("div", { className: "wf-spacer" });
  _e1850.appendChild(_e1852);
  const _e1853 = WF.h("h2", { className: "wf-heading" }, "Interactive App");
  _e1850.appendChild(_e1853);
  const _e1854 = WF.h("p", { className: "wf-text wf-text--muted" }, "Dashboard with routing, stores, forms, modals, animations.");
  _e1850.appendChild(_e1854);
  const _e1855 = WF.h("div", { className: "wf-spacer" });
  _e1850.appendChild(_e1855);
  const _e1856 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-app -t spa");
  _e1850.appendChild(_e1856);
  _e1849.appendChild(_e1850);
  _e1848.appendChild(_e1849);
  const _e1857 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1858 = WF.h("div", { className: "wf-card__body" });
  const _e1859 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Static");
  _e1858.appendChild(_e1859);
  const _e1860 = WF.h("div", { className: "wf-spacer" });
  _e1858.appendChild(_e1860);
  const _e1861 = WF.h("h2", { className: "wf-heading" }, "Static Site");
  _e1858.appendChild(_e1861);
  const _e1862 = WF.h("p", { className: "wf-text wf-text--muted" }, "Marketing site with SSG, i18n, blog, contact form.");
  _e1858.appendChild(_e1862);
  const _e1863 = WF.h("div", { className: "wf-spacer" });
  _e1858.appendChild(_e1863);
  const _e1864 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-site -t static");
  _e1858.appendChild(_e1864);
  _e1857.appendChild(_e1858);
  _e1848.appendChild(_e1857);
  const _e1865 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1866 = WF.h("div", { className: "wf-card__body" });
  const _e1867 = WF.h("span", { className: "wf-badge wf-badge--info" }, "PDF");
  _e1866.appendChild(_e1867);
  const _e1868 = WF.h("div", { className: "wf-spacer" });
  _e1866.appendChild(_e1868);
  const _e1869 = WF.h("h2", { className: "wf-heading" }, "PDF Document");
  _e1866.appendChild(_e1869);
  const _e1870 = WF.h("p", { className: "wf-text wf-text--muted" }, "Reports, invoices, docs. Tables, code blocks, auto page breaks.");
  _e1866.appendChild(_e1870);
  const _e1871 = WF.h("div", { className: "wf-spacer" });
  _e1866.appendChild(_e1871);
  const _e1872 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-report -t pdf");
  _e1866.appendChild(_e1872);
  _e1865.appendChild(_e1866);
  _e1848.appendChild(_e1865);
  _e1830.appendChild(_e1848);
  const _e1873 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1873);
  const _e1874 = WF.h("hr", { className: "wf-divider" });
  _e1830.appendChild(_e1874);
  const _e1875 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1875);
  const _e1876 = WF.h("h2", { className: "wf-heading" }, "Build and Serve");
  _e1830.appendChild(_e1876);
  const _e1877 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1878 = WF.h("div", { className: "wf-card__body" });
  const _e1879 = WF.h("code", { className: "wf-code wf-code--block" }, "cd my-app\nwf build\nwf serve");
  _e1878.appendChild(_e1879);
  _e1877.appendChild(_e1878);
  _e1830.appendChild(_e1877);
  const _e1880 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1880);
  const _e1881 = WF.h("p", { className: "wf-text wf-text--muted" }, "Open http://localhost:3000 in your browser.");
  _e1830.appendChild(_e1881);
  const _e1882 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1882);
  const _e1883 = WF.h("hr", { className: "wf-divider" });
  _e1830.appendChild(_e1883);
  const _e1884 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1884);
  const _e1885 = WF.h("h2", { className: "wf-heading" }, "Project Structure");
  _e1830.appendChild(_e1885);
  const _e1886 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1887 = WF.h("div", { className: "wf-card__body" });
  const _e1888 = WF.h("code", { className: "wf-code wf-code--block" }, "my-app/\n+-- webfluent.app.json       # Config\n+-- src/\n|   +-- App.wf               # Root (router, layout)\n|   +-- pages/\n|   +-- components/\n|   +-- stores/\n|   +-- translations/\n+-- public/\n+-- build/");
  _e1887.appendChild(_e1888);
  _e1886.appendChild(_e1887);
  _e1830.appendChild(_e1886);
  const _e1889 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1889);
  const _e1890 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1891 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/guide"); } }, "Read the Guide");
  _e1890.appendChild(_e1891);
  const _e1892 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/components"); } }, "Browse Components");
  _e1890.appendChild(_e1892);
  _e1830.appendChild(_e1890);
  const _e1893 = WF.h("div", { className: "wf-spacer" });
  _e1830.appendChild(_e1893);
  _root.appendChild(_e1830);
  return _root;
}

function Page_Animation(params) {
  const _showCard = WF.signal(false);
  const _root = document.createDocumentFragment();
  const _e1894 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1895 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1895);
  const _e1896 = WF.h("h1", { className: "wf-heading" }, "Animation System");
  _e1894.appendChild(_e1896);
  const _e1897 = WF.h("p", { className: "wf-text wf-text--muted" }, "Declarative animations built into the language. No CSS keyframes to write.");
  _e1894.appendChild(_e1897);
  const _e1898 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1898);
  const _e1899 = WF.h("h2", { className: "wf-heading" }, "Mount Animations");
  _e1894.appendChild(_e1899);
  const _e1900 = WF.h("p", { className: "wf-text" }, "Add an animation modifier to any component. It plays when the element appears. Hover each card to replay.");
  _e1894.appendChild(_e1900);
  const _e1901 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1901);
  const _e1902 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1903 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn"); } });
  const _e1904 = WF.h("div", { className: "wf-card__body" });
  const _e1905 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "fadeIn");
  _e1904.appendChild(_e1905);
  const _e1906 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Fades from transparent");
  _e1904.appendChild(_e1906);
  _e1903.appendChild(_e1904);
  _e1902.appendChild(_e1903);
  const _e1907 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideUp", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideUp"); } });
  const _e1908 = WF.h("div", { className: "wf-card__body" });
  const _e1909 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideUp");
  _e1908.appendChild(_e1909);
  const _e1910 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from below");
  _e1908.appendChild(_e1910);
  _e1907.appendChild(_e1908);
  _e1902.appendChild(_e1907);
  const _e1911 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-scaleIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "scaleIn"); } });
  const _e1912 = WF.h("div", { className: "wf-card__body" });
  const _e1913 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "scaleIn");
  _e1912.appendChild(_e1913);
  const _e1914 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Scales from 90%");
  _e1912.appendChild(_e1914);
  _e1911.appendChild(_e1912);
  _e1902.appendChild(_e1911);
  const _e1915 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideDown", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideDown"); } });
  const _e1916 = WF.h("div", { className: "wf-card__body" });
  const _e1917 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideDown");
  _e1916.appendChild(_e1917);
  const _e1918 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from above");
  _e1916.appendChild(_e1918);
  _e1915.appendChild(_e1916);
  _e1902.appendChild(_e1915);
  const _e1919 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideLeft", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideLeft"); } });
  const _e1920 = WF.h("div", { className: "wf-card__body" });
  const _e1921 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideLeft");
  _e1920.appendChild(_e1921);
  const _e1922 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from right");
  _e1920.appendChild(_e1922);
  _e1919.appendChild(_e1920);
  _e1902.appendChild(_e1919);
  const _e1923 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-bounce", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "bounce"); } });
  const _e1924 = WF.h("div", { className: "wf-card__body" });
  const _e1925 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "bounce");
  _e1924.appendChild(_e1925);
  const _e1926 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Bouncy entrance");
  _e1924.appendChild(_e1926);
  _e1923.appendChild(_e1924);
  _e1902.appendChild(_e1923);
  const _e1927 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-shake", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "shake"); } });
  const _e1928 = WF.h("div", { className: "wf-card__body" });
  const _e1929 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "shake");
  _e1928.appendChild(_e1929);
  const _e1930 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Horizontal shake");
  _e1928.appendChild(_e1930);
  _e1927.appendChild(_e1928);
  _e1902.appendChild(_e1927);
  const _e1931 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-pulse", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "pulse"); } });
  const _e1932 = WF.h("div", { className: "wf-card__body" });
  const _e1933 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "pulse");
  _e1932.appendChild(_e1933);
  const _e1934 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Gentle scale pulse");
  _e1932.appendChild(_e1934);
  _e1931.appendChild(_e1932);
  _e1902.appendChild(_e1931);
  const _e1935 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideRight", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideRight"); } });
  const _e1936 = WF.h("div", { className: "wf-card__body" });
  const _e1937 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideRight");
  _e1936.appendChild(_e1937);
  const _e1938 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from left");
  _e1936.appendChild(_e1938);
  _e1935.appendChild(_e1936);
  _e1902.appendChild(_e1935);
  _e1894.appendChild(_e1902);
  const _e1939 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1939);
  const _e1940 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1941 = WF.h("div", { className: "wf-card__body" });
  const _e1942 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn) { ... }\nHeading(\"Title\", h1, slideUp)\nButton(\"Click\", primary, bounce)");
  _e1941.appendChild(_e1942);
  _e1940.appendChild(_e1941);
  _e1894.appendChild(_e1940);
  const _e1943 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1943);
  const _e1944 = WF.h("hr", { className: "wf-divider" });
  _e1894.appendChild(_e1944);
  const _e1945 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1945);
  const _e1946 = WF.h("h2", { className: "wf-heading" }, "Live: Conditional Animation");
  _e1894.appendChild(_e1946);
  const _e1947 = WF.h("p", { className: "wf-text" }, "Toggle the switch to see enter/exit animations on the card below.");
  _e1894.appendChild(_e1947);
  const _e1948 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1948);
  const _e1949 = WF.h("label", { className: "wf-switch" });
  const _e1950 = WF.h("input", { type: "checkbox", role: "switch",                  checked: () => _showCard(), "aria-checked": () => _showCard() ? "true" : "false",                  "on:change": () => _showCard.set(!_showCard()) });
  _e1949.appendChild(_e1950);
  const _e1951 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1949.appendChild(_e1951);
  _e1949.appendChild(WF.text("Show animated card"));
  _e1894.appendChild(_e1949);
  const _e1952 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1952);
  WF.condRender(_e1894,
    () => _showCard(),
    () => {
      const _e1953 = document.createDocumentFragment();
      const _e1954 = WF.h("div", { className: "wf-card wf-card--elevated" });
      const _e1955 = WF.h("div", { className: "wf-card__body" });
      const _e1956 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Animated!");
      _e1955.appendChild(_e1956);
      const _e1957 = WF.h("div", { className: "wf-spacer" });
      _e1955.appendChild(_e1957);
      const _e1958 = WF.h("p", { className: "wf-text" }, "This card scales in and fades out.");
      _e1955.appendChild(_e1958);
      const _e1959 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Controlled by: if showCard, animate(scaleIn, fadeOut)");
      _e1955.appendChild(_e1959);
      _e1954.appendChild(_e1955);
      _e1953.appendChild(_e1954);
      return _e1953;
    },
    null,
    { enter: "scaleIn", exit: "fadeOut" }
  );
  const _e1960 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1960);
  const _e1961 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1962 = WF.h("div", { className: "wf-card__body" });
  const _e1963 = WF.h("code", { className: "wf-code wf-code--block" }, "if showCard, animate(scaleIn, fadeOut) {\n    Card(elevated) {\n        Text(\"Animated content\")\n    }\n}");
  _e1962.appendChild(_e1963);
  _e1961.appendChild(_e1962);
  _e1894.appendChild(_e1961);
  const _e1964 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1964);
  const _e1965 = WF.h("hr", { className: "wf-divider" });
  _e1894.appendChild(_e1965);
  const _e1966 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1966);
  const _e1967 = WF.h("h2", { className: "wf-heading" }, "Speed Variants");
  _e1894.appendChild(_e1967);
  const _e1968 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1968);
  const _e1969 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1970 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn", "150ms"); } });
  const _e1971 = WF.h("div", { className: "wf-card__body" });
  const _e1972 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "fast");
  _e1971.appendChild(_e1972);
  const _e1973 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "150ms");
  _e1971.appendChild(_e1973);
  const _e1974 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn, fast)");
  _e1971.appendChild(_e1974);
  _e1970.appendChild(_e1971);
  _e1969.appendChild(_e1970);
  const _e1975 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn"); } });
  const _e1976 = WF.h("div", { className: "wf-card__body" });
  const _e1977 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "default");
  _e1976.appendChild(_e1977);
  const _e1978 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "300ms");
  _e1976.appendChild(_e1978);
  const _e1979 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn)");
  _e1976.appendChild(_e1979);
  _e1975.appendChild(_e1976);
  _e1969.appendChild(_e1975);
  const _e1980 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn", "500ms"); } });
  const _e1981 = WF.h("div", { className: "wf-card__body" });
  const _e1982 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slow");
  _e1981.appendChild(_e1982);
  const _e1983 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "500ms");
  _e1981.appendChild(_e1983);
  const _e1984 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn, slow)");
  _e1981.appendChild(_e1984);
  _e1980.appendChild(_e1981);
  _e1969.appendChild(_e1980);
  _e1894.appendChild(_e1969);
  const _e1985 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1985);
  const _e1986 = WF.h("hr", { className: "wf-divider" });
  _e1894.appendChild(_e1986);
  const _e1987 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1987);
  const _e1988 = WF.h("h2", { className: "wf-heading" }, "All 12 Animations");
  _e1894.appendChild(_e1988);
  const _e1989 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e1989);
  const _e1990 = WF.h("table", { className: "wf-table" });
  const _e1991 = WF.h("thead", {});
  const _e1992 = WF.h("td", {}, "Name");
  _e1991.appendChild(_e1992);
  const _e1993 = WF.h("td", {}, "Effect");
  _e1991.appendChild(_e1993);
  const _e1994 = WF.h("td", {}, "Usage");
  _e1991.appendChild(_e1994);
  _e1990.appendChild(_e1991);
  const _e1995 = WF.h("tr", {});
  const _e1996 = WF.h("td", {}, "fadeIn / fadeOut");
  _e1995.appendChild(_e1996);
  const _e1997 = WF.h("td", {}, "Opacity fade");
  _e1995.appendChild(_e1997);
  const _e1998 = WF.h("td", {}, "Card(elevated, fadeIn)");
  _e1995.appendChild(_e1998);
  _e1990.appendChild(_e1995);
  const _e1999 = WF.h("tr", {});
  const _e2000 = WF.h("td", {}, "slideUp / slideDown");
  _e1999.appendChild(_e2000);
  const _e2001 = WF.h("td", {}, "Vertical slide + fade");
  _e1999.appendChild(_e2001);
  const _e2002 = WF.h("td", {}, "Heading(\"Hi\", h1, slideUp)");
  _e1999.appendChild(_e2002);
  _e1990.appendChild(_e1999);
  const _e2003 = WF.h("tr", {});
  const _e2004 = WF.h("td", {}, "slideLeft / slideRight");
  _e2003.appendChild(_e2004);
  const _e2005 = WF.h("td", {}, "Horizontal slide + fade");
  _e2003.appendChild(_e2005);
  const _e2006 = WF.h("td", {}, "Text(\"Hello\", slideLeft)");
  _e2003.appendChild(_e2006);
  _e1990.appendChild(_e2003);
  const _e2007 = WF.h("tr", {});
  const _e2008 = WF.h("td", {}, "scaleIn / scaleOut");
  _e2007.appendChild(_e2008);
  const _e2009 = WF.h("td", {}, "Scale from/to 90%");
  _e2007.appendChild(_e2009);
  const _e2010 = WF.h("td", {}, "Badge(\"New\", scaleIn)");
  _e2007.appendChild(_e2010);
  _e1990.appendChild(_e2007);
  const _e2011 = WF.h("tr", {});
  const _e2012 = WF.h("td", {}, "bounce");
  _e2011.appendChild(_e2012);
  const _e2013 = WF.h("td", {}, "Bouncy entrance");
  _e2011.appendChild(_e2013);
  const _e2014 = WF.h("td", {}, "Button(\"Go\", bounce)");
  _e2011.appendChild(_e2014);
  _e1990.appendChild(_e2011);
  const _e2015 = WF.h("tr", {});
  const _e2016 = WF.h("td", {}, "shake");
  _e2015.appendChild(_e2016);
  const _e2017 = WF.h("td", {}, "Horizontal shake");
  _e2015.appendChild(_e2017);
  const _e2018 = WF.h("td", {}, "Alert(\"Error!\", shake)");
  _e2015.appendChild(_e2018);
  _e1990.appendChild(_e2015);
  const _e2019 = WF.h("tr", {});
  const _e2020 = WF.h("td", {}, "pulse");
  _e2019.appendChild(_e2020);
  const _e2021 = WF.h("td", {}, "Scale pulse (infinite)");
  _e2019.appendChild(_e2021);
  const _e2022 = WF.h("td", {}, "Badge(\"Live\", pulse)");
  _e2019.appendChild(_e2022);
  _e1990.appendChild(_e2019);
  const _e2023 = WF.h("tr", {});
  const _e2024 = WF.h("td", {}, "spin");
  _e2023.appendChild(_e2024);
  const _e2025 = WF.h("td", {}, "360-degree rotation");
  _e2023.appendChild(_e2025);
  const _e2026 = WF.h("td", {}, "Spinner(spin)");
  _e2023.appendChild(_e2026);
  _e1990.appendChild(_e2023);
  _e1894.appendChild(_e1990);
  const _e2027 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2027);
  const _e2028 = WF.h("hr", { className: "wf-divider" });
  _e1894.appendChild(_e2028);
  const _e2029 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2029);
  const _e2030 = WF.h("h2", { className: "wf-heading" }, "Conditional Animations");
  _e1894.appendChild(_e2030);
  const _e2031 = WF.h("p", { className: "wf-text" }, "Attach enter and exit animations to if blocks.");
  _e1894.appendChild(_e2031);
  const _e2032 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2032);
  const _e2033 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e2034 = WF.h("div", { className: "wf-card__body" });
  const _e2035 = WF.h("code", { className: "wf-code wf-code--block" }, "if visible, animate(slideUp, fadeOut) {\n    Card { Text(\"Appears with slideUp, exits with fadeOut\") }\n}\n\nif expanded, animate(scaleIn, scaleOut) {\n    Text(\"Scales in and out\")\n}");
  _e2034.appendChild(_e2035);
  _e2033.appendChild(_e2034);
  _e1894.appendChild(_e2033);
  const _e2036 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2036);
  const _e2037 = WF.h("hr", { className: "wf-divider" });
  _e1894.appendChild(_e2037);
  const _e2038 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2038);
  const _e2039 = WF.h("h2", { className: "wf-heading" }, "List Stagger");
  _e1894.appendChild(_e2039);
  const _e2040 = WF.h("p", { className: "wf-text" }, "Animate list items with staggered delays.");
  _e1894.appendChild(_e2040);
  const _e2041 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2041);
  const _e2042 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e2043 = WF.h("div", { className: "wf-card__body" });
  const _e2044 = WF.h("code", { className: "wf-code wf-code--block" }, "for item in items, animate(slideUp, fadeOut, stagger: \"50ms\") {\n    Card { Text(item.name) }\n}");
  _e2043.appendChild(_e2044);
  _e2042.appendChild(_e2043);
  _e1894.appendChild(_e2042);
  const _e2045 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2045);
  const _e2046 = WF.h("hr", { className: "wf-divider" });
  _e1894.appendChild(_e2046);
  const _e2047 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2047);
  const _e2048 = WF.h("h2", { className: "wf-heading" }, "Transition Blocks");
  _e1894.appendChild(_e2048);
  const _e2049 = WF.h("p", { className: "wf-text" }, "Smooth CSS transitions on property changes.");
  _e1894.appendChild(_e2049);
  const _e2050 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2050);
  const _e2051 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e2052 = WF.h("div", { className: "wf-card__body" });
  const _e2053 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Hover me\") {\n    transition {\n        background 200ms ease\n        transform 150ms spring\n    }\n}");
  _e2052.appendChild(_e2053);
  _e2051.appendChild(_e2052);
  _e1894.appendChild(_e2051);
  const _e2054 = WF.h("div", { className: "wf-spacer" });
  _e1894.appendChild(_e2054);
  _root.appendChild(_e1894);
  return _root;
}

(function() {
  const _app = document.getElementById('app');
  _app.innerHTML = '';
  const _e2055 = Component_NavBar({});
  _app.appendChild(_e2055);
  const _e2056 = WF.h("div", { className: "wf-row" });
  _app.appendChild(_e2056);
  const _e2057 = Component_DocSidebar({});
  _e2056.appendChild(_e2057);
  const _routerEl = document.createElement('main');
  _routerEl.id = 'wf-main';
  _routerEl.style.flex = '1';
  _e2056.appendChild(_routerEl);
  const _e2058 = Component_SiteFooter({});
  _app.appendChild(_e2058);
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
