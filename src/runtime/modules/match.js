  // ─── Match ───────────────────────────────────────────
  //
  // Renders the arm named by `key()` — a resource's state, or an enum's case
  // — and renders again when it changes. The arm is called with `arg()`: a
  // resource's data or error, nothing for an enum. Rendering is untracked, as
  // it is for a condition, so only the key decides when to redraw.
  function match(parent, key, arg, arms) {
    const marker = document.createComment("wf-match");
    parent.appendChild(marker);
    let currentNodes = [];
    let lastKey;
    let dispose = null;
    effect(() => {
      const k = key();
      if (k === lastKey) return;
      lastKey = k;
      if (dispose) { dispose(); dispose = null; }
      leaveThenRemove(currentNodes, null);
      currentNodes = [];
      const arm = Object.prototype.hasOwnProperty.call(arms, k) ? arms[k] : arms.else;
      if (!arm) return;
      const prev = currentEffect;
      currentEffect = null;
      try {
        const handed = arg ? arg() : undefined;
        const [result, disposer] = scoped(() => arm(handed));
        dispose = disposer;
        const nodes = result instanceof DocumentFragment
          ? [...result.childNodes]
          : [].concat(result).flat().filter(n => n instanceof Node);
        currentNodes = nodes.slice();
        const frag = document.createDocumentFragment();
        for (const n of nodes) frag.appendChild(n);
        if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
      } finally {
        currentEffect = prev;
      }
    });
  }

  // A scoped slot: `values()` reads what the component hands over, and
  // `render(values)` is the fill's nodes. They are drawn again when a
  // value the handing read changes; the fill itself renders untracked, as
  // a match arm does, so only the handing decides when to redraw.
