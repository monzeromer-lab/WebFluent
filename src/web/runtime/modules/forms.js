
  // ─── More form controls ──────────────────────────────
  //
  // Controls the runtime draws whole, each handed its `bind:` as a read and
  // a write: a one-time code, words as chips, stars, a slider with two
  // thumbs, a drop zone for files, a range of days; and what a few flags add
  // to the controls that exist — a textarea that grows, a password that can
  // be shown, a box for all of a group.

  const _opt = (opts, key, fallback) => (opts && typeof opts[key] === "function" ? opts[key]() : fallback);

  /// `InputOTP`: one box a digit; typing moves on, Backspace in an empty
  /// box moves back, a paste fills them all.
  function otp(box, read, write, opts) {
    live(box);
    const count = Number(_opt(opts, "length", 6)) || 6;
    box.setAttribute("role", "group");
    effect(() => box.setAttribute("aria-label", _opt(opts, "label", "Code")));
    const boxes = [];
    const set = () => {
      if (write) write(boxes.map((b) => b.value).join(""));
    };
    for (let i = 0; i < count; i++) {
      const input = el("input", {
        className: "wf-input wf-otp__box",
        type: "text",
        inputmode: "numeric",
        maxlength: "1",
        autocomplete: i === 0 ? "one-time-code" : "off",
        "aria-label": `Digit ${i + 1} of ${count}`,
      });
      input.addEventListener("input", () => {
        const digits = input.value.replace(/\D/g, "");
        if (digits.length > 1) {
          // A code arrived whole (pasted, or offered by the browser).
          digits.slice(0, count - i).split("").forEach((d, k) => { boxes[i + k].value = d; });
          const last = boxes[Math.min(count - 1, i + digits.length - 1)];
          if (last) last.focus();
        } else {
          input.value = digits;
          if (digits && boxes[i + 1]) boxes[i + 1].focus();
        }
        set();
      });
      input.addEventListener("keydown", (e) => {
        if (e.key === "Backspace" && !input.value && boxes[i - 1]) {
          boxes[i - 1].focus();
          boxes[i - 1].value = "";
          set();
          e.preventDefault();
        } else if (e.key === (_isRtl(box) ? "ArrowRight" : "ArrowLeft") && boxes[i - 1]) {
          boxes[i - 1].focus();
        } else if (e.key === (_isRtl(box) ? "ArrowLeft" : "ArrowRight") && boxes[i + 1]) {
          boxes[i + 1].focus();
        }
      });
      input.addEventListener("paste", (e) => {
        const data = (e.clipboardData && e.clipboardData.getData("text")) || "";
        const digits = data.replace(/\D/g, "").slice(0, count);
        if (!digits) return;
        e.preventDefault();
        digits.split("").forEach((d, k) => { if (boxes[k]) boxes[k].value = d; });
        const next = boxes[Math.min(count - 1, digits.length)];
        if (next) next.focus();
        set();
      });
      boxes.push(input);
      box.appendChild(input);
    }
    effect(() => {
      const v = String((read && read()) || "");
      boxes.forEach((b, k) => { if (document.activeElement !== b) b.value = v[k] || ""; });
    });
  }

  /// `TagInput`: Enter or a comma makes what was typed a chip; Backspace in
  /// the empty field takes the last back; each chip has its own button.
  function tagInput(box, read, write, opts) {
    live(box);
    const list = () => (Array.isArray(read && read()) ? read() : []);
    const chips = el("span", { className: "wf-tag-input__chips" });
    const input = el("input", {
      className: "wf-input wf-tag-input__field",
      type: "text",
      "aria-label": () => _opt(opts, "label", ""),
      placeholder: () => _opt(opts, "placeholder", ""),
    });
    box.appendChild(chips);
    box.appendChild(input);
    const add = () => {
      const word = input.value.replace(/,$/, "").trim();
      input.value = "";
      if (!word || !write || list().includes(word)) return;
      write([...list(), word]);
      announce(`${word} added`);
    };
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === ",") {
        e.preventDefault();
        add();
      } else if (e.key === "Backspace" && input.value === "" && list().length && write) {
        const last = list()[list().length - 1];
        write(list().slice(0, -1));
        announce(`${last} removed`);
      }
    });
    input.addEventListener("blur", add);
    effect(() => {
      while (chips.firstChild) chips.removeChild(chips.firstChild);
      for (const word of list()) {
        const remove = el("button", { type: "button", className: "wf-chip__remove", "aria-label": `Remove ${word}` }, "×");
        remove.addEventListener("click", () => {
          if (write) write(list().filter((w) => w !== word));
          input.focus();
        });
        chips.appendChild(el("span", { className: "wf-chip" }, el("span", { className: "wf-chip__label" }, word), remove));
      }
    });
  }

  /// `Rating`: stars as a radio group, one stop for Tab and the arrow keys
  /// between them; `half` takes halves, `readonly` only shows.
  let _ratingSeq = 0;
  function rating(box, read, write, opts) {
    live(box);
    _ratingSeq += 1;
    const max = Number(_opt(opts, "max", 5)) || 5;
    const half = !!_opt(opts, "half", false);
    const readonly = !!_opt(opts, "readonly", false) || !write;
    const label = _opt(opts, "label", "Rating");
    if (readonly) {
      box.setAttribute("role", "img");
      effect(() => box.setAttribute("aria-label", `${label}: ${Number(read ? read() : 0) || 0} out of ${max}`));
    } else {
      box.setAttribute("role", "radiogroup");
      box.setAttribute("aria-label", label);
    }
    const steps = [];
    for (let n = half ? 0.5 : 1; n <= max; n += half ? 0.5 : 1) steps.push(n);
    const stars = [];
    for (let i = 1; i <= max; i++) {
      const star = el("span", { className: "wf-rating__star", "aria-hidden": "true" });
      stars.push(star);
      box.appendChild(star);
    }
    const value = () => Number(read ? read() : 0) || 0;
    effect(() => {
      const v = value();
      stars.forEach((s, i) => s.setAttribute("data-fill", v >= i + 1 ? "full" : v >= i + 0.5 ? "half" : "none"));
    });
    if (readonly) return;
    const radios = steps.map((n) => {
      const r = el("button", {
        type: "button",
        role: "radio",
        className: "wf-rating__choice",
        "aria-label": `${n} of ${max}`,
        "aria-checked": () => (value() === n ? "true" : "false"),
        "data-value": String(n),
      });
      r.addEventListener("click", () => write(n));
      return r;
    });
    const choices = el("span", { className: "wf-rating__choices" }, ...radios);
    box.appendChild(choices);
    roving(choices, { items: () => radios, axis: "both", onMove: (i) => write(steps[i]) });
  }

  /// `Slider.range`: two thumbs, a low and a high, each a native range
  /// input named for its end, neither passing the other.
  function rangeSlider(box, read, write, opts) {
    live(box);
    const min = Number(_opt(opts, "min", 0));
    const max = Number(_opt(opts, "max", 100));
    const step = Number(_opt(opts, "step", 1)) || 1;
    const label = _opt(opts, "label", "Range");
    const pair = () => {
      const v = read ? read() : null;
      return Array.isArray(v) && v.length === 2 ? v.map(Number) : [min, max];
    };
    box.setAttribute("role", "group");
    box.setAttribute("aria-label", label);
    const make = (end) => {
      const input = el("input", { type: "range", className: "wf-slider__input wf-slider__thumb", min: String(min), max: String(max), step: String(step), "aria-label": `${label}, ${end === 0 ? "lowest" : "highest"}` });
      input.addEventListener("input", () => {
        const [lo, hi] = pair();
        const v = Number(input.value);
        if (!write) return;
        write(end === 0 ? [Math.min(v, hi), hi] : [lo, Math.max(v, lo)]);
      });
      return input;
    };
    const low = make(0);
    const high = make(1);
    const shown = el("output", { className: "wf-slider__value" });
    box.appendChild(el("div", { className: "wf-slider__track" }, low, high));
    box.appendChild(shown);
    effect(() => {
      const [lo, hi] = pair();
      low.value = String(lo);
      high.value = String(hi);
      low.setAttribute("aria-valuetext", String(lo));
      high.setAttribute("aria-valuetext", String(hi));
      shown.textContent = `${lo} – ${hi}`;
      box.style.setProperty("--wf-lo", String(((lo - min) / (max - min || 1)) * 100));
      box.style.setProperty("--wf-hi", String(((hi - min) / (max - min || 1)) * 100));
    });
  }

  /// `FileUpload(bind: files).dropzone`: an area files are dropped on or
  /// chosen into, each shown with its preview and a button to take it out.
  function dropzone(box, read, write, opts) {
    live(box);
    const multiple = !!_opt(opts, "multiple", false);
    const files = () => {
      const v = read ? read() : null;
      return multiple ? (Array.isArray(v) ? v : []) : v ? [v] : [];
    };
    const input = el("input", { type: "file", className: "wf-visually-hidden", accept: _opt(opts, "accept", "") || null, multiple: multiple ? "" : null, tabindex: "-1" });
    const pick = el("button", { type: "button", className: "wf-btn wf-dropzone__pick" }, _opt(opts, "label", "Choose files"));
    const zone = el("div", { className: "wf-dropzone__area" }, el("span", { className: "wf-dropzone__hint" }, "Drop files here, or"), pick);
    const list = el("ul", { className: "wf-dropzone__list" });
    box.appendChild(input);
    box.appendChild(zone);
    box.appendChild(list);
    const take = (added) => {
      if (!write || !added.length) return;
      write(multiple ? [...files(), ...added] : added[0]);
      announce(`${added.length} file${added.length === 1 ? "" : "s"} added`);
    };
    pick.addEventListener("click", () => input.click());
    input.addEventListener("change", () => {
      take(Array.from(input.files || []));
      input.value = "";
    });
    zone.addEventListener("dragover", (e) => {
      e.preventDefault();
      zone.setAttribute("data-over", "");
    });
    zone.addEventListener("dragleave", () => zone.removeAttribute("data-over"));
    zone.addEventListener("drop", (e) => {
      e.preventDefault();
      zone.removeAttribute("data-over");
      take(Array.from((e.dataTransfer && e.dataTransfer.files) || []));
    });
    const urls = [];
    onCleanup(() => urls.forEach((u) => URL.revokeObjectURL(u)));
    effect(() => {
      while (list.firstChild) list.removeChild(list.firstChild);
      for (const file of files()) {
        const item = el("li", { className: "wf-dropzone__file" });
        if (file && /^image\//.test(file.type || "") && typeof URL.createObjectURL === "function") {
          const url = URL.createObjectURL(file);
          urls.push(url);
          item.appendChild(el("img", { src: url, alt: "", className: "wf-dropzone__preview" }));
        }
        item.appendChild(el("span", { className: "wf-dropzone__name" }, file ? file.name : ""));
        const remove = el("button", { type: "button", className: "wf-chip__remove", "aria-label": `Remove ${file ? file.name : "file"}` }, "×");
        remove.addEventListener("click", () => {
          if (!write) return;
          write(multiple ? files().filter((f) => f !== file) : null);
        });
        item.appendChild(remove);
        list.appendChild(item);
      }
    });
  }

  /// `DateRangePicker`: a button saying the range, opening two months side
  /// by side and the presets.
  function dateRange(box, read, write, opts) {
    live(box);
    const label = _opt(opts, "label", "Dates");
    const lang = () => document.documentElement.lang || "en";
    const range = () => {
      const v = read ? read() : null;
      return v && typeof v === "object" ? v : { start: null, end: null };
    };
    const open = signal(false);
    const words = (d) => new Intl.DateTimeFormat(lang(), { day: "numeric", month: "short", year: "numeric", timeZone: "UTC" }).format(_day(d));
    const button = el("button", { type: "button", className: "wf-btn wf-date-range__button", "aria-haspopup": "dialog" }, () => {
      const r = range();
      return r.start ? `${label}: ${words(r.start)} – ${r.end ? words(r.end) : "…"}` : label;
    });
    const months = el("div", { className: "wf-date-range__months" });
    const presets = el("div", { className: "wf-date-range__presets", role: "group", "aria-label": "Presets" });
    const panel = el("div", { className: "wf-popover wf-date-range__panel", role: "dialog", "aria-label": label }, presets, months);
    box.appendChild(button);
    box.appendChild(panel);
    button.addEventListener("click", () => open.set(!open()));
    layer(panel, () => open(), (v) => open.set(v), { anchor: () => button, placement: "bottom-start", controls: true });
    const writeRange = (v) => {
      if (write) write(v);
      if (v && v.start && v.end) open.set(false);
    };
    for (let k = 0; k < 2; k++) {
      const cal = el("div", { className: "wf-calendar" });
      months.appendChild(cal);
      calendar(cal, read, writeRange, { mode: "range", offset: k, label: () => `${label}, ${k === 0 ? "first" : "second"} month`, min: opts.min, max: opts.max });
    }
    const today = _iso(new Date(Date.now() - new Date().getTimezoneOffset() * 60000));
    const firstOf = (d) => `${d.slice(0, 7)}-01`;
    const lastOf = (d) => _plus(_plusMonths(firstOf(d), 1), -1);
    const ranges = {
      today: ["Today", { start: today, end: today }],
      last7: ["Last 7 days", { start: _plus(today, -6), end: today }],
      last30: ["Last 30 days", { start: _plus(today, -29), end: today }],
      thisMonth: ["This month", { start: firstOf(today), end: lastOf(today) }],
      lastMonth: ["Last month", { start: firstOf(_plusMonths(today, -1)), end: lastOf(_plusMonths(today, -1)) }],
    };
    for (const name of _opt(opts, "presets", []) || []) {
      const preset = ranges[name];
      if (!preset) continue;
      const b = el("button", { type: "button", className: "wf-btn wf-date-range__preset" }, preset[0]);
      b.addEventListener("click", () => writeRange(preset[1]));
      presets.appendChild(b);
    }
  }

  /// `CheckboxGroup(all:)`: the box for all is mixed while some are ticked.
  function checkboxGroup(box) {
    live(box);
    const sync = () => {
      const all = box.querySelector('[data-all="true"] input, input[data-all="true"]');
      if (!all) return;
      const boxes = Array.from(box.querySelectorAll('input[type="checkbox"]')).filter((i) => i !== all);
      const ticked = boxes.filter((i) => i.checked).length;
      all.indeterminate = ticked > 0 && ticked < boxes.length;
    };
    box.addEventListener("change", () => setTimeout(sync, 0));
    setTimeout(sync, 0);
  }

  /// `Textarea.autosize`: grows with its text. Where the stylesheet's
  /// `field-sizing: content` works it does it; elsewhere this does.
  function autosize(node) {
    live(node);
    const area = node.tagName === "TEXTAREA" ? node : node.querySelector && node.querySelector("textarea");
    if (!area || (typeof CSS !== "undefined" && CSS.supports && CSS.supports("field-sizing", "content"))) return;
    const grow = () => {
      area.style.height = "auto";
      area.style.height = `${area.scrollHeight}px`;
    };
    area.addEventListener("input", grow);
    setTimeout(grow, 0);
  }

  /// `Input.password.reveal`: a button beside the field that shows the
  /// password and hides it again, pressed while it shows.
  function reveal(node) {
    live(node);
    const input = node.tagName === "INPUT" ? node : node.querySelector && node.querySelector("input");
    if (!input) return;
    // Its words say what a press does; pressed, the password shows.
    const button = el("button", { type: "button", className: "wf-reveal", "aria-pressed": "false" }, "Show");
    button.addEventListener("click", () => {
      const showing = input.type === "text";
      input.type = showing ? "password" : "text";
      button.setAttribute("aria-pressed", showing ? "false" : "true");
      button.textContent = showing ? "Show" : "Hide";
      announce(showing ? "Password hidden" : "Password shown");
      input.focus();
    });
    setTimeout(() => {
      if (input.parentNode) input.parentNode.insertBefore(button, input.nextSibling);
    }, 0);
  }
