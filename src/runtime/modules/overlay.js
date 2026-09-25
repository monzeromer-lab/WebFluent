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
    if (write === undefined && typeof read.set === "function") {
      const signal = read;
      write = (v) => signal.set(v);
    }
    effect(() => {
      const shouldBeOpen = read();
      if (shouldBeOpen && !el.open) {
        if (el.showModal) el.showModal();
        else el.setAttribute("open", "");
      } else if (!shouldBeOpen && el.open) {
        if (el.close) el.close();
        else el.removeAttribute("open");
      }
    });
    el.addEventListener("close", () => {
      if (write && read()) write(false);
    });
  }

  /// Close a popup on Escape or an outside click, returning focus to its trigger.
  ///
  /// A keyboard user who opens a menu must be able to leave it without tabbing
  /// through every item, and must land back where they were.
  function popup(root, trigger, openSignal) {
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

  /// Wire a menu button and its `role="menu"` list to the ARIA menu pattern.
  ///
  /// The items are menuitems that the arrow keys move between (Home and End
  /// jump to the ends), Enter and Space activate, Escape closes with focus
  /// back on the button, and Tab leaves and closes. Opening from the
  /// keyboard puts focus on the first item; a pointer keeps focus on the
  /// button. `popup` supplies the outside click and Escape.
  function menu(root, trigger, list, openSignal) {
    popup(root, trigger, openSignal);
    const items = () =>
      Array.from(list.children).filter((el) => {
        const cls = el.className || "";
        if (cls.includes("divider") || cls.includes("separator")) {
          el.setAttribute("role", "separator");
          return false;
        }
        if (!el.hasAttribute("role")) el.setAttribute("role", "menuitem");
        if (!el.hasAttribute("tabindex")) el.setAttribute("tabindex", "-1");
        return true;
      });
    const focusItem = (i) => {
      const all = items();
      if (!all.length) return;
      const target = all[((i % all.length) + all.length) % all.length];
      if (target.focus) target.focus();
    };
    trigger.addEventListener("keydown", (e) => {
      if (e.key === "ArrowDown" || e.key === "ArrowUp" || e.key === "Enter" || e.key === " ") {
        if (e.preventDefault) e.preventDefault();
        openSignal.set(true);
        setTimeout(() => focusItem(e.key === "ArrowUp" ? -1 : 0), 0);
      }
    });
    list.addEventListener("keydown", (e) => {
      const all = items();
      const current = all.indexOf(document.activeElement);
      let consumed = true;
      if (e.key === "ArrowDown") focusItem(current + 1);
      else if (e.key === "ArrowUp") focusItem(current - 1);
      else if (e.key === "Home") focusItem(0);
      else if (e.key === "End") focusItem(-1);
      else if ((e.key === "Enter" || e.key === " ") && current >= 0) all[current].click();
      else if (e.key === "Tab") openSignal.set(false), (consumed = false);
      else consumed = false;
      if (consumed && e.preventDefault) e.preventDefault();
    });
    // Choosing an item closes the menu; the reader lands back on the button.
    list.addEventListener("click", (e) => {
      if (items().some((item) => item.contains(e.target))) {
        openSignal.set(false);
        if (trigger.focus) trigger.focus();
      }
    });
    items();
  }

  /// Arrow-key navigation for a `role="tabs"`.
  ///
  /// The WAI-ARIA pattern puts only the selected tab in the tab order and moves
  /// between tabs with the arrow keys, so Tab leaves the widget rather than
  /// walking through every tab in it.
  function tabs(nav, activeSignal) {
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
