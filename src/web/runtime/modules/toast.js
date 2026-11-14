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

  // The most toasts shown at once: a fourth closes the oldest.
  const _TOAST_LIMIT = 3;
  const _toasts = [];

  /// Show `message` for a while, above everything, and say it to a screen
  /// reader. `options` (or, as before, a tone and a duration):
  ///   tone      "info" (the default), "success", "warning", "danger"
  ///   duration  how long it stays, in ms (3 s; 8 s with an action);
  ///             `Infinity` until it is closed
  ///   action    the words of a button on it ("Undo"), and `onAction` what
  ///             the button does; pressing it closes the toast
  ///   closable  false for no close button
  /// It waits while the pointer is on it or the focus is in it, so there is
  /// time to read it and reach its button. What this returns closes it, or
  /// changes what it says.
  function toast(message, options, duration) {
    const opts = typeof options === "object" && options !== null ? options : { tone: options, duration };
    const container = _toastContainer();
    _rehome();
    const node = document.createElement("div");
    const text = document.createElement("span");
    text.className = "wf-toast__text";
    node.appendChild(text);
    let tone = "info";
    const say = (words, t) => {
      tone = t || tone;
      node.className = `wf-toast wf-toast--${tone}`;
      text.textContent = words == null ? "" : String(words);
      announce(text.textContent, tone === "danger" ? "assertive" : "polite");
    };
    say(message, opts.tone || opts.variant || "info");
    let gone = false;
    let timer = 0;
    let left = 0;
    let since = 0;
    const close = () => {
      if (gone) return;
      gone = true;
      clearTimeout(timer);
      const at = _toasts.indexOf(handle);
      if (at >= 0) _toasts.splice(at, 1);
      node.classList.add("wf-toast--exit");
      let removed = false;
      const remove = () => {
        if (removed) return;
        removed = true;
        node.remove();
        if (!container.firstElementChild) _raiseToasts();
      };
      // Gone when its exit has played, or at once for a reader who asked
      // for less motion (no animation runs, so none ends).
      node.addEventListener("animationend", remove);
      setTimeout(remove, 400);
    };
    const run = (ms) => {
      clearTimeout(timer);
      left = ms;
      if (!(ms < Infinity)) return;
      since = Date.now();
      timer = setTimeout(close, ms);
    };
    const pause = () => {
      if (gone || !(left < Infinity)) return;
      clearTimeout(timer);
      left = Math.max(0, left - (Date.now() - since));
      timer = 0;
    };
    const resume = () => {
      if (gone || timer || !(left < Infinity)) return;
      if (node.matches && (node.matches(":hover") || node.matches(":focus-within"))) return;
      run(Math.max(left, 1000));
    };
    if (opts.action) {
      const act = document.createElement("button");
      act.type = "button";
      act.className = "wf-toast__action";
      act.textContent = String(opts.action);
      act.addEventListener("click", () => {
        if (typeof opts.onAction === "function") opts.onAction();
        close();
      });
      node.appendChild(act);
    }
    if (opts.closable !== false) {
      const x = document.createElement("button");
      x.type = "button";
      x.className = "wf-toast__close";
      x.setAttribute("aria-label", "Close");
      x.textContent = "\u00d7";
      x.addEventListener("click", close);
      node.appendChild(x);
    }
    node.addEventListener("mouseenter", pause);
    node.addEventListener("mouseleave", resume);
    node.addEventListener("focusin", pause);
    node.addEventListener("focusout", () => setTimeout(resume, 0));
    container.appendChild(node);
    _raiseToasts();
    const handle = {
      el: node,
      close,
      /// Say something else, in another tone, and stay for `ms` from now.
      update: (words, t, ms) => {
        if (gone) return;
        say(words, t);
        run(ms === undefined ? 3000 : ms);
      },
    };
    _toasts.push(handle);
    while (_toasts.length > _TOAST_LIMIT) _toasts[0].close();
    run(opts.duration !== undefined && opts.duration !== null ? opts.duration : opts.action ? 8000 : 3000);
    return handle;
  }

  /// `toast.promise(…)` in a program: a toast that follows a promise: `loading` while it runs, then `done`
  /// or `failed` — words, or a function of the value or the error. It
  /// hands back the promise, so the action can still `await` it.
  function toastPromise(promise, options) {
    const opts = options || {};
    const words = (w, v, fallback) => (typeof w === "function" ? w(v) : w == null ? fallback : w);
    const shown = toast(words(opts.loading, undefined, "…"), { tone: "info", duration: Infinity, closable: false });
    Promise.resolve(promise).then(
      (v) => shown.update(words(opts.done, v, "Done"), "success"),
      (e) => shown.update(words(opts.failed, e, e && e.message ? e.message : "Failed"), "danger", 6000),
    );
    return promise;
  }
