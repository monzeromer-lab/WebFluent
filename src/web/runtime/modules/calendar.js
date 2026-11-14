
  // ─── Calendar ────────────────────────────────────────
  //
  // A month as a grid of days, as the WAI-ARIA date grid draws it: one day
  // is the Tab stop; the arrows move a day (the other way round across a
  // right-to-left page) or a week, PageUp and PageDown a month (with Shift,
  // a year), Home and End to the ends of the week, and Enter or Space
  // chooses. Days are ISO dates, `2026-03-14`, as a `Date` is. `mode` is
  // "single" (`bind:` a day), "range" (`{ start, end }`) or "multiple" (a
  // list); `min`, `max` and `disabled` (a list, or a test of a day) say
  // which days cannot be chosen.

  const _iso = (d) => d.toISOString().slice(0, 10);
  const _day = (s) => {
    if (!s || typeof s !== "string") return null;
    const [y, m, d] = s.slice(0, 10).split("-").map(Number);
    if (!y || !m || !d) return null;
    return new Date(Date.UTC(y, m - 1, d));
  };
  const _plus = (s, days) => {
    const d = _day(s);
    d.setUTCDate(d.getUTCDate() + days);
    return _iso(d);
  };
  const _plusMonths = (s, months) => {
    const d = _day(s);
    const day = d.getUTCDate();
    d.setUTCDate(1);
    d.setUTCMonth(d.getUTCMonth() + months);
    const last = new Date(Date.UTC(d.getUTCFullYear(), d.getUTCMonth() + 1, 0)).getUTCDate();
    d.setUTCDate(Math.min(day, last));
    return _iso(d);
  };

  function calendar(box, read, write, options) {
    live(box);
    const opts = options || {};
    const mode = opts.mode || "single";
    const lang = () => (document.documentElement.lang || (typeof navigator !== "undefined" && navigator.language) || "en");
    const today = _iso(new Date(Date.now() - new Date().getTimezoneOffset() * 60000));
    const chosen = () => {
      const v = read();
      if (mode === "range") return v && typeof v === "object" ? v : { start: null, end: null };
      if (mode === "multiple") return Array.isArray(v) ? v : [];
      return v || null;
    };
    const first = () => {
      const v = chosen();
      if (mode === "range") return v.start || today;
      if (mode === "multiple") return v[0] || today;
      return v || today;
    };
    // The day that holds the Tab stop, and so which month shows: `offset`
    // months on, for the second month of a range picker.
    const focus = signal(opts.offset ? _plusMonths(first(), opts.offset) : first());
    const blocked = (day) => {
      const min = opts.min && opts.min();
      const max = opts.max && opts.max();
      if (min && day < String(min).slice(0, 10)) return true;
      if (max && day > String(max).slice(0, 10)) return true;
      const off = opts.disabled && opts.disabled();
      if (Array.isArray(off)) return off.some((d) => String(d).slice(0, 10) === day);
      if (typeof off === "function") return !!off(day);
      return false;
    };
    const isChosen = (day) => {
      const v = chosen();
      if (mode === "range") {
        if (!v.start) return false;
        const end = v.end || v.start;
        return day >= v.start && day <= end;
      }
      if (mode === "multiple") return v.includes(day);
      return v === day;
    };
    const choose = (day) => {
      if (blocked(day) || !write) return;
      const v = chosen();
      if (mode === "range") {
        if (!v.start || v.end) write({ start: day, end: null });
        else if (day < v.start) write({ start: day, end: v.start });
        else write({ start: v.start, end: day });
      } else if (mode === "multiple") {
        write(v.includes(day) ? v.filter((d) => d !== day) : [...v, day].sort());
      } else {
        write(day);
      }
    };
    // Monday or Sunday first, as the reader's language has it.
    const weekStart = () => {
      try {
        const info = new Intl.Locale(lang()).weekInfo || new Intl.Locale(lang()).getWeekInfo?.();
        return info && info.firstDay ? info.firstDay % 7 : 1;
      } catch (e) {
        return 1;
      }
    };

    const prev = el("button", { type: "button", className: "wf-calendar__nav", "aria-label": "Previous month" }, "‹");
    const next = el("button", { type: "button", className: "wf-calendar__nav", "aria-label": "Next month" }, "›");
    const title = el("p", { className: "wf-calendar__title", "aria-live": "polite" });
    const head = el("div", { className: "wf-calendar__head" }, prev, title, next);
    const grid = el("table", { className: "wf-calendar__grid", role: "grid" });
    box.setAttribute("role", "group");
    box.setAttribute("aria-label", (opts.label && opts.label()) || "Calendar");
    box.appendChild(head);
    box.appendChild(grid);
    prev.addEventListener("click", () => focus.set(_plusMonths(focus(), -1)));
    next.addEventListener("click", () => focus.set(_plusMonths(focus(), 1)));

    const draw = () => {
      const at = focus();
      const d = _day(at);
      const year = d.getUTCFullYear();
      const month = d.getUTCMonth();
      const words = new Intl.DateTimeFormat(lang(), { month: "long", year: "numeric", timeZone: "UTC" });
      title.textContent = words.format(d);
      grid.setAttribute("aria-label", words.format(d));
      const hadFocus = grid.contains(document.activeElement);
      while (grid.firstChild) grid.removeChild(grid.firstChild);
      const start = weekStart();
      const weekday = new Intl.DateTimeFormat(lang(), { weekday: "short", timeZone: "UTC" });
      const longDay = new Intl.DateTimeFormat(lang(), { weekday: "long", day: "numeric", month: "long", year: "numeric", timeZone: "UTC" });
      const names = el("tr", {});
      for (let i = 0; i < 7; i++) {
        // 2026-03-01 is a Sunday: its week gives each weekday's name.
        const day = new Date(Date.UTC(2026, 2, 1 + ((start + i) % 7)));
        names.appendChild(el("th", { scope: "col", abbr: new Intl.DateTimeFormat(lang(), { weekday: "long", timeZone: "UTC" }).format(day) }, weekday.format(day)));
      }
      grid.appendChild(el("thead", {}, names));
      const body = el("tbody", {});
      const firstOfMonth = new Date(Date.UTC(year, month, 1));
      const lead = (firstOfMonth.getUTCDay() - start + 7) % 7;
      let cursor = _plus(_iso(firstOfMonth), -lead);
      let focusButton = null;
      for (let w = 0; w < 6; w++) {
        const row = el("tr", {});
        for (let i = 0; i < 7; i++) {
          const day = cursor;
          const inMonth = _day(day).getUTCMonth() === month;
          const off = blocked(day);
          const button = el("button", {
            type: "button",
            className: "wf-calendar__day",
            tabindex: day === at ? "0" : "-1",
            "aria-label": longDay.format(_day(day)),
            "aria-disabled": off ? "true" : null,
            "aria-current": day === today ? "date" : null,
            "data-day": day,
            "data-outside": inMonth ? null : "",
          }, String(_day(day).getUTCDate()));
          button.addEventListener("click", () => {
            focus.set(day);
            choose(day);
          });
          if (day === at) focusButton = button;
          row.appendChild(el("td", { role: "gridcell", "aria-selected": isChosen(day) ? "true" : "false" }, button));
          cursor = _plus(cursor, 1);
        }
        body.appendChild(row);
        if (_day(cursor).getUTCMonth() !== month && w >= 4) break;
      }
      grid.appendChild(body);
      if (hadFocus && focusButton) focusButton.focus();
    };
    effect(() => {
      chosen();
      draw();
    });

    grid.addEventListener("keydown", (e) => {
      if (e.altKey || e.ctrlKey || e.metaKey) return;
      // The day that has the focus, however it got it.
      const at = (e.target && e.target.dataset && e.target.dataset.day) || focus();
      const rtl = _isRtl(box);
      const dow = (_day(at).getUTCDay() - weekStart() + 7) % 7;
      let to = null;
      switch (e.key) {
        case "ArrowRight": to = _plus(at, rtl ? -1 : 1); break;
        case "ArrowLeft": to = _plus(at, rtl ? 1 : -1); break;
        case "ArrowDown": to = _plus(at, 7); break;
        case "ArrowUp": to = _plus(at, -7); break;
        case "PageDown": to = _plusMonths(at, e.shiftKey ? 12 : 1); break;
        case "PageUp": to = _plusMonths(at, e.shiftKey ? -12 : -1); break;
        case "Home": to = _plus(at, -dow); break;
        case "End": to = _plus(at, 6 - dow); break;
        case "Enter":
        case " ":
          e.preventDefault();
          choose(at);
          return;
        default:
          return;
      }
      e.preventDefault();
      focus.set(to);
      const button = grid.querySelector(`[data-day="${to}"]`);
      if (button) button.focus();
    });
  }
