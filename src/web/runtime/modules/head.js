  // ─── Head ────────────────────────────────────────────
  // A page's `head { meta(…) link(…) script(…) }`: the tags are written into
  // the document's head for as long as the page shows, replacing what the
  // static paint put there; an attribute that reads state follows it.
  //
  // An address goes through `safeUrl` as everywhere else, so a value can
  // never point a `link` or a `script` at `javascript:` or `data:`; and a
  // refresh whose address a browser would run is not written at all. (The
  // compiler refuses a `script(src:)` or a refresh a value chooses; this is
  // the same rule where the value is used.)
  const HEAD_URLS = ["src", "href"];
  function headValue(k, value) {
    if (value === false || value == null) return null;
    if (value === true) return "";
    const text = String(value);
    return HEAD_URLS.includes(k.toLowerCase()) ? safeUrl(text) : text;
  }
  function refreshes(node) {
    if ((node.getAttribute("http-equiv") || "").trim().toLowerCase() !== "refresh") return false;
    const content = node.getAttribute("content") || "";
    const at = content.toLowerCase().indexOf("url");
    if (at < 0) return false;
    const rest = content.slice(at + 3).trim();
    if (rest[0] !== "=") return false;
    const url = rest.slice(1).trim().replace(/^['"]|['"]$/g, "");
    return safeUrl(url) === "" && url !== "";
  }
  function head(tags) {
    for (const old of Array.from(document.head.querySelectorAll("[data-wf-head]"))) old.remove();
    const made = [];
    for (const [tag, attrs] of tags) {
      const node = document.createElement(tag);
      node.setAttribute("data-wf-head", "");
      const set = (k, v) => {
        const value = headValue(k, v);
        if (value == null || (value === "" && HEAD_URLS.includes(k.toLowerCase()) && v !== true)) {
          node.removeAttribute(k);
        } else node.setAttribute(k, value);
        if (refreshes(node)) node.removeAttribute("content");
      };
      for (const [k, v] of Object.entries(attrs || {})) {
        if (typeof v === "function") effect(() => set(k, v()));
        else set(k, v);
      }
      // A script whose address was refused is not a script at all.
      if (tag === "script" && !node.getAttribute("src")) continue;
      document.head.appendChild(node);
      made.push(node);
    }
    onCleanup(() => { for (const n of made) n.remove(); });
  }
