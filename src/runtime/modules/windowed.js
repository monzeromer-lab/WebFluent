  // ─── A long list, drawn a window at a time ────────────────────────
  //
  // `VirtualList { for row in rows by row.id { … } }`: the box scrolls, and
  // only the rows in view — and a few either side — are drawn; two spacers
  // stand for the rest, so the scrollbar is the whole list's. A row keeps
  // its nodes for as long as it stays in the window, and is told in place
  // when its item or its position changes, as a keyed `for`'s are.
  //
  // `rowHeight` is every row's height, the fast path; without it each row
  // is measured once drawn, and `estimate` stands for one not yet drawn. A
  // row measured taller or shorter above what is in view moves the scroll
  // by as much, so what the reader is looking at stays put. `reveal` reads
  // a key: when it changes, that row is scrolled into view.

  const OVERSCAN = 6;

  function windowed(box, listFn, itemFn, config) {
    live(box);
    const opts = config || {};
    const key = opts.key || ((item, index) => index);
    const fixed = Number(opts.rowHeight) || 0;
    const estimate = Number(opts.estimate) || fixed || 32;
    const overscan = opts.overscan === undefined ? OVERSCAN : opts.overscan;
    box.setAttribute("data-wf-virtual", "");
    const space = () => {
      const s = document.createElement("div");
      s.className = "wf-virtual-list__space";
      s.setAttribute("aria-hidden", "true");
      return s;
    };
    const top = space();
    const bottom = space();
    box.appendChild(top);
    box.appendChild(bottom);

    let items = [];
    let keys = [];
    let where = new Map();
    // What each row measured, by key, for a list without `rowHeight`.
    const heights = new Map();
    // The rows drawn now, by key.
    let rows = new Map();
    let frame = 0;
    let settling = 0;

    const heightOf = (i) => fixed || heights.get(keys[i]) || estimate;
    let offsets = null;
    const measureAll = () => {
      if (fixed) return;
      offsets = new Float64Array(items.length + 1);
      for (let i = 0; i < items.length; i++) offsets[i + 1] = offsets[i] + heightOf(i);
    };
    const offset = (i) => (fixed ? i * fixed : offsets[i]);
    // The first row whose bottom is below `y`.
    const rowAt = (y) => {
      if (fixed) return Math.min(items.length, Math.max(0, Math.floor(y / fixed)));
      let lo = 0;
      let hi = items.length;
      while (lo < hi) {
        const mid = (lo + hi) >> 1;
        if (offsets[mid + 1] <= y) lo = mid + 1;
        else hi = mid;
      }
      return lo;
    };
    const viewHeight = () => box.clientHeight || (typeof window !== "undefined" ? window.innerHeight : 0) || 0;

    const nodesOf = (made) =>
      made instanceof DocumentFragment ? [...made.childNodes] : [].concat(made).flat().filter((n) => n instanceof Node);
    // A row's nodes as they are now: an `if` or a `for` inside it draws
    // between markers, and what it drew since is the row's too.
    const nodesNowOf = (row) => nodesNow(row.kept);
    const drop = (k, row) => {
      const now = nodesNowOf(row);
      // The reader's focus in a row scrolled away goes to the list, so the
      // keys still reach it.
      if (typeof document !== "undefined" && now.some((n) => n.contains && n.contains(document.activeElement))) {
        if (!box.hasAttribute("tabindex")) box.setAttribute("tabindex", "-1");
        if (box.focus) box.focus({ preventScroll: true });
      }
      if (resized) for (const n of now) if (n.nodeType === 1) resized.unobserve(n);
      for (const n of now) if (n.parentNode) n.parentNode.removeChild(n);
      row.dispose();
      rows.delete(k);
    };
    // A row with no `rowHeight` that grows or shrinks after it is drawn —
    // text that wraps, a font that arrives — is measured again.
    let resized = null;
    const untracked = (fn) => {
      const was = currentEffect;
      currentEffect = null;
      try { return fn(); } finally { currentEffect = was; }
    };

    const layout = () => {
      frame = 0;
      measureAll();
      const n = items.length;
      const total = offset(n);
      // A list that just got shorter is looked at from its end: where the
      // browser will clamp the scroll to.
      const at = Math.min(box.scrollTop || 0, Math.max(0, total - viewHeight()));
      let start = rowAt(at);
      let end = rowAt(at + viewHeight());
      if (end < n) end += 1;
      start = Math.max(0, start - overscan);
      end = Math.min(n, end + overscan);

      // What left the window, or the list, goes.
      for (const [k, row] of [...rows]) {
        const i = where.get(k);
        if (i === undefined || i < start || i >= end) drop(k, row);
      }
      // What is in it is drawn, or told where it is now, in order.
      untracked(() => {
        let after = top;
        for (let i = start; i < end; i++) {
          const k = keys[i];
          let row = rows.get(k);
          if (row) {
            if (row.item !== items[i]) { row.item = items[i]; row.itemOf.set(items[i]); }
            if (row.index !== i) { row.index = i; row.indexOf.set(i); }
          } else {
            const itemOf = signal(items[i]);
            const indexOf = signal(i);
            const [made, dispose] = scoped(() => itemFn(items[i], i, itemOf, indexOf));
            row = { kept: keepNodes(nodesOf(made)), dispose, item: items[i], index: i, itemOf, indexOf };
            rows.set(k, row);
            if (resized) for (const n of nodesNowOf(row)) if (n.nodeType === 1) resized.observe(n);
          }
          const now = nodesNowOf(row);
          for (const node of now) {
            if (after.nextSibling !== node) box.insertBefore(node, after.nextSibling);
            after = node;
          }
          const first = now.find((x) => x.nodeType === 1);
          if (first) {
            first.setAttribute("data-wf-index", String(i));
            // Only the window is in the page: a list's or a tree's item
            // says where it is in the whole.
            const role = first.getAttribute("role") || (first.tagName === "LI" ? "listitem" : "");
            if (/^(listitem|treeitem|option|row|menuitem|article)$/.test(role)) {
              first.setAttribute("aria-posinset", String(i + 1));
              first.setAttribute("aria-setsize", String(n));
            }
          }
        }
      });
      top.style.height = `${offset(start)}px`;
      bottom.style.height = `${Math.max(0, total - offset(end))}px`;

      // Rows without a height of their own: measure what was drawn, and
      // lay out again with what they measured — a few times at most.
      if (!fixed && settling < 4) {
        let changed = false;
        let above = 0;
        for (let i = start; i < end; i++) {
          const row = rows.get(keys[i]);
          if (!row) continue;
          let h = 0;
          for (const node of nodesNowOf(row)) if (node.nodeType === 1 && node.getBoundingClientRect) h += node.getBoundingClientRect().height;
          if (!h) continue;
          const was = heightOf(i);
          if (Math.abs(h - was) > 0.5) {
            heights.set(keys[i], h);
            changed = true;
            if (offset(i) < at) above += h - was;
          }
        }
        if (changed) {
          settling++;
          if (above) box.scrollTop = at + above;
          layout();
          settling--;
        }
      }
    };
    const soon = () => {
      if (frame) return;
      if (typeof requestAnimationFrame === "function") frame = requestAnimationFrame(layout);
      else layout();
    };

    effect(() => {
      const list = listFn() || [];
      untracked(() => {
        items = list;
        keys = list.map((item, i) => key(item, i));
        where = new Map(keys.map((k, i) => [k, i]));
        layout();
      });
    });

    if (typeof opts.reveal === "function") {
      effect(() => {
        const wanted = opts.reveal();
        untracked(() => {
          if (wanted === null || wanted === undefined) return;
          const i = where.get(wanted);
          if (i === undefined) return;
          measureAll();
          const rowTop = offset(i);
          const rowBottom = rowTop + heightOf(i);
          const at = box.scrollTop || 0;
          const view = viewHeight();
          if (rowTop < at) box.scrollTop = rowTop;
          else if (rowBottom > at + view) box.scrollTop = rowBottom - view;
          layout();
        });
      });
    }

    listen(box, "scroll", soon, { passive: true });
    if (typeof ResizeObserver === "function") {
      const watcher = new ResizeObserver(soon);
      watcher.observe(box);
      onCleanup(() => watcher.disconnect());
      if (!fixed) {
        resized = new ResizeObserver(soon);
        onCleanup(() => resized.disconnect());
        for (const row of rows.values()) for (const n of nodesNowOf(row)) if (n.nodeType === 1) resized.observe(n);
      }
    } else if (typeof window !== "undefined") {
      listen(window, "resize", soon);
    }
    onCleanup(() => {
      if (frame && typeof cancelAnimationFrame === "function") cancelAnimationFrame(frame);
      for (const [k, row] of [...rows]) drop(k, row);
      rows = new Map();
    });
  }
