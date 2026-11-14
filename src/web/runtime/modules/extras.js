  // ─── Tier 3 ──────────────────────────────────────────
  //
  // A colour, two lists to move things between, text that turns into a
  // field, a floating action and its speed dial, a way back to the top, a
  // question before a button acts, and the matches of a search marked.

  const _hex = (v) => {
    const t = String(v == null ? "" : v).trim();
    if (/^#[0-9a-f]{6}$/i.test(t)) return t.toLowerCase();
    if (/^#[0-9a-f]{3}$/i.test(t)) return `#${t[1]}${t[1]}${t[2]}${t[2]}${t[3]}${t[3]}`.toLowerCase();
    return null;
  };
  const _given = (opts, key, fallback) => (opts && typeof opts[key] === "function" ? opts[key]() : fallback);

  /// `ColorPicker`: swatches (toggle buttons), the browser's picker and a
  /// hex field, all three over one `#rrggbb`.
  function colorPicker(box, read, write, opts) {
    live(box);
    const label = () => _given(opts, "label", "Colour");
    box.setAttribute("role", "group");
    effect(() => box.setAttribute("aria-label", label()));
    const set = (v) => {
      const hex = _hex(v);
      if (hex && write) write(hex);
    };
    const swatches = el("div", { className: "wf-color-picker__swatches" });
    const native = el("input", { type: "color", className: "wf-color-picker__native" });
    const field = el("input", { type: "text", className: "wf-input wf-color-picker__hex", spellcheck: "false", maxlength: "7", autocomplete: "off" });
    effect(() => {
      native.setAttribute("aria-label", `${label()}, the picker`);
      field.setAttribute("aria-label", `${label()}, as hex`);
    });
    native.addEventListener("input", () => set(native.value));
    field.addEventListener("change", () => {
      if (_hex(field.value)) set(field.value);
      else field.value = _hex(read()) || "";
    });
    box.append(swatches, native, field);
    effect(() => {
      const list = (_given(opts, "swatches", []) || []).map(_hex).filter(Boolean);
      while (swatches.firstChild) swatches.removeChild(swatches.firstChild);
      for (const c of list) {
        const b = el("button", { type: "button", className: "wf-color-picker__swatch", "aria-label": c, title: c });
        b.style.background = c;
        b.dataset.color = c;
        b.addEventListener("click", () => set(c));
        swatches.appendChild(b);
      }
      swatches.hidden = list.length === 0;
    });
    effect(() => {
      const now = _hex(read()) || "#000000";
      native.value = now;
      if (document.activeElement !== field) field.value = now;
      for (const b of swatches.children) b.setAttribute("aria-pressed", b.dataset.color === now ? "true" : "false");
    });
  }

  /// `TransferList`: two native multiple selects and the buttons that move
  /// the selected (or all) from one to the other; `read()` is the keys of
  /// the chosen, in the order of the options.
  function transferList(box, read, write, opts) {
    live(box);
    const textOf = (o) => {
      const f = _given(opts, "text", null);
      return String(typeof f === "function" ? f(o) : o);
    };
    const keyOf = (o) => {
      const f = _given(opts, "key", null);
      return typeof f === "function" ? f(o) : o;
    };
    const label = () => _given(opts, "label", "Choose");
    box.setAttribute("role", "group");
    effect(() => box.setAttribute("aria-label", label()));
    const side = (cls, heading) => {
      const id = uid("wf-transfer");
      const select = el("select", { className: `wf-transfer-list__list wf-transfer-list__${cls}`, multiple: "", size: "8", "aria-labelledby": id });
      const title = el("p", { className: "wf-transfer-list__title", id });
      effect(() => { title.textContent = heading(); });
      return { select, box: el("div", { className: "wf-transfer-list__side" }, title, select) };
    };
    const from = side("from", () => _given(opts, "from", "Available"));
    const to = side("to", () => _given(opts, "to", "Chosen"));
    const button = (words, aria, fn) => {
      const b = el("button", { type: "button", className: "wf-btn wf-btn--sm", "aria-label": aria }, words);
      b.addEventListener("click", fn);
      return b;
    };
    const options = () => _given(opts, "options", []) || [];
    const chosen = () => {
      const v = read();
      return Array.isArray(v) ? v : [];
    };
    const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
    const picked = (select) => [...select.selectedOptions].map((o) => Number(o.dataset.at));
    const save = (keys) => {
      if (!write) return;
      const order = options().map(keyOf);
      write(order.filter((k) => keys.some((c) => same(c, k))));
    };
    const move = (add) => () => {
      const list = options();
      const at = picked(add ? from.select : to.select);
      if (!at.length) return;
      const keys = at.map((i) => keyOf(list[i]));
      save(add ? chosen().concat(keys) : chosen().filter((c) => !keys.some((k) => same(c, k))));
      announce(`${at.length} moved`);
    };
    const all = (add) => () => save(add ? options().map(keyOf) : []);
    const buttons = el("div", { className: "wf-transfer-list__buttons" },
      button("›", "Move the selected across", move(true)),
      button("»", "Move them all across", all(true)),
      button("‹", "Move the selected back", move(false)),
      button("«", "Move them all back", all(false)));
    box.append(from.box, buttons, to.box);
    effect(() => {
      const list = options();
      const keys = chosen();
      for (const s of [from.select, to.select]) while (s.firstChild) s.removeChild(s.firstChild);
      list.forEach((o, i) => {
        const on = keys.some((k) => same(k, keyOf(o)));
        const option = el("option", {}, textOf(o));
        option.dataset.at = String(i);
        (on ? to.select : from.select).appendChild(option);
      });
    });
  }

  /// `Editable`: a button showing the text; pressed, a field in its place —
  /// Enter or leaving keeps what was typed, Escape puts it back.
  function editable(box, read, write, opts) {
    live(box);
    const shown = el("button", { type: "button", className: "wf-editable__text" });
    const field = el("input", { type: "text", className: "wf-input wf-editable__field" });
    field.hidden = true;
    box.append(shown, field);
    effect(() => {
      const v = read() == null ? "" : String(read());
      const label = _given(opts, "label", "Text");
      shown.textContent = v || _given(opts, "placeholder", "") || "—";
      shown.toggleAttribute("data-empty", !v);
      shown.setAttribute("aria-label", `${label}: ${v || "empty"}, edit`);
      field.setAttribute("aria-label", label);
    });
    let editing = false;
    const end = (keep) => {
      if (!editing) return;
      editing = false;
      if (keep && write && field.value !== String(read() ?? "")) write(field.value);
      field.hidden = true;
      shown.hidden = false;
      shown.focus();
    };
    shown.addEventListener("click", () => {
      editing = true;
      field.value = read() == null ? "" : String(read());
      field.placeholder = _given(opts, "placeholder", "") || "";
      shown.hidden = true;
      field.hidden = false;
      field.focus();
      field.select();
    });
    field.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        end(true);
      } else if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        end(false);
      }
    });
    field.addEventListener("blur", () => end(true));
  }

  /// `Fab`: its button; with actions, a speed dial — its button opens them
  /// (`aria-expanded`), the arrow keys go between them, Escape, a press
  /// outside or a choice closes them.
  function fab(root, dial) {
    live(root);
    const trigger = root.querySelector(".wf-fab__trigger");
    if (!trigger) return;
    trigger.setAttribute("type", "button");
    if (!dial) return;
    const actions = () => [...root.querySelectorAll(".wf-fab__action")];
    for (const a of actions()) a.setAttribute("type", "button");
    trigger.setAttribute("aria-expanded", "false");
    trigger.setAttribute("aria-haspopup", "true");
    const set = (open) => {
      trigger.setAttribute("aria-expanded", open ? "true" : "false");
      root.toggleAttribute("data-open", open);
      for (const a of actions()) a.hidden = !open;
    };
    trigger.addEventListener("click", () => {
      const open = trigger.getAttribute("aria-expanded") !== "true";
      set(open);
      if (open) {
        const list = actions();
        if (list.length) list[list.length - 1].focus();
      }
    });
    root.addEventListener("click", (e) => {
      if (e.target.closest && e.target.closest(".wf-fab__action")) {
        set(false);
        trigger.focus();
      }
    });
    root.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && trigger.getAttribute("aria-expanded") === "true") {
        e.preventDefault();
        set(false);
        trigger.focus();
        return;
      }
      const list = actions();
      const at = list.indexOf(e.target);
      if (at < 0) return;
      if (e.key === "ArrowUp") list[(at - 1 + list.length) % list.length].focus();
      else if (e.key === "ArrowDown") list[(at + 1) % list.length].focus();
      else return;
      e.preventDefault();
    });
    const outside = (e) => {
      if (!root.contains(e.target) && trigger.getAttribute("aria-expanded") === "true") set(false);
    };
    document.addEventListener("pointerdown", outside, true);
    onCleanup(() => document.removeEventListener("pointerdown", outside, true));
  }

  /// `BackToTop`: shown once the page has scrolled past `after` px; a
  /// press scrolls to the top and puts the focus on the page's main.
  function backToTop(button, after) {
    live(button);
    button.setAttribute("type", "button");
    const past = Number(after) || 400;
    let frame = 0;
    const mark = () => {
      frame = 0;
      button.hidden = window.scrollY < past;
    };
    const soon = () => {
      if (!frame) frame = requestAnimationFrame(mark);
    };
    window.addEventListener("scroll", soon, { passive: true });
    onCleanup(() => window.removeEventListener("scroll", soon));
    mark();
    button.addEventListener("click", () => {
      const reduce = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
      window.scrollTo({ top: 0, behavior: reduce ? "auto" : "smooth" });
      const main = document.getElementById("wf-main") || document.querySelector("main") || document.body;
      if (!main.hasAttribute("tabindex")) main.setAttribute("tabindex", "-1");
      main.focus({ preventScroll: true });
    });
  }

  /// `Popconfirm`: the first button in `root` asks before it acts — a
  /// layer at it with the question, no (the focus) and yes; `root` hears
  /// `confirm` or `cancel`.
  function popconfirm(root, title, yes, no) {
    live(root);
    const trigger = root.querySelector("button, [role=button]");
    if (!trigger) return;
    const danger = root.classList.contains("wf-popconfirm--danger");
    const shown = signal(false);
    const id = uid("wf-popconfirm");
    const question = el("p", { className: "wf-popconfirm__title", id });
    const noButton = el("button", { type: "button", className: "wf-btn wf-btn--sm" });
    const yesButton = el("button", { type: "button", className: `wf-btn wf-btn--sm ${danger ? "wf-btn--danger" : "wf-btn--primary"}` });
    const card = el("div", { className: "wf-popconfirm__card", role: "alertdialog", "aria-labelledby": id },
      question, el("div", { className: "wf-popconfirm__buttons" }, noButton, yesButton));
    root.appendChild(card);
    effect(() => {
      question.textContent = title();
      yesButton.textContent = yes();
      noButton.textContent = no();
    });
    const say = (name) => root.dispatchEvent(new CustomEvent(name));
    const close = (answer) => {
      if (!shown()) return;
      shown.set(false);
      trigger.focus();
      say(answer);
    };
    layer(card, shown, (v) => (v ? shown.set(true) : close("cancel")), {
      anchor: trigger,
      placement: "top",
      offset: 8,
      controls: true,
      initial: () => noButton,
    });
    // The trigger asks rather than acts: its own press opens the question.
    trigger.addEventListener("click", (e) => {
      if (card.contains(e.target)) return;
      e.stopImmediatePropagation();
      shown.set(!shown());
    }, true);
    noButton.addEventListener("click", () => close("cancel"));
    yesButton.addEventListener("click", () => close("confirm"));
  }

  /// `Highlight` (`WF.markMatches`): `text()` with every match of `query()` in a `<mark>`,
  /// drawn as text nodes, so nothing in either is markup.
  function markMatches(node, text, query) {
    live(node);
    effect(() => {
      const t = text() == null ? "" : String(text());
      const q = query() == null ? "" : String(query()).trim();
      while (node.firstChild) node.removeChild(node.firstChild);
      if (!q) {
        node.appendChild(document.createTextNode(t));
        return;
      }
      const low = t.toLowerCase();
      const want = q.toLowerCase();
      let at = 0;
      for (let i = low.indexOf(want); i >= 0; i = low.indexOf(want, at)) {
        if (i > at) node.appendChild(document.createTextNode(t.slice(at, i)));
        node.appendChild(el("mark", {}, t.slice(i, i + want.length)));
        at = i + want.length;
      }
      if (at < t.length) node.appendChild(document.createTextNode(t.slice(at)));
    });
  }
