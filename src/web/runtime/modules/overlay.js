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
  //
  // `options`:
  //   dismiss  "any" (the default): Escape or a press on the backdrop
  //            closes it; "escape": Escape only; "none": neither, so only
  //            the page's own button does — a form that must be answered
  //   initial  a read of what takes the focus when it opens (an element or
  //            a `ref:` handle); unset, the browser's choice: what says
  //            `autofocus`, else the first thing that takes the focus
  //
  // When it closes, the focus goes back to what had it when it opened, if
  // that is still on the page and the focus was left in the dialog or on
  // nothing. Its exit is the stylesheet's (`allow-discrete`), so Escape,
  // the backdrop and `visible:` all play it.
  function dialog(el, read, write, options) {
    live(el);
    if (write === undefined && typeof read.set === "function") {
      const signal = read;
      write = (v) => signal.set(v);
    }
    const opts = options || {};
    const dismiss = opts.dismiss || "any";
    // A modal open from the start is drawn before its page is in the
    // document, and `showModal()` throws on a dialog that is not — which
    // stopped the whole page drawing. It opens once it is there.
    let wanted = false;
    // What had the focus when it opened, which gets it back when it shuts.
    let before = null;
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
    const restore = () => {
      const back = before;
      before = null;
      if (!back || back.isConnected === false || typeof back.focus !== "function") return;
      const now = typeof document !== "undefined" ? document.activeElement : null;
      if (!now || now === document.body || (el.contains && el.contains(now))) back.focus();
    };
    const open = (tries) => {
      if (!wanted || el.open) return;
      if (el.isConnected === false) {
        if (tries > 0) requestAnimationFrame(() => open(tries - 1));
        return;
      }
      before = typeof document !== "undefined" ? document.activeElement : null;
      if (el.showModal) el.showModal();
      else el.setAttribute("open", "");
      stacked();
      const want = opts.initial ? opts.initial() : null;
      const target = want && want.el !== undefined ? want.el : want;
      if (target && typeof target.focus === "function") target.focus();
    };
    const shut = () => {
      if (el.close) el.close();
      else el.removeAttribute("open");
      unstacked();
      restore();
    };
    effect(() => {
      const shouldBeOpen = read();
      wanted = !!shouldBeOpen;
      if (shouldBeOpen && !el.open) {
        if (el.isConnected !== false) open(0);
        else queueMicrotask(() => open(60));
      } else if (!shouldBeOpen && el.open) {
        shut();
      }
    });
    el.addEventListener("close", () => {
      // The event comes a task after the dialog shut. If it was opened
      // again in between — the close button wrote the state at once, and
      // a press set it back — this one is stale: writing `false` now
      // would shut the dialog the reader just opened.
      if (el.open) return;
      unstacked();
      restore();
      if (write && read()) write(false);
    });
    // `.closable`'s button: shut it and write the state now. The `close`
    // event comes a task later, and a press in between (open it again)
    // would set a state that still said open, and be lost.
    el._wfShut = () => {
      shut();
      if (write && read()) write(false);
    };
    onCleanup(unstacked);
    // `closedby` tells a browser that knows it which requests close the
    // dialog; the handlers below do the same where it does not.
    if (el.setAttribute) el.setAttribute("closedby", dismiss === "any" ? "any" : dismiss === "escape" ? "closerequest" : "none");
    // A press on the backdrop closes it — light dismiss, as a reader
    // expects of anything drawn over the page. The dialog's box is exactly
    // its card, and a press on the backdrop lands on the dialog itself, so
    // a press outside the box that also began there (not a text selection
    // dragged out of the card) is the backdrop's.
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
      if (close && el.open && dismiss === "any") el._wfShut();
    });
    // A dialog that must be answered keeps its Escape: the browser's close
    // request is refused.
    if (dismiss === "none") {
      el.addEventListener("cancel", (e) => { if (e.preventDefault) e.preventDefault(); });
    }
    // The Escape that closes it is its own: nothing behind it — a page's
    // `on key("Escape")`, the back rule of an app — hears it too. The
    // browser still closes the dialog (its `cancel` is not prevented).
    el.addEventListener("keydown", (e) => {
      if (e.key !== "Escape" || !el.open) return;
      if (e.stopPropagation) e.stopPropagation();
      if (dismiss === "none" && e.preventDefault) e.preventDefault();
    });
    return { close: () => el._wfShut() };
  }

  /// `await ask("Delete the project?", { detail, yes, no, danger })`: an
  /// alert dialog with two buttons, and a promise of the answer — `true`
  /// for yes, `false` for no, Escape, or the page leaving. The focus starts
  /// on no, the answer that loses nothing; a press on the backdrop is not
  /// an answer. (The browser's own `confirm` is a different function.)
  let _asked = 0;
  function ask(message, options) {
    const opts = options || {};
    return new Promise((resolve) => {
      if (typeof document === "undefined" || !document.body) {
        resolve(false);
        return;
      }
      _asked += 1;
      const id = `wf-ask-${_asked}`;
      const open = signal(true);
      let answer = false;
      const no = el("button", { className: "wf-btn", type: "button" }, opts.no == null ? "Cancel" : String(opts.no));
      const yes = el("button", { className: `wf-btn wf-btn--${opts.danger ? "danger" : "primary"}`, type: "button" }, opts.yes == null ? "OK" : String(opts.yes));
      const parts = [el("div", { className: "wf-dialog__header" }, el("h3", { id: `${id}-title` }, String(message)))];
      if (opts.detail != null) parts.push(el("p", { id: `${id}-detail`, className: "wf-dialog__detail" }, String(opts.detail)));
      parts.push(el("div", { className: "wf-dialog__footer" }, no, yes));
      const box = el("dialog", {
        className: "wf-dialog wf-alert-dialog",
        role: "alertdialog",
        "aria-labelledby": `${id}-title`,
        "aria-describedby": opts.detail != null ? `${id}-detail` : null,
      }, el("div", { className: "wf-dialog__content" }, ...parts));
      document.body.appendChild(box);
      no.addEventListener("click", () => open.set(false));
      yes.addEventListener("click", () => {
        answer = true;
        open.set(false);
      });
      box.addEventListener("close", () => {
        resolve(answer);
        // Gone once its exit has played.
        setTimeout(() => box.remove(), 400);
      });
      dialog(box, () => open(), (v) => open.set(v), { dismiss: "escape", initial: () => no });
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

  /// A tab strip's keys: the WAI-ARIA pattern, through `roving` — only the
  /// selected tab is in the tab order, the arrow keys move between tabs
  /// (the other way round across a right-to-left page), Home and End go
  /// to the ends, and a disabled tab is passed over. `read` is the selected
  /// tab's index and `write` selects one; with `manual`, moving the focus
  /// does not select (Enter or Space does, as the tab's click).
  function tabs(nav, read, write, options) {
    if (write === undefined && read && typeof read.set === "function") {
      const signal = read;
      read = () => signal();
      write = (i) => signal.set(i);
    }
    const opts = options || {};
    const all = () => Array.from(nav.querySelectorAll('[role="tab"]'));
    return roving(nav, {
      items: all,
      axis: opts.vertical ? "y" : "x",
      active: read,
      onMove: (i) => {
        if (!opts.manual && write) write(i);
      },
    });
  }


