  function when(parent, condFn, thenFn, elseFn, animConfig) {
    const start = region(parent, "wf-if");
    let lastShow = undefined;
    let pendingRemoval = null; // Track in-progress exit animations
    // What the branch's body created, disposed of when the branch leaves.
    let dispose = null;
    // What it holds now goes when what owns it does — a route change, an
    // enclosing branch — not only when it changes its mind.
    onCleanup(() => { if (dispose) { dispose(); dispose = null; } });

    // Only track the condition signal — not signals read during rendering
    effect(() => {
      const show = !!condFn();
      if (show === lastShow) return;
      lastShow = show;
      if (dispose) { dispose(); dispose = null; }

      // A branch that comes back before its exit has played: the old
      // nodes go at once, the new ones take their place.
      if (pendingRemoval) {
        pendingRemoval.pending.cancelled = true;
        removeNodes(pendingRemoval.nodes);
        pendingRemoval = null;
      }

      // Remove what the branch holds now — what a branch inside it drew
      // since included — once what asked for an exit has played it. Its
      // body is disposed of, so nothing in it draws again while it leaves.
      const toRemove = between(start);
      const pending = leaveThenRemove(toRemove, animConfig, () => { pendingRemoval = null; });
      if (pending) pendingRemoval = { nodes: toRemove, pending };

      // Add new nodes (untracked so rendering doesn't subscribe this effect to state signals)
      const renderFn = show ? thenFn : elseFn;
      if (renderFn) {
        const prev = currentEffect;
        currentEffect = null; // Untrack: don't subscribe to signals during render
        try {
          const [result, disposer] = scoped(() => _drawnIn(start, renderFn()));
          dispose = disposer;
          // Collect actual child nodes — DocumentFragments lose children when appended
          let nodes;
          if (result instanceof DocumentFragment) {
            nodes = [...result.childNodes];
          } else {
            nodes = [].concat(result).flat().filter(n => n instanceof Node);
          }
          const frag = document.createDocumentFragment();
          for (const n of nodes) frag.appendChild(n);
          if (start.parentNode) start.parentNode.insertBefore(frag, start.nextSibling);
          if (animConfig && animConfig.enter) {
            nodes.forEach(n => { if (n instanceof Element) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay, animConfig.easing); });
          }
        } finally {
          currentEffect = prev;
        }
      }
    });
  }

  // `if let x = v { … }`: the branch is drawn while `v` is not null, and
  // reads `x` through this — `v` as it is now, so a new value of it reaches
  // what the branch drew in place; or, while the branch is on its way out
  // because `v` went null, the last value it had.
  function held(read) {
    let last;
    return () => {
      const v = read();
      if (v != null) last = v;
      return last;
    };
  }
