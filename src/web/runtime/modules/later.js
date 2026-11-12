  // ─── Scripts that wait for the page ─────────────────
  //
  // A `meta.scripts` entry with `"load": "after"` — analytics, a chat
  // widget — is something nothing on the page waits for. A tag in the HTML
  // would have the browser fetch it while it is still fetching the page, and
  // a page's speed is measured against everything asked for before it first
  // paints. So the page asks for it itself, once it has loaded and its first
  // frame is on the screen: a frame's callbacks run just before it is shown,
  // so the second frame's are the first to run after the page has painted.
  // A page opened in a background tab draws no frame until it is shown, and
  // asks for nothing until then either.
  function loadAfter(scripts) {
    const add = () => {
      for (const s of scripts) {
        const el = document.createElement("script");
        el.src = s.src.includes("://") ? s.src : _basePath + "/" + s.src.replace(/^\/+/, "");
        el.async = true;
        if (s.integrity) {
          el.integrity = s.integrity;
          el.crossOrigin = "anonymous";
        }
        (document.head || document.body).appendChild(el);
      }
    };
    const afterPaint = () => {
      if (typeof requestAnimationFrame === "function") {
        requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(add, 0)));
      } else setTimeout(add, 0);
    };
    if (document.readyState === "complete") afterPaint();
    else window.addEventListener("load", afterPaint, { once: true });
  }
