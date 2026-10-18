  // ─── Match ───────────────────────────────────────────
  //
  // Renders the arm named by `key()` — a resource's state, or an enum's case
  // — and renders again when the arm it names changes: loading, ready and
  // error for a resource, a case for an enum (`else` is one arm, whichever
  // keys it stands for). The arm is called with `arg()` — a resource's data
  // or error, an enum's value — and with `live`, which reads it again: the
  // compiler reads an arm's binding through `live`, so what the arm drew
  // follows a refresh of the data, or a new payload of the same case, in
  // place. `live` reads the value only while its arm is the one shown, so an
  // effect inside an arm on its way out never sees the next arm's value.
  // Rendering is untracked, as it is for a condition, so only the key
  // decides when to redraw.
  function match(parent, key, arg, arms) {
    const start = region(parent, "wf-match");
    // An arm still playing its exit: it finishes, and is not taken twice.
    const leaving = new Set();
    const armOf = (k) => (Object.prototype.hasOwnProperty.call(arms, k) ? arms[k] : arms.else);
    let started = false;
    let shown;
    let dispose = null;
    // What it holds now goes when what owns it does — a route change, an
    // enclosing branch — not only when it changes its mind.
    onCleanup(() => { if (dispose) { dispose(); dispose = null; } });
    effect(() => {
      const arm = armOf(key());
      if (started && arm === shown) return;
      started = true;
      shown = arm;
      if (dispose) { dispose(); dispose = null; }
      // What the arm holds now, a branch inside it included.
      const old = between(start).filter((n) => !leaving.has(n));
      for (const n of old) leaving.add(n);
      leaveThenRemove(old, null, () => { for (const n of old) leaving.delete(n); });
      if (!arm) return;
      const prev = currentEffect;
      currentEffect = null;
      try {
        const handed = arg ? arg() : undefined;
        let current = handed;
        const live = () => {
          if (armOf(key()) === arm) current = arg ? arg() : undefined;
          return current;
        };
        const [result, disposer] = scoped(() => arm(handed, live));
        dispose = disposer;
        const nodes = result instanceof DocumentFragment
          ? [...result.childNodes]
          : [].concat(result).flat().filter(n => n instanceof Node);
        const frag = document.createDocumentFragment();
        for (const n of nodes) frag.appendChild(n);
        if (start.parentNode) start.parentNode.insertBefore(frag, start.nextSibling);
      } finally {
        currentEffect = prev;
      }
    });
  }

  // A scoped slot: `values()` reads what the component hands over, and
  // `render(values)` is the fill's nodes. They are drawn again when a
  // value the handing read changes; the fill itself renders untracked, as
  // a match arm does, so only the handing decides when to redraw.
