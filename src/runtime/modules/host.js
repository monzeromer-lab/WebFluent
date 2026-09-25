  // ─── Somebody else's code ────────────────────────────
  //
  // A chart, a map, an editor: a library that wants a node and gives back
  // a handle. The pattern people write by hand is a `ref`, an `effect` and
  // a `cleanup`, and the part they forget is the cleanup — so the map is
  // still listening after the route changed.
  //
  // `Host` makes the three one thing. The mount runs once; the update runs
  // again whenever the state it reads changes; the cleanup is tied to the
  // scope that owns the element, so it cannot be forgotten.

  /// Hand `node` to `mount`, keep it in step with `update`, and give it
  /// back with `cleanup` when what owns it leaves.
  ///
  /// `attach` rather than `host`, because `host` is a `Url`'s.
  function attach(node, mount, update, cleanup) {
    let handle;
    try {
      handle = typeof mount === "function" ? mount(node) : undefined;
    } catch (e) {
      // A library that throws on mount takes itself out; the page stays.
      console.error("Host: mount failed", e);
      return undefined;
    }
    if (typeof update === "function") {
      // The first run is the mount's: an update that ran immediately would
      // be a second call before the library had settled.
      let first = true;
      effect(() => {
        if (first) {
          // Read what the update reads, so the effect subscribes to it,
          // without acting on it yet.
          first = false;
          try { update(handle); } catch (e) { console.error("Host: update failed", e); }
          return;
        }
        try { update(handle); } catch (e) { console.error("Host: update failed", e); }
      });
    }
    if (typeof cleanup === "function") {
      onCleanup(() => {
        try { cleanup(handle); } catch (e) { console.error("Host: cleanup failed", e); }
      });
    }
    return handle;
  }
