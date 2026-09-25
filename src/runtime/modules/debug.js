  // ─── Exports ─────────────────────────────────────────
  // ─── Studio debug hook (Slice 3 §1.4) ────────────────
  // Ground truth for the studio's e2e assertions and selection sync. tree/query*/
  // dispatch introspect the rendered DOM (via data-wf-node from Slice 2) and need
  // no cooperation from generated code; state() reads a registry that studio-mode
  // codegen populates through __reg.
  const __signals = new Map();
  function __reg(name, sig) {
    __signals.set(name, sig);
    return sig;
  }
  __regHook = __reg;

  function __directText(el) {
    let t = "";
    for (const n of el.childNodes) {
      if (n.nodeType === 3) t += n.textContent;
    }
    return t.trim();
  }

  const __debug = {
    /// Snapshot of registered reactive signals (page/store state + derived).
    /// `scope` optionally filters by an id prefix.
    state(scope) {
      const out = {};
      for (const [name, sig] of __signals) {
        if (scope && !name.startsWith(scope)) continue;
        try {
          out[name] = typeof sig === "function" ? sig() : sig;
        } catch (_e) {
          out[name] = undefined;
        }
      }
      return out;
    },
    /// The rendered node tree keyed by data-wf-node.
    tree(root) {
      const start = root || document.getElementById("app") || document.body;
      const walk = (el) => {
        const nodes = [];
        for (const child of el.children) {
          nodes.push({
            id: child.getAttribute("data-wf-node") || null,
            tag: child.tagName.toLowerCase(),
            text: __directText(child),
            children: walk(child),
          });
        }
        return nodes;
      };
      return walk(start);
    },
    /// Synthesize a user event on the node with the given id. Returns whether the
    /// node was found.
    dispatch(nodeId, event, payload) {
      const el = document.querySelector('[data-wf-node="' + nodeId + '"]');
      if (!el) return false;
      if (event === "click") {
        el.click();
        return true;
      }
      if (event === "input" || event === "change") {
        if (payload != null && "value" in el) el.value = payload;
        el.dispatchEvent(new Event(event, { bubbles: true }));
        return true;
      }
      el.dispatchEvent(new Event(event, { bubbles: true }));
      return true;
    },
    /// Node ids whose text contains `text` (for assertions).
    queryText(text) {
      const out = [];
      for (const el of document.querySelectorAll("[data-wf-node]")) {
        if (el.textContent && el.textContent.includes(text)) {
          out.push(el.getAttribute("data-wf-node"));
        }
      }
      return out;
    },
    /// Node ids carrying the given ARIA/role attribute.
    queryRole(role) {
      return Array.from(document.querySelectorAll('[data-wf-node][role="' + role + '"]'))
        .map((el) => el.getAttribute("data-wf-node"));
    },
  };
