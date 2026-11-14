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
      // The event comes a task after the dialog shut. If it was opened
      // again in between — the close button wrote the state at once, and
      // a press set it back — this one is stale: writing `false` now
      // would shut the dialog the reader just opened.
      if (el.open) return;
      unstacked();
      if (write && read()) write(false);
    });
    // `.closable`'s button: shut it and write the state now. The `close`
    // event comes a task later, and a press in between (open it again)
    // would set a state that still said open, and be lost.
    el._wfShut = () => {
      if (el.close) el.close();
      else el.removeAttribute("open");
      unstacked();
      if (write && read()) write(false);
    };
    onCleanup(unstacked);
    // A press on the backdrop closes it — light dismiss, as a reader
    // expects of anything drawn over the page. The dialog's box is exactly
    // its card, and a press on the backdrop lands on the dialog itself, so
    // a press outside the box that also began there (not a text selection
    // dragged out of the card) is the backdrop's. `closedby` asks the same
    // of a browser that knows it.
    if (el.setAttribute) el.setAttribute("closedby", "any");
    const outside = (e) => {
      if (e.target !== el || !el.getBoundingClientRect) return false;
      const r = el.getBoundingClientRect();
      return e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom;
    };
    let pressedOutside = false;
    el.addEventListener("pointerdown", (e) => { pressedOutside = outside(e); });
    el.addEventListener("click", (e) => {
      const close = pressedOutside && outside(e);
      pressedOutside = false;
      if (close && el.open) el._wfShut();
    });
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
    listen(document, "click", (e) => {
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
      // A key held with a modifier is the browser's or the page's —
      // Alt+ArrowLeft goes back — not a move between tabs.
      if (e.altKey || e.ctrlKey || e.metaKey) return;
      const tabs = nav.querySelectorAll("button");
      if (!tabs.length) return;
      const current = activeSignal();
      // Across a right-to-left page the next tab is to the left.
      const rtl = _isRtl(nav);
      const forward = rtl ? "ArrowLeft" : "ArrowRight";
      const back = rtl ? "ArrowRight" : "ArrowLeft";
      let next = null;
      if (e.key === forward || e.key === "ArrowDown") next = (current + 1) % tabs.length;
      else if (e.key === back || e.key === "ArrowUp") next = (current - 1 + tabs.length) % tabs.length;
      else if (e.key === "Home") next = 0;
      else if (e.key === "End") next = tabs.length - 1;
      if (next === null) return;
      if (e.preventDefault) e.preventDefault();
      activeSignal.set(next);
      if (tabs[next] && tabs[next].focus) tabs[next].focus();
    });
  }

