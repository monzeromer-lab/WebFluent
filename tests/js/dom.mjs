//! The smallest DOM that `runtime.js` actually uses.
//!
//! There was no JavaScript harness in this repository, so every change to
//! `runtime.js` was unverifiable — the reason the runtime accumulated bugs nobody
//! could catch. This is not a browser: it implements exactly the surface the
//! runtime touches (element creation, children, attributes, classes, text,
//! listeners, dispatch), which is enough to assert what the runtime DOES.

/// One of the browser's two storage areas, as much of it as `persist` uses.
function storageArea() {
  const held = new Map();
  return {
    getItem: (k) => (held.has(k) ? held.get(k) : null),
    setItem: (k, v) => held.set(k, String(v)),
    removeItem: (k) => held.delete(k),
    clear: () => held.clear(),
    get length() { return held.size; },
    key: (i) => [...held.keys()][i] ?? null,
  };
}

class ClassList {
  constructor(el) { this.el = el; this._set = new Set(); }
  add(...names) { for (const n of names) if (n) this._set.add(n); }
  remove(...names) { for (const n of names) this._set.delete(n); }
  contains(n) { return this._set.has(n); }
  [Symbol.iterator]() { return this._set.values(); }
  toString() { return [...this._set].join(" "); }
}

/// One running animation, as the Web Animations API hands it back.
///
/// The runtime's motion engine is built on `element.animate`, so a test of
/// it needs an animation it can finish, cancel and inspect. This one runs
/// on demand: `finish()` resolves it, `cancel()` rejects `finished` and
/// takes it off the element, which is what a real cancel does.
class Animation_ {
  constructor(el, frames, options) {
    this.id = (options && options.id) || "";
    this.effect = { target: el, frames, options: options || {} };
    this.playState = "running";
    this.committed = false;
    this.finished = new Promise((resolve, reject) => {
      this._resolve = resolve;
      this._reject = reject;
    });
    // Nobody is waiting on a cancelled animation: not rejecting loudly is
    // what the browser does too.
    this.finished.catch(() => {});
    // Time does not pass here, so an animation finishes as soon as the
    // caller has had a chance to do something with it — which is what a
    // test of what happens *after* an animation needs. A test that drives
    // one itself sets `el._holdAnimations` first.
    if (!el._holdAnimations) Promise.resolve().then(() => this.finish());
  }
  finish() {
    if (this.playState !== "running") return;
    this.playState = "finished";
    this._resolve(this);
  }
  cancel() {
    if (this.playState !== "running") return;
    this.playState = "idle";
    const at = this.effect.target._animations.indexOf(this);
    if (at >= 0) this.effect.target._animations.splice(at, 1);
    this._reject(new Error("cancelled"));
  }
  commitStyles() { this.committed = true; }
}

class Node_ {
  constructor() { this.childNodes = []; this.parentNode = null; }
  get children() { return this.childNodes.filter((n) => n.nodeType === 1); }
  get firstChild() { return this.childNodes[0] || null; }
  get lastChild() { return this.childNodes[this.childNodes.length - 1] || null; }
  get nextSibling() {
    if (!this.parentNode) return null;
    const i = this.parentNode.childNodes.indexOf(this);
    return this.parentNode.childNodes[i + 1] || null;
  }
  get previousSibling() {
    if (!this.parentNode) return null;
    const i = this.parentNode.childNodes.indexOf(this);
    return i > 0 ? this.parentNode.childNodes[i - 1] : null;
  }
  contains(node) {
    if (node === this) return true;
    return this.childNodes.some((c) => c === node || (c.contains && c.contains(node)));
  }
  // A node lives in one place: inserting it elsewhere moves it, and a
  // fragment hands over its children and empties itself, as the DOM does.
  _adopt(n) {
    if (n instanceof DocumentFragment) {
      const moved = n.childNodes.splice(0);
      for (const c of moved) c.parentNode = this;
      return moved;
    }
    if (n.parentNode) n.parentNode.removeChild(n);
    n.parentNode = this;
    return [n];
  }
  removeChild(n) {
    this.childNodes = this.childNodes.filter((c) => c !== n);
    if (n.parentNode === this) n.parentNode = null;
    return n;
  }
  insertBefore(n, ref) {
    const nodes = this._adopt(n);
    const i = ref ? this.childNodes.indexOf(ref) : -1;
    if (i < 0) this.childNodes.push(...nodes); else this.childNodes.splice(i, 0, ...nodes);
    return n;
  }
}

class TextNode extends Node_ {
  constructor(text) { super(); this.nodeType = 3; this._text = String(text); }
  get textContent() { return this._text; }
  set textContent(v) { this._text = String(v); }
}

class Element extends Node_ {
  constructor(tag) {
    super();
    this.nodeType = 1;
    this.tagName = tag.toUpperCase();
    this.attributes = new Map();
    this.classList = new ClassList(this);
    this.style = new Proxy({ _props: new Map() }, {
      get: (t, k) => (k === "setProperty" ? (p, v) => t._props.set(p, v)
        : k === "removeProperty" ? (p) => t._props.delete(p)
        : k === "_props" ? t._props : t._props.get(k)),
      set: (t, k, v) => { t._props.set(k, v); return true; },
    });
    this._listeners = new Map();
    this._animations = [];
    // An animation finishes on its own unless a test says otherwise.
    this._holdAnimations = false;
    // What a layout would have measured. A test sets it; nothing else
    // in this DOM has a size.
    this._height = 0;
    this.scrollHeight = 0;
  }
  /// `element.animate(frames, options)` — the whole of the runtime's
  /// motion engine goes through this.
  animate(frames, options) {
    const animation = new Animation_(this, frames, options);
    this._animations.push(animation);
    return animation;
  }
  getAnimations() { return this._animations.slice(); }
  getBoundingClientRect() {
    return { x: 0, y: 0, top: 0, left: 0, right: 0, bottom: this._height,
             width: 0, height: this._height };
  }
  /// What a published custom element does to put its render in place.
  replaceChildren(...nodes) {
    for (const child of this.childNodes.splice(0)) child.parentNode = null;
    for (const n of nodes) this.appendChild(n);
  }
  appendChild(n) {
    this.childNodes.push(...this._adopt(n));
    // A stylesheet linked here arrives at once, as a cached one would; a
    // test that wants to see the wait between link and load overrides this.
    if (n.tagName === "LINK" && typeof n.onload === "function") n.onload();
    return n;
  }
  get className() { return this.classList.toString(); }
  set className(v) {
    this.classList._set = new Set(String(v).split(/\s+/).filter(Boolean));
  }
  get id() { return this.getAttribute("id") || ""; }
  set id(v) { this.setAttribute("id", v); }
  setAttribute(k, v) {
    if (k === "class") { this.className = v; return; }
    this.attributes.set(k, String(v));
  }
  getAttribute(k) {
    if (k === "class") { const c = this.classList.toString(); return c === "" ? null : c; }
    return this.attributes.has(k) ? this.attributes.get(k) : null;
  }
  hasAttribute(k) { return this.attributes.has(k); }
  removeAttribute(k) { this.attributes.delete(k); }
  addEventListener(type, fn) {
    if (!this._listeners.has(type)) this._listeners.set(type, []);
    this._listeners.get(type).push(fn);
  }
  removeEventListener(type, fn) {
    const l = this._listeners.get(type) || [];
    this._listeners.set(type, l.filter((f) => f !== fn));
  }
  dispatchEvent(ev) {
    ev.target = ev.target || this;
    for (const fn of this._listeners.get(ev.type) || []) fn(ev);
    return true;
  }
  click() { this.dispatchEvent({ type: "click" }); }
  remove() { this.parentNode?.removeChild(this); }

  // <dialog>: the real element hides itself while closed, traps focus and fires
  // `close` on Escape. The shim models the observable part — the `open`
  // property, the attribute and the close event — so a test can tell whether
  // the runtime drove it correctly.
  get open() { return this.hasAttribute("open"); }
  set open(v) { if (v) this.setAttribute("open", ""); else this.removeAttribute("open"); }
  showModal() { this.setAttribute("open", ""); this.setAttribute("aria-modal", "true"); }
  show() { this.setAttribute("open", ""); }
  close(returnValue) {
    this.removeAttribute("open");
    this.removeAttribute("aria-modal");
    this.returnValue = returnValue;
    this.dispatchEvent({ type: "close", target: this });
  }
  focus() { this.ownerDocument_ = true; focused = this; }
  get textContent() { return this.childNodes.map((n) => n.textContent).join(""); }
  set textContent(v) { this.childNodes = []; this.appendChild(new TextNode(v)); }
  // Markup set as text is kept as it was set; the shim does not parse it.
  get innerHTML() { return this._html !== undefined ? this._html : this.textContent; }
  set innerHTML(v) { this.childNodes = []; this._html = v === "" ? undefined : v; }
  querySelector(sel) { return this.querySelectorAll(sel)[0] || null; }
  querySelectorAll(sel) {
    // A list of simple selectors: `tag`, `.class`, `#id`, `[attr]`,
    // `[attr="v"]`, and a tag with one attribute test (`a[href]`).
    const simple = sel.split(",").map((part) => {
      const m = /^([\w-]*)(?:\.([\w-]+))?(?:#([\w-]+))?(?:\[([\w-]+)(?:="(.*)")?\])?$/.exec(part.trim());
      return m ? { tag: m[1], cls: m[2], id: m[3], attr: m[4], value: m[5] } : { tag: part.trim() };
    });
    const matches = (c) =>
      simple.some((q) => {
        if (q.tag && c.tagName.toLowerCase() !== q.tag.toLowerCase()) return false;
        if (q.cls && !c.classList.contains(q.cls)) return false;
        if (q.id && c.getAttribute("id") !== q.id) return false;
        if (q.attr && q.value === undefined && !c.hasAttribute(q.attr)) return false;
        if (q.attr && q.value !== undefined && c.getAttribute(q.attr) !== q.value) return false;
        return true;
      });
    const out = [];
    const walk = (el) => {
      for (const c of el.children) {
        if (matches(c)) out.push(c);
        walk(c);
      }
    };
    walk(this);
    return out;
  }

  /// Every descendant element, for assertions that count or scan the tree.
  all() {
    const out = [];
    const walk = (el) => { for (const c of el.children) { out.push(c); walk(c); } };
    walk(this);
    return out;
  }
}

/// A fragment appends its children into the parent and empties itself, exactly
/// as the DOM does — the runtime branches on `instanceof DocumentFragment` to
/// know when to collect children before that happens.
class DocumentFragment extends Element {
  constructor() { super("#document-fragment"); }
}

let focused = null;

export function makeDom() {
  focused = null;
  const scrolls = [];
  const document = {
    get activeElement() { return focused; },
    createElement: (t) => new Element(t),
    createTextNode: (t) => new TextNode(t),
    createComment: () => new TextNode(""),
    createDocumentFragment: () => new DocumentFragment(),
    body: new Element("body"),
    head: new Element("head"),
    documentElement: new Element("html"),
    addEventListener() {},
    querySelector: (s) => document.body.querySelector(s),
    querySelectorAll: (s) => document.body.querySelectorAll(s),
    getElementById: (id) => document.body.querySelectorAll(`[id="${id}"]`)[0] || null,
  };
  // What the page listens for. A test fires one by calling `window._fire`,
  // which is how a write in another tab arrives.
  const listeners = new Map();
  const window = {
    document,
    addEventListener(type, fn) {
      if (!listeners.has(type)) listeners.set(type, []);
      listeners.get(type).push(fn);
    },
    removeEventListener(type, fn) {
      listeners.set(type, (listeners.get(type) || []).filter((f) => f !== fn));
    },
    _fire(type, event) {
      for (const fn of listeners.get(type) || []) fn(event);
    },
    location: { pathname: "/", search: "", hash: "" },
    // A push or replace moves the location, as the browser's does.
    history: {
      pushState(_s, _t, url) { window._setUrl(url); },
      replaceState(_s, _t, url) { window._setUrl(url); },
    },
    _setUrl(url) {
      if (url == null) return;
      const m = String(url).match(/^([^?#]*)(\?[^#]*)?(#.*)?$/);
      window.location.pathname = m[1] || "/";
      window.location.search = m[2] || "";
      window.location.hash = m[3] || "";
    },
    requestAnimationFrame: (fn) => fn(),
    queueMicrotask: (fn) => fn(),
    setTimeout: (fn) => fn(),
    // What `persist` writes to. Two areas, as a browser has, each a plain
    // map: a test seeds one to stand for a value an earlier visit left.
    localStorage: storageArea(),
    sessionStorage: storageArea(),
    // The design tokens the page would have. `window.tokens.set(name,
    // value)` is how a test says what the stylesheet says.
    tokens: new Map(),
    getComputedStyle: () => ({
      getPropertyValue: (name) => window.tokens.get(name) || "",
    }),
    matchMedia: () => ({ matches: false, addEventListener() {} }),
    scrollTo: (x, y) => scrolls.push([x, y]),
    scrolls,
  };
  // The custom-element registry, and the base class a published element
  // extends. Enough of it to build one, connect it, change an attribute
  // and take it out again — which is the whole of the contract.
  const defined = new Map();
  const customElements = {
    define: (tag, cls) => defined.set(tag, cls),
    get: (tag) => defined.get(tag),
  };
  /// Make a defined element, as `document.createElement(tag)` would in a
  /// browser that knows it.
  const makeCustom = (tag, attrs = {}) => {
    const Cls = defined.get(tag);
    if (!Cls) throw new Error(`no custom element ${tag}`);
    const el = new Cls(tag);
    for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
    el.isConnected = true;
    el.connectedCallback();
    return el;
  };

  // Everything watching for an element to be scrolled to. `seen(el)`
  // is the scroll.
  const watchers = [];
  class IntersectionObserver {
    constructor(fn) { this.fn = fn; this.targets = []; watchers.push(this); }
    observe(el) { this.targets.push(el); }
    disconnect() {
      this.targets = [];
      const at = watchers.indexOf(this);
      if (at >= 0) watchers.splice(at, 1);
    }
  }
  const seen = (el) => {
    for (const w of watchers.slice()) {
      if (w.targets.includes(el)) w.fn([{ target: el, isIntersecting: true }]);
    }
  };
  return {
    window, document, Element, TextNode, DocumentFragment, Node: Node_,
    HTMLElement: Element, customElements, makeCustom,
    CustomEvent: class CustomEvent {
      constructor(type, init) {
        this.type = type;
        this.detail = init && init.detail;
        this.bubbles = !!(init && init.bubbles);
      }
    },
    Animation: Animation_, IntersectionObserver, seen,
    getComputedStyle: window.getComputedStyle,
  };
}
