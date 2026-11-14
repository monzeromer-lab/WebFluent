  // ── Nodes: what a branch, a list and a route need to put nodes in and
  // take them out. An exit animation, when one is marked, is motion's.

  // A reader who asked for less motion gets none: nothing starts, and
  // everything that waits on it is already finished. An editor previewing
  // that asks for it on the page's root.
  // One query, asked again: a keyed `for` asks whenever its items move,
  // and each `matchMedia` is a list the browser keeps and evaluates again
  // on every change of style until it is collected.
  let reduceQuery = null;

  function reducedMotion() {
    if (document.documentElement.getAttribute("data-wf-motion") === "reduce") return true;
    if (typeof window.matchMedia !== "function") return false;
    reduceQuery ??= window.matchMedia("(prefers-reduced-motion: reduce)");
    return reduceQuery.matches;
  }

  /// Milliseconds from whatever the author wrote: `"200ms"`, `"0.2s"`, a
  /// number, or nothing.
  function millis(value, fallback) {
    if (value == null || value === "") return fallback;
    if (typeof value === "number") return value;
    const text = String(value).trim();
    const n = parseFloat(text);
    if (!Number.isFinite(n)) return fallback;
    return text.endsWith("ms") ? n : text.endsWith("s") ? n * 1000 : n;
  }

  // ─── Regions ─────────────────────────────────────────
  // An `if`, a `match`, a slot and a list each draw between two markers of
  // their own, and what one holds is what lies between them — read from
  // the page when it is needed, never remembered from when it was drawn.
  // One at the top of another's body draws into the same parent, between
  // the outer one's markers, so what it drew since goes with the rest. (A
  // branch used to keep the list of nodes it drew; what a branch inside it
  // drew later stayed on the page after both had gone.)
  function region(parent, name) {
    const start = document.createComment(name);
    const end = document.createComment("/" + name);
    start.__wfEnd = end;
    parent.appendChild(start);
    parent.appendChild(end);
    return start;
  }

  /// What lies between a region's markers now. An end that is not after
  /// the start takes nothing, rather than everything that follows.
  function between(start) {
    const out = [];
    for (let n = start.nextSibling; n; n = n.nextSibling) {
      if (n === start.__wfEnd) return out;
      out.push(n);
    }
    return [];
  }

  /// What a drawing made, kept to be read again later: a region among the
  /// nodes is kept by its first marker alone, since what lies between its
  /// markers changes.
  function keepNodes(nodes) {
    const out = [];
    for (let i = 0; i < nodes.length; i++) {
      out.push(nodes[i]);
      const end = nodes[i] && nodes[i].__wfEnd;
      const j = end ? nodes.indexOf(end, i + 1) : -1;
      if (j > i) i = j;
    }
    return out;
  }

  /// The nodes `keepNodes` kept, as they are in the page now: a region's
  /// first marker stands for both its markers and everything between them.
  function nodesNow(nodes) {
    const out = [];
    for (const drawn of nodes) {
      const n = inPage(drawn);
      out.push(n);
      if (n && n.__wfEnd) out.push(...between(n), n.__wfEnd);
    }
    return out;
  }

  // ─── Leaving ─────────────────────────────────────────
  // Every exit the nodes on their way out asked for: the branch's own, on
  // each root, and the one any element beneath carries as `data-wf-exit`.
  // The promises resolve when the last has played; none means remove now.
  function leave(nodes, config) {
    const plays = [];
    // Without motion in the build nothing was marked to leave; a mark that
    // came in with markup the program did not write is left unplayed.
    if (reducedMotion() || typeof animateOut !== "function") return plays;
    const marked = (el) => {
      if (!el.hasAttribute || !el.hasAttribute("data-wf-exit")) return;
      const attr = (name) => el.getAttribute(name) || "";
      plays.push(animateOut(el, attr("data-wf-exit"), attr("data-wf-duration"), attr("data-wf-delay"), attr("data-wf-easing")));
    };
    const roots = [];
    for (const drawn of nodes) {
      const n = inPage(drawn);
      if (!(n instanceof Element)) continue;
      roots.push(n);
      if (config && config.exit) plays.push(animateOut(n, config.exit, config.duration, "", config.easing));
      else marked(n);
      if (typeof n.querySelectorAll === "function") {
        for (const el of n.querySelectorAll("[data-wf-exit]")) marked(el);
      }
    }
    // What is on its way out takes no press and no focus while its exit
    // plays: a click on a closing card, a Tab into a leaving row, lands
    // nowhere. One that comes back (a `show`) is given them again.
    if (plays.length) {
      for (const n of roots) {
        if (n.contains && n.contains(document.activeElement) && document.activeElement.blur) document.activeElement.blur();
        n.inert = true;
      }
    }
    return plays;
  }

  // Remove `nodes` once their exits have played, then call `onDone`. The
  // handle returned (none when nothing played) is cancelled by a branch
  // that comes back before the exit is over.
  function leaveThenRemove(nodes, config, onDone) {
    const plays = leave(nodes, config);
    if (!plays.length) {
      removeNodes(nodes);
      if (onDone) onDone();
      return null;
    }
    const pending = { cancelled: false };
    Promise.all(plays).then(() => {
      if (pending.cancelled) return;
      removeNodes(nodes);
      if (onDone) onDone();
    });
    return pending;
  }
