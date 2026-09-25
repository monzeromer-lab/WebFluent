  // ─── List rendering ─────────────────────────────────
  // ─── Lists ───────────────────────────────────────────
  // Every item renders to its nodes after the marker. With a `key`, an item
  // keeps its nodes for as long as it is the same value under the same key:
  // items are inserted, removed and moved rather than rebuilt, so the focus,
  // scroll and animation state of the others survive a change. An item whose
  // value changed is rebuilt in place; one whose body reads its index is
  // rebuilt when its position changes. Without a key the list is rebuilt.
  function each(parent, listFn, itemFn, config, keyFn) {
    const marker = document.createComment("wf-for");
    parent.appendChild(marker);
    const key = (config && config.key) || keyFn || null;
    const byIndex = !!(config && config.index);
    let entries = new Map();
    let currentNodes = [];
    const exiting = new Set();
    // What each item's body created, disposed of when the item leaves.
    let disposers = [];

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
    const depart = (nodes) => {
      const plays = leave(nodes, config);
      if (!plays.length) { removeNodes(nodes); return; }
      for (const n of nodes) exiting.add(n);
      Promise.all(plays).then(() => {
        for (const n of nodes) exiting.delete(n);
        removeNodes(nodes);
      });
    };
    // Where each kept node was before a change, so a node that moved can
    // slide from there to where it is now (first, last, invert, play).
    const positions = () => {
      const seen = new Map();
      if (reducedMotion() || typeof requestAnimationFrame !== "function") return seen;
      for (const entry of entries.values()) {
        for (const n of entry.nodes) {
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
    // node on its way out does not count as a neighbour.
    const place = (host, nodes, after) => {
      for (const n of nodes) {
        let next = after.nextSibling;
        while (next && exiting.has(next)) next = next.nextSibling;
        if (next !== n) host.insertBefore(n, next);
        after = n;
      }
      return after;
    };
    const untracked = (fn) => {
      const prev = currentEffect;
      currentEffect = null;
      try { return fn(); } finally { currentEffect = prev; }
    };

    effect(() => {
      const items = listFn() || [];
      if (!key) {
        // Rebuilt whole.
        for (const d of disposers.splice(0)) d();
        depart(currentNodes);
        currentNodes = [];
        untracked(() => {
          const frag = document.createDocumentFragment();
          items.forEach((item, index) => {
            const [made, dispose] = scoped(() => itemFn(item, index));
            disposers.push(dispose);
            const nodes = toNodes(made);
            for (const n of nodes) { frag.appendChild(n); currentNodes.push(n); }
            enter(nodes, index);
          });
          if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
        });
        return;
      }
      untracked(() => {
        const host = marker.parentNode;
        if (!host) return;
        const before = positions();
        const next = new Map();
        const seen = new Map();
        let cursor = marker;
        items.forEach((item, index) => {
          let k = key(item, index);
          if (next.has(k)) {
            // Two items under one key: the later ones are told apart by
            // how many came before, and the author is told once.
            const n = (seen.get(k) || 1) + 1;
            seen.set(k, n);
            if (n === 2) console.warn("WF: two items of a list share the key " + JSON.stringify(k) + "; add a `by` that tells them apart");
            k = String(k) + "#" + n;
          }
          const old = entries.get(k);
          let entry;
          if (old && old.item === item && (!byIndex || old.index === index)) {
            entry = old;
            entry.index = index;
          } else {
            const [made, dispose] = scoped(() => itemFn(item, index));
            entry = { item, index, nodes: toNodes(made), dispose };
            if (old) { if (old.dispose) old.dispose(); removeNodes(old.nodes); }
          }
          entries.delete(k);
          cursor = place(host, entry.nodes, cursor);
          if (!old) enter(entry.nodes, index);
          next.set(k, entry);
        });
        // What is left never came back.
        for (const gone of entries.values()) { if (gone.dispose) gone.dispose(); depart(gone.nodes); }
        entries = next;
        slide(before);
      });
    });
  }
