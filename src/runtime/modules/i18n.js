  // ─── i18n ────────────────────────────────────────────
  const RTL_LOCALES = new Set(["ar", "he", "fa", "ur"]);

  function locales(defaultLocale, translations) {
    const locale = signal(defaultLocale);
    const dir = signal(RTL_LOCALES.has(defaultLocale) ? "rtl" : "ltr");

    function t(key, params) {
      const currentLocale = locale();
      const messages = translations[currentLocale] || translations[defaultLocale] || {};
      const fallback = translations[defaultLocale] || {};
      const lookup = (k) => (messages[k] !== undefined ? messages[k] : fallback[k]);
      let text;
      // A `count` picks the plural form: `key.one`, `key.other`, and the
      // locale's other categories when the file has them.
      if (params && typeof params.count === "number" && typeof Intl !== "undefined" && Intl.PluralRules) {
        let category = "other";
        try { category = new Intl.PluralRules(currentLocale).select(params.count); } catch (e) { /* unknown locale */ }
        text = lookup(key + "." + category);
        if (text === undefined) text = lookup(key + ".other");
      }
      if (text === undefined) text = lookup(key);
      // Fallback to key itself
      if (text === undefined) return key;
      // Interpolate {placeholder} tokens
      if (params && text.includes("{")) {
        for (const [k, v] of Object.entries(params)) {
          text = text.replace(new RegExp("\\{" + k + "\\}", "g"), String(v));
        }
      }
      return text;
    }

    function setLocale(newLocale) {
      locale.set(newLocale);
      const newDir = RTL_LOCALES.has(newLocale) ? "rtl" : "ltr";
      dir.set(newDir);
      document.documentElement.setAttribute("lang", newLocale);
      document.documentElement.setAttribute("dir", newDir);
    }

    i18nInstance = { t, locale, dir, setLocale };
    return i18nInstance;
  }
