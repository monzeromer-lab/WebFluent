  // ─── Taking over a pre-rendered page ────────────────
  //
  // A static build paints every page as HTML, and the reader sees it before
  // any script has run. The script then draws the same page again, with its
  // handlers and effects bound to the nodes it made. Replacing the painted
  // page with those nodes worked, but it threw away what the reader was
  // already looking at: the browser painted the page a second time, the
  // second paint counted as the page's Largest Contentful Paint (a paragraph
  // drawn again once its web font has arrived is larger than the one first
  // painted), and the reader's scroll, focus and selection went with the
  // old nodes.
  //
  // So the script draws into a detached copy of the page's root, and then
  // takes over the painted page in place. An element it drew that nothing
  // holds on to is matched to the painted element in the same place, and
  // the painted one is kept — its attributes brought up to date, never
  // moved. An element something does hold — a handler, an effect, a `ref`,
  // a widget, an item of a list — is put where the painted one stood,
  // inside the painted parent, so nothing above it moves. A node that moves
  // is painted again; a node that stays is not.
  //
  // What holds a node says so with `live(node)` (core); a listener added
  // while the page is drawn says so by itself, and the compiler marks the
  // elements its own closures reach. A container that a `when`, `each`,
  // `match` or `slot` draws into is not held: those insert beside a marker,
  // and the marker is moved into the painted parent. The router reaches its
  // container through `inPage()`, which names the painted node once it is
  // kept.

  /// Where the page is drawn. When `host` holds a page painted before this
  /// script ran, a detached copy of it — the painted page is taken over
  /// once it is drawn; otherwise `host` itself, emptied.
  function hydrating(host) {
    if (!_ssgMode || !host || _hyd || !_hasElements(host)) {
      if (host) host.innerHTML = "";
      return host;
    }
    return _begin(host);
  }

  function hydrate(renderFn, container) {
    if (_hyd || !container || !_hasElements(container)) {
      mount(renderFn, container);
      return;
    }
    const shell = _begin(container);
    themeSignal();
    _newPage();
    const el = renderFn();
    if (el instanceof Node) shell.appendChild(el);
    const path = typeof location !== "undefined" ? location.pathname : "/";
    drawn({ route: path, params: {} });
  }

  function _hasElements(node) {
    for (const n of node.childNodes) if (n.nodeType === 1) return true;
    return false;
  }

  function _begin(host) {
    const shell = document.createElement(host.tagName || "div");
    _hyd = { host, shell, live: new WeakSet(), placed: [], restore: _watchListeners() };
    return shell;
  }

  function _end() {
    if (!_hyd) return;
    _hyd.restore();
    _hyd = null;
  }

  // A listener added to a node while the page is drawn holds that node.
  // The prototype that owns `addEventListener` is patched for as long as
  // the drawing lasts.
  function _watchListeners() {
    let proto = Object.getPrototypeOf(document.createElement("div"));
    while (proto && !Object.prototype.hasOwnProperty.call(proto, "addEventListener")) {
      proto = Object.getPrototypeOf(proto);
    }
    if (!proto) return () => {};
    const original = proto.addEventListener;
    proto.addEventListener = function (type, fn, options) {
      if (_hyd && this && this.nodeType) _hyd.live.add(this);
      return original.call(this, type, fn, options);
    };
    return () => { proto.addEventListener = original; };
  }

  /// `fn(node)` with the node the reader sees in `node`'s place: once the
  /// page is taken over, the painted one when it was kept; outside a
  /// takeover, `node`, at once.
  function placedAs(node, fn) {
    if (_hyd) _hyd.placed.push([node, fn]);
    else fn(node);
  }

  // First the plan, then the changes: matching reads both trees and writes
  // neither, so a page that cannot be matched is drawn again whole, as it
  // always was.
  function takeOver() {
    const session = _hyd;
    if (!session) return;
    const { host, shell } = session;
    let ops = [];
    try {
      _plan(host, shell, session.live, ops);
    } catch (e) {
      console.error("WF: the painted page could not be taken over; it is drawn again", e);
      ops = null;
    }
    const expected = ops && devMode() ? _describe(shell) : null;
    if (ops) {
      for (const op of ops) op();
      _tookOver = true;
    } else {
      const y = typeof window !== "undefined" ? window.scrollY || 0 : 0;
      host.innerHTML = "";
      while (shell.firstChild) host.appendChild(shell.firstChild);
      if (y > 0 && typeof window.scrollTo === "function") window.scrollTo(0, y);
    }
    _end();
    for (const [node, fn] of session.placed) {
      try { fn(inPage(node)); } catch (e) { console.error(e); }
    }
    // Under `wf serve`, say so when what the reader sees is not what was
    // drawn: that is a bug in the matching, never in the page.
    if (expected !== null) {
      const got = _describe(host);
      if (got !== expected) {
        console.warn("WF: the page taken over differs from the page drawn — a WebFluent bug; please report it", { expected, got });
      }
    }
  }

  // The nodes that take part in matching: elements and text. A comment
  // the painted page carries stays where it is.
  function _kids(parent) {
    const out = [];
    for (const n of parent.childNodes) if (n.nodeType === 1 || n.nodeType === 3) out.push(n);
    return out;
  }

  function _sameTag(painted, drawn) {
    return !!painted && painted.nodeType === 1 &&
      String(painted.tagName).toUpperCase() === String(drawn.tagName).toUpperCase();
  }

  // Whether a painted element is the one drawn in its place: the same tag
  // and at least half the same classes. Tag alone is not enough — a
  // drawer's scrim and the article beside it are both a `div`.
  function _fits(painted, drawn) {
    if (!_sameTag(painted, drawn)) return false;
    const mine = _classSet(painted.getAttribute("class")).split(" ").filter(Boolean);
    const theirs = _classSet(drawn.getAttribute("class")).split(" ").filter(Boolean);
    if (!mine.length && !theirs.length) return true;
    const shared = theirs.filter((c) => mine.includes(c)).length;
    return shared * 2 >= Math.max(mine.length, theirs.length);
  }

  function _remove(n) {
    return () => { if (n.parentNode) n.parentNode.removeChild(n); };
  }

  function _plan(painted, drawn, held, ops) {
    const have = _kids(painted);
    let i = 0;
    // What is put in goes before the painted node that comes next — or at
    // the end, when none does. That node is still in place when it runs:
    // nothing painted is taken out before what is planned ahead of it.
    const next = () => (i < have.length ? have[i] : null);
    const kids = [...drawn.childNodes];
    for (let k = 0; k < kids.length; k++) {
      const node = kids[k];
      if (node.nodeType === 3) {
        // A run of text, compared whole: one painted node often holds what
        // was drawn as several (`"Hello, "` and a name).
        const run = [node];
        while (k + 1 < kids.length && kids[k + 1].nodeType === 3) run.push(kids[++k]);
        const theirs = [];
        while (i < have.length && have[i].nodeType === 3) theirs.push(have[i++]);
        if (run.length === 1 && theirs.length === 1 && !held.has(node)) {
          const old = theirs[0];
          const text = node.textContent;
          ops.push(() => {
            if (old.textContent !== text) old.textContent = text;
            node.__wfAt = old;
          });
        } else {
          // Text a signal keeps current is the drawn node. Swapping a text
          // node leaves the element around it where it was.
          const ref = next();
          for (const t of theirs) ops.push(_remove(t));
          for (const t of run) ops.push(() => { painted.insertBefore(t, ref); });
        }
        continue;
      }
      if (node.nodeType !== 1) {
        // A marker — `if`, `for`, `match`, a slot — goes where it was
        // drawn: what it puts in later lands beside it, in the painted
        // parent.
        const ref = next();
        ops.push(() => { painted.insertBefore(node, ref); });
        continue;
      }
      // Painted text where the drawing has an element goes.
      while (i < have.length && have[i].nodeType === 3) ops.push(_remove(have[i++]));
      // The painted element that is this one: the next, or one a little
      // further on past painted ones the drawing does not have. When the
      // next painted one is what the drawing has a little later, this drawn
      // element is one the painted page does not have.
      let j = -1;
      for (let a = i; a < Math.min(have.length, i + 4); a++) {
        if (_fits(have[a], node)) { j = a; break; }
      }
      if (j > i) {
        const soon = kids.slice(k + 1, k + 4).filter((n) => n.nodeType === 1);
        if (soon.some((n) => _fits(have[i], n))) j = -1;
        else while (i < j) ops.push(_remove(have[i++]));
      }
      if (j === i) {
        const old = have[i++];
        if (held.has(node)) {
          // Something holds the drawn element: it stands where the painted
          // one stood, and nothing around it moves.
          ops.push(() => {
            if (!old.parentNode) return;
            old.parentNode.insertBefore(node, old);
            old.parentNode.removeChild(old);
          });
        } else {
          ops.push(() => { _syncAttributes(old, node); node.__wfAt = old; });
          _plan(old, node, held, ops);
        }
      } else {
        // An element the painted page does not have.
        const ref = next();
        ops.push(() => { painted.insertBefore(node, ref); });
      }
    }
    // What is left of the painted page was not drawn.
    for (; i < have.length; i++) ops.push(_remove(have[i]));
  }

  function _names(el) {
    if (typeof el.getAttributeNames === "function") return el.getAttributeNames();
    return [...el.attributes].map((a) => (Array.isArray(a) ? a[0] : a.name));
  }

  function _classSet(v) {
    return String(v || "").split(/\s+/).filter(Boolean).sort().join(" ");
  }

  // The painted element takes the drawn one's attributes. A class list in
  // another order is the same class list, and an attribute that already
  // has its value is not set again — an image's `src` set again is
  // fetched again. Inline style goes through the style object, as it does
  // everywhere in this runtime: a policy without 'unsafe-inline' refuses a
  // `style` attribute set as markup, never one set as properties.
  function _syncAttributes(painted, drawn) {
    const want = new Set(_names(drawn));
    for (const name of want) {
      const value = drawn.getAttribute(name);
      const now = painted.getAttribute(name);
      if (now === value) continue;
      if (name === "class" && _classSet(now) === _classSet(value)) continue;
      if (name === "style") painted.style.cssText = drawn.style.cssText;
      else painted.setAttribute(name, value);
    }
    for (const name of _names(painted)) {
      if (!want.has(name)) painted.removeAttribute(name);
    }
  }

  // What a tree shows, for the check `wf serve` makes after a takeover.
  function _describe(root) {
    const out = [];
    const walk = (n) => {
      for (const c of n.childNodes) {
        if (c.nodeType === 3) {
          const t = c.textContent.trim();
          if (t) out.push(JSON.stringify(t));
        } else if (c.nodeType === 1) {
          const attrs = _names(c)
            .map((a) => a + "=" + (a === "class" ? _classSet(c.getAttribute(a)) : c.getAttribute(a)))
            .sort();
          out.push("<" + String(c.tagName).toLowerCase() + (attrs.length ? " " + attrs.join(" ") : "") + ">");
          walk(c);
          out.push("</>");
        }
      }
    };
    walk(root);
    return out.join("\n");
  }
