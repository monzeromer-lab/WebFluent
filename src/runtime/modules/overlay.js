  // ─── Dialogs, popups and tablists ────────────────────

  /// Drive a `<dialog>` from a boolean signal.
  ///
  /// `showModal()` is what buys the focus trap, the inert background, Escape to
  /// close and `aria-modal`. The browser can close the dialog without us — via
  /// Escape or the backdrop — so the `close` event writes back to the signal;
  /// without that the state says "open" while the screen says otherwise, and the
  /// next toggle appears to do nothing.
  // `visible:` as a read, and a write for when the browser closes the
  // dialog itself (Escape, the backdrop) so the state follows; a
  // `visible:` that is a condition has no write, and stays as it is.
  function dialog(el, read, write) {
    live(el);
    if (write === undefined && typeof read.set === "function") {
      const signal = read;
      write = (v) => signal.set(v);
    }
    // A modal open from the start is drawn before its page is in the
    // document, and `showModal()` throws on a dialog that is not — which
    // stopped the whole page drawing. It opens once it is there.
    let wanted = false;
    // While it is open it is one of the layers, when the page has any
    // (`_stackDialog`): its Escape is then the newest layer's — this one's,
    // before a popover's under it.
    let unstack = null;
    const stacked = () => {
      if (!unstack && typeof _stackDialog === "function") unstack = _stackDialog(el);
    };
    const unstacked = () => {
      if (unstack) unstack();
      unstack = null;
    };
    const open = (tries) => {
      if (!wanted || el.open) return;
      if (el.isConnected === false) {
        if (tries > 0) requestAnimationFrame(() => open(tries - 1));
        return;
      }
      if (el.showModal) el.showModal();
      else el.setAttribute("open", "");
      stacked();
    };
    effect(() => {
      const shouldBeOpen = read();
      wanted = !!shouldBeOpen;
      if (shouldBeOpen && !el.open) {
        if (el.isConnected !== false) open(0);
        else queueMicrotask(() => open(60));
      } else if (!shouldBeOpen && el.open) {
        if (el.close) el.close();
        else el.removeAttribute("open");
        unstacked();
      }
    });
    el.addEventListener("close", () => {
      unstacked();
      if (write && read()) write(false);
    });
    onCleanup(unstacked);
    // The Escape that closes it is its own: nothing behind it — a page's
    // `on key("Escape")`, the back rule of an app — hears it too. The
    // browser still closes the dialog (its `cancel` is not prevented).
    el.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && el.open && e.stopPropagation) e.stopPropagation();
    });
  }

  /// Close a popup on Escape or an outside click, returning focus to its trigger.
  ///
  /// A keyboard user who opens a menu must be able to leave it without tabbing
  /// through every item, and must land back where they were.
  function popup(root, trigger, openSignal) {
    live(root, trigger);
    document.addEventListener("click", (e) => {
      if (!root.contains(e.target)) openSignal.set(false);
    });
    root.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && openSignal()) {
        openSignal.set(false);
        if (trigger && trigger.focus) trigger.focus();
        if (e.stopPropagation) e.stopPropagation();
      }
    });
  }

  /// Arrow-key navigation for a `role="tabs"`.
  ///
  /// The WAI-ARIA pattern puts only the selected tab in the tab order and moves
  /// between tabs with the arrow keys, so Tab leaves the widget rather than
  /// walking through every tab in it.
  function tabs(nav, activeSignal) {
    live(nav);
    nav.addEventListener("keydown", (e) => {
      const tabs = nav.querySelectorAll("button");
      if (!tabs.length) return;
      const current = activeSignal();
      let next = null;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") next = (current + 1) % tabs.length;
      else if (e.key === "ArrowLeft" || e.key === "ArrowUp") next = (current - 1 + tabs.length) % tabs.length;
      else if (e.key === "Home") next = 0;
      else if (e.key === "End") next = tabs.length - 1;
      if (next === null) return;
      if (e.preventDefault) e.preventDefault();
      activeSignal.set(next);
      if (tabs[next] && tabs[next].focus) tabs[next].focus();
    });
  }

  /// Move the reader to a page the router just rendered into `container`.
