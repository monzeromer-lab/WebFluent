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
  function effect(fn) {
    let cleanup = null;
    let dead = false;
    const run = () => {
      if (dead) return;
      if (typeof cleanup === "function") { const c = cleanup; cleanup = null; c(); }
      const prev = currentEffect;
      currentEffect = run;
      try { cleanup = fn(); } finally { currentEffect = prev; }
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
  function onKey(target, spelling, fn) {
    listen(target, "keydown", (e) => { if (keyIs(e, spelling)) fn(e); });
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

  // `persist name = value`: a signal whose value lives in localStorage
  // under `wf:<key>`, read on creation and written on every change; the
  // initial value stands where storage is empty or unavailable.
  function persist(key, initial) {
    const name = "wf:" + key;
    let value = initial;
    try {
      const stored = window.localStorage.getItem(name);
      if (stored !== null) value = JSON.parse(stored);
    } catch (e) { /* storage blocked or unreadable: the initial value */ }
    const s = signal(value);
    const set = s.set;
    s.set = (v) => {
      set(v);
      try { window.localStorage.setItem(name, JSON.stringify(s())); } catch (e) { /* full or blocked */ }
    };
    s.update = (fn) => s.set(fn(s()));
    return s;
  }

  // ─── DOM Helpers ─────────────────────────────────────
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
        } else if (k === "markdown") {
          if (typeof v === "function") {
            effect(() => { el.innerHTML = markdown(v()); });
          } else {
            el.innerHTML = markdown(v);
          }
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
        } else if (k === "data-wf-delay" || k === "data-wf-duration" || k === "data-wf-easing") {
          // The element's own timing for the animation class it carries.
          el.setAttribute(k, v);
          el.style[TIMING[k]] = v;
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

  // ─── Animation helpers ──────────────────────────────
  // A reader who asked for less motion gets none: no class, no wait.
  function reducedMotion() {
    return typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  }

  // Play the animation class `name` on `el` once, with the timing given
  // (the stylesheet's when not), and resolve when it has ended.
  function play(el, name, duration, delay, easing) {
    if (!name || reducedMotion()) return Promise.resolve();
    const cls = "wf-animate-" + name;
    if (duration) el.style.animationDuration = duration;
    if (delay) el.style.animationDelay = delay;
    if (easing) el.style.animationTimingFunction = easing;
    el.classList.add(cls);
    return new Promise(resolve => {
      const done = () => {
        el.classList.remove(cls);
        el.style.animationDuration = "";
        el.style.animationDelay = "";
        el.style.animationTimingFunction = "";
        resolve();
      };
      el.addEventListener("animationend", done, { once: true });
      // Fallback timeout
      setTimeout(done, (parseInt(duration) || 300) + (parseInt(delay) || 0) + 100);
    });
  }

  function animateIn(el, name, duration, delay, easing) {
    return play(el, name, duration, delay, easing);
  }

  function animateOut(el, name, duration, delay, easing) {
    return play(el, name, duration, delay, easing);
  }

  // ─── Helpers on lists and strings ─────────────────────
  // `items.sortBy(x => x.name)`, `items.groupBy(x => x.kind)`, `unique`,
  // `take(n)`, `first`, `last`; `text.capitalize()`, `text.truncate(n)`.
  function sortBy(list, key) {
    return Array.from(list).sort((a, b) => {
      const ka = key(a), kb = key(b);
      return ka < kb ? -1 : ka > kb ? 1 : 0;
    });
  }
  function groupBy(list, key) {
    const groups = {};
    for (const item of list) {
      const k = String(key(item));
      (groups[k] || (groups[k] = [])).push(item);
    }
    return groups;
  }
  function unique(list) {
    const seen = new Set();
    const out = [];
    for (const item of list) {
      const k = typeof item === "object" && item !== null ? JSON.stringify(item) : item;
      if (!seen.has(k)) { seen.add(k); out.push(item); }
    }
    return out;
  }
  function take(list, n) { return Array.from(list).slice(0, Math.max(0, n)); }
  // `a..b` and `a..=b`: the whole numbers from `a`, up to `b`.
  function range(a, b, inclusive) {
    const out = [];
    const end = inclusive ? b : b - 1;
    for (let i = a; i <= end; i++) out.push(i);
    return out;
  }
  function first(list) { return list.length ? list[0] : null; }
  function last(list) { return list.length ? list[list.length - 1] : null; }
  function capitalize(text) {
    const s = String(text);
    return s ? s[0].toUpperCase() + s.slice(1) : s;
  }
  function truncate(text, n) {
    const chars = Array.from(String(text));
    return chars.length > n ? chars.slice(0, n).join("") + "\u2026" : chars.join("");
  }

  // ─── Formatting ──────────────────────────────────────
  // `format(value, style, option)` and `ago(date)` speak the page's locale:
  // the i18n locale when the project has one — a signal, so a change of
  // locale redraws every formatted text — else the document's language.
  // A style is a case: `.number` (the default), `.integer`, `.decimal`
  // (option: places, 2 by default), `.currency` (option: the code, USD by
  // default), `.percent` (of a fraction; option: places), `.compact`,
  // `.date`, `.time`, `.datetime` (option: `short`/`medium`/`long`/`full`)
  // and `.relative`. A string style is a date pattern: `yyyy-MM-dd HH:mm`.
  function currentLocale() {
    if (i18nInstance) return i18nInstance.locale();
    return document.documentElement.lang || (typeof navigator !== "undefined" && navigator.language) || "en";
  }
  const DATE_STYLES = new Set(["date", "time", "datetime"]);
  function format(value, style, option) {
    const locale = currentLocale();
    if (value == null) return "";
    if (typeof style === "string" && !DATE_STYLES.has(style) && /[yMdEHhmsa]/.test(style) && !/^(number|integer|decimal|currency|percent|compact|relative)$/.test(style)) {
      return formatPattern(toDate(value), style, locale);
    }
    switch (style) {
      case "integer":
        return new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(Number(value));
      case "decimal": {
        const places = option == null ? 2 : Number(option);
        return new Intl.NumberFormat(locale, { minimumFractionDigits: places, maximumFractionDigits: places }).format(Number(value));
      }
      case "currency":
        return new Intl.NumberFormat(locale, { style: "currency", currency: option || "USD" }).format(Number(value));
      case "percent":
        return new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: option == null ? 0 : Number(option) }).format(Number(value));
      case "compact":
        return new Intl.NumberFormat(locale, { notation: "compact" }).format(Number(value));
      case "date":
        return new Intl.DateTimeFormat(locale, { dateStyle: option || "medium" }).format(toDate(value));
      case "time":
        return new Intl.DateTimeFormat(locale, { timeStyle: option || "short" }).format(toDate(value));
      case "datetime":
        return new Intl.DateTimeFormat(locale, { dateStyle: option || "medium", timeStyle: "short" }).format(toDate(value));
      case "relative":
        return ago(value);
      default:
        return new Intl.NumberFormat(locale).format(Number(value));
    }
  }
  function toDate(value) {
    if (value instanceof Date) return value;
    // A bare date is a day, not midnight UTC shifted into the zone.
    if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}$/.test(value)) {
      const [y, m, d] = value.split("-").map(Number);
      return new Date(y, m - 1, d);
    }
    return new Date(value);
  }
  function formatPattern(date, pattern, locale) {
    const names = (opts) => new Intl.DateTimeFormat(locale, opts).format(date);
    const pad = (n, w = 2) => String(n).padStart(w, "0");
    const h12 = date.getHours() % 12 || 12;
    return pattern.replace(/yyyy|yy|MMMM|MMM|MM|M|dd|d|EEEE|EEE|HH|H|hh|h|mm|ss|a/g, (t) => {
      switch (t) {
        case "yyyy": return String(date.getFullYear());
        case "yy": return pad(date.getFullYear() % 100);
        case "MMMM": return names({ month: "long" });
        case "MMM": return names({ month: "short" });
        case "MM": return pad(date.getMonth() + 1);
        case "M": return String(date.getMonth() + 1);
        case "dd": return pad(date.getDate());
        case "d": return String(date.getDate());
        case "EEEE": return names({ weekday: "long" });
        case "EEE": return names({ weekday: "short" });
        case "HH": return pad(date.getHours());
        case "H": return String(date.getHours());
        case "hh": return pad(h12);
        case "h": return String(h12);
        case "mm": return pad(date.getMinutes());
        case "ss": return pad(date.getSeconds());
        case "a": return date.getHours() < 12 ? "AM" : "PM";
        default: return t;
      }
    });
  }
  // `ago(date)`: "3 minutes ago", "yesterday", "in 2 weeks".
  const AGO_UNITS = [["year", 31536000], ["month", 2592000], ["week", 604800], ["day", 86400], ["hour", 3600], ["minute", 60]];
  function ago(value, now) {
    const locale = currentLocale();
    const then = toDate(value).getTime();
    if (then !== then) return ""; // an unreadable date
    const seconds = Math.round((then - (now == null ? Date.now() : toDate(now).getTime())) / 1000);
    const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
    for (const [unit, size] of AGO_UNITS) {
      if (Math.abs(seconds) >= size) return rtf.format(Math.trunc(seconds / size), unit);
    }
    if (Math.abs(seconds) < 45) return rtf.format(0, "second");
    return rtf.format(seconds, "second");
  }

  // `Form(bind: form)`: a handle on the form — `form.valid` (every control
  // passes its own checks), `form.values` (by field name), `form.reset()`,
  // `form.submit()` — kept current as the reader types.
  function form() {
    const valid = signal(false);
    const values = signal({});
    let el = null;
    const read = () => {
      if (!el) return;
      valid.set(typeof el.checkValidity === "function" ? el.checkValidity() : true);
      const out = {};
      const fields = el.elements ? Array.from(el.elements) : Array.from(el.querySelectorAll("input, select, textarea"));
      for (const f of fields) {
        const name = f.name || (typeof f.getAttribute === "function" && f.getAttribute("name"));
        if (!name) continue;
        const type = f.type || (typeof f.getAttribute === "function" && f.getAttribute("type"));
        if (type === "checkbox") out[name] = !!f.checked;
        else if (type === "radio") { if (f.checked) out[name] = f.value; }
        else out[name] = f.value;
      }
      values.set(out);
    };
    return {
      get current() { return el; },
      set current(node) {
        el = node;
        if (!el) return;
        el.addEventListener("input", read);
        el.addEventListener("change", read);
        el.addEventListener("reset", () => setTimeout(read, 0));
        read();
      },
      get element() { return el; },
      get valid() { return valid(); },
      get values() { return values(); },
      reset() { if (el && typeof el.reset === "function") el.reset(); read(); },
      submit() { if (el && typeof el.requestSubmit === "function") el.requestSubmit(); },
    };
  }

  // ─── Markdown ────────────────────────────────────────
  // The same small Markdown the compiler renders at build time (see
  // `src/codegen/markdown.rs`): headings, paragraphs, fenced code, quotes,
  // one-level lists, rules; code spans, strong, emphasis, links, images.
  // The text is escaped first, so HTML in it is shown, not run.
  function mdEscape(text) {
    return String(text).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\x22/g, "&quot;");
  }
  // Written as `RegExp` over strings, so a tool that scans the bundle for
  // balanced brackets is not misled by a bracket inside a pattern.
  const MD = {
    heading: new RegExp("^(#{1,6}) (.*)$"),
    bullet: new RegExp("^[-*] (.*)$"),
    numbered: new RegExp("^[0-9]+\\. (.*)$"),
    code: new RegExp("\\x60([^\\x60]+)\\x60", "g"),
    image: new RegExp("!\\[([^\\]]*)\\]\\(([^)\\s]+)\\)", "g"),
    link: new RegExp("\\[([^\\]]+)\\]\\(([^)\\s]+)\\)", "g"),
    strong: new RegExp("\\*\\*([^*]+)\\*\\*", "g"),
    em: new RegExp("\\*([^*]+)\\*", "g"),
    em2: new RegExp("(^|[^A-Za-z0-9])_([^_]+)_($|[^A-Za-z0-9])", "g"),
  };
  function mdHeading(line) {
    const m = MD.heading.exec(line);
    return m ? [m[1].length, m[2].trim()] : null;
  }
  function mdListItem(line) {
    let m = MD.bullet.exec(line);
    if (m) return [m[1].trim(), false];
    m = MD.numbered.exec(line);
    if (m) return [m[1].trim(), true];
    return null;
  }
  function mdInline(text) {
    const spans = [];
    let s = mdEscape(text).replace(MD.code, (_, c) => { spans.push("<code>" + c + "</code>"); return "\u0000" + (spans.length - 1) + "\u0000"; });
    s = s.replace(MD.image, '<img src="$2" alt="$1">');
    s = s.replace(MD.link, '<a href="$2">$1</a>');
    s = s.replace(MD.strong, "<strong>$1</strong>");
    s = s.replace(MD.em, "<em>$1</em>");
    s = s.replace(MD.em2, "$1<em>$2</em>$3");
    spans.forEach((span, i) => { s = s.split("\u0000" + i + "\u0000").join(span); });
    return s.replace(/\n/g, "<br>\n");
  }
  function markdown(text) {
    const lines = String(text == null ? "" : text).split("\n");
    let out = "";
    let i = 0;
    const isBlock = (t) => t === "" || t.startsWith("```") || t === "---" || t === "***" || mdHeading(t) || t.startsWith(">") || mdListItem(t);
    while (i < lines.length) {
      const t = lines[i].trim();
      if (t === "") { i++; continue; }
      if (t.startsWith("```")) {
        const lang = t.slice(3).trim();
        let code = "";
        i++;
        while (i < lines.length && lines[i].trim() !== "```") { code += mdEscape(lines[i]) + "\n"; i++; }
        i++;
        out += lang ? '<pre><code class="language-' + mdEscape(lang) + '">' + code + "</code></pre>\n" : "<pre><code>" + code + "</code></pre>\n";
        continue;
      }
      if (t === "---" || t === "***") { out += "<hr>\n"; i++; continue; }
      const h = mdHeading(t);
      if (h) { out += "<h" + h[0] + ">" + mdInline(h[1]) + "</h" + h[0] + ">\n"; i++; continue; }
      if (t.startsWith(">")) {
        const quoted = [];
        while (i < lines.length && lines[i].trim().startsWith(">")) { quoted.push(lines[i].trim().slice(1).replace(/^\s+/, "")); i++; }
        out += "<blockquote>\n" + markdown(quoted.join("\n")) + "</blockquote>\n";
        continue;
      }
      const item = mdListItem(t);
      if (item) {
        const ordered = item[1];
        const tag = ordered ? "ol" : "ul";
        out += "<" + tag + ">\n";
        while (i < lines.length) {
          const it = mdListItem(lines[i].trim());
          if (!it || it[1] !== ordered) break;
          out += "<li>" + mdInline(it[0]) + "</li>\n";
          i++;
        }
        out += "</" + tag + ">\n";
        continue;
      }
      const para = [];
      while (i < lines.length && !isBlock(lines[i].trim())) { para.push(lines[i].trim()); i++; }
      out += "<p>" + mdInline(para.join("\n")) + "</p>\n";
    }
    return out;
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

  // ─── Leaving ─────────────────────────────────────────
  // Every exit the nodes on their way out asked for: the branch's own, on
  // each root, and the one any element beneath carries as `data-wf-exit`.
  // The promises resolve when the last has played; none means remove now.
  function leave(nodes, config) {
    const plays = [];
    if (reducedMotion()) return plays;
    const marked = (el) => {
      if (!el.hasAttribute || !el.hasAttribute("data-wf-exit")) return;
      const attr = (name) => el.getAttribute(name) || "";
      plays.push(animateOut(el, attr("data-wf-exit"), attr("data-wf-duration"), attr("data-wf-delay"), attr("data-wf-easing")));
    };
    for (const n of nodes) {
      if (!(n instanceof Element)) continue;
      if (config && config.exit) plays.push(animateOut(n, config.exit, config.duration, "", config.easing));
      else marked(n);
      if (typeof n.querySelectorAll === "function") {
        for (const el of n.querySelectorAll("[data-wf-exit]")) marked(el);
      }
    }
    return plays;
  }

  // Remove `nodes` once their exits have played, then call `onDone`. The
  // handle returned (none when nothing played) is cancelled by a branch
  // that comes back before the exit is over.
  function leaveThenRemove(nodes, config, onDone) {
    const plays = leave(nodes, config);
    if (!plays.length) {
      removeNodes(nodes);
      if (onDone) onDone();
      return null;
    }
    const pending = { cancelled: false };
    Promise.all(plays).then(() => {
      if (pending.cancelled) return;
      removeNodes(nodes);
      if (onDone) onDone();
    });
    return pending;
  }

  function animate(target, name, duration) {
    const el = typeof target === "string" ? document.querySelector(`[data-ref="${target}"]`) : target;
    if (!el) return;
    return animateIn(el, name, duration);
  }

  function replay(el, name, duration) {
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

  function when(parent, condFn, thenFn, elseFn, animConfig) {
    const marker = document.createComment("wf-if");
    parent.appendChild(marker);
    let currentNodes = [];
    let lastShow = undefined;
    let pendingRemoval = null; // Track in-progress exit animations
    // What the branch's body created, disposed of when the branch leaves.
    let dispose = null;

    // Only track the condition signal — not signals read during rendering
    effect(() => {
      const show = !!condFn();
      if (show === lastShow) return;
      lastShow = show;
      if (dispose) { dispose(); dispose = null; }

      // A branch that comes back before its exit has played: the old
      // nodes go at once, the new ones take their place.
      if (pendingRemoval) {
        pendingRemoval.pending.cancelled = true;
        removeNodes(pendingRemoval.nodes);
        pendingRemoval = null;
      }

      // Remove old nodes, once what asked for an exit has played it.
      const toRemove = [...currentNodes];
      currentNodes = [];
      const pending = leaveThenRemove(toRemove, animConfig, () => { pendingRemoval = null; });
      if (pending) pendingRemoval = { nodes: toRemove, pending };

      // Add new nodes (untracked so rendering doesn't subscribe this effect to state signals)
      const renderFn = show ? thenFn : elseFn;
      if (renderFn) {
        const prev = currentEffect;
        currentEffect = null; // Untrack: don't subscribe to signals during render
        try {
          const [result, disposer] = scoped(renderFn);
          dispose = disposer;
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
            nodes.forEach(n => { if (n instanceof Element) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay, animConfig.easing); });
          }
        } finally {
          currentEffect = prev;
        }
      }
    });
  }

  // ─── Match ───────────────────────────────────────────
  //
  // Renders the arm named by `key()` — a resource's state, or an enum's case
  // — and renders again when it changes. The arm is called with `arg()`: a
  // resource's data or error, nothing for an enum. Rendering is untracked, as
  // it is for a condition, so only the key decides when to redraw.
  function match(parent, key, arg, arms) {
    const marker = document.createComment("wf-match");
    parent.appendChild(marker);
    let currentNodes = [];
    let lastKey;
    let dispose = null;
    effect(() => {
      const k = key();
      if (k === lastKey) return;
      lastKey = k;
      if (dispose) { dispose(); dispose = null; }
      leaveThenRemove(currentNodes, null);
      currentNodes = [];
      const arm = Object.prototype.hasOwnProperty.call(arms, k) ? arms[k] : arms.else;
      if (!arm) return;
      const prev = currentEffect;
      currentEffect = null;
      try {
        const handed = arg ? arg() : undefined;
        const [result, disposer] = scoped(() => arm(handed));
        dispose = disposer;
        const nodes = result instanceof DocumentFragment
          ? [...result.childNodes]
          : [].concat(result).flat().filter(n => n instanceof Node);
        currentNodes = nodes.slice();
        const frag = document.createDocumentFragment();
        for (const n of nodes) frag.appendChild(n);
        if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
      } finally {
        currentEffect = prev;
      }
    });
  }

  // A scoped slot: `values()` reads what the component hands over, and
  // `render(values)` is the fill's nodes. They are drawn again when a
  // value the handing read changes; the fill itself renders untracked, as
  // a match arm does, so only the handing decides when to redraw.
  function slot(parent, values, render) {
    const marker = document.createComment("wf-slot");
    parent.appendChild(marker);
    let currentNodes = [];
    let dispose = null;
    effect(() => {
      const handed = values();
      if (dispose) { dispose(); dispose = null; }
      leaveThenRemove(currentNodes, null);
      currentNodes = [];
      const prev = currentEffect;
      currentEffect = null;
      let result;
      try {
        [result, dispose] = scoped(() => render(handed));
      } finally {
        currentEffect = prev;
      }
      if (result == null) return;
      const nodes = result instanceof DocumentFragment
        ? [...result.childNodes]
        : [].concat(result).flat().filter(n => n instanceof Node);
      currentNodes = nodes.slice();
      const frag = document.createDocumentFragment();
      for (const n of nodes) frag.appendChild(n);
      if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
    });
  }

  // ─── List rendering ─────────────────────────────────
  // ─── Lists ───────────────────────────────────────────
  // Every item renders to its nodes after the marker. With a `key`, an item
  // keeps its nodes for as long as it is the same value under the same key:
  // items are inserted, removed and moved rather than rebuilt, so the focus,
  // scroll and animation state of the others survive a change. An item whose
  // value changed is rebuilt in place; one whose body reads its index is
  // rebuilt when its position changes. Without a key the list is rebuilt.
  function each(parent, listFn, itemFn, config, keyFn) {
    const marker = document.createComment("wf-for");
    parent.appendChild(marker);
    const key = (config && config.key) || keyFn || null;
    const byIndex = !!(config && config.index);
    let entries = new Map();
    let currentNodes = [];
    const exiting = new Set();
    // What each item's body created, disposed of when the item leaves.
    let disposers = [];

    const toNodes = (result) =>
      result instanceof DocumentFragment
        ? [...result.childNodes]
        : [].concat(result).flat().filter(n => n instanceof Node);
    const enter = (nodes, index) => {
      if (!(config && config.enter)) return;
      for (const n of nodes) {
        if (!(n instanceof Element)) continue;
        const delay = config.stagger ? (parseInt(config.stagger) * index) + "ms" : config.delay;
        animateIn(n, config.enter, config.duration, delay, config.easing);
      }
    };
    const depart = (nodes) => {
      const plays = leave(nodes, config);
      if (!plays.length) { removeNodes(nodes); return; }
      for (const n of nodes) exiting.add(n);
      Promise.all(plays).then(() => {
        for (const n of nodes) exiting.delete(n);
        removeNodes(nodes);
      });
    };
    // Where each kept node was before a change, so a node that moved can
    // slide from there to where it is now (first, last, invert, play).
    const positions = () => {
      const seen = new Map();
      if (reducedMotion() || typeof requestAnimationFrame !== "function") return seen;
      for (const entry of entries.values()) {
        for (const n of entry.nodes) {
          if (n instanceof Element && typeof n.getBoundingClientRect === "function") {
            const r = n.getBoundingClientRect();
            seen.set(n, { x: r.left, y: r.top });
          }
        }
      }
      return seen;
    };
    const slide = (before) => {
      for (const [n, was] of before) {
        if (!n.parentNode || typeof n.getBoundingClientRect !== "function") continue;
        const r = n.getBoundingClientRect();
        const dx = was.x - r.left;
        const dy = was.y - r.top;
        if (!dx && !dy) continue;
        n.style.transition = "none";
        n.style.transform = "translate(" + dx + "px, " + dy + "px)";
        requestAnimationFrame(() => {
          n.style.transition = "transform " + ((config && config.duration) || "200ms") + " ease";
          n.style.transform = "";
          const done = () => { n.style.transition = ""; };
          n.addEventListener("transitionend", done, { once: true });
          setTimeout(done, (parseInt(config && config.duration) || 200) + 100);
        });
      }
    };
    // Put `nodes` right after `after`, moving only what is out of place; a
    // node on its way out does not count as a neighbour.
    const place = (host, nodes, after) => {
      for (const n of nodes) {
        let next = after.nextSibling;
        while (next && exiting.has(next)) next = next.nextSibling;
        if (next !== n) host.insertBefore(n, next);
        after = n;
      }
      return after;
    };
    const untracked = (fn) => {
      const prev = currentEffect;
      currentEffect = null;
      try { return fn(); } finally { currentEffect = prev; }
    };

    effect(() => {
      const items = listFn() || [];
      if (!key) {
        // Rebuilt whole.
        for (const d of disposers.splice(0)) d();
        depart(currentNodes);
        currentNodes = [];
        untracked(() => {
          const frag = document.createDocumentFragment();
          items.forEach((item, index) => {
            const [made, dispose] = scoped(() => itemFn(item, index));
            disposers.push(dispose);
            const nodes = toNodes(made);
            for (const n of nodes) { frag.appendChild(n); currentNodes.push(n); }
            enter(nodes, index);
          });
          if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
        });
        return;
      }
      untracked(() => {
        const host = marker.parentNode;
        if (!host) return;
        const before = positions();
        const next = new Map();
        const seen = new Map();
        let cursor = marker;
        items.forEach((item, index) => {
          let k = key(item, index);
          if (next.has(k)) {
            // Two items under one key: the later ones are told apart by
            // how many came before, and the author is told once.
            const n = (seen.get(k) || 1) + 1;
            seen.set(k, n);
            if (n === 2) console.warn("WF: two items of a list share the key " + JSON.stringify(k) + "; add a `by` that tells them apart");
            k = String(k) + "#" + n;
          }
          const old = entries.get(k);
          let entry;
          if (old && old.item === item && (!byIndex || old.index === index)) {
            entry = old;
            entry.index = index;
          } else {
            const [made, dispose] = scoped(() => itemFn(item, index));
            entry = { item, index, nodes: toNodes(made), dispose };
            if (old) { if (old.dispose) old.dispose(); removeNodes(old.nodes); }
          }
          entries.delete(k);
          cursor = place(host, entry.nodes, cursor);
          if (!old) enter(entry.nodes, index);
          next.set(k, entry);
        });
        // What is left never came back.
        for (const gone of entries.values()) { if (gone.dispose) gone.dispose(); depart(gone.nodes); }
        entries = next;
        slide(before);
      });
    });
  }

  // ─── Show/Hide ───────────────────────────────────────
  function show(parent, condFn, contentFn, animConfig) {
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
            for (const n of wrapper.children) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay, animConfig.easing);
          }
        } else {
          const plays = leave([...wrapper.children], animConfig);
          if (plays.length) Promise.all(plays).then(() => { wrapper.style.display = "none"; });
          else wrapper.style.display = "none";
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

  // ─── The browser as values ───────────────────────────
  // `viewport` is the window's size with the breakpoints as booleans
  // (`viewport.md` from 768px up); `query` the URL's search parameters as
  // a map; `hash` the fragment without its `#`. Each is a signal read once
  // and kept current, so a condition on one follows the browser.
  let _viewport = null;
  function viewport() {
    if (!_viewport) {
      const read = () => {
        const width = window.innerWidth || 0;
        const height = window.innerHeight || 0;
        return { width, height, sm: width >= 640, md: width >= 768, lg: width >= 1024, xl: width >= 1280 };
      };
      _viewport = signal(read());
      window.addEventListener("resize", () => _viewport.set(read()));
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

  // `options.transition` — `fade` or `slide` — plays the old page out and
  // the new one in on a route change; `options.duration` times both.
  function router(routes, container, options) {
    const motion = options && options.transition && options.transition !== "none"
      ? { name: options.transition, duration: options.duration }
      : null;
    const ways = { fade: ["fadeOut", "fadeIn"], slide: ["slideLeft", "slideRight"] };
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
    // What the page on show created, disposed of before the next one.
    let disposePage = null;

    function render() {
      const path = currentPath(); // Only subscribe to path changes
      const match = matchRoute(path);
      if (!match) {
        container.innerHTML = "";
        return;
      }

      const paint = (renderFn) => {
        if (disposePage) { disposePage(); disposePage = null; }
        container.innerHTML = "";
        _newPage();
        // The tab, the history entry and a screen reader all read the title;
        // a single-page app used to keep the entry page's title on every route.
        if (match.route.title) document.title = match.route.title;
        // Untrack: don't subscribe the router effect to signals read during page render
        const prev = currentEffect;
        currentEffect = null;
        try {
          // A page that names a layout is rendered inside it.
          const [el, dispose] = scoped(() => match.route.layout
            ? match.route.layout(renderFn, match.params)
            : renderFn(match.params));
          disposePage = dispose;
          if (el instanceof Node) container.appendChild(el);
        } finally {
          currentEffect = prev;
        }
      };
      const settle = () => {
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
      const draw = (renderFn) => {
        // The page arrived after the reader had already moved on.
        if (currentPath() !== path) return;
        const [out, back] = motion ? ways[motion.name] || ways.fade : [];
        // The first paint, and every one for a reader who asked for less
        // motion, is immediate; a route change plays the old page out, then
        // the new one in, and settles focus once the new page is still.
        if (!motion || !rendered || reducedMotion()) {
          paint(renderFn);
          settle();
          return;
        }
        // Where the browser has the View Transitions API, it plays the
        // change between the two paints itself — a crossfade, or the slide
        // the sheet defines for `data-wf-transition="slide"` — and the
        // class-based animation below is the fallback.
        if (typeof document.startViewTransition === "function") {
          const root = document.documentElement;
          root.setAttribute("data-wf-transition", motion.name);
          if (motion.duration) root.style.setProperty("--animation-duration-normal", motion.duration);
          let transition;
          try {
            transition = document.startViewTransition(() => { paint(renderFn); });
          } catch (e) {
            root.removeAttribute("data-wf-transition");
            paint(renderFn);
            settle();
            return;
          }
          const done = () => {
            root.removeAttribute("data-wf-transition");
            if (motion.duration) root.style.removeProperty("--animation-duration-normal");
            settle();
          };
          if (transition && transition.finished && typeof transition.finished.then === "function") {
            transition.finished.then(done, done);
          } else {
            done();
          }
          return;
        }
        const leaving = [...container.children];
        Promise.all(leaving.map((n) => animateOut(n, out, motion.duration))).then(() => {
          if (currentPath() !== path) return;
          paint(renderFn);
          const arriving = [...container.children];
          Promise.all(arriving.map((n) => animateIn(n, back, motion.duration))).then(settle);
        });
      };

      // A route names its page's render function directly, or names a page
      // that lives in its own chunk and is fetched the first time it shows.
      // A page with a sheet of its own is drawn once the sheet has arrived,
      // so it never paints unstyled.
      const styled = (fn) => (match.route.css ? loadSheet(match.route.css, fn) : fn());
      if (match.route.render) styled(() => draw(match.route.render));
      else loadPage(match.route.page, (renderFn) => styled(() => draw(renderFn)));
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

  function page(name, renderFn) {
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

  // A page's own stylesheet, `pages/<Name>.css`, linked the first time its
  // route shows; `cb` runs once the rules apply. A sheet the page's HTML
  // already linked applied before this script ran.
  const sheets = {};
  function loadSheet(name, cb) {
    if (sheets[name]) return cb();
    const linked = document.querySelector('link[data-wf-page-css="' + name + '"]');
    if (linked) {
      sheets[name] = true;
      return cb();
    }
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = _basePath + "/pages/" + name + ".css";
    link.setAttribute("data-wf-page-css", name);
    const done = () => {
      sheets[name] = true;
      cb();
    };
    link.onload = done;
    link.onerror = () => {
      console.error("WebFluent: could not load the stylesheet for " + name);
      done();
    };
    (document.head || document.body).appendChild(link);
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

  function params() {
    return routerInstance ? routerInstance._currentParams || {} : {};
  }

  // ─── Store ───────────────────────────────────────────
  function store(definition) {
    const store = {};
    const states = {};

    // Create signals for each state; a persisted one reads storage first.
    const kept = definition.persist ? definition.persist.names : [];
    if (definition.state) {
      for (const [key, val] of Object.entries(definition.state)) {
        const initial = typeof val === "function" ? val() : val;
        const s = kept.includes(key) ? persist(definition.persist.prefix + "." + key, initial) : signal(initial);
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
        // An async action carries `pending`: true while a call runs.
        if (fn.constructor && fn.constructor.name === "AsyncFunction") {
          const pending = signal(false);
          store[key] = async (...args) => {
            pending.set(true);
            try { return await fn(store, ...args); } finally { pending.set(false); }
          };
          store[key].pending = pending;
        } else {
          store[key] = (...args) => fn(store, ...args);
        }
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

  // ─── Resource ────────────────────────────────────────
  //
  // A request declared once and read anywhere. `state()` is "loading",
  // "ready" or "error"; `data()` and `error()` hold what arrived; `reload()`
  // asks again. A URL that reads state is followed: the request is made again
  // when it changes, and an answer to an older request is ignored.
  function fetchOptions(options) {
    const opts = {};
    if (options) {
      if (options.method) opts.method = options.method;
      if (options.headers) opts.headers = options.headers;
      if (options.body) {
        opts.body = JSON.stringify(typeof options.body === "function" ? options.body() : options.body);
        opts.headers = { "Content-Type": "application/json", ...(opts.headers || {}) };
      }
    }
    return opts;
  }

  function resource(url, options) {
    const state = signal("loading");
    const data = signal(null);
    const error = signal(null);
    let generation = 0;
    const load = (target) => {
      const gen = ++generation;
      state.set("loading");
      error.set(null);
      fetch(target, fetchOptions(options))
        .then(r => { if (!r.ok) throw new Error(`HTTP ${r.status}`); return r.json(); })
        .then(d => { if (gen !== generation) return; data.set(d); state.set("ready"); })
        .catch(e => { if (gen !== generation) return; error.set(e); state.set("error"); });
    };
    if (typeof url === "function") {
      effect(() => { load(url()); });
    } else {
      load(url);
    }
    return {
      state, data, error,
      reload: () => load(typeof url === "function" ? url() : url),
    };
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

  function toast(message, variant, duration) {
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
  function dialog(el, openSignal) {
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
  function popup(root, trigger, openSignal) {
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
  /// button. `popup` supplies the outside click and Escape.
  function menu(root, trigger, list, openSignal) {
    popup(root, trigger, openSignal);
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

  /// Arrow-key navigation for a `role="tabs"`.
  ///
  /// The WAI-ARIA pattern puts only the selected tab in the tab order and moves
  /// between tabs with the arrow keys, so Tab leaves the widget rather than
  /// walking through every tab in it.
  function tabs(nav, activeSignal) {
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
    const wrapper = el("div", { className: "wf-field" });
    if (!control.id) control.id = "wf-field-" + (++fieldSeq);
    const described = [];
    const existing = control.getAttribute("aria-describedby");
    if (existing) described.push(existing);

    if (opts.label != null) {
      wrapper.appendChild(
        el("label", { className: "wf-label", for: control.id }, opts.label),
      );
    }
    wrapper.appendChild(control);

    if (opts.hint != null) {
      const id = control.id + "-hint";
      wrapper.appendChild(el("p", { className: "wf-field__hint", id }, opts.hint));
      described.push(id);
    }
    if (opts.error !== undefined) {
      const id = control.id + "-error";
      const message = el("p", { className: "wf-field__error", id, role: "alert" });
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

    const nav = el("div", { className: "wf-carousel__nav" });
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
      playButton = el("button", {
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
      el("button", {
        className: "wf-carousel__control wf-carousel__prev",
        type: "button",
        "aria-label": _label("wf.carousel.previous", "Previous slide"),
        "on:click": () => show(index() - 1),
      }, ["\u2039"]),
    );
    const dots = el("div", { className: "wf-carousel__dots" });
    slides.forEach((_, i) => {
      dots.appendChild(
        el("button", {
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
      el("button", {
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
  function drawer(panel, toggle, scrim) {
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

  // ─── Head ────────────────────────────────────────────
  // A page's `head { meta(…) link(…) script(…) }`: the tags are written into
  // the document's head for as long as the page shows, replacing what the
  // static paint put there; an attribute that reads state follows it.
  function head(tags) {
    for (const old of Array.from(document.head.querySelectorAll("[data-wf-head]"))) old.remove();
    const made = [];
    for (const [tag, attrs] of tags) {
      const node = document.createElement(tag);
      node.setAttribute("data-wf-head", "");
      for (const [k, v] of Object.entries(attrs || {})) {
        if (typeof v === "function") {
          effect(() => {
            const value = v();
            if (value === false || value == null) node.removeAttribute(k);
            else node.setAttribute(k, value === true ? "" : String(value));
          });
        } else if (v === true) node.setAttribute(k, "");
        else if (v !== false && v != null) node.setAttribute(k, String(v));
      }
      document.head.appendChild(node);
      made.push(node);
    }
    onCleanup(() => { for (const n of made) n.remove(); });
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

  function locales(defaultLocale, translations) {
    const locale = signal(defaultLocale);
    const dir = signal(RTL_LOCALES.has(defaultLocale) ? "rtl" : "ltr");

    function t(key, params) {
      const currentLocale = locale();
      const messages = translations[currentLocale] || translations[defaultLocale] || {};
      const fallback = translations[defaultLocale] || {};
      const lookup = (k) => (messages[k] !== undefined ? messages[k] : fallback[k]);
      let text;
      // A `count` picks the plural form: `key.one`, `key.other`, and the
      // locale's other categories when the file has them.
      if (params && typeof params.count === "number" && typeof Intl !== "undefined" && Intl.PluralRules) {
        let category = "other";
        try { category = new Intl.PluralRules(currentLocale).select(params.count); } catch (e) { /* unknown locale */ }
        text = lookup(key + "." + category);
        if (text === undefined) text = lookup(key + ".other");
      }
      if (text === undefined) text = lookup(key);
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
    // State.
    signal, effect, computed,
    // Elements, and what a body puts inside them.
    el, text, props, onRoot, mark, classes,
    when, each, show, match,
    // Motion.
    animate, replay, animateIn, animateOut,
    // Routing and pages.
    router, navigate, params, activeLink, page, loadPage, loadSheet, mainOf,
    // Data.
    resource, fetch: wfFetch, store, emit,
    sortBy, groupBy, unique, take, first, last, capitalize, truncate, range,
    caseOf, payload, format, ago, slot,
    scoped, onCleanup, every, after, listen, onKey, keyIs, ref, persist,
    viewport, query, hash, theme, setTheme, form, head, markdown,
    locales,
    // Overlays and widgets.
    toast, dialog, popup, tabs, drawer, announce, carousel, tooltip, menu, field,
    // Boot. The base path is read by every link a static build writes.
    mount, hydrate, setSsgMode, setBasePath,
    get _basePath() { return _basePath; },
    __debug, __reg,
  };
})();
