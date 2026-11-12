  function slot(parent, values, render) {
    const start = region(parent, "wf-slot");
    // A fill still playing its exit: it finishes, and is not taken twice.
    const leaving = new Set();
    let dispose = null;
    // What it holds now goes when what owns it does — a route change, an
    // enclosing branch — not only when it changes its mind.
    onCleanup(() => { if (dispose) { dispose(); dispose = null; } });
    effect(() => {
      const handed = values();
      if (dispose) { dispose(); dispose = null; }
      // What the fill holds now, a branch inside it included.
      const old = between(start).filter((n) => !leaving.has(n));
      for (const n of old) leaving.add(n);
      leaveThenRemove(old, null, () => { for (const n of old) leaving.delete(n); });
      const prev = currentEffect;
      currentEffect = null;
      let result;
      try {
        [result, dispose] = scoped(() => _drawnIn(start, render(handed)));
      } finally {
        currentEffect = prev;
      }
      if (result == null) return;
      const nodes = result instanceof DocumentFragment
        ? [...result.childNodes]
        : [].concat(result).flat().filter(n => n instanceof Node);
      const frag = document.createDocumentFragment();
      for (const n of nodes) frag.appendChild(n);
      if (start.parentNode) start.parentNode.insertBefore(frag, start.nextSibling);
    });
  }
