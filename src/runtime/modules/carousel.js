  function carousel(root, options) {
    const opts = options || {};
    const track = root.querySelector(".wf-carousel__track");
    if (!track) return null;
    const slides = Array.from(track.children).filter(
      (el) => el.classList && el.classList.contains("wf-carousel__slide"),
    );
    const count = slides.length;

    root.setAttribute("role", "region");
    root.setAttribute("aria-roledescription", _label("wf.carousel", "carousel"));
    if (!root.hasAttribute("aria-label") && !root.hasAttribute("aria-labelledby")) {
      root.setAttribute("aria-label", opts.label || _label("wf.carousel", "carousel"));
    }
    slides.forEach((slide, i) => {
      slide.setAttribute("role", "group");
      slide.setAttribute("aria-roledescription", _label("wf.carousel.slide", "slide"));
      if (!slide.hasAttribute("aria-label")) {
        slide.setAttribute("aria-label", `${i + 1} ${_label("wf.carousel.of", "of")} ${count}`);
      }
    });

    const index = signal(0);
    const show = (i) => index.set(((i % count) + count) % count);

    effect(() => {
      const current = index();
      track.style.transform = `translateX(-${current * 100}%)`;
      slides.forEach((slide, i) => {
        const shown = i === current;
        slide.setAttribute("aria-hidden", shown ? "false" : "true");
        if (shown) slide.removeAttribute("inert");
        else slide.setAttribute("inert", "");
      });
    });

    if (count < 2) return { show, index };

    const reduced =
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const autoplay = !!opts.autoplay && !reduced;
    let timer = null;
    let paused = false; // by the reader, with the button
    let pointerOver = false;
    let focused = false;

    const nav = el("div", { className: "wf-carousel__nav" });
    let playButton = null;

    const running = () =>
      autoplay && !paused && !pointerOver && !focused && !(document.hidden === true);

    function sync() {
      if (running()) {
        if (timer === null) {
          timer = setInterval(() => show(index() + 1), opts.interval || 5000);
        }
        track.setAttribute("aria-live", "off");
      } else {
        if (timer !== null) {
          clearInterval(timer);
          timer = null;
        }
        track.setAttribute("aria-live", "polite");
      }
      if (playButton) {
        playButton.setAttribute("aria-pressed", paused ? "true" : "false");
        playButton.setAttribute(
          "aria-label",
          paused
            ? _label("wf.carousel.play", "Start automatic slide rotation")
            : _label("wf.carousel.pause", "Stop automatic slide rotation"),
        );
        playButton.textContent = paused ? "\u25B6" : "\u275A\u275A";
      }
    }

    if (autoplay) {
      playButton = el("button", {
        className: "wf-carousel__control wf-carousel__play",
        type: "button",
        "on:click": () => {
          paused = !paused;
          sync();
        },
      });
      nav.appendChild(playButton);
      root.addEventListener("mouseenter", () => { pointerOver = true; sync(); });
      root.addEventListener("mouseleave", () => { pointerOver = false; sync(); });
      root.addEventListener("focusin", () => { focused = true; sync(); });
      root.addEventListener("focusout", (e) => {
        if (!root.contains(e.relatedTarget)) { focused = false; sync(); }
      });
      document.addEventListener("visibilitychange", sync);
    }

    nav.appendChild(
      el("button", {
        className: "wf-carousel__control wf-carousel__prev",
        type: "button",
        "aria-label": _label("wf.carousel.previous", "Previous slide"),
        "on:click": () => show(index() - 1),
      }, ["\u2039"]),
    );
    const dots = el("div", { className: "wf-carousel__dots" });
    slides.forEach((_, i) => {
      dots.appendChild(
        el("button", {
          className: () => (index() === i ? "wf-carousel__dot active" : "wf-carousel__dot"),
          type: "button",
          "aria-label": `${_label("wf.carousel.goto", "Go to slide")} ${i + 1}`,
          "aria-current": () => (index() === i ? "true" : null),
          "on:click": () => show(i),
        }),
      );
    });
    nav.appendChild(dots);
    nav.appendChild(
      el("button", {
        className: "wf-carousel__control wf-carousel__next",
        type: "button",
        "aria-label": _label("wf.carousel.next", "Next slide"),
        "on:click": () => show(index() + 1),
      }, ["\u203A"]),
    );
    root.appendChild(nav);
    sync();
    return { show, index };
  }

  /// The `<main>` landmark inside `container`, created if it is not there.
  ///
  /// A page needs exactly one main landmark and the skip link needs something to
  /// jump to, whether the page has a router (which supplies its own) or mounts
  /// straight into `#app`. In an SSG build the static paint already contains it,
  /// so this finds that one rather than nesting a second inside it.
