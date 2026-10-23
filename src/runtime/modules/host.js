  // ─── Somebody else's code ────────────────────────────
  //
  // A chart, a map, an editor: a library that wants a node and gives back
  // a handle. The pattern people write by hand is a `ref`, an `effect` and
  // a `cleanup`, and the part they forget is the cleanup — so the map is
  // still listening after the route changed.
  //
  // `Host` makes the three one thing. The mount runs once; the update runs
  // again whenever the state it reads changes; the cleanup is tied to the
  // scope that owns the element, so it cannot be forgotten.

  /// Hand `node` to `mount`, keep it in step with `update`, and give it
  /// back with `cleanup` when what owns it leaves.
  ///
  /// `attach` rather than `host`, because `host` is a `Url`'s.
  function attach(node, mount, update, cleanup) {
    // A component's call hands over what it rendered, which may be a
    // fragment: the lifetime belongs to its root element.
    if (node && (node.nodeType === 11 || node.tagName === "#DOCUMENT-FRAGMENT")) {
      node = [...node.childNodes].find((n) => n.nodeType === 1);
      if (!node) return undefined;
    }
    // Somebody else's code holds the node from here on (hydrate).
    live(node);
    // Inside a portal's block the lifetime waits for the block to be drawn,
    // so a mount meets its node where the portal put it — in the page,
    // when the node it draws into is. It runs in the scope it was asked
    // for in, which a branch or an item of the block may have opened.
    if (_portalHeld) {
      const scope = currentScope;
      const tracking = currentEffect;
      _portalHeld.push(() => {
        const [wasScope, wasEffect] = [currentScope, currentEffect];
        currentScope = scope;
        currentEffect = tracking;
        try { _attachNow(node, mount, update, cleanup); } finally {
          currentScope = wasScope;
          currentEffect = wasEffect;
        }
      });
      return undefined;
    }
    return _attachNow(node, mount, update, cleanup);
  }

  function _attachNow(node, mount, update, cleanup) {
    let handle;
    try {
      handle = typeof mount === "function" ? mount(node) : undefined;
    } catch (e) {
      // A library that throws on mount takes itself out; the page stays.
      console.error("mount failed", e);
      return undefined;
    }
    if (typeof update === "function") {
      // The first run is the mount's: an update that ran immediately would
      // be a second call before the library had settled.
      let first = true;
      effect(() => {
        if (first) {
          // Read what the update reads, so the effect subscribes to it,
          // without acting on it yet.
          first = false;
          try { update(handle); } catch (e) { console.error("update failed", e); }
          return;
        }
        try { update(handle); } catch (e) { console.error("update failed", e); }
      });
    }
    if (typeof cleanup === "function") {
      onCleanup(() => {
        try { cleanup(handle); } catch (e) { console.error("cleanup failed", e); }
      });
    }
    return handle;
  }

  // ─── Somebody else's node ────────────────────────────
  //
  // An editor's tooltip, a map's popup, a panel a library placed: a node
  // that somebody else's code made, owns and positions, with nothing in it
  // until the page draws there. `Portal(into: node) { … }` draws its block
  // into a box of its own and puts the box in that node. The box is the
  // page's; the node it sits in is not. So the box follows `into:` — moved,
  // never drawn again, when it names another node; kept aside with its
  // nodes and state when it names none; gone with the scope it was written
  // in — and what the block holds belongs to where it is written, as the
  // rest of that scope does.

  /// The lifetimes held back while a portal's block is drawn, or null.
  let _portalHeld = null;

  /// Keep `box` in the node `read` names: an element, or a `ref:` handle's
  /// element, takes it (unless it is already there); null or undefined
  /// parks it, its nodes and their state kept; the scope's end takes it
  /// away. Returns what runs the lifetimes the block asked for while it
  /// was drawn, once the block is whole and in place.
  function portal(box, read) {
    let alive = true;
    let named;
    // A takeover of a pre-rendered page leaves a box where it is when the
    // node it was put in is a painted one (hydrate).
    box.__wfPortal = true;
    const place = (target) => {
      const node = target && target.el !== undefined ? target.el : target;
      if (node && (node.nodeType === 1 || node.nodeType === 11) && !box.contains(node)) {
        if (box.parentNode !== node) node.appendChild(box);
      } else if (box.parentNode) {
        box.remove();
      }
    };
    effect(() => {
      let target = null;
      try { target = read(); } catch (e) { console.error("WF: a Portal's into: failed", e); }
      named = target;
      place(target);
      // A handle whose element comes later in the drawing is looked at
      // again once the drawing is done: a handle is not a state.
      if (target && target.el === null) {
        queueMicrotask(() => { if (alive && named === target) place(target); });
      }
    });
    onCleanup(() => {
      alive = false;
      box.remove();
    });
    const outer = _portalHeld;
    const held = [];
    _portalHeld = held;
    return () => {
      _portalHeld = outer;
      for (const run of held.splice(0)) run();
    };
  }
