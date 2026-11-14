
  // ─── The sidebar's drawer ─────────────────────────────
  //
  // On a narrow screen a `Sidebar` is a drawer its toggle opens over the
  // page. Open, it is a layer like any other: the focus goes to its first
  // link and is held inside it, Escape (when it is the newest layer) or a
  // press outside closes it, and the focus goes back to the toggle. A
  // `Navbar`'s links (no scrim) close the same way, with no trap: they
  // open in the bar, not over the page.
  function drawer(panel, toggle, scrim) {
    live(panel, toggle, scrim);
    if (!panel || !toggle) return;
    const open = signal(false);
    let entry = null;

    const apply = () => {
      const on = open();
      panel.setAttribute("data-open", on ? "true" : "false");
      toggle.setAttribute("aria-expanded", on ? "true" : "false");
      if (scrim) {
        if (on) scrim.removeAttribute("hidden");
        else scrim.setAttribute("hidden", "");
      }
    };

    const set = (next) => {
      if (open() === next) return;
      // A navbar's links have no trap to give the focus back: closing them
      // with the focus inside puts it on the toggle.
      const inside = !next && !scrim && panel.contains && panel.contains(document.activeElement) && document.activeElement !== toggle;
      open.set(next);
      apply();
      if (inside && toggle.focus) toggle.focus();
      if (next) {
        _hearLayerKeys();
        entry = { el: panel, anchor: toggle, dismiss: true, close: () => set(false) };
        _layers.push(entry);
      } else if (entry) {
        const at = _layers.indexOf(entry);
        if (at >= 0) _layers.splice(at, 1);
        entry = null;
      }
    };

    toggle.addEventListener("click", () => set(!open()));
    if (scrim) {
      scrim.addEventListener("click", () => set(false));
      // Held while it is open: Tab stays in the drawer, and closing it gives
      // the focus back to what had it, the toggle.
      trapFocus(panel, () => open());
    }
    onCleanup(() => set(false));

    apply();
    return { open: () => set(true), close: () => set(false) };
  }
