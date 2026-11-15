  // ─── Display and layout ──────────────────────────────
  //
  // What the second group of display and layout built-ins need of a running
  // page: a count of the avatars past the first few, a ring that fills, the
  // edges of a box that scrolls, a box that knows it is stuck, and handles
  // between panels.

  /// `AvatarGroup(max:)`: past the first `max`, the rest hidden and counted
  /// in one `+3`. Counted again when the avatars change.
  function avatarGroup(root, max) {
    live(root);
    if (!(max > 0)) return;
    const more = el("span", { className: "wf-avatar wf-avatar-group__more" });
    const count = () => {
      const all = [...root.children].filter((c) => c !== more);
      all.forEach((c, i) => { c.hidden = i >= max; });
      const rest = all.length - max;
      more.textContent = `+${rest}`;
      more.setAttribute("aria-label", `${rest} more`);
      if (rest > 0) root.appendChild(more);
      else if (more.parentNode) more.remove();
    };
    count();
    const seen = new MutationObserver(() => {
      seen.disconnect();
      count();
      seen.observe(root, { childList: true });
    });
    seen.observe(root, { childList: true });
    onCleanup(() => seen.disconnect());
  }

  /// `CircularProgress`: an SVG ring that fills to `value()` of `max()`, or
  /// turns while `value()` is null, with the progress bar's values said.
  function ring(root, value, max) {
    live(root);
    const NS = "http://www.w3.org/2000/svg";
    const svg = document.createElementNS(NS, "svg");
    svg.setAttribute("viewBox", "0 0 36 36");
    svg.setAttribute("aria-hidden", "true");
    svg.setAttribute("class", "wf-circular-progress__ring");
    const circle = (cls) => {
      const c = document.createElementNS(NS, "circle");
      for (const [k, v] of [["cx", "18"], ["cy", "18"], ["r", "16"], ["pathLength", "100"], ["class", cls]]) c.setAttribute(k, v);
      svg.appendChild(c);
      return c;
    };
    circle("wf-circular-progress__track");
    const bar = circle("wf-circular-progress__bar");
    root.insertBefore(svg, root.firstChild);
    effect(() => {
      const v = value();
      const top = Number(max()) || 100;
      root.setAttribute("aria-valuemin", "0");
      root.setAttribute("aria-valuemax", String(top));
      if (v === null || v === undefined) {
        root.setAttribute("data-indeterminate", "");
        root.removeAttribute("aria-valuenow");
        bar.setAttribute("stroke-dasharray", "25 75");
        return;
      }
      root.removeAttribute("data-indeterminate");
      const share = Math.max(0, Math.min(1, Number(v) / top));
      root.setAttribute("aria-valuenow", String(v));
      bar.setAttribute("stroke-dasharray", `${share * 100} 100`);
    });
  }

  /// `ScrollArea`: `data-before` and `data-after` while there is more to
  /// scroll to on either side, for the fades at its edges.
  function scrollArea(root) {
    live(root);
    const across = root.classList.contains("wf-scroll-area--horizontal");
    const mark = () => {
      const at = across ? root.scrollLeft : root.scrollTop;
      const room = across ? root.scrollWidth - root.clientWidth : root.scrollHeight - root.clientHeight;
      const back = _isRtl(root) && across ? -1 : 1;
      root.toggleAttribute("data-before", at * back > 1);
      root.toggleAttribute("data-after", room - at * back > 1);
    };
    root.addEventListener("scroll", mark, { passive: true });
    if (typeof ResizeObserver === "function") {
      const sized = new ResizeObserver(mark);
      sized.observe(root);
      onCleanup(() => sized.disconnect());
    }
    requestAnimationFrame(mark);
  }

  /// `Sticky`: `data-stuck` while the box rests at the edge it sticks to.
  function sticky(root) {
    live(root);
    const bottom = root.classList.contains("wf-sticky--bottom");
    let frame = 0;
    const mark = () => {
      frame = 0;
      const style = getComputedStyle(root);
      const r = root.getBoundingClientRect();
      const stuck = bottom
        ? Math.abs(window.innerHeight - parseFloat(style.bottom || "0") - r.bottom) < 1 && r.top > 0
        : Math.abs(r.top - parseFloat(style.top || "0")) < 1 && window.scrollY > 0;
      root.toggleAttribute("data-stuck", stuck);
    };
    const soon = () => {
      if (!frame) frame = requestAnimationFrame(mark);
    };
    window.addEventListener("scroll", soon, { passive: true, capture: true });
    window.addEventListener("resize", soon);
    onCleanup(() => {
      window.removeEventListener("scroll", soon, { capture: true });
      window.removeEventListener("resize", soon);
    });
    soon();
  }

  /// `PanelGroup`: a handle (`role="separator"`) between each two panels;
  /// dragged, or the arrow keys on it (Shift for larger steps), Home and End,
  /// it moves the share between the two, each held to its `data-min`. The
  /// sizes, in percent, come from `read()` when it holds as many as there
  /// are panels, and go out as a `resize` event's `detail`.
  function panelGroup(root, read, label) {
    live(root);
    const vertical = root.classList.contains("wf-panel-group--vertical");
    const panels = () => [...root.children].filter((c) => c.classList.contains("wf-panel-group__panel"));
    let sizes = [];
    const min = (p) => Number(p.getAttribute("data-min")) || 10;
    const paint = () => {
      const list = panels();
      list.forEach((p, i) => { p.style.flexBasis = `${sizes[i]}%`; });
      for (const h of root.querySelectorAll(":scope > .wf-panel-group__handle")) {
        const i = Number(h.dataset.at);
        h.setAttribute("aria-valuenow", String(Math.round(sizes[i])));
        h.setAttribute("aria-valuemin", String(min(list[i])));
        h.setAttribute("aria-valuemax", String(Math.round(sizes[i] + sizes[i + 1] - min(list[i + 1]))));
      }
    };
    const say = () => root.dispatchEvent(new CustomEvent("resize", { detail: sizes.slice() }));
    const setUp = () => {
      const list = panels();
      for (const h of root.querySelectorAll(":scope > .wf-panel-group__handle")) h.remove();
      list.forEach((p, i) => {
        if (i === 0) return;
        const h = el("div", {
          className: "wf-panel-group__handle",
          role: "separator",
          tabindex: "0",
          "aria-orientation": vertical ? "horizontal" : "vertical",
          "aria-label": label || "Resize",
        });
        h.dataset.at = String(i - 1);
        if (!list[i - 1].id) list[i - 1].id = uid("wf-panel");
        h.setAttribute("aria-controls", list[i - 1].id);
        root.insertBefore(h, p);
      });
      const given = read();
      sizes = Array.isArray(given) && given.length === list.length ? given.map(Number) : list.map(() => 100 / list.length);
      paint();
    };
    // Move the line between panel `i` and `i + 1` by `by` percent.
    const folded = new Map();
    const move = (i, by) => {
      if (folded.has(i)) return;
      const list = panels();
      const pair = sizes[i] + sizes[i + 1];
      const left = Math.max(min(list[i]), Math.min(pair - min(list[i + 1]), sizes[i] + by));
      if (left === sizes[i]) return;
      sizes[i] = left;
      sizes[i + 1] = pair - left;
      paint();
      say();
    };
    // A collapsible panel folds away behind the handle after it, and its
    // share goes to its neighbour; again, and it comes back as it was.
    const fold = (i) => {
      const list = panels();
      const panel = list[i];
      if (!panel || panel.getAttribute("data-collapsible") !== "true") return false;
      if (folded.has(i)) {
        const was = folded.get(i);
        folded.delete(i);
        sizes[i + 1] = Math.max(min(list[i + 1]), sizes[i + 1] - was);
        sizes[i] = was;
        panel.hidden = false;
      } else {
        folded.set(i, sizes[i]);
        sizes[i + 1] += sizes[i];
        sizes[i] = 0;
        panel.hidden = true;
      }
      paint();
      say();
      return true;
    };
    root.addEventListener("dblclick", (e) => {
      const h = e.target;
      if (h.classList && h.classList.contains("wf-panel-group__handle")) fold(Number(h.dataset.at));
    });
    root.addEventListener("keydown", (e) => {
      const h = e.target;
      if (!h.classList || !h.classList.contains("wf-panel-group__handle")) return;
      const i = Number(h.dataset.at);
      if (e.key === "Enter" && fold(i)) {
        e.preventDefault();
        return;
      }
      const step = e.shiftKey ? 10 : 2;
      const flip = !vertical && _isRtl(root) ? -1 : 1;
      const less = vertical ? "ArrowUp" : "ArrowLeft";
      const more = vertical ? "ArrowDown" : "ArrowRight";
      if (e.key === less) move(i, -step * flip);
      else if (e.key === more) move(i, step * flip);
      else if (e.key === "Home") move(i, -100);
      else if (e.key === "End") move(i, 100);
      else return;
      e.preventDefault();
    });
    root.addEventListener("pointerdown", (e) => {
      const h = e.target;
      if (!h.classList || !h.classList.contains("wf-panel-group__handle")) return;
      e.preventDefault();
      h.focus();
      const i = Number(h.dataset.at);
      const box = root.getBoundingClientRect();
      const whole = vertical ? box.height : box.width;
      const flip = !vertical && _isRtl(root) ? -1 : 1;
      let last = vertical ? e.clientY : e.clientX;
      const drag = (m) => {
        const now = vertical ? m.clientY : m.clientX;
        move(i, ((now - last) / whole) * 100 * flip);
        last = now;
      };
      const stop = () => {
        window.removeEventListener("pointermove", drag);
        window.removeEventListener("pointerup", stop);
        root.removeAttribute("data-dragging");
      };
      root.setAttribute("data-dragging", "");
      window.addEventListener("pointermove", drag);
      window.addEventListener("pointerup", stop);
    });
    setUp();
    // Sizes written from outside — a `persist` read back, a reset — are
    // drawn; the group's own writes come back equal and change nothing.
    effect(() => {
      const given = read();
      if (Array.isArray(given) && given.length === sizes.length && given.some((v, i) => Math.abs(v - sizes[i]) > 0.01)) {
        sizes = given.map(Number);
        paint();
      }
    });
  }
