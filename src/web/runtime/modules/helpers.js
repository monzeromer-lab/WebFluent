  // ─── Helpers on lists and strings ─────────────────────
  // `items.sortBy(x => x.name)`, `items.groupBy(x => x.kind)`, `unique`,
  // `take(n)`, `first`, `last`; `text.capitalize()`, `text.truncate(n)`.
  //
  // Each takes the receiver first, as a method compiles to. A receiver that
  // declares a method of the name answers for itself, with every argument
  // the call had: a worker's `css.last("diagnostics")` is the worker's, not
  // the last item of a list it is not.
  const declares = (v, name) =>
    v != null && typeof v !== "string" && !Array.isArray(v) && typeof v[name] === "function";
  function sortBy(list, key, ...more) {
    if (declares(list, "sortBy")) return list.sortBy(key, ...more);
    return Array.from(list).sort((a, b) => {
      const ka = key(a), kb = key(b);
      return ka < kb ? -1 : ka > kb ? 1 : 0;
    });
  }
  function groupBy(list, key, ...more) {
    if (declares(list, "groupBy")) return list.groupBy(key, ...more);
    const groups = {};
    for (const item of list) {
      const k = String(key(item));
      (groups[k] || (groups[k] = [])).push(item);
    }
    return groups;
  }
  function unique(list, ...more) {
    if (declares(list, "unique")) return list.unique(...more);
    const seen = new Set();
    const out = [];
    for (const item of list) {
      const k = typeof item === "object" && item !== null ? JSON.stringify(item) : item;
      if (!seen.has(k)) { seen.add(k); out.push(item); }
    }
    return out;
  }
  function take(list, n, ...more) {
    if (declares(list, "take")) return list.take(n, ...more);
    return Array.from(list).slice(0, Math.max(0, n));
  }
  // `items.remove(i)`: the list without the item at `i`, as a new list, so a
  // signal set to it repaints what reads it. An index out of range drops
  // nothing and still hands back a copy.
  function removeAt(list, index) {
    const out = Array.from(list);
    if (index >= 0 && index < out.length) out.splice(index, 1);
    return out;
  }
  // `form.name = v`, `items[i].done = v` on a state: the value with that
  // one place changed, every container on the way copied, so whatever
  // reads the state sees a new value and repaints — and `persist` writes
  // it. Changed in place, the state kept the object it already had, and
  // nothing knew.
  function setIn(target, path, value) {
    if (path.length === 0) return value;
    const [key, ...rest] = path;
    const copy = Array.isArray(target) ? target.slice() : { ...(target || {}) };
    copy[key] = setIn(copy[key], rest, value);
    return copy;
  }
  // `items.sort(f)` as a statement on a state: the method run on a copy,
  // and the copy assigned back.
  function mutated(list, method, args) {
    const copy = Array.from(list || []);
    copy[method](...args);
    return copy;
  }
  // `a..b` and `a..=b`: the whole numbers from `a`, up to `b`.
  function range(a, b, inclusive) {
    const out = [];
    const end = inclusive ? b : b - 1;
    for (let i = a; i <= end; i++) out.push(i);
    return out;
  }
  function first(list, ...more) {
    if (declares(list, "first")) return list.first(...more);
    return list.length ? list[0] : null;
  }
  function last(list, ...more) {
    if (declares(list, "last")) return list.last(...more);
    return list.length ? list[list.length - 1] : null;
  }
  /// The common indentation of a block of text, gone: what a string
  /// carried in from a file or an API keeps, and a reader does not want.
  function dedent(text, ...more) {
    if (declares(text, "dedent")) return text.dedent(...more);
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
  function lines(text, ...more) {
    if (declares(text, "lines")) return text.lines(...more);
    return String(text).split(/\r?\n/);
  }

  /// The words of a text: what is left between runs of whitespace.
  function words(text, ...more) {
    if (declares(text, "words")) return text.words(...more);
    return String(text).trim().split(/\s+/).filter(Boolean);
  }

  function capitalize(text, ...more) {
    if (declares(text, "capitalize")) return text.capitalize(...more);
    const s = String(text);
    return s ? s[0].toUpperCase() + s.slice(1) : s;
  }
  function truncate(text, n, ...more) {
    if (declares(text, "truncate")) return text.truncate(n, ...more);
    const chars = Array.from(String(text));
    return chars.length > n ? chars.slice(0, n).join("") + "\u2026" : chars.join("");
  }
