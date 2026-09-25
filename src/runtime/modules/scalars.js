  // ─── The language's own types ────────────────────────
  //
  // A `Date`, a `Time`, a `DateTime`, a `Duration`, a `Money`, a `Url`, an
  // `Email`, a `Color` — each is a plain JSON value at run time, so it
  // crosses a fetch, a `persist`, the static paint and the template engine
  // unchanged. What the language adds is the arithmetic, and these are the
  // functions that do it: each reads the carrier it is given and gives one
  // back, so nothing has to be wrapped or unwrapped at a boundary.
  //
  // Each takes the receiver first, as a method compiles to. A receiver that
  // has a method of the same name is left to answer for itself, so a record
  // with its own `plus` or `date` is never taken over.
  const own = (v, name) => v != null && typeof v[name] === "function";

  /// What a value is carrying: `date`, `time`, `datetime`, `money`,
  /// `duration`, or `other`.
  function kindOf(value) {
    if (typeof value === "number") return "duration";
    if (value && typeof value === "object" && "amount" in value && "currency" in value) return "money";
    if (value instanceof Date) return "datetime";
    if (typeof value !== "string") return "other";
    if (/^\d{4}-\d{2}-\d{2}$/.test(value)) return "date";
    if (/^\d{2}:\d{2}(:\d{2}(\.\d+)?)?$/.test(value)) return "time";
    if (/^\d{4}-\d{2}-\d{2}T/.test(value)) return "datetime";
    return "other";
  }

  /// A carrier as a platform `Date`, which is what the arithmetic runs on.
  /// A bare time is placed on today, so `.plus(minutes: 30)` still works.
  function native(value) {
    if (own(value, "native")) return value.native();
    const kind = kindOf(value);
    if (value instanceof Date) return new Date(value.getTime());
    if (kind === "date") return new Date(value + "T00:00:00Z");
    if (kind === "time") {
      const today = new Date().toISOString().slice(0, 10);
      return new Date(today + "T" + (value.length === 5 ? value + ":00" : value) + "Z");
    }
    if (kind === "datetime") return new Date(value);
    return new Date(value);
  }

  const pad = (n, w) => String(n).padStart(w || 2, "0");

  /// A platform `Date` back in the carrier of `like`.
  function carry(d, like) {
    switch (kindOf(like)) {
      case "date":
        return `${pad(d.getUTCFullYear(), 4)}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
      case "time":
        return `${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}`;
      default:
        return d.toISOString().replace(/\.\d{3}Z$/, "Z");
    }
  }

  // ─── Reading a date apart ──
  const year = (v) => (own(v, "year") ? v.year() : native(v).getUTCFullYear());
  const month = (v) => (own(v, "month") ? v.month() : native(v).getUTCMonth() + 1);
  const day = (v) => (own(v, "day") ? v.day() : native(v).getUTCDate());
  /// Monday is 1 and Sunday 7, as ISO counts them.
  const weekday = (v) => (own(v, "weekday") ? v.weekday() : native(v).getUTCDay() || 7);
  const hour = (v) => (own(v, "hour") ? v.hour() : native(v).getUTCHours());
  const minute = (v) => (own(v, "minute") ? v.minute() : native(v).getUTCMinutes());
  const second = (v) => (own(v, "second") ? v.second() : native(v).getUTCSeconds());

  /// The date of a moment, and the time of one.
  const dateOf = (v) => (own(v, "date") ? v.date() : carry(native(v), "0000-00-00"));
  const timeOf = (v) => (own(v, "time") ? v.time() : carry(native(v), "00:00"));

  /// `d.plus(days: 3)`, `t.plus(minutes: 30)`, `m.plus(other)`, and a
  /// `Duration` added to any of the three.
  function plus(value, by) {
    if (own(value, "plus")) return value.plus(by);
    if (kindOf(value) === "money") return moneyAdd(value, by, 1);
    if (kindOf(value) === "duration") return value + (typeof by === "number" ? by : 0);
    const d = native(value);
    const parts = typeof by === "number" ? { ms: by } : by || {};
    // Months and years move by the calendar, and land on a day that
    // exists: the 31st plus a month is the 28th, not the 3rd of the next.
    const months = (parts.months || 0) + (parts.years || 0) * 12;
    if (months) {
      const total = d.getUTCFullYear() * 12 + d.getUTCMonth() + months;
      const y = Math.floor(total / 12);
      const m = ((total % 12) + 12) % 12;
      const last = new Date(Date.UTC(y, m + 1, 0)).getUTCDate();
      d.setUTCFullYear(y, m, Math.min(d.getUTCDate(), last));
    }
    const ms =
      (parts.weeks || 0) * 604800000 +
      (parts.days || 0) * 86400000 +
      (parts.hours || 0) * 3600000 +
      (parts.minutes || 0) * 60000 +
      (parts.seconds || 0) * 1000 +
      (parts.ms || 0);
    return carry(new Date(d.getTime() + ms), value);
  }

  /// The same, backwards.
  function minus(value, by) {
    if (own(value, "minus")) return value.minus(by);
    if (kindOf(value) === "money") return moneyAdd(value, by, -1);
    if (kindOf(value) === "duration") return value - (typeof by === "number" ? by : 0);
    const parts = typeof by === "number" ? { ms: by } : by || {};
    const negated = {};
    for (const [k, v] of Object.entries(parts)) negated[k] = -v;
    return plus(value, negated);
  }

  // ─── Comparing ──
  const at = (v) => native(v).getTime();
  const isBefore = (v, other) => (own(v, "isBefore") ? v.isBefore(other) : at(v) < at(other));
  const isAfter = (v, other) => (own(v, "isAfter") ? v.isAfter(other) : at(v) > at(other));
  const isSame = (v, other) => (own(v, "isSame") ? v.isSame(other) : at(v) === at(other));

  /// How long from one moment to another, in milliseconds — a `Duration`.
  const until = (v, other) => (own(v, "until") ? v.until(other) : at(other) - at(v));

  // ─── Rounding down ──
  function startOfDay(v) {
    if (own(v, "startOfDay")) return v.startOfDay();
    const d = native(v);
    d.setUTCHours(0, 0, 0, 0);
    return carry(d, v);
  }
  /// The Monday of its week.
  function startOfWeek(v) {
    if (own(v, "startOfWeek")) return v.startOfWeek();
    const d = native(v);
    d.setUTCDate(d.getUTCDate() - ((d.getUTCDay() || 7) - 1));
    d.setUTCHours(0, 0, 0, 0);
    return carry(d, v);
  }
  function startOfMonth(v) {
    if (own(v, "startOfMonth")) return v.startOfMonth();
    const d = native(v);
    d.setUTCDate(1);
    d.setUTCHours(0, 0, 0, 0);
    return carry(d, v);
  }
  function endOfDay(v) {
    if (own(v, "endOfDay")) return v.endOfDay();
    const d = native(v);
    d.setUTCHours(23, 59, 59, 999);
    return carry(d, v);
  }

  /// The same moment, read in another zone: `stamp.inZone("Europe/Berlin")`.
  function inZone(v, zone) {
    if (own(v, "inZone")) return v.inZone(zone);
    const d = native(v);
    const parts = new Intl.DateTimeFormat("en-CA", {
      timeZone: zone, year: "numeric", month: "2-digit", day: "2-digit",
      hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false,
    }).formatToParts(d);
    const get = (t) => parts.find((p) => p.type === t).value;
    return `${get("year")}-${get("month")}-${get("day")}T${get("hour")}:${get("minute")}:${get("second")}`;
  }

  // ─── A length of time ──
  const days = (v) => (own(v, "days") ? v.days() : Number(v) / 86400000);
  const hours = (v) => (own(v, "hours") ? v.hours() : Number(v) / 3600000);
  const minutes = (v) => (own(v, "minutes") ? v.minutes() : Number(v) / 60000);
  const seconds = (v) => (own(v, "seconds") ? v.seconds() : Number(v) / 1000);
  const ms = (v) => (own(v, "ms") ? v.ms() : Number(v));

  // ─── Money ──
  //
  // The amount is in minor units — cents, pence — so the arithmetic is a
  // whole number's and nothing drifts.
  function money(amount, currency) {
    return { amount: Math.round(amount), currency: currency || "USD" };
  }
  function moneyAdd(a, b, sign) {
    const other = typeof b === "number" ? { amount: b, currency: a.currency } : b;
    if (other.currency !== a.currency) {
      throw new Error(`Cannot add ${other.currency} to ${a.currency}; convert one first`);
    }
    return money(a.amount + sign * other.amount, a.currency);
  }
  const times = (v, n) => (own(v, "times") ? v.times(n) : money(v.amount * Number(n), v.currency));
  /// `price.convert(1.09, "USD")` — the rate is what one unit is worth.
  const convert = (v, rate, currency) =>
    own(v, "convert") ? v.convert(rate, currency) : money(v.amount * Number(rate), currency);

  // ─── A web address ──
  const asUrl = (v) => new URL(String(v), typeof location !== "undefined" ? location.href : "https://x");
  const host = (v) => (own(v, "host") ? v.host() : asUrl(v).host);
  const path = (v) => (own(v, "path") ? v.path() : asUrl(v).pathname);
  function urlQuery(v) {
    if (own(v, "query")) return v.query();
    const out = {};
    asUrl(v).searchParams.forEach((value, key) => { out[key] = value; });
    return out;
  }
  /// `site.with(query: { page: 2 })` — the same URL, that much changed.
  function withParts(v, parts) {
    if (own(v, "with")) return v.with(parts);
    const url = asUrl(v);
    const p = parts || {};
    if (p.path != null) url.pathname = p.path;
    if (p.hash != null) url.hash = p.hash;
    for (const [key, value] of Object.entries(p.query || {})) {
      if (value == null) url.searchParams.delete(key);
      else url.searchParams.set(key, String(value));
    }
    return url.toString();
  }
  /// The part of an address after the `@`.
  const domain = (v) => (own(v, "domain") ? v.domain() : String(v).split("@")[1] || "");

  // ─── Colour ──
  function rgbOf(value) {
    let text = String(value).trim();
    if (text.startsWith("#")) {
      text = text.slice(1);
      if (text.length === 3 || text.length === 4) text = text.split("").map((c) => c + c).join("");
      return [
        parseInt(text.slice(0, 2), 16),
        parseInt(text.slice(2, 4), 16),
        parseInt(text.slice(4, 6), 16),
        text.length === 8 ? parseInt(text.slice(6, 8), 16) / 255 : 1,
      ];
    }
    const nums = text.match(/[\d.]+/g) || [];
    return [Number(nums[0] || 0), Number(nums[1] || 0), Number(nums[2] || 0), nums[3] == null ? 1 : Number(nums[3])];
  }
  // Upper case, as CSS writes a colour and as the compiler paints one.
  const hex = (n) => pad(Math.max(0, Math.min(255, Math.round(n))).toString(16)).toUpperCase();
  /// `brand.mix(ink, 0.2)` — a fifth of the way to the other.
  function mix(value, other, amount) {
    if (own(value, "mix")) return value.mix(other, amount);
    const t = amount == null ? 0.5 : Number(amount);
    const [r1, g1, b1] = rgbOf(value);
    const [r2, g2, b2] = rgbOf(other);
    return "#" + hex(r1 + (r2 - r1) * t) + hex(g1 + (g2 - g1) * t) + hex(b1 + (b2 - b1) * t);
  }
  const lighten = (v, amount) => (own(v, "lighten") ? v.lighten(amount) : mix(v, "#ffffff", amount));
  const darken = (v, amount) => (own(v, "darken") ? v.darken(amount) : mix(v, "#000000", amount));
  /// The same colour, that opaque.
  function alpha(value, a) {
    if (own(value, "alpha")) return value.alpha(a);
    const [r, g, b] = rgbOf(value);
    return `rgba(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)}, ${Number(a)})`;
  }
  function relativeLuminance(value) {
    const [r, g, b] = rgbOf(value);
    const channel = (c) => {
      const s = c / 255;
      return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
    };
    return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
  }
  /// The WCAG contrast ratio between two colours, 1 to 21.
  function contrast(value, other) {
    if (own(value, "contrast")) return value.contrast(other);
    const a = relativeLuminance(value);
    const b = relativeLuminance(other);
    const ratio = (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
    return Math.round(ratio * 100) / 100;
  }

  // ─── A file the reader chose ──
  //
  // The object URL is revoked when the scope that made it leaves, so a
  // preview never outlives the page that showed it.
  function preview(file) {
    if (own(file, "preview")) return file.preview();
    if (typeof URL === "undefined" || !URL.createObjectURL) return "";
    const url = URL.createObjectURL(file);
    onCleanup(() => URL.revokeObjectURL(url));
    return url;
  }

  // ─── A fresh identifier ──
  //
  // `Uuid` is one of the types the language brings with it, and this is
  // where one comes from. The platform's own generator where there is one,
  // and a version-4 layout built from the best randomness available where
  // there is not (an insecure context, an older engine).
  function uuid() {
    const c = typeof crypto !== "undefined" ? crypto : null;
    if (c && c.randomUUID) return c.randomUUID();
    const bytes = new Uint8Array(16);
    if (c && c.getRandomValues) c.getRandomValues(bytes);
    else for (let i = 0; i < 16; i += 1) bytes[i] = Math.floor(Math.random() * 256);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    const hex = [];
    for (let i = 0; i < 16; i += 1) hex.push(bytes[i].toString(16).padStart(2, "0"));
    return (
      hex.slice(0, 4).join("") + "-" + hex.slice(4, 6).join("") + "-" +
      hex.slice(6, 8).join("") + "-" + hex.slice(8, 10).join("") + "-" +
      hex.slice(10, 16).join("")
    );
  }
