  // ─── More overlays ───────────────────────────────────
  //
  // A card a link previews on hover, a region covered while it is busy,
  // pictures opened full screen, a tour of the page's controls, and the
  // reader's say over the scripts that track them.

  /// `HoverCard`: its `.wf-hover-card__content` in a layer at the first
  /// control in `root`, after `delay` ms of hover or at once on focus; it
  /// stays while the pointer is on it, goes a moment after the pointer
  /// leaves both, and on Escape or a press outside.
  function hoverCard(root, delay) {
    live(root);
    const card = root.querySelector(".wf-hover-card__content");
    if (!card) return;
    card.removeAttribute("hidden");
    const trigger = [...root.querySelectorAll("a[href], button, [tabindex]")].find((n) => !card.contains(n)) || root;
    if (trigger === root && !root.hasAttribute("tabindex")) root.setAttribute("tabindex", "0");
    const wait = delay === undefined || delay === null ? 500 : Number(delay);
    const shown = signal(false);
    let timer = 0;
    const later = (v, ms) => {
      clearTimeout(timer);
      timer = setTimeout(() => shown.set(v), ms);
    };
    layer(card, shown, (v) => {
      clearTimeout(timer);
      shown.set(v);
    }, { anchor: trigger, placement: "bottom", offset: 8, focus: false });
    const enter = () => (shown() ? clearTimeout(timer) : later(true, wait));
    const leave = () => later(false, 300);
    trigger.addEventListener("pointerenter", enter);
    trigger.addEventListener("pointerleave", leave);
    card.addEventListener("pointerenter", () => clearTimeout(timer));
    card.addEventListener("pointerleave", leave);
    trigger.addEventListener("focus", () => later(true, 0));
    trigger.addEventListener("blur", (e) => {
      if (!card.contains(e.relatedTarget)) leave();
    });
    card.addEventListener("focusout", (e) => {
      if (e.relatedTarget !== trigger && !card.contains(e.relatedTarget)) leave();
    });
    onCleanup(() => clearTimeout(timer));
  }

  /// `LoadingOverlay`: while `read()` holds, the region is `aria-busy`, its
  /// content out of reach (`inert`), and its cover shown over it.
  function loadingOverlay(root, read) {
    live(root);
    effect(() => {
      const busy = !!read();
      root.setAttribute("aria-busy", busy ? "true" : "false");
      const content = root.querySelector(".wf-loading-overlay__content");
      if (content) content.inert = busy;
    });
  }

  /// The largest picture an `img` has: `data-full`, else the widest of its
  /// (or its `<picture>`'s) candidates, else what it shows.
  function _largest(img) {
    if (img.dataset && img.dataset.full) return img.dataset.full;
    const sets = [img.getAttribute("srcset")];
    const pic = img.parentElement && img.parentElement.tagName === "PICTURE" ? img.parentElement : null;
    if (pic) for (const s of pic.querySelectorAll("source")) sets.push(s.getAttribute("srcset"));
    let best = null;
    let width = 0;
    for (const set of sets) {
      for (const part of (set || "").split(",")) {
        const [url, size] = part.trim().split(/\s+/);
        const w = parseInt(size, 10) || 0;
        if (url && w >= width) {
          best = url;
          width = w;
        }
      }
    }
    return best || img.currentSrc || img.src;
  }

  /// `Lightbox`: every picture in `root` opens, on a press or Enter, in a
  /// modal at full size: the arrow keys (and a swipe) go between them,
  /// Home and End to the ends, a double press or `+`/`-` zooms, Escape or
  /// the close button shuts it and the focus goes back to the picture.
  function lightbox(root, label) {
    live(root);
    const pictures = () => [...root.querySelectorAll("img")].filter((i) => !i.closest(".wf-lightbox__dialog"));
    const open = signal(false);
    const at = signal(0);
    const zoomed = signal(false);
    const shown = el("img", { className: "wf-lightbox__image", alt: "" });
    const caption = el("p", { className: "wf-lightbox__caption" });
    const count = el("p", { className: "wf-lightbox__count", "aria-live": "polite" });
    // Its glyphs are text: an icon the program never names is not in the
    // build's icon table.
    const button = (cls, words, glyph, fn) => {
      const b = el("button", { type: "button", className: `wf-lightbox__${cls}`, "aria-label": words }, el("span", { "aria-hidden": "true" }, glyph));
      b.addEventListener("click", fn);
      return b;
    };
    const go = (step) => {
      const n = pictures().length;
      if (n) at.set((at() + step + n) % n);
      zoomed.set(false);
    };
    const prev = button("prev", "Previous picture", "\u2039", () => go(-1));
    const next = button("next", "Next picture", "\u203a", () => go(1));
    const zoom = button("zoom", "Zoom", "+", () => zoomed.set(!zoomed()));
    const close = button("close", "Close", "\u00d7", () => open.set(false));
    const stage = el("div", { className: "wf-lightbox__stage" }, shown);
    const dlg = el("dialog", { className: "wf-lightbox__dialog", "aria-label": label || "Pictures" },
      el("div", { className: "wf-lightbox__bar" }, count, zoom, close), stage, prev, next, caption);
    root.appendChild(dlg);
    dialog(dlg, open, (v) => open.set(v), { initial: () => close });
    effect(() => {
      const list = pictures();
      const img = list[at()];
      if (!img) return;
      shown.src = _largest(img);
      shown.alt = img.alt || "";
      caption.textContent = img.alt || "";
      count.textContent = `${at() + 1} of ${list.length}`;
      const one = list.length < 2;
      prev.hidden = one;
      next.hidden = one;
    });
    effect(() => {
      const z = zoomed();
      dlg.toggleAttribute("data-zoomed", z);
      zoom.setAttribute("aria-pressed", z ? "true" : "false");
    });
    const start = (img) => {
      at.set(Math.max(0, pictures().indexOf(img)));
      zoomed.set(false);
      open.set(true);
    };
    const ready = () => {
      for (const img of pictures()) {
        if (img.dataset.wfLightbox) continue;
        img.dataset.wfLightbox = "1";
        img.setAttribute("tabindex", "0");
        img.setAttribute("role", "button");
        img.setAttribute("aria-haspopup", "dialog");
      }
    };
    ready();
    setTimeout(ready, 0);
    root.addEventListener("click", (e) => {
      const img = e.target.closest && e.target.closest("img");
      if (img && pictures().includes(img)) start(img);
    });
    root.addEventListener("keydown", (e) => {
      if (dlg.contains(e.target)) return;
      if ((e.key === "Enter" || e.key === " ") && pictures().includes(e.target)) {
        e.preventDefault();
        start(e.target);
      }
    });
    dlg.addEventListener("keydown", (e) => {
      if (e.altKey || e.ctrlKey || e.metaKey) return;
      const rtl = _isRtl(dlg);
      let used = true;
      if (e.key === (rtl ? "ArrowLeft" : "ArrowRight")) go(1);
      else if (e.key === (rtl ? "ArrowRight" : "ArrowLeft")) go(-1);
      else if (e.key === "Home") { at.set(0); zoomed.set(false); }
      else if (e.key === "End") { at.set(pictures().length - 1); zoomed.set(false); }
      else if (e.key === "+" || e.key === "=") zoomed.set(true);
      else if (e.key === "-") zoomed.set(false);
      else used = false;
      if (used) e.preventDefault();
    });
    shown.addEventListener("dblclick", () => zoomed.set(!zoomed()));
    gesture(stage, "swipe", (s) => {
      if (zoomed()) return;
      const ahead = _isRtl(dlg) ? "right" : "left";
      if (s.direction === ahead) go(1);
      else if (s.direction === "left" || s.direction === "right") go(-1);
    });
  }

  /// `Tour`: while `read()` holds, each `.wf-tour__step` in turn in a layer
  /// at its target, the target lit through a cut-out in a dimmed page;
  /// Back, Next and Done below it, Escape ends it. `root` fires `close`
  /// when it ends, and `finish` when Done ends it.
  function tour(root, read, targets) {
    live(root);
    const steps = () => [...root.querySelectorAll(".wf-tour__step")];
    const at = signal(0);
    const open = signal(false);
    const spot = el("div", { className: "wf-tour__spot", "aria-hidden": "true" });
    const card = el("div", { className: "wf-tour__card", role: "dialog" });
    const count = el("span", { className: "wf-tour__count" });
    const back = el("button", { type: "button", className: "wf-btn wf-btn--sm wf-tour__back" }, "Back");
    const next = el("button", { type: "button", className: "wf-btn wf-btn--primary wf-btn--sm wf-tour__next" }, "Next");
    const skip = el("button", { type: "button", className: "wf-tour__skip", "aria-label": "End the tour" }, el("span", { "aria-hidden": "true" }, "\u00d7"));
    const body = el("div", { className: "wf-tour__body" });
    card.append(skip, body, el("div", { className: "wf-tour__nav" }, count, back, next));
    root.append(spot, card);
    for (const s of steps()) body.appendChild(s);
    const say = (name) => root.dispatchEvent(new CustomEvent(name));
    const end = (done) => {
      open.set(false);
      if (done) say("finish");
      say("close");
    };
    const target = () => {
      const t = (targets[at()] || (() => null))();
      return t && t.el !== undefined ? t.el : t;
    };
    const fit = () => {
      const t = target();
      if (!t || !t.getBoundingClientRect) {
        spot.style.display = "none";
        return;
      }
      const r = t.getBoundingClientRect();
      Object.assign(spot.style, { display: "", left: `${r.left - 6}px`, top: `${r.top - 6}px`, width: `${r.width + 12}px`, height: `${r.height + 12}px` });
      if (handle) handle.place();
    };
    const handle = layer(card, open, (v) => (v ? open.set(true) : end(false)), {
      anchor: () => target() || { x: window.innerWidth / 2, y: window.innerHeight / 3 },
      placement: "bottom",
      offset: 14,
      dismiss: "escape",
    });
    if (typeof spot.showPopover === "function") spot.setAttribute("popover", "manual");
    effect(() => {
      const list = steps();
      const i = at();
      list.forEach((s, n) => { s.hidden = n !== i; });
      const heading = list[i] && list[i].querySelector("h1, h2, h3, h4");
      if (heading) {
        if (!heading.id) heading.id = uid("wf-tour-title");
        card.setAttribute("aria-labelledby", heading.id);
      }
      count.textContent = `${i + 1} of ${list.length}`;
      back.hidden = i === 0;
      next.textContent = i === list.length - 1 ? "Done" : "Next";
      if (open()) {
        const t = target();
        if (t && t.scrollIntoView) t.scrollIntoView({ block: "center", inline: "nearest" });
        requestAnimationFrame(fit);
      }
    });
    effect(() => {
      const want = !!read();
      if (want && !open()) {
        at.set(0);
        if (spot.showPopover) spot.showPopover();
        root.setAttribute("data-open", "");
        open.set(true);
        requestAnimationFrame(() => {
          fit();
          next.focus();
        });
      } else if (!want && open()) {
        open.set(false);
      }
      if (!open()) {
        root.removeAttribute("data-open");
        if (spot.hidePopover && spot.matches(":popover-open")) spot.hidePopover();
      }
    });
    back.addEventListener("click", () => at.set(Math.max(0, at() - 1)));
    next.addEventListener("click", () => {
      if (at() >= steps().length - 1) end(true);
      else at.set(at() + 1);
      next.focus();
    });
    skip.addEventListener("click", () => end(false));
    window.addEventListener("resize", fit);
    window.addEventListener("scroll", fit, true);
    onCleanup(() => {
      window.removeEventListener("resize", fit);
      window.removeEventListener("scroll", fit, true);
    });
  }

  // ─── Consent ─────────────────────────────────────────
  //
  // The scripts `meta.scripts` marks `"load": "consent"` are asked for only
  // once the reader says yes in a `ConsentBanner`, and on every visit after
  // that. Nothing loads until they choose, and no is as easy as yes.

  let _consentList = [];
  const _consentKey = () => `wf-consent${_basePath}`;
  function _consentRead() {
    try {
      return localStorage.getItem(_consentKey());
    } catch {
      return null;
    }
  }

  function consentScripts(list) {
    _consentList = list;
    if (_consentRead() === "granted") loadAfter(list);
  }

  /// `ConsentBanner`: shown until the reader chooses; its
  /// `[data-consent="accept"]` loads the consent scripts and remembers it,
  /// `[data-consent="decline"]` remembers that.
  function consentBanner(root) {
    live(root);
    root.hidden = _consentRead() !== null;
    root.addEventListener("click", (e) => {
      const b = e.target.closest && e.target.closest("[data-consent]");
      if (!b) return;
      const yes = b.getAttribute("data-consent") === "accept";
      try {
        localStorage.setItem(_consentKey(), yes ? "granted" : "denied");
      } catch {
        // A browser that keeps nothing asks again next visit.
      }
      root.hidden = true;
      if (yes) loadAfter(_consentList);
    });
  }
