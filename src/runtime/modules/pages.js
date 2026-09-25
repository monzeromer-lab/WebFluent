  // ─── Pages (route chunks) ────────────────────────────
  //
  // A build writes each page as `pages/<Name>.js`, which registers itself
  // here when it runs; app.js holds the runtime, the stores and the
  // components every page shares. A page's HTML links its own chunk beside
  // app.js, so a static build loads the two in parallel; a route change in a
  // single-page build fetches the chunk the first time the route shows.
  const pages = {};
  const waiting = {};

  function page(name, renderFn) {
    pages[name] = renderFn;
    const callbacks = waiting[name];
    delete waiting[name];
    if (callbacks) for (const cb of callbacks) cb(renderFn);
  }

  function loadPage(name, cb) {
    if (pages[name]) {
      cb(pages[name]);
      return;
    }
    (waiting[name] = waiting[name] || []).push(cb);
    // Already linked by the page's HTML, or already requested: it will
    // register itself when it runs.
    if (document.querySelector('script[data-wf-page="' + name + '"]')) return;
    const script = document.createElement("script");
    script.src = _basePath + "/pages/" + name + ".js";
    script.async = true;
    script.setAttribute("data-wf-page", name);
    script.onerror = () => console.error("WebFluent: could not load the page chunk for " + name);
    (document.head || document.body).appendChild(script);
  }

  // A page's own stylesheet, `pages/<Name>.css`, linked the first time its
  // route shows; `cb` runs once the rules apply. A sheet the page's HTML
  // already linked applied before this script ran.
  const sheets = {};
  function loadSheet(name, cb) {
    if (sheets[name]) return cb();
    const linked = document.querySelector('link[data-wf-page-css="' + name + '"]');
    if (linked) {
      sheets[name] = true;
      return cb();
    }
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = _basePath + "/pages/" + name + ".css";
    link.setAttribute("data-wf-page-css", name);
    const done = () => {
      sheets[name] = true;
      cb();
    };
    link.onload = done;
    link.onerror = () => {
      console.error("WebFluent: could not load the stylesheet for " + name);
      done();
    };
    (document.head || document.body).appendChild(link);
  }

  // The classes an expression names, kept in step with it: what it named
  // last time and no longer does is taken off, what it names now is added,
  // and the element's other classes are left alone.
