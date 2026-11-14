  // ─── More navigation ─────────────────────────────────
  //
  // A site's navigation that opens panels, an app's menu bar, and a menu a
  // right-click or a long press opens where it happened. Each is drawn by
  // the compiler as elements; these give them their keys and their timing.

  /// `NavigationMenu`: its top-level controls are one row the arrow keys go
  /// along (the other way round right to left), Home and End to its ends;
  /// a trigger opens its panel as a disclosure — one at a time — and
  /// Escape, a press outside, the focus leaving or a link chosen in the
  /// panel closes it.
  function navMenu(nav) {
    live(nav);
    const mine = (n) => n && n.closest && n.closest(".wf-nav-menu") === nav;
    const triggers = () => [...nav.querySelectorAll(".wf-nav-menu__trigger")].filter(mine);
    const tops = () => [...nav.querySelectorAll(".wf-nav-menu__item")]
      .filter(mine)
      .map((item) => item.querySelector(".wf-nav-menu__trigger, a[href]"))
      .filter((c) => c && c.closest(".wf-nav-menu__panel") === null);
    const panelOf = (t) => {
      const p = t.nextElementSibling;
      return p && p.classList.contains("wf-nav-menu__panel") ? p : null;
    };
    const ready = (t) => {
      const p = panelOf(t);
      t.setAttribute("type", "button");
      if (!p) return null;
      if (!p.id) p.id = uid("wf-nav-panel");
      t.setAttribute("aria-controls", p.id);
      return p;
    };
    const set = (t, open) => {
      const p = ready(t);
      t.setAttribute("aria-expanded", open ? "true" : "false");
      if (p) p.hidden = !open;
    };
    const shut = (except) => {
      for (const t of triggers()) if (t !== except && t.getAttribute("aria-expanded") === "true") set(t, false);
    };
    const openOne = () => triggers().find((t) => t.getAttribute("aria-expanded") === "true");
    triggers().forEach((t) => set(t, false));
    nav.addEventListener("click", (e) => {
      const t = e.target.closest && e.target.closest(".wf-nav-menu__trigger");
      if (t && mine(t)) {
        const open = t.getAttribute("aria-expanded") !== "true";
        shut(t);
        set(t, open);
        return;
      }
      // A link chosen in a panel goes where it says, and the panel goes.
      if (e.target.closest && e.target.closest(".wf-nav-menu__panel a[href]")) shut(null);
    });
    nav.addEventListener("keydown", (e) => {
      if (e.altKey || e.ctrlKey || e.metaKey) return;
      if (e.key === "Escape") {
        const t = openOne();
        if (!t) return;
        set(t, false);
        t.focus();
        e.preventDefault();
        return;
      }
      const row = tops();
      const at = row.indexOf(e.target);
      if (at < 0) return;
      const rtl = _isRtl(nav);
      let to = -1;
      if (e.key === (rtl ? "ArrowLeft" : "ArrowRight")) to = (at + 1) % row.length;
      else if (e.key === (rtl ? "ArrowRight" : "ArrowLeft")) to = (at - 1 + row.length) % row.length;
      else if (e.key === "Home") to = 0;
      else if (e.key === "End") to = row.length - 1;
      else if (e.key === "ArrowDown" && e.target.classList.contains("wf-nav-menu__trigger")) {
        // Into the panel, at its first link.
        shut(e.target);
        set(e.target, true);
        const first = _firstStop(panelOf(e.target), true);
        if (first) first.focus();
        e.preventDefault();
        return;
      }
      if (to < 0) return;
      e.preventDefault();
      row[to].focus();
    });
    // The focus leaving the navigation takes its panel with it.
    nav.addEventListener("focusout", (e) => {
      if (e.relatedTarget && !nav.contains(e.relatedTarget)) shut(null);
    });
    const outside = (e) => {
      if (!nav.contains(e.target)) shut(null);
    };
    document.addEventListener("pointerdown", outside, true);
    onCleanup(() => document.removeEventListener("pointerdown", outside, true));
  }

  /// `Menubar`: its menus' buttons are one stop for Tab, the arrow keys
  /// along it; with a menu open, Left and Right open the one beside it, and
  /// a pointer moved onto another button opens that one.
  function menuBar(bar) {
    live(bar);
    const triggers = () => [...bar.querySelectorAll(".wf-menubar__trigger")].filter((t) => t.closest(".wf-menubar") === bar);
    const rove = (on) => {
      for (const t of triggers()) t.tabIndex = t === on ? 0 : -1;
    };
    const openOne = () => triggers().find((t) => t.getAttribute("aria-expanded") === "true");
    // The arrow down a button hears opens its menu at its first item.
    const into = (t) => t.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: false }));
    rove(triggers()[0]);
    bar.addEventListener("focusin", (e) => {
      if (triggers().includes(e.target)) rove(e.target);
    });
    bar.addEventListener("keydown", (e) => {
      if (e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
      const row = triggers();
      let from = row.indexOf(e.target);
      let inMenu = false;
      if (from < 0) {
        // In one of its menus — not a submenu, whose list sits in its item.
        const list = e.target.closest && e.target.closest('[role="menu"]');
        const wrap = list && list.parentElement;
        if (!wrap || !wrap.classList.contains("wf-menubar__menu")) return;
        from = row.indexOf(wrap.querySelector(".wf-menubar__trigger"));
        inMenu = true;
      }
      if (from < 0) return;
      const rtl = _isRtl(bar);
      let to = -1;
      if (e.key === (rtl ? "ArrowLeft" : "ArrowRight")) to = (from + 1) % row.length;
      else if (e.key === (rtl ? "ArrowRight" : "ArrowLeft")) to = (from - 1 + row.length) % row.length;
      else if (!inMenu && e.key === "Home") to = 0;
      else if (!inMenu && e.key === "End") to = row.length - 1;
      if (to < 0) return;
      e.preventDefault();
      const wasOpen = inMenu || !!openOne();
      if (row[from].getAttribute("aria-expanded") === "true") row[from].click();
      rove(row[to]);
      row[to].focus();
      if (wasOpen) into(row[to]);
    });
    bar.addEventListener("pointerover", (e) => {
      const t = e.target.closest && e.target.closest(".wf-menubar__trigger");
      if (!t || t.closest(".wf-menubar") !== bar) return;
      const open = openOne();
      if (!open || open === t) return;
      open.click();
      t.click();
    });
  }

  /// `ContextMenu`: a right-click, a long press, the menu key or Shift+F10
  /// in `root` opens `list` where it happened (at the focused element for a
  /// key), focus on its first item.
  function contextMenu(root, list, opts) {
    live(root, list);
    const shown = signal(false);
    let at = null;
    const open = (point) => {
      at = point;
      if (shown()) shown.set(false);
      shown.set(true);
    };
    menu(root, null, list, shown, (v) => shown.set(v), Object.assign({ anchor: () => at, placement: "bottom-start", offset: 0 }, opts || {}));
    root.addEventListener("contextmenu", (e) => {
      if (list.contains(e.target)) return;
      e.preventDefault();
      open({ x: e.clientX, y: e.clientY });
    });
    // A finger held still: a touch screen without a `contextmenu` of its own.
    gesture(root, "longpress", (p) => {
      if (p.pointer !== "mouse") open({ x: p.x, y: p.y });
    });
    root.addEventListener("keydown", (e) => {
      if (list.contains(e.target)) return;
      if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
        e.preventDefault();
        open(document.activeElement && document.activeElement !== document.body ? document.activeElement : root);
      }
    });
  }
