  // ─── Roving focus ─────────────────────────────────────
  //
  // A group a reader moves through with the arrow keys rather than Tab: a
  // tab strip, a menu, a toolbar, a row of choices. One item is the
  // group's single Tab stop (`tabindex="0"`, the rest `-1`), so Tab enters
  // the group and leaves it; inside it the arrow keys move, Home and End
  // go to the ends, and with `typeahead` a letter goes to the next item
  // whose name starts with it.
  //
  // Across a right-to-left box the next item is to the left, so
  // ArrowLeft goes forward there. A key held with Alt, Ctrl or ⌘ is the
  // browser's or the page's (Alt+ArrowLeft goes back) and is left alone.
  //
  // `options`:
  //   items     a selector under the box, or a function returning the items;
  //             by default each child (or the first focusable thing in it)
  //   axis      "x" (the default), "y", or "both"
  //   wrap      false to stop at the ends
  //   typeahead true to jump by the first letters typed
  //   active    a read of the current item's index: the Tab stop follows it,
  //             and comes back to it when focus leaves the group. Without
  //             it, the item marked `aria-selected`, `aria-checked` or
  //             `aria-pressed` is the stop
  //   onMove    (index, item, event) after a move
  //   stops     false for a group whose items all stay `tabindex="-1"` (a
  //             menu, which takes the focus itself when it opens)

  const _FOCUSABLE = "a[href], button, input, select, textarea, [tabindex]";

  function _rovingUsable(n) {
    return !!n && !n.hidden && !n.disabled && n.getAttribute("aria-disabled") !== "true";
  }

  function _rovingItems(box, selector) {
    if (typeof selector === "string") return Array.from(box.querySelectorAll(selector));
    const out = [];
    for (const child of Array.from(box.children || [])) {
      const own = child.matches ? child.matches(_FOCUSABLE) : /^(A|BUTTON|INPUT|SELECT|TEXTAREA)$/.test(child.tagName || "");
      const item = own ? child : child.querySelector ? child.querySelector(_FOCUSABLE) : null;
      if (item) out.push(item);
    }
    return out;
  }

  function roving(box, options) {
    live(box);
    const opts = options || {};
    const axis = opts.axis || "x";
    const wrap = opts.wrap !== false;
    const stops = opts.stops !== false;
    const items = () => {
      const all = typeof opts.items === "function" ? opts.items() : _rovingItems(box, opts.items);
      return all.filter(_rovingUsable);
    };
    const holding = (list, node) => list.findIndex((n) => n === node || (n.contains && n.contains(node)));
    const stopAt = (list, i) => {
      if (!stops) return;
      list.forEach((n, j) => n.setAttribute("tabindex", j === i ? "0" : "-1"));
    };
    // The Tab stop: the active item, or the one that already is, or the first.
    const sync = () => {
      const list = items();
      if (!list.length) return;
      let at = -1;
      if (opts.active) {
        const a = opts.active();
        const item = (typeof opts.items === "function" ? opts.items() : _rovingItems(box, opts.items))[a];
        at = list.indexOf(item);
      }
      // A group its author wrote marks its chosen item, and the stop is
      // there: a tab's `aria-selected`, a choice's `aria-checked`, a
      // toggle's `aria-pressed`.
      if (at < 0) at = list.findIndex((n) => ["aria-selected", "aria-checked", "aria-pressed"].some((a) => n.getAttribute(a) === "true"));
      if (at < 0) at = list.findIndex((n) => n.getAttribute("tabindex") === "0");
      stopAt(list, at < 0 ? 0 : at);
    };
    const move = (list, i, e) => {
      stopAt(list, i);
      if (list[i].focus) list[i].focus();
      if (opts.onMove) opts.onMove(holding(typeof opts.items === "function" ? opts.items() : _rovingItems(box, opts.items), list[i]), list[i], e);
    };
    let typed = "";
    let typedAt = 0;
    box.addEventListener("keydown", (e) => {
      if (e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
      const list = items();
      // The focus on one of the group's own items, not on something inside
      // one (a submenu's item in its menu's): that is the inner group's.
      const at = list.indexOf(document.activeElement);
      if (at < 0) return;
      const rtl = _isRtl(box);
      const across = axis !== "y";
      const down = axis !== "x";
      const last = list.length - 1;
      const step = (d) => {
        const i = at + d;
        if (i < 0) return wrap ? last : 0;
        if (i > last) return wrap ? 0 : last;
        return i;
      };
      let to = null;
      if (across && e.key === (rtl ? "ArrowLeft" : "ArrowRight")) to = step(1);
      else if (across && e.key === (rtl ? "ArrowRight" : "ArrowLeft")) to = step(-1);
      else if (down && e.key === "ArrowDown") to = step(1);
      else if (down && e.key === "ArrowUp") to = step(-1);
      else if (e.key === "Home") to = 0;
      else if (e.key === "End") to = last;
      else if (opts.typeahead && e.key && e.key.length === 1 && e.key !== " ") {
        const now = Date.now();
        typed = (now - typedAt < 600 ? typed : "") + e.key.toLowerCase();
        typedAt = now;
        const name = (n) => (n.getAttribute("aria-label") || n.textContent || "").trim().toLowerCase();
        // A first letter, or the same letter again, looks from the next
        // item on (so `b`, `b` goes through every item that starts with
        // b); more letters keep to the item they started on while it
        // still matches.
        const seek = (want, from) => {
          for (let k = from; k <= list.length; k++) {
            const j = (at + k) % list.length;
            if (name(list[j]).startsWith(want)) return j;
          }
          return null;
        };
        const same = [...typed].every((c) => c === typed[0]);
        to = same ? seek(typed[0], 1) : seek(typed, 0);
        // Letters that name nothing together start again from the last.
        if (to === null && typed.length > 1) {
          typed = e.key.toLowerCase();
          to = seek(typed, 1);
        }
        if (to === null) return;
      }
      if (to === null) return;
      if (e.preventDefault) e.preventDefault();
      // A group inside another (a submenu in its menu) moves alone.
      if (e.stopPropagation) e.stopPropagation();
      move(list, to, e);
    });
    if (stops) {
      // A click (or a script) that puts the focus on an item makes it the
      // Tab stop; leaving the group puts the stop back on the active item.
      box.addEventListener("focusin", (e) => {
        const list = items();
        const at = holding(list, e.target);
        if (at >= 0) stopAt(list, at);
      });
      box.addEventListener("focusout", (e) => {
        if (!(e.relatedTarget && box.contains && box.contains(e.relatedTarget))) sync();
      });
      if (opts.active) effect(sync);
      else sync();
      // Items a `for` adds or takes away are put in line with the rest, and
      // the stop follows the chosen item when it changes.
      if (typeof MutationObserver === "function") {
        const seen = new MutationObserver(() => {
          if (!box.contains || !box.contains(document.activeElement)) sync();
        });
        seen.observe(box, { childList: true, subtree: true, attributes: true, attributeFilter: ["aria-selected", "aria-checked", "aria-pressed"] });
        onCleanup(() => seen.disconnect());
      }
    }
    return {
      sync,
      move: (i) => {
        const list = items();
        if (list[i]) move(list, i);
      },
    };
  }
