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
  const _e12 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("footer.built"));
  _e11.appendChild(_e12);
  const _e13 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e14 = WF.h("a", { className: "wf-link", href: WF._basePath + "/" });
  const _e15 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("nav.home"));
  _e14.appendChild(_e15);
  _e13.appendChild(_e14);
  const _e16 = WF.h("a", { className: "wf-link", href: WF._basePath + "/getting-started" });
  const _e17 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("footer.docs"));
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
  const _e23 = WF.h("a", { className: "wf-link", href: WF._basePath + "/" });
  const _e24 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.home"));
  _e23.appendChild(_e24);
  _e22.appendChild(_e23);
  const _e25 = WF.h("a", { className: "wf-link", href: WF._basePath + "/getting-started" });
  const _e26 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.start"));
  _e25.appendChild(_e26);
  _e22.appendChild(_e25);
  const _e27 = WF.h("a", { className: "wf-link", href: WF._basePath + "/guide" });
  const _e28 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.guide"));
  _e27.appendChild(_e28);
  _e22.appendChild(_e27);
  const _e29 = WF.h("a", { className: "wf-link", href: WF._basePath + "/components" });
  const _e30 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.components"));
  _e29.appendChild(_e30);
  _e22.appendChild(_e29);
  const _e31 = WF.h("a", { className: "wf-link", href: WF._basePath + "/styling" });
  const _e32 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.styling"));
  _e31.appendChild(_e32);
  _e22.appendChild(_e31);
  const _e33 = WF.h("a", { className: "wf-link", href: WF._basePath + "/animation" });
  const _e34 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.animation"));
  _e33.appendChild(_e34);
  _e22.appendChild(_e33);
  const _e35 = WF.h("a", { className: "wf-link", href: WF._basePath + "/i18n" });
  const _e36 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.i18n"));
  _e35.appendChild(_e36);
  _e22.appendChild(_e35);
  const _e37 = WF.h("a", { className: "wf-link", href: WF._basePath + "/ssg" });
  const _e38 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.ssg"));
  _e37.appendChild(_e38);
  _e22.appendChild(_e37);
  const _e39 = WF.h("a", { className: "wf-link", href: WF._basePath + "/pdf" });
  const _e40 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.pdf"));
  _e39.appendChild(_e40);
  _e22.appendChild(_e39);
  const _e41 = WF.h("a", { className: "wf-link", href: WF._basePath + "/template-engine" });
  const _e42 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.template"));
  _e41.appendChild(_e42);
  _e22.appendChild(_e41);
  const _e43 = WF.h("a", { className: "wf-link", href: WF._basePath + "/accessibility" });
  const _e44 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.a11y"));
  _e43.appendChild(_e44);
  _e22.appendChild(_e43);
  const _e45 = WF.h("a", { className: "wf-link", href: WF._basePath + "/cli" });
  const _e46 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("nav.cli"));
  _e45.appendChild(_e46);
  _e22.appendChild(_e45);
  _e19.appendChild(_e22);
  const _e47 = WF.h("div", { className: "wf-navbar__actions" });
  const _e48 = WF.h("button", { className: "wf-btn wf-btn--small", "on:click": (e) => { WF.i18n.setLocale("en"); } }, "EN");
  _e47.appendChild(_e48);
  const _e49 = WF.h("button", { className: "wf-btn wf-btn--small", "on:click": (e) => { WF.i18n.setLocale("ar"); } }, "AR");
  _e47.appendChild(_e49);
  _e19.appendChild(_e47);
  _frag.appendChild(_e19);
  return _frag;
}

function Page_Ssg(params) {
  const _root = document.createDocumentFragment();
  const _e50 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e51 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e51);
  const _e52 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Static Site Generation (SSG)");
  _e50.appendChild(_e52);
  const _e53 = WF.h("p", { className: "wf-text wf-text--muted" }, "Pre-render pages to HTML at build time for instant content visibility. JavaScript hydrates the page for interactivity.");
  _e50.appendChild(_e53);
  const _e54 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e54);
  const _e55 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Enable SSG");
  _e50.appendChild(_e55);
  const _e56 = WF.h("p", { className: "wf-text" }, "One config flag is all you need.");
  _e50.appendChild(_e56);
  const _e57 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e57);
  const _e58 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e59 = WF.h("div", { className: "wf-card__body" });
  const _e60 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"build\": {\n    \"ssg\": true\n  }\n}");
  _e59.appendChild(_e60);
  _e58.appendChild(_e59);
  _e50.appendChild(_e58);
  const _e61 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e61);
  const _e62 = WF.h("hr", { className: "wf-divider" });
  _e50.appendChild(_e62);
  const _e63 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e63);
  const _e64 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "How It Works");
  _e50.appendChild(_e64);
  const _e65 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e66 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e67 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e68 = WF.h("div", { className: "wf-card__body" });
  const _e69 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "1. Build");
  _e68.appendChild(_e69);
  const _e70 = WF.h("p", { className: "wf-text wf-text--muted" }, "The compiler walks the AST for each page and generates static HTML from the component tree.");
  _e68.appendChild(_e70);
  _e67.appendChild(_e68);
  _e66.appendChild(_e67);
  _e65.appendChild(_e66);
  const _e71 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e72 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e73 = WF.h("div", { className: "wf-card__body" });
  const _e74 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "2. Serve");
  _e73.appendChild(_e74);
  const _e75 = WF.h("p", { className: "wf-text wf-text--muted" }, "The browser loads pre-rendered HTML. Content is visible immediately — no blank white screen.");
  _e73.appendChild(_e75);
  _e72.appendChild(_e73);
  _e71.appendChild(_e72);
  _e65.appendChild(_e71);
  const _e76 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e77 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e78 = WF.h("div", { className: "wf-card__body" });
  const _e79 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "3. Hydrate");
  _e78.appendChild(_e79);
  const _e80 = WF.h("p", { className: "wf-text wf-text--muted" }, "JavaScript runs and hydrates the page: attaches events, initializes state, fills dynamic content.");
  _e78.appendChild(_e80);
  _e77.appendChild(_e78);
  _e76.appendChild(_e77);
  _e65.appendChild(_e76);
  _e50.appendChild(_e65);
  const _e81 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e81);
  const _e82 = WF.h("hr", { className: "wf-divider" });
  _e50.appendChild(_e82);
  const _e83 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e83);
  const _e84 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Build Output");
  _e50.appendChild(_e84);
  const _e85 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e86 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e87 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e88 = WF.h("div", { className: "wf-card__body" });
  const _e89 = WF.h("p", { className: "wf-text wf-text--bold" }, "SPA (default)");
  _e88.appendChild(_e89);
  const _e90 = WF.h("code", { className: "wf-code wf-code--block" }, "build/\n├── index.html       # Empty shell\n├── app.js\n└── styles.css");
  _e88.appendChild(_e90);
  _e87.appendChild(_e88);
  _e86.appendChild(_e87);
  _e85.appendChild(_e86);
  const _e91 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e92 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e93 = WF.h("div", { className: "wf-card__body" });
  const _e94 = WF.h("p", { className: "wf-text wf-text--bold" }, "SSG mode");
  _e93.appendChild(_e94);
  const _e95 = WF.h("code", { className: "wf-code wf-code--block" }, "build/\n├── index.html       # Pre-rendered /\n├── about/\n│   └── index.html   # Pre-rendered /about\n├── blog/\n│   └── index.html   # Pre-rendered /blog\n├── app.js\n└── styles.css");
  _e93.appendChild(_e95);
  _e92.appendChild(_e93);
  _e91.appendChild(_e92);
  _e85.appendChild(_e91);
  _e50.appendChild(_e85);
  const _e96 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e96);
  const _e97 = WF.h("hr", { className: "wf-divider" });
  _e50.appendChild(_e97);
  const _e98 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e98);
  const _e99 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "What Gets Pre-Rendered");
  _e50.appendChild(_e99);
  const _e100 = WF.h("table", { className: "wf-table" });
  const _e101 = WF.h("thead", {});
  const _e102 = WF.h("td", {}, "Element");
  _e101.appendChild(_e102);
  const _e103 = WF.h("td", {}, "SSG Behavior");
  _e101.appendChild(_e103);
  _e100.appendChild(_e101);
  const _e104 = WF.h("tr", {});
  const _e105 = WF.h("td", {}, "Static text, headings, components");
  _e104.appendChild(_e105);
  const _e106 = WF.h("td", {}, "Fully rendered to HTML");
  _e104.appendChild(_e106);
  _e100.appendChild(_e104);
  const _e107 = WF.h("tr", {});
  const _e108 = WF.h("td", {}, "Container, Row, Column, Card, etc.");
  _e107.appendChild(_e108);
  const _e109 = WF.h("td", {}, "Full HTML with classes");
  _e107.appendChild(_e109);
  _e100.appendChild(_e107);
  const _e110 = WF.h("tr", {});
  const _e111 = WF.h("td", {}, "Modifiers (primary, large, etc.)");
  _e110.appendChild(_e111);
  const _e112 = WF.h("td", {}, "CSS classes applied");
  _e110.appendChild(_e112);
  _e100.appendChild(_e110);
  const _e113 = WF.h("tr", {});
  const _e114 = WF.h("td", {}, "Animation modifiers (fadeIn, etc.)");
  _e113.appendChild(_e114);
  const _e115 = WF.h("td", {}, "Animation classes applied");
  _e113.appendChild(_e115);
  _e100.appendChild(_e113);
  const _e116 = WF.h("tr", {});
  const _e117 = WF.h("td", {}, "t() i18n calls");
  _e116.appendChild(_e117);
  const _e118 = WF.h("td", {}, "Default locale text rendered");
  _e116.appendChild(_e118);
  _e100.appendChild(_e116);
  const _e119 = WF.h("tr", {});
  const _e120 = WF.h("td", {}, "State-dependent text");
  _e119.appendChild(_e120);
  const _e121 = WF.h("td", {}, "Empty placeholder (filled by JS)");
  _e119.appendChild(_e121);
  _e100.appendChild(_e119);
  const _e122 = WF.h("tr", {});
  const _e123 = WF.h("td", {}, "if / for blocks");
  _e122.appendChild(_e123);
  const _e124 = WF.h("td", {}, "Comment placeholder (filled by JS)");
  _e122.appendChild(_e124);
  _e100.appendChild(_e122);
  const _e125 = WF.h("tr", {});
  const _e126 = WF.h("td", {}, "show blocks");
  _e125.appendChild(_e126);
  const _e127 = WF.h("td", {}, "Rendered but hidden (display:none)");
  _e125.appendChild(_e127);
  _e100.appendChild(_e125);
  const _e128 = WF.h("tr", {});
  const _e129 = WF.h("td", {}, "fetch blocks");
  _e128.appendChild(_e129);
  const _e130 = WF.h("td", {}, "Loading block if present, else placeholder");
  _e128.appendChild(_e130);
  _e100.appendChild(_e128);
  const _e131 = WF.h("tr", {});
  const _e132 = WF.h("td", {}, "Event handlers");
  _e131.appendChild(_e132);
  const _e133 = WF.h("td", {}, "Attached during hydration");
  _e131.appendChild(_e133);
  _e100.appendChild(_e131);
  _e50.appendChild(_e100);
  const _e134 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e134);
  const _e135 = WF.h("hr", { className: "wf-divider" });
  _e50.appendChild(_e135);
  const _e136 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e136);
  const _e137 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Dynamic Routes");
  _e50.appendChild(_e137);
  const _e138 = WF.h("p", { className: "wf-text wf-text--muted" }, "Pages with :param segments (e.g., /user/:id) cannot be pre-rendered — they fall back to client-side rendering.");
  _e50.appendChild(_e138);
  const _e139 = WF.h("div", { className: "wf-spacer" });
  _e50.appendChild(_e139);
  _root.appendChild(_e50);
  return _root;
}

function Page_NotFound(params) {
  const _root = document.createDocumentFragment();
  const _e140 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e141 = WF.h("div", { className: "wf-spacer" });
  _e140.appendChild(_e141);
  const _e142 = WF.h("div", { className: "wf-stack wf-stack--gap-md" });
  const _e143 = WF.h("h2", { className: "wf-heading wf-heading--h1 wf-text--center wf-heading--primary" }, "404");
  _e142.appendChild(_e143);
  const _e144 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, "Page Not Found");
  _e142.appendChild(_e144);
  const _e145 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, "The page you are looking for does not exist or has been moved.");
  _e142.appendChild(_e145);
  const _e146 = WF.h("div", { className: "wf-spacer" });
  _e142.appendChild(_e146);
  const _e147 = WF.h("div", { className: "wf-row" });
  const _e148 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/"); } }, "Go Home");
  _e147.appendChild(_e148);
  _e142.appendChild(_e147);
  _e140.appendChild(_e142);
  const _e149 = WF.h("div", { className: "wf-spacer" });
  _e140.appendChild(_e149);
  _root.appendChild(_e140);
  return _root;
}

function Page_Styling(params) {
  const _root = document.createDocumentFragment();
  const _e150 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e151 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e151);
  const _e152 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Design System & Styling");
  _e150.appendChild(_e152);
  const _e153 = WF.h("p", { className: "wf-text wf-text--muted" }, "Token-based design system. Every component uses design tokens for colors, spacing, typography. Change the entire look with a config update.");
  _e150.appendChild(_e153);
  const _e154 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e154);
  const _e155 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Variant Modifiers");
  _e150.appendChild(_e155);
  const _e156 = WF.h("p", { className: "wf-text" }, "Apply common styles with modifier keywords.");
  _e150.appendChild(_e156);
  const _e157 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e157);
  const _e158 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e159 = WF.h("div", { className: "wf-card__header" });
  const _e160 = WF.h("p", { className: "wf-text wf-text--bold" }, "Size Modifiers");
  _e159.appendChild(_e160);
  _e158.appendChild(_e159);
  const _e161 = WF.h("div", { className: "wf-card__body" });
  const _e162 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e163 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Small");
  _e162.appendChild(_e163);
  const _e164 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Medium");
  _e162.appendChild(_e164);
  const _e165 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large" }, "Large");
  _e162.appendChild(_e165);
  _e161.appendChild(_e162);
  _e158.appendChild(_e161);
  _e150.appendChild(_e158);
  const _e166 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e166);
  const _e167 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e168 = WF.h("div", { className: "wf-card__header" });
  const _e169 = WF.h("p", { className: "wf-text wf-text--bold" }, "Color Modifiers");
  _e168.appendChild(_e169);
  _e167.appendChild(_e168);
  const _e170 = WF.h("div", { className: "wf-card__body" });
  const _e171 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e172 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e171.appendChild(_e172);
  const _e173 = WF.h("button", { className: "wf-btn wf-btn--secondary" }, "Secondary");
  _e171.appendChild(_e173);
  const _e174 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e171.appendChild(_e174);
  const _e175 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e171.appendChild(_e175);
  const _e176 = WF.h("button", { className: "wf-btn wf-btn--warning" }, "Warning");
  _e171.appendChild(_e176);
  const _e177 = WF.h("button", { className: "wf-btn wf-btn--info" }, "Info");
  _e171.appendChild(_e177);
  _e170.appendChild(_e171);
  const _e178 = WF.h("div", { className: "wf-spacer" });
  _e170.appendChild(_e178);
  const _e179 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e180 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Primary");
  _e179.appendChild(_e180);
  const _e181 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Success");
  _e179.appendChild(_e181);
  const _e182 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Danger");
  _e179.appendChild(_e182);
  const _e183 = WF.h("span", { className: "wf-badge wf-badge--warning" }, "Warning");
  _e179.appendChild(_e183);
  _e170.appendChild(_e179);
  _e167.appendChild(_e170);
  _e150.appendChild(_e167);
  const _e184 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e184);
  const _e185 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e186 = WF.h("div", { className: "wf-card__header" });
  const _e187 = WF.h("p", { className: "wf-text wf-text--bold" }, "Shape and Elevation");
  _e186.appendChild(_e187);
  _e185.appendChild(_e186);
  const _e188 = WF.h("div", { className: "wf-card__body" });
  const _e189 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e190 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Default");
  _e189.appendChild(_e190);
  const _e191 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--rounded" }, "Rounded");
  _e189.appendChild(_e191);
  const _e192 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--full" }, "Full Width");
  _e189.appendChild(_e192);
  _e188.appendChild(_e189);
  const _e193 = WF.h("div", { className: "wf-spacer" });
  _e188.appendChild(_e193);
  const _e194 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e195 = WF.h("div", { className: "wf-card" });
  const _e196 = WF.h("div", { className: "wf-card__body" });
  const _e197 = WF.h("p", { className: "wf-text" }, "Default");
  _e196.appendChild(_e197);
  _e195.appendChild(_e196);
  _e194.appendChild(_e195);
  const _e198 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e199 = WF.h("div", { className: "wf-card__body" });
  const _e200 = WF.h("p", { className: "wf-text" }, "Elevated");
  _e199.appendChild(_e200);
  _e198.appendChild(_e199);
  _e194.appendChild(_e198);
  const _e201 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e202 = WF.h("div", { className: "wf-card__body" });
  const _e203 = WF.h("p", { className: "wf-text" }, "Outlined");
  _e202.appendChild(_e203);
  _e201.appendChild(_e202);
  _e194.appendChild(_e201);
  _e188.appendChild(_e194);
  _e185.appendChild(_e188);
  _e150.appendChild(_e185);
  const _e204 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e204);
  const _e205 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e206 = WF.h("div", { className: "wf-card__header" });
  const _e207 = WF.h("p", { className: "wf-text wf-text--bold" }, "Text Modifiers");
  _e206.appendChild(_e207);
  _e205.appendChild(_e206);
  const _e208 = WF.h("div", { className: "wf-card__body" });
  const _e209 = WF.h("p", { className: "wf-text wf-text--bold" }, "Bold text.");
  _e208.appendChild(_e209);
  const _e210 = WF.h("p", { className: "wf-text wf-text--italic" }, "Italic text.");
  _e208.appendChild(_e210);
  const _e211 = WF.h("p", { className: "wf-text wf-text--uppercase" }, "Uppercase text.");
  _e208.appendChild(_e211);
  const _e212 = WF.h("p", { className: "wf-text wf-text--muted" }, "Muted text.");
  _e208.appendChild(_e212);
  const _e213 = WF.h("p", { className: "wf-text wf-text--primary" }, "Primary colored text.");
  _e208.appendChild(_e213);
  const _e214 = WF.h("p", { className: "wf-text wf-text--small" }, "Small text.");
  _e208.appendChild(_e214);
  const _e215 = WF.h("p", { className: "wf-text wf-text--large" }, "Large text.");
  _e208.appendChild(_e215);
  _e205.appendChild(_e208);
  _e150.appendChild(_e205);
  const _e216 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e216);
  const _e217 = WF.h("hr", { className: "wf-divider" });
  _e150.appendChild(_e217);
  const _e218 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e218);
  const _e219 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Design Tokens");
  _e150.appendChild(_e219);
  const _e220 = WF.h("p", { className: "wf-text" }, "All styling is built on tokens — CSS custom properties. Override any token in your config.");
  _e150.appendChild(_e220);
  const _e221 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e221);
  const _e222 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e223 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e224 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e225 = WF.h("div", { className: "wf-card__header" });
  const _e226 = WF.h("p", { className: "wf-text wf-text--bold" }, "Colors");
  _e225.appendChild(_e226);
  _e224.appendChild(_e225);
  const _e227 = WF.h("div", { className: "wf-card__body" });
  const _e228 = WF.h("table", { className: "wf-table" });
  const _e229 = WF.h("thead", {});
  const _e230 = WF.h("td", {}, "Token");
  _e229.appendChild(_e230);
  const _e231 = WF.h("td", {}, "Value");
  _e229.appendChild(_e231);
  _e228.appendChild(_e229);
  const _e232 = WF.h("tr", {});
  const _e233 = WF.h("td", {}, "color-primary");
  _e232.appendChild(_e233);
  const _e234 = WF.h("td", {}, "#3B82F6");
  _e232.appendChild(_e234);
  _e228.appendChild(_e232);
  const _e235 = WF.h("tr", {});
  const _e236 = WF.h("td", {}, "color-success");
  _e235.appendChild(_e236);
  const _e237 = WF.h("td", {}, "#22C55E");
  _e235.appendChild(_e237);
  _e228.appendChild(_e235);
  const _e238 = WF.h("tr", {});
  const _e239 = WF.h("td", {}, "color-danger");
  _e238.appendChild(_e239);
  const _e240 = WF.h("td", {}, "#EF4444");
  _e238.appendChild(_e240);
  _e228.appendChild(_e238);
  const _e241 = WF.h("tr", {});
  const _e242 = WF.h("td", {}, "color-warning");
  _e241.appendChild(_e242);
  const _e243 = WF.h("td", {}, "#F59E0B");
  _e241.appendChild(_e243);
  _e228.appendChild(_e241);
  const _e244 = WF.h("tr", {});
  const _e245 = WF.h("td", {}, "color-text");
  _e244.appendChild(_e245);
  const _e246 = WF.h("td", {}, "#0F172A");
  _e244.appendChild(_e246);
  _e228.appendChild(_e244);
  const _e247 = WF.h("tr", {});
  const _e248 = WF.h("td", {}, "color-border");
  _e247.appendChild(_e248);
  const _e249 = WF.h("td", {}, "#E2E8F0");
  _e247.appendChild(_e249);
  _e228.appendChild(_e247);
  _e227.appendChild(_e228);
  _e224.appendChild(_e227);
  _e223.appendChild(_e224);
  _e222.appendChild(_e223);
  const _e250 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e251 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e252 = WF.h("div", { className: "wf-card__header" });
  const _e253 = WF.h("p", { className: "wf-text wf-text--bold" }, "Spacing and Radius");
  _e252.appendChild(_e253);
  _e251.appendChild(_e252);
  const _e254 = WF.h("div", { className: "wf-card__body" });
  const _e255 = WF.h("table", { className: "wf-table" });
  const _e256 = WF.h("thead", {});
  const _e257 = WF.h("td", {}, "Token");
  _e256.appendChild(_e257);
  const _e258 = WF.h("td", {}, "Value");
  _e256.appendChild(_e258);
  _e255.appendChild(_e256);
  const _e259 = WF.h("tr", {});
  const _e260 = WF.h("td", {}, "spacing-xs");
  _e259.appendChild(_e260);
  const _e261 = WF.h("td", {}, "0.25rem");
  _e259.appendChild(_e261);
  _e255.appendChild(_e259);
  const _e262 = WF.h("tr", {});
  const _e263 = WF.h("td", {}, "spacing-sm");
  _e262.appendChild(_e263);
  const _e264 = WF.h("td", {}, "0.5rem");
  _e262.appendChild(_e264);
  _e255.appendChild(_e262);
  const _e265 = WF.h("tr", {});
  const _e266 = WF.h("td", {}, "spacing-md");
  _e265.appendChild(_e266);
  const _e267 = WF.h("td", {}, "1rem");
  _e265.appendChild(_e267);
  _e255.appendChild(_e265);
  const _e268 = WF.h("tr", {});
  const _e269 = WF.h("td", {}, "spacing-lg");
  _e268.appendChild(_e269);
  const _e270 = WF.h("td", {}, "1.5rem");
  _e268.appendChild(_e270);
  _e255.appendChild(_e268);
  const _e271 = WF.h("tr", {});
  const _e272 = WF.h("td", {}, "radius-md");
  _e271.appendChild(_e272);
  const _e273 = WF.h("td", {}, "0.5rem");
  _e271.appendChild(_e273);
  _e255.appendChild(_e271);
  const _e274 = WF.h("tr", {});
  const _e275 = WF.h("td", {}, "radius-full");
  _e274.appendChild(_e275);
  const _e276 = WF.h("td", {}, "9999px");
  _e274.appendChild(_e276);
  _e255.appendChild(_e274);
  _e254.appendChild(_e255);
  _e251.appendChild(_e254);
  _e250.appendChild(_e251);
  _e222.appendChild(_e250);
  _e150.appendChild(_e222);
  const _e277 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e277);
  const _e278 = WF.h("hr", { className: "wf-divider" });
  _e150.appendChild(_e278);
  const _e279 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e279);
  const _e280 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Themes");
  _e150.appendChild(_e280);
  const _e281 = WF.h("p", { className: "wf-text" }, "4 built-in themes. Set in webfluent.app.json. Each preview below shows the actual colors and style of that theme.");
  _e150.appendChild(_e281);
  const _e282 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e282);
  const _e283 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e284 = WF.h("div", { className: "wf-card" });
  const _e285 = WF.h("div", { className: "wf-card__body" });
  const _e286 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e287 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "default");
  _e286.appendChild(_e287);
  const _e288 = WF.h("p", { className: "wf-text wf-text--bold" }, "Default");
  _e286.appendChild(_e288);
  _e285.appendChild(_e286);
  const _e289 = WF.h("div", { className: "wf-spacer" });
  _e285.appendChild(_e289);
  const _e290 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Clean, modern light theme with blue primary.");
  _e285.appendChild(_e290);
  const _e291 = WF.h("div", { className: "wf-spacer" });
  _e285.appendChild(_e291);
  const _e292 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e293 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Primary");
  _e292.appendChild(_e293);
  const _e294 = WF.h("button", { className: "wf-btn wf-btn--success wf-btn--small" }, "Success");
  _e292.appendChild(_e294);
  const _e295 = WF.h("span", { className: "wf-badge wf-badge--info" }, "Tag");
  _e292.appendChild(_e295);
  _e285.appendChild(_e292);
  const _e296 = WF.h("div", { className: "wf-spacer" });
  _e285.appendChild(_e296);
  const _e297 = WF.h("progress", { className: "wf-progress wf-progress--primary", value: 65, max: 100 });
  _e285.appendChild(_e297);
  const _e298 = WF.h("div", { className: "wf-spacer" });
  _e285.appendChild(_e298);
  const _e299 = WF.h("code", { className: "wf-code wf-code--block" }, "\"theme\": { \"name\": \"default\" }");
  _e285.appendChild(_e299);
  _e284.appendChild(_e285);
  _e284.style.background = "#ffffff";
  _e284.style.border = "1px solid #E2E8F0";
  _e284.style.borderRadius = "0.75rem";
  _e283.appendChild(_e284);
  const _e300 = WF.h("div", { className: "wf-card" });
  const _e301 = WF.h("div", { className: "wf-card__body" });
  const _e302 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e303 = WF.h("span", { className: "wf-badge wf-badge--secondary" }, "dark");
  _e302.appendChild(_e303);
  const _e304 = WF.h("p", { className: "wf-text wf-text--bold" }, "Dark");
  _e302.appendChild(_e304);
  _e301.appendChild(_e302);
  const _e305 = WF.h("div", { className: "wf-spacer" });
  _e301.appendChild(_e305);
  const _e306 = WF.h("p", { className: "wf-text wf-text--small" }, "Dark backgrounds with light text and vibrant accents.");
  _e301.appendChild(_e306);
  const _e307 = WF.h("div", { className: "wf-spacer" });
  _e301.appendChild(_e307);
  const _e308 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e309 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Primary");
  _e308.appendChild(_e309);
  const _e310 = WF.h("button", { className: "wf-btn wf-btn--danger wf-btn--small" }, "Danger");
  _e308.appendChild(_e310);
  const _e311 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Live");
  _e308.appendChild(_e311);
  _e301.appendChild(_e308);
  const _e312 = WF.h("div", { className: "wf-spacer" });
  _e301.appendChild(_e312);
  const _e313 = WF.h("progress", { className: "wf-progress wf-progress--info", value: 80, max: 100 });
  _e301.appendChild(_e313);
  const _e314 = WF.h("div", { className: "wf-spacer" });
  _e301.appendChild(_e314);
  const _e315 = WF.h("code", { className: "wf-code wf-code--block" }, "\"theme\": { \"name\": \"dark\" }");
  _e301.appendChild(_e315);
  _e300.appendChild(_e301);
  _e300.style.background = "#0F172A";
  _e300.style.color = "#E2E8F0";
  _e300.style.border = "1px solid #334155";
  _e300.style.borderRadius = "0.75rem";
  _e283.appendChild(_e300);
  const _e316 = WF.h("div", { className: "wf-card" });
  const _e317 = WF.h("div", { className: "wf-card__body" });
  const _e318 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e319 = WF.h("span", { className: "wf-badge" }, "minimal");
  _e318.appendChild(_e319);
  const _e320 = WF.h("p", { className: "wf-text wf-text--bold" }, "Minimal");
  _e318.appendChild(_e320);
  _e317.appendChild(_e318);
  const _e321 = WF.h("div", { className: "wf-spacer" });
  _e317.appendChild(_e321);
  const _e322 = WF.h("p", { className: "wf-text wf-text--small" }, "Black and white. No shadows, no border-radius. Pure content.");
  _e317.appendChild(_e322);
  const _e323 = WF.h("div", { className: "wf-spacer" });
  _e317.appendChild(_e323);
  const _e324 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e325 = WF.h("button", { className: "wf-btn wf-btn--small" }, "Action");
  _e324.appendChild(_e325);
  const _e326 = WF.h("span", { className: "wf-badge" }, "Note");
  _e324.appendChild(_e326);
  _e317.appendChild(_e324);
  const _e327 = WF.h("div", { className: "wf-spacer" });
  _e317.appendChild(_e327);
  const _e328 = WF.h("progress", { className: "wf-progress", value: 50, max: 100 });
  _e317.appendChild(_e328);
  const _e329 = WF.h("div", { className: "wf-spacer" });
  _e317.appendChild(_e329);
  const _e330 = WF.h("code", { className: "wf-code wf-code--block" }, "\"theme\": { \"name\": \"minimal\" }");
  _e317.appendChild(_e330);
  _e316.appendChild(_e317);
  _e316.style.background = "#ffffff";
  _e316.style.border = "2px solid #000000";
  _e316.style.borderRadius = "0";
  _e283.appendChild(_e316);
  const _e331 = WF.h("div", { className: "wf-card" });
  const _e332 = WF.h("div", { className: "wf-card__body" });
  const _e333 = WF.h("div", { className: "wf-row wf-row--gap-sm wf-row--center" });
  const _e334 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "brutalist");
  _e333.appendChild(_e334);
  const _e335 = WF.h("p", { className: "wf-text wf-text--bold" }, "Brutalist");
  _e333.appendChild(_e335);
  _e332.appendChild(_e333);
  const _e336 = WF.h("div", { className: "wf-spacer" });
  _e332.appendChild(_e336);
  const _e337 = WF.h("p", { className: "wf-text wf-text--small" }, "Monospace font, bold red primary, hard offset shadows.");
  _e332.appendChild(_e337);
  const _e338 = WF.h("div", { className: "wf-spacer" });
  _e332.appendChild(_e338);
  const _e339 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e340 = WF.h("button", { className: "wf-btn wf-btn--danger wf-btn--small" }, "Action");
  _e339.appendChild(_e340);
  const _e341 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Alert");
  _e339.appendChild(_e341);
  _e332.appendChild(_e339);
  const _e342 = WF.h("div", { className: "wf-spacer" });
  _e332.appendChild(_e342);
  const _e343 = WF.h("progress", { className: "wf-progress wf-progress--danger", value: 90, max: 100 });
  _e332.appendChild(_e343);
  const _e344 = WF.h("div", { className: "wf-spacer" });
  _e332.appendChild(_e344);
  const _e345 = WF.h("code", { className: "wf-code wf-code--block" }, "\"theme\": { \"name\": \"brutalist\" }");
  _e332.appendChild(_e345);
  _e331.appendChild(_e332);
  _e331.style.background = "#ffffff";
  _e331.style.border = "3px solid #000000";
  _e331.style.borderRadius = "0";
  _e331.style.boxShadow = "4px 4px 0 #000000";
  _e331.style.fontFamily = "monospace";
  _e283.appendChild(_e331);
  _e150.appendChild(_e283);
  const _e346 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e346);
  const _e347 = WF.h("hr", { className: "wf-divider" });
  _e150.appendChild(_e347);
  const _e348 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e348);
  const _e349 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Custom Tokens");
  _e150.appendChild(_e349);
  const _e350 = WF.h("p", { className: "wf-text" }, "Override any design token in your config to customize the theme.");
  _e150.appendChild(_e350);
  const _e351 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e351);
  const _e352 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e353 = WF.h("div", { className: "wf-card__body" });
  const _e354 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"theme\": {\n    \"name\": \"default\",\n    \"tokens\": {\n      \"color-primary\": \"#8B5CF6\",\n      \"color-secondary\": \"#EC4899\",\n      \"font-family\": \"Poppins, sans-serif\",\n      \"radius-md\": \"1rem\"\n    }\n  }\n}");
  _e353.appendChild(_e354);
  _e352.appendChild(_e353);
  _e150.appendChild(_e352);
  const _e355 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e355);
  const _e356 = WF.h("hr", { className: "wf-divider" });
  _e150.appendChild(_e356);
  const _e357 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e357);
  const _e358 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Style Blocks");
  _e150.appendChild(_e358);
  const _e359 = WF.h("p", { className: "wf-text" }, "Override styles on any component with inline style blocks.");
  _e150.appendChild(_e359);
  const _e360 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e360);
  const _e361 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e362 = WF.h("div", { className: "wf-card__body" });
  const _e363 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Custom\") {\n    style {\n        background: \"#8B5CF6\"\n        padding: xl\n        radius: lg\n    }\n}");
  _e362.appendChild(_e363);
  _e361.appendChild(_e362);
  _e150.appendChild(_e361);
  const _e364 = WF.h("div", { className: "wf-spacer" });
  _e150.appendChild(_e364);
  _root.appendChild(_e150);
  return _root;
}

function Page_TemplateEngine(params) {
  const _root = document.createDocumentFragment();
  const _e365 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e366 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e366);
  const _e367 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, () => WF.i18n.t("tpl.title"));
  _e365.appendChild(_e367);
  const _e368 = WF.h("p", { className: "wf-text wf-text--muted" }, () => WF.i18n.t("tpl.subtitle"));
  _e365.appendChild(_e368);
  const _e369 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e369);
  const _e370 = WF.h("div", { className: "wf-alert wf-alert--info" }, "WebFluent can be used as a server-side template engine from Rust and Node.js to render .wf templates into HTML or PDF with JSON data.");
  _e365.appendChild(_e370);
  const _e371 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e371);
  const _e372 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "CLI Usage");
  _e365.appendChild(_e372);
  const _e373 = WF.h("p", { className: "wf-text" }, "Render any .wf template with JSON data directly from the command line.");
  _e365.appendChild(_e373);
  const _e374 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e374);
  const _e375 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e376 = WF.h("div", { className: "wf-card__body" });
  const _e377 = WF.h("code", { className: "wf-code wf-code--block" }, "# Render to HTML\nwf render template.wf --data data.json --format html -o output.html\n\n# Render to HTML fragment (no <html> wrapper)\nwf render template.wf --data data.json --format fragment\n\n# Render to PDF\nwf render template.wf --data data.json --format pdf -o report.pdf\n\n# Pipe JSON from stdin\necho '{\"name\":\"Monzer\"}' | wf render template.wf --format html\n\n# With theme\nwf render template.wf --data data.json --format html --theme dark");
  _e376.appendChild(_e377);
  _e375.appendChild(_e376);
  _e365.appendChild(_e375);
  const _e378 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e378);
  const _e379 = WF.h("table", { className: "wf-table" });
  const _e380 = WF.h("thead", {});
  const _e381 = WF.h("td", {}, "Option");
  _e380.appendChild(_e381);
  const _e382 = WF.h("td", {}, "Description");
  _e380.appendChild(_e382);
  _e379.appendChild(_e380);
  const _e383 = WF.h("tr", {});
  const _e384 = WF.h("td", {}, "template");
  _e383.appendChild(_e384);
  const _e385 = WF.h("td", {}, "Path to the .wf template file");
  _e383.appendChild(_e385);
  _e379.appendChild(_e383);
  const _e386 = WF.h("tr", {});
  const _e387 = WF.h("td", {}, "--data");
  _e386.appendChild(_e387);
  const _e388 = WF.h("td", {}, "Path to JSON data file (reads stdin if omitted)");
  _e386.appendChild(_e388);
  _e379.appendChild(_e386);
  const _e389 = WF.h("tr", {});
  const _e390 = WF.h("td", {}, "--format, -f");
  _e389.appendChild(_e390);
  const _e391 = WF.h("td", {}, "Output format: html, fragment, or pdf");
  _e389.appendChild(_e391);
  _e379.appendChild(_e389);
  const _e392 = WF.h("tr", {});
  const _e393 = WF.h("td", {}, "--output, -o");
  _e392.appendChild(_e393);
  const _e394 = WF.h("td", {}, "Output file path (stdout if omitted)");
  _e392.appendChild(_e394);
  _e379.appendChild(_e392);
  const _e395 = WF.h("tr", {});
  const _e396 = WF.h("td", {}, "--theme");
  _e395.appendChild(_e396);
  const _e397 = WF.h("td", {}, "Theme name (default: \"default\")");
  _e395.appendChild(_e397);
  _e379.appendChild(_e395);
  _e365.appendChild(_e379);
  const _e398 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e398);
  const _e399 = WF.h("hr", { className: "wf-divider" });
  _e365.appendChild(_e399);
  const _e400 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e400);
  const _e401 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Template Syntax");
  _e365.appendChild(_e401);
  const _e402 = WF.h("p", { className: "wf-text" }, "Templates use standard .wf syntax. Data is passed as a JSON object — top-level keys become template variables.");
  _e365.appendChild(_e402);
  const _e403 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e403);
  const _e404 = WF.h("div", { className: "wf-grid wf-grid--gap-lg", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e405 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e406 = WF.h("div", { className: "wf-card__header" });
  const _e407 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Template");
  _e406.appendChild(_e407);
  const _e408 = WF.h("p", { className: "wf-text wf-text--bold" }, "invoice.wf");
  _e406.appendChild(_e408);
  _e405.appendChild(_e406);
  const _e409 = WF.h("div", { className: "wf-card__body" });
  const _e410 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Invoice (path: \"/\", title: \"Invoice\") {\n    Container {\n        Heading(\"Invoice #{number}\", h1)\n        Text(\"Customer: {customer.name}\")\n\n        Table {\n            Thead { Trow { Tcell(\"Item\") Tcell(\"Price\") } }\n            for item in items {\n                Trow {\n                    Tcell(item.name)\n                    Tcell(\"${item.price}\")\n                }\n            }\n        }\n\n        if paid {\n            Badge(\"PAID\", success)\n        } else {\n            Badge(\"UNPAID\", danger)\n        }\n    }\n}");
  _e409.appendChild(_e410);
  _e405.appendChild(_e409);
  _e404.appendChild(_e405);
  const _e411 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e412 = WF.h("div", { className: "wf-card__header" });
  const _e413 = WF.h("span", { className: "wf-badge wf-badge--info" }, "Data");
  _e412.appendChild(_e413);
  const _e414 = WF.h("p", { className: "wf-text wf-text--bold" }, "data.json");
  _e412.appendChild(_e414);
  _e411.appendChild(_e412);
  const _e415 = WF.h("div", { className: "wf-card__body" });
  const _e416 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"number\": \"INV-001\",\n  \"customer\": { \"name\": \"Acme Corp\" },\n  \"items\": [\n    { \"name\": \"Widget\", \"price\": 9.99 },\n    { \"name\": \"Gadget\", \"price\": 24.99 }\n  ],\n  \"paid\": true\n}");
  _e415.appendChild(_e416);
  _e411.appendChild(_e415);
  _e404.appendChild(_e411);
  _e365.appendChild(_e404);
  const _e417 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e417);
  const _e418 = WF.h("hr", { className: "wf-divider" });
  _e365.appendChild(_e418);
  const _e419 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e419);
  const _e420 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Rust API");
  _e365.appendChild(_e420);
  const _e421 = WF.h("p", { className: "wf-text" }, "Add WebFluent as a library dependency to use templates in your Rust application.");
  _e365.appendChild(_e421);
  const _e422 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e422);
  const _e423 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e424 = WF.h("div", { className: "wf-card__header" });
  const _e425 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Cargo.toml");
  _e424.appendChild(_e425);
  _e423.appendChild(_e424);
  const _e426 = WF.h("div", { className: "wf-card__body" });
  const _e427 = WF.h("code", { className: "wf-code wf-code--block" }, "[dependencies]\nwebfluent = \"0.2\"");
  _e426.appendChild(_e427);
  _e423.appendChild(_e426);
  _e365.appendChild(_e423);
  const _e428 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e428);
  const _e429 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e430 = WF.h("div", { className: "wf-card__header" });
  const _e431 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "main.rs");
  _e430.appendChild(_e431);
  _e429.appendChild(_e430);
  const _e432 = WF.h("div", { className: "wf-card__body" });
  const _e433 = WF.h("code", { className: "wf-code wf-code--block" }, "use webfluent::Template;\nuse serde_json::json;\n\nfn main() -> webfluent::Result<()> {\n    let tpl = Template::from_file(\"templates/invoice.wf\")?;\n\n    // HTML document (with embedded CSS)\n    let html = tpl.render_html(&json!({\n        \"number\": \"INV-001\",\n        \"customer\": { \"name\": \"Acme Corp\" },\n        \"items\": [{ \"name\": \"Widget\", \"price\": 9.99 }],\n        \"paid\": true\n    }))?;\n\n    // HTML fragment (no wrapper)\n    let fragment = tpl.render_html_fragment(&data)?;\n\n    // PDF bytes\n    let pdf_bytes = tpl.render_pdf(&data)?;\n    std::fs::write(\"invoice.pdf\", pdf_bytes)?;\n\n    // With custom theme\n    let dark = Template::from_file(\"invoice.wf\")?\n        .with_theme(\"dark\")\n        .with_tokens(&[(\"color-primary\", \"#8B5CF6\")]);\n    let html = dark.render_html(&data)?;\n\n    Ok(())\n}");
  _e432.appendChild(_e433);
  _e429.appendChild(_e432);
  _e365.appendChild(_e429);
  const _e434 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e434);
  const _e435 = WF.h("hr", { className: "wf-divider" });
  _e365.appendChild(_e435);
  const _e436 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e436);
  const _e437 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Node.js API");
  _e365.appendChild(_e437);
  const _e438 = WF.h("p", { className: "wf-text" }, "Use WebFluent templates in Express, Next.js, or any Node.js application.");
  _e365.appendChild(_e438);
  const _e439 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e439);
  const _e440 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e441 = WF.h("div", { className: "wf-card__header" });
  const _e442 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Install");
  _e441.appendChild(_e442);
  _e440.appendChild(_e441);
  const _e443 = WF.h("div", { className: "wf-card__body" });
  const _e444 = WF.h("code", { className: "wf-code wf-code--block" }, "npm install @aspect/webfluent");
  _e443.appendChild(_e444);
  _e440.appendChild(_e443);
  _e365.appendChild(_e440);
  const _e445 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e445);
  const _e446 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e447 = WF.h("div", { className: "wf-card__header" });
  const _e448 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Basic Usage");
  _e447.appendChild(_e448);
  _e446.appendChild(_e447);
  const _e449 = WF.h("div", { className: "wf-card__body" });
  const _e450 = WF.h("code", { className: "wf-code wf-code--block" }, "const { Template } = require('@aspect/webfluent');\n\nconst tpl = Template.fromFile('templates/invoice.wf');\n// or: Template.fromString('Container { Heading(\"Hello!\", h1) }');\n\n// Render to HTML\nconst html = tpl.renderHtml({ name: \"World\" });\n\n// Render to HTML fragment\nconst frag = tpl.renderHtmlFragment({ name: \"World\" });\n\n// Render to PDF (returns Buffer)\nconst pdf = tpl.renderPdf({ name: \"World\" });\nfs.writeFileSync('output.pdf', pdf);");
  _e449.appendChild(_e450);
  _e446.appendChild(_e449);
  _e365.appendChild(_e446);
  const _e451 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e451);
  const _e452 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e453 = WF.h("div", { className: "wf-card__header" });
  const _e454 = WF.h("p", { className: "wf-text wf-text--bold wf-text--muted" }, "Express.js Example");
  _e453.appendChild(_e454);
  _e452.appendChild(_e453);
  const _e455 = WF.h("div", { className: "wf-card__body" });
  const _e456 = WF.h("code", { className: "wf-code wf-code--block" }, "const express = require('express');\nconst { Template } = require('@aspect/webfluent');\n\nconst app = express();\n\napp.get('/invoice/:id', async (req, res) => {\n    const invoice = await db.getInvoice(req.params.id);\n    const tpl = Template.fromFile('templates/invoice.wf');\n    res.send(tpl.renderHtml(invoice));\n});\n\napp.get('/invoice/:id/pdf', async (req, res) => {\n    const invoice = await db.getInvoice(req.params.id);\n    const tpl = Template.fromFile('templates/invoice.wf');\n    res.type('application/pdf').send(tpl.renderPdf(invoice));\n});");
  _e455.appendChild(_e456);
  _e452.appendChild(_e455);
  _e365.appendChild(_e452);
  const _e457 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e457);
  const _e458 = WF.h("hr", { className: "wf-divider" });
  _e365.appendChild(_e458);
  const _e459 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e459);
  const _e460 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Supported Features");
  _e365.appendChild(_e460);
  const _e461 = WF.h("p", { className: "wf-text" }, "Templates support the static, data-driven subset of WebFluent.");
  _e365.appendChild(_e461);
  const _e462 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e462);
  const _e463 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e464 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e465 = WF.h("div", { className: "wf-card__header" });
  const _e466 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Supported");
  _e465.appendChild(_e466);
  _e464.appendChild(_e465);
  const _e467 = WF.h("div", { className: "wf-card__body" });
  const _e468 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e469 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e470 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e469.appendChild(_e470);
  const _e471 = WF.h("p", { className: "wf-text" }, "All layout components");
  _e469.appendChild(_e471);
  _e468.appendChild(_e469);
  const _e472 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e473 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e472.appendChild(_e473);
  const _e474 = WF.h("p", { className: "wf-text" }, "Typography (Text, Heading, Code)");
  _e472.appendChild(_e474);
  _e468.appendChild(_e472);
  const _e475 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e476 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e475.appendChild(_e476);
  const _e477 = WF.h("p", { className: "wf-text" }, "Data display (Card, Table, List, Badge)");
  _e475.appendChild(_e477);
  _e468.appendChild(_e475);
  const _e478 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e479 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e478.appendChild(_e479);
  const _e480 = WF.h("p", { className: "wf-text" }, "for loops over data arrays");
  _e478.appendChild(_e480);
  _e468.appendChild(_e478);
  const _e481 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e482 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e481.appendChild(_e482);
  const _e483 = WF.h("p", { className: "wf-text" }, "if/else conditionals");
  _e481.appendChild(_e483);
  _e468.appendChild(_e481);
  const _e484 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e485 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e484.appendChild(_e485);
  const _e486 = WF.h("p", { className: "wf-text" }, "String interpolation {var}");
  _e484.appendChild(_e486);
  _e468.appendChild(_e484);
  const _e487 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e488 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e487.appendChild(_e488);
  const _e489 = WF.h("p", { className: "wf-text" }, "Nested access (user.name)");
  _e487.appendChild(_e489);
  _e468.appendChild(_e487);
  const _e490 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e491 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e490.appendChild(_e491);
  const _e492 = WF.h("p", { className: "wf-text" }, "Design tokens and themes");
  _e490.appendChild(_e492);
  _e468.appendChild(_e490);
  const _e493 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e494 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e493.appendChild(_e494);
  const _e495 = WF.h("p", { className: "wf-text" }, "Style blocks");
  _e493.appendChild(_e495);
  _e468.appendChild(_e493);
  const _e496 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e497 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Yes");
  _e496.appendChild(_e497);
  const _e498 = WF.h("p", { className: "wf-text" }, "PDF components");
  _e496.appendChild(_e498);
  _e468.appendChild(_e496);
  _e467.appendChild(_e468);
  _e464.appendChild(_e467);
  _e463.appendChild(_e464);
  const _e499 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e500 = WF.h("div", { className: "wf-card__header" });
  const _e501 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Not Supported");
  _e500.appendChild(_e501);
  _e499.appendChild(_e500);
  const _e502 = WF.h("div", { className: "wf-card__body" });
  const _e503 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e504 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e505 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e504.appendChild(_e505);
  const _e506 = WF.h("p", { className: "wf-text" }, "state / derived / effect");
  _e504.appendChild(_e506);
  _e503.appendChild(_e504);
  const _e507 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e508 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e507.appendChild(_e508);
  const _e509 = WF.h("p", { className: "wf-text" }, "Events (on:click, on:submit)");
  _e507.appendChild(_e509);
  _e503.appendChild(_e507);
  const _e510 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e511 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e510.appendChild(_e511);
  const _e512 = WF.h("p", { className: "wf-text" }, "Navigation / Router");
  _e510.appendChild(_e512);
  _e503.appendChild(_e510);
  const _e513 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e514 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e513.appendChild(_e514);
  const _e515 = WF.h("p", { className: "wf-text" }, "Stores (shared state)");
  _e513.appendChild(_e515);
  _e503.appendChild(_e513);
  const _e516 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e517 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e516.appendChild(_e517);
  const _e518 = WF.h("p", { className: "wf-text" }, "fetch (data loading)");
  _e516.appendChild(_e518);
  _e503.appendChild(_e516);
  const _e519 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e520 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e519.appendChild(_e520);
  const _e521 = WF.h("p", { className: "wf-text" }, "Animations");
  _e519.appendChild(_e521);
  _e503.appendChild(_e519);
  const _e522 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e523 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "No");
  _e522.appendChild(_e523);
  const _e524 = WF.h("p", { className: "wf-text" }, "Toast (imperative)");
  _e522.appendChild(_e524);
  _e503.appendChild(_e522);
  _e502.appendChild(_e503);
  _e499.appendChild(_e502);
  _e463.appendChild(_e499);
  _e365.appendChild(_e463);
  const _e525 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e525);
  const _e526 = WF.h("hr", { className: "wf-divider" });
  _e365.appendChild(_e526);
  const _e527 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e527);
  const _e528 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Use Cases");
  _e365.appendChild(_e528);
  const _e529 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e529);
  const _e530 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e531 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e532 = WF.h("div", { className: "wf-card__body" });
  const _e533 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Server-Rendered Pages");
  _e532.appendChild(_e533);
  const _e534 = WF.h("p", { className: "wf-text wf-text--muted" }, "Generate HTML pages on the server with data from your database or API.");
  _e532.appendChild(_e534);
  _e531.appendChild(_e532);
  _e530.appendChild(_e531);
  const _e535 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e536 = WF.h("div", { className: "wf-card__body" });
  const _e537 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "PDF Reports");
  _e536.appendChild(_e537);
  const _e538 = WF.h("p", { className: "wf-text wf-text--muted" }, "Create invoices, receipts, and reports as PDF files from structured data.");
  _e536.appendChild(_e538);
  _e535.appendChild(_e536);
  _e530.appendChild(_e535);
  const _e539 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e540 = WF.h("div", { className: "wf-card__body" });
  const _e541 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Email Templates");
  _e540.appendChild(_e541);
  const _e542 = WF.h("p", { className: "wf-text wf-text--muted" }, "Render HTML emails with WebFluent components and your design system.");
  _e540.appendChild(_e542);
  _e539.appendChild(_e540);
  _e530.appendChild(_e539);
  _e365.appendChild(_e530);
  const _e543 = WF.h("div", { className: "wf-spacer" });
  _e365.appendChild(_e543);
  _root.appendChild(_e365);
  return _root;
}

function Page_Pdf(params) {
  const _root = document.createDocumentFragment();
  const _e544 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e545 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e545);
  const _e546 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "PDF Generation");
  _e544.appendChild(_e546);
  const _e547 = WF.h("p", { className: "wf-text wf-text--muted" }, "Generate PDF documents directly from .wf source files. No external dependencies — raw PDF 1.7 output.");
  _e544.appendChild(_e547);
  const _e548 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e548);
  const _e549 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Enable PDF Output");
  _e544.appendChild(_e549);
  const _e550 = WF.h("p", { className: "wf-text" }, "Set the output type to pdf in your project config.");
  _e544.appendChild(_e550);
  const _e551 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e551);
  const _e552 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e553 = WF.h("div", { className: "wf-card__body" });
  const _e554 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"build\": {\n    \"output_type\": \"pdf\",\n    \"pdf\": {\n      \"page_size\": \"A4\",\n      \"margins\": { \"top\": 72, \"bottom\": 72, \"left\": 72, \"right\": 72 },\n      \"default_font\": \"Helvetica\",\n      \"default_font_size\": 12,\n      \"output_filename\": \"report.pdf\"\n    }\n  }\n}");
  _e553.appendChild(_e554);
  _e552.appendChild(_e553);
  _e544.appendChild(_e552);
  const _e555 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e555);
  const _e556 = WF.h("hr", { className: "wf-divider" });
  _e544.appendChild(_e556);
  const _e557 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e557);
  const _e558 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Quick Start");
  _e544.appendChild(_e558);
  const _e559 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e560 = WF.h("div", { className: "wf-card__body" });
  const _e561 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-report --template pdf\ncd my-report\nwf build");
  _e560.appendChild(_e561);
  _e559.appendChild(_e560);
  _e544.appendChild(_e559);
  const _e562 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e562);
  const _e563 = WF.h("p", { className: "wf-text wf-text--muted" }, "This creates a sample PDF project and builds it to build/my-report.pdf.");
  _e544.appendChild(_e563);
  const _e564 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e564);
  const _e565 = WF.h("hr", { className: "wf-divider" });
  _e544.appendChild(_e565);
  const _e566 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e566);
  const _e567 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Document Structure");
  _e544.appendChild(_e567);
  const _e568 = WF.h("p", { className: "wf-text" }, "PDF documents use the same .wf syntax. Wrap content in a Document element with optional Header and Footer.");
  _e544.appendChild(_e568);
  const _e569 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e569);
  const _e570 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e571 = WF.h("div", { className: "wf-card__body" });
  const _e572 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Report (path: \"/\", title: \"Q1 Report\") {\n    Document(page_size: \"A4\") {\n        Header {\n            Text(\"Company Inc.\", muted, small, right)\n        }\n\n        Footer {\n            Text(\"Confidential\", muted, small, center)\n        }\n\n        Section {\n            Heading(\"Quarterly Report\", h1)\n            Text(\"Revenue grew 15% this quarter.\")\n\n            Table {\n                Thead {\n                    Trow {\n                        Tcell(\"Region\")\n                        Tcell(\"Revenue\")\n                    }\n                }\n                Tbody {\n                    Trow {\n                        Tcell(\"North America\")\n                        Tcell(\"$2.4M\")\n                    }\n                }\n            }\n\n            PageBreak()\n\n            Heading(\"Key Highlights\", h2)\n            List {\n                Text(\"Launched 3 new products\")\n                Text(\"Expanded to 5 new markets\")\n            }\n        }\n    }\n}");
  _e571.appendChild(_e572);
  _e570.appendChild(_e571);
  _e544.appendChild(_e570);
  const _e573 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e573);
  const _e574 = WF.h("hr", { className: "wf-divider" });
  _e544.appendChild(_e574);
  const _e575 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e575);
  const _e576 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Supported Components");
  _e544.appendChild(_e576);
  const _e577 = WF.h("p", { className: "wf-text" }, "These components render in PDF output:");
  _e544.appendChild(_e577);
  const _e578 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e578);
  const _e579 = WF.h("table", { className: "wf-table" });
  const _e580 = WF.h("thead", {});
  const _e581 = WF.h("td", {}, "Component");
  _e580.appendChild(_e581);
  const _e582 = WF.h("td", {}, "PDF Behavior");
  _e580.appendChild(_e582);
  _e579.appendChild(_e580);
  const _e583 = WF.h("tr", {});
  const _e584 = WF.h("td", {}, "Document");
  _e583.appendChild(_e584);
  const _e585 = WF.h("td", {}, "Root element. Sets page size via page_size arg.");
  _e583.appendChild(_e585);
  _e579.appendChild(_e583);
  const _e586 = WF.h("tr", {});
  const _e587 = WF.h("td", {}, "Header / Footer");
  _e586.appendChild(_e587);
  const _e588 = WF.h("td", {}, "Repeated on every page. Positioned in margins.");
  _e586.appendChild(_e588);
  _e579.appendChild(_e586);
  const _e589 = WF.h("tr", {});
  const _e590 = WF.h("td", {}, "Section");
  _e589.appendChild(_e590);
  const _e591 = WF.h("td", {}, "Groups content with spacing.");
  _e589.appendChild(_e591);
  _e579.appendChild(_e589);
  const _e592 = WF.h("tr", {});
  const _e593 = WF.h("td", {}, "Paragraph");
  _e592.appendChild(_e593);
  const _e594 = WF.h("td", {}, "Block of text with paragraph spacing.");
  _e592.appendChild(_e594);
  _e579.appendChild(_e592);
  const _e595 = WF.h("tr", {});
  const _e596 = WF.h("td", {}, "PageBreak");
  _e595.appendChild(_e596);
  const _e597 = WF.h("td", {}, "Forces a new page.");
  _e595.appendChild(_e597);
  _e579.appendChild(_e595);
  const _e598 = WF.h("tr", {});
  const _e599 = WF.h("td", {}, "Heading(text, h1..h6)");
  _e598.appendChild(_e599);
  const _e600 = WF.h("td", {}, "Bold heading. h1=28pt, h2=22pt, h3=18pt...");
  _e598.appendChild(_e600);
  _e579.appendChild(_e598);
  const _e601 = WF.h("tr", {});
  const _e602 = WF.h("td", {}, "Text(text)");
  _e601.appendChild(_e602);
  const _e603 = WF.h("td", {}, "Body text with word wrapping.");
  _e601.appendChild(_e603);
  _e579.appendChild(_e601);
  const _e604 = WF.h("tr", {});
  const _e605 = WF.h("td", {}, "Table / Thead / Tbody / Trow / Tcell");
  _e604.appendChild(_e605);
  const _e606 = WF.h("td", {}, "Gridded table with borders and header styling.");
  _e604.appendChild(_e606);
  _e579.appendChild(_e604);
  const _e607 = WF.h("tr", {});
  const _e608 = WF.h("td", {}, "List");
  _e607.appendChild(_e608);
  const _e609 = WF.h("td", {}, "Bulleted list. Add ordered modifier for numbered.");
  _e607.appendChild(_e609);
  _e579.appendChild(_e607);
  const _e610 = WF.h("tr", {});
  const _e611 = WF.h("td", {}, "Code(text, block)");
  _e610.appendChild(_e611);
  const _e612 = WF.h("td", {}, "Monospace code with gray background.");
  _e610.appendChild(_e612);
  _e579.appendChild(_e610);
  const _e613 = WF.h("tr", {});
  const _e614 = WF.h("td", {}, "Blockquote");
  _e613.appendChild(_e614);
  const _e615 = WF.h("td", {}, "Indented text with left bar.");
  _e613.appendChild(_e615);
  _e579.appendChild(_e613);
  const _e616 = WF.h("tr", {});
  const _e617 = WF.h("td", {}, "Divider");
  _e616.appendChild(_e617);
  const _e618 = WF.h("td", {}, "Horizontal line.");
  _e616.appendChild(_e618);
  _e579.appendChild(_e616);
  const _e619 = WF.h("tr", {});
  const _e620 = WF.h("td", {}, "Alert(text, variant)");
  _e619.appendChild(_e620);
  const _e621 = WF.h("td", {}, "Colored box with left accent bar.");
  _e619.appendChild(_e621);
  _e579.appendChild(_e619);
  const _e622 = WF.h("tr", {});
  const _e623 = WF.h("td", {}, "Badge / Tag");
  _e622.appendChild(_e623);
  const _e624 = WF.h("td", {}, "Colored pill with white text.");
  _e622.appendChild(_e624);
  _e579.appendChild(_e622);
  const _e625 = WF.h("tr", {});
  const _e626 = WF.h("td", {}, "Progress(value, max)");
  _e625.appendChild(_e626);
  const _e627 = WF.h("td", {}, "Horizontal bar.");
  _e625.appendChild(_e627);
  _e579.appendChild(_e625);
  const _e628 = WF.h("tr", {});
  const _e629 = WF.h("td", {}, "Card");
  _e628.appendChild(_e629);
  const _e630 = WF.h("td", {}, "Bordered box around children.");
  _e628.appendChild(_e630);
  _e579.appendChild(_e628);
  const _e631 = WF.h("tr", {});
  const _e632 = WF.h("td", {}, "Image(src)");
  _e631.appendChild(_e632);
  const _e633 = WF.h("td", {}, "Placeholder rectangle (JPEG planned).");
  _e631.appendChild(_e633);
  _e579.appendChild(_e631);
  const _e634 = WF.h("tr", {});
  const _e635 = WF.h("td", {}, "Spacer");
  _e634.appendChild(_e635);
  const _e636 = WF.h("td", {}, "Vertical space. Modifiers: sm, md, lg, xl.");
  _e634.appendChild(_e636);
  _e579.appendChild(_e634);
  _e544.appendChild(_e579);
  const _e637 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e637);
  const _e638 = WF.h("hr", { className: "wf-divider" });
  _e544.appendChild(_e638);
  const _e639 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e639);
  const _e640 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Rejected Components");
  _e544.appendChild(_e640);
  const _e641 = WF.h("p", { className: "wf-text" }, "Interactive and web-only components cause compile-time errors in PDF mode:");
  _e544.appendChild(_e641);
  const _e642 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e642);
  const _e643 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e644 = WF.h("div", { className: "wf-card__body" });
  const _e645 = WF.h("code", { className: "wf-code wf-code--block" }, "error[pdf]: 'Button' cannot be used in PDF output (Page Report)\n  — interactive elements are not supported in PDF\n\nerror[pdf]: 'Input' cannot be used in PDF output (Page Report)\n  — interactive elements are not supported in PDF");
  _e644.appendChild(_e645);
  _e643.appendChild(_e644);
  _e544.appendChild(_e643);
  const _e646 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e646);
  const _e647 = WF.h("p", { className: "wf-text wf-text--muted" }, "Rejected: Button, Input, Select, Checkbox, Switch, Slider, Form, Modal, Dialog, Toast, Router, Navbar, Sidebar, Tabs, Video, Carousel, and all event handlers.");
  _e544.appendChild(_e647);
  const _e648 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e648);
  const _e649 = WF.h("hr", { className: "wf-divider" });
  _e544.appendChild(_e649);
  const _e650 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e650);
  const _e651 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Page Sizes");
  _e544.appendChild(_e651);
  const _e652 = WF.h("table", { className: "wf-table" });
  const _e653 = WF.h("thead", {});
  const _e654 = WF.h("td", {}, "Value");
  _e653.appendChild(_e654);
  const _e655 = WF.h("td", {}, "Dimensions (points)");
  _e653.appendChild(_e655);
  const _e656 = WF.h("td", {}, "Dimensions (mm)");
  _e653.appendChild(_e656);
  _e652.appendChild(_e653);
  const _e657 = WF.h("tr", {});
  const _e658 = WF.h("td", {}, "A4");
  _e657.appendChild(_e658);
  const _e659 = WF.h("td", {}, "595 x 842");
  _e657.appendChild(_e659);
  const _e660 = WF.h("td", {}, "210 x 297");
  _e657.appendChild(_e660);
  _e652.appendChild(_e657);
  const _e661 = WF.h("tr", {});
  const _e662 = WF.h("td", {}, "A3");
  _e661.appendChild(_e662);
  const _e663 = WF.h("td", {}, "842 x 1191");
  _e661.appendChild(_e663);
  const _e664 = WF.h("td", {}, "297 x 420");
  _e661.appendChild(_e664);
  _e652.appendChild(_e661);
  const _e665 = WF.h("tr", {});
  const _e666 = WF.h("td", {}, "A5");
  _e665.appendChild(_e666);
  const _e667 = WF.h("td", {}, "420 x 595");
  _e665.appendChild(_e667);
  const _e668 = WF.h("td", {}, "148 x 210");
  _e665.appendChild(_e668);
  _e652.appendChild(_e665);
  const _e669 = WF.h("tr", {});
  const _e670 = WF.h("td", {}, "Letter");
  _e669.appendChild(_e670);
  const _e671 = WF.h("td", {}, "612 x 792");
  _e669.appendChild(_e671);
  const _e672 = WF.h("td", {}, "216 x 279");
  _e669.appendChild(_e672);
  _e652.appendChild(_e669);
  const _e673 = WF.h("tr", {});
  const _e674 = WF.h("td", {}, "Legal");
  _e673.appendChild(_e674);
  const _e675 = WF.h("td", {}, "612 x 1008");
  _e673.appendChild(_e675);
  const _e676 = WF.h("td", {}, "216 x 356");
  _e673.appendChild(_e676);
  _e652.appendChild(_e673);
  _e544.appendChild(_e652);
  const _e677 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e677);
  const _e678 = WF.h("hr", { className: "wf-divider" });
  _e544.appendChild(_e678);
  const _e679 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e679);
  const _e680 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Fonts");
  _e544.appendChild(_e680);
  const _e681 = WF.h("p", { className: "wf-text" }, "PDF output uses the 14 standard PDF base fonts. No embedding needed.");
  _e544.appendChild(_e681);
  const _e682 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e682);
  const _e683 = WF.h("table", { className: "wf-table" });
  const _e684 = WF.h("thead", {});
  const _e685 = WF.h("td", {}, "Font Family");
  _e684.appendChild(_e685);
  const _e686 = WF.h("td", {}, "Variants");
  _e684.appendChild(_e686);
  _e683.appendChild(_e684);
  const _e687 = WF.h("tr", {});
  const _e688 = WF.h("td", {}, "Helvetica");
  _e687.appendChild(_e688);
  const _e689 = WF.h("td", {}, "Regular, Bold, Oblique, BoldOblique");
  _e687.appendChild(_e689);
  _e683.appendChild(_e687);
  const _e690 = WF.h("tr", {});
  const _e691 = WF.h("td", {}, "Times");
  _e690.appendChild(_e691);
  const _e692 = WF.h("td", {}, "Roman, Bold, Italic, BoldItalic");
  _e690.appendChild(_e692);
  _e683.appendChild(_e690);
  const _e693 = WF.h("tr", {});
  const _e694 = WF.h("td", {}, "Courier");
  _e693.appendChild(_e694);
  const _e695 = WF.h("td", {}, "Regular, Bold, Oblique, BoldOblique");
  _e693.appendChild(_e695);
  _e683.appendChild(_e693);
  _e544.appendChild(_e683);
  const _e696 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e696);
  const _e697 = WF.h("p", { className: "wf-text wf-text--muted" }, "Set the default font in config or override per-element with style blocks:");
  _e544.appendChild(_e697);
  const _e698 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e698);
  const _e699 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e700 = WF.h("div", { className: "wf-card__body" });
  const _e701 = WF.h("code", { className: "wf-code wf-code--block" }, "Heading(\"Title\", h1) {\n    style {\n        font-family: \"Helvetica-Bold\"\n        color: \"#1a1a2e\"\n    }\n}");
  _e700.appendChild(_e701);
  _e699.appendChild(_e700);
  _e544.appendChild(_e699);
  const _e702 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e702);
  const _e703 = WF.h("hr", { className: "wf-divider" });
  _e544.appendChild(_e703);
  const _e704 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e704);
  const _e705 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Styling in PDF");
  _e544.appendChild(_e705);
  const _e706 = WF.h("p", { className: "wf-text" }, "Style blocks support these properties in PDF output:");
  _e544.appendChild(_e706);
  const _e707 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e707);
  const _e708 = WF.h("table", { className: "wf-table" });
  const _e709 = WF.h("thead", {});
  const _e710 = WF.h("td", {}, "Property");
  _e709.appendChild(_e710);
  const _e711 = WF.h("td", {}, "Values");
  _e709.appendChild(_e711);
  const _e712 = WF.h("td", {}, "Example");
  _e709.appendChild(_e712);
  _e708.appendChild(_e709);
  const _e713 = WF.h("tr", {});
  const _e714 = WF.h("td", {}, "font-size");
  _e713.appendChild(_e714);
  const _e715 = WF.h("td", {}, "Number (points)");
  _e713.appendChild(_e715);
  const _e716 = WF.h("td", {}, "font-size: 14");
  _e713.appendChild(_e716);
  _e708.appendChild(_e713);
  const _e717 = WF.h("tr", {});
  const _e718 = WF.h("td", {}, "font-family");
  _e717.appendChild(_e718);
  const _e719 = WF.h("td", {}, "Base14 font name");
  _e717.appendChild(_e719);
  const _e720 = WF.h("td", {}, "font-family: \"Courier\"");
  _e717.appendChild(_e720);
  _e708.appendChild(_e717);
  const _e721 = WF.h("tr", {});
  const _e722 = WF.h("td", {}, "color");
  _e721.appendChild(_e722);
  const _e723 = WF.h("td", {}, "Hex color");
  _e721.appendChild(_e723);
  const _e724 = WF.h("td", {}, "color: \"#333333\"");
  _e721.appendChild(_e724);
  _e708.appendChild(_e721);
  const _e725 = WF.h("tr", {});
  const _e726 = WF.h("td", {}, "text-align");
  _e725.appendChild(_e726);
  const _e727 = WF.h("td", {}, "left, center, right");
  _e725.appendChild(_e727);
  const _e728 = WF.h("td", {}, "text-align: \"center\"");
  _e725.appendChild(_e728);
  _e708.appendChild(_e725);
  _e544.appendChild(_e708);
  const _e729 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e729);
  const _e730 = WF.h("p", { className: "wf-text wf-text--muted" }, "Modifiers also work: bold, muted, primary, danger, success, warning, info, small, large, center, right.");
  _e544.appendChild(_e730);
  const _e731 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e731);
  const _e732 = WF.h("hr", { className: "wf-divider" });
  _e544.appendChild(_e732);
  const _e733 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e733);
  const _e734 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Auto Page Breaks");
  _e544.appendChild(_e734);
  const _e735 = WF.h("p", { className: "wf-text wf-text--muted" }, "Content automatically flows to a new page when it reaches the bottom margin. Headers and footers are rendered on every page, including auto-generated ones.");
  _e544.appendChild(_e735);
  const _e736 = WF.h("div", { className: "wf-spacer" });
  _e544.appendChild(_e736);
  _root.appendChild(_e544);
  return _root;
}

function Page_Guide(params) {
  const _root = document.createDocumentFragment();
  const _e737 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e738 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e738);
  const _e739 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Language Guide");
  _e737.appendChild(_e739);
  const _e740 = WF.h("p", { className: "wf-text wf-text--muted" }, "Learn the core concepts of WebFluent.");
  _e737.appendChild(_e740);
  const _e741 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e741);
  const _e742 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Pages");
  _e737.appendChild(_e742);
  const _e743 = WF.h("p", { className: "wf-text" }, "Pages are top-level route targets. Each page defines a URL path and contains the UI tree for that route.");
  _e737.appendChild(_e743);
  const _e744 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e744);
  const _e745 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e746 = WF.h("div", { className: "wf-card__body" });
  const _e747 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\", title: \"Home\") {\n    Container {\n        Heading(\"Welcome\", h1)\n        Text(\"This is the home page.\")\n    }\n}");
  _e746.appendChild(_e747);
  _e745.appendChild(_e746);
  _e737.appendChild(_e745);
  const _e748 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e748);
  const _e749 = WF.h("p", { className: "wf-text wf-text--bold" }, "Page attributes:");
  _e737.appendChild(_e749);
  const _e750 = WF.h("table", { className: "wf-table" });
  const _e751 = WF.h("thead", {});
  const _e752 = WF.h("td", {}, "Attribute");
  _e751.appendChild(_e752);
  const _e753 = WF.h("td", {}, "Type");
  _e751.appendChild(_e753);
  const _e754 = WF.h("td", {}, "Description");
  _e751.appendChild(_e754);
  _e750.appendChild(_e751);
  const _e755 = WF.h("tr", {});
  const _e756 = WF.h("td", {}, "path");
  _e755.appendChild(_e756);
  const _e757 = WF.h("td", {}, "String");
  _e755.appendChild(_e757);
  const _e758 = WF.h("td", {}, "URL route for this page (required)");
  _e755.appendChild(_e758);
  _e750.appendChild(_e755);
  const _e759 = WF.h("tr", {});
  const _e760 = WF.h("td", {}, "title");
  _e759.appendChild(_e760);
  const _e761 = WF.h("td", {}, "String");
  _e759.appendChild(_e761);
  const _e762 = WF.h("td", {}, "Document title");
  _e759.appendChild(_e762);
  _e750.appendChild(_e759);
  const _e763 = WF.h("tr", {});
  const _e764 = WF.h("td", {}, "guard");
  _e763.appendChild(_e764);
  const _e765 = WF.h("td", {}, "Expression");
  _e763.appendChild(_e765);
  const _e766 = WF.h("td", {}, "Navigation guard — redirects if false");
  _e763.appendChild(_e766);
  _e750.appendChild(_e763);
  const _e767 = WF.h("tr", {});
  const _e768 = WF.h("td", {}, "redirect");
  _e767.appendChild(_e768);
  const _e769 = WF.h("td", {}, "String");
  _e767.appendChild(_e769);
  const _e770 = WF.h("td", {}, "Redirect target when guard fails");
  _e767.appendChild(_e770);
  _e750.appendChild(_e767);
  _e737.appendChild(_e750);
  const _e771 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e771);
  const _e772 = WF.h("hr", { className: "wf-divider" });
  _e737.appendChild(_e772);
  const _e773 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e773);
  const _e774 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Components");
  _e737.appendChild(_e774);
  const _e775 = WF.h("p", { className: "wf-text" }, "Reusable UI blocks that accept props and can have internal state.");
  _e737.appendChild(_e775);
  const _e776 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e776);
  const _e777 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e778 = WF.h("div", { className: "wf-card__body" });
  const _e779 = WF.h("code", { className: "wf-code wf-code--block" }, "Component UserCard (name: String, role: String, active: Bool = true) {\n    Card(elevated) {\n        Row(align: center, gap: md) {\n            Avatar(initials: \"U\", primary)\n            Stack {\n                Text(name, bold)\n                Text(role, muted)\n            }\n            if active {\n                Badge(\"Active\", success)\n            }\n        }\n    }\n}\n\n// Usage\nUserCard(name: \"Monzer\", role: \"Developer\")");
  _e778.appendChild(_e779);
  _e777.appendChild(_e778);
  _e737.appendChild(_e777);
  const _e780 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e780);
  const _e781 = WF.h("p", { className: "wf-text wf-text--muted" }, "Props support types: String, Number, Bool, List, Map. Optional props use ?, defaults use =.");
  _e737.appendChild(_e781);
  const _e782 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e782);
  const _e783 = WF.h("hr", { className: "wf-divider" });
  _e737.appendChild(_e783);
  const _e784 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e784);
  const _e785 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "State and Reactivity");
  _e737.appendChild(_e785);
  const _e786 = WF.h("p", { className: "wf-text" }, "State is declared with the state keyword. It is reactive — any UI that reads it updates automatically when it changes.");
  _e737.appendChild(_e786);
  const _e787 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e787);
  const _e788 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e789 = WF.h("div", { className: "wf-card__body" });
  const _e790 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Counter (path: \"/counter\") {\n    state count = 0\n\n    Container {\n        Text(\"Count: {count}\")\n        Button(\"+1\", primary) { count = count + 1 }\n        Button(\"-1\") { count = count - 1 }\n    }\n}");
  _e789.appendChild(_e790);
  _e788.appendChild(_e789);
  _e737.appendChild(_e788);
  const _e791 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e791);
  const _e792 = WF.h("p", { className: "wf-text wf-text--bold" }, "Derived state:");
  _e737.appendChild(_e792);
  const _e793 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e794 = WF.h("div", { className: "wf-card__body" });
  const _e795 = WF.h("code", { className: "wf-code wf-code--block" }, "state items = [{name: \"A\", price: 3}, {name: \"B\", price: 2}]\nderived total = items.map(i => i.price).sum()\nderived isEmpty = items.length == 0");
  _e794.appendChild(_e795);
  _e793.appendChild(_e794);
  _e737.appendChild(_e793);
  const _e796 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e796);
  const _e797 = WF.h("hr", { className: "wf-divider" });
  _e737.appendChild(_e797);
  const _e798 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e798);
  const _e799 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Events");
  _e737.appendChild(_e799);
  const _e800 = WF.h("p", { className: "wf-text" }, "Event handlers are declared with on:event or via shorthand blocks on buttons.");
  _e737.appendChild(_e800);
  const _e801 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e801);
  const _e802 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e803 = WF.h("div", { className: "wf-card__body" });
  const _e804 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Submit\") {\n    on:click {\n        submitForm()\n    }\n}\n\nInput(text, placeholder: \"Search...\") {\n    on:input {\n        searchQuery = value\n    }\n    on:keydown {\n        if key == \"Enter\" {\n            performSearch()\n        }\n    }\n}\n\n// Shorthand: block on Button defaults to on:click\nButton(\"Save\") { save() }");
  _e803.appendChild(_e804);
  _e802.appendChild(_e803);
  _e737.appendChild(_e802);
  const _e805 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e805);
  const _e806 = WF.h("p", { className: "wf-text wf-text--muted" }, "Supported events: on:click, on:submit, on:input, on:change, on:focus, on:blur, on:keydown, on:keyup, on:mouseover, on:mouseout, on:mount, on:unmount");
  _e737.appendChild(_e806);
  const _e807 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e807);
  const _e808 = WF.h("hr", { className: "wf-divider" });
  _e737.appendChild(_e808);
  const _e809 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e809);
  const _e810 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Control Flow");
  _e737.appendChild(_e810);
  const _e811 = WF.h("p", { className: "wf-text wf-text--bold" }, "Conditionals:");
  _e737.appendChild(_e811);
  const _e812 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e813 = WF.h("div", { className: "wf-card__body" });
  const _e814 = WF.h("code", { className: "wf-code wf-code--block" }, "if isLoggedIn {\n    Text(\"Welcome back!\")\n} else if isGuest {\n    Text(\"Hello, guest\")\n} else {\n    Button(\"Log In\") { navigate(\"/login\") }\n}");
  _e813.appendChild(_e814);
  _e812.appendChild(_e813);
  _e737.appendChild(_e812);
  const _e815 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e815);
  const _e816 = WF.h("p", { className: "wf-text wf-text--bold" }, "Loops:");
  _e737.appendChild(_e816);
  const _e817 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e818 = WF.h("div", { className: "wf-card__body" });
  const _e819 = WF.h("code", { className: "wf-code wf-code--block" }, "for user in users {\n    UserCard(name: user.name, role: user.role)\n}\n\n// With index\nfor item, index in items {\n    Text(\"{index + 1}. {item}\")\n}");
  _e818.appendChild(_e819);
  _e817.appendChild(_e818);
  _e737.appendChild(_e817);
  const _e820 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e820);
  const _e821 = WF.h("p", { className: "wf-text wf-text--bold" }, "Show/Hide (keeps element in DOM, toggles visibility):");
  _e737.appendChild(_e821);
  const _e822 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e823 = WF.h("div", { className: "wf-card__body" });
  const _e824 = WF.h("code", { className: "wf-code wf-code--block" }, "show isExpanded {\n    Card { Text(\"Expanded content\") }\n}");
  _e823.appendChild(_e824);
  _e822.appendChild(_e823);
  _e737.appendChild(_e822);
  const _e825 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e825);
  const _e826 = WF.h("hr", { className: "wf-divider" });
  _e737.appendChild(_e826);
  const _e827 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e827);
  const _e828 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Stores");
  _e737.appendChild(_e828);
  const _e829 = WF.h("p", { className: "wf-text" }, "Stores hold shared state accessible from any page or component.");
  _e737.appendChild(_e829);
  const _e830 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e830);
  const _e831 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e832 = WF.h("div", { className: "wf-card__body" });
  const _e833 = WF.h("code", { className: "wf-code wf-code--block" }, "Store CartStore {\n    state items = []\n\n    derived total = items.map(i => i.price * i.quantity).sum()\n    derived count = items.length\n\n    action addItem(product: Map) {\n        items.push({ id: product.id, name: product.name, price: product.price, quantity: 1 })\n    }\n\n    action removeItem(id: Number) {\n        items = items.filter(i => i.id != id)\n    }\n}\n\n// Usage in a page\nPage Cart (path: \"/cart\") {\n    use CartStore\n\n    Text(\"Total: ${CartStore.total}\")\n    Button(\"Clear\") { CartStore.clear() }\n}");
  _e832.appendChild(_e833);
  _e831.appendChild(_e832);
  _e737.appendChild(_e831);
  const _e834 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e834);
  const _e835 = WF.h("hr", { className: "wf-divider" });
  _e737.appendChild(_e835);
  const _e836 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e836);
  const _e837 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Routing");
  _e737.appendChild(_e837);
  const _e838 = WF.h("p", { className: "wf-text" }, "SPA routing is declared in the App file.");
  _e737.appendChild(_e838);
  const _e839 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e839);
  const _e840 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e841 = WF.h("div", { className: "wf-card__body" });
  const _e842 = WF.h("code", { className: "wf-code wf-code--block" }, "App {\n    Navbar {\n        Navbar.Brand { Text(\"My App\", heading) }\n        Navbar.Links {\n            Link(to: \"/\") { Text(\"Home\") }\n            Link(to: \"/about\") { Text(\"About\") }\n        }\n    }\n\n    Router {\n        Route(path: \"/\", page: Home)\n        Route(path: \"/about\", page: About)\n        Route(path: \"/user/:id\", page: UserProfile)\n        Route(path: \"*\", page: NotFound)\n    }\n}\n\n// Programmatic navigation\nButton(\"Go Home\") { navigate(\"/\") }\n\n// Dynamic routes access params\nPage UserProfile (path: \"/user/:id\") {\n    Text(\"User ID: {params.id}\")\n}");
  _e841.appendChild(_e842);
  _e840.appendChild(_e841);
  _e737.appendChild(_e840);
  const _e843 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e843);
  const _e844 = WF.h("hr", { className: "wf-divider" });
  _e737.appendChild(_e844);
  const _e845 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e845);
  const _e846 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Fetching");
  _e737.appendChild(_e846);
  const _e847 = WF.h("p", { className: "wf-text" }, "Built-in async data loading with automatic loading, error, and success states.");
  _e737.appendChild(_e847);
  const _e848 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e848);
  const _e849 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e850 = WF.h("div", { className: "wf-card__body" });
  const _e851 = WF.h("code", { className: "wf-code wf-code--block" }, "fetch users from \"/api/users\" {\n    loading {\n        Spinner()\n    }\n    error (err) {\n        Alert(\"Failed to load users\", danger)\n    }\n    success {\n        for user in users {\n            UserCard(name: user.name, role: user.role)\n        }\n    }\n}\n\n// With options\nfetch result from \"/api/submit\" (method: \"POST\", body: { name: name, email: email }) {\n    success {\n        Alert(\"Saved!\", success)\n    }\n}");
  _e850.appendChild(_e851);
  _e849.appendChild(_e850);
  _e737.appendChild(_e849);
  const _e852 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e852);
  const _e853 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e854 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/components"); } }, "Components Reference");
  _e853.appendChild(_e854);
  const _e855 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e853.appendChild(_e855);
  _e737.appendChild(_e853);
  const _e856 = WF.h("div", { className: "wf-spacer" });
  _e737.appendChild(_e856);
  _root.appendChild(_e737);
  return _root;
}

function Page_Animation(params) {
  const _showCard = WF.signal(false);
  const _root = document.createDocumentFragment();
  const _e857 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e858 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e858);
  const _e859 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Animation System");
  _e857.appendChild(_e859);
  const _e860 = WF.h("p", { className: "wf-text wf-text--muted" }, "Declarative animations built into the language. No CSS keyframes to write.");
  _e857.appendChild(_e860);
  const _e861 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e861);
  const _e862 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Mount Animations");
  _e857.appendChild(_e862);
  const _e863 = WF.h("p", { className: "wf-text" }, "Add an animation modifier to any component. It plays when the element appears. Hover each card to replay.");
  _e857.appendChild(_e863);
  const _e864 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e864);
  const _e865 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e866 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn"); } });
  const _e867 = WF.h("div", { className: "wf-card__body" });
  const _e868 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "fadeIn");
  _e867.appendChild(_e868);
  const _e869 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Fades from transparent");
  _e867.appendChild(_e869);
  _e866.appendChild(_e867);
  _e865.appendChild(_e866);
  const _e870 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideUp", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideUp"); } });
  const _e871 = WF.h("div", { className: "wf-card__body" });
  const _e872 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideUp");
  _e871.appendChild(_e872);
  const _e873 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from below");
  _e871.appendChild(_e873);
  _e870.appendChild(_e871);
  _e865.appendChild(_e870);
  const _e874 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-scaleIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "scaleIn"); } });
  const _e875 = WF.h("div", { className: "wf-card__body" });
  const _e876 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "scaleIn");
  _e875.appendChild(_e876);
  const _e877 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Scales from 90%");
  _e875.appendChild(_e877);
  _e874.appendChild(_e875);
  _e865.appendChild(_e874);
  const _e878 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideDown", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideDown"); } });
  const _e879 = WF.h("div", { className: "wf-card__body" });
  const _e880 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideDown");
  _e879.appendChild(_e880);
  const _e881 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from above");
  _e879.appendChild(_e881);
  _e878.appendChild(_e879);
  _e865.appendChild(_e878);
  const _e882 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideLeft", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideLeft"); } });
  const _e883 = WF.h("div", { className: "wf-card__body" });
  const _e884 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideLeft");
  _e883.appendChild(_e884);
  const _e885 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from right");
  _e883.appendChild(_e885);
  _e882.appendChild(_e883);
  _e865.appendChild(_e882);
  const _e886 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-bounce", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "bounce"); } });
  const _e887 = WF.h("div", { className: "wf-card__body" });
  const _e888 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "bounce");
  _e887.appendChild(_e888);
  const _e889 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Bouncy entrance");
  _e887.appendChild(_e889);
  _e886.appendChild(_e887);
  _e865.appendChild(_e886);
  const _e890 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-shake", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "shake"); } });
  const _e891 = WF.h("div", { className: "wf-card__body" });
  const _e892 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "shake");
  _e891.appendChild(_e892);
  const _e893 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Horizontal shake");
  _e891.appendChild(_e893);
  _e890.appendChild(_e891);
  _e865.appendChild(_e890);
  const _e894 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-pulse", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "pulse"); } });
  const _e895 = WF.h("div", { className: "wf-card__body" });
  const _e896 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "pulse");
  _e895.appendChild(_e896);
  const _e897 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Gentle scale pulse");
  _e895.appendChild(_e897);
  _e894.appendChild(_e895);
  _e865.appendChild(_e894);
  const _e898 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-slideRight", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "slideRight"); } });
  const _e899 = WF.h("div", { className: "wf-card__body" });
  const _e900 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slideRight");
  _e899.appendChild(_e900);
  const _e901 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted wf-text--small" }, "Slides from left");
  _e899.appendChild(_e901);
  _e898.appendChild(_e899);
  _e865.appendChild(_e898);
  _e857.appendChild(_e865);
  const _e902 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e902);
  const _e903 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e904 = WF.h("div", { className: "wf-card__body" });
  const _e905 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn) { ... }\nHeading(\"Title\", h1, slideUp)\nButton(\"Click\", primary, bounce)");
  _e904.appendChild(_e905);
  _e903.appendChild(_e904);
  _e857.appendChild(_e903);
  const _e906 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e906);
  const _e907 = WF.h("hr", { className: "wf-divider" });
  _e857.appendChild(_e907);
  const _e908 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e908);
  const _e909 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Live: Conditional Animation");
  _e857.appendChild(_e909);
  const _e910 = WF.h("p", { className: "wf-text" }, "Toggle the switch to see enter/exit animations on the card below.");
  _e857.appendChild(_e910);
  const _e911 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e911);
  const _e912 = WF.h("label", { className: "wf-switch" });
  const _e913 = WF.h("input", { type: "checkbox", checked: () => _showCard(), "on:change": () => _showCard.set(!_showCard()) });
  _e912.appendChild(_e913);
  const _e914 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e912.appendChild(_e914);
  _e912.appendChild(WF.text("Show animated card"));
  _e857.appendChild(_e912);
  const _e915 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e915);
  WF.condRender(_e857,
    () => _showCard(),
    () => {
      const _e916 = document.createDocumentFragment();
      const _e917 = WF.h("div", { className: "wf-card wf-card--elevated" });
      const _e918 = WF.h("div", { className: "wf-card__body" });
      const _e919 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Animated!");
      _e918.appendChild(_e919);
      const _e920 = WF.h("div", { className: "wf-spacer" });
      _e918.appendChild(_e920);
      const _e921 = WF.h("p", { className: "wf-text" }, "This card scales in and fades out.");
      _e918.appendChild(_e921);
      const _e922 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, "Controlled by: if showCard, animate(scaleIn, fadeOut)");
      _e918.appendChild(_e922);
      _e917.appendChild(_e918);
      _e916.appendChild(_e917);
      return _e916;
    },
    null,
    { enter: "scaleIn", exit: "fadeOut" }
  );
  const _e923 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e923);
  const _e924 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e925 = WF.h("div", { className: "wf-card__body" });
  const _e926 = WF.h("code", { className: "wf-code wf-code--block" }, "if showCard, animate(scaleIn, fadeOut) {\n    Card(elevated) {\n        Text(\"Animated content\")\n    }\n}");
  _e925.appendChild(_e926);
  _e924.appendChild(_e925);
  _e857.appendChild(_e924);
  const _e927 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e927);
  const _e928 = WF.h("hr", { className: "wf-divider" });
  _e857.appendChild(_e928);
  const _e929 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e929);
  const _e930 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Speed Variants");
  _e857.appendChild(_e930);
  const _e931 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e931);
  const _e932 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e933 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn", "150ms"); } });
  const _e934 = WF.h("div", { className: "wf-card__body" });
  const _e935 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "fast");
  _e934.appendChild(_e935);
  const _e936 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "150ms");
  _e934.appendChild(_e936);
  const _e937 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn, fast)");
  _e934.appendChild(_e937);
  _e933.appendChild(_e934);
  _e932.appendChild(_e933);
  const _e938 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn"); } });
  const _e939 = WF.h("div", { className: "wf-card__body" });
  const _e940 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "default");
  _e939.appendChild(_e940);
  const _e941 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "300ms");
  _e939.appendChild(_e941);
  const _e942 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn)");
  _e939.appendChild(_e942);
  _e938.appendChild(_e939);
  _e932.appendChild(_e938);
  const _e943 = WF.h("div", { className: "wf-card wf-card--outlined wf-animate-fadeIn", "on:mouseenter": (event) => { WF.replayAnimation(event.currentTarget, "fadeIn", "500ms"); } });
  const _e944 = WF.h("div", { className: "wf-card__body" });
  const _e945 = WF.h("p", { className: "wf-text wf-text--center wf-text--bold" }, "slow");
  _e944.appendChild(_e945);
  const _e946 = WF.h("p", { className: "wf-text wf-text--center wf-text--muted" }, "500ms");
  _e944.appendChild(_e946);
  const _e947 = WF.h("code", { className: "wf-code wf-code--block" }, "Card(elevated, fadeIn, slow)");
  _e944.appendChild(_e947);
  _e943.appendChild(_e944);
  _e932.appendChild(_e943);
  _e857.appendChild(_e932);
  const _e948 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e948);
  const _e949 = WF.h("hr", { className: "wf-divider" });
  _e857.appendChild(_e949);
  const _e950 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e950);
  const _e951 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "All 12 Animations");
  _e857.appendChild(_e951);
  const _e952 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e952);
  const _e953 = WF.h("table", { className: "wf-table" });
  const _e954 = WF.h("thead", {});
  const _e955 = WF.h("td", {}, "Name");
  _e954.appendChild(_e955);
  const _e956 = WF.h("td", {}, "Effect");
  _e954.appendChild(_e956);
  const _e957 = WF.h("td", {}, "Usage");
  _e954.appendChild(_e957);
  _e953.appendChild(_e954);
  const _e958 = WF.h("tr", {});
  const _e959 = WF.h("td", {}, "fadeIn / fadeOut");
  _e958.appendChild(_e959);
  const _e960 = WF.h("td", {}, "Opacity fade");
  _e958.appendChild(_e960);
  const _e961 = WF.h("td", {}, "Card(elevated, fadeIn)");
  _e958.appendChild(_e961);
  _e953.appendChild(_e958);
  const _e962 = WF.h("tr", {});
  const _e963 = WF.h("td", {}, "slideUp / slideDown");
  _e962.appendChild(_e963);
  const _e964 = WF.h("td", {}, "Vertical slide + fade");
  _e962.appendChild(_e964);
  const _e965 = WF.h("td", {}, "Heading(\"Hi\", h1, slideUp)");
  _e962.appendChild(_e965);
  _e953.appendChild(_e962);
  const _e966 = WF.h("tr", {});
  const _e967 = WF.h("td", {}, "slideLeft / slideRight");
  _e966.appendChild(_e967);
  const _e968 = WF.h("td", {}, "Horizontal slide + fade");
  _e966.appendChild(_e968);
  const _e969 = WF.h("td", {}, "Text(\"Hello\", slideLeft)");
  _e966.appendChild(_e969);
  _e953.appendChild(_e966);
  const _e970 = WF.h("tr", {});
  const _e971 = WF.h("td", {}, "scaleIn / scaleOut");
  _e970.appendChild(_e971);
  const _e972 = WF.h("td", {}, "Scale from/to 90%");
  _e970.appendChild(_e972);
  const _e973 = WF.h("td", {}, "Badge(\"New\", scaleIn)");
  _e970.appendChild(_e973);
  _e953.appendChild(_e970);
  const _e974 = WF.h("tr", {});
  const _e975 = WF.h("td", {}, "bounce");
  _e974.appendChild(_e975);
  const _e976 = WF.h("td", {}, "Bouncy entrance");
  _e974.appendChild(_e976);
  const _e977 = WF.h("td", {}, "Button(\"Go\", bounce)");
  _e974.appendChild(_e977);
  _e953.appendChild(_e974);
  const _e978 = WF.h("tr", {});
  const _e979 = WF.h("td", {}, "shake");
  _e978.appendChild(_e979);
  const _e980 = WF.h("td", {}, "Horizontal shake");
  _e978.appendChild(_e980);
  const _e981 = WF.h("td", {}, "Alert(\"Error!\", shake)");
  _e978.appendChild(_e981);
  _e953.appendChild(_e978);
  const _e982 = WF.h("tr", {});
  const _e983 = WF.h("td", {}, "pulse");
  _e982.appendChild(_e983);
  const _e984 = WF.h("td", {}, "Scale pulse (infinite)");
  _e982.appendChild(_e984);
  const _e985 = WF.h("td", {}, "Badge(\"Live\", pulse)");
  _e982.appendChild(_e985);
  _e953.appendChild(_e982);
  const _e986 = WF.h("tr", {});
  const _e987 = WF.h("td", {}, "spin");
  _e986.appendChild(_e987);
  const _e988 = WF.h("td", {}, "360-degree rotation");
  _e986.appendChild(_e988);
  const _e989 = WF.h("td", {}, "Spinner(spin)");
  _e986.appendChild(_e989);
  _e953.appendChild(_e986);
  _e857.appendChild(_e953);
  const _e990 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e990);
  const _e991 = WF.h("hr", { className: "wf-divider" });
  _e857.appendChild(_e991);
  const _e992 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e992);
  const _e993 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Conditional Animations");
  _e857.appendChild(_e993);
  const _e994 = WF.h("p", { className: "wf-text" }, "Attach enter and exit animations to if blocks.");
  _e857.appendChild(_e994);
  const _e995 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e995);
  const _e996 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e997 = WF.h("div", { className: "wf-card__body" });
  const _e998 = WF.h("code", { className: "wf-code wf-code--block" }, "if visible, animate(slideUp, fadeOut) {\n    Card { Text(\"Appears with slideUp, exits with fadeOut\") }\n}\n\nif expanded, animate(scaleIn, scaleOut) {\n    Text(\"Scales in and out\")\n}");
  _e997.appendChild(_e998);
  _e996.appendChild(_e997);
  _e857.appendChild(_e996);
  const _e999 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e999);
  const _e1000 = WF.h("hr", { className: "wf-divider" });
  _e857.appendChild(_e1000);
  const _e1001 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e1001);
  const _e1002 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "List Stagger");
  _e857.appendChild(_e1002);
  const _e1003 = WF.h("p", { className: "wf-text" }, "Animate list items with staggered delays.");
  _e857.appendChild(_e1003);
  const _e1004 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e1004);
  const _e1005 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1006 = WF.h("div", { className: "wf-card__body" });
  const _e1007 = WF.h("code", { className: "wf-code wf-code--block" }, "for item in items, animate(slideUp, fadeOut, stagger: \"50ms\") {\n    Card { Text(item.name) }\n}");
  _e1006.appendChild(_e1007);
  _e1005.appendChild(_e1006);
  _e857.appendChild(_e1005);
  const _e1008 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e1008);
  const _e1009 = WF.h("hr", { className: "wf-divider" });
  _e857.appendChild(_e1009);
  const _e1010 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e1010);
  const _e1011 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Transition Blocks");
  _e857.appendChild(_e1011);
  const _e1012 = WF.h("p", { className: "wf-text" }, "Smooth CSS transitions on property changes.");
  _e857.appendChild(_e1012);
  const _e1013 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e1013);
  const _e1014 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1015 = WF.h("div", { className: "wf-card__body" });
  const _e1016 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Hover me\") {\n    transition {\n        background 200ms ease\n        transform 150ms spring\n    }\n}");
  _e1015.appendChild(_e1016);
  _e1014.appendChild(_e1015);
  _e857.appendChild(_e1014);
  const _e1017 = WF.h("div", { className: "wf-spacer" });
  _e857.appendChild(_e1017);
  _root.appendChild(_e857);
  return _root;
}

function Page_I18n(params) {
  const _root = document.createDocumentFragment();
  const _e1018 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1019 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1019);
  const _e1020 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Internationalization (i18n)");
  _e1018.appendChild(_e1020);
  const _e1021 = WF.h("p", { className: "wf-text wf-text--muted" }, "Built-in multi-language support with reactive locale switching and automatic RTL.");
  _e1018.appendChild(_e1021);
  const _e1022 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1022);
  const _e1023 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Setup");
  _e1018.appendChild(_e1023);
  const _e1024 = WF.h("p", { className: "wf-text" }, "Create a JSON file per locale in your translations directory.");
  _e1018.appendChild(_e1024);
  const _e1025 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1025);
  const _e1026 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1027 = WF.h("div", { className: "wf-card__body" });
  const _e1028 = WF.h("code", { className: "wf-code wf-code--block" }, "// src/translations/en.json\n{\n    \"greeting\": \"Hello, {name}!\",\n    \"nav.home\": \"Home\",\n    \"nav.about\": \"About\"\n}\n\n// src/translations/ar.json\n{\n    \"greeting\": \"!أهلاً، {name}\",\n    \"nav.home\": \"الرئيسية\",\n    \"nav.about\": \"حول\"\n}");
  _e1027.appendChild(_e1028);
  _e1026.appendChild(_e1027);
  _e1018.appendChild(_e1026);
  const _e1029 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1029);
  const _e1030 = WF.h("p", { className: "wf-text wf-text--bold" }, "Add i18n config to webfluent.app.json:");
  _e1018.appendChild(_e1030);
  const _e1031 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1032 = WF.h("div", { className: "wf-card__body" });
  const _e1033 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e1032.appendChild(_e1033);
  _e1031.appendChild(_e1032);
  _e1018.appendChild(_e1031);
  const _e1034 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1034);
  const _e1035 = WF.h("hr", { className: "wf-divider" });
  _e1018.appendChild(_e1035);
  const _e1036 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1036);
  const _e1037 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "The t() Function");
  _e1018.appendChild(_e1037);
  const _e1038 = WF.h("p", { className: "wf-text" }, "Use t() to look up translated text. It is reactive — all t() calls update when the locale changes.");
  _e1018.appendChild(_e1038);
  const _e1039 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1039);
  const _e1040 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1041 = WF.h("div", { className: "wf-card__body" });
  const _e1042 = WF.h("code", { className: "wf-code wf-code--block" }, "// Simple key lookup\nText(t(\"nav.home\"))\n\n// With interpolation\nText(t(\"greeting\", name: user.name))\n\n// In any component\nButton(t(\"actions.save\"), primary)\nHeading(t(\"page.title\"), h1)");
  _e1041.appendChild(_e1042);
  _e1040.appendChild(_e1041);
  _e1018.appendChild(_e1040);
  const _e1043 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1043);
  const _e1044 = WF.h("hr", { className: "wf-divider" });
  _e1018.appendChild(_e1044);
  const _e1045 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1045);
  const _e1046 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Locale Switching");
  _e1018.appendChild(_e1046);
  const _e1047 = WF.h("p", { className: "wf-text" }, "Switch the locale at runtime with setLocale(). All translated text updates instantly.");
  _e1018.appendChild(_e1047);
  const _e1048 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1048);
  const _e1049 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1050 = WF.h("div", { className: "wf-card__body" });
  const _e1051 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"English\") { setLocale(\"en\") }\nButton(\"العربية\") { setLocale(\"ar\") }\nButton(\"Espanol\") { setLocale(\"es\") }\n\n// Access current locale\nText(\"Current: {locale}\")\nText(\"Direction: {dir}\")");
  _e1050.appendChild(_e1051);
  _e1049.appendChild(_e1050);
  _e1018.appendChild(_e1049);
  const _e1052 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1052);
  const _e1053 = WF.h("hr", { className: "wf-divider" });
  _e1018.appendChild(_e1053);
  const _e1054 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1054);
  const _e1055 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "RTL Support");
  _e1018.appendChild(_e1055);
  const _e1056 = WF.h("p", { className: "wf-text" }, "WebFluent automatically detects RTL locales and updates the document direction.");
  _e1018.appendChild(_e1056);
  const _e1057 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1057);
  const _e1058 = WF.h("table", { className: "wf-table" });
  const _e1059 = WF.h("thead", {});
  const _e1060 = WF.h("td", {}, "Locale");
  _e1059.appendChild(_e1060);
  const _e1061 = WF.h("td", {}, "Direction");
  _e1059.appendChild(_e1061);
  _e1058.appendChild(_e1059);
  const _e1062 = WF.h("tr", {});
  const _e1063 = WF.h("td", {}, "ar (Arabic)");
  _e1062.appendChild(_e1063);
  const _e1064 = WF.h("td", {}, "RTL");
  _e1062.appendChild(_e1064);
  _e1058.appendChild(_e1062);
  const _e1065 = WF.h("tr", {});
  const _e1066 = WF.h("td", {}, "he (Hebrew)");
  _e1065.appendChild(_e1066);
  const _e1067 = WF.h("td", {}, "RTL");
  _e1065.appendChild(_e1067);
  _e1058.appendChild(_e1065);
  const _e1068 = WF.h("tr", {});
  const _e1069 = WF.h("td", {}, "fa (Farsi)");
  _e1068.appendChild(_e1069);
  const _e1070 = WF.h("td", {}, "RTL");
  _e1068.appendChild(_e1070);
  _e1058.appendChild(_e1068);
  const _e1071 = WF.h("tr", {});
  const _e1072 = WF.h("td", {}, "ur (Urdu)");
  _e1071.appendChild(_e1072);
  const _e1073 = WF.h("td", {}, "RTL");
  _e1071.appendChild(_e1073);
  _e1058.appendChild(_e1071);
  const _e1074 = WF.h("tr", {});
  const _e1075 = WF.h("td", {}, "All others");
  _e1074.appendChild(_e1075);
  const _e1076 = WF.h("td", {}, "LTR");
  _e1074.appendChild(_e1076);
  _e1058.appendChild(_e1074);
  _e1018.appendChild(_e1058);
  const _e1077 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1077);
  const _e1078 = WF.h("p", { className: "wf-text wf-text--muted" }, "When setLocale(\"ar\") is called, the HTML element gets dir=\"rtl\" and lang=\"ar\" automatically.");
  _e1018.appendChild(_e1078);
  const _e1079 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1079);
  const _e1080 = WF.h("hr", { className: "wf-divider" });
  _e1018.appendChild(_e1080);
  const _e1081 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1081);
  const _e1082 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Fallback Behavior");
  _e1018.appendChild(_e1082);
  const _e1083 = WF.h("p", { className: "wf-text" }, "If a key is missing in the current locale:");
  _e1018.appendChild(_e1083);
  const _e1084 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1085 = WF.h("p", { className: "wf-text" }, "1. Falls back to the defaultLocale translation");
  _e1084.appendChild(_e1085);
  const _e1086 = WF.h("p", { className: "wf-text" }, "2. If still missing, returns the key itself (e.g., \"nav.home\")");
  _e1084.appendChild(_e1086);
  _e1018.appendChild(_e1084);
  const _e1087 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1087);
  const _e1088 = WF.h("hr", { className: "wf-divider" });
  _e1018.appendChild(_e1088);
  const _e1089 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1089);
  const _e1090 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "SSG + i18n");
  _e1018.appendChild(_e1090);
  const _e1091 = WF.h("p", { className: "wf-text wf-text--muted" }, "When both SSG and i18n are enabled, pages are pre-rendered with the default locale text. After JavaScript loads, locale switching works normally.");
  _e1018.appendChild(_e1091);
  const _e1092 = WF.h("div", { className: "wf-spacer" });
  _e1018.appendChild(_e1092);
  _root.appendChild(_e1018);
  return _root;
}

function Page_GettingStarted(params) {
  const _root = document.createDocumentFragment();
  const _e1093 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1094 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1094);
  const _e1095 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Getting Started");
  _e1093.appendChild(_e1095);
  const _e1096 = WF.h("p", { className: "wf-text wf-text--muted" }, "Get up and running with WebFluent in under a minute.");
  _e1093.appendChild(_e1096);
  const _e1097 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1097);
  const _e1098 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Install");
  _e1093.appendChild(_e1098);
  const _e1099 = WF.h("p", { className: "wf-text" }, "Build from source (requires Rust):");
  _e1093.appendChild(_e1099);
  const _e1100 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1100);
  const _e1101 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1102 = WF.h("div", { className: "wf-card__body" });
  const _e1103 = WF.h("code", { className: "wf-code wf-code--block" }, "git clone https://github.com/user/webfluent.git\ncd webfluent\ncargo build --release");
  _e1102.appendChild(_e1103);
  _e1101.appendChild(_e1102);
  _e1093.appendChild(_e1101);
  const _e1104 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1104);
  const _e1105 = WF.h("p", { className: "wf-text wf-text--muted" }, "The binary is at target/release/wf. Add it to your PATH.");
  _e1093.appendChild(_e1105);
  const _e1106 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1106);
  const _e1107 = WF.h("hr", { className: "wf-divider" });
  _e1093.appendChild(_e1107);
  const _e1108 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1108);
  const _e1109 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Create a Project");
  _e1093.appendChild(_e1109);
  const _e1110 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1110);
  const _e1111 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1112 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1113 = WF.h("div", { className: "wf-card__body" });
  const _e1114 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "SPA");
  _e1113.appendChild(_e1114);
  const _e1115 = WF.h("div", { className: "wf-spacer" });
  _e1113.appendChild(_e1115);
  const _e1116 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Interactive App");
  _e1113.appendChild(_e1116);
  const _e1117 = WF.h("p", { className: "wf-text wf-text--muted" }, "Dashboard with routing, stores, forms, modals, animations.");
  _e1113.appendChild(_e1117);
  const _e1118 = WF.h("div", { className: "wf-spacer" });
  _e1113.appendChild(_e1118);
  const _e1119 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-app -t spa");
  _e1113.appendChild(_e1119);
  _e1112.appendChild(_e1113);
  _e1111.appendChild(_e1112);
  const _e1120 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1121 = WF.h("div", { className: "wf-card__body" });
  const _e1122 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Static");
  _e1121.appendChild(_e1122);
  const _e1123 = WF.h("div", { className: "wf-spacer" });
  _e1121.appendChild(_e1123);
  const _e1124 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Static Site");
  _e1121.appendChild(_e1124);
  const _e1125 = WF.h("p", { className: "wf-text wf-text--muted" }, "Marketing site with SSG, i18n, blog, contact form.");
  _e1121.appendChild(_e1125);
  const _e1126 = WF.h("div", { className: "wf-spacer" });
  _e1121.appendChild(_e1126);
  const _e1127 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-site -t static");
  _e1121.appendChild(_e1127);
  _e1120.appendChild(_e1121);
  _e1111.appendChild(_e1120);
  const _e1128 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1129 = WF.h("div", { className: "wf-card__body" });
  const _e1130 = WF.h("span", { className: "wf-badge wf-badge--info" }, "PDF");
  _e1129.appendChild(_e1130);
  const _e1131 = WF.h("div", { className: "wf-spacer" });
  _e1129.appendChild(_e1131);
  const _e1132 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "PDF Document");
  _e1129.appendChild(_e1132);
  const _e1133 = WF.h("p", { className: "wf-text wf-text--muted" }, "Reports, invoices, docs. Tables, code blocks, auto page breaks.");
  _e1129.appendChild(_e1133);
  const _e1134 = WF.h("div", { className: "wf-spacer" });
  _e1129.appendChild(_e1134);
  const _e1135 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init my-report -t pdf");
  _e1129.appendChild(_e1135);
  _e1128.appendChild(_e1129);
  _e1111.appendChild(_e1128);
  _e1093.appendChild(_e1111);
  const _e1136 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1136);
  const _e1137 = WF.h("hr", { className: "wf-divider" });
  _e1093.appendChild(_e1137);
  const _e1138 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1138);
  const _e1139 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Build and Serve");
  _e1093.appendChild(_e1139);
  const _e1140 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1141 = WF.h("div", { className: "wf-card__body" });
  const _e1142 = WF.h("code", { className: "wf-code wf-code--block" }, "cd my-app\nwf build\nwf serve");
  _e1141.appendChild(_e1142);
  _e1140.appendChild(_e1141);
  _e1093.appendChild(_e1140);
  const _e1143 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1143);
  const _e1144 = WF.h("p", { className: "wf-text wf-text--muted" }, "Open http://localhost:3000 in your browser.");
  _e1093.appendChild(_e1144);
  const _e1145 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1145);
  const _e1146 = WF.h("hr", { className: "wf-divider" });
  _e1093.appendChild(_e1146);
  const _e1147 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1147);
  const _e1148 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Project Structure");
  _e1093.appendChild(_e1148);
  const _e1149 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1150 = WF.h("div", { className: "wf-card__body" });
  const _e1151 = WF.h("code", { className: "wf-code wf-code--block" }, "my-app/\n+-- webfluent.app.json       # Config\n+-- src/\n|   +-- App.wf               # Root (router, layout)\n|   +-- pages/\n|   +-- components/\n|   +-- stores/\n|   +-- translations/\n+-- public/\n+-- build/");
  _e1150.appendChild(_e1151);
  _e1149.appendChild(_e1150);
  _e1093.appendChild(_e1149);
  const _e1152 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1152);
  const _e1153 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1154 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/guide"); } }, "Read the Guide");
  _e1153.appendChild(_e1154);
  const _e1155 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/components"); } }, "Browse Components");
  _e1153.appendChild(_e1155);
  _e1093.appendChild(_e1153);
  const _e1156 = WF.h("div", { className: "wf-spacer" });
  _e1093.appendChild(_e1156);
  _root.appendChild(_e1093);
  return _root;
}

function Page_Cli(params) {
  const _root = document.createDocumentFragment();
  const _e1157 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1158 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1158);
  const _e1159 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "CLI Reference");
  _e1157.appendChild(_e1159);
  const _e1160 = WF.h("p", { className: "wf-text wf-text--muted" }, "The WebFluent command-line interface.");
  _e1157.appendChild(_e1160);
  const _e1161 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1161);
  const _e1162 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf init");
  _e1157.appendChild(_e1162);
  const _e1163 = WF.h("p", { className: "wf-text" }, "Create a new WebFluent project.");
  _e1157.appendChild(_e1163);
  const _e1164 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1164);
  const _e1165 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1166 = WF.h("div", { className: "wf-card__body" });
  const _e1167 = WF.h("code", { className: "wf-code wf-code--block" }, "wf init <name> [--template spa|static|pdf]");
  _e1166.appendChild(_e1167);
  _e1165.appendChild(_e1166);
  _e1157.appendChild(_e1165);
  const _e1168 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1168);
  const _e1169 = WF.h("table", { className: "wf-table" });
  const _e1170 = WF.h("thead", {});
  const _e1171 = WF.h("td", {}, "Argument");
  _e1170.appendChild(_e1171);
  const _e1172 = WF.h("td", {}, "Description");
  _e1170.appendChild(_e1172);
  _e1169.appendChild(_e1170);
  const _e1173 = WF.h("tr", {});
  const _e1174 = WF.h("td", {}, "name");
  _e1173.appendChild(_e1174);
  const _e1175 = WF.h("td", {}, "Project name (creates a directory)");
  _e1173.appendChild(_e1175);
  _e1169.appendChild(_e1173);
  const _e1176 = WF.h("tr", {});
  const _e1177 = WF.h("td", {}, "--template, -t");
  _e1176.appendChild(_e1177);
  const _e1178 = WF.h("td", {}, "Template: spa (default), static, or pdf");
  _e1176.appendChild(_e1178);
  _e1169.appendChild(_e1176);
  _e1157.appendChild(_e1169);
  const _e1179 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1179);
  const _e1180 = WF.h("p", { className: "wf-text wf-text--muted" }, "SPA: interactive app with routing and state. Static: SSG site with i18n. PDF: document generation with tables, headings, and auto page breaks.");
  _e1157.appendChild(_e1180);
  const _e1181 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1181);
  const _e1182 = WF.h("hr", { className: "wf-divider" });
  _e1157.appendChild(_e1182);
  const _e1183 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1183);
  const _e1184 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf build");
  _e1157.appendChild(_e1184);
  const _e1185 = WF.h("p", { className: "wf-text" }, "Compile .wf files to HTML, CSS, and JavaScript.");
  _e1157.appendChild(_e1185);
  const _e1186 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1186);
  const _e1187 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1188 = WF.h("div", { className: "wf-card__body" });
  const _e1189 = WF.h("code", { className: "wf-code wf-code--block" }, "wf build [--dir DIR]");
  _e1188.appendChild(_e1189);
  _e1187.appendChild(_e1188);
  _e1157.appendChild(_e1187);
  const _e1190 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1190);
  const _e1191 = WF.h("table", { className: "wf-table" });
  const _e1192 = WF.h("thead", {});
  const _e1193 = WF.h("td", {}, "Option");
  _e1192.appendChild(_e1193);
  const _e1194 = WF.h("td", {}, "Description");
  _e1192.appendChild(_e1194);
  _e1191.appendChild(_e1192);
  const _e1195 = WF.h("tr", {});
  const _e1196 = WF.h("td", {}, "--dir, -d");
  _e1195.appendChild(_e1196);
  const _e1197 = WF.h("td", {}, "Project directory (default: current directory)");
  _e1195.appendChild(_e1197);
  _e1191.appendChild(_e1195);
  _e1157.appendChild(_e1191);
  const _e1198 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1198);
  const _e1199 = WF.h("p", { className: "wf-text wf-text--muted" }, "The build pipeline: Lex all .wf files, parse to AST, run accessibility linter, generate HTML + CSS + JS, write to output directory.");
  _e1157.appendChild(_e1199);
  const _e1200 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1200);
  const _e1201 = WF.h("p", { className: "wf-text" }, "Output depends on config:");
  _e1157.appendChild(_e1201);
  const _e1202 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1203 = WF.h("p", { className: "wf-text" }, "SPA mode (default): single index.html + app.js + styles.css");
  _e1202.appendChild(_e1203);
  const _e1204 = WF.h("p", { className: "wf-text" }, "SSG mode (ssg: true): one HTML per page + app.js + styles.css");
  _e1202.appendChild(_e1204);
  const _e1205 = WF.h("p", { className: "wf-text" }, "PDF mode (output_type: pdf): a single .pdf file");
  _e1202.appendChild(_e1205);
  _e1157.appendChild(_e1202);
  const _e1206 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1206);
  const _e1207 = WF.h("hr", { className: "wf-divider" });
  _e1157.appendChild(_e1207);
  const _e1208 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1208);
  const _e1209 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf serve");
  _e1157.appendChild(_e1209);
  const _e1210 = WF.h("p", { className: "wf-text" }, "Start a development server that serves the built output.");
  _e1157.appendChild(_e1210);
  const _e1211 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1211);
  const _e1212 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1213 = WF.h("div", { className: "wf-card__body" });
  const _e1214 = WF.h("code", { className: "wf-code wf-code--block" }, "wf serve [--dir DIR]");
  _e1213.appendChild(_e1214);
  _e1212.appendChild(_e1213);
  _e1157.appendChild(_e1212);
  const _e1215 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1215);
  const _e1216 = WF.h("p", { className: "wf-text wf-text--muted" }, "Serves files from the build directory. SPA fallback: all routes serve index.html so client-side routing works. Port is configured in webfluent.app.json (default: 3000).");
  _e1157.appendChild(_e1216);
  const _e1217 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1217);
  const _e1218 = WF.h("hr", { className: "wf-divider" });
  _e1157.appendChild(_e1218);
  const _e1219 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1219);
  const _e1220 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "wf generate");
  _e1157.appendChild(_e1220);
  const _e1221 = WF.h("p", { className: "wf-text" }, "Scaffold a new page, component, or store.");
  _e1157.appendChild(_e1221);
  const _e1222 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1222);
  const _e1223 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1224 = WF.h("div", { className: "wf-card__body" });
  const _e1225 = WF.h("code", { className: "wf-code wf-code--block" }, "wf generate <kind> <name> [--dir DIR]");
  _e1224.appendChild(_e1225);
  _e1223.appendChild(_e1224);
  _e1157.appendChild(_e1223);
  const _e1226 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1226);
  const _e1227 = WF.h("table", { className: "wf-table" });
  const _e1228 = WF.h("thead", {});
  const _e1229 = WF.h("td", {}, "Kind");
  _e1228.appendChild(_e1229);
  const _e1230 = WF.h("td", {}, "Creates");
  _e1228.appendChild(_e1230);
  const _e1231 = WF.h("td", {}, "Example");
  _e1228.appendChild(_e1231);
  _e1227.appendChild(_e1228);
  const _e1232 = WF.h("tr", {});
  const _e1233 = WF.h("td", {}, "page");
  _e1232.appendChild(_e1233);
  const _e1234 = WF.h("td", {}, "src/pages/Name.wf");
  _e1232.appendChild(_e1234);
  const _e1235 = WF.h("td", {}, "wf generate page About");
  _e1232.appendChild(_e1235);
  _e1227.appendChild(_e1232);
  const _e1236 = WF.h("tr", {});
  const _e1237 = WF.h("td", {}, "component");
  _e1236.appendChild(_e1237);
  const _e1238 = WF.h("td", {}, "src/components/Name.wf");
  _e1236.appendChild(_e1238);
  const _e1239 = WF.h("td", {}, "wf generate component Header");
  _e1236.appendChild(_e1239);
  _e1227.appendChild(_e1236);
  const _e1240 = WF.h("tr", {});
  const _e1241 = WF.h("td", {}, "store");
  _e1240.appendChild(_e1241);
  const _e1242 = WF.h("td", {}, "src/stores/name.wf");
  _e1240.appendChild(_e1242);
  const _e1243 = WF.h("td", {}, "wf generate store CartStore");
  _e1240.appendChild(_e1243);
  _e1227.appendChild(_e1240);
  _e1157.appendChild(_e1227);
  const _e1244 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1244);
  const _e1245 = WF.h("hr", { className: "wf-divider" });
  _e1157.appendChild(_e1245);
  const _e1246 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1246);
  const _e1247 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Configuration");
  _e1157.appendChild(_e1247);
  const _e1248 = WF.h("p", { className: "wf-text" }, "All config is in webfluent.app.json at the project root.");
  _e1157.appendChild(_e1248);
  const _e1249 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1249);
  const _e1250 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1251 = WF.h("div", { className: "wf-card__body" });
  const _e1252 = WF.h("code", { className: "wf-code wf-code--block" }, "{\n  \"name\": \"My App\",\n  \"version\": \"1.0.0\",\n  \"author\": \"Your Name\",\n  \"theme\": {\n    \"name\": \"default\",\n    \"mode\": \"light\",\n    \"tokens\": { \"color-primary\": \"#6366F1\" }\n  },\n  \"build\": {\n    \"output\": \"./build\",\n    \"minify\": true,\n    \"ssg\": false,\n    \"output_type\": \"spa\",\n    \"pdf\": {\n      \"page_size\": \"A4\",\n      \"default_font\": \"Helvetica\",\n      \"output_filename\": \"report.pdf\"\n    }\n  },\n  \"dev\": { \"port\": 3000 },\n  \"meta\": {\n    \"title\": \"My App\",\n    \"description\": \"Built with WebFluent\",\n    \"lang\": \"en\"\n  },\n  \"i18n\": {\n    \"defaultLocale\": \"en\",\n    \"locales\": [\"en\", \"ar\"],\n    \"dir\": \"src/translations\"\n  }\n}");
  _e1251.appendChild(_e1252);
  _e1250.appendChild(_e1251);
  _e1157.appendChild(_e1250);
  const _e1253 = WF.h("div", { className: "wf-spacer" });
  _e1157.appendChild(_e1253);
  _root.appendChild(_e1157);
  return _root;
}

function Page_Home(params) {
  const _counter = WF.signal(0);
  const _taskInput = WF.signal("");
  const _showDemo = WF.signal(false);
  const _root = document.createDocumentFragment();
  const _e1254 = WF.h("div", { className: "wf-container" });
  const _e1255 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1255);
  const _e1256 = WF.h("h2", { className: "wf-heading wf-heading--h1 wf-text--center wf-animate-slideUp" }, () => WF.i18n.t("hero.title"));
  _e1254.appendChild(_e1256);
  const _e1257 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1257);
  const _e1258 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, () => WF.i18n.t("hero.sub1"));
  _e1254.appendChild(_e1258);
  const _e1259 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center wf-animate-fadeIn" }, () => WF.i18n.t("hero.sub2"));
  _e1254.appendChild(_e1259);
  const _e1260 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1260);
  const _e1261 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1262 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, () => WF.i18n.t("hero.cta"));
  _e1261.appendChild(_e1262);
  const _e1263 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { WF.navigate("/guide"); } }, () => WF.i18n.t("hero.guide"));
  _e1261.appendChild(_e1263);
  _e1254.appendChild(_e1261);
  const _e1264 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1264);
  const _e1265 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1266 = WF.h("div", { className: "wf-card__body" });
  const _e1267 = WF.h("code", { className: "wf-code wf-code--block" }, "Page Home (path: \"/\") {\n    Container {\n        Heading(\"Hello, WebFluent!\", h1)\n        Text(\"Build for the web. Nothing else.\")\n\n        Button(\"Get Started\", primary, large) {\n            navigate(\"/docs\")\n        }\n    }\n}");
  _e1266.appendChild(_e1267);
  _e1265.appendChild(_e1266);
  _e1254.appendChild(_e1265);
  const _e1268 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1268);
  const _e1269 = WF.h("hr", { className: "wf-divider" });
  _e1254.appendChild(_e1269);
  const _e1270 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1270);
  const _e1271 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, () => WF.i18n.t("demo.title"));
  _e1254.appendChild(_e1271);
  const _e1272 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, () => WF.i18n.t("demo.subtitle"));
  _e1254.appendChild(_e1272);
  const _e1273 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1273);
  const _e1274 = WF.h("div", { className: "wf-grid wf-grid--gap-lg", style: { gridTemplateColumns: 'repeat(2, 1fr)' } });
  const _e1275 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1276 = WF.h("div", { className: "wf-card__header" });
  const _e1277 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("demo.counter"));
  _e1276.appendChild(_e1277);
  _e1275.appendChild(_e1276);
  const _e1278 = WF.h("div", { className: "wf-card__body" });
  const _e1279 = WF.h("div", { className: "wf-row wf-row--center wf-row--gap-md" });
  const _e1280 = WF.h("button", { className: "wf-btn wf-btn--large", "on:click": (e) => { _counter.set((_counter() - 1)); } }, "-");
  _e1279.appendChild(_e1280);
  const _e1281 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-heading--primary" }, () => `${_counter()}`);
  _e1279.appendChild(_e1281);
  const _e1282 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { _counter.set((_counter() + 1)); } }, "+");
  _e1279.appendChild(_e1282);
  _e1278.appendChild(_e1279);
  const _e1283 = WF.h("div", { className: "wf-spacer" });
  _e1278.appendChild(_e1283);
  const _e1284 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.counter.hint"));
  _e1278.appendChild(_e1284);
  _e1275.appendChild(_e1278);
  _e1274.appendChild(_e1275);
  const _e1285 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1286 = WF.h("div", { className: "wf-card__header" });
  const _e1287 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("demo.binding"));
  _e1286.appendChild(_e1287);
  _e1285.appendChild(_e1286);
  const _e1288 = WF.h("div", { className: "wf-card__body" });
  const _e1289 = WF.h("input", { className: "wf-input", value: () => _taskInput(), "on:input": (e) => _taskInput.set(e.target.value), placeholder: WF.i18n.t("demo.binding.placeholder"), label: "Input", type: "text" });
  _e1288.appendChild(_e1289);
  const _e1290 = WF.h("div", { className: "wf-spacer" });
  _e1288.appendChild(_e1290);
  WF.condRender(_e1288,
    () => (_taskInput() !== ""),
    () => {
      const _e1291 = document.createDocumentFragment();
      const _e1292 = WF.h("div", { className: "wf-alert wf-alert--info" }, () => `You typed: ${_taskInput()}`);
      _e1291.appendChild(_e1292);
      return _e1291;
    },
    null,
    null
  );
  const _e1293 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.binding.hint"));
  _e1288.appendChild(_e1293);
  _e1285.appendChild(_e1288);
  _e1274.appendChild(_e1285);
  const _e1294 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1295 = WF.h("div", { className: "wf-card__header" });
  const _e1296 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("demo.conditional"));
  _e1295.appendChild(_e1296);
  _e1294.appendChild(_e1295);
  const _e1297 = WF.h("div", { className: "wf-card__body" });
  const _e1298 = WF.h("label", { className: "wf-switch" });
  const _e1299 = WF.h("input", { type: "checkbox", checked: () => _showDemo(), "on:change": () => _showDemo.set(!_showDemo()) });
  _e1298.appendChild(_e1299);
  const _e1300 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1298.appendChild(_e1300);
  _e1298.appendChild(WF.text(WF.i18n.t("demo.conditional.toggle")));
  _e1297.appendChild(_e1298);
  const _e1301 = WF.h("div", { className: "wf-spacer" });
  _e1297.appendChild(_e1301);
  WF.condRender(_e1297,
    () => _showDemo(),
    () => {
      const _e1302 = document.createDocumentFragment();
      const _e1303 = WF.h("div", { className: "wf-card wf-card--outlined" });
      const _e1304 = WF.h("div", { className: "wf-card__body" });
      const _e1305 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Visible!");
      _e1304.appendChild(_e1305);
      const _e1306 = WF.h("div", { className: "wf-spacer" });
      _e1304.appendChild(_e1306);
      const _e1307 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("demo.conditional.text"));
      _e1304.appendChild(_e1307);
      _e1303.appendChild(_e1304);
      _e1302.appendChild(_e1303);
      return _e1302;
    },
    null,
    { enter: "slideUp", exit: "fadeOut" }
  );
  _e1294.appendChild(_e1297);
  _e1274.appendChild(_e1294);
  const _e1308 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
  const _e1309 = WF.h("div", { className: "wf-card__header" });
  const _e1310 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("demo.components"));
  _e1309.appendChild(_e1310);
  _e1308.appendChild(_e1309);
  const _e1311 = WF.h("div", { className: "wf-card__body" });
  const _e1312 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1313 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1314 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e1313.appendChild(_e1314);
  const _e1315 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e1313.appendChild(_e1315);
  const _e1316 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e1313.appendChild(_e1316);
  _e1312.appendChild(_e1313);
  const _e1317 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1318 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "New");
  _e1317.appendChild(_e1318);
  const _e1319 = WF.h("span", { className: "wf-badge wf-badge--danger" }, "Sale");
  _e1317.appendChild(_e1319);
  const _e1320 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Active");
  _e1317.appendChild(_e1320);
  const _e1321 = WF.h("span", { className: "wf-tag" }, "WebFluent");
  _e1317.appendChild(_e1321);
  _e1312.appendChild(_e1317);
  const _e1322 = WF.h("progress", { className: "wf-progress", value: 72, max: 100 });
  _e1312.appendChild(_e1322);
  const _e1323 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => WF.i18n.t("demo.components.hint"));
  _e1312.appendChild(_e1323);
  _e1311.appendChild(_e1312);
  _e1308.appendChild(_e1311);
  _e1274.appendChild(_e1308);
  _e1254.appendChild(_e1274);
  const _e1324 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1324);
  const _e1325 = WF.h("hr", { className: "wf-divider" });
  _e1254.appendChild(_e1325);
  const _e1326 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1326);
  const _e1327 = WF.h("h2", { className: "wf-heading wf-heading--h2 wf-text--center" }, () => WF.i18n.t("why.title"));
  _e1254.appendChild(_e1327);
  const _e1328 = WF.h("p", { className: "wf-text wf-text--muted wf-text--center" }, () => WF.i18n.t("why.subtitle"));
  _e1254.appendChild(_e1328);
  const _e1329 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1329);
  const _e1330 = WF.h("div", { className: "wf-grid wf-grid--gap-md", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1331 = Component_FeatureCard({ title: WF.i18n.t("why.syntax"), description: WF.i18n.t("why.syntax.desc") });
  _e1330.appendChild(_e1331);
  const _e1332 = Component_FeatureCard({ title: WF.i18n.t("why.components"), description: WF.i18n.t("why.components.desc") });
  _e1330.appendChild(_e1332);
  const _e1333 = Component_FeatureCard({ title: WF.i18n.t("why.reactivity"), description: WF.i18n.t("why.reactivity.desc") });
  _e1330.appendChild(_e1333);
  const _e1334 = Component_FeatureCard({ title: WF.i18n.t("why.design"), description: WF.i18n.t("why.design.desc") });
  _e1330.appendChild(_e1334);
  const _e1335 = Component_FeatureCard({ title: WF.i18n.t("why.animation"), description: WF.i18n.t("why.animation.desc") });
  _e1330.appendChild(_e1335);
  const _e1336 = Component_FeatureCard({ title: WF.i18n.t("why.i18n"), description: WF.i18n.t("why.i18n.desc") });
  _e1330.appendChild(_e1336);
  const _e1337 = Component_FeatureCard({ title: WF.i18n.t("why.ssg"), description: WF.i18n.t("why.ssg.desc") });
  _e1330.appendChild(_e1337);
  const _e1338 = Component_FeatureCard({ title: WF.i18n.t("why.a11y"), description: WF.i18n.t("why.a11y.desc") });
  _e1330.appendChild(_e1338);
  const _e1339 = Component_FeatureCard({ title: WF.i18n.t("why.zero"), description: WF.i18n.t("why.zero.desc") });
  _e1330.appendChild(_e1339);
  _e1254.appendChild(_e1330);
  const _e1340 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1340);
  const _e1341 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1342 = WF.h("div", { className: "wf-card__body" });
  const _e1343 = WF.h("div", { className: "wf-row wf-row--center wf-row--between" });
  const _e1344 = WF.h("div", { className: "wf-stack" });
  const _e1345 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, () => WF.i18n.t("cta.title"));
  _e1344.appendChild(_e1345);
  const _e1346 = WF.h("p", { className: "wf-text wf-text--muted" }, () => WF.i18n.t("cta.subtitle"));
  _e1344.appendChild(_e1346);
  _e1343.appendChild(_e1344);
  const _e1347 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large", "on:click": (e) => { WF.navigate("/getting-started"); } }, () => WF.i18n.t("hero.cta"));
  _e1343.appendChild(_e1347);
  _e1342.appendChild(_e1343);
  _e1341.appendChild(_e1342);
  _e1254.appendChild(_e1341);
  const _e1348 = WF.h("div", { className: "wf-spacer" });
  _e1254.appendChild(_e1348);
  _root.appendChild(_e1254);
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
  const _e1349 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1350 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1350);
  const _e1351 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Components Reference");
  _e1349.appendChild(_e1351);
  const _e1352 = WF.h("p", { className: "wf-text wf-text--muted" }, "50+ built-in components. Below are live interactive examples you can play with.");
  _e1349.appendChild(_e1352);
  const _e1353 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1353);
  const _e1354 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Buttons");
  _e1349.appendChild(_e1354);
  const _e1355 = WF.h("p", { className: "wf-text" }, "Buttons support size, color, and shape modifiers.");
  _e1349.appendChild(_e1355);
  const _e1356 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1356);
  const _e1357 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1358 = WF.h("div", { className: "wf-card__body" });
  const _e1359 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1360 = WF.h("button", { className: "wf-btn" }, "Default");
  _e1359.appendChild(_e1360);
  const _e1361 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Primary");
  _e1359.appendChild(_e1361);
  const _e1362 = WF.h("button", { className: "wf-btn wf-btn--success" }, "Success");
  _e1359.appendChild(_e1362);
  const _e1363 = WF.h("button", { className: "wf-btn wf-btn--danger" }, "Danger");
  _e1359.appendChild(_e1363);
  const _e1364 = WF.h("button", { className: "wf-btn wf-btn--warning" }, "Warning");
  _e1359.appendChild(_e1364);
  const _e1365 = WF.h("button", { className: "wf-btn wf-btn--info" }, "Info");
  _e1359.appendChild(_e1365);
  _e1358.appendChild(_e1359);
  const _e1366 = WF.h("div", { className: "wf-spacer" });
  _e1358.appendChild(_e1366);
  const _e1367 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1368 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Small");
  _e1367.appendChild(_e1368);
  const _e1369 = WF.h("button", { className: "wf-btn wf-btn--primary" }, "Medium");
  _e1367.appendChild(_e1369);
  const _e1370 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--large" }, "Large");
  _e1367.appendChild(_e1370);
  const _e1371 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--rounded" }, "Rounded");
  _e1367.appendChild(_e1371);
  const _e1372 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--full" }, "Full Width");
  _e1367.appendChild(_e1372);
  _e1358.appendChild(_e1367);
  _e1357.appendChild(_e1358);
  _e1349.appendChild(_e1357);
  const _e1373 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1373);
  const _e1374 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1375 = WF.h("div", { className: "wf-card__body" });
  const _e1376 = WF.h("code", { className: "wf-code wf-code--block" }, "Button(\"Primary\", primary)\nButton(\"Large\", primary, large)\nButton(\"Rounded\", success, rounded)\nButton(\"Full Width\", danger, full)");
  _e1375.appendChild(_e1376);
  _e1374.appendChild(_e1375);
  _e1349.appendChild(_e1374);
  const _e1377 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1377);
  const _e1378 = WF.h("hr", { className: "wf-divider" });
  _e1349.appendChild(_e1378);
  const _e1379 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1379);
  const _e1380 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Cards");
  _e1349.appendChild(_e1380);
  const _e1381 = WF.h("p", { className: "wf-text" }, "Cards are surfaces for grouping content. They support Header, Body, and Footer sub-components.");
  _e1349.appendChild(_e1381);
  const _e1382 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1382);
  const _e1383 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1384 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e1385 = WF.h("div", { className: "wf-card" });
  const _e1386 = WF.h("div", { className: "wf-card__header" });
  const _e1387 = WF.h("p", { className: "wf-text wf-text--bold" }, "Default Card");
  _e1386.appendChild(_e1387);
  _e1385.appendChild(_e1386);
  const _e1388 = WF.h("div", { className: "wf-card__body" });
  const _e1389 = WF.h("p", { className: "wf-text wf-text--muted" }, "Basic card with header and body.");
  _e1388.appendChild(_e1389);
  _e1385.appendChild(_e1388);
  const _e1390 = WF.h("div", { className: "wf-card__footer" });
  const _e1391 = WF.h("button", { className: "wf-btn wf-btn--primary wf-btn--small" }, "Action");
  console.log("clicked");
  _e1390.appendChild(_e1391);
  _e1385.appendChild(_e1390);
  _e1384.appendChild(_e1385);
  _e1383.appendChild(_e1384);
  const _e1392 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e1393 = WF.h("div", { className: "wf-card wf-card--elevated" });
  const _e1394 = WF.h("div", { className: "wf-card__header" });
  const _e1395 = WF.h("p", { className: "wf-text wf-text--bold" }, "Elevated");
  _e1394.appendChild(_e1395);
  _e1393.appendChild(_e1394);
  const _e1396 = WF.h("div", { className: "wf-card__body" });
  const _e1397 = WF.h("p", { className: "wf-text wf-text--muted" }, "Card with shadow elevation.");
  _e1396.appendChild(_e1397);
  _e1393.appendChild(_e1396);
  _e1392.appendChild(_e1393);
  _e1383.appendChild(_e1392);
  const _e1398 = WF.h("div", { className: "wf-col wf-col--4" });
  const _e1399 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1400 = WF.h("div", { className: "wf-card__header" });
  const _e1401 = WF.h("p", { className: "wf-text wf-text--bold" }, "Outlined");
  _e1400.appendChild(_e1401);
  _e1399.appendChild(_e1400);
  const _e1402 = WF.h("div", { className: "wf-card__body" });
  const _e1403 = WF.h("p", { className: "wf-text wf-text--muted" }, "Card with border only.");
  _e1402.appendChild(_e1403);
  _e1399.appendChild(_e1402);
  _e1398.appendChild(_e1399);
  _e1383.appendChild(_e1398);
  _e1349.appendChild(_e1383);
  const _e1404 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1404);
  const _e1405 = WF.h("hr", { className: "wf-divider" });
  _e1349.appendChild(_e1405);
  const _e1406 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1406);
  const _e1407 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Form Controls");
  _e1349.appendChild(_e1407);
  const _e1408 = WF.h("p", { className: "wf-text" }, "All form inputs support two-way binding with the bind: attribute.");
  _e1349.appendChild(_e1408);
  const _e1409 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1409);
  const _e1410 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1411 = WF.h("div", { className: "wf-card__body" });
  const _e1412 = WF.h("div", { className: "wf-stack wf-stack--gap-md" });
  const _e1413 = WF.h("input", { className: "wf-input", value: () => _inputVal(), "on:input": (e) => _inputVal.set(e.target.value), label: "Text Input", placeholder: "Type here...", type: "text" });
  _e1412.appendChild(_e1413);
  WF.condRender(_e1412,
    () => (_inputVal() !== ""),
    () => {
      const _e1414 = document.createDocumentFragment();
      const _e1415 = WF.h("p", { className: "wf-text wf-text--primary wf-text--bold" }, () => `You typed: ${_inputVal()}`);
      _e1414.appendChild(_e1415);
      return _e1414;
    },
    null,
    null
  );
  const _e1416 = WF.h("hr", { className: "wf-divider" });
  _e1412.appendChild(_e1416);
  const _e1417 = WF.h("select", { className: "wf-select", value: () => _selectVal(), "on:input": (e) => _selectVal.set(e.target.value), label: "Select" });
  const _e1418 = WF.h("option", {}, "opt1");
  _e1417.appendChild(_e1418);
  const _e1419 = WF.h("option", {}, "opt2");
  _e1417.appendChild(_e1419);
  const _e1420 = WF.h("option", {}, "opt3");
  _e1417.appendChild(_e1420);
  _e1412.appendChild(_e1417);
  const _e1421 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Selected: ${_selectVal()}`);
  _e1412.appendChild(_e1421);
  const _e1422 = WF.h("hr", { className: "wf-divider" });
  _e1412.appendChild(_e1422);
  const _e1423 = WF.h("label", { className: "wf-checkbox" });
  const _e1424 = WF.h("input", { type: "checkbox", checked: () => _checkVal(), "on:change": () => _checkVal.set(!_checkVal()) });
  _e1423.appendChild(_e1424);
  _e1423.appendChild(WF.text("I agree to the terms"));
  _e1412.appendChild(_e1423);
  const _e1425 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Checked: ${_checkVal()}`);
  _e1412.appendChild(_e1425);
  const _e1426 = WF.h("hr", { className: "wf-divider" });
  _e1412.appendChild(_e1426);
  const _e1427 = WF.h("div", { className: "wf-row wf-row--gap-lg" });
  const _e1428 = WF.h("label", { className: "wf-radio" });
  const _e1429 = WF.h("input", { type: "radio", checked: () => _radioVal() === "a", "on:change": () => _radioVal.set("a") });
  _e1428.appendChild(_e1429);
  _e1428.appendChild(WF.text("Option A"));
  _e1427.appendChild(_e1428);
  const _e1430 = WF.h("label", { className: "wf-radio" });
  const _e1431 = WF.h("input", { type: "radio", checked: () => _radioVal() === "b", "on:change": () => _radioVal.set("b") });
  _e1430.appendChild(_e1431);
  _e1430.appendChild(WF.text("Option B"));
  _e1427.appendChild(_e1430);
  const _e1432 = WF.h("label", { className: "wf-radio" });
  const _e1433 = WF.h("input", { type: "radio", checked: () => _radioVal() === "c", "on:change": () => _radioVal.set("c") });
  _e1432.appendChild(_e1433);
  _e1432.appendChild(WF.text("Option C"));
  _e1427.appendChild(_e1432);
  _e1412.appendChild(_e1427);
  const _e1434 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Selected: ${_radioVal()}`);
  _e1412.appendChild(_e1434);
  const _e1435 = WF.h("hr", { className: "wf-divider" });
  _e1412.appendChild(_e1435);
  const _e1436 = WF.h("label", { className: "wf-switch" });
  const _e1437 = WF.h("input", { type: "checkbox", checked: () => _switchVal(), "on:change": () => _switchVal.set(!_switchVal()) });
  _e1436.appendChild(_e1437);
  const _e1438 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1436.appendChild(_e1438);
  _e1436.appendChild(WF.text("Dark Mode"));
  _e1412.appendChild(_e1436);
  const _e1439 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Enabled: ${_switchVal()}`);
  _e1412.appendChild(_e1439);
  const _e1440 = WF.h("hr", { className: "wf-divider" });
  _e1412.appendChild(_e1440);
  const _e1441 = WF.h("input", { className: "wf-slider", value: () => _sliderVal(), "on:input": (e) => _sliderVal.set(e.target.value), min: 0, max: 100, step: 1, label: "Volume" });
  _e1412.appendChild(_e1441);
  const _e1442 = WF.h("p", { className: "wf-text wf-text--muted wf-text--small" }, () => `Value: ${_sliderVal()}`);
  _e1412.appendChild(_e1442);
  _e1411.appendChild(_e1412);
  _e1410.appendChild(_e1411);
  _e1349.appendChild(_e1410);
  const _e1443 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1443);
  const _e1444 = WF.h("hr", { className: "wf-divider" });
  _e1349.appendChild(_e1444);
  const _e1445 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1445);
  const _e1446 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Feedback");
  _e1349.appendChild(_e1446);
  const _e1447 = WF.h("p", { className: "wf-text" }, "Alerts, modals, progress bars, and loading indicators.");
  _e1349.appendChild(_e1447);
  const _e1448 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1448);
  const _e1449 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1450 = WF.h("div", { className: "wf-card__body" });
  const _e1451 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1452 = WF.h("div", { className: "wf-alert wf-alert--success" }, "This is a success alert.");
  _e1451.appendChild(_e1452);
  const _e1453 = WF.h("div", { className: "wf-alert wf-alert--warning" }, "This is a warning alert.");
  _e1451.appendChild(_e1453);
  const _e1454 = WF.h("div", { className: "wf-alert wf-alert--danger" }, "This is a danger alert.");
  _e1451.appendChild(_e1454);
  const _e1455 = WF.h("div", { className: "wf-alert wf-alert--info" }, "This is an info alert.");
  _e1451.appendChild(_e1455);
  _e1450.appendChild(_e1451);
  const _e1456 = WF.h("div", { className: "wf-spacer" });
  _e1450.appendChild(_e1456);
  const _e1457 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1458 = WF.h("div", { className: "wf-spinner" });
  _e1457.appendChild(_e1458);
  const _e1459 = WF.h("div", { className: "wf-spinner wf-spinner--large wf-spinner--primary" });
  _e1457.appendChild(_e1459);
  const _e1460 = WF.h("progress", { className: "wf-progress", value: _sliderVal(), max: 100 });
  _e1457.appendChild(_e1460);
  _e1450.appendChild(_e1457);
  const _e1461 = WF.h("div", { className: "wf-spacer" });
  _e1450.appendChild(_e1461);
  const _e1462 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { _activeModal.set(true); } }, "Open Modal");
  _e1450.appendChild(_e1462);
  _e1449.appendChild(_e1450);
  _e1349.appendChild(_e1449);
  const _e1463 = WF.h("div", { className: "wf-modal" });
  const _e1464 = WF.h("div", { className: "wf-modal__content" });
  const _e1465 = WF.h("div", { className: "wf-modal__header" }, WF.h("h3", {}, "Example Modal"));
  _e1464.appendChild(_e1465);
  const _e1466 = WF.h("div", { className: "wf-modal__body" });
  const _e1467 = WF.h("p", { className: "wf-text" }, "This is a real modal dialog. It was triggered by clicking the button.");
  _e1466.appendChild(_e1467);
  const _e1468 = WF.h("div", { className: "wf-spacer" });
  _e1466.appendChild(_e1468);
  const _e1469 = WF.h("p", { className: "wf-text wf-text--muted" }, "The modal is controlled by a state variable.");
  _e1466.appendChild(_e1469);
  _e1464.appendChild(_e1466);
  const _e1470 = WF.h("div", { className: "wf-modal__footer" });
  const _e1471 = WF.h("button", { className: "wf-btn", "on:click": (e) => { _activeModal.set(false); } }, "Close");
  _e1470.appendChild(_e1471);
  const _e1472 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { _activeModal.set(false); } }, "Confirm");
  _e1470.appendChild(_e1472);
  _e1464.appendChild(_e1470);
  _e1463.appendChild(_e1464);
  WF.effect(() => { _e1463.className = _activeModal() ? 'wf-modal open' : 'wf-modal'; });
  _e1349.appendChild(_e1463);
  const _e1473 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1473);
  const _e1474 = WF.h("hr", { className: "wf-divider" });
  _e1349.appendChild(_e1474);
  const _e1475 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1475);
  const _e1476 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Data Display");
  _e1349.appendChild(_e1476);
  const _e1477 = WF.h("p", { className: "wf-text" }, "Tables, badges, avatars, tags, and tooltips.");
  _e1349.appendChild(_e1477);
  const _e1478 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1478);
  const _e1479 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1480 = WF.h("div", { className: "wf-card__body" });
  const _e1481 = WF.h("table", { className: "wf-table" });
  const _e1482 = WF.h("thead", {});
  const _e1483 = WF.h("td", {}, "Name");
  _e1482.appendChild(_e1483);
  const _e1484 = WF.h("td", {}, "Role");
  _e1482.appendChild(_e1484);
  const _e1485 = WF.h("td", {}, "Status");
  _e1482.appendChild(_e1485);
  _e1481.appendChild(_e1482);
  const _e1486 = WF.h("tr", {});
  const _e1487 = WF.h("td", {}, "Monzer Omer");
  _e1486.appendChild(_e1487);
  const _e1488 = WF.h("td", {}, "Creator");
  _e1486.appendChild(_e1488);
  const _e1489 = WF.h("td", {}, "Active");
  _e1486.appendChild(_e1489);
  _e1481.appendChild(_e1486);
  const _e1490 = WF.h("tr", {});
  const _e1491 = WF.h("td", {}, "Sara Ali");
  _e1490.appendChild(_e1491);
  const _e1492 = WF.h("td", {}, "Designer");
  _e1490.appendChild(_e1492);
  const _e1493 = WF.h("td", {}, "Active");
  _e1490.appendChild(_e1493);
  _e1481.appendChild(_e1490);
  const _e1494 = WF.h("tr", {});
  const _e1495 = WF.h("td", {}, "Omar Hassan");
  _e1494.appendChild(_e1495);
  const _e1496 = WF.h("td", {}, "Developer");
  _e1494.appendChild(_e1496);
  const _e1497 = WF.h("td", {}, "Away");
  _e1494.appendChild(_e1497);
  _e1481.appendChild(_e1494);
  _e1480.appendChild(_e1481);
  const _e1498 = WF.h("div", { className: "wf-spacer" });
  _e1480.appendChild(_e1498);
  const _e1499 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1500 = WF.h("div", { className: "wf-avatar wf-avatar--primary", initials: "MO" });
  _e1499.appendChild(_e1500);
  const _e1501 = WF.h("div", { className: "wf-avatar wf-avatar--success", initials: "SA" });
  _e1499.appendChild(_e1501);
  const _e1502 = WF.h("div", { className: "wf-avatar wf-avatar--info", initials: "OH" });
  _e1499.appendChild(_e1502);
  const _e1503 = WF.h("span", { className: "wf-badge wf-badge--primary" }, "Admin");
  _e1499.appendChild(_e1503);
  const _e1504 = WF.h("span", { className: "wf-badge wf-badge--success" }, "Online");
  _e1499.appendChild(_e1504);
  const _e1505 = WF.h("span", { className: "wf-tag" }, "WebFluent");
  _e1499.appendChild(_e1505);
  const _e1506 = WF.h("span", { className: "wf-tag" }, "Rust");
  _e1499.appendChild(_e1506);
  const _e1507 = WF.h("span", { className: "wf-tag" }, "Open Source");
  _e1499.appendChild(_e1507);
  _e1480.appendChild(_e1499);
  _e1479.appendChild(_e1480);
  _e1349.appendChild(_e1479);
  const _e1508 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1508);
  const _e1509 = WF.h("hr", { className: "wf-divider" });
  _e1349.appendChild(_e1509);
  const _e1510 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1510);
  const _e1511 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Layout");
  _e1349.appendChild(_e1511);
  const _e1512 = WF.h("p", { className: "wf-text" }, "Container, Row, Column, Grid, Stack, Spacer, Divider.");
  _e1349.appendChild(_e1512);
  const _e1513 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1513);
  const _e1514 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1515 = WF.h("div", { className: "wf-card__body" });
  const _e1516 = WF.h("p", { className: "wf-text wf-text--bold" }, "Grid with 3 columns:");
  _e1515.appendChild(_e1516);
  const _e1517 = WF.h("div", { className: "wf-spacer" });
  _e1515.appendChild(_e1517);
  const _e1518 = WF.h("div", { className: "wf-grid wf-grid--gap-sm", style: { gridTemplateColumns: 'repeat(3, 1fr)' } });
  const _e1519 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1520 = WF.h("div", { className: "wf-card__body" });
  const _e1521 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 1");
  _e1520.appendChild(_e1521);
  _e1519.appendChild(_e1520);
  _e1518.appendChild(_e1519);
  const _e1522 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1523 = WF.h("div", { className: "wf-card__body" });
  const _e1524 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 2");
  _e1523.appendChild(_e1524);
  _e1522.appendChild(_e1523);
  _e1518.appendChild(_e1522);
  const _e1525 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1526 = WF.h("div", { className: "wf-card__body" });
  const _e1527 = WF.h("p", { className: "wf-text wf-text--center" }, "Column 3");
  _e1526.appendChild(_e1527);
  _e1525.appendChild(_e1526);
  _e1518.appendChild(_e1525);
  _e1515.appendChild(_e1518);
  const _e1528 = WF.h("div", { className: "wf-spacer" });
  _e1515.appendChild(_e1528);
  const _e1529 = WF.h("p", { className: "wf-text wf-text--bold" }, "Row with Columns (6/6 split):");
  _e1515.appendChild(_e1529);
  const _e1530 = WF.h("div", { className: "wf-spacer" });
  _e1515.appendChild(_e1530);
  const _e1531 = WF.h("div", { className: "wf-row wf-row--gap-sm" });
  const _e1532 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1533 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1534 = WF.h("div", { className: "wf-card__body" });
  const _e1535 = WF.h("p", { className: "wf-text wf-text--center" }, "Left Half");
  _e1534.appendChild(_e1535);
  _e1533.appendChild(_e1534);
  _e1532.appendChild(_e1533);
  _e1531.appendChild(_e1532);
  const _e1536 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1537 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1538 = WF.h("div", { className: "wf-card__body" });
  const _e1539 = WF.h("p", { className: "wf-text wf-text--center" }, "Right Half");
  _e1538.appendChild(_e1539);
  _e1537.appendChild(_e1538);
  _e1536.appendChild(_e1537);
  _e1531.appendChild(_e1536);
  _e1515.appendChild(_e1531);
  const _e1540 = WF.h("div", { className: "wf-spacer" });
  _e1515.appendChild(_e1540);
  const _e1541 = WF.h("p", { className: "wf-text wf-text--bold" }, "Stack (vertical):");
  _e1515.appendChild(_e1541);
  const _e1542 = WF.h("div", { className: "wf-spacer" });
  _e1515.appendChild(_e1542);
  const _e1543 = WF.h("div", { className: "wf-stack wf-stack--gap-sm" });
  const _e1544 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1545 = WF.h("div", { className: "wf-card__body" });
  const _e1546 = WF.h("p", { className: "wf-text" }, "Item 1");
  _e1545.appendChild(_e1546);
  _e1544.appendChild(_e1545);
  _e1543.appendChild(_e1544);
  const _e1547 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1548 = WF.h("div", { className: "wf-card__body" });
  const _e1549 = WF.h("p", { className: "wf-text" }, "Item 2");
  _e1548.appendChild(_e1549);
  _e1547.appendChild(_e1548);
  _e1543.appendChild(_e1547);
  const _e1550 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1551 = WF.h("div", { className: "wf-card__body" });
  const _e1552 = WF.h("p", { className: "wf-text" }, "Item 3");
  _e1551.appendChild(_e1552);
  _e1550.appendChild(_e1551);
  _e1543.appendChild(_e1550);
  _e1515.appendChild(_e1543);
  _e1514.appendChild(_e1515);
  _e1349.appendChild(_e1514);
  const _e1553 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1553);
  const _e1554 = WF.h("hr", { className: "wf-divider" });
  _e1349.appendChild(_e1554);
  const _e1555 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1555);
  const _e1556 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Navigation");
  _e1349.appendChild(_e1556);
  const _e1557 = WF.h("p", { className: "wf-text" }, "Tabs let you switch between content panels.");
  _e1349.appendChild(_e1557);
  const _e1558 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1558);
  const _e1559 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1560 = WF.h("div", { className: "wf-card__body" });
  const _e1561 = WF.h("div", { className: "wf-tabs" });
  const _e1562 = WF.h("div", { className: "wf-tabs__nav" });
  const _e1563 = WF.signal(0);
  const _e1564 = WF.h("button", { className: () => _e1563() === 0 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1563.set(0) }, "Profile");
  _e1562.appendChild(_e1564);
  const _e1565 = WF.h("button", { className: () => _e1563() === 1 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1563.set(1) }, "Settings");
  _e1562.appendChild(_e1565);
  const _e1566 = WF.h("button", { className: () => _e1563() === 2 ? "wf-tabs__tab active" : "wf-tabs__tab", "on:click": () => _e1563.set(2) }, "About");
  _e1562.appendChild(_e1566);
  _e1561.appendChild(_e1562);
  const _e1567 = WF.h("div", { className: "wf-tab-page" });
  const _e1568 = WF.h("div", { className: "wf-spacer" });
  _e1567.appendChild(_e1568);
  const _e1569 = WF.h("div", { className: "wf-row wf-row--gap-md wf-row--center" });
  const _e1570 = WF.h("div", { className: "wf-avatar wf-avatar--primary wf-avatar--large", initials: "MO" });
  _e1569.appendChild(_e1570);
  const _e1571 = WF.h("div", { className: "wf-stack" });
  const _e1572 = WF.h("p", { className: "wf-text wf-text--bold" }, "Monzer Omer");
  _e1571.appendChild(_e1572);
  const _e1573 = WF.h("p", { className: "wf-text wf-text--muted" }, "Creator of WebFluent");
  _e1571.appendChild(_e1573);
  _e1569.appendChild(_e1571);
  _e1567.appendChild(_e1569);
  WF.effect(() => { _e1567.style.display = _e1563() === 0 ? 'block' : 'none'; });
  _e1561.appendChild(_e1567);
  const _e1574 = WF.h("div", { className: "wf-tab-page" });
  const _e1575 = WF.h("div", { className: "wf-spacer" });
  _e1574.appendChild(_e1575);
  const _e1576 = WF.h("label", { className: "wf-switch" });
  const _e1577 = WF.h("input", { type: "checkbox", checked: () => _switchVal(), "on:change": () => _switchVal.set(!_switchVal()) });
  _e1576.appendChild(_e1577);
  const _e1578 = WF.h("span", { className: "wf-switch__track" }, WF.h("span", { className: "wf-switch__thumb" }));
  _e1576.appendChild(_e1578);
  _e1576.appendChild(WF.text("Enable notifications"));
  _e1574.appendChild(_e1576);
  const _e1579 = WF.h("div", { className: "wf-spacer" });
  _e1574.appendChild(_e1579);
  const _e1580 = WF.h("input", { className: "wf-slider", value: () => _sliderVal(), "on:input": (e) => _sliderVal.set(e.target.value), min: 0, max: 100, label: "Volume" });
  _e1574.appendChild(_e1580);
  WF.effect(() => { _e1574.style.display = _e1563() === 1 ? 'block' : 'none'; });
  _e1561.appendChild(_e1574);
  const _e1581 = WF.h("div", { className: "wf-tab-page" });
  const _e1582 = WF.h("div", { className: "wf-spacer" });
  _e1581.appendChild(_e1582);
  const _e1583 = WF.h("p", { className: "wf-text" }, "WebFluent is a web-first programming language.");
  _e1581.appendChild(_e1583);
  const _e1584 = WF.h("p", { className: "wf-text wf-text--muted" }, "It compiles to HTML, CSS, and JavaScript.");
  _e1581.appendChild(_e1584);
  WF.effect(() => { _e1581.style.display = _e1563() === 2 ? 'block' : 'none'; });
  _e1561.appendChild(_e1581);
  _e1560.appendChild(_e1561);
  _e1559.appendChild(_e1560);
  _e1349.appendChild(_e1559);
  const _e1585 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1585);
  const _e1586 = WF.h("hr", { className: "wf-divider" });
  _e1349.appendChild(_e1586);
  const _e1587 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1587);
  const _e1588 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Typography");
  _e1349.appendChild(_e1588);
  const _e1589 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1589);
  const _e1590 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1591 = WF.h("div", { className: "wf-card__body" });
  const _e1592 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Heading h2");
  _e1591.appendChild(_e1592);
  const _e1593 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Heading h2");
  _e1591.appendChild(_e1593);
  const _e1594 = WF.h("h2", { className: "wf-heading wf-heading--h3" }, "Heading h3");
  _e1591.appendChild(_e1594);
  const _e1595 = WF.h("div", { className: "wf-spacer" });
  _e1591.appendChild(_e1595);
  const _e1596 = WF.h("p", { className: "wf-text" }, "Normal text paragraph.");
  _e1591.appendChild(_e1596);
  const _e1597 = WF.h("p", { className: "wf-text wf-text--bold" }, "Bold text.");
  _e1591.appendChild(_e1597);
  const _e1598 = WF.h("p", { className: "wf-text wf-text--muted" }, "Muted text.");
  _e1591.appendChild(_e1598);
  const _e1599 = WF.h("p", { className: "wf-text wf-text--primary" }, "Primary colored.");
  _e1591.appendChild(_e1599);
  const _e1600 = WF.h("p", { className: "wf-text wf-text--danger" }, "Danger colored.");
  _e1591.appendChild(_e1600);
  const _e1601 = WF.h("p", { className: "wf-text wf-text--small" }, "Small text.");
  _e1591.appendChild(_e1601);
  const _e1602 = WF.h("p", { className: "wf-text wf-text--uppercase" }, "Uppercase.");
  _e1591.appendChild(_e1602);
  const _e1603 = WF.h("p", { className: "wf-text wf-text--center" }, "Centered text.");
  _e1591.appendChild(_e1603);
  const _e1604 = WF.h("div", { className: "wf-spacer" });
  _e1591.appendChild(_e1604);
  const _e1605 = WF.h("blockquote", { className: "wf-blockquote" }, "The best way to predict the future is to create it.");
  _e1591.appendChild(_e1605);
  const _e1606 = WF.h("div", { className: "wf-spacer" });
  _e1591.appendChild(_e1606);
  const _e1607 = WF.h("code", { className: "wf-code" }, "const greeting = \"Hello, WebFluent!\";");
  _e1591.appendChild(_e1607);
  _e1590.appendChild(_e1591);
  _e1349.appendChild(_e1590);
  const _e1608 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1608);
  const _e1609 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1610 = WF.h("button", { className: "wf-btn wf-btn--primary", "on:click": (e) => { WF.navigate("/styling"); } }, "Styling Guide");
  _e1609.appendChild(_e1610);
  const _e1611 = WF.h("button", { className: "wf-btn", "on:click": (e) => { WF.navigate("/animation"); } }, "Animation System");
  _e1609.appendChild(_e1611);
  _e1349.appendChild(_e1609);
  const _e1612 = WF.h("div", { className: "wf-spacer" });
  _e1349.appendChild(_e1612);
  _root.appendChild(_e1349);
  return _root;
}

function Page_Accessibility(params) {
  const _root = document.createDocumentFragment();
  const _e1613 = WF.h("div", { className: "wf-container wf-animate-fadeIn" });
  const _e1614 = WF.h("div", { className: "wf-spacer" });
  _e1613.appendChild(_e1614);
  const _e1615 = WF.h("h2", { className: "wf-heading wf-heading--h1" }, "Accessibility Linting");
  _e1613.appendChild(_e1615);
  const _e1616 = WF.h("p", { className: "wf-text wf-text--muted" }, "WebFluent checks your code for accessibility issues at compile time. Warnings are printed during build but never block compilation.");
  _e1613.appendChild(_e1616);
  const _e1617 = WF.h("div", { className: "wf-spacer" });
  _e1613.appendChild(_e1617);
  const _e1618 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "How It Works");
  _e1613.appendChild(_e1618);
  const _e1619 = WF.h("p", { className: "wf-text" }, "The linter runs automatically after parsing, before code generation. It walks the AST and checks each component against 12 rules.");
  _e1613.appendChild(_e1619);
  const _e1620 = WF.h("div", { className: "wf-spacer" });
  _e1613.appendChild(_e1620);
  const _e1621 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1622 = WF.h("div", { className: "wf-card__body" });
  const _e1623 = WF.h("code", { className: "wf-code wf-code--block" }, "$ wf build\nBuilding my-app...\n  Warning [A01]: Image missing \"alt\" attribute at src/pages/Home.wf:12:5\n    Add alt text: Image(src: \"...\", alt: \"Description of image\")\n  Warning [A03]: Input missing \"label\" attribute at src/pages/Form.wf:8:9\n    Add a label: Input(text, label: \"Username\")\n  3 pages, 2 components, 1 stores\n  Build complete with 2 accessibility warning(s).");
  _e1622.appendChild(_e1623);
  _e1621.appendChild(_e1622);
  _e1613.appendChild(_e1621);
  const _e1624 = WF.h("div", { className: "wf-spacer" });
  _e1613.appendChild(_e1624);
  const _e1625 = WF.h("hr", { className: "wf-divider" });
  _e1613.appendChild(_e1625);
  const _e1626 = WF.h("div", { className: "wf-spacer" });
  _e1613.appendChild(_e1626);
  const _e1627 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Lint Rules");
  _e1613.appendChild(_e1627);
  const _e1628 = WF.h("table", { className: "wf-table" });
  const _e1629 = WF.h("thead", {});
  const _e1630 = WF.h("td", {}, "Rule");
  _e1629.appendChild(_e1630);
  const _e1631 = WF.h("td", {}, "Component");
  _e1629.appendChild(_e1631);
  const _e1632 = WF.h("td", {}, "Check");
  _e1629.appendChild(_e1632);
  _e1628.appendChild(_e1629);
  const _e1633 = WF.h("tr", {});
  const _e1634 = WF.h("td", {}, "A01");
  _e1633.appendChild(_e1634);
  const _e1635 = WF.h("td", {}, "Image");
  _e1633.appendChild(_e1635);
  const _e1636 = WF.h("td", {}, "Must have alt attribute");
  _e1633.appendChild(_e1636);
  _e1628.appendChild(_e1633);
  const _e1637 = WF.h("tr", {});
  const _e1638 = WF.h("td", {}, "A02");
  _e1637.appendChild(_e1638);
  const _e1639 = WF.h("td", {}, "IconButton");
  _e1637.appendChild(_e1639);
  const _e1640 = WF.h("td", {}, "Must have label attribute (no visible text)");
  _e1637.appendChild(_e1640);
  _e1628.appendChild(_e1637);
  const _e1641 = WF.h("tr", {});
  const _e1642 = WF.h("td", {}, "A03");
  _e1641.appendChild(_e1642);
  const _e1643 = WF.h("td", {}, "Input");
  _e1641.appendChild(_e1643);
  const _e1644 = WF.h("td", {}, "Must have label or placeholder");
  _e1641.appendChild(_e1644);
  _e1628.appendChild(_e1641);
  const _e1645 = WF.h("tr", {});
  const _e1646 = WF.h("td", {}, "A04");
  _e1645.appendChild(_e1646);
  const _e1647 = WF.h("td", {}, "Checkbox, Radio, Switch, Slider");
  _e1645.appendChild(_e1647);
  const _e1648 = WF.h("td", {}, "Must have label attribute");
  _e1645.appendChild(_e1648);
  _e1628.appendChild(_e1645);
  const _e1649 = WF.h("tr", {});
  const _e1650 = WF.h("td", {}, "A05");
  _e1649.appendChild(_e1650);
  const _e1651 = WF.h("td", {}, "Button");
  _e1649.appendChild(_e1651);
  const _e1652 = WF.h("td", {}, "Must have text content");
  _e1649.appendChild(_e1652);
  _e1628.appendChild(_e1649);
  const _e1653 = WF.h("tr", {});
  const _e1654 = WF.h("td", {}, "A06");
  _e1653.appendChild(_e1654);
  const _e1655 = WF.h("td", {}, "Link");
  _e1653.appendChild(_e1655);
  const _e1656 = WF.h("td", {}, "Must have text content or children");
  _e1653.appendChild(_e1656);
  _e1628.appendChild(_e1653);
  const _e1657 = WF.h("tr", {});
  const _e1658 = WF.h("td", {}, "A07");
  _e1657.appendChild(_e1658);
  const _e1659 = WF.h("td", {}, "Heading");
  _e1657.appendChild(_e1659);
  const _e1660 = WF.h("td", {}, "Must not be empty");
  _e1657.appendChild(_e1660);
  _e1628.appendChild(_e1657);
  const _e1661 = WF.h("tr", {});
  const _e1662 = WF.h("td", {}, "A08");
  _e1661.appendChild(_e1662);
  const _e1663 = WF.h("td", {}, "Modal, Dialog");
  _e1661.appendChild(_e1663);
  const _e1664 = WF.h("td", {}, "Must have title attribute");
  _e1661.appendChild(_e1664);
  _e1628.appendChild(_e1661);
  const _e1665 = WF.h("tr", {});
  const _e1666 = WF.h("td", {}, "A09");
  _e1665.appendChild(_e1666);
  const _e1667 = WF.h("td", {}, "Video");
  _e1665.appendChild(_e1667);
  const _e1668 = WF.h("td", {}, "Must have controls attribute");
  _e1665.appendChild(_e1668);
  _e1628.appendChild(_e1665);
  const _e1669 = WF.h("tr", {});
  const _e1670 = WF.h("td", {}, "A10");
  _e1669.appendChild(_e1670);
  const _e1671 = WF.h("td", {}, "Table");
  _e1669.appendChild(_e1671);
  const _e1672 = WF.h("td", {}, "Must have Thead header row");
  _e1669.appendChild(_e1672);
  _e1628.appendChild(_e1669);
  const _e1673 = WF.h("tr", {});
  const _e1674 = WF.h("td", {}, "A11");
  _e1673.appendChild(_e1674);
  const _e1675 = WF.h("td", {}, "Heading");
  _e1673.appendChild(_e1675);
  const _e1676 = WF.h("td", {}, "Levels must not skip (h1 to h3)");
  _e1673.appendChild(_e1676);
  _e1628.appendChild(_e1673);
  const _e1677 = WF.h("tr", {});
  const _e1678 = WF.h("td", {}, "A12");
  _e1677.appendChild(_e1678);
  const _e1679 = WF.h("td", {}, "Page");
  _e1677.appendChild(_e1679);
  const _e1680 = WF.h("td", {}, "Must have exactly one h1");
  _e1677.appendChild(_e1680);
  _e1628.appendChild(_e1677);
  _e1613.appendChild(_e1628);
  const _e1681 = WF.h("div", { className: "wf-spacer" });
  _e1613.appendChild(_e1681);
  const _e1682 = WF.h("hr", { className: "wf-divider" });
  _e1613.appendChild(_e1682);
  const _e1683 = WF.h("div", { className: "wf-spacer" });
  _e1613.appendChild(_e1683);
  const _e1684 = WF.h("h2", { className: "wf-heading wf-heading--h2" }, "Examples");
  _e1613.appendChild(_e1684);
  const _e1685 = WF.h("div", { className: "wf-row wf-row--gap-md" });
  const _e1686 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1687 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1688 = WF.h("div", { className: "wf-card__body" });
  const _e1689 = WF.h("p", { className: "wf-text wf-text--danger wf-text--bold" }, "Bad (triggers warning)");
  _e1688.appendChild(_e1689);
  const _e1690 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\")\nIconButton(icon: \"close\")\nInput(text)\nCheckbox(bind: agreed)\nButton()");
  _e1688.appendChild(_e1690);
  _e1687.appendChild(_e1688);
  _e1686.appendChild(_e1687);
  _e1685.appendChild(_e1686);
  const _e1691 = WF.h("div", { className: "wf-col wf-col--6" });
  const _e1692 = WF.h("div", { className: "wf-card wf-card--outlined" });
  const _e1693 = WF.h("div", { className: "wf-card__body" });
  const _e1694 = WF.h("p", { className: "wf-text wf-text--success wf-text--bold" }, "Good (no warnings)");
  _e1693.appendChild(_e1694);
  const _e1695 = WF.h("code", { className: "wf-code wf-code--block" }, "Image(src: \"/photo.jpg\", alt: \"Team photo\")\nIconButton(icon: \"close\", label: \"Close\")\nInput(text, label: \"Username\")\nCheckbox(bind: agreed, label: \"I agree\")\nButton(\"Save\")");
  _e1693.appendChild(_e1695);
  _e1692.appendChild(_e1693);
  _e1691.appendChild(_e1692);
  _e1685.appendChild(_e1691);
  _e1613.appendChild(_e1685);
  const _e1696 = WF.h("div", { className: "wf-spacer" });
  _e1613.appendChild(_e1696);
  _root.appendChild(_e1613);
  return _root;
}

(function() {
  const _app = document.getElementById('app');
  _app.innerHTML = '';
  const _e1697 = Component_NavBar({});
  _app.appendChild(_e1697);
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
    { path: "/pdf", render: (params) => Page_Pdf(params) },
    { path: "/template-engine", render: (params) => Page_TemplateEngine(params) },
    { path: "/accessibility", render: (params) => Page_Accessibility(params) },
    { path: "/cli", render: (params) => Page_Cli(params) },
    { path: "/404", render: (params) => Page_NotFound(params) },
  ];
  WF.createRouter(_routes, _routerEl);
  const _e1698 = Component_SiteFooter({});
  _app.appendChild(_e1698);
})();
