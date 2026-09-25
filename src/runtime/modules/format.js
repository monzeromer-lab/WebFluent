  // ─── Formatting ──────────────────────────────────────
  // `format(value, style, option)` and `ago(date)` speak the page's locale:
  // the i18n locale when the project has one — a signal, so a change of
  // locale redraws every formatted text — else the document's language.
  // A style is a case: `.number` (the default), `.integer`, `.decimal`
  // (option: places, 2 by default), `.currency` (option: the code, USD by
  // default), `.percent` (of a fraction; option: places), `.compact`,
  // `.date`, `.time`, `.datetime` (option: `short`/`medium`/`long`/`full`)
  // and `.relative`. A string style is a date pattern: `yyyy-MM-dd HH:mm`.
  function currentLocale() {
    if (i18nInstance) return i18nInstance.locale();
    return document.documentElement.lang || (typeof navigator !== "undefined" && navigator.language) || "en";
  }
  const DATE_STYLES = new Set(["date", "time", "datetime"]);
  function format(value, style, option) {
    const locale = currentLocale();
    if (value == null) return "";
    // Money says what it is, so `format(price)` needs no style and no
    // currency code: it carries both, in minor units.
    if (typeof value === "object" && value && "amount" in value && "currency" in value) {
      return new Intl.NumberFormat(locale, { style: "currency", currency: value.currency })
        .format(Number(value.amount) / 100);
    }
    if (typeof style === "string" && !DATE_STYLES.has(style) && /[yMdEHhmsa]/.test(style) && !/^(number|integer|decimal|currency|percent|compact|relative)$/.test(style)) {
      return formatPattern(toDate(value), style, locale);
    }
    switch (style) {
      case "integer":
        return new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(Number(value));
      case "decimal": {
        const places = option == null ? 2 : Number(option);
        return new Intl.NumberFormat(locale, { minimumFractionDigits: places, maximumFractionDigits: places }).format(Number(value));
      }
      case "currency":
        return new Intl.NumberFormat(locale, { style: "currency", currency: option || "USD" }).format(Number(value));
      case "percent":
        return new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: option == null ? 0 : Number(option) }).format(Number(value));
      case "compact":
        return new Intl.NumberFormat(locale, { notation: "compact" }).format(Number(value));
      case "date":
        return new Intl.DateTimeFormat(locale, { dateStyle: option || "medium" }).format(toDate(value));
      case "time":
        return new Intl.DateTimeFormat(locale, { timeStyle: option || "short" }).format(toDate(value));
      case "datetime":
        return new Intl.DateTimeFormat(locale, { dateStyle: option || "medium", timeStyle: "short" }).format(toDate(value));
      case "relative":
        return ago(value);
      default:
        return new Intl.NumberFormat(locale).format(Number(value));
    }
  }
  function toDate(value) {
    if (value instanceof Date) return value;
    // A bare date is a day, not midnight UTC shifted into the zone.
    if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}$/.test(value)) {
      const [y, m, d] = value.split("-").map(Number);
      return new Date(y, m - 1, d);
    }
    return new Date(value);
  }
  function formatPattern(date, pattern, locale) {
    const names = (opts) => new Intl.DateTimeFormat(locale, opts).format(date);
    const pad = (n, w = 2) => String(n).padStart(w, "0");
    const h12 = date.getHours() % 12 || 12;
    return pattern.replace(/yyyy|yy|MMMM|MMM|MM|M|dd|d|EEEE|EEE|HH|H|hh|h|mm|ss|a/g, (t) => {
      switch (t) {
        case "yyyy": return String(date.getFullYear());
        case "yy": return pad(date.getFullYear() % 100);
        case "MMMM": return names({ month: "long" });
        case "MMM": return names({ month: "short" });
        case "MM": return pad(date.getMonth() + 1);
        case "M": return String(date.getMonth() + 1);
        case "dd": return pad(date.getDate());
        case "d": return String(date.getDate());
        case "EEEE": return names({ weekday: "long" });
        case "EEE": return names({ weekday: "short" });
        case "HH": return pad(date.getHours());
        case "H": return String(date.getHours());
        case "hh": return pad(h12);
        case "h": return String(h12);
        case "mm": return pad(date.getMinutes());
        case "ss": return pad(date.getSeconds());
        case "a": return date.getHours() < 12 ? "AM" : "PM";
        default: return t;
      }
    });
  }
  // `ago(date)`: "3 minutes ago", "yesterday", "in 2 weeks".
  const AGO_UNITS = [["year", 31536000], ["month", 2592000], ["week", 604800], ["day", 86400], ["hour", 3600], ["minute", 60]];
  function ago(value, now) {
    const locale = currentLocale();
    const then = toDate(value).getTime();
    if (then !== then) return ""; // an unreadable date
    const seconds = Math.round((then - (now == null ? Date.now() : toDate(now).getTime())) / 1000);
    const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
    for (const [unit, size] of AGO_UNITS) {
      if (Math.abs(seconds) >= size) return rtf.format(Math.trunc(seconds / size), unit);
    }
    if (Math.abs(seconds) < 45) return rtf.format(0, "second");
    return rtf.format(seconds, "second");
  }

  // `Form(bind: form)`: a handle on the form — `form.valid` (every control
  // passes its own checks), `form.values` (by field name), `form.reset()`,
  // `form.submit()` — kept current as the reader types.
