  // ─── List rendering ─────────────────────────────────
  // ─── Lists ───────────────────────────────────────────
  // Every item renders to its nodes after the marker. With a `key`, an item
  // keeps its nodes for as long as it is the same value under the same key:
  // items are inserted, removed and moved rather than rebuilt, so the focus,
  // scroll and animation state of the others survive a change. An item whose
  // value changed is rebuilt in place; one whose body reads its index is
  // rebuilt when its position changes. Without a key the list is rebuilt.
  //
  // `live` — what the compiler asks for a keyed `for` — keeps an item's
  // nodes for as long as its key is in the list: the body is also handed
  // the item and its index as accessors, `itemFn(item, index, itemOf,
  // indexOf)`, reads them through those, and a new value or a new position
  // under the same key reaches what it drew in place. A refreshed list whose
  // rows are new objects with the same ids redraws nothing but what changed.
  function each(parent, listFn, itemFn, config, keyFn) {
    const start = region(parent, "wf-for");
    const key = (config && config.key) || keyFn || null;
    const byIndex = !!(config && config.index);
    const live = !!(key && config && config.live);
    // A list that moves with motion — one with an enter or an exit, or that
    // asks to (a sortable table) — slides a moved item from where it was.
    const slides = !!(config && (config.slide || config.enter || config.exit));
    let entries = new Map();
    const exiting = new Set();
    // What each item's body created, disposed of when the item leaves.
    let disposers = [];
    // And when the list itself goes — a route change, an enclosing branch.
    onCleanup(() => {
      for (const d of disposers.splice(0)) d();
      for (const entry of entries.values()) if (entry.dispose) entry.dispose();
      entries = new Map();
    });

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
    // `still`: the caller knows nothing leaving plays an exit.
    const depart = (nodes, still) => {
      if (still) { removeNodes(nodes); return; }
      const plays = leave(nodes, config);
      if (!plays.length) { removeNodes(nodes); return; }
      for (const n of nodes) exiting.add(inPage(n));
      Promise.all(plays).then(() => {
        for (const n of nodes) exiting.delete(inPage(n));
        removeNodes(nodes);
      });
    };
    // Where each kept node that is about to move was before a change, so it
    // can slide from there to where it is now (first, last, invert, play).
    // Only those are measured: reading every row's place forces a layout and
    // costs each row, and a list that only grows at its end, or is keyed by
    // its index, moves none.
    const positions = (moving) => {
      const seen = new Map();
      if (!moving.length || reducedMotion() || typeof requestAnimationFrame !== "function") return seen;
      for (const entry of moving) {
        for (const drawn of nodesNow(entry.nodes)) {
          const n = inPage(drawn);
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
    // node on its way out does not count as a neighbour. A node drawn over
    // a pre-rendered page may stand for the painted one (hydrate).
    const place = (host, nodes, after) => {
      for (const drawn of nodes) {
        const n = inPage(drawn);
        let next = after.nextSibling;
        while (next && exiting.has(next)) next = next.nextSibling;
        if (next !== n) host.insertBefore(n, next);
        after = n;
      }
      return after;
    };
    // One item that throws is one item: the rest of the list still draws.
    // Under `wf serve` it shows where it is, with what it threw; on a
    // deployed site it leaves a gap, and the console says why.
    const drawItem = (itemFn, item, index, itemOf, indexOf) => {
      try {
        return itemFn(item, index, itemOf, indexOf);
      } catch (e) {
        console.error("WF: item " + index + " of a list could not be drawn:", e);
        if (!devMode()) return document.createComment("wf: item " + index + " failed");
        const box = document.createElement("div");
        box.className = "wf-item-error";
        box.setAttribute("role", "alert");
        box.textContent = "Item " + index + " could not be drawn: " + (e && e.message ? e.message : String(e));
        return box;
      }
    };
    // A keyed item, drawn. Its nodes, with a branch or a list of its own
    // kept by its marker, are read as they are now each time the item is
    // moved or taken away. A live item's index is a signal only once its
    // body reads it: most rows never do, and a signal a row is a set and
    // four functions.
    const make = (item, index) => {
      const entry = { item, index, itemOf: null, indexOf: null, nodes: null, dispose: null };
      let indexOf = null;
      if (live) {
        entry.itemOf = signal(item);
        indexOf = () => (entry.indexOf || (entry.indexOf = signal(entry.index)))();
      }
      const [made, dispose] = scoped(() => drawItem(itemFn, item, index, entry.itemOf, indexOf));
      entry.nodes = keepNodes(toNodes(made));
      entry.dispose = dispose;
      return entry;
    };
    const untracked = (fn) => {
      const prev = currentEffect;
      currentEffect = null;
      try { return fn(); } finally { currentEffect = prev; }
    };

    // A change made while the list is being drawn — a handler that runs as
    // a node leaves (an input's `blur`) and writes the list — waits for the
    // drawing to finish and is drawn after it, from the list as it is then.
    // Drawn inside, it would find the items already moved missing and draw
    // them twice.
    let drawing = false;
    let again = false;
    // The list drawn last. Asked again for the same list — the same array,
    // because what it was read from changed somewhere else (a refreshed
    // response whose rows did not change) — there is nothing to draw.
    let lastList;
    effect(() => {
      const items = listFn() || [];
      if (drawing) { again = true; return; }
      drawing = true;
      try {
        draw(items);
        while (again) {
          again = false;
          draw(untracked(() => listFn() || []));
        }
      } finally {
        drawing = false;
      }
    });

    // The places of the longest run of `olds` (old indices, -1 for an item
    // not kept) that still ascends: the items that need not move.
    function staying(olds) {
      const tails = [];
      const prev = new Array(olds.length).fill(-1);
      for (let i = 0; i < olds.length; i++) {
        const v = olds[i];
        if (v < 0) continue;
        let lo = 0, hi = tails.length;
        while (lo < hi) {
          const mid = (lo + hi) >> 1;
          if (olds[tails[mid]] < v) lo = mid + 1; else hi = mid;
        }
        if (lo > 0) prev[i] = tails[lo - 1];
        tails[lo] = i;
      }
      const out = new Set();
      for (let i = tails.length ? tails[tails.length - 1] : -1; i >= 0; i = prev[i]) out.add(i);
      return out;
    }

    function draw(items) {
      if (items === lastList) return;
      lastList = items;
      if (!key) {
        // Rebuilt whole.
        for (const d of disposers.splice(0)) d();
        // What the list holds now, what its items' own branches drew since
        // included; what is already on its way out is not taken twice.
        depart(between(start).filter((n) => !exiting.has(n)));
        untracked(() => {
          const frag = document.createDocumentFragment();
          items.forEach((item, index) => {
            const [made, dispose] = scoped(() => drawItem(itemFn, item, index));
            disposers.push(dispose);
            const nodes = toNodes(made);
            for (const n of nodes) frag.appendChild(n);
            enter(nodes, index);
          });
          if (start.parentNode) start.parentNode.insertBefore(frag, start.nextSibling);
        });
        return;
      }
      untracked(() => {
        const host = start.parentNode;
        if (!host) return;
        // Each item's key, two under one key told apart by how many came
        // before (the author told once).
        const keys = [];
        const counted = new Map();
        const taken = new Set();
        items.forEach((item, index) => {
          let k = key(item, index);
          if (taken.has(k)) {
            const n = (counted.get(k) || 1) + 1;
            counted.set(k, n);
            if (n === 2) console.warn("WF: two items of a list share the key " + JSON.stringify(k) + "; add a `by` that tells them apart");
            k = String(k) + "#" + n;
          }
          taken.add(k);
          keys.push(k);
        });
        const olds = keys.map((k) => entries.get(k));
        const next = new Map();
        // Whether what leaves this time leaves at once: no exit can play
        // (no motion in the build, or a reader who asked for less), or the
        // list asks for none and nothing in its parent carries one. Asked
        // once a change, not once a row.
        let still;
        const quiet = () => {
          if (still === undefined) {
            still = typeof animateOut !== "function" || reducedMotion()
              || (!(config && config.exit) && !_hyd && typeof host.querySelector === "function" && !host.querySelector("[data-wf-exit]"));
          }
          return still;
        };
        const end = start.__wfEnd;
        // Nothing kept — a list drawn from empty, replaced or emptied: the
        // new items are drawn into one fragment and put in at once, and
        // when the list is all its parent holds and nothing plays an exit,
        // the old ones go with one write rather than one removal a row.
        if (!_hyd && end && end.parentNode === host && !olds.some(Boolean)) {
          const frag = document.createDocumentFragment();
          const made = [];
          items.forEach((item, index) => {
            const entry = make(item, index);
            const nodes = nodesNow(entry.nodes);
            for (const n of nodes) frag.appendChild(n);
            made.push(nodes);
            next.set(keys[index], entry);
          });
          if (entries.size && !exiting.size && typeof host.replaceChildren === "function"
              && host.firstChild === start && host.lastChild === end && quiet()) {
            for (const gone of entries.values()) if (gone.dispose) gone.dispose();
            host.replaceChildren(start, frag, end);
          } else {
            // Where the first new item would have been put: after the
            // start, past what is already on its way out.
            let at = start.nextSibling;
            while (at && exiting.has(at)) at = at.nextSibling;
            host.insertBefore(frag, at);
            for (const gone of entries.values()) { if (gone.dispose) gone.dispose(); depart(nodesNow(gone.nodes), quiet()); }
          }
          made.forEach((nodes, index) => enter(nodes, index));
          entries = next;
          return;
        }
        // Which kept items stay where they are: the longest run of them
        // still in their old order. Only the others are moved — a row taken
        // out of a thousand moves none, two rows swapped move two — and only
        // they can slide (a row that merely closed a gap stays still).
        const stay = staying(olds.map((o, i) => (o && (live || (o.item === items[i] && (!byIndex || o.index === i))) ? o.index : -1)));
        const moving = olds.filter((o, i) => o && !stay.has(i));
        const before = slides ? positions(moving) : new Map();
        let cursor = start;
        items.forEach((item, index) => {
          const k = keys[index];
          const old = olds[index];
          let entry;
          if (old && live) {
            // The same key: the same nodes, told what the item and its
            // position are now.
            entry = old;
            if (entry.item !== item) { entry.item = item; entry.itemOf.set(item); }
            if (entry.index !== index) { entry.index = index; if (entry.indexOf) entry.indexOf.set(index); }
          } else if (old && old.item === item && (!byIndex || old.index === index)) {
            entry = old;
            entry.index = index;
          } else {
            entry = make(item, index);
            if (old) { if (old.dispose) old.dispose(); removeNodes(nodesNow(old.nodes)); }
          }
          entries.delete(k);
          const nodes = nodesNow(entry.nodes);
          if (entry === old && stay.has(index) && nodes.length) cursor = nodes[nodes.length - 1];
          else cursor = place(host, nodes, cursor);
          if (!old) enter(nodes, index);
          next.set(k, entry);
        });
        // What is left never came back.
        for (const gone of entries.values()) { if (gone.dispose) gone.dispose(); depart(nodesNow(gone.nodes), quiet()); }
        entries = next;
        if (slides) slide(before);
      });
    }
  }
