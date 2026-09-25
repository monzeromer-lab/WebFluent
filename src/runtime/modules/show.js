  // ─── Show/Hide ───────────────────────────────────────
  function show(parent, condFn, contentFn, animConfig) {
    const wrapper = document.createElement("div");
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
        expand(wrapper, open, animConfig.duration, animConfig.easing);
      });
      return;
    }

    if (animConfig) {
      effect(() => {
        if (condFn()) {
          wrapper.style.display = "contents";
          if (animConfig.enter) {
            for (const n of wrapper.children) animateIn(n, animConfig.enter, animConfig.duration, animConfig.delay, animConfig.easing);
          }
        } else {
          const plays = leave([...wrapper.children], animConfig);
          if (plays.length) Promise.all(plays).then(() => { wrapper.style.display = "none"; });
          else wrapper.style.display = "none";
        }
      });
    } else {
      effect(() => {
        wrapper.style.display = condFn() ? "contents" : "none";
      });
    }
  }
