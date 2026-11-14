  // ─── Toast ───────────────────────────────────────────
  //
  // The stack is drawn in the top layer, as a popover, so nothing on the
  // page covers it: a modal dialog opened before the toast would otherwise.
  // An open modal also makes everything outside it inert, so the stack
  // moves into the topmost one while it is open (`_keepHeard`), and is
  // shown again there, above it. What a toast says is announced through
  // the page's live region, which is in the document from the start.
  let toastContainer = null;

  function _toastContainer() {
    if (!toastContainer) {
      toastContainer = document.createElement("div");
      toastContainer.className = "wf-toast-container";
      if (typeof toastContainer.showPopover === "function") toastContainer.setAttribute("popover", "manual");
      _keepHeard(toastContainer, _raiseToasts);
    }
    return toastContainer;
  }

  /// Show the stack in the top layer above everything opened before it,
  /// and take it out of the top layer when it is empty.
  function _raiseToasts() {
    const c = toastContainer;
    if (!c || typeof c.showPopover !== "function") return;
    try {
      if (c.matches(":popover-open")) c.hidePopover();
      if (c.firstElementChild && c.isConnected) c.showPopover();
    } catch (e) {
      // A container moved while the browser was closing it: the next
      // toast shows it again.
    }
  }

  function toast(message, variant, duration) {
    const container = _toastContainer();
    _rehome();
    const toast = document.createElement("div");
    toast.className = `wf-toast wf-toast--${variant || "info"}`;
    toast.textContent = message;
    container.appendChild(toast);
    _raiseToasts();
    announce(message);
    setTimeout(() => {
      toast.classList.add("wf-toast--exit");
      let gone = false;
      const remove = () => {
        if (gone) return;
        gone = true;
        toast.remove();
        if (!container.firstElementChild) _raiseToasts();
      };
      // Gone when its exit has played, or at once for a reader who asked
      // for less motion (no animation runs, so none ends).
      toast.addEventListener("animationend", remove);
      setTimeout(remove, 400);
    }, duration || 3000);
  }
