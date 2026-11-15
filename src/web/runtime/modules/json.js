  // ─── JSON and dictionaries ───────────────────────────
  // `Json` is JSON as it arrives: a plain value, matched by what it is or
  // decoded into a declared type. A dictionary is a plain object.

  /// What is wrong with `value` against a declared type's shape — where,
  /// and what — or nothing. A shape is `"s"`, `"n"`, `"b"`, `"m"`, `"j"`
  /// (any JSON), `"*"`, `["?", s]`, `["l", s]`, `["d", s]`,
  /// `["r", name, { field: s }]` or `["e", [cases]]`.
  function shapeFault(value, shape, at) {
    const what = (v) => Array.isArray(v) ? "a list" : v === null ? "null" : typeof v === "object" ? "a map" : JSON.stringify(v);
    if (shape === "*") return null;
    if (Array.isArray(shape) && shape[0] === "?") return value == null ? null : shapeFault(value, shape[1], at);
    if (value === undefined) return { at, message: "is missing" };
    if (shape === "j") return isJson(value) ? null : { at, message: "is " + typeof value + ", which JSON cannot hold" };
    if (value === null) return { at, message: "is null" };
    if (Array.isArray(shape)) {
      const [kind, a, b] = shape;
      if (kind === "l") {
        if (!Array.isArray(value)) return { at, message: "is " + what(value) + ", not a list" };
        for (let i = 0; i < value.length; i++) {
          const wrong = shapeFault(value[i], a, at + "[" + i + "]");
          if (wrong) return wrong;
        }
        return null;
      }
      if (kind === "d") {
        if (typeof value !== "object" || Array.isArray(value)) return { at, message: "is " + what(value) + ", not a dictionary" };
        for (const key of Object.keys(value)) {
          const wrong = shapeFault(value[key], a, at + "[" + JSON.stringify(key) + "]");
          if (wrong) return wrong;
        }
        return null;
      }
      if (kind === "r") {
        if (typeof value !== "object" || Array.isArray(value)) return { at, message: "is " + what(value) + ", not a " + a };
        for (const field of Object.keys(b)) {
          const wrong = shapeFault(value[field], b[field], at + "." + field);
          if (wrong) return wrong;
        }
        return null;
      }
      if (kind === "e") {
        const name = Array.isArray(value) ? value[0] : value;
        return a.includes(name) ? null : { at, message: "is " + JSON.stringify(value) + ", which is none of " + a.map((c) => "." + c).join(", ") };
      }
      return null;
    }
    const kinds = { s: ["string", "String"], n: ["number", "Number"], b: ["boolean", "Bool"], m: ["object", "Map"] };
    const want = kinds[shape];
    if (want && (typeof value !== want[0] || (shape === "m" && Array.isArray(value)))) {
      return { at, message: "is " + what(value) + ", not a " + want[1] };
    }
    return null;
  }
  /// The same, as one line: `$[1].name is missing`.
  function shapeProblem(value, shape, at) {
    const fault = shapeFault(value, shape, at);
    return fault ? fault.at + " " + fault.message : null;
  }
  /// Whether `value` is what JSON can carry.
  function isJson(value) {
    if (value === null) return true;
    switch (typeof value) {
      case "string":
      case "boolean":
        return true;
      case "number":
        return Number.isFinite(value);
      case "object":
        return Array.isArray(value)
          ? value.every(isJson)
          : Object.values(value).every(isJson);
      default:
        return false;
    }
  }

  /// `Settings.decode(raw)`: `["ok", value]` when it is the type,
  /// `["err", { at, message }]` when it is not.
  function decode(shape, value) {
    const fault = shapeFault(value, shape, "$");
    return fault ? ["err", fault] : ["ok", value];
  }

  /// The case of a JSON value: `object`, `array`, `string`, `number`,
  /// `bool` or `null`.
  function jsonCase(value) {
    if (value == null) return "null";
    if (Array.isArray(value)) return "array";
    switch (typeof value) {
      case "object":
        return "object";
      case "string":
        return "string";
      case "number":
        return "number";
      case "boolean":
        return "bool";
      default:
        return "null";
    }
  }
  /// A JSON value as the case it is and what it carries, as an enum's
  /// value is held: `["string", "x"]`, `"null"`.
  function jsonTagged(value) {
    const name = jsonCase(value);
    return name === "null" ? name : [name, value];
  }
  /// What a JSON value carries when it is the case asked for, else nothing.
  function jsonPayload(value, name) {
    return jsonCase(value) === name ? value : null;
  }

  // A dictionary's methods. A receiver with a method of the name — a
  // browser `Map`, a script's object — answers for itself.
  const ownMethod = (d, name) => d != null && typeof d[name] === "function";
  const dictKeys = (d) => (ownMethod(d, "keys") ? d.keys() : Object.keys(d));
  const dictValues = (d) => (ownMethod(d, "values") ? d.values() : Object.values(d));
  const dictEntries = (d) =>
    ownMethod(d, "entries") ? d.entries() : Object.entries(d).map(([key, value]) => ({ key, value }));
  const dictHas = (d, key) =>
    ownMethod(d, "has") ? d.has(key) : Object.prototype.hasOwnProperty.call(d, key);
  function dictWithout(d, key) {
    if (ownMethod(d, "without")) return d.without(key);
    const copy = { ...d };
    delete copy[key];
    return copy;
  }
