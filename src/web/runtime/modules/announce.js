  // ─── Announcements ───────────────────────────────────
  //
  // One polite live region for what changed without a focus change to say
  // so: the page a route change landed on. Created up front, since a live
  // region only announces changes made after it is in the document.
  let announcer = null;

  function announce(text) {
    if (!announcer) {
      announcer = document.createElement("div");
      announcer.className = "wf-visually-hidden";
      announcer.setAttribute("role", "status");
      announcer.setAttribute("aria-live", "polite");
      announcer.setAttribute("aria-atomic", "true");
      document.body.appendChild(announcer);
    }
    // Cleared first, so the same title twice is still read twice.
    announcer.textContent = "";
    setTimeout(() => { announcer.textContent = text; }, 50);
  }
