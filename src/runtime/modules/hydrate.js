  // ─── Hydrate (SSG) ─────────────────────────────────
  function hydrate(renderFn, container) {
    // The server paint is the FIRST paint — real HTML, instantly visible and
    // crawlable — and the client render replaces it.
    //
    // This used to keep the pre-rendered DOM and call renderFn purely for its
    // side effects, discarding the nodes it built. But effects and event
    // listeners are bound to the nodes the render CREATES, so they were all
    // attached to detached elements: reactive text never updated what you could
    // see, and buttons on any pre-rendered page did nothing at all. Reusing the
    // server's DOM needs node matching this runtime does not have, so until it
    // does, replacing it is the behaviour that is actually correct.
    mount(renderFn, container);
  }
