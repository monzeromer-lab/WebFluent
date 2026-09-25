  // ─── Motion ──────────────────────────────────────────
  //
  // Every animation the engine plays goes through the Web Animations API.
  // What that buys, which a CSS class toggled around `animationend` could
  // not: an animation that is **interrupted** picks up from where it had
  // got to rather than jumping, several animations on one element compose
  // instead of overwriting each other, and `finished` is a promise the
  // runtime can actually wait on — no `setTimeout` guessing at the
  // duration.
  //
  // A reader who asked for less motion gets none, by construction: every
  // entry point here returns before it starts anything.

  /// The keyframes each built-in animation is made of. A project's own
  /// `animation Name { … }` is CSS the compiler wrote, and is played by
  /// its class as it always was.
  const MOTION = {
    fadeIn: [{ opacity: 0 }, { opacity: 1 }],
    fadeOut: [{ opacity: 1 }, { opacity: 0 }],
    slideUp: [{ opacity: 0, transform: "translateY(20px)" }, { opacity: 1, transform: "none" }],
    slideDown: [{ opacity: 0, transform: "translateY(-20px)" }, { opacity: 1, transform: "none" }],
    slideLeft: [{ opacity: 0, transform: "translateX(20px)" }, { opacity: 1, transform: "none" }],
    slideRight: [{ opacity: 0, transform: "translateX(-20px)" }, { opacity: 1, transform: "none" }],
    slideOutLeft: [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "translateX(-20px)" }],
    scaleIn: [{ opacity: 0, transform: "scale(0.9)" }, { opacity: 1, transform: "none" }],
    scaleOut: [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "scale(0.9)" }],
    bounce: [
      { opacity: 0, transform: "scale(0.3)", offset: 0 },
      { transform: "scale(1.05)", offset: 0.5 },
      { transform: "scale(0.9)", offset: 0.7 },
      { opacity: 1, transform: "none", offset: 1 },
    ],
    shake: [
      { transform: "none", offset: 0 },
      { transform: "translateX(-4px)", offset: 0.1 },
      { transform: "translateX(4px)", offset: 0.2 },
      { transform: "translateX(-4px)", offset: 0.3 },
      { transform: "translateX(4px)", offset: 0.4 },
      { transform: "translateX(-4px)", offset: 0.5 },
      { transform: "translateX(4px)", offset: 0.6 },
      { transform: "translateX(-4px)", offset: 0.7 },
      { transform: "translateX(4px)", offset: 0.8 },
      { transform: "translateX(-4px)", offset: 0.9 },
      { transform: "none", offset: 1 },
    ],
    pulse: [
      { transform: "scale(1)", offset: 0 },
      { transform: "scale(1.05)", offset: 0.5 },
      { transform: "scale(1)", offset: 1 },
    ],
    spin: [{ transform: "rotate(0deg)" }, { transform: "rotate(360deg)" }],
  };

  // A reader who asked for less motion gets none: nothing starts, and
  // everything that waits on it is already finished.
  function reducedMotion() {
    return typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
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

  /// A spring, sampled into the `linear()` easing the platform takes.
  ///
  /// A spring is not a cubic Bézier — it overshoots and settles — so it
  /// cannot be written as one. Sampling the solver is how it becomes an
  /// easing function the browser can run on the compositor.
  function spring(stiffness, damping, mass) {
    const k = Number(stiffness) || 180;
    const c = Number(damping) || 20;
    const m = Number(mass) || 1;
    const steps = 60;
    const points = [];
    let x = 1;
    let v = 0;
    const dt = 1 / steps;
    for (let i = 0; i <= steps; i++) {
      points.push(1 - x);
      const a = (-k * x - c * v) / m;
      v += a * dt;
      x += v * dt;
    }
    // The last point is where it settles, which must be exactly the end.
    points[points.length - 1] = 1;
    return `linear(${points.map((p) => p.toFixed(4)).join(", ")})`;
  }

  /// The easing an author named, as CSS writes it.
  function easingOf(easing) {
    if (!easing) return token("--animation-easing-default", "ease");
    if (typeof easing === "object" && easing.spring) {
      return spring(easing.stiffness, easing.damping, easing.mass);
    }
    const said = String(easing);
    // `easing: .spring` compiles to `var(--ease-spring)`, which is a
    // stylesheet's answer, not one the Web Animations API takes.
    const token_ = said.match(/^var\(\s*(--[\w-]+)\s*\)$/);
    if (token_) return token(token_[1], "ease");
    const named = {
      spring: () => spring(180, 20, 1),
      standard: () => token("--ease-standard", "cubic-bezier(0.2, 0, 0, 1)"),
      linear: () => "linear",
    };
    const make = named[said];
    return make ? make() : said;
  }

  /// What a design token says, or `fallback` where there is no stylesheet
  /// to ask — the `motion` config writes these two, so an author sets the
  /// defaults once.
  function token(name, fallback) {
    if (typeof getComputedStyle !== "function" || !document.documentElement) return fallback;
    try {
      const said = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
      return said || fallback;
    } catch (e) {
      return fallback;
    }
  }

  /// The default length of an animation, which a theme's tokens set.
  function defaultDuration() {
    return millis(token("--animation-duration-normal", "300ms"), 300);
  }

  /// Stop every animation the engine started on this element.
  ///
  /// Only ours: a page's own `el.animate(…)` is left alone.
  function cancelOurs(el) {
    if (!el || typeof el.getAnimations !== "function") return;
    for (const running of el.getAnimations()) {
      if (running.id && running.id.startsWith("wf-")) running.cancel();
    }
  }

  /// Play `name` on `el`, and resolve when it has finished.
  ///
  /// An animation already running on the element is cancelled first, so a
  /// card that fades out and comes back does not fight itself — the new
  /// one starts from where the old one had reached, which is what the DOM
  /// already shows.
  function play(el, name, duration, delay, easing) {
    if (!name || reducedMotion() || !el || typeof el.animate !== "function") {
      return Promise.resolve();
    }
    // Opening and closing to the height of the content: CSS cannot
    // animate to `auto`, so this one is measured.
    if (name === "expand") return expand(el, true, duration, easing);
    if (name === "collapse") return expand(el, false, duration, easing);
    const frames = MOTION[name];
    // A project's own `animation Name { … }` is CSS; play it by its class.
    if (!frames) return playClass(el, name, duration, delay, easing);
    cancelOurs(el);
    const ms = millis(duration, defaultDuration());
    const wait = millis(delay, 0);
    const animation = el.animate(frames, {
      duration: ms,
      delay: wait,
      easing: easingOf(easing),
      fill: "both",
      id: "wf-" + name,
    });
    animation.id = "wf-" + name;
    return settled(animation, ms + wait, () => {
      // The end state belongs to the stylesheet, not to the animation.
      if (animation.commitStyles) {
        try {
          animation.commitStyles();
        } catch (e) {
          /* an element already gone */
        }
      }
    });
  }

  /// An animation's `finished`, or the clock — whichever comes first.
  ///
  /// An animation only starts once the page paints a frame, and a page
  /// that is not painting gives it none: a background tab, a prerender, a
  /// headless browser. Whatever is waiting on it — a route change, a
  /// branch being removed, a box opening — must not wait forever for a
  /// piece of decoration, so this settles on time either way, and the
  /// animation is taken off the element when it does.
  function settled(animation, ms, commit) {
    const stop = () => {
      if (commit) commit();
      animation.cancel();
    };
    return Promise.race([
      animation.finished.then(stop, () => {}),
      new Promise((done) => setTimeout(done, ms + 50)).then(() => {
        if (animation.playState !== "idle") stop();
      }),
    ]);
  }

  /// The older path: a class the stylesheet defines, for keyframes the
  /// project declared and the runtime cannot know.
  function playClass(el, name, duration, delay, easing) {
    const cls = "wf-animate-" + name;
    if (duration) el.style.animationDuration = duration;
    if (delay) el.style.animationDelay = delay;
    if (easing) el.style.animationTimingFunction = easingOf(easing);
    el.classList.add(cls);
    return new Promise((resolve) => {
      const done = () => {
        el.classList.remove(cls);
        el.style.animationDuration = "";
        el.style.animationDelay = "";
        el.style.animationTimingFunction = "";
        resolve();
      };
      el.addEventListener("animationend", done, { once: true });
      setTimeout(done, millis(duration, 300) + millis(delay, 0) + 100);
    });
  }

  function animateIn(el, name, duration, delay, easing) {
    return play(el, name, duration, delay, easing);
  }

  function animateOut(el, name, duration, delay, easing) {
    // An element that opened by expanding closes by collapsing: an exit
    // says what it is leaving by, and `.expand` names the pair.
    return play(el, name === "expand" ? "collapse" : name, duration, delay, easing);
  }

  /// Open or close a box to the height of what is inside it.
  ///
  /// CSS cannot animate to `height: auto`; this measures what auto would
  /// be and animates to that number, then hands the height back.
  function expand(el, open, duration, easing) {
    if (!el) return Promise.resolve();
    // The box ends up open or closed **now**, whatever happens next: the
    // animation is how it gets there, not whether. A page that is not
    // painting, a reader who asked for less motion, a browser with no Web
    // Animations API — each of them gets the end state, immediately.
    const settle = () => {
      el.style.height = open ? "" : "0px";
      el.style.overflow = open ? "" : "hidden";
    };
    if (typeof el.animate !== "function" || reducedMotion()) {
      settle();
      return Promise.resolve();
    }
    const from = el.getBoundingClientRect ? el.getBoundingClientRect().height : 0;
    el.style.height = "auto";
    const to = open ? el.scrollHeight : 0;
    el.style.overflow = "hidden";
    settle();
    const ms = millis(duration, defaultDuration());
    // `fill: none`, because the element already holds the end state: a
    // filled animation would pin it at the height it stopped at.
    const animation = el.animate([{ height: `${from}px` }, { height: `${to}px` }], {
      duration: ms,
      easing: easingOf(easing),
      fill: "none",
    });
    return settled(animation, ms);
  }

  /// Count a number from where it is to where it has got to.
  ///
  /// `format` is how the value is written — the same `format` the page
  /// uses — so a currency counts as a currency.
  function countTo(node, from, to, duration, format) {
    const show = (n) => {
      node.textContent = format ? format(n) : String(Math.round(n));
    };
    if (reducedMotion() || typeof requestAnimationFrame !== "function") {
      show(to);
      return Promise.resolve();
    }
    const total = millis(duration, 600);
    const start = Date.now();
    return new Promise((done) => {
      let arrived = false;
      const finish = () => {
        if (arrived) return;
        arrived = true;
        show(to);
        done();
      };
      const tick = () => {
        if (arrived) return;
        const t = Math.min(1, (Date.now() - start) / total);
        // Ease out: a number that counts should slow as it arrives.
        show(from + (to - from) * (1 - Math.pow(1 - t, 3)));
        if (t < 1) requestAnimationFrame(tick);
        else finish();
      };
      // The count runs on frames, and a page that is not painting has
      // none. The number still arrives, on the timer.
      setTimeout(finish, total + 50);
      tick();
    });
  }

  /// A node whose number counts to each new value it is given.
  ///
  /// The first value is written as it is — a page does not open by
  /// counting up from nothing — and every value after it is counted to
  /// from the one before.
  function counted(node, valueFn, duration, format) {
    const write = (n) => {
      node.textContent = format ? format(n) : String(Math.round(n));
    };
    let shown = null;
    effect(() => {
      const to = Number(valueFn());
      if (!Number.isFinite(to)) return;
      if (shown === null) {
        shown = to;
        write(to);
        return;
      }
      const from = shown;
      shown = to;
      countTo(node, from, to, duration, format);
    });
  }

  /// Play `name` on `el` when it first scrolls into view, and not again.
  function onEnterView(el, name, duration, delay, easing) {
    if (!el || typeof IntersectionObserver !== "function") {
      return play(el, name, duration, delay, easing);
    }
    // Until it is seen, it is not there: without this the element would
    // sit at its final state and then animate from it.
    el.style.opacity = "0";
    const watcher = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        watcher.disconnect();
        el.style.opacity = "";
        play(el, name, duration, delay, easing);
      }
    }, { rootMargin: "0px 0px -10% 0px" });
    watcher.observe(el);
    onCleanup(() => watcher.disconnect());
  }

  /// The name this element is known by across a route change, so the
  /// browser can carry it from one page to the next.
  function shared(el, name) {
    if (el && el.style) el.style.viewTransitionName = String(name).replace(/[^\w-]/g, "-");
  }

  // ─── Leaving ─────────────────────────────────────────
  // Every exit the nodes on their way out asked for: the branch's own, on
  // each root, and the one any element beneath carries as `data-wf-exit`.
  // The promises resolve when the last has played; none means remove now.
  function leave(nodes, config) {
    const plays = [];
    if (reducedMotion()) return plays;
    const marked = (el) => {
      if (!el.hasAttribute || !el.hasAttribute("data-wf-exit")) return;
      const attr = (name) => el.getAttribute(name) || "";
      plays.push(animateOut(el, attr("data-wf-exit"), attr("data-wf-duration"), attr("data-wf-delay"), attr("data-wf-easing")));
    };
    for (const n of nodes) {
      if (!(n instanceof Element)) continue;
      if (config && config.exit) plays.push(animateOut(n, config.exit, config.duration, "", config.easing));
      else marked(n);
      if (typeof n.querySelectorAll === "function") {
        for (const el of n.querySelectorAll("[data-wf-exit]")) marked(el);
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

  /// A handle on one animation, for a page that drives it itself.
  ///
  /// It starts when it is made — `WF.animate(ref, "shake")` on its own is
  /// still a statement that shakes something — and `finished` is that
  /// play, so nothing is animated twice for having been asked about.
  function animate(target, name, duration) {
    const el = typeof target === "string" ? document.querySelector(`[data-ref="${target}"]`) : target;
    const handle = {
      /// Play it — again, from the start, if it is already running.
      play: () => {
        handle.finished = play(el, name, duration);
        return handle.finished;
      },
      /// Stop it where it is.
      cancel: () => cancelOurs(el),
      finished: Promise.resolve(),
    };
    if (!el) return handle;
    handle.finished = play(el, name, duration);
    return handle;
  }

  /// `replayAnimation(node, name)`, kept: play it from the start.
  function replay(el, name, duration) {
    if (!el) return Promise.resolve();
    cancelOurs(el);
    return play(el, name, duration);
  }
