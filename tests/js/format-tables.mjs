//! Writes `src/project/format_locales.rs`: the tables the build formats
//! numbers, money and dates with, read off the `Intl` that the page uses.
//!
//!     TZ=UTC node tests/js/format-tables.mjs > src/project/format_locales.rs
//!
//! The browser formats with `Intl`; the static paint has none, and a table
//! written by hand drifted from it (a German price painted `€1.234,50` and
//! became `1.234,50 €` once the page ran). So the table is ICU's, as Node
//! carries it, and `tests/build_and_runtime_agree.rs` holds the build to
//! the runtime for every locale here — run this again when that test says
//! a newer ICU spells something differently.

const LOCALES = [
  "en", "en-GB", "en-AU", "en-CA", "en-IN",
  "de", "de-AT", "de-CH",
  "fr", "fr-CA", "fr-CH",
  "es", "es-MX", "es-419",
  "it", "pt", "pt-BR", "pt-PT", "nl",
  "ar", "ar-AE", "ar-BH", "ar-DZ", "ar-EG", "ar-IQ", "ar-JO", "ar-KW", "ar-LB", "ar-LY",
  "ar-MA", "ar-OM", "ar-PS", "ar-QA", "ar-SA", "ar-SD", "ar-SY", "ar-TN", "ar-YE",
  "ja", "zh", "zh-TW", "zh-HK", "zh-Hant", "ko",
  "hi", "ru", "uk", "tr", "pl", "cs", "sv", "da", "nb", "fi", "el", "he", "id", "vi",
];
/// Number symbols that follow an `-u-nu-` extension rather than the region.
const NUMBERING = ["ar-u-nu-arab", "ar-u-nu-latn"];
const CURRENCIES = [
  "USD", "EUR", "GBP", "JPY", "CNY", "INR", "KRW", "CAD", "AUD", "BRL", "MXN", "CHF",
  "ILS", "NGN", "PHP", "VND", "SAR", "AED", "EGP", "KWD", "QAR", "MAD", "TRY", "RUB",
];

const esc = (s) =>
  s.replace(/[\\"]/g, (c) => "\\" + c).replace(/[\p{Cc}\p{Cf}\p{Z}]/gu, (c) => c === " " ? c : `\\u{${c.codePointAt(0).toString(16)}}`);
const str = (s) => `"${esc(s)}"`;

/// The pattern a set of parts makes: `#` the number, `¤` the currency,
/// `-` the minus sign; anything else as written.
function shape(parts) {
  let out = "";
  let inNumber = false;
  for (const p of parts) {
    if (["integer", "group", "decimal", "fraction", "compact"].includes(p.type)) {
      if (p.type === "compact") throw new Error("compact in a plain pattern");
      if (!inNumber) out += "#";
      inNumber = true;
      continue;
    }
    inNumber = false;
    if (p.type === "currency") out += "¤";
    else if (p.type === "minusSign") out += "-";
    else if (p.type === "percentSign") out += p.value;
    else if (p.type === "literal") out += p.value;
    else throw new Error("unexpected part " + p.type);
  }
  if (/[#¤-].*[#¤-].*[#¤-].*[#¤-]/.test(out.replace(/[^#¤-]/g, ""))) throw new Error("ambiguous " + out);
  return out;
}

function numbers(locale) {
  const nf = new Intl.NumberFormat(locale);
  const zero = nf.format(0);
  const parts = nf.formatToParts(-12345678.5);
  const val = (t) => parts.find((p) => p.type === t)?.value ?? "";
  const integers = parts.filter((p) => p.type === "integer").map((p) => p.value.length);
  const negative = parts.slice(0, parts.findIndex((p) => p.type === "integer")).map((p) => p.value).join("");
  const minus = val("minusSign");
  const indian = integers.length > 2 && integers[1] === 2;
  const minGrouping = nf.formatToParts(1234).some((p) => p.type === "group") ? 1 : 2;
  const pf = new Intl.NumberFormat(locale, { style: "percent" });
  const percent = shape(pf.formatToParts(0.25));
  const percentNegative = shape(pf.formatToParts(-0.25));
  // The pattern is read off a currency the locale writes as a sign, so
  // no space the symbol's letters would call for is taken for the pattern's.
  const narrow = (code, v) =>
    new Intl.NumberFormat(locale, { style: "currency", currency: code, currencyDisplay: "narrowSymbol" }).formatToParts(v);
  const sign = ["EUR", "USD", "GBP", "JPY"].find(
    (code) => !/\p{L}/u.test(narrow(code, 1).find((p) => p.type === "currency").value),
  );
  if (!sign) throw new Error(`${locale}: no currency written as a sign`);
  const cf = (v) => narrow(sign, v);
  const currency = shape(cf(1234.5));
  const currencyNegative = shape(cf(-1234.5));
  // Some locales set money off with marks of its own (`de-AT`: `€ 1.234,50`
  // beside `1 234,5`).
  const money = cf(-12345678.5);
  const currencyGroup = money.find((p) => p.type === "group").value;
  const currencyDecimal = money.find((p) => p.type === "decimal").value;
  const symbols = [];
  for (const code of CURRENCIES) {
    // The symbol, and the direction marks `Intl` writes after it
    // (`ر.س.‏`).
    const parts = new Intl.NumberFormat(locale, { style: "currency", currency: code }).formatToParts(1);
    const at = parts.findIndex((p) => p.type === "currency");
    let s = parts[at].value;
    for (let i = at + 1; i < parts.length && parts[i].type === "literal" && /^\p{Cf}+$/u.test(parts[i].value); i++) s += parts[i].value;
    if (s !== code) symbols.push([code, s]);
  }
  // The compact notation: for each magnitude, the power it divides by and
  // the pattern for each plural form the language gives it.
  const compactFormat = new Intl.NumberFormat(locale, { notation: "compact" });
  const plural = new Intl.PluralRules(locale);
  const compact = [];
  for (let k = 3; k <= 21; k++) {
    const p = compactFormat.formatToParts(10 ** k);
    if (!p.some((x) => x.type === "compact")) continue;
    const digits = p.filter((x) => x.type === "integer").map((x) => x.value).join("").length;
    const scale = k - (digits - 1);
    const forms = {};
    const lo = 10 ** (k - scale);
    for (const n of [0, 1, 2, 3, 10, 11, 99].map((d) => lo + d).concat([lo * 1.5])) {
      const scaled = n;
      if (scaled < lo || scaled >= lo * 10) continue;
      const parts = compactFormat.formatToParts(scaled * 10 ** scale);
      const cat = plural.select(scaled);
      const pattern = parts.map((x) => (["integer", "group", "decimal", "fraction"].includes(x.type) ? "\u0000" : x.value)).join("").replace(/\u0000+/g, "#");
      if (forms[cat] && forms[cat] !== pattern) throw new Error(`${locale}: two patterns for ${cat} at ${k}`);
      forms[cat] = pattern;
    }
    const other = forms.other ?? forms.many ?? Object.values(forms)[0];
    if (!other) throw new Error(`${locale}: no other form at ${k}`);
    const extra = Object.entries(forms).filter(([c, p]) => c !== "other" && p !== other);
    if (extra.length && !locale.startsWith("ar")) throw new Error(`${locale}: compact plural forms ${JSON.stringify(forms)}`);
    // A magnitude that writes as the one below it adds nothing.
    const last = compact[compact.length - 1];
    if (last && last.scale === scale && last.other === other && JSON.stringify(last.extra) === JSON.stringify(extra)) continue;
    compact.push({ k, scale, other, extra });
  }
  return { zero, group: val("group"), decimal: val("decimal"), minus, negative, indian, minGrouping, percent, percentNegative, currency, currencyNegative, currencyGroup, currencyDecimal, symbols, compact };
}

// ─── Dates ───────────────────────────────────────────
const STYLES = ["short", "medium", "long", "full"];
const day = (m, d = 14, h = 9, min = 5, s = 7) => new Date(Date.UTC(2026, m, d, h, min, s));

function tokens(locale, opts, date, names) {
  const f = new Intl.DateTimeFormat(locale, { ...opts, timeZone: "UTC" });
  const h12 = f.resolvedOptions().hourCycle;
  let out = "";
  let literal = "";
  const flush = () => {
    if (literal) out += /[A-Za-z']/.test(literal) ? "'" + literal.replace(/'/g, "''") + "'" : literal;
    literal = "";
  };
  for (const p of f.formatToParts(date)) {
    if (p.type === "literal" || p.type === "timeZoneName") {
      literal += v8(p.value);
      continue;
    }
    flush();
    switch (p.type) {
      case "year": out += p.value.length === 2 ? "yy" : "yyyy"; break;
      case "month":
        if (/^\p{Nd}+$/u.test(p.value)) out += p.value.length === 2 ? "MM" : "M";
        else {
          // Every month of the year, so a name the long and short forms
          // share (`mars`) does not decide it.
          const all = field(locale, opts, "month", 12, (i) => day(i, 14)).join("|");
          if (names.long && all === names.long.join("|")) out += "MMMM";
          else if (names.short && all === names.short.join("|")) out += "MMM";
          else throw new Error(`${locale}: month ${p.value}`);
        }
        break;
      case "day": out += p.value.length === 2 ? "dd" : "d"; break;
      case "weekday":
        if (p.value !== names.weekdays[date.getUTCDay()]) throw new Error(`${locale}: weekday ${p.value}`);
        out += "EEEE";
        break;
      case "hour": {
        const t = h12 === "h23" ? "H" : h12 === "h12" ? "h" : null;
        if (!t) throw new Error(`${locale}: hour cycle ${h12}`);
        out += p.value.length === 2 ? t + t : t;
        break;
      }
      case "minute": out += p.value.length === 2 ? "mm" : "m"; break;
      case "second": out += p.value.length === 2 ? "ss" : "s"; break;
      case "dayPeriod": out += "a"; break;
      default: throw new Error(`${locale}: part ${p.type}`);
    }
  }
  flush();
  return out;
}

/// V8's `DateTimeFormat.format` writes a narrow no-break space as a plain
/// one (as browsers have since 2023), though `formatToParts` keeps it.
const v8 = (s) => s.replace(/\u202f/g, " ");

/// The value of one field of each month or day, formatted by a style.
function field(locale, opts, type, count, at) {
  const f = new Intl.DateTimeFormat(locale, { ...opts, timeZone: "UTC" });
  return Array.from({ length: count }, (_, i) => v8(f.formatToParts(at(i)).find((p) => p.type === type)?.value));
}

/// What a format of one field alone writes, for each month or day: a
/// pattern's `MMMM` is `Intl.DateTimeFormat(locale, { month: "long" })`.
function alone(locale, opts, count, at) {
  const f = new Intl.DateTimeFormat(locale, { ...opts, timeZone: "UTC" });
  return Array.from({ length: count }, (_, i) => f.format(at(i)));
}

function dates(locale) {
  const calendar = new Intl.DateTimeFormat(locale).resolvedOptions().calendar;
  if (calendar !== "gregory") throw new Error(`${locale}: a ${calendar} calendar, which the build does not reckon`);
  const months = (opts) => field(locale, opts, "month", 12, (i) => day(i, 14));
  // 2026-03-01 is a Sunday.
  const weekdays = (opts) => field(locale, opts, "weekday", 7, (i) => day(2, 1 + i));
  const long = months({ dateStyle: "long" });
  const short = months({ dateStyle: "medium" });
  const names = {
    long: long.every((m) => !/^\p{Nd}+$/u.test(m)) ? long : null,
    short: short.every((m) => !/^\p{Nd}+$/u.test(m)) ? short : null,
    weekdays: weekdays({ dateStyle: "full" }),
  };
  // Patterns are read off 2026-03-04 09:05:07, whose single digits show
  // whether a field is padded.
  const pattern = (opts) => tokens(locale, opts, day(2, 4, 9, 5, 7), names);
  // The day period of each hour: some locales name more than two
  // (`zh-TW`'s 凌晨, 上午, 中午, 下午, 晚上).
  // A pattern's `a` is read off `formatToParts`, as the runtime reads it,
  // so it keeps a narrow space that `format` would not.
  const period = (opts, parts = false) =>
    Array.from({ length: 24 }, (_, h) => {
      const f = new Intl.DateTimeFormat(locale, { ...opts, timeZone: "UTC" });
      const at = (m) => {
        const value = f.formatToParts(day(0, 1, h, m, 0)).find((p) => p.type === "dayPeriod")?.value ?? "";
        return parts ? value : v8(value);
      };
      if (at(0) !== at(30)) throw new Error(`${locale}: the period changes within hour ${h}`);
      return at(0);
    });
  return {
    date: STYLES.map((s) => pattern({ dateStyle: s })),
    time: STYLES.map((s) => pattern({ timeStyle: s })),
    datetime: STYLES.map((s) => pattern({ dateStyle: s, timeStyle: "short" })),
    months: [names.long ?? months({ month: "long" }), names.short ?? months({ month: "short" })],
    weekdays: names.weekdays,
    periods: period({ timeStyle: "short" }),
    standalone: {
      months: [alone(locale, { month: "long" }, 12, (i) => day(i, 14)), alone(locale, { month: "short" }, 12, (i) => day(i, 14))],
      weekdays: [alone(locale, { weekday: "long" }, 7, (i) => day(2, 1 + i)), alone(locale, { weekday: "short" }, 7, (i) => day(2, 1 + i))],
      periods: period({ hour: "numeric", hour12: true }, true),
    },
  };
}

// ─── Emit ────────────────────────────────────────────
const list = (xs) => `[${xs.map(str).join(", ")}]`;
const shared = new Map();
function intern(kind, text) {
  const key = kind + text;
  if (!shared.has(key)) shared.set(key, { name: `${kind}_${shared.size}`, kind, text });
  return shared.get(key).name;
}

function numbersSource(n) {
  const compact = n.compact
    .map((c) => `Compact { magnitude: ${c.k}, scale: ${c.scale}, other: ${str(c.other)}, forms: &[${c.extra.map(([cat, p]) => `(${str(cat)}, ${str(p)})`).join(", ")}] }`)
    .join(",\n        ");
  return `Numbers {
    zero: '${esc(n.zero)}',
    group: ${str(n.group)},
    decimal: ${str(n.decimal)},
    minus: ${str(n.minus)},
    negative: ${str(n.negative)},
    indian: ${n.indian},
    min_grouping: ${n.minGrouping},
    percent: ${str(n.percent)},
    percent_negative: ${str(n.percentNegative)},
    currency: ${str(n.currency)},
    currency_negative: ${str(n.currencyNegative)},
    currency_group: ${str(n.currencyGroup)},
    currency_decimal: ${str(n.currencyDecimal)},
    symbols: &[${n.symbols.map(([c, s]) => `(${str(c)}, ${str(s)})`).join(", ")}],
    compact: &[
        ${compact}
    ],
}`;
}

function datesSource(d) {
  return `Dates {
    date: ${list(d.date)},
    time: ${list(d.time)},
    datetime: ${list(d.datetime)},
    months: [${list(d.months[0])}, ${list(d.months[1])}],
    weekdays: ${list(d.weekdays)},
    periods: ${list(d.periods)},
    standalone_months: [${list(d.standalone.months[0])}, ${list(d.standalone.months[1])}],
    standalone_weekdays: [${list(d.standalone.weekdays[0])}, ${list(d.standalone.weekdays[1])}],
    standalone_periods: ${list(d.standalone.periods)},
}`;
}

const rows = [];
for (const locale of LOCALES) {
  rows.push([locale, intern("NUMBERS", numbersSource(numbers(locale))), intern("DATES", datesSource(dates(locale)))]);
}
const numbering = NUMBERING.map((tag) => [tag, intern("NUMBERS", numbersSource(numbers(tag)))]);
const digits = [...new Set([...CURRENCIES, "BHD", "OMR", "JOD", "TND", "LYD", "IQD", "CLP", "ISK", "UGX", "XOF", "XAF", "KMF", "PYG", "RWF", "DJF", "GNF", "VUV", "XPF", "BIF"])]
  .map((c) => [c, new Intl.NumberFormat("en", { style: "currency", currency: c }).resolvedOptions().maximumFractionDigits])
  .filter(([, n]) => n !== 2);

let out = `// Written by \`tests/js/format-tables.mjs\` from ICU ${process.versions.icu} (Node ${process.versions.node}): do not
// edit by hand — run it again. \`tests/build_and_runtime_agree.rs\` holds the
// build's formatting to the browser's \`Intl\` for these locales.

`;
for (const { name, kind, text } of shared.values()) {
  out += `static ${name}: ${kind === "NUMBERS" ? "Numbers" : "Dates"} = ${text};\n\n`;
}
out += `/// Each locale the table knows, by its tag.\nstatic LOCALES: &[(&str, &Numbers, &Dates)] = &[\n`;
for (const [tag, n, d] of rows) out += `    (${str(tag)}, &${n}, &${d}),\n`;
out += `];\n\n/// The number symbols an \`-u-nu-\` extension picks, by language and system.\nstatic NUMBERING: &[(&str, &Numbers)] = &[\n`;
for (const [tag, n] of numbering) out += `    (${str(tag)}, &${n}),\n`;
out += `];\n\n/// The currencies whose amounts are not written with two places.\nstatic CURRENCY_DIGITS: &[(&str, usize)] = &[\n`;
for (const [c, n] of digits) out += `    (${str(c)}, ${n}),\n`;
out += `];\n`;
process.stdout.write(out);
