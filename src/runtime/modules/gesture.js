  // ─── Gestures: drag, long-press, swipe, and the splitter ───────────
  //
  // What the pointer does over time, as one event with what it means: a
  // long press, a swipe and its direction, a drag that carries a value and
  // lands before, after or inside a target, a splitter's new size. All are
  // pointer events underneath, so a mouse, a pen and a finger are the same
  // code; a finger lifts a drag with a long press, so a plain swipe still
  // scrolls.

  const LONG_PRESS_MS = 500;
  // How far a pointer may wander and still be pressing, not moving.
  const PRESS_SLOP = 8;
  // How far a mouse moves before a press becomes a drag.
  const DRAG_START = 4;
  const SWIPE_MIN = 48;
  const SWIPE_MS = 600;
  // How near an edge of a scrolled box a drag scrolls it, and how fast.
  const EDGE = 28;
  const EDGE_SPEED = 14;

  // `on longpress(p)`, `on swipe(s)`, `on receive(d)`, `on dragmove(d)`:
  // the gesture's recogniser is set up on `el`, and `fn` hears what it saw.
  function gesture(el, name, fn) {
    live(el);
    if (name === "longpress") _longPress(el);
    else if (name === "swipe") _swipe(el);
    listen(el, `wf-${name}`, (e) => fn(e.detail));
  }

  function _say(el, name, detail) {
    if (typeof CustomEvent === "function") el.dispatchEvent(new CustomEvent(`wf-${name}`, { detail }));
    else el.dispatchEvent({ type: `wf-${name}`, detail });
  }

  // The click a gesture ended with is the gesture's, not a click: the one
  // the pointer's coming up makes. A gesture that ends while the pointer
  // is still down (a long press) waits for it to come up.
  function _swallowClick(el, stillDown) {
    const eat = (e) => {
      e.stopPropagation();
      if (e.preventDefault) e.preventDefault();
    };
    el.addEventListener("click", eat, { capture: true, once: true });
    const forget = () => setTimeout(() => el.removeEventListener("click", eat, { capture: true }), 0);
    if (!stillDown) return forget();
    const up = () => {
      document.removeEventListener("pointerup", up, true);
      document.removeEventListener("pointercancel", up, true);
      forget();
    };
    document.addEventListener("pointerup", up, true);
    document.addEventListener("pointercancel", up, true);
  }

  function _longPress(el) {
    if (el.__wfLongPress) return;
    el.__wfLongPress = true;
    let timer = 0;
    let start = null;
    const stop = () => {
      clearTimeout(timer);
      start = null;
    };
    el.addEventListener("pointerdown", (e) => {
      if (e.button !== undefined && e.button !== 0) return;
      start = { x: e.clientX, y: e.clientY };
      clearTimeout(timer);
      timer = setTimeout(() => {
        const at = start;
        start = null;
        if (!at) return;
        _swallowClick(el, true);
        _say(el, "longpress", { x: at.x, y: at.y, pointer: e.pointerType || "mouse" });
      }, LONG_PRESS_MS);
    });
    el.addEventListener("pointermove", (e) => {
      if (start && Math.hypot(e.clientX - start.x, e.clientY - start.y) > PRESS_SLOP) stop();
    });
    el.addEventListener("pointerup", stop);
    el.addEventListener("pointercancel", stop);
    // A finger held down asks the browser for its own menu too.
    el.addEventListener("contextmenu", (e) => {
      if (e.pointerType === "touch" || (start === null && timer && e.button === undefined)) {
        if (e.preventDefault) e.preventDefault();
      }
    });
    onCleanup(() => clearTimeout(timer));
  }

  function _swipe(el) {
    if (el.__wfSwipe) return;
    el.__wfSwipe = true;
    // Across is the swipe's; down the page is still the page's to scroll.
    if (el.style && !el.style.touchAction) el.style.touchAction = "pan-y";
    let start = null;
    let last = null;
    el.addEventListener("pointerdown", (e) => {
      // A mouse drags; a finger and a pen swipe.
      if ((e.pointerType || "mouse") === "mouse") return;
      start = { x: e.clientX, y: e.clientY, at: Date.now() };
      last = { x: e.clientX, y: e.clientY };
    });
    el.addEventListener("pointermove", (e) => {
      if (start) last = { x: e.clientX, y: e.clientY };
    });
    const end = (e) => {
      if (!start) return;
      // A pointer the browser took to scroll ends where it was last seen.
      const at = e.type === "pointercancel" ? last : { x: e.clientX, y: e.clientY };
      const dx = at.x - start.x;
      const dy = at.y - start.y;
      const quick = Date.now() - start.at <= SWIPE_MS;
      start = null;
      if (!quick) return;
      let direction = null;
      if (Math.abs(dx) >= SWIPE_MIN && Math.abs(dx) > 1.5 * Math.abs(dy)) direction = dx < 0 ? "left" : "right";
      else if (Math.abs(dy) >= SWIPE_MIN && Math.abs(dy) > 1.5 * Math.abs(dx)) direction = dy < 0 ? "up" : "down";
      if (direction) _say(el, "swipe", { direction, dx, dy });
    };
    el.addEventListener("pointerup", end);
    // The browser took the pointer to scroll: what it moved still counts.
    el.addEventListener("pointercancel", end);
  }

  // ─── Drag and drop ─────────────────────────────────────────────────

  // The drop targets on the page, each with its group, its value, its axis
  // and whether it takes a value inside.
  const _targets = new Map();

  // `drop…:` and `on receive(d)` on an element: it takes what is dropped
  // on it. `axis` is "y" for a list down the page, "x" for one across it
  // (a tab strip); with `inside`, the middle of it is inside (a folder, a
  // container).
  function dropTarget(el, options) {
    live(el);
    const opts = options || {};
    _targets.set(el, opts);
    el.setAttribute("data-wf-drop", "");
    el.setAttribute("data-drop-axis", opts.axis === "x" ? "x" : "y");
    onCleanup(() => _targets.delete(el));
  }

  function _targetAt(x, y, group, source) {
    if (typeof document === "undefined" || !document.elementFromPoint) return null;
    for (let n = document.elementFromPoint(x, y); n && n.nodeType === 1; n = n.parentNode) {
      const t = _targets.get(n);
      if (t && (t.group || "") === group && n !== source) return [n, t];
    }
    return null;
  }

  function _positionIn(el, t, x, y) {
    const r = el.getBoundingClientRect();
    const across = t.axis === "x";
    const size = across ? r.width : r.height;
    const at = size ? ((across ? x - r.left : y - r.top) / size) : 0.5;
    if (t.inside) return at < 0.25 ? "before" : at > 0.75 ? "after" : "inside";
    return at < 0.5 ? "before" : "after";
  }

  // The box a drag near its edge scrolls: the nearest one that scrolls.
  function _scroller(node) {
    for (let n = node; n && n.nodeType === 1; n = n.parentNode) {
      const s = getComputedStyle(n);
      const scrolls = (v) => v === "auto" || v === "scroll";
      if ((scrolls(s.overflowY) && n.scrollHeight > n.clientHeight) || (scrolls(s.overflowX) && n.scrollWidth > n.clientWidth)) return n;
    }
    return null;
  }

  // `drag: value` on an element: it can be picked up — by a mouse moved a
  // few pixels, or a finger held still — and carried to a drop target.
  // `group` keeps tabs and tree rows apart.
  function draggable(el, value, options) {
    live(el);
    const opts = options || {};
    const group = opts.group || "";
    el.setAttribute("data-wf-drag", "");
    let press = null;
    let drag = null;
    let timer = 0;
    let frame = 0;

    const begin = (e) => {
      const r = el.getBoundingClientRect();
      const ghost = el.cloneNode(true);
      ghost.removeAttribute("id");
      ghost.classList.add("wf-drag-ghost");
      ghost.setAttribute("aria-hidden", "true");
      ghost.style.width = `${r.width}px`;
      ghost.style.height = `${r.height}px`;
      document.body.appendChild(ghost);
      drag = { ghost, dx: press.x - r.left, dy: press.y - r.top, over: null, position: null, x: e.clientX, y: e.clientY };
      el.setAttribute("data-dragging", "");
      document.documentElement.setAttribute("data-wf-dragging", "");
      if (el.setPointerCapture && e.pointerId !== undefined) {
        try { el.setPointerCapture(e.pointerId); } catch (_) { /* a synthetic pointer */ }
      }
      move(e);
    };

    const mark = (target, position) => {
      if (drag.over && (drag.over[0] !== target || drag.position !== position)) drag.over[0].removeAttribute("data-drop");
      if (target) target.setAttribute("data-drop", position);
    };

    const scroll = () => {
      frame = 0;
      if (!drag) return;
      const under = document.elementFromPoint ? document.elementFromPoint(drag.x, drag.y) : null;
      const box = _scroller(under);
      if (!box) return;
      const r = box.getBoundingClientRect();
      let sx = 0;
      let sy = 0;
      if (drag.y < r.top + EDGE) sy = -EDGE_SPEED;
      else if (drag.y > r.bottom - EDGE) sy = EDGE_SPEED;
      if (drag.x < r.left + EDGE) sx = -EDGE_SPEED;
      else if (drag.x > r.right - EDGE) sx = EDGE_SPEED;
      if (!sx && !sy) return;
      box.scrollBy(sx, sy);
      frame = requestAnimationFrame(scroll);
    };

    const move = (e) => {
      drag.x = e.clientX;
      drag.y = e.clientY;
      drag.ghost.style.transform = `translate(${e.clientX - drag.dx}px, ${e.clientY - drag.dy}px)`;
      const hit = _targetAt(e.clientX, e.clientY, group, el);
      const position = hit ? _positionIn(hit[0], hit[1], e.clientX, e.clientY) : null;
      mark(hit && hit[0], position);
      const changed = (hit && hit[0]) !== (drag.over && drag.over[0]) || position !== drag.position;
      drag.over = hit;
      drag.position = position;
      if (changed) {
        const over = hit && hit[1].value ? hit[1].value() : null;
        _say(el, "dragmove", { value: value(), over, position });
      }
      if (!frame && typeof requestAnimationFrame === "function") frame = requestAnimationFrame(scroll);
    };

    const finish = (land) => {
      clearTimeout(timer);
      if (frame && typeof cancelAnimationFrame === "function") cancelAnimationFrame(frame);
      frame = 0;
      press = null;
      if (!drag) return;
      const was = drag;
      drag = null;
      was.ghost.remove();
      el.removeAttribute("data-dragging");
      document.documentElement.removeAttribute("data-wf-dragging");
      if (was.over) was.over[0].removeAttribute("data-drop");
      _swallowClick(el);
      // The drag is over: it is over nothing now.
      _say(el, "dragmove", { value: value(), over: null, position: null });
      if (land && was.over) {
        const [target, t] = was.over;
        _say(target, "receive", { value: value(), over: t.value ? t.value() : null, position: was.position });
      }
    };

    el.addEventListener("pointerdown", (e) => {
      if (e.button !== undefined && e.button !== 0) return;
      press = { x: e.clientX, y: e.clientY, touch: (e.pointerType || "mouse") !== "mouse" };
      if (press.touch) {
        clearTimeout(timer);
        timer = setTimeout(() => { if (press) begin(e); }, LONG_PRESS_MS);
      }
    });
    el.addEventListener("pointermove", (e) => {
      if (drag) return move(e);
      if (!press) return;
      const far = Math.hypot(e.clientX - press.x, e.clientY - press.y);
      if (press.touch) {
        if (far > PRESS_SLOP) { clearTimeout(timer); press = null; }
      } else if (far > DRAG_START) {
        begin(e);
      }
    });
    el.addEventListener("pointerup", () => finish(true));
    el.addEventListener("pointercancel", () => finish(false));
    // A finger that lifted a drag moves it, not the page.
    el.addEventListener("touchmove", (e) => { if (drag && e.cancelable) e.preventDefault(); }, { passive: false });
    // The browser's own drag of a link or an image would take over.
    el.addEventListener("dragstart", (e) => { if (e.preventDefault) e.preventDefault(); });
    const keys = (e) => {
      if (drag && e.key === "Escape") {
        if (e.preventDefault) e.preventDefault();
        if (e.stopPropagation) e.stopPropagation();
        finish(false);
      }
    };
    listen(document, "keydown", keys, true);
    onCleanup(() => finish(false));
  }

  // ─── The splitter ──────────────────────────────────────────────────
  //
  // A handle between two regions that sets the size of one: dragged, it
  // follows the pointer between `min` and `max`; double-clicked, or Enter,
  // it goes back to `reset`; the arrow keys move it 16 px (64 with Shift),
  // Home and End go to the ends. `reverse` is a handle on the start side of
  // what it sizes — a pane on the right, a panel at the bottom — so a drag
  // toward the start grows it.

  function splitter(el, read, write, options) {
    live(el);
    const opts = options || {};
    const across = opts.horizontal ? false : true;
    const sign = opts.reverse ? -1 : 1;
    const min = () => (opts.min === undefined ? 0 : Number(opts.min()));
    const max = () => (opts.max === undefined ? Infinity : Number(opts.max()));
    const clamp = (v) => Math.round(Math.min(Math.max(v, min()), max()));
    const set = (v) => {
      if (write) write(clamp(v));
    };
    // A separator in the tab order, unless its author wrote what it is.
    if (!el.hasAttribute("role")) el.setAttribute("role", "separator");
    if (!el.hasAttribute("tabindex")) el.setAttribute("tabindex", "0");
    el.setAttribute("aria-orientation", across ? "vertical" : "horizontal");
    effect(() => {
      el.setAttribute("aria-valuenow", String(Math.round(Number(read()) || 0)));
      el.setAttribute("aria-valuemin", String(min()));
      if (max() !== Infinity) el.setAttribute("aria-valuemax", String(max()));
    });
    // Where a move starts from: the region's size as it is drawn — the
    // region beside the handle, after it when the handle is `reverse` —
    // so a bound value that means "the default share" (0), or one the
    // page's own rules clamp, starts where the reader sees the edge.
    const drawn = () => {
      const region = opts.reverse ? el.nextElementSibling : el.previousElementSibling;
      const box = region && typeof region.getBoundingClientRect === "function" ? region.getBoundingClientRect() : null;
      const size = box ? (across ? box.width : box.height) : 0;
      return size > 0 ? Math.round(size) : Number(read()) || 0;
    };
    let from = null;
    el.addEventListener("pointerdown", (e) => {
      if (e.button !== undefined && e.button !== 0) return;
      from = { at: across ? e.clientX : e.clientY, size: drawn() };
      el.setAttribute("data-dragging", "");
      document.documentElement.setAttribute("data-wf-resizing", across ? "x" : "y");
      if (el.setPointerCapture && e.pointerId !== undefined) {
        try { el.setPointerCapture(e.pointerId); } catch (_) { /* a synthetic pointer */ }
      }
      if (e.preventDefault) e.preventDefault();
    });
    el.addEventListener("pointermove", (e) => {
      if (!from) return;
      const now = across ? e.clientX : e.clientY;
      set(from.size + sign * (now - from.at));
    });
    const done = () => {
      if (!from) return;
      from = null;
      el.removeAttribute("data-dragging");
      document.documentElement.removeAttribute("data-wf-resizing");
    };
    el.addEventListener("pointerup", done);
    el.addEventListener("pointercancel", done);
    // A reset is the author's own value, taken as written: `reset: 0`
    // can mean "back to the share" where 0 is below `min:`.
    const reset = () => {
      if (opts.reset && write) write(Math.round(Number(opts.reset()) || 0));
    };
    el.addEventListener("dblclick", reset);
    el.addEventListener("keydown", (e) => {
      const step = e.shiftKey ? 64 : 16;
      const grow = across ? (e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0) : (e.key === "ArrowDown" ? 1 : e.key === "ArrowUp" ? -1 : 0);
      let consumed = true;
      if (grow) set(drawn() + sign * grow * step);
      else if (e.key === "Home") set(min());
      else if (e.key === "End" && max() !== Infinity) set(max());
      else if (e.key === "Enter" && opts.reset) reset();
      else consumed = false;
      if (consumed && e.preventDefault) e.preventDefault();
    });
    onCleanup(done);
  }
