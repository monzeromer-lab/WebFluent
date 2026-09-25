  // ─── Helpers on lists and strings ─────────────────────
  // `items.sortBy(x => x.name)`, `items.groupBy(x => x.kind)`, `unique`,
  // `take(n)`, `first`, `last`; `text.capitalize()`, `text.truncate(n)`.
  function sortBy(list, key) {
    return Array.from(list).sort((a, b) => {
      const ka = key(a), kb = key(b);
      return ka < kb ? -1 : ka > kb ? 1 : 0;
    });
  }
  function groupBy(list, key) {
    const groups = {};
    for (const item of list) {
      const k = String(key(item));
      (groups[k] || (groups[k] = [])).push(item);
    }
    return groups;
  }
  function unique(list) {
    const seen = new Set();
    const out = [];
    for (const item of list) {
      const k = typeof item === "object" && item !== null ? JSON.stringify(item) : item;
      if (!seen.has(k)) { seen.add(k); out.push(item); }
    }
    return out;
  }
  function take(list, n) { return Array.from(list).slice(0, Math.max(0, n)); }
  // `items.remove(i)`: the list without the item at `i`, as a new list, so a
  // signal set to it repaints what reads it. An index out of range drops
  // nothing and still hands back a copy.
  function removeAt(list, index) {
    const out = Array.from(list);
    if (index >= 0 && index < out.length) out.splice(index, 1);
    return out;
  }
  // `a..b` and `a..=b`: the whole numbers from `a`, up to `b`.
  function range(a, b, inclusive) {
    const out = [];
    const end = inclusive ? b : b - 1;
    for (let i = a; i <= end; i++) out.push(i);
    return out;
  }
  function first(list) { return list.length ? list[0] : null; }
  function last(list) { return list.length ? list[list.length - 1] : null; }
  /// The common indentation of a block of text, gone: what a string
  /// carried in from a file or an API keeps, and a reader does not want.
  function dedent(text) {
    const rows = String(text).split("\n");
    while (rows.length && !rows[0].trim()) rows.shift();
    while (rows.length && !rows[rows.length - 1].trim()) rows.pop();
    let indent = null;
    for (const row of rows) {
      if (!row.trim()) continue;
      const width = row.length - row.trimStart().length;
      if (indent === null || width < indent) indent = width;
    }
    return rows.map((row) => row.slice(indent || 0)).join("\n");
  }

  /// The lines of a text, however they end.
  function lines(text) {
    return String(text).split(/\r?\n/);
  }

  /// The words of a text: what is left between runs of whitespace.
  function words(text) {
    return String(text).trim().split(/\s+/).filter(Boolean);
  }

  function capitalize(text) {
    const s = String(text);
    return s ? s[0].toUpperCase() + s.slice(1) : s;
  }
  function truncate(text, n) {
    const chars = Array.from(String(text));
    return chars.length > n ? chars.slice(0, n).join("") + "\u2026" : chars.join("");
  }
