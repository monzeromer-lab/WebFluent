  // ─── Theme ───────────────────────────────────────────
  // `theme` is `"light"`, `"dark"` or `"system"`: what the reader chose,
  // kept in storage and written to `<html data-theme>`, which the sheet's
  // dark rules read; `"system"` leaves it to `prefers-color-scheme`.
  let _theme = null;
  function themeSignal() {
    if (!_theme) {
      let chosen = "system";
      try { chosen = window.localStorage.getItem("wf:theme") || "system"; } catch (e) { /* no storage */ }
      _theme = signal(chosen);
      applyTheme(chosen);
    }
    return _theme;
  }
  function applyTheme(chosen) {
    const root = document.documentElement;
    if (chosen === "light" || chosen === "dark") root.setAttribute("data-theme", chosen);
    else root.removeAttribute("data-theme");
  }
  function theme() {
    return themeSignal()();
  }
  function setTheme(chosen) {
    chosen = chosen === "light" || chosen === "dark" ? chosen : "system";
    applyTheme(chosen);
    try {
      if (chosen === "system") window.localStorage.removeItem("wf:theme");
      else window.localStorage.setItem("wf:theme", chosen);
    } catch (e) { /* no storage */ }
    themeSignal().set(chosen);
  }

  // A page drawn or taken over applies the reader's choice first.
  _onStart.push(themeSignal);
