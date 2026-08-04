//! The smallest DOM that `runtime.js` actually uses.
//!
//! There was no JavaScript harness in this repository, so every change to
//! `runtime.js` was unverifiable — the reason the runtime accumulated bugs nobody
//! could catch. This is not a browser: it implements exactly the surface the
//! runtime touches (element creation, children, attributes, classes, text,
//! listeners, dispatch), which is enough to assert what the runtime DOES.

class ClassList {
  constructor(el) { this.el = el; this._set = new Set(); }
  add(...names) { for (const n of names) if (n) this._set.add(n); }
  remove(...names) { for (const n of names) this._set.delete(n); }
  contains(n) { return this._set.has(n); }
  toString() { return [...this._set].join(" "); }
}

class Node_ {
  constructor() { this.childNodes = []; this.parentNode = null; }
  get children() { return this.childNodes.filter((n) => n.nodeType === 1); }
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
  }
  appendChild(n) { n.parentNode = this; this.childNodes.push(n); return n; }
  removeChild(n) { this.childNodes = this.childNodes.filter((c) => c !== n); return n; }
  insertBefore(n, ref) {
    const i = this.childNodes.indexOf(ref);
    n.parentNode = this;
    if (i < 0) this.childNodes.push(n); else this.childNodes.splice(i, 0, n);
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
  focus() { this.ownerDocument_ = true; }
  get textContent() { return this.childNodes.map((n) => n.textContent).join(""); }
  set textContent(v) { this.childNodes = []; this.appendChild(new TextNode(v)); }
  get innerHTML() { return this.textContent; }
  set innerHTML(v) { if (v === "") this.childNodes = []; }
  querySelector(sel) { return this.querySelectorAll(sel)[0] || null; }
  querySelectorAll(sel) {
    const attr = /^\[([\w-]+)="(.*)"\]$/.exec(sel);
    const cls = /^\.([\w-]+)$/.exec(sel);
    const id = /^#([\w-]+)$/.exec(sel);
    const matches = (c) => {
      if (attr) return c.getAttribute(attr[1]) === attr[2];
      if (cls) return c.classList.contains(cls[1]);
      if (id) return c.getAttribute("id") === id[1];
      return c.tagName.toLowerCase() === sel.toLowerCase();
    };
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

export function makeDom() {
  const document = {
    createElement: (t) => new Element(t),
    createTextNode: (t) => new TextNode(t),
    createComment: () => new TextNode(""),
    createDocumentFragment: () => new DocumentFragment(),
    body: new Element("body"),
    documentElement: new Element("html"),
    addEventListener() {},
    querySelector: (s) => document.body.querySelector(s),
    querySelectorAll: (s) => document.body.querySelectorAll(s),
    getElementById: (id) => document.body.querySelectorAll(`[id="${id}"]`)[0] || null,
  };
  const window = {
    document,
    addEventListener() {},
    removeEventListener() {},
    location: { pathname: "/", search: "", hash: "" },
    history: { pushState() {}, replaceState() {} },
    requestAnimationFrame: (fn) => fn(),
    queueMicrotask: (fn) => fn(),
    setTimeout: (fn) => fn(),
    getComputedStyle: () => ({}),
    matchMedia: () => ({ matches: false, addEventListener() {} }),
  };
  return { window, document, Element, TextNode, DocumentFragment, Node: Node_ };
}
