  // ─── Show/Hide ───────────────────────────────────────
  function show(parent, condFn, contentFn, animConfig) {
    const wrapper = document.createElement("div");
    // The wrapper in the page: a painted one, once a pre-rendered page has
    // been taken over (hydrate).
    const box = () => inPage(wrapper);
    wrapper.style.display = "contents";
    const nodes = [].concat(contentFn()).flat();
    for (const n of nodes) {
      if (n instanceof Node) wrapper.appendChild(n);
    }
    parent.appendChild(wrapper);

    // `show x { Stack(animate: .expand) }` opens and closes the box itself:
    // a wrapper of `display: contents` has no height to animate, so this
    // one is a block, and the content stays in the document throughout.
    if (animConfig && (animConfig.enter === "expand" || animConfig.exit === "expand")) {
      wrapper.style.display = "block";
      wrapper.style.overflow = "hidden";
      let first = true;
      effect(() => {
        const open = !!condFn();
        if (first) {
          first = false;
          wrapper.style.height = open ? "" : "0px";
          return;
        }
        expand(box(), open, animConfig.duration, animConfig.easing);
      });
      return;
    }

    if (animConfig) {
      // Each change is a turn: an exit that ends after the content was
      // shown again is an old turn's, and hides nothing.
      let turn = 0;
      effect(() => {
        const mine = ++turn;
        const shown = box();
        if (condFn()) {
          shown.style.display = "contents";
          for (const n of shown.children) n.inert = false;
          if (animConfig.enter) {
            for (const n of shown.children) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay, animConfig.easing);
          }
        } else {
          const plays = leave([...shown.children], animConfig);
          if (plays.length) Promise.all(plays).then(() => { if (mine === turn) shown.style.display = "none"; });
          else shown.style.display = "none";
        }
      });
    } else {
      effect(() => {
        box().style.display = condFn() ? "contents" : "none";
      });
    }
  }
