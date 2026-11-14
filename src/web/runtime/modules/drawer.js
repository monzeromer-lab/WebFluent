  function drawer(panel, toggle, scrim) {
    live(panel, toggle, scrim);
    if (!panel || !toggle) return;
    let open = false;

    const apply = () => {
      panel.setAttribute("data-open", open ? "true" : "false");
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
      if (scrim) {
        if (open) scrim.removeAttribute("hidden");
        else scrim.setAttribute("hidden", "");
      }
    };

    const set = (next, restoreFocus) => {
      open = next;
      apply();
      if (!next && restoreFocus && toggle.focus) toggle.focus();
    };

    // A sidebar opened from its toggle takes the focus to its first link,
    // so a keyboard reader is in the navigation they asked for rather than
    // behind it.
    const enter = () => {
      if (!scrim || !panel.querySelector) return;
      const first = panel.querySelector("a[href], button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex='-1'])");
      if (first && first.focus) first.focus();
    };

    toggle.addEventListener("click", () => {
      set(!open, false);
      if (open) enter();
    });
    if (scrim) scrim.addEventListener("click", () => set(false, true));
    // On the document, so it is taken off when the page leaves.
    listen(document, "keydown", (e) => {
      if (e.key === "Escape" && open) set(false, true);
    });

    apply();
    return { open: () => set(true, false), close: () => set(false, false) };
  }
