  // ─── Head ────────────────────────────────────────────
  // A page's `head { meta(…) link(…) script(…) }`: the tags are written into
  // the document's head for as long as the page shows, replacing what the
  // static paint put there; an attribute that reads state follows it.
  function head(tags) {
    for (const old of Array.from(document.head.querySelectorAll("[data-wf-head]"))) old.remove();
    const made = [];
    for (const [tag, attrs] of tags) {
      const node = document.createElement(tag);
      node.setAttribute("data-wf-head", "");
      for (const [k, v] of Object.entries(attrs || {})) {
        if (typeof v === "function") {
          effect(() => {
            const value = v();
            if (value === false || value == null) node.removeAttribute(k);
            else node.setAttribute(k, value === true ? "" : String(value));
          });
        } else if (v === true) node.setAttribute(k, "");
        else if (v !== false && v != null) node.setAttribute(k, String(v));
      }
      document.head.appendChild(node);
      made.push(node);
    }
    onCleanup(() => { for (const n of made) n.remove(); });
  }
