  function _routeOf(path) {
    return String(path).split(/[?#]/)[0];
  }

  function activeLink(el, href, prefix) {
    const target = _routeOf(href).replace(/\/$/, "") || "/";
    effect(() => {
      const path = pathSignal()().replace(/\/$/, "") || "/";
      const on = path === target || (prefix && target !== "/" && path.startsWith(target + "/"));
      if (on) {
        el.classList.add("active");
        el.setAttribute("aria-current", "page");
      } else {
        el.classList.remove("active");
        el.removeAttribute("aria-current");
      }
    });
  }

  // `options.transition` — `fade` or `slide` — plays the old page out and
  // the new one in on a route change; `options.duration` times both.
  function router(routes, container, options) {
    const motion = options && options.transition && options.transition !== "none"
      ? { name: options.transition, duration: options.duration }
      : null;
    const ways = { fade: ["fadeOut", "fadeIn"], slide: ["slideLeft", "slideRight"] };
    // Check for SPA redirect from 404.html (?p=/path)
    const urlParams = new URLSearchParams(window.location.search);
    const redirectPath = urlParams.get("p");
    if (redirectPath) {
      window.history.replaceState(null, "", _basePath + redirectPath);
    }

    const initialPath = _stripBase(window.location.pathname);
    const currentPath = pathSignal();
    currentPath.set(initialPath);

    function matchRoute(path) {
      path = _routeOf(path);
      for (const route of routes) {
        const params = matchPath(route.path, path);
        if (params !== null) return { route, params };
      }
      // Try wildcard
      const wild = routes.find(r => r.path === "*");
      if (wild) return { route: wild, params: {} };
      return null;
    }

    function matchPath(pattern, path) {
      if (pattern === path) return {};
      const patternParts = pattern.split("/").filter(Boolean);
      const pathParts = path.split("/").filter(Boolean);
      if (patternParts.length !== pathParts.length) return null;

      const params = {};
      for (let i = 0; i < patternParts.length; i++) {
        if (patternParts[i].startsWith(":")) {
          params[patternParts[i].slice(1)] = pathParts[i];
        } else if (patternParts[i] !== pathParts[i]) {
          return null;
        }
      }
      return params;
    }

    // Whether the route change came from the back/forward buttons, whose
    // scroll position the browser restores itself.
    let fromHistory = false;
    let rendered = false;
    // What the page on show created, disposed of before the next one.
    let disposePage = null;

    function render() {
      const path = currentPath(); // Only subscribe to path changes
      const match = matchRoute(path);
      if (!match) {
        container.innerHTML = "";
        return;
      }

      // A guarded route the reader may not see sends them where its
      // `redirect:` says, instead of painting.
      if (typeof match.route.guard === "function") {
        let allowed = false;
        const prev = currentEffect;
        currentEffect = null;
        try { allowed = !!match.route.guard(); } finally { currentEffect = prev; }
        if (!allowed) {
          const to = match.route.redirect || "/";
          if (to !== path) {
            window.history.replaceState(null, "", _basePath + to);
            currentPath.set(to);
          }
          return;
        }
      }
      const paint = (renderFn) => {
        if (disposePage) { disposePage(); disposePage = null; }
        container.innerHTML = "";
        _newPage();
        // A `store X(scope: .route)` belongs to the page that read it: the
        // route it was for has gone, so what it held goes with it. The
        // `typeof` is how a router reaches a module it does not depend on —
        // a site with no stores carries none of that code.
        if (typeof dropRouteStores === "function") dropRouteStores();
        // The tab, the history entry and a screen reader all read the title;
        // a single-page app used to keep the entry page's title on every route.
        if (match.route.title) document.title = match.route.title;
        // Untrack: don't subscribe the router effect to signals read during page render
        const prev = currentEffect;
        currentEffect = null;
        try {
          // A page that names a layout is rendered inside it.
          const [el, dispose] = scoped(() => match.route.layout
            ? match.route.layout(renderFn, match.params)
            : renderFn(match.params));
          disposePage = dispose;
          if (el instanceof Node) container.appendChild(el);
        } finally {
          currentEffect = prev;
        }
      };
      const settle = () => {
        // A full page load lands the reader at the top with focus on the
        // document; a route change used to leave focus on a link that no
        // longer existed, the scroll wherever it was, and say nothing. It now
        // does what the page load does: focus moves to the new page's heading
        // (or the main landmark when it has none), the viewport returns to the
        // top, and the title is announced.
        if (rendered) {
          settleOnNewPage(container, !fromHistory);
        }
        rendered = true;
        fromHistory = false;
      };
      const draw = (renderFn) => {
        // The page arrived after the reader had already moved on.
        if (currentPath() !== path) return;
        const [out, back] = motion ? ways[motion.name] || ways.fade : [];
        // The first paint, and every one for a reader who asked for less
        // motion, is immediate; a route change plays the old page out, then
        // the new one in, and settles focus once the new page is still.
        if (!motion || !rendered || reducedMotion()) {
          paint(renderFn);
          settle();
          return;
        }
        // Where the browser has the View Transitions API, it plays the
        // change between the two paints itself — a crossfade, or the slide
        // the sheet defines for `data-wf-transition="slide"` — and the
        // class-based animation below is the fallback.
        if (typeof document.startViewTransition === "function") {
          const root = document.documentElement;
          root.setAttribute("data-wf-transition", motion.name);
          if (motion.duration) root.style.setProperty("--animation-duration-normal", motion.duration);
          // The browser runs the callback once it has captured the old
          // page, and capturing needs a painted frame. A page that is not
          // painting — a background tab, a prerender — would never be
          // handed one, so the route change is drawn anyway when the
          // transition does not begin. A navigation is not decoration.
          let painted = false;
          const drawOnce = () => {
            if (painted) return;
            painted = true;
            paint(renderFn);
          };
          let transition;
          try {
            transition = document.startViewTransition(drawOnce);
          } catch (e) {
            root.removeAttribute("data-wf-transition");
            drawOnce();
            settle();
            return;
          }
          const done = () => {
            root.removeAttribute("data-wf-transition");
            if (motion.duration) root.style.removeProperty("--animation-duration-normal");
            drawOnce();
            settle();
          };
          if (transition && transition.finished && typeof transition.finished.then === "function") {
            transition.finished.then(done, done);
            // A skipped or aborted transition rejects these, and a
            // rejection nobody is listening for is an error in the
            // console. Nothing waits on them; they are answered so they
            // are not reported.
            for (const promise of [transition.ready, transition.updateCallbackDone]) {
              if (promise && typeof promise.catch === "function") promise.catch(() => {});
            }
            // Whichever comes first: the transition, or the clock.
            setTimeout(() => {
              if (painted) return;
              try {
                if (transition.skipTransition) transition.skipTransition();
              } catch (e) {
                /* it had already finished or been abandoned */
              }
              done();
            }, millis(motion.duration, 300) + 250);
          } else {
            done();
          }
          return;
        }
        const leaving = [...container.children];
        Promise.all(leaving.map((n) => animateOut(n, out, motion.duration))).then(() => {
          if (currentPath() !== path) return;
          paint(renderFn);
          const arriving = [...container.children];
          Promise.all(arriving.map((n) => animateIn(n, back, motion.duration))).then(settle);
        });
      };

      // A route names its page's render function directly, or names a page
      // that lives in its own chunk and is fetched the first time it shows.
      // A page with a sheet of its own is drawn once the sheet has arrived,
      // so it never paints unstyled.
      const styled = (fn) => (match.route.css ? loadSheet(match.route.css, fn) : fn());
      if (match.route.render) styled(() => draw(match.route.render));
      else loadPage(match.route.page, (renderFn) => styled(() => draw(renderFn)));
    }

    window.addEventListener("popstate", () => {
      fromHistory = true;
      currentPath.set(_stripBase(window.location.pathname));
    });

    effect(render);

    routerInstance = {
      navigate: (to) => {
        // `to: "/?filter=done"`: the query and hash go to the address bar
        // (and the `query`/`hash` values, which follow it); the route is
        // the part before them. A scheme the browser would run never gets
        // that far — the page stays where it is.
        const path = safeUrl(to);
        if (!path) return;
        window.history.pushState(null, "", _basePath + path);
        currentPath.set(_routeOf(path));
      },
      currentPath,
      back: () => window.history.back(),
      forward: () => window.history.forward(),
    };

    return routerInstance;
  }

  function params() {
    return routerInstance ? routerInstance._currentParams || {} : {};
  }

  function settleOnNewPage(container, resetScroll) {
    const heading = container.querySelector && container.querySelector("h1");
    const target = heading || container;
    if (target && target.setAttribute && !target.hasAttribute("tabindex")) {
      target.setAttribute("tabindex", "-1");
    }
    if (target && target.focus) {
      try { target.focus({ preventScroll: true }); } catch (_) { target.focus(); }
    }
    if (resetScroll && typeof window.scrollTo === "function") {
      window.scrollTo(0, 0);
    }
    announce(document.title || "");
  }
