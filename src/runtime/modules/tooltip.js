  function tooltip(root, tip) {
    if (!root || !tip) return;
    const focusable = root.querySelector(
      "button, a[href], input, select, textarea, summary, [tabindex]",
    );
    const trigger = focusable || root;
    if (!focusable && !root.hasAttribute("tabindex")) root.setAttribute("tabindex", "0");
    const existing = trigger.getAttribute("aria-describedby");
    trigger.setAttribute("aria-describedby", existing ? `${existing} ${tip.id}` : tip.id);
    root.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        root.setAttribute("data-dismissed", "");
        e.stopPropagation();
      }
    });
    const restore = () => root.removeAttribute("data-dismissed");
    root.addEventListener("mouseleave", restore);
    root.addEventListener("focusout", (e) => {
      if (!root.contains(e.relatedTarget)) restore();
    });
  }

  /// An engine string a project may translate: the key is looked up in the
  /// project's messages when it has i18n, else the English is used.
