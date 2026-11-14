  // ─── Announcements ───────────────────────────────────
  //
  // One polite live region, for what changed with no change of focus to say
  // so: the page a route change landed on, a form's first fault, a toast.
  //
  // A live region announces only what changes after it is in the document,
  // so the region is made when the runtime loads rather than on the first
  // message, which would otherwise be the one lost. And an open modal
  // dialog makes everything outside it inert, a live region included: while
  // one is open, the region (and whatever else asked, such as the toasts)
  // moves into the topmost one, and back out when it closes.
  let announcer = null;
  const _homed = [];
  const _modals = [];

  /// Where something that must stay heard belongs now: the topmost open
  /// modal dialog, or the page.
  function _liveHost() {
    for (let i = _modals.length - 1; i >= 0; i--) {
      const d = _modals[i];
      if (d.open && d.isConnected !== false) return d;
    }
    return document.body;
  }

  function _rehome() {
    if (typeof document === "undefined" || !document.body) return;
    const host = _liveHost();
    for (const h of _homed) {
      if (h.node.parentNode !== host) {
        host.appendChild(h.node);
        if (h.moved) h.moved();
      }
    }
  }

  /// Keep `node` where it stays heard, calling `moved` after each move.
  function _keepHeard(node, moved) {
    _homed.push({ node, moved });
    _rehome();
  }

  function _watchModals() {
    if (typeof document === "undefined" || !document.addEventListener) return;
    const seen = (e) => {
      const d = e.target;
      if (!d || d.tagName !== "DIALOG") return;
      const at = _modals.indexOf(d);
      if (at >= 0) _modals.splice(at, 1);
      const modal = d.open && (!d.matches || d.matches(":modal"));
      if (modal && e.type !== "close") _modals.push(d);
      _rehome();
    };
    // `toggle` comes when a dialog opens or closes; `close` too, where a
    // browser fires no `toggle` for a dialog. Neither bubbles, so both
    // are heard on the way down.
    document.addEventListener("toggle", seen, true);
    document.addEventListener("close", seen, true);
  }

  let assertive = null;

  function _region(urgent) {
    if (typeof document === "undefined" || !document.body) return null;
    if (urgent) {
      if (!assertive) {
        assertive = document.createElement("div");
        assertive.className = "wf-visually-hidden";
        assertive.setAttribute("aria-live", "assertive");
        assertive.setAttribute("aria-atomic", "true");
        _keepHeard(assertive);
      }
      return assertive;
    }
    if (!announcer) {
      announcer = document.createElement("div");
      announcer.className = "wf-visually-hidden";
      announcer.setAttribute("role", "status");
      announcer.setAttribute("aria-live", "polite");
      _keepHeard(announcer);
    }
    return announcer;
  }

  function _announcer() {
    _region(false);
    _region(true);
  }

  function _bootAnnouncer() {
    _watchModals();
    if (typeof document === "undefined") return;
    if (document.body) _announcer();
    else if (document.addEventListener) document.addEventListener("DOMContentLoaded", _announcer);
  }
  _bootAnnouncer();

  /// Say `text` to a screen reader, with nothing on screen moving:
  /// politely, after what it is reading, or with `"assertive"` at once,
  /// for what cannot wait (a connection lost, a save that failed).
  function announce(text, politeness) {
    const region = _region(politeness === "assertive");
    if (!region) return;
    _rehome();
    // Each message is a new line, added a moment after the region is
    // settled, so the same words twice are read twice. A screen reader
    // queues a line when it is added; the one before it is cleared then.
    setTimeout(() => {
      while (region.firstChild) region.removeChild(region.firstChild);
      const line = document.createElement("div");
      line.textContent = text == null ? "" : String(text);
      region.appendChild(line);
    }, 50);
  }
