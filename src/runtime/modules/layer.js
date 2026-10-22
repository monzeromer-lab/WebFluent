  // ─── Layers: popovers, menus and tooltips above everything ─────────
  //
  // A layer is drawn in the browser's top layer (the Popover API), so no
  // ancestor's `overflow` or stacking context can clip or cover it, and is
  // placed against an anchor — an element, or a point such as where the
  // pointer asked for a context menu. It flips to the other side when there
  // is no room, slides along its side to stay on screen, and follows the
  // anchor while the page scrolls or resizes. The open layers are one stack:
  // `Escape` and a press outside close the top one only, and focus goes back
  // to where it was before that layer opened.

  // The margin a layer keeps from the edges of the viewport.
  const LAYER_MARGIN = 8;

  // Where a box of `size` goes against `anchor` (a rect) on `placement`
  // (`bottom-start`, `top`, `right-end`, …), `offset` away from it, inside
  // `viewport` — the side flipped when the box does not fit there and does
  // on the other, then slid along its side to stay in. Pure, so it can be
  // tested without a layout.
  function placeAt(anchor, size, viewport, placement, offset) {
    // `bottom-start`, or `bottomStart` as a `.case` is written.
    const spelled = String(placement || "bottom-start").replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase();
    const [side0, align = "center"] = spelled.split("-");
    const opposite = { top: "bottom", bottom: "top", left: "right", right: "left" };
    const room = {
      top: anchor.top - offset - LAYER_MARGIN,
      bottom: viewport.height - anchor.bottom - offset - LAYER_MARGIN,
      left: anchor.left - offset - LAYER_MARGIN,
      right: viewport.width - anchor.right - offset - LAYER_MARGIN,
    };
    const needs = (side) => (side === "top" || side === "bottom" ? size.height : size.width);
    let side = side0 in opposite ? side0 : "bottom";
    if (needs(side) > room[side] && needs(opposite[side]) <= room[opposite[side]]) {
      side = opposite[side];
    }
    let x;
    let y;
    if (side === "top" || side === "bottom") {
      y = side === "bottom" ? anchor.bottom + offset : anchor.top - offset - size.height;
      x = align === "start" ? anchor.left
        : align === "end" ? anchor.right - size.width
        : anchor.left + (anchor.right - anchor.left - size.width) / 2;
      x = Math.min(Math.max(x, LAYER_MARGIN), viewport.width - size.width - LAYER_MARGIN);
    } else {
      x = side === "right" ? anchor.right + offset : anchor.left - offset - size.width;
      y = align === "start" ? anchor.top
        : align === "end" ? anchor.bottom - size.height
        : anchor.top + (anchor.bottom - anchor.top - size.height) / 2;
      y = Math.min(Math.max(y, LAYER_MARGIN), viewport.height - size.height - LAYER_MARGIN);
    }
    return { x: Math.round(Math.max(x, LAYER_MARGIN)), y: Math.round(Math.max(y, LAYER_MARGIN)), placement: `${side}-${align}` };
  }

  // The open layers, the newest last.
  const _layers = [];

  // The rect of what a layer is anchored to: an element (or a `ref:` handle
  // on one), or a point `{ x, y }`.
  function _anchorRect(anchor) {
    const target = anchor && anchor.el !== undefined ? anchor.el : anchor;
    if (target && typeof target.getBoundingClientRect === "function") {
      return target.getBoundingClientRect();
    }
    if (target && typeof target.x === "number" && typeof target.y === "number") {
      return { left: target.x, right: target.x, top: target.y, bottom: target.y };
    }
    return null;
  }

  // Where focus goes in a layer that opens: what says `autofocus`, else a
  // menu's first item that is not disabled.
  function _firstStop(root) {
    for (const child of root.children || []) {
      const role = (child.getAttribute && child.getAttribute("role")) || "";
      const usable = child.getAttribute && child.getAttribute("aria-disabled") !== "true";
      if (child.hasAttribute && child.hasAttribute("autofocus")) return child;
      if (role.startsWith("menuitem") && usable) return child;
      const deeper = _firstStop(child);
      if (deeper) return deeper;
    }
    return null;
  }

  function _closeTop(reason) {
    const top = _layers[_layers.length - 1];
    if (!top) return false;
    top.close(reason);
    return true;
  }

  let _layerKeysHeard = false;
  function _hearLayerKeys() {
    if (_layerKeysHeard || typeof document === "undefined") return;
    _layerKeysHeard = true;
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && _layers.length && _closeTop("escape")) {
        if (e.preventDefault) e.preventDefault();
        if (e.stopPropagation) e.stopPropagation();
      }
    }, true);
    // A press outside closes every layer it is outside of, from the top
    // down: a press on a menu closes its submenu, and a press on the page
    // closes both.
    document.addEventListener("pointerdown", (e) => {
      const inside = (node) => node && typeof node.contains === "function" && node.contains(e.target);
      for (const layer of _layers.slice().reverse()) {
        if (!layer.dismiss) continue;
        const anchor = layer.anchor && layer.anchor.el !== undefined ? layer.anchor.el : layer.anchor;
        if (inside(layer.el) || inside(anchor)) break;
        layer.close("outside");
      }
    }, true);
  }

  // Show `el` as a layer while `read()` is true. `write(false)` is called
  // when the layer is dismissed (Escape, a press outside), so the state that
  // opened it follows; `onClose(reason)` hears it too. `anchor` is read on
  // every placement, so it may follow state.
  function layer(el, read, write, options) {
    live(el);
    const opts = options || {};
    const offset = opts.offset === undefined ? 6 : opts.offset;
    const anchorOf = typeof opts.anchor === "function" ? opts.anchor : () => opts.anchor;
    if (write === undefined && read && typeof read.set === "function") {
      const signal = read;
      write = (v) => signal.set(v);
    }
    const top = typeof el.showPopover === "function";
    if (top) el.setAttribute("popover", "manual");
    el.classList.add("wf-layer");
    _hearLayerKeys();
    let entry = null;
    let frame = 0;
    const place = () => {
      frame = 0;
      if (!entry) return;
      const rect = _anchorRect(anchorOf());
      if (!rect) return;
      const size = { width: el.offsetWidth || 0, height: el.offsetHeight || 0 };
      const viewport = { width: window.innerWidth || 0, height: window.innerHeight || 0 };
      const at = placeAt(rect, size, viewport, opts.placement, offset);
      el.style.left = `${at.x}px`;
      el.style.top = `${at.y}px`;
      el.setAttribute("data-placement", at.placement);
    };
    const soon = () => {
      if (frame) return;
      if (typeof requestAnimationFrame === "function") frame = requestAnimationFrame(place);
      else place();
    };
    const open = () => {
      if (entry) return;
      entry = {
        el,
        anchor: anchorOf(),
        dismiss: opts.dismiss !== false,
        before: typeof document !== "undefined" ? document.activeElement : null,
        close: (reason) => {
          if (write) write(false);
          else shut();
          if (opts.onClose) opts.onClose(reason);
        },
      };
      _layers.push(entry);
      if (top) el.showPopover();
      else el.removeAttribute("hidden");
      el.setAttribute("data-open", "");
      place();
      window.addEventListener("scroll", soon, true);
      window.addEventListener("resize", soon);
      if (opts.focus !== false && typeof el.focus === "function") {
        (_firstStop(el) || el).focus();
      }
    };
    const shut = () => {
      if (!entry) return;
      const was = entry;
      entry = null;
      const at = _layers.indexOf(was);
      if (at >= 0) _layers.splice(at, 1);
      if (top) el.hidePopover();
      else el.setAttribute("hidden", "");
      el.removeAttribute("data-open");
      window.removeEventListener("scroll", soon, true);
      window.removeEventListener("resize", soon);
      // Focus goes back to where the reader was, if it is still inside.
      if (typeof document !== "undefined" && el.contains && el.contains(document.activeElement)) {
        const back = was.before && was.before.focus ? was.before : null;
        if (back) back.focus();
      }
    };
    if (!top) el.setAttribute("hidden", "");
    // A layer open from the first paint is asked to open before it is in
    // the document, where the top layer cannot take it: it opens on the
    // first frame it is there, unless it was closed in between.
    let waiting = 0;
    const later = (fn) => (typeof requestAnimationFrame === "function" ? requestAnimationFrame(fn) : setTimeout(fn, 0));
    const openWhenThere = () => {
      if (entry || waiting) return;
      if (el.isConnected !== false) return open();
      waiting = later(() => {
        waiting = 0;
        if (read() && !entry) openWhenThere();
      });
    };
    // An anchor that follows state moves an open layer with it.
    effect(() => {
      if (!read()) shut();
      else if (entry) place();
      else openWhenThere();
    });
    onCleanup(() => {
      shut();
      if (waiting && typeof cancelAnimationFrame === "function") cancelAnimationFrame(waiting);
      waiting = 0;
      if (frame && typeof cancelAnimationFrame === "function") cancelAnimationFrame(frame);
    });
    return { place, open, close: shut };
  }

  // An id no other element of the page has: a menu's list, named by its
  // button's `aria-controls`, once for each menu drawn — a `for` draws many.
  let _uids = 0;
  function uid(prefix) {
    _uids += 1;
    return `${prefix}-${_uids}`;
  }

  // ─── Menus ─────────────────────────────────────────────────────────
  //
  // A menu is a `role="menu"` list in a layer: the arrow keys move between
  // its items (Home and End jump to the ends), Enter and Space choose,
  // Escape closes it — a submenu first — and focus goes back to where it
  // was; Tab leaves and closes. A disabled item is skipped and cannot be
  // chosen. An item with a submenu opens it on ArrowRight, Enter or a
  // hover, and ArrowLeft goes back.

  // What each menu list knows: how to close the whole menu, and its
  // submenus' items, so a hover elsewhere closes them.
  const _menus = new WeakMap();

  function _menuItems(list) {
    return Array.from(list.children).filter((el) => {
      const cls = el.className || "";
      if (cls.includes("divider") || cls.includes("separator")) {
        el.setAttribute("role", "separator");
        return false;
      }
      if (!el.hasAttribute("role")) el.setAttribute("role", "menuitem");
      if (!el.hasAttribute("tabindex")) el.setAttribute("tabindex", "-1");
      return true;
    });
  }

  const _usable = (item) => item.getAttribute("aria-disabled") !== "true";

  function _focusItem(list, i) {
    const all = _menuItems(list).filter(_usable);
    if (!all.length) return;
    const target = all[((i % all.length) + all.length) % all.length];
    if (target.focus) target.focus();
  }

  // The keys, the choosing and the hovering of one menu list. `close`
  // closes the whole menu; `back` (a submenu's) closes just this list.
  function _menuList(list, read, close, back) {
    live(list);
    const state = { close, read, subs: [] };
    _menus.set(list, state);
    const itemOf = (node) => _menuItems(list).find((item) => item === node || (item.contains && item.contains(node)));
    list.addEventListener("keydown", (e) => {
      const all = _menuItems(list).filter(_usable);
      const current = all.indexOf(document.activeElement);
      let consumed = true;
      if (e.key === "ArrowDown") _focusItem(list, current + 1);
      else if (e.key === "ArrowUp") _focusItem(list, current - 1);
      else if (e.key === "Home") _focusItem(list, 0);
      else if (e.key === "End") _focusItem(list, -1);
      else if (e.key === "ArrowLeft" && back) back();
      else if ((e.key === "Enter" || e.key === " ") && current >= 0) all[current].click();
      else if (e.key === "Tab") { close("tab"); consumed = false; }
      else consumed = false;
      if (consumed) {
        if (e.preventDefault) e.preventDefault();
        // A submenu's list is inside its parent's: the parent must not
        // move too.
        if (e.stopPropagation) e.stopPropagation();
      }
    });
    // A disabled item hears nothing: this runs before the item's own
    // handler, on the way down.
    list.addEventListener("click", (e) => {
      const item = itemOf(e.target);
      if (item && !_usable(item)) {
        e.stopPropagation();
        if (e.preventDefault) e.preventDefault();
      }
    }, true);
    // Choosing an item closes the menu. An item with a submenu opens it
    // instead; what is chosen in a submenu has been heard there already.
    list.addEventListener("click", (e) => {
      const item = itemOf(e.target);
      if (!item || item.getAttribute("aria-haspopup") === "menu") return;
      close("choose");
    });
    list.addEventListener("pointerover", (e) => {
      const item = itemOf(e.target);
      if (!item) return;
      for (const sub of state.subs) if (sub.item !== item) sub.close();
    });
    _menuItems(list);
    return state;
  }

  // A menu: `list` in a layer while `read()` is true, opened by `trigger`
  // (a button, or null when state opens it — a context menu). `options`
  // are the layer's: `anchor` (the trigger when there is one),
  // `placement`, `offset`, `onClose`.
  function menu(root, trigger, list, read, write, options) {
    live(root, trigger, list);
    if (write === undefined && read && typeof read.set === "function") {
      const signal = read;
      write = (v) => signal.set(v);
    }
    const opts = Object.assign({ placement: "bottom-start", offset: 4 }, options || {});
    if (trigger && opts.anchor === undefined) opts.anchor = trigger;
    // A pointer that opens a menu keeps focus on its button; the keyboard
    // puts it on the first item (below), and a menu that state opens puts
    // it there too.
    if (trigger) opts.focus = false;
    let handle = null;
    const close = (reason) => {
      // A menu's button is where its reader goes back to.
      if (trigger && trigger.focus && typeof document !== "undefined" && list.contains(document.activeElement)) {
        trigger.focus();
      }
      if (write) write(false);
      else if (handle) handle.close();
      if (opts.onClose) opts.onClose(reason);
    };
    _menuList(list, read, close, null);
    handle = layer(list, read, write, opts);
    if (trigger) {
      trigger.addEventListener("keydown", (e) => {
        if (e.key === "ArrowDown" || e.key === "ArrowUp" || e.key === "Enter" || e.key === " ") {
          if (e.preventDefault) e.preventDefault();
          if (write) write(true);
          setTimeout(() => _focusItem(list, e.key === "ArrowUp" ? -1 : 0), 0);
        }
      });
    }
    return handle;
  }

  // A submenu: `list` beside `item` in the menu `parent`, open while
  // its parent is and it was asked for.
  function submenu(item, list, parent) {
    live(item, list);
    const owner = _menus.get(parent);
    const shown = signal(false);
    let asked = false;
    const set = (v) => {
      asked = v;
      shown.set(v);
      item.setAttribute("aria-expanded", v ? "true" : "false");
    };
    item.setAttribute("aria-haspopup", "menu");
    item.setAttribute("aria-expanded", "false");
    const close = (reason) => {
      set(false);
      if (owner) owner.close(reason);
    };
    // Open while it was asked for and its menu is; it closes with its menu.
    const open = () => shown() && (!owner || owner.read());
    effect(() => {
      if (owner && !owner.read() && asked) set(false);
    });
    _menuList(list, open, close, () => {
      set(false);
      if (item.focus) item.focus();
    });
    layer(list, open, set, { anchor: item, placement: "right-start", offset: 2, focus: false });
    if (owner) owner.subs.push({ item, close: () => set(false) });
    const show = (focus) => {
      set(true);
      if (focus) setTimeout(() => _focusItem(list, 0), 0);
    };
    let timer = 0;
    item.addEventListener("pointerenter", () => {
      clearTimeout(timer);
      timer = setTimeout(() => show(false), 120);
    });
    item.addEventListener("pointerleave", () => clearTimeout(timer));
    item.addEventListener("click", (e) => {
      if (list.contains(e.target)) return;
      show(true);
    });
    item.addEventListener("keydown", (e) => {
      if (list.contains(e.target)) return;
      if (e.key === "ArrowRight" || e.key === "Enter" || e.key === " ") {
        if (e.preventDefault) e.preventDefault();
        if (e.stopPropagation) e.stopPropagation();
        show(true);
      }
    });
  }

  // ─── Tooltips ──────────────────────────────────────────────────────
  //
  // A tooltip names what its child does — and, on the desktop, the
  // shortcut that does it. It is a layer too, so no `overflow` clips it:
  // it shows after `delay` milliseconds of hover, at once on keyboard
  // focus, never takes focus, and goes on Escape, on leaving, and on a
  // press.

  function tooltip(root, tip, options) {
    live(root, tip);
    if (!root || !tip) return;
    const opts = options || {};
    const delay = opts.delay === undefined ? 500 : opts.delay;
    const focusable = root.querySelector(
      "button, a[href], input, select, textarea, summary, [tabindex]",
    );
    const trigger = focusable || root;
    if (!focusable && !root.hasAttribute("tabindex")) root.setAttribute("tabindex", "0");
    const existing = trigger.getAttribute("aria-describedby");
    trigger.setAttribute("aria-describedby", existing ? `${existing} ${tip.id}` : tip.id);
    const shown = signal(false);
    layer(tip, shown, null, {
      anchor: trigger,
      placement: opts.placement || "top",
      offset: opts.offset === undefined ? 6 : opts.offset,
      focus: false,
      dismiss: false,
    });
    let timer = 0;
    const show = (wait) => {
      clearTimeout(timer);
      if (wait) timer = setTimeout(() => shown.set(true), wait);
      else shown.set(true);
    };
    const hide = () => {
      clearTimeout(timer);
      shown.set(false);
    };
    root.addEventListener("pointerenter", () => show(delay));
    root.addEventListener("pointerleave", hide);
    root.addEventListener("pointerdown", hide);
    root.addEventListener("focusin", (e) => {
      // Focus a pointer gave is not a reason to show it twice.
      if (e.target && e.target.matches && !e.target.matches(":focus-visible")) return;
      show(0);
    });
    root.addEventListener("focusout", (e) => {
      if (!root.contains(e.relatedTarget)) hide();
    });
    root.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && shown()) {
        hide();
        if (e.stopPropagation) e.stopPropagation();
      }
    });
    onCleanup(() => clearTimeout(timer));
  }

  // ─── A focus trap ──────────────────────────────────────────────────
  //
  // `trapFocus: open` on any element: while `read()` holds, focus moves in
  // (to what says `autofocus`, else the first thing that takes focus), and
  // Tab and Shift+Tab go round inside it; when it stops holding, or the
  // element leaves, focus goes back to where it was. `Modal` and `Dialog`
  // have the browser's own.

  const FOCUSABLE = "a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), summary, iframe, [contenteditable=\"true\"], [tabindex]:not([tabindex=\"-1\"])";
  function _stops(el) {
    if (!el.querySelectorAll) return [];
    return Array.from(el.querySelectorAll(FOCUSABLE)).filter((n) => !n.hidden && n.getAttribute("aria-hidden") !== "true");
  }
  // The traps that hold, the newest last: only it acts, so a trap opened
  // inside another's keeps focus, and the outer one waits.
  const _traps = [];
  // Focus a modal dialog or an open layer took is theirs, not a trap's to
  // pull back: a palette opened over a held find bar, a context menu.
  function _aboveTraps(node) {
    for (let n = node; n && n.nodeType === 1; n = n.parentNode) {
      if (n.tagName === "DIALOG" && n.hasAttribute("open")) return true;
      if (n.classList && n.classList.contains("wf-layer") && n.hasAttribute("data-open")) return true;
    }
    return false;
  }
  function trapFocus(el, read) {
    live(el);
    const self = {};
    const mine = () => _traps[_traps.length - 1] === self;
    let before = null;
    let held = false;
    const onKeys = (e) => {
      if (e.key !== "Tab" || !mine() || _aboveTraps(document.activeElement)) return;
      const stops = _stops(el);
      if (!stops.length) {
        if (e.preventDefault) e.preventDefault();
        return;
      }
      const first = stops[0];
      const last = stops[stops.length - 1];
      const at = document.activeElement;
      if (e.shiftKey && (at === first || !el.contains(at))) {
        if (e.preventDefault) e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && (at === last || !el.contains(at))) {
        if (e.preventDefault) e.preventDefault();
        first.focus();
      }
    };
    // Focus that lands outside — a click on the page — is brought back.
    const onFocus = (e) => {
      if (held && mine() && e.target && !el.contains(e.target) && !_aboveTraps(e.target)) {
        const stops = _stops(el);
        (stops[0] || el).focus();
      }
    };
    const hold = () => {
      if (held) return;
      held = true;
      _traps.push(self);
      before = typeof document !== "undefined" ? document.activeElement : null;
      document.addEventListener("keydown", onKeys, true);
      document.addEventListener("focusin", onFocus, true);
      const auto = el.querySelector && el.querySelector("[autofocus]");
      const target = auto || _stops(el)[0];
      if (target) target.focus();
      else {
        if (!el.hasAttribute("tabindex")) el.setAttribute("tabindex", "-1");
        el.focus();
      }
    };
    const release = () => {
      if (!held) return;
      held = false;
      const at = _traps.indexOf(self);
      if (at >= 0) _traps.splice(at, 1);
      document.removeEventListener("keydown", onKeys, true);
      document.removeEventListener("focusin", onFocus, true);
      if (before && before.focus && before.isConnected !== false) before.focus();
      before = null;
    };
    // After the element is drawn, so what is inside it can take focus.
    effect(() => {
      const on = !!read();
      if (typeof queueMicrotask === "function") queueMicrotask(() => (on ? hold() : release()));
      else if (on) hold();
      else release();
    });
    onCleanup(release);
  }
