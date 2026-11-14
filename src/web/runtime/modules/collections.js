
  // ─── Collections ─────────────────────────────────────
  //
  // What a list of options, a tree, a table of records, a field that offers
  // matches and a command palette do with the keyboard and with a press.
  // Each keeps one Tab stop, says what is chosen with `aria-selected`, and
  // chooses through the write its `bind:` was compiled to.

  // An element's own words, for typing a letter to it or filtering by it.
  const _words = (n) => (n.getAttribute("aria-label") || n.textContent || "").trim();
  const _keyOf = (key, item) => (typeof key === "function" ? key(item) : item);
  const _sameKey = (a, b) => a === b || (a != null && b != null && JSON.stringify(a) === JSON.stringify(b));

  /// `Listbox`: one stop for Tab, the arrow keys through the options (a
  /// single choice follows them), Space or Enter to choose, and with
  /// `multiple` Shift and an arrow to take the next one in, Ctrl or ⌘ with A
  /// for all. Each option's press is the choosing, which the compiler wrote.
  function listbox(box, multiple) {
    live(box);
    if (multiple) box.setAttribute("aria-multiselectable", "true");
    const options = () => Array.from(box.querySelectorAll('[role="option"]'));
    const usable = (n) => n.getAttribute("aria-disabled") !== "true";
    // A disabled option is shown, never chosen.
    box.addEventListener("click", (e) => {
      const option = e.target.closest ? e.target.closest('[role="option"]') : null;
      if (option && !usable(option)) {
        e.stopImmediatePropagation();
        e.preventDefault();
      }
    }, true);
    roving(box, {
      items: () => options().filter(usable),
      axis: "y",
      typeahead: true,
      onMove: (i, item, e) => {
        if (!multiple) {
          if (item.getAttribute("aria-selected") !== "true") item.click();
        } else if (e && e.shiftKey && item.getAttribute("aria-selected") !== "true") {
          item.click();
        }
      },
    });
    box.addEventListener("keydown", (e) => {
      const at = document.activeElement;
      if ((e.key === " " || e.key === "Enter") && at && at.getAttribute("role") === "option") {
        e.preventDefault();
        if (usable(at)) at.click();
      } else if (multiple && (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") {
        e.preventDefault();
        for (const o of options()) if (usable(o) && o.getAttribute("aria-selected") !== "true") o.click();
      }
    });
  }

  /// `Tree`: the items and, under each open one, its children, as rows of
  /// one list a screen reader hears as a tree (`aria-level`,
  /// `aria-expanded`, `aria-setsize`, `aria-posinset`). Up and Down go
  /// through the rows, Right opens a closed item or goes into an open one,
  /// Left closes an open one or goes to its parent (the other way round in
  /// a right-to-left page), Home and End to the ends, a letter to the next
  /// row it starts; Enter or Space chooses.
  function tree(box, opts) {
    live(box);
    box.setAttribute("role", "tree");
    if (opts.multiple) box.setAttribute("aria-multiselectable", "true");
    effect(() => box.setAttribute("aria-label", opts.label ? opts.label() : "Tree"));
    const kids = (item) => {
      const c = typeof opts.children === "function" ? opts.children(item) : null;
      return Array.isArray(c) ? c : [];
    };
    const keyOf = (item) => _keyOf(opts.key, item);
    const open = signal(new Set((opts.open ? opts.open() : []) || []));
    const focusKey = signal(null);
    // One row object for each key, kept while what it says is the same,
    // so a row that did not change keeps its nodes.
    const cache = new Map();
    const rows = () => {
      const out = [];
      const shown = open();
      const walk = (list, level, parentKey) => {
        list.forEach((item, i) => {
          const key = keyOf(item);
          const children = kids(item);
          const was = cache.get(key);
          const row = was && was.item === item && was.level === level && was.size === list.length && was.pos === i + 1 && was.parent === parentKey && was.branch === (children.length > 0)
            ? was
            : { item, key, level, size: list.length, pos: i + 1, parent: parentKey, branch: children.length > 0 };
          cache.set(key, row);
          out.push(row);
          if (row.branch && shown.has(key)) walk(children, level + 1, key);
        });
      };
      walk((opts.items && opts.items()) || [], 1, null);
      return out;
    };
    const chosen = () => {
      const v = opts.read ? opts.read() : null;
      return opts.multiple ? (Array.isArray(v) ? v : []) : v;
    };
    const isChosen = (key) => (opts.multiple ? chosen().some((k) => _sameKey(k, key)) : _sameKey(chosen(), key));
    const choose = (key, add) => {
      if (!opts.write) return;
      if (!opts.multiple) return opts.write(key);
      const now = chosen();
      if (add) {
        if (!now.some((k) => _sameKey(k, key))) opts.write([...now, key]);
      } else {
        opts.write(now.some((k) => _sameKey(k, key)) ? now.filter((k) => !_sameKey(k, key)) : [...now, key]);
      }
    };
    const toggle = (key, to) => {
      const next = new Set(open());
      if (to === undefined ? next.has(key) : !to) next.delete(key);
      else next.add(key);
      open.set(next);
    };
    each(box, rows, (row) => {
      const content = el("span", { className: "wf-tree__content" });
      const li = el("li", {
        className: "wf-tree__item",
        role: "treeitem",
        "aria-level": String(row.level),
        "aria-setsize": String(row.size),
        "aria-posinset": String(row.pos),
        "aria-expanded": () => (row.branch ? (open().has(row.key) ? "true" : "false") : null),
        "aria-selected": () => (isChosen(row.key) ? "true" : "false"),
        tabindex: () => {
          const at = focusKey();
          const first = rows()[0];
          return at === row.key || (at === null && first && first.key === row.key) ? "0" : "-1";
        },
        "data-level": String(row.level),
      }, content);
      li._wfKey = row.key;
      if (row.branch) {
        const twist = el("span", { className: "wf-tree__twist", "aria-hidden": "true" });
        twist.addEventListener("click", (e) => {
          e.stopPropagation();
          toggle(row.key);
        });
        li.insertBefore(twist, content);
      }
      const drawn = opts.row ? opts.row(row.item) : document.createTextNode(String(row.key));
      content.appendChild(drawn);
      li.addEventListener("click", (e) => {
        e.stopPropagation();
        focusKey.set(row.key);
        li.focus();
        choose(row.key, false);
      });
      li.addEventListener("focus", () => focusKey.set(row.key));
      return li;
    }, { key: (row) => JSON.stringify(row.key) });

    let typed = "";
    let typedAt = 0;
    box.addEventListener("keydown", (e) => {
      if (e.altKey) return;
      const list = rows();
      const items = Array.from(box.querySelectorAll('[role="treeitem"]'));
      const at = items.indexOf(document.activeElement);
      if (at < 0) return;
      const row = list[at];
      const rtl = _isRtl(box);
      const go = (i) => {
        const target = items[Math.max(0, Math.min(items.length - 1, i))];
        if (!target) return;
        focusKey.set(target._wfKey);
        target.focus();
        if (opts.multiple && e.shiftKey) choose(target._wfKey, true);
      };
      const forward = rtl ? "ArrowLeft" : "ArrowRight";
      const back = rtl ? "ArrowRight" : "ArrowLeft";
      if ((e.ctrlKey || e.metaKey) && opts.multiple && e.key.toLowerCase() === "a") {
        e.preventDefault();
        if (opts.write) opts.write(list.map((r) => r.key));
        return;
      }
      if (e.ctrlKey || e.metaKey) return;
      let handled = true;
      switch (e.key) {
        case "ArrowDown": go(at + 1); break;
        case "ArrowUp": go(at - 1); break;
        case "Home": go(0); break;
        case "End": go(items.length - 1); break;
        case forward:
          if (row.branch && !open().has(row.key)) toggle(row.key, true);
          else if (row.branch) go(at + 1);
          break;
        case back:
          if (row.branch && open().has(row.key)) toggle(row.key, false);
          else if (row.parent !== null) go(list.findIndex((r) => _sameKey(r.key, row.parent)));
          break;
        case "Enter":
        case " ":
          choose(row.key, false);
          break;
        default:
          handled = false;
          if (e.key.length === 1 && e.key !== " ") {
            const now = Date.now();
            typed = (now - typedAt < 600 ? typed : "") + e.key.toLowerCase();
            typedAt = now;
            for (let k = 1; k <= items.length; k++) {
              const j = (at + k) % items.length;
              if (_words(items[j]).toLowerCase().startsWith(typed)) {
                go(j);
                handled = true;
                break;
              }
            }
          }
      }
      if (handled) {
        e.preventDefault();
        e.stopPropagation();
      }
    });
  }

  /// `DataTable`: a table of the rows, a header cell per column — a button
  /// when the column sorts, its `aria-sort` saying which way — a cell per
  /// column of each row (its `cell` block, or its `value` as text), and,
  /// with `bind:`, a box to choose each row and one for all of them.
  function dataTable(box, opts) {
    live(box);
    const sort = signal({ at: -1, dir: "ascending" });
    const keyOf = (row) => _keyOf(opts.key, row);
    const chosen = () => {
      const v = opts.read ? opts.read() : null;
      return opts.multiple ? (Array.isArray(v) ? v : []) : v;
    };
    const isChosen = (key) => (opts.multiple ? chosen().some((k) => _sameKey(k, key)) : _sameKey(chosen(), key));
    const picking = !!opts.write;
    const table = el("table", { className: "wf-table wf-data-table__table" });
    const caption = el("caption", { className: "wf-visually-hidden" }, () => (opts.label ? opts.label() : ""));
    table.appendChild(caption);
    const headRow = el("tr", {});
    if (picking) {
      const all = el("input", { type: "checkbox", className: "wf-data-table__pick", "aria-label": "Choose every row" });
      if (opts.multiple) {
        effect(() => {
          const rows = (opts.rows() || []);
          const n = rows.filter((r) => isChosen(keyOf(r))).length;
          all.checked = rows.length > 0 && n === rows.length;
          all.indeterminate = n > 0 && n < rows.length;
        });
        all.addEventListener("change", () => opts.write(all.checked ? (opts.rows() || []).map(keyOf) : []));
        headRow.appendChild(el("th", { scope: "col", className: "wf-data-table__check" }, all));
      } else {
        headRow.appendChild(el("th", { scope: "col", className: "wf-data-table__check" }, el("span", { className: "wf-visually-hidden" }, "Chosen")));
      }
    }
    opts.columns.forEach((col, i) => {
      const th = el("th", {
        scope: "col",
        "aria-sort": () => (col.sortable ? (sort().at === i ? sort().dir : "none") : null),
      });
      if (col.sortable && col.value) {
        const button = el("button", { type: "button", className: "wf-data-table__sort" }, () => col.title());
        button.addEventListener("click", () => {
          const s = sort();
          sort.set({ at: i, dir: s.at === i && s.dir === "ascending" ? "descending" : "ascending" });
        });
        th.appendChild(button);
      } else {
        th.appendChild(text(() => col.title()));
      }
      headRow.appendChild(th);
    });
    table.appendChild(el("thead", {}, headRow));
    const body = el("tbody", {});
    table.appendChild(body);
    box.appendChild(table);

    const sorted = () => {
      const rows = (opts.rows() || []).slice();
      const s = sort();
      const col = opts.columns[s.at];
      if (!col || !col.value) return rows;
      const sign = s.dir === "ascending" ? 1 : -1;
      return rows.sort((a, b) => {
        const x = col.value(a);
        const y = col.value(b);
        if (x == null) return 1;
        if (y == null) return -1;
        return (typeof x === "number" && typeof y === "number" ? x - y : String(x).localeCompare(String(y), undefined, { numeric: true })) * sign;
      });
    };
    each(body, sorted, (row) => {
      const key = keyOf(row);
      const tr = el("tr", { "aria-selected": picking ? () => (isChosen(key) ? "true" : "false") : null });
      if (picking) {
        const pick = el("input", {
          type: opts.multiple ? "checkbox" : "radio",
          className: "wf-data-table__pick",
          name: opts.multiple ? null : "wf-data-table",
          "aria-label": "Choose this row",
        });
        effect(() => { pick.checked = isChosen(key); });
        pick.addEventListener("change", () => {
          if (!opts.multiple) return opts.write(key);
          const now = chosen();
          opts.write(pick.checked ? [...now, key] : now.filter((k) => !_sameKey(k, key)));
        });
        tr.appendChild(el("td", { className: "wf-data-table__check" }, pick));
      }
      for (const col of opts.columns) {
        const td = el("td", {});
        if (col.cell) td.appendChild(col.cell(row));
        else if (col.value) {
          const v = col.value(row);
          td.appendChild(document.createTextNode(v == null ? "" : String(v)));
        }
        tr.appendChild(td);
      }
      return tr;
    }, { key: (row) => JSON.stringify(keyOf(row)) });
    if (opts.empty) {
      const span = opts.columns.length + (picking ? 1 : 0);
      const emptyRow = el("tr", { className: "wf-data-table__empty" }, el("td", { colspan: String(span) }, opts.empty()));
      effect(() => {
        const none = (opts.rows() || []).length === 0;
        if (none && !emptyRow.parentNode) body.appendChild(emptyRow);
        else if (!none && emptyRow.parentNode) emptyRow.remove();
      });
    }
  }

  /// `Combobox`: a field (`role="combobox"`) and the options that match what
  /// is typed in it, in a layer at the field: from `options`, or from
  /// `search(q)` once the typing rests. The arrow keys go through them (the
  /// field keeps the focus, `aria-activedescendant` names the active one),
  /// Enter takes one, Escape closes them. With `multiple` each chosen option
  /// is a chip beside the field, and Backspace in an empty field takes the
  /// last one back.
  let _comboSeq = 0;
  function combobox(box, opts) {
    live(box);
    _comboSeq += 1;
    const id = `wf-combo-${_comboSeq}`;
    const textOf = (o) => {
      if (o == null || o === "") return "";
      const t = typeof opts.text === "function" ? opts.text(o) : o;
      return t == null ? "" : String(t);
    };
    const keyOf = (o) => (opts.key ? opts.key(o) : o);
    // What the options a key named were, so a chosen one shows its words
    // after a search has moved on.
    const known = new Map();
    const remember = (o) => known.set(JSON.stringify(keyOf(o)), o);
    const lookup = (k) => {
      const found = known.get(JSON.stringify(k));
      if (found !== undefined) return found;
      const all = opts.options ? opts.options() || [] : [];
      const o = all.find((x) => _sameKey(keyOf(x), k));
      if (o !== undefined) remember(o);
      // A key no option has (an empty binding) shows nothing.
      return o === undefined ? null : o;
    };
    const chosen = () => {
      const v = opts.read ? opts.read() : null;
      return opts.multiple ? (Array.isArray(v) ? v : []) : v;
    };
    const query = signal("");
    const found = signal([]);
    const busy = signal(false);
    const showing = signal(false);
    const active = signal(-1);

    const chips = el("span", { className: "wf-combobox__chips" });
    const input = el("input", {
      type: "text",
      className: "wf-input wf-combobox__input",
      role: "combobox",
      "aria-autocomplete": "list",
      "aria-controls": `${id}-list`,
      "aria-expanded": () => (showing() ? "true" : "false"),
      "aria-activedescendant": () => (showing() && active() >= 0 ? `${id}-${active()}` : null),
      "aria-busy": () => (busy() ? "true" : null),
      "aria-label": () => (opts.label ? opts.label() : ""),
      placeholder: () => (opts.placeholder ? opts.placeholder() : ""),
      autocomplete: "off",
    });
    const list = el("div", { className: "wf-combobox__list wf-popover", role: "listbox", id: `${id}-list`, "aria-label": () => (opts.label ? opts.label() : "") });
    box.appendChild(chips);
    box.appendChild(input);
    box.appendChild(list);
    layer(list, () => showing(), (v) => showing.set(v), { anchor: () => input, placement: "bottom-start", focus: false });

    const matches = () => {
      if (opts.search) return found();
      const q = query().trim().toLowerCase();
      const all = (opts.options && opts.options()) || [];
      return q ? all.filter((o) => textOf(o).toLowerCase().includes(q)) : all;
    };
    const isChosen = (o) => (opts.multiple ? chosen().some((k) => _sameKey(k, keyOf(o))) : _sameKey(chosen(), keyOf(o)));
    const take = (o) => {
      remember(o);
      if (!opts.write) return;
      if (opts.multiple) {
        const now = chosen();
        opts.write(isChosen(o) ? now.filter((k) => !_sameKey(k, keyOf(o))) : [...now, keyOf(o)]);
        input.value = "";
        query.set("");
      } else {
        opts.write(keyOf(o));
        input.value = textOf(o);
        query.set(input.value);
        showing.set(false);
      }
    };

    // The options shown, drawn again when what matches changes.
    effect(() => {
      const shown = matches();
      while (list.firstChild) list.removeChild(list.firstChild);
      shown.forEach((o, i) => {
        const option = el("div", {
          className: "wf-combobox__option",
          role: "option",
          id: `${id}-${i}`,
          "aria-selected": () => (isChosen(o) ? "true" : "false"),
          "data-active": () => (active() === i ? "" : null),
        }, opts.option ? opts.option(o) : textOf(o));
        // A press keeps the focus in the field.
        option.addEventListener("mousedown", (e) => e.preventDefault());
        option.addEventListener("click", () => take(o));
        list.appendChild(option);
      });
      if (!shown.length && opts.empty) list.appendChild(el("div", { className: "wf-combobox__empty", role: "presentation" }, opts.empty()));
      if (active() >= shown.length) active.set(shown.length ? 0 : -1);
    });

    // The chips of what a `multiple` holds, and the words of a single one.
    effect(() => {
      const v = chosen();
      if (opts.multiple) {
        while (chips.firstChild) chips.removeChild(chips.firstChild);
        for (const k of v) {
          const o = lookup(k);
          const remove = el("button", { type: "button", className: "wf-chip__remove", "aria-label": `Remove ${textOf(o)}` }, "×");
          remove.addEventListener("click", () => opts.write(chosen().filter((x) => !_sameKey(x, k))));
          chips.appendChild(el("span", { className: "wf-chip" }, el("span", { className: "wf-chip__label" }, textOf(o)), remove));
        }
      } else if (document.activeElement !== input) {
        input.value = v == null ? "" : textOf(lookup(v));
      }
    });

    let waiting = 0;
    let asked = 0;
    const ask = () => {
      clearTimeout(waiting);
      const q = query();
      waiting = setTimeout(async () => {
        const mine = ++asked;
        busy.set(true);
        try {
          const result = await opts.search(q);
          if (mine === asked) found.set(Array.isArray(result) ? result : []);
        } catch (e) {
          if (mine === asked) found.set([]);
        } finally {
          if (mine === asked) busy.set(false);
        }
      }, Number(opts.debounce) || 0);
    };
    onCleanup(() => clearTimeout(waiting));
    input.addEventListener("input", () => {
      query.set(input.value);
      showing.set(true);
      active.set(0);
      if (opts.search) ask();
    });
    input.addEventListener("focus", () => {
      if (opts.search && !found().length) ask();
    });
    input.addEventListener("keydown", (e) => {
      const shown = matches();
      if (e.key === "ArrowDown") {
        e.preventDefault();
        if (!showing()) { showing.set(true); active.set(0); return; }
        active.set(Math.min(shown.length - 1, active() + 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        active.set(Math.max(0, active() - 1));
      } else if (e.key === "Enter") {
        if (showing() && shown[active()] !== undefined) {
          e.preventDefault();
          take(shown[active()]);
        }
      } else if (e.key === "Backspace" && opts.multiple && input.value === "" && chosen().length) {
        opts.write(chosen().slice(0, -1));
      }
      const current = list.querySelector("[data-active]");
      if (current && current.scrollIntoView) current.scrollIntoView({ block: "nearest" });
    });
  }

  /// `CommandPalette`: the modal's field filters the items by their words
  /// (and a group with none left goes too); the arrow keys go through what
  /// is left, Enter (or a press) runs the active one — its `on select` — and
  /// closes the palette. The field is emptied each time it closes.
  function commandPalette(root) {
    live(root);
    const dialog = root.querySelector("dialog");
    const input = root.querySelector("input");
    const list = root.querySelector('[role="listbox"]');
    if (!dialog || !input || !list) return;
    if (!list.id) list.id = uid("wf-palette");
    input.setAttribute("role", "combobox");
    input.setAttribute("aria-controls", list.id);
    input.setAttribute("aria-autocomplete", "list");
    input.setAttribute("aria-expanded", "true");
    const items = () => Array.from(list.querySelectorAll('[role="option"]'));
    let active = -1;
    const shown = () => items().filter((n) => !n.hidden);
    const mark = (i) => {
      const list = shown();
      active = list.length ? Math.max(0, Math.min(list.length - 1, i)) : -1;
      for (const n of items()) n.removeAttribute("data-active");
      const at = list[active];
      if (at) {
        if (!at.id) at.id = uid("wf-palette-item");
        at.setAttribute("data-active", "");
        input.setAttribute("aria-activedescendant", at.id);
        if (at.scrollIntoView) at.scrollIntoView({ block: "nearest" });
      } else {
        input.removeAttribute("aria-activedescendant");
      }
    };
    const filter = () => {
      const words = input.value.toLowerCase().split(/\s+/).filter(Boolean);
      for (const n of items()) {
        const t = _words(n).toLowerCase();
        n.hidden = !words.every((w) => t.includes(w));
      }
      for (const g of list.querySelectorAll('[role="group"]')) {
        g.hidden = !g.querySelector('[role="option"]:not([hidden])');
      }
      mark(0);
    };
    const run = (n) => {
      if (!n) return;
      if (typeof CustomEvent === "function") n.dispatchEvent(new CustomEvent("select", { bubbles: false }));
      if (dialog._wfShut) dialog._wfShut();
    };
    input.addEventListener("input", filter);
    input.addEventListener("keydown", (e) => {
      if (e.key === "ArrowDown") { e.preventDefault(); mark(active + 1); }
      else if (e.key === "ArrowUp") { e.preventDefault(); mark(active - 1); }
      else if (e.key === "Home" && e.ctrlKey) { e.preventDefault(); mark(0); }
      else if (e.key === "End" && e.ctrlKey) { e.preventDefault(); mark(shown().length - 1); }
      else if (e.key === "Enter") { e.preventDefault(); run(shown()[active]); }
    });
    list.addEventListener("click", (e) => {
      const n = e.target.closest ? e.target.closest('[role="option"]') : null;
      if (n) run(n);
    });
    list.addEventListener("mousemove", (e) => {
      const n = e.target.closest ? e.target.closest('[role="option"]') : null;
      if (n && !n.hasAttribute("data-active")) mark(shown().indexOf(n));
    });
    dialog.addEventListener("close", () => {
      input.value = "";
      filter();
    });
    filter();
  }
