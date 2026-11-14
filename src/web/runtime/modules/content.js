
  // ─── What content built-ins do ───────────────────────
  //
  // The built-ins the compiler draws as others (`CopyButton`, `ThemeToggle`,
  // `LocaleSwitch`, `Field`, `InputGroup.Clear`, `OnThisPage`) are made to
  // work here, each from a `mount:` on the element it is drawn as.

  /// A button that copies `read()`: it says `copied` for a moment, and a
  /// screen reader hears it.
  function copyButton(btn, read, label, copied) {
    live(btn);
    let timer = 0;
    btn.addEventListener("click", async () => {
      const value = read();
      const text = value == null ? "" : String(value);
      try {
        await navigator.clipboard.writeText(text);
      } catch (e) {
        // No clipboard API (an insecure origin): the older way.
        const area = document.createElement("textarea");
        area.value = text;
        area.setAttribute("readonly", "");
        area.style.position = "fixed";
        area.style.opacity = "0";
        document.body.appendChild(area);
        area.select();
        try { document.execCommand("copy"); } catch (e2) { /* nothing to copy with */ }
        area.remove();
      }
      btn.textContent = copied;
      btn.setAttribute("data-copied", "");
      announce(copied);
      clearTimeout(timer);
      timer = setTimeout(() => {
        btn.textContent = label;
        btn.removeAttribute("data-copied");
      }, 1600);
    });
    onCleanup(() => clearTimeout(timer));
  }

  /// A button that switches between light and dark: named for the theme it
  /// switches to. Which icon shows is the stylesheet's, so the static paint
  /// is right before this runs.
  function themeToggle(btn) {
    live(btn);
    const media = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
    const systemDark = signal(!!(media && media.matches));
    if (media && media.addEventListener) {
      const heard = (e) => systemDark.set(e.matches);
      media.addEventListener("change", heard);
      onCleanup(() => media.removeEventListener("change", heard));
    }
    const dark = () => {
      const chosen = theme();
      return chosen === "dark" || (chosen !== "light" && systemDark());
    };
    effect(() => {
      btn.setAttribute("aria-label", dark() ? "Switch to the light theme" : "Switch to the dark theme");
    });
    btn.addEventListener("click", () => setTheme(dark() ? "light" : "dark"));
  }

  /// A button for each locale the site speaks, each written in its own
  /// language, the current one pressed.
  function localeSwitch(box) {
    live(box);
    const i18n = typeof i18nInstance !== "undefined" ? i18nInstance : null;
    if (!i18n || !i18n.codes || i18n.codes.length < 2) return;
    const own = (code) => {
      try {
        const name = new Intl.DisplayNames([code], { type: "language" }).of(code);
        return name ? name.charAt(0).toLocaleUpperCase(code) + name.slice(1) : code;
      } catch (e) {
        return code;
      }
    };
    for (const code of i18n.codes) {
      const button = el("button", {
        className: "wf-btn wf-locale-switch__option",
        type: "button",
        lang: code,
        "aria-pressed": () => (String(i18n.locale()) === code ? "true" : "false"),
      }, own(code));
      button.addEventListener("click", () => i18n.setLocale(code));
      box.appendChild(button);
    }
  }

  /// `Field`: the first control in it is named by the label, described by
  /// the hint and the error, and marked invalid while there is an error.
  let _fieldSeq = 0;
  function fieldFor(box, readError) {
    live(box);
    const label = box.querySelector(":scope > .wf-label");
    const hint = box.querySelector(":scope > .wf-field__hint");
    const error = box.querySelector(":scope > .wf-field__error");
    const control = box.querySelector("input:not([type=hidden]), select, textarea, button, [role=combobox], [role=listbox], [role=radiogroup], [role=slider], [role=spinbutton], [role=switch], [role=textbox], [tabindex]:not([tabindex='-1'])");
    if (!control) return;
    _fieldSeq += 1;
    const base = `wf-cfield-${_fieldSeq}`;
    if (!control.id) control.id = `${base}-control`;
    if (label) {
      if (!label.id) label.id = `${base}-label`;
      label.setAttribute("for", control.id);
      if (!control.hasAttribute("aria-labelledby")) control.setAttribute("aria-labelledby", label.id);
      // A label is a press on its control, which a label `for` a `div` is not.
      label.addEventListener("click", () => {
        if (!/^(INPUT|SELECT|TEXTAREA|BUTTON)$/.test(control.tagName) && control.focus) control.focus();
      });
    }
    const described = [];
    if (hint) {
      if (!hint.id) hint.id = `${base}-hint`;
      described.push(hint.id);
    }
    if (error) {
      if (!error.id) error.id = `${base}-error`;
      error.setAttribute("role", "alert");
      described.push(error.id);
    }
    if (described.length) control.setAttribute("aria-describedby", described.join(" "));
    if (readError) {
      effect(() => {
        const e = readError();
        if (e != null && e !== false && e !== "") control.setAttribute("aria-invalid", "true");
        else control.removeAttribute("aria-invalid");
      });
    }
  }

  /// `InputGroup.Clear`: empties the input beside it — as typing would, so
  /// its binding follows — gives it the focus back, and hides while it is
  /// empty.
  function inputClear(btn) {
    live(btn);
    // The group it sits in, read once it is there: a lifetime starts when
    // the element is made, before it is placed.
    const input = () => {
      const group = btn.parentNode;
      return group && group.querySelector ? group.querySelector("input, textarea") : null;
    };
    const sync = () => {
      const field = input();
      if (field && field.value === "") btn.setAttribute("hidden", "");
      else btn.removeAttribute("hidden");
    };
    let heard = null;
    const follow = () => {
      const field = input();
      if (!field || heard === field) return;
      heard = field;
      field.addEventListener("input", sync);
      sync();
    };
    btn.addEventListener("click", () => {
      const field = input();
      if (!field) return;
      field.value = "";
      field.dispatchEvent(new Event("input", { bubbles: true }));
      field.focus();
      sync();
    });
    setTimeout(follow, 0);
  }

  /// `OnThisPage`: a link to each heading of the page (an id given to one
  /// that has none), the one being read marked `aria-current="location"`.
  function onThisPage(nav, levels) {
    live(nav);
    const depth = Math.max(1, Math.min(5, Number(levels) || 1));
    const tags = [];
    for (let i = 0; i < depth; i++) tags.push(`h${i + 2}`);
    const selector = tags.join(", ");
    const list = el("ul", { className: "wf-on-this-page__list" });
    nav.appendChild(list);
    let seen = null;
    const slug = (text) => text.toLowerCase().trim().replace(/[^\p{L}\p{N}]+/gu, "-").replace(/^-|-$/g, "") || "section";
    const draw = () => {
      if (seen) seen.disconnect();
      while (list.firstChild) list.removeChild(list.firstChild);
      const root = document.querySelector("main") || document.body;
      const headings = Array.from(root.querySelectorAll(selector)).filter((h) => !nav.contains(h) && !h.closest("dialog, [hidden]"));
      const used = new Set();
      const links = new Map();
      for (const h of headings) {
        if (!h.id) {
          let id = slug(h.textContent || "");
          let n = 2;
          while (used.has(id) || document.getElementById(id)) id = `${slug(h.textContent || "")}-${n++}`;
          h.id = id;
        }
        used.add(h.id);
        const level = Number(h.tagName.slice(1));
        const a = el("a", { href: `#${encodeURIComponent(h.id)}`, className: "wf-on-this-page__link", "data-level": String(level) }, (h.textContent || "").trim());
        links.set(h, a);
        list.appendChild(el("li", { className: "wf-on-this-page__item" }, a));
      }
      if (!headings.length) nav.setAttribute("hidden", "");
      else nav.removeAttribute("hidden");
      if (typeof IntersectionObserver !== "function") return;
      const showing = new Set();
      const mark = () => {
        const current = headings.find((h) => showing.has(h)) || null;
        for (const [h, a] of links) {
          if (h === current) a.setAttribute("aria-current", "location");
          else a.removeAttribute("aria-current");
        }
      };
      seen = new IntersectionObserver((entries) => {
        for (const e of entries) {
          if (e.isIntersecting) showing.add(e.target);
          else showing.delete(e.target);
        }
        mark();
      }, { rootMargin: "0px 0px -60% 0px" });
      for (const h of headings) seen.observe(h);
    };
    // After the page has drawn what it holds, and again on every route.
    setTimeout(draw, 0);
    listen(document, "wf:render", () => setTimeout(draw, 0));
    onCleanup(() => {
      if (seen) seen.disconnect();
    });
  }
