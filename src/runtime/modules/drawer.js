  function drawer(panel, toggle, scrim) {
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

    toggle.addEventListener("click", () => set(!open, false));
    if (scrim) scrim.addEventListener("click", () => set(false, true));
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && open) set(false, true);
    });

    apply();
    return { open: () => set(true, false), close: () => set(false, false) };
  }
