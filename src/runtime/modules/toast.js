  // ─── Toast ───────────────────────────────────────────
  //
  // A live region only announces changes made *after* it is in the document, so
  // the container is created once up front rather than lazily on the first
  // toast — otherwise the first notification, the one most worth hearing, is
  // the one that is silently dropped.
  let toastContainer = null;

  function _toastContainer() {
    if (!toastContainer) {
      toastContainer = document.createElement("div");
      toastContainer.className = "wf-toast-container";
      // `polite` waits for a pause rather than interrupting; not atomic, so a
      // second toast announces itself rather than re-reading the whole stack.
      toastContainer.setAttribute("role", "status");
      toastContainer.setAttribute("aria-live", "polite");
      toastContainer.setAttribute("aria-atomic", "false");
      document.body.appendChild(toastContainer);
    }
    return toastContainer;
  }

  function toast(message, variant, duration) {
    const container = _toastContainer();
    const toast = document.createElement("div");
    toast.className = `wf-toast wf-toast--${variant || "info"}`;
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(() => { toast.classList.add("wf-toast--exit"); setTimeout(() => toast.remove(), 300); }, duration || 3000);
  }
