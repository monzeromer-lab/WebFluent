  function when(parent, condFn, thenFn, elseFn, animConfig) {
    const marker = document.createComment("wf-if");
    parent.appendChild(marker);
    let currentNodes = [];
    let lastShow = undefined;
    let pendingRemoval = null; // Track in-progress exit animations
    // What the branch's body created, disposed of when the branch leaves.
    let dispose = null;

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

      // Remove old nodes, once what asked for an exit has played it.
      const toRemove = [...currentNodes];
      currentNodes = [];
      const pending = leaveThenRemove(toRemove, animConfig, () => { pendingRemoval = null; });
      if (pending) pendingRemoval = { nodes: toRemove, pending };

      // Add new nodes (untracked so rendering doesn't subscribe this effect to state signals)
      const renderFn = show ? thenFn : elseFn;
      if (renderFn) {
        const prev = currentEffect;
        currentEffect = null; // Untrack: don't subscribe to signals during render
        try {
          const [result, disposer] = scoped(renderFn);
          dispose = disposer;
          // Collect actual child nodes — DocumentFragments lose children when appended
          let nodes;
          if (result instanceof DocumentFragment) {
            nodes = [...result.childNodes];
          } else {
            nodes = [].concat(result).flat().filter(n => n instanceof Node);
          }
          currentNodes = nodes.slice();
          const frag = document.createDocumentFragment();
          for (const n of nodes) frag.appendChild(n);
          if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
          if (animConfig && animConfig.enter) {
            nodes.forEach(n => { if (n instanceof Element) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay, animConfig.easing); });
          }
        } finally {
          currentEffect = prev;
        }
      }
    });
  }
